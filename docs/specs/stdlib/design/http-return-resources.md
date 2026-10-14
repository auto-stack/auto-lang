# HTTP 返回资源凭据——按值身份、生产点授予与消费协议（单源）

> 路径：`crates/auto-lang/src/vm/ffi/http_return_resource.rs`（凭据类型/解析/计数）+
> `vm/ffi/http_server.rs`（marshal 编组）+ `vm/codegen.rs`（生产点包装）+
> `vm/native.rs`（迭代消费面）
> 状态：active（PLAN-776 交付；决策/验证/复审链见
> `docs/plans/reports/776-return-resource-{decision,verification}.md`，R1..R14 复审史在
> 计划文件 §9）

## 1. 凭据表示

- HTTP 返回资源 = `heap_objects` 表 entry（`TypeTag::HttpReturnResource`）；
  `HttpReturnKind` 四族：`Response(HttpResponseData)`（owned 快照）、
  `Sse { iterator_id }`、`File { descriptor_id }`、`Upload { receipt_id }`。
  NanoValue 侧 `TAG_OBJECT`（`encode_object`），Value 侧 `Value::VmRef`。
- **凭据专用 id 段**：`0x8000_0000 | owner(11bit) << 20 | local(20bit)`
  （与通用 heap id 段 4M+ 增长区间不重叠；凭据不走 `heap_object_id_gen`）。
  **owner 编入 id 且无掩码回绕**：进程内第 2049 个 VM 起 owner 段超界 →
  分配前受控拒绝（`owner space exhausted`）；id 层 owner 校验
  （`id_owner_matches`）先行于堆查找，堆内 owner 字段为二线；local 段
  2^20 耗尽同样分配前拒绝（不截断复用使旧引用复活）。
- 普通 i32/i64（TAG_I32/I64）永不持有资源身份——任何数值运算不可制造
  TAG_OBJECT；`HttpReturnResource` 无 Clone。

## 2. 身份授予点（生产点，声明无关）

- **generator/SSE**：codegen 对 generator fn 调用（CREATE_GENERATOR 后）
  **无条件**插 `WRAP_SSE_RESOURCE`(0xD5)（幂等：已是凭据对象原样回推）；
  generator handler 本体的 dispatch 臂（`call_fn_by_name_http`）与 musk
  `Sse.into_response`（Sse 堆对象 → `push_sse_resource`）同产凭据。
  **声明/表命中/数值全部不构成来源**——旧 `int` 声明 handler 与 `int`
  helper 返回真实 generator 同保 SSE 来源（源码兼容）；声明只用于编组
  校验/能力选择。`return 0` 等普通数值恒数据。
- **Response/File/Upload**：构造点（redirect/musk ctor 8 族/Http.ok 五族/
  `http.response()`+setter 链/Sse.into_response/file/upload）push 凭据对象；
  client Response egress（`push_handle_result`）同产凭据（上游
  status/body 保真转发，不降级 JSON 裸编号）。
- **查表命中制全部退役**（iterator i32 查表、HTTP_RESPONSES 查表、
  upload/file「声明+int+take」→ 纯诊断门 mismatch 500）——P734-D7 查表
  猜测正式替代；legacy 6 wire 用例由真实生产点凭据保持 wire 行为。

## 3. 传播与 Value 载体保真

- 五载体保身份：栈 NanoValue、globals、容器元素、closure captures、
  Future result（VmRef/TAG_OBJECT 双桥）。
- **Future 句柄 = TAG_FUTURE 逐值载荷**（nanobox tag 0xB，
  `encode_future(fid)`）：全部生产点（CREATE_FUTURE 与 External creator）
  产此载荷；marshal 对 TAG_FUTURE 值直接驱动（`drive_future_reply`，
  无函数级门）。Value 载体（容器元素/闭包捕获/异步终值/GET_ELEM/
  typed record 字段——CONSTRUCT_INSTANCE 与 GET_GENERIC_FIELD/
  push_value）保真映射既有 `Value::Future(FutureData{id})` 句柄视图、
  还原回 `encode_future`——数组/record 字段取出、捕获读回的真实 Future
  保持可驱动；**legacy `(fid<<8)|0xF0` 位形态驱动臂退役**——普通 i32
  （位模式撞活跃 fid，如 496）恒数据；AWAIT/POLL 保留位形态兼容解码
  （功能面，无驱动门）；`future_all/race` 列表扫描接受 `Value::Future`。

