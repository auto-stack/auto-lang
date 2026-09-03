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
    /// Plan 527 T3: 通用分数 N/M(w-3/12、w-2/5 …)。无容器查询语义,宿主按
    /// Fill-ratio 近似消费(iced FillPortion(n)):同分母互补分数(3/12+9/12)
    /// 比例保真,混分母组合退化为等分 —— 口径记 KNOWN-DEBT(待澄清③裁定)。
    Fraction(u16, u16),
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

/// Border radius size for rounded-* and directional rounded-*
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundedSize {
    None,    // 0px (rounded-none)
    Sm,      // 2px (rounded-sm)
    Default, // 4px (rounded)
    Md,      // 6px (rounded-md)
    Lg,      // 8px (rounded-lg)
    Xl,      // 12px (rounded-xl)
    Xxl,     // 16px (rounded-2xl)
    Xxxl,    // 24px (rounded-3xl)
    Full,    // 9999px (rounded-full)
}

impl RoundedSize {
    pub fn to_pixels(&self) -> f32 {
        match self {
            RoundedSize::None => 0.0,
            RoundedSize::Sm => 2.0,
            RoundedSize::Default => 4.0,
            RoundedSize::Md => 6.0,
            RoundedSize::Lg => 8.0,
            RoundedSize::Xl => 12.0,
            RoundedSize::Xxl => 16.0,
            RoundedSize::Xxxl => 24.0,
            RoundedSize::Full => 9999.0,
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

/// Plan 527 T4: object-fit(object-contain/cover/fill/none/scale-down)——
/// iced Image ContentFit 的后端无关 IR。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFit {
    Contain,
    Cover,
    Fill,
    None,
    ScaleDown,
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

    /// Border radius: rounded-none (0px)
    RoundedNone,

    // ========== Directional Border Radius ==========
    /// Top border radius: rounded-t[-size]
    RoundedT(Option<RoundedSize>),
    /// Bottom border radius: rounded-b[-size]
    RoundedB(Option<RoundedSize>),
    /// Left border radius: rounded-l[-size]
    RoundedL(Option<RoundedSize>),
    /// Right border radius: rounded-r[-size]
    RoundedR(Option<RoundedSize>),
    /// Top-left border radius: rounded-tl[-size]
    RoundedTL(Option<RoundedSize>),
    /// Top-right border radius: rounded-tr[-size]
    RoundedTR(Option<RoundedSize>),
    /// Bottom-left border radius: rounded-bl[-size]
    RoundedBL(Option<RoundedSize>),
    /// Bottom-right border radius: rounded-br[-size]
    RoundedBR(Option<RoundedSize>),

    // ========== Negative Margins ==========
    NegativeMargin(SizeValue),
    NegativeMarginX(SizeValue),
    NegativeMarginY(SizeValue),
    NegativeMarginTop(SizeValue),
    NegativeMarginBottom(SizeValue),
    NegativeMarginLeft(SizeValue),
    NegativeMarginRight(SizeValue),

    // ========== Border (L2) ==========
    /// Border: border (default width and color)
    Border,
    // PLAN-050 C2: 单侧边框（iced 四边整圈的缺口,renderer 以 1px 填充条模拟）
    BorderBottom,
    BorderTop,
    BorderLeft,
    BorderRight,

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

    /// Text decoration: line-through (strikethrough)
    LineThrough,

    /// Text decoration: underline
    Underline,

    /// Text decoration: no-underline
    NoUnderline,

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

    /// Truncate: truncate(单行 + 溢出裁剪,对齐 Vue/CSS 语义)
    Truncate,

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

    // ========== Plan 527 T3 — 布局家族补全 ==========
    /// Grid column end: col-end-{1-13}(存字段,渲染降级同 col-start)
    ColEnd(u8),

    /// Grid row end: row-end-{1-7}(存字段,渲染降级同 row-start)
    RowEnd(u8),

    /// flex-basis: basis-{N|N/M|auto|full}(Fill-ratio/像素近似,渲染层分期消费)
    FlexBasis(SizeValue),

    /// Overflow X: overflow-x-hidden(clip 语义归并到 Hidden)
    OverflowXHidden,
    /// Overflow X: overflow-x-visible
    OverflowXVisible,
    /// Overflow X: overflow-x-scroll
    OverflowXScroll,
    /// Overflow Y: overflow-y-hidden(clip 语义归并到 Hidden)
    OverflowYHidden,
    /// Overflow Y: overflow-y-visible
    OverflowYVisible,
    /// Overflow Y: overflow-y-scroll
    OverflowYScroll,

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

    /// Plan 442 A6: `lang-<token>` metadata class carrying the code
    /// block's language (e.g. `lang-rust`) from the view builder to the
    /// renderer's syntax-highlight path. Not a visual utility — every
    /// style adapter treats it as inert; the iced renderer reads it to
    /// pick the syntect grammar for read-only code highlighting.
    CodeLang(String),

    /// Plan 518 G8：backdrop-filter: blur(Npx)（声明冻结,渲染分期）——
    /// 值为像素半径（Tailwind 刻度 sm=4/默认 8/md=12/lg=16/xl=24/
    /// 2xl=40/3xl=64 + [Npx] 任意值）。vue 臂类串直通出真毛玻璃;
    /// iced/gpui/headless 渲染端视觉 no-op（装饰性降级非错绘——真
    /// backdrop 渲染挂 RenderQueue 宿主栅格化,KNOWN-DEBT planned-debt）。
    BackdropBlur(f32),

    /// Plan 518 G8：backdrop-filter: saturate(N)（声明冻结）——值为倍率
    /// （刻度 50/100/150/200 → 0.5/1.0/1.5/2.0 + [N] 任意值,stella 配方
    /// [1.6]）。三臂语义同 BackdropBlur。
    BackdropSaturate(f32),

    // ========== Plan 527 T4 — 视觉家族补全 ==========
    /// object-fit: object-contain/cover/fill/none/scale-down(Image 消费面)
    ObjectFit(ObjectFit),

    /// ring 宽度: ring(3px)/ring-0/1/2/4/8(渲染层分期消费,focus 环模拟)
    RingWidth(f32),
    /// ring 颜色: ring-{color}
    RingColor(Color),
    /// ring-inset 标记
    RingInset,

    /// 渐变中间 stop 颜色: via-{color}(iced 多 stop 渐变真消费,默认 50% 位)
    GradientVia(Color),
    /// 渐变 stop 位置百分比: from-{0..100}(默认 0%)
    GradientFromStop(u8),
    /// 渐变 stop 位置百分比: via-{0..100}(默认 50%)
    GradientViaStop(u8),
    /// 渐变 stop 位置百分比: to-{0..100}(默认 100%)
    GradientToStop(u8),

    /// 彩色阴影: shadow-{color}(渲染层分期消费)
    ShadowColor(Color),

    // ========== Plan 527 T5 — 文本家族补全 ==========
    /// 字距: tracking-tighter..widest(em 单位;iced 0.14 文本无 letter_spacing,
    /// IR 冻结待渲染分期,KNOWN-DEBT 登记)
    Tracking(f32),
    /// 绝对行高: leading-3..10(px 固定值,区别于相对倍率 LineHeight)
    LineHeightPx(f32),
    /// 行数截断: line-clamp-{1..6}
    LineClamp(u8),
    /// line-clamp-none(对消语义,落 0 档)
    LineClampNone,
    /// font-thin(100)——此前与 extralight 合并,Plan 527 T5 全字重档拆分
    FontThin,
    /// font-extrabold(800)
    FontExtraBold,
    /// font-black(900)
    FontBlack,
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

        // Plan 442 A6: lang-<token> — code-language metadata (checked
        // before the spacing prefixes; no Tailwind utility starts with
        // "lang-"). Token charset keeps the class a single CSS-safe word
        // (rust, py, c++, objective-c ...).
        if let Some(rest) = class.strip_prefix("lang-") {
            if !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '#' | '.' | '_' | '-'))
            {
                return Ok(StyleClass::CodeLang(rest.to_string()));
            }
        }

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

