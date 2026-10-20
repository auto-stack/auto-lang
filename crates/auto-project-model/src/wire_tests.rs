//! Wire codec tests (PLAN-815 T-03): round-trip over the full request face,
//! field preservation, and honest rejection of unrepresentable values.

use crate::descriptor::SourceInfo;
use crate::generation_request::{DepRoot, PackageAssets, RequestMode, ThemeDeclLite};
use crate::wire::{request_from_wire, request_to_wire, WireError};
use crate::{parse_document, GenerationRequest, RequestOrigin, UnsupportedKind};
use auto_val::{Array, Obj, Value};

fn static_request(text: &str) -> GenerationRequest {
    let descriptor = parse_document(text, SourceInfo::from_path("proj/pac.at")).expect("parse");
    GenerationRequest::from_descriptor(&descriptor)
}

fn round_trip(request: &GenerationRequest) -> GenerationRequest {
    let wire = request_to_wire(request).expect("encode");
    request_from_wire(&wire).expect("decode")
}

/// Rich static request → wire → back: every field survives exactly.
#[test]
fn round_trip_rich_static_request() {
    let mut req = static_request(
        r##"
        name: "联系 表单"
        version: "1.2.3"
        render: ["vue", "jet"]
        backend: "rust"
        title: "Contact"
        title_zh: "联系表单"
        icon: "icon.png"
        category: "demo"
        opens: "window"
        theme: "dark"
        accent: "blue"
        desktop_render: "gl"
        daemon: "off"
        desktop: "yes"
        desktop_exe: "app.exe"
        ui_config: "ui.json"
        api: "rust"
        scene: "ui"
        front_port: 3018
        back_port: 3019
        media_root: "media"
        photo_root: "photos"
        window: "1280x720"
        members: ["a", "b"]
        exe_name: "my-app"
        front: "./front"
        shadcn: off
        default_classes: on
        npm_deps: { vue: "^3.4", pinia: "2" }
        styles: ["dark.css", "light.css"]
        i18n: ["zh", "en"]
        theme: { name: "ocean", extends: "base", mode: "Dark", colors: { primary: "#0366d6" } }
        dep "common" {
            path: "../common"
        }
        dep "extra" { }
        unknown_key: { nested: [1, 2, 3] }
        rust_sidecar("tools") {
            modules: "m1"
        }
        "##,
    );
    // The duplicate `theme` declaration means raw_props carries both — the
    // modeled theme_decl takes the LAST declaration (from_descriptor rule).
    req.mode = RequestMode::Run;
    req.output_root = Some(std::path::PathBuf::from("out/交付"));
    req.front_node_form = Some("./front".into());
    req.input_sources = vec![
        std::path::PathBuf::from("proj/front/main.at"),
        std::path::PathBuf::from("proj/front/app.at"),
    ];

    let back = round_trip(&req);
    assert_eq!(back, req);
}

/// Dynamic arm with every toggle/render shape and numeric value variant.
#[test]
fn round_trip_dynamic_request_value_variants() {
    let mut req = GenerationRequest::dynamic(SourceInfo::from_path("proj/pac.at"));
    req.identity.name = Some("dyn".into());
    req.mode = RequestMode::Run;
    let mut npm = Obj::new();
    npm.set("byte", Value::Byte(7));
    npm.set("int", Value::Int(-42));
    npm.set("uint", Value::Uint(42));
    npm.set("usize", Value::USize(usize::MAX - 3));
    npm.set("i8", Value::I8(-128));
    npm.set("u8", Value::U8(255));
    npm.set("i64", Value::I64(i64::MIN + 5));
    npm.set("float", Value::Float(0.25));
    npm.set("double", Value::Double(1e300));
    npm.set("char", Value::Char('中'));
    npm.set("str", Value::Str("值".into()));
    npm.set("string", Value::String("owned".into()));
    npm.set("cstr", Value::CStr(auto_val::CStr::from_str("c")));
    npm.set("nil", Value::Nil);
    npm.set("null", Value::Null);
    npm.set(
        "arr",
        Value::Array(Array {
            values: vec![Value::Bool(true), Value::Int(9)],
        }),
    );
    npm.set(
        "block",
        Value::Block(Array {
            values: vec![Value::Str("x".into())],
        }),
    );
    let mut inner = Obj::new();
    inner.set(1, Value::Int(1));
    inner.set(true, Value::Int(2));
    npm.set("obj", Value::obj(inner));
    npm.set("pair", Value::Pair("k".into(), Box::new(Value::Int(3))));
    req.package_assets = PackageAssets {
        npm_deps: Some(Value::obj(npm)),
        styles: None,
        i18n: Some(Value::Str("zh".into())),
    };
    req.theme_decl = Some(ThemeDeclLite {
        name: Some("t".into()),
        extends: None,
        mode: Some("dark".into()),
        colors: vec![("a".into(), "b".into())],
    });
    req.dep_roots = vec![DepRoot {
        name: "d".into(),
        path: Some("../d".into()),
    }];
    req.has_rust_sidecar = true;

    let back = round_trip(&req);
    assert_eq!(back, req);
}

