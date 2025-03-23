use alloc::rc::{Rc, Weak};
use core::cell::RefCell;

use crate::browser::Browser;

use super::dom::node::Window;

#[derive(Debug, Clone)]
pub struct Page {
    browser: Weak<RefCell<Browser>>,
    frame: Option<Rc<RefCell<Window>>>,
}
impl Page {
    pub(crate) fn new() -> Self {
        Self {
            browser: Weak::new(),
            frame: None,
        }
    }

    pub(crate) fn set_browser(&mut self, browser: Weak<RefCell<Browser>>) {
        self.browser = browser;
    }
}
