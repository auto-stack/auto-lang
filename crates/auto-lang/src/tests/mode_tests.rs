// Plan 081 Phase 2: Test mode parsing from pac.at files
//
// This test verifies that the ExecutionMode enum can be used
// and that pac.at files can specify execution modes.

use auto_lang::config::AutoConfig;
use auto_lang::mode::ExecutionMode;

#[test]
fn test_mode_enum_from_str() {
    // Test parsing mode strings
    assert_eq!(ExecutionMode::from_str("autovm"), Some(ExecutionMode::AutoVM));
    assert_eq!(ExecutionMode::from_str("c"), Some(ExecutionMode::C));
    assert_eq!(ExecutionMode::from_str("rust"), Some(ExecutionMode::Rust));
    assert_eq!(ExecutionMode::from_str("evaluator"), Some(ExecutionMode::Evaluator));

    // Test aliases
    assert_eq!(ExecutionMode::from_str("vm"), Some(ExecutionMode::AutoVM));
    assert_eq!(ExecutionMode::from_str("eval"), Some(ExecutionMode::Evaluator));

    // Test invalid mode
    assert_eq!(ExecutionMode::from_str("invalid"), None);
}

#[test]
fn test_parse_pac_at_with_mode() {
    // Test parsing a pac.at file that includes a mode field

    let pac_at_code = r#"
name: "test_app"
version: "0.1.0"
mode: "c"

app("test_app") {}
"#;

    let config = AutoConfig::new(pac_at_code).unwrap();

    // Extract name
    let name = config.root.get_prop("name");
    assert_eq!(name.to_str(), "test_app");

    // Extract mode
    let mode = config.root.get_prop("mode");
    assert_eq!(mode.to_str(), "c");

    // Parse mode string to ExecutionMode
    let parsed_mode = ExecutionMode::from_str(mode.to_str().as_str());
    assert_eq!(parsed_mode, Some(ExecutionMode::C));
}

#[test]
fn test_default_mode_when_not_specified() {
    // Test that default mode is autovm when not specified

    let pac_at_code = r#"
name: "test_app"
version: "0.1.0"

app("test_app") {}
"#;

    let config = AutoConfig::new(pac_at_code).unwrap();

    // Mode field not present - should default to autovm
    let mode = config.root.get_prop("mode");
    assert!(mode.is_nil());

    // Default mode is AutoVM
    assert_eq!(ExecutionMode::default(), ExecutionMode::AutoVM);
}

#[test]
fn test_mode_characteristics() {
    // Test mode characteristic methods

    assert!(ExecutionMode::AutoVM.requires_compilation());
    assert!(ExecutionMode::AutoVM.is_bytecode());
    assert!(!ExecutionMode::AutoVM.is_transpilation());

    assert!(ExecutionMode::C.requires_compilation());
    assert!(ExecutionMode::C.is_transpilation());
    assert!(!ExecutionMode::C.is_bytecode());

    assert!(ExecutionMode::Rust.requires_compilation());
    assert!(ExecutionMode::Rust.is_transpilation());
    assert!(!ExecutionMode::Rust.is_bytecode());

    assert!(!ExecutionMode::Evaluator.requires_compilation());
    assert!(ExecutionMode::Evaluator.is_interpreter());
    assert!(!ExecutionMode::Evaluator.is_transpilation());
}
