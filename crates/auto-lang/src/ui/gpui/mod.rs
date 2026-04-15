//! GPUI backend - renders View<M> using the GPUI framework

mod renderer;

pub use renderer::{IntoGpuiElement, ComponentGpui, GpuiContext, GpuiMessageBridge, run_app};
