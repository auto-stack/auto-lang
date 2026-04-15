// Unified Sidebar Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates the Sidebar (fixed-width side panel) component.
//
// Run with:
//   cargo run -p auto-lang --example ui_sidebar --features ui-iced
//   cargo run -p auto-lang --example ui_sidebar --features ui-gpui

use auto_lang::ui::{Component, View};
use auto_lang::ui::view::SidebarPosition;

#[derive(Debug, Default)]
struct SidebarApp {
    selected_page: usize,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Navigate(usize),
}

impl Component for SidebarApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Navigate(index) => {
                self.selected_page = index;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        // Create sidebar content with navigation items
        let sidebar_content = View::col()
            .spacing(10)
            .padding(10)
            .child(View::text("Navigation".to_string()))
            .child(View::button("Home", Message::Navigate(0)))
            .child(View::button("Settings", Message::Navigate(1)))
            .child(View::button("About", Message::Navigate(2)))
            .build();

        // Main content based on selection
        let main_content = match self.selected_page {
            0 => View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("Home Page".to_string()))
                .child(View::text("Welcome to the home page!".to_string()))
                .build(),
            1 => View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("Settings".to_string()))
                .child(View::text("Configure your preferences here.".to_string()))
                .build(),
            2 => View::col()
                .spacing(10)
                .padding(20)
                .child(View::text("About".to_string()))
                .child(View::text("This is a unified sidebar example.".to_string()))
                .build(),
            _ => View::text("Unknown page".to_string()),
        };

        View::row()
            .child(
                View::sidebar(sidebar_content, 250.0)
                    .position(SidebarPosition::Left)
                    .build()
            )
            .child(main_content)
            .build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    // The same code works with both backends!
    #[cfg(feature = "ui-iced")]
    {
        return auto_lang::ui::iced::run_app::<SidebarApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        return auto_lang::ui::gpui::run_app::<SidebarApp>("Sidebar Example");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled. Please enable either 'ui-iced' or 'ui-gpui' feature in Cargo.toml.".into())
    }
}
