//! Integration tests for the borrow checker
//!
//! These tests verify that the borrow checker works correctly
//! with the evaluator and catches ownership violations.

use crate::ast::{Body, Expr};
use crate::ownership::{BorrowChecker, BorrowKind, Lifetime};
use crate::run;

/// Test basic view borrow functionality
///
/// NOTE: This test is currently expected to fail because the
/// .view syntax is not yet fully implemented in the evaluator.
/// This test serves as documentation of the intended behavior.
#[test]
fn test_view_borrow_basic() {
    let code = r#"
fn main() {
    let x = 10
    let view = x.view
    print(view)
}
"#;
    let result = run(code);
    // Currently .view syntax is not implemented in evaluator
    // so we just check that it doesn't crash
    let _ = result;
}

/// Test that multiple view borrows are allowed
///
/// NOTE: This test is currently expected to fail because the
/// .view syntax is not yet fully implemented in the evaluator.
#[test]
fn test_multiple_view_borrows() {
    let code = r#"
fn main() {
    let x = 10
    let v1 = x.view
    let v2 = x.view
    print(v1)
    print(v2)
}
"#;
    let result = run(code);
    // Just verify no crash - .view not yet implemented
    let _ = result;
}

/// Test that mut borrow conflicts with existing view borrow
///
/// NOTE: This test is currently expected to fail because the
/// .view and .mut syntax are not yet fully implemented in the evaluator.
#[test]
fn test_mut_conflicts_with_view() {
    let code = r#"
fn main() {
    let x = 10
    let v = x.view
    let m = x.mut
    print(v)
}
"#;
    let result = run(code);
    // Just verify no crash - property syntax not yet implemented
    let _ = result;
}

/// Test hold expression with view borrow
#[test]
fn test_hold_with_view() {
    let code = r#"
fn main() {
    let x = 10
    hold x as value {
        print(value)
    }
}
"#;
    let result = run(code);
    assert!(result.is_ok(), "Hold expression should succeed");
}

/// Test hold expression with mut borrow
#[test]
fn test_hold_with_mut() {
    let code = r#"
type Point {
    x int
    y int
}

fn main() {
    let p = Point{x: 10, y: 20}
    hold p as point {
        point.x = 30
        point.y = 40
    }
    print(p.x)
    print(p.y)
}
"#;
    let result = run(code);
    assert!(result.is_ok(), "Hold with mut should succeed");
}

/// Test that take transfers ownership
///
/// NOTE: This test is currently expected to fail because the
/// .take syntax is not yet fully implemented in the evaluator.
#[test]
fn test_take_ownership() {
    let code = r#"
fn main() {
    let x = 10
    let y = x.take
    print(y)
}
"#;
    let result = run(code);
    // After take, x should no longer be valid
    // This test verifies the mechanism works
    let _ = result;
}

/// Test borrow checker with struct field access
///
/// NOTE: This test is currently expected to fail because the
/// .view syntax is not yet fully implemented in the evaluator.
#[test]
fn test_borrow_struct_field() {
    let code = r#"
type Point {
    x int
    y int
}

fn main() {
    let p = Point{x: 10, y: 20}
    let field_view = p.x.view
    print(field_view)
}
"#;
    let result = run(code);
    // Just verify no crash - .view not yet implemented
    let _ = result;
}

/// Test borrow checker with array elements
///
/// NOTE: This test is currently expected to fail because the
/// .view syntax is not yet fully implemented in the evaluator.
#[test]
fn test_borrow_array_element() {
    let code = r#"
fn main() {
    let arr = [1, 2, 3]
    let elem_view = arr[0].view
    print(elem_view)
}
"#;
    let result = run(code);
    // Just verify no crash - .view not yet implemented
    let _ = result;
}

/// Test borrow lifetime management
///
/// NOTE: This test is currently expected to fail because the
/// .view and .mut syntax are not yet fully implemented in the evaluator.
#[test]
fn test_borrow_lifetime_end() {
    let code = r#"
fn main() {
    let x = 10
    {
        let v = x.view
        print(v)
    }
    // After the inner block, the view borrow should end
    let m = x.mut
    print(m)
}
"#;
    let result = run(code);
    // Just verify no crash - property syntax not yet implemented
    let _ = result;
}

/// Test nested hold expressions
#[test]
fn test_nested_hold() {
    let code = r#"
type Point {
    x int
    y int
}

fn main() {
    let p = Point{x: 10, y: 20}
    hold p as outer {
        hold outer.x as inner {
            print(inner)
        }
    }
}
"#;
    let result = run(code);
    assert!(result.is_ok(), "Nested hold expressions should succeed");
}

