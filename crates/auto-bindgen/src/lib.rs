//! auto-bindgen: C header manifest types and built-in definitions.
//!
//! This crate provides:
//! - `manifest` — Serde types for JSON manifests (`CHeaderManifest`, `CFunction`, etc.)
//! - `extractor` — Hard-coded manifests for 5 standard C headers
//! - `type_map` — Utility conversions for CTypeDesc
//! - `rust_import` — PLAN-967 F0 annotated Rust provider → neutral manifest,
//!   C-compatible shim source and the shared `ace.foreign.manual/2` manifest
//!
//! Plan 216 Phase 1; rust_import is PLAN-967 T-F0-I.

pub mod extractor;
pub mod manifest;
pub mod rust_import;
pub mod type_map;
