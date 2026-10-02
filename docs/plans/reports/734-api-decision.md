# PLAN-734 T-01 决策报告：契约模型、兼容裁定与实现路线冻结

- 基线：master `6dd609ed9`（730 已 merged/archived；分支 plan-734-dev，worktree
  `D:/autostack/.wt/lang-734/auto-lang`）。
- 730 最终契约核对：`docs/specs/stdlib/design/http-server-uploads.md`（typed UploadRequest/
  Receipt、contains 判定为其实现细节）与本计划无语义冲突；本计划把 contains 判定**规范化为
  类型身份分类**，730 的三重命中门语义保持（分类函数返回同值）。
- 三面测绘结论（探针/静态证据在案）：53+ 处 contains 猜测面、8 个生成入口错误传播面
  （3 硬/5 warn 或静默）、VM 绑定/编组/back-proxy 的规则不对称清单——见正文引用。

## 1. 核心实证（T-01 探针，全部在案可复现）

| # | 实证 | 证据 |
|---|---|---|
| E1 | **VM int 核心是 i64 完好**：字面量 `5000000000` 输出正确、`+1` 算术正确 | `plan734_probe_vm_i64_literal_arith_json`（print 5000000000/5000000001） |
| E2 | **json.encode 把 i64 掩成 u32**：`json.encode(5000000001)` → `4294967295` | 同上探针第三行 |
| E3 | **nv_to_json 无 is_i64 臂**：i64 值 HTTP 回复序列化为 `null` | http_server.rs:655-715（fallthrough `Some("null")`） |
| E4 | **bind 双路 i64→i32 静默截断**：`push_typed_string_arg`（parse_i64→`as i32`）与 `json_to_vm_value` 同 | http_server.rs:6648 / stdlib.rs:3064-3072 |
| E5 | **`{"error":` 前缀→500 无 spec 依据**：http-server.md §8.1 只规定缺参 400 的 error 形状；前缀猜测是实现自有 | spec grep 零命中；json_value_reply:6244 |
| E6 | **ProxyRequest 自带 headers**（现 dead_code）——back-proxy 可加 meta 源 | back_proxy.rs:171 |
| E7 | **tauri v2.12.1 dev-dep + test feature 在本 workspace 编译通过**，`mock_builder`/`get_ipc_response` 在案 | cargo add + check 0 error；tauri-2.12.1/src/test/mod.rs:166/297 |
| E8 | iterator/response-handle 编组臂**无声明类型门**（普通 int 撞 id 即劫持）；`type Response` 存在于 stdlib 词汇（http.at:92）——response 臂可加声明门 | http_server.rs:5285-5326；http.at:92-96 |

## 2. 冻结决策

### D1 契约模型（api/contract.rs）

- **分类单源**：`ResponseKind {Json, Stream, File, Upload, ExplicitResponse, Void}` 与
  `DataKind {Str, Bool, Int, Float, Optional, Array, Record, Opaque, Unknown}` 由
  `classify_endpoint(&ApiEndpoint, &ApiModule.types)` 派生——输入仍是既有 ApiModule
  （type_to_string 串 + 模块 types），**不改 parser/AST**。所有消费者（VM 串表、back-proxy
  fn_meta、三 targets、api_gen 分支）改调分类函数；contains 字面量全部退役（测试断言除外）。
- 分类规则（与现行 contains 语义等价但按类型身份）：
  `File`=返回类型身份 `FileResponse`（含 Future 包装）；`Upload`=参数含 `UploadRequest` 或
  返回 `UploadReceipt`；`Stream`=返回 `~Iter<T>`/`~Stream<T>`/`Iter/Stream` 泛型实例；
  `ExplicitResponse`=返回 `Response`；`Void`=无返回；否则 `Json`。**假同名反例**：用户定义
  `type MyFileResponse` 不命中（身份=裸名精确匹配，非 contains）。
- `ParamPlan`：每参数 `(name, source: {Path, Body, Query, Meta, WholeBody, Upload}, kind,
  required)`——source 由 path 模板/方法/meta 约定/UploadRequest 分类派生；现行
  whole-body 单参容忍与 META_PARAM_NAMES 作为**显式兼容标记**保留在分类里（不再散落）。
