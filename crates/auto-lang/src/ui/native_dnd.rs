//! Plan 488：原生互操作 Phase 3——OLE 拖放双向 + 虚拟文件落地。
//!
//! 拖出（desktop → native）：`DndPayload` → 自实现 `IDataObject`
//! （CF_UNICODETEXT / CF_HDROP / CFSTR_FILEDESCRIPTOR + CFSTR_FILECONTENTS）
//! + `IDropSource`，`DoDragDrop` 跑在专用 STA 线程（消息泵）。
//! 拖入（native → desktop）：自实现 `IDropTarget` 挂宿主 HWND
//! （`RegisterDragDrop`），DragEnter/Over/Leave/Drop → `DesktopMessage`
//! → 指针命中虚拟窗口 → App 级 `on_native_drop` 事件。
//!
//! 本模块承载 payload 模型与纯逻辑（格式清单/优先级），全平台可编译可单测；
//! Win32/COM 调用仅出现在 `#[cfg(all(windows, feature = "native-dnd"))]`
//! 分节（与 native_dock 同型双门控——非 Windows / 未开 feature 时 natives
//! 走 `vm/native.rs` 同名降级 shim，返 false、事件不触发）。

use std::path::PathBuf;

/// 虚拟文件（拖出即落地，用例 A2）：内容在内存，接收方（如 Explorer）
/// 经 FILEDESCRIPTOR/FILECONTENTS 流式写出真实文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualFile {
    /// 落地文件名（含扩展名，不含路径）。
    pub name: String,
    /// 文件字节内容。
    pub bytes: Vec<u8>,
    /// MIME 类型（FILEDESCRIPTOR 以 FD_READURI 附加，接收方可选消费）。
    pub mime: String,
}

/// 拖出载荷：多格式同挂（组合语义）——`text` 拖进 notepad/浏览器生效，
/// `files` 拖进 Explorer 生效，`virtual_files` 在 Explorer 侧落地为真实文件。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DndPayload {
    pub text: Option<String>,
    pub files: Vec<PathBuf>,
    pub virtual_files: Vec<VirtualFile>,
}

impl DndPayload {
    /// 是否含至少一种可拖出的数据（空载荷发起拖拽应被拒绝）。
    pub fn is_empty(&self) -> bool {
        self.text.is_none() && self.files.is_empty() && self.virtual_files.is_empty()
    }
}

// Win32/COM 适配：cfg(windows) × native-dnd feature 双门控（Plan 488 步骤 3+
// 填充：IDataObject/IDropSource/IDropTarget 实现 + STA 线程）。
