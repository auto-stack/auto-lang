# PLAN-776 拟议规范增量（SD-01..07，R2 终形）

> 供 review/merge 按实证沉淀（本次不修改 canonical Specs）。
> 每条 SD：before（现行规则）→ after（本计划落地后的规则）→ 代码/测试证据。
> **本稿为 R2 终形**（2026-10-14，实现基线 `088e47823` + R3 分诊测试
> `73d4f0ff3`）：R1 修订中的「iterator 声明门」「owner 7 位段」「client
> 保持 int 表语义」「setter 链保留表臂」条款已按 R2 实现废止并重写
> （F776-R3-01）。
> 执行细节与门禁见 `776-return-resource-verification.md`；表示决策见
> `776-return-resource-decision.md`（§2/§3 已按 R2 终形修订）。
> **修订史**：初稿（T-08，b4183e761）→ R1 修订（冻结副本
> attachments/776/review-r1/reviewed-spec-delta.md）→ R2 审查版（冻结副本
> attachments/776/review-r2/reviewed-spec-delta.md，R3 审查输入
> attachments/776/review-r3/reviewed-spec-delta.md）→ **R2 终形（本稿）**。

## SD-01（add）`docs/specs/stdlib/design/http-return-resources.md`

**新增全文合同**（R2 终形要点）：

- **凭据表示**：HTTP 返回资源 = `heap_objects` 表 entry
  （`TypeTag::HttpReturnResource`；`HttpReturnKind`：Response owned 快照
  （`ResponseSnapshot{status,headers,body}`）/ Sse{iterator_id} /
  File{descriptor_id} / Upload{receipt_id}）。NanoValue 侧 `TAG_OBJECT`，
  Value 侧 `Value::VmRef`——普通 i32/i64（TAG_I32/I64）永不持有资源身份；
  任何数值运算不可制造 TAG_OBJECT；`HttpReturnResource` 无 Clone。
- **凭据专用 id 段（R2 终形，替代 R1 的 7 位段）**：
  `0x8000_0000 | owner(11bit) << 20 | local(20bit)`（u32 载荷内，与通用
  heap id 段 4M+ 增长区间不重叠；凭据不走 `heap_object_id_gen`）。
  **owner 编入 id 且无掩码回绕**：进程内创建第 2049 个 VM 起 owner 段超界
  → 分配前受控拒绝（`owner space exhausted`）；local 段 2^20 耗尽同样分配
  前拒绝（不截断复用使旧引用复活）。**id 层 owner 校验先行**于堆查找
  （`id_owner_matches`——跨 VM 同号在 id 层拒绝），堆内 owner 字段为二线
  （R2-02 反例：7 位掩码下 A 与 A+128 同段误取，已废止）。
- **身份授予点（R2 终形，替代 R1 的声明门）**：身份只在**返回边界/构造点**
  建立。iterator/SSE 三个生产点：`call_fn_by_name_http` 的
  handler-是-generator dispatch 臂；codegen 对 `~Iter/~Stream` 声明 handler
  的 `return <generator fn 调用>` 静态插 `WRAP_SSE_RESOURCE`(0xD5)
  （iterator 注册表校验后装箱）；musk `Sse.into_response`（Sse 堆对象 →
  `push_sse_resource`）。**声明/表命中/数值全部不构成来源**——marshal int
  臂不再授予 iterator 身份。Response/File/Upload 由构造点 push 凭据。
- **传播**：栈/globals/容器/closure captures/Future result 五载体身份保持
  （VmRef/TAG_OBJECT 双桥）；资源不可经数值转换获得。
- **编组**：marshal 资源臂优先于一切 int 探测；Response=快照 clone（重复
  返回不消费）；Sse=iterators 注册表校验后走 `sse_iterator_reply` 单源
  serve；File/Upload=take 单次移交+mismatch 500。
- **setter 终形（R2-03/06，替代 R1 的「setter 链保留表臂」）**：
  status/header/text/html/bytes 五 setter 经 `mutate_response_snapshot`
  改凭据内快照，owned-slot 返回协议（实参份额 `rc_release`+StakeGuard
  保护、返回槽 `rc_push` 影子重建）——setter 返回值跨 8192 步宽限窗存活
  （9000 步 wire oracle 锁 UAF 修复）。
