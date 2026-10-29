//! PLAN-884 T-04：异步 HTTP 核心（共享 client / 重试 / msg 桥 / auth / 默认头）
//! 自 stdlib 壳迁入。
use super::*;

/// Plan 340: Simple JSON HTTP helpers for VM+VM split mode codegen rewriting.
/// These return the response body as a STRING (not a handle), so codegen can
/// feed it directly into `auto.json.to_value`. Each runs its request on the
/// fixed async client executor (PLAN-705 T-03).
///
/// Plan 340 audit fix: on failure (network error, job panic), return a JSON
/// error object `{"error":"...","status":0}` instead of the literal "null".
/// The old "null" was indistinguishable from a real null response body and made
/// split-mode failures (backend down, wrong port, 404) very hard to debug. The
/// codegen path feeds this into `auto.json.to_value`, which parses it into a
/// VM object with an `error` field the app can surface.
/// PLAN-025 修:split 形态 api.* 家族共享 keep-alive 客户端(30s 超时——back 引导期(等 shell banner 轮询 ~10s)合法长尾,10s 实测误杀冷启 Init)。
/// 原每请求 `Client::new()` = 每调用一条新 TCP 连接:高频轮询应用
/// (auto-term 每 tick 数十次 api 调用)TIME_WAIT 风暴打爆临时端口池
/// (2026-09-21 实录 6986 条 TIME_WAIT),端口耗尽后 reqwest 无默认
/// 超时 → 请求线程永久挂起 → tick task 无限堆积 = 前端"没有响应"
/// + 内存爬升(泄漏修后仍复现的另一半根因)。共享连接池复用连接
/// (稳态 ~数条 ESTABLISHED),超时兜底保证 task 必然终结。

/// PLAN-705 T-03: get_json/post_json 的发射体（原 simple_http_json 的
/// async 形态）。PLAN-712 T-17 起契约分层：传输/读体错误走 Err，由消费端
/// shim 重入臂转成**可捕获 VMError**（.at try/catch 可接——旧形态包成
/// `{"error":..,"status":0}` 成功值是吞错点）；非 2xx 在此包成
/// `{"error":"HTTP n","status":n}`（fetch().json() 同律：HTTP 错误状态
/// 不 reject，值面可检视 `.status`）。
async fn simple_http_json_async(
    method: String,
    url: &str,
    body: Option<&str>,
) -> Result<String, String> {
    // PLAN-648 T-00 仪器:AUTO_LANG_HTTP_TRACE=1 时在 stderr 打印每次
    // api-改写家族请求的方法/URL/状态/响应摘要(split 模式排障用)。
    let trace = std::env::var("AUTO_LANG_HTTP_TRACE").ok().as_deref() == Some("1");
    if trace {
        eprintln!(
            "[HTTP-TRACE] {} {} body={:?}",
            method,
            url,
            body.unwrap_or("")
        );
    }
    let client = shared_async_http_client().clone();
    // Plan 446 E4 / PLAN-048: get_json/post_json 是 #[api] 契约改写
    // (emit_api_http_call)的两条主臂,必须与通用 request 汇聚点同样应用
    // 进程级默认头/默认查询(musk: Authorization Bearer + workspace)。
    let url = append_default_queries(url);
    let default_headers = snapshot_default_headers();
    let mut builder = match method.as_str() {
        "POST" => client.post(&url),
        "PUT" => client.put(&url),
        "DELETE" => client.delete(&url),
        "PATCH" => client.patch(&url),
        _ => client.get(&url),
    };
    // 默认头先落,调用方显式 header 同名追加在后(与 request 臂同序)。
    for (k, v) in &default_headers {
        builder = builder.header(k.as_str(), v.as_str());
    }
    if let Some(b) = body {
        builder = builder
            .header("Content-Type", "application/json")
            .body(b.to_string());
    }
    let resp = builder.send().await.map_err(|e| e.to_string())?;
    let status = resp.status().as_u16();
    // PLAN-705 T-03: 增量响应体预算（text() 换 capped 读）。
    let bytes = crate::vm::ffi::async_http::read_body_capped(
        resp,
        crate::vm::ffi::async_http::client_limits().body_limit,
    )
    .await?;
    let text = String::from_utf8_lossy(&bytes).to_string();
    if trace {
        let summary: String = text.chars().take(120).collect();
        eprintln!("[HTTP-TRACE] <- {} status={} body~{}", url, status, summary);
    }
    if (200..300).contains(&status) {
        // Plan 057 (ash-gui 双执行修复): void 端点(如 /api/run_command)
        // 返回空 body —— 直接把空串喂给 json.to_value 会 parse error,
        // 让 handler 在副作用(POST)之后失败,再触发 on_with_input_for 的
        // legacy 回退误执行根 widget 的同名 handler(App.RunCommand →
        // store.RunCommand 二次执行)。空 body 规范化为 "null"。
        if text.trim().is_empty() {
            Ok("null".to_string())
        } else {
            Ok(text)
        }
    } else {
        Ok(format!(
            r#"{{"error":"HTTP {}","status":{}}}"#,
            status, status
        ))
    }
}

