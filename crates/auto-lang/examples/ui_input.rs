// Unified Input Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates text input fields for data entry with various configurations.
// The same Component code works with both backends through automatic message conversion.
//
// Run with:
//   cargo run --example ui_input --features ui-iced
//   cargo run --example ui_input --features ui-gpui

use auto_lang::ui::{Component, View};

#[derive(Debug, Default)]
struct InputApp {
    username: String,
    email: String,
    password: String,
    bio: String,
}

#[derive(Clone, Debug)]
enum Message {
    UsernameChanged(String),
    EmailChanged(String),
    PasswordChanged(String),
    BioChanged(String),
    Clear,
}

impl Component for InputApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::UsernameChanged(value) => {
                self.username = value;
            }
            Message::EmailChanged(value) => {
                self.email = value;
            }
            Message::PasswordChanged(value) => {
                self.password = value;
            }
            Message::BioChanged(value) => {
                self.bio = value;
            }
            Message::Clear => {
                self.username.clear();
                self.email.clear();
                self.password.clear();
                self.bio.clear();
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("User Information Form".to_string()))
            .child(self.view_form())
            .child(self.view_summary())
            .build()
    }
}

impl InputApp {
    fn view_form(&self) -> View<Message> {
        View::col()
            .spacing(16)
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Username:".to_string()))
                    .child(
                        View::input("Enter your username")
                            .value(&self.username)
                            .width(300)
                            .on_change(Message::UsernameChanged(self.username.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Email:".to_string()))
                    .child(
                        View::input("user@example.com")
                            .value(&self.email)
                            .width(300)
                            .on_change(Message::EmailChanged(self.email.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Password:".to_string()))
                    .child(
                        View::input("Enter password")
                            .value(&self.password)
                            .width(300)
                            .password()
                            .on_change(Message::PasswordChanged(self.password.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::col()
                    .spacing(4)
                    .child(View::text("Bio:".to_string()))
                    .child(
                        View::input("Tell us about yourself")
                            .value(&self.bio)
                            .width(400)
                            .on_change(Message::BioChanged(self.bio.clone()))
                            .build(),
                    )
                    .build(),
            )
            .child(
                View::row()
                    .spacing(8)
                    .child(View::button("Clear Form", Message::Clear))
                    .build(),
            )
            .build()
    }

    fn view_summary(&self) -> View<Message> {
        View::container(
            View::col()
                .spacing(8)
                .child(View::text("Form Summary:".to_string()))
                .child(View::text(format!("Username: {}", self.username)))
                .child(View::text(format!("Email: {}", self.email)))
                .child(View::text(format!(
                    "Password: {}",
                    if self.password.is_empty() {
                        "(empty)"
                    } else {
                        "••••••••"
                    }
                )))
                .child(View::text(format!("Bio: {}", self.bio)))
                .build(),
        )
        .padding(20)
        .width(400)
        .build()
    }
}

// Unified main() - works with BOTH backends!
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("🎨 Running with Iced backend");
        return auto_lang::ui::iced::run_app::<InputApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("🎨 Running with GPUI backend (with auto-conversion!)");
        return auto_lang::ui::gpui::run_app::<InputApp>("Input Demo - AutoUI");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --example ui_input --features ui-iced\n\
             • cargo run --example ui_input --features ui-gpui"
                .into(),
        )
    }
}
