//! PLAN-967 T-05 F1 (AC-INT-1026 / V-INT-1026, IF-CALL): the checked
//! tag parameter frame — an INDEPENDENT generic Rust provider driven
//! through the real metadata → auto C-compatible shim → AV stub chain.
//! Legit shapes (scalars / NUL-carrying text / opaque refs / kwargs)
//! cross for real; bad tags/lengths/counts/trailing bytes are NAMED
//! pre-dispatch rejections (`foreign.frame`, dispatch=0); stale
//! generations, opaque misuse and the panic channel each close on
//! their own. Direct Rust unit tests are NOT the AV evidence — every
//! value assertion here runs through the AV interpreter.
#![cfg(windows)]
use std::path::PathBuf;

use ace_abi::rust_provider::{build_source, f0_authority, f0_tframe_provider_source, load};

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

/// Build one `p967.frame/1` wire frame (test-side writer; the AV-side
/// validator is the authority on what is legal). Final frozen layout:
/// every payload is LF-terminated; no separator after a kwarg name.
fn frame(receiver: &str, positionals: &[(&str, &str)], kwargs: &[(&str, &str, &str)]) -> String {
    let mut f = String::from("p967.frame/1\n");
    let (tag, payload) = split_val(receiver);
    f.push_str(&format!("r {tag} {}\n{payload}\n", payload.len()));
    f.push_str(&format!("p {}\n", positionals.len()));
    for (tag, payload) in positionals {
        f.push_str(&format!("{tag} {}\n{payload}\n", payload.len()));
    }
    f.push_str(&format!("k {}\n", kwargs.len()));
    for (name, tag, payload) in kwargs {
        f.push_str(&format!(
            "{}\n{name}{tag} {}\n{payload}\n",
            name.len(),
            payload.len()
        ));
    }
    f
}

/// Receiver shorthand: "n" | "i:5" | "b:1" | "t:abc" | "o:3" (bare text
/// defaults to the text tag).
fn split_val(v: &str) -> (&str, &str) {
    if v == "n" {
        return ("n", "");
    }
    match v.split_once(':') {
        Some(("i", rest)) => ("i", rest),
        Some(("b", rest)) => ("b", rest),
        Some(("t", rest)) => ("t", rest),
        Some(("o", rest)) => ("o", rest),
        _ => ("t", v),
    }
}

