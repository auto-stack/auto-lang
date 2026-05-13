//! Soul Configuration Loader
//!
//! A Soul is a markdown document that defines an agent's character:
//! values, working style, handoff rituals, and quality standards.
//! Souls are loaded from `.autoforge/souls/{id}.md`.

use std::path::Path;

/// A Soul defines *who* an agent is — its character and values.
#[derive(Debug, Clone)]
pub struct SoulConfig {
    pub id: String,
    pub name: String,
    pub markdown: String,
    /// Extracted Core Values section lines.
    pub values: Vec<String>,
    /// Extracted Handoff Ritual section text.
    pub handoff_ritual: String,
}

impl SoulConfig {
    /// Load a Soul from a markdown file on disk.
    pub fn load(id: &str, souls_dir: &Path) -> Result<Self, SoulError> {
        let path = souls_dir.join(format!("{}.md", id));
        let markdown = std::fs::read_to_string(&path)
            .map_err(|e| SoulError::LoadError(format!("{}: {}", path.display(), e)))?;
        Self::parse(id, &markdown)
    }

    /// Parse a Soul from markdown text.
    pub fn parse(id: &str, markdown: &str) -> Result<Self, SoulError> {
        // Extract title from first # heading
        let name = markdown
            .lines()
            .find(|l| l.trim_start().starts_with("# "))
            .map(|l| l.trim_start()[2..].trim().to_string())
            .unwrap_or_else(|| format!("Soul of {}", id));

        // Extract Core Values bullet list
        let mut values = Vec::new();
        let mut in_values = false;
        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("## core values")
                || trimmed.eq_ignore_ascii_case("## core values\r")
            {
                in_values = true;
                continue;
            }
            if in_values {
                if trimmed.starts_with("## ") {
                    break;
                }
                if trimmed.starts_with("-") || trimmed.starts_with("*") {
                    values.push(trimmed[1..].trim().to_string());
                }
            }
        }

        // Extract Handoff Ritual section
        let mut handoff_ritual = String::new();
        let mut in_ritual = false;
        for line in markdown.lines() {
            let trimmed = line.trim();
            if trimmed.eq_ignore_ascii_case("## handoff ritual")
                || trimmed.eq_ignore_ascii_case("## handoff ritual\r")
            {
                in_ritual = true;
                continue;
            }
            if in_ritual {
                if trimmed.starts_with("## ") {
                    break;
                }
                handoff_ritual.push_str(line);
                handoff_ritual.push('\n');
            }
        }

        Ok(SoulConfig {
            id: id.to_string(),
            name,
            markdown: markdown.to_string(),
            values,
            handoff_ritual: handoff_ritual.trim().to_string(),
        })
    }

    /// Render the Soul into a system-prompt style text block.
    pub fn render(&self) -> String {
        let mut out = format!("# {}\n\n", self.name);
        if !self.values.is_empty() {
            out.push_str("## Core Values\n");
            for v in &self.values {
                out.push_str(&format!("- {}\n", v));
            }
            out.push('\n');
        }
        if !self.handoff_ritual.is_empty() {
            out.push_str("## Handoff Ritual\n");
            out.push_str(&self.handoff_ritual);
            out.push('\n');
        }
        out
    }
}

#[derive(Debug, Clone)]
pub enum SoulError {
    LoadError(String),
}

impl std::fmt::Display for SoulError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SoulError::LoadError(s) => write!(f, "Load error: {}", s),
        }
    }
}

impl std::error::Error for SoulError {}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_SOUL: &str = r#"# Soul of the Architect

## Core Values
- Simplicity over cleverness
- Explicit over implicit
- Stability over novelty

## Working Style
Before proposing any design, I read the current specs.

## Handoff Ritual
When I finish my work, I produce:
1. Decisions Made
2. Open Questions
3. Spec Updates
"#;

    #[test]
    fn test_parse_soul() {
        let soul = SoulConfig::parse("architect", TEST_SOUL).unwrap();
        assert_eq!(soul.id, "architect");
        assert_eq!(soul.name, "Soul of the Architect");
        assert_eq!(soul.values.len(), 3);
        assert!(soul.values[0].contains("Simplicity"));
        assert!(soul.handoff_ritual.contains("Decisions Made"));
    }

    #[test]
    fn test_render_soul() {
        let soul = SoulConfig::parse("architect", TEST_SOUL).unwrap();
        let rendered = soul.render();
        assert!(rendered.contains("Soul of the Architect"));
        assert!(rendered.contains("Simplicity over cleverness"));
        assert!(rendered.contains("Handoff Ritual"));
    }
}
