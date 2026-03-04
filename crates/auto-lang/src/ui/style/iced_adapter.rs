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
    // NOTE: Iced doesn't support margin - these fields are ignored
    pub margin: Option<f32>,        // Not supported by Iced
    pub margin_x: Option<f32>,       // Not supported by Iced
    pub margin_y: Option<f32>,       // Not supported by Iced
    pub gap: Option<f32>,

    // Colors (L1)
    pub background_color: Option<iced::Color>,
    pub text_color: Option<iced::Color>,

    // Sizing (L1)
    pub width: Option<IcedSize>,
    pub height: Option<IcedSize>,

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

    // Grid (L3)
    // NOTE: Iced doesn't support grid layout - these fields are ignored
    pub grid: bool,                 // Not supported by Iced
    pub grid_cols: Option<u8>,      // Not supported by Iced
    pub grid_rows: Option<u8>,      // Not supported by Iced
    pub col_span: Option<u8>,       // Not supported by Iced
    pub row_span: Option<u8>,       // Not supported by Iced
    pub col_start: Option<u8>,      // Not supported by Iced
    pub row_start: Option<u8>,      // Not supported by Iced
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
}

#[derive(Clone, Copy, PartialEq)]
pub enum IcedFontWeight {
    Normal,
    Medium,
    Bold,
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
            margin: None,      // Not supported by Iced
            margin_x: None,    // Not supported by Iced
            margin_y: None,    // Not supported by Iced
            gap: None,
            background_color: None,
            text_color: None,
            width: None,
            height: None,
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
            grid: false,        // Not supported by Iced
            grid_cols: None,    // Not supported by Iced
            grid_rows: None,    // Not supported by Iced
            col_span: None,     // Not supported by Iced
            row_span: None,     // Not supported by Iced
            col_start: None,    // Not supported by Iced
            row_start: None,    // Not supported by Iced
        };

        for class in &style.classes {
            iced_style.apply_class(class);
        }

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

            // ========== Sizing (L1) ==========
            StyleClass::Width(size) => {
                self.width = Some(convert_size(size));
            }
            StyleClass::Height(size) => {
                self.height = Some(convert_size(size));
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
            StyleClass::BorderColor(color) => {
                self.border = true;
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
            // Layout-related styles (flex, items-center, etc.) are handled differently in Iced
            // They're applied through layout methods rather than style objects
            _ => {
                // Ignore layout classes - they're handled separately in Iced
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
        SizeValue::Fixed(units) => IcedSize::Fixed(*units as f32),
        _ => IcedSize::Full, // Default to full for other variants
    }
}

/// Convert a Color to iced::Color
fn convert_color(color: &Color) -> iced::Color {
    match color {
        Color::Rgb { r, g, b } => {
            iced::Color::from_rgb(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0)
        }
        Color::Rgba { r, g, b, a } => {
            iced::Color::from_rgba(*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, *a as f32 / 255.0)
        }
        Color::White => iced::Color::WHITE,
        Color::Black => iced::Color::BLACK,
        Color::Slate(shade) | Color::Gray(shade) | Color::Zinc(shade) | Color::Neutral(shade) => {
            // Grayscale colors
            let value = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(value, value, value)
        }
        Color::Red(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(intensity, 0.0, 0.0)
        }
        Color::Blue(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(0.0, 0.0, intensity)
        }
        Color::Green(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(0.0, intensity, 0.0)
        }
        Color::Yellow(shade) => {
            let intensity = 1.0 - (*shade as f32 / 900.0);
            iced::Color::from_rgb(intensity, intensity, 0.0)
        }
        _ => iced::Color::from_rgb(0.5, 0.5, 0.5), // Default gray (semantic colors)
    }
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
