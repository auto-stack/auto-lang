//! PLAN-925 CA-B (T-03): the C record-text TRUE-CHAIN conformance leg —
//! source → checked frontend → shared semantic view → C provider (AXC2)
//! → session assembly (store-persisted fragments, record-table seed) →
//! REAL MSVC cl /std:c11 compile → real run → parent-side oracle.
//!
//! The four FROZEN required sources (R07/R08/T03/T05, byte-exact ace-m1
//! originals) run against the frozen oracles 42/42/23/42 — the same
//! oracle table as the Rust leg (`record_text_902`), INDEPENDENTLY
//! executed through the C channel. V-03 covers the text/char surface:
//! Unicode scalar counts vs UTF-8 byte counts, NUL-bearing literals,
//! full-byte equality, scalar-indexed char_at/slice (negative/out-of-range
//! = TextIndex), codepoint roundtrip (invalid = InvalidScalar). The
//! cleanup counterexample pins the verifier-plan buffer schedule through
//! the C event ledger; the fault hook turns the n-th C allocation into
//! the typed `Resource` trap. File-set identity/determinism mirrors the
//! 921 scalar leg.
//!
//! A missing MSVC toolchain is an INFRA failure (never a skip); compile
//! errors are infra facts, never Auto diagnostics/traps.

use ace_a2x_c::CTargetProvider;
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;
use std::process::Command;

const R07: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/R07.at");
const R08: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/R08.at");
const T03: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/T03.at");
const T05: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/T05.at");

/// PLAN-902 §5.3 cleanup counterexample (same source as the Rust leg):
/// declare a, declare b, replace a — the explicit cleanup frees the
/// replacement's OLD buffer after the RHS materialized; exits close in
/// the plan's declaration-LIFO order (b before a).
const CLEANUP: &str = r#"fn main() int {
    var a Str = "aa"
    let b Str = a.clone()
    a = a + "bb"
    return a.len() * 10 + b.len()
}"#;

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-rt")
}

fn store_root() -> PathBuf {
    // fresh store PER PROCESS (fragment identities key on the frozen
    // target identity; a reused store across code changes would serve
    // stale bytes — start from a clean namespace like the scalar leg)
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/a2x-out-c-rt/store-{}",
        std::process::id()
    ))
}

fn profile_of(prof: &str) -> SourceProfile {
    match prof {
        "record" => SourceProfile::Record,
        "text" => SourceProfile::Text,
        "text-record" => SourceProfile::TextRecord,
        other => panic!("unknown profile {}", other),
    }
}

/// Open one C record-text session on a per-test store namespace (SQLite
/// single-writer: tests run in parallel threads, each needs its own db).
fn open_session(test: &str) -> A2xSession {
    A2xSession::open(
        &store_root().join(test),
        "a2c-rt",
        "probe",
        128 * 1024 * 1024,
        CTargetProvider::v1(),
    )
    .expect("open C rt session")
}

/// Emit + compile + run one case through the C session chain; returns
/// (stdout, stderr) of the RUN.
fn run_c_chain(
    s: &mut A2xSession,
    name: &str,
    src: &str,
    prof: &str,
    env: &[(&str, &str)],
) -> (String, String) {
    let asm = s
        .cold_assemble_profile(src, profile_of(prof), &[], "main", false)
        .unwrap_or_else(|e| panic!("C rt assembly failed for {}: {:?}", name, e));
    let dir = out_root().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("a2c_rt_case.exe");
    ace_a2x_worker::crun::compile_c_fileset(&dir, &exe)
        .unwrap_or_else(|e| panic!("cl failed for {}: {}", name, e));
    let mut command = Command::new(&exe);
    command.current_dir(&dir);
    for (k, v) in env {
        command.env(k, v);
    }
    let run = auto_tool_execution::run_command_capture(
        command,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    assert_eq!(
        run.kind,
        auto_tool_execution::ExitKind::Success,
        "wrapper exit non-zero for {}",
        name
    );
    (
        String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().expect("captured stderr")).to_string(),
    )
}

fn assert_value(name: &str, out: &str, expected: i32) {
    assert_eq!(out, format!("value:{expected}\n"), "stdout mismatch for {name}");
}

fn assert_trap(name: &str, out: &str, trap: &str) {
    assert_eq!(out, format!("trap:{trap}\n"), "stdout mismatch for {name}");
}

/// V-01 first cut: the four frozen required sources through the C
/// channel, against the frozen oracle table (42/42/23/42) — the same
/// values the Rust leg pins, independently executed here.
#[test]
fn a2c_rt_required_four() {
    let cases: &[(&str, &str, &str, i32)] = &[
        ("R07", R07, "record", 42),
        ("R08", R08, "record", 42),
        ("T03", T03, "text-record", 23),
        ("T05", T05, "text", 42),
    ];
    let mut s = open_session("required_four");
    for (name, src, prof, value) in cases {
        let (out, _err) = run_c_chain(&mut s, name, src, prof, &[]);
        assert_value(name, &out, *value);
    }
}

