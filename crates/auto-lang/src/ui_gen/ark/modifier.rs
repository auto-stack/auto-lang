//! ArkTS Modifier DSL
//!
//! Converts Tailwind CSS classes to ArkTS chainable modifiers.
//! Uses the shared TailwindParser for consistent parsing across backends.
//!
//! ## Supported Classes
//!
//! | Tailwind | ArkTS Modifier |
//! |----------|----------------|
//! | `p-4`, `px-2`, `py-4` | `.padding()` |
//! | `m-4`, `mx-auto` | `.margin()` |
//! | `w-full`, `w-32` | `.width()` |
//! | `h-full`, `h-32` | `.height()` |
//! | `text-lg`, `text-sm` | `.fontSize()` |
//! | `font-bold`, `font-medium` | `.fontWeight()` |
//! | `text-center`, `text-left` | `.textAlign()` |
//! | `text-blue-500` | `.fontColor()` |
//! | `bg-blue-500` | `.backgroundColor()` |
//! | `rounded-lg` | `.borderRadius()` |

use crate::ui_gen::shared::tailwind::{AlignItems, JustifyContent, TailwindParser, ComputedStyle, Dimension, Spacing, Size, FontWeight, TextAlign, Color, ObjectFit, BorderRadiusSpec};

use crate::ast::Type;

/// ArkTS Modifier DSL - converts Tailwind classes to ArkTS modifiers
pub struct ArkModifierDsl {
    parser: TailwindParser,
}

impl ArkModifierDsl {
    /// Create a new ArkModifierDsl instance
    pub fn new() -> Self {
        Self {
            parser: TailwindParser::new(),
        }
    }

    /// Convert a Tailwind style string to ArkTS modifiers
    pub fn convert_style(&self, style_str: &str) -> Vec<String> {
        let style = self.parser.parse(style_str);
        let mut modifiers = self.style_to_modifiers(&style);

        // Handle component-specific styles that aren't Tailwind
        // Swiper modifiers
        if style_str.contains("auto-play") || style_str.contains("autoplay") {
            modifiers.push(".autoPlay(true)".to_string());
        }
        if style_str.contains("loop") && !style_str.contains("animation-loop") {
            modifiers.push(".loop(true)".to_string());
        }
        if style_str.contains("indicator") && !style_str.contains("no-indicator") {
            modifiers.push(".indicator(true)".to_string());
        }

        modifiers
    }

    /// Convert a ComputedStyle to ArkTS modifiers
    fn style_to_modifiers(&self, style: &ComputedStyle) -> Vec<String> {
        let mut modifiers = Vec::new();

        // Padding
        if !style.padding.is_empty() {
            modifiers.push(self.spacing_to_padding(&style.padding));
        }

        // Margin
        if !style.margin.is_empty() {
            modifiers.push(self.spacing_to_margin(&style.margin));
        }

        // Width
        if style.width != Size::Auto {
            modifiers.push(self.size_to_width(&style.width));
        }

        // Height
        if style.height != Size::Auto {
            modifiers.push(self.size_to_height(&style.height));
        }

        // Font size
        if let Some(size) = &style.font_size {
            modifiers.push(self.dimension_to_font_size(size));
        }

        // Font weight
        if let Some(weight) = &style.font_weight {
            modifiers.push(self.font_weight_to_modifier(weight));
        }

        // Text align
        if let Some(align) = &style.text_align {
            modifiers.push(self.text_align_to_modifier(align));
        }

        // Text color (fontColor in ArkTS)
        if let Some(color) = &style.text_color {
            modifiers.push(self.color_to_font_color(color));
        }

        // Background color
        if let Some(color) = &style.background_color {
            modifiers.push(self.color_to_background_color(color));
        }

        // Border radius
        // Use border_radius_spec if available (supports corner-specific), otherwise fall back to border_radius
        if let Some(spec) = &style.border_radius_spec {
            modifiers.push(self.border_radius_spec_to_modifier(spec));
        } else if let Some(radius) = &style.border_radius {
            modifiers.push(self.dimension_to_border_radius(radius));
        }

        // Align items (for Column/Row containers)
        if let Some(align) = &style.align_items {
            modifiers.push(self.align_items_to_modifier(align));
        }

        // Justify content (for Column/Row containers)
        if let Some(justify) = &style.justify_content {
            modifiers.push(self.justify_content_to_modifier(justify));
        }

        // Font family
        if let Some(family) = &style.font_family {
            modifiers.push(self.font_family_to_modifier(family));
        }

        // Line height
        if let Some(height) = &style.line_height {
            modifiers.push(self.line_height_to_modifier(height));
        }

        // Object fit
        if let Some(fit) = &style.object_fit {
            modifiers.push(self.object_fit_to_modifier(fit));
        }

        // Layout weight
        if let Some(weight) = &style.layout_weight {
            modifiers.push(self.layout_weight_to_modifier(*weight as i32));
        }

        // Note: gap-* classes are NOT supported as chainable modifiers in ArkTS
        // Column/Row space must be passed as constructor parameter: Column({ space: 16 })
        // This would require special handling in the generator, so we skip gap for now

        modifiers
    }

