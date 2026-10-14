# PLAN-776 拟议规范增量草稿（SD-01..07）

> 供 review/merge 按实证沉淀（本次不修改 canonical Specs）。
> 每条 SD：before（现行规则）→ after（本计划落地后的规则）→ 代码/测试证据。
> 执行细节与门禁见 `776-return-resource-verification.md`；表示决策见
> `776-return-resource-decision.md`。

## SD-01（add）`docs/specs/stdlib/design/http-return-resources.md`

**新增全文合同**（草案要点）：

- **凭据表示**：HTTP 返回资源 = `heap_objects` 表 entry
  （`TypeTag::HttpReturnResource`；`HttpReturnKind`：Response owned 快照 /
  Sse{iterator_id} / File{descriptor_id} / Upload{receipt_id}）。NanoValue 侧
  `TAG_OBJECT`，Value 侧 `Value::VmRef`——普通 i32/i64（TAG_I32/I64）永不持有
  资源身份；任何数值运算不可制造 TAG_OBJECT。
- **owner/代次**：heap 表 per-VM=owner 边界（编组查本 VM 表，跨 owner/回收
  凭据=受控拒绝）；heap id 单调递增不复用；分配回绕=耗尽拒绝。
- **传播**：栈/globals/容器/closure captures/Future result 五载体身份保持
  （VmRef/TAG_OBJECT 双桥）；资源不可经数值转换获得。
- **编组**：marshal 资源臂优先于一切 int 探测；Response=快照 clone（重复返回
  不消费）；Sse=注册表校验后单源 serve；File/Upload=take 单次移交+mismatch 500。
- **生命周期**：凭据随 RC 归零回收（dying 宽限窗）；handler 返回值槽 RC 在
  dispatch 释放（sync/parked 双点）；File/Upload 表生命周期归 scope 组收口
  （凭据 Drop 不触碰表）。
- **观察**：`live_resource_count`/`probe_last_resource_handle` 仅 cfg(test)
  读（计数器生产写、无伪造能力——读得的是普通数字）。

证据：`vm/ffi/http_return_resource.rs`（AC-01/02/05/06 测试族 25 项）。

## SD-02（modify）`docs/specs/stdlib/design/api-transport-contract.md` §3/§5

- before §3：编组分派=Upload(declared)→iterator/SSE(注册表命中)→File(declared)
  →Response handle(注册表命中)→JSON；普通 int 保护=id 空间分离+e2e。
- after §3：**资源凭据臂**（TAG_OBJECT→本 VM 堆→downcast）置于最前；legacy
  int 查表命中制**原样保留**（client 句柄、`http.response()` setter 死面、
  Plan 326/346 wire）。
- before §5（P734-D7）：6 wire 用例依赖查表命中制。
- after §5：**P734-D7 关闭**——真实资源走对象臂（redirect/musk ctor/SSE 链/
  File/Upload 构造点全部 push 凭据），legacy int 面双轨保留；wire 行为不变
  （plan326 http_e2e 34/34、碰撞族、重放族）。
- §1 身份分类：`from_return_identity(ret, module_types)`——裸名命中模块声明
  （用户）→Json；`from_return_string` 降为无模块上下文保守入口。

## SD-03（modify）`docs/specs/stdlib/design/http-handler-async-lifecycle.md`

- before：Phase A——Failed/缺 result → 200 null；值域窄（Int/Float/Bool/Nil/
  Str，其他降级 null）。
- after：**终值统一**——`final_value_reply` 资源终值与 sync 同一分派；完整值域
  （i64 全宽 48 位内联/BigInt 装箱、VmRef 身分保持、Array/Obj 物化容器即用
  即弃）；Failed/缺 result/不可转换 → 有界 500（`async_failure_reply`）；
  显式成功 Nil → 200 null；取消/断连迟到值不回包（705 机制）所携凭据随
  RC 回收。

证据：e2e async_terminal_resource/i64_full_width/failed_is_500/record_json；
plan705 19/19。

## SD-04（modify）`docs/specs/stdlib/design/http-stream-lifecycle.md`

- 增连接点：`Sse.into_response` 装箱凭据（iterator_id；**0 是合法 id**——
  首个 generator）；marshal Sse 臂校验注册表后走 `sse_iterator_reply` **单源**
  （与 legacy int 命中臂共用 producer spawn/`produce_sse_frames`/cleanup 守卫
  ——无第二输出循环）。
- 取消/背压/断连/半关闭合同不变（plan707/705 族绿）。

## SD-05（modify）`docs/specs/auto-lang/vm/design/ffi.md`

- HTTP 资源 native 逻辑身份：`HttpReturnResource` 的 HeapObject 实现与 push
  模板（insert_heap_object + rc_push，`push_rust_obj` 同款）；实际载荷：各
  kind 的表 backing（iterators/VM_FILE_RESPONSES/VM_UPLOAD_RECEIPTS）不动，
  Response 数据 owned 于凭据。
- 访问器边界：client `.status()/.body()/.header()` CALL_SPEC int 兜底保留
  （client 表面）；server Response 可变 setter 链（`http.response()`+setters）
  保留 int+表（死面零用户，原样）。
- 新增 dispatch 责任：handler 返回值槽 RC 释放（sync `HandlerEnd::Value`/
  parked `HResume::Value` 双点）——编组产物 owned 后释放栈引用。

## SD-06（modify）`docs/specs/auto-man/design/api-generation-integrity.md`

- 返回分类输入升级：`ApiEndpoint.response`（提取期两遍判定冻结）+
  `ResponseKind::from_return_identity`；api_gen/targets/back_proxy 消费同一
  冻结身份（不各自裸串分类）。
- 用户同名遮蔽三层（提取/codegen TypeStore→API_RETURN_SHADOWED/back_proxy
  fn_meta 三段）；strict 门/738 双指纹/751 三态单点不变（plan738 75/75）。
- TypeScriptGenerator.with_types 注入（类型映射来源身份）。

## SD-07（modify）`docs/specs/stdlib/design/backend-assembly.md` + `docs/specs/stdlib/project.md`

- 支持矩阵回写：HTTP 返回资源身份边界（四族凭据+碰撞防线）、sync/async
  终值同形（含 i64 全宽/record 值域）、失败终态 500 化、返回值 RC 生命周期。
- 纠正陈旧描述：redirect/musk ctor 构造点不再经 HTTP_RESPONSES 表（表仅剩
  client 注入+setter 死面）；"async 终值降级 null"条目退役。

---

**AC 映射**：SD-01{AC-01,02,05,06}、SD-02{AC-01,02,07,08}、SD-03{AC-03,04}、
SD-04{AC-03,05,06}、SD-05{AC-02,05,07}、SD-06{AC-07,08}、SD-07{AC-08,09}。
