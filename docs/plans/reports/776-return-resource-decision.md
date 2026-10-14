# PLAN-776 T-01 决策报告——HTTP 返回资源身份与终值编组（表示冻结）

> 阶段：T-01（表示与决策包冻结）｜plan_revision: 1
> 调查基线：master `5e21561d2`（worktree `D:/autostack/.wt/lang-776/auto-lang` 同基）
> 输入版本：`crates/auto-lang/src/vm/ffi/http_server.rs`（8081 行）、`stdlib.rs`（13613）、
> `musk_response_ctor.rs`（874）、`engine.rs`（13468）、`native.rs`、`heap_object.rs`（912）、
> `crates/auto-val/src/nano_value.rs`、`value.rs`、`api/contract.rs`（369）、
> `crates/auto-man/src/api_gen.rs`（8043）；canonical Specs：api-transport-contract、
> http-handler-async-lifecycle、http-stream-lifecycle、backend-assembly、vm/design/ffi、
> api-generation-integrity。742 状态：executing（主线已含其 List/companion/borrow/`?T`
> 返回修复，api_gen 业务体形态；本计划不替其验收）。

## 1. 全路径表（actual producer → 传播 → 消费者；无未归属入口）

### 1.1 Response 族

| 环节 | 现状符号（行号为基线定位） |
|---|---|
| producer（构造） | `stdlib.rs::shim_http_response`（push_i64 handle）；`shim_http_response_redirect`（push_i32 handle）；`musk_response_ctor.rs::push_response`（8 个 ctor 统一，push_i32，行 34-46） |
| producer（client 注入） | `stdlib.rs::8628-8640`（http.get/.send() 把响应数据 insert 进**调用线程**的 HTTP_RESPONSES 后返回句柄） |
| 存储 | thread_local `HTTP_RESPONSES: RefCell<HashMap<u64, HttpResponseData>>`（stdlib.rs:4803），id 源自全局 `NET_HANDLE_COUNTER` |
| 返回消费（sync） | `http_server.rs::marshal_handler_value` i32 臂 :5719 / i64 臂 :5735 `lookup_http_response`（clone 不移除 → 重复返回可行） |
| 返回消费（async） | **无**——`final_value_reply`/`value_to_nv_via_task` 不查表，int 终值直接 JSON 数字（缺口：`~Response` 异步终值降级） |
| 访问器消费 | `engine.rs:8358-8395` CALL_SPEC 兜底（`.status()/.body()/.header()`，is_i32 + 非 heap + 命中表三条件）+ stdlib.rs 6153-6300 访问器 shim 族 |

### 1.2 iterator/SSE 族

| 环节 | 现状符号 |
|---|---|
| producer（通用 iterator） | `native.rs` 5 处 / `stdlib.rs` 3 处 / `websocket.rs:203` `vm.iterators.insert` 后 `push_i32(iter_id)`（iterator 是通用 VM 机制，HTTP 仅消费者之一——**注册机制不动**） |
| producer（SSE 链） | `musk_response_ctor.rs::shim_sse_new`（Sse 装箱 `RustStdlibObject` 堆对象）→ `shim_sse_into_response` :513 **降级 `push_i32(iter_id)`**——身份在此丢失 |
| 存储 | `vm.iterators: DashMap<u32, Iterator>`（per-VM） |
| 返回消费（sync） | `marshal_handler_value` :5677 `vm.iterators.contains_key` 命中制 → SSE serve |
| 返回消费（async） | 无（同 1.1 缺口） |
| 帧生产 | `produce_sse_frames` + `sse_frame_from_nv`（SseEvent/Result.Ok 解包/raw 标量三形态） |

### 1.3 File / Upload 族（已有声明门 + take 单次移交）

| 环节 | 现状符号 |
|---|---|
| File 存储/消费 | `http_server_file.rs::VM_FILE_RESPONSES`（全局 Mutex 表）；`take_file_response`（take 移除）；sync 门 :5700 `handler_declares_file_return` + async 门 :5838 |
| Upload 存储/消费 | `http_upload.rs::VM_UPLOAD_RECEIPTS/SESSIONS/REQUESTS`（独立 id 计数 UPLOAD_ID_GEN）；`take_upload_receipt`；sync 门 :5664 + async 门 :5826 |
| 请求域收口 | `RequestScope.file_response_resources/upload_resources`（finalize 逐 id 幂等清理） |

