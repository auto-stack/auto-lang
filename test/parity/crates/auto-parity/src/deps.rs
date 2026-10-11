// PLAN-594 D1: corpus `dep` statement parsing + Cargo.toml rendering.
//
// The a2r backend builds a standalone binary crate per test and must forward
// the corpus's `dep crate(version: "x.y.z")` declarations into the generated
// `[dependencies]` section. auto-parity is a standalone workspace (no
// auto-lang dependency), so this is a deliberately small line-level parser —
// same syntax subset as the compiler's `dep_scanner` (crates.io form only).
//
// Path/git deps are rejected: real-third-party corpora must pin an exact
// crates.io version so the three legs (VM methods pack / a2r / oracle)
// resolve the same source; local-fixture corpora are PLAN-592's territory
// (ffi_dep_parity_tests, `{{FFI_DUAL_DIR}}` placeholders).

/// One parsed `dep` statement from a corpus `.at` file.
#[derive(Debug, Clone, PartialEq)]
pub struct DepSpec {
    pub name: String,
    pub version: String,
    pub features: Vec<String>,
}

/// Render one dependency as a Cargo.toml `[dependencies]` line (no trailing
/// newline). Version is always pinned; features render as a structured spec.
pub fn render_cargo_line(dep: &DepSpec) -> String {
    if dep.features.is_empty() {
        format!("{} = \"{}\"", dep.name, dep.version)
    } else {
        let feats = dep
            .features
            .iter()
            .map(|f| format!("\"{}\"", f))
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "{} = {{ version = \"{}\", features = [{}] }}",
            dep.name, dep.version, feats
        )
    }
}

/// Scan corpus source for `dep` statements and parse them.
///
/// Accepted forms (crates.io only):
/// - `dep serde_json(version: "1.0.145")`
/// - `dep serde(version: "1.0", features: ["derive", "rc"])`
///
/// Rejected with a descriptive error (the caller surfaces it as a TAP
/// failure): bare `dep foo` (no version — reproducibility requires a pin),
/// `dep foo(path: ...)` / `dep foo(git: ...)` (out of scope), and anything
/// that looks like a dep line but does not parse.
pub fn parse_dep_lines(source: &str) -> Result<Vec<DepSpec>, String> {
    let mut deps = Vec::new();
    for (idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") || trimmed.is_empty() {
            continue;
        }
        let Some(rest) = trimmed.strip_prefix("dep ") else {
            continue;
        };
        let rest = rest.trim();
        let name = rest
            .split(['(', ' ', ':'])
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            return Err(format!(
                "line {}: cannot parse dep crate name from `{}`",
                idx + 1,
                trimmed
            ));
        }
        // Options body (everything inside the outermost parens), if present.
        let body = match (rest.find('('), rest.find(')')) {
            (Some(open), Some(close)) if close > open => &rest[open + 1..close],
            (Some(open), None) => {
                return Err(format!(
                    "line {}: dep {}( — unclosed parenthesis",
                    idx + 1,
                    name
                ));
            }
            _ => "",
        };
        if body.contains("path:") || body.contains("git:") {
            return Err(format!(
                "line {}: dep {} uses path/git source — PLAN-594 dep parity \
                 corpora are crates.io-only (pin a version); local-fixture \
                 parity lives in ffi_dep_parity_tests (PLAN-592)",
                idx + 1,
                name
            ));
        }
        let mut version: Option<String> = None;
        let mut features = Vec::new();
        if !body.is_empty() {
            for part in split_options(body) {
                let part = part.trim();
                if let Some(v) = part.strip_prefix("version:") {
                    version = Some(unquote(v.trim()).ok_or_else(|| {
                        format!(
                            "line {}: dep {} version must be a quoted string",
                            idx + 1,
                            name
                        )
                    })?);
                } else if let Some(f) = part.strip_prefix("features:") {
                    features = parse_features(f.trim()).ok_or_else(|| {
                        format!(
                            "line {}: dep {} features must be [\"a\", \"b\"]",
                            idx + 1,
                            name
                        )
                    })?;
                } else if !part.is_empty() {
                    return Err(format!(
                        "line {}: dep {} has unsupported option `{}`",
                        idx + 1,
                        name,
                        part
                    ));
                }
            }
        }
        let Some(version) = version else {
            return Err(format!(
                "line {}: dep {} has no version — pin an exact crates.io \
                 version, e.g. dep {}(version: \"1.0.0\")",
                idx + 1,
                name,
                name
            ));
        };
        deps.push(DepSpec {
            name,
            version,
            features,
        });
    }
    Ok(deps)
}

