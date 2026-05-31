//! AURA → A2UI Exporter
//!
//! Converts AutoUI's AURA intermediate representation into Google's A2UI v0.8 JSON.

use super::{
    A2UIAction, A2UIComponent, A2UIComponentBody, A2UIContextBinding, A2UIMessage, A2UISurfaceUpdate,
    A2UIValue, A2UIError,
};
use crate::aura::{
    AuraEvent, AuraExpr, AuraNode, AuraPropValue, AuraTextContent, AuraWidget,
};
use std::collections::HashMap;

/// Export an AuraWidget to an A2UI SurfaceUpdate message.
///
/// # Example
/// ```rust,ignore
/// let widget = extract_widget(...);
/// let a2ui = export_widget(&widget)?;
/// let json = serde_json::to_string_pretty(&a2ui)?;
/// ```
pub fn export_widget(widget: &AuraWidget) -> Result<A2UIMessage, A2UIError> {
    let components = export_node_children(&widget.view_tree, &mut IdGen::new(&widget.name))?;

    Ok(A2UIMessage::SurfaceUpdate(
        A2UISurfaceUpdate {
            surface_id: widget.name.clone(),
            components,
        },
    ))
}

// ============================================================================
// ID Generation
// ============================================================================

/// Generates unique component IDs based on a prefix and index path.
struct IdGen {
    prefix: String,
    counter: HashMap<String, usize>,
}

impl IdGen {
    fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_lowercase().replace(' ', "_"),
            counter: HashMap::new(),
        }
    }

    fn next(&mut self, tag: &str, path: &[usize]) -> String {
        let path_str = if path.is_empty() {
            "root".to_string()
        } else {
            path.iter().map(|i| i.to_string()).collect::<Vec<_>>().join("_")
        };
        let key = format!("{}_{}", tag, path_str);
        let count = self.counter.entry(key.clone()).or_insert(0);
        *count += 1;
        if *count > 1 {
            format!("{}_{}_{}", self.prefix, tag, path_str)
        } else {
            format!("{}_{}", tag, path_str)
        }
    }
}

// ============================================================================
// Node Export
// ============================================================================

fn export_node_children(
    node: &AuraNode,
    id_gen: &mut IdGen,
) -> Result<Vec<A2UIComponent>, A2UIError> {
    match node {
        AuraNode::Element { tag, children, .. } if is_layout_wrapper(tag) => {
            // Some AURA patterns wrap content in layout nodes; flatten if appropriate
            let mut result = Vec::new();
            for (i, child) in children.iter().enumerate() {
                let comps = export_node(child, id_gen, &[i])?;
                result.extend(comps);
            }
            Ok(result)
        }
        _ => export_node(node, id_gen, &[]),
    }
}

/// Check if a tag is purely a layout wrapper that should be flattened during export.
fn is_layout_wrapper(tag: &str) -> bool {
    tag == "center"
}

fn export_node(
    node: &AuraNode,
    id_gen: &mut IdGen,
    path: &[usize],
) -> Result<Vec<A2UIComponent>, A2UIError> {
    match node {
        AuraNode::Element {
            tag,
            props,
            events,
            children,
            ..
        } => {
            let id = id_gen.next(tag, path);
            let body = export_element(tag, props, events, children, id_gen, path)?;
            Ok(vec![A2UIComponent::new(id, body)])
        }
        AuraNode::Text(content) => {
            let id = id_gen.next("text", path);
            let value = export_text_content(content)?;
            Ok(vec![A2UIComponent::new(
                id,
                A2UIComponentBody::Text { text: value },
            )])
        }
        AuraNode::ForLoop {
            var: _,
            index: _,
            iterable,
            body,
            ..
        } => {
            let id = id_gen.next("list", path);
            let items_value = A2UIValue::path(format!("/{}", iterable.trim_start_matches('.')));
            // Export template from the first child of the loop body (if any)
            let template = if body.len() == 1 {
                let comps = export_node(&body[0], id_gen, path)?;
                if comps.len() == 1 {
                    Some(Box::new(comps.into_iter().next().unwrap().body))
                } else {
                    None
                }
            } else {
                None
            };
            Ok(vec![A2UIComponent::new(
                id,
                A2UIComponentBody::List {
                    items: items_value,
                    template,
                },
            )])
        }
        AuraNode::Conditional {
            condition: _,
            then_body,
            else_body: _,
            ..
        } => {
            // A2UI does not have a native conditional component.
            // Export the then_body only, with a warning logged implicitly.
            let mut result = Vec::new();
            for (i, child) in then_body.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(i);
                let comps = export_node(child, id_gen, &child_path)?;
                result.extend(comps);
            }
            Ok(result)
        }
        AuraNode::Component { name, props, events, .. } => {
            let id = id_gen.next(&name.to_lowercase(), path);
            let body = export_component(name, props, events)?;
            Ok(vec![A2UIComponent::new(id, body)])
        }
        AuraNode::Outlet => {
            // Outlet has no A2UI equivalent; skip
            Ok(vec![])
        }
        AuraNode::Link {
            to: _,
            text: _,
            href: _,
            children,
            ..
        } => {
            // Map Link to a Navigation component if it has children, otherwise skip
            if children.is_empty() {
                Ok(vec![])
            } else {
                let mut result = Vec::new();
                for (i, child) in children.iter().enumerate() {
                    let mut child_path = path.to_vec();
                    child_path.push(i);
                    let comps = export_node(child, id_gen, &child_path)?;
                    result.extend(comps);
                }
                Ok(result)
            }
        }
    }
}