### 1.4 Future（async 终值入口分母）

| 环节 | 现状符号 |
|---|---|
| 存储 | `vm.futures: DashMap<u32, Arc<RwLock<FutureValue>>>`；`FutureValue { state: Pending/Ready/Failed, result: Option<auto_val::Value>, kind: Internal/External, captures: HashMap<String, Value> }` |
| 编组探测 | `marshal_handler_value` :5597-5659（i32 `(bits & 0xFF) == 0xF0` 位模式 → fid；handler_async 门） |
| 三形态 | External pending → `ParkFuture`；Internal `~{}` pending → 就地驱动 `handle_await_future` / `ParkBody`；Ready → `final_value_reply(vm, ctx, result)` |
| 值桥 | `final_value_reply` :5816 → `value_to_nv_via_task` :5869（**只列 Int/Float/Double/Bool/Nil/Str；`Value::I64`/Array/Obj/VmRef 全部落 `encode_null`**）；Failed/缺 result → null（伪装成功） |
| External 完成入口 | `reg::complete_live_op`（PLAN-705 live-op 注册表，worker 写 result+state；测试可经真实 API 完成——T-01 冻结为外部完成 gate） |

### 1.5 消费者（返回分类来源）

- `api/contract.rs::ResponseKind::from_return_string`（裸名/泛型解包身份匹配）；输入串源自
  `api/mod.rs::type_to_string`（`Type::User(type_decl)` → **裸名**，无来源证明）。
- `crates/auto-man/src/api_gen.rs`（115/3346/3464-3478/3836/4769 消费同一分类）；
  back_proxy 守卫、三 targets 同源。
- legacy int wire 面（P734-D7，6 用例：302 redirect / sse chain / host forward /
  value accessors / musk response 常量等 Plan 326/346 e2e）依赖查表命中制保持。

## 2. 表示决策——两方案比较，冻结方案 A

**方案 A（冻结采用）：HeapObject 载体凭据。**
新增 `TypeTag::HttpReturnResource` + `vm/ffi/http_return_resource.rs::HttpReturnResource`
（实现 `HeapObject`），NanoValue 侧用现成 `TAG_OBJECT`(5) `encode_object(heap_id)`，
Value 侧用现成 `Value::VmRef(VmRef{id})`（`native.rs::nv_to_value` :4239 对 TAG_OBJECT
已产 VmRef；`value_to_nv` 家族对 VmRef 已回 encode_object/insert_heap_object）。

**方案 B（否决）：i64 凭据位编码**（owner|generation|id 位段 + thread_local 凭据表，
marshal 按位拆解验证）。

| 维度 | A：堆对象载体 | B：i64 位编码凭据 |
|---|---|---|
| 不可伪造（AC-01/05） | 算术只能产 TAG_I32/I64/u64 payload；TAG_OBJECT 仅由受控 encode_object 点产生；同值 int 永不命中 | 任意 i64 算术可复现位模式；owner/generation 可猜，猜中即伪造——违反"可制造普通数字的操作不能制造资源身份" |
| 传播保身份（AC-02） | 栈/globals(NanoValue)/容器/closure captures(Value)/Future result(Option<Value>::VmRef) 全现成 | 数值传播保位串，但每个消费点需重验证，交叉污染面大 |
| owner 隔离（AC-05） | heap_objects 是 per-VM 表；marshal 持 `vm: &Rc<AutoVM>` 查表=天然 owner 校验（错 owner=查无/downcast 失败→拒绝） | 需凭据内编 owner id + 全局表双轨 |
| 代次/耗尽（AC-05） | `heap_object_id_gen` 单调递增**不复用**（`insert_heap_object` engine.rs:1372 fetch_add）→ 无旧引用复活；回绕耗尽在分配点拒绝（fetch_add 回绕检测） | 需自建 generation 字段与 id 复用策略 |
| 终结/存活（AC-06） | 走现成 RC（`rc_push`/`rc_release`/dying-grace宽限窗，Plan 419/062）；引用归零即回收，Drop 钩子收口表项 | 需新建引用计数与终结协议 |
| ABI 影响 | NanoValue 零改（TAG_OBJECT 已有）；Value 零改（VmRef 已有）；新增 TypeTag 变体纯增量 | int 消费点全动 + 新凭据表协议 |
| 兼容 | legacy int 查表回退原样保留（真实资源走新臂） | 同左但 int 面全部重验 |

