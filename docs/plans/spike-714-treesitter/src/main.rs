// PLAN-714 T-02 spike smoke — three assertions:
//   (1) parse smoke: minimal .rs/.py samples parse to the expected node tree;
//   (2) incremental smoke: an edit inside one function leaves sibling
//       subtrees unmarked (has_changes == false) — the reuse face of
//       tree.edit + parse(old_tree);
//   (3) highlight query smoke: a highlights query produces a non-empty
//       capture stream with expected categories (comment/string/variable).
// Prints versions, node counts, timing (full vs incremental reparse) and the
// release binary size is recorded separately in the survey receipt.

use std::time::Instant;

use tree_sitter::{InputEdit, Language, Parser, Point};
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

const RUST_SRC: &str = r#"// top comment
fn warm(x: i64) -> i64 {
    let s = "txt"; // string + comment
    x + 1
}

fn cold(y: i64) -> i64 {
    y * 2
}
"#;

const PY_SRC: &str = r#"# top comment
def warm(x):
    s = "txt"  # string + comment
    return x + 1

def cold(y):
    return y * 2
"#;

fn rust_lang() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}

fn py_lang() -> Language {
    tree_sitter_python::LANGUAGE.into()
}

fn assert_parse(lang_name: &str, src: &str, lang: Language, expect_fn_kind: &str) {
    let mut parser = Parser::new();
    parser
        .set_language(&lang)
        .expect("set_language ok (runtime/grammar ABI compatible)");
    let tree = parser.parse(src, None).expect("parse produced a tree");
    let root = tree.root_node();
    assert_eq!(root.kind(), if lang_name == "rust" { "source_file" } else { "module" });
    assert!(!root.has_error(), "root must not be ERROR");
    let mut cursor = root.walk();
    let mut fn_seen = false;
    'walk: loop {
        if cursor.node().kind() == expect_fn_kind {
            fn_seen = true;
            break;
        }
        if !cursor.goto_first_child() {
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                if !cursor.goto_parent() {
                    break 'walk;
                }
            }
        }
    }
    assert!(fn_seen, "{lang_name}: expected a {expect_fn_kind} node");
    println!(
        "[parse:{lang_name}] root={} nodes={} bytes={}",
        root.kind(),
        root.descendant_count(),
        src.len()
    );
}

/// (2) incremental: edit inside `warm` → (a) tree.edit marks warm's subtree
/// (has_changes) on the edited old tree while cold stays unmarked; (b)
/// re-parse with the edited tree reuses untouched subtrees; (c)
/// `Tree::changed_ranges` against the original yields exactly the inserted
/// range — the invalidation-domain API face for the highlight pipeline.
fn incremental_smoke() {
    let mut parser = Parser::new();
    parser.set_language(&rust_lang()).unwrap();

    let full_parse = Instant::now();
    let src = RUST_SRC.to_string();
    let tree = parser.parse(&src, None).unwrap();
    let full_elapsed = full_parse.elapsed();

    // Edit: insert a statement inside `warm`'s body (before `let s = ...`).
    let insert_at = src.find("    let s").expect("warm body anchor");
    let inserted = "    let t = \"more\";\n";
    let mut new_src = String::new();
    new_src.push_str(&src[..insert_at]);
    new_src.push_str(inserted);
    new_src.push_str(&src[insert_at..]);

    let new_end_byte = insert_at + inserted.len();
    let mut edited_tree = tree.clone();
    edited_tree.edit(&InputEdit {
        start_byte: insert_at,
        old_end_byte: insert_at,
        new_end_byte,
        start_position: point_at(&src, insert_at),
        old_end_position: point_at(&src, insert_at),
        new_end_position: point_at(&new_src, new_end_byte),
    });

    // (a) edit marking lives on the edited tree: warm marked, cold untouched.
    // (tree.edit shifts node ranges past the edit point — resolve names
    // against the post-edit text.)
    let (warm_marked, cold_marked) = fn_changes(&edited_tree, &new_src);
    assert_eq!(warm_marked, Some(true), "tree.edit must mark warm's subtree");
    assert_eq!(cold_marked, Some(false), "cold's subtree must stay unmarked");

    // (b) incremental re-parse against the edited tree.
    let inc_parse = Instant::now();
    let new_tree = parser.parse(&new_src, Some(&edited_tree)).unwrap();
    let inc_elapsed = inc_parse.elapsed();

    // (c) invalidation domain: changed_ranges diffs at leaf-token level —
    // exactly one range, contained within the byte-level edit range (a pure
    // insertion of `    let t = "more";` reports the token diff, e.g. leading
    // spaces shared with the next line are excluded).
    let ranges: Vec<_> = edited_tree.changed_ranges(&new_tree).collect();
    assert_eq!(ranges.len(), 1, "exactly one changed range, got {ranges:?}");
    assert!(
        ranges[0].start_byte >= insert_at && ranges[0].end_byte <= new_end_byte,
        "changed range must fall inside the edit range: {:?} vs [{insert_at}..{new_end_byte}]",
        ranges[0]
    );

    println!(
        "[incremental] full_reparse={full_elapsed:?} inc_reparse={inc_elapsed:?} \
         edit_marked=(warm={warm_marked:?},cold={cold_marked:?}) \
         changed_ranges=[{}..{})",
        ranges[0].start_byte, ranges[0].end_byte
    );
}

