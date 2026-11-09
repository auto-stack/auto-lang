//! F-941-10: a catalog edit reports current source coordinates, never its wrapper.
use ace_core::atom_text::Diagnostic;
use ace_core::body_work::capture_stage_attempts;
use ace_core::source::cst::{
    parse_closure_fragment_via_cst, parse_front, units::*, FrontBudget, FrontKind, FrontOutcome,
    FrontRequest, FRONT_CAPS_VERSION,
};
use ace_core::source::SourceProfile;
use std::rc::Rc;

const BUDGET: FrontBudget = FrontBudget::WORK_OWNED_64MIB;
const PROFILE: SourceProfile = SourceProfile::Closures;
const SRC: &str = "fn main() int {\n    let base int = 40\n    let f = (n int) => { return base + n }\n    return f(2)\n}\nfn helper() int { return 7 }\n";
const BAD: &str = "(n int) => { return base + n + }";
const GOOD: &str = "(n int) => { return base + n + 1 }";

fn cold(src: &str) -> UnitCatalog {
    UnitCatalog::build(src, PROFILE, BUDGET).expect("valid cold catalog")
}

fn restore(cat: UnitCatalog) -> UnitCatalog {
    let src = cat.src().to_string();
    let manifest = decode_manifest(&encode_manifest(&cat.to_manifest(FRONT_CAPS_VERSION))).unwrap();
    drop(cat);
    let (restored, work) = capture_stage_attempts(|| {
        UnitCatalog::from_restored(&src, &manifest, PROFILE, FRONT_CAPS_VERSION).unwrap()
    });
    assert_eq!(
        work.whole_total(),
        0,
        "restore must not produce a whole module"
    );
    assert!(work.produced_units().is_empty());
    assert_eq!(restored.meter, ProductionMeter::default());
    assert!(restored.fns.values().all(|f| f.item.is_none()));
    assert!(restored.lambdas.values().all(|l| l.body.is_none()));
    restored
}

fn fresh(src: &str) -> Vec<Diagnostic> {
    match parse_front(FrontRequest {
        src,
        profile: PROFILE,
        kind: FrontKind::WholeModule,
        budget: BUDGET,
    }) {
        FrontOutcome::Diag(ds) => ds,
        other => panic!("expected fresh syntax error, got {other:?}"),
    }
}

fn equal_diags(actual: &[Diagnostic], expected: &[Diagnostic]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        assert_eq!(actual.stage, expected.stage);
        assert_eq!(actual.code, expected.code);
        assert_eq!(actual.message, expected.message);
        assert_eq!(actual.span, expected.span, "current absolute UTF-8 bytes");
    }
}

fn failed_edit(cat: &mut UnitCatalog, ordinal: u32, text: &str, budget: FrontBudget) -> EditError {
    let src = cat.src().to_string();
    let manifest = encode_manifest(&cat.to_manifest(FRONT_CAPS_VERSION));
    let events = format!("{:?}", cat.events);
    let meter = cat.meter.clone();
    let owners: Vec<_> = cat
        .fns
        .iter()
        .map(|(k, v)| (k.clone(), v.item.clone()))
        .collect();
    let lambdas: Vec<_> = cat
        .lambdas
        .iter()
        .map(|(k, v)| (k.clone(), v.body.clone()))
        .collect();
    let (result, work) = capture_stage_attempts(|| cat.edit_lambda("main", ordinal, text, budget));
    let error = result.expect_err("invalid edit must fail");
    assert_eq!(
        work.whole_total(),
        0,
        "diagnostic mapping must not reparse whole source"
    );
    assert_eq!(cat.src(), src);
    assert_eq!(
        encode_manifest(&cat.to_manifest(FRONT_CAPS_VERSION)),
        manifest
    );
    assert_eq!(format!("{:?}", cat.events), events);
    assert_eq!(cat.meter.produced, meter.produced);
    assert_eq!(cat.meter.reused, meter.reused);
    assert_eq!(cat.meter.discarded, meter.discarded + 1);
    for (key, previous) in owners {
        match (previous, &cat.fns[&key].item) {
            (Some(a), Some(b)) => assert!(Rc::ptr_eq(&a, b)),
            (None, None) => {}
            _ => panic!("failed edit changed owner materialization"),
        }
    }
    for (key, previous) in lambdas {
        match (previous, &cat.lambdas[&key].body) {
            (Some(a), Some(b)) => assert!(Rc::ptr_eq(&a, b)),
            (None, None) => {}
            _ => panic!("failed edit changed lambda materialization"),
        }
    }
    error
}

