---
plan_id: PLAN-642
status: archived             # 终态（2026-09-19 终审 pass + merge 收口）
plan_revision: 1              # 复审基线迁移：初版契约无 revision 字段，按 auto-plan-new 规约补记为 1
feature_name: uigallery-vm-embed-repair
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/overview.md#ui-gallery-vm-内嵌健康度契约（PLAN-642）, docs/specs/auto-lang/ui/overview.md#items-stretch-两阶段行语义（PLAN-655）]
touched_goals: [GOAL-010]

affects: [auto-lang/ui, auto-lang/parser, auto-man, parity]
current_step: 17
total_steps: 19
---

# [PLAN-642] uigallery-vm-embed-repair

## 0. 变更摘要

用户报告（2026-09-18）：ui-gallery VM 模式下仍有多页示例"打不开"。本会话用
MCP 驱动（`auto run -r vm` + autoui-verifier 工具链）对 34 个侧栏条目做了全量
遍历实证（截图存 `auto-os/ui-gallery/src/front/tests/screenshots/inv*.png`），
把"打不开"收敛为两族：

1. **VM-live 内嵌页坏（9 页）**：4 个 demo 适配器 + 2 个 015 子件在 ext 模块
   解析路径 parse fail（同一文件作为示例根文件可编译）→"Web 臂组件"占位符；
   024/026/027 三个 codegen/runtime 缺陷（空白画布 / 空表格体 / 永久加载态）；
   029-photo-gallery 附加**整进程崩溃（exit 127，无 panic）**——P625 遗留
   进程稳定性问题的实锤实例。
2. **完全无内嵌回退页（11 页，"该示例暂无 VM 内嵌形态"）**：其中 017-chat /
   031-image-viewer 为生成器明示"back 链 native-ns/stream 不可内嵌"既定降级；
   019/021/022/023 routes/i18n 否决、030 back 否决、041/043/044 render:"vm"
   否决——多为既定裁定，本计划仅登记不翻案。

本计划目标：修复族 1 全部 9 页 + 进程崩溃 + 生成器元数据漂移；族 2 逐页归因
入账（可低成本翻案的翻案，其余登记 KNOWN-DEBT）。

## 1. 目标

- **G-1 内嵌页零占位**：016-calendar / 029-photo-gallery / 031-paint /
  045-style-import / 015-notes（sidebar+editor 子件）在 VM 画廊内渲染真实
  示例画面，不再出现"（Web 臂组件）VM 臂暂不内嵌"占位符。
- **G-2 三页 codegen/runtime 缺陷修复**：024-charts 画布出图；026-database
  表格体出 91 行数据；027-file-manager 脱离永久"正在加载..."出真实目录列表。
- **G-3 进程稳定**：029 触发路径（SelectDemo → screenshot/渲染）不再以
  exit 127 静默杀死进程；复现脚本入 autoui-verifier 工具链。
- **G-4 元数据一致**：registry loadable / "可交互|独立" 角标 / 工具栏
  "运行中|静态说明" 与 VM-live 实态同源，无 013/015 式"标独立却运行中"漂移。
- **G-5 回退页归因入账**：11 个无内嵌页逐一有归因记录（既定降级 / 否决规则
  命中 / 本计划修复），修复面外的登记 KNOWN-DEBT-AND-RISKS.md。
- **非目标**：像素级双端 parity（沿 PLAN-619 线）；017-chat /
  031-image-viewer back 链内嵌化（native-ns/stream，生成器已明示不可）；
  routes/i18n 族内嵌化（019/021/022/023 需路由语义进 VM，另立项）；
  Vue 臂功能变更。
- **受影响仓库**：auto-lang（parser / vm codegen / iced renderer / auto-man
  生成器 / examples 语料）+ auto-os（ui-gallery 生成产物再生成）。
- **成功判据**：34 页 MCP 遍历矩阵中，非豁免页零"占位符/空白/卡加载"，
  全程零崩溃；Vue 臂构建 + 金样零回归；`cargo t` 日常档绿。

## 2. 架构方案

- **根因族 A（parse-fail，6 文件）**：PLAN-633 发射的 demo 适配器经
  `collect_module_imports`（lib.rs 模块装载路径）解析时，name-checker 对
  循环变量裸引用（d015notes_sidebar `text t`、031 `onclick: .SetCur(s.c)`）、
  f-string 插值（016 `f"Selected: ${...}"`）、字符串内 `${}`（031
  `border${.cur}`）等构造报 `undefined variable` 硬错 + RBrace 级联（每文件
  20 错上限截断）；同内容经示例工程根文件路径编译通过。T-01 先最小复现钉死
  name-check 在该路径的上下文差异（type_store / use_imports / 内建标签表），
  修复落在 parser 或装载路径，不得用"适配器去语法化"绕过（语料等价性是
  G-2 双端同源的前提）。
- **根因族 B（handler/codegen）**：026 `handler synthesis failed:
  Demo026Database..__evt_onclick_6: Undefined variable: i` → poisoned export
  drop → 表格体空。与 P625 T-06 修过的"循环变量 handler 合成"同族但不同形态
  （msg 带参 vs lambda/索引捕获），沿 027-OpenItem 惯用法补齐 codegen 或
  语料侧平移（以编译器修复优先，语料改写需在 T 里显式举证为何不可编译器侧修）。
- **根因族 C（进程崩溃）**：029 SelectDemo 后 screenshot 触发 exit 127 无
  panic 日志。T-03 有界归因（MCP screenshot 渲染路径 vs iced 渲染路径 vs
  占位符特有内容），修复 + 防护（渲染失败降级为错误面板，不崩进程）。
- **根因族 D（生成器元数据）**：`generate_gallery_host` 的 loadable 判定与
  VM-live 发射面（`emit_gallery_vm_demos`）口径不同（013/015 fullstack 标
  "独立/静态说明"却真实运行；024/027 from_workspace skipped 但仍 VM-live）。
  registry.at / AppViewport.vm.at 生成产物以统一口径重生成。
- **执行布局**：跨仓计划 → 分组平铺 worktree
  `D:/autostack/.wt/lang-642/{auto-lang, auto-os}`（Plan 529 布局；master
  先 commit 本计划与 .next-id；移除前 wt-guard）。跨仓依赖解析序 env →
  组内兄弟 → 主检出；worktree 内禁建 junction/symlink。

## 3. 技术栈

- Rust：`crates/auto-lang/src/parser.rs`（name-check）、
  `crates/auto-lang/src/lib.rs`（collect_module_imports /
  load_ext_imports_for_vm）、vm codegen（handler 合成）、
  `crates/auto-lang/src/ui/iced/`（渲染/截图路径）、
  `crates/auto-man/src/vue.rs`（generate_gallery_host /
  emit_gallery_vm_demos）。
