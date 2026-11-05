//! PLAN-938: actual ordinary source → shared HIR/MIR → machine C call.
#![cfg(windows)]
use ace_foreign_contract::{
    manual::*, FactsSource, FnFact, FnForm, InterfaceFacts, LaneKind, FACTS_FORMAT,
};
use std::path::{Path, PathBuf};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-938/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cside")
}
fn hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn provider(root: &Path, bias: i32) -> (PathBuf, AdmittedForeign) {
    provider_with(root, bias, false, false)
}

fn provider_with(root: &Path, bias: i32, all: bool, bad_bool: bool) -> (PathBuf, AdmittedForeign) {
    let header = std::fs::read(fixture().join("source_foreign938.h")).unwrap();
    let table = std::fs::read(fixture().join("source_foreign938.contract")).unwrap();
    std::fs::write(root.join("source_foreign938.h"), &header).unwrap();
    let source = std::fs::read_to_string(fixture().join("source_foreign938.c"))
        .unwrap()
        .replace("#define BODY_BIAS 0", &format!("#define BODY_BIAS {bias}"));
    let source = if bad_bool {
        source.replace("return value ? 0 : 1;", "return value ? 0 : 2;")
    } else {
        source
    };
    let source_path = root.join("provider.c");
    let object = root.join("provider.obj");
    std::fs::write(&source_path, source).unwrap();
    let deps = ace_abi::ctool::compile_c_with_deps(&source_path, root, &object).unwrap();
    assert!(deps
        .iter()
        .any(|p| p.file_name().is_some_and(|n| n == "source_foreign938.h")));
    let mut m = ManualManifest {
        schema: MANUAL_SCHEMA.into(),
        review_id: "plan938-independent-provider-v1".into(),
        provider: "independent-c938".into(),
        header_digest: hash(&header),
        physical_table_digest: hash(&table),
        target: ace_foreign_contract::plan::TARGET.into(),
        profile: ace_foreign_contract::plan::PROFILE.into(),
        callconv: "windows-x64".into(),
        direction: "source-import".into(),
        facts: InterfaceFacts {
            format: FACTS_FORMAT.into(),
            logical: "independent-c938".into(),
            content_version: "add-v1".into(),
            source: FactsSource::Typed {
                id: "plan938-independent-provider-v1".into(),
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
            source_name: "foreign_add".into(),
            symbol: "provider_add".into(),
            context_symbol: None,
        }],
    };
    if all {
        let cases = [
            (
                "foreign_not",
                "provider_not",
                FnForm::Pure {
                    params: vec![LaneKind::Bool],
                    ret: LaneKind::Bool,
                },
                None,
            ),
            (
                "foreign_div",
                "provider_div",
                FnForm::Fallible {
                    ctx: Some(false),
                    params: vec![LaneKind::I32, LaneKind::I32],
                    outs: vec![LaneKind::I32],
                },
                Some("provider_ctx".to_string()),
            ),
            (
                "foreign_tick",
                "provider_tick",
                FnForm::Pure {
                    params: vec![LaneKind::I32],
                    ret: LaneKind::I32,
                },
                None,
            ),
            (
                "foreign_trace",
                "provider_trace",
                FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::I32,
                },
                None,
            ),
            (
                "foreign_calls",
                "provider_calls",
                FnForm::Pure {
                    params: vec![],
                    ret: LaneKind::I32,
                },
                None,
            ),
            (
                "foreign_mix",
                "provider_mix",
                FnForm::Pure {
                    params: vec![
                        LaneKind::I32,
                        LaneKind::Bool,
                        LaneKind::I32,
                        LaneKind::Bool,
                        LaneKind::I32,
                    ],
                    ret: LaneKind::I32,
                },
                None,
            ),
        ];
        for (name, symbol, form, context_symbol) in cases {
            let error_model = if matches!(form, FnForm::Fallible { .. }) {
                Some(ace_foreign_contract::ErrorModel {
                    kind: "status-i32".into(),
                    codes: vec![
                        (-3, "context".into()),
                        (-2, "overflow".into()),
                        (-1, "zero".into()),
                    ],
                })
            } else {
                None
            };
            m.facts.functions.push(FnFact {
                symbol: symbol.into(),
                form,
                error_model,
                semantic_ref: None,
            });
            m.declarations.push(ManualDecl {
                source_name: name.into(),
                symbol: symbol.into(),
                context_symbol,
            });
        }
        m.facts.effects.push("checked".into());
    }
    let authority = HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: hash(&std::fs::read(&object).unwrap()),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: [SOURCE_CAP.into()].into_iter().collect(),
        available_symbols: m
            .declarations
            .iter()
            .flat_map(|d| std::iter::once(d.symbol.clone()).chain(d.context_symbol.iter().cloned()))
            .collect(),
        binding_generation: bias as u64 + 1,
    };
    (object, m.admit(&authority).unwrap())
}

fn run(
    root: &Path,
    unit: &ace_av_core::mir::MirModule,
    env: &AdmittedForeign,
    provider: &Path,
) -> (i32, Vec<u8>) {
    let object = ace_native::emit::emit_foreign_unit_coff(unit, env).unwrap();
    let wrapper = ace_native::emit::emit_wrapper_coff(&object.symbol).unwrap();
    let objects = [
        ace_native::link_driver::ObjectInput {
            name: "caller".into(),
            bytes: object.bytes.clone(),
        },
        ace_native::link_driver::ObjectInput {
            name: "independent-provider".into(),
            bytes: std::fs::read(provider).unwrap(),
        },
    ];
    let result = ace_native::link_driver::link_and_run(
        &objects,
        &wrapper,
        "ac_start",
        root,
        &[],
        "source foreign 938",
    )
    .unwrap();
    (result.exit_code.unwrap(), object.bytes)
}

#[test]
fn source_foreign_native_calls_real_c_provider_and_body_edit_keeps_caller_identity() {
    let root = scope("native-add");
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir_all(&first).unwrap();
    std::fs::create_dir_all(&second).unwrap();
    let (p0, env0) = provider(&first, 0);
    let (p1, env1) = provider(&second, 1);
    assert_ne!(env0.provider_digest(), env1.provider_digest());
    assert_eq!(env0.source_declarations(), env1.source_declarations());
    let source = "fn main() int { return foreign_add(20,22) }";
    let mut query = ace_source_query::SourceQuerySession::new("p938", "m938");
    let (product, units) = ace_native::foreign_lower::compile_source_units(
        source,
        "source938",
        &[SOURCE_CAP.into()],
        &env0,
        &mut query,
    )
    .unwrap();
    assert_eq!(product.compiled.checked.bundle().modules[0].bodies.len(), 1);
    {
        use ace_a2x_core::target::{Family, TargetProvider};
        let view = ace_a2x_core::SemanticView::from_checked(&product.compiled.checked).unwrap();
        assert_eq!(
            ace_a2x_rust::RustTargetProvider::v1()
                .runtime_source(&view, Family::Scalar)
                .unwrap_err()
                .code,
            "a2r.foreign-unsupported"
        );
    }
    assert_eq!(units.len(), 1);
    assert!(units[0].funcs[0].blocks.iter().any(|b| b
        .instrs
        .iter()
        .any(|i| matches!(i, ace_av_core::mir::MirInstr::ForeignCall { .. }))));
    assert!(
        ace_native::emit::emit_unit_coff(&units[0]).is_err(),
        "ordinary codegen must not bypass current authority"
    );
    let (actual0, caller0) = run(&first, &units[0], &env0, &p0);
    let (actual1, caller1) = run(&second, &units[0], &env1, &p1);
    assert_eq!(actual0, 42);
    assert_eq!(actual1, 43);
    assert_eq!(
        caller0, caller1,
        "provider body bytes do not enter caller emission"
    );
    let encoded = ace_av_core::abc::encode_abc(&units[0]).unwrap().to_flat();
    assert_eq!(&encoded[..4], b"AVFI");
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    ace_av_core::mir_verify::verify_mir(&decoded.module).unwrap();
    assert!(
        ace_av_core::abc_verify::verify_abc(decoded, &units[0].owner)
            .unwrap_err()
            .to_string()
            .contains("foreign.unsupported")
    );
}

#[test]
fn source_foreign_c_consumes_checked_hir_and_calls_independently_compiled_provider() {
    let root = scope("c-add");
    let mut previous_generated = None;
    let mut query = ace_source_query::SourceQuerySession::new("p938", "m938");
    for bias in [0, 1] {
        let dir = root.join(format!("bias-{bias}"));
        std::fs::create_dir_all(&dir).unwrap();
        let (_, environment) = provider(&dir, bias);
        query.set_foreign(Some(environment.clone()));
        let source = "fn main() int { return foreign_add(20,22) }";
        let compiled = ace_source_query::compile_scalar_product(
            source,
            "source938",
            &[SOURCE_CAP.into()],
            "p938",
            "m938",
            &mut query,
        )
        .unwrap();
        let view = ace_a2x_core::SemanticView::from_checked(&compiled.compiled.checked).unwrap();
        assert!(
            ace_a2x_c::lower::lower_fn_c(&view, &view.functions()[0]).is_err(),
            "old C recipe must not claim this profile"
        );
        let fileset = ace_a2x_c::foreign::assemble_library(
            &view,
            &environment,
            &compiled.compiled.input_hash,
        )
        .unwrap();
        assert!(fileset.manifest.contains(ace_a2x_c::foreign::SCHEMA));
        assert!(
            !fileset.files.iter().any(|(name, _)| name == "main.c"),
            "the library assembler must not trim an old exe wrapper"
        );
        let generated = fileset
            .files
            .iter()
            .find(|(name, _)| name == "generated.c")
            .unwrap()
            .1
            .clone();
        assert!(
            generated.contains("provider_add("),
            "the real business fragment owns the call"
        );
        if let Some(previous) = &previous_generated {
            assert_eq!(previous, &generated);
        }
        previous_generated = Some(generated);
        for (name, text) in fileset.files {
            std::fs::write(dir.join(name), text).unwrap();
        }
        std::fs::write(dir.join("manifest.json"), fileset.manifest).unwrap();
        std::fs::write(dir.join("entry.c"),"#include \"a2x_c_runtime.h\"\nextern a2x_result fn_main(void);\nint main(void) { a2x_result result = fn_main(); return result.trap ? 70 : result.value; }\n").unwrap();
        let exe = dir.join("source-c.exe");
        ace_abi::bridge::compile_bridge_crt_with_objects(
            &dir,
            &["generated.c", "foreign_runtime.c", "entry.c"],
            &[dir.join("provider.obj")],
            &exe,
        )
        .unwrap();
        let result = ace_native::link_driver::run_pe(
            &exe,
            std::time::Duration::from_secs(5),
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
        )
        .unwrap();
        assert_eq!(result.exit_code, Some(42 + bias));
    }
}

