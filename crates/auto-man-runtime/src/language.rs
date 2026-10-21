//! 非 UI 语言执行入口（PLAN-816 T-03 F-02）。
//!
//! 语言面单源 `auto_lang_core`（814 SDK）：服务/`run` 轨的
//! `run_language_file` 不得经 UI seam——未安装 UI provider 时
//! 独立服务仍须进入真实 VM 执行。

use crate::AutoResult;

/// 在大栈线程中按源路径执行 `.at`（模块解析与历史 `run_file` 同语义）。
///
/// 与旧 `run_file_with_args(path, Vec::new())` 同入口语义：**先**复位
/// 脚本 argv（读文件失败出口也不残留），再读源并执行；stdout 非空时
/// 原样打印。执行体与外层 `auto_lang::run_with_path` 同构：
/// `execute_autovm_with_path` + 32MB 栈。
pub fn run_language_file(path: &str) -> AutoResult<String> {
    // 旧入口每次覆盖前值，同进程串行运行不残留（PLAN-816 T-03 R2-02）。
    auto_lang_core::set_script_args(Vec::new());
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

#[cfg(test)]
mod tests {
    use super::*;

    /// R2-02 串行覆盖：成功/读失败出口 argv 复位 + VM 内无泄漏哨兵。
    /// （`SCRIPT_ARGS` 为进程全局，多测并行会互相污染，故合并为单测。）
    #[test]
    fn run_language_file_resets_script_args_like_legacy_run_file() {
        let dir = tempfile::tempdir().unwrap();
        let ok = dir.path().join("ok.at");
        std::fs::write(&ok, "return 1\n").unwrap();

        auto_lang_core::set_script_args(vec!["previous-invocation-only".into()]);
        assert_eq!(
            auto_lang_core::script_args(),
            vec!["previous-invocation-only".to_string()]
        );
        run_language_file(ok.to_str().unwrap()).expect("valid source runs");
        assert!(
            auto_lang_core::script_args().is_empty(),
            "成功出口不得残留先前 argv"
        );

        // 有效源在 VM 内可见 argv 不得含泄漏哨兵。
        // `process.args().len()` 返回**标量长度**（正确="1"，泄漏="2"），
        // 必须解析为整数并断言 ==1；按逗号数片段会把 1 与 2 都判绿。
        let argv_src = dir.path().join("argv.at");
        std::fs::write(&argv_src, "use auto.process\nreturn process.args().len()\n").unwrap();
        auto_lang_core::set_script_args(vec!["previous-invocation-only".into()]);
        let result = run_language_file(argv_src.to_str().unwrap()).expect("valid source runs");
        assert!(
            auto_lang_core::script_args().is_empty(),
            "执行后 argv 表复位"
        );
        let len: usize = result
            .trim()
            .trim_matches(|c| c == '"' || c == '[' || c == ']')
            .trim()
            .parse()
            .unwrap_or_else(|e| panic!("argv 长度须为标量整数，got {:?}: {e}", result));
        assert_eq!(
            len, 1,
            "VM 内 argv 仅程序路径（长度 1）；泄漏前次哨兵会得到 2，got {result}"
        );

        // 读失败出口同样复位。
        auto_lang_core::set_script_args(vec!["previous-invocation-only".into()]);
        let missing = dir.path().join("missing.at");
        assert!(run_language_file(missing.to_str().unwrap()).is_err());
        assert!(
            auto_lang_core::script_args().is_empty(),
            "读失败出口不得残留先前 argv"
        );
    }
}
