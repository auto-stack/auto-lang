---
plan_id: PLAN-635
status: execution_done          # drafting → executing → execution_done → reviewed → archived
feature_name: cross-package-style-recipes（Design 29 Phase 3 v2 + 依赖解析声明门控）
author: [zhaopuming]
created_at: 2026-09-15
updated_at: 2026-09-15
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/overview.md（style recipe 作用域描述「v1 单编译单元可见」——扩展为可 use 跨包导入）
new_spec_components:
  - docs/specs/auto-lang/ui/overview.md#style-recipe（跨包引用语义：use 符号导入 + pub 门控 + 撞名规则）
  - docs/specs/auto-man/project.md（依赖解析门控与 workspace 成员解析规则）
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（样式配方/令牌抽象）"]

affects: [auto-lang/parser, auto-lang/ui, auto-lang/aura, auto-man]
current_step: 8
total_steps: 8
---

# [PLAN-635] 跨包 style recipe 引用 + 依赖解析声明门控（pnpm 借鉴）

## 0. 变更摘要

兑现 [Plan 607](archive/607-autoui-style-recipe.md) 待澄清事项 #1 预留的 v2 方向：
style recipe 获得与 widget/fn/store 对等的**跨包引用能力**；同时按 pnpm/bun 调研
结论对依赖解析做**定向加固**（只做服务于本目标的子集，不做全量包管理器）。

1. **跨包 recipe 引用**：`use <dep>.<module>: <style_name>` 符号导入（零新语法，
   复用既有 use 命名/glob 导入管道），依赖包中 `pub style` 声明的 recipe 可在宿主
   widget 的 `style:` 属性中消费，编译期 desugar 单点展开，双端同源零运行时成本。
2. **声明门控（borrowed from pnpm 严格隔离）**：`resolve_module_path` 的 `deps/`
   探测增加放行条件——依赖名必须出现在祖先 pac.at 的 `dep` 声明或 workspace
   `members` 中；未声明包不再可达（防"幽灵依赖"，对齐 pnpm symlinked
   node_modules 的核心卖点）。
3. **workspace 成员一等解析（borrowed from pnpm workspace 协议）**：`scene:
   workspace` 工程的 members 目录可作为依赖被 use 引用，等价 `deps/<name>` 探测
   （examples/ui 的 `path: "../common"` 手声明形态可省）。
4. **pac.lock 补全（borrowed from bun 文本 lockfile 实践）**：本地 path/workspace
   依赖入锁（记录 resolved path），git 依赖 commit 锁既有。

**非目标**（调研后裁定不入，见 §4.3）：
- 中心化包 registry（延续 Plan 475 G1「不依赖远程索引库」）；
- pnpm 内容寻址全局存储（content-addressable store）——git worktree 快照 +
  pac.lock 已覆盖复现性，当前规模磁盘收益不成比例；
- pnpm catalogs（workspace 版本目录）——版本中心化需求未出现；
- peerDependencies / bun trustedDependencies 协议——`.at` 依赖包不执行脚本、
  主题 token 经封闭词表已天然满足"样式 peer 依赖"语义；
- theme {} 块的跨包共享（Design 29 Phase 2 延伸，另行立项）。

## 1. 目标

1. **G1 语法零新增**：跨包 recipe 引用不引入新关键字/新语法形态——`use
   common.styles: pill, card_base` 与既有 `use common.header: ExampleHeader`
   同构；`use common.styles: *` glob 只导 `pub` recipe（复用既有 pub-glob 语义）。
2. **G2 可见性门控**：仅 `pub style` 声明可被跨包导入；非 pub 的命名导入报编译
   错误；glob 导入静默跳过非 pub。
3. **G3 撞名防御**：宿主本地 recipe 与导入 recipe 同名时报编译错误，诊断形态
   对齐既有 fn 双源撞名（"X is defined in both …"）。
4. **G4 双端同源**：VM/Iced 轨与 Vue 轨的跨包 recipe 展开走同一注册与 desugar
   路径，产物 class 串逐字节一致（编译期单点，不引入运行时差异面）。
5. **G5 声明门控**：未在 pac.at 声明（且非 workspace member）的 `deps/` 包不可
   被 use 解析，错误信息指明需要在 pac.at 声明。
6. **G6 示例实证**：`examples/ui/` 下以两包形态实证跨包 recipe 消费，
   autoui-verifier 双端视觉对拍通过。

## 2. 架构方案

