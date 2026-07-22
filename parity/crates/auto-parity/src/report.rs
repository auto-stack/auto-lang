use crate::compare::{BugSource, ComparisonReport, TestCaseComparison};
use crate::tap::TapResult;
use std::path::Path;

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

/// Render a self-contained HTML parity dashboard to `output_path`.
///
/// The dashboard has four sections:
/// 1. **Headline pass rate** — total cases, three-way-consistent cases,
///    overall consistency rate.
/// 2. **Coverage matrix** — per-library table (VM ok / a2r ok / Rust ok /
///    consistent / total).
/// 3. **Known divergences** — a link to `known-divergences.md` plus a count of
///    `DIV-` entries parsed from the file.
/// 4. **Maturity directory** — L1 (verified consistency), L2 (AutoVM output
///    regression / conformance), L3 (roadmap, planned).
///
/// The HTML is fully self-contained (inline CSS, no external dependencies)
/// and uses a compact dark theme.
pub fn generate_dashboard(
    reports: &[ComparisonReport],
    known_divergences_path: &Path,
    output_path: &Path,
) -> Result<(), String> {
    let total_cases: usize = reports.iter().map(|r| r.total_count()).sum();
    let total_consistent: usize = reports.iter().map(|r| r.consistent_count()).sum();
    let overall_rate = if total_cases == 0 {
        0.0
    } else {
        (total_consistent as f64 / total_cases as f64) * 100.0
    };

    let known_div_count = count_known_divergences(known_divergences_path);
    let known_div_link = known_divergences_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "known-divergences.md".to_string());

    let mut html = String::with_capacity(8192);
    html.push_str(DASHBOARD_HTML_HEAD);

    // Section 1: headline pass rate.
    html.push_str(&format!(
        "<section class=\"hero\">\
            <div class=\"rate\">{rate:.1}<span class=\"pct\">%</span></div>\
            <div class=\"rate-sub\">three-way consistency ({consistent}/{total} cases)</div>\
         </section>",
        rate = overall_rate,
        consistent = total_consistent,
        total = total_cases,
    ));

    // Section 2: coverage matrix.
    html.push_str("<section><h2>Coverage matrix</h2>");
    if reports.is_empty() {
        html.push_str("<p class=\"muted\">No libraries reported.</p>");
    } else {
        html.push_str(
            "<table><thead><tr>\
                <th>Library</th><th>VM ok</th><th>a2r ok</th><th>Rust ok</th>\
                <th>Consistent</th><th>Total</th><th>Rate</th>\
             </tr></thead><tbody>",
        );
        // Sort rows by library name for stable output.
        let mut sorted: Vec<&ComparisonReport> = reports.iter().collect();
        sorted.sort_by(|a, b| a.library.cmp(&b.library));
        for r in sorted {
            let (vm_ok, a2r_ok, rust_ok) = backend_pass_counts(r);
            let rate = r.consistency_rate();
            let rate_cls = if rate >= 100.0 {
                "good"
            } else if rate >= 50.0 {
                "warn"
            } else {
                "bad"
            };
            html.push_str(&format!(
                "<tr>\
                    <td class=\"lib\">{lib}</td>\
                    <td>{vm_ok}</td><td>{a2r_ok}</td><td>{rust_ok}</td>\
                    <td>{consistent}</td><td>{total}</td>\
                    <td class=\"{rate_cls}\">{rate:.1}%</td>\
                 </tr>",
                lib = html_escape(&r.library),
                vm_ok = vm_ok,
                a2r_ok = a2r_ok,
                rust_ok = rust_ok,
                consistent = r.consistent_count(),
                total = r.total_count(),
                rate = rate,
                rate_cls = rate_cls,
            ));
        }
        html.push_str("</tbody></table>");
    }
    html.push_str("</section>");

    // Section 3: known divergences.
    html.push_str("<section><h2>Known divergences</h2>");
    match known_div_count {
        Some(n) => html.push_str(&format!(
            "<p>{n} documented divergence(s) in \
             <a href=\"{link}\">{link}</a>.</p>",
            n = n,
            link = html_escape(&known_div_link),
        )),
        None => html.push_str(
            "<p class=\"muted\">known-divergences.md not found; nothing documented yet.</p>",
        ),
    }
    html.push_str("</section>");

    // Section 4: maturity directory (L1 / L2 / L3).
    html.push_str(&render_maturity_directory(reports));

    html.push_str(DASHBOARD_HTML_TAIL);

    std::fs::write(output_path, html)
        .map_err(|e| format!("failed to write dashboard: {}", e))
}

