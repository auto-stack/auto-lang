// Core widget abstraction
//
// NOTE: This module is kept for backward compatibility but may be deprecated.
// The new design uses the Component trait from component.rs instead.
// Use Component trait for new code.

use super::view::View;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Legacy trait for UI widgets - superseded by Component trait
///
/// # Deprecated
/// Consider using `Component` trait instead for new code.
pub trait Widget: Sized {
    type Message: Clone + Debug + 'static;
    type Props;

    /// Create a view for this widget
    fn view(&self) -> View<Self::Message>;

    /// Handle messages
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }
}

/// Command type for side effects (kept for potential future use)
pub enum Command<M> {
    None,
    Batch(Vec<Box<Command<M>>>),
    // More to be added
    _Marker(PhantomData<M>),
}

impl<M> Command<M> {
    pub fn none() -> Self {
        Command::None
    }

    pub fn batch(commands: Vec<Command<M>>) -> Self {
        Command::Batch(commands.into_iter().map(Box::new).collect())
    }
}
