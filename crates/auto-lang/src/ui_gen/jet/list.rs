//! List Component Generators
//!
//! Generates Jetpack Compose list components from AURA elements.
//!
//! ## Supported Components
//! - `list` → `LazyColumn`
//! - `list-row` → `LazyRow`
//! - `grid` → `LazyVerticalGrid`
//! - `flow-row` → `FlowRow`
//! - `flow-col` → `FlowColumn`

use crate::aura::{AuraExpr, AuraPropValue};
use crate::ui_gen::GenResult;
use std::collections::HashMap;

/// List component generator
pub struct ListGenerator {
    /// Track imports needed for list components
    imports: Vec<String>,
}

/// List item configuration
pub struct ListItemConfig {
    /// Data source variable name
    pub items_source: String,
    /// Key expression (e.g., "item.id")
    pub key_expr: Option<String>,
    /// Content type for lazy list optimization
    pub content_type: Option<String>,
    /// Item variable name (default: "item")
    pub item_var: String,
}

impl ListGenerator {
    /// Create a new list generator
    pub fn new() -> Self {
        Self {
            imports: Vec::new(),
        }
    }

    /// Get required imports for generated list components
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

    /// Extract int value from prop
    fn extract_int(props: &HashMap<String, AuraPropValue>, key: &str) -> Option<i64> {
        props.get(key).and_then(|p| match p {
            AuraPropValue::Expr(AuraExpr::Int(n)) => Some(*n),
            _ => None,
        })
    }

    /// Parse list item configuration from props
    fn parse_item_config(props: &HashMap<String, AuraPropValue>) -> ListItemConfig {
        let items_source = Self::extract_string(props, "items")
            .or_else(|| Self::extract_string(props, "data"))
            .unwrap_or_else(|| "items".to_string());

        let key_expr = Self::extract_string(props, "key")
            .or_else(|| Self::extract_string(props, "keyExpr"));

        let content_type = Self::extract_string(props, "contentType");

        let item_var = Self::extract_string(props, "itemVar")
            .unwrap_or_else(|| "item".to_string());

        ListItemConfig {
            items_source,
            key_expr,
            content_type,
            item_var,
        }
    }

    /// Generate LazyColumn component
    pub fn generate_lazy_column(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        item_content: &str,
    ) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.lazy.LazyColumn");
        self.add_import("androidx.compose.foundation.lazy.items");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.ui.Modifier");

        let item_config = Self::parse_item_config(props);
        let gap = Self::extract_int(props, "gap").map(|n| n as u32);
        let class = Self::extract_string(props, "class");

        let mut params = Vec::new();

        // Modifier
        if let Some(class_str) = class {
            params.push(format!("modifier = Modifier.{}", self.class_to_modifier(&class_str)));
        } else {
            params.push("modifier = Modifier.fillMaxSize()".to_string());
        }

        // Vertical arrangement (gap)
        if let Some(gap_val) = gap {
            let dp = gap_val * 4;
            params.push(format!("verticalArrangement = Arrangement.spacedBy({}.dp)", dp));
        }

        // Generate items block
        let items_block = self.generate_items_block(&item_config, item_content);

