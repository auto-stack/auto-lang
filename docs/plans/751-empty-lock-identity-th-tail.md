---
plan_id: PLAN-751
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: empty-lock-identity-th-tail
author: [zcode]
created_at: 2026-10-10
updated_at: 2026-10-10

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [auto-man/design/api-generation-integrity.md]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man/design/api-generation-integrity.md]
current_step: 0
total_steps: 6
---

# [PLAN-751] empty-lock-identity-th-tail

## 变更摘要

PLAN-738 归档后 R11 复审（`docs/plans/reports/738-review-r11.md`，commit `e0999cee1`）
认定的遗留缺陷收口：

1. **P738-R11-01（P2）**：空 Cargo.lock 的生产者/消费者身份不一致——生成端
   （`api_gen.rs` 收据写入）对成功读取的零字节文件记 FNV 指纹 `811c9dc5`，
   复用门（`rust_ui.rs::backend_generation_is_fresh`）却用 `bytes.is_empty()`
   把成功读取的空文件与 NotFound 哨兵合并归 `absent`。真实生成/消费反例：
   生成时无 lock（收据 absent）→ 空文件出现 → 门 fresh 但绑定缺失 → 同一空
   文件上再生成成功（收据升级为 `811c9dc5`）→ 业务与 lock 均未变，门仍判
   stale（永久陈旧）。
2. **HTTP 验收尾项**：R9 串行 `cargo th` 仅完成 45/102（43 pass / 2 在册红），
   57 项未跑（含上轮超时停跑的 `legacy_bind_failure_cleans`），无完成收据。

738 保持 archived / delivered，不重开原计划；本计划为独立后续修复合同。

## 目标

- 空文件（及其它一切成功读取形态）在生成端与消费端获得**同一实际内容身份**；
  仅 **NotFound** 表示 absent；其它读错误维持 R9-01 fail-closed（不可核验）。
- 一次绑定状态机（R5-01）语义保留并覆盖空文件臂：absent → 空文件出现 →
  门 fresh 且收据绑定 `811c9dc5`；此后漂移/删除/读错误均陈旧。
- 分类/身份逻辑收敛为**单一实现**，生成端与复用门共用，消除平行口径。
- 完成移交的完整串行 HTTP 验收（102/102）并逐名分诊、落盘收据。

**非目标**：不修改 738 归档状态与历史报告；不改非空正常 lock 路径行为；
不处理与 lock 身份无关的 HTTP 代码缺陷（th 分诊只登记归因，不顺手修实现）；
不动 aavm/transpiler/book 面。

## 架构方案

`crates/auto-man` 内新增分类单点 `workspace_lock_identity`（放 `rust_ui.rs`，
`pub(crate)`，与既有 `classify_lock_read` 同层）：

```text
io::Result<Vec<u8>> ──▶ Result<Option<String>, io::Error>
  Ok(bytes)              → Ok(Some(fnv1a(bytes) 的十六进制))   // 含零字节 → "811c9dc5"
  Err(NotFound)          → Ok(None)                            // 调用方归 "absent"
  Err(其它)              → Err(e)                              // 不可核验
```

- **消费端**（`backend_generation_is_fresh`）：`Ok` 值 `None→"absent"`、
  `Some(fingerprint)` 原样；`Err` → `None`（不可核验=保守陈旧，R9-01 保持）。
  一次绑定臂 `(Some("absent"), Some(materialized))` 无需改动——空文件现携带
  实际身份 `811c9dc5`，自然进入绑定；`materialized != "absent"` 守卫语义不变
  （"absent" 非十六进制串，与指纹无碰撞可能）。
- **生成端**（`api_gen.rs` 收据写入）：`Ok(Some))` → 指纹；`Ok(None)` →
  `"absent"`；`Err(e)` → 沿用 `workspace Cargo.lock unreadable` 拒写收据。
- 删除 `classify_lock_read`（空 Vec 同时表示读取成功与缺失的哨兵设计正是
  本次缺陷根源），其单测改写为对新函数的三态断言。