    /// Convert Spacing to ArkTS padding modifier
    fn spacing_to_padding(&self, spacing: &Spacing) -> String {
        // If all sides are equal, use simple form
        if let Some(all) = spacing.all {
            return format!(".padding({})", self.dimension_to_value(&all));
        }

        // If x and y are set (but different), use { left, right, top, bottom }
        let top = spacing.top();
        let right = spacing.right();
        let bottom = spacing.bottom();
        let left = spacing.left();

        // Check if x and y patterns
        if spacing.x.is_some() && spacing.y.is_some() && spacing.top.is_none() && spacing.bottom.is_none() {
            let x_val = self.dimension_to_value(spacing.x.as_ref().unwrap());
            let y_val = self.dimension_to_value(spacing.y.as_ref().unwrap());
            return format!(".padding({{ left: {}, right: {}, top: {}, bottom: {} }})", x_val, x_val, y_val, y_val);
        }

        // Build individual sides
        let mut parts = Vec::new();
        if let Some(v) = top { parts.push(format!("top: {}", self.dimension_to_value(&v))); }
        if let Some(v) = right { parts.push(format!("right: {}", self.dimension_to_value(&v))); }
        if let Some(v) = bottom { parts.push(format!("bottom: {}", self.dimension_to_value(&v))); }
        if let Some(v) = left { parts.push(format!("left: {}", self.dimension_to_value(&v))); }

        if parts.is_empty() {
            String::new()
        } else if parts.len() == 1 {
            format!(".padding({})", parts[0].split(": ").nth(1).unwrap_or("0"))
        } else {
            format!(".padding({{ {} }})", parts.join(", "))
        }
    }

    /// Convert Spacing to ArkTS margin modifier
    fn spacing_to_margin(&self, spacing: &Spacing) -> String {
        // If all sides are equal, use simple form
        if let Some(all) = spacing.all {
            return format!(".margin({})", self.dimension_to_value(&all));
        }

        // Check if x is auto (mx-auto)
        if let Some(Dimension::Auto) = spacing.x {
            return ".margin({ left: 'auto', right: 'auto' })".to_string();
        }

        // If x and y patterns
        if spacing.x.is_some() && spacing.y.is_some() && spacing.top.is_none() && spacing.bottom.is_none() {
            let x_val = self.dimension_to_value(spacing.x.as_ref().unwrap());
            let y_val = self.dimension_to_value(spacing.y.as_ref().unwrap());
            return format!(".margin({{ left: {}, right: {}, top: {}, bottom: {} }})", x_val, x_val, y_val, y_val);
        }

        // Build individual sides
        let mut parts = Vec::new();
        if let Some(v) = spacing.top() { parts.push(format!("top: {}", self.dimension_to_value(&v))); }
        if let Some(v) = spacing.right() { parts.push(format!("right: {}", self.dimension_to_value(&v))); }
        if let Some(v) = spacing.bottom() { parts.push(format!("bottom: {}", self.dimension_to_value(&v))); }
        if let Some(v) = spacing.left() { parts.push(format!("left: {}", self.dimension_to_value(&v))); }

        if parts.is_empty() {
            String::new()
        } else if parts.len() == 1 {
            format!(".margin({})", parts[0].split(": ").nth(1).unwrap_or("0"))
        } else {
            format!(".margin({{ {} }})", parts.join(", "))
        }
    }