        Ok(format!(
            "LazyColumn(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            items_block
        ))
    }

    /// Generate LazyRow component
    pub fn generate_lazy_row(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        item_content: &str,
    ) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.lazy.LazyRow");
        self.add_import("androidx.compose.foundation.lazy.items");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.ui.Modifier");

        let item_config = Self::parse_item_config(props);
        let gap = Self::extract_int(props, "gap").map(|n| n as u32);
        let class = Self::extract_string(props, "class");

        let mut params = Vec::new();

        // Modifier
        if let Some(class_str) = class {
            params.push(format!("modifier = Modifier.{}", self.class_to_modifier(&class_str)));
        } else {
            params.push("modifier = Modifier".to_string());
        }

        // Horizontal arrangement (gap)
        if let Some(gap_val) = gap {
            let dp = gap_val * 4;
            params.push(format!("horizontalArrangement = Arrangement.spacedBy({}.dp)", dp));
        }

        // Generate items block
        let items_block = self.generate_items_block(&item_config, item_content);

        Ok(format!(
            "LazyRow(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            items_block
        ))
    }

    /// Generate LazyVerticalGrid component
    pub fn generate_lazy_grid(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        item_content: &str,
    ) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.lazy.grid.LazyVerticalGrid");
        self.add_import("androidx.compose.foundation.lazy.grid.GridCells");
        self.add_import("androidx.compose.foundation.lazy.grid.items");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.ui.Modifier");

        let item_config = Self::parse_item_config(props);
        let columns = Self::extract_int(props, "columns")
            .or_else(|| Self::extract_int(props, "cols"))
            .unwrap_or(2) as u32;
        let gap = Self::extract_int(props, "gap").map(|n| n as u32);
        let class = Self::extract_string(props, "class");

        let mut params = Vec::new();

        // GridCells
        params.push(format!("columns = GridCells.Fixed({})", columns));

        // Modifier
        if let Some(class_str) = class {
            params.push(format!("modifier = Modifier.{}", self.class_to_modifier(&class_str)));
        } else {
            params.push("modifier = Modifier.fillMaxSize()".to_string());
        }

        // Arrangement (gap)
        if let Some(gap_val) = gap {
            let dp = gap_val * 4;
            params.push(format!("verticalArrangement = Arrangement.spacedBy({}.dp)", dp));
            params.push(format!("horizontalArrangement = Arrangement.spacedBy({}.dp)", dp));
        }

        // Generate items block for grid
        let items_block = self.generate_grid_items_block(&item_config, item_content);

        Ok(format!(
            "LazyVerticalGrid(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            items_block
        ))
    }

    /// Generate FlowRow component (for dynamic data)
    pub fn generate_flow_row(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        item_content: &str,
    ) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.ExperimentalLayoutApi");
        self.add_import("androidx.compose.foundation.layout.FlowRow");
        self.add_import("androidx.compose.ui.Modifier");

        let item_config = Self::parse_item_config(props);
        let class = Self::extract_string(props, "class");

        let modifier = if let Some(class_str) = class {
            format!("Modifier.{}", self.class_to_modifier(&class_str))
        } else {
            "Modifier".to_string()
        };

        // For FlowRow, we don't use lazy items - we map directly
        let map_content = self.generate_flow_item(&item_config, item_content);

        Ok(format!(
            "FlowRow(\n        modifier = {}\n    ) {{\n        {}\n    }}",
            modifier, map_content
        ))
    }

    /// Generate static grid using FlowRow for static children (cards, etc.)
    /// This is used when grid has static child elements instead of dynamic data
    pub fn generate_static_grid(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        children_content: &str,
    ) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.ExperimentalLayoutApi");
        self.add_import("androidx.compose.foundation.layout.FlowRow");
        self.add_import("androidx.compose.foundation.layout.Arrangement");
        self.add_import("androidx.compose.ui.Modifier");
        self.add_import("androidx.compose.foundation.shape.RoundedCornerShape");
        self.add_import("androidx.compose.ui.graphics.Color");
        self.add_import("androidx.compose.ui.draw.clip");

        let mut gap = Self::extract_int(props, "gap").map(|n| n as u32);
        let class = Self::extract_string(props, "class");

        // Parse gap from class if not in props (e.g., "gap-4")
        if gap.is_none() {
            if let Some(class_str) = &class {
                for part in class_str.split_whitespace() {
                    if let Some(rest) = part.strip_prefix("gap-") {
                        if let Ok(n) = rest.parse::<u32>() {
                            gap = Some(n);
                            break;
                        }
                    }
                }
            }
        }

        let mut params = Vec::new();

        // Modifier (excluding gap which is handled by Arrangement)
        let modifier = if let Some(class_str) = &class {
            let class_mods = self.class_to_modifier_excluding_gap(&class_str);
            if class_mods.is_empty() {
                "Modifier".to_string()
            } else {
                format!("Modifier.{}", class_mods)
            }
        } else {
            "Modifier".to_string()
        };
        params.push(format!("modifier = {}", modifier));

        // Horizontal arrangement (gap)
        if let Some(gap_val) = gap {
            let dp = gap_val * 4;
            params.push(format!("horizontalArrangement = Arrangement.spacedBy({}.dp)", dp));
            params.push(format!("verticalArrangement = Arrangement.spacedBy({}.dp)", dp));
        }

        Ok(format!(
            "FlowRow(\n        {}\n    ) {{\n        {}\n    }}",
            params.join(",\n        "),
            children_content
        ))
    }

    /// Convert Tailwind class string to Modifier chain, excluding gap classes
    fn class_to_modifier_excluding_gap(&self, class: &str) -> String {
        let mut modifiers = Vec::new();

        for part in class.split_whitespace() {
            // Skip gap classes - they are handled by Arrangement
            if part.starts_with("gap-") {
                continue;
            }

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

            // Rounded - use clip(RoundedCornerShape(...))
            if part == "rounded" {
                modifiers.push("clip(RoundedCornerShape(4.dp))".to_string());
            }
            if part == "rounded-lg" {
                modifiers.push("clip(RoundedCornerShape(8.dp))".to_string());
            }

            // Background colors
            if let Some(color) = part.strip_prefix("bg-") {
                if let Some(hex) = self.tailwind_color_to_hex(color) {
                    modifiers.push(format!("background(Color({}))", hex));
                }
            }
        }

        if modifiers.is_empty() {
            String::new()
        } else {
            modifiers.join(".")
        }
    }

    /// Generate FlowColumn component
    pub fn generate_flow_column(
        &mut self,
        props: &HashMap<String, AuraPropValue>,
        item_content: &str,
    ) -> GenResult<String> {
        self.add_import("androidx.compose.foundation.layout.ExperimentalLayoutApi");
        self.add_import("androidx.compose.foundation.layout.FlowColumn");
        self.add_import("androidx.compose.ui.Modifier");

        let item_config = Self::parse_item_config(props);
        let class = Self::extract_string(props, "class");

        let modifier = if let Some(class_str) = class {
            format!("Modifier.{}", self.class_to_modifier(&class_str))
        } else {
            "Modifier".to_string()
        };

        let map_content = self.generate_flow_item(&item_config, item_content);

        Ok(format!(
            "FlowColumn(\n        modifier = {}\n    ) {{\n        {}\n    }}",
            modifier, map_content
        ))
    }

    /// Generate items block for LazyColumn/LazyRow
    fn generate_items_block(&self, config: &ListItemConfig, item_content: &str) -> String {
        let item_var = &config.item_var;

        // Build items call
        let mut items_params = vec![format!("items = {}", config.items_source)];

        // Add key
        if let Some(key) = &config.key_expr {
            // Replace "item" with actual variable name if different
            let key_expr = if item_var != "item" {
                key.replace("item", item_var)
            } else {
                key.clone()
            };
            items_params.push(format!("key = {{ {} -> {} }}", item_var, key_expr));
        }

        // Add contentType
        if let Some(ct) = &config.content_type {
            items_params.push(format!("contentType = \"{}\"", ct));
        }

        // Replace item variable in content
        let content = if item_var != "item" {
            item_content.replace("item", item_var)
        } else {
            item_content.to_string()
        };

        format!(
            "items(\n            {}\n        ) {{ {} ->\n            {}\n        }}",
            items_params.join(",\n            "),
            item_var,
            content
        )
    }

    /// Generate items block for LazyVerticalGrid
    fn generate_grid_items_block(&self, config: &ListItemConfig, item_content: &str) -> String {
        let item_var = &config.item_var;

        let mut items_params = vec![format!("items = {}", config.items_source)];

        if let Some(key) = &config.key_expr {
            let key_expr = if item_var != "item" {
                key.replace("item", item_var)
            } else {
                key.clone()
            };
            items_params.push(format!("key = {{ {} -> {} }}", item_var, key_expr));
        }

        if let Some(ct) = &config.content_type {
            items_params.push(format!("contentType = \"{}\"", ct));
        }

        let content = if item_var != "item" {
            item_content.replace("item", item_var)
        } else {
            item_content.to_string()
        };

        format!(
            "items(\n            {}\n        ) {{ {} ->\n            {}\n        }}",
            items_params.join(",\n            "),
            item_var,
            content
        )
    }

    /// Generate flow item (uses map instead of lazy items)
    fn generate_flow_item(&self, config: &ListItemConfig, item_content: &str) -> String {
        let item_var = &config.item_var;

        let content = if item_var != "item" {
            item_content.replace("item", item_var)
        } else {
            item_content.to_string()
        };

        format!(
            "{}.forEach {{ {} ->\n            {}\n        }}",
            config.items_source, item_var, content
        )
    }

    /// Convert Tailwind class string to Modifier chain
    /// Returns empty string if no modifiers are generated
    fn class_to_modifier(&self, class: &str) -> String {
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

            // Rounded - use clip(RoundedCornerShape(...))
            if part == "rounded" {
                modifiers.push("clip(RoundedCornerShape(4.dp))".to_string());
            }
            if part == "rounded-lg" {
                modifiers.push("clip(RoundedCornerShape(8.dp))".to_string());
            }

            // Background colors
            if let Some(color) = part.strip_prefix("bg-") {
                if let Some(hex) = self.tailwind_color_to_hex(color) {
                    modifiers.push(format!("background(Color({}))", hex));
                }
            }
        }

        if modifiers.is_empty() {
            String::new()
        } else {
            modifiers.join(".")
        }
    }

    /// Convert Tailwind color name to hex
    fn tailwind_color_to_hex(&self, name: &str) -> Option<String> {
        let colors = [
            ("white", "0xFFFFFFFF"),
            ("black", "0xFF000000"),
            ("red-500", "0xFFEF4444"),
            ("blue-500", "0xFF3B82F6"),
            ("green-500", "0xFF22C55E"),
            ("gray-100", "0xFFF3F4F6"),
            ("gray-800", "0xFF1F2937"),
        ];

        for (key, value) in colors {
            if name == key {
                return Some(value.to_string());
            }
        }

        if name.starts_with('#') {
            let hex = name.trim_start_matches('#');
            if hex.len() == 6 {
                return Some(format!("0xFF{}", hex.to_uppercase()));
            }
        }

        None
    }
}