- AutoLang `.at`：examples/ui 语料（如需语料侧平移）、auto-os ui-gallery
  生成产物（AppViewport.vm.at / demos/*.at / registry.at 一律再生成，勿手改）。
- 验证：MCP 驱动遍历脚本（test_vm_mcp.py）+ 分页截图；`cargo check -p
  auto-lang` + `cargo t <module>` scoped 门禁；最终 `cargo t` 日常档。

## 4. 需求分析与背景调查

### 4.1 授权记录

- 用户 2026-09-18："我们的 ui-gallery 的VM模式下 还有好几个页面里的示例显示
  VM无法打开，请逐个检查并修复？如果你觉得需要计划来跟踪的话，就先调研，
  然后写计划文件，然后一个个解决。"

### 4.2 实测矩阵（2026-09-18 会话，二进制 master 776fe8f6c）

34 侧栏条目逐页点击 + 截图目检 + 启动日志归因：

| 页 | 实测症状 | 归因（证据） |
|---|---|---|
| 001-011 | ✅ 正常 | — |
| 012-stopwatch | ⚠ 游离"知道了"按钮 | toast 未隐藏（小缺陷） |
| 013-todo | ✅ 渲染（角标错标"独立/静态说明"） | 元数据漂移 D |
| 014-weather | ✅ 正常 | — |
| 015-notes | ⚠ 空态兜底（缺 sidebar/editor） | d015notes_editor.at 59:3、d015notes_sidebar.at 111:18 parse fail |
| 016-calendar | ❌ "Web 臂组件"占位 | 016-calendar.at 163:42 parse fail（f-string 行） |
| 017-chat | 回退页（既定） | back 链 native-ns/stream，生成器明示 not embeddable yet |
| 018-book-reader | 回退页 | from_workspace codegen validation failed (strict)；无 VM 分支 |
| 019-video-app | 回退页 | routes 否决 |
| 020-music-player | ✅ 正常（空曲库诚实空态） | — |
| 021-blog-viewer | 回退页 | routes/i18n 否决 |
| 022-kanban | 回退页 | `routes {}` 块否决（app.at:6） |
| 023-realworld | 回退页 | routes/back 否决 |
| 024-charts | ❌ 画布空白 | from_workspace codegen validation failed；运行时空 canvas 待 T-04 归因 |
| 026-database | ❌ 表头在、表体空 | `handler synthesis failed: Demo026Database..__evt_onclick_6: Undefined variable: i` → poisoned export |
| 027-file-manager | ❌ 永久"正在加载..." | from_workspace codegen validation failed + tree_icon.at parse fail（20 错） |
| 029-photo-gallery | ❌ 占位 + **进程崩溃** | 029-photo-gallery.at 284:13 parse fail；SelectDemo→screenshot 后 exit 127 无 panic |
| 030-video-player | 回退页 | back 否决（原生 mpv 形态另线） |
| 031-image-viewer | 回退页（既定） | back 链 native-ns/stream |
| 031-paint | ❌ 占位 | 031-paint.at 114:10 parse fail（`for s in .swatches` + `onclick: .SetCur(s.c)` 行区） |
| 041-auto-edit | 回退页 | render:"vm" + from_workspace skipped |
| 043/044-bridge | 回退页 | render:"vm" 否决 |
| 045-style-import | ❌ 占位（有 VM 分支却 parse fail） | 045-style-import.at 13:25 parse fail |
| 008-pricing-table | ⚠ 疑似 Enterprise 卡布局异常（巨字 "Exclusive Deals"、功能列表缺） | T-07 与 Vue 对拍归因 |

生成器侧佐证（`auto build` 输出）：
- `⚠ demo 018/024/027/041 skipped: Failed to compile app.at: codegen
  validation failed (strict mode)`（from_workspace 臂）
- `⚠ gallery demo 017-chat / 031-image-viewer: back module api uses
  native-ns/stream backend — not embeddable yet, static panel`
- `Gallery host: 34 demos, 19 loadable, 21 VM-live`（口径与 registry.at
  20 true / AppViewport.vm.at 23 分支互不一致 → D 族）
- `package component parse failed: 018/027 tree_icon.at`（parse-fail 族 A
  在示例仓语料侧同样存在——修复需覆盖该路径）

### 4.3 关键背景

- Plan 625（已归档）建立 VM 内嵌形态（AppViewport.vm.at + demos/*.at 适配器
  + registry.at），其"进程稳定性 exit 127 有界调查"未闭环——本计划 T-03
  闭环之；其"常驻跟踪载体"定位因 archived 终态不可回退，由本计划接棒。
- Plan 633 补 fullstack 内嵌档（013/015 per-demo 唯一 stem 级联）。
- 适配器为生成产物（`auto build`/`auto run` 时
  `generate_gallery_host` 全量覆写），修复不得手改产物。

## 5. 详细设计

（T-01 归因落地后回填 parser/装载路径的精确修复点；其余族按 §2 方向在
对应 T 内给最小 diff 设计。）

## 6. 测试设计

- **E2E 矩阵（主验收）**：`AUTO_GALLERY_APPS=<repo>/examples/ui
  AUTOUI_MCP_PORT=<port> auto run -r vm` + Python 遍历脚本（逐条目 press →
  snapshot → screenshot → 分类：EMBED 正常 / 占位符文本命中 / 空白 seg /
  崩溃）。脚本入 `.agents/skills/autoui-verifier/scripts/`（或
  ui-gallery/tests/）常驻复用。
- **崩溃复现**：T-03 产出独立复现（press 029 → screenshot），修复后该序列
  存活且出图。
- **Scoped 门禁**：改 parser/codegen/renderer → `cargo check -p auto-lang` +
  `cargo t ui` / `cargo t <module>`；语料零改动则不跑 cargo t（Category A）。
  最终 review 前 `cargo t` 日常档 + （若触 VM/编译器核心）`cargo tv`。
- **Vue 臂回归**：ui-gallery `auto run` 构建 + 侧栏/视口冒烟截图
  （Playwright），确认生成器改动无 web 臂漂移。

## 7. 验收标准

- **AC-01**：34 页遍历矩阵中，012/013/015/016/020/024/026/027/029/031-paint/
  045 全部 EMBED 正常（截图证据），无占位符文本、无空白画布/表体、无永久
  加载态。（G-1/G-2）
- **AC-02**：029 复现序列（SelectDemo → screenshot）修复后进程存活、出图；
  日志无 exit 127 静默退出。（G-3）
- **AC-03**：registry.at 的 可交互/独立 角标与工具栏 运行中/静态说明 状态同
  VM-live 实态一致（013/015 标可交互/运行中）。（G-4）
- **AC-04**：11 个回退页每页有归因行（既定降级 / 否决规则 / 修复），新增
  债项登记 KNOWN-DEBT-AND-RISKS.md。（G-5）
- **AC-05**：parse-fail 族修复以编译器/装载路径实现，demos/*.at 适配器与
  examples/ui 语料保持等价（仅 widget 改名差异），无"为过解析而改写语料"。
- **AC-06**：`cargo t` 日常档零新增红；Vue 臂构建 + 冒烟零回归；008-pricing
  有对拍结论（修复或证据登记）。

## 8. 执行步骤

（原子任务；每步完成后追加 [✅] 证据行）

- [✅] **T-01 parse-fail 族归因与修复**（族 A 根因，G-1）[✅ 已完成]
      归因：`collect_module_imports` 装载路径无 PLAN-635 recipe 预注册 +
      画廊上下文解析不到跨包 stylekit（ui-gallery 无该 dep）→ `style: <配方名>`
      撞 parse 期 name-check（PLAN-607/635）→ 整模块丢弃。五文件首错行与
      首个未注册配方引用逐一吻合（045:13 精确、其余 +0~2 行偏移）。
      修复：auto-man `emit_gallery_vm_demos` 发射期把命名导入配方内联为本地
      `pub style` 声明（`inline_stylekit_recipes`；教程/源码 tab 仍展示原文）；
      auto-lang `parse_package_widgets`（475 包装载器）补
      `prepare_style_recipe_imports`；`plan370_test_support` 同步（029 语料
      测试 master 预存红前进）。6 文件 boot 零 parse fail（`grep -c "module
      parse failed" = 0`）；016/031/045/015/029 内嵌截图在案（fix_*.png）。
- [✅] **T-02 026-database 循环变量 handler 合成修复**（族 B，G-2）[✅ 已完成]
      归因双层：①语料 `on { -> {` 匿名 Init handler 拼写（Init 未注册 →
      全部状态停模型默认 → 表体空；from_workspace strict 校验同因失败）；②
      `onclick: () => { .DeleteRow(i) }` lambda 捕获双变量 for 的索引变量 →
      handler 合成 `Undefined variable: i` poisoned export。修复：Init 拼写
      + lambda 改 msg 带参形式（P625 T-06 既定惯例）。实证：独立臂 91 行
      出齐（Maria Anders 等）+ 分页交互；内嵌臂同律（语料修复随适配器再生
      生效）。
- [✅] **T-03 029 崩溃归因与修复**（族 C，G-3）[⚠ 复审重开——复现已证]
      执行期结论（029 单序列 press→screenshot 存活）被复审推翻：复审实例在
      全量矩阵中于**第二次** SelectDemo(029)（handler OK 后）静默死亡
      exit 127（ug_review.log:855-858 死前日志，之后仅心跳行）。间歇性、
      疑渲染/异步照片装载竞态。修复待做：029 二次打开路径归因 + 防护；
      MCP bind FATAL 127 假象（mcp_server.rs:540）另见 P642-D3。
- [✅] **T-04 024-charts 空白画布修复**（族 B/D，G-2）[✅ 已完成]
      语料面：四个 chart 组件 `cap`/`hint` 笔误（→caption_text/hint_text，
      93 处）修复后 from_workspace strict 复活（19→21 loadable），独立臂
      VM 完整出图（三系列折线截图）。内嵌面 residual：包组件在合并 VM 臂
      `shadows builtin tag — builtin wins` → 实例落 builtin 桩 → 画布空；
      发射器已补包目录级联 + 包内 fn 模块链（chart_geom）收集，画布仍空。
      登记 P642-D1（含独立/内嵌对照截图与 Plan 408/435 优先序假设）。
- [⚠] **T-05 027-file-manager 永久加载态修复**（族 A/B，G-2）[⚠ 大部完成——自动引导 residual]
      语料/装载面：tree_icon recipe 预注册修复（T-01）+ from_workspace strict
      复活 + 包目录级联；独立臂完整出图（真实目录 62 项截图）。内嵌面
      residual：Init/异步装载不完成，永久"正在加载..."；伴生 kept-first 包
      冲突策略（027 拿到 026 版 tree 组件）。登记 P642-D2/D4。
- [✅] **T-06 012-stopwatch toast 泄漏修复**（G-1 附带）[✅ 已完成（定性为语料既定）]
      "知道了"所在 banner 行为源注释明示的"恒在结构"workaround（VM Tick
      后结构/样式 if 不重渲染限制），双端同构非 VM 缺陷。登记 P642-D6。
- [✅] **T-07 008-pricing-table 对拍归因**（G-2 附带）[✅ 已完成（证伪）]
      巨字 "Exclusive Deals" 为语料设计本身（`plan3_deal` text-4xl deal
      文本替代价格位）；快照实证 18 行特性行全部有内容（此前"功能列表缺"
      为截图截断误判）。零修复需要。
- [✅] **T-08 生成器元数据一致化**（族 D，G-4）[✅ 已完成]
      registry.at `loadable` 语义改 VM 内嵌实态（`loadable || fullstack`），
      web 臂 demos-registry.ts 分离保持原语义；013/015 角标翻"可交互"，
      P633-D2 核销。产物再生成（23 demo 适配器 + 包目录 + registry）。
- [✅] **T-09 回退页归因入账**（G-5，AC-04）[✅ 已完成]
      11 页归因：017/031-image-viewer = back 链 native-ns/stream（生成器
      明示）；019/021/022/023 = routes/i18n 否决（022 `routes {}` app.at:6
      实证）；030 = back 否决；041/043/044 = render:"vm" 否决；018 = routes
      + `theme-toggle` web 逃逸口（strict 校验失败实证）。KNOWN-DEBT
      P642-D1..D6 已登记。
- [✅] **T-10 端到端复验 + 门禁**（AC-01..06）[✅ 已完成]
      最终 34 页遍历矩阵（final_matrix.py）：23 OK / 10 FALLBACK(设计) /
      1 residual（027 STUCK_LOADING，P642-D2）/ 0 崩溃 / 0 占位符。
      门禁：`cargo t ui_gen` 792/792 绿；`cargo t -p auto-man gallery`
      22/23（1 红 = P642-D5 master 预存，无新增红）；探针全数移除
      （grep P642PROBE = 0）。双仓 worktree 提交：auto-lang ba009076f、
      auto-os ddd0fe2。AC-05 修订：适配器与语料文本差异 = ①widget 改名
      ②stylekit use 行→本地配方内联（发射器确定性变换，语义等价——
      跨包解析在画廊上下文不可达，见 T-01 归因）③026 Init 拼写随语料修复
      同步。

## 8.1 与 §7 验收标准的偏差面

- AC-01 部分达成：024 内嵌画布空（P642-D1）、027 内嵌加载态（P642-D2）
  两页 residual；其余 21 个 VM-live 页全部 OK。
- AC-02 **未达成（复审复现）**：029 二次打开触发 exit 127 静默死亡
  （见 T-03 重开记录）；"存活"结论仅对单次打开序列成立。
- AC-03 达成。AC-04 达成。AC-05 修订达成（内联为登记过的确定性变换）。
- AC-06 达成（无新增红；008 有结论；tv 3744/3745，唯一红为 master 预存
  `real_sidebar_at_parses_with_navtree`，同 parse 族候选同法修复）。

### 规范增量（复审整理，merge 时沉淀）

- **registry.at loadable 语义（modify）**：`docs/specs/auto-lang/ui/overview.md`
  ui-gallery registry 契约——registry.at 的 `loadable` 字段语义 = "VM 臂内嵌
  可交互"（`loadable || fullstack`），web 臂 `demos-registry.ts.loadable` =
  Vue 动态挂载能力；两臂数据源分离为 enduring 决策。依据 AC-03 证据。
- **适配器 stylekit 内联（new）**：`emit_gallery_vm_demos` 把跨包
  `use stylekit.styles: ...` 确定性内联为适配器本地 `pub style` 声明（画廊
  上下文无跨包解析通道 + 装载路径无 recipe 预注册的既定事实下，适配器
  自足性为 enduring 形态）；教程/源码 tab 展示语料原文不变。依据 T-01 归因
  与 AC-05 等价性检查。
- **组件包目录级联（new）**：475 组件包（`use { package: ... from "dir" }`）
  的包目录随适配器级联拷贝进 demos/（package.at 清单跳过；包内组件 fn 模块
  链同链收集；同名异容 kept-first 告警）。依据 T-04/T-05 证据。
- **parse_package_widgets recipe 预注册义务（modify）**：475 包装载器 parse
  前必须执行 `prepare_style_recipe_imports`（与编译入口同契约）。依据
  tree_icon 修复证据。

## 8.2 第二波实测问题（2026-09-18 用户报告，rev2 扩容）

用户以 6 张截图 + 文字报告 ui-gallery VM 模式第二波问题。本会话已逐一实机
复现归因（截图证据 `u2_*.png`，构建 = 修复批次 09bb8e218）：

| # | 问题 | 实测归因 | 解决方案 |
|---|---|---|---|
| P2-008 | 008 三张价格卡特性行文字不见 | **树有 18 个 ✓ 节点、视觉只画第一行**（u2_008.png：Single Developer 仅"✓ 1 Developer"，Team/Enterprise 特性区全空）→ 渲染层丢弃，非数据缺失 | T-11：卡片 col `justify-between` + 特性块行序列的绘制丢弃归因（疑 iced col/row 溢出裁剪或行分布），修复 |
| P2-009 | 009 内容超桌面高度无滚动条 | demo 根 `min-h-screen`（app.at:252/254）在 VM 语义 = **窗口高**（900）而非视口容器高（720）→ 内容溢出 frame，frame `overflow-hidden` 裁剪且无滚动 | T-12：内嵌上下文 screen 单位语义修正（screen→内嵌容器高）+ frame scroll 兜底 |
| P2-016a | 016 打开强制全 app 变浅色 | **主题污染实锤**：016 demo `dark_mode=false` 魔法变量驱动宿主主题运行时（u2_008 深色 → u2_016 起全浅色持久）；且 016 有深色样式分支但默认浅色 | T-13：合并 VM 子件主题魔法变量与宿主主题运行时隔离（主题只认根件）；016 默认主题跟随/改深色（语料） |
| P2-016b | 016 不居中（自由尺寸 app 应居中） | 同 P2-009：demo `min-h-screen`（app.at:40）= 窗口高 → 溢出 frame → 左上对齐 | T-12 同修（screen 语义修正后 items-center 居中生效） |
| P2-017 族 | 017/018/019/021/022/023 完全不显示"暂无 VM 内嵌形态" | 三类否决：routes 单页族（018/019/021/022/023——022 注释自证"routes just render the board page"）；back 链 native-ns/stream（017）；混合（023） | T-14 分档覆盖：(a) routes 单页 stub（渲染首页路由）；(b) back 链进程内 stub/空态降级或接 T-19 proxy；(c) 030/041/043/044 保持独立 + 文案精确化 |
| P2-015 | 015 无默认数据 | db.at **有 6 条种子笔记**（Welcome/Quick Ideas...，List<Note>.new）但合并 VM 视图"No notes yet" → back 链模块级堆初始化或 api CALL 链断点 | T-15：合并 VM back 链模块级 List 初始化/调用链归因修复 |
| P2-020 | 020 曲库空（E:/Music 有 mp3） | 曲库来自 `Http.get_json("/api/media/scan")`（auto-man 生成的 Axum 后端，player_store.at:92）——**内嵌 merged 模式无该 HTTP 后端进程**，调用落空 | T-16：接 T-19 proxy（子 URL）或内嵌进程内媒体扫描适配 |
| P2-024 | 024 右侧绘图不显示 | **最新构建已出图**（u2_024.png 三系列折线完整）——用户所见为修复批次（09bb8e218）前的构建 | T-17：结案记录 + 用户以新构建复验 |
| 附带 | 027 错误 toast 跨 demo 残留（u2_008 右下角） | toast 生命周期无过期/清理 | T-18：toast 过期修复（并入 T-06 族） |

### 多后端 proxy 机制可行性分析（T-19 前置调研）

**需求**：ui-gallery 一站式内嵌 fullstack demo（015/020/017/031-image-viewer
等都需要各自后端），不能每个 demo 起独立后端进程/端口——需要**单进程多后端
宿主**：每个 app 的后端代码在同一进程内运行，按子 URL（`/apps/<id>/api/*`）
或子 domain 路由到对应后端；前端（生成的 api.ts / Http.* 调用）自动适配。

**可行性：高**。依据：
1. **会话隔离已有原型**：`auto serve` daemon（Plan 269）已是"单进程多
   stateful VM session"形态（named pipe、session 管理）——proxy 只需把
   session 前端从 pipe 换成 HTTP 路由。
2. **back 链进程内执行已通**：PLAN-633 merged 模式证明 back .at 模块可在
   宿主 VM 进程内加载执行（013/015 内嵌即此形态）——proxy 的"每 app 一个
   VM session"是同一机制的会话化扩展。
3. **HTTP 生成器已有**：auto-man 的 back→Axum 生成器（020 媒体扫描即此）
   产出路由表——proxy 复用同一生成器的路由清单做挂载，或直接以 VM session
   内 `#[api]` fn 表动态分发（merged 模式 CALL 语义的 HTTP 化）。
4. **前端适配面小**：生成器把 api baseURL 从 `:port` 换成子 URL 前缀
   （`/apps/<id>`）；VM merged 臂不走 HTTP 天然兼容；`Http.get_json` 相对
   路径调用（020 实证）按前缀展开——PLAN-617 已有 `AUTO_HTTP_BASE` 相对
   URL 展开先例（get/post/put/delete/json 五臂）。

**推荐形态**：子 URL 路径前缀（零 DNS/hosts 成本）优于子 domain（需通配
解析，远期选项）。**风险与边界**：① session 崩溃隔离——一个 app 的后端
panic 不得带倒同进程其他 app（需 per-session catch + 重启）；②
SSE/WebSocket/stream 签名后端（017/031-image-viewer 的 native-ns/stream）
转发需专项；③ binary 响应（图片/文件）转发体积；④ 端口收敛收益
（3049..3050+N → 1）与调试可观测性权衡。**工作量估计**：proxy 进程
（路由表 + session 管理 + 热挂载）2-3 天；生成器 baseURL 适配 1 天；
gallery 集成（启动 proxy + registry 注入子 URL）1 天。

- [✅] **T-11 (P2-008) 008 特性行渲染缺失**（rev2）[✅ 已完成]
      归因（headless 定案，p642_t11_008_feat_rows 复现矩阵）：iced 0.14 flex
      第三 pass 对 FillPortion 子 min=max=份额硬钉 + 列内子项按剩余量逐个
      配给——justify-between 卡片列里 flex-1 feat_list 与 2 个 FillPortion
      垫片竞争,份额<内容自然高时行序列被配给至 0×0 隐没（修复前 18/18
      特性文本零尺寸;二分矩阵:去 flex-1 或去 justify-between 任一即全出,
      桌面 1024×768 模拟器）。CSS flex grow 子有 min-content 钳制永不
      隐没,iced 无该钳制——非裁剪、非数据缺失,渲染器语义缺口。
      修复：`axis_fix_col_child_distributed`——distributed 列
      （justify-between/around/evenly）直接子剥 Flex1/FlexAuto/Grow 且不补
      Height(Full),空隙由垫片独占（与 CSS 溢出场景等价;欠额场景差异仅为
      grow 子不再撑高,垫片吸收等量空隙,视觉由 justify 分布吸收）;
      `style_is_distributed` 与 build_column 垫片口径同源;into_iced +
      render_dynamic_view 双入口同修（Plan 319）。
      实证：headless 回归 18/18 零尺寸→0/18（含非 distributed flex-1 仍
      撑满对照面）；layout_tests 54 绿（2 红=master 预存,主检出同红实证）；
      `cargo t ui` 2101 跑零新增红（23 败逐一对照主检出=预存,含 merge
      带入的 shell_pack_hash 漂移——auto-os worktree 合并 main 后
      hash-lock 四件全等复绿）；实机 VM 画廊 008 截图 18 特性行全出、按钮
      贴底（t11_008_fixed.png）,007/013/024/026/029 抽样零回归,6 次切页
      零崩溃。双仓提交：auto-lang 50f623e5c、auto-os 98613b0（产物再生成
      012-clock 改名/046 新增/013+020 随语料,合并对齐非 T-11 生成器变化）。
- [✅] **T-12 (P2-009/P2-016b) 内嵌 screen 单位语义修正**（rev2）[✅ 已完成（用户裁定②方向重交付：语料内容高 + scroll 兜底）]
      用户裁定：T-12 scroll 方案合理（浏览器同构），008 卡片"无限拉伸高"
      是语料自身缺陷——应为内容自然高（fill vs auto 之辨）。
      落地：①语料 008 卡片行去 items-stretch（卡片改自然高；三卡内容
      结构相同高度差可忽略；Vue 臂同步让渡等高语义）；②恢复 scroll
      兜底（2c96c5e40 形态原样：justify-Center/End 路径 overflow-y:
      hidden 列包 Shrink Scrollable）；③回归测试恢复 + 双窗口尺寸守卫
      （1024×800/1920×1200）。fe48a3945。
      验证：headless 双尺寸 40 行全布局 + 短内容垂直居中绿；
      layout_tests 55 绿（2 红=master 预存）；cargo t ui 2101 全跑
      22 败全落 master 基线零新增；实机 009 滚动条+完整内容
      （t12final_009.png）、016 垂直居中（t12final_016.png）复验通过。
      008 状态：真实语料 headless 720 视口探针卡片全出（探针③）；
      实机截图未及卡片区（卡片在 vtree 树中,位于 frame scroll 折叠线下
      的可能性未排除）——滚轮可达性验证留待下一会话（MCP 无框内滚动
      柄 + 实例环境高滚动率所致,非方案性阻塞）。
- [✅] **T-13 (P2-016a) 子件主题魔法变量隔离**（rev2）[✅ 已完成（①写点追踪路线，655 落地后执行）]
      定罪（8 类写点探针 AUTO_DEBUG_THEME_TRACE=dark_mode，A/B 008→016→008
      实机日志）：污染链 = 合并 VM 轨统一状态对象（Plan 419）下，016 挂载时
      **两条 VM 直写**覆写宿主壳根态 `dark_mode`——①`handler_CalendarStore_Init`
      （store 模块级 var 初始化）②匿名模块 init 帧（demo 模型 var 静态初始化）
      → 渲染器每帧状态→主题同步（renderer.rs D-GAP-2 块）翻全局。SEED
      （ensure_child_state）仅为回声非首犯；宿主壳 app.at:23 自声明
      dark_mode=true（根因=字段合并冲突，非路由 bug——独立形态同写合法）。
      v2 写端改名失败的机理同因：store var 与静态初始化不在赋值语句改名面内。
      修复（用户裁定候选"store 模块 var 前缀"完整版）：发射期**全量 α-改名**
      ——`rename_reserved_root_fields`（auto-man vue.rs，word-boundary 字节级）
      把 demo 侧（适配器+自有模块+包级联文件）的 `dark_mode`/`accent_color`
      （恰为渲染器消费的两个保留名）统一改 `<ns>_` 前缀（ns=demo_ns_prefix），
      声明/读/写一体改名语义自洽；语料原文/宿主壳/宿主 deps
      （settings_popover）不动——web 臂 demo 本就各自独立持主题态，改名对齐
      双臂语义。改名安全性预检：14 语料 app 声明面清点、零字符串字面量命中、
      零跨 demo 模块名碰撞（同名异容跳 demo 风险不存在）、package 目录单
      demo 独享。A/B 复验（修复构建）：全序列仅剩 boot 宿主合法播种
      （true→true），VM 直写/SEED/SYNC-FLIP 全消失；三截图（A 008 深/B 016
      打开宿主仍深+日历完整渲染 June 2026 网格/C 008 仍深）+ 像素级壳亮度
      分析（left≈32 dark）双重确认；boot 零 parse 失败；隔离 config 全程
      零写入（无落盘路径触发——R642-P2 历史污染为前会话手动设置交互，
      污染机制已随根态覆写消失而根除）。门禁：gallery 13/13（含新单测
      test_emit_gallery_vm_demos_reserved_field_rename）；auto-man 全套
      298 绿/1 红=master 预存 flaky（test_plan609 并行红隔离绿，master
      同特征实证；P642-D5 的 plan606 红今日已绿）；探针 8 处全摘
      （auto-lang core 与 master 零 diff）。双仓提交：auto-lang dfcc3bbe1、
      auto-os T-13 批次（16 文件，11 适配器+calendar_store+d015notes 模块
      改名映射+registry 语料漂移+A/B 脚本入 tests/）。
- [✅] **T-14 (P2-017 族) 回退页内嵌覆盖分档**（rev2）[✅ 已完成（a/c 两臂；(b) 随 P642-D13 proxy 独立计划）]
      **(a) routes 首页 stub 档**：新 `route_stub` 档（判定 = app.at 含
      `routes {}` 块且非 render:"vm"；不依赖 vp——018 的 from_workspace
      strict 失败是 Vue 装配臂问题）。发射变换（vue.rs `rewrite_routes_
      stub`）：routes 块剔除 + `outlet` 行替换为 `/` 首页路由组件实例
      （无 `/` 时首个无参路由兜底）+ 首页页面文件 per-demo ns 级联
      （pages/home.at 跨 demo 同名异容，平面名必撞 modules_conflict）。
      **三处配套挖掘（迭代三轮实证）**：①store 别名 use 行重链
      （`relink_store_use_lines`——021 `use store: BlogStore` 别名 token
      ≠声明文件名 → 收集 miss → store 不入池 → scan 空表 → 真名限定
      不跑 → 合并单元 plan-446 A1 歧义硬错）；②back 级联接入（四家
      store 数据全走 `use back.api:`——route_stub 与 fullstack 同管线
      级联；纯前端无种子 stub 跳过级联不跳 demo）；③多 store 边界
      降级（023 AuthStore+ArticleStore 泛型调用语义分属，A1 不可消解
      → 回退页，发射期记录原因）。**(c) 回退文案精确化**：AppViewport
      else 分支三行——标题（依赖说明）+ `f"独立运行：cd examples/ui/
      ${.app}"`（插值注意 AutoLang f-string 是 `${}` 非 `{}`）+ 双臂
      运行命令。registry.at loadable 三档并集（loadable||fullstack||
      route_stub——T-08 元数据=VM 实态原则延伸；web 臂 demos-registry
      .ts 不变）。**E2E**：四页 stub 首页真实渲染（022 "Project Board"/
      018 "Library"/021/019 "Home"）+ 三回退页命令插值正确（023/030/
      017）。门禁：gallery 15/15（含新 route_stub 单测）；auto-man 全套
      2 红 = 并行 flaky（plan609+generate_rust_ui_out_of_repo，双测
      隔离重跑绿、master 同特征）。双仓提交：auto-lang d19190a11、
      auto-os T-14 产物批次（4 适配器+页面/链模块+registry+AppViewport）。
- [✅] **T-15 (P2-015) 015 种子数据合并链归因**（rev2）[✅ 已完成（归因反转：数据链通，视图层双断点）]
      归因（state 工具 + 探针实证）：db.at 6 条种子**在内嵌态完整落地**根态
      （notes=6 vmref，store.Init→list_notes→db.all_notes 全链通）——任务
      预设的"back 链断点"不成立，真断点在**视图层求值**：①`.len()` 后缀
      快路径（aura_view_builder eval_condition）把 `.NotesStore.notes.len()`
      剥掉 `.len()` 后的 "NotesStore.notes" **整段当单字段名** read_state
      必 miss → 条件恒 false → 空态分支（015 "No notes yet" 实锤；026
      `.schemaIndexes.len()` 恰为裸字段故幸存）；②store 别名快照
      （VIEW_STORE_ALIAS_SNAPSHOT）为线程级单例——多组件画廊下被后续合成
      覆盖/错主，`.TodoStore.X` 真名限定读在字符串/值解析三消费点落空
      （013 footer f-string 渲染字面模板实锤；探针证实快照内容含全部
      真名自映射但条件求值路径根本不经展平臂=零 MISS）。修复：①快路径
      经 store_source_field 展平为根态裸字段；②快照随 VmBridge 存档
      （合成同线程紧邻捕获）+ 三消费点 bridge 查表兜底。复验：015
      "No notes yet" 消失、Welcome 种子可见（t15_lenfix_015.png）；013
      footer 插值 "3 items left" 正确（t15_lenfix_013.png；快照中另一
      "f\"${...}\"" 节点为教程说明字面量非缺陷）。门禁：新单测
      plan642_store_qualified_len_condition_resolves 绿；cargo t ui
      --no-fail-fast 2155/2177，22 败全落 master 基线（零新增）。
      auto-lang d70662244。
- [ ] **T-16 (P2-020) 020 媒体扫描后端接入**（rev2）[↪ 移交独立计划（用户裁定 2026-09-19）]
      `Http.get_json("/api/media/scan")` 在内嵌无后端进程（player_store.at:92
      实证）。**裁定：随 P642-D13 proxy 独立计划一并解决，不走 per-demo 窄路
      特例**；过渡期 020 内嵌保持诚实空态。本计划内不再执行。
- [✅] **T-17 (P2-024) 024 空画布结案**（rev2）[✅ 已完成（结案记录）]
      最新构建（09bb8e218+，含 P642-D1 核销与 PLAN-643 修复链）内嵌已
      完整出图（u2_024.png 三系列折线实证；本会话 soak 4 轮遍历含 024
      均正常渲染）——用户侧所见为旧构建。**结案：024 以最新构建复验
      即可**；若仍见空画布，以 `auto build` 重新生成产物后复验。
- [✅] **T-18 (P2-附带) 027 错误 toast 跨 demo 残留**（rev2）[✅ 已完成（证伪结案——触发源灭绝，无修复面）]
      实机定性（t18_repro/t18_lifecycle 脚本，端口 2303/2304 双实例）：
      ①**触发源已随 T-05 修复批次灭绝**——027 内嵌 boot 链现正确解析
      home（state: booted=true, home=C:\Users\zhaop），"无法定位主目录"不再
      触发；打开 027 停 3s 无 toast，切 008 后 +1/+5/+10s 快照均无 toast
      文本（t18_A/B1/B5/B10 截图在案）。②**生命周期机制健全**（代码走查+
      单测）：ToastReq 自带 shown_at+duration_ms（默认 4000，vue-sonner
      对齐）；push 后置 view_dirty；250ms __toast_tick 订阅（仅有 toast 时
      订阅）按 retain 清扫、移除再置 dirty；__toast state 消费即清空——
      合并轨跨 demo 无残留通道；plan412_toast_call_rewrites_to_state_assign
      单测锁定重写。③切走后 ≤4s 存留属设计内瞬态（标准 toast 语义）。
      附带发现（登记 P642-D14）：MCP fixture 派发不可达任意 handler
      （AddrGo 实证不执行、SetAddr 走 input 绑定可达）——toast 类 E2E 在
      合并轨无人造触发通道（右键菜单/鼠标区均非 MCP 可寻址）；
      fs.canonical 失败返回原路径（unwrap_or(path)，native.rs:9397）而非
      空串——语料 `can == ""` 失败哨兵分支（027 NavTo 两条 toast.error
      路径）VM 臂不可达，语义漂移小债。
- [ ] **T-19 多后端 proxy 机制**（rev2）[↪ 移交独立计划（用户裁定 2026-09-19）]
      单进程 axum 多后端宿主：per-app VM session + 子 URL `/apps/<id>/api/*`
      路由；生成器 baseURL 子前缀适配（PLAN-617 AUTO_HTTP_BASE 相对展开
      先例）。**裁定：不塞入 642——收尾后另立独立计划（P642-D13）**，范围
      明示扩大：不止 020，**017-chat / 031-image-viewer 的 native-ns/stream
      后端为一等公民目标**（stream/WS 转发从"首期风险排除项"升为立项范围）。
      T-14(b) 臂随本条目：proxy 落地前 back 链族维持回退页诚实空态。

## 8.3 新会话续作指南（T-11..T-19 冷启动手册）

**worktree/分支（续用，勿重建）**：
- 实现仓：`D:/autostack/.wt/lang-642/auto-lang`（branch `plan-642-dev`，
  修复批次 HEAD = `09bb8e218`）
- 产物仓：`D:/autostack/.wt/lang-642/auto-os`（branch `plan-642-dev`）
- 依赖兄弟：`D:/autostack/.wt/lang-642/auto-down`（detached，仅满足 cargo
  path 依赖 `autodown-core`，勿删）
- master 有其他会话并发活动——续作前先在 worktree rebase/合并最新 master。

**构建与启动（每次验证的标准形态）**：
```bash
cd D:/autostack/.wt/lang-642/auto-lang && cargo build -p auto
cd D:/autostack/.wt/lang-642/auto-os/ui-gallery
AUTO_GALLERY_APPS=D:/autostack/.wt/lang-642/auto-lang/examples/ui AUTOUI_MCP_PORT=<空闲端口> D:/autostack/.wt/lang-642/auto-lang/target/debug/auto.exe run -r vm
```
- 启动即再生成 gallery 产物（demos/*.at、AppViewport.vm.at、registry.at，
  勿手改；`auto build` 同效）。启动需 40-60s（预编译全部 demo），MCP 就绪
  以 snapshot 含 `045-style-import` 为准。
- 驱动：`.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`
  （AutoUiMcpClient(port)：snapshot/press/type_text/screenshot）；遍历矩阵
  脚本模式见 §6。截图落 `ui-gallery/src/front/tests/screenshots/`
  （已 gitignore）。

**在途/悬置事项**：
- R642-F1（间歇性原生崩溃 exit 127）blocked——待用户裁定 debt-landing vs
  继续；WER LocalDumps 已配置（HKCU ...LocalDumps/auto.exe →
  %TEMP%\p642dumps，DumpType=2；截至目前 0 dump 产生）。
- T-05 residual：027 Tick 已合成但 Tick 内嵌套 NavTo 自动首列表不完成
  （Env.get 探针实证返回值正确，故障在引导块后段）。
- T-16/T-19 立项需用户裁定（proxy 形态/工期）。
- 门禁基线：cargo tv --no-fail-fast 3745/3745；ui_gen 792/792；auto-man
  gallery 22/23（唯一红 = plan606 data-URL 断言，P642-D11 master 预存）。
- 崩溃类复现偏好：长会话（2+ 轮全遍历 + 截图）复现率 ≈1/4 实例，死亡页
  随机（029/024/空闲均出现过）。

## 9. 复审记录

```yaml
stage: review
plan_id: PLAN-642
plan_revision: 1            # 复审基线迁移补记（初版无 revision 字段）
outcome: needs_fix
reviewed_commit:
  auto-lang: ba009076f2a0553529a17b7bb5f709748e351c9d   # plan-642-dev, worktree .wt/lang-642/auto-lang, clean
  auto-os:   ddd0fe2                                     # plan-642-dev, worktree .wt/lang-642/auto-os, clean
base_commit:
  auto-lang: b9e9f6899   # 分组 worktree 创建基
  auto-os:   a6f3eb5
dependency_revisions:
  auto-down: 60b038f (detached 组内兄弟，仅满足 path 依赖)
spec_inputs:
  - docs/specs/auto-lang/ui/overview.md（registry 契约/内嵌形态节，未在本审中改写；
    规范增量草案已入本计划 §8.1，merge 时沉淀）
acceptance_results:
  AC-01: partial — 复审独立矩阵（port 2160 实例）21 VM-live 页 OK、10 设计回退
         命中清单；024 画布空、027 加载态 residual（P642-D1/D2）
  AC-02: fail — 029 二次打开复现 exit 127 静默死亡（F-1，见下）
  AC-03: pass — 013/015 侧栏角标"可交互"（复审快照 text 节点实证）
  AC-04: pass — KNOWN-DEBT P642-D1..D6 + P633-D2 核销（master e352437b0）
  AC-05: pass — 适配器 vs 语料差异仅 widget 改名 + stylekit 内联 + 空行
         （003/016/031-paint/029/045 五件抽查，参数化 pill 多行声明完整）
  AC-06: pass — cargo t ui_gen 792/792；cargo tv --no-fail-fast 3744/3745
         （唯一红 plan367 real_sidebar_at_parses_with_navtree 为 master 预存，
         本仓主检出同红实证）；008 快照 18 特性行有内容，巨字为语料设计
findings:
  - id: R642-F1
    severity: high
    affects: [AC-02, T-03]
    evidence: 复审实例（2160）全量矩阵中第二次 SelectDemo(029-photo-gallery)
      handler OK 后进程静默死亡 exit 127，死前日志仅剩 MCP 心跳行
      （ug_review.log:855-858）；执行期"单序列存活"结论不充分。
    correction: 回 worktree 归因 029 二次打开/异步照片装载渲染竞态，加防护
      （渲染失败降级），修复后复验需覆盖"重复打开 029 ≥2 次 + 矩阵全遍历"。
  - id: R642-F2
    severity: medium
    affects: [AC-01, T-04]
    evidence: 024 内嵌画布空；包组件在合并臂 `shadows builtin tag — builtin
      wins`（Plan 408/435），独立臂正常（P642-D1）。
    correction: 合并臂包组件/builtin 优先序归因与修复（或显式降级设计）。
  - id: R642-F3
    severity: medium
    affects: [AC-01, T-05]
    evidence: 027 内嵌永久"正在加载..."；独立臂真实目录 62 项（P642-D2，
      伴生 kept-first 包冲突 P642-D4）。
    correction: 合并臂 Init/异步装载链归因；包 per-demo 命名空间化候选。
  - id: R642-F4
    severity: low
    affects: [AC-06]
    evidence: plan367 real_sidebar_at_parses_with_navtree master 预存红
      （同 parse-recipe 族）。
    correction: 修复循环中按 plan370_test_support 同法补预注册（非回归，
      可并入 F-1 修复批次）。
evidence:
  - 本计划 §8 执行回填 + §8.1 偏差面 + 规范增量草案
  - 截图集 auto-os/ui-gallery/src/front/tests/screenshots/
    （inv_*/fix_*/fix2_*/fix3_*/final_*/t03_*/rv_* ，gitignore 不入库，
    结论性截图已在 §8 以文件名引用）
  - KNOWN-DEBT-AND-RISKS.md 2026-09-18 PLAN-642 段
  - 复审复现日志摘录：SelectDemo(029)→VM_HANDLER_OK→（仅心跳）→exit 127
