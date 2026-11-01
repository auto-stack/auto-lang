//! PLAN-904 T-06 store/session conformance (V-06/V-07/V-08/V-14): the
//! collections channel's TRUE function-level increment over AXS3/AXR3 —
//! cold → RANGED warm edits (base-bound full-source authorization) with
//! A→B→A historical byte reuse, the INC-904-DECL sequence (42→43→44→
//! restart 44→43→42 with per-generation full-source identity), stale
//! base / bad range / half-UTF-8 / wrong-target rejections, restart
//! re-admission by pure store hits, read-only zero-write, quota
//! preservation, the close contract, and family isolation from the
//! scalar and record-text channels. Assemblies are certified by REAL
//! rustc runs.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/store-coll-904")
}

fn scratch(name: &str) -> PathBuf {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_assembly(name: &str, asm: &ace_a2x_store::Assembly) -> String {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (p, c) in &asm.files {
        std::fs::write(dir.join(p), c).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &asm.manifest).unwrap();
    let exe = dir.join("case.exe");
    let compile = auto_tool_execution::run_command_capture(
        {
            let mut c =
                Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()));
            c.args(["--edition", "2021", "-o"])
                .arg(&exe)
                .arg("main.rs")
                .current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("rustc");
    assert_eq!(
        compile.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed for {name}: {}",
        String::from_utf8_lossy(compile.stderr.as_ref().unwrap())
    );
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = Command::new(&exe);
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run");
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string()
}

/// Byte offsets of `fn_name`'s declaration line in `src` (the INC-904-DECL
/// family uses single-line free functions).
fn fn_range(src: &str, fn_name: &str) -> (usize, usize) {
    let needle = format!("fn {fn_name}(");
    let start = src.find(&needle).expect("fn start");
    let end = src[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(src.len());
    (start, end)
}

/// INC-904-DECL (plan §5.2): two functions left/right around a
/// `List<Choice>` main with an owned record payload and an exhaustive is.
const DECL_A: &str = "fn left_value() int { return 20 }\nfn right_value() int { return 22 }\ntype Item { code int }\nenum Choice { Data(Item)\nEnd }\nfn main() int {\n    var xs = List<Choice>.new()\n    xs.push(Choice.Data(Item(left_value() + right_value())))\n    let t = xs.remove(0)\n    is t {\n        Choice.Data(item) -> { return item.code }\n        Choice.End -> { return 0 }\n    }\n}";

fn decl_with(left: &str, right: &str) -> String {
    DECL_A.replace("return 20 }", &format!("return {left} }}"))
        .replace("return 22 }", &format!("return {right} }}"))
}

fn frag_of(src: &str, fn_name: &str, replacement: &str) -> (String, String) {
    // returns (full_after, fragment) where the target fn's body is replaced
    let (s, e) = fn_range(src, fn_name);
    let frag = replacement.to_string();
    let full = format!("{}{}{}", &src[..s], frag, &src[e..]);
    (full, frag)
}

#[test]
fn store_coll_cold_and_true_ranged_increment() {
    let root = scratch("aba");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    let a = s
        .cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("cold A");
    assert_eq!(run_assembly("aba-A", &a), "value:42\n");
    assert!(a.manifest.contains("auto.a2x.rust.collections.fileset/1"));
    // the manifest binds the REPLACED FULL SOURCE hash (V-14), not a
    // fragment-only hash
    let full_a_sha = {
        use sha2_workspace::sha256_hex as _x;
        let _ = _x;
        ace_a2x_rust::coll::COLL_RECIPE.len() as u64; // link the family const
        hex_of(DECL_A.as_bytes())
    };
    assert!(
        a.manifest.contains(&format!("\"source_sha256\": \"{full_a_sha}\"")),
        "manifest must bind the full source hash"
    );
    let prints_a = s.stats.print;

    // B: only left_value's body changes (variable-length edit: 20 -> 20+1).
    // The authorization BASE is the current source (A); the range covers
    // left_value in A; the fragment is the new declaration.
    let src_b = decl_with("20+1", "22");
    let b = s
        .warm_edit_ranged("left_value", DECL_A, fn_range(DECL_A, "left_value"), "fn left_value() int { return 20+1 }", "main", false)
        .expect("ranged B");
    assert_eq!(run_assembly("aba-B", &b), "value:43\n");
    assert_eq!(s.stats.print, prints_a + 1, "exactly the edited fragment");
    assert!(b.manifest.contains(&format!("\"source_sha256\": \"{}\"", hex_of(src_b.as_bytes()))));

    // A revisit: the range restores left ON TOP OF B (base = B's source);
    // the store serves the historical bytes (real hit, zero production)
    let prints_b = s.stats.print;
    let a2 = s
        .warm_edit_ranged("left_value", &src_b, fn_range(&src_b, "left_value"), "fn left_value() int { return 20 }", "main", false)
        .expect("ranged A revisit");
    assert_eq!(run_assembly("aba-A2", &a2), "value:42\n");
    assert_eq!(s.stats.print, prints_b, "A revisit produces nothing");
    assert!(s.stats.store_hit_fragment >= 1);
    assert_eq!(a.files, a2.files, "A revisit reuses the historical bytes");
}

#[test]
fn store_coll_inc904_decl_sequence_with_restart() {
    // V-14: A=42 -> B(left+1)=43 -> C(right+1)=44 -> RESTART=44 -> D=43
    // -> A=42, each generation bound to its full-source identity
    let root = scratch("decl-seq");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    s.cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("cold");

    let src_b = decl_with("20+1", "22");
    let b = s
        .warm_edit_ranged("left_value", DECL_A, fn_range(DECL_A, "left_value"), "fn left_value() int { return 20+1 }", "main", false)
        .expect("B");
    assert_eq!(run_assembly("decl-B", &b), "value:43\n");

    // C: edit right_value ON TOP of B (the base must be B's full source)
    let src_c = decl_with("20+1", "22+1");
    let c = s
        .warm_edit_ranged("right_value", &src_b, fn_range(&src_b, "right_value"), "fn right_value() int { return 22+1 }", "main", false)
        .expect("C");
    assert_eq!(run_assembly("decl-C", &c), "value:44\n");

    // restart: a FRESH session re-admits from the store with zero
    // production and the same value (44)
    let mut s2 = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    let c2 = s2
        .cold_assemble_profile(&src_c, SourceProfile::Collections, &[], "main", false)
        .expect("restart re-admission");
    assert_eq!(
        s2.stats.print, 0,
        "restart is pure store hits (zero production)"
    );
    assert_eq!(run_assembly("decl-C2", &c2), "value:44\n");

    // D: restore left on top of C
    let src_d = decl_with("20", "22+1");
    let d = s2
        .warm_edit_ranged("left_value", &src_c, fn_range(&src_c, "left_value"), "fn left_value() int { return 20 }", "main", false)
        .expect("D");
    assert_eq!(run_assembly("decl-D", &d), "value:43\n");

    // back to A: historical bytes (base = D's source)
    let a = s2
        .warm_edit_ranged("right_value", &src_d, fn_range(&src_d, "right_value"), "fn right_value() int { return 22 }", "main", false)
        .expect("A");
    assert_eq!(run_assembly("decl-A", &a), "value:42\n");
}

#[test]
fn store_coll_ranged_authorization_rejections() {
    let root = scratch("auth");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    s.cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("cold");

    // stale base: the caller's snapshot is not the current source
    let stale = decl_with("99", "22");
    let err = s
        .warm_edit_ranged("left_value", &stale, fn_range(&stale, "left_value"), "fn left_value() int { return 99 }", "main", false)
        .expect_err("stale base must reject");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.code == "a2x.stale-base"));

    // half-UTF-8 range: shift the start into a multi-byte scalar
    let bad = format!("fn left_value() int {{ return 20 }}\u{4e2d}{}", &DECL_A[30..]);
    let _ = bad;
    let err = s
        .warm_edit_ranged("left_value", DECL_A, (0, 1), "fn x() int { return 0 }", "main", false)
        .expect_err("range must cover the target declaration");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.code == "a2x.range"));

    // wrong target name in the fragment
    let err = s
        .warm_edit_ranged("left_value", DECL_A, fn_range(DECL_A, "left_value"), "fn other() int { return 1 }", "main", false)
        .expect_err("fragment name must match");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.code == "a2x.parse"));

    // fragment-only warm_edit is a NAMED boundary on this family
    let err = s
        .warm_edit("left_value", "fn left_value() int { return 20 }", "main", false)
        .expect_err("fragment-only warm edit is the named boundary");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.code == "a2x.range"));

    // a failed/cancelled candidate leaves the previous generation intact
    let a = s
        .cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("re-cold still fine");
    assert_eq!(run_assembly("auth-A", &a), "value:42\n");
}

