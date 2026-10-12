# PLAN-736 T-01 决策报告：HTTP 服务部署基线——最终基线、配置与服务运行原型

> 计划：[PLAN-736](../../736-http-service-deployment-and-operations.md) · 状态：executing ·
> 基线：master `de3a64353`（含 PLAN-734 合入 ec0eae41f / 归档 752cb37f4、PLAN-737 合入 59ff4e66f）·
> worktree：`D:/autostack/.wt/lang-736/auto-lang`（分支 `plan-736-dev`）·
> 日期：2026-10-03 · 本报告绑定 plan_revision 1

## 1. 前置核验（734 最终边界）

- 734 R2 复审 pass（R1 三项 F-1/F-2/F-3 已在 `a5284dd84` 回工闭合，D6 Tauri fixture 落地）；merge 收据
  `752cb37f4`（SD-01..07 沉淀、landed ff-only@ec0eae41f）+ cleaned `5ccb3d6c5`。工作交接 outcome: pass。
- **接受债（不冒称清偿）**：P734-D4（泛型 stdlib json.encode i32-lane）、P734-D5（run_with_capture
  测试基建非确定挂死）、P734-D7（D9 iterator/Response 臂注册表命中制兼容回退——R2 冻结 wire 裁定）。
  736 不承接、不宣布清偿；涉及时按 [api-transport-contract §5](../../../docs/specs/stdlib/design/api-transport-contract.md) 兼容制消费。
- 734 可复用地基：`generation.json` ready 记录（FNV 源指纹）+ `AUTO_REUSE_BACKEND=1` 复用新鲜度门
  （`rust_ui.rs::backend_generation_is_fresh`）；strict 解析门；8 生成入口 Err 硬传播；
  `api::contract` 分类单源（ResponseKind/ParamKind/is_upload_param）。

## 2. 静态证据复核（最终源码逐点，plan §4.2 → 复现结论）

