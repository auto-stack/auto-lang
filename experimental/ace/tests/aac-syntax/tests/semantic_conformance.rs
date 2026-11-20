#![cfg(windows)]

use ace_aac_syntax_conformance::{run_semantic_case, run_semantic_project};

fn observe(source: &str) -> String {
    run_semantic_case(source.as_bytes(), None).expect("Auto semantic stage driver")
}

#[test]
fn forwards_and_maps_named_arguments_in_source_order() {
    let source = concat!(
        "var log int = 0\n",
        "fn main() int { return f(b: B(), a: A()) }\n",
        "fn A() int { log = 2\n return 37 }\n",
        "fn B() bool { log = 1\n return true }\n",
        "fn f(a int, b bool) int { return a }\n",
    );
    let report = observe(source);
    assert!(report.starts_with("aac.binding-report/1\n"), "{report}");
    let evaluation_order: Vec<_> = report
        .lines()
        .filter(|line| line.starts_with("ARG "))
        .map(|line| {
            let columns: Vec<_> = line.split_whitespace().collect();
            let (start, end) = columns[5].split_once("..").unwrap();
            let start: usize = start.parse().unwrap();
            let end: usize = end.parse().unwrap();
            (columns[3], &source[start..end])
        })
        .collect();
    assert_eq!(evaluation_order, [("1", "B()"), ("0", "A()")]);
}

#[test]
fn block_shadowing_initializer_sees_previous_scope() {
    let source = concat!(
        "fn f(x int) int {\n",
        " if true { let x = x + 1\n return x }\n",
        " return x\n",
        "}\n",
    );
    let report = observe(source);
    let entries: Vec<_> = report
        .lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>())
        .collect();
    let declarations: Vec<_> = entries
        .iter()
        .filter(|cols| cols[0] == "DECL" && cols[3] == "x")
        .collect();
    assert_eq!(declarations.len(), 2, "{report}");
    assert_eq!((declarations[0][2], declarations[1][2]), ("2", "3"));
    let targets: Vec<_> = entries
        .iter()
        .filter(|cols| cols[0] == "USE" && cols[2] == "x")
        .map(|cols| cols[1])
        .collect();
    assert_eq!(
        targets,
        [declarations[0][1], declarations[1][1], declarations[0][1]]
    );
}

#[test]
fn rejects_out_of_scope_reference_at_the_written_name() {
    let source = "fn f() int { if true { let hidden = 5 }\n return hidden }";
    let start = source.rfind("hidden").unwrap();
    assert_eq!(
        observe(source),
        format!("ERROR bind.unresolved 0 {start}..{}\n", start + 6)
    );
}

#[test]
fn distinguishes_semantic_exhaustion_from_a_source_error() {
    let source = b"fn f() int { return 3 }";
    let report = run_semantic_case(source, Some(2)).expect("Auto semantic stage driver");
    assert!(report.starts_with("ERROR resource.semantic 0 "), "{report}");
    assert!(observe(std::str::from_utf8(source).unwrap()).starts_with("aac.binding-report/1\n"));
}

#[test]
fn rejects_duplicate_named_argument() {
    let source =
        "fn f(a int, b bool) int { return a }\nfn main() int { return f(b: true, b: false) }";
    let start = source.find("b: false").unwrap();
    assert_eq!(
        observe(source),
        format!("ERROR bind.arguments 0 {start}..{}\n", start + 8)
    );
}

fn project(modules: &[(&str, &str)]) -> String {
    run_semantic_project(modules).expect("Auto semantic project driver")
}

#[test]
fn resolves_aliases_and_qualified_recursive_calls_across_modules() {
    let root = "use bridge: labeled\nfn main() int { return labeled(n: 4) }";
    let bridge = "pub use ops.core_fn as labeled";
    let ops =
        "pub fn core_fn(n int) int { if n == 0 { return 1 }\n return app.ops.core_fn(n - 1) }";
    let report = project(&[("root", root), ("bridge", bridge), ("ops", ops)]);
    let declaration = report
        .lines()
        .find(|line| line.contains(" 1 core_fn 2 "))
        .unwrap_or_else(|| panic!("missing public definition\n{report}"));
    let target = declaration.split_whitespace().nth(1).unwrap();
    assert!(
        report.contains(&format!("USE {target} labeled 0 ")),
        "{report}"
    );
    assert!(report.contains(&format!("USE {target} app 2 ")), "{report}");
}

#[test]
fn rejects_private_import_and_import_cycle_separately() {
    let root = "use beta.hidden\nfn main() int { return hidden() }";
    let private = "fn hidden() int { return 4 }";
    let public = "pub fn hidden() int { return 4 }";
    let start = root.find("hidden").unwrap();
    assert_eq!(
        project(&[("alpha", root), ("beta", private)]),
        format!("ERROR bind.private 0 {start}..{}\n", start + 6)
    );
    assert!(project(&[("alpha", root), ("beta", public)]).starts_with("aac.binding-report/1\n"));
    let cycle = project(&[
        (
            "alpha",
            "use beta.back\npub fn there() int { return back() }",
        ),
        (
            "beta",
            "use alpha.there\npub fn back() int { return there() }",
        ),
    ]);
    assert!(cycle.starts_with("ERROR bind.import-cycle "), "{cycle}");
}

#[test]
fn orders_global_initializers_using_transitive_reads() {
    let root = concat!(
        "use base.Seed\n",
        "var Buffer int = calculate()\n",
        "fn calculate() int { return Later }\n",
        "const Later int = Seed + 1\n",
        "fn main() int { return Buffer }\n",
    );
    let base = "pub const Seed int = 37";
    let report = project(&[("root", root), ("base", base)]);
    let values: Vec<_> = report
        .lines()
        .filter_map(|line| line.strip_prefix("INIT "))
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(values, ["Seed", "Later", "Buffer"], "{report}");

    let circular = root.replace("Seed + 1", "Buffer + 1");
    let report = project(&[("root", &circular), ("base", base)]);
    assert!(
        report.starts_with("ERROR bind.initializer-cycle 0 "),
        "{report}"
    );
}
