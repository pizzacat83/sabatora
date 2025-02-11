use super::cssom;
use super::token::{CssToken, CssTokenizer};
use alloc::string::String;
use alloc::{vec, vec::Vec};

mod css_parser {
    use super::super::cssom;
    use super::super::token::{CssToken, CssTokenizer};
    use super::*;
    use alloc::string::String;
    use alloc::{vec, vec::Vec};

    /// <https://www.w3.org/TR/css-syntax-3/#parse-a-css-stylesheet>
    pub fn parse_css_stylesheet(source: String) -> cssom::CssStyleSheet {
        let tokenizer = CssTokenizer::new(source);
        let stylesheet = StyleSheetParser::new(tokenizer).parse_stylesheet();

        let css_rules = stylesheet.rules.iter().map(parse_style_rule).collect();

        cssom::CssStyleSheet { css_rules }
    }

    fn parse_style_rule(rule: &QualifiedRule) -> cssom::CssStyleRule {
        cssom::CssStyleRule {
            selector: parse_selector_list(&rule.prelude),
            declarations: parse_style_block_contents(&rule.block),
        }
    }

    fn parse_style_block_contents(block: &SimpleBlock) -> cssom::CssStyleDeclaration {
        // TODO
        cssom::CssStyleDeclaration {
            declarations: vec![],
        }
    }

    /// <https://www.w3.org/TR/selectors-4/#typedef-selector-list>
    fn parse_selector_list(prelude: &[ComponentValue]) -> cssom::SelectorList {
        let mut selectors = Vec::new();

        for v in prelude {
            let ComponentValue::PreservedToken(v) = v;
            use CssToken::*;
            match v {
                Hash(id) => selectors.push(cssom::SimpleSelector::IdSelector(id.clone())),
                Ident(ty) => selectors.push(cssom::SimpleSelector::TypeSelector(ty.clone())),
                Delim('.') => todo!(".class not implemented yet"),
                _ => {}
            }
        }

        cssom::SelectorList {
            selectors: vec![cssom::ComplexSelector::CompoundSelector(
                cssom::CompoundSelector(selectors),
            )],
        }
    }
}

pub use css_parser::parse_css_stylesheet;

#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheetParser {
    t: CssTokenizer,
    reconsumed: Option<CssToken>,
}

impl StyleSheetParser {
    pub fn new(t: CssTokenizer) -> Self {
        Self {
            t,
            reconsumed: None,
        }
    }

    fn consume_next_input_token(&mut self) -> Option<CssToken> {
        self.reconsumed.take().or_else(|| self.t.next())
    }

    fn reconsume(&mut self, t: CssToken) {
        assert!(self.reconsumed.is_none());
        self.reconsumed = Some(t)
    }

    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        let rules = self.consume_list_of_rules();
        StyleSheet { rules }
    }

    fn consume_list_of_rules(&mut self) -> Vec<QualifiedRule> {
        let mut rules = Vec::new();
        loop {
            match self.consume_next_input_token() {
                None => {
                    break;
                }
                Some(CssToken::Whitespace) => {}
                Some(t) => {
                    self.reconsume(t);
                    rules.push(self.consume_qualified_rule());
                }
            }
        }
        rules
    }

    fn consume_qualified_rule(&mut self) -> QualifiedRule {
        let mut prelude = Vec::new();
        loop {
            match self.consume_next_input_token() {
                None => {
                    unimplemented!();
                }
                Some(CssToken::OpenCurly) => {
                    return QualifiedRule {
                        prelude,
                        block: self.consume_simple_block(CssToken::CloseCurly),
                    }
                }
                Some(t) => {
                    self.reconsume(t);
                    prelude.push(self.consume_component_value());
                }
            }
        }
    }

    fn consume_simple_block(&mut self, ending_token: CssToken) -> SimpleBlock {
        let mut block = SimpleBlock { value: Vec::new() };
        loop {
            match self.consume_next_input_token() {
                None => {
                    unimplemented!();
                }
                Some(t) if t == ending_token => return block,
                Some(t) => {
                    self.reconsume(t);
                    block.value.push(self.consume_component_value());
                }
            }
        }
    }

    fn consume_component_value(&mut self) -> ComponentValue {
        match self.consume_next_input_token() {
            None => unimplemented!(),
            Some(t) => ComponentValue::PreservedToken(t),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    rules: Vec<QualifiedRule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: SimpleBlock,
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
    fn test_stylesheet_one_rule() {
        let style = "p{color:red;}".to_string();
        let t = CssTokenizer::new(style);
        let parsed = StyleSheetParser::new(t).parse_stylesheet();

        let expected = StyleSheet {
            rules: vec![QualifiedRule {
                prelude: vec![ComponentValue::PreservedToken(CssToken::Ident(
                    "p".to_string(),
                ))],
                block: SimpleBlock {
                    value: vec![
                        ComponentValue::PreservedToken(CssToken::Ident("color".to_string())),
                        ComponentValue::PreservedToken(CssToken::Colon),
                        ComponentValue::PreservedToken(CssToken::Ident("red".to_string())),
                        ComponentValue::PreservedToken(CssToken::SemiColon),
                    ],
                },
            }],
        };

        assert_eq!(expected, parsed);
    }

    #[test]
    fn test_css_one_rule() {
        let style = "p{color:red;}".to_string();
        let parsed = parse_css_stylesheet(style);

        let expected = {
            use cssom::*;
            CssStyleSheet {
                css_rules: vec![CssStyleRule {
                    selector: SelectorList {
                        selectors: vec![ComplexSelector::CompoundSelector(CompoundSelector(vec![
                            SimpleSelector::TypeSelector("p".into()),
                        ]))],
                    },
                    declarations: CssStyleDeclaration {
                        declarations: vec![], // TODO
                    },
                }],
            }
        };

        assert_eq!(expected, parsed);
    }
}
