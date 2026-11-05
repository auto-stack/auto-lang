//! PLAN-938 source admission counterexamples. No backend is used as a type oracle.
use ace_foreign_contract::manual::{
    AdmittedForeign, HostAuthority, ManualDecl, ManualManifest, MANUAL_SCHEMA, SOURCE_CAP,
};
use ace_foreign_contract::{
    facts::FACTS_FORMAT, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind,
};

fn environment() -> AdmittedForeign {
    let m = ManualManifest {
        schema: MANUAL_SCHEMA.into(),
        review_id: "review/header-938".into(),
        provider: "provider938".into(),
        header_digest: "a".repeat(64),
        physical_table_digest: "b".repeat(64),
        target: ace_foreign_contract::plan::TARGET.into(),
        profile: ace_foreign_contract::plan::PROFILE.into(),
        callconv: "windows-x64".into(),
        direction: "source-import".into(),
        facts: InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: "foreign/c938".into(),
            content_version: "v1".into(),
            source: FactsSource::Typed {
                id: "review/header-938".into(),
            },
            functions: vec![FnFact {
                symbol: "provider_add".into(),
                form: FnForm::Pure {
                    params: vec![LaneKind::I32, LaneKind::I32],
                    ret: LaneKind::I32,
                },
                error_model: None,
                semantic_ref: None,
            }],
            effects: vec!["pure:provider_add".into()],
            capabilities: vec![SOURCE_CAP.into()],
        },
        declarations: vec![ManualDecl {
            context_symbol: None,
            source_name: "foreign_add".into(),
            symbol: "provider_add".into(),
        }],
    };
    admit(m)
}

fn admit(m: ManualManifest) -> AdmittedForeign {
    let h = HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: "c".repeat(64),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: [SOURCE_CAP.into()].into_iter().collect(),
        available_symbols: ["provider_add".into()].into_iter().collect(),
        binding_generation: 1,
    };
    m.admit(&h).unwrap()
}

#[test]
fn foreign_source_calls_are_typed_by_shared_facts_and_have_no_local_body() {
    let env = environment();
    let mut session = crate::SourceQuerySession::new("p938", "m938");
    session.set_foreign(Some(env.clone()));
    let c = crate::compile_via_facts(
        "fn main() int { return foreign_add(20,22) }",
        "source938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut session,
        ace_core::source::SourceProfile::Scalar,
    )
    .unwrap();
    let bundle = c.checked.bundle();
    assert_eq!(bundle.modules[0].bodies.len(), 1);
    assert_eq!(bundle.modules[0].defs.len(), 2);
    assert!(matches!(
        bundle.modules[0].defs[1].kind,
        ace_core::hir::DefKind::Foreign { .. }
    ));
    assert_eq!(session.counters().solver_runs, 1);
    assert!(ace_core::verify::verify_scalar(bundle.clone())
        .unwrap_err()
        .iter()
        .any(|d| d.code == "verify.foreign-authority"));
    let mut forged = bundle.clone();
    if let ace_core::hir::DefKind::Foreign { declaration, .. } = &mut forged.modules[0].defs[1].kind
    {
        declaration.function.symbol = "other".into();
    }
    assert!(ace_core::verify::verify_foreign_scalar(forged, &env).is_err());
}

#[test]
fn foreign_query_signature_is_an_explicit_dependency_and_unaffected_bodies_hit() {
    let src = "fn independent() int { return 7 }\nfn main() int { return foreign_add(20,22) }";
    let env = environment();
    let mut session = crate::SourceQuerySession::new("p938", "m938");
    session.set_foreign(Some(env));
    let compile = |session: &mut crate::SourceQuerySession| {
        crate::compile_via_facts(
            src,
            "source938",
            &[SOURCE_CAP.into()],
            "p938",
            "m938",
            session,
            ace_core::source::SourceProfile::Scalar,
        )
    };
    let first = compile(&mut session).unwrap();
    assert_eq!(first.applied.len(), 2);
    let before = session.counters().solver_runs;
    session.set_allow_solver(false);
    let second = compile(&mut session).unwrap();
    assert!(second.applied.iter().all(|a| a.body_hit));
    assert_eq!(session.counters().solver_runs, before);
    let program =
        ace_core::source::parse_program_with(src, ace_core::source::SourceProfile::Scalar).unwrap();
    let facts = session.infer_body(&program, "main").unwrap().facts;
    assert!(facts.sig_reads.contains(&"foreign_add".to_string()));
}

