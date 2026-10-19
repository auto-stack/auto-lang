//! Wire codec for the generation request (PLAN-815 T-03 / SD-02 candidate).
//!
//! Same-source serialization of [`GenerationRequest`] at the data owner: the
//! protocol crate, the `auto-ui-tool` host, and every client share this one
//! codec — nobody re-declares a parallel request structure that could lose
//! fields in either direction (PLAN-815 §5.5 "复用 809 单源数据/codec").
//!
//! Shape: a JSON object `{"wire": <version>, "request": { … }}`. Every
//! request field has a fixed, named slot; unknown keys are rejected on
//! decode (codec drift must fail loudly, not silently drop). `auto_val`
//! values are tag-encoded (`{"t":"str","v":…}`) so variants survive a
//! round-trip exactly; variants that have no honest JSON representation
//! (closures, widgets, borrowed slices, …) are rejected with
//! [`WireError::UnsupportedValue`] — never silently coerced.
//!
//! The codec is pure data transformation: no FS, no network, no evaluation.

use crate::descriptor::{
    DisplayFields, Identity, RawNodeDecl, RawProp, RenderDecl, RenderSpec, RuntimeFields,
    SceneKind, SourceInfo, ToggleFields, ToggleValue, WindowSpec,
};
use crate::generation_request::{
    DepRoot, GenerationRequest, PackageAssets, RequestMode, RequestOrigin, ThemeDeclLite,
};
use crate::UnsupportedKind;
use auto_val::{Obj, Value, ValueKey};
use serde_json::{json, Map, Number, Value as Json};

/// Wire format version of this codec's emitted shape.
pub const WIRE_FORMAT_VERSION: u32 = 1;

/// Codec failure — every arm names what was lost or malformed; nothing is
/// silently dropped.
#[derive(Debug, Clone, PartialEq)]
pub enum WireError {
    /// A `Value` variant with no honest JSON representation was carried in
    /// the request (closures, widgets, borrowed slices, …).
    UnsupportedValue { variant: &'static str },
    /// An object key variant with no honest JSON representation.
    UnsupportedKey { variant: &'static str },
    /// A float that JSON cannot carry (NaN / Infinity).
    NonFiniteFloat,
    /// A JSON node with the wrong type or an unknown/missing key.
    Shape { field: &'static str, reason: String },
    /// A tag-encoded node whose tag this codec version does not know.
    UnknownTag { tag: String },
}

impl std::fmt::Display for WireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WireError::UnsupportedValue { variant } => {
                write!(f, "value variant '{variant}' has no wire representation")
            }
            WireError::UnsupportedKey { variant } => {
                write!(f, "object key variant '{variant}' has no wire representation")
            }
            WireError::NonFiniteFloat => write!(f, "non-finite float cannot cross the wire"),
            WireError::Shape { field, reason } => write!(f, "field '{field}': {reason}"),
            WireError::UnknownTag { tag } => write!(f, "unknown wire tag '{tag}'"),
        }
    }
}

impl std::error::Error for WireError {}

// ---------------------------------------------------------------------------
// Value / ValueKey
// ---------------------------------------------------------------------------

fn value_to_wire(value: &Value) -> Result<Json, WireError> {
    match value {
        Value::Nil => Ok(json!({"t": "nil"})),
        Value::Null => Ok(json!({"t": "null"})),
        Value::Bool(b) => Ok(json!({"t": "bool", "v": b})),
        Value::Byte(v) => Ok(json!({"t": "byte", "v": v})),
        Value::Int(v) => Ok(json!({"t": "int", "v": v})),
        Value::Uint(v) => Ok(json!({"t": "uint", "v": v})),
        Value::USize(v) => Ok(json!({"t": "usize", "v": v})),
        Value::I8(v) => Ok(json!({"t": "i8", "v": v})),
        Value::U8(v) => Ok(json!({"t": "u8", "v": v})),
        Value::I64(v) => Ok(json!({"t": "i64", "v": v})),
        Value::Float(v) | Value::Double(v) => {
            let tag = if matches!(value, Value::Float(_)) { "float" } else { "double" };
            if v.is_finite() {
                Ok(json!({"t": tag, "v": v}))
            } else {
                Err(WireError::NonFiniteFloat)
            }
        }
        Value::Char(c) => Ok(json!({"t": "char", "v": c.to_string()})),
        Value::Str(s) => Ok(json!({"t": "str", "v": s.as_str()})),
        Value::String(s) => Ok(json!({"t": "string", "v": s.as_str()})),
        Value::CStr(s) => Ok(json!({"t": "cstr", "v": s.as_str()})),
        Value::Array(a) => Ok(json!({"t": "array", "v": value_slice_to_wire(&a.values)?})),
        Value::Block(a) => Ok(json!({"t": "block", "v": value_slice_to_wire(&a.values)?})),
        Value::Obj(o) => {
            let mut entries = Vec::with_capacity(o.len());
            for (k, v) in o.iter() {
                entries.push(json!([key_to_wire(k)?, value_to_wire(v)?]));
            }
            Ok(json!({"t": "obj", "e": entries}))
        }
        Value::Pair(k, v) => Ok(json!({"t": "pair", "k": key_to_wire(k)?, "v": value_to_wire(v)?})),
        other => Err(WireError::UnsupportedValue {
            variant: value_variant_name(other),
        }),
    }
}

