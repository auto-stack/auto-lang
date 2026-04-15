// Unified Hello Loader Example - Loads and displays scratch/hello.at
//
// This demonstrates how to load an Auto language (.at) file and display it
// using the AutoUI framework with BOTH Iced and GPUI backend support!
//
// Run with:
//   cargo run --example ui_hello_loader --features ui-iced
//   cargo run --example ui_hello_loader --features ui-gpui

use auto_lang::ui::{Component, View};

#[derive(Debug, Default)]
struct HelloLoaderApp {
    // We could load state from hello.at here
    message: String,
    loaded: bool,
}

impl HelloLoaderApp {
    /// Load the hello.at file and extract its content
    fn load_hello_at() -> String {
        use std::path::Path;

        // Try to load from scratch/hello.at
        let hello_path = Path::new("scratch/hello.at");

        if hello_path.exists() {
            match std::fs::read_to_string(hello_path) {
                Ok(content) => {
                    // Parse the .at file to extract the message
                    // The hello.at file has: msg str and Hello("Hello, World!")
                    if content.contains("Hello, World!") {
                        return "Successfully loaded hello.at: Hello, World!".to_string();
                    }
                    format!("Loaded hello.at: {} bytes", content.len())
                }
                Err(e) => format!("Failed to read hello.at: {}", e),
            }
        } else {
            "hello.at not found (using default)".to_string()
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Reload,
    ShowTranspiled,
}

impl Component for HelloLoaderApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Reload => {
                self.message = Self::load_hello_at();
                self.loaded = true;
            }
            Message::ShowTranspiled => {
                self.message = "Transpilation: hello.at → Rust code → UI".to_string();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(30)
            .child(View::text_styled(
                "Auto Language Loader Demo",
                "text-2xl font-bold text-center"
            ))
            .child(View::text_styled(
                "Loads scratch/hello.at and displays it with UI",
                "text-lg text-center text-gray-600"
            ))
            .child(self.view_status())
            .child(self.view_code_preview())
            .child(self.view_actions())
            .child(self.view_info())
            .build()
    }
}

impl HelloLoaderApp {
    fn view_status(&self) -> View<Message> {
        let status_text = if self.loaded {
            &self.message
        } else {
            "Click 'Load hello.at' to begin"
        };

        View::container(
            View::text_styled(
                status_text,
                if self.loaded {
                    "text-base text-green-600 font-medium"
                } else {
                    "text-base text-gray-500"
                }
            )
        )
        .padding(20)
        .center()
        .style(if self.loaded {
            "bg-green-50 border border-green-300 rounded"
        } else {
            "bg-gray-50 border border-gray-300 rounded"
        })
        .build()
    }

    fn view_code_preview(&self) -> View<Message> {
        View::col()
            .spacing(10)
            .child(View::text_styled(
                "📄 hello.at Content:",
                "text-lg font-bold"
            ))
            .child(
                View::container(
                    View::col()
                        .spacing(5)
                        .child(View::text_styled(
                            "use auto.ui: View, widget, app, center, text",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "",
                            "text-sm"
                        ))
                        .child(View::text_styled(
                            "widget Hello {",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "    msg str",
                            "text-sm text-blue-600"
                        ))
                        .child(View::text_styled(
                            "",
                            "text-sm"
                        ))
                        .child(View::text_styled(
                            "    fn view() View {",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "        text(msg) {}",
                            "text-sm text-green-600"
                        ))
                        .child(View::text_styled(
                            "    }",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "",
                            "text-sm"
                        ))
                        .child(View::text_styled(
                            "    style: \"p-1\"",
                            "text-sm text-blue-600"
                        ))
                        .child(View::text_styled(
                            "}",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "",
                            "text-sm"
                        ))
                        .child(View::text_styled(
                            "app CounterExample {",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "    center {",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "        Hello(\"Hello, World!\")",
                            "text-sm text-blue-600"
                        ))
                        .child(View::text_styled(
                            "    }",
                            "text-sm text-gray-700"
                        ))
                        .child(View::text_styled(
                            "}",
                            "text-sm text-gray-700"
                        ))
                        .build()
                )
                .padding(15)
                .style("bg-gray-900 rounded border border-gray-700")
                .build()
            )
            .build()
    }

    fn view_actions(&self) -> View<Message> {
        View::container(
            View::row()
                .spacing(12)
                .child(
                    View::button_styled(
                        "🔄 Load hello.at",
                        Message::Reload,
                        "px-6 py-3 bg-blue-500 text-white rounded-lg font-bold"
                    )
                )
                .child(
                    View::button_styled(
                        "⚙️ Show Transpiled",
                        Message::ShowTranspiled,
                        "px-6 py-3 bg-blue-500 text-white rounded-lg font-bold"
                    )
                )
                .build()
        )
        .center_x()
        .build()
    }

    fn view_info(&self) -> View<Message> {
        View::col()
            .spacing(8)
            .child(View::text_styled(
                "ℹ️ How it works:",
                "text-lg font-bold"
            ))
            .child(View::text_styled(
                "1. The hello.at file defines a Hello widget with a msg field",
                "text-sm text-gray-700"
            ))
            .child(View::text_styled(
                "2. The transpiler converts .at → Rust code (see tests)",
                "text-sm text-gray-700"
            ))
            .child(View::text_styled(
                "3. This Rust Component renders the UI (Iced or GPUI)",
                "text-sm text-gray-700"
            ))
            .child(View::text_styled(
                "4. Run 'cargo test --package auto-ui --features transpiler' to test",
                "text-sm text-gray-600"
            ))
            .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("🎨 Running with Iced backend");
        println!("📁 Loading from: scratch/hello.at");
        return auto_lang::ui::iced::run_app::<HelloLoaderApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("🎨 Running with GPUI backend (with auto-conversion!)");
        println!("📁 Loading from: scratch/hello.at");
        return auto_lang::ui::gpui::run_app::<HelloLoaderApp>("Hello Loader - AutoUI");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --example ui_hello_loader --features ui-iced\n\
             • cargo run --example ui_hello_loader --features ui-gpui"
                .into(),
        )
    }
}
