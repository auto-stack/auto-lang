//! AURA (Auto UI Representation Abstract) - Core Module
//!
//! AURA is the official intermediate representation for AutoUI components.
//! It is extracted from AutoUI source code and serves as the input for
//! multiple backend code generators (React, Compose, GPUI).
//!
//! ## Key Concepts
//!
//! - **Extraction**: Converting WidgetDecl AST to AuraWidget (1:1 lossless mapping)
//! - **Purity**: View tree contains no logic, only layout and bindings
//! - **Handler Payload**: Handlers carry base AST stmts (AstStmts) or pre-compiled Bytecode
//!
//! ## Architecture
//!
//! ```text
//! WidgetDecl (AST)
//!     ↓
//! AuraWidget (Extraction)
//!     ↓
//! Backend Generator (React/Compose/GPUI)
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use auto_lang::aura::{AuraWidget, AuraNode, AuraStateDef, AuraMessage, AuraMsgVariant, LogicPayload};
//! use auto_lang::ast::{Type, Expr};
//!
//! // Create a simple widget
//! let widget = AuraWidget {
//!     name: "Counter".to_string(),
//!     state_vars: vec![AuraStateDef {
//!         name: "count".to_string(),
//!         type_info: Type::Int,
//!         initial: Expr::Int(0),
//!         decorators: vec![],
//!     }],
//!     // ...
//! };
//! ```

mod types;
pub mod extract;
mod atom;
pub mod schema;
pub mod schema_loader;
#[allow(unused)]
pub mod validate;
// Plan 507 T2：元素级 queue 覆盖登记（无 feature 门——schema 漂移围栏
// 日常档可读；运行时消费方 ui/desktop_protocol/coverage.rs 在 ui 档）。
pub mod element_coverage;

pub use types::*;
pub use extract::*;
pub use atom::*;
pub use schema::*;
pub use schema_loader::*;
pub use validate::*;
