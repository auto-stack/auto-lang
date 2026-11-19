//! PLAN-1056 T-04 (F1 option A): canonical Atom DATA reader/writer for the AAC
//! data slice — the pure-Auto `app.atom_data` module compiled by the Gen0
//! (Rust-frontend) assembled `packages-aggregates-ct` lane and executed on the
//! AV executor.
//!
//! Two acceptance modes share one writer:
//!
//!   * `canonicalize(text)` — SCHEMA-FREE: generic Atom data, no tag/field
//!     whitelist; the only content check is the duplicate-field rule (8);
//!   * `canonicalize_with_schema(schema_text, doc_text)` — the REAL descriptor
//!     schema (`experimental/ace/crates/core/schema/core-i32.atom`) is parsed
//!     into a tag/field table and drives validation: 10 for a non-`hir` root or
//!     an undeclared node tag, 9 for an undeclared field, 8 for a duplicate.
//!
//! Every expected byte in this file is written from the frozen writer rules
//! (indent, `", "`-separated group, body layout, final newline), never copied
//! from Auto output: the Auto side either re-emits a document that is already
//! in canonical form (round-trip identity) or reduces an equivalent variant
//! onto that same form. The REAL HIR fixture is exercised for ACCEPTANCE and
//! IDEMPOTENCE only — an exact per-construct reproduction of the reference
//! writer's bytes is an explicit named handoff to 1047/1053.
//!
//! The codec is deliberately a DATA reader: `atom_semantic_odd_data_*` pins
//! that a document whose `kind` names no known id, whose ids repeat, or whose
//! nesting is semantically strange is ACCEPTED, because the reader never
//! claims to build a verified object — the canonical bytes are an untrusted
//! DTO and semantic verification belongs to a later stage.

use ace_aac_support_conformance::{run_program, str_bytes};

/// Auto string literal for exact document text (the four Atom escapes plus
/// the backslash itself; non-ASCII scalars stay literal).
fn auto_lit(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn run_text(tag: &str, src: &str) -> String {
    let v = run_program(tag, src).unwrap_or_else(|e| panic!("{tag}: {e}"));
    String::from_utf8(str_bytes(&v).unwrap_or_else(|e| panic!("{tag}: {e}"))).unwrap()
}

fn hex_of(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// One `canonical_text` driver program for a document (schema-free mode).
fn text_program(doc: &str) -> String {
    format!(
        "use app.atom_data.canonical_text\nfn main() Str {{ return canonical_text({}) }}",
        auto_lit(doc)
    )
}

/// The REAL descriptor schema and the REAL canonical HIR text (embedded with
/// `include_str!`, so the case is pinned to the repository files; line endings
/// are normalised because the lexer treats `\r` as whitespace and the writer
/// emits `\n`).
const REAL_SCHEMA_SRC: &str = include_str!("../../../crates/core/schema/core-i32.atom");
const REAL_HIR_SRC: &str = include_str!("../../../crates/core/fixtures/native/add-2-3.atom");

fn real_schema() -> String {
    REAL_SCHEMA_SRC.replace("\r\n", "\n")
}

fn real_hir() -> String {
    REAL_HIR_SRC.replace("\r\n", "\n")
}

// ---------------------------------------------------------------------------
// The frozen canonical document (written to the writer rules: one body item
// per line at depth+1, 4 spaces per depth, `(k: v, k: v)` groups, `{}` for an
// empty body, the root's closing brace alone at depth 0, final newline).
// ---------------------------------------------------------------------------

const FROZEN: &str = r#"descriptor(schema: "aac.atom-data/v1", revision: 1, profile: "core") {
    module(id: "m.foundation") {
        decl(id: "d.echo", kind: "fn") {
            type(id: "t.unit", kind: "unit") {}
        }
        type(id: "t.text", kind: "record") {}
    }
    module(id: "m.surface") {
        decl(id: "d.run", kind: "fn") {}
    }
}
"#;

/// A canonical document whose string values exercise the four escapes and
/// multi-byte scalars: the lexer is scalar-indexed and a legal escape
/// re-encodes to itself, so this is a fixpoint too.
const ESCAPES: &str = r#"descriptor(schema: "aac.atom-data/v1", revision: 1, profile: "q\"x\" b\\ t\t n\n 中😀") {
    module(id: "m.中😀") {}
}
"#;

