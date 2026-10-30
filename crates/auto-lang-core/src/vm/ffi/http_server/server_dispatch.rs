//! PLAN-843 结构拆分：HTTP 服务 dispatch 职责（原样搬移）。
use super::*;

/// PLAN-698 T-02/SD-02: POST 成功后的 SSE 广播臂——生成 Axum 侧
/// void-POST+broadcast 路径（api_gen broadcast_event_name + events::broadcast）
/// 的 VM 运行面对照实现。仅当工程声明 ~Stream 端点时发布（has_sse 门控同
/// api_gen）。事件形态：
/// - fn 名含 "typing"（void 信号端点）→ `{"event":"Typing","name":<首个
///   str 形参的请求值>}`（api_gen 取首个 body 形参的同一约定）；
/// - 其余 POST（实体创建）→ 响应体（序列化实体）注入 `"event":"New{RetType}"`。
pub fn publish_post_broadcast(fn_name: &str, request_body: &str, response_json: &str) {
    if !has_stream_endpoint() {
        return;
    }
    if fn_name.to_lowercase().contains("typing") {
        let name_field = api_param_sigs(fn_name)
            .and_then(|sigs| {
                sigs.into_iter()
                    .find(|p| p.ty.contains("str"))
                    .map(|p| p.name)
            })
            .unwrap_or_else(|| "sender".to_string());
        let name = serde_json::from_str::<serde_json::Value>(request_body)
            .ok()
            .and_then(|v| v.get(&name_field).cloned())
            .unwrap_or(serde_json::Value::Null);
        crate::vm::ffi::stdlib::bus_broadcast(
            serde_json::json!({ "event": "Typing", "name": name }).to_string(),
        );
        return;
    }
    // 实体创建：响应体必须是 JSON 对象才注入事件名（流式/空响应跳过）。
    let ret = API_RETURN_TYPES
        .lock()
        .ok()
        .and_then(|t| t.get(fn_name).cloned());
    let Some(ret) = ret else { return };
    if ret.contains("Stream") || ret == "()" || ret.is_empty() {
        return;
    }
    let Ok(mut evt) = serde_json::from_str::<serde_json::Value>(response_json) else {
        return;
    };
    if !evt.is_object() {
        return;
    }
    evt["event"] = serde_json::Value::String(format!("New{}", ret));
    crate::vm::ffi::stdlib::bus_broadcast(evt.to_string());
}

