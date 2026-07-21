// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    Add,
    Sub,
    Mul,
    Div,
    Equals,
    Clear,
    Dot,
    Negate,
}

#[derive(Debug)]
pub struct App {
    pub display: String,
    pub val: i32,
    pub prev: i32,
    pub op: String,
    pub fresh: bool,
    pub expr: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            display: "0".to_string(),
            val: 0,
            prev: 0,
            op: "".to_string(),
            fresh: true,
            expr: "".to_string(),
        }
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = AppMsg;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            AppMsg::Digit8 => {
                if self.fresh { self.display = "8".to_string(); self.val = 8 } else { self.display = format!("{}{}", self.display, "8"); self.val = self.val * 10 + 8 };
                self.fresh = false
            }
            AppMsg::Equals => {
                if self.op != "".to_string() { /* unhandled stmt */; self.expr = format!("{}{}", format!("{}{}", format!("{}{}", self.expr, " "), self.display), " ="); /* unhandled stmt */; if self.op == "+".to_string() { self.val = self.prev + self.val }; if self.op == "-".to_string() { self.val = self.prev - self.val }; if self.op == "*".to_string() { self.val = self.prev * self.val }; if self.op == "/".to_string() { if self.val != 0 { self.val = self.prev / self.val } }; self.display = self.val.to_string(); self.prev = 0; self.op = "".to_string(); self.fresh = true }
            }
            AppMsg::Digit5 => {
                if self.fresh { self.display = "5".to_string(); self.val = 5 } else { self.display = format!("{}{}", self.display, "5"); self.val = self.val * 10 + 5 };
                self.fresh = false
            }
            AppMsg::Digit1 => {
                if self.fresh { self.display = "1".to_string(); self.val = 1 } else { self.display = format!("{}{}", self.display, "1"); self.val = self.val * 10 + 1 };
                self.fresh = false
            }
            AppMsg::Digit2 => {
                if self.fresh { self.display = "2".to_string(); self.val = 2 } else { self.display = format!("{}{}", self.display, "2"); self.val = self.val * 10 + 2 };
                self.fresh = false
            }
            AppMsg::Digit6 => {
                if self.fresh { self.display = "6".to_string(); self.val = 6 } else { self.display = format!("{}{}", self.display, "6"); self.val = self.val * 10 + 6 };
                self.fresh = false
            }
            AppMsg::Digit9 => {
                if self.fresh { self.display = "9".to_string(); self.val = 9 } else { self.display = format!("{}{}", self.display, "9"); self.val = self.val * 10 + 9 };
                self.fresh = false
            }
            AppMsg::Sub => {
                if self.op == "+".to_string() { self.val = self.prev + self.val };
                if self.op == "-".to_string() { self.val = self.prev - self.val };
                if self.op == "*".to_string() { self.val = self.prev * self.val };
                if self.op == "/".to_string() { if self.val != 0 { self.val = self.prev / self.val } };
                if self.op != "".to_string() { self.display = self.val.to_string() };
                self.expr = format!("{}{}", self.display, " -");
                self.prev = self.val;
                self.op = "-".to_string();
                self.fresh = true
            }
            AppMsg::Negate => {
                self.val = 0 - self.val
            }
            AppMsg::Digit3 => {
                if self.fresh { self.display = "3".to_string(); self.val = 3 } else { self.display = format!("{}{}", self.display, "3"); self.val = self.val * 10 + 3 };
                self.fresh = false
            }
            AppMsg::Digit4 => {
                if self.fresh { self.display = "4".to_string(); self.val = 4 } else { self.display = format!("{}{}", self.display, "4"); self.val = self.val * 10 + 4 };
                self.fresh = false
            }
            AppMsg::Dot => {
                /* unhandled stmt */
            }
            AppMsg::Digit7 => {
                if self.fresh { self.display = "7".to_string(); self.val = 7 } else { self.display = format!("{}{}", self.display, "7"); self.val = self.val * 10 + 7 };
                self.fresh = false
            }
            AppMsg::Mul => {
                if self.op == "+".to_string() { self.val = self.prev + self.val };
                if self.op == "-".to_string() { self.val = self.prev - self.val };
                if self.op == "*".to_string() { self.val = self.prev * self.val };
                if self.op == "/".to_string() { if self.val != 0 { self.val = self.prev / self.val } };
                if self.op != "".to_string() { self.display = self.val.to_string() };
                self.expr = format!("{}{}", self.display, " *");
                self.prev = self.val;
                self.op = "*".to_string();
                self.fresh = true
            }
            AppMsg::Div => {
                if self.op == "+".to_string() { self.val = self.prev + self.val };
                if self.op == "-".to_string() { self.val = self.prev - self.val };
                if self.op == "*".to_string() { self.val = self.prev * self.val };
                if self.op == "/".to_string() { if self.val != 0 { self.val = self.prev / self.val } };
                if self.op != "".to_string() { self.display = self.val.to_string() };
                self.expr = format!("{}{}", self.display, " /");
                self.prev = self.val;
                self.op = "/".to_string();
                self.fresh = true
            }
            AppMsg::Clear => {
                self.display = "0".to_string();
                self.val = 0;
                self.prev = 0;
                self.op = "".to_string();
                self.fresh = true;
                self.expr = "".to_string()
            }
            AppMsg::Add => {
                if self.op == "+".to_string() { self.val = self.prev + self.val };
                if self.op == "-".to_string() { self.val = self.prev - self.val };
                if self.op == "*".to_string() { self.val = self.prev * self.val };
                if self.op == "/".to_string() { if self.val != 0 { self.val = self.prev / self.val } };
                if self.op != "".to_string() { self.display = self.val.to_string() };
                self.expr = format!("{}{}", self.display, " +");
                self.prev = self.val;
                self.op = "+".to_string();
                self.fresh = true
            }
            AppMsg::Digit0 => {
                if self.fresh { self.display = "0".to_string(); self.val = 0 } else { self.display = format!("{}{}", self.display, "0"); self.val = self.val * 10 };
                self.fresh = false
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::center(View::col().w(80).bg("gray-900").style("rounded-2xl overflow-hidden shadow-xl").child(View::col().w_full().p(6).bg("gray-900").style("rounded-t-2xl").child(View::text_styled(format!("{}", self.expr), "text-sm text-gray-400 text-right w-full")).child(View::text_styled(format!("{}", self.display), "text-4xl font-light text-white text-right w-full")).build()).child(View::col().w_full().p(2).bg("gray-800").gap(1).style("rounded-b-2xl").child(View::row().w_full().child(View::button("C").flex1().bg("gray-600").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Clear).build()).child(View::button("+/-").flex1().bg("gray-600").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Negate).build()).child(View::button("%").flex1().bg("gray-600").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Dot).build()).child(View::button("/").flex1().bg("orange-500").text_color("white").rounded_lg().p(4).font_bold().style("text-lg").on_click(|_| AppMsg::Div).build()).build()).child(View::row().w_full().child(View::button("7").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit7).build()).child(View::button("8").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit8).build()).child(View::button("9").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit9).build()).child(View::button("*").flex1().bg("orange-500").text_color("white").rounded_lg().p(4).font_bold().style("text-lg").on_click(|_| AppMsg::Mul).build()).build()).child(View::row().w_full().child(View::button("4").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit4).build()).child(View::button("5").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit5).build()).child(View::button("6").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit6).build()).child(View::button("-").flex1().bg("orange-500").text_color("white").rounded_lg().p(4).font_bold().style("text-lg").on_click(|_| AppMsg::Sub).build()).build()).child(View::row().w_full().child(View::button("1").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit1).build()).child(View::button("2").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit2).build()).child(View::button("3").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit3).build()).child(View::button("+").flex1().bg("orange-500").text_color("white").rounded_lg().p(4).font_bold().style("text-lg").on_click(|_| AppMsg::Add).build()).build()).child(View::row().w_full().child(View::button("0").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Digit0).build()).child(View::button(".").flex1().bg("gray-700").text_color("white").rounded_lg().p(4).style("text-lg").on_click(|_| AppMsg::Dot).build()).child(View::button("=").flex1().bg("orange-500").text_color("white").rounded_lg().p(4).font_bold().style("text-lg").on_click(|_| AppMsg::Equals).build()).build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("calculator");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
