// StyleClass - Intermediate Representation (IR) for style classes
//
// This enum represents the parsed form of Tailwind-style utility classes.
// It is backend-agnostic and can be translated to GPUI, Iced, or other backends.

use super::Color;

/// Size value (used for width, height, spacing, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SizeValue {
    Full,        // 100%
    Half,        // 50%
    Third,       // 33.333%
    TwoThirds,   // 66.666%
    Quarter,     // 25%
    ThreeQuarters, // 75%
    Auto,
    Fixed(u16),  // Pixels (Tailwind uses 4px base unit, so 1 = 4px, 2 = 8px, etc.)
}

impl SizeValue {
    /// Convert Tailwind spacing unit to pixels (1 unit = 4px)
    pub fn to_pixels(&self) -> u16 {
        match self {
            SizeValue::Fixed(units) => units * 4,
            _ => 0, // Full, Auto, etc. are handled differently by backends
        }
    }
}

/// Gradient direction for bg-gradient-to-{dir}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientDir {
    ToR,
    ToL,
    ToT,
    ToB,
    ToBR,
    ToBL,
    ToTR,
    ToTL,
}

/// Style class IR - represents a single parsed style property
///
/// This enum contains L1 Core + L2 Important features:
/// - Spacing: p-*, px-*, py-*, m-*, mx-*, my-*, gap-*
/// - Colors: bg-*, text-*
/// - Layout: flex, flex-1, flex-row/col, items-*, justify-*
/// - Sizing: w-full, w-*, h-full, h-*
/// - Border Radius: rounded, rounded-*
/// - Border: border, border-{color}
/// - Typography: text-*, font-*
#[derive(Debug, Clone, PartialEq)]
pub enum StyleClass {
    // ========== Spacing (L1 Core + L2) ==========
    /// Padding: p-{0-12} (p-0, p-1, ..., p-12)
    Padding(SizeValue),

    /// Padding X: px-{0-12} (L2)
    PaddingX(SizeValue),

    /// Padding Y: py-{0-12} (L2)
    PaddingY(SizeValue),

    /// Padding Top: pt-{0-12}
    PaddingTop(SizeValue),

    /// Padding Bottom: pb-{0-12}
    PaddingBottom(SizeValue),

    /// Padding Left: pl-{0-12}
    PaddingLeft(SizeValue),

    /// Padding Right: pr-{0-12}
    PaddingRight(SizeValue),

    /// Margin: m-{0-12} (L2) - Note: Iced doesn't support margin
    Margin(SizeValue),

    /// Margin X: mx-{0-12} (L2)
    MarginX(SizeValue),

    /// Margin Y: my-{0-12} (L2)
    MarginY(SizeValue),

    /// Margin Top: mt-{0-12} (L2)
    MarginTop(SizeValue),

    /// Margin Left Auto: ml-auto — push element to the right in a row
    MarginLeftAuto,

    /// Margin Right Auto: mr-auto — push element to the left in a row
    MarginRightAuto,

    /// Gap: gap-{0-12} (gap-0, gap-1, ..., gap-12)
    Gap(SizeValue),

    // ========== Colors (L1 Core) ==========
    /// Background color: bg-{color}
    BackgroundColor(Color),

    /// Gradient direction marker: bg-gradient-to-{dir}
    BgGradient(GradientDir),

    /// Gradient start color: from-{color}
    GradientFrom(Color),

    /// Gradient end color: to-{color}
    GradientTo(Color),

    /// Text color: text-{color}
    TextColor(Color),

    // ========== Layout (L1 Core + L2) ==========
    /// Flex container
    Flex,

    /// Flex: 1 (grow to fill space) - L2
    Flex1,

    /// Flex direction: row (default)
    FlexRow,

    /// Flex direction: column
    FlexCol,

    /// Items center alignment
    ItemsCenter,

    /// Items start alignment - L2
    ItemsStart,

    /// Items end alignment - L2
    ItemsEnd,

    /// Justify center
    JustifyCenter,

    /// Justify between
    JustifyBetween,

    /// Justify start - L2
    JustifyStart,

    /// Justify end - L2
    JustifyEnd,

