---
plan_id: PLAN-746
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: pg-mem-fix
author: [agent]
created_at: 2026-10-09
updated_at: 2026-10-09
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-playground, auto-lang/vm, playground-vue]
current_step: 7
total_steps: 7
---

# [PLAN-746] playground 内存问题修复（PG-MEM-1/2/3）

## 0. 变更摘要

修复 2026-10-09 playground 全量示例走查（1343 例）暴露的三类内存问题
（已登记 `docs/plans/KNOWN-DEBT-AND-RISKS.md` PG-MEM-1/2/3）：

1. **PG-MEM-1（high）**：`/api/run` 无超时/取消——挂起执行（死循环、阻塞
   调用）在服务端永不终止并持续吃内存（实测串行全量后 16MB→1434MB，
   其中 ~740MB 来自 5 个挂起示例；8 并发批 20GB+）。
2. **PG-MEM-2（low）**：`print(x, end=" ")` kwargs 形态不报错反而挂起并
   以 ~3MB/s 膨胀输出缓冲（两个 think-python 围栏 90s 内 +210/+348MB）。
3. **PG-MEM-3（low）**：单进程逐运行基线缓涨 ~1MB/示例（148 条 >1MB）。

方案核心：AutoVM 执行引入**协作式取消**（指令燃料 + 壁钟截止，注入
`execute_task` 执行循环）——Rust 无法安全杀线程，燃料是让 VM 自己停下
的唯一根治路径；playground run_handler 传超时参数并把超时映射为
明确的 504/错误响应；print kwargs 在 codegen 编译期拒绝；
基线缓涨经 profiling 定位后修复主要保留源。

## 1. 目标

**目标（全部解决三类问题）**：

- G-1：playground 运行接口具备执行上限：任何示例（含死循环/挂起形态）
  在限定时间/指令量内必然返回，服务端不留失控执行线程。
- G-2：`print(..., end=...)` 等非法 kwargs 形态编译期报错（明确诊断信息），
  不再挂起膨胀。
- G-3：逐运行基线缓涨定位主要保留源并修复，全量走查后服务端内存增长
  降至可接受水位（见 AC-03 阈值）。
- G-4：全部修复以回归测试固化；全量走查脚本（带逐示例内存记录）复测
  通过。

**非目标**：

- 不重构 VM 调度器/不引入线程抢占（协作式取消足够覆盖观测面）。
- 不修改 FFI 同步阻塞调用自身（如 sync http client 的内部超时）——
  燃料管不到阻塞 native，playground 层超时兜底即可。
- 不改 playground 前端 UI（本计划纯后端/VM）。
- 不处理已知 `#[ignore]` 语料本身的 VM 语义缺陷（plan231 无限循环等
  归原语料隔离债，本计划只保证它们**不再失控吃内存**）。

**受影响仓/模块**：本仓 `crates/auto-playground`（routes/run.rs）、
`crates/auto-lang`（vm/engine.rs、vm/codegen.rs、vm 测试）、
`docs/specs/auto-playground/`（规范增量）。

## 2. 架构方案

```
前端 Run ──POST /api/run──▶ run_handler (新增 timeout_secs 参数, 默认 10s)
                                │ spawn_blocking
                                ▼
                    run_with_capture_*_with_deadline(code, deadline)
                                │ 16MB 栈线程内
                                ▼
                    execute_autovm_with_path(..., fuel/deadline)
                                │
              ┌─────────────────┼──────────────────┐
              ▼                 ▼                  ▼
        execute_task      阻塞 native        超时映射
        循环内每 N 条     （自带超时/管不到）  VMError::Timeout
        指令查燃料/截止                    → run_handler 返回
                                           明确错误(非500 panic)
```

- **燃料机制**：AutoVM 持有 `Option<ExecutionBudget>`（指令计数 + 截止
  Instant）。`execute_task` 内层指令循环每执行 K 条指令（K≈1024，实现期
  可调）检查一次：超预算即返回 `VMError::Timeout`，run_task_loop 收到后
  终止全部任务并向上传播。默认值无预算（None）——**零行为变更**；
  仅 playground/显式调用方传入预算。
- **playground 层**：`/api/run` 请求体加 `timeout_secs`（默认 10，上限
  60）；超时返回结构化错误（`{"error": "execution timeout (10s)"}`），
  HTTP 状态用 200+error 字段还是 504 实现期定（须与前端 Run 错误展示
  兼容——usePlaygroundFull 已渲染 result 的 Error: 前缀，选改动最小面）。
