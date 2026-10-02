# PLAN-734 T-07 绑定/编组报告：参数校验、i64 全域与错误收敛

- 代码基线：worktree `D:/autostack/.wt/lang-734/auto-lang`，分支 plan-734-dev
  @ `438d22c47`（本报告范围内的最终代码见 verification 报告头部）。
- 命令：`cargo nextest run -p auto-lang --lib plan734`（merged/探针）；
  `cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1
  http_e2e_plan734`（VM HTTP wire）。

## 1. i64 全域三修复（E1-E4 实证 → T-03/T-07 落地）

| 位点 | 修复前（实证） | 修复后（e2e/探针锁定） |
|---|---|---|
| `push_typed_string_arg`（path/query） | parse_i64→`as i32` 静默截断 | `encode_i64_with_heap` + 定点范围校验（u32/u64/uint/byte 越界 400） |
| `json_to_vm_value`（body） | `as_i64 → as i32` | i64/u64 全域压栈；不可表示数 → 500 |
| `nv_to_json`（HTTP 回复） | i64 值落 `null` | i64/u64/bigint（48 位内联+堆装箱）按数值序列化 |
| `nv_to_vm_value`/String 编组 | i64 落位型 Debug 串 | i64/u64/bigint 臂（json.encode 编组层） |
| wire | `/api/big` 返回 null；`/api/bigparam/5000000000` 截断 | `5000000000` / `5000000001`（http_e2e_plan734_vm_i64_wire） |

**遗留（P734-D4，如实）**：泛型 stdlib 通路仍有两处 i32-lane 损坏——
`json.encode` 的**返回通路**（编组层已修，shim 收到正确十进制 arg，但返回串
在另一解码点变 4294967295）与 `"id=" + 5000000000` 的 str 拼接（-1705032704
= i32 回绕）。均在 API 契约面之外（应用层 stdlib 泛型转换），HTTP/marshal
面已全修并有 e2e 锁定；归因清偿另案。

## 2. 参数校验（AC-02）

- 来源优先级 path > body > query 保持；body 值按声明类型校验
  （`validate_body_value`）：str-for-int → 400（不再静默 marshal）、缺
  required → 400、非 JSON body 多参端点 → 400、float 非有限 → 400。
- optional 接受 null + 内层标量形态校验；数组/record 结构存在性。
- UploadRequest 注入/META 别名/whole-body 容忍全部经契约单源
  （`is_upload_param`/`is_meta_alias`）显式标记。
- back-proxy 补 meta 源（ProxyRequest.headers → cookies/auth JSON，与
  standalone `build_meta_json` 单源同形）——meta 端点在 back-proxy 不再恒 400。

## 3. 编组声明门（D9/AC-03）

分派顺序：Upload（declared）→ Stream（declared，iterator 注册表探测）→
File（declared）→ ExplicitResponse（declared，response-handle 探测）→ JSON
兜底。iterator/response-handle 两臂此前无声明门——普通 int 撞资源 id 被劫持
为 SSE/Response；现两臂均有 ResponseKind 声明门（`declared_return` +
axum closure 反查），`/api/plain → 734734` e2e 锁定反例。

## 4. 错误收敛（D3）

- `{"error":` 前缀猜测 500 退役：业务 record 含 error 字段 = 200 数据
  （`/api/errorfield` e2e：200 + `{"error": "business-rejected", "value": 7}`）。
  spec 无该前缀规则（决策报告 §3 兼容表 E5）。
- 序列化失败 → 500 `{"error":"response serialization failed"}`（不再 200 空 body）。
- merged noop → 可定位 VMError（D7）。
