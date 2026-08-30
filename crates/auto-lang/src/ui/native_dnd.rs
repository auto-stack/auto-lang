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
//! Win32/COM 调用仅出现在 `win32` 子模块（`#[cfg(all(windows,
//! feature = "native-dnd"))]`——与 native_dock 同型双门控，非 Windows /
//! 未开 feature 时 natives 走 `vm/native.rs` 同名降级 shim，返 false、
//! 事件不触发）。

use std::path::PathBuf;

/// 虚拟文件（拖出即落地，用例 A2）：内容在内存，接收方（如 Explorer）
/// 经 FILEDESCRIPTOR/FILECONTENTS 流式写出真实文件。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VirtualFile {
    /// 落地文件名（含扩展名，不含路径）。
    pub name: String,
    /// 文件字节内容。
    pub bytes: Vec<u8>,
    /// MIME 类型（descriptor 侧目前仅日志用途，接收方可选消费）。
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

// Win32/COM 适配：cfg(windows) × native-dnd feature 双门控（步骤 3+：
// IDataObject/IDropSource 实现；步骤 4/5 续 STA 线程与 IDropTarget）。
#[cfg(all(windows, feature = "native-dnd"))]
pub mod win32 {
    use super::{DndPayload, VirtualFile};
    use windows::core::{implement, w, Error, HRESULT};
    use windows::Win32::Foundation::{
        BOOL, DV_E_FORMATETC, E_NOTIMPL, HGLOBAL, OLE_E_ADVISENOTSUPPORTED, POINTL,
        DRAGDROP_E_ALREADYREGISTERED, DRAGDROP_S_CANCEL, DRAGDROP_S_DROP,
        DRAGDROP_S_USEDEFAULTCURSORS, GlobalFree,
    };
    use windows::Win32::System::Com::{
        DVASPECT_CONTENT, DATADIR_GET, FORMATETC, IDataObject, IDataObject_Impl,
        IEnumFORMATETC, IEnumFORMATETC_Impl, IEnumSTATDATA, STGMEDIUM, STGMEDIUM_0, TYMED,
        TYMED_HGLOBAL,
    };
    use windows::Win32::System::DataExchange::RegisterClipboardFormatW;
    use windows::Win32::System::Memory::{
        GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
    };
    use windows::Win32::System::Ole::{
        DROPEFFECT, DROPEFFECT_NONE, IDropSource, IDropSource_Impl, IDropTarget,
        IDropTarget_Impl,
    };
    use windows::Win32::System::SystemServices::{
        MK_LBUTTON, MK_RBUTTON, MODIFIERKEYS_FLAGS,
    };
    use windows::Win32::UI::Shell::{FILEDESCRIPTORW, CFSTR_FILECONTENTS, CFSTR_FILEDESCRIPTORW};

    /// CF_UNICODETEXT（Ole 域 CLIPBOARD_FORMAT(13)；FORMATETC.cfFormat 是裸 u16）。
    const CF_UNICODETEXT_U16: u16 = 13;
    /// CF_HDROP（CLIPBOARD_FORMAT(15)）。
    const CF_HDROP_U16: u16 = 15;
    /// SDK shlobj_core.h FD_WRITESTREAM = 0x40000000——流式虚拟文件（内容经
    /// FILECONTENTS 取出）；bindings 未生成该常量，本地定义。
    /// FD_READURI 无 SDK 对应物（计划注"视接收方兼容"），v1 不置。
    const FD_WRITESTREAM: u32 = 0x4000_0000;
    /// FD_UNICODE(0x80000000) | FD_FILESIZE(64)——W 结构 + 尺寸双字。
    const FD_BASE: u32 = 0x8000_0000 | 64;

    /// FileGroupDescriptorW 的注册剪贴板格式 ID（进程内单例）。
    fn file_descriptor_cf() -> u16 {
        static CF: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        *CF.get_or_init(|| unsafe { RegisterClipboardFormatW(CFSTR_FILEDESCRIPTORW) }) as u16
    }

