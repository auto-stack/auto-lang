//! Tailwind to Compose Modifier DSL Converter
//!
//! Converts Tailwind CSS classes to Jetpack Compose Modifier chains.
//!
//! ## Unit Conversion
//! Tailwind unit → Dp: value * 4
//! - gap-2 → 8.dp
//! - px-4 → padding(horizontal = 16.dp)

/// Tailwind class to Compose Modifier converter
pub struct ModifierDsl {
    /// Tailwind unit to Dp multiplier (default: 4)
    unit_multiplier: f32,
}

/// Result of converting Tailwind classes
pub struct ModifierResult {
    /// Modifier chain components
    pub modifiers: Vec<String>,
    /// Arrangement for gap (if any)
    pub arrangement: Option<String>,
}

impl ModifierDsl {
    /// Create a new converter
    pub fn new() -> Self {
        Self {
            unit_multiplier: 4.0,
        }
    }

    /// Convert Tailwind value to Dp string
    fn to_dp(&self, value: u32) -> String {
        let dp = value as f32 * self.unit_multiplier;
        format!("{}.dp", dp as u32)
    }

    /// Convert a single Tailwind class to Modifier code
    pub fn convert_single(&self, class: &str) -> Option<String> {
        let class = class.trim();

        // Padding: p-{n}, px-{n}, py-{n}, pt-{n}, pb-{n}, pl-{n}, pr-{n}
        if let Some(rest) = class.strip_prefix("px-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(horizontal = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("py-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(vertical = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("pt-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(top = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("pb-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(bottom = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("pl-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(start = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("pr-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(end = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("p-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding({})", self.to_dp(n)));
            }
        }

        // Margin: m-{n}, mx-{n}, my-{n} (maps to padding in Compose for outer spacing)
        if let Some(rest) = class.strip_prefix("mx-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(horizontal = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("my-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding(vertical = {})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("m-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("padding({})", self.to_dp(n)));
            }
        }

        // Size: w-full, h-full
        if class == "w-full" {
            return Some("fillMaxWidth()".to_string());
        }
        if class == "h-full" {
            return Some("fillMaxHeight()".to_string());
        }
        if class == "size-full" {
            return Some("fillMaxSize()".to_string());
        }

        // Rounded: rounded, rounded-{size}
        if class == "rounded" {
            return Some("rounded(4.dp)".to_string());
        }
        if class == "rounded-sm" {
            return Some("rounded(2.dp)".to_string());
        }
        if class == "rounded-md" {
            return Some("rounded(6.dp)".to_string());
        }
        if class == "rounded-lg" {
            return Some("rounded(8.dp)".to_string());
        }
        if class == "rounded-xl" {
            return Some("rounded(12.dp)".to_string());
        }
        if class == "rounded-2xl" {
            return Some("rounded(16.dp)".to_string());
        }
        if class == "rounded-full" {
            return Some("clip(RoundedCornerShape(percent = 50))".to_string());
        }
        if let Some(rest) = class.strip_prefix("rounded-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("rounded({})", self.to_dp(n)));
            }
        }

        // Background colors
        if let Some(color) = class.strip_prefix("bg-") {
            if let Some(hex) = self.tailwind_color_to_hex(color) {
                return Some(format!("background(Color({}))", hex));
            }
        }

        // Text colors
        if let Some(color) = class.strip_prefix("text-") {
            if let Some(hex) = self.tailwind_color_to_hex(color) {
                return Some(format!("color = Color({})", hex));
            }
        }

        // Shadow
        if class == "shadow" || class == "shadow-md" {
            return Some("shadow(4.dp)".to_string());
        }
        if class == "shadow-sm" {
            return Some("shadow(2.dp)".to_string());
        }
        if class == "shadow-lg" {
            return Some("shadow(8.dp)".to_string());
        }
        if class == "shadow-xl" {
            return Some("shadow(16.dp)".to_string());
        }

        // Border
        if class == "border" {
            return Some("border(1.dp)".to_string());
        }
        if let Some(rest) = class.strip_prefix("border-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("border({})", self.to_dp(n)));
            }
        }

        // Width/Height specific values
        if let Some(rest) = class.strip_prefix("w-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("width({})", self.to_dp(n)));
            }
        }
        if let Some(rest) = class.strip_prefix("h-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("height({})", self.to_dp(n)));
            }
        }

