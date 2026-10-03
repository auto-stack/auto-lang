---
plan_id: PLAN-736
status: executing
feature_name: http-service-deployment-and-operations
author: [agent]
created_at: 2026-10-03
updated_at: 2026-10-03
plan_revision: 1
current_step: 1
total_steps: 8
supersedes_spec_components:
  - docs/specs/stdlib/project.md
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/http-handler-async-lifecycle.md
  - docs/specs/auto-man/project.md
  - docs/specs/auto-cli/project.md
new_spec_components:
  - docs/specs/stdlib/design/http-service-deployment.md
touched_goals: [GOAL-003]
affects: [crates/auto/src/main.rs, crates/auto-man/src/lib.rs, crates/auto-man/src/api_gen.rs, crates/auto-man/src/rust_ui.rs, crates/auto-man/src/util.rs, crates/auto-lang/src/lib.rs, crates/auto-lang/src/vm/ffi/http_transport.rs, crates/auto-lang/src/vm/ffi/http_server.rs, crates/auto-lang/src/http_file_service.rs, crates/auto-lang/src/http_upload_service.rs, deploy, examples/http_server]
---

# [PLAN-736] HTTP 服务部署基线：独立启动、配置与资源预算、观测和关闭验证

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) D2b。730服务端上传已归档；734已有代码完成交接，尚需独立复审/合入。本计划提前规划**VM默认API HTTP和auto-man生成Rust HTTP的可部署服务基线**：独立server入口、明确配置与支持等级、连接/请求预算、CORS/代理边界、健康/就绪与结构化日志、受控关闭和真实代理/负载验收。

首个交付等级为“本机开发 + 受单层反向代理保护的HTTP服务”，不宣称直接公网生产认证。需要观察真实wire与资源回收；不以配置文件存在或Axum默认行为代替验收。两服务轨分别适配Axum0.8/0.7，共享版本无关的配置/策略/观测数据。

本期不开发内置TLS、HTTP/2/3、完整WebSocket、认证账户/JWT/CSRF系统、通用流handler生成、VM CPU抢占、back-proxy传输迁移或全标准库manifest。IPC/merged/back-proxy/Builder等明确列能力矩阵，不把它们当独立部署服务。

## 1. 目标

- G1：增加server-only启动，不启动Vue/Vite、桌面UI或webview；生成Rust与VM都可使用相同服务配置。
- G2：默认loopback、严格配置诊断、显式非loopback监听；部署profile与开发profile可区别，前端连接URL与bind地址分别处理。
- G3：两服务轨都具有连接准入、普通请求生命期与body/header/期限预算；文件/上传独立预算保留，慢连接/慢流不能无限增长资源。
- G4：CORS、Host、单层代理信任及鉴权责任明确；服务profile中框架middleware执行错误拒绝请求，内部媒体/照片扫描路由默认关闭。
- G5：就绪身份、结构化日志与有界观测，startup/drain/completion含义可证；日志不泄露cookie/token/body或磁盘根。
- G6：两服务轨的shutdown真正停止accept、有限排空/强制收口并释放端口；独立CLI正确退出，生成失败/绑定失败不得报ready。
- G7：真实Nginx代理、TLS终止、JSON/SSE/文件/上传/早拒与负载资源曲线，形成可复现的支持报告。

仅在本仓实施；用户授权本轮规划，不授权部署现有服务器或改其他应用。协议/业务正确性仍依据734最终规范，遗留债不通过新计划宣布已清偿。

## 2. 架构方案

```text
CLI / explicit service JSON / legacy port+env → validated HttpServiceConfig
                      ↓ resolved immutable config + origin of each value
auto serve(project, vm|rust) → 734 generation/implementation gate → bind/init
  ├─ VM: HTTP/1.1 net loop + bounded owner/scope
  └─ generated Rust: HTTP/1.1 net loop + real compiled handlers
       ↑ version-neutral CORS/Host/proxy rules, state/counters/log event
       ↓ ready(instance+generation+bound address) / draining / stopped
single trusted proxy: TLS/auth/access policy → HTTP/1.1 upstream
```

拟新增：
- `crates/auto-lang/src/http_service_config.rs`：版本化配置、profile/范围/组合校验、纯策略与来源记录；不依赖Axum/VM。
- `crates/auto-lang/src/http_service_observability.rs`：服务状态、bounded事件/counter、完成记录及实例身份；不承载VM值或业务返回。
- `crates/auto-man/src/http_service.rs`：工程server-only构建/启动、生成新鲜度、子进程/ready与关闭。
- `crates/auto/src/cmd_serve.rs`：窄CLI适配；已有执行/生成入口复用。
- `crates/auto-lang/src/tests/plan736_http_service_tests.rs`、`scripts/http-service-probe.py`、`examples/http_server/deployment/`和`deploy/http-service/`：测试、示例及代理模板。

