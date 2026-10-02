# PLAN-734 T-04/T-06 生成报告：strict 实现、原子发布与错误传播

- 代码基线：@ `438d22c47`。命令：`cargo test -p auto-man api_gen:: -- --test-threads=1`。

## 1. strict 解析门（D4）

full-parse 失败 → `Err`（带文件名与行动指引）；lenient 提取退役为显式
`AUTO_API_LENIENT=1`（产物无业务体，scaffold 语义）。零端点且解析成功 = 合法
no-API（Ok）——实现型 api.at（Plan 559）不受影响。

## 2. 真实实现优先（AC-04）

- a2r 内联体转译失败 → 该端点生成 `Err`（`endpoint '{}' body transpile
  failed: ...`）——CRUD 模板/Default 桩 fallback 链退役（Plan 400 路径）。
- `AUTO_A2R_BODY=0` 重语义 = **显式 scaffold 模式**：CRUD 模板可达但 api.rs
  头部强制 `// PLAN-734: SCAFFOLD MODE` 标注（测试断言锁定——不再冒充同源）。
- db.rs 委派（resolve_db_call 命中）保持为合法实现来源。
- tauri 后端：HTTP 专属种类（File/Upload）端点在命令生成前被矩阵拒绝
  （结构化错误 + HTTP URL 指引）。

## 3. ready 记录与新鲜度门（AC-05）

- `generation.json`（schema_version/endpoints/source_hashes[api.at,db.at 的
  FNV-1a]/scaffold/generated_at）随 rust bundle 最后写入。
- `start_api_server` 的 `AUTO_REUSE_BACKEND=1` 复用臂新增
  `backend_generation_is_fresh` 校验：ready 记录缺失/解析失败/源指纹失配 →
  拒绝复用，回落 kill+重生成+spawn（旧产物不冒充当前契约）。
- 写盘顺序保持（main.rs 后 ensure workspace）；错误在写盘前终结 = 零写或
  尽力清理 + Err（旧 bundle 保留但不被启动）。

## 4. 错误传播 8 入口（D6 表；全部 `?`/Err 硬传播）

| 入口 | 原 | 现 |
|---|---|---|
| vue.rs VueProject::generate | `let _ =` 静默 | `?` |
| vue.rs prepare_vue_sources / run_vue_project | warn 继续 | `?` |
| rust_ui.rs run_vm_ui split | warn 后仍 start | `map_err?`（提前终结） |
| tauri.rs run_tauri_project ×2 + tauri_backend | warn | `?` |
| rust_ui.rs start_api_server / build_rust_ui | 硬（既有） | 保持 |

## 5. 兼容回归

- api_gen 全套 41/42 绿（1 ignore = tauri e2e 二进制计数；预存红
  merged_api_client_crud_fallback 为 master 在案非回归）。
- `test_a2r_body_disabled_falls_back` 更新为断言 SCAFFOLD 标注（语义重定义
  后的合同锁定，非削弱）。