    /// Convert Size to ArkTS width modifier
    fn size_to_width(&self, size: &Size) -> String {
        match size {
            Size::Auto => String::new(),
            Size::Full => ".width('100%')".to_string(),
            Size::Screen => ".width('100%')".to_string(),
            Size::Fixed(dim) => format!(".width({})", self.dimension_to_value(dim)),
            Size::Percent(p) => format!(".width('{}%')", p),
            Size::MinContent => ".width('min-content')".to_string(),
            Size::MaxContent => ".width('max-content')".to_string(),
            Size::FitContent => ".width('fit-content')".to_string(),
        }
    }

    /// Convert Size to ArkTS height modifier
    fn size_to_height(&self, size: &Size) -> String {
        match size {
            Size::Auto => String::new(),
            Size::Full => ".height('100%')".to_string(),
            Size::Screen => ".height('100%')".to_string(),
            Size::Fixed(dim) => format!(".height({})", self.dimension_to_value(dim)),
            Size::Percent(p) => format!(".height('{}%')", p),
            Size::MinContent => ".height('min-content')".to_string(),
            Size::MaxContent => ".height('max-content')".to_string(),
            Size::FitContent => ".height('fit-content')".to_string(),
        }
    }

    /// Convert Dimension to ArkTS fontSize modifier
    fn dimension_to_font_size(&self, dim: &Dimension) -> String {
        format!(".fontSize({})", self.dimension_to_value(dim))
    }

    /// Convert Dimension to ArkTS borderRadius modifier
    fn dimension_to_border_radius(&self, dim: &Dimension) -> String {
        match dim {
            Dimension::Dp(v) if *v >= 9999.0 => ".borderRadius('50%')".to_string(), // full circle
            _ => format!(".borderRadius({})", self.dimension_to_value(dim)),
        }
    }

    /// Convert BorderRadiusSpec to ArkTS borderRadius modifier
    fn border_radius_spec_to_modifier(&self, spec: &BorderRadiusSpec) -> String {
        match spec {
            BorderRadiusSpec::All(dim) => {
                self.dimension_to_border_radius(dim)
            }
            BorderRadiusSpec::Corners { top_left, top_right, bottom_right, bottom_left } => {
                let mut parts = Vec::new();
                if let Some(v) = top_left {
                    parts.push(format!("topLeft: {}", self.dimension_to_value(v)));
                }
                if let Some(v) = top_right {
                    parts.push(format!("topRight: {}", self.dimension_to_value(v)));
                }
                if let Some(v) = bottom_right {
                    parts.push(format!("bottomRight: {}", self.dimension_to_value(v)));
                }
                if let Some(v) = bottom_left {
                    parts.push(format!("bottomLeft: {}", self.dimension_to_value(v)));
                }
                if parts.is_empty() {
                    String::new()
                } else {
                    format!(".borderRadius({{ {} }})", parts.join(", "))
                }
            }
        }
    }

    /// Convert FontWeight to ArkTS fontWeight modifier
    fn font_weight_to_modifier(&self, weight: &FontWeight) -> String {
        let ark_weight = match weight {
            FontWeight::Thin => "Lighter",
            FontWeight::ExtraLight => "Lighter",
            FontWeight::Light => "Light",
            FontWeight::Normal => "Normal",
            FontWeight::Medium => "Medium",
            FontWeight::SemiBold => "Bold",
            FontWeight::Bold => "Bold",
            FontWeight::ExtraBold => "Bolder",
            FontWeight::Black => "Bolder",
        };
        format!(".fontWeight(FontWeight.{})", ark_weight)
    }

