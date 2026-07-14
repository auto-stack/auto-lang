use crate::compare::ComparisonReport;
use crate::tap::TapResult;

/// Generate a human-readable text report from a comparison.
pub fn format_report(report: &ComparisonReport) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "=== Parity Report: {} ===\n\n",
        report.library
    ));

    let consistent = report.consistent_count();
    let total = report.total_count();
    let rate = report.consistency_rate();

    out.push_str(&format!(
        "Consistency: {}/{} ({:.1}%)\n\n",
        consistent, total, rate
    ));

    let divergences = report.divergences();
    if divergences.is_empty() {
        out.push_str("All test cases consistent across three backends. \u{2713}\n");
    } else {
        out.push_str(&format!("Divergences ({}):\n", divergences.len()));
        out.push_str(&format!(
            "{:<30} {:<20} {:<10} {:<10} {:<10}\n",
            "Test", "Classification", "VM", "a2r", "Rust"
        ));
        out.push_str(&"-".repeat(80));
        out.push('\n');

        for case in divergences {
            let class = case.classify();
            out.push_str(&format!(
                "{:<30} {:<20} {:<10} {:<10} {:<10}\n",
                truncate(&case.name, 28),
                class.label(),
                fmt_pass(&case.vm),
                fmt_pass(&case.a2r),
                fmt_pass(&case.rust),
            ));

            // Show diagnostics for failing backends.
            for (backend, result) in [
                ("VM", &case.vm),
                ("a2r", &case.a2r),
                ("Rust", &case.rust),
            ] {
                if let Some(r) = result {
                    if let Some(diag) = &r.diagnostics {
                        out.push_str(&format!("    {} diag: {}\n", backend, diag));
                    }
                }
            }
        }
    }

    out
}

fn fmt_pass(result: &Option<TapResult>) -> &'static str {
    match result {
        Some(r) if r.passed => "pass",
        Some(_) => "FAIL",
        None => "missing",
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compare::{ComparisonReport, TestCaseComparison};
    use crate::tap::TapResult;

    fn pass(name: &str) -> TapResult {
        TapResult {
            passed: true,
            number: 1,
            name: name.to_string(),
            diagnostics: None,
        }
    }

    fn fail(name: &str, diag: &str) -> TapResult {
        TapResult {
            passed: false,
            number: 1,
            name: name.to_string(),
            diagnostics: Some(diag.to_string()),
        }
    }

    #[test]
    fn test_report_all_consistent() {
        let report = ComparisonReport {
            library: "dummy".to_string(),
            cases: vec![TestCaseComparison {
                name: "test_add".to_string(),
                vm: Some(pass("test_add")),
                a2r: Some(pass("test_add")),
                rust: Some(pass("test_add")),
            }],
        };
        let text = format_report(&report);
        assert!(text.contains("Parity Report: dummy"));
        assert!(text.contains("Consistency: 1/1 (100.0%)"));
        assert!(text.contains("All test cases consistent"));
    }

    #[test]
    fn test_report_with_divergence() {
        let report = ComparisonReport {
            library: "dummy".to_string(),
            cases: vec![
                TestCaseComparison {
                    name: "test_ok".to_string(),
                    vm: Some(pass("test_ok")),
                    a2r: Some(pass("test_ok")),
                    rust: Some(pass("test_ok")),
                },
                TestCaseComparison {
                    name: "test_bad".to_string(),
                    vm: Some(fail("test_bad", "got 4 expected 3")),
                    a2r: Some(pass("test_bad")),
                    rust: Some(pass("test_bad")),
                },
            ],
        };
        let text = format_report(&report);
        assert!(text.contains("Consistency: 1/2 (50.0%)"));
        assert!(text.contains("Divergences (1)"));
        assert!(text.contains("AutoVM bug"));
        assert!(text.contains("got 4 expected 3"));
        assert!(text.contains("test_bad"));
    }

    #[test]
    fn test_truncate() {
        // Strings at or under the limit are returned unchanged.
        assert_eq!(truncate("short", 10), "short");
        // Exactly 28 chars -> unchanged.
        assert_eq!(truncate("aaaaaaaaaaaaaaaaaaaaaaaaaaaa", 28), "aaaaaaaaaaaaaaaaaaaaaaaaaaaa");
        // Over the limit -> truncated with "..." suffix (length 28).
        assert_eq!(truncate("a_very_long_test_case_name_here", 10), "a_very_...");
        assert_eq!(truncate("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", 28), "aaaaaaaaaaaaaaaaaaaaaaaaa...");
    }

    #[test]
    fn test_fmt_pass() {
        assert_eq!(fmt_pass(&Some(pass("x"))), "pass");
        assert_eq!(fmt_pass(&Some(fail("x", "d"))), "FAIL");
        assert_eq!(fmt_pass(&None), "missing");
    }
}
