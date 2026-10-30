//! Language-owned declaration snapshots, value conversion and resource hooks.
pub use crate::vm::host_capabilities::http_service::messages::{
    ApiParamSig, HttpProgramMetadata, HttpRoute,
};
use crate::vm::host_capabilities::http_service::VmHttpBridge;
static API_PARAM_SIGS: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, Vec<ApiParamSig>>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// PLAN-698 T-02/SD-02: #[api] fn 返回类型显示串侧信道（与 API_PARAM_SIGS
/// 同生命周期模型）。SSE publisher 臂据此判 has_sse（任一 ~Stream 端点）
/// 并按 api_gen broadcast_event_name 同款约定拼 New{Type} 事件名。
static API_RETURN_TYPES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<String, String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// PLAN-776 T-06：用户同名遮蔽集——返回类型裸名命中编译单元类型声明
///（struct/type alias/enum，经 TypeStore）时登记。VM 编组门消费
///（fn_response_kind 先查遮蔽 → Json）：同裸名用户类型不冒领 stdlib
/// 专属身份（AC-07；无运行时模块上下文，来源在编译期冻结于此）。
static API_RETURN_SHADOWED: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashSet<String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

/// PLAN-705 T-04/SD-01: 声明 `~T`（Future<T>）返回的函数名集——HTTP 编组
/// 阶段判定"返回值可能是 future bits"的**元数据门**（禁止把普通 int 的
/// 位模式猜成 future ID 后误消费；AC-01 普通 int 反例由此挡住）。对全部
/// fn 发布（非仅 #[api]——段驱动 handler/`__axum:` fn-ref closure 同样
/// 依此判定；closure 经 func_addr → exports 反查名，同 resolve_params）。
static API_ASYNC_RETURNS: std::sync::LazyLock<std::sync::Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

/// 编组期判定：该函数（或 fn-ref closure 反查出的函数）声明 `~T` 返回。
pub fn fn_is_api_async(fn_name: &str) -> bool {
    if let Some(found) = with_program_metadata(|p| p.async_returns.contains(fn_name)) {
        return found;
    }
    API_ASYNC_RETURNS
        .lock()
        .ok()
        .map(|t| t.contains(fn_name))
        .unwrap_or(false)
}

