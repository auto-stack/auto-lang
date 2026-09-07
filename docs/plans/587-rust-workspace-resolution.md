---
plan_id: PLAN-587
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B 清障一——rust-server 产物落点可配（解析序化）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-man/rust_ui]   # 受影响的 specs 路径
current_step: 1
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

- [ ] 解析序三分支单测绿（env 权威/框架内零变化/仓外 project-local）。
- [ ] 仓外项目 ensure 后：ws 落 `<project>/rust-workspace/`，auto-lang
      `git status` 零新增产物（V4）；框架 ws Cargo.toml 字节不变。
- [ ] 框架仓内项目解析结果 = 既有共享工作区（零变化断言）。
- [ ] V6 门档：`cargo check -p auto-man` 绿 + 模块测试绿；折叠前 `cargo tf`
      与基线一致（charts 预存豁免）。
- [ ] Design 01 §7 P-2 行回填 + KNOWN-DEBT P584-D1 结案（auto-os 侧随交付）。

## 执行步骤

1. [ ] D1 `resolve_rust_workspace_dir` + `path_contains` + 三分支单测。
2. [ ] D2 rel-path 函数参数化（ws_dir 参）+ ensure_shared_workspace 接线。
3. [ ] D3 三处直调点改 resolve + 落点装配单测（含框架 ws 不变断言）。
4. [ ] V4 实证：temp 仓外项目 generate→落位+两仓 status 检查。
5. [ ] 门档：check+模块测试+折叠前 tf。
6. [ ] 收口：Design 01 §7 回填+P584-D1 结案+specs 沉淀+归档。

## 复审记录

（待 /auto-plan:review 填写。）

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
