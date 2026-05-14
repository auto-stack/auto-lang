// StyleParser - Parse Tailwind-style class strings into StyleClass IR
//
// This parser takes a space-separated string of Tailwind utility classes
// and converts them into a Vec<StyleClass> for further processing.

use super::{Style, StyleClass};

/// Parser for Tailwind-style utility class strings
pub struct StyleParser {
    // For future extensions: caching, custom class definitions, etc.
}

impl StyleParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self {}
    }

    /// Parse a space-separated string of style classes.
    /// Unknown classes (e.g. hover:, max-w-*, leading-*) are silently skipped.
    ///
    /// Example: "p-4 gap-2 bg-white flex items-center"
    pub fn parse(&self, input: &str) -> Result<Vec<StyleClass>, String> {
        let classes: Vec<StyleClass> = input
            .split_whitespace()
            .filter_map(|class| StyleClass::parse_single(class).ok())
            .collect();
        Ok(classes)
    }

    /// Parse and create a Style object directly
    pub fn parse_style(&self, input: &str) -> Result<Style, String> {
        let classes = self.parse(input)?;
        Ok(Style { classes })
    }
}

impl Default for StyleParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::SizeValue;

    #[test]
    fn test_parse_multiple_classes() {
        let parser = StyleParser::new();
        let classes = parser.parse("p-4 gap-2 bg-white flex").unwrap();
        assert_eq!(classes.len(), 4);
        assert_eq!(classes[0], StyleClass::Padding(SizeValue::Fixed(4)));
        assert_eq!(classes[1], StyleClass::Gap(SizeValue::Fixed(2)));
    }

    #[test]
    fn test_parse_empty_string() {
        let parser = StyleParser::new();
        let classes = parser.parse("").unwrap();
        assert_eq!(classes.len(), 0);
    }

    #[test]
    fn test_parse_with_extra_whitespace() {
        let parser = StyleParser::new();
        let classes = parser.parse("  p-4   gap-2  ").unwrap();
        assert_eq!(classes.len(), 2);
    }

    #[test]
    fn test_parse_invalid_class() {
        let parser = StyleParser::new();
        // Unknown classes are silently skipped
        let classes = parser.parse("p-4 invalid-class").unwrap();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0], StyleClass::Padding(SizeValue::Fixed(4)));
    }

    #[test]
    fn test_parse_style_object() {
        let parser = StyleParser::new();
        let style = parser.parse_style("flex items-center w-full").unwrap();
        assert_eq!(style.classes.len(), 3);
    }
}
