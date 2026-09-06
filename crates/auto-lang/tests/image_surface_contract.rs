//! Cross-generator contract coverage for Plan 547 ImageSurface.
//!
//! The fixture is parsed once through the regular file driver, then the same
//! extracted AuraWidget is consumed by the VM builder and Rust generator while
//! the file driver also supplies the Vue output. This keeps all three checks
//! tied to one source contract rather than three hand-built test trees.

#![cfg(feature = "ui-iced")]

use auto_lang::ui::aura_view_builder::AuraViewBuilder;
use auto_lang::ui::view::View;
use auto_lang::ui::vm_bridge::VmBridge;
use auto_lang::ui_gen::{generate_component_from_file, BackendGenerator, ComponentGenOptions, RustGenerator};

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("image_surface_contract.at")
}

#[test]
fn image_surface_contract_is_preserved_across_generators() {
    let generated = generate_component_from_file(
        &fixture_path(),
        ComponentGenOptions::default(),
    )
    .expect("fixture must parse and generate");
    let widget = generated
        .widgets
        .iter()
        .find(|widget| widget.name == "ImageViewer")
        .expect("ImageViewer widget");

    // VM builder: scalar props, media URI and all five typed callbacks survive
    // Aura extraction and DynamicMessage conversion.
    let bridge = VmBridge::new(widget).expect("VM bridge");
    let view = AuraViewBuilder::new(&bridge, &widget.name).build(&widget.view_tree);
    let View::ImageSurface {
        src,
        alt,
        width,
        height,
        quality,
        fit,
        zoom,
        offset_x,
        offset_y,
        rotation,
        filter,
        on_error: Some(on_error),
        on_loaded: Some(on_loaded),
        on_wheel: Some(on_wheel),
        on_pan: Some(on_pan),
        on_double_click: Some(on_double_click),
        ..
    } = view
    else {
        panic!("fixture did not build an ImageSurface: {view:?}");
    };
    assert_eq!(src, "/api/__auto/media/demo/7");
    assert_eq!(alt, "hero");
    assert_eq!((width, height, quality), (640, 480, 80));
    assert_eq!(fit, "width");
    assert_eq!((zoom, offset_x, offset_y, rotation), (1.25, 12.5, -4.0, 90));
    assert_eq!(filter, "linear");

    let event_name = |message: auto_lang::ui::interpreter::DynamicMessage| match message {
        auto_lang::ui::interpreter::DynamicMessage::Typed { event_name, .. } => event_name,
        other => panic!("expected typed ImageSurface callback, got {other:?}"),
    };
    assert_eq!(event_name(on_error), "ImageFailed");
    assert_eq!(event_name(on_loaded), "ImageLoaded");
    assert_eq!(event_name(on_wheel), "ZoomAt");
    assert_eq!(event_name(on_pan), "PanBy");
    assert_eq!(event_name(on_double_click), "ToggleFit");

    // Rust generator: the same fixture emits the backend-neutral constructor,
    // live state bindings, transform props and all event hooks.
    let mut rust_generator = RustGenerator::new();
    let rust = rust_generator.generate(widget).expect("Rust generation");
    for marker in [
        "View::image_surface(",
        "self.asset_src.clone()",
        "image_surface_props",
        "image_surface_events(Some(ImageViewerMsg::ImageFailed)",
        "Some(ImageViewerMsg::ImageLoaded)",
        "Some(ImageViewerMsg::ZoomAt)",
        "Some(ImageViewerMsg::PanBy)",
        "Some(ImageViewerMsg::ToggleFit)",
    ] {
        assert!(rust.contains(marker), "Rust contract marker missing: {marker}\n{rust}");
    }

    // Vue generator: the same media URI and interactive props/events are
    // emitted without introducing a browser-side decode/cache implementation.
    let vue = generated
        .all_widget_codes
        .iter()
        .find(|(name, _)| name == "ImageViewer")
        .map(|(_, code)| code)
        .expect("ImageViewer Vue output");
    for marker in [
        "class=\"relative overflow-hidden\"",
        ":src=\"asset_src\"",
        "objectFit: 'width'",
        "translate(",
        "@load=\"ImageLoaded\"",
        "@error=\"ImageFailed\"",
        "@wheel=\"ZoomAt\"",
        "@pan=\"PanBy\"",
        "@dblclick=\"ToggleFit\"",
    ] {
        assert!(vue.contains(marker), "Vue contract marker missing: {marker}\n{vue}");
    }
    for forbidden in ["FileReader", "decodeImage", "imageCache", "cache.put"] {
        assert!(
            !vue.to_ascii_lowercase().contains(&forbidden.to_ascii_lowercase()),
            "browser-side media implementation leaked into Vue: {forbidden}"
        );
    }
}
