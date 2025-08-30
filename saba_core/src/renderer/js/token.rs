use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-punctuators
    Punctuator(char),
    /// https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-literals-numeric-literals
    Number(u64),
}

pub struct JsLexer {
    pos: usize,
    input: Vec<char>,
}

impl JsLexer {
    pub fn new(js: String) -> Self {
        Self {
            pos: 0,
            input: js.chars().collect(),
        }
    }

    fn consume_number(&mut self) -> Token {
        let mut num = 0;
        loop {
            if self.pos >= self.input.len() {
                return Token::Number(num);
            }

            let c = self.input[self.pos];
            match c {
                '0'..='9' => {
                    num = num * 10 + (c.to_digit(10).unwrap() as u64);
                    self.pos += 1;
                }
                _ => break,
            }
        }
        return Token::Number(num);
    }
}

impl Iterator for JsLexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }

        // skip whitespace
        // TODO: But in JS, a newline can terminate a sentence, right?
        while self.input[self.pos] == ' ' || self.input[self.pos] == '\n' {
            self.pos += 1;

            if self.pos >= self.input.len() {
                return None;
            }
        }

        let c = self.input[self.pos];

        let token = match c {
            '+' | '-' | ';' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                t
            }
            '0'..='9' => self.consume_number(),
            _ => unimplemented!("char '{c}' is not supported yet"),
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_nums() {
        let input = "1 + 2".to_string();
        let mut lexer = JsLexer::new(input);
        let tokens: Vec<_> = lexer.collect();

        dbg!(&tokens);
    }
}
