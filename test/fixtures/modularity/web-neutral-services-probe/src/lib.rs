//! PLAN-890 bounded proof: compile the existing photo service unchanged,
//! without depending on the UI crate. The pinned sibling source remains
//! read-only until the feasibility gate permits its eventual owner move.
#[path = "../../../../../../auto-ui/crates/auto-ui-runtime/src/ui/photo_service.rs"]
pub mod photo_service;
