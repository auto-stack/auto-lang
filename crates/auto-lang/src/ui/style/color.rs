// Color representation for the unified styling system
//
// Supports semantic colors, Tailwind palette colors, and custom RGB/RGBA values

/// Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    // Semantic colors (theme-based)
    Primary,
    Secondary,
    Background,
    Surface,
    Error,
    Warning,
    Success,
    Info,

    // Text colors
    OnPrimary,
    OnSecondary,
    OnBackground,
    OnSurface,

    // Tailwind palette colors (basic set for L1)
    Slate(u16),    // slate-50 to slate-900
    Gray(u16),     // gray-50 to gray-900
    Zinc(u16),     // zinc-50 to zinc-900
    Neutral(u16),  // neutral-50 to neutral-900
    Red(u16),      // red-50 to red-900
    Blue(u16),     // blue-50 to blue-900
    Green(u16),    // green-50 to green-900
    Yellow(u16),   // yellow-50 to yellow-900
    White,
    Black,

    // Custom colors
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Hex(u32), // 0xRRGGBB or 0xRRGGBBAA
}

impl Color {
    /// Create a color from a hex string (e.g., "#ffffff" or "#ffffffff")
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 && hex.len() != 8 {
            return Err(format!("Invalid hex color length: {}", hex.len()));
        }

        let value = u32::from_str_radix(hex, 16)
            .map_err(|e| format!("Failed to parse hex color: {}", e))?;

        Ok(Self::Hex(value))
    }

    /// Parse a color from a Tailwind color name (e.g., "red-500", "blue", "white")
    pub fn from_tailwind(name: &str) -> Result<Self, String> {
        match name {
            "white" => Ok(Color::White),
            "black" => Ok(Color::Black),
            _ => {
                // Try to parse "color-shade" format
                if let Some(pos) = name.find('-') {
                    let color_name = &name[..pos];
                    let shade_str = &name[pos + 1..];
                    let shade: u16 = shade_str.parse()
                        .map_err(|_| format!("Invalid shade value: {}", shade_str))?;

                    match color_name {
                        "slate" => Ok(Color::Slate(shade)),
                        "gray" => Ok(Color::Gray(shade)),
                        "zinc" => Ok(Color::Zinc(shade)),
                        "neutral" => Ok(Color::Neutral(shade)),
                        "red" => Ok(Color::Red(shade)),
                        "blue" => Ok(Color::Blue(shade)),
                        "green" => Ok(Color::Green(shade)),
                        "yellow" => Ok(Color::Yellow(shade)),
                        _ => Err(format!("Unknown color name: {}", color_name)),
                    }
                } else {
                    Err(format!("Invalid color format: {}", name))
                }
            }
        }
    }

    /// Convert to normalized RGB (0.0-1.0)
    pub fn to_rgb_normalized(&self) -> (f32, f32, f32) {
        match self {
            Color::Rgb { r, g, b } => (
                *r as f32 / 255.0,
                *g as f32 / 255.0,
                *b as f32 / 255.0,
            ),
            Color::Rgba { r, g, b, .. } => (
                *r as f32 / 255.0,
                *g as f32 / 255.0,
                *b as f32 / 255.0,
            ),
            Color::Hex(value) => {
                let r = ((value >> 16) & 0xFF) as f32 / 255.0;
                let g = ((value >> 8) & 0xFF) as f32 / 255.0;
                let b = (value & 0xFF) as f32 / 255.0;
                (r, g, b)
            }
            _ => {
                // For semantic colors and Tailwind colors, compute simplified values
                // In a real implementation, these would look up values from a theme
                match self {
                    Color::Slate(s) | Color::Gray(s) | Color::Zinc(s) | Color::Neutral(s) => {
                        let v = 1.0 - (*s as f32 / 900.0);
                        (v, v, v)
                    }
                    Color::Red(s) => {
                        let v = 1.0 - (*s as f32 / 900.0);
                        (v, 0.0, 0.0)
                    }
                    Color::Blue(s) => {
                        let v = 1.0 - (*s as f32 / 900.0);
                        (0.0, 0.0, v)
                    }
                    Color::Green(s) => {
                        let v = 1.0 - (*s as f32 / 900.0);
                        (0.0, v, 0.0)
                    }
                    Color::Yellow(s) => {
                        let v = 1.0 - (*s as f32 / 900.0);
                        (v, v, 0.0)
                    }
                    Color::White => (1.0, 1.0, 1.0),
                    Color::Black => (0.0, 0.0, 0.0),
                    _ => (0.5, 0.5, 0.5),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_hex() {
        let color = Color::from_hex("#ffffff").unwrap();
        assert_eq!(color, Color::Hex(0xffffff));

        let color = Color::from_hex("#000000").unwrap();
        assert_eq!(color, Color::Hex(0x000000));
    }

    #[test]
    fn test_from_tailwind() {
        let color = Color::from_tailwind("white").unwrap();
        assert_eq!(color, Color::White);

        let color = Color::from_tailwind("slate-500").unwrap();
        assert_eq!(color, Color::Slate(500));
    }

    #[test]
    fn test_to_rgb_normalized() {
        let color = Color::Rgb { r: 255, g: 0, b: 0 };
        let (r, g, b) = color.to_rgb_normalized();
        assert_eq!(r, 1.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
    }
}
