//! PLAN-981（SOURCE 批，T-01）：AGE **多模块包面**（`age-package` 裁剪）的存在性
//! 与身份回归。
//!
//! 本文件的断言是 IF-981-MODULE 的第一层（L1）：包模块图与 AGE 组件必须能在
//! **同一个**裁剪面里被解析，且该面与 H0/组件面是两个不同的 grammar 身份；别的
//! profile 的裁剪不得因此改变规则集。L2（真源解析/绑定/verify/双后端）在
//! `ace_core::source::age_package` 与其后续用例。
use ace_core::source::cst::{profile_from_slug, profile_grammar};
use ace_core::source::SourceProfile;

fn rule_names(p: SourceProfile) -> Vec<String> {
    profile_grammar(p)
        .expect("grammar")
        .program()
        .rules
        .iter()
        .map(|r| r.name.clone())
        .collect()
}

fn grammar_id(p: SourceProfile) -> String {
    format!(
        "{}",
        profile_grammar(p).expect("grammar").program().identity.grammar_id.0
    )
}

/// 包模块面与 AGE 组件面在 `age-package` 上**同时**存在。
#[test]
fn p981_age_package_trim_is_the_package_and_age_union() {
    let pkg = rule_names(SourceProfile::AgePackage);
    for want in [
        // 包模块面（PLAN-960 的 packages-modules 表面）
        "use-item-p",
        "public-item",
        "const-item",
        "var-item",
        "use-path",
        // AGE 组件面（PLAN-979 T-03）
        "age-node-c-item",
        "age-model-member",
        "age-on-member",
        "age-view-member",
        "age-text-item",
    ] {
        assert!(
            pkg.iter().any(|r| r == want),
            "the age-package trim is missing `{want}`"
        );
    }
}

/// 三个 AGE 面是**三个**身份：包面不能靠改 H0/组件面来"顺便"得到。
#[test]
fn p981_age_package_is_its_own_grammar_identity() {
    assert_ne!(
        grammar_id(SourceProfile::AgePackage),
        grammar_id(SourceProfile::AgeComponent)
    );
    assert_ne!(
        grammar_id(SourceProfile::AgePackage),
        grammar_id(SourceProfile::AgeH0)
    );
    // H0 保持最小面：有 node 项、无 model/on 成员（新面不改旧合同）。
    let h0 = rule_names(SourceProfile::AgeH0);
    assert!(h0.iter().any(|r| r == "age-node-item"));
    assert!(!h0.iter().any(|r| r == "age-model-member"));
    assert!(!h0.iter().any(|r| r == "age-on-member"));
}

/// profile 身份的往返：slug ↔ profile ↔ 文档 profile 串。
#[test]
fn p981_age_package_profile_identity_round_trips() {
    assert_eq!(
        SourceProfile::of("auto.source.age-package.draft"),
        Some(SourceProfile::AgePackage)
    );
    assert_eq!(
        profile_from_slug("age-package"),
        Some(SourceProfile::AgePackage)
    );
    assert_eq!(
        SourceProfile::AgePackage.as_str(),
        "auto.source.age-package.draft"
    );
    assert_eq!(
        SourceProfile::AgePackage.hir_domain(),
        ace_core::hir::HirDomain::TextRecord
    );
}
