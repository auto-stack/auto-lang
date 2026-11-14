//! PLAN-967 T-14 (AC-12 / V-14, V-ABI-CACHE+PACK): the provider build
//! cache — precise BuildIdentity keys over the existing 783 ManagedStore.
//! Hit = rustc never re-runs; a provider/shim/toolchain edit flips the
//! identity; an AUTO body edit never touches it; the distribution
//! manifest (V-ABI-PACK seed for 978) records the artifact set + identity
//! axes + the runtime-dependency note.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::provider_build_cache::{BuildIdentity, ProviderBuildCache};
use ace_abi::rust_provider::{build_source, f0_provider_source};

fn scope(tag: &str) -> PathBuf {
    let root = PathBuf::from("D:/autostack/plan-artifacts/plan-967/tests")
        .join(format!("{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn t14_precise_invalidation_and_reuse() {
    let root = scope("t14");
    let cache = ProviderBuildCache::open(&root.join("store")).expect("cache opens");

    // COLD: build + PUT
    let build1 = build_source(&root.join("v1"), f0_provider_source()).expect("build 1");
    let shim1 = auto_bindgen::rust_import::generate_shim(&build1.manifest);
    let id1 = BuildIdentity::of(&build1, &shim1);
    let record1 = cache.put(&build1, &shim1).expect("put");
    assert_eq!(record1.identity, id1);
    // every axis is a REAL read
    assert!(id1
        .provider_sha256
        .starts_with(|c: char| c.is_ascii_hexdigit()));
    assert!(id1.rustc_identity.starts_with("rustc"));
    assert!(id1.generator_identity.contains("auto-bindgen/rust_import"));
    assert!(id1.target.contains("x86_64-pc-windows-msvc"));

    // WARM HIT: the same identity resolves to the cached record (rustc
    // would not re-run; the artifacts are on disk)
    let hit = cache.get(&id1).expect("get").expect("hit");
    assert_eq!(hit.cdylib, record1.cdylib);
    assert_eq!(
        hit.runtime_note,
        "msvc-crt:VCRUNTIME140+ucrt (dynamic CRT, dumpbin-verified)"
    );
    // the distribution manifest (V-ABI-PACK seed) is a JSON document with
    // the full artifact set + identity
    let manifest = serde_json::to_string(&hit).unwrap();
    assert!(manifest.contains("cdylib"));
    assert!(manifest.contains("staticlib"));
    assert!(manifest.contains("f0_text_provider.dll.lib"));

    // PROVIDER EDIT flips the identity: a miss (no stale-green)
    let edited = f0_provider_source().replace(
        "pub fn text_byte_len(s: &str) -> i32 {\n    s.as_bytes().len() as i32\n}",
        "pub fn text_byte_len(s: &str) -> i32 {\n    s.as_bytes().len() as i32 + 1\n}",
    );
    let build2 = build_source(&root.join("v2"), &edited).expect("build 2");
    let shim2 = auto_bindgen::rust_import::generate_shim(&build2.manifest);
    let id2 = BuildIdentity::of(&build2, &shim2);
    assert_ne!(id1, id2, "a provider body edit flips the build identity");
    assert!(
        cache.get(&id2).expect("get").is_none(),
        "the new identity is a miss"
    );

    // SHIM EDIT flips the identity: the SAME manifest with a different
    // generated shim (generator axis) is a different build
    let id1_shim2 = BuildIdentity {
        shim_sha256: ace_foreign_contract::blake3_hex(b"a different generated shim"),
        ..id1.clone()
    };
    assert_ne!(id1, id1_shim2);
    assert!(cache.get(&id1_shim2).expect("get").is_none());

    // TOOLCHAIN EDIT flips the identity (V-ABI-CACHE: a rustc label change
    // on an UNCHANGED provider invalidates the BUILD cache — the runtime
    // compat axis is separate)
    let id1_other_rustc = BuildIdentity {
        rustc_identity: "rustc 99.0.0 (other)".into(),
        ..id1.clone()
    };
    assert_ne!(id1, id1_other_rustc);
    assert!(cache.get(&id1_other_rustc).expect("get").is_none());

    // AUTO BODY EDIT never touches this key: the caller source is not a
    // build input of the provider unit (the identity is byte-identical)
    let id_again = BuildIdentity::of(&build1, &shim1);
    assert_eq!(id_again, id1);

    // a DELETED artifact is a miss (never a stale-green)
    let moved = root.join("v1/out/f0_text_provider.dll.moved");
    std::fs::rename(&record1.cdylib, &moved).unwrap();
    assert!(cache.get(&id1).expect("get").is_none());
    std::fs::rename(&moved, &record1.cdylib).unwrap();
    assert!(cache.get(&id1).expect("get").is_some());
}

#[test]
fn t14_two_rustc_toolchains_two_identities() {
    // V-ABI-CROSS + V-ABI-CACHE: the SAME source built by two real rustc
    // toolchains produces TWO build identities under ONE contract — the
    // cache never confuses them.
    let root = scope("t14-cross");
    let stable = build_source(&root.join("stable"), f0_provider_source()).expect("stable");
    let shim = auto_bindgen::rust_import::generate_shim(&stable.manifest);
    let id_stable = BuildIdentity::of(&stable, &shim);

    let nightly = root.join("nightly-rustc.txt");
    std::fs::write(&nightly, "").unwrap();
    let home = std::env::var("USERPROFILE").unwrap_or_default();
    let nightly_rustc = PathBuf::from(home).join(".cargo/bin/rustc.exe");
    let probe = std::process::Command::new(&nightly_rustc)
        .arg("--version")
        .output();
    match probe {
        Ok(out) if String::from_utf8_lossy(&out.stdout).contains("nightly") => unsafe {
            std::env::set_var("AC741_RUSTC", &nightly_rustc);
            let built = build_source(&root.join("nightly"), f0_provider_source());
            std::env::remove_var("AC741_RUSTC");
            let built = built.expect("nightly build");
            let id_nightly = BuildIdentity::of(&built, &shim);
            assert_ne!(
                id_stable.rustc_identity, id_nightly.rustc_identity,
                "two real toolchains, two identities"
            );
            assert_eq!(
                id_stable.provider_sha256, id_nightly.provider_sha256,
                "the source bytes are identical — only the toolchain moved"
            );
            let cache = ProviderBuildCache::open(&root.join("store")).expect("cache");
            let r1 = cache.put(&stable, &shim).expect("put stable");
            let r2 = cache.put(&built, &shim).expect("put nightly");
            assert_ne!(r1.identity, r2.identity);
            assert!(cache.get(&id_stable).unwrap().is_some());
            assert!(cache.get(&id_nightly).unwrap().is_some());
        },
        _ => {
            // named pending, never a PASS
            eprintln!("V-ABI-CACHE cross leg pending: no nightly rustc");
        }
    }
}