pub fn serve_blocking_stdnet(vm: &crate::vm::engine::AutoVM, addr: &str) {
    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[HTTP] Server bind failed on {}: {}", addr, e);
            return;
        }
    };
    eprintln!("[HTTP] Server listening on {}", addr);

    let routes = get_routes();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[HTTP] Accept error: {}", e);
                continue;
            }
        };

        // Parse HTTP request
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let resp = format!(
                "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n{}\r\n",
                cors_headers()
            );
            let _ = stream.write_all(resp.as_bytes());
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Plan 349 步骤 7/8 (W5): CORS preflight short-circuit — respond before reading
        // the body since preflight requests have no body.
        if let Some(preflight) = handle_cors_preflight(&req_method) {
            let _ = stream.write_all(preflight.as_bytes());
            continue;
        }

        // Read headers
        let mut content_length = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).is_err() {
                break;
            }
            let header = header.trim();
            if header.is_empty() {
                break;
            }
            if header.to_lowercase().starts_with("content-length:") {
                content_length = header[15..].trim().parse().unwrap_or(0);
            }
        }

        // Read body
        let body = if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            let _ = (&mut reader).read_exact(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };

        // Route matching
        let route_match = match match_route(&routes, &req_method, &req_path) {
            Some(m) => m,
            None => {
                let resp = format!("HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n{}\r\nNot Found", cors_headers());
                let _ = stream.write_all(resp.as_bytes());
                continue;
            }
        };

        // Call VM handler
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) =
            vm.tasks.get(&handler_task_id)
        {
            let mut ht = handler_task_arc.blocking_lock();

            // PLAN-669: by-name binding when the fn's sigs are published;
            // legacy positional convention otherwise (see
            // bind_api_args_or_legacy — shared across all sync-serve sites).
            let bind_result: Result<usize, ApiArgBindError> = bind_api_args_or_legacy(
                vm,
                &mut ht,
                &route_match.fn_name,
                &route_match.path_params,
                &route_match.query_params,
                &body,
                &req_method,
                &req_path,
            );
            let n_args = match bind_result {
                Ok(n) => n,
                Err(e) => {
                    drop(ht);
                    vm.remove_task(handler_task_id);
                    let (status, msg) = match e {
                        ApiArgBindError::BadRequest(m) => ("400 Bad Request", m),
                        ApiArgBindError::Internal(m) => ("500 Internal Server Error", m),
                    };
                    eprintln!("[HTTP] {} {} → {} ({})", req_method, req_path, status, msg);
                    let err_body = format!("{{\"error\":{}}}", json_escape_string(&msg));
                    let resp = format!(
                        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                        status, err_body.len(), cors_headers(), err_body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    continue;
                }
            };

            // PLAN-705 T-05: legacy 同步驱动保留位——本 fn 是非默认调用图
            // 的串行 stdnet server（决策报告 §2 入口矩阵 legacy 行）；默认
            // Axum 调用图（serve_with → dispatch_api_request_segment）零
            // 同步忙等派发，由 plan705 gate 测试锁定。
            // PLAN-776 R2-01：HTTP dispatch 专用入口——generator 臂在返回
            // 边界建立凭据身份（marshal 资源臂按值分派）。
            match vm.call_fn_by_name_http(&mut ht, &route_match.fn_name, n_args) {
                Ok(()) => {
                    let nv = ht.ram.pop_nv();

                    // Plan 321 SSE: Check if the return value is an iterator ID
                    // (generator/~Stream<T>/~Iter<T> handler → SSE streaming mode).
                    if auto_val::is_i32(nv) {
                        let iter_id = auto_val::decode_i32(nv) as u32;
                        if vm.iterators.contains_key(&(iter_id)) {
                            // SSE streaming mode: write SSE headers, then pull
                            // values from the iterator as SSE data frames.
                            drop(ht);
                            let sse_response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n{}\r\n",
                                cors_headers()
                            );
                            let _ = stream.write_all(sse_response.as_bytes());
                            let _ = stream.flush();

                            // Pull values from the iterator and write SSE frames
                            loop {
                                // Create a temp task for the next() call
                                let next_task_id = vm.spawn_task(0, 1024);
                                let next_result = if let Some(nt_arc) = vm.tasks.get(&next_task_id)
                                {
                                    let mut nt = nt_arc.blocking_lock();
                                    // Push iterator_id for auto.iterator.next
                                    nt.ram.push_i32(iter_id as i32);
                                    // Call the native iterator.next
                                    crate::vm::native::shim_iterator_next(&mut nt, vm).ok();
                                    // Result is on stack (i32) or nothing (done)
                                    if nt.ram.sp > 0 {
                                        Some(nt.ram.pop_nv())
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };
                                vm.remove_task(next_task_id);

                                match next_result {
                                    Some(val) if auto_val::is_i32(val) => {
                                        let v = auto_val::decode_i32(val);
                                        if v == -1 {
                                            // Iterator exhausted
                                            break;
                                        }
                                        // Write SSE data frame
                                        let frame = format!("data: {}\n\n", v);
                                        let _ = stream.write_all(frame.as_bytes());
                                        let _ = stream.flush();
                                    }
                                    Some(val) if auto_val::is_string(val) => {
                                        let idx = auto_val::decode_string(val);
                                        let s = vm
                                            .strings
                                            .read()
                                            .unwrap()
                                            .get(idx as usize)
                                            .map(|b| String::from_utf8_lossy(b).to_string())
                                            .unwrap_or_default();
                                        let frame = format!("data: {}\n\n", s);
                                        let _ = stream.write_all(frame.as_bytes());
                                        let _ = stream.flush();
                                    }
                                    _ => break,
                                }
                            }
                            // Stream ended — close connection
                            continue; // Skip the normal JSON response below
                        }
                    }

                    // PLAN-729 T-04：legacy stdnet 不承载文件响应——声明
                    // 文件返回的路由在此入口明确 500 诊断（决策报告 §6：
                    // 未接入入口不冒充支持；描述符不序列化为 JSON）。
                    if fn_is_api_file_return(&route_match.fn_name) {
                        drop(ht);
                        let resp = format!(
                            "HTTP/1.1 500 Internal Server Error
Content-Type: application/json
Content-Length: {}
Connection: close
{}
{{\"error\":\"file responses require the default HTTP transport\"}}",
                            br#"{"error":"file responses require the default HTTP transport"}"#
                                .len(),
                            cors_headers()
                        );
                        let _ = stream.write_all(resp.as_bytes());
                        continue;
                    }
                    // Normal JSON response mode (Plan 326 Phase 3)
                    // nv_to_json handles string/i32/f64/bool/null, and recognizes
                    // heap object IDs (>= 4_000_000) to expand struct/array/Option
                    // return values into proper JSON instead of bare "null".
                    nv_to_json(vm, nv, 0)
                }
                Err(e) => {
                    eprintln!("[HTTP] Handler '{}' error: {:?}", route_match.fn_name, e);
                    None
                }
            }
        } else {
            None
        };

        vm.remove_task(handler_task_id);

        let (status, body_json) = match result_json {
            Some(s) => ("200 OK", s),
            None => ("500 Internal Server Error", "{}".to_string()),
        };
        // PLAN-698 SD-02: POST 广播臂（Typing / New{Type}，has_sse 门控）。
        if req_method == "POST" && status == "200 OK" {
            publish_post_broadcast(&route_match.fn_name, &body, &body_json);
        }
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
            status, body_json.len(), cors_headers(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

pub(super) fn register_service_shutdown_tx(tx: tokio::sync::watch::Sender<bool>) {
    if let Ok(mut slot) = SERVICE_SHUTDOWN_TX.lock() {
        *slot = Some(tx);
    }
}

pub(super) fn peer_is_loopback(peer: std::net::SocketAddr) -> bool {
    peer.ip().is_loopback()
}

pub(super) fn record_serve_fatal(msg: impl Into<String>) {
    if let Ok(mut slot) = SERVE_FATAL.lock() {
        *slot = Some(msg.into());
    }
}

/// serve 入口在 serve 返回后取致命错误（Some = 非零退出依据）。
pub fn take_serve_fatal_error() -> Option<String> {
    SERVE_FATAL.lock().ok().and_then(|mut s| s.take())
}

/// 只读快照（不消费）——run_file 内的钩子用它把 fatal 转成 Err 返回；
/// 外层 serve 入口随后 `take` 收割消息。take/snapshot 双面分离避免单次
/// take 被钩子吞掉后外层拿不到诊断。
pub fn serve_fatal_snapshot() -> Option<String> {
    SERVE_FATAL.lock().ok().and_then(|s| s.clone())
}

/// serve 入口在 ready 判定后取真实 bound 地址（port=0 场景必须用它）。
pub fn bound_addr_snapshot() -> Option<String> {
    SERVE_BOUND_ADDR.lock().ok().and_then(|s| s.clone())
}

/// 测试/serve 入口复位（进程内多 serve 序列时防陈旧值串场）。
pub fn reset_serve_state() {
    if let Ok(mut f) = SERVE_FATAL.lock() {
        *f = None;
    }
    if let Ok(mut b) = SERVE_BOUND_ADDR.lock() {
        *b = None;
    }
}

/// owner 出队派发：scope 失效（取消/过期）的排队请求**跳过业务函数**
/// 直接 503（AC-04 失效队列不执行）；有效则置 RUNNING 并段驱动。
pub fn dispatch_owner_request(
    vm: &std::rc::Rc<AutoVM>,
    routes: &[HttpRoute],
    api_req: ApiRequest,
    reply_tx: tokio::sync::oneshot::Sender<ApiReply>,
    scope_id: u64,
    parked: &mut Vec<ParkedRequest>,
) {
    let scope = lookup_scope(scope_id);
    let usable = scope.as_ref().map(|s| scope_usable(s)).unwrap_or(false);
    if !usable {
        // 失效队列：终结 scope（释放许可）+ 503，不调用 handler。
        if let Some(s) = &scope {
            cancel_scope(s);
        }
        let _ = reply_tx.send(ApiReply::Full {
            status: 503,
            headers: cors_json_headers(""),
            body: ApiBody::Text(br#"{"error":"request cancelled"}"#.to_vec()),
        });
        return;
    }
    if let Some(s) = &scope {
        s.state
            .store(SCOPE_RUNNING, std::sync::atomic::Ordering::SeqCst);
    }
    match dispatch_api_request_segment(vm, routes, api_req, reply_tx, scope_id) {
        DispatchOutcome::Replied => {}
        DispatchOutcome::Parked(p) => {
            if let Some(s) = lookup_scope(scope_id) {
                s.state
                    .store(SCOPE_PARKED, std::sync::atomic::Ordering::SeqCst);
            }
            parked.push(*p);
        }
    }
}

/// 关闭/退出时放弃全部 parked 请求：503 终态回复、任务清理、未完成的
/// live-op 回收（迟到的 worker 完成被 presence 守卫丢弃，无泄漏面）。
pub(super) fn cancel_all_parked(vm: &std::rc::Rc<AutoVM>, parked: &mut Vec<ParkedRequest>) {
    for mut p in parked.drain(..) {
        // PLAN-730 T-07：关闭废弃的 provisional 文件同样删除。
        cleanup_legacy_files(&p.ctx.legacy_stored);
        match &p.wait {
            ParkedWait::HttpRequest(req_id) => {
                crate::vm::ffi::stdlib::drop_async_result(*req_id);
            }
            // PLAN-707 T-05: 放弃 parked 流等待 → 取消上游流（收口）。
            ParkedWait::HttpStream(stream_id) => {
                crate::vm::ffi::http_stream::stream_cancel(*stream_id);
            }
            ParkedWait::Future(_) => {}
            // PLAN-711 T-11: CPU continuation 无 I/O 资源可取消——栈随任务
            // 弃置清账（HTTP 轨不产 CPU continuation，防御臂）。
            ParkedWait::CpuRunnable => {}
        }
        // T-05: scope 幂等终结（取消信号唤醒等它的桥臂；许可释放）。
        if let Some(s) = lookup_scope(p.scope_id) {
            cancel_scope(&s);
        }
        vm.remove_task(p.task_id);
        if let Some(tx) = p.reply_tx.take() {
            let _ = tx.send(ApiReply::Full {
                status: 503,
                headers: cors_json_headers(&p.ctx.request_id),
                body: ApiBody::Text(br#"{"error":"server shutting down"}"#.to_vec()),
            });
        }
    }
}

/// 编组延续所需的全部请求上下文（park 后恢复重建 reply 的最小集）。
pub struct DispatchCtx {
    pub req_method: String,
    pub req_path: String,
    pub request_id: String,
    /// 解码后的 body 字符串（multipart 已折叠为 JSON 时即其内容）。
    pub body: String,
    pub content_type: String,
    #[allow(dead_code)]
    pub content_type_raw: String,
    pub cookie_header: String,
    pub auth_header: String,
    pub multipart_json: Option<String>,
    pub route: RouteMatch,
    pub axum_route: Option<crate::vm::ffi::axum_adapter::AxumRoute>,
    pub started: std::time::Instant,
    pub middleware_names: Vec<String>,
    /// middleware request-info JSON（method/path/ct/has_body/request_id）。
    pub request_info: String,
    /// handler 任务 id（Handler / AwaitReturnFuture 阶段有效；终态移除）。
    pub handler_task_id: Option<u64>,
    /// 请求作用域 id（T-05：owner 派发/恢复前的失效检查）。
    pub scope_id: u64,
    /// PLAN-730 T-05：上传路由的延迟 body 能力（原始流 + 请求头快照）；
    /// start_handler 按 UploadRequest 参数注入消费（一次性）。
    pub upload_pending: Option<PendingUpload>,
    /// PLAN-730 T-07：legacy multipart 解析产物（纯内存，dispatch 段）；
    /// 落盘在 middleware 链后经宿主执行（ParkStage::LegacyStore）。
    pub legacy_parts: Option<Vec<MultipartPart>>,
    /// PLAN-730 T-07：已折叠的文本字段 JSON（落盘完成后与 files 合并）。
    pub legacy_fields_json: String,
    /// PLAN-730 T-07：本次请求新建的 provisional 文件（绑定失败/handler
    /// Err/取消 → 删除；成功回复按历史语义保留）。
    pub legacy_stored: Vec<String>,
}

/// PLAN-730 T-05：bridge→handler 的上传 body 能力（未授权不解析/不落盘）。
pub struct PendingUpload {
    pub body: a2r_std::http::UploadBodyStream,
    pub headers: Vec<(String, String)>,
}

/// park 的延续点。
#[derive(Debug, Clone, Copy)]
pub enum ParkStage {
    /// `middleware_names[index]` 的段挂起中。
    Middleware { index: usize },
    /// handler 段挂起中。
    Handler,
    /// handler 已 RET；等待 `~T` 返回 future（future id 在 `wait` 里）。
    AwaitReturnFuture,
    /// `~{}` 内部 future 体在 handler RET 后由服务端驱动，中途挂在外层
    /// External await 上（wait=外层 future id；internal_fid=内部 future）。
    AsyncReturnBody { internal_fid: u32 },
    /// PLAN-730 T-07：legacy multipart 落盘在宿主 spawn_blocking 执行中
    ///（middleware 已过；live-op 完成后合并 files JSON 进 body 再进 handler）。
    LegacyStore { op_id: u64 },
}

/// 一次跨 park 的请求延续（owner loop parked 表的条目）。
pub struct ParkedRequest {
    pub reply_tx: Option<tokio::sync::oneshot::Sender<ApiReply>>,
    /// 挂起中的任务 id（仍在 `vm.tasks` 注册表；单 owner try_lock 访问）。
    pub task_id: u64,
    /// 请求作用域 id（取消/deadline 失效检查——T-05）。
    pub scope_id: u64,
    pub seg: ParkedSegment,
    /// 就绪凭据（HttpRequest 的 live-op / Future 的 vm.futures）。
    pub wait: ParkedWait,
    pub stage: ParkStage,
    pub ctx: DispatchCtx,
}

/// 段状态机的推进核心：从 middleware `mw_start` 起跑完剩余链 + handler +
/// 编组；首 park 即返回 Parked（reply_tx 随条目保存）。
pub(super) fn advance_dispatch(
    vm: &std::rc::Rc<AutoVM>,
    mut ctx: DispatchCtx,
    mw_start: usize,
    mut reply_tx: Option<tokio::sync::oneshot::Sender<ApiReply>>,
) -> DispatchOutcome {
    // ── middleware 链（逐个段驱动；Err → 视为无响应继续，legacy 语义）──
    let mut index = mw_start;
    while index < ctx.middleware_names.len() {
        match run_middleware_at(vm, &ctx, index) {
            MWStep::Continue => index += 1,
            MWStep::Reply(reply) => {
                eprintln!(
                    "[HTTP] {} {} [{}] → MW ({}ms)",
                    ctx.req_method,
                    ctx.req_path,
                    ctx.request_id,
                    ctx.started.elapsed().as_millis()
                );
                if let Some(tx) = reply_tx.take() {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
            MWStep::Parked { task_id, seg, wait } => {
                return DispatchOutcome::Parked(Box::new(ParkedRequest {
                    reply_tx,
                    task_id,
                    scope_id: ctx.scope_id,
                    seg,
                    wait,
                    stage: ParkStage::Middleware { index },
                    ctx,
                }));
            }
        }
    }
    // PLAN-730 T-07：legacy multipart 落盘——middleware 全过后才执行（有界
    // 宿主 spawn_blocking + live-op 唤醒；owner 不做阻塞 std::fs::write）。
    if let Some(parts) = ctx.legacy_parts.take() {
        let files: Vec<(String, String, Vec<u8>)> = parts
            .iter()
            .filter_map(|p| {
                p.filename
                    .as_ref()
                    .map(|f| (p.name.clone(), f.clone(), p.data.clone()))
            })
            .collect();
        let op_id = crate::vm::ffi::stdlib::alloc_async_id();
        crate::vm::ffi::async_http::register_live_op(op_id);
        LEGACY_STORE_RESULTS.lock().unwrap().insert(op_id, None);
        tokio::task::spawn_blocking(move || {
            let result = legacy_store_files(&files).map(|stored| {
                (
                    stored
                        .iter()
                        .map(|f| f.path.clone())
                        .collect::<Vec<String>>(),
                    multipart_files_json(&stored),
                )
            });
            if let Ok(mut m) = LEGACY_STORE_RESULTS.lock() {
                m.insert(op_id, Some(result));
            }
            // 唤醒协议同 live-op：迟到（取消后）完成被 presence 守卫丢弃，
            // 结果条目随 abort 清理（consumed=Some 由 resume/abort 取走）。
            if !crate::vm::ffi::async_http::complete_live_op(
                op_id,
                Ok(crate::vm::ffi::stdlib::AsyncResult::Body(String::new())),
            ) {
                if let Ok(mut m) = LEGACY_STORE_RESULTS.lock() {
                    let _ = m.remove(&op_id);
                }
            }
        });
        return DispatchOutcome::Parked(Box::new(ParkedRequest {
            reply_tx,
            task_id: 0, // 无 VM 任务在执行（存储在宿主）
            scope_id: ctx.scope_id,
            seg: ParkedSegment {
                fn_name: ctx.route.fn_name.clone(),
                saved_bp: 0,
                saved_fn_n_args: 0,
            },
            wait: ParkedWait::HttpRequest(op_id),
            stage: ParkStage::LegacyStore { op_id },
            ctx,
        }));
    }
    start_handler(vm, ctx, reply_tx)
}

/// PLAN-730 T-07：落盘完成后的续跑（middleware 已在 park 前全过——从链尾
/// 直达 handler）。
pub(super) fn advance_after_legacy_store(
    vm: &std::rc::Rc<AutoVM>,
    p: &mut ParkedRequest,
) -> ParkedResume {
    let ctx = std::mem::replace(&mut p.ctx, placeholder_ctx());
    match start_handler(vm, ctx, p.reply_tx.take()) {
        DispatchOutcome::Replied => ParkedResume::Consumed,
        DispatchOutcome::Parked(boxed) => {
            // handler 段 park：延续条目（provisional 路径随 ctx 转移）。
            let mut b = *boxed;
            p.task_id = b.task_id;
            p.scope_id = b.scope_id;
            p.seg = b.seg.clone();
            p.wait = b.wait;
            p.stage = b.stage;
            p.ctx = b.ctx;
            ParkedResume::StillParked
        }
    }
}

/// PLAN-730 T-07：provisional 文件清理（绑定失败/handler Err/取消路径）。
pub(super) fn cleanup_legacy_files(paths: &[String]) {
    for path in paths {
        if let Err(e) = std::fs::remove_file(path) {
            if e.kind() != std::io::ErrorKind::NotFound {
                eprintln!("[HTTP] legacy provisional cleanup failed for {path}: {e}");
            }
        }
    }
}

pub(super) enum MWStep {
    Continue,
    Reply(ApiReply),
    Parked {
        task_id: u64,
        seg: ParkedSegment,
        wait: ParkedWait,
    },
}

/// 运行 middleware_names[index]（spawn 任务 + 段驱动）。上传路由的执行
/// 错误 fail-closed（500 短路、零落盘）——不沿用 legacy Err→Continue。
fn run_middleware_at(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx, index: usize) -> MWStep {
    let mw_fn = ctx.middleware_names[index].clone();
    let mw_task_id = vm.spawn_task(0, 65536);
    if let Some(t_arc) = vm.tasks.get(&mw_task_id) {
        if let Ok(mut t) = t_arc.try_lock() {
            push_str_arg(vm, &mut t, &ctx.request_info);
        }
    }
    enum MWEnd {
        Null,
        Response(String),
        Parked(ParkedWait, ParkedSegment),
    }
    let fail_closed = ctx.upload_pending.is_some();
    let end = {
        let Some(t_arc) = vm.tasks.get(&mw_task_id) else {
            vm.remove_task(mw_task_id);
            return MWStep::Continue; // legacy：任务槽缺失 = 无响应继续
        };
        let mut t = match t_arc.try_lock() {
            Ok(t) => t,
            Err(_) => {
                return MWStep::Continue; // legacy：锁忙 = break 静默跳过
            }
        };
        match vm.call_fn_by_name_segment(&mut t, &mw_fn, 1) {
            SegmentOutcome::Completed(Ok(())) => {
                let nv = t.ram.pop_nv();
                if auto_val::is_null(nv) {
                    MWEnd::Null
                } else {
                    match nv_to_json(vm, nv, 0) {
                        Some(resp) if !resp.is_empty() && resp != "null" => MWEnd::Response(resp),
                        _ => MWEnd::Null,
                    }
                }
            }
            // legacy：middleware Err → None → 继续链（行为保持）。上传路由
            // fail-closed：Err → 500（决策 §4.4——Err 静默放行不可接受）。
            SegmentOutcome::Completed(Err(_)) => {
                if fail_closed {
                    return MWStep::Reply(ApiReply::Full {
                        status: 500,
                        headers: cors_json_headers(&ctx.request_id),
                        body: ApiBody::Text(br#"{"error":"middleware failed"}"#.to_vec()),
                    });
                }
                MWEnd::Null
            }
            SegmentOutcome::Parked { wait, seg } => MWEnd::Parked(wait, seg),
            // PLAN-711 T-11: Runnable 仅由 CpuSlice 驱动产生；HTTP server
            // 走 legacy 段契约（非 UI 轨），防御臂按 Err 同族继续链。
            SegmentOutcome::Runnable { .. } => MWEnd::Null,
        }
    };
    match end {
        MWEnd::Parked(wait, seg) => MWStep::Parked {
            task_id: mw_task_id,
            seg,
            wait,
        },
        MWEnd::Null => {
            vm.remove_task(mw_task_id);
            MWStep::Continue
        }
        MWEnd::Response(resp) => {
            vm.remove_task(mw_task_id);
            let headers = cors_json_headers(&ctx.request_id);
            MWStep::Reply(ApiReply::Full {
                status: 200,
                headers,
                body: ApiBody::Text(resp.into_bytes()),
            })
        }
    }
}

pub(super) enum HandlerEnd {
    Value(auto_val::NanoValue),
    Err(crate::vm::engine::VMError),
    Parked(ParkedWait, ParkedSegment),
    /// 任务槽缺失（legacy contract：空体 200）。
    MissingTask,
    /// tokio Mutex try_lock 失败（守卫出作用域后再落 500 副作用）。
    LockBusy,
}

/// handler 阶段：spawn 任务 → 绑定 → 段驱动调用 → 编组/park。
fn start_handler(
    vm: &std::rc::Rc<AutoVM>,
    mut ctx: DispatchCtx,
    reply_tx: Option<tokio::sync::oneshot::Sender<ApiReply>>,
) -> DispatchOutcome {
    let handler_task_id = vm.spawn_task(0, 65536);
    ctx.handler_task_id = Some(handler_task_id);

    let axum_route = if ctx.route.fn_name.starts_with("__axum:") {
        match crate::vm::ffi::axum_adapter::route_by_synthetic_name(&ctx.route.fn_name) {
            Some(r) => Some(r),
            None => {
                eprintln!(
                    "[HTTP] {} {} → 500 (axum route lookup failed for {})",
                    ctx.req_method, ctx.req_path, ctx.route.fn_name
                );
                vm.remove_task(handler_task_id);
                let mut headers =
                    vec![("Content-Type".to_string(), "application/json".to_string())];
                headers.extend(cors_header_pairs());
                let reply = ApiReply::Full {
                    status: 500,
                    headers,
                    body: ApiBody::Text(br#"{"error":"route lookup failed"}"#.to_vec()),
                };
                if let Some(tx) = reply_tx {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
        }
    } else {
        None
    };
    ctx.axum_route = axum_route.clone();

    // PLAN-730 T-05：UploadRequest 注入——路由声明该参数类型时，把延迟
    // body 能力建成宿主 UploadRequest（一次性；scope 组绑定）。分类与
    // bridge 同源（方法+参数类型），不一致在 dispatch 段已 fail-closed。
    let upload_handle: Option<i64> = match ctx.upload_pending.take() {
        Some(pending) => {
            // Injection precedes the handler segment's scope guard. Bind now
            // so an unconsumed capability (early return or argument failure)
            // is reclaimed with this request instead of surviving in the table.
            let _scope_bind = bind_current_scope(ctx.scope_id);
            let req = a2r_std::http::upload_request_from_parts(
                &ctx.req_method,
                &ctx.req_path,
                pending.headers,
                pending.body,
            );
            Some(super::super::http_upload::insert_upload_request(req) as i64)
        }
        None => None,
    };

    let n_args = if let Some(ref axum_route) = axum_route {
        let query_json = if ctx.route.query_params.is_empty() {
            "{}".to_string()
        } else {
            let pairs: Vec<String> = ctx
                .route
                .query_params
                .iter()
                .map(|(k, v)| {
                    format!(
                        "\"{}\":\"{}\"",
                        k.replace('"', "\\\""),
                        v.replace('"', "\\\"")
                    )
                })
                .collect();
            format!("{{{}}}", pairs.join(","))
        };
        let mut pushed = 0usize;
        if let Some(t_arc) = vm.tasks.get(&handler_task_id) {
            if let Ok(mut t) = t_arc.try_lock() {
                let headers_json = if ctx.auth_header.is_empty() {
                    "{}".to_string()
                } else {
                    format!(
                        "{{\"authorization\":\"{}\"}}",
                        ctx.auth_header.replace('"', "")
                    )
                };
                pushed = crate::vm::ffi::axum_adapter::push_extractor_args(
                    vm,
                    &mut t,
                    axum_route,
                    &ctx.route.path_params,
                    &query_json,
                    &ctx.body,
                    &headers_json,
                );
            }
        }
        pushed
    } else {
        match build_handler_args(
            vm,
            handler_task_id,
            &ctx.route,
            &ctx.body,
            &ctx.content_type,
            &ctx.cookie_header,
            &ctx.auth_header,
            ctx.multipart_json.as_deref(),
            &ctx.req_method,
            &ctx.req_path,
            upload_handle,
        ) {
            Ok(n) => n,
            Err(ApiArgBindError::BadRequest(msg)) => {
                eprintln!("[HTTP] {} {} → 400 ({})", ctx.req_method, ctx.req_path, msg);
                vm.remove_task(handler_task_id);
                // PLAN-730 T-07：绑定失败——本次新建 provisional 文件删除。
                cleanup_legacy_files(&ctx.legacy_stored);
                let err_body = format!("{{\"error\":{}}}", json_escape_string(&msg));
                let mut headers =
                    vec![("Content-Type".to_string(), "application/json".to_string())];
                headers.extend(cors_header_pairs());
                let reply = ApiReply::Full {
                    status: 400,
                    headers,
                    body: ApiBody::Text(err_body.into_bytes()),
                };
                if let Some(tx) = reply_tx {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
            Err(ApiArgBindError::Internal(msg)) => {
                eprintln!("[HTTP] {} {} → 500 ({})", ctx.req_method, ctx.req_path, msg);
                vm.remove_task(handler_task_id);
                cleanup_legacy_files(&ctx.legacy_stored);
                let err_body = format!("{{\"error\":{}}}", json_escape_string(&msg));
                let mut headers =
                    vec![("Content-Type".to_string(), "application/json".to_string())];
                headers.extend(cors_header_pairs());
                let reply = ApiReply::Full {
                    status: 500,
                    headers,
                    body: ApiBody::Text(err_body.into_bytes()),
                };
                if let Some(tx) = reply_tx {
                    let _ = tx.send(reply);
                }
                return DispatchOutcome::Replied;
            }
        }
    };

    // DashMap 读守卫纪律：守卫作用域内不得对同一 shard remove——失败形态
    // 编码为变体，守卫出作用域后再落副作用。
    let end = if let Some(t_arc) = vm.tasks.get(&handler_task_id) {
        match t_arc.try_lock() {
            Err(_) => HandlerEnd::LockBusy,
            Ok(mut ht) => {
                // Dispatch: axum routes use closure (Plan 383 fn-ref), legacy
                // use fn-name. PLAN-705: 段驱动形态——等待即 park，不再同步
                // 占线程。PLAN-707 T-06: 段执行期间绑定资源组（流 open 登记）。
                let _scope_bind = bind_current_scope(ctx.scope_id);
                match if let Some(ref axum_route) = axum_route {
                    vm.call_closure_segment(&mut ht, axum_route.closure_id, n_args)
                } else {
                    vm.call_fn_by_name_segment_http(&mut ht, &ctx.route.fn_name, n_args)
                } {
                    SegmentOutcome::Completed(Ok(())) => HandlerEnd::Value(ht.ram.pop_nv()),
                    SegmentOutcome::Completed(Err(e)) => HandlerEnd::Err(e),
                    SegmentOutcome::Parked { wait, seg } => HandlerEnd::Parked(wait, seg),
                    // PLAN-711 T-11: Runnable 仅由 CpuSlice 驱动产生；HTTP
                    // server 走 legacy 段契约（非 UI 轨），防御臂按 Err 同族。
                    SegmentOutcome::Runnable { .. } => {
                        HandlerEnd::Err(crate::vm::engine::VMError::RuntimeError(
                            "internal: cpu-slice outcome escaped legacy http dispatch (unreachable)"
                                .into(),
                        ))
                    }
                }
            }
        }
    } else {
        HandlerEnd::MissingTask
    };

    match end {
        HandlerEnd::MissingTask => {
            // legacy contract：任务槽缺失 → 空体 200。
            let reply = ApiReply::Full {
                status: 200,
                headers: cors_json_headers(&ctx.request_id),
                body: ApiBody::Text(Vec::new()),
            };
            if let Some(tx) = reply_tx {
                let _ = tx.send(reply);
            }
            return DispatchOutcome::Replied;
        }
        HandlerEnd::LockBusy => {
            eprintln!(
                "[HTTP] {} {} → 500 (task lock failed, {}ms)",
                ctx.req_method,
                ctx.req_path,
                ctx.started.elapsed().as_millis()
            );
            vm.remove_task(handler_task_id);
            let reply = ApiReply::Full {
                status: 500,
                headers: cors_json_headers(&ctx.request_id),
                body: ApiBody::Text(br#"{"error":"internal error"}"#.to_vec()),
            };
            if let Some(tx) = reply_tx {
                let _ = tx.send(reply);
            }
            return DispatchOutcome::Replied;
        }
        HandlerEnd::Parked(wait, seg) => DispatchOutcome::Parked(Box::new(ParkedRequest {
            reply_tx,
            task_id: handler_task_id,
            scope_id: ctx.scope_id,
            seg,
            wait,
            stage: ParkStage::Handler,
            ctx,
        })),
        HandlerEnd::Err(e) => {
            vm.remove_task(handler_task_id);
            // PLAN-730 T-07：handler 异常——本次新建 provisional 文件删除。
            cleanup_legacy_files(&ctx.legacy_stored);
            let reply = handler_error_reply(&ctx, &e);
            // R04 修复轮：owner 侧幂等收口——不依赖桥/运行时在收件后存
            // 活（shutdown 排水与 owner 退出/运行时拆除竞速时桥侧
            // complete_scope 不再运行，scope 泄漏在全局表）；Err 回复恒
            // Full 非 SSE/File，桥侧后续 complete 幂等无害。
            let _ = reply_tx.map(|tx| tx.send(reply));
            if let Some(s) = lookup_scope(ctx.scope_id) {
                cancel_scope(&s);
            }
            DispatchOutcome::Replied
        }
        HandlerEnd::Value(nv) => {
            match marshal_handler_value(vm, &ctx, nv) {
                MarshalOutcome::Reply(reply) => {
                    // PLAN-776 T-05：编组产物 owned（快照 clone / 表 take /
                    // iterator_id int），返回值栈槽的 RC 在此释放——凭据
                    // 随引用归零回收（1000 闭环回基线的前提；此前 pop_nv
                    // 只读不释放，remove_task 亦不清栈，堆对象恒泄漏 1 引用）。
                    // R1-F02：只释放本槽实际持有的份额（TAG_OBJECT 栈槽）；
                    // 普通数值槽（含 ≥4M 大数——rc::heap_ref_id 的历史 i32
                    // 启发把其当裸 heap id）不持份额，不得按内容扣减无关
                    // 资源对象的 RC（同号普通 int 误杀全局资源反例）。
                    if auto_val::is_object(nv) {
                        vm.rc_release(nv);
                    }
                    vm.remove_task(handler_task_id);
                    if let Some(tx) = reply_tx {
                        let _ = tx.send(reply);
                    }
                    DispatchOutcome::Replied
                }
                MarshalOutcome::ParkFuture(fid) => {
                    // ~T 返回 future 未完成：handler 任务保留（终值编组需要
                    // 栈暂存），stage 切 AwaitReturnFuture，wait=Future(fid)
                    // 统一就绪探测。
                    DispatchOutcome::Parked(Box::new(ParkedRequest {
                        reply_tx,
                        task_id: handler_task_id,
                        scope_id: ctx.scope_id,
                        seg: ParkedSegment {
                            fn_name: ctx.route.fn_name.clone(),
                            saved_bp: 0,
                            saved_fn_n_args: 0,
                        },
                        wait: ParkedWait::Future(fid),
                        stage: ParkStage::AwaitReturnFuture,
                        ctx,
                    }))
                }
                MarshalOutcome::ParkBody(internal_fid, outer_fid) => {
                    DispatchOutcome::Parked(Box::new(ParkedRequest {
                        reply_tx,
                        task_id: handler_task_id,
                        scope_id: ctx.scope_id,
                        seg: ParkedSegment {
                            fn_name: ctx.route.fn_name.clone(),
                            saved_bp: 0,
                            saved_fn_n_args: 0,
                        },
                        wait: ParkedWait::Future(outer_fid),
                        stage: ParkStage::AsyncReturnBody { internal_fid },
                        ctx,
                    }))
                }
            }
        }
    }
}

/// 编组 handler 返回值：SSE / Response handle / ~T 最终值 / JSON。
/// ~T 判定为元数据门（fn_is_api_async，`__axum:` 反查导出名）+ future
/// bits 形态 + vm.futures 注册表存在性三重闸——普通 int 位模式永不误判
/// （AC-01 反例）。
pub(super) enum MarshalOutcome {
    Reply(ApiReply),
    /// External 返回 future 未完成。
    ParkFuture(u32),
    /// Internal 体挂在外层 External await 上（internal/outer future id）。
    ParkBody(u32, u32),
}

/// park 时任务的外层等待 future id（体挂起时 waiting_future_id 即外层）。
fn ctx_await_outer(vm: &std::rc::Rc<AutoVM>, ctx: &DispatchCtx) -> Option<u32> {
    let task_id = ctx.handler_task_id?;
    let t_arc = vm.tasks.get(&task_id)?.clone();
    let t = t_arc.try_lock().ok()?;
    t.waiting_future_id
}

/// PLAN-776 R7-01：Future 驱动——TAG_FUTURE **逐值来源**臂与 legacy
/// `0xF0` 位形态臂共用。`None` = future id 查无注册表（调用方落后续
/// 数据面/受控失败——位模式撞值不受影响）。
fn drive_future_reply(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    fid: u32,
) -> Option<MarshalOutcome> {
    let fut = vm.futures.get(&fid)?;
    let (pending, is_external, body_offset) = {
        let f = fut.read().unwrap();
        (
            f.state == crate::vm::engine::FutureState::Pending,
            f.kind == crate::vm::engine::FutureKind::External,
            f.body_offset,
        )
    };
    if pending {
        if is_external {
            return Some(MarshalOutcome::ParkFuture(fid));
        }
        // Internal `~{}`：体由 await 上下文驱动——handler 已 RET，服务端
        // 就地驱动（handle_await_future）；体内部再挂 External await 时转
        // AsyncReturnBody 阶段，由完成通知唤醒后续跑体。
        // PLAN-776 R7-03：task 锁被并发持有（如前序流断连清理）时**不得
        // 静默跳过驱动**（残留 Pending → 错误终态路径）——有界重试后仍
        // 失败才放弃（后续 ParkBody/终态判定收口）。
        let task_id = ctx.handler_task_id;
        if let Some(task_id) = task_id {
            if let Some(t_arc) = vm.tasks.get(&task_id) {
                let mut driven = false;
                for _ in 0..100 {
                    if let Ok(mut t) = t_arc.try_lock() {
                        let _ = vm.handle_await_future(&mut t, fid, body_offset);
                        driven = true;
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(2));
                }
                let _ = driven;
            }
        }
        let fut2 = vm.futures.get(&fid)?;
        {
            let f = fut2.read().unwrap();
            if f.state == crate::vm::engine::FutureState::Ready {
                let value = f.result.clone();
                drop(f);
                return Some(MarshalOutcome::Reply(final_value_reply(vm, ctx, value)));
            }
            if f.state == crate::vm::engine::FutureState::Failed {
                drop(f);
                return Some(MarshalOutcome::Reply(final_value_reply(vm, ctx, None)));
            }
        }
        // 体仍挂在外层 await 上 → AsyncReturnBody 阶段。
        if let Some(outer_fid) = ctx_await_outer(vm, ctx) {
            return Some(MarshalOutcome::ParkBody(fid, outer_fid));
        }
        return Some(MarshalOutcome::Reply(final_value_reply(vm, ctx, None)));
    }
    let value = fut.read().unwrap().result.clone();
    Some(MarshalOutcome::Reply(final_value_reply(vm, ctx, value)))
}

pub(super) fn marshal_handler_value(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    nv: auto_val::NanoValue,
) -> MarshalOutcome {
    // PLAN-776 T-03：真实资源凭据臂（TAG_OBJECT → 本 VM 堆 → downcast）。
    // 优先于一切 int 探测——普通 int 永不命中；跨 owner/过期凭据落
    // stale 诊断（不误取其他表项）。legacy int 面走下方原路径。
    if let Some(claim) = super::super::http_return_resource::resolve_resource(vm, nv) {
        return MarshalOutcome::Reply(resource_claim_reply(vm, ctx, claim));
    }
    // PLAN-776 R7-01：Future 的**逐值来源**门——TAG_FUTURE 载荷（全部生产
    // 点：CREATE_FUTURE/External creator 产此形态）即来源证明，**无函数级
    // 门**（声明/位模式/注册表都不是来源——R5/R6/R7 函数级登记反例）。
    // 普通 i32（含撞活跃 fid 位形态的 496）永不进入此臂。
    if auto_val::is_future(nv) {
        return drive_future_reply(vm, ctx, auto_val::decode_future(nv))
            .unwrap_or_else(|| MarshalOutcome::Reply(final_value_reply(vm, ctx, None)));
    }
    if auto_val::is_i32(nv) {
        let bits = auto_val::decode_i32(nv);
        // PLAN-776 R8-01：legacy `0xF0` 位形态**驱动臂退役**——Value 载体
        // 已保真映射 Future（R7 的降级 Int 废弃），任何位形态 i32 都是
        // 普通数字（~T 声明 + 位模式 + 活跃 fid 查表不能证明来源——
        // F776-R8-01 F2：`~int` fn 返回 496 恒数据）。Future 驱动只经上方
        // TAG_FUTURE 臂。
        // R1-F01（F776-R1-01 修复）：普通数值不再通过任何表臂取得或消费
        // 资源——upload/file 的"声明 + int + 表命中"入口与 iterator/
        // HTTP_RESPONSES 查表命中制全部退役（P734-D7 查表猜测被本计划显式
        // 替代：真实 producer 全部产凭据对象，对象臂按值分派）。唯一保留的
        // int→SSE 形态是**声明门**：generator handler 声明 `~Iter<T>`（fn 调用
        // 返回 iterator id 的语言形态）——未声明流返回的普通 int（含活跃
        // iterator 同号碰撞）一律 JSON 数据。
        // R1-F01 恢复 729/730 契约诊断门（无表消费）：声明 File/Upload 专属
        // 返回但值非凭据对象（上方对象臂未命中）→ mismatch 500——普通数值
        // 不取得/不消费资源（未声明专属返回的普通 int 仍永远 JSON 数据）。
        if handler_declares_upload_return(vm, ctx) {
            return MarshalOutcome::Reply(upload_contract_mismatch_reply(ctx, bits as u32 as u64));
        }
        if handler_declares_file_return(vm, ctx) {
            return MarshalOutcome::Reply(file_contract_mismatch_reply(ctx, bits as u32 as u64));
        }
        // R2-01/R5-01（终形）：普通 int 永不取得 iterator 身份——generator 的
        // 真实产出在**生产点**已是凭据对象（codegen 对 ~Iter/~Stream 声明 fn
        // 内 CREATE_GENERATOR 后插 WRAP_SSE_RESOURCE；generator handler 的
        // dispatch 臂直推），走上方对象臂；声明/表命中不授予身份。
        return MarshalOutcome::Reply(json_value_reply(
            vm,
            nv,
            &ctx.req_method,
            &ctx.req_path,
            &ctx.route.fn_name,
            &ctx.body,
            &ctx.request_id,
            ctx.started.elapsed().as_millis(),
        ));
    }
    if auto_val::is_i64(nv) {
        // R1-F01：i64 查表命中制退役——普通 i64 是数据（client 句柄返回
        // 面按 JSON 数字编组；真实 Response 走对象凭据臂）。
        return MarshalOutcome::Reply(json_value_reply(
            vm,
            nv,
            &ctx.req_method,
            &ctx.req_path,
            &ctx.route.fn_name,
            &ctx.body,
            &ctx.request_id,
            ctx.started.elapsed().as_millis(),
        ));
    }
    MarshalOutcome::Reply(json_value_reply(
        vm,
        nv,
        &ctx.req_method,
        &ctx.req_path,
        &ctx.route.fn_name,
        &ctx.body,
        &ctx.request_id,
        ctx.started.elapsed().as_millis(),
    ))
}

/// Response-object 分支的共享编组（status/headers/body + CORS + 请求 id）。
pub(super) fn response_handle_reply(
    ctx: &DispatchCtx,
    res: (u16, Vec<(String, String)>, Vec<u8>),
) -> ApiReply {
    let (status, headers, body) = res;
    let mut all_headers = headers;
    all_headers.extend(cors_header_pairs());
    all_headers.push(("X-Request-Id".to_string(), ctx.request_id.clone()));
    eprintln!(
        "[HTTP] {} {} [{}] → {} ({}ms)",
        ctx.req_method,
        ctx.req_path,
        ctx.request_id,
        status,
        ctx.started.elapsed().as_millis()
    );
    ApiReply::Full {
        status,
        headers: all_headers,
        body: ApiBody::Text(body),
    }
}

/// handler Err 的 500 形态（legacy 同款）。
pub(super) fn handler_error_reply(ctx: &DispatchCtx, e: &crate::vm::engine::VMError) -> ApiReply {
    // PLAN-781 T-22（R04 修复轮）：协作式取消（engine 驱动环的 scope/
    // 排水截止探测中止段）映射为 503 取消终态——与 parked 取消
    // （abort_parked_request/cancel_all_parked）同一客户端可见形态；
    // 前缀约定与 engine 侧 "request cancelled in '..." 错误成对（同计
    // 划内生产/消费，预算耗尽仍走 500 指名资源拒绝）。
    if matches!(e, crate::vm::engine::VMError::RuntimeError(ref m) if m.starts_with("request cancelled in "))
    {
        eprintln!(
            "[HTTP] {} {} [{}] → 503 (cooperative cancel: scope terminal/deadline/drain)",
            ctx.req_method, ctx.req_path, ctx.request_id
        );
        return ApiReply::Full {
            status: 503,
            headers: cors_json_headers(&ctx.request_id),
            body: ApiBody::Text(br#"{"error":"request cancelled"}"#.to_vec()),
        };
    }
    eprintln!(
        "[HTTP] {} {} → 500 (handler '{}' error: {:?}, {}ms)",
        ctx.req_method,
        ctx.req_path,
        ctx.route.fn_name,
        e,
        ctx.started.elapsed().as_millis()
    );
    let error_json = format!(
        r#"{{"error":"internal server error","detail":"{}"}}"#,
        format!("{:?}", e).replace('"', "\\\"").replace('\n', " ")
    );
    ApiReply::Full {
        status: 500,
        headers: cors_json_headers(&ctx.request_id),
        body: ApiBody::Text(error_json.into_bytes()),
    }
}

/// ~T 最终值编组：FutureValue.result → NanoValue → json_value_reply。
/// PLAN-776 T-03：资源终值与 sync 同一 `resource_claim_reply` 分派。
/// PLAN-776 T-04：Failed / result 缺失 / 值不可转换 → 有界 500（不再
/// 200 null）；显式成功 Nil（Some(Nil)）→ 200 null 保持。
pub(super) fn final_value_reply(
    vm: &std::rc::Rc<AutoVM>,
    ctx: &DispatchCtx,
    value: Option<auto_val::Value>,
) -> ApiReply {
    if let Some(v) = &value {
        if let Some(claim) = super::super::http_return_resource::resolve_resource_value(vm, v) {
            return resource_claim_reply(vm, ctx, claim);
        }
    }
    let Some(v) = value else {
        return async_failure_reply(ctx, "future failed or result missing");
    };
    // R1-F01：upload/file 的"声明 + Value::Int + 表 take"门退役——普通
    // 数值不得通过任何表臂消费资源；upload_commit/reject 与
    // http.file_response 已产凭据对象（上方资源臂拦截）。恢复 729/730
    // 契约**诊断**门（无表消费）：声明专属返回但终值非凭据对象 →
    // mismatch 500（与 sync 同形；契约违反不 JSON 200 伪装成功）。
    if handler_declares_upload_return(vm, ctx) {
        let raw = match &v {
            auto_val::Value::Int(i) => *i as u32 as u64,
            _ => u64::MAX,
        };
        return upload_contract_mismatch_reply(ctx, raw);
    }
    if handler_declares_file_return(vm, ctx) {
        let raw = match &v {
            auto_val::Value::Int(i) => *i as u64,
            _ => u64::MAX,
        };
        return file_contract_mismatch_reply(ctx, raw);
    }
    let mut materialized = Vec::new();
    let Some(nv) = value_to_nv_via_task(vm, ctx, v, &mut materialized) else {
        return async_failure_reply(ctx, "terminal value has no JSON wire form");
    };
    let reply = json_value_reply(
        vm,
        nv,
        &ctx.req_method,
        &ctx.req_path,
        &ctx.route.fn_name,
        &ctx.body,
        &ctx.request_id,
        ctx.started.elapsed().as_millis(),
    );
    // PLAN-776 T-04：物化容器即用即弃（编组完成，无后续引用——计数归零
    // 走 dying-grace 真回收，1000 次闭环存活回基线的前提）。
    for id in materialized {
        vm.rc_retain_id(id);
        vm.rc_release_id(id);
    }
    reply
}

/// PLAN-776 T-04：async 终态失败 → 有界 500（Failed / result 缺失 / 值不可
/// 转换；不再伪装 200 null——显式成功 Nil 才是 200 null）。
fn async_failure_reply(ctx: &DispatchCtx, detail: &str) -> ApiReply {
    eprintln!(
        "[HTTP] {} {} [{}] → 500 (async terminal state: {})",
        ctx.req_method, ctx.req_path, ctx.request_id, detail
    );
    let body = format!(
        r#"{{"error":"async handler failed","detail":"{}"}}"#,
        detail.replace('"', "\\\"").replace('\n', " ")
    );
    ApiReply::Full {
        status: 500,
        headers: cors_json_headers(&ctx.request_id),
        body: ApiBody::Text(body.into_bytes()),
    }
}
