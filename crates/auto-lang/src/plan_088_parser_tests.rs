// Plan 088 Phase 3: Parser parameter mode parsing tests
//
// Tests for parsing parameter modes (copy, view, mut, take)

use crate::ast::{Param, ParamMode, Type};
use crate::Parser;

#[cfg(test)]
mod plan_088_parser_tests {
    use super::*;

    /// Helper function to parse function parameters from source code
    /// Note: fn_params() expects the source to end with ')' or other terminator
    fn parse_params(source: &str) -> Vec<Param> {
        use std::rc::Rc;
        use std::cell::RefCell;
        use crate::universe::Universe;

        let scope = Rc::new(RefCell::new(Universe::new()));
        let full_source = format!("{})", source);  // Add closing parenthesis
        let mut parser = Parser::new(&full_source, scope);
        parser.fn_params().unwrap()
    }

    #[test]
    fn test_default_param_mode() {
        // Default mode should be View
        let params = parse_params("a int, b int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::View);
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::View);
    }

    #[test]
    fn test_explicit_copy_mode() {
        // Explicit copy mode
        let params = parse_params("copy a int, copy b int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::Copy);
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::Copy);
    }

    #[test]
    fn test_explicit_view_mode() {
        // Explicit view mode
        let params = parse_params("view x int, view y int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "x");
        assert_eq!(params[0].mode, ParamMode::View);
        assert_eq!(params[1].name.as_str(), "y");
        assert_eq!(params[1].mode, ParamMode::View);
    }

    #[test]
    fn test_explicit_mut_mode() {
        // Explicit mut mode
        let params = parse_params("mut self Point, new_x int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "self");
        assert_eq!(params[0].mode, ParamMode::Mut);
        assert_eq!(params[1].name.as_str(), "new_x");
        assert_eq!(params[1].mode, ParamMode::View); // Default
    }

    #[test]
    fn test_explicit_take_mode() {
        // Explicit take mode
        let params = parse_params("take s str");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.as_str(), "s");
        assert_eq!(params[0].mode, ParamMode::Take);
    }

    #[test]
    fn test_mixed_param_modes() {
        // Mix of different modes
        let params = parse_params("copy a int, view b int, mut c int, take d int");
        assert_eq!(params.len(), 4);
        assert_eq!(params[0].mode, ParamMode::Copy);
        assert_eq!(params[1].mode, ParamMode::View);
        assert_eq!(params[2].mode, ParamMode::Mut);
        assert_eq!(params[3].mode, ParamMode::Take);
    }

    #[test]
    fn test_param_with_type_annotation() {
        // Parameters with type annotations (using ':')
        let params = parse_params("a: int, b: int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::View);
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::View);
    }

    #[test]
    fn test_param_with_default_value() {
        // Parameters with default values
        let params = parse_params("a int = 10, b int = 20");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::View);
        assert!(params[0].default.is_some());
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::View);
        assert!(params[1].default.is_some());
    }

    #[test]
    fn test_param_mode_with_type_annotation() {
        // Parameter mode with type annotation
        let params = parse_params("copy a: int, mut b: int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::Copy);
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::Mut);
    }

    #[test]
    fn test_param_mode_with_default_value() {
        // Parameter mode with default value
        let params = parse_params("copy a int = 5, view b int = 10");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::Copy);
        assert!(params[0].default.is_some());
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::View);
        assert!(params[1].default.is_some());
    }

    #[test]
    fn test_newline_separator() {
        // Parameters separated by newlines
        let params = parse_params("a int\nb int\nc int");
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[2].name.as_str(), "c");
    }

    #[test]
    fn test_comma_separator() {
        // Parameters separated by commas
        let params = parse_params("a int, b int, c int");
        assert_eq!(params.len(), 3);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[2].name.as_str(), "c");
    }

    #[test]
    fn test_complex_function_signature() {
        // Complex function signature with mixed modes
        let params = parse_params(
            "mut self Point, copy x int, view y float, take s str, flag bool"
        );
        assert_eq!(params.len(), 5);
        assert_eq!(params[0].name.as_str(), "self");
        assert_eq!(params[0].mode, ParamMode::Mut);
        assert_eq!(params[1].name.as_str(), "x");
        assert_eq!(params[1].mode, ParamMode::Copy);
        assert_eq!(params[2].name.as_str(), "y");
        assert_eq!(params[2].mode, ParamMode::View);
        assert_eq!(params[3].name.as_str(), "s");
        assert_eq!(params[3].mode, ParamMode::Take);
        assert_eq!(params[4].name.as_str(), "flag");
        assert_eq!(params[4].mode, ParamMode::View); // Default
    }

    #[test]
    fn test_empty_params() {
        // Empty parameter list
        let params = parse_params("");
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_single_param() {
        // Single parameter with different modes
        let params = parse_params("copy x int");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.as_str(), "x");
        assert_eq!(params[0].mode, ParamMode::Copy);

        let params2 = parse_params("mut x int");
        assert_eq!(params2.len(), 1);
        assert_eq!(params2[0].name.as_str(), "x");
        assert_eq!(params2[0].mode, ParamMode::Mut);
    }
}
