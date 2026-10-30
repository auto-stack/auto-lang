//! Pure contract and legacy invocation input; runtime belongs to auto-web.
pub use auto_web_contract::*;
use std::sync::{Arc, Mutex, OnceLock};
static ACTIVE_SERVICE_CONFIG: OnceLock<Mutex<Option<Arc<HttpServiceConfig>>>> = OnceLock::new();

fn active_slot() -> &'static Mutex<Option<Arc<HttpServiceConfig>>> {
    ACTIVE_SERVICE_CONFIG.get_or_init(|| Mutex::new(None))
}

/// serve 入口在 spawn VM 线程**之前**安装；lib.rs 自动启动钩子 take 一次。
pub fn set_active_service_config(cfg: Arc<HttpServiceConfig>) {
    if let Ok(mut slot) = active_slot().lock() {
        *slot = Some(cfg);
    }
}

pub fn take_active_service_config() -> Option<Arc<HttpServiceConfig>> {
    active_slot().lock().ok().and_then(|mut s| s.take())
}
