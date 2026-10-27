//! PLAN-843 结构拆分：http_client 域实现（原样搬移，算法/行为零变化）。
use super::*;

/// URL encode a string
#[auto_macros::rust_fn("Url.encode")]
pub fn shim_url_encode(s: String) -> String {
    urlencoding::encode(&s).to_string()
}

/// URL decode a string
#[auto_macros::rust_fn("Url.decode")]
pub fn shim_url_decode(s: String) -> String {
    urlencoding::decode(&s)
        .map(|c| c.to_string())
        .unwrap_or_default()
}

/// Encode query parameters
/// TODO: Handle Map type properly
#[auto_macros::rust_fn("Url.encode_query")]
pub fn shim_url_encode_query(_placeholder: String) -> String {
    // Placeholder - would need to handle Map type
    String::new()
}

/// Decode query string
/// TODO: Handle Map type properly
#[auto_macros::rust_fn("Url.decode_query")]
pub fn shim_url_decode_query(_query: String) -> Vec<String> {
    // Placeholder - would need to handle Map type
    Vec::new()
}

/// Join URL path segments
#[auto_macros::rust_fn("Url.join_path")]
pub fn shim_url_join_path(segments: Vec<String>) -> String {
    let path = segments
        .iter()
        .map(|s| s.trim_start_matches('/').trim_end_matches('/'))
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    format!("/{}", path)
}

/// Parse a URL and return its components as a JSON string
#[auto_macros::rust_fn("Url.parse")]
pub fn shim_url_parse(url_str: String) -> String {
    // Manual parsing without url crate
    let result = serde_json::json!({
        "url": url_str,
    });
    serde_json::to_string(&result).unwrap_or_default()
}

/// Get the scheme of a URL (e.g., "https")
#[auto_macros::rust_fn("Url.scheme")]
pub fn shim_url_scheme(url_str: String) -> String {
    if let Some(pos) = url_str.find("://") {
        url_str[..pos].to_string()
    } else {
        String::new()
    }
}

/// Get the host of a URL (e.g., "example.com")
#[auto_macros::rust_fn("Url.host")]
pub fn shim_url_host(url_str: String) -> String {
    let without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else {
        &url_str
    };
    // Remove userinfo if present
    let after_at = if let Some(pos) = without_scheme.rfind('@') {
        &without_scheme[pos + 1..]
    } else {
        without_scheme
    };
    // Take up to : or /
    let end = after_at
        .find(|c: char| c == ':' || c == '/' || c == '?' || c == '#')
        .unwrap_or(after_at.len());
    after_at[..end].to_string()
}

/// Get the port of a URL (returns -1 if no explicit port)
#[auto_macros::rust_fn("Url.port")]
pub fn shim_url_port(url_str: String) -> i32 {
    let without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else {
        &url_str
    };
    let after_at = if let Some(pos) = without_scheme.rfind('@') {
        &without_scheme[pos + 1..]
    } else {
        without_scheme
    };
    // Find the colon after host
    if let Some(colon_pos) = after_at.find(':') {
        let after_colon = &after_at[colon_pos + 1..];
        let end = after_colon
            .find(|c: char| c == '/' || c == '?' || c == '#')
            .unwrap_or(after_colon.len());
        after_colon[..end].parse::<i32>().unwrap_or(-1)
    } else {
        -1
    }
}

/// Get the path of a URL
#[auto_macros::rust_fn("Url.path")]
pub fn shim_url_path(url_str: String) -> String {
    let without_scheme = if let Some(pos) = url_str.find("://") {
        &url_str[pos + 3..]
    } else if url_str.starts_with('/') {
        // Relative URL
        return extract_path(url_str);
    } else {
        &url_str
    };
    // Skip host:port
    let after_host = if let Some(pos) = without_scheme.find('/') {
        &without_scheme[pos..]
    } else {
        return "/".to_string();
    };
    extract_path(after_host.to_string())
}

/// Get the query string of a URL
#[auto_macros::rust_fn("Url.query")]
pub fn shim_url_query(url_str: String) -> String {
    if let Some(qpos) = url_str.find('?') {
        let after_q = &url_str[qpos + 1..];
        if let Some(hpos) = after_q.find('#') {
            after_q[..hpos].to_string()
        } else {
            after_q.to_string()
        }
    } else {
        String::new()
    }
}

/// Get the fragment of a URL
#[auto_macros::rust_fn("Url.fragment")]
pub fn shim_url_fragment(url_str: String) -> String {
    if let Some(pos) = url_str.rfind('#') {
        url_str[pos + 1..].to_string()
    } else {
        String::new()
    }
}

/// Bind to address and create TCP listener
/// Returns handle (positive) on success, 0 on failure
pub fn shim_net_tcp_bind(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let listener = match StdTcpListener::bind(&addr) {
        Ok(l) => l,
        Err(_) => {
            task.ram.push_i32(0); // Return 0 on failure
            return Ok(());
        }
    };

    listener.set_nonblocking(false).ok();

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TCP_LISTENERS.with(|listeners| {
        listeners.borrow_mut().insert(handle, listener);
    });

    task.ram.push_i32(handle as i32);
    Ok(())
}

/// Accept a new connection from listener
/// Returns stream handle (positive) on success, 0 on failure
pub fn shim_net_tcp_listener_accept(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let listener_handle: i32 = crate::vm::native::pop_arg_i32(task);

    let stream = TCP_LISTENERS.with(|listeners| {
        let mut listeners = listeners.borrow_mut();
        let listener = listeners.get_mut(&(listener_handle as u64))?;
        listener.accept().ok().map(|(s, _)| s)
    });

    match stream {
        Some(stream) => {
            // Plan 313: Enable TCP_NODELAY by default for low-latency (SSE)
            stream.set_nodelay(true).ok();
            let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            TCP_STREAMS.with(|streams| {
                streams.borrow_mut().insert(handle, stream);
            });
            task.ram.push_i32(handle as i32);
        }
        None => {
            task.ram.push_i32(0); // Return 0 on failure
        }
    }
    Ok(())
}

/// Get local address of listener
#[auto_macros::rust_fn("Net.tcp_listener_local_addr")]
pub fn shim_net_tcp_listener_local_addr(listener_handle: i32) -> String {
    TCP_LISTENERS.with(|listeners| {
        let listeners = listeners.borrow();
        listeners
            .get(&(listener_handle as u64))
            .map(|l| l.local_addr().map(|a| a.to_string()).unwrap_or_default())
            .unwrap_or_default()
    })
}

/// Close listener
pub fn shim_net_tcp_listener_close(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let listener_handle: i32 = crate::vm::native::pop_arg_i32(task);
    TCP_LISTENERS.with(|listeners| {
        listeners.borrow_mut().remove(&(listener_handle as u64));
    });
    Ok(())
}

/// Connect to remote TCP server
/// Returns stream handle (positive) on success, 0 on failure
pub fn shim_net_tcp_connect(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream = match StdTcpStream::connect(&addr) {
        Ok(s) => s,
        Err(_) => {
            task.ram.push_i32(0);
            return Ok(());
        }
    };

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    TCP_STREAMS.with(|streams| {
        streams.borrow_mut().insert(handle, stream);
    });

    task.ram.push_i32(handle as i32);
    Ok(())
}

