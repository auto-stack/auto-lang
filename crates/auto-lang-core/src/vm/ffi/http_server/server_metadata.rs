//! Immutable program metadata selected while polling a local service owner.
//! Later compilation may replace the producer tables but cannot alter an
//! already activated owner's route/signature/return classification snapshot.
use super::*;
use std::{cell::RefCell, sync::Arc};

thread_local! {
    static ACTIVE: RefCell<Option<Arc<HttpProgramMetadata>>> = const { RefCell::new(None) };
}

pub fn capture_program_metadata(routes: &[(String, String, String)]) -> Arc<HttpProgramMetadata> {
    Arc::new(HttpProgramMetadata {
        params: API_PARAM_SIGS
            .lock()
            .expect("API param table poisoned")
            .clone(),
        returns: API_RETURN_TYPES
            .lock()
            .expect("API return table poisoned")
            .clone(),
        shadowed: API_RETURN_SHADOWED
            .lock()
            .expect("API shadow table poisoned")
            .clone(),
        async_returns: API_ASYNC_RETURNS
            .lock()
            .expect("API async table poisoned")
            .clone(),
        routes: routes
            .iter()
            .map(|(method, path, fn_name)| HttpRoute {
                method: method.clone(),
                path: path.clone(),
                fn_name: fn_name.clone(),
            })
            .collect(),
    })
}

pub fn with_program_metadata<R>(read: impl FnOnce(&HttpProgramMetadata) -> R) -> Option<R> {
    ACTIVE.with(|active| active.borrow().as_deref().map(read))
}

pub fn current_program_metadata() -> Option<Arc<HttpProgramMetadata>> {
    ACTIVE.with(|active| active.borrow().clone())
}

pub async fn with_optional_owner_metadata<F: std::future::Future>(
    metadata: Option<Arc<HttpProgramMetadata>>,
    future: F,
) -> F::Output {
    match metadata {
        Some(metadata) => with_owner_metadata(metadata, future).await,
        None => future.await,
    }
}

struct Restore(Option<Arc<HttpProgramMetadata>>);
impl Drop for Restore {
    fn drop(&mut self) {
        ACTIVE.with(|active| *active.borrow_mut() = self.0.take());
    }
}

/// Select the same immutable metadata for a synchronous owner call. Nested
/// calls and unwinding restore the previous owner's context.
pub fn with_owner_metadata_sync<R>(
    metadata: Arc<HttpProgramMetadata>,
    call: impl FnOnce() -> R,
) -> R {
    let previous = ACTIVE.with(|active| active.replace(Some(metadata)));
    let _restore = Restore(previous);
    call()
}

/// Context is restored after every poll, including pending/unwind. Keeping it
/// set for the whole async lifetime would incorrectly affect sibling owners.
pub async fn with_owner_metadata<F: std::future::Future>(
    metadata: Arc<HttpProgramMetadata>,
    future: F,
) -> F::Output {
    let mut future = std::pin::pin!(future);
    std::future::poll_fn(move |cx| {
        with_owner_metadata_sync(metadata.clone(), || future.as_mut().poll(cx))
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};
    use std::{future::Future, task::Poll};

    fn program(path: &str) -> Arc<HttpProgramMetadata> {
        Arc::new(HttpProgramMetadata {
            params: HashMap::new(),
            returns: HashMap::new(),
            shadowed: HashSet::new(),
            async_returns: HashSet::new(),
            routes: vec![HttpRoute {
                method: "GET".into(),
                path: path.into(),
                fn_name: "same".into(),
            }],
        })
    }

    #[test]
    fn owner_metadata_restores_after_pending_nested_poll_and_unwind() {
        assert!(current_program_metadata().is_none());
        let mut calls = 0;
        let future = std::future::poll_fn(|_| {
            assert_eq!(get_routes()[0].path, "/outer");
            let mut inner = std::pin::pin!(with_owner_metadata(program("/inner"), async {
                assert_eq!(get_routes()[0].path, "/inner");
            }));
            let waker = futures::task::noop_waker();
            let mut cx = std::task::Context::from_waker(&waker);
            assert!(inner.as_mut().poll(&mut cx).is_ready());
            assert_eq!(get_routes()[0].path, "/outer");
            calls += 1;
            if calls == 1 {
                Poll::Pending
            } else {
                Poll::Ready(())
            }
        });
        let mut owner = std::pin::pin!(with_owner_metadata(program("/outer"), future));
        let waker = futures::task::noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);
        assert!(owner.as_mut().poll(&mut cx).is_pending());
        assert!(
            current_program_metadata().is_none(),
            "pending must not contaminate caller"
        );
        assert!(owner.as_mut().poll(&mut cx).is_ready());
        assert!(current_program_metadata().is_none());
        let unwound = std::panic::catch_unwind(|| {
            futures::executor::block_on(with_owner_metadata(program("/panic"), async {
                assert_eq!(get_routes()[0].path, "/panic");
                panic!("controlled metadata owner unwind");
            }));
        });
        assert!(unwound.is_err());
        assert!(
            current_program_metadata().is_none(),
            "unwind must restore caller"
        );
    }
}
