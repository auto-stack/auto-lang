//! Scan → descriptor assembly (PLAN-780 T-02).
//!
//! Every modeled field's normalization mirrors `auto_man::Pac::new` (the
//! pilot oracle) point by point; where Pac silently degrades (e.g. members
//! declared as a scalar reads as an empty vec), the model instead classifies
//! the construct unsupported so the file is excluded from any equality
//! claim rather than faked green.

use crate::descriptor::*;
use crate::scanner::{Entry, ScanOutput, ScannedValue};
use auto_val::Value;
use std::collections::HashSet;

/// Canonical backend names accepted by `auto_lang::config::BackendType::
/// from_str` (`remote` normalizes to `vm`, `ark` to `arkts`).
fn canonical_backend_name(s: &str) -> Option<String> {
    let t = s.trim().to_lowercase();
    let canon = match t.as_str() {
        "vue" | "jet" | "tauri" | "gpui" | "iced" | "cangjie" | "godot" | "rust" | "vm"
        | "vscode" => t,
        "arkts" | "ark" => "arkts".to_string(),
        "remote" => "vm".to_string(),
        _ => return None,
    };
    Some(canon)
}

/// Mirror of `BackendConfig::from_value` shapes.
fn render_spec_from_value(value: &Value) -> RenderSpec {
    match value {
        Value::Str(s) => match canonical_backend_name(s.as_str()) {
            Some(name) => RenderSpec::Single(name),
            // Pac: from_str unknown → from_value None (the value survives
            // only in the legacy backend string, preserved as Unknown repr)
            None => RenderSpec::Unknown(s.to_string()),
        },
        Value::Array(arr) => {
            // Pac filter_maps Str elements through from_str, silently
            // dropping unknown names — mirrored verbatim (declared text is
            // recoverable from provenance + raw scan evidence).
            let names: Vec<String> = arr
                .values
                .iter()
                .filter_map(|v| match v {
                    Value::Str(s) => canonical_backend_name(s.as_str()),
                    _ => None,
                })
                .collect();
            match names.len() {
                1 => RenderSpec::Single(names[0].clone()),
                n if n > 1 => RenderSpec::Multi(names),
                _ => RenderSpec::Absent,
            }
        }
        Value::Obj(obj) => {
            let front = obj.get("front").and_then(|v| match v {
                Value::Str(s) => canonical_backend_name(s.as_str()).map(|n| vec![n]),
                Value::Array(arr) => Some(
                    arr.values
                        .iter()
                        .filter_map(|v| match v {
                            Value::Str(s) => canonical_backend_name(s.as_str()),
                            _ => None,
                        })
                        .collect(),
                ),
                _ => None,
            });
            let back = obj.get("back").and_then(|v| match v {
                Value::Str(s) => canonical_backend_name(s.as_str()),
                _ => None,
            });
            match (front, back) {
                (Some(f), Some(b)) => RenderSpec::Split { front: f, back: b },
                // Pac: malformed split → from_value None
                _ => RenderSpec::Unknown(value.to_astr().to_string()),
            }
        }
        other => RenderSpec::Unknown(other.to_astr().to_string()),
    }
}

fn astr(v: &Value) -> String {
    v.to_astr().to_string()
}

fn trimmed_non_empty(v: &Value) -> Option<String> {
    let s = astr(v).trim().to_string();
    (!s.is_empty()).then_some(s)
}

fn lower_trimmed_non_empty(v: &Value) -> Option<String> {
    let s = astr(v).trim().to_lowercase();
    (!s.is_empty()).then_some(s)
}

fn toggle_from_value(v: &Value) -> ToggleValue {
    match v {
        Value::Bool(b) => ToggleValue::Bool(*b),
        other => ToggleValue::Str(astr(other)),
    }
}

