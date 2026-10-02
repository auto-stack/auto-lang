# PLAN-730 T-08 生命期报告：期限切换、取消仲裁与资源回收

- 代码基线：@ `8a2fc567e`。

## 1. 期限分阶段（§5.3 合同实现）

| 阶段 | 期限 | 实现 | 证据 |
|---|---|---|---|
| header/鉴权/队列 | 普通请求期限（30s 默认） | scope deadline（RwLock+watch） | 早拒 401 立即回；queue 等待 bounded by queue_timeout/total |
| receive | total 默认 10min（headers 入 scope 起，排队计入） | `extend_scope_deadline`（只增不减）→ watch 唤醒桥 reply 循环重臂（`sleep_until(当前)+changed()`——修"select 保留旧捕获值"） | 38s 慢上传 201（>30s 普通期限）；owner parked 定时臂每轮重读 |
| idle | 60s（逐 chunk 强制） | 接收泵 select 的 idle sleep（每 chunk reset） | 单测旋钮 + 慢上传期间每 1.5s chunk 持续 reset |
| staged lease | 30s | 看门狗 spawn（commit/reject/过期自身取消句柄） | 300ms 旋钮单测：无人处理 → 自动清理 + 410 墓碑 |

## 2. 取消/断连

- 断连：hyper conn watcher → `cancel_scopes_for_conn` → scope cancel →
  body 流自然终结（receive 错误出口）或 staged 立即终态+清理。
- 半关闭：half_close(true) 既有语义保持（发完 body 半关不影响回复）。
- 取消仲裁：commit gate（hard_link）前取消 → 不发布+清理；gate 后迟到取消 →
  等实际结果（成功保留——不回滚已发布文件）。`upload_late_cancel_count` 探针。
- 磁盘不强制 abort：writer 在途写返回后清柄；writer 通道关闭自清 staging。

## 3. 资源回收（回基线证据）

| 资源 | 探针 | 回收点 |
|---|---|---|
| 会话注册表 | `upload_session_count` | 终态（commit/reject/过期/取消）出表——e2e/单测均断言回 0 |
| staging 文件 | 目录扫描 | commit 后 unlink；reject/失败/过期/取消/writer-drop 清理；e2e 断言目录空 |
| active/queue 许可 | `upload_active_available`/`upload_queue_available` | 终态释放（staged 占许可至终态——lease 不腾槽） |
| VM 注册表 | `vm_upload_counts` | 编组取出 / scope 组收口 |
| live-op | presence 守卫 | 迟到完成丢弃（complete=false 时结果条目同步清除） |

## 4. 缓冲公式（§5.3 应用缓冲上界）

parser carry ≤ boundary(70)+4 + part header buf ≤16KiB + 当前文本字段 ≤16KiB
+ 待写块 ≤2×64KiB（通道 1 + 在写 1）+ 单 wire 读帧。写入侧背压：通道满时接收泵
await（网络读暂停）——磁盘慢反压网络。计数探针：`upload_staged_count` 等。
（注：per-session 磁盘占用硬预算由 file/wire 上限承载：64MiB+65MiB 级。）

## 5. 已知边界（如实记录）

- 接收中（未 staged）的 VM 取消依赖 body 流终结（断连自然发生）；scope cancel
  经 cancel watch 选中接收泵——服务内单测覆盖 staged 态即时清理。
- 异常重启的陈 staging 扫描、全局存储配额、断电持久性不在本期（§计划非目标）；
  运行中临时空间由上述硬预算+lease 回收承载。
- 中间件空串返回 `""` 在既有实现中按 JSON 响应短路（Plan 352 文档语义分歧）——
  本计划不改写该契约；示例/测试用 `?str` nil 放行形态。
