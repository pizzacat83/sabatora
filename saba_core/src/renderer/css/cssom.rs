use alloc::vec::Vec;

/// <https://www.w3.org/TR/cssom-1/#cssstylesheet>
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleSheet {
    css_rules: Vec<CssRule>,
}

// for simplicity
type CssRule = CssStyleRule;
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleRule {
    selector: Selectors,
    declarations: CssStyleDeclaration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleDeclaration {
    declarations: Vec<CssDeclaration>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssDeclaration {
    property_name: String,
    value: Vec<ComponentValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Selectors {
    selectors: Vec<Selector>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Selector {}
