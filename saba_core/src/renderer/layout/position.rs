use super::{
    box_tree::{BlockBox, BlockBoxChildren, BlockBoxData, InlineBoxData},
    computed_style::ComputedStyle,
    layout_object::{LayoutPoint, LayoutSize},
    line::{split_inline_box, LineBox},
};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedBlockBox {
    pub data: BlockBoxData,
    pub style: ComputedStyle,
    pub children: PositionedBlockBoxChildren,
    pub size: LayoutSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositionedBlockBoxChildren {
    Empty,
    Inlines(Vec<PositionedLineBox>),
    Blocks(Vec<PositionedBlockBox>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedLineBox {
    pub children: Vec<PositionedInlineBox>,
    pub size: LayoutSize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedInlineBox {
    pub data: InlineBoxData,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub children: Vec<PositionedInlineBox>,
    pub size: LayoutSize,
}

pub fn position(tree: BlockBox, viewport_size: LayoutSize) -> PositionedBlockBox {
    // 1. Width of blocks are always that of viewport.
    // 2. Inline boxes can be fragmented into lines, based on that width.
    // 3. Width & Height of each inline box can be determined from bottom to up.
    // 4. Height of each block box can be determined from bottom to up.

    position_block_box(tree, viewport_size.width)
}

fn position_block_box(tree: BlockBox, width: i64) -> PositionedBlockBox {
    let children = match tree.children {
        BlockBoxChildren::Empty => PositionedBlockBoxChildren::Empty,
        BlockBoxChildren::Blocks(blocks) => PositionedBlockBoxChildren::Blocks(
            blocks
                .into_iter()
                .map(|block| position_block_box(block, width))
                .collect(),
        ),
        BlockBoxChildren::Inlines(inlines) => {
            let lines = split_inline_box(inlines, width);
            let positioned = lines.into_iter().map(position_line).collect();
            PositionedBlockBoxChildren::Inlines(positioned)
        }
    };

    let height = height_of_block_children(&children);

    PositionedBlockBox {
        data: tree.data,
        style: tree.style,
        children,
        size: LayoutSize { width, height },
    }
}

fn position_line(line: LineBox) -> PositionedLineBox {
    todo!()
}

fn height_of_block_children(children: &PositionedBlockBoxChildren) -> i64 {
    todo!()
}

// fn initialize_positioned_tree(tree: BlockBox, viewport_size: LayoutSize) -> PositionedBlockBox {
//     PositionedBlockBox {
//         data: tree.data,
//         style: tree.style,
//         children: match tree.children {
//             BlockBoxChildren::Empty => PositionedBoxChildren::Empty,
//             BlockBoxChildren::Blocks(blocks) => PositionedBoxChildren::Blocks(
//                 blocks
//                     .into_iter()
//                     .map(|block| initialize_positioned_tree(block, viewport_size))
//                     .collect(),
//             ),
//             BlockBoxChildren::Inlines(inlines) => {
//                 PositionedBoxChildren::Inlines(split_inline_box(inlines, viewport_size.width).map)
//             }
//         },
//         region: Region {
//             left_top: LayoutPoint { x: 0, y: 0 },
//             size: LayoutSize {
//                 width: viewport_size.width,
//                 height: 0,
//             },
//         },
//     }
// }
