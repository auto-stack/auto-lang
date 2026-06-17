//! # Snapshot Builder — Traverse View tree to produce UiSnapshot (Plan 278)
//!
//! Walks the `View<DynamicMessage>` tree produced by `DynamicComponent::view_with_debug()`
//! and extracts component types, properties, and action handlers into a structured
//! `UiSnapshot` that can be formatted as AURA text for MCP consumption.
//!
//! ## Architecture
//!
//! ```text
//! DynamicComponent
//!   .view_with_debug() -> (View<DynamicMessage>, DebugIdMap)
//!                          |
//!   SnapshotBuilder::build() — traverse with DFS
//!                          |
//!                          v
//!                      UiSnapshot (AURA text via to_aura_string())
//! ```

use std::collections::HashMap;

use crate::aura::AuraNodeId;
use crate::ui::debug_id_map::DebugIdMap;
use crate::ui::interpreter::DynamicMessage;
use crate::ui::mcp_types::{UiAction, UiNode, UiSnapshot, format_value, type_hint};
use crate::ui::view::View;

/// Builder that traverses a View tree and produces a UiSnapshot.
pub struct SnapshotBuilder;

impl SnapshotBuilder {
    /// Build a complete snapshot from a View tree and DebugIdMap.
    ///
    /// # Arguments
    ///
    /// * `widget_name` — The name of the widget (e.g., "TodoApp")
    /// * `state` — Current state values from `DynamicComponent::read_all_state()`
    /// * `view` — The View tree from `DynamicComponent::view_with_debug()`
    /// * `id_map` — The DebugIdMap from `DynamicComponent::view_with_debug()`
    pub fn build(
        widget_name: &str,
        state: &HashMap<String, auto_val::Value>,
        view: &View<DynamicMessage>,
        id_map: &DebugIdMap,
    ) -> UiSnapshot {
        // Format state values
        let state_vec: Vec<(String, String, String)> = state
            .iter()
            .map(|(k, v)| (k.clone(), format_value(v), type_hint(v).to_string()))
            .collect();

        // Traverse the view tree
        let tree = Self::traverse_view(view, id_map, &[]);

        UiSnapshot {
            widget_name: widget_name.to_string(),
            state: state_vec,
            tree,
        }
    }

