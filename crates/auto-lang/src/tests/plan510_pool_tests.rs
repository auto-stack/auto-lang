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

    /// Plan 510 G3:审计钩子自证——强造一次多扣款(release 未 retain 的
    /// 槽),underflow_events 必须记账 1(计数器探测链路可用性)。
    #[test]
    fn underflow_counter_detects_forced_over_release() {
        use crate::vm::engine::AutoVM;
        use crate::vm::virt_memory::VirtualFlash;

        let vm = AutoVM::new(VirtualFlash::new_with_code(vec![]), 1024);
        let idx = vm.add_string(b"uncounted".to_vec()); // rc=0,无持有
        let before = vm.pool_health().underflow_events;
        vm.pool_release(idx); // 多扣款:0 → 0xFFFFFFFF
        let after = vm.pool_health().underflow_events;
        assert_eq!(
            after - before,
            1,
            "forced over-release must be counted (audit hook wiring)"
        );
    }

    /// Plan 510 G2/G3 浸泡(短跑档,入日常门禁):字符串 churn 全链
    /// (内化/拼接/容器进出/参数传递)跑毕,记账必须自持——
    /// 无多扣款下溢、无幻影清扫、全池份额归零、freelist 已回收。
    #[tokio::test]
    async fn pool_soak_churn_short() {
        pool_soak_assert(800).await;
    }

    /// Plan 510 G2 浸泡(长跑档,验收等效 churn 用):
    /// `cargo test -p auto-lang --lib pool_soak_churn_long -- --ignored`
    /// 轮数可由 P510_SOAK_ITERS 覆盖(缺省 100_000)。
    #[tokio::test]
    #[ignore = "soak long-run: P510_SOAK_ITERS 可调,显式 --ignored 触发"]
    async fn pool_soak_churn_long() {
        let iters: usize = std::env::var("P510_SOAK_ITERS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(100_000);
        pool_soak_assert(iters).await;
    }

    /// churn 载体:有界长度 f-string 内化(唯一内容→freelist 复用面;重复
    /// 后缀→dedup 命中面)+ 拼接 + List<str> 容器进出 + 跨 fn 参数传递 +
    /// 覆盖赋值——覆盖 add_string dedup/freelist、POP release、容器侧份额
    /// (rc.rs child_pool_idxs)、ListData<String> 读回配平。内容恒有界
    /// (长度不随轮数增长,避免 O(n²) 字节量;churn 强度由轮数承载)。
    /// PLAN-608 T04 相位:l.push(s) 覆盖 CALL_NAT 死区结算路径;无类型
    /// 接收者 m.push 实测同走 CALL_NAT(is_native 按方法名命中)——
    /// CALL_SPEC→resolve 形态由专项测试 pool_settles_callspec_list_push_str
    /// (索引接收者,trace 实证 50 CALL_SPEC)覆盖。
    async fn pool_soak_assert(iters: usize) {
        let code = format!(
            r#"
fn tag(prefix str, i int) str {{
    f"${{prefix}}#${{i}}-tail"
}}

fn mklist() -> List<str> {{
    List<str>.new([])
}}

fn main() int {{
    var acc str = "seed"
    for i in 0..{iters} {{
        let s = tag("item", i % 512)
        acc = s
        var l List<str> = List<str>.new([s, acc])
        l.push(tag("push", i % 512))
        acc = l.get(0)
        if l.len() > 64 {{
            l = List<str>.new([])
        }}
        var m = mklist()
        m.push(tag("spec", i % 512))
        if i % 3 == 0 {{
            var waste str = tag("waste", i % 128)
            waste = "overwritten"
        }}
    }}
    print(acc)
    0
}}
"#
        );
        let (vm, out) = run_code_vm_checked(&code).await;
        assert!(!out.is_empty(), "churn program must produce output");

        let h = vm.pool_health();
        assert_eq!(
            h.underflow_events, 0,
            "多扣款下溢必须为 0(over-release 注入源已清偿)"
        );
        assert_eq!(
            h.phantom_drops, 0,
            "幻影 freelist 清扫必须为 0(注入源已清偿,防线不触发)"
        );
        assert_eq!(
            h.live_shares, 0,
            "程序结束+任务收尾后全池份额必须归 0(配平自持);freelist_len={}, pool_len={}",
            h.freelist_len, h.pool_len
        );
        // 池规模稳定:freelist 回收了运行期死亡槽(复用面非零)。
        assert!(
            h.freelist_len > 0,
            "churn 后应有可复用空闲槽(freelist 恢复,慢性泄漏消失): {h:?}"
        );
    }

    /// 编译并跑一段 Auto 源码到完成,返回 (vm, stdout);跑毕做任务残余
    /// 栈释放(同 tests_rc_lifecycle::run_code_vm 口径)。
    /// PLAN-608:收尾补 reap_all(静止点收割宽限队列,同 rc_stats 语义)——
    /// ListData<i32> 负哨兵容器死于任务收尾时进 dying 宽限队列,不收割
    /// 则其容器池份额残留在 live_shares 读数里(soak 终态断言失真)。
    async fn run_code_vm_checked(code: &str) -> (crate::vm::engine::AutoVM, String) {
        let (vm, stdout, entry, _result_type) =
            crate::create_vm_from_source(code).expect("compile failed");
        let tid = vm.spawn_task(entry, 65536);
        vm.run_task_loop().await;
        if let Some(arc) = vm.tasks.get(&tid) {
            let mut t = arc.lock().await;
            vm.rc_release_task_stack(&mut t);
        }
        vm.reap_all();
        vm.tasks.remove(&tid);
        let out = stdout.read().unwrap().clone();
        (vm, out)
    }

    // ====================================================================
    // PLAN-608: CALL_SPEC 分发路径的池份额结算(KD-VM6)。
    //
    // 关键路径区分(T-01 实证):静态可解析的方法调用编译为 CALL_NAT
    //(自带 rc_release_slot_range 死区结算,池按内容释放,天然配平);
    // 只有 codegen 无法静态解析的调用形态走 CALL_SPEC——未注册方法名
    //(trimEnd/includes/indexOf 族 → 内联臂)或索引接收者
    //(arr[0].push → resolve→shim)。这两条路径修复前暂存份额/接收者
    // 份额每调用孤儿 +1(live_shares 终值=调用数,red 实证 50/50)。
    // ====================================================================

    /// PLAN-608 AC-03:CALL_SPEC→resolve→shim_list_push 的字符串元素
    /// 暂存池份额结算。索引接收者(arr[0])形态——codegen 无法静态解析
    /// 方法目标,emit CALL_SPEC→resolve→shim_list_push(trace 实证
    /// 50 CALL_SPEC)。修复前 resolve 分支无 CALL_NAT 式死区,每 push
    /// 暂存池份额孤儿 +1。
    #[tokio::test]
    async fn pool_settles_callspec_list_push_str() {
        let code = r#"
fn mk() -> List<str> {
    List<str>.new([])
}
fn main() int {
    var n = 0
    var arr List<List<str>> = List<List<str>>.new([])
    while n < 50 {
        arr.push(mk())
        arr[0].push(f"item-${n % 16}")
        n = n + 1
    }
    print("done")
    0
}
"#;
        let (vm, out) = run_code_vm_checked(code).await;
        assert!(out.contains("done"), "program must complete: [{}]", out);
        let h = vm.pool_health();
        assert_eq!(
            h.underflow_events, 0,
            "结算不得引入多扣款下溢"
        );
        assert_eq!(
            h.live_shares, 0,
            "CALL_SPEC push 50 次后池份额必须配平(修复前每调用孤儿 +1): {h:?}"
        );
    }

    /// PLAN-608 AC-04:CALL_SPEC 内联 str 臂接收者池份额结算
    /// (未注册方法 trimEnd → resolve miss → engine.rs 内联臂,
    /// 接收者 copy-on-load 池份额被 raw pop)。
    #[tokio::test]
    async fn pool_settles_callspec_str_method_recv() {
        let code = r#"
fn f(s str) -> str {
    return s.trimEnd()
}
fn main() int {
    var n = 0
    var acc str = "seed"
    while n < 50 {
        acc = f(f"item-${n % 16}   ")
        n = n + 1
    }
    print(acc)
    0
}
"#;
        let (vm, out) = run_code_vm_checked(code).await;
        assert!(!out.is_empty(), "program must produce output: [{}]", out);
        let h = vm.pool_health();
        assert_eq!(
            h.underflow_events, 0,
            "结算不得引入多扣款下溢"
        );
        assert_eq!(
            h.live_shares, 0,
            "trimEnd 50 次调用后接收者池份额必须配平(修复前每调用孤儿 +1): {h:?}"
        );
    }
}
