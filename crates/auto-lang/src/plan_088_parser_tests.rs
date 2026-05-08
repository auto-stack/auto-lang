// Plan 088 Phase 3: Parser parameter mode parsing tests
// Plan 122: Updated for Trinity of Resources (view, mut, move)
//
// Tests for parsing parameter modes (view, mut, move)
// Deprecated: copy (removed), take (use move instead)

use crate::ast::{Param, ParamMode};
use crate::Parser;

#[cfg(test)]
mod plan_088_parser_tests {
    use super::*;

    /// Helper function to parse function parameters from source code
    /// Note: fn_params() expects the source to end with ')' or other terminator
    fn parse_params(source: &str) -> Vec<Param> {
        use std::rc::Rc;
        use std::cell::RefCell;
        // Plan 091: Universe removed

        // Plan 091: Universe removed
    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
        let full_source = format!("{})", source);  // Add closing parenthesis
        let mut parser = Parser::from(&full_source);
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
    fn test_explicit_move_mode() {
        // Plan 122: Explicit move mode (replaces copy/take)
        let params = parse_params("move a int, move b int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::Move);
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::Move);
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
        // Plan 122: 'take' is deprecated, maps to Move
        let params = parse_params("take s str");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.as_str(), "s");
        assert_eq!(params[0].mode, ParamMode::Move); // take now maps to move
    }

    #[test]
    fn test_mixed_param_modes() {
        // Plan 122: Mix of different modes (view, mut, move)
        let params = parse_params("move a int, view b int, mut c int, move d int");
        assert_eq!(params.len(), 4);
        assert_eq!(params[0].mode, ParamMode::Move);
        assert_eq!(params[1].mode, ParamMode::View);
        assert_eq!(params[2].mode, ParamMode::Mut);
        assert_eq!(params[3].mode, ParamMode::Move);
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
        // Plan 122: Parameter mode with type annotation (using move, not copy)
        let params = parse_params("move a: int, mut b: int");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::Move);
        assert_eq!(params[1].name.as_str(), "b");
        assert_eq!(params[1].mode, ParamMode::Mut);
    }

    #[test]
    fn test_param_mode_with_default_value() {
        // Plan 122: Parameter mode with default value (using move, not copy)
        let params = parse_params("move a int = 5, view b int = 10");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name.as_str(), "a");
        assert_eq!(params[0].mode, ParamMode::Move);
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
        // Plan 122: Complex function signature with mixed modes
        let params = parse_params(
            "mut self Point, move x int, view y float, move s str, flag bool"
        );
        assert_eq!(params.len(), 5);
        assert_eq!(params[0].name.as_str(), "self");
        assert_eq!(params[0].mode, ParamMode::Mut);
        assert_eq!(params[1].name.as_str(), "x");
        assert_eq!(params[1].mode, ParamMode::Move);
        assert_eq!(params[2].name.as_str(), "y");
        assert_eq!(params[2].mode, ParamMode::View);
        assert_eq!(params[3].name.as_str(), "s");
        assert_eq!(params[3].mode, ParamMode::Move);
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
        // Plan 122: Single parameter with different modes
        let params = parse_params("move x int");
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name.as_str(), "x");
        assert_eq!(params[0].mode, ParamMode::Move);

        let params2 = parse_params("mut x int");
        assert_eq!(params2.len(), 1);
        assert_eq!(params2[0].name.as_str(), "x");
        assert_eq!(params2[0].mode, ParamMode::Mut);
    }
}
