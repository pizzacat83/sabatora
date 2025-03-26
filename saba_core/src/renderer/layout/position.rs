use super::{
    box_tree::{BlockBox, BlockBoxChildren, BlockBoxData, InlineBox, InlineBoxData},
    computed_style::ComputedStyle,
    layout_object::{LayoutPoint, LayoutSize},
    line::{split_inline_box, LineBox},
    text::size_of_text,
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
            let positioned = lines
                .into_iter()
                .map(|line| position_line(line, width))
                .collect();
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

fn position_line(line: LineBox, width: i64) -> PositionedLineBox {
    let children = line
        .children
        .into_iter()
        .map(position_inline)
        .collect::<Vec<_>>();
    let LayoutSize { height, .. } = size_of_inline_children(&children[..]);
    PositionedLineBox {
        children,
        size: LayoutSize { width, height },
    }
}

fn position_inline(inline: InlineBox) -> PositionedInlineBox {
    let children = inline
        .children
        .into_iter()
        .map(position_inline)
        .collect::<Vec<_>>();

    let size = if let Some(text) = &inline.text {
        size_of_text(text)
    } else {
        size_of_inline_children(&children[..])
    };

    PositionedInlineBox {
        children,
        size,
        data: inline.data,
        style: inline.style,
        text: inline.text,
    }
}

fn size_of_inline_children(children: &[PositionedInlineBox]) -> LayoutSize {
    LayoutSize {
        width: children.iter().map(|c| c.size.width).sum(),
        height: children.iter().map(|c| c.size.height).max().unwrap_or(0),
    }
}

fn height_of_block_children(children: &PositionedBlockBoxChildren) -> i64 {
    use PositionedBlockBoxChildren::*;
    match children {
        Empty => 0,
        Blocks(blocks) => blocks.iter().map(|c| c.size.height).sum(),
        Inlines(lines) => lines.iter().map(|c| c.size.height).sum(),
    }
}
