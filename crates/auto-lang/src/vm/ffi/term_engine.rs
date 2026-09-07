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
    })
    .as_ref()
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

/// 收割引擎输出并刷新 glue 侧快照(feed + 损伤行全量重采)。
fn engine_feed_snapshot(lib: &Library, h: *mut core::ffi::c_void, handle: i64) {
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
        // Full(-1)或脏行集都全量重采(行数由 rows 文本直至 -1 决定)。
        let row_text: libloading::Symbol<
            unsafe extern "C" fn(*mut core::ffi::c_void, c_int, *mut c_char, c_int) -> c_int,
        > = lib.get(b"autoterm_engine_row_text\0").expect("autoterm_engine_row_text symbol");
        let mut lines = Vec::new();
        for r in 0..64 {
            let mut buf = [0 as c_char; 512];
            let need = row_text(h, r as c_int, buf.as_mut_ptr(), 512);
            if need < 0 {
                break;
            }
            let n = (need as usize).saturating_sub(1).min(511);
            let bytes: Vec<u8> = buf[..n].iter().map(|&c| c as u8).collect();
            lines.push(String::from_utf8_lossy(&bytes).into_owned());
        }
        snapshots().insert(handle, lines);
    }
}

/// 视口行文本快照:先收割引擎输出(内联 feed + 损伤刷新)再回读。
fn engine_rows(handle: i64) -> Vec<String> {
    let Some(lib) = lib() else {
        return Vec::new();
    };
    let h = ptr_of(handle);
    if h.is_null() {
        return Vec::new();
    }
    engine_feed_snapshot(lib, h, handle);
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

pub fn shim_term_write_line(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let line: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as i64;
    engine_write_line(handle, &line);
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