fn run_crt_source(
    root: &Path,
    source: &str,
    environment: &AdmittedForeign,
    provider: &Path,
    native: bool,
) -> ace_native::link_driver::PeRun {
    std::fs::create_dir_all(root).unwrap();
    let mut query = ace_source_query::SourceQuerySession::new("p938", "m938");
    let mut objects = vec![provider.to_path_buf()];
    let sources: Vec<&str>;
    if native {
        let (_, units) = ace_native::foreign_lower::compile_source_units(
            source,
            "source938",
            &[SOURCE_CAP.into()],
            environment,
            &mut query,
        )
        .unwrap();
        let mut entry = None;
        for (i, unit) in units.iter().enumerate() {
            let object = ace_native::emit::emit_foreign_unit_coff(unit, environment).unwrap();
            if unit.funcs[0].name == "d_main" {
                entry = Some(object.symbol.clone());
            }
            let path = root.join(format!("native-{i}.obj"));
            std::fs::write(&path, object.bytes).unwrap();
            objects.push(path);
        }
        let entry = entry.unwrap();
        let wrapper = root.join("entry-wrapper.obj");
        std::fs::write(
            &wrapper,
            ace_native::emit::emit_wrapper_coff(&entry).unwrap(),
        )
        .unwrap();
        objects.push(wrapper);
        std::fs::write(
            root.join("entry.c"),
            "extern void ac_start(void);\nint main(void) { ac_start(); return 99; }\n",
        )
        .unwrap();
        std::fs::write(
            root.join("foreign_runtime.c"),
            ace_a2x_c::foreign::FAILURE_RUNTIME,
        )
        .unwrap();
        sources = vec!["entry.c", "foreign_runtime.c"];
    } else {
        query.set_foreign(Some(environment.clone()));
        let compiled = ace_source_query::compile_scalar_product(
            source,
            "source938",
            &[SOURCE_CAP.into()],
            "p938",
            "m938",
            &mut query,
        )
        .unwrap();
        let view = ace_a2x_core::SemanticView::from_checked(&compiled.compiled.checked).unwrap();
        let files =
            ace_a2x_c::foreign::assemble_library(&view, environment, &compiled.compiled.input_hash)
                .unwrap();
        for (name, text) in files.files {
            std::fs::write(root.join(name), text).unwrap();
        }
        std::fs::write(root.join("entry.c"),"#include \"a2x_c_runtime.h\"\nextern a2x_result fn_main(void);\nint main(void) { a2x_result value=fn_main(); return value.trap ? 70 : value.value; }\n").unwrap();
        sources = vec!["entry.c", "foreign_runtime.c", "generated.c"];
    }
    let exe = root.join("source.exe");
    ace_abi::bridge::compile_bridge_crt_with_objects(root, &sources, &objects, &exe).unwrap();
    ace_native::link_driver::run_pe(
        &exe,
        std::time::Duration::from_secs(5),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
    )
    .unwrap()
}

#[test]
fn source_foreign_both_targets_preserve_status_bool_order_short_circuit_and_mixed_slots() {
    let root = scope("calls-matrix");
    let provider_dir = root.join("provider");
    std::fs::create_dir_all(&provider_dir).unwrap();
    let (object, environment) = provider_with(&provider_dir, 0, true, false);
    let cases=[
        ("bool","fn main() int { if foreign_not(false) { return 42 }\nreturn 1 }",42,None),
        ("fallible","fn main() int { return foreign_div(84,2) }",42,None),
        ("zero","fn main() int { return foreign_div(84,0) }",70,Some("foreign.status=-1")),
        ("overflow","fn main() int { return foreign_div(-2147483648,-1) }",70,Some("foreign.status=-2")),
        ("order","fn main() int { let sum = foreign_add(foreign_tick(2),foreign_tick(3))\nlet trace=foreign_trace()\nreturn sum*1000+trace*10+foreign_calls() }",5235,None),
        ("and","fn main() int { if false && foreign_not(false) { return 99 }\nreturn foreign_calls() }",1,None),
        ("or","fn main() int { if true || foreign_not(false) { return foreign_calls() }\nreturn 99 }",1,None),
        ("mixed","fn main() int { return foreign_mix(1,true,2,false,3) }",30211,None),
        ("named-order","fn main() int { return foreign_add(arg1:foreign_tick(3),arg0:foreign_tick(2))+foreign_trace()*10 }",325,None),
    ];
    for native in [true, false] {
        for (name, source, expected, status) in cases {
            let run = run_crt_source(
                &root.join(format!("{native}-{name}")),
                source,
                &environment,
                &object,
                native,
            );
            assert_eq!(
                run.exit_code,
                Some(expected),
                "{native}/{name}: {}",
                String::from_utf8_lossy(&run.stderr)
            );
            if let Some(status) = status {
                assert!(
                    String::from_utf8_lossy(&run.stderr).contains(status),
                    "{native}/{name}: raw cause lost"
                );
            }
        }
    }
    let bad = root.join("bad-bool-provider");
    std::fs::create_dir_all(&bad).unwrap();
    let (object, environment) = provider_with(&bad, 0, true, true);
    for native in [true, false] {
        let run = run_crt_source(
            &root.join(format!("{native}-bad-bool")),
            "fn main() int { if foreign_not(false) { return 42 }\nreturn 1 }",
            &environment,
            &object,
            native,
        );
        assert_eq!(run.exit_code, Some(70));
        assert!(String::from_utf8_lossy(&run.stderr).contains("foreign.status=-2147483647"));
    }
}