#[test]
fn only_actual_foreign_readers_solve_again_after_a_contract_change() {
    let src = "fn independent() int { return 7 }\nfn main() int { return foreign_add(20,22) }";
    let original = environment();
    let mut changed = original.manifest().clone();
    changed.header_digest = "d".repeat(64);
    let changed = admit(changed);
    let mut session = crate::SourceQuerySession::new("p938", "m938");
    let compile = |session: &mut crate::SourceQuerySession| {
        crate::compile_via_facts(
            src,
            "source938",
            &[SOURCE_CAP.into()],
            "p938",
            "m938",
            session,
            ace_core::source::SourceProfile::Scalar,
        )
    };
    session.set_foreign(Some(original));
    compile(&mut session).unwrap();
    let before = session.counters().solver_runs;
    session.set_foreign(Some(changed));
    let second = compile(&mut session).unwrap();
    assert!(
        second
            .applied
            .iter()
            .find(|a| a.name == "independent")
            .unwrap()
            .body_hit
    );
    assert!(
        !second
            .applied
            .iter()
            .find(|a| a.name == "main")
            .unwrap()
            .body_hit
    );
    assert_eq!(session.counters().solver_runs, before + 1);
}

#[test]
fn persisted_foreign_hir_is_untrusted_and_wrong_signature_is_rejected() {
    let env = environment();
    let mut session = crate::SourceQuerySession::new("p938", "m938");
    session.set_foreign(Some(env.clone()));
    let c = crate::compile_via_facts(
        "fn main() int { return foreign_add(20,22) }",
        "source938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut session,
        ace_core::source::SourceProfile::Scalar,
    )
    .unwrap();
    let bundle = c.checked.bundle();
    let mut payload = ace_artifacts::codec::encode_bundle(bundle).unwrap();
    assert_eq!(
        &payload[..2],
        &ace_artifacts::codec::FOREIGN_HIR_CODEC_VERSION.to_le_bytes()
    );
    let mut restored = ace_artifacts::codec::decode_bundle(&payload).unwrap();
    assert_eq!(&restored, bundle);
    assert!(ace_core::verify::verify_scalar(restored.clone()).is_err());
    assert!(ace_core::verify::verify_foreign_scalar(restored.clone(), &env).is_ok());
    payload[..2].copy_from_slice(&ace_artifacts::codec::HIR_CODEC_VERSION.to_le_bytes());
    assert!(ace_artifacts::codec::decode_bundle(&payload).is_err());
    if let ace_core::hir::DefKind::Foreign { ty, .. } = restored.modules[0].defs[1].kind {
        if let ace_core::hir::TypeKind::Function(sig) =
            &mut restored.modules[0].types[ty.0 as usize].kind
        {
            sig.params[0] = ace_core::hir::TypeUse::Builtin(ace_core::hir::BuiltinType::Bool);
        }
    }
    assert!(ace_core::verify::verify_foreign_scalar(restored, &env).is_err());
}

#[test]
fn product_entry_consumes_foreign_facts_and_reports_current_source_diagnostics() {
    let mut session = crate::SourceQuerySession::new("p938", "m938");
    session.set_foreign(Some(environment()));
    let good = "fn main() int { return foreign_add(20,22) }";
    let accepted = crate::compile_scalar_product(
        good,
        "source938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut session,
    )
    .unwrap();
    assert_eq!(accepted.lane, crate::CompileLane::Facts);
    assert_eq!(
        accepted.compiled.checked.bundle().modules[0].bodies.len(),
        1
    );
    assert_eq!(session.counters().diagnostic_verdicts, 0);
    let bad = "\n\nfn main() int { return foreign_add(true,22) }";
    let diagnostics = crate::compile_scalar_product(
        bad,
        "source938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut session,
    )
    .err()
    .expect("wrong logical lane must be rejected");
    assert!(!diagnostics.is_empty());
    assert!(diagnostics
        .iter()
        .all(|d| d.code.starts_with("source.foreign.")));
    assert!(diagnostics
        .iter()
        .all(|d| d.span.start > 0 && d.span.end <= bad.len()));
    // No fall-through into a legacy diagnostic oracle that lacks the manifest.
    assert_eq!(session.counters().diagnostic_verdicts, 0);
    let unknown = "fn main() int { return absent_foreign(20,22) }";
    let diagnostics = crate::compile_scalar_product(
        unknown,
        "source938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut session,
    )
    .err()
    .expect("missing declaration must reject");
    assert!(diagnostics
        .iter()
        .any(|d| d.code == "source.foreign.unknown-name" && d.span.start > 0));
}