fn compare_bad(cat: &mut UnitCatalog, ordinal: u32, bad: &str) {
    let span = cat.lambdas[&("main".to_string(), ordinal)].span;
    let attempted = format!(
        "{}{}{}",
        &cat.src()[..span.start],
        bad,
        &cat.src()[span.end..]
    );
    let expected = fresh(&attempted);
    let EditError::Diag(actual) = failed_edit(cat, ordinal, bad, BUDGET) else {
        panic!("source error must remain typed Diag")
    };
    equal_diags(&actual, &expected);
}

#[test]
fn cold_lambda_structural_positions_equal_fresh() {
    for prefix in [
        "",
        "// prefix drift: 中文\n\n",
        "fn before() int { return 9 }\n",
    ] {
        compare_bad(&mut cold(&format!("{prefix}{SRC}")), 0, BAD);
    }
}

#[test]
fn restored_first_lambda_structural_positions_equal_fresh() {
    for prefix in [
        "",
        "// prefix drift: 中文\n\n",
        "fn before() int { return 9 }\n",
    ] {
        compare_bad(&mut restore(cold(&format!("{prefix}{SRC}"))), 0, BAD);
    }
}

#[test]
fn lexical_error_positions_equal_fresh_in_cold_and_restored_catalogs() {
    for prefix in ["", "// 中文 prefix\n"] {
        for restored in [false, true] {
            let mut cat = cold(&format!("{prefix}{SRC}"));
            if restored {
                cat = restore(cat);
            }
            compare_bad(&mut cat, 0, "(n int) => { return n @ 1 }");
        }
    }
}

#[test]
fn first_failure_repair_then_restore_uses_shifted_current_position() {
    let src = SRC.replace(
        "    return f(2)",
        "    let g = (m int) => { return f(m) + 1 }\n    return g(2)",
    );
    let mut cat = restore(cold(&src));
    let old_sibling = cat.lambdas[&("main".into(), 1)].span;
    compare_bad(&mut cat, 0, BAD);
    let receipt = cat.edit_lambda("main", 0, GOOD, BUDGET).unwrap();
    assert_eq!(receipt.produced.len(), 1);
    assert_eq!(receipt.reused.len(), 1);
    assert!(cat.fns.values().all(|f| f.item.is_none()));
    assert!(cat.lambdas[&("main".into(), 1)].body.is_none());
    let shifted = cat.lambdas[&("main".into(), 1)].span;
    assert!(shifted.start > old_sibling.start);
    let fresh = cold(&receipt.new_src);
    for (name, unit) in &cat.fns {
        assert_eq!(unit.span, fresh.fns[name].span);
        assert_eq!(unit.content_sha, fresh.fns[name].content_sha);
    }
    for (key, unit) in &cat.lambdas {
        assert_eq!(unit.span, fresh.lambdas[key].span);
        assert_eq!(unit.content_sha, fresh.lambdas[key].content_sha);
    }
    let mut twice = restore(cat);
    assert_eq!(twice.lambdas[&("main".into(), 1)].span, shifted);
    // A lexical fault selects the same primary diagnostic in both parsers;
    // syntax recovery in a multi-lambda whole file can select an earlier site.
    compare_bad(&mut twice, 1, "(m int) => { return f(m) @ 1 }");
}