/// Read data from stream
/// Returns number of bytes read, or -1 on error
pub fn shim_net_tcp_stream_read(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let buf_size: i32 = crate::vm::native::pop_arg_i32(task);
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);

    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        let stream = match streams.get_mut(&(stream_handle as u64)) {
            Some(s) => s,
            None => return (-1, Vec::new()),
        };

        let mut buf = vec![0u8; buf_size as usize];
        match stream.read(&mut buf) {
            Ok(n) => {
                buf.truncate(n);
                (n as i32, buf)
            }
            Err(_) => (-1, Vec::new()),
        }
    });

    task.ram.push_i32(result.0);
    // Push bytes as Vec<i32>
    let bytes: Vec<i32> = result.1.into_iter().map(|b| b as i32).collect();
    super::super::convert::VMConvertible::push_to_stack(&bytes, task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Write data to stream
/// Returns number of bytes written, or -1 on error
pub fn shim_net_tcp_stream_write(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data: Vec<i32> = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);

    let bytes: Vec<u8> = data.into_iter().map(|b| b as u8).collect();

    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => stream.write(&bytes).map(|n| n as i32).unwrap_or(-1),
            None => -1,
        }
    });

    task.ram.push_i32(result);
    Ok(())
}

/// Read all data until EOF
pub fn shim_net_tcp_stream_read_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);

    let bytes = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => {
                let mut buf = Vec::new();
                stream.read_to_end(&mut buf).ok();
                buf
            }
            None => Vec::new(),
        }
    });

    let result: Vec<i32> = bytes.into_iter().map(|b| b as i32).collect();
    super::super::convert::VMConvertible::push_to_stack(&result, task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Read a line from stream
#[auto_macros::rust_fn("Net.tcp_stream_read_line")]
pub fn shim_net_tcp_stream_read_line(stream_handle: i32) -> String {
    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => {
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                reader.read_line(&mut line).ok();
                line.trim_end_matches('\n')
                    .trim_end_matches('\r')
                    .to_string()
            }
            None => String::new(),
        }
    })
}

/// Write string to stream
/// Returns number of bytes written, or -1 on error
pub fn shim_net_tcp_stream_write_str(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let s: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);

    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => stream
                .write_all(s.as_bytes())
                .map(|_| s.len() as i32)
                .unwrap_or(-1),
            None => -1,
        }
    });

    task.ram.push_i32(result);
    Ok(())
}

/// Close stream
pub fn shim_net_tcp_stream_close(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);
    TCP_STREAMS.with(|streams| {
        streams.borrow_mut().remove(&(stream_handle as u64));
    });
    Ok(())
}

/// Get peer address
#[auto_macros::rust_fn("Net.tcp_stream_peer_addr")]
pub fn shim_net_tcp_stream_peer_addr(stream_handle: i32) -> String {
    TCP_STREAMS.with(|streams| {
        let streams = streams.borrow();
        streams
            .get(&(stream_handle as u64))
            .map(|s| s.peer_addr().map(|a| a.to_string()).unwrap_or_default())
            .unwrap_or_default()
    })
}

/// Set read timeout
pub fn shim_net_tcp_stream_set_read_timeout(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let ms: i32 = crate::vm::native::pop_arg_i32(task);
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);

    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if let Some(stream) = streams.get_mut(&(stream_handle as u64)) {
            let timeout = if ms > 0 {
                Some(Duration::from_millis(ms as u64))
            } else {
                None
            };
            stream.set_read_timeout(timeout).ok();
        }
    });
    Ok(())
}

/// Set write timeout
pub fn shim_net_tcp_stream_set_write_timeout(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let ms: i32 = crate::vm::native::pop_arg_i32(task);
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);

    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if let Some(stream) = streams.get_mut(&(stream_handle as u64)) {
            let timeout = if ms > 0 {
                Some(Duration::from_millis(ms as u64))
            } else {
                None
            };
            stream.set_write_timeout(timeout).ok();
        }
    });
    Ok(())
}

/// Plan 313: Flush TCP stream (explicit flush for SSE/low-latency writes).
/// Note: TcpStream::flush() is a no-op for raw sockets (no user-space buffer),
/// but this provides API completeness and will be effective if BufWriter is
/// added later. For real SSE latency improvement, use set_nodelay(true).
pub fn shim_net_tcp_stream_flush(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);
    TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        if let Some(stream) = streams.get_mut(&(stream_handle as u64)) {
            use std::io::Write;
            stream.flush().ok();
        }
    });
    task.ram.push_i32(0); // success
    Ok(())
}

/// Plan 313: Set TCP_NODELAY on a stream (disables Nagle's algorithm).
/// Critical for SSE: without this, small data packets (like `data: ...\n\n`)
/// are buffered up to ~40ms before sending. Returns 0 on success, -1 on error.
pub fn shim_net_tcp_stream_set_nodelay(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let enabled: i32 = crate::vm::native::pop_arg_i32(task);
    let stream_handle: i32 = crate::vm::native::pop_arg_i32(task);
    let result = TCP_STREAMS.with(|streams| {
        let mut streams = streams.borrow_mut();
        match streams.get_mut(&(stream_handle as u64)) {
            Some(stream) => stream.set_nodelay(enabled != 0).map(|_| 0).unwrap_or(-1),
            None => -1,
        }
    });
    task.ram.push_i32(result);
    Ok(())
}

/// Create a new HTTP server (placeholder - returns handle)
pub fn shim_http_server(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // For now, return a placeholder handle
    // Full implementation would store route handlers
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    task.ram.push_i64(handle as i64);
    Ok(())
}

/// Add GET route (placeholder)
pub fn shim_http_server_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop handler and path (we'll ignore them for now)
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();

    // Return server handle unchanged
    task.ram.push_i64(server);
    Ok(())
}

