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
    let header = std::fs::read(fixture().join("source_foreign938.h")).unwrap();
    let table = std::fs::read(fixture().join("source_foreign938.contract")).unwrap();
    std::fs::write(root.join("source_foreign938.h"), &header).unwrap();
    let source = std::fs::read_to_string(fixture().join("source_foreign938.c"))
        .unwrap()
        .replace("#define BODY_BIAS 0", &format!("#define BODY_BIAS {bias}"));
    let source_path = root.join("provider.c");
    let object = root.join("provider.obj");
    std::fs::write(&source_path, source).unwrap();
    let deps = ace_abi::ctool::compile_c_with_deps(&source_path, root, &object).unwrap();
    assert!(deps
        .iter()
        .any(|p| p.file_name().is_some_and(|n| n == "source_foreign938.h")));
    let m = ManualManifest {
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
    let authority = HostAuthority {
        review_id: m.review_id.clone(),
        contract_digest: m.contract_digest().unwrap(),
        provider: m.provider.clone(),
        provider_digest: hash(&std::fs::read(&object).unwrap()),
        header_digest: m.header_digest.clone(),
        physical_table_digest: m.physical_table_digest.clone(),
        target: m.target.clone(),
        capabilities: [SOURCE_CAP.into()].into_iter().collect(),
        available_symbols: ["provider_add".into()].into_iter().collect(),
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
