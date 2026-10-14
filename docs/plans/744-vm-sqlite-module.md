---
plan_id: PLAN-744
status: execution_done               # review R1 fix round complete (work 记录 3); next final review
feature_name: vm-sqlite-module
author: []
created_at: 2026-10-06
updated_at: 2026-10-14
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-lang/vm/overview.md, docs/specs/a2r-std/project.md]
new_spec_components: []
touched_goals: [GOAL-003]     # VM/a2r stdlib parity；待最终复审确认规范增量

affects: [auto-lang/vm]
current_step: 3
total_steps: 3
---

# [PLAN-744] vm-sqlite-module

## 变更摘要

为 AutoVM 动态链接面补齐 sqlite 模块（Plan 415-B1 遗留的 415-B2 follow-up）：
`sqlite.open/exec/query/last_insert_rowid/last_error` 在 VM 轨可链接、可执行。
依赖方：auto-os/apps/015-notes NOTES-001 r4（其 D-VM-SQLITE 阻塞项——`auto run -r vm`
启动即 `Undefined symbol: sqlite.open in module App`）。

## 目标

1. `auto run -r vm` 的 VM DynamicComponent 能链接并执行经 `stdlib/auto/sqlite.at`
   声明的全部 5 个 sqlite 调用（open/exec/query/last_insert_rowid/last_error）。
2. 行为与 a2r 轨（a2r-std sqlite.rs）一致：哨兵错误（不 panic，last_error 可查）、
   query 返回 `List<List<str>>`（嵌套堆列表）、exec 返回受影响行数、句柄 RAII 关闭。
3. 015-notes VM 模式保存/冲突场景实际可用（NOTES-001 r4 T-16 的 VM 验收门）。

## 架构方案

- 声明面：`stdlib/auto/sqlite.vm.at`（#[vm]，沿用 env.vm.at/file.vm.at 惯例）——
  句柄在 VM 轨表示为 int（image 模块 opaque ticket 先例），接口类型 SqliteDb 保留在
  sqlite.at（a2r 轨）。
- 执行面：`crates/auto-lang/src/vm/ffi/stdlib.rs` 手写 shim（task/stack 直操作，
  image 模块先例）+ 连接注册表（`Mutex<HashMap<i64, SqliteDb>>` + AtomicI64 计数器，
  image ticket registry 同构；进程级 RAII，无显式 close）。
- 嵌套列表：`query` 返回外层 `ListData<auto_val::Value>`（元素 = `Value::VmRef` 指向
  内层 `ListData<i32>` 负哨兵字符串行）——与数组字面量 materialize 路径同构
  （engine.rs materialize_value：复合元素入堆、外层持堆引用）。
- 注册：`register_shim_by_name` 同时绑 `auto.sqlite.*`（模块形）与
  `auto.sqlitedb.*`（Type.method 小写形，List.push→auto.list.push 同规则），
  覆盖 codegen 两种符号拼法。
- sqlite 本体复用 a2r-std（auto-lang 已依赖）：VM shim 只是句柄/值转换层，
  不重写 SQL 引擎。

当前实现注记（2026-10-14 R1）：上面的进程级连接注册表是初版设计记录，已由
Plan 775 的 `924273fa5` 替换为 AutoVM 实例内、opening task 所有的注册表；
`remove_task` 释放该任务连接，VM drop 释放剩余连接。公共调用范围不变。

## 需求分析与背景调查

- Plan 415-B1 交付了 a2r 轨 sqlite（sqlite.at + sqlite.rs.at + a2r-std/sqlite.rs），
  .at 接口层注释明确 "The AutoVM path has no native implementation yet; VM programs
  must not call into this module until a VM backend lands (415-B2 follow-up scope)"。
- Plan 626 曾把 VM 的 `fs.list_dir` 走 `fs::walk` JSON 串形——本计划新增 API 用
  `dir_names` 名（已落在 742 分支 + v0.6-dev facade），与 Plan 626 面无冲突。
- NOTES-001 r3 复审（2026-10-05）将 notes 后端迁移到 SQLite；r4 的 VM 验收门
  （T-16）被 `Undefined symbol: sqlite.open` 阻塞——本计划解除之。

## 详细设计