- **print kwargs**：codegen 对 `print`/`println`/`write`/`say`/`assert*`
  内建族做参数校验：出现 kwargs（`end=`/`sep=` 等）即编译错误
  "`print` does not accept keyword arguments (got `end`)"；或按 SD-02 裁定
  的替代语义执行。挂起膨胀根因（为何 kwargs 导致 3MB/s 增长）在 T-02
  调查任务中定位并写进测试注释。
- **基线（PG-MEM-3）**：T-04 profiling 定位（候选：惰性 native 注册表、
  类型缓存、CPython attach、Windows 工作集不归还）→ 修主要源；
  若确认为"工作集不归还"而非真泄露，则以 SD-03 规范注记+阈值验收收口。

## 3. 技术栈

- Rust（auto-lang VM：engine/codegen；auto-playground axum 路由）
- 测试：cargo 单测（vm/codegen 回归）+ `scratch/playground-check/` 走查脚本复测
- 内存测量：tasklist 工作集（既有 run_all_examples_mem.py 机制）

## 4. 需求分析与背景调查

**授权**：用户 2026-10-09 指令"根据你的分析，建立一个新的计划去修复
内存问题，最好能够全部解决"——授权覆盖 PG-MEM-1/2/3 三债全部清偿，
仓内 auto-lang + auto-playground，标准 worktree 流程。

**实测背景**（2026-10-09，scratch/playground-check/results_mem.json）：

- 串行 1343 示例后服务端 16→1434MB；挂起 5 例贡献 ~740MB
  （块01 +210MB / 块02 +348MB / nested_mutfn +73MB / ch05块04 +71MB /
  direct_drain +37MB）；批量结束后仍涨至 1.77GB+（执行未停）。
- 挂起源：`print(i, end=" ")` kwargs 形态（3MB/s 膨胀）、`for true {}`
  书摘死循环、已 #[ignore] 的 plan231 无限循环语料。
- 源码定位：
  - run 无超时：`crates/auto-playground/src/routes/run.rs:26`
    （spawn_blocking 裸等，panic 经 `crates/auto-lang/src/lib.rs:471`
    join unwrap 冒泡成 500）。
  - VM 执行循环注入点：`crates/auto-lang/src/vm/engine.rs:11501
    execute_task`；任务循环 `engine.rs:3222 run_task_loop`。
  - print 内建特判：`crates/auto-lang/src/vm/codegen.rs:9732`。
- 既有机制：`BIGVM_NATIVES` 惰性注册（HashMap 去重，有界）；
  `get_global_runtime()` 单例 tokio（1 worker）；VM RAM 8KB/运行——
  均非泄露主嫌，已排除。
- 规范基面：`docs/specs/auto-playground/project.md`（API 面未定义
  超时语义——本计划补）；auto-vm spec 未定义燃料/取消——新增。

**Spec 增量见 §5 规范增量表。**

## 5. 详细设计

### 燃料/取消（PG-MEM-1）

- `vm/engine.rs`：`AutoVM` 增加 `budget: Option<ExecutionBudget>`
  （`struct ExecutionBudget { instructions: u64, deadline: Instant }`）。
  公开 `set_budget`/`with_budget` 构造面。
- `execute_task` 指令循环按批次检查预算：超支返回
  `VMError::RuntimeError` 新变体或专用 `VMError::ExecutionTimeout`
  （实现期二选一，须不破坏既有 RuntimeError 字符串匹配测试）。
- 传播：`run_task_loop` 遇 Timeout 终止所有任务、清空 mailbox/timers，
  把错误冒泡给 `execute_autovm_with_path` 调用方。
- 管线入口：`execute_autovm_with_path` 增加 `budget: Option<...>`
  参数（或独立 `*_with_budget` 变体——避免触动既有调用面的二选一，
  实现期取改动最小者）；`run_with_capture_and_bytecode_with_meta` 同步
  加带预算变体，playground 专用。
- playground：`run_handler` 读取 `timeout_secs`（clamp 1..=60，默认 10），
  换算 budget 传入；捕获 VM Timeout 返回结构化错误；**保留**既有
  panic→500 兜底但 Error 响应不含 unwrap 噪音（lib.rs:471 的
  join unwrap 改返回明确错误信息——独立小修，同属本计划）。

