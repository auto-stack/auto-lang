//! Tailwind to Compose Modifier DSL Converter
//!
//! Converts Tailwind CSS classes to Jetpack Compose Modifier chains.
//! Uses the shared TailwindParser for unified parsing across all generators.
//!
//! ## Unit Conversion
//! Tailwind unit → Dp: value * 4
//! - gap-2 → 8.dp
//! - px-4 → padding(horizontal = 16.dp)

use crate::ui_gen::shared::{Color, ComputedStyle, Dimension, Display, FlexDirection, FontWeight, Size, TailwindParser, TextAlign};

/// Tailwind class to Compose Modifier converter
pub struct ModifierDsl {
    /// Shared Tailwind parser
    parser: TailwindParser,
    /// Tailwind unit to Dp multiplier (default: 4)
    #[allow(dead_code)]
    unit_multiplier: f32,
}

/// Result of converting Tailwind classes
pub struct ModifierResult {
    /// Modifier chain components (for layout, spacing, etc.)
    pub modifiers: Vec<String>,
    /// TextStyle components (for font size, weight, color)
    pub text_style: Vec<String>,
    /// Arrangement for gap (if any)
    pub arrangement: Option<String>,
    /// Parsed computed style for additional use
    pub style: ComputedStyle,
}

impl ModifierDsl {
    /// Create a new converter
    pub fn new() -> Self {
        Self {
            parser: TailwindParser::new(),
            unit_multiplier: 4.0,
        }
    }

    /// Convert Dimension to Dp string for Compose
    fn dimension_to_dp(&self, dim: &Dimension) -> String {
        match dim {
            Dimension::Px(v) => format!("{}.dp", v),
            Dimension::Dp(v) => format!("{}.dp", v),
            Dimension::Rem(v) => format!("{}.dp", v * 16.0), // 1rem = 16px
            Dimension::Percent(v) => format!("{}f", v / 100.0),
            Dimension::Full => "1f".to_string(),
            Dimension::Auto => "0f".to_string(),
            Dimension::Vw(v) => format!("{}f", v / 100.0),
            Dimension::Vh(v) => format!("{}f", v / 100.0),
        }
    }

    /// Convert Dimension to Compose Modifier padding
    fn dimension_to_padding(&self, dim: &Dimension) -> String {
        match dim {
            Dimension::Px(v) => format!("{}.dp", v),
            Dimension::Dp(v) => format!("{}.dp", v),
            Dimension::Rem(v) => format!("{}.dp", v * 16.0),
            Dimension::Percent(_) => "0.dp".to_string(), // Can't use percent for padding
            Dimension::Full => "0.dp".to_string(),
            Dimension::Auto => "0.dp".to_string(),
            Dimension::Vw(_) => "0.dp".to_string(),
            Dimension::Vh(_) => "0.dp".to_string(),
        }
    }

    /// Convert Color to Compose Color format
    fn color_to_compose(color: &Color) -> String {
        // ARGB format: 0xAARRGGBB (AA = alpha, RR = red, GG = green, BB = blue)
        format!("Color(0xFF{:02X}{:02X}{:02X})", color.r, color.g, color.b)
    }