/// Add POST route (placeholder)
pub fn shim_http_server_post(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add PUT route (placeholder)
pub fn shim_http_server_put(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add DELETE route (placeholder)
pub fn shim_http_server_delete(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add static file route (placeholder)
pub fn shim_http_server_static(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _dir: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _prefix: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Start server listening (blocking, using tokio)
pub fn shim_http_server_listen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _server: i64 = task.ram.pop_i64();

    // Plan 312 Phase 3: Real HTTP server with route dispatch.
    // Uses synchronous std::net (not tokio) for simplicity. Each request is
    // handled serially on the listen thread. Handler functions are called via
    // vm.call_fn_by_name on a fresh AutoTask (isolation per request).
    use std::io::{BufRead, Read, Write};
    use std::net::TcpListener;

    let listener = TcpListener::bind(&addr)
        .map_err(|e| VMError::RuntimeError(format!("HTTP server bind failed: {}", e)))?;
    eprintln!("[HTTP] Server listening on {}", addr);

    // Clone routes for the listen loop (avoid holding lock during requests)
    let routes = super::super::http_server::get_routes();

    for stream in listener.incoming() {
        let mut stream = match stream {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[HTTP] Accept error: {}", e);
                continue;
            }
        };

        // Parse HTTP request: method, path, body
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut request_line = String::new();
        if reader.read_line(&mut request_line).is_err() {
            continue;
        }
        let parts: Vec<&str> = request_line.split_whitespace().collect();
        if parts.len() < 2 {
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Read remaining headers (until empty line)
        let mut content_length = 0usize;
        let mut if_none_match = None;
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
            if header.to_lowercase().starts_with("if-none-match:") {
                if_none_match = Some(header[14..].trim().to_string());
            }
        }

        // Read body if present
        let body = if content_length > 0 {
            let mut buf = vec![0u8; content_length];
            let _ = (&mut reader).read_exact(&mut buf);
            String::from_utf8_lossy(&buf).to_string()
        } else {
            String::new()
        };
        drop(reader);
        if write_media_response(
            &mut stream,
            &req_method,
            &req_path,
            if_none_match.as_deref(),
        ) {
            continue;
        }

        // Route matching (PLAN-669: shared match_route — query-string split
        // + percent-decode, superseding the local find_route).
        let route_match = match super::super::http_server::match_route(
            &routes,
            &req_method,
            &req_path,
        ) {
            Some(m) => m,
            None => {
                let resp = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
                let _ = stream.write_all(resp.as_bytes());
                continue;
            }
        };

        // Call VM handler function on a fresh task
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) =
            vm.tasks.get(&handler_task_id)
        {
            // tokio::sync::Mutex — use blocking_lock() for sync context
            let mut ht = handler_task_arc.blocking_lock();

            // PLAN-669: by-name binding (shared with the other serve sites);
            // bind failures → 400/500 instead of calling with wrong args.
            let n_args = match super::super::http_server::bind_api_args_or_legacy(
                vm,
                &mut ht,
                &route_match.fn_name,
                &route_match.path_params,
                &route_match.query_params,
                &body,
                &req_method,
                &req_path,
            ) {
                Ok(n) => n,
                Err(e) => {
                    drop(ht);
                    vm.remove_task(handler_task_id);
                    let (status, msg) = match e {
                        super::super::http_server::ApiArgBindError::BadRequest(m) => {
                            ("400 Bad Request", m)
                        }
                        super::super::http_server::ApiArgBindError::Internal(m) => {
                            ("500 Internal Server Error", m)
                        }
                    };
                    eprintln!("[HTTP] {} {} → {} ({})", req_method, req_path, status, msg);
                    let err_body = format!(
                        "{{\"error\":{}}}",
                        super::super::http_server::json_escape_string(&msg)
                    );
                    let resp = format!(
                        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        status, err_body.len(), err_body
                    );
                    let _ = stream.write_all(resp.as_bytes());
                    continue;
                }
            };

            match vm.call_fn_by_name(&mut ht, &route_match.fn_name, n_args) {
                Ok(()) => {
                    // Result is on top of stack — decode via shared serializer
                    // (Plan 326 Phase 3: handles struct/array/Option returns).
                    let nv = ht.ram.pop_nv();
                    super::super::http_server::nv_to_json(vm, nv, 0)
                }
                Err(e) => {
                    eprintln!("[HTTP] Handler '{}' error: {:?}", route_match.fn_name, e);
                    None
                }
            }
        } else {
            None
        };

        // Clean up handler task
        vm.remove_task(handler_task_id);

        // Serialize result to JSON response
        let (status, body_json) = match result_json {
            Some(s) => ("200 OK", s),
            None => ("500 Internal Server Error", "{}".to_string()),
        };
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status, body_json.len(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }

    Ok(())
}

/// Create a new HTTP response.
/// PLAN-776 R1-F01：构造产 **Response 凭据对象**（默认快照 status=0——
/// 语义同原 HttpResponseData::default()；setter 链直接改凭据快照）。裸 id
/// +表路径退役——普通数值不再能通过表取得 Response 语义。
pub fn shim_http_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    super::super::http_return_resource::push_response_resource(task, vm, 0, Vec::new(), Vec::new())
}

/// Set response status.
/// PLAN-776 R1-F01/F06：receiver 是 Response 凭据对象——直接改快照
///（owner 校验在 mutate 内；非凭据对象/跨 owner → 错误，不猜测）。
pub fn shim_http_response_status(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let code: i32 = crate::vm::native::pop_arg_i32(task);
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    super::super::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
        s.status = code as u16
    })
    .ok_or_else(|| {
        VMError::RuntimeError("response_status: expected a Response credential".into())
    })?;
    drop(stake);
    // R2-03：返回值重建带影子份额（裸 push_nv 零份额 → 跨宽限窗 UAF）。
    vm.rc_push(task, res_nv);
    Ok(())
}

/// Set response header（Response 凭据快照载荷——R1-F01/F06）。
pub fn shim_http_response_header(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    super::super::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
        s.headers.push((key, value));
    })
    .ok_or_else(|| {
        VMError::RuntimeError("response_header: expected a Response credential".into())
    })?;
    drop(stake);
    // R2-03：返回值重建带影子份额（裸 push_nv 零份额 → 跨宽限窗 UAF）。
    vm.rc_push(task, res_nv);
    Ok(())
}

/// Set response text body（Response 凭据快照载荷——R1-F01/F06）。
pub fn shim_http_response_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    super::super::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
        s.body = body.into_bytes();
        // Add Content-Type header if not set
        if !s
            .headers
            .iter()
            .any(|(k, _)| k.to_lowercase() == "content-type")
        {
            s.headers.push((
                "Content-Type".to_string(),
                "text/plain; charset=utf-8".to_string(),
            ));
        }
    })
    .ok_or_else(|| VMError::RuntimeError("response_text: expected a Response credential".into()))?;
    drop(stake);
    // R2-03：返回值重建带影子份额（裸 push_nv 零份额 → 跨宽限窗 UAF）。
    vm.rc_push(task, res_nv);
    Ok(())
}

/// Set response HTML body（Response 凭据快照载荷——R1-F01/F06）。
pub fn shim_http_response_html(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    super::super::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
        s.body = body.into_bytes();
        if !s
            .headers
            .iter()
            .any(|(k, _)| k.to_lowercase() == "content-type")
        {
            s.headers.push((
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ));
        }
    })
    .ok_or_else(|| VMError::RuntimeError("response_html: expected a Response credential".into()))?;
    drop(stake);
    // R2-03：返回值重建带影子份额（裸 push_nv 零份额 → 跨宽限窗 UAF）。
    vm.rc_push(task, res_nv);
    Ok(())
}

/// Plan 346: `http.response.redirect(url: String, status: int) -> response_handle`
/// Create a redirect response (302 or 301).
/// PLAN-776 T-02: 真实资源生产点——返回 owned 快照凭据对象（TAG_OBJECT），
/// 不再进 HTTP_RESPONSES 表；源码不变（`return http.response.redirect(...)`
/// 直接进服务返回），wire 由 marshal 资源臂按快照回复。普通 int 永不命中。
pub fn shim_http_response_redirect(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status: i32 = crate::vm::native::pop_arg_i32(task);
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    super::super::http_return_resource::push_response_resource(
        task,
        vm,
        if status == 301 { 301 } else { 302 },
        vec![("Location".to_string(), url)],
        Vec::new(),
    )
}

/// Plan 346 5e (B6): `http.rate_limit(max_requests, window_ms)` — enable the
/// per-client-IP fixed-window rate limiter on the async HTTP server.
/// Requests beyond `max_requests` within `window_ms` get 429 + Retry-After.
/// `max_requests = 0` disables limiting again.
pub fn shim_http_rate_limit(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let window_ms: i32 = crate::vm::native::pop_arg_i32(task);
    let max_requests: i32 = crate::vm::native::pop_arg_i32(task);
    super::super::http_server::set_rate_limit(max_requests.max(0) as u32, window_ms.max(0) as u64);
    task.ram.push_i32(0);
    Ok(())
}

/// Plan 352: `http.server.use(middleware_fn_name: String)`
/// Register a middleware function to run before every handler.
/// Middleware receives the request path + method as a JSON string argument.
/// If it returns a non-empty string, that string is used as the response
/// (short-circuit). If it returns empty/nil, the handler runs normally.
pub fn shim_http_server_use(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let fn_name: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    if let Ok(mut chain) = MIDDLEWARE_CHAIN.lock() {
        chain.push(fn_name);
    }
    Ok(())
}

/// Set response bytes body
pub fn shim_http_response_bytes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // R2-06：迁移对象载荷（凭据快照 mutate + R2-03 owned-slot 协议——
    // 实参份额消费释放、返回槽 rc_push 重建带影子份额）。
    let data: Vec<i32> = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    let m = super::super::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
        s.body = data.into_iter().map(|b| b as u8).collect();
    });
    drop(stake);
    m.ok_or_else(|| {
        VMError::RuntimeError("response_bytes: expected a Response credential".into())
    })?;
    vm.rc_push(task, res_nv);
    Ok(())
}

