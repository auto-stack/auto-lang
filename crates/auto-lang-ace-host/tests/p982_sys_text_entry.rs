//! PLAN-982（aggregates 文本入口裁定，2026-11-19）：**sys/packages 车道的受检
//! 文本入口**——`fn main(input str)` 在 AGGREGATES 面上收管道 stdin 文本
//! （`request.input` ⇒ 入口借用 `str` 实参，`ac.runtime.text.v1` 谱系），
//! `process.args()` 在**同一次执行**里可读（双通道独立）。
//!
//! 验收（裁定的切默认前合同的最小语言侧腿）：stdin="hello" ⇒ 值 1 ＋ 捕获
//! `got:hello`；stdin="other" ⇒ 值 2 ＋ 捕获 `got:other`（同源按内容分流，
//! 逐请求文本）；argv 同用腿由 man 侧真 CLI 套件持有（本档为宿主门证据）。

use auto_lang_ace_host::run_sys_script;
use auto_lang_core::{AceConsoleReport, AceOutcome, AceRequest};

fn sys_run(source: &str, input: Option<&str>, args: &[&str]) -> (AceOutcome, AceConsoleReport) {
    let request = AceRequest {
        source,
        entry_def_id: "d_main",
        capabilities: &[],
        grants: &[],
        provider: None,
        wall: None,
        cancel: None,
        console: true,
        args,
        input,
    };
    let report = run_sys_script(&request);
    (report.outcome, report.console.unwrap_or_default())
}

const CONTENT_SCRIPT: &str = "fn main(input str) int {\n  if input == \"hello\" {\n    print(\"got:hello\")\n    return 1\n  }\n  print(\"got:other\")\n  return 2\n}\n";

#[test]
fn p982_sys_text_entry_serves_stdin_by_content() {
    let (outcome, console) = sys_run(CONTENT_SCRIPT, Some("hello"), &[]);
    eprintln!(
        "p982 sys text hello: {outcome:?} stdout={:?}",
        String::from_utf8_lossy(&console.stdout)
    );
    assert!(
        format!("{outcome:?}").contains("I32(1)"),
        "the hello leg answers by content: {outcome:?}"
    );
    assert!(
        String::from_utf8_lossy(&console.stdout).contains("got:hello"),
        "the captured console carries the print: {:?}",
        console.stdout
    );

    let (outcome2, console2) = sys_run(CONTENT_SCRIPT, Some("other"), &[]);
    eprintln!(
        "p982 sys text other: {outcome2:?} stdout={:?}",
        String::from_utf8_lossy(&console2.stdout)
    );
    assert!(
        format!("{outcome2:?}").contains("I32(2)"),
        "the other leg takes the other branch: {outcome2:?}"
    );
    assert!(
        String::from_utf8_lossy(&console2.stdout).contains("got:other"),
        "the second request prints its own content: {:?}",
        console2.stdout
    );
}

/// 入口专属纪律：非入口 fn 带借用 `str` 形参 ⇒ 具名拒绝（ aggregates 的
/// `str` 面只属于文本入口，不让冻结面悄悄变宽）。
#[test]
fn p982_sys_text_param_is_entry_only() {
    let source =
        "fn helper(x str) int { return 1 }\nfn main(input str) int { return helper(input) }\n";
    let (outcome, _) = sys_run(source, Some("hi"), &[]);
    eprintln!("p982 sys text non-entry str param: {outcome:?}");
    let text = format!("{outcome:?}");
    assert!(
        text.contains("source.text-entry-param"),
        "a non-entry str param must be a named rejection, got {text}"
    );
}

/// 双通道同执行（裁定①）：同一次运行里 `fn main(input str)` 收 stdin 文本、
/// `process.args()` 读 `[argv0, ...args]`——两路输入独立变化，同腿可观察。
const DUAL_SCRIPT: &str = "use std.process.args\nfn main(input str) int {\n  let a = args()\n  let n = a.len()\n  if input == \"hello\" {\n    if n == 3 { print(\"both:hello:3\")\n      return 31 }\n    print(\"text-only:hello\")\n    return 1\n  }\n  print(\"other\")\n  return 2\n}\n";

#[test]
fn p982_sys_text_entry_and_argv_in_one_run() {
    // 两路都变：stdin=hello + argv 3 项（argv0＋2 参）⇒ 31；stdin=other ⇒ 2。
    let (o1, c1) = sys_run(DUAL_SCRIPT, Some("hello"), &["alpha", "beta"]);
    eprintln!(
        "p982 dual both: {o1:?} out={:?}",
        String::from_utf8_lossy(&c1.stdout)
    );
    assert!(
        format!("{o1:?}").contains("I32(31)"),
        "stdin+argv both vary in one run: {o1:?}"
    );
    assert!(String::from_utf8_lossy(&c1.stdout).contains("both:hello:3"));

    let (o2, c2) = sys_run(DUAL_SCRIPT, Some("other"), &["alpha", "beta"]);
    assert!(
        format!("{o2:?}").contains("I32(2)"),
        "the other stdin branch with the same argv: {o2:?}"
    );
    assert!(String::from_utf8_lossy(&c2.stdout).contains("other"));

    // stdin 不变、argv 变（n != 3）⇒ 文本分支的独占形状。
    let (o3, c3) = sys_run(DUAL_SCRIPT, Some("hello"), &["only"]);
    assert!(
        format!("{o3:?}").contains("I32(1)"),
        "argv change re-routes within the same stdin branch: {o3:?}"
    );
    assert!(String::from_utf8_lossy(&c3.stdout).contains("text-only:hello"));
}

/// 零参入口＋携带 stdin 文本 ⇒ 执行端具名拒（入口单元不声明受检文本入口——
/// 裁定②的运行端核对；文本不被静默丢弃）。
#[test]
fn p982_sys_text_input_without_text_entry_refused() {
    let source = "fn main() int { return 5 }\n";
    let (outcome, _) = sys_run(source, Some("stray"), &[]);
    eprintln!("p982 sys text zero-arg entry with input: {outcome:?}");
    let text = format!("{outcome:?}");
    assert!(
        text.contains("host.text-entry") || text.contains("ace.sys/run"),
        "a zero-arg entry with a text payload must be a named refusal, got {text}"
    );
}
