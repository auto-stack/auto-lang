//! PLAN-967 F0 (V-F0-*): ordinary Auto `use.rs` source -> automatic Rust
//! metadata -> binding + generated C-compatible shim -> one rustc cdylib
//! closure -> shared verify -> AV REAL Rust call through the C ABI.
//! The independent oracle is the same loaded provider consumed directly
//! (a second, non-UI consumer of the identical binding/shim/library).
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{build_f0, build_source, f0_authority, f0_provider_source, load};
use ace_av_core::contract::{ExecOutcome, TrapKind, Value};
use ace_av_core::foreign_exec::{ForeignDispatch, ForeignExecError, ForeignStub, ForeignStubTable};
use ace_foreign_contract::manual::SOURCE_CAP;
use ace_foreign_contract::resource::RESOURCE_CAP;

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn success_i32(out: &ExecOutcome) -> Option<i32> {
    match out {
        ExecOutcome::Success {
            value: Value::I32(n),
            ..
        } => Some(*n),
        _ => None,
    }
}

/// Compile a use.rs foreign source through the shared checked frontdoor
/// and lower it to the AV unit (the exact 938/939 shared path; the F0
/// frontdoor only adds the `use.rs` item).
fn compile_unit(
    source: &str,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> ace_av_core::mir::MirModule {
    let mut query = ace_source_query::SourceQuerySession::new("p967", "m967");
    query.set_foreign(Some(environment.clone()));
    let caps = [SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
    let product = ace_source_query::compile_text_record_product(
        source, "f0967", &caps, "p967", "m967", &mut query,
    )
    .expect("foreign text-record source compiles");
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .expect("shared lowering");
    ace_av_core::mir_verify::verify_mir(&mir).expect("shared MIR verify");
    // AV executes the WHOLE module closure (split_units is the native COFF
    // path, not the AV entry).
    mir
}

fn run_on_table(
    source: &str,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
    table: &mut ace_av_core::foreign_exec::ForeignStubTable,
) -> ExecOutcome {
    let unit = compile_unit(source, environment);
    let encoded = ace_av_core::abc::encode_abc(&unit).unwrap().to_flat();
    // PLAN-939 shape: resource-domain foreign modules travel on the OWNED
    // wire family; the foreign identity is the admitted stub coverage.
    assert_eq!(
        &encoded[..4],
        b"AVAD",
        "the owned wire family carries the F0 resource domain"
    );
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &unit.owner, table)
        .expect("admitted foreign verify");
    ace_av_core::av::execute_foreign(
        &loaded,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
        table,
    )
    .expect("per-run authorization")
    .0
}

fn admit(
    build: &ace_abi::rust_provider::RustProviderBuild,
    generation: u64,
) -> ace_foreign_contract::manual::AdmittedForeign {
    let authority = f0_authority(build, generation);
    build
        .manual
        .clone()
        .admit(&authority)
        .expect("F0 manifest admits")
}

#[test]
fn f0_av_calls_real_rust_provider_end_to_end() {
    let root = scope("e2e");
    let build = build_f0(&root).expect("provider cdylib builds");
    let mut provider = load(&build, 1).expect("provider loads");
    let admitted = admit(&build, 1);

    // The source spells the public use.rs frontdoor; nothing hand-written.
    let len = run_on_table(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }",
        &admitted,
        &mut provider.table,
    );
    assert_eq!(success_i32(&len), Some(9), "UTF-8 byte length, NUL intact");
    let empty = run_on_table(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"\") }",
        &admitted,
        &mut provider.table,
    );
    assert_eq!(success_i32(&empty), Some(0));

    // Independent oracle: the SAME loaded provider consumed directly (a
    // non-UI consumer that bypasses AV entirely) — the generated binding
    // and shim serve both consumers identically.
    let decl = admitted.source_declarations_ref()[0].clone();
    let oracle = provider
        .table
        .dispatch_view(&decl, "A\u{4E2D}\u{1F600}\0".as_bytes())
        .expect("oracle dispatch");
    assert_eq!(oracle, ForeignDispatch::PureI32(9));
    // Both legs really dispatched through the C ABI.
    assert_eq!(provider.table.callee_count(), 3);
    assert_eq!(provider.table.rejected_count(), 0);
    // Build identity is recorded end to end (B-01 seed).
    assert_eq!(
        build.rustc_identity.split_whitespace().next(),
        Some("rustc")
    );
}

