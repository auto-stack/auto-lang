//! Headless renderer - renders Component/View/VTree pipeline without a display
//!
//! The [`HeadlessRenderer`] exercises the full view-to-vtree conversion
//! without creating any windows or binding to a GPU. It is useful for:
//!
//! - Fast iteration on UI logic during development
//! - Testing transpiler output
//! - Benchmarking the Component → VTree pipeline
//!
//! # Example
//!
//! ```ignore
//! use auto_lang::ui::headless::HeadlessRenderer;
//! use auto_lang::ui::Component;
//!
//! let mut renderer = HeadlessRenderer::new();
//! let vtree = renderer.render::<MyComponent>();
//! assert_eq!(renderer.render_count(), 1);
//! ```

use std::cell::Cell;

use super::super::component::Component;
use super::super::vnode::VTree;
use super::super::vnode_converter::view_to_vtree;

/// A headless renderer that runs the full Component/View/VTree pipeline
/// without any window creation, GPU access, or event loop.
pub struct HeadlessRenderer {
    render_count: Cell<usize>,
}

impl HeadlessRenderer {
    /// Create a new headless renderer with a render count of zero.
    pub fn new() -> Self {
        Self {
            render_count: Cell::new(0),
        }
    }

    /// Build a default instance of `C`, call `view()`, convert the result
    /// to a [`VTree`], increment the render count, and return the tree.
    pub fn render<C>(&self) -> VTree
    where
        C: Component + Default,
    {
        let component = C::default();
        self.render_with(&component)
    }

    /// Call `view()` on an existing component instance, convert the result
    /// to a [`VTree`], increment the render count, and return the tree.
    pub fn render_with<C>(&self, component: &C) -> VTree
    where
        C: Component,
    {
        let view = component.view();
        let vtree = view_to_vtree(view);
        self.render_count.set(self.render_count.get() + 1);
        vtree
    }

    /// Return the total number of renders performed by this renderer.
    pub fn render_count(&self) -> usize {
        self.render_count.get()
    }
}

impl Default for HeadlessRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience entry point: create a headless renderer, render the default
/// instance of `C`, and drop the resulting VTree.
///
/// This is the function called by [`App::run`](super::super::app::App::run)
/// when the `ui-headless` feature is enabled.
pub fn run_headless<C>()
where
    C: Component + Default,
{
    let renderer = HeadlessRenderer::new();
    let _vtree = renderer.render::<C>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::view::View;

    #[derive(Debug, Default)]
    struct HelloComponent;

    #[derive(Clone, Debug)]
    enum Msg {
        Click,
    }

    impl Component for HelloComponent {
        type Msg = Msg;

        fn on(&mut self, _msg: Self::Msg) {}

        fn view(&self) -> View<Self::Msg> {
            View::text("Hello, headless!")
        }
    }

    #[test]
    fn test_render_increments_count() {
        let renderer = HeadlessRenderer::new();
        assert_eq!(renderer.render_count(), 0);

        let _tree = renderer.render::<HelloComponent>();
        assert_eq!(renderer.render_count(), 1);

        let _tree = renderer.render::<HelloComponent>();
        assert_eq!(renderer.render_count(), 2);
    }

    #[test]
    fn test_render_with_existing_instance() {
        let renderer = HeadlessRenderer::new();
        let component = HelloComponent;

        let tree = renderer.render_with(&component);
        assert_eq!(renderer.render_count(), 1);

        // The tree should have a single text node
        assert_eq!(tree.node_count(), 1);
        let root = tree.root().unwrap();
        assert!(format!("{}", root.kind).contains("Text"));
    }

    #[test]
    fn test_default_impl() {
        let renderer = HeadlessRenderer::default();
        assert_eq!(renderer.render_count(), 0);
    }

    #[test]
    fn test_run_headless_does_not_panic() {
        run_headless::<HelloComponent>();
    }
}
