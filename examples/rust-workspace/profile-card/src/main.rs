// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Debug)]
pub struct App {
    pub name: String,
    pub role: String,
    pub bio: String,
    pub avatar_url: String,
    pub status: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            name: "Jane Cooper".to_string(),
            role: "Full Stack Developer".to_string(),
            bio: "Passionate about building great user experiences. Open source contributor and coffee enthusiast.".to_string(),
            avatar_url: "https://api.dicebear.com/7.x/avataaars/svg?seed=Jane".to_string(),
            status: "online".to_string(),
        }
    }
}
impl Default for App {
    fn default() -> Self { Self::new() }
}

impl Component for App {
    type Msg = ();

    fn on(&mut self, msg: Self::Msg) {
    }

    fn view(&self) -> View<Self::Msg> {
        View::col().w_full().h_full().justify_center().items_center().child(View::col().bg("white").rounded_lg().border().items_center().gap(4).style("shadow-lg border-gray-200 max-w-sm mx-auto pb-6 overflow-hidden").child(View::col().h(20).bg("gradient-to-r").style("from-blue-500 to-purple-600 rounded-t-lg").build()).child(View::col().items_center().style("-mt-10").child(View::image_styled(format!("{}", self.avatar_url), "w-20 h-20 rounded-full border-4 border-gray-200 shadow-md")).build()).child(View::col().gap(2).items_center().child(View::text(format!("{}", self.name))).child(View::row().text_color("gray-500").style("text-sm").child(View::text(format!("{}", self.status))).child(View::text("Active".to_string())).build()).build()).child(View::row().px(3).py(1).bg("blue-100").text_color("blue-800").font_medium().style("text-sm rounded-full").child(View::text(format!("{}", self.role))).build()).child(View::text_styled(format!("{}", self.bio), "text-gray-600 text-sm text-center px-6 leading-relaxed")).child(View::row().gap(3).child(View::button("Follow").px(4).py(2).bg("blue-500").text_color("white").rounded_lg().style("hover:bg-blue-600").on_click(|_| ()).build()).child(View::button("Message").px(4).py(2).bg("gray-200").text_color("gray-700").rounded_lg().style("hover:bg-gray-300").on_click(|_| ()).build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("profile-card");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
