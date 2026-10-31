---
plan_id: PLAN-900
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: de-ui-dependency-inversion
author: [zcode-session 2026-10-31]
created_at: 2026-10-31
updated_at: 2026-10-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: [auto-lang/modularity(ui 边界段), auto-lang/api-gen(生成工程依赖面)]
new_spec_components: [auto-lang/modularity(零 auto-ui 依赖不变量)]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang, auto-lang-core, auto-api-gen, auto-ui]
current_step: 0
total_steps: 10
---

# [PLAN-900] auto-lang 去 UI（依赖方向反转，Design 52 计划 A）

## 变更摘要

反转 auto-lang→auto-ui 的依赖方向为 auto-ui→auto-lang 基础 crate：facade
`auto-lang` 删除全部 auto-ui 依赖与 UI feature 族，`default = []`；UI 解析/生成
管线、auto-lsp、UI 测试/examples、cosmic demos 迁至 auto-ui 仓；api-gen 生成工程
直接声明 auto-ui-runtime。完成后 auto-lang 工作区零 auto-ui 包（工作区层环解除），
worktree 单仓可独立构建，UI 集成红族（vue×11/ui_snapshots×3/gallery）随迁出
auto-ui 侧处置。

## 目标

1. `cargo tree -p auto-lang -p auto-lang-core | grep auto-ui` 为空；`git grep auto-ui crates/*/Cargo.toml` 零命中
2. auto-lang `default = []`；ui/ui-iced 等 16 个转发 feature 与 3 条 auto-ui 依赖删除
3. 迁移件在 auto-ui 仓全部落位编译（auto-lsp、examples、集成测试、cosmic demos、UI 解析管线装配点）
4. auto-api-gen 生成工程 manifest 直接声明 auto-ui-runtime；存量生成样例（rust-workspace）可构建
5. modularity 夹具（ui-repository-consumer/ui-tool-consumer/media-core/language-sdk）更新并绿
6. UI 集成红族随迁出；sys_natives_vm_tests 先分类（疑非 UI 域，若是 VM 域则留仓内另行 triage）
7. 冻结门：`cargo t`（daily）+ 受影响组并集绿（已知红具名）；`cargo build` 全工作区绿

## 架构方案（按 Design 52 §2.1）

- **依赖方向**：auto-ui → auto-lang 基础 crate（core/parser-api/ast/atom/val/lex/
  media/text/terminal/a2r-std/tool-*）；auto-lang facade 零 auto-ui。
- **UI 解析管线归置**（T-01 精确测绘后定案，预案二选一或组合）：
  a. **库内装配迁移**：facade 只保 core parse；`parse_domain(&UiDialect)`+aura/
     design_tokens/ui_gen/a2ui 组合点迁 auto-ui（auto-ui-runtime/compiler 侧新
     装配函数），auto-man 的 auto CLI/auto-ui 消费方直接调用；
  b. **子进程委托**：生成长流程经 AUTO_UI_TOOL_BIN 先例走 auto-ui-tools 子进程
     （已有跨仓边界先例，root Cargo.toml 244-245）。
- **host 能力安装**：`install_ui_host_capabilities`+SDK 钩子（lib.rs:252-259）迁
  auto-ui 侧 bootstrap；facade 的 `set_ui_host_install_hook` 机制随迁或删除。
- **14 处 `pub use` 转发删除**：破坏面=仓内 src/tests（~145 处，随测试迁移走）、
  auto-lsp（随迁）、api-gen 模板字符串（改直连）、外部生成物（改 manifest 直连）。
- **feature 族处置**：ui/ui-iced/ui-interpreter/ui-headless/image-pipeline/
  highlight-treesitter/ui-clipboard/ui-dialog/iced-layout-tests/native-dock/
  native-clipboard/native-dnd/autodown/test-native-dock/mpv-* 全删；
  code-editor 为双 owner（auto-lang-core/code-editor 是 SDK a2r diff 双轨门，
  **非 UI 渲染，保留**，其 auto-ui-runtime 转发臂删除）。

## 需求分析与背景调查

- Design 52（docs/design/52-single-compile-profile-consolidation.md）：终态合同。
- 依赖测绘（2026-10-31 本会话）：facade lib.rs ~17 处无门生产使用（:94/:105-128
  转发、:252-259 host 安装、:620-680/:755-782 UI 解析管线、:871/:1309/:1332 其他）；
  auto-lsp completion.rs:195-196 唯一其他生产消费者；cosmic demos 直连 auto-ui；
  基础 crate 全净；环仅在工作区层面。
- 896/898 后测试范式：迁移测试用 `cargo t` 固定组语义（组定义随迁 auto-ui 侧）。
- 预存红族（898-N5）：vue_capabilities×11、ui_snapshots×3、gallery_vue_golden
  属 UI 域随迁；sys_natives_vm_tests 待分类。

## 详细设计

### 迁移清单（auto-lang → auto-ui）