## 技术栈

Rust（`crates/auto-man`，workspace 内）；serde_json（收据值级更新）；测试用
tempfile 隔离 workspace + `AUTO_RUST_WORKSPACE` env 指向（沿既有 R5 状态机
测试形态）；th 验收用 nextest `-j 1` 串行（http_e2e 族固定端口，见
`.cargo/config.toml` th 注释）。

## 需求分析与背景调查

（从 docs/specs/overview.md 与相关 module spec 取材）

- **授权**：用户 2026-10-10 会话指令——修复 P738-R11-01（P2）并完成 HTTP
  验收尾项；「738 保持归档，剩余缺陷应走新的后续修复计划」。仓库范围：
  auto-lang 主仓；无预算/自动续跑限制声明。
- **规范依据**：`docs/specs/auto-man/design/api-generation-integrity.md`
  §workspace lock 一次绑定（PLAN-738 R5-01）——「ready 记录 workspace_lock
  （FNV 身份；未建 lock 记 absent）」。R11 裁定 SD-05：规范规则保留、修
  实现、不降低规范。规范未定义空文件归属是实现漂移的空隙，本计划补一句
  三态分类口径（见规范增量 SD-01，属收紧澄清非行为变更）。
- **缺陷锚点**（master `e0999cee1` 实测核对）：
  - 消费端：`crates/auto-man/src/rust_ui.rs:4026-4035`（`bytes.is_empty()`
    → `"absent"`）+ `classify_lock_read`（4054-4060）。
  - 生成端：`crates/auto-man/src/api_gen.rs:1073-1081`（`Ok(bytes) → fnv1a`，
    空文件得 `811c9dc5`）。
  - R11 探针反例与日志：`docs/plans/reports/738-review-r11-probe.py` /
    `738-review-r11-evidence.txt`（0 passed / 1 failed，断言两端身份应相同：
    `absent` vs `811c9dc5`）。
- **既有测试**（`rust_ui.rs` tests 模块，~5057 起）：`lock_freshness_truth_table`、
  `review738_r5_lock_binding_state_machine`（真实 generate_api 端到端）、
  `classify_lock_read_fail_closed`（R9-01 矩阵）、`assembly_freshness_truth_table`。
  均须保持绿（`classify_lock_read_fail_closed` 随实现改写）。
- **验证面事实**：`cargo t` 日常档只含 `-p auto-lang`，**auto-man 测试不在
  日常面**——本计划 scoped 门为 `cargo check -p auto-man` + auto-man lib 测试
  全套（R9 口径：`auto-man api_gen` 44 passed / 1 ignored）。
- **th 现状**：`cargo th` = `nextest run -p auto-lang --lib --features
  test-http-e2e http_e2e`（固定端口族，必须串行 `-j 1`）；R9 完成 45/102：
  43 pass + 2 在册红（`back_proxy` corpora 数据面 400/200 基线；
  `plan730_client730_server_interop` 固定端口 18980 传输失败，环境/历史同形）；
  停跑点 `legacy_bind_failure_cleans` 超 60s。批量回归收据
  `docs/plans/.last-batch-regression.json` = 2026-10-08 / `3c2f3c347` / tf+tt+tb，
  不含 th 腿。
- **并发环境**：主检出存在他方在途 WIP（autodown_*.rs 等）与多个并存
  worktree（lang-750 等）——th 全档在本计划 worktree 内干净提交基面上跑，
  避免主检出 WIP 污染；串行单实例由本计划自觉保证。

## 详细设计

### 实现变更

1. `rust_ui.rs` 新增：

   ```rust
   pub(crate) fn workspace_lock_identity(
       result: std::io::Result<Vec<u8>>,
   ) -> Result<Option<String>, std::io::Error>
   ```

   三态如架构方案；`Ok(bytes)` 一律 `format!("{:x}", crate::api_gen::fnv1a(&bytes))`
   （零字节 → `"811c9dc5"`，与生成端历史值逐字一致——收据无需迁移）。

