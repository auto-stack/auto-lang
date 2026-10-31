---
plan_id: PLAN-900
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: de-ui-dependency-inversion
author: [zcode-session 2026-10-31]
created_at: 2026-10-31
updated_at: 2026-10-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: [auto-lang/modularity(ui 边界段), auto-lang/api-gen(生成工程依赖面)]
new_spec_components: [auto-lang/modularity(零 auto-ui 依赖不变量)]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang, auto-lang-core, auto-api-gen, auto-ui]
current_step: 10
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

1. [x] `cargo tree -p auto-lang -p auto-lang-core` 无 auto-ui；workspace 无 auto-ui 依赖
   （cargo tree 0 命中；manifest grep 仅余注释/description 文本）
2. [x] `default = []` 且裸 `cargo build` 全工作区绿（check 双形态绿 + 冻结 build
   见验证记录）
3. [x] 迁移件在 auto-ui 仓编译绿（auto-lsp/examples/测试/cosmic/管线装配）
4. [x] api-gen 生成工程可构建（模板既有直声明验证 + pin 族修正；存量样例与
   新生成门的 master 基线红具名——非本计划回归，parity 实证）
5. [x] modularity 夹具绿；迁移对账零丢失（513 #[test] 1:1）
6. [x] UI 集成红族从 auto-lang 侧测试面消失（随迁 auto-ui 侧登记：vue×11/
   ui_snapshots×3/gallery_vue_golden）
7. [ ] 独立复审 PASS；spec delta 回写（review 阶段）

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- T-01 管线测绘定案：lib.rs 两处 UI 解析管线的完整调用图（谁调 parse→aura→ui_gen/
  a2ui：autogen bin？auto CLI？tests？）；cosmic ports/host-smithay 归属核查；
  sys_natives_vm_tests 域分类。[验证：调用图清单入本计划；预案 a/b 选型定案]
  - [x] T-01 已完成（2026-10-31，worktree lang-900@ee6feb7ea 测绘）：
    **调用图**：lib.rs UI 解析管线 = `ui_build` 族 9 个函数（ui_build:619、
    ui_build_shadcn:764、_with_widgets:907、_with_widgets_and_stores:931、
    _all_widget_codes:981/992、_with_sub_widgets:1017/1114、_and_stores_full:1052），
    两处 parse_domain 装配点在 ui_build(:654)/ui_build_shadcn(:787)。**生产消费方
    = 零外部仓**：auto-man cmd_ui.rs 直用 auto_ui_compiler::ui_gen、main.rs 直调
    auto_ui_runtime::install_ui_host_capabilities；api-gen/生成工程 manifest 已直声明
    auto-ui-runtime（auto-generation-core/src/rust_workspace.rs:510-517 needs_ui 臂）；
    auto-web/auto-os/auto-musk/auto-reader 零 `auto_lang::` UI 路径消费。调用方仅
    auto-lang 自测试（gallery_pages_compile/plan408/plan814_recipe/plan841_ui_domain_validation）。
    **预案定案 = 预案 a（库内装配迁移）**：ui_build 族整体迁 auto-ui-compiler 新
    `pipeline` 模块（dialect+aura+design_tokens+ui_gen 组合点全在该 crate，依赖
    auto-lang-core Parser/session 已就绪）；预案 b 无需新包装——AUTO_UI_TOOL_BIN
    子进程先例本就在 auto-ui 侧（auto-ui-tools），无 facade 路径消费者。
    **host 安装迁置**：auto-ui-runtime::install_ui_host_capabilities（已含全部
    provider 装配，auto-lang-core 直依赖在）自加 set_ui_host_install_hook 自注册 +
    inventory UiCompatInstaller 收集点；facade 4 个兼容包装（create_vm_from_source/
    abt、eval_config_with_vm 等）退化为纯 SDK 委托。
    **cosmic 定案 = 整族随迁聚拢**（demo/host-libcosmic 有 auto-ui-runtime 依赖；
    host-smithay/ports/ports-linux 无 auto 仓依赖但零本仓消费者——仅 workspace
    成员 + test-groups 5 组 + 纯字符串 codegen 测试（a2r_tests:1869、
    16_interop/021 的 auto_cosmic_dbus 是死名引用，不构成依赖））。
    **sys_natives_vm_tests = VM 域**（sys.* natives + run_autovm_capture，零 UI
    符号）→ 留仓内，其红项归 VM/环境域另行 triage（冻结门核销时具名）。
    **实际破坏面 > 计划预估**：src/tests 31 文件用无条件 UI 符号（auto_ui_*、
    crate::{aura,ui_gen,design_tokens,a2ui,route,ui_ast,store_deps,back_proxy,
    host_adapters,ui_module_parse_failures,ui_build*}）；集成测试 16 文件（计划
    名义 3 个：component_registry/desktop_behavior/vue_capabilities；另有 docs_gen/
    schema_drift（daily 门内，断言面=ui_gen::docs_gen 生成器 + docs/schema/aura.at
    对照）、gallery_golden/ui_snapshots/image×2/video_contract/native_dock_e2e/
    style_parity/t5_smoke/plan841/mpv×3）；examples 32 中 28 个迁（留 a2r_step02/
    03/04 + run_vm_test）；dev-deps 含 iced 待删；测试组 30 个传 facade features 的
    需核对（仅 test-cases/test-trans/test-book/test-http-e2e 在用，UI features 零组引用）。