- **span 边界（如实）**：AST `Fn.span` 从不被 parser 填充、ApiEndpoint 无位置字段——
  v1 诊断携带 `文件 + fn 名 + 阶段 + 原因 + 涉事类型串`（fn 是端点定位单元）；parser 全量
  span 管线超出本期（另案）。诊断结构 `ApiDiagnostic {endpoint, stage, reason, detail}`。
- 契约**不可变按 session**：VM 侧三张全局表保留（进程内单程序模型——run 管线每程序重置，
  既有语义），但读方全部经分类函数；back-proxy 已自持 per-session 表——改存 typed kind。

### D2 int 宽度（E1-E4 修复路线）

- **Auto `int` 语义 = i64**（trans 已映射 i64；VM 核心 i64 完好）。bind 侧：声明 Int 的
  string/JSON 值经 `encode_i64_with_heap`（48 位内联/堆装箱），**删除两处 `as i32`**；
  VM 内算术/比较已支持（E1）。i64 之外的 `byte/uint/u64/u32/usize` 按各自范围校验
  （超界 400，不截断）。
- 序列化：`nv_to_json` 补 `is_i64`（48 位内联→十进制）与堆 BigInt（`BigIntData`）臂；
  stdlib `json.encode` shim 的 u32 掩码同修（同一 NanoValue 读取点）。
- **TS 超安全整数**：生成客户端对 int 参数/响应加运行时守卫 `Number.isSafeInteger`——
  参数超界抛 typed Error（显式拒绝），响应超安全整数返回原 JSON 值并以 `as string`
  codec 交付（响应字段十进制字符串，§5.1 授权的"明确字符串 codec"）。守卫 helper 进
  生成 preamble。

### D3 错误收敛（E5）

- 删除 `{"error":` 前缀→500 猜测：业务 record 含 error 字段=200 数据（AC-03）。
  VM 执行失败（handler_error_reply 500）、序列化失败（E3 修复后 None → **500 空 body
  改 500 + {"error":"serialization failed"}**，不再 200 空体）。
- 兼容扫描（T-03 执行）：全测试面 grep 断言 `500`+error-JSON 的既有用例逐名核对——
  预期命中 musk 015/017 CRUD 模板路径（模板内 `{"error"...}` 返回形态）；模板路径在
  strict 模式退役后这些 fixture 走真实 body（行为不变或按 §6.1 重锁）。

### D4 strict 生成与 scaffold（auto-man）

- **默认 strict**：full-parse 失败 → `Err`（lenient 退役为显式 `AUTO_API_LENIENT=1`，
  其产物按 scaffold 标注）；零端点且解析成功 = 合法 no-API（Ok）。
- **真实实现优先**：endpoint.body 存在（strict 下恒有）→ a2r 转译必须成功否则该端点
  `Err`（诊断 500 handler 退役为生成期 Err）；db 委派（resolve_db_call 命中）仍是
  合法实现；CRUD 模板/Default 桩/TODO 骨架只在 `AUTO_A2R_BODY=0`（重语义=**显式
  scaffold 模式**，生成物头部标注 `// SCAFFOLD`）下可达。`AUTO_A2R_BODY=0` 语义变更
  记入 SD-04（"不能假称同源"——scaffold 标注使其不再冒充）。
- a2r 真实缺口（fixture 触及的分支 return/async 调用）窄修于 trans/rust.rs；超范围
  → needs_replan。

### D5 生成发布

- **内存 bundle**：rust-server 全部产物先在内存生成（含 main.rs）→ 混合状态校验 →
  一次性写盘（失败=零写或尽力清理+Err）；写序保持"main.rs 后 ensure workspace"约束。
- **ready 记录**：`generation.json`（schema_version、端点清单、api.at+db.at+companion
  的 FNV 内容 hash、impl 标签、生成时间）随 bundle 最后写入；`start_api_server` 的
  复用路径（AUTO_REUSE_BACKEND=1）校验 hash 一致才复用，不一致重新生成。
  TS 侧 `.api_functions` 增加同源 hash 行，旧清单失配即失效缓存（既有
  invalidate_if_api_functions_changed 自动生效）。

### D6 错误传播（T-06 改点清单——测绘 8 入口）

| 入口 | 现状 | 改为 |
|---|---|---|
| vue.rs:4346 `let _ =` | 静默 | `?` 传播 |
| vue.rs:5797/6600 warn+继续 | warn | `?` 传播（无 API 契约时 Ok 不受影响） |
| tauri.rs:65/122 warn | warn | `?` 传播 |
| rust_ui.rs:4205 warn 后仍 start | warn | `?`（start_api_server 内二次生成本就硬传） |
| rust_ui.rs:3754/3957 | 硬 | 保持 |

