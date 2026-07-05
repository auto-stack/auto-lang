//! A2UI → AURA Importer
//!
//! Converts Google's A2UI v0.8 JSON into AutoUI's AURA intermediate representation.

use super::{A2UIComponent, A2UIComponentBody, A2UIMessage, A2UIValue, A2UIError};
use crate::ast::Expr;
use crate::aura::{
    AuraEvent, AuraNode, AuraPropValue, AuraStateDef,
    AuraTextContent, AuraWidget, LogicPayload, Type,
};
use std::collections::HashMap;

/// Import an A2UI message into an AuraWidget.
///
/// Event handlers are created as stub LogicPayload entries.
/// State variables are inferred from path bindings found in the component tree.
///
/// # Example
/// ```rust,ignore
/// let a2ui_msg: A2UIMessage = serde_json::from_str(json)?;
/// let aura_widget = import_message(&a2ui_msg)?;
/// ```
pub fn import_message(msg: &A2UIMessage) -> Result<AuraWidget, A2UIError> {
    match msg {
        A2UIMessage::SurfaceUpdate(update) => import_surface_update(update),
    }
}

fn import_surface_update(
    update: &super::A2UISurfaceUpdate,
) -> Result<AuraWidget, A2UIError> {
    let mut nodes = Vec::new();
    let mut state_vars = Vec::new();
    let mut handlers: HashMap<String, LogicPayload> = HashMap::new();

    for comp in &update.components {
        let (node, comp_state, comp_handlers) = import_component(comp)?;
        nodes.push(node);
        state_vars.extend(comp_state);
        for (k, v) in comp_handlers {
            handlers.insert(k, v);
        }
    }

    // Deduplicate state vars by name
    let mut seen = HashMap::new();
    state_vars.retain(|s| {
        if seen.contains_key(&s.name) {
            false
        } else {
            seen.insert(s.name.clone(), ());
            true
        }
    });

    // Build the view tree
    let view_tree = if nodes.len() == 1 {
        nodes.into_iter().next().unwrap()
    } else {
        AuraNode::Element {
            tag: "col".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: nodes,
            span: None,
            debug_id: None,
        }
    };

    Ok(AuraWidget {
        name: update.surface_id.clone(),
        state_vars,
        computed: vec![],
        messages: vec![],
        view_tree,
        handlers,
        props: vec![],
        routes: None,
        lifecycle: vec![],
        tick_interval: None,
        handler_params: HashMap::new(),
        span_map: HashMap::new(),
        key_bindings: HashMap::new(),
        api_imports: vec![],
    })
}

fn import_component(
    comp: &A2UIComponent,
) -> Result<
    (
        AuraNode,
        Vec<AuraStateDef>,
        HashMap<String, LogicPayload>,
    ),
    A2UIError,
> {
    import_component_body(&comp.id, &comp.body)
}

fn import_component_body(
    id: &str,
    body: &A2UIComponentBody,
) -> Result<
    (
        AuraNode,
        Vec<AuraStateDef>,
        HashMap<String, LogicPayload>,
    ),
    A2UIError,
