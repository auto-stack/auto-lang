# PLAN-811 R3 修复轮报告（review-r3 needs_fix → 修复闭合）

2026-10-19 · stage: work · 针对 [review-r3](../review-r3/review.md) 三项
findings 的修复与验证。修复树：同 worktree
`D:/autostack/.wt/lang-811/auto-lang`（`codex/plan-811-dev`），R3 受审
`4a1d8fcf3`，本轮修复提交：**`27d3387af`**（2 文件 +289/-42：
core image_pipeline + renderer）。

## Finding closure 表

### F811-R3-01 · P2 · typed 通知只按 src 去重

- **finding → invariant**：订阅身份 = handler 身份 + src（canonical
  image-surface-events 单次门规则，动态轨 `MediaSurfaceRef::key` 同形）；
  同 src 不同 handler = 不同订阅各单发；换 handler/换 src = 新订阅；
  同订阅重绘不重复；两拍状态不得被同循环第二个表面提前消费。
- **fix**：`TypedNotifyEntry` 增 `wants_loaded/wants_error`；键 =
  `{loaded:?}\u{1}{error:?}\u{1}{src}`（Debug 身份）；逐表面逐键 upsert
  （两个表面各自持有独立 entry，ready_seen 不共享）；删除收集后未用的
  w/h（typed 生成应用 handler 为零参，投影面在动态轨）。
- **current results（复审自己的 typed-consumer 在本树重放）**：
  - distinct（正控）a=1 b=1 ✓
  - **same（同 URI 双 handler）a=1 b=1**（修复前 a=0 b=1）✓
  - **swap（同 URI 换 handler）a=1 b=1**（修复前 a=1 b=0）✓
  - **bad-same（同坏 URI 双 error handler）ea=1 eb=1**（修复前 ea=1 eb=0）✓
  另有本仓身份矩阵 harness（无 timer 探针，见下）5/5 常驻可重放。
- **remaining gap**：无。

### F811-R3-02 · P2 · merged 事件依赖应用 timer

- **finding → invariant**：merged 具备媒体完成/待通知订阅驱动的独立唤醒
  （50ms，空闲零消息），不依赖应用自带 tick；所有提前返回分支后注册/
  退订照常。
- **fix**：
  1. `devtools_subscription` 增门控 50ms 心跳（`typed_media_active`：
     pending>0 或任一 entry `awaiting_notify`——与 VM 轨 `__media_tick`
     泵同语义同门）；全部派发完 → 订阅自动消失（有界）。
  2. 通知表迁进程级单例（`run_app_devtools` 每进程单应用，作用域等价
     动态轨 AppState 表），使 'static 订阅闭包可读门控。
  3. `devtools_update` 拆 inner + 外壳：sweep 在**每一个出口**运行
     （MCP action 提前 return 臂含）。
- **current results**：`rust-identity` 探针**不声明任何 timer**，
  `identity_matrix_run` 5/5——Bind 后 loaded（la/lb/lc1 各 1）、Swap 后
  新订阅 lc2=1、Bad 后 ea/eb 各 1，全部仅由媒体唤醒驱动（收据
  identity_matrix_run.json）。
- **remaining gap**：无。

### F811-R2-02（续 OPEN）· 超时清理与满通道证明

- **finding → invariant**：外层设施截止内判红**并回收**自有线程/字节；
  负控连同真实 owner 的失败清理；满 resize 通道需准确占用观测；
  scan 门控不得 sleep 猜。
- **fix/permanent cases**（core 测试，40/40）：
  1. `bounded_teardown_result`（不 panic 的结果变体）+
     `bounded_teardown_cooperative_failure_reclaims_worker_and_bytes`：
     超时 Err 在预算内 → 取消 → 有界 join → **Weak 字节观测死亡**——
     失败与清理双有界常驻。
  2. `bounded_teardown_real_owner_failure_still_reclaims`：真实
     MediaRuntime + 大 resize 积压使 shutdown 秒级；120ms 预算内 Err；
     **新增 `shutdown_started` swap 门**（首调用独占 teardown；重复调用
     与超时 unwind 后的 Drop 立即返回，不再向满通道重发 Stop 阻塞整个
     积压排水——实测第二次 shutdown 廉价）；runtime 在有界窗内终态化。
  3. **resize 占用仪表**：`resize_enqueued/resize_handled` 原子计；
     `resize_backlog()` = 通道精确占用（recv 侧计数先于处理）；
     饱和测试采样断言 **backlog ≥ 8 = 容量满**（取代 running≥9 的推断
     主证；running 断言保留为在途态旁证）。
  4. scan 活动门**本次真落盘**（R2 补丁锚点未命中、sleep(15ms) 一直
     残留——本轮以 queued>0 轮询门替换并复核文件内容）。
- **current results**：core 40/40（含双负控）；复审 oracle_replay 冻结的
  是 R2 旧 helper 文本（SHA 绑定旧源，输出为历史复验），本树现行
  helper 的回收行为由上述常驻测试证明。
- **remaining gap**：无（R2§4/R3§5 所列范围）。

## 全矩阵收据（本树实跑，全部 provenance 绑定 `27d3387af`）

| harness | 结果 | 覆盖 |
|---|---|---|
| vm_media_run | **12/12** | 投影尺寸/A→B→A URI+尺寸/静止期 Late/卸载/坏票/恢复 |
| vue_media_run | **10/10** | 浏览器真解码/loaded 单次/真 `<img>` error/恢复 |
| merged_media_run | **11/11** | merged loaded/error/静止期/卸载/恢复 |
| identity_matrix_run（无 timer） | **5/5** | R3-01 身份矩阵 + R3-02 独立唤醒 |
| gen_media_http | **13/13** | 生成 HTTP 全 oracle 含 Range 字节匹配 |

复审 R3 replay（原树执行）：R2-01 RACE 拒绝保持 ✓；typed-consumer 四行
全绿（见上）。core/auto-lang 编译零 error；两修改文件 scoped rustfmt
--check exit 0。未运行整档（merge-owned：冻结裸 t+tv、777 strict 复核）。

## 复验入口

```text
python docs/plans/attachments/811/review-r3/replay.py D:/autostack/.wt/lang-811/auto-lang
cargo test -p auto-media-core --lib     # 40/40
python docs/plans/attachments/811/v05-harness/identity_matrix_run.py <wt> <scratch> <out>  # 需先 gen+build rust-identity
python docs/plans/attachments/811/v05-harness/vm_media_run.py     <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/vue_media_run.py    <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/merged_media_run.py <wt> <scratch> <out>    # 需先 gen+build rust-app
python docs/plans/attachments/811/v05-harness/gen_media_http.py   <wt> <scratch> <out>    # 需先 gen+build media-back-proj
```
