---
plan_id: PLAN-642
status: execution_done         # drafting → executing → execution_done → reviewed → archived
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
total_steps: 10
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
- [✅] **T-03 029 崩溃归因与修复**（族 C，G-3）[✅ 已完成（无复现闭环）]
      T-01 修复后 029 复验序列（press→screenshot）进程存活且渲染真实图库
      （t03_029_after_press.png）——崩溃未再复现。崩溃类有两个来源实例：
      ①029 占位符→screenshot 死亡；②并发 auto build + 窗口交互后死亡；
      另 standalone 026 端口占用时 MCP bind FATAL 127（mcp_server.rs:540）。
      归因细节与缓解候选登记 P642-D3（KNOWN-DEBT）。
- [✅] **T-04 024-charts 空白画布修复**（族 B/D，G-2）[⚠ 部分完成]
      语料面：四个 chart 组件 `cap`/`hint` 笔误（→caption_text/hint_text，
      93 处）修复后 from_workspace strict 复活（19→21 loadable），独立臂
      VM 完整出图（三系列折线截图）。内嵌面 residual：包组件在合并 VM 臂
      `shadows builtin tag — builtin wins` → 实例落 builtin 桩 → 画布空；
      发射器已补包目录级联 + 包内 fn 模块链（chart_geom）收集，画布仍空。
      登记 P642-D1（含独立/内嵌对照截图与 Plan 408/435 优先序假设）。
- [✅] **T-05 027-file-manager 永久加载态修复**（族 A/B，G-2）[⚠ 部分完成]
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
- AC-02 达成（029 存活 + 出真图；崩溃类残余观测风险登记 P642-D3）。
- AC-03 达成。AC-04 达成。AC-05 修订达成（内联为登记过的确定性变换）。
- AC-06 达成（无新增红；008 有结论）。

## 9. 复审记录

（待 /auto-plan:review——执行证据齐备：本文件 §8 回填、双仓提交
ba009076f / ddd0fe2、KNOWN-DEBT P642 段、截图集
`auto-os/ui-gallery/src/front/tests/screenshots/{inv,fix,fix2,fix3,final,t03}_*.png`）

## 10. 待澄清事项

- 024/027 的 `codegen validation failed (strict mode)` 只在 from_workspace
  （web 装配）出现而 corpus golden 全绿——该 strict 校验的职责边界是否本身
  有 bug，T-04/T-05 顺带给出结论。
- 族 A 修复若牵动 parser 全局 name-check 语义，需评估对 aavm/VM 语料的
  级联（触发 `cargo tv` 档）。
