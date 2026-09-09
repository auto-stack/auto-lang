//! 规律层:6 条签名分类规则(430 §背景) + 例外表应用。
//! 规则默认值之外,例外表只存三类条目:mono(单态化提示)/skip(跳过)/note(句柄语义注记)。

use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 例外表(rules.json 的 exceptions 字段)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Exceptions {
    /// "Vec.get" -> {"K":"String"} 泛型参数的默认实例化
    #[serde(default)]
    pub mono: HashMap<String, HashMap<String, String>>,
    /// 显式跳过("Vec.sort_by": "closure param")
    #[serde(default)]
    pub skip: HashMap<String, String>,
    /// 句柄语义注记("Vec.push": "invalidates iterators"),仅文档用途
    #[serde(default)]
    pub note: HashMap<String, String>,
}

pub struct Classified {
    pub plans: Vec<MarshalPlan>,
    pub skips: Vec<Skip>,
}

/// std 追加段路径(进程内):Option 装箱为不透明对象(E 阶段既有语义),Result 不涉及。
pub fn classify_all(methods: &[ShimMethod], exc: &Exceptions) -> Classified {
    classify_with(methods, exc, false)
}

/// 三方 cdylib 路径:Result 走 unwrap_ok(解包 + 错误通道);Option 解 Some 压值、
/// None 压 null(PLAN-591 T2,仅 s/p 槽,标量槽哨兵歧义仍跳过)。
pub fn classify_all_third_party(methods: &[ShimMethod], exc: &Exceptions) -> Classified {
    classify_with(methods, exc, true)
}

fn classify_with(methods: &[ShimMethod], exc: &Exceptions, third_party: bool) -> Classified {
    let mut plans = Vec::new();
    let mut skips = Vec::new();
    for m in methods {
        let key = format!("{}.{}", m.type_name, m.method);
        if let Some(reason) = exc.skip.get(&key) {
            skips.push(Skip {
                type_name: m.type_name.clone(),
                method: m.method.clone(),
                reason: format!("exception-skip: {reason}"),
            });
            continue;
        }
        // 规则 0:泛型方法默认不可调用,例外表 mono 提示可解(标注 T->具体类型)。
        if m.generic && !exc.mono.contains_key(&key) {
            skips.push(Skip {
                type_name: m.type_name.clone(),
                method: m.method.clone(),
                reason: "generic method without mono hint".into(),
            });
            continue;
        }
        // 规则 0b:闭包/函数指针/impl Trait 参数 → 跳过(v53 投影后为 Opaque("Unknown")
        // 或路径名含 Fn/impl;保守起见字符串判定)。
        if m.params.iter().any(|p| matches!(p, Ty::Opaque(n) if n.contains("Fn") || n == "Unknown")) {
            skips.push(Skip {
                type_name: m.type_name.clone(),
                method: m.method.clone(),
                reason: "closure/unknown param".into(),
            });
            continue;
        }
        // 三方路径 Option 返回不再整体跳过(PLAN-591 T2)——proj_ret 已解包为
        // T + nullable 标记,槽位合法性(仅 s/p)在 classify_one 内守卫。
        // std 路径:Option 装箱为不透明对象(既有语义,不受 T2 影响)。
        match classify_one(m, third_party) {
            Ok(plan) => plans.push(plan),
            Err(reason) => skips.push(Skip {
                type_name: m.type_name.clone(),
                method: m.method.clone(),
                reason,
            }),
        }
    }
    Classified { plans, skips }
}

