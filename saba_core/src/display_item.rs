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
    // TODO
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutSize {
    // TODO
}
