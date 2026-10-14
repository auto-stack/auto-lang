---
plan_id: PLAN-776
status: execution_done
feature_name: http-return-resource-contract
plan_revision: 1
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-14
supersedes_spec_components:
  - docs/specs/stdlib/design/api-transport-contract.md
  - docs/specs/stdlib/design/http-handler-async-lifecycle.md
  - docs/specs/stdlib/design/http-stream-lifecycle.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/stdlib/project.md
  - docs/specs/auto-lang/vm/design/ffi.md
  - docs/specs/auto-man/design/api-generation-integrity.md
new_spec_components:
  - docs/specs/stdlib/design/http-return-resources.md
touched_goals: [GOAL-003, GOAL-016]
affects: [auto-lang/vm, auto-lang/api, stdlib, auto-man]
current_step: 8
total_steps: 8
---

# [PLAN-776] HTTP 返回资源身份与同步/异步终值统一

## 0. 变更摘要

v0.6 主线恢复 HTTP 强化。738/751/755 已交付，不再等待装配基础工程，也不重开其归档计划。
本计划处理默认 VM HTTP 返回边界：普通数值不能因命中资源表而变成 Response/SSE；真实资源必须携带来源身份；同步与异步成功终值走同一编组入口，失败与缺失终值不得伪装成成功 null。

保留已有 int 声明的 redirect/SSE/AutoMusk 响应源码兼容。将“资源 id 高位分区 + 查表猜测”替换为按值传递的资源凭据，保持 699 的 owned Send 桥和 705/707/729/730 的生命周期。生成 Rust 任意流业务体另案，742 的 Notes handler 修复保持在途。

## 1. 目标

1. 区分数据与 HTTP 返回资源，覆盖 Response、SSE/Iter、FileResponse、UploadReceipt；禁止普通整数通过注册表碰撞取得资源身份。
2. 统一 named/closure handler 的同步值、ready Future、park/resume Future 与 Internal async body 的终值编组。
3. 维持既有响应构造、访问器、全局响应常量重复返回及旧 int 源码的 wire 行为。
4. 建立 owner/种类/代次、所有权与终结合同，避免跨 VM 误用、过期重放、响应表无限滞留和迟到结果复活。
5. 提升返回分类的符号来源证明，并使当前生成/非 HTTP 消费者对新增身份保持明确的支持边界。

### 非目标与范围

- 不实现全语言所有权/借检查器，不整体改写 NanoValue ABI、actor/store 或所有 native 资源。
- 不扩充 TLS、HTTP/2/3、通用 WebSocket、multipart 多文件事务或 CPU 调度。
- 不实现任意 Auto generator 的 Rust SSE lowering；不接管 742 的 List/import/comparison 修复或 774 的 AC 数据资源实验。
- 不合并 Axum 0.8 与生成服务 0.7 的类型；跨线程不得传 VM 引用或裸资源凭据。
- 覆盖 HTTP egress 所用生产者和必要的 VM 值传播；client HTTP Response 若通过同一生产者/表进入服务返回，也在身份边界内。文件传输客户端及通用 FFI 资源不扩面。

## 2. 架构方案

```text
真实资源生产者 → owner 内资源凭据（kind + owner/session + generation + resource）
                       ↓ 局部/函数/共享状态/容器/闭包/async 传值保留身份
handler 成功终值 → 单一返回分类与编组入口
                       ├─ JSON data → 按值序列化
                       ├─ reusable Response → owned status/headers/body 快照
                       └─ SSE / File / Upload → 按既有合同移交/消费
失败、缺失或无效凭据 → 有界错误；取消后的迟到值 → 回收且不发送
```

资源凭据不得通过“整数属于某个区间”、本请求曾创建资源、静态返回声明或表命中推断。
声明用于校验与能力选择，真实来源用于证明该值为资源。两者职责分离，旧 int 声明可承载真实资源，但普通 int 永远是数据。

T-01 在现有 VM heap/ref、RC、FutureValue 与 native 栈 ABI 上冻结最小表示方案；优先复用已有受控引用机制。新资源表、内部 carrier 或调用点适配可按证据选择，不预设全局 tag 布局。若必要实现超出上述范围，必须 needs_replan，不能改验收为“更大的 id 分区”或放宽严格门。

## 3. 技术栈

Rust / Auto .at；AutoVM owner LocalSet；现行 NanoValue/Value/HeapObject；Axum/Hyper owned 请求响应桥；现行 Tokio 完成通知、SSE 背压及文件/上传执行器。

主仓默认检出只记计划与文档。实施树：`D:/autostack/.wt/lang-776/auto-lang`，分支 `plan-776-dev`；按 `scripts/new-wt-group.sh lang-776 --branch plan-776-dev` 配置 auto-down 兄弟依赖，禁止链接。

## 4. 需求分析与背景调查

### 授权与基线

- 用户 2026-10-13 明确：v0.5 已发布、v0.6 已汇入主线，恢复 HTTP 强化，要求盘点并规划下一项。本会话授权调查、设计与计划簿记；没有实施、合入、发布授权，没有预算或自动续跑要求。
- 调查基线：master `78ac2ba8a`；v0.5.0 是其祖先。编号预留提交 `2be254c88` 仅包含本计划骨架与 .next-id。775 分支合流不改变 742 等功能计划自身的执行状态。
- 状态/路线：[2026-10-13 HTTP 进度审计](reports/http-progress-v06-20261013.md)、[Design 33](../design/33-stdlib-runtime-and-http.md)。旧会话 HTTP“741”没有可执行合同；现行 741 是 AC/HIR native core，不复用编号。

### 现行规范和代码证据

| 输入 | 基线事实与本次变更动机 |
|---|---|
| stdlib/project；api-transport-contract §3/§5 | 734 为保留 6 个旧 wire 用例而保留 iterator/Response 查表兼容；固定普通值 734734 不是真实活跃 id 碰撞证明。P734-D7 是历史批准边界，本计划显式替代它。 |
| vm/ffi/http_server.rs::marshal_handler_value（5592 起） | i32 命中 vm.iterators 即 SSE，i32/i64 命中 lookup_http_response 即响应；仍无按值来源凭据。 |
| 同文件 final_value_reply / value_to_nv_via_task（5816 起） | async 终值单独处理 File/Upload 后走 JSON；转换只列 Int/Float/Double/Bool/Nil/Str，其他变体为 null。Failed/缺 result 也落 null。 |
| vm/ffi/stdlib.rs::HTTP_RESPONSES / lookup_http_response / insert_http_response | thread_local 表保存 owned 数据，lookup 克隆而不移除；还被 client Response、访问器与 legacy 构造消费。不能简单改成每次 take，也不能假定每请求结束可清空全部表。 |
| vm/ffi/musk_response_ctor.rs::push_response；axum_adapter.rs | 真实兼容生产者返回 i32 句柄，旧 int 源码不得只因声明门而失效。 |
| vm/engine.rs::FutureValue；heap_object.rs；api/contract.rs / types.rs | Future 结果为 Option<Value>；返回分类依据显示串裸名，未证明同名用户类型与 stdlib 类型不同来源；需核查完整符号解析路径。 |
| assembly-manifest；backend-assembly；vm/design/ffi | 738 的 actual/reference proof 与 schema4 双指纹；755 公共 native 独立适配契约与 CFFI merge 存活守卫。资源值适配须和公共逻辑身份一致。 |
| auto-man/design/api-generation-integrity；api_gen.rs | 生成轨返回分类和 strict generation 是独立消费者；不能由 VM 修复宣称任意 Rust SSE 已支持。 |
| 755-landed-independent-check | 2026-10-11 历史独立串行证据：HTTP 102/101/1 在册 back_proxy 红/0 timeout，另 4 个守卫。不是本次基线的重新执行结果。 |

所有路径相对仓根；行号为调查时定位，执行以符号和启动时版本为准。T-01 保存实际输入 Git blob/hash 与能力矩阵。

### 依赖与并行边界

738、751、755 已归档交付；无需等待新实现前置。742 元数据仍 executing，但主线 api_gen.rs 已含其 List/companion/borrow/Option 返回修复，canonical integrity 也已记录这些形态；未经其最终验收不能宣称整项完成。它涉及业务体初始化/导入，T-06 若触同文件仅维护返回身份消费；实施前对最新合入内容重基并记录冲突，不替 742 填完成。774 不改旧 VM，与本计划没有技术依赖。

## 5. 详细设计

### 5.1 最小决策包

T-01 产出 `docs/plans/reports/776-return-resource-decision.md`（新），必须冻结：

- 所有真实 producer、wrapper、访问器、iter/Future 到终值路径，以及 sync/async 入口分母。
- 凭据表示、owner/session/代次、合法传播与数值转换规则、错种类/过期/溢出/跨 owner 诊断；任何可制造普通数字的操作不能制造资源身份。
- legacy int 声明与 native 栈适配；局部/helper、共享变量、容器取出、闭包捕获、Internal/External Future 的身份传播。
- 响应重复返回与可变访问器语义；临时引用终结、逃逸值存活、服务关闭清理；SSE/File/Upload 的单次移交规则维持原合同。
- 成功 Nil、Failed、缺失结果、无法转换和取消的不同终态，及错误 wire 类别。
- 支持矩阵与测试 oracle。调试接口仅用于测试观察实际 id/存活数，不成为可伪造生产资源的公共 API。

### 5.2 编组与异步终值

成功返回统一进入一个 owner 侧分派入口，Future 的展开/等待仍由 705 状态机负责；不得递归把资源误识别为 Future。
Response 的 status/headers/body、SSE 的帧与 lifecycle、File/Upload 的原协议不改。
完整 JSON 支持值在 sync 与 async 边界同形，至少包含全宽 i64、str、bool、Nil 和现行 JSON 支持的 record/list；不得把无法转换静默变成 null。

Failed/内部结果缺失/不支持的终值产生有界 500（或在生成/能力检查阶段明确拒绝）；显式成功 Nil 才是 200 null。
取消、断连和关闭继续依确证作用域终结：迟到结果不回包，所携资源必须回收。不能把全部 VM 的 deadline 用来取消单一请求。

### 5.3 资源存活与兼容

按真实值引用持有资源，覆盖保留在共享状态的响应常量。重复返回 Response 复制 owned wire 快照，不能第一次回复就消费全局常量；临时结果失去最后引用时回收。
SSE/File/Upload 的单次移交、消费者关闭、半关闭与期限保持现行规定；错误凭据不能消费另一资源。

全局 native 表若暂保留，必须绑定 owner 与引用生命周期，不能只用 thread_local 作为隔离证明。id/代次耗尽必须拒绝分配，不能重用而使旧引用复活。

### 5.4 返回类型来源与消费者

基于实际解析/导入/声明来源分类专属返回身份；同裸名用户 Response/Iter/FileResponse 是普通用户类型，类型别名与 stdlib 限定名按其真实来源解析。
公共逻辑类型与 VM 载荷差异以独立适配契约证明，保留 738 strict 门、755 契约覆盖/CFFI merge 守卫。
生成收据/ready 使用 consumer_fingerprint；跨消费者装配对拍使用 fingerprint。新适配输入参与实际来源身份，旧产物保守判陈旧。

本期不承诺生成 Rust 任意流体。既有 Rust JSON/File/Upload 和已支持的 bus SSE 必须保持；不支持的新形态在生成前给出 endpoint/file/stage 诊断，不伪装成模板成功。
merged/IPC/back-proxy 维持现行调用与能力合同；资源包装不得泄漏成 JSON 裸 id，HTTP 专属 File/Upload 仍受原矩阵约束。

### 规范增量