/// has_changes per top-level function, by name, on the given tree.
fn fn_changes(tree: &tree_sitter::Tree, src: &str) -> (Option<bool>, Option<bool>) {
    let root = tree.root_node();
    let mut warm = None;
    let mut cold = None;
    let mut cursor = root.walk();
    'walk: loop {
        let node = cursor.node();
        if node.kind() == "function_item" {
            let name = node
                .child_by_field_name("name")
                .map(|n| n.utf8_text(src.as_bytes()).unwrap_or("").to_string());
            match name.as_deref() {
                Some("warm") => warm = Some(node.has_changes()),
                Some("cold") => cold = Some(node.has_changes()),
                _ => {}
            }
        }
        if !cursor.goto_first_child() {
            loop {
                if cursor.goto_next_sibling() {
                    break;
                }
                if !cursor.goto_parent() {
                    break 'walk;
                }
            }
        }
    }
    (warm, cold)
}

fn point_at(src: &str, byte: usize) -> Point {
    let mut row = 0usize;
    let mut col = 0usize;
    for b in &src.as_bytes()[..byte] {
        if *b == b'\n' {
            row += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    Point::new(row, col)
}

/// (3) highlight query smoke — two configs per language:
///   (a) the grammar crate's own bundled highlights.scm (real upstream query
///       through the Highlighter pipeline — non-empty, multi-category);
///   (b) a minimal inline query with correct node names, whose capture
///       indices map to the configured names — exact category assertions.
fn highlight_smoke() {
    // Do the grammar crates bundle highlights queries? (recorded for T-02)
    for (name, crate_name) in [("rust", "tree-sitter-rust"), ("python", "tree-sitter-python")] {
        let dir = format!("{}/queries", crate_source_dir(crate_name));
        match std::fs::read_dir(&dir) {
            Ok(entries) => {
                let files: Vec<_> = entries.filter_map(|e| e.ok().map(|e| e.file_name())).collect();
                println!("[queries:{name}] bundled at {dir}: {files:?}");
            }
            Err(e) => println!("[queries:{name}] NOT BUNDLED in crate package ({dir}: {e})"),
        }
    }

    const NAMES: &[&str] = &["comment", "string", "variable"];
    let mut cases = Vec::new();
    for (case, lang, crate_name, inline_q) in [
        (
            "rust",
            rust_lang(),
            "tree-sitter-rust",
            "(line_comment) @comment
(string_literal) @string
(identifier) @variable
",
        ),
        (
            "python",
            py_lang(),
            "tree-sitter-python",
            "(comment) @comment
(string) @string
(identifier) @variable
",
        ),
    ] {
        let qpath = format!("{}/queries/highlights.scm", crate_source_dir(crate_name));
        let (bundled_origin, bundled_query) = match std::fs::read_to_string(&qpath) {
            Ok(q) => (format!("bundled ({qpath})"), Some(q)),
            Err(e) => {
                println!("[highlight:{case}] bundled query unreadable ({e})");
                ("bundled MISSING".to_string(), None)
            }
        };
        let real = bundled_query.map(|q| {
            let mut c = HighlightConfiguration::new(lang.clone(), case, &q, "", "")
                .unwrap_or_else(|e| panic!("{case}: bundled highlights.scm failed: {e}"));
            c.configure(&[
                "attribute", "boolean", "comment", "constant", "constant.builtin", "constructor",
                "escape", "function", "function.builtin", "function.macro", "include", "keyword",
                "label", "module", "number", "operator", "property", "punctuation",
                "punctuation.bracket", "punctuation.delimiter", "punctuation.special", "string",
                "string.escape", "string.special", "tag", "type", "type.builtin", "variable",
                "variable.builtin", "variable.member", "variable.parameter", "variable.super",
            ]);
            c
        });
        let mut minimal = HighlightConfiguration::new(lang, case, inline_q, "", "")
            .unwrap_or_else(|e| panic!("{case}: inline config: {e}"));
        minimal.configure(NAMES);
        cases.push((case.to_string(), real, minimal, bundled_origin));
    }

    let mut highlighter = Highlighter::new();
    for (i, (src_case, src)) in [("rust", RUST_SRC), ("python", PY_SRC)].into_iter().enumerate() {
        let (case, real, minimal, origin) = &cases[i];
        let _ = src_case;
        // (a) real bundled query — pipeline evidence.
        if let Some(real) = real {
            let events: Vec<_> = highlighter
                .highlight(real, src.as_bytes(), None, None, |_| None)
                .expect("highlight runs")
                .map(|e| e.expect("highlight event ok"))
                .collect();
            let distinct: std::collections::BTreeSet<usize> = events
                .iter()
                .filter_map(|e| match e {
                    HighlightEvent::HighlightStart(h) => Some(h.0),
                    _ => None,
                })
                .collect();
            assert!(!distinct.is_empty(), "{case}: bundled query produced no captures");
            assert!(distinct.len() >= 3, "{case}: expected multi-category captures, got {distinct:?}");
            println!("[highlight:{case}] origin={origin} events={} capture_kinds={}", events.len(), distinct.len());
        }
        // (b) minimal query — named category evidence.
        let events: Vec<_> = highlighter
            .highlight(minimal, src.as_bytes(), None, None, |_| None)
            .expect("highlight runs")
            .map(|e| e.expect("highlight event ok"))
            .collect();
        let mut seen: Vec<&str> = Vec::new();
        for ev in &events {
            if let HighlightEvent::HighlightStart(h) = ev {
                seen.push(NAMES.get(h.0 as usize).copied().unwrap_or("?"));
            }
        }
        for want in NAMES {
            assert!(
                seen.contains(want),
                "{case}: expected {want} capture, got {seen:?}"
            );
        }
        println!("[highlight:{case}] minimal-query captures={seen:?}");
    }
}

/// Best-effort registry source dir lookup for the queries-bundling probe.
fn crate_source_dir(crate_name: &str) -> String {
    let home = std::env::var("CARGO_HOME").unwrap_or_else(|_| {
        format!(
            "{}/.cargo",
            std::env::var("USERPROFILE").unwrap_or_default().replace('\\', "/")
        )
    });
    let reg = format!("{home}/registry/src");
    let mut best_name = String::new();
    let mut best_path = String::new();
    if let Ok(roots) = std::fs::read_dir(&reg) {
        for r in roots.filter_map(|e| e.ok()) {
            let root = r.path();
            if let Ok(dirs) = std::fs::read_dir(&root) {
                for d in dirs.filter_map(|e| e.ok()) {
                    let name = d.file_name().to_string_lossy().to_string();
                    if name.starts_with(crate_name) && name.as_str() > best_name.as_str() {
                        best_name = name.clone();
                        best_path = format!("{}/{name}", root.display());
                    }
                }
            }
        }
    }
    best_path
}

fn main() {
    println!("tree-sitter runtime: LANGUAGE_VERSION v{}, MIN grammar ABI v{}",
        tree_sitter::LANGUAGE_VERSION, tree_sitter::MIN_COMPATIBLE_LANGUAGE_VERSION);
    println!("grammar ABI: rust v{}, python v{}",
        rust_lang().abi_version(), py_lang().abi_version());

    assert_parse("rust", RUST_SRC, rust_lang(), "function_item");
    assert_parse("python", PY_SRC, py_lang(), "function_definition");
    incremental_smoke();
    highlight_smoke();
    println!("SMOKE-OK");
}
