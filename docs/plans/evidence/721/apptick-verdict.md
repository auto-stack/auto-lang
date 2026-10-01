# PLAN-721 定谳文档：桌面轨更新路径家族（P712-D1 承接）

日期：2026-10-01 ｜ 载具：lang-721 worktree（plan-721-dev @1cd8262d3，承 master 7491719b8）
日志：desktop-721{b..i}.log（D:/autostack/.wt/lang-721/，未入库）

## 结论一：T-16「桌面轨更新路径断链」不成立——r2 破链已被 712-r3 合并修复

AUTO_SCHED_DIAG 四轴 trace（T-1 工装，sched_diag.rs）在验收桌面实测
（desktop-721b/c，018-book-reader 内嵌 = AppId(5)）：

| 轴 | trace 行 | 实测 |
|---|---|---|
| 订阅装配 | `sub_parked app=AppId(5) parked_total=1` | ✓ park 即装配 |
| 消息到达 | `arm_parked arrive parked_io_before=true` | ✓ ~50ms 内 |
| 泵推进 | `[VM-PARKED] bookshelf_Init resumed (50.9ms)` | ✓ 恢复完成 |
| 置脏传播 | `arm_parked done dirty=true view_dirty_set=true` | ✓ 回填在位 |

r2「桌面内嵌轨 0 条 T16-DIAG / Loading 卡死」为测量期伪象或已被
712-r3 三层修复（back_prefix 装载臂 / route-param 堆化 / Init 代际）
顺带治愈——桌面内嵌 018 书架点开即出数据（本日全程复验）。
**T-16 桌面轨专项可结案。**

## 结论二：T-19 暂停回弹真因——帧级序列已捕获（desktop-721g）

干净序列（无用户事件窗口，AUTO_VM_TRACE + contract trace）：

```
t=35157 [VM_EXEC] handler_Controls_TogglePlay @4000002   ← is_playing→false
t=35158 mpv_apply applied=false→down=true WRITE           ← 新视图烤 paused=true，mpv 真暂停 ✓
t=35158 mpv_poll EDGE→false                               ← 上行 PlayStateChange(false) ✓
        OnPlayState(false) handler 执行                    ← store.is_playing=false（再次）
t=35343 mpv_apply applied=true→down=false WRITE ★         ← OnTime 触发的下一次全新构建烤回 paused=false
t=35344 mpv_poll EDGE→true                                ← mpv 恢复
        OnPlayState(true) → is_playing=true               ← 回弹完成（status=「正在播放」）
```

关键事实：
1. 下行链全通（暂停写到 mpv、上行回灌到 store）——712-r3 修复实锤生效。
2. 回弹写来自**全新视图构建**（非陈旧 primitive——video_build trace 实证
   视频元素每 ~40ms 全新构建，element cache 不旁路 video 子树）；
   该构建把 `.store.is_playing == false` 解析为 false（即 is_playing=true），
   而两次 handler 写全部是 false——**同一 binding 相邻构建读值不稳定**
   （读侧世界分裂 / memo 陈旧回放），为**待下一会话定谳的开放机理**
   （video_build trace 已就位，复现即现形）。
3. VM_EXEC 显示 handler 上下文分裂：App_Tick@4000003 vs Viewport/@4000002
   （bridge.state_obj_id 世界=快照所见，is_playing 恒 true）；
   child_state_map 为死基建（全库零插入，read_all_child_states 实证空）。

## 已落修复（1cd8262d3）

**下行世代单调门**：`VideoContractDown.epoch`（convert 打戳）+ contract
apply 头部整包拒绝 `epoch < applied.epoch` 的回退写（epoch=0 legacy 放行）。
覆盖「陈旧世代烤定值逆转新世代」整类缺陷（含本例与其变体）。
回归锁 `stale_epoch_down_cannot_unpause_fresher_apply`（真引擎，
mpv_contract 13/13 绿，含 712-r3 双锁零扰动）。
活体证据：独立轨真实按钮路径暂停钉住 7s+ 零回弹（030-standalone3）。

## 未竟（登记）

- **T-19 桌面活体 ×10**：桌面实例 I 被用户实机占用（正播放 Loki），
  我的注入零到达（registry_id 反查静默丢——见待澄清）；独立轨复验因
  712 T-07 已登记死因（MCP press → wgpu 告警 → 解释器优雅退出）中断于
  第 2 轮。修法后机理层（读侧分裂）未除前，桌面复验预期仍可复现——
  **本轮修复只挡「世代回退」子类**。
- **T-11 布局塌缩非确定性源**：未动（col_right 438↔658 定谳入口仍 =
  进程内双构建比对 + builder HashMap 审计）。
- 注入静默丢：desktop-721i 中 handler inject 排队后零派发（同仓 g/h
  正常）——候选=窗口 registry 反查在窗重用/多实例下的 miss；无日志
  面（inject 臂 `let _` 吞错），补 trace 后复查。
