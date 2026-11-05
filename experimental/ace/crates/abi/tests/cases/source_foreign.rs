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
