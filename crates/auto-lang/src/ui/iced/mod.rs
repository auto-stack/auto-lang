// ICED backend - renders View<M> using the Iced GUI framework
//
// This module provides adapter traits to convert the abstract View<M>
// into Iced's Element for rendering, with full style support via IcedStyle.

mod layout_collector;
// Plan 422: 锚定弹层原语(iced overlay wrapper,Tooltip 同型)。
pub mod popover;
// Plan 499 M2: 指针移动限频 widget(mouse-area onmousemove 臂承载,
// 坐标换算 + ≤30Hz 限频 + 量化去重)。
pub mod pointer_area;
pub mod renderer;
// Plan 497 T2: 每窗口真缩略快照核心(裁剪式整窗快照,T1 定案)。
pub mod snapshot;
// Plan 462 T3/T4: VirtualWindow 组合层（单 OS 窗口多 App，路线 A）。
pub mod virtual_window;
pub mod broker_surface;
// Plan 481: SelectableText 的选区纯逻辑（归一/词界/扩展/清空，全平台单测）。
pub mod selection;
// Plan 481: 可选文本 widget（text 的选择/复制变体，advanced Widget）。
pub mod selectable_text;

// Plan 414 §8.2: headless layout testbench — `iced_test`-based bounds
// assertions (feature `iced-layout-tests`; see layout_tests.rs header).
#[cfg(all(test, feature = "iced-layout-tests"))]
mod layout_tests;

pub use layout_collector::{BoundsMap, LayoutCollector};
pub use renderer::{IntoIcedElement, ComponentIced, IcedMessage, run_app, run_app_devtools, run_app_with_task, run_app_with_task_devtools, run_dynamic_iced, run_dynamic_iced_multi, run_dynamic_iced_pixels, run_dynamic_desktop, run_dynamic_desktop_with_options, run_dynamic_desktop_fullscreen, DesktopOptions, last_input_text};
pub(crate) use renderer::encode_payload;
