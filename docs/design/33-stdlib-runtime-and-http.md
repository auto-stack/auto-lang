# 33 - Auto 标准库多后台与 Web 服务运行时

> 状态：方案稿（2026-09-23 静态审计；2026-09-28 更新阶段状态与 C1 设计）；现状以 `docs/specs/` 与源码为准。
> 实施入口：阶段 A [PLAN-696](../plans/archive/696-stdlib-http-server-runtime-hardening.md)、阶段 B [PLAN-699](../plans/archive/699-vm-http-transport-axum-bridge.md) 已交付；阶段 C1 [PLAN-705](../plans/705-vm-http-handler-async-lifecycle.md) 待实施。
> 历史输入：[Design 13](13-networking.md)、[多平台填充草案](raw/stdlib-organization.md)、[HTTP 草案](raw/http-server-stdlib.md)。

## 1. 结论与适用边界

**阶段状态**：下文 §2 与 §5 的风险表保留 PLAN-696 实施前的审计基线，不能当成 2026-09-28 的现状。PLAN-696 已修复分段 body、慢 SSE 阻塞和 VM 指针跨线程转运；PLAN-698 已交付 VM publisher SSE；PLAN-699 已用 Axum/Hyper 替换 VM `#[api]` 手写 HTTP 默认入口，补齐协议/队列预算和优雅关闭。当前普通 handler 仍在 owner 线程同步驱动，异步 HTTP 等待存在忙等；PLAN-705 接续普通 handler 挂起/恢复、请求取消和结果生命周期。CPU 长计算、外部流和多后台收敛仍待后续。

Auto 当前足以支撑示例级、本机开发用的 CRUD API，以及已经验证的部分 SSE/媒体路径；不能据此认定 VM HTTP 入口已经具备通用 Web 服务器的协议正确性、并发隔离和运维能力。`#[api]` 是跨后台的用户契约，实际服务能力分散在 AutoVM 原生 shim、`auto-man` 生成的 Axum 服务、VM 合并调用、Tauri IPC，以及 gallery back-proxy 中。不能把“使用同一份 `api.at`”等同于“使用同一 HTTP 实现”。

保持 `api.at` 与 `auto.http` 的平台无关接口，以宿主实现传输层。Rust 生成轨继续用 Axum；VM 轨已由 PLAN-699 接入 Axum/Hyper，下一步补请求作用域、可取消异步等待与完成唤醒。Auto `task`/`~T` 仍是语言语义，不直接暴露 Tokio 句柄。Auto 自实现 HTTP 解析器可作为教学或协议实验，不作为默认生产服务入口。

## 2. PLAN-696 前审计基线：从声明到服务的路径

| 路径 | 入口/实现 | 目前可确认的行为 |
|---|---|---|
| 标准库声明 | `stdlib/auto/{io,net,async,http}.at` | 公共类型与函数；`http.at` 声称基于 net，但服务实际由宿主 Rust shim/生成代码承担。`http.rs.at`、`async.rs.at`、`io.rs.at` 当前不存在。 |
| VM 模块装配 | `compile.rs::resolve_use` 附近的 `.at` 后接 `.vm.at`；`vm/native_registry.rs` | 文本顺序拼接与 `#[vm]` 注册。`parser.rs::get_file_extensions` 列出三目标后缀，但方法标注 `dead_code`，不能单凭它证明所有装载路径都按目标选层。`autovm_persistent.rs` 另有自己的路径计算。 |
| VM `#[api]` HTTP | `lib.rs::execute_autovm*` → `http_server.rs::serve_async` | 单线程 Tokio `LocalSet`，`TcpListener` 接连接；手写 HTTP/1.1 解析、路由、响应、SSE 与简化 WebSocket；handler 经 `call_fn_by_name` 同步运行。入口用 `usize` 传递并重建 `!Send` VM 指针。 |
| VM Builder HTTP | `http.at` / `http.vm.at` → `ffi/stdlib.rs::shim_http_server_listen` | 与自动 `#[api]` 服务存在另一路同步服务器代码，语义和测试需分别核对。`axum_adapter.rs` 的 Router/extractor 是 VM 桥接命名与编组，最终仍由 `http_server.rs` 的 TCP 入口处理。 |
| Rust 服务 | `auto-man/src/api_gen.rs::generate_rust_server` | `api.at` 被提取/转译为 Rust handler 和 Axum `Router`；生成的 `main` 启动 Tokio 与 `axum::serve`。`crates/a2r-std/src/http.rs` 主要是 **HTTP 客户端**，使用 ureq，不是服务端 Axum 的共同实现层。 |
| VM 合并/IPC | `auto run` 的 `--server`/`--no-merge` 裁决、VM back 调用桥、`api_gen.rs` 客户端 | 合并模式直接调用 VM 后端；Tauri 生成 IPC 客户端；分离模式经 HTTP。三者需共享参数/错误/响应语义，但不共享网络传输。 |
| Gallery back-proxy | `back_proxy.rs` | 另有 `std::net::TcpListener` + 每连接线程 + 每 app VM session/消息队列；路径/参数编组部分复用 VM HTTP，协议入口仍独立。 |

