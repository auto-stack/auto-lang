---
plan_id: PLAN-587
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B 清障一——rust-server 产物落点可配（解析序化）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [reports/P587-1, reviews/P587-2]
touched_goals: [GOAL-010]             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man/rust_ui]   # 受影响的 specs 路径
current_step: 6
total_steps: 6
---

# [PLAN-587] Stage B 清障一——rust-server 产物落点可配（解析序化）

## 变更摘要

Design 01 §4-P2（Stage B P-2 批）：`ensure_shared_workspace` 产物落点硬编码
框架仓 `examples/rust-workspace/`（P584-D1 在案债；PLAN-586 worktree 期活实例
实证——测试链重写共享工作区）。本计划落点解析序化：

`AUTO_RUST_WORKSPACE`（新 env，设置即权威）→ **框架仓内项目 = 共享工作区**
（既有行为零变化：in-framework 检测）→ **仓外项目 = 项目仓内
`<project_dir>/rust-workspace/`**（产物不再落框架仓，V4 零污染）。

门档：Category B（auto-man 局部）；worktree=lang-587。

## 目标

1. 仓外项目（auto-kanban/auto-os apps 等）rust 后端产物 member 落项目仓内，
   auto-lang `git status` 零新增产物（V4）。
2. 框架仓内既有项目（examples 系/auto-musk-back）行为零变化（同共享工作区）。
3. env 覆盖通道（AUTO_RUST_WORKSPACE）可用于钉死/测试隔离。

## 架构方案

| 件 | 落点 | 说明 |
|---|---|---|
| D1 落点解析序 | rust_ui.rs `resolve_rust_workspace_dir(project_dir)` 新 pub fn | env → in-framework 检测（project_dir 在框架根下→共享 ws）→ project-local |
| D2 参数化 | `compute_auto_lang_rel_path`/`compute_target_rel_path` 增 ws_dir 参 | rel 起点从框架 ws 改为实际落点 ws（`compute_relative_path` 既有） |
| D3 接线 | ensure_shared_workspace + start_api_server/build_rust_ui/run_rust_ui 三处直调点 | 全走 resolve（单一语义） |

## 需求分析与背景调查

（2026-09-07 实测锚点。）`get_rust_workspace_dir`（rust_ui.rs:1809）CWD 上行
+兄弟探测找框架根；`ensure_shared_workspace`（:1979）建目录/扫 member/重写
workspace Cargo.toml（auto-lang path 依赖注入）+.cargo/config.toml target-dir
（指框架 target，编译缓存共享）；直调点：start_api_server（:2077）/
build_rust_ui（:2262）/run_rust_ui（:2297）均带 project_dir。api_gen 四调用
点经 ensure 漏斗无需改。P584-D1 活实例（KNOWN-DEBT 在案）：跨检出共享工作区
+落点硬编码复合病灶。

## 详细设计

- in-framework 检测：`fw_ws.parent().parent()` = 框架根（验 crates/auto-lang
  存在），project_dir 前缀判断（canonicalize + 小写归一，Windows 宽容）。
- project-local ws 复用全部既有装配（member 扫描/重写/config.toml），仅
  auto-lang path 依赖与 target-dir 的 rel 起点换为实际 ws——
  `compute_relative_path` 已为跨仓相对定位而生（Design 01 原文）。
- 不写额外 .gitignore（产物归属由项目仓主自定；框架 ws 受管不动）。

## 测试设计

Category B：`cargo check -p auto-man` + rust_ui/api_gen 模块测试；折叠前
`cargo tf`。单测：解析序三分支（env 权威/框架内→共享/仓外→project-local，
worktree 内路径 vs temp 路径天然两态）；落点装配（仓外 temp 项目 ensure→
ws 落位+rel 路径正确+**框架 ws Cargo.toml 内容不变**断言）；V4 实证
（temp 项目 generate→auto-lang git status 干净）。

## 验收标准

