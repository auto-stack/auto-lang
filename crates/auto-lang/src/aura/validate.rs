//! AURA Widget Validation
//!
//! This module validates AURA widgets against the schema,
//! providing helpful error messages for incorrect code.

use crate::ast::ViewNode;
use crate::ast::WidgetDecl;
use crate::aura::schema::AuraSchema;
use miette::{Diagnostic, SourceSpan};
use std::collections::HashSet;
use thiserror::Error;

/// Validation error types
#[derive(Debug, Clone, Error, Diagnostic)]
pub enum ValidationError {
    /// E0981: Widget missing required block
    #[error("widget missing required block: {block}")]
    #[diagnostic(code(aura::E0981), help("widget must have a '{block}' block"))]
    MissingRequiredBlock {
        block: String,
        #[label("widget definition")]
        span: SourceSpan,
    },

    /// E0982: Duplicate block in widget
    #[error("duplicate '{block}' block in widget")]
    #[diagnostic(code(aura::E0982), help("widget can have at most one '{block}' block"))]
    DuplicateBlock {
        block: String,
        #[label("duplicate block")]
        span: SourceSpan,
    },

    /// E0983: Unknown view element
    #[error("unknown view element '{tag}'")]
    #[diagnostic(code(aura::E0983))]
    UnknownElement {
        tag: String,
        #[label("unknown element")]
        span: SourceSpan,
        suggestion: Option<String>,
    },

    /// E0984: Invalid prop for element
    #[error("invalid prop '{prop}' for element '{tag}'")]
    #[diagnostic(code(aura::E0984))]
    InvalidProp {
        tag: String,
        prop: String,
        #[label("invalid prop")]
        span: SourceSpan,
        suggestion: Option<String>,
        valid_props: Vec<String>,
    },

    /// E0985: Missing required prop
    #[error("missing required prop '{prop}' for element '{tag}'")]
    #[diagnostic(code(aura::E0985), help("add '{prop}' prop to the '{tag}' element"))]
    MissingRequiredProp {
        tag: String,
        prop: String,
        #[label("element missing required prop")]
        span: SourceSpan,
    },

    /// E0986: Element cannot have children
    #[error("element '{tag}' cannot have children")]
    #[diagnostic(code(aura::E0986), help("'{tag}' is a leaf element, remove the children"))]
    InvalidChildren {
        tag: String,
        #[label("element with invalid children")]
        span: SourceSpan,
    },

    /// Multiple validation errors
    #[error("multiple validation errors")]
    MultipleErrors {
        errors: Vec<ValidationError>,
    },
}

/// Widget validator using AURA schema
pub struct WidgetValidator {
    schema: AuraSchema,
}

impl WidgetValidator {
    /// Create a new validator with the default schema
    pub fn new() -> Result<Self, ValidationError> {
        let schema = crate::aura::load_default_schema()
            .map_err(|e| ValidationError::MultipleErrors {
                errors: vec![ValidationError::UnknownElement {
                    tag: format!("schema load error: {}", e),
                    span: SourceSpan::from(0..0),
                    suggestion: None,
                }],
            })?;
        Ok(Self { schema })
    }

    /// Create a validator with a custom schema
    pub fn with_schema(schema: AuraSchema) -> Self {
        Self { schema }
    }

    /// Validate a widget declaration
    pub fn validate_widget(&self, widget: &WidgetDecl) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Validate widget blocks (msg, model, view required)
        if let Some(err) = self.validate_widget_blocks(widget) {
            errors.extend(err);
        }

