//! rustdoc JSON v53 → Vec<ShimMethod>。
//! 只投影分类器需要的子集:类型/方法/关联函数、sig.inputs/output、self 可变性、方法级泛型。
//!
//! v53 结构注记(2026-08-23 探针实证):
//! - 固有 impl 的归属类型在 `inner.impl.for`(serde 字段名 `for`,**不是** `for_`);
//!   trait impl 的 `trait` 字段非空,全部排除(只取固有 impl);
//! - 按值 self 投影为 `{"generic":"Self"}`(无 borrowed_ref 包裹)→ SelfKind::Move;
//! - 模块级自由函数是 index 顶层 function 项(不被任何 impl 认领)→ free_fns。

use crate::types::{SelfKind, ShimMethod, Ty};
use serde_json::Value;
use std::collections::HashMap;

/// 三方 crate 的解析结果:固有 impl 方法 + 模块级自由函数。
pub struct ParsedCrate {
    /// 固有 impl 的方法/关联函数(type_name = impl 的归属类型短名)
    pub methods: Vec<ShimMethod>,
    /// 模块级自由函数(type_name = "" ;D2 元信息用)
    pub free_fns: Vec<ShimMethod>,
}

/// 兼容入口:只取固有 impl 方法。
pub fn parse(doc: &str) -> Result<Vec<ShimMethod>, String> {
    Ok(parse_all(doc)?.methods)
}