fn value_slice_to_wire(values: &[Value]) -> Result<Json, WireError> {
    let items: Vec<Json> = values.iter().map(value_to_wire).collect::<Result<_, _>>()?;
    Ok(Json::Array(items))
}

fn value_variant_name(value: &Value) -> &'static str {
    match value {
        Value::Byte(_) => "Byte",
        Value::Int(_) => "Int",
        Value::Uint(_) => "Uint",
        Value::USize(_) => "USize",
        Value::I8(_) => "I8",
        Value::U8(_) => "U8",
        Value::I64(_) => "I64",
        Value::Float(_) => "Float",
        Value::Double(_) => "Double",
        Value::Bool(_) => "Bool",
        Value::Char(_) => "Char",
        Value::Str(_) => "Str",
        Value::String(_) => "String",
        Value::StrSlice(_) => "StrSlice",
        Value::CStr(_) => "CStr",
        Value::Array(_) => "Array",
        Value::Block(_) => "Block",
        Value::Pair(_, _) => "Pair",
        Value::Obj(_) => "Obj",
        Value::Node(_) => "Node",
        Value::Range(..) => "Range",
        Value::RangeEq(..) => "RangeEq",
        Value::Fn(_) => "Fn",
        Value::ExtFn(_) => "ExtFn",
        Value::Type(_) => "Type",
        Value::Nil => "Nil",
        Value::Null => "Null",
        Value::Lambda(_) => "Lambda",
        Value::Void => "Void",
        Value::Widget(_) => "Widget",
        Value::Model(_) => "Model",
        Value::View(_) => "View",
        Value::Meta(_) => "Meta",
        Value::Method(_) => "Method",
        Value::Instance(_) => "Instance",
        Value::Args(_) => "Args",
        Value::Ref(_) => "Ref",
        Value::Error(_) => "Error",
        Value::Grid(_) => "Grid",
        Value::VmRef(_) => "VmRef",
        Value::ValueRef(_) => "ValueRef",
        Value::Closure(_) => "Closure",
        Value::Some(_) => "Some",
        Value::None => "None",
        Value::Ok(_) => "Ok",
        Value::Err(_) => "Err",
        Value::Future(_) => "Future",
    }
}

fn key_to_wire(key: &ValueKey) -> Result<Json, WireError> {
    match key {
        ValueKey::Str(s) => Ok(json!({"k": "str", "v": s.as_str()})),
        ValueKey::Int(i) => Ok(json!({"k": "int", "v": i})),
        ValueKey::Bool(b) => Ok(json!({"k": "bool", "v": b})),
    }
}

fn value_from_wire(json: &Json) -> Result<Value, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "value",
        reason: format!("expected tag object, got {}", json_type(json)),
    })?;
    let allowed = match obj.get("t").and_then(Json::as_str) {
        Some("nil") | Some("null") => obj.len() == 1,
        Some("obj") => obj.len() == 2 && obj.contains_key("e"),
        Some("pair") => obj.len() == 3 && obj.contains_key("k") && obj.contains_key("v"),
        _ => obj.len() == 2 && obj.contains_key("v"),
    };
    if !allowed {
        return Err(WireError::Shape {
            field: "value",
            reason: "expected exactly {'t','v'} / {'t','e'} / {'t','k','v'}".into(),
        });
    }
    let tag = obj["t"].as_str().ok_or_else(|| WireError::Shape {
        field: "value.t",
        reason: "tag must be a string".into(),
    })?;
    match tag {
        "nil" => Ok(Value::Nil),
        "null" => Ok(Value::Null),
        "bool" => Ok(Value::Bool(payload_bool(obj)?)),
        "byte" => Ok(Value::Byte(payload_u64(obj)? as u8)),
        "int" => Ok(Value::Int(payload_i64(obj)?.try_into().map_err(|_| payload_range("int"))?)),
        "uint" => Ok(Value::Uint(payload_u64(obj)?.try_into().map_err(|_| payload_range("uint"))?)),
        "usize" => Ok(Value::USize(payload_u64(obj)? as usize)),
        "i8" => Ok(Value::I8(payload_i64(obj)?.try_into().map_err(|_| payload_range("i8"))?)),
        "u8" => Ok(Value::U8(payload_u64(obj)?.try_into().map_err(|_| payload_range("u8"))?)),
        "i64" => Ok(Value::I64(payload_i64(obj)?)),
        "float" | "double" => {
            let n = payload_f64(obj)?;
            Ok(if tag == "float" { Value::Float(n) } else { Value::Double(n) })
        }
        "char" => {
            let s = payload_str(obj)?;
            let mut chars = s.chars();
            let c = chars.next().ok_or_else(|| WireError::Shape {
                field: "value",
                reason: "char tag carries an empty string".into(),
            })?;
            if chars.next().is_some() {
                return Err(WireError::Shape {
                    field: "value",
                    reason: "char tag must carry exactly one character".into(),
                });
            }
            Ok(Value::Char(c))
        }
        "str" => Ok(Value::Str(payload_str(obj)?.into())),
        "string" => Ok(Value::String(payload_str(obj)?.into())),
        "cstr" => Ok(Value::CStr(auto_val::CStr::from_str(payload_str(obj)?))),
        "array" | "block" => {
            let items = arr_key(obj, "v")?;
            let mut values = Vec::with_capacity(items.len());
            for item in items {
                values.push(value_from_wire(item)?);
            }
            let array = auto_val::Array { values };
            Ok(if tag == "array" { Value::Array(array) } else { Value::Block(array) })
        }
        "obj" => {
            let entries = arr_key(obj, "e")?;
            let mut object = Obj::new();
            for entry in entries {
                let pair = entry.as_array().ok_or_else(|| WireError::Shape {
                    field: "value",
                    reason: "obj entry must be a [key, value] pair".into(),
                })?;
                if pair.len() != 2 {
                    return Err(WireError::Shape {
                        field: "value",
                        reason: "obj entry must have exactly two elements".into(),
                    });
                }
                let key = key_from_wire(&pair[0])?;
                let value = value_from_wire(&pair[1])?;
                object.set(key, value);
            }
            Ok(Value::obj(object))
        }
        "pair" => {
            let key = obj.get("k").ok_or_else(|| WireError::Shape {
                field: "value",
                reason: "pair tag requires 'k'".into(),
            })?;
            let value = obj.get("v").ok_or_else(|| WireError::Shape {
                field: "value",
                reason: "pair tag requires 'v'".into(),
            })?;
            Ok(Value::Pair(key_from_wire(key)?, Box::new(value_from_wire(value)?)))
        }
        other => Err(WireError::UnknownTag { tag: other.to_string() }),
    }
}