/// The same document with every legal variation of layout: line breaks,
/// tabs, blank lines, `,` instead of `;`, and comments.
fn equivalent_variants() -> Vec<String> {
    let v1 = concat!(
        "descriptor(schema: \"aac.atom-data/v1\"; revision: 1; profile: \"core\") { ",
        "module(id: \"m.foundation\") { ",
        "decl(id: \"d.echo\"; kind: \"fn\") { type(id: \"t.unit\"; kind: \"unit\") {} } ",
        "type(id: \"t.text\"; kind: \"record\") {} } ",
        "module(id: \"m.surface\") { decl(id: \"d.run\"; kind: \"fn\") {} } }"
    )
    .to_string();
    let v2 = concat!(
        "descriptor(schema: \"aac.atom-data/v1\", revision: 1, profile: \"core\")\n",
        "{\n",
        "    module(id: \"m.foundation\")\n",
        "    {\n",
        "        decl(id: \"d.echo\", kind: \"fn\")\n",
        "        {\n",
        "            type(id: \"t.unit\", kind: \"unit\") { }\n",
        "        }\n",
        "        type(id: \"t.text\", kind: \"record\") { }\n",
        "    }\n",
        "    module(id: \"m.surface\") { decl(id: \"d.run\", kind: \"fn\") { } }\n",
        "}\n"
    )
    .to_string();
    let v3 = concat!(
        "// the AAC data slice fixture\n",
        "descriptor(schema: \"aac.atom-data/v1\"; revision: 1; profile: \"core\") { // root\n",
        "    module(id: \"m.foundation\") {\n",
        "        // a decl with one nested type\n",
        "        decl(id: \"d.echo\"; kind: \"fn\") {\n",
        "            type(id: \"t.unit\"; kind: \"unit\") {}\n",
        "        }\n",
        "        type(id: \"t.text\"; kind: \"record\") {} // sibling type\n",
        "    }\n",
        "    module(id: \"m.surface\") {\n",
        "        decl(id: \"d.run\"; kind: \"fn\") {}\n",
        "    }\n",
        "}\n",
        "// trailing comment only\n"
    )
    .to_string();
    let v4 = concat!(
        "\t\t\n\n",
        "descriptor(schema: \"aac.atom-data/v1\";\trevision: 1;\tprofile: \"core\")\t{\n",
        "\n\n",
        "\t\tmodule(id: \"m.foundation\")\t{\n\n",
        "\t\t\tdecl(id: \"d.echo\";\tkind: \"fn\")\t{\n",
        "\t\t\t\ttype(id: \"t.unit\"; kind: \"unit\") {}\n\n",
        "\t\t\t}\n",
        "\t\t\ttype(id: \"t.text\"; kind: \"record\") {}\n",
        "\t\t}\n",
        "\t\tmodule(id: \"m.surface\") { decl(id: \"d.run\"; kind: \"fn\") {} }\n",
        "}\n\n\n"
    )
    .to_string();
    vec![v1, v2, v3, v4]
}


#[test]
fn atom_canonical_roundtrip_is_identity() {
    // ONE Gen0 compile for both fixtures (per-case cost discipline): the
    // program canonicalizes each canonical document and joins the results with
    // a sentinel the documents cannot contain.
    const SEP: &str = "|@|";
    let src = format!(
        "use app.atom_data.canonical_text\nfn main() Str {{\n    return canonical_text({}).clone() + \"{SEP}\" + canonical_text({}).clone()\n}}",
        auto_lit(FROZEN),
        auto_lit(ESCAPES)
    );
    let got = run_text("atom-roundtrip", &src);
    let parts: Vec<&str> = got.split(SEP).collect();
    assert_eq!(parts.len(), 2, "one canonical text per fixture");
    assert_eq!(parts[0], FROZEN, "canonical document must re-emit byte-identically");
    assert_eq!(parts[1], ESCAPES, "escaped/UTF-8 canonical document must be a fixpoint");
}

/// The equivalent variants all reduce onto the frozen canonical bytes.
#[test]
fn atom_equivalent_variants_canonicalize_identically() {
    let variants = equivalent_variants();
    let mut body = String::from("    var out = \"\"\n");
    for v in &variants {
        body.push_str(&format!("    out = out + canonical_text({})\n", auto_lit(v)));
    }
    let src = format!("use app.atom_data.canonical_text\nfn main() Str {{\n{body}    return out\n}}");
    let got = run_text("atom-variants", &src);
    let want = FROZEN.repeat(variants.len());
    assert_eq!(got, want, "every variant must canonicalize to the frozen form");
}

// ---------------------------------------------------------------------------
// V-04b: semantic-odd but syntactic data is accepted (untrusted DTO)
// ---------------------------------------------------------------------------

/// `kind` naming an unknown id, a repeated `id`, a `type` under a `decl`, and
/// a `decl` with no body at all are all DATA to this reader. In the
/// schema-free mode only the grammar and the duplicate-field rule are
/// enforced: nothing here claims a verified AAC object was constructed.
const ODD: &str = r#"descriptor(schema: "aac.atom-data/v1", revision: 2, profile: "core") {
    module(id: "m.core") {
        decl(id: "d.ghost", kind: "kind.that.does.not.exist") {
            type(id: "t.void", kind: "no.such.kind") {}
        }
        decl(id: "d.dup", kind: "fn") {}
        decl(id: "d.dup", kind: "fn") {}
    }
    module {}
}
"#;

