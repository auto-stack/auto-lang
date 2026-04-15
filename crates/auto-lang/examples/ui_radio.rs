// Unified Radio Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates radio buttons for single selection from multiple options.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run --example ui_radio --features ui-iced
//   cargo run --example ui_radio --features ui-gpui

use auto_lang::ui::{Component, View};

#[derive(Debug, Default)]
struct RadioApp {
    selected_language: Language,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
enum Language {
    #[default]
    Rust,
    Python,
    JavaScript,
    Go,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    SelectLanguage(Language),
}

impl Component for RadioApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::SelectLanguage(lang) => {
                self.selected_language = lang;
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Choose your favorite programming language:".to_string()))
            .child(self.view_language_selection())
            .child(self.view_result())
            .build()
    }
}

impl RadioApp {
    fn view_language_selection(&self) -> View<Message> {
        View::col()
            .spacing(8)
            .child(
                View::radio(self.selected_language == Language::Rust, "Rust")
                    .on_select(Message::SelectLanguage(Language::Rust))
            )
            .child(
                View::radio(self.selected_language == Language::Python, "Python")
                    .on_select(Message::SelectLanguage(Language::Python))
            )
            .child(
                View::radio(self.selected_language == Language::JavaScript, "JavaScript")
                    .on_select(Message::SelectLanguage(Language::JavaScript))
            )
            .child(
                View::radio(self.selected_language == Language::Go, "Go")
                    .on_select(Message::SelectLanguage(Language::Go))
            )
            .build()
    }

    fn view_result(&self) -> View<Message> {
        let lang_name = match self.selected_language {
            Language::Rust => "Rust",
            Language::Python => "Python",
            Language::JavaScript => "JavaScript",
            Language::Go => "Go",
        };

        View::container(
            View::text(format!("You selected: {}", lang_name))
        )
        .padding(20)
        .width(300)
        .center_x()
        .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("🎨 Running with Iced backend");
        return auto_lang::ui::iced::run_app::<RadioApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("🎨 Running with GPUI backend (with auto-conversion!)");
        return auto_lang::ui::gpui::run_app::<RadioApp>("Radio Demo - AutoUI");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --example ui_radio --features ui-iced\n\
             • cargo run --example ui_radio --features ui-gpui"
                .into(),
        )
    }
}
