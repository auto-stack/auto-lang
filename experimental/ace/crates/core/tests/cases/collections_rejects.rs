//! PLAN-864 negative diagnostics (contract C10 / AC-02): each excluded or
//! ill-formed input rejects with its frozen named code — never a silent
//! acceptance, never a generic syntax error where a named code is frozen.

use ace_core::source::compile_collections_source;

fn rejects(name: &str, src: &str, code: &str) {
    match compile_collections_source(src, "probe.at", &[]) {
        Ok(_) => panic!("{name}: expected rejection with `{code}`, compiled"),
        Err(diags) => {
            let codes: Vec<&str> = diags.iter().map(|d| d.code.as_str()).collect();
            assert!(
                codes.contains(&code),
                "{name}: expected code `{code}` in {codes:?} ({})",
                diags
                    .iter()
                    .map(|d| d.render("probe.at", src))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        }
    }
}

#[test]
fn reject_non_exhaustive_is() {
    rejects(
        "non-exhaustive",
        "fn maybe(x int) ?int { return None }\nfn main() int {\n    let x = maybe(1)\n    is x {\n        Some(v) -> { return v }\n    }\n}\n",
        "source.pattern-non-exhaustive",
    );
}

#[test]
fn reject_duplicate_arm() {
    rejects(
        "duplicate arm",
        "fn maybe(x int) ?int { return None }\nfn main() int {\n    let x = maybe(1)\n    is x {\n        Some(v) -> { return v }\n        Some(w) -> { return w }\n    }\n}\n",
        "source.pattern-duplicate",
    );
}

#[test]
fn reject_unknown_arm() {
    rejects(
        "unknown arm",
        "fn maybe(x int) ?int { return None }\nfn main() int {\n    let x = maybe(1)\n    is x {\n        Some(v) -> { return v }\n        Nope -> { return 1 }\n    }\n}\n",
        "source.pattern-unknown-arm",
    );
}

#[test]
fn reject_pattern_scrutinee_not_variant_local() {
    rejects(
        "is scrutinee is not a variant-typed local",
        "fn main() int {\n    let x = 5\n    is x {\n        Some(v) -> { return v }\n        None -> { return 1 }\n    }\n}\n",
        "source.pattern-scrutinee",
    );
}

#[test]
fn reject_view_escape_bare_get() {
    rejects(
        "bare get of owned element",
        "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    let y Item = xs.get(0)\n    return 0\n}\n",
        "source.view-escape",
    );
}

#[test]
fn reject_iterate_mutate() {
    rejects(
        "push during for-in",
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(1)\n    for x in xs {\n        xs.push(2)\n    }\n    return 0\n}\n",
        "source.iterate-mutate",
    );
}

#[test]
fn reject_move_required() {
    rejects(
        "push owned local without .move",
        "type Item { name Str; code int }\nfn main() int {\n    var xs = List<Item>.new()\n    let x Item = Item(\"a\", 1)\n    xs.push(x)\n    return 0\n}\n",
        "source.move-required",
    );
}

#[test]
fn reject_list_param() {
    rejects(
        "List-typed parameter",
        "fn f(xs List<int>) int { return 0 }\nfn main() int { return 0 }\n",
        "source.list-value-context",
    );
}

#[test]
fn reject_nested_list() {
    rejects(
        "List of List",
        "fn main() int {\n    var xs = List<List<int>>.new()\n    return 0\n}\n",
        "source.list-value-context",
    );
}

#[test]
fn reject_bare_none_ambiguous() {
    rejects(
        "bare None without an expected type",
        "fn main() int {\n    let x = None\n    return 0\n}\n",
        "source.type-ambiguous",
    );
}

#[test]
fn reject_record_container_field() {
    rejects(
        "List-typed record field",
        "type Bad { xs List<int> }\nfn main() int { return 0 }\n",
        "source.list-value-context",
    );
}

#[test]
fn reject_str_slice_type() {
    rejects(
        "borrowed str in the collections surface",
        "fn main() int {\n    let s str = \"a\"\n    return 0\n}\n",
        "source.unsupported",
    );
}

#[test]
fn reject_owner_record_method() {
    rejects(
        "record methods stay excluded",
        "type Item { name Str; code int\n    fn hit() int { return 1 }\n}\nfn main() int { return 0 }\n",
        "source.unsupported",
    );
}
