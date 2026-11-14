//! PLAN-967 T-05 F2 (AC-LOAD / V-LOAD): the FFI provider consumes the
//! DELIVERED 1043 source-suffix assembly — a public `.at` adapter plus
//! a `.rs.at` provider implementation compose one logical module; the
//! selected Rs layer feeds the same auto binding/shim/build chain and
//! the adapter runs through the AV lane with REAL foreign calls. The
//! assembly's own named rejections (missing/duplicate/wrong-suffix)
//! propagate; selected-input edits rebuild while unselected siblings
//! leave the provider identity untouched.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::open_assembled_rs_provider_session;
use ace_core::source::packages::{
    AssemblyBackend, PackageAssemblyContext, ProviderChoice, TargetOs,
};

fn scope(tag: &str) -> Scope {
    let root = std::env::temp_dir().join(format!("p967-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

/// T-05 F3: transient scratch under the OS temp dir, removed on drop.
struct Scope(PathBuf);
impl std::ops::Deref for Scope {
    type Target = std::path::Path;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The public Auto adapter (the SAME adapter source for every backend —
/// only the implementation suffix moves).
const ADAPTER: &str = "use.rs p967_load\nfn main() int { return asm_len(\"abcd\") }";

/// The Rust provider implementation (`.rs.at` suffix): the ordinary
/// annotated F0 provider source.
const RS_IMPL: &str = "//! auto-ffi provider p967_load/1 target=x86_64-pc-windows-msvc profile=p967-load.v1\n// auto-ffi fn asm_len symbol=p967.load.len.v1 form=pure ret=i32\npub fn asm_len(s: &str) -> i32 {\n    s.as_bytes().len() as i32\n}\n";

/// A `.py.at` sibling (an UNSELECTED candidate on the AV backend).
const PY_IMPL: &str = "# python adapter source (unselected on the av backend)\n";

fn run_adapter(
    adapter: &str,
    session: &mut ace_abi::rust_provider::F0Session,
) -> ace_av_core::contract::ExecOutcome {
    let environment = session.admitted.clone();
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967ld", "m967ld");
    query.set_foreign(Some(environment));
    let product = ace_source_query::compile_text_record_product(
        adapter, "f0ld", &caps, "p967ld", "m967ld", &mut query,
    )
    .expect("compiles");
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(adapter.as_bytes()),
    )
    .expect("lowers");
    ace_av_core::mir_verify::verify_mir(&mir).expect("mir");
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(
        decoded,
        &mir.owner,
        session.session.table_shared(),
    )
    .expect("admits");
    session
        .session
        .execute(
            &loaded,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .expect("authorized run")
        .0
}

#[test]
fn v_load_assembled_rs_provider_drives_real_foreign_calls() {
    let root = scope("load");
    let proj = root.join("proj");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("p967_load.at"), ADAPTER).unwrap();
    std::fs::write(proj.join("p967_load.rs.at"), RS_IMPL).unwrap();

    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ace_core::source::packages::ProviderKind::Rs),
        TargetOs::Windows,
    );
    let (mut session, adapter) =
        open_assembled_rs_provider_session(&proj, "app::p967_load", &mut ctx, 1)
            .expect("the assembled provider session opens");
    assert_eq!(adapter, ADAPTER, "the public adapter is the composed face");

    // the adapter runs through the AV lane — REAL foreign dispatch from
    // the suffix-assembled provider (no av.rs loader anywhere)
    let out = run_adapter(&adapter, &mut session);
    assert_eq!(
        match out {
            ace_av_core::contract::ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(v),
                ..
            } => v,
            other => panic!("expected the real value, got {other:?}"),
        },
        4
    );
    assert!(session.session.table_shared().callee_count() >= 1);
    let digest_before = session.build.provider_digest.clone();

    // ---- SELECTED-input edit: the provider body moves -> a new
    // identity rebuild through the SAME entry (recovery)
    drop(session);
    let edited = RS_IMPL.replace("s.as_bytes().len() as i32", "s.as_bytes().len() as i32 + 1");
    std::fs::write(proj.join("p967_load.rs.at"), &edited).unwrap();
    let mut ctx2 = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ace_core::source::packages::ProviderKind::Rs),
        TargetOs::Windows,
    );
    let (mut session2, adapter2) =
        open_assembled_rs_provider_session(&proj, "app::p967_load", &mut ctx2, 2)
            .expect("the edited provider rebuilds");
    assert_eq!(adapter2, ADAPTER);
    assert_ne!(
        session2.build.provider_digest, digest_before,
        "a selected-input edit flips the provider identity"
    );
    let out = run_adapter(&adapter2, &mut session2);
    assert!(matches!(
        out,
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(5),
            ..
        }
    ));
    drop(session2);

    // ---- UNSELECTED-input edit: a py sibling joins the directory; the
    // AV configuration still selects the Rs layer, the provider source
    // (and digest) is untouched, the same entry keeps answering
    std::fs::write(proj.join("p967_load.py.at"), PY_IMPL).unwrap();
    let mut ctx3 = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ace_core::source::packages::ProviderKind::Rs),
        TargetOs::Windows,
    );
    let (mut session3, _) =
        open_assembled_rs_provider_session(&proj, "app::p967_load", &mut ctx3, 3)
            .expect("the unselected sibling changes nothing for the rs lane");
    assert_eq!(
        session3.build.provider_digest,
        auto_bindgen::rust_import::provider_digest(&edited),
        "the unselected sibling leaves the provider identity untouched"
    );
    let out = run_adapter(ADAPTER, &mut session3);
    assert!(matches!(
        out,
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(5),
            ..
        }
    ));
}

