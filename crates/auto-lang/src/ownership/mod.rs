//! Ownership-based memory management for AutoLang
//!
//! This module implements AutoLang's ownership system, providing:
//! - Linear types (move-only semantics)
//! - "Last use" detection via control flow analysis
//! - Borrow checker with lifetime tracking
//! - Three borrow types: view, mut, take
//! - Automatic cleanup on drop
//! - Zero-cost memory safety without GC
//!
//! # Phases
//!
//! - **Phase 1**: Move semantics ✅
//! - **Phase 2**: Owned string types ✅
//! - **Phase 3**: Borrow checker 🔄 (in progress)
//!
//! # Borrow Types (Phase 3)
//!
//! AutoLang provides three ways to borrow values:
//!
//! 1. **`view`** - Immutable borrow (like Rust `&T`)
//!    - Multiple view borrows can coexist
//!    - Cannot modify the borrowed value
//!    - Original value remains valid
//!
//! 2. **`mut`** - Mutable borrow (like Rust `&mut T`)
//!    - Only one mut borrow at a time
//!    - Cannot coexist with view borrows
//!    - Can modify the borrowed value
//!    - Original value remains valid
//!
//! 3. **`take`** - Move semantics (like Rust `move` or `std::mem::take`)
//!    - Transfers ownership to new location
//!    - Original value no longer valid
//!    - Conflicts with all other borrows
//!
//! # Example
//!
//! ```auto
//! // View borrow (immutable)
//! let s = "hello"
//! let slice = view s      // Immutable borrow
//! let len = str_len(slice)
//! // s still valid here
//!
//! // Mut borrow (mutable)
//! let s = str_new("hello", 10)
//! let mut_ref = mut s     // Mutable borrow
//! str_append(mut_ref, " world")
//! // s modified in place
//!
//! // Take (move semantics)
//! let s1 = "hello"
//! let s2 = take s1        // Move: s1 no longer valid
//! use(s2)                 // Last use: automatic cleanup
//! // s1 cannot be used here (compiler error)
//! ```

pub mod borrow;
pub mod cfa;
pub mod lifetime;

// Re-export Linear trait from auto-val (unified ownership system)
pub use auto_val::{Linear, MoveState, MoveTracker};

pub use borrow::{Borrow, BorrowChecker, BorrowKind};
pub use cfa::LastUseAnalyzer;
pub use lifetime::{Lifetime, LifetimeContext};
