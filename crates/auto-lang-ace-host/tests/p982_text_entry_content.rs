//! PLAN-982（983 文本面）：受检文本入口的两种典型 handler——按**长度**与按**内容**
//! 使用借用 `str`（`q.len_bytes()` 与 `q == "中文"`），都经 `run_ace_text`
//! （Text profile → 受检 lowering → MIR/ABC → `execute_entry_text`）。

use auto_lang_core::{run_ace_text, AceEngineChoice, AceOutcome, AceRequest};

fn run(src: &str, text: &[u8]) -> AceOutcome {
    let _ = auto_lang_ace_host::install();
    let request = AceRequest::simple(src);
    run_ace_text(AceEngineChoice::AceAV, &request, &[text]).outcome
}

#[test]
fn the_text_entry_reads_length_and_content() {
    let by_len = run(
        "fn main(q str) int {\n  return q.len_bytes()\n}\n",
        "中文".as_bytes(),
    );
    assert!(
        format!("{by_len:?}").contains("I32(6)"),
        "`中文` is 6 UTF-8 bytes: {by_len:?}"
    );

    let by_content = run(
        "fn main(q str) int {\n  if q == \"中文\" {\n    return 1\n  }\n  return 2\n}\n",
        "中文".as_bytes(),
    );
    assert!(
        format!("{by_content:?}").contains("I32(1)"),
        "the handler compares the borrowed text by content: {by_content:?}"
    );

    let other = run(
        "fn main(q str) int {\n  if q == \"中文\" {\n    return 1\n  }\n  return 2\n}\n",
        "日文".as_bytes(),
    );
    assert!(
        format!("{other:?}").contains("I32(2)"),
        "a different value of the same byte length takes the other branch: {other:?}"
    );
}
