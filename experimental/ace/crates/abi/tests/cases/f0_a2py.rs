//! PLAN-967 T-12 (V-COVERAGE A2PY): the Python leg — the generated
//! Python loads the SAME provider cdylib through ctypes and the value
//! crosses for real; the frozen-domain guard keeps the named rejection
//! for wider shapes.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{build_f0, f0_authority};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn emit_and_run(
    root: &std::path::Path,
    source: &str,
    build: &ace_abi::rust_provider::RustProviderBuild,
) -> (i32, String) {
    let authority = f0_authority(build, 1);
    let environment = build.manual.clone().admit(&authority).unwrap();
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967p", "m967p");
    query.set_foreign(Some(environment));
    let product = ace_source_query::compile_text_record_product(
        source, "f0a2py", &caps, "p967p", "m967p", &mut query,
    )
    .expect("compiles");
    let view = ace_a2x_core::SemanticView::from_checked(&product.compiled.checked).unwrap();
    use ace_a2x_core::target::{Family, TargetProvider};
    let provider = ace_a2x_python::PythonTargetProvider::default();
    let family = Family::RecordText;
    let runtime = provider
        .runtime_source(&view, family)
        .expect("the ctypes bridge admits the frozen domain");
    assert!(runtime.contains("_A2X_F0_DLL"), "the bridge loads the DLL");
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let fragments = provider
        .produce_fragments(&view, &fns, family)
        .expect("lowers");
    let dir = root.join("py");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("a2x_runtime.py"), &runtime).unwrap();
    let mut generated = String::from(
        "# AUTO-GENERATED (PLAN-967 A2PY) — DO NOT EDIT.\nfrom a2x_runtime import *\n\n",
    );
    for f in &fragments {
        generated.push_str(f);
        generated.push_str("\n\n");
    }
    generated.push_str("if __name__ == \"__main__\":\n    a2x_run_main(a2x_fn_main)\n");
    std::fs::write(dir.join("generated.py"), &generated).unwrap();
    // the provider DLL rides beside (ctypes CDLL resolves from CWD)
    std::fs::copy(&build.cdylib, dir.join("f0_text_provider.dll")).unwrap();
    let mut cmd = std::process::Command::new("python");
    cmd.arg("-B").arg("generated.py").current_dir(&dir);
    let run = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(60),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .expect("python spawn");
    // combined streams: the fallible-miss leg raises a typed exception
    // (its traceback is part of the observable, not a harness failure)
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    (run.exit_code.unwrap_or(-1), combined)
}

#[test]
fn a2py_runs_the_same_provider_through_ctypes() {
    let root = scope("a2py");
    let build = build_f0(&root).expect("the provider builds");

    // byte_len via ctypes: the NUL-carrying view = 9; empty = 0.
    let (exit, out) = emit_and_run(
        &root,
        "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }",
        &build,
    );
    assert_eq!(exit, 0, "python ran: {}", out);
    assert!(out.contains("9"), "the value crossed for real: {out}");

    let (exit, out) = emit_and_run(
        &root.join("empty"),
        "use.rs f0_text\nfn main() int { return text_byte_len(\"\") }",
        &build,
    );
    assert_eq!(exit, 0);
    assert!(out.contains('0'), "{out}");

    // the fallible leg: success carries 65; the empty input crosses as
    // the typed Foreign status (the exception channel).
    let (exit, out) = emit_and_run(
        &root.join("fall"),
        "use.rs f0_text\nfn main() int { return text_first_byte(\"Abc\") }",
        &build,
    );
    assert_eq!(exit, 0);
    assert!(out.contains("65"), "{out}");
    let (exit, out) = emit_and_run(
        &root.join("miss"),
        "use.rs f0_text\nfn main() int { return text_first_byte(\"\") }",
        &build,
    );
    // the typed exception propagates as a non-zero exit with the status
    // in the traceback — the Python failure channel
    assert_ne!(exit, 0);
    assert!(
        out.contains("foreign.status=-1"),
        "the typed status crossed: {out}"
    );
}

#[test]
fn a2py_keeps_the_named_rejection_outside_the_frozen_domain() {
    // a record-view manifest keeps the historical named rejection.
    let root = scope("a2py-reject");
    let build = build_f0(&root).expect("build");
    use ace_foreign_contract::resource::{
        FieldKind, ResKind, ResOut, ResourceForm, ResourceSig, ViewRole, RESOURCE_SIG_FORMAT,
    };
    let mut wider = build.manual.clone();
    wider.facts.functions.push(ace_foreign_contract::FnFact {
        symbol: "f0_record_view".into(),
        form: ace_foreign_contract::FnForm::Pure {
            params: vec![],
            ret: ace_foreign_contract::LaneKind::I32,
        },
        error_model: None,
        semantic_ref: None,
        resource: Some(ResourceSig {
            format: RESOURCE_SIG_FORMAT.into(),
            params: vec![ResKind::Rec {
                rec: "P".into(),
                fields: vec![("x".into(), FieldKind::I32)],
                role: ViewRole::Const,
            }],
            form: ResourceForm::Pure { ret: ResOut::I32 },
            atomic: false,
            noescape: true,
            typed_peer: false,
        }),
    });
    wider
        .declarations
        .push(ace_foreign_contract::manual::ManualDecl {
            context_symbol: None,
            source_name: "f0_record_view".into(),
            symbol: "f0_record_view".into(),
        });
    wider.validate().expect("self-consistent");
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967p", "m967p");
    query.set_foreign(Some(
        wider
            .clone()
            .admit(&ace_foreign_contract::manual::HostAuthority {
                review_id: wider.review_id.clone(),
                contract_digest: wider.contract_digest().unwrap(),
                provider: wider.provider.clone(),
                provider_digest: "0".repeat(64),
                header_digest: wider.header_digest.clone(),
                physical_table_digest: wider.physical_table_digest.clone(),
                target: wider.target.clone(),
                capabilities: wider.facts.capabilities.iter().cloned().collect(),
                available_symbols: wider
                    .facts
                    .functions
                    .iter()
                    .map(|f| f.symbol.clone())
                    .collect(),
                binding_generation: 1,
            })
            .unwrap(),
    ));
    let product = ace_source_query::compile_text_record_product(
        "use.rs f0_text\ntype P { x int }\nfn main() int { return 0 }",
        "f0a2py",
        &caps,
        "p967p",
        "m967p",
        &mut query,
    )
    .expect("compiles");
    let view = ace_a2x_core::SemanticView::from_checked(&product.compiled.checked).unwrap();
    use ace_a2x_core::target::TargetProvider;
    let rejected = ace_a2x_python::PythonTargetProvider::default()
        .validate_view(&view)
        .unwrap_err();
    assert_eq!(rejected.code, "a2py.foreign-authority");
}