/// Static admission with unsupported constructs keeps origin + unsupported
/// list across the wire (no fake evaluation).
#[test]
fn round_trip_preserves_unsupported_origin() {
    let req = static_request("title: \"a${name}b\"\nname: \"x\"\n");
    assert!(!req.is_evaluated());
    let back = round_trip(&req);
    assert_eq!(back, req);
    assert!(!back.is_evaluated());
    match &back.origin {
        RequestOrigin::StaticAdmitted { unsupported, .. } => assert!(!unsupported.is_empty()),
        RequestOrigin::DynamicEvaluated { .. } => panic!("origin kind changed"),
    }
}

/// Every request field appears by name in the wire object (no silent slot).
#[test]
fn wire_object_names_every_field() {
    let req = static_request("name: \"x\"\nrender: \"vue\"\n");
    let wire = request_to_wire(&req).expect("encode");
    let root = wire.as_object().unwrap();
    assert_eq!(root["wire"], 1);
    let request = root["request"].as_object().unwrap();
    let expected = [
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
    for field in expected {
        assert!(
            request.contains_key(field),
            "wire request missing '{field}'"
        );
    }
}

/// Negative controls: unrepresentable values fail loudly both directions.
#[test]
fn unrepresentable_values_are_rejected() {
    let mut req = GenerationRequest::dynamic(SourceInfo::inline());
    req.package_assets.npm_deps = Some(Value::Lambda("x".into()));
    let err = request_to_wire(&req).unwrap_err();
    assert!(
        matches!(err, WireError::UnsupportedValue { variant: "Lambda" }),
        "{err}"
    );

    let mut req2 = GenerationRequest::dynamic(SourceInfo::inline());
    req2.package_assets.styles = Some(Value::Float(f64::NAN));
    assert!(matches!(
        request_to_wire(&req2).unwrap_err(),
        WireError::NonFiniteFloat
    ));
}

/// Decode rejects: wrong wire version, unknown request keys, missing keys,
/// unknown tags, wrong-typed optional strings.
#[test]
fn decode_rejects_drift() {
    let req = static_request("name: \"x\"\n");
    let wire = request_to_wire(&req).expect("encode");

    // wrong version
    let mut v = wire.clone();
    v["wire"] = serde_json::json!(99);
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::Shape { field, .. } if field == "wire"
    ));

    // unknown extra request key
    let mut v = wire.clone();
    v["request"]["surprise"] = serde_json::json!(1);
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::Shape { field, .. } if field == "request"
    ));

    // missing request key
    let mut v = wire.clone();
    v["request"].as_object_mut().unwrap().remove("toggles");
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::Shape { field, .. } if field == "request"
    ));

    // wrong type inside an optional string slot
    let mut v = wire.clone();
    v["request"]["identity"]["name"] = serde_json::json!(17);
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::Shape { .. }
    ));

    // unknown scene tag
    let mut v = wire.clone();
    v["request"]["scene"] = serde_json::json!("martian");
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::UnknownTag { tag } if tag == "martian"
    ));

    // unknown value tag inside package assets
    let mut v = wire;
    v["request"]["package_assets"]["npm_deps"] = serde_json::json!({"t": "widget", "v": 1});
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::UnknownTag { tag } if tag == "widget"
    ));
}