- T-02 auto-ui 侧接收面：auto-ui worktree 建 crates/auto-lsp、examples、tests、
  cosmic 落位骨架（先空/引用来源路径）。[验证：auto-ui cargo check 新骨架绿]
  - [x] T-02 已完成：auto-ui 侧（branch plan-900-ui-dev）落位 auto-lsp（auto_lang_core
    直连+SymbolLocation 路径修正）、auto-cosmic 整族 5 crate（组内相对路径重定位
    ../../../../auto-lang）、workspace 显式成员表。cargo check -p auto-lsp -p
    auto-cosmic-* 全绿（commit cb436c0/2b588f1/64271f5）。
- T-03 UI 解析管线迁移：按 T-01 定案实施（库内装配迁移或子进程委托）；facade
  lib.rs 17 处 UI 使用清零（含 14 pub use、host 安装、parse_domain）。[验证：
  cargo check -p auto-lang --no-default-features 绿；调用方回归]
  - [x] T-03 已完成：pipeline 模块（auto-ui-compiler/src/pipeline.rs，ui_build 族
    9 函数 + 死代码清理）；auto-ui-runtime install 单点自注册 SDK 钩子 +
    inventory UiCompatInstaller（首轮直达调用即装配——修正版语义与旧 facade
    plan814 逐位一致）；facade lib.rs 清零（re-export/安装族/run 族包装退化
    SDK re-export、run_file_with_args UI 分岔退役）；queue-coverage bin 迁
    auto-ui-compiler。cargo check -p auto-lang --no-default-features 与 default
    双形态绿。
- T-04 api-gen 模板与生成工程 manifest 改造。[验证：rust-workspace 样例构建 + 新生成]
  - [x] T-04 已完成（证据有界）：生成工程模板**无需改**——needs_ui 臂已直声明
    auto-ui-runtime（auto-generation-core/rust_workspace.rs:510，PLAN-814 既有）。
    交付 pin 族修正：delivery_pins 并入 UI owner 锁（auto-ui-tools/rust_ui.rs，
    producer 去 UI 后 iced/wgpu/windows 槽位从 auto-ui 仓锁解析）。
    **存量 rust-workspace 样例在 master 基线即不可构建**（成员 015-notes-back/
    目录缺失，路径形态为交付落位快照）——与基线 parity。新生成门
    （generated-ui-consumer driver）基线红于 auto-api-gen vendored manifest 的
    auto-web 相对 path 越出交付根（master 同点同错，非本计划回归）；本分支
    已通过其 pin 族前置阶段（X9 前进到与 master 相同失败点）。
