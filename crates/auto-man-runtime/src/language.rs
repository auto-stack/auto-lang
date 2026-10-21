//! 非 UI 语言执行入口（PLAN-816 T-03 F-02）。
//!
//! 语言面单源 `auto_lang_core`（814 SDK）：服务/`run` 轨的
//! `run_language_file` 不得经 UI seam——未安装 UI provider 时
//! 独立服务仍须进入真实 VM 执行。

use crate::AutoResult;

/// 在大栈线程中按源路径执行 `.at`（模块解析与历史 `run_file` 同语义）。
///
/// 与外层 `auto_lang::run_with_path` 同构：`execute_autovm_with_path`
/// + 32MB 栈；stdout 非空时原样打印，返回语言结果字符串。
pub fn run_language_file(path: &str) -> AutoResult<String> {
    let code =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file {path}: {e}"))?;
    let path_owned = path.to_string();
    let handle = std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .name("auto-run-language-file".into())
        .spawn(move || -> Result<String, String> {
            auto_lang_core::block_on_autovm_local(async {
                auto_lang_core::execute_autovm_with_path(&code, true, Some(&path_owned))
                    .await
                    .map(|(r, stdout, _, _)| {
                        if !stdout.is_empty() {
                            println!("{stdout}");
                        }
                        r
                    })
                    .map_err(|e| e.to_string())
            })
        })
        .map_err(|e| format!("Failed to spawn language thread: {e}"))?;
    match handle.join() {
        Ok(Ok(r)) => Ok(r),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Err("language execution thread panicked".into()),
    }
}
