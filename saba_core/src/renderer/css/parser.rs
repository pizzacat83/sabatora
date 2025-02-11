use super::token::{CssToken, CssTokenizer};
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheetParser {
    t: CssTokenizer,
}

impl StyleSheetParser {
    fn new(t: CssTokenizer) -> Self {
        Self { t }
    }

    fn parse_stylesheet(&self) -> StyleSheet {
        todo!()
    }
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
    PreservedToken(CssToken),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleBlock {
    value: Vec<ComponentValue>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let t = CssTokenizer::new(style);
        let parsed = StyleSheetParser::new(t).parse_stylesheet();

        let expected = StyleSheet {
            rules: vec![QualifiedRule {
                prelude: vec![ComponentValue::PreservedToken(CssToken::Ident(
                    "p".to_string(),
                ))],
                block: vec![SimpleBlock {
                    value: vec![
                        ComponentValue::PreservedToken(CssToken::Ident("color".to_string())),
                        ComponentValue::PreservedToken(CssToken::Colon),
                        ComponentValue::PreservedToken(CssToken::Ident("red".to_string())),
                        ComponentValue::PreservedToken(CssToken::SemiColon),
                    ],
                }],
            }],
        };

        assert_eq!(expected, parsed);
    }
}
