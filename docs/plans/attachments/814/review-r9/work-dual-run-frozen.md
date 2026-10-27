# PLAN-814 V-03 双端对照实录（R8-F01 修复）

2026-10-21。绑定实现：分支 `codex/plan-814-dev` @ R8 修复批（780415083
+ 本批健康修复）。同源输入：`examples/ui/002-counter/src/app.at`（未改动）。

## 运行方式（可复现）

- **Vue 端**：`auto gen`（canonical 生成，gen/front/vue）→ `npx vite
  --port 33197` → Playwright（msedge headless；playwright 单源
  auto-forge-ui node_modules）驱动 DOM。注意：vite 仅监听 IPv6 `::1`，
  浏览器须走 `http://localhost:33197`（127.0.0.1 拒连）。
- **VM 端**：`AUTOUI_MCP_PORT=39473 auto run -r vm` → MCP JSON-RPC
  （autoui_snapshot/autoui_press_sequence）。

## 状态机对照（逐点一致）

| 操作 | Vue（DOM text） | VM（snapshot text / state） |
|---|---|---|
| 初始 | `Counter: 0`（buttons: -, Reset, +） | `Counter: 0`（同三按钮，onclick 挂接） |
| 点 `-` | `Counter: -1` | `Counter: -1` / `count: -1 (int)` |
| Reset 后点 `+` | `Counter: 1` | `Counter: 1` / `count: 1 (int)` |

## 产物

- Vue 截图：vue_initial2.png / vue_after2.png / vue_final.png（Playwright
  全页）；vue_initial.png / vue_after_click.png（技能脚本产物）。
- VM 侧为 MCP 文本 snapshot/state（上表逐字；无像素截图——renderer 截图
  归 autoui_screenshot 工具，本对照以结构化状态为 oracle）。

## 边界

- 单示例（002-counter）+三状态两操作=合同要求的"有代表性"对照；非全
  组合逐像素 parity（合同未要求）。
- autoui_action 仅支持 autodown_editor 元素——常规按钮交互经
  autoui_press_sequence（button label 路由）。