fn key_from_wire(json: &Json) -> Result<ValueKey, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "value-key",
        reason: format!("expected tag object, got {}", json_type(json)),
    })?;
    if obj.len() != 2 || !obj.contains_key("k") || !obj.contains_key("v") {
        return Err(WireError::Shape {
            field: "value-key",
            reason: "expected exactly {'k', 'v'}".into(),
        });
    }
    match obj["k"].as_str() {
        Some("str") => Ok(ValueKey::Str(payload_str(obj)?.into())),
        Some("int") => Ok(ValueKey::Int(
            payload_i64(obj)?.try_into().map_err(|_| WireError::Shape {
                field: "value-key",
                reason: "int key out of range".into(),
            })?,
        )),
        Some("bool") => Ok(ValueKey::Bool(payload_bool(obj)?)),
        Some(other) => Err(WireError::UnknownTag { tag: other.to_string() }),
        None => Err(WireError::Shape {
            field: "value-key.k",
            reason: "key tag must be a string".into(),
        }),
    }
}

// Tag-payload accessors: the payload slot of a tag object is literally the
// key "v" (never a dotted field path).
fn payload<'a>(obj: &'a Map<String, Json>) -> Result<&'a Json, WireError> {
    obj.get("v").ok_or_else(|| WireError::Shape {
        field: "value",
        reason: "missing 'v' payload".into(),
    })
}

fn payload_range(tag: &str) -> WireError {
    WireError::Shape {
        field: "value",
        reason: format!("'{tag}' payload out of range"),
    }
}

fn payload_bool(obj: &Map<String, Json>) -> Result<bool, WireError> {
    payload(obj)?
        .as_bool()
        .ok_or_else(|| WireError::Shape { field: "value", reason: "'v' must be a bool".into() })
}

fn payload_str<'a>(obj: &'a Map<String, Json>) -> Result<&'a str, WireError> {
    payload(obj)?
        .as_str()
        .ok_or_else(|| WireError::Shape { field: "value", reason: "'v' must be a string".into() })
}

fn payload_u64(obj: &Map<String, Json>) -> Result<u64, WireError> {
    payload(obj)?
        .as_u64()
        .ok_or_else(|| WireError::Shape {
            field: "value",
            reason: "'v' must be a non-negative integer".into(),
        })
}

fn payload_i64(obj: &Map<String, Json>) -> Result<i64, WireError> {
    payload(obj)?
        .as_i64()
        .ok_or_else(|| WireError::Shape { field: "value", reason: "'v' must be an integer".into() })
}

fn payload_f64(obj: &Map<String, Json>) -> Result<f64, WireError> {
    payload(obj)?
        .as_f64()
        .ok_or_else(|| WireError::Shape { field: "value", reason: "'v' must be a number".into() })
}

// ---------------------------------------------------------------------------
// Request <-> JSON
// ---------------------------------------------------------------------------

/// Serialize a request to its wire form.
pub fn request_to_wire(request: &GenerationRequest) -> Result<Json, WireError> {
    let mut root = Map::new();
    root.insert("wire".into(), json!(WIRE_FORMAT_VERSION));
    root.insert("request".into(), request_fields_to_wire(request)?);
    Ok(Json::Object(root))
}

/// Deserialize a request from its wire form. Unknown keys are rejected.
pub fn request_from_wire(json: &Json) -> Result<GenerationRequest, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "root",
        reason: format!("expected object, got {}", json_type(json)),
    })?;
    if obj.len() != 2 {
        return Err(WireError::Shape {
            field: "root",
            reason: "expected exactly {'wire', 'request'}".into(),
        });
    }
    let version = obj
        .get("wire")
        .and_then(Json::as_u64)
        .ok_or_else(|| WireError::Shape {
            field: "wire",
            reason: "missing or non-numeric wire version".into(),
        })?;
    if version != WIRE_FORMAT_VERSION as u64 {
        return Err(WireError::Shape {
            field: "wire",
            reason: format!("unsupported wire version {version} (codec speaks {WIRE_FORMAT_VERSION})"),
        });
    }
    request_fields_from_wire(
        obj.get("request")
            .ok_or_else(|| WireError::Shape { field: "request", reason: "missing".into() })?,
    )
}

