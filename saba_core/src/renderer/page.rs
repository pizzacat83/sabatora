use alloc::rc::{Rc, Weak};
use alloc::vec;
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::{
    browser::Browser,
    display_item::DisplayItem,
    renderer::layout::computed_style::{ComputedStyle, DisplayType},
};

use super::css::cssom::CssStyleSheet;
use super::css::parser::parse_css_stylesheet;
use super::dom::node::Window;
use super::html::parser::HtmlParser;
use super::html::token::HtmlTokenizer;
use super::layout::box_tree::construct_box_tree;
use super::layout::layout_view::{get_style_content, LayoutView};
use super::layout::paint::paint;
use alloc::string::String;

#[derive(Debug, Clone)]
pub struct Page {
    browser: Weak<RefCell<Browser>>,
    frame: Option<Rc<RefCell<Window>>>,
    style: Option<CssStyleSheet>,
}
impl Page {
    pub(crate) fn new() -> Self {
        Self {
            browser: Weak::new(),
            frame: None,
            style: None,
        }
    }

    pub(crate) fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }

    pub fn populate_frame(&mut self, html: String) {
        let frame = HtmlParser::new(HtmlTokenizer::new(html)).construct_tree();
        let dom = frame.borrow().document();
        let style = get_style_content(dom);
        let cssom = parse_css_stylesheet(style);

        self.frame = Some(frame);
        self.style = Some(cssom);
    }

    pub fn display_items(&self) -> Vec<DisplayItem> {
        if let (Some(frame), Some(cssom)) = (&self.frame, &self.style) {
            let dom = frame.borrow().document();
            let layout_view = LayoutView::layout(dom, cssom);

            let box_tree = construct_box_tree(layout_view);

            paint(box_tree)
        } else {
            Vec::new()
        }
    }
}