next: needs_fix → work（worktree .wt/lang-642/ 续用）；重开 T-03/T-04/T-05
  （current_step=7）；修复后重审（plan_revision 仍 1，修复不改变语义契约）
```

（复审限制声明：本次复审在实现同会话执行，非独立角色；结论全部基于
复审期新鲜命令输出——独立矩阵重跑、tv 全量重跑、badge/029 快照、
五件适配器等价性 diff——而非执行期总结。）

---

```yaml
stage: work (needs_fix 修复循环)
plan_id: PLAN-642
plan_revision: 1
outcome: blocked          # 仅 R642-F1；F2/F3/F4 已修复并验证
code_commit:
  auto-lang: 09bb8e218 (plan-642-dev)
task_ids: [T-03, T-04, T-05, T-09]
evidence:
  - R642-F2 ✅: 合并臂适配器包注册补走查（lib.rs）——024 内嵌完整出图
    （f2b_024_recheck.png，三系列折线+轴+图例，与独立臂同形）；
    最终矩阵 024=OK
  - R642-F3 ⚠大部: 子件 Tick 合成（051 C7 条目形态，lib.rs）——027 脱离
    永久"正在加载"、Tick 派发 439 次、地址栏手动导航完整出列表
    （f3_027_navto 序列 + 最终矩阵 027=OK）；residual: Tick 内嵌套
    NavTo 自动首列表不完成（Env.get 探针实证返回值正确，故障在
    引导块后段，需下一轮归因）
  - R642-F4 ✅: real_sidebar 预存红修复 → cargo tv --no-fail-fast
    3745/3745 全绿（执行期唯一红核销）；ui_gen 792/792
  - R642-F1 ❌: 未能在修复批次内根因。修复后事实：13 轮全遍历 soak +
    1 次全矩阵存活，但最终矩阵后空闲期再次 exit 127（复现率≈1/4 长
    会话实例；死亡页随机：029/024/空闲）；WER LocalDumps（DumpType=2）
    配置后 0 dump 产生；无 panic 无 WER 记录。该崩溃类先于本计划
    （master 二进制 inv1 实证，P625 时代即登记）。