pub(crate) fn build(source: SourceInfo, scan: ScanOutput) -> ProjectDescriptor {
    let mut d = ProjectDescriptor {
        source,
        identity: Identity::default(),
        render: RenderDecl {
            render: RenderSpec::Absent,
            legacy_backend: None,
            effective_backend: None,
        },
        members: None,
        defines: None,
        deps: Vec::new(),
        display: DisplayFields::default(),
        runtime: RuntimeFields::default(),
        toggles: ToggleFields::default(),
        external_backend: None,
        raw_props: Vec::new(),
        raw_nodes: Vec::new(),
        duplicates: Vec::new(),
        unsupported: Vec::new(),
        provenance: Vec::new(),
    };
    let mut render_prop: Option<Value> = None;
    let mut backend_prop: Option<Value> = None;
    let mut seen: HashSet<String> = HashSet::new();

    for entry in scan.entries {
        apply_entry(
            entry,
            &mut d,
            &mut render_prop,
            &mut backend_prop,
            &mut seen,
        );
    }

    // RenderDecl finalize — mirrors Pac::new's effective-value chain:
    // render (when its astr is non-empty) wins over backend.
    let effective = render_prop
        .as_ref()
        .filter(|v| !astr(v).is_empty())
        .or(backend_prop.as_ref())
        .cloned();
    d.render.render = effective
        .as_ref()
        .map(render_spec_from_value)
        .unwrap_or(RenderSpec::Absent);
    d.render.legacy_backend = backend_prop.as_ref().map(astr).filter(|s| !s.is_empty());
    let effective_str = effective.as_ref().map(astr).unwrap_or_default();
    d.render.effective_backend = Some(if !effective_str.is_empty() {
        effective_str
    } else {
        // Pac lang inference for the legacy backend string
        match d.identity.lang.as_deref() {
            Some("rust") => "rust".to_string(),
            Some("ts") | Some("typescript") => "ts".to_string(),
            _ => "c".to_string(),
        }
    });

    d
}

