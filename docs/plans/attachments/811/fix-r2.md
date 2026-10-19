# PLAN-811 R2 修复轮报告（review-r2 needs_fix → 修复闭合）

2026-10-19 · stage: work · 针对 [review-r2](../review-r2/review.md) 三项
findings 的修复、oracle 补强与事件合同闭合。修复树：同 worktree
`D:/autostack/.wt/lang-811/auto-lang`（`codex/plan-811-dev`），R2 受审
`33fc23208`，本轮修复提交：**`4a1d8fcf3`**（单提交：race 准入域合并
+bounded oracle+sweep/merged 接线；2 文件 +413/-82）。

## Finding closure 表

### F811-R2-01 · P2 · open_session 关闭交错漏会话准入

- **finding → invariant**：会话插入与 shutdown 清表/准入共用同一同步域
  ——关闭完成后任何交错路径都不能在表中留下可查询/可关闭的有效会话。
- **affected paths/states**：open_session 入场检查与 sessions 锁之间的
  转换窗口（复审以合法自定义 `Into<PathBuf>` 受控复现）；spawn 返回值
  未消费。
- **fix**：锁内复查 `closing`（与 shutdown 清表同一 mutex：竞态 open 要么
  插入在清表前——行被清掉，要么在锁内看到 closing 拒绝）；spawn 失败回
  滚已插行并返回 `""`。
- **permanent cases**：`open_session_racing_shutdown_never_inserts_a_live_
  session`（门控 Into 在转换内停车→完整 shutdown→放行：必须返回 ""、
  snapshot=unknown、close=false——同复审探针窗口的确定性复现）；
  `open_session_sequential_controls`（顺序关闭后拒绝+活 runtime 真开/
  populate/close 正控）。
- **current results**：复审 replay.py R2 探针在本树
  `returned_session=""`、`snapshot={"error":"unknown_session"}`、
  `close_session_after_shutdown=false`（修复前活会话复现）；core 38/38。
- **remaining gap**：无。

### F811-R2-02 · P2 · 生命周期 oracle 不成立（截止/满通道）

- **finding → invariant**：外截止设施必须能在截止内红且不等卡死线程；
  resize 通道满必须有实际观测证明；scan 竞争不靠 sleep 猜。
- **fix/permanent cases**：
  - `bounded_teardown`（detached 线程；超时即 panic **不 join**）+
    `bounded_teardown_negative_control_fails_within_the_deadline`（1500ms
    任务/100ms 截止：<800ms 内红）与 `bounded_teardown_positive_control_
    completes`（正控）——负控+正控常驻。
  - `shutdown_with_real_decode_backlog...`：3200x2400→1920x1440 Lanczos3
    大 rendition 使单 resize 泳道成瓶颈；**采样 registry running 峰值
    ≥9**（2 解码+1 缩放+≥6 通道等待）先被断言，再做 bounded shutdown。
  - `scan_shutdown_reaps...`：**活动门**（queued>0 轮询，5s 上限，不再
    sleep 猜）+ 新 **per-registry `mutation_generation`**（与全局媒体代次
    同点自增、registry 作用域——并行测试下不受邻居噪声影响；比 stats 强
    因含失败条目路径）做静止断言。
- **current results**：core 38/38；负控实测 0.10s 档内红。
- **remaining gap**：无（复审所列 V-03 范围内）。

### F811-R1-03（续 OPEN）· 三形态事件验收

- **finding → invariant**：三形态真实媒体运行 + ImageSurface 渲染后事件
  （含静止期注册、卸载退订、形参投影）+ 生成媒体 HTTP；收据绑定提交/
  输入/源码。
- **fix**：
  1. **sweep 早退门重排**（renderer.rs `media_notify_sweep`）：视图收集
     先行，跳过条件加「当前视图无带回调面」——静止期新挂载 surface 必
     注册并派发（修复前表空+pending0+代次未变即漏注册）。
  2. **merged（静态组件）事件接线**：`typed_media_notify_sweep` 挂
     `devtools_update` 尾部——两拍 Ready→单发 on_loaded（带 resolve 实
     尺寸投影）、终态失败单发 on_error、表随视图生死；遍历走
     `vnode_converter::extract_children_ref`（MCP 寻址同源）；
     `DevToolsWrapper` 增类型化通知表。此前 merged 无任何派发路径
     （PLAN-095 只接 run_session）。
  3. 探针/验收面升级：VM 探针 `ImageLoaded(w,h)` 形参投影 + `Late`（静
     止 2s 后无重排队绑定）+ `Unbind`；Vue 探针 `Bad`（真 `<img>` error
     事件）；merged harness 恢复完整事件断言；四 harness 收据均嵌
     **provenance**（worktree commit/探针源码 SHA-256/二进制 SHA-256/启
     动命令），探针源码冻结于 [v05-harness/probe-sources/](../v05-harness/probe-sources/)。
- **current results**（收据 [v05-receipts/](../v05-receipts/)，全部本树
  实跑原始结果）：
  - **VM/Iced 12/12**：loaded 恰好一次+**投影尺寸**（A=(1024,512)、
    B=(768,768)——viewport 保纵横比的真实 rendition 尺寸）、稳定、
    A→B→A **URI+尺寸双精确**、**静止期 Late loaded=4**（早退门修复的
    真机反例）、**Unbind 后无杂散事件**（退订）、坏票 error 恰好一次、
    恢复 loaded=5。
  - **Vue split 8/8**：浏览器真解码（natural 2×1）、loaded 恰好一次、
    稳定、**真 `<img>` error 事件恰好一次**（error=1 loaded=1）、恢复
    loaded=2 error=1；截图 ready/error/recovered 三帧。
  - **Rust merged 11/11**：Init 真会话、**loaded 恰好一次（0.2s 内两拍
    语义）**、稳定、**坏票 error 恰好一次**、**恢复 loaded=2**、进程健
    康+净退、**静止期 Late loaded=2**、**卸载无杂散事件**——merged 事件合同全矩阵闭合。
  - **生成媒体 HTTP 13/13**（含 provenance）。
- **受影响面复验**：`cargo nextest -p auto-lang --lib -E 'test(media) or
  test(image)'` 40/40、`plan811` 7/7、parity PASS；792 driver 重跑收据
  见 handoff 增补（merged manifest 变更后的受影响生成发行面）。
- **remaining gap**：无——merged 探针已补 Late/Unbind 按钮，
  merged-quiescent-late-bind-fires-loaded（静止 2s 无重排队绑定→
  loaded=2）与 merged-unbind-no-stray-events（卸载后计数不动）双反例
  入收据（merged 11/11）。

## 复验入口

```text
python docs/plans/attachments/811/review-r2/replay.py D:/autostack/.wt/lang-811/auto-lang
cargo test -p auto-media-core --lib     # 38/38
python docs/plans/attachments/811/v05-harness/vm_media_run.py     <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/vue_media_run.py    <wt> <scratch> <out>
python docs/plans/attachments/811/v05-harness/merged_media_run.py <wt> <scratch> <out>  # 需先 gen+build rust-app
python docs/plans/attachments/811/v05-harness/gen_media_http.py   <wt> <scratch> <out>  # 需先 gen+build media-back-proj
```

本轮触面：auto-media-core（准入域+oracle+mutation_generation）、
auto-lang renderer（sweep 门重排+merged 类型化 sweep）。scoped rustfmt
（两文件 + 复审十文件口径）--check exit 0。未运行整档（merge-owned）。
