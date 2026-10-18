# PLAN-812 验证与发布收据

- Code baseline: b2a75c1c2d3f83068b2d262d1448245292326c6c; installer/resource validation descendant: 2ba18261ffb3d595daff90e8d1a8214571c8469a.
- Windows cargo check -p auto-lang: pass; existing 409 library warnings. Scoped transport tests with ui-iced,test-cases: 9/9 pass, 6778 skipped by explicit filter; rustfmt transport.rs and git diff --check pass.
- Linux cargo build --release --locked -p auto-playground -j 8: pass, 3m44s; current binary SHA256 847132bbd1b6d7d2a9f5e434b2f3e379daa9b7260020a2f428b54dd871a71b11.
- Frozen inputs: Windows and Linux two changed files equal after CRLF normalization; remaining crates/packages/Cargo/.cargo inputs unchanged across docs-only rebase (e7177e3fa→b2a75c1c2 range-diff 1/1 equal). Lock SHA256 39820fdef578729b42a0371adaa22aa0deaf2b62c3b702c6bf6868602f9544c7.
- First deployment caught missing stdlib/auto. Automatic rollback was exercised: service active and original binary hash 81772122c972c24dc0727d1be6b4b152bed81eb79c656cad2789f120ad1be9a4 verified. Fix: bundle full stdlib including assembly-providers.json, precheck assets, and isolate local verification from complete checkout.
- Isolated payload verification: native Git worktree moved to build-source after clean wt-guard; only actual payload occupies compiled auto-lang resource root. Local Linux smoke pass. No compiler or VM algorithm was changed for this packaging repair.
- Final archive SHA256 cf13cb3886bb353485b8fe171cae3db34b35818075ec52afe0e7e5da3a5fc390. Server upload checksum matched. ldd resolved all libraries.
- Final production restart: MainPID=601412, active, ExecMainStartTimestamp=2026-10-19 05:17:19 CST. systemd unit unchanged; backup retained at /var/backups/auto-playground/20261019-687efdd35.
- Local Linux/server/public HTTPS smoke: 1343 examples; stdout 7, 15 bytecode instructions with meta; Rust/C/Python/TypeScript/ABT files nonempty; server-backed 09-multi-module outputs Hello, Playground! See linux-smoke.json.txt, server-smoke.json.txt, public-smoke.json.txt.
- Browser on https://auto-lang.cn/zh/playground: Run visibly produced Bytecode instruction rows and output 7; Trans visibly produced Rust function/source pane. Screenshot online-trans.png. API checks used public same-origin URLs, not localhost fallback.
- Omission/defer/workaround audit: no dropped acceptance, no frontend workaround, no new debt. Existing compiler warnings recorded above; not introduced by these declaration changes.
- Merge-owned daily gate: pending final review, then bare cargo t. No VM/compiler/transpiler/ui_gen/book/aavm implementation touched, so tv/tt/tu/tb/taa not triggered by scope. tf remains batch-only.
## 最终 merge-owned 门禁

冻结运行源码与 reviewed 7cab6ca2b 相同；后继提交仅规范/投影/证据。最终裸 cargo t：5252/5252 pass、1549 个预配置 skipped、142.261s；仅 NEXTEST_TEST_THREADS=8 限流，无 CARGO_TARGET_DIR 重定向、无删选测试。tv/tt/tb/tu/taa 未触发，tf 批量到期检查归收尾。

前两轮失败保留原始记录：首轮父 CARGO_TARGET_DIR 导致 fixture 输出目录错误（已撤销并清掉本次无效 marker）；第二轮 AC worker 冷编译超时及一次 P765 IAT heap 状态观测。随后独立构建 worker、AC+P4 定向 14/14（P4=87.863s）、P765 1/1 后最终全档通过，未修改测试选择/实现以掩盖失败。环境候选 P812-D1/P812-D2 已追加 KNOWN-DEBT，其他会话 WIP 未包含进该提交。

完整日志与 Linux 归档恢复件：D:/autostack/.recovery/plan-812-20261019/（含三轮 merge-daily*.log、定向日志、程序/归档哈希与 specs v48 字节备份）。该目录不含 sudo 凭据。