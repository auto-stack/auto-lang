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
    /// 类型布局(PLAN-591 T1:探针实测;CALL_SPEC offset 直读直写依据;
    /// 装载期由 auto__shim_layouts 合并,按 crate::类型 注册)
    pub layout: Option<Arc<shim_metadata::emit_cdylib::TypeLayout>>,
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
    let layout = lookup_layout(crate_name, short_type);
    let obj = DepOpaqueObject {
        crate_name: crate_name.to_string(),
        short_type: short_type.to_string(),
        full_type,
        ptr,
        drop_export: drop_export.to_string(),
        lib,
        layout,
    };
    let handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

// =============================================================================
// 布局注册表与 offset 直读直写(PLAN-591 T1)
// =============================================================================

fn layouts_table() -> &'static RwLock<HashMap<String, Arc<shim_metadata::emit_cdylib::TypeLayout>>> {
    static T: OnceLock<RwLock<HashMap<String, Arc<shim_metadata::emit_cdylib::TypeLayout>>>> =
        OnceLock::new();
    T.get_or_init(|| RwLock::new(HashMap::new()))
}

fn lookup_layout(
    crate_name: &str,
    short_type: &str,
) -> Option<Arc<shim_metadata::emit_cdylib::TypeLayout>> {
    layouts_table()
        .read()
        .expect("plan591 layouts table poisoned")
        .get(&format!("{crate_name}::{short_type}"))
        .cloned()
}

/// 标量字段读值(offset 直读产物;String/嵌套句柄不在直读面,走合成 getter)。
#[derive(Debug, Clone, Copy)]
pub enum ScalarFieldValue {
    I(i64),
    F(f64),
    B(bool),
}

/// offset 直读:layout 命中标量字段 → 按投影类型宽度读 cdylib 堆。
/// SAFETY: ptr 指向 wrapper cdylib 堆上的 Box<T> 本体;偏移由同 rustc 实例
/// 的 offset_of! 探针实测,读侧 read_unaligned 容忍任意对齐填充。
pub fn read_scalar_field(
    obj: &DepOpaqueObject,
    field: &str,
) -> Option<ScalarFieldValue> {
    let layout = obj.layout.as_ref()?;
    let f = layout.fields.iter().find(|f| f.name == field)?;
    let base = obj.ptr as *const u8;
    let v = unsafe {
        match f.ty.as_str() {
            "i8" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<i8>().read_unaligned() as i64),
            "i16" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<i16>().read_unaligned() as i64),
            "i32" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<i32>().read_unaligned() as i64),
            "i64" | "isize" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<i64>().read_unaligned()),
            "u8" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<u8>().read_unaligned() as i64),
            "u16" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<u16>().read_unaligned() as i64),
            "u32" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<u32>().read_unaligned() as i64),
            "u64" | "usize" => ScalarFieldValue::I(base.add(f.offset as usize).cast::<u64>().read_unaligned() as i64),
            "bool" => ScalarFieldValue::B(base.add(f.offset as usize).cast::<u8>().read_unaligned() != 0),
            "f32" => ScalarFieldValue::F(base.add(f.offset as usize).cast::<f32>().read_unaligned() as f64),
            "f64" => ScalarFieldValue::F(base.add(f.offset as usize).cast::<f64>().read_unaligned()),
            _ => return None, // String/嵌套句柄等非标量:走合成 getter 路由
        }
    };
    Some(v)
}

