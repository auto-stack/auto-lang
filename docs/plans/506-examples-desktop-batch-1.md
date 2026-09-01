---
plan_id: PLAN-506
status: reviewed                # drafting → executing → execution_done → reviewed → archived
feature_name: examples-desktop-batch-1
author: [kimi-code]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 「504 示例桌面化三件套」段落——由 011 单例样板扩为 008/009/010/002/003/012/038 七例批量兑现；common/header.at(ExampleHeader 196 行组件包)整体退役，theme/accent 设置面移交 os-config per-app 配置"
new_spec_components: []   # 机制零新造（fit 双路径/os-config 读注链/stdlib 静态分发全部复用 504）；新增的是三件套的批量应用与测试改法范式，随 overview 段落更新沉淀
touched_goals:
  - "GOAL-010: 示例应用轨道——011 样板向首批 7 个示例批量展开"
  - "GOAL-009: 虚拟桌面与桌面 Shell——示例桌面化三件套（fit/os-config/stdlib）批量兑现"
  - "GOAL-007: AutoUI 跨端视觉一致——迁移后双端 e2e 全部复绿"

affects: [auto-lang/ui]
current_step: 6
total_steps: 6
---

# [PLAN-506] 示例桌面化批一：008/009/010 header 退役 + 002/003/012/038 fit/title（011 样板批量展开）

## 变更摘要

Plan 504 以 011-calculator 立起桌面化三件套样板（`window: "fit"` 自适应窗口 /
title+settings 上移 os-config / stdlib 静态分发），其非目标明确"其他示例的批量
迁移后续计划按本样板展开"。本计划是批一，两条线共 7 个示例：

1. **header 退役线（008/009/010）**：`common/header.at`（ExampleHeader，196 行，
   theme/accent 5 色设置面板）现仅剩 008-pricing-table、009-article-feed、
   010-contact-form 三个消费方（011 已在 504 退出）。三 app 删 header/settings、
   pac.at 补 `title:`、theme/accent 注册为 os-config per-app 模块；全部退出后
   `common/header.at` 整体删除并 grep 锁零引用。
2. **fit/title 线（002/003/012/038）**：002-counter、003-converter、
   012-stopwatch、038-minesweeper 四个固定内容小工具补 `title:` +
   `window: "fit"`，根容器居中外壳逐个核查去除（012 有一处 `min-h-screen`
   居中，其余三个扫描未见，开工复核）。

机制零新造：fit 双路径（504 S4/S5）、os-config 读注链（504 S7）、stdlib 静态
分发（504 S2）全部复用；stdlib 审计已做——全仓仅 011 有本地 `is_op`（应用
专有保留），无新通用函数入库需求。

## 目标

- **G1 header 退役**：008/009/010 的 app.at 无 ExampleHeader、无 settings 状态；
  pac.at 补 `title:`；theme/accent 经 os-config per-app 配置
  （`~/.config/autoos/apps/<app>/config.at` + `modules.d/auto-<app>.at`，循
  504 calculator 先例）可编辑并在下次 launch 生效；`common/header.at` 删除，
  全仓零引用。
- **G2 fit/title**：002/003/012/038 pac.at 补 `title:` + `window: "fit"`；
  VM 独立窗（`auto run -r vm`）实测按内容收缩、无居中外壳大片空白。
- **G3 验证不破**：有测试套的 5 个 app（003 `test_converter_mcp.py`、
  008/009/010 双端 `test_*_vm.py`/`test_*_vue.mjs`、038 `desktop_mcp.py`）
  更新后全绿；无测试套的 002/012 走 autoui-verifier 截图级双端留痕。
- **非目标**：P504-2 fit 动态重测（本批 6 个 fit 对象全为静态高度，无需求
  载体，继续挂 KNOWN-DEBT 待有动态高度 app 时再做）；其余示例的 title 债务
  与桌面化（批二另立）；os-config 编辑器自身功能；465 vue 虚拟桌面。

## 架构方案

