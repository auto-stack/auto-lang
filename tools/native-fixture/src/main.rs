//! Plan 473：可编程原生窗口夹具（tools/native-fixture）。
//!
//! 最小 Win32 顶层窗口 + stdout JSON-lines 协议，供 native dock 的
//! T3 fixture E2E（B1/B2/B3/B7 + C3/C4/C5 夹具参数化路径）自动化驱动。
//! Windows-only（真原生窗口是本夹具的存在意义）；协议见 README.md。
//!
//! 参数：
//!   --title T        窗口标题（默认 "native-fixture"）
//!   --min-size WxH   WM_GETMINMAXINFO 强制最小跟踪尺寸（不可信窗口模拟）
//!   --stubborn       周期性自我复位到初始 rect（倔强窗口，C4）
//!   --spawn-modal    窗口内按钮触发模态对话框（B5）
//!   --self-close N   N 秒后自毁（测崩溃路径，B7）
//!   --offer SPEC     Plan 488 拖源：text:<str> 或 files:<p1;p2;…>——
//!                    客户区按下左键即对该载荷发起 DoDragDrop（OLE 拖出
//!                    通道；触发面=WM_LBUTTONDOWN 而非按钮 click——真拖
//!                    拽要求按下时刻按钮处于按住态，click 时已释放）。
//!
//! 输出（每行一个 JSON 对象，stdout）：
//!   {"evt":"start","hwnd":"0x…","pid":N,"title":"…"}
//!   {"evt":"bounds","x":N,"y":N,"w":N,"h":N}   （位置/尺寸变化后回显实际值）
//!   {"evt":"click","x":N,"y":N}                （Plan 494 T3：客户区点击坐标）
//!   {"evt":"drop","formats":[…],"text":…,"files":[…]}  （Plan 488 拖入日志）
//!   {"evt":"dragend","effect":"copy|move|link|none"}   （Plan 488 拖出完成）
//!   {"evt":"close"}

#[cfg(windows)]
mod win {
    use std::io::Write as _;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::OnceLock;

    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
        GetWindowRect, GetWindowThreadProcessId, MessageBoxW, RegisterClassW, SetTimer,
        SetWindowPos, ShowWindow, TranslateMessage, MB_ICONINFORMATION, MB_OK, MINMAXINFO,
        MSG, SWP_NOACTIVATE, SWP_NOZORDER, SET_WINDOW_POS_FLAGS, SW_SHOW,
        WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_GETMINMAXINFO, WM_TIMER,
        WM_LBUTTONDOWN, WM_NCHITTEST, WM_NCLBUTTONDOWN, WM_NCLBUTTONUP, WM_SYSCOMMAND,
        WM_WINDOWPOSCHANGED, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    /// 夹具选项（进程单窗口，静态共享给 wndproc）。
    pub struct Opts {
        pub title: String,
        pub min_size: Option<(i32, i32)>,
        pub stubborn: bool,
        pub spawn_modal: bool,
        pub self_close_secs: Option<u32>,
        /// Plan 486 T4 诊断：追踪 NC 鼠标/SC 消息（stdout trace 行）。
        pub trace: bool,
        /// Plan 488 拖源载荷（None = 无拖源行为）。
        pub offer: Option<Offer>,
        /// Plan 488 T6 诊断：合成拖拽驱动（"x1,y1-x2,y2"：按下→移动→释放后退出；驱动外部进程窗口的拖放测量用）。
        pub synthdrag: Option<((i32, i32), (i32, i32))>,
    }

    /// Plan 488 `--offer` 载荷。
    #[derive(Clone)]
    pub enum Offer {
        Text(String),
        Files(Vec<String>),
    }

    impl Clone for Opts {
        fn clone(&self) -> Self {
            Self {
                title: self.title.clone(),
                min_size: self.min_size,
                stubborn: self.stubborn,
                spawn_modal: self.spawn_modal,
                self_close_secs: self.self_close_secs,
                trace: self.trace,
                offer: self.offer.clone(),
                synthdrag: self.synthdrag,
            }
        }
    }

