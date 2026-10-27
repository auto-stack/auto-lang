# PLAN-842 T-01 退役矩阵与迁移合同（r2，2026-10-27）

本矩阵冻结工作决策，不表示以下迁移、测试、发布或合入已经完成。依据 [合同基线](contract-baseline.md)、[受管源码候选](tracked-inventory-r2-final.json)、[上游 841 交接](../841/handoff.md) 和 [探针草案及实测](retirement-matrix-draft.md)；检索范围为冻结版本三仓的 tracked 生产、示例、测试、脚本与 CI，仓外未知消费者通过版本界定和迁移说明处理，不假称已全部调查。基线 lang `ecc78701124fe6e0ab6f0e262148a1d853ba2af5`、UI `49aa54d6e18e8d893be69b9430022aea14624ce7`、Man `7cc37fa364a2f59bedc2c920638f8007cd0075ce`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。T-00 真实安装证据 [报告](t00-scoped-verification/report.md)；下述未完成测试归各实施步骤执行，不用基线控制替代。

## 版本与迁移

现存受影响 Cargo 包版本为 `0.1.0`，现行安装按四仓 Git commit 固定而非 Cargo 数字确定源码。凡本计划确实改变公开 Rust 源码接口的包，以 `0.2.0` 作为本计划拟交付的不兼容 Cargo 版本：`auto-lang`（R03/R06，及实际删除的其他门面）、`auto-ui-dialect`（R01）和 `auto-ui-runtime`（R02）；只更内部调用而接口不破坏的包不盲目升版。T-05 更新相应版本、所有 lock 与引用，T-07 fresh/locked 实测，T-09 列准确最终 commit、依赖闭包与迁移命令；最终实际源 SHA 仅提交后产生并在交付前冻结，本行不是宣称版本已发布。旧 `0.1.0` 源码与已安装产物保持在旧 Git 固定源、锁、安装备份中，用户要延续旧 API 可固定旧四仓版本；升级到 `0.2.0` 时先按下表迁源码/feature，再更新三仓 pin 和锁并重新构建。旧 `man-install/1`、`generated-ui-delivery/1` 与 UI schema 1 不因 Cargo 升版自动失效；不兼容/未知 producer 与 schema 不得冒充兼容。历史 refs 与恢复载荷不得删除。该方案只退出 841 明确交接的过渡面及明确列出的 deprecated feature；其他 Spec 承诺、默认和能力保留，除非本计划已有受控变更，禁止凭目录或空实现推断可移除。

