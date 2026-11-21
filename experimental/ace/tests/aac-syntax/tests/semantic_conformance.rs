#![cfg(windows)]

use ace_aac_syntax_conformance::run_global_hir_admission;
use ace_aac_syntax_conformance::{
    observe_hir_publication, observe_project_inventory, run_base_mir_case, run_base_mir_control,
    run_core_hir_case, run_core_hir_project, run_semantic_case, run_semantic_project,
    run_type_case, run_type_project,
};

fn observe(source: &str) -> String {
    run_semantic_case(source.as_bytes(), None).expect("Auto semantic stage driver")
}

fn canonical_source<'a>(corpus: &'a str, title: &str) -> &'a str {
    let heading = format!("## Case: {title}\n");
    corpus
        .split_once(&heading)
        .expect("canonical case")
        .1
        .split_once("```auto\n")
        .expect("source block")
        .1
        .split_once("```")
        .expect("source end")
        .0
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
fn verifies_global_accessors_and_effectful_named_argument_order() {
    let source = concat!(
        "var event_log int = 0\n",
        "fn A() int { event_log = event_log * 10 + 2\n return 37 }\n",
        "fn B() bool { event_log = event_log * 10 + 1\n return true }\n",
        "fn pick(a int, b bool) int { return a }\n",
        "fn main() int { let invoked = pick(b: B(), a: A())\n return event_log }\n",
    );
    let document = run_core_hir_case(source.as_bytes(), "verify").expect("Auto HIR");
    let bound = ace_core::descriptor::bind(ace_core::text_record_descriptor(), &document)
        .unwrap_or_else(|error| panic!("{error:?}\n{document}"));
    ace_core::verify::verify_text_record(bound).expect("reference verifies global accessor HIR");
    assert!(document.contains("module.global.get:g3_appm4_main_event_log"));
    assert!(document.contains("module.global.set:g3_appm4_main_event_log"));
    assert!(document.contains("__aac_init_g3_appm4_main_event_log"));
    assert!(document.contains("{ param: 1u32, arg: 0u32 }, { param: 0u32, arg: 1u32 }"));
    let rejection = run_core_hir_case(source.as_bytes(), "immutable-global")
        .expect("mutability is checked on the real draft");
    assert!(
        rejection.starts_with("ERROR hir.mutability 0 "),
        "{rejection}"
    );
}

#[test]
fn rechecks_global_effects_against_the_source_layout() {
    let source = "var count int = 40\nfn main() int { count = count + 2\n return count }";
    let artifact = run_core_hir_case(source.as_bytes(), "verify").expect("Auto HIR");
    let admit = |body: &str| run_global_hir_admission(source, body).expect("Auto HIR admission");
    assert_eq!(admit(&artifact), "ADMITTED text-record-i32-draft\n");
    let stolen = artifact.replace(
        "module.global.set:g3_appm4_main_count",
        "module.global.set:g3_appm4_main_other",
    );
    assert_ne!(stolen, artifact);
    assert_eq!(admit(&stolen), "ERROR hir.global-authority\n");
    let erased = artifact.replace("effect: observable", "effect: pure");
    assert_ne!(erased, artifact);
    assert_eq!(admit(&erased), "ERROR hir.global-authority\n");
    let changed_initializer =
        artifact.replace("__aac_init_g3_appm4_main_count", "incorrect_initializer");
    assert_ne!(changed_initializer, artifact);
    assert_eq!(admit(&changed_initializer), "ERROR hir.global-authority\n");
    let wrong_module = artifact.replace("path: \"app.main\"", "path: \"app.foreign\"");
    assert_ne!(wrong_module, artifact);
    assert_eq!(admit(&wrong_module), "ERROR hir.global-authority\n");
}

