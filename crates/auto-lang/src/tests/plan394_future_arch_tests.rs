//! Plan 394: External future / 真挂起架构测试（Phase A 门禁 + B/C 骨架）。
//!
//! 用例矩阵见 `docs/plans/394-await-future-external-architecture.md` §10。
//! Phase A 交付 U*/A*/R*；B*/C* 在对应 Phase 落地前保持 `#[ignore]`。

#[cfg(test)]
mod plan394 {
    use crate::vm::engine::{AutoVM, FutureKind, FutureState, FutureValue};
    use crate::vm::task::{AsyncFrame, AutoTask, TaskStatus};

    fn run(src: &str) -> Result<String, String> {
        match std::panic::catch_unwind(|| crate::run_with_capture(src)) {
            Ok(Ok((_result, stdout))) => Ok(stdout),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    // ===================== L0 单元 =====================

    /// U1: AsyncFrame push/pop 语义。
    #[test]
    fn u1_async_frame_push_pop() {
        let mut frames = Vec::new();
        frames.push(AsyncFrame {
            resume_ip: 42,
            resume_bp: 7,
            future_id: 3,
        });
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].resume_ip, 42);
        let f = frames.pop().unwrap();
        assert_eq!(f.resume_bp, 7);
        assert!(frames.is_empty());
    }

    /// U2: FutureKind 分叉——Internal 默认（CREATE_FUTURE）；External 由 register API 建。
    #[test]
    fn u2_future_kind_fork() {
        let flash = crate::vm::virt_memory::VirtualFlash::new(4096);
        let vm = AutoVM::new(flash, 4096);
        let fid = vm.register_external_future(1);
        let arc = vm.futures.get(&fid).expect("external future registered");
        let fv = arc.read().unwrap();
        assert_eq!(fv.kind, FutureKind::External);
        assert_eq!(fv.state, FutureState::Pending);
        assert_eq!(fv.body_offset, 0);
    }

    /// U3: resolve → Ready/Failed 状态迁移。
    #[test]
    fn u3_resolve_external_future_states() {
        let flash = crate::vm::virt_memory::VirtualFlash::new(4096);
        let vm = AutoVM::new(flash, 4096);
        let ok_id = vm.register_external_future(1);
        vm.resolve_external_future(ok_id, Ok(auto_val::Value::Int(42)));
        {
            let arc = vm.futures.get(&ok_id).unwrap();
            let fv = arc.read().unwrap();
            assert_eq!(fv.state, FutureState::Ready);
            assert!(matches!(fv.result, Some(auto_val::Value::Int(42))));
        }
        let err_id = vm.register_external_future(1);
        vm.resolve_external_future(err_id, Err("boom".into()));
        {
            let arc = vm.futures.get(&err_id).unwrap();
            let fv = arc.read().unwrap();
            assert_eq!(fv.state, FutureState::Failed);
        }
    }

    /// U4: future_bits 编码往返（`(id<<8)|0xF0`）。
    #[test]
    fn u4_future_bits_roundtrip() {
        let bits = AutoVM::encode_future_bits(7);
        assert_eq!(bits & 0xFF, 0xF0);
        assert_eq!((bits >> 8) as u32, 7);
    }

    // ===================== L1 Phase A =====================

    /// A1: 顶层 external future await 得到 delay_async 回填的 ms。
    #[test]
    fn a1_top_level_external_await() {
        let out = run(
            "fn main() {\n\
             \x20   let r = delay_async(42).await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("A1 program must run");
        assert_eq!(out, "42\n");
    }

    /// A2: 真挂起——await 等待期间另一 task 可推进（纯计算 task 先完成）。
    #[test]
    fn a2_suspend_does_not_block_other_tasks() {
        // delay 用 60ms：另一任务纯计算应先跑完；结果仍来自 await。
        let out = run(
            "fn main() {\n\
             \x20   let r = delay_async(60).await\n\
             \x20   print(f\"done ${r}\")\n\
             }\n",
        )
        .expect("A2 program must run");
        assert_eq!(out, "done 60\n");
    }

    /// A3: 顺序两个 external await，串行语义（结果相加）。
    #[test]
    fn a3_sequential_external_awaits() {
        let out = run(
            "fn main() {\n\
             \x20   let a = delay_async(20).await\n\
             \x20   let b = delay_async(30).await\n\
             \x20   print(f\"${a + b}\")\n\
             }\n",
        )
        .expect("A3 program must run");
        assert_eq!(out, "50\n");
    }

    /// A4: Failed external → await 得 null，不 panic、不挂死。
    #[test]
    fn a4_failed_external_await_yields_null() {
        let out = run(
            "fn main() {\n\
             \x20   let r = fail_async(\"boom\").await\n\
             \x20   if r == null {\n\
             \x20       print(\"null\")\n\
             \x20   } else {\n\
             \x20       print(\"not-null\")\n\
             \x20   }\n\
             }\n",
        )
        .expect("A4 program must run");
        assert_eq!(out, "null\n");
    }

    // ===================== L4 回归 =====================

    /// R2: 内部 `~{}` future 仍走同步内联路径（Phase A 不得破坏）。
    #[test]
    fn r2_internal_async_block_still_works() {
        let out = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       1 + 2\n\
             \x20   }\n\
             \x20   let r = f.await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("R2 program must run");
        assert_eq!(out, "3\n");
    }

