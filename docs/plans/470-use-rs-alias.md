---
plan_id: PLAN-470
status: execution_done         # 批次一二完成;批次三(外部仓)待发版后执行,归档随之
feature_name: use-rs-alias
author: []
created_at: 2026-08-28
updated_at: 2026-08-30

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: ["parser use.rs 别名语法 + use.rust W0005 deprecation（imports/syntax）"]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: ["auto-lang/frontend", "auto-lang/runtime"]  # 受影响的 specs 路径
current_step: 10
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

- [x] `use.rs` 与 `use.rust` 解析出 AST 相等；VM 与 Rust 转译目标行为一致（010_use_rs 用例通过，expected 与 001 逐字节相同）。【test_use_rs_alias_parses_like_use_rust + test_14_modules_010_use_rs PASS】
- [x] `use.rust` 触发 W0005 DeprecatedFeature（"use 'use.rs' instead"），`use.rs` 不触发。【test_use_rust_emits_deprecation_warning / test_use_rs_no_deprecation_warning PASS】
- [x] 本仓正式树 `.at` 资产 `use.rust` 命中归零（豁免：`auto/lib/parser.at` 同步快照、`docs/plans/reports/429-b2-perf/bench.at` 历史报告、`010_use_rs/use_rs.at` 注释中的刻意说明）。
- [x] website/tour/design/specs 现行文档推荐写法为 `use.rs`（12 文件更新 + website 双语页加废弃注记；specs plans.md×3/retrospective 历史页保留）。
- [ ] 外部仓批次三逐仓完成迁移并记录（**前置未满足**：需批次一二发版部署工具链；KNOWN-DEBT 已登记周期与触发条件）。
- [x] 全测试族通过（相对 master 基线零新增失败；见复审记录基线对照）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

### 批次一：语言层（本仓 worktree）

- T1: master 上 commit `.next-id` 与本 plan；建 `.worktrees/plan-470-dev`（分支 `plan-470-dev`）。
  [✅ 已完成] worktree 自 master ecff09c5b 建立；plan 骨架已先行入档（786eb18ae）。开工前核验暂停分支（auto-down-dev/auto-musk-dev/plan-484-dev/plan-485-dev）与迁移集零交叠。
- T2: `crates/auto-lang/src/parser.rs` — 分发处加 `"rs"` 别名 + `deprecated` 区分；W0005 warning；parser 单测 3 个。
  [✅ 已完成] **设计微调**：未引入 `use_rust_stmt_inner`，改为分发处 `name == "rust" || name == "rs"` + `rust` 拼写直接 `self.warn(Warning::DeprecatedFeature)`（span 取 lang 关键字位 kw_span），`use_rust_stmt` 签名零改动。3 测 PASS（test_use_rs_alias_parses_like_use_rust / test_use_rust_emits_deprecation_warning / test_use_rs_no_deprecation_warning）。
- T3: `crates/auto-lang/src/use_scanner.rs` — `.rs ` 前缀分支 + 单测。
  [✅ 已完成] `.rs `/`.rs\t` 分支置于 `.rust ` 之前；3 测 PASS（simple/with_items/新旧混排）。
- T4: `compile.rs:479` 错误消息更新；Rust 内嵌测试串迁移。
  [✅ 已完成] 消息改 "Add `dep {}` before `use.rs`."；plan442_parser_regress_tests（2 串）、plan442_musk_backend_probe_tests（1 串 + **辅助扫描器补 `use.rs ` 前缀**——代码级 strip_prefix，别名必需）、ffi_dual_tests（3 串+机制注释）、parser.rs 内嵌 1 串、trans/c.rs 错误消息、trans/rust.rs 2 处现行语法注释。源码级 `"use.rust"` 字面量已清零（grep 证实）。
- T5: 新增 a2r 端到端用例。
  [✅ 已完成] **编号调整**：`002_pub_use` 已占用 → 落位 `14_modules/010_use_rs/`（use_rs.at + use_rs.expected.rs，expected 与 001 逐字节一致），a2r_tests.rs 注册 test_14_modules_010_use_rs，PASS。

### 批次二：本仓资产与文档全量迁移

- T6: 测试资产 .at 机械化替换（a2r 24、cookbook 88、vm 11、ffi_dual 2 → `use.rs`）。
  [✅ 已完成] 125 文件 sed 替换（GNU grep 列表驱动；首次尝试因环境 ugrep 包装输出 NUL 分隔路径而失败，改 `command grep` 后成功）。