```
批一两线（机制全部复用 504，无 crates/ 源码改动 → 门禁 Category A）

header 退役线（008/009/010，每 app 同构四步）：
  app.at    删 ExampleHeader 引用/settings_open/accent_color 状态与 handler
            （.dark_mode model 初值保留——504 待澄清②裁定方案沿用）
  pac.at    补 title: "..."（theme:/accent: 保留作缺省，os-config 覆盖）
  os-config modules.d/auto-<app>.at 注册 + apps/<app>/config.at 落盘
            （theme/accent shape 循 calculator 先例）
  测试      test_*_vm.py / test_*_vue.mjs 删 settings 面板断言、
            增"无 header 元素"断言（504 test_011 同款改法）
  三线齐后  common/header.at 删除 + grep -r ExampleHeader 全仓零命中锁

fit/title 线（002/003/012/038，每 app 同构三步）：
  pac.at    title: "..." + window: "fit"
  app.at    根容器居中外壳去除（012 已定位 1 处 min-h-screen；其余开工复核）
  验证      VM 独立窗实机收缩截图；003/038 既有脚本增尺寸断言，002/012
            autoui-verifier 截图留痕
```

## 需求分析与背景调查

- 样板与机制：`docs/plans/archive/504-calculator-fit-window-osconfig-stdlib.md`
  （含复审记录与 P504-1..4 债项）；spec 现状段
  `docs/specs/auto-lang/ui/overview.md`「504 示例桌面化三件套」。
- 示例轨道总纲：Design 21 `docs/design/autoui/examples-app-track.md`
  （GOAL-010）。
- 现况扫描（2026-09-01 实测）：
  - `title:` 仅 011/041/043/044 四例有，其余全缺——批一只覆盖目标 7 例；
  - `ExampleHeader` 消费方 = 008/009/010 + common 定义处，别无分店；
  - 本地工具函数审计：仅 011 `is_op`（应用专有，保留），无 stdlib 扩充需求；
  - 居中外壳：012-stopwatch `min-h-screen` 居中 1 处；002/003/038 未见
    （`items-center justify-center` 模式零命中），执行时逐个复核根容器。
- 并行避让：503（execution_done）/505（executing，桌面债批）在另一会话手上，
  本计划只碰 `examples/ui/*` + 用户主目录 os-config 配置，与其零交集。

## 详细设计

### 1. header 退役线（008/009/010）

- **app.at**：删 `use ... common/header.at`（或等价引入）、`ExampleHeader`
  调用点、`settings_open`/`accent_color` 状态与 `.ToggleSettings` 等 handler；
  页面根容器由原"header + 内容"改为内容即页面（011 S6 同款）。
- **pac.at**：`title: "Pricing Table" / "Article Feed" / "Contact Form"`；
  既有 `theme:`/`accent:` 缺省保留（os-config 文件缺席时的回退，504 优先级
  链：os-config > pac.at > 内置）。
- **os-config 注册**（用户主目录，非仓内文件）：
  `~/.config/autoos/modules.d/auto-pricing-table.at` 等三条 +
  `apps/<app>/config.at`（`theme: "dark"`、`accent: "indigo"` 缺省落盘）。
  验证：daemon 热注册后 `/api/config/auto-<app>` 返回 shape 正确（504 S7
  同款手法）。
- **窗口尺寸**：008/009/010 为内容弹性表格/列表/表单，**不上 fit**（保留
  默认窗）——批一裁定，避免动态高度场景撞 P504-2。

### 2. fit/title 线（002/003/012/038）

- **pac.at**：`title:`（Counter/Converter/Stopwatch/Minesweeper）+
  `window: "fit"`。
- **app.at**：012 删 `min-h-screen` 居中外壳；002/003/038 开工先复核根容器
  （若有 `flex-1 items-center justify-center` 类同构外壳一并去除）。
- **038-minesweeper**：已有 `desktop_mcp.py`，疑已在桌面注册表有条目——
  S4 开工时核对其 pac.at 现状与桌面装载路径，避免重复注册。

### 3. common/header.at 退役

- 三线（008/009/010）全部退出后：`git rm examples/ui/common/src/front/header.at`；
  检查 `common/pac.at`（若存在）对 header 的声明一并清除；
  `grep -rn ExampleHeader examples/ --include='*.at'` 零命中作为删除门禁。

