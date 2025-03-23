use alloc::rc::Rc;
use alloc::vec::Vec;
use core::cell::RefCell;

use crate::renderer::page::Page;

#[derive(Debug, Clone)]
pub struct Browser {
    active_page_index: usize,

    // TODO: does this need to be Rc<RefCell<T>> ?
    pages: Vec<Rc<RefCell<Page>>>,
}

impl Browser {
    pub fn new() -> Rc<RefCell<Self>> {
        let mut page = Page::new();

        let browser = Rc::new(RefCell::new(Self {
            active_page_index: 0,
            pages: Vec::new(),
        }));

        page.set_browser(Rc::downgrade(&browser));
        browser.borrow_mut().pages.push(Rc::new(RefCell::new(page)));

        browser
    }

    pub fn current_page(&self) -> Rc<RefCell<Page>> {
        Rc::clone(&self.pages[self.active_page_index])
    }
}
