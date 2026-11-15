# ace-migration: m2-stdlib-network — 新版网络HTTP与流式服务库

> Status: **PLAN-974 整包 A 交付**(T-01..T-08,2026-11-16;lang `206fdf901` 谱系)。
> 本文件只记录**已验**支持、接口、拒绝与边界;TLS/proxy 为 r2 合同的**显式可选
> provider 道**(未实现、未冒充),其余具名 pending 见"边界"。Linux 实跑取证归
> 1063(r8 裁定);Windows x64 实证。

## platform.net/1(已验接口)

| 面 | 接口/事实 | 来源 |
|---|---|---|
| client 面 | `net_tcp_connect(host,port)`/`net_tcp_write_some(sock,Str)`/`net_tcp_read_into(sock,region,off,len)`/`net_tcp_write_region(sock,region,off,len)`(**raw字节面**——RFC6455 帧字节等非UTF-8载荷的必然通道)/`net_tcp_close(sock)`;caps `platform.net.connect/write/read/close` | `av-core/src/platform_io.rs` |
| server 面 | `net_tcp_listen(port)`(0=临时口)/`net_tcp_accept(listener)`(**铸造新socket台账条目**)/`net_tcp_local_port(listener)`;cap `platform.net.listen` | 同上 |
| 纪律 | 16-socket/监听器台账单属主账本(代次编码);call 64KiB/net-run 1MiB 预算;混源校验(read_into/write_region 沿 console_write16 律);具名错误码 -1..-10(**-10=可重试静默窗**);**paced有界等待**:accept≤50ms/read≤500ms(5ms 步进),静默窗返回 -10 由 guest 轮询——不烧墙/步数;Win32 std::net(5s 连/读/写超时);connect 拒绝=**具名-8不挂起** | 同上 |
| 准入 | mir_verify 四+一 net caps(host-console platform profile 同 io/mem 纪律);缺授权=**准入拒绝 host.unauthorized 零 dispatch**;native 道全部 net ops **具名拒**(穷尽匹配防静默穿透) | `mir_verify.rs`/`native/*` |

## std.net / std.url(已验 Auto 面,PLAN-1041 内联约定)

| 面 | 接口/事实 | 来源 |
|---|---|---|
| std.net | `net_send_request`(状态行+体界解析;观察面 `status*10000+body_len`)、`net_get/net_get_with_header/net_post/net_put/net_patch/net_delete/net_head`(**全方法族**)、`net_int_to_str/digit` | `stdlib/ace/std/net.at` |
| std.url | `url_encode/url_encode_path_segment/url_encode_query_component`(unreserved ALPHA/DIGIT/`-._~`+各面safe集)、`url_join_path`(恰一分隔符/双斜杠去重) | `stdlib/ace/std/url.at` |
| 包装缝(T-03 裁定) | host-console 按设计单源(语法事实:use-item 仅 scalar-modules/packages 面)——包装=**内联单源**(同文件拼写未来 packages 面);packages-lane platform host=具名后续(随 973 io 装配) | PLAN-974 §5 |

## 已验协议面(net974 9/9,全真回环+独立oracle)

TCP client 双向/SSE 流式解析(事件界)/HTTP client 全方法/redirect 单跳跟随/timeout 语义(固定窗+-10+拒绝-8)/HTTP server 最小(方法路由)/URL 族/**WebSocket 帧层**(Upgrade→101+canonical accept→RFC6455 掩码文本帧写入→无掩码回显字节精确)。

## 边界(具名开放项,非静默)

- **TLS/proxy**:r2 合同=显式可选 provider 道(967 FFI/OS provider)——未实现未冒充;provider 裁定归后继。
- **子串物化核心决定**:`url_decode`/query 族/HTTP header 值提取需 slice view→owned 促升或 char→str 内建(一个核心决定,未裁)。
- 973 阻塞腿:upload/download 文件体、io.* port 消费。
- 长流慢消费者持续背压、WS ping/pong/close/分片/大帧/并发、多跳 redirect、可配置 timeout API、Response 全存取组/表单/JSON 体。
- 方言事实(console 面):字面量左拼拒/字面量实参不 retag/借用参数不可直返 owned/`^` 仅 numeric 面/裸调用语句仅脚本尾。
