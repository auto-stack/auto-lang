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

/// A region in code where a lifetime is active
///
/// Lifetimes represent scopes where references are valid. This structure
/// tracks the start and end points of each lifetime region for precise
/// overlap detection.
///
/// # Example
/// ```
/// # use auto_lang::ownership::lifetime::{Lifetime, LifetimeRegion};
/// // A lifetime that starts at line 5, column 10 and ends at line 10, column 5
/// let region = LifetimeRegion {
///     lifetime: Lifetime::new(1),
///     start: (5, 10),  // (line, column)
///     end: (10, 5),
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LifetimeRegion {
    /// The lifetime this region belongs to
    pub lifetime: Lifetime,
    /// Start point of the region (line, column)
    pub start: (usize, usize),
    /// End point of the region (line, column)
    pub end: (usize, usize),
}

impl LifetimeRegion {
    /// Create a new lifetime region
    pub fn new(lifetime: Lifetime, start: (usize, usize), end: (usize, usize)) -> Self {
        Self {
            lifetime,
            start,
            end,
        }
    }

    /// Check if this region overlaps with another region
    ///
    /// Two regions overlap if they share any common point in the code.
    ///
    /// # Example
    /// ```
    /// # use auto_lang::ownership::lifetime::{Lifetime, LifetimeRegion};
    /// let region1 = LifetimeRegion::new(Lifetime::new(1), (1, 0), (10, 0));
    /// let region2 = LifetimeRegion::new(Lifetime::new(2), (5, 0), (15, 0));
    ///
    /// // These regions overlap (lines 5-10 are common)
    /// assert!(region1.overlaps(&region2));
    ///
    /// let region3 = LifetimeRegion::new(Lifetime::new(3), (20, 0), (30, 0));
    ///
    /// // These regions don't overlap
    /// assert!(!region1.overlaps(&region3));
    /// ```
    pub fn overlaps(&self, other: &LifetimeRegion) -> bool {
        // Check for overlap in line numbers
        // Two regions overlap if:
        // - self.start <= other.end AND self.end >= other.start

        let self_start = self.start.0 * 1000 + self.start.1; // Convert line:col to comparable number
        let self_end = self.end.0 * 1000 + self.end.1;
        let other_start = other.start.0 * 1000 + other.start.1;
        let other_end = other.end.0 * 1000 + other.end.1;

        // Regions overlap if one starts before the other ends
        self_start <= other_end && self_end >= other_start
    }

    /// Check if a point (line, column) is within this region
    pub fn contains(&self, line: usize, col: usize) -> bool {
        let point = line * 1000 + col;
        let start = self.start.0 * 1000 + self.start.1;
        let end = self.end.0 * 1000 + self.end.1;

        point >= start && point <= end
    }
}

/// Context for managing lifetimes during borrow checking
///
/// The `LifetimeContext` tracks which lifetime each expression has and
/// can generate fresh lifetimes as needed. It now also tracks the regions
/// where each lifetime is active.
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
/// // Define the region where this lifetime is active
/// ctx.set_region(lt1, (5, 0), (10, 0));
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
    /// Maps lifetimes to their active regions
    lifetime_regions: HashMap<Lifetime, LifetimeRegion>,
}

