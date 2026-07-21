// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    GetStarted,
}

#[derive(Debug)]
pub struct App {
}

impl App {
    pub fn new() -> Self {
        Self {
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
            Msg::GetStarted => {
                println!("Getting started!")
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().w_full().h_full().justify_center().items_center().bg("gradient-to-b").text_color("white").p(8).gap(4).style("from-blue-500 to-purple-600").child(View::text("Build Beautiful Apps".to_string())).child(View::text("Write once, run anywhere with AutoLang".to_string())).child(View::button("Get Started").on_click(|_| Msg::GetStarted).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("hero-section");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