/// PLAN-705 T-03: async 客户端共享连接池（对齐旧 blocking 版 30s 超时 +
/// pool 8;每 host keep-alive 复用,零每请求连接/线程）。
fn shared_async_http_client() -> &'static reqwest::Client {
    static C: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
    C.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_default()
    })
}

/// `auto.http.get_json(url) -> String` — GET, return body as string.
/// Helper: check if a previously spawned async HTTP request has completed.
/// Returns Some(body_string) if ready, None if still pending.
/// Plan 349: reads from the unified result table (Body variant).
/// PLAN-027 缺陷 C:Err 条目不再走 `.ok()`→None——take 已经发生,返回
/// None 会被 shim 判为"仍在等待"而永远 Waiting。镜像 simple_http_json 的
/// 错误 body 契约(`{"error":..,"status":0}`),让调用方拿到可解析对象。
/// PLAN-705 T-02:内部改指统一 live-op 表——take 只消费 Completed,
/// Pending(未完成)不删除条目。
/// 消费一个已完成 async HTTP json 请求的结果（take 语义，单次终结）。
///
/// PLAN-712 T-17（候选②）：**传输层失败改抛，不再吞成错误形状 body**——
/// 旧契约把 `Err(e)`（连接拒绝/超时/DNS/队满）包成 `{"error":..,"status":0}`
/// 字符串当成功值推回，`.at` 侧 try/catch 永不触发，坏数据静默入态（018
/// 详情页「0 entries」实机实证）。新契约对齐浏览器 `fetch()`：**网络层失败
/// = 可捕获异常**（handler 的 catch 分支可呈现可诊断错误）；**非 2xx 仍返回
/// 错误形状值**（含 `.status` 字段，语料可检视——与 fetch().json() 的
/// 「HTTP 错误状态不 reject」语义同律）。
pub(crate) fn check_async_http_result(request_id: u64) -> Option<Result<String, String>> {
    crate::vm::ffi::async_http::take_live_op(request_id).map(|result| match result {
        Ok(AsyncResult::Body(s)) => Ok(s),
        Ok(_) => Err("http: unexpected async result variant".to_string()),
        Err(e) => Err(e),
    })
}

/// PLAN-027 缺陷 A / PLAN-705 T-02:超时放弃路径的条目回收=统一协议的
/// cancel 幂等终结入口。engine 的同步 drain(call_fn_by_name Yield 臂 /
/// request-builder send 臂)30s 超时后调用;此后的 worker 迟到完成经
/// `complete_live_op` 的 presence 守卫被丢弃(旧实现裸 remove 与完成写
/// 无同步,取消后迟到完成可复活条目——协议缺陷已修)。
pub fn drop_async_result(request_id: u64) {
    crate::vm::ffi::async_http::cancel_live_op(request_id);
}

/// Plan 446 批二 E2: 非夺取式就绪探测（供 engine 的 CALL_SPEC 同步 drain 用）。
/// 与 check_async_http_result_handle 的 take 语义不同，本函数不消费条目——
/// 消费仍由 shim 的重入分支完成。PLAN-705 T-02:仅 Completed 态算就绪。
pub fn async_http_result_ready(request_id: u64) -> bool {
    crate::vm::ffi::async_http::live_op_ready(request_id)
}

