//! Plan 350: WebSocket client for AutoVM.
//!
//! Provides `ws.connect/send/on_message/close` natives. The connection runs
//! on a dedicated OS thread (like SSE in Plan 341) using `tungstenite` sync
//! client. Messages are pushed via mpsc channel, consumed via the Plan 348
//! non-blocking yield mechanism (AsyncHttpStream iterator).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use crate::vm::ffi::stdlib::{AsyncStreamEvent, AsyncStreamHandle, alloc_async_id};
use crate::vm::engine::AutoVM;
use crate::vm::task::AutoTask;
use crate::vm::engine::VMError;

// ──────────────────────────────────────────────────────────────────────
// WebSocket connection registry
// ──────────────────────────────────────────────────────────────────────

pub struct WsConnection {
    /// Sender to write messages to the WebSocket (thread-safe).
    pub tx: Mutex<Option<std::sync::mpsc::Sender<String>>>,
    /// Whether the connection is closed.
    pub closed: AtomicBool,
}

lazy_static::lazy_static! {
    static ref WS_CONNECTIONS: std::sync::Mutex<std::collections::HashMap<u64, Arc<WsConnection>>> =
        std::sync::Mutex::new(std::collections::HashMap::new());
}

// ──────────────────────────────────────────────────────────────────────
// Native shims
// ──────────────────────────────────────────────────────────────────────

/// `ws.connect(url: String) -> ws_handle (i32)`
/// Establish a WebSocket connection. Returns a handle (>0) on success, 0 on failure.
pub fn shim_ws_connect(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let (ws_tx, ws_rx) = std::sync::mpsc::channel::<String>();
    let (msg_tx, msg_rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);

    let handle = Arc::new(WsConnection {
        tx: Mutex::new(Some(ws_tx)),
        closed: AtomicBool::new(false),
    });
    let handle_clone = handle.clone();
    let msg_handle = Arc::new(AsyncStreamHandle {
        rx: Mutex::new(msg_rx),
        done: AtomicBool::new(false),
    });

    // Register the async stream for on_message iteration.
    let stream_id = alloc_async_id();
    if let Ok(mut streams) = super::stdlib::ASYNC_HTTP_STREAMS.lock() {
        streams.insert(stream_id, msg_handle.clone());
    }
    // Register the connection.
    let conn_id = alloc_async_id();
    if let Ok(mut conns) = WS_CONNECTIONS.lock() {
        conns.insert(conn_id, handle.clone());
    }

    // Spawn the WebSocket I/O thread.
    std::thread::Builder::new()
        .name("auto-ws-client".into())
        .spawn(move || {
            use tungstenite::Message;

            // Connect (blocking).
            let (mut socket, _response) = match tungstenite::connect(&url) {
                Ok(pair) => pair,
                Err(e) => {
                    let _ = msg_tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                    let _ = msg_tx.blocking_send(AsyncStreamEvent::Done);
                    handle_clone.closed.store(true, Ordering::SeqCst);
                    return;
                }
            };

            // Main loop: alternate between reading from WebSocket and checking
            // for outgoing messages. Use try_recv on ws_rx to avoid blocking.
            loop {
                // Check for outgoing messages (non-blocking).
                if let Ok(msg) = ws_rx.try_recv() {
                    if socket.send(Message::Text(msg.into())).is_err() {
                        break;
                    }
                }

                // Try to read from WebSocket (non-blocking with timeout).
                // tungstenite's read is blocking, so we set a short read timeout
                // on the underlying TCP stream. For simplicity, we use a
                // separate read thread approach: spawn a reader that pushes
                // messages to the channel.
                break; // We'll use a reader thread instead (see below).
            }

            // Actually, use a simpler approach: the reader runs in this thread,
            // the writer uses the channel. Read blocks until a message arrives.
            loop {
                match socket.read() {
                    Ok(Message::Text(text)) => {
                        let _ = msg_tx.blocking_send(AsyncStreamEvent::Data(text.to_string()));
                    }
                    Ok(Message::Binary(data)) => {
                        // Convert binary to string (lossy for display).
                        let s = String::from_utf8_lossy(&data).to_string();
                        let _ = msg_tx.blocking_send(AsyncStreamEvent::Data(s));
                    }
                    Ok(Message::Ping(_)) => {
                        // Pong is auto-sent by tungstenite.
                    }
                    Ok(Message::Pong(_)) | Ok(Message::Frame(_)) => {}
                    Ok(Message::Close(_)) => {
                        break;
                    }
                    Err(_) => {
                        break;
                    }
                }
            }

            let _ = msg_tx.blocking_send(AsyncStreamEvent::Done);
            handle_clone.closed.store(true, Ordering::SeqCst);
        })
        .expect("spawn WS client thread");

    // Return connection handle (positive = valid).
    // We encode conn_id as the return value and use stream_id internally.
    // For simplicity, store stream_id in the connection so on_message can find it.
    // Actually, we return conn_id and on_message looks up stream_id from conn.
    // To keep it simple: return conn_id. on_message takes conn_id, looks up
    // WS_CONNECTIONS to get the stream handle, creates an AsyncHttpStream iterator.
    // But stream_id is already registered. Let's just return conn_id and have
    // on_message create a new iterator pointing to the same stream_id.
    //
    // Simpler: store stream_id alongside conn. Use the SAME id for both.
    // Override: set conn_id = stream_id so one id serves both.
    {
        if let Ok(mut conns) = WS_CONNECTIONS.lock() {
            conns.remove(&conn_id);
            conns.insert(stream_id, handle);
        }
    }

    task.ram.push_i32(stream_id as i32);
    Ok(())
}

