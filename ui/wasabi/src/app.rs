use core::cell::RefCell;

use alloc::format;
use alloc::rc::Rc;
use alloc::vec;
use noli::window::StringSize;
use noli::window::Window;
use saba_core::display_item::DisplayItem;
use saba_core::renderer::layout::computed_style::{ComputedStyle, DisplayType};
use saba_core::renderer::layout::layout_object::LayoutPoint;
use saba_core::renderer::layout::layout_object::LayoutSize;
use saba_core::{browser::Browser, error::Error};

#[derive(Debug)]
pub struct WasabiUI {
    browser: Rc<RefCell<Browser>>,
    window: Window,
}

type Result<T> = core::result::Result<T, Error>;

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

    pub fn start(&mut self) -> Result<()> {
        self.setup()?;

        self.run_app()?;

        self.test_display_page()?;

        Ok(())
    }

    fn setup(&mut self) -> Result<()> {
        // TODO: setup toolbar
        self.window.flush();
        Ok(())
    }

    fn run_app(&mut self) -> Result<()> {
        Ok(())
    }

    /// A method just for development
    fn test_display_page(&mut self) -> Result<()> {
        let html = r#"<!doctype html><html><head></head><body><a>inline1 inline1 inline1</a>inline2 inline2 inline2<a>inline3 inline3 inline3<p>block4 block4 block4</p><p>block5 block5 block5</p>inline6 inline6 inline6</body></html>"#;
        self.browser
            .borrow()
            .current_page()
            .borrow_mut()
            .populate_frame(html.into());

        self.update_ui()?;
        Ok(())
    }

    fn update_ui(&mut self) -> Result<()> {
        let display_items =
            self.browser
                .borrow()
                .current_page()
                .borrow()
                .display_items(LayoutSize {
                    width: CONTENT_AREA_WIDTH,
                    height: CONTENT_AREA_HEIGHT,
                });

        for item in display_items {
            match item {
                DisplayItem::Text {
                    text,
                    style,
                    layout_point,
                } => self
                    .window
                    .draw_string(
                        BLACK, // TODO: use style
                        layout_point.x + WINDOW_PADDING,
                        layout_point.y + WINDOW_PADDING + TOOLBAR_HEIGHT,
                        &text,
                        StringSize::Medium, // TODO: use style
                        false,              // TODO: use style
                    )
                    .map_err(|error| {
                        Error::InvalidUI(format!("failed to draw string: {:?}", error))
                    }),
                _ => {
                    todo!()
                }
            }?;
        }
        self.window.flush();

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
pub const BLACK: u32 = 0x000000;
