//! auto-os Plan 013 T2: AutoTerm 引擎 VM 绑定(auto.term.*,ID 2943-2949,
//! native_catalog 三表登记——register_std_shims 自动绑 shim + canonical 名)。
//!
//! 语义对齐 auto-term `at-gen/src/engine.rs` 手写胶水(PLAN-009 T8):
//! libloading 加载 `autoterm_core.dll`,glue 侧持句柄表(i64 → 原始指针)
//! 与快照表(句柄 → 视口行文本),`engine_rows` 内联 feed + 全量重采。
//! 两处刻意差异(宿主桌面适配):
//! - DLL 解析增加「宿主 exe 同目录」首选拜(003 §5 随包分发契约;
//!   at-gen 只需组内 target 扫描);
//! - DLL 缺席时 `spawn` 返 0(契约内失败:句柄 0 = 失败),其余调用按
//!   无效句柄静默 no-op——宿主桌面不因部署缺件崩 VM(at-gen 为 dev
//!   harness,缺席即 panic)。
//!
//! 无环铁律:auto-lang 对 auto-term 零 Cargo 依赖,引擎只经运行期 DLL
//! 加载(at-gen 同款)。

use libloading::Library;
use std::ffi::{c_char, c_int};
use std::sync::{Mutex, OnceLock};

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::convert::VMConvertible;
use crate::vm::task::AutoTask;


static LIB: OnceLock<Option<Library>> = OnceLock::new();
static NEXT_HANDLE: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);
type SnapMap = std::collections::HashMap<i64, Vec<String>>;
static SNAPSHOTS: std::sync::LazyLock<Mutex<SnapMap>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));
static HANDLES: OnceLock<Mutex<std::collections::HashMap<i64, i64>>> = OnceLock::new();
// 014:光标格与视口几何(glue 侧采样;feed 时随拍刷新,spawn 时落初值)。
// PLAN-018 D5 per-handle 化:键 = 引擎句柄,多 Pane 几何/光标互不串线
// (旧进程级单例 CURSOR/VIEWPORT 退役;shim 按柄读缺省 (0,0))。
type GeomMap = std::collections::HashMap<i64, (i64, i64)>;
static CURSORS: std::sync::LazyLock<Mutex<GeomMap>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));
static VIEWPORTS: std::sync::LazyLock<Mutex<GeomMap>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashMap::new()));

fn geom_of(map: &Mutex<GeomMap>, handle: i64) -> (i64, i64) {
    *map.lock().unwrap().get(&handle).unwrap_or(&(0, 0))
}

fn set_geom(map: &Mutex<GeomMap>, handle: i64, v: (i64, i64)) {
    map.lock().unwrap().insert(handle, v);
}

fn lib() -> Option<&'static Library> {
    LIB.get_or_init(|| {
        // 解析顺序:env → 宿主 exe 同目录(003 §5)→ exe 祖先 target/
        // {debug,release}(组内布局,仿 at-gen)。
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(p) = std::env::var("AUTOTERM_ENGINE_DLL") {
            candidates.push(std::path::PathBuf::from(p));
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("autoterm_core.dll"));
                for anc in dir.ancestors().take(4) {
                    candidates.push(anc.join("target/debug/autoterm_core.dll"));
                    candidates.push(anc.join("target/release/autoterm_core.dll"));
                }
            }
        }
        let found = candidates.into_iter().find(|p| p.is_file());
        match found {
            Some(path) => match unsafe { Library::new(&path) } {
                Ok(l) => Some(l),
                Err(e) => {
                    eprintln!("[term-engine] autoterm_core.dll 加载失败({path:?}): {e}");
                    None
                }
            },
            None => {
                eprintln!(
                    "[term-engine] autoterm_core.dll 未找到(部署至宿主同目录,或设 AUTOTERM_ENGINE_DLL;auto-term 侧先 cargo build -p autoterm-core)"
                );
                None
            }
        }
    });
    LIB.get().and_then(|o| o.as_ref())
}

fn handles() -> std::sync::MutexGuard<'static, std::collections::HashMap<i64, i64>> {
    HANDLES
        .get_or_init(|| Mutex::new(std::collections::HashMap::new()))
        .lock()
        .unwrap()
}