### 4. 测试更新

- 008/009/010：`tests/test_*_vm.py` 与 `test_*_vue.mjs` 删 settings 面板
  交互/断言，增"无 header/settings 按钮"断言（504 test_011 改法同款）。
- 003/038：既有脚本增窗口尺寸收缩断言（参考 504 test_011_vm.py 的 fit
  断言形态）。
- 002/012：无测试套——autoui-verifier 双端截图留痕（不新写测试套件，
  见待澄清①）。

## 测试设计

- **门禁分级**：本批不改 `crates/` Rust 源码、不改文档生成器/Schema——
  **Category A**，严禁 `cargo t`/`docs_gen`；验证 = autoui-verifier 双端 +
  各 app 既有测试脚本。
- **双端一致性**：每个迁移 app 走 `.agents/skills/autoui-verifier` 标准化
  脚本（`test_vm_mcp.py` / `test_vue_playwright.mjs`），Vue 模式 `auto run`、
  VM 模式 `auto run -r vm`。
- **fit 实机断言**：VM 独立窗截图 + 窗口尺寸收缩比对（默认 1293x836 →
  内容尺寸 + chrome），每 app 留 `src/front/tests/screenshots/` 或等价留痕位。
- **os-config 链路抽查**：008/009/010 中抽 1 个（建议 009）做全链实测
  （改配置 → 重启 → `UI theme: ... (from os-config)` 打印 + 视觉确认），
  其余两个同构验证注册 API 返回即可。

## 验收标准

1. 7 个 pac.at 均有 `title:`；008/009/010 app.at 无 ExampleHeader/settings；
   `common/header.at` 已删且全仓 grep 零引用。
2. 002/003/012/038 `window: "fit"` 生效：VM 独立窗实测收缩截图留痕，
   无居中外壳空白。
3. 008/009/010 的 theme/accent 可由 os-config 编辑并在下次 launch 生效
   （009 全链实测 + 008/010 注册 API 验证）。
4. 003/008/009/010/038 既有测试脚本更新后全绿；002/012 双端截图留痕。
5. 零 crates/ 源码改动（若执行中发现必须改宿主，升级 Category B 并补
   `cargo check -p auto-lang` + scoped 测试，在复审记录中说明）。

## 执行步骤

1. **[S1] 开工复核**——逐个核对 7 目标 app 的 pac.at/app.at 根容器现状
   （002/003/038 居中外壳复核、038 桌面注册表现状），修正本计划初判表。
   验证：复核结论追加到本文件「待澄清事项」对应条目。
   [✅ 已完成] 复核结论落「待澄清事项 · [S1]」：002/003/038 实测均有
   `center` 外壳（初判修正，S4 四件全拆）；038 无注册表条目无重复风险；
   settings 状态全在 header 组件自持，app 级仅删死声明 `accent_color`。
2. **[S2] 008/009/010 header 退役**——每 app 四步（app.at/pac.at/os-config
   注册/测试更新）。
   验证：三 app 双端测试绿 + 009 os-config 全链实测。
   [✅ 已完成] 三 app VM 测试绿（无 header + 内容标记断言）；008/010 vue
   绿 + 009 vue 绿；009 os-config 全链实测通过（dark→light/coral 改配置
   →重启→`UI theme: light (from os-config)`/`accent: coral` 打印 + 截图
   `009_vue_osconfig_light_coral.png` 视觉确认，验毕恢复缺省 dark/indigo）；
   008/010 注册 API 验证通过（临时起 daemon `:17901`，
   `/api/config/auto-{pricing-table,article-feed,contact-form}` 均返回
   `{theme,accent}` 正确 shape，与 auto-calculator 同构）。
   执行偏差（见待澄清④）：`accent_color` 声明按 504 样板**保留**（os-config
   播种挂钩），不按详细设计字面删除；恢复后三 app VM 复跑仍绿，vue 端
   复跑并入 S5 总验证。
