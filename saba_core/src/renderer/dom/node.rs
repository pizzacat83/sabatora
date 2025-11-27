use core::{cell::RefCell, fmt::Display};

use alloc::{
    format,
    rc::{Rc, Weak},
    string::{String, ToString},
    vec::Vec,
};

use crate::renderer::html::attribute::Attribute;

#[derive(Debug, Clone)]
pub struct Node {
    pub data: NodeData,
    pub window: Weak<RefCell<Window>>,
    pub parent: Weak<RefCell<Node>>,
    pub first_child: Option<Rc<RefCell<Node>>>,
    pub last_child: Weak<RefCell<Node>>,
    pub previous_sibling: Weak<RefCell<Node>>,
    pub next_sibling: Option<Rc<RefCell<Node>>>,
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
    pub kind: ElementKind,
    pub attributes: Vec<Attribute>,
}

impl Element {
    pub fn new(kind: ElementKind) -> Self {
        Self {
            kind,
            attributes: Vec::new(),
        }
    }

    pub fn new_with_attributes(kind: ElementKind, attributes: Vec<Attribute>) -> Self {
        Self { kind, attributes }
    }

    pub fn tag_name(&self) -> &ElementKind {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementKind {
    Html,
    Head,
    Style,
    Body,
    P,
    H1,
    H2,
    A,
    Textarea,
    Script,
    Svg,
    // Note: Maybe a fully-qualified name like SVGSVGElement is better?
}

impl ElementKind {
    fn from_name(local_name: &str, namespace: Namespace) -> Option<Self> {
        match namespace {
            Namespace::Html => match local_name {
                "html" => Some(Self::Html),
                "head" => Some(Self::Head),
                "style" => Some(Self::Style),
                "body" => Some(Self::Body),
                "p" => Some(Self::P),
                "h1" => Some(Self::H1),
                "h2" => Some(Self::H2),
                "a" => Some(Self::A),
                "textarea" => Some(Self::Textarea),
                "script" => Some(Self::Script),
                _ => None,
            },
            Namespace::Svg => match local_name {
                "svg" => Some(Self::Svg),
                _ => None,
            },
        }
    }

    pub fn namespace(&self) -> Namespace {
        match self {
            Self::Html => Namespace::Html,
            Self::Head => Namespace::Html,
            Self::Style => Namespace::Html,
            Self::Body => Namespace::Html,
            Self::P => Namespace::Html,
            Self::H1 => Namespace::Html,
            Self::H2 => Namespace::Html,
            Self::A => Namespace::Html,
            Self::Textarea => Namespace::Html,
            Self::Script => Namespace::Html,
            Self::Svg => Namespace::Svg,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Namespace {
    Html,
    Svg,
}

// Warn: This assumes the tag is in the HTML namespace.
impl TryFrom<&str> for ElementKind {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "html" => Ok(Self::Html),
            "head" => Ok(Self::Head),
            "style" => Ok(Self::Style),
            "body" => Ok(Self::Body),
            "p" => Ok(Self::P),
            "h1" => Ok(Self::H1),
            "h2" => Ok(Self::H2),
            "a" => Ok(Self::A),
            "textarea" => Ok(Self::Textarea),
            "script" => Ok(Self::Script),
            _ => Err(format!("unknown element kind: {}", value)),
        }
    }
}

impl Display for ElementKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Html => write!(f, "html"),
            Self::Head => write!(f, "head"),
            Self::Style => write!(f, "style"),
            Self::Body => write!(f, "body"),
            Self::P => write!(f, "p"),
            Self::H1 => write!(f, "h1"),
            Self::H2 => write!(f, "h2"),
            Self::A => write!(f, "a"),
            Self::Textarea => write!(f, "textarea"),
            Self::Script => write!(f, "script"),
            Self::Svg => write!(f, "svg"),
        }
    }
}

impl Window {
    pub fn document(&self) -> Rc<RefCell<Node>> {
        self.document.clone()
    }

    pub fn new() -> Rc<RefCell<Self>> {
        let document = Rc::new(RefCell::new(Node {
            data: NodeData::Document,
            window: Weak::new(), // will be set after creating the window
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        }));
        let window = Rc::new(RefCell::new(Self {
            document: Rc::clone(&document),
        }));
        document.borrow_mut().window = Rc::downgrade(&window);
        window
    }
}

impl Node {
    pub fn data(&self) -> &NodeData {
        &self.data
    }

    pub fn node_document(&self) -> Rc<RefCell<Node>> {
        self.window
            .upgrade()
            .map(|w| w.borrow().document.clone())
            .unwrap()
    }

    pub fn last_child(&self) -> Option<Rc<RefCell<Node>>> {
        self.last_child.upgrade()
    }

