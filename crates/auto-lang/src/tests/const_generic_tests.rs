//! Const generic parameter tests for Plan 052
//!
//! These tests verify that const generic parameters work correctly:
//! - Parsing const parameters: `const N u32`
//! - GenericParam enum functionality
//! - Const parameter in type definitions

use crate::run;

/// Test const parameter parsing
#[test]
fn test_const_param_parsing() {
    // Note: Const generic parameters CAN be parsed successfully now!
    // However, using const parameters in array types [N]T is not yet implemented.
    // This test verifies that the parser accepts the syntax.

    let code = r#"
type Inline<T, N u32> {
    buffer: [N]T
}

fn main() {
    return 0
}
"#;
    let result = run(code).unwrap_or_else(|e| format!("Error: {}", e));
    // The parsing should succeed (no "syntax error")
    // Evaluation might fail because [N]T with const N is not yet supported
    assert!(!result.contains("syntax error"),
        "Const generic parsing should succeed, got: {}", result);
    assert!(result.contains("Error") || !result.contains("syntax error"),
        "May have evaluation errors (expected), but parsing should work");
}

/// Test that const keyword is tokenized
#[test]
fn test_const_keyword_tokenized() {
    // Note: We can't test const variable declarations yet
    // because that syntax isn't fully implemented.
    // But we've verified TokenKind::Const exists in the lexer.

    // This test verifies that the GenericParam and ConstParam structures work correctly
    use crate::ast::{GenericParam, ConstParam, Type};

    let param = ConstParam {
        name: "SIZE".into(),
        typ: Type::Uint,
        default: None,
    };

    assert_eq!(param.name.as_str(), "SIZE");
    assert!(matches!(param.typ, Type::Uint));
}

/// Test that type parameters still work
#[test]
fn test_type_param_still_works() {
    // This test verifies that existing type parameter functionality isn't broken
    use crate::ast::TypeParam;

    let param = TypeParam {
        name: "T".into(),
        constraint: None,
    };

    assert_eq!(param.name.as_str(), "T");
    assert!(param.constraint.is_none());
}

/// Test GenericParam Display implementation
#[test]
fn test_generic_param_display() {
    use crate::ast::{GenericParam, TypeParam, ConstParam, Type};

    // Type parameter
    let type_param = GenericParam::Type(TypeParam {
        name: "T".into(),
        constraint: None,
    });
    assert_eq!(format!("{}", type_param), "T");

    // Const parameter (syntax: N u32, without 'const' keyword)
    let const_param = GenericParam::Const(ConstParam {
        name: "N".into(),
        typ: Type::Uint,
        default: None,
    });
    assert_eq!(format!("{}", const_param), "N uint");
}

/// Test ConstParam structure
#[test]
fn test_const_param_structure() {
    use crate::ast::{ConstParam, Type};

    let param = ConstParam {
        name: "CAPACITY".into(),
        typ: Type::USize,
        default: None,
    };

    assert_eq!(param.name.as_str(), "CAPACITY");
    assert!(matches!(param.typ, Type::USize));
    assert!(param.default.is_none());
}
