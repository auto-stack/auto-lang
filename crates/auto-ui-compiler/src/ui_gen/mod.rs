//! UI Backend Generators
//!
//! This module provides code generators for various UI backends:
//! - **Vue3/JavaScript**: Vue3 SFC (Single File Component) generator
//! - **Rust/AutoUI**: Rust Component trait generator using auto-ui abstraction
//! - **Jetpack Compose**: Kotlin/Compose for Android
//!
//! The Rust generator produces code using the abstract AutoUI components from
//! the `auto-ui` crate. The auto-ui crate then handles backend-specific
//! implementations (Iced etc.).
//!
//! All generators take `AuraWidget` as input and produce target-specific code.

pub mod shared;
// Plan 482: nav-item/nav-group class-token contract (Vue scaffold ↔ VM builder).
pub mod nav_contract;
// Plan 561: sidebar_* class-token contract (VM 契约子集 ↔ shadcn 原版资产逐 token 锚).
pub mod bp;
pub mod import_policy;
pub mod rust;
pub mod sidebar_contract;
pub mod ts_adapter;
pub mod vue;
// PLAN-674 T-03/T-04：a2r codegen prop/event 词汇单源（per-kind
// builder 方法面；表体落 ui_gen 的组合律依据见 vocab.rs 头注）。
pub mod api;
pub mod ark;
pub mod ark_adapter;
pub mod docs_gen;
pub mod jet;
pub mod kotlin_adapter;
pub mod style;
pub mod validators;
pub mod vocab;
pub mod widget;

// Re-export main types
pub use jet::JetGenerator;
pub use rust::RustGenerator;
pub use style::StyleGenerator;
pub use validators::{validate_sfc, Severity, ValidationContext, ValidationWarning};
pub use vue::VueGenerator;
pub use vue::VueMode;
pub use widget::{WidgetCategory, WidgetRegistry, WidgetSpec};

// Re-export transpiler API (Plan 175 Phase 3 + Plan 361 §3)
pub use api::{
    generate_component_from_file, transpile_aura, transpile_file, transpile_vue_aura,
    ComponentGenOptions, GeneratedComponent,
};

use crate::aura::AuraWidget;

/// Generation error
#[derive(Debug, Clone)]
pub enum GenError {
    /// Unsupported expression type
    UnsupportedExpr(String),

    /// Unsupported statement type
    UnsupportedStmt(String),

    /// Invalid state reference
    InvalidStateRef(String),

    /// IO error
    Io(String),

    /// Unknown widget requested from the library template table (Plan 331)
    UnknownWidget(String),
}

impl std::fmt::Display for GenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenError::UnsupportedExpr(msg) => write!(f, "Unsupported expression: {}", msg),
            GenError::UnsupportedStmt(msg) => write!(f, "Unsupported statement: {}", msg),
            GenError::InvalidStateRef(msg) => write!(f, "Invalid state reference: {}", msg),
            GenError::Io(msg) => write!(f, "IO error: {}", msg),
            GenError::UnknownWidget(msg) => write!(f, "Unknown widget: {}", msg),
        }
    }
}

impl std::error::Error for GenError {}

pub type GenResult<T> = Result<T, GenError>;

/// Backend generator trait
pub trait BackendGenerator {
    /// Generate code from an AuraWidget
    fn generate(&mut self, widget: &AuraWidget) -> GenResult<String>;

    /// Get the file extension for generated code
    fn extension(&self) -> &'static str;
}

// PLAN-814 T-03：normalize_shortcut 单源迁 auto-ui-compiler
// action_config（vue 生成器同 crate 消费）；旧路径经再导出保持
// （plan-451 非 ui 消费者语义不变）。
pub use crate::action_config::normalize_shortcut;
