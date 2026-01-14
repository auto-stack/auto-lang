//! Linear type system for move-only semantics
//!
//! This module provides the foundation for AutoLang's ownership system.
//! Linear types can only be moved, not cloned, and are automatically
//! cleaned up when they go out of scope.

use std::fmt;

/// Marker trait for linear types (move-only)
///
/// Types implementing this trait cannot be implicitly copied
/// and must be explicitly moved when transferred.
pub trait Linear: Sized {
    /// Cleanup function called when the value is dropped
    ///
    /// This is called automatically when a linear value goes out of scope
    /// or is explicitly moved. Implementations should free resources
    /// and mark the value as invalid.
    fn drop_linear(&mut self);
}

/// Tracks whether a value has been moved from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoveState {
    /// Value is available and can be used
    Available,
    /// Value has been moved and is no longer valid
    Moved,
}

impl MoveState {
    /// Check if the value is available
    pub fn is_available(&self) -> bool {
        matches!(self, Self::Available)
    }

    /// Check if the value has been moved
    pub fn is_moved(&self) -> bool {
        matches!(self, Self::Moved)
    }
}

impl fmt::Display for MoveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Available => write!(f, "available"),
            Self::Moved => write!(f, "moved"),
        }
    }
}

/// Type-level move tracking for linear values
///
/// `MoveTracker` wraps a value and tracks whether it has been moved.
/// Attempting to use a moved value will panic.
#[derive(Debug)]
pub struct MoveTracker<T: Linear> {
    value: Option<T>,
    state: MoveState,
}

impl<T: Linear> MoveTracker<T> {
    /// Create a new move tracker wrapping a value
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            state: MoveState::Available,
        }
    }

    /// Take ownership of the value, marking it as moved
    ///
    /// # Panics
    ///
    /// Panics if the value has already been moved (use-after-move)
    pub fn take(&mut self) -> T {
        assert!(
            self.state.is_available(),
            "Use after move: value has already been taken"
        );
        self.state = MoveState::Moved;
        self.value.take().expect("Value should be Some when available")
    }

    /// Get a reference to the value without taking ownership
    ///
    /// This is used for read-only operations that don't consume the value
    pub fn get(&self) -> Option<&T> {
        if self.state.is_available() {
            self.value.as_ref()
        } else {
            None
        }
    }

    /// Check if the value is still available
    pub fn is_available(&self) -> bool {
        self.state.is_available()
    }

    /// Check if the value has been moved
    pub fn is_moved(&self) -> bool {
        self.state.is_moved()
    }

    /// Get the current move state
    pub fn state(&self) -> MoveState {
        self.state
    }
}

impl<T: Linear> Drop for MoveTracker<T> {
    fn drop(&mut self) {
        // Automatically cleanup the value if still available
        if let Some(mut value) = self.value.take() {
            value.drop_linear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test linear type
    struct TestString {
        data: String,
        dropped: bool,
    }

    impl Linear for TestString {
        fn drop_linear(&mut self) {
            self.dropped = true;
        }
    }

    #[test]
    fn test_move_state() {
        let state = MoveState::Available;
        assert!(state.is_available());
        assert!(!state.is_moved());
    }

    #[test]
    fn test_move_tracker() {
        let test_str = TestString {
            data: "hello".to_string(),
            dropped: false,
        };
        let mut tracker = MoveTracker::new(test_str);

        assert!(tracker.is_available());
        assert!(!tracker.is_moved());

        // Take the value
        let _value = tracker.take();

        assert!(!tracker.is_available());
        assert!(tracker.is_moved());
    }

    #[test]
    #[should_panic(expected = "Use after move")]
    fn test_use_after_move_panic() {
        let test_str = TestString {
            data: "hello".to_string(),
            dropped: false,
        };
        let mut tracker = MoveTracker::new(test_str);

        // First take works
        let _value1 = tracker.take();

        // Second take should panic
        let _value2 = tracker.take();
    }

    #[test]
    fn test_automatic_cleanup() {
        let test_str = TestString {
            data: "hello".to_string(),
            dropped: false,
        };

        {
            let _tracker = MoveTracker::new(test_str);
            // tracker goes out of scope here and should cleanup
        } // _tracker dropped here

        // We can't directly test that cleanup happened,
        // but the Drop implementation should have been called
    }
}