| # | 位点 | 复现结果（行号为 de3a64353） |
|---|---|---|
| E1 | VM 默认 API 宽监听 | `crates/auto-lang/src/lib.rs:1736-1740`：`AUTO_HTTP_PORT` env（默认 8080）→ 拼 `0.0.0.0:{port}` → `serve_async`。**证实**：VM 轨默认 0.0.0.0，与生成轨 loopback 不一致 |
| E2 | VM 无连接总许可 | `http_transport.rs:122-174`：accept 循环每连接无条件 `tokio::spawn`（:149），无 semaphore/permit；请求 scope 有界（`AUTO_HTTP_MAX_INFLIGHT`=queued+running+parked 总上限）但**连接数无界**。**证实** |
| E3 | VM 预算部分常量/坏值静默回默认 | `http_transport.rs:64-88`：header 64KiB/慢头 10s/body 10MiB/10s 为常量；`AUTO_HTTP_MAX_INFLIGHT`/`AUTO_HTTP_REQUEST_TIMEOUT_MS`/`AUTO_HTTP_SHUTDOWN_DRAIN_MS` 坏值或 0 静默回默认。**证实** |
| E4 | VM CORS 静态拼装 | `http_server.rs:307-327`：`AUTO_CORS_ORIGIN`（默认 `*`）+ 固定方法/头块追加到**每个**响应；无 Vary、无 credentials、无 preflight 分支、非精确 origin 匹配。**证实** |
| E5 | request-id 原样透传 | `http_server.rs:4418-4427`：`x-request-id` 非空即整串接受（无长度/字符集校验）。**证实**（计划 §5.4 要求 1..64 可见 ASCII 安全子集，否则重生成） |
| E6 | 生成 main 假 ready + unwrap | `api_gen.rs::generate_main_rs:3769-3845` 两分支（734 后合入版）：`println!("Server running...")` 在 `TcpListener::bind().await.unwrap()` **之前**（:3781/:3821 vs :3842）——启动者可见"running"但 bind 可能失败（端口占用=panic 而非非零诊断退出）；`axum::serve(...).await.unwrap()` 无优雅关闭。**证实** |
| E7 | 生成 main 宽 CORS + 内部路由无条件挂载 | 同上：`CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any)`（:3786-3789/:3823-3826）；`/api/media/scan`、`/api/photos/scan|thumb|full` 无条件安装（:3792-3802/:3829-3839）。**证实** |
| E8 | TCP connect 探针假 ready | `rust_ui.rs:3791-3917`（start_api_server）、`:3935-4006`（start_vm_server）：`TcpStream::connect_timeout` 轮询判 ready——**端口上任何监听者**（旧实例/他进程）都算 ready；AUTO_REUSE_BACKEND=1 复用臂已有 734 源指纹新鲜度门（:3803）。**证实** |
| E9 | CLI 端口链 | `auto/src/main.rs:1051-1087`：Run 路径 `-B` > pac.at `back_port` > 不设（下游默认 8080）；注入 `AUTO_HTTP_PORT` + `AUTO_HTTP_BASE`（恒 `http://127.0.0.1:{port}`）。`util.rs:168-173/292-294`：`http_port()` env 坏值回 8080；`http_base_url()` 硬编码 loopback。**证实**：advertised URL 与 bind 地址是两个独立事实，现状 VM 轨 bind 0.0.0.0 而 URL 恒 127.0.0.1 |
| E10 | 子进程硬杀 | `rust_ui.rs:3921-3926` `stop_api_server` = `child.kill()`（TerminateProcess），无排空。**证实** |
| E11 | VM 限速桶无界 | `http_server.rs:335-341`：`RATE_BUCKETS` `HashMap<String,(u64,u32)>` 无容量上限/无 TTL 回收（`http.rate_limit(n, ms)` API 启用后）。**证实**（计划 §5.3 要求有界桶表+TTL） |
| E12 | 文件/上传预算独立 | 729/730 契约在案（[http-server-files](../../../docs/specs/stdlib/design/http-server-files.md)/[http-server-uploads](../../../docs/specs/stdlib/design/http-server-uploads.md)）；`http_file_service.rs`/`http_upload_service.rs` 自持额度与 receive/lease/commit 门。736 **不复制协议逻辑**，仅在服务层补终结传播与计数。 |

## 3. 冻结决策

### 3.1 CLI 面与配置来源（AC-01/02）

- 新子命令 **`auto serve [dir] --server vm|rust [--http-config <path.json>] [--http-config-inline <json>] [-B <port>]`**：
  仅解析/生成+启动服务，**不启动** Vue/Vite/桌面/webview。`--server rust` 走既有 `api_gen` 真实生成链
  （strict 门 + generation.json），产物可直接独立启动；`--server vm` 进程内 VM 网络轨。
  现有 `auto run` 面不动（UI split 维持现状）；merged/IPC 模式收到服务专属配置=显式诊断拒绝，不假装生效。
- **无配置 legacy 面不变**：`auto run` 的 `-B > pac.at back_port > AUTO_HTTP_PORT > 8080` 链原样保留
  （E9）。显式 `--http-config` 定义**完整服务 listen**：显式传了 `-B` 时覆盖 config 的 `listen.port`；
  未显式 `-B` 时 pac.at 端口**不再**覆盖 config listen；config listen 与 legacy env（`AUTO_HTTP_PORT`
  已在环境中设值且无 `-B`）并存时**启动诊断报错**（非零退出），不静默择一。
- **默认行为安全调整（对 VM 旧默认的显式变更，文档给迁移方式）**：development profile 缺省 listen
  **`127.0.0.1:8080`**（旧 `auto run` VM 轨 0.0.0.0 默认**保留不变**——legacy 面不悄悄改端口；仅 `auto serve`
  与显式 http-config 面用新默认）。迁移方式：`auto run` 用户要恢复外网可达→ pac.at/`-B` 不变，另用
  `--http-config` 显式 `0.0.0.0`；`auto serve` 用户要宽监听→ 显式 listen.addr。