blockers:
  - R642-F1 需 crash dump 工具链（cdb/WinDbg 或 WER 诊断）定位故障模块；
    当前环境无此工具且 LocalDumps 未产生 dump
next: |
  用户裁定二选一：(a) F1 以预存间歇性原生不稳定登记 KNOWN-DEBT
  （P642-D3 已有完整证据链），本计划就 F2/F3/F4 修复面重审后 landing，
  崩溃类另立专项；(b) 保持本计划 open，配置 crash dump 工具链后继续
  F1 根因。两案皆不影响 F2/F3/F4 修复面已验证的事实。
```

---

```yaml
stage: work (rev2 波次)
plan_id: PLAN-642
plan_revision: 1
outcome: pass            # T-11 单任务;计划整体仍 executing（T-12..T-19 在途）
code_commit:
  auto-lang: 50f623e5c (plan-642-dev, 合并 master e1f79db2d 后)
  auto-os:   98613b0   (plan-642-dev, 合并 main d5e5ae3 后 + 产物再生成)
task_ids: [T-11]
evidence:
  - headless 复现/守卫:layout_tests::p642_t11_008_feat_rows_visible_in_distributed_cards
    （修复前 18/18 特性文本 0×0;修复后 0/18;非 distributed flex-1 撑满
    对照面保持——防过度剥离）
  - 归因钉死:iced_core 0.14 layout/flex.rs 第三 pass min_main=max_main=
    份额 + 列内子项 first-pass 剩余量逐个配给;二分矩阵（flex-1 ×
    justify-between 缺一不现）
  - 门禁:layout_tests 54 绿+2 master 预存红（主检出同红实证）;
    cargo t ui 2101 全跑 23 败全数对照主检出=预存;cargo check 绿
  - E2E:VM 画廊（worktree 构建,端口 2178）008 截图 18 特性行全出
    （t11_008_fixed.png）;007/013/024/026/029 抽样零回归;6 次切页+
    截图全程进程存活（F1 本会话未复现,不改变其 blocked 状态）
  - 附带修复:master 合并带入的 shell_pack_hash_parity 红——auto-os
    worktree 合并 main 后 hash-lock 校验四件全等（未动快照,方向正确）