#[test]
fn verifies_borrowed_text_lifetime_and_scalar_length() {
    let source = concat!(
        "fn length(text str) int { return text.len() }\n",
        "fn main() int { let sample = \"A中😀\\n\"\n return length(sample) }\n",
    );
    let atom = run_core_hir_case(source.as_bytes(), "verify").expect("Auto HIR");
    let bound = ace_core::descriptor::bind(ace_core::text_record_descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify_text_record(bound).expect("reference verifies borrowed text HIR");
    assert!(
        atom.contains("requires: [\"ac.runtime.text.v1\"]"),
        "{atom}"
    );
    assert!(atom.contains("text: \"A中😀\\n\""), "{atom}");
    assert!(atom.contains("mode: view"), "{atom}");
    let admit = |candidate: &str| {
        run_core_hir_case(candidate.as_bytes(), "descriptor").expect("Auto HIR admission")
    };
    assert_eq!(admit(&atom), "ADMITTED text-record-i32-draft\n");
    let without_runtime = atom.replace(" requires: [\"ac.runtime.text.v1\"]\n", "");
    assert_eq!(admit(&without_runtime), "ERROR hir.unsupported\n");
    let moved_view = atom.replace("mode: view", "mode: move");
    assert_ne!(moved_view, atom);
    assert_eq!(admit(&moved_view), "ERROR hir.type\n");
    let bad_arity = source.replace("text.len()", "text.len(1)");
    let rejected = run_core_hir_case(bad_arity.as_bytes(), "verify").expect("Auto type check");
    assert!(rejected.starts_with("ERROR type.argument 0 "), "{rejected}");
}

#[test]
fn rejects_a_detached_expression_cycle() {
    let source = b"fn main() int { return 42 }";
    let document = run_core_hir_case(source, "verify").expect("Auto HIR");
    assert_eq!(
        run_core_hir_case(document.as_bytes(), "descriptor").expect("Auto HIR admission"),
        "ADMITTED core-i32-draft\n"
    );
    let candidate = document.replacen(
        " expressions {\n",
        concat!(
            " expressions {\n",
            " expr a binary { type: \"t1\"; op: add_i32; lhs: \"b\"; rhs: \"x\"; overflow: trap }\n",
            " expr b binary { type: \"t1\"; op: add_i32; lhs: \"a\"; rhs: \"y\"; overflow: trap }\n",
            " expr x constant { type: \"t1\"; value: 1i32 }\n",
            " expr y constant { type: \"t1\"; value: 2i32 }\n",
        ),
        1,
    );
    assert_ne!(candidate, document);
    assert_eq!(
        run_core_hir_case(candidate.as_bytes(), "descriptor").expect("Auto HIR admission"),
        "ERROR hir.reference\n"
    );
}

#[test]
fn verifies_nominal_record_construction_and_nested_field_reads() {
    let source = concat!(
        "type Pair { number int; enabled bool }\n",
        "type Container { inner Pair }\n",
        "fn fetch(boxed Container) int { return boxed.inner.number }\n",
        "fn main() int { return fetch(Container(Pair(enabled: true, number: 42))) }\n",
    );
    let atom = run_core_hir_case(source.as_bytes(), "verify").expect("Auto HIR");
    let bound = ace_core::descriptor::bind(ace_core::text_record_descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify_text_record(bound).expect("reference verifies nominal record HIR");
    assert!(
        atom.contains("ctor_bindings: [{ field: 1u32, arg: 0u32 }, { field: 0u32, arg: 1u32 }]")
    );
    assert_eq!(atom.matches(" field_get ").count(), 2);

    let admit = |candidate: &str| {
        run_core_hir_case(candidate.as_bytes(), "descriptor").expect("Auto HIR admission")
    };
    assert_eq!(admit(&atom), "ADMITTED text-record-i32-draft\n");
    let invalid_field = atom.replacen("path: [0u32]", "path: [7u32]", 1);
    assert_ne!(invalid_field, atom);
    assert_eq!(admit(&invalid_field), "ERROR hir.reference\n");
    let wrong_mapping = atom.replacen(
        "{ field: 1u32, arg: 0u32 }, { field: 0u32, arg: 1u32 }",
        "{ field: 0u32, arg: 0u32 }, { field: 1u32, arg: 1u32 }",
        1,
    );
    assert_eq!(admit(&wrong_mapping), "ERROR hir.type\n");
    let view = atom
        .lines()
        .find(|line| line.contains(" borrow_view "))
        .expect("record argument is borrowed");
    let borrowed = view.split_whitespace().nth(1).unwrap();
    let base = view
        .split_once("base: ")
        .unwrap()
        .1
        .split_whitespace()
        .next()
        .unwrap();
    let missing_view = atom.replacen(&format!("{view}\n"), "", 1).replacen(
        &format!("eval_args: [\"{borrowed}\"]"),
        &format!("eval_args: [{base}]"),
        1,
    );
    assert_eq!(admit(&missing_view), "ERROR hir.borrow\n");
}

#[test]
fn binds_variant_payloads_in_their_own_arm() {
    let source = concat!(
        "enum Packet { Data(int)\n Done }\n",
        "fn make_packet(value int) Packet { return Packet.Data(value) }\n",
        "fn main() int {\n",
        " let packet = make_packet(42)\n",
        " is packet {\n",
        "  Data(number) -> { return number }\n",
        "  Done -> { return 0 }\n",
        " }\n}\n",
    );
    let accepted = type_report(source);
    assert!(accepted.starts_with("aac.type-report/1\n"), "{accepted}");
    let unknown = source.replace("Data(number)", "Other(number)");
    let site = unknown.find("Other(number)").unwrap();
    assert_eq!(
        type_report(&unknown),
        format!("ERROR type.pattern 0 {site}..{}\n", site + 5)
    );
    let escaped = source.replace("Done -> { return 0 }", "Done -> { return number }");
    let site = escaped.rfind("number").unwrap();
    assert_eq!(
        type_report(&escaped),
        format!("ERROR bind.unresolved 0 {site}..{}\n", site + 6)
    );
    let corpus =
        include_str!("../../../../../crates/auto-lang/test/cases/language/types/variants.case.ad");
    let named = canonical_source(corpus, "named-enum-payload");
    let reordered = named.replace("code: 42, text: \"ok\"", "text: \"ok\", code: 42");
    let report = type_report(&reordered);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let slots: Vec<_> = report
        .lines()
        .filter(|line| line.starts_with("ARG "))
        .map(|line| line.split_whitespace().nth(3).unwrap())
        .collect();
    assert_eq!(slots, ["1", "0"]);
    let canonical = type_report(named);
    assert!(canonical.starts_with("aac.type-report/1\n"), "{canonical}");
    let bad_field = named.replace("code: 42", "code: false");
    let site = bad_field.find("false").unwrap();
    assert_eq!(
        type_report(&bad_field),
        format!("ERROR type.argument 0 {site}..{}\n", site + 5)
    );
    let tuple_arm = "enum Pair { Values(int, bool) }\nfn read(p Pair) int {\n is p { Values(count, enabled) -> { if enabled { return count }; return 0 } }\n}\nfn main() int { return read(Pair.Values(42, true)) }";
    let positional = type_report(tuple_arm);
    assert!(
        positional.starts_with("aac.type-report/1\n"),
        "{positional}"
    );
}

#[test]
fn distinguishes_fallthrough_statements_from_match_expressions() {
    let corpus = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/patterns/nested-match.case.ad"
    );
    let case = |title: &str| canonical_source(corpus, title);
    for title in ["statement-fallthrough", "guard-arm", "or-arm"] {
        let report = type_report(case(title));
        assert!(
            report.starts_with("aac.type-report/1\n"),
            "{title}: {report}"
        );
    }
    let nested = case("nested-result-option");
    let report = type_report(nested);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let type_rows: Vec<Vec<_>> = report
        .lines()
        .filter(|line| line.starts_with("TYPE "))
        .map(|line| line.split_whitespace().collect())
        .collect();
    let option = type_rows
        .iter()
        .find(|row| row[2..] == ["20", "1", "0"])
        .expect("Option<Int> type")[1];
    let result = type_rows
        .iter()
        .find(|row| row[2..] == ["21", option, "3"])
        .expect("Result<Option<Int>, Str> type")[1];
    for (expression, type_id) in [
        ("Ok(Some(42))", result),
        ("Some(42)", option),
        ("Err(\"bad\")", result),
    ] {
        let start = nested.find(expression).unwrap();
        let location = format!(" 0 {start}..{} {type_id}", start + expression.len());
        assert!(
            report
                .lines()
                .any(|line| line.starts_with("TY ") && line.ends_with(&location)),
            "{expression}: {report}"
        );
    }
    let invalid = case("nonexhaustive-expr-rejects");
    let start = invalid.find("is 1").unwrap();
    let stop = start + "is 1 { 1 -> 40 }".len();
    assert_eq!(
        type_report(invalid),
        format!("ERROR type.match-exhaustive 0 {start}..{stop}\n")
    );
    let incompatible = nested.replace("Err(\"bad\")", "Err(9)");
    let start = incompatible.find("Err(9)").unwrap();
    assert_eq!(
        type_report(&incompatible),
        format!("ERROR type.argument 0 {start}..{}\n", start + 6)
    );
}

#[test]
fn rejects_inline_recursive_layout_but_accepts_list_indirection() {
    let cycle = "enum Packet { Data(Packet) }\nfn main() int { return 0 }";
    let site = cycle.find("Data(Packet)").unwrap();
    assert_eq!(
        type_report(cycle),
        format!(
            "ERROR type.recursive-layout 0 {site}..{}\n",
            site + "Data(Packet)".len()
        )
    );
    let indirect = cycle.replace("Data(Packet)", "Data(List<Packet>)");
    let observed = type_report(&indirect);
    assert!(observed.starts_with("aac.type-report/1\n"), "{observed}");
    let trailing = cycle.replace("Data(Packet)", "Data(int, Packet)");
    let site = trailing.find(", Packet").unwrap() + 2;
    assert_eq!(
        type_report(&trailing),
        format!("ERROR type.recursive-layout 0 {site}..{}\n", site + 6)
    );
}

#[test]
fn checks_scalar_pattern_value_and_optional_payload_scope() {
    let literal = "fn main() int { let number = 0\n is number { -0 -> { return 7 }\n else -> { return 0 } } }";
    let accepted = type_report(literal);
    assert!(accepted.starts_with("aac.type-report/1\n"), "{accepted}");
    let duplicate = literal.replace("else ->", "0 -> { return 1 }\n else ->");
    let site = duplicate.find("0 -> { return 1 }").unwrap();
    assert_eq!(
        type_report(&duplicate),
        format!("ERROR type.pattern 0 {site}..{}\n", site + 1)
    );
    let optional = "fn unwrap_or_zero(value ?int) int {\n is value { Some(number) -> { return number }\n None -> { return 0 } }\n}";
    let accepted = type_report(optional);
    assert!(accepted.starts_with("aac.type-report/1\n"), "{accepted}");
}

#[test]
fn freezes_versioned_facts_bound_to_the_exact_source() {
    let initial = "fn answer() int { return 41 }\nfn main() int { return answer() }\n";
    let edited = initial.replace("41", "42");
    let freeze =
        |source: &str| run_core_hir_case(source.as_bytes(), "facts").expect("Auto semantic facts");
    let first = freeze(initial);
    assert_eq!(first, freeze(initial));
    for (source, frozen) in [(initial, first), (&edited, freeze(&edited))] {
        let expected = format!(
            "aac.semantic-facts/1\nSOURCE {} {}\nMODULE 0 app.main 0..{}\naac.type-report/1\n",
            source.len(),
            blake3::hash(source.as_bytes()),
            source.len()
        );
        assert!(frozen.starts_with(&expected), "{frozen}");
        assert!(frozen.contains("\naac.binding-report/1\n"));
        assert!(frozen.contains("\nUSE "));
    }
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
fn resolves_a_reexport_to_its_own_package_namespace() {
    let main = "use facade.result as total\nfn main() int { return total() }";
    let facade = "pub use support.lib.answer as result";
    let support = "pub fn answer() int { return 42 }";
    let local = "pub fn answer() int { return 99 }";
    let report = project(&[
        ("main", main),
        ("facade", facade),
        ("support/lib", support),
        ("lib", local),
    ]);
    let target = report
        .lines()
        .find(|line| line.starts_with("DECL ") && line.contains(" 1 answer 2 "))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap_or_else(|| panic!("{report}"));
    assert!(
        report.contains(&format!("USE {target} total 0 ")),
        "{report}"
    );
    let private = support.strip_prefix("pub ").unwrap();
    let report = project(&[
        ("main", main),
        ("facade", facade),
        ("support/lib", private),
        ("lib", local),
    ]);
    let start = facade.find("answer").unwrap();
    assert_eq!(
        report,
        format!("ERROR bind.private 1 {start}..{}\n", start + "answer".len())
    );
}

#[test]
fn verifies_the_autodown_cross_package_reexport() {
    let corpus = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/modules/imports-globals.case.ad"
    );
    let section = corpus
        .split_once("## Case: dependency-reexport-alias")
        .unwrap()
        .1;
    let case = section.split_once("## Case:").unwrap().0;
    let sources: Vec<_> = case
        .split("```auto")
        .skip(1)
        .map(|source| source.split_once("```").unwrap().0)
        .collect();
    assert_eq!(sources.len(), 3);
    let document = run_core_hir_project(&[
        ("main", sources[0]),
        ("facade", sources[1]),
        ("support/lib", sources[2]),
    ])
    .expect("Auto HIR project");
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), &document)
        .unwrap_or_else(|error| panic!("{error:?}\n{document}"));
    ace_core::verify::verify(bound).expect("reference verifies the cross-package call");
    assert!(
        document.contains("name: \"support.lib.answer\""),
        "{document}"
    );
    assert!(document.contains("value: 42i32"), "{document}");
    assert_eq!(
        run_core_hir_case(document.as_bytes(), "descriptor").expect("Auto descriptor admission"),
        "ADMITTED core-i32-draft\n"
    );
}

