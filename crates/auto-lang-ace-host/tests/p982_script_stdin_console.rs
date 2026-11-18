//! PLAN-982（EXEC · 脚本 stdin）：**console 车道的文本入口**——管道 stdin 作为
//! 运行时文本交给 `fn main(input str)` 形脚本（`execute_entry_text_platform`，
//! 982 round-182 通道），`print` 捕获与 run_console 同纪律。
//!
//! 验收：同源两腿——stdin="hello" ⇒ `I32(1)` + 捕获 `got:hello`；
//! stdin="other" ⇒ `I32(2)` + 捕获 `got:other`（同驻留产物、逐请求文本）。

use auto_lang_core::{run_ace_text, AceEngineChoice, AceOutcome, AceRequest};

fn run(texts: &[&[u8]]) -> (AceOutcome, String) {
    let _ = auto_lang_ace_host::install();
    let grants = vec!["console.stdout".to_string(), "console.stderr".to_string()];
    let request = AceRequest {
        console: true,
        grants: &grants,
        ..AceRequest::simple(
            "fn main(input str) int {\n  if input == \"hello\" {\n    print(\"got:hello\")\n    return 1\n  }\n  print(\"got:other\")\n  return 2\n}\n",
        )
    };
    let report = run_ace_text(AceEngineChoice::AceAV, &request, texts);
    let stdout = report
        .console
        .as_ref()
        .map(|c| c.stdout.clone())
        .unwrap_or_default();
    (report.outcome, String::from_utf8_lossy(&stdout).to_string())
}

#[test]
fn p982_console_text_entry_serves_piped_stdin_by_content() {
    let (outcome, stdout) = run(&[b"hello"]);
    eprintln!("p982 stdin hello: {outcome:?} stdout={stdout:?}");
    assert!(
        format!("{outcome:?}").contains("I32(1)"),
        "the hello leg answers by content: {outcome:?}"
    );
    assert!(
        stdout.contains("got:hello"),
        "the captured console carries the print: {stdout:?}"
    );

    let (outcome2, stdout2) = run(&[b"other"]);
    eprintln!("p982 stdin other: {outcome2:?} stdout={stdout2:?}");
    assert!(
        format!("{outcome2:?}").contains("I32(2)"),
        "the other leg takes the other branch: {outcome2:?}"
    );
    assert!(
        stdout2.contains("got:other"),
        "the second request prints its own content: {stdout2:?}"
    );
}
