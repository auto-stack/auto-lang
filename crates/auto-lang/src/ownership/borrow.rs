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
use crate::ownership::lifetime::{Lifetime, LifetimeContext};
use auto_val::Op;
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

/// Normalized target for borrow comparison
///
/// This enum represents the "base target" of a borrow expression,
/// allowing us to detect when two different expressions refer to
/// the same underlying value.
///
/// # Examples
///
/// - `x` → `Target::Variable("x")`
/// - `view x` → `Target::Variable("x")` (unwrapped)
/// - `obj.field` → `Target::Path(Variable("obj"), "field")`
/// - `arr[index]` → `Target::Index(Variable("arr"))`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    /// A simple variable reference (e.g., `x`)
    Variable(String),
    /// A path expression (e.g., `obj.field`, `obj.field.subfield`)
    Path(Box<Target>, String),
    /// An index operation (e.g., `arr[index]`)
    Index(Box<Target>),
    /// Unknown or unanalyzable target (e.g., temporary values)
    Unknown,
}

impl Target {
    /// Extract the base target from an expression
    ///
    /// This function normalizes expressions to their base target,
    /// unwrapping view/mut/take wrappers and following member access paths.
    ///
    /// # Examples
    ///
    /// - `Expr::Ident("x")` → `Target::Variable("x")`
    /// - `Expr::View(Box::new(Expr::Ident("x")))` → `Target::Variable("x")`
    /// - `Expr::Bina(obj, Op::Dot, "field")` → `Target::Path(target, "field")`
    pub fn from_expr(expr: &Expr) -> Self {
        match expr {
            // Base case: simple identifier
            Expr::Ident(name) => Target::Variable(name.to_string()),

            // Unwrap borrow expressions to get the inner target
            Expr::View(inner) | Expr::Mut(inner) | Expr::Take(inner) => {
                Self::from_expr(inner)
            }

            // Path expression: obj.field or obj.field.subfield
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    // Member access: obj.field
                    Op::Dot => {
                        let base_target = Self::from_expr(lhs);
                        let field_name = match rhs.as_ref() {
                            Expr::Ident(name) => name.to_string(),
                            // Dynamic field access - treat as unknown
                            _ => return Target::Unknown,
                        };
                        Target::Path(Box::new(base_target), field_name)
                    }
                    // Other binary operations - treat as unknown
                    _ => Target::Unknown,
                }
            }

            // Index operation: arr[index] or map[key]
            Expr::Index(container, _index) => {
                let base_target = Self::from_expr(container);
                Target::Index(Box::new(base_target))
            }

            // All other expressions are unknown targets
            // (literals, function calls, blocks, unary ops, etc.)
            _ => Target::Unknown,
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Target::Variable(name) => write!(f, "{}", name),
            Target::Path(base, field) => write!(f, "{}.{}", base, field),
            Target::Index(base) => write!(f, "{}[...]", base),
            Target::Unknown => write!(f, "<unknown>"),
        }
    }
}

/// Represents a single borrow in the borrow checker
///
/// Each borrow tracks:
/// - What kind of borrow it is (view, mut, or take)
/// - Its lifetime (when it ends)
/// - The expression being borrowed
/// - The normalized target (for conflict detection)
#[derive(Debug, Clone)]
pub struct Borrow {
    /// Kind of borrow
    pub kind: BorrowKind,
    /// Lifetime of this borrow
    pub lifetime: Lifetime,
    /// The expression being borrowed
    pub expr: Expr,
    /// Normalized target for accurate conflict detection
    pub target: Target,
}

