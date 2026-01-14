//! Ownership-based memory management for AutoLang
//!
//! This module implements AutoLang's ownership system, providing:
//! - Linear types (move-only semantics)
//! - "Last use" detection via control flow analysis
//! - Automatic cleanup on drop
//! - Zero-cost memory safety without GC
//!
//! # Phases
//!
//! - **Phase 1**: Move semantics only (current)
//! - **Phase 2**: Borrow checker (future)
//! - **Phase 3**: Lifetime tracking (future)
//!
//! # Example
//!
//! ```auto
//! let s = String_new("hello", 5)  // Owns string
//! let t = s                        // Move: s no longer valid
//! use(t)                            // Last use: automatic cleanup
//! ```

pub mod cfa;

// Re-export Linear trait from auto-val (unified ownership system)
pub use auto_val::{Linear, MoveState, MoveTracker};

pub use cfa::LastUseAnalyzer;
