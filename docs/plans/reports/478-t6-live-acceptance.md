# Plan 478 T6 实机验收记录：ui_desktop 全流程

> 2026-08-29 · 基线 plan-478-dev（T1–T5 提交后）。宿主：窗口模式
> `ui_desktop.exe --apps-dir examples/ui`（472 同入口）。
> 注入通道：AUTOUI_MCP_PORT=9478（进程内 MCP HTTP）。驱动脚本：
> `examples/ui/028-launcher/tests/test_478_t6.py`（可重跑——前台空闲时
> OS 注入项可补采）。

## 1. 实机交互清单

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | dock pager 实机渲染：1 基标签、当前分区高亮（primary 底 + 深色文字 vs 非当前 muted）、每分区 × 钮、尾部 + 钮 | **PASS**（实机截图） | `assets/478-t6/10-initial.png` + dock 放大图（boot 双窗、分区 1 当前、布局三键右移）；shell.at 真源编译装载（冒烟测同源） |
| 2 | Ctrl+Tab 召唤 switcher（MRU 序/图标+标题/选中高亮）→ Tab/←→ 推进 → Enter 聚焦 → Esc 隐匿 | **BLOCKED（注入通道）** | 沙箱 OS 键注入被 trusted host 拒（frontmost_pid_mismatch，用户 Chrome 会话活跃，两次重试同败——472 #5–#8 同款）。headless 全链覆盖：`switcher_summon_advance_confirm_roundtrip`（真 switcher.at：懒挂载→MRU 快照 rows 序→Advance 环走 sel→confirm 写 `focus\t<wid>`+自隐→drain 可达执行体）；键位映射为 `desktop_hotkey_subscription` 478 分支（编译绿 + 473 键位同型先例） |
| 3 | pager `+` 增分区即入 / 空分区 `×` 删（非空 toast 门）/ 点击切换 | **BLOCKED（注入通道）** | 同上（dock 非 MCP 快照面——453 T8 单 App 语义）。headless 覆盖：`workspace_v11_host_arms_add_close_send`（宿主臂全语义：+即入新分区/×非空 toast 不删/×空删+clamp/末分区保底 toast）+ `desktop_shell_at_builds_with_dock_defaults` 扩展（真 shell.at 消息臂写 `workspace_add`/`workspace_close\t<n>` 总线记录） |
| 4 | Ctrl+Alt+Shift+←/→ 聚焦窗跨区发送（隐现且状态保留） | **BLOCKED（注入通道）** | 同上。headless 覆盖：`workspace_v11_host_arms_add_close_send`（SendTo 恒等/隐分区让渡臂）+ `workspace_move_win_to_hidden_and_same_partition`（驱动语义：归属迁移/焦点让渡/窗保留/clamp）；热键→命令映射 4 行分支（shift 先序判定，编译绿） |

## 2. BLOCKED 项 headless 覆盖指针（472 T5 先例成文）

| 阻塞项 | 覆盖测试（全绿） | 语义链 |
|---|---|---|
| switcher 键盘流 | `switcher_summon_advance_confirm_roundtrip` | 召唤挂载→注入→rows MRU 序→推进→确认→总线→执行体 |
| switcher 召唤/推进路由 | `desktop_hotkey_subscription` Ctrl+Tab 分支（T3）+ update 臂 visible 三态 | 编译级 + 分支逻辑直读 |
| pager 增删 | `workspace_v11_host_arms_add_close_send` + shell.at 冒烟扩展 | 消息臂→总线→宿主臂→驱动→toast 门 |
| 删除驱动语义 | `workspace_remove_rehomes_windows_and_clamps` / `..._transfers_focus` / `..._guards_last_partition_and_out_of_range` | 重排/压实/clamp/焦点让渡与保持 |
| send_to 驱动语义 | `workspace_move_win_to_hidden_and_same_partition` + `mru_in_workspace_orders_and_filters` | 迁移/让渡/隐现/clamp/MRU 序 |
| v1.1 投影 | `projection_v11_mru_order_and_workspace_label` / `projection_v11_mru_filters_partition_and_fingerprint_segments` | __wm_mru/label/指纹段 |

## 3. 证据目录

- `10-initial.png`：底部 dock 实机初态——⊞ + pinned 三枚 + 运行窗区 +
  **pager（1 高亮/2 muted/× ×/+）** + 布局三键（478 T5 升格后首张实机照）。

## 4. 执行期发现

1. MCP 快照/动作面为 primary 单 App 语义（453 T8 冻结）——shell/dock 不在
   autoui_snapshot/press 可达面内；dock 交互实机验收只能走 OS 注入通道。
2. 注入通道受前台竞争所阻与 472 完全同款（trusted host frontmost 校验）；
   本计划按测试设计预案直接转 headless 指针成文，驱动脚本已留重跑入口。
