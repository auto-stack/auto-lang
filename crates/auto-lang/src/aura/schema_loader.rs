//! AURA Schema Loader - Parses schema/aura.at to build in-memory schema
//!
//! This module loads the AutoLang-based schema file and converts it to
//! the Rust schema structures used for validation.

use crate::aura::schema::{AuraSchema, ElementCategory, ElementDef, PropDef, PropType, WidgetBlockSchema};
use std::collections::HashMap;

/// Error type for schema loading
#[derive(Debug)]
pub enum SchemaLoadError {
    IoError(std::io::Error),
    ParseError { line: usize, message: String },
    InvalidSyntax { message: String },
}

impl std::fmt::Display for SchemaLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SchemaLoadError::IoError(e) => write!(f, "IO error: {}", e),
            SchemaLoadError::ParseError { line, message } => {
                write!(f, "Parse error at line {}: {}", line, message)
            }
            SchemaLoadError::InvalidSyntax { message } => write!(f, "Invalid syntax: {}", message),
        }
    }
}

impl std::error::Error for SchemaLoadError {}

/// Schema loader that parses AutoLang-based schema files
pub struct SchemaLoader {
    /// Constants defined in the schema file
    constants: HashMap<String, String>,
    /// Element definitions
    elements: Vec<ElementDefData>,
    /// Widget block schema
    widget_blocks: Option<WidgetBlockData>,
}

/// Raw element data from parsing
#[derive(Debug, Clone)]
struct ElementDefData {
    tag: String,
    category: String,
    props: Vec<PropDefData>,
    allows_children: bool,
    description: String,
}

/// Raw prop data from parsing
#[derive(Debug, Clone)]
struct PropDefData {
    name: String,
    type_str: String,
    required: bool,
    default: Option<String>,
    description: String,
}

/// Raw widget block data
#[derive(Debug, Clone)]
struct WidgetBlockData {
    required: Vec<String>,
    optional: Vec<String>,
}

impl SchemaLoader {
    /// Create a new schema loader
    pub fn new() -> Self {
        SchemaLoader {
            constants: HashMap::new(),
            elements: Vec::new(),
            widget_blocks: None,
        }
    }

    /// Load schema from file content
    pub fn load(&mut self, content: &str) -> Result<AuraSchema, SchemaLoadError> {
        // Parse the schema file
        self.parse_schema_file(content)?;

        // Convert to AuraSchema
        self.build_schema()
    }

    /// Parse the schema file content
    fn parse_schema_file(&mut self, content: &str) -> Result<(), SchemaLoadError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with("//") {
                i += 1;
                continue;
            }

            // Parse const definitions
            if line.starts_with("const ") {
                self.parse_const(line)?;
                i += 1;
                continue;
            }

            // Parse element definitions
            if line.starts_with("element ") {
                let end_line = self.parse_element(&lines, i)?;
                i = end_line + 1;
                continue;
            }

            // Parse widget_blocks definition
            if line.starts_with("widget_blocks ") || line == "widget_blocks {" {
                let end_line = self.parse_widget_blocks(&lines, i)?;
                i = end_line + 1;
                continue;
            }

            // Skip type definitions and other content
            if line.starts_with("type ") || line.starts_with("schema ") {
                // Skip until we find the closing brace
                let brace_count = line.matches('{').count() - line.matches('}').count();
                if brace_count == 0 {
                    i += 1;
                    continue;
                }
                i += 1;
                let mut depth = brace_count;
                while i < lines.len() && depth > 0 {
                    depth += lines[i].matches('{').count();
                    depth -= lines[i].matches('}').count();
                    i += 1;
                }
                continue;
            }

