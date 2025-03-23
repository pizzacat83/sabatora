#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComputedStyle {
    pub display: Option<DisplayType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisplayType {
    Block,
    Inline,
    None,
}