见「架构方案」。shim 语义对齐 a2r-std/sqlite.rs：
- `open(path)`：Connection::open 失败回退内存库并记 last_error，永不 panic。
- `exec(sql)`：execute；MultipleStatement/ExecuteReturnedResults → execute_batch，
  成功报 0；错误记 last_error 返回 -1。
- `query(sql)`：prepare + 逐行取 Ref（cell_to_string 同款：NULL→""，BLOB→"<N bytes>"），
  错误记 last_error 返回空列表。
- `last_insert_rowid()`：0 起始。
- `last_error()`：进程级最近错误（Mutex<String>），空串=无错。

### 规范增量

以下为 R1 核查提出的收尾增量，尚未取得最终 pass，不发布 canonical Spec/ledger：

| ID | 操作/目标 | before → after | 理由/关联验收 |
|---|---|---|---|
| SD-01 | modify `docs/specs/auto-lang/vm/overview.md` | 未列 sqlite VM 调用面 → 五函数及 module/Type.method native 绑定；query 的外层 VmRef 列表/内层字符串列表；连接归 opening task/VM 所有 | 说明已落地能力及边界；AC-01 |
| SD-02 | modify `docs/specs/a2r-std/project.md` | 已有 VM-local/task 生命周期约定 → 保留该约定，补充 VM shim 复用同一 SQL 引擎和 `List<List<str>>` 转换入口；不把进程级句柄初版重新写回 | 与 `6011bc0e4` 已沉淀内容对齐；AC-01 |

`new_spec_components: []`：扩充既有模块卡，无需新增组件。`auto service --server vm`
的 Type.method 解析失败和 fs 问题保留为 P744-D1；本计划证据不得宣称纯 VM 后端应用全绿。

## 测试设计

- 单测（stdlib.rs 内）：open→exec建表→query读回（嵌套列表逐格断言）、
  语法错误→-1+last_error 非空、last_insert_rowid 序列。
- 链接验证：NOTES-001 应用 `auto run -r vm` 启动不再报 Undefined symbol。
- 端到端：`run_autotest.py 015-notes.autotest --mode vm` 存量场景通过；
  保存冲突/恢复负例经 HTTP 双写者 + VM UI 状态断言。

## 验收标准

1. **AC-01**：`cargo test -p auto-lang sqlite`（新增单测）全绿，既有 VM 测试无回归。
2. **AC-02**：notes 应用 VM 模式可启动、可新建/编辑/保存（与 Vue 轨同数据落盘）。
3. **AC-03**：保存冲突场景：VM 端陈旧保存被拒且 UI 呈现冲突路径（与 SD-10 行为一致）。

## 执行步骤

- [x] **T-01**：sqlite.vm.at + stdlib.rs shims + 注册 + convert 嵌套列表 → cargo test。（实现已落地；R1 重开验证由 work 记录 3 闭环：`sqlite_vm_query_nested_heap_list_via_vm_stack` 经 task/栈 shim 走真实 VM 返回路径逐格断言（F744-R1-01）；绑定 b9fd5691e 的裸 `cargo t` 全日常面 + `cargo tv` 门禁收据齐备（F744-R1-02）。）
- [x] **T-02**：notes 应用 VM boot + autotest + 冲突场景采证。（r2 19/19 + 13/13；VM 前端/Rust 后端边界保留。）
- [x] **T-03**：收据 + NOTES-001 r4 计划回填（D-VM-SQLITE → PLAN-744）。（r2 收据和 notes 计划回填已存在。）

## 复审记录

