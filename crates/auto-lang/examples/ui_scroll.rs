// Unified Scroll Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates scrollable containers for content overflow.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run -p auto-lang --example ui_scroll --features ui-iced
//   cargo run -p auto-lang --example ui_scroll --features ui-gpui

use auto_lang::ui::{Component, View};

#[derive(Debug)]
struct ScrollApp {
    selected_example: Example,
}

impl Default for ScrollApp {
    fn default() -> Self {
        Self {
            selected_example: Example::Basic,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Example {
    Basic,
    LongContent,
    NestedScrollable,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ShowExample(Example),
}

impl Component for ScrollApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ShowExample(example) => {
                self.selected_example = example;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Scrollable Examples".to_string()))
            .child(self.view_navigation())
            .child(self.view_current_example())
            .build()
    }
}

impl ScrollApp {
    fn view_navigation(&self) -> View<Message> {
        View::row()
            .spacing(8)
            .padding(0)
            .child(View::button("Basic", Message::ShowExample(Example::Basic)))
            .child(View::button("Long Content", Message::ShowExample(Example::LongContent)))
            .child(View::button("Nested", Message::ShowExample(Example::NestedScrollable)))
            .build()
    }

    fn view_current_example(&self) -> View<Message> {
        match self.selected_example {
            Example::Basic => self.view_basic_example(),
            Example::LongContent => self.view_long_content_example(),
            Example::NestedScrollable => self.view_nested_example(),
        }
    }

    fn view_basic_example(&self) -> View<Message> {
        // Simple scrollable with fixed size
        View::scrollable(
            View::col()
                .spacing(10)
                .child(View::text("Item 1".to_string()))
                .child(View::text("Item 2".to_string()))
                .child(View::text("Item 3".to_string()))
                .child(View::text("Item 4".to_string()))
                .child(View::text("Item 5".to_string()))
                .child(View::text("Item 6".to_string()))
                .child(View::text("Item 7".to_string()))
                .child(View::text("Item 8".to_string()))
                .child(View::text("Item 9".to_string()))
                .child(View::text("Item 10".to_string()))
                .build()
        )
        .width(300)
        .height(200)
        .build()
    }

    fn view_long_content_example(&self) -> View<Message> {
        // Generate many items to demonstrate scrolling
        let mut items = Vec::new();
        for i in 1..=50 {
            items.push(View::text(format!("Item {}: Long content that scrolls", i)));
        }

        View::col()
            .spacing(16)
            .child(View::text("Scrollable with 50 items:".to_string()))
            .child(
                View::scrollable(
                    View::col()
                        .spacing(8)
                        .children(items)
                        .build()
                )
                .height(300)
                .build()
            )
            .build()
    }

    fn view_nested_example(&self) -> View<Message> {
        // Nested scrollable containers
        View::col()
            .spacing(16)
            .child(View::text("Nested Scrollable Containers".to_string()))
            .child(
                View::container(
                    View::col()
                        .spacing(10)
                        .child(View::text("Outer container - fixed size".to_string()))
                        .child(
                            View::scrollable(
                                View::col()
                                    .spacing(8)
                                    .children((1..=20).map(|i| {
                                        View::text(format!("Nested item {}", i))
                                    }).collect::<Vec<_>>())
                                    .build()
                            )
                            .height(150)
                            .build()
                        )
                        .child(View::text("More content outside scrollable".to_string()))
                        .build()
                )
                .padding(10)
                .build()
            )
            .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<ScrollApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend (with auto-conversion!)");
        return auto_lang::ui::gpui::run_app::<ScrollApp>("Scroll Demo - AutoUI");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err(
            "No backend enabled!\n\n\
             Please run with a backend feature:\n\
             \u{2022} cargo run --features ui-iced\n\
             \u{2022} cargo run --features ui-gpui"
                .into(),
        )
    }
}
