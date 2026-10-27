# PLAN-842 T-01 退役矩阵与迁移方案（草案，未冻结）

当前基线 lang `ecc78701124fe6e0ab6f0e262148a1d853ba2af5`、UI `49aa54d6e18e8d893be69b9430022aea14624ce7`、Man `7cc37fa364a2f59bedc2c920638f8007cd0075ce`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；r2。历史初查按其原版本保留在 contract-baseline。三仓受管候选文件列表见 [source-inventory.json](source-inventory.json)，当前提交逐行候选见 [tracked-inventory-r2-final.json](tracked-inventory-r2-final.json)，原始索引见 [tracked-inventory-r2.json](tracked-inventory-r2.json)；搜索是定位手段，含注释/历史材料、可能漏宏展开/动态路径，不可当作全消费者完成证据。以下分类为 T-01 工作中的候选结论，带「待」的行均未关闭。

| ID | 旧入口/生产路径 → owner/替代 | 已知消费者与现行合同 | 初步迁移和区分性 oracle | 状态 |
|---|---|---|---|---|
| R01 | UI `UiParseContext` → `auto_parser_api::DomainParseContext`，域文法与 UI AST 仍归 UI | UI dialect 源及 lang `ui-grammar-consumer`；841 handoff 明确别名仅过渡 | 先迁 UI 内部文法及 fixture，移除两处公开旧名；保留真实 widget/store/view/style；旧 import 编译失败、canonical 6 测执行通过。受控实验见 contract-baseline | 候选迁后退出；版本/全消费者、最终永久负控待定 |
| R02 | `AuraViewBuilder::with_registry_and_imports` 的 `_import_stmts` → 原注册表 | UI dynamic 4 个生产入口 + aura_view_builder 10 个测试调用；841 handoff/auto-ui-runtime Spec 明确末参仅源码形过渡 | 先迁生产与测试调用后删除末参；保实际 UI-owned 外部组件注册与渲染，普通/缺失导入、旧签名编译失败及新签名真渲染；必须核实外部 consumer | 候选迁后退出；真实 probe/版本待定 |
| R03 | `auto_lang::{ast,parser,session,error}` → 语言 owner / `auto_lang_core`、`auto_ast`、`auto_diagnostics`、`auto_parser_api` | `auto-lsp`、provider fixtures、ui-dialect/diagnostics interop、examples；841 列过渡，现行 auto-lang Spec 冻结旧路径 | 按类型、功能和 feature 清点，逐消费者迁直接单源；不改变类型身份及错误语义；旧 import 拒绝与新外部 consumer 运行，并对稳定承诺单独判定 | 待逐项冻结支持及版本；不可一括删除 |
| R04 | Core `ExternalDialect`/`Dialect`/`ParserBridge`/Override/Append | parser-api Spec 明确保非 UI 三态、scope 和优先级，语言 fixture 真实消费 | 保正式非 UI 功能与类型身份；外部 UI 用 `DomainDialect<Output=UiStmt>` 显式组合，迁测试名称和说明但保让行/错误/快照控制 | 现行正式支持；不得因 UI 移除而删注册路径 |
| R05 | UI fixture `ExternalUiDialect` → 显式 `DomainDialect` 组合 | `dialect-api-consumer` 和 `ui-dialect-consumer`；841 说明其仅为 fixture 组合，parser-api Spec 列明外部 model 优先 | 保显式外部 model 的实际优先、让行、错误和 owned 输出测试；移除不再对应 UI 实际路径的旧 priority 映射，不删非 UI 覆盖 | 待核对测试 ID、具体可退出 wrapper 与永久替代 |
| R06 | `use-bigvm` 空 feature → 直接 AutoVM | auto-lang Cargo 注明 deprecated 且无效果；受管初查仅 manifest 引用；0.1.0 显式选择仍有效 | 迁调用方移除 feature，注明源码版本和迁移，真实 Cargo 旧 feature 删除应解析失败、新版不带 feature 的实际执行保持；受控实验表明旧显式依赖会被破坏 | 候选退出；真实外部用户支持期/退出版本与替代执行控制待冻结 |
| R07 | `ui`/`ui-iced` 等 feature / `auto_lang::ui` 组合转发 | 现行 runtime/auto-lang Spec 明确 feature、旧路径和默认 ui-iced 保持；类型单源 UI owner | 对每个 feature 和 API 按可承诺版本分类，先迁三仓与外部 fixture，再证明默认、no-default、显式组合、provider 退出；任何长期支持条目保 formal API，不以空 allowlist 充数 | 待逐项支持承诺与真实消费者；不得静默改默认 |
| R08 | SDK `UiCompatInstaller` inventory → 显式宿主组合入口 | `auto-lang` inventory submit → SDK 显式 hook 优先、缺席迭代登记 → UI runtime provider；多处 SDK 直达执行/VM 入口 | 构建实际 default/显式/缺席、多实例、线程与生命周期消费对照；先使每条真实路径显式接入后才能删 fallback；不能把纯 SDK 自动引入 UI | 待确认非漏斗调用者及全链机制；不得先删 |
| R09 | `delivery_crate_source` 的语言根 UI 残留优先 | 当前 831 Spec 语言根优先；832 安装 `sdk/crates` 为 lang/UI 混合布局，并用 install-manifest `source_closure.packages` 指示各包 owner + 文件 hash | 区分已验证安装投影与开发旧残留：已登记且经身份和字节校验的混合 SDK 保持可用；未登记旧 UI source 不可抢占/冒充 canonical，显式 UI 源错误不静默回落；真实 `auto gen --delivery-root` 观察来源/文件/hash，失败保旧字节；具体机制需在真实安装 probe 验证 | 真实生成已证 old-shadow 进入 bundle；T-00 真实安装生成/锁定构建已通过，旧源拒绝和版本身份机制待定 |
| R10 | `delivery_producer_root` 环境→安装 sdk→CWD→编译期搜索 | 816/832 Spec 承诺安装邻接与开发解析；明确覆盖错误根行为需核查 | 明确显式 root 权威及无效时具名失败；安装 SDK 邻接可用，不能无差别删除合法来源；实际 CLI 正反控 + 未触碰输出 | 真实安装 CLI 指定不存在的 AUTO_DELIVERY_SOURCE_ROOT 仍成功并指向邻接 SDK；覆盖策略待冻结 |
| R11 | `UI_AST_SCHEMA_VERSION=1`、`auto-ui` domain、`UICache` 生成器+schema 身份 | UI-owned codec、generator、tool 的缓存 load；841 Spec 对旧 schema/坏 tag 具名拒绝且旧缓存重建 | 不放宽校验；schema0/未知 domain/tag、尾随垃圾拒绝、schema1往返和真实 cache 两次读重建，旧已发布内容保留/恢复 | 静态与 codec 定向正反控已测；真实缓存/producer升级待定 |
| R12 | `man-install/1`、`generated-ui-delivery/1`、安装备份、source lock 与历史 refs | Man install staging→verify→backup→swap；生成交付及 792，831/832/841 收据 | 保旧安装回滚与陌生目录保护；明确兼容 schema 和后继版本，旧产物/未知 producer 均不误读；实际升级中断、完整旧字节/恢复/重定位及 fixed source | 合成 verify 反例证明未知 `man-install/999` 被接受；真实安装、升级/回滚及版本策略待验证 |
| R13 | `use-evaluator` / 语言源码及 VM 能力、I-TOOL 已支持协议 | feature 及 VM 现行 contract 保持；旧词不证明无消费者 | 独立判断正式支持；不因为名字旧或目录叫 legacy 删除协议、合法源码或后端 | 持续支持，相关映射待枚举 |