impl LifetimeContext {
    /// Create a new lifetime context
    pub fn new() -> Self {
        Self {
            counter: 1, // Start at 1, 0 is reserved for 'static
            regions: HashMap::new(),
            lifetime_regions: HashMap::new(),
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

    /// Define the region where a lifetime is active
    ///
    /// This tracks the start and end points of a lifetime for precise
    /// overlap detection.
    ///
    /// # Arguments
    /// - `lt`: The lifetime to define a region for
    /// - `start`: (line, column) where the lifetime begins
    /// - `end`: (line, column) where the lifetime ends
    pub fn set_region(&mut self, lt: Lifetime, start: (usize, usize), end: (usize, usize)) {
        let region = LifetimeRegion::new(lt, start, end);
        self.lifetime_regions.insert(lt, region);
    }

    /// Get the lifetime assigned to an expression
    ///
    /// Returns `None` if no lifetime has been assigned to this expression.
    pub fn get_lifetime(&self, expr_id: usize) -> Option<&Lifetime> {
        self.regions.get(&expr_id)
    }

    /// Get the region for a lifetime
    ///
    /// Returns `None` if no region has been defined for this lifetime.
    pub fn get_region(&self, lt: Lifetime) -> Option<&LifetimeRegion> {
        self.lifetime_regions.get(&lt)
    }

    /// Check if two lifetimes have overlapping regions
    ///
    /// This provides precise overlap detection using region information.
    /// If region information is not available for either lifetime, falls
    /// back to conservative behavior (assumes overlap).
    ///
    /// # Returns
    /// - `true` if the lifetimes overlap
    /// - `false` if the lifetimes are definitely non-overlapping
    pub fn regions_overlap(&self, lt1: Lifetime, lt2: Lifetime) -> bool {
        // Same lifetime always overlaps
        if lt1 == lt2 {
            return true;
        }

        // Static lifetime overlaps with everything
        if lt1 == Lifetime::STATIC || lt2 == Lifetime::STATIC {
            return true;
        }

        // Try to get region information
        match (self.get_region(lt1), self.get_region(lt2)) {
            (Some(region1), Some(region2)) => {
                // Both regions have info - check precise overlap
                region1.overlaps(region2)
            }
            (_, _) => {
                // At least one region missing - fall back to conservative check
                // If one lifetime outlives the other, they overlap
                if Lifetime::outlives(lt1, lt2) || Lifetime::outlives(lt2, lt1) {
                    true
                } else {
                    // Different lifetimes where neither outlives - conservatively assume overlap
                    true
                }
            }
        }
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

    #[test]
    fn test_lifetime_region() {
        let lt1 = Lifetime::new(1);
        let lt2 = Lifetime::new(2);
        let lt3 = Lifetime::new(3);

        // Test overlapping regions
        let region1 = LifetimeRegion::new(lt1, (1, 0), (10, 0));
        let region2 = LifetimeRegion::new(lt2, (5, 0), (15, 0));
        let region3 = LifetimeRegion::new(lt3, (20, 0), (30, 0));

        // region1 and region2 overlap (lines 5-10 are common)
        assert!(region1.overlaps(&region2));
        assert!(region2.overlaps(&region1));

        // region1 and region3 don't overlap
        assert!(!region1.overlaps(&region3));
        assert!(!region3.overlaps(&region1));

        // Test contains method
        assert!(region1.contains(5, 0)); // Inside region1
        assert!(region1.contains(1, 0)); // At start
        assert!(region1.contains(10, 0)); // At end
        assert!(!region1.contains(11, 0)); // Outside
    }

    #[test]
    fn test_lifetime_context_regions() {
        let mut ctx = LifetimeContext::new();

        let lt1 = Lifetime::new(1);
        let lt2 = Lifetime::new(2);

        // Set regions
        ctx.set_region(lt1, (1, 0), (10, 0));
        ctx.set_region(lt2, (5, 0), (15, 0));

        // Check overlap detection
        assert!(ctx.regions_overlap(lt1, lt2)); // Overlapping regions
        assert!(ctx.regions_overlap(lt2, lt1));

        // Add non-overlapping region
        let lt3 = Lifetime::new(3);
        ctx.set_region(lt3, (20, 0), (30, 0));

        assert!(!ctx.regions_overlap(lt1, lt3)); // Non-overlapping
        assert!(!ctx.regions_overlap(lt3, lt1));
    }

    #[test]
    fn test_lifetime_context_fallback() {
        let mut ctx = LifetimeContext::new();

        let lt1 = Lifetime::new(1);
        let lt2 = Lifetime::new(2);

        // No regions set - should fall back to conservative behavior
        assert!(ctx.regions_overlap(lt1, lt2));

        // Set region for only one lifetime
        ctx.set_region(lt1, (1, 0), (10, 0));
        assert!(ctx.regions_overlap(lt1, lt2)); // Still conservative
    }
}
