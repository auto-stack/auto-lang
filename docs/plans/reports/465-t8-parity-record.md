# Plan 465 T8：对拍记录与收尾

> 2026-08-28。I4/I5/I6 对拍结论 + 双端验证矩阵 + 零回归核查。

## 1. I4：virtual_window/taskbar 两端同源登记 ✅

- 登记源 `schema/aura.at`：`virtual_window`（vue: → `@/wm/VirtualWindow`）+
  `taskbar`（web 臂 none → component，vue: → `@/wm/Taskbar`）；registry spec +
  schema overlay（Plan 435 P4-4）双落。
- a2vue 金样 `test/a2vue/virtual_window/`：`test_a2vue_virtual_window` 绿
  （10/10 a2vue 全家桶绿）。
- iced 端实现 462/463 既有（ui/iced/virtual_window.rs + shell 装配）；DOM 端
  `auto-man assets/wm/`（rust_embed → 宿主 src/wm/ 每run覆写）。

## 2. I5：028-launcher 源码零分叉 ⏸（按计划 §8 顺延）

- 464 未合入（028-launcher 不存在于 master）——本批以**占位 launcher 槽**先行
  （宿主 overlay：搜索过滤 + Enter 启动，热键流全通），464 合入后 T6/T8 复验
  换真源。**源码零分叉成立**：本批对 028-launcher 零接触（§7 避让兑现）。

## 3. I6：布局期望值表双端共享 ✅

- 共享表 `crates/auto-lang/src/ui/layout_cases.json`（17 例）；
  Rust `layout_parity_cases_shared_table`（ui-iced feature）绿；
  TS `scripts/ui-layout-parity.mjs`（node 原生 strip-types 直跑
  `assets/wm/layout.ts`）17/17 绿。
- 实机印证：Playwright grid 排布截图（`reports/assets/465-t5/12-grid.png`）
  与表内 grid_3 期望矩形逐格一致。

## 4. E1/E2 检验 ✅

- `(AppId, event)` 注入形状 + `AppWindow` 叶枚举（Element | RenderCommand |
  Wayland | DOM）成文：`reports/465-t4-wm-dom-leaf.md`（386 复活可直引）。
- 加法操作：iced 侧零改动（本批未触碰 renderer.rs/session.rs——§7 避让兑现，
  `git log -- crates/auto-lang/src/ui/iced/` 仅 462/463 既有提交）；vue 侧
  叶子构造点单一（宿主 z-stack 内 `<VirtualWindow :win>`）。

## 5. 双端截图矩阵

| 端 | 截图 | 位置 |
|---|---|---|
| vue 宿主（vite） | 初态/3窗级联/grid/关窗/Alt+Tab/崩溃页/键盘流 | `reports/assets/465-t5/`（10–16）、`465-t6/`（17a–17） |
| tauri 宿主 | 全屏上屏实拍（本报告 §6 记录） | 会话内实拍（帧级 webview 重绘导致 CUA 帧过期，存档以日志+进程记录为准） |
| iced 桌面 | 462/463 复审既有存档（本批零改动，不重复取证） | 462/463 计划档案 |

## 6. tauri 实机记录（T7）

`auto run --render tauri --desktop --apps ../ui`：desktop 刷新 + tauri init +
fullscreen conf（"✓ Tauri desktop window: fullscreen"）→ src-tauri 编译 35.42s
→ `Running target\debug\app.exe` → 实屏显示全屏虚拟桌面（taskbar ⊞/布局按钮
可见；进程 app.exe active）。交互取证的 CUA 帧因 webview 持续重绘过期
（transport 限制），内容级交互已由同页 Playwright 流（T5/T6）覆盖。

## 7. 零回归核查（验收 §5.4）

单 App vue 生成（002-counter，非 desktop）重生对比：`src/` 仅标准产物，
**无** `src/wm/`、**无** `src/apps-registry.ts`、App.vue 为 App 本体 ——
生成物 diff 仅宿主模式新增文件成立。

## 8. 门禁（scoped，全量归 /auto-plan:review）

- `cargo test -p auto-lang --features ui-iced --lib test_a2vue` → 10 passed
- `cargo test -p auto-lang --features ui-iced --test vue_capabilities` → 82 passed
- `cargo test -p auto-lang --features ui-iced --lib layout_parity_cases` → 1 passed
- `cargo test -p auto-man --lib desktop_registry` → 1 passed
- `node scripts/ui-layout-parity.mjs` → 17 pass / 0 fail
- `cargo check -p auto-lang -p auto-man` 绿
- 单 App vue 零回归（§7）

## 9. KNOWN-DEBT 候选（已登记 KNOWN-DEBT-AND-RISKS.md）

1. a2vue 组件路径任意 prop 直通（virtual_window `win:` 不透传；宿主叶子自读 store 不依赖）。
2. reka portal 族（dialog/modal/dropdown）DOM 重挂 body，窗口容器不可 CSS 收敛（T2 ①；
   正规解 = 生成器 `DialogPortal :to` + provide/inject，后续）。
3. App 自注册 document 监听跨窗广播（T2 ③；受害 App 白名单先行）。
4. pkg::run_command_live 的 node_modules 存在即 Ok 启发式（pnpm v11 下仍可能吞真失败；
   本批仅修 `--dev`→`-D` 根因）。
5. tauri global-shortcut 系统级热键（计划可选项，未做）。
