---
plan_id: PLAN-974
status: executing
feature_name: 新版网络HTTP与流式服务库
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 9
current_step: 5
total_steps: 8
depends_on: [PLAN-968, PLAN-973, PLAN-1042]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-stdlib-network.md]
touched_goals: [GOAL-002]
---

# [PLAN-974] 新版网络HTTP与流式服务库

## 0. 变更摘要

2026-11-15 r8当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r7当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r6当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-12 r5：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-10 r4当前执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

实现既有net/HTTP client/server、SSE、upload/download、File/Stream return、async handler；Auto网络/HTTP协议及基础transport封装归本包，Web产品宿主/部署接线留983，涵盖取消/背压/断连/错误与权限。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `stdlib/auto`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/http`（本轮确认存在；开工重核符号/版本）。
- `crates/a2r-std/src/sse.rs`（本轮确认存在；开工重核符号/版本）。
- `docs/specs/stdlib/design/http-stream-lifecycle.md`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：TCP/HTTP客户端与服务/流生命周期。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/a2r-std/src/http](../../crates/a2r-std/src/http)：旧HTTP语义/同步异步adapter参考；目录职责已核，确切符号/闭包开工再核

- [crates/a2r-std/src/sse.rs](../../crates/a2r-std/src/sse.rs)：流/EOF/close；读取SHA256前16 `c9cb468006e176a8`

- [docs/specs/stdlib/design/http-stream-lifecycle.md](../specs/stdlib/design/http-stream-lifecycle.md)：已有流生命周期研究/合同；读取SHA256前16 `cdc8e0459a554783`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08 |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


实现既有net/HTTP client/server、SSE、upload/download、File/Stream return、async handler；Auto网络/HTTP协议及基础transport封装归本包，Web产品宿主/部署接线留983，涵盖取消/背压/断连/错误与权限。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### T-01 A期冻结记录（2026-11-16，r9执行）

**授权**：既定裁定模式（goal 续行 2026-11-16，无主包按建议开工 974 A 批）。worktree：`D:/autostack/.wt/lang-974/auto-lang`（分支 `plan-974-dev`，基面 master@03a3fc5db）；材料 `plan-artifacts/plan-974/` 已登记；无 junction/symlink。

**前置核实**：968 已归档（**全量调度器已交**：task/on 声明、Task.spawn、send/ask/reply/await/go/sleep、有界邮箱背压、虚拟定时器、stop-hook 排空，AV+a2r 双道——本包 async 基座 ✓）；1042 已归档 ✓；**973 在途（executing 0/10，他会话）**——依赖分账冻结（见下）。1043 已归档（加载层）。

**973 依赖分账（T-01 冻结）**：本包自有"OS socket/事件薄适配"（r2 合同：标准库以自身 Auto 协议＋OS 薄适配独立验收）——**核心腿（TCP 连接、HTTP client/server 语义、SSE 流、取消/背压/断连/错误）不前置 973**；**973 阻塞腿具名**：File/Stream return、upload/download 的文件体腿、io.* port 消费——待 973 io 域正式落主（缺证不 NA）。

**已交 seam 盘点（策略比较输入）**：`av-core/platform_io.rs`（1039/1040 已交）= `platform.io/1|mem/1|probe/1` 类别（caps `platform.io.stdout/stderr/write`、`platform.mem`、`platform.probe`；导入上限16/24、call/run 字节预算、句柄台账、Win32 实现）——**无 net 类别**（本包补位）；947/863 console=宿主 ConsoleHost recipe 模式；967 F0=外 provider 道（rustc/cdylib）。

**策略比较与选型（T-01 决定）**：S1=**扩展 platform_io 增 `platform.net/1`**（符号表+caps `platform.net.*`+字节/句柄预算+台账+Win32 std::net 实现；auto 源经 host_imports 消费）——与 1039 同纪律、OS 面原生、预算/取消/账本复用；S2=967 FFI Rust provider（cdylib socket）——层不符（OS 面非用户 FFI）、每 provider rustc 重；**保留为显式可选第三方 transport**（r2 合同原义）；S3=native 道独占——失 AV/a2r 对等，拒。**冻结 S1**。

**平台/命令/资源**：Windows x64；真实 TCP 以本机回环服务器（948 消费者探针模式）；Linux 腿归 1063（r8 裁定）。V 真实命令由 T-02/T-04 登记（拟：ace-av 组扩展 net 案例集＋case 语料 `ace-m2/net-974`，沿 977/PY 模式）。资源计量：沿 platform_io 预算形态，net 新增单位（连接数/收发字节/句柄）在 T-02 冻结并配边界反例。r5 具体输入锚（http_client_sync.at 等）由 T-02 开工时重核 hash。

### 本次复核补充：协议特性与host循环解耦（r2）

逐现用DNS/TCP/TLS/HTTP/redirect/proxy/timeout/WebSocket/SSE能力分母核实现；标准库以自身Auto协议＋OS socket/事件薄适配独立验收，第三方transport仅属显式可选provider，983后接产品Web，不以等待983形成循环。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### T-08 协议分母核（2026-11-16，证据基=net974 6/6 + stdlib/ace/std/net.at）

**分母来源**：959 S-16（net 14+vm14 / http 79+vm68 / sse 1+4 / url 22+vm23 的 .at 面）＋r2 合同（DNS/TCP/TLS/HTTP/redirect/proxy/timeout/WebSocket/SSE）。逐族对已交面（已交=真回环执行+独立oracle；未交=具名pending，非NA非静默）：

| 协议族 | 旧面（stdlib/auto/*.at 符号级） | 新面已交 | 证据 | 状态/缺口 |
|---|---|---|---|---|
| **TCP client** | net.at tcp_connect/read/write/read_all/read_line/write_str/close/peer_addr/set_read_timeout/set_write_timeout（10） | platform.net/1 connect/write_some/read_into/close（paced read/accept；caps/预算/台账/具名错码） | net974 roundtrip/negative（真回环） | **已交核心**；read_line/read_all/peer_addr=薄helper可加，具名pending（低危） |
| **TCP server** | net.at tcp_bind/TcpListener.accept/local_addr/close（4） | platform.net/1 listen/accept/local_port（预保留口召回；paced accept） | net974 server_leg（Auto听、真客户端驱动） | **已交** |
| **HTTP client** | http.at request/get/post/put/delete/head/patch/builder链/header/body/send/Response存取组 | std.net **全方法族** net_get/net_get_with_header/net_post/net_put/net_patch/net_delete/net_head/net_send_request | net974 http_api_face + std_net_packaging + http_method_family（PUT200/4+PATCH200/2+DELETE204/0+HEAD200/0） | **方法族已交**；Response全存取组（header值/cookie/重定向策略配置）/表单/JSON体=具名pending（header值提取依子串物化决定） |
| **HTTP server** | http.at Server.get/post/put/delete/static/listen + listen(host,port,handler)（路由/静态/handler注册） | Auto侧应答（listen+accept+读请求+按方法路由响应+close） | net974 server_leg（单连接路由） | **已交最小**；路由表/handler注册/多连接并发/静态文件=具名pending（静态文件腿973阻塞） |
| **redirect** | http.at 内建跟随（follow_redirects配置） | 客户端跟随（302→Location→重发→200） | net974 sse_and_redirect（30200→20000） | **已交**（单跳）；多跳/循环检测/max-redirs=具名pending |
| **timeout** | set_read_timeout/set_write_timeout(ms) | connect 5s/读写5s超时+**paced有界窗**（accept≤50ms/read≤500ms，静默窗返回可重试-10）；拒绝=-8具名 | net974 sse_and_redirect（refused -8）+负面组 | **已交**（语义形=固定窗+可重试，非旧per-stream可配置API）；可配置set_timeout API=具名pending |
| **SSE** | sse.at 事件流解析 | 客户端流式读至EOF+事件界（

）计数+状态行 | net974 sse_and_redirect（两事件120ms间隔流，20002） | **已交**（解析面）；事件字段存取/data多行/event回调形=具名pending |
| **DNS** | 旧tcp_connect(addr str)内含解析（无显式DNS API） | net_tcp_connect(host字节串)——std::net ToSocketAddrs **隐式解析**，无显式DNS面 | 平台实现事实（win32 backend） | **未交显式面**：解析控制/缓存/自定resolver=具名pending（旧面亦无显式API，义务=连接功能已交） |
| **TLS** | 旧无（https经reqwest侧） | **未交** | — | **具名pending**：按r2合同TLS属显式可选provider（967 FFI道或OS provider），非本批静默 |
| **proxy** | 旧配置形（http.proxy_url等） | **未交** | — | **具名pending**：客户端代理配置面 |
| **WebSocket** | 旧无 .at 面（vm侧8注册） | **帧层已交**：net_tcp_write_region（raw字节面，read_into镜像）+真RFC-6455客户端（Upgrade握手→101+canonical accept→掩码文本帧写入→服务端无掩码回显字节精确解析） | net974_websocket_frame_layer（0.06s） | **帧层/握手已交**；ping/pong/close帧/分片/大帧(>125扩展长)/并发会话=具名pending |
| **URL** | url.at encode/decode/encode_path_segment/encode_query_component/decode_query_component/encode_query/decode_query/parse_form/parse/build/join_path（11 pub fn） | **stdlib/ace/std/url.at**（encode×3+join_path；PLAN-1041内联） | net974_url_family_inline（独立oracle 0.05s） | **encode族+join已交**；decode/query族/parse/build=具名pending（子串物化核心决定：slice view→owned促升 或 char→str内建）；Map形归packages Collections面 |
| **upload/download** | http.at body/upload/download（File腿） | **未交** | — | **具名pending：973阻塞腿**（File/Stream域） |

**核结论**：已交核心=TCP双向+HTTP client/server核心+SSE解析+redirect+timeout语义（net974 6/6真回环+独立oracle）；**具名pending**=URL族（11符号）、TLS/proxy/WebSocket、HTTP全方法/全存取组、多跳redirect、可配置timeout、upload/download(973)。分母不删行、不NA——T-04按此表逐行记录，pending腿的关闭序=URL族（Auto可加）→HTTP面扩→WebSocket→TLS/proxy（provider道）；973腿随973落主。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r5当前）

**具体输入与期望。** 原件[crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at](../../crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at)（下列节选L6–L45；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() {
    let base = env.get("PLAN724_URL")

    // 1. GET + Response accessors
    let res = http.get(base)
    let code = res.status_code()
    let ct = res.header_get("X-Plan724")
    let body = str.from_bytes(res.body_bytes())
    print("get status:", code)
    print("get header:", ct)
    print("get body:", body)

    // 2. Builder chain with custom header
    let req = http.request("POST", base)
    let req2 = req.header("X-Plan724", "builder")
    let req3 = req2.body("builder-payload")
    let bres = req3.send()
    let bcode = bres.status_code()
    let bbody = str.from_bytes(bres.body_bytes())
    print("builder status:", bcode)
    print("builder body:", bbody)

    // 3. Two-arg normal post (must NOT hit the auth tuple)
    let pres = http.post(base, "post-payload")
    let pcode = pres.status_code()
    let pbody = str.from_bytes(pres.body_bytes())
    print("post status:", pcode)
    print("post body:", pbody)

    // 4. Stream for-in (EOF sentinel ""; explicit close)
    let s = http.get_stream(base)
    for c in s.iter() {
        print("chunk:", c)
    }
    s.close()

    // 5. last_status reflects the latest sync call
    let code2 = http.last_status()
    print("last_status:", code2)
}
```