2. 消费端 `backend_generation_is_fresh`：删除 `classify_lock_read` 调用与
   `bytes.is_empty()` 合并逻辑，改 `workspace_lock_identity(...).map(|id|
   id.unwrap_or_else(|| "absent".into())).ok()`；注释同步（absent 仅指
   NotFound；空文件=实际身份）。

3. 生成端 `api_gen.rs` 收据段：`match crate::rust_ui::workspace_lock_identity(
   std::fs::read(...))`，三分支如架构方案；错误文案不变
   （`workspace Cargo.lock unreadable: {e}`）。

4. 一次绑定臂与 `lock_freshness` 真值表零改动（纯字符串比较，身份口径
   修正后自动正确）；`bind_workspace_lock` 零改动。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-man/design/api-generation-integrity.md | before：「未建 lock 记 absent」未定义空文件归属，生成/消费两端口径漂移（R11 反例）。after：明确三态分类单点——NotFound=absent；成功读取（含零字节文件）=实际内容身份（FNV）；其它读错误不可核验（生成端拒写收据/消费端保守陈旧）；两端共用同一分类实现 | R11 修复要求「仅 NotFound 视为 absent…宜共用显式分类/身份逻辑」；收紧澄清，非行为放宽 | AC-01..04 |

## 测试设计

- **改写** `classify_lock_read_fail_closed` → `workspace_lock_identity` 三态
  断言：`Ok(vec![1,2])`→`Ok(Some(指纹))`；**`Ok(vec![])`→`Ok(Some("811c9dc5"))`**
  （本缺陷直接断言）；NotFound→`Ok(None)`；PermissionDenied→`Err`；端到端
  语义 `!lock_freshness(Some("absent"), None)` 保持。
- **新增** `review751_empty_lock_identity_matrix`（真实 `generate_api` +
  `backend_generation_is_fresh`，隔离 workspace，沿 R5 状态机测试形态）：
  1. 无 lock 生成 → 收据 `absent`；门 fresh。
  2. 写入**空** Cargo.lock → 门 fresh（一次绑定）；收据绑定 `811c9dc5`；
     再查门 fresh。
  3. **同空文件再生成**（R11 反例转正）→ 收据仍 `811c9dc5`；门 fresh
     （未变输入不得判 stale）。
  4. 非空漂移 → 门 stale。
  5. 删除 → 门 stale。
  6. Cargo.lock 变目录（非 NotFound 读错误）→ 门 stale（R9-01 消费端矩阵
     端到端化）；清理。
- **保持绿**：`lock_freshness_truth_table`、`review738_r5_lock_binding_state_machine`
  （非空路径回归锚）、`assembly_freshness_truth_table`、api_gen 侧
  `generation_consumer_identity_matches_cli_actual` 等 738 遗产测试。
- **门禁**：`cargo check -p auto-man`（零新警告）；auto-man lib 测试全套
  （nextest，进程隔离避免 `AUTO_RUST_WORKSPACE` env 串扰）；裸 `cargo t`
  （标准 per-plan 门，auto-lang 面）；rustfmt check 两改动文件；th 全档
  `-j 1 --no-fail-fast`。

## 验收标准

- **AC-01 空 lock 统一内容身份**：真实 `generate_api` 在空 Cargo.lock 存在时
  收据 `workspace_lock` = `811c9dc5`，与复用门对同一空文件的身份一致；空
  文件上再生成成功后未变输入判 fresh。验证：`review751_empty_lock_identity_matrix`
  第 2/3 步断言绿。
- **AC-02 一次绑定覆盖空文件**：absent → 空文件出现：门 fresh 且收据绑定
  `811c9dc5`；此后非空漂移/删除/读错误均 stale。验证：同测试第 2/4/5/6 步
  + R5 状态机测试保持绿。
- **AC-03 fail-closed 读错误矩阵不回归**：非 NotFound 读错误——生成端拒写
  收据（Err 上抛），消费端判陈旧。验证：`workspace_lock_identity` 单测
  PermissionDenied→Err + 端到端目录负例 stale。
