//! Plan 488 步骤 9：OLE 拖放双向 E2E（T3 拖入 / T4 拖出）。
//!
//! 拖拽模拟沿 486 待澄清③裁定：SendInput 真手势主路径（drag_sim 原语，
//! 本期为 OLE 场景新增 ole_* 三件套 + ole_drag 组合）。
//!
//! - **T3 拖入**：fixture `--offer text:` 起拖（真 DoDragDrop 子进程）→
//!   合成拖到本测试进程的自有目标窗（直接挂我方 `DesktopDropTarget`）→
//!   断言 `take_native_drop()` 收到 text 与屏幕坐标。
//! - **T4 拖出**：我方 `start_drag`（真 DndDataObject + STA DoDragDrop）→
//!   合成按下/移动/释放在 fixture 窗（步骤 8 的 IDropTarget）→ 断言
//!   fixture stdout `{"evt":"drop",…}` 行含载荷。
//!
//! 门控：`native-dnd` + `test-native-dock`（drag_sim 与 fixture 驱动基建）
//! × windows；真实光标交互——开发机/专用 CI 档，不在日常 `cargo t` 内。

#![cfg(all(windows, feature = "native-dnd", feature = "test-native-dock"))]

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Receiver;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use auto_lang::ui::native_dnd::win32::{self as dndw};
use auto_lang::ui::native_dock::win32::drag_sim;
use auto_lang::ui::native_dock::{win32 as ndw, NativeHwnd};

/// fixture exe 路径（缺失时自动构建；与 native_dock_e2e 同款，独立
/// target 防 flock 死锁）。
fn fixture_bin() -> std::path::PathBuf {
    static BIN: OnceLock<std::path::PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.join("../..");
        let manifest = workspace_root.join("tools/native-fixture/Cargo.toml");
        let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
        let bin = workspace_root
            .join("tools/native-fixture/target")
            .join(profile)
            .join("native-fixture.exe");
        if !bin.exists() {
            let status = Command::new("cargo")
                .args(["build", "-q", "--manifest-path"])
                .arg(&manifest)
                .status()
                .expect("cargo build native-fixture");
            assert!(status.success(), "native-fixture 构建失败");
        }
        bin
    })
    .clone()
}

/// 拖拽会话串行锁：合成拖拽操纵全局光标/前台——两个拖拽测试绝不可并发
/// （nextest 进程级隔离已保证；此锁在单进程 libtest 档兜底）。
static DRAG_LOCK: Mutex<()> = Mutex::new(());

struct Fixture {
    child: Child,
    hwnd: NativeHwnd,
    #[allow(dead_code)]
    pid: u32,
    lines: Receiver<String>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn spawn_fixture(extra_args: &[&str]) -> Fixture {
    let mut child = Command::new(fixture_bin())
        .args(extra_args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn native-fixture");
    let stdout = child.stdout.take().expect("fixture stdout");
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    std::thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    // start 行（hwnd/pid）——bounds 行可能先到（WM_WINDOWPOSCHANGED 先于
    // 启动行 emit，实测），循环取直到 start。
    let deadline = Instant::now() + Duration::from_secs(10);
    let start = loop {
        let line = rx.recv_timeout(deadline.saturating_duration_since(Instant::now()))
            .expect("fixture start 行");
        if line.contains("\"evt\":\"start\"") {
            break line;
        }
    };
    let hwnd_val = start
        .split('"')
        .find_map(|s| s.strip_prefix("0x"))
        .and_then(|s| isize::from_str_radix(s, 16).ok())
        .expect("start 行 hwnd");
    let pid = start
        .split("\"pid\":")
        .nth(1)
        .and_then(|s| s.split(',').next())
        .and_then(|s| s.parse().ok())
        .expect("start 行 pid");
    let mut f = Fixture {
        child,
        hwnd: NativeHwnd(hwnd_val),
        pid,
        lines: rx,
    };
    // 等首帧 bounds（窗口可见且几何稳定）。
    let _ = f.lines.recv_timeout(Duration::from_secs(5));
    f
}

/// 本测试进程的自有目标窗：直接挂我方 `DesktopDropTarget`（T3 拖入落点；
/// 本窗口无 winit 目标在位，无需 Revoke）。
struct TargetWindow {
    hwnd: windows::Win32::Foundation::HWND,
}

impl Drop for TargetWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::System::Ole::RevokeDragDrop(self.hwnd);
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(self.hwnd);
        }
    }
}