## 4. 编组与消费协议

- **编组序**：marshal 资源臂（is_object → id 层 owner 校验 → downcast）
  优先于一切 int 探测；Response=快照 clone（重复返回不消费）；Sse=
  iterators 注册表校验后 `sse_iterator_reply` 单源 serve；File/Upload=
  take 单次移交 + mismatch 500；sync `marshal_handler_value` 与 async
  `final_value_reply`（`resolve_resource_value`）汇于同一分派
  （`resource_claim_reply`——同形由构造保证）。
- **setter 恰一份消费**（status/header/text/html/bytes 五 setter 经
  `mutate_response_snapshot` 改凭据内快照）：**StakeGuard Drop 即那一次
  消费**（guard 只记 id 不 retain，Drop 执行 rc_release_id——不得叠加
  显式 rc_release，否则共享 RC2 双释放销毁全局份额）；返回槽 `rc_push`
  影子重建——共享形态 setter 后 RC2=返回槽+全局；错误早退同样恰一份。
- **iterator 消费面凭据感知全族**：`auto.iterator.next/map/filter/
  enumerate/collect/reduce/find` 共享凭据感知弹栈（裸 id 与 Sse 凭据同
  底层数据；弹出槽清零——残留 TAG_OBJECT 会被任务栈扫描误读）+ 派生
  iterator 身份凭据化（凭据源派生 id 同样包装）。
- **异步终值**：完整值域（i64 全宽 48 位内联/BigInt 装箱、VmRef 身份
  保持、Array/Obj 物化容器即用即弃）；Failed/缺 result/不可转换 → 有界
  500（`async_failure_reply`）；显式成功 Nil → 200 null；立即完成
  Internal 体的 dispatch 直驱终值从栈顶正确读取（bp==0 base 形态）。

## 5. 生命周期

- 凭据随 RC 归零回收（dying 宽限窗）；dispatch 双点释放 handler 返回值
  槽（sync `HandlerEnd::Value` / parked `HResume::Value`）；任务终态
  `release_task_credentials` 扫任务栈**全部堆对象槽**——父容器
  （record/list/closure）份额释放并 dying 级联子 stakes，逃逸/共享经
  宽限窗复活保护；File/Upload 表生命周期归 scope 组收口（凭据 Drop
  不触表）。

## 6. 返回类型来源分类（与 api-transport-contract 联动）

- `from_return_identity(ret, module_types)`：`http.` 限定名=stdlib 绑定，
  先于用户名集直派；alias 目标**完整形态**（`type D = Future<FileResponse>`
  按 File）+ **visited 集**链展开（合法深链不限层；真环保守 Json）；
  提取器/back_proxy fn_meta 同语义（见
  [api-transport-contract §1/§5](api-transport-contract.md)）。

## 7. 观察与边界

- `live_resource_count`/`probe_last_resource_handle` 仅 cfg(test) 读。
- 边界（另域债务，非本契约缺口）：sync-await 帧缺陷与 rapid SSE 早断连
  churn 服务死锁归 plan705/707 轨（KNOWN-DEBT P776-D3/D4）；a2r
  `use api:` 跨模块转译阻塞生成真实体 JSON 委托（742 族）。

## 关联

- [api-transport-contract](api-transport-contract.md)（分类/编组序/wire 裁定）
- [http-handler-async-lifecycle](http-handler-async-lifecycle.md)（异步终值）
- [http-stream-lifecycle](http-stream-lifecycle.md)（SSE 流生命周期）
- [vm ffi](../../auto-lang/vm/design/ffi.md)（native 身份/消费协议）