/// PLAN-617 T-10: resolve a **relative** URL (leading `/`) against the base
/// address given by the `AUTO_HTTP_BASE` env (e.g. `http://127.0.0.1:8330`).
///
/// Rationale: a VM UI app and its `#[api]`/media backend live in *different
/// processes*, so the same `.at` source as the Vue build (`Http.get("/api/...")`
/// resolved by the page origin there) yields "relative URL without a base" in
/// reqwest — the app's library pane silently stayed empty (PLAN-617 §9.20).
/// With the base set, VM mode consumes the identical backend as the browser.
/// Absolute URLs and the env being unset are passed through unchanged.
/// PLAN-099 T-05: `pub(crate)`——iced 渲染器 `load_image_bytes` 的相对图片
/// URL（`/api/canvas/frame?...`）复用同一展开，双端同词汇。
pub fn resolve_http_base_url(url: &str) -> String {
    if !url.starts_with('/') {
        return url.to_string();
    }
    if let Ok(base) = std::env::var("AUTO_HTTP_BASE") {
        let base = base.trim().trim_end_matches('/').to_string();
        if !base.is_empty() {
            return format!("{}{}", base, url);
        }
    }
    if let Ok(port) = std::env::var("AUTO_HTTP_PORT") {
        let port = port.trim();
        if !port.is_empty() {
            return format!("http://127.0.0.1:{}{}", port, url);
        }
    }
    url.to_string()
}

/// PLAN-027(线程 churn 归零):api.* json 请求的发射路径。原为每请求
/// `std::thread::spawn`(双层线程时代 160 线程/s;PLAN-027 内层坍缩后
/// 仍 80/s)——L1 修复验证实测斜率 ∝ 线程创建数(2→1 线程,
/// 2.23→~1.3MB/min,量值见 evidence/027)。线程 churn 的终态=PLAN-705 T-03:
/// 固定 async executor(线程数恒定,零每请求线程),队满=终结性错误。
struct HttpJsonJob {
    method: String,
    url: String,
    body: Option<String>,
    req_id: u64,
}

/// PLAN-705 T-03: JSON 族的发射体改为固定 async executor 的 job——旧
/// 形态（2 常驻 worker + 容量 64 sync_channel + 队满每 job spawn 兜底
/// 线程）整体退役；队满现在=终结性错误（`{"error":"http client queue
/// full","status":0}`），零临时线程。
async fn run_http_json_job(job: HttpJsonJob) -> Result<AsyncResult, String> {
    let text = simple_http_json_async(
        job.method,
        &resolve_http_base_url(&job.url),
        job.body.as_deref(),
    )
    .await?;
    Ok(AsyncResult::Body(text))
}

/// Helper: submit an async JSON HTTP request to the fixed client executor.
/// Result is stored in the live-op table (PLAN-705 T-02) as the Body variant.
/// PLAN-027:dedicated thread 已由固定 async executor 取代(PLAN-705 T-03);
/// 队满=提交即终结性错误,无线程兜底。返回 false = 队满被终结性拒绝。
pub fn spawn_async_http(
    method: String,
    url: String,
    body: Option<String>,
    request_id: u64,
) -> bool {
    let job = HttpJsonJob {
        method,
        url,
        body,
        req_id: request_id,
    };
    crate::vm::ffi::async_http::submit_client_job(request_id, run_http_json_job(job))
}

// ============================================================================
// Plan 349 步骤 7/8 (W1): async helpers for handle-returning HTTP natives.
// ============================================================================

/// Check whether a previously spawned handle-returning async HTTP request has
/// completed. Returns `Some(Ok((status, headers, body)))` or `Some(Err(msg))`
/// on completion, `None` while still pending. The entry is `.take()`n so the
/// caller (re-entry branch) owns the data and inserts it into the calling
/// thread's HTTP_RESPONSES thread_local. Plan 349 步骤 7.
/// PLAN-705 T-02: take 只消费 Completed;Pending 保持 live。
pub(crate) fn check_async_http_result_handle(
    request_id: u64,
) -> Option<Result<(u16, Vec<(String, String)>, Vec<u8>), String>> {
    crate::vm::ffi::async_http::take_live_op(request_id).map(|r| {
        r.and_then(|ar| match ar {
            AsyncResult::Structured {
                status,
                headers,
                body,
            } => Ok((status, headers, body)),
            _ => Err("expected Structured async result".to_string()),
        })
    })
}

/// Decide whether a reqwest error is worth retrying. Connection failures,
/// timeouts, and certain status codes (5xx, 429 Too Many Requests) are
/// transient; 4xx client errors are not. Plan 349 步骤 7/8 (W3).
fn is_retryable_send_error(err: &reqwest::Error) -> bool {
    err.is_connect() || err.is_timeout() || err.is_request()
}