pub fn record_api_param_sigs(fn_name: &str, sigs: Vec<ApiParamSig>) {
    VmHttpBridge::record_api_param_sigs(fn_name, &sigs);
    if let Ok(mut table) = API_PARAM_SIGS.lock() {
        table.insert(fn_name.to_string(), sigs);
    }
}
pub fn api_param_sigs(fn_name: &str) -> Option<Vec<ApiParamSig>> {
    if let Some(value) = with_program_metadata(|p| p.params.get(fn_name).cloned()) {
        return value;
    }
    API_PARAM_SIGS
        .lock()
        .ok()
        .and_then(|t| t.get(fn_name).cloned())
}
pub fn record_api_return_type(fn_name: &str, ret_display: String) {
    VmHttpBridge::record_api_return_type(fn_name, &ret_display);
    if let Ok(mut table) = API_RETURN_TYPES.lock() {
        table.insert(fn_name.to_string(), ret_display);
    }
}
pub fn api_return_type(fn_name: &str) -> Option<String> {
    with_program_metadata(|p| p.returns.get(fn_name).cloned()).unwrap_or_else(|| {
        API_RETURN_TYPES
            .lock()
            .ok()
            .and_then(|t| t.get(fn_name).cloned())
    })
}
pub fn record_api_return_shadowed(fn_name: &str) {
    VmHttpBridge::record_api_return_shadowed(fn_name);
    if let Ok(mut table) = API_RETURN_SHADOWED.lock() {
        table.insert(fn_name.to_string());
    }
}
pub fn has_stream_endpoint() -> bool {
    if let Some(value) = with_program_metadata(|p| p.returns.values().any(|r| r.contains("Stream")))
    {
        return value;
    }
    API_RETURN_TYPES
        .lock()
        .map(|t| t.values().any(|r| r.contains("Stream")))
        .unwrap_or(false)
}
pub fn clear_api_param_sigs() {
    if let Ok(mut table) = API_PARAM_SIGS.lock() {
        table.clear();
    }
    if let Ok(mut table) = API_RETURN_TYPES.lock() {
        table.clear();
    }
    if let Ok(mut table) = API_RETURN_SHADOWED.lock() {
        table.clear();
    }
    if let Ok(mut table) = API_ASYNC_RETURNS.lock() {
        table.clear();
    }
}
pub fn record_api_async_return(fn_name: &str) {
    VmHttpBridge::record_api_async_return(fn_name);
    if let Ok(mut table) = API_ASYNC_RETURNS.lock() {
        table.insert(fn_name.to_string());
    }
}
pub fn fn_is_api_file_return(fn_name: &str) -> bool {
    fn_response_kind(fn_name) == crate::api::contract::ResponseKind::File
}
pub fn fn_is_api_upload_return(fn_name: &str) -> bool {
    fn_response_kind(fn_name) == crate::api::contract::ResponseKind::Upload
}
pub fn fn_response_kind(fn_name: &str) -> crate::api::contract::ResponseKind {
    if let Some(value) = with_program_metadata(|p| {
        if p.shadowed.contains(fn_name) {
            crate::api::contract::ResponseKind::Json
        } else {
            p.returns
                .get(fn_name)
                .map(|r| crate::api::contract::ResponseKind::from_return_string(r))
                .unwrap_or(crate::api::contract::ResponseKind::Json)
        }
    }) {
        return value;
    }
    if API_RETURN_SHADOWED
        .lock()
        .map(|t| t.contains(fn_name))
        .unwrap_or(false)
    {
        return crate::api::contract::ResponseKind::Json;
    }
    API_RETURN_TYPES
        .lock()
        .ok()
        .and_then(|t| {
            t.get(fn_name)
                .map(|r| crate::api::contract::ResponseKind::from_return_string(r))
        })
        .unwrap_or(crate::api::contract::ResponseKind::Json)
}
pub fn route_declares_upload(fn_name: &str) -> bool {
    api_param_sigs(fn_name)
        .map(|sigs| {
            sigs.iter()
                .any(|p| crate::api::contract::is_upload_param(&p.ty))
        })
        .unwrap_or(false)
}
pub fn get_routes() -> Vec<HttpRoute> {
    if let Some(routes) = with_program_metadata(|p| p.routes.clone()) {
        return routes;
    }
    crate::vm::ffi::stdlib::get_http_routes()
        .into_iter()
        .map(|(method, path, fn_name)| HttpRoute {
            method,
            path,
            fn_name,
        })
        .collect()
}
pub fn set_rate_limit(max: u32, window_ms: u64) {
    VmHttpBridge::set_default_rate_limit(max, window_ms);
}
pub fn clear_rate_limit() {
    VmHttpBridge::set_default_rate_limit(0, 1);
}
pub(crate) fn current_request_cancelled() -> bool {
    crate::vm::host_capabilities::request_context::current_request_cancelled()
}
pub(crate) fn register_scope_stream(stream_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::Stream, stream_id);
}
pub(crate) fn register_scope_transfer(transfer_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::Transfer, transfer_id);
}
pub(crate) fn register_scope_file_response(descriptor_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::FileResponse, descriptor_id);
}
pub(crate) fn register_scope_upload(upload_id: u64) {
    use crate::vm::host_capabilities::request_context::*;
    register_request_resource(RequestResourceKind::Upload, upload_id);
}
pub fn json_escape_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
#[path = "http_server/server_metadata.rs"]
mod server_metadata;
pub use server_metadata::*;
#[path = "http_server/value_json.rs"]
mod value_json;
pub use value_json::*;
/// 三处 e2e 启动助手（plan730 `start_server`、plan326 `start_server`、
/// `start_example_api_server`）统一经此 spawn：`crate::run` 的 Err 与
/// **panic 原始载荷**都经通道回传，父线程在 ready 窗口内即时呈现真实
/// 原因，不落入客户端 connect-refused 等待（751-R2：吞错 × 本机 ~2.15s/次
/// 拒绝连接把启动失败藏成 nextest 120s TERMINATING）。
///
/// panic 载荷必须 downcast 提取字符串——`{panic:?}` 对 `Box<dyn Any>` 只会
/// 得到 `Any { .. }`，丢失原因（R2 负向反例实证）。
#[cfg(any(test, feature = "test-http-e2e"))]
pub(crate) fn spawn_outcome_reporter<F, T>(
    stack_size: usize,
    label: &str,
    body: F,
) -> std::sync::mpsc::Receiver<String>
where
    F: FnOnce() -> crate::error::AutoResult<T> + Send + 'static,
    T: Send + 'static,
{
    let label = label.to_string();
    let (outcome_tx, outcome_rx) = std::sync::mpsc::channel::<String>();
    std::thread::Builder::new()
        .stack_size(stack_size)
        .spawn(move || {
            let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(body)) {
                Ok(Ok(_)) => format!("{label} returned Ok without serving"),
                Ok(Err(err)) => format!("{label} failed: {err}"),
                Err(panic) => format!("{label} panicked: {}", panic_payload_text(panic)),
            };
            let _ = outcome_tx.send(outcome);
        })
        .expect("spawn server thread");
    outcome_rx
}

