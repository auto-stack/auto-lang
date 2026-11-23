# ace-migration: m2-stdlib-network — 新版网络HTTP与流式服务库

> Status: **PLAN-974 整包 A 交付**(T-01..T-08,2026-11-16;lang `206fdf901` 谱系);
> **PLAN-1123 增量**(2026-11-23):Windows AV 的 HTTPS/代理=显式 `tls` provider 层
> (见下"HTTPS/代理:显式 tls provider(PLAN-1123)")。
> 本文件只记录**已验**支持、接口、拒绝与边界;其余具名 pending 见"边界"。Linux
> 实跑取证归 1063(r8 裁定);Windows x64 实证。

## platform.net/1(已验接口)

| 面 | 接口/事实 | 来源 |
|---|---|---|
| client 面 | `net_tcp_connect(host,port)`/`net_tcp_write_some(sock,Str)`/`net_tcp_read_into(sock,region,off,len)`/`net_tcp_write_region(sock,region,off,len)`(**raw字节面**——RFC6455 帧字节等非UTF-8载荷的必然通道)/`net_tcp_close(sock)`;caps `platform.net.connect/write/read/close` | `av-core/src/platform_io.rs` |
| server 面 | `net_tcp_listen(port)`(0=临时口)/`net_tcp_accept(listener)`(**铸造新socket台账条目**)/`net_tcp_local_port(listener)`;cap `platform.net.listen` | 同上 |
| 纪律 | 16-socket/监听器台账单属主账本(代次编码);call 64KiB/net-run 1MiB 预算;混源校验(read_into/write_region 沿 console_write16 律);具名错误码 -1..-10(**-10=可重试静默窗**);**paced有界等待**:accept≤50ms/read≤500ms(5ms 步进),静默窗返回 -10 由 guest 轮询——不烧墙/步数;Win32 std::net(5s 连/读/写超时);connect 拒绝=**具名-8不挂起** | 同上 |
| 准入 | MIR verifier 核四项 client cap 及 listen cap，缺授权为 `host.unauthorized` 零 dispatch。Windows [AAC 原生调度组合](../auto-ac/aac-service-boundary.md)支持下列 WinSock 接缝，其它 native 车道沿各自明确准入。 | `mir_verify` / `native` |

PLAN-1051 让同一调度 PE 执行已受检的 `platform.net/1`，使用 Windows WinSock，和内存、文件共用平台台账。真回环验证 `accept` 静默窗 `-10`、文本及 raw 字节、错 origin `-3` 和重复关闭 `-6`。监听器在 drain 关闭，未关闭流计入退出 scope 审计。

## std.net / std.url(已验 Auto 面,PLAN-1041 内联约定)

| 面 | 接口/事实 | 来源 |
|---|---|---|
| std.net | `net_send_request`(状态行+体界解析;观察面 `status*10000+body_len`)、`net_get/net_get_with_header/net_post/net_put/net_patch/net_delete/net_head`(**全方法族**)、`net_int_to_str/digit` | `stdlib/ace/std/net.at` |
| std.url | `url_encode/url_encode_path_segment/url_encode_query_component`(unreserved ALPHA/DIGIT/`-._~`+各面safe集)、`url_join_path`(恰一分隔符/双斜杠去重) | `stdlib/ace/std/url.at` |
| 包装缝(T-03 裁定) | host-console 按设计单源(语法事实:use-item 仅 scalar-modules/packages 面)——包装=**内联单源**(同文件拼写未来 packages 面);packages-lane platform host=具名后续(随 973 io 装配) | PLAN-974 §5 |

## 已验协议面(net974 9/9,全真回环+独立oracle)

