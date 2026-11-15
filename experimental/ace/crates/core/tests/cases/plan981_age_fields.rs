//! PLAN-981（SOURCE 批，IF-981-FIELDS）：**AGE 构造清单**与删行反控。
//!
//! 979 V-11 要求"版本化必需清单 + 合法、坏字段及**实际删除必需映射**的反控"。
//! 本文件的判据不是"清单里写了几行"，而是：
//!   1) 一个覆盖全部构造的真实源，用完整清单核对 → 绿；
//!   2) 把产物**削弱**（投影丢掉一类事实）后用完整清单核对 → **必须红**，
//!      且报出的正是被删的那一行；
//!   3) 合法近邻（产物多出别的事实）不受影响。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    age_construct_coverage, collect_facts, compile_age_package, AGE_COMPAT_CONSTRUCTS,
    AGE_CONSTRUCTS,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 覆盖清单里**每一行**构造的真实作者源。
const FULL: &str = "node Widget {\n\
    view { text \"w\" {} }\n\
}\n\
node Counter {\n\
    model { count int = 3 }\n\
    on Inc { count = count + 1 }\n\
    on Life.Tick { count = count + 1 }\n\
    view {\n\
        text \"counter\" {}\n\
        Widget w1 {}\n\
    }\n\
}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    compile_age_package(&[input("main", src)], "app::main", LegacyWidgetAlias::Reject, &caps())
        .unwrap_or_else(|ds| {
            panic!(
                "fixture rejected: {}",
                ds.iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })
}

/// 真实源覆盖清单的**每一行**；完整清单核对必须绿。
#[test]
fn p981_fields_full_source_satisfies_the_construct_table() {
    let out = compile(FULL);
    let facts = collect_facts(&out.description);
    for c in AGE_CONSTRUCTS {
        assert!(
            facts.contains(&c.key),
            "construct `{}` ({}) is required but the product lacks it; present: {facts:?}",
            c.key,
            c.requirement
        );
    }
    age_construct_coverage(&out.description, AGE_CONSTRUCTS)
        .expect("the full construct table is satisfied");
}

/// 删行反控：用完整清单核对**被削弱的产物** → 必须红，且报出被删那一行。
///
/// 削弱方式是真实删除投影事实（清空字段/处理器/引用），不是改清单——
/// 否则只是把分母改小，证明不了覆盖。
#[test]
fn p981_fields_dropping_a_projection_row_is_red() {
    let out = compile(FULL);
    let mut weakened = out.description.clone();
    // 真实地删掉 "on" 这一族事实（连同它的 lifecycle 行）。
    for c in &mut weakened.components {
        c.handler_msgs.clear();
    }
    let missing = age_construct_coverage(&weakened, AGE_CONSTRUCTS)
        .expect_err("dropping the handler projection must be red");
    assert!(missing.contains(&"on".to_string()), "{missing:?}");
    assert!(missing.contains(&"on.lifecycle".to_string()), "{missing:?}");
    assert!(
        !missing.contains(&"node".to_string()),
        "unrelated rows must stay satisfied: {missing:?}"
    );
}

/// 逐行删：清单里任意一行被真正删掉投影，都必须能被抓到（不是只抓 on）。
#[test]
fn p981_fields_every_projection_row_is_individually_guarded() {
    for drop in ["model", "view.text", "view.node-ref"] {
        let out = compile(FULL);
        let mut weakened = out.description.clone();
        for c in &mut weakened.components {
            match drop {
                "model" => {
                    c.fields.clear();
                    c.initialized = false;
                }
                "view.text" => c.texts.clear(),
                "view.node-ref" => c.refs.clear(),
                other => panic!("unknown drop {other}"),
            }
        }
        let missing = age_construct_coverage(&weakened, AGE_CONSTRUCTS)
            .expect_err(&format!("dropping `{drop}` must be detected"));
        assert!(
            missing.iter().any(|m| m == drop || m.starts_with(drop)),
            "the reported missing rows must name `{drop}`: {missing:?}"
        );
    }
}

/// 兼容变体单独验证：旧 `widget` 头在 compat 开启时留下 legacy 事实，
/// 关闭时被具名拒绝（不混进 canonical 构造清单）。
#[test]
fn p981_fields_compat_alias_is_verified_separately() {
    let legacy = FULL.replace("node Counter", "widget Counter");
    // compat 关闭 → 具名拒绝（复用 979 的兼容闸）。
    let ds = compile_age_package(
        &[input("main", &legacy)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
    )
    .expect_err("the legacy head needs the compat config");
    assert_eq!(ds[0].code, "source.age.legacy-widget", "{ds:?}");

    // compat 开启 → 同一 Schema 承认它，且留下 legacy 兼容事实。
    let out = compile_age_package(
        &[input("main", &legacy)],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps(),
    )
    .expect("compat on admits the alias");
    age_construct_coverage(&out.description, AGE_COMPAT_CONSTRUCTS)
        .expect("the legacy head leaves its compat fact");
}

/// 合法近邻：产物多出清单没有的事实不影响判定。
#[test]
fn p981_fields_extra_facts_are_legal_neighbours() {
    let out = compile(FULL);
    // 清单只取一个子集：多出的事实不得让检查变红。
    let subset = &AGE_CONSTRUCTS[0..1];
    age_construct_coverage(&out.description, subset).expect("a subset is satisfied");
}

/// 清单自检：键唯一、且每行都具名到 owner 与规范条款（不是空壳表）。
#[test]
fn p981_fields_table_is_well_formed() {
    let mut keys: Vec<&str> = AGE_CONSTRUCTS.iter().map(|c| c.key).collect();
    let n = keys.len();
    keys.sort_unstable();
    keys.dedup();
    assert_eq!(keys.len(), n, "construct keys must be unique");
    for c in AGE_CONSTRUCTS {
        assert!(!c.spelling.is_empty(), "{} needs a spelling", c.key);
        assert!(!c.fact.is_empty(), "{} needs a typed fact", c.key);
        assert!(!c.consumer.is_empty(), "{} needs a consumer", c.key);
        assert!(
            c.requirement.starts_with("AGE-L") || c.requirement.starts_with("AC-"),
            "{} needs a requirement clause, got {}",
            c.key,
            c.requirement
        );
    }
}