    /// FileContents 的注册剪贴板格式 ID（进程内单例）。
    fn file_contents_cf() -> u16 {
        static CF: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        *CF.get_or_init(|| unsafe { RegisterClipboardFormatW(CFSTR_FILECONTENTS) }) as u16
    }

    fn formatetc(cf: u16, lindex: i32) -> FORMATETC {
        FORMATETC {
            cfFormat: cf,
            ptd: std::ptr::null_mut(),
            dwAspect: DVASPECT_CONTENT.0 as u32,
            lindex,
            tymed: TYMED_HGLOBAL.0 as u32,
        }
    }

    /// FILEGROUPDESCRIPTORW 字节构造：cItems + N × FILEDESCRIPTORW（flags =
    /// UNICODE|FILESIZE|WRITESTREAM，尺寸双字，cFileName 定长 260 u16）。
    /// Rust 侧 fgd 是柔性数组习语（[T;1]），N>1 时逐个 struct 序列化。
    fn build_file_group_descriptorw(vfiles: &[VirtualFile]) -> Vec<u8> {
        let fd_size = std::mem::size_of::<FILEDESCRIPTORW>();
        let mut bytes = Vec::with_capacity(4 + vfiles.len() * fd_size);
        bytes.extend_from_slice(&(vfiles.len() as u32).to_le_bytes());
        for vf in vfiles {
            let mut fd = FILEDESCRIPTORW::default();
            fd.dwFlags = FD_BASE | FD_WRITESTREAM;
            fd.nFileSizeHigh = ((vf.bytes.len() as u64) >> 32) as u32;
            fd.nFileSizeLow = vf.bytes.len() as u32;
            let units: Vec<u16> = vf.name.encode_utf16().collect();
            // packed 结构：字段引用/as_mut_ptr 均被禁（E0793），addr_of_mut 裸写。
            unsafe {
                let dst = std::ptr::addr_of_mut!(fd.cFileName) as *mut u16;
                std::ptr::copy_nonoverlapping(units.as_ptr(), dst, units.len());
            }
            bytes.extend_from_slice(unsafe {
                std::slice::from_raw_parts(
                    (&fd as *const FILEDESCRIPTORW) as *const u8,
                    fd_size,
                )
            });
        }
        bytes
    }

    /// 拖出数据源：DndPayload 多格式同挂的 IDataObject。
    /// 格式族（EnumFormatEtc 定序）：文本(CF_UNICODETEXT) → 真实文件
    /// (CF_HDROP) → 虚拟文件(FileGroupDescriptorW + FileContents[lindex])。
    /// SetData/通知系按源侧最小语义（E_NOTIMPL / OLE_E_ADVISENOTSUPPORTED）。
    #[implement(IDataObject)]
    pub struct DndDataObject {
        payload: DndPayload,
    }

    impl DndDataObject {
        pub fn new(payload: DndPayload) -> Self {
            Self { payload }
        }

        fn formats(&self) -> Vec<FORMATETC> {
            let mut v = Vec::new();
            if self.payload.text.is_some() {
                v.push(formatetc(CF_UNICODETEXT_U16, -1));
            }
            if !self.payload.files.is_empty() {
                v.push(formatetc(CF_HDROP_U16, -1));
            }
            if !self.payload.virtual_files.is_empty() {
                v.push(formatetc(file_descriptor_cf(), -1));
                v.push(formatetc(file_contents_cf(), 0));
            }
            v
        }