#[allow(clippy::too_many_arguments)]
fn apply_entry(
    entry: Entry,
    d: &mut ProjectDescriptor,
    render_prop: &mut Option<Value>,
    backend_prop: &mut Option<Value>,
    seen: &mut HashSet<String>,
) {
    match entry {
        Entry::Prop { key, value, line } => {
            apply_prop(key, value, line, d, render_prop, backend_prop, seen)
        }
        Entry::Node {
            head,
            name,
            body,
            line,
        } => apply_node(head, name, body, line, d),
        Entry::Unsupported(u) => d.unsupported.push(u),
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_prop(
    key: String,
    value: ScannedValue,
    line: usize,
    d: &mut ProjectDescriptor,
    render_prop: &mut Option<Value>,
    backend_prop: &mut Option<Value>,
    seen: &mut HashSet<String>,
) {
    if !seen.insert(key.clone()) {
        d.duplicates.push(DuplicateKey {
            key: key.clone(),
            line,
        });
    }
    let lit: Option<&Value> = match &value {
        ScannedValue::Literal(v) => Some(v),
        ScannedValue::Unsupported(u) => {
            d.unsupported.push(u.clone());
            None
        }
    };

    match key.as_str() {
        "name" => {
            d.provenance.push(("name", line));
            d.identity.name = lit.map(astr);
        }
        "version" => {
            d.provenance.push(("version", line));
            d.identity.version = lit.map(astr);
        }
        "lang" => {
            d.provenance.push(("lang", line));
            // Pac: only Value::Str feeds top_lang; other shapes fall back
            // to "c" — the model records the raw prop instead of guessing.
            match lit {
                Some(Value::Str(s)) => d.identity.lang = Some(s.to_string()),
                Some(other) => {
                    d.identity.lang = None;
                    d.raw_props.push(RawProp {
                        key: key.clone(),
                        value: Some(other.clone()),
                        line,
                    });
                }
                None => d.identity.lang = None,
            }
        }
        "render" => {
            d.provenance.push(("render", line));
            *render_prop = lit.cloned();
        }
        "backend" => {
            d.provenance.push(("backend", line));
            *backend_prop = lit.cloned();
        }
        "api" => {
            d.provenance.push(("api", line));
            d.runtime.api = lit.map(astr);
        }
        "scene" => {
            d.provenance.push(("scene", line));
            d.runtime.scene = lit.map(|v| SceneKind::parse(&astr(v)));
        }
        "members" => {
            d.provenance.push(("members", line));
            match lit.and_then(str_vec_mirror) {
                Some(items) => d.members = Some(items),
                None if lit.is_some() => {
                    // Pac degrades silently to an empty vec; the model
                    // classifies so the file never fakes equality.
                    d.unsupported.push(UnsupportedConstruct {
                        kind: UnsupportedKind::TypeMismatch { field: "members" },
                        line,
                        column: 0,
                        raw: key.clone(),
                        key: Some(key.clone()),
                    });
                    d.members = None;
                }
                None => d.members = None,
            }
        }
        "defines" => {
            d.provenance.push(("defines", line));
            match lit.and_then(str_vec_mirror) {
                Some(items) => d.defines = Some(items),
                None if lit.is_some() => {
                    d.unsupported.push(UnsupportedConstruct {
                        kind: UnsupportedKind::TypeMismatch { field: "defines" },
                        line,
                        column: 0,
                        raw: key.clone(),
                        key: Some(key.clone()),
                    });
                    d.defines = None;
                }
                None => d.defines = None,
            }
        }
        "title" => {
            d.provenance.push(("title", line));
            d.display.title = lit.and_then(trimmed_non_empty);
        }
        "title_zh" => {
            d.provenance.push(("title_zh", line));
            d.display.title_zh = lit.and_then(trimmed_non_empty);
        }
        "icon" => {
            d.provenance.push(("icon", line));
            d.display.icon = lit.and_then(trimmed_non_empty);
        }
        "category" => {
            d.provenance.push(("category", line));
            d.display.category = lit.and_then(trimmed_non_empty);
        }
        "opens" => {
            d.provenance.push(("opens", line));
            d.display.opens = lit.map(astr);
        }
        "theme" => {
            d.provenance.push(("theme", line));
            // Scalar arm only — the theme{} block form (PLAN-601) is a
            // composable declaration consumed via design_tokens; catalogued
            // as a raw prop, not modeled here.
            match lit {
                Some(v @ Value::Str(_)) => d.display.theme = lower_trimmed_non_empty(v),
                Some(other) => {
                    d.display.theme = None;
                    d.raw_props.push(RawProp {
                        key: key.clone(),
                        value: Some(other.clone()),
                        line,
                    });
                }
                None => d.display.theme = None,
            }
        }
        "accent" => {
            d.provenance.push(("accent", line));
            d.display.accent = lit.and_then(lower_trimmed_non_empty);
        }
        "desktop_render" => {
            d.provenance.push(("desktop_render", line));
            d.display.desktop_render = lit.and_then(lower_trimmed_non_empty);
        }
        "daemon" => {
            d.provenance.push(("daemon", line));
            d.display.daemon = lit.map(astr);
        }
        "desktop" => {
            d.provenance.push(("desktop", line));
            d.display.desktop = lit.map(astr);
        }
        "desktop_exe" => {
            d.provenance.push(("desktop_exe", line));
            d.display.desktop_exe = lit.map(astr);
        }
        "ui_config" => {
            d.provenance.push(("ui_config", line));
            d.display.ui_config = lit.and_then(trimmed_non_empty);
        }
        "front_port" => {
            d.provenance.push(("front_port", line));
            d.runtime.front_port = lit.and_then(port_mirror);
        }
        "back_port" => {
            d.provenance.push(("back_port", line));
            d.runtime.back_port = lit.and_then(port_mirror);
        }
        "media_root" => {
            d.provenance.push(("media_root", line));
            d.runtime.media_root = lit.and_then(trimmed_non_empty);
        }
        "photo_root" => {
            d.provenance.push(("photo_root", line));
            d.runtime.photo_root = lit.and_then(trimmed_non_empty);
        }
        "window" => {
            d.provenance.push(("window", line));
            d.runtime.window = lit.map(|v| parse_window(&astr(v)).0).flatten();
        }
        "back" => {
            d.provenance.push(("back", line));
            d.external_backend = lit.and_then(|v| match v {
                Value::Obj(obj) => obj
                    .get("project")
                    .map(|pv: Value| astr(&pv).trim().to_string())
                    .filter(|s| !s.is_empty()),
                other => trimmed_non_empty(other),
            });
        }
        "shadcn" => {
            d.provenance.push(("shadcn", line));
            d.toggles.shadcn =
                lit.map(toggle_from_value).unwrap_or(ToggleValue::Unset);
        }
        "default_classes" => {
            d.provenance.push(("default_classes", line));
            d.toggles.default_classes =
                lit.map(toggle_from_value).unwrap_or(ToggleValue::Unset);
        }
        _ => {
            d.raw_props.push(RawProp {
                key: key.clone(),
                value: lit.cloned(),
                line,
            });
        }
    }
}

fn str_vec_mirror(v: &Value) -> Option<Vec<String>> {
    match v {
        Value::Array(arr) => Some(arr.values.iter().map(astr).collect()),
        _ => None,
    }
}

fn port_mirror(v: &Value) -> Option<u16> {
    astr(v).trim().parse::<u16>().ok()
}

fn node_name_string(name: &NodeName) -> Option<String> {
    match name {
        NodeName::Bare(n) | NodeName::Quoted(n) => Some(n.clone()),
        NodeName::Arg { name: Some(n), .. } => Some(n.clone()),
        _ => None,
    }
}

fn apply_node(
    head: String,
    name: NodeName,
    body: Vec<Entry>,
    line: usize,
    d: &mut ProjectDescriptor,
) {
    if head == "dep" {
        match node_name_string(&name) {
            Some(dep_name) => {
                let form = match &name {
                    NodeName::Bare(_) => DepNameForm::Bare,
                    NodeName::Quoted(_) => DepNameForm::Quoted,
                    _ => DepNameForm::Arg,
                };
                let mut props = Vec::new();
                let mut children = Vec::new();
                collect_body(body, d, &mut props, &mut children);
                let path = props
                    .iter()
                    .find(|(k, _)| k == "path")
                    .map(|(_, v)| astr(v));
                d.deps.push(DependencyDecl {
                    name: dep_name,
                    name_form: form,
                    path,
                    props,
                    line,
                });
            }
            None => {
                d.unsupported.push(UnsupportedConstruct {
                    kind: UnsupportedKind::DepNameUnrepresentable,
                    line,
                    column: 0,
                    raw: "dep (…)".to_string(),
                    key: None,
                });
                let raw = raw_node_from(head, name, body, line, d);
                d.raw_nodes.push(raw);
            }
        }
        return;
    }
    // Unmodeled node kinds (targets: app/lib/bag/device/test, and anything
    // else) — catalogued with full props/children, never materialized.
    let raw = raw_node_from(head, name, body, line, d);
    d.raw_nodes.push(raw);
}

fn raw_node_from(
    head: String,
    name: NodeName,
    body: Vec<Entry>,
    line: usize,
    d: &mut ProjectDescriptor,
) -> RawNodeDecl {
    let mut props = Vec::new();
    let mut children = Vec::new();
    collect_body(body, d, &mut props, &mut children);
    RawNodeDecl {
        head,
        name: node_name_string(&name),
        props,
        children,
        line,
    }
}

/// Flatten node-body entries into (props, child raw nodes). Unsupported
/// body constructs are recorded on the descriptor, not dropped.
fn collect_body(
    body: Vec<Entry>,
    d: &mut ProjectDescriptor,
    props: &mut Vec<(String, Value)>,
    children: &mut Vec<RawNodeDecl>,
) {
    for entry in body {
        match entry {
            Entry::Prop { key, value, line: _ } => match value {
                ScannedValue::Literal(v) => props.push((key, v)),
                ScannedValue::Unsupported(u) => d.unsupported.push(u),
            },
            Entry::Node {
                head,
                name,
                    body,
                line,
            } => children.push(raw_node_from(head, name, body, line, d)),
            Entry::Unsupported(u) => d.unsupported.push(u),
        }
    }
}