/// Run an async reqwest request builder to completion with optional retry.
/// On success returns `(status, headers, body_bytes)`; on terminal failure
/// returns `Err(message)`. `build_request` is invoked per attempt so multipart
/// forms (which are consumed by `.send()`) can be rebuilt each iteration.
/// Plan 349 步骤 7/8 (W3): retry with exponential backoff for transient errors.
/// PLAN-705 T-03: async 形态——退避 sleep 在 job 取消/超时时随 future 丢弃;
/// 响应体经预算化增量读。
pub(crate) async fn send_with_retry_async(
    mut build_request: impl FnMut(&reqwest::Client) -> reqwest::RequestBuilder,
    client: reqwest::Client,
    retry_count: u32,
) -> Result<(u16, Vec<(String, String)>, Vec<u8>), String> {
    let mut last_err: Option<String> = None;
    let max_attempts = retry_count.saturating_add(1); // retry_count=0 → 1 attempt
    for attempt in 0..max_attempts {
        let builder = build_request(&client);
        match builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                // Retry on transient HTTP status (5xx, 429) if attempts remain.
                let transient = (500..600).contains(&status) || status == 429;
                if transient && attempt < max_attempts - 1 {
                    last_err = Some(format!("HTTP {} (transient, retrying)", status));
                    backoff_sleep_async(attempt).await;
                    continue;
                }
                let headers: Vec<(String, String)> = response
                    .headers()
                    .iter()
                    .filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string())))
                    .collect();
                let body_bytes = crate::vm::ffi::async_http::read_body_capped(
                    response,
                    crate::vm::ffi::async_http::client_limits().body_limit,
                )
                .await?;
                return Ok((status, headers, body_bytes));
            }
            Err(e) => {
                let msg = e.to_string();
                if is_retryable_send_error(&e) && attempt < max_attempts - 1 {
                    last_err = Some(msg);
                    backoff_sleep_async(attempt).await;
                    continue;
                }
                return Err(msg);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| "retries exhausted".to_string()))
}

/// Exponential backoff helper: base 200ms * 2^attempt, capped at 5s.
/// Plan 349 步骤 7/8 (W3). PLAN-705 T-03: async sleep——job 被取消（future
/// drop）时退避随之丢弃，绝不继续发送。
async fn backoff_sleep_async(attempt: u32) {
    let base: u64 = 200;
    let delay_ms = base.saturating_mul(1u64 << attempt.min(8)).min(5000);
    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
}

/// Spawn a handle-returning async HTTP request (plain GET/POST/PUT/DELETE with
/// optional JSON body). Mirrors `simple_http_request`'s reqwest logic but runs
/// detached and writes the structured result into the live-op table (Structured).
/// Plan 349 步骤 7 (W1a).
// ─── Plan 446 E4: Http default header/query(认证面) ─────────────────────────
//
// musk VM 前端的 setupAuthFetch 依赖:auto.http get/post/put/delete/plain
// handle 族只收 url(无逐调用 header 面),builder 链踩 446-E2。本族 native
// 提供进程级默认头/默认 query,注入点 = 两个 send 汇聚处
// (spawn_async_http_handle 与 http.request builder 的 send_with_retry 闭包),
// 显式 header 同名覆盖默认。

pub(crate) struct HttpDefaults {
    pub(crate) headers: Vec<(String, String)>,
    pub(crate) queries: Vec<(String, String)>,
}

pub(crate) fn http_defaults() -> &'static std::sync::Mutex<HttpDefaults> {
    static REG: std::sync::OnceLock<std::sync::Mutex<HttpDefaults>> = std::sync::OnceLock::new();
    REG.get_or_init(|| {
        std::sync::Mutex::new(HttpDefaults {
            headers: Vec::new(),
            queries: Vec::new(),
        })
    })
}

pub(crate) fn http_set_default_entry(
    slot: &mut Vec<(String, String)>,
    name: String,
    value: String,
) {
    match slot.iter_mut().find(|(k, _)| *k == name) {
        Some(e) => e.1 = value,
        None => slot.push((name, value)),
    }
}

/// 把默认 query 追加到 url(已含 ? 则用 &)。值不做百分号编码——消费面
/// (workspace id/token 类)为安全字符集;特殊值由调用方自行编码后传入。
pub(crate) fn append_default_queries(url: &str) -> String {
    let qs = {
        let reg = http_defaults().lock().unwrap();
        reg.queries.clone()
    };
    if qs.is_empty() {
        return url.to_string();
    }
    let mut u = url.to_string();
    for (k, v) in qs {
        u.push(if u.contains('?') { '&' } else { '?' });
        u.push_str(&k);
        u.push('=');
        u.push_str(&v);
    }
    u
}

/// 默认 header 快照(发送线程内取一次,避免持锁跨 await/send)。
pub(crate) fn snapshot_default_headers() -> Vec<(String, String)> {
    http_defaults().lock().unwrap().headers.clone()
}

pub static WARNED_API_NOOP: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashSet<String>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashSet::new()));