fn request_fields_to_wire(request: &GenerationRequest) -> Result<Json, WireError> {
    let origin = match &request.origin {
        RequestOrigin::StaticAdmitted { source, unsupported } => json!({
            "kind": "static",
            "source": source_to_wire(source),
            "unsupported": unsupported.iter().map(|k| unsupported_kind_tag(*k)).collect::<Vec<_>>(),
        }),
        RequestOrigin::DynamicEvaluated { source } => json!({
            "kind": "dynamic",
            "source": source_to_wire(source),
        }),
    };
    let raw = raw_props_to_wire(&request.raw)?;
    let raw_nodes = raw_nodes_to_wire(&request.raw_nodes)?;
    Ok(json!({
        "origin": origin,
        "project_root": opt_path(&request.project_root),
        "identity": {
            "name": opt_str(&request.identity.name),
            "version": opt_str(&request.identity.version),
            "lang": opt_str(&request.identity.lang),
        },
        "display": {
            "title": opt_str(&request.display.title),
            "title_zh": opt_str(&request.display.title_zh),
            "icon": opt_str(&request.display.icon),
            "category": opt_str(&request.display.category),
            "opens": opt_str(&request.display.opens),
            "theme": opt_str(&request.display.theme),
            "accent": opt_str(&request.display.accent),
            "desktop_render": opt_str(&request.display.desktop_render),
            "daemon": opt_str(&request.display.daemon),
            "desktop": opt_str(&request.display.desktop),
            "desktop_exe": opt_str(&request.display.desktop_exe),
            "ui_config": opt_str(&request.display.ui_config),
        },
        "runtime": {
            "api": opt_str(&request.runtime.api),
            "scene": request.runtime.scene.map(scene_tag),
            "front_port": request.runtime.front_port,
            "back_port": request.runtime.back_port,
            "media_root": opt_str(&request.runtime.media_root),
            "photo_root": opt_str(&request.runtime.photo_root),
            "window": window_to_wire(&request.runtime.window),
        },
        "scene": scene_tag(request.scene),
        "members": request.members,
        "render": {
            "render": render_spec_to_wire(&request.render.render),
            "legacy_backend": opt_str(&request.render.legacy_backend),
            "effective_backend": opt_str(&request.render.effective_backend),
        },
        "external_backend": opt_str(&request.external_backend),
        "toggles": {
            "shadcn": toggle_to_wire(&request.toggles.shadcn),
            "default_classes": toggle_to_wire(&request.toggles.default_classes),
        },
        "theme_decl": match &request.theme_decl {
            None => Json::Null,
            Some(t) => json!({
                "name": opt_str(&t.name),
                "extends": opt_str(&t.extends),
                "mode": opt_str(&t.mode),
                "colors": t.colors.iter()
                    .map(|(k, v)| json!([k, v]))
                    .collect::<Vec<_>>(),
            }),
        },
        "dep_roots": request.dep_roots.iter().map(|d| json!({
            "name": d.name,
            "path": opt_str(&d.path),
        })).collect::<Vec<_>>(),
        "package_assets": {
            "npm_deps": opt_value(&request.package_assets.npm_deps)?,
            "styles": opt_value(&request.package_assets.styles)?,
            "i18n": opt_value(&request.package_assets.i18n)?,
        },
        "exe_name": opt_str(&request.exe_name),
        "has_rust_sidecar": request.has_rust_sidecar,
        "front_decl": opt_str(&request.front_decl),
        "front_node_form": opt_str(&request.front_node_form),
        "input_sources": request.input_sources.iter()
            .map(|p| json!(path_str(p)))
            .collect::<Vec<_>>(),
        "output_root": opt_path(&request.output_root),
        "mode": match request.mode {
            RequestMode::Generate => "generate",
            RequestMode::Run => "run",
        },
        "raw": raw,
        "raw_nodes": raw_nodes,
    }))
}

