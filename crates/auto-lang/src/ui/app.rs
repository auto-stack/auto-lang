use super::Component;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct App;

impl App {
    /// Run a `Component` app under the auto-detected default backend.
    ///
    /// Plan 365 W1: delegates to `HostBackend::default_for_features().run()`.
    /// The per-backend direct entry points (`iced::run_app`, `gpui::run_app`,
    /// `headless::run_headless`) remain available for callers that need
    /// backend-specific control.
    pub fn run<C>() -> AppResult<()>
    where
        C: Component + Default + 'static,
        C::Msg: Clone + std::fmt::Debug + Send + 'static,
    {
        super::host::HostBackend::default_for_features()?.run::<C>()
    }
}
