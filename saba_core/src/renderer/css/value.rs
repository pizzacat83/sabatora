use alloc::string::String;

/// <https://www.w3.org/TR/css-values-4/#component-types>
///
/// This type represents the syntax of CSS values,
/// defined in <https://www.w3.org/TR/css-values-4/>.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValue {
    /// <https://www.w3.org/TR/css-values-4/#keywords>
    Keyword(String),
}