#[test]
fn f0_bool_and_fallible_lanes_close_with_named_failures() {
    let root = scope("bool-fallible");
    let build = build_f0(&root).expect("build");
    let mut provider = load(&build, 1).expect("load");
    let admitted = admit(&build, 1);

    let ascii = run_on_table(
        "use.rs f0_text\nfn main() bool { return text_is_ascii(\"abc\") }",
        &admitted,
        &mut provider.table,
    );
    assert!(matches!(
        ascii,
        ExecOutcome::Success {
            value: Value::Bool(true),
            ..
        }
    ));

    // Fallible success: the out value is the guest result (65 = 'A').
    let first = run_on_table(
        "use.rs f0_text\nfn main() int { return text_first_byte(\"Abc\") }",
        &admitted,
        &mut provider.table,
    );
    assert_eq!(success_i32(&first), Some(65));

    // Structured failure: the named status is a checked guest failure
    // (Trap), never a normal i32 and never an infra absorption.
    let failed = run_on_table(
        "use.rs f0_text\nfn main() int { return text_first_byte(\"\") }",
        &admitted,
        &mut provider.table,
    );
    assert!(matches!(
        failed,
        ExecOutcome::Trap {
            kind: TrapKind::ForeignStatus,
            ..
        }
    ));
    // The failing call still dispatched (the status came FROM the callee).
    assert_eq!(provider.table.callee_count(), 3);
}

#[test]
fn f0_rejections_happen_before_any_dispatch_and_panic_is_a_host_failure() {
    let root = scope("reject");
    let build = build_f0(&root).expect("build");
    let mut provider = load(&build, 1).expect("load");
    let admitted = admit(&build, 1);
    let unit = compile_unit(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"x\") }",
        &admitted,
    );
    let encoded = ace_av_core::abc::encode_abc(&unit).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();

    // A tampered stub identity (same symbol, different content version)
    // is rejected at admission with zero callee dispatches.
    {
        let mut tampered = ForeignStubTable::new(1);
        let entries: Vec<ForeignStub> = provider
            .table
            .entries()
            .to_vec()
            .into_iter()
            .map(|mut stub| {
                if stub.symbol == "f0_text_byte_len_v1" {
                    stub.content_version = "tampered".into();
                }
                stub
            })
            .collect();
        tampered.set_entries(entries);
        let rejected = ace_av_core::abc_verify::verify_abc_foreign(decoded, &unit.owner, &tampered);
        assert!(rejected
            .unwrap_err()
            .to_string()
            .contains("foreign.unauthorized"));
        assert_eq!(tampered.callee_count(), 0);
    }
    assert_eq!(provider.table.callee_count(), 0);

    // Invalid UTF-8 for a ViewText lane is rejected before the callee.
    let decl = admitted.source_declarations_ref()[0].clone();
    let err = provider
        .table
        .dispatch_view(&decl, &[0xFF, 0xFE])
        .unwrap_err();
    match err {
        ForeignExecError::Reject(r) => assert_eq!(r.code, "foreign.utf8"),
        other => panic!("expected utf8 rejection, got {other:?}"),
    }
    assert_eq!(provider.table.callee_count(), 0);

    // Stale generation: session teardown invalidates the whole binding —
    // admission itself refuses before any execution.
    provider.table.bump_generation();
    let encoded = ace_av_core::abc::encode_abc(&unit).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let stale = ace_av_core::abc_verify::verify_abc_foreign(decoded, &unit.owner, &provider.table);
    assert!(stale
        .unwrap_err()
        .to_string()
        .contains("foreign.unauthorized"));
    assert_eq!(provider.table.callee_count(), 0);

    // A REAL provider panic unwinds exactly to the AV boundary and is
    // reported as an independent host failure — never a guest value.
    let mut fresh = load(&build, 2).expect("reload");
    let readmitted = admit(&build, 2);
    let panicked = run_on_table(
        "use.rs f0_text\nfn main() int { return text_panic_probe(\"boom\") }",
        &readmitted,
        &mut fresh.table,
    );
    match panicked {
        ExecOutcome::Infra(message) => {
            assert!(message.contains("foreign.panic"), "{message}");
        }
        other => panic!("expected a host-failure Infra, got {other:?}"),
    }
    // The panic was contained: the session stays usable.
    let calm = run_on_table(
        "use.rs f0_text\nfn main() int { return text_panic_probe(\"calm\") }",
        &readmitted,
        &mut fresh.table,
    );
    assert_eq!(success_i32(&calm), Some(4));
}