/// PLAN-053 P-053-4: merged 模式下 #[api] 调用桩体的显式一次性告警
pub fn shim_warn_api_noop(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let name_nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, name_nv);
    let api_name = if auto_val::is_string(name_nv) {
        let idx = auto_val::decode_string(name_nv) as usize;
        vm.strings
            .read()
            .unwrap()
            .get(idx)
            .cloned()
            .map(|b| String::from_utf8_lossy(&b).to_string())
            .unwrap_or_default()
    } else {
        auto_val::decode_i32(name_nv).to_string()
    };
    // PLAN-734 T-03（D7）：merged 缺实现 = 可定位失败——不再 warn+null 假
    // 成功（AC-04）；行动指引保留在错误信息里。
    Err(VMError::RuntimeError(format!(
        "merged-mode #[api] \"{}\" has no implementation (set AUTO_VM_MERGE=0 or AUTO_BACKEND=<url> to enable the HTTP bridge)",
        api_name
    )))
}

pub(crate) fn spawn_async_http_handle(
    method: String,
    url: String,
    body: Option<String>,
    request_id: u64,
) {
    // PLAN-705 T-03: 固定 async executor job（原每请求 detached blocking
    // 线程退役）；迟到完成由 complete_live_op 的 presence 守卫丢弃。
    let job = async move {
        let client = shared_async_http_client().clone();
        let m = method.clone();
        let u = append_default_queries(&resolve_http_base_url(&url)); // Plan 446 E4: 默认 query 注入；T-10: 相对 URL 先按基址展开
        let b = body.clone();
        let default_headers = snapshot_default_headers(); // Plan 446 E4
        let result = send_with_retry_async(
            |c| {
                let builder = match m.as_str() {
                    "POST" => c.post(&u),
                    "PUT" => c.put(&u),
                    "DELETE" => c.delete(&u),
                    _ => c.get(&u),
                };
                let mut builder = builder;
                for (k, v) in &default_headers {
                    builder = builder.header(k.as_str(), v.as_str());
                }
                if let Some(ref body_str) = b {
                    builder
                        .header("Content-Type", "application/json")
                        .body(body_str.clone())
                } else {
                    builder
                }
            },
            client,
            0, // plain handle natives have no retry config; RequestBuilder path carries it
        )
        .await;
        result.map(|(status, headers, body)| AsyncResult::Structured {
            status,
            headers,
            body,
        })
    };
    let _ = crate::vm::ffi::async_http::submit_client_job(request_id, job);
}

// ─── PLAN-083 T-01: 异步 HTTP 消息桥（C2 形态） ─────────────────────────────
//
// 根因（PLAN-082 §10-7 实测 + 083 勘定）：store handler 内 #[api] 调用走
// get_json 族的 re-entry yield，而 ui 侧 call_fn_by_name 对 Waiting 任务是
// **忙等**（5ms sleep 轮询，30s deadline）——UI 线程整个 handler 期间被占
// （切 auto-edit 冻结 20.9s 即此）。本桥把数据加载拆两段：
//
//   发起段  Http.get_msg(url, event) —— 入队派生线程即返回，handler 立刻
//           结束，事件循环回 UI（无 task Waiting，无忙等）；
//   回填段  派生线程完成请求后把 {widget, event, payload} 推进进程级队列；
//           iced 侧 http_msg_subscription（AppTickKind::Poll，MCP 动作通道
//           同族）轮询取一条产 IcedMessage，经 update 通用路径
//           on_with_input_for 以 "Event␟s␟<payload>" 载荷触发回填 handler
//           （decode_payload 既有机制，字符串实参可靠——shell SSE 桥先例）。
//
// 载荷协议：payload 恒为 JSON 对象串 {"ok":bool,"status":u16,"body":str}
// （ok=true 时 body=响应体；否则 body=错误文案）——回填段 JSON.parse 一次
// 取全部语义，空体/失败可区分。

/// 一条已完成的异步消息桥请求（派生线程 → 渲染层订阅泵）。
pub struct HttpMsgDone {
    /// 目标 store 名（IcedMessage.widget；空 = 根 handler）。
    pub widget: String,
    /// 已编码回填事件串：`{handler}\u{1F}s\u{1F}{json}`（PLAN-765 T-02 起
    /// 入队侧单缓冲预编码——事件前缀与 JSON 转义体写进同一块按容量预留的
    /// 缓冲，渲染层泵零拷贝直投 IcedMessage.event）。JSON 形如
    /// `{"ok":bool,"status":u16,"body":str}`。
    pub event: String,
}

