//! PLAN-1046 T-02 smoke: the Auto program decode on the AV lane, verified
//! against the Rust grammar compiler's own tables and the frozen hex
//! identity binding. This file grows into the full V-01..V-05 conformance
//! suite as the tasks land.

use ace_aac_syntax_conformance::run_case;

fn obs(tag: &str, fname: &str, call: &str) -> String {
    let bytes = run_case(tag, &format!("app.syntax_smoke.{fname}"), call).expect("av run");
    String::from_utf8(bytes).expect("utf8 observable")
}

/// The embedded grammar hexes equal a fresh compile of the repository
/// grammar (identity binding: grammar drift must fail the suite).
#[test]
fn aac1046_grammar_data_identity_binding() {
    let src = include_str!("../../../grammars/auto-source.at");
    // Rebuild the full hex from the chunk literals in the data module and
    // compare byte-for-byte with a fresh compile of the repository grammar.
    for name in ["auto", "age-package", "packages-ct", "packages-modules", "text-record-instances", "scalar"] {
        let fresh = ace_syntax_grammar::compile_profile(src, name)
            .expect("compile")
            .canonical_bytes();
        let hex: String = fresh.iter().map(|b| format!("{b:02x}")).collect();
        let chunks = extract_profile_chunks(
            ace_aac_syntax_conformance::GRAMMAR_DATA_SOURCE,
            name,
        );
        assert!(!chunks.is_empty(), "profile {name} not found in data module");
        let joined: String = chunks.concat();
        assert_eq!(joined, hex, "embedded {name} grammar hex drifted");
    }
}

#[test]
fn aac1046_decode_all_profiles() {
    for name in [
        "auto",
        "age-package",
        "packages-ct",
        "packages-modules",
        "text-record-instances",
        "scalar",
    ] {
        let out = obs("decode", "decode_case", &format!("decode_case(\"{name}\")"));
        // Independent expected values: recompile here and count the tables.
        let p = ace_syntax_grammar::compile_profile(
            include_str!("../../../grammars/auto-source.at"),
            name,
        )
        .expect("compile");
        let expect = format!(
            "OK tok={} rules={} scan={} pratt={}",
            p.tokens.len(),
            p.rules.len(),
            p.scanner.rules.len(),
            p.pratt.len()
        );
        assert_eq!(out, expect, "profile {name}");
    }
}

#[test]
fn aac1046_decode_truncated_refused() {
    for name in ["auto", "scalar"] {
        let out = obs("trunc", "decode_truncated_case", &format!("decode_truncated_case(\"{name}\")"));
        assert!(out.starts_with("REFUSED "), "{name}: {out}");
    }
}

#[test]
fn aac1046_hex_bytes_roundtrip() {
    let out = obs("hexrt", "hex_roundtrip_case", "hex_roundtrip_case(\"scalar\")");
    let fresh = ace_syntax_grammar::compile_profile(
        include_str!("../../../grammars/auto-source.at"),
        "scalar",
    )
    .unwrap()
    .canonical_bytes();
    assert_eq!(out, format!("bytes={}", fresh.len()));
}

#[test]
fn aac1046_byte_len_probe() {
    for name in ["auto", "age-package", "scalar"] {
        let out = obs("blen", "byte_len_case", &format!("byte_len_case(\"{name}\")"));
        println!("{name}: {out}");
        assert!(out.contains("bytes="), "{out}");
    }
}

#[test]
fn aac1046_decode_dbg_probe() {
    for name in ["auto", "scalar"] {
        let out = obs("dbgd", "decode_dbg_case", &format!("decode_dbg_case(\"{name}\")"));
        println!("{name}: {out}");
    }
}

/// Pull the chunk string literals of `pub fn <slug>_parts()` out of the
/// generated data module text (identity binding aid).
fn extract_profile_chunks(src: &str, profile: &str) -> Vec<String> {
    let slug = profile.replace('-', "_");
    let needle = format!("pub fn {slug}_parts()");
    let start = src.find(&needle).expect("profile fn");
    let body = &src[start..];
    let end = body.find("\n}\n").expect("fn end");
    let body = &body[..end];
    let mut out = Vec::new();
    for line in body.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("p.push(\"") {
            if let Some(v) = rest.strip_suffix("\")") {
                out.push(v.to_string());
            }
        }
    }
    out
}
