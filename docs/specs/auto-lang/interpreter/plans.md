# interpreter 相关 plan 索引

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|------|------|------|------|-----------|
| 068 | autovm-bigvm | ✅ Phase 9 完成（文内另有"部分完成"节） | old/ | 九阶段建成 AutoVM 字节码引擎，本模块的执行底座 |
| 069 | autovm-global-vars | ✅（plan-reports/07 标 Complete，文件无独立状态行） | old/ | 任务复用实现 REPL 顶层变量持久化，"持久状态"需求的起点 |
| 073 | bigvm-migration-roadmap | ✅ COMPLETE (100%) | old/ | AutoVM 全量替换 Evaluator 的路线图：97.4% 测试通过、23.77x 提速 |
| 075 | config-template-modes | ✅ COMPLETE (100%) | old/ | SCRIPT/CONFIG/TEMPLATE 模式差异由独立 Codegen 吸收，VM 保持 mode-agnostic（ADR-03） |
| 080 | autovm-stack-frame-bug | ✅（plan-reports/07 标 Complete，文件无独立状态行） | old/ | 主任务 bp=0 时栈与局部变量共享内存导致 REPL 值累积；催生 RESERVE_STACK（ADR-04） |
| 081 | autovm-default-mode | ✅（Phase 1/2 均标 COMPLETE） | old/ | AutoVM 成为默认引擎，引入 ExecutionEngine 与环境变量覆盖（ADR-01/05） |
| 091 | universe-removal | ✅ 基本完成 | old/ | 删除 eval.rs/interp.rs（约 7,167 行，commit 6862bb4），interpreter/ 重建为 AutoVM 薄封装（ADR-02） |
| 177 | vm-file-test-framework | Planned（文件无状态行；stdout 捕获已落地于 run_with_capture / execute_with_engine_capture） | old/ | 测试用 stdout 捕获先进来，文件式 .expected.out 框架未完成 |
| 197 | vm-adt-generic-lists-pattern-debug | ✅ COMPLETE | old/ | Task 9 把泛型注册表传入 VM 供运行时字段名查找，VmInterpreter 沿用 |
| 221 | nanboxing-migration | ✅ COMPLETE | old/ | VM 值表示切到 NaN-boxing，结果提取协议按 nanbox 标记解码 |
| 298 | remove-non-nanbox | ✅ COMPLETED (2026-06-12) | archive/ | 拆除 non-nanbox 安全网，vm_interpreter.rs 只走 nanbox 单路径 |
| 355 | fix-persistent-session-fn-body-recursion | ✅ 已修复（commit add04447，2026-06-27） | archive/ | 持久 session 每次新建 runtime 耗尽线程栈；修复模式=独立 8MB 栈线程，run_autovm 用同款 4MB 方案 |

## 编号备注

- 活跃区（`docs/plans/`）另有一个 **355-a2r-async-await-transpilation**，与本表 355
  （archive 的持久 session 修复）同号不同 plan；本表 355 指 archive 那份。
- 重编号背景：原 355-rust-library-replication 已改为 **347**（2026-07-23，原号让给
  a2r-async-await-transpilation）；327/336/337/338/342/351/359 同期改为
  317/318/320/322/330/346/348。以上均与本模块无直接关系，仅备查。
- plan-reports/07-vm-runtime.md 的"Source Plans"链接多指向 `docs/plans/` 根目录，
  实际文件在 `old/` 子目录，链接已过时（上表"归档"列以实际位置为准）。

> 来源: docs/plan-reports/07-vm-runtime.md、docs/plans/old/、docs/plans/archive/
