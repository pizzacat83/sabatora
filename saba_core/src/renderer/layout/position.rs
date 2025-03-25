use super::{
    box_tree::{BlockBoxData, InlineBoxData},
    computed_style::ComputedStyle,
    layout_object::{LayoutPoint, LayoutSize},
};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedBlockBox {
    pub data: BlockBoxData,
    pub style: ComputedStyle,
    pub children: PositionedBoxChildren,
    pub region: Region,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PositionedBoxChildren {
    Empty,
    Inlines(Vec<PositionedLineBox>),
    Blocks(Vec<PositionedBlockBox>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedLineBox {
    pub children: Vec<PositionedInlineBox>,
    pub region: Region,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedInlineBox {
    pub data: InlineBoxData,
    pub style: ComputedStyle,
    pub text: Option<String>,
    pub children: Vec<PositionedInlineBox>,
    pub region: Region,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    pub left_top: LayoutPoint,
    pub size: LayoutSize,
}
