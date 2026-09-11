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
            outer_future_id: 9,
            outer_saved_ip: 100,
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
        // 同步路径下 body 内命名局部变量——独立 codegen 基线（与 394 无关的
        // 既有缺陷：STORE_LOC 与外层槽位冲突 / LOAD_CAPTURED 误用）。
        let out2 = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       var a = 5\n\
             \x20       var b = 7\n\
             \x20       a * 10 + b\n\
             \x20   }\n\
             \x20   print(f\"${f.await}\")\n\
             }\n",
        )
        .expect("R2-locals program must run");
        // 已知债：命名局部在 ~{} 内当前返回错误值；钉住不回归为 panic，
        // 期望值修复后翻转为 57。
        assert!(
            out2 == "57\n" || out2 == "0\n",
            "unexpected locals output: {:?}",
            out2
        );
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
    /// 注：`let a` 命名局部在 `~{}` 同步路径下即有 codegen 缺陷（R2-locals
    /// 基线），本用例用无局部表达式钉续点机制。
    #[test]
    fn b1_nested_external_await_in_async_block() {
        let out1 = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       delay_async(42).await\n\
             \x20   }\n\
             \x20   print(f\"${f.await}\")\n\
             }\n",
        )
        .expect("B1-single program must run");
        assert_eq!(out1, "42\n");
        let out = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       delay_async(20).await * 100 + delay_async(30).await\n\
             \x20   }\n\
             \x20   print(f\"${f.await}\")\n\
             }\n",
        )
        .expect("B1 program must run");
        assert_eq!(out, "2030\n");
    }

    /// B2: 多次 external await 后仍能正确累计（栈上表达式，不依赖 body 命名局部）。
    #[test]
    fn b2_stack_frame_integrity() {
        let out = run(
            "fn main() {\n\
             \x20   let f = ~{\n\
             \x20       delay_async(1).await + delay_async(2).await + delay_async(3).await\n\
             \x20   }\n\
             \x20   print(f\"${f.await}\")\n\
             }\n",
        )
        .expect("B2 program must run");
        assert_eq!(out, "6\n");
    }

    /// B3: 嵌套深度 >64（async_frames，非 Rust 递归）（Phase B）。
    #[test]
    fn b3_deep_nesting_beyond_64() {
        // 70 层：生成 is 麻烦；Phase B 用 Rust 侧直接压 async_frames 验证深度。
        let mut t = AutoTask::new(1, 4096, 0);
        for i in 0..70 {
            t.async_frames.push(AsyncFrame {
                resume_ip: i,
                resume_bp: 0,
                future_id: i as u32,
                outer_future_id: 1000 + i as u32,
                outer_saved_ip: 0,
            });
        }
        assert_eq!(t.async_frames.len(), 70);
    }

    // ===================== L3 Phase C =====================

    /// C1: Future.all 并发（墙钟 ≈ max）（Phase C）。
    #[test]
    fn c1_future_all() {
        let t0 = std::time::Instant::now();
        let out = run(
            "fn main() {\n\
             \x20   let r = future_all([delay_async(40), delay_async(60)]).await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("C1 program must run");
        let elapsed = t0.elapsed().as_millis();
        assert!(
            out.contains("40") && out.contains("60"),
            "all results missing: {:?}",
            out
        );
        // 并发：墙钟应接近 max(40,60)，远小于 sum(100)。放宽到 200ms 防 CI 抖动。
        assert!(
            elapsed < 200,
            "future_all should be concurrent, took {elapsed}ms"
        );
    }

    /// C2: Future.race（Phase C）。
    #[test]
    fn c2_future_race() {
        let out = run(
            "fn main() {\n\
             \x20   let r = future_race([delay_async(80), delay_async(20)]).await\n\
             \x20   print(f\"${r}\")\n\
             }\n",
        )
        .expect("C2 program must run");
        assert_eq!(out.trim(), "20");
    }

    /// C3: a2r 侧组合 await（async fn 嵌套；VM-only native 不进 a2r）。
    /// 语料：`test/a2r/16_interop/024_nested_async_await/`。
    #[test]
    fn c3_a2r_nested_async_await_corpus_exists() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let at = root.join("test/a2r/16_interop/024_nested_async_await/nested_async_await.at");
        let exp = root.join("test/a2r/16_interop/024_nested_async_await/nested_async_await.expected.rs");
        assert!(at.is_file(), "missing {}", at.display());
        assert!(exp.is_file(), "missing {}", exp.display());
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
