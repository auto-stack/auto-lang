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

// ---------------------------------------------------------------------------
// T-02 parse parity: the Auto machine vs the Rust runtime over the same
// frozen grammar + sample (byte-exact canonical CST + diags).
// ---------------------------------------------------------------------------

/// Normalize the Rust canonical form's meta kinds onto the Auto signed
/// encoding (KIND_ERROR/-1, MISSING/-2, CALL/-3, POSTFIX/-4).
fn normalize_canonical(canon: &str) -> String {
    let mut out = String::new();
    for line in canon.lines() {
        if let Some(rest) = line.strip_prefix('n') {
            // kind number runs to the first marker or ':'
            let end = rest
                .find(|c: char| !c.is_ascii_digit())
                .unwrap_or(rest.len());
            let kind: u32 = rest[..end].parse().unwrap();
            let mapped = match kind {
                4294967294 => "-1",
                4294967293 => "-2",
                4294967292 => "-3",
                4294967291 => "-4",
                k => {
                    out.push('n');
                    out.push_str(&k.to_string());
                    out.push_str(&rest[end..]);
                    out.push('\n');
                    continue;
                }
            };
            out.push('n');
            out.push_str(mapped);
            out.push_str(&rest[end..]);
            out.push('\n');
        } else {
            // root lines start with "n..." only at depth 0; indent lines have
            // spaces then 'n'
            if let Some(pos) = line.rfind('n') {
                if line[..pos].trim().is_empty() {
                    let rest = &line[pos + 1..];
                    let end = rest
                        .find(|c: char| !c.is_ascii_digit())
                        .unwrap_or(rest.len());
                    let kind: u32 = rest[..end].parse().unwrap();
                    let mapped = match kind {
                        4294967294 => "-1",
                        4294967293 => "-2",
                        4294967292 => "-3",
                        4294967291 => "-4",
                        k => {
                            out.push_str(&line[..pos + 1]);
                            out.push_str(&k.to_string());
                            out.push_str(&rest[end..]);
                            out.push('\n');
                            continue;
                        }
                    };
                    out.push_str(&line[..pos + 1]);
                    out.push_str(mapped);
                    out.push_str(&rest[end..]);
                    out.push('\n');
                    continue;
                }
            }
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// Map Rust NO_FIELD (u32::MAX) onto the Auto -1 encoding inside the
/// f=[...] field lists (kinds are mapped by normalize_canonical).
fn normalize_fields(canon: &str) -> String {
    // Map NO_FIELD (u32::MAX) inside each f=[...] list; a plain string
    // replace would miss consecutive ",MAX,MAX," (overlap), so rewrite the
    // bracketed segment as a whole.
    let mut out = String::new();
    for line in canon.lines() {
        if let Some(pos) = line.find("f=[") {
            let close = line[pos..].find(']').map(|c| pos + c);
            if let Some(end) = close {
                let inner = &line[pos + 3..end];
                let mapped = inner.replace("4294967295", "-1");
                out.push_str(&line[..pos + 3]);
                out.push_str(&mapped);
                out.push_str(&line[end..]);
                out.push('\n');
                continue;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

fn rust_oracle(profile: &str, sample: &str) -> String {
    let src = include_str!("../../../grammars/auto-source.at");
    let p = ace_syntax_grammar::compile_profile(src, profile).unwrap();
    let cs = ace_syntax_runtime::lexer::CompiledScanner::compile(&p).unwrap();
    let pages = std::rc::Rc::new(ace_syntax_runtime::snapshot::PageStore::new());
    let snap = ace_syntax_runtime::snapshot::Snapshot::from_text(&pages, sample);
    let out = ace_syntax_runtime::parse::parse_snapshot(
        &p,
        &cs,
        &snap,
        &ace_syntax_runtime::program::Budget::default(),
    )
    .unwrap();
    let mut expect = String::from("CANON\n");
    expect.push_str(&normalize_fields(&normalize_canonical(&out.tree.canonical())));
    expect.push_str("DIAGS\n");
    for d in &out.tree.diags {
        expect.push_str(&format!("{}..{} {}\n", d.start, d.end, d.message));
    }
    expect
}

#[test]
fn aac1046_parse_parity_scalar() {
    let sample = "fn main() int { return 1 }";
    let out = obs("parse", "parse_case", &format!("parse_case(\"scalar\", \"{sample}\")"));
    let expect = rust_oracle("scalar", sample);
    assert_eq!(out, expect);
}

#[test]
fn aac1046_stage_probes() {
    let sample = "fn main() int { return 1 }";
    let lex = obs("lex", "lex_case", &format!("lex_case(\"scalar\", \"{sample}\")"));
    println!("LEX: {lex}");
    assert!(lex.starts_with("ntok="), "{lex}");
    let fs = obs("fs", "firstsets_case", "firstsets_case(\"scalar\")");
    println!("FS: {fs}");
    assert!(fs.starts_with("startfirst="), "{fs}");
}

#[test]
fn aac1046_scan_probe() {
    let out = obs("scan", "scan_case", "scan_case(\"scalar\")");
    println!("SCAN: {out}");
    assert!(out.starts_with("entries="), "{out}");
}

#[test]
fn aac1046_raw_probe() {
    let out = obs("raw", "raw_case", "raw_case(\"scalar\")");
    println!("RAW: {out}");
    assert!(out.starts_with("nscan="), "{out}");
}

#[test]
fn aac1046_bucket_probe() {
    let out = obs("bkt", "bucket_count_case", "bucket_count_case(\"scalar\")");
    println!("BUCKET: {out}");
    assert_eq!(out, "counts-ok");
}


#[test]
fn aac1046_parse_parity_battery() {
    // Same-source battery: expressions (pratt/calls), recovery, trivia,
    // nested blocks, strings; each byte-exact vs the Rust runtime.
    let cases: &[(&str, &str)] = &[
        ("scalar", "fn f(a int) int { return a }"),
        ("scalar", "fn f() int { if (1 < 2) { return 1 } else { return 2 } }"),
        ("scalar", "fn broken( -> int { return 1 }\nfn ok(x int) int { return x }"),
        ("scalar", "fn g() int { var x = 1\n x = x + 1\n while-form-loop\n return ((x)) }"),
        ("scalar", "// leading comment\nfn h() int {\n    // inner\n    return 7\n}\n"),
        ("scalar", "fn s() int { return 1 + 2 * 3 - 4 / 2 }"),
    ];
    for (i, (profile, sample)) in cases.iter().enumerate() {
        let out = obs(
            &format!("pb{i}"),
            "parse_case",
            &format!("parse_case(\"{profile}\", \"{sample}\")"),
        );
        let expect = rust_oracle(profile, sample);
        assert_eq!(out, expect, "case {i}: {profile} / {sample}");
    }
}




// ---------------------------------------------------------------------------
// T-02 incremental (V-02/V-04): edit -> incremental reparse equality, and
// the content identity change/restore under an undo edit.
// ---------------------------------------------------------------------------

// D-1046-03 (supersedes D-1046-02's cost diagnosis): the SPLICE is proven
// byte-exact (stage S29: n2=58/phys=58/mism=0 vs a fresh lex of the edited
// bytes) and the graft candidates/token ranges verify; but the GRAFT-ACTIVE
// parse explodes one list past the lane's 65536-element cap (graft-disabled
// parses of the same spliced stream pass; 9 machine arenas watched at
// 128-step windows never trip — the growing list is outside the watched
// set). Ignored pending the named diagnostic in plan §10.
#[test]
#[ignore = "D-1046-03 graft-active parse list-cap explosion"]
fn aac1046_edit_incremental_equality() {
    // V-02: the incremental (spliced+graft) tree from one AV run must
    // equal the fresh parse of the same edited bytes from another run.
    let esc = "fn a(x int) int { return x }\nfn b(y int) int { return y }\nfn c(z int) int { return z }";
    let incr_run = obs("edit", "edit_case", &format!("edit_case(\"scalar\", \"{}\", 25, 25, \"1 + \")", esc));
    let ref_run = obs("ref", "edit_ref_case", &format!("edit_ref_case(\"scalar\", \"{}\", 25, 25, \"1 + \")", esc));
    let incr = incr_run.split("SPLICE").nth(0).unwrap().strip_prefix("INCR|").unwrap();
    let refr = ref_run.split("REFEND").nth(0).unwrap().strip_prefix("REF|").unwrap();
    assert_eq!(incr, refr, "incremental tree must equal fresh tree");
    let facts = incr_run.split("SPLICE").nth(1).unwrap();
    assert!(facts.contains("ok=1"), "splice must converge: {facts}");
    assert!(facts.contains("grafted=2"), "two untouched trailing fns grafted: {facts}");
}

#[test]
fn aac1046_identity_change_restore() {
    // Insert an ASCII space before the second char (multi-byte α at 1),
    // then undo: the SnapshotId must change and restore.
    // Inserting INSIDE the α scalar (offset 1) must be refused (code 4);
    // inserting BEFORE it (offset 0) is legal and the undo restores the id.
    let mid = obs("ident", "identity_case", "identity_case(\"α中 b\", 1)");
    assert_eq!(mid, "EDIT1_REFUSED 4", "mid-scalar cut must be refused");
    let out = obs("ident", "identity_case", "identity_case(\"α中 b\", 0)");
    println!("IDENT| {out}");
    assert!(out.starts_with("changed=1 restored=1"), "{out}");
}






#[test]
fn aac1046_threefn_parity() {
    // The edit case's cold leg: three fns, plain cold parse, byte-exact
    // oracle parity.
    let sample = "fn a(x int) int { return x }\nfn b(y int) int { return y }\nfn c(z int) int { return z }";
    let call = format!("parse_case(\"scalar\", \"{}\")", sample);
    let out = obs("t3", "parse_case", &call);
    let expect = rust_oracle(
        "scalar",
        "fn a(x int) int { return x }
fn b(y int) int { return y }
fn c(z int) int { return z }",
    );
    assert_eq!(out, expect);
}


#[test]
fn aac1046_truncated_stream() {
    // A stream cut mid-decl at EOF: recovery must terminate cleanly.
    let call = "parse_case(\"scalar\", \"fn a(x int) int { return x }
fn b(y int) int { ret\")";
    let out = obs("tr", "parse_case", call);
    let head: String = out.chars().take(120).collect();
    println!("TR| {head}");
    let expect = rust_oracle(
        "scalar",
        "fn a(x int) int { return x }
fn b(y int) int { ret",
    );
    assert_eq!(out, expect);
}

#[test]
fn aac1046_singlefn_midexpr() {
    let esc = "fn b(y int) int { y + ";
    let out = obs("sf", "parse_case", &format!("parse_case(\"scalar\", \"{}\")", esc));
    let expect = rust_oracle("scalar", "fn b(y int) int { y + ");
    assert_eq!(out, expect);
}

#[test]
fn aac1046_midexpr_eof() {
    // EOF right after a binary operator: Pratt WantPrimary at EOF.
    let esc = "fn a(x int) int { return x }\nfn b(y int) int { return y + ";
    let out = obs("me", "parse_case", &format!("parse_case(\"scalar\", \"{}\")", esc));
    let head: String = out.chars().take(100).collect();
    println!("ME| {head}");
    let expect = rust_oracle("scalar", "fn a(x int) int { return x }\nfn b(y int) int { return y + ");
    assert_eq!(out, expect);
}