#[test]
fn reports_exact_global_layout_and_accesses_from_autodown_source() {
    let corpus = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/modules/imports-globals.case.ad"
    );
    let section = corpus.split_once("## Case: module-global-once").unwrap().1;
    let case = section.split_once("## Case:").unwrap().0;
    let sources: Vec<_> = case
        .split("```auto")
        .skip(1)
        .map(|source| source.split_once("```").unwrap().0)
        .collect();
    assert_eq!(sources.len(), 2);
    let report = run_type_project(&[("main", sources[0]), ("state", sources[1])])
        .expect("Auto global layout");
    assert!(report.contains("aac.globals/1\n"), "{report}");
    let declaration = report
        .lines()
        .find(|line| line.starts_with("GLOBAL g3_appm5_state_n "))
        .unwrap_or_else(|| panic!("{report}"));
    let layout: Vec<_> = declaration.split_whitespace().collect();
    let initial = sources[1].find("40").unwrap();
    assert_eq!(
        layout[3..],
        ["1", "1", "1", &format!("{initial}..{}", initial + 2)]
    );
    let accesses: Vec<_> = report
        .lines()
        .filter(|line| line.starts_with("GLOBAL-USE "))
        .map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            assert_eq!(fields[2], layout[2]);
            assert_eq!(fields[4], "1");
            fields[3]
        })
        .collect();
    assert_eq!(accesses, ["1", "0", "0"]);
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

#[test]
fn binds_methods_through_their_receiver_and_maps_formals() {
    let source = concat!(
        "type Gauge {\n",
        "    count int\n",
        "    fn choose(v int, enabled bool) int { return v }\n",
        "}\n",
        "type Switch {\n",
        "    fn choose(v bool, enabled bool) bool { return v }\n",
        "}\n",
        "fn read(g Gauge) int { return g.count }\n",
        "fn choose(g Gauge) int { return g.choose(enabled: true, v: g.count) }\n",
    );
    let report = type_report(source);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let declared: Vec<_> = report
        .lines()
        .filter(|line| line.contains(" 9 choose 0 "))
        .map(|line| line.split_whitespace().nth(1).unwrap())
        .collect();
    assert_eq!(declared.len(), 2);
    assert!(
        report.contains(&format!("USE {} choose 0 ", declared[0])),
        "{report}"
    );
    assert!(
        !report.contains(&format!("USE {} choose 0 ", declared[1])),
        "{report}"
    );
    let wrong = source.replace("enabled: true", "enabled: 9");
    let start = wrong.rfind('9').unwrap();
    assert_eq!(
        type_report(&wrong),
        format!("ERROR type.argument 0 {start}..{}\n", start + 1)
    );
    let corpus = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/calls/binding-pipeline.case.ad"
    );
    let pipe = canonical_source(corpus, "pipe-method-and-field");
    let pipe_report = type_report(pipe);
    assert!(
        pipe_report.starts_with("aac.type-report/1\n"),
        "{pipe_report}"
    );
    let add = pipe_report
        .lines()
        .find(|line| line.contains(" 9 add 0 "))
        .expect("method declaration")
        .split_whitespace()
        .nth(1)
        .unwrap();
    assert_eq!(pipe_report.matches(&format!("USE {add} add 0 ")).count(), 2);
    let wrong_pipe = pipe.replace(".add(3)", ".add(false)");
    let start = wrong_pipe.find("false").unwrap();
    assert_eq!(
        type_report(&wrong_pipe),
        format!("ERROR type.argument 0 {start}..{}\n", start + 5)
    );
}

const CORE_HIR_INPUT: &[u8] = b"fn A() int { return 41 }\n\
fn B() bool { return true }\n\
fn f(a int, b bool) int { return a }\n\
fn main() int { return f(b: B(), a: A()) }\n";

#[test]
fn emits_schema_bound_hir_from_auto_with_mapped_eval_order() {
    let atom = run_core_hir_case(CORE_HIR_INPUT, "verify").expect("Auto HIR driver");
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), &atom)
        .unwrap_or_else(|diagnostic| panic!("{diagnostic:?}\n{atom}"));
    let checked = ace_core::verify::verify(bound).expect("independent reference verifier");
    let module = &checked.bundle().modules[0];
    let (body, evaluation, bindings) = module
        .bodies
        .iter()
        .find_map(|body| {
            body.exprs.iter().find_map(|expr| match &expr.kind {
                ace_core::hir::ExprKind::Call {
                    eval_args,
                    bindings,
                    ..
                } if bindings.len() == 2 => Some((body, eval_args, bindings)),
                _ => None,
            })
        })
        .expect("two-argument call");
    let observed: Vec<_> = evaluation
        .iter()
        .map(|id| {
            let ace_core::hir::ExprKind::Call { callee, .. } = &body.exprs[id.0 as usize].kind
            else {
                panic!("expected an evaluated call")
            };
            let ace_core::hir::DefKind::Function { name, .. } =
                &module.defs[callee.0 as usize].kind
            else {
                panic!("expected a function definition")
            };
            name.as_str()
        })
        .collect();
    assert_eq!(observed, ["B", "A"]);
    assert_eq!((bindings[0].param, bindings[0].arg), (1, 0));
    assert_eq!((bindings[1].param, bindings[1].arg), (0, 1));
}

#[test]
fn rejects_a_wrong_owner_and_dangling_defref_in_real_draft() {
    let good = run_core_hir_case(CORE_HIR_INPUT, "verify").expect("Auto HIR driver");
    assert!(good.starts_with("hir("), "{good}");
    for (mode, expected) in [
        ("owner", "ERROR hir.owner "),
        ("defref", "ERROR hir.reference "),
    ] {
        let report = run_core_hir_case(CORE_HIR_INPUT, mode).expect("Auto HIR control");
        assert!(report.starts_with(expected), "{mode}: {report}");
    }
}

#[test]
fn rechecks_descriptor_shape_references_and_mapped_types() {
    let atom = run_core_hir_case(CORE_HIR_INPUT, "verify").expect("Auto HIR producer");
    let admit = |input: &str| {
        run_core_hir_case(input.as_bytes(), "descriptor").expect("Auto descriptor admission")
    };
    assert_eq!(admit(&atom), "ADMITTED core-i32-draft\n", "{atom}");
    let checked_bit = atom.replacen("hir(", "hir(checked: true, ", 1);
    let callee = atom
        .split_once("callee: \"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    let wrong_callee = atom.replacen(
        &format!("callee: \"{callee}\""),
        "callee: \"missing_definition\"",
        1,
    );
    let first = atom
        .split_once("owner: \"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    let wrong_owner = atom.replacen(
        &format!("owner: \"{first}\""),
        "owner: \"missing_owner\"",
        1,
    );
    let wrong_slot = atom.replace(
        "{ param: 1u32, arg: 0u32 }, { param: 0u32, arg: 1u32 }",
        "{ param: 0u32, arg: 0u32 }, { param: 1u32, arg: 1u32 }",
    );
    assert_eq!(admit(&checked_bit), "ERROR hir.shape\n");
    assert_eq!(admit(&wrong_callee), "ERROR hir.reference\n");
    assert_eq!(admit(&wrong_owner), "ERROR hir.owner\n");
    assert_ne!(wrong_slot, atom);
    assert_eq!(admit(&wrong_slot), "ERROR hir.type\n");
}

#[test]
fn checks_integer_width_and_radix_before_type_publication() {
    let corpus = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/numbers/literals-conversions.case.ad"
    );
    for title in ["wide-exact", "mixed-width-explicit", "overflow-is-not-wrap"] {
        let report = type_report(canonical_source(corpus, title));
        assert!(
            report.starts_with("aac.type-report/1\n"),
            "{title}: {report}"
        );
    }
    let shift = "fn main() u32 { return (1u32).shl(5u8) }";
    let written = shift.find("5u8").unwrap();
    assert_eq!(
        type_report(shift),
        format!("ERROR type.argument 0 {written}..{}\n", written + 3)
    );
    let valid = concat!(
        "fn min() i64 { return -9223372036854775808i64 }\n",
        "fn max() u64 { return 18446744073709551615u64 }\n",
        "fn hex() int { return 0x1f32 }\n",
        "fn bits() u8 { return 0b1111_1111u8 }\n",
    );
    let report = type_report(valid);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let bad = valid.replace("18446744073709551615u64", "18446744073709551616u64");
    let start = bad.find("18446744073709551616").unwrap();
    assert_eq!(
        type_report(&bad),
        format!(
            "ERROR type.literal-range 0 {start}..{}\n",
            start + "18446744073709551616u64".len()
        )
    );
}

