// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Clone, Debug, PartialEq)]
pub enum AppMsg {
    Start,
    Stop,
    Reset,
    Lap,
    Tick,
}

#[derive(Debug)]
pub struct App {
    pub running: String,
    pub time_display: String,
    pub ms_display: String,
    pub lap_count: String,
    pub lap1: String,
    pub lap2: String,
    pub lap3: String,
    pub elapsed: i32,
}

impl App {
    pub fn new() -> Self {
        Self {
            running: "false".to_string(),
            time_display: "00:00".to_string(),
            ms_display: ".00".to_string(),
            lap_count: "0".to_string(),
            lap1: "".to_string(),
            lap2: "".to_string(),
            lap3: "".to_string(),
            elapsed: 0,
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
            AppMsg::Tick => {
                if self.running == "true" {
                self.elapsed += 10
                    ;
                    let total_cs = self.elapsed / 10;
                    let cs = total_cs % 100;
                    let total_secs = total_cs / 100;
                    let secs = total_secs % 60;
                    let mins = total_secs / 60;
                    self.time_display = format!("{:02}:{:02}", mins, secs);
                    self.ms_display = format!(".{:02}", cs);
                }
            }
            AppMsg::Stop => {
                self.running = "false".to_string()
            }
            AppMsg::Reset => {
                self.running = "false".to_string();
                self.lap_count = "0".to_string();
                self.lap1 = "".to_string();
                self.lap2 = "".to_string();
                self.lap3 = "".to_string();
                self.time_display = "00:00".to_string();
                self.ms_display = ".00".to_string();
                self.elapsed = 0
            }
            AppMsg::Lap => {
                self.lap3 = self.lap2.clone();
                self.lap2 = self.lap1.clone();
                self.lap1 = format!("{}{}", self.time_display, self.ms_display);
                self.lap_count = (self.lap_count.parse::<i32>().unwrap_or(0) + 1).to_string()
            }
            AppMsg::Start => {
                self.running = "true".to_string()
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::center(View::col().style("w-full max-w-md p-8 bg-white min-h-screen items-center").child(View::col().style("items-center gap-1 py-12").child(View::text_styled(format!("{}", self.time_display), "text-7xl font-mono font-bold text-gray-900 tracking-wider")).child(View::text_styled(format!("{}", self.ms_display), "text-3xl font-mono text-gray-400")).build()).child(View::row().style("gap-4 items-center").child(if self.running == "true" { View::row().child(View::button("Stop").style("px-10 py-4 bg-red-500 text-white rounded-full text-lg font-semibold").on_click(|_| AppMsg::Stop).build()).child(View::button("Lap").style("px-10 py-4 bg-gray-200 text-gray-700 rounded-full text-lg font-semibold").on_click(|_| AppMsg::Lap).build()).build() } else { View::button("Start").style("px-10 py-4 bg-green-500 text-white rounded-full text-lg font-semibold").on_click(|_| AppMsg::Start).build() }).build()).child(View::col().style("w-full max-w-xs mt-8 gap-2 items-center").child(View::text_styled("Laps".to_string(), "text-sm font-semibold text-gray-400 uppercase")).child(View::text_styled(format!("{}", self.lap1), "text-gray-700 font-mono py-2")).child(View::text_styled(format!("{}", self.lap2), "text-gray-700 font-mono py-2")).child(View::text_styled(format!("{}", self.lap3), "text-gray-700 font-mono py-2")).build()).build()).build()
    }

    fn subscription(&self) -> iced::Subscription<Self::Msg> {
        iced::time::every(std::time::Duration::from_millis(10)).map(|_| AppMsg::Tick)
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
        return auto_lang::ui::gpui::run_app::<App>("stopwatch");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
