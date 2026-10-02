# PLAN-729 T-07 生命周期报告（慢发送/取消/关闭/配额）

- worktree `D:/autostack/.wt/lang-729/auto-lang`
- 命令：`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan729`（lifecycle 3/3 绿，总 9/9）
- 单元级（日常档）：`cargo t plan729` 19 绿（含 client-gone/finish-hook 恰一次/截断失败/百分比名 404）。

## 1. §6.2 逐项证据

| 项 | 结果 | 证据 |
|---|---|---|
| 慢客户端占满 active | ✅ | 4 黑洞（只读 headers 不读 body）→ `file_active_count()==4`（对账探针） |
| 断连回收（EOF/Drop/确证断连） | ✅ | 全部断开后 active→0（≤15s 观测窗）；服务随后正常服务新请求（200 全量 4MiB） |
| N+Q 饱和后确定 503 | ✅ | 4 active + 16 queued 后第 21 笔：即时 503 + Retry-After: 1（时间戳证据：serve 端 +0ms 出 503） |
| 排队期零句柄零缓冲 | ✅ | `file_queued_count()==16` 探针（排队者未 open 文件——打开在 active 之后） |
| 排队到期出队 | ✅ | 30s 准备期限到 → 16 笔 503（timeout 形态）→ queued 回 0 |
| 释放后恢复 | ✅ | 全释放 + 队列排空后新请求 200 |
| 727 完整下载 | ✅ | transfer_download → success；落盘字节 == 源（256KiB 确定样本） |
| 727 强 validator 206 续传 | ✅ | 本地 100_000B + offset+etag v1 → success；拼接 == 全量 |
| 727 If-Range 失配 200 重下 | ✅ | 错误 etag → success（完整替换，非 append） |
| 727 416/失败保留目标 | ✅ | 越界 offset 无 validator → failed；原目标字节不变 |
| 727 取消 → server 资源退出 | ✅ | transfer_cancel → cancelled；active→0（≤10s） |
| 单流两端绑定同次请求 | ✅ | 同一 server fixture/port，两端各断言（非独立 mock） |

## 2. 缓冲公式与计数（AC-04 对账）

- 每 active 应用侧缓冲 = 1×读块（≤64KiB）+ 1×通道帧（cap= max_pending-1 =1）≈ ≤128KiB +
  hyper 自身写缓冲（框架另行，不计应用预算）。读驱动（poll 才读）——无预读。
- fs op 并发 ≤4（信号量围每个读/seek；排队者不占）。
- 文件配额独立于普通请求：`/health` 类普通端点不经文件信号量（同一 VM 服务内并存；
  quota 测试中第 21 笔的 503 由文件队列即时给出，transport 层自身队列/许可不涉文件许可）。

## 3. 观察记录（如实）

1. **OS 缓冲吸收**：2MiB 样本下黑洞客户端可使 pump 全量完成（loopback 缓冲+hyper 写缓冲吞完
   2MiB）——配额测试样本改 64MiB 才能真实背压停住。测试矩阵据此定样本尺度。
2. **晋升语义**：active 释放后排队者立即晋升并开始发送（其客户端死活决定 pump 何时终结）——
   "排队取消即移除"在客户端侧持有连接时由其 socket 生命周期收口（RST/关闭），排队期限
   （30s 准备期）覆盖从未晋升的排队者。
3. **hyper 304 剥离 Content-Length**：表示长度在 304 上不可见（§1 注）。
4. **诊断过程中的一个 wire 级观察**：饱和状态下曾见"响应已构建但新 socket 30s 未收到"的
   假象——最终定位为测试自身中间请求时序（时间戳插桩证明 serve 端即时出 503 且第 21 笔
   即时送达；楔死的是恢复阶段的中间请求，其发出时刻与 16 个排队超时定时器到期重叠）。
   最终测试形态避开该时序（等队列排空后再断言恢复），服务端无对应缺陷证据。
