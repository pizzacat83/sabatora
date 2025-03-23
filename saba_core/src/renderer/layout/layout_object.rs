use alloc::rc::{Rc, Weak};
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::{
    display_item::DisplayItem,
    renderer::{
        css::cssom::CssStyleSheet,
        dom::node::{ElementKind, Node},
    },
};

use super::computed_style::ComputedStyle;

#[derive(Debug, Clone)]
pub struct LayoutObject {
    pub kind: LayoutObjectKind,
    pub node: Rc<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<LayoutObject>>>,
    pub next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    pub parent: Weak<RefCell<LayoutObject>>,
    pub style: ComputedStyle,
    pub point: LayoutPoint,
    pub size: LayoutSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutPoint {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutSize {
    pub width: i64,
    pub height: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutObjectKind {
    Block,
    Inline,
    Text,
}

impl LayoutObject {
    pub fn paint(&self) -> Vec<DisplayItem> {
        todo!()
    }
}