impl HttpMsgDone {
    /// 拆出 (handler 名, JSON 载荷)——首个 `\u{1F}` 段前 = handler 名。
    pub fn split_event(&self) -> (&str, &str) {
        match self.event.split_once('\u{1F}') {
            Some((name, rest)) => (name, rest.strip_prefix("s\u{1F}").unwrap_or(rest)),
            None => (self.event.as_str(), ""),
        }
    }
}

fn http_msg_queue() -> &'static std::sync::Mutex<std::collections::VecDeque<HttpMsgDone>> {
    static Q: std::sync::OnceLock<std::sync::Mutex<std::collections::VecDeque<HttpMsgDone>>> =
        std::sync::OnceLock::new();
    Q.get_or_init(|| std::sync::Mutex::new(std::collections::VecDeque::new()))
}

/// 渲染层泵取一条已完成的回填消息（非阻塞；一次 update 一条，防 handler
/// 重入——对齐 poll_shell_events 语义）。
pub fn http_msg_poll_one() -> Option<HttpMsgDone> {
    http_msg_queue().lock().ok().and_then(|mut q| q.pop_front())
}

/// 仅供测试：清空队列（生产面渲染层是唯一消费者）。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn http_msg_queue_clear() {
    if let Ok(mut q) = http_msg_queue().lock() {
        q.clear();
    }
}

/// 消息桥 GET：复用 plain-handle 族同款 send 汇聚路径（默认 query 注入 +
/// 相对 URL 基址展开 + 默认 header 快照），完成即推队列。PLAN-705 T-03:
/// 改走固定 async executor（原派生线程退役）；不写 live-op 表（无人在等
/// 它，submit 的 complete_live_op 落在缺席令牌上被守卫丢弃）。
pub(crate) fn spawn_async_http_msg_get(url: String, widget: String, event: String) {
    let job = async move {
        let u = append_default_queries(&resolve_http_base_url(&url));
        let default_headers = snapshot_default_headers();
        // 30s 超时对齐 get_json 族忙等 deadline——fire-and-forget 无消费侧
        // 兜底，连接悬挂必须由 client 侧收口（超时走 Err 臂→ok:false 入队，
        // poll_inflight 类消费门必然解锁）。
        // PLAN-765 T-02（纪要二 F-1）：共享连接池客户端——原每请求新建
        // Client 自带独立 hyper 池，2 拍/s 轮询负载下"新 TCP+池结构用后即弃"
        // 是空闲期内存残留的主分配源之一。
        let client = shared_async_http_client().clone();
        let mut builder = client.get(&u);
        for (k, v) in &default_headers {
            builder = builder.header(k.as_str(), v.as_str());
        }
        let result = match builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16();
                let body_bytes = crate::vm::ffi::async_http::read_body_capped(
                    response,
                    crate::vm::ffi::async_http::client_limits().body_limit,
                )
                .await
                .unwrap_or_default();
                (
                    (200..300).contains(&status),
                    status,
                    // PLAN-765 T-02：合法 UTF-8 时零拷贝接管（原
                    // from_utf8_lossy().to_string() 恒全文拷贝一次）。
                    match String::from_utf8(body_bytes) {
                        Ok(s) => s,
                        Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
                    },
                )
            }
            Err(e) => (false, 0u16, e.to_string()),
        };
        let (ok, status, body) = result;
        // PLAN-765 T-02（纪要二 F-3）：载荷单缓冲预编码——事件前缀 +
        // `␟s␟` + JSON 转义体写进同一块按容量预留的缓冲（原链 = body→
        // String→json! Value→to_string()→渲染层 format! 再拼，292KB 级
        // 响应全程 5-6 份全文拷贝）。借值序列化（body 以 &str 引用入场），
        // 唯一全文写 = 转义落笔；入队零拷贝移交渲染层泵。
        let mut buf: Vec<u8> = Vec::with_capacity(event.len() + 8 + body.len() + 64);
        buf.extend_from_slice(event.as_bytes());
        buf.push(b'\x1f');
        buf.push(b's');
        buf.push(b'\x1f');
        let payload_ref = HttpMsgPayloadRef {
            ok,
            status,
            body: &body,
        };
        let _ = serde_json::to_writer(&mut buf, &payload_ref);
        let event_full = String::from_utf8(buf)
            .unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned());
        if let Ok(mut q) = http_msg_queue().lock() {
            // PLAN-084 诊断打点（临时）→ PLAN-765 F-6：改 env 门控
            // （AUTOUI_HTTP_BRIDGE_DIAG=1）——常态 2 条/s 的 eprintln 既有
            // stderr 管道 churn，也与 080 在案的"管道关闭打印 panic"族耦合。
            if http_bridge_diag_enabled() {
                eprintln!(
                    "[msg-bridge] queued widget={widget} event={event} qlen={}",
                    q.len() + 1
                );
            }
            q.push_back(HttpMsgDone {
                widget,
                event: event_full,
            });
        }
        // fire-and-forget：无 live-op 令牌，返回值被 submit 的守卫丢弃。
        Ok(AsyncResult::Body(String::new()))
    };
    // PLAN-707 T-02：detached 显式提交形态——不写 live-op、不登记 abort，
    // `cancel_live_op` 对消息桥零影响（LIVE_OPS 缺席不构成误杀面）。
    let _ = crate::vm::ffi::async_http::submit_detached_client_job(job);
}

