//! Compile actual bridge types without Core, VM, Axum, UI or Man.
#[path = "../../../../../crates/auto-lang-core/src/vm/host_capabilities/http_service/messages.rs"]
pub mod messages;

#[cfg(test)]
mod tests {
    use super::messages::*;
    use futures::StreamExt;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    struct Body {
        polls: Arc<AtomicUsize>,
        drops: Arc<AtomicUsize>,
    }
    impl futures::Stream for Body {
        type Item = std::io::Result<Vec<u8>>;
        fn poll_next(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            match self.polls.fetch_add(1, Ordering::SeqCst) {
                0 => std::task::Poll::Ready(Some(Ok(vec![0, 127, 255]))),
                1 => std::task::Poll::Ready(Some(Err(std::io::ErrorKind::ConnectionReset.into()))),
                _ => std::task::Poll::Ready(None),
            }
        }
    }
    impl Drop for Body {
        fn drop(&mut self) {
            self.drops.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn owned_messages_preserve_streams_errors_file_identity_and_terminal_drop() {
        let polls = Arc::new(AtomicUsize::new(0));
        let drops = Arc::new(AtomicUsize::new(0));
        let request = ApiRequest {
            method: "POST".into(),
            path: "/upload?q=1".into(),
            headers: vec![("x-request-id".into(), "owned-request".into())],
            body: vec![],
            raw_upload_body: Some(Box::pin(Body {
                polls: polls.clone(),
                drops: drops.clone(),
            })),
            peer: Some("127.0.0.1:1234".parse().unwrap()),
        };
        assert_eq!(request.header("x-request-id"), Some("owned-request"));
        assert_eq!(polls.load(Ordering::SeqCst), 0);
        // An actual thread boundary is stronger than a DTO name or Send assertion.
        let reply = std::thread::spawn(move || {
            futures::executor::block_on(async move {
                assert_eq!(request.path, "/upload?q=1");
                let mut body = request.raw_upload_body.unwrap();
                assert_eq!(body.next().await.unwrap().unwrap(), [0, 127, 255]);
                assert_eq!(
                    body.next().await.unwrap().unwrap_err().kind(),
                    std::io::ErrorKind::ConnectionReset
                );
                assert!(body.next().await.is_none());
                let (sender, receiver) = tokio::sync::mpsc::channel(1);
                sender.try_send("data: owned\n\n".into()).unwrap();
                assert!(sender.try_send("must not fit".into()).is_err());
                drop(sender);
                ApiReply::Full {
                    status: 200,
                    headers: vec![],
                    body: ApiBody::Sse(receiver),
                }
            })
        })
        .join()
        .unwrap();
        let ApiReply::Full {
            status: 200,
            body: ApiBody::Sse(mut frames),
            ..
        } = reply
        else {
            panic!("wrong reply");
        };
        futures::executor::block_on(async {
            assert_eq!(frames.recv().await.as_deref(), Some("data: owned\n\n"));
            assert!(frames.recv().await.is_none());
        });
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        let descriptor = a2r_std::http::file_response("nonexistent-probe-root", "same.bin", "{}");
        let id = descriptor.id();
        let file = std::thread::spawn(move || {
            ApiBody::File(FileReplySeed {
                descriptor,
                request_id: "same-request".into(),
            })
        })
        .join()
        .unwrap();
        let ApiBody::File(seed) = file else {
            panic!("file descriptor lost");
        };
        assert_eq!(seed.descriptor.id(), id);
        assert_eq!(seed.descriptor.relative(), "same.bin");
        assert_eq!(seed.request_id, "same-request");
        assert!(seed.descriptor.init_error().is_none());
    }
}
