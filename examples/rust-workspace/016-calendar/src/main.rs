// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    PrevMonth,
    NextMonth,
}

#[derive(Debug)]
pub struct App {
    pub month: String,
    pub year_display: String,
    pub d1: String,
    pub d2: String,
    pub d3: String,
    pub d4: String,
    pub d5: String,
    pub d6: String,
    pub d7: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            month: "April".to_string(),
            year_display: "2026".to_string(),
            d1: "Su".to_string(),
            d2: "Mo".to_string(),
            d3: "Tu".to_string(),
            d4: "We".to_string(),
            d5: "Th".to_string(),
            d6: "Fr".to_string(),
            d7: "Sa".to_string(),
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
            AppMsg::PrevMonth => {
                self.month = "March".to_string()
            }
            AppMsg::NextMonth => {
                self.month = "May".to_string()
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::center(View::col().style("w-full max-w-md p-6 bg-white rounded-2xl shadow-lg").child(View::row().style("w-full items-center justify-between").child(View::button("<").style("w-10 h-10 rounded-full hover:bg-gray-100 text-gray-600 font-bold").on_click(|_| AppMsg::PrevMonth).build()).child(View::col().style("items-center").child(View::text_styled(format!("{}", self.month), "text-2xl font-bold text-gray-800")).child(View::text_styled(format!("{}", self.year_display), "text-sm text-gray-500")).build()).child(View::button(">").style("w-10 h-10 rounded-full hover:bg-gray-100 text-gray-600 font-bold").on_click(|_| AppMsg::NextMonth).build()).build()).child(View::grid().cols(7).child(View::text_styled(format!("{}", self.d1), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled(format!("{}", self.d2), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled(format!("{}", self.d3), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled(format!("{}", self.d4), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled(format!("{}", self.d5), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled(format!("{}", self.d6), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled(format!("{}", self.d7), "text-xs font-semibold text-gray-400 text-center py-2")).child(View::text_styled("30".to_string(), "text-sm text-gray-300 text-center py-2")).child(View::text_styled("31".to_string(), "text-sm text-gray-300 text-center py-2")).child(View::text_styled("1".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("2".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("3".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("4".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("5".to_string(), "text-sm text-gray-700 text-center py-2 bg-blue-100 rounded-full")).child(View::text_styled("6".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("7".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("8".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("9".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("10".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("11".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("12".to_string(), "text-sm text-gray-700 text-center py-2 bg-blue-100 rounded-full")).child(View::text_styled("13".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("14".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("15".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("16".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("17".to_string(), "text-sm text-gray-700 text-center py-2 bg-blue-100 rounded-full")).child(View::text_styled("18".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("19".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("20".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("21".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("22".to_string(), "text-sm text-gray-700 text-center py-2 bg-blue-100 rounded-full")).child(View::text_styled("23".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("24".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("25".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("26".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("27".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("28".to_string(), "text-sm text-gray-700 text-center py-2 bg-blue-100 rounded-full")).child(View::text_styled("29".to_string(), "text-sm text-gray-700 text-center py-2")).child(View::text_styled("30".to_string(), "text-sm text-gray-700 text-center py-2")).style("w-full mt-4").build()).build()).build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app_devtools::<App>();
    }
    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend");
        return auto_lang::ui::gpui::run_app::<App>("calendar");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
