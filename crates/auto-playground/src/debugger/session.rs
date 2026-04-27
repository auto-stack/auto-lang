use axum::extract::ws::{Message, WebSocket};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use super::controller::{DebugCommand, DebugState, PlaygroundController};

/// Run a debug session over WebSocket.
/// Compiles source, sends bytecode, then enters a relay loop between VM and frontend.
pub async fn run_debug_session(mut ws: WebSocket) {
    let source = match wait_for_source(&mut ws).await {
        Some(s) => s,
        None => return,
    };

    // Move WebSocket and source into a dedicated OS thread.
    // AutoVM is !Send due to GenericRegistry containing Rc, so we create and run it
    // entirely inside a single OS thread via block_on.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async move {
            run_debug_thread(ws, &source).await;
        });
    });
}

async fn run_debug_thread(mut ws: WebSocket, source: &str) {
    // 1. Compile and create VM
    let (mut vm, output_buffer, entry_point) = match auto_lang::create_vm_from_source(source) {
        Ok(v) => v,
        Err(e) => {
            send_json(
                &mut ws,
                serde_json::json!({"type": "error", "message": format!("{:?}", e) }),
            )
            .await
            .ok();
            return;
        }
    };

    // 2. Disassemble bytecode and send to frontend
    let disasm = auto_lang::vm::disasm::Disassembler::new(&vm.flash);
    let lines = disasm.disassemble_range(0, vm.flash.memory.len());
    let bytecode_json: Vec<serde_json::Value> = lines
        .iter()
        .map(|l| {
            serde_json::json!({
                "offset": l.offset,
                "mnemonic": l.mnemonic,
                "operands": l.operands,
                "line": l.line,
            })
        })
        .collect();
    send_json(
        &mut ws,
        serde_json::json!({ "type": "bytecode", "lines": bytecode_json }),
    )
    .await
    .ok();

    // 3. Setup controller channels
    let (cmd_tx, cmd_rx) = std::sync::mpsc::channel::<DebugCommand>();
    let (state_tx, state_rx) = tokio::sync::mpsc::channel::<DebugState>(16);

    let (controller, breakpoints) =
        PlaygroundController::new(cmd_rx, state_tx, Some(output_buffer));
    vm.set_debugger(Box::new(controller));

    // 4. Spawn relay task on tokio runtime (handles WS ↔ controller messaging)
    let relay_handle = tokio::spawn(relay_task(ws, cmd_tx, state_rx, breakpoints));

    // 5. Run VM inline in this OS thread
    vm.spawn_task(entry_point, 16384);
    vm.run_task_loop().await;

    // 6. VM finished — abort relay and close connection
    relay_handle.abort();
}

async fn wait_for_source(ws: &mut WebSocket) -> Option<String> {
    while let Some(msg) = ws.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    if json.get("type").and_then(|v| v.as_str()) == Some("debug.start") {
                        return json.get("source").and_then(|v| v.as_str()).map(|s| s.to_string());
                    }
                }
            }
            Ok(Message::Close(_)) | Err(_) => return None,
            _ => {}
        }
    }
    None
}

async fn relay_task(
    mut ws: WebSocket,
    cmd_tx: std::sync::mpsc::Sender<DebugCommand>,
    mut state_rx: tokio::sync::mpsc::Receiver<DebugState>,
    breakpoints: Arc<Mutex<HashSet<u32>>>,
) {
    loop {
        tokio::select! {
            state = state_rx.recv() => {
                match state {
                    Some(s) => {
                        let payload = serde_json::json!({
                            "type": "state",
                            "data": {
                                "status": match s.status {
                                    super::controller::DebugStatus::Paused => "paused",
                                    super::controller::DebugStatus::Running => "running",
                                    super::controller::DebugStatus::Finished => "finished",
                                    super::controller::DebugStatus::Error => "error",
                                },
                                "line": s.line,
                                "ip": s.ip,
                                "op": s.op,
                                "stack": s.stack,
                                "call_stack": s.call_stack,
                                "locals": s.locals,
                                "registers": s.registers,
                                "stdout": s.stdout,
                                "stderr": s.stderr,
                                "result": s.result,
                            }
                        });
                        if send_json(&mut ws, payload).await.is_err() {
                            break;
                        }
                    }
                    None => break,
                }
            }
            msg = ws.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if handle_client_message(&text, &cmd_tx, &breakpoints).is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                    _ => {}
                }
            }
        }
    }
}

fn handle_client_message(
    text: &str,
    cmd_tx: &std::sync::mpsc::Sender<DebugCommand>,
    breakpoints: &Arc<Mutex<HashSet<u32>>>,
) -> Result<(), ()> {
    let json: serde_json::Value = serde_json::from_str(text).map_err(|_| ())?;
    let msg_type = json.get("type").and_then(|v| v.as_str()).ok_or(())?;

    match msg_type {
        "command" => {
            let cmd = json.get("cmd").and_then(|v| v.as_str()).ok_or(())?;
            let debug_cmd = match cmd {
                "continue" => DebugCommand::Continue,
                "step" => DebugCommand::Step,
                "step_over" | "next" => DebugCommand::StepOver,
                "step_out" | "finish" => DebugCommand::StepOut,
                "stop" => DebugCommand::Stop,
                _ => return Err(()),
            };
            let is_stop = matches!(debug_cmd, DebugCommand::Stop);
            cmd_tx.send(debug_cmd).map_err(|_| ())?;
            if is_stop {
                return Err(()); // signal loop exit
            }
        }
        "breakpoints.set" => {
            let lines: Vec<u32> = json
                .get("lines")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_u64().map(|n| n as u32))
                        .collect()
                })
                .unwrap_or_default();
            let mut bp = breakpoints.lock().unwrap();
            bp.clear();
            bp.extend(lines);
        }
        _ => {}
    }

    Ok(())
}

async fn send_json(ws: &mut WebSocket, value: serde_json::Value) -> Result<(), axum::Error> {
    ws.send(Message::Text(value.to_string().into())).await
}
