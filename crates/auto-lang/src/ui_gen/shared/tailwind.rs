//! Tailwind CSS Parser
//!
//! Parses Tailwind CSS class names into semantic style data structures.
//! This module is shared by all UI generators (Vue, Jet, Tauri).
//!
//! ## Supported Classes
//!
//! | Category | Examples |
//! |----------|----------|
//! | Layout | `flex`, `flex-col`, `flex-row`, `grid`, `gap-4` |
//! | Spacing | `p-4`, `px-2`, `py-4`, `m-2`, `mx-auto` |
//! | Size | `w-full`, `w-32`, `h-screen`, `h-64` |
//! | Typography | `text-lg`, `font-bold`, `text-center`, `text-white` |
//! | Background | `bg-blue-500`, `bg-gray-100` |
//! | Border | `border`, `rounded-lg`, `border-gray-300` |
//! | Effects | `shadow-lg`, `opacity-50` |

use std::collections::HashMap;

/// Parsed semantic style from Tailwind classes
#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    // Layout
    pub display: Display,
    pub flex_direction: Option<FlexDirection>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
    pub gap: Option<Dimension>,

    // Spacing
    pub padding: Spacing,
    pub margin: Spacing,

    // Size
    pub width: Size,
    pub height: Size,

    // Typography
    pub font_size: Option<Dimension>,
    pub font_weight: Option<FontWeight>,
    pub text_align: Option<TextAlign>,
    pub text_color: Option<Color>,

    // Background
    pub background_color: Option<Color>,

    // Border
    pub border_radius: Option<Dimension>,
    pub border_width: Option<Dimension>,
    pub border_color: Option<Color>,

    // Effects
    pub shadow: Option<Shadow>,
    pub opacity: Option<f32>,

    // Additional typography
    pub font_family: Option<String>,
    pub line_height: Option<Dimension>,

    // Object fit (for images)
    pub object_fit: Option<ObjectFit>,

    // Layout weight (flex children)
    pub layout_weight: Option<u32>,

    // Custom classes (not recognized)
    pub custom_classes: Vec<String>,
}

/// Display type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Display {
    #[default]
    Block,
    Flex,
    Grid,
    Inline,
    InlineFlex,
    Hidden,
    None,
}

/// Flex direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

/// Align items (cross axis)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

/// Justify content (main axis)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}

/// Generic dimension value
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    /// Fixed pixel value (e.g., 16px)
    Px(f32),
    /// Dp value (Android)
    Dp(f32),
    /// Rem value (web)
    Rem(f32),
    /// Percentage
    Percent(f32),
    /// Full (100%)
    Full,
    /// Auto
    Auto,
    /// Viewport-based
    Vw(f32),
    Vh(f32),
}

impl Dimension {
    /// Convert to dp (density-independent pixels)
    pub fn to_dp(&self) -> f32 {
        match self {
            Dimension::Px(v) => *v, // Assuming 1:1 mapping
            Dimension::Dp(v) => *v,
            Dimension::Rem(v) => v * 16.0, // 1rem = 16px default
            Dimension::Percent(_) => 0.0,  // Cannot convert directly
            Dimension::Full => 0.0,
            Dimension::Auto => 0.0,
            Dimension::Vw(_) => 0.0,
            Dimension::Vh(_) => 0.0,
        }
    }

    /// Convert to Compose dp string
    pub fn to_compose_dp(&self) -> String {
        match self {
            Dimension::Px(v) => format!("{}.dp", v),
            Dimension::Dp(v) => format!("{}.dp", v),
            Dimension::Rem(v) => format!("{}.dp", v * 16.0),
            Dimension::Percent(v) => format!("{}f", v / 100.0),
            Dimension::Full => "1f".to_string(),
            Dimension::Auto => "0f".to_string(),
            Dimension::Vw(v) => format!("{}f", v / 100.0),
            Dimension::Vh(v) => format!("{}f", v / 100.0),
        }
    }
}

