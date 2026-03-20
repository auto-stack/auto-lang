//! Layout Component Generators
//!
//! Generates Jetpack Compose layout components from AURA elements.
//!
//! ## Supported Components
//! - `col` → `Column`
//! - `row` → `Row`
//! - `box`/`container` → `Box`
//! - `card` → `Card`
//! - `scroll` → `Column + verticalScroll`

use crate::aura::{AuraExpr, AuraPropValue};
use crate::ui_gen::GenResult;
use std::collections::HashMap;

/// Layout component generator
pub struct LayoutGenerator {
    /// Track imports needed for layout components
    imports: Vec<String>,
}

/// Layout properties extracted from AURA
#[allow(dead_code)]
pub struct LayoutProps {
    /// Gap between children (in Tailwind units)
    pub gap: Option<u32>,
    /// Padding (in dp)
    pub padding: Option<u32>,
    /// Vertical alignment (for Row) or horizontal alignment (for Column)
    pub vertical_align: Option<String>,
    /// Horizontal arrangement (for Row)
    pub horizontal_arrange: Option<String>,
    /// Vertical arrangement (for Column) - top/center/bottom/between/around/evenly
    pub vertical_arrange: Option<String>,
    /// Tailwind CSS classes
    pub class: Option<String>,
    /// Modifier chain
    pub modifier: Option<String>,
}