- review R1（2026-10-14，用户请求“检查完成情况”）：stage=review | review_scope=final（就绪性核查，因发现未完成的必需 oracle 提前退回 work，非完整运行时复审） | reviewer_identity=当前 Codex 核查上下文，与文内 work 会话分离；未调用其他 agent；不声称外部独立审核 | plan_id=PLAN-744 | plan_revision=1（将 work 记录 2 已使用的 revision 补入 frontmatter；AC/T 稳定 ID 仅规范化原三项，未改验收含义，未追认历史审批） | outcome=needs_fix | worktree_path=D:/autostack/auto-lang（历史已合入核查；744 无专属在册 worktree） | branch=master | reviewed_commit=c171c1a09740fbd61e4bf6235286a3424e2e4a81 | base_commit=6baed9bba84016dd8610221a9a0358195444385e（初始实现 cb7a59ede 的父提交，仅历史溯源） | dependency_revisions=sqlite 初始 cb7a59edec448801e7730b3d8f2c1d3be0d3e573、任务寿命修复 924273fa5accf883a6298ee8a5133d513fb17065（均为 master 祖先）；auto-notes main@1391bd7eb2f6b3b08e9841d30b25125e47ced058 | acceptance_results=AC-01 partial（2 项辅助函数/任务寿命单测已有记录，但测试设计承诺的 VM 嵌套堆列表逐格验证未实现，既有 VM 无回归也无 744 绑定门禁收据）；AC-02/03 pass on retained r2 evidence（本轮审计原始场景日志、JSON 和负例脚本，不声称重新运行） | findings=F744-R1-01/02/03 | pending_closeout_gates=修复 T-01 后完整最终复审、SD-01/02 冻结复核、merge/spec/ledger/归档；已有 Spec 的 task-local 生命周期描述不能被初版设计覆盖 | evidence=[R1 核查报告](reports/744-completion-review-r1.md) | next=/auto-plan:work 在同一 Plan 的恢复 worktree 补齐 T-01 验证和明确门禁证据，提交后再 /auto-plan:review；本次不修改实现、不合入、不归档。