    // ========== Sizing (L1 Core) ==========
    /// Width: w-{size}
    Width(SizeValue),

    /// Height: h-{size}
    Height(SizeValue),

    // ========== Max Sizing (L1 Core) ==========
    /// Max width: max-w-{named|numeric} (pixels)
    MaxWidth(f32),

    /// Max height: max-h-{named|numeric} (pixels)
    MaxHeight(f32),

    // ========== Border Radius (L1 Core + L2) ==========
    /// Border radius: rounded (default)
    Rounded,

    /// Border radius: rounded-sm (L2)
    RoundedSm,

    /// Border radius: rounded-md (L2)
    RoundedMd,

    /// Border radius: rounded-lg (L2)
    RoundedLg,

    /// Border radius: rounded-xl (L2)
    RoundedXl,

    /// Border radius: rounded-2xl (L2)
    Rounded2Xl,

    /// Border radius: rounded-3xl (L2)
    Rounded3Xl,

    /// Border radius: rounded-full (L2)
    RoundedFull,

    // ========== Border (L2) ==========
    /// Border: border (default width and color)
    Border,

    /// Border: 0 (no border) - L2
    Border0,

    /// Border width: border-N (numeric pixels) - L2
    BorderWidth(f32),

    /// Border color: border-{color} - L2
    BorderColor(Color),

    // ========== Typography (L2) ==========
    /// Font size: text-xs (12px) - L2
    TextXs,

    /// Font size: text-sm (14px) - L2
    TextSm,

    /// Font size: text-base (16px) - L2
    TextBase,

    /// Font size: text-lg (18px) - L2
    TextLg,

    /// Font size: text-xl (20px) - L2
    TextXl,

    /// Font size: text-2xl (24px) - L2
    Text2Xl,

    /// Font size: text-3xl (30px) - L2
    Text3Xl,

    /// Font size: text-4xl (36px) - L2
    Text4Xl,

    /// Font weight: font-bold (L2)
    FontBold,

    /// Font weight: font-medium (L2)
    FontMedium,

    /// Font weight: font-normal (L2)
    FontNormal,

    /// Text alignment: text-center (L2)
    TextCenter,

    /// Text alignment: text-left (L2)
    TextLeft,

    /// Text alignment: text-right (L2)
    TextRight,

    // ========== Effects (L3 Advanced) ==========
    /// Shadow: shadow (default) - L3
    Shadow,

    /// Shadow: shadow-sm - L3
    ShadowSm,

    /// Shadow: shadow-md - L3
    ShadowMd,

    /// Shadow: shadow-lg - L3
    ShadowLg,

    /// Shadow: shadow-xl - L3
    ShadowXl,

    /// Shadow: shadow-2xl - L3
    Shadow2Xl,

    /// Shadow: shadow-none - L3
    ShadowNone,

    /// Opacity: opacity-{0-100} - L3
    Opacity(u8),

    // ========== Position (L3 Advanced) ==========
    /// Position: relative - L3
    Relative,

    /// Position: absolute - L3 (Note: Iced doesn't support absolute positioning)
    Absolute,

    /// Z-index: z-{0-50} - L3
    ZIndex(i16),

    // ========== Overflow (L3 Advanced) ==========
    /// Overflow: overflow-auto - L3
    OverflowAuto,

    /// Overflow: overflow-hidden - L3
    OverflowHidden,

    /// Overflow: overflow-visible - L3
    OverflowVisible,

    /// Overflow: overflow-scroll - L3
    OverflowScroll,

    /// Overflow X: overflow-x-auto - L3
    OverflowXAuto,

    /// Overflow Y: overflow-y-auto - L3
    OverflowYAuto,

    // ========== Grid (L3 Advanced) ==========
    /// Display: grid - L3 (Note: Iced doesn't support grid)
    Grid,

    /// Grid columns: grid-cols-{1-12} - L3
    GridCols(u8),

    /// Grid rows: grid-rows-{1-6} - L3
    GridRows(u8),

    /// Grid column: col-span-{1-12} - L3
    ColSpan(u8),

    /// Grid row: row-span-{1-6} - L3
    RowSpan(u8),

    /// Grid column start: col-start-{1-7} - L3
    ColStart(u8),