不重写HTTP parser；生成轨必要时从axum::serve换成其锁版本可用的hyper/hyper-util连接驱动，以显式限制连接/慢头和注入退出。版本相关adapter分别实现，禁止把0.8 Response直接送给0.7。scope/文件/上传资源归原执行模块，本期补服务终结传播和计数，不复制729/730协议逻辑。

## 3. 技术栈

- 现有clap、serde/serde_json、Tokio、Hyper/hyper-util、tower-http以及734的生成bundle。
- 服务配置JSON，不引入新pac.at解析器；配置在启动时解析一次，显式服务实例持有，不按每个请求读取env。
- 标准库已有file/upload执行器；健康和日志只使用owned消息/原子计数，不跨线程读取!Send VM。
- 单层Nginx反向代理与测试证书；模板使用已实测版本支持的指令，官方latest资料不代替锁版本核对。
- 轻量Python wire/混合负载探针；有界并发、port=0/OS临时端口、gate+deadline；无本轮负载运行。

## 4. 需求分析与背景调查

### 4.1 来源、基线与授权

起草master基线 `16351ca58`；736取号提交 `7b93bfcc5`，独占锁重扫活跃/归档max735后运行new-plan.sh。当前main包含730合入；734工作交接引用 `2850e7d24`（`D:/autostack/.wt/lang-734/auto-lang`）。起草期间其独立复审R1更新为needs_fix：F-1类型分类残留、F-2报告修正、F-3真实Tauri dispatcher缺失，须回工并复审R2；D4/D5/D7兼容债已由R1明确接受。本计划只读该worktree相关代码/报告，不修改其进度或执行复审。