| R14 | `auto_ui_compiler::assemble_ui_grammar` 空桩 / `UiGrammarSource` → `Parser::parse_domain(&UiDialect)` | 841 最终 [handoff](../841/handoff.md) 与 compiler Spec 明确已删除；中期 v03-closeout 曾列待 842 删除，最终状态优先 | 当前三仓源码搜索未见该符号；保 841 领域组合实际行为及旧入口消失的对照，不能重复宣称本计划已删 | 已由 841 退出；需纳入最终逐行核销 |
| R15 | `auto_lang::{find_store_decl_files,resolve_use_module,UseModuleResolution}` → `auto_ui_compiler::ui_ast` 单源 | 841 中期交接要求评估；当前语言层只再导出，UI runtime/compiler 直接消费 UI owner；当前兼容包文档承诺旧路径 | 逐项核对受管及外部消费者、支持期和旧路径版本策略，迁移至 UI compiler；保真实模块/StoreFiles 行为 | 候选迁后退出；版本及消费者待确认 |
| R16 | `auto_lang::ui_build*`、`set_generator_fingerprint` | 841 表明前者为实际受支持的生成入口而非空桥；SDK Spec 明确后者进程生命周期语义，UI 工具/compiler 仍生产调用 | 保持真实生成准入、缓存身份和生命周期；不可仅凭交接“评估更名”删除或改名 | 正式功能保留；生产链及用户行为需验 |