        // Validate view tree if present
        if let Some(view_block) = &widget.view {
            self.validate_view_tree(&view_block.root, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate that widget has required blocks
    fn validate_widget_blocks(&self, widget: &WidgetDecl) -> Option<Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Check required blocks
        for block in &self.schema.widget_blocks.required {
            let has_block = match *block {
                "msg" => !widget.messages.is_empty(),
                "model" => widget.model.is_some(),
                "computed" => widget.computed.is_some(),
                "view" => widget.view.is_some(),
                "on" => widget.on.is_some(),
                _ => false,
            };

            if !has_block {
                errors.push(ValidationError::MissingRequiredBlock {
                    block: block.to_string(),
                    span: SourceSpan::from(0..0), // Would need position info
                });
            }
        }

        if errors.is_empty() {
            None
        } else {
            Some(errors)
        }
    }

    /// Validate a view tree recursively
    fn validate_view_tree(&self, node: &ViewNode, errors: &mut Vec<ValidationError>) {
        match node {
            ViewNode::Element { tag, props, children, .. } => {
                // Check if element is known
                if let Some(element_def) = self.schema.get_element(tag) {
                    // Validate props
                    self.validate_element_props(tag, props, element_def.props.as_slice(), errors);

                    // Validate children
                    if !element_def.allows_children && !children.is_empty() {
                        errors.push(ValidationError::InvalidChildren {
                            tag: tag.clone(),
                            span: SourceSpan::from(0..0),
                        });
                    }

                    // Recursively validate children
                    for child in children {
                        self.validate_view_tree(child, errors);
                    }
                } else {
                    // Unknown element - provide suggestion
                    let suggestion = self.schema.suggest_similar(tag);
                    errors.push(ValidationError::UnknownElement {
                        tag: tag.clone(),
                        span: SourceSpan::from(0..0),
                        suggestion: suggestion.map(|s| s.to_string()),
                    });

                    // Still validate children for unknown elements
                    for child in children {
                        self.validate_view_tree(child, errors);
                    }
                }
            }
            ViewNode::Conditional { then_body, else_body, .. } => {
                for child in then_body {
                    self.validate_view_tree(child, errors);
                }
                if let Some(else_nodes) = else_body {
                    for child in else_nodes {
                        self.validate_view_tree(child, errors);
                    }
                }
            }
            ViewNode::ForLoop { body, .. } => {
                for child in body {
                    self.validate_view_tree(child, errors);
                }
            }
            ViewNode::Text(_) => {
                // Text nodes don't need validation
            }
            ViewNode::Component { name, props, .. } => {
                // User-defined components - no children to validate
                // Note: Component validation could be extended to check against component schemas
                let _ = (name, props); // Suppress unused warning
            }
            ViewNode::Outlet => {
                // Router outlet - no children or props to validate
            }
            ViewNode::Link { to, text, href, children } => {
                // Navigation link - validate that 'to' is valid
                let _ = (to, text, href); // Suppress unused warning
                for child in children {
                    self.validate_view_tree(child, errors);
                }
            }
        }
    }

    /// Validate element props against schema
    fn validate_element_props(
        &self,
        tag: &str,
        props: &[crate::ast::ViewProp],
        schema_props: &[crate::aura::schema::PropDef],
        errors: &mut Vec<ValidationError>,
    ) {
        let valid_prop_names: HashSet<&str> = schema_props.iter().map(|p| p.name).collect();
        let provided_props: HashSet<&str> = props.iter().map(|p| p.name.as_str()).collect();

        // Check for invalid props
        for prop in props {
            if !valid_prop_names.contains(prop.name.as_str()) {
                // Try to find a similar prop name
                let suggestion = self.find_similar_prop(&prop.name, schema_props);

                errors.push(ValidationError::InvalidProp {
                    tag: tag.to_string(),
                    prop: prop.name.clone(),
                    span: SourceSpan::from(0..0),
                    suggestion,
                    valid_props: schema_props.iter().map(|p| p.name.to_string()).collect(),
                });
            }
        }

        // Check for missing required props
        for schema_prop in schema_props {
            if schema_prop.required && !provided_props.contains(schema_prop.name) {
                errors.push(ValidationError::MissingRequiredProp {
                    tag: tag.to_string(),
                    prop: schema_prop.name.to_string(),
                    span: SourceSpan::from(0..0),
                });
            }
        }
    }

    /// Find a similar prop name using Levenshtein distance
    fn find_similar_prop(
        &self,
        prop_name: &str,
        schema_props: &[crate::aura::schema::PropDef],
    ) -> Option<String> {
        let mut best_match: Option<&str> = None;
        let mut best_score = usize::MAX;

        for schema_prop in schema_props {
            let score = Self::levenshtein_distance(prop_name, schema_prop.name);
            if score < best_score && score <= 3 {
                best_score = score;
                best_match = Some(schema_prop.name);
            }
        }

        best_match.map(|s| s.to_string())
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for (i, row) in matrix.iter_mut().enumerate() {
            row[0] = i;
        }

        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for (i, a_char) in a_chars.iter().enumerate() {
            for (j, b_char) in b_chars.iter().enumerate() {
                let cost = if a_char == b_char { 0 } else { 1 };
                matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                    .min(matrix[i + 1][j] + 1)
                    .min(matrix[i][j] + cost);
            }
        }

        matrix[a_len][b_len]
    }

    /// Get the schema reference
    pub fn schema(&self) -> &AuraSchema {
        &self.schema
    }

    /// Validate a single view node
    pub fn validate_element(&self, tag: &str) -> ElementValidation {
        ElementValidation {
            tag: tag.to_string(),
            element_def: self.schema.get_element(tag).cloned(),
            schema: &self.schema,
        }
    }
}

impl Default for WidgetValidator {
    fn default() -> Self {
        Self::new().expect("Failed to create default validator")
    }
}

/// Validation result for a single element
pub struct ElementValidation<'a> {
    tag: String,
    element_def: Option<crate::aura::schema::ElementDef>,
    schema: &'a AuraSchema,
}