| 来源 | 本期使用 |
|---|---|
| `docs/specs/overview.md`、`stdlib/project.md` | 本仓模块/标准库后台职责；不能把HTTP服务配置视为全标准库装配manifest。 |
| `docs/specs/stdlib/design/http-server.md` §7.3/§8.1 | VM默认入口HTTP/1.1、已有队列/header/body/期限及注入shutdown；目标API章节不等于全支持。 |
| `http-handler-async-lifecycle.md`、`http-stream-lifecycle.md` | queued/running/parked请求生命期、scope及流背压/取消；不重订CPU执行语义。 |
| `http-server-files.md`、`http-server-uploads.md` | 文件/上传独立额度、在途FS不强制中断、staging/commit仲裁与专属期限。 |
| `docs/specs/auto-man/project.md`、`auto-cli/project.md` | 启动、生成与CLI职责；无新的服务部署统一规范，需SD-01补齐。 |
| [PLAN-734](734-api-contract-and-generation-integrity.md) §9及其worktree verification报告 | 真实实现/新鲜度门可复用；AC-06 Tauri未闭合、iterator/Response注册表兼容等债需由734复审核实，不能称五形态全部通过。 |
| [Nginx官方proxy模块](https://nginx.org/en/docs/http/ngx_http_proxy_module.html) | 请求/响应buffering和读取间隔期限分别配置；SSE/上传早拒须实际代理验证。 |
| [tower-http CorsLayer官方文档](https://docs.rs/tower-http/latest/tower_http/cors/struct.CorsLayer.html) | origin/credentials/方法/header组合及Vary；实装按各adapter锁版本。 |

授权：用户“734已经完成；请规划下一个计划”，允许本仓只读调查及计划/Design33簿记；不实施、部署、运行测试或改变734合同。**736执行前置：734独立review通过并合入，最终Spec/债/receipt可读取**；若review需修复，先修734，新计划不承接未批准延期。730依赖已满足。

### 4.2 静态证据（实施T-01复现）

| 文件/符号 | 观察与缺口 |
|---|---|
| `auto-lang/src/lib.rs` 默认API启动 | 拼 `0.0.0.0:AUTO_HTTP_PORT`，与生成Rust loopback不同；不能称现状默认都仅本机。 |
| `vm/ffi/http_transport.rs::TransportConfig::from_env/serve_network` | header/body部分常量，部分env坏值回默认；每accept spawn，无连接总许可；请求scope有界不等于连接有界。 |
| `vm/ffi/http_server.rs::cors_origin/cors_headers/dispatch_api_request` | 默认*，静态拼CORS；request-id原样接受，文本日志；client IP当前取peer，未建立代理可信身份。 |
| `auto-man/api_gen.rs::generate_main_rs`（main与734worktree） | 两生成分支都loopback+allow_origin(Any)，bind/serve unwrap、ready消息早于bind；未见统一资源/关停装配。 |
| `auto/src/main.rs` Run端口、`auto-man/util.rs::http_port/http_base_url` | CLI -B > pac.at > env/default；同时注入AUTO_HTTP_BASE、前端/child消费，必须防新的配置与旧环境互相覆盖。 |
| `auto-man/rust_ui.rs::start_api_server/start_vm_server` | 轮询TCP connect判ready，可能连接其他实例；kill/线程生命期须追踪至真实server。 |
| `deploy/nginx-auto-playground.conf` | playground专项、含固定站点及WS配置，不能直接当Auto api.at服务模板。 |
| `KNOWN-DEBT-AND-RISKS.md` P729-D1 | 文件body在shutdown drain窗内缺专项wire；736补自然完成/超时强制与资源最终回收测试。 |

不把静态推断当已复现漏洞；T-01读734最终源码再固定消费者/差距。认证失败传播、资源回收和代理行为需实际wire证据。

## 5. 详细设计

### 5.1 服务入口与配置

拟提供 `auto serve <project> --server vm|rust --http-config <json>`；不启前端/桌面。显式server选择沿用现有拼接/解析/实际生成链，Rust standalone产物本身可直接启动。UI split现有run保留，只窄接同一配置/ready；merged/IPC模式收到服务专属配置不得假装生效。

profile：
- `development`：缺省127.0.0.1:8080；开发CORS legacy *（credentials=false），保留既有端口来源与已公开env；这是对VM旧0.0.0.0默认的显式安全行为调整，文档给迁移方式。
- `proxy_service`：缺省loopback，显式allowed_hosts、auth responsibility(app|edge)、日志/预算；CORS缺省无跨域授权，允许显式精确origin列表。edge模式只接受明确trusted peer，样例backend loopback且代理鉴权，不能仅填写字段便认定真实鉴权有效。
- 非loopback必须显式地址配置；监听权限/端口冲突/非法设置非零退出，不悄悄换端口或回默认。port=0只用于独立serve/test，ready发实际bound address；UI split若未闭合动态地址发现须明确拒绝0，不生成:0客户端。

JSON schema_version、profile、listen、ordinary limits、cors、allowed_hosts、trusted_proxy_ips、auth责任、observability及shutdown；未知字段/坏值/非法组合指名失败。硬性有限范围、有单位、配置来源与生效摘要。729/730 file/upload额度仍由其规范管理，汇总有效值；profile不得不经校验扩大这些额度。

既有无文件run维持CLI -B > pac.at back_port > AUTO_HTTP_PORT >8080。显式http-config定义完整服务listen：CLI显式-B可覆盖其port，未显式的pac.at端口不再覆盖它；与显式legacy env/listen冲突时诊断，不偷偷优先。T-01核对build/run/直接生成binary并冻结逐字段表及范围；不能静默改变旧无配置用户端口。effective_config hash与734 implementation hash分开，运行值不假称新业务实现；启动校验仍消费734当前ready generation。

bind address、本机control/probe address与advertised client URL分离；0.0.0.0不能直接成为浏览器连接地址。不把服务配置文件内容重复拼进多个generator，不在请求路径读取进程级可变env。

### 5.2 资源预算与服务生命周期

普通默认继续64KiB头缓冲/100头、慢头10s、JSON10MiB/body10s、请求生命期64/30s、drain10s；连接上限拟128。配置项可有限下调/明确上调，T-01冻结合法上下界与两adapter能力，不使用0=无限。生成Rust缺失的普通budget须实装，不把VM有界规则仅写进生成注释。

连接permit覆盖idle/正在读头/keep-alive/发送及upgrade；连接满在开始解析前关闭，计conn_rejected，不承诺此时能返回HTTP503。解析后的普通request满503+Retry-After；许可覆盖queue+running+parked及流body终态，不能headers一出就释放。有界普通body读/期限和head约束在进入业务前生效；413/408等状态按既有约定，header原生431/超时close保持框架差异明确。新资源上限覆盖路由错误/预检/健康与内部路由，不能由隐藏路径绕过。

文件/上传/SSE沿原专属期限和预算；UploadRequest早拒/100-continue前不拉body，上传不用30s普通handler期限截断10min接收。所有body在断连/取消/服务关闭时传播终结；生成Rust流/body Drop要释放普通/连接许可及宿主资源，不能只让VM scope回收。现有WS简化echo在service profile默认关闭，不新增协议支持。

shutdown：Starting→Ready→Draining→Stopped，停accept、ready=503、停止接收新业务、排空在途，超过drain终结连接/task并取消原scope/stream/upload。在途FS操作等实际退出后才计许可归还，不伪称abort立即退出；受控测试FS需在界内结束，真实不可中断磁盘长阻塞边界写支持报告。有限关闭保证适用于本期可取消IO参考负载；VM无限CPU段/不可中断native不在该保证内，不能安全强杀线程，也不能把parent kill后的崩溃退出记成优雅关闭。

### 5.3 CORS、Host与代理/鉴权边界

CORS共享纯策略：合法preflight校验origin/method/request-headers；精确origin匹配、credentials禁止*、恰当Vary，allowed_methods/headers/exposed_headers列出，开发与service profile分开。不允许Origin不获得allow头，不能仅以CORS拒绝代替鉴权。正常、框架错误、SSE、file/upload路径统一套策略；Hyper在service前产生的431/断连仅记录connection事件，不虚构其CORS/request-id。

service精确Host允许表，未允许400零业务/FS；不新增任意regex规则。应用自己的授权middleware保持业务语义，但service profile里middleware执行/编组错误fail-closed 500；普通旧开发语义如需保留须显式矩阵。edge与app的鉴权责任不自动给所有API加登录；fixture有保护路由证明缺凭据/错凭据拒绝，body/FS零工作。

首期单层可信代理：trusted_proxy_ips为numeric IP显式列表，默认空；仅socket peer命中才消费代理重写的单个X-Forwarded-For IP与http|https的X-Forwarded-Proto。拒绝多段/坏值歧义；不信任客户端X-Real-IP/Forwarded/任意链，不做DNS解析。模板覆盖而非追加用户XFF，外来不可信头忽略；effective client identity用于明确日志/既有按IP限速，socket peer独立保留。service启用按IP限速时，bucket表按实例加有限容量/过期回收（拟4096条，T-01冻结窗口TTL），满表新身份保守429而不绕过限速；不扩展成分布式限流。代理若担edge鉴权须保证backend无绕过路径，真实测验证；service host检查不根据随意X-Forwarded-Host放宽。

生成main自动挂载的媒体/照片扫描及调试路由在service profile默认off，opt-in须显式能力与保护策略；普通api.at的file/upload路由保留应用root/auth。配置不会构成完整文件系统沙箱、CSRF防护或CPU故障隔离。

### 5.4 ready、观测与进程退出

保留前端启动有界ready门，改验证服务身份，不只TCP连接成功：bind+路由/VM初始化+734generation验证完成才Ready；包括instance_id、generation fingerprint、实际地址/配置hash。已有其他进程占端口或旧generation不能被当本实例ready。

保留路径 `/__auto/health/live` 与 `/__auto/health/ready`（与业务同名route在装配时诊断）。live表示net服务尚能应答；ready仅表示启动成功/未drain/接受业务，不证明数据库或CPU健康。健康不经过业务VM/auth、不能读取VM/调用外部服务；受连接/独立小请求预算，拥塞时可失败不虚称永远可用。返回最小状态/身份，不暴露配置根/路由/资源详情；edge代理默认不对外开放操作面。

日志JSONL至少startup/config_error、request完成、drain/stop、resource/connection拒绝事件；服务身份、合法有界request-id、route模板、method/status、header_latency_ms/body_duration_ms、bytes_sent、outcome（completed/canceled/error）等。响应headers成功不等于body成功，流/文件结束才记终态；重复cancel/drop不双计。请求ID只接受1..64可见ASCII安全子集，否则重新生成；字段以serde编码，禁止原样拼接query/用户路径/错误体。

不记录Authorization/Cookie/原始query/body、token、文件绝对根或上传filename；高基数字段不成为counter维度。日志sink有界、不阻塞reactor/owner；队满可丢观测并计log_dropped，业务不等待磁盘日志。暴露测试/本地内部snapshot：connections/requests/parked/stream/file/upload/FS许可/日志队列等；不是新增公网metrics管理API。

auto serve收到Ctrl+C/SIGTERM进入相同shutdown入口（平台支持矩阵准确）；Rust child有界排空/退出后parent退出，VM网络线程join；端口与自己创建子进程零残留。CLI错误有退出码，正常关闭0，bind/config/generation错误非零；现有UI关闭管理复用该句柄。Windows headless不能模拟真实console时须记注入信号范围，不能冒称实机Ctrl+C证据。

### 5.5 部署样例与支持报告

新 `deploy/http-service/`，不改正在使用的playground部署文件。loopback backend+Nginx TLS终止、明确Host/CORS/auth责任、access规则、HTTP/1.1 upstream；普通JSON、SSE与upload路径分组，SSE关闭响应buffering，upload关闭request buffering并对齐wire/idle/total边界。关闭非幂等自动重试以免重复上传/业务副作用。读取间隔timeout不等于上传总期限；相关规则依据[官方proxy文档](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_read_timeout)。

fixture生成临时证书/凭据，只测试localhost；实际HTTPS→proxy→VM/Rust，证书被测试client按测试CA验证，不靠--insecure当TLS验证。SSE首帧/分帧、100-continue授权早拒、上传下载hash与Range经真实代理观察。Nginx -t通过只是一环，不能以手写转发器或样例文本替代代理运行。实施T-01明确可用Nginx/OpenSSL版本和运行环境；缺执行条件明确blocked，不能跳过AC-07标绿。

支持报告列操作系统、架构、toolchain/锁版本、CLI/生成产物hash、profile/业务fixture/机器负载、raw数据、资源预算和限制。首期认证范围仅测试环境受单代理保护的IO型API；VM CPU/阻塞native隔离、任意外部流生成、直接公网和跨OS实机未测面逐项明示。TLS/鉴权终止在代理，Auto不是自带HTTPS实现。

### 规范增量

本节是提案，起草不修改canonical；734合入后再对最终规范做增量。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/http-service-deployment.md` | 分散默认/无支持等级 → profiles/配置/代理边界/预算/观测/关闭与验证等级 | 核心部署合同 | AC-01..09 |
| SD-02 | modify | `docs/specs/stdlib/design/http-server.md` | VM宽监听/生成permissive差异、有限VMshutdown → loopback迁移、两轨装配/能力/关闭与canonical链接 | 避免目标当现状 | AC-01/03/04/06/09 |
| SD-03 | modify | `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | VM请求scope → service请求/响应终态许可、middleware错误profile与shutdown边界 | 不破坏705/730期限 | AC-03/04/06 |
| SD-04 | modify | `docs/specs/auto-man/project.md` | 启动轮询端口/生成main单独装配 → config/ready身份/进程托管与版本adapter | 真生成服务消费 | AC-01/02/05/06 |
| SD-05 | modify | `docs/specs/auto-cli/project.md` | run面含UI/端口分散 → auto serve、config来源/动态端口/退出码 | 独立服务入口 | AC-01/02/06 |
| SD-06 | modify | `docs/specs/stdlib/project.md` | 服务支持边界分散 → 部署合同/两入口及限制索引 | 统一文档入口 | AC-09 |

## 6. 测试设计

### 6.1 真实两轨与负向矩阵

| 族 | 必须观察/期望 |
|---|---|
| 启动/配置 | auto serve两server实际启动，无UI/Vite子进程；生成binary直跑；配置来源/无配置legacy端口/错误与重复字段/非法范围、listen冲突、占端口旧实例、port0真实地址；非零及无假ready。 |
| CORS/代理/保护 | 合法/非法origin、credentials+*、preflight方法/header、Vary、坏Host、伪XFF/多段/可信peer/IPv4/6；启用限速时多身份bucket容量/TTL/满表429；保护上传无/坏凭据及middleware故障body/FS计数0；内部scan route默认404。 |
| 预算/期限 | 超连接/idle/慢头/chunked超JSON/慢body/请求满；conn close及request503分开；固定低配置便于测试，合法普通请求正常，body许可终态回收。 |
| 流/文件/上传 | 729/730与既有两端publisher SSE支持面；slow send/download、receive/lease/commit gate，期间短JSON可完成；断连/早拒不遗留scope/temp/job/许可。 |
| 观测 | startup先于ready但ready晚于bind/init；body终态日志恰一次、cancel归类、字节真实；恶意reqid/query/token/body/filename不泄漏/不换行注入；sink满有drop计数且业务不堵。 |
| drain | 注入shutdown两轨：短请求自然完成、长SSE/慢文件/上传强制结束；drain期间ready503与新业务拒绝、停止accept、port重绑；parent/child退出；commit已胜出文件保留。 |
| 代理 | nginx -t + HTTPS实际运行两轨；SSE首帧可见、非缓冲早拒100-continue、upload→download hash、Range；trusted身份不可伪造，诊断精确。 |

新HTTP族命名 `http_e2e_plan736`，仅test-http-e2e，串行；生成服务与CLI整链测试绑定实际exe/deps/source hash，不手写server替代。734受支持普通API、729/730保留回归；Tauri/merged/back-proxy不纳入服务负载认证，配置误用于它们应明确不可用。

### 6.2 受控负载门（本合同阈值，T-01固定环境不能事后降低）

两轨分别构建release fixture，排除冷编译/建库时间；同机单实例顺序跑，至少3轮：
- 普通JSON：warmup后16并发、10,000请求，每轮零意外非2xx/错误值，p95≤1s，记录p50/p95/p99/RPS而不预造性能数字。
- 混合IO：2慢SSE、2慢下载、2上传，加8并发短JSON，持续60s；健康/短JSONp95≤1s，文件hash一致，无超预算或无界队列。共同流fixture按734最终支持能力选择，不能把未支持流生成当成功。
- 超载/恢复：连接cap=32、请求cap=8，压到128连接+64并发请求；实际conn≤32、活动request≤8（独立health额度计入各自表），拒绝规则成立。释放压力后受控worker在5s内回基线，短请求恢复；超限不能导致panic/OOM。
- 资源：同一10min进程做10个混合/取消周期，warmup后第2..10周期闲态RSS最大差≤64MiB、周期峰值RSS≤512MiB；连接/permit/job/temp/打开文件按测试可观察面回基线。测量包含业务server进程，proxy与生成编译进程另记不混入。
- 高负载日志sink堵塞另测，不跑无限产生日志；全部探针自带总deadline、进程退出/端口释放清理。

这些是小型参考服务的本机门，不证明任意应用吞吐或公网安全。无法达标应定位/修复或needs_replan，并明确合同修订；不能仅报机器忙而勾选，也不能为过门缩小固定并发/阈值。每轮保留机器/负载/采样方法/原始JSONL，计数回基线不能仅靠RSS猜。

### 6.3 分级实施门禁（本轮不跑）

在 `D:/autostack/.wt/lang-736/auto-lang`：
- 开发 `cargo check -p auto-lang`、`cargo check -p auto-man`、`cargo check -p auto`；`cargo t plan736`；`cargo test -p auto-man http_service -- --test-threads=1`与`cargo test -p auto --bin auto serve -- --test-threads=1`（新增对应族）。
- 真TCP `cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan736`，auto-man api_gen scoped及真实generated/CLI/proxy运行。
- 复审裸 `cargo t` + `cargo th --test-threads=1`；VM执行/编译接线触面跑`cargo tv`，trans实际改动才`cargo tt`，ui_gen改动才`cargo tu`；另auto-man/CLI全crate适用门禁及必要a2r-std scoped。release只构建本期fixture/CLI，不跑全仓release测试池。
- 不触aavm/schema/book，无taa/docs_gen；tf仅merge到期主检出单实例/auto-plan:regress。负载探针独占顺序执行，不和tf/其他性能任务并跑，不在日常测试池嵌入10min负载。

报告新 `docs/plans/reports/736-http-{decision,config,policy,lifecycle,proxy,load,verification}.md`，绑定最终revision与每AC/SD的真入口/命令/原始证据。

## 7. 验收标准

| ID | 可观察交付 | 验证/期望 |
|---|---|---|
| AC-01 | 两轨独立服务/生成binary与配置真实生效 | CLI实际启动、无前端/桌面；默认loopback、明确监听、端口/URL/legacy优先级，坏配置/绑定错误非零；scoped+wire。 |
| AC-02 | 734实现新鲜度和服务ready身份贯通 | 旧实例/旧bundle/生成失败/路由或VM初始化失败不能ready；port0返回真地址；JSON ready实例/指纹与启动者一致。 |
| AC-03 | 两轨连接/request/head/body/期限预算实际有界 | §6.1低预算wire及§6.2超载；超连接close/request503分明，许可持续至终态；file/upload期限不误用普通budget。 |
| AC-04 | CORS/Host/单代理信任/鉴权责任正确 | 两轨合法/非法wire、Vary/credentials、伪头/可信peer、middleware失败与上传早拒0FS、service内部route默认off。 |
| AC-05 | 有界、可关联、无敏感内容的观测 | 日志schema/恶意值脱敏、真实body完成/取消与bytes、恰一次计数；sink阻塞/drop不会卡业务，health边界与资源snapshot正确。 |
| AC-06 | 两轨关闭/parent child管理与资源收口 | 自然排空/强制终结/port重绑、注入信号及可用平台真实进程信号；文件/上传commit胜负与FS最终退出有证据，P729-D1专项补齐。 |
| AC-07 | 实际HTTPS单代理兼容JSON/SSE/file/upload | nginx版本与-t、测试CA验证/真实代理wire、首帧/早拒/hash/Range；不能用配置文本/自写proxy替代。 |
| AC-08 | 固定参考负载/资源恢复门达标 | §6.2两轨3轮raw统计及10min资源曲线，全部固定阈值成立；无scope/temp/permit泄漏或无限连接/日志增长。 |
| AC-09 | canonical增量/支持等级及旧能力兼容可沉淀 | SD-01..06+代表734/729/730/JSON/SSE回归，零新增确定性红；明确没验证的OS/CPU/IPC/公网边界，不冒称734债已清偿。 |

## 8. 执行步骤

### T-01：最终依赖、配置与服务运行原型

- [x] 前置734独立review pass+merge；读最终SD/receipt/批准边界，不接受仅work pass。发现未批准AC延期先交734review解决，不在736更改其合同。 [✅ 已完成] R2 pass+归档752cb37f4/cleaned 5ccb3d6c5；接受债 D4/D5/D7 登记（reports/736-http-decision.md §1）
- [x] `bash scripts/new-wt-group.sh lang-736 --branch plan-736-dev`，auto-down兄弟只读，禁止链接。 [✅ 已完成] 组建@de3a64353（auto-lang plan-736-dev + auto-down detached@895f8d0）
- [x] trace lib默认API、CLI Run/build、rust_ui VM/Rust、api_gen两main分支与734bundle消费者；复现bind默认、ready假成功、无连接cap、生成关闭差异。 [✅ 已完成] E1..E12 逐点复现（decision §2）：lib.rs:1737 0.0.0.0、http_transport.rs:149 无conn cap、api_gen.rs:3781/3842 假ready+unwrap、rust_ui.rs:3893/3990 TCP假ready、util.rs:294 URL恒loopback
- [x] 有界调查产出`736-http-decision.md`：CLI/config字段/来源/范围、0.7/0.8 adapter依赖、body终态许可与shutdown/文件在途FS、health/日志机制、支持矩阵；实际Nginx/OpenSSL可用版本和两轨最小独立serve原型。 [✅ 已完成] reports/736-http-decision.md（提交 a043bb4fc）；nginx 1.31.6（scoop本会话安装）+ OpenSSL 3.2.3 + Win11/20核/32GB 冻结
- [x] 冻结测试机器及§6.2条件/合法流fixture；若依赖缺失或方案无法满足固定AC则blocked/needs_replan，不降低阈值。涉及全部AC/SD。 [✅ 已完成] 无blocked：依赖在位、原型可行（decision §3.7-3.9）

### T-02：公共配置和CLI server-only入口

- 依赖T-01。新http_service_config.rs、auto-man/http_service.rs、auto/cmd_serve.rs，改lib.rs导出、auto/main.rs命令、auto-man/lib.rs及util配置消费。
- profile/严格校验/来源、default loopback、显式监听/port0、旧无文件兼容；复用真实生成/VM装配，不启动UI。
- scoped config/serve单测及实际CLI两轨启动、坏配置/端口冲突非零；AC-01/02，SD-01/02/04/05。

### T-03：两代服务adapter与预算/完成许可

- 依赖T-02。改vm/ffi/http_transport.rs、http_server.rs、auto-lang/lib.rs默认入口、auto-man/api_gen.rs::generate_main_rs；必要生成连接adapter为新模板/模块。共享配置不共享Axum对象。
- 连接cap/header/body/ordinary request许可、timeout/drop/stream完成接线；保留729/730 host预算与typed早拒。
- scoped+两轨低预算wire，JSON/流/文件/上传结束和断连计数；AC-01/03/06，SD-01/02/03/04。

### T-04：CORS/Host/代理策略与middleware失败边界

- 依赖T-03。纯策略归config模块，VM dispatch及network/gen adapter消费；生成main内部媒体/照片route安装按profile门控。
- 精确origin/preflight/Vary与坏Host；单可信peer重写身份、字段校验、既有限速身份与有界bucket/TTL；service middleware错误拒绝，保护上传不预读/写盘。
- 实际两轨policy与0FS负测；AC-04/07，SD-01/02/03/04。

### T-05：观测、ready与关闭/进程管理

- 依赖T-03/04。新http_service_observability.rs；改HTTP response/body收口、file/upload宿主必要终态hook、auto-man/http_service.rs、rust_ui.rs及实际vue等启动消费者（T-01定位后窄改）。
- 状态/health/identity ready、JSONL bounded sink、真实body完成/bytes/计数、734generationgate；替代纯TCP探测并管理真实VM线程/Rust child。
- deadline drain/force结束、signal→shutdown同入口、自然完成/长流/文件/receive/staged/commit gate回收；补P729-D1。真实CLI退出/重绑与日志sink负测；AC-02/05/06，SD-01..05。

### T-06：同源部署示例与真正代理测试

- 依赖T-04/05。新examples/http_server/deployment/{README.md,pac.at,src/back/api.at,service.json}、deploy/http-service/{README.md,nginx.conf.example}及fixture runner，test临时CA/凭据不入库。
- 示例有短JSON、受保护上传与下载、两轨已有支持的SSE；不手写替代编译业务。现有playground文件不改；明确edge/app责任和模板版本/期限。
- nginx -t +真实HTTPS VM/Rust早拒/帧/hash/Range/身份；scoped测试注册test-http-e2e；AC-04/07/09，SD-01/02/06。

### T-07：受控负载、资源曲线与支持分级

- 依赖T-03..06。新scripts/http-service-probe.py和plan736_http_service_tests.rs，注册tests.rs；可用测量依赖T-01冻结，所有参数/输出显式，禁止无限并发。
- 完成§6.1及§6.2全部两轨3轮/资源10min，generated和CLI hash/机器/原始数据到报告；CPU/native不具强抢占保证，Tauri/merged/back-proxy不当server认证。
- 资源回收counter/FS/temp与RSS分开，负载单实例，不把内部health快速返回当VM业务可响应；AC-03/05/06/08/09，SD-01/02/03/06。

### T-08：门禁、独立复审和规范交接

- 依赖T-01..07；§6.3门禁、fmt/warnings/debug及遗漏/延期扫描，债有归属；零新增确定性红，报告绑定最终revision和全部AC/SD。
- /auto-plan:review独立按真实CLI/generated/proxy/资源证据验证，不信勾选；准备SD沉淀稿，merge才更新canonical/ledger/Design33并归档。
- 清理前lang-736两兄弟worktree各跑wt-guard clean；tf只merge到期main单实例，不在多个worktree并行。
- 全AC/SD闭合才交reviewed/archived；不能以跳过proxy/load或声明“仅开发”缩小本计划的service等级。

## 9. 复审记录

### 起草交接（2026-10-03）

- stage: new
- plan_id: PLAN-736
- plan_revision: 1
- outcome: blocked
- blocker: 734复审R1为needs_fix（F-1/F-2/F-3），尚无pass/合入后规范；本计划共享API生成/启动/HTTP核心，顺序实施。
- next: 734修复R1三项并复审pass+merged → T-01最终基线/adapter与工具/机器条件冻结 → /auto-plan:work PLAN-736。
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-09（新）
- blocked仅为实施依赖，无需用户为本次规划补信息；实施使用前需具体合同确认，不从“规划”推定自动部署授权。
- 本轮仅计划/Design簿记，不创建736 worktree/不跑Cargo/不改canonical/不修改734进度。交接检查通过：11章节、8任务/9AC/6SD覆盖、现有Spec路径与新增标识、无新增失效链接、唯一ID及diff；Design索引既有513链接失效未纳入本期。

### 工作交接（2026-10-03 09:10，/auto-plan:work 入口核查）

- stage: work
- plan_id: PLAN-736
- plan_revision: 1
- outcome: blocked
- code_commit: 无（未创建 736 worktree/分支，未进入实施）
- task_ids: 无（T-01 入口前置未满足；T-01..T-08 全部链式依赖 T-01，无不受影响任务可先行）
- evidence: T-01 前置"734独立review pass+merge"实测未满足——master `docs/plans/734-*.md` 仍 `status: executing`（最新记录=复审 R1 needs_fix，master c16187f44）；`plan-734-dev`（tip a5284dd84，2026-10-03 08:47 +0800）已含 R1 回工提交但未合入 master；734 worktree 另有未提交 R2 阶段修复（`stdlib.rs` vm_value_to_json I64 臂回补，mtime 08:59:48，代码注释自标 "PLAN-734 R2"）。本会话 09:10 实测距该写入仅 11 分钟——**并发会话正活跃推进 734 修复**；按单写者纪律与起草授权（"本计划不修改其进度或执行复审"），736 不夺取 734 所有权、不代跑其 R2 复审/合并。
- blockers: 唯一阻塞=PLAN-734 生命周期未闭合（R2 复审 pass + merge）；无用户决策项。
- next: 734 会话完成 R2 复审 pass 并 merge → 主检出可读最终 SD/债/receipt/批准边界 → 重跑 `/auto-plan:work PLAN-736`（自 T-01 起：new-wt-group lang-736、736-http-decision.md、冻结测试机器与 §6.2 负载条件）。734 复审若再 needs_fix，仍先修 734，736 不承接未批准延期。

## 10. 待澄清事项

- 734仍在review队列，Tauri fixture与资源声明兼容债以其最终review裁定为准；不挪到736当作获准延期，不宣称五形态parity已全部完成。
- 本合同选择单可信代理保护的参考服务；若用户以后要求直接公网/内置TLS、多层代理、跨OS生产认证或CPU/native隔离，应另立计划，不在本期添加模糊安全保证。
- T-01负责0.7依赖、单层代理运行环境、信号平台和固定测试机器确认；无法运行真实proxy则明确blocked，不只提交样例。阈值调整需要具体合同修订，不能测试后改期望。
- 不能中断的FS/native/CPU、无actor/session分片、729路径中间段TOCTOU、730无崩溃恢复/全局磁盘配额仍按既有Spec边界披露。
- 后续优先候选：D3全标准库目标装配manifest；734的未闭合调用/typed资源债按独立review结论另立API演进计划；WebSocket/直接公网与CPU隔离按应用需求分开。
