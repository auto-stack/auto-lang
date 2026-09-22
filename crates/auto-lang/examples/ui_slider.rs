// Slider example
//
// Demonstrates slider widgets and value handling
//
// Run with:
//   cargo run --example ui_slider --features ui-iced

use auto_lang::ui::{Component, View};

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
                    .child(View::slider(0.0..=100.0, self.value)
                            .on_change(Message::ValueChanged)
                            .build())
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
                        View::slider(0.0..=1.0, self.volume)
                            .on_change(Message::VolumeChanged)
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


    #[cfg(not(feature = "ui-iced"))]
    {
        Err(
            "❌ No backend enabled!\n\n\
             Please run with a backend feature:\n\
             • cargo run --example ui_slider --features ui-iced\n\
                .into(),
        )
    }
}