- **port=0**：仅 `auto serve`/测试允许（OS 临时端口，ready 报真实 `local_addr`）；`auto run` UI split
  链未闭合动态地址发现→ serve 配置仅作用于服务端时允许 0，但 `AUTO_HTTP_BASE` 注入链（前端消费）
  拒绝 0（诊断，不生成 `:0` 客户端 URL）。
- 配置 JSON（v1 schema）字段与合法范围（**0 永不表示无限**；越界=指名诊断非零退出）：

| 字段 | 默认（development） | proxy_service 缺省 | 合法范围 |
|---|---|---|---|
| `schema_version` | 1 | 1 | 恒 1（其他值拒绝） |
| `profile` | `development` | —（必填） | `development \| proxy_service` |
| `listen.addr` | `127.0.0.1` | `127.0.0.1` | 合法 IP（含 0.0.0.0 显式宽监听） |
| `listen.port` | 8080 | 必填或 0 | 0..65535（0=临时端口，见上） |
| `limits.max_connections` | 128 | 128 | 1..=4096 |
| `limits.max_inflight_requests` | 64 | 64 | 1..=1024 |
| `limits.header_buf_bytes` | 65536 | 65536 | 8192..=1048576 |
| `limits.header_read_timeout_ms` | 10000 | 10000 | 1000..=60000 |
| `limits.body_limit_bytes` | 10485760 | 10485760 | 1024..=104857600 |
| `limits.body_timeout_ms` | 10000 | 10000 | 1000..=300000 |
| `limits.request_timeout_ms` | 30000 | 30000 | 1000..=600000 |
| `limits.drain_timeout_ms` | 10000 | 10000 | 100..=60000 |
| `cors.allowed_origins` | `["*"]`（credentials 禁用） | `[]`（=不授权任何跨域） | 精确 origin 或唯一 `"*"` |
| `cors.allowed_methods/headers/exposed_headers` | 旧固定表 | 显式列出 | 方法 token / 头名 token |
| `cors.allow_credentials` | false | false | 恒 false（本期） |
| `allowed_hosts` | `[]`（不校验 Host） | **必填非空** | 精确 host[:port] |
| `trusted_proxy_ips` | `[]` | `[]` | numeric IP 显式列表（单层） |
| `auth.responsibility` | `app` | 必填 | `app \| edge` |
| `rate_limit.{max_requests,window_ms,bucket_capacity,bucket_ttl_ms}` | 未启用 | 可选 | 桶容量默认 4096（1..=65536）、TTL 默认 60000ms（1000..=3600000）；满表新身份 429 |
| `features.websocket_echo` | true（现状） | false | bool |
| `features.media_scan/photo_scan` | true（现状） | false | bool |
| `observability.log_sink_capacity` | 1024 | 1024 | 64..=65536 |
| `health.request_budget` | 8 | 8 | 1..=64（健康端点独立并发额） |

- 未知字段=错误（严格解析，serde deny_unknown_fields）；重复键=错误；每个生效值记录来源
  （`config_default`/`profile_default`/`explicit`/`cli_override`）。effective_config 以**解析后不可变
  结构的 FNV hash** 暴露（与 734 implementation/generation hash 分开，不互相冒充）。
- file/upload 额度**不进本配置**（729/730 规范自管）；profile 不得未经其校验扩大这些额度，服务层只汇总有效值。

### 3.2 两轨 adapter 依赖（AC-01/03/06）

- **版本事实**：VM 轨 axum **0.8**（`auto-lang/Cargo.toml:327`，hyper 1.x + hyper-util）；生成轨 axum
  **0.7**（`api_gen.rs:4584` 生成 `axum = "0.7"`，同样 hyper 1.x 系）。**共享版本无关层**=
  `http_service_config.rs`（配置/校验/纯 CORS 决策/Host/代理信任/预算/观测事件）与
  `http_service_observability.rs`；**版本相关层**=连接驱动装配与 Response 类型（0.8 与 0.7 的
  `Response<Body>` 不互送，禁止跨轨复用 handler 对象）。
