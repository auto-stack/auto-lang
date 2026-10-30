use ace_core::{hir, source, verify};
use source::adapter::compile_host_console_source as compile;

fn good(src: &str) -> source::CompiledSource {
    compile(src, "console.at", &[]).unwrap_or_else(|ds| panic!("{ds:#?}"))
}

fn rejects(src: &str, code: &str) {
    let ds = compile(src, "console.at", &[]).err().expect("must reject");
    assert!(
        ds.iter().any(|d| d.code == code),
        "wanted {code}, got {ds:#?}"
    );
}

#[test]
fn av_console_typed_symbols_and_capabilities() {
    let c = good("fn main() int {\n print(-2147483648)\n print(true)\n eprint(false)\n print(\"你好\\0\\n\")\n eprint('界')\n return 7\n}\n");
    assert_eq!(c.checked.domain(), hir::HirDomain::HostConsole);
    let caps = c.checked.required_capabilities();
    assert!(caps.iter().any(|c| c == "console.stdout"));
    assert!(caps.iter().any(|c| c == "console.stderr"));
    assert!(!caps.iter().any(|c| c == source::TRACE_CAPABILITY));
    let m = &c.checked.bundle().modules[0];
    let symbols: Vec<_> = m
        .defs
        .iter()
        .filter_map(|d| match &d.kind {
            hir::DefKind::Intrinsic { symbol, .. } => Some(symbol.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(symbols.len(), 5);
    assert!(symbols.contains(&"auto.console.print.i32.v1"));
    assert!(symbols.contains(&"auto.console.eprint.char.v1"));
    let rebound =
        ace_core::descriptor::bind(ace_core::host_console_descriptor(), &c.atom_text).unwrap();
    verify::verify_host_console(rebound).unwrap();
}

#[test]
fn av_console_unit_entry_script_and_nested_effects() {
    good("fn main() void {\n if true {\n print(1)\n }\n eprint(2)\n}\n");
    good("print(1)\neprint(2)\n");
    good("print(1)\n7\n");
    rejects("fn main() int {\n print(1)\n 7\n}\n", "source.entry-tail");
    rejects("print(1)\ntrue\n", "source.entry-tail-type");
}

#[test]
fn av_console_owned_and_borrowed_text() {
    let c = good("fn main() int {\n let text Str = \"你好\"\n print(text)\n eprint(text)\n print(\"ok\")\n return text.len()\n}\n");
    assert!(c.atom_text.contains("text_to_borrowed"));
    assert!(c.checked.requires_runtime_text());
}

#[test]
fn av_console_wrong_arguments_and_unit_storage_reject() {
    for call in [
        "print()",
        "print(1, 2)",
        "eprint(value: 1)",
        "print(print(1))",
    ] {
        rejects(
            &format!("fn main() void {{\n {call}\n}}\n"),
            "source.console-argument",
        );
    }
    rejects(
        "fn main() void {\n let x = print(1)\n}\n",
        "source.unit-placement",
    );
}

#[test]
fn av_console_reserved_names_only_in_new_domain() {
    for src in [
        "fn print() int { return 1 }\nfn main() int { return 0 }",
        "fn main() int {\n let print = 1\n return print\n}",
        "fn helper(eprint int) int { return 0 }\nfn main() int { return 0 }",
    ] {
        rejects(src, "source.console-reserved");
    }
    source::compile_scalar_source(
        "fn print() int { return 3 }\nfn main() int { return print() }",
        "old.at",
        &[],
    )
    .unwrap();
    let ds = source::compile_entry_unit_source("fn main() void {\n print(1)\n}", "old.at", &[])
        .err()
        .unwrap();
    assert!(ds.iter().any(|d| d.code == "source.undefined"));
}

#[test]
fn av_console_hir_identity_and_signature_mutations() {
    let c = good("fn main() void {\n print(1)\n}\n");
    let original = c.checked.bundle().clone();
    assert!(verify::verify_entry_unit(original.clone())
        .err()
        .unwrap()
        .iter()
        .any(|d| d.code == "verify.domain"));
    let mut b = original.clone();
    b.requires.retain(|r| r != "console.stdout");
    assert!(verify::verify_host_console(b)
        .err()
        .unwrap()
        .iter()
        .any(|d| d.code == "host.unauthorized"));
    let mut b = original.clone();
    for d in &mut b.modules[0].defs {
        if let hir::DefKind::Intrinsic { symbol, .. } = &mut d.kind {
            *symbol = "auto.console.print.bool.v1".into();
        }
    }
    assert!(verify::verify_host_console(b)
        .err()
        .unwrap()
        .iter()
        .any(|d| d.code == "host.signature"));
    let mut b = original;
    for d in &mut b.modules[0].defs {
        if let hir::DefKind::Intrinsic { symbol, .. } = &mut d.kind {
            *symbol = "auto.console.print.unknown.v1".into();
        }
    }
    assert!(verify::verify_host_console(b)
        .err()
        .unwrap()
        .iter()
        .any(|d| d.code == "host.unknown-symbol"));
}

#[test]
fn av_console_no_host_intrinsic_in_legacy_domain() {
    let old = source::compile_scalar_source(
        "fn main() int { return mark_a() }",
        "old.at",
        &[source::TRACE_CAPABILITY.into()],
    )
    .unwrap();
    let mut b = old.checked.bundle().clone();
    for d in &mut b.modules[0].defs {
        if let hir::DefKind::Intrinsic { symbol, .. } = &mut d.kind {
            *symbol = "auto.console.print.i32.v1".into();
        }
    }
    assert!(verify::verify_scalar(b)
        .err()
        .unwrap()
        .iter()
        .any(|d| d.code == "host.profile"));
}

#[test]
fn av_console_invalid_even_in_dead_source_and_no_unit_coercion() {
    rejects(
        "fn main() void {\n if false { print() }\n}\n",
        "source.console-argument",
    );
    rejects(
        "fn unused() int { return print(1) }\nfn main() int { return 0 }",
        "source.type-mismatch",
    );
    rejects(
        "fn main() int {\n if print(1) { return 1 }\n return 0\n}",
        "source.type-mismatch",
    );
    rejects(
        "fn main() void {\n let x void = print(1)\n}",
        "source.unit-placement",
    );
    rejects(
        "fn main() int { return 0 }\nprint(1)",
        "source.entry-ambiguous",
    );
}
