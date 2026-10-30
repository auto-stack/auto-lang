//! Explicit HTTP host handoff. Core never installs an implicit implementation.
//!
//! The provider is process-shareable, but its per-service future and owner are
//! local. The !Send VM therefore remains on the LocalSet's owning thread.
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use crate::vm::engine::AutoVM;
use crate::AutoResult;

mod bridge;
pub(crate) mod program_metadata;
pub use program_metadata::CompilationMetadataGuard;
/// Framework-neutral, owned messages shared by network and local VM owners.
pub mod messages;
pub use bridge::VmHttpBridge;
mod launch;
pub use launch::*;

// Transitional compatibility input for existing Man assembly. New hosts use
// the explicit launch context; this take happens at invocation entry only.
pub(crate) fn invocation_launch() -> Option<HttpServiceLaunch> {
    if launch::launch_context_is_bound() {
        return current_http_service_launch();
    }
    current_http_service_launch().or_else(|| {
        crate::http_service_config::take_active_service_config().map(HttpServiceLaunch::new)
    })
}

// Compatibility embedding default: pure settings, not window state or policy
// execution. Both the external adapter and the old SDK facade use this source.
static DEFAULT_RATE_LIMIT: RwLock<Option<(u32, u64)>> = RwLock::new(None);

pub type HttpServiceFuture = Pin<Box<dyn Future<Output = AutoResult<()>> + 'static>>;

/// Distinct supported legacy grades; the host must not silently substitute one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockingListenerKind {
    ServerHandle,
    LegacyStdnet,
}

pub trait HttpServiceHost: Send + Sync {
    fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture;

    /// Synchronous language/embedding listener on the existing VM owner thread.
    /// An async-only custom provider fails explicitly rather than falling back
    /// to an implicit Core network runtime.
    fn listen_blocking(
        &self,
        _vm: &AutoVM,
        _address: &str,
        _metadata: Arc<messages::HttpProgramMetadata>,
        _kind: BlockingListenerKind,
    ) -> AutoResult<()> {
        Err(crate::error::AutoError::Msg(
            "installed HTTP service host does not support blocking listeners".into(),
        ))
    }
}

/// One local VM service owner. The handoff reuses the existing public VM SDK;
/// its Rc remains !Send, and no private VM fields or raw pointers are exported.
pub struct HttpVmOwner {
    vm: Rc<AutoVM>,
    routes: Vec<(String, String, String)>,
    metadata: Arc<crate::vm::ffi::http_server::HttpProgramMetadata>,
}

impl HttpVmOwner {
    pub fn launch(&self) -> Option<&HttpServiceLaunch> {
        self.vm.http_service_launch()
    }

    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Consume the owner on its local execution thread. The serving adapter
    /// owns the framework; Core supplies the existing language VM object and
    /// one immutable compiler-metadata snapshot, without starting a listener.
    pub fn into_local_parts(self) -> (Rc<AutoVM>, Arc<messages::HttpProgramMetadata>) {
        (self.vm, self.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct RestoreHost(Option<Arc<dyn HttpServiceHost>>);

    impl Drop for RestoreHost {
        fn drop(&mut self) {
            *DEFAULT_HOST.write().expect("HTTP host registry poisoned") = self.0.take();
        }
    }

    struct RecordingHost {
        id: u8,
        seen: Arc<Mutex<Vec<(u8, usize)>>>,
        gate: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
    }

    impl HttpServiceHost for RecordingHost {
        fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
            let id = self.id;
            let seen = self.seen.clone();
            let gate = self.gate.lock().unwrap().take();
            let owner_thread = std::thread::current().id();
            Box::pin(async move {
                if let Some(gate) = gate {
                    gate.await.expect("release active service");
                }
                assert_eq!(std::thread::current().id(), owner_thread);
                seen.lock().unwrap().push((id, owner.route_count()));
                Ok(())
            })
        }
    }

    struct CompletingHost(Arc<Mutex<Vec<(u64, bool)>>>);
    impl HttpServiceHost for CompletingHost {
        fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
            let seen = self.0.clone();
            let (vm, _) = owner.into_local_parts();
            Box::pin(async move {
                seen.lock().unwrap().push((vm.instance_id, true));
                Ok(())
            })
        }

        fn listen_blocking(
            &self,
            vm: &AutoVM,
            address: &str,
            _metadata: Arc<messages::HttpProgramMetadata>,
            _kind: BlockingListenerKind,
        ) -> AutoResult<()> {
            if address == "reject-before-start" {
                return Err("controlled host rejection".into());
            }
            self.0.lock().unwrap().push((vm.instance_id, false));
            Ok(())
        }
    }

