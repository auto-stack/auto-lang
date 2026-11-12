//! PLAN-933 T-05 (V-05): the twenty frozen `a2r-hir-collections-904`
//! sources through the REAL `A2xSession` store/session entries, per
//! case: fresh cold assembly binding the full-source identity (schema +
//! source_sha256 + frozen execution config); a STALE-BASE failed edit
//! rejected by name (state intact); a WHITESPACE-only body edit that
//! rebinds the manifest identity to the new full source while producing
//! ZERO fragments (identity is byte-based, production is semantics-based
//! — the 未改函数生产0 gate); a SEMANTIC literal edit producing exactly
//! one fragment; a historical revisit served from the store with zero
//! production; and a post-restart cold re-assembly reproducing the same
//! session-channel bytes.
//!
//! Axes kept separate (plan §5.4): the session channel assembles its
//! own fragment-based file shape (AX-A session layout) — byte-identity
//! with the daily goldens belongs to the WORKER/parent axis
//! (`case-a2r-hir-collections-904`, 20/20, plus the PLAN-933
//! independent manifest oracle); value certification (real rustc) stays
//! in `collections_904.rs`/`cert_coll_904.rs`; the recovery→first-edit
//! SYN leg is owned by PLAN-929 (D-931-1 pending) and is NOT claimed.

use std::path::{Path, PathBuf};

use ace_a2x_core::sha256::sha256_hex;
use ace_a2x_store::{A2xRejectOrResource, A2xSession};
use ace_core::source::SourceProfile;

/// The twenty frozen case names (suite order).
const CASES: &[&str] = &[
    "L01",
    "L02",
    "L03",
    "L04",
    "L05",
    "L06",
    "L07",
    "V01",
    "V02",
    "V03",
    "V04",
    "V05",
    "V06",
    "V07",
    "LSTR864",
    "FIN864A",
    "FIN864B",
    "N889-list-variant",
    "N889-nested-variant",
    "N889-optional-variant",
];

const COLL_FILESET_SCHEMA: &str = "auto.a2x.rust.collections.fileset/1";
const FROZEN_CONFIG: &str =
    "{\"budget_work_units\":16777216,\"cancel\":false,\"events\":false,\"fault_alloc\":null,\"mode\":\"debug\"}";

fn case_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/a2r-hir-collections-904")
        .canonicalize()
        .expect("904 case root resolves")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/store-coll-933")
}