#[test]
fn atom_semantic_odd_data_is_accepted_as_untrusted_dto() {
    let got = run_text("atom-odd", &text_program(ODD));
    assert_eq!(
        got, ODD,
        "a semantically odd but syntactic document must be accepted unchanged \
         (empty output would mean a rejection)"
    );
}

// ---------------------------------------------------------------------------
// V-04c: rejection taxonomy, one isolated case per code
// ---------------------------------------------------------------------------

#[test]
fn atom_rejection_taxonomy() {
    // (name, document, expected outcome byte). Codes 9/10 belong to the
    // real-schema mode only (`atom_real_schema_drives_validation`), because the
    // schema-free mode has no tag/field whitelist.
    let cases: Vec<(&str, String, u8)> = vec![
        ("empty", String::new(), 1),
        ("only whitespace/comments", "  \n\t// nothing\n".to_string(), 1),
        ("unexpected char", "descriptor(schema: \"x\") @ {}".to_string(), 2),
        ("unterminated string", "descriptor(schema: \"abc) {}".to_string(), 3),
        ("bad escape", r#"descriptor(schema: "a\q") {}"#.to_string(), 4),
        ("invalid number", "descriptor(revision: 1u3x) {}".to_string(), 5),
        ("dash without digits", "descriptor(revision: -x) {}".to_string(), 5),
        ("unbalanced body", "descriptor(schema: \"x\") {".to_string(), 6),
        ("missing group close", "descriptor(schema: \"x\" { }".to_string(), 6),
        (
            "trailing tokens",
            "descriptor() {}\nmodule(id: \"m\") {}".to_string(),
            7,
        ),
        (
            "duplicate field",
            "descriptor(schema: \"a\", schema: \"b\") {}".to_string(),
            8,
        ),
        (
            "duplicate across group and body",
            "descriptor() { module(id: \"a\") { id: \"b\" } }".to_string(),
            8,
        ),
        (
            "duplicate in an inline node",
            "descriptor(x: { k: 1, k: 2 }) {}".to_string(),
            8,
        ),
    ];
    let mut body = String::from("    var cs = List<int>.new()\n");
    for (_, doc, _) in &cases {
        body.push_str(&format!(
            "    cs.push(status_of(canonicalize({})))\n",
            auto_lit(doc)
        ));
    }
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.atom_data.status_of\n\
         use app.digest.hex_bytes\nfn main() Str {{\n{body}    return hex_bytes(cs)\n}}"
    );
    let got = run_text("atom-taxonomy", &src);
    assert_eq!(
        got.len(),
        2 * cases.len(),
        "one outcome byte per case: got {got} for {} cases",
        cases.len()
    );
    for (i, (name, _, want)) in cases.iter().enumerate() {
        let seg = &got[2 * i..2 * i + 2];
        assert_eq!(
            seg,
            format!("{want:02x}"),
            "case `{name}`: expected outcome {want}"
        );
    }
}

// ---------------------------------------------------------------------------
// V-04d: the byte-list public shape (`[0, ...canonical bytes]`, payload_of)
// ---------------------------------------------------------------------------

#[test]
fn atom_byte_list_shape_is_outcome_first() {
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.atom_data.payload_of\nuse app.digest.hex_bytes\n\
         fn main() Str {{ return hex_bytes(payload_of(canonicalize({}))) }}",
        auto_lit(FROZEN)
    );
    let got = run_text("atom-bytes", &src);
    assert_eq!(
        got,
        hex_of(FROZEN.as_bytes()),
        "payload_of must drop exactly the outcome byte and keep the canonical UTF-8 bytes"
    );
}
/// The length admission rule and its reachable legal neighbour (V-04 boundary).
///
/// The rule is `text.len() > 262144 -> 6` and it is checked BEFORE any lexing
/// work, so 262145 scalars is refused with 6 — the refusal is a pure arithmetic
/// admission, never an allocation of the announced size — while a 2048-scalar
/// document is canonicalized (0).
///
/// The 262144-scalar NEIGHBOUR cannot be exercised on this executor, and that
/// limit is measured rather than skipped (see `atom_admission_envelope` and
/// SD-01 §8 "准入规则与本车道的可执行包络"):
///
///   * every entry point converts the text into ONE `List<int>` of its UTF-8
///     bytes (`text_bytes`), and the frozen collections contract caps a single
///     list at 65536 elements (`av-core::contract::MAX_LIST_ELEMS`), so the
///     65537-scalar document is refused by the EXECUTOR with
///     `av.resource.list-elems` — the codec's 262144 bound is wider than this
///     lane's materializable envelope;
///   * even below that cap the chunked conversion is O(n^2/512) (each 512-scalar
///     chunk re-resolves the whole source buffer) and 65536 scalars also
///     exhausts the slot budget, so the admitted neighbours this slice can
///     actually canonicalize inside the per-case budget are a couple of KiB.
#[test]
fn atom_admission_rules() {
    let statuses = "use app.atom_data.canonicalize\nuse app.codec.status_of\n\
                    use app.digest.hex_bytes\nfn main() Str {\n    var xs = List<int>.new()\n\
                    \x20   let base = \"descriptor()\"\n    let pad = dup(\" \", 11)\n\
                    \x20   let fit = base + pad.slice(0, 2036).clone()\n\
                    \x20   xs.push(status_of(canonicalize(fit)))\n\
                    \x20   let big = dup(\" \", 18) + \" \"\n\
                    \x20   xs.push(status_of(canonicalize(big)))\n\
                    \x20   return hex_bytes(xs)\n}\n\
                    fn dup(s Str, k int) Str {\n    if (k == 0) { return s.clone() }\n\
                    \x20   let doubled = s + s.clone()\n    return dup(doubled, k - 1)\n}";
    assert_eq!(
        run_text("atom-admission-len", statuses),
        "0006",
        "2048 scalars accepted = 0, 262145 scalars refused by the admission rule = 6"
    );
}