    pub fn create_element(
        document: Rc<RefCell<Node>>,
        local_name: &str,
        namespace: Namespace,
    ) -> Rc<RefCell<Node>> {
        let element = Node {
            data: NodeData::Element(Element {
                kind: ElementKind::from_name(local_name, namespace).unwrap(),
                attributes: Vec::new(),
            }),
            window: document.borrow().window.clone(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        };
        Rc::new(RefCell::new(element))
    }

    pub fn create_text_node(document: Rc<RefCell<Node>>, c: String) -> Rc<RefCell<Node>> {
        let text_node = Rc::new(RefCell::new(Node {
            data: NodeData::Text(c),
            window: document.borrow().window.clone(),
            parent: Weak::new(),
            first_child: None,
            last_child: Weak::new(),
            previous_sibling: Weak::new(),
            next_sibling: None,
        }));

        text_node
    }

    pub fn extend_element_attributes(&mut self, attributes: Vec<Attribute>) {
        match &mut self.data {
            NodeData::Element(element) => {
                element.attributes.extend(attributes);
            }
            _ => panic!("not an element"),
        }
    }

    pub fn append_text_character(&mut self, c: char) {
        match &mut self.data {
            NodeData::Text(text) => {
                text.push(c);
            }
            _ => panic!("not a text node"),
        }
    }

    /// append the node as a child of self, after the last child
    pub fn append_child(parent: Rc<RefCell<Node>>, node: Rc<RefCell<Node>>) {
        // TODO: refer to the specificaton

        let mut parent_ref = parent.borrow_mut();
        let mut node_ref = node.borrow_mut();

        assert!(Weak::ptr_eq(&parent_ref.window, &node_ref.window));

        // assert that this node is has no parent or siblings
        assert!(node_ref.parent.upgrade().is_none());
        assert!(node_ref.previous_sibling.upgrade().is_none());
        assert!(node_ref.next_sibling.is_none());

        if let Some(last_child) = Weak::clone(&parent_ref.last_child).upgrade() {
            parent_ref.last_child = Rc::downgrade(&node);

            last_child.borrow_mut().next_sibling = Some(Rc::clone(&node));

            node_ref.parent = Rc::downgrade(&parent);
            node_ref.previous_sibling = Rc::downgrade(&last_child);
        } else {
            // make the node the first child
            assert!(parent_ref.first_child.is_none());
            parent_ref.first_child = Some(Rc::clone(&node));
            parent_ref.last_child = Rc::downgrade(&node);

            node_ref.parent = Rc::downgrade(&parent);
        }
    }

    pub fn get_element_by_tag_name(
        node: Rc<RefCell<Self>>,
        tag: ElementKind,
    ) -> Option<Rc<RefCell<Self>>> {
        if let NodeData::Element(Element { kind, .. }) = &node.borrow().data {
            if kind == &tag {
                return Some(node.clone());
            }
        }
        node.borrow()
            .children()
            .find_map(|node| Self::get_element_by_tag_name(node, tag.clone()))
    }

    pub fn text_content(&self) -> String {
        let mut content = String::new();
        if let NodeData::Text(s) = &self.data {
            content += s;
        }
        for node in self.children() {
            content += &node.borrow().text_content()
        }

        content
    }
}

pub struct NodeChildrenIterator {
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
        if let Some(current) = self.next.take() {
            self.next = current.borrow().next_sibling.as_ref().map(Rc::clone);
            Some(current)
        } else {
            None
        }
    }
}

impl Node {
    pub fn assert_tree_structure(node: Rc<RefCell<Node>>) {
        Self::assert_tree_structure_rec(node, None);
    }

    fn assert_tree_structure_rec(node: Rc<RefCell<Node>>, parent: Option<Rc<RefCell<Node>>>) {
        if let Some(ref parent) = parent {
            assert!(Weak::ptr_eq(&parent.borrow().window, &node.borrow().window));
            assert!(Weak::ptr_eq(&node.borrow().parent, &Rc::downgrade(parent)));

            if let Some(ref next_sibling) = node.borrow().next_sibling {
                assert!(Weak::ptr_eq(
                    &next_sibling.borrow().previous_sibling,
                    &Rc::downgrade(&node)
                ));
            } else {
                assert!(Weak::ptr_eq(
                    &parent.borrow().last_child,
                    &Rc::downgrade(&node)
                ));
            }
        } else {
            assert!(node.borrow().parent.upgrade().is_none());
            assert!(node.borrow().previous_sibling.upgrade().is_none());
            assert!(node.borrow().next_sibling.is_none());
        }

        if let Some(first_child) = node.borrow().first_child.as_ref() {
            assert!(&first_child.borrow().previous_sibling.upgrade().is_none());
        }

        if let Some(last_child) = node.borrow().last_child.upgrade() {
            assert!(&last_child.borrow().next_sibling.is_none());
        }

        for child in node.borrow().children() {
            Self::assert_tree_structure_rec(child, Some(Rc::clone(&node)));
        }
    }
}

impl Node {
    pub fn build_ascii_tree(node: Rc<RefCell<Node>>) -> String {
        let mut buf = String::new();
        Self::build_ascii_tree_rec(node, &mut buf, 0);

        buf
    }

    fn build_ascii_tree_rec(node: Rc<RefCell<Node>>, writer: &mut String, depth: usize) {
        writer.push_str(&"   ".repeat(depth));
        writer.push_str("|- ");

        let node = node.borrow();
        writer.push_str(&format!("{:?}\n", node.data));

        for child in node.children() {
            Self::build_ascii_tree_rec(child, writer, depth + 1);
        }
    }
}
