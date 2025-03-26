use super::layout_object::LayoutSize;

// TODO: consider font size
pub fn size_of_text(text: &str) -> LayoutSize {
    LayoutSize {
        width: CHAR_WIDTH * (text.len() as i64),
        height: CHAR_HEIGHT,
    }
}

const CHAR_WIDTH: i64 = 8;
const CHAR_HEIGHT: i64 = 16;
