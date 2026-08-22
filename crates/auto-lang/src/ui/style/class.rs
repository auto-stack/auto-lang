// StyleClass - Intermediate Representation (IR) for style classes
//
// This enum represents the parsed form of Tailwind-style utility classes.
// It is backend-agnostic and can be translated to GPUI, Iced, or other backends.

use super::Color;

/// Size value (used for width, height, spacing, etc.)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizeValue {
    Full,        // 100%
    Half,        // 50%
    Third,       // 33.333%
    TwoThirds,   // 66.666%
    Quarter,     // 25%
    ThreeQuarters, // 75%
    Auto,
    Fixed(u16),  // Tailwind spacing units (1 = 4px, 2 = 8px, etc.)
    Pixels(f32), // Arbitrary pixel value (e.g. w-[30px] → Pixels(30.0))
}

impl SizeValue {
    /// Convert Tailwind spacing unit to pixels (1 unit = 4px)
    pub fn to_pixels(&self) -> u16 {
        match self {
            SizeValue::Fixed(units) => units * 4,
            SizeValue::Pixels(px) => *px as u16,
            _ => 0, // Full, Auto, etc. are handled differently by backends
        }
    }

    /// Convert to f32 pixels (for cases needing sub-pixel precision)
    pub fn to_pixels_f32(&self) -> f32 {
        match self {
            SizeValue::Fixed(units) => (*units as f32) * 4.0,
            SizeValue::Pixels(px) => *px,
            _ => 0.0,
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

    /// Margin Bottom: mb-{0-12} (L2)
    MarginBottom(SizeValue),

    /// Margin Left: ml-{0-12} (L2)
    MarginLeft(SizeValue),

    /// Margin Right: mr-{0-12} (L2)
    MarginRight(SizeValue),

    /// Margin Left Auto: ml-auto — push element to the right in a row
    MarginLeftAuto,

    /// Margin Right Auto: mr-auto — push element to the left in a row
    MarginRightAuto,

    /// Margin X Auto: mx-auto — center horizontally (both margins auto)
    MarginXAuto,

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

    /// display: block / inline / inline-block / inline-flex (Plan 409 §10 续)
    /// — iced 无 inline/block 区别,主要用于响应式覆盖 Hidden(hidden md:flex)。
    Block,
    Inline,
    InlineBlock,
    InlineFlex,

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

    /// Font size: text-5xl (48px) - L2 (Plan 411: hero headings)
    Text5Xl,

    /// Font size: text-6xl (60px) - L2
    Text6Xl,

    /// Font size: text-7xl (72px) - L2
    Text7Xl,

    /// Font size: text-8xl (96px) - L2
    Text8Xl,

    /// Font size: text-9xl (128px) - L2
    Text9Xl,

    /// Font weight: font-bold (L2)
    FontBold,

    /// Font weight: font-medium (L2)
    FontMedium,

    /// Font weight: font-normal (L2)
    FontNormal,

    /// Font family: font-serif (L2) — iced maps to Serif
    FontSerif,

    /// Font family: font-sans (L2) — iced maps to SansSerif (default)
    FontSans,

    /// Font family: font-mono (L2) — iced maps to Monospace
    FontMono,

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

    // ========== Min/Max Sizing ==========
    /// Min height: min-h-screen, min-h-[Npx]
    MinHeight(f32),

    /// Min width: min-w-[Npx]
    MinWidth(f32),

    // ========== Typography Extended ==========
    /// Arbitrary font size: text-[80px], text-[14px]
    TextArbitrary(f32),

    /// Font weight: font-light (300)
    FontLight,

    /// Font weight: font-extralight (200)
    FontExtraLight,

    /// Font weight: font-semibold (600)
    FontSemiBold,

    /// Line height: leading-[1.4], leading-[1.2]
    LineHeight(f32),

    /// Line height: leading-none
    LineHeightNone,

    // ========== Text Control ==========
    /// Whitespace: whitespace-nowrap
    WhitespaceNowrap,

    /// Word break: break-words
    BreakWords,

    // ========== Interaction ==========
    /// Cursor: cursor-pointer
    CursorPointer,

    // ========== Outline/Border ==========
    /// Outline: outline-none
    OutlineNone,

    /// Border: border-none
    BorderNone,

    // ========== Shadow Extended ==========
    /// Arbitrary shadow: shadow-[...complex value...]
    ShadowArbitrary(String),

    // ========== Flex Extended ==========
    /// Flex shrink: shrink-0
    Shrink0,

    // ========== List ==========
    /// List style: list-none
    ListNone,

    // ========== Position Offsets ==========
    /// Top offset: top-[Npx], -top-[Npx]
    TopOffset(f32),

    /// Bottom offset: bottom-[Npx], -bottom-[Npx]
    BottomOffset(f32),

    /// Right offset: right-[Npx], -right-[Npx]
    RightOffset(f32),

    /// Left offset: left-[Npx], -left-[Npx]
    LeftOffset(f32),

    // ========== Transform ==========
    /// Rotation: rotate-90, rotate-45, etc.
    Rotate(f32),

    // ========== Visibility ==========
    /// Hidden (display: none)
    Hidden,

    // ========== Transition ==========
    /// Transition colors
    TransitionColors,

    /// Transition duration: duration-200
    TransitionDuration(u16),

    // ========== Accent ==========
    /// Accent color: accent-[#hex]
    AccentColor(Color),

    // ========== Font Smoothing ==========
    /// Font smoothing: antialiased
    Antialiased,

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

    // ========== Layout Extended (Plan 412 — Layout Gallery) ==========
    /// Column gap: gap-x-{N} — main-axis spacing on a Row
    GapX(SizeValue),

    /// Row gap: gap-y-{N} — main-axis spacing on a Column
    GapY(SizeValue),

    /// Horizontal space between children: space-x-{N} (≈ gap-x)
    SpaceX(SizeValue),

    /// Vertical space between children: space-y-{N} (≈ gap-y)
    SpaceY(SizeValue),

    /// justify-content: space-around
    JustifyAround,

    /// justify-content: space-evenly
    JustifyEvenly,

    /// align-items: stretch (cross-axis fill)
    ItemsStretch,

    /// flex-direction: row-reverse
    FlexRowReverse,

    /// flex-direction: column-reverse
    FlexColReverse,

    /// flex: 1 1 auto (grow from content basis)
    FlexAuto,

    /// flex: 0 1 auto (CSS default)
    FlexInitial,

    /// flex: none (neither grow nor shrink)
    FlexNone,

    /// flex-grow: 1
    Grow,

    /// flex-grow: 0
    Grow0,

    /// flex-shrink: 1 (default; shrink-0 已有 Shrink0)
    Shrink,

    /// flex-wrap: wrap — VM degrades to single row (Plan 412 §5 降级矩阵)
    FlexWrap,

    /// flex-wrap: wrap-reverse — VM degrades to single row
    FlexWrapReverse,

    /// flex-wrap: nowrap
    FlexNowrap,

    /// align-self: start — VM degrades to container's items-* (Plan 412 §5)
    SelfStart,

    /// align-self: center — VM degrades to container's items-*
    SelfCenter,

    /// align-self: end — VM degrades to container's items-*
    SelfEnd,

    /// align-self: stretch — VM degrades to container's items-*
    SelfStretch,

    /// order: N — VM renders source order (Plan 412 §5)
    Order(i16),

    /// inset: N (all offsets) — VM has no absolute positioning
    Inset(f32),

    /// position: fixed — VM degrades to in-flow position
    Fixed,

    /// position: sticky — VM degrades to in-flow position
    Sticky,
}

impl StyleClass {
    /// Parse a single style class string into a StyleClass
    pub fn parse_single(class: &str) -> Result<Self, String> {
        let class = class.trim();

        // Skip empty strings
        if class.is_empty() {
            return Err("Empty style class".to_string());
        }

        // Plan 409 §10 续: VM 桌面窗口按 md+/lg+ 宽屏语义,剥离 Tailwind
        // min-width 响应前缀(sm/md/lg/xl/2xl),让基础 utility 生效。必须在
        // arbitrary value 提取之前 strip,否则 md:text-[14px] 的方括号识别会
        // 把 prefix 误判为 "md:text-"。max-* 变体语义相反,gallery 未用,留作后续。
        let class = match class.split_once(':') {
            Some(("sm" | "md" | "lg" | "xl" | "2xl", rest)) => rest,
            _ => class,
        };

        // Support Tailwind arbitrary value syntax: text-[#6CB0DD], bg-[#fff]
        // Extract bracket content as `arbitrary_value`, keep prefix as `class`
        let (class, arbitrary_value): (&str, Option<&str>) =
            if class.starts_with(|c: char| c.is_ascii_alphabetic()) && class.ends_with(']') {
                if let Some(bracket_start) = class.find('[') {
                    (&class[..bracket_start], Some(&class[bracket_start + 1..class.len() - 1]))
                } else {
                    (class, None)
                }
            } else {
                (class, None)
            };

        // ========== Spacing (L1 + L2) ==========

        // Parse padding: p-{0-12} or p-[Npx]
        if let Some(rest) = class.strip_prefix("p-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::Padding(size));
        }

        // Parse padding X: px-{0-12} or px-[Npx]
        if let Some(rest) = class.strip_prefix("px-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::PaddingX(size));
        }

        // Parse padding Y: py-{0-12} or py-[Npx]
        if let Some(rest) = class.strip_prefix("py-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::PaddingY(size));
        }

        // Parse per-side padding: pt/pb/pl/pr with arbitrary: pt-[Npx]
        if let Some(rest) = class.strip_prefix("pt-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::PaddingTop(size));
        }
        if let Some(rest) = class.strip_prefix("pb-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::PaddingBottom(size));
        }
        if let Some(rest) = class.strip_prefix("pl-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::PaddingLeft(size));
        }
        if let Some(rest) = class.strip_prefix("pr-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::PaddingRight(size));
        }

        // Parse margin: m-{0-12} or m-[Npx]
        if let Some(rest) = class.strip_prefix("m-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::Margin(size));
        }

        // Parse margin auto classes (exact matches BEFORE prefix matches)
        if class == "mx-auto" {
            // mx-auto = center horizontally (both margins auto)
            return Ok(StyleClass::MarginXAuto);
        }
        if class == "ml-auto" {
            return Ok(StyleClass::MarginLeftAuto);
        }
        if class == "mr-auto" {
            return Ok(StyleClass::MarginRightAuto);
        }

        // Parse margin X: mx-{0-12} or mx-[Npx]
        if let Some(rest) = class.strip_prefix("mx-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::MarginX(size));
        }

        // Parse margin Y: my-{0-12} or my-[Npx]
        if let Some(rest) = class.strip_prefix("my-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::MarginY(size));
        }

        // Parse margin top: mt-{0-12} or mt-[Npx]
        if let Some(rest) = class.strip_prefix("mt-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::MarginTop(size));
        }

        // Parse margin bottom: mb-{0-12} or mb-[Npx]
        if let Some(rest) = class.strip_prefix("mb-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::MarginBottom(size));
        }

        // Parse margin left: ml-{0-12} or ml-[Npx]
        if let Some(rest) = class.strip_prefix("ml-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::MarginLeft(size));
        }

        // Parse margin right: mr-{0-12} or mr-[Npx]
        if let Some(rest) = class.strip_prefix("mr-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::MarginRight(size));
        }

        // Plan 412: gap-x-/gap-y- MUST be matched before the bare `gap-`
        // prefix below, otherwise "gap-x-4" would strip to "x-4" and error.
        if let Some(rest) = class.strip_prefix("gap-x-") {
            let size = parse_gap_value(rest, arbitrary_value)?;
            return Ok(StyleClass::GapX(size));
        }
        if let Some(rest) = class.strip_prefix("gap-y-") {
            let size = parse_gap_value(rest, arbitrary_value)?;
            return Ok(StyleClass::GapY(size));
        }

        // Parse gap: gap-{0-12} or gap-[Npx] or fractional gap-{0.5/1.5/2.5...}
        if let Some(rest) = class.strip_prefix("gap-") {
            let size = parse_gap_value(rest, arbitrary_value)?;
            return Ok(StyleClass::Gap(size));
        }

        // Plan 412: space-x-N / space-y-N — Tailwind 的 per-child margin 间隔,
        // 视觉效果等价于 gap-N,VM 按 gap 处理(vue 端是原生 CSS 类,天然精确)。
        if let Some(rest) = class.strip_prefix("space-x-") {
            let size = parse_gap_value(rest, arbitrary_value)?;
            return Ok(StyleClass::SpaceX(size));
        }
        if let Some(rest) = class.strip_prefix("space-y-") {
            let size = parse_gap_value(rest, arbitrary_value)?;
            return Ok(StyleClass::SpaceY(size));
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
            let color = parse_color_with_alpha(color_name, arbitrary_value)?;
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

        // Parse text size: text-{xs,sm,base,lg,xl,2xl,3xl} or arbitrary text-[80px]
        match class {
            "text-xs" => return Ok(StyleClass::TextXs),
            "text-sm" => return Ok(StyleClass::TextSm),
            "text-base" => return Ok(StyleClass::TextBase),
            "text-lg" => return Ok(StyleClass::TextLg),
            "text-xl" => return Ok(StyleClass::TextXl),
            "text-2xl" => return Ok(StyleClass::Text2Xl),
            "text-3xl" => return Ok(StyleClass::Text3Xl),
            "text-4xl" => return Ok(StyleClass::Text4Xl),
            "text-5xl" => return Ok(StyleClass::Text5Xl),
            "text-6xl" => return Ok(StyleClass::Text6Xl),
            "text-7xl" => return Ok(StyleClass::Text7Xl),
            "text-8xl" => return Ok(StyleClass::Text8Xl),
            "text-9xl" => return Ok(StyleClass::Text9Xl),
            _ => {}
        }

        // Parse arbitrary text size: text-[80px], text-[14px], text-[11px]
        // Also handle text-[#hex] as text color (e.g. text-[#111], text-[#333])
        // NOTE: When arbitrary brackets are present, class becomes "text-" (dash before [)
        // so we check both "text" and "text-".
        if class == "text" || class == "text-" {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::TextArbitrary(px));
            }
            // Try as hex color for text-[#hex]
            if let Some(av) = arbitrary_value {
                if let Ok(color) = Color::from_hex(av) {
                    return Ok(StyleClass::TextColor(color));
                }
            }
        }

        // Parse font weight
        match class {
            "font-bold" => return Ok(StyleClass::FontBold),
            "font-medium" => return Ok(StyleClass::FontMedium),
            "font-normal" => return Ok(StyleClass::FontNormal),
            "font-serif" => return Ok(StyleClass::FontSerif),
            "font-sans" => return Ok(StyleClass::FontSans),
            "font-mono" => return Ok(StyleClass::FontMono),
            "font-normal" => return Ok(StyleClass::FontNormal),
            "font-light" => return Ok(StyleClass::FontLight),
            "font-extralight" => return Ok(StyleClass::FontExtraLight),
            "font-semibold" => return Ok(StyleClass::FontSemiBold),
            _ => {}
        }

        // Skip font-inherit (no iced equivalent, but don't error)
        if class == "font-inherit" {
            return Err("font-inherit: not supported in native renderer".to_string());
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
            let color = parse_color_with_alpha(color_name, arbitrary_value)?;
            return Ok(StyleClass::TextColor(color));
        }

        // ========== Line Height ==========
        // Parse leading-[1.4], leading-[1.2]
        // NOTE: With arbitrary brackets, class becomes "leading-" (dash before [)
        if class == "leading" || class == "leading-" {
            if let Some(av) = arbitrary_value {
                if let Ok(lh) = av.parse::<f32>() {
                    return Ok(StyleClass::LineHeight(lh));
                }
            }
        }
        // Parse leading-none
        if class == "leading-none" {
            return Ok(StyleClass::LineHeightNone);
        }

        // ========== Whitespace & Text Control ==========
        if class == "whitespace-nowrap" {
            return Ok(StyleClass::WhitespaceNowrap);
        }
        if class == "break-words" {
            return Ok(StyleClass::BreakWords);
        }

        // ========== Interaction ==========
        if class == "cursor-pointer" {
            return Ok(StyleClass::CursorPointer);
        }

        // ========== Outline & Border ==========
        if class == "outline-none" {
            return Ok(StyleClass::OutlineNone);
        }
        if class == "border-none" {
            return Ok(StyleClass::BorderNone);
        }

        // ========== Accent Color ==========
        if let Some(color_name) = class.strip_prefix("accent-") {
            if let Ok(color) = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))
                .or_else(|_| {
                    arbitrary_value
                        .and_then(|v| Color::from_hex(v).ok())
                        .ok_or_else(|| format!("Unknown accent color: {}", color_name))
                }) {
                return Ok(StyleClass::AccentColor(color));
            }
        }

