//! Invocation settings belong to the installing domain, never to the compiler.
use std::{
    any::Any,
    cell::{Cell, RefCell},
    future::Future,
    sync::Arc,
};

/// Immutable, domain-owned launch input. Its concrete type is shared by the
/// host and caller; Core neither interprets nor duplicates deployment policy.
#[derive(Clone)]
pub struct HttpServiceLaunch(Arc<dyn Any + Send + Sync>);

impl HttpServiceLaunch {
    pub fn new<T: Any + Send + Sync>(settings: Arc<T>) -> Self {
        Self(settings)
    }

    pub fn settings<T: Any + Send + Sync>(&self) -> Option<Arc<T>> {
        self.0.clone().downcast().ok()
    }
}

impl std::fmt::Debug for HttpServiceLaunch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HttpServiceLaunch(domain-owned)")
    }
}

thread_local! {
    static BOUND: Cell<bool> = const { Cell::new(false) };
    static CURRENT: RefCell<Option<HttpServiceLaunch>> = const { RefCell::new(None) };
}

pub(crate) fn launch_context_is_bound() -> bool {
    BOUND.with(Cell::get)
}

pub fn current_http_service_launch() -> Option<HttpServiceLaunch> {
    CURRENT.with(|slot| slot.borrow().clone())
}

/// A sync SDK invocation copies this context before crossing its worker bridge.
pub fn with_http_service_launch<R>(
    launch: Option<HttpServiceLaunch>,
    run: impl FnOnce() -> R,
) -> R {
    struct Restore(Option<HttpServiceLaunch>, bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            CURRENT.with(|slot| *slot.borrow_mut() = self.0.take());
            BOUND.with(|slot| slot.set(self.1));
        }
    }
    let _restore = Restore(
        CURRENT.with(|slot| slot.replace(launch)),
        BOUND.with(|slot| slot.replace(true)),
    );
    run()
}

/// Restore on every poll, including Pending/error/unwind. Interleaved local
/// invocations cannot overwrite one another's deployment input.
pub async fn with_http_service_launch_async<F: Future>(
    launch: Option<HttpServiceLaunch>,
    future: F,
) -> F::Output {
    let mut future = std::pin::pin!(future);
    std::future::poll_fn(move |cx| {
        with_http_service_launch(launch.clone(), || future.as_mut().poll(cx))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_context_restores_after_nested_unwind_and_interleaved_polls() {
        fn value() -> Option<u32> {
            current_http_service_launch().map(|v| *v.settings::<u32>().unwrap())
        }
        let first = HttpServiceLaunch::new(Arc::new(17u32));
        let second = HttpServiceLaunch::new(Arc::new(29u32));
        with_http_service_launch(Some(first.clone()), || {
            assert_eq!(value(), Some(17));
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                with_http_service_launch(Some(second.clone()), || {
                    assert_eq!(value(), Some(29));
                    panic!("controlled");
                });
            }));
            assert!(failure.is_err());
            assert_eq!(value(), Some(17));
            with_http_service_launch(None, || {
                assert!(launch_context_is_bound());
                assert_eq!(value(), None);
            });
        });
        assert_eq!(value(), None);
        assert!(!launch_context_is_bound());
        futures::executor::block_on(async {
            let (tx, rx) = futures::channel::oneshot::channel::<()>();
            let mut a = Box::pin(with_http_service_launch_async(Some(first), async {
                assert_eq!(value(), Some(17));
                rx.await.unwrap();
                assert_eq!(value(), Some(17));
            }));
            assert!(futures::poll!(a.as_mut()).is_pending());
            with_http_service_launch_async(Some(second), async {
                assert_eq!(value(), Some(29));
            })
            .await;
            assert_eq!(value(), None);
            tx.send(()).unwrap();
            a.await;
        });
        assert_eq!(value(), None);
    }
}