## 实施依赖与版本

T-01 完成前仅进行可恢复、具名定向 probe，不将 T-02..09 视为完成。T-02 迁 UI 方言与真实 consumer；T-03 更新生成器/安装/source 和三仓 pin；T-04 才删已迁旧符号；T-05 版本/缓存与恢复；T-06 显式宿主接线；T-07/08 实际生产全链、777/792 与矩阵清账；T-09 文档和交接。表中顺序不放宽计划原有任务依赖。

当前各相关 Cargo 包均声明 `0.1.0`，发行按固定 Git 提交；现阶段没有经事实核验的公开 API 升版/兼容期限规则，不能捏造已发布或已获准的退役版本。T-01 仍需查实际版本与外部消费承诺、确定各行可退出版本及说明；若必须改变正式长期支持则按计划 needs_replan，不按日历推测已迁。

本草案不代表任务完成：入口全枚举、真实全链源身份正反控、宿主默认/缺席/多实例 probe、版本迁移/回滚与每行支持期均待完成，不能勾选 T-01 或开始依赖它的实施步骤。

## 来源迁移候选机制（未冻结）

1. 语言包与 UI 包分开确定 source owner。正常开发形态的 UI 交付源码取显式 UI 根或同组 UI owner；存在但无 owner 身份的 `lang/crates/auto-ui-*` 不能抢占。显式配置 root 指向无效/缺包时具名失败，不能转而拾取另一路旧残留。不能对语言 crate 误用 UI 根。
2. 安装形态的 `sdk/crates` 可以包含属于不同源仓的包。仅路径本身不构成识别依据：需要将安装根、`man-install/1`、profile、`sources`、`source_closure.packages` 的 owner/path 与 `files` 的完整性关联，确认实际被使用的源码和 manifest 所声明的一致。如何验证完整闭包、处理有明确缺陷的旧清单以及安装位置与用户显式覆盖的相互作用，仍需真实安装实证；不能把可由用户修改的 hash 当作密码学签名。
3. `delivery-manifest` 必须记录实际每个源/版本与输出内容的关联，并使错误或未知 schema/producer 在产生输出或覆盖前拒绝；保持 792 staging、保旧和恢复规则。对受支持旧安装/旧 bundle 给迁移或保留读取路径，对不可兼容数据具名拒绝并提供重建或恢复。候选没有证明任何旧清单都可无歧义迁移。
4. 探针固定四个区分态：规范 UI 单源成功；规范 UI+语言旧影子时结果仍来自规范 UI；只剩未登记旧影子具名失败且旧输出哈希不变；真实安装 SDK 单源正常生成/重定位。再验显式坏根、源版本漂移、未知/缺失/损坏清单、hash 篡改、取消/中断与恢复，检查真实生成产物和独立外部构建而非仅 selector 返回。

该方案是初始工作假设，不取代 T-01 要求的实际安装探针和完整逐入口版本决策；未以此授权删代码。