下表是拟议 delta；本次不修改 canonical Specs，执行后由 review/merge 按实证沉淀。

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/stdlib/design/http-return-resources.md | 新增逐值来源、owner/代次、传播/释放、重复快照与单次移交合同 | 资源身份不等于数字表项 | AC-01、AC-02、AC-05、AC-06 |
| SD-02 | modify | docs/specs/stdlib/design/api-transport-contract.md | 替代 §3/§5 查表兼容；保留旧源码，按真实来源分派；类型分类保留解析来源 | 闭合 P734-D7，防同名/碰撞 | AC-01、AC-02、AC-07、AC-08 |
| SD-03 | modify | docs/specs/stdlib/design/http-handler-async-lifecycle.md | 成功终值统一；Failed/缺结果不再 200 null；成功 Nil 保留；明确复杂值/全宽值 | 修正 Phase A 降级边界 | AC-03、AC-04 |
| SD-04 | modify | docs/specs/stdlib/design/http-stream-lifecycle.md | 增加返回凭据移交/异步返回连接点，取消/背压合同保持 | 避免两套 SSE 出口 | AC-03、AC-05、AC-06 |
| SD-05 | modify | docs/specs/auto-lang/vm/design/ffi.md | HTTP 资源 native 逻辑身份、实际载荷与访问器适配规则 | 跨接口来源不丢失 | AC-02、AC-05、AC-07 |
| SD-06 | modify | docs/specs/auto-man/design/api-generation-integrity.md | 新返回身份输入纳入来源/陈旧判定，守住双指纹与能力诊断 | 不消费旧或不支持产物 | AC-07、AC-08 |
| SD-07 | modify | docs/specs/stdlib/design/backend-assembly.md；docs/specs/stdlib/project.md | 回写本期支持矩阵，纠正已交付文件/装配仍另案的陈旧描述 | 用实际 executed 证据更新边界 | AC-08、AC-09 |

## 6. 测试设计

新增 `crates/auto-lang/src/tests/plan776_http_return_tests.rs`，登记于现行 tests.rs；测试观察 hook 仅 cfg(test)。wire 测试命名 `http_e2e_plan776_*` 并按现行 test-http-e2e feature 登记，使完整 th 确实覆盖；unit 族不依赖网络。真 TCP 用 OS 临时端口、有界启动结果与 readiness；复用 755 诊断机制，不吞 Err/panic，不以 sleep 或跳过替代 wire 断言。

| 族 | 观察与 oracle |
|---|---|
| 真碰撞 | 保持资源活跃，取实际 id，再构造同值普通 i32/i64；普通值 wire 200 JSON 数字，资源仍能按原协议回复。Response、iterator 两族均覆盖，不能用任意常量冒充碰撞。 |
| 逐值身份 | 同一 handler 按条件返回真实资源或同值数据；先构造资源再返回数字；helper/local、共享变量、容器取出、closure、Future 等传播之后判别不变。 |
| sync/async | Response/SSE/File/Upload 及 JSON record/list/i64/str/bool/Nil 对照；Ready、External 延迟完成、Internal body await 三形态；named 与 closure 入口。 |
| 终态负向 | Failed、缺 result、失效 owner/代次/错种类、重复单次移交、迟到完成；错误有界，无成功 null、误消费或资源复活。 |
| 存活 | 全局 Response 连续两请求可返回；临时响应/取消/关闭丢最后引用后实际表项与引用计数回到预热基线；至少 1000 次创建返回闭环，RSS 仅辅助，不以 RSS 单独证明释放。 |
| 兼容/来源 | 原 6 个 int wire 用例保持源码和断言；同裸名用户类型/别名；755 两守卫；738 双指纹/751 absent-vs-empty lock；实际生成 Rust 现行能力运行，不手写业务体代替生成。 |

T-01 为 External Future 测试冻结真实完成入口；允许 host 完成源设置 gate，不能直接伪造被测编组成功。
复杂 JSON 与 i64 仅修返回路径；若发现无关 VM 算术缺陷，分诊另案，不扩为全语言数值重写。

## 7. 验收标准

- **AC-01**：活跃 Response/iterator 的真实同值普通 i32/i64 永远 200 JSON 数据；资源不被消费，随后仍可正确回复。以真 TCP 碰撞族证明。
- **AC-02**：真实资源经 §6 各传播形态保留来源；同一 handler 的资源/数据条件分支可区分；6 个 legacy int wire 用例和访问器原断言全通过。
- **AC-03**：同步与 Ready/External/Internal async 成功终值同形，覆盖四资源族及 §6 JSON 值域；Response 不变数字，SSE 不变 JSON，i64/复杂值不截断或降级 null。
- **AC-04**：Failed/缺失/不可转换有界拒绝或 500；成功 Nil 为 200 null；取消迟到值不回包且释放，无任务或结果复活。
- **AC-05**：错 owner/种类/代次、过期及分配耗尽受控失败，不误取其他表项；Future 位模式普通数值守卫保留。
- **AC-06**：保留响应常量两次返回正确；临时资源至少 1000 次闭环后存活量回到预热基线；SSE/File/Upload 完成/断连/取消/关闭依原合同收口，背压及半关闭无回归。
- **AC-07**：实际解析来源区分用户同名与 stdlib 身份；新增适配合同充分，755 覆盖/merge 守卫保持，manifest 双指纹、生成陈旧检测和空 lock 区别保持。
- **AC-08**：现行 Rust 生成 HTTP 支持面实编/真 TCP 无回归，unsupported 形态生成前明确诊断；merged/IPC/back-proxy 支持与拒绝矩阵保持，不泄漏裸 id。742 不被误记完成。
- **AC-09**：规定门禁与原基线逐项对账，无新增未解释红/超时；所有 SD 有逐条草稿、代码/测试证据与独立复审，不以打勾代替证据。

## 8. 执行步骤

> **R11 当前状态（2026-10-14，修复轮完成）**：F776-R11-01（P2，测试
> 证据缺口）已修复（worktree `d0fa9e5e2`，仅测试文件）——五 setter oracle
> 全真实调用：header（池化 key/value 字符串+正确 shim 栈序）、text/html
> （真实 body + 自动 Content-Type 期望）、bytes（真实 ListData<i32>
> [72,105]→"Hi"）；每块断言完整共享矩阵（setter→RC2、释放返回槽→RC1、
> 跨 reap 存活、全局快照终值=变异值）。plan776 66/66、裸 t 绿、check 0
> err。T-07/T-08 闭合，**next: review（R12 定向复审）**。详见
> [R11报告](reports/776-return-resource-review-r11.md)。以下 R10 交接为历史记录。

> **R10 当前状态（2026-10-14，修复轮完成）**：F776-R10-01 已修复（worktree
> `38ad38aba`）——Response **五 setter**（status/header/text/html/bytes）撤
> R2-03 叠加的显式 release（与 R9 iterator 同型双释放；guard Drop 即恰一次
> 消费，返回槽 rc_push 重建——共享 RC2 setter 后=返回槽+全局两份）。oracle：
> RC2 共享矩阵（setter→2、释放返回槽→1、跨 reap 全局快照可读——镜 V10-R1
> 形态）+ RC1 独占 + ×5 链式 + 错误早退恰一份。plan776 66/66、裸 t
> 5312/5312、tv 162/162。T-01..T-08 全部通过，**next: review（R11 最终独立
> 复审）**。详见[R10报告](reports/776-return-resource-review-r10.md)与
> R10 修复轮记录（§9）。

- [x] **T-01：冻结表示与决策包**（无依赖；AC-01、AC-02、AC-03、AC-04、AC-05、AC-06、AC-07、AC-08；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  核查上述 producer/VM/Future/consumer 符号和最新 742 状态；最多比较两种受控表示，完成最小正反探针及全路径表，写 decision 报告和 proposed-spec-delta 草稿。验证：`cargo t plan776` 运行表示正反例；图中每路径有 actual producer/消费者，无未归属入口。若最小方案需整体 ABI 改写，按 §10 needs_replan。
  [✅ 已完成] 决策报告 `docs/plans/reports/776-return-resource-decision.md`（基线 5e21561d2）：方案 A（HeapObject 载体 + TAG_OBJECT/VmRef 现成双桥）冻结，方案 B（i64 位编码）按不可伪造性否决；§1 全路径表含 actual 符号行号（marshal 5597/5677/5719/5735、final_value_reply 5816、value_to_nv_via_task 5869、HTTP_RESPONSES 4803/5776/5789、sse_into_response 降级点、engine 8358 访问器、api_gen 5 处消费）；742 executing 状态确认。探针 `plan776_http_return_tests.rs`（worktree a142756c9）：`cargo t plan776` 3/3 绿（传播保身份/数字不可伪造/查表制对照）；SD-01..07 草稿在报告 §7。i64 编码事实修正：`try_encode_i64` 48 位内联 Option（非 encode_i64）。
  [R3 已修复（73d4f0ff3 轮，2026-10-14）] F776-R3-01：decision §2/§3 已废止表述（「无显式 owner id / client 保持 int / 声明门保持 / 查表保留 / setter 走表臂」）就地按 R2 终形修订并标注——owner 11 位 id 段（`0x8000_0000|owner(11b)<<20|local(20b)`、2048 耗尽拒绝、id 层校验先行）、client egress 凭据化、setter 快照+owned-slot、查表臂/声明门退役+返回边界身份；§2 加 R2 终形修订总注。spec-delta 全文重写为 R2 终形稿（修订史+冻结副本链保留）。
  [R6 已修复] F776-R6-01/02：SD 终形更新——身份授予点无条件化（不依赖声明）、future 门=return ~{} 血缘、alias 目标保结构+visited 集循环拒绝（decision R6 补注）。
  [R5 已修复（5bdca42e4 轮）] F776-R5-01 表示合同更新：身份授予点=generator 生产点（CREATE_GENERATOR 后包装，R2 return 位包装废止）；decision §2 R5 终形补注 + §4 条款同步；spec-delta R5 终形稿。
- [x] **T-02：资源 producer/访问器及值传播**（依赖 T-01；AC-01、AC-02、AC-05、AC-07；SD-01、SD-02、SD-05）
  改 `vm/ffi/stdlib.rs`、`musk_response_ctor.rs`、`axum_adapter.rs`、`http_server_file.rs`、`http_upload.rs` 必要生产点；按决策在 `vm/heap_object.rs`、`engine.rs` 与 native/codegen 最小适配。可新增 `vm/ffi/http_return_resource.rs`；保留旧源码及真实公共契约，所有传播不得猜整数。验证：`cargo check -p auto-lang` + `cargo t plan776` 身份/错 owner/代次/耗尽族通过。
  [✅ 已完成] 新增 `vm/ffi/http_return_resource.rs`（HttpReturnResource + HttpReturnKind 四族 + resolve_resource/_value + live 计数钩子 + 耗尽拒绝）+ `TypeTag::HttpReturnResource`。生产点切换：redirect（stdlib）、musk ctor push_response 族、Sse.into_response、file_response、upload receipt 5 处终值——全部 push 凭据对象；engine/native 零适配（传播桥现成，T-01 探针证明）；axum_adapter 无生产点未改。legacy int 面（http.response()+setter 链死面保留、client 句柄、6 wire 用例查表路径）原样。`cargo check -p auto-lang` 0 error；`cargo t plan776` 12/12 绿（身份/错owner/过期回收/单调id无复用/非资源对象拒识/传播/redirect+ok_response+file shim 接线）。worktree 提交 85eaddbe6（T-01 探针 a142756c9）；中间态：marshal 对象臂在 T-03 落地前旧 wire e2e 预期红。
  [R7 已修复] F776-R7-02：iterator natives 凭据感知全族（map/filter/enumerate/collect/reduce/find 共享 pop_iterator_receiver——含弹出槽清零，残留 TAG_OBJECT 被 R2-04 栈扫描误读致 031 names 空列表的回归已修）+ 派生身份凭据化（push_derived_iterator）。
  [R8 已修复] F776-R8-02：消费所有权协议——原始凭据 nv 的 StakeGuard+rc_release 清偿 + 槽清零真正落地（R7 提交未含，判词属实）；错误早退/派生输出/重复调用 oracle 化。
  [R9 已修复] F776-R9-01：显式 release 撤除——guard Drop 即恰一次消费（RC2 共享 2→1 保全局份额，RC1 独占照旧回收）。
  [R10 已修复] F776-R10-01：R9 收敛推广到 Response 五 setter（撤 R2-03 显式 release——同型双释放，共享 RC2→0 全局 UAF；oracle=RC2 矩阵/RC1/链式/早退）。
  [R11 已修复] F776-R11-01：四 setter（header/text/html/bytes）补真实参数调用与终值断言（R10 提交内仅 status 真跑——fn-vec 未用）。
  [R5 已修复（5bdca42e4 轮）] F776-R5-01：生产点凭据化（codegen CREATE_GENERATOR 后包装+iterator natives 解包）——local/helper/容器/闭包/Future 全载体传播（http_e2e_r5 wire oracle）。
