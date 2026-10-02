# HTTP 上传 typed lowering——trans/rust.rs 降级契约

> 状态：active（PLAN-730 交付；金样 `crates/auto-lang/test/a2r/33_plan730/
> 001_http_upload`；报告见 `docs/plans/reports/730-upload-{decision,parity}.md`）

## 1. 类型映射（全限定，同 FileResponse 先例）

`UploadRequest` / `UploadSession` / `UploadReceipt` → `a2r_std::http::{...}`
（参数位 `rust_type_name` 与返回位 `rust_return_type_name` 双面；`~UploadReceipt`
解包后的返回位同映射；PascalCase 启发式不得误判为 trait）。

## 2. 自由函数族（`("http", name)` 分派）

| Auto 调用 | 发射 | 形参收敛 |
|---|---|---|
| `upload_receive/commit/reject` | `a2r_std::http::{name}(<args>).await` | facade 形参 `impl AsRef<str>`——字面量/String/&str 直传（**不**发射 `.as_str()`：字面量上该方法不稳定 E0658） |
| `upload_metadata` | `a2r_std::http::upload_metadata(&x)` | 同步 |
| `upload_error` | `a2r_std::http::upload_error(status, msg)` | 同步 |

- receive/commit/reject 是 await 点：**同步上下文（非 `~` 函数体）指名诊断**，
  不发射伪同步调用（错误文案携带"declare the handler with a ~UploadReceipt return"）。
- api_gen 的 `try_transpile_body` 对 `Future<T>` 返回端点置位 async 上下文
  （`set_async_ctx`）——内联体 await 合法。

## 3. 生成腿消费（api_gen/api targets）

- 上传端点（参数含 `UploadRequest`）生成：method/headers 提取器在前、
  `request: axum::extract::Request` **最后**（不预读、不过早鉴权）；
  `__upload_request` 投影（Request → 版本无关 `UploadBodyStream`）+ `__upload_reply`
  （收据 → 真实 status + JSON）；非 POST/PUT → 405；转译失败 = 诊断 500
  （不落 CRUD 模板）。`endpoint_body_params` 排除注入参数（不进 JSON body 结构）。
- 独立 axum 生成器同形（`__plan730_upload_*`）；main 模板安装宿主 executor（幂等）；
  生成 crate 依赖模板 `futures` 无条件（上传 glue 与 SSE 共用）。
- TS 客户端：注入参数从签名删除、新增 `body: FormData` 形参直传（浏览器产
  boundary；不设 Content-Type 头、不 JSON.stringify）；返回 `response.json()`
  （非 2xx 也携带收据 JSON）。Tauri IPC = Unsupported 诊断；back-proxy = 501。

## 4. 边界

- `UploadRequest` 无用户构造 native（不能从 JSON/query 伪造）；字节流不能从
  JSON 参数填造；VM 腿同词汇经 natives 9937-9941（park/resume），与生成腿共享
  同一宿主执行——两腿不复制协议决策。
