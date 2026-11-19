//! PLAN-1056 conformance: the Auto support package (exact digests, framed
//! identity, compiler codec) compiled by the Gen0 (Rust-frontend) assembled
//! `packages-aggregates-ct` lane and executed on the AV executor, compared
//! byte-for-byte against independent references:
//!
//!   * SHA-256 — the independent `sha2` crate (FIPS 180-4);
//!   * BLAKE3 — the independent `blake3` crate (unkeyed 32-byte output);
//!   * framed identity — `ace_artifacts::identity::domain_hash`, the exact
//!     Rust reference the artifact layer uses;
//!   * artifact frame — `ace_artifacts::codec::frame_chunk`, the real chunk
//!     wire format consumers read;
//!   * manifest DTO — an independent Rust encoder of the frozen
//!     `aac.manifest/1` layout written from the profile text.
//!
//! No expected value is copied from Auto output.

use ace_aac_support_conformance::{run_program, str_bytes};
use sha2::{Digest, Sha256};

fn sha256_hex_reference(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    format!("{:x}", h.finalize())
}

fn blake3_hex_reference(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}


fn run_hex(tag: &str, src: &str) -> String {
    let v = run_program(tag, src).unwrap_or_else(|e| panic!("{tag}: {e}"));
    String::from_utf8(str_bytes(&v).unwrap_or_else(|e| panic!("{tag}: {e}"))).unwrap()
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}


fn expect_codes(tag: &str, report_hex: &str, want: &[u8]) {
    let got: Vec<u8> = (0..report_hex.len() / 2)
        .map(|i| u8::from_str_radix(&report_hex[2 * i..2 * i + 2], 16).unwrap())
        .collect();
    assert_eq!(got, want, "{tag}");
}

// ---------------------------------------------------------------------------
// V-01: exact digests
// ---------------------------------------------------------------------------

/// SHA-256 (FIPS padding boundaries, embedded NUL, multi-byte UTF-8) and BLAKE3
/// (block boundaries) byte-exactness. ONE driver program computes every digest,
/// so the case costs one Gen0 compile; the test splits the 64-char digests and
/// compares each against the independent reference implementation.
#[test]
fn v01_digest_vectors() {
    let sha_payloads: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"abc".to_vec(),
        b"a\0b".to_vec(),
        "A中😀".as_bytes().to_vec(),
        "α".as_bytes().to_vec(),
        "e\u{0301}".as_bytes().to_vec(),
    ];
    let mut sha_cases = sha_payloads;
    for n in [55usize, 56, 63, 64, 65, 119, 120, 127] {
        sha_cases.push(vec![b'a'; n]);
    }
    let mut b3_cases: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"abc".to_vec(),
        b"a\0b".to_vec(),
        "A中😀".as_bytes().to_vec(),
    ];
    for n in [1usize, 64, 65, 128] {
        b3_cases.push(vec![b'a'; n]);
    }
    let mut want = String::new();
    for p in &sha_cases {
        want.push_str(&sha256_hex_reference(p));
    }
    for p in &b3_cases {
        want.push_str(&blake3_hex_reference(p));
    }
    let got = run_hex(
        "v01-digests",
        "use app.driver.digest_report\nfn main() Str { return digest_report() }",
    );
    assert_eq!(got.len(), want.len(), "digest report length");
    for (i, chunk) in got.as_bytes().chunks(64).enumerate() {
        let name = if i < sha_cases.len() { "sha256" } else { "blake3" };
        let want_chunk = &want[i * 64..(i + 1) * 64];
        assert_eq!(chunk, want_chunk.as_bytes(), "{name} case {i}");
    }
}

/// The BLAKE3 tree path: a 1025-byte payload is two chunks, so the first chunk
/// CV must feed a parent merge and the partial second chunk must produce the
/// ROOT output. BLAKE3 in pure Auto costs ~1M AV steps per KiB, so this is its
/// own case to keep every case's wall time bounded.
#[test]
fn v01_blake3_tree_vectors() {
    let payload = vec![b'a'; 2049];
    let got = run_hex(
        "v01-b3-tree",
        "use app.driver.blake3_tree_report\nfn main() Str { return blake3_tree_report() }",
    );
    assert_eq!(got, blake3_hex_reference(&payload));
}

