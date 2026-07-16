use std::collections::HashMap;

/// A single TAP test result line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TapResult {
    pub passed: bool,
    pub number: usize,
    pub name: String,
    pub diagnostics: Option<String>,
}

/// Parse TAP output into a list of results.
///
/// Recognised lines:
/// - `ok <N> - <name>`
/// - `ok <N> <name>`
/// - `not ok <N> - <name> # <diag>`
/// - `not ok <N> <name> # <diag>`
pub fn parse_tap(output: &str) -> Vec<TapResult> {
    let mut results = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("ok ") {
            // "ok 1 - test_name" or "ok 1 test_name"
            let (num, name) = split_tap_line(rest);
            results.push(TapResult {
                passed: true,
                number: num,
                name: name.trim().to_string(),
                diagnostics: None,
            });
        } else if let Some(rest) = line.strip_prefix("not ok ") {
            // "not ok 2 - test_name # got X expected Y"
            let (num, rest) = split_tap_line(rest);
            let (name, diag) = split_diagnostics(rest.trim());
            results.push(TapResult {
                passed: false,
                number: num,
                name: name.trim().to_string(),
                diagnostics: diag.map(|s| s.trim().to_string()),
            });
        }
    }
    results
}

/// Parse TAP output and sort results by test name, renumbering sequentially.
/// Used for async tests where completion order is non-deterministic.
pub fn parse_tap_sorted(output: &str) -> Vec<TapResult> {
    let mut results = parse_tap(output);
    results.sort_by(|a, b| a.name.cmp(&b.name));
    // Renumber after sorting so numbers stay contiguous.
    for (i, r) in results.iter_mut().enumerate() {
        r.number = i + 1;
    }
    results
}

/// Split a TAP line body like `1 - test_name` or `1 test_name` into its
/// numeric test number and the remainder (which still contains the optional
/// `- ` prefix and any diagnostics).
fn split_tap_line(s: &str) -> (usize, String) {
    // "1 - test_name" or "1 test_name"
    let mut iter = s.splitn(2, ' ');
    let num: usize = iter.next().unwrap_or("0").parse().unwrap_or(0);
    let rest = iter.next().unwrap_or("");
    // strip leading "- " if present
    let name = rest.strip_prefix("- ").unwrap_or(rest);
    (num, name.to_string())
}

/// Split a test name body into `(name, Some(diagnostics))` on the first
/// ` # ` separator. If there is no separator, returns `(s, None)`.
fn split_diagnostics(s: &str) -> (String, Option<String>) {
    if let Some(idx) = s.find(" # ") {
        (s[..idx].to_string(), Some(s[idx + 3..].to_string()))
    } else {
        (s.to_string(), None)
    }
}

/// Convert raw TAP output into a name -> TapResult map.
pub fn tap_map(output: &str) -> HashMap<String, TapResult> {
    parse_tap(output)
        .into_iter()
        .map(|r| (r.name.clone(), r))
        .collect()
}

/// Build a name -> TapResult map from already-parsed results.
pub fn tap_map_from_results(results: &[TapResult]) -> HashMap<String, TapResult> {
    results
        .iter()
        .map(|r| (r.name.clone(), r.clone()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pass() {
        let tap = "ok 1 - test_encode_empty\nok 2 - test_encode_single\n";
        let results = parse_tap(tap);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert_eq!(results[0].name, "test_encode_empty");
        assert_eq!(results[0].number, 1);
        assert_eq!(results[1].number, 2);
    }

    #[test]
    fn test_parse_fail() {
        let tap = "not ok 3 - test_decode_bad # got \"abc\" expected \"abd\"\n";
        let results = parse_tap(tap);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert_eq!(results[0].number, 3);
        assert_eq!(results[0].name, "test_decode_bad");
        assert_eq!(
            results[0].diagnostics.as_deref(),
            Some("got \"abc\" expected \"abd\"")
        );
    }

    #[test]
    fn test_parse_fail_no_diagnostics() {
        // A `not ok` line without ` # ` should still parse cleanly.
        let tap = "not ok 7 - test_something\n";
        let results = parse_tap(tap);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert_eq!(results[0].name, "test_something");
        assert!(results[0].diagnostics.is_none());
    }

    #[test]
    fn test_tap_map() {
        let tap = "ok 1 - alpha\nnot ok 2 - beta\n";
        let map = tap_map(tap);
        assert_eq!(map.len(), 2);
        assert!(map["alpha"].passed);
        assert!(!map["beta"].passed);
    }

    #[test]
    fn test_tap_map_from_results() {
        let parsed = parse_tap("ok 1 - alpha\nok 2 - beta\n");
        let map = tap_map_from_results(&parsed);
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("alpha"));
        assert!(map.contains_key("beta"));
    }

    #[test]
    fn test_parse_ignores_non_tap_lines() {
        let tap = "TAP version 14\n1..3\nok 1 - alpha\nsome random log line\nok 2 - beta\n";
        let results = parse_tap(tap);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "alpha");
        assert_eq!(results[1].name, "beta");
    }

    #[test]
    fn test_parse_tap_sorted() {
        let tap = "ok 2 - test_b\nok 1 - test_a\n";
        let results = parse_tap_sorted(tap);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].name, "test_a");
        assert_eq!(results[0].number, 1);
        assert_eq!(results[1].name, "test_b");
        assert_eq!(results[1].number, 2);
    }
}
