//! Named language-side async/resource operations for a local HTTP adapter.
//! The registries and native protocols stay single-source in the VM. Private
//! registry maps, completion notifier mutation and raw VM fields are not exported.
pub struct VmHttpBridge;

impl VmHttpBridge {
    pub fn record_api_param_sigs(name: &str, params: &[super::messages::ApiParamSig]) {
        super::program_metadata::collect(|p| {
            p.params.insert(name.to_owned(), params.to_vec());
        });
    }

    pub fn record_api_return_type(name: &str, ty: &str) {
        super::program_metadata::collect(|p| {
            p.returns.insert(name.to_owned(), ty.to_owned());
        });
    }

    pub fn record_api_return_shadowed(name: &str) {
        super::program_metadata::collect(|p| {
            p.shadowed.insert(name.to_owned());
        });
    }

    pub fn record_api_async_return(name: &str) {
        super::program_metadata::collect(|p| {
            p.async_returns.insert(name.to_owned());
        });
    }

    /// Immutable declarations bound before executing a compiled program.
    pub fn program_metadata(
        vm: &crate::vm::engine::AutoVM,
    ) -> Option<std::sync::Arc<super::messages::HttpProgramMetadata>> {
        vm.http_program_metadata().cloned()
    }

    /// A custom SDK compiler can freeze its CompilationMetadataGuard and bind
    /// that owned result while it still exclusively owns the mutable VM.
    pub fn bind_program_metadata(
        vm: &mut crate::vm::engine::AutoVM,
        metadata: std::sync::Arc<super::messages::HttpProgramMetadata>,
    ) {
        vm.set_http_program_metadata(metadata);
    }

    /// Current language-owned settings. Read on the existing local VM thread;
    /// the external host supplies its own rate-window state and algorithm.
    pub fn rate_limit(vm: &crate::vm::engine::AutoVM) -> Option<(u32, u64)> {
        vm.http_rate_limit().unwrap_or_else(|| {
            *super::DEFAULT_RATE_LIMIT
                .read()
                .expect("HTTP default settings poisoned")
        })
    }

    /// Explicit process default for the legacy embedding setter. A language
    /// native setting (including disable) overrides it for that VM only.
    pub fn set_default_rate_limit(max: u32, window_ms: u64) {
        *super::DEFAULT_RATE_LIMIT
            .write()
            .expect("HTTP default settings poisoned") =
            (max != 0).then_some((max, window_ms.max(1)));
    }

    pub fn finalize_resources(
        kind: crate::vm::host_capabilities::request_context::RequestResourceKind,
        ids: &[u64],
    ) {
        use crate::vm::host_capabilities::request_context::RequestResourceKind;
        match kind {
            RequestResourceKind::Stream => {
                for id in ids {
                    crate::vm::ffi::http_stream::stream_cancel(*id);
                }
            }
            RequestResourceKind::Transfer => Self::finalize_transfers(ids),
            RequestResourceKind::FileResponse => Self::finalize_file_responses(ids),
            RequestResourceKind::Upload => Self::finalize_uploads(ids),
        }
    }

    pub fn completion_notified() -> tokio::sync::futures::Notified<'static> {
        crate::vm::ffi::async_http::COMPLETION_NOTIFY.notified()
    }

    pub fn notify_completion() {
        crate::vm::ffi::async_http::COMPLETION_NOTIFY.notify_waiters();
    }

    pub fn discard_async_result(id: u64) {
        let _ = crate::vm::ffi::stdlib::check_async_http_result(id);
    }

    pub async fn wait_stream_ready(id: u64) {
        crate::vm::ffi::http_stream::wait_stream_ready(id).await;
    }

    pub async fn wait_legacy_stream_ready(id: u64) {
        crate::vm::ffi::http_stream::wait_legacy_stream_ready(id).await;
    }

    pub fn close_legacy_stream(id: u64) {
        if let Ok(mut streams) = crate::vm::ffi::stdlib::ASYNC_STREAMS.lock() {
            streams.remove(&id);
        }
    }

    pub fn insert_upload_request(request: a2r_std::http::UploadRequest) -> u64 {
        crate::vm::ffi::http_upload::insert_upload_request(request)
    }

    pub fn take_upload_receipt(id: u64) -> Option<a2r_std::http::UploadReceipt> {
        crate::vm::ffi::http_upload::take_upload_receipt(id)
    }

    pub fn finalize_transfers(ids: &[u64]) {
        crate::vm::ffi::http_transfer::scope_finalize_transfers(ids);
    }

    pub fn finalize_file_responses(ids: &[u64]) {
        crate::vm::ffi::http_server_file::scope_finalize_file_responses(ids);
    }

    pub fn finalize_uploads(ids: &[u64]) {
        crate::vm::ffi::http_upload::scope_finalize_uploads(ids);
    }
}

#[cfg(test)]
mod tests {
    use super::VmHttpBridge;
    use crate::vm::{engine::AutoVM, virt_memory::VirtualFlash};

    #[test]
    fn rate_settings_preserve_sdk_default_and_per_vm_explicit_disable() {
        struct Restore(Option<(u32, u64)>);
        impl Drop for Restore {
            fn drop(&mut self) {
                *super::super::DEFAULT_RATE_LIMIT.write().unwrap() = self.0;
            }
        }
        let _restore = Restore(*super::super::DEFAULT_RATE_LIMIT.read().unwrap());
        let first = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 8192);
        let second = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 8192);
        crate::vm::ffi::http_server::set_rate_limit(7, 123);
        assert_eq!(VmHttpBridge::rate_limit(&first), Some((7, 123)));
        first.set_http_rate_limit(2, 0);
        second.set_http_rate_limit(0, 10);
        assert_eq!(VmHttpBridge::rate_limit(&first), Some((2, 1)));
        assert_eq!(VmHttpBridge::rate_limit(&second), None);
        crate::vm::ffi::http_server::set_rate_limit(9, 456);
        assert_eq!(VmHttpBridge::rate_limit(&first), Some((2, 1)));
        assert_eq!(VmHttpBridge::rate_limit(&second), None);
        crate::vm::ffi::http_server::clear_rate_limit();
        assert_eq!(VmHttpBridge::rate_limit(&first), Some((2, 1)));
        let later = AutoVM::new_without_host_snapshot(VirtualFlash::new(0), 8192);
        assert_eq!(VmHttpBridge::rate_limit(&later), None);
    }
}