    /// Convert ComputedStyle to Modifier chain
    pub fn from_style(&self, style: &ComputedStyle) -> ModifierResult {
        let mut modifiers: Vec<String> = Vec::new();
        let mut text_style: Vec<String> = Vec::new();
        let mut arrangement: Option<String> = None;

        // Gap → Arrangement
        if let Some(gap) = &style.gap {
            arrangement = Some(format!("Arrangement.spacedBy({})", self.dimension_to_dp(gap)));
        }

        // Padding - check for explicit x/y (px-*, py-*) first
        if !style.padding.is_empty() {
            // Check if we have explicit x or y axis values
            let has_x = style.padding.x.is_some();
            let has_y = style.padding.y.is_some();

            if let Some(all) = &style.padding.all {
                // p-* sets all sides
                modifiers.push(format!("padding({})", self.dimension_to_padding(all)));
            } else if has_x || has_y {
                // Handle px-* / py-* combinations
                if let Some(x) = &style.padding.x {
                    modifiers.push(format!("padding(horizontal = {})", self.dimension_to_padding(x)));
                }
                if let Some(y) = &style.padding.y {
                    modifiers.push(format!("padding(vertical = {})", self.dimension_to_padding(y)));
                }
                // Also handle any explicit sides
                if let Some(t) = &style.padding.top {
                    modifiers.push(format!("padding(top = {})", self.dimension_to_padding(t)));
                }
                if let Some(b) = &style.padding.bottom {
                    modifiers.push(format!("padding(bottom = {})", self.dimension_to_padding(b)));
                }
                if let Some(l) = &style.padding.left {
                    modifiers.push(format!("padding(start = {})", self.dimension_to_padding(l)));
                }
                if let Some(r) = &style.padding.right {
                    modifiers.push(format!("padding(end = {})", self.dimension_to_padding(r)));
                }
            } else {
                // Only explicit sides (pt-*, pb-*, pl-*, pr-*)
                let top = style.padding.top();
                let bottom = style.padding.bottom();
                let left = style.padding.left();
                let right = style.padding.right();

                // Check for all four sides equal
                if left == right && top == bottom && left.is_some() && top.is_some() {
                    if let (Some(l), Some(t)) = (left, top) {
                        if self.dimension_to_padding(&l) == self.dimension_to_padding(&t) {
                            modifiers.push(format!("padding({})", self.dimension_to_padding(&l)));
                        } else {
                            modifiers.push(format!("padding(horizontal = {}, vertical = {})",
                                self.dimension_to_padding(&l), self.dimension_to_padding(&t)));
                        }
                    }
                } else if left == right && left.is_some() {
                    if let Some(l) = left {
                        modifiers.push(format!("padding(horizontal = {})", self.dimension_to_padding(&l)));
                    }
                    if let Some(t) = top {
                        modifiers.push(format!("padding(top = {})", self.dimension_to_padding(&t)));
                    }
                    if let Some(b) = bottom {
                        if Some(b) != top {
                            modifiers.push(format!("padding(bottom = {})", self.dimension_to_padding(&b)));
                        }
                    }
                } else if top == bottom && top.is_some() {
                    if let Some(t) = top {
                        modifiers.push(format!("padding(vertical = {})", self.dimension_to_padding(&t)));
                    }
                    if let Some(l) = left {
                        modifiers.push(format!("padding(start = {})", self.dimension_to_padding(&l)));
                    }
                    if let Some(r) = right {
                        if Some(r) != left {
                            modifiers.push(format!("padding(end = {})", self.dimension_to_padding(&r)));
                        }
                    }
                } else {
                    // Individual sides
                    if let Some(t) = top {
                        modifiers.push(format!("padding(top = {})", self.dimension_to_padding(&t)));
                    }
                    if let Some(b) = bottom {
                        modifiers.push(format!("padding(bottom = {})", self.dimension_to_padding(&b)));
                    }
                    if let Some(l) = left {
                        modifiers.push(format!("padding(start = {})", self.dimension_to_padding(&l)));
                    }
                    if let Some(r) = right {
                        modifiers.push(format!("padding(end = {})", self.dimension_to_padding(&r)));
                    }
                }
            }
        }

        // Margin (in Compose, outer spacing is often handled by parent or padding)
        // We'll convert margin to padding for outer containers
        if !style.margin.is_empty() {
            if let Some(all) = &style.margin.all {
                modifiers.push(format!("padding({})", self.dimension_to_padding(all)));
            } else {
                if let Some(t) = style.margin.top() {
                    modifiers.push(format!("padding(top = {})", self.dimension_to_padding(&t)));
                }
                if let Some(b) = style.margin.bottom() {
                    modifiers.push(format!("padding(bottom = {})", self.dimension_to_padding(&b)));
                }
                if let Some(l) = style.margin.left() {
                    modifiers.push(format!("padding(start = {})", self.dimension_to_padding(&l)));
                }
                if let Some(r) = style.margin.right() {
                    modifiers.push(format!("padding(end = {})", self.dimension_to_padding(&r)));
                }
            }
        }

        // Width
        match &style.width {
            Size::Full => modifiers.push("fillMaxWidth()".to_string()),
            Size::Screen => modifiers.push("fillMaxWidth()".to_string()),
            Size::Fixed(dim) => modifiers.push(format!("width({})", self.dimension_to_dp(dim))),
            Size::Percent(v) => modifiers.push(format!("fillMaxWidth({}f)", v / 100.0)),
            _ => {}
        }

        // Height
        match &style.height {
            Size::Full => modifiers.push("fillMaxHeight()".to_string()),
            Size::Screen => modifiers.push("fillMaxHeight()".to_string()),
            Size::Fixed(dim) => modifiers.push(format!("height({})", self.dimension_to_dp(dim))),
            Size::Percent(v) => modifiers.push(format!("fillMaxHeight({}f)", v / 100.0)),
            _ => {}
        }

        // Background color
        if let Some(color) = &style.background_color {
            modifiers.push(format!("background({})", Self::color_to_compose(color)));
        }

        // Border radius
        if let Some(radius) = &style.border_radius {
            if radius.to_dp() >= 9999.0 {
                modifiers.push("clip(CircleShape)".to_string());
            } else {
                modifiers.push(format!("clip(RoundedCornerShape({}))", self.dimension_to_dp(radius)));
            }
        }

        // Border width
        if let Some(width) = &style.border_width {
            modifiers.push(format!("border({})", self.dimension_to_dp(width)));
        }

        // Border color (combined with border width)
        if let Some(color) = &style.border_color {
            if style.border_width.is_some() {
                // Already added border, update last modifier to include color
                if let Some(last) = modifiers.last_mut() {
                    if last.starts_with("border(") {
                        *last = format!("border(1.dp, {})", Self::color_to_compose(color));
                    }
                }
            } else {
                modifiers.push(format!("border(1.dp, {})", Self::color_to_compose(color)));
            }
        }

        // Shadow
        if let Some(shadow) = &style.shadow {
            modifiers.push(format!("shadow({})", self.dimension_to_dp(&shadow.elevation)));
        }

        // Opacity
        if let Some(opacity) = style.opacity {
            modifiers.push(format!("alpha({:.2}f)", opacity));
        }

        // Layout weight (flex-1 → weight(1f))
        if let Some(weight) = style.layout_weight {
            modifiers.push(format!("weight({}f)", weight));
        }

        // Text color (goes into TextStyle, not Modifier)
        if let Some(color) = &style.text_color {
            text_style.push(format!("color = {}", Self::color_to_compose(color)));
        }

        // Font size (goes into TextStyle, not Modifier)
        if let Some(size) = &style.font_size {
            let sp = size.to_dp();
            text_style.push(format!("fontSize = {}.sp", sp));
        }

        // Font weight (goes into TextStyle)
        if let Some(weight) = &style.font_weight {
            let weight_str = match weight {
                FontWeight::Thin => "FontWeight.Thin",
                FontWeight::ExtraLight => "FontWeight.ExtraLight",
                FontWeight::Light => "FontWeight.Light",
                FontWeight::Normal => "FontWeight.Normal",
                FontWeight::Medium => "FontWeight.Medium",
                FontWeight::SemiBold => "FontWeight.SemiBold",
                FontWeight::Bold => "FontWeight.Bold",
                FontWeight::ExtraBold => "FontWeight.ExtraBold",
                FontWeight::Black => "FontWeight.Black",
            };
            text_style.push(format!("fontWeight = {}", weight_str));
        }

        // Text align (goes into TextStyle)
        if let Some(align) = &style.text_align {
            let align_str = match align {
                TextAlign::Left | TextAlign::Start => "TextAlign.Start",
                TextAlign::Center => "TextAlign.Center",
                TextAlign::Right | TextAlign::End => "TextAlign.End",
                TextAlign::Justify => "TextAlign.Justify",
            };
            text_style.push(format!("textAlign = {}", align_str));
        }

        ModifierResult {
            modifiers,
            text_style,
            arrangement,
            style: style.clone(),
        }
    }

