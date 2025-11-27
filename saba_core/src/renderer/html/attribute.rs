use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attribute {
    pub name: String,
    pub value: String,
}

impl Attribute {
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }

    pub fn empty() -> Self {
        Self {
            name: String::new(),
            value: String::new(),
        }
    }

    pub(crate) fn append_name_char(&mut self, c: char) {
        self.name.push(c);
    }

    pub(crate) fn append_value_char(&mut self, c: char) {
        self.value.push(c);
    }
}