    /// Convert TextAlign to ArkTS textAlign modifier
    fn text_align_to_modifier(&self, align: &TextAlign) -> String {
        let ark_align = match align {
            TextAlign::Left => "Start",
            TextAlign::Center => "Center",
            TextAlign::Right => "End",
            TextAlign::Justify => "Justify",
            TextAlign::Start => "Start",
            TextAlign::End => "End",
        };
        format!(".textAlign(TextAlign.{})", ark_align)
    }

    /// Convert Color to ArkTS fontColor modifier
    fn color_to_font_color(&self, color: &Color) -> String {
        format!(".fontColor('{}')", self.color_to_hex(color))
    }

    /// Convert Color to ArkTS backgroundColor modifier
    fn color_to_background_color(&self, color: &Color) -> String {
        format!(".backgroundColor('{}')", self.color_to_hex(color))
    }

    /// Convert Color to hex string
    fn color_to_hex(&self, color: &Color) -> String {
        if color.a < 1.0 {
            format!("#{:02X}{:02X}{:02X}{:02X}", color.r, color.g, color.b, (color.a * 255.0) as u8)
        } else {
            format!("#{:02X}{:02X}{:02X}", color.r, color.g, color.b)
        }
    }

    /// Convert AlignItems to ArkTS alignItems modifier
    fn align_items_to_modifier(&self, align: &AlignItems) -> String {
        let ark_align = match align {
            AlignItems::Start => "HorizontalAlign.Start",
            AlignItems::Center => "HorizontalAlign.Center",
            AlignItems::End => "HorizontalAlign.End",
            AlignItems::Stretch => "HorizontalAlign.Start", // No direct equivalent
            AlignItems::Baseline => "HorizontalAlign.Start", // No direct equivalent
        };
        format!(".alignItems({})", ark_align)
    }

    /// Convert JustifyContent to ArkTS justifyContent modifier
    fn justify_content_to_modifier(&self, justify: &JustifyContent) -> String {
        let ark_justify = match justify {
            JustifyContent::Start => "FlexAlign.Start",
            JustifyContent::Center => "FlexAlign.Center",
            JustifyContent::End => "FlexAlign.End",
            JustifyContent::Between => "FlexAlign.SpaceBetween",
            JustifyContent::Around => "FlexAlign.SpaceAround",
            JustifyContent::Evenly => "FlexAlign.SpaceEvenly",
        };
        format!(".justifyContent({})", ark_justify)
    }

    /// Convert Dimension to ArkTS value string
    fn dimension_to_value(&self, dim: &Dimension) -> String {
        match dim {
            Dimension::Px(v) => v.to_string(),
            Dimension::Dp(v) => v.to_string(),
            Dimension::Rem(v) => (v * 16.0).to_string(), // Convert rem to px-like value
            Dimension::Percent(v) => format!("'{}%'", v),
            Dimension::Full => "'100%'".to_string(),
            Dimension::Auto => "'auto'".to_string(),
            Dimension::Vw(v) => format!("'{}%'", v),
            Dimension::Vh(v) => format!("'{}%'", v),
        }
    }

    /// Convert ObjectFit to ArkTS objectFit modifier
    fn object_fit_to_modifier(&self, fit: &ObjectFit) -> String {
        let ark_fit = match fit {
            ObjectFit::Contain => "ImageFit.Contain",
            ObjectFit::Cover => "ImageFit.Cover",
            ObjectFit::Fill => "ImageFit.Fill",
            ObjectFit::ScaleDown => "ImageFit.ScaleDown",
            ObjectFit::None => "ImageFit.None",
        };
        format!(".objectFit({})", ark_fit)
    }

    /// Convert Dimension to ArkTS lineHeight modifier
    fn dimension_to_line_height(&self, dim: &Dimension) -> String {
        format!(".lineHeight({})", self.dimension_to_value(dim))
    }

    /// Convert font family string to ArkTS fontFamily modifier
    pub fn font_family_to_modifier(&self, family: &str) -> String {
        format!(".fontFamily('{}')", family)
    }

    /// Convert layout weight to ArkTS layoutWeight modifier
    pub fn layout_weight_to_modifier(&self, weight: i32) -> String {
        format!(".layoutWeight({})", weight)
    }

