# API 传输契约——普通 JSON API 的类型身份、参数来源与能力分类（单源）

> 路径：`crates/auto-lang/src/api/contract.rs`（分类单源）+ `api/diagnostic.rs`
> （结构化生成诊断）+ 消费面（VM 绑定/编组、back-proxy、三 targets、auto-man api_gen）
> 状态：active（PLAN-734 交付；决策/验证报告见 `docs/plans/reports/734-api-*.md`）

## 1. 类型身份分类（非 contains）

```rust
ResponseKind::from_return_string(ret) -> {Json, Stream, File, Upload, ExplicitResponse, Void}
ParamKind::from_param_string(ty, module_types) -> {Str, Bool, Int, Float, Optional, Array, Record, Unsupported}
EndpointContract::build(module) -> 参数计划（name/source/kind/required/declared_ty）
is_upload_param(ty) / is_meta_alias(name)（META_PARAM_NAMES 单源）
```

- 判定是**裸名/泛型解包精确匹配**：`Future<FileResponse>` 解一层 Future 后裸名等值；
  用户定义 `MyFileResponse`/`List<FileResponse>` 不命中 File（假同名反例由测试锁定）。
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

## 3. 返回编组与错误收敛

- 编组分派：Upload（declared）→ iterator/SSE（注册表命中——**兼容制**，见 §5）→
  File（declared）→ Response handle（注册表命中——兼容制）→ JSON 兜底。
- 普通 int 不被资源 id 误判的保护由 id 空间分离承载（iterator/Response 各自计数器）
  + e2e 锁定（`/api/plain → 734734`）。
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

已知边界（债 P734-D4）：泛型 stdlib 的 json.encode 返回通路与 str+int 拼接
仍有 i32-lane 损坏——应用层泛型转换，API 契约面外。

## 5. wire 兼容裁定（R2 冻结）

Plan 326/346 wire 契约的 SSE 链/redirect handler 声明 `int` 返回（axum
Sse.into_response 返回持有 iterator/Response 的句柄）——**声明门会破坏既有 6 个
e2e**（302 redirect/sse chain/host forward/value accessors/musk response 常量，
merge-base 绿→声明门红→单臂回退恢复，二分定位在 verification §3.3）。因此
iterator/SSE 与 response-handle 臂保持注册表命中制；strict 声明分派保留于
Upload/File 两臂（其 wire 契约本就要求声明）。

## 6. 结构化生成诊断（api/diagnostic.rs）

`ApiDiagnostic {endpoint, file, stage(Parse/Type/Transpile/Implementation/
Capability/Publish), reason, detail}`——生成失败的结构化载体（v1 定位粒度 =
文件 + 端点名 + 阶段；AST span 管线另案）。
