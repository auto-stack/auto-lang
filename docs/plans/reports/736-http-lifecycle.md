# PLAN-736 T-05 报告：观测、ready 与关闭/进程管理（AC-02/05/06, SD-01..05）

> 绑定：`0ff2b95a1` · 实机证据均为 worktree 实测（端口/日志/状态事件时序）。

## 状态机与身份（AC-02）

- `Starting→Ready→Draining→Stopped` 全链 JSONL state 事件实测时序：
  `ready → draining → stopped`（shutdown 端点触发；`VM owner loop exited (port released)`）。
- 身份四元组（instance_id/bound/profile/config_hash）三个消费点同源：
  `AUTO_SERVICE_READY` stdout 行、`GET /__auto/health/ready` 体、health/snapshot。
- ready 判据：bind + 路由/VM 初始化 + 无装配冲突（业务路由占 `/__auto/*` = 装配 Err；
  生成保留路径冲突 = exit 2）。

## 控制面（AC-02/05）

| 端点 | 准入 | 实测 |
|---|---|---|
| `GET /__auto/health/live` | 开放 | `{"state":"..."}` |
| `GET /__auto/health/ready` | 开放 | 200=身份四元组；503=starting/draining |
| `GET /__auto/health/snapshot` | **仅 loopback** | state+identity+counters |
| `POST /__auto/shutdown` | **仅 loopback** | `{"draining":true}` → 与 Ctrl+C 同一 watch |

## 观测（AC-05）

- JSONL（stderr，有界 sink 1024）：`startup`/`state`/`request`（恰一次终态：
  completed/canceled + status + bytes_sent + route）/`drain`/`config_error`。
- 恰一次：scope finalize 单点收敛（complete/cancel 双路径防双计——原子闸）。
- 恶意值脱敏：request-id 1..64 可见 ASCII 子集校验（不合重生成）；字段 serde 编码
  （无拼接注入面）；不记录 Authorization/Cookie/query/body/filename/磁盘根。
- 计数器与事件同点收敛：`requests_total/completed/canceled/bytes_sent` 实测
  3/2/6 与 JSONL 一致；sink 满丢弃+log_dropped 计数（单测锁非阻塞）。

## 关闭与进程管理（AC-06）

- 注入面：`POST /__auto/shutdown`（loopback）→ 同一 watch → 停 accept → 排空
  （drain_timeout）→ 强制收口 → 端口释放。双轨实测：VM（重绑成功）/生成
  （`AUTO_SERVICE_STOPPED` + 子进程 0 退出 + 父进程 "stopped cleanly"）。
- 信号面：Ctrl+C/（unix SIGTERM）既有 699 注入口保留；Windows headless 的
  真实 console 信号注入不做（SD-01 如实记录范围）。
- **P729-D1 专项**（T-06 落地）：`http_e2e_plan736_file_body_drain_window_and_rebind`
  ——真 TCP 在途文件体跨 drain 窗**完整送达**（64KiB 自然完成脸）+ drain 期
  ready 不再 200 + 端口重绑；4MiB 体超窗被强制收口的脸在案（行为=合同 §5.2
  "超过 drain 终结连接"，非泄漏）。
- 生成轨桩体路径 drain/文件在途：route-A 边界（P670-D1 域）与 SSE 竞速（698 域）
  记录于 736-http-proxy.md。

## 预存根因修复（本任务发现）

- `instance_id()` 的 OnceLock **同线程重入** `get_or_init` 死锁（closure 嵌套自调
  init_instance_id）——VM ready 臂挂死根因；修复+注释防复发。证据：探针序列
  （arm entered → 无后续输出）→ 修复后双轨全链绿。
