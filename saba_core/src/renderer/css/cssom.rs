use alloc::string::String;
use alloc::vec::Vec;

use super::value::ComponentValue;

/// <https://www.w3.org/TR/cssom-1/#cssstylesheet>
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleSheet {
    pub css_rules: Vec<CssRule>,
}

// for simplicity
type CssRule = CssStyleRule;
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleRule {
    pub selector: SelectorList,
    pub declarations: CssStyleDeclaration,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleDeclaration {
    pub declarations: Vec<CssDeclaration>,
}

/// <https://www.w3.org/TR/cssom-1/#css-declaration>
#[derive(Debug, Clone, PartialEq)]
pub struct CssDeclaration {
    property_name: String,
    value: Vec<ComponentValue>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectorList {
    pub selectors: Vec<ComplexSelector>,
}

/// <https://www.w3.org/TR/selectors-4/#complex>
/// A complex selector is a sequence of one or more compound selectors separated by combinators.
#[derive(Debug, Clone, PartialEq)]
pub enum ComplexSelector {
    CompoundSelector(CompoundSelector), // TODO: support combinators
}

/// <https://www.w3.org/TR/selectors-4/#compound>
/// A compound selector is a sequence of simple selectors that are not separated by a combinator
#[derive(Debug, Clone, PartialEq)]
pub struct CompoundSelector(pub Vec<SimpleSelector>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    TypeSelector(String),
    ClassSelector(String),
    IdSelector(String),
    // TODO: support more simple selectors
}
