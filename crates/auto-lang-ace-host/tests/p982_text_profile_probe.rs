//! PLAN-982（983 文本面，用户裁定 2026-11-18）：**Text profile 能编译带 `str`
//! 参数的服务 handler**。
//!
//! 探针结论（永久钉住，作为"AV 服务入口接通借用文本"的底座）：
//! - 默认服务入口走 Core/Scalar profile，`str` 被 `source.unsupported` 拒绝；
//! - **Text profile**（`SourceProfile::Text`，`is_text() == true`）接受
//!   `fn main(q str) int { return q.len_bytes() }`；
//! - 该 profile 不接受 `use`（`source.unsupported`），文本原语走**视图方法**
//!   （`q.len_bytes()`），与 ACE 文本契约一致。
//!
//! 下一步（round 162+）：服务入口改用 Text profile 编译，并把请求文本作为
//! **运行时数据**（StrView 实参）传给入口——源码与编译产物身份保持不变。

use ace_core::source::SourceProfile;

const TEXT_CAP: &str = "ac.runtime.text.v1";

fn compile(src: &str) -> Result<(), Vec<ace_core::atom_text::Diagnostic>> {
    let caps = vec![TEXT_CAP.to_string()];
    ace_core::source::compile_text_source(src, "p982-text.at", &caps).map(|_| ())
}

#[test]
fn text_profile_accepts_str_parameters_and_view_methods() {
    assert!(
        SourceProfile::Text.is_text(),
        "the Text profile is a text profile"
    );
    assert!(
        compile("fn main(q str) int {\n  return q.len_bytes()\n}\n").is_ok(),
        "the Text profile compiles a handler with a `str` parameter"
    );
}

#[test]
fn text_profile_refuses_use_imports() {
    let error = compile("use std.text\nfn main(q str) int {\n  return 0\n}\n")
        .expect_err("`use` is outside this profile");
    assert!(
        error.iter().any(|d| d.code == "source.unsupported"),
        "the refusal is named: {error:?}"
    );
}