- **连接驱动统一形态**（两轨同构、仅 import 路径异）：替换生成轨 `axum::serve` 为 hyper-util
  `auto::Builder`（`http1_only`+`TokioTimer`+`max_buf_size`+`header_read_timeout`+`half_close`）
  手写 accept 循环——与 VM 轨 `serve_network` 同款。生成后端 Cargo.toml 模板新增
  `hyper = "1"`、`hyper-util = { version = "0.1", features = ["tokio", "server-auto", "service"] }`
  （0.7 生态兼容线）。连接 permit（`max_connections` 的 `tokio::sync::Semaphore`）在 accept 处
  try_acquire，满=**解析前关闭**（conn_rejected 计数，不承诺 503）；request permit 在 bridge 进入
  owner 队列前 acquire，满=503+Retry-After；**许可持有至 body/流/上传终态**（ Drop/取消/完成三路
  释放），不 headers 即放。
- **VM 轨改动位**：`http_transport.rs`（TransportConfig←HttpServiceConfig 注入 + 连接 permit）+
  `http_server.rs`（dispatch 前 CORS/Host/代理身份纯策略消费 + request-id 校验 + 观测事件）+
  `lib.rs:1731-1741`（serve 入口接配置）。生成轨改动位：`api_gen.rs::generate_main_rs`（main 模板
  重写：bind→identity ready→预算→关闭，media/photo 路由按 profile 门控）。
- **file/upload/SSE**：沿原专属期限与预算；上传端点不经 30s 普通期限截断 10min 接收；断连/关闭时
  普通与连接许可随终态释放（含生成 Rust 流 body Drop 路径）。WS 简化 echo：development 默认保留
  （现状），proxy_service 默认关。

### 3.3 ready 身份与健康（AC-02/05）

- **Ready 判据链**：bind 成功 → 路由/VM 初始化完成 → 734 generation 验证通过（生成轨=generation.json
  与当前源指纹一致；VM 轨=api.at 编译+路由表非空）→ **Ready**。任一失败=非零退出，**不输出 ready**。
- **身份四元组**：`instance_id`（进程启动时随机构造）、`generation`（734 指纹或 api.at 内容 FNV）、
  `bound_addr`（`local_addr()` 真实值）、`config_hash`（§3.1）。ready 时单行 JSON 到 stdout：
  `AUTO_SERVICE_READY {...}`（机器可读），人读行照旧。
- **健康端点**（两轨同路径）：`GET /__auto/health/live`（进程内 net 应答即 200，最小体）与
  `GET /__auto/health/ready`（200=Ready 未 drain，503=draining/未就绪；返回身份四元组 JSON）。
  不经业务 VM/auth，不读外部；受独立小请求预算（health.request_budget 并发额，与普通请求表分开）。
  `/__auto/health/snapshot`（**仅 loopback peer**）：connections/requests/parked/stream/file/upload/
  FS 许可/日志队列等原子计数快照——测试与本地观测面，非公网 metrics API。
- **父进程 ready 判定改真**：`rust_ui.rs` 的 TCP connect 探针替换为轮询
  `GET /__auto/health/ready` 直到 200 且身份匹配本次 spawn 期望（instance_id 由父进程注入子进程 env
  `AUTO_SERVICE_INSTANCE_ID`，子进程在 ready 记录与健康体中回显）——占端口的他者实例因身份不匹配
  不再被误判。`start_vm_server` 同法。
- 业务同名 `/api/*` 路由与 `/__auto/*` 冲突在装配时诊断拒绝。

### 3.4 关闭与进程管理（AC-06）

- **状态机**：`Starting→Ready→Draining→Stopped`。触发：VM 轨=Ctrl+C/SIGTERM（既有 watch 注入口，
  §7.3）+ 进程内 stop 句柄；生成轨=子进程自身 Ctrl+C（继承控制台时随父组到达）+ **loopback 专用
  `POST /__auto/shutdown`**（无 body、幂等；不进代理模板=不对公网暴露——headless 测试的确定性
  注入路径；Windows 无真实 console 信号注入能力，实机 Ctrl+C 证据范围如实记录）。
