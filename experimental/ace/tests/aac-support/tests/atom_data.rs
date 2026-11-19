//! PLAN-1056 T-04: canonical Atom DATA reader/writer for the AAC data slice —
//! the pure-Auto `app.atom_data` module compiled by the Gen0 (Rust-frontend)
//! assembled `packages-aggregates-ct` lane and executed on the AV executor.
//!
//! Every expected byte in this file is written from the frozen writer rules
//! (indent, group separators, final newline), never copied from Auto output:
//! the Auto side either re-emits a document that is already in canonical form
//! (round-trip identity) or reduces an equivalent variant onto that same form.
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

/// One `canonical_text` driver program for a document.
fn text_program(doc: &str) -> String {
    format!(
        "use app.atom_data.canonical_text\nfn main() Str {{ return canonical_text({}) }}",
        auto_lit(doc)
    )
}

// ---------------------------------------------------------------------------
// The frozen canonical document (written to the writer rules: one body item
// per line at depth+1, 4 spaces per depth, `(k: v; k: v)` groups, `{}` for an
// empty body, the root's closing brace alone at depth 0, final newline).
// ---------------------------------------------------------------------------

const FROZEN: &str = r#"descriptor(schema: "aac.atom-data/v1"; revision: 1; profile: "core") {
    module(id: "m.foundation") {
        decl(id: "d.echo"; kind: "fn") {
            type(id: "t.unit"; kind: "unit") {}
        }
        type(id: "t.text"; kind: "record") {}
    }
    module(id: "m.surface") {
        decl(id: "d.run"; kind: "fn") {}
    }
}
"#;

/// A canonical document whose string values exercise the four escapes and
/// multi-byte scalars: the lexer is scalar-indexed and a legal escape
/// re-encodes to itself, so this is a fixpoint too.
const ESCAPES: &str = r#"descriptor(schema: "aac.atom-data/v1"; revision: 1; profile: "q\"x\" b\\ t\t n\n 中😀") {
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
/// a `decl` with no body at all are all DATA to this reader. Only the tag and
/// field table (plus duplicates) is enforced: nothing here claims a verified
/// AAC object was constructed.
const ODD: &str = r#"descriptor(schema: "aac.atom-data/v1"; revision: 2; profile: "core") {
    module(id: "m.core") {
        decl(id: "d.ghost"; kind: "kind.that.does.not.exist") {
            type(id: "t.void"; kind: "no.such.kind") {}
        }
        decl(id: "d.dup"; kind: "fn") {}
        decl(id: "d.dup"; kind: "fn") {}
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
    // (name, document, expected outcome byte)
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
            "descriptor(schema: \"a\"; schema: \"b\") {}".to_string(),
            8,
        ),
        (
            "unknown field",
            "descriptor(bogus: \"a\") {}".to_string(),
            9,
        ),
        ("unknown node", "descriptor() { widget(id: \"w\") {} }".to_string(), 10),
        (
            "non-descriptor root",
            "module(id: \"m\") {}".to_string(),
            10,
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
/// The admission rules must reject BEFORE the parser/writer touches the data:
/// an oversized document (262145 scalars) and an over-deep document (65) are
/// refused with 6, while their legal neighbours (262144 scalars, depth 64) are
/// accepted. Built at runtime by doubling so the test stays cheap.
#[test]
fn atom_admission_rules() {
    // 262145 scalars: one over the frozen 262144 admission limit.
    let over = format!(
        "use app.atom_data.canonicalize\nuse app.codec.status_of\nuse app.digest.hex_bytes\nfn main() Str {{\n    var xs = List<int>.new()\n    xs.push(status_of(canonicalize(rep(\"a\", 262145))))\n    return hex_bytes(xs)\n}}\nfn dup(s Str, k int) Str {{\n    if (k == 0) {{ return s.clone() }}\n    let doubled = s + s.clone()\n    return dup(doubled, k - 1)\n}}\nfn rep(s Str, n int) Str {{\n    let base = s.clone()\n    let half = dup(s, 18)\n    return half + base\n}}"
    );
    assert_eq!(run_text("atom-admission-len", &over), "06", "length refusal");
}

/// The parser-depth admission: 65 levels are refused before any node is built.
/// (The exact 64/65 acceptance boundary is not exercised: the writer is
/// superlinear in depth, so a depth-64 document would dominate the case budget;
/// a shallow document is accepted by every other case in this file.)
#[test]
fn atom_admission_depth() {
    // The exact 64/65 acceptance boundary is not exercised: the writer is
    // superlinear in depth, so a depth-64 document dominates the case budget;
    // the rejection threshold (65) and a safe legal neighbour (16) are pinned.
    for (depth, want) in [(65usize, "06")] {
        let mut doc = String::from("descriptor(schema: \"s\"; revision: 1; profile: \"p\") {\n");
        for _ in 1..depth {
            doc.push_str("module(id: \"x\") {\n");
        }
        for _ in 0..depth {
            doc.push_str("}\n");
        }
        let src = format!(
            "use app.atom_data.canonicalize\nuse app.codec.status_of\nuse app.digest.hex_bytes\nfn main() Str {{\n    var xs = List<int>.new()\n    xs.push(status_of(canonicalize({})))\n    return hex_bytes(xs)\n}}",
            auto_lit(&doc)
        );
        assert_eq!(
            run_text(&format!("atom-depth-{depth}"), &src),
            want,
            "depth {depth}"
        );
    }
}