fn export_element(
    tag: &str,
    props: &HashMap<String, AuraPropValue>,
    events: &HashMap<String, AuraEvent>,
    children: &[AuraNode],
    id_gen: &mut IdGen,
    path: &[usize],
) -> Result<A2UIComponentBody, A2UIError> {
    // Extract common props
    let get_prop = |key: &str| -> Option<A2UIValue> {
        props.get(key).and_then(|pv| match pv {
            AuraPropValue::Expr(expr) => export_expr(expr).ok(),
            _ => None,
        })
    };

    let get_event = |key: &str| -> Option<A2UIAction> {
        events.get(key).map(|ev| {
            let mut action = A2UIAction::new(&ev.handler);
            if !ev.params.is_empty() {
                let context = ev
                    .params
                    .iter()
                    .map(|p| A2UIContextBinding {
                        name: p.clone(),
                        path: format!("/ {}", p),
                    })
                    .collect();
                action = action.with_context(context);
            }
            action
        })
    };

    let mut export_children = || -> Result<Vec<A2UIComponent>, A2UIError> {
        let mut result = Vec::new();
        for (i, child) in children.iter().enumerate() {
            let mut child_path = path.to_vec();
            child_path.push(i);
            let comps = export_node(child, id_gen, &child_path)?;
            result.extend(comps);
        }
        Ok(result)
    };

    match tag {
        // Layout
        "col" | "column" => Ok(A2UIComponentBody::Column {
            children: export_children()?,
        }),
        "row" => Ok(A2UIComponentBody::Row {
            children: export_children()?,
        }),
        "container" | "div" | "stack" | "frame" => Ok(A2UIComponentBody::Container {
            children: export_children()?,
        }),
        "scroll" | "scrollable" | "scroll_view" => {
            let child = if children.len() == 1 {
                let comps = export_node(&children[0], id_gen, path)?;
                if comps.len() == 1 {
                    Some(Box::new(comps.into_iter().next().unwrap().body))
                } else {
                    None
                }
            } else {
                None
            };
            Ok(A2UIComponentBody::ScrollView { child })
        }

        // Form
        "button" | "btn" => Ok(A2UIComponentBody::Button {
            child: get_prop("text").or_else(|| get_prop("label")).unwrap_or_else(|| A2UIValue::string("Button")),
            action: get_event("onclick").or_else(|| get_event("click")).or_else(|| get_event("tap")),
        }),
        "input" | "textinput" | "text_input" => Ok(A2UIComponentBody::TextInput {
            value: get_prop("value").unwrap_or_else(|| A2UIValue::string("")),
            hint: get_prop("placeholder"),
        }),
        "numberinput" | "number_input" => Ok(A2UIComponentBody::NumberInput {
            value: get_prop("value").unwrap_or_else(|| A2UIValue::number(0.0)),
            min: None,
            max: None,
        }),
        "datetimeinput" | "datetime" | "date" => Ok(A2UIComponentBody::DateTimeInput {
            value: get_prop("value").unwrap_or_else(|| A2UIValue::string("")),
        }),
        "checkbox" | "check" => Ok(A2UIComponentBody::Checkbox {
            value: get_prop("checked").or_else(|| get_prop("value")).unwrap_or_else(|| A2UIValue::bool(false)),
            label: get_prop("label"),
        }),
        "radio" => Ok(A2UIComponentBody::Radio {
            value: get_prop("selected").or_else(|| get_prop("value")).unwrap_or_else(|| A2UIValue::bool(false)),
            label: get_prop("label"),
        }),
        "select" | "dropdown" => {
            let options = get_prop("options")
                .and_then(|v| parse_options(&v))
                .unwrap_or_default();
            Ok(A2UIComponentBody::Select {
                value: get_prop("value").unwrap_or_else(|| A2UIValue::string("")),
                options,
            })
        }
        "slider" | "range" => Ok(A2UIComponentBody::Slider {
            value: get_prop("value").unwrap_or_else(|| A2UIValue::number(0.0)),
            min: get_prop("min").and_then(|v| extract_number(&v)),
            max: get_prop("max").and_then(|v| extract_number(&v)),
            step: get_prop("step").and_then(|v| extract_number(&v)),
        }),

        // Display
        "text" | "span" | "label" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let value = get_prop("text")
                .or_else(|| get_prop("content"))
                .unwrap_or_else(|| A2UIValue::string(""));
            Ok(A2UIComponentBody::Text { text: value })
        }
        "image" | "img" => Ok(A2UIComponentBody::Image {
            src: get_prop("src").or_else(|| get_prop("source")).unwrap_or_else(|| A2UIValue::string("")),
        }),
        "icon" => Ok(A2UIComponentBody::Icon {
            name: get_prop("name").or_else(|| get_prop("icon")).unwrap_or_else(|| A2UIValue::string("")),
        }),
        "divider" | "hr" | "separator" => Ok(A2UIComponentBody::Divider {}),
        "spacer" => Ok(A2UIComponentBody::Spacer {}),

        // Data
        "list" => {
            let items = get_prop("items")
                .or_else(|| get_prop("data"))
                .unwrap_or_else(|| A2UIValue::path("/items"));
            Ok(A2UIComponentBody::List { items, template: None })
        }
        "table" => {
            let items = get_prop("items")
                .or_else(|| get_prop("data"))
                .unwrap_or_else(|| A2UIValue::path("/items"));
            let columns = Vec::new(); // TODO: extract columns from props
            Ok(A2UIComponentBody::Table { columns, items })
        }

        // Navigation
        "tabs" => {
            let mut tabs = Vec::new();
            for (i, child) in children.iter().enumerate() {
                if let AuraNode::Element { tag, children: tab_children, .. } = child {
                    if tag == "tab" {
                        let label = A2UIValue::string(format!("Tab {}", i + 1));
                        let tab_body = if tab_children.len() == 1 {
                            let comps = export_node(&tab_children[0], id_gen, path)?;
                            if comps.len() == 1 {
                                Box::new(comps.into_iter().next().unwrap().body)
                            } else {
                                Box::new(A2UIComponentBody::Container { children: comps })
                            }
                        } else {
                            let mut tab_comps = Vec::new();
                            for (j, tc) in tab_children.iter().enumerate() {
                                let mut tp = path.to_vec();
                                tp.push(j);
                                tab_comps.extend(export_node(tc, id_gen, &tp)?);
                            }
                            Box::new(A2UIComponentBody::Container { children: tab_comps })
                        };
                        tabs.push(super::A2UITab { label, child: tab_body });
                    }
                }
            }
            Ok(A2UIComponentBody::Tabs { tabs })
        }

        // Unknown tag
        _ => Err(A2UIError::UnsupportedComponent(tag.to_string())),
    }
}