/// The incremental SHA-256 path equals the one-shot path for the same payload
/// at five boundary splits (the boundary is block buffering, never a C cut).
#[test]
fn v01_sha256_part_split_invariance() {
    let payload = "AutoLang AAC support: exact digest / codec 中 😀 a\0b".as_bytes().to_vec();
    let want = sha256_hex_reference(&payload);
    let got = run_hex(
        "v01-split",
        "use app.driver.split_report\nfn main() Str { return split_report() }",
    );
    assert_eq!(got.len(), 6 * 64, "one-shot plus five splits");
    for (i, chunk) in got.as_bytes().chunks(64).enumerate() {
        assert_eq!(std::str::from_utf8(chunk).unwrap(), want, "split {i}");
    }
}
// ---------------------------------------------------------------------------
/// The framed-identity family: `domain_hash` agreement for four payloads,
/// `[ab,c]` vs `[a,bc]` separation, cross-family separation, and the exact
/// preimage bytes. ONE program computes the whole family (one Gen0 compile).
#[test]
fn v02_identity_report() {
    let mut want = String::new();
    let payloads: [&[u8]; 4] = [
        b"",
        b"a\0b",
        "A中😀".as_bytes(),
        b"fn main() int { return 0 }",
    ];
    for p in payloads {
        want.push_str(&ace_artifacts::identity::domain_hash("snap/v1", &[p, b"app"]));
    }
    want.push_str(&ace_artifacts::identity::domain_hash("f", &[b"ab", b"c"]));
    want.push_str(&ace_artifacts::identity::domain_hash("f", &[b"a", b"bc"]));
    assert_ne!(
        ace_artifacts::identity::domain_hash("f", &[b"ab", b"c"]),
        ace_artifacts::identity::domain_hash("f", &[b"a", b"bc"]),
        "length framing must separate the concatenations"
    );
    want.push_str(&ace_artifacts::identity::domain_hash("snap/v1", &[b"same"]));
    want.push_str(&ace_artifacts::identity::domain_hash("art/v1", &[b"same"]));
    assert_ne!(
        ace_artifacts::identity::domain_hash("snap/v1", &[b"same"]),
        ace_artifacts::identity::domain_hash("art/v1", &[b"same"]),
        "same payload under different families must not reuse a key"
    );
    let mut frame = Vec::new();
    frame.extend_from_slice(b"blake3-1.5");
    frame.push(0);
    frame.extend_from_slice(b"snap/v1");
    frame.push(0);
    frame.extend_from_slice(&2u64.to_le_bytes());
    frame.extend_from_slice(b"ab");
    frame.extend_from_slice(&1u64.to_le_bytes());
    frame.extend_from_slice(b"c");
    want.push_str(&hex_of(&frame));
    let got = run_hex(
        "v02-identity",
        "use app.driver.identity_report\nfn main() Str { return identity_report() }",
    );
    assert_eq!(got, want, "identity family report");
}
// V-03: artifact frame + manifest DTO
// ---------------------------------------------------------------------------

/// Framed bytes are identical to `ace_artifacts::codec::frame_chunk` for every
/// registered chunk kind over a payload family that includes the empty
/// payload, an embedded NUL and multi-byte UTF-8. One driver program frames
/// all (payload, kind) pairs and returns one concatenated report; the test
/// splits it by the reference frame lengths, so a single compile exercises
/// every vector.
#[test]
fn v03_frame_bytes_match_reference() {
    use ace_artifacts::codec::ChunkKind;
    let payloads: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"a\0b".to_vec(),
        "A中😀".as_bytes().to_vec(),
        b"fn main() int { return 0 }".to_vec(),
        vec![b'a'; 100],
    ];
    let kinds = [
        ChunkKind::Hir,
        ChunkKind::DeclSummary,
        ChunkKind::Object,
        ChunkKind::SnapshotRecord,
        ChunkKind::ObjectManifest,
        ChunkKind::LayoutData,
        ChunkKind::SyntaxGrammar,
        ChunkKind::TokenStream,
        ChunkKind::CstPage,
        ChunkKind::CstManifest,
        ChunkKind::SourcePage,
    ];
    let mut want = Vec::new();
    for payload in &payloads {
        for kind in &kinds {
            want.extend_from_slice(&ace_artifacts::codec::frame_chunk(*kind, payload));
        }
    }
    let got = run_hex(
        "v03-frame-report",
        "use app.driver.frame_report\nfn main() Str { return frame_report() }",
    );
    assert_eq!(got, hex_of(&want), "framed wire bytes vs reference");
}
/// The decoder taxonomy: acceptance plus every named rejection class.
#[test]
fn v03_frame_decode_taxonomy() {
    let got = run_hex(
        "v03-frame-codes",
        "use app.driver.frame_case_codes\nfn main() Str { return frame_case_codes() }",
    );
    let want: &[u8] = &[0, 0, 0, 1, 2, 3, 4, 5, 6, 7, 8];
    expect_codes("frame taxonomy", &got, want);
}