> {
    let mut state_vars = Vec::new();
    let mut handlers = HashMap::new();

    let node = match body {
        A2UIComponentBody::Container { children } => {
            let (child_nodes, child_states, child_handlers) =
                import_components(children)?;
            state_vars.extend(child_states);
            handlers.extend(child_handlers);
            AuraNode::Element {
                tag: "container".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: child_nodes,
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Row { children } => {
            let (child_nodes, child_states, child_handlers) =
                import_components(children)?;
            state_vars.extend(child_states);
            handlers.extend(child_handlers);
            AuraNode::Element {
                tag: "row".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: child_nodes,
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Column { children } => {
            let (child_nodes, child_states, child_handlers) =
                import_components(children)?;
            state_vars.extend(child_states);
            handlers.extend(child_handlers);
            AuraNode::Element {
                tag: "col".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children: child_nodes,
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::ScrollView { child } => {
            let mut children = Vec::new();
            if let Some(c) = child {
                let (node, s, h) = import_component_body(id, c)?;
                children.push(node);
                state_vars.extend(s);
                handlers.extend(h);
            }
            AuraNode::Element {
                tag: "scroll".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children,
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Text { text } => {
            let (value, mut sv) = import_value(text);
            state_vars.append(&mut sv);
            let text_content = match value {
                Expr::Str(s) => AuraTextContent::Literal(s.to_string()),
                Expr::Ident(name) => AuraTextContent::Interpolated {
                    template: format!("${{{}}}", name),
                    bindings: vec![name.to_string()],
                },
                _ => AuraTextContent::Literal(format!("{:?}", value)),
            };
            AuraNode::Text(text_content)
        }
        A2UIComponentBody::Button { child, action } => {
            let (child_expr, mut sv) = import_value(child);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            if let Expr::Str(s) = &child_expr {
                props.insert(
                    "text".to_string(),
                    AuraPropValue::Expr(Expr::Str(s.clone())),
                );
            }

            let mut events = HashMap::new();
            if let Some(action) = action {
                events.insert(
                    "onclick".to_string(),
                    AuraEvent {
                        handler: action.name.clone(),
                        params: action.context.iter().map(|c| c.name.clone()).collect(),
                    },
                );
                // Create stub handler
                handlers.insert(
                    action.name.clone(),
                    LogicPayload::AstStmts(vec![]),
                );
            }

            AuraNode::Element {
                tag: "button".to_string(),
                props,
                events,
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::TextInput { value, hint } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("value".to_string(), AuraPropValue::Expr(val_expr));
            if let Some(h) = hint {
                let (h_expr, mut h_sv) = import_value(h);
                state_vars.append(&mut h_sv);
                props.insert("placeholder".to_string(), AuraPropValue::Expr(h_expr));
            }

            AuraNode::Element {
                tag: "input".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::NumberInput { value, min, max } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("value".to_string(), AuraPropValue::Expr(val_expr));
            if let Some(m) = min {
                props.insert(
                    "min".to_string(),
                    AuraPropValue::Expr(Expr::Double(*m, "".into())),
                );
            }
            if let Some(m) = max {
                props.insert(
                    "max".to_string(),
                    AuraPropValue::Expr(Expr::Double(*m, "".into())),
                );
            }

            AuraNode::Element {
                tag: "input".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::DateTimeInput { value } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("value".to_string(), AuraPropValue::Expr(val_expr));

            AuraNode::Element {
                tag: "input".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Checkbox { value, label } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("checked".to_string(), AuraPropValue::Expr(val_expr));
            if let Some(l) = label {
                let (l_expr, mut l_sv) = import_value(l);
                state_vars.append(&mut l_sv);
                props.insert("label".to_string(), AuraPropValue::Expr(l_expr));
            }

            AuraNode::Element {
                tag: "checkbox".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Radio { value, label } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("selected".to_string(), AuraPropValue::Expr(val_expr));
            if let Some(l) = label {
                let (l_expr, mut l_sv) = import_value(l);
                state_vars.append(&mut l_sv);
                props.insert("label".to_string(), AuraPropValue::Expr(l_expr));
            }

            AuraNode::Element {
                tag: "radio".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Select { value, options } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("value".to_string(), AuraPropValue::Expr(val_expr));
            // Options are not directly representable in AURA props; skip for now
            let _ = options;

            AuraNode::Element {
                tag: "select".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Slider { value, min, max, step } => {
            let (val_expr, mut sv) = import_value(value);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("value".to_string(), AuraPropValue::Expr(val_expr));
            if let Some(m) = min {
                props.insert(
                    "min".to_string(),
                    AuraPropValue::Expr(Expr::Double(*m, "".into())),
                );
            }
            if let Some(m) = max {
                props.insert(
                    "max".to_string(),
                    AuraPropValue::Expr(Expr::Double(*m, "".into())),
                );
            }
            if let Some(s) = step {
                props.insert(
                    "step".to_string(),
                    AuraPropValue::Expr(Expr::Double(*s, "".into())),
                );
            }

            AuraNode::Element {
                tag: "slider".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Image { src } => {
            let (src_expr, mut sv) = import_value(src);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("src".to_string(), AuraPropValue::Expr(src_expr));

            AuraNode::Element {
                tag: "image".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Icon { name } => {
            let (name_expr, mut sv) = import_value(name);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("name".to_string(), AuraPropValue::Expr(name_expr));

            AuraNode::Element {
                tag: "icon".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Divider {} => AuraNode::Element {
            tag: "divider".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        },
        A2UIComponentBody::Spacer {} => AuraNode::Element {
            tag: "spacer".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        },
        A2UIComponentBody::List { items, template } => {
            let (items_expr, mut sv) = import_value(items);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("items".to_string(), AuraPropValue::Expr(items_expr));

            let mut children = Vec::new();
            if let Some(t) = template {
                let (t_node, t_s, t_h) = import_component_body(id, t)?;
                children.push(t_node);
                state_vars.extend(t_s);
                handlers.extend(t_h);
            }

            AuraNode::Element {
                tag: "list".to_string(),
                props,
                events: HashMap::new(),
                children,
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Table { columns, items } => {
            let (items_expr, mut sv) = import_value(items);
            state_vars.append(&mut sv);

            let mut props = HashMap::new();
            props.insert("items".to_string(), AuraPropValue::Expr(items_expr));
            let _ = columns;

            AuraNode::Element {
                tag: "table".to_string(),
                props,
                events: HashMap::new(),
                children: vec![],
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Tabs { tabs } => {
            let mut children = Vec::new();
            for (i, tab) in tabs.iter().enumerate() {
                let (tab_node, tab_s, tab_h) = import_component_body(
                    &format!("{}_tab_{}", id, i),
                    &tab.child,
                )?;
                children.push(AuraNode::Element {
                    tag: "tab".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        let (label_expr, mut l_sv) = import_value(&tab.label);
                        state_vars.append(&mut l_sv);
                        m.insert("label".to_string(), AuraPropValue::Expr(label_expr));
                        m
                    },
                    events: HashMap::new(),
                    children: vec![tab_node],
                    span: None,
                    debug_id: None,
                });
                state_vars.extend(tab_s);
                handlers.extend(tab_h);
            }
            AuraNode::Element {
                tag: "tabs".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children,
                span: None,
                debug_id: None,
            }
        }
        A2UIComponentBody::Navigation { items } => {
            let mut children = Vec::new();
            for (_i, item) in items.iter().enumerate() {
                let (label_expr, mut l_sv) = import_value(&item.label);
                state_vars.append(&mut l_sv);
                children.push(AuraNode::Element {
                    tag: "nav_item".to_string(),
                    props: {
                        let mut m = HashMap::new();
                        m.insert("label".to_string(), AuraPropValue::Expr(label_expr));
                        m.insert(
                            "path".to_string(),
                            AuraPropValue::Expr(Expr::Str(item.path.clone().into())),
                        );
                        m
                    },
                    events: HashMap::new(),
                    children: vec![],
                    span: None,
                    debug_id: None,
                });
            }
            AuraNode::Element {
                tag: "navigation".to_string(),
                props: HashMap::new(),
                events: HashMap::new(),
                children,
                span: None,
                debug_id: None,
            }
        }
    };

    Ok((node, state_vars, handlers))
}

fn import_components(
    components: &[A2UIComponent],
) -> Result<
    (
        Vec<AuraNode>,
        Vec<AuraStateDef>,
        HashMap<String, LogicPayload>,
    ),
    A2UIError,
> {
    let mut nodes = Vec::new();
    let mut state_vars = Vec::new();
    let mut handlers = HashMap::new();

    for comp in components {
        let (node, comp_states, comp_handlers) = import_component(comp)?;
        nodes.push(node);
        state_vars.extend(comp_states);
        handlers.extend(comp_handlers);
    }

    Ok((nodes, state_vars, handlers))
}

/// Convert an A2UIValue to an Expr and collect inferred state variables.
fn import_value(value: &A2UIValue) -> (Expr, Vec<AuraStateDef>) {
    let mut state_vars = Vec::new();

    let expr = match value {
        A2UIValue::Path { path } => {
            let var_name = path.trim_start_matches('/').trim().to_string();
            if !var_name.is_empty() {
                state_vars.push(AuraStateDef {
                    name: var_name.clone(),
                    type_info: Type::StrOwned,
                    initial: Expr::Str("".into()),
                    decorators: vec![],
                });
            }
            Expr::Ident(var_name.into())
        }
        A2UIValue::LiteralString { literal_string } => Expr::Str(literal_string.clone().into()),
        A2UIValue::LiteralNumber { literal_number } => {
            if literal_number.fract() == 0.0 {
                Expr::Int(*literal_number as i32)
            } else {
                Expr::Double(*literal_number, "".into())
            }
        }
        A2UIValue::LiteralBool { literal_bool } => Expr::Bool(*literal_bool),
    };

    (expr, state_vars)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_import_simple_text() {
        let msg = A2UIMessage::SurfaceUpdate(
            super::super::A2UISurfaceUpdate::new("demo").with_components(vec![
                super::super::A2UIComponent::new(
                    "greeting",
                    A2UIComponentBody::Text {
                        text: A2UIValue::string("Hello World"),
                    },
                ),
            ]),
        );

        let widget = import_message(&msg).unwrap();
        assert_eq!(widget.name, "demo");
        assert!(matches!(widget.view_tree, AuraNode::Text(_)));
    }

    #[test]
    fn test_import_button_with_action() {
        let msg = A2UIMessage::SurfaceUpdate(
            super::super::A2UISurfaceUpdate::new("test").with_components(vec![
                super::super::A2UIComponent::new(
                    "btn",
                    A2UIComponentBody::Button {
                        child: A2UIValue::string("Submit"),
                        action: Some(super::super::A2UIAction::new("submit_form")),
                    },
                ),
            ]),
        );

        let widget = import_message(&msg).unwrap();
        if let AuraNode::Element { tag, events, .. } = &widget.view_tree {
            assert_eq!(tag, "button");
            assert!(events.contains_key("onclick"));
            assert_eq!(events["onclick"].handler, "submit_form");
        } else {
            panic!("Expected Element node");
        }
        assert!(widget.handlers.contains_key("submit_form"));
    }

    #[test]
    fn test_import_path_binding_creates_state() {
        let msg = A2UIMessage::SurfaceUpdate(
            super::super::A2UISurfaceUpdate::new("test").with_components(vec![
                super::super::A2UIComponent::new(
                    "input",
                    A2UIComponentBody::TextInput {
                        value: A2UIValue::path("/username"),
                        hint: None,
                    },
                ),
            ]),
        );

        let widget = import_message(&msg).unwrap();
        assert!(widget.state_vars.iter().any(|s| s.name == "username"));
    }

    #[test]
    fn test_import_nested_layout() {
        let msg = A2UIMessage::SurfaceUpdate(
            super::super::A2UISurfaceUpdate::new("test").with_components(vec![
                super::super::A2UIComponent::new(
                    "root",
                    A2UIComponentBody::Column {
                        children: vec![
                            super::super::A2UIComponent::new(
                                "row1",
                                A2UIComponentBody::Row {
                                    children: vec![
                                        super::super::A2UIComponent::new(
                                            "t1",
                                            A2UIComponentBody::Text {
                                                text: A2UIValue::string("A"),
                                            },
                                        ),
                                        super::super::A2UIComponent::new(
                                            "t2",
                                            A2UIComponentBody::Text {
                                                text: A2UIValue::string("B"),
                                            },
                                        ),
                                    ],
                                },
                            ),
                        ],
                    },
                ),
            ]),
        );

        let widget = import_message(&msg).unwrap();
        if let AuraNode::Element { tag, children, .. } = &widget.view_tree {
            assert_eq!(tag, "col");
            assert_eq!(children.len(), 1);
            if let AuraNode::Element { tag: row_tag, children: row_children, .. } = &children[0] {
                assert_eq!(row_tag, "row");
                assert_eq!(row_children.len(), 2);
            } else {
                panic!("Expected row element");
            }
        } else {
            panic!("Expected column element");
        }
    }

    #[test]
    fn test_roundtrip_counter() {
        // Create a simple A2UI message
        let original = A2UIMessage::SurfaceUpdate(
            super::super::A2UISurfaceUpdate::new("Counter").with_components(vec![
                super::super::A2UIComponent::new(
                    "title",
                    A2UIComponentBody::Text {
                        text: A2UIValue::string("Counter"),
                    },
                ),
                super::super::A2UIComponent::new(
                    "inc",
                    A2UIComponentBody::Button {
                        child: A2UIValue::string("+"),
                        action: Some(super::super::A2UIAction::new("increment")),
                    },
                ),
            ]),
        );

        // Import to AURA
        let widget = import_message(&original).unwrap();
        assert_eq!(widget.name, "Counter");

        // Export back to A2UI
        let exported = super::super::export::export_widget(&widget).unwrap();

        // Both should be surface updates with the same surface ID
        match (original, exported) {
            (
                A2UIMessage::SurfaceUpdate(orig),
                A2UIMessage::SurfaceUpdate(exported),
            ) => {
                assert_eq!(orig.surface_id, exported.surface_id);
                // Original has 2 top-level comps; round-trip wraps them in a Column
                assert_eq!(exported.components.len(), 1);
                if let A2UIComponentBody::Column { children } = &exported.components[0].body {
                    assert_eq!(children.len(), orig.components.len());
                } else {
                    panic!("Expected Column wrapper");
                }
            }
        }
    }
}
