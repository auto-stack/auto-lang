// Iced Adapter - Convert StyleClass IR to Iced style objects
//
// This adapter translates the unified StyleClass IR into Iced-specific
// style objects for styling components.

use super::{Style, StyleClass, SizeValue, Color};

/// Iced style representation
///
/// Iced has a more traditional style system with separate Style, Theme, and layout objects.
/// This adapter converts StyleClass IR into Iced-compatible structures.
///
/// NOTE: Iced does not support margin - margin-related classes will be ignored
pub struct IcedStyle {
    // Spacing (L1 + L2)
    pub padding: Option<f32>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
    pub padding_top: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,
    pub padding_right: Option<f32>,
    // NOTE: Iced doesn't support margin - these fields are handled as external spacing
    pub margin: Option<f32>,        // Not supported by Iced
    pub margin_x: Option<f32>,       // Not supported by Iced
    pub margin_y: Option<f32>,       // Not supported by Iced
    pub margin_top: Option<f32>,      // Converted to external top spacing
    pub margin_left_auto: bool,       // ml-auto: push element to right in row
    pub margin_right_auto: bool,      // mr-auto: push element to left in row
    pub gap: Option<f32>,

    // Colors (L1)
    pub background_color: Option<iced::Color>,
    pub text_color: Option<iced::Color>,
    pub gradient_dir: Option<crate::ui::style::class::GradientDir>,
    pub gradient_from: Option<iced::Color>,
    pub gradient_to: Option<iced::Color>,

    // Sizing (L1)
    pub width: Option<IcedSize>,
    pub height: Option<IcedSize>,
    pub max_width: Option<f32>,  // pixels
    pub max_height: Option<f32>, // pixels

    // Border Radius (L1 + L2)
    pub rounded: bool,
    pub border_radius: Option<f32>,

    // Border (L2)
    pub border: bool,
    pub border_width: Option<f32>,
    pub border_color: Option<iced::Color>,

    // Typography (L2)
    pub font_size: Option<IcedFontSize>,
    pub font_weight: Option<IcedFontWeight>,
    pub text_align: Option<IcedTextAlign>,

    // Effects (L3)
    pub shadow: bool,
    pub shadow_size: Option<IcedShadowSize>,
    pub opacity: Option<f32>,

    // Position (L3)
    // NOTE: Iced doesn't support absolute positioning - these fields are ignored
    pub position: Option<IcedPosition>,
    pub z_index: Option<i16>,       // Not supported by Iced

    // Overflow (L3)
    pub overflow_x: Option<IcedOverflow>,
    pub overflow_y: Option<IcedOverflow>,

    // Layout (L1 Core)
    pub align_items: Option<IcedAlign>,
    pub justify_content: Option<IcedJustify>,

    // Grid (L3)
    // NOTE: Iced doesn't support grid layout - these fields are ignored
    pub grid: bool,                 // Not supported by Iced
    pub grid_cols: Option<u8>,      // Not supported by Iced
    pub grid_rows: Option<u8>,      // Not supported by Iced
    pub col_span: Option<u8>,       // Not supported by Iced
    pub row_span: Option<u8>,       // Not supported by Iced
    pub col_start: Option<u8>,      // Not supported by Iced
    pub row_start: Option<u8>,      // Not supported by Iced

    // Extended sizing
    pub min_height: Option<f32>,
    pub min_width: Option<f32>,

    // Extended typography
    pub font_size_arbitrary: Option<f32>,
    pub line_height: Option<f32>,

    // Shadow extended
    pub shadow_arbitrary: Option<String>,

    // Position offsets (not fully supported by Iced)
    pub top_offset: Option<f32>,
    pub bottom_offset: Option<f32>,
    pub right_offset: Option<f32>,
    pub left_offset: Option<f32>,

    // Transform (not supported by Iced, stored for reference)
    pub rotate: Option<f32>,