- Draining：停 accept、ready=503、拒新业务（503+Retry-After）、在途排空至 `drain_timeout_ms`，
  超期强制终结连接/task 并取消原 scope/stream/upload。在途 FS 操作**不中断**：等自然退出后才归还
  许可；drain 窗口不足=报告"forced_after_drain"，不伪称优雅。P729-D1（文件 body drain 窗 wire）由
  本期专项测试补齐。
- 父进程（`auto serve --server rust` / run split）：signal 或 shutdown 端点 → 等子进程有界退出
  （≤drain+5s）→ 超时 kill → 非零收场如实报告。退出码：正常关闭 0；bind/config/generation 失败非零。
- 端口释放后立即可重绑（复用 699 的 drop(listener)+E2E 验证法）。

### 3.5 CORS/Host/代理（AC-04）

- 纯策略函数（`http_service_config.rs`，两轨同源消费）：
  preflight（OPTIONS+Origin+Access-Control-Request-Method）→ 校验 origin/method/request-headers →
  200 空 body + 允许头；非 preflight 携 Origin → 命中才加 `Access-Control-Allow-Origin` +
  `Vary: Origin`（`*` 时 Vary 可省——development `*` 保持不加，精确列表必加）；credentials 本期恒 false
  （`*`+credentials 组合在配置校验期即拒绝）。
- **Host 校验**（proxy_service）：Host/`:authority` 不在 `allowed_hosts` → 400 零业务/FS；不新增
  regex；不按 `X-Forwarded-Host` 放宽。
- **单层可信代理**：`trusted_proxy_ips` 精确命中 socket peer 才消费 `X-Forwarded-For` 的**单个**
  重写 IP（多段/坏值=拒绝该头按 peer 记身份）与 `X-Forwarded-Proto`（仅 http|https）；`X-Real-IP`/
  `Forwarded`/任意链不消费；代理模板用 `proxy_set_header X-Forwarded-For $remote_addr` **覆盖**语义。
  effective client identity 用于日志与既有限速；socket peer 独立保留。
- **鉴权责任**：`auth.responsibility=edge` 时模板承担 TLS/鉴权/access 规则，后端只信任来自 trusted
  peer 的流量（本期限 loopback bind 兜底）；`app` 时应用自带 middleware 照旧，但 proxy_service 下
  middleware 执行/编组错误 fail-closed 500（开发语义矩阵显式列出差异）。fixture 有保护路由证明
  缺/错凭据拒绝且 body/FS 计数 0。
- media/photo scan 路由：proxy_service 默认不安装（404），development 保留现状；opt-in 需显式
  `features.*=true` 且仍受 Host/预算约束。

### 3.6 观测（AC-05）

- `http_service_observability.rs`：原子计数器组 + 有界 mpsc→后台写线程（**stderr** JSONL）；
  队满丢弃+`log_dropped` 计数，业务不等待磁盘。事件：`startup`/`config_error`/`request`（完成时：
  request_id/route 模板/method/status/header_latency_ms/body_duration_ms/bytes_sent/outcome
  completed|canceled|error|rejected）/`drain`/`stopped`/`conn_rejected`/`permit_rejected`。
- request-id：入参校验 1..64 可见 ASCII 安全子集（`[0-9A-Za-z._:@-]`），否则重生成 `req-<counter>`；
  日志字段 serde_json 编码（无字符串拼接注入面）；不记录 Authorization/Cookie/query/body/filename/
  磁盘根；高基数字段（路径原值、IP 明细入 counter 维度）不做 counter 维度。
- 响应 headers 发出 ≠ body 成功：流/文件/上传以终态事件收口（cancel/drop 不双计——幂等终态表）。

### 3.7 代理与运行环境冻结（AC-07，T-01 职责）

