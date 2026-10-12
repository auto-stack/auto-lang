//! Parity with `auto-atom::AtomParser` (Plan 768 feasibility §7): both
//! parsers agree on valid inputs; the strict parser additionally rejects
//! what the contract layer must not silently normalize (duplicate keys,
//! node args/children, bare identifiers).

use auto_atom::AtomParser;
use auto_test_case::atom::{parse_contract, CVal};

const VALID: &[&str] = &[
    "case { schema: \"autolang.case/1\" }",
    "obj { a: 1, b: -2, c: 3.5, d: 1e3, e: -2.5e-2 }",
    "obj { s: \"quoted \\\"str\\\"\", t: \"tab\\there\" }",
    "obj { t: true, f: false, n: null }",
    "[1, \"two\", [3, [4]], { k: \"v\" }]",
    "// comment\nobj { /* mid */ k: \"v\", }",
    "node { a: 1 }",
    "obj { nested: { deep: { deeper: \"x\" } } }",
];

const INVALID_BOTH: &[&str] = &[
    "obj { k: }",
    "obj { : 1 }",
    "obj { k 1 }",
    "[1, 2,",
    "\"unterminated",
    "obj { k: 1 } trailing",
    "obj { k: 1 } obj { }",
];

#[test]
fn both_parsers_accept_the_valid_subset() {
    for src in VALID {
        let strict =
            parse_contract(src).unwrap_or_else(|e| panic!("strict rejected {:?}: {}", src, e));
        let atom = AtomParser::parse(src)
            .unwrap_or_else(|e| panic!("auto-atom rejected {:?}: {}", src, e));
        // Same document shape class: node vs object vs scalar/array.
        match (&strict, atom.is_node()) {
            (CVal::Node { .. }, true) => {}
            (CVal::Node { .. }, false) => panic!("shape disagreement on {:?}", src),
            (other, node) => {
                assert!(
                    !node,
                    "auto-atom saw a node where strict saw {}",
                    other.type_name()
                );
            }
        }
    }
}

#[test]
fn both_parsers_reject_shared_malformed_input() {
    for src in INVALID_BOTH {
        assert!(parse_contract(src).is_err(), "strict accepted {:?}", src);
        assert!(
            AtomParser::parse(src).is_err(),
            "auto-atom accepted {:?}",
            src
        );
    }
}

#[test]
fn strict_rejects_what_upstream_silently_normalizes() {
    // Empty input: upstream produces `Atom::Empty`; a contract must be an
    // actual value, so strict rejects.
    assert!(parse_contract("").is_err());

    // Duplicate keys: upstream upserts (last wins); strict errors.
    assert!(parse_contract("obj { k: 1, k: 2 }").is_err());
    assert!(
        AtomParser::parse("obj { k: 1, k: 2 }").is_ok(),
        "upstream keeps accepting (documented asymmetry)"
    );

    // Node args: upstream parses; strict rejects (not in the contract subset).
    assert!(parse_contract("n(a: 1) { }").is_err());
    assert!(AtomParser::parse("n(a: 1) { }").is_ok());

    // Bare identifiers as values: upstream maps them to strings; strict
    // rejects so misspelled keywords cannot degrade into string literals.
    assert!(parse_contract("obj { k: bare }").is_err());
    assert!(AtomParser::parse("obj { k: bare }").is_ok());
}

#[test]
fn strict_decodes_the_design_example_shape() {
    let src = r#"case {
  schema: "autolang.case/1"
  id: "lang.scalar.precedence"
  tags: ["scalar", "precedence"]
  source: { block: "auto-source", syntax: "auto", path: "main.at" }
  entry: { mode: "program" }
  checks: [
    { id: "vm", adapter: "avm", action: "run",
      expect: { completion: "success", result: { format: "legacy-display", text: "14" } } }
  ]
}"#;
    let v = parse_contract(src).unwrap();
    let (name, props) = v.as_node().unwrap();
    assert_eq!(name, "case");
    assert_eq!(props.len(), 6);
}