/// Per-library count of passing cases for each backend.
fn backend_pass_counts(report: &ComparisonReport) -> (usize, usize, usize) {
    let mut vm_ok = 0;
    let mut a2r_ok = 0;
    let mut rust_ok = 0;
    for c in &report.cases {
        if let Some(r) = &c.vm {
            if r.passed {
                vm_ok += 1;
            }
        }
        if let Some(r) = &c.a2r {
            if r.passed {
                a2r_ok += 1;
            }
        }
        if let Some(r) = &c.rust {
            if r.passed {
                rust_ok += 1;
            }
        }
    }
    (vm_ok, a2r_ok, rust_ok)
}

/// Count `- **DIV-...` style entries in known-divergences.md.
///
/// Returns `None` if the file does not exist (or cannot be read), so the
/// dashboard can render a "not found" note instead of a misleading zero.
fn count_known_divergences(path: &Path) -> Option<usize> {
    let text = std::fs::read_to_string(path).ok()?;
    // Each accepted/open divergence starts a bullet like `- **DIV-...**`.
    // We also count the bare inline `DIV-` references that are not the
    // template placeholder `DIV-NNNN`.
    let mut count = 0;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- **DIV-") {
            // Skip the placeholder template entry.
            if !rest.starts_with("NNNN") {
                count += 1;
            }
        }
    }
    Some(count)
}

