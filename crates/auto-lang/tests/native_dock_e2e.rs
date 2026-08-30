//! Plan 473 T8：native dock fixture E2E（真第三方进程路径）。
//!
//! 拉起 `tools/native-fixture` 独立进程，经 `native_dock::win32` 公开层驱动
//! dock 全流程原语，覆盖 B1（pid 发现 + 几何落槽）/ B2（undock 恢复 pre-dock
//! bounds）/ B3（relayout 跟随）/ C3（min-size 探测）/ C4（倔强窗口复位 →
//! 拖走判定）/ C5（最大化先 restore）/ B7（self-close → WinEventHook DESTROY）。
//! WmState 注册表/状态机组合已在 session 单测覆盖（T4/T5）；renderer 装配层
//! 的视觉与交互归 T4 手动冒烟清单。
//!
//! 门控：feature `test-native-dock` + `#[cfg(windows)]`；fixture 缺失时
//! 首测自动 `cargo build`（独立项目，非 workspace 成员）。

#![cfg(all(windows, feature = "test-native-dock"))]

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::Receiver;
use std::sync::OnceLock;
use std::time::{Duration, Instant};

use auto_lang::ui::native_dock::{self, win32 as ndw, NativeHwnd, NativeSlotEventKind, Rect, Size};

/// fixture exe 路径（缺失时自动构建；profile 随当前构建档）。
fn fixture_bin() -> std::path::PathBuf {
    static BIN: OnceLock<std::path::PathBuf> = OnceLock::new();
    BIN.get_or_init(|| {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace_root = manifest_dir.join("../..");
        let manifest = workspace_root.join("tools/native-fixture/Cargo.toml");
        let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
        // fixture 是独立 workspace，产物落它自己的 target（与外层 cargo test
        // 的 target 锁互不相干——测试内对共享 target 构建 = flock 死锁，实测）。
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

#[allow(dead_code)]
struct Fixture {
    child: Child,
    hwnd: NativeHwnd,
    pid: u32,
    lines: Receiver<String>, // 诊断通道（--trace 时读回；常规测试不消费）
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// 拉起 fixture 并解析 start 行（hwnd/pid）；stdout 行进 channel 供断言。
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
    // bounds 行可能先于 start 行（CreateWindowExW 期间即触发
    // WM_WINDOWPOSCHANGED）——读到 start 行为止。
    let deadline = Instant::now() + Duration::from_secs(10);
    let start = loop {
        let line = rx
            .recv_timeout(Duration::from_secs(1))
            .ok()
            .filter(|l| l.contains("\"evt\":\"start\""))
            .or(None);
        if let Some(start) = line {
            break start;
        }
        assert!(Instant::now() < deadline, "fixture start 行 10s 未到");
    };
    let hwnd = extract_hex(&start, "\"hwnd\":\"0x").expect("start 行缺 hwnd");
    let pid = extract_num(&start, "\"pid\":").expect("start 行缺 pid");
    Fixture {
        child,
        hwnd: NativeHwnd(hwnd),
        pid,
        lines: rx,
    }
}

fn extract_hex(line: &str, key: &str) -> Option<isize> {
    let i = line.find(key)? + key.len();
    let end = line[i..].find('"')? + i;
    isize::from_str_radix(&line[i..end], 16).ok()
}

fn extract_num(line: &str, key: &str) -> Option<u32> {
    let i = line.find(key)? + key.len();
    let tail = &line[i..];
    let end = tail
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(tail.len());
    tail[..end].parse().ok()
}

/// 测试进程声明 per-monitor v2（对齐 fixture 与生产宿主；否则跨进程
/// SetWindowPos 坐标被系统虚拟化缩放——200% 屏实测 ×2 偏移）。
fn ensure_dpi_aware() {
    static DONE: OnceLock<bool> = OnceLock::new();
    let _ = DONE.get_or_init(|| ndw::set_process_dpi_aware_per_monitor_v2());
}

fn bounds_close(got: Rect, want: Rect, tol: i32) -> bool {
    (got.x - want.x).abs() <= tol
        && (got.y - want.y).abs() <= tol
        && (got.w - want.w).abs() <= tol
        && (got.h - want.h).abs() <= tol
}

/// 轮询等窗口几何到达目标值（异步窗口移动落地）。
fn wait_bounds_eq(hwnd: NativeHwnd, want: Rect, tol: i32, secs: u64) -> bool {
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        let got = ndw::get_bounds(hwnd);
        if got.map_or(false, |g| bounds_close(g, want, tol)) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

// ---- B1/B2：pid 发现 + dock 几何落槽 + undock 恢复 pre-dock bounds ----

#[test]
fn fixture_dock_then_undock_restores_bounds() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-dock-undock"]);
    // B1：按 pid 发现目标。
    let found = ndw::find_top_level_by_pid(fixture.pid);
    assert!(found.iter().any(|h| h.0 == fixture.hwnd.0), "pid 发现应命中 fixture");
    let pre = ndw::get_bounds_window(fixture.hwnd).expect("pre bounds");
    let saved_style = ndw::strip_chrome(fixture.hwnd).expect("strip_chrome");
    // 落槽（dock 几何写入）。
    let slot = Rect::new(500, 300, 620, 460);
    ndw::set_bounds(fixture.hwnd, slot).expect("dock set_bounds");
    assert!(
        wait_bounds_eq(fixture.hwnd, slot, 2, 3),
        "B1: dock 后几何应等于槽位（got {:?}）",
        ndw::get_bounds(fixture.hwnd)
    );
    // B2：undock = 恢复 pre-dock bounds/样式。
    let _ = ndw::set_bounds(fixture.hwnd, pre);
    if saved_style != 0 {
        let _ = ndw::restore_chrome(fixture.hwnd, saved_style);
    }
    let _ = ndw::restore_corner_preference(fixture.hwnd);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let got = ndw::get_bounds_window(fixture.hwnd);
        if got.map_or(false, |g| bounds_close(g, pre, 2)) {
            break;
        }
        assert!(Instant::now() < deadline, "B2: undock 后应恢复 pre-dock bounds（got {got:?} want {pre:?}）");
        std::thread::sleep(Duration::from_millis(50));
    }
}

// ---- B3：relayout（布局变化）→ 几何跟随 ----

#[test]
fn fixture_geometry_follows_relayout() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-relayout"]);
    ndw::strip_chrome(fixture.hwnd).expect("strip");
    let first = Rect::new(100, 100, 400, 300);
    ndw::set_bounds(fixture.hwnd, first).expect("初摆");
    assert!(wait_bounds_eq(fixture.hwnd, first, 2, 3), "初摆应生效");
    // 布局变化（分隔条拖动/relayout 的几何同步同一路径）。
    let second = Rect::new(700, 240, 520, 380);
    ndw::set_bounds(fixture.hwnd, second).expect("relayout");
    assert!(
        wait_bounds_eq(fixture.hwnd, second, 2, 3),
        "B3: relayout 后几何应跟随（got {:?}）",
        ndw::get_bounds(fixture.hwnd)
    );
}

// ---- C3：min-size 大于请求尺寸 → 探测缓存估计（不撕裂） ----

#[test]
fn fixture_min_size_probe_c3() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-minsize", "--min-size", "300x200"]);
    ndw::strip_chrome(fixture.hwnd).expect("strip");
    // 请求 40x30 → fixture 的 WM_GETMINMAXINFO 强制 ≥300x200。
    let tiny = ndw::probe_bounds(fixture.hwnd, Rect::new(60, 60, 40, 30)).expect("probe tiny");
    assert!(
        tiny.w >= 298 && tiny.h >= 198,
        "C3: 读回应 ≥ min-size（got {tiny:?}）"
    );
    let est = native_dock::observe_min_size_estimate(Size::new(40, 30), tiny.size());
    assert_eq!(est, Some(tiny.size()), "C3: min-size 估计应缓存实际值");
    // 正常尺寸请求 → 无截断 → 不产生新估计。
    let ok = ndw::probe_bounds(fixture.hwnd, Rect::new(60, 60, 500, 400)).expect("probe ok");
    assert_eq!(
        native_dock::observe_min_size_estimate(Size::new(500, 400), ok.size()),
        None
    );
}

