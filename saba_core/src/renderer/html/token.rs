use core::panic;

use alloc::{
    string::{String, ToString},
    vec,
    vec::Vec,
};

use super::attribute::Attribute;

/// The output of the tokenization step is a series of zero or more of the following tokens: DOCTYPE, start tag, end tag, comment, character, end-of-file.
/// https://html.spec.whatwg.org/multipage/parsing.html#data-state:~:text=The%20output%20of%20the%20tokenization%20step%20is%20a%20series%20of%20zero%20or%20more%20of%20the%20following%20tokens%3A%20DOCTYPE%2C%20start%20tag%2C%20end%20tag%2C%20comment%2C%20character%2C%20end%2Dof%2Dfile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HtmlToken {
    DoctypeTag {
        name: Option<String>,
    },
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
struct HtmlTokenizeStateMachine {
    state: State,
    pos: usize,
    latest_token: Option<HtmlToken>,
    input: Vec<char>,
    buf: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlTokenizer {
    state_machine: HtmlTokenizeStateMachine,
    eof_observed: bool,
    yielded_tokens: Vec<HtmlToken>,
}
impl HtmlTokenizer {
    pub fn new(html: String) -> Self {
        Self {
            state_machine: HtmlTokenizeStateMachine::new(html),
            eof_observed: false,
            yielded_tokens: Vec::new(),
        }
    }

    fn take_remaining_token(&mut self) -> Option<HtmlToken> {
        if !self.yielded_tokens.is_empty() {
            return Some(self.yielded_tokens.remove(0));
        }
        None
    }
}
impl HtmlTokenizeStateMachine {
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

