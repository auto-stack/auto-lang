use ace_core::atom_text::Stage;
use ace_core::source::ast::{ExprKind, Item, Stmt};
use ace_core::source::{compile_language_source, parse_program_with, SourceProfile};

#[test]
fn projects_a_slice_with_nested_colons_and_an_omitted_bound() {
    let source = "fn main() int { return row[f(n: 0): :2] }";
    let program = parse_program_with(source, SourceProfile::Auto).expect("Auto grammar");
    let [Item::Fn(main)] = program.items.as_slice() else {
        panic!("expected a function");
    };
    let [Stmt::Return { value, .. }] = main.body.stmts.as_slice() else {
        panic!("expected a return expression");
    };
    let ExprKind::PySlice { base, bounds } = &value.kind else {
        panic!("expected a projected slice");
    };
    assert_eq!(
        &source[value.span.start..value.span.end],
        "row[f(n: 0): :2]"
    );
    assert_eq!(&source[base.span.start..base.span.end], "row");
    let start = bounds[0].as_ref().expect("start");
    let step = bounds[2].as_ref().expect("step");
    assert_eq!(&source[start.span.start..start.span.end], "f(n: 0)");
    assert!(bounds[1].is_none());
    assert_eq!(&source[step.span.start..step.span.end], "2");
    assert!(parse_program_with(source, SourceProfile::Aggregates).is_err());
}

#[test]
fn rejects_malformed_or_multiaxis_slice_at_the_source() {
    for key in ["[:::]", "[:,1]"] {
        let src = format!("fn main() int {{ return row{key} }}");
        let diag = parse_program_with(&src, SourceProfile::Auto).unwrap_err();
        assert_eq!(diag.stage, Stage::Text);
        let colon = src.find(key).expect("subscript") + 1;
        assert_eq!(diag.span.start..diag.span.end, colon..colon + 1, "{diag:?}");
    }
}

#[test]
fn distinguishes_write_rejection_from_receiver_rejection() {
    let src = "fn main() int { row[0][:] += 1; return 0 }";
    let diag = parse_program_with(src, SourceProfile::Auto).unwrap_err();
    assert_eq!(
        (diag.stage, diag.code.as_str()),
        (Stage::Text, "source.py-slice-write")
    );
    assert_eq!(&src[diag.span.start..diag.span.end], "[:]");

    let caps = ["ac.source.language.av/v1".to_string()];
    for (binding, base) in [(r#"var s = "auto""#, "s"), ("var xs = [1,2]", "xs")] {
        let src = format!("fn main() int {{ {binding}; return {base}[:] }}");
        let Err(diags) = compile_language_source(&src, "main.as", &caps) else {
            panic!("accepted a slice on a non-Python receiver");
        };
        let diag = diags
            .iter()
            .find(|d| d.code == "source.py-slice-receiver")
            .unwrap_or_else(|| panic!("missing checked receiver diagnostic: {diags:?}"));
        assert_eq!(diag.stage, Stage::Bind);
        assert_eq!(&src[diag.span.start..diag.span.end], base);
    }
}

#[test]
fn the_package_face_preserves_single_module_python_admission() {
    use ace_core::source::adapter::resolve_auto_package;
    use ace_core::source::cst::FrontBudget;
    use ace_core::source::packages::{bind_packages_auto, PackageModuleInput, PACKAGES_CAPABILITY};

    let caps = [
        "ac.source.language.av/v1".to_string(),
        PACKAGES_CAPABILITY.to_string(),
    ];
    let mut inputs = [PackageModuleInput {
        package: "auto".into(),
        version: "0".into(),
        id: "main".into(),
        source: "fn main() int { var row = [1, 2]; return row[:] }".into(),
        provider: None,
    }];
    let budget = FrontBudget::WORK_OWNED_64MIB;
    let bound = bind_packages_auto(&inputs, "auto::main", &caps, budget)
        .expect("the package grammar admits the subscript shape");
    let Err(diags) = resolve_auto_package(&bound.program, &caps) else {
        panic!("the package resolved a slice on an Auto collection");
    };
    assert!(
        diags.iter().any(|d| d.code == "source.py-slice-receiver"),
        "{diags:?}"
    );

    inputs[0].source =
        "use.py numpy: arange\nfn main() int { var row = arange(2); return row[:] }".into();
    let Err(diag) = bind_packages_auto(&inputs, "auto::main", &caps, budget) else {
        panic!("the package admitted a single-module Python import");
    };
    assert_eq!(diag.diagnostic.code, "source.use-py.surface");
}