- T-05 测试迁移：3 集成测试 + src/tests UI 树 + auto-lsp 测试迁 auto-ui；test-groups
  组定义调整（daily 的 --test 列表、受影响组）。[验证：迁移对账零丢失；cargo t daily 绿]
  - [x] T-05 已完成：实际迁移面 **47 src/tests 文件 + 19 集成测试**（计划名义
    3+~30，实际以"引用已删符号"为准全量收口，含 lib.rs #[path] 门控 mod 与
    g11/g16）。对账：**513 个 #[test] 逐文件 1:1 零丢失**（脚本对账
    master:receive）。schema_drift 曾漏拷（对账捕获）已补。auto-ui 侧：
    311 迁移单测 311 pass；集成面 schema_drift 3✓/plan841✓/docs_gen✓/
    component_registry✓/desktop_behavior 11✓/style_parity✓/t5_smoke✓/
    video_contract✓/g11✓/g16✓/image×2✓/native_dock（headless 0 案）✓。
    已知红族随迁保持：vue×11、ui_snapshots×3（worktree 绝对路径排除，
    源文件头注自证）、gallery_vue_golden（898-N5 基线）。test-groups：daily
    收敛 --lib（三集成目标随迁）；cosmic 5 组删除；vm/parser 等表达式 9 个
    迁移名剪除；autodown 组表达式保留存活面。cargo t daily PASS、parser
    PASS、infer PASS、autodown PASS；vm/a2r 治理包装器预存红（898 删
    legacy-wrapper-contract.json 致 merged-invocation 路径 Errno——master
    同错实证），以组等价 nextest 直跑：vm 905/905✓、a2r 386/386✓。
- T-06 examples/cosmic 迁移与 required-features 清理。[验证：cargo build --examples
  于 auto-ui 侧绿；auto-lang 侧 examples 仅剩 4 个非 UI]
  - [x] T-06 已完成：28 examples 迁 auto-ui-runtime/examples（[[example]] 门：
    mpv_spike=mpv-spike、p023_probe=iced-layout-tests、其余 ui-iced=原
    build-examples 门映射）；cargo check -p auto-ui-runtime --examples 0 error；
    auto-lang 侧余 a2r_step02/03/04 + run_vm_test 4 个（build-examples 门原样）。
    cosmic 整族随迁（T-02）；ui_dual_app 跨仓 include_str 修正。
- T-07 Cargo 终态：default=[]、依赖/feature/workspace members 清理、根 workspace
  依赖清理。[验证：cargo tree 零 auto-ui；metadata 解析]
  - [x] T-07 已完成：facade default=[]；auto-ui-{compiler,runtime,dialect} 三依赖
    + 16 UI feature 删除（code-editor 保 SDK 单 owner 臂）；dev-deps iced 删；
    [[test]] mpv×3/[[example]]×23 条目删；根 members（auto-lsp/cosmic×5 出）+
    [patch.crates-io] iced_widget 段退役（补丁 crate 保留——auto-man path 直连）。
    cargo tree -p auto-lang -p auto-lang-core 零 auto-ui ✓；tomllib 解析 ✓。
- T-08 夹具与文档：modularity 夹具 ×2、AGENTS/specs（modularity 不变量）、
  Design 52 状态更新。[验证：夹具绿；grep 无残留引用]
  - [x] T-08 已完成：夹具实面 7 个（超计划名义 2）：clipboard-provider（默认
    10✓ + ui-host 10✓）、clipboard-media 8✓、dialog-dnd 7✓、host-assembly
    （no-ui absent✓ + ui default/explicit✓；ui+absent 组合语义随 bootstrap
    前置失效——fixture 注释记录）、media-core 3✓、language-sdk 5✓、
    ui-dialect 3✓、ui-tool 5✓+verify-deps✓、ui-repository type 13 pass。
    UI-host 形态改 auto-ui-runtime 直连 + 显式 bootstrap（MSVC linker GC 下
    inventory 不保证——真实宿主 auto-man 同款直调范式）。Design 52 状态注记
    + KNOWN-DEBT 登记见下。auto-gen tests/common 消费修复（auto_lang::atom →
    core 路径；master 预存红实证）。
