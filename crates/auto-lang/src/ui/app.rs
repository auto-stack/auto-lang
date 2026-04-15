use super::Component;

pub type AppResult<T> = Result<T, Box<dyn std::error::Error>>;

pub struct App;

impl App {
    pub fn run<C>() -> AppResult<()>
    where
        C: Component + Default + 'static,
    {
        #[cfg(feature = "ui-headless")]
        {
            super::headless::run_headless::<C>();
            return Ok(());
        }

        #[cfg(all(feature = "ui-iced", not(feature = "ui-headless")))]
        {
            return Err("Please use the ICED backend crate directly.".into());
        }

        #[cfg(all(feature = "ui-gpui", not(any(feature = "ui-headless", feature = "ui-iced"))))]
        {
            return Err("Please use the GPUI backend crate directly.".into());
        }

        #[cfg(not(any(feature = "ui-headless", feature = "ui-iced", feature = "ui-gpui")))]
        {
            return Err(
                "No backend enabled. Enable one of: 'ui-headless', 'ui-iced', or 'ui-gpui'."
                    .into(),
            );
        }
    }
}