`auto run --server` 目前接受 `vm` / `rust`；渲染目标、`pac.at api`、`--no-merge`、`--merged` 一起决定传输形式。不能仅凭 `--server=vm` 推断一定走 TCP。典型回归样本：015-notes（CRUD/JSON）、017-chat（SSE）、020-music-player（媒体）、023-realworld（认证/多路由）、027-file-manager（文件）、031-image-viewer（合并/媒体）。

## 3. 多后台装配机制：保留分层，补上可验证契约

### 3.1 当前机制与文档差距

主文件 `.at` 在前、同名目标文件（VM `.vm.at`、C `.c.at`、Rust `.rs.at`）在后的模型已写在历史草案和 `stdlib/project.md`。实际代码中 VM 的 `.at + .vm.at` 连接最清楚；Rust 轨还有手写/镜像的 `a2r-std`，仅部分模块有 `.rs.at` 与签名对拍；Vue 前端的 `use.web` / 适配器链属于 UI 目标装配，不等同于 `auto.http` 自动加载一个 `.vue.at`。`io.at` 的 `#[vm] read_line`、`io.vm.at` 的 `ext File`、`io.c.at` 的 C 字段填充说明公共声明与目标实现已经混合，但未见统一的“每个公开符号恰有一个实现”门禁。

审计前的 networking Spec 和 Design 13 曾把 `http.rs.at`、`http→net→async` 当作已落地依赖图；PLAN-696 已修订现状说明。PLAN-699 后 VM 与生成 Rust 均依赖 Axum，但仍是两条独立的服务/handler 实现，不能据此宣布共享 server 已落地。新能力继续随 review/merge 沉淀到 canonical Specs。

### 3.2 目标装配契约

1. 公开 `.at` 定义可移植语义、类型、错误与能力；目标文件只提供实现或内部资源布局。目标选择必须来自显式编译/运行目标，而非文件存在性猜测。
2. 装载器输出 manifest：公共符号、目标、选中的实现文件、版本/内容指纹、能力集。缺实现、重定义、签名漂移、目标不可用均在构建/装载时诊断；不可静默落到另一个后台。
3. 保留纯 Auto 实现复用；对文件、Socket、Server、Task 等宿主资源优先用不透明句柄/受控资源表。历史草案允许 `ext` 填充私有物理字段，这一规则在跨 VM/C/Rust 的 ABI、析构、泛型实例化验证前不能作为公共资源 ABI 的默认做法。
4. `web/vue` 是运行环境/前端适配维度，`vm/rs/c` 是执行目标维度；分别选择、分别诊断。浏览器不可用的文件/监听能力必须显式报 `Unsupported`，不能给空返回值冒充成功。
5. 建立“声明 → 后台实现 → 原生 shim/转译产物 → parity 样本”的机器可读覆盖表。`a2r-std` 的手写镜像暂作为明确登记的实现源，最终是否生成由单独证据决定。

## 4. HTTP 与 task 的目标边界

```text
api.at / auto.http 公共契约
       ↓ 路由签名、参数绑定、响应/错误、流语义
HTTP/IPC/合并适配器
       ├─ Rust：Axum + Tokio handler
       ├─ VM HTTP：Axum/Hyper I/O → 有界请求队列 → VM 所在线程
       └─ IPC/合并：同一调用契约 → 宿主传输
                                  ↓
                         Auto task / ~T / ~Stream
```

