//! Plan 321/322: AutoHttpServer — unified HTTP server backend wrapping Axum.
//!
//! This module is shared by both VM mode (via native shim) and a2r mode
//! (via generated Rust code). It encapsulates Axum Router construction,
//! route matching, SSE streaming, and the !Send VM bridging.
//!
//! ## VM mode bridging strategy
//!
//! AutoVM is !Send (Rc<RefCell> in type system). Axum handlers must be
//! Send + 'static futures. To bridge:
//!
//! 1. The HTTP server runs on a dedicated OS thread (not the tokio runtime
//!    that drives the VM's async task system).
//! 2. On that thread, we create a `current_thread` tokio runtime and run
//!    `axum::serve` inside `block_on`.
//! 3. Each Axum handler is a thin async wrapper that uses `spawn_blocking`
//!    to call the VM synchronously (the VM lives on the same thread, so
//!    the blocking call is safe — it just blocks the current_thread runtime,
//!    which is fine since there's only one worker).
//!
//! Alternatively (simpler for MVP): skip Axum entirely for VM mode and
//! keep the existing std::net implementation, but route it through this
//! module's route table for unified route matching logic. Axum can be
//! added later for a2r mode.

use std::collections::HashMap;
use std::io::{Read, Write, BufRead};
use std::net::TcpListener;

/// An HTTP route registered with the server.
#[derive(Debug, Clone)]
pub struct HttpRoute {
    pub method: String,
    pub path: String,
    pub fn_name: String,
}

/// Result of matching a request against routes.
pub struct RouteMatch {
    pub fn_name: String,
    pub path_params: Vec<(String, String)>,
}

/// Match a request (method, path) against a list of routes.
/// Supports `:param` path parameter extraction (e.g. /api/notes/:id).
pub fn match_route(routes: &[HttpRoute], method: &str, path: &str) -> Option<RouteMatch> {
    for route in routes {
        if route.method.to_uppercase() != method.to_uppercase() {
            continue;
        }
        let route_segments: Vec<&str> = route.path.split('/').collect();
        let path_segments: Vec<&str> = path.split('/').collect();
        if route_segments.len() != path_segments.len() {
            continue;
        }
        let mut params = Vec::new();
        let mut matched = true;
        for (rs, ps) in route_segments.iter().zip(path_segments.iter()) {
            if let Some(param_name) = rs.strip_prefix(':') {
                params.push((param_name.to_string(), ps.to_string()));
            } else if rs != ps {
                matched = false;
                break;
            }
        }
        if matched {
            return Some(RouteMatch {
                fn_name: route.fn_name.clone(),
                path_params: params,
            });
        }
    }
    None
}

/// Get the global HTTP routes (populated by VM startup from #[api] annotations).
/// This delegates to the existing HTTP_ROUTES global in stdlib.rs.
pub fn get_routes() -> Vec<HttpRoute> {
    crate::vm::ffi::stdlib::get_http_routes()
        .into_iter()
        .map(|(method, path, fn_name)| HttpRoute { method, path, fn_name })
        .collect()
}

/// Run the HTTP server in blocking mode using std::net (MVP).
///
/// This is the current implementation — synchronous, serial request handling.
/// Each request is dispatched to a VM handler function via call_fn_by_name.
///
/// Future: replace with Axum for concurrency, SSE, TLS support.
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
            let _ = stream.write_all(b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n");
            continue;
        }
        let req_method = parts[0].to_uppercase();
        let req_path = parts[1].to_string();

        // Read headers
        let mut content_length = 0usize;
        loop {
            let mut header = String::new();
            if reader.read_line(&mut header).is_err() { break; }
            let header = header.trim();
            if header.is_empty() { break; }
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
                let resp = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: 9\r\nConnection: close\r\n\r\nNot Found";
                let _ = stream.write_all(resp.as_bytes());
                continue;
            }
        };

        // Call VM handler
        let handler_task_id = vm.spawn_task(0, 8192);
        let result_json: Option<String> = if let Some(handler_task_arc) = vm.tasks.get(&handler_task_id) {
            let mut ht = handler_task_arc.blocking_lock();

            let mut n_args = 0;
            for (_param_name, param_val) in &route_match.path_params {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(param_val.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }
            if !body.is_empty() {
                let idx = {
                    let mut strings = vm.strings.write().unwrap();
                    let i = strings.len();
                    strings.push(body.as_bytes().to_vec());
                    i
                };
                ht.ram.push_nv(auto_val::encode_string(idx as u32));
                n_args += 1;
            }

            match vm.call_fn_by_name(&mut ht, &route_match.fn_name, n_args) {
                Ok(()) => {
                    let nv = ht.ram.pop_nv();
                    if auto_val::is_string(nv) {
                        let idx = auto_val::decode_string(nv);
                        vm.strings.read().unwrap().get(idx as usize)
                            .map(|b| String::from_utf8_lossy(b).to_string())
                    } else if auto_val::is_i32(nv) {
                        Some(auto_val::decode_i32(nv).to_string())
                    } else if auto_val::is_null(nv) {
                        Some("null".to_string())
                    } else {
                        Some("null".to_string())
                    }
                }
                Err(e) => {
                    eprintln!("[HTTP] Handler '{}' error: {:?}", route_match.fn_name, e);
                    None
                }
            }
        } else {
            None
        };

        vm.tasks.remove(&handler_task_id);

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
