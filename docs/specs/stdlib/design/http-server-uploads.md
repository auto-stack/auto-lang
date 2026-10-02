# HTTP 服务端上传（multipart/raw 接收面）——stdlib 服务端上传契约

> 路径：`crates/a2r-std/src/http/server_upload.rs`（owned 类型/严格 options/收据/
> async facade→宿主 hook）+ `crates/auto-lang/src/http_upload_service.rs`（宿主执行：
> 增量 parser/staging/发布/取消仲裁/lease）+ `vm/ffi/http_upload.rs`（VM 桥）+
> auto-man/api_gen 上传分支（生成 Rust 腿）
> 状态：active（PLAN-730 交付；报告见 `docs/plans/reports/730-upload-*.md`）

## 1. 公共面（Auto / VM / 原生 Rust 同词汇）

```text
type UploadRequest / UploadSession / UploadReceipt        # opaque，无用户构造面
http.upload_receive(req UploadRequest, root str, staging_root str, options str) UploadSession
http.upload_metadata(session UploadSession) str
http.upload_commit(session UploadSession, relative_target str) UploadReceipt
http.upload_reject(session UploadSession, status int, message str) UploadReceipt
http.upload_error(status int, message str) UploadReceipt
```

- `UploadRequest` 只能由宿主注入（VM 桥按路由声明建 / 生成腿 `Request` 提取器投影）；
  一次性消费；请求字段不能指定 root/staging_root/最终 target。
- receive/commit/reject 为 await 点（VM park 经 live-op；Rust `.await`；同步上下文
  指名诊断）；metadata/error 纯同步零 I/O。上传端点=POST/PUT + `UploadRequest` 参数
  + `UploadReceipt`/`~UploadReceipt` 返回（加载/生成期诊断其他形态）。
- 收据 = 真实 HTTP status + 有界 JSON（`{"ok":true,"path","size":"十进制串",...}` /
  `{"ok":false,"kind","message"}`）；成功 commit=201；receipt 不持在途资源。
- metadata：`{"state":"received","mode","field","filename","size":"串","content_type",
  "fields":[有序含重复]}`；失败会话 `{"state":"failed","kind","message",
  "suggested_status"}`——`upload_reject(session, 0, ..)` 采纳建议状态。
- `upload_error` 状态域 400..=599（域外→500 诊断收据）；早拒零 I/O、不能伪造 201。

## 2. 接收合同（§5 语义）

- **顺序**：路由/中间件/header 校验先于任何 body 解析或落盘（早拒无需发体——
  `Expect: 100-continue` 实测）；上传路由 middleware 执行错误 fail-closed 500。
- **延迟 body**：VM 桥按方法+参数类型双条件判定（不按 Content-Type 猜），原始流
  随请求入队；生成腿 `axum::extract::Request` 置最后、不预读。
- **mode 显式** multipart|raw；multipart=RFC 7578 单文件子集（恰 1 个声明 file
  字段 + 有序有界文本字段；缺 file/第二 file/未知字段/嵌套/CTE/非 utf-8 charset
  =明确错误；boundary ≤70B；假前缀跨块回吐为内容）；raw=整 body 为文件
  （Content-Encoding 非 identity→415）。
- **预算**（默认，可 env `AUTO_HTTP_UPLOAD_*` 或 options 下调）：单文件 64MiB、
  wire 65MiB（含 framing/epilogue）、文本 ≤16 个·单 16KiB·合计 64KiB、part ≤32、
  part headers ≤16KiB·≤32 项、块 64KiB、待写块 2、fs_ops 4。
- **期限**：准入（queue+active 等待）受 queue 期限（默认 30s，**不是 total**——
  满额下一笔 503 于期限内）；接收 idle 60s；total 10min（headers 入 scope 起，
  排队计入）——VM 腿经 scope deadline watch 切换 30s 普通请求期限；staged lease 30s。
- **配额**：active 4 / queue 16（不读 body 不开文件）；staged 占 active 至终态。

## 3. staging 与提交

- staging 在受信 `staging_root`（同卷、公开 root 之外；嵌套/缺失/跨卷在读取 body
  前拒绝）；`create_new` 独占（unix 0600）；EOF+flush+sync 后才 staged。
- commit：目标词法校验（复用 server_file 的 `validate_relative_path`）→ 父目录
  no-follow walk（仅已存在目录）→ **`hard_link` 原子 create-only 发布**（存在即
  409 原文件不变；EXDEV=配置诊断；不支持 no-replace 的 FS 明确失败）→ unlink
  staging。客户端 filename 绝不作存储路径。
- 取消仲裁：gate（hard_link）前取消胜出（不发布+清理）；gate 后迟到取消等实际
  结果（成功保留——不回滚已发布文件）；断连经连接终结级联；writer 通道关闭未
  Finish 自清 staging。
- 清理失败记 request id + 受限诊断（不宣称零残留）；lease 到期自动清理 + 过期
  墓碑（迟到 commit 真实 410）。

## 4. 调用形态支持矩阵

| 形态 | 状态 |
|---|---|
| VM 默认 `#[api]` POST/PUT（sync/`~`） | ✅（`UploadRequest` 类型优先注入——先于 path/body/query/meta 规则） |
| auto-man 生成 Rust HTTP | ✅（同 api.at 生成 adapter；转译失败=诊断 500 不落模板） |
| `api/targets/axum` 单独生成器 | ✅ 生成（实编验证于 auto-man fixture 腿） |
| TypeScript HTTP 客户端 | ✅ `body: FormData` 直传（浏览器产 boundary；不 JSON.stringify） |
| Tauri IPC / back-proxy | ❌ 明确 Unsupported/501 + HTTP URL 指引 |
| 727 客户端 / 729 下载 | ✅ 互通闭环（upload→201→file_response 路由→transfer_download 同字节） |
| 多文件事务/断点分片/配额持久化 | ❌ 非本期面（计划非目标） |

## 5. legacy multipart 兼容（本计划修复面）

旧 `form str` 路径保留 10MiB 上限与 `{"fields","files"}` 成功形状；**顺序修复**：
路由先行（404/方法不匹配零解析零写盘）→ 解析（纯内存）→ **落盘在 middleware 后**
经宿主 spawn_blocking（owner 零阻塞）→ 错误传播（写失败真实 500，不返回假路径）；
provisional 文件在绑定失败/handler Err/取消时清理，成功按历史语义保留。

## 6. 跨实现一致性（单源）

类型/options/收据/错误映射只在 `a2r_std::http::server_upload`；parser/staging/发布/
仲裁只在 `auto_lang::http_upload_service`（无 axum 类型，两代 axum 消费端各自投影）；
VM 桥（natives 9937-9941 + 收据三重编组门）与生成腿（trans lowering + api_gen
UPLOAD glue）只做投影。收据/会话/注入句柄均登记 scope 资源组（finalize 幂等收口）。
