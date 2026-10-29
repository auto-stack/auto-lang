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
    let addr: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let addr: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    crate::vm::ffi::convert::VMConvertible::push_to_stack(&bytes, task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// Write data to stream
/// Returns number of bytes written, or -1 on error
pub fn shim_net_tcp_stream_write(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data: Vec<i32> = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    crate::vm::ffi::convert::VMConvertible::push_to_stack(&result, task, vm)
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
    let s: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let _path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();

    // Return server handle unchanged
    task.ram.push_i64(server);
    Ok(())
}

/// Add POST route (placeholder)
pub fn shim_http_server_post(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add PUT route (placeholder)
pub fn shim_http_server_put(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add DELETE route (placeholder)
pub fn shim_http_server_delete(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _handler: i64 = task.ram.pop_i64();
    let _path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Add static file route (placeholder)
pub fn shim_http_server_static(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _dir: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _prefix: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let server: i64 = task.ram.pop_i64();
    task.ram.push_i64(server);
    Ok(())
}

/// Start server listening (blocking, using tokio)
pub fn shim_http_server_listen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let routes = crate::vm::ffi::http_server::get_routes();

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
        let route_match = match crate::vm::ffi::http_server::match_route(
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
            let n_args = match crate::vm::ffi::http_server::bind_api_args_or_legacy(
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
                        crate::vm::ffi::http_server::ApiArgBindError::BadRequest(m) => {
                            ("400 Bad Request", m)
                        }
                        crate::vm::ffi::http_server::ApiArgBindError::Internal(m) => {
                            ("500 Internal Server Error", m)
                        }
                    };
                    eprintln!("[HTTP] {} {} → {} ({})", req_method, req_path, status, msg);
                    let err_body = format!(
                        "{{\"error\":{}}}",
                        crate::vm::ffi::http_server::json_escape_string(&msg)
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
                    crate::vm::ffi::http_server::nv_to_json(vm, nv, 0)
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
    crate::vm::ffi::http_return_resource::push_response_resource(
        task,
        vm,
        0,
        Vec::new(),
        Vec::new(),
    )
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
    crate::vm::ffi::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
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
    let value: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    crate::vm::ffi::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
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
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    crate::vm::ffi::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
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
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    crate::vm::ffi::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    crate::vm::ffi::http_return_resource::push_response_resource(
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
    crate::vm::ffi::http_server::set_rate_limit(
        max_requests.max(0) as u32,
        window_ms.max(0) as u64,
    );
    task.ram.push_i32(0);
    Ok(())
}

/// Plan 352: `http.server.use(middleware_fn_name: String)`
/// Register a middleware function to run before every handler.
/// Middleware receives the request path + method as a JSON string argument.
/// If it returns a non-empty string, that string is used as the response
/// (short-circuit). If it returns empty/nil, the handler runs normally.
pub fn shim_http_server_use(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let fn_name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let data: Vec<i32> = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let res_nv = crate::vm::native::pop_arg_nv(task);
    // PLAN-776 R10-01：StakeGuard Drop 即**恰一次**消费（其 Drop 执行
    // rc_release_id；R2-03 起叠加的显式 rc_release 是第二份——共享 RC2
    // 被销成 0，释放返回槽后全局合法 Response 被回收触发 UAF）。读取
    // 期间份额未动（release 只在 Drop）；错误早退同样恰一份。
    let stake = crate::vm::native::StakeGuard::nv(vm, res_nv);
    let m = crate::vm::ffi::http_return_resource::mutate_response_snapshot(vm, res_nv, |s| {
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
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(200, body);
    crate::vm::ffi::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 201 Created response（凭据对象载荷）。
pub fn shim_http_created(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(201, body);
    crate::vm::ffi::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 400 Bad Request response（凭据对象载荷）。
pub fn shim_http_bad_request(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(400, msg);
    crate::vm::ffi::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 404 Not Found response（凭据对象载荷）。
pub fn shim_http_not_found(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(404, msg);
    crate::vm::ffi::http_return_resource::push_response_resource(task, vm, status, headers, body)
}

/// Create a 500 Internal Server Error response（凭据对象载荷）。
pub fn shim_http_internal_error(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let msg: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let (status, headers, body) = text_plain_response(500, msg);
    crate::vm::ffi::http_return_resource::push_response_resource(task, vm, status, headers, body)
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let method: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let value: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let key: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let data: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let _ = crate::vm::ffi::async_http::submit_client_job(req_id, job);
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `RequestBuilder.tls_ca_cert(path: String) -> RequestBuilder`
/// Load a PEM-format custom CA certificate for verifying server certs.
pub fn shim_request_builder_tls_ca_cert(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let cert_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let key_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let cert_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let file_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let field_name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let value: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let field_name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
        if let Some(result) = crate::vm::ffi::http_transfer::consume_legacy_upload(req_id) {
            task.waiting_http_request_id = None;
            return push_handle_result(task, vm, result);
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let file_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    crate::vm::ffi::http_transfer::start_legacy_upload_and_park(task, &url, &file_path)
}

/// `http.download(url: String, file_path: String) -> bool`
/// Download a file to disk (blocking, writes directly to file).
pub fn shim_http_download(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // PLAN-727 T-05：legacy download 迁移共享传输核心——增量落盘 + staging
    // 原子提交（失败保留原目标），yield 模式（不再每调用一线程 + 整文件
    // 入内存）。bool 形状保留：success 收据 = true。
    if let Some(req_id) = task.waiting_http_request_id {
        if crate::vm::ffi::async_http::live_op_ready_or_gone(req_id) {
            let ok = crate::vm::ffi::http_transfer::consume_legacy_download(task, req_id);
            task.ram.push_nv(auto_val::encode_bool(ok));
            return Ok(());
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let file_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    crate::vm::ffi::http_transfer::start_legacy_download_and_park(task, &url, &file_path, "{}")
}

/// `http.download_resume(url: String, file_path: String, offset: i64) -> bool`
/// Resume download from a given byte offset (HTTP Range header).
pub fn shim_http_download_resume(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // PLAN-727 T-05：legacy resume 迁移共享核心——严格续传（offset==本地
    // 长度 + 206 Content-Range 对齐；200 = 完整重启，绝不盲目 append）。
    // offset 自 i32 栈（既有 32 位 ABI 事实，负数 = 可观察 options 失败）。
    if let Some(req_id) = task.waiting_http_request_id {
        if crate::vm::ffi::async_http::live_op_ready_or_gone(req_id) {
            let ok = crate::vm::ffi::http_transfer::consume_legacy_download(task, req_id);
            task.ram.push_nv(auto_val::encode_bool(ok));
            return Ok(());
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }
    let offset: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    let file_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let options = format!("{{\"offset\":{}}}", offset);
    crate::vm::ffi::http_transfer::start_legacy_download_and_park(task, &url, &file_path, &options)
}

/// `http.download_with_progress(url: String, file_path: String) -> iterator_id`
/// Download with progress events via an async iterator (reuses Plan 348
/// non-blocking yield mechanism). Each event is a JSON string:
/// {"downloaded": N, "total": M, "percent": P}
pub fn shim_http_download_with_progress(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let file_path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream_id = alloc_async_id();
    let handle = spawn_download_with_progress(url, file_path, stream_id);

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let iter_id = vm.register_iterator(crate::vm::engine::Iterator::AsyncHttpStream(
        crate::vm::engine::AsyncStreamIterator {
            stream_id,
            done: false,
        },
    ));
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
    let key: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let path: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_id =
        crate::vm::ffi::http_stream::open_stream(crate::vm::ffi::http_stream::StreamOpen {
            url: resolve_http_base_url(&url),
            method: "GET".to_string(),
            body: None,
            headers: Vec::new(),
            mode: crate::vm::ffi::http_stream::StreamMode::Raw,
        });
    crate::vm::ffi::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    // 单槽 i32（.at int 变量链/for-in _iterator 局部同链，sse_open 先例）。
    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// 创建流式 HTTP POST 请求（非阻塞；Content-Type: application/json 保留）。
pub fn shim_http_post_stream(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_id =
        crate::vm::ffi::http_stream::open_stream(crate::vm::ffi::http_stream::StreamOpen {
            url: resolve_http_base_url(&url),
            method: "POST".to_string(),
            body: Some(body),
            headers: vec![("Content-Type".to_string(), "application/json".to_string())],
            mode: crate::vm::ffi::http_stream::StreamMode::Raw,
        });
    crate::vm::ffi::http_server::register_scope_stream(stream_id);
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
    let done = crate::vm::ffi::http_stream::stream_terminal_error(handle as u64).is_some();
    task.ram.push_i32(if done { 1 } else { 0 });
    Ok(())
}

/// 关闭流（语义升级：终结状态机 + abort 生产者 + 出表——连接真正关闭）。
pub fn shim_http_stream_close(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ = vm;
    let handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    crate::vm::ffi::http_stream::stream_cancel(handle as u64);
    Ok(())
}

/// Plan 321: Create an iterator from an HTTPStream for for-loop consumption.
/// `for chunk in http_stream { }` calls this via the Iter protocol.
/// PLAN-707：句柄即 `http_stream::STREAMS` 的流 id（同源发号）。
pub fn shim_http_stream_iter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = crate::vm::native::pop_arg_i32(task) as i64;

    let iter_id = vm.register_iterator(crate::vm::engine::Iterator::HttpStream(
        crate::vm::engine::HttpStreamIterator {
            stream_handle: handle as u64,
            done: false,
        },
    ));
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream_id =
        crate::vm::ffi::http_stream::open_stream(crate::vm::ffi::http_stream::StreamOpen {
            url: resolve_http_base_url(&url),
            method: "GET".to_string(),
            body: None,
            headers: Vec::new(),
            mode: crate::vm::ffi::http_stream::StreamMode::Sse,
        });
    crate::vm::ffi::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    let handle = crate::vm::ffi::http_stream::spawn_legacy_channel_bridge(stream_id);

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    // Create the VM-side iterator.
    let iter_id = vm.register_iterator(crate::vm::engine::Iterator::AsyncHttpStream(
        crate::vm::engine::AsyncStreamIterator {
            stream_id,
            done: false,
        },
    ));
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// PLAN-658 T-04: `auto.http.sse_open(url) -> stream_id (i64)`
///
/// 与 `auto.http.sse_get_stream` 同源（同一 open + legacy 通道桥），但返回
/// **流句柄**而非迭代器 id——配合 sse_poll 供 Tick 驱动的逐步消费循环用
///（迭代器形态只能整流 for-loop，无法分帧跨 Tick 拉取）。
pub fn shim_http_stream_sse_open(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let stream_id =
        crate::vm::ffi::http_stream::open_stream(crate::vm::ffi::http_stream::StreamOpen {
            url: resolve_http_base_url(&url),
            method: "GET".to_string(),
            body: None,
            headers: Vec::new(),
            mode: crate::vm::ffi::http_stream::StreamMode::Sse,
        });
    crate::vm::ffi::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    let handle = crate::vm::ffi::http_stream::spawn_legacy_channel_bridge(stream_id);
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
    crate::vm::ffi::http_stream::stream_cancel(stream_id as u64);
    Ok(())
}

/// PLAN-707 T-04: `auto.http.sse_error(stream_id) -> str`——终态错误查询
///（"" = 未终结或无错；Failed = 诊断；Cancelled = "stream cancelled"）。
/// 非消费式，可重复查询。
pub fn shim_http_stream_sse_error(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let _ = vm;
    let stream_id: i64 = task.ram.pop_i32() as i64;
    let err = crate::vm::ffi::http_stream::stream_terminal_error(stream_id as u64)
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

    let stream_id =
        crate::vm::ffi::http_stream::open_stream(crate::vm::ffi::http_stream::StreamOpen {
            url: resolve_http_base_url(&url),
            method: "POST".to_string(),
            body: Some(body),
            headers: headers_map.into_iter().collect(),
            mode: crate::vm::ffi::http_stream::StreamMode::Raw,
        });
    crate::vm::ffi::http_server::register_scope_stream(stream_id);
    task.owned_stream_ids.push(stream_id);
    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// Plan 446 E4: `Http.set_default_header(name, value)` —— 进程级默认请求头
/// (musk: Authorization: Bearer <jwt>)。同名替换。
pub fn shim_http_set_default_header(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let mut reg = http_defaults().lock().unwrap();
    http_set_default_entry(&mut reg.headers, name, value);
    task.ram.push_nv(0);
    Ok(())
}

/// Plan 446 E4: `Http.set_default_query(name, value)` —— 进程级默认 query
/// (musk: workspace=<wid>)。同名替换。
pub fn shim_http_set_default_query(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let value: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let event: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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

    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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

    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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

    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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

    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let api_key: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let api_key: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let body: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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
    let url: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
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

// ---- PLAN-884 T-04：自 stdlib 壳迁入（原样搬移） ----

/// Helper: extract path portion from a string that starts with /
fn extract_path(s: String) -> String {
    let end = s.find(|c: char| c == '?' || c == '#').unwrap_or(s.len());
    if end == 0 {
        "/".to_string()
    } else {
        s[..end].to_string()
    }
}

// ============================================================================
// Net Functions (ID 2100-2199)
// ============================================================================

/// Global handle counter for net resources
pub(crate) static NET_HANDLE_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Plan 312: Global HTTP route table — (method, path_pattern) → fn_name.
/// Populated at VM startup from CompiledPackage.api_routes, consulted by
/// shim_http_server_listen to dispatch requests to VM handler functions.
/// Format: Vec<(method: String, path: String, fn_name: String)>.
static HTTP_ROUTES: std::sync::Mutex<Vec<(String, String, String)>> =
    std::sync::Mutex::new(Vec::new());

/// Plan 312: Register API routes into the global table. Called at VM startup.
pub fn register_http_routes(routes: Vec<(String, String, String)>) {
    if let Ok(mut table) = HTTP_ROUTES.lock() {
        *table = routes;
    }
}

/// Plan 312: Get the current HTTP routes (for testing / introspection).
pub fn get_http_routes() -> Vec<(String, String, String)> {
    HTTP_ROUTES.lock().map(|t| t.clone()).unwrap_or_default()
}

/// Plan 317 §11 Phase 11: Clear the global HTTP routes table.
///
/// Needed for HTTP e2e test isolation: each test calls `crate::run(&code)`,
/// which calls `register_http_routes` (overwrite semantics) — but a server
/// thread from a *previous* test may still be alive (detached) and reading the
/// global table via `serve_async`'s `get_routes()` snapshot. Clearing before
/// each test's run eliminates cross-test route contamination that caused
/// `e2e_sse_indirect_generator` to intermittently return an empty body (404
/// from a stale route snapshot). Test-only helper; production never clears.
pub fn clear_http_routes() {
    if let Ok(mut table) = HTTP_ROUTES.lock() {
        table.clear();
    }
    // Plan 346 5e (B6): also reset the per-IP rate limiter — e2e tests share
    // 127.0.0.1, so a previous test's http.rate_limit(2, 60000) would 429
    // every later test in the same process.
    crate::vm::ffi::http_server::clear_rate_limit();
    // PLAN-669: also drop the #[api] param-sig side channel — a stale entry
    // from a prior test would by-name-bind a same-named handler against the
    // wrong signature.
    crate::vm::ffi::http_server::clear_api_param_sigs();
    // PLAN-729 T-04: also drop idle file-response descriptors — a stale entry
    // from a prior test would let a plain int marshal as a file body.
    if let Ok(mut map) = crate::vm::ffi::http_server_file::VM_FILE_RESPONSES.lock() {
        map.clear();
    }
}

// Thread-local storage for TCP listeners
thread_local! {
    static TCP_LISTENERS: std::cell::RefCell<std::collections::HashMap<u64, StdTcpListener>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
    static TCP_STREAMS: std::cell::RefCell<std::collections::HashMap<u64, StdTcpStream>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// ============================================================================
// HTTP Functions (ID 2200-2299)
// ============================================================================

// HTTP Response data stored in thread-local
thread_local! {
    static HTTP_RESPONSES: std::cell::RefCell<std::collections::HashMap<u64, HttpResponseData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

// PLAN-707 T-04：旧 thread-local HTTP_STREAMS（reqwest::blocking Response
// 表）退役——外部流统一迁入全局 `http_stream::STREAMS` 资源表（异步生产、
// 可取消、有界；见 http_stream.rs 模块注释与 707-stream-decision.md D-3）。
// 流 id 与句柄族同源于 NET_HANDLE_COUNTER，语义键不变。

// ============================================================================
// Plan 341: 异步 HTTP 客户端基础设施（对称 server 的 serve_async）
// ============================================================================
//
// 架构：reqwest async future 是 Send 的（不含 &AutoVM），所以用普通
// tokio::spawn（无需 LocalSet）即可。future 跑异步请求，把结果经
// tokio::sync::mpsc channel 推回。VM 侧迭代器从 Receiver try_recv。
// GLOBAL_RT 是单 worker 多线程 runtime，spawn 的 future 在 await 网络
// 时让出，run_task_loop 得以推进 → 不阻塞 UI。

/// Plan 341: 异步流事件 —— 从异步 HTTP 请求/SSE 流推给 VM 侧的事件。
#[derive(Debug)]
pub enum AsyncStreamEvent {
    /// 一个数据块/响应体（普通请求的完整 body，或 SSE 的一帧）。
    Data(String),
    /// 流结束（对端关闭 / 请求完成）。
    Done,
    /// 出错。
    Error(String),
}

/// Plan 341: 异步流句柄 —— 持有一个 mpsc Receiver，VM 侧迭代器从此拉事件。
pub struct AsyncStreamHandle {
    pub rx: std::sync::Mutex<tokio::sync::mpsc::Receiver<AsyncStreamEvent>>,
    pub done: std::sync::atomic::AtomicBool,
}

// Plan 341/353/349: 统一异步流注册表（原 ASYNC_HTTP_STREAMS + IO_STREAMS 合并）。
// key = stream_id（从 NET_HANDLE_COUNTER 分配，全局唯一，故 HTTP 流与 IO 流合表无冲突）。
// 用 Mutex<HashMap> 而非 thread_local，因为 tokio::spawn 的 future 与 VM
// task 可能在不同异步上下文（虽同线程，但 Mutex 更稳妥）。
lazy_static::lazy_static! {
    pub(crate) static ref ASYNC_STREAMS: std::sync::Mutex<std::collections::HashMap<u64, Arc<AsyncStreamHandle>>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
    // Plan 352: Session storage (in-memory, process-lifetime).
    pub(crate) static ref SESSIONS: std::sync::Mutex<std::collections::HashMap<String, String>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
    // Plan 352: Middleware chain — list of VM fn addresses to execute before handler.
    pub static ref MIDDLEWARE_CHAIN: std::sync::Mutex<Vec<String>> =
        std::sync::Mutex::new(Vec::new());
    // Plan 352: Template engine — compiled templates (name → template string).
    pub(crate) static ref TEMPLATES: std::sync::Mutex<std::collections::HashMap<String, String>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

/// Plan 341: 分配一个新的异步流 id。
pub fn alloc_async_id() -> u64 {
    NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// PLAN-698 T-02/SD-02: VM 侧进程内 SSE 事件总线——生成 Axum 工程
/// events.rs 模板（api_gen::generate_events_rs）的运行面镜像：同款
/// `broadcast::channel(256)`，发送失败（无订阅者）忽略。
pub(crate) static EVENT_BUS: std::sync::LazyLock<tokio::sync::broadcast::Sender<String>> =
    std::sync::LazyLock::new(|| {
        let (tx, _rx) = tokio::sync::broadcast::channel(256);
        tx
    });

/// 向 VM 事件总线广播一条 SSE 事件 JSON（http_server 的 POST publisher
/// 臂调用；对齐生成侧 `crate::events::broadcast`）。
pub fn bus_broadcast(json: String) {
    let _ = EVENT_BUS.send(json);
}

/// 测试专用：领取一个总线订阅者（http_server publisher 臂单测收流用）。
#[cfg(test)]
pub fn bus_subscribe_for_test() -> tokio::sync::broadcast::Receiver<String> {
    EVENT_BUS.subscribe()
}

// Plan 195: RequestBuilder data storage
#[derive(Debug, Clone)]
struct HttpRequestBuilderData {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
    // Plan 350: TLS configuration
    ca_cert_path: Option<String>,
    skip_verify: bool,
    client_cert_path: Option<String>,
    client_key_path: Option<String>,
    // Plan 349: Multipart file upload
    multipart_files: Vec<(String, String)>, // (field_name, file_path)
    multipart_texts: Vec<(String, String)>, // (field_name, value)
    // Plan 349 step 8: Cookie / retry / compression
    cookie_store: bool,
    retry_count: u32,
    gzip: bool,
    brotli: bool, // Plan 349 步骤 7/8 (W4)
    // PLAN-727 T-05：multipart 预算越界（字段数/文本总量）在 builder 期
    // 记录，send 时终结性失败（坏请求不发出）。
    build_error: Option<String>,
}

/// HTTP Response data
#[derive(Debug, Clone, Default)]
struct HttpResponseData {
    status: u16,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

/// Must be called from a non-tokio thread (std::thread::spawn), because it
/// uses blocking_lock() on handler tasks.
pub fn run_http_server_blocking(vm: &AutoVM, addr: &str) {
    use std::io::{BufRead, Read, Write};
    use std::net::TcpListener;

    let listener = match TcpListener::bind(addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[HTTP] Server bind failed on {}: {}", addr, e);
            return;
        }
    };
    eprintln!("[HTTP] Server listening on {}", addr);

    let routes = crate::vm::ffi::http_server::get_routes();

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
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Read headers
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

        // Read body
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
        let route_match = match crate::vm::ffi::http_server::match_route(
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

        // Call VM handler on a fresh task
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) =
            vm.tasks.get(&handler_task_id)
        {
            // blocking_lock is safe: we're on a std::thread, NOT in tokio context
            let mut ht = handler_task_arc.blocking_lock();

            // PLAN-669: by-name binding (shared with the other serve sites);
            // bind failures → 400/500 instead of calling with wrong args.
            let n_args = match crate::vm::ffi::http_server::bind_api_args_or_legacy(
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
                        crate::vm::ffi::http_server::ApiArgBindError::BadRequest(m) => {
                            ("400 Bad Request", m)
                        }
                        crate::vm::ffi::http_server::ApiArgBindError::Internal(m) => {
                            ("500 Internal Server Error", m)
                        }
                    };
                    eprintln!("[HTTP] {} {} → {} ({})", req_method, req_path, status, msg);
                    let err_body = format!(
                        "{{\"error\":{}}}",
                        crate::vm::ffi::http_server::json_escape_string(&msg)
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
                    let nv = ht.ram.pop_nv();
                    // Plan 326 Phase 3: delegate to shared serializer (handles
                    // struct/array/Option return values, not just string/i32/null).
                    crate::vm::ffi::http_server::nv_to_json(vm, nv, 0)
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
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            status, body_json.len(), body_json
        );
        let _ = stream.write_all(response.as_bytes());
    }
}

/// Plan 346 3c: server-side lookup of a pending response by handle. Lets the
/// HTTP server serve a handler-returned Response object (status/headers/body,
/// e.g. from `http.response.redirect(url, 302)` or `http.response()` +
/// `response_status`) instead of JSON-encoding the raw handle integer.
/// Returns None for ints that are not response handles.
pub fn lookup_http_response(handle: u64) -> Option<(u16, Vec<(String, String)>, Vec<u8>)> {
    HTTP_RESPONSES.with(|r| {
        r.borrow()
            .get(&handle)
            .map(|res| (res.status, res.headers.clone(), res.body.clone()))
    })
}

/// Plan 442 C2: allocate + store a structured HTTP response and return its
/// handle. The http_server's response-object path (`lookup_http_response`)
/// recognizes the handle and writes status/headers/body directly, instead of
/// JSON-encoding the raw handle. Mirrors `shim_http_response_redirect`'s
/// allocation convention (NET_HANDLE_COUNTER + thread-local HTTP_RESPONSES).
pub fn insert_http_response(status: u16, headers: Vec<(String, String)>, body: Vec<u8>) -> u64 {
    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    HTTP_RESPONSES.with(|r| {
        r.borrow_mut().insert(
            handle,
            HttpResponseData {
                status,
                headers,
                body,
            },
        );
    });
    handle
}

// ============================================================================
// Plan 352: Session management natives
// ============================================================================

// ============================================================================
// Plan 352: OpenAPI auto-generation
// ============================================================================

/// Plan 352: `openapi.generate() -> String`
/// Generate OpenAPI 3.0 JSON spec from registered #[api] routes.
pub fn shim_openapi_generate(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let routes = get_http_routes();
    let mut paths = serde_json::Map::new();

    for (method, path, fn_name) in &routes {
        let method_lower = method.to_lowercase();
        let openapi_path = convert_path_to_openapi(path);

        let path_entry = paths
            .entry(openapi_path)
            .or_insert_with(|| serde_json::json!({}));

        let summary = fn_name.replace('_', " ");

        let mut operation = serde_json::json!({
            "summary": summary,
            "operationId": fn_name,
            "responses": {
                "200": {"description": "OK"}
            }
        });

        // Extract path parameters.
        let mut params = Vec::new();
        for segment in path.split('/') {
            if let Some(param_name) = segment.strip_prefix(':') {
                params.push(serde_json::json!({
                    "name": param_name,
                    "in": "path",
                    "required": true,
                    "schema": {"type": "integer"}
                }));
            }
        }
        if !params.is_empty() {
            operation["parameters"] = serde_json::Value::Array(params);
        }

        if method_lower == "post" || method_lower == "put" {
            operation["requestBody"] = serde_json::json!({
                "content": {
                    "application/json": {"schema": {"type": "object"}}
                }
            });
        }

        if let Some(obj) = path_entry.as_object_mut() {
            obj.insert(method_lower, operation);
        }
    }

    let spec = serde_json::json!({
        "openapi": "3.0.0",
        "info": {"title": "Auto API", "version": "1.0.0"},
        "paths": serde_json::Value::Object(paths)
    });

    let json_str = serde_json::to_string_pretty(&spec).unwrap_or_default();
    let idx = vm.add_string(json_str.into_bytes());
    vm.rc_push_str_idx(task, idx);
    Ok(())
}

/// Convert `/api/notes/:id` → `/api/notes/{id}` for OpenAPI.
fn convert_path_to_openapi(path: &str) -> String {
    let mut result = String::new();
    let mut chars = path.chars().peekable();
    while let Some(c) = chars.next() {
        if c == ':' {
            let param: String = chars
                .by_ref()
                .take_while(|ch| ch.is_alphanumeric() || *ch == '_')
                .collect();
            if !param.is_empty() {
                result.push('{');
                result.push_str(&param);
                result.push('}');
            } else {
                result.push(':');
            }
        } else {
            result.push(c);
        }
    }
    result
}

// ============================================================================
// Plan 352: Template engine (SSR) natives
// ============================================================================

/// `template.compile(name: String, template_str: String)`
/// Compile and store a template by name. Templates support:
/// {{var}} — variable interpolation from JSON data
/// {{#if key}}...{{/if}} — conditional block (key truthy)
/// {{#each key}}...{{/each}} — loop over array
pub fn shim_template_compile(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let template_str: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    if let Ok(mut templates) = TEMPLATES.lock() {
        templates.insert(name, template_str);
    }
    Ok(())
}

/// `template.render(name: String, data_json: String) -> String`
/// Render a compiled template with JSON data. Returns HTML string.
pub fn shim_template_render(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data_json: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let name: String = crate::vm::ffi::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let template_str = TEMPLATES
        .lock()
        .ok()
        .and_then(|t| t.get(&name).cloned())
        .unwrap_or_default();

    let data: serde_json::Value =
        serde_json::from_str(&data_json).unwrap_or(serde_json::Value::Null);
    let rendered = render_template(&template_str, &data);

    let idx = vm.add_string(rendered.into_bytes());
    vm.rc_push_str_idx(task, idx);
    Ok(())
}

/// Simple template renderer: {{var}}, {{#if key}}...{{/if}}, {{#each key}}...{{/each}}.
fn render_template(template: &str, data: &serde_json::Value) -> String {
    let mut result = String::new();
    let mut remaining = template;

    while !remaining.is_empty() {
        if let Some(end) = remaining.find("{{") {
            result.push_str(&remaining[..end]);
            remaining = &remaining[end + 2..];

            if remaining.starts_with('#') {
                // Block tag: #if or #each
                if let Some(close) = remaining.find("}}") {
                    let tag = &remaining[1..close]; // e.g. "if key" or "each key"
                    remaining = &remaining[close + 2..];

                    if let Some(key) = tag.strip_prefix("if ") {
                        // Find matching {{/if}}
                        let close_tag = "{{/if}}";
                        if let Some(block_end) = remaining.find(close_tag) {
                            let block_content = &remaining[..block_end];
                            remaining = &remaining[block_end + close_tag.len()..];
                            // Check condition
                            let cond = get_json_value(data, key.trim());
                            if is_truthy(&cond) {
                                result.push_str(&render_template(block_content, data));
                            }
                        }
                    } else if let Some(key) = tag.strip_prefix("each ") {
                        let close_tag = "{{/each}}";
                        if let Some(block_end) = remaining.find(close_tag) {
                            let block_content = &remaining[..block_end];
                            remaining = &remaining[block_end + close_tag.len()..];
                            // Iterate array
                            let arr = get_json_value(data, key.trim());
                            if let Some(items) = arr.as_array() {
                                for item in items {
                                    let mut item_data = data.clone();
                                    if let Some(obj) = item_data.as_object_mut() {
                                        obj.insert("this".to_string(), item.clone());
                                    } else {
                                        item_data = serde_json::json!({"this": item});
                                    }
                                    result.push_str(&render_template(block_content, &item_data));
                                }
                            }
                        }
                    } else {
                        // Unknown block — skip to }}
                        if let Some(skip) = remaining.find("}}") {
                            remaining = &remaining[skip + 2..];
                        }
                    }
                }
            } else if let Some(close) = remaining.find("}}") {
                // Variable: {{var}}
                let var = &remaining[..close];
                remaining = &remaining[close + 2..];
                let val = get_json_value(data, var.trim());
                match &val {
                    serde_json::Value::String(s) => result.push_str(s),
                    serde_json::Value::Number(n) => result.push_str(&n.to_string()),
                    serde_json::Value::Bool(b) => result.push_str(&b.to_string()),
                    _ => {}
                }
            } else {
                result.push_str("{{");
                result.push_str(remaining);
                break;
            }
        } else {
            result.push_str(remaining);
            break;
        }
    }
    result
}

/// Get a nested JSON value by dotted key (e.g. "user.name" or "this.title").
fn get_json_value(data: &serde_json::Value, key: &str) -> serde_json::Value {
    let mut current = data;
    for part in key.split('.') {
        current = current.get(part).unwrap_or(&serde_json::Value::Null);
    }
    current.clone()
}

/// Check if a JSON value is truthy (non-null, non-false, non-empty, non-zero).
fn is_truthy(val: &serde_json::Value) -> bool {
    match val {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(true),
        serde_json::Value::String(s) => !s.is_empty(),
        serde_json::Value::Array(a) => !a.is_empty(),
        serde_json::Value::Object(o) => !o.is_empty(),
    }
}

/// PLAN-776 R1-F01：Http.ok/created/bad_request/not_found/internal_error 五个
/// 真实服务端 Response producer 从"裸 i64 句柄 + HTTP_RESPONSES 表"迁移为
/// **凭据对象**（owned 快照，marshal 资源臂按值分派）。查表命中制退役后
/// 裸 id 不再具有资源语义；inventory 契约条目保持（签名面零漂移）。
fn text_plain_response(status: u16, body: String) -> (u16, Vec<(String, String)>, Vec<u8>) {
    (
        status,
        vec![(
            "Content-Type".to_string(),
            "text/plain; charset=utf-8".to_string(),
        )],
        body.into_bytes(),
    )
}

inventory::submit! {
    crate::vm::ffi::StaticFFIRegistration {
        name: "Http.ok",
        shim: shim_http_ok,
        parameters: &["String"],
        returns: "i64",
        producer: "auto_lang::vm::ffi::stdlib::shim_http_ok",
    }
}
inventory::submit! {
    crate::vm::ffi::StaticFFIRegistration {
        name: "Http.created",
        shim: shim_http_created,
        parameters: &["String"],
        returns: "i64",
        producer: "auto_lang::vm::ffi::stdlib::shim_http_created",
    }
}
inventory::submit! {
    crate::vm::ffi::StaticFFIRegistration {
        name: "Http.bad_request",
        shim: shim_http_bad_request,
        parameters: &["String"],
        returns: "i64",
        producer: "auto_lang::vm::ffi::stdlib::shim_http_bad_request",
    }
}
inventory::submit! {
    crate::vm::ffi::StaticFFIRegistration {
        name: "Http.not_found",
        shim: shim_http_not_found,
        parameters: &["String"],
        returns: "i64",
        producer: "auto_lang::vm::ffi::stdlib::shim_http_not_found",
    }
}
inventory::submit! {
    crate::vm::ffi::StaticFFIRegistration {
        name: "Http.internal_error",
        shim: shim_http_internal_error,
        parameters: &["String"],
        returns: "i64",
        producer: "auto_lang::vm::ffi::stdlib::shim_http_internal_error",
    }
}

/// `RequestBuilder.multipart_file(field_name: String, file_path: String) -> RequestBuilder`
/// Attach a file to a multipart form upload.
/// PLAN-727 T-05：multipart 文件 part 流式化——保持路径描述直到发送；
/// 每次构建重开文件、有限块读取（64KiB、通道深 2 = 待处理块上界）。
/// 缺失/打开失败 = 立即出错的流（in-band body error → 请求终结），配合
/// `file_errors` 消息合并；绝不产生空 part 假成功。
fn stream_multipart_file_part(
    path: &str,
    file_errors: &std::sync::Arc<std::sync::Mutex<Vec<String>>>,
) -> reqwest::multipart::Part {
    const READ_BLOCK: usize = 64 * 1024;
    struct PartStream {
        rx: tokio::sync::mpsc::Receiver<Vec<u8>>,
    }
    impl futures::Stream for PartStream {
        type Item = Result<Vec<u8>, std::io::Error>;
        fn poll_next(
            mut self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Option<Self::Item>> {
            match self.rx.poll_recv(cx) {
                std::task::Poll::Ready(Some(v)) => std::task::Poll::Ready(Some(Ok(v))),
                std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
                std::task::Poll::Pending => std::task::Poll::Pending,
            }
        }
    }
    fn erroring_part(msg: String) -> reqwest::multipart::Part {
        // 首项即 Err：reqwest 发送失败（body error），不产生截断假成功。
        let stream = futures::stream::once(async move {
            Err::<Vec<u8>, std::io::Error>(std::io::Error::other(msg))
        });
        reqwest::multipart::Part::stream(reqwest::Body::wrap_stream(stream))
    }
    let meta = match std::fs::metadata(path) {
        Ok(m) if m.is_file() => m,
        Ok(_) => {
            let msg = format!("multipart file is not a regular file: {path}");
            file_errors.lock().unwrap().push(msg.clone());
            return erroring_part(msg);
        }
        Err(e) => {
            let msg = format!("multipart file open failed: {path}: {e}");
            file_errors.lock().unwrap().push(msg.clone());
            return erroring_part(msg);
        }
    };
    let len = meta.len();
    let (tx, rx) = tokio::sync::mpsc::channel::<Vec<u8>>(2);
    let errors = std::sync::Arc::clone(file_errors);
    let path_owned = path.to_string();
    crate::vm::ffi::async_http::client_runtime().spawn(async move {
        let mut file = match tokio::fs::File::open(&path_owned).await {
            Ok(f) => f,
            Err(e) => {
                errors
                    .lock()
                    .unwrap()
                    .push(format!("multipart file open failed: {path_owned}: {e}"));
                return; // 流提前结束 → 长度失配 → 发送失败
            }
        };
        use tokio::io::AsyncReadExt;
        let mut buf = vec![0u8; READ_BLOCK];
        loop {
            match file.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).await.is_err() {
                        return; // 消费端消失（请求失败/取消）
                    }
                }
                Err(e) => {
                    errors
                        .lock()
                        .unwrap()
                        .push(format!("multipart file read failed: {path_owned}: {e}"));
                    return; // 不发尾：长度失配 → 发送失败
                }
            }
        }
    });
    let body = reqwest::Body::wrap_stream(PartStream { rx });
    reqwest::multipart::Part::stream_with_length(body, len)
}

/// PLAN-727 T-05：multipart 预算（决策报告冻结）：文件 part ≤16、文本字段
/// ≤64、单值 ≤64KiB、文本总量 ≤1MiB；越界在 builder 期记录，send 终结。
const MULTIPART_MAX_FILES: usize = 16;
const MULTIPART_MAX_TEXT_FIELDS: usize = 64;
const MULTIPART_MAX_TEXT_VALUE: usize = 64 * 1024;
const MULTIPART_MAX_TEXT_TOTAL: usize = 1024 * 1024;

// ============================================================================
// Plan 349: File download + resume + progress natives
// ============================================================================

/// Plan 349: Spawn a download with progress reporting (PLAN-727 T-05 起
/// 经共享传输核心执行，观察句柄泵进度/终态进既有通道)。
fn spawn_download_with_progress(
    url: String,
    file_path: String,
    _stream_id: u64,
) -> Arc<AsyncStreamHandle> {
    // PLAN-727 T-05：进度生产者迁移共享传输核心——专用线程/独立
    // current-thread runtime 退役；同步 write_all 退役（核心内增量落盘 +
    // staging 提交）；进度经观察句柄在内核 runtime 上泵入既有通道
    //（try_send 满则合并丢弃 = 慢消费者不反压落盘；终态不丢）。
    let (tx, rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);
    let handle = Arc::new(AsyncStreamHandle {
        rx: std::sync::Mutex::new(rx),
        done: std::sync::atomic::AtomicBool::new(false),
    });
    crate::vm::ffi::http_transfer::spawn_core_download_with_progress(
        &url,
        &file_path,
        tx,
        handle.clone(),
    );
    handle
}
// ============================================================================
// Plan 195: Enhanced Response access methods
// ============================================================================

/// PLAN-776 R1-F06：Response 载荷统一读取——**对象凭据**（本 VM owner 校验
/// 后读快照）或 **client int 句柄**（HTTP_RESPONSES 表——http.get 产句柄的
/// 访问器面，FFI 资源范围保留表语义）。普通 int 未命中表 = None（调用方
/// 按原语义降级，不猜测）。
fn read_response_payload(
    vm: &AutoVM,
    res_nv: auto_val::NanoValue,
) -> Option<crate::vm::ffi::http_return_resource::ResponseSnapshot> {
    if auto_val::is_object(res_nv) {
        return crate::vm::ffi::http_return_resource::read_response_snapshot(vm, res_nv);
    }
    if auto_val::is_i32(res_nv) {
        let h = auto_val::decode_i32(res_nv) as i64;
        return HTTP_RESPONSES.with(|r| {
            r.borrow().get(&(h as u64)).map(|res| {
                crate::vm::ffi::http_return_resource::ResponseSnapshot {
                    status: res.status,
                    headers: res.headers.clone(),
                    body: res.body.clone(),
                }
            })
        });
    }
    None
}

// ============================================================================
// PLAN-707 T-04：真正的流式 HTTP 函数（异步非阻塞形态）
// ============================================================================
//
// 签名与哨兵逐字节保留（决策 D-4）：get/post(+headers) → 句柄 i64；
// next → str（EOF=`[DONE]`）；is_done → 0/1；close → void。
// 变化在内部：建立与读取全部经 `http_stream` 资源表（异步生产者、
// 独立许可、可取消），零阻塞线程、零 blocking read。

/// stream_next 的 pull→推值主体（首调/重入共用）。
fn push_stream_next_value(task: &mut AutoTask, vm: &AutoVM, handle: i64) -> Result<(), VMError> {
    match crate::vm::ffi::http_stream::stream_pull(handle as u64) {
        crate::vm::ffi::http_stream::Pull::Data(text) => {
            text.push_to_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
            Ok(())
        }
        crate::vm::ffi::http_stream::Pull::Eof => {
            "[DONE]"
                .to_string()
                .push_to_stack(task, vm)
                .map_err(|e| VMError::RuntimeError(e.to_string()))?;
            Ok(())
        }
        crate::vm::ffi::http_stream::Pull::Failed(e) => {
            Err(VMError::RuntimeError(format!("Stream read error: {}", e)))
        }
        crate::vm::ffi::http_stream::Pull::Pending | crate::vm::ffi::http_stream::Pull::Opening => {
            task.waiting_http_stream_id = Some(handle as u64);
            task.status = crate::vm::task::TaskStatus::Waiting("http-stream".into());
            Ok(())
        }
    }
}

/// PLAN-698 T-02/SD-02: `auto.bus.subscribe() -> iterator_id`（VM 运行面）
///
/// 进程内 SSE 事件总线（bridge 案①：进程内 iterator 注册表）——与生成
/// Axum 侧 events.rs 模板同构（`broadcast::channel(256)`，发送失败忽略）。
/// 每个 subscribe 领一个 broadcast Receiver，由专属转发线程泵进统一
/// ASYNC_STREAMS 表，再以既有 `Iterator::AsyncHttpStream` 臂被 SSE serve
/// 循环与 .at 侧 iterator 家族消费（断连/停端=696 回收语义：Done→-1）。
pub fn shim_bus_subscribe(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let stream_id = alloc_async_id();
    let (tx, rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);
    let handle = Arc::new(AsyncStreamHandle {
        rx: std::sync::Mutex::new(rx),
        done: std::sync::atomic::AtomicBool::new(false),
    });
    // 线程只持 Weak——强引用归 ASYNC_STREAMS 独占：cleanup 移除句柄
    // （rx drop）即 `blocking_send` 失败；Weak 升级失败即线程收割位。
    let handle_weak = Arc::downgrade(&handle);
    let mut bus_rx = EVENT_BUS.subscribe();

    // F-1 转发线程（终形）：有界轮询（150ms）——`blocking_recv` 会睡过
    // 收割时机（无后续广播时线程永不醒，实机 LEAK 实证），try_recv 轮询
    // 使 cleanup 后线程在 ~150ms 内自检退出。Lagged 跳过（慢消费者丢帧
    // 不断流）；Closed（总线消失，防御臂）结束流。事件语义不变：
    // Data 直通、Done 收尾（镜像生成侧"任何 RecvError 终止"）。
    std::thread::Builder::new()
        .name("auto-bus-subscribe".into())
        .spawn(move || {
            use tokio::sync::broadcast::error::TryRecvError;
            loop {
                if handle_weak.upgrade().is_none() {
                    break; // 句柄已被 cleanup 回收——线程收割
                }
                match bus_rx.try_recv() {
                    Ok(json) => {
                        if tx.blocking_send(AsyncStreamEvent::Data(json)).is_err() {
                            break; // 消费端已丢弃（连接关闭）——线程退出
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                    Err(TryRecvError::Lagged(_)) => continue, // 慢消费者丢帧继续
                    Err(TryRecvError::Closed) => break,       // 总线消失——流终止
                }
            }
            let _ = tx.blocking_send(AsyncStreamEvent::Done);
        })
        .expect("spawn bus-subscribe thread");

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let iter_id = vm.register_iterator(crate::vm::engine::Iterator::AsyncHttpStream(
        crate::vm::engine::AsyncStreamIterator {
            stream_id,
            done: false,
        },
    ));
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

// ============================================================================
// Plan 152: SSE Parser Functions
// ============================================================================

/// Parse SSE text chunk
/// Returns: Array of SSE event objects (JSON strings)
#[auto_macros::rust_fn("parse_sse")]
pub fn shim_sse_parse(chunk: String) -> Vec<String> {
    // Simple SSE parser implementation
    let mut events = Vec::new();
    let mut current_data = String::new();
    let mut current_event = String::new();
    let mut current_id = String::new();

    for line in chunk.lines() {
        // Skip comment lines
        if line.starts_with(':') {
            continue;
        }

        if line.is_empty() {
            // Empty line means event end
            if !current_data.is_empty() {
                let mut json = String::from("{\"data\":\"");
                json.push_str(&current_data.replace('"', "\\\""));
                json.push_str("\"");
                if !current_event.is_empty() {
                    json.push_str(",\"event\":\"");
                    json.push_str(&current_event);
                    json.push_str("\"");
                }
                if !current_id.is_empty() {
                    json.push_str(",\"id\":\"");
                    json.push_str(&current_id);
                    json.push_str("\"");
                }
                json.push_str("}");
                events.push(json);
            }
            current_data.clear();
            current_event.clear();
            current_id.clear();
        } else if line.starts_with("data:") {
            let data = &line[5..].trim_start();
            if !current_data.is_empty() {
                current_data.push('\n');
            }
            current_data.push_str(data);
        } else if line.starts_with("event:") {
            current_event = line[6..].trim().to_string();
        } else if line.starts_with("id:") {
            current_id = line[3..].trim().to_string();
        }
    }

    // Don't forget the last event if no trailing newline
    if !current_data.is_empty() {
        let mut json = String::from("{\"data\":\"");
        json.push_str(&current_data.replace('"', "\\\""));
        json.push_str("\"");
        if !current_event.is_empty() {
            json.push_str(",\"event\":\"");
            json.push_str(&current_event);
            json.push_str("\"");
        }
        if !current_id.is_empty() {
            json.push_str(",\"id\":\"");
            json.push_str(&current_id);
            json.push_str("\"");
        }
        json.push_str("}");
        events.push(json);
    }

    events
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_net_tcp_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Net/TCP functions (manual shims — use heap objects for TCP state)
    // Note: registry uses underscores in method portion (auto.net.tcp_bind, not auto.net.tcp.bind)
    natives.register_typed_shim_by_name(
        "auto.net.tcp_bind",
        shim_net_tcp_bind,
        &["str"],
        "TcpListener?",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_listener_accept",
        shim_net_tcp_listener_accept,
        "TcpListener",
        &["TcpListener"],
        "TcpStream?",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_listener_close",
        shim_net_tcp_listener_close,
        "TcpListener",
        &["TcpListener"],
        "void",
    );
    natives.register_typed_shim_by_name(
        "auto.net.tcp_connect",
        shim_net_tcp_connect,
        &["str"],
        "TcpStream?",
    );
    natives.register_shim_by_name("auto.net.tcp_stream_read", shim_net_tcp_stream_read);
    natives.register_method_shim_by_name(
        "auto.net.tcp_stream_write",
        shim_net_tcp_stream_write,
        "TcpStream",
        &["TcpStream", "[]byte"],
        "int",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_stream_read_all",
        shim_net_tcp_stream_read_all,
        "TcpStream",
        &["TcpStream"],
        "[]byte",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_stream_write_str",
        shim_net_tcp_stream_write_str,
        "TcpStream",
        &["TcpStream", "str"],
        "int",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_stream_close",
        shim_net_tcp_stream_close,
        "TcpStream",
        &["TcpStream"],
        "void",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_stream_set_read_timeout",
        shim_net_tcp_stream_set_read_timeout,
        "TcpStream",
        &["TcpStream", "int"],
        "void",
    );
    natives.register_method_shim_by_name(
        "auto.net.tcp_stream_set_write_timeout",
        shim_net_tcp_stream_set_write_timeout,
        "TcpStream",
        &["TcpStream", "int"],
        "void",
    );
    // Plan 313: TCP flush + nodelay for SSE/low-latency writes
    natives.register_shim_by_name("auto.net.tcp_stream_flush", shim_net_tcp_stream_flush);
    natives.register_shim_by_name(
        "auto.net.tcp_stream_set_nodelay",
        shim_net_tcp_stream_set_nodelay,
    );
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_http_server_shims(natives: &mut crate::vm::native::NativeInterface) {
    // HTTP server functions (manual shims — heap objects for server state)
    natives.register_shim_by_name("auto.http.server", shim_http_server);
    natives.register_shim_by_name("auto.http.server_get", shim_http_server_get);
    natives.register_shim_by_name("auto.http.server_post", shim_http_server_post);
    natives.register_shim_by_name("auto.http.server_put", shim_http_server_put);
    natives.register_shim_by_name("auto.http.server_delete", shim_http_server_delete);
    natives.register_shim_by_name("auto.http.server_static", shim_http_server_static);
    natives.register_shim_by_name("auto.http.server_listen", shim_http_server_listen);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_http_server_feature_shims(natives: &mut crate::vm::native::NativeInterface) {
    // auto-os Plan 013 T2: AutoTerm 引擎桥(auto.term.*)注册走 native_catalog
    // 静态表(ID 2943-2949,register_std_shims 自动绑 shim + canonical 名,
    // PLAN-057 同款"codegen 任意时刻可解析"口径)——shim 实现在
    // vm/ffi/term_engine.rs,此处不再手工登记。
    natives.register_shim_by_name("auto.http.response", shim_http_response);
    natives.register_shim_by_name("auto.http.response_status", shim_http_response_status);
    natives.register_shim_by_name("auto.http.response_header", shim_http_response_header);
    natives.register_shim_by_name("auto.http.response_text", shim_http_response_text);
    natives.register_shim_by_name("auto.http.response_html", shim_http_response_html);
    natives.register_shim_by_name("auto.http.response_bytes", shim_http_response_bytes);
    // Plan 346: Redirect
    natives.register_shim_by_name("auto.http.response.redirect", shim_http_response_redirect);
    natives.register_shim_by_name("auto.http.response_redirect", shim_http_response_redirect);
    natives.register_shim_by_name("http.response.redirect", shim_http_response_redirect);
    // Plan 346 5e (B6): Rate limiting
    natives.register_shim_by_name("auto.http.rate_limit", shim_http_rate_limit);
    natives.register_shim_by_name("http.rate_limit", shim_http_rate_limit);
    // Plan 352: Session management
    natives.register_shim_by_name("auto.session.create", shim_session_create);
    natives.register_shim_by_name("session.create", shim_session_create);
    natives.register_shim_by_name("auto.session.get", shim_session_get);
    natives.register_shim_by_name("session.get", shim_session_get);
    natives.register_shim_by_name("auto.session.set", shim_session_set);
    natives.register_shim_by_name("session.set", shim_session_set);
    natives.register_shim_by_name("auto.session.destroy", shim_session_destroy);
    natives.register_shim_by_name("session.destroy", shim_session_destroy);
    // Plan 352: Middleware chain
    natives.register_shim_by_name("auto.http.server_use", shim_http_server_use);
    natives.register_shim_by_name("http.server.use", shim_http_server_use);
    // 'use' is a keyword in Auto, so provide 'middleware' alias.
    natives.register_shim_by_name("auto.http.server_middleware", shim_http_server_use);
    natives.register_shim_by_name("http.server.middleware", shim_http_server_use);
    // Plan 352: Template engine (SSR)
    natives.register_shim_by_name("auto.template.compile", shim_template_compile);
    natives.register_shim_by_name("template.compile", shim_template_compile);
    natives.register_shim_by_name("auto.template.render", shim_template_render);
    natives.register_shim_by_name("template.render", shim_template_render);
    // Plan 352: OpenAPI auto-generation
    natives.register_shim_by_name("auto.openapi.generate", shim_openapi_generate);
    natives.register_shim_by_name("openapi.generate", shim_openapi_generate);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_http_client_builder_shims(natives: &mut crate::vm::native::NativeInterface) {
    // HTTP client functions (manual shims — heap objects for request/response)
    natives.register_shim_by_name("auto.http.get", shim_http_get);
    // PLAN-083 T-01: 异步消息桥 GET（fire-and-forget + 完成消息回填）。
    natives.register_shim_by_name("auto.http.get_msg", shim_http_get_msg);
    natives.register_shim_by_name("http.get_msg", shim_http_get_msg);
    natives.register_shim_by_name("auto.http.set_default_header", shim_http_set_default_header);
    natives.register_shim_by_name("auto.http.set_default_query", shim_http_set_default_query);
    natives.register_shim_by_name("auto.http.clear_default_auth", shim_http_clear_default_auth);
    natives.register_shim_by_name("auto.http.post", shim_http_post);
    natives.register_shim_by_name("auto.http.put", shim_http_put);
    natives.register_shim_by_name("auto.http.delete", shim_http_delete);
    natives.register_shim_by_name("auto.http.request", shim_http_request);
    natives.register_shim_by_name(
        "auto.http.request_builder_header",
        shim_request_builder_header,
    );
    natives.register_shim_by_name("auto.http.request_builder_body", shim_request_builder_body);
    natives.register_shim_by_name(
        "auto.http.request_builder_timeout",
        shim_request_builder_timeout,
    );
    natives.register_shim_by_name("auto.http.request_builder_json", shim_request_builder_json);
    natives.register_shim_by_name("auto.http.request_builder_send", shim_request_builder_send);

    // RequestBuilder method aliases for CALL_SPEC dispatch
    // When CALL_SPEC detects type "RequestBuilder", it looks up "RequestBuilder.method"
    natives.register_shim_by_name("RequestBuilder.header", shim_request_builder_header);
    natives.register_shim_by_name("RequestBuilder.body", shim_request_builder_body);
    natives.register_shim_by_name("RequestBuilder.timeout", shim_request_builder_timeout);
    natives.register_shim_by_name("RequestBuilder.json", shim_request_builder_json);
    natives.register_shim_by_name("RequestBuilder.send", shim_request_builder_send);
    // Plan 350: TLS configuration
    natives.register_shim_by_name(
        "RequestBuilder.tls_ca_cert",
        shim_request_builder_tls_ca_cert,
    );
    natives.register_shim_by_name(
        "RequestBuilder.tls_skip_verify",
        shim_request_builder_tls_skip_verify,
    );
    natives.register_shim_by_name(
        "RequestBuilder.tls_client_cert",
        shim_request_builder_tls_client_cert,
    );
    // Plan 349: Multipart file upload
    natives.register_shim_by_name(
        "RequestBuilder.multipart_file",
        shim_request_builder_multipart_file,
    );
    natives.register_shim_by_name(
        "RequestBuilder.multipart_text",
        shim_request_builder_multipart_text,
    );
    // Plan 349 step 8: Cookie / retry / compression
    natives.register_shim_by_name(
        "RequestBuilder.cookie_store",
        shim_request_builder_cookie_store,
    );
    natives.register_shim_by_name("RequestBuilder.retry", shim_request_builder_retry);
    natives.register_shim_by_name("RequestBuilder.gzip", shim_request_builder_gzip);
    natives.register_shim_by_name("RequestBuilder.brotli", shim_request_builder_brotli);
    natives.register_shim_by_name("auto.http.upload", shim_http_upload);
    natives.register_shim_by_name("http.upload", shim_http_upload);
    // Plan 349: File download + resume + progress
    natives.register_shim_by_name("auto.http.download", shim_http_download);
    natives.register_shim_by_name("http.download", shim_http_download);
    natives.register_shim_by_name("auto.http.download_resume", shim_http_download_resume);
    natives.register_shim_by_name("http.download_resume", shim_http_download_resume);
    natives.register_shim_by_name(
        "auto.http.download_with_progress",
        shim_http_download_with_progress,
    );
    natives.register_shim_by_name(
        "http.download_with_progress",
        shim_http_download_with_progress,
    );
    // PLAN-727 T-05: 可取消文件传输公共面（共享核心；契约见
    // docs/plans/reports/727-transfer-decision.md）
    natives.register_shim_by_name(
        "auto.http.transfer_download",
        crate::vm::ffi::http_transfer::shim_http_transfer_download,
    );
    natives.register_shim_by_name(
        "http.transfer_download",
        crate::vm::ffi::http_transfer::shim_http_transfer_download,
    );
    natives.register_shim_by_name(
        "auto.http.transfer_upload",
        crate::vm::ffi::http_transfer::shim_http_transfer_upload,
    );
    natives.register_shim_by_name(
        "http.transfer_upload",
        crate::vm::ffi::http_transfer::shim_http_transfer_upload,
    );
    natives.register_shim_by_name(
        "auto.http.transfer_wait",
        crate::vm::ffi::http_transfer::shim_http_transfer_wait,
    );
    natives.register_shim_by_name(
        "http.transfer_wait",
        crate::vm::ffi::http_transfer::shim_http_transfer_wait,
    );
    natives.register_shim_by_name(
        "auto.http.transfer_next_progress",
        crate::vm::ffi::http_transfer::shim_http_transfer_next_progress,
    );
    natives.register_shim_by_name(
        "http.transfer_next_progress",
        crate::vm::ffi::http_transfer::shim_http_transfer_next_progress,
    );
    natives.register_shim_by_name(
        "auto.http.transfer_cancel",
        crate::vm::ffi::http_transfer::shim_http_transfer_cancel,
    );
    natives.register_shim_by_name(
        "http.transfer_cancel",
        crate::vm::ffi::http_transfer::shim_http_transfer_cancel,
    );
    natives.register_shim_by_name(
        "auto.http.transfer_error",
        crate::vm::ffi::http_transfer::shim_http_transfer_error,
    );
    natives.register_shim_by_name(
        "http.transfer_error",
        crate::vm::ffi::http_transfer::shim_http_transfer_error,
    );
    // PLAN-729 T-04: 服务端文件响应描述符构造（零 I/O；执行在 transport
    // 侧共享宿主服务）。
    natives.register_shim_by_name(
        "auto.http.file_response",
        crate::vm::ffi::http_server_file::shim_http_file_response,
    );
    natives.register_shim_by_name(
        "http.file_response",
        crate::vm::ffi::http_server_file::shim_http_file_response,
    );
    // PLAN-730 T-02: 服务端上传公共面（执行在共享宿主 executor，
    // a2r_std::http facade → http_upload_service 注入 hook）。
    natives.register_shim_by_name(
        "auto.http.upload_receive",
        crate::vm::ffi::http_upload::shim_http_upload_receive,
    );
    natives.register_shim_by_name(
        "http.upload_receive",
        crate::vm::ffi::http_upload::shim_http_upload_receive,
    );
    natives.register_shim_by_name(
        "auto.http.upload_metadata",
        crate::vm::ffi::http_upload::shim_http_upload_metadata,
    );
    natives.register_shim_by_name(
        "http.upload_metadata",
        crate::vm::ffi::http_upload::shim_http_upload_metadata,
    );
    natives.register_shim_by_name(
        "auto.http.upload_commit",
        crate::vm::ffi::http_upload::shim_http_upload_commit,
    );
    natives.register_shim_by_name(
        "http.upload_commit",
        crate::vm::ffi::http_upload::shim_http_upload_commit,
    );
    natives.register_shim_by_name(
        "auto.http.upload_reject",
        crate::vm::ffi::http_upload::shim_http_upload_reject,
    );
    natives.register_shim_by_name(
        "http.upload_reject",
        crate::vm::ffi::http_upload::shim_http_upload_reject,
    );
    natives.register_shim_by_name(
        "auto.http.upload_error",
        crate::vm::ffi::http_upload::shim_http_upload_error,
    );
    natives.register_shim_by_name(
        "http.upload_error",
        crate::vm::ffi::http_upload::shim_http_upload_error,
    );
    // PLAN-738 §5.3：既有生产能力面的独立逻辑契约在 production() 工厂
    // build_from_inventory 之后统一声明（inventory 条目会覆盖同 ID 契约），
    // 见 native.rs production()。

    // Plan 350: WebSocket client
    crate::vm::ffi::websocket::register_ws_natives(natives);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_json_http_helpers_shims(natives: &mut crate::vm::native::NativeInterface) {
    // PLAN-781 T-25（R05 r6）：多态类型谓词（Reader meta 数字域判别）。
    // 裸名单注册供 .at 直接 `type_of(x)` 调用（delay_async 先例）。
    natives.register_shim_by_name("auto.type_of", shim_type_of);
    natives.register_shim_by_name("type_of", shim_type_of);
    natives.register_shim_by_name("auto.http.post_json", shim_http_post_json);
    natives.register_shim_by_name("auto.http.put_json", shim_http_put_json);
    natives.register_shim_by_name("auto.http.delete_json", shim_http_delete_json);
    natives.register_shim_by_name("auto.http.patch_json", shim_http_patch_json);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_sse_stream_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Plan 341: 异步 SSE 流式接收 — 返回 iterator_id 供 for-in 消费。
    natives.register_shim_by_name("auto.http.sse_get_stream", shim_http_sse_get_stream);
    // PLAN-658 T-04: SSE 逐步拉取面 + ~Stream 端点编译 seam。
    natives.register_shim_by_name("auto.http.sse_open", shim_http_stream_sse_open);
    natives.register_shim_by_name("auto.http.sse_poll", shim_http_stream_sse_poll);
    // PLAN-707 T-04: SSE 显式 close 与终态错误查询。
    natives.register_shim_by_name("auto.http.sse_close", shim_http_stream_sse_close);
    natives.register_shim_by_name("auto.http.sse_error", shim_http_stream_sse_error);
    natives.register_shim_by_name("auto.bus.subscribe", shim_bus_subscribe);
    natives.register_shim_by_name("http.sse_get_stream", shim_http_sse_get_stream);
    natives.register_shim_by_name("http.sse_stream", shim_http_sse_get_stream);

    natives.register_shim_by_name("Response.status_code", shim_response_status_code);
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_response_stream_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Plan 046 (auto-musk T2): dynamic-receiver method family — eager so the
    // linker/peek paths see them without prior reference (peek has no lazy reg).
    natives.register_shim_by_name("auto.obj.keys", crate::vm::native::shim_obj_keys);
    natives.register_shim_by_name("auto.obj.values", crate::vm::native::shim_obj_values);
    natives.register_shim_by_name("auto.obj.find", crate::vm::native::shim_obj_find);
    natives.register_shim_by_name("Response.header_get", shim_response_header_get);
    natives.register_shim_by_name("Response.body", shim_response_body);
    natives.register_shim_by_name("Response.body_bytes", shim_response_body_bytes);
    natives.register_shim_by_name("Response.body_to_file", shim_response_body_to_file);

    // HTTP streaming (manual shims — heap objects for stream state)
    natives.register_shim_by_name("auto.http_stream.get_stream", shim_http_get_stream);
    natives.register_shim_by_name("auto.http_stream.post_stream", shim_http_post_stream);
    natives.register_shim_by_name("auto.http_stream.stream_next", shim_http_stream_next);
    natives.register_shim_by_name("auto.http_stream.stream_is_done", shim_http_stream_is_done);
    natives.register_shim_by_name("auto.http_stream.stream_close", shim_http_stream_close);
    // Plan 321: HTTPStream → Iter protocol for for-loop consumption
    natives.register_shim_by_name("auto.http_stream.stream_iter", shim_http_stream_iter);
    natives.register_shim_by_name(
        "auto.http.post_stream_with_headers",
        shim_http_post_stream_with_headers,
    );

    // HTTP client sync with auth (for Anthropic API from AutoVM)
    natives.register_shim_by_name("auto.http.post_sync", shim_http_post_sync);
    natives.register_shim_by_name("auto.http.post_bearer", shim_http_post_bearer);
    natives.register_shim_by_name("auto.http.last_status", shim_http_last_status);
    natives.register_shim_by_name("auto.http.get_sync", shim_http_get_sync);
    natives.register_shim_by_name("http.get_sync", shim_http_get_sync);
    natives.register_shim_by_name("auto.http.listen", shim_http_listen);
}
