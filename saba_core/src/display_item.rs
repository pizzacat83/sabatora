use crate::renderer::layout::computed_style::ComputedStyle;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayItem {
    Rect {
        style: ComputedStyle,
        layout_point: LayoutPoint,
        layout_size: LayoutSize,
    },
    Text {
        text: String,
        style: ComputedStyle,
        layout_point: LayoutPoint,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutPoint {
    pub x: i64,
    pub y: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutSize {
    // TODO
}
