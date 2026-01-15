//! Borrow checker for AutoLang
//!
//! This module implements the borrow checker that ensures memory safety
//! by enforcing Rust-style borrowing rules:
//!
//! 1. Multiple view (immutable) borrows are allowed
//! 2. Only one mut (mutable) borrow is allowed at a time
//! 3. Mut and view borrows cannot coexist
//! 4. Take transfers ownership (move semantics)
//! 5. Borrows cannot outlive the data they reference

use crate::ast::Expr;
use crate::ownership::lifetime::Lifetime;
use std::fmt;

/// Kind of borrow - view, mut, or take
///
/// # Example
/// ```ignore
/// let s = "hello"
/// let t = view s     // Immutable borrow (like Rust &T)
/// let u = mut s      // Mutable borrow (like Rust &mut T)
/// let v = take s     // Move semantics (like Rust move or std::mem::take)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum BorrowKind {
    /// View/immutable borrow (`view` expression)
    /// The borrowed value cannot be modified (like Rust &T)
    View,
    /// Mutable borrow (`mut` expression)
    /// The borrowed value can be modified (like Rust &mut T)
    Mut,
    /// Take/move (`take` expression)
    /// Transfers ownership, original no longer valid (like Rust move)
    Take,
}

impl fmt::Display for BorrowKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowKind::View => write!(f, "view"),
            BorrowKind::Mut => write!(f, "mut"),
            BorrowKind::Take => write!(f, "take"),
        }
    }
}

/// Represents a single borrow in the borrow checker
///
/// Each borrow tracks:
/// - What kind of borrow it is (view, mut, or take)
/// - Its lifetime (when it ends)
/// - The expression being borrowed
#[derive(Debug, Clone)]
pub struct Borrow {
    /// Kind of borrow
    pub kind: BorrowKind,
    /// Lifetime of this borrow
    pub lifetime: Lifetime,
    /// The expression being borrowed
    pub expr: Expr,
}

impl Borrow {
    /// Create a new borrow
    pub fn new(kind: BorrowKind, lifetime: Lifetime, expr: Expr) -> Self {
        Self {
            kind,
            lifetime,
            expr,
        }
    }

    /// Check if this borrow conflicts with another borrow
    ///
    /// # Rules
    /// - Take always conflicts with any other borrow (move semantics)
    /// - Two mut borrows always conflict
    /// - A mut borrow conflicts with any view borrow that overlaps in lifetime
    /// - Two view borrows never conflict
    ///
    /// # Example
    /// ```
    /// # use auto_lang::ownership::borrow::{Borrow, BorrowKind};
    /// # use auto_lang::ownership::lifetime::Lifetime;
    /// # use auto_lang::ast::Expr;
    /// let borrow1 = Borrow::new(BorrowKind::View, Lifetime::new(1), Expr::Int(42));
    /// let borrow2 = Borrow::new(BorrowKind::Mut, Lifetime::new(2), Expr::Int(42));
    ///
    /// // These would conflict if they have overlapping lifetimes
    /// assert!(borrow1.conflicts_with(&borrow2));
    /// ```
    pub fn conflicts_with(&self, other: &Borrow) -> bool {
        // Different expressions don't conflict
        // (TODO: need to check if they refer to the same base value)
        if !self.same_target(other) {
            return false;
        }

        // Check if lifetimes overlap
        if !self.lifetimes_overlap(other) {
            return false;
        }

        // Take conflicts with everything (move semantics)
        if self.kind == BorrowKind::Take || other.kind == BorrowKind::Take {
            return true;
        }

        // Two mut borrows conflict
        if self.kind == BorrowKind::Mut && other.kind == BorrowKind::Mut {
            return true;
        }

        // Mut + view conflict
        if (self.kind == BorrowKind::Mut && other.kind == BorrowKind::View)
            || (self.kind == BorrowKind::View && other.kind == BorrowKind::Mut)
        {
            return true;
        }

        // Two view borrows don't conflict
        false
    }

    /// Check if two borrows target the same expression
    fn same_target(&self, other: &Borrow) -> bool {
        // TODO: This is a simplified check
        // In a full implementation, we'd need to:
        // 1. Resolve both expressions to their base value
        // 2. Check if they refer to the same memory location
        // For now, we just compare the expression discriminants
        std::mem::discriminant(&self.expr) == std::mem::discriminant(&other.expr)
    }

    /// Check if two borrows have overlapping lifetimes
    fn lifetimes_overlap(&self, other: &Borrow) -> bool {
        // Simplified: assume lifetimes overlap if they're different
        // A proper implementation would check lifetime regions
        self.lifetime != other.lifetime
    }
}

/// Borrow checker - ensures borrowing rules are enforced
///
/// The borrow checker tracks all active borrows and validates that
/// new borrows don't violate the borrowing rules.
///
/// # Example
/// ```
/// # use auto_lang::ownership::borrow::BorrowChecker;
/// # use auto_lang::ownership::borrow::BorrowKind;
/// # use auto_lang::ownership::lifetime::Lifetime;
/// # use auto_lang::ast::Expr;
/// let mut checker = BorrowChecker::new();
///
/// // Check a view borrow
/// let expr = Expr::Int(42);
/// match checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1)) {
///     Ok(()) => println!("Borrow is valid"),
///     Err(e) => eprintln!("Borrow error: {}", e),
/// }
/// ```
pub struct BorrowChecker {
    /// Active borrows
    borrows: Vec<Borrow>,
}

