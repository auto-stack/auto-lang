//! Plan 365 W3: libcosmic host backend (VTree → libcosmic Element).
//!
//! This is Host ② (the COSMIC deliverable) from Plan 365 D2. It lowers the
//! platform-neutral `VTree` to libcosmic/iced `Element`s, producing a real
//! Wayland citizen (layer-shell, xdg activation, cosmic-protocols).
//!
//! ## Cross-platform build
//!
//! The real VTree→Element lowering uses `libcosmic` (Linux/Wayland-only). On
//! non-Linux targets, this crate compiles to a stub that delegates to the
//! headless backend (no window, just builds the VTree) — so Windows dev/test
//! builds are unaffected and `run_libcosmic()` has a working (headless) path
//! everywhere. On Linux, `mod linux` provides the real libcosmic lowering.

/// Real libcosmic lowering (Linux/Wayland only).
#[cfg(target_os = "linux")]
pub mod linux;

/// On non-Linux, delegate to the headless backend as a no-op stub.
#[cfg(not(target_os = "linux"))]
pub mod fallback;

/// Run a `Component` under the libcosmic host.
///
/// - **Linux**: lowers `Component::view()` → VTree → libcosmic Elements and
///   runs a real iced/libcosmic application (Wayland citizen).
/// - **Non-Linux (dev)**: delegates to `auto_lang::ui::headless::run_headless`
///   (builds the VTree in memory, no window) — so the call site works on
///   Windows for testing the app logic without a Wayland compositor.
pub fn run_libcosmic<C>() -> auto_lang::ui::AppResult<()>
where
    C: auto_lang::ui::Component + Default + 'static,
    C::Msg: Clone + std::fmt::Debug + Send + 'static,
{
    #[cfg(target_os = "linux")]
    {
        linux::run_libcosmic::<C>()
    }
    #[cfg(not(target_os = "linux"))]
    {
        fallback::run_libcosmic_fallback::<C>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auto_lang::ui::{Component, View};

    /// A minimal component for testing. On Linux this would be lowered to
    /// libcosmic Elements; on Windows it runs via the headless fallback.
    #[derive(Debug, Default)]
    struct TestApp;

    impl Component for TestApp {
        type Msg = u32;
        fn on(&mut self, _msg: Self::Msg) {}
        fn view(&self) -> View<Self::Msg> {
            View::Text {
                content: "libcosmic test".into(),
                style: None,
            }
        }
    }

    #[test]
    fn run_libcosmic_works_on_all_platforms() {
        // On Linux: builds VTree + coverage report (no real window in test mode).
        // On Windows: delegates to headless backend.
        let result = run_libcosmic::<TestApp>();
        assert!(result.is_ok(), "run_libcosmic failed: {:?}", result);
    }
}