fn create_target_window(x: i32, y: i32, w: i32, h: i32) -> TargetWindow {
    use windows::core::w;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Ole::OleInitialize;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassW, SW_SHOW, ShowWindow, WINDOW_EX_STYLE,
        WINDOW_STYLE, WNDCLASSW, WS_OVERLAPPEDWINDOW,
    };

    unsafe extern "system" fn wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    unsafe {
        let _ = OleInitialize(None);
        let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: HINSTANCE(hmodule.0),
            lpszClassName: w!("auto_lang_dnd_e2e_target"),
            ..Default::default()
        };
        assert_ne!(RegisterClassW(&wc), 0, "RegisterClassW");
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("auto_lang_dnd_e2e_target"),
            w!("dnd-e2e-target"),
            WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0),
            x,
            y,
            w,
            h,
            None,
            None,
            Some(&HINSTANCE(hmodule.0)),
            None,
        )
        .expect("CreateWindowExW target");
        let _ = ShowWindow(hwnd, SW_SHOW);
        // STA 代理注册（与宿主同路径——T6 一轮修正后的真实机制；winit
        // 目标不在位，无需 Revoke 前置）。
        assert!(dndw::register_drop_target(hwnd), "RegisterDragDrop target");
        std::thread::sleep(Duration::from_millis(150));
        TargetWindow { hwnd }
    }
}

/// T4 源窗的拖出载荷与完成旗标（wndproc ↔ 测试主线程）。
static T4_PAYLOAD: Mutex<Option<auto_lang::ui::native_dnd::DndPayload>> = Mutex::new(None);
static T4_DRAG_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// T4 源窗：客户区 WM_LBUTTONDOWN → 内联 `start_drag`（调用线程 = 输入
/// 线程 = 主测试线程——OLE 拖拽循环必须在收到按下的线程上跑，见
/// start_drag 文档；DoDragDrop 自带消息泵接管后续输入）。
struct SourceWindow {
    hwnd: windows::Win32::Foundation::HWND,
}

impl Drop for SourceWindow {
    fn drop(&mut self) {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(self.hwnd);
        }
    }
}

fn create_source_window(x: i32, y: i32, w: i32, h: i32) -> SourceWindow {
    use windows::core::w;
    use windows::Win32::Foundation::{HINSTANCE, HWND, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::System::Ole::OleInitialize;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, RegisterClassW, SW_SHOW, ShowWindow, WINDOW_EX_STYLE,
        WINDOW_STYLE, WNDCLASSW, WM_LBUTTONDOWN, WS_OVERLAPPEDWINDOW,
    };

    unsafe extern "system" fn src_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> LRESULT {
        if msg == WM_LBUTTONDOWN {
            if let Some(payload) = T4_PAYLOAD.lock().unwrap().take() {
                let _ = dndw::start_drag(payload);
                T4_DRAG_DONE.store(true, std::sync::atomic::Ordering::SeqCst);
            }
            return LRESULT(0);
        }
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    unsafe {
        let _ = OleInitialize(None);
        let hmodule = GetModuleHandleW(None).expect("GetModuleHandleW");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(src_wndproc),
            hInstance: HINSTANCE(hmodule.0),
            lpszClassName: w!("auto_lang_dnd_e2e_source"),
            ..Default::default()
        };
        assert_ne!(RegisterClassW(&wc), 0, "RegisterClassW source");
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            w!("auto_lang_dnd_e2e_source"),
            w!("dnd-e2e-source"),
            WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0),
            x,
            y,
            w,
            h,
            None,
            None,
            Some(&HINSTANCE(hmodule.0)),
            None,
        )
        .expect("CreateWindowExW source");
        let _ = ShowWindow(hwnd, SW_SHOW);
        std::thread::sleep(Duration::from_millis(150));
        SourceWindow { hwnd }
    }
}

