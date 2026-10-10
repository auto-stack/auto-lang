# PLAN-755 T-01：HTTP 公共面契约盘点表（2026-10-10）

盘点方法：`crates/auto-lang/src/tests/plan755_http_contract_tests.rs::plan755_http_public_face_contract_coverage`
机器走查——遍历 `stdlib/auto/http.at` 公共层 pub fn/method，按
`validate::public_native_name` 求 native 名并解析 id（catalog peek → interface resolve），
按「解析成功 + shim 已绑定」判定 gate 可达（真实程序引用时走 `verify_core_reference`）。
**本表由测试自动产出，修复后走查即常驻守卫（SD-01）**，静态表格仅为 2026-10-10 快照。

## 1. 修复前缺口（46 项 = 41 缺契约 + 5 契约漂移）

### 1a. 缺独立适配契约（41 项，全部 resolved+bound，shim 已存在）

| 家族 | 符号 → native 名（id） | .at 签名要点 |
|---|---|---|
| Server 链 | server #2200；Server.get/post/put/delete #2201-2204；Server.static #2205；Server.listen #2206；listen #2259 | handler 参数 `fn(Request) Response` |
| Response 构造 | response #2210；Response.status/header/text/html/bytes #2211-2215；**response_redirect #3108（751-R2 实证）** | 链式返回 Response |
| 客户端 | post #2231、put #2232、delete #2233；post_sync #2256、post_bearer #2258（返回 str）；last_status #2257 | |
| RequestBuilder | header/body/json/timeout/send #2235-2239 | **json[T] 声明级泛型方法**（新增 `declare_generic_method_contract_identity`） |
| 流式 | post_stream #2241、post_stream_with_headers #2255；HTTPStream.close/iter → auto.http_stream.stream_close/iter #2244/2245 | 同 id 双名沿用 stream_next 方法形 |
| SSE | sse_open #3145、sse_poll #3146、sse_close #3149、sse_error #3150 | stream_id int 句柄 |
| transfer | transfer_upload #9931、transfer_next_progress #9933 | |
| **上传（PLAN-730）** | **upload_receive #9937（751-R2 实证）**、upload_commit #9939、upload_reject #9940 | UploadSession/Receipt 句柄 |

### 1b. 契约漂移（5 项：inventory 推导契约与公共层身份不符）

| 符号 → native 名（id） | 漂移 | 处置 |
|---|---|---|
| ok #2220 / created #2221 / bad_request #2222 / not_found #2223 / internal_error #2224 | `build_from_inventory` 按 Rust 签名推导返回 `int`（句柄 id 栈载荷），.at 公共面声明 `Response` | production() 尾部按 §5.3「返回按公共身份」覆盖重声明 |

## 2. 既有契约（无需补，2026-10-10 核对）

http.get/get_stream/request/transfer_download/file_response/transfer_wait/error/cancel/upload_metadata/upload_error、
json.is_valid/encode/decode、方法面 http_stream stream_next/stream_is_done（声明名 `auto.http.*` 与注册名
`auto.http_stream.*` 经 catalog 同 id 2242/2243 统一——别名错位疑点**解除**，非缺陷）。

## 3. 知情登记：resolved-but-unbound（13 项，非契约缺口，另案）

Request.{method #9923, path #9924, version #9925, query #9926, query_string #9927, header #9928,
body #9929, text #9942, param #9943, content_length #9944, content_type #9945} 与
Response.{status_code #9957, header_get #9958}——公共名解析到 registry id 但 production 接口未在该 id 绑 shim
（运行时分派走 CALL_SPEC 类型名 `Response.status_code` 等，注册于 stdlib.rs:9756-9764）。若某发射路径
落到公共名 id 会以 PROVIDER_CLAIM_NO_CALLEE 拒绝（与 SIGNATURE_UNVERIFIED 不同类）；plan738 t04
已冻结该诚实状态（"公共面全量重写下轮"），755 不在契约侧越权补（无 producer 可证）。

## 4. 修复落点与验证

- `crates/auto-lang/src/vm/native.rs` production() 尾部 +46 条声明（41 新增 + 5 覆盖）；
  新增 `declare_generic_method_contract_identity`（receiver+generics 同时声明）。
- 走查守卫：`plan755_http_public_face_contract_coverage`（缺契约/漂移即红，covered≥20 空转守卫）
  与 `plan755_http_contracts_survive_cffi_merge`（AutoVM::new 同款 merge 后契约存活）。
- 修复后：两测试 PASS（0.10s/0.11s）；原反例 `http_e2e_plan730_vm_plain_endpoint_untouched` 与
  `e2e_a_redirect_302_with_location` 经真实 `crate::run` 启动并通过 wire 断言（0.59s 各，T-04）。
