// Slider example
//
// Demonstrates slider widgets and value handling
//
// Run with:
//   cargo run --example ui_slider --features ui-iced
//   cargo run --example ui_slider --features ui-gpui

use auto_lang::ui::{Component, View, App};

#[derive(Debug, Default)]
struct SliderExample {
    value: f32,
    volume: f32,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ValueChanged(f32),
    VolumeChanged(f32),
}

impl Component for SliderExample {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ValueChanged(value) => self.value = value,
            Message::VolumeChanged(volume) => self.volume = volume,
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(30)
            .padding(40)
            .child(View::text("Slider Controls"))
            .child(
                // Value Slider Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Value:"))
                    .child(View::text(format!("{:.2}", self.value)))
                    .child(View::slider(0.0..=100.0, self.value, Message::ValueChanged).build())
                    .build()
            )
            .child(
                // Volume Slider Section
                View::col()
                    .spacing(10)
                    .padding(20)
                    .child(View::text("Volume:"))
                    .child(View::text(format!("{:.1}%", self.volume * 100.0)))
                    .child(
                        View::slider(0.0..=1.0, self.volume, Message::VolumeChanged)
                            .step(0.01)
                            .build()
                    )
                    .build()
            )
            .build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    #[cfg(feature = "ui-iced")]
    {
        println!("🎨 Running slider example with Iced backend");
        return auto_lang::ui::iced::run_app::<SliderExample>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        println!("🎨 Running slider example with GPUI backend");
        return auto_lang::ui::gpui::run_app::<SliderExample>("Slider - AutoUI");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --example ui_slider --features ui-iced\n\
             • cargo run --example ui_slider --features ui-gpui"
                .into(),
        )
    }
}
