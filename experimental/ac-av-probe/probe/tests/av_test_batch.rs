//! PLAN-888 bounded C feasibility: same real producer, no checksum oracle.
use auto_av_core::contract::{ExecOutcome, Value};

fn combined_source() -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../crates/auto-lang/test/cases/governance-887/src");
    let expressions: Vec<_> = ["arithmetic", "unary", "group"]
        .iter()
        .map(|name| std::fs::read_to_string(root.join(format!("{name}.at"))).unwrap())
        .collect();
    let source = format!("type Results {{ arithmetic int; unary int; group int }}\nfn main() Results {{\n return Results({}, {}, {})\n}}\n",
                         expressions[0], expressions[1], expressions[2]);
    for expression in &expressions {
        assert!(source.contains(expression));
    }
    source
}

#[test]
fn c_data_profile_keeps_its_scalar_entry_boundary() {
    let source = combined_source();
    let data_boundary =
        auto_av_probe::data_driver::request_data(None, "plan888-c", &source, "d_main", &[])
            .expect_err("data profile must retain its scalar entry boundary");
    assert!(
        data_boundary.contains("mir.entry-sig") && data_boundary.contains("records do not cross")
    );
}

#[test]
fn c_three_original_expressions_return_distinct_actuals() {
    let source = combined_source();
    // The already-delivered owned profile explicitly permits a record result.
    // It still runs the same compiler/MIR/ABC/AV chain once, without changing
    // either product's entry contract or the original A/B scalar obligations.
    let observed =
        auto_av_probe::owned_driver::request_owned(None, "plan888-c", &source, "d_main", &[]);
    match observed {
        Ok(result) => match result.av {
            Some(ExecOutcome::Success {
                value: Value::Record(record),
                ..
            }) => {
                let actuals: Vec<_> = record.fields.iter().map(Value::as_i32).collect();
                assert_eq!(actuals, vec![Some(3), Some(-6), Some(9)]);
                println!("C applicable: one request/compile/AV execution, three record actuals {actuals:?}");
            }
            other => panic!("C feasibility requires explicit result decision: {other:?}"),
        },
        Err(error) => panic!("C feasibility requires explicit unsupported decision: {error}"),
    }
}

#[cfg(feature = "test-case-batch")]
#[test]
fn c_observations_count_producer_calls_and_stop_before_vm_on_compile_rejection() {
    let (rejected, metrics) = auto_av_probe::test_observations::capture(|| {
        auto_av_probe::owned_driver::request_owned(
            None,
            "c-rejected",
            "fn main() int { return missing_name }",
            "d_main",
            &[],
        )
    })
    .unwrap();
    assert!(rejected.is_err());
    assert_eq!(metrics.calls.get("compile"), Some(&1));
    assert!(!metrics.calls.contains_key("vm"));
    let source = combined_source();
    let (accepted, metrics) = auto_av_probe::test_observations::capture(|| {
        auto_av_probe::owned_driver::request_owned(None, "c-accepted", &source, "d_main", &[])
    })
    .unwrap();
    assert!(accepted.is_ok());
    assert_eq!(metrics.calls.get("compile"), Some(&1));
    assert_eq!(metrics.calls.get("vm"), Some(&1));
    assert_eq!(metrics.spans_ms.len(), 6);
}
