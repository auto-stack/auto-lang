//! Explicit-host legacy serial stdnet grade: CORS, JSON and iterator SSE.
//! File replies remain an explicit unsupported 500; no default transport alias.
use super::*;
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::host_capabilities::request_context::{RequestContext, RequestResourceKind};

#[derive(Default)]
struct LegacyResources(std::sync::Mutex<(bool, Vec<(RequestResourceKind, u64)>)>);
impl RequestContext for LegacyResources {
    fn register_resource(&self, kind: RequestResourceKind, id: u64) {
        let mut state = self.0.lock().unwrap();
        if !state.0 {
            state.1.push((kind, id));
            return;
        }
        drop(state);
        VmHttpBridge::finalize_resources(kind, &[id]);
    }
    fn cancelled(&self) -> bool {
        self.0.lock().unwrap().0
    }
    // This legacy serial grade never advertised a request deadline.
    fn extend_deadline(&self, _deadline: std::time::Instant) {}
}
impl LegacyResources {
    fn finish(&self) {
        let mut state = self.0.lock().unwrap();
        state.0 = true;
        let ids = std::mem::take(&mut state.1);
        drop(state);
        for (kind, id) in ids {
            VmHttpBridge::finalize_resources(kind, &[id]);
        }
    }
}
struct LegacyTaskCleanup<'a> {
    vm: &'a AutoVM,
    task: u64,
}
impl Drop for LegacyTaskCleanup<'_> {
    fn drop(&mut self) {
        self.vm.remove_task(self.task);
    }
}
struct LegacyRequestCleanup<'a> {
    vm: &'a AutoVM,
    task: u64,
    resources: std::sync::Arc<LegacyResources>,
}
impl Drop for LegacyRequestCleanup<'_> {
    fn drop(&mut self) {
        self.vm.remove_task(self.task);
        self.resources.finish();
    }
}
struct LegacyIteratorCleanup<'a> {
    vm: &'a AutoVM,
    iterator: u32,
}
impl Drop for LegacyIteratorCleanup<'_> {
    fn drop(&mut self) {
        cleanup_sse_iterator(self.vm, self.iterator);
    }
}

pub fn serve_legacy_stdnet(
    vm: &AutoVM,
    addr: &str,
    metadata: std::sync::Arc<HttpProgramMetadata>,
    stop: Option<&std::sync::atomic::AtomicBool>,
) -> Result<(), VMError> {
    with_owner_metadata_sync(metadata, || serve_inner(vm, addr, stop))
}
fn serve_inner(
    vm: &AutoVM,
    addr: &str,
    stop: Option<&std::sync::atomic::AtomicBool>,
) -> Result<(), VMError> {
    let listener = TcpListener::bind(addr).map_err(|error| {
        crate::vm::engine::VMError::RuntimeError(format!("HTTP server bind failed: {error}"))
    })?;
    eprintln!("[HTTP] Server listening on {}", addr);

    let routes = get_routes();

    for stream in listener.incoming() {
        if stop.is_some_and(|stop| stop.load(std::sync::atomic::Ordering::Acquire)) {
            break;
        }
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
        let resources = std::sync::Arc::new(LegacyResources::default());
        let _cleanup = LegacyRequestCleanup {
            vm,
            task: handler_task_id,
            resources: resources.clone(),
        };
        let _context =
            crate::vm::host_capabilities::request_context::bind_request_context(Some(resources));
        let result_json: Option<String> = if let Some(handler_task_arc) = vm
            .tasks
            .get(&handler_task_id)
            .map(|task| task.value().clone())
        {
            let mut ht = handler_task_arc.try_lock().map_err(|_| {
                crate::vm::engine::VMError::RuntimeError(
                    "fresh legacy HTTP handler task is busy".into(),
                )
            })?;

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
                    let return_slot = ht.ram.sp;
                    ht.ram.raw_nv[return_slot] = 0;
                    let _stake = crate::vm::native::StakeGuard::nv(vm, nv);

                    // Current return-resource contract: only an owned Sse
                    // credential grants streaming. Ordinary integers remain data
                    // even when they collide with a live iterator registry key.
                    let iterator =
                        match crate::vm::ffi::http_return_resource::resolve_resource(vm, nv) {
                            Some(crate::vm::ffi::http_return_resource::ResourceClaim::Sse {
                                iterator_id,
                            }) => Some(iterator_id),
                            _ => None,
                        };
                    if let Some(iter_id) = iterator {
                        if vm.iterators.contains_key(&iter_id) {
                            let _iterator_cleanup = LegacyIteratorCleanup {
                                vm,
                                iterator: iter_id,
                            };
                            // SSE streaming mode: write SSE headers, then pull
                            // values from the iterator as SSE data frames.
                            drop(ht);
                            let sse_response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n{}\r\n",
                                cors_headers()
                            );
                            if stream
                                .write_all(sse_response.as_bytes())
                                .and_then(|_| stream.flush())
                                .is_err()
                            {
                                continue;
                            }

                            // Pull values from the iterator and write SSE frames
                            loop {
                                // Create a temp task for the next() call
                                let next_task_id = vm.spawn_task(0, 1024);
                                let _next_cleanup = LegacyTaskCleanup {
                                    vm,
                                    task: next_task_id,
                                };
                                let next_result = if let Some(nt_arc) =
                                    vm.tasks.get(&next_task_id).map(|task| task.value().clone())
                                {
                                    let mut nt = nt_arc.try_lock().map_err(|_| {
                                        crate::vm::engine::VMError::RuntimeError(
                                            "fresh SSE iterator task is busy".into(),
                                        )
                                    })?;
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

                                match next_result {
                                    Some(val) if auto_val::is_i32(val) => {
                                        let v = auto_val::decode_i32(val);
                                        if v == -1 {
                                            // Iterator exhausted
                                            break;
                                        }
                                        // Write SSE data frame
                                        let frame = format!("data: {}\n\n", v);
                                        if stream
                                            .write_all(frame.as_bytes())
                                            .and_then(|_| stream.flush())
                                            .is_err()
                                        {
                                            break;
                                        }
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
                                        if stream
                                            .write_all(frame.as_bytes())
                                            .and_then(|_| stream.flush())
                                            .is_err()
                                        {
                                            break;
                                        }
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
    Ok(())
}
