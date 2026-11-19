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

/// Auto string literal for exact UTF-8 bytes.
fn lit(bytes: &[u8]) -> String {
    let text = std::str::from_utf8(bytes).expect("vector payloads are UTF-8 text");
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn run_hex(tag: &str, src: &str) -> String {
    let v = run_program(tag, src).unwrap_or_else(|e| panic!("{tag}: {e}"));
    String::from_utf8(str_bytes(&v).unwrap_or_else(|e| panic!("{tag}: {e}"))).unwrap()
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest_case(func: &str, payload: &[u8]) -> String {
    let src = format!(
        "use app.digest.{func}\nfn main() Str {{ return {func}({}) }}",
        lit(payload)
    );
    run_hex(&format!("digest-{func}"), &src)
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

/// SHA-256 byte-exactness over the frozen payload family — empty, an embedded
/// NUL, multi-byte UTF-8 and the FIPS padding boundaries.
#[test]
fn v01_sha256_exact_vectors() {
    let mut cases: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"abc".to_vec(),
        b"a\0b".to_vec(),
        "A中😀".as_bytes().to_vec(),
        "α".as_bytes().to_vec(),
        "e\u{0301}".as_bytes().to_vec(),
    ];
    for n in [55usize, 56, 63, 64, 65, 119, 120, 127] {
        cases.push(vec![b'a'; n]);
    }
    for (i, payload) in cases.iter().enumerate() {
        let got = digest_case("sha256_hex", payload);
        assert_eq!(got, sha256_hex_reference(payload), "sha256 case {i}");
    }
}

/// The incremental path (part-split input) equals the one-shot path for the
/// same payload: the boundary is block buffering, never a C-string cut.
#[test]
fn v01_sha256_part_split_invariance() {
    let payload = "AutoLang AAC support: exact digest / codec 中 😀 a\0b".as_bytes().to_vec();
    let one_shot = digest_case("sha256_hex", &payload);
    assert_eq!(one_shot, sha256_hex_reference(&payload));
    let text = std::str::from_utf8(&payload).unwrap();
    let bounds: Vec<usize> = text
        .char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
        .collect();
    let picks = [
        1usize,
        2,
        bounds.len() / 3,
        2 * bounds.len() / 3,
        bounds.len() - 1,
    ];
    for (i, &p) in picks.iter().enumerate() {
        let a = bounds[p.min(bounds.len() - 1)];
        let b = bounds[(p + 1).min(bounds.len() - 1)].max(a);
        let parts = [&text[..a], &text[a..b], &text[b..]];
        let parts_src = parts
            .iter()
            .map(|p| format!("    ps.push({})", lit(p.as_bytes())))
            .collect::<Vec<_>>()
            .join("\n");
        let src = format!(
            "use app.digest.sha256_hex_parts\nfn main() Str {{\n    var ps = List<Str>.new()\n{parts_src}\n    return sha256_hex_parts(ps)\n}}"
        );
        let got = run_hex(&format!("v01-sha-split-{i}"), &src);
        assert_eq!(got, one_shot, "split {i} must match one-shot");
    }
}

/// BLAKE3 byte-exactness across the chunk/block/tree boundaries (the algorithm
/// chunks at 1024 bytes and 64-byte blocks, so these lengths exercise the
/// internal chunking path against the reference).
#[test]
fn v01_blake3_exact_vectors() {
    let mut cases: Vec<Vec<u8>> = vec![
        b"".to_vec(),
        b"abc".to_vec(),
        b"a\0b".to_vec(),
        "A中😀".as_bytes().to_vec(),
    ];
    for n in [1usize, 64, 65, 128, 1023, 1024, 1025, 3000] {
        cases.push(vec![b'a'; n]);
    }
    for (i, payload) in cases.iter().enumerate() {
        let got = digest_case("blake3_hex", payload);
        assert_eq!(got, blake3_hex_reference(payload), "blake3 case {i}");
    }
}

// ---------------------------------------------------------------------------
// V-02: framed identity
// ---------------------------------------------------------------------------

#[test]
fn v02_identity_framing_matches_rust_reference() {
    let payloads: [&[u8]; 4] = [
        b"",
        b"a\0b",
        "A中😀".as_bytes(),
        b"fn main() int { return 0 }",
    ];
    for (i, payload) in payloads.iter().enumerate() {
        let text = std::str::from_utf8(payload).unwrap();
        let want = ace_artifacts::identity::domain_hash("snap/v1", &[payload, b"app"]);
        let src = format!(
            "use app.digest.identity_hex2\nfn main() Str {{ return identity_hex2(\"snap/v1\", {}, \"app\") }}",
            lit(text.as_bytes())
        );
        let got = run_hex(&format!("v02-frame-{i}"), &src);
        assert_eq!(got, want, "identity frame case {i}");
    }
    let ab_c = run_hex(
        "v02-ab-c",
        "use app.digest.identity_hex2\nfn main() Str { return identity_hex2(\"f\", \"ab\", \"c\") }",
    );
    let a_bc = run_hex(
        "v02-a-bc",
        "use app.digest.identity_hex2\nfn main() Str { return identity_hex2(\"f\", \"a\", \"bc\") }",
    );
    assert_ne!(ab_c, a_bc, "length framing must separate concatenations");
    assert_eq!(ab_c, ace_artifacts::identity::domain_hash("f", &[b"ab", b"c"]));
    assert_eq!(a_bc, ace_artifacts::identity::domain_hash("f", &[b"a", b"bc"]));
    let fam_a = run_hex(
        "v02-fam-a",
        "use app.digest.identity_hex\nfn main() Str { return identity_hex(\"snap/v1\", \"same\") }",
    );
    let fam_b = run_hex(
        "v02-fam-b",
        "use app.digest.identity_hex\nfn main() Str { return identity_hex(\"art/v1\", \"same\") }",
    );
    assert_ne!(fam_a, fam_b);
    assert_eq!(fam_a, ace_artifacts::identity::domain_hash("snap/v1", &[b"same"]));
    assert_eq!(fam_b, ace_artifacts::identity::domain_hash("art/v1", &[b"same"]));
}

#[test]
fn v02_identity_frame_bytes_are_exact() {
    let got = run_hex(
        "v02-frame-bytes",
        "use app.digest.identity_frame_hex\nfn main() Str { return identity_frame_hex(\"snap/v1\", \"ab\", \"c\") }",
    );
    let mut want = Vec::new();
    want.extend_from_slice(b"blake3-1.5");
    want.push(0);
    want.extend_from_slice(b"snap/v1");
    want.push(0);
    want.extend_from_slice(&2u64.to_le_bytes());
    want.extend_from_slice(b"ab");
    want.extend_from_slice(&1u64.to_le_bytes());
    want.extend_from_slice(b"c");
    assert_eq!(got, hex_of(&want));
}

// ---------------------------------------------------------------------------
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

#[test]
fn v03_manifest_canonical_and_i64_anchor() {
    let pairs = [
        ("profile", "core-i32-draft"),
        ("capabilities", "io.read"),
        ("snapshot", "snap0000"),
    ];
    let edges = [("art0002", "provenance"), ("art0001", "required")];
    let want_a07 = hex_of(&manifest_oracle("A07", 0, &pairs, &edges));
    for case in ["encode-unsorted", "encode-sorted", "reencode-decoded"] {
        let got = run_hex(
            &format!("v03-manifest-{case}"),
            &format!(
                "use app.driver.manifest_case\nfn main() Str {{ return manifest_case(\"{case}\") }}"
            ),
        );
        assert_eq!(got, want_a07, "{case}");
    }
    let want_min = hex_of(&manifest_oracle("A00", i64::MIN, &pairs, &edges));
    let want_max = hex_of(&manifest_oracle("A00", i64::MAX, &pairs, &edges));
    for (case, want) in [
        ("anchor-min", &want_min),
        ("anchor-max", &want_max),
        ("anchor-min-roundtrip", &want_min),
    ] {
        let got = run_hex(
            &format!("v03-manifest-{case}"),
            &format!(
                "use app.driver.manifest_case\nfn main() Str {{ return manifest_case(\"{case}\") }}"
            ),
        );
        assert_eq!(&got, want, "{case}");
    }
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



