//! Independent oracle at the real source -> shared MIR -> ABC -> AV seam.
use auto_av_probe::collections_864_support::compile_and_execute;

fn assert_42(source: &str) {
    let run = compile_and_execute(source, "d_main").expect("full collections chain");
    assert_eq!(run.value, 42);
    assert_eq!(run.live_after, 0);
    assert_eq!(run.allocs, run.frees);
    assert!(
        run.allocs >= 2,
        "list and owned text must actually allocate"
    );
}

#[test]
fn named_variant_in_owned_list_runs_and_releases() {
    assert_42(include_str!(
        "../../../ac-core/tests/fixtures/collections_889/list_variant.at"
    ));
}

#[test]
fn nested_forward_variant_cross_function_runs_and_releases() {
    assert_42(include_str!(
        "../../../ac-core/tests/fixtures/collections_889/nested_variant.at"
    ));
}

#[test]
fn optional_variant_in_owned_list_runs_and_releases() {
    assert_42(include_str!(
        "../../../ac-core/tests/fixtures/collections_889/optional_variant.at"
    ));
}
