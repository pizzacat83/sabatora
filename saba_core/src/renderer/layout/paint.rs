use crate::display_item::{self, DisplayItem};
use crate::renderer::layout::computed_style::{ComputedStyle, DisplayType};
use crate::renderer::layout::layout_object::LayoutPoint;
use alloc::vec;
use alloc::vec::Vec;

use super::position::{
    PositionedBlockBox, PositionedBlockBoxChildren, PositionedInlineBox, PositionedLineBox,
};

pub fn paint(tree: PositionedBlockBox) -> Vec<DisplayItem> {
    paint_block(tree, &LayoutPoint { x: 0, y: 0 })
}

fn paint_block(tree: PositionedBlockBox, left_top: &LayoutPoint) -> Vec<DisplayItem> {
    match tree.children {
        PositionedBlockBoxChildren::Empty => Vec::new(),
        PositionedBlockBoxChildren::Blocks(blocks) => {
            let mut left_top = left_top.clone();
            let mut display_items = Vec::new();

            for block in blocks {
                let block_height = block.size.height;
                display_items.extend(paint_block(block, &left_top));
                // Blocks stack vertically.
                left_top = LayoutPoint {
                    x: left_top.x,
                    y: left_top.y + block_height,
                }
            }
            display_items
        }
        PositionedBlockBoxChildren::Inlines(lines) => {
            let mut left_top = left_top.clone();
            let mut display_items = Vec::new();

            for line in lines {
                let line_height = line.size.height;
                display_items.extend(paint_line(line, &left_top));
                // Lines stack vertically.
                left_top = LayoutPoint {
                    x: left_top.x,
                    y: left_top.y + line_height,
                }
            }
            display_items
        }
    }
}

fn paint_line(line: PositionedLineBox, left_top: &LayoutPoint) -> Vec<DisplayItem> {
    let mut left_top = left_top.clone();
    let mut display_item = Vec::new();

    for inline in line.children {
        let inline_width = inline.size.width;
        display_item.extend(paint_inline(inline, &left_top));
        // Inline items stack horizontally.
        left_top = LayoutPoint {
            x: left_top.x + inline_width,
            y: left_top.y,
        }
    }

    display_item
}

fn paint_inline(inline: PositionedInlineBox, left_top: &LayoutPoint) -> Vec<DisplayItem> {
    let mut display_item = Vec::new();
    if let Some(text) = inline.text {
        display_item.push(DisplayItem::Text {
            text,
            style: inline.style,
            layout_point: left_top.clone(),
        });
        assert!(inline.children.is_empty());
    }

    let mut left_top = left_top.clone();
    for inline in inline.children {
        let inline_width = inline.size.width;
        display_item.extend(paint_inline(inline, &left_top));
        // Inline items stack horizontally.
        left_top = LayoutPoint {
            x: left_top.x + inline_width,
            y: left_top.y,
        };
    }

    display_item
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
