//! Plan 315 Phase 1 — runtime integration test for arbitrary-command completion.
//!
//! Verifies the cache-tier path of `ensure_spec`: a command with NO built-in /
//! user / generated spec gets completion from the cache tier. (The `--help`
//! probe parse logic is unit-tested in `help_parser`; here we pre-seed the
//! cache so the test is deterministic and doesn't depend on any command being
//! on PATH.)

use ash_core::completions::spec::{CompletionSpec, FlagSpec};
use ash_core::completions::CompletionProvider;
use auto_shell::completions::reedline::{CompletionState, ShellCompleter};
use auto_shell::completions::spec_tiers;
use reedline::Completer;

/// Unique fake command name (no real command, no built-in spec) → exercises the
/// runtime tier path without probing a real binary.
const FAKE_CMD: &str = "ash_zzz_probe_test_cmd_xyz";

fn spec_with_flag() -> CompletionSpec {
    CompletionSpec::new(FAKE_CMD).flag(FlagSpec::long("zzz-distinctive-flag"))
}

#[test]
fn cache_tier_completion_works_end_to_end() {
    // 1. Pre-seed the cache tier with a spec for the fake command.
    let spec = spec_with_flag();
    spec_tiers::write_cache(FAKE_CMD, &spec).expect("write cache");

    // 2. Build a ShellCompleter with an empty provider (no built-ins for this
    //    command) and empty signatures (not a registered command).
    let provider = CompletionProvider::new();
    let state = std::sync::Arc::new(std::sync::Mutex::new(CompletionState::new(
        std::env::current_dir().unwrap(),
    )));
    let mut completer = ShellCompleter::new(vec![], provider, state);

    // 3. Complete `FAKE_CMD --` → ensure_spec loads the cache spec, resolve
    //    returns the distinctive flag.
    let line = format!("{} --", FAKE_CMD);
    let suggestions = completer.complete(&line, line.len());

    // 4. Cleanup the seeded cache file regardless of assertion outcome.
    let _ = spec_tiers::cache_dir().map(|d| std::fs::remove_file(d.join(format!("{}.at", FAKE_CMD))));

    let values: Vec<String> = suggestions.iter().map(|s| s.value.clone()).collect();
    assert!(
        values.iter().any(|v| v.contains("zzz-distinctive-flag")),
        "expected the cached flag in suggestions, got: {:?}",
        values
    );
}

#[test]
fn unknown_command_without_cache_falls_back() {
    // A command with no spec anywhere and no cache → ensure_spec probes
    // `<cmd> --help` (fails for a fake name), registers nothing, and complete()
    // falls through to default completion (empty/registry). Must not panic and
    // must not register a bogus spec.
    const NOPE: &str = "ash_definitely_not_a_real_cmd_12345";
    // Ensure no cache entry pollutes this.
    let _ = spec_tiers::cache_dir().map(|d| std::fs::remove_file(d.join(format!("{}.at", NOPE))));

    let provider = CompletionProvider::new();
    let state = std::sync::Arc::new(std::sync::Mutex::new(CompletionState::new(
        std::env::current_dir().unwrap(),
    )));
    let mut completer = ShellCompleter::new(vec![], provider, state);

    let line = format!("{} --", NOPE);
    let _ = completer.complete(&line, line.len()); // must not panic

    // The probe should have written a (possibly empty) cache marker; clean up.
    let _ = spec_tiers::cache_dir().map(|d| std::fs::remove_file(d.join(format!("{}.at", NOPE))));
}