/// The EXECUTOR envelope above the codec's admission rule: a 65537-scalar
/// document passes the codec's 262144-scalar check and then hits the frozen
/// single-list cap while its UTF-8 byte vector is materialized
/// (`av-core::contract::MAX_LIST_ELEMS` = 65536, `av.resource.list-elems`).
/// Its own case so each compile stays inside the per-case wall budget.
#[test]
fn atom_admission_envelope() {
    let envelope = "use app.atom_data.canonicalize\nfn main() List<int> {\n\
                    \x20   let base = \"descriptor()\"\n    let pad = dup(\" \", 16)\n\
                    \x20   return canonicalize(base + pad.slice(0, 65525).clone())\n}\n\
                    fn dup(s Str, k int) Str {\n    if (k == 0) { return s.clone() }\n\
                    \x20   let doubled = s + s.clone()\n    return dup(doubled, k - 1)\n}";
    let err = run_program("atom-admission-envelope", envelope)
        .expect_err("a 65537-scalar document exceeds the executor's single-list cap");
    assert!(
        err.contains("ListElems"),
        "the 65537-scalar refusal must be the frozen list-elems resource, got: {err}"
    );
}

/// The parser-depth admission, measured at its REAL boundary rather than
/// asserted from the comment. Depth is counted over ENCLOSING nodes (the root is
/// depth 0) and a field VALUE sits one level below its owner, so the two path
/// lengths differ by exactly the leaf's field:
///
///   * 63 nested nodes + leaf `id` -> 0   (that value is parse_value(depth 64))
///   * 64 nested nodes + leaf `id` -> 6   (that value is parse_value(depth 65):
///                                          the NODE count is still legal, the
///                                          VALUE path is what refuses it)
///   * 64 nested nodes, no leaf field -> 0 (64 nodes is the admitted node depth)
///   * 65 nested nodes, no leaf field -> 6 (65 nodes is refused by parse_node)
///
/// So "depth 64" is the admitted NODE nesting and the effective document limit
/// for a chain whose leaf carries a field is 63. The canonical output of the
/// admitted cases is ~8 KiB, so nothing here is skipped for cost.
#[test]
fn atom_admission_depth() {
    fn chain(n: usize, leaf_field: bool) -> String {
        let mut doc = String::from("descriptor(schema: \"s\", revision: 1, profile: \"p\") {\n");
        for i in 0..n {
            if i + 1 == n {
                if leaf_field {
                    doc.push_str("module(id: \"x\") {}\n");
                } else {
                    doc.push_str("module {}\n");
                }
            } else {
                doc.push_str("module(id: \"x\") {\n");
            }
        }
        for _ in 0..n {
            doc.push_str("}\n");
        }
        doc
    }
    let cases: Vec<(&str, String, &str)> = vec![
        ("63 nodes, leaf field", chain(63, true), "00"),
        ("64 nodes, leaf field", chain(64, true), "06"),
        ("64 nodes, bodiless leaf", chain(64, false), "00"),
        ("65 nodes, bodiless leaf", chain(65, false), "06"),
    ];
    let mut body = String::from("    var xs = List<int>.new()\n");
    for (_, doc, _) in &cases {
        body.push_str(&format!(
            "    xs.push(status_of(canonicalize({})))\n",
            auto_lit(doc)
        ));
    }
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.codec.status_of\n\
         use app.digest.hex_bytes\nfn main() Str {{\n{body}    return hex_bytes(xs)\n}}"
    );
    let got = run_text("atom-depth-boundary", &src);
    assert_eq!(got.len(), 2 * cases.len(), "one outcome byte per case");
    for (i, (name, _, want)) in cases.iter().enumerate() {
        assert_eq!(&got[2 * i..2 * i + 2], *want, "depth case `{name}`");
    }
}

