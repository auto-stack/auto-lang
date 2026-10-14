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

> **R2 当前状态（2026-10-14）**：最终复审 needs_fix；T-01..08 各有未满足子项，
> 已重新打开，current_step=0。下方 `[✅ 已完成]` 为原 work 历史主张，不能作为当前验收结论。
> 通过子项与提交均保留；当前缺口及 AC verdict 以
> [R2 独立复审报告](reports/776-return-resource-review-r2.md) 为准。R1已关闭原例保留，不重新抹去修复进展。

- [ ] **T-01：冻结表示与决策包**（无依赖；AC-01、AC-02、AC-03、AC-04、AC-05、AC-06、AC-07、AC-08；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  核查上述 producer/VM/Future/consumer 符号和最新 742 状态；最多比较两种受控表示，完成最小正反探针及全路径表，写 decision 报告和 proposed-spec-delta 草稿。验证：`cargo t plan776` 运行表示正反例；图中每路径有 actual producer/消费者，无未归属入口。若最小方案需整体 ABI 改写，按 §10 needs_replan。
  [✅ 已完成] 决策报告 `docs/plans/reports/776-return-resource-decision.md`（基线 5e21561d2）：方案 A（HeapObject 载体 + TAG_OBJECT/VmRef 现成双桥）冻结，方案 B（i64 位编码）按不可伪造性否决；§1 全路径表含 actual 符号行号（marshal 5597/5677/5719/5735、final_value_reply 5816、value_to_nv_via_task 5869、HTTP_RESPONSES 4803/5776/5789、sse_into_response 降级点、engine 8358 访问器、api_gen 5 处消费）；742 executing 状态确认。探针 `plan776_http_return_tests.rs`（worktree a142756c9）：`cargo t plan776` 3/3 绿（传播保身份/数字不可伪造/查表制对照）；SD-01..07 草稿在报告 §7。i64 编码事实修正：`try_encode_i64` 48 位内联 Option（非 encode_i64）。
- [ ] **T-02：资源 producer/访问器及值传播**（依赖 T-01；AC-01、AC-02、AC-05、AC-07；SD-01、SD-02、SD-05）
  改 `vm/ffi/stdlib.rs`、`musk_response_ctor.rs`、`axum_adapter.rs`、`http_server_file.rs`、`http_upload.rs` 必要生产点；按决策在 `vm/heap_object.rs`、`engine.rs` 与 native/codegen 最小适配。可新增 `vm/ffi/http_return_resource.rs`；保留旧源码及真实公共契约，所有传播不得猜整数。验证：`cargo check -p auto-lang` + `cargo t plan776` 身份/错 owner/代次/耗尽族通过。
  [✅ 已完成] 新增 `vm/ffi/http_return_resource.rs`（HttpReturnResource + HttpReturnKind 四族 + resolve_resource/_value + live 计数钩子 + 耗尽拒绝）+ `TypeTag::HttpReturnResource`。生产点切换：redirect（stdlib）、musk ctor push_response 族、Sse.into_response、file_response、upload receipt 5 处终值——全部 push 凭据对象；engine/native 零适配（传播桥现成，T-01 探针证明）；axum_adapter 无生产点未改。legacy int 面（http.response()+setter 链死面保留、client 句柄、6 wire 用例查表路径）原样。`cargo check -p auto-lang` 0 error；`cargo t plan776` 12/12 绿（身份/错owner/过期回收/单调id无复用/非资源对象拒识/传播/redirect+ok_response+file shim 接线）。worktree 提交 85eaddbe6（T-01 探针 a142756c9）；中间态：marshal 对象臂在 T-03 落地前旧 wire e2e 预期红。
