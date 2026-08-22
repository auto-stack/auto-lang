// Plan 060 M3(2026-08-22):通用宿主桥 —— merged 模式"进程内直调"的机制件。
//
// 背景:ash-gui 的后端真实现是 ash-server(worker 线程持有 auto_shell::Shell,
// 调 ash-core)。auto-lang 不能依赖 ash-server/auto-shell crate(cargo 环:
// auto-shell 依赖 auto-lang 本体),因此直连桥拆成两半:
//   - 本模块(机制,零业务):名字 → 函数 的运行时注册表 + native
//     `auto.host.call` / `auto.host.call_value`(参数与返回均为 JSON 串,
//     与 HTTP 模式的传输编码同构 —— 前端 .at 桩可用 json.to_value 复用
//     HTTP 模式已验证的 JSON→VM 值转换)。
//   - auto-shell 仓的 ash-runner(策略):进程内起 ash_server worker,把
//     api.at 契约端点注册进来,ShellEvent 经 inject_shell_event 回流。
// 依赖方向:auto-lang 只提供机制;所有 shell 语义在 ash-server/ash-core。

use std::collections::HashMap;
use std::sync::Arc;

/// 桥函数:入参 JSON 串 → 返回 JSON 串(或错误信息)。
pub type HostCallFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

lazy_static::lazy_static! {
    static ref HOST_CALLS: std::sync::Mutex<HashMap<String, HostCallFn>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 注册一个宿主桥函数(ash-runner 启动时调用;同名覆盖便于测试)。
pub fn register_host_call(name: &str, f: HostCallFn) {
    HOST_CALLS
        .lock()
        .unwrap()
        .insert(name.to_string(), f);
}

/// 调用已注册的宿主桥函数(VM native 侧进入)。
pub fn call_host(name: &str, args_json: &str) -> Result<String, String> {
    let f = HOST_CALLS
        .lock()
        .unwrap()
        .get(name)
        .cloned()
        .ok_or_else(|| format!("host bridge: no function registered for '{}'", name))?;
    f(args_json)
}

/// 是否已有任何桥函数注册(runner 判定 merged-host 模式用)。
pub fn has_host_calls() -> bool {
    !HOST_CALLS.lock().unwrap().is_empty()
}