        // ========== List Style ==========
        if class == "list-none" {
            return Ok(StyleClass::ListNone);
        }

        // ========== Font Smoothing ==========
        if class == "antialiased" {
            return Ok(StyleClass::Antialiased);
        }

        // ========== Flex Extended ==========
        if class == "shrink-0" {
            return Ok(StyleClass::Shrink0);
        }

        // ========== Visibility ==========
        if class == "hidden" {
            return Ok(StyleClass::Hidden);
        }

        // ========== Transition ==========
        if class == "transition-colors" {
            return Ok(StyleClass::TransitionColors);
        }
        if let Some(rest) = class.strip_prefix("duration-") {
            if let Ok(ms) = rest.parse::<u16>() {
                return Ok(StyleClass::TransitionDuration(ms));
            }
        }

        // ========== Transform ==========
        if let Some(rest) = class.strip_prefix("rotate-") {
            if let Ok(deg) = rest.parse::<f32>() {
                return Ok(StyleClass::Rotate(deg));
            }
        }

        // ========== Layout (L1 + L2) ==========

        // Parse flex
        if class == "flex" {
            return Ok(StyleClass::Flex);
        }

        // Plan 409 §10 续: display 变体(主要用于响应式覆盖 Hidden)
        if class == "block" {
            return Ok(StyleClass::Block);
        }
        if class == "inline" {
            return Ok(StyleClass::Inline);
        }
        if class == "inline-block" {
            return Ok(StyleClass::InlineBlock);
        }
        if class == "inline-flex" {
            return Ok(StyleClass::InlineFlex);
        }

