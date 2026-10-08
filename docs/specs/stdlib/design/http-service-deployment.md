# HTTP 服务部署基线（独立启动、配置、预算、观测、关闭与代理验收）

> **Status**: current（PLAN-736 交付；决策/配置/策略/生命周期/代理/负载/验证报告见
> `docs/plans/reports/736-http-{decision,config,policy,lifecycle,proxy,load,verification}.md`）
> 层：auto-lang（配置/观测/VM 传输桥）+ auto-man（进程托管/生成模板）+ auto-cli（`auto service`）
> 2026-10-08

本文是 VM 默认 API HTTP 与 auto-man 生成 Rust HTTP **两服务轨共用的部署合同**：
服务入口、profile 与严格配置、连接/请求预算、CORS/Host/单层代理信任边界、
健康/就绪与有界观测、受控关闭，以及"本机开发 + 受单层反向代理保护"的支持等级
与验收基线。协议/业务正确性（API 传输、文件、上传、handler 生命期）归各自
canonical；本文只定**服务如何被装配、约束、观测和关闭**。

## 1. 支持等级（首期认证范围）

- **已认证**：本机参考服务，受**单层**可信反向代理保护（TLS 终止 + edge 鉴权在
  代理）；IO 型 API（JSON/SSE/文件/上传）。两轨（VM 进程内 / 生成 Rust 子进程）
  均经真实 nginx 1.31.6 + 测试 CA 实测（`deploy/http-service/nginx.conf.example`
  + `examples/http_server/deployment/` fixture，runner 现场生成测试证书不入库）。
- **不在等级内（明示）**：直接公网、内置 TLS（Auto 不是 HTTPS 实现）、HTTP/2/3、
  完整 WebSocket（简化 echo 在 service profile 默认关闭）、多层代理、跨 OS 生产
  实机认证、VM CPU 抢占/阻塞 native 隔离。要求这些能力须另立计划，不在本文添加
  模糊安全保证。
- 遗留债不经本文宣布清偿（734 的 Tauri/typed 资源债、729 中间段 TOCTOU、
  730 无崩溃恢复/全局磁盘配额按原规范边界披露）。

## 2. 服务入口与配置

- **入口**：`auto service <project> --server vm|rust [--http-config <json> |
  --http-config-inline <json>] [-B <port>]`。只启动服务端：无 Vue/Vite、无桌面 UI、
  无 webview。命名注记：原拟 `auto serve` 与 Plan 269 AutoVM daemon 撞名，
  服务入口定名 `auto service`；`auto serve` 保持 daemon 语义零改动。
  生成 Rust 轨产物本身可直接 `cargo run --release` 启动（同一 service JSON 消费）。
- **配置单一来源**：`HttpServiceConfig`（`crates/auto-lang/src/http_service_config.rs`）
  版本化、纯数据、不依赖 Axum/VM；严格解析（deny unknown fields、重复字段指名、
  非法范围指名、非法组合指名），启动时解析一次，显式实例持有，**不按请求读 env**。
  生效摘要带每个值的来源；`effective_config_hash` 与 734 implementation hash 分开
  ——运行值不假称新业务实现。
- **profile**：
  - `development`（缺省）：`127.0.0.1:8080`；legacy CORS `*`（credentials=false）。
  - `proxy_service`：缺省 loopback；`allowed_hosts` 必填非空；CORS 缺省无跨域授权，
    允许显式精确 origin 列表；内部媒体/照片扫描路由默认 off（opt-in 需显式能力）。
  - 非 loopback 必须显式 `listen.addr`（hostname 拒解析、0.0.0.0 须显式写出）；
    监听权限/端口冲突/非法设置 = 指名错误**非零退出**，不静默换端口或回默认。
  - `port=0` 只用于独立 serve/test：ready 上报内核分配的真实地址；UI split 模式
    未闭合动态地址发现时须拒绝 0，不生成 `:0` 客户端。
- **旧链兼容**：无配置文件时 `auto run` 链维持原端口优先级（CLI `-B` > pac.at
  `back_port` > `AUTO_HTTP_PORT` > 8080）零改动。显式 http-config 定义完整 listen：
  CLI 显式 `-B` 可覆盖其 port，未显式的 pac.at 端口不再覆盖它；与显式 legacy
  env/listen 冲突时诊断，不偷偷优先。VM 旧默认 `0.0.0.0:{AUTO_HTTP_PORT}` 的
  宽监听仅在 legacy 默认入口保留；**服务路径默认 loopback 是对旧行为的显式安全
  调整**（迁移方式=写显式 listen）。
- bind 地址、本机 control/probe 地址与 advertised client URL 三者分离；
  `0.0.0.0` 不能直接成为浏览器连接地址。

## 3. 资源预算（两轨同值；0=无限不存在）

```text
max_connections=128  max_inflight=64  request_timeout=30s  drain=10s
header_buf=64KiB/≤100 头  header_read=10s  body=10MiB/10s
（普通默认值；配置可有限下调/明确上调，合法范围冻结于决策报告 §3）
```