#[test]
fn f0_budget_ledger_and_rounds_stay_bounded() {
    let root = scope("life");
    let build = build_f0(&root).expect("build");
    let mut provider = load(&build, 1).expect("load");
    let admitted = admit(&build, 1);
    let decl = admitted.source_declarations_ref()[0].clone();

    // Exactly at the per-call budget dispatches; one byte over rejects
    // before the callee (the boundary is by BYTES, never strlen).
    let at_limit = vec![b'a'; ace_av_core::foreign_exec::MAX_CALL_BYTES];
    assert!(provider.table.dispatch_view(&decl, &at_limit).is_ok());
    let over = vec![b'a'; ace_av_core::foreign_exec::MAX_CALL_BYTES + 1];
    let err = provider.table.dispatch_view(&decl, &over).unwrap_err();
    assert!(format!("{err:?}").contains("foreign.call-bytes"));
    assert_eq!(provider.table.callee_count(), 1);

    // Twenty create/call/close rounds on one session binding: each run
    // resets the per-run ledger, results stay exact, nothing grows.
    let before = provider.table.callee_count();
    for round in 0..20 {
        let out = run_on_table(
            "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }",
            &admitted,
            &mut provider.table,
        );
        assert_eq!(success_i32(&out), Some(9), "round {round}");
    }
    assert_eq!(provider.table.callee_count(), before + 20);
    assert!(provider.table.run_bytes() <= ace_av_core::foreign_exec::MAX_RUN_BYTES);
}

