use core::cell::RefCell;

use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::renderer::dom::node::{Element, ElementKind, Namespace, Node, NodeData, Window};

use super::{
    attribute::Attribute,
    token::{self, HtmlToken, HtmlTokenizer},
};

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Rc<RefCell<Window>>,
    mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    t: HtmlTokenizer,
}
impl HtmlParser {
    pub fn new(t: HtmlTokenizer) -> Self {
        Self {
            window: Window::new(),
            mode: InsertionMode::Initial,
            original_insertion_mode: InsertionMode::Initial,
            stack_of_open_elements: Vec::new(),
            t,
        }
    }

    pub fn construct_tree(&mut self) -> Rc<RefCell<Window>> {
        let mut maybe_token = self.t.next();
        while let Some(token) = &maybe_token {
            let output = self.step(token);
            if let Some(state) = output.set_tokenizer_state {
                self.t.set_state(state);
            }
            if !output.reprocess {
                maybe_token = self.t.next();
            }
            if output.stop {
                break;
            }
        }
        self.window.clone()
    }

    fn step(&mut self, token: &HtmlToken) -> StepOutput {
        let adjusted_current_node_namespace = self.adjusted_current_node().and_then(|node| {
            // TODO: more cases?
            if let NodeData::Element(e) = &node.borrow().data {
                Some(e.kind.namespace())
            } else {
                None
            }
        });
        if self.stack_of_open_elements.is_empty()
            || adjusted_current_node_namespace == Some(Namespace::Html)
            || token == &HtmlToken::Eof
        {
            self.process_token_based_on_mode(token)
        } else {
            self.process_token_in_foreign_content(token)
        }
    }

