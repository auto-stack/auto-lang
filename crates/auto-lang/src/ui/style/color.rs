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
    Purple(u16),   // purple-50 to purple-900
    Pink(u16),     // pink-50 to pink-900
    Indigo(u16),   // indigo-50 to indigo-900
    Orange(u16),   // orange-50 to orange-900
    Cyan(u16),     // cyan-50 to cyan-900
    Teal(u16),     // teal-50 to teal-900
    White,
    Black,

    // Custom colors
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Hex(u32), // 0xRRGGBB or 0xRRGGBBAA
}

impl From<&str> for Color {
    fn from(s: &str) -> Self {
        Color::from_tailwind(s).unwrap_or(Color::Hex(0x000000))
    }
}

impl From<String> for Color {
    fn from(s: String) -> Self {
        Color::from(s.as_str())
    }
}

impl Color {
    /// Create a color from a hex string (e.g., "#ffffff" or "#ffffffff")
    pub fn from_hex(hex: &str) -> Result<Self, String> {
        let hex = hex.trim_start_matches('#');

        // Expand 3-digit hex (#RGB → RRGGBB) and 4-digit hex (#RGBA → RRGGBBAA)
        let expanded: String;
        let hex = if hex.len() == 3 || hex.len() == 4 {
            expanded = hex.chars().flat_map(|c| [c, c]).collect();
            &expanded
        } else {
            hex
        };

        if hex.len() != 6 && hex.len() != 8 {
            return Err(format!("Invalid hex color length: {}", hex.len()));
        }

        let value = u32::from_str_radix(hex, 16)
            .map_err(|e| format!("Failed to parse hex color: {}", e))?;

        Ok(Self::Hex(value))
    }