next: T-12..T-19 续作（worktree 续用;T-16/T-19 仍待用户裁定立项）
```

---

```yaml
stage: work (rev2 波次)
plan_id: PLAN-642
plan_revision: 1
outcome: pass            # T-12 单任务;计划整体仍 executing（T-13..T-19 在途）
code_commit:
  auto-lang: 2c96c5e40 (plan-642-dev, 含 T-11 批次 50f623e5c + master 合并 e1f79db2d)
  auto-os:   无新提交   (生成器未动,产物零变化)
task_ids: [T-12]
evidence:
  - 归因修正:scratch 双变体实测(h-300 精确生效/行 13+ 框底截断)证伪
    "screen=窗口高"原判;headless 768/900/1250 三窗口 + 条件式/字面量/
    双 widget 管线全部钳 720 → 真因 = iced 0.14 定高列配给塌缩
  - 修复:apply_column_style justify-Center/End 路径 overflow-y:hidden
    列包 Shrink Scrollable(短内容居中/长内容滚动);首版未限作用域致
    画廊根 h-screen 整页塌缩,随即收窄并重验
  - headless 回归 p642_t12_overflow_frame_scrolls_and_centers 绿
  - layout_tests 55 绿(2 红=master 预存同前);cargo t ui 2101 全跑
    22 败全落 master 基线(零新增)
  - E2E:009 frame 内滚动条(t12v5_009.png)+ 016 垂直居中恢复
    (t12v5_016_clean.png);水平居中受 P2-016a light-on-light 干扰,
    留 T-13 复验
  - 环境注记:MCP 端口避开 Windows 排除区 2180-2279(改 2300);实例
    "自杀"实为 TaskStop 孤儿 + 单实例冲突(与 F1 崩溃家族区分)
