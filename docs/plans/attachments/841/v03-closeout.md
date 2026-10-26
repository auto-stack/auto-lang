# PLAN-841 T-06 / V-03 阶段证据

## 控制与结果

| 不变量 | 永久入口与断言 | 当前结果 |
|---|---|---|
| Clone、codec 与最后 owner | `auto-ui-dialect::codec::tests::codec_clone_and_decode_have_bounded_last_owner_lifetimes`：原 `Rc` 的 `Weak` 在 Clone 存活时可升级、Clone 释放后失效；decode 的独立 `Weak` 在最后 owner 释放后失效，证明无全局 side-table 保留原图 | 5 个 codec 测试全绿 |
| 字段、span、源行、版本/tag 拒绝 | 既有 `codec_roundtrip_*` 与 `codec_rejects_bad_domain_version_and_tag` 继续执行；真实 parse 的 View byte span、Core/UI 项与 `source_lines` 往返一致 | 同上 |
| 跨文件模块 owner | `auto-ui-compiler::ui_ast::tests::representative_cross_file_import_uses_owned_domain_modules`：宿主只持有 `App`，模块只持有 `Imported`，Core `answer` 经真实 import 收集 | ui_ast 4 项全绿 |
| 缓存重启与旧 schema | `auto-ui-compiler/tests/ui_cache_schema.rs`：真实 `.auto/ui-cache.json` 落盘后 `UICache::load` 保留同身份条目；把持久身份改为旧 `ui-ast-schema-N` 后条目清空且源文件判 dirty | 1 项绿 |
| 所有真实缓存入口覆盖数据版本 | `auto_ui_compiler::ui_cache_fingerprint()` 组合生成器源与 `UI_AST_SCHEMA_VERSION`；compiler `generate_component_from_file`、tools Vue/Jet 在 load 前统一注入 | 源码复核 + 上述持久化反例 |

执行命令：

- `cargo test -p auto-ui-dialect codec::tests -- --test-threads=1`：5 passed。
- `cargo test -p auto-ui-compiler --test ui_cache_schema -- --test-threads=1`：1 passed。
- `cargo test -p auto-ui-compiler --lib ui_ast::tests -- --test-threads=1`：4 passed。
- `cargo check -p auto-ui-tools --message-format=short`：未通过；23 个既有 T-07 旧 Core UI AST 消费点（`Stmt::WidgetDecl` / `ast::ui` 等）尚待迁移。本阶段 cache 调用改动无独立错误签名，不能把此命令记为通过。

## 过渡上层 adapter 登记

| adapter | 当前语义 | 终态处理 |
|---|---|---|
| `auto_ui_compiler::assemble_ui_grammar()` | 空操作，仅保留旧上层调用点；真实文法由 `parse_domain(&UiDialect)` 显式传入 | 交 842 在消费者清理后删除 |
| `auto_lang::{find_store_decl_files, resolve_use_module, UseModuleResolution}` | 旧公开路径薄 re-export 到 `auto_ui_compiler::ui_ast`，无 Core UI 数据或实现 | 保留上层兼容；交 842 评估退役 |
| `auto_lang::ui_build*` | 旧 SDK 公开门面内部已改走 `UiCode` 领域端口，返回值保持既有 AURA/生成结果 | 上层支持面，不是 Core 兼容臂 |
| `auto_lang_core::database::set_generator_fingerprint` | 公开名称保持不变，注入值现由生成器源 + UI AST schema 组成 | 交 842 评估是否更名；缓存语义已由本计划切换 |
| 旧 UI AST 字节/缓存 | 不提供隐式迁移 adapter；schema/domain/tag 不匹配具名拒绝，旧缓存身份整缓存失效并重新生成 | 本计划已收口，无后续隐式适配 |

## 警告归因

以基线 diff 的新增/修改行与构建警告交叉：迁移触面共 4 条（Core parser 1、UI compiler 2、runtime 1），均已修复。runtime 报告的其余约 216 条为预存警告，未冒充迁移问题批量改写。compiler design-token 9 项回归通过；runtime 全测试目标构建通过。`plan632` 过滤实际选中 0 项，不作为行为证据。
