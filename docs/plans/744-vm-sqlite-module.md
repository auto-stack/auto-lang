---
plan_id: PLAN-744
status: execution_done               # drafting → executing → execution_done → reviewed → archived
feature_name: vm-sqlite-module
author: []
created_at: 2026-10-06
updated_at: 2026-10-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

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

## 测试设计

- 单测（stdlib.rs 内）：open→exec建表→query读回（嵌套列表逐格断言）、
  语法错误→-1+last_error 非空、last_insert_rowid 序列。
- 链接验证：NOTES-001 应用 `auto run -r vm` 启动不再报 Undefined symbol。
- 端到端：`run_autotest.py 015-notes.autotest --mode vm` 存量场景通过；
  保存冲突/恢复负例经 HTTP 双写者 + VM UI 状态断言。

## 验收标准

1. `cargo test -p auto-lang sqlite`（新增单测）全绿，既有 VM 测试无回归。
2. notes 应用 VM 模式可启动、可新建/编辑/保存（与 Vue 轨同数据落盘）。
3. 保存冲突场景：VM 端陈旧保存被拒且 UI 呈现冲突路径（与 SD-10 行为一致）。

## 执行步骤

1. sqlite.vm.at + stdlib.rs shims + 注册 + convert 嵌套列表 → cargo test。
2. notes 应用 VM boot + autotest + 冲突场景采证。
3. 收据 + NOTES-001 r4 计划回填（D-VM-SQLITE → PLAN-744）。

## 复审记录

- work 记录 1：stage=work | plan_id=PLAN-744 | outcome=executing（交付完成，1 项环境残留） | code_commit=cb7a59ede（v0.6-dev）/ 5bbb51caa（plan-742-dev worktree，另含 fs.dir_names VM shim 与种子） | evidence=stdlib 单测 sqlite_vm_open_exec_query_roundtrip（含哨兵错误/未知句柄）、NOTES-001 应用 `auto run -r vm --server rust` 启动成功（MCP :9247 listening + 首帧 state sync + App Init 经 sqlite 路径完成——seed 6 条经 VM sqlite shims 落 inbox.db，autotest T0 初始状态断言通过=VM 端到端执行了 SQLite 存储）、evidence=notes 仓 tests/probe/evidence/vm-r4-boot.log + vm-r4/data/inbox.db（6 行+schema_v=2） | 残留=MCP 服务在无头会话中服务完首批请求后监听消失（PLAN-066 已登记的 9247 共址/存活脆弱类；本轮 19 场景套件仅 T0 跑通即断连），VM 全套冲突编排未能在会话内采——store 逻辑同一 .at 已在 Vue 轨 negatives-r4 3/3 实证 | next=review（VM 冲突编排的完整采证依赖 MCP 服务存活问题的修复，属 ui/mcp_server.rs 基建项，建议随本计划复审一并裁决是否入 744 范围或另立）
