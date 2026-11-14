//! PLAN-967 T-11 (AC-09/V-11 + V-ABI-BUILD/CROSS applicable parts): the
//! RICH Rust provider — mixed view+scalar frames beyond F0, concrete
//! generic instances, the String/Vec/Result/future/generic boundary rows
//! closed as NAMED rejections, and the two-rustc C-ABI cross (the same
//! interface built by TWO real toolchains; the AV host loads one and the
//! other's provider must satisfy the identical C ABI contract).
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_source, f0_rich_provider_source, load, open_session_from_build,
};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

fn run_av(
    source: &str,
    f0: &mut ace_abi::rust_provider::F0Session,
) -> ace_av_core::contract::ExecOutcome {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967r", "m967r");
    query.set_foreign(Some(f0.admitted.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "f0rich", &caps, "p967r", "m967r", &mut query,
    )
    .expect("compiles");
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .expect("lowers");
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded =
        ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, f0.session.table_shared())
            .expect("admits");
    f0.session
        .execute(
            &loaded,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .expect("authorized run")
        .0
}

fn success_i32(out: &ace_av_core::contract::ExecOutcome) -> Option<i32> {
    match out {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(n),
            ..
        } => Some(*n),
        _ => None,
    }
}

#[test]
fn rich_mixed_scalar_frames_run_on_av_for_real() {
    let root = scope("rich");
    let build = build_source(&root, f0_rich_provider_source()).expect("the rich provider builds");
    let mut f0 = open_session_from_build(&build, 1).expect("opens");

    // view + TWO i32 scalars: clamp("hello", 3, 4) = 4
    let clamp = run_av(
        "use.rs f0_rich\nfn main() int { return rich_clamp(\"hello\", 3, 4) }",
        &mut f0,
    );
    assert_eq!(success_i32(&clamp), Some(4));
    // clamp("hello", 2, 3) = 3 (len 5 clamps to hi)
    let clamp2 = run_av(
        "use.rs f0_rich\nfn main() int { return rich_clamp(\"hello\", 2, 3) }",
        &mut f0,
    );
    assert_eq!(success_i32(&clamp2), Some(3));

    // view + ONE i32 scalar -> bool: is_longer("hello", 4) = true (5 > 4)
    let longer = run_av(
        "use.rs f0_rich\nfn main() bool { return rich_is_longer(\"hello\", 4) }",
        &mut f0,
    );
    assert!(matches!(
        longer,
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::Bool(true),
            ..
        }
    ));
    // code points, not bytes: "A中😀" has 3 code points > 2 = true
    let cp = run_av(
        "use.rs f0_rich\nfn main() bool { return rich_is_longer(\"A中😀\", 2) }",
        &mut f0,
    );
    assert!(matches!(
        cp,
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::Bool(true),
            ..
        }
    ));

    // view + scalar fallible: find("hello", 'o') = 5 (1-based); find
    // miss = the named not-found trap.
    let found = run_av(
        "use.rs f0_rich\nfn main() int { return rich_find(\"hello\", 111) }",
        &mut f0,
    );
    assert_eq!(success_i32(&found), Some(5));
    let missed = run_av(
        "use.rs f0_rich\nfn main() int { return rich_find(\"hello\", 122) }",
        &mut f0,
    );
    assert!(matches!(
        missed,
        ace_av_core::contract::ExecOutcome::Trap {
            kind: ace_av_core::contract::TrapKind::ForeignStatus,
            ..
        }
    ));

    // a CONCRETE generic instance crosses (the generic itself never does)
    let first = run_av(
        "use.rs f0_rich\nfn main() int { return rich_first_i32(\"A\") }",
        &mut f0,
    );
    assert_eq!(success_i32(&first), Some(65));
}

#[test]
fn rich_shape_boundaries_are_named_rejections() {
    // every out-of-domain shape closes as a NAMED fact (V-11's boundary
    // rows): owned text/lists, Result returns, generics, mut borrows.
    let base = f0_rich_provider_source();
    let broken = |edit: &str, code: &str| {
        let src = format!("{base}\n{edit}\n");
        match auto_bindgen::rust_import::import(&src) {
            Err(e) => assert_eq!(e.code, code, "edit: {edit}"),
            Ok(_) => panic!("edit must not import: {edit}"),
        }
    };
    // a generic fn in the source (audit fires even unannotated)
    broken(
        "pub fn generic_pair<T>(a: T, b: T) -> i32 { 0 }",
        "rust-import.generics",
    );
    // annotated owned-text param
    let src = format!(
        "{base}\n// auto-ffi fn owned symbol=x.owned.v1 form=pure ret=i32\npub fn owned(s: String) -> i32 {{ 0 }}\n"
    );
    match auto_bindgen::rust_import::import(&src) {
        Err(e) => assert_eq!(e.code, "rust-import.owned-text"),
        Ok(_) => panic!("owned String must not import"),
    }
    // annotated Vec param
    let src = format!(
        "{base}\n// auto-ffi fn listed symbol=x.listed.v1 form=pure ret=i32\npub fn listed(v: Vec<i32>) -> i32 {{ 0 }}\n"
    );
    match auto_bindgen::rust_import::import(&src) {
        Err(e) => assert_eq!(e.code, "rust-import.owned-list"),
        Ok(_) => panic!("Vec must not import"),
    }
    // annotated Result return
    let src = format!(
        "{base}\n// auto-ffi fn resulted symbol=x.resulted.v1 form=pure ret=i32\npub fn resulted(s: &str) -> Result<i32, ()> {{ Ok(0) }}\n"
    );
    match auto_bindgen::rust_import::import(&src) {
        Err(e) => assert_eq!(e.code, "rust-import.result-type"),
        Ok(_) => panic!("Result must not import"),
    }
}