#[test]
fn v_load_named_rejections_missing_duplicate_and_wrong_suffix() {
    let root = scope("load-neg");

    // MISSING implementation: the public .at alone never binds an FFI
    // provider for this configuration
    let proj = root.join("missing");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("p967_load.at"), ADAPTER).unwrap();
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ace_core::source::packages::ProviderKind::Rs),
        TargetOs::Windows,
    );
    let err = match open_assembled_rs_provider_session(&proj, "app::p967_load", &mut ctx, 1) {
        Err(e) => e,
        Ok(_) => panic!("a module without an implementation must reject"),
    };
    assert!(err.contains("load.no-provider-impl"), "{err}");

    // WRONG suffix: only a py candidate on the AV backend — the
    // assembly's own named selection error (never a silent fallback)
    let proj = root.join("wrong-suffix");
    std::fs::create_dir_all(&proj).unwrap();
    std::fs::write(proj.join("p967_load.at"), ADAPTER).unwrap();
    std::fs::write(proj.join("p967_load.py.at"), PY_IMPL).unwrap();
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ace_core::source::packages::ProviderKind::Rs),
        TargetOs::Windows,
    );
    let err = match open_assembled_rs_provider_session(&proj, "app::p967_load", &mut ctx, 1) {
        Err(e) => e,
        Ok(_) => panic!("a wrong-suffix candidate must reject"),
    };
    assert!(
        err.contains("load.assembly") || err.contains("assembly."),
        "the assembly's own named error: {err}"
    );

    // DUPLICATE implementation: two rs candidates for one module (the
    // files closure is synthetic — a real dir cannot hold duplicates)
    let files = vec![
        ("p967_load.at".to_string(), ADAPTER.to_string()),
        ("p967_load.rs.at".to_string(), RS_IMPL.to_string()),
        ("p967_load.rs.at".to_string(), RS_IMPL.to_string()),
    ];
    let mut ctx = PackageAssemblyContext::new(
        AssemblyBackend::Av,
        ProviderChoice::Kind(ace_core::source::packages::ProviderKind::Rs),
        TargetOs::Windows,
    );
    let err = match ace_source_query::modules::packages::assemble_package_inputs(
        &files,
        "app::p967_load",
        &mut ctx,
    ) {
        Err(e) => e,
        Ok(_) => panic!("duplicate candidates must reject"),
    };
    assert!(err.contains("duplicate"), "{err}");
}
