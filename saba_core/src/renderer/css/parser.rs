use super::token::CssTokenizer;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheetParser {
    t: CssTokenizer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    rules: Vec<QualifiedRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: Vec<SimpleBlock>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    PreservedTokens, // TODO
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleBlock {
    value: Vec<ComponentValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_one_rule() {
    //     let style= "p { color: red; }".to_string();
    //     let t = CssParser::new(style);
    //     let cssom = CssParser::new(t).parse_stylesheet();

    //     let expected = [
    //         cssom::QualifiedRule{

    //         }
    //     ]
    // }
}