/// Box spacing (padding/margin)
#[derive(Debug, Clone, Copy, Default)]
pub struct Spacing {
    pub all: Option<Dimension>,
    pub x: Option<Dimension>,
    pub y: Option<Dimension>,
    pub top: Option<Dimension>,
    pub right: Option<Dimension>,
    pub bottom: Option<Dimension>,
    pub left: Option<Dimension>,
}

impl Spacing {
    /// Check if any spacing is set
    pub fn is_empty(&self) -> bool {
        self.all.is_none()
            && self.x.is_none()
            && self.y.is_none()
            && self.top.is_none()
            && self.right.is_none()
            && self.bottom.is_none()
            && self.left.is_none()
    }

    /// Get effective top value
    pub fn top(&self) -> Option<Dimension> {
        self.top.or(self.y).or(self.all)
    }

    /// Get effective right value
    pub fn right(&self) -> Option<Dimension> {
        self.right.or(self.x).or(self.all)
    }

    /// Get effective bottom value
    pub fn bottom(&self) -> Option<Dimension> {
        self.bottom.or(self.y).or(self.all)
    }

    /// Get effective left value
    pub fn left(&self) -> Option<Dimension> {
        self.left.or(self.x).or(self.all)
    }
}

/// Size value
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Size {
    #[default]
    Auto,
    Full,
    Screen,
    Fixed(Dimension),
    Percent(f32),
    MinContent,
    MaxContent,
    FitContent,
}

/// Font weight
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontWeight {
    Thin,       // 100
    ExtraLight, // 200
    Light,      // 300
    Normal,     // 400
    Medium,     // 500
    SemiBold,   // 600
    Bold,       // 700
    ExtraBold,  // 800
    Black,      // 900
}

/// Text alignment
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
    Start,
    End,
}