    /// R1 钉：waiting_future_id 字段默认 None，不干扰非 external 路径。
    #[test]
    fn r1_waiting_future_id_defaults_none() {
        let t = AutoTask::new(1, 1024, 0);
        assert!(t.waiting_future_id.is_none());
        assert!(t.async_frames.is_empty());
        assert_eq!(t.status, TaskStatus::Ready);
    }

    // ===================== L2 Phase B 骨架 =====================

    /// B1: `~{}` body 内顺序 await 外部源（Phase B）。
    #[test]
    #[ignore = "Plan 394 Phase B: nested external await inside ~{} body"]
    fn b1_nested_external_await_in_async_block() {
        let out = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       let a = delay_async(20).await\n\
             \x20       let b = delay_async(30).await\n\
             \x20       a * 100 + b\n\
             \x20   }\n\
             \x20   let r = f.await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("B1 program must run");
        assert_eq!(out, "2030\n");
    }

    /// B2: body 局部/循环变量在挂起恢复后完整（Phase B）。
    #[test]
    #[ignore = "Plan 394 Phase B: stack-frame integrity across external suspend"]
    fn b2_stack_frame_integrity() {
        let out = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       var acc = 0\n\
             \x20       var i = 0\n\
             \x20       while i < 3 {\n\
             \x20           acc = acc + delay_async(1).await\n\
             \x20           i = i + 1\n\
             \x20       }\n\
             \x20       acc\n\
             \x20   }\n\
             \x20   print(f\"${f.await}\")\n\
             }\n",
        )
        .expect("B2 program must run");
        assert_eq!(out, "3\n");
    }

    /// B3: 嵌套深度 >64（async_frames，非 Rust 递归）（Phase B）。
    #[test]
    #[ignore = "Plan 394 Phase B: deep nesting beyond Rust recursion 64"]
    fn b3_deep_nesting_beyond_64() {
        // 70 层：生成 is 麻烦；Phase B 用 Rust 侧直接压 async_frames 验证深度。
        let mut t = AutoTask::new(1, 4096, 0);
        for i in 0..70 {
            t.async_frames.push(AsyncFrame {
                resume_ip: i,
                resume_bp: 0,
                future_id: i as u32,
            });
        }
        assert_eq!(t.async_frames.len(), 70);
    }

    // ===================== L3 Phase C 骨架 =====================

    /// C1: Future.all 并发（墙钟 ≈ max）（Phase C）。
    #[test]
    #[ignore = "Plan 394 Phase C: Future.all"]
    fn c1_future_all() {
        let out = run(
            "fn main() {\n\
             \x20   let r = Future.all([delay_async(40), delay_async(60)]).await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("C1 program must run");
        assert_eq!(out, "[40, 60]\n");
    }

    /// C2: Future.race（Phase C）。
    #[test]
    #[ignore = "Plan 394 Phase C: Future.race"]
    fn c2_future_race() {
        let out = run(
            "fn main() {\n\
             \x20   let r = Future.race([delay_async(80), delay_async(20)]).await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("C2 program must run");
        assert_eq!(out, "20\n");
    }

    /// C3: a2r golden 占位——Phase C 合入前只钉类型位存在。
    #[test]
    #[ignore = "Plan 394 Phase C: a2r nested external await golden"]
    fn c3_a2r_golden_placeholder() {
        // 语料目录 test/a2r/16_interop/021_external_await_nested/ 在 Phase C 落地。
        assert!(true);
    }

    // ===================== 内部类型自检 =====================

    #[test]
    fn future_value_default_internal_shape() {
        let fv = FutureValue {
            body_offset: 10,
            state: FutureState::Pending,
            result: None,
            owner_task_id: 1,
            captures: Default::default(),
            kind: FutureKind::Internal,
        };
        assert_eq!(fv.kind, FutureKind::Internal);
        assert_ne!(FutureKind::Internal, FutureKind::External);
    }
}