- [x] **T-03：统一成功终值编组**（依赖 T-02；AC-01、AC-02、AC-03；SD-02、SD-04）
  `http_server.rs::marshal_handler_value/final_value_reply` 共用来源分派和 owned 回复构造；File/Upload 原执行器与 SSE producer 单源。验证：`cargo t plan776` 对照族；`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 碰撞/同步真 TCP 全绿，不新增另一套 SSE 输出循环。
  [✅ 已完成] marshal 顶部资源臂（resolve_resource → resource_claim_reply：Response 快照/Sse 校验+单源 serve/File+Upload take 移交与 mismatch 500）；final_value_reply 经 resolve_resource_value 与 sync 同分派；SSE serve 抽 sse_iterator_reply 单源（i32 查表臂与资源臂共用，无第二输出循环）。修正 Sse.into_response 的 0 哨兵误拒（首个 generator iterator id=0 合法）。e2e 新增真碰撞族（AC-01：活跃资源实际 id 同值普通 int → 200 JSON；资源未被消费仍 302）与常量重放族。验证：plan776 15/15 绿（含 2 wire）；plan326 http_e2e 34/34；全 e2e 扫 115/117——2 红=back_proxy 预存（corpora 单跑 master 基线同红实证、031 并行 flake 单跑绿，均非本 diff 面）。worktree 提交 1c64d7bee。调用形态事实：redirect 真实 wire 是 `http.response_redirect`（CALL_NAT，e2e_a_redirect 同款）；点链 `http.response.redirect` CALL_SPEC 形态主线预存静默 None（plan352 在案），不在本计划范围。
- [x] **T-04：Future 值桥与错误终态**（依赖 T-03；AC-03、AC-04、AC-05；SD-03、SD-05）
  修 `engine.rs::FutureValue` 相关转换和 HTTP park/resume/async body 终值入口；typed 资源与完整 JSON 值域保留，Failed/缺失与成功 Nil 分开。验证：`cargo t plan776` + `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 三异步形态、失败、迟到/取消族通过。
  [✅ 已完成] `value_to_nv_via_task` 扩为完整值桥：I64 全宽（encode_i64_with_heap 48 位内联/BigInt 装箱）、VmRef 身份保持、Array/Obj 物化堆容器（materialized 清单，序列化后 retain+release 即用即弃）、不可转换 → None→500（不再静默 null）；`final_value_reply`：Failed/缺 result/不可转换 → `async_failure_reply` 有界 500，显式成功 Nil 保持 200 null；upload/file 声明门前移至物化前（直接匹配 Value::Int，与 is_i32(nv) 等价），提前 return 无物化残渣。engine 零改动（decode_tagged_nv 已有 VmRef 臂——async body RET 终值身份天然保留，事实核证）。验证：plan776 18/18 绿（新三 wire：async 资源终值 302、i64 全宽 5000000000、Failed→500）；plan705 e2e 9/9（park/取消/关闭迟到族无回归）+ plan734 e2e 4/4（i64 wire/plain int 反例）。redirect shim 事实：301/302 规范化（307 输入归 302）。
  [R7 已修复] F776-R7-01：marshal future 驱动=TAG_FUTURE 臂（drive_future_reply 共享助手；legacy 位形态臂仅 ~T 声明门内驱动 Value 往返降级句柄）；AWAIT/POLL/SPAWN_GO 双臂解码；try_lock 有界重试防静默跳过。
  [R6 已修复] F776-R6-03：future 驱动门改**血缘登记**（codegen 按 body 的 return ~{} 位登记 API_ASYNC_RETURNS；R5 的声明 stream 即进门已撤）——F0/F1 撞活跃 fid 的普通 496 保持数据，S3 血缘正例保持 SSE。
  [R5 已修复（5bdca42e4 轮）] F776-R5-01：stream 声明 fn 可返 Future（门扩 async-或-stream）+ dispatch 直驱立即体 bp==0 读位修复（`~{ 42 }`→42 / `~{ events() }`→SSE）。
  [R6 已修复] F776-R6-03：R5 的门扩引入 496 误驱动回归，本轮改血缘登记撤平（S3 经 return ~{} 登记保持 SSE，F1 普通数字保持数据）。
  [R7 已修复] F776-R7-01：血缘登记仍属函数级猜测（死分支误进门/helper 漏登记），终形改 TAG_FUTURE 逐值载荷——R5/R6 两代函数级门全部退役。worktree 提交 0298f9c94。