/// An independent Rust encoder of the frozen `aac.manifest/1` layout — the
/// oracle for the Auto encoder/decoder (it is written from the profile text,
/// not from the Auto output).
fn manifest_oracle(
    layer: &str,
    anchor: i64,
    pairs: &[(&str, &str)],
    edges: &[(&str, &str)],
) -> Vec<u8> {
    let mut out = vec![1u8, 0, 1, 0];
    out.extend_from_slice(&(layer.len() as u32).to_le_bytes());
    out.extend_from_slice(layer.as_bytes());
    out.extend_from_slice(&anchor.to_le_bytes());
    let mut ps = pairs.to_vec();
    ps.sort();
    out.extend_from_slice(&(ps.len() as u32).to_le_bytes());
    for (k, v) in &ps {
        out.extend_from_slice(&(k.len() as u32).to_le_bytes());
        out.extend_from_slice(k.as_bytes());
        out.extend_from_slice(&(v.len() as u32).to_le_bytes());
        out.extend_from_slice(v.as_bytes());
    }
    let mut es = edges.to_vec();
    es.sort();
    out.extend_from_slice(&(es.len() as u32).to_le_bytes());
    for (i, k) in &es {
        out.extend_from_slice(&(i.len() as u32).to_le_bytes());
        out.extend_from_slice(i.as_bytes());
        out.extend_from_slice(&(k.len() as u32).to_le_bytes());
        out.extend_from_slice(k.as_bytes());
    }
    out
}

/// The manifest family in ONE program: caller-order independence, canonical
/// re-encode after decode, and the i64 anchor boundaries. The oracle is an
/// independent Rust encoder written from the profile text (it sorts pairs and
/// edges itself), never a copy of the Auto output.
#[test]
fn v03_manifest_report() {
    let pairs = [
        ("profile", "core-i32-draft"),
        ("capabilities", "io.read"),
        ("snapshot", "snap0000"),
    ];
    let edges = [("art0002", "provenance"), ("art0001", "required")];
    let a07 = manifest_oracle("A07", 0, &pairs, &edges);
    let amin = manifest_oracle("A00", i64::MIN, &pairs, &edges);
    let amax = manifest_oracle("A00", i64::MAX, &pairs, &edges);
    let mut want = String::new();
    want.push_str(&hex_of(&a07)); // encode-unsorted
    want.push_str(&hex_of(&a07)); // encode-sorted
    want.push_str(&hex_of(&a07)); // reencode-decoded
    want.push_str(&hex_of(&amin)); // anchor-min
    want.push_str(&hex_of(&amax)); // anchor-max
    want.push_str(&hex_of(&amin)); // anchor-min-roundtrip
    let got = run_hex(
        "v03-manifest",
        "use app.driver.manifest_report\nfn main() Str { return manifest_report() }",
    );
    assert_eq!(got, want, "manifest family report");
}
#[test]
fn v03_manifest_taxonomy() {
    let got = run_hex(
        "v03-manifest-codes",
        "use app.driver.manifest_codes\nfn main() Str { return manifest_codes() }",
    );
    // duplicate-key 12 | non-canonical-order 13 | trailing 7 |
    // unsupported-version 10 | unknown-schema 11 | non-ascii 14 | empty 15
    let want: &[u8] = &[12, 13, 7, 10, 11, 14, 15];
    expect_codes("manifest taxonomy", &got, want);
}

// ---------------------------------------------------------------------------
// V-04: resource admission before allocation
// ---------------------------------------------------------------------------

#[test]
fn v04_admission_before_allocation() {
    let got = run_hex(
        "v04-admission",
        "use app.driver.admission_case_codes\nfn main() Str { return admission_case_codes() }",
    );
    let want: &[u8] = &[5, 6, 0, 5, 9, 0];
    expect_codes("admission", &got, want);
}







