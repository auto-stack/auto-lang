// ICED backend - renders View<M> using the Iced GUI framework
//
// This module provides adapter traits to convert the abstract View<M>
// into Iced's Element for rendering, with full style support via IcedStyle.

mod renderer;

pub use renderer::{IntoIcedElement, ComponentIced, IcedMessage, run_app, run_dynamic_iced};
