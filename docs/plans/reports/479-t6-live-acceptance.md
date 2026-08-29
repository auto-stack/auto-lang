# Plan 479 T6 实机验收记录：通知中心（S6）+ notify 动词

> 2026-08-29 · 基线 plan-479-dev（T1–T5 提交后）。宿主：窗口模式
> `ui_desktop.exe --apps-dir examples/ui`（472/478 同入口）。
> 注入通道：AUTOUI_MCP_PORT=9478（进程内 MCP HTTP）；storage 隔离
> `AUTO_VM_STORAGE_FILE=tmp/479-t6-storage.json`。驱动脚本：
> `examples/ui/028-launcher/tests/test_479_t6.py`（可重跑——前台空闲时
> OS 注入项可补采）。

## 1. 实机交互清单

| # | 验收项 | 结果 | 证据 |
|---|---|---|---|
| 1 | dock 铃铛实机渲染：行尾 bell 图标、未读 0 无 badge（空串/零双守卫）、与 pager/布局键并列不挤占 | **PASS**（实机截图） | `assets/479-t6/10-initial.png`；shell.at 真源编译装载（冒烟测同源） |
| 2 | LaunchApp 成败等既有 toast **自动入史** + 落盘（8 调用点改道的实机链） | **PASS**（机会性实机流量 + 落盘铁证） | 会话期间真实发生一次分区 × 非空门 toast（用户前台操作窗口所致），storage 落盘：`shell.notes.0 = {"id":1,"kind":"error","msg":"分区 1 含窗口，请先移动或关闭","at":"16:00"}` + 槽 1..9 全 `""`——push_notification→persist_notes 全链实机执行，HH:MM 格式实机确认 |
| 3 | **boot 恢复**：带已落盘槽位重启，restore_notifications 读回、历史保持、无 panic | **PASS**（实机重启 + storage 铁证） | `assets/479-t6/20-restart-restored.png`（二次 boot 渲染正常）+ 重启后 storage 文件 slot0 条目原样（恢复读侧无破坏）；headless 语义金样 `notif_storage_roundtrip_slots` |
| 4 | 铃铛点击开面板（未读清零/badge 消失）→ 历史 MRU 序/kind 图标/时间串 → 逐条 × → 全部清除 → Esc 关闭 | **BLOCKED（注入通道）** | 用户会话活跃（前台竞争，472 #5–#8 / 478 T6.2–4 同款；trusted host 拒绝前台抢占，激活窗口后用户窗口立即回前台）。headless 全链覆盖：`notif_center_summon_headless`（真 notification_center.at：懒挂载→快照 rows MRU 序→未读清零→Dismiss 写 `notes_dismiss\t<id>`→ClearAll 写 `notes_clear`→Escape 自隐）+ `notif_end_to_end_toggle_dismiss_restore`（notify→badge→toggle→dismiss→落盘→重读） |
| 5 | notify 动词实机注入（App 主动请求通知） | **BLOCKED（表面受限）** | MCP 快照/动作面为 primary 单 App 语义（453 T8 冻结），shell/特权 App 的 `__desktop_cmd` 不在 autoui_* 可达面；headless 覆盖：`notif_commands_encode_parse_round_trip`（四动词双轨往返）+ e2e 的 Notify 臂链 |

## 2. BLOCKED 项 headless 覆盖指针（472/478 先例成文）

| 阻塞项 | 覆盖测试（全绿） | 语义链 |
|---|---|---|
| badge 翻转/清零 | `notif_unread_semantics_panel_visibility` + `notif_projection_notes_and_fingerprint` | 面板关 +1 / 开不加 / 开清零 / 投影串与指纹段 |
| 面板交互全流 | `notif_center_summon_headless` | 召唤挂载→注入→rows MRU 序→dismiss/clearAll→Esc 自隐→总线可达宿主臂 |
| 端到端持久化 | `notif_end_to_end_toggle_dismiss_restore` + `notif_storage_roundtrip_slots` | notify→badge→开面板→dismiss→空槽落盘→新会话重读；12 条截断 10 槽 |
| notify 动词 | `notif_commands_encode_parse_round_trip` | encode/parse 双轨分符 + 坏载荷跳过 |
| dock 铃铛接线 | `notif_shell_at_smoke_toggle_and_badge` | 真 shell.at 编译 + NotificationToggle→`notes_toggle` 记录 |

## 3. 证据目录

- `10-initial.png`：dock 实机初态——⊞ + pinned 三枚 + 运行窗区 + pager +
  布局三键 + **行尾铃铛（未读 0 无 badge）**。
- `20-restart-restored.png`：二次 boot（带 slot0 已落盘条目）渲染正常。
- `tmp/479-t6-storage.json`：机会性实机通知落盘铁证（§1 #2；JSON 槽位形态
  = T1 定案 4 全量重写 10 槽语义）。

> **复审注记（/auto-plan:review）**：两帧 PNG 经 md5 比对**逐字节相同**——
> 桌面初态为确定性渲染（同窗同坞、badge 两次皆隐：boot 恢复语义即未读归
> 零），第二帧证明「二次 boot 正常渲染无 panic」，**不**单独构成「历史恢复」
> 的可视证据；恢复语义的实证 = storage 铁证（§1 #2/重启后槽位原样）+
> headless 金样 `notif_storage_roundtrip_slots`。可视面板交互项留待前台
> 空闲重跑（驱动脚本入口）。

## 4. 执行期发现

1. 前台竞争与 472/478 完全同款：trusted host 校验 frontmost，用户会话活跃
   时 OS 注入通道不可用；MCP 截图通道不受影响。
2. **机会性实机流量**：用户在实机窗口前台期间的自然点击触发了一次分区 ×
   非空门 toast，意外完成了 #2 的实机入史+落盘链——8 调用点改道在生产
   路径（非测试路径）生效的直接证据。
3. 面板打开期间未读不加（visible 门控）在实机 §1 #2 中同样隐含验证：该
   toast 入史时面板从未打开，未读落库 1 次；badge 面因前台受限留 headless。
