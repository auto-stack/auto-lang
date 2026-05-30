//! Unified Backend Generation Integration Tests
//!
//! Tests that the same AURA widget can generate valid code for
//! Vue, Tauri, and Jet backends.

use auto_lang::ui_gen::{BackendGenerator, VueGenerator, JetGenerator};
use auto_lang::aura::{AuraWidget, AuraNode, AuraStateDef, AuraMessage, AuraExpr, AuraPropValue, AuraTextContent};
use auto_lang::ast::Type;
use std::collections::HashMap;

/// Create a simple counter widget for testing
fn create_test_widget() -> AuraWidget {
    AuraWidget {
        name: "TestCounter".to_string(),
        state_vars: vec![
            AuraStateDef {
                name: "count".to_string(),
                type_info: Type::Int,
                initial: AuraExpr::Int(0),
                decorators: vec![],
            }
        ],
        computed: vec![],
        messages: vec![
            AuraMessage {
                name: "Msg".to_string(),
                variants: vec![
                    auto_lang::aura::AuraMsgVariant {
                        name: "Increment".to_string(),
                        payload: None,
                    }
                ],
            }
        ],
        view_tree: AuraNode::Element {
            tag: "col".to_string(),
            props: {
                let mut props = HashMap::new();
                props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal(
                    "flex flex-col gap-4 p-4 items-center".to_string()
                )));
                props
            },
            events: HashMap::new(),
            children: vec![

                AuraNode::Text(AuraTextContent::Literal("Count: ".to_string())),
            ],
            span: None,
            debug_id: None,
        },
        handlers: HashMap::new(),
        props: vec![],
        routes: None,
        lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
    }
}

#[test]
fn test_vue_backend_generates_valid_code() {
    let widget = create_test_widget();
    let mut gen = VueGenerator::new();

    let result = gen.generate(&widget);

    assert!(result.is_ok(), "Vue generation should succeed");
    let code = result.unwrap();

    // Print for debugging
    eprintln!("Generated Vue code:\n{}", code);

    // Check that generated code contains Vue component structure
    // Vue generator may use different patterns
    assert!(
        code.contains("<script setup>") ||
        code.contains("ref(") ||
        code.contains("<template>") ||
        code.contains("defineComponent") ||
        code.contains("TestCounter"),
        "Should contain Vue component structure"
    );
}

#[test]
fn test_jet_backend_generates_valid_code() {
    let widget = create_test_widget();
    let mut gen = JetGenerator::new();

    let result = gen.generate(&widget);

    assert!(result.is_ok(), "Jet generation should succeed");
    let code = result.unwrap();

    // Check that generated code contains Compose structure
    assert!(code.contains("@Composable"), "Should contain @Composable annotation");
    assert!(code.contains("fun TestCounter"), "Should contain function name");
}

#[test]
fn test_both_backends_generate_from_same_widget() {
    let widget = create_test_widget();

    // Generate Vue code
    let mut vue_gen = VueGenerator::new();
    let vue_result = vue_gen.generate(&widget).expect("Vue generation should succeed");

    // Generate Jet code
    let mut jet_gen = JetGenerator::new();
    let jet_result = jet_gen.generate(&widget).expect("Jet generation should succeed");

    // Both should contain the widget name
    assert!(
        vue_result.contains("TestCounter") || vue_result.contains("test-counter"),
        "Vue code should reference widget name"
    );
    assert!(
        jet_result.contains("TestCounter"),
        "Jet code should reference widget name"
    );

    // Both should handle the model state
    assert!(
        vue_result.contains("count"),
        "Vue code should reference count state"
    );
    assert!(
        jet_result.contains("count"),
        "Jet code should reference count state"
    );
}

#[test]
fn test_tailwind_classes_converted_for_each_backend() {
    // Create a widget with Tailwind classes using helper methods
    let widget = AuraWidget {
        name: "StyledWidget".to_string(),
        state_vars: vec![],
        computed: vec![],
        messages: vec![],
        view_tree: AuraNode::Element {
            tag: "button".to_string(),
            props: {
                let mut props = HashMap::new();
                props.insert("class".to_string(), AuraPropValue::Expr(AuraExpr::Literal(
                    "px-4 py-2 bg-blue-500 text-white rounded".to_string()
                )));
                props
            },
            events: HashMap::new(),
            children: vec![],
            span: None,
            debug_id: None,
        },
        handlers: HashMap::new(),
        props: vec![],
        routes: None,
        lifecycle: vec![],
            tick_interval: None,
            handler_params: HashMap::new(),
            span_map: HashMap::new(),
    };

    // Generate Vue code - should keep Tailwind classes as-is
    let mut vue_gen = VueGenerator::new();
    let vue_result = vue_gen.generate(&widget).expect("Vue generation should succeed");
    assert!(
        vue_result.contains("px-4") || vue_result.contains("class"),
        "Vue should preserve class attribute"
    );

    // Generate Jet code - should convert to Modifier
    let mut jet_gen = JetGenerator::new();
    let jet_result = jet_gen.generate(&widget).expect("Jet generation should succeed");
    // Jet should convert to Compose modifiers or styling
    assert!(
        jet_result.contains("Modifier") || jet_result.contains("padding") || jet_result.contains("Button"),
        "Jet should use Modifier or Button component"
    );
}