fn classify_one(m: &ShimMethod, third_party: bool) -> Result<MarshalPlan, String> {
    let key = format!("{}.{}", m.type_name, m.method);
    // 规则 5 默认借用;owned-key 方法(insert 类)转移/克隆(v1 硬编码名单,后续入例外表)
    const OWNED_KEY: &[&str] = &["HashMap.insert", "HashSet.insert"];
    // Option<&T> → .copied()(v1 硬编码,后续入例外表)
    const COPY_RESULT: &[&str] = &["Vec.get", "Vec.first", "Vec.last", "HashMap.get"];
    let owned = OWNED_KEY.contains(&key.as_str());
    // 按值 self:wrapper 会 Box::from_raw 消耗对象本体。chain-in-place 的别名
    // (多个 VM 句柄指向同一对象)与消耗语义冲突——正是"句柄失效语义"例外类
    // (430 §背景例外层第 3 类)。v1 一律跳过,待例外层给失效策略后解锁。
    if matches!(m.self_kind, SelfKind::Move) {
        return Err("by-value self (handle invalidation pending)".into());
    }
    // 参数规划
    let mut args = Vec::new();
    for p in &m.params {
        args.push(match p {
            // rustdoc 投影已区分所有权:按值 String → StrOwned(转移),
            // &str/&String → Str(规则 5:默认借用)。OWNED_KEY 仅兜 std 手编目录。
            Ty::StrOwned => ArgPlan::TakeStr,
            Ty::Str if owned => ArgPlan::TakeStr,
            Ty::Str => ArgPlan::BorrowStr, // 规则 5:默认借用
            Ty::I8 | Ty::I16 | Ty::I32 | Ty::U8 | Ty::U16 | Ty::U32 => ArgPlan::ScalarI32,
            Ty::I64 | Ty::U64 => ArgPlan::ScalarI64,
            Ty::Usize => ArgPlan::ScalarUsize,
            Ty::F32 | Ty::F64 => ArgPlan::ScalarF64,
            Ty::Bool => ArgPlan::ScalarBool,
            // 128 位标量:超 i64 槽,VM 侧无法承载(顺序在前,防被通配臂吃掉)
            Ty::Opaque(n) if n == "i128" || n == "u128" => {
                return Err("128-bit param unsupported".into())
            }
            // Option 参数(PLAN-591 v1:仅收口返回侧;None→null/值→Some 的
            // 参数构造留后续)——显式理由跳过,不再落"owned opaque param"泛化文案
            Ty::Opaque(n) | Ty::OpaqueOwned(n) if n == "Option" => {
                return Err("option param (plan-591 v1: return-side only)".into())
            }
            Ty::Opaque(_) => ArgPlan::OpaqueHandle,
            // 拥有的外来类型参数:VM 侧无法构造该值,v1 跳过
            Ty::OpaqueOwned(_) => return Err("owned opaque param".into()),
            Ty::Generic(_) | Ty::SelfTy => return Err("generic/self param".into()),
            Ty::Void => return Err("void param".into()),
        });
    }
    // 返回值规划(规则 1/3/6)
    let ret = match &m.ret {
        Ty::I8 | Ty::I16 | Ty::I32 | Ty::U8 | Ty::U16 | Ty::U32 => RetPlan::ScalarI32,
        Ty::I64 | Ty::U64 | Ty::Usize => RetPlan::ScalarI64, // 规则 6:宽整型用 i64 槽,不做有损截断
        Ty::F32 | Ty::F64 => RetPlan::ScalarF64,
        Ty::Bool => RetPlan::ScalarBool,
        Ty::Str | Ty::StrOwned => RetPlan::ScalarStr,
        Ty::SelfTy => {
            // 规则 3:链式(返回 Self/&mut Self)——原地修改、压回句柄。
            // 仅对 &mut self 方法成立;&self 返回 Self 是拷贝语义,同样压新句柄,先按 Chain 处理。
            RetPlan::ChainSelf
        }
        // Option/Result 返回:rustdoc 投影已把 Result<T,E> 解包为 T(m.fallible=true,
        // wrapper 解 Ok + 错误通道)、Option<T> 解包为 T(m.nullable=true,PLAN-591 T2);
        // 这里只会再见到残缺的 Option/Result(泛型实参缺失)——显式拒绝,防误装箱。
        Ty::Opaque(n) | Ty::OpaqueOwned(n) if n == "Result" && third_party => {
            return Err("result return without projectable Ok type".into())
        }
        Ty::Opaque(n) | Ty::OpaqueOwned(n) if n == "Option" && third_party => {
            return Err("option return without projectable inner type".into())
        }
        // 借用返回(&T):wrapper 无法装箱一个引用,v1 跳过(&str 已投影为 Str 不在此列)。
        // 例外:builder 链式(&self/&mut self 返回 &Self/&mut Self)→ 原地改、压回原句柄。
        Ty::Opaque(n)
            if n == &format!("&{}", m.type_name)
                && !matches!(m.self_kind, SelfKind::Static) =>
        {
            RetPlan::ChainInPlace
        }
        Ty::Opaque(n) if n.starts_with('&') => return Err("borrowed return".into()),
        Ty::Opaque(n) | Ty::OpaqueOwned(n) => RetPlan::Opaque(n.clone()),
        Ty::Void => RetPlan::Void,
        Ty::Generic(_) => return Err("generic return".into()),
    };
    let copy_result = COPY_RESULT.contains(&key.as_str());
    // PLAN-591 T2:nullable 返回仅 s/p 槽(None→null 指针无歧义);
    // 标量槽 None→哨兵值(0/false/0.0)与合法值冲突,显式跳过留痕。
    let nullable = m.nullable && third_party;
    if nullable {
        match ret {
            RetPlan::ScalarStr | RetPlan::Opaque(_) | RetPlan::ChainSelf | RetPlan::ChainInPlace => {}
            _ => return Err("nullable scalar return (None→sentinel ambiguity; plan-591 v1: s/p slots only)".into()),
        }
    }
    Ok(MarshalPlan {
        method: m.clone(),
        ret,
        args,
        copy_result,
        fallible: m.fallible && third_party,
        nullable,
    })
}