本包所需独立结果：沿PLAN724本地stub各verb/二参post/headers/body/last_status断言；用动态端口真实TCP与独立服务端请求记录，Unicode/二进制body不截断。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** Auto网络API/协议实现→968任务/973资源→本包OS socket/事件薄适配（显式外库另走967）→真实TCP/流→typed响应/关闭。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct RequestContext { id: RequestId, method: Method, headers: HeaderList, body: ByteStream, cancel: CancelToken }
struct StreamLease { owner: OwnerId, state: StreamState, generation: Generation }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/a2r-std/src/http](../../crates/a2r-std/src/http) | 旧HTTP语义/同步异步adapter参考 |
| [crates/a2r-std/src/sse.rs](../../crates/a2r-std/src/sse.rs) | 流/EOF/close |
| [docs/specs/stdlib/design/http-stream-lifecycle.md](../specs/stdlib/design/http-stream-lifecycle.md) | 已有流生命周期研究/合同 |

**状态、错误与资源。** 断连/畸形头/body/超限/未授权与取消各分态；空chunk不随意当任意协议EOF，不用公开互联网或假response代transport。 中途取消/后台late丢弃/close释放端口与buffer；旧707取消债另账，不因新后端可用冒旧缺陷关闭。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at](../../crates/auto-lang/test/a2r/30_plan724/001_http_client_sync/http_client_sync.at) | 同步verb/响应/二参post完整矩阵 | `http_client_sync.expected.rs`；SHA256前16 `3ab20dc82b46f669` | 拟ace.m2.p974.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2r/30_plan724/002_http_client_async/http_client_async.at](../../crates/auto-lang/test/a2r/30_plan724/002_http_client_async/http_client_async.at) | 异步同语义 | `http_client_async.expected.rs`；SHA256前16 `a0b73ed0126acd5f` | 拟ace.m2.p974.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/a2r/31_plan727/001_http_transfer_sync/http_transfer_sync.at](../../crates/auto-lang/test/a2r/31_plan727/001_http_transfer_sync/http_transfer_sync.at) | 上传/下载/流失败与保旧 | `http_transfer_sync.expected.rs`；SHA256前16 `38e4a4275d44368c` | 拟ace.m2.p974.source03：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/src/tests/plan724_http_client_tests.rs](../../crates/auto-lang/src/tests/plan724_http_client_tests.rs) | 独立loopback oracle | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `57320d9701dec45b` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 同步/异步/stream精确API、loopback请求记录、body编码/headers多值/timeout/cancel/EOF规则冻结；不反向等983全产品。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

