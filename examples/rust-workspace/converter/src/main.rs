// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    CelsiusChanged,
    FahrenheitChanged,
}

#[derive(Debug)]
pub struct App {
    pub celsius: f64,
    pub fahrenheit: f64,
}

impl App {
    pub fn new() -> Self {
        Self {
            celsius: 0.0,
            fahrenheit: 32.0,
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
            Msg::FahrenheitChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.fahrenheit = _text.parse::<f64>().unwrap_or(self.fahrenheit);
                self.celsius = (self.fahrenheit - 32.0) * 5.0 / 9.0
            }
            Msg::CelsiusChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.celsius = _text.parse::<f64>().unwrap_or(self.celsius);
                self.fahrenheit = self.celsius * 9.0 / 5.0 + 32.0
            }
            _ => {}
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().p(6).gap(4).style("max-w-md mx-auto").child(View::text("Temperature Converter".to_string())).child(View::row().child(View::col().child(View::text("Celsius".to_string())).child(View::input("Enter Celsius").value(format!("{}", self.celsius)).on_change(Msg::CelsiusChanged).build()).build()).child(View::col().child(View::text("Fahrenheit".to_string())).child(View::input("Enter Fahrenheit").value(format!("{}", self.fahrenheit)).on_change(Msg::FahrenheitChanged).build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("converter");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