- **AC-04 分类单点共用**：生成端与消费端均调 `workspace_lock_identity`，
  `classify_lock_read` 删除，无平行身份口径。验证：代码审查 + grep 两调用点。
- **AC-05 完整串行 th 收口**：`cargo th --jobs 1 --no-fail-fast` 于 lang-751
  worktree 干净基面 102/102 跑完，逐名分诊（含 R9 承接的 2 在册红与停跑点
  `legacy_bind_failure_cleans`），收据落 `docs/plans/reports/751-th-full-receipt.md`。
  验证：selected=run=102 计数 + 收据文件在树。
- **AC-06 健康门**：改动文件 rustfmt check exit 0；`cargo check -p auto-man`
  零新警告；auto-man lib 全套与裸 `cargo t` 无新增红（预存名册外）。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-01 分类单点（rust_ui.rs）**
  依赖：无。文件：`crates/auto-man/src/rust_ui.rs`（~4020-4060）。
  操作：新增 `pub(crate) fn workspace_lock_identity`；消费端改用之（NotFound
  →absent、成功读取→内容指纹、Err→None）；删除 `classify_lock_read`；注释
  更新（absent 仅指 NotFound）。→ AC-01/03/04
  验证：`cargo check -p auto-man`。
- **T-02 生成端对齐（api_gen.rs）**
  依赖：T-01。文件：`crates/auto-man/src/api_gen.rs`（~1068-1081）。
  操作：收据 `workspace_lock` 写入改调 `crate::rust_ui::workspace_lock_identity`，
  三分支映射（指纹/absent/unreadable 拒写）。→ AC-01/04
  验证：`cargo check -p auto-man`。
- **T-03 测试改写与新增**
  依赖：T-01/T-02。文件：`crates/auto-man/src/rust_ui.rs` tests 模块。
  操作：改写 `classify_lock_read_fail_closed`；新增
  `review751_empty_lock_identity_matrix`（六步矩阵，见测试设计）。
  → AC-01/02/03
  验证：`cargo nextest run -p auto-man --lib rust_ui::tests` 绿 + 既有
  738 遗产测试绿。
- **T-04 工作树验证门**
  依赖：T-03。操作：`cargo check -p auto-man`（零新警告）→ auto-man lib
  全套 → 裸 `cargo t` → `rustfmt --check`（rust_ui.rs / api_gen.rs）。
  → AC-06
- **T-05 th 全档串行收口**
  依赖：与本计划代码改动无耦合（auto-lang 面），T-04 后同 worktree 执行。
  操作：`cargo th --jobs 1 --no-fail-fast`（102 项，串行单实例）；逐名分诊
  新红（对照 R9 名册与批量回执在册红）；写
  `docs/plans/reports/751-th-full-receipt.md`。→ AC-05
- **T-06 复审、合并、归档、清理**
  依赖：T-04/T-05。操作：`/auto-plan:review`（独立复审，证据绑修订）→
  merge 回 master（Conventional Commit）→ spec 沉淀（SD-01 + specs.json/
  索引）→ 归档本计划 → `wt-guard` → 移除 worktree/分支/组目录；merge 后
  按到期判定批量回归（如到期交 `/auto-plan:regress` 主检出单实例执行）。

## 复审记录

- 2026-10-10 stage: new（/auto-plan:new）。plan_revision: 1。outcome: pass
  （合同要素齐备，任务覆盖全部 AC 与 SD-01；授权=用户 2026-10-10 会话指令，
  见 §需求分析）。next: `/auto-plan:work` 于 `D:/autostack/.wt/lang-751/auto-lang`
  执行 T-01..T-06。

## 待澄清事项

（无——R11 修复要求已给出明确口径：仅 NotFound 视为 absent、成功读取统一
内容身份、共用分类实现、补空文件一次绑定与再生成测试、保留读错误矩阵。）
