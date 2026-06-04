//! # Layout Property Extractor for MCP Snapshot (Plan 281)
//!
//! Extracts box model values (padding, margin, gap, width, height, max-width)
//! from tailwind class strings and formats them as inline annotations for
//! the AURA MCP snapshot output.

use super::{SizeValue, Style, StyleClass};

/// Resolved box layout values extracted from tailwind class strings.
/// All values in pixels (from Tailwind 4px base unit). `None` = not specified.
#[derive(Debug, Clone, Default)]
pub struct BoxLayout {
    // Padding (pixels)
    pub padding_top: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,

    // Margin (pixels)
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,

    // Margin auto flags
    pub margin_left_auto: bool,
    pub margin_right_auto: bool,

    // Sizing
    pub width: Option<SizeValue>,
    pub height: Option<SizeValue>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub min_height: Option<f32>,

    // Spacing
    pub gap: Option<f32>,
}

impl BoxLayout {
    /// Extract box layout values from a parsed Style (Vec<StyleClass>).
    /// Classes are applied in order — later ones override earlier ones.
    pub fn from_style(style: &Style) -> Self {
        Self::from_classes(&style.classes)
    }

    /// Extract box layout values from a raw class string.
    /// Parses the string first, then extracts layout properties.
    /// Falls back to raw string scanning for classes not in StyleClass enum (e.g. ml-auto).
    pub fn from_class_string(class_str: &str) -> Self {
        let mut layout = match Style::parse(class_str) {
            Ok(style) => Self::from_classes(&style.classes),
            Err(_) => Self::default(),
        };

        // Scan for classes not handled by StyleClass parser
        for class in class_str.split_whitespace() {
            match class {
                "ml-auto" => {
                    layout.margin_left = Some(0.0);
                    layout.margin_left_auto = true;
                }
                "mr-auto" => {
                    layout.margin_right = Some(0.0);
                    layout.margin_right_auto = true;
                }
                "mx-auto" => {
                    layout.margin_left = Some(0.0);
                    layout.margin_right = Some(0.0);
                    layout.margin_left_auto = true;
                    layout.margin_right_auto = true;
                }
                "min-h-screen" => {
                    layout.min_height = Some(9999.0); // marker for "screen height"
                }
                _ => {
                    // Handle ml-N, mr-N, mb-N not in StyleClass enum
                    if let Some(rest) = class.strip_prefix("ml-") {
                        if let Some(px) = parse_size_to_px(rest) {
                            layout.margin_left = Some(px);
                        }
                    } else if let Some(rest) = class.strip_prefix("mr-") {
                        if let Some(px) = parse_size_to_px(rest) {
                            layout.margin_right = Some(px);
                        }
                    } else if let Some(rest) = class.strip_prefix("mb-") {
                        if let Some(px) = parse_size_to_px(rest) {
                            layout.margin_bottom = Some(px);
                        }
                    }
                }
            }
        }

        layout
    }

    /// Extract from a slice of StyleClass values.
    pub fn from_classes(classes: &[StyleClass]) -> Self {
        let mut layout = Self::default();

        for class in classes {
            match class {
                // Padding
                StyleClass::Padding(v) => {
                    let px = v.to_pixels() as f32;
                    layout.padding_top = Some(px);
                    layout.padding_right = Some(px);
                    layout.padding_bottom = Some(px);
                    layout.padding_left = Some(px);
                }
                StyleClass::PaddingX(v) => {
                    let px = v.to_pixels() as f32;
                    layout.padding_left = Some(px);
                    layout.padding_right = Some(px);
                }
                StyleClass::PaddingY(v) => {
                    let px = v.to_pixels() as f32;
                    layout.padding_top = Some(px);
                    layout.padding_bottom = Some(px);
                }
                StyleClass::PaddingTop(v) => {
                    layout.padding_top = Some(v.to_pixels() as f32);
                }
                StyleClass::PaddingBottom(v) => {
                    layout.padding_bottom = Some(v.to_pixels() as f32);
                }
                StyleClass::PaddingLeft(v) => {
                    layout.padding_left = Some(v.to_pixels() as f32);
                }
                StyleClass::PaddingRight(v) => {
                    layout.padding_right = Some(v.to_pixels() as f32);
                }

                // Margin
                StyleClass::Margin(v) => {
                    let px = v.to_pixels() as f32;
                    layout.margin_top = Some(px);
                    layout.margin_right = Some(px);
                    layout.margin_bottom = Some(px);
                    layout.margin_left = Some(px);
                }
                StyleClass::MarginX(v) => {
                    let px = v.to_pixels() as f32;
                    layout.margin_left = Some(px);
                    layout.margin_right = Some(px);
                }
                StyleClass::MarginY(v) => {
                    let px = v.to_pixels() as f32;
                    layout.margin_top = Some(px);
                    layout.margin_bottom = Some(px);
                }
                StyleClass::MarginTop(v) => {
                    layout.margin_top = Some(v.to_pixels() as f32);
                }

                // Gap
                StyleClass::Gap(v) => {
                    layout.gap = Some(v.to_pixels() as f32);
                }

                // Sizing
                StyleClass::Width(v) => {
                    layout.width = Some(*v);
                }
                StyleClass::Height(v) => {
                    layout.height = Some(*v);
                }
                StyleClass::MaxWidth(px) => {
                    layout.max_width = Some(*px);
                }
                StyleClass::MaxHeight(px) => {
                    layout.max_height = Some(*px);
                }

                // All other classes are not layout-related — skip
                _ => {}
            }
        }

        layout
    }