    fn owner_vm() -> AutoVM {
        AutoVM::new_without_host_snapshot(crate::vm::virt_memory::VirtualFlash::new(0), 8192)
    }

    #[test]
    fn file_entries_preserve_launch_snapshot_and_restore_caller_context() {
        struct LaunchHost(Arc<Mutex<Vec<Option<u32>>>>);
        impl HttpServiceHost for LaunchHost {
            fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
                let launch = owner.launch().map(|v| *v.settings::<u32>().unwrap());
                self.0.lock().unwrap().push(launch);
                Box::pin(async { Ok(()) })
            }
        }
        let _restore = RestoreHost(DEFAULT_HOST.read().unwrap().clone());
        let seen = Arc::new(Mutex::new(Vec::new()));
        set_process_default_http_service_host(Arc::new(LaunchHost(seen.clone())));
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("file-entry.at");
        std::fs::write(
            &source,
            "#[api(method=\"GET\",path=\"/value\")]\nfn value() int { 42 }\n",
        )
        .unwrap();
        let path = source.to_str().unwrap();
        for value in [17u32, 29] {
            with_http_service_launch(Some(HttpServiceLaunch::new(Arc::new(value))), || {
                crate::run_file_with_args(path, vec!["owned-argument".into()]).unwrap();
                assert_eq!(
                    *current_http_service_launch()
                        .unwrap()
                        .settings::<u32>()
                        .unwrap(),
                    value
                );
            });
            assert!(current_http_service_launch().is_none());
        }
        with_http_service_launch(None, || crate::run_file(path)).unwrap();
        assert_eq!(*seen.lock().unwrap(), [Some(17), Some(29), None]);
        std::fs::write(&source, "fn main() int { 42 }\n").unwrap();
        with_http_service_launch(Some(HttpServiceLaunch::new(Arc::new(31u32))), || {
            assert_eq!(crate::run_file(path).unwrap(), "42");
            assert_eq!(
                *current_http_service_launch()
                    .unwrap()
                    .settings::<u32>()
                    .unwrap(),
                31
            );
        });
        assert_eq!(
            seen.lock().unwrap().len(),
            3,
            "ordinary file must not invoke the host"
        );
        assert!(current_http_service_launch().is_none());
    }

    #[test]
    fn explicit_http_host_selection_is_stable_for_active_local_owners() {
        let saved = DEFAULT_HOST.read().unwrap().clone();
        let _restore = RestoreHost(saved);
        clear_process_default_http_service_host();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (release, gate) = tokio::sync::oneshot::channel();
        let first: Arc<dyn HttpServiceHost> = Arc::new(RecordingHost {
            id: 1,
            seen: seen.clone(),
            gate: Mutex::new(Some(gate)),
        });
        let second: Arc<dyn HttpServiceHost> = Arc::new(RecordingHost {
            id: 2,
            seen: seen.clone(),
            gate: Mutex::new(None),
        });
        assert!(install_process_http_service_host_if_absent(first.clone()));
        assert!(!install_process_http_service_host_if_absent(second.clone()));
        assert!(listen_registered(&owner_vm(), "127.0.0.1:0")
            .unwrap_err()
            .to_string()
            .contains("does not support blocking listeners"));
        futures::executor::block_on(async {
            let routes = vec![("GET".into(), "/first".into(), "first".into())];
            let mut active = Box::pin(serve_registered(owner_vm(), routes));
            assert!(matches!(
                futures::poll!(active.as_mut()),
                std::task::Poll::Pending
            ));
            set_process_default_http_service_host(second);
            assert!(!install_process_http_service_host_if_absent(first));
            let later_routes = vec![
                ("GET".into(), "/later-a".into(), "later_a".into()),
                ("GET".into(), "/later-b".into(), "later_b".into()),
            ];
            serve_registered(owner_vm(), later_routes).await.unwrap();
            assert_eq!(*seen.lock().unwrap(), [(2, 2)]);
            release.send(()).unwrap();
            active.await.unwrap();
            assert_eq!(*seen.lock().unwrap(), [(2, 2), (1, 1)]);
            clear_process_default_http_service_host();
            let missing = serve_registered(owner_vm(), Vec::new()).await.unwrap_err();
            assert!(missing.to_string().contains("adapter is not installed"));
            assert!(listen_registered(&owner_vm(), "127.0.0.1:0")
                .unwrap_err()
                .to_string()
                .contains("adapter is not installed"));
            let owner_events = Arc::new(Mutex::new(Vec::new()));
            let completion_host: Arc<dyn HttpServiceHost> =
                Arc::new(CompletingHost(owner_events.clone()));
            let completed = owner_vm();
            let completed_id = completed.instance_id;
            let fresh = owner_vm();
            let fresh_id = fresh.instance_id;
            set_process_default_http_service_host(completion_host.clone());
            listen_registered(&completed, "bounded-owner").unwrap();
            assert!(completed.http_service_claimed());
            assert!(!fresh.http_service_claimed(), "completion is per VM");
            clear_process_default_http_service_host();
            serve_registered(completed, Vec::new()).await.unwrap();
            assert!(serve_registered(owner_vm(), Vec::new()).await.is_err());
            set_process_default_http_service_host(completion_host);
            assert!(listen_registered(&fresh, "reject-before-start").is_err());
            assert!(
                !fresh.http_service_claimed(),
                "failure must not claim owner"
            );
            serve_registered(fresh, Vec::new()).await.unwrap();
            assert_eq!(
                *owner_events.lock().unwrap(),
                [(completed_id, false), (fresh_id, true)]
            );
        });
    }
}

