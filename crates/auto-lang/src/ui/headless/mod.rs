//! Headless backend - no-op renderer for development and testing
//!
//! This backend provides the full Component/View/VTree pipeline
//! without any window creation, GPU access, or event loop.
//! Use it for fast iteration on UI logic and transpiler output.

mod renderer;

pub use renderer::{HeadlessRenderer, run_headless};