    /// Check if any layout property is set (non-empty).
    pub fn has_any(&self) -> bool {
        self.padding_top.is_some()
            || self.padding_right.is_some()
            || self.padding_bottom.is_some()
            || self.padding_left.is_some()
            || self.margin_top.is_some()
            || self.margin_right.is_some()
            || self.margin_bottom.is_some()
            || self.margin_left.is_some()
            || self.margin_left_auto
            || self.margin_right_auto
            || self.width.is_some()
            || self.height.is_some()
            || self.max_width.is_some()
            || self.max_height.is_some()
            || self.min_height.is_some()
            || self.gap.is_some()
    }

    /// Format as inline snapshot string (e.g. `"pad=24 w=full max-w=448 gap=8"`).
    /// Returns `None` if no layout properties are set.
    /// `viewport` is used to resolve `SizeValue::Full` to actual pixel values.
    pub fn format_inline(&self, viewport: Option<(f32, f32)>) -> Option<String> {
        if !self.has_any() {
            return None;
        }

        let mut parts: Vec<String> = Vec::new();

        // Padding: pad=N | pad=V/H | pad=T/R/B/L
        if let Some(s) = self.format_padding() {
            parts.push(format!("pad={}", s));
        }

        // Margin
        if let Some(s) = self.format_margin() {
            parts.push(format!("margin={}", s));
        }

        // Width
        if let Some(ref w) = self.width {
            parts.push(format!("w={}", format_size_value(w, viewport.map(|v| v.0))));
        }

        // Height
        if let Some(ref h) = self.height {
            parts.push(format!("h={}", format_size_value(h, viewport.map(|v| v.1))));
        }

        // Max width
        if let Some(mw) = self.max_width {
            parts.push(format!("max-w={}", mw as i32));
        }

        // Max height
        if let Some(mh) = self.max_height {
            parts.push(format!("max-h={}", mh as i32));
        }

        // Min height
        if let Some(_mh) = self.min_height {
            parts.push("min-h=screen".to_string());
        }

        // Gap
        if let Some(g) = self.gap {
            parts.push(format!("gap={}", g as i32));
        }

        if parts.is_empty() {
            None
        } else {
            Some(parts.join(" "))
        }
    }

    fn format_padding(&self) -> Option<String> {
        let t = self.padding_top?;
        let r = self.padding_right?;
        let b = self.padding_bottom?;
        let l = self.padding_left?;

        // All same
        if (t - r).abs() < 0.01 && (t - b).abs() < 0.01 && (t - l).abs() < 0.01 {
            Some(format!("{}", t as i32))
        }
        // Symmetric vertical/horizontal
        else if (t - b).abs() < 0.01 && (r - l).abs() < 0.01 {
            Some(format!("{}/{}", t as i32, r as i32))
        }
        // Full per-side
        else {
            Some(format!("{}/{}/{}/{}", t as i32, r as i32, b as i32, l as i32))
        }
    }

    fn format_margin(&self) -> Option<String> {
        // Handle auto margins
        if self.margin_left_auto && self.margin_right_auto {
            // mx-auto
            let t = self.margin_top.unwrap_or(0.0);
            let b = self.margin_bottom.unwrap_or(0.0);
            if (t - b).abs() < 0.01 && t < 0.01 {
                return Some("auto".to_string());
            }
            return Some(format!("{}/auto/{}", t as i32, b as i32));
        }
        if self.margin_left_auto {
            let t = self.margin_top.unwrap_or(0.0);
            let r = self.margin_right.unwrap_or(0.0);
            let b = self.margin_bottom.unwrap_or(0.0);
            return Some(format!("{}/{}/{}/auto", t as i32, r as i32, b as i32));
        }
        if self.margin_right_auto {
            let t = self.margin_top.unwrap_or(0.0);
            let b = self.margin_bottom.unwrap_or(0.0);
            let l = self.margin_left.unwrap_or(0.0);
            return Some(format!("{}/auto/{}/{}", t as i32, b as i32, l as i32));
        }

        let t = self.margin_top?;
        let r = self.margin_right?;
        let b = self.margin_bottom?;
        let l = self.margin_left?;

        if (t - r).abs() < 0.01 && (t - b).abs() < 0.01 && (t - l).abs() < 0.01 {
            Some(format!("{}", t as i32))
        } else if (t - b).abs() < 0.01 && (r - l).abs() < 0.01 {
            Some(format!("{}/{}", t as i32, r as i32))
        } else {
            Some(format!("{}/{}/{}/{}", t as i32, r as i32, b as i32, l as i32))
        }
    }
}