#[cfg(test)]
mod tests {
    // PLAN-591 T2:nullable 槽位守卫与 Option 参数/残缺返回的显式拒绝
    use super::*;

    fn m(method: &str, ret: Ty) -> ShimMethod {
        ShimMethod {
            type_name: "Widget".into(),
            method: method.into(),
            self_kind: SelfKind::Read,
            params: vec![],
            ret,
            generic: false,
            fallible: false,
            nullable: false,
            field: None,
        }
    }

    #[test]
    fn nullable_sp_slots_pass_scalar_rejected() {
        let methods = vec![
            // Option<&str> 投影形态:Str + nullable → 应入计划
            ShimMethod { nullable: true, ret: Ty::Str, ..m("find", Ty::Str) },
            // Option<Widget> 投影形态:OpaqueOwned + nullable → 应入计划
            ShimMethod { nullable: true, ret: Ty::OpaqueOwned("Widget".into()), ..m("find_obj", Ty::OpaqueOwned("Widget".into())) },
            // Option<u16> 投影形态:I32 + nullable → 哨兵歧义,须跳过
            ShimMethod { nullable: true, ret: Ty::U16, ..m("port", Ty::U16) },
            // Option<()> 投影形态:Void + nullable → 须跳过
            ShimMethod { nullable: true, ret: Ty::Void, ..m("check", Ty::Void) },
        ];
        let c = classify_all_third_party(&methods, &Exceptions::default());
        let planned: Vec<&str> = c.plans.iter().map(|p| p.method.method.as_str()).collect();
        assert_eq!(planned, vec!["find", "find_obj"], "s/p 槽 nullable 应入计划");
        assert!(c.plans.iter().all(|p| p.nullable));
        let skipped: Vec<&str> = c.skips.iter().map(|s| s.method.as_str()).collect();
        assert_eq!(skipped, vec!["port", "check"], "标量/void 槽 nullable 应显式跳过");
        assert!(c.skips.iter().all(|s| s.reason.contains("sentinel ambiguity")));
    }

    #[test]
    fn option_param_and_residual_ret_rejected() {
        // Option 参数:显式理由(返回侧先行),不再落泛化 "owned opaque param"
        let opt_param = ShimMethod {
            params: vec![Ty::OpaqueOwned("Option".into())],
            ..m("set_opt", Ty::Void)
        };
        let c = classify_all_third_party(&[opt_param], &Exceptions::default());
        assert!(c.skips[0].reason.contains("option param"));

        // 残缺 Option 返回(泛型实参缺失):拒绝而非误装箱
        let residual = m("opt_bare", Ty::OpaqueOwned("Option".into()));
        let c = classify_all_third_party(&[residual], &Exceptions::default());
        assert!(c.skips[0].reason.contains("without projectable inner type"));
    }

    #[test]
    fn std_path_option_still_boxes() {
        // std 路径(第三参 false):Option 装箱语义不受 T2 影响
        let c = classify_all(&[m("get", Ty::Opaque("Option".into()))], &Exceptions::default());
        assert_eq!(c.plans.len(), 1);
        assert!(!c.plans[0].nullable);
        assert!(matches!(c.plans[0].ret, RetPlan::Opaque(_)));
    }
}
