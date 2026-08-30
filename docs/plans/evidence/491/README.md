# Plan 491 T6 实机代验证据

日期：2026-08-30 · 二进制：`.worktrees/plan-491-dev/target/debug/auto.exe`
（含 Tab/Shift+Tab 焦点环遍历全部改动）· 通道：应用内 MCP（AUTOUI_MCP_PORT）

## 1. 机制级（主证，7/7 绿）

`cargo test -p auto-lang --lib --features iced-layout-tests p491`
→ 7 passed, 0 failed（tab_next / shift_tab_prev / wrap / find_focused_probe /
unfocused_fallback / single_input / prompt_tab_captured_not_fallback）。

覆盖链：未捕获 Tab 按 shift 分派（生产纯函数 keyboard_event_message）→
FindFocusedInput 探针读实际聚焦（含点击直聚）→ focus_traverse 登记表回环
求址 → 内建 focus operation 置焦 → 键入单投递归因。

## 2. 042-two-inputs-child MCP 全流程（无回归 + 表单完成）

`p491_042_mcp_flow.txt`（含双图）：

- autoui_type username=admin → handler `.LoginChild.UserChanged`
- autoui_type password=admin → handler `.LoginChild.PassChanged`
- press Login → `.LoginChild.Submit`
- 终态 `authed=true, user="admin", pass="admin"`，LoginPage 卸载
  （快照无 "Enter username"）。

图：`p491_042_login_filled.png`（双框已填）/ `p491_042_authed_in.png`（"in"）。

注：流程途中 state 出现 `123 (int)` 中间值——P483-4 已登记的 MCP type
通道对 oninput 的写序怪癖（master 既有），终态正确，非本计划回归。

## 3. musk 登录页归因（VM 轨，auto-musk 主检出只读运行）

- autoui_type username → handler `.LoginPage.UsernameChanged`
- autoui_type password → handler `.LoginPage.PasswordChanged`（验收项
  「password 归因正确」✓）
- press Login → `.LoginPage.Submit`

admin/admin 全键盘登录未在 MCP 通道完成（username 变量不随 autoui_type
持久化——P483-4 同族通道怪癖；与 483 基线一致：归因 ✓、全键盘真人顺延）。

## 4. 真键盘 Tab 尝试（blocked → 顺延 P483-3 真人清单）

computer-use 前台真键通道被并行会话持续抢占：每次 激活→截图→派发 之间
前台即被夺走（full-display 帧反复 stale；窗口域帧 dispatch 身份失配；
随后本计划两个 Auto - App 窗被外部关闭）——与 P483-3 已登记的环境阻塞
同象。真键盘 Tab/Shift+Tab 实机复验按计划非目标条款并入 P483-3 真人
清单（含 musk 登录页 Tab 流），不在本计划闭合。
