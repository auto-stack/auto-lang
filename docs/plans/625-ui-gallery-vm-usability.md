---
plan_id: PLAN-625
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: ui-gallery-vm-usability
author: [agent]
created_at: 2026-09-14
updated_at: 2026-09-14
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/auto-lang/ui/overview.md#573-预存限制注记（VM 列表空留待立项）]
new_spec_components: [docs/specs/auto-lang/ui/overview.md#ui-gallery-registry-双产物与-vm-可用性]
touched_goals: [GOAL-010, GOAL-007]   # 引用 docs/specs/goals.md 的 GOAL-NNN（沿 573 引用，暂定）

affects: [auto-lang/ui, parity]
current_step: 4
total_steps: 9
---

# [PLAN-625] ui-gallery-vm-usability

## 0. 变更摘要

ui-gallery（示例画廊，源在 auto-os 仓顶层）VM 模式当前"窗口能开但空壳"：侧栏
demo 列表空、右栏标题/描述/教程/源码全空、内嵌视口空占位、分类 pills 不上屏。
本计划把 2026-09-14 会话实证的全部问题统一收敛为一个改善合同：

1. **数据源跨端化**（根因，清偿 KNOWN-DEBT P573-D1 与 573 待澄清①②）：
   `auto-man generate_gallery_host` 扩展为双产物——TS registry（web 臂，含
   `load()` 动态导入）+ `.at` 静态 registry 数据模块（VM 臂可执行），app.at
   按端消费；
2. **VM 渲染侧缺陷归因与修复**：分类 pills 零像素上屏（归因→修复或登记）、
   `bg-clip-text` 渐变标题渲染为不可读色块（降级 fallback）、for 循环变量
   onclick handler 合成失败（VM codegen）；
3. **AppViewport VM 形态决策**：有界调研后裁定 v1 形态（进程内子应用渲染 vs
   元数据卡 + web-only 提示），落地；
4. **进程稳定性**：VM 实例异常退出码（127/1，无 panic）有界调查，结论入账。

**定位**：本计划是 ui-gallery 后续一切修改的**常驻跟踪载体**——后续会话新发现
的 ui-gallery 问题一律以 plan_revision 修订进入本计划（新增 T/AC 条目），不再
另立零散计划；语义修订遵循 auto-plan-new §Revisions 授权规则。

## 1. 目标

- **G-1 VM 可浏览**：VM 端完成画廊浏览主链路——侧栏列表渲染全部条目、分类
  过滤与搜索过滤生效、条目点击切换选中、右栏标题/描述/教程/源码/工程配置
  五类内容非空。
- **G-2 数据单一来源**：registry 以 examples/ui 语料为唯一来源，由
  `generate_gallery_host` 同时产出 TS 与 `.at` 双产物，消除双端数据漂移。
- **G-3 渲染缺陷收口**：pills、渐变文字、循环 handler 三个 VM 渲染/codegen
  缺陷全部归因（修复，或有证据地登记为既定降级/债项）。
- **G-4 AppViewport 有形态**：VM 端内嵌视口有经用户裁定的形态决策与 v1 落地。
- **非目标**：像素级双端 parity（沿 parity/PLAN-619 线）；backdrop-\* 装饰降级
  翻转（P518 白名单）；VM 端真实 iframe 网页内嵌；AutoDown 教程渲染器跨端
  （VM 端教程 tab 只求纯文本/基础 markdown 内容可见）。
- **受影响仓库**：auto-lang（`crates/auto-man` 生成器、`crates/auto-lang` iced
  renderer 与 VM codegen）+ auto-os（`ui-gallery/` 应用源）。
- **成功判据**：G-1 全链路 MCP 快照实证通过；启动日志 registry 相关 ext stub
  警告清零；Vue 臂零回归（构建 + 既有测试 + 金样流程）。

## 2. 架构方案

- **数据流（G-2）**：
  `examples/ui 语料`（唯一来源）→ `auto-man generate_gallery_host`
  （`crates/auto-man/src/vue.rs:4045`，现仅产 TS）→ **双产物**：
  - `src/demos-registry.ts`（web 臂，现状不动：元数据 + `load` 动态导入）；
  - `src/front/registry.at`（新增，VM 臂：纯数据——id/title/category/icon/
    description/tags/loadable/doc/source/pac，**不含** `load`）。
  `app.at` 的 `use` 块引入 `.at` registry，`computed`（`filteredDemos`/
  `currentTitle` 等 6 项）改读 `.at` fn；TS `demos.ts` 8 fn 保留仅供 web 臂
  消费路径（AppViewport.vue 仍需 TS `load`）。`.at` 数据模块语法与 app 消费
  惯用法由 T-01 有界调研钉死（决策 artifact）后再实现。
- **渲染修复（G-3）**：全部落在 auto-lang iced/VM 臂——pills 归因走 A/B 复跑
  （干净基线二进制 vs 含他会话脏改动的现役二进制）+ 最小 sidebar 复现；
  `bg-clip-text text-transparent` 走降级 fallback（可读前景色）；for 循环变量
  的 onclick handler 合成走 codegen 修复 + 单测。
- **AppViewport（G-4）**：T-08 有界调研三候选——(a) 进程内子应用渲染（loadable
  示例的 .at 源已在 registry 表内，VM 具备编译渲染能力）；(b) 元数据卡 +
  "web 专属"提示；(c) 静态截图栅格。产出决策 artifact（工作量/风险/收益），
  用户裁定后 T-09 落地 v1。
- **执行布局**：跨仓计划 → 分组平铺 worktree
  `D:/autostack/.wt/lang-625/{auto-lang, auto-os}`（Plan 529 布局；创建前先在
  master 提交本计划与 .next-id，移除前跑 wt-guard）。跨仓依赖解析序遵守
  AGENTS.md 红线（env → 组内兄弟 → 主检出），worktree 内禁建 junction/symlink。

## 3. 技术栈

- Rust：auto-man 生成器（`crates/auto-man/src/vue.rs`）、iced renderer
  （`crates/auto-lang/src/ui/iced/renderer.rs`）、VM codegen（handler 合成路径）。
- AutoLang `.at`：registry 数据模块、app.at 改造。
- Vue/TS：web 臂保持现状（demos-registry.ts / AppViewport.vue 不迁移）。
- 验证工具：autoui-verifier 技能的 Python MCP 驱动（`test_vm_mcp.py`）做 VM
  快照/交互/截图实证；Playwright 做 Vue 回归对拍；`cargo t <module>` 做 scoped
  门禁；ui-gen 金样漂移走 `AUTO_LANG_UPDATE_GOLDEN` 流程。

## 4. 需求分析与背景调查

### 4.1 授权记录

- **授权范围**（用户 2026-09-14 会话）：综合本会话发现的全部 ui-gallery 问题
  统一立项；此后 ui-gallery 的一切修改继续以本计划文件跟踪。仓库范围
  auto-lang + auto-os。无额外预算或自动继续限制输入（未授权即无预算）。

### 4.2 发现清单（本会话实证，2026-09-14）

| ID | 现象 | 根因 | 去向 |
|---|---|---|---|
| F-1 | 侧栏列表空、右栏标题/描述/教程/源码/配置全空（仅 `002-counter` 徽章来自 model 默认值） | registry 数据源 8 个 TS extern fn（`src/front/utils/demos.ts`）VM 无从执行，全部绑成 no-op stub；`filteredDemos` 恒空 | T-01/T-02/T-03（P573-D1） |
| F-2 | 内嵌视口空（AURA 树 `[Image]` 占位） | `AppViewport` 为 Vue SFC，VM 无实现 → no-op stub | T-08/T-09 |
| F-3 | 分类 pills（全部/基础/组件/应用/系统）在 AURA 树存在但零像素上屏（2x 放大截图取证） | 未归因；干扰源=现役二进制含他会话未提交 iced renderer 脏改动 | T-04 |
| F-4 | 顶栏标题 "AutoUI Gallery" 渲染为不可读蓝紫色块 | `bg-gradient-to-r bg-clip-text text-transparent` 无 iced 臂降级策略，透明文字叠渐变盒 | T-05 |
| F-5 | 启动日志 `handler synthesis failed: App..__evt_onclick_6: Undefined variable: demo`，poisoned export 被丢弃 | for 循环体内 onclick 引用循环变量 `demo`，VM handler 合成不解析循环变量 | T-06 |
| F-6 | 两次 VM 实例异常退出码（127、1），日志无 panic | 未归因（正常关窗应为 0；不能排除用户手动关窗） | T-07 |

### 4.3 证据与版本

- 启动日志 ext stub 警告清单（`AppViewport`/`getAllDemos`/`filterDemosBy`/
  `getDemoDesc`/`getDemoDoc`/`getDemoPac`/`getDemoSource`/`getDemoTitle`/
  `isDemoLoadable` 共 9 条）+ `[VM-HANDLER] App.Init failed: handler not found:
  Init`（伴生噪音，model 默认值仍正常填充）。
- AURA 快照 v2（会话内抓取）：pills row + 空 list col 同屏；对照
  `docs/reports/p573-master-baseline/snapshot_uigallery_vm_home.txt`（573 基线
  同构，仅文本无 PNG）。
- 已知债登记：`docs/plans/KNOWN-DEBT-AND-RISKS.md` P573-D1（低）；
  `docs/plans/archive/573-uigallery-sidebar-migration.md` 待澄清①②。
- Specs 现状：`docs/specs/auto-lang/ui/overview.md:40`（549 画廊架构段——
  "auto-man generate_gallery_host + demos-registry 动态装配"、43 示例）、
  `:545`（573 段——"VM 列表空为预存限制……跨端化留待后续立项"，即本计划）。
- 生成器锚点：`crates/auto-man/src/vue.rs:4037`（`is_ui_gallery`）、`:4045`
  （`generate_gallery_host`）、`:4221`（materialize AppViewport.vue）；
  `crates/auto/src/main.rs` 有 ui-gallery/desktop-host 特判。
- registry 形态：`D:/autostack/auto-os/ui-gallery/src/demos-registry.ts` 为
  bootstrap 占位（头部注释：`auto run` 时被 auto-man 生成器从 examples/ui
  全量覆写），`DemoMeta` 字段 id/title/category/icon/description/tags/doc/
  source/pac/loadable/load?。
- 版本基线：`auto 0.1.0+v0.4.2-551-ge0c404f57-dirty`（target/debug，2026-09-14
  14:29 构建，**含他会话未提交 iced renderer/terminal 修改**——F-3 归因干扰源，
  T-04 处置）。

## 5. 详细设计

### 5.1 T-01 生成器双产物（G-2 核心）

`generate_gallery_host` 在现有 TS 装配旁新增 `.at` 产物发射：

- **有界调研结论（2026-09-14，决策 artifact）**——`registry.at` 形态钉死：
  - **模块形态**：`src/front/registry.at` 独立纯函数模块，app.at 以
    `use registry: <fns>` 消费（tree_util.at PLAN-522/614 先例，双端同源：
    VM import_aliases / vue 臂 SFC 转译）。`.at` ext 源在 VM 走 port-adapter
    链真实编译（ext_stubs.rs Plan 442 A3：`X.at`→`X.vm.at`→`X.web.at`），
    仅 TS/npm 源才合 no-op stub——设计成立的机制根据。
  - **数据**：`all_demos() List` 返回记录列表；记录=匿名 Obj 字面量**全键
    书写**（形状锁定，缺键=硬错）：id/title/category/icon/description/tags/
    doc/source/pac/loadable + 生成期预计算 `search_lc`（title+id+tags 拼接
    小写，免运行时 lower 堆料）。tags=str 列表字段（tree_util children: List
    先例）。
  - **过滤/查询**：`filter_demos(query, category) List`——**while+索引遍历**
    （P614 纪律：for-in 对参数列表 VM 零迭代）；匹配=`q == "" ||
    d.search_lc.contains(q)`（`.contains` VM 实证，tree_util:209）+ category
    相等；查询侧 `query.to_lower()`（VM 内建 str 方法，engine.rs:7068）。
  - **getters**：demo_title/demo_desc/demo_doc/demo_source/demo_pac/
    demo_loadable(id)——while 扫描 + 回退语义对齐 TS findDemo（title 回退
    id，其余回退 ""/false）。
  - **转义集**（lexer.rs `str()`）：`\n` `\t` `\r` `\0` `\\` `\"`；发射器
    对 doc/source/pac 做 `\`→`\\`、`"`→`\"`、LF→`\n`、CR→`\r`、TAB→`\t`。
  - **挂接点**：vue 臂 `generate_gallery_host`（demo_rows 聚合后追加发射，
    与 demos-registry.ts 同点）；**VM 臂 `run_vm_ui`（rust_ui.rs）entry 检查
    前新增刷新 hook**（现状 VM 运行路径不触发 gallery 生成——实证：首轮启动
    日志无 "Gallery demos" 行；vue 臂每次 run 刷新先例 vue.rs:5245）。
    行构建逻辑抽 `gallery_demo_row` 单一来源供两臂共用（消 loadable/category/
    tags 漂移）。
- 实现发射器 + golden 测试（数量 + 首末条字段 + 转义正确性）；
- 兼容：`crates/auto/src/main.rs` 的 ui-gallery 特判路径行为不变。

### 5.2 T-02 app.at 按端消费

- `use` 块引入 registry `.at` fn；`computed` 六项（filteredDemos/currentTitle/
  currentDesc/currentDoc/currentSource/currentPac + currentLoadable）改读
  `.at` 表；
- TS 8 fn 从 VM 消费路径摘除（保留 web 臂 AppViewport 所需的 `load` 装配），
  目标=启动日志 F-1 的 8 条 stub 警告清零；
- `for demo in .filteredDemos` 循环结构不变（配合 T-06 使 onclick 可合成）。

### 5.3 T-04 pills 归因（先归因后修复）

A/B 矩阵：{现役 dirty 二进制, 干净基线二进制} × {ui-gallery 全量, 最小
sidebar 复现样例}。判读：(a) 干净基线亦不上屏 → 预存缺陷，转修复 + sidebar
golden 扩展 pills case；(b) 仅 dirty 二进制不复现 → 他会话在途改动引入，登记
台账并待其收口后复核；(c) 均上屏 → 会话环境误判，关账留证。修复方向候选：
sidebar_header 合成路径样式/布局钳制（`convert_sidebar_region`/
`HEADER_BASE` 契约）。

### 5.4 T-05 渐变文字降级

`bg-clip-text text-transparent`（含 `bg-gradient-to-*`）在 iced 臂降级为可读
形态：文字用 foreground/primary 实色，放弃渐变背景盒。降级策略登记入 parity
spec 降级矩阵（SD-02），对齐 Plan 412 §5 惯例（声明式降级、不报错不 not-yet）。

### 5.5 T-06 循环变量 handler 合成

VM codegen 对 `for` 循环体内 onclick 的循环变量捕获：合成 handler 时将循环
变量作为形参注入（或按 item 闭包绑定），使 `__evt_onclick_N` 导出可通过
FN_PROLOG 校验、不再 poisoned。单测：循环体 handler 编译 + 导出存在 + VM 执行
变量取值正确。此为 AC-02（条目点击）的前置。

> **revision 2 实现裁定（2026-09-14）**：执行期证据表明仓内已有该场景的
> 成熟惯用法——循环体事件用 msg 带参形式（`onclick: .SelectDemo(demo.id)`，
> 027 `OpenItem(item.id)` 同型），循环变量由渲染器在分发期对每 item 求值后
> 作为 handler 参数传入，与"捕获"语义等价且零编译器风险。T-06 按此落地
> （app.at 三处编辑）；编译器侧 lambda 捕获循环变量作为**通用能力缺口**
> 登记限制（merge 时入 KNOWN-DEBT），本计划不扩权实现。goal 与 AC-02 不变。

### 5.6 T-08/T-09 AppViewport 形态

决策 artifact 对比三候选（§2）：渲染保真度、工作量、风险（进程内编译栈深度
——注意 AGENTS.md 双重解释器路径裁定：重型路径避免 run_with_capture 形态）。
用户裁定后 v1 落地；若裁定触发目标/验收语义变化，按 Revisions 规则先修订本
合同再执行。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/overview.md | before：registry 全系 Vue-only TS extern fn，VM 端列表/标题空为预存限制（573 注记"留待后续立项"）；after：generate_gallery_host 双产物（TS 含 load 供 web / .at 纯数据供 VM），app.at 按端消费，VM 浏览链路可用 | 清偿 P573-D1 与 573 待澄清① | AC-01, AC-02, AC-03, AC-04 |
| SD-02 | modify | docs/specs/parity/project.md | before：`bg-clip-text` 渐变裁剪文字无降级约定（渲染为透明文字+色块）；after：iced 臂声明式降级为实色可读文字（登记降级矩阵，对齐 Plan 412 §5 形态） | 消除不可读渲染 | AC-06 |
| SD-03 | modify | docs/specs/auto-lang/ui/overview.md（573 段注记） | before：menu_button for 内 active/onclick 的 VM 实证缺对象（待澄清② open）；after：实证补齐并关闭待澄清② | 数据源就绪后补证 | AC-02 |

## 6. 测试设计

- **生成器 golden**（T-01）：`cargo t gallery`（auto-man 测试模块滤串，命名以
  实际为准）——`.at` 产物条数=语料扫描数、首末条字段值、转义正确性。
- **VM 端到端实证**（T-03，autoui-verifier 工作流）：`AUTOUI_MCP_PORT=<动态>
  auto run -r vm` + `test_vm_mcp.py`——
  snapshot 断言侧栏条目数=registry 条数；press 首条目后 `selected_id` 与右栏
  标题/描述联动；三 tab 内容非空；pills 过滤（点击"组件"后条目数变化）与
  搜索过滤（type 后条目数变化）各一组快照；截图存档本地
  （`.gitignore` 排除，严禁入库）。
- **归因探针**（T-04）：最小 sidebar 复现样例（或扩展既有 sidebar golden）
  在 A/B 二进制下各截一帧。
- **单元**（T-05/T-06）：iced 渲染降级路径单测；循环 handler 合成单测（编译
  导出 + VM 执行取值）。
- **Vue 回归**（AC-07）：`auto run` 构建成功 + Playwright 对拍截图；ui-gen
  金样若漂移走 `AUTO_LANG_UPDATE_GOLDEN` 重生成并逐行审查 diff；
  scoped 门禁 `cargo t iced` + 生成器/codegen 相关模块滤串。
- **门禁纪律**：Category B（局部 Rust）——`cargo check -p auto-lang` +
  `cargo t <module>`；不触发 aavm 档（改动不涉 VM/编译器语料语义，若 T-06
  触及共享 codegen 路径再按 AAVM 触发表复裁）。

## 7. 验收标准

- **AC-01** VM 侧栏列表可用：`auto run -r vm` 启动后，MCP snapshot 侧栏条目数
  = generator registry 条数；点击分类 pill 与输入搜索词后条目数按数据正确
  变化。验证：test_vm_mcp.py snapshot 计数断言。
- **AC-02** 条目交互可用：press 任一条目后 `selected_id` 切换、右栏徽章/标题/
  描述更新（快照 diff + 截图）；573 待澄清②实证补齐并关闭。
- **AC-03** 右栏内容非空：教程/完整源码/工程配置三 tab 在 VM 端快照文本长度
  > 0（内容来自 `.at` registry 的 doc/source/pac 字段）。
- **AC-04** stub 警告清零：启动日志无 F-1 清单中的 8 条 registry ext stub
  警告（`AppViewport` stub 是否保留按 T-08 决策单独评估，不在本 AC 内）。
- **AC-05** pills 可见且有归因结论：VM 截图中分类 pills 可见（截图取证）；
  归因结论三选一落账（修复 commit / KNOWN-DEBT 登记 / 误判关账留证）。
- **AC-06** 渐变标题可读：VM 截图顶栏标题文字可读（实色 fallback）；
  parity spec 降级矩阵含该条目（SD-02）。
- **AC-07** Vue 零回归：web 臂构建成功、既有 vue/ui-gen 测试绿、金样漂移已按
  流程重生成且 diff 审查通过。
- **AC-08** 退出码结论入账：复现矩阵（X 关窗 / taskkill / MCP 调用后退）+
  归因写入 KNOWN-DEBT 或文档化关账（允许"确认为 wrapper 退出语义，文档化"结案）。
- **AC-09** AppViewport 有决策有形态：决策 artifact 经用户裁定；v1 按裁定落地
  （形态级验收随决策修订细化）。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成]
