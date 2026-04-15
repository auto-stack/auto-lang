// Unified Accordion Example - Works with BOTH Iced and GPUI backends!
//
// This demonstrates the Accordion (collapsible sections) component.
//
// Run with:
//   cargo run -p auto-lang --example ui_accordion --features ui-iced
//   cargo run -p auto-lang --example ui_accordion --features ui-gpui

use auto_lang::ui::{Component, View};
use auto_lang::ui::view::AccordionItem;

#[derive(Debug)]
struct AccordionApp {
    // Track which sections are expanded
    expanded_sections: Vec<bool>,
}

impl Default for AccordionApp {
    fn default() -> Self {
        Self {
            expanded_sections: vec![true, false, false, false], // First section expanded by default
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Message {
    ToggleSection(usize, bool),
}

impl Component for AccordionApp {
    type Msg = Message;

    fn on(&mut self, msg: Self::Msg) {
        match msg {
            Message::ToggleSection(index, expanded) => {
                if index < self.expanded_sections.len() {
                    self.expanded_sections[index] = expanded;
                }
            }
        }
    }

    fn view(&self) -> View<Self::Msg> {
        View::col()
            .spacing(20)
            .padding(20)
            .child(View::text("Accordion Example".to_string()))
            .child(View::text("Click on section headers to expand/collapse".to_string()))
            .child(
                View::accordion()
                    .items(vec![
                        AccordionItem::new("Getting Started")
                            .with_icon('\u{1f3e0}')
                            .with_children(vec![
                                View::text("\u{2022} Home Page".to_string()),
                                View::text("\u{2022} Hello World".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[0]),
                        AccordionItem::new("Basic Widgets")
                            .with_icon('\u{1f4e6}')
                            .with_children(vec![
                                View::text("\u{2022} Button".to_string()),
                                View::text("\u{2022} Checkbox".to_string()),
                                View::text("\u{2022} Slider".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[1]),
                        AccordionItem::new("Forms & Input")
                            .with_icon('\u{1f4dd}')
                            .with_children(vec![
                                View::text("\u{2022} Text Input".to_string()),
                                View::text("\u{2022} Select".to_string()),
                                View::text("\u{2022} Todos".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[2]),
                        AccordionItem::new("Layout & Style")
                            .with_icon('\u{1f3a8}')
                            .with_children(vec![
                                View::text("\u{2022} Layout".to_string()),
                                View::text("\u{2022} Container".to_string()),
                            ])
                            .with_expanded(self.expanded_sections[3]),
                    ])
                    .allow_multiple(true)
                    .on_toggle(|idx, expanded| Message::ToggleSection(idx, expanded))
                    .build()
            )
            .build()
    }
}

fn main() -> auto_lang::ui::AppResult<()> {
    // The same code works with both backends!
    // Just change the feature flag in Cargo.toml or CLI:
    //   --features ui-iced   -> Iced backend
    //   --features ui-gpui   -> GPUI backend

    #[cfg(feature = "ui-iced")]
    {
        return auto_lang::ui::iced::run_app::<AccordionApp>();
    }

    #[cfg(feature = "ui-gpui")]
    {
        return auto_lang::ui::gpui::run_app::<AccordionApp>("Accordion Example");
    }

    #[cfg(not(any(feature = "ui-iced", feature = "ui-gpui")))]
    {
        Err("No backend enabled. Please enable either 'ui-iced' or 'ui-gpui' feature in Cargo.toml.".into())
    }
}