#[test]
fn source_foreign_selected_targets_restore_then_immediate_edit_and_failed_first_edit() {
    use ace_abi::source_foreign::{Meter, Session, Snapshot, Target};
    use std::sync::Arc;
    let root = scope("source-restart");
    let p0 = root.join("p0");
    std::fs::create_dir_all(&p0).unwrap();
    let (object, environment) = provider(&p0, 0);
    let bytes = Arc::new(std::fs::read(&object).unwrap());
    let source="fn independent() int { return 7 }\nfn helper() int { return foreign_add(20,22) }\nfn main() int { return helper() }";
    for target in [Target::Native, Target::C] {
        let label = format!("{target:?}");
        let cold = Session::cold(
            source,
            "source938",
            "p938",
            &[SOURCE_CAP.into()],
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        let saved = cold.snapshot().unwrap();
        let restart = |saved: &Snapshot| {
            Session::restore(
                &saved.root,
                &saved.digest,
                target,
                &[SOURCE_CAP.into()],
                environment.clone(),
                |id| saved.pages.get(id).cloned(),
            )
            .unwrap()
        };
        drop(cold);
        let mut restored = restart(&saved);
        let rm = restored.meter();
        assert!(rm.restore_pages > 0);
        assert_eq!(
            rm,
            Meter {
                restore_pages: rm.restore_pages,
                ..Meter::default()
            }
        );
        assert_eq!(restored.frontend().query_counters().solver_runs, 0);
        let untouched = restored.artifact_bytes("main").unwrap().clone();
        let old_body = restored.frontend().body_identity("main");
        assert!(restored
            .prepare("helper", "fn helper() int { return foreign_add(true,22) }")
            .is_err());
        assert_eq!(restored.snapshot().unwrap().digest, saved.digest);
        assert_eq!(
            restored
                .run(&root.join(format!("{label}-old-after-failure")))
                .unwrap()
                .exit_code,
            Some(42)
        );
        let candidate = restored
            .prepare("helper", "fn helper() int { return foreign_add(21,22) }")
            .unwrap();
        restored.commit(candidate).unwrap();
        assert_eq!(restored.function_meter("main"), Meter::default());
        assert_eq!(restored.function_meter("independent"), Meter::default());
        assert_eq!(restored.frontend().body_identity("main"), old_body);
        assert!(Arc::ptr_eq(
            &untouched,
            restored.artifact_bytes("main").unwrap()
        ));
        assert_eq!(
            restored
                .run(&root.join(format!("{label}-first-repair")))
                .unwrap()
                .exit_code,
            Some(43)
        );
        let fresh = Session::cold(
            restored.frontend().source(),
            "source938",
            "p938",
            &[SOURCE_CAP.into()],
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        // Byte-equivalent source and target business artifacts, independently
        // cold-produced after the first failure + repair.
        assert_eq!(
            fresh.artifact_bytes("helper").unwrap(),
            restored.artifact_bytes("helper").unwrap()
        );
        let saved = restored.snapshot().unwrap();
        drop(restored);
        let mut second = restart(&saved);
        let helper = second.artifact_bytes("helper").unwrap().clone();
        let edit = second
            .prepare("independent", "fn independent() int { return 1234567 }")
            .unwrap();
        second.commit(edit).unwrap();
        assert!(Arc::ptr_eq(
            &helper,
            second.artifact_bytes("helper").unwrap()
        ));
        assert_eq!(second.function_meter("helper"), Meter::default());
        let shifted = second.snapshot().unwrap();
        drop(second);
        let mut third = restart(&shifted);
        let edit = third
            .prepare("helper", "fn helper() int { return foreign_add(22,22) }")
            .unwrap();
        third.commit(edit).unwrap();
        assert_eq!(
            third
                .run(&root.join(format!("{label}-second-restart")))
                .unwrap()
                .exit_code,
            Some(44)
        );
        let main = third.artifact_bytes("main").unwrap().clone();
        let counts = third.meter();
        let frontend = third.frontend().meter();
        let solve = third.frontend().query_counters();
        let p1 = root.join(format!("{label}-p1"));
        std::fs::create_dir_all(&p1).unwrap();
        let (object, new_environment) = provider(&p1, 1);
        third
            .rebind_provider(new_environment, Arc::new(std::fs::read(&object).unwrap()))
            .unwrap();
        assert_eq!(third.meter(), counts);
        assert_eq!(third.frontend().meter(), frontend);
        assert_eq!(third.frontend().query_counters(), solve);
        assert!(Arc::ptr_eq(&main, third.artifact_bytes("main").unwrap()));
        assert_eq!(
            third
                .run(&root.join(format!("{label}-provider-only")))
                .unwrap()
                .exit_code,
            Some(45)
        );
    }
}

#[test]
fn source_foreign_real_process_restart_uses_required_managed_pages_before_first_edit() {
    use ace_abi::source_foreign::{Session, Target};
    use std::{process::Command, sync::Arc};
    let root = scope("real-restart");
    let p0 = root.join("provider");
    std::fs::create_dir_all(&p0).unwrap();
    let (object, environment) = provider(&p0, 0);
    let manifest = p0.join("manifest.json");
    std::fs::write(
        &manifest,
        serde_json::to_vec(environment.manifest()).unwrap(),
    )
    .unwrap();
    let provider = Arc::new(std::fs::read(&object).unwrap());
    let store_path = root.join("managed-store");
    let store = ace_artifacts::store::ManagedStore::open(&store_path).unwrap();
    for target in [Target::Native, Target::C] {
        let source="fn independent() int { return 7 }\nfn helper() int { return foreign_add(20,22) }\nfn main() int { return helper() }";
        let session = Session::cold(
            source,
            "source938",
            "p938",
            &[SOURCE_CAP.into()],
            environment.clone(),
            provider.clone(),
            target,
        )
        .unwrap();
        let saved = session.snapshot().unwrap();
        let stored = saved
            .publish(&store, &format!("source938-{target:?}"), None)
            .unwrap();
        let second = saved
            .publish(&store, &format!("source938-{target:?}"), Some(&stored))
            .unwrap();
        assert_eq!(
            stored.artifact_id, second.artifact_id,
            "identical publication reuses exact managed closure"
        );
        drop(saved);
        drop(session);
        let request = root.join(format!("request-{target:?}.json"));
        std::fs::write(&request,serde_json::to_vec(&serde_json::json!({"schema":"source938/restart-probe/1",
            "store":store_path,"stored":stored,"target":target,"manifest":manifest,"provider":object,
            "output":root.join(format!("run-{target:?}")),"edits":[
                {"owner":"helper","declaration":"fn helper() int { return foreign_add(true,22) }"},
                {"owner":"helper","declaration":"fn helper() int { return foreign_add(21,22) }"}
            ]})).unwrap()).unwrap();
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_source-foreign-restore-probe"));
        cmd.arg("--request")
            .arg(&request)
            .arg("--contract")
            .arg(environment.contract_digest())
            .arg("--provider")
            .arg(environment.provider_digest())
            .arg("--root")
            .arg(&stored.digest);
        let result = ace_core::link::run_deadlined_captured(
            &mut cmd,
            std::time::Duration::from_secs(70),
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
        )
        .unwrap();
        assert_eq!(
            result.exit_code,
            Some(0),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(report["schema"], "source938/restart-probe-result/1");
        for (key, value) in report["restored"].as_object().unwrap() {
            assert_eq!(value, 0, "restore produced {key}");
        }
        assert_eq!(report["steps"][0]["accepted"], false);
        assert_eq!(report["steps"][0]["before"], report["steps"][0]["after"]);
        assert_eq!(report["steps"][1]["accepted"], true);
        assert_eq!(report["exit"], 43, "parent-side independent oracle");
        assert_eq!(
            report["solver_runs"], 2,
            "failed attempt and repair are both real solves"
        );
    }
}

#[test]
fn source_foreign_actual_provider_symbols_are_checked_before_source_or_target_production() {
    use ace_abi::source_foreign::{Session, Target};
    use std::sync::Arc;
    let root = scope("missing-provider-symbol");
    let (path, environment) = provider(&root, 0);
    let mut object = std::fs::read(path).unwrap();
    let needle = b"provider_add";
    let replacement = b"provider_bad";
    let mut found = 0;
    for start in 0..object.len() - needle.len() {
        if &object[start..start + needle.len()] == needle {
            object[start..start + needle.len()].copy_from_slice(replacement);
            found += 1;
        }
    }
    assert!(found > 0);
    let m = environment.manifest().clone();
    // Even a host inventory claiming the name is present cannot turn the
    // actual wrong object into a valid source binding or a linker fallback.
    let h = HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: hash(&object),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: [SOURCE_CAP.into()].into_iter().collect(),
        available_symbols: ["provider_add".into()].into_iter().collect(),
        binding_generation: 1,
    };
    let admitted = m.admit(&h).unwrap();
    for target in [Target::Native, Target::C] {
        let result = Session::cold(
            "not even parseable Auto source",
            "m938",
            "p938",
            &[SOURCE_CAP.into()],
            admitted.clone(),
            Arc::new(object.clone()),
            target,
        );
        let error = result
            .err()
            .expect("provider rejection must precede parser/codegen/link");
        assert!(error.starts_with("foreign.provider-symbol:"), "{error}");
    }
}

#[test]
fn source_foreign_context_binding_requires_actual_data_symbol_and_keeps_nonzero_context_legal() {
    use ace_abi::source_foreign::{Session, Target};
    use std::sync::Arc;
    let root = scope("provider-context-kind");
    let (path, environment) = provider_with(&root, 0, true, false);
    let bytes = Arc::new(std::fs::read(path).unwrap());
    let mut m = environment.manifest().clone();
    m.declarations
        .iter_mut()
        .find(|d| d.source_name == "foreign_div")
        .unwrap()
        .context_symbol = Some("provider_add".into());
    let h = HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: hash(&bytes),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: [SOURCE_CAP.into()].into_iter().collect(),
        available_symbols: m
            .declarations
            .iter()
            .flat_map(|d| std::iter::once(d.symbol.clone()).chain(d.context_symbol.iter().cloned()))
            .collect(),
        binding_generation: 1,
    };
    let wrong = m.admit(&h).unwrap();
    for target in [Target::Native, Target::C] {
        Session::cold(
            "fn main() int { return foreign_div(84,2) }",
            "m938",
            "p938",
            &[SOURCE_CAP.into()],
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        let error = Session::cold(
            "not parseable",
            "m938",
            "p938",
            &[SOURCE_CAP.into()],
            wrong.clone(),
            bytes.clone(),
            target,
        )
        .err()
        .unwrap();
        assert!(
            error.starts_with("foreign.provider-symbol-kind:"),
            "{error}"
        );
    }
}

#[test]
fn source_foreign_protocol_rejects_unknown_fields_and_empty_selection_before_any_store_or_tool() {
    use std::process::Command;
    let root = scope("source-protocol");
    let store = root.join("must-not-exist");
    let request = serde_json::json!({"recipe":ace_abi::worker_source::RECIPE,"nonce":"n938","request_hash":"h938","deck_b64":"","leg":"native",
        "store_root":store,"steps":[{"op":"restart"}]});
    let wrap = |request: &serde_json::Value| {
        serde_json::json!({"op":"request","request":request}).to_string()
    };
    let begin = "{\"op\":\"begin\",\"schema\":\"abiworker/1\"}";
    let end = "{\"op\":\"end\"}";
    let mut unknown = serde_json::json!({"op":"request","request":request});
    unknown["safe"] = true.into();
    let mut empty = request.clone();
    empty["steps"] = serde_json::json!([]);
    let mut ignored = request.clone();
    ignored["steps"][0]["from"] = "ignored".into();
    let inputs = [
        (wrap(&request), end.to_string(), None),
        (unknown.to_string(), end.to_string(), Some("protocol")),
        (
            wrap(&request),
            "{\"op\":\"end\",\"unknown\":1}".into(),
            Some("protocol"),
        ),
        (wrap(&empty), end.into(), Some("recipe")),
        (wrap(&ignored), end.into(), Some("op")),
    ];
    for (request, end, error) in inputs {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_abi-worker"));
        let run = ace_core::link::run_deadlined_typed(
            &mut cmd,
            std::time::Duration::from_secs(5),
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
            format!("{begin}\n{request}\n{end}\n").into_bytes(),
        )
        .unwrap();
        let response: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
        if let Some(code) = error {
            assert_eq!(response["error"]["code"], code);
        } else {
            assert_eq!(response["ok"]["restart_pending"], true);
        }
        assert!(
            !store.exists(),
            "invalid/non-query operation touched the store or producers"
        );
    }
}

// The policy in these controls is the fixture owner, separate from serialized
// session data. Known header/table and live provider content are re-approved.
fn fixture_readmit(m: ManualManifest, bytes: &[u8], generation: u64) -> AdmittedForeign {
    let authority = HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: hash(bytes),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: m.facts.capabilities.iter().cloned().collect(),
        available_symbols: m
            .declarations
            .iter()
            .flat_map(|d| std::iter::once(d.symbol.clone()).chain(d.context_symbol.iter().cloned()))
            .collect(),
        binding_generation: generation,
    };
    m.admit(&authority).unwrap()
}

#[test]
fn source_foreign_declaration_readers_atomic_after_restore() {
    use ace_abi::source_foreign::{Meter, Session, Target};
    use std::sync::Arc;
    let root = scope("source-declaration");
    let (object, environment) = provider_with(&root, 0, true, false);
    let bytes = Arc::new(std::fs::read(&object).unwrap());
    let source = "fn independent() int { return 7 }\nfn helper() int { return foreign_add(20,22) }\nfn flag() bool { return foreign_not(false) }\nfn main() int { return helper() }";
    let caps = vec![SOURCE_CAP.into()];
    for target in [Target::Native, Target::C] {
        let label = format!("{target:?}");
        let cold = Session::cold(
            source,
            "source938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        let saved = cold.snapshot().unwrap();
        drop(cold);
        let mut session = Session::restore(
            &saved.root,
            &saved.digest,
            target,
            &caps,
            environment.clone(),
            |id| saved.pages.get(id).cloned(),
        )
        .unwrap();
        assert_eq!(session.frontend().query_counters().solver_runs, 0);
        assert_eq!(session.frontend().meter().whole_parse_attempts, 0);
        let unchanged: Vec<_> = ["independent", "flag", "main"]
            .into_iter()
            .map(|name| {
                (
                    name,
                    session.artifact_bytes(name).unwrap().clone(),
                    session.frontend().page(name).unwrap().bytes.clone(),
                    session.frontend().body_identity(name),
                )
            })
            .collect();
        // First operation after R is an illegal signature edit. It must attempt
        // the actual reader but leave every published layer at the old root.
        let mut bad = environment.manifest().clone();
        let FnForm::Pure { params, .. } = &mut bad.facts.functions[0].form else {
            panic!()
        };
        params[0] = LaneKind::Bool;
        let bad = fixture_readmit(bad, &bytes, 2);
        let error = session.prepare_foreign(bad, bytes.clone()).err().unwrap();
        assert!(error.contains("source.foreign.type-mismatch"), "{error}");
        assert!(error.contains("span:"), "{error}");
        assert_eq!(session.snapshot().unwrap().digest, saved.digest);
        assert_eq!(
            session
                .run(&root.join(format!("{label}-after-bad")))
                .unwrap()
                .exit_code,
            Some(42)
        );
        // Removing the redundant pure tag changes only add's declaration;
        // the separately reviewed global checked contract remains available.
        let mut changed = environment.manifest().clone();
        changed.facts.effects.retain(|e| e != "pure:provider_add");
        let changed = fixture_readmit(changed, &bytes, 3);
        let cancelled = session
            .prepare_foreign(changed.clone(), bytes.clone())
            .unwrap();
        assert_eq!(session.snapshot().unwrap().digest, saved.digest);
        drop(cancelled);
        assert_eq!(session.snapshot().unwrap().digest, saved.digest);
        let candidate = session
            .prepare_foreign(changed.clone(), bytes.clone())
            .unwrap();
        session.commit_foreign(candidate).unwrap();
        assert_ne!(session.snapshot().unwrap().digest, saved.digest);
        for (name, artifact, facts, body) in unchanged {
            assert_eq!(session.function_meter(name), Meter::default(), "{name}");
            assert_eq!(
                session.frontend().function_meter(name),
                Default::default(),
                "{name}"
            );
            assert!(
                Arc::ptr_eq(&artifact, session.artifact_bytes(name).unwrap()),
                "{name}"
            );
            assert!(
                Arc::ptr_eq(&facts, &session.frontend().page(name).unwrap().bytes),
                "{name}"
            );
            assert_eq!(body, session.frontend().body_identity(name), "{name}");
        }
        assert_eq!(session.frontend().query_counters().solver_runs, 2);
        assert_eq!(
            session
                .frontend()
                .function_meter("helper")
                .body_parse_attempts,
            3
        );
        assert_eq!(session.frontend().function_meter("helper").hir_produced, 2);
        assert_eq!(
            session
                .run(&root.join(format!("{label}-after-change")))
                .unwrap()
                .exit_code,
            Some(42)
        );
        let next = session.snapshot().unwrap();
        drop(session);
        let mut restored = Session::restore(
            &next.root,
            &next.digest,
            target,
            &caps,
            changed.clone(),
            |id| next.pages.get(id).cloned(),
        )
        .unwrap();
        assert_eq!(restored.frontend().query_counters().solver_runs, 0);
        // No warmup: revert the declaration as the first edit after the second R.
        let candidate = restored
            .prepare_foreign(environment.clone(), bytes.clone())
            .unwrap();
        restored.commit_foreign(candidate).unwrap();
        assert_eq!(restored.frontend().query_counters().solver_runs, 1);
        assert_eq!(
            restored
                .frontend()
                .function_meter("helper")
                .body_parse_attempts,
            1
        );
        assert_eq!(restored.function_meter("flag"), Meter::default());
        assert_eq!(
            restored
                .run(&root.join(format!("{label}-second-restore")))
                .unwrap()
                .exit_code,
            Some(42)
        );
        // A candidate from an older declaration root cannot overwrite a later
        // ordinary source commit, even with identical source owner names.
        let stale = restored.prepare_foreign(changed, bytes.clone()).unwrap();
        let body = restored
            .prepare("independent", "fn independent() int { return 8 }")
            .unwrap();
        restored.commit(body).unwrap();
        let accepted = restored.snapshot().unwrap().digest;
        assert!(restored.commit_foreign(stale).is_err());
        assert_eq!(restored.snapshot().unwrap().digest, accepted);
        let before_cap = restored.frontend().meter();
        let mut cap = environment.manifest().clone();
        cap.facts.capabilities.push("foreign.extra".into());
        let denied = fixture_readmit(cap, &bytes, 4);
        assert!(restored.prepare_foreign(denied, bytes.clone()).is_err());
        assert_eq!(restored.frontend().meter(), before_cap);
        assert_eq!(restored.snapshot().unwrap().digest, accepted);
        // A real shared-header edit is a dependency of both actual foreign
        // readers, including the one whose call site's text never changed.
        let header_dir = root.join(format!("{label}-header"));
        std::fs::create_dir_all(&header_dir).unwrap();
        let mut header = std::fs::read(fixture().join("source_foreign938.h")).unwrap();
        header.extend_from_slice(b"\n/* reviewed header revision 2 */\n");
        std::fs::write(header_dir.join("source_foreign938.h"), &header).unwrap();
        let c_path = header_dir.join("provider.c");
        let o_path = header_dir.join("provider.obj");
        std::fs::copy(fixture().join("source_foreign938.c"), &c_path).unwrap();
        let deps = ace_abi::ctool::compile_c_with_deps(&c_path, &header_dir, &o_path).unwrap();
        assert!(deps
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "source_foreign938.h")));
        let new_bytes = Arc::new(std::fs::read(o_path).unwrap());
        let mut header_manifest = environment.manifest().clone();
        header_manifest.header_digest = hash(&header);
        let header_env = fixture_readmit(header_manifest, &new_bytes, 5);
        let before: Vec<_> = ["helper", "flag", "main", "independent"]
            .into_iter()
            .map(|name| {
                (
                    name,
                    restored.frontend().function_meter(name),
                    restored.function_meter(name),
                    restored.artifact_bytes(name).unwrap().clone(),
                    restored.frontend().page(name).unwrap().bytes.clone(),
                )
            })
            .collect();
        let candidate = restored
            .prepare_foreign(header_env.clone(), new_bytes)
            .unwrap();
        restored.commit_foreign(candidate).unwrap();
        for (name, front, target_counts, artifact, facts) in before {
            if name == "main" || name == "independent" {
                assert_eq!(restored.frontend().function_meter(name), front);
                assert_eq!(restored.function_meter(name), target_counts);
                assert!(Arc::ptr_eq(
                    &artifact,
                    restored.artifact_bytes(name).unwrap()
                ));
                assert!(Arc::ptr_eq(
                    &facts,
                    &restored.frontend().page(name).unwrap().bytes
                ));
            } else {
                assert_eq!(
                    restored.frontend().function_meter(name).hir_produced,
                    front.hir_produced + 1
                );
                assert_ne!(restored.function_meter(name), target_counts);
            }
        }
        assert_eq!(
            restored.run(&header_dir.join("linked")).unwrap().exit_code,
            Some(42)
        );
        let snapshot = restored.snapshot().unwrap();
        let again = Session::restore(
            &snapshot.root,
            &snapshot.digest,
            target,
            &caps,
            header_env,
            |id| snapshot.pages.get(id).cloned(),
        )
        .unwrap();
        assert_eq!(again.frontend().query_counters().solver_runs, 0);
    }
}

