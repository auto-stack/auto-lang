//! PLAN-967 T-05 F5 (AC-ABI-COMPAT remainder): the V-ABI-TARGET /
//! V-ABI-IFACE / V-ABI-LIFE legs deliverable on Windows —
//! * TARGET: the manifest target cross-checked against the REAL PE
//!   machine field of the produced artifact; a wrong-target annotation
//!   is the frozen named rejection;
//! * IFACE: per-axis named rejections (ghost symbol / form flip /
//!   required-set break) AND the optional-increment re-admission (an
//!   old caller runs against a superset provider v2; identities differ);
//! * LIFE: v1/v2 providers coexist as independent loaded units; a
//!   v1-created opaque reference does NOT cross to v2 (named rejection,
//!   no dispatch of the wrong generation's object); each frees cleanly.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{
    build_source, f0_provider_source, f0_tframe_provider_source,
    open_session_from_build,
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

/// The REAL PE machine field of a produced artifact (cross-checks the
/// manifest target against the artifact header — not a trust-the-label).
fn pe_machine_of(path: &std::path::Path) -> u16 {
    let bytes = std::fs::read(path).expect("artifact readable");
    let pe_off = u32::from_le_bytes([bytes[0x3c], bytes[0x3d], bytes[0x3e], bytes[0x3f]]) as usize;
    assert_eq!(&bytes[pe_off..pe_off + 4], b"PE\0\0", "the PE signature");
    u16::from_le_bytes([bytes[pe_off + 4], bytes[pe_off + 5]])
}

const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;

#[test]
fn v_abi_target_artifact_cross_check_and_wrong_target_rejects() {
    let root = scope("abi-target");
    let build = build_source(&root, f0_provider_source()).expect("build");
    // the manifest declares x86_64-pc-windows-msvc; the artifact's OWN
    // header says AMD64 — the cross-check closes on the real bytes.
    // (The cdylib is the PE artifact; the staticlib is an ar archive —
    // checked by its archive magic, not a PE header.)
    assert!(build.manifest.target.contains("x86_64-pc-windows"));
    assert_eq!(
        pe_machine_of(&build.cdylib),
        IMAGE_FILE_MACHINE_AMD64,
        "the cdylib machine field matches the declared target"
    );
    let slib = std::fs::read(&build.staticlib).expect("staticlib readable");
    assert!(
        slib.starts_with(b"!<arch>"),
        "the staticlib is the native-relink archive form"
    );

    // a wrong-target annotation is the FROZEN named rejection (never a
    // reinterpretation): the projection pins the shared target constant
    let wrong = f0_provider_source().replace(
        "target=x86_64-pc-windows-msvc",
        "target=aarch64-pc-windows-msvc",
    );
    let err = match build_source(&root.join("wrong-target"), &wrong) {
        Err(e) => e,
        Ok(_) => panic!("a wrong-target annotation must reject"),
    };
    assert!(
        err.contains("not the frozen F0 target") || err.contains("rust-import"),
        "named: {err}"
    );
}