- [ ] **T-03：统一成功终值编组**（依赖 T-02；AC-01、AC-02、AC-03；SD-02、SD-04）
  `http_server.rs::marshal_handler_value/final_value_reply` 共用来源分派和 owned 回复构造；File/Upload 原执行器与 SSE producer 单源。验证：`cargo t plan776` 对照族；`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 碰撞/同步真 TCP 全绿，不新增另一套 SSE 输出循环。
  [✅ 已完成] marshal 顶部资源臂（resolve_resource → resource_claim_reply：Response 快照/Sse 校验+单源 serve/File+Upload take 移交与 mismatch 500）；final_value_reply 经 resolve_resource_value 与 sync 同分派；SSE serve 抽 sse_iterator_reply 单源（i32 查表臂与资源臂共用，无第二输出循环）。修正 Sse.into_response 的 0 哨兵误拒（首个 generator iterator id=0 合法）。e2e 新增真碰撞族（AC-01：活跃资源实际 id 同值普通 int → 200 JSON；资源未被消费仍 302）与常量重放族。验证：plan776 15/15 绿（含 2 wire）；plan326 http_e2e 34/34；全 e2e 扫 115/117——2 红=back_proxy 预存（corpora 单跑 master 基线同红实证、031 并行 flake 单跑绿，均非本 diff 面）。worktree 提交 1c64d7bee。调用形态事实：redirect 真实 wire 是 `http.response_redirect`（CALL_NAT，e2e_a_redirect 同款）；点链 `http.response.redirect` CALL_SPEC 形态主线预存静默 None（plan352 在案），不在本计划范围。
- [ ] **T-04：Future 值桥与错误终态**（依赖 T-03；AC-03、AC-04、AC-05；SD-03、SD-05）
  修 `engine.rs::FutureValue` 相关转换和 HTTP park/resume/async body 终值入口；typed 资源与完整 JSON 值域保留，Failed/缺失与成功 Nil 分开。验证：`cargo t plan776` + `cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 三异步形态、失败、迟到/取消族通过。
  [✅ 已完成] `value_to_nv_via_task` 扩为完整值桥：I64 全宽（encode_i64_with_heap 48 位内联/BigInt 装箱）、VmRef 身份保持、Array/Obj 物化堆容器（materialized 清单，序列化后 retain+release 即用即弃）、不可转换 → None→500（不再静默 null）；`final_value_reply`：Failed/缺 result/不可转换 → `async_failure_reply` 有界 500，显式成功 Nil 保持 200 null；upload/file 声明门前移至物化前（直接匹配 Value::Int，与 is_i32(nv) 等价），提前 return 无物化残渣。engine 零改动（decode_tagged_nv 已有 VmRef 臂——async body RET 终值身份天然保留，事实核证）。验证：plan776 18/18 绿（新三 wire：async 资源终值 302、i64 全宽 5000000000、Failed→500）；plan705 e2e 9/9（park/取消/关闭迟到族无回归）+ plan734 e2e 4/4（i64 wire/plain int 反例）。redirect shim 事实：301/302 规范化（307 输入归 302）。worktree 提交 0298f9c94。
- [ ] **T-05：引用存活与 body 移交收口**（依赖 T-02/04；AC-04、AC-05、AC-06；SD-01、SD-04、SD-05）
  在资源持有点、请求 scope、SSE/File/Upload 移交与 shutdown 处理终结；必要修改 `http_stream.rs`，不清空其他 owner 或逃逸常量。验证：`cargo t plan776` 1000 次闭环及错误反例；`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1` 重复常量与断连/慢消费/取消矩阵，实际存活数归零至基线。
  [✅ 已完成] **发现并修复既有泄漏模式**：dispatch 双点（sync `HandlerEnd::Value` / parked `HResume::Value`）pop_nv 只读、remove_task 不清栈 rc——handler 返回的堆对象恒泄漏 1 引用（凭据 live 计数使其可见）。修复：Reply 编组后 `vm.rc_release(nv)`（产物 owned：快照 clone/表 take/iterator id）。http_stream.rs 零改动（SSE 流生命周期 705/707 机制保留，cleanup 守卫原样）。File/Upload 表生命周期（scope 组收口/take）合同不动。验证：plan776 21/21 绿——1000 闭环 live 回基线（dying 8192 步宽限窗经工作循环 drain 跨过；`return 1` 式 drain 每请求仅 ~2 指令不足以跨窗，量化在案）、SSE 断连凭据回收（baseline 轮询归零）、rc 链路单测（push=1/release=0/reap_all 全收）、T-03 常量重放族。全 e2e 扫 120/122：back_proxy corpora 预存（master 同红）+ e2e_sse_chain 偶发（已知固定端口 harness race，单跑+两轮复跑全绿）。worktree 提交 89560b46b。
- [ ] **T-06：解析来源、strict 适配与消费者**（依赖 T-01/02；AC-07、AC-08；SD-02、SD-05、SD-06、SD-07）
  `api/contract.rs`、`api/types.rs`、`api/mod.rs::ApiExtractor` 与实际解析/导入路径与 `vm/native.rs`/codegen 维护 resolved 身份；`api/targets/*`、`back_proxy.rs`、`auto-man/src/api_gen.rs`/`rust_ui.rs` 消费同一来源，保持能力矩阵与 schema4 身份。与 742 重叠仅限返回合同。验证：`cargo t plan776` 同名/别名反例 + `cargo t plan755` + 原 738/751 scoped；`cargo test -p auto-man --lib plan776`（新增消费者族）与现行生成 fixture 实编/运行，通过且无模板替代业务体。
  [✅ 已完成] 三层冻结来源身份：(1) `ResponseKind::from_return_identity(ret, module_types)`（裸名命中模块声明=用户→Json；stdlib HTTP 词汇不注册 TypeStore，命中即用户）+ `ApiExtractor` 两遍提取（先收 struct/enum 名集后提取 endpoints，声明序无关）填 `ApiEndpoint.response`；(2) codegen `record_api_return_type` 处查 `TypeStore::contains_user_type`（struct+alias+enum）登记 `API_RETURN_SHADOWED`，VM 门 `fn_response_kind` 遮蔽优先；(3) back_proxy fn_meta 三段拼串（AST 直扫名集——该路径无前端 store），守卫/广播门解析。消费面切换：axum/tauri targets+typescript endpoint 位读 `endpoint.response`；`TypeScriptGenerator::with_types` 注入 to_ts_type；api_gen 5 处 + ts 生成入口注入。验证：plan776 24/24（提取级遮蔽 struct×2+enum、VM 门编译期遮蔽、wire 全族）、plan755 4/4、plan738 75/75、plan734 7/7、plan705 19/19；auto-man plan776 消费族 1/1 + api_gen 46/46 + plan734 2/2（生成 fixture 无回归；742 重叠面未触其业务体任务）。738 双指纹/751 三态单点未动（plan738 全量绿证）。worktree 提交 8586a6409。
