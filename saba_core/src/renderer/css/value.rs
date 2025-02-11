use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComponentValue {
    Keyword(String),
}