fn snapshots() -> std::sync::MutexGuard<'static, SnapMap> {
    SNAPSHOTS.lock().unwrap()
}

fn ptr_of(handle: i64) -> *mut core::ffi::c_void {
    handles().get(&handle).copied().unwrap_or(0) as *mut core::ffi::c_void
}

/// spawn(缺省 shell = COMSPEC/cmd);返回句柄(0 = 失败/DLL 缺席)。
fn engine_spawn(cols: i64, rows: i64) -> i64 {
    let Some(lib) = lib() else {
        return 0;
    };
    unsafe {
        let spawn: libloading::Symbol<
            unsafe extern "C" fn(c_int, c_int, *const c_char) -> *mut core::ffi::c_void,
        > = lib.get(b"autoterm_engine_spawn\0").expect("autoterm_engine_spawn symbol");
        let h = spawn(cols as c_int, rows as c_int, std::ptr::null());
        if h.is_null() {
            return 0;
        }
        let handle = NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // SAFETY: 句柄由 spawn 签名约定持有,free 前有效;原始指针无法
        // round-trip 进 VM int,故 glue 侧持句柄→指针表(at-gen 同款)。
        handles().insert(handle, h as i64);
        set_geom(&VIEWPORTS, handle, (cols, rows));
        set_geom(&CURSORS, handle, (0, 0));
        handle
    }
}

/// PLAN-018 D5 SpawnSpec:engine_spawn_ex(program/argv/cwd/几何)VM 轨。
/// cwd 空 = 继承宿主;DLL 缺席/失败 = 0(契约内失败,与旧 spawn 同)。
fn engine_spawn_ex(program: &str, argv: Vec<String>, cwd: &str, cols: i64, rows: i64) -> i64 {
    let Some(lib) = lib() else {
        return 0;
    };
    let c_prog = match std::ffi::CString::new(program) {
        Ok(s) => s,
        Err(_) => return 0,
    };
    let c_args: Vec<std::ffi::CString> = argv
        .iter()
        .filter_map(|a| std::ffi::CString::new(a.as_str()).ok())
        .collect();
    let c_argv: Vec<*const c_char> = c_args.iter().map(|a| a.as_ptr()).collect();
    let c_cwd = if cwd.is_empty() {
        None
    } else {
        std::ffi::CString::new(cwd).ok()
    };
    unsafe {
        let spawn_ex: libloading::Symbol<
            unsafe extern "C" fn(
                *const c_char,
                *const *const c_char,
                c_int,
                *const c_char,
                c_int,
                c_int,
            ) -> *mut core::ffi::c_void,
        > = match lib.get(b"autoterm_engine_spawn_ex\0") {
            Ok(s) => s,
            Err(_) => return 0, // 旧 DLL(无符号)= 契约内失败
        };
        let h = spawn_ex(
            c_prog.as_ptr(),
            if c_argv.is_empty() { std::ptr::null() } else { c_argv.as_ptr() },
            c_argv.len() as c_int,
            c_cwd.as_ref().map(|c| c.as_ptr()).unwrap_or(std::ptr::null()),
            cols as c_int,
            rows as c_int,
        );
        if h.is_null() {
            return 0;
        }
        let handle = NEXT_HANDLE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        handles().insert(handle, h as i64);
        set_geom(&VIEWPORTS, handle, (cols, rows));
        set_geom(&CURSORS, handle, (0, 0));
        handle
    }
}

/// 宿主→子进程一行输入(补 \r\n)。
fn engine_write_line(handle: i64, line: &str) {
    let Some(lib) = lib() else { return };
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    let mut bytes = line.as_bytes().to_vec();
    bytes.extend_from_slice(b"\r\n");
    unsafe {
        let write: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize),
        > = lib.get(b"autoterm_engine_write_input\0").expect("autoterm_engine_write_input symbol");
        write(h, bytes.as_ptr(), bytes.len());
    }
}

