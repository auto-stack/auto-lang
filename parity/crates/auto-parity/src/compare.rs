use crate::tap::TapResult;

/// Which backend produced this result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Backend {
    Vm,
    A2r,
    Rust,
}

impl Backend {
    pub fn name(&self) -> &'static str {
        match self {
            Backend::Vm => "AutoVM",
            Backend::A2r => "a2r",
            Backend::Rust => "Rust",
        }
    }
}

/// Bug source classification for a divergent test case.
/// Mirrors design spec section 2.2.5 (bug classification table).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BugSource {
    /// All three backends agree — test passes.
    Consistent,
    /// VM and a2r agree, Rust differs -> replication bug.
    ReplicationBug,
    /// VM passes, a2r fails, Rust passes -> a2r transpiler bug.
    A2rBug,
    /// VM fails, a2r passes, Rust passes -> AutoVM bug.
    VmBug,
    /// All three fail differently, or a backend is missing -> needs manual review.
    TestCaseIssue,
}

impl BugSource {
    pub fn label(&self) -> &'static str {
        match self {
            BugSource::Consistent => "consistent",
            BugSource::ReplicationBug => "replication bug",
            BugSource::A2rBug => "a2r transpiler bug",
            BugSource::VmBug => "AutoVM bug",
            BugSource::TestCaseIssue => "test case issue",
        }
    }
}

/// A single test case's results across all three backends.
#[derive(Debug, Clone)]
pub struct TestCaseComparison {
    pub name: String,
    pub vm: Option<TapResult>,
    pub a2r: Option<TapResult>,
    pub rust: Option<TapResult>,
}

impl TestCaseComparison {
    /// Classify the bug source based on the three-way comparison.
    ///
    /// Per the plan's classification table:
    /// - All pass -> Consistent
    /// - VM+a2r pass, Rust fails -> ReplicationBug
    /// - VM passes, a2r fails, Rust passes -> A2rBug
    /// - VM fails, a2r passes, Rust passes -> VmBug
    /// - VM and a2r agree (same pass/fail) but Rust differs -> ReplicationBug
    /// - Otherwise -> TestCaseIssue
    pub fn classify(&self) -> BugSource {
        let vm_pass = self.vm.as_ref().map(|r| r.passed);
        let a2r_pass = self.a2r.as_ref().map(|r| r.passed);
        let rust_pass = self.rust.as_ref().map(|r| r.passed);

        match (vm_pass, a2r_pass, rust_pass) {
            // All present and agree.
            (Some(true), Some(true), Some(true)) => BugSource::Consistent,
            // VM + a2r agree that the test passes, but Rust disagrees.
            (Some(true), Some(true), Some(false)) => BugSource::ReplicationBug,
            // a2r is the odd one out.
            (Some(true), Some(false), Some(true)) => BugSource::A2rBug,
            // VM is the odd one out.
            (Some(false), Some(true), Some(true)) => BugSource::VmBug,
            // VM and a2r agree (both pass or both fail) but Rust differs from them.
            // Per design spec §2.2.5: "VM 和 a2r 一致地错，但与原始库不一致" → ReplicationBug.
            (Some(a), Some(b), Some(c)) if a == b && a != c => BugSource::ReplicationBug,
            // Everything else (missing backends, total disagreement) -> manual review.
            _ => BugSource::TestCaseIssue,
        }
    }
}

/// Overall comparison result for a library.
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub library: String,
    pub cases: Vec<TestCaseComparison>,
}

impl ComparisonReport {
    pub fn consistent_count(&self) -> usize {
        self.cases
            .iter()
            .filter(|c| c.classify() == BugSource::Consistent)
            .count()
    }

    pub fn total_count(&self) -> usize {
        self.cases.len()
    }

    /// Three-way consistency rate as a percentage:
    /// (three-way consistent cases / total cases) x 100.
    /// An empty report is treated as fully consistent (100.0).
    pub fn consistency_rate(&self) -> f64 {
        if self.cases.is_empty() {
            return 100.0;
        }
        let consistent = self.consistent_count() as f64;
        let total = self.total_count() as f64;
        (consistent / total) * 100.0
    }