impl LayoutGenerator {
    /// Create a new layout generator
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }

    /// Get required imports for generated layout components
    pub fn get_imports(&self) -> &[String] {
        &self.imports
    }

    /// Clear imports for fresh generation
    pub fn clear_imports(&mut self) {
        self.imports.clear();
    }

    /// Add import if not already present
    fn add_import(&mut self, import: &str) {
        if !self.imports.iter().any(|i| i == import) {
            self.imports.push(import.to_string());
        }
    }

    /// Extract string value from prop
    fn extract_string(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<String> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Literal(s)) => Some(s.clone()),
            AuraPropValue::Expr(AuraExpr::StateRef(s)) => Some(s.clone()),
            _ => None,
        })
    }

    /// Extract int value from prop (handles Int, Float, and String forms)
    fn extract_int(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<i64> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n),
            AuraPropValue::Expr(AuraExpr::Float(n)) => Some(*n as i64),
            AuraPropValue::Expr(AuraExpr::Literal(s)) => s.parse::<i64>().ok(),
            _ => None,
        })
    }

    /// Parse layout properties from AURA props
    fn parse_layout_props(&mut self, props: &HashMap<String, AuraPropValue>) -> LayoutProps {
        let gap = Self::extract_int(props, "gap").map(|n| n as u32);
        let padding = Self::extract_int(props, "padding").map(|n| n as u32);
        let vertical_align = Self::extract_string(props, "align")
            .or_else(|| Self::extract_string(props, "vertical_align"));
        let horizontal_arrange = Self::extract_string(props, "justify")
            .or_else(|| Self::extract_string(props, "horizontal_arrange"));
        let vertical_arrange = Self::extract_string(props, "arrange")
            .or_else(|| Self::extract_string(props, "vertical_arrange"));
        let class = Self::extract_string(props, "class");

        LayoutProps {
            gap,
            padding,
            vertical_align,
            horizontal_arrange,
            vertical_arrange,
            class,
            modifier: None,
        }
    }

    /// Generate Column component
    pub fn generate_column(&mut self, props: &HashMap<String, AuraPropValue>, children: &str) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.Column");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.foundation.layout.padding");
        self.add_import("androidx.compose.ui.Alignment");
        self.add_import("androidx.compose.ui.Modifier");

        let layout_props = self.parse_layout_props(props);

        let mut params = Vec::new();

        // Build modifier chain
        let mut modifier_parts = Vec::new();

        // Add padding if specified
        if let Some(padding) = layout_props.padding {
            modifier_parts.push(format!("padding({}.dp)", padding));
        }

        // Add class-based modifiers
        if let Some(class) = &layout_props.class {
            modifier_parts.push(self.class_to_modifier(class));
        }

        // Build final modifier
        if modifier_parts.is_empty() {
            params.push("modifier = Modifier".to_string());
        } else {
            params.push(format!("modifier = Modifier.{}", modifier_parts.join(".")));
        }

        // Vertical arrangement (gap + arrange)
        let arrangement = if let Some(gap) = layout_props.gap {
            let dp = gap * 4; // Tailwind to Dp multiplier
            match layout_props.vertical_arrange.as_deref() {
                Some("center") => format!("Arrangement.spacedBy({}.dp, Alignment.CenterVertically)", dp),
                Some("bottom" | "end") => format!("Arrangement.spacedBy({}.dp, Alignment.Bottom)", dp),
                Some("between") => format!("Arrangement.spacedBy({}.dp, Alignment.SpaceBetween)", dp),
                Some("around") => format!("Arrangement.spacedBy({}.dp, Alignment.SpaceAround)", dp),
                Some("evenly") => format!("Arrangement.spacedBy({}.dp, Alignment.SpaceEvenly)", dp),
                _ => format!("Arrangement.spacedBy({}.dp)", dp),
            }
        } else if let Some(arrange) = &layout_props.vertical_arrange {
            match arrange.as_str() {
                "center" => "Arrangement.Center".to_string(),
                "bottom" | "end" => "Arrangement.Bottom".to_string(),
                "between" => "Arrangement.SpaceBetween".to_string(),
                "around" => "Arrangement.SpaceAround".to_string(),
                "evenly" => "Arrangement.SpaceEvenly".to_string(),
                _ => "Arrangement.Top".to_string(),
            }
        } else {
            String::new()
        };

        if !arrangement.is_empty() {
            params.push(format!("verticalArrangement = {}", arrangement));
        }

        // Horizontal alignment
        if let Some(align) = &layout_props.vertical_align {
            let alignment = match align.as_str() {
                "start" | "left" => "Alignment.Start",
                "center" => "Alignment.CenterHorizontally",
                "end" | "right" => "Alignment.End",
                _ => "Alignment.Start",
            };
            params.push(format!("horizontalAlignment = {}", alignment));
        }

        Ok(format!(
            "Column(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            children
        ))
    }

    /// Generate Row component
    pub fn generate_row(&mut self, props: &HashMap<String, AuraPropValue>, children: &str) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.Row");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.foundation.layout.padding");
        self.add_import("androidx.compose.ui.Alignment");
        self.add_import("androidx.compose.ui.Modifier");

        let layout_props = self.parse_layout_props(props);

        let mut params = Vec::new();

        // Build modifier chain
        let mut modifier_parts = Vec::new();

        // Add padding if specified
        if let Some(padding) = layout_props.padding {
            modifier_parts.push(format!("padding({}.dp)", padding));
        }

        // Add class-based modifiers
        if let Some(class) = &layout_props.class {
            modifier_parts.push(self.class_to_modifier(class));
        }

        // Build final modifier
        if modifier_parts.is_empty() {
            params.push("modifier = Modifier".to_string());
        } else {
            params.push(format!("modifier = Modifier.{}", modifier_parts.join(".")));
        }

        // Horizontal arrangement (gap + justify)
        let arrangement = if let Some(gap) = layout_props.gap {
            let dp = gap * 4;
            match layout_props.horizontal_arrange.as_deref() {
                Some("between") => format!("Arrangement.spacedBy({}.dp, Alignment.SpaceBetween)", dp),
                Some("around") => format!("Arrangement.spacedBy({}.dp, Alignment.SpaceAround)", dp),
                Some("evenly") => format!("Arrangement.spacedBy({}.dp, Alignment.SpaceEvenly)", dp),
                _ => format!("Arrangement.spacedBy({}.dp)", dp),
            }
        } else {
            match layout_props.horizontal_arrange.as_deref() {
                Some("between") => "Arrangement.SpaceBetween".to_string(),
                Some("around") => "Arrangement.SpaceAround".to_string(),
                Some("evenly") => "Arrangement.SpaceEvenly".to_string(),
                _ => "Arrangement.Start".to_string(),
            }
        };
        params.push(format!("horizontalArrangement = {}", arrangement));

        // Vertical alignment
        if let Some(align) = &layout_props.vertical_align {
            let alignment = match align.as_str() {
                "top" | "start" => "Alignment.Top",
                "center" => "Alignment.CenterVertically",
                "bottom" | "end" => "Alignment.Bottom",
                _ => "Alignment.CenterVertically",
            };
            params.push(format!("verticalAlignment = {}", alignment));
        }

        Ok(format!(
            "Row(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            children
        ))
    }

    /// Generate Box component
    pub fn generate_box(&mut self, props: &HashMap<String, AuraPropValue>, children: &str) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.Box");
        self.add_import("androidx.compose.foundation.layout.padding");
        self.add_import("androidx.compose.ui.Alignment");
        self.add_import("androidx.compose.ui.Modifier");

        let layout_props = self.parse_layout_props(props);

        let mut params = Vec::new();

        // Build modifier chain
        let mut modifier_parts = Vec::new();

        // Add padding if specified
        if let Some(padding) = layout_props.padding {
            modifier_parts.push(format!("padding({}.dp)", padding));
        }

        // Add class-based modifiers
        if let Some(class) = &layout_props.class {
            modifier_parts.push(self.class_to_modifier(class));
        }

        // Build final modifier
        if modifier_parts.is_empty() {
            params.push("modifier = Modifier".to_string());
        } else {
            params.push(format!("modifier = Modifier.{}", modifier_parts.join(".")));
        }

        // Content alignment
        if let Some(align) = &layout_props.vertical_align {
            let alignment = match align.as_str() {
                "top-start" => "Alignment.TopStart",
                "top-center" => "Alignment.TopCenter",
                "top-end" => "Alignment.TopEnd",
                "center-start" => "Alignment.CenterStart",
                "center" => "Alignment.Center",
                "center-end" => "Alignment.CenterEnd",
                "bottom-start" => "Alignment.BottomStart",
                "bottom-center" => "Alignment.BottomCenter",
                "bottom-end" => "Alignment.BottomEnd",
                _ => "Alignment.TopStart",
            };
            params.push(format!("contentAlignment = {}", alignment));
        }

        Ok(format!(
            "Box(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            children
        ))
    }

    /// Generate Card component
    pub fn generate_card(&mut self, props: &HashMap<String, AuraPropValue>, children: &str) -> GenResult<String> {
        self.add_import("androidx.compose.material3.Card");
        self.add_import("androidx.compose.ui.Modifier");

        let class = Self::extract_string(props, "class");

        let modifier = if let Some(class_str) = class {
            format!("Modifier.{}", self.class_to_modifier(&class_str))
        } else {
            "Modifier".to_string()
        };

        Ok(format!(
            "Card(\n        modifier = {}\n    ) {{\n        {}\n    }}",
            modifier, children
        ))
    }

    /// Generate Scroll component (scrollable Column)
    pub fn generate_scroll(&mut self, props: &HashMap<String, AuraPropValue>, children: &str) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.Column");
        self.add_import("androidx.compose.foundation.verticalScroll");
        self.add_import("androidx.compose.foundation.rememberScrollState");
        self.add_import("androidx.compose.ui.Modifier");

        let class = Self::extract_string(props, "class");

        let mut modifier_parts = vec!["verticalScroll(rememberScrollState())".to_string()];

        if let Some(class_str) = class {
            modifier_parts.push(self.class_to_modifier(&class_str));
        }

        Ok(format!(
            "Column(\n        modifier = Modifier.{}\n    ) {{\n        {}\n    }}",
            modifier_parts.join("."),
            children
        ))
    }

    /// Convert Tailwind class string to Modifier chain
    fn class_to_modifier(&self, class: &str) -> String {
        // Simplified conversion - in production, use ModifierDsl
        let mut modifiers = Vec::new();

        for part in class.split_whitespace() {
            // Padding
            if let Some(rest) = part.strip_prefix("px-") {
                if let Ok(n) = rest.parse::<u32>() {
                    modifiers.push(format!("padding(horizontal = {}.dp)", n * 4));
                }
            }
            if let Some(rest) = part.strip_prefix("py-") {
                if let Ok(n) = rest.parse::<u32>() {
                    modifiers.push(format!("padding(vertical = {}.dp)", n * 4));
                }
            }
            if let Some(rest) = part.strip_prefix("p-") {
                if let Ok(n) = rest.parse::<u32>() {
                    modifiers.push(format!("padding({}.dp)", n * 4));
                }
            }

            // Size
            if part == "w-full" {
                modifiers.push("fillMaxWidth()".to_string());
            }
            if part == "h-full" {
                modifiers.push("fillMaxHeight()".to_string());
            }

            // Rounded
            if part == "rounded" {
                modifiers.push("rounded(4.dp)".to_string());
            }
            if part == "rounded-sm" {
                modifiers.push("rounded(2.dp)".to_string());
            }
            if part == "rounded-lg" {
                modifiers.push("rounded(8.dp)".to_string());
            }
            if part == "rounded-xl" {
                modifiers.push("rounded(12.dp)".to_string());
            }

            // Background colors (simplified)
            if let Some(color) = part.strip_prefix("bg-") {
                if let Some(hex) = self.tailwind_color_to_hex(color) {
                    modifiers.push(format!("background(Color({}))", hex));
                }
            }
        }

        if modifiers.is_empty() {
            "Modifier".to_string()
        } else {
            modifiers.join(".")
        }
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
}