/// Create a 200 OK response（凭据对象载荷）。
pub fn shim_http_ok(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(200, body);
    super::super::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 201 Created response（凭据对象载荷）。
pub fn shim_http_created(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(201, body);
    super::super::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 400 Bad Request response（凭据对象载荷）。
pub fn shim_http_bad_request(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(400, msg);
    super::super::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 404 Not Found response（凭据对象载荷）。
pub fn shim_http_not_found(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(404, msg);
    super::super::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 500 Internal Server Error response（凭据对象载荷）。
pub fn shim_http_internal_error(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(500, msg);
    super::super::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Perform a GET request (simple HTTP client). Returns a response handle.
/// Perform a GET request (simple HTTP client). Returns a response handle.
/// Plan 349 步骤 7: non-blocking via re-entry yield (same pattern as
/// `shim_http_get_json`). `http.get(url)` dispatches via CALL_NAT (nat#2230),
/// so the engine's `waiting_http_request_id` IP-rewind works here exactly as
/// it does for the `*_json` family. On re-entry the args were already popped
/// on the first call, so we check the pending request BEFORE popping.
/// PLAN-617 T-10: `Http.get` 的**返回协议是句柄**（Plan 446 E1/E2/E3 的测试
/// 钉子）：`res.status()/.body()/.header(k)` 访问器建立在 `HTTP_RESPONSES`
/// 句柄上，不能改成返回 body（那会砸掉已文档化、已测试的契约——本次实测
/// tv 档 5 红即证）。**双端一致的取 body 形态是 `json.to_value(Http.get_json(url))`**：
/// VM 侧 get_json 走 spawn_async_http（含 T-10 的相对 URL 基址展开）推 body
/// 字符串、to_value 解析成 Value（与 #[api] 改写的既有形状同型）；Vue 侧
/// ts_adapter 把 `Http.get_json` 映射为 fetch().json()、`json.to_value` 映射为
/// 恒等（Vue 侧已是解析后的对象）。030 的 store 即此写法。
/// PLAN-043 Phase 2 修订（2026-09-24）：UI 路径裸 `Http.get_json` 自
/// PLAN-080 F-2③ 起编译期自带 to_value（vm/codegen.rs Call 臂改发
/// `auto.http.get_json` + `auto.json.to_value`），**返回的已是解析产物
/// 而非 body 字符串**——app 侧单写 `Http.get_json(url)` 即可；旧配方
/// `json.to_value(Http.get_json(..))` 依靠 shim_json_to_value 的幂等臂
/// 透传（双重解析曾致对象被强转成 NanoValue 调试串、数据静默全空，
/// 020/029/030 全族中招，auto-os PLAN-043 Part 2 实证）。
pub fn shim_http_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_handle(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_handle("GET".into(), resolve_http_base_url(&url), None, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// Perform a POST request. Plan 349 步骤 7: non-blocking re-entry yield.
pub fn shim_http_post(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_handle(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_handle("POST".into(), url, Some(body), req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// Perform a PUT request. Plan 349 步骤 7: non-blocking re-entry yield.
pub fn shim_http_put(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_handle(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_handle("PUT".into(), url, Some(body), req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// Perform a DELETE request. Plan 349 步骤 7: non-blocking re-entry yield.
pub fn shim_http_delete(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_handle(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_handle("DELETE".into(), url, None, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// Create a new RequestBuilder handle (stored in heap_objects for CALL_SPEC dispatch)
/// http_request(method, url) -> RequestBuilder handle
pub fn shim_http_request(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let method: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let data = HttpRequestBuilderData {
        method,
        url: resolve_http_base_url(&url), // PLAN-617 T-10: 相对 URL 按基址展开
        headers: vec![],
        body: None,
        timeout_ms: None,
        ca_cert_path: None,
        skip_verify: false,
        client_cert_path: None,
        client_key_path: None,
        multipart_files: Vec::new(),
        multipart_texts: Vec::new(),
        cookie_store: false,
        retry_count: 0,
        gzip: false,
        brotli: false,
        build_error: None,
    };
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
        "RequestBuilder",
        std::sync::Mutex::new(data),
    );
    let heap_id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, heap_id as u64); // Plan 419
    Ok(())
}

/// Add a header to RequestBuilder
/// request_builder_header(rb, key, value) -> rb
pub fn shim_request_builder_header(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.headers.push((key, value));
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Set body on RequestBuilder
/// request_builder_body(rb, body) -> rb
pub fn shim_request_builder_body(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.body = Some(body);
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Set timeout on RequestBuilder
/// request_builder_timeout(rb, ms) -> rb
pub fn shim_request_builder_timeout(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let ms: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.timeout_ms = Some(ms as u64);
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Set JSON body on RequestBuilder
/// request_builder_json(rb, data) -> rb
pub fn shim_request_builder_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut builder) = mutex.lock() {
                    builder.body = Some(data);
                    if !builder
                        .headers
                        .iter()
                        .any(|(k, _)| k.eq_ignore_ascii_case("content-type"))
                    {
                        builder
                            .headers
                            .push(("Content-Type".to_string(), "application/json".to_string()));
                    }
                }
            }
        }
    }

    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// Send RequestBuilder and return Response handle
/// request_builder_send(rb) -> Response handle
/// Plan 349 步骤 7: non-blocking re-entry yield. The heap object is consumed
/// on first entry (pop + remove + clone all fields); subsequent re-entries
/// only check the pending result and push the handle when ready. Retry (W3)
/// and brotli (W4) run inside the detached worker thread.
pub fn shim_request_builder_send(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping the rb handle (the handle was popped on
    // the first call and the heap object already removed).
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_handle(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    let heap_id = rb_handle as u64;

    // Extract builder data and remove from heap
    let obj = vm.remove_heap_object(heap_id).ok_or_else(|| {
        VMError::RuntimeError(format!("Invalid RequestBuilder handle: {}", rb_handle))
    })?;
    let guard = obj.write().unwrap();
    let rso = guard
        .as_any()
        .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let mutex = rso
        .downcast_ref::<std::sync::Mutex<HttpRequestBuilderData>>()
        .ok_or_else(|| VMError::RuntimeError("Not a RequestBuilder".to_string()))?;
    let builder_data = mutex
        .lock()
        .map_err(|e| VMError::RuntimeError(format!("Mutex poison: {}", e)))?;
    let method = builder_data.method.clone();
    let url = builder_data.url.clone();
    let body = builder_data.body.clone();
    let headers = builder_data.headers.clone();
    let timeout_ms = builder_data.timeout_ms;
    let ca_cert_path = builder_data.ca_cert_path.clone();
    let skip_verify = builder_data.skip_verify;
    let client_cert = builder_data.client_cert_path.clone();
    let client_key = builder_data.client_key_path.clone();
    let mp_files = builder_data.multipart_files.clone();
    let mp_texts = builder_data.multipart_texts.clone();
    let cookie_store = builder_data.cookie_store;
    let retry_count = builder_data.retry_count; // Plan 349 步骤 8 (W3): now honored.
    let gzip = builder_data.gzip;
    let brotli = builder_data.brotli; // Plan 349 步骤 8 (W4)
    let build_error = builder_data.build_error.clone(); // PLAN-727 T-05: 预算越界
    drop(builder_data);
    if let Some(err) = build_error {
        // 坏请求不发出：终结性错误经 live-op 交付（重入拿错误句柄）。
        let req_id = alloc_async_id();
        crate::vm::ffi::async_http::register_live_op(req_id);
        crate::vm::ffi::async_http::complete_live_op(req_id, Err(err));
        task.waiting_http_request_id = Some(req_id);
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    drop(guard);

    // Submit to the fixed async client executor; result lands in the live-op
    // table (Structured). PLAN-705 T-03: 每请求 detached 线程退役；builder
    // 属性矩阵（header/timeout/cookie/gzip/brotli/multipart/retry/TLS 覆盖）
    // 原样迁入 async job。登记先于提交 worker；迟到完成由 presence 守卫丢弃。
    let req_id = alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_id);
    let job = async move {
        // PLAN-727 T-05：multipart 文件 part 流式——不再全量预读为 Vec；
        // 缺失/读失败终结性失败（不跳过 part、不假成功），错误经
        // file_errors 在发送失败时并入消息。每次允许的重试重开文件。
        let file_errors: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let job_file_errors = std::sync::Arc::clone(&file_errors);
        let result = (|| async move {
            let mut client_builder = reqwest::Client::builder();
            if skip_verify {
                client_builder = client_builder.danger_accept_invalid_certs(true);
            }
            if let Some(ref ca_path) = ca_cert_path {
                if let Ok(pem) = std::fs::read(ca_path) {
                    if let Ok(cert) = reqwest::Certificate::from_pem(&pem) {
                        client_builder = client_builder.add_root_certificate(cert);
                    }
                }
            }
            if let (Some(ref cert_path), Some(ref _key_path)) = (client_cert, client_key) {
                // Plan 350: mTLS client cert requires PKCS12 + native-tls/rustls-tls
                // feature; API registered, impl deferred until feature added.
                let _ = cert_path;
            }
            if let Some(ms) = timeout_ms {
                client_builder = client_builder.timeout(std::time::Duration::from_millis(ms));
            }
            if cookie_store {
                client_builder = client_builder.cookie_store(true);
            }
            if gzip {
                client_builder = client_builder.gzip(true);
            }
            if brotli {
                client_builder = client_builder.brotli(true);
            }
            let client = client_builder.build().map_err(|e| e.to_string())?;
            let url = append_default_queries(&url); // Plan 446 E4: 默认 query 注入
            let default_headers = snapshot_default_headers(); // Plan 446 E4
            send_with_retry_async(
                |c| {
                    let mut builder = match method.as_str() {
                        "POST" => c.post(&url),
                        "PUT" => c.put(&url),
                        "DELETE" => c.delete(&url),
                        _ => c.get(&url),
                    };
                    // Plan 446 E4: 默认头先落,显式 header 同名追加在后
                    // (reqwest 后设覆盖先设的发送语义)。
                    for (k, v) in &default_headers {
                        builder = builder.header(k.as_str(), v.as_str());
                    }
                    for (k, v) in &headers {
                        builder = builder.header(k.as_str(), v.as_str());
                    }
                    if !mp_files.is_empty() || !mp_texts.is_empty() {
                        let mut form = reqwest::multipart::Form::new();
                        for (name, value) in &mp_texts {
                            form = form.text(name.clone(), value.clone());
                        }
                        for (name, path) in &mp_files {
                            let part = stream_multipart_file_part(path, &file_errors);
                            form = form.part(name.clone(), part);
                        }
                        builder = builder.multipart(form);
                    } else if let Some(ref b) = body {
                        builder = builder
                            .header("Content-Type", "application/json")
                            .body(b.clone());
                    }
                    builder
                },
                client,
                retry_count,
            )
            .await
        })()
        .await;
        // PLAN-727 T-05：multipart 文件读失败并入错误消息（不省略 part、
        // 不假成功）；无文件错误时保持原错误文本。
        let mut result = result;
        if let Err(ref mut e) = result {
            let errs = job_file_errors
                .lock()
                .map(|g| g.join("; "))
                .unwrap_or_default();
            if !errs.is_empty() {
                *e = format!("{e}; multipart file errors: {errs}");
            }
        }
        result.map(|(status, headers, body)| AsyncResult::Structured {
            status,
            headers,
            body,
        })
    };
    let _ = super::super::async_http::submit_client_job(req_id, job);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `RequestBuilder.tls_ca_cert(path: String) -> RequestBuilder`
/// Load a PEM-format custom CA certificate for verifying server certs.
pub fn shim_request_builder_tls_ca_cert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cert_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.ca_cert_path = Some(cert_path);
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32); // Return builder for chaining
    Ok(())
}

/// `RequestBuilder.tls_skip_verify(skip: bool) -> RequestBuilder`
/// Skip TLS certificate verification (for dev/test).
pub fn shim_request_builder_tls_skip_verify(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let skip_val = crate::vm::native::pop_arg_i32(task);

    let _stake_skip_val = crate::vm::native::StakeGuard::new(vm, skip_val as i64 as u64);
    let skip = skip_val != 0;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.skip_verify = skip;
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `RequestBuilder.tls_client_cert(cert_path: String, key_path: String) -> RequestBuilder`
/// Set client certificate for mTLS (mutual TLS).
pub fn shim_request_builder_tls_client_cert(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let key_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let cert_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.client_cert_path = Some(cert_path);
                    data.client_key_path = Some(key_path);
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `RequestBuilder.cookie_store(enable: bool) -> RequestBuilder`
/// Enable automatic cookie jar (session persistence across requests).
pub fn shim_request_builder_cookie_store(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let enable = crate::vm::native::pop_arg_i32(task) != 0;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.cookie_store = enable;
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `RequestBuilder.retry(count: int) -> RequestBuilder`
/// Set automatic retry count for failed requests.
pub fn shim_request_builder_retry(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let count = crate::vm::native::pop_arg_i32(task) as u32;

    let _stake_count = crate::vm::native::StakeGuard::new(vm, count as i64 as u64);
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.retry_count = count;
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `RequestBuilder.gzip(enable: bool) -> RequestBuilder`
/// Enable automatic gzip decompression.
pub fn shim_request_builder_gzip(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let enable = crate::vm::native::pop_arg_i32(task) != 0;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.gzip = enable;
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `RequestBuilder.brotli(enable: int) -> RequestBuilder` — enable brotli
/// response decompression. Plan 349 步骤 7/8 (W4).
pub fn shim_request_builder_brotli(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let enable = crate::vm::native::pop_arg_i32(task) != 0;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    data.brotli = enable;
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

pub fn shim_request_builder_multipart_file(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let file_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let field_name: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    // PLAN-727 T-05：文件 part 数预算（越界记录，send 终结）。
                    if data.multipart_files.len() >= MULTIPART_MAX_FILES {
                        data.build_error = Some(format!(
                            "multipart file parts exceed budget {}",
                            MULTIPART_MAX_FILES
                        ));
                    } else {
                        data.multipart_files.push((field_name, file_path));
                    }
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `RequestBuilder.multipart_text(field_name: String, value: String) -> RequestBuilder`
/// Attach a text field to a multipart form upload.
pub fn shim_request_builder_multipart_text(
    task: &mut AutoTask,
    vm: &AutoVM,
) -> Result<(), VMError> {
    let value: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let field_name: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let rb_handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    if let Some(obj) = vm.get_heap_object(rb_handle as u64) {
        let mut guard = obj.write().unwrap();
        if let Some(rso) = guard
            .as_any_mut()
            .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_mut::<std::sync::Mutex<HttpRequestBuilderData>>() {
                if let Ok(mut data) = mutex.lock() {
                    // PLAN-727 T-05：文本字段预算（数量/单值/总量；越界记录，
                    // send 终结——坏请求不发出）。
                    if data.multipart_texts.len() >= MULTIPART_MAX_TEXT_FIELDS {
                        data.build_error = Some(format!(
                            "multipart text fields exceed budget {}",
                            MULTIPART_MAX_TEXT_FIELDS
                        ));
                    } else if value.len() > MULTIPART_MAX_TEXT_VALUE {
                        data.build_error = Some(format!(
                            "multipart text value {} exceeds budget {}",
                            value.len(),
                            MULTIPART_MAX_TEXT_VALUE
                        ));
                    } else {
                        let total: usize = data.multipart_texts.iter().map(|(_, v)| v.len()).sum();
                        if total + value.len() > MULTIPART_MAX_TEXT_TOTAL {
                            data.build_error = Some(format!(
                                "multipart text total {} exceeds budget {}",
                                total + value.len(),
                                MULTIPART_MAX_TEXT_TOTAL
                            ));
                        } else {
                            data.multipart_texts.push((field_name, value));
                        }
                    }
                }
            }
        }
    }
    task.ram.push_i32(rb_handle as i32);
    Ok(())
}

/// `http.upload(url: String, file_path: String) -> response_handle`
/// Simple single-file upload using multipart/form-data.
pub fn shim_http_upload(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // PLAN-727 T-05：legacy upload 迁移共享传输核心——流式 multipart
    //（field=file），yield 模式（不再每调用一线程 + blocking client）；
    // Response 形状保留：失败/无响应 = 内部错误句柄（旧伪 500 语义）。
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = super::super::http_transfer::consume_legacy_upload(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let file_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    super::super::http_transfer::start_legacy_upload_and_park(task, &url, &file_path)
}

/// `http.download(url: String, file_path: String) -> bool`
/// Download a file to disk (blocking, writes directly to file).
pub fn shim_http_download(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // PLAN-727 T-05：legacy download 迁移共享传输核心——增量落盘 + staging
    // 原子提交（失败保留原目标），yield 模式（不再每调用一线程 + 整文件
    // 入内存）。bool 形状保留：success 收据 = true。
    if let Some(req_id) = task.waiting_http_request_id {
        if crate::vm::ffi::async_http::live_op_ready_or_gone(req_id) {
            let ok = super::super::http_transfer::consume_legacy_download(task, req_id);
            task.ram.push_nv(auto_val::encode_bool(ok));
            return Ok(());
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let file_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    super::super::http_transfer::start_legacy_download_and_park(task, &url, &file_path, "{}")
}

/// `http.download_resume(url: String, file_path: String, offset: i64) -> bool`
/// Resume download from a given byte offset (HTTP Range header).
pub fn shim_http_download_resume(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // PLAN-727 T-05：legacy resume 迁移共享核心——严格续传（offset==本地
    // 长度 + 206 Content-Range 对齐；200 = 完整重启，绝不盲目 append）。
    // offset 自 i32 栈（既有 32 位 ABI 事实，负数 = 可观察 options 失败）。
    if let Some(req_id) = task.waiting_http_request_id {
        if crate::vm::ffi::async_http::live_op_ready_or_gone(req_id) {
            let ok = super::super::http_transfer::consume_legacy_download(task, req_id);
            task.ram.push_nv(auto_val::encode_bool(ok));
            return Ok(());
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let offset: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    let file_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let options = format!("{{\"offset\":{}}}", offset);
    super::super::http_transfer::start_legacy_download_and_park(task, &url, &file_path, &options)
}

/// `http.download_with_progress(url: String, file_path: String) -> iterator_id`
/// Download with progress events via an async iterator (reuses Plan 348
/// non-blocking yield mechanism). Each event is a JSON string:
/// {"downloaded": N, "total": M, "percent": P}
pub fn shim_http_download_with_progress(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let file_path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream_id = alloc_async_id();
    let handle = spawn_download_with_progress(url, file_path, stream_id);

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// Get status code from Response handle
/// response_status_code(res_handle) -> int
/// R1-F06：receiver 接受对象凭据或 client int 句柄（统一载荷）。
pub fn shim_response_status_code(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let res_nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    let status = read_response_payload(vm, res_nv)
        .map(|s| s.status as i32)
        .unwrap_or(0);

    task.ram.push_i32(status);
    Ok(())
}

/// Get header value from Response handle
/// response_header_get(res_handle, key) -> str
pub fn shim_response_header_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, res_nv);

    let value = read_response_payload(vm, res_nv)
        .and_then(|s| {
            s.headers
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(&key))
                .map(|(_, v)| v.clone())
        })
        .unwrap_or_default();

    value
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Get raw body bytes from Response handle
/// response_body(res_handle) -> str
/// Plan 446 批四 E3: 返回体改推 UTF-8 文本（lossy）。此前推 Vec<i32> 字节，
/// print/str.* 消费面只见堆 id（现场"巨大数字串"垃圾值——官方示例
/// 02_http_client 即以 `str.find(res.body(), ...)` 作文本消费）。
/// `Response.body_bytes(res_handle) -> List<int>` — byte-FAITHFUL body
/// accessor (plan-022 auto-down D4): binary responses (zip export) must
/// round-trip without UTF-8 lossy corruption; `Response.body` is the text
/// convenience twin. Bytes land as Vec<i32> for `File.write_bytes`.
/// `Response.body_to_file(res_handle, path)` — write the response body to a
/// local file BYTE-FAITHFULLY without ferrying bytes through VM values (the
/// returned-list route loses RC lifetime across CALL_SPEC, plan-022 D4).
/// Returns the byte count.
pub fn shim_response_body_to_file(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, res_nv);

    let body_bytes = read_response_payload(vm, res_nv)
        .map(|s| s.body)
        .unwrap_or_default();

    std::fs::write(&path, &body_bytes)
        .map_err(|e| VMError::RuntimeError(format!("body_to_file write failed: {}", e)))?;
    task.ram.push_i32(body_bytes.len() as i32);
    Ok(())
}

pub fn shim_response_body_bytes(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let res_nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, res_nv);

    let body_bytes = read_response_payload(vm, res_nv)
        .map(|s| s.body)
        .unwrap_or_default();
    let ints: Vec<i32> = body_bytes.iter().map(|&b| b as i32).collect();
    let list_id = crate::vm::native::create_list_from_i32(vm, ints);
    // Plan 432 D26: 新堆 id 入栈必须 rc_push_id（裸 push_i32 零份额，
    // 列表在局部槽引用释放时即死——length 恒 0 的根因）。
    vm.rc_push_id(task, list_id);
    Ok(())
}

pub fn shim_response_body(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let res_nv = crate::vm::native::pop_arg_nv(task);
    let _stake = crate::vm::native::StakeGuard::nv(vm, res_nv);

    let body_bytes = read_response_payload(vm, res_nv)
        .map(|s| s.body)
        .unwrap_or_default();

    let text = String::from_utf8_lossy(&body_bytes).to_string();
    text.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// 创建流式 HTTP GET 请求（非阻塞：立即返回句柄，建立经异步生产者）。
pub fn shim_http_get_stream(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_id = super::super::http_stream::open_stream(super::super::http_stream::StreamOpen {
        url: resolve_http_base_url(&url),
        method: "GET".to_string(),
        body: None,
        headers: Vec::new(),
        mode: super::super::http_stream::StreamMode::Raw,
    });
    super::super::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    // 单槽 i32（.at int 变量链/for-in _iterator 局部同链，sse_open 先例）。
    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// 创建流式 HTTP POST 请求（非阻塞；Content-Type: application/json 保留）。
pub fn shim_http_post_stream(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_id = super::super::http_stream::open_stream(super::super::http_stream::StreamOpen {
        url: resolve_http_base_url(&url),
        method: "POST".to_string(),
        body: Some(body),
        headers: vec![("Content-Type".to_string(), "application/json".to_string())],
        mode: super::super::http_stream::StreamMode::Raw,
    });
    super::super::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// 从流中读取下一个数据块（typed pull）。
///
/// Data → 文本块；EOF/取消 → `[DONE]` 哨兵（既有适配保留，迭代器终结凭
/// 状态而非字符串比较——决策 D-4）；Failed → RuntimeError（既有读错误
/// 形态）；无数据/建立中 → 段挂起（`waiting_http_stream_id`，引擎 CALL_NAT
/// 重试协议与 waiting_sse_stream_id 同构）。
pub fn shim_http_stream_next(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // 重入臂（Plan 349 step 7 协议，与 HttpRequest 族同构）：CALL_NAT
    // rewind 重触发时凭据即句柄——首轮已 pop 栈参，重入**不得再 pop**
    //（栈上无参，重 pop 读垃圾）。
    if let Some(id) = task.waiting_http_stream_id.take() {
        return push_stream_next_value(task, vm, id as i64);
    }
    let handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    push_stream_next_value(task, vm, handle)
}

/// 检查流是否完成（非消费式：终态且队列已空 → 1；否则 0）。
pub fn shim_http_stream_is_done(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ = vm;
    let handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    // .at 声明返回 int（1=done）——推 i32 而非 bool 位型（`== 1` 比较
    // 按 int 语义；bool 位型比较恒假，707 实测）。
    let done = super::super::http_stream::stream_terminal_error(handle as u64).is_some();
    task.ram.push_i32(if done { 1 } else { 0 });
    Ok(())
}

/// 关闭流（语义升级：终结状态机 + abort 生产者 + 出表——连接真正关闭）。
pub fn shim_http_stream_close(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ = vm;
    let handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    super::super::http_stream::stream_cancel(handle as u64);
    Ok(())
}

/// Plan 321: Create an iterator from an HTTPStream for for-loop consumption.
/// `for chunk in http_stream { }` calls this via the Iter protocol.
/// PLAN-707：句柄即 `http_stream::STREAMS` 的流 id（同源发号）。
pub fn shim_http_stream_iter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    let hs_iter = crate::vm::engine::HttpStreamIterator {
        stream_handle: handle as u64,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators
            .insert(next_id, crate::vm::engine::Iterator::HttpStream(hs_iter));
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// Plan 341: `auto.http.sse_get_stream(url) -> iterator_id`
///
/// Opens an SSE (Server-Sent Events) connection to `url` and returns an
/// iterator id. PLAN-707 T-04：生产端迁入 `http_stream` 资源表（共享固定
/// runtime、增量 SseDecoder、可取消），经 legacy 通道桥以既有
/// AsyncStreamEvent（Data/Done/Error）形态供给 `Iterator::AsyncHttpStream`
/// 的 try_recv 消费面——事件语义逐字节保留（决策 D-4）。
///
/// Usage in Auto: `for event in http.sse_get_stream(url) { print(event) }`
pub fn shim_http_sse_get_stream(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream_id = super::super::http_stream::open_stream(super::super::http_stream::StreamOpen {
        url: resolve_http_base_url(&url),
        method: "GET".to_string(),
        body: None,
        headers: Vec::new(),
        mode: super::super::http_stream::StreamMode::Sse,
    });
    super::super::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    let handle = super::super::http_stream::spawn_legacy_channel_bridge(stream_id);

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    // Create the VM-side iterator.
    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// PLAN-658 T-04: `auto.http.sse_open(url) -> stream_id (i64)`
///
/// 与 `auto.http.sse_get_stream` 同源（同一 open + legacy 通道桥），但返回
/// **流句柄**而非迭代器 id——配合 sse_poll 供 Tick 驱动的逐步消费循环用
///（迭代器形态只能整流 for-loop，无法分帧跨 Tick 拉取）。
pub fn shim_http_stream_sse_open(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_id = super::super::http_stream::open_stream(super::super::http_stream::StreamOpen {
        url: resolve_http_base_url(&url),
        method: "GET".to_string(),
        body: None,
        headers: Vec::new(),
        mode: super::super::http_stream::StreamMode::Sse,
    });
    super::super::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    let handle = super::super::http_stream::spawn_legacy_channel_bridge(stream_id);
    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }
    // 单槽 i32（.at int 变量链）——i64 双槽编码在 int 赋值位会丢高位字。
    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// PLAN-658 T-04: `auto.http.sse_poll(stream_id) -> str`
///
/// 非阻塞 try_recv：Data → 载荷；缓冲空且未结束 → ""（pending 哨兵）；
/// 结束/出错 → "[DONE]"（与句柄族 stream_next 的终止哨兵一致）。
pub fn shim_http_stream_sse_poll(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let stream_id: i64 = task.ram.pop_i32() as i64;
    let handle = ASYNC_STREAMS
        .lock()
        .ok()
        .and_then(|m| m.get(&(stream_id as u64)).cloned());
    let Some(handle) = handle else {
        return Err(VMError::RuntimeError(format!(
            "Invalid SSE stream handle: {stream_id}"
        )));
    };
    let out = match handle.rx.lock() {
        Ok(mut rx) => match rx.try_recv() {
            Ok(AsyncStreamEvent::Data(s)) => s,
            Ok(AsyncStreamEvent::Done) => "[DONE]".to_string(),
            Ok(AsyncStreamEvent::Error(_)) => "[DONE]".to_string(),
            Err(_) => {
                if handle.done.load(Ordering::SeqCst) {
                    "[DONE]".to_string()
                } else {
                    String::new()
                }
            }
        },
        Err(_) => "[DONE]".to_string(),
    };
    out.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// PLAN-707 T-04: `auto.http.sse_close(stream_id)`——显式释放未完流
///（终结状态机 + abort 生产者 + 双表出表）。幂等。
pub fn shim_http_stream_sse_close(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ = vm;
    let stream_id: i64 = task.ram.pop_i32() as i64;
    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.remove(&(stream_id as u64));
    }
    super::super::http_stream::stream_cancel(stream_id as u64);
    Ok(())
}

/// PLAN-707 T-04: `auto.http.sse_error(stream_id) -> str`——终态错误查询
///（"" = 未终结或无错；Failed = 诊断；Cancelled = "stream cancelled"）。
/// 非消费式，可重复查询。
pub fn shim_http_stream_sse_error(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ = vm;
    let stream_id: i64 = task.ram.pop_i32() as i64;
    let err = super::super::http_stream::stream_terminal_error(stream_id as u64)
        .and_then(|opt| opt)
        .unwrap_or_default();
    err.push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))
}

/// POST streaming with custom headers (Plan 159: AutoCode agent support).
///
/// Stack args (bottom to top): url, body, headers_json
/// - url: String
/// - body: String (JSON body)
/// - headers_json: String (JSON object like `{"Authorization": "Bearer xxx", "Content-Type": "application/json"}`)
///
/// Returns an HTTP stream handle (i64) on the stack.
/// PLAN-707 T-04：非阻塞异步建立；VM JSON 字符串头原样透传（决策 D-4）。
pub fn shim_http_post_stream_with_headers(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let headers_json: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    // Parse headers JSON
    let headers_map: std::collections::HashMap<String, String> =
        serde_json::from_str(&headers_json).unwrap_or_default();

    let stream_id = super::super::http_stream::open_stream(super::super::http_stream::StreamOpen {
        url: resolve_http_base_url(&url),
        method: "POST".to_string(),
        body: Some(body),
        headers: headers_map.into_iter().collect(),
        mode: super::super::http_stream::StreamMode::Raw,
    });
    super::super::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// Plan 446 E4: `Http.set_default_header(name, value)` —— 进程级默认请求头
/// (musk: Authorization: Bearer <jwt>)。同名替换。
pub fn shim_http_set_default_header(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let name: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let mut reg = http_defaults().lock().unwrap();
    http_set_default_entry(&mut reg.headers, name, value);
    task.ram.push_nv(0);
    Ok(())
}

/// Plan 446 E4: `Http.set_default_query(name, value)` —— 进程级默认 query
/// (musk: workspace=<wid>)。同名替换。
pub fn shim_http_set_default_query(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let name: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let mut reg = http_defaults().lock().unwrap();
    http_set_default_entry(&mut reg.queries, name, value);
    task.ram.push_nv(0);
    Ok(())
}

/// Plan 446 E4: `Http.clear_default_auth()` —— 清空默认头/默认 query
/// (musk logout 面)。
pub fn shim_http_clear_default_auth(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let mut reg = http_defaults().lock().unwrap();
    reg.headers.clear();
    reg.queries.clear();
    task.ram.push_nv(0);
    Ok(())
}

/// PLAN-083 T-01: `Http.get_msg(url, event)` —— 消息桥 GET（fire-and-forget）。
/// event 形如 "ForgeStore.SessionsLoaded"（首个点前 = 目标 store 名，点后 =
/// 回填 handler 名；无点 = 根 handler）。立即返回 0，不置 task Waiting——
/// 发起段 handler 可当场结束。完成经 http_msg 队列由渲染层泵回填。
pub fn shim_http_get_msg(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let event: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (widget, event) = http_msg_split_target(&event);
    spawn_async_http_msg_get(url, widget, event);
    task.ram.push_nv(0);
    Ok(())
}

/// 仅供测试：以完整 event 串驱动消息桥（切分 + spawn），绕开 task/vm 栈。
// PLAN-814: 跨 crate 测试支持面（上层测试引用）
pub fn shim_http_get_msg_spawn_for_test(url: &str, event: &str) {
    let (widget, event) = http_msg_split_target(event);
    spawn_async_http_msg_get(url.to_string(), widget, event);
}

pub fn shim_http_get_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Plan 349 step 7: Non-blocking async HTTP. The engine retries this CALL_NAT
    // after a yield (it rewinds IP, so the shim re-enters from the top). On
    // re-entry the args were already popped on the first call, so we must
    // check the pending request BEFORE popping — otherwise we eat the wrong
    // values off the stack and underflow.
    if let Some(req_id) = task.waiting_http_request_id {
        // PLAN-712 T-17：传输层失败 = 可捕获异常（.at try/catch 可接），
        // 不再吞成 `{"error":..,"status":0}` 成功值（详情见
        // check_async_http_result 文档）。
        match check_async_http_result(req_id) {
            Some(Ok(body)) => {
                task.waiting_http_request_id = None;
                return push_string_result(task, vm, body);
            }
            Some(Err(e)) => {
                task.waiting_http_request_id = None;
                return Err(VMError::RuntimeError(format!("Http.get_json: {e}")));
            }
            None => {}
        }
        // Still pending — yield again.
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    // First call — pop the url, spawn the request and yield.
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http("GET".into(), url, None, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `auto.http.post_json(url, body) -> String` — POST JSON, return body.
pub fn shim_http_post_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping args (see shim_http_get_json for rationale).
    if let Some(req_id) = task.waiting_http_request_id {
        // PLAN-712 T-17：传输层失败 = 可捕获异常（同 get_json）。
        match check_async_http_result(req_id) {
            Some(Ok(resp)) => {
                task.waiting_http_request_id = None;
                return push_string_result(task, vm, resp);
            }
            Some(Err(e)) => {
                task.waiting_http_request_id = None;
                return Err(VMError::RuntimeError(format!("Http.post_json: {e}")));
            }
            None => {}
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http("POST".into(), url, Some(body), req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `auto.http.put_json(url, body) -> String` — PUT JSON, return body.
pub fn shim_http_put_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping args (see shim_http_get_json for rationale).
    if let Some(req_id) = task.waiting_http_request_id {
        // PLAN-712 T-17：传输层失败 = 可捕获异常（同 get_json）。
        match check_async_http_result(req_id) {
            Some(Ok(resp)) => {
                task.waiting_http_request_id = None;
                return push_string_result(task, vm, resp);
            }
            Some(Err(e)) => {
                task.waiting_http_request_id = None;
                return Err(VMError::RuntimeError(format!("Http.put_json: {e}")));
            }
            None => {}
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http("PUT".into(), url, Some(body), req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `auto.http.delete_json(url) -> String` — DELETE, return body.
pub fn shim_http_delete_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping args (see shim_http_get_json for rationale).
    if let Some(req_id) = task.waiting_http_request_id {
        // PLAN-712 T-17：传输层失败 = 可捕获异常（同 get_json）。
        match check_async_http_result(req_id) {
            Some(Ok(resp)) => {
                task.waiting_http_request_id = None;
                return push_string_result(task, vm, resp);
            }
            Some(Err(e)) => {
                task.waiting_http_request_id = None;
                return Err(VMError::RuntimeError(format!("Http.delete_json: {e}")));
            }
            None => {}
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http("DELETE".into(), url, None, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `auto.http.patch_json(url, body) -> String` — PATCH JSON, return body.
///
/// Plan 340 gap fix: PATCH was accepted by the AST/parser and used by real
/// `#[api]` endpoints (e.g. 015-notes `toggle_pin`), but had no native and no
/// codegen branch, so any PATCH `#[api]` call in split mode hard-failed at
/// codegen. Mirrors shim_http_put_json (url + body).
pub fn shim_http_patch_json(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping args (see shim_http_get_json for rationale).
    if let Some(req_id) = task.waiting_http_request_id {
        // PLAN-712 T-17：传输层失败 = 可捕获异常（同 get_json）。
        match check_async_http_result(req_id) {
            Some(Ok(resp)) => {
                task.waiting_http_request_id = None;
                return push_string_result(task, vm, resp);
            }
            Some(Err(e)) => {
                task.waiting_http_request_id = None;
                return Err(VMError::RuntimeError(format!("Http.patch_json: {e}")));
            }
            None => {}
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http("PATCH".into(), url, Some(body), req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// HTTP POST with auth — returns body string directly, stores status code.
/// Plan 349 步骤 7: non-blocking re-entry yield (x-api-key style for Anthropic).
/// Stack: [url, body, api_key] -> [response_body_str]
pub fn shim_http_post_sync(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_auth(req_id) {
            task.waiting_http_request_id = None;
            return push_auth_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let api_key: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key_opt = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_auth("POST".into(), url, Some(body), key_opt, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// HTTP POST with Bearer auth — for OpenAI-compatible APIs.
/// Plan 349 步骤 7: non-blocking re-entry yield.
/// Stack: [url, body, api_key] -> [response_body_str]
pub fn shim_http_post_bearer(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_auth(req_id) {
            task.waiting_http_request_id = None;
            return push_auth_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let api_key: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key_opt = if api_key.is_empty() {
        None
    } else {
        Some(api_key)
    };
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_bearer("POST".into(), url, Some(body), key_opt, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// Return the status code from the last HTTP request.
/// Stack: [] -> [status_i32]
pub fn shim_http_last_status(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let status = LAST_HTTP_STATUS.with(|s| s.get());
    task.ram.push_i32(status);
    Ok(())
}

/// Synchronous HTTP GET — returns response body as string.
/// Plan 349 步骤 7: non-blocking re-entry yield.
/// Stack: [url] -> [response_body_str]
pub fn shim_http_get_sync(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(req_id) = task.waiting_http_request_id {
        if let Some(result) = check_async_http_result_auth(req_id) {
            task.waiting_http_request_id = None;
            return push_auth_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let url: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    // PLAN-705 T-02: 登记先于提交 worker（协议顺序）。
    crate::vm::ffi::async_http::register_live_op(req_id);
    spawn_async_http_auth("GET".into(), url, None, None, req_id);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// HTTP listen stub — starts a simple HTTP server.
/// Stack: [callback_closure, port, host] -> []
/// Currently a stub that prints a message and returns.
pub fn shim_http_listen(_task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop 3 args: host, port, callback
    // For now, just print a warning and return void
    eprintln!("WARN: http.listen() stub called — HTTP server not yet implemented in AutoVM");
    Ok(())
}
