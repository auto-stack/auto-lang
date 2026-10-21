//! PLAN-816 T-04 R1-01：discovery 收尾失败的具名测试宿主。
//!
//! 以真实 framing 回答一帧合法 DescribeOk，再在 stdin EOF 后按
//! `T04_HOST_MODE` 分支：`good`=exit 0；`tailbad`=坏控制帧前缀；
//! `exit1`/`exit7`=指定退出码。仅供 fixture 测试 spawn。

use std::io::{Read, Write};

fn main() {
    let mode = std::env::var("T04_HOST_MODE").unwrap_or_else(|_| "good".into());
    let mut stdin = std::io::stdin();
    let mut header = [0u8; 4];
    if stdin.read_exact(&mut header).is_err() {
        return;
    }
    let mut body = vec![0u8; u32::from_be_bytes(header) as usize];
    if stdin.read_exact(&mut body).is_err() {
        return;
    }
    let request: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(_) => return,
    };
    let descriptor = serde_json::json!({
        "tool_name": "auto-ui-tool",
        "tool_version": "0.1.0",
        "protocol_version": 1,
        "capabilities": [
            {"operation": "describe", "cancellable": false, "interactive": false},
            {"operation": "validate_project", "cancellable": false, "interactive": false}
        ],
        "lifecycle": {
            "shutdown_on_eof": true,
            "logs_on_stderr_only": true,
            "exit_codes": [[0, "clean EOF shutdown"], [1, "internal failure"], [2, "fatal stream"]]
        }
    });
    let response = serde_json::json!({
        "id": request["id"],
        "body": {"type": "describe_ok", "descriptor": descriptor}
    });
    let bytes = serde_json::to_vec(&response).expect("encode describe_ok");
    let mut stdout = std::io::stdout();
    stdout
        .write_all(&(bytes.len() as u32).to_be_bytes())
        .expect("write header");
    stdout.write_all(&bytes).expect("write body");
    stdout.flush().expect("flush");

    // 等 EOF（客户端 shutdown 发出）
    let mut rest = Vec::new();
    let _ = stdin.read_to_end(&mut rest);

    match mode.as_str() {
        "tailbad" => {
            // 收尾阶段坏控制帧前缀（4×0xFF）。
            stdout.write_all(&[255u8; 4]).expect("write bad prefix");
            stdout.flush().expect("flush bad");
        }
        "exit1" => std::process::exit(1),
        "exit7" => std::process::exit(7),
        _ => {}
    }
}