/// Cross-crate HTTP test harness port; not present in ordinary product builds.
#[cfg(feature = "test-http-e2e")]
#[doc(hidden)]
pub fn spawn_outcome_reporter_for_test<F, T>(
    stack_size: usize,
    label: &str,
    body: F,
) -> std::sync::mpsc::Receiver<String>
where
    F: FnOnce() -> crate::error::AutoResult<T> + Send + 'static,
    T: Send + 'static,
{
    spawn_outcome_reporter(stack_size, label, body)
}

/// catch_unwind 载荷的字符串提取（`&'static str` / `String` / 未知兜底）。
#[cfg(any(test, feature = "test-http-e2e"))]
pub(crate) fn panic_payload_text(panic: Box<dyn std::any::Any + Send>) -> String {
    match panic.downcast::<String>() {
        Ok(text) => *text,
        Err(panic) => match panic.downcast::<&'static str>() {
            Ok(text) => (*text).to_string(),
            Err(_) => "<non-string panic payload>".to_string(),
        },
    }
}

#[cfg(test)]
mod plan755_outcome_reporter_tests {
    /// P755-R2-01 负向冻结：真实 spawn/通道/catch_unwind 路径必须转发 panic
    /// 原始载荷——父线程侧消息携带原因，而不是 `Any { .. }`。
    #[test]
    fn panic_payload_is_forwarded_through_real_spawn_path() {
        let rx = super::spawn_outcome_reporter::<_, String>(8 * 1024 * 1024, "test-run", || {
            panic!("PLAN755_PANIC_PAYLOAD_MARKER")
        });
        let outcome = rx
            .recv_timeout(std::time::Duration::from_secs(10))
            .expect("outcome reported");
        assert!(
            outcome.contains("PLAN755_PANIC_PAYLOAD_MARKER"),
            "父线程消息丢失 panic 载荷: {outcome}"
        );
    }

    #[test]
    fn panic_payload_text_covers_string_static_and_foreign() {
        assert_eq!(
            super::panic_payload_text(Box::new(String::from("owned"))),
            "owned"
        );
        assert_eq!(super::panic_payload_text(Box::new("static")), "static");
        assert_eq!(
            super::panic_payload_text(Box::new(755usize)),
            "<non-string panic payload>"
        );
    }
}

// =============================================================================
// Plan 326 Phase 3: serialization unit tests
// =============================================================================

pub fn serve_blocking_stdnet(vm: &crate::vm::engine::AutoVM, addr: &str) {
    if let Err(error) =
        crate::vm::host_capabilities::http_service::listen_legacy_registered(vm, addr)
    {
        eprintln!("[HTTP] Legacy stdnet server failed on {addr}: {error}");
    }
}
