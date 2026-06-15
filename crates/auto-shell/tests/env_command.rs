//! Integration tests for the `env` / `env.path` commands.
//!
//! Plan 301 / Plan 309 Task 1.2 — Phase 2.
//!
//! Uses only the public `Shell::execute` API. Runs via `cargo test --test env_command`,
//! which compiles the lib WITHOUT its internal `#[cfg(test)]` modules (those have
//! pre-existing compile errors unrelated to env), so these tests can run in isolation.
//!
//! Parallel-test safety: process env is shared across test threads, so each test uses
//! a DISTINCT unique variable name, and ALL PATH mutations live in a single test.

use auto_shell::shell::Shell;

/// Set a value, then query it back through `env`.
#[test]
fn env_set_then_query() {
    let mut shell = Shell::new();
    shell.execute("env ASH_ENV_IT_SET=hello").unwrap();
    let out = shell.execute("env ASH_ENV_IT_SET").unwrap();
    assert_eq!(out.as_deref(), Some("hello"));
    // Visible to child processes too.
    assert_eq!(std::env::var("ASH_ENV_IT_SET").unwrap(), "hello");
    std::env::remove_var("ASH_ENV_IT_SET");
}

/// Querying an absent variable returns an empty string, not an error (Plan 301 §1.6).
#[test]
fn env_query_absent_is_empty() {
    let mut shell = Shell::new();
    std::env::remove_var("ASH_ENV_IT_ABSENT");
    let out = shell.execute("env ASH_ENV_IT_ABSENT").unwrap();
    assert_eq!(out.as_deref(), Some(""));
}

/// `env -rm NAME` removes the variable.
#[test]
fn env_rm_removes_variable() {
    let mut shell = Shell::new();
    shell.execute("env ASH_ENV_IT_RM=temp").unwrap();
    assert_eq!(std::env::var("ASH_ENV_IT_RM").unwrap(), "temp");
    shell.execute("env -rm ASH_ENV_IT_RM").unwrap();
    assert!(std::env::var("ASH_ENV_IT_RM").is_err());
}

/// Removing PATH must be refused (Plan 301 §1.6).
#[test]
fn env_rm_path_refused() {
    let mut shell = Shell::new();
    let res = shell.execute("env -rm PATH");
    assert!(res.is_err(), "removing PATH must error");
}

/// `env` with no args renders a table that includes a known variable.
#[test]
fn env_list_renders_table() {
    let mut shell = Shell::new();
    shell.execute("env ASH_ENV_IT_LIST=listval").unwrap();
    let out = shell.execute("env").unwrap().unwrap();
    assert!(out.contains("NAME"), "table header missing: {out}");
    assert!(out.contains("ASH_ENV_IT_LIST"));
    assert!(out.contains("listval"));
    std::env::remove_var("ASH_ENV_IT_LIST");
}

/// All PATH mutations in ONE test to avoid parallel races on the shared process PATH.
#[test]
fn env_path_operations() {
    let saved_path = std::env::var("PATH").ok();
    let mut shell = Shell::new();

    // `env.path` renders a table.
    let table = shell.execute("env.path").unwrap().unwrap();
    assert!(table.contains("PATH"), "path table header missing: {table}");
    assert!(table.contains("EXISTS"));

    // add → appears in the table.
    shell.execute("env.path add /ash_it_path_marker").unwrap();
    let t = shell.execute("env.path").unwrap().unwrap();
    assert!(t.contains("/ash_it_path_marker"), "add did not append: {t}");

    // pre → moves to front (appears before the marker added by `add`, i.e. index 0).
    shell.execute("env.path pre /ash_it_path_front").unwrap();
    let t = shell.execute("env.path").unwrap().unwrap();
    let front_line = t
        .lines()
        .find(|l| l.contains("/ash_it_path_front"))
        .unwrap();
    assert!(front_line.trim_start().starts_with('0'), "pre did not place at index 0: {front_line}");

    // rm by path → removes the marker.
    shell.execute("env.path rm /ash_it_path_marker").unwrap();
    let t = shell.execute("env.path").unwrap().unwrap();
    assert!(!t.contains("/ash_it_path_marker"), "rm by path failed: {t}");

    // dedup: add the front marker twice, then dedup removes the duplicate.
    shell.execute("env.path add /ash_it_path_front").unwrap();
    let before = shell.execute("env.path").unwrap().unwrap();
    let occ_before = before.matches("/ash_it_path_front").count();
    assert_eq!(occ_before, 2, "expected 2 occurrences before dedup, got {occ_before}");
    shell.execute("env.path dedup").unwrap();
    let after = shell.execute("env.path").unwrap().unwrap();
    let occ_after = after.matches("/ash_it_path_front").count();
    assert_eq!(occ_after, 1, "dedup did not collapse duplicates: {after}");

    // rm by index #0 (now the front marker) succeeds; out-of-range errors.
    shell.execute("env.path rm #0").unwrap();
    assert!(shell.execute("env.path rm #99999").is_err(), "out-of-range index must error");

    // Restore the process PATH exactly.
    match saved_path {
        Some(p) => std::env::set_var("PATH", p),
        None => std::env::remove_var("PATH"),
    }
}

// ── Plan 309 Task 1.2 Phase 3: inline K=V env prefixes (execution side) ──

/// `VAR=val cmd` — the var is visible to the command, then restored.
#[test]
fn env_prefix_visible_then_restored() {
    let mut shell = Shell::new();
    std::env::remove_var("ASH_IT_PREFIX");

    // The prefix var is visible during the command.
    let out = shell.execute("ASH_IT_PREFIX=hello echo $ASH_IT_PREFIX").unwrap();
    assert_eq!(out.as_deref(), Some("hello"));

    // After the scoped command, the var is gone (restored to absent).
    let after = shell.execute("env ASH_IT_PREFIX").unwrap();
    assert_eq!(after.as_deref(), Some(""), "scoped var leaked past the command");
}

/// Multiple leading prefixes.
#[test]
fn env_prefix_multiple() {
    let mut shell = Shell::new();
    let out = shell
        .execute("ASH_IT_A=1 ASH_IT_B=2 echo $ASH_IT_A $ASH_IT_B")
        .unwrap();
    assert_eq!(out.as_deref(), Some("1 2"));
    std::env::remove_var("ASH_IT_A");
    std::env::remove_var("ASH_IT_B");
}

/// Assignment-only (`VAR=val` with no command) persists in the current shell
/// (bash semantics), unlike the scoped prefix+command form.
#[test]
fn env_prefix_assignment_only_persists() {
    let mut shell = Shell::new();
    std::env::remove_var("ASH_IT_ASSIGN");
    let out = shell.execute("ASH_IT_ASSIGN=persisted").unwrap();
    assert_eq!(out, None, "assignment-only should produce no output");
    // Persists for a later query.
    let q = shell.execute("env ASH_IT_ASSIGN").unwrap();
    assert_eq!(q.as_deref(), Some("persisted"));
    std::env::remove_var("ASH_IT_ASSIGN");
}

/// Quoted prefix value (with a space) is passed through correctly.
#[test]
fn env_prefix_quoted_value() {
    let mut shell = Shell::new();
    let out = shell
        .execute("ASH_IT_Q=\"hi there\" echo $ASH_IT_Q")
        .unwrap();
    assert_eq!(out.as_deref(), Some("hi there"));
    std::env::remove_var("ASH_IT_Q");
}