一行证据）

**执行布局（2026-09-14 work 起run 登记）**：auto-lang worktree
`D:/autostack/.wt/lang-625/auto-lang`（branch `plan-625-dev`，base
d2f983d63=计划提交）；auto-os 兄弟 worktree
`D:/autostack/.wt/lang-625/auto-os`（branch `plan-625-dev`，base
48ae6da）；auto-os 主检出另有他会话在途脏文件（apps/025-sys-monitor），
不在本计划范围、不触碰。跨仓解析靠组内兄弟 `../auto-lang` 命中 worktree
（禁 junction/symlink）。

- **T-01** 生成器双产物：`crates/auto-lang/crates/auto-man/src/vue.rs`
  （`generate_gallery_host` :4045）扩展 `.at` 发射 + golden 测试。先行有界调研
  （`.at` 数据表惯用法，决策 artifact 入本节附录）。验证：`cargo t gallery`。
  → AC-01/03/04（前置）。**无依赖，立即可执行。**
  [✅ 已完成] commit d046eab17（auto-lang worktree）。`write_registry_at` +
  `gallery_demo_row` 单一来源抽取 + `run_vm_ui` 刷新 hook（rust_ui.rs）+
  golden 测试 x3 全绿（`cargo t -p auto-man gallery_registry` 4/4，含存量
  plan_549 测试无回归）；真实语料端到端：VM run 发射 33 条（675KB）、
  `use registry:` 链接成功、启动日志 registry ext stub 8→0（仅余 AppViewport
  1 条，按 T-08 决策保留）。字段避 .at 关键字：`pac`→`pac_text`（"Expected
  key, got Pac" 实证修复）。