        // Parse flex-1
        if class == "flex-1" {
            return Ok(StyleClass::Flex1);
        }

        // Plan 412: flex 伸缩变体。flex-auto≈flex-1(basis auto);
        // flex-initial/flex-none 是 CSS 默认/固定(iced 默认即不伸缩,no-op)。
        match class {
            "flex-auto" => return Ok(StyleClass::FlexAuto),
            "flex-initial" => return Ok(StyleClass::FlexInitial),
            "flex-none" => return Ok(StyleClass::FlexNone),
            "grow" => return Ok(StyleClass::Grow),
            "grow-0" => return Ok(StyleClass::Grow0),
            "shrink" => return Ok(StyleClass::Shrink),
            _ => {}
        }

        // Plan 412: flex-wrap 系。iced 无 wrap widget — 解析保存,VM 渲染降级
        // 为单行(Plan 412 §5 降级矩阵),一次性 eprintln 提示。
        match class {
            "flex-wrap" => return Ok(StyleClass::FlexWrap),
            "flex-wrap-reverse" => return Ok(StyleClass::FlexWrapReverse),
            "flex-nowrap" => return Ok(StyleClass::FlexNowrap),
            _ => {}
        }

        // Parse flex-row
        if class == "flex-row" {
            return Ok(StyleClass::FlexRow);
        }

