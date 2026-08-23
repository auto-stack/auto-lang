//! std 方法目录(v1 手工维护,源自 plan-429 B1 盘点报告的方法面枚举;
//! std 的 rustdoc 生成配方打通后此表自动生成,见 430-a1 报告挂账)。
//! 只登记 AAVM/核心自举所需子集;签名为 std 真实签名的人工投影。

use crate::types::*;

pub fn std_methods() -> Vec<ShimMethod> {
    let mut v: Vec<ShimMethod> = Vec::new();
    let mut m = |type_name: &str, method: &str, self_kind: SelfKind, params: &[Ty], ret: Ty| {
        v.push(ShimMethod {
            type_name: type_name.into(),
            method: method.into(),
            self_kind,
            params: params.to_vec(),
            ret,
            generic: false,
        });
    };
    // ---- String ----
    m("String", "new", SelfKind::Static, &[], Ty::Str);
    m("String", "from", SelfKind::Static, &[Ty::Str], Ty::Str);
    m("String", "push_str", SelfKind::Write, &[Ty::Str], Ty::Void);
    m("String", "pop", SelfKind::Write, &[], Ty::Str);
    m("String", "clear", SelfKind::Write, &[], Ty::Void);
    m("String", "len", SelfKind::Read, &[], Ty::U64);
    m("String", "is_empty", SelfKind::Read, &[], Ty::Bool);
    m("String", "clone", SelfKind::Read, &[], Ty::Str);
    m("String", "as_str", SelfKind::Read, &[], Ty::Str);
    m("String", "to_string", SelfKind::Read, &[], Ty::Str);
    m("String", "contains", SelfKind::Read, &[Ty::Str], Ty::Bool);
    m("String", "starts_with", SelfKind::Read, &[Ty::Str], Ty::Bool);
    m("String", "ends_with", SelfKind::Read, &[Ty::Str], Ty::Bool);
    m("String", "find", SelfKind::Read, &[Ty::Str], Ty::Opaque("Option".into()));
    m("String", "replace", SelfKind::Read, &[Ty::Str, Ty::Str], Ty::Str);
    m("String", "trim", SelfKind::Read, &[], Ty::Str);
    m("String", "trim_start", SelfKind::Read, &[], Ty::Str);
    m("String", "trim_end", SelfKind::Read, &[], Ty::Str);
    m("String", "to_lowercase", SelfKind::Read, &[], Ty::Str);
    m("String", "to_uppercase", SelfKind::Read, &[], Ty::Str);
    m("String", "strip_prefix", SelfKind::Read, &[Ty::Str], Ty::Opaque("Option".into()));
    m("String", "strip_suffix", SelfKind::Read, &[Ty::Str], Ty::Opaque("Option".into()));
    m("String", "split", SelfKind::Read, &[Ty::Str], Ty::Opaque("Split".into()));
    m("String", "chars", SelfKind::Read, &[], Ty::Opaque("Chars".into()));
    m("String", "truncate", SelfKind::Write, &[Ty::U64], Ty::Void);
    m("String", "insert", SelfKind::Write, &[Ty::U64, Ty::Str], Ty::Void);
    // ---- Vec(元素按不透明处理:AAVM v2 语境下元素是 Auto 值,经由 op/句柄编码) ----
    m("Vec", "new", SelfKind::Static, &[], Ty::Opaque("Vec".into()));
    m("Vec", "push", SelfKind::Write, &[Ty::I32], Ty::Void);
    m("Vec", "pop", SelfKind::Write, &[], Ty::Opaque("Option".into()));
    m("Vec", "len", SelfKind::Read, &[], Ty::U64);
    m("Vec", "is_empty", SelfKind::Read, &[], Ty::Bool);
    m("Vec", "clear", SelfKind::Write, &[], Ty::Void);
    m("Vec", "get", SelfKind::Read, &[Ty::U64], Ty::Opaque("Option".into()));
    m("Vec", "first", SelfKind::Read, &[], Ty::Opaque("Option".into()));
    m("Vec", "last", SelfKind::Read, &[], Ty::Opaque("Option".into()));
    m("Vec", "insert", SelfKind::Write, &[Ty::U64, Ty::I32], Ty::Void);
    m("Vec", "remove", SelfKind::Write, &[Ty::U64], Ty::Opaque("Option".into()));
    m("Vec", "set", SelfKind::Write, &[Ty::U64, Ty::I32], Ty::Void);
    m("Vec", "reverse", SelfKind::Write, &[], Ty::Void);
    m("Vec", "dedup", SelfKind::Write, &[], Ty::Void);
    m("Vec", "clone", SelfKind::Read, &[], Ty::Opaque("Vec".into()));
    m("Vec", "iter", SelfKind::Read, &[], Ty::Opaque("Iter".into()));
    m("Vec", "to_vec", SelfKind::Read, &[], Ty::Opaque("Vec".into()));
    // ---- HashMap<str, int> 语境(AAVM 符号表) ----
    m("HashMap", "new", SelfKind::Static, &[], Ty::Opaque("HashMap".into()));
    m("HashMap", "insert", SelfKind::Write, &[Ty::Str, Ty::I64], Ty::Opaque("Option".into()));
    m("HashMap", "get", SelfKind::Read, &[Ty::Str], Ty::Opaque("Option".into()));
    m("HashMap", "get_int", SelfKind::Read, &[Ty::Str], Ty::I64);
    m("HashMap", "contains_key", SelfKind::Read, &[Ty::Str], Ty::Bool);
    m("HashMap", "remove", SelfKind::Write, &[Ty::Str], Ty::Opaque("Option".into()));
    m("HashMap", "len", SelfKind::Read, &[], Ty::U64);
    m("HashMap", "is_empty", SelfKind::Read, &[], Ty::Bool);
    m("HashMap", "clear", SelfKind::Write, &[], Ty::Void);
    m("HashMap", "entry", SelfKind::Write, &[Ty::Str], Ty::Opaque("Entry".into()));
    // ---- HashSet<str> ----
    m("HashSet", "new", SelfKind::Static, &[], Ty::Opaque("HashSet".into()));
    m("HashSet", "insert", SelfKind::Write, &[Ty::Str], Ty::Bool);
    m("HashSet", "contains", SelfKind::Read, &[Ty::Str], Ty::Bool);
    m("HashSet", "remove", SelfKind::Write, &[Ty::Str], Ty::Bool);
    m("HashSet", "len", SelfKind::Read, &[], Ty::U64);
    m("HashSet", "is_empty", SelfKind::Read, &[], Ty::Bool);
    m("HashSet", "iter", SelfKind::Read, &[], Ty::Opaque("Iter".into()));
    v
}