    fn process_token_based_on_mode(&mut self, token: &HtmlToken) -> StepOutput {
        match self.mode {
            InsertionMode::Initial => match token {
                HtmlToken::Char('\t' | '\n' | '\x0c' | ' ') => StepOutput::default(),
                HtmlToken::DoctypeTag { .. } => {
                    self.mode = InsertionMode::BeforeHtml;
                    StepOutput::default()
                }
                _ => {
                    self.mode = InsertionMode::BeforeHtml;
                    StepOutput {
                        reprocess: true,
                        ..Default::default()
                    }
                }
            },
            InsertionMode::BeforeHtml => {
                match token {
                    HtmlToken::StartTag { tag, .. } if tag == "html" => {
                        let element =
                            self.create_element_for_token(token, Namespace::Html, self.document());
                        Node::append_child(self.document(), Rc::clone(&element));
                        self.stack_of_open_elements.push(Rc::clone(&element));
                        self.mode = InsertionMode::BeforeHead;
                        StepOutput::default()
                    }
                    HtmlToken::EndTag { tag: _ } => StepOutput::default(),
                    _ => {
                        // TODO
                        self.mode = InsertionMode::BeforeHead;
                        unimplemented!()
                    }
                }
            }
            InsertionMode::BeforeHead => match token {
                HtmlToken::Char('\t' | '\n' | '\x0c' | '\r' | ' ') => StepOutput::default(),
                HtmlToken::StartTag { tag, .. } if tag == "head" => {
                    self.insert_element_for_token(token);
                    self.mode = InsertionMode::InHead;
                    StepOutput::default()
                }
                _ => unimplemented!(),
            },
            InsertionMode::InHead => match token {
                HtmlToken::Char('\t' | '\n' | '\x0c' | '\r' | ' ') => StepOutput::default(),
                HtmlToken::EndTag { tag } if tag == "head" => {
                    self.stack_of_open_elements.pop();
                    self.mode = InsertionMode::AfterHead;
                    StepOutput::default()
                }
                _ => unimplemented!(),
            },
            InsertionMode::AfterHead => match token {
                HtmlToken::Char(c) if ['\t', '\n', '\x0c', '\r', ' '].contains(c) => {
                    self.insert_character(*c);
                    StepOutput::default()
                }
                HtmlToken::StartTag { tag, .. } if tag == "body" => {
                    self.insert_element_for_token(token);
                    self.mode = InsertionMode::InBody;
                    StepOutput::default()
                }
                _ => {
                    self.insert_element_for_token(&HtmlToken::StartTag {
                        tag: "body".to_string(),
                        self_closing: false,
                        attributes: Vec::new(),
                    });
                    self.mode = InsertionMode::InBody;
                    StepOutput {
                        reprocess: true,
                        ..Default::default()
                    }
                }
            },
            InsertionMode::InBody => match token {
                HtmlToken::Char(c @ ('\t' | '\n' | '\x0c' | '\r' | ' ')) => {
                    self.insert_character(*c);
                    StepOutput::default()
                }
                HtmlToken::Char(c) => {
                    self.insert_character(*c);
                    StepOutput::default()
                }
                HtmlToken::Eof => StepOutput::default(),
                HtmlToken::EndTag { tag } if tag == "body" => {
                    if self.stack_has_element_in_scope(|e| e.tag_name() == &ElementKind::Body) {
                        self.mode = InsertionMode::AfterBody;
                        StepOutput::default()
                    } else {
                        StepOutput::default()
                    }
                }
                HtmlToken::StartTag { tag, .. } if tag == "p" => {
                    if self.stack_has_element_in_button_scope(|e| e.tag_name() == &ElementKind::P) {
                        self.close_p_element();
                    }
                    self.insert_element_for_token(token);
                    StepOutput::default()
                }
                HtmlToken::EndTag { tag } if tag == "p" => {
                    if !self.stack_has_element_in_button_scope(|e| e.tag_name() == &ElementKind::P)
                    {
                        self.insert_element_for_token(&HtmlToken::StartTag {
                            tag: "p".to_string(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                    }
                    self.close_p_element();
                    StepOutput::default()
                }
                HtmlToken::StartTag { tag, .. } if tag == "h1" || tag == "h2" => {
                    if self.stack_has_element_in_button_scope(|e| e.tag_name() == &ElementKind::P) {
                        self.close_p_element();
                    }
                    let has_open_heading = self
                        .stack_of_open_elements
                        .last()
                        .map(|node| {
                            if let NodeData::Element(element) = node.borrow().data() {
                                [ElementKind::H1, ElementKind::H2].contains(element.tag_name())
                            } else {
                                false
                            }
                        })
                        .unwrap_or_default();
                    if has_open_heading {
                        self.stack_of_open_elements.pop();
                    }
                    self.insert_element_for_token(token);
                    StepOutput::default()
                }
                HtmlToken::StartTag { tag, .. } if tag == "a" => {
                    self.insert_element_for_token(token);
                    StepOutput::default()
                }
                HtmlToken::StartTag { tag, .. } if tag == "img" => {
                    self.insert_element_for_token(token);
                    self.stack_of_open_elements.pop();

                    StepOutput::default()
                }
                HtmlToken::EndTag { tag } if tag == "h1" || tag == "h2" => {
                    if !self.stack_has_element_in_scope(|e| {
                        [ElementKind::H1, ElementKind::H2].contains(e.tag_name())
                    }) {
                        StepOutput::default()
                    } else {
                        self.generate_implied_end_tags();
                        self.pop_stack_of_open_elements_up_to_including(|node| {
                            match node.borrow().data() {
                                NodeData::Element(element) => {
                                    [ElementKind::H1, ElementKind::H2].contains(element.tag_name())
                                }
                                _ => false,
                            }
                        });
                        StepOutput::default()
                    }
                }
                HtmlToken::StartTag { tag, .. } if tag == "textarea" => {
                    self.insert_element_for_token(token);
                    // TODO: ignore next LF
                    self.original_insertion_mode = self.mode.clone();
                    self.mode = InsertionMode::Text;
                    StepOutput {
                        set_tokenizer_state: Some(token::State::Rcdata),
                        ..Default::default()
                    }
                }
                HtmlToken::StartTag { tag, .. } if tag == "script" => {
                    let adjusted_insertion_location =
                        self.calc_appropriate_insertion_location_for_inserting_node();
                    let element = self.create_element_for_token(
                        token,
                        Namespace::Html,
                        adjusted_insertion_location.intended_parent(),
                    );
                    adjusted_insertion_location.insert(Rc::clone(&element));
                    self.stack_of_open_elements.push(Rc::clone(&element));
                    self.original_insertion_mode = self.mode.clone();
                    self.mode = InsertionMode::Text;
                    StepOutput {
                        set_tokenizer_state: Some(token::State::ScriptData),
                        ..Default::default()
                    }
                }
                HtmlToken::StartTag { tag, .. } if tag == "style" => {
                    self.parse_raw_text_element(token)
                }

                HtmlToken::StartTag {
                    tag, self_closing, ..
                } if tag == "svg" => {
                    // TODO: Reconstruct active formatting elements
                    // TODO: Adjust SVG attributes
                    // TODO: Adjust foreign attributes

                    // TODO: Insert a foreign element(SVG namespace, false)
                    self.insert_foreign_element_for_token(token, Namespace::Svg, false);

                    if *self_closing {
                        unimplemented!()
                    }

                    StepOutput::default()
                }

                HtmlToken::EndTag { tag } => {
                    for node in self.stack_of_open_elements.iter().rev().map(Rc::clone) {
                        if let NodeData::Element(element) = node.borrow().data() {
                            if &element.tag_name().to_string() == tag {
                                self.generate_implied_end_tags_except_for(&[tag]);
                                self.pop_stack_of_open_elements_up_to_including_node(Rc::clone(
                                    &node,
                                ));
                                break;
                            }
                        }
                    }
                    StepOutput::default()
                }
                _ => unimplemented!("unimplemented: {token:?}"),
            },
            InsertionMode::AfterBody => match token {
                HtmlToken::Char('\t' | '\n' | '\x0c' | '\r' | ' ') => StepOutput::default(),
                HtmlToken::EndTag { tag } if tag == "html" => {
                    self.mode = InsertionMode::AfterAfterBody;
                    StepOutput::default()
                }
                _ => unimplemented!(),
            },
            InsertionMode::AfterAfterBody => match token {
                HtmlToken::Char('\t' | '\n' | '\x0c' | '\r' | ' ') => StepOutput::default(),
                HtmlToken::Eof => StepOutput {
                    stop: true,
                    ..Default::default()
                },
                _ => unimplemented!(),
            },
            InsertionMode::Text => match token {
                HtmlToken::Char(c) => {
                    self.insert_character(*c);
                    StepOutput::default()
                }
                HtmlToken::EndTag { tag } if tag == "script" => {
                    self.stack_of_open_elements.pop().unwrap();
                    self.mode = self.original_insertion_mode.clone();
                    StepOutput::default()
                }
                HtmlToken::EndTag { .. } => {
                    self.stack_of_open_elements.pop();
                    self.mode = self.original_insertion_mode.clone();
                    // TODO: handle reentrance
                    // TODO: prepare the script element
                    StepOutput {
                        set_tokenizer_state: Some(token::State::Data),
                        ..Default::default()
                    }
                }
                _ => unreachable!(),
            },

            _ => unimplemented!(),
        }
    }

    fn process_token_in_foreign_content(&mut self, token: &HtmlToken) -> StepOutput {
        match token {
            HtmlToken::StartTag { tag, .. }
                if [
                    "b",
                    "big",
                    "blockquote",
                    "body",
                    "br",
                    "center",
                    "code",
                    "dd",
                    "div",
                    "dl",
                    "dt",
                    "em",
                    "embed",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "h5",
                    "h6",
                    "head",
                    "hr",
                    "i",
                    "img",
                    "li",
                    "listing",
                    "menu",
                    "meta",
                    "nobr",
                    "ol",
                    "p",
                    "pre",
                    "ruby",
                    "s",
                    "small",
                    "span",
                    "strong",
                    "strike",
                    "sub",
                    "sup",
                    "table",
                    "tt",
                    "u",
                    "ul",
                    "var",
                ]
                .iter()
                .any(|t| *t == tag) =>
            {
                self.pop_stack_of_open_elements_while(|node| {
                    let element_kind = {
                        if let NodeData::Element(e) = node.borrow().data() {
                            Some(e.kind.clone())
                        } else {
                            None
                        }
                    };

                    element_kind
                        .map(|kind|
                    // TODO: false if MathML|HTML text integration point
                        kind.namespace() != Namespace::Html)
                        .unwrap_or(false)
                });

                self.process_token_based_on_mode(token)
            }

            // Comment out here to reproduce CVE-2020-6413!
            HtmlToken::EndTag { tag } if tag == "br" || tag == "p" => {
                self.pop_stack_of_open_elements_while(|node| {
                    let element_kind = {
                        if let NodeData::Element(e) = node.borrow().data() {
                            Some(e.kind.clone())
                        } else {
                            None
                        }
                    };

                    element_kind
                        .map(|kind|
                    // TODO: false if MathML|HTML text integration point
                        kind.namespace() != Namespace::Html)
                        .unwrap_or(false)
                });

                self.process_token_based_on_mode(token)
            }
            HtmlToken::StartTag { self_closing, .. } => {
                let adjusted_current_node_namespace = self
                    .adjusted_current_node()
                    .and_then(|node| {
                        if let NodeData::Element(e) = &node.borrow().data {
                            Some(e.kind.namespace())
                        } else {
                            None
                        }
                    })
                    // The spec seems to assume there IS an element in the stack.
                    // Makes sense – probably it's impossible to reach here without surrounding elements
                    .unwrap();

                // TODO: adjust tag name, attributes

                self.insert_foreign_element_for_token(
                    token,
                    adjusted_current_node_namespace,
                    false,
                );

                if *self_closing {
                    unimplemented!()
                }

                StepOutput::default()
            }

            HtmlToken::EndTag { tag } => {
                if self.stack_of_open_elements.len() == 1 {
                    // "fragment case"
                    return StepOutput::default();
                }

                // Iterate over the stack until it's the 0th node
                let mut nodes = self
                    .stack_of_open_elements
                    .iter()
                    .skip(1)
                    .rev()
                    .map(Rc::clone);
                let mut maybe_node = nodes.next();
                loop {
                    match maybe_node {
                        None => {
                            // "fragment case"
                            return StepOutput::default();
                        }
                        Some(ref node) => {
                            if let NodeData::Element(e) = node.borrow().data() {
                                // TODO: No need to lowercase tag? :thinking:
                                if &e.tag_name().to_string().to_lowercase() == tag {
                                    self.pop_stack_of_open_elements_up_to_including_node(
                                        Rc::clone(node),
                                    );
                                    return StepOutput::default();
                                }
                            }

                            maybe_node = nodes.next();
                            let maybe_namespace = {
                                maybe_node.as_ref().and_then(|n| {
                                    if let NodeData::Element(e) = n.borrow().data() {
                                        Some(e.kind.namespace())
                                    } else {
                                        None
                                    }
                                })
                            };
                            if maybe_namespace == Some(Namespace::Html) {
                                return self.process_token_based_on_mode(token);
                            }
                        }
                    }
                }
            }
            _ => {
                unimplemented!("unsupported token in foreign content: {token:?}")
            }
        }
    }

    fn document(&self) -> Rc<RefCell<Node>> {
        self.window.borrow().document()
    }

    fn create_element_for_token(
        &self,
        token: &HtmlToken,
        namespace: Namespace,
        intended_parent: Rc<RefCell<Node>>,
    ) -> Rc<RefCell<Node>> {
        let local_name: &str;
        let attributes: Vec<Attribute>;
        if let HtmlToken::StartTag {
            tag,
            attributes: attrs,
            ..
        } = token
        {
            local_name = tag;
            attributes = attrs.clone();
        } else {
            unimplemented!("not a start tag");
        }
        let document = intended_parent.borrow().node_document();
        let element = Node::create_element(document, local_name, namespace);
        element.borrow_mut().extend_element_attributes(attributes);
        element
    }

    fn insert_element_for_token(&mut self, token: &HtmlToken) -> Rc<RefCell<Node>> {
        self.insert_foreign_element_for_token(token, Namespace::Html, false)
    }

    fn insert_foreign_element_for_token(
        &mut self,
        token: &HtmlToken,
        namespace: Namespace,
        only_add_to_element_stack: bool,
    ) -> Rc<RefCell<Node>> {
        let adjusted_inserted_location =
            self.calc_appropriate_insertion_location_for_inserting_node();
        let element = self.create_element_for_token(
            token,
            namespace,
            adjusted_inserted_location.intended_parent(),
        );
        if !only_add_to_element_stack {
            adjusted_inserted_location.insert(Rc::clone(&element));
        }
        self.stack_of_open_elements.push(Rc::clone(&element));
        element
    }

    fn calc_appropriate_insertion_location_for_inserting_node(&self) -> InsertionLocation {
        let target = self.current_node().unwrap();
        InsertionLocation::InsideNodeAfterLastChild(target)
    }

    fn current_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.stack_of_open_elements.last().map(Rc::clone)
    }

    fn adjusted_current_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.current_node()
    }

    fn stack_has_element_in_scope<P>(&self, predicate: P) -> bool
    where
        P: FnMut(&Element) -> bool,
    {
        self.stack_has_element_in_specific_scope(predicate, &DEFAULT_SCOPE)
    }

    fn stack_has_element_in_button_scope<P>(&self, predicate: P) -> bool
    where
        P: FnMut(&Element) -> bool,
    {
        let mut scope = vec!["button"];
        scope.extend_from_slice(&DEFAULT_SCOPE);
        self.stack_has_element_in_specific_scope(predicate, &scope)
    }

    fn stack_has_element_in_specific_scope<P>(&self, mut predicate: P, scope: &[&str]) -> bool
    where
        P: FnMut(&Element) -> bool,
    {
        for node in self.stack_of_open_elements.iter().rev() {
            if let NodeData::Element(element) = node.borrow().data() {
                if predicate(element) {
                    return true;
                }
                if scope.contains(&element.tag_name().to_string().as_str()) {
                    return false;
                }
            }
        }
        unreachable!()
    }

    fn insert_character(&self, c: char) {
        let adjusted_inserted_location =
            self.calc_appropriate_insertion_location_for_inserting_node();
        if adjusted_inserted_location.intended_parent().borrow().data() == &NodeData::Document {
            return;
        }
        if let Some(before_node) = adjusted_inserted_location.before_element() {
            if matches!(before_node.borrow().data(), NodeData::Text(_)) {
                before_node.borrow_mut().append_text_character(c);
                return;
            }
        }
        let text_node =
            Node::create_text_node(adjusted_inserted_location.document(), String::from(c));
        adjusted_inserted_location.insert(text_node);
    }

    fn close_p_element(&mut self) {
        self.generate_implied_end_tags_except_for(&["p"]);
        self.pop_stack_of_open_elements_up_to_including_tag("p");
    }

    fn generate_implied_end_tags(&mut self) {
        self.generate_implied_end_tags_except_for(&[]);
    }

    fn generate_implied_end_tags_except_for(&mut self, tags: &[&str]) {
        loop {
            if let Some(node) = self.stack_of_open_elements.last() {
                if let NodeData::Element(element) = Rc::clone(node).borrow().data() {
                    if ELEMENT_NEEDS_IMPLIED_END_TAG.contains(element.tag_name())
                        && !tags.contains(&element.tag_name().to_string().as_str())
                    {
                        self.stack_of_open_elements.pop();
                        continue;
                    }
                }
            }
            break;
        }
    }

    fn pop_stack_of_open_elements_while<P>(&mut self, mut continuing_condition: P)
    where
        P: FnMut(Rc<RefCell<Node>>) -> bool,
    {
        while let Some(open_element) = self.stack_of_open_elements.last() {
            if !continuing_condition(Rc::clone(open_element)) {
                break;
            }

            self.stack_of_open_elements.pop();
        }
    }

    fn pop_stack_of_open_elements_up_to_including<P>(&mut self, mut terminating_condition: P)
    where
        P: FnMut(Rc<RefCell<Node>>) -> bool,
    {
        while let Some(open_element) = self.stack_of_open_elements.pop() {
            if terminating_condition(open_element) {
                break;
            }
        }
    }

    fn pop_stack_of_open_elements_up_to_including_node(&mut self, node: Rc<RefCell<Node>>) {
        self.pop_stack_of_open_elements_up_to_including(|open_element| {
            Rc::ptr_eq(&open_element, &node)
        });
    }

    fn pop_stack_of_open_elements_up_to_including_tag(&mut self, tag: &str) {
        self.pop_stack_of_open_elements_up_to_including(|open_element| {
            match open_element.borrow().data() {
                NodeData::Element(element) => element.tag_name().to_string() == tag,
                _ => false,
            }
        })
    }

    fn parse_raw_text_element(&mut self, token: &HtmlToken) -> StepOutput {
        self.insert_element_for_token(token);

        self.original_insertion_mode = self.mode.clone();
        self.mode = InsertionMode::Text;

        StepOutput {
            set_tokenizer_state: Some(token::State::Rawtext),
            ..Default::default()
        }
    }
}

const ELEMENT_NEEDS_IMPLIED_END_TAG: [ElementKind; 1] = [ElementKind::P];

const DEFAULT_SCOPE: [&str; 9] = [
    "applet", "caption", "html", "table", "td", "th", "marquee", "object", "template",
];

#[derive(Debug, Clone)]
enum InsertionLocation {
    InsideNodeAfterLastChild(Rc<RefCell<Node>>),
}

impl InsertionLocation {
    fn intended_parent(&self) -> Rc<RefCell<Node>> {
        match self {
            InsertionLocation::InsideNodeAfterLastChild(parent) => parent.clone(),
        }
    }