// ---------------------------------------------------------------------------
// Real schema + real HIR fixture (F1 option A contract)
// ---------------------------------------------------------------------------

/// Run a program whose `main` returns `List<int>` and materialize its elements
/// as bytes. The canonical output IS UTF-8 text, so this is how the cases below
/// compare bytes without spending an Auto-side `decode_utf8` (whose chunk lists
/// dominate the AV live-slot budget).
fn run_bytes(tag: &str, src: &str) -> Vec<u8> {
    let v = run_program(tag, src).unwrap_or_else(|e| panic!("{tag}: {e}"));
    match v {
        ace_av_core::contract::Value::List(h) => h
            .rc
            .borrow()
            .elems
            .iter()
            .map(|e| match e {
                ace_av_core::contract::Value::I32(n) => *n as u8,
                other => panic!("{tag}: non-i32 list element {other:?}"),
            })
            .collect(),
        other => panic!("{tag}: expected a List<int> result, got {other:?}"),
    }
}

/// The REAL canonical HIR text (`crates/core/fixtures/native/add-2-3.atom`) is
/// ACCEPTED by both modes: the schema-free structural mode (root `hir`, tags
/// `module`/`types`/`expr`/`let`, fields `path`/`op`/`eval_args`, inline
/// `{ param, arg }` bindings — none of which the old frozen table admitted) and
/// the real-schema mode driven by `crates/core/schema/core-i32.atom`.
#[test]
fn atom_real_hir_fixture_accepted_by_both_modes() {
    let doc = real_hir();
    let schema = real_schema();
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.atom_data.canonicalize_with_schema\n\
         use app.atom_data.status_of\nuse app.digest.hex_bytes\nfn main() Str {{\n\
         \x20   let d = {}\n    let s = {}\n    var cs = List<int>.new()\n\
         \x20   cs.push(status_of(canonicalize(d.clone())))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s, d)))\n\
         \x20   return hex_bytes(cs)\n}}",
        auto_lit(&doc),
        auto_lit(&schema)
    );
    assert_eq!(
        run_text("atom-real-hir-accept", &src),
        "0000",
        "the real HIR fixture must be accepted by the schema-free mode AND by \
         the real-schema mode (a rejection code would appear here)"
    );
}

/// Canonicalization is IDEMPOTENT on the real fixture, byte-for-byte, in the
/// schema-free mode: the document is canonicalized, then its OWN emitted bytes
/// are canonicalized again through the byte-vector entry (no decode), and the
/// two byte vectors are compared inside Auto. The byte vector the real-schema
/// mode emits for the same document is asserted equal as well, so both modes
/// provably share one writer.
#[test]
fn atom_real_hir_canonicalization_is_idempotent() {
    let doc = real_hir();
    let schema = real_schema();
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.atom_data.canonicalize_bytes\n\
         use app.atom_data.canonicalize_with_schema\n\
         use app.atom_data.payload_of\nuse app.digest.hex_bytes\n\
         fn equal(a List<int>, b List<int>) bool {{\n\
         \x20   if (a.len() != b.len()) {{ return false }}\n\
         \x20   var i = 0\n    for (i < a.len()) {{\n\
         \x20       if (a.get(i) != b.get(i)) {{ return false }}\n        i = i + 1\n    }}\n\
         \x20   return true\n}}\n\
         fn main() Str {{\n\
         \x20   let d = {}\n    let s = {}\n\
         \x20   let a1 = payload_of(canonicalize(d.clone()))\n\
         \x20   let a2 = payload_of(canonicalize_bytes(a1))\n\
         \x20   let b1 = payload_of(canonicalize_with_schema(s, d))\n\
         \x20   var flags = List<int>.new()\n\
         \x20   if (equal(a1, a2)) {{ flags.push(1) }} else {{ flags.push(0) }}\n\
         \x20   if (equal(a1, b1)) {{ flags.push(1) }} else {{ flags.push(0) }}\n\
         \x20   flags.push(a1.len() / 256)\n\
         \x20   flags.push(a1.len() % 256)\n\
         \x20   return hex_bytes(flags)\n}}",
        auto_lit(&doc),
        auto_lit(&schema)
    );
    let got = run_text("atom-real-hir-idem", &src);
    assert_eq!(
        &got[0..4],
        "0101",
        "flags: the schema-free pass is a fixpoint and both modes emit the same \
         bytes (got {got})"
    );
    let len = u16::from_str_radix(&got[4..8], 16).unwrap() as usize;
    assert!(
        len > 1024 && len < 8192,
        "the canonical document must be the whole fixture (~2.4 KiB), got {len} bytes"
    );
}

