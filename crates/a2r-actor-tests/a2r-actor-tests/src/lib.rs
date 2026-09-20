//! Plan 387 W5b: a2r-actor-tests — behavior-parity harness.
//!
//! This crate hosts integration tests that verify a2r-transpiled Auto actor
//! programs behave identically to the AutoVM actor suite. Each test:
//!   1. reads an actor `.at` from `test/a2r/22_actors/<case>/`,
//!   2. transpiles it to Rust via `auto_lang::trans::rust::transpile_rust`,
//!   3. writes a throwaway crate (depending on `a2r-std` + `tokio`) into a temp dir,
//!   4. runs `cargo run` in that crate,
//!   5. asserts the captured stdout equals the VM golden `test/vm/23_actor/<case>/*.expected.out`.
//!
//! These tests are SLOW (one `cargo build` per case) and are `#[ignore]`d by
//! default; run explicitly with `cargo test -p a2r-actor-tests -- --ignored`.

// The crate is test-only; nothing to export.
