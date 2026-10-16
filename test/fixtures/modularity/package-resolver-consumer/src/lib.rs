//! PLAN-800 V01 fixture — external consumer of `auto-package-resolver`.
//!
//! This crate exists to prove the resolver core compiles and runs outside
//! the main workspace without the language/AST/Man/VM/UI stack; see the
//! tests and `verify-deps.sh` for the closure oracle.