next: T-13..T-19 续作;T-16/T-19 仍待用户裁定立项
```

---

```yaml
stage: work (rev2 波次,T-12 replan 循环)
plan_id: PLAN-642
plan_revision: 1
outcome: needs_replan → 已按用户裁定②重交付 (T-12);计划整体仍 executing
code_commit:
  auto-lang: fe48a3945 (plan-642-dev)
    历史:2c96c5e40(T-12 首版) → 267d76904(回退) → fe48a3945(②重交付)
  auto-os:   无新提交(生成器未动;产物再生成为 008 语料修正映射)
task_ids: [T-12]
evidence:
  - 用户裁定:scroll 方案合理(浏览器同构);卡片应为内容高(fill vs auto)
  - 008 语料去 items-stretch + scroll 兜底恢复 + 双尺寸回归守卫
  - 门禁:layout_tests 55 绿(2 红=master 预存);cargo t ui 2101 全跑
    22 败全落 master 基线
  - E2E:009/016 实机复验通过;008 headless 全出/实机待滚轮复验
    (卡片在 vtree 树中;折叠线下假设未证伪;MCP 无框内滚动柄)
  - 本轮额外发现资产:MCP 端口避开 Windows 排除区 2180-2279;
    TaskStop 孤儿 auto.exe + 单实例冲突 = 实例"自杀"真因(非 F1);
    iced Scrollable 内容臂 compression=true(Fill→内容高,009/016 为证)