#[test]
fn preserves_signed_i32_bits_and_radix_through_hir() {
    let source = b"fn hex() int { return 0x2Ai32 }\nfn minimum() int { return -2147483648i32 }";
    let document = run_core_hir_case(source, "verify").expect("Auto HIR");
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), &document)
        .unwrap_or_else(|error| panic!("{error:?}\n{document}"));
    ace_core::verify::verify(bound).expect("reference verifies both integer constants");
    assert!(document.contains("value: 42i32"), "{document}");
    assert!(document.contains("value: -2147483648i32"), "{document}");
    let admit = |draft: &str| {
        run_core_hir_case(draft.as_bytes(), "descriptor").expect("Auto HIR admission")
    };
    assert_eq!(admit(&document), "ADMITTED core-i32-draft\n");
    let outside = document.replace("-2147483648i32", "-184467440737095516160i32");
    assert_eq!(admit(&outside), "ERROR hir.type\n");
}

#[test]
fn rejects_empty_components_in_the_project_inventory() {
    assert!(observe_project_inventory("main")
        .expect("valid inventory")
        .starts_with("hir("));
    for inventory in ["", "main\n", "main\n\nextra", "/main", "main/"] {
        let report = observe_project_inventory(inventory).expect("malformed inventory diagnostic");
        assert_eq!(report, "ERROR source.module\n", "{inventory:?}");
    }
}

#[test]
fn checks_float_width_and_ties_at_the_overflow_boundary() {
    let f32_limit = "340282356779733661637539395458142568448";
    let f32_neighbor = "340282356779733661637539395458142568447";
    let lower = format!("{f32_neighbor}.999");
    let upper = format!("{f32_limit}.0");
    assert_eq!(lower.parse::<f32>().unwrap(), f32::MAX);
    assert!(upper.parse::<f32>().unwrap().is_infinite());
    let source = format!("fn read() float {{ return 000_{}f32 }}", lower);
    assert!(type_report(&source).starts_with("aac.type-report/1\n"));
    let source = format!("fn read() float {{ return {upper}f32 }}");
    let start = source.find(&upper).unwrap();
    assert_eq!(
        type_report(&source),
        format!(
            "ERROR type.literal-range 0 {start}..{}\n",
            start + upper.len() + 3
        )
    );
    let source = format!("fn read() double {{ return {upper}f64 }}");
    assert!(type_report(&source).starts_with("aac.type-report/1\n"));

    let huge = format!("1{}.0", "0".repeat(309));
    assert!(huge.parse::<f64>().unwrap().is_infinite());
    let source = format!("fn read() double {{ return {huge}f64 }}");
    let start = source.find(&huge).unwrap();
    assert_eq!(
        type_report(&source),
        format!(
            "ERROR type.literal-range 0 {start}..{}\n",
            start + huge.len() + 3
        )
    );
}

#[test]
fn publishes_target_width_float_bits_without_double_rounding() {
    let samples = [
        "0.0",
        "-0.0",
        "0.1",
        "16777217.0",
        "1.000000059604644775390625",
        "1.000000059604644775390626",
    ];
    let mut source = String::new();
    for (index, sample) in samples.iter().enumerate() {
        source.push_str(&format!(
            "fn sample{index}() float {{ return {sample}f32 }}\n"
        ));
    }
    let report = type_report(&source);
    let observed: Vec<_> = report
        .lines()
        .filter_map(|line| line.strip_prefix("FLOAT f32 "))
        .map(|line| line.split_whitespace().next().unwrap().to_owned())
        .collect();
    let expected: Vec<_> = samples
        .iter()
        .map(|sample| format!("{:08x}", sample.parse::<f32>().unwrap().to_bits()))
        .collect();
    assert_eq!(observed, expected, "{report}");
}

#[test]
fn preserves_a_double_subnormal_and_signed_underflow() {
    let subnormal = format!("0.{}5", "0".repeat(323));
    let underflow = format!("-0.{}1", "0".repeat(323));
    let expected = [1u64, 1u64 << 63];
    for (sample, bits) in [(subnormal, expected[0]), (underflow, expected[1])] {
        assert_eq!(sample.parse::<f64>().unwrap().to_bits(), bits);
        let source = format!("fn sample() double {{ return {sample}f64 }}");
        let report = type_report(&source);
        assert!(
            report.contains(&format!("FLOAT f64 {bits:016x} ")),
            "{report}"
        );
    }
}

#[test]
fn checks_text_concatenation_and_rejects_float_bit_operators() {
    let good = "fn join(a str, b Str) Str { return a + b }";
    assert!(type_report(good).starts_with("aac.type-report/1\n"));
    let bad = "fn join(a float, b float) float { return a & b }";
    let start = bad.find("a & b").unwrap();
    assert_eq!(
        type_report(bad),
        format!("ERROR type.operator 0 {start}..{}\n", start + 5)
    );
}

#[test]
fn stable_annotations_point_to_their_declaration_and_reject_a_field_owner() {
    let good = "#[api(method = \"GET\", path = \"/read\")]\npub fn read() int { return 1 }";
    let report = type_report(good);
    let annotation = report
        .lines()
        .find(|line| line.starts_with("ANNOTATION api "))
        .unwrap_or_else(|| panic!("{report}"));
    let spans: Vec<_> = annotation.split_whitespace().collect();
    let span = |column: &str| {
        let (start, end) = column.split_once("..").unwrap();
        &good[start.parse::<usize>().unwrap()..end.parse::<usize>().unwrap()]
    };
    assert!(span(spans[3]).starts_with("fn read"));
    assert!(span(spans[5]).starts_with("api(method"));
    let bad = "type Settings {\n #[api(method = \"GET\")] value int\n}";
    let start = bad.find("api").unwrap();
    assert_eq!(
        type_report(bad),
        format!("ERROR type.annotation-owner 0 {start}..{}\n", start + 3)
    );
}

#[test]
fn hands_off_unresolved_templates_ct_and_foreign_without_hir() {
    let source = concat!(
        "use.rs probe_host\n",
        "fn log2<const N>(n int) int { return n }\n",
        "fn main() int { let v = #{ log2<2>(21) }\n return v }\n",
    );
    let report = run_core_hir_case(source.as_bytes(), "verify").expect("Auto stage driver");
    assert!(report.starts_with("aac.pending-work/1\n"), "{report}");
    let pending: Vec<_> = report
        .lines()
        .filter_map(|line| line.strip_prefix("PENDING "))
        .map(|line| line.split_whitespace().next().unwrap())
        .collect();
    assert_eq!(pending, ["4", "1", "2"], "{report}");
}

#[test]
fn emits_core_arithmetic_traps_and_rejects_an_erased_overflow_policy() {
    let source = b"fn main() int { return 17 + 3 * 5 }";
    let atom = run_core_hir_case(source, "verify").expect("Auto HIR driver");
    let bundle = ace_core::descriptor::bind(ace_core::descriptor(), &atom).unwrap();
    ace_core::verify::verify(bundle).expect("reference verifies typed arithmetic");
    assert_eq!(atom.matches("overflow: trap").count(), 2, "{atom}");
    let corrupted = atom.replacen("overflow: trap; ", "", 1);
    let report = run_core_hir_case(corrupted.as_bytes(), "descriptor").expect("Auto admission");
    assert_eq!(report, "ERROR hir.unsupported\n");
}

#[test]
fn aac1055_admits_lazy_logic_and_rechecks_operand_types() {
    let source = include_bytes!(
        "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B04-shortcircuit.at"
    );
    let atom = run_core_hir_case(source, "verify").expect("Auto HIR production");
    let logical = atom
        .lines()
        .find(|line| line.contains(" logical {"))
        .unwrap_or_else(|| panic!("missing logical expression: {atom}"));
    assert!(logical.contains("op: and;"), "{logical}");
    assert!(atom.contains("op: div_i32;"), "{atom}");
    assert!(matches!(
        reference_scalar_outcome(&atom),
        ace_av_core::ExecOutcome::Success {
            value: ace_av_core::Value::I32(42),
            ..
        }
    ));
    let eager = std::str::from_utf8(source)
        .unwrap()
        .replacen("false &&", "true &&", 1);
    let eager_atom = run_core_hir_case(eager.as_bytes(), "verify").expect("Auto eager control HIR");
    assert!(matches!(
        reference_scalar_outcome(&eager_atom),
        ace_av_core::ExecOutcome::Trap {
            kind: ace_av_core::contract::TrapKind::DivByZero,
            ..
        }
    ));
    let read_back =
        |text: &str| run_core_hir_case(text.as_bytes(), "descriptor").expect("Auto Atom readback");
    assert_eq!(read_back(&atom), "ADMITTED scalar-i32-draft\n");

    let i32_id = atom
        .lines()
        .find(|line| line.contains(" constant { type: \"t1\""))
        .and_then(|line| line.split_whitespace().nth(1))
        .unwrap();
    let (prefix, _) = logical.split_once("rhs: ").unwrap();
    let wrong_lane = atom.replacen(logical, &format!("{prefix}rhs: \"{i32_id}\" }}"), 1);
    assert_eq!(read_back(&wrong_lane), "ERROR hir.type\n");

    let wrong_profile = atom
        .replace("auto.hir.scalar.draft", "auto.hir.core.draft")
        .replace("scalar-i32-draft", "core-i32-draft");
    assert_eq!(read_back(&wrong_profile), "ERROR hir.unsupported\n");
}

