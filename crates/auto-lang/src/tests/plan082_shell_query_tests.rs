// PLAN-082: 脚本与 `>` 命令的结构化互操作 —— auto-lang 侧单测。
//
// T-01 spike 的三个数据面前提在此锁死（结论记入 designs/039 草稿）：
//   P1 host shim 压栈 Value::Array<Obj> 后 for-in 可迭代、循环变量可 Dot 取字段；
//   P2 捕获的记录数组支持 Index+Dot 链（rows[0].name）与 .len()；
//   P3 记录字段（Str）上 .to_uint() 可用（examples/csvsum 同款方法面）。
// T-02 的 native 契约：shell_query/run 经 ShellHost 桥转发、host=None 默认空表、
// shell_run 栈中立（表达式位不炸栈）。
//
// 判例：plan585_loop_concat_system_strings_repl_parity（AutovmReplSession +
// set_host(mock) 的既有骨架，本文件沿用）。

use crate::autovm_persistent::AutovmReplSession;
use std::sync::Mutex;

/// 测试 ShellHost：`dirs` 返回两条记录 {name,size}（size 故意是 Str——
/// ash 的表格记录字段即字符串，逼出 .to_uint() 消费路径）；run 记录调用。
struct QueryHost {
    runs: Mutex<Vec<String>>,
}

impl QueryHost {
    fn new() -> Self {
        QueryHost { runs: Mutex::new(Vec::new()) }
    }
}

fn record(name: &str, size: &str) -> auto_val::Value {
    let mut o = auto_val::Obj::new();
    o.set("name", auto_val::Value::Str(auto_val::AutoStr::from(name)));
    o.set("size", auto_val::Value::Str(auto_val::AutoStr::from(size)));
    auto_val::Value::Obj(Box::new(o))
}

impl crate::host::ShellHost for QueryHost {
    fn system(&self, _cmd: &str) -> String {
        String::new()
    }
    fn system_status(&self) -> i32 {
        0
    }
    fn export(&self, _key: &str, _val: &str) {}
    fn exit(&self, _code: i32) {}
    fn exit_requested(&self) -> bool {
        false
    }
    fn requested_exit_code(&self) -> i32 {
        0
    }
    fn query(&self, cmd: &str) -> auto_val::Value {
        if cmd == "dirs" {
            auto_val::Value::Array(auto_val::Array {
                values: vec![record("alpha", "41"), record("beta", "42")],
            })
        } else {
            auto_val::Value::Array(auto_val::Array::default())
        }
    }
    fn run(&self, cmd: &str) {
        self.runs.lock().unwrap().push(cmd.to_string());
    }
}

/// T-01 P1 + T-02：for-in 直接消费 shell_query 结果，循环变量 Dot 取字段。
/// 这正是 ash 预处理层为 `for rec in > cmd { }` 生成的目标形态。
#[test]
fn plan082_for_in_over_shell_query_records() {
    let mut session = AutovmReplSession::new();
    session.vm.set_host(std::sync::Arc::new(QueryHost::new()));
    let script = r#"fn main() {
    var out = ""
    for r in shell_query("dirs") {
        out = out + r.name + ";"
    }
    out
}
main()
"#;
    let result = session.run(script);
    assert!(result.is_ok(), "script should run: {:?}", result.err());
    assert_eq!(
        session.format_last_result().as_deref(),
        Some("alpha;beta;"),
        "for-in 必须逐记录迭代且 r.name 取到字段值"
    );
}

/// T-01 P2：捕获记录数组后 Index+Dot 链与 .len() 可用。
#[test]
fn plan082_index_dot_and_len_on_captured_records() {
    let mut session = AutovmReplSession::new();
    session.vm.set_host(std::sync::Arc::new(QueryHost::new()));
    let script = r#"fn main() {
    var rows = shell_query("dirs")
    rows[0].name + "/" + rows[1].name + ":" + rows.len()
}
main()
"#;
    let result = session.run(script);
    assert!(result.is_ok(), "script should run: {:?}", result.err());
    assert_eq!(
        session.format_last_result().as_deref(),
        Some("alpha/beta:2"),
        "rows[0].name 与 rows.len() 必须在 host 构造的动态记录上可用"
    );
}

/// T-01 P3：记录字段（Str）上 .to_uint() 参与数值比较（csvsum 同款方法面）。
#[test]
fn plan082_to_uint_on_record_field() {
    let mut session = AutovmReplSession::new();
    session.vm.set_host(std::sync::Arc::new(QueryHost::new()));
    let script = r#"fn main() {
    var n = 0
    for r in shell_query("dirs") {
        if r.size.to_uint() > 41 { n = n + 1 }
    }
    n
}
main()
"#;
    let result = session.run(script);
    assert!(result.is_ok(), "script should run: {:?}", result.err());
    assert_eq!(
        session.format_last_result().as_deref(),
        Some("1"),
        "r.size.to_uint() 必须把 Str 字段转成可比较的数值（仅 beta=42 过阈值）"
    );
}

/// T-02：shell_run 透传命令到 host，且在语句位栈中立（后续表达式不串位）。
#[test]
fn plan082_shell_run_forwards_and_stack_neutral() {
    let host = std::sync::Arc::new(QueryHost::new());
    let mut session = AutovmReplSession::new();
    session.vm.set_host(host.clone());
    let script = r#"fn main() {
    shell_run("echo hi")
    shell_run("echo bye")
    "done"
}
main()
"#;
    let result = session.run(script);
    assert!(result.is_ok(), "script should run: {:?}", result.err());
    assert_eq!(session.format_last_result().as_deref(), Some("done"));
    assert_eq!(
        *host.runs.lock().unwrap(),
        vec!["echo hi".to_string(), "echo bye".to_string()],
        "shell_run 必须把命令原文透传给 host.run"
    );
}

/// T-02：host=None（纯 AutoLang 场景）时 shell_query 走 trait 默认空表，
/// 不 panic、不炸栈。
#[test]
fn plan082_query_without_host_defaults_to_empty_array() {
    let mut session = AutovmReplSession::new();
    let script = r#"fn main() {
    shell_query("anything").len()
}
main()
"#;
    let result = session.run(script);
    assert!(result.is_ok(), "script should run: {:?}", result.err());
    assert_eq!(
        session.format_last_result().as_deref(),
        Some("0"),
        "无 host 时 shell_query 必须返回空表（trait 默认实现）"
    );
}
