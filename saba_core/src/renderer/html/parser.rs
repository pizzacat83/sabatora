use core::cell::RefCell;

use alloc::{
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use crate::renderer::dom::node::{ElementKind, Node, NodeData, Window};

use super::{
    attribute::Attribute,
    token::{HtmlToken, HtmlTokenizer},
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
                        let element = self.create_element_for_token(token, self.document());
                        Node::append_child(self.document(), Rc::clone(&element));
                        self.stack_of_open_elements.push(Rc::clone(&element));
                        self.mode = InsertionMode::BeforeHead;
                        StepOutput::default()
                    }
                    HtmlToken::EndTag { tag: _ } => StepOutput::default(),
                    _ => {
                        // TODO
                        self.mode = InsertionMode::BeforeHead;
                        todo!()
                    }
                }
            }
            InsertionMode::BeforeHead => match token {
                HtmlToken::StartTag { tag, .. } if tag == "head" => {
                    self.insert_element_for_token(token);
                    self.mode = InsertionMode::InHead;
                    StepOutput::default()
                }
                _ => todo!(),
            },
            InsertionMode::InHead => match token {
                HtmlToken::EndTag { tag } if tag == "head" => {
                    self.stack_of_open_elements.pop();
                    self.mode = InsertionMode::AfterHead;
                    StepOutput::default()
                }
                _ => todo!(),
            },
            InsertionMode::AfterHead => match token {
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
                HtmlToken::Char(c) => {
                    self.insert_character(*c);
                    StepOutput::default()
                }
                HtmlToken::EndTag { tag } if tag == "body" => {
                    if self.stack_has_element_in_scope("body") {
                        self.mode = InsertionMode::AfterBody;
                        StepOutput::default()
                    } else {
                        StepOutput::default()
                    }
                }
                HtmlToken::StartTag { tag, .. } if tag == "p" => {
                    if self.stack_has_element_in_button_scope("p") {
                        self.close_p_element();
                    }
                    self.insert_element_for_token(token);
                    StepOutput::default()
                }
                HtmlToken::EndTag { tag } if tag == "p" => {
                    if !self.stack_has_element_in_button_scope("p") {
                        self.insert_element_for_token(&HtmlToken::StartTag {
                            tag: "p".to_string(),
                            self_closing: false,
                            attributes: Vec::new(),
                        });
                    }
                    self.close_p_element();
                    StepOutput::default()
                }
                HtmlToken::StartTag { tag, .. } if tag == "a" => {
                    self.insert_element_for_token(token);
                    StepOutput::default()
                }
                HtmlToken::EndTag { tag } => {
                    for node in self.stack_of_open_elements.iter().rev().map(Rc::clone) {
                        if let NodeData::Element(element) = node.borrow().data() {
                            if &element.tag_name().to_string() == tag {
                                self.generate_implied_end_tags_except_for(tag);
                                self.pop_stack_of_open_elements_up_to_including_node(Rc::clone(
                                    &node,
                                ));
                                break;
                            }
                        }
                    }
                    StepOutput::default()
                }
                _ => todo!(),
            },
            InsertionMode::AfterBody => match token {
                HtmlToken::EndTag { tag } if tag == "html" => {
                    self.mode = InsertionMode::AfterAfterBody;
                    StepOutput::default()
                }
                _ => todo!(),
            },
            InsertionMode::AfterAfterBody => match token {
                HtmlToken::Eof => StepOutput {
                    stop: true,
                    ..Default::default()
                },
                _ => todo!(),
            },

            _ => todo!(),
        }
    }

    fn document(&self) -> Rc<RefCell<Node>> {
        self.window.borrow().document()
    }

    fn create_element_for_token(
        &self,
        token: &HtmlToken,
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
        let element = Node::create_element(document, local_name);
        element.borrow_mut().extend_element_attributes(attributes);
        element
    }

    fn insert_element_for_token(&mut self, token: &HtmlToken) -> Rc<RefCell<Node>> {
        self.insert_foreign_element_for_token(token, false)
    }

    fn insert_foreign_element_for_token(
        &mut self,
        token: &HtmlToken,
        only_add_to_element_stack: bool,
    ) -> Rc<RefCell<Node>> {
        let adjusted_inserted_location =
            self.calc_appropriate_insertion_location_for_inserting_node();
        let element =
            self.create_element_for_token(token, adjusted_inserted_location.intended_parent());
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

    fn stack_has_element_in_scope(&self, tag_name: &str) -> bool {
        self.stack_has_element_in_specific_scope(tag_name, &DEFAULT_SCOPE)
    }

    fn stack_has_element_in_button_scope(&self, tag_name: &str) -> bool {
        let mut scope = vec!["button"];
        scope.extend_from_slice(&DEFAULT_SCOPE);
        self.stack_has_element_in_specific_scope(tag_name, &scope)
    }

    fn stack_has_element_in_specific_scope(&self, tag_name: &str, scope: &[&str]) -> bool {
        for node in self.stack_of_open_elements.iter().rev() {
            if let NodeData::Element(element) = node.borrow().data() {
                if element.tag_name().to_string() == tag_name {
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
        self.generate_implied_end_tags_except_for("p");
        self.pop_stack_of_open_elements_up_to_including_tag("p");
    }

    fn generate_implied_end_tags_except_for(&mut self, tag: &str) {
        loop {
            if let Some(node) = self.stack_of_open_elements.last() {
                if let NodeData::Element(element) = Rc::clone(node).borrow().data() {
                    if ELEMENT_NEEDS_IMPLIED_END_TAG.contains(&element.tag_name())
                        && element.tag_name().to_string() != tag
                    {
                        self.stack_of_open_elements.pop();
                        continue;
                    }
                }
            }
            break;
        }
    }

    fn pop_stack_of_open_elements_up_to_including_node(&mut self, node: Rc<RefCell<Node>>) {
        loop {
            let open_element = match self.stack_of_open_elements.pop() {
                Some(n) => n,
                None => break,
            };
            if Rc::ptr_eq(&open_element, &node) {
                break;
            }
        }
    }

    fn pop_stack_of_open_elements_up_to_including_tag(&mut self, tag: &str) {
        while let Some(open_element) = self.stack_of_open_elements.pop() {
            if let NodeData::Element(element) = open_element.borrow().data() {
                if element.tag_name().to_string() == tag {
                    break;
                }
            };
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
}

impl Default for StepOutput {
    fn default() -> Self {
        Self {
            reprocess: false,
            stop: false,
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
        let html = "<!doctype html><html><head></head><body>text</body></html>".to_string();
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
            assert_eq!("text", text);
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
}
