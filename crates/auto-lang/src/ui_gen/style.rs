//! Style Generator
//!
//! Generates CSS styles for UI components.
//! Currently supports Tailwind CSS classes and scoped styles.

use super::GenResult;

/// Style generator
#[allow(dead_code)]
pub struct StyleGenerator {
    /// Whether to use Tailwind CSS
    use_tailwind: bool,
}

impl StyleGenerator {
    /// Create a new style generator
    pub fn new() -> Self {
        Self {
            use_tailwind: true,
        }
    }

    /// Create a style generator with Tailwind disabled
    pub fn no_tailwind() -> Self {
        Self {
            use_tailwind: false,
        }
    }

    /// Generate scoped CSS from style definitions
    pub fn generate_scoped(&self, styles: &[(String, String)]) -> GenResult<String> {
        let mut css = String::new();

        for (selector, rules) in styles {
            css.push_str(&format!("{} {{\n  {}\n}}\n\n", selector, rules));
        }

        Ok(css)
    }

    /// Convert AutoUI tag to Tailwind classes
    pub fn tag_to_tailwind(tag: &str) -> &'static str {
        match tag {
            "col" | "column" => "flex flex-col",
            "row" => "flex flex-row",
            "center" => "flex items-center justify-center",
            "gap" => "gap-4",
            "p" | "padding" => "p-4",
            "m" | "margin" => "m-4",
            "flex" => "flex",
            "flex-1" => "flex-1",
            "w-full" => "w-full",
            "h-full" => "h-full",
            _ => "",
        }
    }

    /// Merge multiple class strings
    pub fn merge_classes(classes: &[&str]) -> String {
        classes
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Default for StyleGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_style_generator_creation() {
        let gen = StyleGenerator::new();
        assert!(gen.use_tailwind);
    }

    #[test]
    fn test_tag_to_tailwind() {
        assert_eq!(StyleGenerator::tag_to_tailwind("col"), "flex flex-col");
        assert_eq!(StyleGenerator::tag_to_tailwind("row"), "flex flex-row");
        assert_eq!(StyleGenerator::tag_to_tailwind("center"), "flex items-center justify-center");
    }

    #[test]
    fn test_merge_classes() {
        let merged = StyleGenerator::merge_classes(&["flex", "flex-col", ""]);
        assert_eq!(merged, "flex flex-col");
    }

    #[test]
    fn test_generate_scoped() {
        let gen = StyleGenerator::new();
        let styles = vec![
            (".button".to_string(), "color: blue;".to_string()),
            (".text".to_string(), "font-size: 14px;".to_string()),
        ];

        let css = gen.generate_scoped(&styles).unwrap();
        assert!(css.contains(".button"));
        assert!(css.contains("color: blue"));
    }
}
