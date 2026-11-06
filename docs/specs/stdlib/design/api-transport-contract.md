# API 传输契约——普通 JSON API 的类型身份、参数来源与能力分类（单源）

> 路径：`crates/auto-lang/src/api/contract.rs`（分类单源）+ `api/diagnostic.rs`
> （结构化生成诊断）+ 消费面（VM 绑定/编组、back-proxy、三 targets、auto-man api_gen）
> 状态：active（PLAN-734 交付；PLAN-776 修订——按值资源凭据替代查表兼容、
> P734-D7 关闭、限定名/alias 链来源分类；决策/验证报告见
> `docs/plans/reports/734-api-*.md` 与 `776-return-resource-*.md`）

## 1. 类型身份分类（非 contains）

```rust
ResponseKind::from_return_string(ret) -> {Json, Stream, File, Upload, ExplicitResponse, Void}
ParamKind::from_param_string(ty, module_types) -> {Str, Bool, Int, Float, Optional, Array, Record, Unsupported}
EndpointContract::build(module) -> 参数计划（name/source/kind/required/declared_ty）
is_upload_param(ty) / is_meta_alias(name)（META_PARAM_NAMES 单源）
```

- 判定是**裸名/泛型解包精确匹配**：`Future<FileResponse>` 解一层 Future 后裸名等值；
  用户定义 `MyFileResponse`/`List<FileResponse>` 不命中 File（假同名反例由测试锁定）。
- **PLAN-776 来源身份**：`from_return_identity(ret, module_types)`——`http.`
  限定名是 stdlib 绑定，**先于**用户名集判定（用户 `type FileResponse {}`
  不遮蔽 `http.FileResponse`）；alias 目标携**完整形态**（提取器存
  `type_to_string(target)`——`type D = Future<FileResponse>` 的实参不丢，
  分类与直接 `Future<FileResponse>` 同规则）并按**目标链 visited 集**
  递归展开（合法深链不限层；真环保保守 Json；back_proxy fn_meta 同语义
  ——target 全形存储 + 单步规范化 + visited 集）。
- **双族输入**：API 元数据侧（`type_to_string`/`unique_name` 裸名）与 VM 参数签名侧
  （ast `Type` Display——User 类型是 s-expr `(type-decl (name X) ...)`）都被
  `is_upload_param` 按名字等值接受；s-expr 形态仅在名字段等值时命中。
- 所有消费面（VM 编组门/绑定、back-proxy 守卫、typescript/tauri/axum targets、
  auto-man api_gen 分支）经本模块分类；`contains("FileResponse")` 等子串猜测
  在生产代码面退役（测试断言位可留）。
- VM 参数身份在 codegen 采集时用 `unique_name()`（User 的 Display 是 s-expr）。

## 2. 参数来源与绑定（§5.2 契约）

- 优先级 `path > JSON body > query`；兼容面为**显式标记**：
  `ParamSource::Meta`（cookies/auth JSON，名字约定 META_PARAM_NAMES =
  meta/metadata/req/request）、`ParamSource::WholeBody`（单参容忍）、
  `ParamSource::Upload`（730 注入，绑定先于一切规则）。
- body 值按声明类型校验（`validate_body_value`）：str-for-int → 400（不静默 marshal）、
  缺 required → 400、float 非有限 → 400、optional 接受 null。int 语义 i64 全域
  （bind 经 `encode_i64_with_heap`，48 位内联/堆装箱；定点变体 u32/u64/uint/byte
  按范围校验超界 400）。
- back-proxy 补 meta 源：ProxyRequest.headers → cookies/auth JSON（与 standalone
  `build_meta_json` 单源同形）——meta 端点不再恒 400。

## 3. 返回编组与错误收敛（PLAN-776 修订）

- 编组分派：**资源凭据臂最前**（TAG_OBJECT → id 层 owner 校验 → downcast
  四族：Response 快照 clone / Sse 注册表校验后单源 serve / File+Upload take
  移交 + mismatch 500）→ upload/file 纯诊断门（声明专属返回且值非凭据对象 →
  mismatch 500，不触表）→ JSON 兜底。
- **查表臂全部退役；声明门废止——声明不授予身份**：iterator/SSE 身份在
  generator 生产点建立（codegen 无条件 WRAP_SSE_RESOURCE + dispatch 臂 +
  Sse.into_response；见 [http-return-resources](http-return-resources.md)），
  未声明专属返回的普通数值（含活跃 iterator 同号碰撞）永远 JSON 数据。