## r2 T-00 对本矩阵的影响

真实 Full 安装邻接 SDK 的正常投影、独立 fresh/locked 生成构建及坏源保旧/恢复由 [T-00 最终验证](t00-scoped-verification/report.md)完成；这不证明 R09 的旧影子源拒绝，也不证明旧安装凭其目录可可信判定、未知清单 schema 已处理。Man 发行验证引入本验证并 pin 新 UI 提交；接下来依 T-01 逐行确权、区分影响和测试，而非将 T-00 的单路径通过推导成其他项通过。


## 841 交接补查

中期 [v03-closeout](../841/v03-closeout.md) 列出的空 `assemble_ui_grammar` 与最终 handoff/现行 Spec 存在时间差：最终交接明确其已在 841 删除；本次实际三仓源码检索无命中，按最终状态而非中期清单执行。UI module resolution 三个再导出及受支持 `ui_build*`、SDK 指纹 setter 已补入 R15/R16；当前搜索含历史材料，仍需按可执行目标逐项核销。T-00 最终来源见上，新版候选索引绑定最终三仓 commit，但检索不构成未知外部消费者的完整证明。


## 指定错误源码根的真实生产反例

最终 Full 安装 `probe-install-r2-scoped` 的真实 `auto.exe gen --delivery-root`，使用已验证项目，清除其它 `AUTO_*` 后显式设置 `AUTO_DELIVERY_SOURCE_ROOT` 为经确认不存在的 `D:/autostack/.wt/lang-842/plan842-definitely-missing-source-root`；命令 exit 0，实际生成的 delivery manifest 记录 producer 为该安装的邻接 `sdk`。另用仅设置不存在的 `AUTO_UI_SOURCE_ROOT` 的同入口生成到另一新目录，exit 0，交付 UI 源字节与邻接安装一致，证明该显式 UI 覆盖亦被忽略。因此当前实现会静默忽略明确指定的错误源根；这证明 R10 的真实行为，并非已满足本计划错误根拒绝要求。测试使用新的专用输出目录，不覆盖 T-00 验证包。之后 T-03 需构造同一生产入口的负控：具名错误、原输出不变，另保无显式覆盖时的已验证邻接安装正控。


## feature 逐族清点（候选处置，仍须按来源和实际 consumer 冻结）

依据当前 `auto-lang/crates/auto-lang/Cargo.toml` 与 UI runtime/compiler manifest；下列是现有接口，不因为实现为空就视为无使用。

| 原功能族 | 当前连接及合同 | 初步处理 |
|---|---|---|
| `default` = `with-file-history`, `ui-iced` | 明确的默认 UI 合同；`ui-iced` 再组合 interpreter、image、editor、clipboard/dialog/dock 等 | 保留默认效果；迁移后用真实默认、显式及无 UI consumer 对照 |
| `ui`, `ui-interpreter`, `ui-headless` | runtime 同名 feature；compiler `ui` / `ui-headless` 目前为空但受上层显式依赖 | 在 UI owner 保持对应编译选择和效果；上层转发是否退出须先对受支持旧路径和具体退役版本决策 |
| `image-pipeline`, `ui-iced`, `code-editor`, `highlight-treesitter`, `ui-clipboard`, `ui-dialog` | UI runtime 有同名生产 feature；`code-editor` 同时接语言 SDK | 保留实际能力与独立选择；迁移时不能只直连 runtime 而漏 SDK 边 |
| `iced-layout-tests`, `mpv-native`, `mpv-gpu`, `mpv-widget`, `mpv-spike`, `autodown`, `native-dock`, `native-clipboard`, `native-dnd`, `test-native-dock` | 同名 runtime 及关联 feature、跨仓和测试选择 | 保持能力与测试被选择；先核对实际 CI/fixture 消费及迁移路径 |
| `test-trans`, `test-cases`, `test-book`, `test-aavm`, `test-http-e2e`, `build-examples` | 测试档或显式依赖门；`test-cases` 同时启用 Core 和可选依赖 | 保留已支持测试/安装投影语义；不能以删除测试名通过 |
| `mem-profile`, `with-file-history`, `use-evaluator`, `streaming-http`, `python` | 正式语言/执行/配置能力 | 保留；不可按旧功能名称退役 |
| `use-bigvm` | 已标 deprecated 的空兼容开关，真实 Cargo 负控证明删后旧调用无法解析 | 候选迁后退出；必须绑定明确版本/迁移说明及替代执行正控 |

