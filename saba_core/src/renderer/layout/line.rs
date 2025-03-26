//! <https://www.w3.org/TR/CSS2/visuren.html#inline-formatting>
//! <https://developer.mozilla.org/en-US/docs/Web/CSS/CSS_display/Visual_formatting_model#line_boxes>

use crate::renderer::dom::node::{Element, ElementKind};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use super::{
    box_tree::InlineBox,
    computed_style::{ComputedStyle, DisplayType},
    layout_view::LayoutView,
};

/// A box containing contents within a single line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineBox {
    pub children: Vec<InlineBox>,
}

pub fn split_inline_box(tree: Vec<InlineBox>, max_width: i64) -> Vec<LineBox> {
    vec![]
}

#[cfg(test)]
mod tests {
    use crate::renderer::layout::box_tree::InlineBoxData;
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_simple() {
        let tree = vec![InlineBox {
            data: InlineBoxData::Anonymous,
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            text: Some("text text text text text text".into()),
            children: Vec::new(),
        }];

        let expected = vec![
            LineBox {
                children: vec![InlineBox {
                    data: InlineBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Inline),
                    },
                    text: Some("text text text text".into()),
                    children: Vec::new(),
                }],
            },
            LineBox {
                children: vec![InlineBox {
                    data: InlineBoxData::Anonymous,
                    style: ComputedStyle {
                        display: Some(DisplayType::Inline),
                    },
                    text: Some("text text".into()),
                    children: Vec::new(),
                }],
            },
        ];

        let actual = split_inline_box(tree, 8 * (5 * 4 + 1));
        assert_eq!(expected, actual);
    }
}