impl Default for ListGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_lazy_column_basic() {
        let mut gen = ListGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_lazy_column(&props, "Text(item.name)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("LazyColumn"));
        assert!(code.contains("items"));
    }

    #[test]
    fn test_generate_lazy_column_with_items() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("items".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("users".to_string())));

        let result = gen.generate_lazy_column(&props, "UserItem(user = item)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("items = users"));
    }

    #[test]
    fn test_generate_lazy_column_with_key() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("items".to_string(), AuraPropValue::Expr(AuraExpr::StateRef("todos".to_string())));
        props.insert("key".to_string(), AuraPropValue::Expr(AuraExpr::Literal("item.id".to_string())));

        let result = gen.generate_lazy_column(&props, "TodoItem(todo = item)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("key = { item -> item.id }"));
    }

    #[test]
    fn test_generate_lazy_column_with_gap() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("gap".to_string(), AuraPropValue::Expr(AuraExpr::Int(4)));

        let result = gen.generate_lazy_column(&props, "Text(\"Item\")");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("spacedBy(16.dp)")); // 4 * 4 = 16
    }

    #[test]
    fn test_generate_lazy_row_basic() {
        let mut gen = ListGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_lazy_row(&props, "Card { item }");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("LazyRow"));
    }

    #[test]
    fn test_generate_lazy_row_with_gap() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("gap".to_string(), AuraPropValue::Expr(AuraExpr::Int(2)));

        let result = gen.generate_lazy_row(&props, "Chip(text = item)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("horizontalArrangement"));
        assert!(code.contains("spacedBy(8.dp)")); // 2 * 4 = 8
    }

    #[test]
    fn test_generate_lazy_grid_basic() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("columns".to_string(), AuraPropValue::Expr(AuraExpr::Int(3)));

        let result = gen.generate_lazy_grid(&props, "PhotoItem(photo = item)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("LazyVerticalGrid"));
        assert!(code.contains("GridCells.Fixed(3)"));
    }

    #[test]
    fn test_generate_lazy_grid_with_gap() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("columns".to_string(), AuraPropValue::Expr(AuraExpr::Int(2)));
        props.insert("gap".to_string(), AuraPropValue::Expr(AuraExpr::Int(4)));

        let result = gen.generate_lazy_grid(&props, "Card { item }");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("verticalArrangement"));
        assert!(code.contains("horizontalArrangement"));
    }

    #[test]
    fn test_generate_flow_row() {
        let mut gen = ListGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_flow_row(&props, "Chip(item.name)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("FlowRow"));
        assert!(code.contains("forEach"));
    }

    #[test]
    fn test_generate_flow_column() {
        let mut gen = ListGenerator::new();
        let props = HashMap::new();

        let result = gen.generate_flow_column(&props, "Tag(item)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("FlowColumn"));
    }

    #[test]
    fn test_item_variable_customization() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("itemVar".to_string(), AuraPropValue::Expr(AuraExpr::Literal("user".to_string())));

        let result = gen.generate_lazy_column(&props, "UserItem(u = user)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("user ->"));
    }

    #[test]
    fn test_content_type() {
        let mut gen = ListGenerator::new();
        let mut props = HashMap::new();

        props.insert("contentType".to_string(), AuraPropValue::Expr(AuraExpr::Literal("todo-item".to_string())));

        let result = gen.generate_lazy_column(&props, "Text(item.title)");
        assert!(result.is_ok());
        let code = result.unwrap();
        assert!(code.contains("contentType = \"todo-item\""));
    }

    #[test]
    fn test_import_collection() {
        let mut gen = ListGenerator::new();
        let props = HashMap::new();

        let _ = gen.generate_lazy_column(&props, "Text(\"test\")");

        let imports = gen.get_imports();
        assert!(imports.iter().any(|i| i.contains("LazyColumn")));
        assert!(imports.iter().any(|i| i.contains("items")));
    }

    #[test]
    fn test_class_to_modifier() {
        let gen = ListGenerator::new();

        let modifier = gen.class_to_modifier("p-4 w-full rounded-lg");
        assert!(modifier.contains("padding(16.dp)"));
        assert!(modifier.contains("fillMaxWidth()"));
        assert!(modifier.contains("rounded(8.dp)"));
    }

    #[test]
    fn test_tailwind_color_to_hex() {
        let gen = ListGenerator::new();

        assert_eq!(gen.tailwind_color_to_hex("white"), Some("0xFFFFFFFF".to_string()));
        assert_eq!(gen.tailwind_color_to_hex("blue-500"), Some("0xFF3B82F6".to_string()));
        assert_eq!(gen.tailwind_color_to_hex("#FF5733"), Some("0xFFFF5733".to_string()));
    }

    #[test]
    fn test_clear_imports() {
        let mut gen = ListGenerator::new();
        let props = HashMap::new();

        let _ = gen.generate_lazy_column(&props, "Text(\"test\")");
        assert!(!gen.get_imports().is_empty());

        gen.clear_imports();
        assert!(gen.get_imports().is_empty());
    }
}
