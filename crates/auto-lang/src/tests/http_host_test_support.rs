//! Explicit SDK assembly for legacy service tests during PLAN-890 extraction.
//! No default is installed in product code. T-05 replaces this one adapter with
//! the real Web host when the temporary Core runtime bridge is removed.
use auto_lang_core::vm::host_capabilities::http_service::{
    install_process_http_service_host_if_absent, HttpServiceFuture, HttpServiceHost, HttpVmOwner,
};

struct ExistingRuntimeTestHost;

impl HttpServiceHost for ExistingRuntimeTestHost {
    fn serve(&self, owner: HttpVmOwner) -> HttpServiceFuture {
        Box::pin(owner.run_existing_runtime_for_probe())
    }
}

pub(super) fn install() {
    install_process_http_service_host_if_absent(std::sync::Arc::new(ExistingRuntimeTestHost));
}
