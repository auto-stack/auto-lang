# Plan 490 T7 实机冒烟证据

日期：2026-08-30 · 二进制：`.worktrees/plan-490-dev/target/debug/auto.exe`
与 `--example ui_desktop`（含 T1-T6 全部改动）· MCP 通道

## 1. 机制级（主证全绿）

- `--features iced-layout-tests p490`：**5/5**（row/col/div onclick 三形态
  + launcher for-loop 形态四测**由红转绿**——点击发射 .Pick/.Launch 声明
  消息；无 onclick 锚绿）。缺陷 2（候选行鼠标点不中）端到端自 .at 源
  复现并修复。
- `--features ui-iced hotkey`：**7/7**（内置表矩阵/launcher 双收/解析/
  覆盖往返/订阅臂×2/storage boot 往返）。**Ctrl+Alt+Space →
  SummonLauncher 的匹配语义**由 hotkey_sub_builtin_arms 锁定。
- 相邻回归：p483 6/6、p491 7/7。

## 2. 028-launcher standalone 实机（MCP，47735 端口）

- boot ✓；召唤按钮文案含 **IME 兜底键位标注**「Ctrl+Space · IME 机
  Ctrl+Alt+Space」✓（G2 显性化交付面）。
- `.App.Open` 召唤 → visible=1 + 搜索框（`.App.SetQ` 派发）✓；Esc 关闭
  ✓（`autoui_keyboard` handler 通道）。
- 候选行 `row{onclick:.Launch}` 经 MCP `press` 驱动不到（MCP 状态直派
  发层不消费布局件 onclick——与渲染层 mouse_area 不同层，非缺陷；
  行为主证在 §1 机制测）。

## 3. 桌面模式实机（--example ui_desktop --fullscreen --apps-dir，47737）

- 全屏桌面 boot ✓（MCP live，注册表/dock 装配正常）。

## 4. 真键盘/真鼠标通道受阻实录（顺延真人清单，沿 491 先例）

- standalone Ctrl+Alt+Space 真键实测 visible 不翻转——**standalone 模式
  无桌面热键订阅（by design，订阅仅 desktop 模式）**，非键位缺陷；
- 桌面模式 activate→ctrl+alt+space **两连拒**（frontmost_pid_mismatch
  pid 4560 Chrome——并行会话持续锁定前台；键入安全闸正确拒发防止误伤
  对方窗口）= P483-3 已登记环境阻塞同象；
- 真鼠标候选行点击同通道同阻塞。
- 处置：真键盘 Ctrl+Alt+Space 召唤 / Ctrl+Alt+[ ] 分区切换 / 候选行
  真鼠标点击三项并入 P483-3 真人清单（机制级语义均已被测试锁定）。
