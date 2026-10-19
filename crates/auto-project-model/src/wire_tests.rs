//! Wire codec tests (PLAN-815 T-03): round-trip over the full request face,
//! field preservation, and honest rejection of unrepresentable values.

use crate::descriptor::SourceInfo;
use crate::generation_request::{DepRoot, PackageAssets, RequestMode, ThemeDeclLite};
use crate::wire::{request_from_wire, request_to_wire, WireError};
use crate::{parse_document, GenerationRequest, RequestOrigin};
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
        Value::Array(Array { values: vec![Value::Bool(true), Value::Int(9)] }),
    );
    npm.set(
        "block",
        Value::Block(Array { values: vec![Value::Str("x".into())] }),
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
    req.dep_roots = vec![DepRoot { name: "d".into(), path: Some("../d".into()) }];
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
        "origin", "project_root", "identity", "display", "runtime", "scene", "members",
        "render", "external_backend", "toggles", "theme_decl", "dep_roots", "package_assets",
        "exe_name", "has_rust_sidecar", "front_decl", "front_node_form", "input_sources",
        "output_root", "mode", "raw", "raw_nodes",
    ];
    for field in expected {
        assert!(request.contains_key(field), "wire request missing '{field}'");
    }
}

/// Negative controls: unrepresentable values fail loudly both directions.
#[test]
fn unrepresentable_values_are_rejected() {
    let mut req = GenerationRequest::dynamic(SourceInfo::inline());
    req.package_assets.npm_deps = Some(Value::Lambda("x".into()));
    let err = request_to_wire(&req).unwrap_err();
    assert!(matches!(err, WireError::UnsupportedValue { variant: "Lambda" }), "{err}");

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
    v["request"]["package_assets"]["npm_deps"] =
        serde_json::json!({"t": "widget", "v": 1});
    assert!(matches!(
        request_from_wire(&v).unwrap_err(),
        WireError::UnknownTag { tag } if tag == "widget"
    ));
}