- work 记录 1：stage=work | plan_id=PLAN-744 | outcome=executing（交付完成，1 项环境残留） | code_commit=cb7a59ede（v0.6-dev）/ 5bbb51caa（plan-742-dev worktree，另含 fs.dir_names VM shim 与种子） | evidence=stdlib 单测 sqlite_vm_open_exec_query_roundtrip（含哨兵错误/未知句柄）、NOTES-001 应用 `auto run -r vm --server rust` 启动成功（MCP :9247 listening + 首帧 state sync + App Init 经 sqlite 路径完成——seed 6 条经 VM sqlite shims 落 inbox.db，autotest T0 初始状态断言通过=VM 端到端执行了 SQLite 存储）、evidence=notes 仓 tests/probe/evidence/vm-r4-boot.log + vm-r4/data/inbox.db（6 行+schema_v=2） | 残留=MCP 服务在无头会话中服务完首批请求后监听消失（PLAN-066 已登记的 9247 共址/存活脆弱类；本轮 19 场景套件仅 T0 跑通即断连），VM 全套冲突编排未能在会话内采——store 逻辑同一 .at 已在 Vue 轨 negatives-r4 3/3 实证 | next=review（VM 冲突编排的完整采证依赖 MCP 服务存活问题的修复，属 ui/mcp_server.rs 基建项，建议随本计划复审一并裁决是否入 744 范围或另立）
- work 记录 2（2026-10-14，残留闭环）：stage=work | plan_id=PLAN-744 | plan_revision=1（范围未变） | outcome=pass（记录 1 残留全闭环，无代码改动——纯验证+簿记，故未开 worktree） | worktree_path=无（验证会话：auto-lang 主检出仅跑测试/构建，notes 仓 main@1391bd7 只读运行，写入均落 gitignored .auto/） | code_commit=无新增（sqlite 代码 = 924273fa5 状态；binary SHA-256 6e30c87f5c…f3524c5 全新构建） | evidence=[r2-evidence](attachments/744/r2-evidence/README.md)：①AC#1 重绑定 `cargo test -p auto-lang --lib sqlite` 2 passed（roundtrip + 924273fa5 的 handles_are_vm_local_and_released_with_request_task；1 预存 ignore=cookbook needs-sqlite）②AC#2 重绑定 VM autotest 19/19 EXIT=0 净库单轮（MCP 78s 就绪——rust 后端冷构建可超 r5 脚本 90s 窗）③AC#3 重绑定 VM 负例 13/13 EXIT=0 fails=[]（notes r5 交付的 run_vm_negative.py 原样重跑：外部写者冲突 dirty/conflict 保留+双恢复槽、陈旧保存被拒+Discard 取最新服务器版、after_manifest fault 裁决+幂等重试）④残留闭环依据=NOTES-001 r4 work 记录 9 把 MCP 断连根因定为套件自身契约缺陷（跨轮数据污染+筛选残留+索引期望陈旧+240s 时长，服务端 157/157 全服务；非 mcp_server.rs 缺陷）+AUTOUI_MCP_PORT/Connection:close/净库单轮三件套已落 notes 测面——原「入 744 或另立」的裁决不再需要 | 勘误（记录 1）=`--server rust` 启动下种子落盘走 a2r_std 直连，不经 VM shims——「seed 6 条经 VM sqlite shims」表述过当；VM shims 真实执行的证据=单测 + 本次 `--server vm` 探测（后端 AutoVM HTTP 起服 26 路由，inbox.db 三表+meta 经 shims 落盘=方法形 db.exec/db.query 在 run 模式可链接可执行） | 新发现（744 范围外，登记 KNOWN-DEBT P744-D1）=纯 VM 后端轨三连：fs.copy_recursive 两轨语义分裂（fs.at:58 声明 void+#[vm] 抛 RuntimeError vs a2r 返 bool→notes legacy 门 500）；`auto service --server vm` 链接失败 Undefined symbol: SqliteDb.exec in module p2probe（service 每-模块解析缺 Type.method 形映射，run 模式扁平化有）；后端 VM handler 上下文 fs.exists 疑似误报（未隔离，独立脚本探针正确） | blockers=无 | next=review（/auto-plan:review 对本记录+附件复审；P744-D1 由复审裁决路由）
- work 记录 3（2026-10-14，R1 needs_fix 修复）：stage=work | plan_id=PLAN-744 | plan_revision=1 | outcome=pass（F744-R1-01/02 闭环；F744-R1-03 规范收尾归 review/merge，frontmatter 影响元数据维持 R1 版） | worktree_path=D:/autostack/.wt/lang-744/auto-lang（恢复 worktree；组内兄弟 auto-down detached@c39ac7f 补齐 workspace 解析） | branch=plan-744-dev | base_commit=c171c1a09（=reviewed_commit） | code_commit=b9fd5691e（test-only：`sqlite_vm_query_nested_heap_list_via_vm_stack`，stdlib.rs +171 行，无实现改动） | dependency_revisions=auto-down@c39ac7f（组内兄弟，master HEAD detached；运行时依赖路径解析用，未改其内容） | task_ids=T-01 | evidence=①F744-R1-01 闭环：新测试五调用全走 task/栈 shim（open/exec/query/last_insert_rowid/last_error，参数经 VMConvertible 入栈=引擎分派同路径，非 `*_impl` 助手）；query 结果按 .at 面接收的堆布局消费——外层 `ListData<Value>` 的 VmRef → 内层 `ListData<i32>` 负哨兵 → 字符串池 utf-8 逐格断言；覆盖 3 行 ×4 列（中文/引号/NULL→""/BLOB→"<N bytes>"）、空结果（已分配外层列表 0 行）、错误哨兵（坏 SQL→空列表+last_error 经其 shim 非空返回）；既有 roundtrip/任务寿命两测保留未动 ②F744-R1-02 闭环（绑定 b9fd5691e，worktree 实跑）：`cargo test -p auto-lang --lib sqlite` 3 passed/0 failed（1 预存 ignore）；裸 `cargo t --no-fail-fast` 全日常面 5279 run / 5278 passed / 1 timeout——失败归因：plan738 p4_rust_emission_rejects_missing_net_provider 并行负载下超 120s 慢超时（隔离重跑 81.5s PASS），首跑另见 plan484_024_charts_streaming_recompute flake（--no-fail-fast 轮已过+隔离 PASS），两处均与本 diff（sqlite 测试新增）无因果，0 新红；`cargo tv` 语料档 162/162 | gates=Category B 分级门禁（VM 面触改：裸 t + tv；tf 归批量回归档非 per-plan 门禁） | 健康检查=stdlib.rs fmt 干净（repo 28 处 fmt diff 均为 examples 预存噪声，非本 diff 文件）；新代码零警告 | blockers=无 | next=review（最终复审：对 b9fd5691e 验 F744-R1-01/02 闭合 + F744-R1-03 的 SD-01/02 冻结裁决；P744-D1 路由）