/// 014 直键入泵:排空 terminal 组件键入队列(同进程注册表
/// `ui::terminal`,iced widget 键盘捕获入队),逐键**裸写**引擎(无
/// \r\n 补缀——VT 串里 Enter 已是 \r、控制码/CSI 原样)。返回泵送键数
/// (DLL 缺席/句柄无效时队列照样排空丢弃,不回灌)。
/// 旧件 = 广播排空(单端应用行为不变)。
fn engine_pump_input(handle: i64) -> i64 {
    pump_inner(handle, drain_pending_keys())
}

/// PLAN-018 D5 定向泵:只排空 `key` terminal 的队列并裸写该柄——
/// 多 Pane 键入互不串线。
fn engine_pump_input_for(handle: i64, key: &str) -> i64 {
    pump_inner(handle, drain_pending_keys_for(key))
}

fn pump_inner(handle: i64, keys: Vec<String>) -> i64 {
    let n = keys.len() as i64;
    if n == 0 {
        return 0;
    }
    let Some(lib) = lib() else { return n };
    let h = ptr_of(handle);
    if h.is_null() {
        return n;
    }
    unsafe {
        let write: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *const u8, usize),
        > = lib.get(b"autoterm_engine_write_input\0").expect("autoterm_engine_write_input symbol");
        for key in &keys {
            let bytes = key.as_bytes();
            write(h, bytes.as_ptr(), bytes.len());
        }
    }
    n
}

/// 收割引擎输出并刷新 glue 侧快照(feed + 损伤行全量重采 + 光标采样 +
/// 逐格样式旁路)。`sideband` 决定样式上屏目标:旧 `engine_rows` 广播
/// 全部注册 terminal(兼容面);PLAN-018 D5 `rows_for` 按 key 定向。
fn engine_feed_snapshot(
    lib: &Library,
    h: *mut core::ffi::c_void,
    handle: i64,
    sideband: Sideband<'_>,
) {
    unsafe {
        let feed: libloading::Symbol<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int> = lib
            .get(b"autoterm_engine_feed_ready\0")
            .expect("autoterm_engine_feed_ready symbol");
        feed(h);
        let take: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_int, c_int) -> c_int,
        > = lib.get(b"autoterm_engine_take_dirty_rows\0").expect("autoterm_engine_take_dirty_rows symbol");
        let mut rows = [0 as c_int; 64];
        take(h, rows.as_mut_ptr(), 64);
        // 光标格随拍采样(可见时刷新;隐藏保持上次值;per-handle D5)。
        let cursor: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, *mut c_int, *mut c_int) -> c_int,
        > = lib.get(b"autoterm_engine_cursor\0").expect("autoterm_engine_cursor symbol");
        let (mut r, mut c) = (0 as c_int, 0 as c_int);
        if cursor(h, &mut r, &mut c) == 1 {
            set_geom(&CURSORS, handle, (r as i64, c as i64));
        }
        // Full(-1)或脏行集都全量重采(行数由 rows 文本直至 -1 决定),
        // 逐行取文本 + 逐格样式,文本进快照、样式走组件旁路。
        let row_text: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut c_char, c_int) -> c_int,
        > = lib.get(b"autoterm_engine_row_text\0").expect("autoterm_engine_row_text symbol");
        let row_style: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut u32, c_int) -> c_int,
        > = lib.get(b"autoterm_engine_row_style\0").expect("autoterm_engine_row_style symbol");
        let mut lines = Vec::new();
        for r in 0..256i32 {
            let mut buf = [0 as c_char; 512];
            let need = row_text(h, r, buf.as_mut_ptr(), 512);
            if need < 0 {
                break;
            }
            let n = (need as usize).saturating_sub(1).min(511);
            let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
            let text = String::from_utf8_lossy(&bytes).into_owned();
            // 逐格样式:fg/bg 交错 u32(kind<<24|value),2×cols 容量;
            // 旁路上屏走 ui 适配层(无 ui 特征时丢弃,文本面不受影响)。
            let mut styles = [0u32; 1024];
            let styled = row_style(h, r, styles.as_mut_ptr(), 1024);
            feed_styled_sideband(sideband, r, &text, &styles[..styled.max(0) as usize]);
            lines.push(text);
        }
        snapshots().insert(handle, lines);
    }
}