- **T-02** app.at 切换数据源：`D:/autostack/auto-os/ui-gallery/src/front/app.at`
  （use 块 + computed 六项）+ Vue 臂保留路径核验。验证：vue 构建通过 +
  VM 启动日志无 stub。依赖 T-01。→ AC-01/03/04。
  [✅ 已完成] commit 40014a7（auto-os worktree）。`use registry: filter_demos,
  demo_title, ...`（PLAN-522 形态，tree_util 先例）；computed 七项改读 .at 表；
  TS demos.ts 8 fn 从消费路径摘除。VM 实证：侧栏 33 项渲染（20 可交互+13 独立）、
  currentTitle/Desc/Doc/Source/Pac 全部非空。vue 臂构建回归待 review 门禁补跑。
- **T-03** VM 浏览链路实证：MCP 快照/交互/截图全套（§6 清单）；回写 573 待澄
  清②实证引用。依赖 T-02（点击依赖 T-06）。→ AC-01/02/03。
  [✅ 已完成] 2026-09-14 会话内实证（MCP 动态端口，截图/快照存
  tests/screenshots/p625_*，gitignore 排除不入库）：列表 33 项=发射数 ✓；
  pill 过滤 33→7（组件）✓；搜索框输入 "chart" 过滤 ✓；条目点击切换
  selected_id（详情徽章 002-counter→001-helloworld、标题/教程联动、
  window title 更新）✓ —— **573 待澄清②（menu_button for 内 active/onclick
  VM 实证）随之关闭**；源码 tab（pre 渲染 currentSource）✓；工程配置 tab
  （pac 内容）✓；教程 tab markdown 降级渲染可见（autodown 队列臂有基础
  文本形态输出）✓。截图 `p625_vm_t03_final.png` 为切换后证据。
