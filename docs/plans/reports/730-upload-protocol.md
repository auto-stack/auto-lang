# PLAN-730 T-08 协议报告：双端上传 wire 矩阵

- 代码基线：worktree `D:/autostack/.wt/lang-730/auto-lang` @ `8a2fc567e`（分支 plan-730-dev）。
- 命令（Windows 实机）：`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan730`（VM 腿）；
  `cargo test -p auto-man --lib --features test-http-e2e plan730_e2e -- --test-threads=1`（生成 Rust 腿）。
- 端口分配：VM 18970-18980；生成腿 18976（crate 名 plan730-gen-e2e 隔离产物；
  共享 worktree target 缓存，增量编译后整测 16s）。

## 1. VM 默认 HTTP 腿（`plan730_http_upload_tests::http_e2e`）

| 用例 | 结果 | 关键断言 |
|---|---|---|
| multipart 上传→commit→下载 | PASS | 12MiB（>普通 10MiB）201；receipt size/path/fields 精确；磁盘字节相同；729 路由 GET 200 同字节；同名再传 409 且原文件不变 |
| raw 上传 + 业务拒绝 | PASS | raw 201 字节相同；note=deny → 422、无公开文件、staging 清零 |
| 鉴权先于存储 | PASS | 404 零 staging；`Expect: 100-continue` 只发 headers → 401 立即回（无需发体）；middleware 短路零落盘 |
| wire 上限/畸形 | PASS | 声明 CL 100MiB → 413（读体前）；截断 multipart → 400 无发布 |
| >30s 慢上传 | PASS | 38s 受控慢发仍 201（scope deadline watch 切换 30s→上传总期限） |
| 普通端点并存 | PASS | 普通 int 端点 200 `730730`（上传门不误判） |
| legacy 中间件拒绝 | PASS | 拦截请求零写盘；成功请求 1 文件 |
| legacy 写失败 | PASS | AUTO_UPLOAD_DIR 不可建 → 真实 500（错误信息含原因，非假路径） |
| legacy 绑定失败 | PASS | 双缺参 400 → provisional 文件清理（目录空） |
| 727 客户端互通闭环 | PASS | transfer_upload(multipart+note) → 201 磁盘同字节；729 路由 + transfer_download 同字节；raw 201；冲突 409=客户端 Failed；业务拒 422=Failed；staging 清零 |

## 2. 生成 Rust/Axum 腿（auto-man `plan730_e2e`，真实编译运行）

| 用例 | 结果 | 关键断言 |
|---|---|---|
| 生成上传矩阵 | PASS | 生成 crate cargo run（axum 0.7 + auto-lang + a2r-std）；multipart 2MiB 201 + receipt；staging 清零；729 下载路由同字节；409 原文件不变；note=deny 422；raw 201；404 零落盘 |

## 3. 双端一致性来源

两腿共用：a2r-std `server_upload`（类型/options/收据/facade）+ `auto_lang::http_upload_service`
（增量 parser/staging/hard_link 发布/取消仲裁/lease）。差异仅在桥：
VM 腿 bridge deferral（方法+类型双条件）+ scope 期限 watch + 收据三重编组门；
生成腿 `Request` 提取器置最后 + `__upload_request` 投影 + `__upload_reply` 真实 status。

## 4. 兼容回归

`e2e_b6_multipart_upload_field_and_file`（B6 legacy 成功形状）PASS——解析与落盘
分离后 JSON 形状逐字节保持；plan729 全族 e2e PASS（见 verification 报告回归节）。