/// 等待谓词为真（屏幕交互异步结算）。
fn wait_until(timeout: Duration, mut f: impl FnMut() -> bool) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        if f() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// 泵消息版等待：本线程创建的 STA 目标窗（DesktopDropTarget）会收到跨
/// 进程 DragEnter/Over/Drop COM 调用——它们以窗口消息送达本线程，等待期
/// 必须 Dispatch（sleep 等待 = 死锁：拖拽源侧 DoDragDrop 阻塞在 Drop 调用
/// 上，实测 keys 恒 0x1 现象即此）。
fn pump_wait_until(timeout: Duration, mut f: impl FnMut() -> bool) -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW, PM_REMOVE, QS_ALLINPUT, MSG,
    };
    let start = Instant::now();
    unsafe {
        let mut msg = MSG::default();
        loop {
            if f() {
                return true;
            }
            if start.elapsed() >= timeout {
                return false;
            }
            // 无消息 → 20ms 定时醒；有消息（含 STA COM 调用）→ 取发循环。
            let _ = MsgWaitForMultipleObjects(None, false, 20, QS_ALLINPUT);
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&msg);
                DispatchMessageW(&msg);
                if f() {
                    return true;
                }
            }
        }
    }
}

/// T3 拖入：fixture（--offer text）→ 本进程目标窗（DesktopDropTarget）。
#[test]
fn t3_drag_in_fixture_to_desktop_drop_target() {
    let _guard = DRAG_LOCK.lock().unwrap();
    let fixture = spawn_fixture(&["--offer", "text:e2e-drag-in-payload"]);
    let target = create_target_window(720, 240, 320, 240);

    // 源 = fixture 客户区中下（避开标题条按钮）；落点 = 目标窗中心。
    let src_rect = ndw::get_bounds_window(fixture.hwnd).expect("fixture bounds");
    let from = (src_rect.x + src_rect.w / 2, src_rect.y + src_rect.h * 2 / 3);
    let to = (720 + 160, 240 + 140);

    assert!(drag_sim::raise_top(fixture.hwnd), "raise fixture");
    drag_sim::force_foreground(fixture.hwnd);
    std::thread::sleep(Duration::from_millis(150));
    // 目标窗同样置顶：OLE 落点按 WindowFromPoint 取该屏点顶层窗——目标
    // 被他窗遮挡时 Drop 送错窗口。
    assert!(
        drag_sim::raise_top(NativeHwnd(target.hwnd.0 as isize)),
        "raise target"
    );
    std::thread::sleep(Duration::from_millis(50));    assert!(drag_sim::ole_drag(from, to, 12), "ole_drag 序列");
    let mut got = None;
    let ok = pump_wait_until(Duration::from_secs(5), || {
        got = dndw::take_native_drop();
        got.is_some()
    });
    assert!(ok, "5s 内应收到 NativeDrop");
    let payload = got.expect("payload");
    assert_eq!(payload.text.as_deref(), Some("e2e-drag-in-payload"));
    assert!(payload.formats.iter().any(|f| f == "CF_UNICODETEXT"));
    assert!(
        payload.screen_x >= 720 && payload.screen_x <= 720 + 320,
        "screen_x={}",
        payload.screen_x
    );
    assert!(
        payload.screen_y >= 240 && payload.screen_y <= 240 + 240,
        "screen_y={}",
        payload.screen_y
    );
    drop(target);
    drop(fixture);
}