#[test]
fn f0_provider_identity_rebinds_and_auto_body_edits_do_not() {
    let root = scope("identity");
    let build = build_f0(&root).expect("build");
    let mut provider = load(&build, 1).expect("load");
    let admitted = admit(&build, 1);
    let first = run_on_table(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"abc\") }",
        &admitted,
        &mut provider.table,
    );
    assert_eq!(success_i32(&first), Some(3));

    // An ordinary AUTO body edit reuses the SAME admitted interface: the
    // foreign declaration identities are unchanged, only the caller body
    // differs (no provider rebuild).
    let unit_a = compile_unit(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"abc\") }",
        &admitted,
    );
    let unit_b = compile_unit(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"abcd\") }",
        &admitted,
    );
    let decls = |u: &ace_av_core::mir::MirModule| -> Vec<(String, String)> {
        let mut out = Vec::new();
        for f in &u.funcs {
            for b in &f.blocks {
                for i in &b.instrs {
                    if let ace_av_core::mir::MirInstr::ForeignCall { declaration, .. } = i {
                        out.push((
                            declaration.function.symbol.clone(),
                            declaration.content_version.clone(),
                        ));
                    }
                }
            }
        }
        out
    };
    assert_eq!(decls(&unit_a), decls(&unit_b));
    let second = run_on_table(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"abcd\") }",
        &admitted,
        &mut provider.table,
    );
    assert_eq!(success_i32(&second), Some(4));

    // A PROVIDER body edit changes the provider digest: a fresh admit at
    // a new generation dispatches the NEW provider's behavior, while the
    // interface facts stay shared (annotations unchanged).
    let edited = f0_provider_source().replace(
        "pub fn text_byte_len(s: &str) -> i32 {\n    s.as_bytes().len() as i32\n}",
        "pub fn text_byte_len(s: &str) -> i32 {\n    s.as_bytes().len() as i32 + 1\n}",
    );
    assert_ne!(edited, f0_provider_source());
    let rebuilt = build_source(&root.join("edited"), &edited).expect("rebuild");
    assert_ne!(rebuilt.provider_digest, build.provider_digest);
    assert_eq!(
        rebuilt.manual.facts.functions, build.manual.facts.functions,
        "a provider body edit never rewrites the interface"
    );
    let mut new_provider = load(&rebuilt, 2).expect("reload");
    let readmitted = admit(&rebuilt, 2);
    let third = run_on_table(
        "use.rs f0_text\nfn main() int { return text_byte_len(\"abc\") }",
        &readmitted,
        &mut new_provider.table,
    );
    assert_eq!(
        success_i32(&third),
        Some(4),
        "the rebound provider's +1 body"
    );
}

