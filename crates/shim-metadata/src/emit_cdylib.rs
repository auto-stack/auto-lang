//! 生成器:MarshalPlan → 三方 crate cdylib shim 包源码(plan-430 C1)。
//!
//! 与 std 路径(emit.rs,追加段编入 auto-lang)相对,三方路径产物是独立 cdylib:
//! 方法 wrapper 以裸指针跨 C ABI(`auto_<Type>_<method>_<sig>`),对象本体留在
//! shim 包 crate 的堆上,VM 侧只持有句柄(auto-lang 的 DepOpaqueObject)。
//! manifest/signatures/rules 三份 JSON 与源码同目录落盘(430-a1 shimpack v1 格式)。
//!
//! 签名码字母表沿用 plan-212 sig_code:v/i/l/f/b/s/p(p = 裸指针/不透明句柄)。
//! 方法 ABI 参数码**含接收者前导 'p'**(read/write/move),static 无接收者。

use crate::classify::{Classified, Exceptions};
use crate::types::*;
use serde::{Deserialize, Serialize};

pub const GENERATOR: &str = "shim-metadata v1.1 (plan-430 C1)";
pub const MANIFEST_FORMAT: u32 = 1;
pub const CLASSIFIER_VERSION: u32 = 1;

/// shim 包清单(运行期契约:auto-lang 按此注册 dispatch 与 marshaller)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShimManifest {
    pub format: u32,
    pub crate_name: String,
    pub crate_version: String,
    /// 元信息版本指纹(C3:工具链 × crate × 生成器 × 签名集)
    pub fingerprint: String,
    /// 提取该元信息用的 rustdoc 工具链描述(含 nightly 版本)
    pub toolchain: String,
    pub generator: String,
    pub methods: Vec<MethodEntry>,
    /// 自由函数仅作元信息(D2:known_signature 元数据优先);
    /// 代码生成仍走 plan-212 syn 路径,避免双生成器符号冲突。
    pub functions: Vec<FunctionEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodEntry {
    /// 短类型名(dispatch 3000 的 key,如 "Counter")
    pub type_name: String,
    /// 全限定类型标签(VM 堆 tag,如 "my_crate::Counter")
    pub full_type: String,
    pub method: String,
    /// 导出符号全名
    pub export: String,
    /// "static" | "read" | "write" | "move"
    pub self_kind: String,
    /// C ABI 参数码(非 static 时含接收者前导 'p')
    pub params: String,
    /// 返回码 v/i/l/f/b/s/p
    pub ret: String,
    /// ret == "p" 时的返回类型短名(VM 堆标签用),否则空
    pub ret_type: String,
    /// ChainInPlace 标记:返回 &Self/&mut Self(同一对象)→ VM 压回原句柄,
    /// 区别于返回同类**新**实例(clone_reset 类)的普通 'p'
    #[serde(default)]
    pub chain: bool,
    /// unwrap_ok 标记:原返回 Result<T,E>,wrapper 解 Ok;
    /// Err 经 auto__last_error 通道传出 → VM 侧转 VMError(430-F)
    #[serde(default)]
    pub fallible: bool,
    /// 该类型的析构符号(auto__drop_<Type>)
    pub drop_export: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    pub name: String,
    pub params: String,
    pub ret: String,
}

/// 包元信息(指纹输入)。
pub struct PackMeta {
    pub crate_name: String,
    pub crate_version: String,
    pub toolchain: String,
}

/// 一个 shim 包的全部落盘文件。
pub struct PackFiles {
    pub cargo_toml: String,
    pub lib_rs: String,
    pub manifest_json: String,
    pub signatures_json: String,
    pub rules_json: String,
}

// =============================================================================
// C3:版本指纹
// =============================================================================

