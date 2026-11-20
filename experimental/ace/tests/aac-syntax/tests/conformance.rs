//! PLAN-1046 T-02 smoke: the Auto program decode on the AV lane, verified
//! against the Rust grammar compiler's own tables and the frozen hex
//! identity binding. This file grows into the full V-01..V-05 conformance
//! suite as the tasks land.

#![cfg(all(windows, target_arch = "x86_64"))]

use ace_aac_syntax_conformance::{run_native_case, run_native_projection, SYNTAX_PROFILES};

fn obs(case: &str, profile: &str, source: &str) -> String {
    run_native_case(case, profile, source, None).expect("prepared AAC native run")
}

/// The embedded grammar hexes equal a fresh compile of the repository
/// grammar (identity binding: grammar drift must fail the suite).
#[test]
fn aac1046_grammar_data_identity_binding() {
    let src = include_str!("../../../grammars/auto-source.at");
    // Rebuild the full hex from the chunk literals in the data module and
    // compare byte-for-byte with a fresh compile of the repository grammar.
    for name in SYNTAX_PROFILES {
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
    for name in SYNTAX_PROFILES {
        let out = obs("decode", name, "");
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
        let out = obs("decode-truncated", name, "");
        assert!(out.starts_with("REFUSED "), "{name}: {out}");
    }
}

#[test]
fn aac1046_hex_bytes_roundtrip() {
    let out = obs("hex-roundtrip", "scalar", "");
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
        let out = obs("byte-length", name, "");
        println!("{name}: {out}");
        assert!(out.contains("bytes="), "{out}");
    }
}

#[test]
fn aac1046_decode_dbg_probe() {
    for name in ["auto", "scalar"] {
        let out = obs("decode-debug", name, "");
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

struct OracleTokenText<'a>(&'a [u8]);

impl ace_syntax_runtime::parse::CtxTokenText for OracleTokenText<'_> {
    fn token_equals(&self, start: u64, end: u64, want: &str) -> bool {
        self.0.get(start as usize..end as usize) == Some(want.as_bytes())
    }
}

fn rust_parse(
    profile: &str,
    sample: &str,
) -> (
    ace_syntax_runtime::program::GrammarProgram,
    ace_syntax_runtime::parse::ParseOutput,
) {
    let src = include_str!("../../../grammars/auto-source.at");
    let p = ace_syntax_grammar::compile_profile(src, profile).unwrap();
    let cs = ace_syntax_runtime::lexer::CompiledScanner::compile(&p).unwrap();
    let pages = std::rc::Rc::new(ace_syntax_runtime::snapshot::PageStore::new());
    let snap = ace_syntax_runtime::snapshot::Snapshot::from_text(&pages, sample);
    let out = ace_syntax_runtime::parse::parse_snapshot_with_ctx(
        &p,
        &cs,
        &snap,
        &ace_syntax_runtime::program::Budget::default(),
        Some(&OracleTokenText(sample.as_bytes())),
    )
    .unwrap();
    (p, out)
}

fn rust_oracle(profile: &str, sample: &str) -> String {
    let (_, out) = rust_parse(profile, sample);
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
    let out = obs("parse", "scalar", sample);
    let expect = rust_oracle("scalar", sample);
    assert_eq!(out, expect);
}

#[test]
fn aac1046_stage_probes() {
    let sample = "fn main() int { return 1 }";
    let lex = obs("lex", "scalar", sample);
    println!("LEX: {lex}");
    assert!(lex.starts_with("ntok="), "{lex}");
    let fs = obs("first-sets", "scalar", "");
    println!("FS: {fs}");
    assert!(fs.starts_with("startfirst="), "{fs}");
}

#[test]
fn aac1046_scan_probe() {
    let out = obs("scan", "scalar", "");
    println!("SCAN: {out}");
    assert!(out.starts_with("entries="), "{out}");
}

#[test]
fn aac1046_raw_probe() {
    let out = obs("raw", "scalar", "");
    println!("RAW: {out}");
    assert!(out.starts_with("nscan="), "{out}");
}

