use alloc::rc::{Rc, Weak};
use core::cell::RefCell;

use crate::renderer::dom::node::Node;

#[derive(Debug, Clone)]
pub struct LayoutObject {
    kind: LayoutObjectKind,
    node: Rc<RefCell<Node>>,
    first_child: Option<Rc<RefCell<LayoutObject>>>,
    next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    parent: Weak<RefCell<LayoutObject>>,
    style: ComputedStyle,
    size: LayoutSize,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedStyle {
    display: Option<DisplayType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayType {
    Block,
    Inline,
    None,
}