| 件 | 来源 | 去处 |
|---|---|---|
| auto-lsp crate | crates/auto-lsp | auto-ui/crates/auto-lsp（completion 直用 auto-ui-compiler::aura） |
| UI examples ×29 | crates/auto-lang/examples（除 a2r_step*/run_vm_test 4 个外） | auto-ui/examples；`required-features` 随迁 |
| UI 集成测试 ×3 | crates/auto-lang/tests/{component_registry_test,desktop_behavior,vue_capabilities}.rs | auto-ui/tests（红族随迁，auto-ui 侧债务登记） |
| src/tests UI 树 | crates/auto-lang/src/tests 内 UI 域 ~30 文件 | auto-ui 侧 auto-ui-runtime/compiler 测试树 |
| cosmic demos ×2 | crates/auto-cosmic/{demo,host-libcosmic} | auto-ui/crates/auto-cosmic/*（ports/host-smithay 留或随迁，T-01 定） |
| UI 解析管线装配 | lib.rs 两处 parse_domain+aura/dt/ui_gen/a2ui | auto-ui 侧装配函数（预案 a/b） |

### Cargo 面

- crates/auto-lang/Cargo.toml：删 auto-ui-compiler/auto-ui-runtime/auto-ui-dialect
  三依赖与 16 feature；default = []；[[example]] required-features 清理。
- 根 Cargo.toml：members 调整（auto-lsp/cosmic demos 移出）；exclude 不变
  （experimental/test/fixtures 已排除）；workspace.dependencies 里 UI 相关项清理。
- auto-ui 仓：members 增补；其 auto-lang 基础 crate path 依赖不变。

### api-gen/夹具

- crates/auto-api-gen/src/api_gen.rs 模板（:3607-3779 auto_ui_runtime 引用 +
  manifest 生成处）改直接声明 auto-ui-runtime 依赖（版本/path 策略 T-04 定）。
- test/fixtures/modularity/{ui-repository-consumer,ui-tool-consumer}：断言面从
  "facade 带出 UI" 改 "auto-ui 直连"；media-core/language-sdk-consumer 不动（本无 UI）。

## 测试设计

- 迁移对账：每类件"来源删除清单 = 去处新增清单"逐文件核对（防丢测试）；测试
  身份计数前后对比（896 select-only 口径）。
- 单仓独立性：`git worktree add` 裸 auto-lang（无兄弟仓）`cargo check -p auto-lang` 通过。
- 生成物回归：rust-workspace 样例 + 新生成一个最小 UI 应用，构建绿。
- 冻结门：`cargo t`（daily，注意 daily 组的 --test 列表随迁移调整）+ parser/infer/
  vm/a2r 等受影响组并集；预存红按 898-N5 基线核销（UI 族随迁后应从 auto-lang 侧消失）。
- 体积记录：合并后主检出 clean build 对比（预期 iced/wgpu/ash/naga 移出）。

## 验收标准

1. [ ] `cargo tree -p auto-lang -p auto-lang-core` 无 auto-ui；workspace 无 auto-ui 依赖
2. [ ] `default = []` 且裸 `cargo build` 全工作区绿
3. [ ] 迁移件在 auto-ui 仓编译绿（auto-lsp/examples/测试/cosmic/管线装配）
4. [ ] api-gen 生成工程可构建（存量样例 + 新生成）
5. [ ] modularity 夹具绿；迁移对账零丢失
6. [ ] UI 集成红族从 auto-lang 侧测试面消失（随迁 auto-ui 侧登记）
7. [ ] 独立复审 PASS；spec delta 回写

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T-01 管线测绘定案：lib.rs 两处 UI 解析管线的完整调用图（谁调 parse→aura→ui_gen/
  a2ui：autogen bin？auto CLI？tests？）；cosmic ports/host-smithay 归属核查；
  sys_natives_vm_tests 域分类。[验证：调用图清单入本计划；预案 a/b 选型定案]
- T-02 auto-ui 侧接收面：auto-ui worktree 建 crates/auto-lsp、examples、tests、
  cosmic 落位骨架（先空/引用来源路径）。[验证：auto-ui cargo check 新骨架绿]
- T-03 UI 解析管线迁移：按 T-01 定案实施（库内装配迁移或子进程委托）；facade
  lib.rs 17 处 UI 使用清零（含 14 pub use、host 安装、parse_domain）。[验证：
  cargo check -p auto-lang --no-default-features 绿；调用方回归]
- T-04 api-gen 模板与生成工程 manifest 改造。[验证：rust-workspace 样例构建 + 新生成]
- T-05 测试迁移：3 集成测试 + src/tests UI 树 + auto-lsp 测试迁 auto-ui；test-groups
  组定义调整（daily 的 --test 列表、受影响组）。[验证：迁移对账零丢失；cargo t daily 绿]
- T-06 examples/cosmic 迁移与 required-features 清理。[验证：cargo build --examples
  于 auto-ui 侧绿；auto-lang 侧 examples 仅剩 4 个非 UI]
- T-07 Cargo 终态：default=[]、依赖/feature/workspace members 清理、根 workspace
  依赖清理。[验证：cargo tree 零 auto-ui；metadata 解析]
- T-08 夹具与文档：modularity 夹具 ×2、AGENTS/specs（modularity 不变量）、
  Design 52 状态更新。[验证：夹具绿；grep 无残留引用]
- T-09 独立 worktree 单仓验证 + 体积记录。[验证：裸 worktree cargo check；体积数字]
- T-10 冻结门：受影响组并集 + daily + 对账。[验证：全绿或已知红具名核销]

## 复审记录

## 待澄清事项

- 预案 a/b 选型（T-01 数据说话；倾向 a+b 组合：库内装配给库消费者、子进程给生成长流程）
- cosmic ports/host-smithay（无 auto-ui 依赖）留 auto-lang 还是整体随迁（倾向随迁聚拢，T-01 定）
- mpv-*/highlight-treesitter 等 auto-ui 侧 feature 保留策略属计划 D，不在本计划
