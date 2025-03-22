use super::cssom;
use super::token::{CssToken, CssTokenizer};
use alloc::string::String;
use alloc::{vec, vec::Vec};

mod css_parser {
    use crate::renderer::css::cssom::CssDeclaration;

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

    /// <https://www.w3.org/TR/css-syntax-3/#parse-a-stylesheet>
    fn parse_style_rule(rule: &QualifiedRule) -> cssom::CssStyleRule {
        cssom::CssStyleRule {
            selector: parse_selector_list(&rule.prelude),
            declarations: parse_style_block_contents(&rule.block),
        }
    }

    /// <https://www.w3.org/TR/css-syntax-3/#parse-a-style-blocks-contents>
    fn parse_style_block_contents(block: &SimpleBlock) -> cssom::CssStyleDeclaration {
        cssom::CssStyleDeclaration {
            declarations: DeclarationsParser::parse(block),
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

    struct DeclarationsParser {
        values: Vec<ComponentValue>,
        pos: usize,
    }

    impl DeclarationsParser {
        /// <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-declarations>
        pub fn parse(block: &SimpleBlock) -> Vec<cssom::CssDeclaration> {
            let mut parser = Self {
                values: block.value.clone(),
                pos: 0,
            };
            let mut declarations = Vec::new();

            loop {
                match parser.consume_next_value() {
                    None => {
                        break;
                    }
                    Some(t) => {
                        let ComponentValue::PreservedToken(t) = t;
                        use CssToken::*;
                        match t {
                            Whitespace | SemiColon => {}
                            Ident(_) => {
                                let mut list = vec![t.clone()];
                                loop {
                                    match parser.peek_next_value() {
                                        None => {
                                            break;
                                        }
                                        Some(t) => {
                                            let ComponentValue::PreservedToken(t) = t;
                                            match t {
                                                SemiColon => {
                                                    break;
                                                }
                                                _ => {
                                                    list.push(t.clone());
                                                }
                                            }
                                        }
                                    }
                                    parser.consume_next_value();
                                }
                                if let Some(d) = DeclarationParser::parse(list) {
                                    declarations.push(d);
                                }
                            }
                            _ => unimplemented!(),
                        }
                    }
                }
            }

            declarations
        }

        fn consume_next_value(&mut self) -> Option<ComponentValue> {
            if self.pos < self.values.len() {
                let v = &self.values[self.pos];
                self.pos += 1;
                Some(v.clone())
            } else {
                None
            }
        }

        fn peek_next_value(&mut self) -> Option<&ComponentValue> {
            if self.pos < self.values.len() {
                Some(&self.values[self.pos])
            } else {
                None
            }
        }
    }

    struct DeclarationParser {
        tokens: Vec<CssToken>,
        pos: usize,
    }

    impl DeclarationParser {
        pub fn parse(tokens: Vec<CssToken>) -> Option<CssDeclaration> {
            let mut parser = DeclarationParser { tokens, pos: 0 };
            parser.consume_declaration()
        }

        fn consume_next_token(&mut self) -> Option<CssToken> {
            if self.pos < self.tokens.len() {
                let v = &self.tokens[self.pos];
                self.pos += 1;
                Some(v.clone())
            } else {
                None
            }
        }

        fn peek_next_token(&mut self) -> Option<&CssToken> {
            if self.pos < self.tokens.len() {
                Some(&self.tokens[self.pos])
            } else {
                None
            }
        }

        /// <https://www.w3.org/TR/css-syntax-3/#consume-a-declaration>
        fn consume_declaration(&mut self) -> Option<cssom::CssDeclaration> {
            let property_name = match self.consume_next_token() {
                Some(CssToken::Ident(id)) => id,
                // Note: This algorithm assumes that the next input token has already been checked to be an <ident-token>.
                _ => unreachable!(),
            };

            let mut declaration = cssom::CssDeclaration {
                property_name,
                value: Vec::new(),
            };

            while matches!(self.peek_next_token(), Some(&CssToken::Whitespace),) {
                self.consume_next_token();
            }

            assert_eq!(self.consume_next_token(), Some(CssToken::Colon),);

            while matches!(self.peek_next_token(), Some(&CssToken::Whitespace),) {
                self.consume_next_token();
            }

            let mut declaration_value = Vec::new();

            while let Some(t) = self.consume_next_token() {
                declaration_value.push(t);
            }

            // TODO: handle !important

            while let Some(CssToken::Whitespace) = declaration_value.last() {
                declaration_value.pop();
            }

            declaration.value = parse_value(declaration_value);

            Some(declaration)
        }
    }

    fn parse_value(declaration_value: Vec<CssToken>) -> Vec<super::super::value::ComponentValue> {
        if declaration_value.len() == 1 {
            if let CssToken::Ident(id) = &declaration_value[0] {
                return vec![super::super::value::ComponentValue::Keyword(id.into())];
            }
        }
        unimplemented!("not supported value: {declaration_value:#?}")
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

    /// <https://www.w3.org/TR/css-syntax-3/#reconsume-the-current-input-token>
    fn reconsume(&mut self, t: CssToken) {
        assert!(self.reconsumed.is_none());
        self.reconsumed = Some(t)
    }

    /// <https://www.w3.org/TR/css-syntax-3/#parse-a-stylesheet>
    pub fn parse_stylesheet(&mut self) -> StyleSheet {
        let rules = self.consume_list_of_rules();
        StyleSheet { rules }
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-list-of-rules>
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

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-qualified-rule>
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

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-simple-block>
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

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-component-value>
    fn consume_component_value(&mut self) -> ComponentValue {
        match self.consume_next_input_token() {
            None => unimplemented!(),
            Some(t) => ComponentValue::PreservedToken(t),
        }
    }
}

/// <https://www.w3.org/TR/css-syntax-3/#stylesheet-diagram>
#[derive(Debug, Clone, PartialEq)]
pub struct StyleSheet {
    rules: Vec<QualifiedRule>,
}

/// <https://www.w3.org/TR/css-syntax-3/#qualified-rule>
#[derive(Debug, Clone, PartialEq)]
pub struct QualifiedRule {
    prelude: Vec<ComponentValue>,
    block: SimpleBlock,
}

/// <https://www.w3.org/TR/css-syntax-3/#component-value>
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValue {
    /// <https://www.w3.org/TR/css-syntax-3/#preserved-tokens>
    PreservedToken(CssToken),
}

/// <https://www.w3.org/TR/css-syntax-3/#simple-block>
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
                        declarations: vec![CssDeclaration {
                            property_name: "color".into(),
                            value: vec![super::super::value::ComponentValue::Keyword("red".into())],
                        }],
                    },
                }],
            }
        };

        assert_eq!(expected, parsed);
    }
}
