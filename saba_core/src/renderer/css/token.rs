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
            Some(':') => Some(CssToken::Colon),
            Some(';') => Some(CssToken::SemiColon),
            Some('{') => Some(CssToken::OpenCurly),
            Some('}') => Some(CssToken::CloseCurly),
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
                Some(c) if is_ident_start_code_point(c) => {
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
}

fn is_ident_start_code_point(c: char) -> bool {
    c.is_ascii_alphabetic() || !c.is_ascii() || c == '_'
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
}