// ---------------------------------------------------------------------------
// Review R1 counterexamples as permanent regressions (F-01/F-02/F-03/F-05)
// ---------------------------------------------------------------------------

/// F-01: `TypeMismatch` keeps its diagnostic payload across the wire.
#[test]
fn r1_type_mismatch_payload_survives_round_trip() {
    let req = static_request("members: 7\nname: \"x\"\n");
    match &req.origin {
        RequestOrigin::StaticAdmitted { unsupported, .. } => assert!(unsupported
            .iter()
            .any(|k| matches!(k, UnsupportedKind::TypeMismatch { field: "members" }))),
        other => panic!("expected static origin, got {other:?}"),
    }
    let back = round_trip(&req);
    assert_eq!(back, req, "field payload must not degrade to '?'");
    // invented payloads are rejected loudly (closed scanner set)
    let mut wire = request_to_wire(&req).expect("encode");
    wire["request"]["origin"]["unsupported"][0] = serde_json::json!("type-mismatch:invented-field");
    assert!(matches!(
        request_from_wire(&wire).unwrap_err(),
        WireError::Shape { field, .. } if field == "origin.unsupported"
    ));
}

/// F-01: a non-Unicode path is rejected, never silently rewritten (the
/// lossy round-trip used to hand back a different path).
#[cfg(windows)]
#[test]
fn r1_non_unicode_path_is_rejected_not_rewritten() {
    use std::os::windows::ffi::OsStringExt;
    let raw = std::ffi::OsString::from_wide(&[0xD800]); // unpaired surrogate
    let mut req = GenerationRequest::dynamic(SourceInfo::inline());
    req.project_root = Some(std::path::PathBuf::from(raw));
    let err = request_to_wire(&req).unwrap_err();
    assert!(
        matches!(
            err,
            WireError::UnsupportedPath {
                role: "project_root"
            }
        ),
        "{err}"
    );
}

/// F-02: nested unknown keys and missing nested slots are rejected at every
/// level — no silent defaulting.
#[test]
fn r2_nested_schema_drift_is_rejected() {
    let req = static_request("name: \"x\"\nrender: \"vue\"\n");
    let wire = request_to_wire(&req).expect("encode");

    // unknown key inside a nested object
    let mut v = wire.clone();
    v["request"]["identity"]["future_field"] = serde_json::json!(1);
    assert!(
        request_from_wire(&v).is_err(),
        "nested unknown key accepted"
    );

    // missing slot inside a nested object (null must be explicit)
    let mut v = wire.clone();
    v["request"]["display"]
        .as_object_mut()
        .unwrap()
        .remove("title_zh");
    assert!(
        request_from_wire(&v).is_err(),
        "missing nested slot accepted"
    );

    // variant drift: extra key on a render variant
    let mut v = wire.clone();
    v["request"]["render"]["render"]["extra"] = serde_json::json!(true);
    assert!(
        request_from_wire(&v).is_err(),
        "render variant drift accepted"
    );

    // variant drift: missing payload on a toggle variant
    let mut v = wire;
    v["request"]["toggles"]["shadcn"] = serde_json::json!({"kind": "str"});
    assert!(
        request_from_wire(&v).is_err(),
        "toggle variant drift accepted"
    );
}

