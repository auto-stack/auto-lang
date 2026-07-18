//! Plan 359 Phase 5 (G1/G2/G3): Concurrency bug fixes.
//!
//! - G1: `~{ ... }.go` (spawn) previously caused a stack underflow because the
//!   `Expr::Go` codegen pushed a single Future value while the `SPAWN_GO`
//!   engine handler expected a `[func_addr, arg_count, args...]` stack layout,
//!   and because the statement-level POP after SPAWN_GO ate a value below the
//!   stack. These tests pin the fixed behavior.
//! - G2: `Handle[T]` / `List[T]` square-bracket generic type syntax is now
//!   accepted in type position, matching the existing `<T>` form.

#[cfg(test)]
mod plan359_concurrency_tests {
    use crate::run_with_capture;

    // ====================================================================
    // G1: `.go` spawn no longer crashes the VM
    // ====================================================================

    /// G1: An async block spawned with `.go` must not crash the VM.
    ///
    /// Before the fix, `SPAWN_GO` popped `func_addr`, `arg_count`, and args
    /// from a stack that only held a single Future value, causing an immediate
    /// stack underflow. The fix makes `SPAWN_GO` pop exactly the single value
    /// pushed by the `Expr::Go` codegen.
    #[test]
    fn test_g1_go_spawn_does_not_crash() {
        let code = r#"
fn main() {
    var counter int = 0
    ~{
        counter = 42
    }.go
    print(counter.to(str))
}
"#;
        let result = run_with_capture(code);
        // The key assertion: the VM completes without a stack-underflow error.
        assert!(
            result.is_ok(),
            "spawn (.go) crashed the VM: {:?}",
            result.err()
        );
        let (_result_str, _stdout) = result.unwrap();
        // We do not assert on the printed value: whether the spawned task's
        // body has run before `print` is a separate timing concern. The fix
        // only guarantees no stack underflow.
    }

    /// G1: Multiple `.go` spawns must not corrupt the caller's stack.
    ///
    /// A second defect left a stray statement-level POP after SPAWN_GO (which
    /// returns void), so each spawn ate one value from below the stack — the
    /// crash only surfaced after several spawns once the stack drained.
    #[test]
    fn test_g1_multiple_go_spawns_no_crash() {
        let code = r#"
fn main() {
    var i int = 0
    ~{
        i = 1
    }.go
    ~{
        i = 2
    }.go
    ~{
        i = 3
    }.go
    print("done")
}
"#;
        let result = run_with_capture(code);
        assert!(
            result.is_ok(),
            "multiple .go spawns crashed the VM: {:?}",
            result.err()
        );
        let (_result_str, stdout) = result.unwrap();
        assert!(
            stdout.contains("done"),
            "expected 'done' in output, got: {:?}",
            stdout
        );
    }

    // ====================================================================
    // G2: Square-bracket generic type syntax `Ident[T]`
    // ====================================================================

    /// G2: `Handle[int]` parses the same as `Handle<int>`.
    #[test]
    fn test_g2_handle_square_bracket() {
        let code = r#"
fn main() {
    var h Handle[int]
    print("ok")
}
"#;
        let result = run_with_capture(code);
        assert!(
            result.is_ok(),
            "Handle[int] failed to parse/run: {:?}",
            result.err()
        );
        let (_r, stdout) = result.unwrap();
        assert!(stdout.contains("ok"), "got: {:?}", stdout);
    }

    /// G2: `List[int]` and other generics accept square brackets.
    #[test]
    fn test_g2_generic_square_bracket() {
        let code = r#"
fn main() {
    var l List[int]
    var m Map[str, int]
    print("ok")
}
"#;
        let result = run_with_capture(code);
        assert!(
            result.is_ok(),
            "square-bracket generics failed: {:?}",
            result.err()
        );
    }

    /// G2: Nested square-bracket generics parse (e.g. `List[Handle[int]]`).
    #[test]
    fn test_g2_nested_generic_square_bracket() {
        let code = r#"
fn main() {
    var l List[Handle[int]]
    print("ok")
}
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "nested generics failed: {:?}", result.err());
    }

    /// G2: Square-bracket generics work in function parameter position too.
    #[test]
    fn test_g2_generic_square_bracket_in_fn_param() {
        let code = r#"
fn take(h Handle[int]) {
    print("ok")
}
fn main() {
    take(0)
}
"#;
        let result = run_with_capture(code);
        assert!(
            result.is_ok(),
            "generic in fn param failed: {:?}",
            result.err()
        );
    }

    /// G2: Regression guard — array type syntax `[N]T` still works and is not
    /// confused with the new `Ident[T]` generic syntax.
    #[test]
    fn test_g2_array_type_still_works() {
        let code = r#"
fn main() {
    var a [4]int
    a[0] = 1
    print(a[0].to(str))
}
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "array type regressed: {:?}", result.err());
        let (_r, stdout) = result.unwrap();
        assert!(stdout.contains("1"), "expected '1', got: {:?}", stdout);
    }
}