fn request_fields_from_wire(json: &Json) -> Result<GenerationRequest, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "request",
        reason: format!("expected object, got {}", json_type(json)),
    })?;
    let mut seen: std::collections::HashSet<&str> = Default::default();
    for key in obj.keys() {
        if !seen.insert(key) || !REQUEST_FIELDS.contains(&key.as_str()) {
            return Err(WireError::Shape {
                field: "request",
                reason: format!("unknown request key '{key}'"),
            });
        }
    }
    for required in REQUEST_FIELDS {
        if !seen.contains(required) {
            return Err(WireError::Shape {
                field: "request",
                reason: format!("missing request key '{required}'"),
            });
        }
    }

    let origin_obj = field_obj(obj, "origin")?;
    let origin = match str_field(origin_obj, "origin.kind")? {
        "static" => RequestOrigin::StaticAdmitted {
            source: source_from_wire(field_obj(origin_obj, "origin.source")?, "origin.source")?,
            unsupported: field_arr(origin_obj, "origin.unsupported")?
                .iter()
                .map(|t| unsupported_kind_from_tag(str_of(t, "origin.unsupported")?))
                .collect::<Result<Vec<_>, _>>()?,
        },
        "dynamic" => RequestOrigin::DynamicEvaluated {
            source: source_from_wire(field_obj(origin_obj, "origin.source")?, "origin.source")?,
        },
        other => {
            return Err(WireError::UnknownTag { tag: other.to_string() });
        }
    };

    let identity_obj = field_obj(obj, "identity")?;
    let display_obj = field_obj(obj, "display")?;
    let runtime_obj = field_obj(obj, "runtime")?;
    let render_obj = field_obj(obj, "render")?;
    let toggles_obj = field_obj(obj, "toggles")?;
    let assets_obj = field_obj(obj, "package_assets")?;

    let theme_decl = match obj.get("theme_decl") {
        Some(Json::Null) | None => None,
        Some(t) => {
            let t = t.as_object().ok_or_else(|| WireError::Shape {
                field: "theme_decl",
                reason: "expected object or null".into(),
            })?;
            let colors = field_arr(t, "theme_decl.colors")?
                .iter()
                .map(|pair| -> Result<(String, String), WireError> {
                    let pair = pair.as_array().filter(|a| a.len() == 2).ok_or_else(|| {
                        WireError::Shape {
                            field: "theme_decl.colors",
                            reason: "expected [key, value] pairs".into(),
                        }
                    })?;
                    Ok((
                        str_of(&pair[0], "theme_decl.colors")?.to_string(),
                        str_of(&pair[1], "theme_decl.colors")?.to_string(),
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Some(ThemeDeclLite {
                name: opt_str_field(t, "name")?,
                extends: opt_str_field(t, "extends")?,
                mode: opt_str_field(t, "mode")?,
                colors,
            })
        }
    };

    let window = match runtime_obj.get("window") {
        Some(Json::Null) | None => None,
        Some(w) => Some(window_from_wire(w)?),
    };
    let runtime_scene = match runtime_obj.get("scene") {
        Some(Json::Null) | None => None,
        Some(s) => Some(scene_from_tag(str_of(s, "runtime.scene")?)?),
    };

    Ok(GenerationRequest {
        origin,
        project_root: opt_path_field(obj, "project_root")?,
        identity: Identity {
            name: opt_str_field(identity_obj, "name")?,
            version: opt_str_field(identity_obj, "version")?,
            lang: opt_str_field(identity_obj, "lang")?,
        },
        display: DisplayFields {
            title: opt_str_field(display_obj, "title")?,
            title_zh: opt_str_field(display_obj, "title_zh")?,
            icon: opt_str_field(display_obj, "icon")?,
            category: opt_str_field(display_obj, "category")?,
            opens: opt_str_field(display_obj, "opens")?,
            theme: opt_str_field(display_obj, "theme")?,
            accent: opt_str_field(display_obj, "accent")?,
            desktop_render: opt_str_field(display_obj, "desktop_render")?,
            daemon: opt_str_field(display_obj, "daemon")?,
            desktop: opt_str_field(display_obj, "desktop")?,
            desktop_exe: opt_str_field(display_obj, "desktop_exe")?,
            ui_config: opt_str_field(display_obj, "ui_config")?,
        },
        runtime: RuntimeFields {
            api: opt_str_field(runtime_obj, "api")?,
            scene: runtime_scene,
            front_port: opt_u16_field(runtime_obj, "front_port")?,
            back_port: opt_u16_field(runtime_obj, "back_port")?,
            media_root: opt_str_field(runtime_obj, "media_root")?,
            photo_root: opt_str_field(runtime_obj, "photo_root")?,
            window,
        },
        scene: scene_from_tag(str_of(obj.get("scene").unwrap_or(&Json::Null), "scene")?)?,
        members: str_list(obj, "members")?,
        render: RenderDecl {
            render: render_spec_from_wire(field_json(render_obj, "render.render")?)?,
            legacy_backend: opt_str_field(render_obj, "legacy_backend")?,
            effective_backend: opt_str_field(render_obj, "effective_backend")?,
        },
        external_backend: opt_str_field(obj, "external_backend")?,
        toggles: ToggleFields {
            shadcn: toggle_from_wire(field_json(toggles_obj, "toggles.shadcn")?)?,
            default_classes: toggle_from_wire(field_json(toggles_obj, "toggles.default_classes")?)?,
        },
        theme_decl,
        dep_roots: field_arr(obj, "dep_roots")?
            .iter()
            .map(|d| -> Result<DepRoot, WireError> {
                let d = d.as_object().ok_or_else(|| WireError::Shape {
                    field: "dep_roots",
                    reason: "expected objects".into(),
                })?;
                Ok(DepRoot {
                    name: str_field(d, "dep_roots.name")?.to_string(),
                    path: opt_str_field(d, "path")?,
                })
            })
            .collect::<Result<Vec<_>, _>>()?,
        package_assets: PackageAssets {
            npm_deps: opt_value_field(assets_obj, "npm_deps")?,
            styles: opt_value_field(assets_obj, "styles")?,
            i18n: opt_value_field(assets_obj, "i18n")?,
        },
        exe_name: opt_str_field(obj, "exe_name")?,
        has_rust_sidecar: bool_field(obj, "has_rust_sidecar")?,
        front_decl: opt_str_field(obj, "front_decl")?,
        front_node_form: opt_str_field(obj, "front_node_form")?,
        input_sources: field_arr(obj, "input_sources")?
            .iter()
            .map(|p| Ok(PathBuf::from(str_of(p, "input_sources")?)))
            .collect::<Result<Vec<_>, WireError>>()?,
        output_root: opt_path_field(obj, "output_root")?,
        mode: match str_of(obj.get("mode").unwrap_or(&Json::Null), "mode")? {
            "generate" => RequestMode::Generate,
            "run" => RequestMode::Run,
            other => return Err(WireError::UnknownTag { tag: other.to_string() }),
        },
        raw: raw_props_from_wire(field_arr(obj, "raw")?)?,
        raw_nodes: raw_nodes_from_wire(field_arr(obj, "raw_nodes")?)?,
    })
}

const REQUEST_FIELDS: &[&str] = &[
    "origin",
    "project_root",
    "identity",
    "display",
    "runtime",
    "scene",
    "members",
    "render",
    "external_backend",
    "toggles",
    "theme_decl",
    "dep_roots",
    "package_assets",
    "exe_name",
    "has_rust_sidecar",
    "front_decl",
    "front_node_form",
    "input_sources",
    "output_root",
    "mode",
    "raw",
    "raw_nodes",
];

fn raw_props_to_wire(props: &[RawProp]) -> Result<Json, WireError> {
    let items = props
        .iter()
        .map(|p| {
            Ok(json!({
                "key": p.key,
                "value": opt_value(&p.value)?,
                "line": p.line,
            }))
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    Ok(Json::Array(items))
}

fn raw_props_from_wire(json: &[Json]) -> Result<Vec<RawProp>, WireError> {
    json.iter()
        .map(|p| {
            let obj = p.as_object().ok_or_else(|| WireError::Shape {
                field: "raw",
                reason: "expected objects".into(),
            })?;
            Ok(RawProp {
                key: str_field(obj, "raw.key")?.to_string(),
                value: opt_value_field(obj, "value")?,
                line: usize_field(obj, "raw.line")?,
            })
        })
        .collect()
}

fn raw_nodes_to_wire(nodes: &[RawNodeDecl]) -> Result<Json, WireError> {
    let items = nodes
        .iter()
        .map(|n| {
            let props = n
                .props
                .iter()
                .map(|p| {
                    Ok(json!({
                        "key": p.key,
                        "value": value_to_wire(&p.value)?,
                        "line": p.line,
                    }))
                })
                .collect::<Result<Vec<_>, WireError>>()?;
            Ok(json!({
                "head": n.head,
                "name": opt_str(&n.name),
                "raw_args": opt_str(&n.raw_args),
                "props": props,
                "children": raw_nodes_to_wire(&n.children)?,
                "line": n.line,
            }))
        })
        .collect::<Result<Vec<_>, WireError>>()?;
    Ok(Json::Array(items))
}

fn raw_nodes_from_wire(json: &[Json]) -> Result<Vec<RawNodeDecl>, WireError> {
    json.iter()
        .map(|n| {
            let obj = n.as_object().ok_or_else(|| WireError::Shape {
                field: "raw_nodes",
                reason: "expected objects".into(),
            })?;
            let props = field_arr(obj, "raw_nodes.props")?
                .iter()
                .map(|p| -> Result<crate::descriptor::NodeProp, WireError> {
                    let p = p.as_object().ok_or_else(|| WireError::Shape {
                        field: "raw_nodes.props",
                        reason: "expected objects".into(),
                    })?;
                    Ok(crate::descriptor::NodeProp {
                        key: str_field(p, "props.key")?.to_string(),
                        value: value_from_wire(field_json(p, "props.value")?)?,
                        line: usize_field(p, "props.line")?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(RawNodeDecl {
                head: str_field(obj, "raw_nodes.head")?.to_string(),
                name: opt_str_field(obj, "name")?,
                raw_args: opt_str_field(obj, "raw_args")?,
                props,
                children: raw_nodes_from_wire(field_arr(obj, "raw_nodes.children")?)?,
                line: usize_field(obj, "raw_nodes.line")?,
            })
        })
        .collect()
}

fn unsupported_kind_tag(kind: UnsupportedKind) -> &'static str {
    match kind {
        UnsupportedKind::Interpolation => "interpolation",
        UnsupportedKind::FString => "f-string",
        UnsupportedKind::BareIdentValue => "bare-ident-value",
        UnsupportedKind::Assignment => "assignment",
        UnsupportedKind::Call => "call",
        UnsupportedKind::OnOffOutsideToggle => "on-off-outside-toggle",
        UnsupportedKind::SingleQuotedString => "single-quoted-string",
        UnsupportedKind::DepNameUnrepresentable => "dep-name-unrepresentable",
        UnsupportedKind::RootWrapper => "root-wrapper",
        UnsupportedKind::ValueSyntax => "value-syntax",
        UnsupportedKind::TypeMismatch { .. } => "type-mismatch",
    }
}

fn unsupported_kind_from_tag(tag: &str) -> Result<UnsupportedKind, WireError> {
    match tag {
        "interpolation" => Ok(UnsupportedKind::Interpolation),
        "f-string" => Ok(UnsupportedKind::FString),
        "bare-ident-value" => Ok(UnsupportedKind::BareIdentValue),
        "assignment" => Ok(UnsupportedKind::Assignment),
        "call" => Ok(UnsupportedKind::Call),
        "on-off-outside-toggle" => Ok(UnsupportedKind::OnOffOutsideToggle),
        "single-quoted-string" => Ok(UnsupportedKind::SingleQuotedString),
        "dep-name-unrepresentable" => Ok(UnsupportedKind::DepNameUnrepresentable),
        "root-wrapper" => Ok(UnsupportedKind::RootWrapper),
        "value-syntax" => Ok(UnsupportedKind::ValueSyntax),
        "type-mismatch" => Ok(UnsupportedKind::TypeMismatch { field: "?" }),
        other => Err(WireError::UnknownTag { tag: other.to_string() }),
    }
}

// ---------------------------------------------------------------------------
// Small helpers (strict: wrong types and unknown keys are errors)
// ---------------------------------------------------------------------------

use std::path::{Path, PathBuf};

fn json_type(json: &Json) -> &'static str {
    match json {
        Json::Null => "null",
        Json::Bool(_) => "bool",
        Json::Number(_) => "number",
        Json::String(_) => "string",
        Json::Array(_) => "array",
        Json::Object(_) => "object",
    }
}

fn opt_str(value: &Option<String>) -> Json {
    match value {
        Some(s) => Json::String(s.clone()),
        None => Json::Null,
    }
}

fn path_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn opt_path(path: &Option<PathBuf>) -> Json {
    match path {
        Some(p) => Json::String(path_str(p)),
        None => Json::Null,
    }
}

fn opt_value(value: &Option<Value>) -> Result<Json, WireError> {
    match value {
        Some(v) => value_to_wire(v),
        None => Ok(Json::Null),
    }
}

fn source_to_wire(source: &SourceInfo) -> Json {
    json!({ "path": opt_path(&source.path) })
}

fn source_from_wire(obj: &Map<String, Json>, field: &'static str) -> Result<SourceInfo, WireError> {
    if obj.len() != 1 || !obj.contains_key("path") {
        return Err(WireError::Shape {
            field,
            reason: "expected exactly {'path'}".into(),
        });
    }
    Ok(SourceInfo {
        path: opt_path_field(obj, "path")?,
    })
}

fn scene_tag(scene: SceneKind) -> &'static str {
    match scene {
        SceneKind::Workspace => "workspace",
        SceneKind::Ui => "ui",
        SceneKind::C => "c",
        SceneKind::Default => "default",
    }
}

fn scene_from_tag(tag: &str) -> Result<SceneKind, WireError> {
    match tag {
        "workspace" => Ok(SceneKind::Workspace),
        "ui" => Ok(SceneKind::Ui),
        "c" => Ok(SceneKind::C),
        "default" => Ok(SceneKind::Default),
        other => Err(WireError::UnknownTag { tag: other.to_string() }),
    }
}

fn window_to_wire(window: &Option<WindowSpec>) -> Json {
    match window {
        None => Json::Null,
        Some(WindowSpec::Fit) => json!({"kind": "fit"}),
        Some(WindowSpec::Size(w, h)) => json!({"kind": "size", "w": w, "h": h}),
    }
}

fn window_from_wire(json: &Json) -> Result<WindowSpec, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "runtime.window",
        reason: format!("expected object, got {}", json_type(json)),
    })?;
    match str_field(obj, "runtime.window.kind")? {
        "fit" => Ok(WindowSpec::Fit),
        "size" => Ok(WindowSpec::Size(
            f32_of(field_json(obj, "w")?, "runtime.window.w")? as f32,
            f32_of(field_json(obj, "h")?, "runtime.window.h")? as f32,
        )),
        other => Err(WireError::UnknownTag { tag: other.to_string() }),
    }
}

fn render_spec_to_wire(spec: &RenderSpec) -> Json {
    match spec {
        RenderSpec::Absent => json!({"kind": "absent"}),
        RenderSpec::Single(name) => json!({"kind": "single", "name": name}),
        RenderSpec::Multi(names) => json!({"kind": "multi", "names": names}),
        RenderSpec::Split { front, back } => json!({"kind": "split", "front": front, "back": back}),
        RenderSpec::Unknown(raw) => json!({"kind": "unknown", "raw": raw}),
    }
}

fn render_spec_from_wire(json: &Json) -> Result<RenderSpec, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "render.render",
        reason: format!("expected object, got {}", json_type(json)),
    })?;
    match str_field(obj, "render.render.kind")? {
        "absent" => Ok(RenderSpec::Absent),
        "single" => Ok(RenderSpec::Single(str_field(obj, "render.render.name")?.to_string())),
        "multi" => Ok(RenderSpec::Multi(str_list(obj, "render.render.names")?)),
        "split" => Ok(RenderSpec::Split {
            front: str_list(obj, "render.render.front")?,
            back: str_field(obj, "render.render.back")?.to_string(),
        }),
        "unknown" => Ok(RenderSpec::Unknown(str_field(obj, "render.render.raw")?.to_string())),
        other => Err(WireError::UnknownTag { tag: other.to_string() }),
    }
}