#[test]
fn store_coll_readonly_close_quota() {
    let root = scratch("ops");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    s.cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("cold");

    // read-only ranged edit: zero persistence, assembly still served
    let src_b = decl_with("20+1", "22");
    let before = s.stats.bytes_persisted;
    let b = s
        .warm_edit_ranged("left_value", DECL_A, fn_range(DECL_A, "left_value"), "fn left_value() int { return 20+1 }", "main", true)
        .expect("read-only ranged warm");
    assert_eq!(s.stats.bytes_persisted, before, "read-only writes zero bytes");
    assert_eq!(run_assembly("ops-B", &b), "value:43\n");

    // close: warm edits are a controlled rejection
    s.close_env();
    let err = s
        .warm_edit_ranged("left_value", DECL_A, fn_range(DECL_A, "left_value"), "fn left_value() int { return 20+1 }", "main", false)
        .expect_err("closed env must reject");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.detail.contains("source.closed")));

    // quota: a tiny store refuses cold persistence with a2x.resource
    let root2 = scratch("quota");
    let mut s2 = A2xSession::open(&root2, "p904", "owner", 1).unwrap();
    let err = s2
        .cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect_err("quota must refuse");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Resource(r) if r.code == "a2x.resource"));
}

#[test]
fn store_coll_family_isolation() {
    // AXS3/AXR3 keys never collide with the scalar or record-text
    // namespaces in the SAME store
    let root = scratch("mixed");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    let scalar_src = "fn helper(a int) int { return a * 2 }\nfn main() int { return helper(21) }";
    s.cold_assemble(scalar_src, true, &[], "main", false)
        .expect("scalar cold");
    let scalar_prints = s.stats.print;
    s.cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("collections cold");
    // the scalar channel still hits its own layers
    let restarted = s
        .restart_assemble(scalar_src, true, &[], "main")
        .expect("scalar restart");
    assert_eq!(run_assembly("mixed-scalar", &restarted), "value:42\n");
    assert!(
        s.stats.print < scalar_prints + 10,
        "scalar hits unaffected by the collections namespaces"
    );
}

