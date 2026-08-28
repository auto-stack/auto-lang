---
plan_id: PLAN-470
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: use-rs-alias
author: []
created_at: 2026-08-28
updated_at: 2026-08-28

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: []                   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 16
---

# [PLAN-470] use-rs-alias

## 变更摘要

新增 `use.rs` 作为 Rust FFI 导入的官方语法（与 `use.py` 对齐），并对旧拼写 `use.rust` 发出 deprecation warning（W0005）。分三批次执行：① 语言层别名 + warning；② 本仓 .at 资产与文档全量迁移；③ auto 系列外部仓库迁移。旧拼写在本 plan 内只迁移不删除，最终移除留给后续独立 plan。

## 目标

1. `use.rs <crate>::<path>::{items}` 与 `use.rust ...` 完全等价（同一 `UseKind::Rust` AST，转译/VM 输出零差异）。
2. `use.rust` 触发 `Warning::DeprecatedFeature`（W0005），提示改用 `use.rs`；warning 不致命，不破坏既有构建。
3. 本仓 136 个 `.at` 命中文件全量迁移到 `use.rs`，测试套件收敛为"零 deprecation warning"状态。
4. 本仓现行文档（website、docs/tour、docs/design、docs/specs 现行页）以 `use.rs` 为推荐写法。
5. 外部 auto 系列仓库（auto-musk / auto-ai / book / auto-down / auto-shell / auto-code-rs / auto-forge）完成迁移，合计约 78 个 `.at` + 45 个 md 命中。

## 架构方案

背景（已勘察确认）：

- **语法入口**：`crates/auto-lang/src/parser.rs:6455-6470` — `use.` 后按关键字分发：`c` → `use_c_stmt`、`rust` → `use_rust_stmt`、`py` → `use_py_stmt`、`web` → `use_web_stmt`；其余 fall-through 为模块路径导入（Plan 131）。
- **AST 层**：`crates/auto-lang/src/ast/use_.rs:7` — `UseKind::{Auto, C, Rust, Py}` 语义化表示，与拼写无关。`trans/{rust,c,python,gdscript}.rs`、`vm/codegen.rs`、`compile.rs`、`infer/mod.rs` 全部匹配 `UseKind`，**不感知语法拼写** → 别名不产生任何下游改动，`.at` 资产替换后 expected 输出零变化。
- **warning 机制现成**：`crates/auto-lang/src/error.rs` `Warning::DeprecatedFeature`（W0005，`{name, message, span}`），先例 parser.rs:9086（`take` → `move` deprecation，`self.warn()`，非致命，测试不因 warning 失败——`take` 至今仍存活于个别测试资产即为证据）。
- **文本扫描层**：`crates/auto-lang/src/use_scanner.rs:208` — 行级扫描器按 `.rust `/`.py ` 前缀识别，消费者包括 `compile.rs`、`lib.rs`、`autovm_persistent.rs`、`ui/handler_codegen.rs`、`ui/vm_bridge.rs`、`ui_gen/api.rs`、`auto-lsp/src/workspace.rs`。
- **词法无冲突**：`rs` 不是 lexer 关键字；仓内外均无 `use.rs.foo`（导入名为 `rs` 的模块）用法。

设计决策：

- `rs` 成为 `use.` 保留段（与 `c`/`rust`/`py`/`web` 同列），影响可忽略。
- deprecation warning 在 **parser** 层发（`use_rust_stmt` 经由 `rust` 拼写进入时），一次性覆盖编译、VM、LSP、转译全部路径；`use_scanner` 为内部工具不发 warning。
- 本仓 `docs/plans/**`（11 个非归档 plan 文档）为历史执行记录，**不迁移**，仅在 KNOWN-DEBT 记录 deprecation 周期。
- `auto/lib/parser.at` 带 "AAVM v2 Sync Snapshot" 头（Plan 432，从 parser.rs 同步生成）：迁移前先确认其生成链（autodown/sync 工具），从源头迁移再重新生成，避免手改生成物被覆盖。

## 需求分析与背景调查

用户需求：`use.rust` 书写冗长且与 `use.py` 不统一；`rs` 是 Rust 生态通认简称。要求：① 所有用到 `use.rust` 的 Auto 代码迁移（含标准库与示例）；② `use.rust` 加 deprecation warning；③ 全部相关文档（设计文档、website、示例）更新；④ 覆盖外部 auto 系列仓库。

### 影响面盘点（2026-08-28 实测）

**本仓 auto-lang（正式树，排除 .worktree*/tmpb）**：