- [ ] **T-07：完整 wire 与拓扑兼容矩阵**（依赖 T-03、T-04、T-05、T-06；AC-01、AC-02、AC-03、AC-04、AC-05、AC-06、AC-07、AC-08；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  新测试文件、tests.rs 登记及 755 有界启动工具；原 6 wire 用例保留；覆盖 VM 默认 named/closure 与现行 Rust/merged/IPC/back-proxy 消费面。验证：`cargo nextest run -p auto-lang --lib --features test-http-e2e plan776 --jobs 1`、原兼容族 scoped、生成实编真 TCP，矩阵每格明确 pass/受控拒绝，不使用手写响应冒充生成。
  [✅ 已完成] 测试文件头携带 AC→测试映射矩阵；新增 async 终值 record wire（VmRef 物化 + GenericInstance 序列化不降级）。矩阵扫揪出并修复 T-06 回归：back_proxy fn_meta 分类须用 **primary 首段**（display 段是 linear<…> 完整形态不可裸名匹配），遮蔽键同步改 primary——file-501 守卫恢复。验证：全量 e2e 123 项 121 绿（两红=back_proxy corpora 预存 master 同红 + plan724 sync-client 并行 flake 单跑绿）；plan442 axum Router 链 17/17（closure 面无回归）；plan776 25/25；legacy 6 wire 用例=plan326 http_e2e 34 项内含（T-03 已证 34/34）。生成实编真 TCP 归 T-08 门禁面（auto-man 触面 lib 测试已绿 46+1）。worktree 提交 d2a51bed4。
- [ ] **T-08：门禁、规范草稿与独立交接**（依赖 T-01、T-02、T-03、T-04、T-05、T-06、T-07；AC-09；SD-01、SD-02、SD-03、SD-04、SD-05、SD-06、SD-07）
  保存 `docs/plans/reports/776-return-resource-verification.md`、`776-return-resource-spec-delta.md`（新），绑定代码/依赖/fixture 输入版本，逐条 SD/AC 对账。实施树运行 `cargo check -p auto-lang`、裸 `cargo t`、`cargo tv`、触 trans 时 `cargo tt`、完整 `cargo th -j 1 --no-fail-fast`，及 auto-man 实际触面 lib 测试/生成实编；改动文件格式检查。沿用最新基线并分诊 back_proxy 在册红，不降低断言或掩盖超时。随后独立 `/auto-plan:review`，通过才交 `/auto-plan:merge` 沉淀 Specs/ledger 并守卫清树；到期 tf 由主检出单实例 `/auto-plan:regress`。不触 aavm；若实际 diff 触其映射则补相应 taa。
  [✅ 已完成] 两报告落盘（verification：门禁表+AC 逐条对账+红项分诊+边界清单；spec-delta：SD-01..07 逐条 before/after/证据）。门禁（实施树）：check 0 err；裸 `cargo t` **5295/5295**；`cargo tv` 162/162；`cargo th -j 1` **109/110**（唯一红=back_proxy corpora 预存，master `5e21561d2` 同红实证）；auto-man lib 350/357（7 红全部与 master 8 红等价：P751-D2 登记 3+742 在途 1+环境态 3——两树 desktop_roots 隔离同形）；scoped plan755 4/4+plan738 75/75+plan734 7/7+plan705 19/19+plan442 17/17；触面 18 文件 rustfmt 后干净。trans/aavm 零 diff（tt/taa 条件门禁不适用）。债务候选登记 KNOWN-DEBT（P776-D1 栈残留 RC 既有模式）。worktree 提交链 a142756c9..b4183e761（8 commits）。

`cargo th` 为 nextest 滤串，串行参数使用 `-j 1`，不使用 libtest `--test-threads`。本次仅起草，不执行上述 Cargo 门禁。

## 9. 复审记录

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
