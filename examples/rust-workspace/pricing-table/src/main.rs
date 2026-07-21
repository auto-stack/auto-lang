// Auto-generated from Auto language by a2rust-ui

use auto_lang::ui::{Component, View};

// Auto-generated from Auto language
// DO NOT EDIT - changes will be overwritten


#[derive(Debug)]
pub struct App {
    pub plan1_name: String,
    pub plan1_subtitle: String,
    pub plan1_price: String,
    pub plan1_devs: String,
    pub plan1_btn: String,
    pub plan2_name: String,
    pub plan2_subtitle: String,
    pub plan2_price: String,
    pub plan2_devs: String,
    pub plan2_btn: String,
    pub plan3_name: String,
    pub plan3_subtitle: String,
    pub plan3_deal: String,
    pub plan3_devs: String,
    pub plan3_btn: String,
    pub feat1: String,
    pub feat2: String,
    pub feat3: String,
    pub feat4: String,
    pub feat5: String,
}

impl App {
    pub fn new() -> Self {
        Self {
            plan1_name: "Single Developer".to_string(),
            plan1_subtitle: "For individual developers".to_string(),
            plan1_price: "$39".to_string(),
            plan1_devs: "1 Developer".to_string(),
            plan1_btn: "Buy Now".to_string(),
            plan2_name: "Team".to_string(),
            plan2_subtitle: "For small teams".to_string(),
            plan2_price: "$99".to_string(),
            plan2_devs: "Up to 5 Developers".to_string(),
            plan2_btn: "Buy Now".to_string(),
            plan3_name: "Enterprise".to_string(),
            plan3_subtitle: "For larger teams".to_string(),
            plan3_deal: "Exclusive Deals".to_string(),
            plan3_devs: "Unlimited Developers".to_string(),
            plan3_btn: "Contact Us".to_string(),
            feat1: "All Marketing + Application UI Blocks".to_string(),
            feat2: "Figma UI Kit".to_string(),
            feat3: "Lifetime Support".to_string(),
            feat4: "Unlimited Updates".to_string(),
            feat5: "Use on Unlimited Projects".to_string(),
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
        View::col().w_full().p(8).gap(8).bg("gray-50").items_center().style("min-h-screen").child(View::col().gap(2).items_center().child(View::text("Pricing Plans".to_string())).child(View::text("Choose the plan that fits your needs".to_string())).build()).child(View::row().gap(6).child(View::col().bg("white").p(6).flex1().style("rounded-xl border-2 border-blue-500").child(View::col().gap(1).items_center().child(View::text_styled(format!("{}", self.plan1_name), "text-lg font-bold text-gray-900")).child(View::text_styled(format!("{}", self.plan1_subtitle), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.plan1_price), "text-4xl font-bold text-gray-900 mt-2")).build()).child(View::col().gap(3).style("mt-6").child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-blue-500 font-bold")).child(View::text(format!("{}", self.plan1_devs))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-blue-500 font-bold")).child(View::text(format!("{}", self.feat1))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-blue-500 font-bold")).child(View::text(format!("{}", self.feat2))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-blue-500 font-bold")).child(View::text(format!("{}", self.feat3))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-blue-500 font-bold")).child(View::text(format!("{}", self.feat4))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-blue-500 font-bold")).child(View::text(format!("{}", self.feat5))).build()).build()).child(View::button(format!("{}", self.plan1_btn)).w_full().bg("blue-500").text_color("white").rounded_lg().style("py-2.5 font-semibold mt-6").on_click(|_| ()).build()).build()).child(View::col().bg("white").p(6).flex1().style("rounded-xl border-2 border-orange-500").child(View::col().gap(1).items_center().child(View::text_styled(format!("{}", self.plan2_name), "text-lg font-bold text-gray-900")).child(View::text_styled(format!("{}", self.plan2_subtitle), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.plan2_price), "text-4xl font-bold text-gray-900 mt-2")).build()).child(View::col().gap(3).style("mt-6").child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-orange-500 font-bold")).child(View::text(format!("{}", self.plan2_devs))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-orange-500 font-bold")).child(View::text(format!("{}", self.feat1))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-orange-500 font-bold")).child(View::text(format!("{}", self.feat2))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-orange-500 font-bold")).child(View::text(format!("{}", self.feat3))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-orange-500 font-bold")).child(View::text(format!("{}", self.feat4))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-orange-500 font-bold")).child(View::text(format!("{}", self.feat5))).build()).build()).child(View::button(format!("{}", self.plan2_btn)).w_full().bg("orange-500").text_color("white").rounded_lg().style("py-2.5 font-semibold mt-6").on_click(|_| ()).build()).build()).child(View::col().bg("white").p(6).flex1().style("rounded-xl border-2 border-gray-400").child(View::col().gap(1).items_center().child(View::text_styled(format!("{}", self.plan3_name), "text-lg font-bold text-gray-900")).child(View::text_styled(format!("{}", self.plan3_subtitle), "text-sm text-gray-500")).child(View::text_styled(format!("{}", self.plan3_deal), "text-4xl font-bold text-gray-900 mt-2")).build()).child(View::col().gap(3).style("mt-6").child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-gray-400 font-bold")).child(View::text(format!("{}", self.plan3_devs))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-gray-400 font-bold")).child(View::text(format!("{}", self.feat1))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-gray-400 font-bold")).child(View::text(format!("{}", self.feat2))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-gray-400 font-bold")).child(View::text(format!("{}", self.feat3))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-gray-400 font-bold")).child(View::text(format!("{}", self.feat4))).build()).child(View::row().gap(2).items_center().child(View::text_styled("✓".to_string(), "text-gray-400 font-bold")).child(View::text(format!("{}", self.feat5))).build()).build()).child(View::button(format!("{}", self.plan3_btn)).w_full().bg("gray-500").text_color("white").rounded_lg().style("py-2.5 font-semibold mt-6").on_click(|_| ()).build()).build()).build()).build()
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
        return auto_lang::ui::gpui::run_app::<App>("pricing-table");
    }
    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled! Use --features ui-iced or ui-gpui".into())
    }
}