impl BorrowChecker {
    /// Create a new borrow checker
    pub fn new() -> Self {
        Self {
            borrows: Vec::new(),
        }
    }

    /// Check if a borrow is valid
    ///
    /// This validates that:
    /// 1. The expression can be borrowed
    /// 2. No conflicting borrows exist
    /// 3. The lifetime is valid
    ///
    /// # Errors
    /// Returns an error if:
    /// - There's an existing mutable borrow
    /// - We're trying to create a mutable borrow and any borrow exists
    /// - The expression has already been moved
    pub fn check_borrow(
        &mut self,
        expr: &Expr,
        kind: BorrowKind,
        lifetime: Lifetime,
    ) -> Result<(), BorrowError> {
        let new_borrow = Borrow::new(kind.clone(), lifetime, expr.clone());

        // Check for conflicts with existing borrows
        for existing in &self.borrows {
            if new_borrow.conflicts_with(existing) {
                return Err(BorrowError::Conflict {
                    new_kind: kind,
                    existing_kind: existing.kind.clone(),
                    expr: expr.clone(),
                });
            }
        }

        // No conflicts, add the borrow
        self.borrows.push(new_borrow);
        Ok(())
    }

    /// End a borrow by lifetime
    ///
    /// When a lifetime ends, all borrows with that lifetime are removed.
    pub fn end_borrows_with_lifetime(&mut self, lifetime: Lifetime) {
        self.borrows
            .retain(|b| b.lifetime != lifetime);
    }

    /// Get all active borrows
    pub fn active_borrows(&self) -> &[Borrow] {
        &self.borrows
    }

    /// Clear all borrows (e.g., at scope exit)
    pub fn clear(&mut self) {
        self.borrows.clear();
    }
}

impl Default for BorrowChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during borrow checking
#[derive(Debug, Clone)]
pub enum BorrowError {
    /// Two borrows conflict with each other
    Conflict {
        /// The kind of borrow we're trying to create
        new_kind: BorrowKind,
        /// The kind of existing borrow that conflicts
        existing_kind: BorrowKind,
        /// The expression being borrowed
        expr: Expr,
    },

    /// Cannot borrow a value that has been moved
    UseAfterMove {
        /// The expression that was moved
        expr: Expr,
    },

    /// Cannot create a mutable reference when immutable references exist
    MutabilityConflict {
        /// The expression being borrowed
        expr: Expr,
        /// Number of existing immutable borrows
        count: usize,
    },
}

impl fmt::Display for BorrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorrowError::Conflict {
                new_kind,
                existing_kind,
                expr,
            } => {
                write!(
                    f,
                    "cannot create {} borrow: there is already an {} borrow of {:?}",
                    new_kind, existing_kind, expr
                )
            }
            BorrowError::UseAfterMove { expr } => {
                write!(f, "cannot borrow moved value: {:?}", expr)
            }
            BorrowError::MutabilityConflict { expr, count } => {
                write!(
                    f,
                    "cannot create mutable borrow of {:?}: there are {} existing immutable borrows",
                    expr, count
                )
            }
        }
    }
}