```
pac.at: dep "common" { path: "../common" }          workspace: members: ["common", "010-contact-form"]
        │  Plan 475 既有物化（junction/symlink → deps/common）    │  本计划新增 members 解析
        ▼                                                        ▼
宿主 app.at:  use common.styles: pill, card_base
        │
        ▼
collect_module_imports（lib.rs）──新增 StyleRecipeDecl 收集臂──▶ 合并 stmts（pub 门控 + 撞名检查）
        │
        ▼
load_and_validate_style_recipes（注册时机统一：合并后单次注册进 thread-local REGISTRY）
        │
        ▼
desugar_style_expr（aura/extract.rs:1016 既有单点）──▶ 展开 class 串/动态表达式
        ├──▶ Vue 转译（SFC class 串）
        └──▶ VM/Iced（StyleClass IR）        双端视觉全等（Plan 607 既有承诺延伸到跨包）
```

- **展开时机不变**：仍在 AST → AuraNode 归一化单点 desugar，本计划只把
  「依赖包的 recipe 声明」送进注册表，不触碰展开引擎本体。
- **Vue 轨接线**：auto-man 编译宿主模块前预加载依赖包 styles 模块并注册（调用
  与 VM 轨同一个 `register_style_recipe` 入口，防双端注册路径分叉）。
- **门控位置**：`resolve_module_path` 的 `probe_dep` 放行条件扩展；pac.at 声明
   读取沿用既有文本扫描（auto-lang 不得依赖 auto-man crate，依赖方向是反向的）。

## 3. 技术栈

- Rust：`crates/auto-lang/src/lib.rs`（collect_module_imports /
  resolve_module_path / 注册时机）、`crates/auto-lang/src/design_tokens/recipe.rs`
  （pub 门控/撞名检查）、`crates/auto-lang/src/parser.rs`（若 use 项解析需扩展
  样式符号类别）、`crates/auto-man/src/vue.rs`（宿主编译前注册）、
  `crates/auto-man/src/lock.rs`（path 依赖入锁）；
- 测试：`cargo t`（单元）、`cargo tv`（VM 语料 golden）、`autoui-verifier`
  （双端截图对拍）。

## 4. 需求分析与背景调查

（2026-09-15 会话实勘，均带 file:line 锚点）

### 4.1 语言面现状（缺口定位）

- **注册面只扫单编译单元**：`load_and_validate_style_recipes(&ast.stmts)` 三处
  调用（lib.rs:3609、5832、5949）均只收当前文件自己的 stmts——被 use 导入模块
  的 recipe 不可见。
- **use 合并器显式丢弃 recipe**：`collect_module_imports`（lib.rs:3298）的 stmt
  合并 match 只收 Fn/TypeDecl/EnumDecl/Ext/Use/Store/StoreDecl，StyleRecipeDecl
  落入 `_ => {}` 兜底臂（lib.rs:3400-3440）。
- **use 语法面已完备**：命名导入 `use db: a, b`、glob `use db: *`（只导 pub）、
  双源撞名诊断（"X is defined in both db and helpers — disambiguate with
  use db: X"，docs/design/10-language-syntax.md:189-204）——recipe 符号可完整
  蹭用，零新语法。
- **pub 已就绪**：`pub style brand_btn = "…"` 已可解析（parser.rs:20755 测试内
  证）；`is_pub` 字段在 `StyleRecipeDecl`（ast/ui.rs:27）与 `StyleRecipe`
  （recipe.rs:24）均已存在——导出面只缺消费端门控。
- **部分能力边界澄清**：依赖包内部 recipe 编译其自身组件时已生效（desugar 产物
  烤进组件），本计划补的是「宿主按名引用第三方 recipe」。

### 4.2 依赖体系现状（Plan 475 交付面）

- pac.at `dep "common" { path: "../common" }` → junction/symlink 物化到
  `deps/<name>`（pac.rs，Plan 475）；git `version:` 时 worktree 快照。
- `resolve_module_path`（lib.rs:2729）`deps/` 探测：向上遍历 4 层找 deps 目录
  （lib.rs:2762 起），**无声明门控**——手工放进 deps/ 的未声明包同样可解析
  （pnpm 恰以根治此问题著称）；另有 pac.at 文本扫描 local path dep 的既有逻辑
  （lib.rs:2845 起，文本 hack 形态保留）。