        // Min/Max width
        if let Some(rest) = class.strip_prefix("min-w-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("width(IntrinsicSize.Min).width({})", self.to_dp(n)));
            }
            if rest == "full" {
                return Some("width(IntrinsicSize.Min).fillMaxWidth()".to_string());
            }
        }
        if let Some(rest) = class.strip_prefix("max-w-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("width(IntrinsicSize.Max).width({})", self.to_dp(n)));
            }
            if rest == "full" {
                return Some("width(IntrinsicSize.Max).fillMaxWidth()".to_string());
            }
        }

        // Min/Max height
        if let Some(rest) = class.strip_prefix("min-h-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("height(IntrinsicSize.Min).height({})", self.to_dp(n)));
            }
            if rest == "full" {
                return Some("height(IntrinsicSize.Min).fillMaxHeight()".to_string());
            }
        }
        if let Some(rest) = class.strip_prefix("max-h-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("height(IntrinsicSize.Max).height({})", self.to_dp(n)));
            }
            if rest == "full" {
                return Some("height(IntrinsicSize.Max).fillMaxHeight()".to_string());
            }
        }

        // Flexbox: direction
        if class == "flex" || class == "flex-row" {
            // In Compose, this is handled by Row/Column choice, not modifier
            return None;
        }
        if class == "flex-col" {
            // In Compose, this is handled by Row/Column choice, not modifier
            return None;
        }
        if class == "flex-wrap" {
            // Not directly supported in Compose modifiers
            return None;
        }
        if class == "flex-1" {
            return Some("weight(1f)".to_string());
        }

        // Opacity (0-100)
        if let Some(rest) = class.strip_prefix("opacity-") {
            if let Ok(n) = rest.parse::<u32>() {
                let alpha = (n as f32 / 100.0).min(1.0);
                return Some(format!("alpha({:.2}f)", alpha));
            }
        }

        // Font size
        if class == "text-xs" {
            return Some("fontSize(12.sp)".to_string());
        }
        if class == "text-sm" {
            return Some("fontSize(14.sp)".to_string());
        }
        if class == "text-base" {
            return Some("fontSize(16.sp)".to_string());
        }
        if class == "text-lg" {
            return Some("fontSize(18.sp)".to_string());
        }
        if class == "text-xl" {
            return Some("fontSize(20.sp)".to_string());
        }
        if class == "text-2xl" {
            return Some("fontSize(24.sp)".to_string());
        }
        if class == "text-3xl" {
            return Some("fontSize(30.sp)".to_string());
        }
        if class == "text-4xl" {
            return Some("fontSize(36.sp)".to_string());
        }
        // Custom font size: text-{n}
        if let Some(rest) = class.strip_prefix("text-size-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("fontSize({}.sp)", n));
            }
        }

        // Font weight
        if class == "font-thin" {
            return Some("fontWeight(FontWeight.Thin)".to_string());
        }
        if class == "font-light" {
            return Some("fontWeight(FontWeight.Light)".to_string());
        }
        if class == "font-normal" {
            return Some("fontWeight(FontWeight.Normal)".to_string());
        }
        if class == "font-medium" {
            return Some("fontWeight(FontWeight.Medium)".to_string());
        }
        if class == "font-semibold" {
            return Some("fontWeight(FontWeight.SemiBold)".to_string());
        }
        if class == "font-bold" {
            return Some("fontWeight(FontWeight.Bold)".to_string());
        }
        if class == "font-extrabold" {
            return Some("fontWeight(FontWeight.ExtraBold)".to_string());
        }

        // Text alignment
        if class == "text-left" {
            return Some("textAlign(TextAlign.Start)".to_string());
        }
        if class == "text-center" {
            return Some("textAlign(TextAlign.Center)".to_string());
        }
        if class == "text-right" {
            return Some("textAlign(TextAlign.End)".to_string());
        }
        if class == "text-justify" {
            return Some("textAlign(TextAlign.Justify)".to_string());
        }

        // Elevation (z-index equivalent)
        if class == "z-0" {
            return None; // default
        }
        if let Some(rest) = class.strip_prefix("z-") {
            if let Ok(n) = rest.parse::<u32>() {
                return Some(format!("zIndex({}f)", n));
            }
        }

        // Clip/Circle
        if class == "rounded-full" {
            return Some("clip(RoundedCornerShape(percent = 50))".to_string());
        }
        if class == "circle" {
            return Some("clip(CircleShape)".to_string());
        }

        // Aspect ratio
        if let Some(rest) = class.strip_prefix("aspect-") {
            if rest == "square" {
                return Some("aspectRatio(1f)".to_string());
            }
            if rest == "video" {
                return Some("aspectRatio(16f/9f)".to_string());
            }
            // Parse ratio like "aspect-4-3" or "aspect-16-9"
            let parts: Vec<&str> = rest.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(w), Ok(h)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                    return Some(format!("aspectRatio({}f/{}f)", w, h));
                }
            }
        }

        // Clickable
        if class == "cursor-pointer" || class == "clickable" {
            return Some("clickable { }".to_string());
        }

        // Scroll
        if class == "overflow-auto" || class == "overflow-scroll" {
            return Some("verticalScroll(rememberScrollState())".to_string());
        }
        if class == "overflow-x-auto" || class == "overflow-x-scroll" {
            return Some("horizontalScroll(rememberScrollState())".to_string());
        }

        None
    }

    /// Convert Tailwind color name to hex
    fn tailwind_color_to_hex(&self, name: &str) -> Option<String> {
        let colors = [
            ("white", "0xFFFFFFFF"),
            ("black", "0xFF000000"),
            ("transparent", "0x00000000"),
            ("red-500", "0xFFEF4444"),
            ("blue-500", "0xFF3B82F6"),
            ("green-500", "0xFF22C55E"),
            ("yellow-500", "0xFFEAB308"),
            ("purple-500", "0xFFA855F7"),
            ("pink-500", "0xFFEC4899"),
            ("gray-500", "0xFF6B7280"),
            ("gray-100", "0xFFF3F4F6"),
            ("gray-200", "0xFFE5E7EB"),
            ("gray-300", "0xFFD1D5DB"),
            ("gray-400", "0xFF9CA3AF"),
            ("gray-600", "0xFF4B5563"),
            ("gray-700", "0xFF374151"),
            ("gray-800", "0xFF1F2937"),
            ("gray-900", "0xFF111827"),
        ];

        for (key, value) in colors {
            if name == key {
                return Some(value.to_string());
            }
        }

        // Try parsing as hex color directly
        if name.starts_with('#') {
            let hex = name.trim_start_matches('#');
            if hex.len() == 6 {
                return Some(format!("0xFF{}", hex.to_uppercase()));
            }
        }

        None
    }

    /// Convert full class string to ModifierResult
    pub fn convert_class(&self, class: &str) -> ModifierResult {
        let mut modifiers: Vec<String> = Vec::new();
        let mut arrangement: Option<String> = None;

        for part in class.split_whitespace() {
            // Handle gap separately (it's for Column/Row arrangement)
            if let Some(rest) = part.strip_prefix("gap-") {
                if let Ok(n) = rest.parse::<u32>() {
                    arrangement = Some(format!("Arrangement.spacedBy({})", self.to_dp(n)));
                }
                continue;
            }

            if let Some(modifier) = self.convert_single(part) {
                modifiers.push(modifier);
            }
        }

        ModifierResult {
            modifiers,
            arrangement,
        }
    }

    /// Generate Modifier chain code
    pub fn generate_modifier_chain(&self, class: &str) -> String {
        let result = self.convert_class(class);
        if result.modifiers.is_empty() {
            "Modifier".to_string()
        } else {
            format!("Modifier.{}", result.modifiers.join("."))
        }
    }
}

