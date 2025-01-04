use core::cell::RefCell;

use alloc::{borrow::ToOwned, rc::Rc, string::ToString, vec::Vec};

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
        }
        self.window.clone()
    }

    fn step(&mut self, token: &HtmlToken) -> StepOutput {
        match self.mode {
            InsertionMode::Initial => match token {
                HtmlToken::Char('\t' | '\n' | '\x0c' | ' ') => StepOutput::default(),
                HtmlToken::DoctypeTag { name } => {
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
                    HtmlToken::StartTag {
                        tag,
                        self_closing,
                        attributes,
                    } if tag == "html" => {
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
                HtmlToken::StartTag {
                    tag,
                    self_closing,
                    attributes,
                } if tag == "head" => {
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
                HtmlToken::StartTag {
                    tag,
                    self_closing,
                    attributes,
                } if tag == "body" => {
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
                HtmlToken::EndTag { tag } if tag == "body" => {
                    if self.stack_has_element_in_scope("body") {
                        self.mode = InsertionMode::AfterBody;
                        StepOutput::default()
                    } else {
                        StepOutput::default()
                    }
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
            self_closing,
            attributes: attrs,
        } = token
        {
            local_name = tag;
            attributes = attrs.clone();
        } else {
            unimplemented!("not a start tag");
        }
        let document = intended_parent.borrow().node_document();
        let mut element = Node::create_element(document, local_name);
        element.borrow_mut().extend_element_attributes(attributes);
        element
    }

    fn insert_element_for_token(&mut self, token: &HtmlToken) -> Rc<RefCell<Node>> {
        return self.insert_foreign_element_for_token(token, false);
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
            adjusted_inserted_location.insert_element(Rc::clone(&element));
        }
        self.stack_of_open_elements.push(Rc::clone(&element));
        element
    }

    fn calc_appropriate_insertion_location_for_inserting_node(&self) -> InsertionLocation {
        let target = self.current_node().unwrap();
        InsertionLocation::InsideNodeAfterLastChild(target)
    }

    fn current_node(&self) -> Option<Rc<RefCell<Node>>> {
        self.stack_of_open_elements.last().map(|e| Rc::clone(e))
    }

    fn stack_has_element_in_scope(&self, tag_name: &str) -> bool {
        return self.stack_has_element_in_specific_scope(tag_name, &DEFAULT_SCOPE);
    }

    fn stack_has_element_in_specific_scope(&self, tag_name: &str, default_scope: &[&str]) -> bool {
        for node in self.stack_of_open_elements.iter().rev() {
            if let NodeData::Element(element) = node.borrow().data() {
                if element.tag_name() == tag_name {
                    return true;
                }
                if default_scope.contains(&element.tag_name().as_str()) {
                    return false;
                }
            }
        }
        unreachable!()
    }
}

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

    fn insert_element(self, element: Rc<RefCell<Node>>) {
        match self {
            InsertionLocation::InsideNodeAfterLastChild(parent) => {
                Node::append_child(parent, element);
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

    use crate::renderer::dom::node::Element;

    use super::*;

    #[test]
    fn test_body() {
        let html = "<!doctype html><html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(&NodeData::Document, document.borrow().data());

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
}