- Vue 轨：`VueProject` 扫 `deps/*/src/front/` 转译组件进 `components/`、合并
  npm_deps/styles（vue.rs:2887、2969）——依赖包 widget 已可跨包消费，recipe
  注册链缺位。
- pac.lock：`LockEntry { name, version, url, commit, path }`（lock.rs:14）——
  git 依赖 commit 锁已有；本地 path 依赖入锁缺。
- workspace：`Scene::Workspace` + `members` 字段已存在（pac.rs:28-48、130+），
  但 members 不参与模块解析。

### 4.3 pnpm/bun 调研结论（2026-09-15，web 调研）

| 前端实践 | 内容 | 本仓取舍 |
|---|---|---|
| pnpm symlinked 严格布局 | 只有直接声明依赖可达，防幽灵依赖 | **采纳（G5）**：deps/ 探测加声明门控 |
| pnpm workspace: 协议 | monorepo 内部包一等引用 | **采纳（G6 关联）**：members 等价 deps/ |
| pnpm/bun lockfile | 确定性复现、文本可 diff | **部分采纳**：path/workspace 依赖入 pac.lock |
| pnpm content-addressable store | 全局按内容哈希去重 | 不采纳：worktree 快照+lock 已覆盖，规模不匹配 |
| pnpm catalogs | workspace 版本常量目录 | 不采纳：需求未出现 |
| bun trustedDependencies | 生命周期脚本信任白名单 | 不采纳：.at 依赖包无脚本执行面 |

（来源：pnpm.io/motivation、pnpm.io/symlinked-node-modules-structure、
pnpm.io/workspaces、pnpm.io/catalogs、bun.com/docs/pm/cli/install——详见计划
起草会话调研记录。）

### 4.4 授权记录

- 用户 2026-09-15 会话指示：「起草一个新的正式计划」+「借鉴 pnpm/bun 等前端组件
  框架的优点，调研考虑一下再做计划」。调研先行已完成（§4.3），授权范围为计划
  起草；执行授权待计划确认时给出。

## 5. 详细设计

**D1 use 导入形态（推荐项，待用户裁定——见待澄清#1）**：

```auto
// 依赖包 common 的 styles.at：
pub style card_base = "bg-card rounded-xl shadow-sm border border-border"
pub style pill(bg: str = "bg-primary", ...) = "{pad} {bg} {fg} rounded-full …"
style internal_only = "…"            // 非 pub：不可跨包导入

// 宿主 app.at：
use common.styles: pill, card_base   // 命名导入（与 widget/fn 同构）
button "New" { style: pill() }       // 消费面与本地 recipe 完全一致
```

- glob：`use common.styles: *` 只导 pub recipe（复用既有 pub-glob 语义）。
- 备选形态（若用户改选）：`use styles from common: pill` 新关键字——语法面新增
  概念，无额外能力收益，不推荐。

**D2 collect_module_imports 收集臂**：match 增加
`Stmt::StyleRecipeDecl(r)`，dedup key `__style_recipe:<module>.<name>`；
跨包导入的 recipe 在合并时做两类检查——非 pub 且被命名 use 引用 → 编译错误；
与宿主本地（或其他导入源）同名 → 编译错误（诊断对齐既有双源撞名形态）。

**D3 注册时机统一**：VM 轨在 collect 合并后、宿主
`load_and_validate_style_recipes` 扫描前，将导入的 StyleRecipeDecl 注册进
thread-local REGISTRY（导入 recipe 视为已验证，不重复跑循环检测的全量重扫）；
Vue 轨 auto-man 在编译宿主模块前解析 use、预注册依赖包 recipe——两端调用同一
`register_style_recipe` 入口。

**D4 声明门控**：`resolve_module_path` 的 `probe_dep` 放行条件 = `deps/<name>`
存在 **且**（<name> 出现在祖先 pac.at 的 `dep "<name>"` 声明中，或为 workspace
members 成员）。未命中返回 None，上层 use 解析错误信息附带「在 pac.at 声明
dep "<name>" 或加入 workspace members」提示。pac.at 读取沿用既有文本扫描扩展。
向后兼容爆炸半径由 T-01 探针先行量化（预期：examples 与 test 语料全部走声明
形态，零命中）。

**D5 workspace members 解析**：向上遍历遇 `scene: workspace` 的 pac.at 时，
`members/<name>` 目录与 `deps/<name>` 同权探测（候选路径序列复用 probe_dep）。

