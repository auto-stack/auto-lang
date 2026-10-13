# HTTP 强化进度与 v0.6 后续路线（2026-10-13）

## 1. 调查口径与结论

调查基线：auto-lang master `78ac2ba8a`，v0.5.0 为其祖先；775 已将已提交的 v0.6 更新汇入默认分支，功能计划仍按各自状态继续。
本次只读代码/规范/既有复审收据，更新设计与新计划，没有运行 Cargo、HTTP 负载或发布操作。历史测试结果不能充当当前 HEAD 的新验收。

**已完成基础建造阶段，进入返回语义、执行纪律与跨后台一致性的加强阶段。** 不是 HTTP 传输只实现了一半：11 个主要计划和 2 个后续修复已经交付，默认 VM HTTP 使用成熟协议库，异步网络等待、流式 IO、文件上传下载和受单代理保护的部署基线均有交付合同。
未达到的是“任意 api.at 业务体在 VM/生成 Rust 都等价、CPU/阻塞操作不会拖住 owner、整个公共 API 面均经语义验证”的下一层目标。不能以计划数或目录中的符号数换算功能完成百分比。

## 2. 已交付工作

| 阶段 | 计划（均 archived/delivered） | 当前交付边界 |
|---|---|---|
| 初始安全修复 | [696](../archive/696-stdlib-http-server-runtime-hardening.md) | 分段 body、慢 SSE 让出、VM 跨线程指针转运修复；多后台/拓扑文档。 |
| 成熟协议传输 | [699](../archive/699-vm-http-transport-axum-bridge.md) | 默认 VM `#[api]` HTTP/1.1 改 Axum/Hyper；网络与 VM owner 只传 owned Send 数据，有界队列、读取预算/关闭。 |
| 普通异步请求 | [705](../archive/705-vm-http-handler-async-lifecycle.md) | 段执行、External/Internal Future 等待、请求作用域、取消/期限/迟到结果不复活。 |
| 外部流 IO | [707](../archive/707-vm-http-stream-async-relay.md) | 外部 HTTP/SSE 读取 park/resume、增量解码、背压、managed 实际 abort；stream 终结回收。 |
| Rust/a2r 客户端 | [724](../archive/724-a2r-http-client-async-convergence.md) | 两 facade 共享 reqwest async 内核，typed 结果、流等待/取消、async lowering；P707-R1 测试提交协议债已闭。 |
| 客户端文件传输 | [727](../archive/727-http-file-transfer-lifecycle.md) | 下载与上传增量执行、staging/提交、续传校验、进度/取消、queued 期限。 |
| 服务端文件下载 | [729](../archive/729-http-server-file-responses.md) | FileResponse、GET/HEAD、单 Range/条件请求、受限根目录、有界 pump；VM/Rust 共享无 Axum 类型的宿主执行。 |
| 服务端文件上传 | [730](../archive/730-http-server-upload-ingress.md) | 单文件 multipart/raw、有界字段、授权先行、staging/create-only commit、取消清理；与 727/729 互通。 |
| API/生成正确性 | [734](../archive/734-api-contract-and-generation-integrity.md) | 参数/返回分类、类型化绑定、i64 API 编组、生成真实性/新鲜度/错误诊断；保留明确历史边界。 |
| 部署基线 | [736](../archive/736-http-service-deployment-and-operations.md) | `auto service`、loopback 默认、严格配置、Host/CORS/单 trusted proxy、预算、ready 身份/日志/排水关闭；真实 Nginx HTTPS 代理、文件/SSE/混合 IO 负载。 |
| 多后台装配基础 | [738](../archive/738-stdlib-assembly-manifest-and-core-validation.md) | 实际装配 manifest 与独立 inventory、选层/源映射/缓存身份、六核心 strict 证明、独立 native/Rust 适配合同、schema4 双指纹。 |
| 装配身份修复 | [751](../archive/751-empty-lock-identity-th-tail.md) | 缺失/空 Cargo.lock 区别、生成/消费身份一致性；完整 HTTP 尾档分诊。 |
| HTTP 契约/启动修复 | [755](../archive/755-http-contract-startup-diagnosis.md) | 公共面 41 项缺契约与 5 项漂移修复、契约覆盖/merge 守卫、启动 Err/panic 有界诊断，清除上传/redirect 超时族。 |

698 publisher SSE、702 handler 段机制、711 UI CPU slice、746 Playground VM deadline 是相关工程；它们不能替代 HTTP 返回语义或 HTTP CPU 请求隔离的专项证据。

### 最近验收信号