- 返回 async 块的 fn 可被驱动 Future：marshal 对 **TAG_FUTURE 逐值载荷**
  直接驱动（Value 载体保真映射 `Value::Future`——数组/record 字段取出、
  helper 传递、捕获读回的真实 Future 天然展开）；legacy `(fid<<8)|0xF0`
  位形态**驱动臂退役**——普通 i32（位模式撞活跃 fid）恒数据。
- 普通 int 不被资源 id 误判由凭据表示本身承载（数值不可制造 TAG_OBJECT/
  TAG_FUTURE）+ e2e 锁定。
- `{"error":` 前缀猜测 500 退役：业务 record 含 error 字段 = 200 数据；
  序列化失败 → 500 `{"error":"response serialization failed"}`（不再 200 空 body）；
  VM 执行失败 → 500（既有）。
- merged 缺实现 = 可定位 VMError（`has no implementation` + 行动指引），不再 warn+null。

## 4. i64 全域（序列化三修复）

| 位点 | 契约 |
|---|---|
| `push_typed_string_arg`（path/query） | i64 全域压栈 + 定点范围校验 |
| `json_to_vm_value`（body） | as_i64/as_u64 全域；不可表示数 → 500 |
| `nv_to_json`（HTTP 回复） | i64/u64/bigint（48 位内联+堆装箱）按数值序列化 |
| `nv_to_vm_value`/`String` 编组 | i64/u64/bigint 臂（json.encode 编组层）；`vm_value_to_json` 有 `Value::I64` 臂（i32→I64 统一后必须存在，丢失即 664 值域红——R2 实证） |

### VM JSON 值编码（PLAN-949）

`Json.encode`/`json.encode` 的 native wrapper 在 String 编组抹去类型之前，
使用现有 VM JSON 转换器序列化真实值。Bool/Null/Number 保持 JSON 值；
普通字符串 `"true"`、`"false"`、`"null"`、`"123"` 始终编码为 JSON 字符串，
不得按内容猜类型。既有文档和 string-list ABI 保持；解析文档中的嵌套
整数保留 i64/u64 数值，不截断到 i32。转换器既有深度和不支持值边界保持，
此处不承诺任意资源对象都能编码。

旧 native list 的裸 i32 句柄以既有栈槽 stake 表示拥有的引用；无该份额的
i32 是普通数值，即使其位值恰与活文档/list id 相同。encode 在查堆前区分
这两者，消费时转移/清除原槽份额，不能借序列化普通数字释放同号对象。
不改 string-list 的负字符串池索引布局，也不新建 JSON 身份表。

legacy 静态 `Json.get/get_at` 直接返回序列化的 JSON **文本片段**，直接
文本工具链保持兼容。当实际内建 getter 表达式直接作为内建 encode 的参数，
codegen 对位置参数和既有命名参数 `value:` 均在单次求值后，通过既有
`json.to_value` 恢复 JSON 值再编码。
同名变量 receiver、普通函数返回字符串及字符串池复用不触发这种转换。
片段保存到变量后若要作为 JSON 值使用，调用方显式 `to_value`；普通字符串
不自动重解析。无全局文本 provenance 表或 String 编组规则变更。

P734-D4 的上述 encode 路径由 PLAN-949 的真实 parse/codegen/VM 用例覆盖；
独立的 str+int 泛型转换边界仍保留，不据此宣布整个泛型 ABI 债关闭。

## 5. P734-D7 关闭（PLAN-776 替代 R2 冻结裁定）

Plan 326/346 wire 契约的 6 个 e2e（302 redirect/sse chain/host forward/
value accessors/musk response 常量）wire 行为不变由**真实生产点凭据**保持
（redirect/musk ctor 8 族/Http.ok 五族/`http.response()`+setter 链/
Sse.into_response/file/upload 构造点全部产凭据对象；plan326 http_e2e
34/34、碰撞族、常量重放族）；R2 的「注册表命中制兼容」与 P734-D7 历史批准
边界被本计划显式替代——查表猜测不再作为身份来源。strict 声明分派语义保留于
Upload/File 诊断门（声明专属返回 + 非凭据对象 → mismatch 500）。

## 6. 结构化生成诊断（api/diagnostic.rs）

`ApiDiagnostic {endpoint, file, stage(Parse/Type/Transpile/Implementation/
Capability/Publish), reason, detail}`——生成失败的结构化载体（v1 定位粒度 =
文件 + 端点名 + 阶段；AST span 管线另案）。
