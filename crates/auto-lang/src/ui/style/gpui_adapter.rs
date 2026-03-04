// GPUI Adapter - Convert StyleClass IR to GPUI builder methods
//
// This adapter translates the unified StyleClass IR into GPUI-specific
// builder method calls for styling components.

use super::{Style, StyleClass, SizeValue, Color};
use gpui_lib as gpui;

/// Apply a Style to a GPUI component builder
///
/// This is a trait that can be implemented for different GPUI builder types.
/// For the MVP prototype, we'll provide helper functions that return
/// style parameters that can be applied to GPUI builders.

pub struct GpuiStyle {
    // For GPUI, styles are typically applied via builder methods
    // This struct collects the style parameters

    // Spacing (L1 + L2)
    pub padding: Option<GpuiPadding>,
    pub padding_x: Option<f32>,
    pub padding_y: Option<f32>,
    pub margin: Option<f32>,
    pub margin_x: Option<f32>,
    pub margin_y: Option<f32>,
    pub gap: Option<f32>,

    // Layout (L1 + L2)
    pub flex: Option<bool>,
    pub flex1: bool,  // flex-1
    pub flex_direction: Option<GpuiFlexDirection>,
    pub items_align: Option<GpuiAlignment>,
    pub justify_align: Option<GpuiAlignment>,

    // Sizing (L1)
    pub width: Option<GpuiSize>,
    pub height: Option<GpuiSize>,

    // Colors (L1)
    pub background_color: Option<gpui::Rgba>,
    pub text_color: Option<gpui::Rgba>,

    // Border Radius (L1 + L2)
    pub rounded: bool,
    pub rounded_size: Option<GpuiRoundedSize>,

    // Border (L2)
    pub border: bool,
    pub border_width: Option<f32>,
    pub border_color: Option<gpui::Rgba>,

    // Typography (L2)
    pub font_size: Option<GpuiFontSize>,
    pub font_weight: Option<GpuiFontWeight>,
    pub text_align: Option<GpuiTextAlign>,

    // Effects (L3)
    pub shadow: bool,
    pub shadow_size: Option<GpuiShadowSize>,
    pub opacity: Option<f32>,

    // Position (L3)
    pub position: Option<GpuiPosition>,
    pub z_index: Option<i16>,

    // Overflow (L3)
    pub overflow_x: Option<GpuiOverflow>,
    pub overflow_y: Option<GpuiOverflow>,

