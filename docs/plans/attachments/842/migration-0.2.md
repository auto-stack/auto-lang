# AutoLang 0.2 兼容入口迁移说明（PLAN-842 审查稿）

本说明对应计划工作树的冻结提交，**尚未合入或发布**。`auto-lang`、`auto-ui-dialect`、`auto-ui-runtime` 从 0.1.0 升至 0.2.0；其他包不因本次内部消费迁移而自动升版。路径依赖或 Git 依赖必须同时更新真实源码和锁文件，不要只修改版本号。升级前保存项目、锁和受管安装备份。

| 0.1 入口 | 0.2 迁移 |
|---|---|
| `auto_lang::ast::*` | `auto_ast::ast::*`；SDK 也可用 `auto_lang_core::ast::*`，同一类型。 |
| `auto_lang::parser::*` | `auto_lang_core::parser::*`；正式 `auto_lang::Parser` 根入口仍保留。 |
| `auto_lang::session::*` | `auto_parser_api::{CompilerSession, Scenario}`，或 SDK 同源 session。 |
| `auto_lang::error::*` | `auto_diagnostics::error::*`，或 SDK 同源 error。 |
| `auto_ui_dialect::UiParseContext` 和 `auto_ui_dialect::grammar::UiParseContext` | `auto_parser_api::DomainParseContext`。UI 文法以 `DomainDialect` 显式组合，不再依靠旧文法登记。 |
| `AuraViewBuilder::with_registry_and_imports(bridge, name, registry, imports)` | `AuraViewBuilder::with_registry(bridge, name, registry)`；组件通过 UI registry 供给。`DynamicComponent::with_registry_and_imports` 有实际导入语义，仍保留。 |
| `auto-lang/use-bigvm` | 从依赖特性和构建命令中移除；AutoVM 默认可用。`use-evaluator` 和运行时引擎选择保留。 |

现有 `auto_lang::ui`、`back_proxy`、`host_adapters`、`find_store_decl_files`、`resolve_use_module`、`UseModuleResolution`、UI build 与生成器指纹等明确支持的薄入口仍可用；新生成代码和已迁第一方消费直接使用 UI runtime/compiler 与语言 SDK。非 UI 的 `Dialect`、`ExternalDialect`、Override/Append 与 `ParserBridge` 保留。不要把 T-04 清单之外的这些稳定接口视为已删除。

需要显式组合 UI 宿主的应用，可在启动入口调用 `auto_lang_core::set_ui_host_install_hook(auto_ui_runtime::install_ui_host_capabilities)`，再调用 `auto_ui_runtime::install_ui_host_capabilities()`；应用显式 provider 优先，纯 SDK 无 UI 时保持缺席。为兼容已链接 UI 语言包却直接调用 SDK 的现有消费者，当前单份 inventory 登记仍保留：删除它的真实反例会丢失默认能力。此登记与 UI 文法已退出的 inventory 是不同机制。

## 来源与安装

按同组源码冻结，不混用任意不同提交。本工作期候选为：

- auto-lang `a8b4a5bcf53d9be484cc38f028453a4cebbc947f`
- auto-ui `bd0e0f38253c5fd0676c8ef729c8c030d372ae82`
- auto-man `9ac33ad871c4e0278b322e9712930ed059029e9c`
- auto-down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`

这些是审查候选，实际合入后的来源身份以交付收据为准。继续使用旧 API 时可保留已验证的 0.1.0 组合：lang `831d5ec14ae76736d02baf220749941bdd31ff58`、UI `407cf367d4db26c528f86f2b63063e092e9c9cc7`、Man `6bde652fce522e62f4205ea5294e5ef6c51a96f8`、down 同上，连同原锁与安装备份。旧 Full 安装及其恢复副本已实际验证；这不表示已向任何远端发布上述版本。

Full 交付中，UI 源只能来自指定 UI 根、同组 UI 仓或受管 Full 安装 SDK；SDK 的 `man-install/1` owner、路径与所消费文件哈希必须匹配。`AUTO_DELIVERY_SOURCE_ROOT` 优先于 `AUTO_LANG_ROOT`，二者及 `AUTO_UI_SOURCE_ROOT` 一旦显式指定即为权威：无效时报错，不回退旧影子。清单和哈希校验用于一致性检查，**不是数字签名或来源认证**。

`man-install/1` 与 `generated-ui-delivery/1` 仍支持；未知 schema/profile、损坏安装或 producer 在覆盖前明确拒绝。UI AST schema 1 正常往返，旧/未知 schema、错误 domain/tag 拒绝；旧缓存身份失效后重建。旧交付自带的历史 verifier 是原版本程序；升级到新交付后才使用新 verifier 的未知版本保护。

安装升级保留独占备份；失败时先核对诊断和旧文件，不要删除旧包。按 auto-man 仓的 `docs/installation.md` 操作：把当前候选移到安全位置，恢复相应备份，把事务 `.man-backup.marker` 保存在恢复根之外，然后运行 `python scripts/man-install.py verify --profile full --install-root <恢复根>`（Core 使用 `--profile core`）；通过后再使用。保留备份及 0.1 固定源，不以新版清单重写历史收据。

T-05 真实演练了旧 Core/Full 安装升级、备份恢复和 0.1→0.2 交付构建；T-07 对工作期最终来源重装与复测。已验证范围与未测 MPV 硬件播放等限制见 [验证报告](t07-progress/report.md) 和 [审计](t08-audit/report.md)。
