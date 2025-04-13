//! <https://www.w3.org/TR/CSS2/visuren.html>
//! <https://www.w3.org/TR/css-display-3/>

use core::{cell::RefCell, mem};

use crate::renderer::dom::node::{Element, ElementKind, Node, NodeData};
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
pub struct BlockBoxBlockChildrenBuilder {
    committed_children: Vec<BlockBox>,
    uncommitted_inlines: Option<Vec<InlineBox>>,
    style: ComputedStyle,
}

impl BlockBoxBlockChildrenBuilder {
    fn build_from_boxes(children: Vec<Box>, style: ComputedStyle) -> Vec<BlockBox> {
        let mut builder = Self::new(style);

        for child in children {
            match child {
                Box::Block(b) => builder.push_block(b.clone()),
                Box::Inline(inline) => builder.push_inline(inline.clone()),
            }
        }

        builder.build()
    }

    fn new(style: ComputedStyle) -> Self {
        Self {
            committed_children: Vec::new(),
            uncommitted_inlines: None,
            style,
        }
    }

    fn push_block(&mut self, block: BlockBox) {
        self.commit_inlines();
        self.committed_children.push(block)
    }

    fn commit_inlines(&mut self) {
        if let Some(inlines) = self.uncommitted_inlines.take() {
            self.committed_children.push(BlockBox {
                data: BlockBoxData::Anonymous,
                style: self.style.clone(),
                children: BlockBoxChildren::Inlines(inlines),
            });
        }
    }

    fn push_inline(&mut self, inline: InlineBox) {
        if let Some(ref mut inlines) = self.uncommitted_inlines {
            inlines.push(inline);
        } else {
            self.uncommitted_inlines = Some(vec![inline]);
        }
    }

    fn build(mut self) -> Vec<BlockBox> {
        self.commit_inlines();
        self.committed_children
    }
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

    // If the object is block, it produces a block box.
    //   If all children are inline boxes, children is an inline box.
    //   If some children are block boxes, children is a list of block boxes.
    //     Adjacent inline boxes are wrapped in an anonymous block box.
    // If the object is inline,
    //   If all children are inline boxes, it produces an inline box.
    //   If some children are block boxes, it produces an anonymous block box containing the inline segments before the block, the block, and inline segments after the block.
    let b = match object.style.display {
        Some(DisplayType::Block) => {
            let children = object
                .children()
                .flat_map(produce_boxes)
                .collect::<Vec<_>>();

            let children = if children.is_empty() {
                BlockBoxChildren::Empty
            } else if children.iter().any(|c| matches!(c, Box::Block(_))) {
                BlockBoxChildren::Blocks(BlockBoxBlockChildrenBuilder::build_from_boxes(
                    children,
                    object.style.clone(),
                ))
            } else {
                BlockBoxChildren::Inlines(
                    children
                        .into_iter()
                        .map(|c| match c {
                            Box::Inline(c) => c,
                            _ => unreachable!(),
                        })
                        .collect(),
                )
            };

            Box::Block(BlockBox {
                data: match &object.node.borrow().data {
                    NodeData::Element(element) => BlockBoxData::Element(element.clone()),
                    NodeData::Text(_) => unreachable!(),
                    NodeData::Document => unreachable!(),
                },
                style: object.style.clone(),
                children,
            })
        }
        Some(DisplayType::Inline) => 'block: {
            let children = object.children().collect::<Vec<_>>();
            if children.len() == 1 {
                if let NodeData::Text(text) = &children[0].borrow().node.borrow().data {
                    let data = match &object.node.borrow().data {
                        NodeData::Element(element) => InlineBoxData::Element(element.clone()),
                        _ => unreachable!(),
                    };
                    break 'block Box::Inline(InlineBox {
                        data,
                        style: object.style.clone(),
                        text: Some(text.clone()),
                        children: Vec::new(),
                    });
                }
            }

            let children = children
                .into_iter()
                .flat_map(produce_boxes)
                .collect::<Vec<_>>();

            if children.iter().any(|c| matches!(c, Box::Block(_))) {
                // If some children are block boxes, it produces an anonymous block box containing the inline segments before the block, the block, and inline segments after the block.
                Box::Block(BlockBox {
                    data: BlockBoxData::Anonymous,
                    style: object.style.clone(),
                    children: BlockBoxChildren::Blocks(
                        BlockBoxBlockChildrenBuilder::build_from_boxes(
                            children,
                            object.style.clone(),
                        ),
                    ),
                })
            } else {
                Box::Inline(match &object.node.borrow().data {
                    NodeData::Element(element) => InlineBox {
                        data: InlineBoxData::Element(element.clone()),
                        style: object.style.clone(),
                        text: None,
                        children: children
                            .into_iter()
                            .map(|c| match c {
                                Box::Inline(c) => c,
                                _ => unreachable!(),
                            })
                            .collect(),
                    },
                    NodeData::Text(text) => InlineBox {
                        data: InlineBoxData::Anonymous,
                        style: object.style.clone(),
                        text: Some(text.clone()),
                        children: vec![],
                    },
                    NodeData::Document => unreachable!(),
                })
            }
        }
        Some(DisplayType::None) => unreachable!(),
        None => unreachable!(),
    };

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
                            text: Some("inline2 inline2 inline2".into()),
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
                        text: Some("block5 block5 block5".into()),
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
