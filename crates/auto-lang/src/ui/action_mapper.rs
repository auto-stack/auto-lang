//! # Action Mapper — Map MCP actions to VmBridge calls (Plan 278)
//!
//! Translates high-level MCP actions (press, type_text, toggle, etc.)
//! into concrete VmBridge operations (call_handler, write_state).
//!
//! ## Action → Handler Mapping
//!
//! | Action        | Component       | VmBridge Call                                |
//! |---------------|-----------------|----------------------------------------------|
//! | press         | Button          | call_handler(event_name, [])                 |
//! | type_text     | Input/Textarea  | write_state(field, value) + call_handler     |
//! | toggle        | Checkbox        | call_handler(event_name, [])                 |
//! | select_option | Select/Radio    | call_handler(event_name, [index, label])     |
//! | set_value     | Slider          | write_state(field, value) + call_handler     |

use std::collections::HashMap;

use crate::aura::AuraNodeId;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::mcp_types::{ActionResult, UiActionType, UiNode, format_value};
use crate::ui::view::View;

/// Error returned when an action cannot be performed.
#[derive(Debug)]
pub enum ActionError {
    /// The target element was not found in the view tree.
    ElementNotFound(AuraNodeId),
    /// The action is not supported for this component type.
    InvalidAction {
        action: UiActionType,
        component_kind: String,
    },
    /// The component has no handler for this action.
    NoHandler {
        action: UiActionType,
        element_id: AuraNodeId,
    },
    /// The value parameter is required but not provided.
    MissingValue(UiActionType),
    /// The provided value is invalid.
    InvalidValue(String),
    /// VmBridge execution error.
    ExecutionError(String),
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::ElementNotFound(id) => write!(f, "Element not found: #{}", id),
            ActionError::InvalidAction { action, component_kind } => {
                write!(f, "Action '{}' is not valid for component type '{}'", action, component_kind)
            }
            ActionError::NoHandler { action, element_id } => {
                write!(f, "No handler for action '{}' on element #{}", action, element_id)
            }
            ActionError::MissingValue(action) => {
                write!(f, "Action '{}' requires a value parameter", action)
            }
            ActionError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ActionError::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
        }
    }
}

/// Context needed to perform an action.
///
/// This trait abstracts over the actual state mutation mechanism,
/// allowing the action mapper to work with any backend that can
/// read/write state and call handlers.
pub trait ActionContext {
    /// Read all current state values.
    fn read_all_state(&self) -> HashMap<String, auto_val::Value>;

    /// Write a state field value.
    fn write_state(&mut self, field: &str, value: auto_val::Value) -> Result<(), String>;

    /// Call an event handler by name.
    fn call_handler(&mut self, event_name: &str, args: &[auto_val::Value]) -> Result<(), String>;

    /// Get the input-to-state mapping (event_name -> state_field_name).
    fn input_state_map(&self) -> &HashMap<String, String>;
}

/// Result of resolving an action against a View tree node.
struct ResolvedAction {
    /// Handler event name (e.g., "AddTodo")
    event_name: String,
    /// Whether write_state is needed before calling the handler
    needs_state_write: bool,
    /// State field to write (if needs_state_write)
    state_field: Option<String>,
    /// Value to write to state (if needs_state_write)
    state_value: Option<auto_val::Value>,
}

/// Resolve and execute an MCP action on the View tree.
///
/// This function:
/// 1. Finds the target element by AuraNodeId
/// 2. Validates the action is compatible with the component type
/// 3. Resolves the handler from the DynamicMessage
/// 4. Captures before/after state
/// 5. Executes the action via the ActionContext
pub fn execute_action(
    ctx: &mut dyn ActionContext,
    view: &View<DynamicMessage>,
    _snapshot_tree: &UiNode,
    element_id: AuraNodeId,
    action: UiActionType,
    value: Option<auto_val::Value>,
) -> Result<ActionResult, ActionError> {
    // 1. Capture before state
    let before_state = ctx.read_all_state();

    // 2. Find the target node in the snapshot tree
    let target_node = find_node(_snapshot_tree, element_id)
        .ok_or(ActionError::ElementNotFound(element_id))?;

    // 3. Find the corresponding View node and resolve the action
    let resolved = resolve_action_from_view(view, target_node, &action, value, ctx.input_state_map())?;

    // 4. Execute the action
    if resolved.needs_state_write {
        if let (Some(field), Some(val)) = (&resolved.state_field, &resolved.state_value) {
            ctx.write_state(field, val.clone())
                .map_err(ActionError::ExecutionError)?;
        }
    }

    ctx.call_handler(&resolved.event_name, &[])
        .map_err(ActionError::ExecutionError)?;

    // 5. Capture after state and compute diff
    let after_state = ctx.read_all_state();
    let state_changes = compute_state_diff(&before_state, &after_state);

    Ok(ActionResult {
        status: "ok".to_string(),
        element_id,
        action: action.to_string(),
        handler: Some(format!(".{}", resolved.event_name)),
        state_changes,
    })
}

