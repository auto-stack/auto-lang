# PLAN-811 R4 修复轮报告（review-r4 needs_fix → 修复闭合）

2026-10-19 · stage: work · 针对 [review-r4](../review-r4/review.md) 三项
findings 的修复与验证。修复树：同 worktree
`D:/autostack/.wt/lang-811/auto-lang`（`codex/plan-811-dev`），R4 受审
`27d3387af`，本轮修复提交：**`53f1111a2`**（2 文件 +293/-44）。
**本轮收据流程修正**（回应 R4 §6 provenance 备注）：先提交代码、再从
提交后的树重建全部产物、然后运行 harness——六份 JSON 的
`worktree_commit` 均为 `53f1111a2`（提交后实跑，非 WIP 前值）。

## Finding closure 表

### F811-R4-01 · P2 · resize_backlog 下溢

- **finding → invariant**：占用仪表 = 真实 Work 消息通道占用——Stop 不
  计、send/recv 竞争下不变量成立、读取不产生负差或假"满"。
- **fix**：`resize_enqueued` 在（可能阻塞的）send **之前**预留、接收端
  已消失时结算回冲；`resize_handled` 只计 **Work**（recv 后、处理前），
  **Stop 双侧都不计**；公开读取**先 load handled 后 load enqueued**
  （任意时刻 handled ≤ enqueued，后读 enqueued 只会更大）——差值永不
  下溢、竞争下只会瞬时低报不会高报；注释改为如实语义（不再声称 exact）。
- **permanent cases**：`resize_backlog_gauge_lifecycle_controls`（空闲 0
  → 真实 Work 瞬时 >0 → 完成后精确归 0 → 关闭后 0 且 10 次重复读稳定
  ——即复审公共 API 反例形状）；饱和测试的 ≥8 断言保留（现在由健全仪表
  支撑）。
- **current results**：复审 replay 本树
  `after_idle_shutdown_panicked=false value=Some(0)`（修复前
  `panicked=true`）。
- **remaining gap**：无。

### F811-R4-02 · P2 · shutdown_started 把"关闭中"当"关闭完成"

- **finding → invariant**：区分 closing/closed——首调用独占执行（不重复
  Stop），重复调用**等待同一完成结果**；任何 shutdown() 返回即完全关闭
  （无准入窗口）；超时清理语义不得改变生产生命周期。
- **fix**：`shutdown_runtime_inner` 重写：`closing` 在**任何**调用者起步
  即置（准入即刻死亡）；`shutdown_started` swap 给首调用者独占执行权；
  `shutdown_done` + `shutdown_finish` Mutex/Condvar 让后续调用者（重复
  调用、超时 unwind 后的 Drop）**等待完成**并共享同一结果。
- **permanent cases**：`concurrent_duplicate_shutdown_returns_only_when_
fully_closed`（复审反例形状常驻：真解码积压在途、调用者 2 在 closing
  锁存后进入——其返回时 registry 已关、无活票据）；真实 owner 负控改判
  **完成合同**（第二次关闭等待完成 + 返回后 registry 已关 + fresh 票据
  dead——R3 版把"廉价返回"当通过恰是漏检点，已按 R4 指出修正）。
- **current results**：复审 replay 本树 `second_returned_ms=9935
  registry_shutdown=true pending=0 new_ticket_pending=false
  new_transition_accepted=false`（修复前 0ms/registry 未关/票据准入）。
- **remaining gap**：无。

### F811-R3-02（续 OPEN）· 异步入口 + 回调内换 src

- **finding → invariant**：同步/async 启动共用一致媒体唤醒装配；事件回调
  改写视图后仍获注册/退订与后续拍；不要求应用 timer；空闲零消息保持。
- **fix**：
  1. 提取 `typed_media_wake_subscription`（50ms 门控心跳），
     **同步 runner 与 `run_app_with_task_devtools`（async-boot 入口）共用
     同一装配**；
  2. **TYPED_RESWEEP 门**：sweep 派发过事件的拍为唤醒门续一拍——
     loaded/error handler 可能在回调内换 src/卸载/重挂载，改写后的视图
     需要至少一次后续 sweep 完成注册/退订；事件链静止后门关闭（空闲零
     消息保持）。
- **current results**：
  - 复审 typed-consumer 本树：**async-no-timer a=1**（修复前 a=0）、
    **cascade a=1 b=1**（修复前 a=1 b=0），distinct/same/swap/bad-same/
    no-timer 全行保持；
  - 本仓新增手写 async-boot 探针（真 `run_app_with_task_devtools` 消费
    者、无 timer）：`async_wake_run` **4/4**（Prepare 真排队→Mount 后
    loaded 恰好一次仅靠唤醒→Bad error 恰好一次）；
  - identity 探针 cascade 段（无 timer）：A 的 loaded 回调内换绑已 Ready
    的 B——重绑后所有挂载 handler（A/B/C2）对新订阅各单发
    （la 2→3、lb 2→3、lc2 2→3），`identity_matrix_run` **6/6**。
- **remaining gap**：无。

## 复审重放与全套件（本树实跑）

```text
python docs/plans/attachments/811/review-r4/replay.py D:/autostack/.wt/lang-811/auto-lang
```

RACE 拒绝保持；BACKLOG 空闲关闭 0 不 panic；SHUTDOWN 完成语义；
typed-consumer 七模式全绿；core **42/42**（82s 串行）。六 harness 全部
重绑 `53f1111a2`：VM 12/12、Vue 10/10、merged 11/11、identity 6/6
（含 cascade）、async 4/4、生成 HTTP 13/13；792 driver 在修复树重跑收据
见 handoff。两修改文件 scoped rustfmt --check 0；构建零 warning
（清掉了 unused 变量）。并行大图负载互扰说明：plain `cargo test` 线程
并行下多个 3200×2400×12 工作负载会争 CPU 致宽限超时——nextest（每测试
一进程，仓标准档）与复审的串行口径均稳定 42/42；已在测试注释注明。
