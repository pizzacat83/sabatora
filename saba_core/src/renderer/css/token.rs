use alloc::string::String;
use alloc::vec::Vec;

/// <https://www.w3.org/TR/css-syntax-3/#token-diagrams>
#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    /// <https://www.w3.org/TR/css-syntax-3/#hash-token-diagram>
    Hash(String),
    /// <https://www.w3.org/TR/css-syntax-3/#delim-token-diagram>
    Delim(char),
    /// <https://www.w3.org/TR/css-syntax-3/#number-token-diagram>
    Number(f64),
    /// <https://www.w3.org/TR/css-syntax-3/#colon-token-diagram>
    Colon,
    /// <https://www.w3.org/TR/css-syntax-3/#semicolon-token-diagram>
    SemiColon,
    /// <https://www.w3.org/TR/css-syntax-3/#open-paren-token-diagram>
    OpenParenthesis,
    /// <https://www.w3.org/TR/css-syntax-3/#close-paren-token-diagram>
    CloseParenthesis,
    /// <https://www.w3.org/TR/css-syntax-3/#open-curly-token-diagram>
    OpenCurly,
    /// <https://www.w3.org/TR/css-syntax-3/#close-curly-token-diagram>
    CloseCurly,
    /// <https://www.w3.org/TR/css-syntax-3/#ident-token-diagram>
    Ident(String),
    /// <https://www.w3.org/TR/css-syntax-3/#string-token-diagram>
    String(String),
    /// <https://www.w3.org/TR/css-syntax-3/#at-keyword-token-diagram>
    AtKeyword(String),
    /// <https://www.w3.org/TR/css-syntax-3/#whitespace-token-diagram>
    Whitespace,
}

/// <https://www.w3.org/TR/css-syntax-3/#tokenization>
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl CssTokenizer {
    /// Creates a new tokenizer for the given CSS input
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }
}

impl Iterator for CssTokenizer {
    type Item = CssToken;

    fn next(&mut self) -> Option<Self::Item> {
        self.consume_token()
    }
}

