# PLAN-736 T-03/T-04 报告：预算/许可与 CORS/Host/代理策略（AC-03/04, SD-01/02/03/04）

> 绑定：`700e916bc`（T-03）+ `1d65ff70e`（T-04）· 实机证据均为 worktree 实测。

## AC-03 证据：两轨连接/请求/头/body/期限预算实际有界

| 面 | VM 轨 | 生成轨 | 实测 |
|---|---|---|---|
| 连接许可 | accept 处 `try_acquire_owned`（满=解析前关闭+conn_rejected） | SERVICE_RUN_LOOP 同构（hyper-util accept loop） | **cap=4**：4 占位连接后第 5 连接 curl exit 56（立即关闭）；释放后恢复 404。**cap=2**（VM）：第 3 连接关闭、恢复 734734 |
| 头预算 | hyper `max_buf_size`+`header_read_timeout` ← config | 同（生成模板同参） | from_service_config 单源（env 不参与服务面） |
| body 预算 | 既有 10MiB/408 语义 ← config body_limit/timeout | `DefaultBodyLimit::max(body_limit)` | 单测锁定范围 |
| 普通请求许可 | 705 生命期许可（queued+running+parked）满=503+Retry-After（既有） | `__inflight_gate` 503+Retry-After；**permit 经 response extensions 持至 body 终态**（流式不 headers 即放） | 单测+实机（低预算 inflight=2） |
| 断连收口 | 连接任务终结→cancel_scopes_for_conn（705 既有） | extensions drop=终态释放 | plan705 族 80 绿含取消三阶段 |
| file/upload 期限 | 729/730 专属契约零触碰 | 同 | plan730 26 绿 |

## AC-04 证据：CORS/Host/单代理信任/鉴权责任

实机 **proxy_service 双轨矩阵**（真实 wire）：

| 检查 | VM 轨 | 生成轨 |
|---|---|---|
| 坏 Host → 400 零业务 | ✓（`{"error":"host not allowed"}`） | ✓ |
| media/photo scan 路由默认关 | ✓（装配面契约冲突诊断+门控） | ✓（`/api/media/scan` 404） |
| preflight 拒（空 origins）→ 403 | ✓ | ✓ |
| 实际请求带 Origin → 零 ACAO 泄漏 | ✓（dispatch legacy `*` 头块服务面停用） | ✓（legacy CorsLayer 收进 legacy 臂） |
| dev 面 CORS 附件 | preflight 204 ACAO=`*`+actual 200 ACAO=`*` | 同构 |
| request-id 校验 | 1..64 可见 ASCII 子集，不合重生成 | 同构（观测面） |
| 可信代理身份 | peer 精确命中才消费单段 XFF/XFP（伪造链拒绝） | `__policy_gate` 同构 |

单测：CORS preflight 三要素/Vary/`*`+credentials 拒、host 规范化（端口/IPv6/大小写）、
单段 XFF 语义、限速器有界+TTL+满表 429——`cargo t plan736` 10 测锁定。

策略纯度：Host/CORS/限速/代理身份全部先于 body 读取与上传预检（0FS 负序）；
middleware fail-closed：应用 middleware 编组错误面在 734 合同内已有 500 语义（T-08 复审复核项）。