        // Parse flex-col
        if class == "flex-col" {
            return Ok(StyleClass::FlexCol);
        }

        // Plan 412: 反向布局 — children 反序(build 期,双端语义一致)。
        match class {
            "flex-row-reverse" => return Ok(StyleClass::FlexRowReverse),
            "flex-col-reverse" => return Ok(StyleClass::FlexColReverse),
            _ => {}
        }

        // Parse items-*
        match class {
            "items-center" => return Ok(StyleClass::ItemsCenter),
            "items-start" => return Ok(StyleClass::ItemsStart),
            "items-end" => return Ok(StyleClass::ItemsEnd),
            // Plan 412: align-items: stretch(交叉轴 Fill,渲染层处理)
            "items-stretch" => return Ok(StyleClass::ItemsStretch),
            _ => {}
        }

        // Parse justify-*
        match class {
            "justify-center" => return Ok(StyleClass::JustifyCenter),
            "justify-between" => return Ok(StyleClass::JustifyBetween),
            "justify-start" => return Ok(StyleClass::JustifyStart),
            "justify-end" => return Ok(StyleClass::JustifyEnd),
            // Plan 412: space-around / space-evenly(FillPortion spacer 精确模拟)
            "justify-around" => return Ok(StyleClass::JustifyAround),
            "justify-evenly" => return Ok(StyleClass::JustifyEvenly),
            _ => {}
        }

