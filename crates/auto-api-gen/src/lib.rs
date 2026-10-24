//! # auto-api-gen
//!
//! API server/client generation core (PLAN-815 T-04) — the single-algorithm
//! home for the generic API extraction/transpilation and rust-server /
//! vue-client / tauri-command generation extracted from `auto-man`'s
//! `api_gen.rs`, plus the `rust_sidecar` effect adapter and the shared
//! rust-generation workspace layout.
//!
//! Owner: **lang-service-adapter** (777 `split-man-service-gen`; domain
//! registration `man-ecosystem` — consumed from both the `man` domain
//! (`auto-man::http_service`) and the `ui` domain (`auto-ui-tools`), so it
//! must sit below both. It does not move *with* the UI tools and holds no
//! UI-tool dependency.
//!
//! Boundaries:
//! - depends on the language SDK (`auto-lang-core`) for api/ast/trans/config
//!   and `auto-project-model` for the `rust_sidecar` declaration single source (PLAN-831 T-05);
//! - never depends on `auto-man` or `auto-ui-tools` (facade direction only:
//!   those crates re-export from here);
//! - generated-code template strings deliberately keep referencing the
//!   `auto_lang` crate — that is the *generated app's* dependency, not ours.
//!
//! Historic public paths stay alive through the `auto-man` facade
//! (`auto_man::api_gen::*`, `auto_man::sidecar::*`,
//! `auto_man::rust_ui::{ensure_shared_workspace, ...}`), so existing
//! consumers compile unchanged.

pub mod api_gen;
pub mod rust_workspace;
pub mod sidecar;
