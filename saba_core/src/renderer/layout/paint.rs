use crate::display_item::DisplayItem;
use crate::renderer::layout::computed_style::{ComputedStyle, DisplayType};
use crate::renderer::layout::layout_object::LayoutPoint;
use alloc::vec;
use alloc::vec::Vec;

use super::position::PositionedBlockBox;

pub fn paint(tree: PositionedBlockBox) -> Vec<DisplayItem> {
    // TODO
    vec![
        DisplayItem::Text {
            text: "inline1 inline1 inline1".into(),
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            layout_point: LayoutPoint { x: 0, y: 0 },
        },
        DisplayItem::Text {
            text: "inline2 inline2 inline2".into(),
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            layout_point: LayoutPoint { x: 8 * 23, y: 0 },
        },
        DisplayItem::Text {
            text: "inline3 inline3 inline3".into(),
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            layout_point: LayoutPoint {
                x: 8 * 23 * 2,
                y: 0,
            },
        },
        DisplayItem::Text {
            text: "block4 block4 block4".into(),
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            layout_point: LayoutPoint { x: 0, y: 16 },
        },
        DisplayItem::Text {
            text: "block5 block5 block5".into(),
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            layout_point: LayoutPoint { x: 0, y: 16 * 2 },
        },
        DisplayItem::Text {
            text: "inline6 inline6 inline6".into(),
            style: ComputedStyle {
                display: Some(DisplayType::Inline),
            },
            layout_point: LayoutPoint { x: 0, y: 16 * 3 },
        },
    ]
}

#[cfg(test)]
mod tests {
    use crate::display_item::DisplayItem;
    use crate::renderer::dom::node::{Element, ElementKind};
    use crate::renderer::layout::computed_style::{ComputedStyle, DisplayType};
    use crate::renderer::layout::layout_object::LayoutPoint;

    use super::super::box_tree::*;
    use super::*;

    #[test]
    fn test_simple() {
        #[test]
        fn test_simple() {
            // let box_tree = BlockBox {
            //     data: BlockBoxData::Element(Element::new(ElementKind::Body)),
            //     style: ComputedStyle {
            //         display: Some(DisplayType::Block),
            //     },
            //     children: BlockBoxChildren::Blocks(vec![
            //         BlockBox {
            //             data: BlockBoxData::Anonymous,
            //             style: ComputedStyle {
            //                 display: Some(DisplayType::Block),
            //             },
            //             children: BlockBoxChildren::Inlines(vec![
            //                 InlineBox {
            //                     data: InlineBoxData::Element(Element::new(ElementKind::A)),
            //                     style: ComputedStyle {
            //                         display: Some(DisplayType::Inline),
            //                     },
            //                     text: Some("inline1 inline1 inline1".into()),
            //                     children: vec![],
            //                 },
            //                 InlineBox {
            //                     data: InlineBoxData::Anonymous,
            //                     style: ComputedStyle {
            //                         display: Some(DisplayType::Inline),
            //                     },
            //                     text: Some("inline1 inline1 inline1".into()),
            //                     children: vec![],
            //                 },
            //                 InlineBox {
            //                     data: InlineBoxData::Element(Element::new(ElementKind::A)),
            //                     style: ComputedStyle {
            //                         display: Some(DisplayType::Inline),
            //                     },
            //                     text: Some("inline3 inline3 inline3".into()),
            //                     children: vec![],
            //                 },
            //             ]),
            //         },
            //         BlockBox {
            //             data: BlockBoxData::Element(Element::new(ElementKind::P)),
            //             style: ComputedStyle {
            //                 display: Some(DisplayType::Block),
            //             },
            //             children: BlockBoxChildren::Inlines(vec![InlineBox {
            //                 data: InlineBoxData::Anonymous,
            //                 style: ComputedStyle {
            //                     display: Some(DisplayType::Inline),
            //                 },
            //                 text: Some("block4 block4 block4".into()),
            //                 children: vec![],
            //             }]),
            //         },
            //         BlockBox {
            //             data: BlockBoxData::Element(Element::new(ElementKind::P)),
            //             style: ComputedStyle {
            //                 display: Some(DisplayType::Block),
            //             },
            //             children: BlockBoxChildren::Inlines(vec![InlineBox {
            //                 data: InlineBoxData::Anonymous,
            //                 style: ComputedStyle {
            //                     display: Some(DisplayType::Inline),
            //                 },
            //                 text: Some("block4 block4 block4".into()),
            //                 children: vec![],
            //             }]),
            //         },
            //         BlockBox {
            //             data: BlockBoxData::Anonymous,
            //             style: ComputedStyle {
            //                 display: Some(DisplayType::Block),
            //             },
            //             children: BlockBoxChildren::Inlines(vec![InlineBox {
            //                 data: InlineBoxData::Anonymous,
            //                 style: ComputedStyle {
            //                     display: Some(DisplayType::Inline),
            //                 },
            //                 text: Some("inline6 inline6 inline6".into()),
            //                 children: vec![],
            //             }]),
            //         },
            //     ]),
            // };

            // // TODO: lines should be wrapped
            // let expected = vec![
            //     DisplayItem::Text {
            //         text: "inline1 inline1 inline1".into(),
            //         style: ComputedStyle {
            //             display: Some(DisplayType::Inline),
            //         },
            //         layout_point: LayoutPoint { x: 0, y: 0 },
            //     },
            //     DisplayItem::Text {
            //         text: "inline2 inline2 inline2".into(),
            //         style: ComputedStyle {
            //             display: Some(DisplayType::Inline),
            //         },
            //         layout_point: LayoutPoint { x: 8 * 23, y: 0 },
            //     },
            //     DisplayItem::Text {
            //         text: "inline3 inline3 inline3".into(),
            //         style: ComputedStyle {
            //             display: Some(DisplayType::Inline),
            //         },
            //         layout_point: LayoutPoint {
            //             x: 8 * 23 * 2,
            //             y: 0,
            //         },
            //     },
            //     DisplayItem::Text {
            //         text: "block4 block4 block4".into(),
            //         style: ComputedStyle {
            //             display: Some(DisplayType::Inline),
            //         },
            //         layout_point: LayoutPoint { x: 16, y: 0 },
            //     },
            //     DisplayItem::Text {
            //         text: "block5 block5 block5".into(),
            //         style: ComputedStyle {
            //             display: Some(DisplayType::Inline),
            //         },
            //         layout_point: LayoutPoint { x: 16 * 2, y: 0 },
            //     },
            //     DisplayItem::Text {
            //         text: "inline6 inline6 inline6".into(),
            //         style: ComputedStyle {
            //             display: Some(DisplayType::Inline),
            //         },
            //         layout_point: LayoutPoint { x: 16 * 3, y: 0 },
            //     },
            // ];

            // let actual = paint(box_tree);
            // assert_eq!(expected, actual);
        }
    }
}