next: T-13 方向已裁定=①写点追踪(655 落地后执行) → T-18 → T-15 → T-17;
  T-16/T-19/F1 仍待用户裁定
```

---

```yaml
stage: review (phase landing——wave1 + T-11 + T-12②;overall 仍 executing)
plan_id: PLAN-642
plan_revision: 1
outcome: pass (phase verdict;非终审——T-13/T-14..T-19 不在本审范围)
reviewed_commit:
  auto-lang: fe48a3945 (plan-642-dev, worktree clean)
base_commit:
  auto-lang: 1f4e3e32c (phase 前 master);phase diff = 50f623e5c(T-11)
  + e1f79db2d(master 合并) + 2c96c5e40/267d76904/fe48a3945(T-12 循环)
dependency_revisions:
  auto-os: 98613b0+ (plan-642-dev, 产物随语料同步)
  auto-down: 60b038f (detached 兄弟, path 依赖)
acceptance_results:
  - cargo tf 3615/3615 全绿(本审全新,复审全量门)
  - cargo tv 3760/3760 全绿(008 语料修正 golden)
  - cargo t ui 2101: 2079 绿/22 红 全落 master 基线(零新增)
  - layout_tests 55 绿(2 红=master 预存);auto-man gallery 22/23
    (1 红=P642-D5 master 预存)
  - E2E: 009 滚动条+完整内容(t12final_009.png);016 垂直居中
    (t12final_016.png);008 卡片 headless 全出(探针③)+live 滚轮
    可达性待复验(R642-P1)
  - wave1 AC-01..06 与 T-11 证据沿用前审记录(同基线未变)
findings:
  - R642-F1 (沿用): 间歇性 exit 127——本轮部分实例退出已查明为
    TaskStop 孤儿+单实例冲突(环境),崩溃家族死亡仍在案;F1 债处置
  - R642-P1 (new, low): 008 卡片实机滚轮可达性未复验(MCP 无框内
    滚动柄);headless 已证卡片全出
  - R642-P2 (new, info): 主题配置持久污染——修复前会话曾存
    dark_theme=false,新会话默认浅色,需设置面板手动恢复一次
next: merge(phase landing plan-642-dev → master);overall 仍 executing
  (剩余: T-13 方向裁定 / T-18 / T-15 / T-17 / T-14 / T-16+T-19 / F1)
```

---

```yaml
stage: work (rev2 波次,T-13① 写点追踪路线)
plan_id: PLAN-642
plan_revision: 1
outcome: pass            # T-13 单任务;计划整体仍 executing（T-14..T-19 在途）
code_commit:
  auto-lang: dfcc3bbe1 (plan-642-dev, 自 master 4817b51e1 重建 worktree)
  auto-os:   T-13① 批次 (plan-642-dev, 16 文件产物再生成 + A/B 脚本)
task_ids: [T-13]
evidence:
  - 探针定罪:8 类写点(SEED/VM-SET×2/RUST-WRITE/CMD-SET_THEME/SYNC-FLIP/
    CFG-HOTAPPLY/CFG-SAVE) AUTO_DEBUG_THEME_TRACE 门控;A/B 日志定罪
    首犯=handler_CalendarStore_Init+匿名模块 init 两条 SET_FIELD 直写;
    SEED 为回声;独立形态对照(016 standalone,store var 落自有根态=合法)
    证明冲突仅在合并边界
  - 修复:rename_reserved_root_fields 发射期全量 α-改名(声明/读/写/
    包级联一体,<ns>_ 前缀);三接线点(适配器 6317/row_modules 统一单遍/
    package 文件);语料原文与宿主壳/deps 不动
  - A/B 复验:修复构建全序列仅剩 boot 合法播种(true→true),直写/SEED/
    SYNC-FLIP 全消失;三截图视觉+壳亮度分析(全 DARK);016 本体完整渲染;
    boot 零 parse 失败;隔离 config 零写入
  - 门禁:gallery 13/13(含新单测 reserved_field_rename);auto-man 全套
    1 红=master 预存 flaky(plan609 并行红隔离绿,master 同特征);探针
    8 处全摘(auto-lang core 与 master 零 diff,唯一改动=auto-man vue.rs)
next: T-18(toast 过期) → T-15 → T-17;T-14 分档覆盖与 T-16+T-19 仍待
  用户裁定立项;R642-F1 崩溃族 blocked 不变(本轮启动期遇 1 次 exit 127
  重试即过,长会话复现率特征不变)
```

---

```yaml
stage: ruling (两项待裁定事项落定——用户 2026-09-19)
plan_id: PLAN-642
plan_revision: 1
rulings:
  - item: T-16+T-19 多后端 proxy 立项
    decision: 独立立项,不塞入 642——642 收尾后另立计划(KNOWN-DEBT P642-D13)
    scope_note: 用户明示范围扩大——不止 020,017-chat/031-image-viewer 的
      native-ns/stream 后端为一等公民目标(stream/WS 转发入立项范围,
      非首期排除项);T-16 随之移交(不走 per-demo 窄路特例);T-14(b) 臂
      过渡期维持回退页诚实空态
  - item: R642-F1 崩溃族处置
    decision: 方案 a——预存间歇性原生不稳定立足 P642-D3,642 修复面照常
      收口(不为 F1 押后 landing),崩溃根因另立专项
    immediate_step: exit(127) 主动退出点全仓枚举 + AUTO_DEBUG_EXIT_TRACE
      门控追踪(POLLTRACE 先例形态)——判定死亡是否混有未被识破的主动
      退出路径;本会话执行,结果记入 P642-D3
    result: 枚举判定完成(2026-09-19)——全仓零 deliberate exit(127),代码
      侧无可 trace 点,转向 crashprobe 退出码映射控制实验:F1 的 bash 127
      = fastfail/栈溢出(AV 报 139,panic 报 101,均排除);审计日志 959 行
      零 F1 时刻记录吻合"绕过 panic 钩子"死法;另录 wgpu offscreen 纹理
      与 min>max 布局两条 panic 线索;修复构建 soak 4 轮全遍历存活零审计
      新增。详见 P642-D3 枚举小步执行结果段
queue_after_ruling: T-18 → T-15 → T-17 (+T-14 a/c 可选,既定授权面内)
  → 复审 → merge 收口;proxy 独立计划与崩溃专项另行立项
```

---

```yaml
stage: work (rev2 波次,T-18 证伪结案)
plan_id: PLAN-642
plan_revision: 1
outcome: pass            # T-18 单任务,定性=无修复面;计划整体仍 executing
code_commit:
  auto-lang: 无代码改动（零 diff）
  auto-os:   T-18 验证脚本批次（t18_repro/t18_lifecycle 入 tests/）
task_ids: [T-18]
evidence:
  - 触发源灭绝:027 内嵌 boot 链正确解析 home（state booted=true
    home=C:\Users\zhaop）,"无法定位主目录"不再触发;027 开 3s 无 toast,
    切 008 后 +1/+5/+10s 快照零 toast 文本（t18_A/B1/B5/B10）
  - 机制健全:ToastReq shown_at+duration(4000ms) + __toast_tick 订阅
    retain 清扫 + __toast 消费即清空;重写单测在案（plan412）
  - 附带发现两条登记 P642-D14:fixture 派发不可达任意 handler（AddrGo
    实证）;fs.canonical 失败返原路径使 can=="" 哨兵分支 VM 臂不可达
next: T-15（015 种子数据合并链归因）→ T-17（024 结案）→ 复审
```

---

```yaml
stage: work (rev2 波次,T-15 归因反转 + T-17 结案)
plan_id: PLAN-642
plan_revision: 1
outcome: pass            # T-15/T-17 两任务;计划整体仍 executing（T-14 a/c 可选在途）
code_commit:
  auto-lang: d70662244 (plan-642-dev;T-17 无代码改动)
task_ids: [T-15, T-17]
evidence:
  - T-15 归因反转:state 工具证实种子数据全链落地根态(notes=6 vmref);
    真断点=视图层——.len() 快路径整段字段名 miss + store 别名快照
    线程级单例被多组件合成覆盖(015 No notes yet/013 footer 字面模板双实锤)
  - 修复:len() 快路径 store 展平 + 快照随 VmBridge 存档 + 三消费点
    bridge 兜底;E2E 015 Welcome 可见/013 footer "3 items left" 插值正确
  - 门禁:plan642_store_qualified_len_condition_resolves 新单测绿;
    cargo t ui --no-fail-fast 22 败全落 master 基线(零新增)
  - T-17:024 最新构建出图实证在案(u2_024 + 本会话 soak 4 轮),
    结案=用户以新构建复验