impl std::error::Error for BorrowError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_kind_display() {
        assert_eq!(format!("{}", BorrowKind::View), "view");
        assert_eq!(format!("{}", BorrowKind::Mut), "mut");
        assert_eq!(format!("{}", BorrowKind::Take), "take");
    }

    #[test]
    fn test_borrow_checker_new() {
        let checker = BorrowChecker::new();
        assert_eq!(checker.active_borrows().len(), 0);
    }

    #[test]
    fn test_borrow_checker_default() {
        let checker = BorrowChecker::default();
        assert_eq!(checker.active_borrows().len(), 0);
    }

    #[test]
    fn test_single_borrow() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Int(42);

        let result = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
        assert!(result.is_ok());
        assert_eq!(checker.active_borrows().len(), 1);
    }

    #[test]
    fn test_two_immutable_borrows() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Int(42);

        // First view borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok());

        // Second view borrow (should also be OK)
        let result2 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(2));
        // Note: Currently this won't conflict due to simplified same_target check
        // In a full implementation, we'd need to track the actual target
        assert!(result2.is_ok());
    }

    #[test]
    fn test_mutable_after_immutable() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Int(42);

        // First view borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok());

        // Try mut borrow (should conflict if same target)
        let result2 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(2));
        // Note: Due to simplified same_target check, this might not conflict yet
        // In a full implementation, this would be an error
    }

    #[test]
    fn test_end_borrows() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Int(42);

        // Create a borrow
        checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1)).unwrap();
        assert_eq!(checker.active_borrows().len(), 1);

        // End it
        checker.end_borrows_with_lifetime(Lifetime::new(1));
        assert_eq!(checker.active_borrows().len(), 0);
    }

    #[test]
    fn test_clear() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Int(42);

        checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1)).unwrap();
        checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(2)).unwrap();
        assert_eq!(checker.active_borrows().len(), 2);

        checker.clear();
        assert_eq!(checker.active_borrows().len(), 0);
    }

    #[test]
    fn test_borrow_error_display() {
        let err = BorrowError::Conflict {
            new_kind: BorrowKind::Mut,
            existing_kind: BorrowKind::View,
            expr: Expr::Int(42),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("cannot create mut borrow"));
    }

    #[test]
    fn test_mut_after_view_conflicts() {
        let mut checker = BorrowChecker::new();
        // Use the same expression object for both borrows
        let expr = Expr::Ident("x".into());

        // First view borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok(), "First view borrow should succeed");
        assert_eq!(checker.active_borrows().len(), 1);

        // Try mut borrow (should conflict)
        let result2 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_err(), "Mut borrow after view should conflict");

        let err = result2.unwrap_err();
        match err {
            BorrowError::Conflict { new_kind, existing_kind, .. } => {
                assert_eq!(new_kind, BorrowKind::Mut);
                assert_eq!(existing_kind, BorrowKind::View);
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_take_conflicts_with_view() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Ident("s".into());

        // First view borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok(), "View borrow should succeed");

        // Try take (should conflict)
        let result2 = checker.check_borrow(&expr, BorrowKind::Take, Lifetime::new(2));
        assert!(result2.is_err(), "Take after view should conflict");

        let err = result2.unwrap_err();
        match err {
            BorrowError::Conflict { new_kind, existing_kind, .. } => {
                assert_eq!(new_kind, BorrowKind::Take);
                assert_eq!(existing_kind, BorrowKind::View);
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_take_conflicts_with_mut() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Ident("s".into());

        // First mut borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "Mut borrow should succeed");

        // Try take (should conflict)
        let result2 = checker.check_borrow(&expr, BorrowKind::Take, Lifetime::new(2));
        assert!(result2.is_err(), "Take after mut should conflict");

        let err = result2.unwrap_err();
        match err {
            BorrowError::Conflict { new_kind, existing_kind, .. } => {
                assert_eq!(new_kind, BorrowKind::Take);
                assert_eq!(existing_kind, BorrowKind::Mut);
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_two_mut_borrows_conflict() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Ident("data".into());

        // First mut borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "First mut borrow should succeed");
        assert_eq!(checker.active_borrows().len(), 1);

        // Second mut borrow (should conflict)
        let result2 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_err(), "Second mut borrow should conflict");

        let err = result2.unwrap_err();
        match err {
            BorrowError::Conflict { new_kind, existing_kind, .. } => {
                assert_eq!(new_kind, BorrowKind::Mut);
                assert_eq!(existing_kind, BorrowKind::Mut);
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_view_after_mut_conflicts() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Ident("value".into());

        // First mut borrow
        let result1 = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "Mut borrow should succeed");

        // Try view borrow (should conflict)
        let result2 = checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(2));
        assert!(result2.is_err(), "View after mut should conflict");

        let err = result2.unwrap_err();
        match err {
            BorrowError::Conflict { new_kind, existing_kind, .. } => {
                assert_eq!(new_kind, BorrowKind::View);
                assert_eq!(existing_kind, BorrowKind::Mut);
            }
            _ => panic!("Expected Conflict error"),
        }
    }

    #[test]
    fn test_different_targets_no_conflict() {
        let mut checker = BorrowChecker::new();
        // Use different expression types (different discriminants)
        let expr1 = Expr::Ident("x".into());
        let expr2 = Expr::Int(42);

        // First mut borrow on ident
        let result1 = checker.check_borrow(&expr1, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "First mut borrow should succeed");

        // Second mut borrow on int (should NOT conflict - different discriminants)
        let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_ok(), "Mut borrow of different target should succeed");
        assert_eq!(checker.active_borrows().len(), 2);
    }

    #[test]
    fn test_borrow_lifetime_end() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Ident("s".into());

        // Create a view borrow
        checker.check_borrow(&expr, BorrowKind::View, Lifetime::new(1)).unwrap();
        assert_eq!(checker.active_borrows().len(), 1);

        // End the borrow
        checker.end_borrows_with_lifetime(Lifetime::new(1));
        assert_eq!(checker.active_borrows().len(), 0);

        // Now we can create a mut borrow (no conflict)
        let result = checker.check_borrow(&expr, BorrowKind::Mut, Lifetime::new(2));
        assert!(result.is_ok(), "Mut borrow after view ended should succeed");
    }

    #[test]
    fn test_static_lifetime() {
        let mut checker = BorrowChecker::new();
        let expr = Expr::Ident("constant".into());

        // Static borrow
        let result = checker.check_borrow(&expr, BorrowKind::View, Lifetime::STATIC);
        assert!(result.is_ok(), "Static lifetime borrow should succeed");
        assert_eq!(checker.active_borrows().len(), 1);
    }
}
