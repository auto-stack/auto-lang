---
plan_id: PLAN-465
status: reviewed
feature_name: Vue 虚拟桌面——Web / tauri 宿主（M4）
author: [zcode]
created_at: 2026-08-28T00:00:00+08:00
updated_at: 2026-08-29T10:30:00+08:00

supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 修改——桌面线状态推进（465 落地：vue 页面级虚拟桌面、WM DOM 叶）；virtual_window/taskbar 登记源（schema/aura.at）现含 vue 端映射（@/wm/*），与 iced 实现同源"
  - "docs/specs/auto-man/project.md: 修改——vue/tauri 管线新增 desktop 宿主 scaffold 段（auto run --desktop/--apps + AUTO_DESKTOP env）；pkg add_packages dev 臂 --dev→-D（pnpm v11 兼容）；tauri conf 桌面全屏窗口"
new_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——桌面 vue 宿主运行时（auto-man assets/wm：store.ts WmStore + layout.ts 463 布局 TS 直译 + keyboard.ts R12 热键捕获段 + VirtualWindow/Taskbar.vue DOM 叶；E1 (AppId,event) 注入形状与 E2 AppWindow 叶枚举成文 reports/465-t4-wm-dom-leaf.md）"
  - "docs/specs/auto-lang/ui/overview.md: 新增组件——I6 布局共享期望值表 layout_cases.json（双端消费：ui/layout.rs 测试 + scripts/ui-layout-parity.mjs，17 例）"
  - "docs/specs/auto-man/project.md: 新增组件——desktop 宿主 scaffold（generate_desktop_host：scan_apps render:vue 过滤 → src/apps/<id>/App.vue 预生成 + src/apps-registry.ts 静态 import 注册表 + src/components|stores 合并 + shadcn/npm 依赖 union）；examples/desktop-host E2E 载体项目"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——Web 端（vue/tauri）虚拟桌面落地，双端（iced/vue）同源登记与 I6 布局对拍闭环（M4）"

current_step: 8
total_steps: 8
---

# Plan 465: Vue 虚拟桌面（Web / tauri 宿主，M4）

> **状态**：reviewed（2026-08-29 复审通过，待 /auto-plan:merge）
> **来源**：产品需求「平台扩展：支持 WEB 平台（vue/tauri）」（Design 24 §1 N2）；
> 里程碑 M4（Design 23 §6 提案编号 456，实际编号经程序跟踪文件解析为本号）。
> **架构依据**：Design 23（R2 Web=DOM 子树、R4 接缝「DOM 节点」叶、R5「Web 永远是
> A 形态」、I4）；Design 24（§4 平台矩阵、R8/R9/R10/R12、I5/I6）。
> **依赖**: 462 的 widget 契约（schema/aura.at 登记；**可与 463/464 并行**，463 的
> 布局参数与接缝语义合入后对齐）。**基线**: 462 合入后的 master。

## 1. 目标

把同一桌面体验带到 Web：**一个 vue 页面 = 一个虚拟桌面**（DOM 子树虚拟窗口），
消费 462 登记的 `virtual_window`/`taskbar` widget 契约与 463/464 的 shell/launcher：

1. **宿主项目模式**：auto-man 新 scaffold 模式（`auto run --desktop` 或 pac
   `render:"desktop"`，T1 定名）——生成 shell 宿主页 + 构建期应用注册表；
2. **多 App 多挂载**：每虚拟窗口一个 `createApp(AppSFC).mount(container)`（组件实例
   级隔离 ≈ AppSession 对应物）；
3. **WM 的 vue 叶**（R4 第四个叶）：`virtual_window` 的 a2vue/DOM 实现——absolute
   定位 + clip + pointer 事件区域路由 + 焦点/键盘路由；
4. **launcher 复用**：`examples/ui/028-launcher` 的 vue 形态在宿主页 overlay 运行
   （I5 复验）；启动 = 注册表动态 import + 挂新容器；
5. **tauri 壳**：宿主页以全屏 webview 打包（复用现有 tauri 脚手架管线）；
   系统级召唤热键（tauri-plugin-global-shortcut）列可选。

**非目标**：iframe 隔离（R5 的可选档，后续）；BroadcastChannel 跨页总线（v1 单页内
无此需求，宿主↔launcher 经 props/事件直连；跨窗口协作留 456 原案后续）；每窗独立
主题（页面级主题 v1 全局一份）；路线 B。

## 2. 关键事实（vue 管线盘点，2026-08-28）

- **代码生成**：`crates/auto-lang/src/ui_gen/vue.rs`（22.4k 行）一 widget 一 SFC；
  `dyn` → `<component :is>`、`teleport` → `<Teleport>`；金样
  `crates/auto-lang/test/a2vue/*`、能力锁 `tests/vue_capabilities.rs`。
- **脚手架**：`crates/auto-man/src/vue.rs`（workspace 模式权威）——`index.html`
  单 `#app`（~L899）、`main.ts` 单 `createApp(App).mount('#app')`（~L1004）、
  App.vue = 根 widget SFC 原样（`generate_app_vue` L1024，**天然是可嵌套子树**）；
  vite 端口/代理（`generate_vite_config` L657）；`run_vue_project` L3892
  （增量编译→scaffold→pnpm→vite dev）。`App.vue` 根元素 class 来自 .at 源
  （`w-full h-full`），**子树假设成立**。
- **多挂载先例**：shadcn chart tooltip 已用命令式 `createApp(...).mount(div)`
  （`crates/auto-man/assets/shadcn-ui/chart/ChartCrosshair.vue:6,31`）——本计划的
  挂载模型有在产先例。
- **动态加载先例**：router 懒加载 `() => import('@/pages/X.vue')`（auto-man vue.rs
  1924/1930）——registry 动态 import 同型。
- **tauri 管线**：`crates/auto-man/src/tauri.rs` `run_tauri_project`（L29 起，
  vue 生成→tauri init→`tauri dev`，`gen/front/vue/src-tauri/`）；CLI
  `--render tauri`（crates/auto/src/main.rs:347）。
- **页面级假设清单**（虚拟窗口容器的改造点）：
  ① `modal` → `fixed inset-0`（ui_gen/vue.rs:6310，逃逸容器除非窗口 div 建立
  containing block）；② `teleport(to:"body")` 逃到 document.body（L3599/4057）；
  ③ `.window/.document` 全局监听（~L4253）；④ 主题页面级（`html.dark` +
  `documentElement` CSS 变量 + `window.__AUTO_UI_*__`）。
- **验证设施**：Playwright 驱动已起 vite 页（`test_vue_playwright.mjs`，
  `--actions` 支持 click/fill/press/screenshot）——多窗口 = 页内选择器断言。
- **BroadcastChannel**：仓库 0 命中（456 原案词汇，本批不做）。

## 3. 设计要点与决策点

### 3.1 宿主形态（T1 定案）

- **宿主页**：新的 `App.vue`（shell 宿主，不是某个业务 App）：背景层 + 窗口 z-stack
  容器 + taskbar + overlay（launcher 挂载点）——与 462 `view_desktop_fn` desktop
  模式同构的 vue 版；shell 布局声明尽量由 463 的 `shell.at` 转译而来（R8/I5），
  不足部分以宿主模板补齐并登记到对拍清单。
- **registry**：构建期扫描 apps 目录（复用 463 `scan_apps` 的清单逻辑，rust 侧生成
  `src/apps-registry.ts`）：`{ name: { title, icon, category, load: () => import(ENTRY) } }`；
  动态 import 需要静态可分析路径集合，故 registry 必须构建期生成（运行时拼路径
  vite 不支持）。
- **挂载语义**：`launch(name)` → 取 registry.load() → 新虚拟窗容器 div →
  `createApp(comp).mount(div)`；实例句柄表 ≈ `WmState`（Wid 分配、z、焦点、矩形
  ——TS 侧 `WmStore`，reactive）。关闭 = `app.unmount()` + 容器移除（对应 459
  「窗关 App 随之退」）。

### 3.2 WM 的 vue 叶（消费 462 契约）

- `virtual_window`/`taskbar` 的 DOM 实现进 a2vue 生成体系（I4：同一登记源出两端
  实现，金样 `test/a2vue/virtual_window*` 补齐）；WM 语义（拖拽/snap/hit-test/键盘
  路由）以 462 的 iced 实现为规范：
  - 拖拽/resize：pointer events（setPointerCapture）改 `WmStore.rect`；
  - 事件区域路由：容器 pointerdown 命中测试 → 置焦点/阻断穿透（与 iced 吞噬语义
    对齐的测试项）；
  - 键盘：`document keydown` 捕获段做桌面热键（R12：Ctrl+Space/Alt+Tab/布局切换），
    未捕获键派发给焦点窗口（DOM 焦点天然在窗内元素，`Tab` 循环限制在焦点窗内
    ——v1 用容器级 `keydown` + `Inert`/tabindex 策略，T2 实测定案）。
- 布局：463 布局规范的 TS 直译（grid/master-stack/snap 纯函数）+ 同一单测断言集
  （I6 对拍：矩形期望值表共享）。
- **E1/E2 接缝语义（I1 评审产出，`reports/462-i1-seam-review.md`）**：DOM 叶是
  "叶子不在宿主 widget 树内"的第一个真实消费者——本计划落地时把事件路由按
  **E1 的 `(AppId, event)` 注入形状**实现（命中矩形 → 归属 App → 扇出），并把
  **E2 的 `AppWindow` 叶子形态枚举**（Element | RenderCommand | Wayland | DOM）
  写进 WM 语义规范；386（路线 B RenderCommand 叶）复活时照抄该规范，
  无需再设计（见 I1 报告 §3）。

### 3.3 页面级假设处置（v1 收敛）

| 假设 | v1 处置 |
|---|---|
| `modal fixed inset-0` | 窗口容器 div 设 `transform: translateZ(0)`/`contain: layout` 建立 containing block（fixed 相对窗口收敛）；T2 spike 验证 shadcn dialog 实际表现 |
| `teleport to:"body"` | v1 限制：桌面内运行的 App 禁用 body teleport（登记清单+启动时警告占位），不做 portal 改写 |
| `.window/.document` 监听 | 事件经容器冒泡天然带窗口路径；全局监听 v1 登记为「可能跨窗触发」已知限制 |
| 主题页面级 | v1 全桌面单主题（`index.html` 既有机制），每窗主题列后续 |

### 3.4 tauri 壳

`run_tauri_project` 打包宿主页：tauri.conf 窗口 `fullscreen: true`/kiosk、devUrl
指向宿主 vite；可选任务：`tauri-plugin-global-shortcut` 注册系统级 Ctrl+Space
转发 `SummonLauncher`（窗口未聚焦时也能召唤——这是 vue 端唯一能做系统级热键的位置，
R12 的 tauri 增强臂）。

## 4. 任务表

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | 宿主 scaffold 施工图 | 宿主模式命名/触发（`--desktop` vs `render:"desktop"`）、registry 生成器位置（auto-man vue.rs）、shell 来源（shell.at 转译 vs 模板）定案；报告 `reports/465-t1-host-blueprint.md` | 评审通过 |[✅ 已完成] `docs/plans/reports/465-t1-host-blueprint.md` 定案三决策：`auto run --desktop`+env 注入、registry 落 auto-man vue.rs（复用 scan_apps render:"vue" 过滤）、宿主 App.vue 内置模板 1:1 镜像 shell.at（差异表登记，正式评审由 /auto-plan:review 承载）
| T2 | containment spike | §3.3 表格四项实测（modal/teleport/监听/主题）+ DOM 焦点策略定案 | spike 记录 + 截图入报告 |[✅ 已完成] `docs/plans/reports/465-t2-containment-spike.md` + 4 截图：dialog/teleport 经 reka Portal DOM 重挂 BODY（CSS containing block 无效，限制清单）；document 监听跨窗双涨（热键走捕获段）；单主题证实；Tab 篱笆定案（捕获段+手动循环焦点，不用 inert）；双 createApp 多挂载独立运行证实
| T3 | registry 生成器 | 构建期扫 apps 目录 → `src/apps-registry.ts`（动态 import 映射） | 单测：生成物含全部 vm/vue 兼容 App；vite 构建通过 |[✅ 已完成] auto-man vue.rs `generate_desktop_host`：26 个 render:"vue" App 扫描 → 20 入册（api/router/ext/i18n 越界白名单跳过）；静态 import 映射 `src/apps-registry.ts`；子组件/stores/npm deps/shadcn 组件 union 落宿主；单测 desktop_registry_maps_vue_apps_only 绿；宿主项目 vite build 2146 模块 2.50s 通过（examples/desktop-host E2E 载体入库）
| T4 | WmStore + virtual_window DOM 叶 | `virtual_window` a2vue 实现 + WmStore（rect/z/focus）+ 拖拽/resize/hit-test + 布局 TS 直译；事件路由按 **E1 `(AppId, event)` 注入形状**、`AppWindow` 叶子枚举入 WM 语义规范（E2，消费 I1 评审报告） | a2vue 金样（I4）+ 布局对拍单测（与 463 期望值表共享） |[✅ 已完成] schema/aura.at 登记 virtual_window/taskbar(vue:@/wm/*)+registry spec+a2vue 金样 test_a2vue_virtual_window 绿；E1/E2 成文 reports/465-t4-wm-dom-leaf.md；assets/wm 五件(store/layout/keyboard/两叶)经 rust_embed 落宿主；layout_cases.json 共享表 17 例双端绿(Rust ui-iced + node scripts/ui-layout-parity.mjs)；已知缺口：a2vue 组件路径任意 prop 直通(宿主叶子自读 store 不依赖,KNOWN-DEBT 候选)
| T5 | 多挂载生命周期 | launch（动态 import+createApp+容器挂载）/close（unmount）/panic 兜底（errorCaptured 崩溃页 ≈ 459 崩溃页语义） | Playwright：连续启动 ≥3 个 vue App、关闭回收、崩溃页隔离 |[✅ 已完成] 宿主 shell 升级 WmStore z-stack（launch 动态 import → attachClient ref 回调挂载；close=unmount+容器移除+焦点让渡；errorHandler→crashed 崩溃页）；Playwright 实证：3 App 级联启动、grid 排布（I6 几何）、关窗回收、Alt+Tab 轮转、崩溃页隔离（crash 注入 counter；截图 reports/assets/465-t5/）
| T6 | shell/launcher/热键 | taskbar DOM 实现 + 028-launcher 挂 overlay + document 键盘路由（召唤/Alt+Tab/布局） | Playwright 全键盘流：Ctrl+Space→搜索→Enter→新窗；Alt+Tab 聚焦轮转 |[✅ 已完成] taskbar DOM 叶（T4 登记+Taskbar.vue）+ 宿主消费；document 捕获段键盘路由（keyboard.ts：Ctrl+Space 召唤/Alt+Tab 轮转/Ctrl+Alt+f/g/m 布局/Tab 篱笆）；464 占位槽升级搜索过滤流；Playwright 全键盘流实证（Ctrl+Space→搜索 count→Enter→新窗；截图 reports/assets/465-t6/）；028-launcher 真源挂载按 §8 待 464 合入后复验
| T7 | tauri 壳 | 宿主页 tauri 打包全屏（可选：global-shortcut） | `auto run --render tauri`（宿主项目）实机全屏桌面可用 |[✅ 已完成] AUTO_DESKTOP 透传 tauri 管线（desktop 刷新 + tauri.conf fullscreen + devUrl 随 --front-port）；顺修两处管线阻断：pkg add `--dev`→`-D`（pnpm v11 拒绝且 node_modules 启发式吞错）、update_tauri_lib_rs 仅在 commands 产物存在时注册（无 api 项目 E0583）；实机验证：tauri dev 编译 35s + 全屏窗口上屏（taskbar 可见，进程 app.exe active）；global-shortcut 可选项未做（登记后续）
| T8 | 对拍与收尾 | 与 iced 桌面对拍清单执行（I4/I5/I6）；双端截图矩阵；文档 | 对拍记录 + `cargo t`（a2vue 金样/能力锁绿） |[✅ 已完成] reports/465-t8-parity-record.md：I4 金样 10/10 绿、I5 零分叉兑现（464 合入后复验）、I6 共享表 17 例双端绿、E1/E2 成文核销、单 App vue 零回归实测（无 wm/registry 泄漏）；scoped 门禁全绿（a2vue 10 + capabilities 82 + parity 17 + registry 1）；全量 `cargo t` 归 /auto-plan:review 门禁（work 技能收尾禁全量）

## 5. 验收

1. **web 端到端**：vite 起宿主页（`auto run --desktop`，apps 指向 examples/ui）→
   launcher 召唤 → 启动 **≥4 个 vue App**（001/006/013/015/025 等 vue 形态）→
   grid/master-stack 排布 → 拖拽/缩放/关闭 → Playwright 全绿。
2. **tauri 端到端**：同一宿主页全屏 webview 可用（T7）。
3. **I4**：`virtual_window`/`taskbar` 两端实现同源登记 + a2vue 金样；
   **I5**：028-launcher 源码零分叉；**I6**：布局期望值表双端共享。
4. **E1/E2 检验**：叶子形态（DOM ↔ 树内 Element）切换为**加法操作**——
   不出现第二条桌面代码路径（与 I3 同型）；`(AppId, event)` 注入形状
   在 WM 语义规范中成文，386 复活可直接引用。
4. `auto run`（单 App vue）零回归（生成物 diff 仅宿主模式新增文件）。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| modal/teleport 页面级假设破坏窗内体验 | T2 spike 前置；v1 限制清单 + 占位警告，不硬改生成器语义 |
| 多 createApp 实例开销（N 窗 N 实例 + 全局样式重复） | 实例级懒挂载（滚动/隐藏不挂）；开销实测记录（对比 Design 23 内存目标，数据归档） |
| 462 契约漂移（并行开发） | 本计划只消费 schema/aura.at 登记契约；对拍在 T8 统一执行，中途以金样锁定 |
| 全局监听跨窗误触发 | 已知限制清单（§3.3）；受害 App 白名单先行，改进项登记 |
| pnpm/tauri 环境抖动（Windows） | 复用既有 run_tauri_project 管线与 kill-vue-dev 惯例；T7 独立提交可回滚 |

## 7. 并发边界

- **拥有**：auto-man 宿主 scaffold 段、`src/apps-registry.ts` 生成器、
  `virtual_window`/`taskbar` 的 a2vue DOM 实现、宿主 App.vue 模板、tauri 宿主配置。
- **避让**：不碰 renderer.rs/session.rs（iced 侧归 462/463）；028-launcher 源码只读
  （归 464）；若与 463 同期改 `schema/aura.at`，错峰合入。
- **消费**：462 widget 契约、463 布局规范与注册表清单逻辑、464 launcher 源。

## 8. 关联

- 依赖：462（契约）；并行：463/464；T6/T8 需要 463/464 实际合入后复验。
- 后续入口：BroadcastChannel 跨窗协作、iframe 隔离档、每窗主题、系统级热键深化
  ——均登记 Design 24 后续，不在本批。

## 9. 复审记录（/auto-plan:review，2026-08-29）

**Reviewer**: zcode（auto-plan-review）· **入口状态**: execution_done ·
**worktree**: `.worktrees/plan-465-dev`（基 cf05b4868，8 commits，工作区净）

### 9.1 逐项验收（重验证据）

| # | 验收项 | 判定 | 证据（本复审重跑） |
|---|---|---|---|
| 1 | web 端到端：宿主页→launcher 召唤→**≥4 App**→grid/master-stack→拖拽/缩放/关闭→Playwright 全绿 | **PASS** | 重跑四连：4 App（hello-world/counter/login/pricing-table）+ master-stack 上屏（`reports/assets/465-review/20-review-4apps.png`：master 左 55%+右列三窗均分，≣ 高亮）；拖拽 (+120,+60)→(200,140) 逐像素吻合、SE 缩放 +80/+50→720×530、grid 重排回 (0,0,1280,752)（`21-review-drag-resize.png`，坐标级断言 ALL PASS）；关窗/键盘流 T5/T6 既有实证复认 |
| 2 | tauri 端到端全屏可用 | **PASS** | T7 实机：tauri dev 编译 35.42s → app.exe 全屏上屏（taskbar 可见，进程 active 记录；webview 持续重绘致 CUA 帧过期，交互由同页 Playwright 覆盖） |
| 3 | I4 两端同源登记+金样 | **PASS** | `test_a2vue_virtual_window` 绿（a2vue 全家桶 10/10）；aura.at vue:→registry overlay 机制核对（Plan 435 P4-4 路径） |
| 4 | I5 028-launcher 零分叉 | **PASS（部分顺延）** | 本批对 028-launcher 零接触（§7 避让兑现；464 未合入，master 无此目录）；真源挂 overlay 归 464 后复验（计划 §8 预授权，非 silent deferral） |
| 5 | I6 期望值表双端共享 | **PASS** | layout_cases.json 17 例：Rust `layout_parity_cases_shared_table` 绿 + `node scripts/ui-layout-parity.mjs` 17/17（本复审重跑） |
| 6 | E1/E2 加法操作+成文 | **PASS** | iced 侧零触碰（`git log -- crates/auto-lang/src/ui/iced/` 基线后无新提交）；规范文 `reports/465-t4-wm-dom-leaf.md` §1–2 |
| 7 | 单 App vue 零回归 | **PASS** | 002-counter 重生成：无 src/wm/、无 apps-registry.ts，App.vue 为 App 本体 |
| 8 | 全量门禁 `cargo tf` | **PASS** | **3234/3234 passed, 89 skipped**（含 1M churn str_churn_bounded_large；本计划未触 VM/transpiler/book，tv/tt/tb 不适用） |

### 9.2 遗漏/延后/workaround 猎查

- **遗漏**：未发现 Done 项丢子件。T4 金样不含 win prop 透传——已在执行期登记
  KNOWN-DEBT（宿主叶子自读 store 不依赖），非 silent drop。
- **延后**（均有计划内授权，非擅自缩范围）：I5 真源复验待 464（§8 明文）；
  global-shortcut 可选项未做（§3.4 明文"可选"）；BroadcastChannel/iframe/每窗主题
  为计划 §1 非目标。
- **workaround**：KNOWN-DEBT 4 条（prop 直通缺口/reka portal 逸出/document 广播/
  pkg 吞错启发式）均 T2/T4/T7 执行期主动登记，无隐藏 hack；diff 内 TODO/FIXME
  扫描为零；新增代码零编译警告（现存 193 条警告全部基线既有，`is_api_use` 等在
  cf05b4868 已存在）。

### 9.3 结论

**全部验收项 PASS，无未授权缩水** → `status: reviewed`。
spec-impact 元数据已填（§frontmatter），`/auto-plan:merge` 可直接消费；
worktree/branch 留置，fold+清理归 merge。
