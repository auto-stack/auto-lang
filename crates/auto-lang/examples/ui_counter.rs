// Unified Counter Example with Auto-Conversion
//
// This example demonstrates TRUE unification - the same Component code
//
// Run with:
//   cargo run --example ui_counter --features ui-iced

use auto_lang::ui::{Component, View};

#[derive(Debug, Default)]
struct Counter {
    count: i64,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Increment,
    Decrement,
}

impl Component for Counter {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::Increment => self.count += 1,
            Message::Decrement => self.count -= 1,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(16)
            .padding(20)
            .child(View::button("Increment (+)").on_click(|_| Message::Increment).build())
            .child(View::text(format!("Count: {}", self.count)))
            .child(View::button("Decrement (-)").on_click(|_| Message::Decrement).build())
            .build()
    }
}

// Unified main() — Plan 365 W1: uses the HostBackend entry point.
// This replaces the former hand-written cfg-ladder. The backend is selected
// by Cargo feature; see HostBackend::default_for_features() for priority.
fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("🎨 Running with Iced backend");
        return auto_lang::ui::HostBackend::Iced.run::<Counter>();
    }


    #[cfg(not(feature = "ui-iced"))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --example ui_counter --features ui-iced\n\
                .into(),
        )
    }
}