// ---- C4：倔强窗口自我复位 → 偏差超阈判用户拖走（undock 语义） ----

#[test]
fn fixture_stubborn_reset_detected_as_drag_c4() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-stubborn", "--stubborn"]);
    ndw::strip_chrome(fixture.hwnd).expect("strip");
    // 槽位摆在远离初始位置（200,200）处：dock 后 stubborn 定时器拉回初始
    // 位置 → 读回 vs 槽位偏差远超阈值 → 判定拖走（宿主随走 undock）。
    let slot = Rect::new(900, 500, 420, 320);
    ndw::set_bounds(fixture.hwnd, slot).expect("dock set_bounds");
    // 等 stubborn 复位（1s 周期 + 余量）。
    std::thread::sleep(Duration::from_millis(1600));
    let cur = ndw::get_bounds(fixture.hwnd).expect("post-reset bounds");
    assert!(
        native_dock::detect_user_drag(cur, slot, native_dock::USER_DRAG_THRESHOLD_PX),
        "C4: 复位后偏差应判为用户拖走（cur {cur:?} slot {slot:?}）"
    );
}

// ---- C5：已最大化窗口 dock 前先 restore ----

#[test]
fn fixture_maximized_restores_before_dock_c5() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-maximize"]);
    ndw::show_window(fixture.hwnd, ndw::ShowMode::Maximize).expect("maximize");
    assert!(ndw::is_maximized(fixture.hwnd), "C5 前置：应已最大化");
    // dock 流程首步（T6 执行体同型）：is_maximized → SW_RESTORE。
    if ndw::is_maximized(fixture.hwnd) {
        ndw::show_window(fixture.hwnd, ndw::ShowMode::Restore).expect("restore");
    }
    assert!(!ndw::is_maximized(fixture.hwnd), "C5：restore 后不应再最大化");
}

