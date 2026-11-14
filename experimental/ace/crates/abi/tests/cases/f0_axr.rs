//! PLAN-967 T-12 (AXR0 / AC-10 first slice, V-AXR0-RUST): the A2R leg —
//! the frozen F0 domain bridges through the SAME C-compatible shim
//! exports: the generated Rust fileset declares the provider's
//! `#[no_mangle]` symbols and calls them through generated
//! `A2xResult` wrappers; a REAL rustc compiles the fileset linking the
//! provider's import library; the binary runs and the value crosses for
//! real. The old blanket `a2r.foreign-unsupported` stays for every
//! shape outside the frozen domain.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{build_f0, f0_authority};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

/// Compile a use.rs foreign source to the A2R fileset and rustc-run it
/// against the REAL provider, returning (exit, stdout).
fn a2r_compile_and_run(
    root: &std::path::Path,
    source: &str,
    build: &ace_abi::rust_provider::RustProviderBuild,
) -> (i32, String) {
    let authority = f0_authority(build, 1);
    let environment = build.manual.clone().admit(&authority).expect("admits");
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967a", "m967a");
    query.set_foreign(Some(environment));
    let product = ace_source_query::compile_text_record_product(
        source, "f0axr", &caps, "p967a", "m967a", &mut query,
    )
    .expect("compiles");
    let view = ace_a2x_core::SemanticView::from_checked(&product.compiled.checked).expect("view");
    let provider = ace_a2x_rust::RustTargetProvider::v1();
    use ace_a2x_core::target::{Family, TargetProvider};
    let family = Family::RecordText;
    let runtime = provider
        .runtime_source(&view, family)
        .expect("the AXR0 bridge admits the frozen domain");
    assert!(
        runtime.contains("extern \"C\""),
        "the bridge carries the extern block"
    );
    // one fragment per function (the entry `main` + the callees)
    let fns: Vec<&ace_a2x_core::FnFacts<'_>> = view.functions().iter().collect();
    let fragments = provider
        .produce_fragments(&view, &fns, family)
        .expect("lowers");
    let out_dir = root.join("fileset");
    std::fs::create_dir_all(&out_dir).unwrap();
    std::fs::write(out_dir.join("a2x_runtime.rs"), &runtime).unwrap();
    let mut generated = String::from("// AUTO-GENERATED (PLAN-967 AXR0) — DO NOT EDIT.\n");
    generated.push_str("use super::a2x_runtime::*;\n\n");
    for f in &fragments {
        generated.push_str(f);
        generated.push_str("\n\n");
    }
    std::fs::write(out_dir.join("generated.rs"), &generated).unwrap();
    let wrapper = "// AUTO-GENERATED (PLAN-967 AXR0) — DO NOT EDIT.\nmod a2x_runtime;\nmod generated;\n\nfn main() {\n    let outcome = generated::fn_main();\n    match outcome {\n        Ok(v) => println!(\"value:{}\", v),\n        Err(t) => println!(\"trap:{}\", t.name()),\n    }\n}\n";
    std::fs::write(out_dir.join("main.rs"), wrapper).unwrap();
    // the provider DLL rides beside the exe (loader); the import lib is
    // staged under the rustc-expected name (rustc appends .lib to
    // #[link(name)] — the MSVC product is <name>.dll.lib)
    std::fs::copy(&build.cdylib, out_dir.join("f0_text_provider.dll")).unwrap();
    std::fs::copy(
        build.cdylib.with_file_name("f0_text_provider.dll.lib"),
        out_dir.join("f0_text_provider.lib"),
    )
    .unwrap();
    let import_lib_dir = out_dir.clone();
    let exe = out_dir.join("axr0.exe");
    let mut cmd = std::process::Command::new("rustc");
    cmd.args([
        "--edition",
        "2021",
        "-O",
        "main.rs",
        "-o",
        exe.to_str().unwrap(),
    ]);
    cmd.arg("-L");
    cmd.arg(import_lib_dir.display().to_string());
    cmd.current_dir(&out_dir);
    let compiled = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(120),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .expect("rustc spawn");
    assert_eq!(
        compiled.exit_code,
        Some(0),
        "rustc compiled the fileset: {}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let mut run_cmd = std::process::Command::new(&exe);
    run_cmd.current_dir(&out_dir);
    let run = ace_core::link::run_deadlined_typed(
        &mut run_cmd,
        std::time::Duration::from_secs(30),
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 16,
            stderr_max: 1 << 16,
        },
        vec![],
    )
    .expect("run spawn");
    (
        run.exit_code.unwrap_or(-1),
        String::from_utf8_lossy(&run.stdout).into_owned(),
    )
}

#[test]
fn axr0_runs_the_same_binding_through_generated_rust() {
    let root = scope("axr0");
    let build = build_f0(&root).expect("the provider builds");

    // the SAME source shape the AV/native lanes run: byte_len of the
    // NUL-carrying view = 9; the empty view = 0.
    let (exit, out) = a2r_compile_and_run(
        &root,
        "use.rs f0_text\nfn main() int { return text_byte_len(\"A中😀\\0\") }",
        &build,
    );
    assert_eq!(exit, 0);
    assert!(out.contains("value:9"), "the value crossed for real: {out}");

    let (exit, out) = a2r_compile_and_run(
        &root.join("empty"),
        "use.rs f0_text\nfn main() int { return text_byte_len(\"\") }",
        &build,
    );
    assert_eq!(exit, 0);
    assert!(out.contains("value:0"), "{out}");

    // the fallible leg: success carries the out value; the empty input
    // crosses as the named Foreign trap (the status channel).
    let (exit, out) = a2r_compile_and_run(
        &root.join("fall"),
        "use.rs f0_text\nfn main() int { return text_first_byte(\"Abc\") }",
        &build,
    );
    assert_eq!(exit, 0);
    assert!(out.contains("value:65"), "{out}");
    let (exit, out) = a2r_compile_and_run(
        &root.join("miss"),
        "use.rs f0_text\nfn main() int { return text_first_byte(\"\") }",
        &build,
    );
    assert_eq!(exit, 0);
    assert!(
        out.contains("trap:Foreign"),
        "the named status crossed: {out}"
    );
}

#[test]
fn axr0_keeps_the_named_rejection_outside_the_frozen_domain() {
    // a resource lane outside the bridge domain (a record view) keeps the
    // historical named rejection — the blanket lift never widens.
    let root = scope("axr0-reject");
    let build = build_f0(&root).expect("build");
    let authority = f0_authority(&build, 1);
    let environment = build.manual.clone().admit(&authority).unwrap();
    // a manifest with a record-view entry (outside the bridge domain)
    let mut wider = build.manual.clone();
    use ace_foreign_contract::resource::{
        FieldKind, ResKind, ResOut, ResourceForm, ResourceSig, ViewRole, RESOURCE_SIG_FORMAT,
    };
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
    wider
        .validate()
        .expect("the wider manifest is self-consistent");
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let _ = environment;
    let mut query = ace_source_query::SourceQuerySession::new("p967a", "m967a");
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
        "f0axr",
        &caps,
        "p967a",
        "m967a",
        &mut query,
    )
    .expect("compiles");
    let view = ace_a2x_core::SemanticView::from_checked(&product.compiled.checked).unwrap();
    use ace_a2x_core::target::{Family, TargetProvider};
    let rejected = ace_a2x_rust::RustTargetProvider::v1()
        .runtime_source(&view, Family::RecordText)
        .unwrap_err();
    assert_eq!(rejected.code, "a2r.foreign-unsupported");
}