/// PLAN-018 D5:样式旁路上屏目标(旧广播语义保留为兼容面)。
#[derive(Clone, Copy)]
enum Sideband<'k> {
    /// 广播全部注册 terminal(旧 `engine_rows`;单端应用行为不变)。
    All,
    /// 只投喂该 key 的 terminal(定向;缺 key = no-op)。
    Key(&'k str),
}

// ── ui 注册表适配层(F-02:vm/ffi 无 ui 特征编译时优雅降级为 no-op;
//    报警/泵/几何通道只在与渲染同进程时有意义)──────────────────────

/// 排空 terminal 组件键入队列;无 ui 特征恒空(泵变 no-op)。
#[cfg(feature = "ui")]
fn drain_pending_keys() -> Vec<String> {
    crate::ui::terminal::terminal_drain_all_inputs()
}
#[cfg(not(feature = "ui"))]
fn drain_pending_keys() -> Vec<String> {
    Vec::new()
}

/// 取注册表待定几何;无 ui 特征恒 None(apply_resize 变 no-op)。
#[cfg(feature = "ui")]
fn take_pending_resize() -> Option<(u16, u16)> {
    crate::ui::terminal::terminal_take_any_resize()
}
#[cfg(not(feature = "ui"))]
fn take_pending_resize() -> Option<(u16, u16)> {
    None
}

/// PLAN-018 D5 定向排空:只取 `key` terminal 的键入队列(缺 key/无 ui
/// 特征恒空——载荷不跨 key 串线)。
#[cfg(feature = "ui")]
fn drain_pending_keys_for(key: &str) -> Vec<String> {
    crate::ui::terminal::terminal_core(key)
        .map(crate::ui::terminal::terminal_drain_inputs_for)
        .unwrap_or_default()
}
#[cfg(not(feature = "ui"))]
fn drain_pending_keys_for(_key: &str) -> Vec<String> {
    Vec::new()
}

/// PLAN-018 D5 定向几何:只取 `key` terminal 的待定请求(缺 key/无 ui
/// 特征恒 None)。
#[cfg(feature = "ui")]
fn take_pending_resize_for(key: &str) -> Option<(u16, u16)> {
    crate::ui::terminal::terminal_core(key).and_then(crate::ui::terminal::terminal_take_resize_for)
}
#[cfg(not(feature = "ui"))]
fn take_pending_resize_for(_key: &str) -> Option<(u16, u16)> {
    None
}

/// 逐格样式旁路上屏;无 ui 特征丢弃(快照文本面不受影响)。
/// `sideband` = 广播(旧 rows)/按 key 定向(D5 rows_for)。
#[cfg(feature = "ui")]
fn feed_styled_sideband(sideband: Sideband<'_>, row: i32, text: &str, styles: &[u32]) {
    let pairs = styles.len() / 2;
    let mut cells: Vec<crate::ui::terminal::TermCell> = Vec::with_capacity(pairs);
    for (ci, ch) in text.chars().enumerate() {
        if ci >= pairs {
            break;
        }
        cells.push(crate::ui::terminal::TermCell {
            ch,
            fg: decode_style_color(styles[ci * 2]),
            bg: decode_style_color(styles[ci * 2 + 1]),
        });
    }
    if cells.is_empty() {
        return;
    }
    match sideband {
        Sideband::All => crate::ui::terminal::terminal_feed_cells_all(row as usize, cells),
        Sideband::Key(key) => {
            crate::ui::terminal::terminal_feed_cells_for(key, row as usize, cells)
        }
    }
}
#[cfg(not(feature = "ui"))]
fn feed_styled_sideband(_sideband: Sideband<'_>, _row: i32, _text: &str, _styles: &[u32]) {}

/// FFI 标量色 → 组件色((kind<<24)|value:0=Default 1=Indexed 2=RGB)。
#[cfg(feature = "ui")]
fn decode_style_color(v: u32) -> crate::ui::terminal::TermColor {
    let kind = v >> 24;
    let value = v & 0x00FF_FFFF;
    match kind {
        1 => crate::ui::terminal::TermColor::Indexed(value as u8),
        2 => crate::ui::terminal::TermColor::Rgb(
            (value >> 16) as u8,
            (value >> 8) as u8,
            value as u8,
        ),
        _ => crate::ui::terminal::TermColor::Default,
    }
}

/// 视口行文本快照:先收割引擎输出(内联 feed + 损伤刷新)再回读。
/// 旧件 = 广播样式旁路(单端应用行为不变)。
fn engine_rows(handle: i64) -> Vec<String> {
    engine_rows_inner(handle, Sideband::All)
}

/// PLAN-018 D5:定向变体——样式旁路只投喂 `key` 对应的 terminal。
fn engine_rows_for(handle: i64, key: &str) -> Vec<String> {
    engine_rows_inner(handle, Sideband::Key(key))
}

fn engine_rows_inner(handle: i64, sideband: Sideband<'_>) -> Vec<String> {
    let Some(lib) = lib() else {
        return Vec::new();
    };
    let h = ptr_of(handle);
    if h.is_null() {
        return Vec::new();
    }
    engine_feed_snapshot(lib, h, handle, sideband);
    snapshots().get(&handle).cloned().unwrap_or_default()
}

fn engine_resize(handle: i64, cols: i64, rows: i64) {
    let Some(lib) = lib() else { return };
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    unsafe {
        let resize: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, c_int),
        > = lib.get(b"autoterm_engine_resize\0").expect("autoterm_engine_resize symbol");
        resize(h, cols as c_int, rows as c_int);
    }
}

