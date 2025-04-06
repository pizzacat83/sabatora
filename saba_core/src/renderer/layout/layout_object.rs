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
    pub node: Rc<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<LayoutObject>>>,
    pub next_sibling: Option<Rc<RefCell<LayoutObject>>>,
    pub parent: Weak<RefCell<LayoutObject>>,
    pub style: ComputedStyle,
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

impl LayoutObject {
    pub fn paint(&self) -> Vec<DisplayItem> {
        todo!()
    }

    pub fn children(&self) -> ChildrenIterator {
        ChildrenIterator {
            next: self.first_child.clone(),
        }
    }
}

pub struct ChildrenIterator {
    next: Option<Rc<RefCell<LayoutObject>>>,
}

impl Iterator for ChildrenIterator {
    type Item = Rc<RefCell<LayoutObject>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next.take() {
            self.next = current.borrow().next_sibling.as_ref().map(Rc::clone);
            Some(current)
        } else {
            None
        }
    }
}
