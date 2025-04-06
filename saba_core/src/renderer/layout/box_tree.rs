//! <https://www.w3.org/TR/CSS2/visuren.html>
//! <https://www.w3.org/TR/css-display-3/>

use core::{cell::RefCell, mem};

use crate::renderer::dom::node::{Element, ElementKind, NodeData};
use alloc::rc::Rc;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::{
    computed_style::{ComputedStyle, DisplayType},
    layout_object::LayoutObject,
    layout_view::LayoutView,
};

/// TODO: distinguish block-level and block-container to support inline-box
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockBox {
    pub data: BlockBoxData,
    pub style: ComputedStyle,
    pub children: BlockBoxChildren,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBoxData {
    Element(Element),
    Anonymous,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockBoxChildren {
    Empty,
    Inlines(Vec<InlineBox>),
    Blocks(Vec<BlockBox>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineBox {
    pub data: InlineBoxData,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub children: Vec<InlineBox>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineBoxData {
    Element(Element),
    Anonymous,
}

pub fn construct_box_tree(layout_view: LayoutView) -> BlockBox {
    match layout_view.root {
        Some(node) => {
            // TODO: consider other cases
            let boxes = produce_boxes(node);
            match boxes.into_iter().next() {
                Some(Box::Block(b)) => b,
                _ => unimplemented!(),
            }
        }
        // TODO: maybe we should just return None
        None => BlockBox {
            data: BlockBoxData::Anonymous,
            style: ComputedStyle {
                display: Some(DisplayType::Block),
            },
            children: BlockBoxChildren::Empty,
        },
    }
}

fn produce_boxes(object: Rc<RefCell<LayoutObject>>) -> Vec<Box> {
    let object = object.borrow();
    let children = object.children().flat_map(produce_boxes);

    let mut b = match object.style.display {
        Some(DisplayType::Block) => {
            if let NodeData::Element(element) = &object.node.borrow().data {
                Box::Block(BlockBox {
                    data: BlockBoxData::Element(element.clone()),
                    style: object.style.clone(),
                    children: BlockBoxChildren::Empty,
                })
            } else {
                unreachable!()
            }
        }
        Some(DisplayType::Inline) => {
            let data = match &object.node.borrow().data {
                NodeData::Element(element) => InlineBoxData::Element(element.clone()),
                NodeData::Text(_) => InlineBoxData::Anonymous,
                NodeData::Document => unreachable!(),
            };
            let text = match &object.node.borrow().data {
                NodeData::Text(text) => Some(text.into()),
                _ => None,
            };
            Box::Inline(InlineBox {
                data,
                text,
                style: object.style.clone(),
                children: Vec::new(),
            })
        }
        Some(DisplayType::None) => unreachable!(),
        None => unreachable!(),
    };

    for child in children {
        match (&mut b, child) {
            (Box::Inline(ref mut b), Box::Inline(c)) => {
                b.children.push(c);
            }
            (Box::Block(ref mut b_), Box::Inline(c)) => match &mut b_.children {
                BlockBoxChildren::Empty => {
                    b_.children = BlockBoxChildren::Inlines(vec![c]);
                }
                BlockBoxChildren::Inlines(children) => {
                    children.push(c);
                }
                BlockBoxChildren::Blocks(children) => {
                    match children
                        .last_mut()
                        .filter(|block| matches!(block.data, BlockBoxData::Anonymous))
                    {
                        Some(anon) => {
                            if let BlockBoxChildren::Inlines(ref mut children) = anon.children {
                                children.push(c);
                            } else {
                                // Anonymous block always contain inline boxes
                                unreachable!();
                            }
                        }
                        None => {
                            let anon = BlockBox {
                                data: BlockBoxData::Anonymous,
                                style: b_.style.clone(),
                                children: BlockBoxChildren::Inlines(vec![c]),
                            };
                            children.push(anon);
                        }
                    }
                }
            },
            (Box::Block(ref mut b_), Box::Block(c)) => match &mut b_.children {
                BlockBoxChildren::Empty => {
                    b_.children = BlockBoxChildren::Blocks(vec![c]);
                }
                BlockBoxChildren::Blocks(children) => {
                    children.push(c);
                }
                BlockBoxChildren::Inlines(inline_children) => {
                    let block_children = vec![
                        BlockBox {
                            data: BlockBoxData::Anonymous,
                            style: b_.style.clone(),
                            children: BlockBoxChildren::Inlines(inline_children.clone()),
                        },
                        c,
                    ];
                    b_.children = BlockBoxChildren::Blocks(block_children);
                }
            },
            (Box::Inline(b_), Box::Block(c)) => {
                b = Box::Block(BlockBox {
                    data: BlockBoxData::Anonymous,
                    style: b_.style.clone(),
                    children: BlockBoxChildren::Blocks(vec![
                        BlockBox {
                            data: BlockBoxData::Anonymous,
                            style: b_.style.clone(),
                            // TODO: is it possible to avoid clone here?
                            children: BlockBoxChildren::Inlines(vec![b_.clone()]),
                        },
                        c,
                    ]),
                })
            }
        }
    }

    vec![b]
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Box {
    Block(BlockBox),
    Inline(InlineBox),
}

#[cfg(test)]
mod tests {
    use crate::renderer::{
        css::parser::parse_css_stylesheet,
        html::{parser::HtmlParser, token::HtmlTokenizer},
        layout::layout_view::get_style_content,
    };

    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_simple() {
        let html = r#"<!doctype html><html><head></head><body><a>inline1 inline1 inline1</a>inline2 inline2 inline2<a>inline3 inline3 inline3</a><p>block4 block4 block4</p><p>block5 block5 block5</p>inline6 inline6 inline6</body></html>"#;
        let expected = BlockBox {
            data: BlockBoxData::Element(Element::new(ElementKind::Body)),
            style: ComputedStyle {
                display: Some(DisplayType::Block),
            },
            children: BlockBoxChildren::Blocks(vec![
                BlockBox {
                    data: BlockBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![
                        InlineBox {
                            data: InlineBoxData::Element(Element::new(ElementKind::A)),
                            style: ComputedStyle {
                                display: Some(DisplayType::Inline),
                            },
                            text: Some("inline1 inline1 inline1".into()),
                            children: vec![],
                        },
                        InlineBox {
                            data: InlineBoxData::Anonymous,
                            style: ComputedStyle {
                                display: Some(DisplayType::Inline),
                            },
                            text: Some("inline1 inline1 inline1".into()),
                            children: vec![],
                        },
                        InlineBox {
                            data: InlineBoxData::Element(Element::new(ElementKind::A)),
                            style: ComputedStyle {
                                display: Some(DisplayType::Inline),
                            },
                            text: Some("inline3 inline3 inline3".into()),
                            children: vec![],
                        },
                    ]),
                },
                BlockBox {
                    data: BlockBoxData::Element(Element::new(ElementKind::P)),
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![InlineBox {
                        data: InlineBoxData::Anonymous,
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("block4 block4 block4".into()),
                        children: vec![],
                    }]),
                },
                BlockBox {
                    data: BlockBoxData::Element(Element::new(ElementKind::P)),
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![InlineBox {
                        data: InlineBoxData::Anonymous,
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("block4 block4 block4".into()),
                        children: vec![],
                    }]),
                },
                BlockBox {
                    data: BlockBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![InlineBox {
                        data: InlineBoxData::Anonymous,
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("inline6 inline6 inline6".into()),
                        children: vec![],
                    }]),
                },
            ]),
        };

        let t = HtmlTokenizer::new(html.into());
        let window = HtmlParser::new(t).construct_tree();
        let dom = window.borrow().document();
        let style = get_style_content(dom.clone());
        let cssom = parse_css_stylesheet(style);
        let layout_view = LayoutView::layout(dom, &cssom);
        let actual = construct_box_tree(layout_view);
        assert_eq!(expected, actual);
    }

    #[ignore]
    #[test]
    fn test_block_inside_inline() {
        let html = r#"<a href="https://app.example">before div<p>inside div</p>after div</a>"#;
        let expected = BlockBox {
            data: BlockBoxData::Element(Element::new(ElementKind::Body)),
            style: ComputedStyle {
                display: Some(DisplayType::Block),
            },
            children: BlockBoxChildren::Blocks(vec![
                BlockBox {
                    data: BlockBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![InlineBox {
                        data: InlineBoxData::Element(Element::new(ElementKind::A)),
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("before p".into()),
                        children: vec![],
                    }]),
                },
                BlockBox {
                    data: BlockBoxData::Element(Element::new(ElementKind::P)),
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![InlineBox {
                        data: InlineBoxData::Anonymous,
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("inside p".into()),
                        children: vec![],
                    }]),
                },
                BlockBox {
                    data: BlockBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Block),
                    },
                    children: BlockBoxChildren::Inlines(vec![InlineBox {
                        data: InlineBoxData::Element(Element::new(ElementKind::A)),
                        style: ComputedStyle {
                            display: Some(DisplayType::Inline),
                        },
                        text: Some("after p".into()),
                        children: vec![],
                    }]),
                },
            ]),
        };
    }
}