impl Default for ModifierDsl {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("px-4 py-2");
        assert!(result.modifiers.iter().any(|m| m.contains("padding(horizontal = 16.dp)")));
        assert!(result.modifiers.iter().any(|m| m.contains("padding(vertical = 8.dp)")));
    }

    #[test]
    fn test_gap_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("gap-4");
        assert!(result.arrangement.is_some());
        assert!(result.arrangement.unwrap().contains("Arrangement.spacedBy(16.dp)"));
    }

    #[test]
    fn test_fill_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("w-full h-full");
        assert!(result.modifiers.iter().any(|m| m.contains("fillMaxWidth()")));
        assert!(result.modifiers.iter().any(|m| m.contains("fillMaxHeight()")));
    }

    #[test]
    fn test_rounded_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("rounded-lg");
        assert!(result.modifiers.iter().any(|m| m.contains("rounded(8.dp)")));
    }

    #[test]
    fn test_background_color() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("bg-blue-500");
        assert!(result.modifiers.iter().any(|m| m.contains("background(Color(") && m.contains("0xFF3B82F6")));
    }

    #[test]
    fn test_shadow_conversion() {
        let dsl = ModifierDsl::new();
        let result = dsl.convert_class("shadow-md");
        assert!(result.modifiers.iter().any(|m| m.contains("shadow(4.dp)")));
    }

    #[test]
    fn test_modifier_chain_generation() {
        let dsl = ModifierDsl::new();
        let chain = dsl.generate_modifier_chain("px-4 rounded-lg");
        assert!(chain.starts_with("Modifier."));
        assert!(chain.contains("padding"));
        assert!(chain.contains("rounded"));
    }

    // =========================================================================
    // Phase 3: Enhanced Modifier Tests
    // =========================================================================

    #[test]
    fn test_opacity_modifier() {
        let dsl = ModifierDsl::new();

        // Test opacity-50
        let result = dsl.convert_single("opacity-50");
        assert!(result.is_some());
        let modifier = result.unwrap();
        assert!(modifier.contains("alpha"));
        assert!(modifier.contains("0.50"));

        // Test opacity-100
        let result = dsl.convert_single("opacity-100");
        assert!(result.is_some());
        assert!(result.unwrap().contains("1.00"));

        // Test opacity-0
        let result = dsl.convert_single("opacity-0");
        assert!(result.is_some());
        assert!(result.unwrap().contains("0.00"));
    }

    #[test]
    fn test_font_size_modifiers() {
        let dsl = ModifierDsl::new();

        // Test standard sizes
        assert!(dsl.convert_single("text-xs").unwrap().contains("12.sp"));
        assert!(dsl.convert_single("text-sm").unwrap().contains("14.sp"));
        assert!(dsl.convert_single("text-base").unwrap().contains("16.sp"));
        assert!(dsl.convert_single("text-lg").unwrap().contains("18.sp"));
        assert!(dsl.convert_single("text-xl").unwrap().contains("20.sp"));
        assert!(dsl.convert_single("text-2xl").unwrap().contains("24.sp"));
        assert!(dsl.convert_single("text-3xl").unwrap().contains("30.sp"));
        assert!(dsl.convert_single("text-4xl").unwrap().contains("36.sp"));

        // Test custom size
        assert!(dsl.convert_single("text-size-48").unwrap().contains("48.sp"));
    }

    #[test]
    fn test_font_weight_modifiers() {
        let dsl = ModifierDsl::new();

        assert!(dsl.convert_single("font-thin").unwrap().contains("FontWeight.Thin"));
        assert!(dsl.convert_single("font-light").unwrap().contains("FontWeight.Light"));
        assert!(dsl.convert_single("font-normal").unwrap().contains("FontWeight.Normal"));
        assert!(dsl.convert_single("font-medium").unwrap().contains("FontWeight.Medium"));
        assert!(dsl.convert_single("font-semibold").unwrap().contains("FontWeight.SemiBold"));
        assert!(dsl.convert_single("font-bold").unwrap().contains("FontWeight.Bold"));
        assert!(dsl.convert_single("font-extrabold").unwrap().contains("FontWeight.ExtraBold"));
    }

    #[test]
    fn test_text_alignment_modifiers() {
        let dsl = ModifierDsl::new();

        assert!(dsl.convert_single("text-left").unwrap().contains("TextAlign.Start"));
        assert!(dsl.convert_single("text-center").unwrap().contains("TextAlign.Center"));
        assert!(dsl.convert_single("text-right").unwrap().contains("TextAlign.End"));
        assert!(dsl.convert_single("text-justify").unwrap().contains("TextAlign.Justify"));
    }

    #[test]
    fn test_z_index_modifier() {
        let dsl = ModifierDsl::new();

        let result = dsl.convert_single("z-10");
        assert!(result.is_some());
        assert!(result.unwrap().contains("zIndex(10f)"));

        let result = dsl.convert_single("z-0");
        // z-0 should return None (default, no modifier needed)
        assert!(result.is_none());
    }

    #[test]
    fn test_aspect_ratio_modifiers() {
        let dsl = ModifierDsl::new();

        // Test square
        assert!(dsl.convert_single("aspect-square").unwrap().contains("aspectRatio(1f)"));

        // Test video (16:9)
        assert!(dsl.convert_single("aspect-video").unwrap().contains("aspectRatio(16f/9f)"));

        // Test custom ratio (4:3)
        assert!(dsl.convert_single("aspect-4-3").unwrap().contains("aspectRatio(4f/3f)"));
    }

    #[test]
    fn test_min_max_width_modifiers() {
        let dsl = ModifierDsl::new();

        // Min width
        let result = dsl.convert_single("min-w-100");
        assert!(result.is_some());
        let modifier = result.unwrap();
        assert!(modifier.contains("IntrinsicSize.Min"));
        assert!(modifier.contains("400.dp"));

        // Max width
        let result = dsl.convert_single("max-w-200");
        assert!(result.is_some());
        assert!(result.unwrap().contains("IntrinsicSize.Max"));
    }

    #[test]
    fn test_min_max_height_modifiers() {
        let dsl = ModifierDsl::new();

        // Min height
        let result = dsl.convert_single("min-h-50");
        assert!(result.is_some());
        let modifier = result.unwrap();
        assert!(modifier.contains("IntrinsicSize.Min"));
        assert!(modifier.contains("200.dp"));

        // Max height
        let result = dsl.convert_single("max-h-100");
        assert!(result.is_some());
        assert!(result.unwrap().contains("IntrinsicSize.Max"));
    }

    #[test]
    fn test_circle_modifier() {
        let dsl = ModifierDsl::new();

        assert!(dsl.convert_single("circle").unwrap().contains("CircleShape"));
    }

    #[test]
    fn test_clickable_modifier() {
        let dsl = ModifierDsl::new();

        assert!(dsl.convert_single("clickable").unwrap().contains("clickable"));
        assert!(dsl.convert_single("cursor-pointer").unwrap().contains("clickable"));
    }

    #[test]
    fn test_scroll_modifiers() {
        let dsl = ModifierDsl::new();

        // Vertical scroll
        let result = dsl.convert_single("overflow-auto");
        assert!(result.unwrap().contains("verticalScroll"));

        let result = dsl.convert_single("overflow-scroll");
        assert!(result.unwrap().contains("verticalScroll"));

        // Horizontal scroll
        let result = dsl.convert_single("overflow-x-auto");
        assert!(result.unwrap().contains("horizontalScroll"));
    }

    #[test]
    fn test_border_width_modifiers() {
        let dsl = ModifierDsl::new();

        // Border uses Tailwind to Dp multiplier (value * 4)
        assert!(dsl.convert_single("border-0").unwrap().contains("0.dp"));
        assert!(dsl.convert_single("border-2").unwrap().contains("8.dp")); // 2 * 4 = 8
        assert!(dsl.convert_single("border-4").unwrap().contains("16.dp")); // 4 * 4 = 16
        assert!(dsl.convert_single("border-8").unwrap().contains("32.dp")); // 8 * 4 = 32
    }

    #[test]
    fn test_flex_modifiers() {
        let dsl = ModifierDsl::new();

        // flex-1 should generate weight modifier
        assert!(dsl.convert_single("flex-1").unwrap().contains("weight(1f)"));

        // flex-row and flex-col return None (handled by component choice)
        assert!(dsl.convert_single("flex-row").is_none());
        assert!(dsl.convert_single("flex-col").is_none());
        assert!(dsl.convert_single("flex").is_none());
    }

    #[test]
    fn test_combined_modifiers() {
        let dsl = ModifierDsl::new();

        // Test combining multiple modifiers
        let result = dsl.convert_class("px-4 py-2 rounded-lg bg-blue-500 text-white opacity-90");
        assert!(result.modifiers.iter().any(|m| m.contains("padding(horizontal")));
        assert!(result.modifiers.iter().any(|m| m.contains("padding(vertical")));
        assert!(result.modifiers.iter().any(|m| m.contains("rounded")));
        assert!(result.modifiers.iter().any(|m| m.contains("background")));
        assert!(result.modifiers.iter().any(|m| m.contains("color")));
        assert!(result.modifiers.iter().any(|m| m.contains("alpha")));

        // Generate chain
        let chain = dsl.generate_modifier_chain("px-4 rounded-lg opacity-80 text-bold");
        assert!(chain.starts_with("Modifier."));
    }

    #[test]
    fn test_unknown_modifier() {
        let dsl = ModifierDsl::new();

        // Unknown class should return None
        assert!(dsl.convert_single("unknown-class").is_none());
        assert!(dsl.convert_single("random-value").is_none());
    }
}