- **连接许可**覆盖 idle/读头/keep-alive/发送/upgrade；连接满在**解析前**关闭并计
  `conn_rejected`（不承诺此时能回 503）。VM 轨 = accept 处 `Semaphore::try_acquire`；
  生成轨 = hyper-util accept loop 同语义。
- **普通请求许可**（queue+running+parked 生命期）满 = 503 + `Retry-After`；
  许可覆盖到 body 终态：生成轨经 response extensions 由 hyper 在 body 完成/断连
  后释放；VM 轨经 scope/FileBodyAdapter/FrameStream 代持（不以 headers 送达为释放点）。
- head/body/期限约束在进入业务前生效（413/408 按既有约定；Hyper 原生 431/慢头
  直关为框架差异，明示保留）。新上限覆盖路由错误/预检/健康/内部路由，无隐藏绕过路径。
- 文件/上传/SSE 沿用 729/730 专属预算与期限（本期**零触碰**其规范）；上传早拒/
  100-continue 前不拉 body，普通 30s handler 期限不误杀 10min 接收。
- 所有 body 在断连/取消/服务关闭时传播终结：生成轨流/body Drop 释放普通+连接
  许可及宿主资源，不只靠 VM scope 回收。

## 4. CORS / Host / 单层代理信任 / 鉴权责任

- 共享**纯策略**（config 模块）两轨各自装配，Axum 0.7/0.8 对象不跨轨共享。
- 策略链在 **body/上传预检之前**全短路（拒绝路径零文件系统工作）：Host 允许表
  （未允许 400 零业务）→ 可信代理身份 → 限速 → CORS preflight/附件。
- CORS：精确 origin 匹配；credentials 恒 false 且禁止 `*` 混列；恰当 `Vary`；
  无 Origin 不获 allow 头；不允许以 CORS 拒绝代替鉴权。正常/框架错误/SSE/file/
  upload 路径同一策略。Hyper 在 service 前产生的 431/断连只记 connection 事件，
  不虚构其 CORS/request-id。
- **单层可信代理**：`trusted_proxy_ips` = numeric IP 显式列表（默认空）；仅
  **socket peer** 命中才消费代理重写的单段 X-Forwarded-For IP 与
  `http|https` 的 X-Forwarded-Proto；多段/坏值/歧义拒绝（记 `xff_rejected`）。
  不信任客户端 X-Real-IP/Forwarded/任意链；不做 DNS 解析；X-Forwarded-Host 不用于
  放宽 Host 检查。有效身份用于日志与既有按 IP 限速；socket peer 独立保留。
- **限速**：按 IP bucket 表有界（默认 4096 条 + TTL 过期回收），满表新身份保守
  429 而不绕过限速；非分布式限流。
- **鉴权责任**：`auth_responsibility = app|edge` 显式声明；edge 模式代理须保证
  backend 无绕过路径（样例 backend loopback + 代理鉴权）；service profile 中
  应用 middleware 编组/执行错误 **fail-closed 500**；普通旧开发语义如需保留须
  显式矩阵。配置不构成文件系统沙箱/CSRF/CPU 故障隔离。

## 5. ready、健康与控制面

- 状态机 `Starting → Ready → Draining → Stopped`；ready 信号在 **bind + 路由/VM
  初始化 + 734 generation 验证** 之后才发；ready 载荷含四元组身份
  `instance_id / bound(真实地址) / profile / config_hash`。
- 旧实例/旧 bundle/生成失败/初始化失败**不得 ready**：占口 = bind 失败非零退出；
  stale generation = 734 freshness 门拒绝复用；启动期 health 返回 503（旧 TCP-connect
  判 ready 的语义已废弃——health 优先探针：503=未 ready，404=legacy 无控制面按
  可继续处理）。
- 控制面路径 `/__auto/health/live`、`/__auto/health/ready` 开放；
  `/__auto/snapshot`、`/__auto/shutdown` **仅 loopback socket peer**（真实 peer 检查，
  非 header）。health 不经过业务 VM/auth、不读 VM/不调用外部服务；受独立小预算，
  拥塞时可失败，不虚称永远可用；返回最小状态/身份，不暴露配置根/路由/资源详情。
  与业务同名路由在装配时诊断。edge 代理默认不对外开放操作面（模板 403 双保险）。
- 已知边界：父进程当前以子进程退出码 + 状态码探针承载 rust 轨 ready 判定，
  不消费 health 身份体做核对（P736-R1 纵深防御增强候选）；身份一致性由同 JSON
  注入构造保证并经代理实测核对。

## 6. 观测（有界 JSONL）

- 事件：startup/config_error、request 完成（method/route 模板≤128 字符/query 剥离/
  status/bytes_sent/outcome=completed|canceled|error）、drain/stop、
  resource/connection 拒绝、state 迁移。请求 ID 只接受 1..64 可见 ASCII 安全子集
  `[0-9A-Za-z._:@-]`，不合重生成。
