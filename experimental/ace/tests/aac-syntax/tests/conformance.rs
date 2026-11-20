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


#[test]
fn aac1046_module_provider_and_conditional_catalog() {
    use ace_aac_syntax_conformance::run_native_catalog;
    use ace_core::source::packages::{classify_provider_file_name, ProviderFileKind};

    let files = [
        (
            "entry.at",
            "use math.calc\n#in sys is current {\n Windows -> use win.call\n Linux -> use lnx.call\n}\nfn main() int { return calc() }",
        ),
        ("math.at", "pub spec fn calc() int"),
        ("math.at.at", "fn calc() int { return 1 }"),
        ("math.c.at", "fn calc() int { return 2 }"),
        ("math.py.at", "fn calc() int { return 3 }"),
        ("math.rs.at", "fn calc() int { return 4 }"),
    ];
    let inventory: Vec<_> = files.iter().map(|(name, _)| *name).collect();
    let out = run_native_catalog("packages-ct", "app", &files, &inventory).unwrap();
    let expected: Vec<_> = inventory.iter().enumerate().map(|(idx, name)| {
        let (kind, module) = classify_provider_file_name(name).unwrap();
        let provider = match kind {
            ProviderFileKind::Public => "public",
            ProviderFileKind::Provider(kind) => kind.label(),
        };
        format!("MODULE {idx} app.{module} {provider} {name}")
    }).collect();
    assert_eq!(out.lines().filter(|line| line.starts_with("MODULE ")).collect::<Vec<_>>(), expected);
    assert!(out.lines().any(|line| line == "USE 0 -1 math.calc"), "{out}");
    for path in ["win.call", "lnx.call"] {
        assert!(out.lines().any(|line| line.starts_with("USE 0 ")
            && !line.starts_with("USE 0 -1 ")
            && line.ends_with(path)), "missing guarded import {path}");
    }
    assert_eq!(
        out.lines().filter(|line| line.contains(" in-arm ")).count(),
        2,
        "both target arms retain their own AST node",
    );
}

#[test]
fn aac1046_module_catalog_missing_duplicate_and_import_cycle() {
    use ace_aac_syntax_conformance::run_native_catalog;

    let first = ("alpha.at", "use beta.y\nfn x() int { return y() }");
    let second = ("beta.at", "use alpha.x\nfn y() int { return x() }");
    assert_eq!(
        run_native_catalog("packages-ct", "app", &[first], &["alpha.at"]).unwrap(),
        "MODULE_ERR MISSING beta",
    );
    let single = ("alpha.at", "fn x() int { return 1 }");
    assert_eq!(
        run_native_catalog("packages-ct", "app", &[single], &["alpha.at", "alpha.at"]).unwrap(),
        "MODULE_ERR DUPLICATE alpha.at",
    );
    assert_eq!(
        run_native_catalog("packages-ct", "app", &[first, second], &["alpha.at", "beta.at"]).unwrap(),
        "MODULE_ERR IMPORT_CYCLE",
    );
    let no_import = ("beta.at", "pub fn y() int { return 1 }");
    let accepted = run_native_catalog("packages-ct", "app", &[first, no_import], &["alpha.at", "beta.at"]).unwrap();
    assert!(accepted.starts_with("CATALOG\n"), "{accepted}");
}

