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

### 装配指纹与收据身份（PLAN-738）

- generation.json 收据 assembly 块（schema 4 双指纹）：业务 source_hashes
  与 assembly 指纹各自记录；收据/烘焙常量（`__assembly_fingerprint`）/
  在途核对/新鲜度门统一使用 **consumer_fingerprint**（同消费者跨时语义
  与 schema 3 兼容）。
- 生成装配引用闭包：endpoint 内联体 + db.at + 伴生 `src/back/*.at` back
  模块全部按 **Embedded** 运行形态转译收集证明并入 manifest（back 产物经
  qualify_a2r_std 链接内嵌镜像，不把 endpoint 面冒称全部装配）。

### workspace lock 一次绑定（PLAN-738 R5-01）

- ready 记录 `workspace_lock`（生成产物运行时依赖输入的 FNV 身份；未建
  lock 记 absent）——与生成器自身 Cargo 输入（manifest provider 面）分开
  记录。
- **身份分类三态单点（PLAN-751，P738-R11-01）**：NotFound=未物化（absent）；
  成功读取（含零字节空文件）=实际内容身份（FNV 指纹，空文件=`811c9dc5`，
  **不得与 absent 合并**）；其它读错误=不可核验（生成端拒绝写收据、复用
  门保守判陈旧）。生成端写收据与复用门对拍**共用同一分类实现**
  （`rust_ui.rs::workspace_lock_identity`），禁止两端口径漂移——R11 反例：
  生成端记指纹、复用门归 absent，空 lock 再生成成功后未变输入仍被判
  stale。
- 复用门为**一次绑定状态机**：absent → 首次出现实际 lock 时判新鲜并把
  实际身份**绑定写回收据**（一次性收敛；写失败保守判陈旧）；此后严格
  比较——依赖版本漂移/lock 删除/读取失败/旧收据缺字段均陈旧走再生臂。
  纯 absent 豁免（永久放行后续漂移）是 R5 复审反例，禁止回退。

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