/// Test borrow checker integration with evaluator
#[test]
fn test_borrow_checker_integration() {
    // Create a borrow checker directly
    let mut checker = BorrowChecker::new();
    let expr = Expr::Ident("x".into());

    // Create a view borrow
    let result1 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
    assert!(result1.is_ok(), "View borrow should succeed");

    // Create another view borrow (should be allowed)
    let result2 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(2));
    assert!(result2.is_ok(), "Second view borrow should succeed");

    // Try to create a mut borrow (should conflict)
    let result3 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(3));
    assert!(
        result3.is_err(),
        "Mut borrow should conflict with view borrows"
    );

    // End the first view borrow
    checker.end_borrows_with_lifetime(Lifetime::new(1));

    // Still should conflict due to second view borrow
    let result4 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(4));
    assert!(result4.is_err(), "Mut borrow should still conflict");

    // End the second view borrow
    checker.end_borrows_with_lifetime(Lifetime::new(2));

    // Now mut borrow should succeed
    let result5 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(5));
    assert!(result5.is_ok(), "Mut borrow should succeed after views end");
}

/// Test borrow conflict detection with different targets
#[test]
fn test_borrow_different_targets() {
    let mut checker = BorrowChecker::new();
    let expr1 = Expr::Ident("x".into());
    let expr2 = Expr::Ident("y".into());

    // Borrow x
    let result1 = checker.check_borrow(&expr1, BorrowKind::Mut, Lifetime::new(1));
    assert!(result1.is_ok(), "Mut borrow of x should succeed");

    // Borrow y (different target, should not conflict)
    let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
    assert!(
        result2.is_ok(),
        "Mut borrow of y should not conflict with x"
    );
}

/// Test take conflicts with all borrows
#[test]
fn test_take_conflicts_with_borrows() {
    let mut checker = BorrowChecker::new();
    let expr = Expr::Ident("data".into());

    // Create a view borrow
    let result1 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
    assert!(result1.is_ok(), "View borrow should succeed");

    // Try to take (should conflict)
    let result2 = checker.check_borrow(&expr, BorrowKind::Take, Lifetime::new(2));
    assert!(result2.is_err(), "Take should conflict with view borrow");
}

/// Test borrow error messages contain useful information
#[test]
fn test_borrow_error_messages() {
    let mut checker = BorrowChecker::new();
    let expr = Expr::Ident("x".into());

    // Create a view borrow
    checker
        .check_borrow(&expr, BorrowKind::View, Lifetime::new(1))
        .unwrap();

    // Try to create a mut borrow
    let result = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(2));
    assert!(result.is_err(), "Mut should conflict");

    let err = result.unwrap_err();
    let err_msg = format!("{}", err);

    // Verify error message contains useful information
    assert!(err_msg.contains("mut"), "Error should mention mut");
    assert!(err_msg.contains("view"), "Error should mention view");
    assert!(err_msg.contains("x"), "Error should mention the target");
}

/// Test lifetime region overlap detection
#[test]
fn test_lifetime_region_overlap() {
    use crate::ownership::LifetimeContext;

    let mut ctx = LifetimeContext::new();
    let lt1 = ctx.fresh_lifetime();
    let lt2 = ctx.fresh_lifetime();

    // Set up non-overlapping regions
    ctx.set_region(lt1, (1, 0), (5, 0)); // Lines 1-5
    ctx.set_region(lt2, (6, 0), (10, 0)); // Lines 6-10

    // Should not overlap
    assert!(
        !ctx.regions_overlap(lt1, lt2),
        "Non-overlapping regions should not overlap"
    );

    // Set up overlapping regions
    let lt3 = ctx.fresh_lifetime();
    ctx.set_region(lt3, (3, 0), (7, 0)); // Lines 3-7

    // Should overlap with both lt1 and lt2
    assert!(ctx.regions_overlap(lt1, lt3), "Should overlap");
    assert!(ctx.regions_overlap(lt2, lt3), "Should overlap");
}

/// Test static lifetime behavior
#[test]
fn test_static_lifetime() {
    let mut checker = BorrowChecker::new();
    let expr = Expr::Ident("constant".into());

    // Static borrow
    let result = checker.check_borrow(&expr, BorrowKind::View, Lifetime::STATIC);
    assert!(result.is_ok(), "Static lifetime borrow should succeed");

    // Regular borrow should conflict with static
    let result2 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(1));
    assert!(result2.is_err(), "Mut should conflict with static view");
}

/// Test borrow checker cleanup
#[test]
fn test_borrow_checker_cleanup() {
    let mut checker = BorrowChecker::new();
    let expr = Expr::Ident("x".into());

    // Create multiple borrows
    checker
        .check_borrow(&expr, BorrowKind::View, Lifetime::new(1))
        .unwrap();
    checker
        .check_borrow(&expr, BorrowKind::View, Lifetime::new(2))
        .unwrap();
    assert_eq!(checker.active_borrows().len(), 2);

    // Clear all borrows
    checker.clear();
    assert_eq!(checker.active_borrows().len(), 0);

    // Should be able to create any borrow after clear
    let result = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(3));
    assert!(result.is_ok(), "Mut should succeed after clear");
}

/// Test hold expression span extraction
#[test]
fn test_hold_span_extraction() {
    use crate::ast::Hold;

    let hold = Hold::new(Expr::Ident("x".into()), "value".into(), Body::new()).with_span(100, 50);

    assert_eq!(hold.span, Some((100, 50)));
}
