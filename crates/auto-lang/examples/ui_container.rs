// Unified Container Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates styling and layout options for containers.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run -p auto-lang --example ui_container --features ui-iced
//   cargo run -p auto-lang --example ui_container --features ui-gpui

use auto_lang::ui::{Component, View};

#[derive(Debug)]
struct ContainerApp {
    selected_example: Example,
}

impl Default for ContainerApp {
    fn default() -> Self {
        Self {
            selected_example: Example::Padding,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Example {
    Padding,
    Sizing,
    Centering,
    Nested,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ShowExample(Example),
}

impl Component for ContainerApp {
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
            .child(View::text("Container Examples".to_string()))
            .child(self.view_navigation())
            .child(self.view_current_example())
            .build()
    }
}

impl ContainerApp {
    fn view_navigation(&self) -> View<Message> {
        View::row()
            .spacing(8)
            .padding(0)
            .child(View::button("Padding").on_click(|_| Message::ShowExample(Example::Padding)).build())
            .child(View::button("Sizing").on_click(|_| Message::ShowExample(Example::Sizing)).build())
            .child(View::button("Centering").on_click(|_| Message::ShowExample(Example::Centering)).build())
            .child(View::button("Nested").on_click(|_| Message::ShowExample(Example::Nested)).build())
            .build()
    }

    fn view_current_example(&self) -> View<Message> {
        match self.selected_example {
            Example::Padding => self.view_padding_example(),
            Example::Sizing => self.view_sizing_example(),
            Example::Centering => self.view_centering_example(),
            Example::Nested => self.view_nested_example(),
        }
    }

    fn view_padding_example(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::container(View::text("No Padding"))
                    .padding(0)
                    .build()
            )
            .child(
                View::container(View::text("Padding 20"))
                    .padding(20)
                    .build()
            )
            .child(
                View::container(View::text("Padding 40"))
                    .padding(40)
                    .build()
            )
            .build()
    }

    fn view_sizing_example(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::container(
                    View::text("Fixed Width: 200")
                )
                .width(200)
                .padding(10)
                .build()
            )
            .child(
                View::container(
                    View::text("Fixed Size: 200x100")
                )
                .width(200)
                .height(100)
                .padding(10)
                .build()
            )
            .build()
    }

    fn view_centering_example(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::container(
                    View::text("Centered X")
                )
                .width(300)
                .padding(10)
                .center_x()
                .build()
            )
            .child(
                View::container(
                    View::text("Centered Y")
                )
                .width(300)
                .height(100)
                .padding(10)
                .center_y()
                .build()
            )
            .child(
                View::container(
                    View::text("Centered Both")
                )
                .width(300)
                .height(100)
                .padding(10)
                .center()
                .build()
            )
            .build()
    }

    fn view_nested_example(&self) -> View<Message> {
        View::container(
            View::col()
                .spacing(10)
                .child(View::text("Outer Container".to_string()))
                .child(
                    View::container(
                        View::row()
                            .spacing(8)
                            .child(View::text("Nested".to_string()))
                            .child(View::text("Layout".to_string()))
                            .build()
                    )
                    .padding(20)
                    .center_x()
                    .build()
                )
                .build()
        )
        .padding(30)
        .width(400)
        .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("Running with Iced backend");
        return auto_lang::ui::iced::run_app::<ContainerApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("Running with GPUI backend (with auto-conversion!)");
        return auto_lang::ui::gpui::run_app::<ContainerApp>("Container Demo - AutoUI");
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
