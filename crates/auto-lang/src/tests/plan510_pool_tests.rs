//! Plan 510 / P499-7: VM native ID 冲突与字符串池记账回归钉。
//!
//! P499-7 根因(2026-09-01 分诊): native_catalog 把 Log 族
//! (debug/info/warn/error = 1800-1803)登记进 NATIVE_ID_ENTRIES,而
//! native.rs 的 Shell 族(Plan 011)早已占用同段 ID 并在 engine.rs 显式
//! 注册。`#error(...)` 经 CALL_NAT 1803 派发到 shim_shell_exit →
//! ExitRequested(-1),程序当场死亡(cookbook cb_devtools_log_error 红)。
//! 本文件钉住:Log 四名解析到的 ID 必须离开 Shell 段,且 #error 运行
//! 不得退出。

#[cfg(test)]
mod plan510 {

    /// 运行 .at 源码并返回捕获的 stdout;Err 携带失败描述。
    fn run(src: &str) -> Result<String, String> {
        match std::panic::catch_unwind(|| crate::run_with_capture(src)) {
            Ok(Ok((_result, stdout))) => Ok(stdout),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    /// Log 四宏的行为契约:记录并继续,不得命中 Shell 族 shim。
    /// 修复前:#error → CALL_NAT 1803 → shim_shell_exit → ExitRequested,
    /// print("done") 永不执行,stdout 缺 "done"。
    /// 注:Log.* shim 走 Rust println/eprintln,不进 VM 捕获通道,故捕获
    /// stdout 只含 VM print 的 "done";「跑完」即契约本体。
    #[test]
    fn hash_log_macros_log_and_continue() {
        let out = run(
            "fn main() {\n\
             \x20   #info(\"i\")\n\
             \x20   #warn(\"w\")\n\
             \x20   #error(\"e\")\n\
             \x20   print(\"done\")\n\
             }\n",
        )
        .expect("#info/#warn/#error program must run to completion");
        assert_eq!(out, "done\n");
    }

    /// 数据级契约:Log 族 ID 与 Shell 族常量(1800-1803)零交集。
    /// 这是 ID 撞号回归的直接钉——任何一侧改号都必须显式看到本测试。
    #[test]
    fn log_native_ids_do_not_collide_with_shell_family() {
        use crate::vm::native::{
            NATIVE_SHELL_EXIT, NATIVE_SHELL_EXPORT, NATIVE_SHELL_SYSTEM,
            NATIVE_SHELL_SYSTEM_STATUS,
        };
        use crate::vm::native_registry::NATIVE_ID_MAP;

        let shell_ids = [
            NATIVE_SHELL_SYSTEM,
            NATIVE_SHELL_SYSTEM_STATUS,
            NATIVE_SHELL_EXPORT,
            NATIVE_SHELL_EXIT,
        ];
        for name in ["Log.debug", "Log.info", "Log.warn", "Log.error"] {
            let id = *NATIVE_ID_MAP
                .get(name)
                .unwrap_or_else(|| panic!("{name} missing from NATIVE_ID_MAP"));
            assert!(
                !shell_ids.contains(&id),
                "{name} resolved to {id}, which belongs to the Shell native family — \
                 CALL_NAT would dispatch to a shell shim (P499-7 collision)"
            );
        }
    }
}
