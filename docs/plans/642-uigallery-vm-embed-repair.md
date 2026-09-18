---
plan_id: PLAN-642
status: executing             # 复审 needs_fix → 由 execution_done 回退（R642-1）
plan_revision: 1              # 复审基线迁移：初版契约无 revision 字段，按 auto-plan-new 规约补记为 1
feature_name: uigallery-vm-embed-repair
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [docs/specs/auto-lang/ui/overview.md#ui-gallery-vm-内嵌健康度（PLAN-642）]
touched_goals: [GOAL-010]

affects: [auto-lang/ui, auto-lang/parser, auto-man, parity]
current_step: 10
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

- [ ] **T-11 (P2-008) 008 特性行渲染缺失**（rev2）
      树有 18 ✓ 节点视觉只画 1 行（u2_008.png 实证）——渲染层丢弃归因
      （疑 justify-between 卡片 col 行分布/裁剪），修复 + 截图。
- [ ] **T-12 (P2-009/P2-016b) 内嵌 screen 单位语义修正**（rev2）
      `min-h-screen`/`h-screen` 在内嵌上下文 = 窗口高而非容器高（009 溢出
      被裁、016 不居中同根因，app.at:252/40 实证）——VM 渲染器对
      screen 类的解析在内嵌装配时改映射容器高，frame 加 scroll 兜底；
      009 出滚动条、016 居中截图验收。
- [ ] **T-13 (P2-016a) 子件主题魔法变量隔离**（rev2）
      016 的 `dark_mode=false` 驱动宿主主题运行时（u2_008 深 → u2_016 起
      全浅色持久实证）——合并 VM 子件主题魔法变量与宿主隔离（主题只认
      根件）；016 默认主题语料评估（跟随宿主或改深色，其有深色样式分支）。
- [ ] **T-14 (P2-017 族) 回退页内嵌覆盖分档**（rev2）
      (a) routes 单页族（018/019/021/022/023）：VM 内嵌支持 `routes {}`
      首页路由 stub 渲染；(b) back 链 native-ns/stream 族（017）：
      进程内 stub/诚实空态降级或接 T-19 proxy；(c) 030/041/043/044 保持
      独立、回退文案精确化（列独立运行命令）。
- [ ] **T-15 (P2-015) 015 种子数据合并链归因**（rev2）
      db.at 有 6 条种子（List<Note>.new）但内嵌视图"No notes yet"——
      合并 VM back 链模块级堆初始化/`api.list_notes` 调用链断点归因修复。
- [ ] **T-16 (P2-020) 020 媒体扫描后端接入**（rev2，依赖 T-19 或独立挂载）
      `Http.get_json("/api/media/scan")` 在内嵌无后端进程（player_store.at:92
      实证）——经 T-19 proxy 子 URL 或内嵌进程内扫描适配后出曲库。
- [ ] **T-17 (P2-024) 024 空画布结案**（rev2）
      最新构建（09bb8e218+）已出图（u2_024.png 实证）——用户侧为旧构建；
      记录结案 + 请用户以新构建复验。
- [ ] **T-18 (P2-附带) 027 错误 toast 跨 demo 残留**（rev2）
      u2_008 右下角残留 027 的"无法定位主目录"toast——toast 过期/清理
      修复（与 T-06 同族，14263 toast 修正先例）。
- [ ] **T-19 多后端 proxy 机制**（rev2，可行性分析见 §8.2，立项待用户裁定）
      单进程 axum 多后端宿主：per-app VM session + 子 URL `/apps/<id>/api/*`
      路由；生成器 baseURL 子前缀适配（PLAN-617 AUTO_HTTP_BASE 相对展开
      先例）；风险项：session 崩溃隔离、stream/WS 转发、binary 响应。

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
  继续；WER LocalDumps 已配置（HKCU ...LocalDumpsuto.exe →
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


## 10. 待澄清事项

- 024/027 的 `codegen validation failed (strict mode)` 只在 from_workspace
  （web 装配）出现而 corpus golden 全绿——该 strict 校验的职责边界是否本身
  有 bug，T-04/T-05 顺带给出结论。
- 族 A 修复若牵动 parser 全局 name-check 语义，需评估对 aavm/VM 语料的
  级联（触发 `cargo tv` 档）。