    /// Convert full class string to ModifierResult
    pub fn convert_class(&self, class: &str) -> ModifierResult {
        let style = self.parser.parse(class);
        self.from_style(&style)
    }

    /// Generate Modifier chain code
    pub fn generate_modifier_chain(&self, class: &str) -> String {
        let result = self.convert_class(class);
        if result.modifiers.is_empty() {
            "Modifier".to_string()
        } else {
            format!("Modifier.{}", result.modifiers.join("."))
        }
    }

    /// Generate TextStyle code from class (for Text components)
    pub fn generate_text_style(&self, class: &str, base_style: Option<&str>) -> Option<String> {
        let result = self.convert_class(class);
        if result.text_style.is_empty() && base_style.is_none() {
            return None;
        }

        // Build TextStyle with base style and overrides
        let mut style_parts = Vec::new();

        // If we have a base style (like MaterialTheme.typography.headlineLarge), use it
        if let Some(base) = base_style {
            style_parts.push(base.to_string());
        }

        // Add text style overrides
        style_parts.extend(result.text_style);

        if style_parts.is_empty() {
            None
        } else {
            Some(style_parts.join(", "))
        }
    }

    /// Get the display type from classes (for component selection)
    pub fn get_display_type(&self, class: &str) -> Display {
        let style = self.parser.parse(class);
        style.display
    }

    /// Get the flex direction from classes
    pub fn get_flex_direction(&self, class: &str) -> Option<FlexDirection> {
        let style = self.parser.parse(class);
        style.flex_direction
    }

    /// Get the ComputedStyle from class string
    pub fn parse(&self, class: &str) -> ComputedStyle {
        self.parser.parse(class)
    }