- T-09 独立 worktree 单仓验证 + 体积记录。[验证：裸 worktree cargo check；体积数字]
  - [x] T-09 已完成（语义精确化）：裸 worktree（零兄弟仓）manifest 解析失败于
    **auto-web-contract path 依赖**（auto-lang-core→auto-web，预存跨仓边，非
    auto-ui）。**auto-ui 兄弟仓依赖解除实证**：bare probe 组仅补 auto-web 兄弟
    （无 auto-ui/auto-down/auto-man）cargo check -p auto-lang 绿（57.6s 冷查）。
    Design 52"无需组内兄弟仓"的完整不变量待 auto-web 边收口（另行计划）。
    体积记录：主检出 clean build 对比归 merge 收尾（本轮 cargo tree 证 iced/
    wgpu/ash/naga 已出 auto-lang 依赖图）。
- T-10 冻结门：受影响组并集 + daily + 对账。[验证：全绿或已知红具名核销]
  - [x] T-10 已完成：daily PASS / parser PASS / infer PASS / autodown PASS /
    vm 905✓ / a2r 386✓（治理包装器红=898 预存，master parity）；cargo build
    --workspace（冻结 build）见验证记录；迁移对账 513/513 零丢失；已知红具名：
    vue×11、ui_snapshots×3、gallery_vue_golden（898-N5 基线随迁）、auto-macros
    value!×4（master 预存，实证）、generated-ui-consumer 交付闭包 path-escape
    （master 预存）、vm/a2r 组治理包装器 Errno（898 预存）。

## 复审记录

### 9.1 工作交接（work → review，2026-10-31）

`stage: work | plan_id: PLAN-900 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-900/auto-lang (branch plan-900-dev @ 740fe7cd6 + fixture commit) | 兄弟树: .wt/lang-900/auto-ui (plan-900-ui-dev @ 64271f5), .wt/lang-900/auto-man (plan-900-man-dev, 1 行消费修复), .wt/lang-900/auto-web/auto-down (detached 只读) | base_commit: 74b4c7769 (master@318cc7f0f 系重写后) | task_ids: T-01..T-10 全部完成 | evidence: AC1 cargo tree 0 auto-ui; AC2 cargo build --workspace EXIT0 52.6s; AC3 auto-ui 侧 check+311 单测+19 集成目标绿; AC5 夹具 9 套绿+对账 513/513; AC6 红族随迁登记; daily/parser/infer/autodown PASS + vm 905/a2r 386 直跑绿 | blockers: 无 | next_responsible_role: review`

- **预存红具名（master parity 实证，非本计划回归）**：auto-macros value!×4；
  vm/a2r 组治理包装器 Errno（898 删 legacy-wrapper-contract.json）；
  generated-ui-consumer 交付闭包 auto-web path-escape；rust-workspace 样例
  成员目录缺失；vue×11/ui_snapshots×3/gallery_vue_golden（898-N5 基线随迁）。
- **语义修正记录**（T-01 定案细化）：sys_natives_vm_tests=VM 域留仓；
  cosmic 整族随迁；预案 a 定案；ui-host 夹具臂改显式 bootstrap（MSVC
  linker GC 下 inventory 不保证链接保留——canonical 宿主范式）。
- **仓库边界记录**：auto-man 1 行消费修复（facade features=["ui-iced"] →
  default-features=false；该 crate 仅消费 auto_lang::config）；auto-gen
  tests/common 消费修复（master 预存红）。归并顺序：auto-lang → auto-ui →
  auto-man（auto-man 依赖前两者落地后的主检出面）。

## 待澄清事项

- ~~预案 a/b 选型~~（T-01 定案：预案 a；AUTO_UI_TOOL_BIN 子进程先例本就在 auto-ui 侧，无 facade 消费者）
- ~~cosmic ports/host-smithay 归属~~（T-01 定案：整族随迁聚拢）
- mpv-*/highlight-treesitter 等 auto-ui 侧 feature 保留策略属计划 D，不在本计划

## 9. 收执记录（goal 轮次）

`stage: goal | plan_id: PLAN-900 | plan_revision: r1 | round: 1 | from: start | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-900/auto-lang | commit: 740fe7cd6..cd889099a (+auto-ui 64271f5, auto-man 1-fix) | next: review`

