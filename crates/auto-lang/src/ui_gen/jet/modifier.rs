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
}