    /// Convert a single Tailwind class to Modifier code (for backward compatibility)
    pub fn convert_single(&self, class: &str) -> Option<String> {
        let style = self.parser.parse(class);
        let result = self.from_style(&style);
        result.modifiers.into_iter().next()
    }
}

impl Default for ModifierDsl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("px-4 py-2");
        assert!(result.modifiers.iter().any(|m| m.contains("padding(horizontal")));
        assert!(result.modifiers.iter().any(|m| m.contains("padding(vertical")));
    }

    #[test]
    fn test_gap_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("gap-4");
        assert!(result.arrangement.is_some());
        assert!(result.arrangement.unwrap().contains("Arrangement.spacedBy"));
    }

    #[test]
    fn test_fill_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("w-full h-full");
        assert!(result.modifiers.iter().any(|m| m.contains("fillMaxWidth()")));
        assert!(result.modifiers.iter().any(|m| m.contains("fillMaxHeight()")));
    }

    #[test]
    fn test_rounded_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("rounded-lg");
        assert!(result.modifiers.iter().any(|m| m.contains("clip(RoundedCornerShape")));
    }

    #[test]
    fn test_background_color() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("bg-blue-500");
        assert!(result.modifiers.iter().any(|m| m.contains("background(Color(")));
    }

    #[test]
    fn test_shadow_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("shadow-lg");
        assert!(result.modifiers.iter().any(|m| m.contains("shadow")));
    }

    #[test]
    fn test_modifier_chain_generation() {
        let dsl = ModifierDsl::new();
        let chain = dsl.generate_modifier_chain("px-4 rounded-lg");
        assert!(chain.starts_with("Modifier."));
        assert!(chain.contains("padding"));
        assert!(chain.contains("RoundedCornerShape"));
    }

    #[test]
    fn test_display_type_detection() {
        let dsl = ModifierDsl::new();

        assert_eq!(dsl.get_display_type("flex"), Display::Flex);
        assert_eq!(dsl.get_display_type("flex-col"), Display::Flex);
        assert_eq!(dsl.get_display_type("grid"), Display::Grid);
        assert_eq!(dsl.get_display_type("block"), Display::Block);
        assert_eq!(dsl.get_display_type("hidden"), Display::Hidden);
    }

    #[test]
    fn test_flex_direction_detection() {
        let dsl = ModifierDsl::new();

        assert_eq!(dsl.get_flex_direction("flex-row"), Some(FlexDirection::Row));
        assert_eq!(dsl.get_flex_direction("flex-col"), Some(FlexDirection::Column));
        assert_eq!(dsl.get_flex_direction("flex"), None);
    }

    #[test]
    fn test_combined_modifiers() {
        let dsl = ModifierDsl::new();

        let result = dsl.convert_class("px-4 py-2 rounded-lg bg-blue-500 opacity-90");
        assert!(result.modifiers.iter().any(|m| m.contains("padding")));
        assert!(result.modifiers.iter().any(|m| m.contains("RoundedCornerShape")));
        assert!(result.modifiers.iter().any(|m| m.contains("background")));
        assert!(result.modifiers.iter().any(|m| m.contains("alpha")));
    }

    #[test]
    fn test_circle_shape() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("rounded-full");
        assert!(result.modifiers.iter().any(|m| m.contains("CircleShape")));
    }

    #[test]
    fn test_layout_weight_conversion() {
        let dsl = ModifierDsl::new();

        // flex-1 → weight(1f)
        let result = dsl.convert_class("flex-1");
        println!("flex-1 modifiers: {:?}", result.modifiers);
        assert!(result.modifiers.iter().any(|m| m.contains("weight(1f)")));

        // flex-2 → weight(2f)
        let result = dsl.convert_class("flex-2");
        println!("flex-2 modifiers: {:?}", result.modifiers);
        assert!(result.modifiers.iter().any(|m| m.contains("weight(2f)")));

        // Test generate_modifier_chain
        let chain = dsl.generate_modifier_chain("flex-1");
        println!("flex-1 chain: {}", chain);
        assert_eq!(chain, "Modifier.weight(1f)");

        // Test with combined classes
        let chain = dsl.generate_modifier_chain("flex-1 p-2");
        println!("flex-1 p-2 chain: {}", chain);
        assert!(chain.contains("weight(1f)"));
    }

    #[test]
    fn test_table_cell_style() {
        // Test the style pattern used in table.at
        let dsl = ModifierDsl::new();

        // Gap in Row parent
        let result = dsl.convert_class("gap-4");
        assert!(result.arrangement.is_some());
        assert!(result.arrangement.unwrap().contains("spacedBy(16.dp)"));

        // flex-1 in child Text
        let result = dsl.convert_class("flex-1");
        assert!(result.modifiers.iter().any(|m| m.contains("weight(1f)")));
    }
}