### print kwargs（PG-MEM-2）

- T-02 先定位挂起+膨胀根因（3MB/s 输出缓冲增长的确切路径），再定修复：
  首选 codegen 编译期拒绝（内建族 kwargs → 编译错误）；
  若调查表明 kwargs 本有合法语义通道，则修该通道的缓冲 bug 并按
  SD-02 记录语义。

### 基线（PG-MEM-3）

- T-04 profiling：对全量走查做分桶内存采样（每 100 例 RSS 曲线 +
  典型增长例名单跑对照），区分"Windows 工作集不归还"与"真保留"。
- 真保留源修复（按定位结果选做，任务内记录证据）；工作集型则以
  规范注记收口（不需要强行归还——强行 EmptyWorkingSet 反伤性能）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-playground/project.md | before：/api/run 无执行时限，挂起执行永不终止；after：`timeout_secs`（默认 10s，1..=60），超时返回结构化错误，服务端无失控执行 | PG-MEM-1 根治；防 20GB 复发 | AC-01, AC-04 |
| SD-02 | add | docs/specs/auto-vm/project.md | before：print 内建 kwargs 行为未定义（实测挂起膨胀）；after：编译期拒绝 kwargs（或裁定语义写清） | PG-MEM-2 | AC-02 |
| SD-03 | add | docs/specs/auto-vm/project.md | before：VM 无执行预算概念；after：ExecutionBudget（指令+壁钟）为可选执行约束，默认无（None），超时错误语义明确 | PG-MEM-1 的 VM 面 | AC-01 |
| SD-04 | modify | docs/specs/auto-playground/project.md | before：run 错误面未约定；after：执行 panic 不再以 unwrap 噪音 500 冒泡，统一结构化错误 | 可观测性 | AC-04 |

## 6. 测试设计

- **单测（crates/auto-lang）**：
  - VM 燃料：`for true {}` + 预算 → 确定返回 Timeout（<1s），
    无预算 → 行为不变（受控短循环对照）；actor/生成器/消息循环任务
    同样被截止；Timeout 后 VM 任务表清空。
  - print kwargs：`print(1, end=" ")` 编译期报错且信息含 kwargs 名；
    合法 `print(x)` 回归不变。
  - lib.rs:471 join unwrap 改错误返回：执行线程 panic 时返回
    Err 而非 panic 传播（用非法源码语料触发解析 panic 验证）。
- **playground e2e**：`crates/auto-playground/frontend/tests/` 增补
  spec：POST /api/run 带 `for true {}` → 10s 内错误响应且后续请求
  正常（服务端无失控线程）；默认 timeout 下 demo 示例仍全绿。
- **全量走查复测**（验收档）：run_all_examples_mem.py 全量 1343 例：
  - 无客户端 90s 超时挂起（超时条款仅余合法慢例 <60s）；
  - 结束后服务端内存净增 <300MB（vs 修复前 +1417MB）；
  - 结束后 5 分钟静置内存不涨（挂起执行已清零）。

## 7. 验收标准

| id | 标准 | 验证方法 |
|---|---|---|
| AC-01 | VM 执行预算：传预算后 `for true {}`/actor 死循环/生成器死循环 ≤2s 返回 Timeout 错误；不传预算行为零变化（既有 cargo t 语料/日常档全绿） | 新增单测 + `cargo t vm` + `cargo tv` 定向 |
| AC-02 | `print(x, end=" ")` 编译期明确报错（信息含 `end`/kwargs 提示），不再挂起；`cargo t` codegen 相关档无回归 | 新增单测 |
| AC-03 | 全量走查（1343 例）结束后服务端内存净增 <300MB，且结束后静置 5 分钟不涨 | run_all_examples_mem.py 复测报告 |
| AC-04 | /api/run 挂起示例（块01/块02/nested_mutfn/ch05块04/direct_drain 五例）在 ≤60s 内返回明确超时错误；panic 型非法源码返回结构化错误而非 unwrap 噪音 500 | 五例定向 curl + 全量走查无 90s 超时挂起 |
| AC-05 | 既有门禁：cargo check -p auto-lang、裸 `cargo t`、`cargo tv` 全绿（vm_runner/管线触面回归） | 复审门禁档 |
| AC-06 | Spec 增量 SD-01..04 落入 docs/specs 对应 project.md（merge 档沉淀） | auto-plan:merge 回执 |

