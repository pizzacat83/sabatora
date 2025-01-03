use core::{borrow::BorrowMut, cell::RefCell};

use alloc::{
    rc::{Rc, Weak},
    string::String,
    vec::Vec,
};

use crate::renderer::html::attribute::Attribute;

#[derive(Debug, Clone)]
pub struct Node {
    pub data: NodeData,
    window: Weak<RefCell<Window>>,
    parent: Weak<RefCell<Node>>,
    first_child: Option<Rc<RefCell<Node>>>,
    last_child: Weak<RefCell<Node>>,
    previous_sibling: Weak<RefCell<Node>>,
    next_sibling: Option<Rc<RefCell<Node>>>,
}

// sababook did a custom implementation of PartialEq for Node, but I'm not sure why it's necessary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeData {
    Document,
    Element(Element),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Window {
    document: Rc<RefCell<Node>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    kind: ElementKind,
    attributes: Vec<Attribute>,
}

impl Element {
    pub fn new(kind: ElementKind) -> Self {
        Self {
            kind,
            attributes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Html,
    Head,
    Style,
    Body,
}

impl Window {
    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }

    pub fn new() -> Rc<RefCell<Self>> {
        let mut window = Rc::new(RefCell::new(Self {
            document: Rc::new(RefCell::new(Node::new_orphan(NodeData::Document))),
        }));
        window.get_mut().document.get_mut().window = Rc::downgrade(&window);
        window
    }
}

impl Node {
    fn new_orphan(data: NodeData) -> Self {
        Self {
            data,
            window: Weak::new(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        }
    }

    pub fn data(&self) -> &NodeData {
        &self.data
    }
}

struct NodeChildrenIterator {
    next: Option<Rc<RefCell<Node>>>,
}

impl Node {
    pub fn children(&self) -> NodeChildrenIterator {
        NodeChildrenIterator {
            next: self.first_child.clone(),
        }
    }
}

impl Iterator for NodeChildrenIterator {
    type Item = Rc<RefCell<Node>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(current) = self.next {
            self.next = current.borrow().next_sibling;
            Some(current)
        } else {
            None
        }
    }
}
