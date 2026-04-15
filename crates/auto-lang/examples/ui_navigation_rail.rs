// Unified NavigationRail Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates the NavigationRail (compact side navigation) component.
//
// Run with:
//   cargo run -p auto-lang --example ui_navigation_rail --features ui-iced
//   cargo run -p auto-lang --example ui_navigation_rail --features ui-gpui

use auto_lang::ui::{Component, View};
use auto_lang::ui::view::NavigationRailItem;

#[derive(Debug, Default)]
struct NavigationRailApp {
    selected_item: usize,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Navigate(usize),
}

impl Component for NavigationRailApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Navigate(index) => {
                self.selected_item = index;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::row()
            .child(
                View::navigation_rail()
                    .items(vec![
                        NavigationRailItem::new('H', "Home"),
                        NavigationRailItem::new('S', "Settings").with_badge("3"),
                        NavigationRailItem::new('P', "Profile"),
                        NavigationRailItem::new('A', "About"),
                    ])
                    .selected(self.selected_item)
                    .width(72.0)
                    .show_labels(true)
                    .on_select(|index| Message::Navigate(index))
                    .build()
            )
            .child(
                // Main content area
                match self.selected_item {
                    0 => View::col()
                        .spacing(10)
                        .padding(20)
                        .child(View::text("Home".to_string()))
                        .child(View::text("Welcome to the home page!".to_string()))
                        .build(),
                    1 => View::col()
                        .spacing(10)
                        .padding(20)
                        .child(View::text("Settings".to_string()))
                        .child(View::text("3 notifications pending".to_string()))
                        .build(),
                    2 => View::col()
                        .spacing(10)
                        .padding(20)
                        .child(View::text("Profile".to_string()))
                        .child(View::text("User information".to_string()))
                        .build(),
                    3 => View::col()
                        .spacing(10)
                        .padding(20)
                        .child(View::text("About".to_string()))
                        .child(View::text("NavigationRail example".to_string()))
                        .build(),
                    _ => View::text("Unknown page".to_string()),
                }
            )
            .build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    // The same code works with both backends!
    #[cfg(feature = "ui-iced")]
    {
        return auto_lang::ui::iced::run_app::<NavigationRailApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        return auto_lang::ui::gpui::run_app::<NavigationRailApp>("NavigationRail Example");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled. Please enable either 'ui-iced' or 'ui-gpui' feature in Cargo.toml.".into())
    }
}
