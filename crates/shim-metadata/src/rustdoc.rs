//! rustdoc JSON v53 → Vec<ShimMethod>。
//! 只投影分类器需要的子集:类型/方法/关联函数、sig.inputs/output、self 可变性、方法级泛型。

use crate::types::{SelfKind, ShimMethod, Ty};
use serde_json::Value;
use std::collections::HashMap;

pub fn parse(doc: &str) -> Result<Vec<ShimMethod>, String> {
    let root: Value = serde_json::from_str(doc).map_err(|e| e.to_string())?;
    let index = root
        .get("index")
        .and_then(|v| v.as_object())
        .ok_or_else(|| "missing index".to_string())?;

    // impl items: id -> (impl 的 for 类型名)。v53 中 impl 的 inner.impl.for/by 名称
    // 取 resolved_path 或 generic;我们只关心固有 impl(paths 里 crate 内的)。
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
        let ret = sig
            .get("output")
            .and_then(|o| match o {
                Value::Null => Some(Ty::Void),
                _ => Some(proj_ty(o)),
            })
            .unwrap_or(Ty::Void);

        // 归属类型:从父 impl 找。v53 impl 的 inner.impl.for_ 是 Type;简化——
        // 在同一 index 里找 span 覆盖或直接用 item.parent_id? v53 无 parent 字段时,
        // 用 paths + impl.for_。这里实现为:遍历 impl 项建立 method_id -> impl_for 映射。
        let _ = &mut self_kind;
        methods.push(RawMethod {
            id: item.get("id").and_then(|v| v.as_u64()).unwrap_or(u64::MAX),
            name: name.to_string(),
            self_kind,
            params,
            ret,
            generic,
        });
    }

    // 建立 impl -> for 类型 与 impl 的 items 列表,把 RawMethod 挂到所属类型。
    let mut impl_for: HashMap<u64, String> = HashMap::new();
    let mut impl_items: HashMap<u64, Vec<u64>> = HashMap::new();
    for item in index.values() {
        let Some(inner) = item.get("inner").and_then(|v| v.as_object()) else {
            continue;
        };
        if let Some(imp) = inner.get("impl").and_then(|v| v.as_object()) {
            let id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(u64::MAX);
            // 固有 impl: for 是 resolved_path 且 trait 字段缺失
            if imp.get("trait").is_none() {
                if let Some(for_ty) = imp.get("for_") {
                    if let Some(n) = path_name(for_ty) {
                        impl_for.insert(id, n);
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
    // 自由函数(type_name = crate 名或空)也可枚举;shim 语境主要用方法,这里只挂 impl 的。
    let mut out = Vec::new();
    for raw in methods {
        if let Some((&imp, _)) = impl_items.iter().find(|(_, items)| items.contains(&raw.id)) {
            if let Some(ty) = impl_for.get(&imp) {
                out.push(ShimMethod {
                    type_name: ty.clone(),
                    method: raw.name,
                    self_kind: raw.self_kind,
                    params: raw.params,
                    ret: raw.ret,
                    generic: raw.generic,
                });
            }
        }
    }
    Ok(out)
}

struct RawMethod {
    id: u64,
    name: String,
    self_kind: SelfKind,
    params: Vec<Ty>,
    ret: Ty,
    generic: bool,
}

fn self_kind_of(ty: &Value) -> SelfKind {
    // borrowed_ref{is_mutable,type:generic Self} / generic Self / owned
    if let Some(br) = ty.get("borrowed_ref") {
        let is_mut = br.get("is_mutable").and_then(|v| v.as_bool()).unwrap_or(false);
        return if is_mut { SelfKind::Write } else { SelfKind::Read };
    }
    if matches!(ty.get("generic").and_then(|v| v.as_str()), Some("Self")) {
        return SelfKind::Write; // 按值 self 视为 Write(move)
    }
    SelfKind::Static
}

fn path_name(ty: &Value) -> Option<String> {
    if let Some(rp) = ty.get("resolved_path") {
        return rp.get("name").and_then(|v| v.as_str()).map(String::from);
    }
    if let Some(s) = ty.get("primitive").and_then(|v| v.as_str()) {
        return Some(title_primitive(s));
    }
    None
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

fn proj_ty(ty: &Value) -> Ty {
    if let Some(p) = ty.get("primitive").and_then(|v| v.as_str()) {
        return match p {
            "i8" | "i16" | "i32" | "isize" | "usize" => Ty::I32,
            "u8" | "u16" | "u32" => Ty::U32,
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
        // &String 参数按 Str 处理(借用)
        let inner = br.get("type").map(proj_ty).unwrap_or(Ty::Void);
        return match inner {
            Ty::Str => Ty::Str,
            other => other,
        };
    }
    if let Some(rp) = ty.get("resolved_path") {
        let name = rp
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown")
            .to_string();
        return match name.as_str() {
            "String" => Ty::Str,
            "Vec" => Ty::Opaque("Vec".into()),
            "Option" | "Result" => Ty::Opaque(name),
            _ => Ty::Opaque(name),
        };
    }
    Ty::Opaque("Unknown".into())
}

