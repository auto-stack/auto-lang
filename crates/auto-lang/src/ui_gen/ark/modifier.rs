//! ArkTS Modifier DSL
//!
//! Converts AURA style properties to ArkTS chainable modifiers.

use crate::ast::Type;

/// Convert a style property to ArkTS modifier
pub fn style_to_modifier(key: &str, value: &str) -> Option<String> {
    match key {
        // Size modifiers
        "width" => Some(format!(".width('{}')", value)),
        "height" => Some(format!(".height('{}')", value)),

        // Text modifiers
        "fontSize" => Some(format!(".fontSize({})", value)),
        "fontWeight" => Some(format!(".fontWeight(FontWeight.{})", value)),
        "fontColor" => Some(format!(".fontColor('{}')", value)),

        // Spacing modifiers
        "margin" => Some(format!(".margin({})", value)),
        "padding" => Some(format!(".padding({})", value)),

        // Layout modifiers
        "justifyContent" => Some(format!(".justifyContent(FlexAlign.{})", value)),
        "alignItems" => Some(format!(".alignItems(HorizontalAlign.{})", value)),

        // Background
        "backgroundColor" => Some(format!(".backgroundColor('{}')", value)),

        // Border
        "borderRadius" => Some(format!(".borderRadius({})", value)),
        "borderWidth" => Some(format!(".borderWidth({})", value)),
        "borderColor" => Some(format!(".borderColor('{}')", value)),

        // Opacity
        "opacity" => Some(format!(".opacity({})", value)),

        _ => None,
    }
}

/// Convert AURA prop to ArkTS modifier
pub fn prop_to_modifier(key: &str, value: &str, _value_type: Option<&Type>) -> Option<String> {
    match key {
        // Text content
        "text" => Some(value.to_string()),

        // Style properties
        "width" | "height" | "fontSize" | "margin" | "padding" | "borderRadius"
        | "backgroundColor" | "opacity" | "borderWidth" | "borderColor" => {
            style_to_modifier(key, value)
        }

        // Event handlers
        "onclick" => Some(format!(".onClick(() => {{\n    {}\n  }})", value)),

        _ => None,
    }
}

/// Convert AURA class to ArkTS style modifier
pub fn class_to_modifier(class_name: &str) -> Option<String> {
    // Map common Tailwind-like classes to ArkTS modifiers
    match class_name {
        // Flex
        "flex-1" => Some(".layoutWeight(1)".to_string()),
        "flex-2" => Some(".layoutWeight(2)".to_string()),

        // Padding
        cls if cls.starts_with("p-") => {
            let size = cls.strip_prefix("p-").unwrap();
            Some(format!(".padding({})", size))
        }
        cls if cls.starts_with("px-") => {
            let size = cls.strip_prefix("px-").unwrap();
            Some(format!(".padding({{ left: {}, right: {} }})", size, size))
        }
        cls if cls.starts_with("py-") => {
            let size = cls.strip_prefix("py-").unwrap();
            Some(format!(".padding({{ top: {}, bottom: {} }})", size, size))
        }

        // Margin
        cls if cls.starts_with("m-") => {
            let size = cls.strip_prefix("m-").unwrap();
            Some(format!(".margin({})", size))
        }

        // Width/Height
        cls if cls.starts_with("w-") => {
            let size = cls.strip_prefix("w-").unwrap();
            Some(format!(".width('{}')", size))
        }
        cls if cls.starts_with("h-") => {
            let size = cls.strip_prefix("h-").unwrap();
            Some(format!(".height('{}')", size))
        }

        // Text
        "text-center" => Some(".textAlign(TextAlign.Center)".to_string()),
        "text-left" => Some(".textAlign(TextAlign.Start)".to_string()),
        "text-right" => Some(".textAlign(TextAlign.End)".to_string()),
        "font-bold" => Some(".fontWeight(FontWeight.Bold)".to_string()),
        "font-normal" => Some(".fontWeight(FontWeight.Normal)".to_string()),

        // Rounded
        "rounded" => Some(".borderRadius(4)".to_string()),
        "rounded-lg" => Some(".borderRadius(8)".to_string()),
        "rounded-full" => Some(".borderRadius(9999)".to_string()),

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_width_modifier() {
        let result = style_to_modifier("width", "100%");
        assert_eq!(result, Some(".width('100%')".to_string()));
    }

    #[test]
    fn test_font_size_modifier() {
        let result = style_to_modifier("fontSize", "16");
        assert_eq!(result, Some(".fontSize(16)".to_string()));
    }

    #[test]
    fn test_unknown_modifier_returns_none() {
        let result = style_to_modifier("unknown", "value");
        assert!(result.is_none());
    }

    #[test]
    fn test_class_to_modifier() {
        assert_eq!(class_to_modifier("p-4"), Some(".padding(4)".to_string()));
        assert_eq!(class_to_modifier("w-full"), Some(".width('full')".to_string()));
        assert_eq!(class_to_modifier("font-bold"), Some(".fontWeight(FontWeight.Bold)".to_string()));
    }
}
