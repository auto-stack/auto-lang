//! Handoff Document
//!
//! The core token-efficiency mechanism. When an agent finishes,
//! its work is compressed into a structured Handoff Document —
//! NOT raw chat history. This prevents context explosion.

use serde::{Deserialize, Serialize};

/// A structured handoff between agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffDocument {
    pub from: String,
    pub to: String,
    pub run_id: String,
    pub checkpoint_id: u64,
    pub summary: String,
    pub decisions: Vec<Decision>,
    pub open_questions: Vec<Question>,
    pub spec_updates: Vec<SpecUpdate>,
    pub work_product: Vec<WorkProduct>,
    pub context_for_next: ContextPointers,
    pub token_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub status: String, // "made", "deferred", "rejected"
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Question {
    pub id: String,
    pub text: String,
    pub status: String, // "open", "answered", "blocked"
    pub assigned_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecUpdate {
    pub section_id: String,
    pub item_id: Option<String>,
    pub change_type: String, // "added", "modified", "removed"
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProduct {
    pub path: String,
    pub description: String,
    pub lines: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContextPointers {
    pub files_to_read: Vec<String>,
    pub specs_to_follow: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub step_input: u64,
    pub step_output: u64,
    pub cumulative: u64,
    pub budget_remaining: u64,
}

impl HandoffDocument {
    pub fn new(from: &str, to: &str, run_id: &str, checkpoint_id: u64) -> Self {
        Self {
            from: from.to_string(),
            to: to.to_string(),
            run_id: run_id.to_string(),
            checkpoint_id,
            summary: String::new(),
            decisions: Vec::new(),
            open_questions: Vec::new(),
            spec_updates: Vec::new(),
            work_product: Vec::new(),
            context_for_next: ContextPointers::default(),
            token_usage: TokenUsage::default(),
        }
    }

    /// Render as markdown for the next agent's consumption.
    pub fn render(&self) -> String {
        let mut lines = Vec::new();
        lines.push(format!(
            "/// handoff:from={} to={} run={} checkpoint={}\n",
            self.from, self.to, self.run_id, self.checkpoint_id
        ));
        lines.push(format!("# Handoff: {} → {}", self.from, self.to));
        lines.push(String::new());

        // Summary
        lines.push("## Summary".to_string());
        lines.push(self.summary.clone());
        lines.push(String::new());

        // Decisions
        if !self.decisions.is_empty() {
            lines.push("## Decisions Made".to_string());
            for d in &self.decisions {
                lines.push(format!("- **{}** ({}): {}", d.id, d.status, d.title));
                if !d.rationale.is_empty() {
                    lines.push(format!("  - Rationale: {}", d.rationale));
                }
            }
            lines.push(String::new());
        }

        // Open Questions
        if !self.open_questions.is_empty() {
            lines.push("## Open Questions".to_string());
            for q in &self.open_questions {
                lines.push(format!("- **{}** ({}): {}", q.id, q.status, q.text));
            }
            lines.push(String::new());
        }

        // Spec Updates
        if !self.spec_updates.is_empty() {
            lines.push("## Spec Updates".to_string());
            for u in &self.spec_updates {
                lines.push(format!(
                    "- {} {}: {}",
                    u.change_type, u.section_id, u.description
                ));
            }
            lines.push(String::new());
        }

        // Work Product
        if !self.work_product.is_empty() {
            lines.push("## Work Product".to_string());
            for wp in &self.work_product {
                let size = wp.lines.map(|l| format!(" ({} lines)", l)).unwrap_or_default();
                lines.push(format!("- `{}`{}{}", wp.path, size, wp.description));
            }
            lines.push(String::new());
        }

        // Context for Next Agent
        if !self.context_for_next.files_to_read.is_empty()
            || !self.context_for_next.specs_to_follow.is_empty()
            || !self.context_for_next.warnings.is_empty()
        {
            lines.push("## Context for Next Agent".to_string());
            if !self.context_for_next.files_to_read.is_empty() {
                lines.push("\n### Files to Read".to_string());
                for f in &self.context_for_next.files_to_read {
                    lines.push(format!("- {}", f));
                }
            }
            if !self.context_for_next.specs_to_follow.is_empty() {
                lines.push("\n### Specs to Follow".to_string());
                for s in &self.context_for_next.specs_to_follow {
                    lines.push(format!("- {}", s));
                }
            }
            if !self.context_for_next.warnings.is_empty() {
                lines.push("\n### Warnings".to_string());
                for w in &self.context_for_next.warnings {
                    lines.push(format!("- ⚠️ {}", w));
                }
            }
            lines.push(String::new());
        }

        // Token usage
        lines.push(format!(
            "## Token Spend\n- This step: {} tokens\n- Cumulative: {} tokens\n- Budget remaining: {} tokens\n",
            self.token_usage.step_input + self.token_usage.step_output,
            self.token_usage.cumulative,
            self.token_usage.budget_remaining
        ));

        lines.join("\n")
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handoff_render() {
        let mut handoff = HandoffDocument::new("planner", "architect", "run-42", 3);
        handoff.summary = "Planner decomposed OAuth2 into 5 sub-tasks.".to_string();
        handoff.decisions.push(Decision {
            id: "D1".to_string(),
            title: "Use JWT instead of sessions".to_string(),
            status: "made".to_string(),
            rationale: "Stateless, scales horizontally".to_string(),
        });
        handoff.open_questions.push(Question {
            id: "Q1".to_string(),
            text: "Should refresh tokens be rotated?".to_string(),
            status: "open".to_string(),
            assigned_to: Some("security-reviewer".to_string()),
        });
        handoff.work_product.push(WorkProduct {
            path: "docs/specs/plans.ad".to_string(),
            description: "Updated Plans section".to_string(),
            lines: Some(45),
        });
        handoff.context_for_next.files_to_read.push("src/auth/mod.rs".to_string());
        handoff.token_usage = TokenUsage {
            step_input: 5240,
            step_output: 3180,
            cumulative: 8420,
            budget_remaining: 91580,
        };

        let rendered = handoff.render();
        assert!(rendered.contains("Handoff: planner → architect"));
        assert!(rendered.contains("Planner decomposed OAuth2"));
        assert!(rendered.contains("D1"));
        assert!(rendered.contains("Q1"));
        assert!(rendered.contains("src/auth/mod.rs"));
        assert!(rendered.contains("8420"));
    }

    #[test]
    fn test_empty_handoff_render() {
        let handoff = HandoffDocument::new("coder", "tester", "run-1", 1);
        let rendered = handoff.render();
        assert!(rendered.contains("Handoff: coder → tester"));
    }
}