    /// Recursively traverse a View node and build a UiNode tree.
    ///
    /// Uses the DebugIdMap to find the AuraNodeId for each node by its path.
    fn traverse_view(
        view: &View<DynamicMessage>,
        id_map: &DebugIdMap,
        path: &[usize],
    ) -> UiNode {
        // Look up AuraNodeId from the path
        let id = id_map.get(path).unwrap_or(AuraNodeId(u32::MAX));

        match view {
            View::Empty => UiNode {
                id,
                kind: "Empty".to_string(),
                props: vec![],
                actions: vec![],
                children: vec![],
            },

            View::Text { content, .. } => UiNode {
                id,
                kind: "Text".to_string(),
                props: vec![("content".to_string(), content.clone())],
                actions: vec![],
                children: vec![],
            },

            View::Button { label, onclick, .. } => UiNode {
                id,
                kind: "Button".to_string(),
                props: vec![("label".to_string(), label.clone())],
                actions: vec![Self::extract_action("press", onclick)],
                children: vec![],
            },

            View::Input { placeholder, value, on_change, password, .. } => {
                let mut props = vec![
                    ("placeholder".to_string(), placeholder.clone()),
                    ("value".to_string(), value.clone()),
                ];
                if *password {
                    props.push(("password".to_string(), "true".to_string()));
                }
                let actions = on_change.as_ref()
                    .map(|msg| vec![Self::extract_action("type", msg)])
                    .unwrap_or_default();
                UiNode { id, kind: "Input".to_string(), props, actions, children: vec![] }
            }

            View::Textarea { placeholder, value, on_change, .. } => {
                let props = vec![
                    ("placeholder".to_string(), placeholder.clone()),
                    ("value".to_string(), value.clone()),
                ];
                let actions = on_change.as_ref()
                    .map(|msg| vec![Self::extract_action("type", msg)])
                    .unwrap_or_default();
                UiNode { id, kind: "Textarea".to_string(), props, actions, children: vec![] }
            },

            View::Checkbox { is_checked, label, on_toggle, .. } => {
                let props = vec![
                    ("checked".to_string(), is_checked.to_string()),
                    ("label".to_string(), label.clone()),
                ];
                let actions = on_toggle.as_ref()
                    .map(|msg| vec![Self::extract_action("toggle", msg)])
                    .unwrap_or_default();
                UiNode { id, kind: "Checkbox".to_string(), props, actions, children: vec![] }
            },

            View::Radio { label, is_selected, on_select, .. } => {
                let props = vec![
                    ("label".to_string(), label.clone()),
                    ("selected".to_string(), is_selected.to_string()),
                ];
                let actions = on_select.as_ref()
                    .map(|msg| vec![Self::extract_action("select", msg)])
                    .unwrap_or_default();
                UiNode { id, kind: "Radio".to_string(), props, actions, children: vec![] }
            },

            View::Select { options, selected_index, .. } => {
                let props = vec![
                    ("options".to_string(), format!("{:?}", options)),
                    ("selected".to_string(), selected_index
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "none".to_string())),
                ];
                // SelectCallback doesn't implement handler extraction easily,
                // so we just mark it as having a select action
                UiNode { id, kind: "Select".to_string(), props, actions: vec![], children: vec![] }
            },

            View::Slider { min, max, value, step, .. } => {
                let mut props = vec![
                    ("min".to_string(), min.to_string()),
                    ("max".to_string(), max.to_string()),
                    ("value".to_string(), value.to_string()),
                ];
                if let Some(s) = step {
                    props.push(("step".to_string(), s.to_string()));
                }
                // Slider's on_change is a fn(f32) -> M, not directly extractable
                UiNode { id, kind: "Slider".to_string(), props, actions: vec![], children: vec![] }
            },

            View::ProgressBar { progress, .. } => UiNode {
                id,
                kind: "ProgressBar".to_string(),
                props: vec![("progress".to_string(), format!("{:.0}%", progress * 100.0))],
                actions: vec![],
                children: vec![],
            },

            View::Image { src, .. } => UiNode {
                id,
                kind: "Image".to_string(),
                props: vec![("src".to_string(), src.clone())],
                actions: vec![],
                children: vec![],
            },

            View::Row { children, spacing, padding, .. } => {
                let child_nodes = Self::traverse_children(children, id_map, path);
                UiNode {
                    id,
                    kind: "Row".to_string(),
                    props: vec![
                        ("spacing".to_string(), spacing.to_string()),
                        ("padding".to_string(), padding.to_string()),
                    ],
                    actions: vec![],
                    children: child_nodes,
                }
            },

            View::Column { children, spacing, padding, .. } => {
                let child_nodes = Self::traverse_children(children, id_map, path);
                UiNode {
                    id,
                    kind: "Column".to_string(),
                    props: vec![
                        ("spacing".to_string(), spacing.to_string()),
                        ("padding".to_string(), padding.to_string()),
                    ],
                    actions: vec![],
                    children: child_nodes,
                }
            },

            // Grid (Plan 319): cells are the snapshot children.
            View::Grid { cols, gap, cells, .. } => {
                let child_nodes = Self::traverse_children(cells, id_map, path);
                UiNode {
                    id,
                    kind: "Grid".to_string(),
                    props: vec![
                        ("cols".to_string(), cols.to_string()),
                        ("gap".to_string(), gap.to_string()),
                    ],
                    actions: vec![],
                    children: child_nodes,
                }
            },

            View::Container { padding, width, height, center_x, center_y, child, .. } => {
                let mut props = vec![("padding".to_string(), padding.to_string())];
                if let Some(w) = width {
                    props.push(("width".to_string(), w.to_string()));
                }
                if let Some(h) = height {
                    props.push(("height".to_string(), h.to_string()));
                }
                if *center_x {
                    props.push(("center_x".to_string(), "true".to_string()));
                }
                if *center_y {
                    props.push(("center_y".to_string(), "true".to_string()));
                }
                let child_path = [path, &[0]].concat();
                let child_node = Self::traverse_view(child, id_map, &child_path);
                UiNode { id, kind: "Container".to_string(), props, actions: vec![], children: vec![child_node] }
            },

            View::Scrollable { width, height, child, .. } => {
                let mut props = vec![];
                if let Some(w) = width {
                    props.push(("width".to_string(), w.to_string()));
                }
                if let Some(h) = height {
                    props.push(("height".to_string(), h.to_string()));
                }
                let child_path = [path, &[0]].concat();
                let child_node = Self::traverse_view(child, id_map, &child_path);
                UiNode { id, kind: "Scrollable".to_string(), props, actions: vec![], children: vec![child_node] }
            },

            View::List { items, spacing, .. } => {
                let child_nodes = Self::traverse_children(items, id_map, path);
                UiNode {
                    id,
                    kind: "List".to_string(),
                    props: vec![("spacing".to_string(), spacing.to_string())],
                    actions: vec![],
                    children: child_nodes,
                }
            },

            View::Table { headers, rows, spacing, col_spacing, .. } => {
                let mut all_children = Vec::new();
                // Headers
                for (i, h) in headers.iter().enumerate() {
                    let child_path = [path, &[0, i]].concat();
                    all_children.push(Self::traverse_view(h, id_map, &child_path));
                }
                // Rows
                for (ri, row) in rows.iter().enumerate() {
                    for (ci, cell) in row.iter().enumerate() {
                        let child_path = [path, &[1 + ri, ci]].concat();
                        all_children.push(Self::traverse_view(cell, id_map, &child_path));
                    }
                }
                UiNode {
                    id,
                    kind: "Table".to_string(),
                    props: vec![
                        ("rows".to_string(), rows.len().to_string()),
                        ("cols".to_string(), headers.len().to_string()),
                        ("spacing".to_string(), spacing.to_string()),
                        ("col_spacing".to_string(), col_spacing.to_string()),
                    ],
                    actions: vec![],
                    children: all_children,
                }
            },

            View::Accordion { items, allow_multiple, .. } => {
                let child_nodes: Vec<UiNode> = items
                    .iter()
                    .enumerate()
                    .flat_map(|(i, item)| {
                        let mut nodes = vec![UiNode {
                            id: id_map.get(&[path, &[i]].concat()).unwrap_or(AuraNodeId(u32::MAX)),
                            kind: "AccordionHeader".to_string(),
                            props: vec![
                                ("title".to_string(), item.title.clone()),
                                ("expanded".to_string(), item.expanded.to_string()),
                            ],
                            actions: vec![],
                            children: vec![],
                        }];
                        for (j, child) in item.children.iter().enumerate() {
                            let child_path = [path, &[i, j]].concat();
                            nodes.push(Self::traverse_view(child, id_map, &child_path));
                        }
                        nodes
                    })
                    .collect();
                UiNode {
                    id,
                    kind: "Accordion".to_string(),
                    props: vec![("allow_multiple".to_string(), allow_multiple.to_string())],
                    actions: vec![],
                    children: child_nodes,
                }
            },

            View::Tabs { labels, selected, position, .. } => {
                let labels_str = format!("{:?}", labels);
                UiNode {
                    id,
                    kind: "Tabs".to_string(),
                    props: vec![
                        ("labels".to_string(), labels_str),
                        ("selected".to_string(), selected.to_string()),
                        ("position".to_string(), format!("{:?}", position).to_lowercase()),
                    ],
                    actions: vec![],
                    children: vec![], // Tab contents are complex, skip for now
                }
            },

            View::Sidebar { width, collapsible, position, .. } => {
                UiNode {
                    id,
                    kind: "Sidebar".to_string(),
                    props: vec![
                        ("width".to_string(), width.to_string()),
                        ("collapsible".to_string(), collapsible.to_string()),
                        ("position".to_string(), format!("{:?}", position).to_lowercase()),
                    ],
                    actions: vec![],
                    children: vec![], // Sidebar content is a Box<View>, skip deep traversal
                }
            },

            View::NavigationRail { items, selected, show_labels, .. } => {
                let labels: Vec<String> = items.iter().map(|i| i.label.clone()).collect();
                UiNode {
                    id,
                    kind: "NavigationRail".to_string(),
                    props: vec![
                        ("items".to_string(), format!("{:?}", labels)),
                        ("selected".to_string(), selected.to_string()),
                        ("show_labels".to_string(), show_labels.to_string()),
                    ],
                    actions: vec![],
                    children: vec![],
                }
            },
        }
    }

    /// Traverse a slice of child Views.
    fn traverse_children(
        children: &[View<DynamicMessage>],
        id_map: &DebugIdMap,
        parent_path: &[usize],
    ) -> Vec<UiNode> {
        children
            .iter()
            .enumerate()
            .map(|(i, child)| {
                let child_path = [parent_path, &[i]].concat();
                Self::traverse_view(child, id_map, &child_path)
            })
            .collect()
    }

    /// Extract an action from a DynamicMessage.
    ///
    /// Converts the message's event_name into a handler pattern (e.g., ".Inc").
    fn extract_action(action_name: &str, msg: &DynamicMessage) -> UiAction {
        let handler = match msg {
            DynamicMessage::Typed { event_name, .. } => format!(".{}", event_name),
            DynamicMessage::String(name) => format!(".{}", name),
        };
        UiAction {
            name: action_name.to_string(),
            handler,
        }
    }

    /// Find a node by AuraNodeId in the snapshot tree.
    pub fn find_node<'a>(tree: &'a UiNode, target_id: AuraNodeId) -> Option<&'a UiNode> {
        if tree.id == target_id {
            return Some(tree);
        }
        for child in &tree.children {
            if let Some(found) = Self::find_node(child, target_id) {
                return Some(found);
            }
        }
        None
    }
}