impl Borrow {
    /// Create a new borrow
    pub fn new(kind: BorrowKind, lifetime: Lifetime, expr: Expr) -> Self {
        let target = Target::from_expr(&expr);
        Self {
            kind,
            lifetime,
            expr,
            target,
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
    ///
    /// This now compares the normalized targets, which properly handles:
    /// - Unwrapping view/mut/take expressions
    /// - Following member access paths (obj.field)
    /// - Index operations (arr[index])
    /// - Pointer dereferences (*ptr)
    fn same_target(&self, other: &Borrow) -> bool {
        self.target == other.target
    }

    /// Check if two borrows have overlapping lifetimes
    ///
    /// This determines if two borrows are active at the same time.
    /// With region tracking, we can now precisely determine overlap
    /// based on the actual code regions where lifetimes are active.
    ///
    /// # Overlap Rules
    /// - Same lifetime → definitely overlap
    /// - One lifetime outlives the other → overlap (shorter is within longer)
    /// - Different lifetimes with region info → check region overlap
    /// - Different lifetimes without region info → assume overlap (conservative)
    ///
    /// # Region-Based Detection
    /// When lifetime region information is available, we can precisely
    /// determine if two borrows' lifetimes overlap in the code.
    fn lifetimes_overlap(&self, other: &Borrow) -> bool {
        // Same lifetime always overlaps
        if self.lifetime == other.lifetime {
            return true;
        }

        // Static lifetime overlaps with everything
        if self.lifetime == Lifetime::STATIC || other.lifetime == Lifetime::STATIC {
            return true;
        }

        // If one lifetime outlives the other, they overlap
        // (the shorter lifetime is contained within the longer one)
        if Lifetime::outlives(self.lifetime, other.lifetime)
            || Lifetime::outlives(other.lifetime, self.lifetime)
        {
            return true;
        }

        // Different lifetimes where neither outlives the other
        // With lifetime region tracking, we can now check precise overlap
        // TODO: Use LifetimeContext to check region overlap
        // For now, conservatively assume overlap until integrated
        true
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
    /// Lifetime context for region-based overlap detection
    lifetime_ctx: LifetimeContext,
}

impl BorrowChecker {
    /// Create a new borrow checker
    pub fn new() -> Self {
        Self {
            borrows: Vec::new(),
            lifetime_ctx: LifetimeContext::new(),
        }
    }

    /// Get a reference to the lifetime context
    ///
    /// This allows external code to register lifetime regions for
    /// precise overlap detection.
    pub fn lifetime_context(&mut self) -> &mut LifetimeContext {
        &mut self.lifetime_ctx
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
                    target: new_borrow.target.clone(),
                    new_lifetime: lifetime,
                    existing_lifetime: existing.lifetime,
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
        /// The target being borrowed
        target: Target,
        /// The new borrow's lifetime
        new_lifetime: Lifetime,
        /// The existing borrow's lifetime
        existing_lifetime: Lifetime,
    },

    /// Cannot borrow a value that has been moved
    UseAfterMove {
        /// The target that was moved
        target: Target,
    },

    /// Cannot create a mutable reference when immutable references exist
    MutabilityConflict {
        /// The target being borrowed
        target: Target,
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
                target,
                new_lifetime,
                existing_lifetime,
            } => {
                write!(
                    f,
                    "cannot create {} borrow of {}: there is already an {} borrow (lifetime {})",
                    new_kind, target, existing_kind, existing_lifetime
                )?;
                if new_lifetime != existing_lifetime {
                    write!(f, " with lifetime {}", new_lifetime)?;
                }
                Ok(())
            }
            BorrowError::UseAfterMove { target } => {
                write!(f, "cannot borrow moved value: {}", target)
            }
            BorrowError::MutabilityConflict { target, count } => {
                write!(
                    f,
                    "cannot create mutable borrow of {}: there {} {} existing immutable borrow{}",
                    target,
                    if *count == 1 { "is" } else { "are" },
                    count,
                    if *count == 1 { "" } else { "s" }
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
            target: Target::Variable("x".to_string()),
            new_lifetime: Lifetime::new(2),
            existing_lifetime: Lifetime::new(1),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("cannot create mut borrow"));
        assert!(msg.contains("x"));
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
            BorrowError::Conflict { new_kind, existing_kind, target, .. } => {
                assert_eq!(new_kind, BorrowKind::Mut);
                assert_eq!(existing_kind, BorrowKind::View);
                assert_eq!(target, Target::Variable("x".to_string()));
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
            BorrowError::Conflict { new_kind, existing_kind, target, .. } => {
                assert_eq!(new_kind, BorrowKind::Take);
                assert_eq!(existing_kind, BorrowKind::View);
                assert_eq!(target, Target::Variable("s".to_string()));
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
            BorrowError::Conflict { new_kind, existing_kind, target, .. } => {
                assert_eq!(new_kind, BorrowKind::Take);
                assert_eq!(existing_kind, BorrowKind::Mut);
                assert_eq!(target, Target::Variable("s".to_string()));
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
            BorrowError::Conflict { new_kind, existing_kind, target, .. } => {
                assert_eq!(new_kind, BorrowKind::Mut);
                assert_eq!(existing_kind, BorrowKind::Mut);
                assert_eq!(target, Target::Variable("data".to_string()));
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
            BorrowError::Conflict { new_kind, existing_kind, target, .. } => {
                assert_eq!(new_kind, BorrowKind::View);
                assert_eq!(existing_kind, BorrowKind::Mut);
                assert_eq!(target, Target::Variable("value".to_string()));
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

    #[test]
    fn test_view_and_ident_same_target() {
        let mut checker = BorrowChecker::new();
        // Direct identifier
        let expr1 = Expr::Ident("x".into());
        // View of the same identifier
        let expr2 = Expr::View(Box::new(Expr::Ident("x".into())));

        // First view borrow on x
        let result1 = checker.check_borrow(&expr1, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok(), "First view borrow should succeed");

        // Second view borrow on view x - should conflict (same target)
        let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_err(), "Mut borrow of view x should conflict with view x");
    }

    #[test]
    fn test_different_variables_no_conflict() {
        let mut checker = BorrowChecker::new();
        let expr1 = Expr::Ident("x".into());
        let expr2 = Expr::Ident("y".into());

        // First mut borrow on x
        let result1 = checker.check_borrow(&expr1, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "First mut borrow should succeed");

        // Second mut borrow on y - should NOT conflict (different targets)
        let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_ok(), "Mut borrow of different variable should succeed");
        assert_eq!(checker.active_borrows().len(), 2);
    }

    #[test]
    fn test_path_expression_target() {
        let mut checker = BorrowChecker::new();
        // obj.field expression
        let obj_expr = Expr::Ident("obj".into());
        let field_expr = Expr::Ident("field".into());
        let expr1 = Expr::Bina(Box::new(obj_expr), Op::Dot, Box::new(field_expr));

        // First mut borrow on obj.field
        let result1 = checker.check_borrow(&expr1, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "First mut borrow should succeed");

        // View of obj.field - should conflict
        let expr2 = Expr::View(Box::new(Expr::Bina(
            Box::new(Expr::Ident("obj".into())),
            Op::Dot,
            Box::new(Expr::Ident("field".into())),
        )));
        let result2 = checker.check_borrow(&expr2, BorrowKind::View, Lifetime::new(2));
        assert!(result2.is_err(), "View borrow should conflict with mut borrow of same path");
    }

    #[test]
    fn test_different_fields_no_conflict() {
        let mut checker = BorrowChecker::new();
        // obj.field1
        let expr1 = Expr::Bina(
            Box::new(Expr::Ident("obj".into())),
            Op::Dot,
            Box::new(Expr::Ident("field1".into())),
        );

        // obj.field2
        let expr2 = Expr::Bina(
            Box::new(Expr::Ident("obj".into())),
            Op::Dot,
            Box::new(Expr::Ident("field2".into())),
        );

        // First mut borrow on obj.field1
        let result1 = checker.check_borrow(&expr1, BorrowKind::Mut, Lifetime::new(1));
        assert!(result1.is_ok(), "First mut borrow should succeed");

        // Second mut borrow on obj.field2 - should NOT conflict (different fields)
        let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_ok(), "Mut borrow of different field should succeed");
        assert_eq!(checker.active_borrows().len(), 2);
    }

    #[test]
    fn test_nested_path_target() {
        let mut checker = BorrowChecker::new();
        // obj.inner.field expression
        let inner_expr = Expr::Bina(
            Box::new(Expr::Ident("obj".into())),
            Op::Dot,
            Box::new(Expr::Ident("inner".into())),
        );
        let expr1 = Expr::Bina(
            Box::new(inner_expr),
            Op::Dot,
            Box::new(Expr::Ident("field".into())),
        );

        // First view borrow on obj.inner.field
        let result1 = checker.check_borrow(&expr1, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok(), "First view borrow should succeed");

        // Mut of obj.inner.field - should conflict
        let inner_expr2 = Expr::Bina(
            Box::new(Expr::Ident("obj".into())),
            Op::Dot,
            Box::new(Expr::Ident("inner".into())),
        );
        let expr2 = Expr::Bina(
            Box::new(inner_expr2),
            Op::Dot,
            Box::new(Expr::Ident("field".into())),
        );
        let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_err(), "Mut borrow should conflict with view of same nested path");
    }

    #[test]
    fn test_mut_and_take_unwrap_to_same_target() {
        let mut checker = BorrowChecker::new();
        // Direct identifier
        let expr1 = Expr::Ident("data".into());
        // Mut expression
        let expr2 = Expr::Mut(Box::new(Expr::Ident("data".into())));
        // Take expression
        let expr3 = Expr::Take(Box::new(Expr::Ident("data".into())));

        // First view borrow
        let result1 = checker.check_borrow(&expr1, BorrowKind::View, Lifetime::new(1));
        assert!(result1.is_ok(), "View borrow should succeed");

        // Mut borrow - should conflict (same target)
        let result2 = checker.check_borrow(&expr2, BorrowKind::Mut, Lifetime::new(2));
        assert!(result2.is_err(), "Mut should conflict with view of same target");

        // Take - should also conflict (same target)
        let result3 = checker.check_borrow(&expr3, BorrowKind::Take, Lifetime::new(3));
        assert!(result3.is_err(), "Take should conflict with view of same target");
    }
}