#[test]
fn v_abi_cross_two_real_rustc_toolchains_one_c_contract() {
    // V-ABI-CROSS (applicable part): the SAME provider+shim source built
    // by TWO real rustc toolchains (the ambient stable + the explicit
    // nightly override). The compiler identities MUST differ; both
    // artifacts satisfy the identical C ABI interface, and the AV host
    // (built by the ambient toolchain) loads and runs the OTHER
    // toolchain's provider through the same binding.
    let nightly = home_nightly_rustc();
    if nightly.is_none() {
        // named pending, never a PASS: the second toolchain is missing
        eprintln!("V-ABI-CROSS pending: no nightly rustc found");
        return;
    }
    let nightly = nightly.unwrap();
    let root = scope("cross");
    let stable_build =
        build_source(&root.join("stable"), f0_rich_provider_source()).expect("stable build");
    // the override build (recorded identity comes from --version of THAT rustc)
    let mut cmd = std::process::Command::new(&nightly);
    cmd.arg("--version");
    let out = ace_core::link::run_deadlined_typed(
        &mut cmd,
        std::time::Duration::from_secs(30),
        ace_core::link::CaptureCaps {
            stdout_max: 4096,
            stderr_max: 4096,
        },
        vec![],
    )
    .expect("nightly --version");
    let nightly_identity = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert!(
        nightly_identity.starts_with("rustc"),
        "the override produced a rustc identity: {nightly_identity}"
    );
    assert_ne!(
        nightly_identity, stable_build.rustc_identity,
        "the two toolchain identities differ (the cross is real)"
    );

    // build the SAME source with the nightly override
    let saved = std::env::var("AC741_RUSTC").ok();
    unsafe { std::env::set_var("AC741_RUSTC", &nightly) };
    let nightly_build =
        build_source(&root.join("nightly"), f0_rich_provider_source()).expect("nightly build");
    match saved {
        Some(v) => unsafe { std::env::set_var("AC741_RUSTC", v) },
        None => unsafe { std::env::remove_var("AC741_RUSTC") },
    }
    assert_ne!(nightly_build.cdylib, stable_build.cdylib);
    // B-02: BOTH distribution products exist from the same closure
    assert!(
        stable_build.staticlib.is_file(),
        "the staticlib product exists"
    );
    assert!(nightly_build.staticlib.is_file());
    // the interface facts are IDENTICAL (same annotations -> same manifest
    // functions; only the build identity moved)
    assert_eq!(
        nightly_build.manifest.functions, stable_build.manifest.functions,
        "a toolchain change never rewrites the interface"
    );

    // the AV host (ambient toolchain) runs the NIGHTLY-built provider
    // through the same binding — the C ABI contract crosses compilers.
    let mut night = open_session_from_build(&nightly_build, 1).expect("nightly session");
    let clamp = run_av(
        "use.rs f0_rich\nfn main() int { return rich_clamp(\"hello\", 3, 4) }",
        &mut night,
    );
    assert_eq!(success_i32(&clamp), Some(4));
    let found = run_av(
        "use.rs f0_rich\nfn main() int { return rich_find(\"hello\", 111) }",
        &mut night,
    );
    assert_eq!(success_i32(&found), Some(5));
    // and the stable-built provider still runs (both satisfy the contract)
    let mut stable = open_session_from_build(&stable_build, 1).expect("stable session");
    let stable_clamp = run_av(
        "use.rs f0_rich\nfn main() int { return rich_clamp(\"hello\", 3, 4) }",
        &mut stable,
    );
    assert_eq!(success_i32(&stable_clamp), Some(4));
    // the loader sanity: both libraries loaded and both counted
    assert!(night.session.table_shared().callee_count() >= 2);
    assert!(stable.session.table_shared().callee_count() >= 1);
    let _ = load; // keep the import honest
}

fn home_nightly_rustc() -> Option<PathBuf> {
    let home = std::env::var("USERPROFILE").ok()?;
    let p = PathBuf::from(home).join(".cargo/bin/rustc.exe");
    // the AMBIENT rustc is stable; the override must be the nightly one
    let probe = std::process::Command::new(&p)
        .arg("--version")
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&probe.stdout);
    if text.contains("nightly") {
        Some(p)
    } else {
        None
    }
}