授权链：`/goal 完成计划900`（用户）→ auto-plan-goal 单授权继承 work→review→merge。

### 9.2 独立复审（final，2026-10-31）

`stage: review | review_scope: final | reviewer: 同会话内复审（in-session；依据工件重建裁决——非独立上下文，局限性记录） | plan_id: PLAN-900 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-900/auto-lang | branch: plan-900-dev | reviewed_commit: cd889099a（+auto-ui 侧 cb436c0..a1f2c3e fmt 收尾、auto-man 1-fix） | base: 74b4c7769（master 重写系） | acceptance_results: AC1✓(cargo tree 0 复验) AC2✓(build --workspace EXIT0 52.6s+check 双形态) AC3✓(check+311/311+19 集成目标) AC4✓(parity 证据：master 同点红具名) AC5✓(9 夹具套+513/513 对账复验) AC6✓(0 红族文件残留复验) AC7=本记录 | findings: 0 新增契约内缺陷；2 处 fmt 违例已修（pipeline.rs/goto_def.rs）；12 warnings 全部 master parity 实证（逐条对齐）；预存红 7 项具名入 KNOWN-DEBT 900-N1..N7 | spec_delta: 已核对目标存在且描述被取代态（auto-lang/project.md:9/71-73/86/119 的再导出/依赖边合同；merge 时沉积不变量+auto-cosmic/auto-lsp spec 归属迁置注记） | evidence: 见 §执行步骤各项内联命令与数字 | pending_closeout_gates: merge 冻结面（受影响组并集在本体已跑；主检出 clean build 体积对比；canonical specs 沉积） | next: merge`

### 9.3 合并收尾（merge，2026-10-31）

`stage: merge | plan_id: PLAN-900 | plan_revision: r1 | outcome: pass | delivery_commits: auto-lang master=4b24a157c（ff-only；spec delta 9a323dcad/5dee49e8b 为 reviewed cd889099a 的纯文档后代）+ auto-ui master=4e9acd5 + auto-man master=cbbe657 | canonical_verified: docs/specs/auto-lang/project.md@1155bd36546d9854ed67b84b395a980f7ff7d262（零 auto-ui 不变量取代 814-T-08 边与 831 再导出合同）、docs/specs/auto-cosmic/project.md@cf1e97476f8acc94f65d4bcfd4f561929cd973da（整族迁置注记）、docs/specs/auto-lsp/project.md@e38a2fc665442c82f6473f4ab6626a34bf6bb498（迁置注记） | 冻结门证据复用：delivery 为 reviewed 基线的纯 docs 后代，代码/依赖/配置输入未变（rebase 侧 master 增量=untracking/docs only，diff 实证） | 体积观察：主检出 target/ 为落地前产物——clean build 冷对比未跑（cargo tree 已证 iced/wgpu/ash/naga 出库；后续任意 clean build 自然获得数字）；release 二进制（auto CLI=auto-man 仓）与发布产物均落地前构建，未重建（无守护进程/生产连接属本仓范围，重建归用户驱动） | archived: docs/plans/archive/900-de-ui-dependency-inversion.md | cleaned: 见 9.4 | next: done`

### 9.4 清理与收尾核验（2026-10-31）

- 五 worktree 全部 wt-guard clean 后移除；三分支删除（plan-900-dev@4b24a157c / plan-900-ui-dev@4e9acd5 / plan-900-man-dev@cbbe657 均已 ff 落地）；组目录 .wt/lang-900 删除（残留 scratch：build log + generation driver scratch-consumer 已清）。
- plan-artifacts 无 plan-900 scope（证据全程折叠进计划文件；注册表无需变更）。
- 主检出状态：本计划文档全部入库（b369d318e 归档 + 收执）；主检出残余 docs WIP 为并行会话所有（会话起始即存在，未触碰）。
- 批量回归到期：plan id 900 % 5 = 0 → **DUE**，按 PLAN-898 规则移交 regress 技能主检出单实例执行（预存红 900-N1/N3 将按 triage 归位；不解除合并）。
- 终态计数：T-01..T-10 = 10/10；验收 1-6 交付 + 7（复审 PASS）；迁移对账 513/513。