    /// Grid row start: row-start-{1-7} - L3
    RowStart(u8),
}

impl StyleClass {
    /// Parse a single style class string into a StyleClass
    pub fn parse_single(class: &str) -> Result<Self, String> {
        let class = class.trim();

        // Skip empty strings
        if class.is_empty() {
            return Err("Empty style class".to_string());
        }

        // ========== Spacing (L1 + L2) ==========

        // Parse padding: p-{0-12}
        if let Some(rest) = class.strip_prefix("p-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Padding(size));
        }

        // Parse padding X: px-{0-12}
        if let Some(rest) = class.strip_prefix("px-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::PaddingX(size));
        }

        // Parse padding Y: py-{0-12}
        if let Some(rest) = class.strip_prefix("py-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::PaddingY(size));
        }

        // Parse per-side padding: pt/pb/pl/pr
        if let Some(rest) = class.strip_prefix("pt-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::PaddingTop(size));
        }
        if let Some(rest) = class.strip_prefix("pb-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::PaddingBottom(size));
        }
        if let Some(rest) = class.strip_prefix("pl-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::PaddingLeft(size));
        }
        if let Some(rest) = class.strip_prefix("pr-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::PaddingRight(size));
        }

        // Parse margin: m-{0-12}
        if let Some(rest) = class.strip_prefix("m-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Margin(size));
        }

        // Parse margin X: mx-{0-12}
        if let Some(rest) = class.strip_prefix("mx-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::MarginX(size));
        }

        // Parse margin Y: my-{0-12}
        if let Some(rest) = class.strip_prefix("my-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::MarginY(size));
        }

        // Parse margin top: mt-{0-12}
        if let Some(rest) = class.strip_prefix("mt-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::MarginTop(size));
        }

        // Parse margin auto classes
        if class == "ml-auto" {
            return Ok(StyleClass::MarginLeftAuto);
        }
        if class == "mr-auto" {
            return Ok(StyleClass::MarginRightAuto);
        }
        if class == "mx-auto" {
            // mx-auto = both ml-auto and mr-auto (handled as a pair)
            // We only emit MarginLeftAuto here; the adapter will set both
            return Ok(StyleClass::MarginLeftAuto);
        }

        // Parse gap: gap-{0-12}
        if let Some(rest) = class.strip_prefix("gap-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Gap(size));
        }

        // ========== Colors (L1) ==========

        // Parse background: bg-{color}
        if let Some(color_name) = class.strip_prefix("bg-") {
            // Handle gradient markers
            let dir = match color_name {
                "gradient-to-r" => Some(GradientDir::ToR),
                "gradient-to-l" => Some(GradientDir::ToL),
                "gradient-to-t" => Some(GradientDir::ToT),
                "gradient-to-b" => Some(GradientDir::ToB),
                "gradient-to-br" => Some(GradientDir::ToBR),
                "gradient-to-bl" => Some(GradientDir::ToBL),
                "gradient-to-tr" => Some(GradientDir::ToTR),
                "gradient-to-tl" => Some(GradientDir::ToTL),
                _ => None,
            };
            if let Some(d) = dir {
                return Ok(StyleClass::BgGradient(d));
            }
            let color = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))?;
            return Ok(StyleClass::BackgroundColor(color));
        }

        // Parse gradient start: from-{color}
        if let Some(color_name) = class.strip_prefix("from-") {
            if let Ok(color) = Color::from_tailwind(color_name).or_else(|_| Color::from_hex(color_name)) {
                return Ok(StyleClass::GradientFrom(color));
            }
        }

        // Parse gradient end: to-{color}
        if let Some(color_name) = class.strip_prefix("to-") {
            if let Ok(color) = Color::from_tailwind(color_name).or_else(|_| Color::from_hex(color_name)) {
                return Ok(StyleClass::GradientTo(color));
            }
        }

        // ========== Typography (L2) ==========

        // Parse text size: text-{xs,sm,base,lg,xl,2xl,3xl}
        match class {
            "text-xs" => return Ok(StyleClass::TextXs),
            "text-sm" => return Ok(StyleClass::TextSm),
            "text-base" => return Ok(StyleClass::TextBase),
            "text-lg" => return Ok(StyleClass::TextLg),
            "text-xl" => return Ok(StyleClass::TextXl),
            "text-2xl" => return Ok(StyleClass::Text2Xl),
            "text-3xl" => return Ok(StyleClass::Text3Xl),
            "text-4xl" => return Ok(StyleClass::Text4Xl),
            _ => {}
        }

        // Parse font weight
        match class {
            "font-bold" => return Ok(StyleClass::FontBold),
            "font-medium" => return Ok(StyleClass::FontMedium),
            "font-normal" => return Ok(StyleClass::FontNormal),
            _ => {}
        }

        // Parse text alignment
        match class {
            "text-center" => return Ok(StyleClass::TextCenter),
            "text-left" => return Ok(StyleClass::TextLeft),
            "text-right" => return Ok(StyleClass::TextRight),
            _ => {}
        }

        // Parse text color: text-{color} (must come after text-size/align)
        if let Some(color_name) = class.strip_prefix("text-") {
            let color = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))?;
            return Ok(StyleClass::TextColor(color));
        }

        // ========== Layout (L1 + L2) ==========

        // Parse flex
        if class == "flex" {
            return Ok(StyleClass::Flex);
        }

        // Parse flex-1
        if class == "flex-1" {
            return Ok(StyleClass::Flex1);
        }

        // Parse flex-row
        if class == "flex-row" {
            return Ok(StyleClass::FlexRow);
        }

        // Parse flex-col
        if class == "flex-col" {
            return Ok(StyleClass::FlexCol);
        }

        // Parse items-*
        match class {
            "items-center" => return Ok(StyleClass::ItemsCenter),
            "items-start" => return Ok(StyleClass::ItemsStart),
            "items-end" => return Ok(StyleClass::ItemsEnd),
            _ => {}
        }

        // Parse justify-*
        match class {
            "justify-center" => return Ok(StyleClass::JustifyCenter),
            "justify-between" => return Ok(StyleClass::JustifyBetween),
            "justify-start" => return Ok(StyleClass::JustifyStart),
            "justify-end" => return Ok(StyleClass::JustifyEnd),
            _ => {}
        }

        // ========== Sizing (L1) ==========

        // Parse width: w-{size}
        if let Some(rest) = class.strip_prefix("w-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Width(size));
        }

        // Parse height: h-{size}
        if let Some(rest) = class.strip_prefix("h-") {
            let size = parse_size_value(rest)?;
            return Ok(StyleClass::Height(size));
        }

        // ========== Max Sizing (L1) ==========

        // Parse max-width: max-w-{named|numeric}
        if let Some(rest) = class.strip_prefix("max-w-") {
            if let Some(px) = parse_max_size_value(rest) {
                return Ok(StyleClass::MaxWidth(px));
            }
        }

        // Parse max-height: max-h-{named|numeric}
        if let Some(rest) = class.strip_prefix("max-h-") {
            if let Some(px) = parse_max_size_value(rest) {
                return Ok(StyleClass::MaxHeight(px));
            }
        }

        // ========== Border Radius (L1 + L2) ==========

        // Parse rounded-*
        match class {
            "rounded" => return Ok(StyleClass::Rounded),
            "rounded-sm" => return Ok(StyleClass::RoundedSm),
            "rounded-md" => return Ok(StyleClass::RoundedMd),
            "rounded-lg" => return Ok(StyleClass::RoundedLg),
            "rounded-xl" => return Ok(StyleClass::RoundedXl),
            "rounded-2xl" => return Ok(StyleClass::Rounded2Xl),
            "rounded-3xl" => return Ok(StyleClass::Rounded3Xl),
            "rounded-full" => return Ok(StyleClass::RoundedFull),
            _ => {}
        }

        // ========== Border (L2) ==========

        // Parse border
        if class == "border" {
            return Ok(StyleClass::Border);
        }

        // Parse border-0
        if class == "border-0" {
            return Ok(StyleClass::Border0);
        }

        // Parse border-N (numeric width, e.g. border-2, border-4)
        if let Some(rest) = class.strip_prefix("border-") {
            if let Ok(width) = rest.parse::<f32>() {
                return Ok(StyleClass::BorderWidth(width));
            }
        }

        // Parse border color: border-{color}
        if let Some(color_name) = class.strip_prefix("border-") {
            // Skip border-0 which we already handled
            if color_name == "0" {
                return Ok(StyleClass::Border0);
            }
            let color = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))?;
            return Ok(StyleClass::BorderColor(color));
        }

        // ========== Effects (L3) ==========

        // Parse shadow variants
        match class {
            "shadow" => return Ok(StyleClass::Shadow),
            "shadow-sm" => return Ok(StyleClass::ShadowSm),
            "shadow-md" => return Ok(StyleClass::ShadowMd),
            "shadow-lg" => return Ok(StyleClass::ShadowLg),
            "shadow-xl" => return Ok(StyleClass::ShadowXl),
            "shadow-2xl" => return Ok(StyleClass::Shadow2Xl),
            "shadow-none" => return Ok(StyleClass::ShadowNone),
            _ => {}
        }

        // Parse opacity: opacity-{0-100}
        if let Some(rest) = class.strip_prefix("opacity-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid opacity value: {}", rest))?;
            if value > 100 {
                return Err(format!("Opacity value must be 0-100, got: {}", value));
            }
            return Ok(StyleClass::Opacity(value));
        }

        // ========== Position (L3) ==========

        // Parse position
        match class {
            "relative" => return Ok(StyleClass::Relative),
            "absolute" => return Ok(StyleClass::Absolute),
            _ => {}
        }

        // Parse z-index: z-{0-50}
        if let Some(rest) = class.strip_prefix("z-") {
            // Handle z-{0}, z-10, z-20, z-50, etc.
            let value: i16 = rest.parse()
                .map_err(|_| format!("Invalid z-index value: {}", rest))?;
            if value < 0 || value > 50 {
                return Err(format!("Z-index value must be 0-50, got: {}", value));
            }
            return Ok(StyleClass::ZIndex(value));
        }

        // ========== Overflow (L3) ==========

        // Parse overflow variants
        match class {
            "overflow-auto" => return Ok(StyleClass::OverflowAuto),
            "overflow-hidden" => return Ok(StyleClass::OverflowHidden),
            "overflow-visible" => return Ok(StyleClass::OverflowVisible),
            "overflow-scroll" => return Ok(StyleClass::OverflowScroll),
            "overflow-x-auto" => return Ok(StyleClass::OverflowXAuto),
            "overflow-y-auto" => return Ok(StyleClass::OverflowYAuto),
            _ => {}
        }

        // ========== Grid (L3) ==========

        // Parse grid
        if class == "grid" {
            return Ok(StyleClass::Grid);
        }

        // Parse grid-cols-{1-12}
        if let Some(rest) = class.strip_prefix("grid-cols-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid grid-cols value: {}", rest))?;
            if value < 1 || value > 12 {
                return Err(format!("Grid columns must be 1-12, got: {}", value));
            }
            return Ok(StyleClass::GridCols(value));
        }

        // Parse grid-rows-{1-6}
        if let Some(rest) = class.strip_prefix("grid-rows-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid grid-rows value: {}", rest))?;
            if value < 1 || value > 6 {
                return Err(format!("Grid rows must be 1-6, got: {}", value));
            }
            return Ok(StyleClass::GridRows(value));
        }

        // Parse col-span-{1-12}
        if let Some(rest) = class.strip_prefix("col-span-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid col-span value: {}", rest))?;
            if value < 1 || value > 12 {
                return Err(format!("Column span must be 1-12, got: {}", value));
            }
            return Ok(StyleClass::ColSpan(value));
        }

        // Parse row-span-{1-6}
        if let Some(rest) = class.strip_prefix("row-span-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid row-span value: {}", rest))?;
            if value < 1 || value > 6 {
                return Err(format!("Row span must be 1-6, got: {}", value));
            }
            return Ok(StyleClass::RowSpan(value));
        }

        // Parse col-start-{1-7}
        if let Some(rest) = class.strip_prefix("col-start-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid col-start value: {}", rest))?;
            if value < 1 || value > 7 {
                return Err(format!("Column start must be 1-7, got: {}", value));
            }
            return Ok(StyleClass::ColStart(value));
        }

        // Parse row-start-{1-7}
        if let Some(rest) = class.strip_prefix("row-start-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid row-start value: {}", rest))?;
            if value < 1 || value > 7 {
                return Err(format!("Row start must be 1-7, got: {}", value));
            }
            return Ok(StyleClass::RowStart(value));
        }

        Err(format!("Unknown style class: {}", class))
    }
}

