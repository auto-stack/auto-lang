// ICED backend - renders View<M> using the Iced GUI framework
//
// This module provides adapter traits to convert the abstract View<M>
// into Iced's Element for rendering, with full style support via IcedStyle.

mod layout_collector;
mod renderer;

pub use layout_collector::{BoundsMap, LayoutCollector};
pub use renderer::{IntoIcedElement, ComponentIced, IcedMessage, run_app, run_app_with_task, run_dynamic_iced, last_input_text};
