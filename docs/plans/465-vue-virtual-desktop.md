# Plan 465: Vue 虚拟桌面（Web / tauri 宿主，M4）

> **状态**：已立项 2026-08-28，未开工
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
| T1 | 宿主 scaffold 施工图 | 宿主模式命名/触发（`--desktop` vs `render:"desktop"`）、registry 生成器位置（auto-man vue.rs）、shell 来源（shell.at 转译 vs 模板）定案；报告 `reports/465-t1-host-blueprint.md` | 评审通过 |
| T2 | containment spike | §3.3 表格四项实测（modal/teleport/监听/主题）+ DOM 焦点策略定案 | spike 记录 + 截图入报告 |
| T3 | registry 生成器 | 构建期扫 apps 目录 → `src/apps-registry.ts`（动态 import 映射） | 单测：生成物含全部 vm/vue 兼容 App；vite 构建通过 |
| T4 | WmStore + virtual_window DOM 叶 | `virtual_window` a2vue 实现 + WmStore（rect/z/focus）+ 拖拽/resize/hit-test + 布局 TS 直译 | a2vue 金样（I4）+ 布局对拍单测（与 463 期望值表共享） |
| T5 | 多挂载生命周期 | launch（动态 import+createApp+容器挂载）/close（unmount）/panic 兜底（errorCaptured 崩溃页 ≈ 459 崩溃页语义） | Playwright：连续启动 ≥3 个 vue App、关闭回收、崩溃页隔离 |
| T6 | shell/launcher/热键 | taskbar DOM 实现 + 028-launcher 挂 overlay + document 键盘路由（召唤/Alt+Tab/布局） | Playwright 全键盘流：Ctrl+Space→搜索→Enter→新窗；Alt+Tab 聚焦轮转 |
| T7 | tauri 壳 | 宿主页 tauri 打包全屏（可选：global-shortcut） | `auto run --render tauri`（宿主项目）实机全屏桌面可用 |
| T8 | 对拍与收尾 | 与 iced 桌面对拍清单执行（I4/I5/I6）；双端截图矩阵；文档 | 对拍记录 + `cargo t`（a2vue 金样/能力锁绿） |

## 5. 验收

1. **web 端到端**：vite 起宿主页（`auto run --desktop`，apps 指向 examples/ui）→
   launcher 召唤 → 启动 **≥4 个 vue App**（001/006/013/015/025 等 vue 形态）→
   grid/master-stack 排布 → 拖拽/缩放/关闭 → Playwright 全绿。
2. **tauri 端到端**：同一宿主页全屏 webview 可用（T7）。
3. **I4**：`virtual_window`/`taskbar` 两端实现同源登记 + a2vue 金样；
   **I5**：028-launcher 源码零分叉；**I6**：布局期望值表双端共享。
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