#[test]
fn source_foreign_name_changes_keep_nonreader_slots() {
    use ace_abi::source_foreign::{Session, Target};
    use std::sync::Arc;
    let root = scope("source-name-slots");
    let (object, environment) = provider_with(&root, 0, true, false);
    let bytes = Arc::new(std::fs::read(object).unwrap());
    let caps = vec![SOURCE_CAP.into()];
    let source = "fn flag() bool { return foreign_not(false) }\nfn main() int { if flag() { return foreign_add(20,22) }\nreturn 9 }";
    for target in [Target::Native, Target::C] {
        let mut session = Session::cold(
            source,
            "source938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        let original = session.snapshot().unwrap();
        let old_flag = session.frontend().body_identity("flag");
        let flag_bytes = session.artifact_bytes("flag").unwrap().clone();
        let counters = session.meter();
        let cold_flag_count = session.function_meter("flag");
        let front = session.frontend().meter();
        // Delete an unused declaration BEFORE not's cold slot, then append a
        // new name. Existing bodies and target artifacts must not be rewritten.
        let mut manifest = environment.manifest().clone();
        manifest
            .declarations
            .iter_mut()
            .find(|d| d.source_name == "foreign_calls")
            .unwrap()
            .source_name = "foreign_new".into();
        manifest.declarations.reverse(); // transport order is not semantic identity
        let next = fixture_readmit(manifest, &bytes, 2);
        let candidate = session
            .prepare_foreign(next.clone(), bytes.clone())
            .unwrap();
        session.commit_foreign(candidate).unwrap();
        assert_eq!(session.frontend().meter(), front);
        assert_eq!(session.meter(), counters);
        assert_eq!(session.frontend().body_identity("flag"), old_flag);
        assert!(Arc::ptr_eq(
            &flag_bytes,
            session.artifact_bytes("flag").unwrap()
        ));
        let expected = session.snapshot().unwrap().digest;
        let mut removed_reader = next.manifest().clone();
        removed_reader
            .declarations
            .retain(|d| d.source_name != "foreign_not");
        removed_reader
            .facts
            .functions
            .retain(|f| f.symbol != "provider_not");
        removed_reader
            .facts
            .effects
            .retain(|e| e != "pure:provider_not");
        assert!(session
            .prepare_foreign(fixture_readmit(removed_reader, &bytes, 3), bytes.clone())
            .is_err());
        assert_eq!(session.snapshot().unwrap().digest, expected);
        assert_eq!(session.frontend().body_identity("flag"), old_flag);
        // Resolve under the new name table without moving the retained not slot.
        let candidate = session
            .prepare(
                "main",
                "fn main() int { if flag() { return 40 + foreign_new() }\nreturn 9 }",
            )
            .unwrap();
        session.commit(candidate).unwrap();
        assert_eq!(
            session
                .run(&root.join(format!("{target:?}-appended")))
                .unwrap()
                .exit_code,
            Some(42)
        );
        let flag_count = session.function_meter("flag");
        assert_eq!(flag_count, cold_flag_count);
        // Rechecking the old reader after append/remove still resolves not.
        let candidate = session
            .prepare("flag", "fn flag() bool { return foreign_not(true) }")
            .unwrap();
        session.commit(candidate).unwrap();
        assert_eq!(
            session
                .run(&root.join(format!("{target:?}-retained")))
                .unwrap()
                .exit_code,
            Some(9)
        );
        let saved = session.snapshot().unwrap();
        drop(session);
        let mut restored =
            Session::restore(&saved.root, &saved.digest, target, &caps, next, |id| {
                saved.pages.get(id).cloned()
            })
            .unwrap();
        assert_eq!(restored.frontend().query_counters().solver_runs, 0);
        let candidate = restored
            .prepare("flag", "fn flag() bool { return foreign_not(false) }")
            .unwrap();
        restored.commit(candidate).unwrap();
        assert_eq!(
            restored
                .run(&root.join(format!("{target:?}-restored")))
                .unwrap()
                .exit_code,
            Some(42)
        );
        assert_ne!(saved.digest, original.digest);
    }
}

#[test]
fn source_foreign_formal_worker_rebinds_provider_without_any_caller_production() {
    use base64::Engine as _;
    use std::process::Command;
    let root = scope("formal-provider-swap");
    let deck_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-ca-d-source-938/src/add.atomdata");
    let deck: serde_json::Value =
        serde_json::from_slice(&std::fs::read(deck_path).unwrap()).unwrap();
    for leg in ["native", "c"] {
        let store = root.join(leg);
        let mut first_contract = None;
        let mut first_provider = None;
        for (n, bias) in [0, 1, 0].into_iter().enumerate() {
            let mut deck = deck.clone();
            deck["provider_bias"] = bias.into();
            let request = serde_json::json!({"recipe":ace_abi::worker_source::RECIPE,
                "nonce":format!("n938-{n}"),"request_hash":format!("h938-{n}"),
                "deck_b64":base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(&deck).unwrap()),
                "leg":leg,"store_root":store,"steps":[{"op":"query"}]});
            let input = format!(
                "{}\n{}\n{}\n",
                serde_json::json!({"op":"begin","schema":"abiworker/1"}),
                serde_json::json!({"op":"request","request":request}),
                serde_json::json!({"op":"end"})
            );
            let run = ace_core::link::run_deadlined_typed(
                &mut Command::new(env!("CARGO_BIN_EXE_abi-worker")),
                std::time::Duration::from_secs(30),
                ace_core::link::CaptureCaps {
                    stdout_max: 1 << 20,
                    stderr_max: 1 << 20,
                },
                input.into_bytes(),
            )
            .unwrap();
            let response: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
            assert!(response.get("error").is_none(), "{response}");
            let report = &response["ok"]["source_import"];
            assert_eq!(report["actual_exit"], 42 + bias);
            assert_eq!(report["reference_exit"], 42 + bias);
            assert_eq!(report["actual_events"], serde_json::json!(["provider_add"]));
            assert_eq!(
                report["reference_events"],
                serde_json::json!(["provider_add"])
            );
            if n == 0 {
                first_contract = Some(report["contract_digest"].clone());
                first_provider = Some(report["provider_digest"].clone());
            } else {
                assert_eq!(Some(report["contract_digest"].clone()), first_contract);
                for field in [
                    "source_parse",
                    "solver",
                    "hir",
                    "mir",
                    "native",
                    "c_fragment",
                ] {
                    assert_eq!(
                        report[field], 0,
                        "{leg} provider-only swap produced {field}"
                    );
                }
                assert_eq!(
                    response["ok"]["steps"][0]["compute_calls"],
                    if n == 1 { 1 } else { 0 }
                );
                if n == 1 {
                    assert_ne!(Some(report["provider_digest"].clone()), first_provider);
                } else {
                    assert_eq!(Some(report["provider_digest"].clone()), first_provider);
                }
            }
        }
    }
}

