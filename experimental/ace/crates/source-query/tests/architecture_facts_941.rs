//! Observe actual fact authority and restore identity admission. A successful
//! observer process is evidence capture, not a claim that every observed lane
//! passed its acceptance criterion; the parent evaluates exported outcomes.
use ace_core::source::{
    compile_scalar_source, emit_resolved_parts, parse_program_with, SourceProfile,
};
use ace_core::verify::verify_scalar;
use ace_source_query::{
    compile_via_facts, restore_bind, restore_verify, BindFactsError, ConsumerError, FactCompiled,
    InferBodyFacts, PortableTy, QueryError, SourceQuerySession,
};
use std::{collections::BTreeMap, io::Write};

const SOURCE: &str = include_str!(
    "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/query-instances/fact-authority.at"
);
const PACKAGE: &str = "plan941";
const MODULE: &str = "facts.at";
fn session() -> SourceQuerySession {
    SourceQuerySession::new(PACKAGE, MODULE)
}
fn compile(s: &mut SourceQuerySession) -> Result<FactCompiled, ConsumerError> {
    compile_via_facts(
        SOURCE,
        MODULE,
        &[],
        PACKAGE,
        MODULE,
        s,
        SourceProfile::Scalar,
    )
}
fn digest(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex().to_string()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn counters(s: &SourceQuerySession) -> String {
    let c = s.counters();
    format!("{{\"solver_runs\":{},\"sig_hits\":{},\"sig_misses\":{},\"body_hits\":{},\"body_misses\":{},\"rejections\":{},\"diagnostic_verdicts\":{}}}",
        c.solver_runs,c.sig_hits,c.sig_misses,c.body_hits,c.body_misses,c.rejections,c.diagnostic_verdicts)
}
fn outcome(result: Result<FactCompiled, ConsumerError>) -> String {
    match result {
        Ok(product) => format!(
            "{{\"phase\":\"verified\",\"hir_blake3\":\"{}\"}}",
            digest(&product.atom_text)
        ),
        Err(ConsumerError::Bind(BindFactsError::Query(QueryError::SolverDisabled {
            query,
            owner,
        }))) => format!(
            "{{\"phase\":\"query-disabled\",\"query_hex\":\"{}\",\"owner_hex\":\"{}\"}}",
            hex(query.as_bytes()),
            hex(owner.as_bytes())
        ),
        Err(ConsumerError::Bind(BindFactsError::Resolve(ds))) => {
            let codes = ds
                .iter()
                .map(|d| {
                    format!(
                        "{{\"stage\":\"{}\",\"code_hex\":\"{}\"}}",
                        d.stage.as_str(),
                        hex(d.code.as_bytes())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!("{{\"phase\":\"bind-refused\",\"diagnostics\":[{codes}]}}")
        }
        Err(other) => panic!("unexpected fact consumer failure: {other:?}"),
    }
}
fn applied(product: &FactCompiled) -> String {
    product.applied.iter().map(|f| format!("{{\"name_hex\":\"{}\",\"sig_hit\":{},\"body_hit\":{},\"body_version\":\"{}\",\"exprs\":{},\"bindings\":{}}}",
        hex(f.name.as_bytes()),f.sig_hit,f.body_hit,f.body_version,f.report.exprs,f.report.bindings)).collect::<Vec<_>>().join(",")
}
fn restored(
    program: &ace_core::source::ast::Program,
    pages: &BTreeMap<String, InferBodyFacts>,
) -> String {
    let verify = restore_verify(program, PACKAGE, MODULE, "leaf", pages.get("leaf"));
    let (phase, bodies, hir) =
        match restore_bind(program, &[], PACKAGE, MODULE, pages, SourceProfile::Scalar) {
            Err(_) => ("restore-refused", 0, String::new()),
            Ok((resolved, _, supply)) => {
                let parts =
                    emit_resolved_parts(&resolved, SOURCE, MODULE, SourceProfile::Scalar).unwrap();
                let hir = parts.atom_text;
                verify_scalar(parts.bundle).unwrap();
                ("verified", supply.admitted_bodies(), hir)
            }
        };
    format!("{{\"verify_admitted\":{},\"phase\":\"{phase}\",\"admitted_bodies\":{bodies},\"hir_blake3\":\"{}\",\"leaf_logical\":\"{}\"}}",verify.is_ok(),if hir.is_empty() { String::new() } else { digest(&hir) },pages["leaf"].logical.as_str())
}

#[test]
fn ordinary_source_fact_authority_and_restore_identity_941() {
    let program = parse_program_with(SOURCE, SourceProfile::Scalar).unwrap();
    let mut disabled = session().with_solver(false);
    let denied = outcome(compile(&mut disabled));
    assert_eq!(disabled.counters().solver_runs, 0);
    let denied_state = counters(&disabled);
    let mut warm = session();
    let cold = compile(&mut warm).unwrap();
    assert_eq!(warm.counters().solver_runs, 3);
    assert_eq!(cold.supply.admitted_bodies(), 3);
    let cold_state = counters(&warm);
    let cold_applied = applied(&cold);
    warm = warm.with_solver(false);
    let hit = compile(&mut warm).unwrap();
    assert_eq!(warm.counters().solver_runs, 3);
    assert!(hit.applied.iter().all(|f| f.body_hit && f.sig_hit));
    let legacy = compile_scalar_source(SOURCE, MODULE, &[]).unwrap();
    assert_eq!(hit.atom_text, cold.atom_text);
    assert_eq!(hit.atom_text, legacy.atom_text);
    assert_eq!(hit.input_hash, legacy.input_hash);
    let hit_state = counters(&warm);
    let hit_applied = applied(&hit);
    let mut producer = session();
    let mut pages = BTreeMap::new();
    for name in ["leaf", "twin", "main"] {
        pages.insert(
            name.to_string(),
            producer.infer_body(&program, name).unwrap().facts,
        );
    }
    let own_restore = restored(&program, &pages);
    let own_logical = pages["leaf"].logical.as_str().to_string();
    // Shape-valid wrong types must actually reach fact-supplied bind and be
    // refused there. No HIR/target is manufactured by this observer.
    let mut poisoned = pages.clone();
    for node in &mut poisoned.get_mut("leaf").unwrap().nodes {
        node.ty = PortableTy::Bool;
    }
    let mut seeded = session().with_solver(false);
    for page in poisoned.values() {
        seeded.seed(&program, page).unwrap();
    }
    let poisoned_result = outcome(compile(&mut seeded));
    assert_eq!(seeded.counters().solver_runs, 0);
    let poison_state = counters(&seeded);
    // Natural same-name/shape pages from foreign package and module domains.
    let mut package = SourceQuerySession::new("another-project", MODULE);
    let foreign_package = package.infer_body(&program, "leaf").unwrap().facts;
    let mut module = SourceQuerySession::new(PACKAGE, "another.at");
    let foreign_module = module.infer_body(&program, "leaf").unwrap().facts;
    // Same-body owner substitution: preserve twin's logical id while changing
    // only the advertised name, so structural shape cannot distinguish it.
    let mut foreign_owner = pages["twin"].clone();
    assert_eq!(foreign_owner.body_version, pages["leaf"].body_version);
    foreign_owner.name = "leaf".into();
    let mut identities = Vec::new();
    for (label, page) in [
        ("package", foreign_package),
        ("module", foreign_module),
        ("owner", foreign_owner),
    ] {
        assert_ne!(page.logical, pages["leaf"].logical);
        let mut guard = session().with_solver(false);
        let seed_refused = guard.seed(&program, &page).is_err();
        assert!(seed_refused);
        let mut substituted = pages.clone();
        substituted.insert("leaf".into(), page);
        identities.push(format!(
            "{{\"axis\":\"{label}\",\"seed_refused\":{seed_refused},\"restore\":{}}}",
            restored(&program, &substituted)
        ));
    }
    if let Some(path) = std::env::var_os("AUTO_PLAN_941_SQ_FACT_OBSERVATIONS") {
        let path = std::path::PathBuf::from(path);
        let scope = std::path::Path::new("D:/autostack/plan-artifacts/plan-941")
            .canonicalize()
            .unwrap();
        assert!(path
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(scope));
        let payload = format!("{{\"schema\":\"ace.m1.sq-fact-authority/1\",\"source_hex\":\"{}\",\"own_logical\":\"{own_logical}\",\"denied\":{{\"outcome\":{denied},\"counters\":{denied_state}}},\"cold\":{{\"hir_blake3\":\"{}\",\"counters\":{cold_state},\"applied\":[{cold_applied}]}},\"hit\":{{\"hir_blake3\":\"{}\",\"counters\":{hit_state},\"applied\":[{hit_applied}]}},\"legacy_hir_blake3\":\"{}\",\"poisoned\":{{\"outcome\":{poisoned_result},\"counters\":{poison_state}}},\"own_restore\":{own_restore},\"foreign\":[{}]}}\n",
            hex(SOURCE.as_bytes()),digest(&cold.atom_text),digest(&hit.atom_text),digest(&legacy.atom_text),identities.join(","));
        assert!(payload.len() <= 16 << 10);
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .unwrap();
        file.write_all(payload.as_bytes()).unwrap();
        file.sync_all().unwrap();
    }
}