| ID | 处置与退出版本 / 替代 | 已知 owner / 受管消费者 | 迁移与实际关闭控制（owner） |
|---|---|---|---|
| R01 | `UiParseContext` 两处 UI 公开别名：`auto-ui-dialect 0.2.0` 移除；`auto_parser_api::DomainParseContext` | UI grammar/internal、lang `ui-grammar-consumer`/`plan810_full_ui_grammar`；841 明列过渡 | T-02 先迁全部已知 consumer；T-04 删除别名，旧 import 编译失败，canonical grammar 6 测及真实 UI 文法保持。已做受控移除 red→迁名 6/6 证据。 |
| R02 | Aura builder 末参：`auto-ui-runtime 0.2.0` 移除，迁移至现有 `AuraViewBuilder::with_registry` 三参入口，registry 供给组件 | UI runtime `dynamic.rs` 4 生产点及 builder 测试；841 指定退役 | T-02 更新全部调用至既有 `with_registry`；T-04 删除末参并核销无消费者的旧方法；旧签名编译失败，三参入口真实导入/未导入渲染控制；既有 imported-icon 定向基线 1/1。 |
| R03 | `auto_lang::{ast,parser,session,error}` 四个过渡模块：`auto-lang 0.2.0` 移除公开旧路径；保其他现行正式根入口与能力 | SDK parser、`auto_ast`、`auto_diagnostics`、`auto_parser_api` 为单源；lang LSP/测试/fixture/示例按索引迁 | T-02 先迁至 `auto_ast::ast`、`auto_lang_core::parser`、`auto_parser_api::{CompilerSession,Scenario}`、`auto_diagnostics::error`；T-04 移除公开 re-export，内部若需要使用私有同源别名。旧 import 编译失败；新消费者真实执行、AST 类型/错误/语义保持；基线 cross-path 2/2。 |
| R04 | 保留 `ExternalDialect`、`Dialect`、`ParserBridge`、Override/Append 非 UI 契约 | parser-api/SDK、`dialect-api-consumer`、`ui-dialect-consumer` 等；现行 Spec 明确保留 | T-02 将 UI 外部模型仅经 `DomainDialect<Output=UiStmt>` 显式组合；非 UI 三态、作用域/优先级及旧正式端口不得删，原测试 ID 保持可选。 |
| R05 | 删除 `ExternalUiDialect` 旧优先级测试映射而非删除外部 model 能力；示例组合按领域组合重写 | `dialect-api-consumer`/`ui-dialect-consumer`；841 handoff | T-02 保留外部 model 优先、让行、错误和 owned 值的实际断言；原非 UI Override/Append 控制不删除。 |
| R06 | `use-bigvm` 空过渡 feature：`auto-lang 0.2.0` 移除；AutoVM 无 feature 默认启用 | auto-lang Cargo；旧启用者会实际解析失败，受控红/绿已证 | T-02 迁受管调用，T-06 删 feature；以真实 Cargo 旧配置具名失败与新配置 VM 执行对照；保 `use-evaluator`。 |
| R07 | 正式 UI feature 及 `auto_lang::ui`、`back_proxy`、`host_adapters` 兼容面保留，不因本矩阵全删；已知生成/Man 消费迁 canonical owner | auto-lang、UI runtime/compiler、Man；runtime Spec 承诺 feature 组合与旧路径 | T-02/03 迁真实 first-party、生成物用 runtime/compiler/SDK 直连；保 default/no-default/显式 feature 及 UI 路径，可核销为正式薄 API，非第二实现。 |
| R08 | 正式宿主 provider、hook、default/显式/缺席/多实例/线程/退出语义保留；仅已证明可替代的 inventory 兜底才可退出 | auto-lang inventory→SDK hook→UI runtime；SDK 多个直达执行入口 | T-06 先迁各直达路径并永久覆盖真实默认/显式覆盖/无 UI/两实例/线程及退出；直接移除 inventory 的实际反例已红，未满足不可宣称删除。 |
| R09 | 未登记的语言仓 UI 影子源退出；开发 UI owner 优先；合法受管安装混合 SDK 保持 | `delivery_crate_source`、安装 `source_closure.packages` 与 hash | T-03 绑定 UI 源，合法安装要查 `man-install/1`、profile、owner/path 与被消费文件 hashes；显式源无效失败。真实 canonical+shadow、only-shadow、安装迁移/重定位、失败保旧及锁构建由 T-03/07 验证。 |
| R10 | 显式 `AUTO_DELIVERY_SOURCE_ROOT` 优先于 `AUTO_LANG_ROOT`，选中的显式语言根及显式 `AUTO_UI_SOURCE_ROOT` 权威，指定错误立即失败；未指定时保合法安装邻接和开发发现 | UI tool `delivery_producer_root`/`delivery_crate_source`；真实安装 CLI | T-03 同入口重放错误根 red→具名失败、旧产物字节不变；无覆盖真实安装成功，避免把开发缺源误判成合法安装。 |
| R11 | 保 UI AST schema/domain 1 和 generator+schema cache identity；旧/未知 schema/tag 拒绝或已有缓存失效重建 | UI codec、compiler、tools cache；841 Spec | T-05/07 保有效 schema1 roundtrip、未知拒绝、真实 cache 重建及失败保旧；不以升级目录名删除仍支持的数据。 |
| R12 | `man-install/1` 与 `generated-ui-delivery/1` 继续支持；未知版本/profile/坏 producer 在覆盖前具名拒绝 | Man installer/verify、UI delivery、历史固定源/旧安装 | T-05 检验 verify **及 install**，包含真实 999、旧安装升级、prepare/swap 中断、旧三件 hash/恢复与重定位；旧安装 verify/脚本实测通过不替代回退。 |
| R13 | `use-evaluator`、合法语言/VM/非 UI 协议与 I-TOOL 支持保留 | lang SDK、Man、UI tool；现行正式 Spec | 迁调用不改结果、能力发现、取消/退出；相关定向控制 T-02/07，不能用 allowlist 删除原测试。 |
| R14 | `assemble_ui_grammar`、UI `UiGrammarSource` 在 841 已退出，842 不重复认领 | 841 最终 handoff 和 compiler Spec | T-08 核源码及测试/文档差额；中期 v03 清单不能覆盖最终交接。 |
| R15 | `auto_lang::{find_store_decl_files,resolve_use_module,UseModuleResolution}` 为已有正式薄 API，暂保；受管新消费直接 `auto_ui_compiler::ui_ast` | UI compiler 实现、runtime/Man 与已有公开兼容面 | T-02/03 迁 first-party，T-08 确保单源；稳定薄面保留且明确列入 Spec，不以它代替 R03/R01 等必须退出项。 |
| R16 | `auto_lang::ui_build*` 真实生成入口与 `set_generator_fingerprint` 正式 SDK 接口保留 | Man Vue/Tauri、UI tools/compiler | T-02/03 可改 first-party canonical 调用但保现有外部语义；真实生成准入、缓存身份与生命周期定向复测。 |

feature 分组和完整候选文件逐行索引见 [草案](retirement-matrix-draft.md)及 [索引](tracked-inventory-r2-final.json)。`ui`、`ui-iced`、`ui-interpreter`、`ui-headless`、image/editor/clipboard/dialog/dock/MPV/autodown 与相关 test feature 均按现行支持保留；所有测试档及生成器输入选择必须核销，不把空 feature、空包装与正式 API 机械等同。

## 生产路径与阶段关闭边界