/// The same idempotence contract for the REAL-SCHEMA mode: the fixture is
/// validated + canonicalized, then its own bytes are validated + canonicalized
/// again (the schema is parsed for each call, so this is its own case).
#[test]
fn atom_real_schema_mode_is_idempotent() {
    let doc = real_hir();
    let schema = real_schema();
    let src = format!(
        "use app.atom_data.canonicalize_with_schema\n\
         use app.atom_data.canonicalize_bytes_with_schema\n\
         use app.atom_data.payload_of\nuse app.digest.hex_bytes\n\
         fn equal(a List<int>, b List<int>) bool {{\n\
         \x20   if (a.len() != b.len()) {{ return false }}\n\
         \x20   var i = 0\n    for (i < a.len()) {{\n\
         \x20       if (a.get(i) != b.get(i)) {{ return false }}\n        i = i + 1\n    }}\n\
         \x20   return true\n}}\n\
         fn main() Str {{\n\
         \x20   let d = {}\n    let s = {}\n\
         \x20   let b1 = payload_of(canonicalize_with_schema(s.clone(), d))\n\
         \x20   let b2 = payload_of(canonicalize_bytes_with_schema(s, b1))\n\
         \x20   var flags = List<int>.new()\n\
         \x20   if (equal(b1, b2)) {{ flags.push(1) }} else {{ flags.push(0) }}\n\
         \x20   flags.push(b1.len() / 256)\n\
         \x20   flags.push(b1.len() % 256)\n\
         \x20   return hex_bytes(flags)\n}}",
        auto_lit(&doc),
        auto_lit(&schema)
    );
    let got = run_text("atom-real-schema-idem", &src);
    assert_eq!(
        &got[0..2],
        "01",
        "the real-schema pass must be a byte-for-byte fixpoint (got {got})"
    );
    let len = u16::from_str_radix(&got[2..6], 16).unwrap() as usize;
    assert!(
        len > 1024 && len < 8192,
        "the canonical document must be the whole fixture, got {len} bytes"
    );
}

/// The canonical bytes of the real fixture are the frozen layout: the real
/// writer's `", "`-separated header group, `name: value` body lines at 4 spaces
/// per depth, one-line lists of inline nodes, and no `;` anywhere. (This is the
/// module's OWN layout rule; reproducing the reference writer's per-construct
/// bytes is the named 1047/1053 handoff and is NOT asserted.)
#[test]
fn atom_real_hir_canonical_layout() {
    let doc = real_hir();
    let src = format!(
        "use app.atom_data.canonicalize\nfn main() List<int> {{ return canonicalize({}) }}",
        auto_lit(&doc)
    );
    let bytes = run_bytes("atom-real-hir-layout", &src);
    assert_eq!(bytes.first(), Some(&0u8), "status 0 expected");
    let text = String::from_utf8(bytes[1..].to_vec()).expect("canonical bytes are UTF-8");
    assert!(
        text.starts_with(
            "hir(schema: \"auto.hir.core.draft\", revision: 1u32, profile: \"core-i32-draft\") {\n"
        ),
        "canonical header must use the `\", \"`-separated group: {:?}",
        &text[..text.len().min(120)]
    );
    assert!(
        !text.contains(';'),
        "the canonical form never uses `;` (group items are `\", \"`-separated, \
         body items are one per line)"
    );
    assert!(
        text.contains("\n                    local(l_a, t_i32, name: \"a\", role: param, index: 0u32) {}\n"),
        "every field is in the node group (primaries first, then fields), one node per line"
    );
    assert!(
        text.contains("bindings: [{ param: 0u32, arg: 0u32 }, { param: 1u32, arg: 1u32 }]) {}\n"),
        "a list of inline nodes is emitted on ONE line inside the group"
    );
    assert!(text.ends_with("}\n"), "the root's closing brace ends the text");
    // Idempotence of the TEXT form too: the layout has no alternative spelling.
    let again = format!(
        "use app.atom_data.canonicalize\nfn main() List<int> {{ return canonicalize({}) }}",
        auto_lit(&text)
    );
    let bytes2 = run_bytes("atom-real-hir-layout2", &again);
    assert_eq!(bytes[1..], bytes2[1..], "re-canonicalizing the text is a fixpoint");
}