**结论**：A 在全部硬性维度占优且复用已有受控引用机制（计划 §2 "优先复用"）。
B 的唯一优势（不占堆 id 空间）不构成需求。**冻结方案 A。**

> **R2 终形修订（2026-10-14，F776-R3-01）**：本报告为 T-01 基线冻结稿，
> §3 下列条款已按 R1/R2 复审实现演进废止，终形以本节修订为准（实现
> `088e47823`）：① owner 不再依赖"per-VM 表即边界"——凭据 id 编入
> 11 位 owner 段（§3.1 修订）；② client Response 进 egress 边界凭据化
> （§3.3 修订）；③ setter 链迁移快照+owned-slot（§3.5 修订）；④ 查表臂
> 与声明门全部退役，iterator 身份在返回边界建立（§4 修订）。逐条 SD 终形
> 见 `776-return-resource-spec-delta.md`（R2 终形稿）。
> **R5 终形补注（2026-10-14，F776-R5-01）**：④ 再进一步——SSE 身份授予
> 点从「return 位静态包装」改为 **generator 生产点凭据化**（CREATE_GENERATOR
> 后按所在 fn 声明流返回包装；local/helper/容器/闭包/Future 全载体传播；
> for-in 迭代 natives 凭据感知解包兼容）；stream 声明 fn 可返回 Future
> （marshal future 门扩为 async-或-stream）；立即体直驱终值读位修复
> （bp==0 base 形态）。alias 解析升为**目标链**递归展开 + 限定名先于
> 用户名集（§5/F776-R5-02）。终形见 spec-delta R5 终形稿。
> **R6 终形补注（2026-10-14，F776-R6-01..03）**：④ 再收敛——生产点包装
> **无条件**化（不依赖所在 fn 声明：旧 int 声明/int helper 同保来源，
> Plan §0 兼容）；future 驱动门从「声明 stream」改为 **return ~{} 血缘
> 登记**（位模式撞活跃 fid 的普通数字不误驱动——R5 门扩展的回归修正）；
> alias 目标保**完整形态**（Future<…> 实参）+ visited 集循环拒绝（合法
> 深链不截断）。终形见 spec-delta R6 终形稿。
> **R7 终形补注（2026-10-14，F776-R7-01..04）**：future 门从 R6 的函数级
> 血缘登记改为 **TAG_FUTURE 逐值载荷**（全部生产点产 encode_future(fid)；
> 位模式撞值恒数据、helper 传递天然可驱动——函数级标记全部退役）；
> iterator natives 凭据感知全族 + 派生身份凭据化；SSE 早断连 churn
> 服务死锁经 base `5e21561d2` 判别实验定性预存（归 plan707 轨，
> P776-D4）。终形见 spec-delta R7 终形稿。

## 3. 凭据合同（方案 A 细则）

### 3.1 结构与 kind

```rust
// vm/ffi/http_return_resource.rs（T-02 新建）
pub enum HttpReturnKind {
    /// owned wire 快照（status/headers/body）——重复返回 = 每次 clone 快照
    Response(HttpResponseData),
    /// SSE 输出凭据：底层 iterator 仍在 vm.iterators（注册机制不动）
    Sse { iterator_id: u32 },
    /// 文件描述符凭据：数据仍在 VM_FILE_RESPONSES（take 单次移交不动）
    File { descriptor_id: u64 },
    /// 上传收据凭据：数据仍在 VM_UPLOAD_RECEIPTS（take 单次移交不动）
    Upload { receipt_id: u64 },
}
pub struct HttpReturnResource { pub kind: HttpReturnKind }
```

- `TypeTag::HttpReturnResource`（新变体，`name()` → `"HttpReturnResource"`）。
- 生产 push 模板 = `push_rust_obj` 同款：`vm.insert_heap_object` + `vm.rc_push(task,
  encode_object(handle))`（stdlib.rs:10521 现成模式，RC 簿记齐全）。
