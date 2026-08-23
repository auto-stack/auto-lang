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

/// 三方 cdylib 路径:Option/Result 不能跨 ABI 装箱语义未定——
/// Result 走 unwrap_ok(解包 + 错误通道),Option v1 跳过。
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
        // 三方路径:Option 返回 v1 跳过(None 语义待例外层)。
        // std 路径:Option 装箱为不透明对象(既有语义)。
        if third_party && matches!(&m.ret, Ty::Opaque(n) | Ty::OpaqueOwned(n) if n == "Option") {
            skips.push(Skip {
                type_name: m.type_name.clone(),
                method: m.method.clone(),
                reason: "option return (None semantics pending)".into(),
            });
            continue;
        }
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
        // wrapper 解 Ok + 错误通道);这里只会再见到 Option(三方跳过,std 装箱)
        // 或残缺的 Result(泛型实参缺失)。
        Ty::Opaque(n) | Ty::OpaqueOwned(n) if n == "Result" && third_party => {
            return Err("result return without projectable Ok type".into())
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
    Ok(MarshalPlan {
        method: m.clone(),
        ret,
        args,
        copy_result,
        fallible: m.fallible && third_party,
    })
}