/// V-03: the text/char surface — Unicode scalar vs byte counts, NUL
/// literals, full-byte equality (no normalization), scalar-indexed
/// char_at/slice with typed TextIndex traps, codepoint roundtrip with
/// typed InvalidScalar.
#[test]
fn a2c_rt_text_char_surface() {
    let mut s = open_session("text_char_surface");
    // scalar count 4 (h é l l o... "héllo" = 5 chars) vs byte count 6
    let (out, _) = run_c_chain(
        &mut s,
        "v03_len_scalars_bytes",
        "fn main() int {\n    let s Str = \"héllo\"\n    return s.len() * 100 + s.len_bytes()\n}",
        "text",
        &[],
    );
    assert_value("v03_len_scalars_bytes", &out, 506);
    // NUL is data: never strlen-truncated (len 3, bytes 3)
    let (out, _) = run_c_chain(
        &mut s,
        "v03_nul_data",
        "fn main() int {\n    let s Str = \"a\\u{0}b\"\n    return s.len() * 10 + s.len_bytes()\n}",
        "text",
        &[],
    );
    assert_value("v03_nul_data", &out, 33);
    // full-byte equality: equal text, different scalars describing é
    let (out, _) = run_c_chain(
        &mut s,
        "v03_eq_full_bytes",
        "fn main() int {\n    let a Str = \"é\"\n    let b Str = \"é\"\n    if a == b { return 1 }\n    return 0\n}",
        "text",
        &[],
    );
    assert_value("v03_eq_full_bytes", &out, 1);
    // char_at is scalar-indexed; é at index 1 → 233
    let (out, _) = run_c_chain(
        &mut s,
        "v03_char_at",
        "fn main() int {\n    let s Str = \"héllo\"\n    let c char = s.char_at(1)\n    return c.to_codepoint()\n}",
        "text",
        &[],
    );
    assert_value("v03_char_at", &out, 233);
    // negative index → TextIndex (never a byte misread)
    let (out, _) = run_c_chain(
        &mut s,
        "v03_char_at_negative",
        "fn main() int {\n    let s Str = \"abc\"\n    let c char = s.char_at(-1)\n    return c.to_codepoint()\n}",
        "text",
        &[],
    );
    assert_trap("v03_char_at_negative", &out, "TextIndex");
    // out-of-range slice end → TextIndex
    let (out, _) = run_c_chain(
        &mut s,
        "v03_slice_oob",
        "fn main() int {\n    let s Str = \"héllo\"\n    let t str = s.slice(0, 6)\n    return t.len()\n}",
        "text",
        &[],
    );
    assert_trap("v03_slice_oob", &out, "TextIndex");
    // valid slice: scalar range on multi-byte text ("llo" → len 3)
    let (out, _) = run_c_chain(
        &mut s,
        "v03_slice_scalars",
        "fn main() int {\n    let s Str = \"héllo\"\n    let t str = s.slice(2, 5)\n    return t.len()\n}",
        "text",
        &[],
    );
    assert_value("v03_slice_scalars", &out, 3);
    // codepoint roundtrip: valid → char; surrogate → InvalidScalar
    let (out, _) = run_c_chain(
        &mut s,
        "v03_codepoint_ok",
        "fn main() int {\n    let c char = char.from_codepoint(20013)\n    return c.to_codepoint()\n}",
        "text",
        &[],
    );
    assert_value("v03_codepoint_ok", &out, 20013);
    let (out, _) = run_c_chain(
        &mut s,
        "v03_codepoint_surrogate",
        "fn main() int {\n    let c char = char.from_codepoint(55296)\n    return c.to_codepoint()\n}",
        "text",
        &[],
    );
    assert_trap("v03_codepoint_surrogate", &out, "InvalidScalar");
    // concat + clone + to(Str) chain over multi-byte text
    let (out, _) = run_c_chain(
        &mut s,
        "v03_concat_clone",
        "fn main() int {\n    let a Str = \"héllo\"\n    let b Str = a.clone()\n    let c Str = a + b\n    return c.len() * 10 + c.len_bytes()\n}",
        "text",
        &[],
    );
    assert_value("v03_concat_clone", &out, 112);
}

/// The cleanup counterexample through C: value oracle plus the EXACT C
/// runtime alloc/free event order (replacement frees the OLD buffer only
/// after the RHS materialized; exits in the plan's declaration-LIFO
/// order — b before a).
#[test]
fn a2c_rt_cleanup_counterexample_events() {
    let mut s = open_session("cleanup_events");
    let (out, err) = run_c_chain(&mut s, "CLEANUP", CLEANUP, "text", &[("A2X_RT_EVENTS", "1")]);
    assert_value("CLEANUP", &out, 42);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // a = "aa"
            "rt:alloc:2", // b = a.clone()
            "rt:alloc:4", // a = a + "bb" (RHS materialized FIRST)
            "rt:free:2",  // REPLACE: the old a buffer dies post-RHS
            "rt:free:2",  // exit: b (declaration-LIFO)
            "rt:free:4",  // exit: a
        ],
        "C event ledger must pin the verifier-plan cleanup order"
    );
}