fn engine_interrupt(handle: i64) -> i64 {
    let Some(lib) = lib() else { return -1 };
    let h = ptr_of(handle);
    if h.is_null() {
        return -1;
    }
    unsafe {
        let interrupt: libloading::Symbol<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int> =
            lib.get(b"autoterm_engine_interrupt\0").expect("autoterm_engine_interrupt symbol");
        interrupt(h) as i64
    }
}

fn engine_is_exited(handle: i64) -> bool {
    let Some(lib) = lib() else { return true };
    let h = ptr_of(handle);
    if h.is_null() {
        return true;
    }
    unsafe {
        let exited: libloading::Symbol<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int> =
            lib.get(b"autoterm_engine_is_exited\0").expect("autoterm_engine_is_exited symbol");
        exited(h) == 1
    }
}

fn engine_free(handle: i64) {
    let Some(lib) = lib() else { return };
    let h = ptr_of(handle);
    if h.is_null() {
        return;
    }
    unsafe {
        let free: libloading::Symbol<unsafe extern "C" fn(*mut core::ffi::c_void)> =
            lib.get(b"autoterm_engine_free\0").expect("autoterm_engine_free symbol");
        free(h);
    }
    handles().remove(&handle);
    snapshots().remove(&handle);
}

// ── VM shim 薄壳(参数逆序 pop,net/tcp 族同款惯例)────────────────

pub fn shim_term_spawn(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let rows = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_spawn(cols, rows) as i32));
    Ok(())
}

/// PLAN-018 D5:engine_spawn_ex(program str, argv []str, cwd str,
/// cols int, rows int) int——SpawnSpec VM 面(cwd 空 = 继承宿主)。
pub fn shim_term_spawn_ex(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let rows = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = crate::vm::native::pop_arg_i32(task) as i64;
    let cwd: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let argv: Vec<String> = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let program: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    task.ram.push_nv(auto_val::encode_i32(
        engine_spawn_ex(&program, argv, &cwd, cols, rows) as i32,
    ));
    Ok(())
}

pub fn shim_term_write_line(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let line: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    engine_write_line(handle, &line);
    Ok(())
}

/// 014 直键入:engine_pump_input(handle) int。
pub fn shim_term_pump_input(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_pump_input(handle) as i32));
    Ok(())
}

/// PLAN-018 D5:engine_pump_for(handle int, key str) int——定向键入泵。
pub fn shim_term_pump_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_pump_input_for(handle, &key) as i32));
    Ok(())
}

/// PLAN-018 D5:engine_rows_for(handle int, key str) []str——定向快照。
pub fn shim_term_rows_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    engine_rows_for(handle, &key)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

