use core::cell::RefCell;

use alloc::{rc::Rc, vec::Vec};

use crate::renderer::dom::node::{Node, NodeData, Window};

use super::token::HtmlTokenizer;

#[derive(Debug, Clone)]
pub struct HtmlParser {
    window: Window,
    mode: InsertionMode,
    original_insertion_mode: InsertionMode,
    stack_of_open_elements: Vec<Rc<RefCell<Node>>>,
    t: HtmlTokenizer,
}
impl HtmlParser {
    fn new(t: HtmlTokenizer) -> Self {
        Self {
            window: todo!(),
            mode: todo!(),
            original_insertion_mode: todo!(),
            stack_of_open_elements: todo!(),
            t,
        }
    }

    fn construct_tree(&self) -> Rc<RefCell<Window>> {
        todo!()
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

    use super::*;

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>".to_string();
        let t = HtmlTokenizer::new(html);
        let window = HtmlParser::new(t).construct_tree();
        let document = window.borrow().document();
        assert_eq!(&NodeData::Document, document.borrow().data());
    }
}