/// RGBA color
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f32,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// Convert to Compose Color format
    pub fn to_compose(&self) -> String {
        format!("Color(0x{:02X}{:02X}{:02X})", self.r, self.g, self.b)
    }

    /// Convert to CSS hex format
    pub fn to_css(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

/// Shadow definition
#[derive(Debug, Clone, Copy)]
pub struct Shadow {
    pub elevation: Dimension,
}

/// Object fit mode for images
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFit {
    Contain,
    Cover,
    Fill,
    ScaleDown,
    None,
}

/// Tailwind class parser
pub struct TailwindParser {
    /// Tailwind color palette (simplified)
    colors: HashMap<&'static str, Color>,
}

impl TailwindParser {
    /// Create a new parser
    pub fn new() -> Self {
        let mut colors = HashMap::new();

        // Tailwind color palette (500 variants)
        colors.insert("slate-500", Color::rgb(100, 116, 139));
        colors.insert("gray-500", Color::rgb(107, 114, 128));
        colors.insert("zinc-500", Color::rgb(113, 113, 122));
        colors.insert("neutral-500", Color::rgb(115, 115, 115));
        colors.insert("stone-500", Color::rgb(120, 113, 108));
        colors.insert("red-500", Color::rgb(239, 68, 68));
        colors.insert("orange-500", Color::rgb(249, 115, 22));
        colors.insert("amber-500", Color::rgb(245, 158, 11));
        colors.insert("yellow-500", Color::rgb(234, 179, 8));
        colors.insert("lime-500", Color::rgb(132, 204, 22));
        colors.insert("green-500", Color::rgb(34, 197, 94));
        colors.insert("emerald-500", Color::rgb(16, 185, 129));
        colors.insert("teal-500", Color::rgb(20, 184, 166));
        colors.insert("cyan-500", Color::rgb(6, 182, 212));
        colors.insert("sky-500", Color::rgb(14, 165, 233));
        colors.insert("blue-500", Color::rgb(59, 130, 246));
        colors.insert("indigo-500", Color::rgb(99, 102, 241));
        colors.insert("violet-500", Color::rgb(139, 92, 246));
        colors.insert("purple-500", Color::rgb(168, 85, 247));
        colors.insert("fuchsia-500", Color::rgb(217, 70, 239));
        colors.insert("pink-500", Color::rgb(236, 72, 153));
        colors.insert("rose-500", Color::rgb(244, 63, 94));

        // Gray scale
        colors.insert("white", Color::rgb(255, 255, 255));
        colors.insert("black", Color::rgb(0, 0, 0));
        colors.insert("transparent", Color::rgba(0, 0, 0, 0.0));

        // Additional shades for common colors
        colors.insert("blue-100", Color::rgb(219, 234, 254));
        colors.insert("blue-200", Color::rgb(191, 219, 254));
        colors.insert("blue-300", Color::rgb(147, 197, 253));
        colors.insert("blue-400", Color::rgb(96, 165, 250));
        colors.insert("blue-600", Color::rgb(37, 99, 235));
        colors.insert("blue-700", Color::rgb(29, 78, 216));
        colors.insert("blue-800", Color::rgb(30, 64, 175));
        colors.insert("blue-900", Color::rgb(30, 58, 138));

        colors.insert("gray-100", Color::rgb(243, 244, 246));
        colors.insert("gray-200", Color::rgb(229, 231, 235));
        colors.insert("gray-300", Color::rgb(209, 213, 219));
        colors.insert("gray-400", Color::rgb(156, 163, 175));
        colors.insert("gray-600", Color::rgb(75, 85, 99));
        colors.insert("gray-700", Color::rgb(55, 65, 81));
        colors.insert("gray-800", Color::rgb(31, 41, 55));
        colors.insert("gray-900", Color::rgb(17, 24, 39));

        Self { colors }
    }

    /// Parse a class string into a ComputedStyle
    pub fn parse(&self, class_str: &str) -> ComputedStyle {
        let mut style = ComputedStyle::default();
        let classes: Vec<&str> = class_str.split_whitespace().collect();

        for class in classes {
            if !self.parse_single(class, &mut style) {
                style.custom_classes.push(class.to_string());
            }
        }

        style
    }

    /// Parse a single class, returns true if recognized
    fn parse_single(&self, class: &str, style: &mut ComputedStyle) -> bool {
        // Layout classes
        if class == "flex" {
            style.display = Display::Flex;
            return true;
        }
        if class == "inline-flex" {
            style.display = Display::InlineFlex;
            return true;
        }
        if class == "grid" {
            style.display = Display::Grid;
            return true;
        }
        if class == "block" {
            style.display = Display::Block;
            return true;
        }
        if class == "inline" {
            style.display = Display::Inline;
            return true;
        }
        if class == "hidden" {
            style.display = Display::Hidden;
            return true;
        }

        // Flex direction (also sets display: flex)
        if class == "flex-row" {
            style.display = Display::Flex;
            style.flex_direction = Some(FlexDirection::Row);
            return true;
        }
        if class == "flex-row-reverse" {
            style.display = Display::Flex;
            style.flex_direction = Some(FlexDirection::RowReverse);
            return true;
        }
        if class == "flex-col" || class == "flex-column" {
            style.display = Display::Flex;
            style.flex_direction = Some(FlexDirection::Column);
            return true;
        }
        if class == "flex-col-reverse" {
            style.display = Display::Flex;
            style.flex_direction = Some(FlexDirection::ColumnReverse);
            return true;
        }

        // Gap
        if let Some(gap) = self.parse_spacing_value(class, "gap-") {
            style.gap = Some(gap);
            return true;
        }

        // Align items (cross axis)
        if class == "items-start" {
            style.align_items = Some(AlignItems::Start);
            return true;
        }
        if class == "items-center" {
            style.align_items = Some(AlignItems::Center);
            return true;
        }
        if class == "items-end" {
            style.align_items = Some(AlignItems::End);
            return true;
        }
        if class == "items-stretch" {
            style.align_items = Some(AlignItems::Stretch);
            return true;
        }
        if class == "items-baseline" {
            style.align_items = Some(AlignItems::Baseline);
            return true;
        }

        // Justify content (main axis)
        if class == "justify-start" {
            style.justify_content = Some(JustifyContent::Start);
            return true;
        }
        if class == "justify-center" {
            style.justify_content = Some(JustifyContent::Center);
            return true;
        }
        if class == "justify-end" {
            style.justify_content = Some(JustifyContent::End);
            return true;
        }
        if class == "justify-between" {
            style.justify_content = Some(JustifyContent::Between);
            return true;
        }
        if class == "justify-around" {
            style.justify_content = Some(JustifyContent::Around);
            return true;
        }
        if class == "justify-evenly" {
            style.justify_content = Some(JustifyContent::Evenly);
            return true;
        }

        // Padding
        if let Some(val) = self.parse_spacing_value(class, "p-") {
            style.padding.all = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "px-") {
            style.padding.x = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "py-") {
            style.padding.y = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "pt-") {
            style.padding.top = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "pr-") {
            style.padding.right = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "pb-") {
            style.padding.bottom = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "pl-") {
            style.padding.left = Some(val);
            return true;
        }

        // Margin
        if let Some(val) = self.parse_spacing_value(class, "m-") {
            style.margin.all = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "mx-") {
            style.margin.x = Some(val);
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "my-") {
            style.margin.y = Some(val);
            return true;
        }
        if class == "mx-auto" {
            style.margin.x = Some(Dimension::Auto);
            return true;
        }

        // Width
        if class == "w-full" {
            style.width = Size::Full;
            return true;
        }
        if class == "w-screen" {
            style.width = Size::Screen;
            return true;
        }
        if class == "w-auto" {
            style.width = Size::Auto;
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "w-") {
            style.width = Size::Fixed(val);
            return true;
        }

        // Height
        if class == "h-full" {
            style.height = Size::Full;
            return true;
        }
        if class == "h-screen" {
            style.height = Size::Screen;
            return true;
        }
        if class == "h-auto" {
            style.height = Size::Auto;
            return true;
        }
        if let Some(val) = self.parse_spacing_value(class, "h-") {
            style.height = Size::Fixed(val);
            return true;
        }

        // Font size
        if let Some(size) = self.parse_font_size(class) {
            style.font_size = Some(size);
            return true;
        }

        // Font weight
        if let Some(weight) = self.parse_font_weight(class) {
            style.font_weight = Some(weight);
            return true;
        }

        // Text alignment
        if class == "text-left" {
            style.text_align = Some(TextAlign::Left);
            return true;
        }
        if class == "text-center" {
            style.text_align = Some(TextAlign::Center);
            return true;
        }
        if class == "text-right" {
            style.text_align = Some(TextAlign::Right);
            return true;
        }
        if class == "text-justify" {
            style.text_align = Some(TextAlign::Justify);
            return true;
        }

        // Text color
        if class.starts_with("text-") {
            let color_name = &class[5..];
            if let Some(&color) = self.colors.get(color_name) {
                style.text_color = Some(color);
                return true;
            }
        }

        // Background color
        if class.starts_with("bg-") {
            let color_name = &class[3..];
            if let Some(&color) = self.colors.get(color_name) {
                style.background_color = Some(color);
                return true;
            }
        }

        // Border radius
        if let Some(radius) = self.parse_border_radius(class) {
            style.border_radius = Some(radius);
            return true;
        }

        // Border
        if class == "border" {
            style.border_width = Some(Dimension::Px(1.0));
            return true;
        }
        if class.starts_with("border-") {
            let rest = &class[7..];
            // border-2, border-4, etc.
            if let Ok(width) = rest.parse::<f32>() {
                style.border_width = Some(Dimension::Px(width));
                return true;
            }
            // border-gray-300, etc.
            if let Some(&color) = self.colors.get(rest) {
                style.border_color = Some(color);
                return true;
            }
        }

        // Shadow
        if class == "shadow" || class == "shadow-sm" {
            style.shadow = Some(Shadow { elevation: Dimension::Dp(1.0) });
            return true;
        }
        if class == "shadow-md" {
            style.shadow = Some(Shadow { elevation: Dimension::Dp(4.0) });
            return true;
        }
        if class == "shadow-lg" {
            style.shadow = Some(Shadow { elevation: Dimension::Dp(8.0) });
            return true;
        }
        if class == "shadow-xl" {
            style.shadow = Some(Shadow { elevation: Dimension::Dp(16.0) });
            return true;
        }
        if class == "shadow-2xl" {
            style.shadow = Some(Shadow { elevation: Dimension::Dp(24.0) });
            return true;
        }

        // Opacity
        if class.starts_with("opacity-") {
            if let Ok(val) = class[8..].parse::<u32>() {
                style.opacity = Some(val as f32 / 100.0);
                return true;
            }
        }

        // Object fit
        if class == "object-contain" {
            style.object_fit = Some(ObjectFit::Contain);
            return true;
        }
        if class == "object-cover" {
            style.object_fit = Some(ObjectFit::Cover);
            return true;
        }
        if class == "object-fill" {
            style.object_fit = Some(ObjectFit::Fill);
            return true;
        }
        if class == "object-scale-down" {
            style.object_fit = Some(ObjectFit::ScaleDown);
            return true;
        }
        if class == "object-none" {
            style.object_fit = Some(ObjectFit::None);
            return true;
        }

        // Line height (leading-{n})
        if let Some(val) = self.parse_spacing_value(class, "leading-") {
            style.line_height = Some(val);
            return true;
        }

        // Layout weight (flex-{n})
        if class.starts_with("flex-") {
            if let Ok(val) = class[5..].parse::<u32>() {
                style.layout_weight = Some(val);
                return true;
            }
        }

        // Layout weight alternative (layout-weight-{n})
        if class.starts_with("layout-weight-") {
            if let Ok(val) = class[14..].parse::<u32>() {
                style.layout_weight = Some(val);
                return true;
            }
        }

        false
    }

    /// Parse a spacing value (p-4, m-2, gap-4, etc.)
    fn parse_spacing_value(&self, class: &str, prefix: &str) -> Option<Dimension> {
        if !class.starts_with(prefix) {
            return None;
        }
        let value_str = &class[prefix.len()..];

        // Handle fractional values like w-1/2
        if let Some(slash_pos) = value_str.find('/') {
            let num: f32 = value_str[..slash_pos].parse().ok()?;
            let denom: f32 = value_str[slash_pos + 1..].parse().ok()?;
            return Some(Dimension::Percent(num / denom * 100.0));
        }

        // Handle pixel values
        let value: f32 = value_str.parse().ok()?;
        Some(Dimension::Dp(value * 4.0)) // Tailwind uses 0.25rem = 4px per unit
    }

    /// Parse font size
    fn parse_font_size(&self, class: &str) -> Option<Dimension> {
        if !class.starts_with("text-") {
            return None;
        }
        let size = &class[5..];

        let dp = match size {
            "xs" => 12.0,
            "sm" => 14.0,
            "base" => 16.0,
            "lg" => 18.0,
            "xl" => 20.0,
            "2xl" => 24.0,
            "3xl" => 30.0,
            "4xl" => 36.0,
            "5xl" => 48.0,
            "6xl" => 60.0,
            "7xl" => 72.0,
            "8xl" => 96.0,
            "9xl" => 128.0,
            _ => return None,
        };
        Some(Dimension::Dp(dp))
    }

    /// Parse font weight
    fn parse_font_weight(&self, class: &str) -> Option<FontWeight> {
        if !class.starts_with("font-") {
            return None;
        }
        let weight = &class[5..];  // "font-" is 5 chars

        match weight {
            "thin" => Some(FontWeight::Thin),
            "extralight" | "lighter" => Some(FontWeight::ExtraLight),
            "light" => Some(FontWeight::Light),
            "normal" => Some(FontWeight::Normal),
            "medium" => Some(FontWeight::Medium),
            "semibold" => Some(FontWeight::SemiBold),
            "bold" => Some(FontWeight::Bold),
            "extrabold" => Some(FontWeight::ExtraBold),
            "black" | "heavy" => Some(FontWeight::Black),
            _ => None,
        }
    }

    /// Parse border radius
    fn parse_border_radius(&self, class: &str) -> Option<Dimension> {
        if class == "rounded" || class == "rounded-md" {
            return Some(Dimension::Dp(4.0));
        }
        if class == "rounded-none" {
            return Some(Dimension::Dp(0.0));
        }
        if class == "rounded-sm" {
            return Some(Dimension::Dp(2.0));
        }
        if class == "rounded-lg" {
            return Some(Dimension::Dp(8.0));
        }
        if class == "rounded-xl" {
            return Some(Dimension::Dp(12.0));
        }
        if class == "rounded-2xl" {
            return Some(Dimension::Dp(16.0));
        }
        if class == "rounded-3xl" {
            return Some(Dimension::Dp(24.0));
        }
        if class == "rounded-full" {
            return Some(Dimension::Dp(9999.0)); // Circular
        }
        None
    }
}

impl Default for TailwindParser {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed Tailwind class with semantic meaning
#[derive(Debug, Clone)]
pub enum TailwindClass {
    /// Layout class (flex, grid, etc.)
    Layout(Display),
    /// Flex direction
    FlexDirection(FlexDirection),
    /// Gap
    Gap(Dimension),
    /// Padding
    Padding(Spacing),
    /// Margin
    Margin(Spacing),
    /// Width
    Width(Size),
    /// Height
    Height(Size),
    /// Font size
    FontSize(Dimension),
    /// Font weight
    FontWeight(FontWeight),
    /// Text alignment
    TextAlign(TextAlign),
    /// Text color
    TextColor(Color),
    /// Background color
    BackgroundColor(Color),
    /// Border radius
    BorderRadius(Dimension),
    /// Border width
    BorderWidth(Dimension),
    /// Border color
    BorderColor(Color),
    /// Shadow
    Shadow(Shadow),
    /// Opacity
    Opacity(f32),
    /// Unknown/custom class
    Custom(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_layout() {
        let parser = TailwindParser::new();
        let style = parser.parse("flex flex-col gap-4");

        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.flex_direction, Some(FlexDirection::Column));
        assert_eq!(style.gap, Some(Dimension::Dp(16.0)));
    }

    #[test]
    fn test_parse_spacing() {
        let parser = TailwindParser::new();
        let style = parser.parse("p-4 px-2 py-4");

        assert_eq!(style.padding.all, Some(Dimension::Dp(16.0)));
        assert_eq!(style.padding.x, Some(Dimension::Dp(8.0)));
        assert_eq!(style.padding.y, Some(Dimension::Dp(16.0)));
    }

    #[test]
    fn test_parse_size() {
        let parser = TailwindParser::new();
        let style = parser.parse("w-full h-screen");

        assert_eq!(style.width, Size::Full);
        assert_eq!(style.height, Size::Screen);
    }

    #[test]
    fn test_parse_typography() {
        let parser = TailwindParser::new();

        // Test individual classes first
        let style_font_size = parser.parse("text-lg");
        println!("text-lg -> font_size: {:?}", style_font_size.font_size);

        let style_font_weight = parser.parse("font-bold");
        println!("font-bold -> font_weight: {:?}", style_font_weight.font_weight);
        println!("font-bold -> custom_classes: {:?}", style_font_weight.custom_classes);

        // Test direct parse_font_weight call
        let weight = parser.parse_font_weight("font-bold");
        println!("Direct parse_font_weight('font-bold'): {:?}", weight);

        // Test combined
        let style = parser.parse("text-lg font-bold text-center");
        println!("\nCombined:");
        println!("font_size: {:?}", style.font_size);
        println!("font_weight: {:?}", style.font_weight);
        println!("text_align: {:?}", style.text_align);
        println!("custom_classes: {:?}", style.custom_classes);

        assert_eq!(style.font_size, Some(Dimension::Dp(18.0)));
        assert_eq!(style.font_weight, Some(FontWeight::Bold));
        assert_eq!(style.text_align, Some(TextAlign::Center));
    }

    #[test]
    fn test_parse_colors() {
        let parser = TailwindParser::new();
        let style = parser.parse("bg-blue-500 text-white");

        assert!(style.background_color.is_some());
        assert!(style.text_color.is_some());
    }

    #[test]
    fn test_parse_border() {
        let parser = TailwindParser::new();
        let style = parser.parse("rounded-lg border border-gray-300");

        assert_eq!(style.border_radius, Some(Dimension::Dp(8.0)));
        assert_eq!(style.border_width, Some(Dimension::Px(1.0)));
        assert!(style.border_color.is_some());
    }

    #[test]
    fn test_parse_shadow() {
        let parser = TailwindParser::new();
        let style = parser.parse("shadow-lg");

        assert!(style.shadow.is_some());
    }

    #[test]
    fn test_custom_classes() {
        let parser = TailwindParser::new();
        let style = parser.parse("flex my-custom-class another-one");

        assert_eq!(style.display, Display::Flex);
        assert_eq!(style.custom_classes, vec!["my-custom-class", "another-one"]);
    }

    // ========================================================================
    // Task 3 Tests: New Tailwind Classes (object-fit, leading, flex-weight)
    // ========================================================================

    #[test]
    fn test_parse_object_fit() {
        let parser = TailwindParser::new();

        let style = parser.parse("object-contain");
        assert_eq!(style.object_fit, Some(ObjectFit::Contain));

        let style = parser.parse("object-cover");
        assert_eq!(style.object_fit, Some(ObjectFit::Cover));

        let style = parser.parse("object-fill");
        assert_eq!(style.object_fit, Some(ObjectFit::Fill));

        let style = parser.parse("object-scale-down");
        assert_eq!(style.object_fit, Some(ObjectFit::ScaleDown));

        let style = parser.parse("object-none");
        assert_eq!(style.object_fit, Some(ObjectFit::None));
    }

    #[test]
    fn test_parse_line_height() {
        let parser = TailwindParser::new();

        // leading-4 = 4 * 4 = 16dp
        let style = parser.parse("leading-4");
        assert_eq!(style.line_height, Some(Dimension::Dp(16.0)));

        // leading-6 = 6 * 4 = 24dp
        let style = parser.parse("leading-6");
        assert_eq!(style.line_height, Some(Dimension::Dp(24.0)));

        // leading-8 = 8 * 4 = 32dp
        let style = parser.parse("leading-8");
        assert_eq!(style.line_height, Some(Dimension::Dp(32.0)));
    }

    #[test]
    fn test_parse_layout_weight() {
        let parser = TailwindParser::new();

        // flex-{n} for layout weight
        let style = parser.parse("flex-1");
        assert_eq!(style.layout_weight, Some(1));

        let style = parser.parse("flex-2");
        assert_eq!(style.layout_weight, Some(2));

        let style = parser.parse("flex-3");
        assert_eq!(style.layout_weight, Some(3));

        // layout-weight-{n} alternative
        let style = parser.parse("layout-weight-1");
        assert_eq!(style.layout_weight, Some(1));

        let style = parser.parse("layout-weight-5");
        assert_eq!(style.layout_weight, Some(5));
    }

    #[test]
    fn test_parse_combined_new_classes() {
        let parser = TailwindParser::new();

        // Test combining multiple new classes
        let style = parser.parse("object-cover leading-6 flex-1");
        assert_eq!(style.object_fit, Some(ObjectFit::Cover));
        assert_eq!(style.line_height, Some(Dimension::Dp(24.0)));
        assert_eq!(style.layout_weight, Some(1));
    }
}
