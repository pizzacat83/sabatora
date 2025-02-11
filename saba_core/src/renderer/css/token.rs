use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum CssToken {
    Hash(String),
    Delim(char),
    Number(f64),
    Colon,
    SemiColon,
    OpenParenthesis,
    CloseParenthesis,
    OpenCurly,
    CloseCurly,
    Ident(String),
    String(String),
    AtKeyword(String),
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CssTokenizer {
    input: Vec<char>,
    pos: usize,
}

impl CssTokenizer {
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

    fn consume_ident_like_token(&mut self) -> CssToken {
        let string = self.consume_ident_sequence();
        // TODO: handle url, function
        CssToken::Ident(string)
    }

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

    fn consume_numeric_token(&mut self) -> CssToken {
        let number = self.consume_number();
        // TODO: handle dimension
        CssToken::Number(number)
    }

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

fn is_ident_start_code_point(c: char) -> bool {
    c.is_ascii_alphabetic() || !c.is_ascii() || c == '_'
}

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