        // Plan 412: align-self — iced 无 per-child 对齐,解析保存,渲染降级到容器 items。
        match class {
            "self-start" => return Ok(StyleClass::SelfStart),
            "self-center" => return Ok(StyleClass::SelfCenter),
            "self-end" => return Ok(StyleClass::SelfEnd),
            "self-stretch" => return Ok(StyleClass::SelfStretch),
            _ => {}
        }

        // ========== Sizing (L1) ==========

        // Parse width: w-{size} (supports arbitrary: w-[30px])
        if let Some(rest) = class.strip_prefix("w-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::Width(size));
        }

        // Parse height: h-{size} (supports arbitrary: h-[65px])
        if let Some(rest) = class.strip_prefix("h-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::Height(size));
        }

        // Parse min-height: min-h-{size|screen} (supports arbitrary: min-h-[40px])
        if let Some(rest) = class.strip_prefix("min-h-") {
            let px = match rest {
                "screen" => f32::MAX,
                _ => parse_pixel_arbitrary(arbitrary_value)
                    .unwrap_or_else(|| {
                        rest.parse::<f32>().unwrap_or(0.0) * 4.0
                    }),
            };
            return Ok(StyleClass::MinHeight(px));
        }

        // Parse min-width: min-w-{size} (supports arbitrary: min-w-[Npx])
        if let Some(rest) = class.strip_prefix("min-w-") {
            let px = parse_pixel_arbitrary(arbitrary_value)
                .unwrap_or_else(|| {
                    rest.parse::<f32>().unwrap_or(0.0) * 4.0
                });
            return Ok(StyleClass::MinWidth(px));
        }

        // ========== Max Sizing (L1) ==========

        // Parse max-width: max-w-{named|numeric} or max-w-[Npx]
        if let Some(rest) = class.strip_prefix("max-w-") {
            if let Some(px) = parse_max_size_value_arbitrary(rest, arbitrary_value) {
                return Ok(StyleClass::MaxWidth(px));
            }
        }