- **不记录**：Authorization/Cookie/原始 query/body/token/文件绝对根/上传 filename；
  高基数字段不成 counter 维度；serde 编码，禁原样拼接用户值（防换行注入）。
- 响应 headers 成功 ≠ body 成功：流/文件以 body 终态记 completed/canceled，
  `finalize_scope` 单点恰一次（slot take + swap 守卫，重复 cancel/drop 不双计）。
- sink 有界（sync_channel + 独立写线程，`try_send` 队满丢弃并计 `log_dropped`），
  业务不等待磁盘日志、不阻塞 reactor。测试/本地内部 snapshot（connections/requests/
  parked/stream/file/upload/日志队列等）仅供 loopback 消费，**不是公网 metrics API**。
- 已知边界：VM 桥 stderr 的 413/超时诊断打原始 method+path（可含 query），在
  JSONL sink 合同之外（P736-R3，后续脱敏）。

## 7. 受控关闭与进程退出

- shutdown 端点与 Ctrl+C/SIGTERM（平台支持矩阵以决策报告为准；Windows headless
  记注入信号范围，不冒称实机 console 证据）汇入**同一 watch**。
- 语义：停 accept → ready 翻转 503、停止接收新业务 → 排空在途（drain，默认 10s）
  → 超期强制终结连接/task 并取消原 scope/stream/upload → `Stopped` + 端口释放可复绑。
- 在途 FS 操作等实际退出后才计许可归还（逻辑取消 ≠ 立即退出）；有限关闭保证适用
  于可取消 IO 参考负载；VM 无限 CPU 段/不可中断 native 不在保证内；不把 parent kill
  后的崩溃退出记成优雅关闭。P729-D1 专项（drain 窗内在途文件体跨窗完整送达 +
  ready 不再 200 + 端口重绑）由真 TCP e2e `http_e2e_plan736_file_body_drain_window_and_rebind`
  锁定。
- 进程面：rust 子进程有界排空后 parent 退出；VM 网络线程 join；CLI 正常关闭 0，
  bind/config/generation 错误非零；端口与自己创建的子进程零残留。现有 UI split 关闭
  管理复用同一句柄。

## 8. 部署模板与代理边界

- `deploy/http-service/nginx.conf.example`（基线 nginx 1.31.6）：TLS 终止、明确
  Host/CORS/auth 责任、access 规则、HTTP/1.1 upstream；JSON/SSE/upload 路径分组——
  SSE `proxy_buffering off`，upload `proxy_request_buffering off` 并对齐 wire/idle/
  total 边界；关闭非幂等自动重试防重复上传。`proxy_read_timeout` 是读取间隔期限
  不等于上传总期限。模板覆盖（而非追加）用户 XFF。`nginx -t` 只是一环，验收必须
  真实代理 wire（首帧/早拒/hash/Range/身份核对）。
- 正在使用的 playground 部署文件（远端专项站）不在本文范围、零触碰。

## 9. 验收基线（固定负载门）

小型参考服务本机门（全参数在 `scripts/http-service-probe.py`，有界并发 + 总
deadline，无无限形态；release fixture 排除冷编译；同机单实例顺序执行）：

- 普通 JSON：16 并发 × 10,000 请求 × 3 轮，零意外非 2xx，p95 ≤ 1s（首期实测两轨
  p95 3.9–12.8ms 量级，RPS ~6000）。
- 混合 IO：2 慢 SSE + 2 慢下载 + 2 上传 + 8 并发短 JSON × 60s；健康/短 JSON p95 ≤ 1s，
  文件 hash 一致，无超预算/无界队列。
- 超载/恢复：连接 cap=32、请求 cap=8，压 128 连接 + 64 burst；实际 conn ≤ 32、
  活动 request ≤ 8（health 独立额度计各自表）；释放后 5s 内回基线。
- 资源：同进程 10 个混合/取消周期；闲态 RSS 漂移 ≤ 64MiB、周期峰值 ≤ 512MiB，
  许可/job/temp/打开文件回基线（首期实测：VM 漂移 14.3/峰值 59.6；生成 2.7/15.2）。

这些是门槛而非性能承诺；无法达标须定位/修复或修订合同，不得事后放宽阈值。

## 10. 关联

- [http-server](http-server.md)（协议预算/装配/VM 宽监听 legacy 差异）、
  [http-handler-async-lifecycle](http-handler-async-lifecycle.md)（请求生命期/终态许可）、
  [http-server-files](http-server-files.md)（729 文件预算）、
  [http-server-uploads](http-server-uploads.md)（730 上传预算）、
  [api-transport-contract](api-transport-contract.md)（734 传输/生成完整性）
- auto-man：[project](../../auto-man/project.md)（进程托管/ready 探针）；
  auto-cli：[project](../../auto-cli/project.md)（`auto service` 入口）
- Design 33 D2b；债与观察项：KNOWN-DEBT-AND-RISKS.md P736-R1..R5