/// The default-off fault hook: the n-th C allocation fails as the typed
/// `Resource` trap (no partial commit — the old value stays intact for a
/// same-process retry; the retry counterexample lands with T-05's matrix).
#[test]
fn a2c_rt_fault_alloc_resource() {
    // a="aa"(1) b=clone(2) concat(3): fault the concat → Resource
    let mut s = open_session("fault3");
    let (out, _) = run_c_chain(&mut s, "FAULT3", CLEANUP, "text", &[("A2X_RT_FAULT_ALLOC", "3")]);
    assert_trap("FAULT3", &out, "Resource");
    // faulting past the last allocation changes nothing
    let mut s9 = open_session("fault9");
    let (out, _) = run_c_chain(&mut s9, "FAULT9", CLEANUP, "text", &[("A2X_RT_FAULT_ALLOC", "9")]);
    assert_value("FAULT9", &out, 42);
}

/// File-set identity: the C record-text manifest carries the rt schema,
/// and the emitted file set is byte-deterministic across two fresh
/// sessions (same inputs → same bytes).
#[test]
fn a2c_rt_fileset_identity_and_determinism() {
    let src = "fn main() int {\n    let s Str = \"abc\"\n    return s.len()\n}";
    let assemble = |root: &std::path::Path| {
        let mut s = A2xSession::open(root, "a2c-rt-det", "probe", 8 << 20, CTargetProvider::v1())
            .unwrap();
        let asm = s
            .cold_assemble_profile(src, SourceProfile::Text, &[], "main", false)
            .unwrap();
        (asm.files, asm.manifest)
    };
    let dir1 = store_root().join("det1");
    let dir2 = store_root().join("det2");
    let (files1, manifest1) = assemble(&dir1);
    let (files2, manifest2) = assemble(&dir2);
    assert_eq!(files1, files2, "file set must be byte-deterministic");
    assert_eq!(manifest1, manifest2, "manifest must be byte-deterministic");
    assert!(manifest1.contains("auto.a2x.c.record-text.fileset/1"));
    assert!(manifest1.contains("\"entry\": \"main\""));
    assert!(manifest1.contains("a2x_c_rt_runtime.h"));
    // no runtime-header knob ever changes the emitted bytes: the fault
    // config is environment-side, not text-side
    assert!(!files1
        .iter()
        .any(|(_, c)| c.contains("A2X_RT_FAULT_ALLOC=")));
}

/// The prelude-seed codec: encode/decode roundtrip and STRICT decode
/// (schema line, counts, no trailing content, malformed rejections).
#[test]
fn a2c_rt_seed_codec_strictness() {
    use ace_a2x_c::seed::{decode_seed, encode_seed, C_RT_SEED_SCHEMA};
    use ace_a2x_core::SemanticView;
    use ace_core::source::compile_text_record_source;
    let src = "type Inner { n int }\ntype Wrap {
    inner Inner
    s Str
}\nfn main() int {\n    let w Wrap = Wrap(Inner(1), \"x\")\n    return w.inner.n\n}";
    let compiled = compile_text_record_source(src, "seed", &["ac.runtime.text.v1".to_string()])
        .expect("frontend");
    let view = SemanticView::from_checked(&compiled.checked).expect("view");
    let seed = encode_seed(&view).expect("seed");
    assert!(seed.starts_with(C_RT_SEED_SCHEMA));
    let recs = decode_seed(&seed).expect("decode");
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].name, "Inner");
    assert_eq!(recs[1].fields.len(), 2);
    // roundtrip is byte-stable
    assert_eq!(encode_seed(&view).unwrap(), seed);
    // strict decode: wrong schema line
    assert!(decode_seed("wrong-schema\nR A 0 0\n").is_err());
    // strict decode: field count overrun
    assert!(decode_seed(&format!("{}\nR A 0 2\nF x i32\n", C_RT_SEED_SCHEMA)).is_err());
    // strict decode: trailing garbage
    assert!(decode_seed(&format!("{}\nR A 0 0\ngarbage\n", C_RT_SEED_SCHEMA)).is_err());
    // strict decode: malformed field type
    assert!(decode_seed(&format!("{}\nR A 0 1\nF x bogus\n", C_RT_SEED_SCHEMA)).is_err());
    // strict decode: digit-leading names rejected
    assert!(decode_seed(&format!("{}\nR 9A 0 0\n", C_RT_SEED_SCHEMA)).is_err());
}
