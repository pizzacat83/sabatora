use alloc::string::String;
use alloc::vec::Vec;

use super::value::ComponentValue;

/// <https://www.w3.org/TR/cssom-1/#cssstylesheet>
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleSheet {
    pub css_rules: Vec<CssRule>,
}

// for simplicity
/// <https://www.w3.org/TR/cssom-1/#cssrule>
type CssRule = CssStyleRule;

/// <https://www.w3.org/TR/cssom-1/#cssstylerule>
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleRule {
    pub selector: SelectorList,
    pub declarations: CssStyleDeclaration,
}

/// <https://www.w3.org/TR/cssom-1/#cssstyledeclaration>
#[derive(Debug, Clone, PartialEq)]
pub struct CssStyleDeclaration {
    pub declarations: Vec<CssDeclaration>,
}

/// <https://www.w3.org/TR/cssom-1/#css-declaration>
#[derive(Debug, Clone, PartialEq)]
pub struct CssDeclaration {
    pub property_name: String,

    // <https://www.w3.org/TR/cssom-1/#css-declaration-value>
    // The value of the declaration represented as a list of component values.
    // TODO: is the type correct?
    pub value: Vec<ComponentValue>,
}

/// <https://www.w3.org/TR/selectors-4/#typedef-selector-list>
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

/// <https://www.w3.org/TR/selectors-4/#simple>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelector {
    /// <https://www.w3.org/TR/selectors-4/#type-selector>
    TypeSelector(String),
    /// <https://www.w3.org/TR/selectors-4/#class-selector>
    ClassSelector(String),
    /// <https://www.w3.org/TR/selectors-4/#id-selector>
    IdSelector(String),
    // TODO: support more simple selectors
}