    // Visibility
    pub hidden: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedAlign {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedJustify {
    Start,
    Center,
    End,
    Between,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedShadowSize {
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedPosition {
    Relative,
    Absolute, // Not supported by Iced
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedOverflow {
    Auto,
    Hidden,
    Visible,
    Scroll,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedSize {
    Full,
    FillPortion(u16),
    Fixed(f32),
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedFontSize {
    Xs,   // 12px
    Sm,   // 14px
    Base, // 16px
    Lg,   // 18px
    Xl,   // 20px
    Xxl,  // 24px
    X3xl, // 30px
    X4xl, // 36px
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedFontWeight {
    Normal,
    Medium,
    Bold,
    Light,
    ExtraLight,
    SemiBold,
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedTextAlign {
    Left,
    Center,
    Right,
}

impl IcedStyle {
    /// Convert a Style to IcedStyle
    pub fn from_style(style: &Style) -> Self {
        let mut iced_style = IcedStyle {
            padding: None,
            padding_x: None,
            padding_y: None,
            padding_top: None,
            padding_bottom: None,
            padding_left: None,
            padding_right: None,
            margin: None,      // Not supported by Iced
            margin_x: None,    // Not supported by Iced
            margin_y: None,    // Not supported by Iced
            margin_top: None,
            margin_left_auto: false,
            margin_right_auto: false,
            gap: None,
            background_color: None,
            text_color: None,
            gradient_dir: None,
            gradient_from: None,
            gradient_to: None,
            width: None,
            height: None,
            max_width: None,
            max_height: None,
            rounded: false,
            border_radius: None,
            border: false,
            border_width: None,
            border_color: None,
            font_size: None,
            font_weight: None,
            text_align: None,
            // L3
            shadow: false,
            shadow_size: None,
            opacity: None,
            position: None,
            z_index: None,      // Not supported by Iced
            overflow_x: None,
            overflow_y: None,
            align_items: None,
            justify_content: None,
            grid: false,        // Not supported by Iced
            grid_cols: None,    // Not supported by Iced
            grid_rows: None,    // Not supported by Iced
            col_span: None,     // Not supported by Iced
            row_span: None,     // Not supported by Iced
            col_start: None,    // Not supported by Iced
            row_start: None,    // Not supported by Iced
            // Extended sizing
            min_height: None,
            min_width: None,
            // Extended typography
            font_size_arbitrary: None,
            line_height: None,
            // Shadow extended
            shadow_arbitrary: None,
            // Position offsets
            top_offset: None,
            bottom_offset: None,
            right_offset: None,
            left_offset: None,
            // Transform
            rotate: None,
            // Visibility
            hidden: false,
        };

        for class in &style.classes {
            iced_style.apply_class(class);
        }

        // margin_top is kept as-is — the renderer wraps the element in a container
        // with external top padding to simulate margin-top, separate from internal padding.
        // This avoids visual-wrap elements (gradient cards, bordered cols) absorbing
        // mt-* into their internal padding.

        iced_style
    }

    /// Apply a single StyleClass to this IcedStyle
    fn apply_class(&mut self, class: &StyleClass) {
        match class {
            // ========== Spacing (L1 + L2) ==========
            StyleClass::Padding(size) => {
                self.padding = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingX(size) => {
                self.padding_x = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingY(size) => {
                self.padding_y = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingTop(size) => {
                self.padding_top = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingBottom(size) => {
                self.padding_bottom = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingLeft(size) => {
                self.padding_left = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingRight(size) => {
                self.padding_right = Some(size.to_pixels() as f32);
            }
            StyleClass::Margin(size) => {
                // Iced doesn't support margin - store but will be ignored
                self.margin = Some(size.to_pixels() as f32);
            }
            StyleClass::MarginX(size) => {
                // Iced doesn't support margin - store but will be ignored
                self.margin_x = Some(size.to_pixels() as f32);
            }
            StyleClass::MarginY(size) => {
                // Iced doesn't support margin - store but will be ignored
                self.margin_y = Some(size.to_pixels() as f32);
            }
            StyleClass::MarginTop(size) => {
                self.margin_top = Some(size.to_pixels() as f32);
            }
            StyleClass::MarginLeftAuto => {
                self.margin_left_auto = true;
            }
            StyleClass::MarginRightAuto => {
                self.margin_right_auto = true;
            }
            StyleClass::MarginXAuto => {
                // mx-auto = center horizontally: both flags set
                self.margin_left_auto = true;
                self.margin_right_auto = true;
            }
            StyleClass::Gap(size) => {
                self.gap = Some(size.to_pixels() as f32);
            }

            // ========== Colors (L1) ==========
            StyleClass::BackgroundColor(color) => {
                self.background_color = Some(convert_color(color));
            }
            StyleClass::TextColor(color) => {
                self.text_color = Some(convert_color(color));
            }
            StyleClass::BgGradient(dir) => {
                self.gradient_dir = Some(*dir);
            }
            StyleClass::GradientFrom(color) => {
                self.gradient_from = Some(convert_color(color));
                if self.background_color.is_none() {
                    self.background_color = Some(convert_color(color));
                }
            }
            StyleClass::GradientTo(color) => {
                self.gradient_to = Some(convert_color(color));
            }

            // ========== Sizing (L1) ==========
            StyleClass::Width(size) => {
                self.width = Some(convert_size(size));
            }
            StyleClass::Height(size) => {
                self.height = Some(convert_size(size));
            }
            StyleClass::MaxWidth(px) => {
                self.max_width = Some(*px);
            }
            StyleClass::MaxHeight(px) => {
                self.max_height = Some(*px);
            }

            // ========== Border Radius (L1 + L2) ==========
            StyleClass::Rounded => {
                self.rounded = true;
                self.border_radius = Some(4.0);
            }
            StyleClass::RoundedSm => {
                self.rounded = true;
                self.border_radius = Some(2.0);
            }
            StyleClass::RoundedMd => {
                self.rounded = true;
                self.border_radius = Some(4.0);
            }
            StyleClass::RoundedLg => {
                self.rounded = true;
                self.border_radius = Some(8.0);
            }
            StyleClass::RoundedXl => {
                self.rounded = true;
                self.border_radius = Some(12.0);
            }
            StyleClass::Rounded2Xl => {
                self.rounded = true;
                self.border_radius = Some(16.0);
            }
            StyleClass::Rounded3Xl => {
                self.rounded = true;
                self.border_radius = Some(24.0);
            }
            StyleClass::RoundedFull => {
                self.rounded = true;
                self.border_radius = Some(9999.0); // Effectively full
            }

            // ========== Border (L2) ==========
            StyleClass::Border => {
                self.border = true;
                self.border_width = Some(1.0);
            }
            StyleClass::Border0 => {
                self.border = false;
                self.border_width = Some(0.0);
            }
            StyleClass::BorderWidth(width) => {
                self.border = true;
                self.border_width = Some(*width);
            }
            StyleClass::BorderColor(color) => {
                self.border_color = Some(convert_color(color));
            }

            // ========== Typography (L2) ==========
            StyleClass::TextXs => {
                self.font_size = Some(IcedFontSize::Xs);
            }
            StyleClass::TextSm => {
                self.font_size = Some(IcedFontSize::Sm);
            }
            StyleClass::TextBase => {
                self.font_size = Some(IcedFontSize::Base);
            }
            StyleClass::TextLg => {
                self.font_size = Some(IcedFontSize::Lg);
            }
            StyleClass::TextXl => {
                self.font_size = Some(IcedFontSize::Xl);
            }
            StyleClass::Text2Xl => {
                self.font_size = Some(IcedFontSize::Xxl);
            }
            StyleClass::Text3Xl => {
                self.font_size = Some(IcedFontSize::X3xl);
            }
            StyleClass::Text4Xl => {
                self.font_size = Some(IcedFontSize::X4xl);
            }
            StyleClass::FontBold => {
                self.font_weight = Some(IcedFontWeight::Bold);
            }
            StyleClass::FontMedium => {
                self.font_weight = Some(IcedFontWeight::Medium);
            }
            StyleClass::FontNormal => {
                self.font_weight = Some(IcedFontWeight::Normal);
            }
            StyleClass::TextCenter => {
                self.text_align = Some(IcedTextAlign::Center);
            }
            StyleClass::TextLeft => {
                self.text_align = Some(IcedTextAlign::Left);
            }
            StyleClass::TextRight => {
                self.text_align = Some(IcedTextAlign::Right);
            }

            // ========== Effects (L3) ==========
            StyleClass::Shadow => {
                self.shadow = true;
                self.shadow_size = Some(IcedShadowSize::Md);
            }
            StyleClass::ShadowSm => {
                self.shadow = true;
                self.shadow_size = Some(IcedShadowSize::Sm);
            }
            StyleClass::ShadowMd => {
                self.shadow = true;
                self.shadow_size = Some(IcedShadowSize::Md);
            }
            StyleClass::ShadowLg => {
                self.shadow = true;
                self.shadow_size = Some(IcedShadowSize::Lg);
            }
            StyleClass::ShadowXl => {
                self.shadow = true;
                self.shadow_size = Some(IcedShadowSize::Xl);
            }
            StyleClass::Shadow2Xl => {
                self.shadow = true;
                self.shadow_size = Some(IcedShadowSize::Xxl);
            }
            StyleClass::ShadowNone => {
                self.shadow = false;
                self.shadow_size = Some(IcedShadowSize::None);
            }
            StyleClass::Opacity(value) => {
                self.opacity = Some(*value as f32 / 100.0);
            }

            // ========== Position (L3) ==========
            StyleClass::Relative => {
                self.position = Some(IcedPosition::Relative);
            }
            StyleClass::Absolute => {
                // Iced doesn't support absolute positioning - store but will be ignored
                self.position = Some(IcedPosition::Absolute);
            }
            StyleClass::ZIndex(z) => {
                // Iced doesn't support z-index - store but will be ignored
                self.z_index = Some(*z);
            }

            // ========== Overflow (L3) ==========
            StyleClass::OverflowAuto => {
                self.overflow_x = Some(IcedOverflow::Auto);
                self.overflow_y = Some(IcedOverflow::Auto);
            }
            StyleClass::OverflowHidden => {
                self.overflow_x = Some(IcedOverflow::Hidden);
                self.overflow_y = Some(IcedOverflow::Hidden);
            }
            StyleClass::OverflowVisible => {
                self.overflow_x = Some(IcedOverflow::Visible);
                self.overflow_y = Some(IcedOverflow::Visible);
            }
            StyleClass::OverflowScroll => {
                self.overflow_x = Some(IcedOverflow::Scroll);
                self.overflow_y = Some(IcedOverflow::Scroll);
            }
            StyleClass::OverflowXAuto => {
                self.overflow_x = Some(IcedOverflow::Auto);
            }
            StyleClass::OverflowYAuto => {
                self.overflow_y = Some(IcedOverflow::Auto);
            }

            // ========== Grid (L3) ==========
            StyleClass::Grid => {
                // Iced doesn't support grid - store but will be ignored
                self.grid = true;
            }
            StyleClass::GridCols(cols) => {
                // Iced doesn't support grid - store but will be ignored
                self.grid_cols = Some(*cols);
            }
            StyleClass::GridRows(rows) => {
                // Iced doesn't support grid - store but will be ignored
                self.grid_rows = Some(*rows);
            }
            StyleClass::ColSpan(span) => {
                // Iced doesn't support grid - store but will be ignored
                self.col_span = Some(*span);
            }
            StyleClass::RowSpan(span) => {
                // Iced doesn't support grid - store but will be ignored
                self.row_span = Some(*span);
            }
            StyleClass::ColStart(start) => {
                // Iced doesn't support grid - store but will be ignored
                self.col_start = Some(*start);
            }
            StyleClass::RowStart(start) => {
                // Iced doesn't support grid - store but will be ignored
                self.row_start = Some(*start);
            }

            // ========== Layout styles ==========
            StyleClass::Flex | StyleClass::FlexRow | StyleClass::FlexCol => {
                // Flex is implicit in Iced's Column/Row — no extra action needed
            }
            StyleClass::Flex1 => {
                // flex-1: expand to fill available space along the main axis.
                // In CSS Tailwind, flex-1 means flex-grow:1 flex-shrink:1 flex-basis:0%.
                // IcedStyle doesn't know the parent's flex direction, so we only set
                // width=Full (the most common case: items in a Row). Setting height=Full
                // here causes unintended vertical stretching (e.g. buttons in a Row).
                // For vertical flex-1, users should add h-full explicitly.
                if self.width.is_none() {
                    self.width = Some(IcedSize::Full);
                }
            }
            StyleClass::ItemsCenter => {
                self.align_items = Some(IcedAlign::Center);
            }
            StyleClass::ItemsStart => {
                self.align_items = Some(IcedAlign::Start);
            }
            StyleClass::ItemsEnd => {
                self.align_items = Some(IcedAlign::End);
            }
            StyleClass::JustifyCenter => {
                self.justify_content = Some(IcedJustify::Center);
            }
            StyleClass::JustifyBetween => {
                self.justify_content = Some(IcedJustify::Between);
            }
            StyleClass::JustifyStart => {
                self.justify_content = Some(IcedJustify::Start);
            }
            StyleClass::JustifyEnd => {
                self.justify_content = Some(IcedJustify::End);
            }

            // ========== Extended Sizing ==========
            StyleClass::MinHeight(px) => {
                self.min_height = Some(*px);
            }
            StyleClass::MinWidth(px) => {
                self.min_width = Some(*px);
            }

            // ========== Extended Typography ==========
            StyleClass::TextArbitrary(px) => {
                self.font_size_arbitrary = Some(*px);
            }
            StyleClass::FontLight => {
                self.font_weight = Some(IcedFontWeight::Light);
            }
            StyleClass::FontExtraLight => {
                self.font_weight = Some(IcedFontWeight::ExtraLight);
            }
            StyleClass::FontSemiBold => {
                self.font_weight = Some(IcedFontWeight::SemiBold);
            }
            StyleClass::LineHeight(lh) => {
                self.line_height = Some(*lh);
            }
            StyleClass::LineHeightNone => {
                self.line_height = Some(1.0);
            }

            // ========== Text Control ==========
            StyleClass::WhitespaceNowrap | StyleClass::BreakWords => {
                // Iced text doesn't directly support these, but no error
            }

            // ========== Interaction ==========
            StyleClass::CursorPointer => {
                // Iced doesn't have explicit cursor styling per element
            }

            // ========== Outline/Border ==========
            StyleClass::OutlineNone | StyleClass::BorderNone => {
                self.border = false;
                self.border_width = Some(0.0);
            }

            // ========== Shadow Extended ==========
            StyleClass::ShadowArbitrary(s) => {
                self.shadow = true;
                self.shadow_arbitrary = Some(s.clone());
                if self.shadow_size.is_none() {
                    self.shadow_size = Some(IcedShadowSize::Md);
                }
            }

            // ========== Flex Extended ==========
            StyleClass::Shrink0 => {
                // Iced doesn't support flex-shrink directly
            }

            // ========== List Style ==========
            StyleClass::ListNone => {
                // Iced doesn't have list style concept
            }

            // ========== Font Smoothing ==========
            StyleClass::Antialiased => {
                // Iced handles font smoothing natively
            }

            // ========== Visibility ==========
            StyleClass::Hidden => {
                self.hidden = true;
            }

            // ========== Transition ==========
            StyleClass::TransitionColors | StyleClass::TransitionDuration(_) => {
                // Iced doesn't support CSS transitions
            }

            // ========== Transform ==========
            StyleClass::Rotate(deg) => {
                self.rotate = Some(*deg);
            }

            // ========== Position Offsets ==========
            StyleClass::TopOffset(px) => {
                self.top_offset = Some(*px);
            }
            StyleClass::BottomOffset(px) => {
                self.bottom_offset = Some(*px);
            }
            StyleClass::RightOffset(px) => {
                self.right_offset = Some(*px);
            }
            StyleClass::LeftOffset(px) => {
                self.left_offset = Some(*px);
            }

            // ========== Accent Color ==========
            StyleClass::AccentColor(_) => {
                // Iced checkbox accent color not directly controllable
            }
        }
    }

    // Note: Iced 0.14+ uses theme-based styling
    // Style conversion methods are deprecated
}

/// Convert a SizeValue to IcedSize
fn convert_size(size: &SizeValue) -> IcedSize {
    match size {
        SizeValue::Full => IcedSize::Full,
        SizeValue::Half => IcedSize::FillPortion(1),
        SizeValue::Third => IcedSize::FillPortion(1),
        SizeValue::TwoThirds => IcedSize::FillPortion(2),
        SizeValue::Quarter => IcedSize::FillPortion(1),
        SizeValue::ThreeQuarters => IcedSize::FillPortion(3),
        SizeValue::Auto => IcedSize::Full,
        SizeValue::Fixed(_) => IcedSize::Fixed(size.to_pixels() as f32),
        SizeValue::Pixels(px) => IcedSize::Fixed(*px),
    }
}

/// Convert a Color to iced::Color
fn convert_color(color: &Color) -> iced::Color {
    let (r, g, b) = color.to_rgb8();
    iced::Color::from_rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_style() {
        let style = Style::parse("p-4 bg-white").unwrap();
        let iced_style = IcedStyle::from_style(&style);

        assert_eq!(iced_style.padding, Some(16.0));
    }

    #[test]
    fn test_convert_color() {
        let white = convert_color(&Color::White);
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
    }
}
