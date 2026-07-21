// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Msg {
    NameChanged,
    EmailChanged,
    MessageChanged,
    Submit,
}

#[derive(Debug)]
pub struct App {
    pub name: String,
    pub email: String,
    pub message: String,
    pub submitted: bool,
    pub faq1_q: String,
    pub faq1_a: String,
    pub faq2_q: String,
    pub faq3_q: String,
    pub faq4_q: String,
    pub faq5_q: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            name: "".to_string(),
            email: "".to_string(),
            message: "".to_string(),
            submitted: false,
            faq1_q: "What are your business hours?".to_string(),
            faq1_a: "Our support team is available Monday through Friday, 9am to 6pm EST. For urgent matters outside these hours, please email urgent@company.com.".to_string(),
            faq2_q: "How long does it take to get a response?".to_string(),
            faq3_q: "Do you offer phone support?".to_string(),
            faq4_q: "Can I schedule a demo?".to_string(),
            faq5_q: "What information should I include in my message?".to_string(),
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
                self.submitted = true
            }
            Msg::MessageChanged => {
                self.message = self.message
            }
            Msg::EmailChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.email = _text;
            }
            Msg::NameChanged => {
                let _text = auto_lang::ui::iced::last_input_text();
                self.name = _text;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().w_full().h_full().justify_center().items_center().child(View::col().w_full().p(8).gap(6).bg("white").style("max-w-4xl min-h-screen").child(View::col().gap(2).items_center().child(View::text_styled("How can we help?".to_string(), "text-3xl font-bold text-gray-900")).child(View::text_styled("Check our FAQ for quick answers or send us a message.".to_string(), "text-gray-500")).build()).child(View::row().gap(8).child(View::col().flex1().gap(4).child(View::text_styled("Frequently Asked Questions".to_string(), "text-lg font-semibold text-gray-900")).child(View::col().w_full().child(View::col().py(3).style("border-b border-gray-200").child(View::text_styled(format!("{}", self.faq1_q), "font-medium text-gray-900 text-sm")).child(View::text_styled(format!("{}", self.faq1_a), "text-sm text-gray-500 mt-1")).build()).child(View::col().py(3).style("border-b border-gray-200").child(View::text_styled(format!("{}", self.faq2_q), "font-medium text-gray-900 text-sm")).build()).child(View::col().py(3).style("border-b border-gray-200").child(View::text_styled(format!("{}", self.faq3_q), "font-medium text-gray-900 text-sm")).build()).child(View::col().py(3).style("border-b border-gray-200").child(View::text_styled(format!("{}", self.faq4_q), "font-medium text-gray-900 text-sm")).build()).child(View::col().py(3).child(View::text_styled(format!("{}", self.faq5_q), "font-medium text-gray-900 text-sm")).build()).build()).build()).child(View::col().flex1().gap(4).child(View::text_styled("Still have questions?".to_string(), "text-lg font-semibold text-gray-900")).child(View::col().w_full().gap(4).child(View::col().gap(1).child(View::text_styled("Name *".to_string(), "text-sm font-medium text-gray-700")).child(View::input("Your name").value(format!("{}", self.name)).on_change(Msg::NameChanged).build()).build()).child(View::col().gap(1).child(View::text_styled("Email *".to_string(), "text-sm font-medium text-gray-700")).child(View::input("you@example.com").value(format!("{}", self.email)).on_change(Msg::EmailChanged).build()).build()).child(View::col().gap(1).child(View::text_styled("Message *".to_string(), "text-sm font-medium text-gray-700")).child(View::textarea().on_change(|_| Msg::MessageChanged).build()).build()).child(View::button("Send Message").w_full().bg("gray-900").text_color("white").rounded_lg().style("py-2.5 font-semibold mt-4").on_click(|_| Msg::Submit).build()).build()).build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("contact-form");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