- **owner（R2 终形修订——原文「不再携带显式 owner id」废止）**：凭据
  走**专用 id 段** `0x8000_0000 | owner(11bit) << 20 | local(20bit)`，
  **owner 编入 id 且无掩码回绕**——id 层 owner 校验（`id_owner_matches`）
  先行于堆查找（跨 VM 同号在 id 层拒绝），堆内 owner 字段为二线；进程内
  第 2049 个 VM 起 owner 段超界 → 分配前受控拒绝（`owner space
  exhausted`，R2-02：7 位掩码 A/A+128 同段误取已修正）。凭据不走
  `heap_object_id_gen`（与通用 heap id 段 4M+ 区间不重叠）。
- **代次（R2 终形修订）**：local 段单调不复用；2^20 耗尽 → 分配前拒绝
  （诊断 `http return resource id space exhausted`），不截断重用。

### 3.2 传播与数值转换规则（身份保持分母）

| 传播形态 | 载体 | 身份 |
|---|---|---|
| 局部/helper/返回值 | 栈 NanoValue TAG_OBJECT | 保持（copy 语义 + rc stake） |
| 共享全局 | `vm.globals`（NanoValue） | 保持 |
| 容器取出 | List/Map 元素 Value::VmRef（TAG_OBJECT 入栈） | 保持 |
| closure 捕获 | `FutureValue.captures` HashMap<String, Value>::VmRef | 保持 |
| Internal/External Future 终值 | `FutureValue.result: Option<Value>::VmRef` | 保持（T-04 value_to_nv_via_task 补 VmRef 臂） |

**数值转换红线**：任何显式/算术 int 构造（`encode_i32`/`encode_i64`/i64 算术/位模式
0xF0 探测）不产生资源身份；HttpReturnResource 只能由 §3.3 producer 构造。反之，
资源对象经 print/str 拼接/算术（若源码如此写）按现有对象错误路径报错，不静默降级
为数字。

### 3.3 真实 producer 改造清单（T-02 执行）

| producer | 改造 |
|---|---|
| `musk_response_ctor::push_response`（ok/json/error/err/err_json/text/empty/to_response 8 ctor） | insert_http_response 后**改为** insert_heap_object(Response(data)) + rc_push（表 insert 同步保留供 legacy 访问器/兼容，见 §4） |
| `stdlib::shim_http_response_redirect` | 同上（302/301 wire 用例源码不变，返回值从 int 变对象——marshal 新臂按快照回复，wire 不变） |
| `stdlib::shim_http_response`（空响应构造） | 同上 |
| `musk_response_ctor::shim_sse_into_response` | `push_i32(iter_id)` → push Sse 凭据对象（iterator_id 装箱；sse chain wire 用例同上兼容） |
| File 描述符构造点（http_server_file native 面） | push File 凭据对象 |
| Upload 收据 native 面（http_upload） | push Upload 凭据对象 |
| client http.get/.send()（stdlib 8628 族） | **R2 终形修订（原「保持 int 句柄」废止，R2-05）**：`push_handle_result` 产 **Response 凭据对象**（owned 快照）——client Response 进 HTTP egress 边界（Plan §1.4），上游 status/body 保真转发，不再降级 JSON 裸编号；访问器统一载荷（快照/int 表）兼容，HTTP_RESPONSES 表 insert 退役（int 查表面成为死兼容） |

### 3.4 错凭据诊断（AC-05）

| 情形 | 行为 |
|---|---|
| TAG_OBJECT 但 downcast 非 HttpReturnResource | JSON 兜底按普通对象序列化（现状 heap_object_to_json）；若声明专属返回 → mismatch 500 |
| TAG_OBJECT 查无（跨 owner/已回收） | 受控 500 `{"error":"stale or foreign http return resource"}`（不误取其他表项——per-VM 表天然隔离） |
| 错种类（声明 File 返回 Receipt 凭据等） | declaration mismatch 500（沿用 729/730 诊断体例） |
| id 耗尽 | 分配点拒绝（§3.1） |

### 3.5 响应可变访问器与重复返回（§5.3 合同）

- Response 凭据持有 owned 快照；`response_status/header/text/html` 等可变
  shim **R2 终形修订（原「shim 改表/legacy 面走表臂」废止，R2-03/06）**：
  五 setter（含 bytes）经 `mutate_response_snapshot` 直接改**凭据内快照**，
  owned-slot 返回协议（实参份额 `rc_release`+StakeGuard 保护、返回槽
  `rc_push` 影子重建）——setter 返回值是新凭据且跨宽限窗存活（9000 步
  wire oracle）；`response_bytes` 同迁快照路径（body=AB roundtrip）。