#[test]
fn tiny_budget_keeps_resource_classification_and_old_catalog() {
    for restored in [false, true] {
        let mut cat = cold(SRC);
        if restored {
            cat = restore(cat);
        }
        let tiny = FrontBudget {
            work_bytes: 1,
            max_parse_depth: 1 << 20,
        };
        assert!(matches!(
            failed_edit(&mut cat, 0, GOOD, tiny),
            EditError::ResourceExhausted { .. }
        ));
    }
}

#[test]
fn wrapper_suffix_diagnostic_is_bounded_by_current_empty_fragment() {
    let mut cat = restore(cold(SRC));
    let start = cat.lambdas[&("main".into(), 0)].span.start;
    let EditError::Diag(ds) = failed_edit(&mut cat, 0, "", BUDGET) else {
        panic!("expected syntax diagnostic")
    };
    assert!(!ds.is_empty());
    for d in ds {
        assert_eq!((d.span.start, d.span.end), (start, start));
    }
}

#[test]
fn non_lambda_expression_rejection_uses_current_fragment_range() {
    let mut cat = restore(cold(&format!("// 中文 prefix\n{SRC}")));
    let start = cat.lambdas[&("main".into(), 0)].span.start;
    let EditError::Diag(ds) = failed_edit(&mut cat, 0, "1", BUDGET) else {
        panic!("expected fragment shape diagnostic")
    };
    assert_eq!(ds.len(), 1);
    assert_eq!(ds[0].code, "source.incremental-fragment");
    assert_eq!((ds[0].span.start, ds[0].span.end), (start, start + 1));
}

#[test]
fn adjacent_owner_and_public_closure_fragment_keep_absolute_positions() {
    let src = format!("// 中文 prefix\n{SRC}");
    let mut cat = restore(cold(&src));
    let span = cat.lambdas[&("main".into(), 0)].span;
    let owner = cat.fns["main"].span;
    let attempted = format!("{}{}{}", &src[..span.start], BAD, &src[span.end..]);
    let expected = fresh(&attempted);
    let bad_owner = format!(
        "{}{}{}",
        &src[owner.start..span.start],
        BAD,
        &src[span.end..owner.end]
    );
    let EditError::Diag(ds) = cat.edit_owner("main", &bad_owner, BUDGET).unwrap_err() else {
        panic!("owner diagnostic")
    };
    equal_diags(&ds, &expected);
    let Err(EditError::Diag(ds)) = cat.stage_owner_edit("main", &bad_owner, BUDGET) else {
        panic!("staged owner diagnostic")
    };
    equal_diags(&ds, &expected);
    let d = parse_closure_fragment_via_cst(&attempted, span.start..span.start + BAD.len(), PROFILE)
        .unwrap_err();
    equal_diags(&[d], &expected);
}

#[test]
fn public_closures_frontend_keeps_absolute_diagnostic_and_old_state() {
    use ace_core::source::closures_fn::ClosuresFunctionFrontend;
    let src = format!("// 中文 prefix\n{SRC}");
    let (mut frontend, _parts, _checked) =
        ClosuresFunctionFrontend::cold(&src, "f10.at", &[]).unwrap();
    let lambda = frontend
        .units()
        .iter()
        .find(|u| u.name == "__closure_main_0")
        .unwrap()
        .span;
    let sha = frontend.source_sha().to_string();
    let count = frontend.registry().lambda_count();
    let attempted = format!("{}{}{}", &src[..lambda.start], BAD, &src[lambda.end..]);
    let Err(ds) = frontend.edit_body(&sha, lambda.start..lambda.end, BAD) else {
        panic!("invalid public frontend edit must return diagnostics")
    };
    equal_diags(&ds, &fresh(&attempted));
    assert_eq!(frontend.source(), src);
    assert_eq!(frontend.source_sha(), sha);
    assert_eq!(frontend.registry().lambda_count(), count);
}