// This is an independent oracle for the Auto-produced HIR. The AAC MIR
// phase and bridge must never call this reference lowering as a service.
fn reference_scalar_outcome(atom: &str) -> ace_av_core::ExecOutcome {
    let bundle = ace_core::descriptor::bind(ace_core::scalar_descriptor(), atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    let checked = ace_core::verify::verify_scalar(bundle).expect("reference HIR verification");
    let entry = checked.bundle().modules[0]
        .defs
        .iter()
        .find(|def| def.name_for_diagnostic() == "main")
        .expect("main definition");
    let mir = ace_lowering::lower_checked(&checked, &entry.id_text, 0).expect("reference lowering");
    let bytes = ace_av_core::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::decode_abc(&bytes).unwrap();
    let loaded = ace_av_core::verify_abc(decoded, &mir.owner).expect("reference MIR admission");
    ace_av_core::execute(&loaded, &Default::default(), &Default::default()).0
}

#[test]
fn aac1055_auto_mir_runs_in_av_without_reference_lowering() {
    use ace_av_core::{ExecOutcome, Value};
    let cases: &[(&[u8], Value)] = &[
        (
            b"fn f(x int) int { return 2 * x + 1 }\nfn main() int { return f(3) }",
            Value::I32(7),
        ),
        (b"fn main() bool { return false && 1 / 0 > 0 }", Value::Bool(false)),
        (
            include_bytes!(
                "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-for-loops.at"
            ),
            Value::I32(0),
        ),
        (
            b"fn f(x i64) i64 { return x + 1i64 }\nfn main() i64 { return f(9223372036854775806i64) }",
            Value::I64(i64::MAX),
        ),
    ];
    for (source, expected) in cases {
        let bytes = run_base_mir_case(source).expect("Auto MIR and ABC production");
        let outcome = execute_auto_mir(&bytes);
        assert!(
            matches!(&outcome, ExecOutcome::Success { value, .. } if value == expected),
            "expected {expected:?} for {:?}, got {outcome:?}",
            String::from_utf8_lossy(source)
        );
    }
}

fn execute_auto_mir(bytes: &[u8]) -> ace_av_core::ExecOutcome {
    let decoded = ace_av_core::decode_abc(bytes).expect("AVAD shape and checksum");
    let loaded = ace_av_core::verify_abc(
        decoded,
        &ace_av_core::mir::OwnerScope {
            schema: ace_av_core::contract::MIR_DATA_SCHEMA.into(),
            module: "app.main".into(),
        },
    )
    .expect("independent MIR consumer admission");
    ace_av_core::execute(&loaded, &Default::default(), &Default::default()).0
}

#[test]
fn aac1055_auto_mir_constant_mutation_changes_the_av_result() {
    use ace_av_core::{ExecOutcome, Value};
    let source = b"fn f(x int) int { return 2 * x + 1 }\nfn main() int { return f(3) }";
    let original = run_base_mir_case(source).expect("original MIR");
    let (report, mutated) =
        run_base_mir_control(source, "mir-change-constant").expect("mutated MIR");
    let mutated = mutated.unwrap_or_else(|| panic!("control failed: {report}"));
    let result_is = |bytes: &[u8], expected| {
        matches!(execute_auto_mir(bytes), ExecOutcome::Success { value: Value::I32(actual), .. }
            if actual == expected)
    };
    assert!(result_is(&original, 7));
    assert!(result_is(&mutated, 8));
    assert!(!result_is(&mutated, 7));
}

#[test]
fn aac1055_unresolved_ct_does_not_publish_base_mir() {
    let (report, artifact) = run_base_mir_control(b"fn main() int { return #{ 1 } }", "mir")
        .expect("Auto admission of pending CT source");
    assert!(
        report.starts_with("aac.pending-work/1\nPENDING 2 "),
        "{report}"
    );
    assert!(artifact.is_none());
}

#[cfg(feature = "aac-ct-bootstrap")]
#[test]
fn aac1055_ct_executes_literal_arguments_and_lazy_bool_through_bridge() {
    use ace_aac_syntax_conformance::run_ct_bootstrap_case;
    use serde_json::json;

    for (source, lane, arguments, expected) in [
        (
            "fn f(x int) int { return 2 * x + 1 }",
            "int",
            "3",
            json!({"i32": 7}),
        ),
        (
            "fn f() bool { return false && 1 / 0 > 0 }",
            "bool",
            "",
            json!({"bool": false}),
        ),
    ] {
        let (report, outcome) = run_ct_bootstrap_case(source.as_bytes(), lane, arguments).unwrap();
        assert_eq!(report, "CT value\n");
        let outcome = outcome.expect("typed CT outcome");
        assert_eq!(outcome["status"], "value");
        assert_eq!(outcome["value"], expected);
        assert_eq!(outcome["facts"]["foreign_calls"], 0);
        assert_eq!(outcome["facts"]["live_text"], 0);
        assert_eq!(outcome["facts"]["read_set"], json!([]));
    }
}

#[cfg(feature = "aac-ct-bootstrap")]
#[test]
fn aac1055_ct_resource_failure_has_no_value_and_recovers() {
    use ace_aac_syntax_conformance::run_ct_bootstrap_case;
    use serde_json::json;

    let (report, outcome) =
        run_ct_bootstrap_case(b"fn f() int { for true { }\n return 7 }", "int", "").unwrap();
    assert_eq!(report, "CT failure\n");
    let outcome = outcome.expect("typed resource failure");
    assert_eq!(outcome["status"], "failure");
    assert_eq!(outcome["error"]["kind"], "resource");
    assert_eq!(outcome["error"]["code"], "av.resource.instr-steps");
    assert!(outcome.get("value").is_none());

    let input = b"fn f(x int) int { return 2 * x + 1 }";
    let (report, outcome) = run_ct_bootstrap_case(input, "int", "f(2)").unwrap();
    assert_eq!(report, "ERROR ct.entry\n");
    assert!(outcome.is_none());

    let (report, outcome) = run_ct_bootstrap_case(input, "int", "3").unwrap();
    assert_eq!(report, "CT value\n");
    assert_eq!(outcome.unwrap()["value"], json!({"i32": 7}));
}

#[test]
fn aac1055_local_mir_admission_rejects_mutations_without_publication() {
    let source = b"fn f(value int, enabled bool) int { if enabled { return value }\n return 0 }\nfn main() int { return f(enabled: true, value: 7) }";
    for (mode, expected) in [
        ("mir-defref", "ERROR mir.reference "),
        ("mir-type", "ERROR mir.type "),
        ("mir-cfg", "ERROR mir.reference "),
        ("mir-uninitialized", "ERROR mir.uninitialized "),
    ] {
        let (report, bytes) = run_base_mir_control(source, mode).expect("Auto MIR control");
        assert!(report.starts_with(expected), "{mode}: {report}");
        assert!(bytes.is_none(), "{mode} published unadmitted bytes");
    }
    assert!(!run_base_mir_case(source)
        .expect("valid recovery")
        .is_empty());
}

#[test]
fn aac1055_wide_integer_hir_preserves_width_and_extrema() {
    let source = concat!(
        "fn min() i64 { return -9223372036854775808i64 }\n",
        "fn max() u64 { return 18446744073709551615u64 }\n",
        "fn plus(x i64) i64 { return x + 1i64 }\n",
    );
    let atom = run_core_hir_case(source.as_bytes(), "verify").expect("Auto HIR production");
    for expected in [
        "value: -9223372036854775808i64",
        "value: 18446744073709551615u64",
        "op: add_i64; overflow: trap",
    ] {
        assert!(atom.contains(expected), "missing {expected}: {atom}");
    }
    let bound = ace_core::descriptor::bind(ace_core::numeric_descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify_closures(bound).expect("independent typed HIR verification");
    let admit = |text: &str| {
        run_core_hir_case(text.as_bytes(), "descriptor").expect("Auto numeric HIR readback")
    };
    assert_eq!(admit(&atom), "ADMITTED source-numeric-draft\n");
    let exceeds_u64 = atom.replacen("18446744073709551615u64", "18446744073709551616u64", 1);
    assert_eq!(admit(&exceeds_u64), "ERROR hir.type\n");
    let wrong_lane = atom.replacen("op: add_i64;", "op: add_u64;", 1);
    assert_eq!(admit(&wrong_lane), "ERROR hir.type\n");
}

#[test]
fn aac1055_float_hir_preserves_rounded_bits_and_signed_zero() {
    let source = concat!(
        "fn zero() f64 { return -0.0f64 }\n",
        "fn bump(x f32) f32 { return x + 1.000000059604644775390625f32 }\n",
    );
    let atom = run_core_hir_case(source.as_bytes(), "verify").expect("Auto float HIR");
    for expression in [
        "value: 9223372036854775808f64",
        "value: 1065353216f32",
        "op: add_f32",
    ] {
        assert!(atom.contains(expression), "missing {expression}: {atom}");
    }
    let bound = ace_core::descriptor::bind(ace_core::numeric_descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify_closures(bound).expect("independent typed HIR verification");
    let admit = |text: &str| {
        run_core_hir_case(text.as_bytes(), "descriptor").expect("Auto float Atom readback")
    };
    assert_eq!(admit(&atom), "ADMITTED source-numeric-draft\n");
    let oversized = atom.replacen("1065353216f32", "4294967296f32", 1);
    assert_eq!(admit(&oversized), "ERROR hir.type\n");
}

#[test]
fn aac1055_conditional_loop_preserves_scopes_and_continuations() {
    let source = include_bytes!(
        "../../../../../crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-for-loops.at"
    );
    let atom = run_core_hir_case(source, "verify").expect("Auto loop HIR production");
    assert_eq!(atom.matches("loop(id:").count(), 3, "{atom}");
    assert_eq!(atom.matches("break(target:").count(), 4, "{atom}");
    assert_eq!(atom.matches("continue(target:").count(), 1, "{atom}");
    assert!(matches!(
        reference_scalar_outcome(&atom),
        ace_av_core::ExecOutcome::Success {
            value: ace_av_core::Value::I32(0),
            ..
        }
    ));
    let admit = |text: &str| {
        run_core_hir_case(text.as_bytes(), "descriptor").expect("Auto loop HIR readback")
    };
    assert_eq!(admit(&atom), "ADMITTED scalar-i32-draft\n");
    let target = atom
        .split_once("continue(target: \"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    let outer = atom
        .split("loop(id: \"")
        .skip(1)
        .map(|tail| tail.split('"').next().unwrap())
        .find(|id| *id != target)
        .unwrap();
    let wrong_owner = atom.replacen(
        &format!("continue(target: \"{target}\""),
        &format!("continue(target: \"{outer}\""),
        1,
    );
    assert_eq!(admit(&wrong_owner), "ERROR hir.reference\n");
}

#[test]
fn verifies_terminating_branches_and_rejects_a_back_edge() {
    let source = b"fn choose(flag bool) int { if flag { return 13 } else { return 7 } }";
    let atom = run_core_hir_case(source, "verify").expect("Auto HIR driver");
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify(bound).expect("independent reference verifies both returns");
    assert_eq!(
        run_core_hir_case(atom.as_bytes(), "descriptor").expect("Auto HIR admission"),
        "ADMITTED core-i32-draft\n"
    );
    let field = |prefix: &str| {
        atom.split_once(prefix)
            .unwrap()
            .1
            .split('"')
            .next()
            .unwrap()
    };
    let root = field("entry: \"");
    let then = field("then: \"");
    assert_ne!(root, then);
    let corrupt = atom.replacen(
        &format!("then: \"{then}\""),
        &format!("then: \"{root}\""),
        1,
    );
    assert_eq!(
        run_core_hir_case(corrupt.as_bytes(), "descriptor").expect("Auto HIR admission"),
        "ERROR hir.reference\n"
    );
}

#[test]
fn rejects_unused_and_shared_evaluations_in_descriptor_input() {
    let source = b"fn compute() int { return 2 + 3 }";
    let atom = run_core_hir_case(source, "verify").expect("Auto HIR driver");
    let admit = |document: &str| {
        run_core_hir_case(document.as_bytes(), "descriptor").expect("Auto HIR admission")
    };
    assert_eq!(admit(&atom), "ADMITTED core-i32-draft\n");
    let extra = atom.replacen(
        " expressions {\n",
        " expressions {\n expr unused constant {type: \"t1\"; value: 4i32}\n",
        1,
    );
    assert_ne!(extra, atom);
    assert_eq!(admit(&extra), "ERROR hir.reference\n");
    let left = atom
        .split_once("lhs: \"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    let right = atom
        .split_once("rhs: \"")
        .unwrap()
        .1
        .split('"')
        .next()
        .unwrap();
    assert_ne!(left, right);
    let shared = atom.replacen(&format!("rhs: \"{right}\""), &format!("rhs: \"{left}\""), 1);
    assert_eq!(admit(&shared), "ERROR hir.reference\n");
}

#[test]
fn verifies_guarded_return_with_a_typed_continuation() {
    let source = concat!(
        "fn side(a int, flag bool) int {\n",
        " let earlier = a + 1\n",
        " if flag { return earlier }\n",
        " let fallback = earlier + 4\n",
        " return fallback\n",
        "}\n",
        "fn main() int { return side(flag: true, a: 10) + side(flag: false, a: 26) }\n",
    );
    let atom = run_core_hir_case(source.as_bytes(), "verify").expect("Auto HIR production");
    let draft = ace_core::descriptor::bind(ace_core::descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify(draft).expect("reference verifies both terminating paths");
    assert_eq!(
        run_core_hir_case(atom.as_bytes(), "descriptor").expect("Auto transport verification"),
        "ADMITTED core-i32-draft\n"
    );
    let dead = "fn main() int { return 1; return 2 }";
    let written = dead.find("return 2").unwrap();
    assert_eq!(
        run_core_hir_case(dead.as_bytes(), "verify").expect("dead statement diagnostic"),
        format!("ERROR hir.unsupported 0 {written}..{}\n", written + 8)
    );
}

#[test]
fn rejects_an_unreachable_terminating_block() {
    let source = b"fn compute() int { return 2 }";
    let atom = run_core_hir_case(source, "verify").expect("Auto HIR driver");
    let extra = atom
        .replacen(
            " expressions {\n",
            " expressions {\n expr orphan constant {type: \"t1\"; value: 4i32}\n",
            1,
        )
        .replacen(
            " blocks {\n",
            " blocks {\n block unused {return(value: \"orphan\") {}}\n",
            1,
        );
    assert_ne!(extra, atom);
    assert_eq!(
        run_core_hir_case(extra.as_bytes(), "descriptor").expect("Auto HIR admission"),
        "ERROR hir.reference\n"
    );
}

#[test]
fn verifies_local_reads_and_writes_in_statement_order() {
    let source = b"fn step(a int) int { var count = a\n count = count + 2\n let snapshot = count\n return snapshot }";
    let document = run_core_hir_case(source, "verify").expect("Auto HIR");
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), &document)
        .unwrap_or_else(|error| panic!("{error:?}\n{document}"));
    ace_core::verify::verify(bound)
        .expect("reference verifies local initialization and assignment");
    let admit =
        |text: &str| run_core_hir_case(text.as_bytes(), "descriptor").expect("Auto HIR admission");
    assert_eq!(admit(&document), "ADMITTED core-i32-draft\n");
    let immutable = document.replacen(
        "role: binding; mutable: true",
        "role: binding; mutable: false",
        1,
    );
    assert_ne!(immutable, document);
    assert_eq!(admit(&immutable), "ERROR hir.mutability\n");
    let init = document.find("let(local:").unwrap();
    let assign = document.find("assign(place:").unwrap();
    let end = assign + document[assign..].find(") {}").unwrap() + 4;
    let reordered = format!(
        "{}{} {}{}",
        &document[..init],
        &document[assign..end],
        &document[init..assign],
        &document[end..]
    );
    assert_eq!(admit(&reordered), "ERROR hir.initialization\n");
}

#[test]
fn verifies_an_ancestor_binding_in_both_terminating_branches() {
    let source = b"fn both(flag bool) int { let base_value = 1\n if flag { let inside = base_value + 2\n return inside } else { return base_value } }";
    let document = run_core_hir_case(source, "verify").expect("Auto HIR");
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), &document)
        .unwrap_or_else(|error| panic!("{error:?}\n{document}"));
    ace_core::verify::verify(bound).expect("reference verifies both branches");
    assert_eq!(
        run_core_hir_case(document.as_bytes(), "descriptor").expect("Auto HIR admission"),
        "ADMITTED core-i32-draft\n"
    );
}

#[test]
fn preserves_checked_artifact_on_source_resource_and_replace_failures() {
    let original =
        run_core_hir_case(b"fn compute() int { return 3 }", "verify").expect("Auto HIR baseline");
    let update = b"fn compute() int { return 9 }";
    let controls: &[(&[u8], Option<u32>, bool, &str)] = &[
        (
            b"fn compute() int { return true }",
            None,
            false,
            "ERROR type.return ",
        ),
        (update, Some(2), false, "ERROR resource.semantic "),
        (update, None, true, "ERROR os.replace\n"),
    ];
    for (source, steps, lock, expected) in controls {
        let (report, bytes, staged) =
            observe_hir_publication(source, original.as_bytes(), *steps, *lock).unwrap();
        assert!(report.starts_with(expected), "{report}");
        assert_eq!(bytes, original.as_bytes());
        assert!(!staged);
    }
    let (report, updated, staged) =
        observe_hir_publication(update, original.as_bytes(), None, false).unwrap();
    assert_eq!(report, "PUBLISHED core-i32-draft\n");
    assert_ne!(updated, original.as_bytes());
    assert!(!staged);
    let text = std::str::from_utf8(&updated).unwrap();
    let bound = ace_core::descriptor::bind(ace_core::descriptor(), text).unwrap();
    ace_core::verify::verify(bound).expect("replacement is valid HIR");
}

#[test]
fn preserves_tuple_order_and_checks_indirect_signature() {
    let source = concat!(
        "fn invoke(f fn(int) -> bool) bool { return f(7) }\n",
        "fn positive(x int) bool { return x > 0 }\n",
        "fn pair() (int,bool) { return (7, invoke(positive)) }\n",
    );
    let report = type_report(source);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    assert!(report
        .lines()
        .any(|line| line.starts_with("TYPE ") && line.split_whitespace().nth(2) == Some("27")));
    let bad = source.replace("f(7)", "f(false)");
    let from = bad.find("false").unwrap();
    assert_eq!(
        type_report(&bad),
        format!("ERROR type.argument 0 {from}..{}\n", from + 5)
    );
}

#[test]
fn preserves_nested_payload_types_through_public_fields_and_calls() {
    let source = concat!(
        "use payload.CargoBox\n",
        "fn transfer(x Future<Result<List<?int>,bool>>) Future<Result<List<?int>,bool>> { return x }\n",
        "fn load(boxed CargoBox) Future<Result<List<?int>,bool>> { return transfer(boxed.content) }\n",
    );
    let payload = "pub type CargoBox { content Future<Result<List<?int>,bool>> }";
    let run = |definition: &str| {
        run_type_project(&[("main", source), ("payload", definition)]).expect("Auto project typing")
    };
    let report = run(payload);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let incorrect = payload.replacen("List<?int>,bool", "List<?int>,int", 1);
    let start = source.find("boxed.content").unwrap();
    assert_eq!(
        run(&incorrect),
        format!(
            "ERROR type.argument 0 {start}..{}\n",
            start + "boxed.content".len()
        )
    );
}

#[test]
fn builds_a_list_of_nested_futures_and_rejects_distinct_payloads() {
    let source = "fn collect(a Future<Result<?int,bool>>, b Future<Result<?int,bool>>) \
                  List<Future<Result<?int,bool>>> { return [a, b] }";
    let report = type_report(source);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let bad = source.replace("b Future<Result<?int,bool>>", "b Future<Result<?int,int>>");
    let start = bad.find("[a, b]").unwrap();
    let report = type_report(&bad);
    assert_eq!(
        report,
        format!("ERROR type.value 0 {start}..{}\n", start + 6)
    );
}

#[test]
fn completes_implicit_signatures_and_rejects_a_conflicting_call() {
    let document = include_str!(
        "../../../../../crates/auto-lang/test/cases/language/calls/binding-pipeline.case.ad"
    );
    let section = document.split_once("## Case: untyped-old-form").unwrap().1;
    let source = section
        .split_once("```auto")
        .unwrap()
        .1
        .split_once("```")
        .unwrap()
        .0;
    let atom = run_core_hir_case(source.as_bytes(), "verify").expect("Auto source HIR");
    let bundle = ace_core::descriptor::bind(ace_core::descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    ace_core::verify::verify(bundle).expect("reference verifies inferred signatures");
    let bad = "fn same(a) { return a }\nfn main() int { return same(3) + same(false) }";
    let start = bad.find("false").unwrap();
    assert_eq!(
        type_report(bad),
        format!("ERROR type.argument 0 {start}..{}\n", start + 5)
    );
    let complete = canonical_source(document, "forward-decl");
    let report = type_report(complete);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let body = report
        .lines()
        .find(|line| line.contains(" 1 test 0 "))
        .expect("function body declaration")
        .split_whitespace()
        .nth(1)
        .unwrap();
    assert!(report.contains(&format!("USE {body} test 0 ")), "{report}");
    let missing = canonical_source(document, "prototype-no-body");
    let prototype = "fn test() int;";
    let start = missing.find(prototype).unwrap();
    assert_eq!(
        type_report(missing),
        format!(
            "ERROR bind.unresolved 0 {start}..{}\n",
            start + prototype.len()
        )
    );
    let seeded =
        "fn add(n int) int;\nfn add(value) { value + 2 }\nfn main() int { return add(40) }";
    let report = type_report(seeded);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let atom = run_core_hir_case(seeded.as_bytes(), "verify").expect("Auto prototype HIR");
    let draft = ace_core::descriptor::bind(ace_core::descriptor(), &atom)
        .unwrap_or_else(|error| panic!("{error:?}\n{atom}"));
    let checked = ace_core::verify::verify(draft).expect("reference verifies resolved prototype");
    assert_eq!(checked.bundle().modules[0].defs.len(), 2);
    let wrong_tail = seeded.replace("value + 2", "false");
    let start = wrong_tail.find("false").unwrap();
    assert_eq!(
        type_report(&wrong_tail),
        format!("ERROR type.return 0 {start}..{}\n", start + 5)
    );
}

#[test]
fn normalizes_forward_type_aliases_and_rejects_a_cycle() {
    let good = "type Count = Width\nfn pass(x Count) int { return x }\ntype Width = int";
    let report = type_report(good);
    assert!(report.starts_with("aac.type-report/1\n"), "{report}");
    let bad = "type Count = Width\ntype Width = Count\nfn pass(x Count) int { return x }";
    let start = bad.find("Width").unwrap();
    assert_eq!(
        type_report(bad),
        format!(
            "ERROR type.unresolved 0 {start}..{}\n",
            start + "Width".len()
        )
    );
}

// ---------------------------------------------------------------------------
// PLAN-1048: generic instantiation and instance identity (V-01..V-05).
// ---------------------------------------------------------------------------

#[test]
fn aac1048_instances_id_distinguishes_int_and_bool_instances() {
    use ace_aac_syntax_conformance::run_instance_case;
    use ace_source_query::ident::sq_hash;

    let source = concat!(
        "fn id<T>(x T) T { return x }\n",
        "fn main() int { return id(7) }\n",
    );
    let (report, sheet) = run_instance_case(source.as_bytes()).expect("Auto instance pass");
    // The template is stripped from the expanded program and the call is
    // rewritten to the concrete function (assembly spelling uses the
    // identity suffixes, so the Int instance is id_i32).
    assert!(!report.contains("<T>"), "template survived expansion: {report}");
    assert!(report.contains("id_i32"), "concrete call missing: {report}");
    assert!(
        report.lines().any(|l| l.starts_with("DECL") && l.contains("id_i32")),
        "concrete declaration missing: {report}"
    );
    let mut templates = 0;
    let mut instances = 0;
    for line in sheet.lines() {
        if line.starts_with("TEMPLATE ") { templates += 1 }
        if line.starts_with("INSTANCE ") { instances += 1 }
    }
    assert_eq!((templates, instances), (1, 1), "worksheet: {sheet}");
    // The logical identity is byte-compatible with the Rust reference.
    let logical = sq_hash(
        "ac935/instance-logical/v1",
        &[b"app.main", b"tpl", b"id"],
    );
    assert!(
        sheet.contains(&format!(" {logical} app.main id ")),
        "logical mismatch: {sheet}"
    );
    // A bool argument is a distinct instance with a distinct key.
    let (_, sheet2) = run_instance_case(b"fn id<T>(x T) T { return x }\nfn main() int { let b = id(true)\n return 7 }".as_slice()).expect("bool instance");
    let mut key1 = String::new();
    let mut key2 = String::new();
    for line in sheet.lines().chain(sheet2.lines()) {
        if let Some(rest) = line.strip_prefix("INSTANCE ") {
            let first = rest.split_whitespace().next().unwrap();
            if key1.is_empty() { key1 = first.to_string() } else if first != key1 && key2.is_empty() { key2 = first.to_string() }
        }
    }
    assert!(!key1.is_empty() && !key2.is_empty(), "expected two instance keys: {sheet} / {sheet2}");
    assert_ne!(key1, key2, "Int and Bool instances must not share a key");
}

#[test]
fn aac1048_instances_swap_and_arity_refuse_without_reuse() {
    use ace_aac_syntax_conformance::run_instance_case;

    // Two type parameters, swapped argument order: distinct keys, no reuse.
    let (report, sheet) = run_instance_case(
        b"fn pair<A, B>(a A, b B) A { return a }\nfn main() int { return pair(1, true) }".as_slice(),
    ).expect("two-param instance");
    assert!(report.contains("pair_i32_bool"), "concrete name: {report}");
    let (_, sheet2) = run_instance_case(
        b"fn pair<A, B>(a A, b B) A { return a }\nfn main() bool { return pair(true, 1) }".as_slice(),
    ).expect("swapped instance");
    let mut keys = std::collections::BTreeSet::new();
    for line in sheet.lines().chain(sheet2.lines()) {
        if let Some(rest) = line.strip_prefix("INSTANCE ") {
            keys.insert(rest.split_whitespace().next().unwrap().to_string());
        }
    }
    assert_eq!(keys.len(), 2, "swapped args must not reuse an instance: {sheet} / {sheet2}");
    // Missing const args are refused (instance.arity), never silently
    // resolved. The const parameter's declared type defaults to u32.
    let refused = run_instance_case(
        b"fn rep<const N>(x int) int { return x + 1 }\nfn main() int { return rep(21) }".as_slice(),
    );
    assert!(refused.is_err() || refused.unwrap().0.starts_with("ERROR instance.arity"), "const arity must refuse");
    // An explicit const argument materializes the value instance.
    let (_, sheet3) = run_instance_case(
        b"fn rep<const N>(x int) int { return x + N }\nfn main() int { return rep<2>(21) }".as_slice(),
    ).expect("const instance");
    assert!(
        sheet3.lines().any(|l| l.starts_with("INSTANCE ") && l.contains(" u32=2 ") && l.contains("rep_u32_2")),
        "const instance identity: {sheet3}"
    );
}

#[test]
fn aac1048_instances_cross_module_owner_identity() {
    use ace_aac_syntax_conformance::run_instance_project;

    // The same template name in two modules, each called from its own
    // module: two logicals, never merged.
    let (_, sheet) = run_instance_project(&[
        ("alpha", "fn id<T>(x T) T { return x }\nfn use_id() int { return id(3) }"),
        ("beta", "fn id<T>(x T) T { return x }\nfn use_id() bool { return id(true) }"),
    ]).expect("cross-module instances");
    let mut logicals = std::collections::BTreeSet::new();
    for line in sheet.lines() {
        if let Some(rest) = line.strip_prefix("TEMPLATE ") {
            logicals.insert(rest.split_whitespace().nth(1).unwrap().to_string());
        }
    }
    assert_eq!(logicals.len(), 2, "same-name templates in different owners must stay distinct: {sheet}");
    let mut instances = 0;
    for line in sheet.lines() {
        if line.starts_with("INSTANCE ") { instances += 1 }
    }
    assert_eq!(instances, 2, "each module's own call materializes its own instance: {sheet}");
}

#[test]
fn aac1048_meta_static_projection_is_in_process_and_never_av() {
    use ace_aac_syntax_conformance::run_meta_case;

    // The meta projection is a pure static report: no binding pipeline runs
    // (no TYPE/DECL lines) and no AV bridge is invoked (no CT outcome).
    let source = concat!(
        "fn id<T>(x T) T { return x }\n",
        "fn pick(a int, b bool) int { return a }\n",
        "type Point { x int\n y int }\n",
        "fn main() int { return 7 }\n",
    );
    let report = run_meta_case(source.as_bytes()).expect("static meta projection");
    assert!(report.starts_with("aac.meta-report/1\n"), "{report}");
    assert!(!report.contains("TYPE "), "meta must not run the type pipeline: {report}");
    assert!(report.contains("META-FN app.main pick (a:int, b:bool) -> int"), "{report}");
    assert!(report.contains("META-REC app.main Point { x:int, y:int }"), "{report}");
}

#[test]
fn aac1048_meta_resolves_required_labels_on_owners() {
    use ace_aac_syntax_conformance::run_meta_case;

    // `#[required(...)]` annotations resolve to their declaration owner with
    // the argument spellings; other labels are not reported.
    let source = concat!(
        "fn setup() int { return 1 }\n",
        "fn main() int { return 7 }\n",
        "#[required(host = \"x86_64\", mode = \"aac\")] fn prepare(x int) int { return x }\n",
    );
    let report = run_meta_case(source.as_bytes()).expect("required projection");
    assert!(report.contains("META-REQUIRED prepare host = \"x86_64\", mode = \"aac\""), "{report}");
    assert!(!report.contains("META-REQUIRED setup"), "unannotated owner leaked: {report}");
}

#[test]
fn aac1048_ct_read_set_projection_is_static_and_names_constants() {
    use ace_aac_syntax_conformance::run_ct_report_case;

    // The read-set projection is in-process: it names the module constants
    // the CT target reads and never touches the AV bridge.
    let source = concat!(
        "const K int = 3\n",
        "fn f(x int) int { return K + x }\n",
        "fn plain(x int) int { return x + 1 }\n",
        "fn main() int { return 7 }\n",
    );
    let report = run_ct_report_case(source.as_bytes(), "f").expect("CT read set");
    assert!(report.starts_with("aac.ct-report/1\n"), "{report}");
    assert!(report.contains("CT-READ K"), "{report}");
    assert!(!report.contains("CT-READ plain"), "{report}");

    let plain = run_ct_report_case(source.as_bytes(), "plain").expect("plain read set");
    assert!(plain.starts_with("aac.ct-report/1\n"), "{plain}");
    assert!(!plain.contains("CT-READ"), "plain reads nothing: {plain}");
}

#[cfg(feature = "aac-ct-bootstrap")]
#[test]
fn aac1048_ct_executes_instanced_generic_closure_through_bridge() {
    use ace_aac_syntax_conformance::run_instance_ct_case;
    use serde_json::json;

    // V-04 CT tail: the generic call is instantiated (id_i32) and the CT
    // closure executes through the 1055 client lane with the concrete
    // function as the target.
    let source = concat!(
        "fn id<T>(x T) T { return x }\n",
        "fn main() int { return id(7) }\n",
    );
    let (report, outcome) =
        run_instance_ct_case(source.as_bytes(), "id_i32", "int", "7", true).expect("instanced CT");
    assert_eq!(report, "CT value\n", "{report}");
    let outcome = outcome.expect("typed CT outcome");
    assert_eq!(outcome["status"], "value");
    assert_eq!(outcome["value"], json!({"i32": 7}));
}

#[cfg(feature = "aac-ct-bootstrap")]
#[test]
fn aac1048_ct_constant_change_recomputes_no_stale_freeze() {
    use ace_aac_syntax_conformance::run_instance_ct_case;
    use serde_json::json;

    // V-04/V-05: the CT closure binds local constant K; changing its value
    // must recompute (the frozen 3 is never reused by the new query).
    let k3 = concat!(
        "fn f(x int) int { let K = 3\n return K + x }\n",
        "fn main() int { return 7 }\n",
    );
    let k4 = k3.replace("let K = 3", "let K = 4");
    let (_, outcome3) = run_instance_ct_case(k3.as_bytes(), "f", "int", "4", false).expect("K=3");
    assert_eq!(outcome3.unwrap()["value"], json!({"i32": 7}));
    let (_, outcome4) = run_instance_ct_case(k4.as_bytes(), "f", "int", "4", false).expect("K=4");
    assert_eq!(outcome4.unwrap()["value"], json!({"i32": 8}));
}

// ---------------------------------------------------------------------------
// PLAN-1049: control effects and bounded resource audit (V-01..V-05).
// ---------------------------------------------------------------------------

#[test]
fn aac1049_control_lazy_sites_classify_trap_rhs() {
    use ace_aac_syntax_conformance::run_control_case;

    // V-04 first pair: the lazy && / || RHS is classified, never an
    // unconditional effect; the classification is pure and in-process.
    let source = concat!(
        "fn f(a bool, b int) bool { return false && 1 / 0 > 0 }\n",
        "fn g(a bool) bool { return true || a }\n",
        "fn main() int { return 7 }\n",
    );
    let report = run_control_case(source.as_bytes()).expect("control classification");
    assert!(report.starts_with("aac.control-report/1\n"), "{report}");
    assert!(
        report.contains("CTL-LAZY andand") && report.contains("RHS=trap"),
        "lazy && with trap RHS: {report}"
    );
    assert!(report.contains("CTL-LAZY oror"), "lazy || site: {report}");
}

#[test]
fn aac1049_resources_rejects_use_after_move_before_backend() {
    use ace_aac_syntax_conformance::run_resources_case;

    // V-02/V-04: reading a moved local is rejected with a named diagnostic;
    // the legal neighbour (no read after the transfer) proceeds.
    let moved = concat!(
        "fn f() int { let s = 5\n let t = s.move\n return s }\n",
        "fn main() int { return 7 }\n",
    );
    let rejected = run_resources_case(moved.as_bytes()).expect("audit runs");
    assert!(
        rejected.starts_with("ERROR resource.use-after-move s "),
        "named rejection expected: {rejected}"
    );

    let fine = concat!(
        "fn f() int { let s = 5\n return s }\n",
        "fn main() int { return 7 }\n",
    );
    let accepted = run_resources_case(fine.as_bytes()).expect("audit passes");
    assert!(accepted.starts_with("aac.type-report/1"), "{accepted}");
    assert!(!accepted.contains("use-after-move"), "{accepted}");
}
