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
//! - **Dual Payload**: Handlers can be AstBlock (AOT) or Bytecode (AutoVM)
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
//! ```rust
//! use auto_lang::aura::{AuraWidget, AuraNode, AuraStateDef, LogicPayload};
//!
//! // Create a simple widget
//! let widget = AuraWidget {
//!     name: "Counter".to_string(),
//!     state_vars: vec![AuraStateDef {
//!         name: "count".to_string(),
//!         type_info: Type::Int,
//!         initial: Expr::Int(0),
//!     }],
//!     view_tree: AuraNode::Element {
//!         tag: "col".to_string(),
//!         props: HashMap::new(),
//!         events: HashMap::new(),
//!         children: vec![],
//!     },
//!     handlers: HashMap::new(),
//! };
//! ```

mod types;
mod extract;
mod atom;

pub use types::*;
pub use extract::*;
pub use atom::*;
