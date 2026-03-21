//! AURA Widget Library
//!
//! This module provides the widget specification system for AURA UI generation.
//! Widget specs define how AURA elements map to different backend components.
//!
//! ## Overview
//!
//! The widget library replaces hardcoded component registries with a flexible,
//! data-driven approach. Each widget is defined by a `WidgetSpec` that contains:
//!
//! - Widget name and category
//! - Primary prop for shorthand syntax
//! - Backend-specific mappings (Vue, Jet, Ark, etc.)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use auto_lang::ui_gen::widget::{WidgetSpec, WidgetCategory, BackendMapping};
//!
//! let mut spec = WidgetSpec::new("Button", WidgetCategory::Form);
//! spec.primary_prop = Some("text".to_string());
//! spec.has_children = true;
//!
//! // Add backend mappings
//! let mapping = BackendMapping {
//!     component: "Button".to_string(),
//!     import: Some("androidx.compose.material3.Button".to_string()),
//!     props: HashMap::new(),
//!     events: HashMap::new(),
//! };
//! spec.backends.insert("jet".to_string(), mapping);
//! ```

mod registry;
mod spec;

pub use registry::WidgetRegistry;
pub use spec::{BackendMapping, WidgetCategory, WidgetSpec};