- **T-04** pills 归因：A/B 二进制矩阵 + 最小复现；结论落账（修复或 KNOWN-DEBT
  行）。**无依赖，可并行。** → AC-05。
  [进展] 归因 (b) 臂已排除：worktree 干净构建（无他会话脏改动）复现同现象
  ——pills+列表在 AURA 树存在（33 项）但零像素绘制 → **预存渲染缺陷实锤**，
  转 (a) 修复路径（sidebar_provider/header 合成绘制链，T-04 余下部分）。
- **T-05** 渐变文字降级：`crates/auto-lang/src/ui/iced/renderer.rs` 文字渲染
  样式解析路径 + 单测 + parity spec 矩阵行。**无依赖，可并行。** → AC-06、SD-02。
- **T-06** 循环 handler 合成修复：VM codegen handler 合成路径（定位
  `__evt_onclick_N` 合成与 `Undefined variable` 报错点）+ 单测。**无依赖，
  可并行；T-03 点击实证的前置。** → AC-02。
  [✅ 已完成] 实现路径按证据调整为 app 端惯用法对齐（revision 2，§5.5）：
  commit 38d7bd7（auto-os worktree）——循环体 onclick 改 `SelectDemo(demo.id)`
  msg 带参形式（027 `OpenItem(item.id)` 仓级惯用法，循环变量在分发期求值），
  msg/on 臂新增。VM 实证：`handler synthesis failed` 0 条、poisoned export
  消失、点击切换详情 PASS。编译器侧"lambda 捕获循环变量"能力缺口登记为
  限制（merge 时入 KNOWN-DEBT），不在本计划扩权实现。