TCP client 双向/SSE 流式解析(事件界)/HTTP client 全方法/redirect 单跳跟随/timeout 语义(固定窗+-10+拒绝-8)/HTTP server 最小(方法路由)/URL 族/**WebSocket 帧层**(Upgrade→101+canonical accept→RFC6455 掩码文本帧写入→无掩码回显字节精确)。

## HTTPS/代理:显式 tls provider(PLAN-1123,Windows AV 已验)

r2 合同的显式可选 provider 道由 AV/Rust 标准库的 `tls` 层承接
(`stdlib/ace/avrs/tls.{at,rs.at}`,沿 1043 选择与 1114 `avrs/1` 组合)。

| 面 | 已验事实 | 来源 |
|---|---|---|
| 来源(T-01 裁定) | **受检外库**:钉版 `rustls =0.23.46`(关闭默认特性,取 `ring`/`std`/`tls12`;离线 registry 无 aws-lc 闭包)+ `rustls-native-certs =0.8.4`,经 `// auto-ffi dep` 注解(新增内联键 `default-features=`/`features=`)。OS 接缝(SChannel)未选:私有测试信任需原始 SSPI 自建链引擎且仅 Windows,无 Linux 臂 | `stdlib/ace/avrs/tls.rs.at` |
| 选择 | 显式选层:整单元或 `str`+`tls` 选择;**plain HTTP 不要求该层**(`net` 层零依赖不变) | `compose_delivered_avrs` |
| 分工 | provider 只搬运字节(TCP 连接、CONNECT 隧道、受检握手、写请求、读到流尾、关闭);**HTTP 请求构造与响应解析留在调用方 Auto 源**,公共请求/响应合同不变(`status*10000+body_len`) | `tests/fixtures/tls1123/https_app.at` |
| 公开面 | `tls_exchange(spec)`→owned 交换句柄;`tls_result`/`tls_read_byte`(0..255,尾 -1)/`tls_close` | `stdlib/ace/avrs/tls.at` |
| spec | `key value` 头块+空行+请求字节:`host`、`port`、`trust file:<path>\|system`(https 必填,**无不安全来源**)、`via <host:port>`、`mode https\|http`、`timeout <ms>`(默认 1500,≤60000)。未知/重复键、`port 0`=具名 `tls.spec`;`host` 也接受未加方括号的 IPv6 字面量 | 同上 |
| 信任与主机名 | 证书链须落在**本次** trust 来源内,且证书对 `host` 有效;两者分开具名(`-9 tls.untrusted`/`-10 tls.hostname`),验证失败发生在写任何请求字节之前(对端零请求) | `avrs_tls_1123::v01_verification_failures_are_named_and_send_nothing` |
| 无跨请求借用 | 每次交换只读本请求 spec;套接字在调用返回前关闭,句柄只拥有响应字节;关闭/旧代次句柄在被调方执行前以 `foreign.handle` 拒 | `handle_lifecycle_closed_and_stale_are_rejected_before_the_callee` |
| 错误表 | 0 干净流尾;-1 spec、-2 resolve、-3 connect、-4 timeout、-5 proxy.rejected、-6 proxy.closed、-7 proxy.bad、-8 trust 来源缺失/不可读/为空、-9 untrusted、-10 hostname、-11 其它证书失败、-12 握手/握手中对端关闭、-13 write、-14 read(如对端复位,不保留字节)、-15 无 close_notify 关闭(字节保留)、-16 超限(请求 >64 KiB 时不发送;响应 >1 MiB 时保留前 1 MiB) | `stdlib/ace/avrs/tls.at` |
| 预算关系 | 一次交换=一次 foreign 调用:执行墙钟(`ExecLimits.wall`,默认 2 s)须大于该请求 `timeout`,否则执行器的墙钟结果先于具名 `net.timeout`;默认 1500 ms 因此低于默认墙钟。单次调用的字节预算(`MAX_CALL_BYTES`=65536,作用于整个 spec)先于 provider 的 64 KiB 请求上限,故经 Auto 到达的请求不会触发 -16 的请求腿 | `timeout_refused_and_truncated_close_are_named` |
| Windows 事实 | 被拒 SYN 由栈重试约 2 s:期限不足时拒绝端口表现为 `-4`,期限足够才是 `-3`。名字解析出的不可达首地址只消耗剩余期限的公平份额(在试地址均分) | `refused_connect_is_timeout_then_connect_by_the_deadline` |
| 预备闭包 | 依赖闭包由显式 `prepare-avrs-tls` 步骤构建一次(`ace-abi/provider-closure/1`:源/shim 摘要、rustc 身份、锁摘要、cdylib 摘要);日常道只校验并打开,缺失/陈旧/损坏=`prepared.missing/stale/corrupt`,**从不编译**。工程道以 `RustProjectInputs.prepared` 采用,源/shim/工具链/声明锁任一不符=`m2build.rust-prepared` | `ace_abi::rust_provider::{verify_provider_closure, prepare_provider_closure}` |
| 部署依赖 | provider DLL 的真实 PE 导入(12 个系统库:advapi32、api-ms-win-core-synch-l1-2-0、api-ms-win-crt-{heap,runtime,string}-l1-1-0、bcrypt、bcryptprimitives、crypt32、kernel32、ntdll、vcruntime140、ws2_32)由 SDK 打包记为 `system-lib` 运行依赖;`trust system` 读取当前用户的 ROOT 证书库,经 Crypt32——已是上述 12 个静态导入之一;单元不引用任何 SSPI 接口(TLS 由 rustls 完成),导入表之外无其它运行依赖。12 个库中 vcruntime140.dll 属 VC++ 运行库而非操作系统组件,目标机须已安装 | `tls_project_1123` |
| 分发 | 普通工程 build→run→SDK pack→重定位→删除开发缓存→自迁出 SDK 运行,同一退出码;对端记录两次(各直连+隧道)请求与两次 CONNECT | `tls_project_1123::p1123_tls_project_builds_runs_and_sdk_relocates_from_the_prepared_closure` |

## 边界(具名开放项,非静默)

- **TLS/proxy**:Windows AV 由显式 `tls` 层交付(上节);**Linux 实跑归 1063**(同一源码臂,Windows 环境不执行);代理认证、TLS 客户端证书、更广旧 HTTP API 按旧消费者证据另列,不从本节测试输出推定。
- **子串物化核心决定**:`url_decode`/query 族/HTTP header 值提取需 slice view→owned 促升或 char→str 内建(一个核心决定,未裁)。
- 973 阻塞腿:upload/download 文件体、io.* port 消费。
- 长流慢消费者持续背压、WS ping/pong/close/分片/大帧/并发、多跳 redirect、可配置 timeout API、Response 全存取组/表单/JSON 体。
- 方言事实(console 面):字面量左拼拒/字面量实参不 retag/借用参数不可直返 owned/`^` 仅 numeric 面/裸调用语句仅脚本尾。