- **协议层**：让 Axum/Hyper 负责请求分帧、Header/Body、连接复用和标准 HTTP 行为。内部仅保留 Auto 路由、参数映射与返回值编组。Rust 与 VM 路径尽量共享可序列化的 `ApiRequest/ApiReply` 契约，而非共用 VM 对象。
- **VM 所有权**：在专属线程构造并持有 `!Send` VM，外部只传 owned、`Send` 的请求/响应消息；移除把 `&AutoVM` 洗成 `usize` 跨线程传递的入口。每 app/session 的隔离和可并发度显式配置。
- **任务调度**：连接任务可由 Tokio 驱动，但 Auto handler 仅在 VM 所有者执行；同步/CPU handler 有执行预算和隔离策略，外部 I/O 通过可 await 的 future/完成事件唤醒，不在唯一 reactor 线程执行阻塞请求或同步 `recv/join`。`task` 的 mailbox、`~T` future 和 `~Stream<T>` 生命周期与 Tokio 的取消/超时/背压做一一映射。
- **SSE**：producer 与网络 writer 间用有界通道；慢客户端使 producer 背压或按显式策略断开。断连、超时、server shutdown 时取消 generator/外部 I/O、释放资源。WebSocket 应采用完整协议库和独立能力声明；当前简化 echo 不代表通用 WebSocket 服务。
- **调用契约**：路径 > body > query 的现有绑定优先级与缺参 400、typed conversion、元数据注入、`Response` 状态/headers、错误 JSON 和 SSE 帧格式先冻结为 parity 表。生成 Rust handler 时转译失败应报错，不应悄悄改用 CRUD 模板/默认值。
- **服务配置**：开发缺省仅 loopback；公开监听需明确配置。提供可配置 body/header/连接/队列上限、读取/handler/写入超时、优雅关闭、结构化日志和 request ID。CORS 与鉴权按应用策略设置；TLS 可先由受支持的代理终止，但必须写清部署边界。

## 5. 静态审计发现与优先级

| 级别 | 证据 | 后果/验证方向 |
|---|---|---|
| P0 | `http_server.rs::handle_connection_async` 仅把首次读到的 `buf` 转成 `raw`，普通 JSON body 从 `raw.lines()` 拼接；只有 multipart 继续按 `Content-Length` 读取 | TCP 分段或 body 内换行可导致 JSON 截断/变形；用首包仅含 headers、body 分多段的原始 TCP 测试确认。 |
| P0 | SSE 循环把每帧提取放入 `std::thread::spawn`，紧接着在 `LocalSet` 上 `std::sync::mpsc::Receiver::recv()` + `join()` | 慢 generator 可阻塞唯一 reactor，使其他 HTTP 请求饥饿；用慢 SSE + 并发健康请求计时确认。 |
| P0 | `lib.rs`/`http_server.rs` 将 `!Send` VM 地址转成整数再在另一线程恢复引用 | 安全性依赖隐含线程/生命周期约定，未来多线程或关闭路径易破坏；迁入 VM owner 模型。 |
| P1 | VM `serve_async` 对连接无显式上限或读取 deadline；宣称优雅关闭，但 accept 是无限循环；`back_proxy.rs` 每连接开线程且队列无界 | 慢连接/高并发时资源不可控；做慢头、慢 body、并发和关闭测试。 |
| P1 | VM 简化 WebSocket 手写握手/帧解析，未构成完整协议实现；生成 Rust 服务默认 permissive CORS、`unwrap` bind/serve | 非示例部署时风险；按能力声明和部署配置收口。 |
| P1 | `api_gen.rs` 对端点体转译错误可 fallback 模板，字段提取中有 `unwrap_or_default`；`stdlib/auto/http.at` 与原生客户端/服务功能清单未同步 | VM/Rust/IPC/合并可能“成功但返回错值”；用跨形态金样和生成失败诊断守卫。 |
| P2 | 文档中 `http.rs.at` 与“HTTP 依赖 net/async”标为现状；`io` 目标实现、`a2r-std` 手写镜像与 Vue 适配关系未明确 | 扩展新后台时可能错误复用或漏实现；建立覆盖表与文档分层。 |