next: 剩余 T-14 a/c(可选,授权面内) → 复审 → merge 收口
```

---

```yaml
stage: work (rev2 波次,T-14 a/c 完成——计划执行面收官)
plan_id: PLAN-642
plan_revision: 1
outcome: pass            # T-14 a/c;全部执行任务完成,计划待复审收口
code_commit:
  auto-lang: d19190a11 (plan-642-dev, auto-man vue.rs +319/-14)
  auto-os:   T-14 产物批次 (plan-642-dev, 4 stub 适配器+页面/链模块+
    registry+AppViewport 文案)
task_ids: [T-14]
evidence:
  - (a) route_stub 档:rewrite_routes_stub(routes 剔除/outlet→首页组件)+
    页面 ns 级联 + relink_store_use_lines(别名 use 重链,021 实证) +
    back 级联接入(四家 store 全走 back.api) + 多 store 边界降级(023)
  - (c) 回退文案三行:依赖说明 + cd 命令 ${.app} 插值 + 双臂运行命令
  - E2E:022"Project Board"/018"Library"/021/019"Home" 四页 stub 真实
    渲染;023/030/017 回退页命令插值正确
  - 门禁:gallery 15/15(新 route_stub 单测);auto-man 全套 2 红并行
    flaky(隔离重跑绿,master 同特征,零新增)
remaining: 无执行任务——T-16/T-19 移交 P642-D13 独立计划,T-14(b) 随之;
  全部在途任务(T-01..T-18)完成或定性结案
next: /auto-plan:review 复审 → merge 收口（landing 后按 P642-D13 立
  proxy 独立计划、按 P642-D3 裁定立崩溃专项）
```

---

```yaml
stage: review (终审——执行面收官后收口复审)
plan_id: PLAN-642
plan_revision: 1
outcome: pass           # 附 registered deviations（AC-02 裁定债 / AC-06
                        # Vue 臂预存红，均非本计划 diff 交集）
reviewed_commit:
  auto-lang: 17f2ab8b3 (plan-642-dev, worktree clean; 实现链 dfcc3bbe1
    T-13 + d70662244 T-15 + d19190a11 T-14 + 17f2ab8b3 spec 终稿)
  auto-os:   8d376c6 (plan-642-dev, T-14 产物批次 head)
base_commit:
  auto-lang: 4817b51e1 (worktree 重建基,含 wave1 phase landing 链)
  auto-os:   98613b0 (T-11 批次;本 phase 产物链 8b75d9f..8d376c6)
dependency_revisions:
  auto-down: 60b038f (detached 组内兄弟, path 依赖)
spec_inputs:
  - docs/specs/auto-lang/ui/overview.md §ui-gallery VM 内嵌健康度契约
    （收口终稿 7→10 条已落 worktree 17f2ab8b3,merge 时随行发布）
acceptance_results:
  AC-01: pass — 复审独立抽查 13 页全 EMBED 正常（stub 018/019/021/022
    首页真实渲染;015 Welcome 种子;013 footer "3 items left" 插值;024
    图表/027 目录列表(修正驱动 stale-id 假阴后)/045/031-paint/016/020）;
    012 banner = P642-D6 语料既定（在案偏差,非 VM 缺陷）
  AC-02: registered-deviation — F1 崩溃族按用户裁定(a)立足 P642-D3
    （预存 P625 时代;本会话收窄 bash127=栈溢出/fastfail+审计判读法+soak
    脚本常驻;029 序列 soak 4 轮存活）
  AC-03: pass — registry 三档 loadable（T-08+T-14）;022 侧栏角标"可交互"
    实态复验
  AC-04: pass — 11 回退页全部归因:4 页升级 stub/3 页精确文案（独立复验
    插值）/其余 T-09 归因行 + D13 移交在案
  AC-05: pass — α-改名 = 发射器确定性变换族（AC-05 修订版契约）
  AC-06: partial-registered — cargo tf 3640/3641（1 红 display_family_
    codegen_arm_fixture = 并行 flaky,worktree/master 隔离重跑双绿）;
    gallery 15/15;ui 档 22 红=master 基线;008 有结论（T-07 证伪+T-11
    根修+PLAN-655 StretchLine 落地）;**Vue 臂构建预存红**（folder-music
    × lucide-vue-next 0.312.0 无导出;时间线归因:语料 a5e26d558 09-17
    早于 wave1 T-10 09-18,与本计划 diff 零交集）→ R642-R5 债,非回归
findings:
  - id: R642-R5
    severity: medium
    affects: [AC-06]
    evidence: vue 臂 vite 构建 NavSidebar.vue "FolderMusic" is not exported
      by lucide-vue-next@0.312.0;语料 icon folder-music 由 a5e26d558
      （09-17,早于 T-10）引入;本计划 diff 不触 vue 转译/icon 映射
    correction: 语料换图标名或升级 lucide lock（归 auto-os/语料域;登记
      债册 P642-D15）
  - id: R642-R6
    severity: low
    affects: [AC-06]
    evidence: 并行 flaky 家族三成员（plan609_unresolved_dep_import_guard/
      generate_rust_ui_out_of_repo/test_display_family_codegen_arm_fixture
      ）——全套并行红、隔离绿,master 同特征
    correction: 并行 tempdir/资源竞争排查（环境债,登记 P642-D15）
  - id: R642-R7
    severity: info
    evidence: P642-D12 近期臂（EqualHeightRow 两遍测量）已由 PLAN-655
      StretchLine 落地交付（master 契约节在案）
    correction: merge 时债册 D12 标记近期臂核销（远期 iced 升级补丁臂留）
  - id: R642-R8
    severity: info
    affects: [验证方法论]
    evidence: 复审驱动脚本复用过期 vnode id → press 误开宿主设置弹窗,
      027 首验假阴;fresh-id 重 press 即真
    correction: MCP press 前必须重 snapshot 取新 id（驱动脚本纪律,已注
      记于 tests/ 脚本使用面）
  - id: R642-R9
    severity: info
    evidence: 画廊 boot 噪音 "dependency '' is materialized at deps but
      not declared in pac.at" ×N（wave1 起各会话均在,auto-os 工作区卫生）
    correction: 归 auto-os 域清理（不阻塞,随 D15 登记）
evidence:
  - 复审期新鲜命令输出:tf 全量/E2E 独立矩阵（2320 实例）/isolated 重跑
    对照/vue build 归因链/角标快照——非执行期总结复用
  - 执行期同会话同 binary 的 E2E（T-13 A/B 三截图/T-15 双页/T-14 四页+
    三回退）与复审矩阵基线一致（worktree HEAD 未变）,声明复用理由成立
  - 债册 P642-D1..D14 状态核对:D1(643 核销)/D7-D9(T-11/12/13 核销)/
    D12(近期臂 655 交付)/D3+D10+D13(裁定在案)/D2+D4+D5+D6+D11+D14(开放
    债,均已归因)
  - 唯一工作树脏文件（rust-workspace 生成物）已 checkout 还原——审基
    全提交（17f2ab8b3）
next: merge（plan-642-dev → master 终landing;spec 终稿随行;D12 近期臂
  核销标记 + D15 登记）;landing 后按裁定立 P642-D13 proxy 独立计划与
  P642-D3 崩溃专项
```

（终审限制声明：本复审与执行同会话执行，非独立角色；全部结论基于复审期
新鲜命令输出——tf 全量重跑、2320 实例独立 E2E 矩阵、隔离重跑对照、vue
build 归因链、角标/快照复验——而非执行期总结。）

---

```yaml
stage: merge (phase landing 收据——非归档;overall 计划保持 executing)
plan_id: PLAN-642:r1
outcome: pass (phase landing)
checkpoints:
  prepared: reviewed fe48a3945 + spec delta 落 worktree
  landed: master merge 1039d998e（含 fe48a3945 全链）+ 7375cd027(spec 节)
    经并发 PLAN-074 merge ffe2dac6d 一并入主;master cargo check 绿
  ledger_refreshed: .autoos/specs.json P642-1 upsert(读回验证,gitignore
    运行时数据就地发布);docs/specs/INDEX.md 重算无变化;spec 节已入主
  archived: N/A——分阶段落地,overall 保持 executing(T-13/T-14..T-19 在途)
  cleaned: wt-guard clean(024-charts/deps/stylekit junction 系 auto run
    产物,已按规程 os.rmdir 摘除)→ worktree 移除 + branch plan-642-dev
    删除(was 7375cd027,已含于 master)→ 组目录 .wt/lang-642 移除
master_wip_note: master 存在他会话未提交 WIP(examples/rust-workspace/
  Cargo.toml 加 013/015-back members + docs/plans/evidence/653/)——非本
  计划产物,未纳入落地,已表面化待其属主路由
next: 续作须重建 worktree(git worktree add D:/autostack/.wt/lang-642/
  auto-lang -b plan-642-dev,自最新 master);队列=T-13①(655 落地后)/
  T-18/T-15/T-17;计划外队列=PLAN-655 执行(另一 agent 进行中)/T-14/
  T-16+T-19/F1
```




## 10. 待澄清事项

- 024/027 的 `codegen validation failed (strict mode)` 只在 from_workspace
  （web 装配）出现而 corpus golden 全绿——该 strict 校验的职责边界是否本身
  有 bug，T-04/T-05 顺带给出结论。
- 族 A 修复若牵动 parser 全局 name-check 语义，需评估对 aavm/VM 语料的
  级联（触发 `cargo tv` 档）。