/// F-03: every narrowing conversion is checked; window components are
/// finite, in Pac bounds, and exactly f32-representable on both sides.
#[test]
fn r3_numeric_conversions_are_checked() {
    let mut req = GenerationRequest::dynamic(SourceInfo::inline());
    let mut npm = auto_val::Obj::new();
    npm.set("byte", Value::Byte(255));
    req.package_assets.npm_deps = Some(Value::obj(npm));
    let wire = request_to_wire(&req).expect("encode");
    let mut v = wire.clone();
    v["request"]["package_assets"]["npm_deps"]["e"][0][1] =
        serde_json::json!({"t": "byte", "v": 256});
    assert!(
        request_from_wire(&v).is_err(),
        "byte=256 must be rejected (was Byte(0))"
    );

    // window overflow to inf on decode
    let mut v = wire.clone();
    v["request"]["runtime"]["window"] = serde_json::json!({"kind": "size", "w": 1e300, "h": 300.0});
    assert!(
        request_from_wire(&v).is_err(),
        "w=1e300 must be rejected (was Size(inf, 300))"
    );

    // NaN window on encode (json! would silently make it null)
    let mut req2 = GenerationRequest::dynamic(SourceInfo::inline());
    req2.runtime.window = Some(crate::descriptor::WindowSpec::Size(f32::NAN, 300.0));
    assert!(
        request_to_wire(&req2).is_err(),
        "NaN window must fail encode, not emit null"
    );

    // sub-f32 precision is distortion — rejected
    let mut req3 = GenerationRequest::dynamic(SourceInfo::inline());
    req3.runtime.window = Some(crate::descriptor::WindowSpec::Size(1280.0, 300.0));
    let mut wire3 = request_to_wire(&req3).expect("representable encode");
    wire3["request"]["runtime"]["window"] =
        serde_json::json!({"kind": "size", "w": 1280.123456789012, "h": 300.0});
    assert!(
        request_from_wire(&wire3).is_err(),
        "non-f32-exact component must be rejected"
    );
}

/// F-05: the transmittable nesting envelope is enforced symmetrically —
/// anything the codec accepts is frame-readable; deeper trees fail loudly
/// at the codec, not as an unexplained "bad JSON" on the reader side.
#[test]
fn r5_nesting_envelope_is_symmetric() {
    fn nested(depth: usize) -> Value {
        let mut value = Value::Int(1);
        for _ in 0..depth {
            value = Value::Array(auto_val::Array {
                values: vec![value],
            });
        }
        value
    }
    // within the envelope: encode + decode + framing all succeed
    let mut req = GenerationRequest::dynamic(SourceInfo::inline());
    req.package_assets.npm_deps = Some(nested(50));
    let wire = request_to_wire(&req).expect("depth 50 within envelope");
    assert_eq!(request_from_wire(&wire).expect("decode symmetric"), req);

    // beyond the envelope: rejected at the codec with a named error
    let mut deep = GenerationRequest::dynamic(SourceInfo::inline());
    deep.package_assets.npm_deps = Some(nested(70));
    assert!(matches!(
        request_to_wire(&deep).unwrap_err(),
        WireError::DepthExceeded {
            limit: crate::wire::MAX_WIRE_JSON_DEPTH
        }
    ));
    // decode direction shares the envelope
    let deep_json = deeply_nested_json(200);
    assert!(matches!(
        request_from_wire(&deep_json).unwrap_err(),
        WireError::DepthExceeded { .. }
    ));
}

fn deeply_nested_json(depth: usize) -> serde_json::Value {
    let mut value = serde_json::json!(1);
    for _ in 0..depth {
        value = serde_json::json!([value]);
    }
    value
}

// ---------------------------------------------------------------------------
// Review R2 counterexamples as permanent regressions
// (F-815-R1-02 / F-815-R1-05 / F-815-R2-01)
// ---------------------------------------------------------------------------

/// R1-02: a tag object without 't' is a named Shape error — never a Map
/// indexing panic that kills the tool (exit 101).
#[test]
fn r2_value_object_without_tag_is_a_named_error() {
    for bad in [
        serde_json::json!({"v": 1, "extra": 2}),
        serde_json::json!({"v": 1}),
        serde_json::json!({"t": 7, "v": 1}),
        serde_json::json!({"t": "int", "v": 1, "extra": 3}),
        serde_json::json!({"t": "obj", "v": []}),
        serde_json::json!({"t": "pair", "v": 1}),
    ] {
        let err = request_from_wire(&serde_json::json!({
            "wire": 1,
            "request": minimal_request_with_npm(&bad),
        }))
        .unwrap_err();
        assert!(matches!(err, WireError::Shape { .. }), "{bad} -> {err}");
    }
    // the same payload placed in raw props / raw node props stays controlled
    let err = request_from_wire(&serde_json::json!({
        "wire": 1,
        "request": minimal_request_with_raw_value(&serde_json::json!({"v": 1, "extra": 2})),
    }))
    .unwrap_err();
    assert!(matches!(err, WireError::Shape { .. }), "{err:?}");
}