#[test]
fn aac1046_catalog_distinguishes_body_and_initializer_edges() {
    use ace_aac_syntax_conformance::run_native_catalog;
    use std::collections::BTreeMap;

    let functions = "fn first() int { return second() }\nfn second() int { return first() }";
    let values = "var alpha int = beta\nvar beta int = alpha";
    for input in [functions, values] {
        assert!(rust_oracle("packages-ct", input).ends_with("DIAGS\n"));
    }
    let files = [("functions.at", functions), ("storage.at", values)];
    let catalog = run_native_catalog("packages-ct", "app", &files, &["functions.at", "storage.at"]).unwrap();
    assert!(catalog.starts_with("CATALOG\n"), "{catalog}");

    let mut declarations = BTreeMap::new();
    let mut observed = Vec::new();
    for line in catalog.lines() {
        if !line.starts_with("DECL ") && !line.starts_with("CANDIDATE ") {
            continue;
        }
        let fields: Vec<_> = line.split_whitespace().collect();
        let [tag, id, owner, category, name, span] = fields.as_slice() else { panic!("{line}") };
        let source = files[id.parse::<usize>().unwrap()].1;
        let (start, end) = span.split_once("..").unwrap();
        let start = start.parse::<usize>().unwrap();
        let end = end.parse::<usize>().unwrap();
        assert_eq!(&source[start..end], *name);
        if *tag == "DECL" {
            assert!(declarations.insert((*id, *owner), (*category, *name)).is_none());
        } else {
            let (kind, from) = declarations[&(*id, *owner)];
            assert_eq!(kind, *category);
            observed.push((*id, kind, from, *name));
        }
    }
    assert_eq!(observed, [
        ("0", "function", "first", "second"),
        ("0", "function", "second", "first"),
        ("1", "value", "alpha", "beta"),
        ("1", "value", "beta", "alpha"),
    ]);
}

#[test]
fn aac1046_aac_combined_modules_preserve_source_owners() {
    use ace_aac_syntax_conformance::run_native_catalog;

    let entry = concat!(
        "#[api(method = \"GET\", path = \"/api/notes\")]\n",
        "pub fn list_notes() int { return 1 }\n",
        "fn lift<T>(a Option<T>, b Result<T,int>, c Future<T>) ?T { return a }\n",
        "#in sys is current {\n Windows -> use views.render\n Linux -> use views.render\n}\n",
    );
    let views = "node Hello { view { text \"Hello\" {} } }\npub fn render() int { return 1 }";
    let inventory = ["entry.at", "views.at"];
    let files = [(inventory[0], entry), (inventory[1], views)];
    let expected = files
        .iter()
        .enumerate()
        .map(|(id, (_, source))| rust_ast_report("aac", source, id as i32, 0))
        .collect::<Vec<_>>();
    let catalog = run_native_catalog("aac", "app", &files, &inventory).unwrap();
    assert!(catalog.starts_with("CATALOG\n"), "{catalog}");
    let mut modules = catalog.split("MODULE ").skip(1);
    for (id, (name, _)) in files.iter().enumerate() {
        let module = modules.next().expect("catalog includes both sources");
        assert!(module.lines().next().unwrap().ends_with(name));
        let (_, ast) = module.split_once("AST\n").unwrap();
        assert_eq!(format!("AST\n{ast}DIAGS\n"), expected[id]);
    }
    assert!(modules.next().is_none());
    assert_eq!(
        catalog
            .lines()
            .filter(|line| line.starts_with("USE 0 ") && line.ends_with(" views.render"))
            .count(),
        2
    );

    let ast = catalog
        .split_once("AST\n")
        .unwrap()
        .1
        .split_once("MODULE ")
        .unwrap()
        .0;
    let row = |role: &str, kind: &str, spelling: &str| {
        let start = entry.find(spelling).unwrap();
        let expected = format!(" {role} {kind} {start}..{} 0 0 0", start + spelling.len());
        ast.lines()
            .find(|line| line.ends_with(&expected))
            .unwrap_or_else(|| panic!("missing {expected}"))
    };
    let attribute = row(
        "attribute",
        "declaration-attribute",
        "#[api(method = \"GET\", path = \"/api/notes\")]",
    );
    let owner = row(
        "owner",
        "attribute-owner",
        "pub fn list_notes() int { return 1 }",
    );
    assert_eq!(
        attribute.split_whitespace().next(),
        owner.split_whitespace().next()
    );
    let option = row("generic", "generic-ty", "Option<T>");
    row("generic", "generic-ty", "Result<T,int>");
    row("generic", "generic-ty", "Future<T>");
    row("optional", "optional-ty", "?T");

    for erased in [option, attribute] {
        let changed = ast.replacen(&format!("{erased}\n"), "", 1);
        assert_ne!(format!("AST\n{changed}DIAGS\n"), expected[0]);
    }
    let foreign_owner = format!("{} 1 0 0", owner.strip_suffix(" 0 0 0").unwrap());
    let relocated = ast.replacen(owner, &foreign_owner, 1);
    assert_ne!(format!("AST\n{relocated}DIAGS\n"), expected[0]);

    for broken in [
        entry.replace(")]", ")"),
        entry.replace("?T", "?"),
        entry.replace("a Option<T>", "#[test] a Option<T>"),
    ] {
        let bad = run_native_catalog(
            "aac",
            "app",
            &[(inventory[0], &broken), files[1]],
            &inventory,
        )
        .unwrap();
        assert_eq!(bad, "MODULE_ERR SYNTAX entry.at");
    }
}