        // Parse max-height: max-h-{named|numeric} or max-h-[Npx]
        if let Some(rest) = class.strip_prefix("max-h-") {
            if let Some(px) = parse_max_size_value_arbitrary(rest, arbitrary_value) {
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

        // Parse border color: border-{color} (supports arbitrary: border-[#hex])
        if let Some(color_name) = class.strip_prefix("border-") {
            // Skip border-0 which we already handled
            if color_name == "0" {
                return Ok(StyleClass::Border0);
            }
            if color_name == "none" {
                return Ok(StyleClass::BorderNone);
            }
            if color_name == "transparent" {
                // Use a hex color that iced adapter can treat as transparent
                // Use a special hex value — iced adapter should handle it
                return Ok(StyleClass::BorderColor(Color::from_hex("#00000000").unwrap_or_else(|_| {
                    // Fallback: just use white, the border-none style above is preferred
                    Color::from_hex("#ffffff").unwrap()
                })));
            }
            let color = parse_color_with_alpha(color_name, arbitrary_value)?;
            return Ok(StyleClass::BorderColor(color));
        }

        // ========== Effects (L3) ==========

        // Parse shadow variants
        match class {
            "shadow" | "shadow-" => {
                // "shadow-" is the prefix when arbitrary value present (shadow-[...])
                if let Some(av) = arbitrary_value {
                    return Ok(StyleClass::ShadowArbitrary(av.to_string()));
                }
                return Ok(StyleClass::Shadow);
            }
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
            // Plan 412: fixed/sticky — iced 无视口定位,解析保存,VM 降级为就近布局位。
            "fixed" => return Ok(StyleClass::Fixed),
            "sticky" => return Ok(StyleClass::Sticky),
            _ => {}
        }

        // Plan 412: order-N — iced 无 per-child 排序,解析保存,VM 按源码序渲染。
        if let Some(rest) = class.strip_prefix("order-") {
            if let Ok(v) = rest.parse::<i16>() {
                return Ok(StyleClass::Order(v));
            }
        }

        // Plan 412: inset-N(all offsets)— iced 无绝对定位,解析保存(降级)。
        // 支持 inset-0 / inset-N(N×4px)/ inset-[Npx]。
        if let Some(rest) = class.strip_prefix("inset-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::Inset(px));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::Inset(n * 4.0));
            }
        }

        // Parse position offsets: top-[Npx], -top-[Npx], bottom/right/left
        if let Some(rest) = class.strip_prefix("top-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::TopOffset(px));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::TopOffset(n * 4.0));
            }
        }
        if let Some(rest) = class.strip_prefix("bottom-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::BottomOffset(px));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::BottomOffset(n * 4.0));
            }
        }
        if let Some(rest) = class.strip_prefix("right-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::RightOffset(px));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::RightOffset(n * 4.0));
            }
        }
        if let Some(rest) = class.strip_prefix("left-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::LeftOffset(px));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::LeftOffset(n * 4.0));
            }
        }

        // Handle negative offsets: -top-[Npx], -bottom-[Npx], etc.
        if class.starts_with("-top-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::TopOffset(-px));
            }
        }
        if class.starts_with("-bottom-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::BottomOffset(-px));
            }
        }
        if class.starts_with("-right-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::RightOffset(-px));
            }
        }
        if class.starts_with("-left-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::LeftOffset(-px));
            }
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
            // Plan 057 (ash-gui 表格列对齐): grid-cols-[N] 任意值形式(纯数字,
            // 如动态拼接 "grid grid-cols-" + n 产出)—— 剥 [] 后按普通值解析。
            let rest = rest.trim_start_matches('[').trim_end_matches(']');
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
/// Plan 047:解析颜色名(可能带 /N 透明度修饰符)。
/// "card/30" → Color::Surface 带.alpha(76); "sky-300" → Color::Sky(300) alpha=255。
/// 支持 tailwind 色名 + hex + 语义色。失败返回 Err。
fn parse_color_with_alpha(color_name: &str, arbitrary: Option<&str>) -> Result<Color, String> {
    // 分离 /N alpha 修饰符(如 "card/30"、"sky-300/90")。
    let (base_name, alpha) = if let Some(slash_pos) = color_name.find('/') {
        let (base, rest) = color_name.split_at(slash_pos);
        // rest 形如 "/30",去掉 '/' 后 parse
        let alpha_pct: u8 = rest[1..].parse().unwrap_or(100);
        (base, alpha_pct)
    } else {
        (color_name, 100u8)
    };

    let color = Color::from_tailwind(base_name)
        .or_else(|_| Color::from_hex(base_name))
        .or_else(|_| {
            arbitrary
                .and_then(|v| Color::from_hex(v).ok())
                .ok_or_else(|| format!("Unknown color: {}", base_name))
        })?;

    if alpha < 100 {
        // Plan 409 §10 回归修复:语义色(Background/Surface/Primary/…)必须
        // dark-mode-aware 解析。直接 to_rgb8() 会取硬编码 light 值
        // (Background→白),在深色主题下丢失深色信息 —— 表现为 header
        // `bg-background/95` 被画成白色。故语义色优先 resolve_semantic_rgb
        // 取主题正确 RGB 再乘 alpha;resolve_semantic_rgb 对非语义色返回
        // None,自然落到下面的 to_rgb8() 路径(Tailwind 色板/hex/rgb)。
        if let Some((r, g, b)) = super::iced_adapter::resolve_semantic_rgb(&color) {
            let a = (alpha as u32 * 255 / 100).min(255) as u8;
            return Ok(Color::Rgba { r, g, b, a });
        }
        // 非语义色:乘 alpha,转 Rgba。
        let (r, g, b) = color.to_rgb8();
        let a = (alpha as u32 * 255 / 100).min(255) as u8;
        Ok(Color::Rgba { r, g, b, a })
    } else {
        Ok(color)
    }
}

