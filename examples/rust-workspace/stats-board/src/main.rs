// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Debug)]
pub struct App {
    pub dashboard_title: String,
    pub dashboard_subtitle: String,
    pub stat1_title: String,
    pub stat1_value: String,
    pub stat2_title: String,
    pub stat2_value: String,
    pub stat3_title: String,
    pub stat3_value: String,
    pub stat4_title: String,
    pub stat4_value: String,
    pub perf_all_traffic: String,
    pub perf_instagram: String,
    pub perf_linkedin: String,
    pub perf_google: String,
    pub perf_x: String,
    pub target_progress: i32,
    pub target_current: String,
    pub target_goal: String,
    pub kpi1: String,
    pub kpi1_value: String,
    pub kpi2: String,
    pub kpi2_value: String,
    pub kpi3: String,
    pub kpi3_value: String,
    pub kpi4: String,
    pub kpi4_value: String,
    pub member1_name: String,
    pub member1_role: String,
    pub member1_posts: String,
    pub member1_followers: String,
    pub member1_engagement: String,
    pub member2_name: String,
    pub member2_role: String,
    pub member2_posts: String,
    pub member2_followers: String,
    pub member2_engagement: String,
    pub member3_name: String,
    pub member3_role: String,
    pub member3_posts: String,
    pub member3_followers: String,
    pub member3_engagement: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            dashboard_title: "Dashboard".to_string(),
            dashboard_subtitle: "Campaign Insights".to_string(),
            stat1_title: "Campaign Alerts".to_string(),
            stat1_value: "123k".to_string(),
            stat2_title: "System Alerts".to_string(),
            stat2_value: "89k".to_string(),
            stat3_title: "Promotional Offers".to_string(),
            stat3_value: "3k".to_string(),
            stat4_title: "Traffic Distribution".to_string(),
            stat4_value: "175k".to_string(),
            perf_all_traffic: "14,582".to_string(),
            perf_instagram: "8,421".to_string(),
            perf_linkedin: "3,219".to_string(),
            perf_google: "1,987".to_string(),
            perf_x: "952".to_string(),
            target_progress: 86,
            target_current: "8,571".to_string(),
            target_goal: "10,000".to_string(),
            kpi1: "Revenue Target".to_string(),
            kpi1_value: "$32.5k".to_string(),
            kpi2: "Active Users".to_string(),
            kpi2_value: "2,450".to_string(),
            kpi3: "Conversion Rate".to_string(),
            kpi3_value: "5.6%".to_string(),
            kpi4: "Avg. Order Value".to_string(),
            kpi4_value: "$128".to_string(),
            member1_name: "Sarah Johnson".to_string(),
            member1_role: "Marketing Lead".to_string(),
            member1_posts: "142".to_string(),
            member1_followers: "12.5k".to_string(),
            member1_engagement: "4.8%".to_string(),
            member2_name: "Mike Chen".to_string(),
            member2_role: "Content Strategist".to_string(),
            member2_posts: "98".to_string(),
            member2_followers: "8.2k".to_string(),
            member2_engagement: "6.1%".to_string(),
            member3_name: "Emily Davis".to_string(),
            member3_role: "Social Media Manager".to_string(),
            member3_posts: "215".to_string(),
            member3_followers: "15.8k".to_string(),
            member3_engagement: "5.3%".to_string(),
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
        View::col().w_full().p(6).gap(6).bg("gray-50").style("min-h-screen").child(View::row().w_full().items_center().justify_between().child(View::col().gap(1).child(View::text_styled(format!("{}", self.dashboard_title), "text-2xl font-bold text-gray-900")).child(View::text_styled(format!("{}", self.dashboard_subtitle), "text-sm text-gray-500")).build()).child(View::spacer()).build()).child(View::row().gap(4).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::col().w(12).h(12).rounded_lg().bg("red-100").flex().items_center().justify_center().text_color("red-500").font_bold().style("text-xl").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.stat1_title), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.stat1_value), "text-2xl font-bold text-gray-900")).build()).build()).build()).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::col().w(12).h(12).rounded_lg().bg("blue-100").flex().items_center().justify_center().text_color("blue-500").font_bold().style("text-xl").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.stat2_title), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.stat2_value), "text-2xl font-bold text-gray-900")).build()).build()).build()).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::col().w(12).h(12).rounded_lg().bg("green-100").flex().items_center().justify_center().text_color("green-500").font_bold().style("text-xl").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.stat3_title), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.stat3_value), "text-2xl font-bold text-gray-900")).build()).build()).build()).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::col().w(12).h(12).rounded_lg().bg("purple-100").flex().items_center().justify_center().text_color("purple-500").font_bold().style("text-xl").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.stat4_title), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.stat4_value), "text-2xl font-bold text-gray-900")).build()).build()).build()).build()).child(View::row().gap(4).child(View::col().bg("white").border().p(5).flex1().gap(4).style("rounded-xl shadow-sm border-gray-100").child(View::text_styled("Campaign Performance".to_string(), "text-lg font-semibold text-gray-900")).child(View::col().w_full().child(View::row().w_full().items_center().py(3).style("border-b border-gray-100").child(View::text("All Traffic".to_string())).child(View::spacer()).child(View::text_styled(format!("{}", self.perf_all_traffic), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(3).style("border-b border-gray-100").child(View::text("Instagram".to_string())).child(View::spacer()).child(View::text_styled(format!("{}", self.perf_instagram), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(3).style("border-b border-gray-100").child(View::text("LinkedIn".to_string())).child(View::spacer()).child(View::text_styled(format!("{}", self.perf_linkedin), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(3).style("border-b border-gray-100").child(View::text("Google".to_string())).child(View::spacer()).child(View::text_styled(format!("{}", self.perf_google), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(3).child(View::text("X".to_string())).child(View::spacer()).child(View::text_styled(format!("{}", self.perf_x), "font-semibold text-gray-900")).build()).build()).build()).child(View::col().bg("white").border().p(5).flex1().gap(4).style("rounded-xl shadow-sm border-gray-100").child(View::text_styled("Campaign Targets".to_string(), "text-lg font-semibold text-gray-900")).child(View::col().w_full().child(View::row().w_full().items_center().child(View::text("Overall Progress".to_string())).child(View::spacer()).child(View::text("85.7%".to_string())).build()).child(View::progress_bar_styled(self.target_progress as f32 / 100 as f32, "mt-2 mb-4")).child(View::row().text_color("gray-500").gap(1).style("text-sm").child(View::text(format!("{}", self.target_current))).child(View::text("/".to_string())).child(View::text(format!("{}", self.target_goal))).build()).build()).child(View::divider()).child(View::col().w_full().child(View::row().w_full().items_center().py(2).style("border-b border-gray-100").child(View::text(format!("{}", self.kpi1))).child(View::spacer()).child(View::text_styled(format!("{}", self.kpi1_value), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(2).style("border-b border-gray-100").child(View::text(format!("{}", self.kpi2))).child(View::spacer()).child(View::text_styled(format!("{}", self.kpi2_value), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(2).style("border-b border-gray-100").child(View::text(format!("{}", self.kpi3))).child(View::spacer()).child(View::text_styled(format!("{}", self.kpi3_value), "font-semibold text-gray-900")).build()).child(View::row().w_full().items_center().py(2).child(View::text(format!("{}", self.kpi4))).child(View::spacer()).child(View::text_styled(format!("{}", self.kpi4_value), "font-semibold text-gray-900")).build()).build()).build()).build()).child(View::col().w_full().gap(4).child(View::text_styled("Team Performance".to_string(), "text-lg font-semibold text-gray-900")).child(View::row().gap(4).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::avatar().w(10).h(10).bg("blue-100").style("rounded-full").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.member1_name), "font-semibold text-gray-900 text-sm")).child(View::text_styled(format!("{}", self.member1_role), "text-xs text-gray-500")).build()).build()).child(View::row().gap(4).style("mt-3").child(View::col().gap(0).items_center().child(View::text("Posts".to_string())).child(View::text_styled(format!("{}", self.member1_posts), "font-semibold text-gray-900 text-sm")).build()).child(View::col().gap(0).items_center().child(View::text("Followers".to_string())).child(View::text_styled(format!("{}", self.member1_followers), "font-semibold text-gray-900 text-sm")).build()).child(View::col().gap(0).items_center().child(View::text("Engagement".to_string())).child(View::text_styled(format!("{}", self.member1_engagement), "font-semibold text-gray-900 text-sm")).build()).build()).build()).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::avatar().w(10).h(10).bg("green-100").style("rounded-full").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.member2_name), "font-semibold text-gray-900 text-sm")).child(View::text_styled(format!("{}", self.member2_role), "text-xs text-gray-500")).build()).build()).child(View::row().gap(4).style("mt-3").child(View::col().gap(0).items_center().child(View::text("Posts".to_string())).child(View::text_styled(format!("{}", self.member2_posts), "font-semibold text-gray-900 text-sm")).build()).child(View::col().gap(0).items_center().child(View::text("Followers".to_string())).child(View::text_styled(format!("{}", self.member2_followers), "font-semibold text-gray-900 text-sm")).build()).child(View::col().gap(0).items_center().child(View::text("Engagement".to_string())).child(View::text_styled(format!("{}", self.member2_engagement), "font-semibold text-gray-900 text-sm")).build()).build()).build()).child(View::col().bg("white").border().p(4).flex1().style("rounded-xl shadow-sm border-gray-100").child(View::row().gap(3).items_center().child(View::avatar().w(10).h(10).bg("purple-100").style("rounded-full").build()).child(View::col().gap(0).child(View::text_styled(format!("{}", self.member3_name), "font-semibold text-gray-900 text-sm")).child(View::text_styled(format!("{}", self.member3_role), "text-xs text-gray-500")).build()).build()).child(View::row().gap(4).style("mt-3").child(View::col().gap(0).items_center().child(View::text("Posts".to_string())).child(View::text_styled(format!("{}", self.member3_posts), "font-semibold text-gray-900 text-sm")).build()).child(View::col().gap(0).items_center().child(View::text("Followers".to_string())).child(View::text_styled(format!("{}", self.member3_followers), "font-semibold text-gray-900 text-sm")).build()).child(View::col().gap(0).items_center().child(View::text("Engagement".to_string())).child(View::text_styled(format!("{}", self.member3_engagement), "font-semibold text-gray-900 text-sm")).build()).build()).build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("stats-board");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