    fn document(&self) -> Rc<RefCell<Node>> {
        match self {
            InsertionLocation::InsideNodeAfterLastChild(parent) => parent.borrow().node_document(),
        }
    }

    fn before_element(&self) -> Option<Rc<RefCell<Node>>> {
        match self {
            InsertionLocation::InsideNodeAfterLastChild(parent) => parent.borrow().last_child(),
        }
    }

    fn insert(self, node: Rc<RefCell<Node>>) {
        match self {
            InsertionLocation::InsideNodeAfterLastChild(parent) => {
                Node::append_child(parent, node);
            }
        }
    }
}

struct StepOutput {
    reprocess: bool,
    stop: bool,
    set_tokenizer_state: Option<token::State>,
}

impl Default for StepOutput {
    fn default() -> Self {
        Self {
            reprocess: false,
            stop: false,
            set_tokenizer_state: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertionMode {
    Initial,
    BeforeHtml,
    BeforeHead,
    InHead,
    AfterHead,
    InBody,
    Text,
    AfterBody,
    AfterAfterBody,
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use crate::renderer::dom::node::{Element, ElementKind};

    use super::*;

    #[test]
    fn test_body() {
        let html = "<!doctype html><html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();

        let document = window.borrow().document();
        assert_eq!(&NodeData::Document, document.borrow().data());

        Node::assert_tree_structure(document.clone());

        let document_children: Vec<_> = document.borrow().children().collect();
        assert_eq!(1, document_children.len());
        let html = document_children[0].clone();
        if let NodeData::Element(element) = html.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Html), element);
        } else {
            panic!("not an element");
        };

        let html_children: Vec<_> = html.borrow().children().collect();
        assert_eq!(2, html_children.len());
        let head = html_children[0].clone();
        if let NodeData::Element(element) = head.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Head), element);
        } else {
            panic!("not an element");
        };
        assert!(head.borrow().children().next().is_none());

        let body = html_children[1].clone();
        if let NodeData::Element(element) = body.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Body), element);
        } else {
            panic!("not an element");
        };
        assert!(body.borrow().children().next().is_none());
    }

    #[test]
    fn test_text() {
        let html = "<!doctype html><html><head></head><body>text text</body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();

        let document = window.borrow().document();
        assert_eq!(&NodeData::Document, document.borrow().data());

        Node::assert_tree_structure(document.clone());

        let document_children: Vec<_> = document.borrow().children().collect();
        assert_eq!(1, document_children.len());
        let html = document_children[0].clone();
        if let NodeData::Element(element) = html.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Html), element);
        } else {
            panic!("not an element");
        };

        let html_children: Vec<_> = html.borrow().children().collect();
        assert_eq!(2, html_children.len());
        let head = html_children[0].clone();
        if let NodeData::Element(element) = head.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Head), element);
        } else {
            panic!("not an element");
        };
        assert!(head.borrow().children().next().is_none());

        let body = html_children[1].clone();
        if let NodeData::Element(element) = body.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Body), element);
        } else {
            panic!("not an element");
        };

        let body_children: Vec<_> = body.borrow().children().collect();
        assert_eq!(1, body_children.len());
        let text = body_children[0].clone();
        if let NodeData::Text(text) = text.borrow().data() {
            assert_eq!("text text", text);
        } else {
            panic!("not a text");
        };
        assert!(text.borrow().children().next().is_none());
    }

    #[test]
    fn test_multiple_nodes() {
        {
            let html =
                "<!doctype html><html><head></head><body><p><a foo=bar>text</a></p></body></html>"
                    .to_string();
            let t = HtmlTokenizer::new(html);
            let window = HtmlParser::new(t).construct_tree();

            let document = window.borrow().document();
            assert_eq!(&NodeData::Document, document.borrow().data());

            Node::assert_tree_structure(document.clone());
            eprintln!("tree:\n{}", Node::build_ascii_tree(Rc::clone(&document)));

            let document_children: Vec<_> = document.borrow().children().collect();
            assert_eq!(1, document_children.len());
            let html = document_children[0].clone();
            if let NodeData::Element(element) = html.borrow().data() {
                assert_eq!(&Element::new(ElementKind::Html), element);
            } else {
                panic!("not an element");
            };

            let html_children: Vec<_> = html.borrow().children().collect();
            assert_eq!(2, html_children.len());
            let head = html_children[0].clone();
            if let NodeData::Element(element) = head.borrow().data() {
                assert_eq!(&Element::new(ElementKind::Head), element);
            } else {
                panic!("not an element");
            };
            assert!(head.borrow().children().next().is_none());

            let body = html_children[1].clone();
            if let NodeData::Element(element) = body.borrow().data() {
                assert_eq!(&Element::new(ElementKind::Body), element);
            } else {
                panic!("not an element");
            };

            let body_children: Vec<_> = body.borrow().children().collect();
            assert_eq!(1, body_children.len());

            let p = body_children[0].clone();
            if let NodeData::Element(element) = p.borrow().data() {
                assert_eq!(&Element::new(ElementKind::P), element);
            } else {
                panic!("not an element");
            };

            let p_children: Vec<_> = p.borrow().children().collect();
            assert_eq!(1, p_children.len());

            let a = p_children[0].clone();
            let a_expected = Element::new_with_attributes(
                ElementKind::A,
                vec![Attribute::new("foo".to_string(), "bar".to_string())],
            );

            if let NodeData::Element(element) = a.borrow().data() {
                assert_eq!(&a_expected, element);
            } else {
                panic!("not an element");
            };

            let a_children: Vec<_> = a.borrow().children().collect();
            assert_eq!(1, a_children.len());

            let text = a_children[0].clone();
            if let NodeData::Text(text) = text.borrow().data() {
                assert_eq!("text", text);
            } else {
                panic!("not a text");
            };
        }
    }

    /// https://challenge-hamayan.quiz.flatt.training/
    #[test]
    fn test_flatt_security_xss_challenge_hamayan() {
        let html = r#"
<html>
<head>
</head>
<body>
    <p id="</textarea><script>alert(origin);</script>"></p>
    <textarea name="message"><p id="</textarea><script>alert(origin);</script>"></p></textarea>
</body>
</html>
"#
        .to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();

        let document = window.borrow().document();
        assert_eq!(&NodeData::Document, document.borrow().data());

        Node::assert_tree_structure(document.clone());
        eprintln!("tree:\n{}", Node::build_ascii_tree(Rc::clone(&document)));
    }

    /// https://aszx87410.github.io/beyond-xss/en/ch2/mutation-xss/
    #[test]
    fn test_cve_2020_6413() {
        let html = r#"<!doctype html><html><head></head><body><svg></p><style><a id="</style><img src=1 onerror=alert(1)>"></body></html>"#.to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();

        let document = window.borrow().document();
        assert_eq!(&NodeData::Document, document.borrow().data());

        Node::assert_tree_structure(document.clone());
        eprintln!("tree:\n{}", Node::build_ascii_tree(Rc::clone(&document)));
    }
}