## 8. 执行步骤

- [x] T-01 VM 执行预算：engine.rs deadline 字段（run_task_loop 每轮清扫检查，
  超时终止全部任务置 ExecutionTimeout last_error——利用既有 100 指令帧预算机制，
  检查粒度=清扫周期）+ execute_autovm_with_deadline + 两个公开 deadline 变体。
  文件：crates/auto-lang/src/vm/engine.rs、lib.rs。commit 9c3b66cfa。
  验证：8 单测全绿（死循环/分配循环 500ms 截止、并发 4 线程各自 2.00s、
  默认面无预算行为不变、panic 干净 Err）+ `cargo tv` 162/162 零回归。AC-01
- [x] T-02 print kwargs 根因调查 + 修复：先复现定位 3MB/s 膨胀路径
  （decision artifact 写入计划 §9），再按 SD-02 落地编译期拒绝。
  文件：crates/auto-lang/src/vm/codegen.rs（内建族参数校验）。
  验证：2 单测（裸形态/循环形态编译期报错）。根因定谳：parser 将 `end=" "`
  产出为 Bina(Asn) 赋值实参，codegen dup+store 序列致每调用泄漏栈槽，
  循环内失衡腐蚀 sp 相对局部变量 → 死循环+3MB/s 输出膨胀。commit 9a0dae02f。AC-02
- [x] T-03 playground run 超时接线：run.rs timeout_secs（默认 10，clamp
  60）→ budget；超时/错误结构化响应；lib.rs:471 join unwrap 改明确错误。
  文件：crates/auto-playground/src/routes/run.rs、
  crates/auto-lang/src/lib.rs、crates/auto-playground/src/vm_runner.rs。
  验证：`for true {}` 单发/4 并发均 2.00s 返回 ExecutionTimeout（服务端内存
  稳定 22-35MB）；单发/并发/_kwargs/panic 共 4 条 playwright e2e 绿。
  commit e1cb8632f。AC-04
- [x] T-04 基线 profiling 与修复：全量走查分桶采样定位保留源；真泄露修、
  工作集型记 SD-04 注记。验证：对比报告（修复前后 RSS 曲线）。AC-03
- [x] T-05 playground e2e 增补：frontend/tests 超时 spec + demo 回归（含 F-746-R1 拦截断言）。
  验证：`npx playwright test tests/run-timeout.spec.ts` 4/4 绿（12s）。
  注：worktree 走查期发现的端口互踩/旧二进制残留为环境事故非代码缺陷
  （干净服务单发+并发均正确）。AC-04
- [x] T-06 全量走查复测：run_all_examples_mem.py 出报告（内存曲线 + 挂起清零）。
  验收档：scratch/playground-check/p746-mem-report.md（gitignored 本地证据）。
  终版数据（worktree 终版二进制，Path 已核验）：1343 例净增 +11MB（30→41MB），
  静置 5 分钟 41MB 不涨，运行异常 0（修复前 11）。前两跑无效（后台任务默认
  cwd=主检出起成旧二进制；worktree 缺 notes.json 回退 28 例）——三跑有效。
  AC-03, AC-04
- [x] T-07 复审门禁：cargo check、裸 `cargo t`、`cargo tv`；已知红族白名单
  对照。AC-05, AC-06

## 9. 复审记录

- 2026-10-09 `PLAN-746:r1 merge 收据 | prepared(d7ae203e2→rebase 95c093001,range-diff=) |
  landed(master tip=d0c94b303,ff-only 无 merge commit;主检出产物重建:playground 二进制+
  frontend dist 已刷新) | ledger_refreshed(.autoos/specs.json designs 段 P746-1/P746-2,
  docsha 冻结,README §5 手工回退路径;spec-index.py 再生 26 projects) | archived(docs/plans/archive/) |
  cleaned(wt-guard 待执行——见下方最终回执) | batch_regression: 746%5≠1;due 判定见 .last-batch-regression.json 回执 |
  completion_kind: delivered`
