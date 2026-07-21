// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    TabDaily,
    TabHourly,
    Refresh,
}

#[derive(Debug)]
pub struct App {
    pub tab: String,
    pub city: String,
    pub temp: String,
    pub condition: String,
    pub info: String,
    pub forecast: String,
    pub hourly: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            tab: "daily".to_string(),
            city: "Beijing".to_string(),
            temp: "23°".to_string(),
            condition: "Partly Cloudy".to_string(),
            info: "Humidity: 65% | Wind: 12 km/h".to_string(),
            forecast: "Mon 18~25°  Tue 20~28°  Wed 16~22°  Thu 17~24°  Fri 19~27°".to_string(),
            hourly: "14:00 24°  15:00 25°  16:00 24°  17:00 22°  18:00 20°".to_string(),
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
            AppMsg::TabHourly => {
                self.tab = "hourly".to_string()
            }
            AppMsg::Refresh => {
                
            }
            AppMsg::TabDaily => {
                self.tab = "daily".to_string()
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::center(View::col().style("w-full max-w-md p-6 bg-gray-50 min-h-screen").child(View::row().style("w-full items-center").child(View::text_styled(format!("{}", self.city), "text-2xl font-bold text-gray-800")).child(View::button("Refresh").style("ml-auto px-4 py-2 bg-blue-500 text-white rounded-lg text-sm").on_click(|_| AppMsg::Refresh).build()).build()).child(View::col().style("bg-gradient-to-br from-blue-400 to-blue-600 rounded-2xl p-6 mt-4 items-center gap-2 py-6").child(View::text_styled(format!("{}", self.condition), "text-lg text-blue-100")).child(View::text_styled(format!("{}", self.temp), "text-8xl font-thin text-white")).child(View::text_styled(format!("{}", self.info), "text-sm text-blue-200")).build()).child(View::row().style("gap-2 mt-6").child(if self.tab == "daily" { View::button("Daily").style("px-4 py-2 rounded-lg text-sm font-medium bg-blue-500 text-white").on_click(|_| AppMsg::TabDaily).build() } else { View::Empty }).child(if self.tab != "daily" { View::button("Daily").style("px-4 py-2 rounded-lg text-sm font-medium bg-gray-200 text-gray-600").on_click(|_| AppMsg::TabDaily).build() } else { View::Empty }).child(if self.tab == "hourly" { View::button("Hourly").style("px-4 py-2 rounded-lg text-sm font-medium bg-blue-500 text-white").on_click(|_| AppMsg::TabHourly).build() } else { View::Empty }).child(if self.tab != "hourly" { View::button("Hourly").style("px-4 py-2 rounded-lg text-sm font-medium bg-gray-200 text-gray-600").on_click(|_| AppMsg::TabHourly).build() } else { View::Empty }).build()).child(if self.tab == "daily" { View::col().style("w-full").child(View::text_styled("5-Day Forecast".to_string(), "text-sm font-semibold text-gray-400 uppercase mt-2")).child(View::divider()).child(View::text_styled(format!("{}", self.forecast), "text-gray-700 py-2")).build() } else { View::Empty }).child(if self.tab == "hourly" { View::col().style("w-full").child(View::text_styled("Today".to_string(), "text-sm font-semibold text-gray-400 uppercase mt-2")).child(View::divider()).child(View::text_styled(format!("{}", self.hourly), "text-gray-700 py-2")).build() } else { View::Empty }).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("weather");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