/// Format a SizeValue for inline display.
fn format_size_value(v: &SizeValue, parent_size: Option<f32>) -> String {
    match v {
        SizeValue::Full => {
            if let Some(px) = parent_size {
                format!("full({})", px as i32)
            } else {
                "full".to_string()
            }
        }
        SizeValue::Half => "half".to_string(),
        SizeValue::Auto => "auto".to_string(),
        SizeValue::Fixed(units) => (units * 4).to_string(),
        _ => format!("{:?}", v),
    }
}

/// Parse a tailwind size string (e.g. "4", "auto", "full") to pixels.
/// Returns None for unrecognized values.
fn parse_size_to_px(s: &str) -> Option<f32> {
    match s {
        "auto" | "full" | "screen" => None,
        _ => s.parse::<u16>().ok().map(|u| (u * 4) as f32),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uniform_padding() {
        let layout = BoxLayout::from_class_string("p-4");
        assert_eq!(layout.padding_top, Some(16.0));
        assert_eq!(layout.padding_right, Some(16.0));
        assert_eq!(layout.padding_bottom, Some(16.0));
        assert_eq!(layout.padding_left, Some(16.0));
        assert_eq!(layout.format_inline(None), Some("pad=16".to_string()));
    }

    #[test]
    fn test_axis_padding() {
        let layout = BoxLayout::from_class_string("p-4 px-8");
        assert_eq!(layout.padding_top, Some(16.0));
        assert_eq!(layout.padding_bottom, Some(16.0));
        assert_eq!(layout.padding_left, Some(32.0));
        assert_eq!(layout.padding_right, Some(32.0));
        assert_eq!(layout.format_inline(None), Some("pad=16/32".to_string()));
    }

    #[test]
    fn test_per_side_override() {
        let layout = BoxLayout::from_class_string("p-4 pt-2");
        assert_eq!(layout.padding_top, Some(8.0));
        assert_eq!(layout.padding_right, Some(16.0));
        assert_eq!(layout.padding_bottom, Some(16.0));
        assert_eq!(layout.padding_left, Some(16.0));
        assert_eq!(layout.format_inline(None), Some("pad=8/16/16/16".to_string()));
    }

    #[test]
    fn test_margin_extraction() {
        let layout = BoxLayout::from_class_string("m-4 mx-8");
        assert_eq!(layout.margin_top, Some(16.0));
        assert_eq!(layout.margin_bottom, Some(16.0));
        assert_eq!(layout.margin_left, Some(32.0));
        assert_eq!(layout.margin_right, Some(32.0));
    }

    #[test]
    fn test_ml_auto() {
        let layout = BoxLayout::from_class_string("ml-auto px-4 py-2");
        assert!(layout.margin_left_auto);
        assert_eq!(layout.padding_left, Some(16.0));
        assert_eq!(layout.padding_right, Some(16.0));
        assert_eq!(layout.padding_top, Some(8.0));
        assert_eq!(layout.padding_bottom, Some(8.0));
        let inline = layout.format_inline(None).unwrap();
        assert!(inline.contains("ml=auto") || inline.contains("margin="));
    }

    #[test]
    fn test_sizing() {
        let layout = BoxLayout::from_class_string("w-full h-12 gap-2 max-w-md");
        assert!(matches!(layout.width, Some(SizeValue::Full)));
        assert!(matches!(layout.height, Some(SizeValue::Fixed(3))));
        assert_eq!(layout.gap, Some(8.0));
        assert!(layout.max_width.is_some());
        let inline = layout.format_inline(Some((1600.0, 900.0))).unwrap();
        assert!(inline.contains("w=full(1600)"));
        assert!(inline.contains("gap=8"));
        assert!(inline.contains("max-w="));
    }

    #[test]
    fn test_no_layout_classes() {
        let layout = BoxLayout::from_class_string("bg-white text-sm font-bold rounded-lg");
        assert!(!layout.has_any());
        assert_eq!(layout.format_inline(None), None);
    }

    #[test]
    fn test_gap() {
        let layout = BoxLayout::from_class_string("gap-2");
        assert_eq!(layout.gap, Some(8.0));
        assert_eq!(layout.format_inline(None), Some("gap=8".to_string()));
    }

    #[test]
    fn test_margin_top() {
        let layout = BoxLayout::from_class_string("mt-4");
        assert_eq!(layout.margin_top, Some(16.0));
    }

    #[test]
    fn test_ml_numeric() {
        let layout = BoxLayout::from_class_string("ml-4");
        assert_eq!(layout.margin_left, Some(16.0));
    }
}
