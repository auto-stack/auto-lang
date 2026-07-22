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

pub use types::*;
pub use extract::*;
pub use atom::*;
pub use schema::*;
pub use schema_loader::*;
pub use validate::*;