- 重复返回：marshal 对 Response 凭据 clone 快照（不消费、不 first-return-take）——
  全局常量两次回复 ✓。Sse/File/Upload 凭据回复走 take 单次移交（原合同）；凭据
  对象本身随 RC 归零回收，Drop 时把未消费的 File/Upload 表项幂等移除（防滞留）。

## 4. legacy int 兼容与编组统一（T-03）

`marshal_handler_value` 分派序（i32 位模式 Future 探测不变，其后）：

1. **新增资源臂**：`is_object(nv)` → 查 vm 堆 → downcast HttpReturnKind：
   Response → `response_handle_reply(ctx, 快照)`；Sse → iterators 校验后 SSE serve
   （复用现 serve 路径/`produce_sse_frames` 单源）；File/Upload → take 移交 +
   mismatch 诊断。
2. **R2/R5 终形修订（原「声明门保持」废止）**：upload/file 的「声明+int」
   入口 → **纯诊断门**（声明专属返回且值非凭据对象 → mismatch 500，不触
   表）；iterator/SSE 身份在**generator 生产点**建立（R5-01：codegen 对
   `~Iter/~Stream` 声明 fn 内 CREATE_GENERATOR 后静态插 `WRAP_SSE_RESOURCE`
   ——凭据随 local/helper/容器/闭包/Future 全载体传播；迭代 natives 凭据
   感知解包）。
3. **R2/R5 终形修订（原「查表命中制原样保留」废止）**：iterator/Response
   查表臂与 `handler_declares_stream_return` 声明门全部退役——声明/表
   命中/数值都不授予身份；6 legacy wire 用例由真实生产点凭据保持 wire
   行为（plan326 34/34）；client 句柄面已凭据化（§3.3 修订）。
4. JSON 兜底（i64/普通值——普通数值永远数据）。

`final_value_reply` 统一（T-03/T-04）：
- 先经 `value_to_nv_via_task` 改造版：补 `Value::I64`（encode_i64 全宽）、
  `Value::VmRef`（资源/普通对象 identity 保持——TAG_OBJECT 回栈）、`Value::Array/Obj`
  （物化堆对象后 encode_object，nv_to_json 现成可序列化）。**不再静默 null**。
- 资源终值（VmRef → HttpReturnResource）走与 sync 同一 §4.1 分派（Response/Sse/
  File/Upload 四族 sync/async 同形）。
- Failed / result 缺失 → **有界 500**（`{"error":"async handler failed"}`；
  不再 200 null——现状缺口修正）；显式成功 Nil（Ready + result=Some(Nil)）→ 200
  `null`。取消/断连：scope 终结幂等回收，迟到完成不回包（705 机制不动），
  迟到结果所携资源凭据随任务/引用回收（T-05 收口）。

## 5. 返回类型来源（T-06 预冻结）

- `ResponseKind::from_return_string` 消费侧改造：`ApiEndpoint` 增携带
  **resolved 来源标记**（`Type::User(type_decl)` 解析路径上，stdlib 导入绑定与
  用户本地声明在 type_decl 侧可区分的现有字段/或提取期查导入表），分类输入从
  裸名串升级为 (裸名, 来源) 身份。同裸名用户类型 → Json（普通用户类型）；
  stdlib 限定名/别名按真实解析来源命中专属种类。742 重基后执行。
- manifest 双指纹（738）、755 契约覆盖/CFFI merge 守卫、751 三态单点不动；
  新返回身份（HttpReturnResource 形态）作为新适配输入参与 fingerprint。

## 6. 支持矩阵与测试 oracle（T-01 冻结）

| 族 | oracle |
|---|---|
| 真碰撞（AC-01） | 构造真实 Response/Sse 资源（经 §3.3 producer），观察实际 heap id（cfg(test) 观察钩子），再 `encode_i32(id)/encode_i64(id)` 同值普通数 → wire 200 JSON 数字；资源随后按原协议正确回复 |
| 逐值身份（AC-02） | 同 handler 条件分支（资源 vs 同值数据）；local/共享/容器/闭包/Future 传播后 downcast 不变 |
| sync/async（AC-03） | 四资源族 × {sync, Ready, External 延迟, Internal body await} × named/closure；JSON 值域 record/list/i64 全宽/str/bool/Nil 同形 |
| 终态负向（AC-04） | Failed/缺 result/失效凭据（手动 drop 模拟过期）/错种类/重复 take/迟到完成 |
| 存活（AC-06） | 全局常量连续两请求；1000 次构造-返回闭环后 heap 资源对象计数与表长度回基线（观察钩子计数，不以 RSS 单独证明） |
| 兼容/来源（AC-07/08） | 6 legacy 用例原断言；同名用户类型/别名反例；755 两守卫；738 双指纹；生成 Rust 现行能力实编真 TCP |

