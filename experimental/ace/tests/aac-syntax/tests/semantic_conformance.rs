#![cfg(windows)]

use ace_aac_syntax_conformance::{run_semantic_case, run_semantic_project, run_type_case};

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

fn type_report(source: &str) -> String {
    run_type_case(source.as_bytes(), None).expect("Auto type stage driver")
}

#[test]
fn checks_named_arguments_against_their_formal_types() {
    let source = concat!(
        "fn bump() int { return 19 }\n",
        "fn ready() bool { return true }\n",
        "fn f(a int, b bool) int { return a }\n",
        "fn main() int { return f(b: ready(), a: bump()) }\n",
    );
    assert!(type_report(source).starts_with("aac.type-report/1\n"));
    let bad = source.replace("b: ready()", "b: 19");
    let start = bad.rfind("19").unwrap();
    assert_eq!(
        type_report(&bad),
        format!("ERROR type.argument 0 {start}..{}\n", start + 2)
    );
}

#[test]
fn rejects_wrong_return_and_conflicting_branch_types() {
    let good = "fn main() int { return if true { 17 } else { 9 } }";
    assert!(type_report(good).starts_with("aac.type-report/1\n"));
    let bad = good.replace("{ 9 }", "{ false }");
    let from = bad.find("if true").unwrap();
    let to = bad.rfind('}').unwrap() - 1;
    assert_eq!(
        type_report(&bad),
        format!("ERROR type.branch 0 {from}..{to}\n")
    );
    let wrong_return = "fn main() int { return false }";
    let from = wrong_return.find("false").unwrap();
    assert_eq!(
        type_report(wrong_return),
        format!("ERROR type.return 0 {from}..{}\n", from + 5)
    );
}

#[test]
fn preserves_nested_option_result_future_and_iterator_identity() {
    let source = concat!(
        "fn first(x Option<Result<int,bool>>) Option<Result<int,bool>> { return x }\n",
        "fn later(x Future<List<?int>>) Future<List<?int>> { return x }\n",
        "fn stream(x ~Iter<int>) ~Iter<int> { return x }\n",
    );
    let report = type_report(source);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let entries: Vec<[u32; 4]> = report
        .lines()
        .filter_map(|line| line.strip_prefix("TYPE "))
        .map(|fields| {
            fields
                .split_whitespace()
                .map(|part| part.parse().unwrap())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap()
        })
        .collect();
    let lookup = |kind, first, second| {
        entries
            .iter()
            .find(|row| row[1..] == [kind, first, second])
            .unwrap_or_else(|| panic!("missing TYPE {kind} {first} {second}\n{report}"))[0]
    };
    let result_int_bool = lookup(21, 1, 2);
    let option_result = lookup(20, result_int_bool, 0);
    let option_int = lookup(20, 1, 0);
    let list_option = lookup(24, option_int, 0);
    let future_list = lookup(22, list_option, 0);
    assert_ne!(future_list, option_result);
    assert_ne!(lookup(23, 1, 0), future_list);
    let unwrapped = "fn show(x ?int) int { return x }";
    let start = unwrapped.rfind('x').unwrap();
    assert_eq!(
        type_report(unwrapped),
        format!("ERROR type.return 0 {start}..{}\n", start + 1)
    );
}