    /// Convert Dimension to lineHeight modifier (public API)
    pub fn line_height_to_modifier(&self, dim: &Dimension) -> String {
        self.dimension_to_line_height(dim)
    }

    /// Convert ObjectFit to objectFit modifier (public API)
    #[allow(dead_code)]
    pub fn object_fit_to_modifier_public(&self, fit: &ObjectFit) -> String {
        self.object_fit_to_modifier(fit)
    }

    /// Convert multiple styles to a single modifier string
    #[allow(dead_code)]
    pub fn convert_style_to_string(&self, style_str: &str) -> String {
        self.convert_style(style_str).join("")
    }
}

impl Default for ArkModifierDsl {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Legacy functions kept for backwards compatibility
// ============================================================================

/// Convert a style property to ArkTS modifier
pub fn style_to_modifier(key: &str, value: &str) -> Option<String> {
    match key {
        // Size modifiers
        "width" => Some(format!(".width('{}')", value)),
        "height" => Some(format!(".height('{}')", value)),

        // Text modifiers
        "fontSize" => Some(format!(".fontSize({})", value)),
        "fontWeight" => Some(format!(".fontWeight(FontWeight.{})", value)),
        "fontColor" => Some(format!(".fontColor('{}')", value)),

        // Spacing modifiers
        "margin" => Some(format!(".margin({})", value)),
        "padding" => Some(format!(".padding({})", value)),

        // Layout modifiers
        "justifyContent" => Some(format!(".justifyContent(FlexAlign.{})", value)),
        "alignItems" => Some(format!(".alignItems(HorizontalAlign.{})", value)),

        // Background
        "backgroundColor" => Some(format!(".backgroundColor('{}')", value)),

        // Border
        "borderRadius" => Some(format!(".borderRadius({})", value)),
        "borderWidth" => Some(format!(".borderWidth({})", value)),
        "borderColor" => Some(format!(".borderColor('{}')", value)),

        // Opacity
        "opacity" => Some(format!(".opacity({})", value)),

        _ => None,
    }
}

/// Convert AURA prop to ArkTS modifier
pub fn prop_to_modifier(key: &str, value: &str, _value_type: Option<&Type>) -> Option<String> {
    match key {
        // Text content
        "text" => Some(value.to_string()),

        // Style properties
        "width" | "height" | "fontSize" | "margin" | "padding" | "borderRadius"
        | "backgroundColor" | "opacity" | "borderWidth" | "borderColor" => {
            style_to_modifier(key, value)
        }

        // Event handlers
        "onclick" => Some(format!(".onClick(() => {{\n    {}\n  }})", value)),

        _ => None,
    }
}

/// Convert a single Tailwind style to ArkTS modifier (legacy, use ArkModifierDsl instead)
#[allow(dead_code)]
pub fn style_str_to_modifier(style_name: &str) -> Option<String> {
    let dsl = ArkModifierDsl::new();
    let modifiers = dsl.convert_style(style_name);
    modifiers.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_modifier() {
        let result = style_to_modifier("width", "100%");
        assert_eq!(result, Some(".width('100%')".to_string()));
    }

    #[test]
    fn test_font_size_modifier() {
        let result = style_to_modifier("fontSize", "16");
        assert_eq!(result, Some(".fontSize(16)".to_string()));
    }

    #[test]
    fn test_unknown_modifier_returns_none() {
        let result = style_to_modifier("unknown", "value");
        assert!(result.is_none());
    }

    // ========================================================================
    // ArkModifierDsl Tests
    // ========================================================================

    #[test]
    fn test_ark_dsl_padding() {
        let dsl = ArkModifierDsl::new();

        // Simple padding
        let mods = dsl.convert_style("p-4");
        assert!(mods.iter().any(|m| m.contains(".padding")));

        // Horizontal padding
        let mods = dsl.convert_style("px-4");
        assert!(mods.iter().any(|m| m.contains("left") && m.contains("right")));
    }

    #[test]
    fn test_ark_dsl_margin() {
        let dsl = ArkModifierDsl::new();

        // Simple margin
        let mods = dsl.convert_style("m-4");
        assert!(mods.iter().any(|m| m.contains(".margin")));

        // Auto margin
        let mods = dsl.convert_style("mx-auto");
        assert!(mods.iter().any(|m| m.contains("auto")));
    }

    #[test]
    fn test_ark_dsl_typography() {
        let dsl = ArkModifierDsl::new();

        // Font size
        let mods = dsl.convert_style("text-lg");
        assert!(mods.iter().any(|m| m.contains(".fontSize")));

        // Font weight
        let mods = dsl.convert_style("font-bold");
        assert!(mods.iter().any(|m| m.contains(".fontWeight") && m.contains("Bold")));

        // Text align
        let mods = dsl.convert_style("text-center");
        assert!(mods.iter().any(|m| m.contains(".textAlign") && m.contains("Center")));
    }

    #[test]
    fn test_ark_dsl_colors() {
        let dsl = ArkModifierDsl::new();

        // Text color
        let mods = dsl.convert_style("text-blue-500");
        assert!(mods.iter().any(|m| m.contains(".fontColor")));

        // Background color
        let mods = dsl.convert_style("bg-blue-500");
        assert!(mods.iter().any(|m| m.contains(".backgroundColor")));
    }

    #[test]
    fn test_ark_dsl_border_radius() {
        let dsl = ArkModifierDsl::new();

        // Rounded
        let mods = dsl.convert_style("rounded-lg");
        assert!(mods.iter().any(|m| m.contains(".borderRadius")));

        // Full circle
        let mods = dsl.convert_style("rounded-full");
        assert!(mods.iter().any(|m| m.contains("50%")));
    }

    #[test]
    fn test_ark_dsl_combined_classes() {
        let dsl = ArkModifierDsl::new();

        // Multiple classes
        let mods = dsl.convert_style("p-4 text-lg font-bold bg-blue-500 rounded-lg");
        assert!(mods.len() >= 4);

        // Check each modifier is present
        let combined = mods.join("");
        assert!(combined.contains(".padding"));
        assert!(combined.contains(".fontSize"));
        assert!(combined.contains(".fontWeight"));
        assert!(combined.contains(".backgroundColor"));
        assert!(combined.contains(".borderRadius"));
    }

    #[test]
    fn test_ark_dsl_convert_to_string() {
        let dsl = ArkModifierDsl::new();

        let result = dsl.convert_style_to_string("p-4 text-lg");
        assert!(result.contains(".padding"));
        assert!(result.contains(".fontSize"));
    }

    #[test]
    fn test_legacy_class_to_modifier() {
        // Ensure backwards compatibility
        assert!(class_to_modifier("p-4").unwrap().contains(".padding"));
        assert!(class_to_modifier("font-bold").unwrap().contains(".fontWeight"));
    }

    // ========================================================================
    // Additional Modifier Tests (Task 2 - QuickStart Sprint A)
    // ========================================================================

    #[test]
    fn test_font_family_modifier() {
        let dsl = ArkModifierDsl::new();

        let result = dsl.font_family_to_modifier("HarmonyOS Sans");
        assert_eq!(result, ".fontFamily('HarmonyOS Sans')");

        let result = dsl.font_family_to_modifier("Arial");
        assert_eq!(result, ".fontFamily('Arial')");
    }

    #[test]
    fn test_line_height_modifier() {
        let dsl = ArkModifierDsl::new();

        // With Dp value
        let result = dsl.line_height_to_modifier(&Dimension::Dp(24.0));
        assert_eq!(result, ".lineHeight(24)");

        // With Px value
        let result = dsl.line_height_to_modifier(&Dimension::Px(20.0));
        assert_eq!(result, ".lineHeight(20)");

        // With Rem value (converted to px-like)
        let result = dsl.line_height_to_modifier(&Dimension::Rem(1.5));
        assert_eq!(result, ".lineHeight(24)"); // 1.5 * 16 = 24
    }

    #[test]
    fn test_object_fit_modifier() {
        let dsl = ArkModifierDsl::new();

        let result = dsl.object_fit_to_modifier_public(&ObjectFit::Contain);
        assert_eq!(result, ".objectFit(ImageFit.Contain)");

        let result = dsl.object_fit_to_modifier_public(&ObjectFit::Cover);
        assert_eq!(result, ".objectFit(ImageFit.Cover)");

        let result = dsl.object_fit_to_modifier_public(&ObjectFit::Fill);
        assert_eq!(result, ".objectFit(ImageFit.Fill)");

        let result = dsl.object_fit_to_modifier_public(&ObjectFit::ScaleDown);
        assert_eq!(result, ".objectFit(ImageFit.ScaleDown)");

        let result = dsl.object_fit_to_modifier_public(&ObjectFit::None);
        assert_eq!(result, ".objectFit(ImageFit.None)");
    }

    #[test]
    fn test_layout_weight_modifier() {
        let dsl = ArkModifierDsl::new();

        let result = dsl.layout_weight_to_modifier(1);
        assert_eq!(result, ".layoutWeight(1)");

        let result = dsl.layout_weight_to_modifier(2);
        assert_eq!(result, ".layoutWeight(2)");

        let result = dsl.layout_weight_to_modifier(0);
        assert_eq!(result, ".layoutWeight(0)");
    }

    // ========================================================================
    // Swiper Modifier Tests (Task 8 - QuickStart Sprint A)
    // ========================================================================

    #[test]
    fn test_swiper_auto_play_modifier() {
        let dsl = ArkModifierDsl::new();

        // With hyphen
        let mods = dsl.convert_style("auto-play");
        assert!(mods.iter().any(|m| m == ".autoPlay(true)"), "Expected .autoPlay(true) for 'auto-play'");

        // Without hyphen
        let mods = dsl.convert_style("autoplay");
        assert!(mods.iter().any(|m| m == ".autoPlay(true)"), "Expected .autoPlay(true) for 'autoplay'");
    }

    #[test]
    fn test_swiper_loop_modifier() {
        let dsl = ArkModifierDsl::new();

        let mods = dsl.convert_style("loop");
        assert!(mods.iter().any(|m| m == ".loop(true)"), "Expected .loop(true) for 'loop'");
    }

    #[test]
    fn test_swiper_indicator_modifier() {
        let dsl = ArkModifierDsl::new();

        let mods = dsl.convert_style("indicator");
        assert!(mods.iter().any(|m| m == ".indicator(true)"), "Expected .indicator(true) for 'indicator'");
    }

    #[test]
    fn test_swiper_combined_modifiers() {
        let dsl = ArkModifierDsl::new();

        let mods = dsl.convert_style("auto-play loop");
        assert!(mods.iter().any(|m| m == ".autoPlay(true)"), "Expected .autoPlay(true)");
        assert!(mods.iter().any(|m| m == ".loop(true)"), "Expected .loop(true)");

        // With additional Tailwind classes
        let mods = dsl.convert_style("auto-play loop w-full h-200");
        assert!(mods.iter().any(|m| m == ".autoPlay(true)"), "Expected .autoPlay(true)");
        assert!(mods.iter().any(|m| m == ".loop(true)"), "Expected .loop(true)");
        assert!(mods.iter().any(|m| m.contains(".width")), "Expected width modifier");
        assert!(mods.iter().any(|m| m.contains(".height")), "Expected height modifier");
    }

    #[test]
    fn test_swiper_no_indicator_excluded() {
        let dsl = ArkModifierDsl::new();

        // "no-indicator" should NOT add .indicator(true)
        let mods = dsl.convert_style("no-indicator");
        assert!(!mods.iter().any(|m| m.contains("indicator")), "Should not have indicator modifier for 'no-indicator'");
    }

    #[test]
    fn test_swiper_animation_loop_not_confused() {
        let dsl = ArkModifierDsl::new();

        // "animation-loop" should NOT add .loop(true) for Swiper
        let mods = dsl.convert_style("animation-loop");
        assert!(!mods.iter().any(|m| m == ".loop(true)"), "animation-loop should not add Swiper .loop(true)");
    }
}