#[test]
fn v_abi_iface_axes_named_and_optional_increment_re_admits() {
    let root = scope("abi-iface");

    // ---- per-axis named rejections
    // ghost symbol: the annotation describes a fn the source lacks
    let ghost = format!(
        "{}\n// auto-ffi fn ghost_fn symbol=f0.ghost.v1 form=pure ret=i32\n",
        f0_provider_source()
    );
    let err = match build_source(&root.join("ghost"), &ghost) {
        Err(e) => e,
        Ok(_) => panic!("a ghost symbol must reject"),
    };
    assert!(err.contains("rust-import.domain"), "{err}");

    // form flip: the annotation says bool, the signature says i32
    let flipped = f0_provider_source().replace(
        "// auto-ffi fn text_is_ascii symbol=f0.text.is_ascii.v1 form=pure ret=bool",
        "// auto-ffi fn text_is_ascii symbol=f0.text.is_ascii.v1 form=pure ret=i32",
    );
    let err = match build_source(&root.join("flip"), &flipped) {
        Err(e) => e,
        Ok(_) => panic!("a form flip must reject"),
    };
    assert!(err.contains("rust-import.domain"), "{err}");

    // ---- optional increment: v2 = v1 + one new fn
    let v1 = f0_provider_source();
    let v2 = format!(
        "{v1}\n// auto-ffi fn text_tail symbol=f0.text.tail.v1 form=pure ret=i32\npub fn text_tail(s: &str) -> i32 {{ s.as_bytes().len() as i32 + 100 }}\n"
    );
    let build1 = build_source(&root.join("v1"), &v1).expect("v1");
    let build2 = build_source(&root.join("v2"), &v2).expect("v2");
    // the identities differ (provider bytes moved) — no stale hit
    assert_ne!(build1.provider_digest, build2.provider_digest);
    // v2's REQUIRED set is a SUPERSET: every v1 symbol exists in v2
    let v1_syms: Vec<_> = build1
        .manifest
        .functions
        .iter()
        .map(|f| f.export.clone())
        .collect();
    let v2_syms: Vec<_> = build2
        .manifest
        .functions
        .iter()
        .map(|f| f.export.clone())
        .collect();
    for s in &v1_syms {
        assert!(v2_syms.contains(s), "v2 keeps the v1 required subset ({s})");
    }
    assert!(v2_syms.contains(&"f0_text_tail_v1".to_string()));

    // the OLD caller (the v1 Auto source) runs against the v2 provider
    // session — re-admission under the explicit optional-increment rule
    let mut session2 = open_session_from_build(&build2, 1).expect("v2 session");
    let old_caller = "use.rs f0_text\nfn main() int { return text_byte_len(\"abc\") }".to_string();
    let out = run_av(&old_caller, &mut session2);
    assert_eq!(success_i32(&out), Some(3), "the old caller re-admits on v2");
    // and the NEW symbol is callable in the same session
    let new_caller = "use.rs f0_text\nfn main() int { return text_tail(\"abc\") }".to_string();
    let out = run_av(&new_caller, &mut session2);
    assert_eq!(success_i32(&out), Some(103), "the optional symbol answers");
    drop(session2);

    // ---- required-set BREAK: v3 drops a required fn; the old caller's
    // declaration no longer resolves — the named frontdoor rejection
    let v3 = f0_provider_source().replace(
        "// auto-ffi fn text_is_ascii symbol=f0.text.is_ascii.v1 form=pure ret=bool\npub fn text_is_ascii(s: &str) -> bool {\n    s.is_ascii()\n}\n",
        "",
    );
    let build3 =
        build_source(&root.join("v3"), &v3).expect("v3 builds (its own callers may use it)");
    let mut session3 = open_session_from_build(&build3, 1).expect("v3 session");
    let ascii_caller =
        "use.rs f0_text\nfn main() bool { return text_is_ascii(\"abc\") }".to_string();
    let err = match run_av_raw(&ascii_caller, &mut session3) {
        Err(e) => e,
        Ok(_) => panic!("a removed required symbol must not resolve"),
    };
    assert!(
        err.contains("source.undefined") || err.contains("no function named"),
        "the named unresolved-symbol rejection: {err}"
    );
}