fn toggle_to_wire(toggle: &ToggleValue) -> Json {
    match toggle {
        ToggleValue::Unset => json!({"kind": "unset"}),
        ToggleValue::Bool(b) => json!({"kind": "bool", "v": b}),
        ToggleValue::Str(s) => json!({"kind": "str", "v": s}),
    }
}

fn toggle_from_wire(json: &Json) -> Result<ToggleValue, WireError> {
    let obj = json.as_object().ok_or_else(|| WireError::Shape {
        field: "toggles",
        reason: format!("expected object, got {}", json_type(json)),
    })?;
    match str_field(obj, "toggles.kind")? {
        "unset" => Ok(ToggleValue::Unset),
        "bool" => Ok(ToggleValue::Bool(bool_field(obj, "toggles.v")?)),
        "str" => Ok(ToggleValue::Str(str_field(obj, "toggles.v")?.to_string())),
        other => Err(WireError::UnknownTag { tag: other.to_string() }),
    }
}

fn field_obj<'a>(obj: &'a Map<String, Json>, field: &'static str) -> Result<&'a Map<String, Json>, WireError> {
    obj.get(field.rsplit('.').next().unwrap_or(field))
        .and_then(Json::as_object)
        .ok_or_else(|| WireError::Shape {
            field,
            reason: "expected an object".into(),
        })
}