    /// Parse a color from a Tailwind color name (e.g., "red-500", "blue", "white")
    /// or a semantic token name (e.g., "primary", "foreground", "background").
    pub fn from_tailwind(name: &str) -> Result<Self, String> {
        match name {
            "white" => Ok(Color::White),
            "black" => Ok(Color::Black),
            "transparent" => Ok(Color::Rgba { r: 0, g: 0, b: 0, a: 0 }),
            // Plan 370 D-GAP-1: semantic token names (shadcn-style)
            "primary" => Ok(Color::Primary),
            "secondary" => Ok(Color::Secondary),
            // "foreground" / "text" → text color on background
            "foreground" | "text" => Ok(Color::OnBackground),
            // "background" / "card" / "surface" / "popover" → surface colors
            "background" | "card" | "surface" | "popover" => Ok(Color::Background),
            // "muted" → slightly darker surface
            "muted" => Ok(Color::Surface),
            "muted-foreground" => Ok(Color::OnSurface),
            // "accent" → interactive highlight surface
            "accent" => Ok(Color::Secondary),
            "accent-foreground" => Ok(Color::OnSecondary),
            "primary-foreground" => Ok(Color::OnPrimary),
            "secondary-foreground" => Ok(Color::OnSecondary),
            "destructive" | "danger" | "error" => Ok(Color::Error),
            "destructive-foreground" => Ok(Color::OnPrimary), // white text on red
            "success" => Ok(Color::Success),
            "warning" => Ok(Color::Warning),
            "info" => Ok(Color::Info),
            // "border" / "input" / "ring" → subtle surface
            "border" | "input" | "ring" => Ok(Color::Surface),
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
                        "purple" => Ok(Color::Purple(shade)),
                        "pink" => Ok(Color::Pink(shade)),
                        "indigo" => Ok(Color::Indigo(shade)),
                        "orange" => Ok(Color::Orange(shade)),
                        "cyan" => Ok(Color::Cyan(shade)),
                        "teal" => Ok(Color::Teal(shade)),
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
        let (r, g, b) = self.to_rgb8();
        (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    /// Convert to (r, g, b) u8 values using Tailwind CSS palette.
    pub fn to_rgb8(&self) -> (u8, u8, u8) {
        match self {
            Color::Rgb { r, g, b } => (*r, *g, *b),
            Color::Rgba { r, g, b, .. } => (*r, *g, *b),
            Color::Hex(value) => {
                let r = ((value >> 16) & 0xFF) as u8;
                let g = ((value >> 8) & 0xFF) as u8;
                let b = (value & 0xFF) as u8;
                (r, g, b)
            }
            Color::White => (255, 255, 255),
            Color::Black => (0, 0, 0),
            Color::Gray(s) => tailwind_gray(*s),
            Color::Slate(s) => tailwind_slate(*s),
            Color::Zinc(s) => tailwind_zinc(*s),
            Color::Neutral(s) => tailwind_neutral(*s),
            Color::Red(s) => tailwind_red(*s),
            Color::Blue(s) => tailwind_blue(*s),
            Color::Green(s) => tailwind_green(*s),
            Color::Yellow(s) => tailwind_yellow(*s),
            Color::Purple(s) => tailwind_purple(*s),
            Color::Pink(s) => tailwind_pink(*s),
            Color::Indigo(s) => tailwind_indigo(*s),
            Color::Orange(s) => tailwind_orange(*s),
            Color::Cyan(s) => tailwind_cyan(*s),
            Color::Teal(s) => tailwind_teal(*s),
            // Plan 370 D-GAP-1: semantic colors with hardcoded light-mode RGB
            Color::Primary => (99, 102, 241),       // indigo-500
            Color::Secondary => (139, 92, 246),      // violet-500
            Color::Background => (255, 255, 255),    // white
            Color::Surface => (249, 250, 251),       // gray-50
            Color::Error => (239, 68, 68),           // red-500
            Color::Warning => (234, 179, 8),         // yellow-500
            Color::Success => (34, 197, 94),         // green-500
            Color::Info => (59, 130, 246),           // blue-500
            Color::OnPrimary | Color::OnSecondary => (255, 255, 255),   // white
            Color::OnBackground => (17, 24, 39),     // near-black
            Color::OnSurface => (107, 114, 128),     // gray-500
            _ => (128, 128, 128),
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

// ============================================================================
// Tailwind CSS v3 palette — exact RGB values
// ============================================================================

macro_rules! palette {
    ($name:ident, [$($shade:expr => ($r:expr, $g:expr, $b:expr)),+ $(,)?]) => {
        fn $name(shade: u16) -> (u8, u8, u8) {
            match shade {
                $($shade => ($r, $g, $b),)+
                _ => (128, 128, 128),
            }
        }
    };
}

// Tailwind Gray (gray-50 through gray-900)
palette!(tailwind_gray, [
    50  => (249, 250, 251),
    100 => (243, 244, 246),
    200 => (229, 231, 235),
    300 => (209, 213, 219),
    400 => (156, 163, 175),
    500 => (107, 114, 128),
    600 => (75, 85, 99),
    700 => (55, 65, 81),
    800 => (31, 41, 55),
    900 => (17, 24, 39),
]);

// Tailwind Blue (blue-50 through blue-900)
palette!(tailwind_blue, [
    50  => (239, 246, 255),
    100 => (219, 234, 254),
    200 => (191, 219, 254),
    300 => (147, 197, 253),
    400 => (96, 165, 250),
    500 => (59, 130, 246),
    600 => (37, 99, 235),
    700 => (29, 78, 216),
    800 => (30, 64, 175),
    900 => (30, 58, 138),
]);

// Tailwind Red (red-50 through red-900)
palette!(tailwind_red, [
    50  => (254, 242, 242),
    100 => (254, 226, 226),
    200 => (254, 202, 202),
    300 => (252, 165, 165),
    400 => (248, 113, 113),
    500 => (239, 68, 68),
    600 => (220, 38, 38),
    700 => (185, 28, 28),
    800 => (153, 27, 27),
    900 => (127, 29, 29),
]);

// Tailwind Green (green-50 through green-900)
palette!(tailwind_green, [
    50  => (240, 253, 244),
    100 => (220, 252, 231),
    200 => (187, 247, 208),
    300 => (134, 239, 172),
    400 => (74, 222, 128),
    500 => (34, 197, 94),
    600 => (22, 163, 74),
    700 => (21, 128, 61),
    800 => (22, 101, 52),
    900 => (20, 83, 45),
]);

// Tailwind Yellow (yellow-50 through yellow-900)
palette!(tailwind_yellow, [
    50  => (254, 252, 232),
    100 => (254, 249, 195),
    200 => (254, 240, 138),
    300 => (253, 224, 71),
    400 => (250, 204, 21),
    500 => (234, 179, 8),
    600 => (202, 138, 4),
    700 => (161, 98, 7),
    800 => (133, 77, 14),
    900 => (113, 63, 18),
]);

// Tailwind Slate (slate-50 through slate-900)
palette!(tailwind_slate, [
    50  => (248, 250, 252),
    100 => (241, 245, 249),
    200 => (226, 232, 240),
    300 => (203, 213, 225),
    400 => (148, 163, 184),
    500 => (100, 116, 139),
    600 => (71, 85, 105),
    700 => (51, 65, 85),
    800 => (30, 41, 59),
    900 => (15, 23, 42),
]);

// Tailwind Zinc (zinc-50 through zinc-900)
palette!(tailwind_zinc, [
    50  => (250, 250, 250),
    100 => (244, 244, 245),
    200 => (228, 228, 231),
    300 => (212, 212, 216),
    400 => (161, 161, 170),
    500 => (113, 113, 122),
    600 => (82, 82, 91),
    700 => (63, 63, 70),
    800 => (39, 39, 42),
    900 => (24, 24, 27),
]);

// Tailwind Neutral (neutral-50 through neutral-900)
palette!(tailwind_neutral, [
    50  => (250, 250, 250),
    100 => (245, 245, 245),
    200 => (229, 229, 229),
    300 => (212, 212, 212),
    400 => (163, 163, 163),
    500 => (115, 115, 115),
    600 => (82, 82, 82),
    700 => (64, 64, 64),
    800 => (38, 38, 38),
    900 => (23, 23, 23),
]);

// Tailwind Purple (purple-50 through purple-900)
palette!(tailwind_purple, [
    50  => (250, 245, 255),
    100 => (243, 232, 255),
    200 => (233, 213, 255),
    300 => (216, 180, 254),
    400 => (192, 132, 252),
    500 => (168, 85, 247),
    600 => (147, 51, 234),
    700 => (126, 34, 206),
    800 => (107, 33, 168),
    900 => (88, 28, 135),
]);

// Tailwind Pink (pink-50 through pink-900)
palette!(tailwind_pink, [
    50  => (253, 242, 248),
    100 => (252, 231, 243),
    200 => (251, 207, 232),
    300 => (249, 168, 212),
    400 => (244, 114, 182),
    500 => (236, 72, 153),
    600 => (219, 39, 119),
    700 => (190, 24, 93),
    800 => (157, 23, 77),
    900 => (131, 24, 67),
]);

// Tailwind Indigo (indigo-50 through indigo-900)
palette!(tailwind_indigo, [
    50  => (238, 242, 255),
    100 => (224, 231, 255),
    200 => (199, 210, 254),
    300 => (165, 180, 252),
    400 => (129, 140, 248),
    500 => (99, 102, 241),
    600 => (79, 70, 229),
    700 => (67, 56, 202),
    800 => (55, 48, 163),
    900 => (49, 46, 129),
]);

// Tailwind Orange (orange-50 through orange-900)
palette!(tailwind_orange, [
    50  => (255, 247, 237),
    100 => (255, 237, 213),
    200 => (254, 215, 170),
    300 => (253, 186, 116),
    400 => (251, 146, 60),
    500 => (249, 115, 22),
    600 => (234, 88, 12),
    700 => (194, 65, 12),
    800 => (154, 52, 18),
    900 => (124, 45, 18),
]);

// Tailwind Cyan (cyan-50 through cyan-900)
palette!(tailwind_cyan, [
    50  => (236, 254, 255),
    100 => (207, 250, 254),
    200 => (165, 243, 252),
    300 => (103, 232, 249),
    400 => (34, 211, 238),
    500 => (6, 182, 212),
    600 => (8, 145, 178),
    700 => (14, 116, 144),
    800 => (21, 94, 117),
    900 => (22, 78, 99),
]);

// Tailwind Teal (teal-50 through teal-900)
palette!(tailwind_teal, [
    50  => (240, 253, 250),
    100 => (204, 251, 241),
    200 => (153, 246, 228),
    300 => (94, 234, 212),
    400 => (45, 212, 191),
    500 => (20, 184, 166),
    600 => (13, 148, 136),
    700 => (15, 118, 110),
    800 => (17, 94, 89),
    900 => (19, 78, 74),
]);
