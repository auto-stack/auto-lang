//! Lifetime inference and tracking for borrow checker
//!
//! This module provides the lifetime system used by the borrow checker to track
//! the validity of references and ensure memory safety.

use std::collections::HashMap;

/// A lifetime represents a region of code where a reference is valid
///
/// Lifetimes are used by the borrow checker to ensure that references
/// cannot outlive the data they reference.
///
/// # Example
/// ```ignore
/// let s = "hello";        // Lifetime 'a
/// let t = take s;         // t has lifetime 'b, where 'b: 'a (shorter than or equal to 'a)
/// use(t);                 // 'b ends here
/// // s is still valid
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lifetime(pub u32);

impl Lifetime {
    /// The static lifetime - valid for the entire program
    pub const STATIC: Lifetime = Lifetime(0);

    /// Create a new non-static lifetime
    pub fn new(id: u32) -> Self {
        Lifetime(id)
    }

    /// Get the underlying ID
    pub fn id(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for Lifetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if *self == Self::STATIC {
            write!(f, "'static")
        } else {
            write!(f, "'{}", self.0)
        }
    }
}

impl Lifetime {
    /// Check if one lifetime outlives another
    ///
    /// Returns `true` if `a` lives at least as long as `b`.
    /// The static lifetime outlives all other lifetimes.
    ///
    /// # Example
    /// ```
    /// # use auto_lang::ownership::lifetime::Lifetime;
    /// let static_lt = Lifetime::STATIC;
    /// let lt1 = Lifetime::new(1);
    /// let lt2 = Lifetime::new(2);
    ///
    /// assert!(Lifetime::outlives(static_lt, lt1));
    /// assert!(Lifetime::outlives(lt1, lt1)); // Equal lifetimes outlive each other
    /// ```
    pub fn outlives(a: Lifetime, b: Lifetime) -> bool {
        if a == Lifetime::STATIC {
            true
        } else if b == Lifetime::STATIC {
            false
        } else {
            // Lower ID = longer lifetime, so a outlives b if a.0 <= b.0
            a.0 <= b.0
        }
    }

    /// Create a new lifetime that is the intersection of two lifetimes
    ///
    /// The intersection is the shorter of the two lifetimes.
    /// This is useful when a value is constrained by multiple lifetimes.
    ///
    /// # Example
    /// ```
    /// # use auto_lang::ownership::lifetime::Lifetime;
    /// let lt1 = Lifetime::new(1);
    /// let lt2 = Lifetime::new(2);
    ///
    /// // The shorter lifetime (higher ID = shorter in our convention)
    /// let intersection = Lifetime::intersect(lt1, lt2);
    /// assert_eq!(intersection, lt2); // Higher ID = shorter
    /// ```
    pub fn intersect(a: Lifetime, b: Lifetime) -> Lifetime {
        if a == Lifetime::STATIC {
            b
        } else if b == Lifetime::STATIC {
            a
        } else {
            // Intersection is the shorter lifetime (higher ID)
            Lifetime(a.0.max(b.0))
        }
    }
}

/// Context for managing lifetimes during borrow checking
///
/// The `LifetimeContext` tracks which lifetime each expression has and
/// can generate fresh lifetimes as needed.
///
/// # Example
/// ```
/// # use auto_lang::ownership::lifetime::{Lifetime, LifetimeContext};
/// let mut ctx = LifetimeContext::new();
///
/// // Create a new lifetime
/// let lt1 = ctx.fresh_lifetime();
///
/// // Assign lifetime to an expression (using expression index)
/// ctx.assign_lifetime(1, lt1);
///
/// // Lookup the lifetime of an expression
/// if let Some(&lt) = ctx.get_lifetime(1) {
///     println!("Expression has lifetime: {}", lt);
/// }
/// ```
pub struct LifetimeContext {
    /// Counter for generating unique lifetimes
    counter: u32,
    /// Maps expression IDs to their assigned lifetimes
    regions: HashMap<usize, Lifetime>,
}

impl LifetimeContext {
    /// Create a new lifetime context
    pub fn new() -> Self {
        Self {
            counter: 1, // Start at 1, 0 is reserved for 'static
            regions: HashMap::new(),
        }
    }

    /// Generate a fresh lifetime
    ///
    /// Each call to this function returns a unique lifetime that hasn't
    /// been used before in this context.
    pub fn fresh_lifetime(&mut self) -> Lifetime {
        let lt = Lifetime(self.counter);
        self.counter += 1;
        lt
    }

    /// Assign a lifetime to an expression
    ///
    /// This records that the given expression has the specified lifetime.
    pub fn assign_lifetime(&mut self, expr_id: usize, lt: Lifetime) {
        self.regions.insert(expr_id, lt);
    }

    /// Get the lifetime assigned to an expression
    ///
    /// Returns `None` if no lifetime has been assigned to this expression.
    pub fn get_lifetime(&self, expr_id: usize) -> Option<&Lifetime> {
        self.regions.get(&expr_id)
    }
}

impl Default for LifetimeContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifetime_display() {
        assert_eq!(format!("{}", Lifetime::STATIC), "'static");
        assert_eq!(format!("{}", Lifetime::new(1)), "'1");
        assert_eq!(format!("{}", Lifetime::new(42)), "'42");
    }

    #[test]
    fn test_lifetime_context() {
        let mut ctx = LifetimeContext::new();

        // Generate fresh lifetimes
        let lt1 = ctx.fresh_lifetime();
        assert_eq!(lt1, Lifetime::new(1));

        let lt2 = ctx.fresh_lifetime();
        assert_eq!(lt2, Lifetime::new(2));

        // Assign and lookup lifetimes
        let expr_id = 100;
        ctx.assign_lifetime(expr_id, lt1);

        assert_eq!(ctx.get_lifetime(expr_id), Some(&lt1));
        assert_eq!(ctx.get_lifetime(999), None);
    }

    #[test]
    fn test_outlives() {
        let static_lt = Lifetime::STATIC;
        let lt1 = Lifetime::new(1);
        let lt2 = Lifetime::new(2);

        // Static outlives everything
        assert!(Lifetime::outlives(static_lt, lt1));
        assert!(Lifetime::outlives(static_lt, lt2));

        // Equal lifetimes outlive each other
        assert!(Lifetime::outlives(lt1, lt1));

        // Higher ID = shorter lifetime (outlives less)
        assert!(Lifetime::outlives(lt1, lt2));
        assert!(!Lifetime::outlives(lt2, lt1));
    }

    #[test]
    fn test_intersect() {
        let lt1 = Lifetime::new(1);
        let lt2 = Lifetime::new(2);
        let lt3 = Lifetime::new(3);
        let static_lt = Lifetime::STATIC;

        // Intersection with static
        assert_eq!(Lifetime::intersect(static_lt, lt1), lt1);
        assert_eq!(Lifetime::intersect(lt1, static_lt), lt1);

        // Intersection of two lifetimes (higher ID = shorter)
        assert_eq!(Lifetime::intersect(lt1, lt2), lt2);
        assert_eq!(Lifetime::intersect(lt2, lt3), lt3);
        assert_eq!(Lifetime::intersect(lt1, lt3), lt3);
    }
}