- T-02 入口：UI dialect/grammar、`ui-grammar-consumer`、语言内 grammar 测试、四类 facade 的 LSP/fixture/示例；只迁 consumer，待 T-04 才去掉 R01/R03，待 R02 新旧调用迁齐才移参。现有 old-name 消融与 grammar、AST 证据证明控制能区分错误；实施后重新跑受影响实际 consumer/生产路径。维护原功能及测试，不删除验证来制造零引用。
- T-03：UI generator/tool 和 Man 生产 consumer、安装/manifest/pin。实际 Full 安装 SDK 同目录含多 owner crate，不能按路径把真实安装当 shadow；受管身份仅作一致性/篡改检查，不称密码签名。
- T-04..06：按上表执行移除与版本/装配；每行旧配置和合法替代各有实际入口与断言。R08 若不能同时保持上述默认和多实例合同，应停止该删除并按本计划处理具体设计冲突，不可暗中破坏能力。
- T-07..09：真实 Core/Full 消费、失败恢复、777/792、全部矩阵核销、实际版本与迁移说明/Spec delta。当前真实安装与 feature/alias 宿主反例只是基线，不代表上述未来任务通过。review、merge 与其全档门仍 pending。


### T-03 补充核实：Man 代码实际挂载

先前草案将 `crates/auto/src/cmd_vue.rs` 与 `cmd_tauri.rs` 当成当前 CLI 生产调用；经三仓 tracked `git grep`，两文件未由 `mod` 声明、没有可执行调用，命中仅为它们自身注释，不能据此声称已验证真实 CLI。真实安装 CLI 的 UI 路径经 `tool_ui_ops`/I-TOOL；`auto-man/src/compat.rs::create_jet_project` 是仍通过 `UiOps` 使用的独立兼容库真实调用，迁向 `auto_ui_compiler::ui_gen::jet`。两份未挂载旧文件的退役/历史引用与旧代码扫描在 T-03/T-08 核销，旧 git 历史保留。


进一步跨仓核查修正：尽管旧 `cmd_vue.rs` 没有 Man CLI 的编译挂载，语言仓 `crates/auto-lang/tests/schema_drift.rs` 原来仍从该文件扫描 shadcn 安装表；因此它并非全三仓零引用，前段“没有可执行调用”只适用于 Man CLI。T-03 必须先将此独立围栏迁到真实 UI 工具 `crates/auto-ui-tools/src/vue.rs` 的表，并用真实表条目消融控制证明扫描敏感；原 `schema_drift_fence` 测试 ID 及原 schema 判断仍保留。迁完并复核后才能退出旧文件。


再补充一项跨仓真实读取：`crates/auto-lang/tests/gallery_golden.rs` 也在画廊 import 围栏中读取旧 `cmd_vue.rs`（原路径还是旧单仓布局）。已迁为读取同一 UI owner `vue.rs` 安装表并新增定向正反控；该控制实际验证仅靠安装表可解析的 `input-otp` 及未登记项拒绝。完整画廊 golden/重型围栏未在工作阶段运行，后续适用门仍待执行。


### T-03 新发现：后端生成与开发工作区来源

`auto-api-gen/src/api_gen.rs` 真实 Rust 服务模板仍发射 `auto_lang::http_service_config`/observability/file/upload、`auto_lang::a2r_std` 与 `auto_lang::ui::photo_service`，生成 server manifest 直接依赖旧 `auto-lang`。这些并非 UI 应用模板，不能用前述 Rust UI bundle 通过替代后端验收。应将纯语言/HTTP 能力映射到 `auto_lang_core`，照片服务映射到实际 UI runtime owner（同时审计 feature 与正常/无 UI 路径），迁移实际 manifest 与其 locked 外部构建、保留 `qualify_a2r_std` 的字符串/注释、独立 sys/sqlite、幂等与名称边界原有控制。真实代码、错误及现有测试 ID 不能删除；必要的 generated service 正反控归 T-03/07。

另外 `auto-api-gen/src/rust_workspace.rs::resolve_ui_runtime_path` 对语言仓旧同名 crate 优先，再查独立 UI 兄弟路径；这是开发工作区入口的另一旧来源 fallback，必须与已修交付模式一致地区分规范 UI 根和不再受支持的影子源，同时保合法 installed SDK/显式根与真正生成构建，不可只修改 Rust UI delivery selector。两项为原 AC-02/T-03 对生成器和旧来源的实际覆盖，不扩大为新协议或新后端。

### T-06 R08 必需自动装配的实证核销

现有 `UiCompatInstaller` 链接登记用于已链接 UI 语言包但直接调用 SDK 的正式路径。最终独立 consumer 对照：UI 默认、显式 provider、无 UI/纯 SDK、重复装配、线程与退出均通过；仅移除登记时直接 SDK 默认实际失败，恢复原字节后通过。冻结矩阵已明确只有可替代的兜底才退出，故本项保留这一有实测需求的单份登记与正式 hook，不制造替代自动注册，也不宣称该入口已删除；T-09 Spec delta 须明确其语义。若今后要完全删除，须以具体迁移合同改变该直接消费默认行为。证据见 [T-06 验证](t06-progress/report.md)。