fn scratch(name: &str) -> PathBuf {
    let dir = out_base().join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn open_session(root: &Path) -> A2xSession {
    A2xSession::open(
        root,
        "p933",
        "owner",
        8 << 20,
        ace_a2x_rust::RustTargetProvider::v1(),
    )
    .expect("session opens")
}

/// The session-channel identity contract for a collections assembly:
/// the manifest binds the family schema, THIS full source's sha256 and
/// the frozen canonical execution config.
fn assert_binds_source(asm: &ace_a2x_store::Assembly, src: &str, case: &str) {
    assert!(
        asm.manifest
            .contains(&format!("\"schema\": \"{COLL_FILESET_SCHEMA}\"")),
        "{case}: session manifest carries the collections fileset schema"
    );
    assert!(
        asm.manifest.contains(&format!(
            "\"source_sha256\": \"{}\"",
            sha256_hex(src.as_bytes())
        )),
        "{case}: manifest binds the full-source sha"
    );
    assert!(
        asm.manifest
            .contains(&format!("\"execution_config\": {FROZEN_CONFIG},")),
        "{case}: manifest binds the frozen canonical config"
    );
}

/// Top-level function declarations as (name, start, end): a declaration
/// runs to the next declaration (the frozen sources place every
/// type/enum before the first fn — T-01 freeze).
fn fn_decls(src: &str) -> Vec<(String, usize, usize)> {
    let bytes = src.as_bytes();
    let mut starts: Vec<(String, usize)> = Vec::new();
    let mut i = 0;
    while let Some(rel) = src[i..].find("fn ") {
        let at = i + rel;
        let at_line_start = at == 0 || bytes[at - 1] == b'\n';
        let after = &src[at + 3..];
        let name: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if at_line_start && !name.is_empty() && after[name.len()..].starts_with('(') {
            starts.push((name, at));
        }
        i = at + 3;
    }
    starts
        .into_iter()
        .map(|(n, s)| {
            let e = src[s + 1..]
                .find("\nfn ")
                .map(|rel| s + 1 + rel + 1)
                .unwrap_or(src.len());
            (n, s, e)
        })
        .collect()
}

/// A uniform WHITESPACE-only edit of `main` (present in every frozen
/// source): one extra space in the signature — bytes change, semantics
/// do not. Returns (range, fragment, new_src).
fn whitespace_edit(src: &str) -> ((usize, usize), String, String) {
    let start = src.find("fn main(").expect("frozen sources declare main");
    let range = (start, src.len());
    let frag = src[start..].replacen("fn main() int {", "fn main()  int {", 1);
    assert_ne!(frag.len(), src[start..].len(), "whitespace edit applied");
    let new_src = format!("{}{}", &src[..start], frag);
    (range, frag, new_src)
}

/// A uniform SEMANTIC edit: bump the first ASCII digit of the source
/// (9 -> 0 still changes the literal), or — for the one frozen source
/// with no digits — extend its first string literal. The edited
/// declaration is the one containing that position. Returns
/// (fn_name, range, fragment, new_src).
fn semantic_edit(src: &str, case: &str) -> (String, (usize, usize), String, String) {
    let decls = fn_decls(src);
    assert!(!decls.is_empty(), "{case}: frozen sources declare fns");
    let (pos, mutate) = match src.bytes().position(|b| b.is_ascii_digit()) {
        Some(p) => {
            let old = src.as_bytes()[p];
            let new = if old == b'9' { b'0' } else { old + 1 };
            (p, new)
        }
        None => {
            // V03: no digits anywhere; extend the first string literal
            let q0 = src.find('"').expect("{case}: a quote exists");
            assert!(src[q0 + 1..].contains('"'), "{case}: closing quote exists");
            (q0, b'x') // marker: insert inside the literal below
        }
    };
    let (name, start, end) = decls
        .iter()
        .rev()
        .find(|(_, s, _)| *s <= pos)
        .unwrap_or_else(|| panic!("{case}: edit position inside a declaration"))
        .clone();
    let mut frag = src[start..end].to_string();
    if src.bytes().position(|b| b.is_ascii_digit()).is_some() {
        let rel = pos - start;
        frag.replace_range(rel..rel + 1, &(mutate as char).to_string());
    } else {
        frag = frag.replacen('"', "\"x", 1);
    }
    let new_src = format!("{}{}{}", &src[..start], frag, &src[end..]);
    assert_ne!(&new_src, src, "{case}: semantic edit changes the source");
    (name, (start, end), frag, new_src)
}

#[test]
fn store_coll_933_twenty_source_four_state_sweep() {
    let root = case_root();
    for (i, case) in CASES.iter().enumerate() {
        let src = std::fs::read_to_string(root.join("src").join(format!("{case}.at"))).unwrap();
        let store = scratch(case);
        let mut s = open_session(&store);

        // (1) fresh: the REAL cold session entry binds the tracked
        // source's full identity through the collections family.
        let a = s
            .cold_assemble_profile(&src, SourceProfile::Collections, &[], "main", false)
            .unwrap_or_else(|e| panic!("{case}: cold: {e:?}"));
        assert_binds_source(&a, &src, case);

        // (2) failed edit: a stale base (a DIFFERENT frozen source) is a
        // named rejection; the session stays usable afterwards.
        let neighbor = std::fs::read_to_string(
            root.join("src")
                .join(format!("{}.at", CASES[(i + 1) % CASES.len()])),
        )
        .unwrap();
        let (m0, _, _) = {
            let start = src.find("fn main(").unwrap();
            ((start, src.len()), "", String::new())
        };
        let bad = s.warm_edit_ranged("main", &neighbor, m0, &src[m0.0..], "main", false);
        match bad {
            Err(A2xRejectOrResource::Reject(r)) => assert_eq!(
                r.code, "a2x.stale-base",
                "{case}: stale base must be rejected by name"
            ),
            other => panic!("{case}: stale base must reject, got {other:?}"),
        }

        // (3a) whitespace-only edit: the manifest identity REBINDS to the
        // new full source (bytes changed) while NO fragment is produced
        // (semantics unchanged — the 未改函数生产0 gate).
        let prints_cold = s.stats.print;
        let reuse_before = s.stats.fragments_reused_resident;
        let (w_range, w_frag, w_src) = whitespace_edit(&src);
        let w = s
            .warm_edit_ranged("main", &src, w_range, &w_frag, "main", false)
            .unwrap_or_else(|e| panic!("{case}: whitespace edit: {e:?}"));
        assert_binds_source(&w, &w_src, case);
        assert_eq!(
            s.stats.print, prints_cold,
            "{case}: whitespace edit produces nothing"
        );
        assert!(
            s.stats.fragments_reused_resident > reuse_before,
            "{case}: whitespace edit reuses the resident fragment"
        );

        // (3b) semantic edit: exactly ONE fragment produced; identity
        // rebinds to the new full source.
        let (edit_fn, s_range, s_frag, s_src) = semantic_edit(&w_src, case);
        let b = s
            .warm_edit_ranged(&edit_fn, &w_src, s_range, &s_frag, "main", false)
            .unwrap_or_else(|e| panic!("{case}: semantic edit on `{edit_fn}`: {e:?}"));
        assert_binds_source(&b, &s_src, case);
        assert_eq!(s.stats.print, prints_cold + 1, "{case}: one fragment");

        // (4) hit: revisit the ORIGINAL declaration on top of the edited
        // source — the store serves the historical bytes, zero production.
        let prints_mid = s.stats.print;
        let hits_before = s.stats.store_hit_fragment;
        let (_, o_start, o_end) = fn_decls(&src)
            .into_iter()
            .find(|(n, _, _)| n == &edit_fn)
            .unwrap_or_else(|| panic!("{case}: original decl of `{edit_fn}`"));
        let orig_frag = &src[o_start..o_end];
        let a2 = s
            .warm_edit_ranged(&edit_fn, &s_src, s_range, orig_frag, "main", false)
            .unwrap_or_else(|e| panic!("{case}: revisit: {e:?}"));
        assert_eq!(
            s.stats.print, prints_mid,
            "{case}: revisit produces nothing"
        );
        assert!(
            s.stats.store_hit_fragment > hits_before,
            "{case}: revisit is a real store hit"
        );
        assert_eq!(a2.files, a.files, "{case}: historical bytes reused");

        // (5) restart: drop the session, reopen the SAME store root, cold
        // re-assembly reproduces the same session-channel bytes.
        drop(s);
        let mut s2 = open_session(&store);
        let a3 = s2
            .cold_assemble_profile(&src, SourceProfile::Collections, &[], "main", false)
            .unwrap_or_else(|e| panic!("{case}: restart cold: {e:?}"));
        assert_eq!(a3.files, a.files, "{case}: restart re-assembles same bytes");
        assert_eq!(
            a3.manifest, a.manifest,
            "{case}: restart manifest identical"
        );
    }
}