// ---- B7：目标自毁 → WinEventHook DESTROY（槽位回收事件源） ----

#[test]
fn fixture_self_close_emits_destroy_b7() {
    ensure_dpi_aware();
    let mut fixture = spawn_fixture(&["--title", "e2e-selfclose", "--self-close", "1"]);
    // 事件钩子（本测试二进制独占钩子槽位；与并行测试竞争时退避重试）。
    let (_hook, rx) = match ndw::spawn_event_hook(false) {
        Ok(pair) => pair,
        Err(_) => {
            std::thread::sleep(Duration::from_millis(500));
            ndw::spawn_event_hook(false).expect("spawn hook 重试")
        }
    };
    // fixture 1s 自毁 → DESTROY 事件到达（宿主随后走 TargetClosed 回收）。
    let deadline = Instant::now() + Duration::from_secs(8);
    let mut got_destroy = false;
    while Instant::now() < deadline {
        match rx.recv_timeout(Duration::from_millis(300)) {
            Ok(evt)
                if evt.hwnd == fixture.hwnd && evt.kind == NativeSlotEventKind::Destroy =>
            {
                got_destroy = true;
                break;
            }
            Ok(_) => continue,
            Err(_) => continue,
        }
    }
    assert!(got_destroy, "B7: 8s 内未收到 fixture 的 DESTROY 事件");
    // hwnd 可能被系统复用致 IsWindow 失真，以子进程退出为准（2s 轮询）。
    let mut exited = false;
    for _ in 0..40 {
        if fixture.child.try_wait().ok().flatten().is_some() {
            exited = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    assert!(exited, "B7: fixture 子进程应已退出");
}

// ---- Plan 486 T4：拖入手势（SendInput 真拖 → 事件序列 → DragWatch →
//      DockCandidate + 高亮 + dock 落槽）；拖出（位移 → C4 判定 → undock 恢复） ----

/// 事件钩子（钩子槽位互斥；竞争时退避重试——b7 同型）。
fn hook_with_retry() -> (ndw::NativeSlotEventHook, Receiver<native_dock::NativeSlotEvent>) {
    for _ in 0..20 {
        match ndw::spawn_event_hook(false) {
            Ok(pair) => return pair,
            Err(_) => std::thread::sleep(Duration::from_millis(250)),
        }
    }
    panic!("钩子槽位 5s 未释放（并行 native-dock 测试死锁？）");
}

/// T4 拖入：SendInput 真实拖 fixture 标题栏到"桌面矩形"内 → 断言真实
/// MOVESIZESTART/LOCATIONCHANGE/MOVESIZEEND 事件序列 + DragWatch 全链终态
/// DockCandidate + 拖动中候选槽位高亮产出；随后走 dock 执行臂同型原语
/// 落槽（几何断言）。
#[test]
fn fixture_drag_in_produces_dock_candidate_t4() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-drag-in"]);
    // 保留 caption（生产手势对未 docked 窗发生）；提到 z 序顶部后找一个
    // 标题栏不被遮挡的摆放点（前台/提权窗可能覆盖——非提权进程无法置顶）。
    ndw::drag_sim::raise_top(fixture.hwnd);
    let mut caption_visible = false;
    for cand in [(60, 60), (500, 100), (1200, 200), (300, 700), (1800, 600)] {
        let probe = Rect::new(cand.0, cand.1, 1400, 900);
        let _ = ndw::set_bounds(fixture.hwnd, probe);
        ndw::drag_sim::raise_top(fixture.hwnd);
        std::thread::sleep(Duration::from_millis(80));
        let _ = ndw::drag_sim::raise_top(fixture.hwnd);
        let caption = (probe.x + probe.w / 3, probe.y + 16);
        if ndw::drag_sim::window_from_point(caption.0, caption.1) == fixture.hwnd.0 {
            caption_visible = true;
            break;
        }
    }
    std::thread::sleep(Duration::from_millis(100));
    let (_hook, rx) = hook_with_retry();
    // 注入式手势上下文：桌面矩形（屏中央区域）+ 唯一候选槽位（free-cell）。
    let desktop = Rect::new(400, 300, 2000, 1400);
    let cell = Rect::new(800, 600, 1000, 800);
    let target = (cell.x + cell.w / 2, cell.y + cell.h / 2);
    // 待澄清①执行期定案（两级退路）：
    // ① 主路径 SendInput caption 真拖（raise+前台化+激活结算后拖）；
    // ② 短窗核对 MOVESIZESTART 未到（本机实测：ToDesk 输入钩子类环境对
    //    合成 caption 拖拽不生效——光标/命中/前台全部正确但 move-size
    //    循环不起）→ SC_MOVE|HTCAPTION 注入退路——同样进入真实 move-size
    //    模态循环，MOVESIZESTART/LOCATIONCHANGE/MOVESIZEEND 序列同源，
    //    仅发起手段不同（PostMessage vs 输入管线）。
    let mut watch = native_dock::DragWatch::new();
    let mut saw_start = false;
    // 预排空：清掉探针摆位的噪声事件。
    while rx.try_recv().is_ok() {}
    if caption_visible {
        let _ = ndw::drag_sim::caption_drag_to(fixture.hwnd, target, 20);
        let deadline = Instant::now() + Duration::from_millis(1200);
        while Instant::now() < deadline {
            match rx.recv_timeout(Duration::from_millis(150)) {
                Ok(evt)
                    if evt.hwnd == fixture.hwnd
                        && evt.kind == NativeSlotEventKind::MoveSizeStart =>
                {
                    saw_start = true;
                    watch.start(evt.hwnd);
                    break;
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }
    }
    if !saw_start {
        assert!(
            ndw::drag_sim::syscommand_drag_to(fixture.hwnd, target, 20),
            "注入退路拖拽链失败"
        );
    }
    // 消费真实事件流 → 驱动 DragWatch（与 renderer drive_drag_watch 同型）。
    let mut saw_location = false;
    let mut saw_highlight = false;
    let mut outcome = None;
    let mut evlog: Vec<String> = Vec::new();
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let evt = match rx.recv_timeout(Duration::from_millis(200)) {
            Ok(evt) => evt,
            Err(_) => {
                if outcome.is_some() || saw_start && !watch.is_watching() {
                    break;
                }
                continue;
            }
        };
        evlog.push(format!("{:x}/{}", evt.hwnd.0, match evt.kind {
            NativeSlotEventKind::MoveSizeStart => "start",
            NativeSlotEventKind::MoveSizeEnd => "end",
            NativeSlotEventKind::LocationChange => "loc",
            NativeSlotEventKind::MinimizeStart => "min-s",
            NativeSlotEventKind::MinimizeEnd => "min-e",
            NativeSlotEventKind::Destroy => "destroy",
        }));
        if evt.hwnd != fixture.hwnd {
            continue;
        }
        let pointer = ndw::cursor_pos().unwrap_or((target.0, target.1));
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        match evt.kind {
            NativeSlotEventKind::MoveSizeStart => {
                saw_start = true;
                watch.start(evt.hwnd);
            }
            NativeSlotEventKind::LocationChange
                if watch.watched_hwnd() == Some(evt.hwnd) =>
            {
                saw_location = true;
                if let native_dock::DragSample::Overlay(Some(_)) =
                    watch.sample(pointer, desktop, &[cell], now_ms)
                {
                    saw_highlight = true;
                }
            }
            NativeSlotEventKind::MoveSizeEnd
                if watch.watched_hwnd() == Some(evt.hwnd) =>
            {
                outcome = watch.end(pointer, desktop);
            }
            _ => {}
        }
        if outcome.is_some() {
            break;
        }
    }
    assert!(
        saw_start,
        "T4: 未收到 MOVESIZESTART（两条路径都未进入 move-size 循环）；事件日志: {evlog:?}"
    );
    assert!(saw_location, "T4: 未收到 LOCATIONCHANGE");
    assert!(saw_highlight, "T4: 拖动中应产出候选槽位高亮");
    assert_eq!(
        outcome,
        Some(native_dock::DragWatchOutcome::DockCandidate(fixture.hwnd)),
        "T4: 桌面内松手应产出 DockCandidate"
    );
    // dock 执行臂同型（execute_dock_native 原语序列）：strip → 落槽 → 断言。
    let pre = ndw::get_bounds_window(fixture.hwnd).expect("pre bounds");
    let saved_style = ndw::strip_chrome(fixture.hwnd).expect("strip_chrome");
    ndw::set_bounds(fixture.hwnd, cell).expect("dock set_bounds");
    assert!(
        wait_bounds_eq(fixture.hwnd, cell, 2, 3),
        "T4: 收编后几何应落槽位（got {:?}）",
        ndw::get_bounds(fixture.hwnd)
    );
    // 清场：恢复（不留剥离样式的窗口）。
    let _ = ndw::set_bounds(fixture.hwnd, pre);
    if saved_style != 0 {
        let _ = ndw::restore_chrome(fixture.hwnd, saved_style);
    }
}

/// T4 拖出：docked 槽位位移超阈（docked 无 caption——位移即拖离的几何事实，
/// C4 判定与恢复同路径）→ detect_user_drag 判拖走 → undock 原语恢复
/// pre-dock bounds + 样式。
#[test]
fn fixture_drag_out_undock_restores_bounds_t4() {
    ensure_dpi_aware();
    let fixture = spawn_fixture(&["--title", "e2e-drag-out"]);
    let pre = ndw::get_bounds_window(fixture.hwnd).expect("pre bounds");
    let saved_style = ndw::strip_chrome(fixture.hwnd).expect("strip_chrome");
    let slot = Rect::new(500, 300, 620, 460);
    ndw::set_bounds(fixture.hwnd, slot).expect("dock set_bounds");
    assert!(wait_bounds_eq(fixture.hwnd, slot, 2, 3), "先落槽");
    // 拖离：位移远超 USER_DRAG_THRESHOLD_PX（32 物理px × DPI 前的保守倍距）。
    let dragged = Rect::new(slot.x + 400, slot.y + 350, slot.w, slot.h);
    ndw::set_bounds(fixture.hwnd, dragged).expect("drag displacement");
    std::thread::sleep(Duration::from_millis(100));
    let cur = ndw::get_bounds(fixture.hwnd).expect("post-drag bounds");
    assert!(
        native_dock::detect_user_drag(cur, slot, native_dock::USER_DRAG_THRESHOLD_PX),
        "T4: 位移超阈应判用户拖走（cur {cur:?} slot {slot:?}）"
    );
    // undock：恢复 pre-dock bounds + 样式（473 执行臂序列）。
    let _ = ndw::set_bounds(fixture.hwnd, pre);
    if saved_style != 0 {
        let _ = ndw::restore_chrome(fixture.hwnd, saved_style);
    }
    let _ = ndw::restore_corner_preference(fixture.hwnd);
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        let got = ndw::get_bounds_window(fixture.hwnd);
        if got.map_or(false, |g| bounds_close(g, pre, 2)) {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "T4: undock 后应恢复 pre-dock bounds（got {got:?} want {pre:?}）"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
}