/// The REAL schema drives admission: a declared tag with an undeclared field is
/// 9, an undeclared node tag is 10, and a legal document is 0. The same
/// documents are ACCEPTED by the schema-free mode (no whitelist), which is what
/// proves the two modes differ.
#[test]
fn atom_real_schema_drives_validation() {
    let schema = real_schema();
    let legal = concat!(
        "hir(schema: \"auto.hir.core.draft\", revision: 1u32, profile: \"core-i32-draft\") {\n",
        "    module m_legal {\n",
        "        path: \"examples.legal\"\n",
        "        types {\n",
        "            type t_i32 i32 {}\n",
        "        }\n",
        "    }\n",
        "}\n"
    );
    let undeclared_field = "hir(schema: \"s\", bogus: 1) {}\n";
    let undeclared_node = "hir(schema: \"s\") {\n    widget(id: \"w\") {}\n}\n";
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.atom_data.canonicalize_with_schema\n\
         use app.atom_data.status_of\nuse app.digest.hex_bytes\nfn main() Str {{\n\
         \x20   var cs = List<int>.new()\n    let s = {}\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s, {})))\n\
         \x20   var free = List<int>.new()\n\
         \x20   free.push(status_of(canonicalize({})))\n\
         \x20   free.push(status_of(canonicalize({})))\n\
         \x20   return hex_bytes(cs) + \"|@|\" + hex_bytes(free)\n}}",
        auto_lit(&schema),
        auto_lit(legal),
        auto_lit(undeclared_field),
        auto_lit(undeclared_node),
        auto_lit(undeclared_field),
        auto_lit(undeclared_node)
    );
    let got = run_text("atom-real-schema", &src);
    let parts: Vec<&str> = got.split("|@|").collect();
    assert_eq!(parts.len(), 2, "schema-mode codes and schema-free codes");
    assert_eq!(
        parts[0], "00090a",
        "legal = 0, undeclared field = 9, undeclared node = 10"
    );
    assert_eq!(
        parts[1], "0000",
        "the schema-free mode has no whitelist: the same documents are accepted"
    );
}

/// A repeated field is still 8 under the real schema — including inside an
/// INLINE node (`{ k: 1, k: 2 }`), which `schema_doc_check` never visits (an
/// inline node is a field VALUE, not a child item), so the schema mode runs the
/// schema-free mode's whole-arena duplicate pass as well. The same inline
/// duplicate is 8 through `canonicalize` too: ONE rule, two modes.
#[test]
fn atom_real_schema_duplicate_field_is_8() {
    let schema = real_schema();
    let duplicate = "hir(schema: \"s\", schema: \"t\") {}\n";
    let legal = "hir(schema: \"s\") {}\n";
    let inline_duplicate = "hir(schema: \"s\", requires: {k: 1, k: 2}) {}\n";
    let src = format!(
        "use app.atom_data.canonicalize\nuse app.atom_data.canonicalize_with_schema\n\
         use app.atom_data.status_of\nuse app.digest.hex_bytes\nfn main() Str {{\n\
         \x20   var cs = List<int>.new()\n    let s = {}\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s, {})))\n\
         \x20   cs.push(status_of(canonicalize({})))\n\
         \x20   return hex_bytes(cs)\n}}",
        auto_lit(&schema),
        auto_lit(duplicate),
        auto_lit(legal),
        auto_lit(inline_duplicate),
        auto_lit(inline_duplicate)
    );
    assert_eq!(
        run_text("atom-real-schema-dup", &src),
        "08000808",
        "tagged dup = 8, legal = 0, inline dup = 8 under the schema and 8 schema-free"
    );
}

/// N-3: the schema table maps a node's `slots:` entries to the REAL field
/// spellings (`crates/core/src/descriptor.rs:815-822`): the first `Id` slot is
/// the field `id`, `TypeUse`/`TypeRef` are both the field `type`, and any other
/// (enum) kind is the field `kind`. Registering the raw LABELS as field names
/// inverted both directions — the declared named spelling was rejected (9) while
/// `module(Id: …)` was admitted (0) where the real binder reports
/// `bind.unknown-field`.
#[test]
fn atom_real_schema_slot_field_mapping() {
    let schema = real_schema();
    // `module` slots: [Id] -> field `id`; `path` is a declared named field.
    let named = concat!(
        "hir(schema: \"s\", revision: 1u32, profile: \"p\") {\n",
        "    module(id: \"m\") {\n",
        "        path: \"p\"\n",
        "    }\n",
        "}\n"
    );
    // The slot LABEL is not a field name.
    let label = "hir(schema: \"s\") {\n    module(Id: \"m\") {}\n}\n";
    // `type` slots: [Id, TypeKind] -> fields `id` + `kind`.
    let enum_kind =
        "hir(schema: \"s\") {\n    types {\n        type(id: \"t\", kind: i32) {}\n    }\n}\n";
    // `local` slots: [Id, TypeUse] -> fields `id` + `type`.
    let type_use = concat!(
        "hir(schema: \"s\") {\n",
        "    bodies {\n",
        "        body(id: \"b\") {\n",
        "            locals {\n",
        "                local(id: \"l\", type: t_i32) {}\n",
        "            }\n",
        "        }\n",
        "    }\n",
        "}\n"
    );
    let src = format!(
        "use app.atom_data.canonicalize_with_schema\nuse app.atom_data.status_of\n\
         use app.digest.hex_bytes\nfn main() Str {{\n    var cs = List<int>.new()\n\
         \x20   let s = {}\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s, {})))\n\
         \x20   return hex_bytes(cs)\n}}",
        auto_lit(&schema),
        auto_lit(named),
        auto_lit(label),
        auto_lit(enum_kind),
        auto_lit(type_use)
    );
    assert_eq!(
        run_text("atom-real-schema-slots", &src),
        "00090000",
        "named-slot document = 0, raw slot label = 9, Id+kind = 0, Id+type = 0"
    );
}