        /// QueryGetData/GetData 共用的格式匹配（tymed 必须 HGLOBAL；
        /// FileContents 按 lindex 精确选文件；普通格式接受 -1/0 的"整体"）。
        fn matches(&self, f: &FORMATETC) -> bool {
            if f.tymed & (TYMED_HGLOBAL.0 as u32) == 0 {
                return false;
            }
            let whole = f.lindex == -1 || f.lindex == 0;
            if f.cfFormat == CF_UNICODETEXT_U16 {
                whole && self.payload.text.is_some()
            } else if f.cfFormat == CF_HDROP_U16 {
                whole && !self.payload.files.is_empty()
            } else if f.cfFormat == file_descriptor_cf() {
                whole && !self.payload.virtual_files.is_empty()
            } else if f.cfFormat == file_contents_cf() {
                f.lindex >= 0 && (f.lindex as usize) < self.payload.virtual_files.len()
            } else {
                false
            }
        }

        /// 产出目标格式的字节（GetData 的 HGLOBAL 内容）。
        fn render(&self, f: &FORMATETC) -> Result<Vec<u8>, Error> {
            if !self.matches(f) {
                return Err(Error::from(DV_E_FORMATETC));
            }
            if f.cfFormat == CF_UNICODETEXT_U16 {
                let s = self.payload.text.as_deref().unwrap_or("");
                let mut bytes: Vec<u8> = s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
                bytes.extend_from_slice(&0u16.to_le_bytes());
                Ok(bytes)
            } else if f.cfFormat == CF_HDROP_U16 {
                let paths: Vec<String> = self
                    .payload
                    .files
                    .iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
                Ok(crate::ui::clipboard_native::build_dropfiles(&paths))
            } else if f.cfFormat == file_descriptor_cf() {
                Ok(build_file_group_descriptorw(&self.payload.virtual_files))
            } else {
                // matches() 已保证 lindex 在界内。
                Ok(self.payload.virtual_files[f.lindex as usize].bytes.clone())
            }
        }
    }

    /// HGLOBAL ← 字节（GMEM_MOVEABLE；所有权经 STGMEDIUM 交调用方）。
    fn hglobal_from_bytes(bytes: &[u8]) -> Result<HGLOBAL, Error> {
        unsafe {
            let hg = GlobalAlloc(GMEM_MOVEABLE, bytes.len())?;
            let ptr = GlobalLock(hg);
            if ptr.is_null() {
                let _ = GlobalFree(hg);
                return Err(Error::from(E_NOTIMPL));
            }
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
            let _ = GlobalUnlock(hg);
            Ok(hg)
        }
    }

    /// HGLOBAL → 字节（测试读回用；GlobalSize 定长整块复制）。
    fn hglobal_to_bytes(hg: HGLOBAL) -> Option<Vec<u8>> {
        unsafe {
            let size = GlobalSize(hg);
            if size == 0 {
                return None;
            }
            let ptr = GlobalLock(hg);
            if ptr.is_null() {
                return None;
            }
            let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
            let _ = GlobalUnlock(hg);
            Some(bytes)
        }
    }

    fn stg_hglobal(hg: HGLOBAL) -> STGMEDIUM {
        STGMEDIUM {
            tymed: TYMED_HGLOBAL.0 as u32,
            u: STGMEDIUM_0 { hGlobal: hg },
            pUnkForRelease: std::mem::ManuallyDrop::new(None),
        }
    }

    // windows 0.58：trait 实现挂宏生成的 `<T>_Impl` 类型（Deref 透传字段）。
    #[allow(non_snake_case)]
    impl IDataObject_Impl for DndDataObject_Impl {
        fn GetData(&self, pformatetcin: *const FORMATETC) -> windows::core::Result<STGMEDIUM> {
            let bytes = self.render(unsafe { &*pformatetcin })?;
            Ok(stg_hglobal(hglobal_from_bytes(&bytes)?))
        }

        fn GetDataHere(
            &self,
            _pformatetc: *const FORMATETC,
            _pmedium: *mut STGMEDIUM,
        ) -> windows::core::Result<()> {
            Err(Error::from(E_NOTIMPL))
        }

        fn QueryGetData(&self, pformatetc: *const FORMATETC) -> HRESULT {
            if self.matches(unsafe { &*pformatetc }) {
                HRESULT(0)
            } else {
                DV_E_FORMATETC
            }
        }