pub fn parse_all(doc: &str) -> Result<ParsedCrate, String> {
    let root: Value = serde_json::from_str(doc).map_err(|e| e.to_string())?;
    let index = root
        .get("index")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "missing index".to_string())?;

    let mut methods = Vec::new();
    for item in index.values() {
        let Some(inner) = item.get("inner").and_then(|v| v.as_object()) else {
            continue;
        };
        let is_fn = inner.contains_key("function") || inner.contains_key("method");
        if !is_fn {
            continue;
        }
        let f = inner
            .get("function")
            .or_else(|| inner.get("method"))
            .and_then(|v| v.as_object())
            .ok_or_else(|| "fn item without function body".to_string())?;
        let Some(name) = item.get("name").and_then(|v| v.as_str()) else {
            continue; // 匿名(impl 块等)
        };
        // 只取公开项
        if item.get("visibility").and_then(|v| v.as_str()) != Some("public") {
            continue;
        }
        let sig = f.get("sig").and_then(|v| v.as_object()).ok_or_else(|| "missing sig".to_string())?;
        let generic = f
            .get("generics")
            .and_then(|g| g.get("params"))
            .and_then(|p| p.as_array())
            .map(|p| !p.is_empty())
            .unwrap_or(false);

        let mut params = Vec::new();
        let mut self_kind = SelfKind::Static;
        if let Some(arr) = sig.get("inputs").and_then(|v| v.as_array()) {
            for pair in arr {
                let pname = pair.get(0).and_then(|v| v.as_str()).unwrap_or("");
                let pty = pair.get(1).ok_or_else(|| "input without type".to_string())?;
                if pname == "self" {
                    self_kind = self_kind_of(pty);
                    continue;
                }
                params.push(proj_ty(pty));
            }
        }
        let (ret, fallible) = sig
            .get("output")
            .and_then(|o| match o {
                Value::Null => Some((Ty::Void, false)),
                _ => Some(proj_ret(o)),
            })
            .unwrap_or((Ty::Void, false));

        methods.push(RawMethod {
            id: item.get("id").and_then(|v| v.as_u64()).unwrap_or(u64::MAX),
            name: name.to_string(),
            self_kind,
            params,
            ret,
            fallible,
            generic,
        });
    }

    // 建立 impl -> for 类型 与 impl 的 items 列表,把 RawMethod 挂到所属类型。
    // v53 serde 命名:固有 impl 的归属类型在 `for` 字段(renamed `for_`)。
    // for_ 带泛型实参的 impl(如 `impl<R: Read> Reader<R>`)标记其方法 generic——
    // wrapper 无法引用裸泛型类型,v1 走"无 mono 提示跳过"。
    let mut impl_for: HashMap<u64, String> = HashMap::new();
    let mut impl_items: HashMap<u64, Vec<u64>> = HashMap::new();
    let mut generic_impl_methods: HashMap<u64, bool> = HashMap::new();
    for item in index.values() {
        let Some(inner) = item.get("inner").and_then(|v| v.as_object()) else {
            continue;
        };
        if let Some(imp) = inner.get("impl").and_then(|v| v.as_object()) {
            let id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
            // 固有 impl:trait 字段缺失或显式为 null(v53 序列化为 "trait": null)
            let trait_absent = imp.get("trait").map_or(true, |t| t.is_null());
            if trait_absent {
                let for_ty = imp.get("for").or_else(|| imp.get("for_"));
                if let Some(for_ty) = for_ty {
                    if let Some(n) = path_name(for_ty) {
                        impl_for.insert(id, n);
                        if for_type_is_generic(for_ty) {
                            if let Some(items) =
                                imp.get("items").and_then(|v| v.as_array())
                            {
                                for it in items {
                                    if let Some(mid) = it.as_u64() {
                                        generic_impl_methods.insert(mid, true);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(items) = imp.get("items").and_then(|v| v.as_array()) {
                for it in items {
                    if let Some(mid) = it.as_u64() {
                        impl_items.entry(id).or_default().push(mid);
                    }
                }
            }
        }
    }
    // 被 impl 认领的 → 方法;未认领的顶层公开函数 → 自由函数(type_name = "")。
    let mut out = Vec::new();
    let mut free = Vec::new();
    for raw in methods {
        let owner = impl_items
            .iter()
            .find(|(_, items)| items.contains(&raw.id))
            .and_then(|(&imp, _)| impl_for.get(&imp).cloned());
        match owner {
            Some(ty) => out.push(ShimMethod {
                type_name: ty,
                method: raw.name,
                self_kind: raw.self_kind,
                params: raw.params,
                ret: raw.ret,
                generic: raw.generic
                    || generic_impl_methods.get(&raw.id).copied().unwrap_or(false),
                fallible: raw.fallible,
                field: None,
            }),
            None => free.push(ShimMethod {
                type_name: String::new(),
                method: raw.name,
                self_kind: SelfKind::Static,
                params: raw.params,
                ret: raw.ret,
                generic: raw.generic,
                fallible: raw.fallible,
                field: None,
            }),
        }
    }

    // ---- F 轮解阻断的合成面 ----
    // ① 公共字段 getter(semver 的 major/minor/patch 是字段非方法);
    // ② Display 类型的 to_string(trait 方法不进固有 impl 枚举)。
    // 与固有方法同名时固有优先(去重);不透明字段走 clone,无 Clone 的
    // 由 rustc 检查器剔除环兜底。
    let mut existing: std::collections::HashSet<String> =
        out.iter().map(|m| format!("{}.{}", m.type_name, m.method)).collect();
    let mut synthetics: Vec<ShimMethod> = Vec::new();
    for item in index.values() {
        let Some(inner) = item.get("inner").and_then(|v| v.as_object()) else {
            continue;
        };
        let vis_public =
            item.get("visibility").and_then(|v| v.as_str()) == Some("public");
        // 公共结构体的公共字段 getter
        if vis_public {
            if let Some(st) = inner.get("struct").and_then(|v| v.as_object()) {
                let Some(struct_name) = item.get("name").and_then(|v| v.as_str()) else {
                    continue;
                };
                let fields = st
                    .get("kind")
                    .and_then(|k| k.get("plain"))
                    .and_then(|p| p.get("fields"))
                    .and_then(|f| f.as_array());
                if let Some(fields) = fields {
                    for fid in fields {
                        let Some(fid) = fid.as_u64() else { continue };
                        let Some(f) = index.get(&fid.to_string()) else { continue };
                        if f.get("visibility").and_then(|v| v.as_str()) != Some("public") {
                            continue;
                        }
                        let Some(fname) = f.get("name").and_then(|v| v.as_str()) else {
                            continue;
                        };
                        let Some(fty) = f
                            .get("inner")
                            .and_then(|v| v.get("struct_field"))
                            // v53:struct_field 的值即类型表示(个别版本或有 type 包裹,兜底)
                            .and_then(|v| v.get("type").or(Some(v)))
                        else {
                            continue;
                        };
                        let proj = proj_ty(fty);
                        let field_ok = proj.is_scalar()
                            || matches!(proj, Ty::Str | Ty::StrOwned | Ty::Opaque(_) | Ty::OpaqueOwned(_));
                        if !field_ok {
                            continue;
                        }
                        let key = format!("{struct_name}.{fname}");
                        if existing.insert(key) {
                            synthetics.push(ShimMethod {
                                type_name: struct_name.to_string(),
                                method: fname.to_string(),
                                self_kind: SelfKind::Read,
                                params: vec![],
                                ret: proj,
                                generic: false,
                                fallible: false,
                                field: Some(fname.to_string()),
                            });
                        }
                    }
                }
            }
        }
        // Display → to_string(blacket 经 ToString 调用)
        if let Some(imp) = inner.get("impl").and_then(|v| v.as_object()) {
            let is_display = imp
                .get("trait")
                .and_then(|t| t.get("path"))
                .and_then(|p| p.as_str())
                .map(|p| p == "Display" || p.ends_with("::Display"))
                .unwrap_or(false);
            if is_display {
                let for_ty = imp.get("for").or_else(|| imp.get("for_"));
                if let Some(ty) = for_ty.and_then(path_name) {
                    let key = format!("{ty}.to_string");
                    if existing.insert(key) {
                        synthetics.push(ShimMethod {
                            type_name: ty,
                            method: "to_string".to_string(),
                            self_kind: SelfKind::Read,
                            params: vec![],
                            ret: Ty::Str,
                            generic: false,
                            fallible: false,
                            field: None,
                        });
                    }
                }
            }
        }
    }
    out.extend(synthetics);

    Ok(ParsedCrate { methods: out, free_fns: free })
}

struct RawMethod {
    id: u64,
    name: String,
    self_kind: SelfKind,
    params: Vec<Ty>,
    ret: Ty,
    fallible: bool,
    generic: bool,
}

fn self_kind_of(ty: &Value) -> SelfKind {
    // borrowed_ref{is_mutable,type:generic Self} / generic Self(按值) / 无 self
    if let Some(br) = ty.get("borrowed_ref") {
        let is_mut = br.get("is_mutable").and_then(|v| v.as_bool()).unwrap_or(false);
        return if is_mut { SelfKind::Write } else { SelfKind::Read };
    }
    if matches!(ty.get("generic").and_then(|v| v.as_str()), Some("Self")) {
        return SelfKind::Move; // 按值 self(消耗接收者)
    }
    SelfKind::Static
}

fn path_name(ty: &Value) -> Option<String> {
    if let Some(rp) = ty.get("resolved_path") {
        // v53 字段是 "path"(旧字段名 "name" 兜底);根重导出的 impl 会带
        // "crate::" 前缀(crate::Uuid),剥掉后取短名(深模块路径 v1 不支持,见报告)
        let n = rp.get("path").or_else(|| rp.get("name"));
        return n
            .and_then(|v| v.as_str())
            .map(|s| s.strip_prefix("crate::").unwrap_or(s).to_string());
    }
    if let Some(s) = ty.get("primitive").and_then(|v| v.as_str()) {
        return Some(title_primitive(s));
    }
    None
}

/// impl 的 for_ 是否带(泛型)实参:`Reader<R>` → true,`Reader` → false。
/// v1 对泛型接收者统一按"无 mono 提示"跳过。
fn for_type_is_generic(for_ty: &Value) -> bool {
    match for_ty.get("resolved_path") {
        Some(rp) => rp.get("args").map_or(false, |a| !a.is_null()),
        None => false,
    }
}

fn title_primitive(p: &str) -> String {
    match p {
        "str" => "String".into(),
        other => upper_first(other),
    }
}

fn upper_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// 参数/嵌套位置的类型投影:拥有的外来路径 → OpaqueOwned(区分借用,见 classify)。
fn proj_ty(ty: &Value) -> Ty {
    if let Some(p) = ty.get("primitive").and_then(|v| v.as_str()) {
        return match p {
            "i8" => Ty::I8,
            "i16" => Ty::I16,
            "i32" => Ty::I32,
            // isize 指针宽:按 i64 宽槽(规则 6 不做有损截断)
            "isize" => Ty::I64,
            "u8" => Ty::U8,
            "u16" => Ty::U16,
            "u32" => Ty::U32,
            "usize" => Ty::Usize,
            "i64" => Ty::I64,
            "u64" => Ty::U64,
            "f32" => Ty::F32,
            "f64" => Ty::F64,
            "bool" => Ty::Bool,
            "str" => Ty::Str,
            _ => Ty::Opaque(p.to_string()),
        };
    }
    if let Some(g) = ty.get("generic").and_then(|v| v.as_str()) {
        return if g == "Self" { Ty::SelfTy } else { Ty::Generic(g.to_string()) };
    }
    if let Some(br) = ty.get("borrowed_ref") {
        // &String/&str 参数按借用 Str 处理;&Foreign → Opaque(传句柄)
        let inner = br.get("type").map(proj_ty).unwrap_or(Ty::Void);
        return match inner {
            Ty::Str | Ty::StrOwned => Ty::Str,
            Ty::OpaqueOwned(n) => Ty::Opaque(n),
            other => other,
        };
    }
    if let Some(rp) = ty.get("resolved_path") {
        let name = rp
            .get("path")
            .or_else(|| rp.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        return match name.as_str() {
            // 按值 String 参数:我们持有 CString 拷贝的所有权,可直接转移
            "String" => Ty::StrOwned,
            _ => Ty::OpaqueOwned(name),
        };
    }
    Ty::Opaque("Unknown".into())
}

/// 返回位置投影:借用的外来返回 → Opaque("&Name")(分类器跳过标记);&str → Str;
/// `Result<T, E>` → 解包为 T 并标记 fallible(430-F unwrap_ok 策略;
/// Option 不解包——None 语义待例外层,v1 仍跳过)。
fn proj_ret(ty: &Value) -> (Ty, bool) {
    if let Some(rp) = ty.get("resolved_path") {
        let name = rp.get("path").and_then(|v| v.as_str()).unwrap_or("");
        if name == "Result" {
            if let Some(inner) = first_generic_arg(rp) {
                let (t, _) = proj_ret(inner);
                return (t, true);
            }
        }
    }
    if let Some(br) = ty.get("borrowed_ref") {
        let inner = br.get("type").map(proj_ty).unwrap_or(Ty::Void);
        return match inner {
            Ty::Str | Ty::StrOwned => (Ty::Str, false),
            Ty::OpaqueOwned(n) => (Ty::Opaque(format!("&{n}")), false),
            other => (other, false),
        };
    }
    (proj_ty(ty), false)
}

/// resolved_path 的第一个泛型实参(Result<T,E> 的 T)。
fn first_generic_arg(rp: &Value) -> Option<&Value> {
    rp.get("args")?
        .get("angle_bracketed")?
        .get("args")?
        .as_array()?
        .first()?
        .get("type")
}