#[test]
fn aac1046_aac_attributes_keep_shapes_and_field_targets() {
    let source = concat!(
        "#[test]\nfn exercises_unit() int { return 0 }\n",
        "#[c,vm,rs]\nfn native_num(x int) int;\n",
        "#[export(system)]\n#[test]\nfn exported() int { return 0 }\n",
        "type Record {\n #[serde(default, skip_serializing_if = \"absent\")] enabled bool\n}\n",
    );
    let out = run_native_projection("aac", source, (13, 6), None).unwrap();
    assert_eq!(out, rust_ast_report("aac", source, 13, 6));
    assert!(out.starts_with("AST\n") && out.ends_with("DIAGS\n"));
    let rows: Vec<_> = out
        .lines()
        .skip(1)
        .take_while(|row| *row != "DIAGS")
        .collect();
    let parent_for = |role: &str, kind: &str, needle: &str| {
        let start = source.find(needle).unwrap();
        let coordinate = format!(" {role} {kind} {start}..{} 13 6 0", start + needle.len());
        rows.iter()
            .find_map(|row| row.strip_suffix(&coordinate))
            .unwrap_or_else(|| panic!("missing {coordinate}"))
            .parse::<usize>()
            .unwrap()
    };
    let field_owner = parent_for(
        "attribute",
        "declaration-attribute",
        "#[serde(default, skip_serializing_if = \"absent\")]",
    );
    assert_eq!(
        rows[field_owner].split_whitespace().nth(2),
        Some("field-member")
    );
    let c_owner = parent_for("attribute", "declaration-attribute", "#[c,vm,rs]");
    assert_eq!(
        rows[c_owner].split_whitespace().nth(2),
        Some("attributed-item")
    );
    parent_for("head", "ident", "export");
    parent_for("value", "expr", "system");
}

#[test]
fn aac1046_aac_preserves_const_arguments_constraints_and_ct_expressions() {
    let source = concat!(
        "spec Comparable { fn rank() int }\n",
        "fn sort<T has Comparable>(a T) T { return a }\n",
        "fn log2<const N>(n int) int { return n }\n",
        "fn main() int { let v = #{ log2<2>(21) }\n return v }\n",
    );
    let actual = run_native_projection("aac", source, (19, 0), None).unwrap();
    assert_eq!(actual, rust_ast_report("aac", source, 19, 0));
    assert!(
        actual.starts_with("AST\n") && actual.ends_with("DIAGS\n"),
        "{actual}"
    );
    for (kind, spelling) in [
        ("spec-item", "spec Comparable { fn rank() int }"),
        ("type-param-constraint", "has Comparable"),
        ("type-param", "const N"),
        ("generic-call", "log2<2>(21)"),
        ("hash-expr", "#{ log2<2>(21) }"),
    ] {
        let start = source.find(spelling).unwrap();
        assert!(
            actual.contains(&format!(
                " {kind} {start}..{} 19 0 0",
                start + spelling.len()
            )),
            "missing authored {kind}"
        );
    }
    let bad = source.replace("#{ log2<2>(21) }", "#{ log2<2>(21)");
    let recovered = obs("parse", "aac", &bad);
    assert!(
        recovered.starts_with("CANON\n") && !recovered.ends_with("DIAGS\n"),
        "{recovered}"
    );
}


