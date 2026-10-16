//! Shared helpers for the PLAN-791 frozen-behavior suites
//! (`v02_compat_golden.rs` / `v03_failure_semantics.rs`): fixture paths and
//! the standard in-memory data atom. Golden-tree machinery lives inside
//! `v02_compat_golden.rs` (only that suite compiles it).

use std::path::PathBuf;

pub fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures")
}

pub fn templates_dir() -> PathBuf {
    fixture_dir().join("templates")
}

/// Standard in-memory data atom used by the frozen cases. Mirrors the shape
/// auto-man passes to the legacy API (a to_atom() node) and exercises both
/// scalar interpolation and `$ for` code lines over an array of nodes.
pub fn sample_atom() -> auto_lang::atom::Atom {
    use auto_lang::atom::Atom;
    use auto_val::{Node, Value};

    let mut alpha = Node::new("item");
    alpha.set_prop("name", "alpha");
    alpha.set_prop("id", 1);
    let mut beta = Node::new("item");
    beta.set_prop("name", "beta");
    beta.set_prop("id", 2);
    let items = auto_val::Array::from_vec(vec![Value::node(alpha), Value::node(beta)]);

    Atom::assemble(vec![
        Value::pair("name", "AuthService"),
        Value::pair("version", 2),
        Value::pair("items", Value::array(items)),
    ])
    .unwrap()
}
