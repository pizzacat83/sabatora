use core::panic;

use alloc::{string::String, vec, vec::Vec};

use super::attribute::Attribute;

/// The output of the tokenization step is a series of zero or more of the following tokens: DOCTYPE, start tag, end tag, comment, character, end-of-file.
/// https://html.spec.whatwg.org/multipage/parsing.html#data-state:~:text=The%20output%20of%20the%20tokenization%20step%20is%20a%20series%20of%20zero%20or%20more%20of%20the%20following%20tokens%3A%20DOCTYPE%2C%20start%20tag%2C%20end%20tag%2C%20comment%2C%20character%2C%20end%2Dof%2Dfile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    StartTag {
        tag: String,
        self_closing: bool,
        attributes: Vec<Attribute>,
    },
    EndTag {
        tag: String,
    },
    Char(char),
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state: State,
    pos: usize,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state: State::Data,
            pos: 0,
            latest_token: None,
            input: html.chars().collect(),
            buf: String::new(),
        }
    }

    fn consume_next_input(&mut self) -> Option<char> {
        if self.pos >= self.input.len() {
            return None;
        }
        let c = self.input[self.pos];
        self.pos += 1;
        Some(c)
    }

    fn create_tag(&mut self, start_tag_token: bool) {
        if start_tag_token {
            self.latest_token = Some(HtmlToken::StartTag {
                tag: String::new(),
                self_closing: false,
                attributes: Vec::new(),
            });
        } else {
            self.latest_token = Some(HtmlToken::EndTag { tag: String::new() });
        }
    }

    fn append_tag_name(&mut self, c: char) {
        match self.latest_token {
            Some(
                HtmlToken::StartTag {
                    ref mut tag,
                    self_closing: _,
                    attributes: _,
                }
                | HtmlToken::EndTag { ref mut tag },
            ) => {
                tag.push(c);
            }
            _ => panic!("append_tag_name: latest_token is not StartTag or EndTag"),
        }
    }

    fn take_latest_token(&mut self) -> Option<HtmlToken> {
        assert!(self.latest_token.is_some());
        let token = self.latest_token.clone();
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        token
    }

    fn reconsume(&mut self) {
        assert!(self.pos > 0);
        self.pos -= 1;
    }
}

// https://html.spec.whatwg.org/multipage/parsing.html#parse-state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    /// https://html.spec.whatwg.org/multipage/parsing.html#data-state
    Data,
    /// https://html.spec.whatwg.org/multipage/parsing.html#tag-open-state
    TagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#end-tag-open-state
    EndTagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#tag-name-state
    TagName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-name-state
    BeforeAttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-name-state
    AttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-name-state
    AfterAttributeName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#before-attribute-value-state
    BeforeAttributeValue,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(double-quoted)-state
    AttributeValueDoubleQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(single-quoted)-state
    AttributeValueSingleQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#attribute-value-(unquoted)-state
    AttributeValueUnquoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#after-attribute-value-(quoted)-state
    AfterAttributeValueQuoted,
    /// https://html.spec.whatwg.org/multipage/parsing.html#self-closing-start-tag-state
    SelfClosingStartTag,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    ScriptData,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-state
    ScriptDataLessThanSign,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    ScriptDataEndTagOpen,
    /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name
    ScriptDataEndTagName,
    /// https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
    TemporaryBuffer,
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;
    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.input.len() {
            return None;
        }
        loop {
            match self.state {
                State::Data => {
                    let c = self.consume_next_input();
                    match c {
                        Some('&') => {
                            todo!()
                        }
                        Some('<') => {
                            self.state = State::TagOpen;
                        }
                        None => {
                            return Some(HtmlToken::Eof);
                        }
                        Some(c) => {
                            return Some(HtmlToken::Char(c));
                        }
                    }
                }
                State::TagOpen => {
                    let c = self.consume_next_input();
                    match c {
                        Some('/') => {
                            self.state = State::EndTagOpen;
                        }
                        Some(c) if c.is_ascii_alphabetic() => {
                            self.create_tag(true);
                            self.reconsume();
                            self.state = State::TagName;
                        }
                        Some('?') => {
                            todo!()
                        }
                        Some(_) => {
                            self.reconsume();
                            return Some(HtmlToken::Char('<'));
                        }
                        None => {
                            // return vec![HtmlToken::Char('<'), HtmlToken::Eof];
                            return Some(HtmlToken::Char('<'));
                        }
                    }
                }
                State::EndTagOpen => {
                    let c = self.consume_next_input();
                    match c {
                        Some(c) if c.is_ascii_alphabetic() => {
                            self.create_tag(false);
                            self.reconsume();
                            self.state = State::TagName;
                        }
                        Some('>') => {
                            self.state = State::Data;
                        }
                        None => {
                            // TODO: </ EOF
                            return Some(HtmlToken::Eof);
                        }
                        Some(_) => todo!(),
                    }
                }
                State::TagName => {
                    let c = self.consume_next_input();
                    match c {
                        Some('>') => {
                            self.state = State::Data;
                            return self.take_latest_token();
                        }
                        None => {
                            return Some(HtmlToken::Eof);
                        }
                        Some(c) => {
                            // todo
                            self.append_tag_name(c);
                        }
                    }
                }

                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::{string::ToString, vec};

    use super::*;

    #[test]
    fn test_empty() {
        let html = "".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        assert_eq!(None, tokenizer.next());
    }

    #[test]
    fn test_start_and_end_tag() {
        let html = "<body></body>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "body".to_string(),
                self_closing: false,
                attributes: vec![],
            },
            HtmlToken::EndTag {
                tag: "body".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }
}