/// N-7: a `containers { name: [tags] }` entry is a CHILD TAG, never a named
/// field of its parent. The real binder keeps container names in
/// `spec.containers` (`crates/core/src/descriptor.rs:446-449`) and admits a
/// named field only from `spec.fields` plus the slot mapping (`:815-833`), so
/// `module(types: …)` / `module(bodies: …)` are `bind.unknown-field` (9) while
/// `module { types { … } }` is the legal spelling (0). The two field controls
/// (`path` = declared field, `bogus` = undeclared) pin that the whitelist still
/// works in both directions after the change.
#[test]
fn atom_real_schema_container_names_are_tags_not_fields() {
    let schema = real_schema();
    let container_as_field = "hir(schema: \"s\") {\n    module(types: 1) {}\n}\n";
    let container_as_field2 = "hir(schema: \"s\") {\n    module(bodies: 1) {}\n}\n";
    let declared_field = "hir(schema: \"s\") {\n    module(path: \"p\") {}\n}\n";
    let undeclared_field = "hir(schema: \"s\") {\n    module(bogus: 1) {}\n}\n";
    let container_as_child = concat!(
        "hir(schema: \"s\") {\n",
        "    module {\n",
        "        types {\n",
        "            type(id: \"t\", kind: i32) {}\n",
        "        }\n",
        "    }\n",
        "}\n"
    );
    let src = format!(
        "use app.atom_data.canonicalize_with_schema\nuse app.atom_data.status_of\n\
         use app.digest.hex_bytes\nfn main() Str {{\n    var cs = List<int>.new()\n\
         \x20   let s = {}\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s.clone(), {})))\n\
         \x20   cs.push(status_of(canonicalize_with_schema(s, {})))\n\
         \x20   return hex_bytes(cs)\n}}",
        auto_lit(&schema),
        auto_lit(container_as_field),
        auto_lit(container_as_field2),
        auto_lit(declared_field),
        auto_lit(undeclared_field),
        auto_lit(container_as_child)
    );
    assert_eq!(
        run_text("atom-real-schema-containers", &src),
        "0909000900",
        "container-as-field = 9, second container-as-field = 9, declared field = 0, \
         undeclared field = 9, container-as-child-node = 0"
    );
}

/// The real-schema mode emits the SAME canonical layout as the schema-free mode
/// (one writer): a small legal HIR document canonicalized under
/// `core-i32.atom` must equal the bytes written from the writer rules — the
/// `", "` group, the 4-space body lines, the one-line list and the final
/// newline. The expectation is written from the frozen rules, not copied from
/// Auto output.
#[test]
fn atom_real_schema_mode_canonical_layout() {
    let schema = real_schema();
    let doc = concat!(
        "hir(schema: \"auto.hir.core.draft\", revision: 1u32, profile: \"core-i32-draft\") {\n",
        "    module m_legal {\n",
        "        path: \"examples.legal\"\n",
        "        types { type t_i32 i32 {} }\n",
        "    }\n",
        "}\n"
    );
    let expected = concat!(
        "hir(schema: \"auto.hir.core.draft\", revision: 1u32, profile: \"core-i32-draft\") {\n",
        "    module(m_legal, path: \"examples.legal\") {\n",
        "        types {\n",
        "            type(t_i32, i32) {}\n",
        "        }\n",
        "    }\n",
        "}\n"
    );
    let src = format!(
        "use app.atom_data.canonicalize_with_schema\nfn main() List<int> {{\n\
         \x20   return canonicalize_with_schema({}, {})\n}}",
        auto_lit(&schema),
        auto_lit(doc)
    );
    let bytes = run_bytes("atom-real-schema-layout", &src);
    assert_eq!(bytes.first(), Some(&0u8), "status 0 expected");
    assert_eq!(
        String::from_utf8(bytes[1..].to_vec()).unwrap(),
        expected,
        "the real-schema mode emits the frozen canonical layout"
    );
}