fn restart(snapshot: &crate::frontend::Checkpoint) -> crate::frontend::Frontend {
    crate::frontend::Frontend::restore(
        &snapshot.root,
        &snapshot.digest,
        &[SOURCE_CAP.into()],
        environment(),
        |digest| {
            snapshot
                .pages
                .values()
                .find(|p| p.digest == digest)
                .map(|p| p.bytes.clone())
        },
    )
    .unwrap()
}

#[test]
fn shared_frontend_restores_without_production_and_immediately_edits_one_source_body() {
    use crate::frontend::{Frontend, Meter};
    let source = "fn independent() int { return 7 }\nfn main() int { return foreign_add(20,22) }";
    let (front, _) =
        Frontend::cold(source, "m938", "p938", &[SOURCE_CAP.into()], environment()).unwrap();
    let saved = front.checkpoint().unwrap();
    let mut restored = restart(&saved);
    let r = restored.meter();
    assert!(r.restore_nodes > 0);
    assert_eq!(
        r,
        Meter {
            restore_pages: 2,
            restore_nodes: r.restore_nodes,
            ..Meter::default()
        }
    );
    assert_eq!(restored.query_counters().solver_runs, 0);
    let untouched = restored.page("independent").unwrap().bytes.clone();
    let identity = restored.body_identity("independent");
    let candidate = restored
        .prepare("main", "fn main() int { return foreign_add(21,22) }")
        .unwrap();
    assert_eq!(
        restored.source(),
        source,
        "prepare cannot install source or semantics"
    );
    assert_eq!(candidate.parts().planned_fns, 1);
    restored.commit(candidate).unwrap();
    assert_eq!(restored.function_meter("independent"), Meter::default());
    assert_eq!(restored.body_identity("independent"), identity);
    assert!(std::sync::Arc::ptr_eq(
        &untouched,
        &restored.page("independent").unwrap().bytes
    ));
    assert_eq!(restored.query_counters().solver_runs, 1);
    assert_eq!(restored.meter().body_parse_produced, 1);
    assert_eq!(restored.meter().hir_produced, 1);
    let saved = restored.checkpoint().unwrap();
    let mut second = restart(&saved);
    let edit = second
        .prepare("independent", "fn independent() int { return 1234567 }")
        .unwrap();
    second.commit(edit).unwrap();
    assert_eq!(second.function_meter("main"), Meter::default());
    // Prefix growth relocates the untouched owner's structural witness without
    // reading/encoding its payload. Repeated restart is still R-only.
    let shifted = second.checkpoint().unwrap();
    let mut third = restart(&shifted);
    let candidate = third
        .prepare("main", "fn main() int { return foreign_add(23,22) }")
        .unwrap();
    third.commit(candidate).unwrap();
    assert_eq!(third.function_meter("independent"), Meter::default());
    assert_eq!(third.query_counters().solver_runs, 1);
}

#[test]
fn shared_frontend_failed_first_edit_stale_candidates_and_bad_required_pages_preserve_root() {
    use crate::frontend::Frontend;
    let source = "fn independent() int { return 7 }\nfn main() int { return foreign_add(20,22) }";
    let (front, _) =
        Frontend::cold(source, "m938", "p938", &[SOURCE_CAP.into()], environment()).unwrap();
    let saved = front.checkpoint().unwrap();
    let mut restored = restart(&saved);
    assert!(restored
        .prepare("main", "fn main() int { return foreign_add(true,22) }")
        .is_err());
    assert_eq!(restored.checkpoint().unwrap().digest, saved.digest);
    let stale = restored
        .prepare("main", "fn main() int { return foreign_add(24,22) }")
        .unwrap();
    let good = restored
        .prepare("main", "fn main() int { return foreign_add(25,22) }")
        .unwrap();
    restored.commit(good).unwrap();
    let current = restored.checkpoint().unwrap().digest;
    assert!(restored.commit(stale).is_err());
    assert_eq!(restored.checkpoint().unwrap().digest, current);
    let restore = |root: &[u8], digest: &str, missing: bool| {
        Frontend::restore(root, digest, &[SOURCE_CAP.into()], environment(), |d| {
            if missing {
                None
            } else {
                saved
                    .pages
                    .values()
                    .find(|p| p.digest == d)
                    .map(|p| p.bytes.clone())
            }
        })
    };
    assert!(restore(&saved.root, &saved.digest, true).is_err());
    assert!(restore(&saved.root, &"0".repeat(64), false).is_err());
    let mut corrupted = saved.root.clone();
    corrupted.push(b' ');
    assert!(restore(&corrupted, &saved.digest, false).is_err());
    assert!(Frontend::restore(&saved.root, &saved.digest, &[], environment(), |_| None).is_err());
    assert!(Frontend::restore(
        &saved.root,
        &saved.digest,
        &[SOURCE_CAP.into()],
        environment(),
        |d| {
            saved.pages.values().find(|p| p.digest == d).map(|p| {
                let mut bytes = p.bytes.as_ref().clone();
                bytes.push(0);
                std::sync::Arc::new(bytes)
            })
        }
    )
    .is_err());
}