// ── 014:几何随动 + 光标格(glue 静态量读写,泵同款管线)────────────

/// 应用注册表里的待定几何:`terminal_take_any_resize()` → 引擎 resize →
/// 该柄 VIEWPORT 刷新。返回 1=已应用 0=无请求。(旧件:任意端语义。)
fn engine_apply_resize(handle: i64) -> i64 {
    let Some((cols, rows)) = take_pending_resize() else {
        return 0;
    };
    engine_resize(handle, cols as i64, rows as i64);
    set_geom(&VIEWPORTS, handle, (cols as i64, rows as i64));
    1
}

/// PLAN-018 D5:定向变体——只消费 `key` terminal 的待定几何请求。
fn engine_apply_resize_for(handle: i64, key: &str) -> i64 {
    let Some((cols, rows)) = take_pending_resize_for(key) else {
        return 0;
    };
    engine_resize(handle, cols as i64, rows as i64);
    set_geom(&VIEWPORTS, handle, (cols as i64, rows as i64));
    1
}

pub fn shim_term_apply_resize(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_apply_resize(handle) as i32));
    Ok(())
}

/// PLAN-018 D5:engine_apply_resize_for(handle int, key str) int——定向
/// 几何泵(只消费 key terminal 的待定请求,多 Pane 请求互不串线)。
pub fn shim_term_apply_resize_for(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_apply_resize_for(handle, &key) as i32));
    Ok(())
}

pub fn shim_term_viewport_cols(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // PLAN-018 D5 per-handle 化:读该柄视口(缺省 0,0;旧件为进程级
    // 单例,多柄下串线——本面从此按柄隔离)。
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = geom_of(&VIEWPORTS, handle).0;
    task.ram.push_nv(auto_val::encode_i32(cols as i32));
    Ok(())
}

pub fn shim_term_viewport_rows(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let rows = geom_of(&VIEWPORTS, handle).1;
    task.ram.push_nv(auto_val::encode_i32(rows as i32));
    Ok(())
}

pub fn shim_term_cursor_row(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let row = geom_of(&CURSORS, handle).0;
    task.ram.push_nv(auto_val::encode_i32(row as i32));
    Ok(())
}

pub fn shim_term_cursor_col(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    let col = geom_of(&CURSORS, handle).1;
    task.ram.push_nv(auto_val::encode_i32(col as i32));
    Ok(())
}

// ── 014 积压报警回读面(VM 轨;语义对齐 at-app 侧车 term.rs,db.at 同源)──

/// 告警迟滞态(pending 越过 2MB 置位,回落 512KB 解除;上升沿即报警
/// 即取走,无累计字段)。
static BACKLOG_STATE: std::sync::Mutex<(bool, ())> = std::sync::Mutex::new((false, ()));
const BACKLOG_WARN_BYTES: i32 = 2 * 1024 * 1024;
const BACKLOG_CLEAR_BYTES: i32 = 512 * 1024;

/// 当前积压 MB(live 采样;句柄无效 = 0)。
fn engine_backlog_pending_mb(handle: i64) -> i64 {
    let Some(lib) = lib() else { return 0 };
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let pending: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib.get(b"autoterm_engine_pending_bytes\0").expect("autoterm_engine_pending_bytes symbol");
        (pending(h).max(0) as i64) / (1024 * 1024)
    }
}

/// reader 是否反压暂停中(1/0;旧 DLL 无此符号 → 0——报警面必须比被
/// 报警的路径更皮实,侧车同款)。
fn engine_backlog_paused(handle: i64) -> i64 {
    let Some(lib) = lib() else { return 0 };
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let Ok(paused) = (lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int>(
            b"autoterm_engine_backlog_paused\0",
        )) else {
            return 0;
        };
        paused(h) as i64
    }
}