- **T-07** 退出码调查：复现矩阵脚本化 + 结论落账。**无依赖，可并行。** → AC-08。
  [进展] 高频复现条件已固化：ui-gallery VM run（worktree 干净构建）静默
  退出（exit 1/127，无 panic），死亡窗口随机——依赖扫描期、GPU 初始化后、
  MCP 运行数分钟后均观测到；死亡前日志尾部=心跳失败刷屏（2026-09-14 六次
  运行四死）。已升级为 T-03 互锁阻塞项，待专项排查（怀疑方向：VM 执行线程
  栈/心跳派发路径），结论仍按原 AC-08 落账。
- **T-08** AppViewport 形态调研：三候选对比 artifact（工作量/风险/收益，含
  执行栈形态合规性核对）→ **交用户裁定**。依赖 T-01（源可得性影响候选 a 可行
  性评估）。→ AC-09（决策部分）。
- **T-09** AppViewport v1 落地：按裁定实施；若触发合同语义变化先修订本计划。
  依赖 T-08 + 用户裁定。→ AC-09（落地部分）。

## 9. 复审记录

- **2026-09-14 draft handoff（plan_revision 1）**：stage `new`；PLAN-625；
  outcome `pass`——任务覆盖全部 AC 与 SD（T-01..T-09 ↔ AC-01..AC-09、
  SD-01..SD-03），路径/命令已对仓核实（生成器锚点、registry 形态、specs 落点
  均实证）；未决事项 4 条已入 §10 并各有 owner/next。`next: work`（T-01、T-04、
  T-05、T-06、T-07 可立即并行开工；T-02/T-03 依赖 T-01；T-08 后需用户裁定）。