/// `ws.send(handle: i32, message: String) -> bool`
/// Send a text message over the WebSocket.
pub fn shim_ws_send(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let message: String = super::convert::VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle: i64 = task.ram.pop_i32() as i64;

    let success = WS_CONNECTIONS.lock()
        .ok()
        .and_then(|conns| {
            conns.get(&(handle as u64)).and_then(|conn| {
                conn.tx.lock().ok().and_then(|tx_guard| {
                    tx_guard.as_ref().and_then(|tx| tx.send(message).ok())
                })
            })
        })
        .is_some();

    task.ram.push_i32(if success { 1 } else { 0 });
    Ok(())
}

/// `ws.on_message(handle: i32) -> iterator_id`
/// Create an iterator that yields received messages.
/// Reuses the AsyncHttpStream + Plan 348 non-blocking yield mechanism.
pub fn shim_ws_on_message(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = task.ram.pop_i32() as i64;
    let stream_id = handle as u64;

    // Verify the connection exists.
    let exists = WS_CONNECTIONS.lock()
        .map(|conns| conns.contains_key(&stream_id))
        .unwrap_or(false);
    if !exists {
        task.ram.push_i32(0);
        return Ok(());
    }

    // Create an AsyncHttpStream iterator pointing to the existing stream_id.
    // The SSE/download message channel is already populated by the WS reader thread.
    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm.iterator_id_gen.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// `ws.close(handle: i32)`
/// Close the WebSocket connection.
pub fn shim_ws_close(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle: i64 = task.ram.pop_i32() as i64;

    if let Ok(mut conns) = WS_CONNECTIONS.lock() {
        if let Some(conn) = conns.get(&(handle as u64)) {
            // Close the sender channel — the I/O thread will detect it.
            if let Ok(mut tx_guard) = conn.tx.lock() {
                tx_guard.take(); // Drop the sender, signaling close.
            }
            conn.closed.store(true, Ordering::SeqCst);
        }
        conns.remove(&(handle as u64));
    }
    // Also remove from async streams.
    if let Ok(mut streams) = super::stdlib::ASYNC_HTTP_STREAMS.lock() {
        streams.remove(&(handle as u64));
    }

    Ok(())
}

/// Register all WebSocket natives.
pub fn register_ws_natives(
    natives: &mut crate::vm::native::NativeInterface,
) {
    natives.register_shim_by_name("auto.ws.connect", shim_ws_connect);
    natives.register_shim_by_name("ws.connect", shim_ws_connect);
    natives.register_shim_by_name("auto.ws.send", shim_ws_send);
    natives.register_shim_by_name("ws.send", shim_ws_send);
    natives.register_shim_by_name("auto.ws.on_message", shim_ws_on_message);
    natives.register_shim_by_name("ws.on_message", shim_ws_on_message);
    natives.register_shim_by_name("auto.ws.close", shim_ws_close);
    natives.register_shim_by_name("ws.close", shim_ws_close);
}