/// T4 拖出：自有源窗（主线程内联 start_drag——OLE 拖拽循环在输入线程，
/// 见 start_drag 文档）→ 合成按下/移动/释放落在 fixture 窗（步骤 8 的
/// IDropTarget）→ 断言 fixture stdout `{"evt":"drop",…}` 行含载荷。
#[test]
fn t4_drag_out_desktop_to_fixture() {
    let _guard = DRAG_LOCK.lock().unwrap();
    let fixture = spawn_fixture(&[]); // 无 --offer：纯放置目标。

    // 源窗（左）与 fixture（右）各占一角，互不遮挡。
    let source = create_source_window(200, 600, 320, 200);
    assert!(drag_sim::raise_top(NativeHwnd(source.hwnd.0 as isize)), "raise source");
    assert!(drag_sim::raise_top(fixture.hwnd), "raise fixture");
    std::thread::sleep(Duration::from_millis(120));

    *T4_PAYLOAD.lock().unwrap() = Some(auto_lang::ui::native_dnd::DndPayload {
        text: Some("e2e-drag-out-payload".into()),
        ..Default::default()
    });
    T4_DRAG_DONE.store(false, std::sync::atomic::Ordering::SeqCst);

    let src_rect = ndw::get_bounds_window(NativeHwnd(source.hwnd.0 as isize)).expect("src bounds");
    let press_at = (src_rect.x + src_rect.w / 2, src_rect.y + src_rect.h / 2);
    let f_rect = ndw::get_bounds_window(fixture.hwnd).expect("fixture bounds");
    let drop_at = (f_rect.x + f_rect.w / 2, f_rect.y + f_rect.h * 2 / 3);

    // 驱动线程：按下送达源窗 wndproc → 内联 DoDragDrop 进入循环（自带
    // 泵）后，驱动分步移动到 fixture 客户区并释放。
    let driver = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(450));
        for i in 1..=14 {
            let t = i as f32 / 14.0;
            let x = press_at.0 as f32 + (drop_at.0 as f32 - press_at.0 as f32) * t;
            let y = press_at.1 as f32 + (drop_at.1 as f32 - press_at.1 as f32) * t;
            if !drag_sim::ole_move_to(x as i32, y as i32) {
                return;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        std::thread::sleep(Duration::from_millis(60));
        let _ = drag_sim::ole_release();
    });

    assert!(drag_sim::ole_move_to(press_at.0, press_at.1), "move to source");
    std::thread::sleep(Duration::from_millis(80));
    assert!(drag_sim::ole_press(), "press on source");
    // 泵等待：WM_LBUTTONDOWN 送达本线程 wndproc → 内联 DoDragDrop（其
    // 内部泵接管输入）→ 返回后旗标置位。
    let done = pump_wait_until(Duration::from_secs(10), || {
        T4_DRAG_DONE.load(std::sync::atomic::Ordering::SeqCst)
    });
    assert!(done, "10s 内源窗拖出会话应结束");
    driver.join().expect("driver join");

    // 断言 fixture drop 行（JSON lines 文本包含断言——载荷为安全 ASCII）。
    let mut drop_line: Option<String> = None;
    let ok = wait_until(Duration::from_secs(5), || {
        while let Ok(line) = fixture.lines.try_recv() {
            if line.contains("\"evt\":\"drop\"") {
                drop_line = Some(line);
            }
        }
        drop_line.is_some()
    });
    assert!(ok, "5s 内应见 fixture drop 行");
    let line = drop_line.expect("drop line");
    assert!(line.contains("CF_UNICODETEXT"), "formats 含文本格式: {line}");
    assert!(line.contains("e2e-drag-out-payload"), "drop 文本载荷: {line}");

    // 完成效果读掉防跨测试残留。
    let _ = dndw::take_finished_effect();
    drop(source);
    drop(fixture);
}