        fn GetCanonicalFormatEtc(
            &self,
            _pformatectin: *const FORMATETC,
            _pformatetcout: *mut FORMATETC,
        ) -> HRESULT {
            E_NOTIMPL
        }

        fn SetData(
            &self,
            _pformatetc: *const FORMATETC,
            _pmedium: *const STGMEDIUM,
            _frelease: BOOL,
        ) -> windows::core::Result<()> {
            Err(Error::from(E_NOTIMPL))
        }

        fn EnumFormatEtc(&self, dwdirection: u32) -> windows::core::Result<IEnumFORMATETC> {
            if dwdirection != DATADIR_GET.0 as u32 {
                return Err(Error::from(E_NOTIMPL));
            }
            Ok(FormatEtcEnum {
                items: self.formats(),
                pos: std::cell::Cell::new(0),
            }
            .into())
        }

        fn DAdvise(
            &self,
            _pformatetc: *const FORMATETC,
            _advf: u32,
            _padvsink: Option<&windows::Win32::System::Com::IAdviseSink>,
        ) -> windows::core::Result<u32> {
            Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
        }

        fn DUnadvise(&self, _dwconnection: u32) -> windows::core::Result<()> {
            Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
        }

        fn EnumDAdvise(&self) -> windows::core::Result<IEnumSTATDATA> {
            Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
        }
    }

    /// 格式枚举器（EnumFormatEtc 用）：格式快照 + Cell 游标。
    #[implement(IEnumFORMATETC)]
    struct FormatEtcEnum {
        items: Vec<FORMATETC>,
        pos: std::cell::Cell<usize>,
    }

    #[allow(non_snake_case)]
    impl IEnumFORMATETC_Impl for FormatEtcEnum_Impl {
        fn Next(
            &self,
            celt: u32,
            rgelt: *mut FORMATETC,
            pceltfetched: *mut u32,
        ) -> HRESULT {
            let celt = celt as usize;
            let remaining = self.items.len().saturating_sub(self.pos.get());
            let take = celt.min(remaining);
            unsafe {
                for (i, item) in self.items[self.pos.get()..][..take].iter().enumerate() {
                    *rgelt.add(i) = *item;
                }
                if !pceltfetched.is_null() {
                    *pceltfetched = take as u32;
                }
            }
            self.pos.set(self.pos.get() + take);
            if take == celt {
                HRESULT(0)
            } else {
                HRESULT(1) // S_FALSE
            }
        }

        fn Skip(&self, celt: u32) -> windows::core::Result<()> {
            let new_pos = self.pos.get() + celt as usize;
            if new_pos > self.items.len() {
                self.pos.set(self.items.len());
                return Err(Error::from(HRESULT(1))); // S_FALSE
            }
            self.pos.set(new_pos);
            Ok(())
        }

        fn Reset(&self) -> windows::core::Result<()> {
            self.pos.set(0);
            Ok(())
        }

        fn Clone(&self) -> windows::core::Result<IEnumFORMATETC> {
            Ok(FormatEtcEnum {
                items: self.items.clone(),
                pos: std::cell::Cell::new(self.pos.get()),
            }
            .into())
        }
    }

    /// 拖出反馈源：Esc 取消 / 拖拽键全放 = 落下 / 其余继续；光标走系统默认。
    #[implement(IDropSource)]
    pub struct DndDropSource;

    impl IDropSource_Impl for DndDropSource_Impl {
        fn QueryContinueDrag(
            &self,
            fescapepressed: BOOL,
            grfkeystate: MODIFIERKEYS_FLAGS,
        ) -> HRESULT {
            if fescapepressed.as_bool() {
                DRAGDROP_S_CANCEL
            } else if grfkeystate.0 & (MK_LBUTTON.0 | MK_RBUTTON.0) == 0 {
                DRAGDROP_S_DROP
            } else {
                HRESULT(0)
            }
        }

