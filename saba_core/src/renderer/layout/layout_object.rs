use alloc::rc::{Rc, Weak};
use core::cell::RefCell;

use crate::renderer::{
    css::cssom::CssStyleSheet,
    dom::node::{ElementKind, Node},
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
    pub size: LayoutSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutSize {
    width: i64,
    height: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutObjectKind {
    Block,
    Inline,
}