- [755 2026-10-11 落地后独立核查](755-landed-independent-check.md)：102 项 HTTP 串行实跑，101 过、1 个在册 back_proxy corpora 参数红、0 timeout；另 4 个启动/契约守卫通过，合并 106 项。早先 14 个上传/redirect timeout 已清零，不再列为当前阻塞。
- 最新批量收据 `docs/plans/.last-batch-regression.json` 覆盖 766、commit `3ecd375d6`（2026-10-12），tf/tt/tb；tf 有两项负载 flake，tt/tb 全过。该收据没有 th 腿，不能宣称重新覆盖了合流后的全 HTTP 面。
- 当前还没有本次调查基线上的新 HTTP 全档收据。后续实施在新树按规定 scoped + 串行完整 th 对账，不以旧绿替代新验证。

## 3. 当前实现到底是什么

### Server 与 task/Tokio

VM 默认服务：Axum/Hyper 在网络侧处理 HTTP/1.1，owner LocalSet 独占 `!Send` AutoVM；有界请求桥 + oneshot/owned 回复连接两侧。Auto task/Future 的等待与恢复由 VM 段状态机实现，不是把 AutoVM 或 task 交给 Tokio 任意线程运行。
Rust 生成服务：api.at 经 auto-man 生成 Axum/Tokio 后端，与 VM 是两条独立路由/handler 实现。目前 VM Axum 0.8、生成 workspace Axum 0.7；文件/上传执行器采用版本无关投影共享，不共享 Axum Response 类型。
客户端：VM managed job 与两 Rust facade 的 async 核心/等待协议分别适配；取消与背压已有实证。不能把“使用 Tokio”理解为所有语言 task 语义已统一。

### 多后台装配

公共 `.at` 在前、选定目标层在后；执行目标 vm/rust/c 与 native/browser 环境分开。Rust 可选择实际宿主 producer，没有 `.rs.at` 不等于没有实现；反之有文件不等于生产消费者已使用。Vue/use.web 是前端适配链，不自动意味着 HTTP 后端使用 `.vue.at`。

现在已有正式 [assembly-manifest](../../specs/stdlib/design/assembly-manifest.md) 与 [backend-assembly](../../specs/stdlib/design/backend-assembly.md)。738 解决了原来“是否有文档/实际选择可追溯”的基础问题，并区分 declared/resolved/bound/signature_checked/executed。六核心被引用闭包严格验证；全库 inventory 包括失败/未验证分母。**签名证明不是所有目标行为相同的证明。**

schema4 的 fingerprint 是消费者中立装配身份，consumer_fingerprint 是具体消费者收据身份；生成新鲜度与 ready 核对不得混用。

这套分层机制足以继续承载当前目标，无证据要求推倒重来。接下来补资源语义/生命周期与执行样本，而非仅加目标后缀或全符号白名单。

## 4. 未闭合的主要缺口

| 优先级 | 当前代码/规范证据 | 后续方向 |
|---|---|---|
| P1 返回资源消歧 | http_server.rs::marshal_handler_value 对 i32 查 iterator，i32/i64 查 Response 表；734 §5 为旧 int wire 保留兼容。不同计数区只降低误撞概率，不能证明普通业务数字不是活跃资源 id。 | 776：逐值来源、owner/代次与透明兼容，不用整数分区作类型证明。 |
| P1 异步终值差异 | final_value_reply 另走 File/Upload + JSON；value_to_nv_via_task 只转换少数 Value 变体，其他→null；Failed/缺结果也→null。 | 776：统一成功终值；全宽/复杂 JSON 与资源身份保留，区分成功 Nil 与失败。此为静态代码观察，反例由实施门禁冻结。 |
| P1 生成真实业务体/流 | [742](../742-api-handler-body-parity.md) 元数据仍 executing/current_step=0，但主线已有 List/companion 导入、borrow 与 Option/404 修复，integrity Spec 也已记这些支持形态；不将旧清单三缺陷直接当当前可复现红。api_gen.rs 的传统 Stream 分支订阅 events bus。现有 bus SSE 验证不能证明任意 generator/外部流 body lowering。 | 742 继续；另立真实流 handler 生成/生命周期计划，精确核查 strict 生产入口，不把兼容模板位置自动等同于当前必经调用图。 |
| P1 HTTP CPU/native 阻塞纪律 | HTTP 默认使用 call_fn_by_name_segment/call_closure_segment；CPU slice 已在 UI 路径实施，但 HTTP parked_is_ready 对 CpuRunnable 不拾取，代码明示 HTTP 不产该凭据；746 全 VM deadline 也不是每请求隔离。 | 接入请求级可续执行/公平预算，分类阻塞 native；CPU 与原生阻塞需分别验证，跨 await store 保持非事务合同。 |
| P2 更深公共面与拓扑一致性 | 装配 strict 只证明选中引用与签名。Builder、Request/Response、IPC/merged/back-proxy 和 standalone/embedded Rust 的支持边界不同；P734-D4 泛型 json.encode/str+int 数值面仍在规范记账。 | 按真实公共面更新 executed 矩阵，优先实际应用必需路径；不要求 C/browser 监听 HTTP 或所有入口立即全等价。 |
| P2 运维/回归收尾 | 755 后仍有 back_proxy 参数在册红；736 R1 父进程身份核对、R2 stderr 原始 path、R3 事件风格、R4 auto-man 测试目标腐坏等历史债需启动时核实。 | 当前代码/测试重新分诊，修实际问题、退役失效债条目，再完成真代理/负载/长寿/取消/关闭证据。不得将历史债直接宣称当前仍可复现。 |

