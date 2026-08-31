# Plan 505 C 族：实机验收通道（ADR + 操作规程）

> 2026-08-31 · Plan 505 步骤 6 落地。目标：终结"每计划一条受阻债"的滚雪球
> ——P487-1 / P496-1 / P501-2 及 472/478/479 先例族的"前台竞争 / CUA 像素
> 身份守卫阻断实机照"统一解。

## 1. 阻断机制（为什么需要专用通道）

实机验收的既往做法是 OS 级注入（SendInput / CUA 坐标点击）。在 iced 活
渲染面上系统性失败，两个变体（487 T4 报告 §4.1）：

- **窗口域坐标点击** → `identity mismatch`（守卫无法在活渲染面上确认
  像素身份）；
- **全屏域点击** → 过身份关但败于 `live pixel owner changed`（快照与
  派发间隙任何重绘即失效——ServiceTick 400ms 帧泵 + 时钟注入使表面
  恒在重绘）。

停 MCP 帧泵（`AUTOUI_MCP_DISABLE=1`）复测仍复现：守卫对连续重绘面
**系统性**拒绝，非节拍问题。结论：不再与 OS 注入通道缠斗，改走宿主
**内进程注入**。

## 2. 通道形态（本仓入库面）

```
驱动脚本（autoui-verifier/scripts/acceptance_channel.py）
   │ HTTP JSON-RPC :PORT/mcp
   ▼
autoui_desktop 工具（mcp_server.rs；AUTOUI_ACCEPTANCE=1 门控，生产缺省拒绝）
   │ DesktopInject 入队（进程级，session.rs）
   ▼
iced 更新环 ServiceTick 节拍排空（apply_desktop_injects，≤400ms 生效）
   ├─ Bus 记录 → shell `__desktop_cmd` → drain_and_execute_desktop_commands
   │            （真实 shell 按钮的同一执行臂）
   └─ Handler 直呼 → 特权 App（shell/settings/notification/launcher）
                onclick 同一 handler 管线
   ▼
真窗渲染 → autoui_screenshot（既有工具）→ PNG 实机照
```

**保真声明**：注入不绕过任何执行臂——Bus 记录与真实按钮写入同一条
`__desktop_cmd` 总线、经同一 drain/execute 消费；Handler 直呼与 onclick
同一 handler 管线（比动词更贴按钮的一步）。渲染为真实 iced 桌面窗，
截图即实机可视证据。单测锚：`desktop_injects_flow_through_real_arms`。

**边界（通道不覆盖）**：OS 输入法（IME 输入）、真提权（UAC）、多显示器
矩阵——这些本质要求真实 OS 交互，仍留用户人手（473 债行 B6/C1/B9）。

## 3. 操作规程（runbook）

### 3.1 前置

```bash
cargo build --features ui-iced --example ui_desktop   # 宿主（单次）
```

### 3.2 启动专用验收会话

```bash
AUTOUI_ACCEPTANCE=1 \
AUTOUI_MCP_PORT=<空闲端口> \
AUTO_VM_STORAGE_FILE=tmp/<场景>-storage.json \
<repo>/target/debug/examples/ui_desktop.exe --apps-dir examples/ui
```

- `AUTOUI_ACCEPTANCE=1`：`autoui_desktop` 工具放行（缺省拒绝——不改
  生产守卫默认行为的原则落点）。
- `AUTO_VM_STORAGE_FILE`：storage 隔离（预写键可控制 boot 形态——
  dock 位置/pinned/壁纸等，472 T5 先例）。
- 无人值守窗口：桌面窗自渲染自截图，不与用户前台竞争——**不需要**
  用户会话空闲（与既往"前台空闲可补采"条件解耦）。

### 3.3 可注入面

| 面 | 形态 | 例 |
|---|---|---|
| DesktopBus 动词 | `{"action":"bus","verb":"..."}` | `open_settings`、`layout\tgrid`、`summon\tlauncher`、`workspace_next`（词表 = 投影协议 §4 + v1.4/v1.5 增量） |
| shell handler | `{"action":"handler","app":"shell","handler":"OpenSettingsPanel"}` | 齿轮 onclick 同名 handler |
| settings 面板 handler | `{"action":"handler","app":"settings","handler":"...","arg":"..."}` | 位置选择/壁纸写手/Escape 自隐（handler 名见 `assets/settings.at` msg 块） |
| notification/launcher | 同上 `app` 换名 | 面板内交互 |
| App 内按钮 | 既有 `autoui_press_sequence` / `autoui_action` | 计算器按键等（已可达） |
| boot 形态 | storage 预写键（不经 MCP） | `shell.dock.position=top` 等 |

### 3.4 节拍与确认

注入 ≤400ms（ServiceTick）生效；脚本轮询 `autoui_screenshot` 或
`autoui_snapshot` 确认状态后再进下一步（`acceptance_channel.py` 已内建）。

### 3.5 失败回退

1. 通道自身不可用（工具拒绝/宿主未起）：退 headless 等价测试族
   （487 报告 §2 对表先例）+ 预写键 boot 单帧法（496 先例）。
2. 场景语义回归：以单测锚（`desktop_injects_flow_through_real_arms`）
   定位注入面 vs 执行臂归属。

## 4. 演练记录（步骤 6 验证）

2026-08-31：`acceptance_channel.py --scenario drill` 一轮绿——真桌面宿主
acceptance 态 boot → MCP 注入 shell `OpenSettingsPanel` → 设置面板实开
（截图 `tmp/505-drill/drill-01-settings-panel.png`：设置模态 + Dock 分区
导航 + 位置/启用/固定应用三组控件实拍）→ 截图归档。单测锚
`desktop_injects_flow_through_real_arms` 绿（注入 → shell 总线 → 同臂
排空链）。

## 5. 三债补拍索引（步骤 7）

| 债 | 场景 | 证据 |
|---|---|---|
| P487-1 | 齿轮开面板 + dock 位置热切换 + Esc 自隐 | （S7 填） |
| P496-1 | 壁纸写手 + 桌面图标交互 | （S7 填） |
| P501-2 | 齿轮 → os-config 全链 | （S7 填） |
