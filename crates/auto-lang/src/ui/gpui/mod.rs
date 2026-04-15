//! GPUI backend - renders View<M> using the GPUI framework

mod renderer;
pub mod auto_render;
pub mod vnode_entity;

pub use renderer::{IntoGpuiElement, ComponentGpui, GpuiContext, GpuiMessageBridge, run_app};
pub use auto_render::{GpuiComponentState, ViewExt};
pub use vnode_entity::VNodeEntity;
