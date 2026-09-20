pub mod widget;

pub use widget::{DEFAULT_BG, PAD, Terminal, TerminalState, CELL_H, CELL_W, cell_w};
pub use widget::rgb_u32;

// PLAN-025 T-01 滚动闪屏 headless 复现器(重排计数曲线 + 泵滞后模型)。
#[cfg(all(test, feature = "iced-layout-tests"))]
#[path = "p025_scroll_render_tests.rs"]
mod p025_scroll_render_tests;