以上是源码审计推断，未做负载基准或生产部署认证。具体瓶颈与误差要由计划中的复现实验和对拍证据收敛。

## 6. 分阶段改良路线

| 阶段 | 主要交付 | 验收门 |
|---|---|---|
| A：契约与 P0 加固（PLAN-696，已交付） | 标准库后台覆盖表、HTTP/IPC/合并调用矩阵；分段 body 与慢 SSE 红转绿；去掉 VM 地址整数跨线程转运；修订现状 Specs | 原始 TCP/并发/生命周期回归与文档核对已完成；详见 PLAN-696 归档记录。 |
| B：VM HTTP 传输替换（PLAN-699，已交付） | Axum/Hyper 接入 VM owner 消息桥；旧手写解析退出默认路径；body/header、队列、读取/回复期限与关闭预算 | chunked/keep-alive/431/慢头/503/关闭复绑与示例回归已验证；CLI Ctrl+C 实机终端受环境限制在归档注记，不扩大交付声明。 |
| C1：普通 HTTP handler 异步等待与取消（PLAN-705，待实施） | 复用 PLAN-702 段执行，完成通知唤醒、请求作用域与生命期总上限、迟到结果不复活、有界非流式 async 客户端 | 上游 gate 未解除时 health 已返回；cancel/timeout/shutdown 后 task/result/许可回基线；无每请求或队满兜底线程。 |
| C2：CPU 纪律与外部流（待立项） | CPU 段预算/阻塞纪律依独立裁定；HTTPStream/外部 SSE 的可等待 Iter、流式上传下载；a2r 客户端阻塞边界治理 | CPU/等待/流分别有明示保证；外部慢流、背压/取消/错误对拍通过。 |
| D：多后台收敛与部署 | Rust 生成/VM/IPC/合并/back-proxy 共用 API 契约和失败诊断；覆盖 manifest；安全配置与性能报告 | 指定示例矩阵跨后台同语义；p95、内存、最大连接与拒绝策略有可重复记录，明确支持等级。 |

阶段 C1/C2/D 各自按独立验收立 Plan；CPU 纪律、外部流、a2r 治理可再拆分，不把整个 C 压入一份计划。PLAN-705 本轮为文档起草，不运行 Cargo 测试；实施门禁以最新 AGENTS.md 为准（PLAN-700 后 tf 已包含语料，tv 仅定向快捷档）。

### 6.1 阶段 C1 的设计收敛（2026-09-28）

699 的有界队列只限制待取请求，不能解决 handler 内等待上游；702 已为 UI 交付 `SegmentOutcome::Parked` 与原栈恢复，故下一步复用该机制改造 HTTP owner。新的活动请求上限覆盖 queued/running/parked 总生命期，挂起不能让队列边界失效。每个执行段串行，共享状态跨 await 可被其他请求观察；请求不是事务，取消不回滚此前副作用。

取消要与异步结果登记/完成一起设计：当前 `drop_async_result` 仅删条目，部分 worker 完成端仍直接 insert，静态可见“删除后迟到完成复活”的竞态；JSON 池满 fallback spawn 与 handle 每次 spawn 也不构成资源上限。PLAN-705 先以确定性顺序实验验证，再建立 live completion token、通知、单次消费/取消与有限 async executor。现有 UI 消费接口保留；不在本轮重做 actor mailbox 或强制 UI 改泵。

网络取消只接受可证明的任务销毁/错误、deadline 和 server shutdown；请求输入半关闭仍可能需要正常响应，不能把任意 EOF 当作取消。同步 CPU 段或阻塞 native 无强制抢占保证，Design 34 的 Q1/Q2 仍是独立决策，外部 SSE/HTTPStream 与 a2r 客户端另案推进。

## 7. 待决策

1. VM 服务的目标并发模型：单 owner + 有界串行 handler（先保正确性），还是 actor/session 分片。需要 A 阶段基准与共享状态语义决定。
2. 对外服务支持等级：仅开发本机、受反向代理保护的内网服务、还是直接公网监听。安全/性能验收阈值随等级设定。
3. `Server` Builder 与 `#[api]` 是否承诺完全同语义，以及 `Request` 对象注入、TLS、WebSocket 的首个正式支持版本。
