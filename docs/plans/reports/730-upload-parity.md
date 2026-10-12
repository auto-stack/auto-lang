# PLAN-730 T-08 对等报告：单源投影与两腿差异面

- 代码基线：@ `8a2fc567e`。

## 1. 单源投影表（语义只写一处，消费端只做投影）

| 语义 | 单源 | VM 腿消费 | 生成 Rust 腿消费 |
|---|---|---|---|
| 类型/严格 options/限额/收据 JSON/错误种类→status | `a2r-std/src/http/server_upload.rs` | shim 经 facade | 转译体直调 facade |
| 增量 multipart parser/raw 接收/预算 | `auto-lang/src/http_upload_service.rs`（executor） | `receive_with_phase`（hook 透传） | a2r facade → executor hook |
| staging 写入/自清 | 同上 writer 任务 | 同 | 同 |
| create-only 发布/取消仲裁/lease/墓碑 | 同上 commit/reject/watchdog | 同 | 同 |
| 受信目录校验（同卷/嵌套/存在） | 同上 `validate_roots` | 同 | 同 |
| 目标词法校验 | `a2r-std server_file::validate_relative_path`（729 复用） | facade/executor 双层 | 同 |
| 类型映射（Auto→Rust） | `trans/rust.rs`（a2r_std::http:: 全限定） | — | 生成 api_impl/api.rs |
| 收据→HTTP 回复 | — | `upload_receipt_reply`（receipt.status+JSON+CORS+请求 id） | `__upload_reply`（同形胶水） |

## 2. 桥差异面（各自归属，不复制协议决策）

| 面 | VM 腿 | 生成腿 |
|---|---|---|
| body 延迟 | bridge 分类（POST/PUT+参数类型）→ `ApiRequest.raw_upload_body` → start_handler 建 `UploadRequest` | `axum::extract::Request`（最后提取器，不预读）→ `__upload_request` 投影 |
| 鉴权先于 body | owner 中间件/早拒在 receive 前；middleware 执行错误 fail-closed 500 | handler 提取器序（method/headers 先）；应用鉴权在 handler 体内 receive 前 |
| 期限 | scope deadline watch（30s→total 切换） | executor 内部 idle/total 计时（无 scope 概念） |
| 等待语义 | live-op park/resume（sync/async handler 同形） | async fn `.await`（转译层保证 async 上下文，同步上下文指名诊断） |
| 注入参数识别 | `bind_api_args_by_name` 类型优先（防 req meta 名/whole-body 吞参） | `endpoint_body_params` 排除 + 提取器签名生成 |
| CORS/请求 id | `upload_receipt_reply` 追加 | 生成服务全局 CorsLayer（tower_http） |

## 3. 消费形态矩阵（§6 决策 §6）

| 形态 | 状态 | 证据 |
|---|---|---|
| VM 默认 `#[api]` POST/PUT（sync/`~`） | ✅ | e2e 10 用例（含 38s 慢上传、100-continue 早拒） |
| auto-man 生成 Rust HTTP | ✅ | 真实编译运行 e2e（multipart/raw/409/422/404/729 互通） |
| 独立 axum 生成器 | ✅ 生成（`__plan730_upload_*` 同形胶水 + main 安装；实编 wire 验证于 auto-man fixture 腿——同 729 惯例） | api_gen/axum.rs 生成串 |
| TypeScript HTTP 客户端 | ✅ `body: FormData` 直传（浏览器产 boundary；不 JSON.stringify）；返回收据 JSON | `test_plan730_upload_ts_formdata` |
| Tauri IPC | ❌ Unsupported 诊断（Err + HTTP URL 指引） | `test_plan730_tauri_upload_unsupported` |
| 进程内 back-proxy | ❌ 501（注入能力非可序列化数据） | `http_e2e_back_proxy_upload_endpoint_rejected_501` |
| 727 客户端 | ✅ transfer_upload → 201/409/422 端到端（失败=非 2xx 终态，默认不重放） | `http_e2e_plan730_client730_server_interop` |
| 729 下载 | ✅ 上传后同文件下载同字节（两腿） | 同上 + 生成 e2e |

## 4. 复现命令

见 `730-upload-protocol.md` 头部；对等面逐项的生成串锁定测试在
auto-man `api_gen::tests::test_plan730_*`（5 串测）。
