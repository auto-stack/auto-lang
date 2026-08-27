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
// Phase 3 v1 余量（登记）：markdown 输入规则未接线（Enter 块内软换行不
// 拆块）；块首退格不跨块合并；选区/拖选不跨块；IME 手验清单（微软拼音，
// 413 清单）待实机执行。

#[cfg(all(feature = "autodown", feature = "code-editor"))]
pub mod core;

#[cfg(all(feature = "autodown", feature = "code-editor"))]
pub use core::{
    autodown_editor, autodown_editor_dispose, autodown_editor_sync, autodown_editor_text,
    storage_key, AutodownEditorCore, DocDrawList, DocInput, DocLayout, DocOutput, DocRun,
};
