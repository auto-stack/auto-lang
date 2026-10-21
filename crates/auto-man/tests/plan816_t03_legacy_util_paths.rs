//! PLAN-816 T-03 F-03：旧 `auto_man::util` 模块路径编译回归。
use auto_man::util::{
    hash_string, http_base_url, http_port, kill_process_on_port, parse_tasklist_name,
    should_skip_kill,
};

#[test]
fn legacy_util_module_paths_resolve() {
    // 仅证明六条旧模块路径可导入可调用；算法 owner 仍在 auto-ui-tools。
    let _ = hash_string("x");
    let _ = http_base_url();
    let _ = http_port();
    let _ = should_skip_kill("node.exe");
    let _ = parse_tasklist_name("123,foo.exe");
    // kill_process_on_port 可解析；不在测试里真杀进程。
    let f: fn(u16) = kill_process_on_port;
    let _ = f;
}
