//! Plan 430 C2: 三方 crate 方法 shim 包的运行期注册表与 dispatch。
//!
//! dep 管线(auto-cache methods_pack)把三方 crate 的方法面编译成独立 cdylib,
//! 本模块在 init_rust_ffi 阶段加载其 shim_manifest 并注册:
//! - `DepOpaqueObject`:cdylib 侧对象在 VM 堆中的形态(裸指针 + 析构符号 + 库保活);
//! - `METHODS` 注册表:"短类型名.方法" → marshaller 闭包,挂在 dispatch 3000 的
//!   兜底段(生成段/手写臂/native_catalog 均未命中后的最后一段);
//! - `FUNCTION_SIGS` 注册表:自由函数签名元数据(D2:known_signature 元数据优先)。
//!
//! 调用约定:marshaller 按 manifest 的 ABI 参数码(含接收者前导 'p')从 VM 栈
//! 右到左弹参,按 C ABI 调 cdylib 符号,再按返回码压栈。整型统一走 i64 槽
//! (规则 6:不做有损截断;x64/ARM64 下 32 位读低寄存器,跨位宽安全)。

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::ffi::convert::VMConvertible;
use crate::vm::heap_object::{HeapObject, TypeTag};
use crate::vm::native::ShimFunc;
use crate::vm::task::AutoTask;
use shim_metadata::emit_cdylib::{MethodEntry, ShimManifest};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::{Arc, OnceLock, RwLock};

// =============================================================================
// cdylib 对象的 VM 堆形态
// =============================================================================

/// 三方 cdylib 拥有的对象在本 VM 堆中的句柄载体。
///
/// `ptr` 指向 shim 包 cdylib 堆上的 Box<T>;本对象 Drop 时回调 cdylib 的
/// `auto__drop_<Type>` 析构符号。`lib` 同时保活 cdylib。
// SAFETY: 指针仅经 cdylib wrapper 访问;VM 侧所有访问都在 heap_objects 的
// 读写锁纪律下进行(与 RustStdlibObject 同一约束级别)。
pub struct DepOpaqueObject {
    pub crate_name: String,
    pub short_type: String,
    /// VM 堆标签(形如 "my_crate::Counter";含 '::' 使 CALL_SPEC 路由到 dispatch 3000)
    pub full_type: String,
    pub ptr: *mut c_void,
    /// 析构符号名(auto__drop_<Type>)
    pub drop_export: String,
    pub lib: Arc<libloading::Library>,
}

// SAFETY: 见结构体注释。
unsafe impl Send for DepOpaqueObject {}
unsafe impl Sync for DepOpaqueObject {}

impl Drop for DepOpaqueObject {
    fn drop(&mut self) {
        if self.ptr.is_null() {
            return;
        }
        let result = unsafe {
            self.lib
                .get::<unsafe extern "C" fn(*mut c_void)>(self.drop_export.as_bytes())
        };
        match result {
            Ok(sym) => unsafe { sym(self.ptr) },
            Err(e) => log::warn!(
                "plan430: drop symbol {} unavailable (leaked {}): {}",
                self.drop_export,
                self.full_type,
                e
            ),
        }
    }
}

