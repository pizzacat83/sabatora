use alloc::string::String;

/// <https://www.w3.org/TR/css-values-4/#component-types>
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValue {
    /// <https://www.w3.org/TR/css-values-4/#keywords>
    Keyword(String),
}
