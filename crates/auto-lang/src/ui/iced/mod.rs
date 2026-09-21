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

/// `progress` 的 `onseek` 在 iced 端的指针承载（可拖拽进度条）。
pub mod seek_area;
// PLAN-002 B: 布局件 hover 态 widget(row/col/div 的 `hover:` 变体类消费;
// 共享标志 + request_redraw,无 view 重建)。
pub mod hover_area;
// PLAN-631 F-7: 指针按下记忆根 wrapper(popover placement "pointer" 锚源;
// 窗口根单包装,ButtonPressed 现场记账窗口逻辑坐标到会话级单槽,纯委托)。
pub mod right_press_area;
// PLAN-631 F-7: pointer placement headless 测试(iced_test 管线,
// iced-layout-tests 门控)。
#[cfg(all(test, feature = "iced-layout-tests"))]
mod pointer_placement_tests;
// Plan 547: native display surface backed by the shared media registry.
pub mod image_surface;
// Plan 563: pen 事件层 widget(canvas onpenstart/onpenmove/onpenend 承载,
// 按下门控 + ≤30Hz 限频 + 出界收笔,PointerArea 同型扩展)。
pub mod pen_area;
pub mod renderer;
// Plan 045 T3: 表格列宽拖拽 widget（View::Table::on_col_resize 的 iced
// 承载——自持网格布局 + Drag 态临时宽实时重排 + 松手落定消息）。
pub mod table_resize;
// Plan 497 T2: 每窗口真缩略快照核心(裁剪式整窗快照,T1 定案)。
pub mod snapshot;
// Plan 515 D1: native 窗口真图标（HICON→RGBA）缓存（486 占位清偿）。
pub mod native_icon;
// PLAN-018: iconfile:<stem> 双主题位图图标后端（回退链 iconfile→hicon→lucide）。
pub mod icon_file;

/// PLAN-617 后续：lucide 全量字形表（由 scripts/gen-lucide-table.mjs 生成，勿手改）。
mod lucide_generated;
// Plan 462 T3/T4: VirtualWindow 组合层（单 OS 窗口多 App，路线 A）。
pub mod virtual_window;
pub mod broker_surface;
// PLAN-012 W3: 整桌面等比预览（workspace_preview 布局件宿主数据面）。
pub mod workspace_preview;
// Plan 481: SelectableText 的选区纯逻辑（归一/词界/扩展/清空，全平台单测）。
pub mod selection;
// Plan 481: 可选文本 widget（text 的选择/复制变体，advanced Widget）。
pub mod selectable_text;
// PLAN-655: items-stretch 行两阶段等高布局（CSS align-items:stretch 原语）。
pub mod stretch_line;
// PLAN-656 T-06: synthetic managed scroll content iced widget（logical
// extent 布局 + draw 期 viewport 观察；capability-test 专用，非 public widget）。
pub mod managed_content;
// PLAN-656 review F-4: scrollable 六测量读回 operation（controller 注册表
// 预热/校正通道）。
pub mod scroll_state_reader;

// Plan 414 §8.2: headless layout testbench — `iced_test`-based bounds
// assertions (feature `iced-layout-tests`; see layout_tests.rs header).
#[cfg(all(test, feature = "iced-layout-tests"))]
mod layout_tests;
// PLAN-010 T8: terminal 组件像素级 headless 自动化(同款 iced_test 管线)。
#[cfg(all(test, feature = "iced-layout-tests"))]
mod terminal_pixel_tests;
// 014 直键入:terminal 键盘捕获 → 键入队列 → on_input 消息的集成轨。
#[cfg(all(test, feature = "iced-layout-tests"))]
mod p022_stack_click_tests;
mod terminal_input_tests;

pub use layout_collector::{BoundsMap, LayoutCollector};
pub use renderer::{IntoIcedElement, ComponentIced, IcedMessage, run_app, run_app_with_title, run_app_devtools, run_app_with_task, run_app_with_task_devtools, run_dynamic_iced, run_dynamic_iced_multi, run_dynamic_iced_pixels, run_native_iced_pixels, run_dynamic_desktop, run_dynamic_desktop_with_options, run_dynamic_desktop_fullscreen, DesktopOptions, last_input_text, store_input_text};
pub(crate) use renderer::encode_payload;
// PLAN-077 (auto-musk): `max-w-[N%]` 百分比上限委托 widget——layout 期按
// 父级 offered 宽度收窄 limits.max_width（iced Container::max_width 只收像素）。
pub mod max_width;