- **生命周期（R2 终形）**：凭据随 RC 归零回收（dying 宽限窗）；dispatch
  双点释放 handler 返回值槽（sync `HandlerEnd::Value`/parked
  `HResume::Value`）；任务终态 `release_task_credentials` 扫任务栈**全部
  堆对象槽**——父容器（record/list/closure）份额释放并 dying 级联子
  stakes，逃逸/共享经宽限窗复活保护（R2-04：`Box{response}` 异常 11 请求
  后回基线）；File/Upload 表生命周期归 scope 组收口（凭据 Drop 不触表）。
- **观察**：`live_resource_count`/`probe_last_resource_handle` 仅
  cfg(test) 读（计数器生产写、无伪造能力）。

证据：`vm/ffi/http_return_resource.rs`；plan776 全族 **49/49**（含 owner
11 位耗尽、setter 跨窗、nested 清理、bytes roundtrip、R3 分诊负控制）。

## SD-02（modify）`docs/specs/stdlib/design/api-transport-contract.md` §3/§5

- before §3：编组分派=Upload(declared)→iterator/SSE(注册表命中)→
  File(declared)→Response handle(注册表命中)→JSON；普通 int 保护=id
  空间分离+e2e。
- after §3（R2 终形）：**查表臂全部退役；声明门废止——声明不授予身份**
  （R2-01：`handler_declares_stream_return` 查表臂成死代码；HTTP_RESPONSES
  查表删除）。分派序=资源臂（is_object→id 层 owner 校验→downcast）置于
  最前；generator/SSE 身份由 SD-01 生产点包装；upload/file「声明专属返回+
  值非凭据对象」→纯诊断门 mismatch 500（不触表）；**未声明专属返回的普通
  数值永远 JSON 数据**（R2 反例 `fn p() ~Iter { return 0 }` → JSON 0）。
- before §5（P734-D7）：6 wire 用例依赖查表命中制。
- after §5：**P734-D7 关闭**——legacy 6 wire 用例 wire 行为不变由**真实
  生产点凭据**保持（redirect/musk ctor 8 族/Http.ok 五族/
  `http.response()`+setter 链/Sse.into_response/file/upload 构造点全部产
  凭据对象；plan326 http_e2e 34/34、碰撞族、常量重放族）。
- §1 身份分类：`from_return_identity(ret, module_types)`——alias 按 target
  展开、`http.` 限定名直接按 stdlib 分派（见 SD-06）；`from_return_string`
  降为无模块上下文保守入口。

## SD-03（modify）`docs/specs/stdlib/design/http-handler-async-lifecycle.md`

- before：Phase A——Failed/缺 result → 200 null；值域窄（Int/Float/Bool/
  Nil/Str，其他降级 null）。
- after：**终值统一**——`final_value_reply` 资源终值与 sync 同一分派
  （`resolve_resource_value`）；完整值域（i64 全宽 48 位内联/BigInt 装箱、
  VmRef 身份保持、Array/Obj 物化容器即用即弃）；Failed/缺 result/不可转换
  → 有界 500（`async_failure_reply`）；显式成功 Nil → 200 null；取消/
  断连迟到值不回包（705 机制）所携凭据随 RC 回收。
- **边界注记（R3 分诊，F776-R3-03）**：Future 的展开/等待归 705 状态机
  （Plan §5.2）——缓存 Internal Future 的 `.await` 组合（全局缓存 future
  再 await/重放）属 705 域 sync-await 预存缺陷（`execute_future_body`
  帧复用 → RET bp==0 Terminated → 有界 500；数据体同形失败、776 diff 零触
  该机制），不在本 SD 终值编组边界内。776 边界在该缺陷下的不变量由负控制
  `http_e2e_plan776_r3_cached_future_await_triage` 钉住：首回有界 500、
  重放普通 int 数据面（无身份泄漏）、半驱动凭据经任务终态清偿回基线。
  归属 705 轨（KNOWN-DEBT P776-D3）；705 修复后断言翻转即移交。

证据：e2e async_terminal_resource/i64_full_width/failed_is_500/record_json；
plan705 19/19；R3 分诊负控制 1/1。

## SD-04（modify）`docs/specs/stdlib/design/http-stream-lifecycle.md`

- 增连接点：SSE 身份三个生产点（SD-01）——musk `Sse.into_response` 装箱
  凭据（iterator_id；**0 是合法 id**——首个 generator）；codegen 对
  `~Iter/~Stream` handler 的 `return <generator 调用>` 静态插
  `WRAP_SSE_RESOURCE`（编译期静态证明的 generator 产出——普通数值/非
  generator 调用不包）；`call_fn_by_name_http` generator dispatch 臂。
  marshal Sse 臂校验 iterators 注册表后走 `sse_iterator_reply` **单源**
  （与 legacy 命中臂共用 producer spawn/`produce_sse_frames`/cleanup 守卫
  ——无第二输出循环）。
