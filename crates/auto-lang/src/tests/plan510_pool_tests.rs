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

    /// Plan 510 G1-1:http_server handler 实参入池必须走 add_string 咽喉
    /// (dedup 可见 + rc 覆盖 + push/release 配平)。裸推(P-053-5 收口
    /// 漏掉本文件)产生无计数活引用——消费侧 release 即多扣,over-release
    /// 注入源(musk 实机:点击会话实参由 id 漂移成会话名/404 JSON)。
    #[test]
    fn http_server_param_interning_is_pool_visible() {
        use crate::vm::engine::AutoVM;
        use crate::vm::rc::pool_idx_nv;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;
        use std::sync::atomic::Ordering;

        let vm = AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024);
        let mut task = AutoTask::new(1, 1024, 0);

        crate::vm::ffi::http_server::push_str_arg(&vm, &mut task, "/session/8f20abcd");

        // 栈顶实参必须是 TAG_STRING 池索引
        let nv = task.ram.pop_nv();
        let idx = pool_idx_nv(nv).expect("handler arg must be TAG_STRING") as usize;

        // (a) dedup 可见:同内容 add_string 命中同槽(裸推无键必 miss)
        let again = vm.add_string(b"/session/8f20abcd".to_vec());
        assert_eq!(again, idx, "same content must dedup-hit the pushed slot");

        // (b) rc 覆盖且配平:入栈恰好 +1 份;release 一次归零
        let rc_now = {
            let st = vm.pool_state.read().unwrap();
            assert!(idx < st.rc.len(), "rc array must cover the slot (ensure_len)");
            st.rc[idx].load(Ordering::Relaxed)
        };
        assert_eq!(rc_now, 1, "push must retain exactly one share");
        vm.pool_release(idx);
        let st = vm.pool_state.read().unwrap();
        assert_eq!(
            st.rc[idx].load(Ordering::Relaxed),
            0,
            "release must pair with the push retain"
        );
        // 归零槽位健康入 freelist(rc==0 条目,非幻影)
        assert!(st.freelist.contains(&idx), "freed slot must enter freelist");
    }

    /// Plan 510 G1-2:native 返回串入栈必须配平(add_string + retain)。
    /// 裸 push(add_string 后直接 push_nv)产生无计数引用:返回值被
    /// POP 即多扣;dedup 命中他人活槽时直接把活槽打到 0 进 freelist
    /// (幻影条目主通道)。以 url_encode/env_var 为族代表钉契约。
    #[test]
    fn native_string_returns_are_counted() {
        use crate::vm::engine::AutoVM;
        use crate::vm::native::{pop_arg_nv, shim_env_var, shim_url_encode};
        use crate::vm::rc::pool_idx_nv;
        use crate::vm::task::AutoTask;
        use crate::vm::virt_memory::VirtualFlash;
        use std::sync::atomic::Ordering;

        let vm = AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024);

        // url_encode("?a=b c") → "%3Fa%3Db+c"(新内容,dedup 必 miss)
        let mut task = AutoTask::new(1, 1024, 0);
        vm.intern_runtime_str(&mut task, b"?a=b c".to_vec());
        shim_url_encode(&mut task, &vm).expect("url_encode runs");
        let nv = pop_arg_nv(&mut task);
        let idx = pool_idx_nv(nv).expect("url_encode must return TAG_STRING") as usize;
        let st = vm.pool_state.read().unwrap();
        assert!(
            idx < st.rc.len(),
            "rc array must cover native-returned slot"
        );
        assert_eq!(
            st.rc[idx].load(Ordering::Relaxed),
            1,
            "native return push must retain exactly one share (url_encode)"
        );
        drop(st);
        // 消费侧 POP 一次即配平归零(无多扣)
        vm.pool_release(idx);
        assert_eq!(
            vm.pool_state.read().unwrap().rc[idx].load(Ordering::Relaxed),
            0,
            "single release must zero the share"
        );

        // env_var(存在的环境变量)同契约
        let mut task = AutoTask::new(2, 1024, 0);
        vm.intern_runtime_str(&mut task, b"PATH".to_vec());
        shim_env_var(&mut task, &vm).expect("env_var runs");
        let nv = pop_arg_nv(&mut task);
        let idx = pool_idx_nv(nv).expect("env_var must return TAG_STRING") as usize;
        assert_eq!(
            vm.pool_state.read().unwrap().rc[idx].load(Ordering::Relaxed),
            1,
            "native return push must retain exactly one share (env_var)"
        );
    }
}