/// 取走自上次调用以来的积压报警次数(0/1,上升沿即取走;内联采样
/// pending 做迟滞判定——越过 2MB 报警并 stderr 留痕,回落 512KB 解除)。
fn engine_backlog_take_alerts(handle: i64) -> i64 {
    let pending = engine_backlog_pending_bytes(handle);
    let mut st = BACKLOG_STATE.lock().unwrap();
    let (active, _) = *st;
    if !active && pending >= BACKLOG_WARN_BYTES {
        *st = (true, ());
        drop(st);
        eprintln!(
            "[term-backlog] ⚠ reader→drain 积压 {pending} 字节越过告警线 \
             {BACKLOG_WARN_BYTES}——产出侧洪峰或消费侧停摆"
        );
        1
    } else {
        if active && pending <= BACKLOG_CLEAR_BYTES {
            *st = (false, ());
        }
        0
    }
}

/// 累计环逐出块数(丢帧计数;符号缺席/句柄无效 = 0)。
fn engine_backlog_dropped(handle: i64) -> i64 {
    let Some(lib) = lib() else { return 0 };
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let Ok(dropped) = (lib.get::<unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int>(
            b"autoterm_engine_overflow_count\0",
        )) else {
            return 0;
        };
        dropped(h).max(0) as i64
    }
}

/// 积压字节原始采样(take_alerts 判定用)。
fn engine_backlog_pending_bytes(handle: i64) -> i32 {
    let Some(lib) = lib() else { return 0 };
    let h = ptr_of(handle);
    if h.is_null() {
        return 0;
    }
    unsafe {
        let pending: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void) -> c_int,
        > = lib.get(b"autoterm_engine_pending_bytes\0").expect("autoterm_engine_pending_bytes symbol");
        pending(h).max(0)
    }
}

pub fn shim_term_backlog_pending_mb(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_backlog_pending_mb(handle) as i32));
    Ok(())
}

pub fn shim_term_backlog_paused(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_backlog_paused(handle) as i32));
    Ok(())
}

pub fn shim_term_backlog_take_alerts(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_backlog_take_alerts(handle) as i32));
    Ok(())
}

pub fn shim_term_backlog_dropped(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_backlog_dropped(handle) as i32));
    Ok(())
}

pub fn shim_term_rows(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    engine_rows(handle)
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

pub fn shim_term_resize(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let rows = crate::vm::native::pop_arg_i32(task) as i64;
    let cols = crate::vm::native::pop_arg_i32(task) as i64;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    engine_resize(handle, cols, rows);
    Ok(())
}

pub fn shim_term_interrupt(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_i32(engine_interrupt(handle) as i32));
    Ok(())
}

pub fn shim_term_is_exited(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    task.ram.push_nv(auto_val::encode_bool(engine_is_exited(handle)));
    Ok(())
}

pub fn shim_term_free(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    engine_free(handle);
    Ok(())
}

// ── 014 泄漏探针:at-app「键入→泵→收割→样式旁路」完整循环的无头压测
// (隔离 iced;ash 为 shell,外部 powershell 同步采样进程内存)。
#[cfg(all(test, feature = "ui"))]
mod ash_leak_probe {
    use super::*;
    use std::time::Duration;

    #[test]
    fn ash_stream_leak_probe() {
        // DLL 定位:组内布局(auto-term 仓 target)。
        std::env::set_var(
            "AUTOTERM_ENGINE_DLL",
            r"D:\autostack\auto-term\target\debug\autoterm_core.dll",
        );
        let core = crate::ui::terminal::terminal("leak-probe", 100, 30);
        let handle = engine_spawn(100, 30);
        assert!(handle != 0, "spawn ash 失败(先 cargo build -p autoterm-core)");
        std::thread::sleep(Duration::from_millis(1000));
        engine_rows(handle);
        println!("probe: ash spawned, start streaming");
        for i in 0..20000u32 {
            // 一次键入的完整生命周期:widget 入队 → pump 裸写 → 收割。
            crate::ui::terminal::terminal_push_input(core, "x");
            engine_pump_input(handle);
            engine_rows(handle);
            if i % 1000 == 0 {
                let pending_len = crate::ui::terminal::terminal_take_damage(core).dirty_count();
                println!(
                    "iter {i}: handles={} snapshots={} pending_dirty={pending_len}",
                    handles().len(),
                    snapshots().len(),
                );
            }
        }
        println!("probe: done 20000 keystroke cycles");
        std::thread::sleep(Duration::from_millis(500));
    }
}