fn parse_size_value(input: &str) -> Result<SizeValue, String> {
    match input {
        "full" | "screen" => Ok(SizeValue::Full),
        "auto" => Ok(SizeValue::Auto),
        // Tailwind *-px = 1px(2026-08-22:w-px/h-px 此前静默丢弃 —— sep 的
        // w-px 宽度丢失、041 横向发丝线 h-px 高度丢失,均源于此)。
        "px" => Ok(SizeValue::Pixels(1.0)),
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

/// Parse a size value with optional arbitrary value support (e.g. [30px], [65px])
fn parse_size_value_arbitrary(input: &str, arbitrary: Option<&str>) -> Result<SizeValue, String> {
    // Try arbitrary pixel value first: [30px], [65px], [calc(100%-43px)]
    if let Some(av) = arbitrary {
        // Try Npx
        if let Some(px_str) = av.strip_suffix("px") {
            if let Ok(px) = px_str.parse::<f32>() {
                return Ok(SizeValue::Pixels(px));
            }
        }
        // Try bare number
        if let Ok(px) = av.parse::<f32>() {
            return Ok(SizeValue::Pixels(px));
        }
    }
    parse_size_value(input)
}

/// Parse a gap-like value (gap/gap-x/gap-y/space-x/space-y): named sizes,
/// arbitrary [Npx], integers (Tailwind units) or fractional (0.5/1.5/2.5…).
/// Plan 412: extracted from the gap- branch so gap-x-/gap-y-/space-* share it.
fn parse_gap_value(input: &str, arbitrary: Option<&str>) -> Result<SizeValue, String> {
    parse_size_value_arbitrary(input, arbitrary).or_else(|_| {
        // Plan 409 §10 续: SizeValue::Fixed 是 u16,不支持 fractional。
        // Tailwind 0.5/1.5/2.5/3.5 常用(1 unit = 4px)→ 用 Pixels。
        input
            .parse::<f32>()
            .map(|f| SizeValue::Pixels(f * 4.0))
            .map_err(|e| e.to_string())
    })
}

/// Parse a float pixel value from arbitrary syntax: [Npx]
fn parse_pixel_arbitrary(arbitrary: Option<&str>) -> Option<f32> {
    arbitrary.and_then(|v| {
        let v = v.strip_suffix("px").unwrap_or(v);
        v.parse::<f32>().ok()
    })
}

/// Helper to parse max-width/height named sizes to pixels.
/// Tailwind: none=0, xs=320, sm=384, md=448, lg=512, xl=576, 2xl=672, 3xl=768, 4xl=896, full=∞
/// Numeric values (e.g. max-w-96) use Tailwind spacing units (N * 4px).
/// Parse max-width/height with optional arbitrary value support (e.g. [550px]).
fn parse_max_size_value_arbitrary(input: &str, arbitrary: Option<&str>) -> Option<f32> {
    // Try arbitrary pixel value first: [550px], [300]
    if let Some(av) = arbitrary {
        if let Some(px_str) = av.strip_suffix("px") {
            if let Ok(px) = px_str.parse::<f32>() {
                return Some(px);
            }
        }
        if let Ok(px) = av.parse::<f32>() {
            return Some(px);
        }
    }
    match input {
        "none" | "0" | "" if arbitrary.is_some() => None, // No constraint
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
    fn test_semantic_color_with_alpha_is_dark_aware() {
        // Plan 409 §10 回归:bg-background/95 在深色主题下应是 gray-900 (17,24,39)
        // + alpha≈242,而非 to_rgb8() 拍平出的白色 (255,255,255)。
        crate::ui::style::iced_adapter::set_dark_mode(true);
        match StyleClass::parse_single("bg-background/95") {
            Ok(StyleClass::BackgroundColor(Color::Rgba { r, g, b, a })) => {
                assert_eq!((r, g, b), (17, 24, 39), "深色主题下 background 应为 gray-900,而非白色");
                assert_eq!(a, 242, "95% alpha → 242");
            }
            other => panic!("期望 BackgroundColor(Rgba),得到 {:?}", other),
        }
        // bg-primary/10:语义色 + 小 alpha 应保留 alpha 且取主题 primary,
        // 而非丢失 alpha 变全饱和色。
        match StyleClass::parse_single("bg-primary/10") {
            Ok(StyleClass::BackgroundColor(Color::Rgba { r, g, b, a })) => {
                assert_ne!((r, g, b), (255, 255, 255), "primary 不应是白色");
                assert_eq!(a, 25, "10% alpha → 25");
            }
            other => panic!("期望 BackgroundColor(Rgba),得到 {:?}", other),
        }
        // 非语义色不受影响:bg-gray-500/50 仍走 to_rgb8() 乘 alpha。
        match StyleClass::parse_single("bg-gray-500/50") {
            Ok(StyleClass::BackgroundColor(Color::Rgba { a, .. })) => {
                assert_eq!(a, 127, "50% alpha → 127");
            }
            other => panic!("期望 BackgroundColor(Rgba),得到 {:?}", other),
        }
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

    // ========== Plan 412 Tests ==========

    #[test]
    fn test_parse_plan412_axis_gaps() {
        // gap-x-/gap-y- 不能落进裸 gap- 前缀(会因 "x-4" 解析失败而丢类)
        assert_eq!(StyleClass::parse_single("gap-x-4"), Ok(StyleClass::GapX(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("gap-y-1.5"), Ok(StyleClass::GapY(SizeValue::Pixels(6.0))));
        assert_eq!(StyleClass::parse_single("space-x-2"), Ok(StyleClass::SpaceX(SizeValue::Fixed(2))));
        assert_eq!(StyleClass::parse_single("space-y-8"), Ok(StyleClass::SpaceY(SizeValue::Fixed(8))));
        // 裸 gap 不回归
        assert_eq!(StyleClass::parse_single("gap-4"), Ok(StyleClass::Gap(SizeValue::Fixed(4))));
    }

    #[test]
    fn test_parse_plan412_justify_items() {
        assert_eq!(StyleClass::parse_single("justify-around"), Ok(StyleClass::JustifyAround));
        assert_eq!(StyleClass::parse_single("justify-evenly"), Ok(StyleClass::JustifyEvenly));
        assert_eq!(StyleClass::parse_single("items-stretch"), Ok(StyleClass::ItemsStretch));
        assert_eq!(StyleClass::parse_single("self-center"), Ok(StyleClass::SelfCenter));
        assert_eq!(StyleClass::parse_single("self-start"), Ok(StyleClass::SelfStart));
    }

    #[test]
    fn test_parse_plan412_flex_variants() {
        assert_eq!(StyleClass::parse_single("flex-auto"), Ok(StyleClass::FlexAuto));
        assert_eq!(StyleClass::parse_single("flex-initial"), Ok(StyleClass::FlexInitial));
        assert_eq!(StyleClass::parse_single("flex-none"), Ok(StyleClass::FlexNone));
        assert_eq!(StyleClass::parse_single("flex-row-reverse"), Ok(StyleClass::FlexRowReverse));
        assert_eq!(StyleClass::parse_single("flex-col-reverse"), Ok(StyleClass::FlexColReverse));
        assert_eq!(StyleClass::parse_single("flex-wrap"), Ok(StyleClass::FlexWrap));
        assert_eq!(StyleClass::parse_single("grow"), Ok(StyleClass::Grow));
        assert_eq!(StyleClass::parse_single("grow-0"), Ok(StyleClass::Grow0));
        assert_eq!(StyleClass::parse_single("shrink"), Ok(StyleClass::Shrink));
        // 既有 flex 类不回归
        assert_eq!(StyleClass::parse_single("flex-1"), Ok(StyleClass::Flex1));
        assert_eq!(StyleClass::parse_single("shrink-0"), Ok(StyleClass::Shrink0));
    }

    #[test]
    fn test_parse_plan412_position_degraded() {
        assert_eq!(StyleClass::parse_single("fixed"), Ok(StyleClass::Fixed));
        assert_eq!(StyleClass::parse_single("sticky"), Ok(StyleClass::Sticky));
        assert_eq!(StyleClass::parse_single("inset-0"), Ok(StyleClass::Inset(0.0)));
        assert_eq!(StyleClass::parse_single("inset-4"), Ok(StyleClass::Inset(16.0)));
        assert_eq!(StyleClass::parse_single("order-2"), Ok(StyleClass::Order(2)));
        // 响应式前缀剥离对新类同样生效
        assert_eq!(StyleClass::parse_single("md:grid-cols-2"), Ok(StyleClass::GridCols(2)));
    }

}