impl CssTokenizer {
    /// <https://www.w3.org/TR/css-syntax-3/#consume-token>
    fn consume_token(&mut self) -> Option<CssToken> {
        self.consume_comments();
        match self.consume_input() {
            Some(c) if c.is_ascii_whitespace() => {
                self.consume_whitespaces();
                Some(CssToken::Whitespace)
            }
            Some('"') => Some(self.consume_string_token()),
            Some(':') => Some(CssToken::Colon),
            Some(';') => Some(CssToken::SemiColon),
            Some('{') => Some(CssToken::OpenCurly),
            Some('}') => Some(CssToken::CloseCurly),
            Some('.') => {
                // TODO: If the input stream starts with a number, reconsume the current input code point, consume a numeric token, and return it.
                Some(CssToken::Delim('.'))
            }
            Some('#') => {
                // TODO: Otherwise, return a <delim-token> with its value set to the current input code point.
                let ident = self.consume_ident_sequence();
                Some(CssToken::Hash(ident))
            }
            Some(c) if c.is_ascii_digit() => {
                self.reconsume_input();
                Some(self.consume_numeric_token())
            }
            Some(c) if is_ident_start_code_point(c) => {
                self.reconsume_input();
                Some(self.consume_ident_like_token())
            }
            None => None,
            _ => unimplemented!(),
        }
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-comments>
    fn consume_comments(&mut self) {
        // TODO: implement
    }

    fn consume_input(&mut self) -> Option<char> {
        if self.pos < self.input.len() {
            let c = self.input[self.pos];
            self.pos += 1;
            Some(c)
        } else {
            None
        }
    }

    fn reconsume_input(&mut self) {
        assert!(self.pos > 0);
        self.pos -= 1;
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-an-ident-like-token>
    fn consume_ident_like_token(&mut self) -> CssToken {
        let string = self.consume_ident_sequence();
        // TODO: handle url, function
        CssToken::Ident(string)
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-name>
    fn consume_ident_sequence(&mut self) -> String {
        let mut result = String::new();
        loop {
            match self.consume_input() {
                Some(c) if is_ident_code_point(c) => {
                    result.push(c);
                }
                // TODO: handle escape
                _ => {
                    self.reconsume_input();
                    return result;
                }
            }
        }
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-token> (step 2)
    fn consume_whitespaces(&mut self) {
        loop {
            match self.consume_input() {
                Some(c) if c.is_ascii_whitespace() => {}
                _ => {
                    self.reconsume_input();
                    break;
                }
            }
        }
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-string-token>
    fn consume_string_token(&mut self) -> CssToken {
        let ending_code_point = self.current_input();
        let mut string = String::new();
        loop {
            match self.consume_input() {
                Some(c) if c == ending_code_point => {
                    return CssToken::String(string);
                }
                None => {
                    return CssToken::String(string);
                }
                // TODO: handle escape
                Some(c) => {
                    string.push(c);
                }
            }
        }
    }

    fn current_input(&self) -> char {
        assert!(self.pos > 0);
        self.input[self.pos - 1]
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-numeric-token>
    fn consume_numeric_token(&mut self) -> CssToken {
        let number = self.consume_number();
        // TODO: handle dimension
        CssToken::Number(number)
    }

    /// <https://www.w3.org/TR/css-syntax-3/#consume-a-number>
    fn consume_number(&mut self) -> f64 {
        let mut repr = String::new();
        loop {
            match self.consume_input() {
                Some(c) if c.is_ascii_digit() => {
                    repr.push(c);
                }

                _ => {
                    self.reconsume_input();
                    break;
                }
            };
        }
        // TODO: handle fraction
        repr.parse().unwrap()
    }
}

/// <https://www.w3.org/TR/css-syntax-3/#check-if-three-code-points-would-start-an-identifier>
fn is_ident_start_code_point(c: char) -> bool {
    c.is_ascii_alphabetic() || !c.is_ascii() || c == '_'
}

/// <https://www.w3.org/TR/css-syntax-3/#ident-code-point>
fn is_ident_code_point(c: char) -> bool {
    is_ident_start_code_point(c) || c.is_ascii_digit() || c == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_rule() {
        let style = "p { color: red; }".to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::Whitespace,
            CssToken::OpenCurly,
            CssToken::Whitespace,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Whitespace,
            CssToken::Ident("red".to_string()),
            CssToken::SemiColon,
            CssToken::Whitespace,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert_eq!(None, t.next());
    }

    #[test]
    fn test_multiple_rules() {
        let style = r#"p { content: "Hey"; } h1 { font-size: 40; color: blue; }"#.to_string();
        let mut t = CssTokenizer::new(style);
        let expected = [
            CssToken::Ident("p".to_string()),
            CssToken::Whitespace,
            CssToken::OpenCurly,
            CssToken::Whitespace,
            CssToken::Ident("content".to_string()),
            CssToken::Colon,
            CssToken::Whitespace,
            CssToken::String("Hey".to_string()),
            CssToken::SemiColon,
            CssToken::Whitespace,
            CssToken::CloseCurly,
            CssToken::Whitespace,
            CssToken::Ident("h1".to_string()),
            CssToken::Whitespace,
            CssToken::OpenCurly,
            CssToken::Whitespace,
            CssToken::Ident("font-size".to_string()),
            CssToken::Colon,
            CssToken::Whitespace,
            CssToken::Number(40.0),
            CssToken::SemiColon,
            CssToken::Whitespace,
            CssToken::Ident("color".to_string()),
            CssToken::Colon,
            CssToken::Whitespace,
            CssToken::Ident("blue".to_string()),
            CssToken::SemiColon,
            CssToken::Whitespace,
            CssToken::CloseCurly,
        ];
        for e in expected {
            assert_eq!(Some(e.clone()), t.next());
        }
        assert_eq!(None, t.next());
    }
}