**D6 pac.lock 补全**：`LockEntry` 对 path/workspace 依赖记录 resolved path 与
可选 version；`verify()` 对 path 依赖校验存在性。轻量，不引入新文件格式
（既有 TOML 文本锁，保持可 diff——bun 从二进制锁回退文本锁的教训吸收）。

### 规范增量

| delta_id | op | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md#style-recipe | before：recipe 单编译单元可见（Plan 607 v1）；after：可经 use 符号导入跨包消费，pub 门控 + 撞名规则 + desugar 单点不变 | 跨包引用语义成文 | AC-01..04 |
| SD-02 | modify | docs/design/10-language-syntax.md | 补 use 导入样式符号示例（命名/glob/pub 门控） | 语言参考更新 | AC-01 |
| SD-03 | add | docs/specs/auto-man/project.md | 新增依赖解析门控与 workspace members 规则（未声明不可达） | pnpm 严格隔离借鉴成文 | AC-05 |
| SD-04 | modify | docs/plans/KNOWN-DEBT-AND-RISKS.md | 登记 Plan 607 待澄清#1 兑现与本计划遗留（若有） | 台账同步 | AC-06 |

## 6. 测试设计

1. **语言面单测**（`cargo t style_recipe`）：跨包 fixture（common 包定义
   pub/private recipe + 宿主 use 导入）——命名导入可达、glob 只导 pub、非 pub
   命名导入报错、撞名报错、循环引用跨包形态报错。
2. **desugar 测试**：跨包 recipe 在宿主 `style:` 消费面展开正确（含参数化与
   组合配方跨包派生）。
3. **门控测试**：未声明 dep 的 `deps/<name>` use 解析拒绝 + 错误信息断言；
   声明后放行（正反例）。
4. **Vue 轨单测**（`cargo t -p auto-man`）：宿主 SFC 产物含展开后 class 串
   （golden 断言）；依赖包 styles 模块注册链单测。
5. **双端集成**：examples 两包形态（010-contact-form 扩展或新增 mini 示例），
   `autoui-verifier` 双端对拍零漂移。
6. **门禁**：`cargo t`、`cargo tv`、fold 前 `cargo tf`。

## 7. 验收标准

- **AC-01 语法完备**：`use <dep>.<module>: <style_name>` 命名导入与 `*` glob
  在 `.at` 中解析无歧义，与既有 use 管道行为一致（含 `use super/pac` 寻址），
  语法单测全绿。
- **AC-02 可见性门控**：仅 `pub style` 可跨包导入；非 pub 命名导入编译错误，
  glob 静默跳过；负例用例全绿。
- **AC-03 撞名与循环防御**：宿主/导入源同名 recipe 报编译错误（诊断含双源
  名）；跨包循环引用检测拦截。
- **AC-04 双端一致**：跨包 recipe 消费在 Vue 与 VM 端展开逐字节一致（golden
  对拍）；运行时零新增开销（展开仍全量发生在编译期）。
- **AC-05 声明门控**：未声明包不可达（错误信息含修复指引）；声明/workspace
  member 形态放行；现有 examples/test 语料零回归。
- **AC-06 示例实证**：两包示例 autoui-verifier 双端视觉对拍通过；KNOWN-DEBT
  台账同步。