该表尚未证明每个 feature 的全部消费方、支持期和可退出版本，不代替完整矩阵。


## 未知安装版本的升级前控制

在真实、已校验的 637 文件 Core 安装上，仅临时将 manifest 的 schema 从 `man-install/1` 改成 `man-install/999`，其余文件保持；以正式 `man-install.py install --profile core --skip-build --fail-point prepare` 尝试升级。结果退出 2，但原因是注入的 `prepare` 故障而非未知版本：说明未知版本没有在开始准备前被识别或拒绝。受控故障下未知 schema 清单与旧 auto.exe 哈希不变，finally 恢复原清单后完整 verify 637 文件成功。此控制没有执行实际替换，不能据此声称未知版本升级后必然保旧；T-05 须针对真实 `verify` 与 `install` 入口同时加入版本门、重放合法升级/rollback 和旧字节保护。


## 受管生成物中的旧门面（真实产物）

T-00 实际安装 CLI 所生成、fresh lock 后 locked build 成功的 `t00-scoped-delivery-check/installed-rust-delivery/app/src/main.rs`，同时包含 canonical `auto_ui_runtime::ui::{Component, View}` 和 `auto_lang::ui::mem_guard`、`AppResult`、`desktop_protocol` 等旧门面（如源文件行 8、103、123）。其来源是 UI `auto-ui-tools/src/rust_ui.rs` 的实际 Rust 模板；不是只有过时注释或测试字符串。Man `cmd_vue.rs` / `cmd_tauri.rs` 也实际经 `auto_lang::ui_gen`、`auto_lang::ui::style` 与 `ui_build*` 调用；UI tooling 的 `shadcn_build` 已提供部分直接入口，但替代必须核对接口与结果。T-02/03 应先迁真实模板与 Man 生产调用，重新以安装 CLI 生成产物搜索和独立 locked build 控制，随后才能考虑 T-04 删除目标门面；不能删旧 import 检测或把一次 build 成功当 canonical 消费已达成。


## AST 跨路径兼容实测

当前版本 `cargo test --locked --offline -p auto-lang --test ast_cross_path_identity -- --test-threads=1`：2 passed、0 ignored，真实 Cargo consumer 的旧 `auto_lang::ast` 与 `auto_ast::ast` 的 `Code` 可双向传值，两个路径 `ToAtom` 输出相等；六类 parser 便利路径还对坏输入或合法名字做控制。原有 warnings 存在。该对照证明现行类型身份和错误语义，未来迁移必须用 canonical 路径保持同等行为；若旧路径明确退役，此测试不能以简单删除或只比较文本替代，需要另立旧入口失败控制与迁后 consumer 正例。


## 入口迁移图（逐项工作草案）