fn export_component(
    _name: &str,
    _props: &HashMap<String, AuraExpr>,
    _events: &HashMap<String, AuraEvent>,
) -> Result<A2UIComponentBody, A2UIError> {
    // Component instances are not directly representable in A2UI v0.8.
    // Export as a Container with a placeholder.
    Ok(A2UIComponentBody::Container { children: vec![] })
}

// ============================================================================
// Expression Export
// ============================================================================

fn export_expr(expr: &AuraExpr) -> Result<A2UIValue, A2UIError> {
    match expr {
        AuraExpr::Literal(s) => Ok(A2UIValue::string(s.clone())),
        AuraExpr::Int(n) => Ok(A2UIValue::number(*n as f64)),
        AuraExpr::Float(f) => Ok(A2UIValue::number(*f)),
        AuraExpr::Bool(b) => Ok(A2UIValue::bool(*b)),
        AuraExpr::StateRef(name) => Ok(A2UIValue::path(format!("/ {}", name))),
        AuraExpr::MsgVariant { msg_type, variant } => {
            Ok(A2UIValue::string(format!("{}::{}", msg_type, variant)))
        }
        _ => Err(A2UIError::UnsupportedExpression(format!("{:?}", expr))),
    }
}

fn export_text_content(content: &AuraTextContent) -> Result<A2UIValue, A2UIError> {
    match content {
        AuraTextContent::Literal(text) => Ok(A2UIValue::string(text.clone())),
        AuraTextContent::Interpolated { template, .. } => {
            // Interpolated text becomes a literal in A2UI (bindings are lost at this level)
            Ok(A2UIValue::string(template.clone()))
        }
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn extract_number(value: &A2UIValue) -> Option<f64> {
    match value {
        A2UIValue::LiteralNumber { literal_number } => Some(*literal_number),
        _ => None,
    }
}

fn parse_options(_value: &A2UIValue) -> Option<Vec<super::A2UISelectOption>> {
    // For MVP, options are not parsed from expressions.
    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aura::{AuraExpr, AuraNode, AuraPropValue, AuraStateDef, Type};
    use std::collections::HashMap;

    fn make_counter_widget() -> AuraWidget {
        let view_tree = AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![
                AuraNode::Element {
                    tag: "text".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Counter".to_string())));
                        m
                    },
                    events: HashMap::new(),
                    children: vec![],
                    span: None,
                    debug_id: None,
                },
                AuraNode::Element {
                    tag: "button".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("text".to_string(), AuraPropValue::Expr(AuraExpr::Literal("Increment".to_string())));
                        m
                    },
                    events: {
                        let mut m = HashMap::new();
                        m.insert("onclick".to_string(), AuraEvent { handler: ".Inc".to_string(), params: vec![] });
                        m
                    },
                    children: vec![],
                    span: None,
                    debug_id: None,
                },
            ],
            span: None,
            debug_id: None,
        };

        AuraWidget {
            name: "Counter".to_string(),
            state_vars: vec![AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }],
            computed: vec![],
            messages: vec![],
            view_tree,
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
}
    }

    #[test]
    fn test_export_counter() {
        let widget = make_counter_widget();
        let result = export_widget(&widget).unwrap();

        match result {
            A2UIMessage::SurfaceUpdate(update) => {
                assert_eq!(update.surface_id, "Counter");
                assert_eq!(update.components.len(), 1);

                let col = &update.components[0];
                assert!(matches!(col.body, A2UIComponentBody::Column { .. }));

                if let A2UIComponentBody::Column { children } = &col.body {
                    assert_eq!(children.len(), 2);
                    assert!(matches!(children[0].body, A2UIComponentBody::Text { .. }));
                    assert!(matches!(children[1].body, A2UIComponentBody::Button { .. }));
                }
            }
        }
    }

    #[test]
    fn test_export_button_action() {
        let widget = make_counter_widget();
        let result = export_widget(&widget).unwrap();

        let A2UIMessage::SurfaceUpdate(update) = result;
        if let A2UIComponentBody::Column { children } = &update.components[0].body {
            if let A2UIComponentBody::Button { action, .. } = &children[1].body {
                assert!(action.is_some());
                assert_eq!(action.as_ref().unwrap().name, ".Inc");
            } else {
                panic!("Expected button");
            }
        }
    }

    #[test]
    fn test_export_unsupported_component() {
        let view_tree = AuraNode::Element {
            tag: "unknown_xyz".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        };

        let widget = AuraWidget {
            name: "Test".to_string(),
            state_vars: vec![],
            computed: vec![],
            messages: vec![],
            view_tree,
            handlers: HashMap::new(),
            props: vec![],
            routes: None,
            lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
            key_bindings: HashMap::new(),
};

        let result = export_widget(&widget);
        assert!(result.is_err());
    }
}