#[test]
fn v_abi_life_v1_v2_coexist_and_opaque_refs_never_cross() {
    let root = scope("abi-life");
    // v1: the frozen tframe provider; v2: a body edit (the opaque seed
    // round-trips +1) — same interface facts, different build identity
    let v1 = f0_tframe_provider_source();
    let v2 = v1.replace(
        "    r.insert(id, seed);\n    Some(id as i32)",
        "    r.insert(id, seed + 1);\n    Some(id as i32)",
    );
    assert_ne!(v1, v2);
    let build1 = build_source(&root.join("v1"), &v1).expect("v1");
    let build2 = build_source(&root.join("v2"), &v2).expect("v2");

    // BOTH providers loaded SIMULTANEOUSLY as independent units (each
    // DLL pins its own registry — the provider owns its objects)
    let mut s1 = open_session_from_build(&build1, 1).expect("v1 session");
    let mut s2 = open_session_from_build(&build2, 2).expect("v2 session");

    // create an opaque object through v1
    let store = "use.rs f0_tframe\nfn main() int { return tframe_opaque_store(F1) }";
    let f = frame1("41");
    let out = run_av(&store.replace("F1", &auto_lit(&f)), &mut s1);
    let _id = success_i32(&out).expect("v1 stores the object");
    // v1 reads its own object back
    let f = frame_read("1");
    let out = run_av(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
            auto_lit(&f)
        ),
        &mut s1,
    );
    assert_eq!(success_i32(&out), Some(41), "v1 owns its object");

    // the SAME handle handed to v2 is a NAMED rejection — v2's registry
    // never saw it (opaque refs do not cross provider generations); the
    // call is a fallible named status, NOT a silent reinterpretation
    let out = run_av(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
            auto_lit(&f)
        ),
        &mut s2,
    );
    match out {
        ace_av_core::contract::ExecOutcome::Trap {
            kind: ace_av_core::contract::TrapKind::ForeignStatus,
            ..
        } => {}
        other => panic!("v2 must reject the v1 object by name: {other:?}"),
    }
    // v2's OWN object answers with the +1 body (both generations live)
    let f = frame1("41");
    let out = run_av(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_store({}) }}",
            auto_lit(&f)
        ),
        &mut s2,
    );
    let id2 = success_i32(&out).expect("v2 stores its own object");
    assert_eq!(id2, 1, "v2's registry is its own (fresh id space)");
    let f = frame_read("1");
    let out = run_av(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
            auto_lit(&f)
        ),
        &mut s2,
    );
    assert_eq!(
        success_i32(&out),
        Some(42),
        "v2 answers with the edited body"
    );

    // release v1 (its library frees); v2 keeps answering (its unit is
    // independently pinned) — the coexistence ends cleanly per owner
    drop(s1);
    let f = frame_read("1");
    let out = run_av(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
            auto_lit(&f)
        ),
        &mut s2,
    );
    assert_eq!(success_i32(&out), Some(42), "v2 survives v1's exit");
    drop(s2);

    // a FRESH v1 load works after the exit (no corrupted state)
    let mut s1b = open_session_from_build(&build1, 3).expect("v1 reloads");
    let f = frame1("7");
    let out = run_av(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_store({}) }}",
            auto_lit(&f)
        ),
        &mut s1b,
    );
    assert_eq!(success_i32(&out), Some(1), "the reloaded v1 answers");
}

// ---- helpers (the delivered AV pipeline over an open session)

fn frame1(seed: &str) -> String {
    format!(
        "p967.frame/1\nr n 0\n\np 1\ni {}\n{seed}\nk 0\n",
        seed.len()
    )
}

fn frame_read(handle: &str) -> String {
    format!(
        "p967.frame/1\nr n 0\n\np 0\nk 1\n6\nhandleo {}\n{handle}\n",
        handle.len()
    )
}

fn auto_lit(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn run_av_raw(
    source: &str,
    session: &mut ace_abi::rust_provider::F0Session,
) -> Result<ace_av_core::contract::ExecOutcome, String> {
    let environment = session.admitted.clone();
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967ax", "m967ax");
    query.set_foreign(Some(environment));
    let product = ace_source_query::compile_text_record_product(
        source, "f0ax", &caps, "p967ax", "m967ax", &mut query,
    )
    .map_err(|e| format!("compile: {e:?}"))?;
    let mir = ace_lowering::hir_adapter::lower_checked_owned(
        &product.compiled.checked,
        "d_main",
        ace_av_core::contract::fnv1a64(source.as_bytes()),
    )
    .map_err(|e| format!("lower: {e:?}"))?;
    ace_av_core::mir_verify::verify_mir(&mir).map_err(|e| format!("mir: {e:?}"))?;
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(
        decoded,
        &mir.owner,
        session.session.table_shared(),
    )
    .map_err(|e| format!("admit: {e:?}"))?;
    session
        .session
        .execute(
            &loaded,
            &ace_av_core::av::ExecLimits::default(),
            &ace_av_core::av::CancelFlag::new(false),
        )
        .map_err(|e| format!("execute: {e:?}"))
        .map(|(out, _)| out)
}

fn run_av(
    source: &str,
    session: &mut ace_abi::rust_provider::F0Session,
) -> ace_av_core::contract::ExecOutcome {
    match run_av_raw(source, session) {
        Ok(out) => out,
        Err(e) => panic!("the pipeline rejected a legitimate run: {e}"),
    }
}

fn success_i32(out: &ace_av_core::contract::ExecOutcome) -> Option<i32> {
    match out {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } => Some(*v),
        _ => None,
    }
}