- T7: 非测试 .at：stdlib/parity/playground×3/tour×4；`auto/lib/parser.at` 按同步链处理。
  [✅ 已完成] 9 文件迁移。**parser.at 裁定豁免**：勘察确认其 `use.rust` 仅存在于头注（"use(含 use.rust)" 范围描述），无关键字分发代码；系 Plan 432 钉在 b3bd64f5 的同步快照（M2 闸门被阻），手改无意义——降级为 KNOWN-DEBT 登记"快照漂移，随 Plan 432 重生成携带"。
- T8: 文档：website×2、strategy×2、specs 现行页。
  [✅ 已完成] 12 文件替换 + website 双语页 Rust 嵌入模式 bullet 加 `use.rs` 推荐/`use.rust` 废弃注记；specs plans.md×3 + retrospective + ui/plans.md 历史页保留原拼写。
- T9: `docs/plans/KNOWN-DEBT-AND-RISKS.md` 登记。
  [✅ 已完成] 🟢 已知限制表新增两行：P470 deprecation 周期（含移除触发条件与豁免清单）、P470 parser.at 快照漂移。
- T10: 收尾验证 + 归零确认。
  [✅ 已完成] `cargo t` 3285/3285 全绿；`cargo tv`/`cargo tt`/`cargo tb` 与 master 基线逐失败对照零新增（见复审记录）；正式树 grep 归零确认（仅 2 豁免 + 1 刻意注释）。

### 批次三：外部仓库迁移（各仓直接执行，不占本仓 worktree）

- 前置：批次一、二合入 master 并发版（或部署到各仓使用的工具链）。
  [⏸ 待发版后执行] **不可提前**：外部仓工具链尚不识别 `use.rs`，先行替换会破坏其构建。各仓升级工具链后以 W0005 告警为迁移信号，按 T11-T16 清单执行并在复审记录逐仓登记。

## 复审记录

**批次一、二复审（2026-08-30，执行者自审 + 基线对照）**：

1. **验收逐项核对**：见验收标准勾选——批次一二全项达成；批次三一项未竟（前置=发版，结构性等待非遗漏）。
2. **基线对照（关键证据）**：迁移前在 master 同跑三档，失败集合完全一致——`cargo tt` 2 败（a2r 02_types_001_struct/004_pointer，`1 as u32` 强转渲染，在飞工作存量）；`cargo tv` 3 败（cookbook cb_devtools_log_error/cb_asynchronous_channel + aavm2_m4_corpus，master 带 use.rust 原文同样失败）；`cargo tb` 2 败（存量）。**worktree 相对基线零新增失败**。
3. **worktree 环境假象甄别**：worktree 内 `cargo tb` 19 败——book_listing_tests 以 `../../../book/rust/listings` 相对路径读外部 ../book 仓，从 `.worktrees/plan-470-dev` 解析到不存在的 `.worktrees/book`（os error 3 路径找不到，非断言失败）；master 上回归 2 败基线。非本计划回归。
4. **遗漏/延后扫描**：批次三延后已显式登记（前置+触发条件）；parser.at 豁免裁定有勘察证据（仅注释命中）；无未经批准的 workaround（设计微调 2 处已在 T2/T5 证据中注明：去 inner 包装、编号 002→010）。
5. **健康检查**：`cargo check` 无新增警告（160 为存量）；无调试打印；新代码格式与邻近代码一致（仓库整体非 rustfmt-clean，未引入新差异）。
6. **spec-impact**：`new_spec_components`: parser `use.rs` 别名 + W0005（syntax/imports）；`touched_goals`: FFI 导入语法一致性。前端 overview/module-resolution、runtime architecture/ffi-bridges、types design 三处 spec 文档已同步改为 `use.rs` 表述。

## 待澄清事项

1. 未来移除 `use.rust` 的时机：外部仓迁移完成 + 一个发布周期零存量后，独立 plan 执行（parser/scanner 删分支改报错）。本 plan 不引入 error。
2. `auto/lib/parser.at` 的同步链若无法在本 plan 内确认，T7 允许降级为"手改 + KNOWN-DEBT 登记"。
3. 外部仓各自是否有独立 AGENTS.md 工作流要求（如 worktree 规约）：批次三执行时逐仓确认，本 plan 只约定迁移清单与验证责任。