impl<'a> ElementValidation<'a> {
    /// Check if element is valid
    pub fn is_valid(&self) -> bool {
        self.element_def.is_some()
    }

    /// Get suggestion for unknown element
    pub fn suggestion(&self) -> Option<&'static str> {
        if self.element_def.is_none() {
            self.schema.suggest_similar(&self.tag)
        } else {
            None
        }
    }

    /// Get valid props for this element
    pub fn valid_props(&self) -> Vec<&'static str> {
        if let Some(def) = &self.element_def {
            def.props.iter().map(|p| p.name).collect()
        } else {
            vec![]
        }
    }

    /// Check if element allows children
    pub fn allows_children(&self) -> bool {
        self.element_def.as_ref().map(|d| d.allows_children).unwrap_or(false)
    }
}

/// Format validation errors into a user-friendly message
pub fn format_validation_errors(errors: &[ValidationError]) -> String {
    let mut output = String::new();

    for error in errors {
        output.push_str(&format!("{}\n", error));

        // Add suggestion if available
        match error {
            ValidationError::UnknownElement { tag, suggestion, .. } => {
                if let Some(s) = suggestion {
                    output.push_str(&format!("  = help: did you mean '{}'?\n", s));
                }
                output.push_str(&format!("  = help: available elements: col, row, button, text, input, ...\n"));
            }
            ValidationError::InvalidProp { tag, suggestion, valid_props, .. } => {
                if let Some(s) = suggestion {
                    output.push_str(&format!("  = help: did you mean '{}'?\n", s));
                }
                output.push_str(&format!("  = help: valid props for '{}': {}\n", tag, valid_props.join(", ")));
            }
            _ => {}
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = WidgetValidator::new();
        assert!(validator.is_ok());
    }

    #[test]
    fn test_valid_element() {
        let validator = WidgetValidator::new().unwrap();
        let validation = validator.validate_element("button");

        assert!(validation.is_valid());
        assert!(validation.valid_props().contains(&"text"));
        assert!(validation.valid_props().contains(&"onclick"));
        assert!(!validation.allows_children());
    }

    #[test]
    fn test_unknown_element_suggestion() {
        let validator = WidgetValidator::new().unwrap();
        let validation = validator.validate_element("buton");

        assert!(!validation.is_valid());
        assert_eq!(validation.suggestion(), Some("button"));
    }

    #[test]
    fn test_valid_element_with_children() {
        let validator = WidgetValidator::new().unwrap();
        let validation = validator.validate_element("col");

        assert!(validation.is_valid());
        assert!(validation.allows_children());
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(WidgetValidator::levenshtein_distance("button", "button"), 0);
        assert_eq!(WidgetValidator::levenshtein_distance("buton", "button"), 1);  // insert 't'
        assert_eq!(WidgetValidator::levenshtein_distance("buttn", "button"), 1);  // insert 'o'
        assert_eq!(WidgetValidator::levenshtein_distance("col", "row"), 2);  // substitute c→r, l→w
    }

    #[test]
    fn test_schema_validation() {
        let validator = WidgetValidator::new().unwrap();
        let schema = validator.schema();

        // Check widget blocks
        assert!(schema.widget_blocks.is_required("msg"));
        assert!(schema.widget_blocks.is_required("model"));
        assert!(schema.widget_blocks.is_required("view"));
        assert!(!schema.widget_blocks.is_required("computed"));
        assert!(!schema.widget_blocks.is_required("on"));
    }

    #[test]
    fn test_format_errors() {
        let errors = vec![
            ValidationError::UnknownElement {
                tag: "buton".to_string(),
                span: SourceSpan::from(0..0),
                suggestion: Some("button".to_string()),
            },
        ];

        let output = format_validation_errors(&errors);
        assert!(output.contains("unknown view element 'buton'"));
        assert!(output.contains("did you mean 'button'"));
    }
}
