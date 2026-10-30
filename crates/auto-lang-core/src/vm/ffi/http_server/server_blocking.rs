//! Synchronous server-handle runtime, single source for both legacy entry names.
//! Core's language shim only selects an explicitly installed service host.
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::host_capabilities::http_service::messages::HttpProgramMetadata;
use std::sync::{atomic::AtomicBool, Arc};

/// Run on the borrowed VM's existing owner thread. The optional stop signal is
/// used by bounded embedding probes; the controller wakes accept with a socket.
/// It is not a slow-body cancellation or graceful-drain guarantee.
pub fn serve_server_handle(
    vm: &AutoVM,
    addr: &str,
    metadata: Arc<HttpProgramMetadata>,
    stop: Option<&AtomicBool>,
) -> Result<(), VMError> {
    super::with_owner_metadata_sync(metadata, || serve_inner(vm, addr, stop))
}

fn serve_inner(vm: &AutoVM, addr: &str, stop: Option<&AtomicBool>) -> Result<(), VMError> {
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
        let result_json: Option<String> = if let Some(handler_task_arc) = vm
            .tasks
            .get(&handler_task_id)
            .map(|task| task.value().clone())
        {
            // A fresh task is private to this owner. try_lock also works when
            // the synchronous language shim is called inside a Tokio runtime.
            let mut ht = match handler_task_arc.try_lock() {
                Ok(task) => task,
                Err(_) => {
                    vm.remove_task(handler_task_id);
                    return Err(VMError::RuntimeError(
                        "fresh HTTP handler task is unexpectedly busy".into(),
                    ));
                }
            };

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

// PLAN-814: 811 前残留门（体纯 auto-media-core，renderer 中性）随 Core 迁 SDK 去门
pub(crate) fn write_media_response(
    stream: &mut impl std::io::Write,
    method: &str,
    path: &str,
    if_none_match: Option<&str>,
) -> bool {
    let Some(response) =
        crate::vm::ffi::stdlib::media_response_for_vm_request(method, path, if_none_match)
    else {
        return false;
    };
    let reason = match response.status {
        200 => "OK",
        304 => "Not Modified",
        404 => "Not Found",
        410 => "Gone",
        422 => "Unprocessable Entity",
        503 => "Service Unavailable",
        _ => "Error",
    };
    let body = response
        .body
        .map(|bytes| bytes.to_vec())
        .unwrap_or_default();
    let mut wire = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        response.status,
        reason,
        response
            .headers
            .get("content-length")
            .map(String::as_str)
            .unwrap_or("0"),
    );
    for (name, value) in response.headers {
        wire.push_str(&format!("{}: {}\r\n", name, value));
    }
    wire.push_str("\r\n");
    let _ = stream.write_all(wire.as_bytes());
    if method == "GET" {
        let _ = stream.write_all(&body);
    }
    true
}
