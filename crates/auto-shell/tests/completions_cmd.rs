//! Plan 315 Phase 2 — `completions` builtin integration tests.
//!
//! Exercises the management subcommands via the public `Shell::execute` API.
//! Avoids the destructive `clear --cache` (which would wipe the real cache dir);
//! tests use read-only subcommands or a unique non-existent name for `clear`.

use auto_shell::shell::Shell;

#[test]
fn completions_no_args_shows_help() {
    let mut shell = Shell::new();
    let out = shell.execute("completions").unwrap().unwrap();
    assert!(out.contains("USAGE"), "help text:\n{out}");
    assert!(out.contains("generate"));
    assert!(out.contains("tiers"));
}

#[test]
fn completions_path_lists_three_tiers() {
    let mut shell = Shell::new();
    let out = shell.execute("completions path").unwrap().unwrap();
    assert!(out.contains("user:"), "path output:\n{out}");
    assert!(out.contains("generated:"));
    assert!(out.contains("cache:"));
}

#[test]
fn completions_list_runs_without_panic() {
    let mut shell = Shell::new();
    let out = shell.execute("completions list").unwrap().unwrap();
    // Each tier label present.
    assert!(out.contains("user"));
    assert!(out.contains("generated"));
    assert!(out.contains("cache"));
}

#[test]
fn completions_generate_unknown_command_errors() {
    let mut shell = Shell::new();
    // A command that is essentially guaranteed not to exist → --help yields no
    // output → generate should error (not panic).
    let res = shell.execute("completions generate ash_definitely_not_a_cmd_98765");
    assert!(res.is_err(), "expected error for unknown command, got: {:?}", res);
}

#[test]
fn completions_clear_missing_entry_reports_zero() {
    let mut shell = Shell::new();
    // Clearing a unique non-existent name is safe and reports 0 removed.
    let out = shell
        .execute("completions clear ash_unique_clear_test_424242")
        .unwrap()
        .unwrap();
    assert!(out.contains("removed 0"), "clear output:\n{out}");
}

#[test]
fn completions_unknown_subcommand_errors() {
    let mut shell = Shell::new();
    let res = shell.execute("completions frobnicate");
    assert!(res.is_err());
}
