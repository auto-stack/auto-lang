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
//! use auto_lang::aura::{AuraWidget, AuraNode, AuraStateDef, AuraMessage, AuraMsgVariant, AuraExpr, LogicPayload};
//! use auto_lang::ast::Type;
//!
//! // Create a simple widget
//! let widget = AuraWidget {
//!     name: "Counter".to_string(),
//!     state_vars: vec![AuraStateDef {
//!         name: "count".to_string(),
//!         type_info: Type::Int,
//!         initial: AuraExpr::Int(0),
//!     }],
//!     messages: vec![AuraMessage {
//!         name: "Msg".to_string(),
//!         variants: vec![
//!             AuraMsgVariant { name: "Inc".to_string(), payload: None },
//!             AuraMsgVariant { name: "Dec".to_string(), payload: None },
//!         ],
//!     }],
//!     view_tree: AuraNode::Element {
//!         tag: "col".to_string(),
//!         props: std::collections::HashMap::new(),
//!         events: std::collections::HashMap::new(),
//!         children: vec![],
//!     },
//!     handlers: std::collections::HashMap::new(),
//!     props: vec![],
//! };
//! ```

mod types;
mod extract;
mod atom;

pub use types::*;
pub use extract::*;
pub use atom::*;
