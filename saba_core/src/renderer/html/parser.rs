use core::{cell::RefCell, iter::Step};

use alloc::{rc::Rc, string::ToString, vec::Vec};

use crate::renderer::dom::node::{ElementKind, InsertionLocation, Node, NodeData, Window};

use super::token::{HtmlToken, HtmlTokenizer};

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
                    StepOutput { reprocess: true }
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
                        self.document().borrow_mut().append(element);
                        self.stack_of_open_elements.push(element.clone());
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
                    let element = self.insert_element_for_token(token);
                    self.mode = InsertionMode::InHead;
                    StepOutput::default()
                }
                _ => todo!(),
            },
            InsertionMode::InHead => match token {
                &HtmlToken::EndTag { tag } if tag == "head" => {
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

    fn create_element_for_token(&self, token: &HtmlToken, intended_parent: Rc<RefCell<Node>>) -> ! {
        let local_name: &str;
        if let HtmlToken::StartTag {
            tag,
            self_closing,
            attributes,
        } = token
        {
            local_name = tag;
        } else {
            panic!("not a start tag");
        }
        let document = intended_parent.node_document();
        let element = self.create_element(document, local_name);
        element.attributes = token.attributes().clone();
        element
    }

    fn create_element(&self, document: !, local_name: &str) -> ! {
        Element::new(ElementKind::from(local_name), document)
    }

    fn insert_element_for_token(&mut self, token: &HtmlToken) -> ! {
        return self.insert_foreign_element_for_token(token, false);
    }

    fn insert_foreign_element_for_token(
        &self,
        token: &HtmlToken,
        only_add_to_element_stack: bool,
    ) -> ! {
        let adjusted_inserted_location =
            self.calc_appropriate_insertion_location_for_inserting_node();
        let element =
            self.create_element_for_token(token, adjusted_inserted_location.intended_parent());
        if !only_add_to_element_stack {
            adjusted_inserted_location.insert(element);
        }
        self.stack_of_open_elements.push(element);
        element
    }

    fn calc_appropriate_insertion_location_for_inserting_node(&self) -> InsertionLocation {
        let target = self.current_node();
        InsertionLocation::InsideNodeAfterLastChild(target)
    }

    fn current_node(&self) -> ! {
        self.stack_of_open_elements.last().unwrap().clone()
    }
}

#[derive(Debug, Clone)]
pub enum InsertionLocation {
    InsideNodeAfterLastChild(Rc<RefCell<Node>>),
}

impl InsertionLocation {
    fn intended_parent(&self) -> Rc<RefCell<Node>> {
        match self {
            InsertionLocation::InsideNodeAfterLastChild(parent) => parent.clone(),
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
        assert_eq!(2, document_children.len());
        let html = document_children[0].clone();
        if let NodeData::Element(element) = html.borrow().data() {
            assert_eq!(&Element::new(ElementKind::Html), element);
        } else {
            panic!("not an element");
        }
    }
}