static DEFAULT_HOST: RwLock<Option<Arc<dyn HttpServiceHost>>> = RwLock::new(None);

/// Explicit replacement. A service already handed off retains its host/future.
pub fn set_process_default_http_service_host(host: Arc<dyn HttpServiceHost>) {
    *DEFAULT_HOST.write().expect("HTTP host registry poisoned") = Some(host);
}

/// Explicit installation that preserves an already selected custom host.
pub fn install_process_http_service_host_if_absent(host: Arc<dyn HttpServiceHost>) -> bool {
    let mut slot = DEFAULT_HOST.write().expect("HTTP host registry poisoned");
    if slot.is_some() {
        return false;
    }
    *slot = Some(host);
    true
}

pub fn clear_process_default_http_service_host() {
    *DEFAULT_HOST.write().expect("HTTP host registry poisoned") = None;
}

fn selected_host() -> AutoResult<Arc<dyn HttpServiceHost>> {
    DEFAULT_HOST
        .read()
        .expect("HTTP host registry poisoned")
        .clone()
        .ok_or_else(|| {
            crate::error::AutoError::Msg(
                "HTTP service host adapter is not installed; explicitly install the Web adapter before running a service"
                    .to_owned(),
            )
        })
}

/// Select before any socket is bound. The borrowed VM cannot leave its owner
/// thread; the host receives an immutable metadata snapshot for this call.
pub fn listen_registered(vm: &AutoVM, address: &str) -> AutoResult<()> {
    listen_registered_kind(vm, address, BlockingListenerKind::ServerHandle)
}

pub fn listen_legacy_registered(vm: &AutoVM, address: &str) -> AutoResult<()> {
    listen_registered_kind(vm, address, BlockingListenerKind::LegacyStdnet)
}

fn listen_registered_kind(
    vm: &AutoVM,
    address: &str,
    kind: BlockingListenerKind,
) -> AutoResult<()> {
    let host = selected_host()?;
    let routes = crate::vm::ffi::stdlib::get_http_routes();
    let metadata = VmHttpBridge::program_metadata(vm)
        .unwrap_or_else(|| crate::vm::ffi::http_server::capture_program_metadata(&routes));
    host.listen_blocking(vm, address, metadata, kind)?;
    vm.note_http_service_completed();
    Ok(())
}

pub(crate) async fn serve_registered(
    vm: AutoVM,
    routes: Vec<(String, String, String)>,
) -> AutoResult<()> {
    // A direct native/SDK listener already used this VM and returned. Its
    // explicit owner completion consumes the automatic annotation activation.
    if vm.http_service_claimed() {
        return Ok(());
    }
    let host = selected_host()?;
    let metadata = VmHttpBridge::program_metadata(&vm)
        .unwrap_or_else(|| crate::vm::ffi::http_server::capture_program_metadata(&routes));
    let routes = metadata
        .routes
        .iter()
        .map(|r| (r.method.clone(), r.path.clone(), r.fn_name.clone()))
        .collect();
    let owner = HttpVmOwner {
        vm: Rc::new(vm),
        routes,
        metadata,
    };
    host.serve(owner).await
}
