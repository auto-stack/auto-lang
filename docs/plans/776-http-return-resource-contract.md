---
plan_id: PLAN-776
status: drafting
feature_name: http-return-resource-contract
plan_revision: 1
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
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
current_step: 0
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

- [ ] **T-01：冻结表示与决策包**（无依赖；AC-01、AC-02、AC-03、AC-04、AC-05、AC-06、AC-07、AC-08；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  核查上述 producer/VM/Future/consumer 符号和最新 742 状态；最多比较两种受控表示，完成最小正反探针及全路径表，写 decision 报告和 proposed-spec-delta 草稿。验证：`cargo t plan776` 运行表示正反例；图中每路径有 actual producer/消费者，无未归属入口。若最小方案需整体 ABI 改写，按 §10 needs_replan。
- [ ] **T-02：资源 producer/访问器及值传播**（依赖 T-01；AC-01、AC-02、AC-05、AC-07；SD-01、SD-02、SD-05）
  改 `vm/ffi/stdlib.rs`、`musk_response_ctor.rs`、`axum_adapter.rs`、`http_server_file.rs`、`http_upload.rs` 必要生产点；按决策在 `vm/heap_object.rs`、`engine.rs` 与 native/codegen 最小适配。可新增 `vm/ffi/http_return_resource.rs`；保留旧源码及真实公共契约，所有传播不得猜整数。验证：`cargo check -p auto-lang` + `cargo t plan776` 身份/错 owner/代次/耗尽族通过。
- [ ] **T-03：统一成功终值编组**（依赖 T-02；AC-01、AC-02、AC-03；SD-02、SD-04）
  `http_server.rs::marshal_handler_value/final_value_reply` 共用来源分派和 owned 回复构造；File/Upload 原执行器与 SSE producer 单源。验证：`cargo t plan776` 对照族；`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 碰撞/同步真 TCP 全绿，不新增另一套 SSE 输出循环。
- [ ] **T-04：Future 值桥与错误终态**（依赖 T-03；AC-03、AC-04、AC-05；SD-03、SD-05）
  修 `engine.rs::FutureValue` 相关转换和 HTTP park/resume/async body 终值入口；typed 资源与完整 JSON 值域保留，Failed/缺失与成功 Nil 分开。验证：`cargo t plan776` + `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 三异步形态、失败、迟到/取消族通过。
- [ ] **T-05：引用存活与 body 移交收口**（依赖 T-02/04；AC-04、AC-05、AC-06；SD-01、SD-04、SD-05）
  在资源持有点、请求 scope、SSE/File/Upload 移交与 shutdown 处理终结；必要修改 `http_stream.rs`，不清空其他 owner 或逃逸常量。验证：`cargo t plan776` 1000 次闭环及错误反例；`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 重复常量与断连/慢消费/取消矩阵，实际存活数归零至基线。
- [ ] **T-06：解析来源、strict 适配与消费者**（依赖 T-01/02；AC-07、AC-08；SD-02、SD-05、SD-06、SD-07）
  `api/contract.rs`、`api/types.rs`、`api/mod.rs::ApiExtractor` 与实际解析/导入路径与 `vm/native.rs`/codegen 维护 resolved 身份；`api/targets/*`、`back_proxy.rs`、`auto-man/src/api_gen.rs`/`rust_ui.rs` 消费同一来源，保持能力矩阵与 schema4 身份。与 742 重叠仅限返回合同。验证：`cargo t plan776` 同名/别名反例 + `cargo t plan755` + 原 738/751 scoped；`cargo test -p auto-man --lib plan776`（新增消费者族）与现行生成 fixture 实编/运行，通过且无模板替代业务体。
- [ ] **T-07：完整 wire 与拓扑兼容矩阵**（依赖 T-03、T-04、T-05、T-06；AC-01、AC-02、AC-03、AC-04、AC-05、AC-06、AC-07、AC-08；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  新测试文件、tests.rs 登记及 755 有界启动工具；原 6 wire 用例保留；覆盖 VM 默认 named/closure 与现行 Rust/merged/IPC/back-proxy 消费面。验证：`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1`、原兼容族 scoped、生成实编真 TCP，矩阵每格明确 pass/受控拒绝，不使用手写响应冒充生成。
- [ ] **T-08：门禁、规范草稿与独立交接**（依赖 T-01、T-02、T-03、T-04、T-05、T-06、T-07；AC-09；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  保存 `docs/plans/reports/776-return-resource-verification.md`、`776-return-resource-spec-delta.md`（新），绑定代码/依赖/fixture 输入版本，逐条 SD/AC 对账。实施树运行 `cargo check -p auto-lang`、裸 `cargo t`、`cargo tv`、触 trans 时 `cargo tt`、完整 `cargo th -j 1 --no-fail-fast`，及 auto-man 实际触面 lib 测试/生成实编；改动文件格式检查。沿用最新基线并分诊 back_proxy 在册红，不降低断言或掩盖超时。随后独立 `/auto-plan:review`，通过才交 `/auto-plan:merge` 沉淀 Specs/ledger 并守卫清树；到期 tf 由主检出单实例 `/auto-plan:regress`。不触 aavm；若实际 diff 触其映射则补相应 taa。

`cargo th` 为 nextest 滤串，串行参数使用 `-j 1`，不使用 libtest `--test-threads`。本次仅起草，不执行上述 Cargo 门禁。

## 9. 复审记录

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