    static OPTS: OnceLock<Opts> = OnceLock::new();
    /// 初始 rect（stubborn 复位目标；首次 bounds 事件时锁定）。
    static INIT_X: AtomicU32 = AtomicU32::new(0);
    static INIT_Y: AtomicU32 = AtomicU32::new(0);
    static INIT_W: AtomicU32 = AtomicU32::new(0);
    static INIT_H: AtomicU32 = AtomicU32::new(0);
    static INIT_LOCKED: AtomicBool = AtomicBool::new(false);
    /// 倔强窗口复位定时器（1s）与自毁定时器 ID。
    const TIMER_STUBBORN: usize = 1;
    const TIMER_SELF_CLOSE: usize = 2;
    /// 本夹具只写几何：不动 z 序、不激活（0.58 的 BitOr 非 const，运行时合成）。
    fn swp_geometry_only() -> SET_WINDOW_POS_FLAGS {
        SWP_NOZORDER | SWP_NOACTIVATE
    }
    /// 模态按钮的子控件 ID（WM_COMMAND wParam 低位）。
    const BUTTON_ID: isize = 1001;

    fn json_escape(s: &str) -> String {
        s.replace('\\', "\\\\").replace('"', "\\\"")
    }

    fn emit(line: &str) {
        // 忽略写失败（宿主 kill 后管道断开；panic 会污染测试输出）。
        let mut out = std::io::stdout();
        let _ = writeln!(out, "{line}");
        let _ = out.flush();
    }

