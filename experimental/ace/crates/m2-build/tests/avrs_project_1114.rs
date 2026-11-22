//! `ace.m2.p1114.avrs-project` — T-05 (IF-ACE-AV-RS-STDLIB/1): the
//! AV/Rust STDLIB through the DELIVERED ordinary-project lane. The
//! composed `avrs/1` provider (every selected Rs layer of the dependency
//! -free core modules, ONE unit from selection to distribution) drives
//! `M2Build` build → run → SDK pack → RELOCATE → run-from-SDK: the same
//! public byte face (`A中😀\0` = 9 by UTF-8 bytes, NUL is content) answers
//! on the development run and from the moved SDK with the development
//! cache deleted — the distribution keeps the provider closure alive.
mod common;

use std::path::PathBuf;

use ace_m2_build::driver::{M2Build, ProjectInputs, RustProjectInputs};
use ace_m2_build::identity::TargetKind;

fn avrs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../stdlib/ace/avrs")
}

/// Compose the dependency-free core unit from the delivered tree (the
/// same composition the avrs session builds — `compose_avrs_provider`).
fn composed_core() -> String {
    let scratch = common::scratch("p1114-compose");
    for module in ["str", "math", "console", "file", "env", "time", "net"] {
        for suffix in ["at", "rs.at"] {
            let name = format!("{module}.{suffix}");
            let bytes = std::fs::read(avrs_dir().join(&name))
                .unwrap_or_else(|e| panic!("stage {name}: {e}"));
            std::fs::write(scratch.join(&name), bytes).unwrap();
        }
    }
    std::fs::write(scratch.join("main.at"), "fn main() int { return 0 }").unwrap();
    let mut ctx = ace_core::source::packages::PackageAssemblyContext::new(
        ace_core::source::packages::AssemblyBackend::Av,
        ace_core::source::packages::ProviderChoice::Kind(
            ace_core::source::packages::ProviderKind::Rs,
        ),
        ace_core::source::packages::TargetOs::Windows,
    );
    let (composed, inputs) =
        ace_abi::rust_provider::compose_avrs_provider(&scratch, "app::main", &mut ctx)
            .expect("the core selection composes");
    assert!(inputs.iter().any(|i| i.id == "str" && i.provider.is_some()));
    composed
}

/// The app: the ordinary single-module source over the composed
/// provider's PUBLIC names — the byte face plus a math and a rand-free
/// core witness: acc = ((9*100 + 20013)*10 + 7) = 209137.
const APP: &str = r#"use.rs avrs
fn main() int {
    let text = "A中😀\0"
    var acc = str_byte_len(text)
    acc = acc * 100 + char_at(text, 1)
    acc = acc * 10 + abs(0 - 7)
    return acc
}"#;

#[test]
fn p1114_avrs_project_builds_runs_and_sdk_relocates() {
    let root = common::scratch("p1114-avrs-project");
    let dev = root.join("dev");
    let sdk = root.join("sdk");
    let mut inputs = ProjectInputs::new("avrs-core", "main", "main.at", APP);
    inputs.caps = [
        "ac.source.language.av/v1",
        "foreign.scalar",
        "foreign.resource",
    ]
    .map(str::to_owned)
    .to_vec();
    inputs.rust_provider = Some(RustProjectInputs {
        source: composed_core(),
        cargo_lock: None,
    });

    // Development build + run: the REAL provider answers the byte face.
    let mut driver = M2Build::new(&dev);
    let project = driver
        .build(TargetKind::Av, &inputs)
        .expect("the avrs project builds");
    assert_eq!(
        project.run.as_ref().and_then(|r| r.exit_code),
        Some(209137),
        "the composed provider answers the public face"
    );

    // SDK pack → relocate → delete the development cache → run from the
    // moved SDK (the provider closure survives distribution).
    driver
        .build_sdk(&inputs, &[TargetKind::Av], &sdk, vec![], "run:avrs")
        .expect("pack SDK");
    drop(driver);
    std::fs::rename(&sdk, root.join("moved")).expect("relocate SDK");
    std::fs::remove_dir_all(&dev).expect("remove development cache");
    let run = M2Build::new(&root.join("consumer"))
        .run_sdk(&root.join("moved"), TargetKind::Av, "run:avrs")
        .expect("run from the relocated SDK");
    assert_eq!(run.exit_code, Some(209137));
    std::fs::remove_dir_all(&root).expect("remove test scratch");
}
