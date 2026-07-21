// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    EmailChanged,
    PasswordChanged,
    Submit,
}

#[derive(Debug)]
pub struct App {
    pub email: String,
    pub password: String,
    pub email_error: String,
    pub password_error: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            email: "".to_string(),
            password: "".to_string(),
            email_error: "".to_string(),
            password_error: "".to_string(),
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
            Msg::Submit => {
                self.email_error = "".to_string()
            }
            Msg::PasswordChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.password = _text;
            }
            Msg::EmailChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.email = _text;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().w_full().h_full().justify_center().items_center().child(View::col().bg("white").border().p(8).w_full().style("rounded-xl shadow-lg border-gray-200 max-w-md mx-auto").child(View::text_styled("Sign In".to_string(), "text-3xl font-extrabold text-gray-900")).child(View::text_styled("Welcome back! Please enter your credentials.".to_string(), "text-gray-500 mt-3")).child(View::col().child(View::text_styled("Email".to_string(), "text-sm font-medium text-gray-700 mt-6")).child(View::input("you@example.com").value(format!("{}", self.email)).w_full().px(3).py(2).border().rounded_lg().style("mt-2").on_change(Msg::EmailChanged).build()).child(if self.email_error != "" { View::text_styled(format!("{}", self.email_error), "text-red-500 text-xs mt-1") } else { View::empty() }).build()).child(View::col().child(View::text_styled("Password".to_string(), "text-sm font-medium text-gray-700 mt-6")).child(View::input("Enter your password").value(format!("{}", self.password)).w_full().px(3).py(2).border().rounded_lg().style("mt-2").on_change(Msg::PasswordChanged).build()).child(if self.password_error != "" { View::text_styled(format!("{}", self.password_error), "text-red-500 text-xs mt-1") } else { View::empty() }).build()).child(View::button("Sign In").w_full().bg("blue-500").text_color("white").rounded_lg().style("py-2.5 font-semibold mt-6").on_click(|_| Msg::Submit).build()).child(View::row().justify_center().items_center().child(View::text_styled("Don't have an account? ".to_string(), "text-sm text-gray-500 mt-4")).child(View::col().text_color("blue-500").px(3).style("text-sm font-semibold underline").build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("login");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
