# PLAN-727 T-07 报告：资源边界与取消收口（资源报告）

- 阶段：work（T-07）
- 性质：文件传输的内存/许可/句柄边界证据与取消收口实测。限额冻结值见
  [727-transfer-decision.md](727-transfer-decision.md) §7；本报告给实测与
  回基线证据。

## 1. 应用内存公式（结构性钉界）

```
传输峰值内存 ≈ max_pending_blocks(2) × app_block_bytes(64KiB)      # 网络→写盘管道
            + 进度/收据常量                                        # watch 通道单值
            + HTTP 库/OS 内核缓冲（不在应用公式内，见下）
```

- 下载管道：producer（bytes_stream → ≤64KiB 块）→ **有界 mpsc(2)** →
  writer 任务顺序写 staging。慢磁盘 = 通道满 → producer 反压（网络读取
  停止），内存不随文件大小增长。
- 上传管道：reader 任务（≤64KiB 块）→ **有界 mpsc(2)** → reqwest body 流。
  慢网络 = 通道满 → 磁盘读取反压。
- 进度：watch 单值槽（保留最新值）——慢/停消费者只丢合并进度，不阻止
  落盘与关闭；终态收据独立槽（不因通道满丢失）。
- HTTP 库/OS 缓冲（reqwest/hyper 连接缓冲、内核页缓存）不在应用内存公式
  内——与 724 内核的申报口径一致。
- 实证：`it_large_download_12mib_streaming`（12MiB 用 2×64KiB 管道完成，
  hash 对拍一致）；`plan724_kernel_stream_backpressure_high_watermark_bounded`
  同构背压语义已在内核面钉界。

## 2. 许可/计数回基线（实测）

| 资源 | 探针 | 用例 | 结果 |
|---|---|---|---|
| 传输活跃许可（4） | `transfer_active_available()`（实例）/ 在途注册表 `active_transfer_len()`（全局） | `plan724_kernel_cancel_dropped_future_stops_job_and_returns_permit`（同构语义）、`it_queue_saturation_rejects_then_recovers` 取消段 | 取消后回基线 |
| 传输队列许可（16） | `transfer_queue_available()` | `plan727_execute_queued_cancel_returns_permit_without_gate`（gate 不开的排队取消） | 取消后归还 |
| 文件操作许可（4） | `fs_ops_available()` | 单传输整个文件阶段持 1 许可；终态随 writer/reader 任务退出释放 | 结构保证 + 任务 join 回收 |
| 同目标仲裁表 | `in_flight_target_len()` | `it_queue_saturation_rejects_then_recovers` 尾断言 | 终态（含取消）→ 0 |
| 传输注册表 | `active_transfer_len()` | 同上 | 全部终态 → 0 |
| VM 注册表 | `vm_transfer_count()`（cfg(test)） | wait 消费/scope 组收口移除（`scope_finalize_transfers`） | 消费即清 |
| staging 文件 | 目录清点 | `plan727_download_full_commit_replaces_target`（提交后目录 1 文件）、`plan727_budget_exceeded_preserves_target`、`plan727_cancel_mid_transfer_preserves_target_and_cleans_staging` | 失败/取消后 0 遗留 |

## 3. 取消收口（FS 不可强制中止的实证边界）

冻结规则：取消 = 停止 issuing 新文件操作 → **等在途操作完成**（写盘任务
在下一个操作边界检查取消旗标）→ 关柄 → 清理 staging → 释放许可 → 交付
终态。不把丢弃 async wrapper 等同于物理写盘已停止。

| 竞态点 | 用例 | 行为 |
|---|---|---|
| 排队中取消 | `plan727_execute_queued_cancel_returns_permit_without_gate`（T-02） | 退出排队、归还许可 |
| 建立期取消 | 内核 select 臂（open/select cancel） | 连接中止、Cancelled 收据 |
| 传输中取消 | `plan727_cancel_mid_transfer_preserves_target_and_cleans_staging` | 停网络 + 收口在途写 + staging 清理 + 原目标字节不变 |
| 提交 gate 期取消 | `plan727_commit_gate_late_cancel_does_not_rollback` | gate 后迟到取消不回滚（替换成功 = Success 收据） |
| VM scope 级联 | `register_scope_transfer` + `finalize_scope` 组收口（有类型 transfer 组，与流 id 分离） | 组内传输取消 + 出 VM 注册表 |
| watch future 丢弃 | `transfer_wait_async` guard（结构化取消）；progress 观察不取消 | wait future Drop = 取消；进度读取 Drop = 仅放弃观察 |

## 4. 与普通 HTTP/SSE 的配额隔离（实测）

- 传输许可组独立于 `ClientLimits`/`StreamLimits`（KernelInstance 内三组
  信号量）：`plan707_client_stream_permits_independent_of_client`（既有）
  + 本计划传输组同构隔离。
- 大传输不挤占普通响应配额：传输 job 走 `transfer_active/transfer_queue`，
  普通 `execute` 走原组；共享内核 runtime/连接池（决策 §7）。
- 队列语义冻结：**在途总量上限 = queue_capacity(16)**（队列许可提交即取，
  其中至多 max_active(4) 并发执行）；第 17 笔提交即终态 QueueFull——
  `it_queue_saturation_rejects_then_recovers` 实证（初版测试误设 4+16=20
  在途，实测纠正为总量 16，语义与内核 execute 一致）。

## 5. 预存红登记（非本计划引入）

- `plan707_client/stream` 族 11 例在 **master 主检出同过滤即红**（注册表
  跨测试污染族，失败集随序漂移；主检出 3 红 vs 本 worktree 2 红）。与本
  计划触面无因果（T-05 改动前基线同红）。已在复审时按名核对。