fn field_json<'a>(obj: &'a Map<String, Json>, field: &'static str) -> Result<&'a Json, WireError> {
    obj.get(field.rsplit('.').next().unwrap_or(field))
        .ok_or_else(|| WireError::Shape {
            field,
            reason: "missing".into(),
        })
}

fn field_arr<'a>(obj: &'a Map<String, Json>, field: &'static str) -> Result<&'a [Json], WireError> {
    obj.get(field.rsplit('.').next().unwrap_or(field))
        .and_then(Json::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| WireError::Shape {
            field,
            reason: "expected an array".into(),
        })
}

/// Direct-key array access for tag payload slots ("v" / "e"), where the
/// lookup key is not derivable from a dotted field path.
fn arr_key<'a>(obj: &'a Map<String, Json>, key: &str) -> Result<&'a [Json], WireError> {
    obj.get(key)
        .and_then(Json::as_array)
        .map(Vec::as_slice)
        .ok_or_else(|| WireError::Shape {
            field: "value",
            reason: format!("tag payload '{key}' must be an array"),
        })
}

fn str_of<'a>(json: &'a Json, field: &'static str) -> Result<&'a str, WireError> {
    json.as_str().ok_or_else(|| WireError::Shape {
        field,
        reason: "expected a string".into(),
    })
}