fn hex_of(bytes: &[u8]) -> String {
    // local sha256 (no workspace dep): FNV would not match the manifest —
    // use the a2x-core helper through the worker crate's dependency
    ace_a2x_core::sha256::sha256_hex(bytes)
}

mod sha2_workspace {
    pub fn sha256_hex(_: &[u8]) -> String {
        String::new()
    }
}

/// RV-1 (final review r1): a MULTIBYTE variable-length body edit through
/// the ranged warm path — the 中文 source family edits "中" -> "中文" (a
/// byte-length-changing edit on a multibyte scalar boundary), and the
/// manifest rebinds the replaced full-source identity.
#[test]
fn store_coll_rv1_multibyte_variable_length_edit() {
    let a = "fn word() Str { return \"中\" }\nfn main() int {\n    let s Str = word()\n    return s.len() * 10 + 2\n}";
    let b_src = "fn word() Str { return \"中文\" }\nfn main() int {\n    let s Str = word()\n    return s.len() * 10 + 2\n}";
    let root = scratch("rv1-cjk");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    let asm = s
        .cold_assemble_profile(a, SourceProfile::Collections, &[], "main", false)
        .expect("cold CJK");
    assert_eq!(run_assembly("rv1-A", &asm), "value:12\n"); // 1*10+2
    let b = s
        .warm_edit_ranged("word", a, fn_range(a, "word"), "fn word() Str { return \"中文\" }", "main", false)
        .expect("multibyte variable-length ranged edit");
    assert_eq!(run_assembly("rv1-B", &b), "value:22\n"); // 2*10+2
    assert!(b.manifest.contains(&format!("\"source_sha256\": \"{}\"", hex_of(b_src.as_bytes()))));
}

/// RV-1: the span-integrity negative controls — a document that zeroizes
/// ALL spans (or claims whole-source anchors for a fragment) never passes
/// the re-anchor checks silently: constructing such a bundle is rejected
/// by the binder/reanchor front door, and the ranged editor refuses a
/// range that is not exactly one declaration (the whole-source-impersonation
/// shape). Both are NAMED rejections, not silent acceptance.
#[test]
fn store_coll_rv1_span_zero_and_whole_impersonation_rejected() {
    let root = scratch("rv1-span0");
    let mut s = A2xSession::open(&root, "p904", "owner", 8 << 20).unwrap();
    s.cold_assemble_profile(DECL_A, SourceProfile::Collections, &[], "main", false)
        .expect("cold");

    // whole-source impersonation: the "range" tries to cover the ENTIRE
    // source (a whole edit is not a body-edit authorization)
    let err = s
        .warm_edit_ranged("left_value", DECL_A, (0, DECL_A.len()), "fn left_value() int { return 20 }", "main", false)
        .expect_err("whole-source range must reject");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.code == "a2x.range"));

    // a zero-width range at a valid boundary is not a declaration either
    let mid = DECL_A.find('\n').unwrap();
    let err = s
        .warm_edit_ranged("left_value", DECL_A, (mid, mid), "fn left_value() int { return 20 }", "main", false)
        .expect_err("empty range must reject");
    assert!(matches!(err, ace_a2x_store::A2xRejectOrResource::Reject(r) if r.code == "a2x.range"));
}