| 当前入口 | 实际 owner / 候选消费路径 | 迁移必须维持的行为 |
|---|---|---|
| `auto_lang::ast::*` | `auto_ast::ast::*`，`auto_lang_core::ast` 当前为同一模型的薄再导出 | `Code` 双向传值与 `ToAtom` 输出；解析便利 trait 由 SDK parser 提供，不能误认为它属于纯 AST crate |
| `auto_lang::parser::{Parser,ParserExt,...}` | `auto_lang_core::parser` | 实际 Parser 与 `ParserExt` 的合法/错误结果及外部方言注册；先核每个导出，避免漏掉根级 `auto_lang::Parser` |
| `auto_lang::session::{CompilerSession,Scenario}` | `auto_parser_api::{CompilerSession,Scenario}`（SDK 同一类型） | UI/Core scenario 行为及跨路径类型一致 |
| `auto_lang::error::*` | `auto_diagnostics::error::*`（SDK 同一诊断族） | 具体 `AutoResult`/错误/span/输出身份；不得将 AST 的不同结果类型粗暴 glob 合并 |
| `auto_ui_dialect::UiParseContext` | `auto_parser_api::DomainParseContext` | UI grammar widget/store/view/style 与外部端口；移除别名后真实旧 import 有编译失败控制 |
| `AuraViewBuilder::with_registry_and_imports(..., import_stmts)` | 既有 `auto_ui_runtime` builder + 已装载 UI registry | 删除不读取的末参之前迁 `dynamic.rs` 4 生产调用及 builder 测试；实际导入组件注册/图标渲染与缺失导入对照 |
| `auto_lang::{find_store_decl_files,resolve_use_module,UseModuleResolution}` | `auto_ui_compiler::ui_ast::{...}` | StoreFiles 与普通模块解析行为，路径只换 owner，不复制算法 |
| `auto_lang::ui::*` in generated Rust | 直接 `auto_ui_runtime::ui::*`，语言功能从 `auto_lang_core` 消费 | 模板的 `mem_guard`、`AppResult`、desktop/rqhost、renderer 及被生成组件引用全部一并迁；真实安装生成包、fresh lock、locked build 和 desktop/runtime 控制 |
| `auto_lang::ui_gen::*` in Man | `auto_ui_compiler::ui_gen::*`；部分命令需真实 `auto-ui-tools` 的 shadcn 入口 | Man Vue/Jet/Tauri 真实命令与产物；不得把 `ui_build*` 误当无行为空壳 |
| `use-bigvm` | 删消费方 Cargo feature；直接 AutoVM 是现行默认 | 旧依赖解析拒绝、新依赖实际执行成功，另保 `use-evaluator` 支持 |

路径映射来自当前源码与现行 Spec；本表没有冻结外部支持期、退役版本，也未代替逐 consumer 核验。旧 Git 固定源可用于恢复，但必须验证被保留的旧生产载荷及源图，不能把 Git ref 的存在直接等同用户安装可恢复。


## 版本身份实证与待定迁移边界

目前 `auto-lang`、`auto-ui-runtime`、`auto-ui-compiler` 和 `auto-ui-dialect` 的 Cargo package version 都是 `0.1.0`；不同于仓库已有的 `v0.2.0`、`v0.3`、`v0.4`、`v0.5.0` Git 标签，不能把这些历史仓库标签等同将来受影响 Cargo 包的发布版本。当前 Full 安装清单实际绑定四仓 Git SHA，Core 只绑定 lang/Man；842 的修复尚未合入或发布。可重放前版本基线分别为 lang `ecc78701124fe6e0ab6f0e262148a1d853ba2af5`、UI `4903bef64532750d1597627b1197f3d2cac1effc`、Man `9a75d3bab6eaeda961d5dcbf7932258f89e650af`，down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；T-00 修复后 UI/Man 新提交见本矩阵开头。保旧恢复必须实际重建并验证匹配的安装和锁，单列 SHA 不构成恢复通过。

可行的候选是将有源码破坏的 Cargo 包下一个兼容域明确升级，并将 842 最终实际三仓 SHA 与 Cargo lock/安装来源作为交付身份；调用方保持旧固定版本或迁到新 API，用户产物需要明确重建/恢复指导。具体 Cargo 包及版本、已有发布支持期、受影响外部依赖、同步 pin 和旧安装恢复尚未核定，故仅为方案候选，不得当成已经批准或已经发布的新版本。


旧固定版本的实际 Full 安装 `probe-install-full` 在最终 Man 脚本下完整 verify 6800 文件成功，来源四 SHA 与上一段基线一致；其旧 `auto.exe` 实际运行既有 `probe.at` 输出 `plan842-core-ok`、exit 0。该保留实物证明旧安装的内容和语言执行仍可用，不证明其原有 Rust delivery 缺口已修复，也不是升级后从备份恢复的证据；T-05 仍需执行真实升级/回退。