#[test]
fn aac1046_bucket_probe() {
    let out = obs("bucket-count", "scalar", "");
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
        let out = obs("parse", profile, sample);
        let expect = rust_oracle(profile, sample);
        assert_eq!(out, expect, "case {i}: {profile} / {sample}");
    }
}




// ---------------------------------------------------------------------------
// T-02 incremental (V-02/V-04): edit -> incremental reparse equality, and
// the content identity change/restore under an undo edit.
// ---------------------------------------------------------------------------

#[test]
fn aac1046_edit_incremental_equality() {
    use ace_syntax_runtime::{
        edit::incremental_reparse,
        lexer::CompiledScanner,
        parse::parse_snapshot,
        program::Budget,
        snapshot::{Edit, PageStore, Snapshot},
    };

    let original = "fn a(x int) int { return x }\nfn b(y int) int { return y }\nfn c(z int) int { return z }";
    let mut edited = original.to_owned();
    edited.insert_str(25, "1 + ");
    let grammar = ace_syntax_grammar::compile_profile(
        include_str!("../../../grammars/auto-source.at"),
        "scalar",
    )
    .unwrap();
    let scanner = CompiledScanner::compile(&grammar).unwrap();
    let pages = std::rc::Rc::new(PageStore::new());
    let old = Snapshot::from_text(&pages, original);
    let new = Snapshot::from_text(&pages, &edited);
    let budget = Budget::default();
    let old_parse = parse_snapshot(&grammar, &scanner, &old, &budget).unwrap();
    let splice = incremental_reparse(
        &grammar,
        &scanner,
        &old_parse,
        &old,
        &new,
        &[Edit {
            start: 25,
            end: 25,
            insert: b"1 + ".to_vec(),
        }],
        &budget,
    )
    .unwrap();
    assert!(splice.fallback.is_none());
    assert!(splice.reused_decls > 0, "fixture must exercise actual grafting");

    let incr_run = run_native_case("edit", "scalar", original, Some((25, 25, "1 + ")))
        .expect("prepared AAC native edit");
    let (tree, facts) = incr_run
        .split_once("SPLICE")
        .unwrap_or_else(|| panic!("missing splice result: {incr_run}"));
    let tree = tree
        .strip_prefix("INCR|")
        .unwrap_or_else(|| panic!("missing incremental tree: {tree}"));
    assert_eq!(tree, rust_oracle("scalar", &edited), "incremental vs fresh");
    assert_eq!(
        facts,
        format!(" ok=1 fb=0 rescan={} grafted={}", splice.rescan_bytes, splice.reused_decls),
        "actual rescan bytes and reuse count must match the Rust incremental parser",
    );
}


#[test]
fn aac1046_identity_change_restore() {
    // Insert an ASCII space before the second char (multi-byte α at 1),
    // then undo: the SnapshotId must change and restore.
    // Inserting INSIDE the α scalar (offset 1) must be refused (code 4);
    // inserting BEFORE it (offset 0) is legal and the undo restores the id.
    let mid = run_native_case("identity", "", "α中 b", Some((1, 1, ""))).unwrap();
    assert_eq!(mid, "EDIT1_REFUSED 4", "mid-scalar cut must be refused");
    let out = run_native_case("identity", "", "α中 b", Some((0, 0, ""))).unwrap();
    println!("IDENT| {out}");
    assert!(out.starts_with("changed=1 restored=1"), "{out}");
}






#[test]
fn aac1046_threefn_parity() {
    // The edit case's cold leg: three fns, plain cold parse, byte-exact
    // oracle parity.
    let sample = "fn a(x int) int { return x }\nfn b(y int) int { return y }\nfn c(z int) int { return z }";
    let out = obs("parse", "scalar", sample);
    let expect = rust_oracle("scalar", sample);
    assert_eq!(out, expect);
}


#[test]
fn aac1046_truncated_stream() {
    // A stream cut mid-decl at EOF: recovery must terminate cleanly.
    let sample = "fn a(x int) int { return x }\nfn b(y int) int { ret";
    let out = obs("parse", "scalar", sample);
    let expect = rust_oracle("scalar", sample);
    assert_eq!(out, expect);
}

