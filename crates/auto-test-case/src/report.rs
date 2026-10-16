//! Per-check report model (`autolang.test-report/1`), projections, and the
//! gate exit reducer.
//!
//! Statuses are per CHECK. A green suite pass never hides zero checks: counts
//! include NOT_RUN/UNDECLARED, and required-coverage gaps force a non-zero
//! exit. Exit codes: 0 = selected gate satisfied; 1 = assertion or required
//! coverage failure; 2 = contract/infrastructure error (takes precedence
//! when both apply).

use std::collections::BTreeMap;

use crate::compare::Mismatch;

pub const REPORT_SCHEMA: &str = "autolang.test-report/1";
pub const REPORT_SCHEMA_V2: &str = "autolang.test-report/2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Status {
    Pass,
    Fail,
    Error,
    Blocked,
    Skip,
    NotApplicable,
    Undeclared,
    NotRun,
}

impl Status {
    pub fn as_str(&self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Fail => "FAIL",
            Status::Error => "ERROR",
            Status::Blocked => "BLOCKED",
            Status::Skip => "SKIP",
            Status::NotApplicable => "NOT_APPLICABLE",
            Status::Undeclared => "UNDECLARED",
            Status::NotRun => "NOT_RUN",
        }
    }
}

/// Identity hashes a result is bound to; any drift makes an old PASS stale.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IdentityHashes {
    pub suite: String,
    pub case_file: String,
    /// Combined virtual-path+bytes identity of the case's input project.
    pub source: String,
    /// Hash of the check's expectation block.
    pub oracle: String,
    /// Hash of the case contract fence text.
    pub contract: String,
}

