// ICED backend - renders View<M> using the Iced GUI framework
//
// This module provides adapter traits to convert the abstract View<M>
// into Iced's Element for rendering, with full style support via IcedStyle.

mod layout_collector;
pub mod renderer;

// Plan 414 §8.2: headless layout testbench — `iced_test`-based bounds
// assertions (feature `iced-layout-tests`; see layout_tests.rs header).
#[cfg(all(test, feature = "iced-layout-tests"))]
mod layout_tests;

pub use layout_collector::{BoundsMap, LayoutCollector};
pub use renderer::{IntoIcedElement, ComponentIced, IcedMessage, run_app, run_app_devtools, run_app_with_task, run_app_with_task_devtools, run_dynamic_iced, last_input_text};
pub(crate) use renderer::encode_payload;
