//! PLAN-807 text-core consumer fixture — the "external crate" proving the
//! text foundation is consumable without the language/VM/UI stack.
//!
//! Dependency discipline: `auto-text-core` is the ONLY dependency (std
//! beyond it). Any auto-lang/auto-man/auto-ui/auto-vm reference here is a
//! boundary violation caught by verify-deps.sh.
