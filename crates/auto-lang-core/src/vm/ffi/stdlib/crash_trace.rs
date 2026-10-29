//! PLAN-843 结构拆分：crash_trace 域实现（原样搬移，算法/行为零变化）。
use super::*;

/// Exit the process with a code
#[auto_macros::rust_fn("Process.exit")]
pub fn shim_process_exit(code: i32) {
    // PLAN-575 D1 挂点①：裸 exit 前落审计（site=vm_process_exit），
    // 退出码与语义零变化（G3）。
    exit_audit(code, "vm_process_exit");
    std::process::exit(code);
}

/// Real argv as a List of strings: [程序路径] + CLI 透传参数（无透传时仅
/// 程序路径）。Plan 524 修复：此前返回 join 空格字符串（517 W3 实测 argc=76
/// 即对 join 串取长度的伪 argc），且含空格参数不可表达；List 契约对齐
/// a2r 路径消费形态（examples/a2rs/03_image_scraper.at 的 list.len/下标）。
#[auto_macros::rust_fn("Process.args")]
pub fn shim_process_args() -> Vec<String> {
    let mut argv = vec![std::env::args().next().unwrap_or_default()];
    argv.extend(crate::script_args());
    argv
}

/// Get current working directory
#[auto_macros::rust_fn("Process.current_dir")]
pub fn shim_process_current_dir() -> String {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Set current working directory
#[auto_macros::rust_fn("Process.set_current_dir")]
pub fn shim_process_set_current_dir(path: String) -> Result<(), String> {
    std::env::set_current_dir(&path)
        .map_err(|e| format!("Process.set_current_dir failed: {} - {}", path, e))
}

/// Spawn an external process and wait for it to complete
#[auto_macros::rust_fn("Process.spawn")]
pub fn shim_process_spawn(args: Vec<String>) -> Result<i32, String> {
    if args.is_empty() {
        return Err("Process.spawn failed: empty arguments".to_string());
    }

    let cmd = &args[0];
    let cmd_args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    match std::process::Command::new(cmd).args(&cmd_args).status() {
        Ok(status) => Ok(status.code().unwrap_or(-1)),
        Err(e) => Err(format!("Process.spawn failed: {} - {}", cmd, e)),
    }
}

/// Spawn an external process and capture its stdout/stderr output.
///
/// Returns a JSON object: `{"exit_code": i32, "stdout": String, "stderr": String}`
///
/// This is a manual shim (not `#[rust_fn]`) because it returns a complex JSON value
/// that doesn't map cleanly to a single primitive type.
pub fn shim_process_spawn_with_output(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Pop args as JSON array string: ["cmd", "arg1", "arg2"]
    let args_json: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let args: Vec<String> = serde_json::from_str(&args_json)
        .map_err(|e| VMError::RuntimeError(format!("Invalid args JSON: {}", e)))?;

    if args.is_empty() {
        return Err(VMError::RuntimeError(
            "Process.spawn_with_output failed: empty arguments".into(),
        ));
    }

    let cmd = &args[0];
    let cmd_args: Vec<&str> = args.iter().skip(1).map(|s| s.as_str()).collect();

    let output = std::process::Command::new(cmd)
        .args(&cmd_args)
        .output()
        .map_err(|e| {
            VMError::RuntimeError(format!("Process.spawn_with_output failed: {} - {}", cmd, e))
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    // Truncate stdout at 64KB to avoid excessive memory use
    let stdout_truncated = if stdout.len() > 65536 {
        format!(
            "{}...(truncated, {} bytes total)",
            &stdout[..65536],
            stdout.len()
        )
    } else {
        stdout
    };

    let result_json = serde_json::json!({
        "exit_code": exit_code,
        "stdout": stdout_truncated,
        "stderr": stderr,
    });

    let result_str = serde_json::to_string(&result_json)
        .map_err(|e| VMError::RuntimeError(format!("JSON serialization failed: {}", e)))?;

    result_str
        .push_to_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    Ok(())
}

/// PLAN-884 T-04：自 stdlib 壳 register_stdlib_ffi 按域迁入的注册组
/// （连续原语句切片，序=原序）。注册序是 plan872 有效绑定的载荷，不得重排。
pub(super) fn register_process_spawn_shims(natives: &mut crate::vm::native::NativeInterface) {
    // Process functions (manual shims only)
    natives.register_shim_by_name(
        "auto.process.spawn_with_output",
        shim_process_spawn_with_output,
    );
    natives.register_shim_by_name("auto.sys.exec", shim_sys_exec);
}
