//! Block spec parsing & validation (Plan 342).
//!
//! A `spec.md` is `+++`-delimited TOML frontmatter followed by a markdown body.
//! This module parses the frontmatter into [`BlockSpec`] and cross-checks it
//! against the package's reference files and the widget registry.

use std::collections::HashMap;

/// Typed `dataSource` slot signatures (fn name -> signature string).
pub type DataSourceSignature = HashMap<String, String>;

/// Parsed frontmatter of a block `spec.md`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct BlockSpec {
    pub kind: String,
    pub name: String,
    /// Widgets this block composes; each must exist in `WidgetRegistry`.
    pub palette: Vec<String>,
    /// Bounded vocabulary of variable points (EDIT regions).
    #[serde(default)]
    pub extension_points: Vec<String>,
    /// Named presets; each must have `reference/<variant>.at`.
    #[serde(default)]
    pub variants: Vec<String>,
    /// Typed fetcher signatures the block expects.
    #[serde(default, rename = "dataSource")]
    pub data_source: HashMap<String, String>,
    /// Acceptance checklist (prose; checked where machine-feasible elsewhere).
    #[serde(default)]
    pub acceptance: Vec<String>,
}

/// Allow TOML's `dataSource` key to deserialize into the `data_source` field.
#[allow(clippy::derivable_impls)]
impl Default for BlockSpec {
    fn default() -> Self {
        Self {
            kind: String::new(),
            name: String::new(),
            palette: Vec::new(),
            extension_points: Vec::new(),
            variants: Vec::new(),
            data_source: HashMap::new(),
            acceptance: Vec::new(),
        }
    }
}

/// Split a `spec.md` into `(frontmatter, body)`. Frontmatter is the text
/// between the first `+++` pair. Returns `Err` if absent/malformed.
pub fn split_frontmatter(spec_md: &str) -> Result<(&str, &str), String> {
    let trimmed = spec_md.trim_start_matches(['\u{feff}', '\n', '\r', ' ', '\t']);
    if !trimmed.starts_with("+++") {
        return Err("spec.md must start with `+++` frontmatter".into());
    }
    let after_open = &trimmed[3..];
    let close = after_open
        .find("\n+++")
        .ok_or_else(|| "spec.md frontmatter is missing closing `+++`".to_string())?;
    let frontmatter = &after_open[..close];
    let body = &after_open[close + 4..]; // skip "\n+++"
    // Drop a single leading newline from the body.
    let body = body.strip_prefix('\n').or_else(|| body.strip_prefix("\r\n")).unwrap_or(body);
    Ok((frontmatter, body))
}

impl BlockSpec {
    /// Parse frontmatter (TOML) into a [`BlockSpec`].
    pub fn parse(frontmatter: &str) -> Result<Self, String> {
        let mut spec: BlockSpec = toml::from_str(frontmatter)
            .map_err(|e| format!("invalid spec frontmatter: {e}"))?;
        if spec.kind.trim().is_empty() {
            return Err("spec: `kind` is required".into());
        }
        if spec.name.trim().is_empty() {
            return Err("spec: `name` is required".into());
        }
        // Normalize: trim + dedupe palette/variants, sort for stable output.
        spec.palette = normalize_list(&spec.palette);
        spec.variants = normalize_list(&spec.variants);
        spec.extension_points = normalize_list(&spec.extension_points);
        Ok(spec)
    }

    /// Parse a full `spec.md` document (frontmatter + body).
    pub fn parse_document(spec_md: &str) -> Result<(Self, &str), String> {
        let (fm, body) = split_frontmatter(spec_md)?;
        Ok((Self::parse(fm)?, body))
    }
}

fn normalize_list(input: &[String]) -> Vec<String> {
    let mut out: Vec<String> = input
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "+++
kind = \"form\"
name = \"login\"
palette = [\"Input\", \"Button\", \"Label\"]
extension_points = [\"fields\", \"submit\"]
variants = [\"minimal\", \"with_sso\"]

[dataSource]
attempt = \"(creds) -> Session\"
+++

# Intent
A login form.
";

    #[test]
    fn parses_frontmatter() {
        let (spec, body) = BlockSpec::parse_document(SAMPLE).unwrap();
        assert_eq!(spec.kind, "form");
        assert_eq!(spec.name, "login");
        assert_eq!(spec.palette, vec!["Button", "Input", "Label"]); // sorted
        assert_eq!(spec.variants, vec!["minimal", "with_sso"]);
        assert_eq!(spec.data_source.get("attempt").unwrap(), "(creds) -> Session");
        assert!(body.contains("# Intent"));
    }

    #[test]
    fn rejects_missing_frontmatter() {
        let res = BlockSpec::parse_document("no frontmatter here");
        assert!(res.is_err());
    }

    #[test]
    fn rejects_missing_kind() {
        let bad = "+++\nname = \"x\"\npalette = []\nvariants = []\n+++\n";
        assert!(BlockSpec::parse_document(bad).is_err());
    }
}
