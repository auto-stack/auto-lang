//! Plan 348 Phase 5 (G1/G2/G3): Concurrency bug fixes.
//!
//! - G1: `~{ ... }.go` (spawn) previously caused a stack underflow because the
//!   `Expr::Go` codegen pushed a single Future value while the `SPAWN_GO`
//!   engine handler expected a `[func_addr, arg_count, args...]` stack layout,
//!   and because the statement-level POP after SPAWN_GO ate a value below the
//!   stack. These tests pin the fixed behavior.
//! - G2: `Handle[T]` / `List[T]` square-bracket generic type syntax is now
//!   accepted in type position, matching the existing `<T>` form.

#[cfg(test)]
mod plan348_concurrency_tests {
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

    // ====================================================================
    // Task 22: `.go` spawn truly runs the body as a separate task
    // ====================================================================
    //
    // Before Task 22, `Expr::AsyncBlock` compiled the body INLINE and passed
    // CREATE_FUTURE a placeholder body_offset of 0. SPAWN_GO's guard
    // (`if body_offset != 0 && body_offset < len`) therefore skipped the real
    // `spawn_task` call, so `~{ counter = 42 }.go` ran the body synchronously
    // in the caller. The Task 22 fix compiles the body out-of-line (like a
    // closure) and feeds CREATE_FUTURE a real bytecode address, so SPAWN_GO
    // spawns a genuine background task and the caller's locals are untouched.

    /// Task 22: `~{ ... }.go` spawns a real task. The caller's `counter` is
    /// NOT mutated synchronously, so it stays at its initial value (0) when
    /// read immediately after the spawn. (Before the fix, the body ran inline
    /// and `counter` would be 42 by the time `print` ran.)
    #[test]
    fn test_task22_spawn_does_not_run_inline() {
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
        assert!(
            result.is_ok(),
            "spawn crashed the VM: {:?}",
            result.err()
        );
        let (_r, stdout) = result.unwrap();
        // The spawned task runs in its own RAM with its own captured copy of
        // `counter`, so the caller's `counter` stays 0. If the body were still
        // compiled inline (the pre-Task-22 bug), this would print 42.
        assert!(
            stdout.trim().ends_with('0'),
            "expected caller's counter to stay 0 (true spawn), got: {:?}",
            stdout
        );
    }

    /// Task 22: `.await` on an async block still returns the body's value.
    /// The body is compiled out-of-line and executed in the caller's task via
    /// handle_await_future; the synthetic closure installed for captures must
    /// not corrupt the caller's existing closure context.
    #[test]
    fn test_task22_await_returns_body_value() {
        let code = r#"
fn main() {
    var result int = ~{ 42 }.await
    print(result.to(str))
}
"#;
        let result = run_with_capture(code);
        assert!(result.is_ok(), "await crashed the VM: {:?}", result.err());
        let (_r, stdout) = result.unwrap();
        assert!(
            stdout.contains("42"),
            "expected await to return 42, got: {:?}",
            stdout
        );
    }

    /// Task 22: `.go` spawn followed by `.await` on another block in the same
    /// function. Both paths must coexist without crashing or corrupting the
    /// stack — spawn installs a synthetic closure on the spawned task, while
    /// await installs one on the current task and restores it afterwards.
    #[test]
    fn test_task22_spawn_then_await_coexist() {
        let code = r#"
fn main() {
    var x int = 0
    ~{ x = 1 }.go
    var y int = ~{ 7 }.await
    print(y.to(str))
    print(x.to(str))
}
"#;
        let result = run_with_capture(code);
        assert!(
            result.is_ok(),
            "spawn+await crashed the VM: {:?}",
            result.err()
        );
        let (_r, stdout) = result.unwrap();
        // await returns the body's value (7); caller's x stays 0 (spawn does
        // not write back to the caller's locals).
        assert!(stdout.contains("7"), "expected y=7, got: {:?}", stdout);
        assert!(
            stdout.contains('0'),
            "expected caller x to stay 0, got: {:?}",
            stdout
        );
    }
}
