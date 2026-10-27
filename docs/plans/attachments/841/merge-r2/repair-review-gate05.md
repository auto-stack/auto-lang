# M841-GATE-05 修复受影响复审 — pass

2026-10-27；独立复审 agent（本会话派发，未参与修复实施）；对象：auto-ui 4903bef64532750d1597627b1197f3d2cac1effc（父 bfad5149）+ auto-man 22cec06936af64d2aea4a1d450484d6aa45a7066（UI 源 pin 同步）。

## 结论

**pass，无阻断/非阻断缺陷。** 可在新冻结基线（lang ecc2f54a7 / UI 4903bef / Man 22cec06）重跑统一门。

## 要点（全量验证见会话记录）

1. **根因链成立**：派发循环 Ok+parked→InFlight+break（vm_bridge :2609-2625）；修复前两处 resume 失败臂（:2048/:2298）只进 report 不回写；派发第 1 步"已出册=收敛"把滞留 InFlight 静默翻 Done（:2534-2558）。隔离跑片内同步失败走 Err 臂记 Failed——隔离绿/整档红分叉与整档日志实录（slice-resume FAILED 后金丝雀见 Some(Done)）完全吻合。
2. **修复正确**：mark_init_demand_failed_by_fn（:2516）双门=InFlight 且 namespaced fn 名精确相等；fn 名在引擎各 park/Runnable 点同串打戳（engine.rs :3430/3464/3545/3574），Init 需求与失败段一一对应；非 Init 事件/非 InFlight 记录无误伤；全文件仅两处 Completed(Err) 臂，无第三条遗漏路径；无 RefCell 双借。
3. **回归测试有效（红-绿实测）**：max_steps=1 纯确定性 park（steps=1 不查钟/不查 runaway/不触累计帽，零墙钟依赖）；断言链逐段自证；临时删除两处回写 → 测试红在 resume failure must reach the demand record synchronously（left Some(InFlight)），stderr 完整呈现 park→slice-resume FAILED；恢复后 1/1 绿。
4. **无 oracle 弱化/越界**：diff 单文件 vm_bridge.rs（+70/-5）；plan492 金丝雀 oracle :481-484 原样；两处 hunk 纯格式。
5. **契约未变**：同代际终态 AlreadyKnown 不重派、Failed 不阻塞后继 demand 队列。
6. **Man pin**：22cec069 单行与 UI HEAD 全 40-hex 一致；bfad5149 全树零残留绑定；FIXED 的 auto-lang 旧 pin 由 main() 运行时覆盖，无碍。
7. **定向**：plan841_init_snapshot 10/10、plan492 14/14 全绿；三 worktree 退出 clean、HEAD 未动。

备注（无行动）：handler_<W>_<Event> 名字键在下划线组合下理论可混叠——全系统既有约定（exports_by_name/parked 去重同键），本修复未引入新混叠面，且已被 InFlight 门收窄。