- **2026-09-14 work round 1（plan_revision 2）**：stage `work`；code commits
  auto-lang `d046eab17` / auto-os `40014a7` + `38d7bd7`；task_ids T-01,T-02,
  T-03,T-06 全部完成（[✅] 证据见 §8），T-04 完成归因半程（预存缺陷实锤，
  修复待做）。evidence：`cargo t -p auto-man gallery_registry` 4/4；VM 实测
  33 项列表/过滤/搜索/点击切换/三 tab 内容全套 PASS（截图 p625_vm_*.png）；
  启动日志 registry stub 8→0、synthesis failed 1→0。blockers：F-6 VM 进程
  静默退出高频化（六运行四死，随机阶段），已并入 T-07 排查范围；worktree
  运行需 `AUTO_GALLERY_APPS` env 指向组内语料（组布局探测多一层目录，AGENTS.md
  解析序 env 档覆盖）。plan_revision 1→2：T-06 实现路径按证据调整为 app 端
  msg 带参惯用法（§5.5 裁定注记，goal/AC 不变）。`next: work`（余 T-04 修复
  半程、T-05、T-07、T-08 调研、T-09）。

## 10. 待澄清事项

1. **AppViewport VM 形态选型**（owner：用户裁定；next：T-08 artifact 呈批）：
   候选 a/b/c（§2）；若选 a（进程内子应用渲染）需遵守 AGENTS.md 双重解释器
   路径裁定（避免 run_with_capture 重型栈形态）。裁定后可能触发合同修订
   （plan_revision 2）。
2. **pills 归因对他会话脏改动的依赖**（owner：本计划 T-04；next：最小复现
   先行，A/B 矩阵在他会话收口后补全）：若收口晚于执行窗口，先按 (a)/(b) 分支
   结论暂记。
3. **auto-os 仓侧平行跟踪形态**（owner：执行期确认；next：核对 auto-os 仓
   plan 规则，确定本计划为唯一主跟踪或需镜像条目）。
4. **T-06 触碰共享 codegen 路径时的门禁升级**（owner：执行期 T-06 完成时；
   next：按 AGENTS.md AAVM 触发表复裁，预计改动不涉 aavm 触发路径，零触发）。
