//! Shared Style Types
//!
//! Common style types used by all UI generators.
//! These types represent the semantic meaning of CSS/Tailwind styles.

// Re-export types from tailwind.rs for convenience
pub use super::tailwind::{
    Color, ComputedStyle, Dimension, Display, FlexDirection, FontWeight, Shadow, Size, Spacing,
    TextAlign,
};

/// Computed style with additional metadata
#[derive(Debug, Clone, Default)]
pub struct ResolvedStyle {
    /// Core computed style
    pub computed: ComputedStyle,
    /// CSS class names (for Vue)
    pub css_classes: Vec<String>,
    /// Inline styles (for Vue)
    pub inline_styles: Vec<(String, String)>,
}

impl ResolvedStyle {
    /// Create a new resolved style
    pub fn new() -> Self {
        Self::default()
    }

    /// Merge another style into this one
    pub fn merge(&mut self, other: &ResolvedStyle) {
        // Merge CSS classes
        for class in &other.css_classes {
            if !self.css_classes.contains(class) {
                self.css_classes.push(class.clone());
            }
        }

        // Merge inline styles (later values override)
        for (prop, value) in &other.inline_styles {
            if let Some(pos) = self.inline_styles.iter().position(|(p, _)| p == prop) {
                self.inline_styles[pos] = (prop.clone(), value.clone());
            } else {
                self.inline_styles.push((prop.clone(), value.clone()));
            }
        }
    }

    /// Generate CSS class string for Vue
    pub fn to_class_string(&self) -> String {
        self.css_classes.join(" ")
    }

    /// Generate inline style string for Vue
    pub fn to_style_string(&self) -> String {
        self.inline_styles
            .iter()
            .map(|(prop, value)| format!("{}: {}", prop, value))
            .collect::<Vec<_>>()
            .join("; ")
    }
}

/// Tailwind spacing scale (4px per unit)
pub const TW_SPACING_SCALE: &[f32] = &[
    0.0,  // 0
    4.0,  // 1
    8.0,  // 2
    12.0, // 3
    16.0, // 4
    20.0, // 5
    24.0, // 6
    28.0, // 7
    32.0, // 8
    36.0, // 9
    40.0, // 10
    44.0, // 11
    48.0, // 12
    52.0, // 13
    56.0, // 14
    60.0, // 15
    64.0, // 16
];

/// Convert Tailwind spacing index to pixels
pub fn tw_spacing(index: usize) -> f32 {
    TW_SPACING_SCALE.get(index).copied().unwrap_or_else(|| index as f32 * 4.0)
}

/// Tailwind font size scale
pub const TW_FONT_SIZE_SCALE: &[(&str, f32)] = &[
    ("xs", 12.0),
    ("sm", 14.0),
    ("base", 16.0),
    ("lg", 18.0),
    ("xl", 20.0),
    ("2xl", 24.0),
    ("3xl", 30.0),
    ("4xl", 36.0),
    ("5xl", 48.0),
    ("6xl", 60.0),
    ("7xl", 72.0),
    ("8xl", 96.0),
    ("9xl", 128.0),
];

/// Get font size by name
pub fn tw_font_size(name: &str) -> Option<f32> {
    TW_FONT_SIZE_SCALE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, size)| *size)
}

/// Tailwind border radius scale
pub const TW_RADIUS_SCALE: &[(&str, f32)] = &[
    ("none", 0.0),
    ("sm", 2.0),
    ("", 4.0),    // default
    ("md", 4.0),
    ("lg", 8.0),
    ("xl", 12.0),
    ("2xl", 16.0),
    ("3xl", 24.0),
    ("full", 9999.0),
];

/// Get border radius by name
pub fn tw_radius(name: &str) -> Option<f32> {
    TW_RADIUS_SCALE
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, r)| *r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tw_spacing() {
        assert_eq!(tw_spacing(4), 16.0);
        assert_eq!(tw_spacing(8), 32.0);
    }

    #[test]
    fn test_tw_font_size() {
        assert_eq!(tw_font_size("lg"), Some(18.0));
        assert_eq!(tw_font_size("2xl"), Some(24.0));
        assert_eq!(tw_font_size("nonexistent"), None);
    }

    #[test]
    fn test_tw_radius() {
        assert_eq!(tw_radius("lg"), Some(8.0));
        assert_eq!(tw_radius("full"), Some(9999.0));
    }

    #[test]
    fn test_resolved_style_merge() {
        let mut style = ResolvedStyle::new();
        style.css_classes.push("flex".to_string());

        let other = ResolvedStyle {
            css_classes: vec!["flex-col".to_string()],
            inline_styles: vec![],
            computed: ComputedStyle::default(),
        };

        style.merge(&other);
        assert!(style.css_classes.contains(&"flex".to_string()));
        assert!(style.css_classes.contains(&"flex-col".to_string()));
    }
}
