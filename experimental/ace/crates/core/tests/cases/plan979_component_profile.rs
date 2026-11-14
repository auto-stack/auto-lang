//! PLAN-979 T-03：`age-component` profile 的接线面（H0 面不动的证明）。
//!
//! 本步只验证 profile 身份与入口闭合：裁剪程序可编译且与 `age-h0` 的 GrammarId
//! 隔离；所有通用入口（含片段）具名拒绝；age-h0 的既有行为不受影响。

use ace_core::source::cst::{
    parse_fragment_front, parse_front, parse_program_via_cst, profile_grammar, FrontBudget,
    FrontKind, FrontOutcome, FrontRequest,
};
use ace_core::source::SourceProfile;

const SRC: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    view {\n        text \"x\" {}\n    }\n}\n";

#[test]
fn p979_component_profile_has_its_own_trimmed_grammar() {
    let component = profile_grammar(SourceProfile::AgeComponent).expect("component trim compiles");
    let h0 = profile_grammar(SourceProfile::AgeH0).expect("h0 trim compiles");
    // 组件面包含 H0 的规则 + 组件成员规则；H0 面不含组件规则。
    let component_rules: Vec<&str> = component
        .program()
        .rules
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    let h0_rules: Vec<&str> = h0.program().rules.iter().map(|r| r.name.as_str()).collect();
    for rule in [
        "age-node-c-item",
        "age-model-member",
        "age-on-member",
        "age-view-member",
    ] {
        assert!(component_rules.contains(&rule), "component trim has {rule}");
        assert!(!h0_rules.contains(&rule), "h0 trim must not grow {rule}");
    }
    assert!(h0_rules.contains(&"age-node-item"));
    // 两个裁剪面的 identity 不同（同一 grammar 源、不同 profile）。
    assert_ne!(
        format!("{:?}", component.program().identity),
        format!("{:?}", h0.program().identity)
    );
}

#[test]
fn p979_component_profile_is_refused_by_generic_entries() {
    match parse_front(FrontRequest {
        src: SRC,
        profile: SourceProfile::AgeComponent,
        kind: FrontKind::WholeModule,
        budget: FrontBudget::WORK_OWNED_64MIB,
    }) {
        FrontOutcome::Diag(ds) => assert_eq!(ds[0].code, "source.domain-frontend"),
        other => panic!("expected the domain refusal, got {other:?}"),
    }
    let e = parse_program_via_cst(SRC, SourceProfile::AgeComponent)
        .expect_err("bridge refuses the component profile");
    assert_eq!(e.code, "source.domain-frontend");
    match parse_fragment_front(
        SRC,
        SourceProfile::AgeComponent,
        FrontBudget::WORK_OWNED_64MIB,
        &(0..SRC.len()),
        None,
        ace_core::source::ast::Receiver::Free,
    ) {
        FrontOutcome::Diag(ds) => assert_eq!(ds[0].code, "source.domain-frontend"),
        other => panic!("fragment entry refuses the component profile, got {other:?}"),
    }
}