/// Split `version: "1.0", features: ["a", "b"]` on commas that are not
/// inside quotes or brackets.
fn split_options(body: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut cur = String::new();
    let mut in_quote = false;
    let mut bracket = 0usize;
    for ch in body.chars() {
        match ch {
            '"' => {
                in_quote = !in_quote;
                cur.push(ch);
            }
            '[' => {
                bracket += 1;
                cur.push(ch);
            }
            ']' => {
                bracket = bracket.saturating_sub(1);
                cur.push(ch);
            }
            ',' if !in_quote && bracket == 0 => {
                parts.push(std::mem::take(&mut cur));
            }
            _ => cur.push(ch),
        }
    }
    if !cur.trim().is_empty() {
        parts.push(cur);
    }
    parts
}

/// `"xxx"` → `xxx`; anything else → None.
fn unquote(s: &str) -> Option<String> {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        Some(s[1..s.len() - 1].to_string())
    } else {
        None
    }
}

/// `["a", "b"]` → vec!["a", "b"]; anything else → None.
fn parse_features(s: &str) -> Option<Vec<String>> {
    let inner = s.strip_prefix('[')?.strip_suffix(']')?;
    if inner.trim().is_empty() {
        return Some(Vec::new());
    }
    let mut feats = Vec::new();
    for part in inner.split(',') {
        feats.push(unquote(part)?);
    }
    Some(feats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_version_and_features() {
        let src = "dep serde(version: \"1.0\", features: [\"derive\", \"rc\"])\n\
                   dep serde_json(version: \"1.0.145\")\n";
        let deps = parse_dep_lines(src).expect("parses");
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "serde");
        assert_eq!(deps[0].version, "1.0");
        assert_eq!(deps[0].features, vec!["derive", "rc"]);
        assert_eq!(deps[1].name, "serde_json");
        assert_eq!(deps[1].version, "1.0.145");
        assert!(deps[1].features.is_empty());
    }

    #[test]
    fn skips_comments_and_non_dep_lines() {
        let src = "// dep fake(version: \"9\")\nfn main() {\n    print(1)\n}\n";
        assert!(parse_dep_lines(src).expect("parses").is_empty());
    }

    #[test]
    fn rejects_bare_dep_without_version() {
        let err = parse_dep_lines("dep serde_json\n").expect_err("must reject");
        assert!(err.contains("no version"), "err: {err}");
    }

    #[test]
    fn rejects_path_and_git_sources() {
        assert!(parse_dep_lines("dep foo(path: \"../foo\")").is_err());
        assert!(parse_dep_lines("dep foo(git: \"https://x\")").is_err());
    }

    #[test]
    fn rejects_unsupported_option_and_unquoted_version() {
        assert!(parse_dep_lines("dep foo(version: \"1\", default-features: false)").is_err());
        assert!(parse_dep_lines("dep foo(version: 1)").is_err());
    }

    #[test]
    fn renders_pinned_line_and_features_spec() {
        assert_eq!(
            render_cargo_line(&DepSpec {
                name: "serde_json".into(),
                version: "1.0.145".into(),
                features: vec![],
            }),
            "serde_json = \"1.0.145\""
        );
        assert_eq!(
            render_cargo_line(&DepSpec {
                name: "serde".into(),
                version: "1.0".into(),
                features: vec!["derive".into()],
            }),
            "serde = { version = \"1.0\", features = [\"derive\"] }"
        );
    }
}