### D7 merged 缺实现

- `warn_api_noop`（warn+null）→ `VMError::RuntimeError("api fn '{name}' has no
  implementation in merged mode …")`——可定位失败（§5.2）。行为变更记 SD-01 兼容表。

### D8 back-proxy

- fn_meta 拼串 → per-session `fn_kind: ResponseKind` + `fn_params`（已有）分类派生；
  501/SSE 守卫按 kind。**meta 源补齐**：`ParamSource::Meta` 参数从 ProxyRequest.headers
  构造 cookies/auth JSON（与 standalone 同形，E6）。

### D9 编组分派（E8）

- 声明 kind 先行：`Upload`（declared）→ `Stream`（declared）→ `File`（declared）→
  `ExplicitResponse`（declared，新增门）→ JSON 兜底。iterator/response-handle 注册表
  探测**仅在 declared kind 匹配时**执行——普通 int 撞 id 返回数值（AC-03 反例）。
  无声明签名的 legacy 路径保持现状（兼容矩阵明示）。

### D10 Tauri 测试腿（E7）

- `tauri = "2.12"` dev-dep（features=["test"]）已加并编译通过；用 `mock_builder()` +
  `get_ipc_response()` 走真实 command dispatcher（生成 commands.rs 编入测试 crate）。
  不证明 webview/打包。dev-dep 新增记入 SD-04（仅测试依赖，非运行时）。

## 3. 兼容表（Spec ↔ 代码 ↔ 消费者）

| 面 | canonical 规则 | 代码现状 | 裁定 |
|---|---|---|---|
| 缺参 400 error 形状 | §8.1 规定 | 一致 | 保持 |
| `{"error":` 前缀 500 | **无规则** | 实现猜测 | 删除（AC-03 授权） |
| str whole-body 单参容忍 | 无明文（Plan 346 事实） | 在 | 保留为显式 ParamSource::WholeBody |
| meta 别名（meta/metadata/req/request） | Plan 317 §11 约定 | 在（http_server）；back-proxy 缺 | 保留 + back-proxy 补 headers 源 |
| ?T 缺失 → 404 vs null | spec 记"待定"（http-server.md §8 注） | http=Option.None→null（Plan 326 注）；生成腿按 Result | **保持 null**（326 已裁定），差异入矩阵 |
| int 宽度 | 无明文 | trans=i64、VM bind/marshal=32 位有损 | 以 i64 冻结（E1 授权），SD-01 记 |
| AUTO_A2R_BODY=0 | 无 spec | 模板回退 | 重语义=scaffold 显式模式（SD-04） |
| merged noop null | 无 spec | warn+null | 改为定位失败（SD-01） |
| 响应头 404（str ?T） | spec §8.1 提及 404 映射"可后加" | 未实现 | 不做（现状 null 保持） |

## 4. 支持子集冻结（首期普通 JSON 面）

参数/返回数据：str/Str、bool、int(i64 全域)、float/double（有限值；NaN/Inf 参数 400）、
`?T`、`[]T`、命名 record（模块 types 已解析，嵌套 ≤ 既有 marshal 深度 32）。未解析类型/
泛型/enum → 生成期诊断（不降 String）。i64 参数 TS 侧安全整数守卫（D2）。
**不支持面**（矩阵明示诊断）：char/byte 形参、Map 形参、union/tag、循环 record。

## 5. 原型验证状态

- i64 语义探针（E1/E2）在 `plan734_api_contract_tests.rs`（探针断言冻结后转正式矩阵）。
- tauri dev-dep 编译（E7）通过；MockRuntime 腿待 T-07 fixture。
- 五形态最小原型：T-02 契约分类落地后以 `examples/http_server/api_contract`（T-07 交付物）
  逐形态点亮——VM HTTP（既有 harness）、生成 Rust（spawn fixture）、back-proxy
  （既有 harness）、merged（run_with_capture）、Tauri（mock_builder）。
- **无 needs_replan 项**：所有 AC/SD 均有上述路线覆盖；最大不确定（a2r 真实缺口面）
  在 T-04 fixture 驱动下窄修，超范围即停。

## 6. 依赖

新增 dev-dep：`tauri 2.12 (test)`（仅 auto-man 测试面，编译已验）。运行时零新依赖。