- [x] 解析序三分支单测绿（env 权威/框架内零变化/仓外 project-local）。
- [x] 仓外项目 ensure 后：ws 落 `<project>/rust-workspace/`，auto-lang
      `git status` 零新增产物（V4）；框架 ws Cargo.toml 字节不变。
- [x] 框架仓内项目解析结果 = 既有共享工作区（零变化断言）；015-notes
      既有生成测试零回归。
- [x] V6 门档：check 绿 + auto-man lib 269/269；tf 2619/2620 唯红 charts
      预存与基线一致。
- [x] Design 01 §7 P-2 行+§4-P2 落地注记 + KNOWN-DEBT P584-D1 结案
      （auto-os 侧随交付）。

## 执行步骤

1. [✅ 已完成] **D1** `resolve_rust_workspace_dir` + `path_contains`
   （canonicalize 双方同源——任一失败双方退化原样，防 verbatim `\?\` 错配，
   实测教训回修）+ 三分支单测（env 权威/框架内零变化/仓外 project-local）。
   [✅ 已完成] `rust_workspace_resolution_order` 绿。
2. [✅ 已完成] **D2** `compute_auto_lang_rel_path`/`compute_target_rel_path`
   增 ws_dir 参（rel 起点从框架 ws 改为实际落点）；ensure_shared_workspace
   接线 resolve。
3. [✅ 已完成] **D3** start_api_server/build_rust_ui/run_rust_ui 三直调点改
   resolve（直调残留=函数定义+resolver 内部+target 共享，均正确）；
   `ensure_workspace_lands_project_local` 绿（落位+manifest+config+框架 ws
   字节不变+幂等）。
4. [✅ 已完成] **V4 实证**：`generate_rust_ui_out_of_repo_lands_project_local`
   ——仓外 helloworld fixture 完整生成链 member 落
   `<tmp>/helloworld-out/rust-workspace/helloworld-out/`，框架共享工作区
   Cargo.toml 字节不变（"Generating Rust UI code" 全链执行非 skip 分支）。
5. [✅ 已完成] **门档**：`cargo check -p auto-man` 零错；auto-man lib
   269/269；折叠前 tf **2619/2620 唯红=test_charts_gallery_compiles（在册
   charts 预存）与基线一致**。
6. [✅ 已完成] **收口**：Design 01 §4-P2 落地注记+§7 P-2 行 ✅（auto-os
   acf6a39 随后提交）；KNOWN-DEBT P584-D1 结案；specs.json P587-1/2 沉淀；
   auto-man/plans.md 587 行回写；INDEX 重生；归档终态。

## 复审记录

（2026-09-07 复审，verify-don't-trust。）

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| C1 | 解析序三分支 | PASS | `rust_workspace_resolution_order`（含 verbatim 前缀教训回修） |
| C2 | V4 零污染 | PASS | `ensure_workspace_lands_project_local` + `generate_rust_ui_out_of_repo_lands_project_local`（框架 ws 字节不变双断言；生成链全跑非 skip） |
| C3 | 框架内零变化 | PASS | in-framework 解析=共享 ws 断言 + 015-notes 既有测试绿 |
| C4 | V6 门档 | PASS | check 零错；269/269；tf 2619/2620 唯红 charts 预存（基线一致） |
| C5 | 双仓簿记 | PASS | Design 01 §4-P2 注记+§7 行；P584-D1 结案；P587-1/2 沉淀 |

遗漏/延后扫描：无——方案即 Design 01 §4-P2 原文；`shared_cargo_target_dir`
target 缓存共享语义保留（设计内）；无 workaround（verbatim 前缀问题为实测
教训当场回修并有注释）。健康：零新增警告；无 debug 残留。

## 待澄清事项

（无——Design 01 §4-P2 方案即定案；in-framework 零变化 gating 为方案原文
「框架仓内既有项目行为零变化」的直接实现。）

## 变更摘要

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

## 复审记录

## 待澄清事项