fn corpus_source<'a>(document: &'a str, case: &str) -> &'a str {
    let heading = format!("## Case: {case}\n");
    let (_, section) = document.split_once(&heading).expect("shared case heading");
    let (_, source) = section.split_once("```auto\n").expect("shared Auto source block");
    source.split_once("\n```").expect("source block end").0
}

fn preserve_expected_root_trivia(mut canonical: String, input: &str, space: usize) -> String {
    // The Rust reference discards one pending space immediately before the
    // first appended error leaf in these two recovery cases. The independent
    // expected tree keeps that source byte at root level.
    assert_eq!(input.as_bytes()[space..space + 2], *b" {");
    let error = format!(" n1048655^:[{},{}) exp=0 f=[]\n", space + 1, space + 2);
    let trivia = format!(" n1048576~:[{space},{}) exp=0 f=[]\n", space + 1);
    assert_eq!(canonical.matches(&error).count(), 1);
    canonical = canonical.replace(&error, &(trivia + &error));
    let root = canonical["CANON\n".len()..].find(']').unwrap() + "CANON\n".len();
    canonical.insert_str(root, "-1,");
    canonical
}

fn check_shared_corpus(document: &str, name: &str, tail_space: Option<usize>) {
    let source = corpus_source(document, name);
    let close = source.rfind('}').expect("fixture has a final declaration brace");
    let mut differences = Vec::new();
    for (label, input, valid) in [("full", source, true), ("truncated", &source[..close], false)] {
        let mut expected = rust_oracle("auto", input);
        if let (false, Some(space)) = (valid, tail_space) {
            expected = preserve_expected_root_trivia(expected, input, space);
        }
        if expected.ends_with("DIAGS\n") != valid {
            differences.push(format!("{name} {label}: reference has unexpected recovery state"));
        }
        match run_native_case("parse", "auto", input, None) {
            Ok(actual) if actual != expected => {
                let row = actual.lines().zip(expected.lines()).find(|(a, e)| a != e);
                differences.push(format!("{name} {label}: first differing row {row:?}"));
            }
            Err(err) => differences.push(format!("{name} {label}: {err}")),
            Ok(_) => {}
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

macro_rules! shared_corpus_case {
    ($test:ident, $path:literal, $case:literal, $tail_space:expr) => {
        #[test]
        fn $test() {
            check_shared_corpus(
                include_str!(concat!(
                    "../../../../../crates/auto-lang/test/cases/language/",
                    $path
                )),
                $case,
                $tail_space,
            );
        }
    };
}

shared_corpus_case!(aac1046_corpus_basics, "basics/expressions.case.ad", "baseline-value", None);
shared_corpus_case!(aac1046_corpus_calls, "calls/binding-pipeline.case.ad", "pipe-method-and-field", None);
shared_corpus_case!(aac1046_corpus_collections, "collections/list-places.case.ad", "bracket-read-write", None);
shared_corpus_case!(aac1046_corpus_functions, "functions/references-closures.case.ad", "named-function-value", None);
shared_corpus_case!(aac1046_corpus_numbers, "numbers/literals-conversions.case.ad", "wide-exact", None);
shared_corpus_case!(aac1046_corpus_patterns, "patterns/nested-match.case.ad", "nested-result-option", Some(194));
shared_corpus_case!(aac1046_corpus_types, "types/variants.case.ad", "named-enum-payload", Some(198));

#[test]
fn aac1046_corpus_text() {
    use ace_core::source::{fstr::desugar_fstrings, parse_program_with, SourceProfile};

    let source = corpus_source(
        include_str!("../../../../../crates/auto-lang/test/cases/language/text/interpolation-unicode.case.ad"),
        "text-nul-and-format",
    );
    let truncated = &source[..source.rfind('}').unwrap()];
    assert!(parse_program_with(source, SourceProfile::Auto).is_ok());
    assert_eq!(parse_program_with(truncated, SourceProfile::Auto).unwrap_err().code, "source.syntax");

    let bad = obs("parse", "auto", truncated);
    let (tree, diagnostics) = bad.split_once("DIAGS\n").unwrap_or_else(|| panic!("{bad}"));
    assert!(tree.starts_with("CANON\n") && !diagnostics.is_empty());
    assert!(diagnostics.lines().any(|diag| diag == format!("{0}..{0} expected `}}` before end of source", truncated.len())));
    for diagnostic in diagnostics.lines() {
        let (span, _) = diagnostic.split_once(' ').unwrap();
        let (start, end) = span.split_once("..").unwrap();
        assert!(start.parse::<usize>().unwrap() <= end.parse::<usize>().unwrap());
        assert!(end.parse::<usize>().unwrap() <= truncated.len(), "{diagnostic}");
    }

    let normalized = desugar_fstrings(source).unwrap();
    let expected = rust_ast_report("auto", &normalized, 41, 7);
    let actual = run_native_projection("auto", source, (41, 7), None).unwrap();
    assert!(actual.starts_with("AST\n") && actual.ends_with("DIAGS\n"), "{actual}");
    let shape = |report: &str| {
        report.lines().skip(1).take_while(|line| *line != "DIAGS").map(|line| {
            line.split_whitespace().take(3).collect::<Vec<_>>().join(" ")
        }).collect::<Vec<_>>()
    };
    assert_eq!(shape(&actual), shape(&expected));
    let generated: Vec<_> = actual.lines().filter(|line| line.ends_with(" 41 7 8")).collect();
    let starts: Vec<_> = source.match_indices("f\"").map(|(index, _)| index).collect();
    assert_eq!(generated.len(), starts.len());
    for (row, start) in generated.iter().zip(starts) {
        assert!(row.contains(&format!(" ident {start}..{start} 41 7 8")), "{row}");
    }
    let calls: Vec<_> = ["left()", "right()"].into_iter().map(|call| {
        let start = source.rfind(call).unwrap();
        let end = start + call.len();
        actual.lines().position(|row| row.contains(&format!(" call {start}..{end} ")))
            .unwrap_or_else(|| panic!("missing interpolation call {call} at {start}..{end}"))
    }).collect();
    assert!(calls[0] < calls[1]);
}

#[test]
fn aac1046_format_delimiters_report_original_bytes() {
    use ace_core::source::{parse_program_with, SourceProfile};

    let good = "// 😀\r\nfn main() int { let s = f\"${42}\"\r\n return 1 }";
    assert!(parse_program_with(good, SourceProfile::Auto).is_ok());
    assert!(obs("parse", "auto", good).ends_with("DIAGS\n"));
    let delimiter = good.find("}\"").unwrap();
    let no_brace = format!("{}{}", &good[..delimiter], &good[delimiter + 1..]);
    let no_quote = &good[..delimiter + 1];
    for (source, start, end) in [(&no_brace[..], delimiter, delimiter + 1), (no_quote, no_quote.len(), no_quote.len())] {
        assert_eq!(parse_program_with(source, SourceProfile::Auto).unwrap_err().code, "source.fstr-syntax");
        let out = obs("parse", "auto", source);
        assert!(out.starts_with("CANON\n"), "{out}");
        assert!(out.lines().any(|diag| diag == format!("{start}..{end} source.fstr-syntax malformed format string")), "{out}");
    }
    let short = "fn f() int { let s=f\"${1}\" }";
    assert!(short.len() <= 128);
    assert_eq!(obs("parse-source-work", "auto", short), "PARSE_ERR 3");
}

#[test]
fn aac1046_format_modes_keep_only_authored_interpolations() {
    use ace_core::source::{parse_program_with, SourceProfile};

    let source = r#"fn main() int {
 let name = 3
 // f"comment"
 let plain = "f\"ordinary"
 let explicit = __fstr("literal")
 let s = f"$name ${1 + 2} \$ {name}"
 let empty = f""
 return 1
}"#;
    for (slug, profile, expected_refs) in [("auto", SourceProfile::Auto, 1), ("scheduler", SourceProfile::Scheduler, 2)] {
        parse_program_with(source, profile).unwrap_or_else(|diag| panic!("{slug}: {diag:?}"));
        let out = run_native_projection(slug, source, (17, 0), None).unwrap();
        assert!(out.ends_with("DIAGS\n"), "{out}");
        let raw_name_references = source.match_indices("name").skip(1).take(2).map(|(start, _)| {
            format!(" ident {start}..{} 17 0 0", start + 4)
        });
        assert_eq!(raw_name_references.filter(|span| out.contains(span)).count(), expected_refs);
        assert_eq!(out.lines().filter(|line| line.ends_with(" 17 0 8")).count(), 2);
        let explicit = source.find("__fstr").unwrap();
        assert!(out.contains(&format!(" ident {explicit}..{} 17 0 0", explicit + 6)));
    }
    let scalar = "fn main() int { return f\"${1}\" }";
    assert!(parse_program_with(scalar, SourceProfile::Scalar).is_err());
    let actual = obs("parse", "scalar", scalar);
    assert!(actual.starts_with("CANON\n") && !actual.ends_with("DIAGS\n"), "{actual}");
}

#[test]
fn aac1046_format_edits_match_cold_with_reuse() {
    let source = concat!(
        "fn a() int {\r\n // 😀\r\n let word = f\"é${1 + 2}\"\r\n return 1\r\n}\r\n",
        "fn b(x int) int { return x }\r\n",
        "fn c(y int) int { return y + 3 }\r\n",
        "fn d(z int) int { return z }\r\n",
    );
    let expr = source.find("1 + 2").unwrap();
    let prefix = source.find('😀').unwrap();
    let mut edited = source.to_owned();
    edited.replace_range(expr..expr + 5, "20 + 22");
    for (input, at, end, insert) in [
        (source, expr, expr + 5, "20 + 22"),
        (&edited, expr, expr + 7, "1 + 2"),
        (source, prefix, prefix, " "),
    ] {
        let mut expected = input.to_owned();
        expected.replace_range(at..end, insert);
        let cold = obs("parse", "auto", &expected);
        let hot = run_native_case("edit", "auto", input, Some((at as i32, end as i32, insert))).unwrap();
        let (tree, facts) = hot.split_once("SPLICE ").unwrap_or_else(|| panic!("{hot}"));
        assert_eq!(tree.strip_prefix("INCR|"), Some(cold.as_str()));
        assert!(facts.starts_with("ok=1 fb=0 "), "{facts}");
        let grafted = facts.split_once("grafted=").unwrap().1.split_whitespace().next().unwrap().parse::<usize>().unwrap();
        assert!(grafted > 0, "{facts}");
        let start = input[..at].rfind('\n').map_or(0, |index| index + 1);
        let expected_scan = expected[start..].find('\n').unwrap() + 1;
        assert!(expected_scan < expected.len());
        assert!(facts.ends_with(&format!(" normalized={expected_scan}")), "{facts}");
    }

    let cold_ast = run_native_projection("auto", &edited, (55, 9), None).unwrap();
    let hot_ast = run_native_projection("auto", source, (55, 9), Some((expr as i32, (expr + 5) as i32, "20 + 22"))).unwrap();
    let (projected, splice) = hot_ast.split_once("SPLICE ").unwrap_or_else(|| panic!("{hot_ast}"));
    assert_eq!(projected.strip_prefix("INCR|"), Some(cold_ast.as_str()));
    assert!(splice.starts_with("ok=1 fb=0 "), "{splice}");

    let quote = source.find("}\"").unwrap() + 1;
    let mut broken = source.to_owned();
    broken.remove(quote);
    let hot = run_native_case("edit", "auto", source, Some((quote as i32, quote as i32 + 1, ""))).unwrap();
    let (tree, facts) = hot.split_once("SPLICE ").unwrap_or_else(|| panic!("{hot}"));
    assert_eq!(tree.strip_prefix("INCR|"), Some(obs("parse", "auto", &broken).as_str()));
    assert!(facts.starts_with("ok=0 fb=1 ") && facts.ends_with("grafted=0"), "{facts}");
}

#[test]
fn aac1046_recovered_diagnostic_positions_match_raw_snapshot() {
    use ace_syntax_runtime::{position::LineIndex, snapshot::{PageStore, Snapshot}};

    let source = "// first line\r\nfn main() int { let s = f\"😀 ${1}\"\r\n return 1 }";
    let close = source.find("}\"").unwrap();
    let mut malformed = source.to_owned();
    malformed.remove(close);
    assert_eq!(obs("diag-positions", "auto", source), "DIAGPOS\n");
    let cold = obs("diag-positions", "auto", &malformed);
    let hot = run_native_case("edit-positions", "auto", source, Some((close as i32, close as i32 + 1, ""))).unwrap();
    let (positions, facts) = hot.split_once("SPLICE ").unwrap_or_else(|| panic!("{hot}"));
    assert_eq!(positions.strip_prefix("INCR|"), Some(cold.as_str()));
    assert!(facts.starts_with("ok=0 fb=1 "), "{facts}");

    let pages = std::rc::Rc::new(PageStore::new());
    let raw = Snapshot::from_text(&pages, &malformed);
    let index = LineIndex::build(&raw);
    let mut found_format = false;
    for row in cold.lines().skip(1) {
        let fields: Vec<_> = row.split_whitespace().collect();
        let [kind, span, ..] = fields.as_slice() else { panic!("{row}") };
        let (start, end) = span.split_once("..").unwrap();
        let points = [start.parse::<u64>().unwrap(), end.parse::<u64>().unwrap()];
        let expected: Vec<_> = points.into_iter().flat_map(|offset| {
            let (line, utf8) = index.line_col_utf8(&raw, offset);
            let (_, utf16) = index.line_col_utf16(&raw, offset);
            [line, utf8, utf16].map(|value| value.to_string())
        }).collect();
        assert_eq!(fields[2..], expected);
        if *kind == "10" {
            assert_eq!(points, [close as u64, close as u64 + 1]);
            found_format = true;
        }
    }
    assert!(found_format);
    let inside_emoji = source.find('😀').unwrap() as i32 + 1;
    let refused = run_native_case("edit-rollback", "auto", source, Some((inside_emoji, inside_emoji, "x"))).unwrap();
    assert_eq!(refused, "EDIT_REFUSED 4 unchanged");
}

#[test]
fn aac1046_rejected_edit_retains_snapshot_bytes() {
    let sample = "// α 😀\nfn main() int { return 1 }";
    let scalar = sample.find('α').unwrap() as i32;
    for (edit, expected) in [
        ((scalar + 1, scalar + 1, "x"), "EDIT_REFUSED 4 unchanged"),
        ((0, sample.len() as i32 + 1, ""), "EDIT_REFUSED 3 unchanged"),
        ((scalar, scalar, " "), "EDIT_ACCEPTED"),
    ] {
        let actual = run_native_case("edit-rollback", "scalar", sample, Some(edit)).unwrap();
        assert_eq!(actual, expected);
    }
}


#[test]
fn aac1046_positions_use_byte_and_utf16_units() {
    use ace_syntax_runtime::{
        position::LineIndex,
        snapshot::{PageStore, Snapshot},
    };
    let source = "a😀b\r\n第二行 😀\r\n";
    let pages = std::rc::Rc::new(PageStore::new());
    let snapshot = Snapshot::from_text(&pages, source);
    let index = LineIndex::build(&snapshot);
    let first_emoji = source.find('😀').unwrap() as u64;
    let line_two = source.find('第').unwrap() as u64;
    for (offset, column) in [
        (0, 1),
        (first_emoji, 3),
        (first_emoji + 4, 4),
        (line_two - 1, 100),
        (line_two, 2),
        (source.len() as u64, 100),
    ] {
        let (line, col8) = index.line_col_utf8(&snapshot, offset);
        let (_, col16) = index.line_col_utf16(&snapshot, offset);
        let back8 = index.offset_of_utf8(line, column);
        let back16 = index.offset_of_utf16(&snapshot, line, column);
        let actual = run_native_case("positions", "", source, Some((offset as i32, column as i32, ""))).unwrap();
        assert_eq!(actual, format!("{line} {col8} {col16} {back8} {back16}"));
    }
}
