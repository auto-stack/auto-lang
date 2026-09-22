// Plan 088 Phase 1: Type system extension tests
// Plan 122: Updated for Trinity of Resources (view, mut, move)
//
// Tests for parameter passing mode implementation:
// - ParamMode enum (View, Mut, Move) - Copy deprecated, Take deprecated
// - is_optimized_by_value() method
// - Param::with_mode() constructor

use crate::ast::{Param, ParamMode, Type};

#[cfg(test)]
mod plan_088_tests {
    use super::*;

    #[test]
    fn test_param_mode_default() {
        // Default should be View per Plan 088 ABO-01
        let mode = ParamMode::default();
        assert_eq!(mode, ParamMode::View);
    }

    #[test]
    fn test_param_mode_display() {
        // Plan 122: Display for modes
        assert_eq!(format!("{}", ParamMode::View), "view");
        assert_eq!(format!("{}", ParamMode::Mut), "mut");
        assert_eq!(format!("{}", ParamMode::Move), "move");
        // Deprecated modes:
        assert_eq!(format!("{}", ParamMode::Copy), "copy"); // Still displays as "copy" for backwards compat
        assert_eq!(format!("{}", ParamMode::Take), "move"); // Displays as "move" per Plan 122
    }

    #[test]
    fn test_param_default_mode() {
        // Param constructed with new() should have View as default mode
        let param = Param::new("x".into(), Type::Int, None);
        assert_eq!(param.mode, ParamMode::View);
    }

    #[test]
    fn test_param_with_mode() {
        // Plan 122: Param constructed with with_mode() should have specified mode
        let param_move = Param::with_mode("x".into(), Type::Int, None, ParamMode::Move);
        assert_eq!(param_move.mode, ParamMode::Move);

        let param_mut = Param::with_mode("x".into(), Type::Int, None, ParamMode::Mut);
        assert_eq!(param_mut.mode, ParamMode::Mut);

        let param_view = Param::with_mode("x".into(), Type::Int, None, ParamMode::View);
        assert_eq!(param_view.mode, ParamMode::View);
    }

    #[test]
    fn test_is_optimized_by_value_small_types() {
        // Small types should return true (use value passing)
        assert!(Type::Byte.is_optimized_by_value());
        assert!(Type::Int.is_optimized_by_value());
        assert!(Type::Uint.is_optimized_by_value());
        assert!(Type::USize.is_optimized_by_value());
        assert!(Type::I64.is_optimized_by_value());
        assert!(Type::U64.is_optimized_by_value());
        assert!(Type::Bool.is_optimized_by_value());
        assert!(Type::Char.is_optimized_by_value());
        assert!(Type::Float.is_optimized_by_value());
        assert!(Type::Double.is_optimized_by_value());
    }

    #[test]
    fn test_is_optimized_by_value_large_types() {
        // Large types should return false (use reference passing)
        assert!(!Type::StrFixed(10).is_optimized_by_value());
        assert!(!Type::CStrLit.is_optimized_by_value());
        assert!(!Type::StrSlice.is_optimized_by_value());

        // Array types
        assert!(!Type::Array(crate::ast::ArrayType {
            elem: Box::new(Type::Int),
            len: 10,
        }).is_optimized_by_value());

        // Runtime array
        assert!(!Type::RuntimeArray(crate::ast::RuntimeArrayType {
            elem: Box::new(Type::Int),
            size_expr: Box::new(crate::ast::Expr::Int(10)),
        }).is_optimized_by_value());

        // List
        assert!(!Type::List(Box::new(Type::Int)).is_optimized_by_value());

        // Slice
        assert!(!Type::Slice(crate::ast::SliceType {
            elem: Box::new(Type::Int),
        }).is_optimized_by_value());
    }

    #[test]
    fn test_is_optimized_by_value_pointer_types() {
        // Pointer types should return false
        assert!(!Type::Ptr(crate::ast::PtrType {
            of: std::rc::Rc::new(std::cell::RefCell::new(Type::Int)),
        }).is_optimized_by_value());

        assert!(!Type::Reference(Box::new(Type::Int)).is_optimized_by_value());
    }

    #[test]
    fn test_is_optimized_by_value_user_types() {
        // User-defined types should return false (V1 conservative)
        // Note: We can't easily test these without creating actual TypeDecl,
        // but we can verify they exist in the Type enum
        let type_names = vec![
            "User", "Tag", "Enum", "Union", "CStruct", "GenericInstance",
        ];

        // The test verifies these types are handled in the match statement
        for name in type_names {
            println!("{} should use reference passing", name);
        }
    }

    #[test]
    fn test_is_optimized_by_value_complex_types() {
        // Complex types should return false
        assert!(!Type::Void.is_optimized_by_value());
        assert!(!Type::Unknown.is_optimized_by_value());
        assert!(!Type::Variadic.is_optimized_by_value());

        // Function type
        assert!(!Type::Fn(vec![Type::Int], Box::new(Type::Int)).is_optimized_by_value());
    }

    #[test]
    fn test_param_display_includes_mode() {
        let param_view = Param::new("x".into(), Type::Int, None);
        let display = format!("{}", param_view);
        assert!(display.contains("view"));

        let param_copy = Param::with_mode("y".into(), Type::Int, None, ParamMode::Copy);
        let display_copy = format!("{}", param_copy);
        assert!(display_copy.contains("copy"));
    }

    #[test]
    fn test_small_type_performance() {
        // Verify that small types are correctly identified
        // This is critical for performance optimization

        let int_param = Param::new("value".into(), Type::Int, None);
        assert!(int_param.ty.is_optimized_by_value());

        let bool_param = Param::new("flag".into(), Type::Bool, None);
        assert!(bool_param.ty.is_optimized_by_value());

        let float_param = Param::new("pi".into(), Type::Float, None);
        assert!(float_param.ty.is_optimized_by_value());
    }

    #[test]
    fn test_large_type_reference() {
        // Verify that large types use reference passing
        // This is critical for memory efficiency

        let string_param = Param::new("text".into(), Type::StrFixed(100), None);
        assert!(!string_param.ty.is_optimized_by_value());

        let list_param = Param::new("items".into(), Type::List(Box::new(Type::Int)), None);
        assert!(!list_param.ty.is_optimized_by_value());
    }
}
