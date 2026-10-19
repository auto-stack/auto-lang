# PLAN-811 R5 修复轮报告（review-r5 needs_fix → 修复闭合）

2026-10-19 · stage: work · 针对 [review-r5](../review-r5/review.md) 两项
findings 的修复与验证。修复树：同 worktree
`D:/autostack/.wt/lang-811/auto-lang`（`codex/plan-811-dev`），R5 受审
`53f1111a2`，本轮修复提交：**`2ffb87755`**（单文件 +144/-31，仅 core
image_pipeline）。流程：先提交→重建→实跑收据（六 JSON 的
`worktree_commit` 均为 `2ffb87755`）。

## Finding closure 表

### F811-R4-01（续 OPEN）· P2 · 在途关闭后 backlog 残留 1

- **finding → invariant**：占用仪表与实际终态一致——所有返回完成后读数
  为 0；预留/占用/丢弃语义分别定义；满通道 oracle 可区分。
- **root cause**（复审 §3 定位准确）：`MediaWorkerPool::shutdown` 先发
  Stop 后 join——已取出的 decode 任务在 Stop 排队后仍可把 Work 成功送入
  通道剩余容量，resize 收到 Stop 即 break，这些"Stop 后被接受的 Work"
  随 receiver 销毁而未结算 → enqueued 永久多 1。
- **fix**：
  1. **关闭顺序重排**：先 join **decode workers**（通道唯一发送方，
     stopping 下有界退出）→ 再发 Stop → 再 join resize lane。发送方全
     亡后不可能有 Work 被接受到 Stop 之后；发送失败路径无需回滚（未接
     受即无计）。
  2. **resize lane stopping 快路径**：stopping 期间收到的 Work 结算计数
     后直接丢弃（不做重变换——registry 已在关闭，publish 本会被拒）；
     shutdown 由 decode 工作量界定而非 resize 积压（饱和测试 ~57s→~5s）。
  3. **语义诚实化**（R5 命名/方向指正）：`resize_enqueued` 只计**成功
     send 后被接受**的 Work（阻塞在 send 中的解码线程**不计**）；
     `resize_handled` 计 lane **已取走**的 Work；getter 先读 enqueued 后
     读 handled——取走发生在两读之间时对着稍后读的 enqueued 扣减，只会
     **瞬时低报**、不会超过真实未取数也不会为负（饱和减）。因此
     backlog ≥ 8 = 8 槽全部持有"已接受未取走"的 Work——可区分的真满
     通道 oracle。
- **permanent case**：`resize_backlog_is_exactly_zero_after_inflight_
shutdown`——复审 occupancy-r5 形状（第一张 Transforming、第二张解码中
  关闭）× 3 试：线程归零、registry 终态、**backlog 精确 0**。
- **current results**：复审 occupancy-r5 探针在本树直跑三试
  `after_shutdown=0 workers=0 registry_shutdown=true pending=0`
  （修复前 `after_shutdown=1` ×3）；BACKLOG 空闲关闭 0 不 panic 保持。
- **remaining gap**：无。

### F811-R5-01 · P3 · unused import warning

- **fix**：删除并发关闭测试中未用的 `Ordering` 导入。
- **current results**：`cargo check -p auto-media-core --tests` 零
  warning（R4 报告"零 warning"只查了 lib cfg——已如实修正口径）。
- **remaining gap**：无。

## 复审重放与全套件（本树实跑）

- `review-r5/replay.py`：RACE 拒绝 / BACKLOG 空闲 0 / SHUTDOWN 完成语义
  （second_returned→registry 关/无新票）/ 七窗口 typed-consumer 全绿
  （distinct/same/swap/bad-same/no-timer/async-no-timer/cascade）/
  core **43/43**。
- 六 harness 提交后实跑重绑 `2ffb87755`：VM 12/12、Vue 10/10、
  merged 11/11、identity 6/6、async 4/4、生成 HTTP 13/13。
- 792 driver 修复树重跑收据见 handoff 增补。
- scoped rustfmt（本文件）--check 0；`cargo check --tests` 零 warning。
