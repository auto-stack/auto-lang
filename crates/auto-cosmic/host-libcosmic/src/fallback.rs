//! Non-Linux fallback: delegate to the headless backend.
//!
//! On Windows/dev machines without Wayland, `run_libcosmic()` builds the
//! VTree via the headless backend (no window). This lets the same call site
//! work for testing app logic. The real libcosmic lowering lives in
//! [`crate::linux`] (compiled only on `target_os = "linux"`).

pub fn run_libcosmic_fallback<C>() -> auto_ui_runtime::ui::AppResult<()>
where
    C: auto_ui_runtime::ui::Component + Default + 'static,
    C::Msg: Clone + std::fmt::Debug + Send + 'static,
{
    auto_ui_runtime::ui::headless::run_headless::<C>();
    Ok(())
}