    // Grid (L3)
    pub grid: bool,
    pub grid_cols: Option<u8>,
    pub grid_rows: Option<u8>,
    pub col_span: Option<u8>,
    pub row_span: Option<u8>,
    pub col_start: Option<u8>,
    pub row_start: Option<u8>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiShadowSize {
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    None,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiPosition {
    Relative,
    Absolute,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiOverflow {
    Auto,
    Hidden,
    Visible,
    Scroll,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiPadding {
    Uniform(f32), // p-4 = 16px
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiRoundedSize {
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    Full,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiFontSize {
    Xs,   // 12px
    Sm,   // 14px
    Base, // 16px
    Lg,   // 18px
    Xl,   // 20px
    Xxl,  // 24px
    X3xl, // 30px
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiFontWeight {
    Normal,
    Medium,
    Bold,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiFlexDirection {
    Row,
    Col,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiAlignment {
    Center,
    Between,
    Start,
    End,
}

#[derive(Clone, Copy, PartialEq)]
pub enum GpuiSize {
    Full,
    Fixed(f32),
}

impl GpuiStyle {
    /// Convert a Style to GpuiStyle
    pub fn from_style(style: &Style) -> Self {
        let mut gpui_style = GpuiStyle {
            padding: None,
            padding_x: None,
            padding_y: None,
            margin: None,
            margin_x: None,
            margin_y: None,
            gap: None,
            flex: None,
            flex1: false,
            flex_direction: None,
            items_align: None,
            justify_align: None,
            width: None,
            height: None,
            background_color: None,
            text_color: None,
            rounded: false,
            rounded_size: None,
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
            z_index: None,
            overflow_x: None,
            overflow_y: None,
            grid: false,
            grid_cols: None,
            grid_rows: None,
            col_span: None,
            row_span: None,
            col_start: None,
            row_start: None,
        };

        for class in &style.classes {
            gpui_style.apply_class(class);
        }

        gpui_style
    }

    /// Apply a single StyleClass to this GpuiStyle
    fn apply_class(&mut self, class: &StyleClass) {
        match class {
            // ========== Spacing (L1 + L2) ==========
            StyleClass::Padding(size) => {
                self.padding = Some(GpuiPadding::Uniform(size.to_pixels() as f32));
            }
            StyleClass::PaddingX(size) => {
                self.padding_x = Some(size.to_pixels() as f32);
            }
            StyleClass::PaddingY(size) => {
                self.padding_y = Some(size.to_pixels() as f32);
            }
            StyleClass::Margin(size) => {
                self.margin = Some(size.to_pixels() as f32);
            }
            StyleClass::MarginX(size) => {
                self.margin_x = Some(size.to_pixels() as f32);
            }
            StyleClass::MarginY(size) => {
                self.margin_y = Some(size.to_pixels() as f32);
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

            // ========== Layout (L1 + L2) ==========
            StyleClass::Flex => {
                self.flex = Some(true);
            }
            StyleClass::Flex1 => {
                self.flex = Some(true);
                self.flex1 = true;
            }
            StyleClass::FlexRow => {
                self.flex = Some(true);
                self.flex_direction = Some(GpuiFlexDirection::Row);
            }
            StyleClass::FlexCol => {
                self.flex = Some(true);
                self.flex_direction = Some(GpuiFlexDirection::Col);
            }
            StyleClass::ItemsCenter => {
                self.items_align = Some(GpuiAlignment::Center);
            }
            StyleClass::ItemsStart => {
                self.items_align = Some(GpuiAlignment::Start);
            }
            StyleClass::ItemsEnd => {
                self.items_align = Some(GpuiAlignment::End);
            }
            StyleClass::JustifyCenter => {
                self.justify_align = Some(GpuiAlignment::Center);
            }
            StyleClass::JustifyBetween => {
                self.justify_align = Some(GpuiAlignment::Between);
            }
            StyleClass::JustifyStart => {
                self.justify_align = Some(GpuiAlignment::Start);
            }
            StyleClass::JustifyEnd => {
                self.justify_align = Some(GpuiAlignment::End);
            }

            // ========== Sizing (L1) ==========
            StyleClass::Width(size) => {
                self.width = Some(convert_size(&size));
            }
            StyleClass::Height(size) => {
                self.height = Some(convert_size(&size));
            }

            // ========== Border Radius (L1 + L2) ==========
            StyleClass::Rounded => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Md);
            }
            StyleClass::RoundedSm => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Sm);
            }
            StyleClass::RoundedMd => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Md);
            }
            StyleClass::RoundedLg => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Lg);
            }
            StyleClass::RoundedXl => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Xl);
            }
            StyleClass::Rounded2Xl => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Xxl);
            }
            StyleClass::Rounded3Xl => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Xxl);
            }
            StyleClass::RoundedFull => {
                self.rounded = true;
                self.rounded_size = Some(GpuiRoundedSize::Full);
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
            StyleClass::BorderColor(color) => {
                self.border = true;
                self.border_color = Some(convert_color(color));
            }

            // ========== Typography (L2) ==========
            StyleClass::TextXs => {
                self.font_size = Some(GpuiFontSize::Xs);
            }
            StyleClass::TextSm => {
                self.font_size = Some(GpuiFontSize::Sm);
            }
            StyleClass::TextBase => {
                self.font_size = Some(GpuiFontSize::Base);
            }
            StyleClass::TextLg => {
                self.font_size = Some(GpuiFontSize::Lg);
            }
            StyleClass::TextXl => {
                self.font_size = Some(GpuiFontSize::Xl);
            }
            StyleClass::Text2Xl => {
                self.font_size = Some(GpuiFontSize::Xxl);
            }
            StyleClass::Text3Xl => {
                self.font_size = Some(GpuiFontSize::X3xl);
            }
            StyleClass::FontBold => {
                self.font_weight = Some(GpuiFontWeight::Bold);
            }
            StyleClass::FontMedium => {
                self.font_weight = Some(GpuiFontWeight::Medium);
            }
            StyleClass::FontNormal => {
                self.font_weight = Some(GpuiFontWeight::Normal);
            }
            StyleClass::TextCenter => {
                self.text_align = Some(GpuiTextAlign::Center);
            }
            StyleClass::TextLeft => {
                self.text_align = Some(GpuiTextAlign::Left);
            }
            StyleClass::TextRight => {
                self.text_align = Some(GpuiTextAlign::Right);
            }

            // ========== Effects (L3) ==========
            StyleClass::Shadow => {
                self.shadow = true;
                self.shadow_size = Some(GpuiShadowSize::Md);
            }
            StyleClass::ShadowSm => {
                self.shadow = true;
                self.shadow_size = Some(GpuiShadowSize::Sm);
            }
            StyleClass::ShadowMd => {
                self.shadow = true;
                self.shadow_size = Some(GpuiShadowSize::Md);
            }
            StyleClass::ShadowLg => {
                self.shadow = true;
                self.shadow_size = Some(GpuiShadowSize::Lg);
            }
            StyleClass::ShadowXl => {
                self.shadow = true;
                self.shadow_size = Some(GpuiShadowSize::Xl);
            }
            StyleClass::Shadow2Xl => {
                self.shadow = true;
                self.shadow_size = Some(GpuiShadowSize::Xxl);
            }
            StyleClass::ShadowNone => {
                self.shadow = false;
                self.shadow_size = Some(GpuiShadowSize::None);
            }
            StyleClass::Opacity(value) => {
                self.opacity = Some(*value as f32 / 100.0);
            }

            // ========== Position (L3) ==========
            StyleClass::Relative => {
                self.position = Some(GpuiPosition::Relative);
            }
            StyleClass::Absolute => {
                self.position = Some(GpuiPosition::Absolute);
            }
            StyleClass::ZIndex(z) => {
                self.z_index = Some(*z);
            }

            // ========== Overflow (L3) ==========
            StyleClass::OverflowAuto => {
                self.overflow_x = Some(GpuiOverflow::Auto);
                self.overflow_y = Some(GpuiOverflow::Auto);
            }
            StyleClass::OverflowHidden => {
                self.overflow_x = Some(GpuiOverflow::Hidden);
                self.overflow_y = Some(GpuiOverflow::Hidden);
            }
            StyleClass::OverflowVisible => {
                self.overflow_x = Some(GpuiOverflow::Visible);
                self.overflow_y = Some(GpuiOverflow::Visible);
            }
            StyleClass::OverflowScroll => {
                self.overflow_x = Some(GpuiOverflow::Scroll);
                self.overflow_y = Some(GpuiOverflow::Scroll);
            }
            StyleClass::OverflowXAuto => {
                self.overflow_x = Some(GpuiOverflow::Auto);
            }
            StyleClass::OverflowYAuto => {
                self.overflow_y = Some(GpuiOverflow::Auto);
            }

            // ========== Grid (L3) ==========
            StyleClass::Grid => {
                self.grid = true;
            }
            StyleClass::GridCols(cols) => {
                self.grid = true;
                self.grid_cols = Some(*cols);
            }
            StyleClass::GridRows(rows) => {
                self.grid = true;
                self.grid_rows = Some(*rows);
            }
            StyleClass::ColSpan(span) => {
                self.col_span = Some(*span);
            }
            StyleClass::RowSpan(span) => {
                self.row_span = Some(*span);
            }
            StyleClass::ColStart(start) => {
                self.col_start = Some(*start);
            }
            StyleClass::RowStart(start) => {
                self.row_start = Some(*start);
            }
        }
    }
}

