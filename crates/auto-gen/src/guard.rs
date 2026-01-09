use crate::error::GenResult;
use auto_val::AutoStr;
use regex::Regex;
use std::collections::HashMap;

/// Guard block processor for preserving hand-written code
pub struct GuardProcessor {
    start_pattern: Regex,
    end_pattern: Regex,
}

impl GuardProcessor {
    pub fn new() -> Self {
        Self {
            start_pattern: Regex::new(r#"///\s*----------\s+begin\s+of\s+guard:\s*<(\w+)>"#)
                .unwrap(),
            end_pattern: Regex::new(r#"///\s*----------\s+end\s+of\s+guard:"#).unwrap(),
        }
    }

    /// Extract all guarded sections from content
    pub fn extract_guards(&self, content: &str) -> HashMap<AutoStr, GuardedSection> {
        let mut guards = HashMap::new();
        let mut current_guard: Option<GuardedSection> = None;
        let mut line_number = 0;

        for line in content.lines() {
            line_number += 1;

            if let Some(caps) = self.start_pattern.captures(line) {
                let id: AutoStr = caps[1].to_string().into();
                current_guard = Some(GuardedSection {
                    id: id.clone(),
                    content: AutoStr::new(),
                    start_line: line_number,
                    end_line: 0,
                });
            } else if self.end_pattern.is_match(line) {
                if let Some(mut guard) = current_guard.take() {
                    guard.end_line = line_number;
                    guards.insert(guard.id.clone(), guard);
                }
            } else if let Some(ref mut guard) = current_guard {
                guard.content.push_str(line);
                guard.content.push('\n');
            }
        }

        guards
    }

    /// Merge existing content with generated content, preserving guarded sections
    pub fn merge(&self, existing: &str, generated: &str) -> GenResult<String> {
        let existing_guards = self.extract_guards(existing);
        let generated_guards = self.extract_guards(generated);

        let mut result = String::new();
        let mut in_guard = false;
        let mut current_guard_id: Option<AutoStr> = None;

        for line in generated.lines() {
            if let Some(caps) = self.start_pattern.captures(line) {
                let guard_id: AutoStr = caps[1].to_string().into();
                in_guard = true;
                current_guard_id = Some(guard_id.clone());
                result.push_str(line);
                result.push('\n');

                // Use existing guard content if available
                if let Some(existing_guard) = existing_guards.get(&guard_id) {
                    result.push_str(&existing_guard.content);
                }
            } else if in_guard && self.end_pattern.is_match(line) {
                in_guard = false;
                current_guard_id = None;
                result.push_str(line);
                result.push('\n');
            } else if !in_guard {
                result.push_str(line);
                result.push('\n');
            }
        }

        Ok(result)
    }

    /// Detect conflicts between existing and generated guarded sections
    pub fn detect_conflicts(&self, existing: &str, generated: &str) -> Vec<Conflict> {
        let existing_guards = self.extract_guards(existing);
        let generated_guards = self.extract_guards(generated);

        let mut conflicts = Vec::new();

        for (id, generated_guard) in generated_guards.iter() {
            if let Some(existing_guard) = existing_guards.get(id) {
                if existing_guard.content != generated_guard.content {
                    conflicts.push(Conflict {
                        guard_id: id.clone(),
                        existing_content: existing_guard.content.clone(),
                        generated_content: generated_guard.content.clone(),
                    });
                }
            }
        }

        conflicts
    }
}

impl Default for GuardProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// A guarded section of code
#[derive(Debug, Clone)]
pub struct GuardedSection {
    pub id: AutoStr,
    pub content: AutoStr,
    pub start_line: usize,
    pub end_line: usize,
}

/// A conflict between existing and generated guarded content
#[derive(Debug, Clone)]
pub struct Conflict {
    pub guard_id: AutoStr,
    pub existing_content: AutoStr,
    pub generated_content: AutoStr,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_guards() {
        let processor = GuardProcessor::new();
        let content = r#"
/// ---------- begin of guard: <test> ---
hello world
/// ---------- end of guard: ---
"#;
        let guards = processor.extract_guards(content);
        assert_eq!(guards.len(), 1);
        let key: AutoStr = "test".into();
        assert_eq!(guards.get(&key).unwrap().content, "hello world\n");
    }

    #[test]
    fn test_merge_preserves_existing() {
        let processor = GuardProcessor::new();
        let existing = r#"
/// ---------- begin of guard: <test> ---
original content
/// ---------- end of guard: ---
"#;
        let generated = r#"
/// ---------- begin of guard: <test> ---
new content
/// ---------- end of guard: ---
"#;
        let merged = processor.merge(existing, generated).unwrap();
        assert!(merged.contains("original content"));
        assert!(!merged.contains("new content"));
    }

    #[test]
    fn test_detect_conflicts() {
        let processor = GuardProcessor::new();
        let existing = r#"
/// ---------- begin of guard: <test> ---
original
/// ---------- end of guard: ---
"#;
        let generated = r#"
/// ---------- begin of guard: <test> ---
changed
/// ---------- end of guard: ---
"#;
        let conflicts = processor.detect_conflicts(existing, generated);
        assert_eq!(conflicts.len(), 1);
        assert_eq!(conflicts[0].guard_id, "test");
    }
}