- **AC-07 门禁通过**：`cargo t` 零新增红（对拍 master 基线红名单）；`cargo tv`
  全绿；fold 前 `cargo tf`。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-01 探针与决策工件（零代码）**：
  - 量化 D4 门控爆炸半径：`grep -rn "deps/" examples/ test/` + 实跑现有
    VM/Vue 示例确认全部走 pac.at 声明形态（预期零未声明命中，若有命中逐条
    定性回填本档）；
  - 定位 Vue 轨宿主编译链的注册挂点（vue.rs 编译宿主 .at 的确切调用路径与
    auto-lang 编译 API 入口对应关系）；
  - 探明 use 符号导入的解析器侧符号类别接线点（样式符号是否需要 parse_use_items
    之外的扩展）。
  - 验证：探针结论回填本档（含 file:line 锚点）。
  - [✅ 已完成 2026-09-15] 三探针结论（master 检出实勘）：
    **P1 门控爆炸半径=受控文件零破坏**。已声明 3 例（006/010/016 pac.at
    `dep settings { path: "../common/settings" }`——裸名形态，非 `dep "settings"`
    引号形态，grep 模式须两者都扫）；未声明 `deps/` junction 共 4 例
    （011/015/019/ui-gallery）**全部为失效残留**：011 的 `use prog_util` 本地
    src/front/prog_util.at 直探命中、015 无任何跨包 use、019 有本地
    src/front/settings.at（直探优先于 deps/）、ui-gallery 源码无 use 语句；
    其余 `use settings` 消费面（book-reader/k1/039）均为本地 pages/settings.at、
    settings_card.at 模块而非依赖。硬门控落地后残留 junction 变惰性，无需删除
    （未受控本地状态不触碰）。
    **P2 Vue 生产链挂点=auto-man 零改动**。vue.rs:2494 每_widget 调
    `auto_lang::ui_build_shadcn_with_widgets(path, None)`（含 deps/* 的 widget
    文件同路），链条 `ui_build_shadcn_with_widgets(6116) → _and_stores(6139) →
    ui_build_shadcn(5922，含 5949 注册点)`；VM 轨 `build_dynamic_component_inner
    (3583，含 3609 注册点)`；`ui_build(5796，含 5832 注册点)` 为通用兜底。三处
    注册点同调 `load_and_validate_style_recipes(&ast.stmts)`——**跨包注册改造
    收敛为该单函数**（内部 clear_style_recipes 先清后注，recipe.rs:116——导入
    注册必须整合进同一函数避免被清）。D3 的「auto-man 预注册」不再需要，
    T-04 相应收缩为「三入口接线 + Vue 轨回归」。
    **P3 use 符号接线=parser 零改动**。`parse_use_items`（parser.rs:6523）产
    纯名字列表，符号类别无关；recipe 按名蹭用即可。命名导入非 pub 检查在
    收集依赖模块 stmts 时与 use items 求交实现；撞名检测放注册层（需给
    StyleRecipe 加来源标记）。
- **T-02 语言面收集与门控**（执行形态依 T-01 P2/P3 收敛，未动 collect_module_imports）：
  - `recipe.rs` 新增 `prepare_style_recipe_imports`（源码扫描→模块解析→传递
    收集，两阶段防 parser 实时注册污染）+ `register_style_recipe_checked`
  （source 追踪撞名检测）；pub 门控/非 pub 命名导入硬错误；
  - 四注册点接线：build_dynamic_component_inner（VM）/ui_build/
    ui_build_shadcn/ui_gen::api::generate_component_from_file（Vue 生产链）。
  - [✅ 已完成 2026-09-15] commit f34531641；plan635 测试 7/7（命名/glob/
    非pub/撞名/传递/deps布局/vue链e2e）；design_tokens 21/21。
  - 执行偏差记录：D2 原文写 collect_module_imports 收集臂——T-01 实勘表明
    UI 提取三入口不走该合并器，正确挂点为 load_and_validate 单函数族；
    D3「auto-man 预注册」收缩为 auto-lang 内单点（auto-man 零改动），
    契约目标（双端同一注册路径）不变。
- **T-03 desugar 注册接线**（并入 T-02 实现——注册与展开同函数族单点）：
  - [✅ 已完成 2026-09-15] commit f34531641；参数化/组合跨包展开用例
    （test_plan635_named_import_registers_and_desugars /
    test_plan635_transitive_import）绿；cargo tv 见 T-08。
- **T-04 Vue 轨注册**（T-01 P2 收敛：接线在 auto-lang api.rs 入口，auto-man 无改动）：
  - [✅ 已完成 2026-09-15] commit f34531641；test_plan635_vue_chain_expands_
    imported_recipe 走 ui_build_shadcn 生产链断言 SFC 含展开串；auto-man
    回归（lock 套件 8/8，见 T-07）。
- **T-05 声明门控与诊断（硬门控，2026-09-15 用户裁定）**：
  - `lib.rs` resolve_module_path walk-up：deps/<name> 探测过 pac.at `dep`
    声明门（裸名/引号双形态 + 词边界检查）；未声明幽灵依赖阻断 + 修复指引
    eprintln；旧 probe_dep 闭包收敛为统一 probe_pkg 候选助手。
  - [✅ 已完成 2026-09-15] commit df00f06d4；test_plan635_gate_blocks_
    undeclared_deps（负例）+ plan475 fixture 补声明（语义变更预期内）；
    plan339 5/5 + plan475 1/1 绿。
- **T-06 workspace members 解析**：
  - pac_workspace_member_dir 文本扫描（members: [...] 表项末段匹配依赖名）
    + members 目录同权 deps/ 探测。
  - [✅ 已完成 2026-09-15] commit df00f06d4；test_plan635_workspace_member_
    resolves 绿。
- **T-07 示例实证 + pac.lock 补全**：
  - 新增受控跟踪两包示例：examples/ui/stylekit（共享包，pub card_base/
    参数化 pill/非 pub internal_only）+ examples/ui/045-style-import
  （pac.at `dep stylekit { path: "../stylekit" }` + 命名 use 导入 + 宿主
    pill_danger 派生）。010 扩展弃用——T-01 实勘其依赖 examples/ui/common
    为未跟踪本地目录（junction 悬空，settings 已迁址 auto-os），不可作
    受控实证载体；
  - D6：lock.rs from_target 收录本地 path 依赖（git commit 可选）+ verify()
    对 commit-less 条目校验物化路径存在；
  - [✅ 已完成 2026-09-15] commit 28c6e718e；auto-man lock 8/8；
    test_plan635_example_fixture_vue_chain（真实示例生产链）绿。
  - **双端实机实证**（AC-06 证据）：VM 轨 `auto run -r vm` + AUTOUI_MCP_PORT
    ——结构快照三配方全展开（card_base 卡片串/pill() 主按钮串/pill_danger
    派生串逐类吻合）+ iced 帧截图；Vue 轨 `auto run` 脚手架 + vite 就绪 +
    Playwright dark 1280x800 截图；截图已留档（会话交付展示）。注：
    parity_shot_diff.py 采样点为 015-notes 专用，本示例为新增布局，
    未注册采样点（结构对拍+截图代偿），评审知悉。
- **T-08 门禁与收口**：
  - [✅ 已完成 2026-09-15] `cargo t --no-fail-fast` 全量红名单对拍（同
    worktree 同环境 master tip d96973a80 detached 对照）：分支 25 unique 红
    ⊆ master 26 unique 红，**零新增回归**（消失 1 红 plan394::c1_future_all
    为 master 侧 flaky，非本分支触碰面）；`cargo tv` 3725/3725 全绿
    （35.3s）；plan635 专属 10/10 + plan339 5/5 + plan475 1/1 +
    auto-man lock 8/8；警告面：新改文件零新增（cargo check 181=基线，
    Name 未用导入顺修）；无遗留 debug 输出（唯一 eprintln 为幽灵依赖
    硬门控诊断通道，与既有 collect_module_imports 诊断形态一致）。
    代码 commits：f34531641 / df00f06d4 / 28c6e718e / 683b842d7
    （worktree D:/autostack/.wt/lang-635/auto-lang，branch plan-635-dev，
    base d96973a80）；`cargo tf` 按 T-08 契约移交 /auto-plan:review 执行。

## 9. 复审记录

- （draft 起草 handoff 2026-09-15：`stage: new | PLAN-635 | plan_revision: 1 |
  outcome: pass | next: work`——D1 推荐形态待用户确认时若改选备选，
  plan_revision 递增为 2 并调整 T-02。）
- （用户确认 2026-09-15：待澄清 #1/#2/#3 全部按推荐方案裁定——use 符号导入
  复用、硬门控、D6 入 scope；契约文本与 D1/D4/D6 原书写一致，
  plan_revision 维持 1，授权进入 work。）
- （execution_done handoff 2026-09-15：`stage: work | PLAN-635 |
  plan_revision: 1 | outcome: pass | code_commit: 683b842d7 (tip,
  f34531641+df00f06d4+28c6e718e+683b842d7) | task_ids: T-01..T-08 全勾 |
  evidence: plan635 10/10 + tv 3725/3725 + 红名单对拍分支25⊆master26零新增
  + 双端实机快照/截图（VM MCP snapshot 三配方全展开 / Vue vite+Playwright
  dark） | blockers: 无 | next: review`）

## 10. 待澄清事项

1. **use 导入形态裁定**：✅ **已裁定（2026-09-15 用户）**——复用既有符号导入
   `use common.styles: pill`（与 widget/fn/store 同构，零新语法）；备选
   `use styles from common: pill` 否决。T-02 按本档 D1 书写执行。
2. **门控严格度**：✅ **已裁定（2026-09-15 用户）**——硬门控：未声明 =
   编译错误（D4 按此执行）；T-01 探针若发现存量未声明使用面，逐条修复
   （不降级为警告）。
3. **D6（pac.lock 补全）**：✅ **已裁定（2026-09-15 用户）**——保留在本计划
   scope 内（T-07 一并落地）。
