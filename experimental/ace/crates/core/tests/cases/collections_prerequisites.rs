//! Named variants must use one nominal lookup in every legal source position.
use ace_core::source::compile_collections_source;

fn accepted(src: &str) {
    compile_collections_source(src, "p889.at", &[])
        .unwrap_or_else(|ds| panic!("expected valid source: {ds:?}"));
}

fn rejected(src: &str, code: &str) {
    match compile_collections_source(src, "p889.at", &[]) {
        Ok(_) => panic!("expected {code}, but compiled: {src}"),
        Err(ds) => assert!(ds.iter().any(|d| d.code == code), "expected {code}: {ds:?}"),
    }
}

#[test]
fn named_variant_list_owned_payload() {
    accepted(include_str!("../fixtures/collections_889/list_variant.at"));
}

#[test]
fn nested_forward_variant_return_and_local() {
    accepted(include_str!(
        "../fixtures/collections_889/nested_variant.at"
    ));
}

#[test]
fn optional_variant_list_control() {
    accepted(include_str!(
        "../fixtures/collections_889/optional_variant.at"
    ));
}

#[test]
fn owned_text_clone_uses_the_existing_shared_expression() {
    accepted("fn main() int { let a Str = \"中\"\n let b = a.clone()\n return a.len() * 40 + b.len() * 2 }");
}

#[test]
fn recursive_variant_graphs_reject_before_emission() {
    for src in [
        "enum A { Next(A) }\nfn main() int { return 0 }",
        "enum A { Next(B) }\nenum B { Next(A) }\nfn main() int { return 0 }",
        "enum A { Next(?A) }\nfn main() int { return 0 }",
    ] {
        rejected(src, "source.type-mismatch");
    }
}

#[test]
fn nominal_lookup_does_not_admit_unknown_or_wrong_type() {
    rejected(
        "fn main() int { var xs = List<Missing>.new()\n return 0 }",
        "source.undefined",
    );
    rejected("enum A { Data(int) }\nenum B { Data(int) }\nfn main() int { var xs = List<A>.new()\n xs.push(B.Data(42))\n return 0 }", "source.type-mismatch");
    rejected(
        "enum A { Data(int) }\nenum A { End }\nfn main() int { return 0 }",
        "source.duplicate",
    );
}

#[test]
fn named_variant_context_restrictions_match_optional() {
    for src in [
        "enum A { Data(int) }\nfn f(x A) int { return 0 }\nfn main() int { return 0 }",
        "enum A { Data(int) }\ntype R { x A }\nfn main() int { return 0 }",
        "enum A { Data(int) }\nfn f() List<A> { var xs = List<A>.new()\n return xs }\nfn main() int { return 0 }",
        "enum A { Data(int) }\nfn main() int { var xs = List<List<A>>.new()\n return 0 }",
    ] {
        rejected(src, "source.list-value-context");
    }
}

#[test]
fn container_source_clone_still_rejected() {
    rejected(
        "fn main() int { var xs = List<int>.new()\n let ys = xs.clone()\n return 0 }",
        "source.unsupported",
    );
    rejected("enum A { Data(Str) }\nfn main() int { let a = A.Data(\"x\")\n let b = a.clone()\n return 0 }", "source.undefined");
}
