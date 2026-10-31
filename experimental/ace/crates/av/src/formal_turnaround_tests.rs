//! Structural/correctness counter-controls; public formal_828 API retained.
use crate::formal_828::{validate_and_gate, FormalReport, FormalSample, StructuralSnapshot};

#[test]
fn ace_gate_preserves_correctness_and_structure() {
    let mut report = FormalReport {
        samples: (0..10)
            .map(|n| FormalSample {
                revision: n,
                edit_to_result_ms: 2000.0,
                expected: 0,
                structural: StructuralSnapshot {
                    unaffected_stage_total: 0,
                    whole_module_total: 0,
                    install_set_len: 1,
                    reparsed_decls: 1,
                    reused_decls: 0,
                },
            })
            .collect(),
        p50_ms: 2000.0,
        p90_ms: 2000.0,
        p95_ms: 2000.0,
        p99_ms: 2000.0,
        all_correct: true,
        peak_slot_bytes: 0,
    };
    report.samples[9].edit_to_result_ms = 2001.0;
    assert!(
        validate_and_gate(&report, 2000.0, 10).is_empty(),
        "one outlier permitted"
    );
    report.samples[8].edit_to_result_ms = 2001.0;
    assert!(
        !validate_and_gate(&report, 2000.0, 10).is_empty(),
        "two outliers rejected even with forged p90 field"
    );
    report.samples[8].edit_to_result_ms = 2000.0;
    report.samples[0].structural.whole_module_total = 1;
    assert!(
        !validate_and_gate(&report, 2000.0, 10).is_empty(),
        "structure remains mandatory"
    );
    report.samples[0].structural.whole_module_total = 0;
    report.all_correct = false;
    assert!(
        !validate_and_gate(&report, 2000.0, 10).is_empty(),
        "correctness remains mandatory"
    );
    report.all_correct = true;
    report.samples[0].edit_to_result_ms = f64::NAN;
    assert!(
        !validate_and_gate(&report, 2000.0, 10).is_empty(),
        "invalid observations cannot pass"
    );
}
