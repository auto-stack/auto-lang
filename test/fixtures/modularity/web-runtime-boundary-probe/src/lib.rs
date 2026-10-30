//! PLAN-890 bounded external-crate audit of the actual service source.
//! No source/VM/AST copy, no public re-export of private Core modules, and
//! no production Web repository. Default feature selection is not a check.
pub use auto_lang_core::*;

#[cfg(feature = "boundary")]
#[path = "../../../../../crates/auto-lang-core/src/vm/ffi/http_server.rs"]
pub mod http_server;
#[cfg(feature = "boundary")]
#[path = "../../../../../crates/auto-lang-core/src/vm/ffi/http_transport.rs"]
pub mod http_transport;

pub use auto_lang_core::vm::ffi::{
    async_http, axum_adapter, http_return_resource, http_server_file, http_stream, http_transfer,
    http_upload, stdlib,
};

#[cfg(feature = "boundary")]
pub mod vm {
    pub use auto_lang_core::vm::*;
    pub mod ffi {
        pub use crate::{http_server, http_transport};
        pub use auto_lang_core::vm::ffi::*;
    }
}

/// The actual service owner/transport run in this external crate. No call to
/// Core's temporary run_existing_runtime_for_probe is made on this path.
#[cfg(feature = "boundary")]
pub async fn serve_owned(
    owner: auto_lang_core::vm::host_capabilities::http_service::HttpVmOwner,
    address: String,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> auto_lang_core::AutoResult<()> {
    let (vm, metadata) = owner.into_local_parts();
    let config = http_transport::TransportConfig::from_env();
    http_server::with_owner_metadata(
        metadata,
        http_server::serve_with(vm, &address, config, shutdown),
    )
    .await;
    Ok(())
}