        fn GiveFeedback(&self, _dweffect: DROPEFFECT) -> HRESULT {
            DRAGDROP_S_USEDEFAULTCURSORS
        }
    }

    /// 步骤 2 共存 spike 的最小放置目标：全方法 no-op（探针只验注册语义，
    /// 不消费数据——真实现见步骤 5）。
    #[implement(IDropTarget)]
    pub struct SpikeDropTarget;

    #[allow(non_snake_case)]
    impl IDropTarget_Impl for SpikeDropTarget_Impl {
        fn DragEnter(
            &self,
            _pdataobj: Option<&windows::Win32::System::Com::IDataObject>,
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
            _pdataobj: Option<&windows::Win32::System::Com::IDataObject>,
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
        use windows::Win32::System::Com::DATADIR_SET;
        use windows::Win32::System::LibraryLoader::GetModuleHandleW;
        use windows::Win32::System::Ole::{
            OleInitialize, OleUninitialize, RegisterDragDrop, ReleaseStgMedium, RevokeDragDrop,
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

        // ---------- T1：IDataObject / IDropSource 单元（本进程直调，无需真拖） ----------

        fn full_payload() -> DndPayload {
            DndPayload {
                text: Some("你好 dnd".into()),
                files: vec!["C:/tmp/a.txt".into(), "C:/tmp/b.png".into()],
                virtual_files: vec![
                    VirtualFile {
                        name: "note.md".into(),
                        bytes: b"# hi".to_vec(),
                        mime: "text/markdown".into(),
                    },
                    VirtualFile {
                        name: "data.bin".into(),
                        bytes: vec![0, 1, 2, 255],
                        mime: "application/octet-stream".into(),
                    },
                ],
            }
        }

        fn text_only_payload() -> DndPayload {
            DndPayload {
                text: Some("plain".into()),
                ..Default::default()
            }
        }

        #[test]
        fn t1_enum_format_etc_order() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                let en = obj.EnumFormatEtc(DATADIR_GET.0 as u32).expect("enum");
                let mut buf = [FORMATETC::default(); 4];
                let mut fetched = 0u32;
                let hr = en.Next(&mut buf, Some(&mut fetched));
                assert_eq!(hr, HRESULT(0));
                assert_eq!(fetched, 4, "text+files+descriptor+contents");
                // 计划定序：文本 → HDROP → FILEDESCRIPTOR → FILECONTENTS。
                assert_eq!(buf[0].cfFormat, CF_UNICODETEXT_U16);
                assert_eq!(buf[1].cfFormat, CF_HDROP_U16);
                assert_eq!(buf[2].cfFormat, file_descriptor_cf());
                assert_eq!(buf[3].cfFormat, file_contents_cf());
                for f in &buf[..4] {
                    assert_eq!(f.tymed, TYMED_HGLOBAL.0 as u32);
                }
                // 取尽后再取：S_FALSE + 0。
                let mut extra = [FORMATETC::default(); 1];
                let hr2 = en.Next(&mut extra, Some(&mut fetched));
                assert_eq!(hr2, HRESULT(1), "S_FALSE");
                assert_eq!(fetched, 0);

                // Reset 回首格式；再 Reset 后 Skip 前进到第 4 格式。
                en.Reset().expect("reset");
                let mut again = [FORMATETC::default(); 1];
                let hr3 = en.Next(&mut again, None);
                assert_eq!(hr3, HRESULT(0));
                assert_eq!(again[0].cfFormat, CF_UNICODETEXT_U16, "reset 后回到首格式");
                en.Reset().expect("reset 2");
                en.Skip(3).expect("skip");
                let mut after_skip = [FORMATETC::default(); 1];
                let hr4 = en.Next(&mut after_skip, None);
                assert_eq!(hr4, HRESULT(0));
                assert_eq!(after_skip[0].cfFormat, file_contents_cf());
            }
        }

