//! PLAN-816 T-05 R2-04：op 级执行选项协商的旧 v1 宿主（永久负控）。
//!
//! describe 回答**不含 `options` 字段**的 v1 基线描述符（与升级前的
//! 完整 v1 工具同形——serde 对未知请求字段的静默忽略是解码兼容，不是
//! 语义兑现）。随后按帧循环：
//! - 携带执行选项（output/project/delivery_root/strict/args）的工作请求
//!   到达 = 客户端协商未拦截、选项将被静默丢弃（副作用窗口）——写
//!   `T05_GATE_MARKER` marker 并以 Internal 如实回答；
//! - 不携带选项的基础 generate_ui 按协议正常回答（旧基础请求正控）。
//! 仅供 fixture 测试 spawn。

use std::io::{Read, Write};

fn main() {
    let marker = std::env::var_os("T05_GATE_MARKER");
    // relay-lie 模式（F-816-T05-R2-04 端到端核实）：describe **广播**选项
    // （冒充新工具/转发新工具描述符的中继），但对携带选项的 generate 回
    // 一个落点在默认布局的"成功"报告——同构旧 wire 中间人剥掉请求选项
    // 后工具按默认布局生成。客户端落点核对必须具名 OptionIgnored。
    let advertise = std::env::var("T05_GATE_ADVERTISE").is_ok();
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        let mut header = [0u8; 4];
        if stdin.read_exact(&mut header).is_err() {
            return; // EOF（客户端 shutdown 发出）
        }
        let mut body = vec![0u8; u32::from_be_bytes(header) as usize];
        if stdin.read_exact(&mut body).is_err() {
            return;
        }
        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => return,
        };
        let op = request["body"]["op"].as_str().unwrap_or("");
        let reply = answer(op, &request, &marker, advertise);
        let bytes = serde_json::to_vec(&reply).expect("encode reply");
        stdout
            .write_all(&(bytes.len() as u32).to_be_bytes())
            .expect("write header");
        stdout.write_all(&bytes).expect("write body");
        stdout.flush().expect("flush");
    }
}

fn answer(
    op: &str,
    request: &serde_json::Value,
    marker: &Option<std::ffi::OsString>,
    advertise: bool,
) -> serde_json::Value {
    let id = request["id"].clone();
    if advertise {
        return answer_advertising(op, &request, marker);
    }
    match op {
        "describe" => serde_json::json!({
            "id": id,
            "body": {"type": "describe_ok", "descriptor": {
                "tool_name": "auto-ui-tool",
                "tool_version": "0.1.0",
                "protocol_version": 1,
                "capabilities": [
                    {"operation": "describe", "cancellable": false, "interactive": false},
                    {"operation": "validate_project", "cancellable": false, "interactive": false},
                    {"operation": "generate_ui", "cancellable": false, "interactive": false},
                    {"operation": "build_ui", "cancellable": true, "interactive": false},
                    {"operation": "run_ui", "cancellable": true, "interactive": true}
                ],
                "lifecycle": {
                    "shutdown_on_eof": true,
                    "logs_on_stderr_only": true,
                    "exit_codes": [[0, "clean EOF shutdown"], [1, "internal failure"], [2, "fatal stream"]]
                }
            }}
        }),
        "generate_ui" => {
            let carries = ["output", "project", "delivery_root", "strict"]
                .iter()
                .any(|k| !request["body"][k].is_null());
            if carries {
                leak(marker, op);
                error_reply(
                    id,
                    "generate options would be silently dropped by this v1 host",
                )
            } else {
                serde_json::json!({
                    "id": id,
                    "body": {"type": "generate_ok", "report": {
                        "output_root": ".", "frontends": ["vue"], "files_written": 0, "duration_ms": 1
                    }}
                })
            }
        }
        "build_ui" => {
            // 协商负控里该请求不应到达；到达即泄漏。
            leak(marker, op);
            error_reply(id, "strict would be silently dropped by this v1 host")
        }
        "run_ui" => {
            leak(marker, op);
            error_reply(id, "run args would be silently dropped by this v1 host")
        }
        other => error_reply(id, &format!("legacy host: unexpected op {other}")),
    }
}

/// relay-lie 臂：describe 广播全部选项；携带选项的 generate 回落点在
/// 默认布局的成功报告（选项被"静默忽略"的镜像）。
fn answer_advertising(
    op: &str,
    request: &serde_json::Value,
    marker: &Option<std::ffi::OsString>,
) -> serde_json::Value {
    let id = request["id"].clone();
    match op {
        "describe" => serde_json::json!({
            "id": id,
            "body": {"type": "describe_ok", "descriptor": {
                "tool_name": "auto-ui-tool",
                "tool_version": "0.1.0",
                "protocol_version": 1,
                "capabilities": [
                    {"operation": "describe", "cancellable": false, "interactive": false},
                    {"operation": "validate_project", "cancellable": false, "interactive": false},
                    {"operation": "generate_ui", "cancellable": false, "interactive": false,
                     "options": ["output", "project", "delivery_root", "strict"]},
                    {"operation": "build_ui", "cancellable": true, "interactive": false,
                     "options": ["strict"]},
                    {"operation": "run_ui", "cancellable": true, "interactive": true,
                     "options": ["args"]}
                ],
                "lifecycle": {
                    "shutdown_on_eof": true,
                    "logs_on_stderr_only": true,
                    "exit_codes": [[0, "clean EOF shutdown"], [1, "internal failure"], [2, "fatal stream"]]
                }
            }}
        }),
        "generate_ui" => {
            let carries = ["output", "project", "delivery_root", "strict"]
                .iter()
                .any(|k| !request["body"][k].is_null());
            if carries {
                leak(marker, op);
                serde_json::json!({
                    "id": id,
                    "body": {"type": "generate_ok", "report": {
                        "output_root": "C:/lie/default-layout-root",
                        "frontends": ["rust"], "files_written": 2, "duration_ms": 1
                    }}
                })
            } else {
                serde_json::json!({
                    "id": id,
                    "body": {"type": "generate_ok", "report": {
                        "output_root": ".", "frontends": ["vue"], "files_written": 0, "duration_ms": 1
                    }}
                })
            }
        }
        other => error_reply(id, &format!("relay-lie host: unexpected op {other}")),
    }
}

fn leak(marker: &Option<std::ffi::OsString>, op: &str) {
    if let Some(m) = marker {
        let _ = std::fs::write(m, format!("REQUEST_LEAKED op={op}"));
    }
}

fn error_reply(id: serde_json::Value, message: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "body": {"type": "error", "error": {"kind": "internal", "message": message}}
    })
}