fn base_request_json() -> serde_json::Value {
    let req = static_request("name: \"x\"\n");
    request_to_wire(&req).expect("encode")
}

fn minimal_request_with_npm(value: &serde_json::Value) -> serde_json::Value {
    let mut wire = base_request_json();
    wire["request"]["package_assets"]["npm_deps"] = value.clone();
    wire
}

fn minimal_request_with_raw_value(value: &serde_json::Value) -> serde_json::Value {
    let mut wire = base_request_json();
    wire["request"]["raw"] = serde_json::json!([
        {"key": "k", "value": value, "line": 1}
    ]);
    wire
}

/// R2-01: the diagnostic tag prefix is consumed exactly once; encode and
/// decode share the closed field-set admission.
#[test]
fn r2_diagnostic_tag_admission_is_exact() {
    // positives: real scanner fields round-trip
    for text in ["members: 7\n", "defines: 7\n"] {
        let req = static_request(text);
        assert_eq!(round_trip(&req), req);
    }
    // repeated prefix: the prefix is consumed once, so the payload
    // "type-mismatch:members" fails the closed-set check — rejected as a
    // named Shape error, never normalized into members
    let wire = base_request_json();
    let err = unsupported_decode(&wire, "type-mismatch:type-mismatch:members");
    assert!(
        matches!(&err, WireError::Shape { field, .. } if *field == "origin.unsupported"
            && err.to_string().contains("type-mismatch:members")),
        "{err}"
    );
    // empty payload and unknown fields are rejected
    assert!(matches!(
        unsupported_decode(&wire, "type-mismatch:"),
        WireError::Shape { .. }
    ));
    assert!(matches!(
        unsupported_decode(&wire, "type-mismatch:future"),
        WireError::Shape { .. }
    ));
    // encode side: out-of-set field must not produce unreadable wire
    let mut req = static_request("name: \"x\"\n");
    if let RequestOrigin::StaticAdmitted { unsupported, .. } = &mut req.origin {
        unsupported.push(UnsupportedKind::TypeMismatch { field: "future" });
    }
    assert!(matches!(
        request_to_wire(&req).unwrap_err(),
        WireError::Shape { field, .. } if field == "origin.unsupported"
    ));
}

fn unsupported_decode(wire: &serde_json::Value, tag: &str) -> WireError {
    let mut wire = wire.clone();
    wire["request"]["origin"]["unsupported"] = serde_json::json!([tag]);
    request_from_wire(&wire).unwrap_err()
}

/// R1-05 (R2 scope): the wire-document depth envelope is the codec's own
/// boundary — 57 array layers fit, 58 do not (measured on the real wire
/// tree, not a hand-waved count).
#[test]
fn r2_wire_depth_boundary_on_the_real_tree() {
    fn nested(depth: usize) -> Value {
        let mut value = Value::Int(1);
        for _ in 0..depth {
            value = Value::Array(auto_val::Array {
                values: vec![value],
            });
        }
        value
    }
    let mut req = GenerationRequest::dynamic(SourceInfo::inline());
    req.package_assets.npm_deps = Some(nested(57));
    let wire = request_to_wire(&req).expect("57 layers fit the wire envelope");
    assert_eq!(request_from_wire(&wire).expect("decode symmetric"), req);
    let mut deep = GenerationRequest::dynamic(SourceInfo::inline());
    deep.package_assets.npm_deps = Some(nested(58));
    assert!(matches!(
        request_to_wire(&deep).unwrap_err(),
        WireError::DepthExceeded {
            limit: crate::wire::MAX_WIRE_JSON_DEPTH
        }
    ));
}
