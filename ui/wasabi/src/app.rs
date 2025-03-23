use core::cell::RefCell;

use alloc::rc::Rc;
use noli::window::Window;
use saba_core::{browser::Browser, error::Error};

#[derive(Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    window: Window,
}

impl WasabiUI {
    pub fn new(browser: Rc<RefCell<Browser>>) -> Self {
        Self {
            browser,
            window: Window::new(
                "sabatora".into(),
                WHITE,
                WINDOW_INIT_X_POS,
                WINDOW_INIT_Y_POS,
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )
            .unwrap(),
        }
    }

    pub fn start(&mut self) -> Result<(), Error> {
        self.setup()?;

        self.run_app()?;

        Ok(())
    }

    fn setup(&mut self) -> Result<(), Error> {
        // TODO: setup toolbar
        self.window.flush();
        Ok(())
    }

    fn run_app(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub const WINDOW_WIDTH: i64 = 600;
pub const WINDOW_HEIGHT: i64 = 400;
pub const WINDOW_PADDING: i64 = 5;

pub const TITLE_BAR_HEIGHT: i64 = 24;
pub const TOOLBAR_HEIGHT: i64 = 26;

pub const CONTENT_AREA_WIDTH: i64 = WINDOW_WIDTH - WINDOW_PADDING * 2;
pub const CONTENT_AREA_HEIGHT: i64 =
    WINDOW_HEIGHT - TITLE_BAR_HEIGHT - TOOLBAR_HEIGHT - WINDOW_PADDING * 2;

pub const CHAR_WIDTH: i64 = 8;
pub const CHAR_HEIGHT: i64 = 16;
pub const CHAR_HEIGHT_WITH_PADDING: i64 = CHAR_HEIGHT + 4;

pub const WINDOW_INIT_X_POS: i64 = 30;
pub const WINDOW_INIT_Y_POS: i64 = 50;

pub const WHITE: u32 = 0xffffff;
