// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    Inc,
    Dec,
    Reset,
}

#[derive(Debug)]
pub struct App {
    pub count: i32,
}

impl App {
    pub fn new() -> Self {
        Self {
            count: 0,
        }
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = Msg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Msg::Dec => {
                self.count = self.count - 1
            }
            Msg::Reset => {
                self.count = 0
            }
            Msg::Inc => {
                self.count = self.count + 1
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().w_full().h_full().justify_center().items_center().child(View::text(format!("Counter: {}", self.count))).child(View::row().child(View::button("-").on_click(|_| Msg::Dec).build()).child(View::button("Reset").on_click(|_| Msg::Reset).build()).child(View::button("+").on_click(|_| Msg::Inc).build()).build()).build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<App>();
    }
    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("counter");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
