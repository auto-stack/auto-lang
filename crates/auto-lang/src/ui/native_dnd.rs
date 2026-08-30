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
#[cfg(all(windows, feature = "native-dnd"))]
pub mod win32 {
    use windows::core::{implement, w};
    use windows::Win32::Foundation::{POINTL, DRAGDROP_E_ALREADYREGISTERED};
    use windows::Win32::System::Com::IDataObject;
    use windows::Win32::System::Ole::{
        DROPEFFECT, DROPEFFECT_NONE, IDropTarget, IDropTarget_Impl,
    };
    use windows::Win32::System::SystemServices::MODIFIERKEYS_FLAGS;

    /// 步骤 2 共存 spike 的最小放置目标：全方法 no-op（探针只验注册语义，
    /// 不消费数据——真实现见步骤 5）。
    #[implement(IDropTarget)]
    pub struct SpikeDropTarget;

    // windows 0.58：implement 宏的 vtbl 委托要求 trait 实现挂在生成的
    // `<T>_Impl` 包装类型上（字段经 Deref 透传；wry 0.55 同型用法）。
    #[allow(non_snake_case)]
    impl IDropTarget_Impl for SpikeDropTarget_Impl {
        fn DragEnter(
            &self,
            _pdataobj: Option<&IDataObject>,
            _grfkeystate: MODIFIERKEYS_FLAGS,
            _pt: &POINTL,
            pdweffect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            unsafe { *pdweffect = DROPEFFECT_NONE };
            Ok(())
        }

        fn DragOver(
            &self,
            _grfkeystate: MODIFIERKEYS_FLAGS,
            _pt: &POINTL,
            pdweffect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            unsafe { *pdweffect = DROPEFFECT_NONE };
            Ok(())
        }

        fn DragLeave(&self) -> windows::core::Result<()> {
            Ok(())
        }

        fn Drop(
            &self,
            _pdataobj: Option<&IDataObject>,
            _grfkeystate: MODIFIERKEYS_FLAGS,
            _pt: &POINTL,
            pdweffect: *mut DROPEFFECT,
        ) -> windows::core::Result<()> {
            unsafe { *pdweffect = DROPEFFECT_NONE };
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use windows::Win32::Foundation::{HINSTANCE, HWND, LRESULT, WPARAM};
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows::Win32::System::Ole::{
            OleInitialize, OleUninitialize, RegisterDragDrop, RevokeDragDrop,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, WINDOW_EX_STYLE,
            WINDOW_STYLE, WNDCLASSW, WS_OVERLAPPED,
        };

        unsafe extern "system" fn spike_wndproc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: windows::Win32::Foundation::LPARAM,
        ) -> LRESULT {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }

        /// 步骤 2 spike（待澄清①）：自有隐藏窗口上的注册语义探针——
        /// ① Register→S_OK；② 同 HWND 二次注册→DRAGDROP_E_ALREADYREGISTERED
        /// （单 HWND 单目标，OLE 规则）；③ Revoke→再注册→S_OK。
        /// 推论：winit 0.30 宿主 HWND 已被其自带 IDropTarget 占用时，
        /// 我方注册前必须先 Revoke（见计划待澄清①结论）。
        #[test]
        fn spike_register_revoke_roundtrip() {
            unsafe {
                // 测试线程独立 STA（S_FALSE = 已初始化，同样需平衡 Uninitialize）。
                let _ = OleInitialize(None);

                let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
                let wc = WNDCLASSW {
                    lpfnWndProc: Some(spike_wndproc),
                    hInstance: HINSTANCE(hmodule.0),
                    lpszClassName: w!("auto_lang_dnd_spike"),
                    ..Default::default()
                };
                assert_ne!(RegisterClassW(&wc), 0, "RegisterClassW failed");

                // 隐藏窗口即可——探针只验注册 API，不接收真实拖放。
                let hwnd = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    w!("auto_lang_dnd_spike"),
                    w!("dnd_spike"),
                    WINDOW_STYLE(WS_OVERLAPPED.0),
                    0,
                    0,
                    100,
                    50,
                    None,
                    None,
                    Some(&HINSTANCE(hmodule.0)),
                    None,
                )
                .expect("CreateWindowExW failed");

                let target: IDropTarget = SpikeDropTarget.into();
                RegisterDragDrop(hwnd, &target).expect("first RegisterDragDrop");

                // 单 HWND 单目标：二次注册必须被拒（winit 目标在位时同型）。
                let second = RegisterDragDrop(hwnd, &target);
                assert_eq!(second.unwrap_err().code(), DRAGDROP_E_ALREADYREGISTERED);

                // Revoke 后可换目标重注——winit 目标的替换路径。
                RevokeDragDrop(hwnd).expect("RevokeDragDrop");
                RegisterDragDrop(hwnd, &target).expect("re-RegisterDragDrop");
                RevokeDragDrop(hwnd).expect("final RevokeDragDrop");

                let _ = DestroyWindow(hwnd);
                OleUninitialize();
            }
        }
    }
}
