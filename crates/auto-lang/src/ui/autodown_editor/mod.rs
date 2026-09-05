// Plan 019 Phase 3 编辑壳 — autodown 文档编辑器（cosmic-text Buffer/块）。
//
// 分层复用 Plan 413 CodeEditorCore 的两段约定：
//
//   core.rs     ① 状态机 + 绘制抽取契约 — 禁止 import iced（硬分层约束）
//   iced 下沉    ② widget 位于 ui/iced/renderer.rs 的 View lowering +
//               autodown_editor::widget（唯一 iced 依赖点）
//
// 门控：feature `autodown`（块模型单源 autodown-core crate）×
// feature `code-editor`（cosmic-text 栈）。二者缺一时 VM 臂退回只读渲染
// / textarea 降级链，不编译本模块。
//
// PLAN-048 收口后的余量台账（原 Phase 3 v1 四项：行首输入规则、跨块
// 合并、跨块选区、IME 实机——前三项已落地，余量改口径如下）：
//   已解（PLAN-048）：跨块选区/拖选（SelAnchor×2 + MouseDragged 通路，
//     shift+↑/↓ 跨块扩展、ctrl+a）；块首退格跨容器合并（same_host 闸
//     撤除，fence 边界维持不做=代码块语义登记）；行首输入规则 7 条
//     （`# `/`## `/`### `/`- `/`* `/`+ `/`> `，整块精确检定对齐 vue）；
//     编辑壳空态 placeholder（view 扩字段 + 空态浅灰渲染）。
//   余量（登记）：shift+←/→ 跨块水平扩展不做（块内 motion 不越界）；
//   粘贴/IME/结构操作不入 undo（413 蓝本同口径，打字/删除级 undo 已
//   钉死）；quote 空段 Enter 退出列表式 dissolve 不做（vue 在册，登记）；
//   缩进/反缩进（Tab 归页面焦点链）；嵌套列表 VM 不产生（parser 扁平
//   化）；范围删除跨只读 Raw 段保留；copy 拼接不含容器前缀（"> "/列表
//   标记）；IME 候选窗组合实机验证受自动化通道限制（commit 形态已真窗
//   验证，PLAN-048 T9 待澄清⑨）。

#[cfg(all(feature = "autodown", feature = "code-editor"))]
pub mod core;

// iced 下沉（autodown 隐含 ui-iced；code-editor 提供 cosmic-text 栈）。
#[cfg(all(feature = "autodown", feature = "code-editor"))]
pub mod widget;

#[cfg(all(feature = "autodown", feature = "code-editor"))]
pub use core::{
    autodown_editor, autodown_editor_dispose, autodown_editor_sync, autodown_editor_text,
    retheme_all_fence_buffers,
    storage_key, AutodownEditorCore, BlockDrawCtx, DocDrawList, DocFrame, DocInput, DocLayout,
    DocOutput, DocRun, SelAnchor, VIEW_FENCE_PREFIX,
};