**外部 Future 完成入口 gate**：测试仅经真实完成 API（`reg::complete_live_op`/
register-resolve 面）完成 External future——允许设置完成源与时机，**不得**直接写
`FutureValue.result` 伪造被测编组成功。观察钩子（实际 id/存活计数）仅 `cfg(test)`。

**探针（本报告随附，`cargo t plan776`）**：
`plan776_http_return_tests.rs` 表示正反例三则——(a) 对象凭据经
insert_heap_object→encode_object→globals/futures 传播→get_heap_object downcast
身份保持；(b) 任意 encode_i32/encode_i64（含与活跃 heap id 同值）is_object 恒
false，无路径变 TAG_OBJECT；(c) 现状查表制对照（insert_http_response 命中 vs
同值 int 非对象）——冻结 A/B 差别的可执行证据。

## 7. 拟议 spec delta 草稿（SD-01..07 逐条，供 review/merge 沉淀）

- **SD-01**（add `stdlib/design/http-return-resources.md`）：§3 凭据合同全文
  （kind/owner/per-VM 表隔离/单调 id/传播表/诊断表/快照重复返回/take 单次移交/Drop
  幂等收口）。
- **SD-02**（modify api-transport-contract §3/§5）：§3 编组分派序替换为 §4.1
  （资源臂优先 + legacy 查表保留）；§5 P734-D7 关闭——保留的 6 用例改由"真实资源
  臂 + legacy int 臂"双轨覆盖；类型分类保留解析来源（§5 of 本报告）。
- **SD-03**（modify http-handler-async-lifecycle）：成功终值统一入口
  （final_value_reply 补 VmRef/I64/复杂值；Failed/缺失 → 500；成功 Nil → 200 null）。
- **SD-04**（modify http-stream-lifecycle）：SSE 返回凭据连接点（sse_into_response
  装箱、marshal Sse 臂复用 produce_sse_frames 单源）；背压/断连/期限合同不变。
- **SD-05**（modify vm/design/ffi）：HttpReturnResource 的 native 逻辑身份
  （TypeTag/HeapObject 实现/push 模板/访问器边界——CALL_SPEC 兜底维持 int 面，
  新资源访问器经对象分派）；实际载荷（各 kind 的表 backing 不动）。
- **SD-06**（modify api-generation-integrity）：返回分类 (裸名,来源) 身份输入；
  新适配输入入 fingerprint；strict 门/双指纹/三态单点不变。
- **SD-07**（modify backend-assembly + stdlib/project.md）：支持矩阵回写
  （HTTP 返回资源身份边界 + sync/async 终值同形 + 失败不再 null）。

## 8. AC 对账（本决策包覆盖映射）

| AC | 冻结内容 | 落地任务 |
|---|---|---|
| AC-01 | §2 不可伪造 + §6 真碰撞 oracle | T-02/T-03/T-07 |
| AC-02 | §3.2 传播表 + §3.3 producer 清单 + §4 legacy | T-02/T-03/T-07 |
| AC-03 | §4 final_value_reply 统一 + 值域 | T-03/T-04/T-07 |
| AC-04 | §4 终态语义（500 vs 200 null vs 取消回收） | T-04/T-05 |
| AC-05 | §3.1 owner/单调 id/耗尽 + §3.4 诊断 + Future 位模式守卫保留 | T-02/T-04 |
| AC-06 | §3.5 快照重复返回/Drop 收口 + §6 存活 oracle | T-05/T-07 |
| AC-07 | §5 来源身份 + 738/751/755 不动 | T-06 |
| AC-08 | §5/§6 生成面现行能力 + unsupported 诊断 + 742 边界 | T-06/T-07 |
| AC-09 | 本报告 + 后续 verification 报告 | T-08 |

无未归属入口：§1 表每行有 actual 符号；worker/外部完成/生成面各自挂靠。
