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
//!
//! 输出（每行一个 JSON 对象，stdout）：
//!   {"evt":"start","hwnd":"0x…","pid":N,"title":"…"}
//!   {"evt":"bounds","x":N,"y":N,"w":N,"h":N}   （位置/尺寸变化后回显实际值）
//!   {"evt":"close"}
//!
//! Phase 3 预留（本期只留 TODO 注释，不实现）：`--offer {text|files}` 拖源、
//! 放置目标日志（OLE 拖放用例 A1/A2/A5/A6）。

#[cfg(windows)]
mod win {
    use std::io::Write as _;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
    use std::sync::OnceLock;

    use windows::core::PCWSTR;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW,
        GetWindowRect, GetWindowThreadProcessId, MessageBoxW, RegisterClassW, SetTimer,
        SetWindowPos, ShowWindow, TranslateMessage, MB_ICONINFORMATION, MB_OK, MINMAXINFO,
        MSG, SWP_NOACTIVATE, SWP_NOZORDER, SET_WINDOW_POS_FLAGS, SW_SHOW,
        WINDOW_EX_STYLE, WM_COMMAND, WM_DESTROY, WM_GETMINMAXINFO, WM_TIMER,
        WM_WINDOWPOSCHANGED, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    };

    /// 夹具选项（进程单窗口，静态共享给 wndproc）。
    pub struct Opts {
        pub title: String,
        pub min_size: Option<(i32, i32)>,
        pub stubborn: bool,
        pub spawn_modal: bool,
        pub self_close_secs: Option<u32>,
    }

    impl Clone for Opts {
        fn clone(&self) -> Self {
            Self {
                title: self.title.clone(),
                min_size: self.min_size,
                stubborn: self.stubborn,
                spawn_modal: self.spawn_modal,
                self_close_secs: self.self_close_secs,
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
        println!("{line}");
        let _ = std::io::stdout().flush();
    }

    fn class_name() -> &'static [u16] {
        static NAME: OnceLock<Vec<u16>> = OnceLock::new();
        NAME.get_or_init(|| "auto_lang_native_fixture\0".encode_utf16().collect())
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
                        PCWSTR(text.as_mut_ptr()),
                        PCWSTR(caption.as_mut_ptr()),
                        MB_OK | MB_ICONINFORMATION,
                    );
                    LRESULT(0)
                }
                WM_DESTROY => {
                    emit("{\"evt\":\"close\"}");
                    windows::Win32::UI::WindowsAndMessaging::PostQuitMessage(0);
                    LRESULT(0)
                }
                _ => DefWindowProcW(hwnd, msg, wparam, lparam),
            }
        }
    }

    pub fn run(opts: Opts) -> i32 {
        OPTS.set(opts.clone()).ok();
        unsafe {
            let Ok(hmodule) = GetModuleHandleW(None) else {
                return 1;
            };
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: HINSTANCE(hmodule.0),
                lpszClassName: PCWSTR(class_name().as_ptr()),
                ..Default::default()
            };
            if RegisterClassW(&wc) == 0 {
                return 1;
            }
            let mut title_w: Vec<u16> = opts.title.encode_utf16().collect();
            title_w.push(0);
            let Ok(hwnd) = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(class_name().as_ptr()),
                PCWSTR(title_w.as_ptr()),
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

            if opts.stubborn {
                SetTimer(hwnd, TIMER_STUBBORN, 1000, None);
            }
            if let Some(secs) = opts.self_close_secs {
                SetTimer(hwnd, TIMER_SELF_CLOSE, secs * 1000, None);
            }
            if opts.spawn_modal {
                create_modal_button(hwnd, hmodule);
            }

            let mut msg = MSG::default();
            while GetMessageW(&mut msg, HWND::default(), 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            0
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
            PCWSTR(class.as_ptr()),
            PCWSTR(label.as_ptr()),
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
            "--spawn-modal" => {
                opts.spawn_modal = true;
                i += 1;
            }
            "--self-close" if i + 1 < args.len() => {
                opts.self_close_secs = args[i + 1].parse().ok();
                i += 2;
            }
            _ => i += 1,
        }
    }
    opts
}
