# PLAN-730 验证报告（T-09 分级门禁）

- 最终代码 hash：`12c508341`（worktree `D:/autostack/.wt/lang-730/auto-lang`，分支 `plan-730-dev`；
  master 基线 `7d50989f7a`——729 已 merged 后开工）。
- 平台：Windows 10 x64（win32 10.0.26200）；Linux CI 侧同断言集（volume_key/dev、EEXIST 语义同源）。
- 依赖变化：零新增 crate（multer 拒绝理由见决策报告 §4.8；`futures = "0.3"` 在生成 crate
  模板中由 SSE 条件改为无条件——生成产物依赖，非本仓依赖）。

## 1. 门禁全表（§6.3；全部本 worktree 实跑）

| 门禁 | 命令 | 结果 |
|---|---|---|
| 快速类型 | `cargo check -p auto-lang` / `-p a2r-std` / `-p auto-man --lib` | 0 error；我的文件 0 warning |
| feature 隔离 | `cargo check -p auto-lang --no-default-features` | 4 error 预存（711 frame_bench `crate::ui` 引用，非 730 文件——预存债 P730-D1） |
| scoped | `cargo t plan730` | 23/23 |
| a2r-std 串行 | `cargo test -p a2r-std -- --test-threads=1` | 87+7+6 全绿 |
| auto-man 串行 | `cargo test -p auto-man api_gen:: -- --test-threads=1`（全 lib） | plan730 5/5；全量 339/342（3 预存，见 §2） |
| VM e2e | `cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan730` | 10/10（含 38s 慢上传） |
| 生成 e2e | `cargo test -p auto-man --lib --features test-http-e2e plan730_e2e` | 1/1（真实 cargo build+run） |
| back_proxy | `http_e2e_back_proxy_upload_endpoint_rejected_501` | 1/1 |
| 复审裸 `cargo t` | 全日常档 | 1479/1482（3 预存） |
| `cargo tv` | 语料三族 | 162/162 |
| `cargo tt` | trans 档 | 1869/1872（3 预存） |
| `cargo th --test-threads=1` | 真 TCP 串行 | 92/93（1 预存） |
| docs_gen / tu / taa / tf | 未触发 | 未改 schema/文档生成器/ui_gen/aavm；tf 归批量回归档（730%5=0 → 到期，merge 后由 /auto-plan:regress 主检出执行） |
| 格式 | 我的文件 fmt 通过 | 无关文件预存未格式化（未动） |
| 残留扫描 | dbg!/todo!/unimplemented! | 0；eprintln 全部带 `[upload730]`/`[HTTP]` 受控前缀 |

## 2. 预存红逐名对照（零新增确定性红）

| 红名 | 档位 | 基线核对 |
|---|---|---|
| musk p053 widget_computed ×2 + p054 merged_api_warning | t/tt | 在案 14 预存红族（729 verification §2 同名） |
| back_proxy_real_routes_corpora_data_face | th | **master 主检出实测同名红**（本会话 stash/主检出双核对） |
| auto-man: merged_api_client_crud_fallback / test_shell_pack_lib_freshness / index_css_values_match_p1_baseline | auto-man lib | **master 主检出实测三红同名** |
| plan724_sync_client_matrix | e2e（长套负载下偶发） | 隔离运行两态均绿（4f123a50e 在案 flake 族） |

## 3. AC 对照（§7）

| ID | 结论 | 证据 |
|---|---|---|
| AC-01 公共类型+两腿命名 handler | ✅ | a2r-std 87 测 + VM e2e 201 + 生成 e2e（真实编译运行）+ 金样 33_plan730 + 普通 int 反例（plain 730730 / 编组门反例测试） |
| AC-02 parser/预算正确、普通 JSON 额度不变 | ✅ | parser 6 矩阵测试（逐字节切块/假前缀/文件名变体/错误 8 形/预算 ±1/重复字段保序）+ wire 413（声明 CL 读前拒）+ 普通路由 10MiB 桥行为原样 |
| AC-03 授权先于存储、staging 私有、root 限制 | ✅ | 早拒 401（100-continue 只发 headers）+ 404/middleware 零 staging（e2e）+ 嵌套/缺失/跨卷配置读取前拒 + 目标矩阵 403 + hard_link create-only 探针 |
| AC-04 有界 async intake/期限 | ✅ | active/queue/fs_ops/pending 信号量 + lease 300ms 旋钮 + 38s 慢上传跨 30s 期限成功（watch 重臂）+ 无每请求线程/runtime（kernel 共享） |
| AC-05 两阶段提交/取消/故障收口 | ✅ | Received 后 reject 无最终文件 + 同名仅一成功（409 原文件不变）+ lease 过期墓碑 410 + 迟到取消计数探针 + writer 自清/staging 回基线断言 |
| AC-06 legacy 保持+缺口清偿 | ✅ | B6 e2e 成功形状 PASS + 404/中间件零写盘 + 写失败真实 500 + 绑定失败清理 + JSON/SSE/729 回归（t/th 档全绿面） |
| AC-07 生成/客户端消费与 727+729 互通 | ✅ | 转译失败诊断 500（不落模板）+ TS FormData 直传 + Tauri Unsupported + back_proxy 501 + 互通闭环（upload→download→transfer_download 同字节；409/422 客户端失败面） |

## 4. SD 对照（§5 规范增量）

SD-01..08 的沉淀稿随本报告交付（canonical 沉淀在 merge 阶段执行）：SD-01 上传契约
（server_upload + http_upload_service + 两桥）、SD-02/08 http-server.md typed/legacy 区分
+ legacy 落盘顺序/错误规则、SD-03 backend-assembly/stdlib project 上传公共声明、
SD-04 async-lifecycle body capability/staging 资源组/分阶段期限/仲裁、SD-05 a2r-std
上传 owned 类型与 hook、SD-06 http-upload-lowering 新增、SD-07 trans overview/api-man
生成消费面。

## 5. 已知债（候选 KNOWN-DEBT）

- **P730-D1** `--no-default-features` 预存 4 error（711 frame_bench 的 `crate::ui` 引用，
  a2r_std.rs:1109/1114 + native.rs:891/900）——先于 730 存在，未扩大。
- **P730-D2** 中间件空串 `""` 返回在实现中按 JSON 响应短路（Plan 352 文档写"空继续"）——
  语义分歧先于 730；730 示例/测试采用 `?str` nil 放行形态并在 lifecycle 报告注记。
- **P730-D3** 独立 axum 生成器的上传 glue 实编 wire 验证复用 auto-man fixture 腿
  （同 729 惯例——该生成器输出无独立实编档）。
- 接收中（未 staged）VM 取消依赖 body 流终结（断连自然发生）——lifecycle 报告 §5 如实记录。

## 6. 复审 R1 回工后门禁（回工提交见分支）

- 定向面：`cargo t plan730` 26/26；e2e 全族 `http_e2e_plan730` 14/14
  （新增 chunked/quoted-wire/配额+health/断连/total 五用例）。
- 回归：裸 `cargo t`（预存 musk p053 族 2-3 红同名）、`cargo tv` 162/162、
  `cargo tt`（同 3 预存）、a2r-std 串行全绿（首轮 1 传输族 flake 复跑绿——
  plan724 同族）、auto-man plan730 5/5。
- 复审发现并修复的实现缺陷：准入期限误用 total（§lifecycle 6）；
  测试数据事故两起（heredoc 转义损坏 0B 用例体、批量替换误伤 wire-caps 的
  CL 值）均已修正并在位复跑——不改变 R1 结论的发现归属。
