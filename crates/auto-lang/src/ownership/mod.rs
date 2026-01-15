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
//! AutoLang provides three keywords for memory management:
//!
//! 1. **`view`** - Immutable borrow (like Rust `&T`)
//!    - Multiple view borrows can coexist
//!    - Cannot modify the borrowed value
//!    - Original value remains valid
//!
//! 2. **`mut`** - Mutable borrow (like Rust `&mut T`)
//!    - Only one mut borrow at a time
//!    - Cannot coexist with other borrows
//!    - Can modify the borrowed value
//!    - Original value remains valid
//!
//! 3. **`take`** - Move semantics (like Rust `move`)
//!    - Transfers ownership to new location
//!    - Original value no longer valid
//!    - Conflicts with all other borrows
//!
//! # Example
//!
//! ```auto
//! // View borrow (immutable reference)
//! let s = str_new("hello", 5)
//! let slice = view s      // Immutable borrow: like &s in Rust
//! let len = str_len(slice) // Can read through the borrow
//! // s still valid here, both s and slice can be used
//!
//! // Mut borrow (mutable reference)
//! let s = str_new("hello", 5)
//! let mut_ref = mut s     // Mutable borrow: like &mut s in Rust
//! str_push(mut_ref, '!')  // Can modify through the borrow
//! // s reflects the modification
//!
//! // Take (move semantics)
//! let s1 = str_new("hello", 5)
//! let s2 = take s1        // Move: s1 invalidated, ownership transferred
//! str_drop(s2)            // s2 owns the data now
//! // s1 cannot be used here (use-after-move error)
//! ```

pub mod borrow;
pub mod cfa;
pub mod lifetime;

// Re-export Linear trait from auto-val (unified ownership system)
pub use auto_val::{Linear, MoveState, MoveTracker};

pub use borrow::{Borrow, BorrowChecker, BorrowKind};
pub use cfa::LastUseAnalyzer;
pub use lifetime::{Lifetime, LifetimeContext};