/// fnv1a 64 位(指纹用;不引入外部依赖)。
pub fn fnv1a64(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf_29ce_4842_2235;
    for &b in data {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// 元信息版本指纹:工具链 × crate × 生成器 × 分类结果签名集(C3,防签名漂移)。
/// 同一 (crate, toolchain, generator) 下签名集变化 → 指纹变化 → 缓存失效重建。
pub fn fingerprint(meta: &PackMeta, c: &Classified, free_fns: &[ShimMethod]) -> String {
    fingerprint_parts(meta, &c.plans, free_fns)
}

/// parts 版指纹(供 rustc 检查器剔除环重算用)。
/// 430 复审修复:泛型自由函数不进指纹——它们的签名含未解决泛型,
/// `classify_ret` 只能给出 Void 假码,指纹行反而失真;这类函数
/// 连同 manifest 条目一并跳过(见 emit_pack_parts)。
pub fn fingerprint_parts(
    meta: &PackMeta,
    plans: &[MarshalPlan],
    free_fns: &[ShimMethod],
) -> String {
    let mut lines: Vec<String> = plans.iter().map(plan_sig_line).collect();
    lines.extend(free_fns.iter().filter(|f| !f.generic).map(|f| {
        format!(
            "fn {}.{}|{}|{}",
            f.method,
            "",
            f.params.iter().map(param_char_of).collect::<String>(),
            ret_char_of(&classify_ret(&f.ret))
        )
    }));
    lines.sort();
    let payload = format!(
        "{}\n{}\n{}\n{}\n{}\n",
        meta.toolchain,
        meta.crate_name,
        meta.crate_version,
        GENERATOR,
        CLASSIFIER_VERSION
    ) + &lines.join("\n");
    format!("{:016x}", fnv1a64(payload.as_bytes()))
}

fn plan_sig_line(p: &MarshalPlan) -> String {
    let m = &p.method;
    // 参数同时记录 Ty 名与 ABI 码:宽度信息(u64 vs i64 等)变化必须改变指纹;
    // fallible/chain 策略位同样入指纹。
    let params: Vec<String> = m
        .params
        .iter()
        .zip(p.args.iter())
        .map(|(t, a)| format!("{}:{}", t.rust_name(), arg_char(a)))
        .collect();
    format!(
        "m {}.{}|{:?}|{}|{:?}|f{}|c{}|fld:{:?}",
        m.type_name,
        m.method,
        m.self_kind,
        params.join(","),
        p.ret,
        p.fallible as u8,
        matches!(p.ret, RetPlan::ChainInPlace) as u8,
        m.field,
    )
}

/// 分类器自由函数条目的返回码(独立小函数,避免借 classify_one 的私有逻辑)。
fn classify_ret(t: &Ty) -> RetPlan {
    match t {
        Ty::I8 | Ty::I16 | Ty::I32 | Ty::U8 | Ty::U16 | Ty::U32 => RetPlan::ScalarI32,
        Ty::I64 | Ty::U64 | Ty::Usize => RetPlan::ScalarI64,
        Ty::F32 | Ty::F64 => RetPlan::ScalarF64,
        Ty::Bool => RetPlan::ScalarBool,
        Ty::Str | Ty::StrOwned => RetPlan::ScalarStr,
        Ty::SelfTy => RetPlan::ChainSelf,
        Ty::Opaque(n) | Ty::OpaqueOwned(n) => RetPlan::Opaque(n.clone()),
        Ty::Void => RetPlan::Void,
        Ty::Generic(_) => RetPlan::Void,
    }
}

fn param_char_of(t: &Ty) -> char {
    match t {
        Ty::Str | Ty::StrOwned => 's',
        Ty::I8 | Ty::I16 | Ty::I32 | Ty::U8 | Ty::U16 | Ty::U32 => 'i',
        Ty::I64 | Ty::U64 | Ty::Usize => 'l',
        Ty::F32 | Ty::F64 => 'f',
        Ty::Bool => 'b',
        Ty::Opaque(_) | Ty::OpaqueOwned(_) | Ty::SelfTy => 'p',
        Ty::Generic(_) | Ty::Void => '?',
    }
}

// =============================================================================
// 生成主入口
// =============================================================================

/// 生成 shim 包全部文件。返回 (指纹, 文件集);指纹同时写入 manifest 与
/// lib_rs 内嵌的 shim_manifest 导出,加载侧据文件名核对防漂移。
pub fn emit_pack(
    meta: &PackMeta,
    dep_line: &str,
    c: &Classified,
    exc: &Exceptions,
    free_fns: &[ShimMethod],
) -> (String, PackFiles) {
    emit_pack_parts(meta, dep_line, &c.plans, &c.skips, exc, free_fns)
}

/// parts 版生成(供 rustc 检查器剔除环按缩减后的计划集重生成)。
pub fn emit_pack_parts(
    meta: &PackMeta,
    dep_line: &str,
    plans: &[MarshalPlan],
    skips: &[Skip],
    exc: &Exceptions,
    free_fns: &[ShimMethod],
) -> (String, PackFiles) {
    let fp = fingerprint_parts(meta, plans, free_fns);
    let crate_ident = meta.crate_name.replace('-', "_");

    // 方法条目与 wrapper 源码
    let mut entries = Vec::new();
    let mut wrappers = String::new();
    let mut drop_fns = String::new();
    let mut dropped_types: Vec<String> = Vec::new();
    for p in plans {
        let m = &p.method;
        let full = format!("{crate_ident}::{}", m.type_name);
        if !dropped_types.contains(&full) {
            dropped_types.push(full.clone());
            drop_fns.push_str(&emit_drop_fn(&m.type_name, &full));
        }
        let (entry, src) = emit_wrapper(&crate_ident, p);
        entries.push(entry);
        wrappers.push_str(&src);
    }

    // 自由函数元信息(仅清单)。
    // 430 复审修复:泛型自由函数(fn foo<T>(x: T) -> T)过滤——rustdoc 无法
    // 单态化,`fn foo<T>(x: T) -> T` 经 classify_ret 只能落 RetPlan::Void,
    // 以假 'v' 签名进 manifest 会被 resolve_signature 采信,错误类型推断
    // 传导进 codegen。与方法的泛型跳过规则(classify)口径一致。
    let functions: Vec<FunctionEntry> = free_fns
        .iter()
        .filter(|f| !f.generic)
        .map(|f| FunctionEntry {
            name: f.method.clone(),
            params: f.params.iter().map(param_char_of).collect(),
            ret: ret_char_of(&classify_ret(&f.ret)).to_string(),
        })
        .collect();
    let skipped_generic_fns: Vec<&str> = free_fns
        .iter()
        .filter(|f| f.generic)
        .map(|f| f.method.as_str())
        .collect();
    if !skipped_generic_fns.is_empty() {
        log::warn!(
            "plan430: skipped {} generic free function(s) in {}: {:?}",
            skipped_generic_fns.len(),
            meta.crate_name,
            skipped_generic_fns
        );
    }

    let manifest = ShimManifest {
        format: MANIFEST_FORMAT,
        crate_name: meta.crate_name.clone(),
        crate_version: meta.crate_version.clone(),
        fingerprint: fp.clone(),
        toolchain: meta.toolchain.clone(),
        generator: GENERATOR.to_string(),
        methods: entries,
        functions,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest).unwrap_or_default();

    let lib_rs = emit_lib_rs(&manifest_json, &drop_fns, &wrappers);

    let cargo_toml = format!(
        "# Generated by shim-metadata (plan-430 C1). DO NOT EDIT BY HAND.\n\
         [package]\n\
         name = \"{crate_ident}_methods_wrapper\"\n\
         version = \"1.0.0\"\n\
         edition = \"2021\"\n\
         \n\
         # 独立工作区:防止被宿主 repo 的 workspace 吸收\n\
         [workspace]\n\
         \n\
         [lib]\n\
         crate-type = [\"cdylib\"]\n\
         \n\
         [dependencies]\n\
         {dep_line}\n"
    );

    let signatures_json = serde_json::json!({
        "crate": meta.crate_name,
        "version": meta.crate_version,
        "methods": plans.iter().map(|p| serde_json::json!({
            "type": p.method.type_name,
            "method": p.method.method,
            "self": format!("{:?}", p.method.self_kind),
            "params": p.method.params.iter().map(|t| t.rust_name()).collect::<Vec<_>>(),
            "ret": p.method.ret.rust_name(),
            "generic": p.method.generic,
            "fallible": p.fallible,
        })).collect::<Vec<_>>(),
        "free_functions": free_fns.iter().filter(|f| !f.generic).map(|f| serde_json::json!({
            "name": f.method,
            "params": f.params.iter().map(|t| t.rust_name()).collect::<Vec<_>>(),
            "ret": f.ret.rust_name(),
        })).collect::<Vec<_>>(),
        "skips": skips.iter().map(|s| serde_json::json!({
            "type": s.type_name, "method": s.method, "reason": s.reason,
        })).chain(free_fns.iter().filter(|f| f.generic).map(|f| serde_json::json!({
            "type": "", "method": f.method, "reason": "generic free fn (430 review): no monomorphization available",
        }))).collect::<Vec<_>>(),
    });
    let signatures_json = serde_json::to_string_pretty(&signatures_json).unwrap_or_default();

    let rules_json = serde_json::json!({
        "classifier_version": CLASSIFIER_VERSION,
        "generator": GENERATOR,
        "exceptions": exc,
    });
    let rules_json = serde_json::to_string_pretty(&rules_json).unwrap_or_default();

    (
        fp,
        PackFiles {
            cargo_toml,
            lib_rs,
            manifest_json,
            signatures_json,
            rules_json,
        },
    )
}

fn emit_lib_rs(manifest_json: &str, drop_fns: &str, wrappers: &str) -> String {
    format!(
        "// Generated by shim-metadata (plan-430 C1). DO NOT EDIT BY HAND.\n\
         // 方法 wrapper:裸指针跨 C ABI;对象所有权留在本 cdylib(VM 侧 DepOpaqueObject 持句柄)。\n\
         use std::ffi::{{CStr, CString}};\n\
         use std::os::raw::c_char;\n\
         \n\
         fn __s_in(p: *const c_char) -> String {{\n\
         \x20   if p.is_null() {{\n\
         \x20       String::new()\n\
         \x20   }} else {{\n\
         \x20       unsafe {{ CStr::from_ptr(p) }}.to_string_lossy().into_owned()\n\
         \x20   }}\n\
         }}\n\
         \n\
         fn __s_out(s: String) -> *mut c_char {{\n\
         \x20   CString::new(s).unwrap_or_else(|_| CString::new(\"\").unwrap()).into_raw()\n\
         }}\n\
         \n\
         /// VM 侧拷贝完字符串后回调释放\n\
         #[no_mangle]\n\
         pub extern \"C\" fn auto__free_cstring(p: *mut c_char) {{\n\
         \x20   if !p.is_null() {{\n\
         \x20       drop(unsafe {{ CString::from_raw(p) }});\n\
         \x20   }}\n\
         }}\n\
         \n\
         // ---- unwrap_ok 错误通道(430-F):Err 暂存线程局部,VM 侧读取后转 VMError ----\n\
         thread_local! {{\n\
         \x20   static __LAST_ERR: std::cell::RefCell<Option<CString>> =\n\
         \x20       const {{ std::cell::RefCell::new(None) }};\n\
         }}\n\
         \n\
         fn __set_err(msg: String) {{\n\
         \x20   __LAST_ERR.with(|e| *e.borrow_mut() = Some(CString::new(msg).unwrap_or_default()));\n\
         }}\n\
         \n\
         fn __clear_err() {{\n\
         \x20   __LAST_ERR.with(|e| *e.borrow_mut() = None);\n\
         }}\n\
         \n\
         /// 返回线程局部错误串的指针(所有权留在 cdylib;VM 拷贝后调 auto__clear_error)\n\
         #[no_mangle]\n\
         pub extern \"C\" fn auto__last_error() -> *mut c_char {{\n\
         \x20   __LAST_ERR.with(|e| match &*e.borrow() {{\n\
         \x20       Some(c) => c.as_ptr() as *mut c_char,\n\
         \x20       None => std::ptr::null_mut(),\n\
         \x20   }})\n\
         }}\n\
         \n\
         #[no_mangle]\n\
         pub extern \"C\" fn auto__clear_error() {{\n\
         \x20   __clear_err();\n\
         }}\n\
         \n\
         {drop_fns}\n\
         {wrappers}\n\
         /// shim 包清单(JSON;VM 加载侧解析后注册 dispatch)\n\
         #[no_mangle]\n\
         pub extern \"C\" fn auto__shim_manifest() -> *const c_char {{\n\
         \x20   let s = CString::new(r#\"{manifest}\"#).unwrap();\n\
         \x20   s.into_raw() as *const c_char\n\
         }}\n",
        manifest = manifest_json,
        drop_fns = drop_fns,
        wrappers = wrappers,
    )
}

fn emit_drop_fn(short: &str, full: &str) -> String {
    format!(
        "#[no_mangle]\n\
         pub extern \"C\" fn auto__drop_{short}(h: *mut std::ffi::c_void) {{\n\
         \x20   if !h.is_null() {{\n\
         \x20       drop(unsafe {{ Box::from_raw(h as *mut {full}) }});\n\
         \x20   }}\n\
         }}\n\n",
        short = short,
        full = full,
    )
}

fn arg_char(a: &ArgPlan) -> char {
    match a {
        ArgPlan::BorrowStr | ArgPlan::TakeStr => 's',
        ArgPlan::ScalarI32 => 'i',
        ArgPlan::ScalarI64 | ArgPlan::ScalarUsize => 'l',
        ArgPlan::ScalarF64 => 'f',
        ArgPlan::ScalarBool => 'b',
        ArgPlan::SelfHandle | ArgPlan::OpaqueHandle => 'p',
    }
}

fn ret_char(r: &RetPlan) -> char {
    match r {
        RetPlan::Void => 'v',
        RetPlan::ScalarI32 => 'i',
        RetPlan::ScalarI64 => 'l',
        RetPlan::ScalarF64 => 'f',
        RetPlan::ScalarBool => 'b',
        RetPlan::ScalarStr => 's',
        RetPlan::Opaque(_) | RetPlan::ChainSelf | RetPlan::ChainInPlace => 'p',
    }
}

fn ret_char_of(r: &RetPlan) -> char {
    ret_char(r)
}

fn self_kind_str(s: SelfKind) -> &'static str {
    match s {
        SelfKind::Static => "static",
        SelfKind::Read => "read",
        SelfKind::Write => "write",
        SelfKind::Move => "move",
    }
}

fn ret_type_label(p: &MarshalPlan) -> String {
    match &p.ret {
        // 链式:压回接收者,标签即接收者类型
        RetPlan::ChainSelf | RetPlan::ChainInPlace => p.method.type_name.clone(),
        RetPlan::Opaque(n) => n.clone(),
        _ => String::new(),
    }
}

/// 计算 MarshalPlan 的导出符号全名(与 emit_wrapper 生成的完全一致)。
/// 430 复审修复:供 rustc 检查器剔除环做**精确**匹配——此前 auto-cache 侧用
/// `starts_with("auto_Type_method")` 前缀匹配,`auto_Counter_newest_p_p` 会误伤 `new`。
pub fn plan_export_symbol(p: &MarshalPlan) -> String {
    let m = &p.method;
    let mut chars = String::new();
    if !matches!(m.self_kind, SelfKind::Static) {
        chars.push('p');
    }
    for a in &p.args {
        chars.push(arg_char(a));
    }
    let rc = ret_char(&p.ret);
    // 导出符号:auto_<Type>_<method>_<params>_<ret>,无参时 params 段留空(auto_X_m__r)
    if chars.is_empty() {
        format!("auto_{}_{}__{rc}", m.type_name, m.method)
    } else {
        format!("auto_{}_{}_{chars}_{rc}", m.type_name, m.method)
    }
}

/// 生成单个方法 wrapper 与其 manifest 条目。
fn emit_wrapper(crate_ident: &str, p: &MarshalPlan) -> (MethodEntry, String) {
    let m = &p.method;
    let short = &m.type_name;
    let full = format!("{crate_ident}::{short}");

    // ABI 参数码(接收者前导)
    let mut chars = String::new();
    if !matches!(m.self_kind, SelfKind::Static) {
        chars.push('p');
    }
    for a in &p.args {
        chars.push(arg_char(a));
    }
    let rc = ret_char(&p.ret);
    let export = plan_export_symbol(p);

    // 参数声明与调用实参(源码顺序;接收者占 arg_0)
    let mut idx = 0usize;
    let mut decls: Vec<String> = Vec::new();
    let mut call_args: Vec<String> = Vec::new();
    if !matches!(m.self_kind, SelfKind::Static) {
        decls.push("arg_0: *mut std::ffi::c_void".to_string());
        idx = 1;
    }
    for (i, a) in p.args.iter().enumerate() {
        let name = format!("arg_{idx}");
        let ty_char = arg_char(a);
        decls.push(match ty_char {
            's' => format!("{name}: *const c_char"),
            'i' => format!("{name}: i32"),
            'l' => format!("{name}: i64"),
            'f' => format!("{name}: f64"),
            'b' => format!("{name}: bool"),
            _ => format!("{name}: *mut std::ffi::c_void"),
        });
        call_args.push(match a {
            ArgPlan::BorrowStr => format!("&__s_in({name})"),
            ArgPlan::TakeStr => format!("__s_in({name})"),
            // 数值参数按 i64/i32/f64 宽槽传递,调用处按真实宽度收窄(u8/usize/f32 等)
            ArgPlan::ScalarI32 | ArgPlan::ScalarI64 | ArgPlan::ScalarUsize | ArgPlan::ScalarF64 | ArgPlan::ScalarBool
                if matches!(&m.params[i], Ty::U8 | Ty::U16 | Ty::U32 | Ty::U64 | Ty::Usize | Ty::F32) =>
            {
                let cast = match &m.params[i] {
                    Ty::U8 => " as u8",
                    Ty::U16 => " as u16",
                    Ty::U32 => " as u32",
                    Ty::U64 => " as u64",
                    Ty::Usize => " as usize",
                    Ty::F32 => " as f32",
                    _ => "",
                };
                format!("{name}{cast}")
            }
            ArgPlan::OpaqueHandle => {
                let arg_ty = match &m.params[i] {
                    Ty::Opaque(n) => format!("{crate_ident}::{n}"),
                    _ => "std::ffi::c_void".to_string(),
                };
                format!("unsafe {{ &*({name} as *const {arg_ty}) }}")
            }
            _ => name.clone(),
        });
        idx += 1;
    }

    // 接收者重构 + 调用表达式
    let call_expr = |args: &[String]| -> String { args.join(", ") };
    let invocation = if let Some(f) = &m.field {
        // 合成字段 getter:标量 Copy 直读;String/不透明字段 clone
        // (无 Clone 的类型由 rustc 检查器剔除环兜底)
        let access = match &p.ret {
            RetPlan::ScalarStr => format!("__recv.{f}.clone()"),
            RetPlan::Opaque(_) => format!("__recv.{f}.clone()"),
            _ => format!("__recv.{f}"),
        };
        format!(
            "{{ let __recv: &{full} = unsafe {{ &*(arg_0 as *const {full}) }}; {access} }}"
        )
    } else {
        match m.self_kind {
        SelfKind::Static => {
            format!("<{full}>::{}({})", m.method, call_expr(&call_args))
        }
        SelfKind::Read => format!(
            "{{ let __recv: &{full} = unsafe {{ &*(arg_0 as *const {full}) }}; __recv.{}({}) }}",
            m.method,
            call_expr(&call_args)
        ),
        SelfKind::Write => format!(
            "{{ let __recv: &mut {full} = unsafe {{ &mut *(arg_0 as *mut {full}) }}; __recv.{}({}) }}",
            m.method,
            call_expr(&call_args)
        ),
        SelfKind::Move => format!(
            "{{ let __recv = unsafe {{ Box::from_raw(arg_0 as *mut {full}) }}; (*__recv).{}({}) }}",
            m.method,
            call_expr(&call_args)
        ),
        }
    };

    // 返回值包装(先落 __r 再转,避免块表达式直接接 `as` 的解析歧义)。
    // ChainInPlace:返回的就是接收者引用本身 → 丢弃返回值、把接收者指针原样传回
    // (VM 侧压回原句柄,不新建对象)。
    // fallible(unwrap_ok,430-F):原返回 Result<T,E> → 解 Ok;Err 写入错误通道
    // (auto__last_error)并返回默认值,VM 侧读通道转 VMError。
    let (ret_decl, body_tail) = if matches!(p.ret, RetPlan::ChainInPlace) {
        if p.fallible {
            (
                " -> *mut std::ffi::c_void".to_string(),
                format!(
                    "let __raw = {invocation}; match __raw {{ Ok(_) => {{ __clear_err(); arg_0 }}, Err(e) => {{ __set_err(format!(\"{{e}}\")); std::ptr::null_mut() }} }}"
                ),
            )
        } else {
            (
                " -> *mut std::ffi::c_void".to_string(),
                format!("{invocation}; arg_0"),
            )
        }
    } else if p.fallible {
        // 解 Ok 落 __r,再按返回码走收尾表达式;Err 写错误通道并返回默认值。
        let err_arm =
            |default: &str| format!("Err(e) => {{ __set_err(format!(\"{{e}}\")); {default} }}");
        match rc {
            'v' => (
                "".to_string(),
                format!(
                    "let __raw = {invocation}; match __raw {{ Ok(()) => (), {} }}",
                    err_arm("return;")
                ),
            ),
            'i' => (
                " -> i32".to_string(),
                format!(
                    "let __r = match {invocation} {{ Ok(v) => {{ __clear_err(); v }}, {} }}; __r as i32",
                    err_arm("return 0;")
                ),
            ),
            'l' => (
                " -> i64".to_string(),
                format!(
                    "let __r = match {invocation} {{ Ok(v) => {{ __clear_err(); v }}, {} }}; __r as i64",
                    err_arm("return 0;")
                ),
            ),
            'f' => (
                " -> f64".to_string(),
                format!(
                    "let __r = match {invocation} {{ Ok(v) => {{ __clear_err(); v }}, {} }}; __r as f64",
                    err_arm("return 0.0;")
                ),
            ),
            'b' => (
                " -> bool".to_string(),
                format!(
                    "let __r = match {invocation} {{ Ok(v) => {{ __clear_err(); v }}, {} }}; __r",
                    err_arm("return false;")
                ),
            ),
            's' => (
                " -> *mut c_char".to_string(),
                format!(
                    "let __r = match {invocation} {{ Ok(v) => {{ __clear_err(); v }}, {} }}; __s_out(__r.to_string())",
                    err_arm("return std::ptr::null_mut();")
                ),
            ),
            _ => (
                " -> *mut std::ffi::c_void".to_string(),
                format!(
                    "let __r = match {invocation} {{ Ok(v) => {{ __clear_err(); v }}, {} }}; Box::into_raw(Box::new(__r)) as *mut std::ffi::c_void",
                    err_arm("return std::ptr::null_mut();")
                ),
            ),
        }
    } else {
        match rc {
        'v' => ("".to_string(), format!("{invocation};")),
        'i' => (
            " -> i32".to_string(),
            format!("let __r = {invocation}; __r as i32"),
        ),
        'l' => (
            " -> i64".to_string(),
            format!("let __r = {invocation}; __r as i64"),
        ),
        'f' => (
            " -> f64".to_string(),
            format!("let __r = {invocation}; __r as f64"),
        ),
        'b' => (
            " -> bool".to_string(),
            format!("let __r = {invocation}; __r"),
        ),
        's' => (
            " -> *mut c_char".to_string(),
            format!("let __r = {invocation}; __s_out(__r.to_string())"),
        ),
        _ => (
            " -> *mut std::ffi::c_void".to_string(),
            format!(
                "let __r = {invocation}; Box::into_raw(Box::new(__r)) as *mut std::ffi::c_void"
            ),
        ),
        }
    };

    let src = format!(
        "#[no_mangle]\n\
         pub extern \"C\" fn {export}({decls}){ret_decl} {{\n\
         \x20   {body_tail}\n\
         }}\n\n",
        export = export,
        decls = decls.join(", "),
        ret_decl = ret_decl,
        body_tail = body_tail,
    );

    let entry = MethodEntry {
        type_name: short.clone(),
        full_type: full,
        method: m.method.clone(),
        export,
        self_kind: self_kind_str(m.self_kind).to_string(),
        params: chars,
        ret: rc.to_string(),
        ret_type: ret_type_label(p),
        chain: matches!(p.ret, RetPlan::ChainInPlace),
        fallible: p.fallible,
        drop_export: format!("auto__drop_{short}"),
    };
    (entry, src)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::classify_all_third_party;
    use crate::types::{ArgPlan, SelfKind, ShimMethod, Ty};

    fn demo_classified() -> Classified {
        let methods = vec![
            ShimMethod {
                type_name: "Counter".into(),
                method: "new".into(),
                self_kind: SelfKind::Static,
                params: vec![Ty::StrOwned],
                ret: Ty::OpaqueOwned("Counter".into()),
                generic: false,
                fallible: false,
                field: None,
            },
            ShimMethod {
                type_name: "Counter".into(),
                method: "increment".into(),
                self_kind: SelfKind::Write,
                params: vec![],
                ret: Ty::Void,
                generic: false,
                fallible: false,
                field: None,
            },
            ShimMethod {
                type_name: "Counter".into(),
                method: "value".into(),
                self_kind: SelfKind::Read,
                params: vec![],
                ret: Ty::I64,
                generic: false,
                fallible: false,
                field: None,
            },
            ShimMethod {
                type_name: "Counter".into(),
                method: "maybe".into(),
                self_kind: SelfKind::Read,
                params: vec![],
                ret: Ty::Opaque("Option".into()),
                generic: false,
                fallible: false,
                field: None,
            },
            // unwrap_ok:Result<Counter, String> 已由投影解包为 Counter + fallible
            ShimMethod {
                type_name: "Counter".into(),
                method: "parse".into(),
                self_kind: SelfKind::Static,
                params: vec![Ty::StrOwned],
                ret: Ty::OpaqueOwned("Counter".into()),
                generic: false,
                fallible: true,
                field: None,
            },
        ];
        classify_all_third_party(&methods, &Exceptions::default())
    }

    #[test]
    fn wrapper_exports_and_skips() {
        let c = demo_classified();
        assert_eq!(c.plans.len(), 4, "Option 返回应跳过: {:?}", c.skips);
        assert!(c.skips.iter().any(|s| s.method == "maybe"));

        let (fp, files) = emit_pack(
            &PackMeta {
                crate_name: "my_crate".into(),
                crate_version: "0.1.0".into(),
                toolchain: "rustc 1.90.0-nightly".into(),
            },
            "my_crate = { path = \"../my_crate\" }",
            &c,
            &Exceptions::default(),
            &[],
        );
        assert_eq!(fp.len(), 16);
        assert!(files.lib_rs.contains("auto_Counter_new_s_p"));
        assert!(files.lib_rs.contains("auto_Counter_increment_p_v"));
        assert!(files.lib_rs.contains("auto_Counter_value_p_l"));
        assert!(files.lib_rs.contains("auto__drop_Counter"));
        assert!(files.lib_rs.contains("auto__shim_manifest"));
        assert!(files.lib_rs.contains(
            "let __r = <my_crate::Counter>::new(__s_in(arg_0)); Box::into_raw(Box::new(__r))"
        ));
        // unwrap_ok:Err 写错误通道并返回空指针
        assert!(files
            .lib_rs
            .contains("Err(e) => { __set_err(format!(\"{e}\")); return std::ptr::null_mut(); }"));
        assert!(files.lib_rs.contains("fn auto__last_error()"));
        assert!(files.cargo_toml.contains("[workspace]"));
        let man: ShimManifest = serde_json::from_str(&files.manifest_json).unwrap();
        assert_eq!(man.methods.len(), 4);
        assert_eq!(man.methods[0].params, "s");
        assert_eq!(man.methods[1].params, "p");
        assert_eq!(man.fingerprint, fp);
        let parse_entry = man.methods.iter().find(|e| e.method == "parse").unwrap();
        assert!(parse_entry.fallible);
    }

    #[test]
    fn fingerprint_changes_with_signatures() {
        let c1 = demo_classified();
        let meta = PackMeta {
            crate_name: "my_crate".into(),
            crate_version: "0.1.0".into(),
            toolchain: "t".into(),
        };
        let fp1 = fingerprint(&meta, &c1, &[]);
        let fp2 = fingerprint(
            &PackMeta { toolchain: "other".into(), ..PackMeta { crate_name: "my_crate".into(), crate_version: "0.1.0".into(), toolchain: "t".into() } },
            &c1,
            &[],
        );
        assert_ne!(fp1, fp2, "工具链变化必须改变指纹");

        let mut c2 = demo_classified();
        // 签名行按 (Ty 名, ABI 码) 对齐 zip:两边同时追加才有效
        c2.plans[1].method.params.push(Ty::I64);
        c2.plans[1].args.push(ArgPlan::ScalarI64);
        let fp3 = fingerprint(&meta, &c2, &[]);
        assert_ne!(fp1, fp3, "签名集变化必须改变指纹");

        // 确定性
        assert_eq!(fp1, fingerprint(&meta, &demo_classified(), &[]));
    }

    #[test]
    fn generic_free_fns_are_excluded() {
        // 430 复审修复:泛型自由函数不得以假 'v' 签名进 manifest/指纹
        // (resolve_signature 元数据优先会采信 → 错误类型推断传导 codegen)。
        let c = demo_classified();
        let meta = PackMeta {
            crate_name: "my_crate".into(),
            crate_version: "0.1.0".into(),
            toolchain: "t".into(),
        };
        let generic_fn = ShimMethod {
            type_name: String::new(),
            method: "identity".into(),
            self_kind: SelfKind::Static,
            params: vec![Ty::Generic("T".into())],
            ret: Ty::Generic("T".into()),
            generic: true,
            fallible: false,
            field: None,
        };
        let plain_fn = ShimMethod {
            type_name: String::new(),
            method: "add_one".into(),
            self_kind: SelfKind::Static,
            params: vec![Ty::I64],
            ret: Ty::I64,
            generic: false,
            fallible: false,
            field: None,
        };
        let (_, files) = emit_pack(&meta, "dep", &c, &Exceptions::default(), &[generic_fn.clone(), plain_fn.clone()]);
        let man: ShimManifest = serde_json::from_str(&files.manifest_json).unwrap();
        let names: Vec<&str> = man.functions.iter().map(|f| f.name.as_str()).collect();
        assert_eq!(names, vec!["add_one"], "泛型自由函数必须被过滤");
        // signatures.json 的 skips 里要有留痕
        let sigs: serde_json::Value = serde_json::from_str(&files.signatures_json).unwrap();
        let skip_methods: Vec<&str> = sigs["skips"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|s| s["method"].as_str())
            .collect();
        assert!(skip_methods.contains(&"identity"), "跳过需留痕: {:?}", skip_methods);

        // 指纹同样不受泛型函数影响(其假签名行不进指纹):
        // fp([generic]) == fp([]),且 fp([generic, plain]) == fp([plain])
        let fp_g = fingerprint_parts(&meta, &c.plans, &[generic_fn.clone()]);
        let fp_none = fingerprint_parts(&meta, &c.plans, &[]);
        assert_eq!(fp_g, fp_none, "泛型函数不得影响指纹");
        let fp_gp = fingerprint_parts(&meta, &c.plans, &[generic_fn.clone(), plain_fn.clone()]);
        let fp_p = fingerprint_parts(&meta, &c.plans, &[plain_fn]);
        assert_eq!(fp_gp, fp_p, "泛型函数不得影响指纹");
        assert_ne!(fp_none, fp_p, "非泛型函数仍应影响指纹");
    }

    #[test]
    fn plan_export_symbol_matches_wrapper() {
        // 430 复审修复:剔除环的精确匹配依赖此函数与 wrapper 实际导出名一致
        let c = demo_classified();
        for p in &c.plans {
            let (_, src) = emit_wrapper("my_crate", p);
            let sym = plan_export_symbol(p);
            assert!(
                src.contains(&format!("fn {}", sym)),
                "export symbol {} not found in wrapper source",
                sym
            );
        }
        let new_plan = &c.plans[0];
        assert_eq!(plan_export_symbol(new_plan), "auto_Counter_new_s_p");
    }
}