#[test]
fn aac1046_singlefn_midexpr() {
    let esc = "fn b(y int) int { y + ";
    let out = obs("parse", "scalar", esc);
    let expect = rust_oracle("scalar", "fn b(y int) int { y + ");
    assert_eq!(out, expect);
}

#[test]
fn aac1046_midexpr_eof() {
    // EOF right after a binary operator: Pratt WantPrimary at EOF.
    let esc = "fn a(x int) int { return x }\nfn b(y int) int { return y + ";
    let out = obs("parse", "scalar", esc);
    let expect = rust_oracle("scalar", "fn a(x int) int { return x }\nfn b(y int) int { return y + ");
    assert_eq!(out, expect);
}

#[test]
fn aac1046_typed_age_and_scheduler_profile_parity() {
    let cases = [
        ("auto", "fn adapt(a ?int, b ~Iter<int>) !?int { return a }"),
        (
            "age-package",
            "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n",
        ),
        (
            "scheduler",
            "task Worker { on { _ -> { reply(1) } } }\nfn worker() int { yield 1\nreturn 1 }\nfn entry()! int { go worker()\nreturn await worker() }",
        ),
    ];
    for (profile, source) in cases {
        let expected = rust_oracle(profile, source);
        assert!(expected.ends_with("DIAGS\n"), "fixture must parse: {expected}");
        assert_eq!(obs("parse", profile, source), expected, "{profile}");
    }
}

#[test]
fn aac1046_source_bytes_and_lossless_reconstruction() {
    let source = "// α 😀\r\nfn main() int { return 1 }\r\n";
    let actual = obs("parse-source", "scalar", source);
    assert_eq!(actual, rust_oracle("scalar", source));
    let invalid = b"// \xf0\x9f\x98\nfn main() int { return 1 }";
    let actual = run_native_case("parse-source", "scalar", invalid, None).unwrap();
    assert_eq!(actual, "SOURCE_ERR UTF8");
}

#[test]
fn aac1046_depth_and_work_budget_admission() {
    let shallow = "fn main() int { return 1 }";
    let deep = format!("fn main() int {{ return {}1{} }}", "(".repeat(24), ")".repeat(24));
    assert_eq!(
        obs("parse-source-depth", "scalar", shallow),
        rust_oracle("scalar", shallow)
    );
    assert_eq!(
        obs("parse-source-depth", "scalar", &deep),
        "PARSE_ERR 2"
    );
    assert_eq!(obs("parse-source-work", "scalar", shallow), "PARSE_ERR 3");
}

#[test]
fn aac1046_edit_unicode_span_and_undo() {
    let source = "// α note\r\nfn a(x int) int { return x }\r\nfn b(y int) int { return y }\r\n";
    let at = source.find('α').unwrap();
    let mut edited = source.to_owned();
    edited.insert(at, ' ');
    for (input, result, end, insert) in [
        (source, edited.as_str(), at, " "),
        (edited.as_str(), source, at + 1, ""),
    ] {
        let out = run_native_case(
            "edit", "scalar", input, Some((at as i32, end as i32, insert)),
        ).unwrap();
        let (tree, _) = out.split_once("SPLICE").unwrap_or_else(|| panic!("{out}"));
        assert_eq!(tree.strip_prefix("INCR|").unwrap(), rust_oracle("scalar", result));
    }
}