/// Resolve an action by looking up the handler in the actual View tree.
fn resolve_action_from_view(
    _view: &View<DynamicMessage>,
    target: &UiNode,
    action: &UiActionType,
    value: Option<auto_val::Value>,
    input_map: &HashMap<String, String>,
) -> Result<ResolvedAction, ActionError> {
    match action {
        UiActionType::Press => {
            // Only valid for Button
            if target.kind != "Button" {
                return Err(ActionError::InvalidAction {
                    action: action.clone(),
                    component_kind: target.kind.clone(),
                });
            }
            // Find the onclick handler from actions list
            let handler = target.actions.iter()
                .find(|a| a.name == "press")
                .map(|a| a.handler.trim_start_matches('.').to_string())
                .ok_or(ActionError::NoHandler {
                    action: action.clone(),
                    element_id: target.id,
                })?;

            Ok(ResolvedAction {
                event_name: handler,
                needs_state_write: false,
                state_field: None,
                state_value: None,
            })
        }

        UiActionType::TypeText => {
            // Valid for Input, Textarea
            if target.kind != "Input" && target.kind != "Textarea" {
                return Err(ActionError::InvalidAction {
                    action: action.clone(),
                    component_kind: target.kind.clone(),
                });
            }

            let text = match value {
                Some(auto_val::Value::Str(s)) => s.to_string(),
                Some(other) => other.to_string(),
                None => return Err(ActionError::MissingValue(action.clone())),
            };

            // Find the handler
            let handler = target.actions.iter()
                .find(|a| a.name == "type")
                .map(|a| a.handler.trim_start_matches('.').to_string())
                .ok_or(ActionError::NoHandler {
                    action: action.clone(),
                    element_id: target.id,
                })?;

            // Find the bound state field from input_map
            let state_field = input_map.get(&handler).cloned();

            Ok(ResolvedAction {
                event_name: handler,
                needs_state_write: state_field.is_some(),
                state_field,
                state_value: Some(auto_val::Value::str(&text)),
            })
        }

        UiActionType::Toggle => {
            // Valid for Checkbox
            if target.kind != "Checkbox" {
                return Err(ActionError::InvalidAction {
                    action: action.clone(),
                    component_kind: target.kind.clone(),
                });
            }

            let handler = target.actions.iter()
                .find(|a| a.name == "toggle")
                .map(|a| a.handler.trim_start_matches('.').to_string())
                .ok_or(ActionError::NoHandler {
                    action: action.clone(),
                    element_id: target.id,
                })?;

            Ok(ResolvedAction {
                event_name: handler,
                needs_state_write: false,
                state_field: None,
                state_value: None,
            })
        }

        UiActionType::SelectOption => {
            // Valid for Select, Radio
            if target.kind != "Select" && target.kind != "Radio" {
                return Err(ActionError::InvalidAction {
                    action: action.clone(),
                    component_kind: target.kind.clone(),
                });
            }

            // Value should be the option index or label
            let _val = value.ok_or(ActionError::MissingValue(action.clone()))?;

            // For now, just call the handler if one exists
            // TODO: Support select callbacks when they carry DynamicMessage
            Err(ActionError::NoHandler {
                action: action.clone(),
                element_id: target.id,
            })
        }

        UiActionType::SetValue => {
            // Valid for Slider
            if target.kind != "Slider" {
                return Err(ActionError::InvalidAction {
                    action: action.clone(),
                    component_kind: target.kind.clone(),
                });
            }

            let num = value.ok_or(ActionError::MissingValue(action.clone()))?;
            let fval = match num {
                auto_val::Value::Int(i) => i as f32,
                auto_val::Value::Float(f) => f as f32,
                other => {
                    return Err(ActionError::InvalidValue(
                        format!("expected number, got {:?}", other)
                    ));
                }
            };

            // Slider handler is a fn(f32) -> M, not easily extractable from View
            // For now, mark as needing state write if we can find the bound field
            Ok(ResolvedAction {
                event_name: format!("slider_{}", target.id.0),
                needs_state_write: false,
                state_field: None,
                state_value: Some(auto_val::Value::Float(fval as f64)),
            })
        }

        UiActionType::Clear => {
            // Valid for Input, Textarea — clear sends empty string
            if target.kind != "Input" && target.kind != "Textarea" {
                return Err(ActionError::InvalidAction {
                    action: action.clone(),
                    component_kind: target.kind.clone(),
                });
            }

            let handler = target.actions.iter()
                .find(|a| a.name == "type")
                .map(|a| a.handler.trim_start_matches('.').to_string())
                .ok_or(ActionError::NoHandler {
                    action: action.clone(),
                    element_id: target.id,
                })?;

            let state_field = input_map.get(&handler).cloned();

            Ok(ResolvedAction {
                event_name: handler,
                needs_state_write: state_field.is_some(),
                state_field,
                state_value: Some(auto_val::Value::str("")),
            })
        }
    }
}

/// Find a UiNode by AuraNodeId.
fn find_node<'a>(tree: &'a UiNode, target_id: AuraNodeId) -> Option<&'a UiNode> {
    if tree.id == target_id {
        return Some(tree);
    }
    for child in &tree.children {
        if let Some(found) = find_node(child, target_id) {
            return Some(found);
        }
    }
    None
}

/// Compute the diff between two state maps.
fn compute_state_diff(
    before: &HashMap<String, auto_val::Value>,
    after: &HashMap<String, auto_val::Value>,
) -> Vec<(String, String, String)> {
    let mut changes = Vec::new();
    for (key, after_val) in after {
        let before_val = before.get(key);
        if before_val.map_or(true, |bv| format_value(bv) != format_value(after_val)) {
            changes.push((
                key.clone(),
                before_val.map_or("null".to_string(), format_value),
                format_value(after_val),
            ));
        }
    }
    changes
}
