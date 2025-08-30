use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-punctuators
    Punctuator(char),
    /// https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-literals-numeric-literals
    Number(u64),
    /// <https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-identifier-names>
    Identifier(String),
    /// <https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-keywords-and-reserved-words>
    Keyword(Keyword),
    /// <https://tc39.es/ecma262/multipage/ecmascript-language-lexical-grammar.html#sec-literals-string-literals>
    StringLiteral(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Var,
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

    fn peek_char(&self) -> Option<char> {
        if self.pos < self.input.len() {
            Some(self.input[self.pos])
        } else {
            None
        }
    }

    fn consume_char(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.pos += 1;
        Some(c)
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

    fn try_consume_keyword(&mut self) -> Option<Keyword> {
        let keyword = "var";
        for (i, c) in keyword.chars().enumerate() {
            if self.input[self.pos + i] != c {
                return None;
            }
        }
        self.pos += keyword.len();
        Some(Keyword::Var)
    }

    fn consume_identifier(&mut self) -> String {
        let mut id = String::new();

        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                id.push(c);
                self.consume_char();
            } else {
                break;
            }
        }

        id
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

        if let Some(keyword) = self.try_consume_keyword() {
            return Some(Token::Keyword(keyword));
        }

        let c = self.input[self.pos];

        let token = match c {
            '+' | '-' | ';' | '=' | '(' | ')' | '{' | '}' | ',' | '.' => {
                let t = Token::Punctuator(c);
                self.pos += 1;
                t
            }
            '0'..='9' => self.consume_number(),
            c if c.is_alphabetic() || c == '_' || c == '$' => {
                Token::Identifier(self.consume_identifier())
            }
            _ => unimplemented!("char '{c}' is not supported yet"),
        };

        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(input: &str) -> Vec<Token> {
        let mut lexer = JsLexer::new(input.into());
        let tokens: Vec<_> = lexer.collect();
        tokens
    }

    #[test]
    fn test_add_nums() {
        let input = "1 + 2";
        let tokens = tokenize(input);

        dbg!(&tokens);
    }

    #[test]
    fn test_assign_variable() {
        let input = "var foo = 1;";
        let tokens = tokenize(input);

        dbg!(&tokens);
    }
}
