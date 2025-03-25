//! <https://www.w3.org/TR/CSS2/visuren.html>
//! <https://www.w3.org/TR/css-display-3/>

use crate::renderer::dom::node::{Element, ElementKind};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::{
    computed_style::{ComputedStyle, DisplayType},
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
    BlockBox {
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
    }
}

#[cfg(test)]
mod tests {
    use crate::renderer::{
        css::parser::parse_css_stylesheet,
        html::{parser::HtmlParser, token::HtmlTokenizer},
        layout::layout_view::get_style_content,
    };

    use super::*;

    #[test]
    fn test_simple() {
        let html = r#"<!doctype html><html><head></head><body><a>inline1 inline1 inline1</a>inline2 inline2 inline2<a>inline3 inline3 inline3<p>block4 block4 block4</p><p>block5 block5 block5</p>inline6 inline6 inline6</body></html>"#;
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