    fn class_name() -> &'static [u16] {
        static NAME: OnceLock<Vec<u16>> = OnceLock::new();
        NAME.get_or_init(|| "auto_lang_native_fixture\0".encode_utf16().collect())
    }

    // ── Plan 488：最小 COM 拖放三件套（独立 bin，不复用 auto-lang——
    //    避免整仓编译开销；接口面与 ui/native_dnd.rs 同型）。 ─────────────

    mod dnd {
        use super::{emit, json_escape, Offer};
        use windows::core::{implement, Error, HRESULT};
        use windows::Win32::Foundation::{
            BOOL, DV_E_FORMATETC, E_NOTIMPL, HGLOBAL, HWND, OLE_E_ADVISENOTSUPPORTED, POINTL,
            DRAGDROP_S_CANCEL, DRAGDROP_S_DROP, DRAGDROP_S_USEDEFAULTCURSORS, GlobalFree,
        };
        use windows::Win32::System::Com::{
            DVASPECT_CONTENT, DATADIR_GET, FORMATETC, IDataObject, IDataObject_Impl,
            IEnumFORMATETC, IEnumFORMATETC_Impl, IEnumSTATDATA, STGMEDIUM, STGMEDIUM_0,
            TYMED_HGLOBAL,
        };
        use windows::Win32::System::Memory::{
            GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
        };
        use windows::Win32::System::Ole::{
            DROPEFFECT, DROPEFFECT_COPY, DROPEFFECT_LINK, DROPEFFECT_MOVE, DROPEFFECT_NONE,
            DoDragDrop, IDropSource, IDropSource_Impl, IDropTarget, IDropTarget_Impl,
        };
        use windows::Win32::System::SystemServices::{MK_LBUTTON, MK_RBUTTON, MODIFIERKEYS_FLAGS};

        const CF_UNICODETEXT_U16: u16 = 13;
        const CF_HDROP_U16: u16 = 15;

        fn formatetc(cf: u16, lindex: i32) -> FORMATETC {
            FORMATETC {
                cfFormat: cf,
                ptd: std::ptr::null_mut(),
                dwAspect: DVASPECT_CONTENT.0 as u32,
                lindex,
                tymed: TYMED_HGLOBAL.0 as u32,
            }
        }

        /// DROPFILES 构造（fWide=1；与 auto-lang clipboard_native 同构）。
        fn build_dropfiles(paths: &[String]) -> Vec<u8> {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&20u32.to_le_bytes());
            bytes.extend_from_slice(&0i32.to_le_bytes());
            bytes.extend_from_slice(&0i32.to_le_bytes());
            bytes.extend_from_slice(&0i32.to_le_bytes());
            bytes.extend_from_slice(&1i32.to_le_bytes());
            for p in paths {
                for u in p.encode_utf16() {
                    bytes.extend_from_slice(&u.to_le_bytes());
                }
                bytes.extend_from_slice(&0u16.to_le_bytes());
            }
            bytes.extend_from_slice(&0u16.to_le_bytes());
            bytes
        }

        /// DROPFILES 解析（fWide=1；坏形状返回空）。
        fn parse_dropfiles(bytes: &[u8]) -> Vec<String> {
            let ok = bytes.len() >= 20
                && u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) == 1;
            if !ok {
                return Vec::new();
            }
            let p = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            if p < 20 || p + 2 > bytes.len() {
                return Vec::new();
            }
            let units: Vec<u16> = bytes[p..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            let mut out = Vec::new();
            let mut cur: Vec<u16> = Vec::new();
            for u in units {
                if u == 0 {
                    if cur.is_empty() {
                        break;
                    }
                    out.push(String::from_utf16_lossy(&cur));
                    cur.clear();
                } else {
                    cur.push(u);
                }
            }
            out
        }

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
                let b = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
                let _ = GlobalUnlock(hg);
                Some(b)
            }
        }

        fn stg_hglobal(hg: HGLOBAL) -> STGMEDIUM {
            STGMEDIUM {
                tymed: TYMED_HGLOBAL.0 as u32,
                u: STGMEDIUM_0 { hGlobal: hg },
                pUnkForRelease: std::mem::ManuallyDrop::new(None),
            }
        }

        /// 已知格式名（drop 日志 formats 域用；未知名以 "cf:N" 记）。
        fn format_name(cf: u16) -> String {
            match cf {
                CF_UNICODETEXT_U16 => "CF_UNICODETEXT".into(),
                CF_HDROP_U16 => "CF_HDROP".into(),
                17 => "CF_DIBV5".into(),
                8 => "CF_DIB".into(),
                other => format!("cf:{other}"),
            }
        }

        /// 拖源数据对象：text（CF_UNICODETEXT）/ files（CF_HDROP）两种载荷。
        #[implement(IDataObject)]
        pub struct FixtureDataObject {
            pub text: Option<String>,
            pub files: Vec<String>,
        }

        impl FixtureDataObject {
            fn formats(&self) -> Vec<FORMATETC> {
                let mut v = Vec::new();
                if self.text.is_some() {
                    v.push(formatetc(CF_UNICODETEXT_U16, -1));
                }
                if !self.files.is_empty() {
                    v.push(formatetc(CF_HDROP_U16, -1));
                }
                v
            }

            fn matches(&self, f: &FORMATETC) -> bool {
                if f.tymed & (TYMED_HGLOBAL.0 as u32) == 0 {
                    return false;
                }
                let whole = f.lindex == -1 || f.lindex == 0;
                if f.cfFormat == CF_UNICODETEXT_U16 {
                    whole && self.text.is_some()
                } else {
                    f.cfFormat == CF_HDROP_U16 && whole && !self.files.is_empty()
                }
            }
        }

        #[allow(non_snake_case)]
        impl IDataObject_Impl for FixtureDataObject_Impl {
            fn GetData(&self, pformatetcin: *const FORMATETC) -> windows::core::Result<STGMEDIUM> {
                let f = unsafe { &*pformatetcin };
                if !self.matches(f) {
                    return Err(Error::from(DV_E_FORMATETC));
                }
                let bytes = if f.cfFormat == CF_UNICODETEXT_U16 {
                    let s = self.text.as_deref().unwrap_or("");
                    let mut b: Vec<u8> = s.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
                    b.extend_from_slice(&0u16.to_le_bytes());
                    b
                } else {
                    build_dropfiles(&self.files)
                };
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
                _a: *const FORMATETC,
                _b: *mut FORMATETC,
            ) -> HRESULT {
                E_NOTIMPL
            }

            fn SetData(
                &self,
                _a: *const FORMATETC,
                _b: *const STGMEDIUM,
                _c: BOOL,
            ) -> windows::core::Result<()> {
                Err(Error::from(E_NOTIMPL))
            }

            fn EnumFormatEtc(&self, dwdirection: u32) -> windows::core::Result<IEnumFORMATETC> {
                if dwdirection != DATADIR_GET.0 as u32 {
                    return Err(Error::from(E_NOTIMPL));
                }
                Ok(Enum {
                    items: self.formats(),
                    pos: std::cell::Cell::new(0),
                }
                .into())
            }

            fn DAdvise(
                &self,
                _a: *const FORMATETC,
                _b: u32,
                _c: Option<&windows::Win32::System::Com::IAdviseSink>,
            ) -> windows::core::Result<u32> {
                Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
            }

            fn DUnadvise(&self, _a: u32) -> windows::core::Result<()> {
                Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
            }

            fn EnumDAdvise(&self) -> windows::core::Result<IEnumSTATDATA> {
                Err(Error::from(OLE_E_ADVISENOTSUPPORTED))
            }
        }

        #[implement(IEnumFORMATETC)]
        struct Enum {
            items: Vec<FORMATETC>,
            pos: std::cell::Cell<usize>,
        }

        #[allow(non_snake_case)]
        impl IEnumFORMATETC_Impl for Enum_Impl {
            fn Next(
                &self,
                celt: u32,
                rgelt: *mut FORMATETC,
                pceltfetched: *mut u32,
            ) -> HRESULT {
                let celt = celt as usize;
                let take = celt.min(self.items.len().saturating_sub(self.pos.get()));
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
                    HRESULT(1)
                }
            }

            fn Skip(&self, celt: u32) -> windows::core::Result<()> {
                let np = self.pos.get() + celt as usize;
                if np > self.items.len() {
                    self.pos.set(self.items.len());
                    return Err(Error::from(HRESULT(1)));
                }
                self.pos.set(np);
                Ok(())
            }

            fn Reset(&self) -> windows::core::Result<()> {
                self.pos.set(0);
                Ok(())
            }

            fn Clone(&self) -> windows::core::Result<IEnumFORMATETC> {
                Ok(Enum {
                    items: self.items.clone(),
                    pos: std::cell::Cell::new(self.pos.get()),
                }
                .into())
            }
        }

        #[implement(IDropSource)]
        pub struct FixtureDropSource;

        impl IDropSource_Impl for FixtureDropSource_Impl {
            fn QueryContinueDrag(&self, esc: BOOL, keys: MODIFIERKEYS_FLAGS) -> HRESULT {
                use std::sync::atomic::{AtomicU64, Ordering};
                static N: AtomicU64 = AtomicU64::new(0);
                if super::OPTS.get().map(|o| o.trace).unwrap_or(false) {
                    let n = N.fetch_add(1, Ordering::Relaxed);
                    if n < 30 || n % 20 == 0 {
                        emit(&format!(
                            "{{\"evt\":\"trace\",\"msg\":\"qcd\",\"n\":{n},\"esc\":{},\"keys\":{:#x}}}",
                            esc.as_bool() as u8, keys.0
                        ));
                    }
                }
                if esc.as_bool() {
                    DRAGDROP_S_CANCEL
                } else if keys.0 & (MK_LBUTTON.0 | MK_RBUTTON.0) == 0 {
                    DRAGDROP_S_DROP
                } else {
                    HRESULT(0)
                }
            }

            fn GiveFeedback(&self, _e: DROPEFFECT) -> HRESULT {
                DRAGDROP_S_USEDEFAULTCURSORS
            }
        }

        /// 放置目标：全窗挂载，Drop 抽取 → stdout JSON lines（E2E 断言面）。
        #[implement(IDropTarget)]
        pub struct FixtureDropTarget;

        fn probe(data: &IDataObject) -> Vec<(u16, &'static str)> {
            let probes: Vec<(u16, &'static str)> = vec![
                (CF_HDROP_U16, "CF_HDROP"),
                (CF_UNICODETEXT_U16, "CF_UNICODETEXT"),
            ];
            probes
                .into_iter()
                .filter(|(cf, _)| unsafe { data.QueryGetData(&formatetc(*cf, -1)) } == HRESULT(0))
                .collect()
        }

        #[allow(non_snake_case)]
        impl IDropTarget_Impl for FixtureDropTarget_Impl {
            fn DragEnter(
                &self,
                pdataobj: Option<&IDataObject>,
                _k: MODIFIERKEYS_FLAGS,
                _pt: &POINTL,
                pdweffect: *mut DROPEFFECT,
            ) -> windows::core::Result<()> {
                let usable = pdataobj.map(|d| !probe(d).is_empty()).unwrap_or(false);
                unsafe { *pdweffect = if usable { DROPEFFECT_COPY } else { DROPEFFECT_NONE } };
                Ok(())
            }

            fn DragOver(
                &self,
                _k: MODIFIERKEYS_FLAGS,
                _pt: &POINTL,
                pdweffect: *mut DROPEFFECT,
            ) -> windows::core::Result<()> {
                unsafe { *pdweffect = DROPEFFECT_COPY };
                Ok(())
            }

            fn DragLeave(&self) -> windows::core::Result<()> {
                Ok(())
            }

            fn Drop(
                &self,
                pdataobj: Option<&IDataObject>,
                _k: MODIFIERKEYS_FLAGS,
                _pt: &POINTL,
                pdweffect: *mut DROPEFFECT,
            ) -> windows::core::Result<()> {
                unsafe { *pdweffect = DROPEFFECT_COPY };
                let Some(data) = pdataobj else { return Ok(()) };
                let mut formats = Vec::new();
                let mut text: Option<String> = None;
                let mut files: Vec<String> = Vec::new();
                for (cf, name) in probe(data) {
                    formats.push(name.to_string());
                    if let Ok(mut m) = unsafe { data.GetData(&formatetc(cf, -1)) } {
                        if let Some(b) = hglobal_to_bytes(unsafe { m.u.hGlobal }) {
                            if cf == CF_HDROP_U16 {
                                files = parse_dropfiles(&b);
                            } else if text.is_none() {
                                let units: Vec<u16> = b
                                    .chunks_exact(2)
                                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                                    .collect();
                                text = Some(
                                    String::from_utf16_lossy(&units).trim_end_matches('\0').into(),
                                );
                            }
                        }
                        unsafe {
                            windows::Win32::System::Ole::ReleaseStgMedium(&mut m);
                        }
                    }
                }
                // 枚举全格式名（未知名以 cf:N 记）补全观察面。
                if let Ok(en) = unsafe { data.EnumFormatEtc(DATADIR_GET.0 as u32) } {
                    let mut buf = [FORMATETC::default(); 16];
                    let mut fetched = 0u32;
                    if unsafe { en.Next(&mut buf, Some(&mut fetched)) } == HRESULT(0) {
                        for f in &buf[..fetched as usize] {
                            let n = format_name(f.cfFormat);
                            if !formats.contains(&n) {
                                formats.push(n);
                            }
                        }
                    }
                }
                let files_json = files
                    .iter()
                    .map(|f| format!("\"{}\"", json_escape(f)))
                    .collect::<Vec<_>>()
                    .join(",");
                emit(&format!(
                    "{{\"evt\":\"drop\",\"formats\":[{}],\"text\":{},\"files\":[{}]}}",
                    formats
                        .iter()
                        .map(|f| format!("\"{}\"", json_escape(f)))
                        .collect::<Vec<_>>()
                        .join(","),
                    text.map(|t| format!("\"{}\"", json_escape(&t)))
                        .unwrap_or_else(|| "null".into()),
                    files_json,
                ));
                Ok(())
            }
        }

        /// 发起拖出（--offer 载荷 → DoDragDrop；阻塞至落下/取消）。
        pub fn start_drag(offer: &Offer) {
            let (text, files) = match offer {
                Offer::Text(t) => (Some(t.clone()), Vec::new()),
                Offer::Files(fs) => (None, fs.clone()),
            };
            unsafe {
                let data: IDataObject = FixtureDataObject { text, files }.into();
                let source: IDropSource = FixtureDropSource.into();
                let mut effect = DROPEFFECT_NONE;
                let _ = DoDragDrop(
                    &data,
                    &source,
                    DROPEFFECT(DROPEFFECT_COPY.0 | DROPEFFECT_MOVE.0 | DROPEFFECT_LINK.0),
                    &mut effect,
                );
                let name = if effect.0 & DROPEFFECT_COPY.0 != 0 {
                    "copy"
                } else if effect.0 & DROPEFFECT_MOVE.0 != 0 {
                    "move"
                } else if effect.0 & DROPEFFECT_LINK.0 != 0 {
                    "link"
                } else {
                    "none"
                };
                emit(&format!("{{\"evt\":\"dragend\",\"effect\":\"{name}\"}}"));
            }
        }

        /// 窗口挂载拖入目标（启动时一次；无 winit 目标在位，直接注册）。
        pub fn register_target(hwnd: HWND) {
            unsafe {
                let target: IDropTarget = FixtureDropTarget.into();
                let _ = windows::Win32::System::Ole::RegisterDragDrop(hwnd, &target);
            }
        }
    }

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            match msg {
                WM_GETMINMAXINFO => {
                    if let Some(min) = OPTS.get().and_then(|o| o.min_size) {
                        let mmi = lparam.0 as *mut MINMAXINFO;
                        (*mmi).ptMinTrackSize = POINT { x: min.0, y: min.1 };
                        return LRESULT(0);
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_LBUTTONDOWN => {
                    // Plan 494 T3：点击坐标日志（client 域，JSON lines——穿透
                    // E2E 的断言源）。--offer 模式叠加拖源触发（点击先记后拖）。
                    let x = (lparam.0 & 0xFFFF) as u16 as i16 as i32;
                    let y = ((lparam.0 >> 16) & 0xFFFF) as u16 as i16 as i32;
                    emit(&format!("{{\"evt\":\"click\",\"x\":{x},\"y\":{y}}}"));
                    if OPTS.get().and_then(|o| o.offer.as_ref()).is_some() {
                        if OPTS.get().map(|o| o.trace).unwrap_or(false) {
                            emit("{\"evt\":\"trace\",\"msg\":\"offer-press\"}");
                        }
                        if let Some(offer) = OPTS.get().and_then(|o| o.offer.clone()) {
                            dnd::start_drag(&offer);
                        }
                    }
                    LRESULT(0)
                }

                WM_WINDOWPOSCHANGED => {
                    // 回显实际 rect（写后读回断言的被动响应路径）。
                    let mut r = RECT::default();
                    if GetWindowRect(hwnd, &mut r).is_ok() {
                        if !INIT_LOCKED.swap(true, Ordering::SeqCst) {
                            INIT_X.store(r.left as u32, Ordering::SeqCst);
                            INIT_Y.store(r.top as u32, Ordering::SeqCst);
                            INIT_W.store((r.right - r.left) as u32, Ordering::SeqCst);
                            INIT_H.store((r.bottom - r.top) as u32, Ordering::SeqCst);
                        }
                        emit(&format!(
                            "{{\"evt\":\"bounds\",\"x\":{},\"y\":{},\"w\":{},\"h\":{}}}",
                            r.left,
                            r.top,
                            r.right - r.left,
                            r.bottom - r.top
                        ));
                    }
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                WM_TIMER if wparam.0 == TIMER_STUBBORN => {
                    // C4 倔强窗口：周期性复位到初始 rect（不死守判定对向）。
                    let (x, y, w, h) = (
                        INIT_X.load(Ordering::SeqCst) as i32,
                        INIT_Y.load(Ordering::SeqCst) as i32,
                        INIT_W.load(Ordering::SeqCst) as i32,
                        INIT_H.load(Ordering::SeqCst) as i32,
                    );
                    if w > 0 && h > 0 {
                        let _ = SetWindowPos(hwnd, HWND::default(), x, y, w, h, swp_geometry_only());
                    }
                    LRESULT(0)
                }
                WM_TIMER if wparam.0 == TIMER_SELF_CLOSE => {
                    let _ = DestroyWindow(hwnd);
                    LRESULT(0)
                }
                WM_COMMAND => {
                    // B5：模态对话框（MessageBox 阻塞本窗口消息处理）。
                    let title = OPTS
                        .get()
                        .map(|o| o.title.clone())
                        .unwrap_or_else(|| "fixture".into());
                    let mut text: Vec<u16> = "模态对话框（fixture）\0".encode_utf16().collect();
                    let mut caption: Vec<u16> = format!("{title}\0").encode_utf16().collect();
                    let _ = MessageBoxW(
                        hwnd,
                        windows::core::PCWSTR(text.as_mut_ptr()),
                        windows::core::PCWSTR(caption.as_mut_ptr()),
                        MB_OK | MB_ICONINFORMATION,
                    );
                    LRESULT(0)
                }
                WM_DESTROY => {
                    emit("{\"evt\":\"close\"}");
                    windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                    LRESULT(0)
                }
                WM_NCLBUTTONDOWN | WM_NCLBUTTONUP | WM_SYSCOMMAND
                    if OPTS.get().map(|o| o.trace).unwrap_or(false) =>
                {
                    let ht = if msg == WM_SYSCOMMAND {
                        DefWindowProcW(hwnd, WM_NCHITTEST, WPARAM(0), lparam).0
                    } else {
                        wparam.0 as isize
                    };
                    emit(&format!(
                        "{{\"evt\":\"trace\",\"msg\":{msg},\"wparam\":{:#x},\"ht\":{ht},\"lparam\":{}}}",
                        wparam.0, lparam.0
                    ));
                    DefWindowProcW(hwnd, msg, wparam, lparam)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    pub fn run(opts: Opts) -> i32 {
        OPTS.set(opts.clone()).ok();
        unsafe {
            // 与驱动方（测试/宿主进程）对齐 DPI 感知：per-monitor v2 下
            // SetWindowPos 坐标域 = 物理像素，跨进程几何写读回不失真。
            let _ = windows::Win32::UI::HiDpi::SetProcessDpiAwarenessContext(
                windows::Win32::UI::HiDpi::DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            );
            // Plan 488：OLE STA（拖源 DoDragDrop 与拖入目标注册共用）。
            let _ = windows::Win32::System::Ole::OleInitialize(None);
            let Ok(hmodule) = GetModuleHandleW(None) else {
                return 1;
            };
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: HINSTANCE(hmodule.0),
                lpszClassName: windows::core::PCWSTR(class_name().as_ptr()),
                ..Default::default()
            };
            if RegisterClassW(&wc) == 0 {
                return 1;
            }
            let mut title_w: Vec<u16> = opts.title.encode_utf16().collect();
            title_w.push(0);
            let Ok(hwnd) = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                windows::core::PCWSTR(class_name().as_ptr()),
                windows::core::PCWSTR(title_w.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                200,
                200,
                480,
                320,
                HWND::default(),
                windows::Win32::UI::WindowsAndMessaging::HMENU::default(),
                hmodule,
                None,
            ) else {
                return 1;
            };
            let _ = ShowWindow(hwnd, SW_SHOW);

            // 启动行（hwnd/pid/title——E2E 驱动解析此行定位目标）。
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            emit(&format!(
                "{{\"evt\":\"start\",\"hwnd\":\"{:#x}\",\"pid\":{},\"title\":\"{}\"}}",
                hwnd.0 as usize,
                pid,
                json_escape(&opts.title)
            ));

            // Plan 488：全窗挂载拖入目标（drop 日志面）。
            dnd::register_target(hwnd);

            if opts.stubborn {
                SetTimer(hwnd, TIMER_STUBBORN, 1000, None);
            }
            if let Some(secs) = opts.self_close_secs {
                SetTimer(hwnd, TIMER_SELF_CLOSE, secs * 1000, None);
            }
            if opts.spawn_modal {
                create_modal_button(hwnd, hmodule);
            }

            if let Some((from, to)) = opts.synthdrag {
                // 驱动模式：窗口只为存在而建（拖拽源点/落点均在外部窗口），驱动完成即退出。
                emit("{\"evt\":\"synthdrag-begin\"}");
                synth_drag(from, to);
                emit("{\"evt\":\"synthdrag-end\"}");
                let _ = DestroyWindow(hwnd);
                return 0;
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            0
        }
    }

    /// 合成拖拽（SendInput：按下→12 步移动→释放；步间 20ms）。
    fn synth_drag(from: (i32, i32), to: (i32, i32)) {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
            MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MOVE, MOUSEINPUT, MOUSE_EVENT_FLAGS,
        };
        use windows::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
        };
        fn send(dx: i32, dy: i32, flags: MOUSE_EVENT_FLAGS) {
            let input = INPUT {
                r#type: INPUT_MOUSE,
                Anonymous: INPUT_0 {
                    mi: MOUSEINPUT {
                        dx,
                        dy,
                        mouseData: 0,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            };
            unsafe {
                SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
            }
        }
        fn move_to(x: i32, y: i32) {
            unsafe {
                let sw = GetSystemMetrics(SM_CXSCREEN).max(1);
                let sh = GetSystemMetrics(SM_CYSCREEN).max(1);
                send(
                    (x.clamp(0, sw - 1) * 65535) / (sw - 1),
                    (y.clamp(0, sh - 1) * 65535) / (sh - 1),
                    MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                );
            }
        }
        unsafe {
            move_to(from.0, from.1);
            std::thread::sleep(std::time::Duration::from_millis(150));
            send(0, 0, MOUSEEVENTF_LEFTDOWN);
            std::thread::sleep(std::time::Duration::from_millis(150));
            for i in 1..=12 {
                let t = i as f32 / 12.0;
                let x = from.0 as f32 + (to.0 as f32 - from.0 as f32) * t;
                let y = from.1 as f32 + (to.1 as f32 - from.1 as f32) * t;
                move_to(x as i32, y as i32);
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            std::thread::sleep(std::time::Duration::from_millis(80));
            send(0, 0, MOUSEEVENTF_LEFTUP);
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    unsafe fn create_modal_button(hwnd: HWND, hmodule: windows::Win32::Foundation::HMODULE) {
        use windows::Win32::UI::WindowsAndMessaging::{
            BS_PUSHBUTTON, HMENU, WINDOW_STYLE, WS_CHILD,
        };
        let class: Vec<u16> = "BUTTON\0".encode_utf16().collect();
        let label: Vec<u16> = "modal\0".encode_utf16().collect();
        let _ = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            windows::core::PCWSTR(class.as_ptr()),
            windows::core::PCWSTR(label.as_ptr()),
            (WS_CHILD | WS_VISIBLE) | WINDOW_STYLE(BS_PUSHBUTTON as u32),
            10,
            10,
            80,
            28,
            hwnd,
            HMENU(BUTTON_ID as *mut core::ffi::c_void),
            hmodule,
            None,
        );
    }
}

#[cfg(windows)]
fn main() {
    let opts = parse_args();
    std::process::exit(win::run(opts));
}

#[cfg(not(windows))]
fn main() {
    eprintln!("native-fixture 仅支持 Windows（真原生窗口夹具）");
}

#[cfg(windows)]
fn parse_args() -> win::Opts {
    let mut opts = win::Opts {
        title: "native-fixture".into(),
        min_size: None,
        stubborn: false,
        spawn_modal: false,
        self_close_secs: None,
        trace: false,
        offer: None,
        synthdrag: None,
    };
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--title" if i + 1 < args.len() => {
                opts.title = args[i + 1].clone();
                i += 2;
            }
            "--min-size" if i + 1 < args.len() => {
                // 形如 300x200（x 或 X 分隔）。
                let spec = args[i + 1].replace('X', "x");
                if let Some((w, h)) = spec.split_once('x') {
                    opts.min_size = w.trim().parse().ok().zip(h.trim().parse().ok());
                }
                i += 2;
            }
            "--stubborn" => {
                opts.stubborn = true;
                i += 1;
            }
            "--trace" => {
                opts.trace = true;
                i += 1;
            }
            "--spawn-modal" => {
                opts.spawn_modal = true;
                i += 1;
            }
            "--self-close" if i + 1 < args.len() => {
                opts.self_close_secs = args[i + 1].parse().ok();
                i += 2;
            }
            // Plan 488 拖源：text:<str> 或 files:<p1;p2;…>。
            "--offer" if i + 1 < args.len() => {
                let spec = args[i + 1].clone();
                if let Some(t) = spec.strip_prefix("text:") {
                    opts.offer = Some(win::Offer::Text(t.to_string()));
                } else if let Some(fs) = spec.strip_prefix("files:") {
                    let files = fs.split(';').map(|s| s.to_string()).collect();
                    opts.offer = Some(win::Offer::Files(files));
                }
                i += 2;
            }
            // Plan 488 T6 诊断：合成拖拽驱动 x1,y1-x2,y2。
            "--synthdrag" if i + 1 < args.len() => {
                let spec = args[i + 1].replace('-', ",");
                let nums: Vec<i32> = spec
                    .split(',')
                    .filter_map(|n| n.trim().parse().ok())
                    .collect();
                if nums.len() == 4 {
                    opts.synthdrag = Some(((nums[0], nums[1]), (nums[2], nums[3])));
                }
                i += 2;
            }
            _ => i += 1,
        }
    }
    opts
}
