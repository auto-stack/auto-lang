# FileResponse a2r 降级契约（PLAN-729）

> 路径：`crates/auto-lang/src/trans/rust.rs`（lowering + 类型映射）；
> golden `crates/auto-lang/test/a2r/32_plan729/001_http_file_response/`
> 状态：active（SD-06；同源实编证明于 auto-man 生成服务 e2e）

## 1. 构造发射

`http.file_response(root, rel, options)` → `a2r_std::http::file_response(<root>,
<rel>, <options>)`：三实参 `expr_as_str`（String 标识自动 `.as_str()`），options 为
字符串字面量时**字节保真透传**（严格 JSON 校验在描述符构造器，不在发射层）。
sync/async 上下文**同形**（构造零 I/O，无 async 变体）。

## 2. 类型映射

- `FileResponse`（User/参数/返回位）→ `a2r_std::http::FileResponse`（全限定，
  StringBuilder/SqliteDb 先例；`qualify_type_name` 与 `rust_return_type_name` 双点）。
- 返回位是**具体类型**：PascalCase trait 启发式对它不适用（不产 `impl FileResponse`）。
- `~FileResponse`（`Future<FileResponse>`）：异步 handler 语义沿用 705——future 终值
  经 VM 编组门识别（见运行契约），a2r 独立产物不涉。

## 3. 生成消费（api_gen 文件分支）

- 声明 `FileResponse`/`Future<FileResponse>` 返回的 `#[api]` 端点：handler 签名
  `-> axum::response::Response` + `method: axum::http::Method` +
  `headers: axum::http::HeaderMap` 提取器（先于 Json）；体尾 `return X;` →
  `return __file_reply(X, &method, &headers).await;`（宿主 serve + 版本无关
  FileReply → 本地 axum Response）。
- 委派路径（route A/db 覆盖）同形 + 首个路径参数 `Path<T>` 提取（`&name` 传参）。
- 非 GET/HEAD → 405 诊断 handler（生成期 eprintln 位置信息）；转译失败 → 500 诊断
  handler（**不落模板**）。GET 文件路由自动 `.head()`（显式 HEAD 路由优先）。
- 同名用户函数不受影响（分支以**声明返回类型含 `FileResponse`** 为门，非名字）。

## 4. 运行契约引用

stdlib [http-server-files](../../../stdlib/design/http-server-files.md)（描述符/
协议/配额/支持矩阵）。