/// T-01 bounded full-chain probe (longest producer→consumer path): the
/// SAME auto-imported binding/shim drives the NATIVE machine-code leg —
/// ordinary use.rs source → shared checked frontdoor → native foreign
/// lowering → caller COFF → real linked cdylib callee via its import
/// library → real process exit code. The illegal neighbor (tampered
/// identity) refuses admission, and the failure-recovery leg rebinds a
/// rebuilt provider through the same entrance.
#[test]
fn f0_native_machine_code_consumes_the_same_auto_binding() {
    let root = scope("native-probe");
    let build = build_f0(&root).expect("build");
    let import_lib = build.cdylib.with_file_name("f0_text_provider.dll.lib");
    assert!(
        import_lib.is_file(),
        "the cdylib build carries its import library"
    );
    let admitted = admit(&build, 1);
    let caps = [SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];

    let compile_native = |environment: &ace_foreign_contract::manual::AdmittedForeign,
                          source: &str|
     -> Result<ace_av_core::mir::MirModule, String> {
        let mut query = ace_source_query::SourceQuerySession::new("p967", "m967");
        let (product, units) = ace_native::foreign_lower::compile_source_units(
            source,
            "f0967n",
            &caps,
            environment,
            &mut query,
        )
        .map_err(|e| format!("native compile: {e}"))?;
        assert!(product.compiled.checked.bundle().modules[0].bodies.len() == 1);
        assert_eq!(units.len(), 1);
        Ok(units.into_iter().next().unwrap())
    };

    let unit = compile_native(
        &admitted,
        "use.rs f0_text
fn main() int { return text_byte_len(\"A中😀\0\") }",
    )
    .expect("native compile");
    let object =
        ace_native::emit::emit_foreign_unit_coff(&unit, &admitted).expect("foreign caller COFF");
    let wrapper = ace_native::emit::emit_wrapper_coff(&object.symbol).expect("wrapper");

    let run_dir = root.join("native-run");
    std::fs::create_dir_all(&run_dir).unwrap();
    // the loader resolves the import library against the DLL beside the exe
    std::fs::copy(&build.cdylib, run_dir.join("f0_text_provider.dll")).unwrap();
    let linked = ace_native::link_driver::link_only(
        &[ace_native::link_driver::ObjectInput {
            name: "caller".into(),
            bytes: object.bytes.clone(),
        }],
        &wrapper,
        "ac_start",
        &run_dir,
        &[import_lib.clone()],
        "plan967 f0 native probe",
    )
    .expect("link");
    let run = ace_native::link_driver::run_pe(
        &linked.exe_path,
        std::time::Duration::from_secs(20),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
    )
    .expect("run");
    assert_eq!(
        run.exit_code,
        Some(9),
        "the real machine-code callee returned the UTF-8 byte length"
    );

    // Illegal neighbor: a tampered identity never admits (fail-closed at
    // native admission, exactly like 938 — no caller bytes emitted).
    {
        let mut tampered = build.manual.clone();
        tampered.facts.functions[0].symbol = "f0_text_ghost_v1".into();
        let authority = f0_authority(&build, 1);
        // The ghost-symbol manifest is refused at MANIFEST VALIDATION
        // itself (`foreign.symbol`: declarations and facts must cover each
        // other exactly) — the tampered identity can never admit, so no
        // caller bytes and no machine code can exist for it.
        let refused = match tampered.admit(&authority) {
            Ok(_) => panic!("a ghost-symbol manifest must not admit"),
            Err(message) => {
                assert!(message.contains("foreign.symbol"), "{message}");
                true
            }
        };
        assert!(refused, "a ghost-symbol manifest cannot reach machine code");
    }

    // Failure-recovery: a rebuilt provider body (+1) re-admits at a new
    // generation through the SAME entrance and the next link runs the new
    // behavior — recovery without any caller rebuild shortcut.
    let edited = f0_provider_source().replace(
        "pub fn text_byte_len(s: &str) -> i32 {
    s.as_bytes().len() as i32
}",
        "pub fn text_byte_len(s: &str) -> i32 {
    s.as_bytes().len() as i32 + 1
}",
    );
    let rebuilt = build_source(&root.join("edited"), &edited).expect("rebuild");
    let readmitted = admit(&rebuilt, 2);
    let unit2 = compile_native(
        &readmitted,
        "use.rs f0_text
fn main() int { return text_byte_len(\"A中😀\0\") }",
    )
    .expect("native compile 2");
    let object2 = ace_native::emit::emit_foreign_unit_coff(&unit2, &readmitted).expect("caller 2");
    let run_dir2 = root.join("native-run-2");
    std::fs::create_dir_all(&run_dir2).unwrap();
    std::fs::copy(&rebuilt.cdylib, run_dir2.join("f0_text_provider.dll")).unwrap();
    let import_lib2 = rebuilt.cdylib.with_file_name("f0_text_provider.dll.lib");
    let linked2 = ace_native::link_driver::link_only(
        &[ace_native::link_driver::ObjectInput {
            name: "caller2".into(),
            bytes: object2.bytes.clone(),
        }],
        &wrapper,
        "ac_start",
        &run_dir2,
        &[import_lib2],
        "plan967 f0 native probe recovery",
    )
    .expect("link 2");
    let run2 = ace_native::link_driver::run_pe(
        &linked2.exe_path,
        std::time::Duration::from_secs(20),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
    )
    .expect("run 2");
    assert_eq!(
        run2.exit_code,
        Some(10),
        "the rebound provider's +1 body ran for real"
    );
}

/// T-02 permanent lifecycle fixtures (V-01/02/06 shapes + recovery on the
/// AUTO lane; V-03 cancel/late and the full V-04 callback destroy stay on
/// the 939 native lane until T-08's completion ports, V-05 reentry is the
/// foreign_exec unit test). One session: legal identity succeeds →
/// teardown refuses everything later with the dispatch counter frozen →
/// an empty binding is unauthorized (dispatch=0) → a rebuilt provider
/// rebinds through the same entrance.
#[test]
fn f0_session_lifecycle_success_teardown_unauthorized_rebind() {
    use ace_abi::rust_provider::{open_f0_session, open_session_from_build};
    let root = scope("session");
    let mut f0 = open_f0_session(&root).expect("open");

    let loaded_of = |environment: &ace_foreign_contract::manual::AdmittedForeign,
                     table: &ace_av_core::foreign_exec::ForeignStubTable,
                     source: &str|
     -> ace_av_core::abc_verify::LoadedModule {
        let unit = compile_unit(source, environment);
        let encoded = ace_av_core::abc::encode_abc(&unit).unwrap().to_flat();
        let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
        ace_av_core::abc_verify::verify_abc_foreign(decoded, &unit.owner, table)
            .expect("admitted verify")
    };

    // V-02 shape: a legal, fully-admitted identity succeeds for real.
    let out = f0
        .session
        .execute(
            &loaded_of(
                &f0.admitted,
                f0.session.table_shared(),
                "use.rs f0_text
fn main() int { return text_byte_len(\"A中😀\0\") }",
            ),
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .expect("authorized run")
        .0;
    assert_eq!(success_i32(&out), Some(9));
    let dispatched = f0.session.table_shared().callee_count();
    assert_eq!(dispatched, 1);

    // V-01 shape: session teardown. A module admitted BEFORE the
    // teardown is still refused at EXECUTION (per-run authority), a
    // module admitted AFTER it is refused at admission — and the
    // dispatch counter stays FROZEN at its last value either way.
    let admitted_module = loaded_of(
        &f0.admitted,
        f0.session.table_shared(),
        "use.rs f0_text
fn main() int { return text_byte_len(\"x\") }",
    );
    f0.session.teardown();
    let refused = f0.session.execute(
        &admitted_module,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
    );
    assert!(refused.is_err(), "a torn-down session never executes");
    {
        let unit = compile_unit(
            "use.rs f0_text
fn main() int { return text_byte_len(\"x\") }",
            &f0.admitted,
        );
        let encoded = ace_av_core::abc::encode_abc(&unit).unwrap().to_flat();
        let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
        assert!(ace_av_core::abc_verify::verify_abc_foreign(
            decoded,
            &unit.owner,
            f0.session.table_shared()
        )
        .is_err());
    }
    assert_eq!(f0.session.table_shared().callee_count(), dispatched);

    // V-06 shape: an EMPTY binding is unauthorized for every foreign
    // call — the module cannot even ADMIT against it, so no dispatch can
    // exist (callee_count 0), and execution of nothing else changes that.
    {
        let mut empty = ace_av_core::foreign_exec::ForeignSession::new();
        empty.adopt(ace_av_core::foreign_exec::ForeignStubTable::new(1));
        let unit = compile_unit(
            "use.rs f0_text
fn main() int { return text_byte_len(\"x\") }",
            &f0.admitted,
        );
        let encoded = ace_av_core::abc::encode_abc(&unit).unwrap().to_flat();
        let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
        let refusal =
            ace_av_core::abc_verify::verify_abc_foreign(decoded, &unit.owner, empty.table_shared())
                .unwrap_err();
        assert!(refusal.to_string().contains("foreign.unauthorized"));
        assert_eq!(empty.table_shared().callee_count(), 0);
        assert_eq!(empty.table_shared().rejected_count(), 0);
    }

    // Recovery: a rebuilt provider body (+1) rebinds through the SAME
    // entrance at a fresh generation and the next run observes it.
    let edited = f0_provider_source().replace(
        "pub fn text_byte_len(s: &str) -> i32 {
    s.as_bytes().len() as i32
}",
        "pub fn text_byte_len(s: &str) -> i32 {
    s.as_bytes().len() as i32 + 1
}",
    );
    let rebuilt = build_source(&root.join("edited"), &edited).expect("rebuild");
    let mut next = open_session_from_build(&rebuilt, 2).expect("rebind session");
    let out = next
        .session
        .execute(
            &loaded_of(
                &next.admitted,
                next.session.table_shared(),
                "use.rs f0_text
fn main() int { return text_byte_len(\"abc\") }",
            ),
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .expect("rebound run")
        .0;
    assert_eq!(success_i32(&out), Some(4), "the rebound provider's +1 body");
    // Dropping the session drops the loaded library exactly once.
    drop(next);
}