fn fixture_current(
    environment: &AdmittedForeign,
    target: ace_abi::source_foreign::Target,
) -> ace_abi::source_foreign::CurrentAuthority {
    let m = environment.manifest();
    ace_abi::source_foreign::CurrentAuthority {
        foreign: HostAuthority {
            review_id: m.review_id.clone(),
            contract_digest: m.contract_digest().unwrap(),
            provider: m.provider.clone(),
            provider_digest: environment.provider_digest().into(),
            header_digest: m.header_digest.clone(),
            physical_table_digest: m.physical_table_digest.clone(),
            target: m.target.clone(),
            capabilities: m.facts.capabilities.iter().cloned().collect(),
            available_symbols: m
                .declarations
                .iter()
                .flat_map(|d| {
                    std::iter::once(d.symbol.clone()).chain(d.context_symbol.iter().cloned())
                })
                .collect(),
            binding_generation: environment.generation(),
        },
        runtime_digest: ace_abi::source_foreign::certification_runtime_policy(target),
        emitter: target.identity(),
    }
}
// Only these reviewed fixtures provide this convenience. Production Session
// has no unchecked run method: every execution requires a fresh host callback.
trait RunFixture {
    fn run(&mut self, path: &Path) -> Result<ace_native::link_driver::PeRun, String>;
}
impl RunFixture for ace_abi::source_foreign::Session {
    fn run(&mut self, path: &Path) -> Result<ace_native::link_driver::PeRun, String> {
        let authority = fixture_current(self.frontend().foreign(), self.target());
        self.run_authorized(path, &mut || Ok(authority.clone()), None)
    }
}