fn run(
    source: &str,
    table: &mut ace_av_core::foreign_exec::ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> ace_av_core::contract::ExecOutcome {
    match run_raw(source, table, environment) {
        Ok(out) => out,
        Err(e) => panic!("the pipeline rejected a legitimate run: {e}"),
    }
}

/// The full AV pipeline; `Err` carries the named ADMISSION rejection
/// (the stale-generation oracle — a bumped table refuses to admit).
fn run_raw(
    source: &str,
    table: &mut ace_av_core::foreign_exec::ForeignStubTable,
    environment: &ace_foreign_contract::manual::AdmittedForeign,
) -> Result<ace_av_core::contract::ExecOutcome, String> {
    let caps = [
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ];
    let mut query = ace_source_query::SourceQuerySession::new("p967f1", "m967f1");
    query.set_foreign(Some(environment.clone()));
    let product = ace_source_query::compile_text_record_product(
        source, "f0f1", &caps, "p967f1", "m967f1", &mut query,
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
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
        .map_err(|e| format!("admit: {e:?}"))?;
    ace_av_core::av::execute_foreign(
        &loaded,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
        table,
    )
    .map_err(|e| format!("execute: {e:?}"))
    .map(|(out, _)| out)
}

fn auto_lit(s: &str) -> String {
    format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
}

fn expect_i32(out: ace_av_core::contract::ExecOutcome, want: i32, what: &str) {
    use ace_av_core::contract::{ExecOutcome, Value};
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(v, want, "{what}"),
        other => panic!("{what}: expected Success(I32({want})), got {other:?}"),
    }
}

fn expect_named(out: ace_av_core::contract::ExecOutcome, what: &str) {
    use ace_av_core::contract::{ExecOutcome, TrapKind};
    match out {
        ExecOutcome::Trap {
            kind: TrapKind::ForeignStatus,
            ..
        } => {}
        other => panic!("{what}: expected ForeignStatus, got {other:?}"),
    }
}

#[test]
fn v_int_1026_tag_frame_crosses_for_real_and_bad_frames_reject_pre_dispatch() {
    let root = scope("int1026");
    let build =
        build_source(&root, f0_tframe_provider_source()).expect("the frame provider builds");
    // the frame axis is real metadata: every fn declares p967.frame/1
    assert!(build
        .manifest
        .functions
        .iter()
        .all(|f| f.frame.as_deref() == Some("p967.frame/1")));

    let mut loaded = load(&build, 1).expect("loads");
    let environment = build
        .manual
        .clone()
        .admit(&f0_authority(&build, 1))
        .expect("admits");

    // ---- legit shapes through the AV lane
    // scalars: positionals + kwargs (the receiver is none)
    let f = frame("n", &[("i", "4"), ("t", "x")], &[("k", "i", "5")]);
    expect_i32(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_sum_i32({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        ),
        9,
        "scalar positionals + kwarg sum",
    );

    // text: receiver + kwarg text lengths, NUL rides by LENGTH (the
    // frame never truncates at NUL — the V-NATIVE byte discipline);
    // "中\0z" is 3+1+1 = 5 UTF-8 bytes
    let f = frame("t:abc", &[("t", "中\0z")], &[("note", "t", "hi")]);
    expect_i32(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_text_len({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        ),
        3 + 5 + 2,
        "receiver+positional(+NUL)+kwarg text lengths",
    );

    // opaque: store (positional seed) -> id, read back via kwarg handle
    let f = frame("n", &[("i", "41")], &[]);
    let stored = match run(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_store({}) }}",
            auto_lit(&f)
        ),
        &mut loaded.table,
        &environment,
    ) {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(id),
            ..
        } => id,
        other => panic!("opaque store: {other:?}"),
    };
    assert_eq!(stored, 1, "the first registry id");
    let f = frame("n", &[], &[("handle", "o", "1")]);
    expect_i32(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        ),
        41,
        "opaque round-trip via the kwarg handle",
    );
    // the opaque object outlives the frame that created it (a LATER
    // frame still reads it — the provider owns the lifecycle)
    let f = frame("n", &[], &[("handle", "o", "1")]);
    expect_i32(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        ),
        41,
        "the opaque object outlives its frame",
    );

    // no-i32 frame -> the NAMED fallible failure
    let f = frame("t:only-text", &[], &[]);
    expect_named(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_sum_i32({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        ),
        "no-i32 is the named no-i32 status",
    );
    // an unknown opaque handle -> named bad-handle
    let f = frame("n", &[], &[("handle", "o", "99")]);
    expect_named(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_opaque_read({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        ),
        "unknown handle is the named bad-handle status",
    );

    // ---- BAD FRAMES: named `foreign.frame` rejections BEFORE dispatch
    let before = loaded.table.callee_count();
    let bad_frames = [
        ("bad magic", "nope\nr n 0\np 0\nk 0\n".to_string()),
        (
            "unknown tag",
            "p967.frame/1\nr x 1\n1\np 0\nk 0\n".to_string(),
        ),
        ("bad bool payload", frame("b:2", &[], &[])),
        ("bad i32 payload", frame("n", &[("i", "4x")], &[])),
        ("count overrun", frame("n", &[("i", "1"); 9], &[])),
        (
            "truncated payload",
            "p967.frame/1\nr t 40\nabc\np 0\nk 0\n".to_string(),
        ),
        ("trailing bytes", format!("{}garbage", frame("n", &[], &[]))),
        (
            "kwarg name overrun",
            "p967.frame/1\nr n 0\np 0\nk 1\n99\nx i 1\n1\n".to_string(),
        ),
    ];
    for (what, f) in bad_frames {
        let out = run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_text_len({}) }}",
                auto_lit(&f)
            ),
            &mut loaded.table,
            &environment,
        );
        // pre-dispatch rejections surface as Infra carrying the NAMED
        // code (the delivered F0 semantics); the callee never ran
        match out {
            ace_av_core::contract::ExecOutcome::Infra(ref m) if m.contains("foreign.frame") => {}
            other => panic!("{what}: expected the named frame rejection, got {other:?}"),
        }
    }
    assert_eq!(
        loaded.table.callee_count(),
        before,
        "every bad frame rejected with dispatch=0"
    );

    // ---- stale generation: the frame lane inherits the session
    // discipline (a bumped generation refuses ADMISSION)
    loaded.table.bump_generation();
    let f = frame("n", &[("i", "1")], &[]);
    let rejected = run_raw(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_sum_i32({}) }}",
            auto_lit(&f)
        ),
        &mut loaded.table,
        &environment,
    )
    .expect_err("a bumped generation must refuse admission");
    assert!(
        rejected.contains("generation"),
        "the stale-generation rejection is named: {rejected}"
    );

    // ---- the panic channel (receiver "boom") is an independent HOST
    // failure — never an unwind across the boundary, never a value
    let mut fresh = load(&build, 1).expect("reload at a fresh generation");
    let f = frame("t:calm", &[], &[]);
    expect_i32(
        run(
            &format!(
                "use.rs f0_tframe\nfn main() int {{ return tframe_panic_probe({}) }}",
                auto_lit(&f)
            ),
            &mut fresh.table,
            &environment,
        ),
        7,
        "calm receiver answers",
    );
    let f = frame("t:boom", &[], &[]);
    let out = run(
        &format!(
            "use.rs f0_tframe\nfn main() int {{ return tframe_panic_probe({}) }}",
            auto_lit(&f)
        ),
        &mut fresh.table,
        &environment,
    );
    assert!(
        matches!(
            out,
            ace_av_core::contract::ExecOutcome::Infra(ref m) if m.contains("foreign.panic")
        ),
        "provider panic is the independent host failure: {out:?}"
    );
    drop(fresh);
    drop(loaded);
}