/// PLAN-765 T-02 F-3：消息桥载荷借值序列化形态（`{"ok","status","body"}`，
/// 与原 `serde_json::json!` 宏输出同构、字段序按结构体声明——消费侧
/// JSON.parse 按名取字段，序不参与契约）。
#[derive(serde::Serialize)]
struct HttpMsgPayloadRef<'a> {
    ok: bool,
    status: u16,
    body: &'a str,
}

/// PLAN-765 T-02 F-6：消息桥诊断打点总门（缺省关）。
fn http_bridge_diag_enabled() -> bool {
    static E: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *E.get_or_init(|| std::env::var("AUTOUI_HTTP_BRIDGE_DIAG").is_ok())
}

/// 消息桥目标切分：`"Store.Handler"` → (Store, Handler)；无点/空段 →
/// （"", 原串）即根 handler。首个点切分（handler 名 CamelCase 不含点）。
pub fn http_msg_split_target(event: &str) -> (String, String) {
    match event.split_once('.') {
        Some((w, e)) if !w.is_empty() && !e.is_empty() => (w.to_string(), e.to_string()),
        _ => (String::new(), event.to_string()),
    }
}

/// Push a response-handle result onto the stack: allocate a NET_HANDLE_COUNTER
/// id, insert the structured data into the *calling thread's* HTTP_RESPONSES
/// thread_local, push the handle as i32. Plan 349 步骤 7/8 (W1).
/// PLAN-776 R1-F01：本函数是 **client 面**（http.get 产句柄、`.status()`/
/// `.body()` 访问器经 engine CALL_SPEC 查表消费——FFI 资源范围，不在服务端
/// 返回编组边界内）；错误臂内联 500 构造（`shim_http_internal_error` 已迁
/// 凭据对象，不再共用）。
pub(crate) fn push_handle_result(
    task: &mut AutoTask,
    vm: &AutoVM,
    result: Result<(u16, Vec<(String, String)>, Vec<u8>), String>,
) -> Result<(), VMError> {
    // R2-05：client Response 在 HTTP egress 边界内（Plan §1.4）——producer
    // 产 **Response 凭据对象**（owned 快照），不再返回裸 int+表：marshal
    // 资源臂保留真实 status/body（此前降级 JSON 裸编号）；访问器统一
    // 载荷（快照/int 表）兼容；表 insert 退役（int 查表面成为死兼容）。
    let (status, headers, body) = match result {
        Ok((status, headers, body_bytes)) => (status, headers, body_bytes),
        Err(msg) => (
            500,
            vec![(
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            )],
            format!("HTTP failed: {}", msg).into_bytes(),
        ),
    };
    crate::vm::ffi::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Helper: push a response-body string onto the stack as a tagged string.
pub(crate) fn push_string_result(
    task: &mut AutoTask,
    vm: &AutoVM,
    s: String,
) -> Result<(), VMError> {
    let bytes = s.into_bytes();
    let str_idx = vm.add_string(bytes);
    vm.rc_push_str_idx(task, str_idx);
    Ok(())
}

thread_local! {
    pub(crate) static LAST_HTTP_STATUS: std::cell::Cell<i32> = std::cell::Cell::new(0);
}

/// Check whether a spawned auth-bearing async HTTP request has completed.
/// Returns `Some(Ok((status, body)))` / `Some(Err(msg))` on completion, `None`
/// while pending. Plan 349 步骤 7/8 (W1c). PLAN-705 T-02: take 只消费
/// Completed;Pending 保持 live。
pub(crate) fn check_async_http_result_auth(
    request_id: u64,
) -> Option<Result<(i32, String), String>> {
    crate::vm::ffi::async_http::take_live_op(request_id).map(|r| {
        r.and_then(|ar| match ar {
            AsyncResult::Auth { status, body } => Ok((status, body)),
            _ => Err("expected Auth async result".to_string()),
        })
    })
}

/// Spawn an auth-bearing async HTTP request (x-api-key style, for Anthropic).
/// Writes `(status, body)` into the live-op table (Auth variant) on completion.
/// Plan 349 步骤 7/8 (W1c).
pub(crate) fn spawn_async_http_auth(
    method: String,
    url: String,
    body: Option<String>,
    api_key: Option<String>,
    request_id: u64,
) {
    let url = resolve_http_base_url(&url); // PLAN-617 T-10: 相对 URL 按基址展开
    eprintln!(
        "[HTTP_REQ] {} url={} body_len={:?}",
        method,
        url,
        body.as_ref().map(|b| b.len())
    );
    // PLAN-705 T-03: 固定 async executor job（原每请求 detached 线程退役）。
    let job = async move {
        let client = shared_async_http_client().clone();
        let mut builder = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };
        builder = builder
            .header("Content-Type", "application/json")
            .header("anthropic-version", "2023-06-01");
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                builder = builder.header("x-api-key", key);
            }
        }
        if let Some(b) = body {
            builder = builder.body(b);
        }
        let result = match builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16() as i32;
                let body_bytes = crate::vm::ffi::async_http::read_body_capped(
                    response,
                    crate::vm::ffi::async_http::client_limits().body_limit,
                )
                .await
                .unwrap_or_default();
                let body_text = String::from_utf8_lossy(&body_bytes).to_string();
                eprintln!(
                    "[HTTP_RESP] {} url={} -> status={} body_len={}",
                    method,
                    url,
                    status,
                    body_text.len()
                );
                Ok((status, body_text))
            }
            Err(e) => {
                eprintln!("[HTTP_RESP_ERR] {} url={} -> err={}", method, url, e);
                Err(format!("HTTP error: {}", e))
            }
        };
        result.map(|(status, body)| AsyncResult::Auth { status, body })
    };
    let _ = crate::vm::ffi::async_http::submit_client_job(request_id, job);
}