#[test]
fn source_foreign_publication_faults_and_revocation_preserve_the_accepted_root() {
    use ace_abi::source_foreign::{
        read_pointer_bytes, CurrentAuthority, Edit, Session, Stored, Target,
    };
    use std::{
        cell::RefCell,
        sync::{
            atomic::{AtomicBool, Ordering},
            Arc,
        },
    };
    let root = scope("publication-faults");
    let (object, environment) = provider(&root, 0);
    let bytes = Arc::new(std::fs::read(object).unwrap());
    let caps = vec![SOURCE_CAP.into()];
    let source = "fn independent() int { return 7 }\nfn helper() int { return foreign_add(20,22) }\nfn main() int { return helper() }";
    for target in [Target::Native, Target::C] {
        let dir = root.join(format!("{target:?}"));
        std::fs::create_dir_all(&dir).unwrap();
        let store = ace_artifacts::store::ManagedStore::open(&dir.join("store")).unwrap();
        let pointer = dir.join("current.json");
        let mut session = Session::cold(
            source,
            "source938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        let authority = fixture_current(&environment, target);
        let mut current = || Ok(authority.clone());
        let baseline = session
            .prepare_publication(Edit::Current, &dir.join("old"), &mut current, None)
            .unwrap();
        let old_digest = baseline.snapshot().digest.clone();
        assert_eq!(
            baseline.run(&mut current, None).unwrap().exit_code,
            Some(42)
        );
        let stored = session
            .publish_prepared(
                baseline,
                &store,
                &pointer,
                None,
                None,
                |s| serde_json::to_vec(s).map_err(|e| e.to_string()),
                &mut current,
                None,
            )
            .unwrap();
        let old_pointer = read_pointer_bytes(&pointer).unwrap().unwrap();
        let pointer_id = ace_a2x_core::sha256::sha256_hex(&old_pointer);
        let original_stored: Stored = serde_json::from_slice(&old_pointer).unwrap();
        let untouched = session.artifact_bytes("main").unwrap().clone();
        let old_facts = session.frontend().page("main").unwrap().bytes.clone();
        let declaration = "fn helper() int { let extra=1\nreturn foreign_add(20,22)+extra }";
        let body =
            |session: &mut Session| Edit::Body(session.prepare("helper", declaration).unwrap());
        for fault in [
            "revoked-before-link",
            "revoked-after-link",
            "cancel-before-link",
            "cancel-after-link",
            "runtime",
        ] {
            let cancel = AtomicBool::new(fault == "cancel-before-link");
            let mut calls = 0;
            let mut policy = || -> Result<CurrentAuthority, String> {
                calls += 1;
                let mut a = authority.clone();
                if fault == "revoked-before-link" || (fault == "revoked-after-link" && calls == 2) {
                    a.foreign.capabilities.clear();
                }
                if fault == "cancel-after-link" && calls == 2 {
                    cancel.store(true, Ordering::Release);
                }
                if fault == "runtime" {
                    a.runtime_digest = "0".repeat(64);
                }
                Ok(a)
            };
            let edit = body(&mut session);
            assert!(
                session
                    .prepare_publication(edit, &dir.join(fault), &mut policy, Some(&cancel))
                    .is_err(),
                "{fault}"
            );
            assert_eq!(session.snapshot().unwrap().digest, old_digest, "{fault}");
            assert_eq!(std::fs::read(&pointer).unwrap(), old_pointer);
        }
        // Runtime/authority can change AFTER a target became ready too.
        let edit = body(&mut session);
        let ready = session
            .prepare_publication(edit, &dir.join("ready-revoked"), &mut current, None)
            .unwrap();
        let mut denied = authority.clone();
        denied.foreign.capabilities.clear();
        assert!(ready.run(&mut || Ok(denied.clone()), None).is_err());
        let mut wrong_runtime = authority.clone();
        wrong_runtime.runtime_digest = "0".repeat(64);
        assert!(ready.run(&mut || Ok(wrong_runtime.clone()), None).is_err());
        assert_eq!(session.snapshot().unwrap().digest, old_digest);
        drop(ready);
        for fault in [
            "cancel-at-last-qualification",
            "metadata-failure",
            "cancel-before-switch",
            "replace-failure",
            "late-base",
            "late-pointer",
            "ready-tampered",
        ] {
            let cancel = AtomicBool::new(false);
            let edit = body(&mut session);
            let ready = session
                .prepare_publication(edit, &dir.join(fault), &mut current, None)
                .unwrap();
            if fault == "ready-tampered" {
                std::fs::write(ready.executable(), b"changed executable").unwrap();
            }
            let blocker = RefCell::new(None);
            let encode = |stored: &Stored| -> Result<Vec<u8>, String> {
                if fault == "metadata-failure" {
                    return Err("injected pointer encoding failure".into());
                }
                if fault == "cancel-before-switch" {
                    cancel.store(true, Ordering::Release);
                }
                if fault == "replace-failure" {
                    use std::os::windows::fs::OpenOptionsExt;
                    // Allow the final identity read, deny replacing this file.
                    *blocker.borrow_mut() = Some(
                        std::fs::OpenOptions::new()
                            .read(true)
                            .share_mode(0x1 | 0x2)
                            .open(&pointer)
                            .unwrap(),
                    );
                }
                if fault == "late-pointer" {
                    let mut record: serde_json::Value =
                        serde_json::from_slice(&old_pointer).unwrap();
                    record["other-publisher"] = true.into();
                    std::fs::write(&pointer, serde_json::to_vec(&record).unwrap()).unwrap();
                }
                serde_json::to_vec(stored).map_err(|e| e.to_string())
            };
            let expected = if fault == "late-base" {
                "0".repeat(64)
            } else {
                pointer_id.clone()
            };
            let mut final_checks = 0;
            let mut publish_current = || {
                final_checks += 1;
                if fault == "cancel-at-last-qualification" && final_checks == 2 {
                    cancel.store(true, Ordering::Release);
                }
                Ok(authority.clone())
            };
            let result = session.publish_prepared(
                ready,
                &store,
                &pointer,
                Some(&expected),
                Some(&stored),
                encode,
                &mut publish_current,
                Some(&cancel),
            );
            assert!(result.is_err(), "{fault}");
            drop(blocker);
            assert_eq!(session.snapshot().unwrap().digest, old_digest, "{fault}");
            if fault == "late-pointer" {
                assert_ne!(
                    std::fs::read(&pointer).unwrap(),
                    old_pointer,
                    "late writer was overwritten"
                );
                std::fs::write(&pointer, &old_pointer).unwrap();
            } else {
                assert_eq!(std::fs::read(&pointer).unwrap(), old_pointer, "{fault}");
            }
            assert!(
                !std::fs::read_dir(&dir).unwrap().any(|e| e
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .starts_with("current.tmp-")),
                "temporary pointer leaked: {fault}"
            );
        }
        assert_eq!(
            session.run(&dir.join("after-failures")).unwrap().exit_code,
            Some(42)
        );
        assert!(Arc::ptr_eq(
            &untouched,
            session.artifact_bytes("main").unwrap()
        ));
        assert!(Arc::ptr_eq(
            &old_facts,
            &session.frontend().page("main").unwrap().bytes
        ));
        // Repair installs exactly the candidate snapshot that was linked, never
        // an old source with a new object or a new source with an old object.
        let edit = body(&mut session);
        let ready = session
            .prepare_publication(edit, &dir.join("repair"), &mut current, None)
            .unwrap();
        let expected = ready.snapshot().digest.clone();
        assert_eq!(ready.run(&mut current, None).unwrap().exit_code, Some(43));
        let stored = session
            .publish_prepared(
                ready,
                &store,
                &pointer,
                Some(&pointer_id),
                Some(&stored),
                |s| serde_json::to_vec(s).map_err(|e| e.to_string()),
                &mut current,
                None,
            )
            .unwrap();
        assert_eq!(session.snapshot().unwrap().digest, expected);
        assert_eq!(
            session.run(&dir.join("installed")).unwrap().exit_code,
            Some(43)
        );
        let stored_pointer: Stored =
            serde_json::from_slice(&std::fs::read(pointer).unwrap()).unwrap();
        assert_eq!(stored_pointer.artifact_id, stored.artifact_id);
        drop(session);
        let mut restored =
            Session::restore_stored(&store, &stored, target, &caps, environment.clone()).unwrap();
        assert_eq!(restored.frontend().query_counters().solver_runs, 0);
        assert_eq!(restored.snapshot().unwrap().digest, expected);
        let edit = Edit::Body(
            restored
                .prepare("helper", "fn helper() int { return foreign_add(20,24) }")
                .unwrap(),
        );
        let ready = restored
            .prepare_publication(edit, &dir.join("first-restored-edit"), &mut current, None)
            .unwrap();
        assert_eq!(ready.run(&mut current, None).unwrap().exit_code, Some(44));
        assert_eq!(restored.snapshot().unwrap().digest, expected);
        // A different host session prepared from the exact earlier pointer
        // bytes is genuinely late after this accepted publication.
        let mut delayed =
            Session::restore_stored(&store, &original_stored, target, &caps, environment.clone())
                .unwrap();
        let edit = Edit::Body(
            delayed
                .prepare("helper", "fn helper() int { return foreign_add(20,24) }")
                .unwrap(),
        );
        let ready = delayed
            .prepare_publication(edit, &dir.join("delayed-session"), &mut current, None)
            .unwrap();
        let accepted_pointer = std::fs::read(dir.join("current.json")).unwrap();
        let error = delayed
            .publish_prepared(
                ready,
                &store,
                &dir.join("current.json"),
                Some(&pointer_id),
                Some(&original_stored),
                |s| serde_json::to_vec(s).map_err(|e| e.to_string()),
                &mut current,
                None,
            )
            .err()
            .unwrap();
        assert_eq!(error, "foreign.publication.late-pointer");
        assert_eq!(delayed.snapshot().unwrap().digest, old_digest);
        assert_eq!(
            std::fs::read(dir.join("current.json")).unwrap(),
            accepted_pointer
        );
    }
}

#[test]
fn source_foreign_real_signature_and_source_repair_publish_together_after_restore() {
    use ace_abi::source_foreign::{Edit, Session, Target};
    use std::sync::Arc;
    let root = scope("real-signature-repair");
    let (object, environment) = provider(&root, 0);
    let original = Arc::new(std::fs::read(&object).unwrap());
    let changed_dir = root.join("new-reviewed-provider");
    std::fs::create_dir_all(&changed_dir).unwrap();
    let header = std::fs::read_to_string(fixture().join("source_foreign938.h"))
        .unwrap()
        .replace("provider_add(int32_t a,", "provider_add(uint8_t a,");
    let c = std::fs::read_to_string(fixture().join("source_foreign938.c"))
        .unwrap()
        .replace("provider_add(int32_t a,", "provider_add(uint8_t a,");
    let mut table: serde_json::Value = serde_json::from_slice(
        &std::fs::read(fixture().join("source_foreign938.contract")).unwrap(),
    )
    .unwrap();
    table["functions"]["provider_add"] =
        "i32(u8,i32); logical bool input 0/1; total unsigned wrapping addition".into();
    let table = serde_json::to_vec(&table).unwrap();
    std::fs::write(changed_dir.join("source_foreign938.h"), &header).unwrap();
    std::fs::write(changed_dir.join("source_foreign938.contract"), &table).unwrap();
    let c_path = changed_dir.join("provider.c");
    let o_path = changed_dir.join("provider.obj");
    std::fs::write(&c_path, c).unwrap();
    ace_abi::ctool::compile_c_with_deps(&c_path, &changed_dir, &o_path).unwrap();
    let changed_bytes = Arc::new(std::fs::read(&o_path).unwrap());
    let mut manifest = environment.manifest().clone();
    manifest.header_digest = hash(header.as_bytes());
    manifest.physical_table_digest = hash(&table);
    manifest.facts.content_version = "bool-add-v2".into();
    let FnForm::Pure { params, .. } = &mut manifest.facts.functions[0].form else {
        panic!()
    };
    params[0] = LaneKind::Bool;
    let changed = fixture_readmit(manifest, &changed_bytes, 2);
    let source = "fn independent() int { return 7 }\nfn helper() int { return foreign_add(20,22) }\nfn main() int { return helper() }";
    let new_source = "fn independent() int { return 7 }\nfn helper() int { let value=true\nreturn foreign_add(value,42) }\nfn main() int { return helper() }";
    let caps = vec![SOURCE_CAP.into()];
    for target in [Target::Native, Target::C] {
        let dir = root.join(format!("{target:?}"));
        std::fs::create_dir_all(&dir).unwrap();
        let store = ace_artifacts::store::ManagedStore::open(&dir.join("store")).unwrap();
        let cold = Session::cold(
            source,
            "source938",
            "p938",
            &caps,
            environment.clone(),
            original.clone(),
            target,
        )
        .unwrap();
        let snapshot = cold.snapshot().unwrap();
        drop(cold);
        let child_root = dir.join("declaration-first-process");
        std::fs::create_dir_all(&child_root).unwrap();
        let old_manifest = child_root.join("old-manifest.json");
        let new_manifest = child_root.join("new-manifest.json");
        std::fs::write(
            &old_manifest,
            serde_json::to_vec(environment.manifest()).unwrap(),
        )
        .unwrap();
        std::fs::write(
            &new_manifest,
            serde_json::to_vec(changed.manifest()).unwrap(),
        )
        .unwrap();
        let stored_for_child = snapshot
            .publish(&store, "declaration-first-process", None)
            .unwrap();
        let request_path = child_root.join("request.json");
        let request = serde_json::json!({"schema":"source938/restart-probe/1","store":dir.join("store"),
            "stored":stored_for_child,"target":target,"manifest":old_manifest,"provider":object,
            "output":child_root.join("output"),"edits":[
                {"foreign_manifest":new_manifest,"foreign_provider":o_path,"source":null},
                {"foreign_manifest":new_manifest,"foreign_provider":o_path,"source":new_source}]});
        std::fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let mut command =
            std::process::Command::new(env!("CARGO_BIN_EXE_source-foreign-restore-probe"));
        command
            .arg("--request")
            .arg(&request_path)
            .arg("--contract")
            .arg(environment.contract_digest())
            .arg("--provider")
            .arg(environment.provider_digest())
            .arg("--root")
            .arg(&stored_for_child.digest)
            .arg("--candidate-contract")
            .arg(changed.contract_digest())
            .arg("--candidate-header")
            .arg(&changed.manifest().header_digest)
            .arg("--candidate-table")
            .arg(&changed.manifest().physical_table_digest)
            .arg("--candidate-provider")
            .arg(changed.provider_digest());
        let child = ace_core::link::run_deadlined_typed(
            &mut command,
            std::time::Duration::from_secs(30),
            ace_core::link::CaptureCaps {
                stdout_max: 1 << 20,
                stderr_max: 1 << 20,
            },
            Vec::new(),
        )
        .unwrap();
        assert_eq!(
            child.exit_code,
            Some(0),
            "{}",
            String::from_utf8_lossy(&child.stderr)
        );
        let child: serde_json::Value = serde_json::from_slice(&child.stdout).unwrap();
        for (_, value) in child["restored"].as_object().unwrap() {
            assert_eq!(value, 0);
        }
        assert_eq!(child["steps"][0]["accepted"], false);
        assert_eq!(child["steps"][0]["before"], child["steps"][0]["after"]);
        assert_eq!(child["steps"][1]["accepted"], true);
        assert_eq!(child["exit"], 43);
        assert_eq!(child["solver_runs"], 2);
        assert_eq!(child["main_hir_produced"], 0);
        assert_eq!(child["independent_hir_produced"], 0);
        let old_authority = fixture_current(&environment, target);
        let mut session = Session::restore_authorized(
            &snapshot.root,
            &snapshot.digest,
            target,
            &caps,
            environment.clone(),
            |id| snapshot.pages.get(id).cloned(),
            &mut || Ok(old_authority.clone()),
        )
        .unwrap();
        let main = session.artifact_bytes("main").unwrap().clone();
        let facts = session.frontend().page("main").unwrap().bytes.clone();
        assert_eq!(session.frontend().query_counters().solver_runs, 0);
        // The new independent header/provider/table agree with each other. Only
        // the old ordinary source is invalid under the new logical signature.
        assert!(session
            .prepare_foreign(changed.clone(), changed_bytes.clone())
            .is_err());
        assert_eq!(session.snapshot().unwrap().digest, snapshot.digest);
        assert_eq!(
            session
                .run(&dir.join("after-first-declaration-error"))
                .unwrap()
                .exit_code,
            Some(42)
        );
        let candidate = session
            .prepare_foreign_source(changed.clone(), changed_bytes.clone(), Some(new_source))
            .unwrap();
        let new_authority = fixture_current(&changed, target);
        let mut current = || Ok(new_authority.clone());
        let ready = session
            .prepare_publication(
                Edit::Foreign(candidate),
                &dir.join("ready"),
                &mut current,
                None,
            )
            .unwrap();
        let digest = ready.snapshot().digest.clone();
        assert_eq!(ready.run(&mut current, None).unwrap().exit_code, Some(43));
        assert_eq!(session.snapshot().unwrap().digest, snapshot.digest);
        let stored = session
            .publish_prepared(
                ready,
                &store,
                &dir.join("current.json"),
                None,
                None,
                |s| serde_json::to_vec(s).map_err(|e| e.to_string()),
                &mut current,
                None,
            )
            .unwrap();
        assert_eq!(session.snapshot().unwrap().digest, digest);
        assert_eq!(session.frontend().source(), new_source);
        assert_eq!(
            session.frontend().foreign().contract_digest(),
            changed.contract_digest()
        );
        assert!(Arc::ptr_eq(&main, session.artifact_bytes("main").unwrap()));
        assert!(Arc::ptr_eq(
            &facts,
            &session.frontend().page("main").unwrap().bytes
        ));
        assert_eq!(
            session.frontend().function_meter("main"),
            Default::default()
        );
        assert_eq!(session.frontend().meter().whole_parse_attempts, 0);
        let mut fresh = Session::cold(
            new_source,
            "source938",
            "p938",
            &caps,
            changed.clone(),
            changed_bytes.clone(),
            target,
        )
        .unwrap();
        assert_eq!(
            fresh.artifact_bytes("helper").unwrap().as_slice(),
            session.artifact_bytes("helper").unwrap().as_slice()
        );
        assert_eq!(
            fresh.run(&dir.join("independent-fresh")).unwrap().exit_code,
            Some(43)
        );
        drop(session);
        let mut again = Session::restore_stored_authorized(
            &store,
            &stored,
            target,
            &caps,
            changed.clone(),
            &mut current,
        )
        .unwrap();
        assert_eq!(again.frontend().query_counters().solver_runs, 0);
        let next = new_source.replace("value,42", "value,43");
        let edit = Edit::Body(again.prepare_source(&next).unwrap());
        let ready = again
            .prepare_publication(
                edit,
                &dir.join("second-restore-first-edit"),
                &mut current,
                None,
            )
            .unwrap();
        assert_eq!(ready.run(&mut current, None).unwrap().exit_code, Some(44));
        assert_eq!(again.snapshot().unwrap().digest, digest);
    }
}

#[test]
fn source_foreign_required_restore_rejects_each_missing_corrupt_or_misbound_page() {
    use ace_abi::source_foreign::{runtime_identity, Session, Target};
    use std::sync::Arc;
    let root = scope("required-pages");
    let (object, environment) = provider(&root, 0);
    let bytes = Arc::new(std::fs::read(object).unwrap());
    let caps = vec![SOURCE_CAP.into()];
    let digest = ace_a2x_core::sha256::sha256_hex;
    for target in [Target::Native, Target::C] {
        let session = Session::cold(
            "fn leaf() int { return 7 }\nfn main() int { return foreign_add(20,22) }",
            "source938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            target,
        )
        .unwrap();
        let snapshot = session.snapshot().unwrap();
        let authority = fixture_current(&environment, target);
        let restore =
            |root: &[u8],
             expected: &str,
             pages: &std::collections::BTreeMap<String, Arc<Vec<u8>>>| {
                Session::restore_authorized(
                    root,
                    expected,
                    target,
                    &caps,
                    environment.clone(),
                    |id| pages.get(id).cloned(),
                    &mut || Ok(authority.clone()),
                )
            };
        let restored = restore(&snapshot.root, &snapshot.digest, &snapshot.pages).unwrap();
        assert_eq!(restored.frontend().meter().whole_parse_attempts, 0);
        assert_eq!(restored.frontend().meter().body_parse_attempts, 0);
        assert_eq!(restored.frontend().query_counters().solver_runs, 0);
        assert_eq!(
            restored.meter().mir_attempts
                + restored.meter().native_attempts
                + restored.meter().c_attempts,
            0
        );
        for id in snapshot.pages.keys() {
            let mut missing = snapshot.pages.clone();
            missing.remove(id);
            assert!(
                restore(&snapshot.root, &snapshot.digest, &missing).is_err(),
                "missing required {id}"
            );
            let mut corrupt = snapshot.pages.clone();
            let mut bytes = (*corrupt[id]).clone();
            *bytes.last_mut().unwrap() ^= 1;
            corrupt.insert(id.clone(), Arc::new(bytes));
            assert!(
                restore(&snapshot.root, &snapshot.digest, &corrupt).is_err(),
                "corrupt required {id}"
            );
        }
        let root: serde_json::Value = serde_json::from_slice(&snapshot.root).unwrap();
        for (field, value) in [
            ("schema", "foreign/fake"),
            ("target_identity", "wrong compiler"),
            ("contract", "wrong contract"),
            ("provider_digest", "wrong provider"),
        ] {
            let mut changed = root.clone();
            changed[field] = value.into();
            let bytes = serde_json::to_vec(&changed).unwrap();
            assert!(
                restore(&bytes, &digest(&bytes), &snapshot.pages).is_err(),
                "{field}"
            );
        }
        let mut fake_checked = root.clone();
        fake_checked["safe"] = true.into();
        let fake_checked = serde_json::to_vec(&fake_checked).unwrap();
        assert!(restore(&fake_checked, &digest(&fake_checked), &snapshot.pages).is_err());
        let mut denied = authority.clone();
        denied.foreign.capabilities.clear();
        assert!(Session::restore_authorized(
            &snapshot.root,
            &snapshot.digest,
            target,
            &caps,
            environment.clone(),
            |id| snapshot.pages.get(id).cloned(),
            &mut || Ok(denied.clone())
        )
        .is_err());
        let mut runtime = authority.clone();
        runtime.runtime_digest = "0".repeat(64);
        assert!(Session::restore_authorized(
            &snapshot.root,
            &snapshot.digest,
            target,
            &caps,
            environment.clone(),
            |id| snapshot.pages.get(id).cloned(),
            &mut || Ok(runtime.clone())
        )
        .is_err());
        // Even a self-consistent rehashed page/root cannot mint a current SDK
        // runtime authorization or a foreign declaration credential.
        for which in ["runtime", "foreign-declaration", "source", "target-owner"] {
            let mut changed_root = root.clone();
            let mut pages = snapshot.pages.clone();
            if which == "runtime" {
                let old = root["runtime"].as_str().unwrap();
                let mut bytes = (*pages[old]).clone();
                bytes.extend_from_slice(b"\n/* unapproved runtime */\n");
                let id = digest(&bytes);
                changed_root["runtime"] = id.clone().into();
                changed_root["runtime_identity"] = runtime_identity(target, &id).into();
                pages.insert(id, Arc::new(bytes));
            } else if which == "target-owner" {
                let old = root["functions"]["main"].as_str().unwrap();
                let mut wire: serde_json::Value = serde_json::from_slice(&pages[old]).unwrap();
                wire["name"] = "another-owner".into();
                let bytes = serde_json::to_vec(&wire).unwrap();
                let id = digest(&bytes);
                changed_root["functions"]["main"] = id.clone().into();
                pages.insert(id, Arc::new(bytes));
            } else {
                let old = root["frontend"].as_str().unwrap();
                let mut wire: serde_json::Value = serde_json::from_slice(&pages[old]).unwrap();
                if which == "source" {
                    wire["source"] = "forged source".into();
                } else {
                    wire["declarations"] = wire["declarations"]
                        .as_str()
                        .unwrap()
                        .replace(&environment.manifest().header_digest, &"0".repeat(64))
                        .into();
                }
                let bytes = serde_json::to_vec(&wire).unwrap();
                let id = digest(&bytes);
                changed_root["frontend"] = id.clone().into();
                pages.insert(id, Arc::new(bytes));
            }
            let bytes = serde_json::to_vec(&changed_root).unwrap();
            assert!(
                restore(&bytes, &digest(&bytes), &pages).is_err(),
                "{target:?} {which}"
            );
        }
        assert_eq!(session.snapshot().unwrap().digest, snapshot.digest);
    }
}

fn source_worker_query(
    deck: &serde_json::Value,
    store: &Path,
    leg: &str,
    tag: &str,
) -> serde_json::Value {
    use base64::Engine as _;
    let request = serde_json::json!({"recipe":ace_abi::worker_source::RECIPE,"nonce":tag,"request_hash":tag,
        "deck_b64":base64::engine::general_purpose::STANDARD.encode(serde_json::to_vec(deck).unwrap()),
        "leg":leg,"store_root":store,"steps":[{"op":"query"}]});
    let input = format!(
        "{}\n{}\n{}\n",
        serde_json::json!({"op":"begin","schema":"abiworker/1"}),
        serde_json::json!({"op":"request","request":request}),
        serde_json::json!({"op":"end"})
    );
    let run = ace_core::link::run_deadlined_typed(
        &mut std::process::Command::new(env!("CARGO_BIN_EXE_abi-worker")),
        std::time::Duration::from_secs(30),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
        input.into_bytes(),
    )
    .unwrap();
    let response: serde_json::Value = serde_json::from_slice(&run.stdout).unwrap();
    assert!(response.get("error").is_none(), "{response}");
    response
}

#[test]
fn source_foreign_formal_restart_first_error_repair_preserves_structured_diagnostics_and_pointer() {
    let root = scope("formal-first-error");
    let deck_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/ace-ca-d-source-938/src/add.atomdata");
    let deck: serde_json::Value =
        serde_json::from_slice(&std::fs::read(deck_path).unwrap()).unwrap();
    for leg in ["native", "c"] {
        let store = root.join(leg);
        let first = source_worker_query(&deck, &store, leg, "cold-938");
        assert_eq!(first["ok"]["source_import"]["actual_exit"], 42);
        let pointer = store.join(format!("source938-current-{leg}.json"));
        let original = std::fs::read(&pointer).unwrap();
        // Each query is a new process: these are actual first edits after R,
        // with no ensure-env, empty edit or successful warmup query.
        for (tag, source, stage) in [
            (
                "type",
                "fn main() int { return foreign_add(true,22) }",
                "resolve",
            ),
            (
                "syntax",
                "fn main() int { return foreign_add(20,22)",
                "parse",
            ),
        ] {
            let mut changed = deck.clone();
            changed["source"] = source.into();
            let response = source_worker_query(&changed, &store, leg, tag);
            let report = &response["ok"]["source_import"];
            assert_eq!(response["ok"]["steps"][0]["outcome"], "diags");
            assert_eq!(report["source_parse"], 1);
            assert_eq!(report["solver"], if tag == "type" { 1 } else { 0 });
            assert_eq!(report["diagnostic_stage"], stage);
            assert_eq!(report["diagnostics"][0]["stage"], stage);
            if tag == "type" {
                assert_eq!(
                    report["diagnostics"][0]["code"],
                    "source.foreign.type-mismatch"
                );
                let at = source.find("true").unwrap();
                assert_eq!(
                    report["diagnostics"][0]["span"],
                    serde_json::json!([at, at + 4])
                );
            }
            for field in ["hir", "mir", "native", "c_fragment"] {
                assert_eq!(report[field], 0);
            }
            assert!(report["actual_exit"].is_null());
            assert_eq!(report["actual_events"], serde_json::json!([]));
            assert_eq!(std::fs::read(&pointer).unwrap(), original);
        }
        let mut repaired = deck.clone();
        repaired["source"] = "fn main() int { let a=19\nreturn foreign_add(a,23) }".into();
        let response = source_worker_query(&repaired, &store, leg, "repaired-938");
        let report = &response["ok"]["source_import"];
        assert_eq!(report["actual_exit"], 42);
        assert_eq!(report["reference_exit"], 42);
        assert_eq!(report["source_parse"], 1);
        assert_eq!(report["solver"], 1);
        assert_eq!(report["hir"], 1);
        assert_ne!(std::fs::read(&pointer).unwrap(), original);
        let warm = source_worker_query(&repaired, &store, leg, "restored-after-repair-938");
        let report = &warm["ok"]["source_import"];
        for field in [
            "source_parse",
            "solver",
            "hir",
            "mir",
            "native",
            "c_fragment",
        ] {
            assert_eq!(report[field], 0);
        }
        assert_eq!(report["actual_exit"], 42);
        assert_eq!(report["reference_exit"], 42);
    }
}

#[test]
fn source_foreign_history_hits_before_lowering_and_uses_original_a2x_cache_after_restart() {
    use ace_abi::source_foreign::{Session, Target};
    use std::sync::Arc;
    let root = scope("target-history");
    let (object, environment) = provider(&root, 0);
    let bytes = Arc::new(std::fs::read(object).unwrap());
    let caps = vec![SOURCE_CAP.into()];
    let source = "fn independent() int { return 7 }\nfn main() int { return foreign_add(20,22) }";
    for target in [Target::Native, Target::C] {
        let dir = root.join(format!("{target:?}"));
        let store = ace_artifacts::store::ManagedStore::open(&dir).unwrap();
        let mut session = Session::cold_stored(
            source,
            "source938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            target,
            &store,
        )
        .unwrap();
        let a = session.artifact_bytes("main").unwrap().clone();
        let independent = session.artifact_bytes("independent").unwrap().clone();
        let edit = session
            .prepare("main", "fn main() int { return foreign_add(20,23) }")
            .unwrap();
        session.commit(edit).unwrap();
        let before = session.function_meter("main");
        let edit = session
            .prepare("main", "fn main() int { return foreign_add(20,22) }")
            .unwrap();
        session.commit(edit).unwrap();
        let after = session.function_meter("main");
        assert_eq!(after.mir_attempts, before.mir_attempts);
        assert_eq!(after.native_attempts, before.native_attempts);
        assert_eq!(after.c_attempts, before.c_attempts);
        assert_eq!(after.cache_hits, before.cache_hits + 1);
        assert!(Arc::ptr_eq(&a, session.artifact_bytes("main").unwrap()));
        assert!(Arc::ptr_eq(
            &independent,
            session.artifact_bytes("independent").unwrap()
        ));
        let edit = session
            .prepare("main", "fn main() int { return foreign_add(20,23) }")
            .unwrap();
        session.commit(edit).unwrap();
        let snapshot = session
            .snapshot()
            .unwrap()
            .publish(&store, "history-current", None)
            .unwrap();
        drop(session);
        let mut restored =
            Session::restore_stored(&store, &snapshot, target, &caps, environment.clone()).unwrap();
        assert_eq!(restored.frontend().query_counters().solver_runs, 0);
        let work = restored.frontend().work();
        assert!(work.rows().iter().all(|r| r.totals.p == 0));
        let nonreader = work
            .rows()
            .into_iter()
            .filter(|r| r.owner == "independent")
            .map(|r| (r.stage, r.totals))
            .collect::<Vec<_>>();
        let edit = restored
            .prepare("main", "fn main() int { return foreign_add(20,22) }")
            .unwrap();
        restored.commit(edit).unwrap();
        let meter = restored.function_meter("main");
        assert_eq!(
            meter.mir_attempts + meter.native_attempts + meter.c_attempts,
            0
        );
        assert_eq!(meter.cache_hits, 1);
        assert!(meter.cache_bytes_read > 0);
        assert_eq!(
            restored.artifact_bytes("main").unwrap().as_slice(),
            a.as_slice()
        );
        assert_eq!(restored.function_meter("independent"), Default::default());
        assert_eq!(
            nonreader,
            work.rows()
                .into_iter()
                .filter(|r| r.owner == "independent")
                .map(|r| (r.stage, r.totals))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            restored.run(&dir.join("restored-a")).unwrap().exit_code,
            Some(42)
        );
        assert_eq!(work.totals("*", "shared-assembly").p, 1);
        assert_eq!(work.totals("*", "compile-link-tool").p, 1);
        assert!(work.totals("*", "assembly-output").written_bytes > 0);
        if target == Target::C {
            // Remove only the new wrapper cache. AXCF1 remains the canonical
            // C fragment owner: B must still hit its original A2X store port.
            store.purge_layer("S938F").unwrap();
            let before = restored.function_meter("main");
            let edit = restored
                .prepare("main", "fn main() int { return foreign_add(20,23) }")
                .unwrap();
            restored.commit(edit).unwrap();
            let after = restored.function_meter("main");
            assert_eq!(after.c_attempts, before.c_attempts);
            assert_eq!(after.c_produced, before.c_produced);
            assert_eq!(after.cache_hits, before.cache_hits + 1);
            assert_eq!(
                restored.run(&dir.join("original-a2x-b")).unwrap().exit_code,
                Some(43)
            );
        }
    }
}

#[test]
fn source_foreign_work_tracks_target_discard_accept_and_unchanged_publication_payload_io() {
    use ace_abi::source_foreign::{Session, Target};
    use std::sync::Arc;
    let root = scope("work-disposition");
    let (object, environment) = provider(&root, 0);
    let bytes = Arc::new(std::fs::read(object).unwrap());
    let caps = vec![SOURCE_CAP.into()];
    let source = "fn independent() int { return 7 }\nfn main() int { return foreign_add(20,22) }";
    for target in [Target::Native, Target::C] {
        let store =
            ace_artifacts::store::ManagedStore::open(&root.join(format!("{target:?}"))).unwrap();
        let mut session = Session::cold_stored(
            source,
            "source938",
            "p938",
            &caps,
            environment.clone(),
            bytes.clone(),
            target,
            &store,
        )
        .unwrap();
        let before = session
            .snapshot()
            .unwrap()
            .publish(&store, "disposition", None)
            .unwrap();
        let work = session.frontend().work();
        let old = work.totals("main", "target-encode");
        let unchanged_io = work.totals("independent", "store-leaf");
        let dropped = session
            .prepare("main", "fn main() int { return foreign_add(21,22) }")
            .unwrap();
        let pending = work.totals("main", "target-encode");
        assert_eq!(pending.produced, old.produced + 1);
        assert_eq!(pending.pending, 1);
        assert_eq!(pending.accepted, old.accepted);
        drop(dropped);
        assert_eq!(
            work.totals("main", "target-encode").discarded,
            old.discarded + 1
        );
        assert_eq!(session.frontend().source(), source);
        let accepted = session
            .prepare("main", "fn main() int { return foreign_add(22,22) }")
            .unwrap();
        session.commit(accepted).unwrap();
        session
            .snapshot()
            .unwrap()
            .publish(&store, "disposition", Some(&before))
            .unwrap();
        let t = work.totals("main", "target-encode");
        assert_eq!(t.accepted, old.accepted + 1);
        assert_eq!(t.pending, 0);
        assert_eq!(t.accepted + t.discarded, t.produced);
        let io = work.totals("independent", "store-leaf");
        assert_eq!(
            (io.read_bytes, io.written_bytes),
            (unchanged_io.read_bytes, unchanged_io.written_bytes)
        );
        assert_eq!(io.reused, unchanged_io.reused + 2); // required facts and target pages
    }
}
