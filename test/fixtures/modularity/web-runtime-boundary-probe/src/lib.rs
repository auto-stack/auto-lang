//! PLAN-890 bounded external-crate audit of the actual service source.
//! No source/VM/AST copy, no public re-export of private Core modules, and
//! no production Web repository. Default feature selection is not a check.
pub use auto_lang_core::*;

#[cfg(feature = "boundary")]
#[path = "../../../../../crates/auto-lang-core/src/vm/ffi/http_server.rs"]
pub mod http_server;
#[cfg(feature = "boundary")]
#[path = "../../../../../crates/auto-lang-core/src/vm/ffi/http_transport.rs"]
mod http_transport_impl;

/// Fixture-only thread-terminal delay after the real transport returns. The
/// entire production transport still executes from its original source.
#[cfg(feature = "boundary")]
pub mod http_transport {
    pub use super::http_transport_impl::TransportConfig;
    static LIVE_TAILS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    pub fn live_network_tails() -> usize {
        LIVE_TAILS.load(std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) async fn serve_network(
        addr: String,
        cfg: TransportConfig,
        requests: tokio::sync::mpsc::Sender<(
            crate::http_server::ApiRequest,
            tokio::sync::oneshot::Sender<crate::http_server::ApiReply>,
            u64,
        )>,
        shutdown: tokio::sync::watch::Receiver<bool>,
        ready: tokio::sync::oneshot::Sender<Result<String, String>>,
        metadata: Option<std::sync::Arc<crate::http_server::HttpProgramMetadata>>,
    ) {
        super::http_transport_impl::serve_network(addr, cfg, requests, shutdown, ready, metadata)
            .await;
        if let Ok(delay) = std::env::var("PLAN890_NETWORK_TAIL_MS") {
            let delay: u64 = delay.parse().expect("fixture network tail delay");
            assert!((1..=1000).contains(&delay));
            struct Tail;
            impl Drop for Tail {
                fn drop(&mut self) {
                    LIVE_TAILS.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                }
            }
            LIVE_TAILS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let _tail = Tail;
            eprintln!("probe real network thread tail started");
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
            eprintln!("probe real network thread tail completed");
        }
    }
}

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
    sender: tokio::sync::watch::Sender<bool>,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> auto_lang_core::AutoResult<()> {
    let configured = owner
        .launch()
        .map(|launch| {
            launch
                .settings::<auto_lang_core::http_service_config::HttpServiceConfig>()
                .ok_or_else(|| {
                    auto_lang_core::error::AutoError::Msg(
                        "unsupported HTTP launch settings type".into(),
                    )
                })
        })
        .transpose()?;
    if let Some(config) = configured {
        let address = format!("{}:{}", config.listen_addr, config.listen_port);
        return serve_service_owned(owner, address, config, sender, shutdown).await;
    }
    let (vm, metadata) = owner.into_local_parts();
    if std::env::var("PLAN890_METADATA_GAP").as_deref() == Ok("1") {
        let first = metadata.routes.iter().any(|r| r.path == "/one/:n");
        let declared_async = http_server::with_owner_metadata_sync(metadata.clone(), || {
            http_server::fn_is_api_async("later")
        });
        assert_eq!(
            declared_async, first,
            "actual external reader must select this owner's async declarations"
        );
    }
    let config = http_transport::TransportConfig::from_env();
    http_server::with_owner_metadata(
        metadata,
        http_server::serve_with(vm, &address, config, shutdown),
    )
    .await
}

#[cfg(feature = "boundary")]
pub async fn serve_service_owned(
    owner: auto_lang_core::vm::host_capabilities::http_service::HttpVmOwner,
    address: String,
    config: std::sync::Arc<auto_lang_core::http_service_config::HttpServiceConfig>,
    sender: tokio::sync::watch::Sender<bool>,
    shutdown: tokio::sync::watch::Receiver<bool>,
) -> auto_lang_core::AutoResult<()> {
    let mut runtime = auto_lang_core::http_service_config::ServiceRuntime::new(config);
    // Deliberate fixture-only mutation: the oracle must reject the old shared
    // observation model instead of accepting two listeners as isolation proof.
    if std::env::var("PLAN890_SHARED_OBSERVATION_NEGATIVE").as_deref() == Ok("1") {
        runtime.observation = auto_lang_core::http_service_observability::process_observation();
    }
    if !runtime.bind_shutdown(sender) {
        return Err("service runtime already bound".into());
    }
    runtime
        .observation
        .install_log_sink(runtime.config.observability.log_sink_capacity);
    let observation = runtime.observation.clone();
    observations_slot()
        .lock()
        .unwrap()
        .push(observation.clone());
    let runtime = std::sync::Arc::new(runtime);
    sessions_slot().lock().unwrap().push(runtime.clone());
    let config = http_transport::TransportConfig::from_service_runtime(runtime);
    let (vm, metadata) = owner.into_local_parts();
    http_server::with_owner_metadata(
        metadata,
        http_server::serve_with(vm, &address, config, shutdown),
    )
    .await?;
    assert_eq!(observation.counters().snapshot()["connections_active"], 0);
    assert_eq!(observation.log_threads_active(), 0);
    eprintln!("probe logger joined instance={}", observation.instance_id());
    Ok(())
}

/// Borrowed synchronous handoff uses this external crate's one-source runtime.
#[cfg(feature = "boundary")]
pub fn listen_borrowed(
    vm: &auto_lang_core::vm::engine::AutoVM,
    address: &str,
    metadata: std::sync::Arc<
        auto_lang_core::vm::host_capabilities::http_service::messages::HttpProgramMetadata,
    >,
    kind: auto_lang_core::vm::host_capabilities::http_service::BlockingListenerKind,
    stop: &std::sync::atomic::AtomicBool,
) -> auto_lang_core::AutoResult<()> {
    let result = match kind {
        auto_lang_core::vm::host_capabilities::http_service::BlockingListenerKind::ServerHandle => {
            http_server::server_blocking::serve_server_handle(vm, address, metadata, Some(stop))
        }
        auto_lang_core::vm::host_capabilities::http_service::BlockingListenerKind::LegacyStdnet => {
            http_server::server_legacy::serve_legacy_stdnet(vm, address, metadata, Some(stop))
        }
    };
    result.map_err(|error| auto_lang_core::error::AutoError::Msg(format!("{error:?}")))
}

#[cfg(feature = "boundary")]
fn observations_slot() -> &'static std::sync::Mutex<
    Vec<std::sync::Arc<auto_lang_core::http_service_observability::ServiceObservation>>,
> {
    static OBS: std::sync::OnceLock<
        std::sync::Mutex<
            Vec<std::sync::Arc<auto_lang_core::http_service_observability::ServiceObservation>>,
        >,
    > = std::sync::OnceLock::new();
    OBS.get_or_init(Default::default)
}
#[cfg(feature = "boundary")]
pub fn observations(
) -> Vec<std::sync::Arc<auto_lang_core::http_service_observability::ServiceObservation>> {
    observations_slot().lock().unwrap().clone()
}

#[cfg(feature = "boundary")]
fn sessions_slot() -> &'static std::sync::Mutex<
    Vec<std::sync::Arc<auto_lang_core::http_service_config::ServiceRuntime>>,
> {
    static SESSIONS: std::sync::OnceLock<
        std::sync::Mutex<Vec<std::sync::Arc<auto_lang_core::http_service_config::ServiceRuntime>>>,
    > = std::sync::OnceLock::new();
    SESSIONS.get_or_init(Default::default)
}
#[cfg(feature = "boundary")]
pub fn sessions() -> Vec<std::sync::Arc<auto_lang_core::http_service_config::ServiceRuntime>> {
    sessions_slot().lock().unwrap().clone()
}