类型“精确裸名”分类仍不是完整 resolved 符号身份；同名用户类型/别名必须作为下一计划反例。更多数值/通用 native 资源工程按证据另案，不把 HTTP 计划扩大为全语言重构。

## 5. 还要多少步

定义这一轮目标为：**当前受单代理保护的 HTTP/1.1 API 服务，在返回值、IO/CPU 执行边界、VM/生成 Rust 已承诺能力和部署验收上有完整合同与证据**。暂不纳入内置 TLS、HTTP/2/3、完整 WebSocket、直接公网认证、所有 C/browser 后台等选配工程。

| 顺序 | 工作包 | 预估新增计划 | 收口结果 |
|---|---|---|---|
| 1 | 返回资源与 sync/async 终值 | 1（[776](../776-http-return-resource-contract.md)） | 普通数值不误判资源；异步 Response/SSE/JSON 正确；生命周期有证明。 |
| 2 | HTTP CPU 公平预算与阻塞 native 隔离 | 1–2 | 长 CPU 与支持的阻塞操作不会使短请求失去响应；取消/关闭与状态一致性边界清晰。 |
| 3 | 真实 handler/SSE 的 Rust 生成与跨端对照 | 1–2，另有 742 在途 | 742 复核已合入实现并完成原合同收口；已承诺流端点执行真实 api.at 体，失败/取消/背压一致或明确拒绝。 |
| 4 | 公共面与部署收口 | 1 | 选定 API 支持矩阵、已知红清偿/准确分诊、完整串行 th 与实际代理/负载/资源证据，文档可自圆其说。 |

**当前先按 4 阶段、约 4–6 个新增中等计划估算，另有 742 在实施。** 这是规划工作量，不是已批准的全部后续计划，也不是“任何 HTTP 功能全部完备”的承诺。第 2/3 阶段是否拆为两项由有界调查与实际 diff 决定；完成 776 后重新核对剩余面。

## 6. 下一项计划与兼容决定

[PLAN-776](../776-http-return-resource-contract.md)：8 项任务、9 项验收、7 项规范增量；先冻结最小资源表示，再依次处理生产/传播、统一编组、Future 终态、存活、解析来源/消费者和 wire 验收。

旧 int 声明的资源源码兼容保留；不能简单加“必须声明 Response/Iter”门清绿。Response 全局常量可重复回复，不能改为查表即 take。SSE/File/Upload 仍按既有单次移交与 scope 收口。所有资源跨网络桥先投影为 owned 数据，VM 引用不跨线程。

738/751/755 archived 不重开；现行 741 是 AC/HIR native core，不能复用旧会话编号。742 的执行状态与 774 的 AC 资源实验各自保持。

## 7. 文档差异处理

本次更新 Design 33 与 00-intro 的进度/路线，并保留原初审计为历史基线；canonical Specs 不在起草阶段改写。
backend-assembly 的部分总表仍写“manifest/服务端文件另案”，与其后 729/730/738 专节及交付事实矛盾；handler-async-lifecycle 底部也残留 a2r client 未迁移描述（724 已交付）。本期 SD-07 与后续复审应明确修正这些陈旧摘要，不能将陈旧行当成未实施证据。

## 8. 关键输入身份

以下 Git blob 均取调查基线 `78ac2ba8a`；用于后续识别输入变化，不表示本次跑过行为门禁。

| 输入 | Git blob |
|---|---|
| docs/specs/stdlib/design/api-transport-contract.md | c536f1abfdcf78d91f389d3804a01fb3b1ae74c3 |
| docs/specs/stdlib/design/assembly-manifest.md | 383d2dabeca541647be1444022b31b9bef63e0fd |
| docs/specs/stdlib/design/http-handler-async-lifecycle.md | 88aafe7724b84db55bcd56c48382fbbbc4e9366e |
| docs/specs/auto-man/design/api-generation-integrity.md | 3fa554c551389be8250455dbab03bc38f4e1080d |
| crates/auto-lang/src/vm/ffi/http_server.rs | 62c016736cb4c5854dcef036df7b5a3a8b0d2972 |
| crates/auto-lang/src/vm/ffi/stdlib.rs | 8bec56841f6c5ff6c557ee50adc4c94e697ce5a5 |
| crates/auto-lang/src/vm/engine.rs | d18e30e86cc114fd30749cbe21f5ea63ad35478a |
| crates/auto-lang/src/api/contract.rs | 286696b2f0ba1d6cf2d1f4abed671c937f7c8d85 |
| crates/auto-man/src/api_gen.rs | 9881ac80a736d3149fdc98da21fb0c0162e44ade |
