use alloc::rc::{Rc, Weak};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::display_item::{self, DisplayItem};
use crate::renderer::layout::computed_style::{ComputedStyle, DisplayType};
use crate::renderer::{
    css::cssom::CssStyleSheet,
    dom::node::{ElementKind, Node},
};

use super::layout_object::{LayoutObject, LayoutObjectKind, LayoutPoint, LayoutSize};

// TODO: refactoring
#[derive(Debug, Clone)]
pub struct LayoutView {
    root: Option<Rc<RefCell<LayoutObject>>>,
}

impl LayoutView {
    pub fn layout(dom: Rc<RefCell<Node>>, cssom: &CssStyleSheet) -> LayoutView {
        let body_dom = Node::get_element_by_tag_name(dom, ElementKind::Body);
        LayoutView {
            root: build_layout_tree(&body_dom, &None, cssom),
        }
    }
}

fn build_layout_tree(
    node: &Option<Rc<RefCell<Node>>>,
    parent_obj: &Option<Rc<RefCell<LayoutObject>>>,
    cssom: &CssStyleSheet,
) -> Option<Rc<RefCell<LayoutObject>>> {
    // TODO
    if let Some(node) = node {
        Some(Rc::new(RefCell::new(LayoutObject {
            kind: LayoutObjectKind::Text,
            node: Rc::clone(node),
            first_child: None,
            next_sibling: None,
            parent: Weak::new(),
            style: ComputedStyle {
                display: Some(DisplayType::Block),
            },
        })))
    } else {
        None
    }
}

// TODO: find specification of the list of stylesheet sources
pub fn get_style_content(root: Rc<RefCell<Node>>) -> String {
    Node::get_element_by_tag_name(root, ElementKind::Style)
        .map(|elem| elem.borrow().text_content())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use crate::renderer::{
        css::{
            parser::{parse_css_stylesheet, StyleSheetParser},
            token::CssTokenizer,
        },
        dom::node::{Element, ElementKind, NodeData},
        html::{parser::HtmlParser, token::HtmlTokenizer},
        layout::layout_object::LayoutObjectKind,
    };

    use super::*;

    #[test]
    fn test_body() {
        let html = "<html><head></head><body></body></html>";
        let layout_view = create_layout_view(html);
        let root = layout_view.root.unwrap();
        assert_eq!(LayoutObjectKind::Block, root.borrow().kind);
        assert_eq!(
            NodeData::Element(Element::new(ElementKind::Body)),
            root.borrow().node.borrow().data
        );
    }

    fn create_layout_view(html: &str) -> LayoutView {
        let t = HtmlTokenizer::new(html.into());
        let window = HtmlParser::new(t).construct_tree();
        let dom = window.borrow().document();
        let style = get_style_content(dom.clone());
        let cssom = parse_css_stylesheet(style);
        LayoutView::layout(dom, &cssom)
    }
}
