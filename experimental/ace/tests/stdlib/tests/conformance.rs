//! Real SDK evidence is deliberately integrated in an explicit test package,
//! keeping ordinary SDK compilation free of all ACE compiler/backend workers.
#![cfg(all(windows, target_arch = "x86_64", target_env = "msvc"))]
use ace_a2x_rust::RustTargetProvider;
use ace_a2x_store::modules::ModuleA2xTarget;
use ace_av::modules::{assembly_consumer, ModuleAvTarget};
use ace_core::source::{
    cst::FrontBudget,
    modules::{bind_modules, ModuleInput},
};
use ace_semantics_contract::assembly::{
    Declaration, Manifest, SemanticAssembly, Signature, SCALAR_MODULES,
};
use ace_source_query::{
    assembly::{
        compile_assembly_product, source_inputs, AssemblyHost, AssemblyTarget, ASSEMBLY_HOST,
        ASSEMBLY_PLATFORM,
    },
    modules::{ModuleSession, ModuleStore},
    SourceQuerySession,
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
const B: FrontBudget = FrontBudget::WORK_OWNED_64MIB;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        Self(
            std::env::var_os("AUTO_PLAN_ARTIFACTS")
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join(format!(
                    "stdlib944-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                )),
        )
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn rust_value(assembly: &ace_a2x_store::Assembly, dir: &Path, expected: i32) {
    std::fs::create_dir_all(dir).unwrap();
    for (p, s) in &assembly.files {
        std::fs::write(dir.join(p), s).unwrap();
    }
    let exe = dir.join("stdlib-module.exe");
    let mut cmd = Command::new(std::env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()));
    cmd.args(["--edition", "2021", "main.rs", "-o"])
        .arg(&exe)
        .current_dir(dir);
    let compile = auto_tool_execution::run_command_capture(
        cmd,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "{}",
        String::from_utf8_lossy(compile.stderr.as_deref().unwrap_or(&[]))
    );
    let run = auto_tool_execution::run_command_capture(
        Command::new(exe),
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .unwrap();
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    assert_eq!(
        String::from_utf8_lossy(run.stdout.as_deref().unwrap()).trim(),
        format!("value:{expected}")
    );
}
fn actual_semantics(inputs: &[ModuleInput], caps: &[String]) -> SemanticAssembly {
    let bound = bind_modules(inputs, "app", caps, B).unwrap();
    let proof = auto_lang_core::stdlib_assembly::ace::current_operation_reference(
        "io.read_line",
        &auto_lang_core::vm::native::NativeInterface::production(),
    )
    .unwrap();
    assert_eq!(proof.signature.returns, "str");
    assert_eq!(proof.requirements, ["stdin"]);
    assert_eq!(
        proof.verification,
        ace_semantics_contract::assembly::ReferenceLevel::SignatureChecked
    );
    let declarations = [("std", "answer"), ("app", "answer"), ("app", "main")]
        .into_iter()
        .map(|(module, symbol)| Declaration {
            module: module.into(),
            symbol: symbol.into(),
            semantic_version: 1,
            signature: Signature {
                parameters: vec![],
                modes: vec![],
                returns: "i32".into(),
            },
            requirements: vec![],
        })
        .collect();
    SemanticAssembly {
        profile: SCALAR_MODULES.into(),
        host: ASSEMBLY_HOST.into(),
        platform: ASSEMBLY_PLATFORM.into(),
        declarations,
        references: vec![proof],
        sources: source_inputs(&bound),
    }
}
#[test]
fn actual_standard_reference_enters_both_real_consumer_manifests_without_claiming_callable_support()
{
    let mut reference_identities = None;
    for local in [false, false, true] {
        let temp = Temp::new();
        let path = temp.0.join("store");
        let inputs = vec![
            ModuleInput {
                id: "std".into(),
                source: "pub fn answer() int { return 42 }".into(),
            },
            ModuleInput {
                id: "app".into(),
                source: format!(
                    "use std.answer\nfn answer() int {{ return 7 }} fn main() int {{ return {} }}",
                    if local { "answer()" } else { "std.answer()" }
                ),
            },
        ];
        let caps = vec![SCALAR_MODULES.into()];
        let semantic = actual_semantics(&inputs, &caps);
        let av_manifest = Manifest::freeze(semantic, assembly_consumer(vec![])).unwrap();
        let mut query = SourceQuerySession::new("stdlib944", "app");
        let product =
            compile_assembly_product(&inputs, "app", &caps, &av_manifest, &mut query, B).unwrap();
        let rust_manifest = Manifest::freeze(
            av_manifest.semantic().clone(),
            ModuleA2xTarget::consumer_for_product(&product, &RustTargetProvider::v1(), vec![])
                .unwrap(),
        )
        .unwrap();
        let identities = (
            av_manifest.fingerprint().to_string(),
            av_manifest.consumer_fingerprint().to_string(),
            rust_manifest.consumer_fingerprint().to_string(),
        );
        if !local {
            if let Some(previous) = &reference_identities {
                assert_eq!(
                    &identities, previous,
                    "legitimate distinct absolute stores must retain portable identities"
                );
            } else {
                reference_identities = Some(identities);
            }
        }
        assert!(!String::from_utf8(av_manifest.encode().unwrap())
            .unwrap()
            .contains("autostack/.wt"));
        for field in [
            "host",
            "platform",
            "producer",
            "abi",
            "runtime",
            "toolchain",
        ] {
            let mut semantic = av_manifest.semantic().clone();
            let mut a = av_manifest.consumer().clone();
            let mut r = rust_manifest.consumer().clone();
            match field {
                "host" => semantic.host = "unknown-current-host".into(),
                "platform" => semantic.platform = "unknown-platform".into(),
                "producer" => {
                    a.producer = ace_semantics_contract::assembly::RUST_PRODUCER.into();
                    r.producer = ace_semantics_contract::assembly::AV_PRODUCER.into();
                }
                "abi" => {
                    a.abi = "wrong-abi".into();
                    r.abi = "wrong-abi".into();
                }
                "runtime" => {
                    a.runtime = "wrong-runtime".into();
                    r.runtime = "wrong-runtime".into();
                }
                _ => {
                    a.toolchain = "wrong-recipe".into();
                    r.toolchain = "wrong-recipe".into();
                }
            }
            let bad_av = Manifest::freeze(semantic.clone(), a).unwrap();
            let bad_rust = Manifest::freeze(semantic, r).unwrap();
            assert!(
                ModuleAvTarget::cold_assembly(
                    &temp.0.join(format!("bad-{field}-av")),
                    &product,
                    &bad_av
                )
                .is_err(),
                "{field}"
            );
            assert!(
                ModuleA2xTarget::cold_assembly(
                    &temp.0.join(format!("bad-{field}-rust")),
                    "negative",
                    "pkg",
                    256 * 1024 * 1024,
                    RustTargetProvider::v1(),
                    &product,
                    &bad_rust
                )
                .is_err(),
                "{field}"
            );
        }
        let legacy = auto_lang_core::stdlib_assembly::manifest::AssemblyManifest::freeze(
            Default::default(),
            "legacy",
            Path::new("."),
            Path::new("stdlib"),
            &[],
            vec![],
        );
        assert!(Manifest::decode(&serde_json::to_vec(&legacy).unwrap()).is_err());
        assert!(
            auto_lang_core::stdlib_assembly::manifest::AssemblyManifest::decode_v4(
                &av_manifest.encode().unwrap()
            )
            .is_err()
        );
        let av = ModuleAvTarget::cold_assembly(&path, &product, &av_manifest).unwrap();
        let rust = ModuleA2xTarget::cold_assembly(
            &path,
            "stdlib944",
            "pkg",
            256 * 1024 * 1024,
            RustTargetProvider::v1(),
            &product,
            &rust_manifest,
        )
        .unwrap();
        let source = ModuleSession::from_product(&product, query).unwrap();
        let host = AssemblyHost::publish_initial(
            source,
            ModuleStore::open(&path, "stdlib944").unwrap(),
            av,
            rust,
        )
        .unwrap();
        let (av, rust) = host.targets();
        assert_eq!(
            av.current_manifest().unwrap().semantic().references[0].operation,
            "io.read_line"
        );
        assert_eq!(
            av.current_manifest().unwrap().semantic().references,
            rust.current_manifest().unwrap().semantic().references
        );
        assert_eq!(
            av.current_manifest().unwrap().fingerprint(),
            rust.current_manifest().unwrap().fingerprint()
        );
        let expected = if local { 7 } else { 42 };
        assert_eq!(
            av.run_authorized(&mut |_, req| {
                assert!(
                    req.is_empty(),
                    "unused audited IO reference is not a callable source operation or a grant"
                );
                Ok(())
            })
            .unwrap(),
            expected
        );
        rust.run_authorized(
            &mut |_, req| {
                assert!(req.is_empty());
                Ok(())
            },
            |a| {
                rust_value(a, &temp.0.join("rust"), expected);
                Ok(())
            },
        )
        .unwrap();
        // Re-admission consumes current actual evidence. A tampered reference
        // cannot pass by retaining the previously valid consumer fingerprints.
        let mut bad: serde_json::Value =
            serde_json::from_slice(&av_manifest.encode().unwrap()).unwrap();
        bad["semantic"]["references"][0]["signature"]["returns"] = "i32".into();
        assert!(Manifest::admit(&serde_json::to_vec(&bad).unwrap(), &av_manifest).is_err());
        let mut bad: serde_json::Value =
            serde_json::from_slice(&av_manifest.encode().unwrap()).unwrap();
        bad["semantic"]["references"][0]["verification"] = "executed".into();
        assert!(Manifest::decode(&serde_json::to_vec(&bad).unwrap())
            .unwrap_err()
            .contains("unknown variant"));
    }
}