3. **[S3] common/header.at 退役**——删除 + 零引用锁。
   验证：`grep -rn ExampleHeader examples/ --include='*.at'` 零命中。
   [✅ 已完成] `git rm examples/ui/common/{pac.at,src/front/header.at}`
   （整包移除，含空壳 pac.at）；`ExampleHeader`/`dep "common"`/
   `common/header` 三道 grep 全仓 .at 零命中；008 VM 复跑绿确认无隐性
   引用。执行注记：app.at 注释措辞改写避开门禁误命中；009 曾被
   `auto run` 回写 dep 声明 + 物化 deps/common，已一并清理。
4. **[S4] 002/003/012/038 fit/title**——每 app 三步（pac.at/app.at 外壳/
   验证留痕）。
   验证：四 app VM 独立窗收缩截图 + 003/038 脚本尺寸断言绿。
   [✅ 已完成] 四 app pac.at 补 title + window:"fit"；app.at 拆 center
   外壳（002/003/038 实测均有，初判已修正）+ 012 删 min-h-screen + 038
   清死声明 window_width/height。VM 独立窗 fit 实测留痕：002 400x400 /
   003 400x720 / 012 550x774 / 038 647x878（默认 1293x836，
   `<app>_vm_fit.png` 四份落 src/front/tests/screenshots/）。003 尺寸
   断言绿（套件内 PNG 尺寸 < 900）；038 T6 尺寸断言被 master 预存
   **VM RC use-after-free**（rc.rs:530 canary，Reveal 的 struct 字面量
   板重建触发，主 checkout 复现同崩，与 506 改动无关）挡住——套件加
   防御后 12 pass / 1 FAIL（UAF）/ 2 skip，fit 证据由独立 probe 补
   （647x878）；UAF 登记 KNOWN-DEBT。038 事件定位同步修复（label 法，
   rendered-vtree 快照无事件注记，master 预存失效）。002/012 vue 端
   截图留痕并入 S5。
5. **[S5] 双端总验证**——autoui-verifier 七 app 双端一致性全跑。
   验证：七 app 全部留痕通过。
   [✅ 已完成] **Vue 端**：008/009/010 测试脚本绿（无 header + 内容标记
   断言，accent_color 恢复后最终代码复跑）；002/003/012/038 标准脚本
   截图留痕（`<app>_vue.png`，038 走 front_port 4038）。**VM 端**（S2-S4
   已覆盖）：003/008/009/010 套件绿；002/012/038 快照+fit 截图留痕；
   038 唯一 FAIL 为 master 预存 RC UAF（见 S4 注记）。双端留痕 20 份
   （`src/front/tests/screenshots/`，gitignored 本地留痕，已同步主
   checkout 同路径）。启动打印三 app 均 `UI theme/accent (from
   os-config)` + `VM window title (from pac.at)`；fit 四 app 均
   `VM window size: fit (content-measured, from pac.at)`。
6. **[S6] 收尾**——复审记录、KNOWN-DEBT 新增项登记（如有）、spec 沉淀、
   批二范围建议（剩余 title 债务 app 清单）。
   [✅ 已完成] KNOWN-DEBT 登记 P506-1（038 Reveal VM RC UAF，master
   预存，疑 511 回归）+ P506-2（MCP 快照无事件注记，master 预存）；
   spec 沉淀归 /auto-plan:merge；批二建议见下。**执行侧验收自检**：
   ① 7 pac.at 均有 title ✓；② 008/009/010 无 header/settings（双端
   断言绿）+ common/header.at 已删三道 grep 零命中 ✓；③ fit 四 app
   实测收缩（400x400/400x720/550x774/647x878）留痕 ✓；④ os-config
   三模块注册 API shape 正确 + 009 全链实测（light/coral 视觉确认）
   ✓；⑤ 既有脚本：003 绿（含新 fit 断言）、008/009/010 双端绿、038
   12p/1f（唯一 FAIL=master 预存 UAF，非本批改动）、002/012 截图级
   留痕 ✓；⑥ 零 crates/ 改动（Category A 守住，本批只碰 examples/ +
   用户主目录 os-config）✓。
   **批二范围建议**（剩余 title 债务 20 个正式示例）：001-helloworld、
   004-profile-card、005-login、006-hero-section、007-stats-board、
   013-todo、014-weather、015-notes、016-calendar、017-chat、
   018-book-reader、019-video-app、020-music-player、021-blog-viewer、
   022-kanban、023-realworld、024-charts、025-dashboard、028-launcher、
   042-two-inputs-child（p051/p493/p507 为计划工作目录不计）。建议批二
   分两线：title-only 线（纯补 title 无行为变化，可快速批）与 fit 线
   （逐个判内容形态 + 拆外壳，注意 P504-2 动态高度债与 P506-1 UAF
   修复后再碰交互型 app）。