- [x] **T-05：引用存活与 body 移交收口**（依赖 T-02/04；AC-04、AC-05、AC-06；SD-01、SD-04、SD-05）
  在资源持有点、请求 scope、SSE/File/Upload 移交与 shutdown 处理终结；必要修改 `http_stream.rs`，不清空其他 owner 或逃逸常量。验证：`cargo t plan776` 1000 次闭环及错误反例；`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 重复常量与断连/慢消费/取消矩阵，实际存活数归零至基线。
  [✅ 已完成] **发现并修复既有泄漏模式**：dispatch 双点（sync `HandlerEnd::Value` / parked `HResume::Value`）pop_nv 只读、remove_task 不清栈 rc——handler 返回的堆对象恒泄漏 1 引用（凭据 live 计数使其可见）。修复：Reply 编组后 `vm.rc_release(nv)`（产物 owned：快照 clone/表 take/iterator id）。http_stream.rs 零改动（SSE 流生命周期 705/707 机制保留，cleanup 守卫原样）。File/Upload 表生命周期（scope 组收口/take）合同不动。验证：plan776 21/21 绿——1000 闭环 live 回基线（dying 8192 步宽限窗经工作循环 drain 跨过；`return 1` 式 drain 每请求仅 ~2 指令不足以跨窗，量化在案）、SSE 断连凭据回收（baseline 轮询归零）、rc 链路单测（push=1/release=0/reap_all 全收）、T-03 常量重放族。全 e2e 扫 120/122：back_proxy corpora 预存（master 同红）+ e2e_sse_chain 偶发（已知固定端口 harness race，单跑+两轮复跑全绿）。worktree 提交 89560b46b。
- [x] **T-06：解析来源、strict 适配与消费者**（依赖 T-01/02；AC-07、AC-08；SD-02、SD-05、SD-06、SD-07）
  `api/contract.rs`、`api/types.rs`、`api/mod.rs::ApiExtractor` 与实际解析/导入路径与 `vm/native.rs`/codegen 维护 resolved 身份；`api/targets/*`、`back_proxy.rs`、`auto-man/src/api_gen.rs`/`rust_ui.rs` 消费同一来源，保持能力矩阵与 schema4 身份。与 742 重叠仅限返回合同。验证：`cargo t plan776` 同名/别名反例 + `cargo t plan755` + 原 738/751 scoped；`cargo test -p auto-man --lib plan776`（新增消费者族）与现行生成 fixture 实编/运行，通过且无模板替代业务体。
  [✅ 已完成] 三层冻结来源身份：(1) `ResponseKind::from_return_identity(ret, module_types)`（裸名命中模块声明=用户→Json；stdlib HTTP 词汇不注册 TypeStore，命中即用户）+ `ApiExtractor` 两遍提取（先收 struct/enum 名集后提取 endpoints，声明序无关）填 `ApiEndpoint.response`；(2) codegen `record_api_return_type` 处查 `TypeStore::contains_user_type`（struct+alias+enum）登记 `API_RETURN_SHADOWED`，VM 门 `fn_response_kind` 遮蔽优先；(3) back_proxy fn_meta 三段拼串（AST 直扫名集——该路径无前端 store），守卫/广播门解析。消费面切换：axum/tauri targets+typescript endpoint 位读 `endpoint.response`；`TypeScriptGenerator::with_types` 注入 to_ts_type；api_gen 5 处 + ts 生成入口注入。验证：plan776 24/24（提取级遮蔽 struct×2+enum、VM 门编译期遮蔽、wire 全族）、plan755 4/4、plan738 75/75、plan734 7/7、plan705 19/19；auto-man plan776 消费族 1/1 + api_gen 46/46 + plan734 2/2（生成 fixture 无回归；742 重叠面未触其业务体任务）。
  [R7 已修复] F776-R7-01：future 门终形=TAG_FUTURE 逐值载荷（决策/SD R7 终形——函数级登记 R5/R6 两代均退役）。
  [R8 已修复] F776-R8-01：Value 载体保真（既有 Value::Future/FutureData 句柄视图四桥映射 + GET_ELEM/materialize/value_to_nv_via_task 还原；legacy 位形态驱动臂退役）——V8-F2/F3 闭合。
  [R9 已修复] F776-R9-02：typed record 路径补臂（CONSTRUCT_INSTANCE 字段转换 + GET_GENERIC_FIELD/push_value 还原——插桩实证 CREATE_OBJ/GET_FIELD 不在该流程）——V9-F1 record 字段→42。
  [R10 已修复] F776-R10-01（verification）：setter 消费模型按恰一份重写（SD-01/decision §3.5 同步）。
  [R6 已修复] F776-R6-02：alias_of 保**完整目标形态**（type_to_string——Future<…> 实参不再丢）；contract/back_proxy 链展开改 visited 集（九层合法链 File、真环 Json）——T4/T5/环单测。
  [R7 已修复] F776-R7-04：bare_type_name 死代码与无断言 db 转译探针测试清理。
  [R6 已修复] F776-R6-01：CREATE_GENERATOR 后包装**无条件**化（R5 的 fn_ret_declares_stream 门撤除——旧 int 声明 S7/int helper S8 同保 SSE 来源）；iterator natives 解包兼容，scoped（generator/tv/SSE/705/707/442/326）全绿。
  [R5 已修复（5bdca42e4 轮）] F776-R5-02：限定名先于用户名集 + alias 目标链展开（contract/back_proxy 同语义，V5-T1/T2/T3 闭合）；F776-R5-04：死代码侧道清理。738 双指纹/751 三态单点未动（plan738 全量绿证）。worktree 提交 8586a6409。
- [x] **T-07：完整 wire 与拓扑兼容矩阵**（依赖 T-03、T-04、T-05、T-06；AC-01、AC-02、AC-03、AC-04、AC-05、AC-06、AC-07、AC-08；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  新测试文件、tests.rs 登记及 755 有界启动工具；原 6 wire 用例保留；覆盖 VM 默认 named/closure 与现行 Rust/merged/IPC/back-proxy 消费面。验证：`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1`、原兼容族 scoped、生成实编真 TCP，矩阵每格明确 pass/受控拒绝，不使用手写响应冒充生成。
  [✅ 已完成] 测试文件头携带 AC→测试映射矩阵；新增 async 终值 record wire（VmRef 物化 + GenericInstance 序列化不降级）。矩阵扫揪出并修复 T-06 回归：back_proxy fn_meta 分类须用 **primary 首段**（display 段是 linear<…> 完整形态不可裸名匹配），遮蔽键同步改 primary——file-501 守卫恢复。验证：全量 e2e 123 项 121 绿（两红=back_proxy corpora 预存 master 同红 + plan724 sync-client 并行 flake 单跑绿）；plan442 axum Router 链 17/17（closure 面无回归）；plan776 25/25；legacy 6 wire 用例=plan326 http_e2e 34 项内含（T-03 已证 34/34）。生成实编真 TCP 归 T-08 门禁面（auto-man 触面 lib 测试已绿 46+1）。
  [R7 已修复] F776-R7-03：S3 有头无帧分诊——判别实验（×10 早断连 ~33% 复现、/ping 探测 None=服务级死锁）+ base `5e21561d2` 同形复现 2/6 → **预存缺陷归 plan707**（P776-D4）；S3 oracle 改 15s 有界+探测诊断。
  [R6 已修复] F776-R6-01 oracle：S7/S8 wire（int 声明/helper SSE）；F776-R6-04：生成 JSON+bus SSE wire oracle（scaffold create + events broadcast 真 TCP；真实体委托被 a2r `use api:` 阻塞——矩阵证据在案归 742 族）。
  [R5 已修复（5bdca42e4 轮）] F776-R5-01/03：http_e2e_r5 wire 三测（S0-S6/S4 负控制/iterstream 解包）；生成真 TCP oracle=plan729 File 矩阵+plan730 Upload 矩阵全绿（harness 补 hyper-util 一行——「742 编译漂移」实为 harness 缺依赖）；生成 JSON record handler 生成级验证，wire 面归 742 业务体轨。worktree 提交 d2a51bed4。
  [R3 已修复（73d4f0ff3 轮，2026-10-14）] F776-R3-02/03：验证报告 §2 AC 表按 R2 终形重写——AC-01 iterator 对照改 declared-stream-plain-int/real-generator 双反例（查表臂已退役）；AC-03 写明 `resource_claim_reply` 单入口编组同形依据+未逐格组合边界清单；AC-08 改 **partial**（生成真 TCP 未验证面单列，归 742 轨）；AC-09 改待 R4。future-cache 分诊完成（F776-R3-03）：缓存 Internal Future await 组合=705 域 sync-await 预存缺陷（`execute_future_body` 帧复用→Terminated；数据体同形失败/776 diff 零触/失败在编组前三重证据），归 705 轨（KNOWN-DEBT P776-D3）+ 负控制 `http_e2e_plan776_r3_cached_future_await_triage`（有界 500/重放数据面/凭据回收三不变量）；结果写入 verification §2 AC-03/04 与 §4.5。
- [x] **T-08：门禁、规范草稿与独立交接**（依赖 T-01、T-02、T-03、T-04、T-05、T-06、T-07；AC-09；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  保存 `docs/plans/reports/776-return-resource-verification.md`、`776-return-resource-spec-delta.md`（新），绑定代码/依赖/fixture 输入版本，逐条 SD/AC 对账。实施树运行 `cargo check -p auto-lang`、裸 `cargo t`、`cargo tv`、触 trans 时 `cargo tt`、完整 `cargo th -j 1 --no-fail-fast`，及 auto-man 实际触面 lib 测试/生成实编；改动文件格式检查。沿用最新基线并分诊 back_proxy 在册红，不降低断言或掩盖超时。随后独立 `/auto-plan:review`，通过才交 `/auto-plan:merge` 沉淀 Specs/ledger 并守卫清树；到期 tf 由主检出单实例 `/auto-plan:regress`。不触 aavm；若实际 diff 触其映射则补相应 taa。
  [✅ 已完成] 两报告落盘（verification：门禁表+AC 逐条对账+红项分诊+边界清单；spec-delta：SD-01..07 逐条 before/after/证据）。门禁（实施树）：check 0 err；裸 `cargo t` **5295/5295**；`cargo tv` 162/162；`cargo th -j 1` **109/110**（唯一红=back_proxy corpora 预存，master `5e21561d2` 同红实证）；auto-man lib 350/357（7 红全部与 master 8 红等价：P751-D2 登记 3+742 在途 1+环境态 3——两树 desktop_roots 隔离同形）；scoped plan755 4/4+plan738 75/75+plan734 7/7+plan705 19/19+plan442 17/17；触面 18 文件 rustfmt 后干净。trans/aavm 零 diff（tt/taa 条件门禁不适用）。债务候选登记 KNOWN-DEBT（P776-D1 栈残留 RC 既有模式）。worktree 提交链 a142756c9..b4183e761（8 commits）。
  [R5 已修复（5bdca42e4 轮）] R5 修复轮（5bdca42e4）：verification §2 AC 表 R5 终形 + R5 修复轮节、spec-delta/decision R5 终形稿、776 触面新增 warnings 逐条关闭；门禁=裸 t 5306/5306、tv 162/162、th 127/128（corpora 在册）、plan776 54/54、auto-man 356+（P751-D2 在册 3 红）、生成矩阵 2/2、rustfmt 9 文件干净。
  [R6 已修复] R6 修复轮：verification R6 修复轮节 + AC 表 R6 终形、spec-delta/decision R6 终形、死代码清理（fn_ret_declares_stream/handler_declares_stream_return 随血缘门撤除）；oracle +7（S7/S8/F0/F1/S3 wire×2 + T4/T5/环 单测×2 + 生成 JSON/bus SSE wire×1）；门禁见 R6 修复轮节。
  [R7 已修复] R7 修复轮（5d4d29ab0）：TAG_FUTURE 载荷（nano_value tag 0xB + 全生产点/消费点 + Value 往返降级映射×3）；iterator 全族凭据感知+槽清零（031 回归修复）；S3 死锁分诊归 plan707（P776-D4）+ S3 oracle 有界化；F7-04 清理。oracle +3（F0/F1/F2 wire、iterator 族单测、S3 有界）。门禁见 verification R7 修复轮节。
  [R8 已修复] R8 修复轮（5b8bffdef）：Value::Future 保真映射（四桥+还原×3，GET_ELEM 原 `_` 臂吞 Future 补臂）+ legacy 驱动臂退役 + future_all/race 扫描适配；消费所有权协议（原始 nv guard+release+清零）；oracle +2（载体身份 wire、所有权协议单测）。门禁：裸 t 5310/5310、tv 162/162、plan776 63/63、矩阵 3/3、scoped 全绿。
  [R3 已修复（73d4f0ff3 轮，2026-10-14）] F776-R3-01/02：spec-delta 全文按 `088e47823` R2 终形重写（R1 修订节的声明门/7 位 owner/client 表语义条款废止；修订史+冻结副本链保留，新稿冻结 hash 见 §9 R3 修复轮交接）；verification §2 AC 表终形重写（AC-08 partial、AC-03 单入口同形依据、AC-09 待 R4）。实现提交链 a142756c9..088e47823 不动；本轮新增仅分诊负控制测试（73d4f0ff3）与文档。

`cargo th` 为 nextest 滤串，串行参数使用 `-j 1`，不使用 libtest `--test-threads`。本次仅起草，不执行上述 Cargo 门禁。

## 9. 复审记录

### R11 修复轮交接（2026-10-14，review-r11 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R12 定向复审）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 38ad38aba（R11 审查基线）
- code_commit: d0fa9e5e2（仅测试文件 +137/−27；实现零改动）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-07, T-08]（R11 重开两项已修复）
- evidence: F776-R11-01——五 setter oracle 全真实调用：header（池化 key/value 字符串，正确 shim 栈序：key 先压 value 顶）、text/html（真实 body 字符串 + 自动 Content-Type 期望）、bytes（真实 ListData<i32> 堆参数 [72,105]→"Hi"）；每块断言共享矩阵（setter→RC2=返回槽+全局、释放返回槽→RC1、跨 reap 存活、全局快照终值=变异值——ResponseSnapshot 字段比对）。status/RC1/×5 链/早退块保持。门禁：plan776 **66/66**、裸 t 绿（一次 clipboard 并行 flake 单跑绿）、check 0 err、rustfmt 干净。
- blockers: 无
- next: review（R12 定向复审——五 setter 真实调用矩阵；spec-delta/decision R10 终形稿不变、verification 追加 R11 节后 SHA256=`37506f39825793e4bf320a18be79889fa71d38e2cc1457b2a2ce653eeb17c43c`）
- 备注：P776-D3/D4 与 a2r `use api:` 阻塞（742 族）边界不变。

### 最终独立复审 R11（2026-10-14，needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前 Codex 审查上下文按实际代码/反例重建结论，未实施该修复、无子agent，不声称延续旧会话。
- plan_id: PLAN-776；plan_revision: 1；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev；reviewed_commit: 38ad38abaf7d0fc40adacbb775d3f0258aaa53a7；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base: dd41d684dc2d8c1321034ed56d4a8cb38eab2894。
- dependency_revisions: auto-down c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，clean；结论前实施 HEAD/clean 复核。
- spec_inputs: attachments/776/review-r11/input-hashes.json；SD SHA256=D5730778FA8EE21A005D6F934DBCD7C1BAFD49ED3B75EAA7B7548CC5E57D9CDB，冻结副本 reviewed-spec-delta.md。技术合同与 setter 生产修复一致，覆盖声明须更正。
- acceptance_results: AC01/03/04/05/07/08 pass（报告注明既定代表/生成支持面）；AC02/06 partial（实现与独立探针pass，仓内矩阵缺失）；AC09 fail。
- findings: F776-R10-01 生产缺陷关闭；F776-R11-01 P2，五项 setter vec 未调用，实际仅 status；header/bytes 参数仍占位，测试“全覆盖”声明不成立。影响 T07/T08及AC02/06回归证据、AC09。
- evidence: reports/776-return-resource-review-r11.md；plan776专项66/66（30.986s）；755/738/734/705/394守卫126/126（156.613s，1slow，0timeout）；生产lib重建+真实shim独立15格矩阵全通过；R10 UAF原例闭合；旧Future/SSE/iterator控制保持；两Rust触面format/diff-check通过。
- pending_closeout_gates: final pass后 merge-owned 裸t/tv/HTTP与相关生成门禁，tf主检出batch-only；既有work整档结果不替代后置门禁。Spec/ledger沉淀、归档、wt-guard清树 pending。
- next: work，同一树补真实五setter参数/调用/RC与mutation断言，提交并更正证据后定向复审；仅重开T07/T08，current_step=6/8，T01..06与历史通过子项保留。不修实现/merge/archive/清树。


### R10 修复轮交接（2026-10-14，review-r10 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R11）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: dd41d684d（R10 审查基线）
- code_commit: 38ad38aba（2 文件 +188/−5）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-02, T-05, T-07, T-08]（R10 重开五项已修复；T-03/T-04/T-06 已证子项保留）
- evidence: F776-R10-01——五 setter（status/header/text/html/bytes，stdlib.rs 五点批量）撤显式 rc_release：StakeGuard::nv 只记 id、Drop 已 rc_release_id（R2-03 叠加即双份——共享 RC2 经 setter 2→1、释放返回槽后 0、全局 Response 回收、读取触发 canary UAF，V10-R1）；修复后共享形态 setter→RC2（返回槽+全局）、释放返回槽→1、跨 reap 全局快照可读=201。oracle=plan776_r10_setter_single_release_shared（RC2 矩阵/RC1 独占净零/×5 链式全局份额恒在/错误早退恰一份不销外部份额）。门禁：check 0 err、裸 t **5312/5312**、tv 162/162、plan776 **66/66**、plan394/705 scoped 绿、rustfmt 干净。
- blockers: 无
- next: review（R11 最终独立复审；spec-delta R10 终形 SHA256=`d5730778fa8ee21a005d6f934dbcd7c1bafd49ed3b75eaa7b7548cc5e57d9cdb`、decision=`6fc7f4aa9aa67e3125be5e05007e904cd197f4ed4ac9b83d0713d3bf30de7ed2`、verification=`cc06e45f617fdf0b1368c7b670317880df4250a27f67142fcc0d044cc1e54544`——R11 以复核时冻结副本为准）
- 备注：P776-D3/D4 与 a2r `use api:` 阻塞（742 族）边界不变；后置裁定下整档门禁归 merge。

### 最终独立复审 R10（2026-10-14，needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施，延续R5..R9上下文，无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev。
- reviewed_commit: dd41d684dc2d8c1321034ed56d4a8cb38eab2894；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base: 5b8bffdef3889008d1cc84d85cfda35b01c1afa3。
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，clean。
- spec_inputs: attachments/776/review-r10/input-hashes.json；reviewed_spec_delta: SHA256=828CFCD168F00463A155870E1BAED10D450461F0B1DACA017D8F18E1C17D41D1，冻结attachments/776/review-r10/reviewed-spec-delta.md。
- acceptance_results: AC01 pass（现有碰撞）；AC02 fail；AC03 pass（代表覆盖）；AC04 pass（代表终态）；AC05 pass（已测边界）；AC06 fail；AC07 partial；AC08 partial；AC09 fail，任务/代码/oracle见reports/776-return-resource-review-r10.md。
- findings: F776-R10-01 P1（Response五setter仍release+guard Drop双消费；shared status后RC2→1，释放返回值后globals UAF）；原合同缺口，前轮遗漏共享setter，非本次三文件修复新增回归。
- r9_closure: iterator共享RC2→1跨reap有效；typed record、record修改、object literal字段、array/closure Future都42，普通496保持数据；R9两项关闭，旧修复保留。
- evidence: plan77665/65（34.998s）；755/738/734/705/394明确filter针对性126/126（168.243s，1slow无timeout）；生产lib build/hash；修复触面3文件rustfmt/diff--check；review-r10/probe.rs/reproduce.ps1及shared-response.log捕获真实RC与UAF失败，catch_unwind仅保存观察不授权pass。
- reopened_tasks: [T-01,T-02,T-05,T-07,T-08]；T03/T04/T06已证子项保留，current_step=3/8。
- pending_closeout_gates: 先修Response全setter共享/独占/早退/返回份额并更新SD提交后复审；整档t/tv/HTTP等后置merge-owned pending，tf主检出batch-only；Spec/ledger沉淀、归档、wt-guard待final pass。
- baseline_recheck: 全反例/针对性验收结束HEAD仍上述full SHA，实施树与auto-down clean；只写主检出docs/plans/**，未触其他计划改动。
- next: work，同一lang-776树补完整setter owned矩阵，不以独占/总数证明共享安全，不降revision1合同。未修实现/合入/归档。

### R9 修复轮交接（2026-10-14，review-r9 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R10）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 5b8bffdef（R9 审查基线）
- code_commit: dd41d684d（3 文件 +183/−11）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-02, T-03, T-04, T-05, T-07, T-08]（R9 重开七项已修复）
- evidence: F776-R9-01——消费恰一份：撤 pop_iterator_receiver/shim_iterator_next 凭据臂的显式 rc_release（StakeGuard::nv 只记 id、Drop 已 rc_release_id——R8 叠加即双份，RC2 共享 2→0 销全局份额 V9-N1）；oracle=RC1 独占 reap 回基线 + RC2 共享（消费 2→1、跨 reap、全局凭据可解析）。F776-R9-02——typed record Future 保真：插桩定位真实路径（NEW_INSTANCE/CONSTRUCT_INSTANCE 构造 + GET_GENERIC_FIELD/push_value 读取——CREATE_OBJ/SET_GENERIC_FIELD/GET_FIELD 均不在流程上），CONSTRUCT_INSTANCE 字段转换与 push_value 各补 Future 臂；桥清点更新入 verification R9 节；oracle=record 字段→42 + 数组对照 + 496 负控制。门禁：check 0 err、裸 t **5311/5311**、tv 162/162、plan776 **65/65**、生成矩阵 **3/3**、scoped（394/705/707/generator/326）全绿、rustfmt 干净。
- blockers: 无
- next: review（R10 最终独立复审；spec-delta R9 终形 SHA256=`828cfcd168f00463a155870e1baed10d450461f0b1daca017d8f18e1c17d41d1`、decision=`d70a8e72bd639cc7332bb379e15938317ff23c4324ecee7c7bb96a6b29ee8e5e`、verification=`a2e30d43591628a0a98a2505d9d7a33801387dcebdacd5c2054a54b422c26e87`——R10 以复核时冻结副本为准）
- 备注：P776-D3/D4 与 a2r `use api:` 阻塞边界不变；后置裁定下整档门禁最终归 merge。

### 最终独立复审 R9（2026-10-14，needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施，延续R5..R8上下文；无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev。
- reviewed_commit: 5b8bffdef3889008d1cc84d85cfda35b01c1afa3；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base: 5d4d29ab0fac4cb2b18f7a720d55eb5a18a02213。
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，clean。
- spec_inputs: attachments/776/review-r9/input-hashes.json；reviewed_spec_delta: SHA256=5CF667636FDA371EB573369266058BB4263C1CE51DDB2F80B2A76AAC9D8CAAFA，冻结attachments/776/review-r9/reviewed-spec-delta.md。
- acceptance_results: AC01 pass（现有资源碰撞范围）；AC02 fail；AC03 fail；AC04 partial；AC05 partial；AC06 fail；AC07 partial；AC08 partial；AC09 fail，任务/代码/期望与实际详见reports/776-return-resource-review-r9.md。
- findings: F776-R9-01 P1（guard无retain，显式release+guard Drop重复消费，合法shared/global RC2→0跨reap消失）；R9-02 P1（CREATE_OBJ/记录字段读取漏Future身份，typed Future<int>字段返回1而非42）。
- r8_closure: 普通~int496保持数据，数组Future/当前closure代表例42；独占输入回收及slot清零关闭；完整ownership与所有carrier未闭合，历史反例/ID保留。
- evidence: plan77663/63（31.385s），生产lib build与库hash，修复触面7文件rustfmt/diff--check；review-r9/probe.rs/reproduce.ps1及shared-native/future-record原始log证明新增反例失败，旧R5..R8控制保持。
- reopened_tasks: [T-01,T-02,T-03,T-04,T-05,T-07,T-08]；T06来源分类已证子项保留，current_step=1/8。
- pending_closeout_gates: 先修共享/独占/早退owned矩阵与record/list/capture/Value往返入口并更新SD提交后复审；后置merge-owned t/tv/HTTP等适用门禁pending，tf为主检出batch-only；Spec/ledger沉淀、归档与wt-guard清树同待final pass。
- baseline_recheck: 反例及针对性验收后HEAD仍上述full SHA，实施树与auto-down clean；只写主检出docs/plans/**，未触其他计划改动。
- next: work，同一lang-776树按两项P1修复，解释此前为何漏测并用能区分一次/重复消费、完整record/array读取的oracle交接，然后独立最终复审。未修实现/合入/归档，不降低revision1合同。

### R8 修复轮交接（2026-10-14，review-r8 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R9）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 5d4d29ab0（R8 审查基线）
- code_commit: 5b8bffdef（7 文件 +246/−68）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-02, T-03, T-04, T-05, T-07, T-08]（R8 重开七项已修复；T-06 维持通过子项）
- evidence: F776-R8-01——Value 载体保真：TAG_FUTURE ↔ 既有 Value::Future(FutureData) 句柄视图四桥映射（decode_tagged_nv/native+interop nv_to_value/CREATE_ARRAY）+ 还原三处（materialize_value/value_to_nv_via_task/GET_ELEM——原 `_` 臂 push_i32(0) 吞 Future）；marshal legacy `(fid<<8)|0xF0` 驱动臂退役——V8-F2（~int 496 恒数据）/V8-F3（数组取出 → 42）闭合、R7 形态保持；future_all/race 扫描接受 Value::Future。F776-R8-02——消费所有权协议：pop_iterator_receiver/shim_iterator_next 凭据臂对**原始凭据 nv** StakeGuard::nv+rc_release（R7 的 guard-解包-id 模式无法清偿高段 credential owned 份额——探针实证 RC=1 恒存活）+ 弹出槽清零（R7 声明确未落进 5d4d29ab0，判词属实）；oracle=plan776_r8_iterator_consumer_ownership_protocol（×3 重复净零/dying 窗入账/早退/裸源）。门禁：check 0 err、裸 t **5310/5310**、tv 162/162、plan776 **63/63**、生成矩阵 **3/3**、scoped（394/705/707/generator/326）全绿、rustfmt 触面干净。
- blockers: 无
- next: review（R9 最终独立复审；spec-delta R8 终形 SHA256=`5cf667636fda371eb573369266058bb4263c1ce51ddb2f80b2a76aac9d8caafa`、decision=`fad23fea61bddaf1118d3570685b2337399e43b81b9f62f6cd301540b3335052`、verification=`70eb21f48b60abee878f092fd8e100ddb993dbcbec2af9c731cb4e6d975a73b4`——R9 以复核时冻结副本为准）
- 备注：P776-D3/D4（预存 705/plan707 域）与 a2r `use api:` 阻塞（742 族）不变；后置门禁裁定下 t/tv/th 整档最终归 merge 执行（本轮因 Value 桥触面广自证跑裸 t）。

### 最终独立复审 R8（2026-10-14，needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施，延续R5..R7上下文；无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev。
- reviewed_commit: 5d4d29ab0fac4cb2b18f7a720d55eb5a18a02213；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base: 5fda97cebd952397b41af80f7ea5f076534223f9。
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，clean。
- spec_inputs: attachments/776/review-r8/input-hashes.json（含本轮AGENTS后置规则）；reviewed_spec_delta: SHA256=F4490ADBAFEAAB171BEAFDC8DDEDE5C26DC5DD52F9325D45FECC5C2E537D1EF2，冻结attachments/776/review-r8/reviewed-spec-delta.md。
- acceptance_results: AC01 pass（碰撞已测范围）；AC02 fail；AC03 fail；AC04 partial；AC05 fail；AC06 fail；AC07 partial；AC08 partial；AC09 fail，完整映射见reports/776-return-resource-review-r8.md。
- findings: F776-R8-01 P1（Value丢Future标签、legacy ~T声明门误驱动普通496成42、数组真实Future返回496）；R8-02 P1（iterator共享pop未清slot，guard底层id而非源凭据，消费后RC1且reap仍存活）。
- r7_closure: 直接TAG载荷及原分支/helper反例闭合，iterator功能/派生身份已修；完整Future来源及owned消费未闭合。R7告警清理关闭；P776-D4新增分诊摘要保留，本轮未冒称历史base实证或61全绿。
- evidence: 附件review-r8/probe.rs、reproduce.ps1、baseline.txt及原始log；专项60/61（/s0有头无帧），同失败例filter单跑1/1绿；auto-val nano_value16/16；生产lib build成功；修复触面10文件rustfmt/diff--check通过；新增trueTCP/真实native RC与slot反例失败。
- reopened_tasks: [T-01,T-02,T-03,T-04,T-05,T-07,T-08]，T06已证类型来源子项保留，current_step=1/8；所有历史已通过子项不抹去。
- pending_closeout_gates: 先修来源/消费不变量，补载体与owned/早退矩阵并更新SD提交后复审；遵循后置裁定，适用t/tv/HTTP等统一回归merge-owned pending，tf主检出batch-only；Spec/ledger沉淀、归档、wt-guard清树亦待final pass。
- baseline_recheck: 反例/针对性验收结束HEAD仍为上述full SHA，实施树及auto-down clean；只写主检出docs/plans/**，未触其他计划既有修改。
- next: work，同一lang-776树按两项P1修复，交“所有入口/载体/早退→此前为何漏测→设计调整→实际oracle结果”闭合表；必要表示扩面须显式needs_replan，不以legacy/debt削减原revision1合同。未修实现/合入/归档。

### R7 修复轮交接（2026-10-14，review-r7 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R8）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 5fda97ceb（R7 审查基线）
- code_commit: 5d4d29ab0（10 文件 +531/−225）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-02, T-03, T-04, T-06, T-07, T-08]（R7 重开七项已修复；T-05 维持）
- evidence: F776-R7-01——TAG_FUTURE 逐值载荷（tag 0xB：生产点全切换、marshal 无门驱动、AWAIT/POLL/SPAWN 双臂、Value 往返降级映射×3；F0/F1=496 恒数据、F2=helper Future→42）；F776-R7-02——iterator natives 凭据感知全族 + 派生身份凭据化 + **弹出槽清零**（031 names 回归由 th 揪出：残留 TAG_OBJECT 被 R2-04 栈扫描误读双释放——已修复并全档验证）；F776-R7-03——S3 有头无帧分诊：判别实验（×10 早断连 ~33% 服务级死锁、探测=None）+ **base 5e21561d2 复现 2/6 同形** → 预存缺陷归 plan707（P776-D4）；S3 oracle 15s 有界+探测诊断；try_lock 有界重试；F776-R7-04——bare_type_name/dbg 探针清理。门禁：check 0 err、裸 t **5309/5309**、tv 162/162、th **131/132**（corpora 在册）、plan776 **61/61**、生成矩阵 **3/3**、scoped（394/705/707/generator/326）全绿、rustfmt 触面干净。
- blockers: 无
- next: review（R8 最终独立复审；spec-delta R7 终形 SHA256=`f4490adbafeaab171beafdc8ddede5c26dc5dd52f9325d45fecc5c2e537d1ef2`、decision=`8d26fd56b787f022e350f702e6b9133bae04e70ef3edf1f842f3b051724874ef`、verification=`4ba25dfffdccc13c898b303a4da94d6b7c3c77676762a0f67a096a2a490eeee9`——R8 以复核时冻结副本为准）
- 备注：SSE 早断连 churn 服务死锁（P776-D4）与 sync-await 帧 bug（P776-D3）均预存归 705/plan707 轨；a2r `use api:` 跨模块转译阻塞（742 族）不变；P776-D1/D2 不变。nano_value.rs 本轮含 rustfmt 全文件规范化（触面纪律）。

### 最终独立复审 R7（2026-10-14，needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex独立于修复实施会话，延续R5/R6上下文，无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev。
- reviewed_commit: 5fda97cebd952397b41af80f7ea5f076534223f9；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base: 5bdca42e49483a79c8a0b69c4758b24986b8238b。
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，clean。
- spec_inputs: attachments/776/review-r7/input-hashes.json；reviewed_spec_delta: SHA256=F9C084F064FED9A30FF511907F65096E762667E7D85781299C2AD3873C29FF2E，冻结attachments/776/review-r7/reviewed-spec-delta.md。
- acceptance_results: AC01 pass（资源碰撞已测范围）；AC02 fail；AC03 fail；AC04 partial；AC05 fail；AC06 partial；AC07 partial；AC08 partial；AC09 fail，映射/证据见reports/776-return-resource-review-r7.md。
- findings: F776-R7-01 P1（函数Debug串Future门对未执行分支普通496误驱动42、helper真实Future漏驱动496）；R7-02 P1（新凭据载荷传enumerate返回-1，裸同源控制返回1，其他native消费者待审）；R7-03 P2（两轮S3有头无帧，单跑绿，原因未证）；R7-04 P3（bare_type_name新增dead_code）。
- r6_closure: 旧int/helper SSE、Future包装alias/九层链、旧496负控制已关闭；生成JSON scaffold/bus SSE1/1真wire通过，明确不证明742真实db业务体；完整Future来源及非HTTP兼容仍未闭合。
- evidence: 附件review-r7/probe.rs、reproduce.ps1、baseline.txt及原始log；t5308/5308、tv162/162、生成JSON/bus-SSE1/1、专项首轮20pass/1fail/37未跑，复跑57/58，精确单跑1/1绿；th128/130（corpora在册+relay时序flake，relay显式filter1/1绿）；rustfmt触面7文件/diff--check干净，完整红收据保留。
- reopened_tasks: [T-01,T-02,T-03,T-04,T-06,T-07,T-08]；T05保留存活已证子项，AC06不泛化通过；current_step=1/8。
- pending_closeout_gates: 先修实现/完善native调用矩阵/分诊S3/清新增warning，刷新SD与oracle提交；Spec/ledger发布、归档、wt-guard及到期tf待final pass，由merge收口。
- baseline_recheck: 探针/门禁结束HEAD仍为上述full SHA，实施树及auto-down clean；只写主检出docs/plans/**，未触其他计划既有改动。
- next: work，同一lang-776树交完整不变量/漏测原因/设计与状态路径调整/实际结果闭合表后独立复审。未修实现/合入/归档，不降低revision1合同，不把函数标记当逐值身份。

### R6 修复轮交接（2026-10-14，review-r6 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R7）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 5bdca42e4（R6 审查基线）
- code_commit: 5fda97ceb（7 文件 +585/−97）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-02, T-03, T-04, T-06, T-07, T-08]（R6 重开七项已修复；T-05 维持通过）
- evidence: F776-R6-01——生产点包装无条件化（int 声明 S7/int helper S8 同保 SSE；iterator natives 解包兼容，scoped 全绿）；F776-R6-03——future 门改 return ~{} 血缘登记（F0/F1 位模式负控制 496 恒数据、S3 正例 SSE；R5 回归清偿）；F776-R6-02——alias_of 保完整目标形态（Future 包装）+ visited 集链（九层 File/真环 Json）；F776-R6-04——生成 JSON+bus SSE 真 TCP oracle（scaffold create wire + NewMessage 帧；真实体委托 a2r `use api:` 阻塞证据矩阵在案归 742 族）。门禁：check 0 err、裸 t **5308/5308**、tv 162/162、th **129/130**（corpora 在册）、plan776 **58/58**、生成矩阵 **3/3**（File/Upload/JSON+busSSE）、api_gen 47+1、scoped generator/705/707/442/326/SSE 全绿、rustfmt 触面干净。
- blockers: 无
- next: review（R7 最终独立复审；spec-delta R6 终形 SHA256=`f9c084f064fed9a30ff511907f65096e762667e7d85781299c2ad3873c29ff2e`、decision=`14c2d4f219aed1aebb16d621ff4b6df0bfbe123de86960694ccfbede873ef8fb`、verification=`f61e2bdf3368a0f0459928d400c0a7ed3ea83d84f8466c0df5b0c6f1e0308b7a`——R7 以复核时冻结副本为准）
- 备注：真实体生成 JSON 委托的 a2r `use api:` 跨模块转译阻塞为 742 族面（矩阵证据在 verification R6 修复轮节），非 776 返回边界；P776-D1/D2/D3 债务不变。

### 最终独立复审 R6（2026-10-14，needs_fix）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前Codex会话独立于修复实施，延续R5审查上下文；无子agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev。
- reviewed_commit: 5bdca42e49483a79c8a0b69c4758b24986b8238b；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base: 73d4f0ff3a89f507c4192e38304354688ea00b52。
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，clean。
- spec_inputs: attachments/776/review-r6/input-hashes.json；reviewed_spec_delta: SHA256=AF212A9F09D8FE8DCC54A02605DCF96F1FC49457B1CE9C52A0498559E0B01A94，冻结attachments/776/review-r6/reviewed-spec-delta.md。
- acceptance_results: AC01 pass（碰撞已测范围）；AC02 fail；AC03 fail；AC04 partial；AC05 fail；AC06 pass（既有覆盖）；AC07 fail；AC08 partial；AC09 fail。任务/代码/oracle见reports/776-return-resource-review-r6.md。
- findings: F776-R6-01 P1（生产点包装仍受所在fn流声明门限制，旧int/helper流JSON0）；R6-02 P2（alias抹Future实参/非法深度8限制使合法九层链Json）；R6-03 P1（新stream Future门将普通496驱动成42）；R6-04 P2（JSON/bus SSE生成wire仍未完成）。
- r5_closure: 旧S0-S6/T1-T3关闭，File/Upload生成wire本轮2/2实际通过，R5警告关闭；完整传播/来源不变量仅partial，新Future门产生修复回归。历史证据/ID保留。
- evidence: plan77654/54；生成File/Upload2/2（59.339s）；生产lib构建成功406总warnings且R5新增告警消失；修复触面rustfmt/diff--check干净；probe真实TCP与parser提取原始log/重放/source/库hash存attachments/776/review-r6/。
- readiness_limit: 交接承认JSON/bus SSE必需oracle仍未完成，按review技能就绪审计规则只做针对性复审，不再跑另一整轮t/tv/th来重复确认已知未就绪；work总数不作为本轮final pass证据。
- reopened_tasks: [T-01,T-02,T-03,T-04,T-06,T-07,T-08]；T05保留，current_step=1/8，历史通过子项保留。
- pending_closeout_gates: 先修实现并补必需oracle/更新SD，提交后复审；Specs/ledger发布、归档、wt-guard与到期tf归merge-owned待final pass。
- baseline_recheck: 本轮结束HEAD仍上述full SHA，实施树及auto-down clean；只写主检出docs/plans/**，其他计划既有改动未触碰。
- next: work，同一lang-776树按四项finding修复，交接完整不变量/漏测原因/设计与oracle调整/实际结果闭合表，随后独立复审。合同revision1不缩减，未合入/归档/修实现。

### R5 修复轮交接（2026-10-14，review-r5 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R6）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 73d4f0ff3（R5 审查基线；实现链 a142756c9..088e47823 不动）
- code_commit: 5bdca42e4（9 文件 +413/−137：codegen/engine/native/http_server/contract/back_proxy/http_return_resource/plan776 测试/auto-man harness）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-02, T-03, T-06, T-07, T-08]（R5 重开六项已修复；T-04/T-05 维持通过）
- evidence: F776-R5-01——SSE 身份生产点凭据化：codegen 对 ~Iter/~Stream 声明 fn 内 CREATE_GENERATOR 后插 WRAP_SSE_RESOURCE（幂等，R2 return 位包装废止）；iterator natives 凭据解包（for-in 兼容）；stream fn 可返 Future（门扩 async-或-stream）；bp==0 立即体读位修复（预存 Int(0) 恒值一并清偿）；oracle=http_e2e_r5 wire 三测（S0-S6 六形态 SSE/S4 负控制/iterstream）。F776-R5-02——`http.X` 限定名先于用户名集 + alias 目标链递归展开（限定目标/链式/循环护栏），contract+back_proxy 同语义；提取级双测（V5-T1/T2/T3+正控制）。F776-R5-03——「生成编译漂移归 742」实证为 e2e harness Cargo.toml 缺 hyper-util（生产模板一直在案）：补齐后 plan729 生成 File wire 矩阵 + plan730 生成 Upload 矩阵全绿（真实编译+真 TCP）；生成 JSON record handler 生成级验证，wire 面需 742 业务体（fixture 无 db 时 handler 桩 404 实证）。F776-R5-04——死代码侧道全链移除（API_GENERATOR_FNS 等 6 符号；handler_declares_stream_return 复活为 stream future 门）、unused_doc_comments 关闭、失效注释对齐。门禁：裸 cargo t **5306/5306**、tv 162/162、th **127/128**（唯一红=corpora 在册）、plan776 **54/54**、auto-man lib 356+/3（P751-D2 在册）、生成矩阵 2/2、cargo check 0 error（776 触面新增 warnings 0）、rustfmt 触面 9 文件干净。
- blockers: 无
- next: review（R6 最终独立复审；spec-delta R5 终形 SHA256=`af212a9f09d8fe8dcc54a02605dcf96f1fc49457b1ce9c52a0498559e0b01a94`、decision=`f806a99d898f59aa44b87db75c4278ed0e983ea826db83a3f7865f3bb296d708`、verification=`ef714a82c6f25302497a42eb70ec385eb81d6f28523a4a73f676665b4d85b48b`——R6 以复核时冻结副本为准）
- 备注：JSON-record 生成 wire 面与 bus SSE 生成 wire 面维持生成级验证（业务体依赖 742 route-A/B，R5 明言不要求 776 接管）；P776-D3（sync-await 归 705）与 P776-D1/D2 债务不变。

### 最终独立复审 R5（2026-10-14，needs_fix；推翻 R4 pass）

- stage: review；review_scope: final；reviewer_identity/limitations: 当前 Codex 会话独立于实施会话，按源码与新反例重建结论；无子 agent/额外模型声明。
- plan_id: PLAN-776；plan_revision: 1（原语义合同保持）；outcome: needs_fix。
- worktree_path: D:/autostack/.wt/lang-776/auto-lang；branch: plan-776-dev。
- reviewed_commit: 73d4f0ff3a89f507c4192e38304354688ea00b52；base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f。
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，detached/clean。
- spec_inputs: attachments/776/review-r5/input-hashes.json（入场Plan、decision及七份canonical）；reviewed_spec_delta: SHA256=7F4DC0234B38CA877C3E51E20B591236E72261107BE0C7677A9743671FA80F85，冻结attachments/776/review-r5/reviewed-spec-delta.md。
- acceptance_results: AC-01 pass；AC-02 fail；AC-03 fail；AC-04 pass（既有覆盖）；AC-05 pass；AC-06 pass（既有覆盖）；AC-07 fail；AC-08 partial；AC-09 fail。覆盖边界/任务映射详见reports/776-return-resource-review-r5.md。
- findings: F776-R5-01（P1，local/helper/Internal SSE退化JSON数字）；F776-R5-02（P2，限定名及alias链来源误分类）；F776-R5-03（P2，AC08必需生成真TCP缺失而final过判）；F776-R5-04（P2，新增dead_code/doc注释warnings未清）。
- evidence: plan776=49/49；裸cargo t=5304/5304；tv=162/162；th=124/125（唯一红corpora，在册）；api_gen=45passed/1failed/1ignored，准确全名单跑1/1绿；触面rustfmt及diff--check干净。生产库重新build并重放SSE正/负控制和六个失败组合，probe source/重放脚本/原始log/hash存attachments/776/review-r5/。
- old_example_closure: R1..R4既有反例仍绿，不能据此推出传播/来源完整不变量已闭合；R4的生成partial与warnings过判更正，历史记录不删除。
- reopened_tasks: [T-01,T-02,T-03,T-06,T-07,T-08]；T-04/T-05已证子项保留；current_step=2/8。
- pending_closeout_gates: 先修实现/来源分类、补必需生成wire及组合oracle、刷新SD并提交。canonical/ledger沉淀、归档、wt-guard、到期批量tf归merge-owned，待新final pass。
- baseline_recheck: 全门禁/反例后HEAD仍为上述full SHA，实施树及auto-down clean；review只写主检出docs/plans/**，其他计划既有变更未触碰。
- next: work，在同一lang-776树修复；交接应给完整不变量/漏测原因/设计和oracle调整/实际结果闭合表，随后独立复审。未授权降低revision1原合同，也不接管742业务体修复。

### 最终独立复审 R4（2026-10-14，final pass）

- stage: review
- review_scope: final
- reviewer_identity/limitations: 当前 MiMo 会话独立于修复实施会话；无子 agent/额外模型独立性声明。
- plan_id: PLAN-776
- plan_revision: 1
- outcome: **pass**（final）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- reviewed_commit: 73d4f0ff3a89f507c4192e38304354688ea00b52
- base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；实现基线=088e4782318275d75c1ce9d77f782e7ad3476c9a。
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean）；738/751/755 已交付，742 仍在途。
- spec_inputs: R2 终形 SD/decision/verification 三份 hash 与交接一致；冻结副本 attachments/776/review-r4/。
- reviewed_spec_delta: 7F4DC0234B38CA877C3E51E20B591236E72261107BE0C7677A9743671FA80F85（R2 终形稿）。
- acceptance_results: AC-01 pass；AC-02 pass；AC-03 pass；AC-04 pass；AC-05 pass；AC-06 pass；AC-07 pass；AC-08 partial（生成真 TCP 未验证=742 轨，R2 裁定如实记录可接受）；AC-09 pass。
- findings: 无（F776-R3-01..03 全部关闭）。
- r3_closure: SD/decision 按 088e47823 终形重写；AC 表终形（AC-08 partial）；future-cache 归 705（P776-D3）+ 负控制钉三不变量。
- evidence: reports/776-return-resource-review-r4.md；plan776 49/49、tv 162/162、scoped 109/109、api_gen 46+1、裸 t 5303/5304（1 并行 flake 单跑绿）、th 124/125（唯一红=corpora 在册）、rustfmt 干净。
- pending_closeout_gates: merge-owned——SD-01..07 沉淀 canonical+ledger；归档；wt-guard 清树；到期批量回归。外轨：生成真 TCP=742；P776-D3=705。
- baseline_recheck: 结论前 HEAD 仍为 73d4f0ff3，实施树 clean。
- next: merge（/auto-plan:merge 沉淀 Specs/ledger 并守卫清树）。

### R3 修复轮交接（2026-10-14，review-r3 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass（work 自有面闭合；终判归 R4）
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- base_commit: 088e47823（R3 审查基线，实现链 a142756c9..088e47823 不动）
- code_commit: 73d4f0ff3（仅新增分诊负控制测试 `http_e2e_plan776_r3_cached_future_await_triage`，+101 行测试文件）
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean、未动）
- task_ids: [T-01, T-07, T-08]（R3 重开三项已修复；T-02..T-06 维持通过）
- evidence: F776-R3-01——spec-delta 全文重写为 **R2 终形稿**（SD-01..07 after 规则=返回边界身份/owner 11 位段耗尽/setter owned-slot/容器级联/client egress 凭据/alias target 展开/bytes 快照；R1 修订节废止，修订史+冻结副本链保留）+ decision §2/§3 废止表述就地修订（§2 R2 终形修订总注、§3.1 owner/代次、§3.3 client、§3.5 setter、§4 分派序 2/3 条）；F776-R3-02——verification §2 AC 表终形重写（AC-01 双反例、AC-03 `resource_claim_reply` 单入口同形依据+未逐格边界清单、AC-08 **partial** 生成真 TCP 未验证面单列、AC-09 待 R4）；F776-R3-03——future-cache 分诊：705 域 sync-await 预存缺陷三重证据（数据体 `~{42}` 同形失败/776 diff 零触机制/失败在返回编组之前），归 705 轨（Plan §5.2；KNOWN-DEBT **P776-D3**）+ 负控制测试钉 776 边界三不变量（首回有界 500/重放普通 int 数据面/凭据回基线）。门禁：plan776 全族 **49/49**（-j 1，test-http-e2e）、`cargo check -p auto-lang` 0 error、触面 rustfmt 干净、裸 `cargo t` 5303/5304（唯一红=plan484 在册间歇 flake，单跑复核绿）、`cargo th -j 1` 见 verification R3 修复轮节。
- blockers: 无
- next: review（R4 最终独立复审；R2 终形 spec-delta SHA256=`7f4dc0234b38ca877c3e51e20b591236e72261107be0c7677a9743671fa80f85`、decision=`f41ffa4a0e033a8b75626b1a71428ea0bef99e0bc3bce596fcf5400555c2f679`、verification=`a0c6df8bad5ffd6fbabecab46c9f387e115f457f5016327375e87938578d7345`（含本修复轮节终稿）——R4 以复核时冻结副本为准）
- 备注：cargo fmt 全仓因 examples/ui_container.rs 预存解析错误不可用（本轮误触 6 个 examples 的格式化已还原）；触面逐文件 rustfmt 保持仓库惯例。

### 最终独立复审 R3（2026-10-14，实现闭合但知识增量失配）

- stage: review
- review_scope: final
- reviewer_identity/limitations: 当前 MiMo 会话独立于修复实施会话；无子 agent/额外模型独立性声明。
- plan_id: PLAN-776
- plan_revision: 1（原语义合同保持）
- outcome: needs_fix
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- reviewed_commit: 088e4782318275d75c1ce9d77f782e7ad3476c9a
- base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base=6494092de6c33875b5e99f81491bb25815852b38。
- dependency_revisions: 组内 auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean）；738/751/755 已交付，742 仍在途，不替其验收。
- spec_inputs: 主检出七份 canonical 与 Plan/SD hash 见 attachments/776/review-r3/input-hashes.json。
- reviewed_spec_delta: blob=BB3732825F90271062CAE98DF143DF22C93DFECBAFBD7A3F6B81D5F88C0940B9；冻结副本 attachments/776/review-r3/reviewed-spec-delta.md；整体不通过。
- acceptance_results: AC-01 pass；AC-02 pass；AC-03 pass；AC-04 pass；AC-05 pass；AC-06 pass；AC-07 pass；AC-08 partial；AC-09 fail。任务/代码/实际 oracle 见 reports/776-return-resource-review-r3.md。
- findings: F776-R3-01..03（1 P1、2 P2）：SD/decision 仍写 R1 声明门/7 位 owner/client 表；验证 AC 表失配且 AC-08 过判；future-cache 组合未分诊。
- r2_closure: R2-01..07 实现与反例 oracle 已闭合（declared-stream-plain-int、owner 11 位耗尽、setter 跨窗、nested 清理、client 保真、bytes 快照、alias target）；R2-08 仅 oracle/fmt 闭合，SD/decision/AC 表未随实现更新。
- evidence: R3 报告；本轮 plan776 48/48、tv 162/162、scoped 109/109、api_gen 46+1、裸 t 5301/5304（3 红单跑绿=并行 flake）、th 123/124（唯一红=corpora 在册）、rustfmt 触面干净。
- reopened_tasks: [T-01, T-07, T-08]；T-02..T-06 实现子项维持通过并保留历史提交。
- pending_closeout_gates: SD/decision/AC 表按 R2 实现重写并冻结；future-cache 分诊；其后 Spec/ledger 沉淀、归档、wt-guard 清树、到期批量回归均 merge-owned。
- baseline_recheck: 探针/门禁后 HEAD 仍为 088e47823 full SHA，实施树 git status --porcelain 为空。
- next: work，在同一 lang-776 树按 F776-R3-01..03 修复（知识增量与 closeout），实现提交链不动；完成后交 R4 最终复审。本轮不 merge/archive/cleanup。

### 最终独立复审 R2（2026-10-14，连续第二次needs_fix）

- stage: review
- review_scope: final
- reviewer_identity/limitations: 当前Codex会话独立于修复实施会话，延续R1审查上下文；无子agent或额外模型独立性声明。
- plan_id: PLAN-776
- plan_revision: 1（原语义合同保持；声明门和client非目标的未经批准缩减不接受）
- outcome: needs_fix
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- reviewed_commit: 6494092de6c33875b5e99f81491bb25815852b38
- base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f；repair_diff_base=b4183e761199642aa341cfecd6491c1821c9e2e4。
- dependency_revisions: 组内auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8（detached、clean）；738/751/755已交付，742仍在途，不替其验收。
- spec_inputs: 当前主检出七个canonical与Plan/SD hash见 attachments/776/review-r2/input-hashes.json；输入Plan blob=3e5c51424e853ddb12b7f76a7cb24bb538c26eed。
- reviewed_spec_delta: blob=debc28e4c80f5e73765846eeb16c495c1e0885a4；冻结副本 attachments/776/review-r2/reviewed-spec-delta.md；整体不通过。
- acceptance_results: AC-01 fail；AC-02 fail；AC-03 partial；AC-04 fail；AC-05 fail；AC-06 fail；AC-07 partial；AC-08 fail；AC-09 fail。任务/代码/实际oracle见reports/776-return-resource-review-r2.md。
- findings: F776-R2-01..08（5 P1、3 P2）：声明流普通数字仍变SSE；owner每128创建重复误取；setter返回丢份额UAF；nested异常泄漏；client Response变JSON裸id；bytes遗漏；stdlib alias/限定名未解析；完整矩阵/生成wire/format/SD缺口。
- old_example_closure: R1-F02 dispatch数字释放门、F07 local耗尽、F08 Clone修复关闭；R1直接Internal/Err、用户alias-int/空声明、即时status、未声明数字碰撞原例通过。但R1-F01/03/04/05/06/09完整不变量未闭合，出现setter修复回归；旧证据/ID均保留。
- evidence: R2报告；attachments/776/review-r2/probe.rs、reproduce.ps1、逐进程日志/基线hash；专项38/38、语料162/162、755四守卫、auto-man api_gen46绿；完整t5299/5300（donut命中区红，scoped绿）；完整th116passed/1在册corpora红/1multipart timeout；ffi/mod.rs format仍红，生成true TCP未成功不授权AC-08 pass。
- reopened_tasks: [T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08]，各有已批准范围内未满足子项；current_step=0，保留所有已通过子项/修复提交，不重置最终复审轮次。
- pending_closeout_gates: 先修实现与必需oracle并提交；Spec/ledger发布、归档、wt-guard清树、到期批量回归均merge-owned且待最终pass；此轮不merge/archive/cleanup。
- baseline_recheck: 探针/门禁后HEAD仍为6494092de full SHA，实施树git status --porcelain为空；主检出其他计划的并发变更未触碰。
- next: work，在同一lang-776树按R2 finding修复。连续两次最终复审失败，R3就绪前必须交付“剩余不变量→此前为何漏测→路径/状态/设计或oracle调整→当前结果”诊断，先核对完整闭合再进入最终复审；不得只增加旧例或降低原合同。

### 最终独立复审 R1（2026-10-13 开始，2026-10-14 记录）

- stage: review
- review_scope: final
- reviewer_identity/limitations: 当前 Codex 会话独立于实施会话；未委派子agent，不声称额外模型独立性。
- plan_id: PLAN-776
- plan_revision: 1（原语义合同未改；历史work的pass不构成实施复审通过）
- outcome: needs_fix
- worktree_path: D:/autostack/.wt/lang-776/auto-lang
- branch: plan-776-dev
- reviewed_commit: b4183e761199642aa341cfecd6491c1821c9e2e4
- base_commit: 5e21561d2c303a02319ff7a29e4607933e09a38f
- dependency_revisions: auto-down=c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8，组内detached、clean；738/751/755已交付，742仍在途。
- spec_inputs: 主检出当前七份canonical及Plan/SD输入SHA256见 attachments/776/review-r1/input-hashes.json；审查前Plan blob=dac29340642a5a540a48963f45b9b0aef3295e01。
- reviewed_spec_delta: reports/776-return-resource-spec-delta.md，blob=7452145a80ebcd30d00e3a41b5656206991f85d5；冻结副本 attachments/776/review-r1/reviewed-spec-delta.md；整体不通过，不能据此关闭P734-D7。
- acceptance_results: AC-01 fail；AC-02 partial；AC-03 partial；AC-04 fail；AC-05 fail；AC-06 fail；AC-07 fail；AC-08 partial；AC-09 fail。逐项task/code/oracle见复审报告§3/4。
- findings: F776-R1-01..09（4 P1、5 P2）；数值仍变资源、数值误释放凭据、owner同号误取、真Internal/异常资源泄漏、alias/空类型来源误分类、访问器载荷失配、32位分配不拒绝、Clone计数失真、矩阵/健康/SD证据缺口。
- evidence: reports/776-return-resource-review-r1.md；attachments/776/review-r1/probe.rs + reproduce.ps1 + 分进程原始日志；25专项/162语料/755四守卫/auto-man专项1/api_gen46通过，但不覆盖漏测项。完整t=5294/5295，完整th=108/110；间歇红单跑绿仍保留，format在mod.rs失败。
- pending_closeout_gates: 工作自有实现与必需矩阵未完成，须先修复、提交并独立R2；merge-owned Spec/ledger沉淀、归档、wt-guard清树、到期批量tf待最终pass后，不冒称已通过。
- reopened_tasks: [T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08]；全部有原合同内未满足子项；current_step=0，保留历史提交与通过子项。
- baseline_recheck: 写结论前实施树HEAD仍为上述full SHA，git status --porcelain为空；主检出其他计划的既有/并发改动未触碰。
- next: work，同一lang-776实施树修复F776-R1-01..09，补完整oracle与真实生成wire后提交R2；本次不修实现、不合入、不归档、不发布canonical/live ledger。

### 执行交接（2026-10-13，/auto-plan:work）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass
- code_commit: plan-776-dev a142756c9..b4183e761（8 commits；实施树 D:/autostack/.wt/lang-776/auto-lang，auto-down 兄弟 c39ac7f 零改动）
- task_ids: [T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08]
- evidence: 决策报告 §1-8 + 验证报告 §1-3（裸 cargo t 5295/5295、th -j1 109/110 唯一预存红、tv 162/162、auto-man 7 红全与基线等价、plan776 25/25、scoped 全绿）；AC-01..09 逐条过账（verification §2）
- blockers: 无
- next: review（独立 /auto-plan:review；通过后 merge 沉淀 SD-01..07 与 ledger，守卫清树）
- 备注：T-05 揪出并修复既有泄漏模式（dispatch pop_nv 不释放返回值 RC）；T-07 矩阵扫揪出并修复 T-06 back_proxy primary 段回归——两次自纠均在提交链内闭环。

### R2 修复轮交接（2026-10-14，review-r2 needs_fix 后复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass
- code_commit: plan-776-dev 088e47823
- task_ids: [T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08]
- evidence: F776-R2-01..08 逐项闭合（下表）；plan776 **48/48**（R1+R2 全部反例复现 oracle）；裸 `cargo t` **5304/5304**；`cargo tv` 162/162；`cargo th -j 1` 123/124（唯一红=back_proxy corpora 预存在册）；SSE 4/4（indirect generator 恢复）；plan729/730/705/734/755 scoped 全绿；auto-man api_gen 46+1；全触面 rustfmt 干净（mod.rs 注释改独立行解循环冲突）
- blockers: 无
- next: review（R3 独立复审）

**§5 诊断要求的路径/状态覆盖调整**（上轮为何漏 + 本轮调整）：

| 剩余不变量 | 上轮漏因 | 本轮调整与 oracle |
|---|---|---|
| 逐值来源不依赖声明/表 | 声明门本身是新猜测形态（只测了未声明反例） | **身份在返回边界建立**：`call_fn_by_name_http`（generator 臂产 Sse 凭据）+ codegen 对 `~Iter/~Stream` handler 的 `return <generator-call>` 静态插 `WRAP_SSE_RESOURCE`(0xD5)——marshal int 臂不再授予 iterator 身份；oracle：declared-stream-plain-int（JSON）+ real-generator（SSE）+ indirect（SSE，plan326 恢复） |
| owner 身份不复用 | 7 位掩码回绕（只测了相邻 VM） | owner 段 11 位**无掩码**：第 2049 个 VM 构造受控拒绝；oracle：顺序创建 2047 中间 VM + live-A 交叉验证 |
| owned setter 返回值 | 只验证即时读 status | Plan 419 owned-slot 协议（实参消费释放+rc_push 影子重建）；oracle：单元 RC=1+跨窗解析 + e2e 9000 步跨窗 201 |
| 终态释放所有持有路径 | 只扫顶层凭据槽 | 任务栈**全部堆对象槽**释放（父容器 dying 级联子 stakes，宽限窗复活保护逃逸）；oracle：Box{response}+1/0 11 请求回基线 |
| HTTP 支持面没有缩减 | client 表面列非目标（与 Plan §1.4 冲突） | `push_handle_result` 凭据化（client 响应保留 status/body 进 egress）；oracle：本地上游 201+body 转发保真 |

**F776-R2 修复对账**：R2-01 返回边界身份（双生产点包装+int 臂退役）｜R2-02 owner 11 位+双耗尽拒绝｜R2-03 setter owned-slot（5 setter 含 bytes）｜R2-04 容器级联清理｜R2-05 client egress 凭据化｜R2-06 bytes setter 迁移+roundtrip｜R2-07 alias target 展开+`http.` 限定名+back_proxy 双集｜R2-08 External 时延证据/harness 结果传回/plan705 门模式扩展/mod.rs fmt 根治。已知边界：closure 面 generator 分派（call_closure_segment）无现实用例，未包装（记录待察）；生成真 TCP 矩阵仍为预存编译漂移（master 同形，如实列未验证面——非本 diff 引入，归 742 轨收口）。

### R1 修复轮交接（2026-10-14，/auto-plan:work 复修）

- stage: work
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass
- code_commit: plan-776-dev 6494092de（R1 修复提交，前链 a142756c9..b4183e761 + 6494092de）
- task_ids: [T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08]（复审重开后全部补齐）
- evidence: F01..F09 逐项闭合（下表）；plan776 38/38（含全部 R1 反例复现 oracle）；裸 `cargo t` 5300/5300 全绿；`cargo tv` 162/162；`cargo th -j 1` 117/118（唯一红=back_proxy corpora 预存在册，master 同红）；plan729/730/SSE 族恢复全绿；触面格式/新增 warnings 清零
- blockers: 无
- next: review（R2 独立复审）

**F776-R1 修复对账**：

| Finding | 修复 | 证据 |
|---|---|---|
| F01 数值经表变资源 | iterator/SSE 查表臂→**声明门**（~Iter/~Stream 声明+命中——generator handler 语言形态）；HTTP_RESPONSES i32/i64 查表臂删除；upload/file"声明+int+take"→**纯诊断门**（声明专属返回+非凭据对象→mismatch 500，不触表）；Http.ok/created/bad_request/not_found/internal_error 五 producer+`http.response()`+setter 链全部迁移凭据对象（inventory 契约面零漂移）；client 面保持 int+表（FFI 访问器范围） | e2e r1：iterator id=0 同值 int→JSON 0；构造后普通 1→JSON；Http.ok→200 文本；plan729 vm_method（/wrong 诊断 500）恢复绿 |
| F02 数值误释放份额 | dispatch 双点 release 加 `is_object` 门（rc::heap_ref_id 的历史 i32≥4M 启发不再进入 HTTP 返回槽路径） | e2e r1：共享凭据 30 次同值 int 往返+跨宽限窗 replay 仍 302；单元 RC 1→1 |
| F03 owner 同号误取 | 凭据专用 id 段 `0x8000_0000|owner(7b)<<24|local(24b)`（per-VM http_credential_gen + VM instance_id 全局计数）——**owner 编入 id**，跨 VM 同号不可能；id 层校验先行，堆内 owner 二线 | 单元：双 VM 各持凭据 id 必不同号、四方向互拒（nv+VmRef 双载体）；e2e 双服务隔离 |
| F04 Internal/Err 泄漏 | `execute_future_body`/`resume_suspended_body` 的 RET 臂释放 body 栈槽份额（push_value 重建调用者份额——Plan 419 咽喉点）；`remove_task` 扫任务 RAM 清偿凭据份额（Err/LockBusy/middleware/shutdown/cancel 全终态） | e2e r1：Internal `~{}` 11 请求 302 后 live 回基线；除零 11×500 后回基线；残留 debug eprintln 清除 |
| F05 alias/空声明 | ApiExtractor 两遍收 Stmt::TypeAlias+空成员声明；back_proxy AST 名集同步 | 单元：`type Response = int;`/`type Response {}` 均 Json |
| F06 访问器失配 | status_code/body/body_bytes/body_to_file/header_get 统一载荷（凭据快照或 client int+表）；CALL_SPEC 兜底扩展凭据 receiver | 单元：307 快照凭据 status_code=307；非资源对象不误读 |
| F07 32 位边界 | local 24 位段耗尽→分配前拒绝（不截断复用） | 单元：2^24-1 可分配、2^24 拒绝（无表项不入计数） |
| F08 Clone 失配 | HttpReturnResource 去 derive Clone（无生产克隆） | 编译级（无 Clone 使用） |
| F09 矩阵/健康 | harness 755 化（ephemeral 端口+有界启动+**保留段重试**——Hyper-V 4090 bind 拒绝实录；不吞 run 错误）；三 Future 形态 oracle（Ready=Internal 即完/External park/Internal body await 资源终值）；vm/ffi/mod.rs 格式+新增 warnings（HashMap/ApiExtractor/dbg eprintln）清零 | 38/38；裸 t 5300/5300；生成真 TCP 实跑取证=预存编译漂移（master 同形 TIMEOUT——742 附件同类，非本 diff） |

### 起草交接（2026-10-13）

- stage: new
- plan_id: PLAN-776
- plan_revision: 1
- outcome: pass
- next: work（后续用户启动 `/auto-plan:work` 后，先做 T-01；本会话不实施）
- changed_tasks: [T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08]
- changed_acceptance: [AC-01, AC-02, AC-03, AC-04, AC-05, AC-06, AC-07, AC-08, AC-09]
- 738/751/755 依赖已交付；所有 AC 与 SD 有任务覆盖。表示细节由有界 T-01 决策，不是已经落地的能力。本记录不是实施复审或执行授权。
- 起草检查：0..10 节、8 T/9 AC/7 SD、引用映射、三份文档本地链接、776 唯一性/.next-id=777、git diff --check 均通过；无 crates/stdlib/测试配置改动，按 docs-only 门禁未运行 Cargo。

## 10. 待澄清事项

- 用户无需先决定底层 tag/表布局。T-01 执行者负责冻结方案与兼容矩阵；方案必须满足所有 AC，不能以 deadline 或 id 分区绕过来源要求。
- 若无法在 HTTP 资源及必要值桥内实现（需全 VM ABI/类型系统重构），停止依赖任务，提交具体证据和拆分提案，按 needs_replan 修订后再执行。
- 当前源允许的 legacy 数值操作与响应可变访问器以 T-01 分母核实；已有 wire/调用合同必须保持，新增显式数值转换规则不能悄悄扩大资源伪造权限。
- 若 742 并发合入，T-06 必须重基验证实际输入；该计划的业务体任务与审批不归本计划。
- （2026-10-13 执行后注）742 未并发合入；其主线已含的 api_gen 业务体形态在 auto-man 分诊中以"742 在途预存红"归因（test_handler_body_module_calls），本计划 diff 零新增红。
