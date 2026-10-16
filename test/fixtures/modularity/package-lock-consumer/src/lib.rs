//! PLAN-795 V01 fixture — external consumer of `auto-package-lock`.
//!
//! This crate exists to prove the pac.lock core compiles and runs outside
//! the main workspace without the language/VM/renderer stack; see the tests
//! and `verify-deps.sh` for the closure oracle.