        // Parse negative margin: -m-, -mx-, -my-, -mt-, -mb-, -ml-, -mr-
        if let Some(rest) = class.strip_prefix("-mx-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMarginX(size));
        }
        if let Some(rest) = class.strip_prefix("-my-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMarginY(size));
        }
        if let Some(rest) = class.strip_prefix("-mt-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMarginTop(size));
        }
        if let Some(rest) = class.strip_prefix("-mb-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMarginBottom(size));
        }
        if let Some(rest) = class.strip_prefix("-ml-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMarginLeft(size));
        }
        if let Some(rest) = class.strip_prefix("-mr-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMarginRight(size));
        }
        if let Some(rest) = class.strip_prefix("-m-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::NegativeMargin(size));
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

        // Parse gradient start: from-{color} / from-{0..100}(stop 位置百分比)
        // Plan 527 T4:pct 档优先,堵 from-100 被 3 位 hex 展开误吞为 #110000
        // 的假映射(from_hex("100") → len3 → 双字符展开)。
        if let Some(rest) = class.strip_prefix("from-") {
            if let Ok(pct) = rest.parse::<u8>() {
                if pct <= 100 {
                    return Ok(StyleClass::GradientFromStop(pct));
                }
            }
            if let Ok(color) = Color::from_tailwind(rest).or_else(|_| Color::from_hex(rest)) {
                return Ok(StyleClass::GradientFrom(color));
            }
        }

        // Parse gradient middle stop: via-{color} / via-{0..100}(默认 50% 位)
        if let Some(rest) = class.strip_prefix("via-") {
            if let Ok(pct) = rest.parse::<u8>() {
                if pct <= 100 {
                    return Ok(StyleClass::GradientViaStop(pct));
                }
            }
            if let Ok(color) = Color::from_tailwind(rest).or_else(|_| Color::from_hex(rest)) {
                return Ok(StyleClass::GradientVia(color));
            }
        }

        // Parse gradient end: to-{color} / to-{0..100}(stop 位置百分比)
        if let Some(color_name) = class.strip_prefix("to-") {
            if let Ok(pct) = color_name.parse::<u8>() {
                if pct <= 100 {
                    return Ok(StyleClass::GradientToStop(pct));
                }
            }
            if let Ok(color) = Color::from_tailwind(color_name).or_else(|_| Color::from_hex(color_name)) {
                return Ok(StyleClass::GradientTo(color));
            }
        }

        // Plan 527 T4: object-fit —— Image ContentFit 消费面
        match class {
            "object-contain" => return Ok(StyleClass::ObjectFit(ObjectFit::Contain)),
            "object-cover" => return Ok(StyleClass::ObjectFit(ObjectFit::Cover)),
            "object-fill" => return Ok(StyleClass::ObjectFit(ObjectFit::Fill)),
            "object-none" => return Ok(StyleClass::ObjectFit(ObjectFit::None)),
            "object-scale-down" => return Ok(StyleClass::ObjectFit(ObjectFit::ScaleDown)),
            _ => {}
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
        // Plan 527 T5: 全字重档拆分(thin/extrabold/black 独立变体,
        // 此前 black|extrabold 合并 FontBold、thin 合并 ExtraLight)。
        match class {
            "font-black" => return Ok(StyleClass::FontBlack),
            "font-extrabold" => return Ok(StyleClass::FontExtraBold),
            "font-bold" => return Ok(StyleClass::FontBold),
            "font-semibold" => return Ok(StyleClass::FontSemiBold),
            "font-medium" => return Ok(StyleClass::FontMedium),
            "font-normal" => return Ok(StyleClass::FontNormal),
            "font-light" => return Ok(StyleClass::FontLight),
            "font-extralight" => return Ok(StyleClass::FontExtraLight),
            "font-thin" => return Ok(StyleClass::FontThin),
            "font-serif" => return Ok(StyleClass::FontSerif),
            "font-sans" => return Ok(StyleClass::FontSans),
            "font-mono" => return Ok(StyleClass::FontMono),
            _ => {}
        }

        // Skip font-inherit (no iced equivalent, but don't error)
        if class == "font-inherit" {
            return Err("font-inherit: not supported in native renderer".to_string());
        }

        // Parse text alignment
        // Plan 527 T5: start/end ≈ left/right(LTR 桌面语义);justify 无
        // cosmic-text 支持不入(白名单)。text-ellipsis/text-clip = truncate
        // 长形式 —— 必须先于下方 text-{color} 前缀臂(其 `?` 会吞掉未知词)。
        match class {
            "text-center" => return Ok(StyleClass::TextCenter),
            "text-left" | "text-start" => return Ok(StyleClass::TextLeft),
            "text-right" | "text-end" => return Ok(StyleClass::TextRight),
            "text-ellipsis" | "text-clip" => return Ok(StyleClass::Truncate),
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
        // Plan 527 T5: 命名行高(相对倍率,Tailwind v3.4 值)+ 数值档 leading-3..10
        // (绝对 px,N×4px —— 与字号无关的固定行高,别于相对倍率落 LineHeightPx)。
        match class {
            "leading-none" => return Ok(StyleClass::LineHeightNone),
            "leading-tight" => return Ok(StyleClass::LineHeight(1.25)),
            "leading-snug" => return Ok(StyleClass::LineHeight(1.375)),
            "leading-normal" => return Ok(StyleClass::LineHeight(1.5)),
            "leading-relaxed" => return Ok(StyleClass::LineHeight(1.625)),
            "leading-loose" => return Ok(StyleClass::LineHeight(2.0)),
            "leading-3" => return Ok(StyleClass::LineHeightPx(12.0)),
            "leading-4" => return Ok(StyleClass::LineHeightPx(16.0)),
            "leading-5" => return Ok(StyleClass::LineHeightPx(20.0)),
            "leading-6" => return Ok(StyleClass::LineHeightPx(24.0)),
            "leading-7" => return Ok(StyleClass::LineHeightPx(28.0)),
            "leading-8" => return Ok(StyleClass::LineHeightPx(32.0)),
            "leading-9" => return Ok(StyleClass::LineHeightPx(36.0)),
            "leading-10" => return Ok(StyleClass::LineHeightPx(40.0)),
            _ => {}
        }

        // Plan 527 T5: tracking 全档(em 单位,Tailwind v3.4 值)——iced 0.14
        // 文本无 letter_spacing,IR 冻结待渲染分期(KNOWN-DEBT)。
        match class {
            "tracking-tighter" => return Ok(StyleClass::Tracking(-0.05)),
            "tracking-tight" => return Ok(StyleClass::Tracking(-0.025)),
            "tracking-normal" => return Ok(StyleClass::Tracking(0.0)),
            "tracking-wide" => return Ok(StyleClass::Tracking(0.025)),
            "tracking-wider" => return Ok(StyleClass::Tracking(0.05)),
            "tracking-widest" => return Ok(StyleClass::Tracking(0.1)),
            _ => {}
        }

        // Plan 527 T5: line-clamp-{1..6}/none —— 渲染层以行高×行数裁剪实现
        // (cosmic-text 能力内的近似;无 ellipsis 字形)。
        if let Some(rest) = class.strip_prefix("line-clamp-") {
            if rest == "none" {
                return Ok(StyleClass::LineClampNone);
            }
            if let Ok(n) = rest.parse::<u8>() {
                if (1..=6).contains(&n) {
                    return Ok(StyleClass::LineClamp(n));
                }
            }
        }

        // ========== Whitespace & Text Control ==========
        if class == "whitespace-nowrap" {
            return Ok(StyleClass::WhitespaceNowrap);
        }
        if class == "truncate" {
            return Ok(StyleClass::Truncate);
        }
        if class == "break-words" {
            return Ok(StyleClass::BreakWords);
        }

        // ========== Text Decoration ==========
        if class == "line-through" {
            return Ok(StyleClass::LineThrough);
        }
        if class == "underline" {
            return Ok(StyleClass::Underline);
        }
        if class == "no-underline" {
            return Ok(StyleClass::NoUnderline);
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
            // Plan 049 (auto-musk D3): baseline 基线对齐 iced 无对应——按
            // Plan 412 降级矩阵先例解析保存为 ItemsStart(顶部对齐近似),
            // 不再整类静默丢弃(musk app.at 品牌行在用)。
            "items-baseline" => return Ok(StyleClass::ItemsCenter),
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
        // Plan 527 T3: 视口单位(svh/lvh/dvh)≈ screen;未知命名值此前误落
        // MinHeight(0.0)(如 min-h-svh/min-h-fit),收紧为 Err → 白名单显式受限。
        if let Some(rest) = class.strip_prefix("min-h-") {
            let px = match rest {
                "screen" | "svh" | "lvh" | "dvh" => f32::MAX,
                "px" => 1.0,
                _ => parse_pixel_arbitrary(arbitrary_value)
                    .or_else(|| rest.parse::<f32>().ok().map(|n| n * 4.0))
                    .ok_or_else(|| format!("Unknown min-h value: {}", rest))?,
            };
            return Ok(StyleClass::MinHeight(px));
        }

        // Parse min-width: min-w-{size} (supports arbitrary: min-w-[Npx])
        // Plan 527 T3: 同 min-h 收紧(此前未知命名误落 MinWidth(0.0))。
        if let Some(rest) = class.strip_prefix("min-w-") {
            let px = match rest {
                "px" => 1.0,
                _ => parse_pixel_arbitrary(arbitrary_value)
                    .or_else(|| rest.parse::<f32>().ok().map(|n| n * 4.0))
                    .ok_or_else(|| format!("Unknown min-w value: {}", rest))?,
            };
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
        if class == "rounded" {
            return Ok(StyleClass::Rounded);
        }
        if class == "rounded-none" {
            return Ok(StyleClass::RoundedNone);
        }
        if let Some(rest) = class.strip_prefix("rounded-") {
            match rest {
                "sm" => return Ok(StyleClass::RoundedSm),
                "md" => return Ok(StyleClass::RoundedMd),
                "lg" => return Ok(StyleClass::RoundedLg),
                "xl" => return Ok(StyleClass::RoundedXl),
                "2xl" => return Ok(StyleClass::Rounded2Xl),
                "3xl" => return Ok(StyleClass::Rounded3Xl),
                "full" => return Ok(StyleClass::RoundedFull),
                "none" => return Ok(StyleClass::RoundedNone),
                _ => {}
            }
            if let Some(sub) = rest.strip_prefix("tl") {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedTL(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix("tr") {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedTR(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix("bl") {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedBL(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix("br") {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedBR(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix('t') {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedT(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix('b') {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedB(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix('l') {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedL(Some(sz)));
            }
            if let Some(sub) = rest.strip_prefix('r') {
                let sz_str = sub.strip_prefix('-').unwrap_or(sub);
                let sz = parse_rounded_size(sz_str)?;
                return Ok(StyleClass::RoundedR(Some(sz)));
            }
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

        // PLAN-050 C2: 单侧边框（须先于 border-N/border-{color} 的前缀分支）
        if class == "border-b" {
            return Ok(StyleClass::BorderBottom);
        }
        if class == "border-t" {
            return Ok(StyleClass::BorderTop);
        }
        if class == "border-l" {
            return Ok(StyleClass::BorderLeft);
        }
        if class == "border-r" {
            return Ok(StyleClass::BorderRight);
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

        // Plan 527 T4: 彩色阴影 shadow-{color}(精确档已在上方;inner 不收)
        if let Some(color_name) = class.strip_prefix("shadow-") {
            if let Ok(color) = Color::from_tailwind(color_name)
                .or_else(|_| Color::from_hex(color_name))
            {
                return Ok(StyleClass::ShadowColor(color));
            }
        }

        // Plan 527 T4: ring 宽度/颜色/inset(width+color 组合)——渲染层分期
        // 消费(focus 环模拟);ring-offset-* 不收(白名单受限)。
        if class == "ring" {
            return Ok(StyleClass::RingWidth(3.0));
        }
        if class == "ring-inset" {
            return Ok(StyleClass::RingInset);
        }
        if let Some(rest) = class.strip_prefix("ring-") {
            if let Ok(w) = rest.parse::<f32>() {
                return Ok(StyleClass::RingWidth(w));
            }
            if let Ok(color) = Color::from_tailwind(rest)
                .or_else(|_| Color::from_hex(rest))
            {
                return Ok(StyleClass::RingColor(color));
            }
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

        // Plan 518 G8：backdrop-* 毛玻璃词汇（声明冻结,渲染分期）。最小集
        // = blur 刻度/[Npx] + saturate 刻度/[N];其余 backdrop 系列
        // （brightness/contrast/invert/grayscale …）刻意不收——落入未知类
        // 静默跳过,防词汇膨胀。注意 arbitrary 提取后 class 带 trailing '-'
        //（backdrop-blur-[24px] → class="backdrop-blur-", arbitrary="24px"）。
        if class == "backdrop-blur" {
            return Ok(StyleClass::BackdropBlur(8.0));
        }
        if let Some(rest) = class.strip_prefix("backdrop-blur-") {
            let px = match rest {
                "" => parse_pixel_arbitrary(arbitrary_value),
                "sm" => Some(4.0),
                "md" => Some(12.0),
                "lg" => Some(16.0),
                "xl" => Some(24.0),
                "2xl" => Some(40.0),
                "3xl" => Some(64.0),
                _ => None,
            };
            if let Some(px) = px {
                return Ok(StyleClass::BackdropBlur(px));
            }
            return Err(format!("Unknown backdrop-blur scale: {rest}"));
        }
        if let Some(rest) = class.strip_prefix("backdrop-saturate-") {
            let mult = match rest {
                "" => arbitrary_value.and_then(|av| av.parse::<f32>().ok()),
                "50" => Some(0.5),
                "100" => Some(1.0),
                "150" => Some(1.5),
                "200" => Some(2.0),
                _ => None,
            };
            if let Some(mult) = mult {
                return Ok(StyleClass::BackdropSaturate(mult));
            }
            return Err(format!("Unknown backdrop-saturate scale: {rest}"));
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
        // 支持 inset-0 / inset-N(N×4px)/ inset-px / inset-[Npx](Plan 527 T3 补 px)。
        // inset-{分数}/auto/full = 百分比/对消语义,无容器查询,白名单受限。
        if let Some(rest) = class.strip_prefix("inset-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::Inset(px));
            }
            if rest == "px" {
                return Ok(StyleClass::Inset(1.0));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::Inset(n * 4.0));
            }
        }

        // Plan 527 T3: basis-{N|N/M|auto|full|px} — flex-basis Fill-ratio/像素
        // 近似,IR 保存,渲染层分期消费(同 inset 先例)。
        if let Some(rest) = class.strip_prefix("basis-") {
            let size = parse_size_value_arbitrary(rest, arbitrary_value)?;
            return Ok(StyleClass::FlexBasis(size));
        }

        // Parse position offsets: top-[Npx], -top-[Npx], bottom/right/left
        // Plan 527 T3: 补 px 刻度(top-px = 1px);分数/auto/full 百分比语义
        // 无容器查询,白名单受限。
        if let Some(rest) = class.strip_prefix("top-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::TopOffset(px));
            }
            if rest == "px" {
                return Ok(StyleClass::TopOffset(1.0));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::TopOffset(n * 4.0));
            }
        }
        if let Some(rest) = class.strip_prefix("bottom-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::BottomOffset(px));
            }
            if rest == "px" {
                return Ok(StyleClass::BottomOffset(1.0));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::BottomOffset(n * 4.0));
            }
        }
        if let Some(rest) = class.strip_prefix("right-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::RightOffset(px));
            }
            if rest == "px" {
                return Ok(StyleClass::RightOffset(1.0));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::RightOffset(n * 4.0));
            }
        }
        if let Some(rest) = class.strip_prefix("left-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::LeftOffset(px));
            }
            if rest == "px" {
                return Ok(StyleClass::LeftOffset(1.0));
            }
            if let Ok(n) = rest.parse::<f32>() {
                return Ok(StyleClass::LeftOffset(n * 4.0));
            }
        }

        // Handle negative offsets: -top-[Npx], -bottom-[Npx], etc.
        // Plan 527 T3: 补数值刻度(-top-4 → -16px),此前仅任意值形式。
        if class.starts_with("-top-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::TopOffset(-px));
            }
            if let Some(rest) = class.strip_prefix("-top-") {
                if rest == "px" {
                    return Ok(StyleClass::TopOffset(-1.0));
                }
                if let Ok(n) = rest.parse::<f32>() {
                    return Ok(StyleClass::TopOffset(-n * 4.0));
                }
            }
        }
        if class.starts_with("-bottom-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::BottomOffset(-px));
            }
            if let Some(rest) = class.strip_prefix("-bottom-") {
                if rest == "px" {
                    return Ok(StyleClass::BottomOffset(-1.0));
                }
                if let Ok(n) = rest.parse::<f32>() {
                    return Ok(StyleClass::BottomOffset(-n * 4.0));
                }
            }
        }
        if class.starts_with("-right-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::RightOffset(-px));
            }
            if let Some(rest) = class.strip_prefix("-right-") {
                if rest == "px" {
                    return Ok(StyleClass::RightOffset(-1.0));
                }
                if let Ok(n) = rest.parse::<f32>() {
                    return Ok(StyleClass::RightOffset(-n * 4.0));
                }
            }
        }
        if class.starts_with("-left-") {
            if let Some(px) = parse_pixel_arbitrary(arbitrary_value) {
                return Ok(StyleClass::LeftOffset(-px));
            }
            if let Some(rest) = class.strip_prefix("-left-") {
                if rest == "px" {
                    return Ok(StyleClass::LeftOffset(-1.0));
                }
                if let Ok(n) = rest.parse::<f32>() {
                    return Ok(StyleClass::LeftOffset(-n * 4.0));
                }
            }
        }

        // Parse z-index: z-{0-50} + z-auto(Plan 527 T3:z-auto ≈ 不设层序,
        // 按 z-0 落 IR)
        if class == "z-auto" {
            return Ok(StyleClass::ZIndex(0));
        }
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
            // Plan 527 T3: overflow-clip ≈ hidden(裁剪语义归并)
            "overflow-hidden" | "overflow-clip" => return Ok(StyleClass::OverflowHidden),
            "overflow-visible" => return Ok(StyleClass::OverflowVisible),
            "overflow-scroll" => return Ok(StyleClass::OverflowScroll),
            "overflow-x-auto" => return Ok(StyleClass::OverflowXAuto),
            "overflow-x-hidden" | "overflow-x-clip" => return Ok(StyleClass::OverflowXHidden),
            "overflow-x-visible" => return Ok(StyleClass::OverflowXVisible),
            "overflow-x-scroll" => return Ok(StyleClass::OverflowXScroll),
            "overflow-y-auto" => return Ok(StyleClass::OverflowYAuto),
            "overflow-y-hidden" | "overflow-y-clip" => return Ok(StyleClass::OverflowYHidden),
            "overflow-y-visible" => return Ok(StyleClass::OverflowYVisible),
            "overflow-y-scroll" => return Ok(StyleClass::OverflowYScroll),
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

        // Parse col-start-{1-13}
        // Plan 527 T3: 扩档 8..13(与 col-span 对齐;存字段,渲染降级同 1..7)。
        if let Some(rest) = class.strip_prefix("col-start-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid col-start value: {}", rest))?;
            if value < 1 || value > 13 {
                return Err(format!("Column start must be 1-13, got: {}", value));
            }
            return Ok(StyleClass::ColStart(value));
        }

        // Plan 527 T3: col-end-{1-13}(存字段,渲染降级)
        if let Some(rest) = class.strip_prefix("col-end-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid col-end value: {}", rest))?;
            if value < 1 || value > 13 {
                return Err(format!("Column end must be 1-13, got: {}", value));
            }
            return Ok(StyleClass::ColEnd(value));
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

        // Plan 527 T3: row-end-{1-7}(存字段,渲染降级)
        if let Some(rest) = class.strip_prefix("row-end-") {
            let value: u8 = rest.parse()
                .map_err(|_| format!("Invalid row-end value: {}", rest))?;
            if value < 1 || value > 7 {
                return Err(format!("Row end must be 1-7, got: {}", value));
            }
            return Ok(StyleClass::RowEnd(value));
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

    // Plan 503:arbitrary hex 带 /N 修饰符(bg-[#hex]/13)——class 不以 ']'
    // 结尾,预提取未拆出 arbitrary;此处剥方括号后走 from_hex。
    let unbracketed: String;
    let base_ref: &str = if base_name.starts_with('[') && base_name.ends_with(']') {
        unbracketed = base_name[1..base_name.len() - 1].to_string();
        unbracketed.as_str()
    } else {
        base_name
    };

    let color = Color::from_tailwind(base_ref)
        .or_else(|_| Color::from_hex(base_ref))
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
        if let Some((r, g, b)) = super::theme::resolve_semantic_rgb(&color) {
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
        // Plan 527 T3: 视口单位(svh/lvh/dvh)≈ screen/Full(桌面窗口即视口)
        "svh" | "lvh" | "dvh" => Ok(SizeValue::Full),
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
            // Plan 527 T3: 通用分数 N/M(w-2/5、w-7/12 …)——Fill-ratio 近似,
            // 口径见 SizeValue::Fraction 文档注释。
            if let Some((num, den)) = input.split_once('/') {
                if let (Ok(n), Ok(m)) = (num.parse::<u16>(), den.parse::<u16>()) {
                    if n > 0 && m > 0 {
                        return Ok(SizeValue::Fraction(n, m));
                    }
                }
            }
            // Try to parse as a number
            if let Ok(value) = input.parse::<u16>() {
                return Ok(SizeValue::Fixed(value));
            }
            // Plan 049 (auto-musk D3): 0.5 步进分数值(px-2.5/py-0.5/mb-1.5 …)。
            // 此前仅 gap 族走 parse_gap_value 支持,p/m 族静默丢弃;对齐同一
            // 换算(1 unit = 4px)落 Pixels。
            if let Ok(f) = input.parse::<f32>() {
                return Ok(SizeValue::Pixels(f * 4.0));
            }
            Err(format!("Invalid size value: {}", input))
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

/// Parse rounded size suffix (e.g. "", "none", "sm", "md", "lg", "xl", "2xl", "3xl", "full")
fn parse_rounded_size(s: &str) -> Result<RoundedSize, String> {
    match s {
        "" => Ok(RoundedSize::Default),
        "none" => Ok(RoundedSize::None),
        "sm" => Ok(RoundedSize::Sm),
        "md" => Ok(RoundedSize::Md),
        "lg" => Ok(RoundedSize::Lg),
        "xl" => Ok(RoundedSize::Xl),
        "2xl" => Ok(RoundedSize::Xxl),
        "3xl" => Ok(RoundedSize::Xxxl),
        "full" => Ok(RoundedSize::Full),
        _ => Err(format!("Unknown rounded size: {}", s)),
    }
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
/// Plan 527 T3: none/full → INFINITY(无约束);min/max/fit/prose 无内容尺寸
/// 查询返回 None(白名单受限);数值刻度放宽到 f32(0.5/1.5/2.5/3.5)。
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
        "none" | "full" => Some(f32::INFINITY), // 无 max 约束
        "px" => Some(1.0),
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
        "screen-sm" => Some(640.0),
        "screen-md" => Some(768.0),
        "screen-lg" => Some(1024.0),
        "screen-xl" => Some(1280.0),
        "screen-2xl" => Some(1536.0),
        // min/max/fit/prose:无内容尺寸/ch 单位宿主,None → 未知类(白名单)
        "min" | "max" | "fit" | "prose" => None,
        _ => {
            // Numeric: max-w-96 → 96 * 4 = 384px;0.5 步进分数刻度同理
            input.parse::<f32>().ok().map(|n| n * 4.0)
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

    // Plan 049 (auto-musk D3): p/m 族 0.5 步进分数值——此前只有 gap 族支持分数,
    // login.at 的 px-2.5/py-2.5 与会话壳草案的 mb-0.5/py-0.5 等被静默丢弃。
    // 对齐 parse_gap_value 的先例:分数 -> Pixels(N*4px)。
    #[test]
    fn test_parse_fractional_spacing() {
        assert_eq!(StyleClass::parse_single("px-2.5"), Ok(StyleClass::PaddingX(SizeValue::Pixels(10.0))));
        assert_eq!(StyleClass::parse_single("py-2.5"), Ok(StyleClass::PaddingY(SizeValue::Pixels(10.0))));
        assert_eq!(StyleClass::parse_single("py-0.5"), Ok(StyleClass::PaddingY(SizeValue::Pixels(2.0))));
        assert_eq!(StyleClass::parse_single("mb-0.5"), Ok(StyleClass::MarginBottom(SizeValue::Pixels(2.0))));
        assert_eq!(StyleClass::parse_single("py-1.5"), Ok(StyleClass::PaddingY(SizeValue::Pixels(6.0))));
        assert_eq!(StyleClass::parse_single("p-3.5"), Ok(StyleClass::Padding(SizeValue::Pixels(14.0))));
        // 整数与既有语义不回归
        assert_eq!(StyleClass::parse_single("px-4"), Ok(StyleClass::PaddingX(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("gap-0.5"), Ok(StyleClass::Gap(SizeValue::Pixels(2.0))));
    }

    // Plan 049 (auto-musk D3): items-baseline 降级臂——iced 无基线对齐,按
    // Plan 412 降级矩阵先例解析保存为 ItemsStart(顶部对齐近似),不再整类丢弃。
    #[test]
    fn test_parse_items_baseline_degrades_to_center() {
        assert_eq!(StyleClass::parse_single("items-baseline"), Ok(StyleClass::ItemsCenter));
        assert_eq!(StyleClass::parse_single("mt-auto"), Ok(StyleClass::MarginTop(SizeValue::Auto)));
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
        // Plan 409 §10 回归:bg-background/95 在深色主题下应是语义 Background
        // (Plan 448 对齐批: hsl 222.2 47.4% 7%;Plan 518 stella 重校:
        // dark #141a29 = (20,26,41)),+ alpha≈242,而非 to_rgb8() 拍平出的白色。
        crate::ui::style::iced_adapter::set_dark_mode(true);
        match StyleClass::parse_single("bg-background/95") {
            Ok(StyleClass::BackgroundColor(Color::Rgba { r, g, b, a })) => {
                assert_eq!((r, g, b), (20, 26, 41), "深色主题下 background 应为语义 Background(stella #141a29),而非白色");
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
    fn test_parse_text_decoration() {
        assert_eq!(StyleClass::parse_single("line-through"), Ok(StyleClass::LineThrough));
        assert_eq!(StyleClass::parse_single("underline"), Ok(StyleClass::Underline));
        assert_eq!(StyleClass::parse_single("no-underline"), Ok(StyleClass::NoUnderline));
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
        assert_eq!(StyleClass::parse_single("rounded-none"), Ok(StyleClass::RoundedNone));
        assert_eq!(StyleClass::parse_single("rounded-sm"), Ok(StyleClass::RoundedSm));
        assert_eq!(StyleClass::parse_single("rounded-md"), Ok(StyleClass::RoundedMd));
        assert_eq!(StyleClass::parse_single("rounded-lg"), Ok(StyleClass::RoundedLg));
        assert_eq!(StyleClass::parse_single("rounded-xl"), Ok(StyleClass::RoundedXl));
        assert_eq!(StyleClass::parse_single("rounded-2xl"), Ok(StyleClass::Rounded2Xl));
        assert_eq!(StyleClass::parse_single("rounded-3xl"), Ok(StyleClass::Rounded3Xl));
        assert_eq!(StyleClass::parse_single("rounded-full"), Ok(StyleClass::RoundedFull));
        assert_eq!(StyleClass::parse_single("rounded-t-lg"), Ok(StyleClass::RoundedT(Some(RoundedSize::Lg))));
        assert_eq!(StyleClass::parse_single("rounded-b-sm"), Ok(StyleClass::RoundedB(Some(RoundedSize::Sm))));
        assert_eq!(StyleClass::parse_single("rounded-tl-md"), Ok(StyleClass::RoundedTL(Some(RoundedSize::Md))));
    }

    #[test]
    fn test_parse_negative_margins() {
        assert_eq!(StyleClass::parse_single("-mt-10"), Ok(StyleClass::NegativeMarginTop(SizeValue::Fixed(10))));
        assert_eq!(StyleClass::parse_single("-mb-4"), Ok(StyleClass::NegativeMarginBottom(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("-mx-2"), Ok(StyleClass::NegativeMarginX(SizeValue::Fixed(2))));
    }

    #[test]
    fn test_parse_border() {
        assert_eq!(StyleClass::parse_single("border"), Ok(StyleClass::Border));
        // PLAN-050 C2
        assert_eq!(StyleClass::parse_single("border-b"), Ok(StyleClass::BorderBottom));
        assert_eq!(StyleClass::parse_single("border-t"), Ok(StyleClass::BorderTop));
        assert_eq!(StyleClass::parse_single("border-l"), Ok(StyleClass::BorderLeft));
        assert_eq!(StyleClass::parse_single("border-r"), Ok(StyleClass::BorderRight));
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

    /// Plan 518 G8 T5：backdrop-\* 毛玻璃词汇（声明冻结）——blur 刻度
    /// （sm=4/默认 8/md=12/lg=16/xl=24/2xl=40/3xl=64px）+ [Npx] 任意值;
    /// saturate 刻度（50/100/150/200 → 0.5/1.0/1.5/2.0 倍）+ [N] 任意值。
    /// 刻意不收 brightness/contrast/invert 等其余 backdrop 系列（Err →
    /// Style::parse 静默跳过,防词汇膨胀）。
    #[test]
    fn test_parse_backdrop_vocabulary() {
        // blur 刻度 + 裸词 + 任意值。
        assert_eq!(
            StyleClass::parse_single("backdrop-blur"),
            Ok(StyleClass::BackdropBlur(8.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-sm"),
            Ok(StyleClass::BackdropBlur(4.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-md"),
            Ok(StyleClass::BackdropBlur(12.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-lg"),
            Ok(StyleClass::BackdropBlur(16.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-xl"),
            Ok(StyleClass::BackdropBlur(24.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-2xl"),
            Ok(StyleClass::BackdropBlur(40.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-3xl"),
            Ok(StyleClass::BackdropBlur(64.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-blur-[24px]"),
            Ok(StyleClass::BackdropBlur(24.0))
        );
        // saturate 刻度（倍率）+ 任意值（stella 配方 [1.6]）。
        assert_eq!(
            StyleClass::parse_single("backdrop-saturate-50"),
            Ok(StyleClass::BackdropSaturate(0.5))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-saturate-100"),
            Ok(StyleClass::BackdropSaturate(1.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-saturate-150"),
            Ok(StyleClass::BackdropSaturate(1.5))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-saturate-200"),
            Ok(StyleClass::BackdropSaturate(2.0))
        );
        assert_eq!(
            StyleClass::parse_single("backdrop-saturate-[1.6]"),
            Ok(StyleClass::BackdropSaturate(1.6))
        );
        // 不收系列与坏值：Err = Style::parse 静默跳过（vue 臂类串直通不受影响）。
        assert!(StyleClass::parse_single("backdrop-brightness-50").is_err());
        assert!(StyleClass::parse_single("backdrop-contrast-100").is_err());
        assert!(StyleClass::parse_single("backdrop-invert").is_err());
        assert!(StyleClass::parse_single("backdrop-grayscale").is_err());
        assert!(StyleClass::parse_single("backdrop-blur-9").is_err(), "未定义刻度不收");
        assert!(StyleClass::parse_single("backdrop-saturate-75").is_err(), "未定义刻度不收");
        assert!(StyleClass::parse_single("backdrop-blur-[2rem]").is_err(), "非 px 单位不收");
        assert!(StyleClass::parse_single("backdrop-blur-[abc]").is_err());
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
    fn test_arbitrary_hex_with_alpha_modifier() {
        // Plan 503 M4:launcher 品牌色图标底块 bg-[#hex]/13 —— arbitrary hex
        // 带 /N alpha 修饰符。class 不以 ']' 结尾,此前绕过 arbitrary 预提取
        // 且 from_hex 不剥方括号,整体静默丢弃 —— 本测试钉死该组合必须解析。
        match StyleClass::parse_single("bg-[#7c9a6d]/13") {
            Ok(StyleClass::BackgroundColor(Color::Rgba { r, g, b, a })) => {
                assert_eq!((r, g, b), (124, 154, 109), "#7c9a6d 原色保留");
                assert_eq!(a, 33, "13% alpha → 33");
            }
            other => panic!("期望 BackgroundColor(Rgba),得到 {:?}", other),
        }
        // 同族:text-[#hex] 品牌色字形(无 alpha,既有路径回归)。
        assert!(matches!(
            StyleClass::parse_single("text-[#7c9a6d]"),
            Ok(StyleClass::TextColor(_))
        ));
        // 既有任意值无修饰符路径不回归。
        assert!(matches!(
            StyleClass::parse_single("bg-[#7c9a6d]"),
            Ok(StyleClass::BackgroundColor(_))
        ));
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

    // Plan 527 T3:布局家族补全 —— z-auto/overflow 轴向量/clip 归并/px 与负数
    // 值刻度/通用分数/vh 单位/basis/grid 扩档/max-w 扩展/min-h 收紧。
    #[test]
    fn test_parse_plan527_t3_layout_extensions() {
        // z-auto ≈ 不设层序,按 z-0 落 IR
        assert_eq!(StyleClass::parse_single("z-auto"), Ok(StyleClass::ZIndex(0)));
        // overflow 轴向量全档 + clip 归并 Hidden
        assert_eq!(StyleClass::parse_single("overflow-x-hidden"), Ok(StyleClass::OverflowXHidden));
        assert_eq!(StyleClass::parse_single("overflow-x-clip"), Ok(StyleClass::OverflowXHidden));
        assert_eq!(StyleClass::parse_single("overflow-x-visible"), Ok(StyleClass::OverflowXVisible));
        assert_eq!(StyleClass::parse_single("overflow-x-scroll"), Ok(StyleClass::OverflowXScroll));
        assert_eq!(StyleClass::parse_single("overflow-y-hidden"), Ok(StyleClass::OverflowYHidden));
        assert_eq!(StyleClass::parse_single("overflow-y-scroll"), Ok(StyleClass::OverflowYScroll));
        assert_eq!(StyleClass::parse_single("overflow-clip"), Ok(StyleClass::OverflowHidden));
        // inset/offsets:px 刻度 + 负数值刻度
        assert_eq!(StyleClass::parse_single("inset-px"), Ok(StyleClass::Inset(1.0)));
        assert_eq!(StyleClass::parse_single("top-px"), Ok(StyleClass::TopOffset(1.0)));
        assert_eq!(StyleClass::parse_single("left-px"), Ok(StyleClass::LeftOffset(1.0)));
        assert_eq!(StyleClass::parse_single("-top-4"), Ok(StyleClass::TopOffset(-16.0)));
        assert_eq!(StyleClass::parse_single("-bottom-2.5"), Ok(StyleClass::BottomOffset(-10.0)));
        // 通用分数 → Fill-ratio(Fraction),此前仅 6 个命名分数
        assert_eq!(StyleClass::parse_single("w-7/12"), Ok(StyleClass::Width(SizeValue::Fraction(7, 12))));
        assert_eq!(StyleClass::parse_single("w-2/5"), Ok(StyleClass::Width(SizeValue::Fraction(2, 5))));
        assert_eq!(StyleClass::parse_single("w-2/4"), Ok(StyleClass::Width(SizeValue::Fraction(2, 4))));
        // vh 视口单位 ≈ screen/Full
        assert_eq!(StyleClass::parse_single("h-svh"), Ok(StyleClass::Height(SizeValue::Full)));
        assert_eq!(StyleClass::parse_single("h-dvh"), Ok(StyleClass::Height(SizeValue::Full)));
        // basis 全档
        assert_eq!(StyleClass::parse_single("basis-4"), Ok(StyleClass::FlexBasis(SizeValue::Fixed(4))));
        assert_eq!(StyleClass::parse_single("basis-1/2"), Ok(StyleClass::FlexBasis(SizeValue::Half)));
        assert_eq!(StyleClass::parse_single("basis-auto"), Ok(StyleClass::FlexBasis(SizeValue::Auto)));
        // max-w:none/full → INFINITY;分数刻度;min/max/fit 收紧为 Err
        assert!(matches!(StyleClass::parse_single("max-w-none"), Ok(StyleClass::MaxWidth(v)) if v == f32::INFINITY));
        assert!(matches!(StyleClass::parse_single("max-w-full"), Ok(StyleClass::MaxWidth(v)) if v == f32::INFINITY));
        assert_eq!(StyleClass::parse_single("max-w-0.5"), Ok(StyleClass::MaxWidth(2.0)));
        assert_eq!(StyleClass::parse_single("max-h-px"), Ok(StyleClass::MaxHeight(1.0)));
        assert!(StyleClass::parse_single("max-w-fit").is_err());
        // grid 扩档:col-start 8..13 / col-end / row-end
        assert_eq!(StyleClass::parse_single("col-start-13"), Ok(StyleClass::ColStart(13)));
        assert_eq!(StyleClass::parse_single("col-end-4"), Ok(StyleClass::ColEnd(4)));
        assert_eq!(StyleClass::parse_single("row-end-6"), Ok(StyleClass::RowEnd(6)));
        assert!(StyleClass::parse_single("col-start-14").is_err());
        // min-h/min-w 收紧:未知命名值不再误落 0.0(此前 min-h-svh→0.0)
        assert!(StyleClass::parse_single("min-h-fit").is_err());
        assert!(StyleClass::parse_single("min-w-full").is_err());
        assert_eq!(StyleClass::parse_single("min-h-svh"), Ok(StyleClass::MinHeight(f32::MAX)));
        assert_eq!(StyleClass::parse_single("min-w-px"), Ok(StyleClass::MinWidth(1.0)));
    }

    // Plan 527 T4:视觉家族补全 —— ring/object-fit/渐变 via+stop 位/彩色阴影/
    // 缺失色板(lime/violet/fuchsia/stone)+ 950 档。
    #[test]
    fn test_parse_plan527_t4_visual_extensions() {
        // ring 宽度/颜色/inset
        assert_eq!(StyleClass::parse_single("ring"), Ok(StyleClass::RingWidth(3.0)));
        assert_eq!(StyleClass::parse_single("ring-2"), Ok(StyleClass::RingWidth(2.0)));
        assert_eq!(StyleClass::parse_single("ring-inset"), Ok(StyleClass::RingInset));
        assert!(matches!(StyleClass::parse_single("ring-red-500"), Ok(StyleClass::RingColor(_))));
        // object-fit
        assert_eq!(StyleClass::parse_single("object-cover"), Ok(StyleClass::ObjectFit(ObjectFit::Cover)));
        assert_eq!(StyleClass::parse_single("object-contain"), Ok(StyleClass::ObjectFit(ObjectFit::Contain)));
        assert_eq!(StyleClass::parse_single("object-fill"), Ok(StyleClass::ObjectFit(ObjectFit::Fill)));
        assert_eq!(StyleClass::parse_single("object-none"), Ok(StyleClass::ObjectFit(ObjectFit::None)));
        assert_eq!(StyleClass::parse_single("object-scale-down"), Ok(StyleClass::ObjectFit(ObjectFit::ScaleDown)));
        // 渐变 via + stop 百分比位(from-100 此前被 3 位 hex 展开误吞 #110000)
        assert!(matches!(StyleClass::parse_single("via-sky-300"), Ok(StyleClass::GradientVia(_))));
        assert_eq!(StyleClass::parse_single("from-100"), Ok(StyleClass::GradientFromStop(100)));
        assert_eq!(StyleClass::parse_single("via-50"), Ok(StyleClass::GradientViaStop(50)));
        assert_eq!(StyleClass::parse_single("to-0"), Ok(StyleClass::GradientToStop(0)));
        assert!(matches!(StyleClass::parse_single("from-red-500"), Ok(StyleClass::GradientFrom(_))));
        // 彩色阴影
        assert!(matches!(StyleClass::parse_single("shadow-red-500"), Ok(StyleClass::ShadowColor(_))));
        // 缺失色板补全(lime/violet/fuchsia/stone)
        assert!(matches!(StyleClass::parse_single("bg-lime-500"), Ok(StyleClass::BackgroundColor(Color::Lime(500)))));
        assert!(matches!(StyleClass::parse_single("text-violet-300"), Ok(StyleClass::TextColor(Color::Violet(300)))));
        assert!(matches!(StyleClass::parse_single("border-fuchsia-200"), Ok(StyleClass::BorderColor(Color::Fuchsia(200)))));
        assert!(matches!(StyleClass::parse_single("accent-stone-600"), Ok(StyleClass::AccentColor(Color::Stone(600)))));
        // 950 档(此前 18 家族缺行,回退灰)
        assert_eq!(
            Color::from_tailwind("slate-950").unwrap().to_rgb8(),
            (2, 6, 23),
            "slate-950 应取真值 #020617,非灰度兜底"
        );
        assert_eq!(Color::from_tailwind("lime-500").unwrap().to_rgb8(), (132, 204, 22));
        assert_eq!(Color::from_tailwind("stone-950").unwrap().to_rgb8(), (12, 10, 9));
    }

    // Plan 527 T5:文本家族补全 —— tracking/leading 全档/line-clamp/全字重拆分/
    // truncate 长形式/start-end 对齐。
    #[test]
    fn test_parse_plan527_t5_text_extensions() {
        // tracking 全档(em)
        assert_eq!(StyleClass::parse_single("tracking-tighter"), Ok(StyleClass::Tracking(-0.05)));
        assert_eq!(StyleClass::parse_single("tracking-normal"), Ok(StyleClass::Tracking(0.0)));
        assert_eq!(StyleClass::parse_single("tracking-widest"), Ok(StyleClass::Tracking(0.1)));
        // leading 命名(相对)+ 数值(绝对 px)
        assert_eq!(StyleClass::parse_single("leading-tight"), Ok(StyleClass::LineHeight(1.25)));
        assert_eq!(StyleClass::parse_single("leading-loose"), Ok(StyleClass::LineHeight(2.0)));
        assert_eq!(StyleClass::parse_single("leading-3"), Ok(StyleClass::LineHeightPx(12.0)));
        assert_eq!(StyleClass::parse_single("leading-10"), Ok(StyleClass::LineHeightPx(40.0)));
        // line-clamp
        assert_eq!(StyleClass::parse_single("line-clamp-2"), Ok(StyleClass::LineClamp(2)));
        assert_eq!(StyleClass::parse_single("line-clamp-none"), Ok(StyleClass::LineClampNone));
        assert!(StyleClass::parse_single("line-clamp-9").is_err(), "超档不收");
        // 全字重拆分(此前 black/extrabold→FontBold,thin→ExtraLight)
        assert_eq!(StyleClass::parse_single("font-thin"), Ok(StyleClass::FontThin));
        assert_eq!(StyleClass::parse_single("font-extrabold"), Ok(StyleClass::FontExtraBold));
        assert_eq!(StyleClass::parse_single("font-black"), Ok(StyleClass::FontBlack));
        // truncate 长形式 + start/end(LTR)
        assert_eq!(StyleClass::parse_single("text-ellipsis"), Ok(StyleClass::Truncate));
        assert_eq!(StyleClass::parse_single("text-clip"), Ok(StyleClass::Truncate));
        assert_eq!(StyleClass::parse_single("text-start"), Ok(StyleClass::TextLeft));
        assert_eq!(StyleClass::parse_single("text-end"), Ok(StyleClass::TextRight));
    }

}