/// Convert a SizeValue to GpuiSize
fn convert_size(size: &SizeValue) -> GpuiSize {
    match size {
        SizeValue::Full => GpuiSize::Full,
        SizeValue::Fixed(units) => GpuiSize::Fixed((*units * 4) as f32),
        _ => GpuiSize::Full, // Default to full for other variants
    }
}

/// Convert a Color to gpui::Rgba
fn convert_color(color: &Color) -> gpui::Rgba {
    match color {
        Color::Rgb { r, g, b } => {
            gpui::Rgba { r: *r as f32 / 255.0, g: *g as f32 / 255.0, b: *b as f32 / 255.0, a: 1.0 }
        }
        Color::Rgba { r, g, b, a } => {
            gpui::Rgba { r: *r as f32 / 255.0, g: *g as f32 / 255.0, b: *b as f32 / 255.0, a: *a as f32 / 255.0 }
        }
        Color::White => gpui::Rgba { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
        Color::Black => gpui::Rgba { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
        Color::Slate(shade) | Color::Gray(shade) | Color::Zinc(shade) | Color::Neutral(shade) => {
            // Grayscale colors
            let value = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: value, g: value, b: value, a: 1.0 }
        }
        Color::Red(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: intensity, g: 0.0, b: 0.0, a: 1.0 }
        }
        Color::Blue(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: 0.0, g: 0.0, b: intensity, a: 1.0 }
        }
        Color::Green(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: 0.0, g: intensity, b: 0.0, a: 1.0 }
        }
        Color::Yellow(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            gpui::Rgba { r: intensity, g: intensity, b: 0.0, a: 1.0 }
        }
        _ => gpui::Rgba { r: 0.5, g: 0.5, b: 0.5, a: 1.0 }, // Default gray (semantic colors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_simple_style() {
        let style = Style::parse("p-4 bg-white flex").unwrap();
        let gpui_style = GpuiStyle::from_style(&style);

        assert!(gpui_style.flex.unwrap());
        assert_eq!(gpui_style.padding, Some(GpuiPadding::Uniform(16.0)));
    }

    #[test]
    fn test_convert_color() {
        let white = convert_color(&Color::White);
        assert_eq!(white.r, 1.0);
        assert_eq!(white.g, 1.0);
        assert_eq!(white.b, 1.0);
    }
}