fn str_field<'a>(obj: &'a Map<String, Json>, field: &'static str) -> Result<&'a str, WireError> {
    str_of(field_json(obj, field)?, field)
}

fn opt_str_field(
    obj: &Map<String, Json>,
    key: &str,
) -> Result<Option<String>, WireError> {
    match obj.get(key) {
        Some(Json::Null) | None => Ok(None),
        Some(Json::String(s)) => Ok(Some(s.clone())),
        Some(other) => Err(WireError::Shape {
            field: "request",
            reason: format!("key '{key}': expected string or null, got {}", json_type(other)),
        }),
    }
}

fn opt_path_field(
    obj: &Map<String, Json>,
    key: &str,
) -> Result<Option<PathBuf>, WireError> {
    Ok(opt_str_field(obj, key)?.map(PathBuf::from))
}

fn opt_value_field(obj: &Map<String, Json>, key: &str) -> Result<Option<Value>, WireError> {
    match obj.get(key) {
        Some(Json::Null) | None => Ok(None),
        Some(v) => value_from_wire(v).map(Some),
    }
}

fn opt_u16_field(obj: &Map<String, Json>, key: &str) -> Result<Option<u16>, WireError> {
    match obj.get(key) {
        Some(Json::Null) | None => Ok(None),
        Some(Json::Number(n)) => u16::try_from(n.as_u64().ok_or_else(|| WireError::Shape {
            field: "request",
            reason: format!("key '{key}': expected u16 number"),
        })?)
        .map(Some)
        .map_err(|_| WireError::Shape {
            field: "request",
            reason: format!("key '{key}': value out of u16 range"),
        }),
        Some(other) => Err(WireError::Shape {
            field: "request",
            reason: format!("key '{key}': expected number or null, got {}", json_type(other)),
        }),
    }
}

fn str_list(obj: &Map<String, Json>, field: &'static str) -> Result<Vec<String>, WireError> {
    field_arr(obj, field)?
        .iter()
        .map(|s| Ok(str_of(s, field)?.to_string()))
        .collect()
}

fn bool_field(obj: &Map<String, Json>, field: &'static str) -> Result<bool, WireError> {
    field_json(obj, field)?
        .as_bool()
        .ok_or_else(|| WireError::Shape {
            field,
            reason: "expected a bool".into(),
        })
}

fn usize_field(obj: &Map<String, Json>, field: &'static str) -> Result<usize, WireError> {
    field_json(obj, field)?
        .as_u64()
        .map(|v| v as usize)
        .ok_or_else(|| WireError::Shape {
            field,
            reason: "expected a non-negative integer".into(),
        })
}

fn f64_of(json: &Json, field: &'static str) -> Result<f64, WireError> {
    json.as_f64().ok_or_else(|| WireError::Shape {
        field,
        reason: "expected a number".into(),
    })
}

fn f32_of(json: &Json, field: &'static str) -> Result<f32, WireError> {
    f64_of(json, field).map(|v| v as f32)
}

// Keep Number construction referenced (unused-warning guard for json! paths
// that construct numbers from f64 in tests only).
#[allow(dead_code)]
fn number_from_f64(v: f64) -> Option<Number> {
    Number::from_f64(v)
}