/// Spawn an auth-bearing async HTTP request (Bearer style, for OpenAI).
/// Plan 349 步骤 7/8 (W1c).
pub(crate) fn spawn_async_http_bearer(
    method: String,
    url: String,
    body: Option<String>,
    api_key: Option<String>,
    request_id: u64,
) {
    let url = resolve_http_base_url(&url); // PLAN-617 T-10: 相对 URL 按基址展开
                                           // PLAN-705 T-03: 固定 async executor job。
    let job = async move {
        let client = shared_async_http_client().clone();
        let mut builder = match method.as_str() {
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => client.get(&url),
        };
        builder = builder.header("Content-Type", "application/json");
        if let Some(ref key) = api_key {
            if !key.is_empty() {
                builder = builder.header("Authorization", format!("Bearer {}", key));
            }
        }
        if let Some(b) = body {
            builder = builder.body(b);
        }
        let result = match builder.send().await {
            Ok(response) => {
                let status = response.status().as_u16() as i32;
                let body_bytes = crate::vm::ffi::async_http::read_body_capped(
                    response,
                    crate::vm::ffi::async_http::client_limits().body_limit,
                )
                .await
                .unwrap_or_default();
                Ok((status, String::from_utf8_lossy(&body_bytes).to_string()))
            }
            Err(e) => Err(format!("HTTP error: {}", e)),
        };
        result.map(|(status, body)| AsyncResult::Auth { status, body })
    };
    let _ = crate::vm::ffi::async_http::submit_client_job(request_id, job);
}

/// Push an auth result: set LAST_HTTP_STATUS and push the body String (or an
/// error message). Plan 349 步骤 7/8 (W1c).
pub(crate) fn push_auth_result(
    task: &mut AutoTask,
    vm: &AutoVM,
    result: Result<(i32, String), String>,
) -> Result<(), VMError> {
    let (status, body) = match result {
        Ok(sb) => sb,
        Err(msg) => (0, msg),
    };
    LAST_HTTP_STATUS.with(|s| s.set(status));
    crate::vm::ffi::convert::VMConvertible::push_to_stack(&body, task, vm)?;
    Ok(())
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_warn_noop_shims(natives: &mut crate::vm::native::NativeInterface) {
    // PLAN-053 P-053-4: merged 模式下 #[api] no-op 告警
    natives.register_shim_by_name("auto.vm.warn_api_noop", shim_warn_api_noop);
}