- 取消/背压/断连/半关闭合同不变（plan707/705 族绿；indirect generator
  形态恢复，SSE 族 4/4）。

## SD-05（modify）`docs/specs/auto-lang/vm/design/ffi.md`

- HTTP 资源 native 逻辑身份：`HttpReturnResource` 的 HeapObject 实现与
  `push_kind` 模板（`credential_id` 编 owner 段 + `heap_objects.insert` +
  `rc_push`，`push_rust_obj` 同款簿记）；实际载荷：各 kind 的表 backing
  （iterators/VM_FILE_RESPONSES/VM_UPLOAD_RECEIPTS）不动，Response 数据
  owned 于凭据。
- **访问器/迁移终形（R2 终形，替代 R1 的「client 保持表语义」）**：
  - 读访问器（status_code/body/body_bytes/body_to_file/header_get）统一
    快照/凭据载荷（`read_response_snapshot`/`resolve_resource`）。
  - setter 五族（含 bytes）`mutate_response_snapshot`+owned-slot 返回
    协议（SD-01）；`response_bytes` 迁移快照路径（body=AB roundtrip）。
  - **client Response 进 egress 边界凭据化**（R2-05）：`push_handle_result`
    产 Response 凭据对象（owned 快照）——上游 status/body 保真转发；访问器
    统一载荷（快照/int 表）兼容；HTTP_RESPONSES 表 insert 退役，int 查表
    面成为死兼容（零用户）。
  - engine CALL_SPEC 兜底对凭据 receiver 分派同一 shim 族。
- 新增 dispatch 责任：handler 返回值槽 RC 释放（sync/parked 双点）+
  `release_task_credentials` 任务栈全部堆对象槽清偿（含父容器级联）。

## SD-06（modify）`docs/specs/auto-man/design/api-generation-integrity.md`

- 返回分类输入升级：`ApiEndpoint.response`（提取期两遍判定冻结）+
  `ResponseKind::from_return_identity`；api_gen/targets/back_proxy 消费同一
  冻结身份（不各自裸串分类）。
- **alias 按 target 展开（R2-07 终形，替代 R1 的「alias 入名集」）**：
  `ApiType.alias_of` 携 target——`type Download = FileResponse` 按 target
  解析为 File；alias→用户名 → Json；`http.` 限定名是 stdlib 绑定，直接按
  stdlib 裸名分派（**不受用户名集遮蔽**）；用户直接声明（struct/enum，含
  空成员）→ Json。back_proxy fn_meta **双集**（直接声明名集 +
  alias→target 表，primary 首段分类）。
- 三层遮蔽：提取级（ApiExtractor 两遍）/codegen TypeStore→
  `API_RETURN_SHADOWED`/back_proxy fn_meta；strict 门/738 双指纹/751 三态
  单点不变（plan738 75/75）。
- TypeScriptGenerator.with_types 注入（类型映射来源身份）。

## SD-07（modify）`docs/specs/stdlib/design/backend-assembly.md` + `docs/specs/stdlib/project.md`

- 支持矩阵回写：HTTP 返回资源身份边界（四族凭据+碰撞防线+返回边界身份
  授予）、owner 11 位段耗尽拒绝、setter owned-slot、任务终态容器级联
  清偿、client egress 凭据化、sync/async 终值同形（含 i64 全宽/record
  值域）、失败终态 500 化、返回值 RC 生命周期。
- 纠正陈旧描述：redirect/musk ctor 构造点不再经 HTTP_RESPONSES 表（表
  仅剩 client 死兼容面）；"async 终值降级 null"条目退役。
- **未验证面如实单列（R3，AC-08 partial 依据）**：生成 Rust 真 TCP 矩阵
  =master 同形预存编译漂移，未在本计划验证（归 742 轨收口，不据此判
  AC-08 pass）。

---

**AC 映射**：SD-01{AC-01,02,05,06}、SD-02{AC-01,02,07,08}、SD-03{AC-03,04}、
SD-04{AC-03,05,06}、SD-05{AC-02,05,07}、SD-06{AC-07,08}、SD-07{AC-08,09}。
