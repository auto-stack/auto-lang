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
            fallible: false,
            field: None,
        });
    };
    // ---- String(仅静态构造器;方法走引擎 str 原生路径,字符串是标量池值非堆对象) ----
    m("String", "new", SelfKind::Static, &[], Ty::Str);
    m("String", "from", SelfKind::Static, &[Ty::Str], Ty::Str);
    // ---- Vec(元素按不透明处理:AAVM v2 语境下元素是 Auto 值,经由 op/句柄编码) ----
    m("Vec", "new", SelfKind::Static, &[], Ty::Opaque("Vec".into()));
    m("Vec", "push", SelfKind::Write, &[Ty::I64], Ty::Void);
    m("Vec", "pop", SelfKind::Write, &[], Ty::Opaque("Option".into()));
    m("Vec", "len", SelfKind::Read, &[], Ty::U64);
    m("Vec", "is_empty", SelfKind::Read, &[], Ty::Bool);
    m("Vec", "clear", SelfKind::Write, &[], Ty::Void);
    m("Vec", "get", SelfKind::Read, &[Ty::Usize], Ty::Opaque("Option".into()));
    m("Vec", "first", SelfKind::Read, &[], Ty::Opaque("Option".into()));
    m("Vec", "last", SelfKind::Read, &[], Ty::Opaque("Option".into()));
    m("Vec", "insert", SelfKind::Write, &[Ty::Usize, Ty::I64], Ty::Void);
    m("Vec", "remove", SelfKind::Write, &[Ty::Usize], Ty::I64);
    m("Vec", "reverse", SelfKind::Write, &[], Ty::Void);
    m("Vec", "dedup", SelfKind::Write, &[], Ty::Void);
    m("Vec", "clone", SelfKind::Read, &[], Ty::Opaque("Vec".into()));
    // ---- Duration(plan-430 D3:自手写臂迁移;u64 走 i64 宽槽修正遗留有损截断) ----
    // 注意:days/hours/seconds 是 chrono::Duration(同标签异类型),暂留手写臂;
    // as_millis/as_micros/as_nanos 返回 u128(超 i64 槽),同样暂留。
    m("Duration", "from_secs", SelfKind::Static, &[Ty::U64], Ty::Opaque("Duration".into()));
    m("Duration", "from_millis", SelfKind::Static, &[Ty::U64], Ty::Opaque("Duration".into()));
    m("Duration", "from_secs_f64", SelfKind::Static, &[Ty::F64], Ty::Opaque("Duration".into()));
    m("Duration", "as_secs", SelfKind::Read, &[], Ty::U64);
    m("Duration", "as_secs_f64", SelfKind::Read, &[], Ty::F64);
    // ---- Instant / PathBuf(F 轮续:自手写臂迁移) ----
    // PathBuf.join 不迁:std PathBuf::join 经 deref 走 Path::join(返回新对象),
    // 遗留臂语义是 push(原地改)——迁移会静默改行为,保留手写。
    m("Instant", "now", SelfKind::Static, &[], Ty::Opaque("Instant".into()));
    m("Instant", "elapsed", SelfKind::Read, &[], Ty::Opaque("Duration".into()));
    m("PathBuf", "from", SelfKind::Static, &[Ty::StrOwned], Ty::Opaque("PathBuf".into()));
    // HashMap/HashSet:走 Auto 原生 Map 路径(VM auto.hashmap natives + a2r 真 HashMap),v1 不生成 shim
    v
}