impl IdentityHashes {
    pub fn is_stale(&self, current: &IdentityHashes) -> bool {
        self != current
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArtifactRef {
    pub logical_path: String,
    /// Path under the run's artifact directory (display form).
    pub stored_path: String,
    pub sha256: String,
    pub bytes: Option<usize>,
}

/// PLAN-788 v2: native-adapter observation record attached to a check
/// outcome — the worker's stage trace, cache state, real counters and
/// toolchain identity. Purely observational (comparisons stay in the
/// parent); absent on v1 reports and on non-native checks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NativeRecord {
    pub exit_code: Option<i32>,
    /// (stage, wall_ms) spans from the worker trace, in observation order.
    pub stage_spans: Vec<(String, u64)>,
    pub cache_mode: Option<String>,
    /// Driver hit/miss summary lines (summary / hir / object).
    pub cache_hits: Vec<(String, String)>,
    /// (counter, value) pairs from the real driver counters.
    pub counters: Vec<(String, u32)>,
    /// Worker toolchain identity axes (lld hash, SDK dir, versions).
    pub toolchain: Vec<(String, String)>,
    /// Metrics this batch cannot capture — recorded as gaps, never as 0
    /// (unimplemented stages are absent spans, not 0ms entries).
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CheckOutcome {
    pub suite_id: String,
    pub case_id: String,
    pub check_id: String,
    /// `suite_id:suite_sha:case_id:check_id` — one per suite binding.
    pub execution_key: String,
    pub adapter: String,
    pub action: String,
    pub tier: Option<String>,
    pub status: Status,
    /// Assertion mismatches (status Fail); empty otherwise.
    pub mismatches: Vec<Mismatch>,
    /// Error detail (status Error/Blocked); skip reason (status Skip).
    pub detail: Option<String>,
    pub identity: IdentityHashes,
    pub artifacts: Vec<ArtifactRef>,
    pub duration_ms: u128,
    /// PLAN-788 v2 native observation record (None on v1-shaped outcomes).
    pub native: Option<NativeRecord>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunReport {
    pub schema: String,
    pub suite_id: String,
    pub suite_sha256: String,
    /// Adapters the suite requires of every selected case.
    pub required_adapters: Vec<String>,
    /// Toolchain/config identity the results are bound to (Plan 773 F-06):
    /// core crate version, vendored parser fingerprint, driver crate
    /// version, and the effective admission limits. Populated by the runner;
    /// a stale check compares it axis by axis.
    pub toolchain: BTreeMap<String, String>,
    pub results: Vec<CheckOutcome>,
    /// (case, adapter) pairs selected but missing a declared, non-skipped
    /// check — required-coverage contract errors.
    pub coverage_gaps: Vec<(String, String)>,
    /// Suite/case-file level load errors (unreadable or undecodable files,
    /// duplicate identities, resource overruns at load). These are contract
    /// errors and force exit 2 even when every loaded check passes.
    pub load_errors: Vec<String>,
}

impl RunReport {
    pub fn new(suite_id: &str, suite_sha256: &str, required_adapters: &[String]) -> Self {
        RunReport {
            schema: REPORT_SCHEMA.to_string(),
            suite_id: suite_id.to_string(),
            suite_sha256: suite_sha256.to_string(),
            required_adapters: required_adapters.to_vec(),
            toolchain: BTreeMap::new(),
            results: Vec::new(),
            coverage_gaps: Vec::new(),
            load_errors: Vec::new(),
        }
    }

    pub fn count(&self, status: Status) -> usize {
        self.results.iter().filter(|r| r.status == status).count()
    }

    pub fn counts(&self) -> BTreeMap<&'static str, usize> {
        let mut m = BTreeMap::new();
        for r in &self.results {
            *m.entry(r.status.as_str()).or_insert(0) += 1;
        }
        m
    }

    /// Gate exit code: 2 (contract/infra errors) over 1 (assertion/coverage)
    /// over 0. A report with ZERO dynamic checks is a contract error (exit
    /// 2) — an aggregate green must never hide an empty run (Plan 773 F-06).
    pub fn exit_code(&self) -> i32 {
        if self.results.is_empty() {
            return 2;
        }
        if self.count(Status::Error) > 0
            || self.count(Status::Blocked) > 0
            || !self.coverage_gaps.is_empty()
            || !self.load_errors.is_empty()
        {
            return 2;
        }
        if self.count(Status::Fail) > 0 || self.count(Status::NotRun) > 0 {
            return 1;
        }
        0
    }

    /// Hand-rolled JSON (dependency-light core; same object feeds console).
    pub fn to_json(&self) -> String {
        let mut s = String::new();
        s.push_str("{\n");
        s.push_str(&format!("  \"schema\": {},\n", json_str(&self.schema)));
        s.push_str(&format!("  \"suite\": {},\n", json_str(&self.suite_id)));
        s.push_str(&format!(
            "  \"suite_sha256\": {},\n",
            json_str(&self.suite_sha256)
        ));
        s.push_str(&format!(
            "  \"required_adapters\": [{}],\n",
            self.required_adapters
                .iter()
                .map(|a| json_str(a))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        let counts = self.counts();
        let mut summary_parts = vec![format!("\"total\": {}", self.results.len())];
        for (k, v) in &counts {
            summary_parts.push(format!("\"{}\": {}", k, v));
        }
        s.push_str(&format!(
            "  \"summary\": {{ {} }},\n",
            summary_parts.join(", ")
        ));
        s.push_str(&format!(
            "  \"toolchain\": {{ {} }},\n",
            self.toolchain
                .iter()
                .map(|(k, v)| format!("{}: {}", json_str(k), json_str(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str(&format!(
            "  \"coverage_gaps\": [{}],\n",
            self.coverage_gaps
                .iter()
                .map(|(c, a)| format!("[{}, {}]", json_str(c), json_str(a)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str(&format!(
            "  \"load_errors\": [{}],\n",
            self.load_errors
                .iter()
                .map(|e| json_str(e))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        s.push_str("  \"results\": [\n");
        for (i, r) in self.results.iter().enumerate() {
            s.push_str("    {\n");
            s.push_str(&format!(
                "      \"execution_key\": {},\n",
                json_str(&r.execution_key)
            ));
            s.push_str(&format!("      \"case\": {},\n", json_str(&r.case_id)));
            s.push_str(&format!("      \"check\": {},\n", json_str(&r.check_id)));
            s.push_str(&format!(
                "      \"adapter\": {},\n \"action\": {},\n",
                json_str(&r.adapter),
                json_str(&r.action)
            ));
            s.push_str(&format!(
                "      \"tier\": {},\n",
                r.tier.as_deref().map(json_str).unwrap_or("null".into())
            ));
            s.push_str(&format!(
                "      \"status\": {},\n",
                json_str(r.status.as_str())
            ));
            if let Some(d) = &r.detail {
                s.push_str(&format!("      \"detail\": {},\n", json_str(d)));
            }
            if !r.mismatches.is_empty() {
                s.push_str("      \"mismatches\": [\n");
                for (j, m) in r.mismatches.iter().enumerate() {
                    s.push_str(&format!(
                        "        {}{}\n",
                        json_str(&m.to_string()),
                        if j + 1 < r.mismatches.len() { "," } else { "" }
                    ));
                }
                s.push_str("      ],\n");
            }
            if !r.artifacts.is_empty() {
                s.push_str("      \"artifacts\": [\n");
                for (j, a) in r.artifacts.iter().enumerate() {
                    s.push_str(&format!(
                        "        {{ \"path\": {}, \"stored\": {}, \"sha256\": {}, \"bytes\": {} }}{}\n",
                        json_str(&a.logical_path),
                        json_str(&a.stored_path),
                        json_str(&a.sha256),
                        a.bytes.unwrap_or(0),
                        if j + 1 < r.artifacts.len() { "," } else { "" }
                    ));
                }
                s.push_str("      ],\n");
            }
            s.push_str(&format!(
                "      \"identity\": {{ \"suite\": {}, \"case_file\": {}, \"source\": {}, \"oracle\": {}, \"contract\": {} }},\n",
                json_str(&r.identity.suite),
                json_str(&r.identity.case_file),
                json_str(&r.identity.source),
                json_str(&r.identity.oracle),
                json_str(&r.identity.contract)
            ));
            if r.native.is_some() {
                // native is the LAST field: duration_ms needs its comma now.
                s.push_str(&format!("      \"duration_ms\": {},\n", r.duration_ms));
            } else {
                s.push_str(&format!("      \"duration_ms\": {}\n", r.duration_ms));
            }
            if let Some(n) = &r.native {
                let mut parts: Vec<String> = Vec::new();
                if let Some(code) = n.exit_code {
                    parts.push(format!("\"exit_code\": {}", code));
                }
                if let Some(mode) = &n.cache_mode {
                    parts.push(format!("\"cache_mode\": {}", json_str(mode)));
                }
                if !n.cache_hits.is_empty() {
                    parts.push(format!(
                        "\"cache_hits\": [{}]",
                        n.cache_hits
                            .iter()
                            .map(|(k, v)| format!("[{}, {}]", json_str(k), json_str(v)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !n.counters.is_empty() {
                    parts.push(format!(
                        "\"counters\": [{}]",
                        n.counters
                            .iter()
                            .map(|(k, v)| format!("[{}, {}]", json_str(k), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !n.stage_spans.is_empty() {
                    parts.push(format!(
                        "\"stage_spans\": [{}]",
                        n.stage_spans
                            .iter()
                            .map(|(k, v)| format!("[{}, {}]", json_str(k), v))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !n.toolchain.is_empty() {
                    parts.push(format!(
                        "\"toolchain\": [{}]",
                        n.toolchain
                            .iter()
                            .map(|(k, v)| format!("[{}, {}]", json_str(k), json_str(v)))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !n.gaps.is_empty() {
                    parts.push(format!(
                        "\"gaps\": [{}]",
                        n.gaps
                            .iter()
                            .map(|g| json_str(g))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                if !parts.is_empty() {
                    // LAST field of the result object — no trailing comma.
                    s.push_str(&format!(
                        "      \"native\": {{\n        {}\n      }}\n",
                        parts.join(",\n        ")
                    ));
                }
            }
            s.push_str(&format!(
                "    }}{}\n",
                if i + 1 < self.results.len() { "," } else { "" }
            ));
        }
        s.push_str("  ]\n");
        s.push('}');
        s
    }

    /// Console projection — same data, human-sized.
    pub fn to_console(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "suite {} ({}), schema {}\n",
            self.suite_id, self.suite_sha256, self.schema
        ));
        for r in &self.results {
            out.push_str(&format!(
                "{:<12} {} / {} [{}::{}]{}{}\n",
                r.status.as_str(),
                r.case_id,
                r.check_id,
                r.adapter,
                r.action,
                r.tier
                    .as_deref()
                    .map(|t| format!(" tier={}", t))
                    .unwrap_or_default(),
                r.detail
                    .as_deref()
                    .map(|d| format!(" — {}", d))
                    .unwrap_or_default(),
            ));
            for m in &r.mismatches {
                out.push_str(&format!("    {}\n", m));
            }
            if let Some(n) = &r.native {
                let bits: Vec<String> = std::iter::once(match n.exit_code {
                    Some(c) => format!("exit={}", c),
                    None => "exit=none".to_string(),
                })
                .chain(n.cache_hits.iter().map(|(k, v)| format!("{}={}", k, v)))
                .collect();
                if !bits.is_empty() {
                    out.push_str(&format!("    native {}\n", bits.join(" ")));
                }
            }
        }
        let counts = self.counts();
        out.push_str(&format!(
            "total {} ({})\n",
            self.results.len(),
            counts
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(" ")
        ));
        if !self.load_errors.is_empty() {
            out.push_str("load errors (contract):\n");
            for e in &self.load_errors {
                out.push_str(&format!("    {}\n", e));
            }
        }
        if !self.coverage_gaps.is_empty() {
            out.push_str("required-coverage gaps (contract errors):\n");
            for (case, adapter) in &self.coverage_gaps {
                out.push_str(&format!(
                    "    {} missing required adapter {}\n",
                    case, adapter
                ));
            }
        }
        out.push_str(&format!("exit {}\n", self.exit_code()));
        out
    }
}

fn json_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn outcome(status: Status, case: &str, check: &str) -> CheckOutcome {
        CheckOutcome {
            suite_id: "s".into(),
            case_id: case.into(),
            check_id: check.into(),
            execution_key: format!("s:h:{}:{}", case, check),
            adapter: "avm".into(),
            action: "run".into(),
            tier: None,
            status,
            mismatches: vec![],
            detail: None,
            identity: IdentityHashes::default(),
            artifacts: vec![],
            duration_ms: 1,
            native: None,
        }
    }

    #[test]
    fn exit_code_precedence() {
        let mut r = RunReport::new("s", "h", &[]);
        r.results = vec![outcome(Status::Pass, "a", "vm")];
        assert_eq!(r.exit_code(), 0);

        r.results.push(outcome(Status::Fail, "a", "rust"));
        assert_eq!(r.exit_code(), 1);

        r.results.push(outcome(Status::Error, "b", "vm"));
        assert_eq!(
            r.exit_code(),
            2,
            "contract errors take precedence over assertion failures"
        );

        r.coverage_gaps.push(("c".into(), "a2r".into()));
        assert_eq!(r.exit_code(), 2);
    }

    #[test]
    fn coverage_gap_alone_fails_the_gate() {
        let mut r = RunReport::new("s", "h", &["a2r".into()]);
        r.results = vec![outcome(Status::Pass, "a", "vm")];
        r.coverage_gaps.push(("a".into(), "a2r".into()));
        assert_eq!(r.exit_code(), 2);
        assert!(r.to_console().contains("missing required adapter a2r"));
    }

    #[test]
    fn not_run_breaks_green() {
        let mut r = RunReport::new("s", "h", &[]);
        r.results = vec![
            outcome(Status::Pass, "a", "vm"),
            outcome(Status::NotRun, "a", "rust"),
        ];
        assert_eq!(r.exit_code(), 1);
    }

    #[test]
    fn json_and_console_share_the_data() {
        let mut r = RunReport::new("s", "h", &[]);
        let mut o = outcome(Status::Fail, "a", "vm");
        o.mismatches.push(Mismatch {
            observation: "stdout".into(),
            kind: "bytes",
            expected: "x".into(),
            actual: "y".into(),
            first_diff: Some(0),
        });
        r.results = vec![o];
        let json = r.to_json();
        assert!(json.contains("\"status\": \"FAIL\""));
        assert!(json.contains("\"stdout[bytes] mismatch"));
        assert!(r.to_console().contains("stdout[bytes] mismatch"));
    }

    #[test]
    fn staleness_on_any_identity_drift() {
        let a = IdentityHashes {
            suite: "s1".into(),
            case_file: "c1".into(),
            source: "q1".into(),
            oracle: "o1".into(),
            contract: "k1".into(),
        };
        assert!(!a.is_stale(&a.clone()));
        let mut b = a.clone();
        b.oracle = "o2".into();
        assert!(a.is_stale(&b), "oracle drift must invalidate an old PASS");
    }
}