            i += 1;
        }

        Ok(())
    }

    /// Parse a const definition: const NAME: str = "value"
    fn parse_const(&mut self, line: &str) -> Result<(), SchemaLoadError> {
        // Remove "const " prefix
        let rest = line.strip_prefix("const ").unwrap_or(line);

        // Find the colon for type annotation
        let colon_pos = rest.find(':').ok_or_else(|| SchemaLoadError::ParseError {
            line: 0,
            message: format!("Missing ':' in const definition: {}", line),
        })?;

        let name = rest[..colon_pos].trim().to_string();
        let after_colon = &rest[colon_pos + 1..];

        // Find the equals sign
        let eq_pos = after_colon.find('=').ok_or_else(|| SchemaLoadError::ParseError {
            line: 0,
            message: format!("Missing '=' in const definition: {}", line),
        })?;

        // Extract value (handle quoted strings)
        let value_part = after_colon[eq_pos + 1..].trim();
        let value = if value_part.starts_with('"') && value_part.ends_with('"') {
            value_part[1..value_part.len()-1].to_string()
        } else {
            value_part.to_string()
        };

        self.constants.insert(name, value);
        Ok(())
    }

    /// Parse an element definition
    fn parse_element(&mut self, lines: &[&str], start: usize) -> Result<usize, SchemaLoadError> {
        let first_line = lines[start].trim();
        let element_name = first_line
            .strip_prefix("element ")
            .and_then(|s| s.strip_suffix('{'))
            .map(|s| s.trim())
            .ok_or_else(|| SchemaLoadError::ParseError {
                line: start + 1,
                message: format!("Invalid element definition: {}", first_line),
            })?;

        // Collect lines until we find the closing brace
        let mut content = String::new();
        let mut i = start + 1;
        let mut depth = 1;

        while i < lines.len() && depth > 0 {
            let line = lines[i];
            depth += line.matches('{').count() as i32;
            depth -= line.matches('}').count() as i32;

            if depth > 0 {
                content.push_str(line);
                content.push('\n');
            }
            i += 1;
        }

        // Parse the element content
        let element_data = self.parse_element_content(element_name, &content)?;

        self.elements.push(element_data);

        Ok(i - 1)
    }

    /// Parse element content (key: value pairs)
    fn parse_element_content(&self, _name: &str, content: &str) -> Result<ElementDefData, SchemaLoadError> {
        let mut tag = String::new();
        let mut category = String::new();
        let mut props: Vec<PropDefData> = Vec::new();
        let mut allows_children = true;
        let mut description = String::new();

        // Simple key-value parsing
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            if let Some(value) = self.extract_string_value(line, "tag:") {
                tag = value;
            } else if let Some(value) = self.extract_string_value(line, "category:") {
                category = value;
            } else if let Some(value) = self.extract_bool_value(line, "allows_children:") {
                allows_children = value;
            } else if let Some(value) = self.extract_string_value(line, "description:") {
                description = value;
            } else if line.starts_with("props:") {
                // Parse props array
                props = self.parse_props_array(content)?;
            }
        }

        Ok(ElementDefData {
            tag,
            category,
            props,
            allows_children,
            description,
        })
    }

    /// Parse props array from element content
    fn parse_props_array(&self, content: &str) -> Result<Vec<PropDefData>, SchemaLoadError> {
        let mut props = Vec::new();

        // Find props: [ ... ]
        let props_start = content.find("props:").ok_or_else(|| SchemaLoadError::InvalidSyntax {
            message: "props not found".to_string(),
        })?;

        let bracket_start = content[props_start..].find('[').ok_or_else(|| SchemaLoadError::InvalidSyntax {
            message: "props array not found".to_string(),
        })?;

        let start_pos = props_start + bracket_start + 1;

        // Find matching closing bracket
        let mut depth = 1;
        let mut end_pos = start_pos;
        let chars: Vec<char> = content.chars().collect();

        while end_pos < chars.len() && depth > 0 {
            match chars[end_pos] {
                '[' => depth += 1,
                ']' => depth -= 1,
                _ => {}
            }
            end_pos += 1;
        }

        let props_content: String = chars[start_pos..end_pos-1].iter().collect();

        // Parse individual prop objects: { name: "...", type: "...", ... }
        let mut current_obj = String::new();
        let mut brace_depth = 0;

        for ch in props_content.chars() {
            match ch {
                '{' => {
                    brace_depth += 1;
                    if brace_depth == 1 {
                        current_obj.clear();
                    } else {
                        current_obj.push(ch);
                    }
                }
                '}' => {
                    brace_depth -= 1;
                    if brace_depth == 0 {
                        // Parse the prop object
                        if let Ok(prop) = self.parse_prop_object(&current_obj) {
                            props.push(prop);
                        }
                        current_obj.clear();
                    } else {
                        current_obj.push(ch);
                    }
                }
                _ if brace_depth > 0 => {
                    current_obj.push(ch);
                }
                _ => {}
            }
        }

        Ok(props)
    }

    /// Parse a single prop object
    fn parse_prop_object(&self, content: &str) -> Result<PropDefData, SchemaLoadError> {
        let mut name = String::new();
        let mut type_str = String::new();
        let mut required = false;
        let mut default = None;
        let mut description = String::new();

        // Split by comma, but be careful of nested structures
        let parts = self.split_by_comma(content);

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some(value) = self.extract_string_value(part, "name:") {
                name = value;
            } else if let Some(value) = self.extract_value(part, "type:") {
                // Resolve constant reference if needed
                type_str = self.resolve_type(&value);
            } else if let Some(value) = self.extract_bool_value(part, "required:") {
                required = value;
            } else if let Some(value) = self.extract_string_value(part, "default:") {
                default = Some(value);
            } else if let Some(value) = self.extract_string_value(part, "description:") {
                description = value;
            }
        }

        Ok(PropDefData {
            name,
            type_str,
            required,
            default,
            description,
        })
    }

    /// Parse widget_blocks definition
    fn parse_widget_blocks(&mut self, lines: &[&str], start: usize) -> Result<usize, SchemaLoadError> {
        // Collect content until closing brace
        let mut content = String::new();
        let mut i = start;
        let mut depth = 0;
        let mut started = false;

        while i < lines.len() {
            let line = lines[i].trim();

            if line.contains('{') {
                if !started {
                    started = true;
                    depth = 1;
                    // Extract content after the opening brace
                    if let Some(pos) = line.find('{') {
                        content.push_str(&line[pos + 1..]);
                        content.push('\n');
                    }
                } else {
                    depth += line.matches('{').count();
                    depth -= line.matches('}').count();
                    content.push_str(line);
                    content.push('\n');
                }
            } else if started {
                depth += line.matches('{').count();
                depth -= line.matches('}').count();
                if depth <= 0 {
                    break;
                }
                content.push_str(line);
                content.push('\n');
            }

            i += 1;
        }

        // Parse required and optional arrays
        let required = self.extract_string_array(&content, "required:")
            .unwrap_or_default();
        let optional = self.extract_string_array(&content, "optional:")
            .unwrap_or_default();

        self.widget_blocks = Some(WidgetBlockData { required, optional });

        Ok(i)
    }

    /// Extract a string value from a key: "value" pattern
    fn extract_string_value(&self, text: &str, key: &str) -> Option<String> {
        if let Some(pos) = text.find(key) {
            let after_key = &text[pos + key.len()..];
            let trimmed = after_key.trim();

            // Handle quoted string
            if trimmed.starts_with('"') {
                if let Some(end) = trimmed[1..].find('"') {
                    return Some(trimmed[1..end+1].to_string());
                }
            }
        }
        None
    }

    /// Extract any value from a key: value pattern
    fn extract_value(&self, text: &str, key: &str) -> Option<String> {
        if let Some(pos) = text.find(key) {
            let after_key = &text[pos + key.len()..];
            let trimmed = after_key.trim();

            // Find the end of the value (comma or end of line)
            let end = trimmed.find(',').unwrap_or(trimmed.len());
            return Some(trimmed[..end].trim().to_string());
        }
        None
    }

    /// Extract a bool value from a key: value pattern
    fn extract_bool_value(&self, text: &str, key: &str) -> Option<bool> {
        if let Some(pos) = text.find(key) {
            let after_key = &text[pos + key.len()..];
            let trimmed = after_key.trim();

            if trimmed.starts_with("true") {
                return Some(true);
            } else if trimmed.starts_with("false") {
                return Some(false);
            }
        }
        None
    }

    /// Extract a string array from a key: ["a", "b"] pattern
    fn extract_string_array(&self, content: &str, key: &str) -> Option<Vec<String>> {
        if let Some(pos) = content.find(key) {
            let after_key = &content[pos + key.len()..];

            // Find the array brackets
            if let Some(start) = after_key.find('[') {
                let after_bracket = &after_key[start + 1..];
                if let Some(end) = after_bracket.find(']') {
                    let array_content = &after_bracket[..end];

                    // Parse quoted strings
                    let mut result = Vec::new();
                    let mut in_string = false;
                    let mut current = String::new();

                    for ch in array_content.chars() {
                        match ch {
                            '"' => {
                                if in_string {
                                    if !current.is_empty() {
                                        result.push(current.clone());
                                        current.clear();
                                    }
                                }
                                in_string = !in_string;
                            }
                            _ if in_string => {
                                current.push(ch);
                            }
                            _ => {}
                        }
                    }

                    return Some(result);
                }
            }
        }
        None
    }

    /// Split content by comma, respecting nested structures
    fn split_by_comma(&self, content: &str) -> Vec<String> {
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in content.chars() {
            match ch {
                '{' | '[' | '(' => {
                    depth += 1;
                    current.push(ch);
                }
                '}' | ']' | ')' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    if !current.trim().is_empty() {
                        parts.push(current.trim().to_string());
                    }
                    current.clear();
                }
                _ => {
                    current.push(ch);
                }
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        parts
    }

    /// Resolve a type reference (might be a constant)
    fn resolve_type(&self, type_str: &str) -> String {
        // If it's a constant reference (not a string literal), look it up
        if !type_str.starts_with('"') {
            if let Some(value) = self.constants.get(type_str) {
                return value.clone();
            }
        }
        // Otherwise return as-is (stripping quotes if present)
        if type_str.starts_with('"') && type_str.ends_with('"') {
            type_str[1..type_str.len()-1].to_string()
        } else {
            type_str.to_string()
        }
    }

    /// Build the final AuraSchema from parsed data
    fn build_schema(&self) -> Result<AuraSchema, SchemaLoadError> {
        let mut elements = HashMap::new();

        for elem_data in &self.elements {
            let category = match elem_data.category.as_str() {
                "layout" => ElementCategory::Layout,
                "content" => ElementCategory::Content,
                "typography" => ElementCategory::Typography,
                "list" => ElementCategory::List,
                "media" => ElementCategory::Media,
                "utility" => ElementCategory::Utility,
                _ => ElementCategory::Content,
            };

            let props: Vec<PropDef> = elem_data.props.iter().map(|p| {
                let prop_type = self.parse_prop_type(&p.type_str);
                PropDef {
                    name: Box::leak(p.name.clone().into_boxed_str()),
                    type_: prop_type,
                    required: p.required,
                    default: p.default.as_ref().map(|d| Box::leak(d.clone().into_boxed_str()) as &'static str),
                    description: Box::leak(p.description.clone().into_boxed_str()),
                }
            }).collect();

            let element_def = ElementDef {
                tag: Box::leak(elem_data.tag.clone().into_boxed_str()),
                category,
                props,
                allows_children: elem_data.allows_children,
                description: Box::leak(elem_data.description.clone().into_boxed_str()),
            };

            elements.insert(element_def.tag, element_def);
        }

        let widget_blocks = if let Some(wb) = &self.widget_blocks {
            WidgetBlockSchema {
                required: wb.required.iter().map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str).collect(),
                optional: wb.optional.iter().map(|s| Box::leak(s.clone().into_boxed_str()) as &'static str).collect(),
            }
        } else {
            // Default widget blocks
            WidgetBlockSchema {
                required: vec!["msg", "model", "view"],
                optional: vec!["computed", "on"],
            }
        };

        Ok(AuraSchema {
            elements,
            widget_blocks,
        })
    }

    /// Parse a prop type string into a PropType
    fn parse_prop_type(&self, type_str: &str) -> PropType {
        if type_str.starts_with("union:") {
            let types: Vec<&str> = type_str[6..].split(',').collect();
            PropType::Union(types.iter().map(|t| match *t {
                "string" => PropType::String,
                "int" => PropType::Int,
                "float" => PropType::Float,
                "bool" => PropType::Bool,
                "state_ref" => PropType::StateRef,
                "msg_ref" => PropType::MsgRef,
                "class_binding" => PropType::ClassBinding,
                _ => PropType::String,
            }).collect())
        } else if type_str.starts_with("one_of:") {
            let options: Vec<&str> = type_str[7..].split(',').collect();
            // Leak strings to make them 'static
            PropType::OneOf(options.iter().map(|s| Box::leak(s.to_string().into_boxed_str()) as &'static str).collect())
        } else {
            match type_str {
                "string" => PropType::String,
                "int" => PropType::Int,
                "float" => PropType::Float,
                "bool" => PropType::Bool,
                "color" => PropType::Color,
                "state_ref" => PropType::StateRef,
                "msg_ref" => PropType::MsgRef,
                "expr" => PropType::Expr,
                "closure" => PropType::Closure,
                "class_binding" => PropType::ClassBinding,
                "interpolated" => PropType::Interpolated,
                _ => PropType::String,
            }
        }
    }
}

impl Default for SchemaLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Load the default AURA schema from the embedded schema file
pub fn load_default_schema() -> Result<AuraSchema, SchemaLoadError> {
    let mut loader = SchemaLoader::new();
    loader.load(include_str!("../../../../schema/aura.at"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_schema() {
        let schema = load_default_schema().expect("Failed to load schema");

        // Check that we have elements
        assert!(schema.get_element("button").is_some());
        assert!(schema.get_element("col").is_some());
        assert!(schema.get_element("input").is_some());

        // Check button element
        let button = schema.get_element("button").unwrap();
        assert_eq!(button.tag, "button");
        assert!(!button.allows_children);
        assert!(button.get_prop("text").is_some());
        assert!(button.get_prop("onclick").is_some());

        // Check col element
        let col = schema.get_element("col").unwrap();
        assert_eq!(col.tag, "col");
        assert!(col.allows_children);

        // Check widget blocks
        assert!(schema.widget_blocks.is_required("msg"));
        assert!(schema.widget_blocks.is_required("model"));
        assert!(schema.widget_blocks.is_required("view"));
        assert!(!schema.widget_blocks.is_required("computed"));
    }

    #[test]
    fn test_schema_from_content() {
        let content = r#"
const TEST_TYPE: str = "test_type"

element test_elem {
    tag: "test"
    category: "content"
    props: [
        { name: "value", type: "string", description: "Test value" }
    ]
    allows_children: false
    description: "Test element"
}

widget_blocks {
    required: ["msg", "model"]
    optional: ["on"]
}
"#;

        let mut loader = SchemaLoader::new();
        let schema = loader.load(content).expect("Failed to load schema");

        assert!(schema.get_element("test").is_some());
        let elem = schema.get_element("test").unwrap();
        assert_eq!(elem.props.len(), 1);
        assert_eq!(elem.props[0].name, "value");
    }

    #[test]
    fn test_parse_const() {
        let mut loader = SchemaLoader::new();
        loader.parse_const(r#"const FOO: str = "bar""#).unwrap();
        assert_eq!(loader.constants.get("FOO"), Some(&"bar".to_string()));
    }

    #[test]
    fn test_resolve_type() {
        let mut loader = SchemaLoader::new();
        loader.constants.insert("MSG_REF_TYPE".to_string(), "msg_ref".to_string());

        assert_eq!(loader.resolve_type("MSG_REF_TYPE"), "msg_ref");
        assert_eq!(loader.resolve_type(r#""string""#), "string");
    }
}
