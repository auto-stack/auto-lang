//! F-941-09: structural equality does not authorize another logical owner.
use ace_core::source::{
    compile_scalar_source, emit_resolved_parts, parse_program_with, SourceProfile,
};
use ace_core::verify::verify_scalar;
use ace_source_query::{
    compile_via_facts, decode_page, encode_page, restore_bind, restore_verify, FactsError,
    InferBodyFacts, QueryKey, RestoreError, SourceQuerySession,
};
use std::collections::BTreeMap;

const SOURCE: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/query-instances/fact-authority.at"
);
const PACKAGE: &str = "plan941";
const MODULE: &str = "facts.at";
fn parse(src: &str) -> ace_core::source::ast::Program {
    parse_program_with(src, SourceProfile::Scalar).unwrap()
}
fn pages(program: &ace_core::source::ast::Program) -> BTreeMap<String, InferBodyFacts> {
    let mut session = SourceQuerySession::new(PACKAGE, MODULE);
    ["leaf", "twin", "main"]
        .into_iter()
        .map(|name| {
            (
                name.into(),
                session.infer_body(program, name).unwrap().facts,
            )
        })
        .collect()
}
fn foreign(axis: &str) -> (QueryKey, InferBodyFacts) {
    let program = parse(SOURCE);
    let (package, module, name) = match axis {
        "package" => ("another-project", MODULE, "leaf"),
        "module" => (PACKAGE, "another.at", "leaf"),
        "owner" => (PACKAGE, MODULE, "twin"),
        _ => panic!("unknown axis"),
    };
    let mut session = SourceQuerySession::new(package, module);
    let key = session.body_key(&program, name).unwrap();
    let mut page = session.infer_body(&program, name).unwrap().facts;
    page.name = "leaf".into();
    assert_eq!(page.body_version, pages(&program)["leaf"].body_version);
    assert_ne!(page.logical, pages(&program)["leaf"].logical);
    (key, page)
}
fn refused(axis: &str) {
    let program = parse(SOURCE);
    let (key, page) = foreign(axis);
    let (_, decoded) = decode_page(&encode_page(&key, &page).unwrap()).unwrap();
    assert_eq!(
        decoded, page,
        "serialization is not proof of owner applicability"
    );
    for candidate in [page, decoded] {
        let error =
            restore_verify(&program, PACKAGE, MODULE, "leaf", Some(&candidate)).unwrap_err();
        assert!(
            matches!(
                error,
                RestoreError::Facts(FactsError::BodyVersionMismatch { .. })
            ),
            "{axis}: {error:?}"
        );
        let mut substituted = pages(&program);
        substituted.insert("leaf".into(), candidate.clone());
        let error = restore_bind(
            &program,
            &[],
            PACKAGE,
            MODULE,
            &substituted,
            SourceProfile::Scalar,
        )
        .err()
        .expect("foreign page must never yield a resolved program or type supply");
        assert!(
            matches!(
                error,
                RestoreError::Facts(FactsError::BodyVersionMismatch { .. })
            ),
            "{axis}: {error:?}"
        );
        let mut seeded = SourceQuerySession::new(PACKAGE, MODULE).with_solver(false);
        assert!(
            seeded.seed(&program, &candidate).is_err(),
            "existing seed policy remains closed"
        );
        assert_eq!(seeded.counters().solver_runs, 0);
    }
}
#[test]
fn f94109_foreign_package_refused_before_fact_bind() {
    refused("package");
}
#[test]
fn f94109_foreign_module_refused_before_fact_bind() {
    refused("module");
}
#[test]
fn f94109_same_shape_foreign_owner_refused_before_fact_bind() {
    refused("owner");
}

#[test]
fn f94109_own_pages_and_source_position_drift_keep_fresh_hir() {
    let program = parse(SOURCE);
    let original_pages = pages(&program);
    let producer = SourceQuerySession::new(PACKAGE, MODULE);
    let pages: BTreeMap<_, _> = original_pages
        .iter()
        .map(|(name, page)| {
            let key = producer.body_key(&program, name).unwrap();
            let (_, decoded) = decode_page(&encode_page(&key, page).unwrap()).unwrap();
            (name.clone(), decoded)
        })
        .collect();
    for source in [
        SOURCE.to_string(),
        format!("\n\n// source map drift\n{SOURCE}"),
    ] {
        let current = parse(&source);
        for (name, page) in &pages {
            restore_verify(&current, PACKAGE, MODULE, name, Some(page)).unwrap();
        }
        let (resolved, applied, supply) = restore_bind(
            &current,
            &[],
            PACKAGE,
            MODULE,
            &pages,
            SourceProfile::Scalar,
        )
        .unwrap();
        assert_eq!(applied.len(), 3);
        assert_eq!(supply.admitted_bodies(), 3);
        let restored =
            emit_resolved_parts(&resolved, &source, MODULE, SourceProfile::Scalar).unwrap();
        let fresh = compile_scalar_source(&source, MODULE, &[]).unwrap();
        assert_eq!(restored.atom_text, fresh.atom_text);
        assert_eq!(restored.input_hash, fresh.input_hash);
        verify_scalar(restored.bundle).unwrap();
        let mut seeded = SourceQuerySession::new(PACKAGE, MODULE).with_solver(false);
        for page in pages.values() {
            seeded.seed(&current, page).unwrap();
        }
        let product = compile_via_facts(
            &source,
            MODULE,
            &[],
            PACKAGE,
            MODULE,
            &mut seeded,
            SourceProfile::Scalar,
        )
        .unwrap();
        assert!(product.applied.iter().all(|f| f.body_hit));
        assert_eq!(seeded.counters().solver_runs, 0);
        assert_eq!(seeded.counters().body_hits, 3);
        assert_eq!(seeded.counters().diagnostic_verdicts, 0);
    }
}