| 类别 | 数量 | 明细 |
|---|---|---|
| 测试 .at | 125 | a2r 24、cookbook 88、vm 11、ffi_dual 2 |
| 非测试 .at | 11 | `stdlib/auto/storage.rs.at`、`parity/libs/rust/rusqlite/auto/rusqlite.at`、`examples/playground-demo/`×3、`docs/tour/ch12-interop/`×4、`auto/lib/parser.at`（同步快照）、`docs/plans/reports/429-b2-perf/bench.at` |
| 现行文档 md | 15 | website 2（`rust.md`、`zh/rust.md`）、docs/design/strategy 2、docs/specs 11 |
| 历史记录（不动） | 12+ | docs/plans 非归档 11、docs/plans/reports bench.at、归档 plans |
| Rust 源内嵌串/注释 | ~10 文件 | plan442_parser_regress_tests、plan442_musk_backend_probe_tests、a2r_tests、ffi_dual_tests（内嵌测试串，兼作别名的一手覆盖）；compile.rs/parser.rs/lib.rs/types.rs/use_scanner.rs（多为注释 + 错误消息 1 处） |

**外部仓库（正式树，排除 .worktree*/node_modules/target/tmp）**：

| 仓库 | .at | md | 备注 |
|---|---|---|---|
| auto-musk | 30 | 10 | 集中在 `backend/` |
| auto-ai | 22 | 8 | 集中在 `crates/` |
| book | 22 | 21 | `rust/`17 + `tapl/`5，书稿资产，面向读者 |
| auto-down | 4 | 2 | 全在 `tmp/dsl-probes/`（丢弃性探针，可不动） |
| auto-shell | 0 | 3 | 主树 .at 干净；**158 个 .at 命中全在其 `.worktrees/`**（陈旧分支副本，随活跃分支合并时回流，迁移以活跃分支为准） |
| auto-code-rs | 0 | 2 | 仅文档 |
| auto-forge | 0 | 1 | 仅文档 |

外部合计 ≈ 78 .at + 45 md。**外部仓迁移前置条件：本仓批次一、二合入并发版**，各仓升级 auto-lang 工具链后按 deprecation warning 指引迁移。

## 详细设计

### D1. parser.rs 语法分发 + deprecation warning（核心改动）

分发处（parser.rs:6462）区分拼写，`rs` 走新入口、`rust` 走带警告入口：

```rust
} else if name == "rust" || name == "rs" {
    let deprecated = name == "rust";
    return self.use_rust_stmt_inner(deprecated);
}
```

`use_rust_stmt` 改为 `use_rust_stmt_inner(deprecated: bool)`（保留 `use_rust_stmt()` 兼容包装供既有测试调用，默认 `deprecated=false` 或按语义定），进入时若 `deprecated` 则：

```rust
self.warn(Warning::DeprecatedFeature {
    name: "use.rust".into(),
    message: "use 'use.rs' instead".into(),
    span,
});
```

与 `take` → `move` 先例（parser.rs:9086）同构。AST 产出两拼写完全一致（`UseKind::Rust`）。

### D2. use_scanner.rs 行级扫描同步

`parse_use_line()` 新增 `.rs ` / `.rs\t` 前缀分支 → `parse_rust_import`（`.rust` 第 4 字符为 `u`，与 `.rs` 前缀互斥，顺序不敏感）。保留 `.rust ` 识别（外部旧代码的依赖扫描不因迁移进度中断）。

### D3. 错误消息与文档引导

- `compile.rs:479`：错误消息改为 "Add `dep {}` before `use.rs`."
- website `rust.md`/`zh/rust.md`、`docs/tour/ch12-interop/`（教程正文 + .at 示例）、`docs/design/strategy/`×2、`docs/specs/` 现行 design/overview 页：示例改 `use.rs`，注明 `use.rust` 已 deprecated。

### D4. 迁移策略

- 本仓测试 .at（125 个）与非测试 .at（10 个）：机械化替换 `use.rust` → `use.rs`（等价语法，expected 输出零变化）。`docs/plans/reports/bench.at` 属历史报告，跳过。
- Rust 内嵌测试串（plan442_*、a2r_tests、ffi_dual_tests 等）：迁移到 `use.rs`，兼作别名的一手测试覆盖。
- 注释中的 `use.rust`：涉及"现行语法说明"的顺手更新，纯历史注释（如 "Plan 092: ..."）保留。
- `auto/lib/parser.at`：先查同步链（Plan 432 parse_dump / autodown），从源头迁移再同步；若同步工具不可独立运行，则手改并在 KNOWN-DEBT 记录重新同步义务。

### D5. 未来移除路径（不在本 plan）

后续独立 plan：删 parser `.rust` 分发与 scanner `.rust` 分支，改为报错引导。触发条件：外部仓迁移完成 + 一个发布周期无 `use.rust` 存量。

## 测试设计

1. **parser 单测**：`use.rs std::collections::{HashMap, HashSet}` → `UseKind::Rust`、paths/items 正确；`use.rust ...` 解析成功且 `warnings` 含 `DeprecatedFeature`；`use.rs ...` 解析 `warnings` 为空。
2. **scanner 单测**：`use.rs serde::json`、`use.rs serde::json::{from_str}` 识别为 Rust 导入；`.rust` 旧前缀仍识别。
3. **端到端**：新增 `crates/auto-lang/test/a2r/14_modules/002_use_rs/`（.at + expected.rs，与 001_rust_use 对照，转译输出一致）。
4. **回归**：全量迁移后 `cargo ta`（a2r/cookbook/vm/ffi_dual 全测试族）零失败、输出零变化。