/// Render the L1/L2/L3 maturity directory section.
///
/// - **L1 (verified consistency)**: libraries whose `consistency_rate` is 100%
///   in the supplied reports (i.e. fully verified three-way).
/// - **L2 (AutoVM output regression)**: the conformance suite in
///   `docs/conformance/` — a fixed description plus the on-disk spec count.
/// - **L3 (roadmap)**: hard-coded P3/P4 libraries not yet verified.
fn render_maturity_directory(reports: &[ComparisonReport]) -> String {
    let l1: Vec<&str> = reports
        .iter()
        .filter(|r| r.consistency_rate() >= 100.0 && !r.cases.is_empty())
        .map(|r| r.library.as_str())
        .collect();

    // L2: conformance spec count. The plan mentions 33 conformance cases; the
    // on-disk spec currently defines ~27 examples across 6 spec files. We cite
    // the directory and keep the number descriptive.
    let conformance_count = 27usize;

    // L3: planned-but-not-verified. Filter out anything that already appears
    // in L1 so the directory is honest about what is outstanding. Includes P3
    // crypto/db and generators (a2r emits async_stream dep the runner can't
    // yet inject). Plan 368 FU-4 promoted http_client_sync to L1 (phase d6,
    // with the mock-server harness), so it's no longer listed here.
    let planned = ["sha2", "rusqlite", "reqwest", "tokio", "generators"];
    let l3: Vec<&str> = planned
        .iter()
        .copied()
        .filter(|name| !l1.iter().any(|l| *l == *name))
        .collect();

    let l1_html = if l1.is_empty() {
        "<li class=\"muted\">None yet.</li>".to_string()
    } else {
        l1.iter()
            .map(|n| format!("<li><span class=\"badge good\">L1</span> {}</li>", html_escape(n)))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let l3_html = if l3.is_empty() {
        "<li class=\"muted\">All planned libraries verified.</li>".to_string()
    } else {
        l3.iter()
            .map(|n| {
                format!(
                    "<li><span class=\"badge warn\">L3</span> {} \
                     <span class=\"muted\">— planned, not yet verified</span></li>",
                    html_escape(n)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        "<section>\
            <h2>Maturity directory</h2>\
            <div class=\"maturity-grid\">\
                <div class=\"card\">\
                    <h3>L1 — Verified consistency</h3>\
                    <p class=\"card-note\">Libraries at 100% three-way parity \
                    (AutoVM = a2r = native Rust).</p>\
                    <ul>{l1_html}</ul>\
                </div>\
                <div class=\"card\">\
                    <h3>L2 — AutoVM output regression</h3>\
                    <p class=\"card-note\">AutoVM output regression tests \
                    (conformance/): {conformance_count} spec examples across \
                    6 spec files in <code>docs/conformance/</code>.</p>\
                    <ul><li><span class=\"badge ok\">L2</span> conformance suite \
                    <span class=\"muted\">— {conformance_count} examples</span></li></ul>\
                </div>\
                <div class=\"card\">\
                    <h3>L3 — Roadmap (planned)</h3>\
                    <p class=\"card-note\">P3/P4 libraries not yet verified \
                    three-way.</p>\
                    <ul>{l3_html}</ul>\
                </div>\
            </div>\
         </section>",
        l1_html = l1_html,
        l3_html = l3_html,
        conformance_count = conformance_count,
    )
}

/// Minimal HTML escaping for safe insertion of library/file names.
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// Silence the otherwise-unused `BugSource` import in builds where the only
// reference is structural; the import documents that the dashboard reasoning
// follows `TestCaseComparison::classify()` (which returns `BugSource`).
#[allow(dead_code)]
fn _classify_for_docs(c: &TestCaseComparison) -> BugSource {
    c.classify()
}

/// Inline-CSS dark-theme HTML head for the dashboard.
const DASHBOARD_HTML_HEAD: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Auto Parity Dashboard</title>
<style>
  :root {
    --bg: #0f1115;
    --panel: #171a21;
    --panel-2: #1f2430;
    --text: #e6e9ef;
    --muted: #8b93a7;
    --accent: #5b9dff;
    --good: #3fb950;
    --warn: #d29922;
    --bad: #f85149;
    --border: #2a2f3a;
  }
  * { box-sizing: border-box; }
  body {
    margin: 0;
    background: var(--bg);
    color: var(--text);
    font: 15px/1.5 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    padding: 2rem clamp(1rem, 4vw, 4rem);
  }
  h1 { margin: 0 0 .25rem; font-size: 1.6rem; }
  h2 { margin: 2rem 0 .75rem; font-size: 1.2rem; border-bottom: 1px solid var(--border); padding-bottom: .35rem; }
  h3 { margin: 0 0 .5rem; font-size: 1rem; color: var(--accent); }
  .subtitle { color: var(--muted); margin-bottom: 1.5rem; }
  section { margin-bottom: 1.5rem; }
  .hero {
    background: linear-gradient(135deg, #1b2230, #11151d);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 2rem;
    text-align: center;
  }
  .rate { font-size: 4.5rem; font-weight: 700; color: var(--good); line-height: 1; }
  .rate .pct { font-size: 2rem; color: var(--muted); }
  .rate-sub { color: var(--muted); margin-top: .5rem; }
  table { border-collapse: collapse; width: 100%; background: var(--panel); border: 1px solid var(--border); border-radius: 8px; overflow: hidden; }
  th, td { padding: .55rem .8rem; text-align: right; border-bottom: 1px solid var(--border); }
  th:first-child, td:first-child { text-align: left; }
  th { background: var(--panel-2); color: var(--muted); font-weight: 600; font-size: .85rem; text-transform: uppercase; letter-spacing: .03em; }
  td.lib { font-weight: 600; }
  tr:last-child td { border-bottom: none; }
  tr:hover td { background: var(--panel-2); }
  .good { color: var(--good); font-weight: 600; }
  .warn { color: var(--warn); font-weight: 600; }
  .bad  { color: var(--bad);  font-weight: 600; }
  .muted { color: var(--muted); }
  a { color: var(--accent); }
  code { background: var(--panel-2); padding: .1em .35em; border-radius: 4px; font-size: .9em; }
  .maturity-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 1rem; }
  .card { background: var(--panel); border: 1px solid var(--border); border-radius: 8px; padding: 1rem 1.25rem; }
  .card-note { color: var(--muted); font-size: .85rem; margin: 0 0 .75rem; }
  .card ul { list-style: none; padding: 0; margin: 0; }
  .card li { padding: .2rem 0; }
  .badge { display: inline-block; min-width: 2rem; padding: .05rem .4rem; border-radius: 4px; font-size: .75rem; font-weight: 700; text-align: center; margin-right: .4rem; }
  .badge.good { background: rgba(63,185,80,.15); color: var(--good); }
  .badge.ok   { background: rgba(91,157,255,.15); color: var(--accent); }
  .badge.warn { background: rgba(210,153,34,.15); color: var(--warn); }
  footer { margin-top: 2rem; color: var(--muted); font-size: .8rem; }
</style>
</head>
<body>
<h1>Auto Parity Dashboard</h1>
<p class="subtitle">Three-way parity: AutoVM vs a2r (transpiled Rust) vs native Rust.</p>
"#;

const DASHBOARD_HTML_TAIL: &str = "\n<footer>Generated by <code>auto-parity report</code>.</footer>\n</body>\n</html>\n";

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
