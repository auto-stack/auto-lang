//! Named language-side async/resource operations for a local HTTP adapter.
//! The registries and native protocols stay single-source in the VM. Private
//! registry maps, completion notifier mutation and raw VM fields are not exported.
pub struct VmHttpBridge;

impl VmHttpBridge {
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