## 复审记录

**复审人**：ZCode 会话（/auto-plan:review，2026-09-01）
**复审基线**：worktree `.worktrees/plan-506-dev` @ `115d368e7`（3 commits：
ef2d3634f S2 / 8a548b563 S3 / 115d368e7 S4）；diff 24 files、+288/−582，
全部位于 `examples/ui/`，`git diff --stat -- crates/` 为空。

**门禁裁定**：本批零 `crates/` Rust 源码改动（验收 5 实测成立）→ 按
AGENTS.md Change-Scoped Verification Gate **Category A**，复审不运行
`cargo tf`/`cargo t`（仓库规则明令严禁，覆盖 review 技能通用 full-suite
条款）；full-suite 门禁本就不适用于纯示例/测试脚本批次。

**逐条验收重验**（复核现跑，非引执行期输出）：

1. **PASS** — 7 pac.at `title:` 逐一 grep 命中；008/009/010 app.at 无
   ExampleHeader/settings_open/accent_color 死声明（accent_color 为带
   注释的播种挂钩保留，见待澄清④）；`examples/ui/common/` 整目录已删；
   `ExampleHeader`/`dep "common"`/`common/header` 三道 grep 全仓 .at
   零命中（exit=1）。
2. **PASS** — 4 pac.at `window: "fit"` 命中；4 app.at 无 `center {` 外壳
   （012 仅注释文字残留 min-h-screen 字样，样式串为零）；fit 截图四份
   复测 PNG 尺寸 400x400 / 400x720 / 550x774 / 647x878（默认窗
   1293x836，全部 <900 收缩阈值）。
3. **PASS** — os-config 六文件落盘在位；临时 daemon `:17901` 复验
   `/api/config/auto-{pricing-table,article-feed,contact-form}` 三条均
   返回 `{theme,accent}` + meta 正确；008 vue 复跑启动打印
   `VM window title: Pricing Table (from pac.at)` + `UI theme/accent
   (from os-config)`；009 全链实测（light/coral→重启→打印+视觉截图）
   执行期完成并留痕 `009_vue_osconfig_light_coral.png`。
4. **PASS（带披露豁免）** — 003/008/009/010 VM 套件复跑全绿（003 含
   fit 断言 400x720）；008 vue 复跑绿；038 desktop_mcp.py 复跑 12
   passed / 1 failed / 2 skipped——唯一 FAIL 为 **master 预存 VM RC
   use-after-free**（rc.rs:530 canary；执行期已在主 checkout 复现同
   崩 + RUST_BACKTRACE 定位，非本批 diff 引入），本批并将其从
   "IndexError 崩溃不可跑"修复为可报告（label 定位法 + 防御），
   038 自身交付（fit/title/拆外壳）完整且有独立 fit 证据。豁免依
   KNOWN-DEBT 既有实践（P499-6/7、P507-2 预存移入债表与计划完成
   并存）。002/012 双端截图留痕在位。
5. **PASS** — `git diff --stat -- crates/` 空；Category A 维持，无需
   补 cargo check/scoped 测试。

**遗漏/延后/workaround 扫描**：

- 遗漏：无——S2 四步/S3 删除+门禁/S4 三步在 diff 中逐项对应；
  common/pac.at 空壳随包移除；gen//deps/ 为 gitignored 本地生成物，
  无陈旧入库风险。
- 延后：①P504-2 fit 动态重测为计划**非目标**明示排除（非静默）；
  ②批二（剩余 20 例 title 债务）在 S6 给出建议清单，属计划外后续
  轨道；③待澄清①"002/012 截图级验证"按计划建议路线执行（测试套
  建设归批二/专项），**需用户在 merge 前知悉确认**。