    pub fn divergences(&self) -> Vec<&TestCaseComparison> {
        self.cases
            .iter()
            .filter(|c| c.classify() != BugSource::Consistent)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pass(name: &str) -> TapResult {
        TapResult {
            passed: true,
            number: 1,
            name: name.to_string(),
            diagnostics: None,
        }
    }

    fn fail(name: &str) -> TapResult {
        TapResult {
            passed: false,
            number: 1,
            name: name.to_string(),
            diagnostics: Some("mismatch".to_string()),
        }
    }

    #[test]
    fn test_all_pass() {
        let c = TestCaseComparison {
            name: "t1".to_string(),
            vm: Some(pass("t1")),
            a2r: Some(pass("t1")),
            rust: Some(pass("t1")),
        };
        assert_eq!(c.classify(), BugSource::Consistent);
    }

    #[test]
    fn test_vm_bug() {
        let c = TestCaseComparison {
            name: "t2".to_string(),
            vm: Some(fail("t2")),
            a2r: Some(pass("t2")),
            rust: Some(pass("t2")),
        };
        assert_eq!(c.classify(), BugSource::VmBug);
    }

    #[test]
    fn test_a2r_bug() {
        let c = TestCaseComparison {
            name: "t3".to_string(),
            vm: Some(pass("t3")),
            a2r: Some(fail("t3")),
            rust: Some(pass("t3")),
        };
        assert_eq!(c.classify(), BugSource::A2rBug);
    }

    #[test]
    fn test_replication_bug_vm_a2r_pass() {
        // VM and a2r agree (pass), Rust differs (fail) -> ReplicationBug
        let c = TestCaseComparison {
            name: "t4".to_string(),
            vm: Some(pass("t4")),
            a2r: Some(pass("t4")),
            rust: Some(fail("t4")),
        };
        assert_eq!(c.classify(), BugSource::ReplicationBug);
    }

    #[test]
    fn test_replication_bug_vm_a2r_both_fail() {
        // (vm=fail, a2r=fail, rust=pass): VM and a2r agree on the failure,
        // but Rust disagrees. Per design spec §2.2.5, this is a ReplicationBug
        // ("VM 和 a2r 一致地错，但与原始库不一致").
        let c = TestCaseComparison {
            name: "t5".to_string(),
            vm: Some(fail("t5")),
            a2r: Some(fail("t5")),
            rust: Some(pass("t5")),
        };
        assert_eq!(c.classify(), BugSource::ReplicationBug);
    }

    #[test]
    fn test_test_case_issue_missing_backend() {
        // Missing Rust result -> TestCaseIssue
        let c = TestCaseComparison {
            name: "t6".to_string(),
            vm: Some(pass("t6")),
            a2r: Some(pass("t6")),
            rust: None,
        };
        assert_eq!(c.classify(), BugSource::TestCaseIssue);
    }

    #[test]
    fn test_test_case_issue_total_disagreement() {
        // All three differ -> TestCaseIssue
        // vm pass, a2r fail, rust fail -> a==false (rust), b==false (a2r) so VM != a2r
        // wait: vm=true, a2r=false, rust=false
        //   (Some(true), Some(false), Some(false)) -> rust=false branch
        //   guard: a == b => vm_pass == a2r_pass => true == false => false, falls to TestCaseIssue ✓
        let c = TestCaseComparison {
            name: "t7".to_string(),
            vm: Some(pass("t7")),
            a2r: Some(fail("t7")),
            rust: Some(fail("t7")),
        };
        assert_eq!(c.classify(), BugSource::TestCaseIssue);
    }

    #[test]
    fn test_consistency_rate() {
        let report = ComparisonReport {
            library: "test".to_string(),
            cases: vec![
                TestCaseComparison {
                    name: "a".to_string(),
                    vm: Some(pass("a")),
                    a2r: Some(pass("a")),
                    rust: Some(pass("a")),
                },
                TestCaseComparison {
                    name: "b".to_string(),
                    vm: Some(fail("b")),
                    a2r: Some(pass("b")),
                    rust: Some(pass("b")),
                },
            ],
        };
        assert_eq!(report.consistent_count(), 1);
        assert_eq!(report.total_count(), 2);
        assert_eq!(report.consistency_rate(), 50.0);
        assert_eq!(report.divergences().len(), 1);
    }

    #[test]
    fn test_consistency_rate_empty() {
        let report = ComparisonReport {
            library: "empty".to_string(),
            cases: vec![],
        };
        assert_eq!(report.total_count(), 0);
        assert_eq!(report.consistency_rate(), 100.0);
    }

    #[test]
    fn test_backend_names() {
        assert_eq!(Backend::Vm.name(), "AutoVM");
        assert_eq!(Backend::A2r.name(), "a2r");
        assert_eq!(Backend::Rust.name(), "Rust");
    }

    #[test]
    fn test_bug_source_labels() {
        assert_eq!(BugSource::Consistent.label(), "consistent");
        assert_eq!(BugSource::ReplicationBug.label(), "replication bug");
        assert_eq!(BugSource::A2rBug.label(), "a2r transpiler bug");
        assert_eq!(BugSource::VmBug.label(), "AutoVM bug");
        assert_eq!(BugSource::TestCaseIssue.label(), "test case issue");
    }
}