## 验收标准

- [ ] `use.rs` 与 `use.rust` 解析出 AST 相等；VM 与 Rust 转译目标行为一致（002_use_rs 用例通过）。
- [ ] `use.rust` 触发 W0005 DeprecatedFeature（"use 'use.rs' instead"），`use.rs` 不触发。
- [ ] 本仓正式树 `.at` 资产 `use.rust` 命中归零（docs/plans/reports 与历史记录除外）。
- [ ] website/tour/design/specs 现行文档推荐写法为 `use.rs`。
- [ ] 外部仓批次三逐仓完成迁移并记录（auto-down tmp 探针与 auto-shell 陈旧 worktree 除外，需显式记录豁免理由）。
- [ ] `cargo ta` 全测试族通过。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

### 批次一：语言层（本仓 worktree）

- T1: master 上 commit `.next-id` 与本 plan；建 `.worktrees/plan-470-dev`（分支 `plan-470-dev`）。
- T2: `crates/auto-lang/src/parser.rs` — 分发处加 `"rs"` 别名 + `deprecated` 区分；`use_rust_stmt_inner` + W0005 warning；parser 单测 3 个（别名解析/旧拼写告警/新拼写无告警）。验证：`cargo check -p auto-lang` + 对应模块测试。
- T3: `crates/auto-lang/src/use_scanner.rs` — `.rs ` 前缀分支 + 单测。验证：`cargo t use_scanner`。
- T4: `crates/auto-lang/src/compile.rs:479` 错误消息更新；Rust 内嵌测试串迁移（plan442_parser_regress_tests、plan442_musk_backend_probe_tests、tests/a2r_tests、tests/ffi_dual_tests）。验证：`cargo check -p auto-lang`。
- T5: 新增 `crates/auto-lang/test/a2r/14_modules/002_use_rs/`（use_rs.at + use_rs.expected.rs）。验证：`cargo t a2r`。

### 批次二：本仓资产与文档全量迁移

- T6: 测试资产 .at 机械化替换（a2r 24、cookbook 88、vm 11、ffi_dual 2 → `use.rs`）。
- T7: 非测试 .at：`stdlib/auto/storage.rs.at`、`parity/libs/rust/rusqlite/auto/rusqlite.at`、`examples/playground-demo/`×3、`docs/tour/ch12-interop/`×4；`auto/lib/parser.at` 按同步链处理（见 D4）。
- T8: 文档：`website/rust.md`、`website/zh/rust.md`、`docs/design/strategy/`×2、`docs/specs/` 现行 design/overview 页（11 个 specs 命中中，plans.md/retrospective 类历史页只加迁移注记）。
- T9: `docs/plans/KNOWN-DEBT-AND-RISKS.md` 记录：deprecation 周期、未来移除 plan 的触发条件、auto/lib 快照重新同步义务（如适用）。
- T10: 收尾验证（Category B/C 门禁）：`cargo check -p auto-lang` → `cargo ta`（全测试族一次）；`grep -rn "use\.rust" --include="*.at"` 正式树归零确认；更新 plan 勾选与 frontmatter（execution_done）。

### 批次三：外部仓库迁移（各仓直接执行，不占本仓 worktree）

- 前置：批次一、二合入 master 并发版（或部署到各仓使用的工具链）。
- T11: auto-musk（30 .at + 10 md，`backend/` 为主）— 替换后跑该仓构建/测试命令。
- T12: auto-ai（22 .at + 8 md，`crates/` 为主）— 同上。
- T13: book（22 .at + 21 md）— 书稿示例迁移，正文注明 `use.rs` 为现行写法。
- T14: auto-down — tmp 探针豁免（记录）；md 2 处顺手更新。
- T15: auto-shell — 主树 md 3 处更新；`.worktrees/` 158 命中不直接改，记录"随活跃分支合并时按 warning 指引迁移"。
- T16: auto-code-rs（2 md）+ auto-forge（1 md）文档更新；批次三完成后在本 plan 复审记录中逐仓登记结果。

## 复审记录

（待 /auto-plan:review 填写）

## 待澄清事项

1. 未来移除 `use.rust` 的时机：外部仓迁移完成 + 一个发布周期零存量后，独立 plan 执行（parser/scanner 删分支改报错）。本 plan 不引入 error。
2. `auto/lib/parser.at` 的同步链若无法在本 plan 内确认，T7 允许降级为"手改 + KNOWN-DEBT 登记"。
3. 外部仓各自是否有独立 AGENTS.md 工作流要求（如 worktree 规约）：批次三执行时逐仓确认，本 plan 只约定迁移清单与验证责任。