fn rust_ast_report(profile: &str, source: &str, source_id: i32, origin_id: i32) -> String {
    use ace_syntax_runtime::cst::{FLAG_TRIVIA, NO_FIELD};
    use ace_syntax_runtime::program::{
        KIND_CALL, KIND_ERROR, KIND_MISSING, KIND_POSTFIX, RULE_KIND_BASE, TOKEN_KIND_BASE,
    };
    use std::fmt::Write;

    let (grammar, parsed) = rust_parse(profile, source);
    let tree = parsed.tree;
    let mut report = String::from("AST\n");
    let mut stack = vec![(tree.root, -1, NO_FIELD)];
    let mut next = 0;
    while let Some((node_id, parent, field)) = stack.pop() {
        let node = tree.node(node_id);
        if node.flags & FLAG_TRIVIA != 0 {
            continue;
        }
        let kind = match node.kind {
            KIND_ERROR => "error",
            KIND_MISSING => "missing",
            KIND_CALL => "call",
            KIND_POSTFIX => "postfix",
            k if k >= RULE_KIND_BASE => &grammar.rules[(k - RULE_KIND_BASE) as usize].name,
            k => &grammar.tokens[(k - TOKEN_KIND_BASE) as usize].name,
        };
        let role = if field == NO_FIELD {
            "_"
        } else {
            &grammar.field_names[field as usize]
        };
        let (start, end) = tree.abs_range(node_id);
        writeln!(
            report,
            "{parent} {role} {kind} {start}..{end} {source_id} {origin_id} {}",
            node.flags,
        ).unwrap();
        let children = tree.children(node_id);
        let fields = tree.child_fields(node_id);
        stack.extend(children.into_iter().zip(fields).rev().map(|(n, f)| (n, next, f)));
        next += 1;
    }
    report.push_str("DIAGS\n");
    for d in &tree.diags {
        writeln!(report, "{}..{} {}", d.start, d.end, d.message).unwrap();
    }
    report
}

#[test]
fn aac1046_ast_source_and_domain_projection() {
    for (profile, source, file, origin) in [
        ("auto", "fn adapt(a ?int, b ~Iter<int>, c Future<int>) !?int { return a }", 31, 7),
        ("age-package", "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\n", 32, 0),
        ("packages-ct", "fn id<T>(x T) T { return x }", 33, 0),
        ("scalar", "fn broken( -> int { return 1 }\nfn ok(x int) int { return x }", 34, 0),
    ] {
        let actual = run_native_projection(profile, source, (file, origin), None).unwrap();
        assert_eq!(actual, rust_ast_report(profile, source, file, origin), "{profile}");
    }
}

#[test]
fn aac1046_ast_type_roles_match_rust_projection() {
    use ace_core::source::ast::{Item, TypeRef};
    use ace_core::source::{cst::parse_program_via_cst, SourceProfile};

    let code = "fn adapt(a ?int, b ~Iter<int>, c Future<int>) !?int { return a }";
    let rust = parse_program_via_cst(code, SourceProfile::Auto).unwrap();
    let [Item::Fn(function)] = &rust.items[..] else {
        panic!("fixture must project exactly one function")
    };
    assert!(matches!(&function.params[0].ty.kind, TypeRef::Optional(_)));
    assert!(matches!(&function.params[1].ty.kind, TypeRef::Stream(_)));
    assert!(matches!(&function.params[2].ty.kind, TypeRef::Generic { name, .. } if name.name == "Future"));
    assert!(matches!(&function.ret.kind, TypeRef::ResultBang(inner) if matches!(inner.kind, TypeRef::Optional(_))));
    let actual = run_native_projection("auto", code, (9, 2), None).unwrap();
    for (kind, span) in [
        ("optional-ty", function.params[0].ty.span),
        ("tilde-ty", function.params[1].ty.span),
        ("generic-ty", function.params[2].ty.span),
        ("err-ty", function.ret.span),
    ] {
        let role_and_span = format!(" {kind} {}..{} 9 2 ", span.start, span.end);
        assert!(actual.contains(&role_and_span), "missing projected type {role_and_span}");
    }
}


#[test]
fn aac1046_incremental_ast_retains_origin_and_byte_spans() {
    let source = "// α note\r\nfn a(x int) int { return x }\r\nfn b(y int) int { return y }\r\n";
    let at = source.find('α').unwrap();
    let mut edited = source.to_owned();
    edited.insert(at, ' ');
    for (input, expected, end, insert) in [
        (source, edited.as_str(), at, " "),
        (edited.as_str(), source, at + 1, ""),
    ] {
        let actual = run_native_projection(
            "scalar", input, (19, 3), Some((at as i32, end as i32, insert)),
        ).unwrap();
        let (ast, facts) = actual.split_once("SPLICE").unwrap_or_else(|| panic!("{actual}"));
        assert_eq!(ast.strip_prefix("INCR|").unwrap(), rust_ast_report("scalar", expected, 19, 3));
        assert!(facts.starts_with(" ok=1 fb=0 "), "fixture must exercise splicing: {facts}");
    }
}