impl Default for LayoutGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_column_basic() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_column(&props, "Text(\"Hello\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Column"));
        assert!(code.contains("Text(\"Hello\")"));
    }

    #[test]
    fn test_generate_column_with_gap() {
        let mut gen = LayoutGenerator::new();
        let mut props = HashMap::new();

        props.insert("gap".to_string(), AuraPropValue::Expr(AuraExpr::Int(4)));

        let result = gen.generate_column(&props, "Text(\"Hello\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("spacedBy(16.dp)")); // 4 * 4 = 16
    }

    #[test]
    fn test_generate_column_with_alignment() {
        let mut gen = LayoutGenerator::new();
        let mut props = HashMap::new();

        props.insert("align".to_string(), AuraPropValue::Expr(AuraExpr::Literal("center".to_string())));

        let result = gen.generate_column(&props, "Text(\"Hello\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Alignment.CenterHorizontally"));
    }

    #[test]
    fn test_generate_row_basic() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_row(&props, "Text(\"Left\")\n        Text(\"Right\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Row"));
    }

    #[test]
    fn test_generate_row_with_justify() {
        let mut gen = LayoutGenerator::new();
        let mut props = HashMap::new();

        props.insert("justify".to_string(), AuraPropValue::Expr(AuraExpr::Literal("between".to_string())));

        let result = gen.generate_row(&props, "Text(\"A\")\n        Text(\"B\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("SpaceBetween"));
    }

    #[test]
    fn test_generate_box_basic() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_box(&props, "Text(\"Overlay\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Box"));
    }

    #[test]
    fn test_generate_card_basic() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_card(&props, "Text(\"Card content\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Card"));
    }

    #[test]
    fn test_generate_card_with_class() {
        let mut gen = LayoutGenerator::new();
        let mut props = HashMap::new();

        props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal("p-4 rounded-lg".to_string())));

        let result = gen.generate_card(&props, "Text(\"Card\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Card"));
        assert!(code.contains("padding"));
    }

    #[test]
    fn test_generate_scroll_basic() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_scroll(&props, "Text(\"Line 1\")\n        Text(\"Line 2\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("Column"));
        assert!(code.contains("verticalScroll"));
        assert!(code.contains("rememberScrollState"));
    }

    #[test]
    fn test_import_collection() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let _ = gen.generate_column(&props, "Text(\"Test\")");

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("Column")));
        assert!(imports.iter().any(|i| i.contains("Modifier")));
    }

    #[test]
    fn test_class_to_modifier() {
        let gen = LayoutGenerator::new();

        let modifier = gen.class_to_modifier("px-4 py-2 w-full rounded-lg");
        assert!(modifier.contains("padding(horizontal = 16.dp)"));
        assert!(modifier.contains("padding(vertical = 8.dp)"));
        assert!(modifier.contains("fillMaxWidth()"));
        assert!(modifier.contains("rounded(8.dp)"));
    }

    #[test]
    fn test_tailwind_color_to_hex() {
        let gen = LayoutGenerator::new();

        assert_eq!(gen.tailwind_color_to_hex("white"), Some("0xFFFFFFFF".to_string()));
        assert_eq!(gen.tailwind_color_to_hex("black"), Some("0xFF000000".to_string()));
        assert_eq!(gen.tailwind_color_to_hex("blue-500"), Some("0xFF3B82F6".to_string()));
        assert_eq!(gen.tailwind_color_to_hex("#FF5733"), Some("0xFFFF5733".to_string()));
        assert_eq!(gen.tailwind_color_to_hex("unknown"), None);
    }

    #[test]
    fn test_clear_imports() {
        let mut gen = LayoutGenerator::new();
        let props = HashMap::new();

        let _ = gen.generate_column(&props, "Text(\"Test\")");
        assert!(!gen.get_imports().is_empty());

        gen.clear_imports();
        assert!(gen.get_imports().is_empty());
    }
}