- Workaround：①accent_color 保留=计划文本相对 504 样板的修正执行
  （机制依据：seed_app_config/renderer env 播种均要求已声明变量，
  删除即验收 3 静默失效；已记待澄清④）；②038 UAF 防御（记 FAIL
  不死锁）+ label 定位法=上游 VM bug 所迫，已登记 P506-1/P506-2。

**债项**：P506-1（038 Reveal RC UAF，疑 511 回归，修后 038 套件应回
全绿）、P506-2（MCP rendered-vtree 快照无事件注记）——均已在
KNOWN-DEBT-AND-RISKS.md 登记。

**结论**：5/5 验收通过（验收 4 带披露豁免），无阻塞债 →
`status: reviewed`，就绪 `/auto-plan:merge`。

## 待澄清事项

1. 002/012 无测试套——本期截图级验证是否足够，还是补最小 desktop_mcp
   脚本？（建议截图级，测试套建设归批二或专项；待用户裁定）
2. os-config 模块 id 命名：`auto-<app>` 循 calculator 先例（auto-counter /
   auto-converter / ...），配置 shape 统一 theme/accent 两键（无 settings
   的 fit 四件不注册模块）——开工按此执行，如有异议立项确认时提出。
3. 008/009/010 不上 fit 的裁定（内容弹性 + P504-2 未偿）——若用户希望
   这三个也 fit，需先立项偿 P504-2（内容尺寸变化信号→宿主重测）。
4. **[S2 执行裁定] `accent_color` 保留不删**：计划详细设计 1 写"删
   accent_color 状态"，但 504 样板（011-calculator）与宿主双播种路径
   （desktop launch 的 `seed_app_config`：os-config → 已声明
   `dark_mode`/`accent_color`；`auto run` 的 env 播种 + vue codegen 的
   `has_accent_color`）都要求 App **声明**该变量才能让 os-config accent
   生效——删声明 = 验收 3（accent 可由 os-config 编辑生效）静默失效。
   已按样板保留（带注释标注播种用途）；此为计划文本相对其引用先例的
   修正执行，非范围变更。

### [S1] 开工复核结论（2026-09-01 实测，修正初判表）

- **初判修正（002/003/038 居中外壳）**：计划初判"002/003/038 扫描未见
  居中外壳"不成立——三 app 根容器实为 `center { ... }` 包裹（002:
  `center { text+row }`；003: `center { col }`；038: `center { col }`，
  038 内层已是 `max-w-fit`）；012 为 `center` + 内层 `min-h-screen` 双重。
  对照 011 样板（view 根直接 `col`、无 center 包裹），S4 四 app 均需拆
  center 外壳（内容即页面），012 另删 `min-h-screen`。
- **038 桌面注册表现状**：`~/.config/autoos/apps/` 仅 calculator/musk、
  `modules.d/` 仅 auto-calculator.at——无 minesweeper 条目，无重复注册
  风险；`tests/desktop_mcp.py` 是纯测试脚本，与注册表无关。fit 四件
  按待澄清②不注册 os-config 模块。
- **header 消费形态**：008/009/010 的 settings 状态（settings_open/
  ToggleSettings/SetTheme/SetAccent）全部在 header 组件内部自持；
  app 级仅 `dark_mode`（内容区大量 `if .dark_mode` 引用→保留）+
  `accent_color`（三 app 均声明未使用→删）。app `on{}` 无 settings
  handler，ExampleHeader 调用点各 1 处（带 icon/title/badge 三参数）。
- **fit 断言手段**：`autoui_snapshot` 支持 `include_bounds: true` 输出
  根节点 `@rect x,y,w,h`（逻辑像素）——003/038 尺寸断言用此实现
  （504 test_011 仅截图留痕无程序化断言，本批增强）。
- **common 包残留**：`common/src/front/` 仅 header.at 一个文件；
  `common/pac.at`（name: ui-common）无 header 显式声明。S3 删
  header.at 时需同步删 008/009/010 pac.at 的 `dep "common"` 块
  （否则指向空包），common 目录随之空壳——S3 一并移除。