    fn try_consume_const_insensitive(&mut self, s: &str) -> bool {
        for (i, c) in s.chars().enumerate() {
            if self.pos + i >= self.input.len()
                || self.input[self.pos + i].to_ascii_lowercase() != c.to_ascii_lowercase()
            {
                return false;
            }
        }
        self.pos += s.len();
        true
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

    fn take_latest_token(&mut self) -> HtmlToken {
        assert!(self.latest_token.is_some());
        let token = self.latest_token.clone().unwrap();
        self.latest_token = None;
        assert!(self.latest_token.is_none());

        token
    }

    fn reconsume(&mut self) {
        assert!(self.pos > 0);
        self.pos -= 1;
    }

    fn set_self_closing_flag(&mut self) {
        if let Some(HtmlToken::StartTag {
            tag: _,
            ref mut self_closing,
            attributes: _,
        }) = &mut self.latest_token
        {
            *self_closing = true;
        } else {
            panic!("set_self_closing_flag: latest_token is not StartTag");
        }
    }

    fn start_new_attribute(&mut self) {
        if let Some(HtmlToken::StartTag {
            tag: _,
            self_closing: _,
            ref mut attributes,
        }) = &mut self.latest_token
        {
            attributes.push(Attribute::empty());
        } else {
            panic!("start_new_attribute: latest_token is not StartTag");
        }
    }

    fn append_attribute_name(&mut self, c: char) {
        if let Some(HtmlToken::StartTag {
            tag: _,
            self_closing: _,
            ref mut attributes,
        }) = &mut self.latest_token
        {
            assert!(!attributes.is_empty());
            let attribute = attributes.last_mut().unwrap();
            attribute.append_name_char(c);
        }
    }

    fn append_attribute_value(&mut self, c: char) {
        if let Some(HtmlToken::StartTag {
            tag: _,
            self_closing: _,
            ref mut attributes,
        }) = &mut self.latest_token
        {
            assert!(!attributes.is_empty());
            let attribute = attributes.last_mut().unwrap();
            attribute.append_value_char(c);
        } else {
            panic!("append_attribute_value: latest_token is not StartTag");
        }
    }

    fn create_doctype(&mut self, c: char) {
        self.latest_token = Some(HtmlToken::DoctypeTag {
            name: Some(c.to_string()),
        });
    }

    fn append_doctype_name(&mut self, c: char) {
        if let Some(HtmlToken::DoctypeTag {
            name: Some(ref mut name),
        }) = &mut self.latest_token
        {
            name.push(c);
        } else {
            panic!("append_doctype_name: latest_token is not DoctypeTag");
        }
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
    MarkupDeclarationOpen,
    Doctype,
    BeforeDoctypeName,
    DoctypeName,
    // TODO
    // /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-state
    // ScriptData,
    // /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-less-than-state
    // ScriptDataLessThanSign,
    // /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-open-state
    // ScriptDataEndTagOpen,
    // /// https://html.spec.whatwg.org/multipage/parsing.html#script-data-end-tag-name
    // ScriptDataEndTagName,
    // /// https://html.spec.whatwg.org/multipage/parsing.html#temporary-buffer
    // TemporaryBuffer,
}

impl Iterator for HtmlTokenizer {
    type Item = HtmlToken;
    fn next(&mut self) -> Option<Self::Item> {
        if self.eof_observed {
            return None;
        }
        loop {
            if let Some(token) = self.take_remaining_token() {
                if token == HtmlToken::Eof {
                    self.eof_observed = true;
                }
                return Some(token);
            }
            if let Some(tokens) = self.state_machine.step() {
                self.yielded_tokens = tokens;
            }
        }
    }
}
impl HtmlTokenizeStateMachine {
    fn step(&mut self) -> Option<Vec<HtmlToken>> {
        match self.state {
            State::Data => {
                let c = self.consume_next_input();
                match c {
                    Some('&') => {
                        todo!()
                    }
                    Some('<') => {
                        self.state = State::TagOpen;
                        None
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(c) => Some(vec![HtmlToken::Char(c)]),
                }
            }
            State::TagOpen => {
                let c = self.consume_next_input();
                match c {
                    Some('!') => {
                        self.state = State::MarkupDeclarationOpen;
                        None
                    }
                    Some('/') => {
                        self.state = State::EndTagOpen;
                        None
                    }
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.create_tag(true);
                        self.reconsume();
                        self.state = State::TagName;
                        None
                    }
                    Some('?') => {
                        todo!()
                    }
                    Some(_) => {
                        self.reconsume();
                        Some(vec![HtmlToken::Char('<')])
                    }
                    None => Some(vec![HtmlToken::Char('<'), HtmlToken::Eof]),
                }
            }
            State::EndTagOpen => {
                let c = self.consume_next_input();
                match c {
                    Some(c) if c.is_ascii_alphabetic() => {
                        self.create_tag(false);
                        self.reconsume();
                        self.state = State::TagName;
                        None
                    }
                    Some('>') => {
                        self.state = State::Data;
                        None
                    }
                    None => Some(vec![
                        HtmlToken::Char('<'),
                        HtmlToken::Char('/'),
                        HtmlToken::Eof,
                    ]),
                    Some(_) => todo!(),
                }
            }
            State::TagName => {
                let c = self.consume_next_input();
                match c {
                    Some('\t' | '\n' | '\x0c' | ' ') => {
                        self.state = State::BeforeAttributeName;
                        None
                    }
                    Some('/') => {
                        self.state = State::SelfClosingStartTag;
                        None
                    }
                    Some('>') => {
                        self.state = State::Data;
                        Some(vec![self.take_latest_token()])
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(c) => {
                        // todo
                        self.append_tag_name(c);
                        None
                    }
                }
            }
            State::SelfClosingStartTag => {
                let c = self.consume_next_input();
                match c {
                    Some('>') => {
                        self.set_self_closing_flag();
                        self.state = State::Data;
                        Some(vec![self.take_latest_token()])
                    }
                    _ => todo!(),
                }
            }
            State::BeforeAttributeName => {
                let c = self.consume_next_input();
                match c {
                    Some('\t' | '\n' | '\x0c' | ' ') => {
                        // ignore
                        None
                    }
                    None => {
                        self.state = State::AfterAttributeName;
                        None
                    }
                    Some(_) => {
                        self.state = State::AttributeName;
                        self.reconsume();
                        self.start_new_attribute();
                        None
                    }
                }
            }
            State::AttributeName => {
                let c = self.consume_next_input();
                match c {
                    None => {
                        self.reconsume();
                        self.state = State::AfterAttributeName;
                        None
                    }
                    Some('=') => {
                        self.state = State::BeforeAttributeValue;
                        None
                    }
                    Some(c) => {
                        self.append_attribute_name(c);
                        None
                    }
                }
            }
            State::AfterAttributeName => {
                let c = self.consume_next_input();
                match c {
                    Some('=') => {
                        self.state = State::BeforeAttributeValue;
                        None
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(_) => todo!(),
                }
            }
            State::BeforeAttributeValue => {
                let c = self.consume_next_input();
                match c {
                    Some('"') => {
                        self.state = State::AttributeValueDoubleQuoted;
                        None
                    }
                    Some('\'') => {
                        self.state = State::AttributeValueSingleQuoted;
                        None
                    }

                    Some(_) | None => {
                        self.reconsume();
                        self.state = State::AttributeValueUnquoted;
                        None
                    }
                }
            }
            State::AttributeValueDoubleQuoted => {
                let c = self.consume_next_input();
                match c {
                    Some('"') => {
                        self.state = State::AfterAttributeValueQuoted;
                        None
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(c) => {
                        self.append_attribute_value(c);
                        None
                    }
                }
            }
            State::AttributeValueSingleQuoted => {
                let c = self.consume_next_input();
                match c {
                    Some('\'') => {
                        self.state = State::AfterAttributeValueQuoted;
                        None
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(c) => {
                        self.append_attribute_value(c);
                        None
                    }
                }
            }
            State::AttributeValueUnquoted => {
                let c = self.consume_next_input();
                match c {
                    Some('\t' | '\n' | '\x0c' | ' ') => {
                        self.state = State::BeforeAttributeName;
                        None
                    }
                    Some('>') => {
                        self.state = State::Data;
                        Some(vec![self.take_latest_token()])
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(c) => {
                        self.append_attribute_value(c);
                        None
                    }
                }
            }
            State::AfterAttributeValueQuoted => {
                let c = self.consume_next_input();
                match c {
                    Some('\t' | '\n' | '\x0c' | ' ') => {
                        self.state = State::BeforeAttributeName;
                        None
                    }
                    Some('/') => {
                        self.state = State::SelfClosingStartTag;
                        None
                    }
                    Some('>') => {
                        self.state = State::Data;
                        Some(vec![self.take_latest_token()])
                    }
                    None => Some(vec![HtmlToken::Eof]),
                    Some(_) => {
                        self.reconsume();
                        self.state = State::BeforeAttributeName;
                        None
                    }
                }
            }
            State::MarkupDeclarationOpen => {
                if self.try_consume_const_insensitive("doctype") {
                    self.state = State::Doctype;
                    None
                } else {
                    todo!()
                }
            }
            State::Doctype => {
                let c = self.consume_next_input();
                match c {
                    Some('\t' | '\n' | '\x0c' | ' ') => {
                        self.state = State::BeforeDoctypeName;
                        None
                    }
                    Some(_) => todo!(),
                    None => Some(vec![HtmlToken::DoctypeTag { name: None }, HtmlToken::Eof]),
                }
            }
            State::BeforeDoctypeName => {
                let c = self.consume_next_input();
                match c {
                    Some('\t' | '\n' | '\x0c' | ' ') => None,
                    Some(c) => {
                        self.state = State::DoctypeName;
                        self.create_doctype(c);
                        None
                    }
                    None => Some(vec![HtmlToken::DoctypeTag { name: None }, HtmlToken::Eof]),
                }
            }
            State::DoctypeName => {
                let c = self.consume_next_input();
                match c {
                    Some('>') => {
                        self.state = State::Data;
                        Some(vec![self.take_latest_token()])
                    }
                    None => Some(vec![self.take_latest_token(), HtmlToken::Eof]),
                    Some(c) => {
                        self.append_doctype_name(c);
                        None
                    }
                }
            }
            _ => unimplemented!(),
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
        let expected = [HtmlToken::Eof];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
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

    #[test]
    fn test_self_closing_tag() {
        let html = "<img/>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [HtmlToken::StartTag {
            tag: "img".to_string(),
            self_closing: true,
            attributes: Vec::new(),
        }];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_attributes() {
        let html = r#"<p class="A" id='B' foo=bar></p>"#.to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let attr_class = Attribute::new("class".to_string(), "A".to_string());
        let attr_id = Attribute::new("id".to_string(), "B".to_string());
        let attr_foo = Attribute::new("foo".to_string(), "bar".to_string());
        let expected = [HtmlToken::StartTag {
            tag: "p".to_string(),
            self_closing: false,
            attributes: vec![attr_class, attr_id, attr_foo],
        }];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_script_tag() {
        let html = "<script>alert(1)</script>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::StartTag {
                tag: "script".to_string(),
                self_closing: false,
                attributes: Vec::new(),
            },
            HtmlToken::Char('a'),
            HtmlToken::Char('l'),
            HtmlToken::Char('e'),
            HtmlToken::Char('r'),
            HtmlToken::Char('t'),
            HtmlToken::Char('('),
            HtmlToken::Char('1'),
            HtmlToken::Char(')'),
            HtmlToken::EndTag {
                tag: "script".to_string(),
            },
        ];
        for e in expected {
            assert_eq!(Some(e), tokenizer.next());
        }
    }

    #[test]
    fn test_doctype() {
        let html = "<!doctype html><body></body>".to_string();
        let mut tokenizer = HtmlTokenizer::new(html);
        let expected = [
            HtmlToken::DoctypeTag {
                name: Some("html".to_string()),
            },
            HtmlToken::StartTag {
                tag: "body".to_string(),
                self_closing: false,
                attributes: Vec::new(),
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
