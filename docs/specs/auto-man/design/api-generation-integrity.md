# API 生成完整性——strict 实现、显式 scaffold、ready 记录与错误传播

> 路径：`crates/auto-man/src/api_gen.rs`（strict 门/ready 记录/FNV）+
> `rust_ui.rs`（新鲜度门）+ `vue.rs`/`tauri.rs`（错误传播）
> 状态：active（PLAN-734 交付；PLAN-776 修订——alias 目标链/限定名来源
  分类输入；报告见 `docs/plans/reports/734-api-{generation,verification}.md`）

## 1. strict 解析门

- `try_full_parse` 失败 → `Err`（解析错误不再静默落 lenient/空提取）。
  lenient 提取退役为显式 `AUTO_API_LENIENT=1`（产物无业务体，scaffold 语义）。
  零端点且解析成功 = 合法 no-API（Ok——实现型 api.at 不受影响）。

## 1b. 返回分类输入（PLAN-776）

- `ApiEndpoint.response` 由 `ResponseKind::from_return_identity(ret,
  module_types)` 冻结（提取期两遍判定）；**alias 目标携完整形态**
  （`type D = Future<FileResponse>` → File，与直接 `Future<FileResponse>`
  同规则）+ **visited 集链展开**（合法深链不限层、真环保保守 Json）；
  `http.` 限定名先于用户名集（不受同名遮蔽）。api_gen/targets/
  back_proxy 消费同一冻结身份（back_proxy fn_meta：target 全形存储 +
  单步规范化 + visited 集——与 VM 分类单源同语义），不各自裸串分类。
- 用户同名遮蔽三层（提取级/codegen TypeStore→`API_RETURN_SHADOWED`/
  back_proxy fn_meta）；strict 门/738 双指纹/751 三态单点不变。

## 2. 真实实现优先

- a2r 内联体转译失败 → 该端点生成 `Err`（`endpoint '{fn}' body transpile
  failed: ...`）——CRUD 模板/Default 桩 fallback 链退役（Plan 400 路径）。
- JSON handler 业务体按形态分流：简单委派走已解析的 `db.rs` 调用；带控制流或
  嵌套实参等非薄体走 a2r 内联转译。Plan 742 补齐该路径的 `List<T>.new` 构造、
  companion module 导入/限定调用、借用参数修正和 `?T` Result/404 返回包装；
  这些修正只覆盖相应发射形态，不扩大语言转译器的通用支持面。装配预检对
  body 转译失败 fail closed，拒绝生成；运行时生成阶段不得静默落回 CRUD 桩。

### 体路径不变量（Plan 742 r1–r5，六轮复审冻结）

- **字面量保持**：api.rs/companion 后处理链（语句分割/集合降级/模块限定/
  借用修正/标准库限定/strip_collection_new）只作用于代码区——`code_region_mask`
  转义感知掩码（字符串/字符/行+块注释）门控，业务字符串载荷逐字节保留；
  标准库限定为单趟优先级扫描（已限定 > sys/sqlite 直连 > 通用限定 > use
  行），无占位符恢复步（恢复步无法区分自插入 marker 与同拼写合法串）。
- **模块 token 身份**：companion 限定化按调用形态（`mod.fn(`）且要求标识符
  边界——代码区命中 ≠ 完整 token 身份，名字以 stdlib token 结尾的合法
  companion（`custom_a2r_std`）保持自身模块身份；绑定按真实作用域遮蔽
  （栈式帧：params 帧起、每个代码区花括号一帧），子块局部只在块内遮蔽，
  外层同名模块调用不受影响。
- **借用**：按实际签名与实参切分补 `&`；比较 `<`/`>` 不是泛型定界——判别
  取发射器形态规律（泛型 `<` 紧贴标识符、比较两侧带空格）；嵌套调用参数
  逐层递归修复（外不需借/内需借组合必须正确）。
- **返回出口矩阵**：值/Option/void × has_path/PUT/DELETE 各形态类型匹配
  （`InlineRetWrap` 五态）：Option 变量按运行时 match 映射 Some→200/
  None→404（不得按 Some/None 拼写判定类型）；void 体 `return None;` 映射
  状态值、无顶层 return 时合成尾（分支内早退不覆盖 fallthrough）；File/
  Upload 体入口共享同一限定化；失败不能假成功（生成期 fail closed 保持）。
  永久 oracle：真生成 → rustc 实编 → 运行 → 值对拍矩阵（api_gen tests
  内联），不是子串断言。
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