impl HeapObject for DepOpaqueObject {
    fn type_tag(&self) -> TypeTag {
        TypeTag::RustStdlib(self.full_type.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// 装箱一个 cdylib 对象指针并压句柄栈。
pub fn push_dep_obj(
    task: &mut AutoTask,
    vm: &AutoVM,
    crate_name: &str,
    short_type: &str,
    ptr: *mut c_void,
    drop_export: &str,
    lib: Arc<libloading::Library>,
) -> Result<(), VMError> {
    let full_type = format!("{crate_name}::{short_type}");
    let obj = DepOpaqueObject {
        crate_name: crate_name.to_string(),
        short_type: short_type.to_string(),
        full_type,
        ptr,
        drop_export: drop_export.to_string(),
        lib,
    };
    let handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

// =============================================================================
// 注册表
// =============================================================================

type Table = HashMap<String, ShimFunc>;

fn methods_table() -> &'static RwLock<Table> {
    static T: OnceLock<RwLock<Table>> = OnceLock::new();
    T.get_or_init(|| RwLock::new(HashMap::new()))
}

fn sigs_table() -> &'static RwLock<HashMap<String, (String, String)>> {
    static T: OnceLock<RwLock<HashMap<String, (String, String)>>> = OnceLock::new();
    T.get_or_init(|| RwLock::new(HashMap::new()))
}

/// dispatch 3000 兜底段入口:命中并处理返回 true,未命中返回 false。
pub fn dispatch(
    type_name: &str,
    method: &str,
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<bool, VMError> {
    let shim = {
        let table = methods_table()
            .read()
            .expect("plan430 methods table poisoned");
        table.get(&format!("{type_name}.{method}")).cloned()
    };
    match shim {
        Some(f) => {
            f(task, vm)?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// D2:查询自由函数签名元数据((params, ret) 字符码)。
pub fn lookup_function_sig(crate_name: &str, func_name: &str) -> Option<(String, String)> {
    sigs_table()
        .read()
        .expect("plan430 sigs table poisoned")
        .get(&format!("{crate_name}::{func_name}"))
        .cloned()
}

/// D2:注册一条自由函数签名元数据(resolve_deps 阶段由 dep 管线调用)。
pub fn register_function_sig(crate_name: &str, func_name: &str, params: &str, ret: &str) {
    sigs_table()
        .write()
        .expect("plan430 sigs table poisoned")
        .insert(
            format!("{crate_name}::{func_name}"),
            (params.to_string(), ret.to_string()),
        );
}

/// 加载并注册一个方法 shim 包(manifest 来自 cdylib 的 auto__shim_manifest 导出)。
pub fn register_pack(crate_name: &str, lib: Arc<libloading::Library>, manifest_json: &str) {
    let manifest: ShimManifest = match serde_json::from_str(manifest_json) {
        Ok(m) => m,
        Err(e) => {
            log::warn!("plan430: bad shim manifest for {crate_name}: {e}");
            return;
        }
    };

    // D2:自由函数签名元数据
    {
        let mut sigs = sigs_table()
            .write()
            .expect("plan430 sigs table poisoned");
        for f in &manifest.functions {
            sigs.insert(
                format!("{crate_name}::{}", f.name),
                (f.params.clone(), f.ret.clone()),
            );
        }
    }

    let drop_of = |short: &str| format!("auto__drop_{short}");
    let mut table = methods_table()
        .write()
        .expect("plan430 methods table poisoned");
    for entry in &manifest.methods {
        let shim = make_method_shim(crate_name, lib.clone(), entry, &drop_of);
        let key = format!("{}.{}", entry.type_name, entry.method);
        if table.contains_key(&key) {
            log::warn!("plan430: method key {key} already registered (crate {crate_name}), overwriting");
        }
        table.insert(key, shim);
    }
    log::info!(
        "plan430: registered {} methods + {} function sigs for {crate_name} (fp={})",
        manifest.methods.len(),
        manifest.functions.len(),
        manifest.fingerprint
    );
}

// =============================================================================
// marshaller
// =============================================================================

/// 从 cdylib 读取 shim_manifest 导出并释放返回的 CString。
pub fn read_shim_manifest(lib: &libloading::Library) -> Result<String, VMError> {
    type ManifestFn = unsafe extern "C" fn() -> *const c_char;
    let sym: libloading::Symbol<ManifestFn> = unsafe { lib.get(b"auto__shim_manifest") }
        .map_err(|e| VMError::FFI(format!("auto__shim_manifest: {e}")))?;
    let ptr = unsafe { sym() };
    if ptr.is_null() {
        return Err(VMError::FFI("auto__shim_manifest returned null".into()));
    }
    let s = unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned();
    call_free_cstring(lib, ptr as *mut c_char);
    Ok(s)
}

fn call_free_cstring(lib: &libloading::Library, p: *mut c_char) {
    if p.is_null() {
        return;
    }
    if let Ok(sym) = unsafe { lib.get::<unsafe extern "C" fn(*mut c_char)>(b"auto__free_cstring") } {
        unsafe { sym(p) };
    }
}

/// unwrap_ok 错误通道读取:auto__last_error 返回非空 → 拷贝消息并清除,
/// 返回 Some(msg);无错误返回 None(指针归属 cdylib 线程局部,只拷贝不释放)。
fn take_last_error(lib: &libloading::Library) -> Option<String> {
    type ErrFn = unsafe extern "C" fn() -> *mut c_char;
    let sym = (unsafe { lib.get::<ErrFn>(b"auto__last_error") }).ok()?;
    let ptr = unsafe { sym() };
    if ptr.is_null() {
        return None;
    }
    let msg = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
    if let Ok(clear) = unsafe { lib.get::<unsafe extern "C" fn()>(b"auto__clear_error") } {
        unsafe { clear() };
    }
    Some(msg)
}

/// 弹出的参数中间表示(已按类归并:i/l/b → I,f → F,s → S,p → P)。
#[derive(Clone, Copy)]
enum CArg {
    I(i64),
    F(f64),
    S(*const c_char),
    P(*mut c_void),
}

fn arg_class(c: char) -> u8 {
    match c {
        'i' | 'l' | 'b' => b'I',
        'f' => b'F',
        's' => b'S',
        _ => b'P',
    }
}

fn ret_class(c: char) -> u8 {
    match c {
        'v' => b'v',
        'i' | 'l' | 'b' => b'I',
        'f' => b'F',
        's' => b'S',
        _ => b'P',
    }
}

/// 弹一个不透明对象句柄,返回其 cdylib 侧裸指针。
/// 弹一个不透明对象句柄,返回 (堆句柄, cdylib 侧裸指针)。
fn pop_dep_handle(
    task: &mut AutoTask,
    vm: &AutoVM,
    ctx: &str,
) -> Result<(u64, *mut c_void), VMError> {
    let nv = task.ram.pop_nv();
    let handle = if auto_val::is_object(nv) {
        auto_val::decode_object(nv) as u64
    } else if auto_val::is_i32(nv) {
        auto_val::decode_i32(nv) as u64
    } else {
        return Err(VMError::RuntimeError(format!(
            "{ctx}: expected object handle, got non-handle"
        )));
    };
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError(format!("{ctx}: bad handle {handle}")))?;
    let guard = obj.read().unwrap();
    let dep = guard
        .as_any()
        .downcast_ref::<DepOpaqueObject>()
        .ok_or_else(|| {
            VMError::RuntimeError(format!(
                "{ctx}: receiver/arg handle is not a dep crate object \
                 (该类型若由遗留手写臂/native_catalog 层服务,方法 shim 包不接管;\
                 待 F 阶段逐 crate 迁移)"
            ))
        })?;
    Ok((handle, dep.ptr))
}

/// 弹一个不透明对象句柄,返回其 cdylib 侧裸指针(非接收者 opaque 参数用)。
fn pop_dep_ptr(task: &mut AutoTask, vm: &AutoVM, ctx: &str) -> Result<*mut c_void, VMError> {
    pop_dep_handle(task, vm, ctx).map(|(_, ptr)| ptr)
}

macro_rules! getsym {
    ($lib:expr, $name:expr, ($($t:ty),*) -> $r:ty) => {{
        let s: libloading::Symbol<'_, unsafe extern "C" fn($($t),*) -> $r> = unsafe {
            $lib.get($name.as_bytes())
        }
        .map_err(|e| {
            VMError::FFI(format!(
                "plan430 symbol {}: {}",
                String::from_utf8_lossy($name.as_bytes()),
                e
            ))
        })?;
        s
    }};
}

/// unwrap_ok 错误检查:fallible 方法调用后查 cdylib 错误通道,命中即转 VMError。
macro_rules! check_err {
    ($lib:expr, $ctx:expr, $fallible:expr) => {
        if $fallible {
            if let Some(msg) = take_last_error(&$lib) {
                return Err(VMError::RuntimeError(format!("{}: {}", $ctx, msg)));
            }
        }
    };
}

/// 按返回类调用并压栈。参数已归并为 (类型表, 取值表)。
/// $chain:原地链式(返回 &Self)→ 压回 $recv_handle 原句柄,不新建对象。
/// $fallible:unwrap_ok——压栈前查错误通道,Err 即转 VMError。
macro_rules! ret_call {
    ($lib:expr, $name:expr, $task:expr, $vm:expr, $ctx:expr, ($($t:ty),*), ($($a:expr),*), $retc:expr, $ret_label:expr, $drop_sym:expr, $crate_nm:expr, $chain:expr, $fallible:expr, $recv_handle:expr, $ret_raw:expr) => {{
        match $retc {
            b'v' => {
                let sym = getsym!($lib, $name, ($($t),*) -> ());
                unsafe { sym($($a),*) };
                check_err!($lib, $ctx, $fallible);
                $task.ram.push_i32(0);
            }
            b'I' => {
                let sym = getsym!($lib, $name, ($($t),*) -> i64);
                let r = unsafe { sym($($a),*) };
                check_err!($lib, $ctx, $fallible);
                // bool 返回只保证 al 有效(高位是垃圾),必须掩码;
                // 其余整型低位即值(x64 写 32 位寄存器零扩展)。
                // 大整数走 heap-aware 压栈(virt_memory 48 位内联范围限制)。
                if $ret_raw == 'b' {
                    $task.ram.push_nv(auto_val::encode_bool((r & 0xFF) != 0));
                } else {
                    $vm.push_i64_vm($task, r);
                }
            }
            b'F' => {
                let sym = getsym!($lib, $name, ($($t),*) -> f64);
                let r = unsafe { sym($($a),*) };
                check_err!($lib, $ctx, $fallible);
                $task.ram.push_f64(r);
            }
            b'S' => {
                let sym = getsym!($lib, $name, ($($t),*) -> *mut c_char);
                let r = unsafe { sym($($a),*) };
                check_err!($lib, $ctx, $fallible);
                let s = if r.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(r) }.to_string_lossy().into_owned()
                };
                call_free_cstring(&$lib, r);
                let idx = $vm.add_string(s.into_bytes());
                $vm.rc_push_str_idx($task, idx);
            }
            _ => {
                let sym = getsym!($lib, $name, ($($t),*) -> *mut c_void);
                let r = unsafe { sym($($a),*) };
                check_err!($lib, $ctx, $fallible);
                if $chain {
                    let h = (*$recv_handle).unwrap_or(0);
                    if h == 0 {
                        return Err(VMError::RuntimeError(format!(
                            "{}: chain return without receiver handle",
                            $ctx
                        )));
                    }
                    $vm.rc_push($task, auto_val::encode_object(h as u32));
                } else {
                    push_dep_obj($task, $vm, &$crate_nm, &$ret_label, r, &$drop_sym, $lib.clone())?;
                }
            }
        }
    }};
}

fn make_method_shim(
    crate_name: &str,
    lib: Arc<libloading::Library>,
    entry: &MethodEntry,
    drop_of: &dyn Fn(&str) -> String,
) -> ShimFunc {
    let name = CString::new(entry.export.clone()).unwrap_or_default();
    let classes: Vec<u8> = entry.params.chars().map(arg_class).collect();
    let retc = ret_class(entry.ret.chars().next().unwrap_or('v'));
    let ret_raw = entry.ret.chars().next().unwrap_or('v');
    let ret_label = if entry.ret_type.is_empty() {
        entry.type_name.clone()
    } else {
        entry.ret_type.clone()
    };
    let drop_sym = drop_of(&ret_label);
    let has_recv = entry.self_kind != "static";
    let is_move = entry.self_kind == "move";
    let chain = entry.chain;
    let fallible = entry.fallible;
    let crate_name = crate_name.to_string();
    let ctx = format!("{}.{}", entry.type_name, entry.method);

    Arc::new(move |task: &mut AutoTask, vm: &AutoVM| -> Result<(), VMError> {
        // 右到左弹参;接收者(参数位 0)单独弹以记录其堆句柄(chain 压回/move 置空要用)
        let mut pool: Vec<CString> = Vec::new();
        let mut cargs: Vec<CArg> = Vec::with_capacity(classes.len());
        let mut recv_handle: Option<u64> = None;
        for (i, &cl) in classes.iter().enumerate().rev() {
            if i == 0 && has_recv {
                let (h, ptr) = pop_dep_handle(task, vm, &ctx)?;
                recv_handle = Some(h);
                cargs.push(CArg::P(ptr));
                continue;
            }
            match cl {
                // 布尔与整型统一 i64 槽(pop_int 按 nanbox 标签解码)
                b'I' => cargs.push(CArg::I(pop_int(task)?)),
                b'F' => cargs.push(CArg::F(
                    f64::pop_from_stack(task, vm)
                        .map_err(|e| VMError::RuntimeError(format!("{ctx} pop: {e}")))?,
                )),
                b'S' => {
                    let s = String::pop_from_stack(task, vm)
                        .map_err(|e| VMError::RuntimeError(format!("{ctx} pop: {e}")))?;
                    let cs = CString::new(s)
                        .unwrap_or_else(|_| CString::new("").unwrap());
                    let p = cs.as_ptr();
                    pool.push(cs);
                    cargs.push(CArg::S(p));
                }
                _ => cargs.push(CArg::P(pop_dep_ptr(task, vm, &ctx)?)),
            }
        }
        cargs.reverse();

        // 调用(按元数×参数类归并;v1 支持至多 3 个 ABI 参数)
        let ai = |i: usize| -> i64 {
            match cargs.get(i) {
                Some(CArg::I(v)) => *v,
                _ => 0,
            }
        };
        let af = |i: usize| -> f64 {
            match cargs.get(i) {
                Some(CArg::F(v)) => *v,
                _ => 0.0,
            }
        };
        let as_ = |i: usize| -> *const c_char {
            match cargs.get(i) {
                Some(CArg::S(p)) => *p,
                _ => std::ptr::null(),
            }
        };
        let ap = |i: usize| -> *mut c_void {
            match cargs.get(i) {
                Some(CArg::P(p)) => *p,
                _ => std::ptr::null_mut(),
            }
        };
        match classes.as_slice() {
            [] => ret_call!(lib, name, task, vm, ctx, (), (), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
            [a] => match a {
                b'I' => ret_call!(lib, name, task, vm, ctx, (i64), (ai(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                b'F' => ret_call!(lib, name, task, vm, ctx, (f64), (af(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                b'S' => ret_call!(lib, name, task, vm, ctx, (*const c_char), (as_(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                _ => ret_call!(lib, name, task, vm, ctx, (*mut c_void), (ap(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
            },
            [a, b] => match (a, b) {
                (b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, i64), (ai(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, f64), (ai(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char), (ai(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', _) => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void), (ai(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, i64), (af(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, f64), (af(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char), (af(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', _) => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void), (af(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64), (as_(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64), (as_(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char), (as_(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', _) => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void), (as_(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (_, b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64), (ap(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (_, b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64), (ap(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (_, b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char), (ap(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                _ => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void), (ap(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
            },
            [a, b, c] => match (a, b, c) {
                (b'I', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, i64, i64), (ai(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, i64, f64), (ai(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, i64, *const c_char), (ai(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, i64, *mut c_void), (ai(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, f64, i64), (ai(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, f64, f64), (ai(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, f64, *const c_char), (ai(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, f64, *mut c_void), (ai(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, i64), (ai(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, f64), (ai(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, *const c_char), (ai(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, *mut c_void), (ai(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, i64), (ai(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, f64), (ai(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, *const c_char), (ai(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'I', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, *mut c_void), (ai(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, i64, i64), (af(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, i64, f64), (af(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, i64, *const c_char), (af(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, i64, *mut c_void), (af(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, f64, i64), (af(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, f64, f64), (af(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, f64, *const c_char), (af(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, f64, *mut c_void), (af(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, i64), (af(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, f64), (af(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, *const c_char), (af(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, *mut c_void), (af(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, i64), (af(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, f64), (af(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, *const c_char), (af(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'F', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, *mut c_void), (af(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, i64), (as_(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, f64), (as_(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, *const c_char), (as_(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, *mut c_void), (as_(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, i64), (as_(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, f64), (as_(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, *const c_char), (as_(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, *mut c_void), (as_(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, i64), (as_(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, f64), (as_(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, *const c_char), (as_(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, *mut c_void), (as_(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, i64), (as_(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, f64), (as_(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, *const c_char), (as_(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'S', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, *mut c_void), (as_(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, i64), (ap(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, f64), (ap(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, *const c_char), (ap(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, *mut c_void), (ap(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, i64), (ap(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, f64), (ap(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, *const c_char), (ap(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, *mut c_void), (ap(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, i64), (ap(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, f64), (ap(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, *const c_char), (ap(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, *mut c_void), (ap(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, i64), (ap(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, f64), (ap(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, *const c_char), (ap(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                (b'P', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, *mut c_void), (ap(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
                // 未知类字节兜底(按指针处理;正常 manifest 不会出现)
                _ => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, *mut c_void), (ap(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, &recv_handle, ret_raw),
            },
            _ => {
                return Err(VMError::RuntimeError(format!(
                    "{ctx}: too many ABI params ({}) for dep method (v1 supports ≤3)",
                    classes.len()
                )));
            }
        }

        // 按值 self(wrapper 侧 Box::from_raw 已消耗对象):旧句柄置空,
        // 防止其 Drop 时对已释放指针二次析构。
        if is_move {
            if let Some(h) = recv_handle {
                if let Some(obj) = vm.get_heap_object(h) {
                    if let Ok(mut guard) = obj.write() {
                        if let Some(dep) = guard.as_any_mut().downcast_mut::<DepOpaqueObject>() {
                            dep.ptr = std::ptr::null_mut();
                            dep.drop_export.clear();
                        }
                    }
                }
            }
        }
        Ok(())
    })
}

/// 整型/布尔统一按 i64 槽弹(布尔经 nv 解码)。
fn pop_int(task: &mut AutoTask) -> Result<i64, VMError> {
    let nv = task.ram.pop_nv();
    if auto_val::is_i32(nv) {
        Ok(auto_val::decode_i32(nv) as i64)
    } else if auto_val::is_i64(nv) {
        Ok(auto_val::decode_i64(nv))
    } else if auto_val::is_bool(nv) {
        Ok(auto_val::decode_bool(nv) as i64)
    } else if auto_val::is_null(nv) {
        Ok(0)
    } else {
        Err(VMError::RuntimeError(
            "plan430: expected integer/bool arg".into(),
        ))
    }
}