#[test]
fn recursive_owner_fact_restore_keeps_one_frozen_self_signature_dependency() {
    let source="fn recurse(n int) int { if n==0 { return 0 }\nreturn foreign_add(n,recurse(n-1)) }\nfn main() int { return recurse(4) }";
    let program =
        ace_core::source::parse_program_with(source, ace_core::source::SourceProfile::Scalar)
            .unwrap();
    let mut query = crate::SourceQuerySession::new("p938", "m938").with_capabilities([SOURCE_CAP]);
    query.set_foreign(Some(environment()));
    let facts = query.infer_body(&program, "recurse").unwrap().facts;
    assert!(facts.sig_reads.iter().any(|n| n == "recurse"));
    query.clear();
    query.set_allow_solver(false);
    query.seed(&program, &facts).unwrap();
    assert!(query.infer_body(&program, "recurse").unwrap().hit);
}

#[test]
fn source_foreign_conflict_and_capability_keep_current_name_spans_before_any_solve() {
    for (source, caps, code) in [
        (
            "\nfn foreign_add(a int,b int) int { return a+b }\nfn main() int { return 42 }",
            vec![SOURCE_CAP.to_string()],
            "source.foreign-conflict",
        ),
        (
            "\nfn main() int { return foreign_add(20,22) }",
            vec![],
            "source.foreign-capability",
        ),
    ] {
        let mut query = crate::SourceQuerySession::new("p938", "m938");
        query.set_foreign(Some(environment()));
        let ds = crate::compile_scalar_product(source, "m938", &caps, "p938", "m938", &mut query)
            .err()
            .expect("must refuse before foreign call");
        assert!(ds
            .iter()
            .any(|d| d.code == code && d.span.start > 0 && d.span.end <= source.len()));
        assert_eq!(query.counters().solver_runs, 0);
    }
}

#[test]
fn source_foreign_names_obey_local_shadowing_instead_of_substituting_a_call() {
    let mut query = crate::SourceQuerySession::new("p938", "m938");
    query.set_foreign(Some(environment()));
    let good = "fn main() int { let foreign_add=42\nreturn foreign_add }";
    crate::compile_scalar_product(
        good,
        "m938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut query,
    )
    .unwrap();
    let facts = query.resident_page("main").unwrap().0;
    assert!(
        facts.sig_reads.is_empty(),
        "reading a local must not become a foreign dependency"
    );
    let bad = "fn main() int { let foreign_add=42\nreturn foreign_add(20,22) }";
    let ds = crate::compile_scalar_product(
        bad,
        "m938",
        &[SOURCE_CAP.into()],
        "p938",
        "m938",
        &mut query,
    )
    .err()
    .expect("scalar local is not callable");
    assert!(ds
        .iter()
        .any(|d| d.code == "source.not-callable" && d.span.start > 0));
}

#[test]
fn malformed_source_preserves_parser_diagnostics_and_counts_the_failed_parse_attempt() {
    let source = "fn main() int { return ";
    let mut observation = crate::frontend::ColdObservation::default();
    let error = crate::frontend::Frontend::cold_observed(
        source,
        "m938",
        "p938",
        &[SOURCE_CAP.into()],
        environment(),
        &mut observation,
    )
    .err()
    .unwrap();
    let crate::frontend::Error::Diagnostics(ds) = error else {
        panic!("parser language error became infrastructure");
    };
    assert!(!ds.is_empty());
    assert!(ds
        .iter()
        .all(|d| d.stage == ace_core::atom_text::Stage::Text && d.span.end <= source.len()));
    assert_eq!(observation.whole_parse_attempts, 1);
    assert_eq!(observation.queries.solver_runs, 0);
}