#[test]
fn v_int_1026_named_domain_rejections() {
    // the importer domain: a frame fn MUST take exactly one
    // `frame: &F0Frame`; the frame type without the axis (and the axis
    // on an ordinary shape) are named rejections — the metadata cannot
    // describe a call the provider does not actually export.
    let src = f0_tframe_provider_source();
    // axis without the type
    let broken = src.replace(
        "pub fn tframe_text_len(frame: &F0Frame) -> i32 {",
        "pub fn tframe_text_len(s: &str) -> i32 {\n    let _ = s; 0\n}\npub fn __unused_tframe_text_len(frame: &F0Frame) -> i32 {",
    );
    let err = match ace_abi::rust_provider::build_source(&scope("int1026-rej-a"), &broken) {
        Err(e) => e,
        Ok(_) => panic!("a frame axis on an ordinary shape must reject"),
    };
    assert!(err.contains("rust-import.frame"), "{err}");

    // type without the axis
    let broken2 = src.replace(
        " frame=p967.frame/1\npub fn tframe_sum_i32",
        "\npub fn tframe_sum_i32",
    );
    let err2 = match ace_abi::rust_provider::build_source(&scope("int1026-rej-b"), &broken2) {
        Err(e) => e,
        Ok(_) => panic!("the frame type without the axis must reject"),
    };
    assert!(err2.contains("rust-import.frame"), "{err2}");

    // a wrong frame version is a named rejection too (PLAN-1129 made
    // `p967.frame/2` the typed-value superset; the next spelling is unknown)
    let broken3 = src.replace(
        "frame=p967.frame/1\npub fn tframe_sum_i32",
        "frame=p967.frame/3\npub fn tframe_sum_i32",
    );
    let err3 = match ace_abi::rust_provider::build_source(&scope("int1026-rej-c"), &broken3) {
        Err(e) => e,
        Ok(_) => panic!("an unknown frame version must reject"),
    };
    assert!(err3.contains("rust-import.frame"), "{err3}");
    assert!(err3.contains("p967.frame/1 | p967.frame/2"), "{err3}");

    // ...while the typed superset itself imports, and a declaration naming
    // it keeps the version as part of its interface identity
    let typed = src.replace(
        "frame=p967.frame/1\npub fn tframe_sum_i32",
        "frame=p967.frame/2\npub fn tframe_sum_i32",
    );
    let manifest = auto_bindgen::rust_import::import(&typed).expect("a /2 annotation imports");
    let sum = manifest
        .functions
        .iter()
        .find(|f| f.rust_name == "tframe_sum_i32")
        .expect("tframe_sum_i32");
    assert_eq!(sum.frame.as_deref(), Some("p967.frame/2"));
    let legacy = auto_bindgen::rust_import::import(src).expect("the frozen /1 source imports");
    assert_ne!(
        auto_bindgen::rust_import::provider_digest(&typed),
        auto_bindgen::rust_import::provider_digest(src),
        "the frame version is part of the provider identity"
    );
    // one decoder serves both versions: the prelude is emitted once
    let shim = auto_bindgen::rust_import::generate_shim(&manifest);
    assert_eq!(shim.matches("fn __f0_frame_decode").count(), 1);
    assert!(shim.contains("p967.frame/2\\n"));
}