- 2026-10-09 `stage: review | PLAN-746 | r1 | outcome: pass | reviewed d7ae203e2 |
  base 394f90919 | dep auto-down @895f8d0 | acceptance_results: AC-01 pass（8 单测重跑）/
  AC-02 pass/AC-03 pass（后端码自走查后未变，证据复用有效）/AC-04 pass（F-746-R1 修复后
  4 慢例输出与修复前逐字节一致+e2e 5/5）/AC-05 pass（后端零变更，前端 delta 由 e2e 档覆盖；
  tv 162/162 重跑）/AC-06 pass（SD-01..04 文本核验，宿主行为补记）|
  findings: F-746-R1 resolved（timeout_secs=60 宿主显式请求+拦截断言固化）|
  evidence: d7ae203e2 diff、e2e run-timeout 5/5、4 慢例 TAP 对照、spec 预埋文本 |
  next: merge`
- 2026-10-09 `stage: review | PLAN-746 | r1 | outcome: needs_fix | reviewed b5f11e6c0 |
  base 394f90919 | dep auto-down @895f8d0 | acceptance: AC-01/02/05/06 pass（单测/tv/红集对拍/
  spec 增量核验）；AC-03 内存数据有效；AC-04 与"0 异常"判定 needs_fix |
  findings: F-746-R1 | evidence: 4 parity 慢例 10.0s ExecutionTimeout 实录（c_crawler 等，
  走查脚本 ok 逻辑只看 result 字段漏判 stdout Error）| next: work（修复后复审）`
- 2026-10-09 F-746-R1（medium-high，AC-04/慢例正确性）：默认 10s 截止截断合法慢示例——
  c_crawler/c_http_get/c_wget/http_client_sync 合法耗时 ~10.5s（sync FFI 网络超时），
  修复后全部在 10.0s 被 ExecutionTimeout 截断（前端不发 timeout_secs 时用默认 10s）。
  走查"运行异常 0"为测量假象（脚本只查 result 字段；vm_runner 把 Error 放 stdout）。
  纠正（Q-2 以走查数据定谳）：前端 usePlayground/usePlaygroundFull 的
  projectRequestBody 显式发 timeout_secs=60（上限内自选；server 默认 10s 不变——
  API 消费方仍受紧保护）；修正走查脚本 ok 逻辑（并查 stdout）；补 e2e 拦截断言
  前端确实发送 60；4 慢例全量 TAP 输出复验。修 affected: T-03 政策面/T-05。
- 2026-10-09 `stage: work | PLAN-746 | r1 | outcome: pass | code plan-746-dev @ b5f11e6c0
  （base 394f90919）| tasks T-01..T-07 全完成 | evidence：8+2 单测绿、tv 162/162、
  e2e 4/4+全量 23/23、全量走查 +11MB/静置不涨/异常 0（p746-mem-report.md）、
  红集对拍零新增 | blockers: 无 | next: review`
- 2026-10-09 `/auto-plan:new` r1 起草：`stage: new`，PLAN-746 revision 1。
  `outcome: pass`（授权内可开工：用户已批"全部解决"三债 + worktree 流程）。
  `next: work`。待办：T-02 根因调查（bounded，artifact 回写 §9）；
  504 vs 200+error 取舍实现期定（取前端改动最小面）。

## 10. 待澄清事项

- Q-1：`print` kwargs 的正确语义是"编译期拒绝"还是"支持 end=/sep="？
  倾向拒绝（Auto 无 kwargs 内建先例）；T-02 调查后以证据定谳并落 SD-02。
- Q-2：默认超时 10s 是否合适（demo 全绿前提下，慢例如 parity C 库
  编译型可放宽到 60s 上限内自选）——实现期用走查数据定，不改 AC。

## spec-sync 回写记录

- `docs/specs/auto-playground/project.md`：PLAN-746 节（执行时限/宿主 60s 行为/panic 错误面/print kwargs 拒绝锚点）。
- `docs/specs/auto-vm/project.md`：PLAN-746 节（ExecutionBudget/deadline 语义、公开 deadline 面、join_execution、SD-02 根因）。
- `docs/specs/auto-playground/plans.md` + `docs/specs/auto-vm/plans.md`（新建）：746 行一句话沉淀。
- `.autoos/specs.json` designs 段：P746-1（auto-playground）、P746-2（auto-vm），commit:95c093001 + docsha 冻结。
- `docs/specs/INDEX.md`：spec-index.py 再生（26 projects）。
- KNOWN-DEBT：PG-MEM-1/2 随修复销账；PG-MEM-3 走查归因修正（基线缓涨=挂起旋转窗口的误判，净增实为 +11MB 工作集涨落）——建议 review 档销账注记（本计划 T-04 定谳）。