/// offset 直写(PLAN-591 V1-2):标量字段按 VM 值宽度写 cdylib 堆。
/// V1 纪律:单线程批处理执行;别名/写后 Rust 侧缓存失效语义登记 KNOWN-DEBT。
/// String/嵌套句柄不在 V1 写面(需 clone/构造语义,归后续计划)。
pub fn write_scalar_field(
    vm: &AutoVM,
    obj: &DepOpaqueObject,
    field: &str,
    nv: auto_val::NanoValue,
) -> Result<(), VMError> {
    let ctx = format!("{}.{}", obj.short_type, field);
    let Some(layout) = obj.layout.as_ref() else {
        return Err(VMError::RuntimeError(format!(
            "{ctx}: no layout info (manifest v2 layouts missing) — field write requires a rebuilt methods pack"
        )));
    };
    let Some(f) = layout.fields.iter().find(|f| f.name == field) else {
        return Err(VMError::RuntimeError(format!(
            "unknown field '{field}' on dep object {} (无布局条目:非 pub 字段不可写)",
            obj.full_type
        )));
    };
    let scalar_ty = f.ty.as_str();
    let base = obj.ptr as *mut u8;
    unsafe {
        match scalar_ty {
            "bool" => {
                let b = if auto_val::is_bool(nv) {
                    auto_val::decode_bool(nv)
                } else {
                    return Err(VMError::RuntimeError(format!("{ctx}: expected bool value")));
                };
                base.add(f.offset as usize).cast::<u8>().write_unaligned(b as u8);
            }
            "f32" | "f64" => {
                let v = if auto_val::is_f64(nv) {
                    auto_val::decode_f64(nv)
                } else if auto_val::is_i32(nv) {
                    auto_val::decode_i32(nv) as f64
                } else {
                    return Err(VMError::RuntimeError(format!("{ctx}: expected numeric value")));
                };
                if scalar_ty == "f32" {
                    base.add(f.offset as usize).cast::<f32>().write_unaligned(v as f32);
                } else {
                    base.add(f.offset as usize).cast::<f64>().write_unaligned(v);
                }
            }
            int if int.starts_with(['i', 'u']) || int == "isize" || int == "usize" => {
                let v = if auto_val::is_i32(nv) {
                    auto_val::decode_i32(nv) as i64
                } else if auto_val::is_i64(nv) {
                    auto_val::decode_i64(nv)
                } else if auto_val::is_bool(nv) {
                    auto_val::decode_bool(nv) as i64
                } else {
                    crate::vm::ffi::convert::decode_i64_full(vm, nv)
                };
                let p = base.add(f.offset as usize);
                match int {
                    "i8" => p.cast::<i8>().write_unaligned(v as i8),
                    "i16" => p.cast::<i16>().write_unaligned(v as i16),
                    "i32" => p.cast::<i32>().write_unaligned(v as i32),
                    "i64" | "isize" => p.cast::<i64>().write_unaligned(v),
                    "u8" => p.cast::<u8>().write_unaligned(v as u8),
                    "u16" => p.cast::<u16>().write_unaligned(v as u16),
                    "u32" => p.cast::<u32>().write_unaligned(v as u32),
                    "u64" | "usize" => p.cast::<u64>().write_unaligned(v as u64),
                    other => {
                        return Err(VMError::RuntimeError(format!(
                            "{ctx}: unsupported integer field type '{other}'"
                        )))
                    }
                }
            }
            other => {
                return Err(VMError::RuntimeError(format!(
                    "{ctx}: field type '{other}' is not in the V1 scalar write surface (String/嵌套句柄需 clone/构造语义)"
                )));
            }
        }
    }
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
    // PLAN-591 T2:.unwrap() 语义桥——unwrap 范式(594 语料惯例)要求 .at 侧
    // `.unwrap()` 原文透传(a2r 轨发射合法 Rust);VM 侧 T2 已在调用边界解包
    // (Some 压值/None 压 null/Err 转 VMError),unwrap 对 dep 值为透明恒等
    // (原样弹压,不参与 RC)。null 接收者 unwrap → VMError(对齐 Option unwrap)。
    // 已知让步:若三方类型自带真实 unwrap 固有方法,会被本桥遮蔽(dep 面)。
    if method == "unwrap" {
        let nv = task.ram.pop_nv();
        if auto_val::is_null(nv) {
            return Err(VMError::RuntimeError(format!(
                "{type_name}.unwrap: unwrap on null (None) — PLAN-591 T2 在调用边界以 null 表达 None"
            )));
        }
        task.ram.push_nv(nv);
        return Ok(true);
    }
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

/// PLAN-591 D2(DIV-DEP-8 print 半边):dep 对象的 Display 字符串化——
/// 经 shim 包合成 to_string(rustdoc 对 impl Display 类型合成,见 rustdoc.rs
/// F 轮面)。无 Display 面 → Ok(None),调用方维持占位行为。
/// RC 纪律:obj_id 以 raw push 入栈作接收者(shim 侧 raw pop,净零),不增不减。
pub fn display_string(
    task: &mut AutoTask,
    vm: &AutoVM,
    obj_id: u64,
    short_type: &str,
) -> Result<Option<String>, VMError> {
    task.ram.push_nv(auto_val::encode_object(obj_id as u32));
    let hit = dispatch(short_type, "to_string", task, vm)?;
    if !hit {
        return Ok(None);
    }
    let nv = task.ram.pop_nv();
    if auto_val::is_string(nv) {
        let idx = auto_val::decode_string(nv);
        Ok(vm.get_string(idx).map(|b| String::from_utf8_lossy(&b).into_owned()))
    } else {
        Ok(None)
    }
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

    // PLAN-591 T1(AC-04):manifest format 装载校验拒载旧版——旧 format 的
    // 布局段缺失,offset 语义不成立;拒载强制管线重建(format bump 通道)。
    if manifest.format != shim_metadata::emit_cdylib::MANIFEST_FORMAT {
        log::warn!(
            "plan430: manifest format {} for {crate_name} unsupported (expected {}); rebuild the methods pack",
            manifest.format,
            shim_metadata::emit_cdylib::MANIFEST_FORMAT
        );
        return;
    }

    // PLAN-596 T5:注入宿主跳板——回调方法所在的包导出
    // auto__register_host_trampoline(无回调面的包无此导出,缺席即跳过);
    // 跳板经线程局部回调帧在 marshaller 调用窗口内重入解释器。
    unsafe {
        if let Ok(reg) = lib.get(b"auto__register_host_trampoline") {
            let reg: libloading::Symbol<unsafe extern "C" fn(unsafe extern "C" fn(u64, i64) -> i64)> = reg;
            reg(dep_host_trampoline);
        }
    }

    // PLAN-591 T1:布局探针装载——cdylib 内嵌 auto__shim_layouts(offset_of!/
    // size_of 实测,同编译保证),合并进 layouts 注册表(key = crate::类型)。
    // 导出缺失(空探针/旧包)按缺省容忍:DepOpaqueObject.layout = None。
    type LayoutsFn = unsafe extern "C" fn() -> *const c_char;
    if let Ok(sym) = unsafe { lib.get::<LayoutsFn>(b"auto__shim_layouts") } {
        let ptr = unsafe { sym() };
        if !ptr.is_null() {
            let s = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
            call_free_cstring(&lib, ptr as *mut c_char);
            match serde_json::from_str::<shim_metadata::emit_cdylib::LayoutMap>(&s) {
                Ok(map) => {
                    let mut table = layouts_table()
                        .write()
                        .expect("plan591 layouts table poisoned");
                    for (ty, layout) in map {
                        table.insert(format!("{crate_name}::{ty}"), Arc::new(layout));
                    }
                }
                Err(e) => log::warn!("plan430: bad layouts json for {crate_name}: {e}"),
            }
        }
    }

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
/// $nullable(PLAN-591 T2):null 指针返回 → 压 null 值(仅 s/p 槽)。
macro_rules! ret_call {
    ($lib:expr, $name:expr, $task:expr, $vm:expr, $ctx:expr, ($($t:ty),*), ($($a:expr),*), $retc:expr, $ret_label:expr, $drop_sym:expr, $crate_nm:expr, $chain:expr, $fallible:expr, $nullable:expr, $recv_handle:expr, $ret_raw:expr) => {{
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
                // bool 返回只保证 al 有效(高位是垃圾),必须掩码;i32 槽返回
                // x64 写 32 位寄存器时零扩展,负数需符号扩展回 i64(PLAN-592:
                // echo_i8(-128) 曾呈现 4294967168);i64 直传。
                // 大整数走 heap-aware 压栈(virt_memory 48 位内联范围限制)。
                if $ret_raw == 'b' {
                    $task.ram.push_nv(auto_val::encode_bool((r & 0xFF) != 0));
                } else if $ret_raw == 'i' {
                    $vm.push_i64_vm($task, r as u32 as i32 as i64);
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
                if $nullable && r.is_null() {
                    // PLAN-591 T2:Option None → null 值(区别于空串)
                    $task.ram.push_nv(auto_val::encode_null());
                } else {
                    let s = if r.is_null() {
                        String::new()
                    } else {
                        unsafe { CStr::from_ptr(r) }.to_string_lossy().into_owned()
                    };
                    call_free_cstring(&$lib, r);
                    let idx = $vm.add_string(s.into_bytes());
                    $vm.rc_push_str_idx($task, idx);
                }
            }
            _ => {
                let sym = getsym!($lib, $name, ($($t),*) -> *mut c_void);
                let r = unsafe { sym($($a),*) };
                check_err!($lib, $ctx, $fallible);
                if $nullable && r.is_null() {
                    // PLAN-591 T2:Option None → null 值(chain 句柄也不压)
                    $task.ram.push_nv(auto_val::encode_null());
                } else if $chain {
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
    let nullable = entry.nullable;
    let crate_name = crate_name.to_string();
    let ctx = format!("{}.{}", entry.type_name, entry.method);
    // PLAN-596 T5:回调形参清单(调用窗口设帧用)
    let entry_callbacks = entry.callbacks.clone();

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
                // 布尔与整型统一 i64 槽(pop_int 按 nanbox 标签 + 堆感知 BigInt 解码)
                b'I' => cargs.push(CArg::I(pop_int(task, vm)?)),
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

        // PLAN-596 T5:回调方法——调用窗口设帧(跳板经线程局部读 task/vm),
        // 调后清帧并检查 panic 通道(闭包 panic 被跳板 catch_unwind 捕获)。
        let has_callbacks = !entry_callbacks.is_empty();
        let _ = &entry_callbacks;
        if has_callbacks {
            CB_PANIC.with(|p| *p.borrow_mut() = None);
            CB_FRAME.with(|f| {
                *f.borrow_mut() = Some((task as *mut AutoTask, vm as *const AutoVM))
            });
        }

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
            [] => ret_call!(lib, name, task, vm, ctx, (), (), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
            [a] => match a {
                b'I' => ret_call!(lib, name, task, vm, ctx, (i64), (ai(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                b'F' => ret_call!(lib, name, task, vm, ctx, (f64), (af(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                b'S' => ret_call!(lib, name, task, vm, ctx, (*const c_char), (as_(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                _ => ret_call!(lib, name, task, vm, ctx, (*mut c_void), (ap(0)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
            },
            [a, b] => match (a, b) {
                (b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, i64), (ai(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, f64), (ai(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char), (ai(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', _) => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void), (ai(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, i64), (af(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, f64), (af(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char), (af(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', _) => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void), (af(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64), (as_(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64), (as_(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char), (as_(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', _) => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void), (as_(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (_, b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64), (ap(0), ai(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (_, b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64), (ap(0), af(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (_, b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char), (ap(0), as_(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                _ => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void), (ap(0), ap(1)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
            },
            [a, b, c] => match (a, b, c) {
                (b'I', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, i64, i64), (ai(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, i64, f64), (ai(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, i64, *const c_char), (ai(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, i64, *mut c_void), (ai(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, f64, i64), (ai(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, f64, f64), (ai(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, f64, *const c_char), (ai(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, f64, *mut c_void), (ai(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, i64), (ai(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, f64), (ai(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, *const c_char), (ai(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, *const c_char, *mut c_void), (ai(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, i64), (ai(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, f64), (ai(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, *const c_char), (ai(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'I', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (i64, *mut c_void, *mut c_void), (ai(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, i64, i64), (af(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, i64, f64), (af(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, i64, *const c_char), (af(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, i64, *mut c_void), (af(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, f64, i64), (af(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, f64, f64), (af(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, f64, *const c_char), (af(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, f64, *mut c_void), (af(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, i64), (af(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, f64), (af(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, *const c_char), (af(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, *const c_char, *mut c_void), (af(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, i64), (af(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, f64), (af(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, *const c_char), (af(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'F', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (f64, *mut c_void, *mut c_void), (af(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, i64), (as_(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, f64), (as_(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, *const c_char), (as_(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, i64, *mut c_void), (as_(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, i64), (as_(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, f64), (as_(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, *const c_char), (as_(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, f64, *mut c_void), (as_(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, i64), (as_(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, f64), (as_(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, *const c_char), (as_(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *const c_char, *mut c_void), (as_(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, i64), (as_(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, f64), (as_(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, *const c_char), (as_(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'S', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (*const c_char, *mut c_void, *mut c_void), (as_(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'I', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, i64), (ap(0), ai(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'I', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, f64), (ap(0), ai(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'I', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, *const c_char), (ap(0), ai(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'I', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, i64, *mut c_void), (ap(0), ai(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'F', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, i64), (ap(0), af(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'F', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, f64), (ap(0), af(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'F', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, *const c_char), (ap(0), af(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'F', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, f64, *mut c_void), (ap(0), af(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'S', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, i64), (ap(0), as_(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'S', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, f64), (ap(0), as_(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'S', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, *const c_char), (ap(0), as_(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'S', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *const c_char, *mut c_void), (ap(0), as_(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'P', b'I') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, i64), (ap(0), ap(1), ai(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'P', b'F') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, f64), (ap(0), ap(1), af(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'P', b'S') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, *const c_char), (ap(0), ap(1), as_(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                (b'P', b'P', b'P') => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, *mut c_void), (ap(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
                // 未知类字节兜底(按指针处理;正常 manifest 不会出现)
                _ => ret_call!(lib, name, task, vm, ctx, (*mut c_void, *mut c_void, *mut c_void), (ap(0), ap(1), ap(2)), retc, ret_label, drop_sym, crate_name, chain, fallible, nullable, &recv_handle, ret_raw),
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
        if has_callbacks {
            CB_FRAME.with(|f| *f.borrow_mut() = None);
            if let Some(msg) = CB_PANIC.with(|p| p.borrow_mut().take()) {
                return Err(VMError::RuntimeError(msg));
            }
        }
        Ok(())
    })
}

// =============================================================================
// PLAN-596 T5: 反向回调 adapter(原型)——宿主跳板 + 线程局部回调帧
// =============================================================================

/// 同步调用期有效的回调帧(裸指针仅在 marshaller 调用窗口内解引用;
/// 单线程原型,深度 1)。Option 空即"不在回调窗口"。
thread_local! {
    static CB_FRAME: std::cell::RefCell<Option<(*mut AutoTask, *const AutoVM)>> =
        const { std::cell::RefCell::new(None) };
    static CB_PANIC: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// 宿主跳板:wrapper adapter 经注入指针调用——token=VM 闭包 id,
/// 重入解释器执行 .at 闭包(单参 i64→i64),返回值直传。
/// 深度守卫:帧非空 = 嵌套回调 → 记错误返回 0(marshaller 转 VMError)。
unsafe extern "C" fn dep_host_trampoline(token: u64, arg: i64) -> i64 {
    let frame = CB_FRAME.with(|f| *f.borrow());
    let Some((task_ptr, vm_ptr)) = frame else {
        CB_PANIC.with(|p| {
            *p.borrow_mut() = Some("callback re-entry outside call window (depth>1 or stray)".into())
        });
        return 0;
    };
    // 深度守卫:执行期置帧为 None,回调内再调回调形参即被拒
    CB_FRAME.with(|f| *f.borrow_mut() = None);
    let task = unsafe { &mut *task_ptr };
    let vm = unsafe { &*vm_ptr };
    task.ram.push_i32(arg as i32);
    let closure_id = token as u32;
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vm.call_closure(task, closure_id, 1)
    }));
    CB_FRAME.with(|f| *f.borrow_mut() = Some((task_ptr, vm_ptr)));
    match outcome {
        Ok(Ok(())) => {
            // 闭包返回值在栈顶(镜像 CALL_CLOSURE 消费约定)
            task.ram.pop_i32() as i64
        }
        Ok(Err(e)) => {
            CB_PANIC.with(|p| *p.borrow_mut() = Some(format!("callback error: {e:?}")));
            0
        }
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "unknown panic in callback".into());
            CB_PANIC.with(|p| *p.borrow_mut() = Some(format!("callback panicked: {msg}")));
            0
        }
    }
}

/// 整型/布尔统一按 i64 槽弹(布尔经 nv 解码)。
/// PLAN-592:超 48 位内联的 i64 字面量在 VM 侧装箱为 BigInt(Plan 377)——参数
/// 方向此前只认 nanbox 标签,>2^48 直接弹栈失败。堆感知兜底走 convert 的
/// decode_i64_full(TAG_U64/TAG_BIGINT/原始 f64 槽),非数值形态维持原报错。
fn pop_int(task: &mut AutoTask, vm: &AutoVM) -> Result<i64, VMError> {
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
        let t = auto_val::tag_of(nv);
        if t == 9 || t == 0xA || !auto_val::is_nanboxed(nv) {
            Ok(crate::vm::ffi::convert::decode_i64_full(vm, nv))
        } else {
            Err(VMError::RuntimeError(
                "plan430: expected integer/bool arg".into(),
            ))
        }
    }
}