| 项 | 冻结值 |
|---|---|
| Nginx | **1.31.6**（scoop 用户级安装 `C:\Users\zhaop\scoop\apps\nginx\current`；2026-10-03 安装；`nginx -v` 实测） |
| OpenSSL | **3.2.3**（Git for Windows 自带 `C:\Program Files\Git\mingw64\bin\openssl.exe`） |
| 测试机 | Windows 11 专业版 26200 · x64 · 20 逻辑核 · 32,581 MiB RAM · 单机单实例顺序跑负载 |
| TLS | 测试 fixture 生成临时 CA+证书（仅 localhost SAN，**不入库**，`nginx -t` 前 materialize）；client 按 CA 验证，不用 `--insecure` |
| 拓扑 | HTTPS(client)→nginx(单层, TLS 终止)→HTTP/1.1 loopback→VM/Rust 后端；模板见 T-06 `deploy/http-service/` |
| 探针 | Python 3 标准库+requests（仓内既有探针依赖面），总 deadline+有界并发+OS 临时端口 |

§6.2 负载门条件冻结：release fixture 两轨（排除编译/建库时间）；同机单实例**顺序**执行（不与 tf/
其他负载并跑）；每档 ≥3 轮 + 10min 资源曲线按计划原文阈值（16 并发 10k 请求 p95≤1s；混合 IO 60s；
超载 conn cap=32/req cap=8 压到 128/64；10min RSS 界 64MiB/512MiB）——**不事后放宽**。合法流 fixture
选择（按 734 最终支持面）：SSE=`~Stream<str>`/`~Iter<str>` relay 形态（017-chat 形态）、文件=
FileResponse 端点、上传=UploadRequest 端点（730）；Tauri/merged/back-proxy **不**纳入服务认证。

### 3.8 最小独立 serve 原型

- VM 轨：`examples/http_server/api_contract`（734 契约 fixture）经 `auto run`（现状链）启动 VM server
  ——T-02 完成后改用 `auto serve --server vm --http-config`；原型验证 bind 默认/ready 输出/关闭，
  证据入 T-02 报告（本报告只冻结方式，原型执行随 T-02 门禁一并复现，避免双跑）。
- 生成轨：`auto generate`/`auto run --server rust` 现状产物直跑（loopback+Any+CORS），T-02 后走
  `auto serve --server rust`。
- 结论：两轨最小原型**可行**，无需新外部依赖（hyper-util 已在 0.7/0.8 生态内）。

### 3.9 支持矩阵（首期交付等级，写进 SD-01）

| 维度 | 本期认证 | 明示未认证 |
|---|---|---|
| OS | Windows 11（本机实测）+ CI Linux 面（既有 e2e 双平台） | 其他 OS 实机 |
| 拓扑 | loopback 直连；HTTPS→单层 nginx→HTTP/1.1 | 直接公网、多层代理、TLS 终止在后端（无内置 TLS） |
| 认证 | 测试环境 edge/app 责任 fixture | 账户/JWT/CSRF 系统 |
| 负载 | §6.2 参考服务本机门 | 任意应用吞吐、公网安全、分布式限流 |
| CPU | 同步 handler 段预算内跑完（无抢占，既有） | 无限 CPU 段强杀、不可中断 native |
| 进程 | 本期 serve/run 子进程管理 | merged/IPC/back-proxy/Builder 当部署服务 |

## 4. T-01 出口检查

- [x] 734 前置核验（§1）
- [x] 追踪复现 E1..E12（§2，最终合入源码）
- [x] 决策冻结：CLI/config/范围（§3.1）、adapter（§3.2）、ready/health（§3.3）、关闭（§3.4）、
      CORS/Host/代理（§3.5）、观测（§3.6）
- [x] Nginx/OpenSSL 版本与机器冻结（§3.7）；§6.2 条件冻结（§3.7）
- [x] 两轨最小原型可行性（§3.8）
- 无 blocked 项：依赖全部在位（nginx 本会话装齐）。