        #[test]
        fn t1_enum_direction_set_rejected() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(text_only_payload()).into();
                let err = obj.EnumFormatEtc(DATADIR_SET.0 as u32).unwrap_err();
                assert_eq!(err.code(), E_NOTIMPL);
            }
        }

        #[test]
        fn t1_query_get_data_hits_and_misses() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                assert_eq!(obj.QueryGetData(&formatetc(CF_UNICODETEXT_U16, -1)), HRESULT(0));
                assert_eq!(obj.QueryGetData(&formatetc(CF_HDROP_U16, -1)), HRESULT(0));
                assert_eq!(
                    obj.QueryGetData(&formatetc(file_descriptor_cf(), -1)),
                    HRESULT(0)
                );
                assert_eq!(
                    obj.QueryGetData(&formatetc(file_contents_cf(), 0)),
                    HRESULT(0)
                );
                assert_eq!(
                    obj.QueryGetData(&formatetc(file_contents_cf(), 1)),
                    HRESULT(0)
                );
                // 未注册格式 / 越界 lindex / 非 HGLOBAL tymed。
                let unknown = RegisterClipboardFormatW(w!("AutoLangDndUnknown")) as u16;
                assert_eq!(obj.QueryGetData(&formatetc(unknown, -1)), DV_E_FORMATETC);
                assert_eq!(
                    obj.QueryGetData(&formatetc(file_contents_cf(), 2)),
                    DV_E_FORMATETC
                );
                let mut file_tymed = formatetc(CF_UNICODETEXT_U16, -1);
                file_tymed.tymed = 4; // TYMED_FILE
                assert_eq!(obj.QueryGetData(&file_tymed), DV_E_FORMATETC);

                // 文本独占载荷：HDROP 应未挂。
                let text_only: IDataObject = DndDataObject::new(text_only_payload()).into();
                assert_eq!(
                    text_only.QueryGetData(&formatetc(CF_HDROP_U16, -1)),
                    DV_E_FORMATETC
                );
            }
        }

        #[test]
        fn t1_get_data_text_roundtrip() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                let mut medium = obj.GetData(&formatetc(CF_UNICODETEXT_U16, -1)).expect("text");
                assert_eq!(medium.tymed, TYMED_HGLOBAL.0 as u32);
                let bytes = hglobal_to_bytes(medium.u.hGlobal).expect("read hglobal");
                let units: Vec<u16> = bytes
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                let s = String::from_utf16_lossy(&units);
                assert_eq!(s, "你好 dnd\0");
                ReleaseStgMedium(&mut medium);
            }
        }

        #[test]
        fn t1_get_data_hdrop_roundtrip() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                let mut medium = obj.GetData(&formatetc(CF_HDROP_U16, -1)).expect("hdrop");
                let bytes = hglobal_to_bytes(medium.u.hGlobal).expect("read hglobal");
                let paths = crate::ui::clipboard_native::parse_dropfiles(&bytes);
                assert_eq!(
                    paths,
                    vec!["C:/tmp/a.txt".to_string(), "C:/tmp/b.png".to_string()]
                );
                ReleaseStgMedium(&mut medium);
            }
        }

        #[test]
        fn t1_get_data_file_group_descriptor_bytes() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                let mut medium = obj
                    .GetData(&formatetc(file_descriptor_cf(), -1))
                    .expect("descriptor");
                let bytes = hglobal_to_bytes(medium.u.hGlobal).expect("read hglobal");
                ReleaseStgMedium(&mut medium);

                // 字节级断言：cItems + 2 × FILEDESCRIPTORW。
                let fd_size = std::mem::size_of::<FILEDESCRIPTORW>();
                assert!(bytes.len() >= 4 + 2 * fd_size);
                let c_items = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                assert_eq!(c_items, 2);
                let fd0: FILEDESCRIPTORW = std::ptr::read(bytes[4..].as_ptr() as *const FILEDESCRIPTORW);
                assert_eq!(fd0.dwFlags & FD_WRITESTREAM, FD_WRITESTREAM, "流式虚拟文件");
                assert_eq!(fd0.dwFlags & FD_BASE, FD_BASE, "UNICODE|FILESIZE");
                assert_eq!(
                    ((fd0.nFileSizeHigh as u64) << 32) | fd0.nFileSizeLow as u64,
                    4
                );
                let mut name0_buf = [0u16; 260];
                unsafe {
                    let src = std::ptr::addr_of!(fd0.cFileName) as *const u16;
                    std::ptr::copy_nonoverlapping(src, name0_buf.as_mut_ptr(), 260);
                }
                assert_eq!(
                    String::from_utf16_lossy(&name0_buf).trim_end_matches('\0'),
                    "note.md"
                );
                let fd1: FILEDESCRIPTORW =
                    std::ptr::read(bytes[4 + fd_size..].as_ptr() as *const FILEDESCRIPTORW);
                assert_eq!(
                    ((fd1.nFileSizeHigh as u64) << 32) | fd1.nFileSizeLow as u64,
                    4
                );
                let mut name1_buf = [0u16; 260];
                unsafe {
                    let src = std::ptr::addr_of!(fd1.cFileName) as *const u16;
                    std::ptr::copy_nonoverlapping(src, name1_buf.as_mut_ptr(), 260);
                }
                assert_eq!(
                    String::from_utf16_lossy(&name1_buf).trim_end_matches('\0'),
                    "data.bin"
                );
            }
        }

        #[test]
        fn t1_get_data_file_contents_by_index() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                let mut m0 = obj
                    .GetData(&formatetc(file_contents_cf(), 0))
                    .expect("contents 0");
                let b0 = hglobal_to_bytes(m0.u.hGlobal).expect("read");
                assert_eq!(b0, b"# hi".to_vec());
                ReleaseStgMedium(&mut m0);

                let mut m1 = obj
                    .GetData(&formatetc(file_contents_cf(), 1))
                    .expect("contents 1");
                let b1 = hglobal_to_bytes(m1.u.hGlobal).expect("read");
                assert_eq!(b1, vec![0, 1, 2, 255]);
                ReleaseStgMedium(&mut m1);

                // 越界：DV_E_FORMATETC。
                let err = match obj.GetData(&formatetc(file_contents_cf(), 2)) {
                    Err(e) => e,
                    Ok(_) => panic!("越界 lindex 应 DV_E_FORMATETC"),
                };
                assert_eq!(err.code(), DV_E_FORMATETC);
            }
        }

        #[test]
        fn t1_source_side_minimal_semantics() {
            unsafe {
                let obj: IDataObject = DndDataObject::new(full_payload()).into();
                // SetData → E_NOTIMPL（源侧最小）。
                let err = obj
                    .SetData(&formatetc(CF_UNICODETEXT_U16, -1), std::ptr::null(), false)
                    .unwrap_err();
                assert_eq!(err.code(), E_NOTIMPL);
                // GetCanonicalFormatEtc → E_NOTIMPL。
                assert_eq!(
                    obj.GetCanonicalFormatEtc(&formatetc(CF_UNICODETEXT_U16, -1), std::ptr::null_mut()),
                    E_NOTIMPL
                );
            }
        }

        #[test]
        fn t1_drop_source_query_continue() {
            let src: IDropSource = DndDropSource.into();
            unsafe {
                assert_eq!(src.QueryContinueDrag(true, MK_LBUTTON), DRAGDROP_S_CANCEL);
                assert_eq!(
                    src.QueryContinueDrag(false, MODIFIERKEYS_FLAGS(0)),
                    DRAGDROP_S_DROP
                );
                assert_eq!(src.QueryContinueDrag(false, MK_LBUTTON), HRESULT(0));
                assert_eq!(src.QueryContinueDrag(false, MK_RBUTTON), HRESULT(0));
                assert_eq!(src.GiveFeedback(DROPEFFECT_NONE), DRAGDROP_S_USEDEFAULTCURSORS);
            }
        }
    }
}
