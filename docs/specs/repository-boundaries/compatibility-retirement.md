# 兼容入口退役与迁移（PLAN-842）

> Status: active；受影响源码 API 以 0.2.0 为不兼容边界。

| ID | 当前处置 |
|---|---|
| R01 | `UiParseContext` 两处 UI 公开别名：`auto-ui-dialect 0.2.0` 移除；`auto_parser_api::DomainParseContext` |
| R02 | Aura builder 末参：`auto-ui-runtime 0.2.0` 移除，迁移至现有 `AuraViewBuilder::with_registry` 三参入口，registry 供给组件 |
| R03 | `auto_lang::{ast,parser,session,error}` 四个过渡模块：`auto-lang 0.2.0` 移除公开旧路径；保其他现行正式根入口与能力 |
| R04 | 保留 `ExternalDialect`、`Dialect`、`ParserBridge`、Override/Append 非 UI 契约 |
| R05 | 删除 `ExternalUiDialect` 旧优先级测试映射而非删除外部 model 能力；示例组合按领域组合重写 |
| R06 | `use-bigvm` 空过渡 feature：`auto-lang 0.2.0` 移除；AutoVM 无 feature 默认启用 |
| R07 | 正式 UI feature 及 `auto_lang::ui`、`back_proxy`、`host_adapters` 兼容面保留，不因本矩阵全删；已知生成/Man 消费迁 canonical owner |
| R08 | 正式宿主 provider、hook、default/显式/缺席/多实例/线程/退出语义保留；仅已证明可替代的 inventory 兜底才可退出 |
| R09 | 未登记的语言仓 UI 影子源退出；开发 UI owner 优先；合法受管安装混合 SDK 保持 |
| R10 | 显式 `AUTO_DELIVERY_SOURCE_ROOT` 优先于 `AUTO_LANG_ROOT`，选中的显式语言根及显式 `AUTO_UI_SOURCE_ROOT` 权威，指定错误立即失败；未指定时保合法安装邻接和开发发现 |
| R11 | 保 UI AST schema/domain 1 和 generator+schema cache identity；旧/未知 schema/tag 拒绝或已有缓存失效重建 |
| R12 | `man-install/1` 与 `generated-ui-delivery/1` 继续支持；未知版本/profile/坏 producer 在覆盖前具名拒绝 |
| R13 | `use-evaluator`、合法语言/VM/非 UI 协议与 I-TOOL 支持保留 |
| R14 | `assemble_ui_grammar`、UI `UiGrammarSource` 在 841 已退出，842 不重复认领 |
| R15 | `auto_lang::{find_store_decl_files,resolve_use_module,UseModuleResolution}` 为已有正式薄 API，暂保；受管新消费直接 `auto_ui_compiler::ui_ast` |
| R16 | `auto_lang::ui_build*` 真实生成入口与 `set_generator_fingerprint` 正式 SDK 接口保留 |

`auto-lang`、`auto-ui-dialect`、`auto-ui-runtime` 为 0.2.0；旧 API 使用者可固定原 0.1.0 四仓源码、对应锁与安装备份。已支持的 `man-install/1`、`generated-ui-delivery/1` 和 UI AST schema 1 不因包版本升级失效；未知或损坏状态具名拒绝并保留旧产物。受管仓内生产、示例、测试、脚本按冻结范围迁移，不宣称已调查未知仓外消费者。

宿主 `UiCompatInstaller` 保留用于已链接 UI 而直接调用 SDK 的默认能力；这与 UI 文法 inventory 已退役是不同机制。显式 provider 优先，纯 SDK 无 UI 时缺席/no-op。

旧固定版本、具体替代与恢复操作见 [迁移说明](../../plans/attachments/842/migration-0.2.md)；计划和验收来源：[PLAN-842](../../plans/archive/842-legacy-compat-and-distribution-retirement.md)。