/// Helper function to parse size values
fn parse_size_value(input: &str) -> Result<SizeValue, String> {
    match input {
        "full" => Ok(SizeValue::Full),
        "auto" => Ok(SizeValue::Auto),
        "1/2" => Ok(SizeValue::Half),
        "1/3" => Ok(SizeValue::Third),
        "2/3" => Ok(SizeValue::TwoThirds),
        "1/4" => Ok(SizeValue::Quarter),
        "3/4" => Ok(SizeValue::ThreeQuarters),
        _ => {
            // Try to parse as a number
            let value: u16 = input.parse()
                .map_err(|_| format!("Invalid size value: {}", input))?;
            Ok(SizeValue::Fixed(value))
        }
    }
}

/// Helper to parse max-width/height named sizes to pixels.
/// Tailwind: none=0, xs=320, sm=384, md=448, lg=512, xl=576, 2xl=672, 3xl=768, 4xl=896, full=∞
/// Numeric values (e.g. max-w-96) use Tailwind spacing units (N * 4px).
fn parse_max_size_value(input: &str) -> Option<f32> {
    match input {
        "none" | "0" => None, // No constraint
        "xs" => Some(320.0),
        "sm" => Some(384.0),
        "md" => Some(448.0),
        "lg" => Some(512.0),
        "xl" => Some(576.0),
        "2xl" => Some(672.0),
        "3xl" => Some(768.0),
        "4xl" => Some(896.0),
        "5xl" => Some(1024.0),
        "6xl" => Some(1152.0),
        "7xl" => Some(1280.0),
        "full" => None, // No max constraint (fills parent)
        "screen-sm" => Some(640.0),
        "screen-md" => Some(768.0),
        "screen-lg" => Some(1024.0),
        "screen-xl" => Some(1280.0),
        "screen-2xl" => Some(1536.0),
        _ => {
            // Numeric: max-w-96 → 96 * 4 = 384px
            input.parse::<u16>().ok().map(|n| n as f32 * 4.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== L1 Tests ==========

    #[test]
    fn test_parse_padding() {
        assert_eq!(StyleClass::parse_single("p-4"), Ok(StyleClass::Padding(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("p-0"), Ok(StyleClass::Padding(SizeValue::Fixed(0))));
    }

    #[test]
    fn test_parse_gap() {
        assert_eq!(StyleClass::parse_single("gap-2"), Ok(StyleClass::Gap(SizeValue::Fixed(2))));
    }

    #[test]
    fn test_parse_colors() {
        assert!(matches!(StyleClass::parse_single("bg-white"), Ok(StyleClass::BackgroundColor(_))));
        assert!(matches!(StyleClass::parse_single("text-slate-500"), Ok(StyleClass::TextColor(_))));
    }

    #[test]
    fn test_parse_layout() {
        assert_eq!(StyleClass::parse_single("flex"), Ok(StyleClass::Flex));
        assert_eq!(StyleClass::parse_single("flex-row"), Ok(StyleClass::FlexRow));
        assert_eq!(StyleClass::parse_single("flex-col"), Ok(StyleClass::FlexCol));
        assert_eq!(StyleClass::parse_single("items-center"), Ok(StyleClass::ItemsCenter));
    }

    #[test]
    fn test_parse_sizing() {
        assert_eq!(StyleClass::parse_single("w-full"), Ok(StyleClass::Width(SizeValue::Full)));
        assert_eq!(StyleClass::parse_single("h-12"), Ok(StyleClass::Height(SizeValue::Fixed(12))));
    }

    #[test]
    fn test_parse_border_radius() {
        assert_eq!(StyleClass::parse_single("rounded"), Ok(StyleClass::Rounded));
    }

    #[test]
    fn test_size_to_pixels() {
        assert_eq!(SizeValue::Fixed(4).to_pixels(), 16); // 4 * 4px = 16px
    }

    // ========== L2 Tests ==========

    #[test]
    fn test_parse_padding_xy() {
        assert_eq!(StyleClass::parse_single("px-4"), Ok(StyleClass::PaddingX(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("py-2"), Ok(StyleClass::PaddingY(SizeValue::Fixed(2))));
    }

    #[test]
    fn test_parse_margin() {
        assert_eq!(StyleClass::parse_single("m-4"), Ok(StyleClass::Margin(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("mx-2"), Ok(StyleClass::MarginX(SizeValue::Fixed(2))));
        assert_eq!(StyleClass::parse_single("my-2"), Ok(StyleClass::MarginY(SizeValue::Fixed(2))));
    }

    #[test]
    fn test_parse_flex1() {
        assert_eq!(StyleClass::parse_single("flex-1"), Ok(StyleClass::Flex1));
    }

    #[test]
    fn test_parse_text_size() {
        assert_eq!(StyleClass::parse_single("text-xs"), Ok(StyleClass::TextXs));
        assert_eq!(StyleClass::parse_single("text-sm"), Ok(StyleClass::TextSm));
        assert_eq!(StyleClass::parse_single("text-base"), Ok(StyleClass::TextBase));
        assert_eq!(StyleClass::parse_single("text-lg"), Ok(StyleClass::TextLg));
        assert_eq!(StyleClass::parse_single("text-xl"), Ok(StyleClass::TextXl));
        assert_eq!(StyleClass::parse_single("text-2xl"), Ok(StyleClass::Text2Xl));
        assert_eq!(StyleClass::parse_single("text-3xl"), Ok(StyleClass::Text3Xl));
    }

    #[test]
    fn test_parse_font_weight() {
        assert_eq!(StyleClass::parse_single("font-bold"), Ok(StyleClass::FontBold));
        assert_eq!(StyleClass::parse_single("font-medium"), Ok(StyleClass::FontMedium));
        assert_eq!(StyleClass::parse_single("font-normal"), Ok(StyleClass::FontNormal));
    }

    #[test]
    fn test_parse_text_align() {
        assert_eq!(StyleClass::parse_single("text-center"), Ok(StyleClass::TextCenter));
        assert_eq!(StyleClass::parse_single("text-left"), Ok(StyleClass::TextLeft));
        assert_eq!(StyleClass::parse_single("text-right"), Ok(StyleClass::TextRight));
    }

    #[test]
    fn test_parse_items_align() {
        assert_eq!(StyleClass::parse_single("items-start"), Ok(StyleClass::ItemsStart));
        assert_eq!(StyleClass::parse_single("items-end"), Ok(StyleClass::ItemsEnd));
    }

    #[test]
    fn test_parse_justify_align() {
        assert_eq!(StyleClass::parse_single("justify-start"), Ok(StyleClass::JustifyStart));
        assert_eq!(StyleClass::parse_single("justify-end"), Ok(StyleClass::JustifyEnd));
    }

    #[test]
    fn test_parse_rounded_variants() {
        assert_eq!(StyleClass::parse_single("rounded-sm"), Ok(StyleClass::RoundedSm));
        assert_eq!(StyleClass::parse_single("rounded-md"), Ok(StyleClass::RoundedMd));
        assert_eq!(StyleClass::parse_single("rounded-lg"), Ok(StyleClass::RoundedLg));
        assert_eq!(StyleClass::parse_single("rounded-xl"), Ok(StyleClass::RoundedXl));
        assert_eq!(StyleClass::parse_single("rounded-2xl"), Ok(StyleClass::Rounded2Xl));
        assert_eq!(StyleClass::parse_single("rounded-3xl"), Ok(StyleClass::Rounded3Xl));
        assert_eq!(StyleClass::parse_single("rounded-full"), Ok(StyleClass::RoundedFull));
    }

    #[test]
    fn test_parse_border() {
        assert_eq!(StyleClass::parse_single("border"), Ok(StyleClass::Border));
        assert_eq!(StyleClass::parse_single("border-0"), Ok(StyleClass::Border0));
        assert!(matches!(StyleClass::parse_single("border-white"), Ok(StyleClass::BorderColor(_))));
        assert!(matches!(StyleClass::parse_single("border-red-500"), Ok(StyleClass::BorderColor(_))));
    }

    // ========== L3 Tests ==========

    #[test]
    fn test_parse_shadow() {
        assert_eq!(StyleClass::parse_single("shadow"), Ok(StyleClass::Shadow));
        assert_eq!(StyleClass::parse_single("shadow-sm"), Ok(StyleClass::ShadowSm));
        assert_eq!(StyleClass::parse_single("shadow-md"), Ok(StyleClass::ShadowMd));
        assert_eq!(StyleClass::parse_single("shadow-lg"), Ok(StyleClass::ShadowLg));
        assert_eq!(StyleClass::parse_single("shadow-xl"), Ok(StyleClass::ShadowXl));
        assert_eq!(StyleClass::parse_single("shadow-2xl"), Ok(StyleClass::Shadow2Xl));
        assert_eq!(StyleClass::parse_single("shadow-none"), Ok(StyleClass::ShadowNone));
    }

    #[test]
    fn test_parse_opacity() {
        assert_eq!(StyleClass::parse_single("opacity-0"), Ok(StyleClass::Opacity(0)));
        assert_eq!(StyleClass::parse_single("opacity-50"), Ok(StyleClass::Opacity(50)));
        assert_eq!(StyleClass::parse_single("opacity-100"), Ok(StyleClass::Opacity(100)));
    }

    #[test]
    fn test_parse_position() {
        assert_eq!(StyleClass::parse_single("relative"), Ok(StyleClass::Relative));
        assert_eq!(StyleClass::parse_single("absolute"), Ok(StyleClass::Absolute));
    }

    #[test]
    fn test_parse_z_index() {
        assert_eq!(StyleClass::parse_single("z-0"), Ok(StyleClass::ZIndex(0)));
        assert_eq!(StyleClass::parse_single("z-10"), Ok(StyleClass::ZIndex(10)));
        assert_eq!(StyleClass::parse_single("z-50"), Ok(StyleClass::ZIndex(50)));
    }

    #[test]
    fn test_parse_overflow() {
        assert_eq!(StyleClass::parse_single("overflow-auto"), Ok(StyleClass::OverflowAuto));
        assert_eq!(StyleClass::parse_single("overflow-hidden"), Ok(StyleClass::OverflowHidden));
        assert_eq!(StyleClass::parse_single("overflow-visible"), Ok(StyleClass::OverflowVisible));
        assert_eq!(StyleClass::parse_single("overflow-scroll"), Ok(StyleClass::OverflowScroll));
        assert_eq!(StyleClass::parse_single("overflow-x-auto"), Ok(StyleClass::OverflowXAuto));
        assert_eq!(StyleClass::parse_single("overflow-y-auto"), Ok(StyleClass::OverflowYAuto));
    }

    #[test]
    fn test_parse_grid() {
        assert_eq!(StyleClass::parse_single("grid"), Ok(StyleClass::Grid));
        assert_eq!(StyleClass::parse_single("grid-cols-2"), Ok(StyleClass::GridCols(2)));
        assert_eq!(StyleClass::parse_single("grid-cols-12"), Ok(StyleClass::GridCols(12)));
        assert_eq!(StyleClass::parse_single("grid-rows-3"), Ok(StyleClass::GridRows(3)));
    }

    #[test]
    fn test_parse_grid_span() {
        assert_eq!(StyleClass::parse_single("col-span-2"), Ok(StyleClass::ColSpan(2)));
        assert_eq!(StyleClass::parse_single("col-span-6"), Ok(StyleClass::ColSpan(6)));
        assert_eq!(StyleClass::parse_single("row-span-2"), Ok(StyleClass::RowSpan(2)));
    }

    #[test]
    fn test_parse_grid_position() {
        assert_eq!(StyleClass::parse_single("col-start-2"), Ok(StyleClass::ColStart(2)));
        assert_eq!(StyleClass::parse_single("row-start-1"), Ok(StyleClass::RowStart(1)));
    }
}
