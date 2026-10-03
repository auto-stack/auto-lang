# API 生成完整性——strict 实现、显式 scaffold、ready 记录与错误传播

> 路径：`crates/auto-man/src/api_gen.rs`（strict 门/ready 记录/FNV）+
> `rust_ui.rs`（新鲜度门）+ `vue.rs`/`tauri.rs`（错误传播）
> 状态：active（PLAN-734 交付；报告见 `docs/plans/reports/734-api-{generation,verification}.md`）

## 1. strict 解析门

- `try_full_parse` 失败 → `Err`（解析错误不再静默落 lenient/空提取）。
  lenient 提取退役为显式 `AUTO_API_LENIENT=1`（产物无业务体，scaffold 语义）。
  零端点且解析成功 = 合法 no-API（Ok——实现型 api.at 不受影响）。

## 2. 真实实现优先

- a2r 内联体转译失败 → 该端点生成 `Err`（`endpoint '{fn}' body transpile
  failed: ...`）——CRUD 模板/Default 桩 fallback 链退役（Plan 400 路径）。
- `AUTO_A2R_BODY=0` = **显式 scaffold 模式**：CRUD 模板可达但 api.rs 头部强制
  `// PLAN-734: SCAFFOLD MODE` 标注（测试锁定——不冒充同源）。db.rs 委派
  保持为合法实现来源。
- tauri 后端：HTTP 专属种类（File/Upload）端点在命令生成前被矩阵拒绝
  （结构化错误 + HTTP URL 指引）。

## 3. ready 记录与新鲜度门

- `generation.json`（schema_version/endpoints/source_hashes[api.at,db.at 的
  FNV-1a]/scaffold/generated_at）随 rust bundle 最后写入。
- `start_api_server` 的 `AUTO_REUSE_BACKEND=1` 复用臂经 `backend_generation_is_fresh`
  校验：ready 缺失/解析失败/源指纹失配 → 拒绝复用，回落 kill+重生成+spawn。

## 4. 错误传播（8 入口全部 `?`/Err）

vue.rs（VueProject::generate 的 `let _ =` 静默丢弃/prepare_vue_sources/run 的
warn×2）、rust_ui.rs run_vm_ui split（warn 后仍 start_api_server）、tauri.rs ×2、
tauri_backend——全部硬传播；rust_ui.rs start_api_server/build_rust_ui 既有硬传播保持。
生成失败不以旧产物继续启动。

## 5. 测试基建注意（R1 实证）

- `tauri` dev-dep 必须 `default-features = false, features = ["test"]`——
  默认 features 引入 wry/WebView2Loader，测试二进制在本机
  STATUS_ENTRYPOINT_NOT_FOUND（mock_builder/generate_handler!/get_ipc_response
  全链实测形态见 `plan734_tauri_mock_dispatcher_invokes_generated_command` +
  fixture 锁测试）。