DNS/TCP/TLS/HTTP/redirect/proxy/timeout/WebSocket/SSE按原能力分母补六路径；复用中立端口/同一调度与协议算法，不为AV三来源另造executor，Python缺端口由本包/977-PY具名承接，不删API行。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

经973消费1043正式加载层；网络API/transport实现沿统一来源与目标选择，不按AV×provider×OS另造文件组合/loader，原完整网络和真实provider证明保留。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-stdlib-network.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-stdlib-network.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-stdlib-network.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |
### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-stdlib-network.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/ace-migration/m2-stdlib-network.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**网络/HTTP/流式协议及socket事件接线**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

默认地址/DNS、TCP/UDP API、HTTP/framing/redirect/proxy/stream/WebSocket/SSE等原协议逻辑由Auto实现，底层用Linux socket/syscall或Windows Winsock/事件API。OS TCP/IP栈允许；reqwest/hyper/libcurl/Tokio net等不能代默认自有net。TLS加密/握手等高级引擎暂可显式FFI，依赖和边界列明，不能接管普通HTTP逻辑。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

T-01在原有界前探冻结Winsock初始化/错误码、socket/非阻塞/IOCP或等价事件接缝、Linux socket/epoll、DNS配置/解析、取消/buffer保活及各协议原分母；T-02/03/T-08实现Auto协议/OS接线，T-04汇总验证。先闭共享算法与Windows真实路径，Linux同源平台分支交1063；不反等983整产品。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/ace-migration/m2-stdlib-network.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |
| SD-AUTO-NET | modify | docs/specs/ace-migration/m2-stdlib-network.md | 中立接口/外部transport可验→普通协议由Auto实现＋OS socket/事件，TLS等高级依赖显式分账 | 用户自有net要求 | AC-AUTO-NET |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 本地真实服务完成流式请求 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 慢消费者背压不无限增长 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 断连释放上传文件与stream | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 中途错误/取消保错误类别 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 无端口/环境排除不计PASS | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t stdlib ace-stdlib http a2r-std ace-host`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 本地TLS合法/坏证书按原政策分别成功/失败；redirect/断流/取消保权限与资源；必需协议不在最小HTTP样本覆盖时具名补测。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 沿PLAN724本地stub各verb/二参post/headers/body/last_status断言；用动态端口真实TCP与独立服务端请求记录，Unicode/二进制body不截断。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 断连/畸形头/body/超限/未授权与取消各分态；空chunk不随意当任意协议EOF，不用公开互联网或假response代transport。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 中途取消/后台late丢弃/close释放端口与buffer；旧707取消债另账，不因新后端可用冒旧缺陷关闭。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-AUTO-NET / AC-AUTO-NET | T-02/03/T-08及T-04：移除可选C/Rust/第三方网络provider，以默认Auto源码发真实loopback DNS/HTTP/stream等原适用请求；独立对端记录wire字节/关闭，审计实际调用链 | Auto协议解析/序列化及socket/事件适配来源正确，原错误/取消/背压通过；不能以reqwest/hyper/libcurl/Tokio net默认代理。高级TLS显式分账，Linux此时只审实现与1063移交，未来实跑。 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 协议特性与host循环解耦的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

- [ ] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

- [ ] **AC-AUTO-NET / work**：V-AUTO-NET满足，普通net/HTTP协议与资源逻辑由Auto实现并经OS薄入口真实运行；外部transport不能代默认native。T-02/03/T-08及最终T-04负责，原review核；Linux按AC-LINUX-PHASE移交。

## 8. 执行步骤

当前共8任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

- [x] **T-01｜M2-A｜work｜AC-01**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] 冻结记录见§5"T-01 A期冻结记录"：968/1042/1043 已交核实；973 依赖分账（核心腿不前置、File/Stream/upload 文件腿具名阻塞）；已交 seam 盘点（platform_io 无 net=补位面）；策略比较冻结 S1（platform.net/1 扩展）；平台/命令/资源冻结。worktree 已建。T-02 为下一实现单元（platform.net/1 provider＋最小 HTTP 往返）。

- [ ] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。
  [进行中 2026-11-16, worktree commit c226a7215] **OS薄适配+最小真实HTTP往返已交付**（T-02首个增量）：`platform.net/1` 全栈落地——av-core（CATEGORY_NET四op：connect/write_some/read_into/close；每动作caps；call/run字节预算；16-socket台账单属主账本NetSocket；混源region读沿console_write16纪律；具名错误码）＋Win32后端（std::net，5s超时，取消轮询写，槽位存储）＋mir_verify四net caps准入＋ace-core源表四符号注册＋**native道两导入点具名拒platform.net**（原支持域外，匹配保持穷尽）。永久证 `net_974.rs`：`net974_real_tcp_http_roundtrip`（**Auto侧自有HTTP协议**驱动真实回环服务器：GET写入→region读→状态行解析；独立oracle 200*100+43=20043；0.09s）＋`net974_negative_controls`（wrong-origin -3/invalid-handle -2/bad-bounds -4/double-close -6/bad-port -7/准入host.unauthorized且零dispatch）。回归：platform_io_1039+sched+console 48/48 PASS。**增量2已交（54e09069e）**：Auto侧HTTP协议API面——`net974_http_api_face`（旧a2p `30_plan724/001_http_client_sync` 义务：http.get存取组/自定义头builder链/两参post，协议全在Auto源：send_request+req_get+req_with_header+req_post+int_to_str，console profile owned-Str方言；回环服务器按方法路由GET→200"hello"+X-Echo/POST→201+载荷长回显；独立手算oracle 2000005+2000005+2010002=6010012，三请求成形服务；三测全绿0.06s）。**增量3已交（23d47952f）**：platform.net/1 **server面**——listen/accept/local_port三op（cap platform.net.listen；NetListener台账载荷；accept铸造新socket条目；16-socket预算；**paced有界等待accept**：每次调用5ms睡眠×≤50ms，可重试-10，不烧墙/步数）＋Win32监听器槽位/超时流/local_port召回＋mir_verify准入＋native穷尽匹配。**谜题已关闭（d090a48bb，net974 4/4绿）**：根因三层具名——①printf构建的补丁脚本把Auto body字面量与Rust客户端请求串都转成了真实换行（客户端实发29个LF字节，服务端CRLFCRLF界永不能匹配——非内核行为）；②accepted socket非阻塞复原改错误传播；③read增paced有界等待（5ms×≤500ms，静默窗返回可重试-10）——与accept同律，闭accept/read交错竞态。server腿E2E **4/4绿**（0.56s，多次复跑稳定），回归17/17。**增量4已交（cf1b29c9e）**：SSE流式解析+redirect跟随+拒绝负控——`net974_sse_and_redirect`（三真实连接：SSE 200头+两事件120ms间隔流式，Auto侧累积分批读至EOF，状态行+事件界解析，独立oracle 200*100+2=20002；redirect /start→302→/next→200=30200/20000；死端口connect拒绝=具名-8不挂起）；**net974 5/5绿**（新用例2.25s≤10s门）。**T-02余量**：timeout读窗语义（-10可重试已交）、多模块std.net包装、973阻塞腿。

- [ ] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-08｜M2-A｜owner_stage: work｜AC-06**：逐现用DNS/TCP/TLS/HTTP/redirect/proxy/timeout/WebSocket/SSE能力分母核实现；标准库以自身Auto协议＋OS socket/事件薄适配独立验收，第三方transport仅属显式可选provider，983后接产品Web，不以等待983形成循环。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [核已交 2026-11-16] 分母核表见§5"T-08 协议分母核"（证据基=net974 6/6；已交核心七族+具名pending清单+关闭序）；实现腿（URL族/HTTP扩面/WebSocket/TLS/proxy）按关闭序续施——T-08保持未勾至实现面闭。

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。
  [✅ 2026-11-16] T-04全V行以冻结受影响并集执行（974 worktree @206fdf901）：**ace-av=1257例PASS（8m37s含冷编译）+ ace-native=280 PASS + ace-source=597 PASS（1m27s）**，净974永久证 net974 9/9（client双向/http API/server/SSE+redirect/负控/URL族/方法族/WS帧层）＋std.net/std.url跟踪模块内联证。V-01真实流式请求✓（SSE/方法族）；V-02背压：有界窗(-10可重试)+预算(net run bytes)实证、慢消费者持续负压=具名pending（需长流服务器）；V-03断连释放：close回收+refused -8✓、上传文件腿=973阻塞具名；V-04中途错误/取消：错误类别具名(-1..-10)✓；V-05环境排除：回环动态端口全真实✓。TLS/proxy（V-08）：r2合同=显式可选provider道，自持面按合同不实现冒充——具名pending（provider裁定归967后继/用户）；V-COVERAGE六路径：语言侧AV实跑✓，其余路径按a2x域owner。机器占用核查0进程×2。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-974/auto-lang`，分支 `plan-974-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-974/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-04全V行) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: 206fdf901 | evidence: 冻结并集 ace-av 1257+ace-native 280+ace-source 597 全PASS；V-01..05逐行状态（含具名pending：长流背压/上传文件973/TLS-provider道）；新用例全部≤10s门 | next_responsible_role: T-05独立复审（停写后）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-08 WebSocket帧层+raw写面) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: 206fdf901 | evidence: net974 **9/9绿**（ws 0.06s，真RFC-6455往返）；net_tcp_write_region（0x81等帧字节非UTF-8的必然要求，native穷尽拒） | next_responsible_role: work（TLS/proxy provider道→T-04..07 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-08 HTTP方法族) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: 5925768e9 | evidence: net974 **8/8绿**（method_family 0.08s，独立oracle PUT/PATCH/DELETE/HEAD） | next_responsible_role: work（WebSocket→TLS/proxy→T-04..07 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-08 URL族实现腿) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: c397befd2 | evidence: stdlib/ace/std/url.at（encode×3+join_path 独立oracle）+net974 **7/7绿**（2.24s）；方言事实新增：slice右拼合法/字面量左拼拒/字面量实参不retag/借用参数不可直返owned（全数留痕迭代） | next_responsible_role: work（HTTP扩面→WebSocket→TLS/proxy→T-04..07 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-08分母核交付；实现腿续施) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: bd173aff0（核基线） | evidence: §5"T-08 协议分母核"逐族表（13族：已交7核心+具名pending 6+973腿，分母零删行） | next_responsible_role: work（URL族实现→HTTP扩面→…→T-04..07 或按裁定）


stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-03装配缝裁定+std.net包装) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: bd173aff0 | 决定: 缝=**(c) PLAN-1041内联单源约定**（语法事实：use-item仅scalar-modules/packages面；host-console设计单源，其stdlib故事即内联pub fn同拼写两世界）——`stdlib/ace/std/net.at`（pub fn协议模块）+`net974_std_net_packaging_inline`（verbatim内联、仅公共helper、真回环6010012、0.09s）；**packages面接线=具名后续**（packages-lane platform host，随973 io装配，非静默缩范围） | evidence: net974 **6/6绿**（2.24s）；零编译错误 | next_responsible_role: work（T-08协议分母核→T-04..07 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: design-probe (T-03装配缝冻结输入) | 记录: T-03开缝探针——console前门按设计单源（ConsoleProgram=单src、AV_HOST_CONSOLE_PROFILE、restore/persist同单源身份），platform.net/1 的宿主执行只在此道；**多模块 std.net 包装需 T-03 裁定装配缝**：(a) console前门接纳 packages 产物（identity/admission 扩展）或 (b) 新宿主车道复用 execute_platform。此缝是 T-03"真实consumer、目标/装配/权限接线"的首个有界决定，非静默缩范围——std.net 多模块包装因此挂 T-03 决定后实施。已交面不变：net974 5/5绿。

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-02 inc4: SSE+redirect+refused) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: cf1b29c9e | evidence: net974 5/5绿（SSE/redirect 2.25s）；独立oracle 20002/30200/20000/-8 | next_responsible_role: work（std.net包装 或按裁定/T-03/T-08）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-02 inc3 finding CLOSED) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: d090a48bb | evidence: net974 4/4绿（server腿0.56s稳定复跑）；回归17/17；根因三层具名（转义损坏/非阻塞复原/paced read） | next_responsible_role: work（SSE/redirect/timeout 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: in-progress (T-02 inc3: server面ops落地, server腿E2E具名挂起) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: 23d47952f | evidence: 三测3/3绿+回归13/13；server腿#[ignore]具名发现(29/31回环部分投递)非静默 | next_responsible_role: work（谜题关闭/SSE/redirect 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: in-progress (T-02 inc2: Auto http API面) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: 54e09069e | evidence: net974三测全绿0.06s（roundtrip/negative/http_api_face 6010012精确命中）；回归platform_io_1039 8/8 | next_responsible_role: work（SSE/server腿/redirect/timeout 或按裁定）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: in-progress (T-02 OS seam increment delivered) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev | code_commit: c226a7215（+754行：av-core platform.net/1全栈+Win32后端+mir_verify准入+core源表+native具名拒+net_974双测） | evidence: net974双测0.09s绿（真实TCP往返20043+六负控）；回归48/48；新用例≤10s门✓ | blockers: T-02完整协议面（http API/SSE/server/redirect）余量；973阻塞腿不变 | next_responsible_role: work（T-02续：Auto http协议API面+SSE+server腿，或按裁定继续）

stage: work | PLAN-974 | r9 | 2026-11-16 | outcome: pass (T-01) | worktree: D:/autostack/.wt/lang-974/auto-lang @ plan-974-dev（基面 master@03a3fc5db，纯冻结核验无代码提交） | task_ids: T-01 | evidence: §5"T-01 A期冻结记录" | authorization: 既定裁定模式（goal续行2026-11-16） | blockers: 973阻塞腿具名（File/Stream/upload文件腿/io port消费）；GD无关 | next_responsible_role: work（T-02：platform.net/1 薄适配＋最小真实HTTP往返I/V，沿977/PY批模式）

2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r8：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-12 stage:new / PLAN-974 r5：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原8任务/全部AC与V/依赖/阶段/预算和drafting 0/8；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
