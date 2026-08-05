//! Plan 365 W1: Unified host backend interface.
//!
//! Each render backend (headless / iced / gpui) sits behind one `HostBackend`
//! entry. This is the seam that W2 (dev-host mock framework) and W3 (libcosmic
//! host) build on.
//!
//! Design notes:
//! - A `dyn HostBackend` trait object is infeasible because the per-backend
//!   bounds differ (iced requires `C::Msg: Send`) and the backend element types
//!   are foreign. Instead, `HostBackend` is an enum whose variants are each
//!   `#[cfg(feature = "ui-*")]`-gated, with a single `run::<C>()` method that
//!   delegates to the existing per-backend entry functions.
//! - The per-backend direct entry points (`iced::run_app`, `gpui::run_app`,
//!   `headless::run_headless`) remain public and unchanged — examples that
//!   call them directly still work. `HostBackend` is the additive unified path.

use super::Component;
use super::app::AppResult;

/// Identifies which host backend to run an app under.
///
/// Each variant is gated on its `ui-*` Cargo feature, so the enum compiles
/// (with at least one variant) whenever any UI backend feature is enabled.
#[derive(Debug, Clone)]
pub enum HostBackend {
    /// In-memory VTree builder, no window — used for tests and equivalence
    /// checks (Plan 174).
    #[cfg(feature = "ui-headless")]
    Headless,

    /// Iced render backend (cross-platform GUI).
    #[cfg(feature = "ui-iced")]
    Iced,

    /// GPUI render backend (Zed's GPU UI layer).
    #[cfg(feature = "ui-gpui")]
    Gpui {
        /// Window title (GPUI requires one at open-window time).
        title: String,
    },
}

impl HostBackend {
    /// Run a `Component` app under this backend.
    ///
    /// The iced backend additionally requires `C::Msg: Send` (enforced by
    /// `iced::run_app`'s own where-clause); headless and gpui do not.
    ///
    /// **Note**: the `Iced` variant can only be used via [`run`](Self::run) when
    /// `C::Msg: Send`. If your message type is not `Send`, use the `Headless`
    /// or `Gpui` variant instead, or call `iced::run_app` directly (which has
    /// the `Send` bound in its own signature and gives a clear error at that
    /// call site).
    pub fn run<C>(self) -> AppResult<()>
    where
        C: Component + Default + 'static,
        C::Msg: Clone + std::fmt::Debug + Send + 'static,
    {
        match self {
            #[cfg(feature = "ui-headless")]
            HostBackend::Headless => {
                super::headless::run_headless::<C>();
                Ok(())
            }
            #[cfg(feature = "ui-iced")]
            HostBackend::Iced => {
                super::iced::run_app::<C>()
            }
            #[cfg(feature = "ui-gpui")]
            HostBackend::Gpui { title } => {
                super::gpui::run_app::<C>(&title)
            }
        }
    }

    /// Pick the best-available backend given the enabled Cargo features.
    ///
    /// Priority: Headless > Iced > Gpui (matches the historical `App::run`
    /// cfg-ladder, where headless short-circuits first). Returns `Err` if no
    /// `ui-*` feature is enabled.
    pub fn default_for_features() -> AppResult<HostBackend> {
        #[cfg(feature = "ui-headless")]
        {
            return Ok(HostBackend::Headless);
        }
        #[cfg(all(feature = "ui-iced", not(feature = "ui-headless")))]
        {
            return Ok(HostBackend::Iced);
        }
        #[cfg(all(
            feature = "ui-gpui",
            not(any(feature = "ui-headless", feature = "ui-iced"))
        ))]
        {
            return Ok(HostBackend::Gpui {
                title: "Auto App".to_string(),
            });
        }
        #[cfg(not(any(feature = "ui-headless", feature = "ui-iced", feature = "ui-gpui")))]
        {
            return Err(
                "No UI backend enabled. Enable one of: 'ui-headless', 'ui-iced', 'ui-gpui'."
                    .into(),
            );
        }

        // Unreachable, but Rust needs a fallback when the cfg-gated returns
        // above are all compiled out (shouldn't happen — the last cfg is the
        // catch-all `not(any(...))`).
        #[allow(unreachable_code)]
        Err("unreachable: backend selection logic gap".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "ui-headless")]
    #[test]
    fn test_default_picks_headless_when_enabled() {
        // When ui-headless is on, it always wins (highest priority).
        let backend = HostBackend::default_for_features().unwrap();
        assert!(matches!(backend, HostBackend::Headless));
    }

    #[cfg(feature = "ui-headless")]
    #[test]
    fn test_headless_runs_simple_component() {
        // A minimal component that the headless backend can render.
        #[derive(Debug, Default)]
        struct Hello;
        impl Component for Hello {
            type Msg = u32; // trivial message type
            fn on(&mut self, _msg: Self::Msg) {}
            fn view(&self) -> super::super::View<Self::Msg> {
                use super::super::View;
                View::text("hello")
            }
        }
        let result = HostBackend::Headless.run::<Hello>();
        assert!(result.is_ok());
    }
}
