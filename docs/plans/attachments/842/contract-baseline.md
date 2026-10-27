# PLAN-842 T-01 合同基线（进行中，未冻结）

2026-10-27；plan r1。G842 的 831/832/841 完整交付证据见主计划 §9。
语言工作树 `D:/autostack/.wt/lang-842/auto-lang`，分支 `codex/plan-842-dev`，基线 `ecc78701124fe6e0ab6f0e262148a1d853ba2af5`；UI `4903bef64532750d1597627b1197f3d2cac1effc`、Man `9a75d3bab6eaeda961d5dcbf7932258f89e650af`；down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`（同组 detached，干净）。UI/Man 同组工作树各为 `codex/plan-842-dev`、基线同上述提交，tracked 状态干净；UI/Man 主检出 tracked 状态干净。语言主检出有未跟踪 Cargo.lock 构建文件，均未改动。

## 起始核对（非完整退役矩阵）

| 来源 / 候选 | 实际 owner / 已知消费者 | 现行承诺 / 待证明 |
|---|---|---|
| 841 交接 `auto_lang::{ast,parser,session,error}` | `crates/auto-lang/src/lib.rs` 再导出 `auto_lang_core`；语言 `auto-lsp`、`ui-dialect-consumer`、diagnostics interop 等实际引用 | 现行 auto-lang Spec 将上层定义为兼容/默认组合；需逐条核对支持和迁移版本，不能仅凭同类型转发就删。 |
| `AuraViewBuilder::with_registry_and_imports` 末参 | UI runtime `aura_view_builder` 与 `dynamic.rs` 调用链；Spec 明示末参源码形过渡且不读取 | 清点所有三仓/外部调用后迁移；真实构造/导入正反控与参数移除编译反控待执行。 |
| Core `Dialect` / `ExternalDialect` / Override/Append | parser-api 是轻端口 owner；SDK parser 三段派发，语言外部方言 fixture 等消费 | parser-api Spec 明确保留非 UI 三态与旧 Dialect/ParserBridge 公开兼容控制；不能把整套非 UI 宿主契约当死代码删除。确定可退出范围前须映射全部真实消费者。 |
| `UiParseContext` | parser-api Spec 称旧 UI 端口已退出；UI crate 与 `ui-grammar-consumer` 使用同名别名 | 需区分 parser-api 已退役的旧接口与 UI owner 的现存别名，明确 canonical 入口和 fixture 迁移后再验证。 |
| 旧 UI feature 与路径转发 | `auto-lang` 上层 → UI 实际 owner | runtime Spec 明确旧 UI 路径和 feature 组合保持；可否变更及具体版本迁移/支持期尚待逐项核验。 |
| 旧数据/producer/cache/install | UI codec、生成/安装路径和 792 事务 | 841 与 832 交付收据、生成发行 Spec 为输入；待核实各数据版本及可恢复载荷、真实拒绝和恢复 oracle。 |

## 执行入口及关闭条件（待补齐）

T-01 必须枚举所有受管消费者、支持承诺、替代入口、退出版本和验证控制，补 producer→owner→consumer、default/缺席/多实例/退出与迁移事务的实际路径。每一候选需真实合法对照及能区分错误实现的反例，确认探针来源、输入、命令及结果；上述只是静态初查，尚无 probe、逐行处置结论或已完成 AC。遇到已有稳定承诺不能遵循时按计划 needs_replan 路由；绝不以全部保留充当 T-01 关闭。

## 初次定向证据

- `cargo test --manifest-path test/fixtures/modularity/ui-grammar-consumer/Cargo.toml --test v01_light_grammar`：6 passed，0 failed，0 ignored；实际使用 UI 同组树中的 `UiParseContext` 及 parser-api，验证 toy port 的对象安全和 widget/store/view/style 真实文法消费。该正控只能证明当前别名消费，不能证明移除后的兼容性或完整三仓迁移。fixture fresh 解析生成了本地 Cargo.lock，依赖身份随后冻结时应记录。
- `AuraViewBuilder::with_registry_and_imports` 的定义体确认末参 `_import_stmts` 未读取；生产 `dynamic.rs::view_with_debug` 实际传入 `self.import_stmts`，另有多个视图构造及测试调用。一个别名/参数的静态发现不能代替全矩阵或合法/失效控制。

## 已核实的生产路径及未决关闭条件

- UI host：`auto-lang` 的 `plan814_ui_compat_install` 通过 `UiCompatInstaller` inventory 注册；SDK 的 `install_ui_host_capabilities` 先查显式 hook，缺席才迭代 inventory，最终由 `auto-ui-runtime::install_ui_host_capabilities` 注册 provider。`create_vm_from_source`、`eval_config_with_vm` 等真实入口消费该链。不能只删登记项而丢默认装配；需分别证明 explicit/无 hook/无 UI、多实例、线程与退出，以及替代入口接线后的相同消费结果。
- 792 来源：`auto-ui-tools/src/rust_ui.rs::delivery_crate_source` 先查 producer 语言根的 `crates/<name>`，再查 `AUTO_UI_SOURCE_ROOT`、组内兄弟 UI；发行 Spec 的 PLAN-831 条目也明记该顺序。842 要求 canonical 源且坏根不回落，必须先区分合法安装 bundle 和旧残留，制定版本/迁移与正反控，不能只移除所有 producer 根读取破坏安装 SDK。入口 `delivery_producer_root` 还包含显式环境、安装邻接 SDK、CWD 与编译期候选；其中多项属于现行安装承诺。
- Man 安装：`scripts/man-install.py` 用 staging、唯一 backup 和失败回滚；`docs/installation.md` 给出既有备份恢复。版本迁移反例需要验证真实旧字节与完整恢复，而非仅检查错误码。

以上尚未枚举完整入口/版本/三仓消费者或形成最终冻结决策，不是 T-01 完成结论。

- `cargo test --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml --test r1_scope_expiry`：12 passed，0 failed，0 ignored，绑定同组三仓基线、down `c39ac7f9`、Rust/Cargo 1.98.0；真实 Parser + ExternalDialect 覆盖 Ident/token 的 Some/None/Err、槽重用后 scope expiry 和合法 no-hole 控制。该测试验证已支持的生命周期，不构成整个优先级或退役合同通过。构建日志出现大量既存 warnings，后续健康审查必须归因。
- 两次 fixture fresh 解析锁 SHA-256：ui-grammar `80cad5383795c5ad430524e40fa1da3e7513ec2850a7d08ab42528efe43cd34b`；ui-dialect `828bada75fb85ef0f551b1156b41df0184602361914ca580d4f0e1a8ee0abef1`。两锁为本次生成的未跟踪文件，保留以复现；不代表交付锁已冻结。
- UI-owned codec 当前 `UI_AST_SCHEMA_VERSION=1`、domain `auto-ui`，先完整反序列化并检查尾部，再校验 domain/schema 和领域语义；UICache 指纹含 generator 与 schema。该静态观察不等于真实旧版本升级/回滚验收。
- UI 树 `cargo test -p auto-ui-dialect --lib codec_rejects_bad_domain_version_and_tag`：1 passed，0 failed；实际断言错误域、schema 0、未知 tag 分别具名拒绝。配对 `cargo test -p auto-ui-dialect --lib codec_roundtrip_preserves_domain_core_source_and_clone`：1 passed，0 failed；验证合法 envelope 往返。两项均为 841 源码未变的当前基线控制，不证明旧安装/producer 升级恢复。
- feature 候选 `use-bigvm = []` 明确标 deprecated 且无效果，但 Cargo feature 名是公开源码接口；须先清点真实受管消费者、历史支持和可退役版本，不能仅凭空实现删除。`ui-*` 默认与转发则有明确现行承诺。

## 本轮 T-01 探针（2026-10-27）

- 别名受控消融：临时将 UI 文法内部的 `UiParseContext` 改为 `DomainParseContext` 并撤销旧再导出；真实 ui-grammar-consumer 旧导入编译以 E0432 拒绝。该 fixture 的 src 与 v01 测试切至 `auto_parser_api::DomainParseContext` 后，`cargo test --locked --manifest-path test/fixtures/modularity/ui-grammar-consumer/Cargo.toml --test v01_light_grammar` 6/6、0 ignored。探针完成后两仓 tracked 源码字节恢复。该实验证明该消费路径可迁、检测旧符号消失；不代替其他消费者和正式回归。
- feature 受控消融：`ui-dialect-consumer` 临时启用 `auto-lang/use-bigvm`，`cargo test --locked --offline --manifest-path test/fixtures/modularity/ui-dialect-consumer/Cargo.toml --test r1_scope_expiry` 12/12；随后删除空 feature，原真实消费者 Cargo 解析 exit 101，具名 `does not have that feature`。两文件恢复。说明该 feature 虽无运行效果仍是源码兼容面，必须具名迁移与版本化，不能以无实现为依据宣称无破坏。
- 语言基线相对 841 落地 f16bb468b 在 crates/Cargo.toml/.cargo/scripts/test-modularity 范围 `git diff --name-only` 为零；这只能支持相关未变材料按输入逐项评估复用，不能直接将完整 792 收据搬作当前 T-01 正反探针。
- 已识别三仓候选文件索引 [source-inventory.json](source-inventory.json) 与 [退役矩阵草案](retirement-matrix-draft.md)；前者是文本检索，含注释/历史材料并可能漏动态消费者，尚未完成逐项人工核销。832 安装采用 combined-crates：`source_closure.packages` 标记源仓/安装路径、`files` 记录 hash，故真实已验安装 SDK 和语言仓陈旧 UI 残留路径需要身份区分。

### T-01 已复现：安装清单版本不参与 verify 判定

入口 `auto-man/scripts/man-install.py verify --profile core`，构造仅含该 verifier 要求的 4 个有 hash 的合成文件，`sources` 只有 lang/man：清单 `man-install/1` exit 0；仅将 `schema` 改为 `man-install/999`，其余文件/哈希完全不变，再调用同一入口仍 exit 0。该输入是合成形态，不包含可执行的真实 auto 二进制，只证明当前 verifier 对未知 schema 不拒绝，不证明真实安装能启动或升级。代码 `verify` 读取 profile/files/sources 而未检查 schema；`install` 现以 manifest 文件存在判断可管理目录，不能假定 verify 的缺口只在报告面。842 AC-04 要求未知版本具名拒绝，T-05 需要对真实新旧版本、坏结构及失败前旧字节/恢复一起验收，不能只补 schema 字符串单测或把合成正控当实际安装。首次执行因隔离临时目录写入拒绝而失败；同组专用 scratch 重试成功并清理。

### T-01 已复现：旧目录抢占规范 UI 源并进入实际交付

1. 在 UI 实际 `rust_ui.rs` 内临时添加同模块测试 `plan842_current_source_selection`：合成语言根无旧包、`AUTO_UI_SOURCE_ROOT` 指向 UI 根含 marker `canonical` 时，`delivery_crate_source(lang, "auto-ui-dialect")` 读取 canonical；再在语言根建立同名目录 marker `old-shadow`，同一函数实际读取 old-shadow。`cargo test --locked --offline -p auto-ui-tools --lib plan842_current_source_selection -- --test-threads=1` 1/1 通过。两段对照只证明 selector 行为。
2. 进一步在组内语言 `crates/auto-ui-dialect`（事先确认不存在）临时复制当前 UI 的真实 dialect crate，额外写 `PLAN842-SHADOW=old-shadow`，显式 `AUTO_UI_SOURCE_ROOT` 指向组内 UI；在现有生产 `rust_ui::tests::delivery_bundle_end_to_end` 的真实 `generate_rust_ui_delivery` 调用后插入断言，检查交付 bundle 的 `crates/auto-ui-dialect/PLAN842-SHADOW` 字节为 `old-shadow`。`cargo test --locked --offline -p auto-ui-tools --lib delivery_bundle_end_to_end -- --test-threads=1` 1/1 通过，故影子源确实进入生成包，而非仅 selector 辅助函数返回；该单测不构成完整外部构建/安装升级通过。测试源码字节恢复，影子目录仅检查绝对路径/marker/无链接后删除，工作树最终状态再核。
3. 当前 831 Spec 明记 primary 语言根优先；842 的 canonical-only 目标要求修改此行为。832 的合法安装 SDK 恰含相同布局，不能按目录名单独判旧或盲目拒绝；必须用受管安装清单的 owner/path/hash 与真实可用安装身份区分，旧/坏清单不可当可信证据。最终 T-03/T-05/T-07 应保存此同名双源控制为真实入口永久回归，并含合法安装、显式坏根、只剩旧源、全部缺源、生成前保旧、外部 locked 构建和恢复。机制与完整版本迁移未冻结。

## r2 T-01 宿主及消费者探针

- 三仓当前已跟踪文件逐行索引 [tracked-inventory-r2.json](tracked-inventory-r2.json)，lang `ecc787011`、UI `82217b759`、Man `03e825279`；含注释与历史脚本，不能把命中数当实际消费者数。`auto_lang::{ast,parser,session,error}` 分别 29/17/16/17 个语言仓文件（含注释）；UI 旧别名 8 个 UI 文件、9 个语言文件；Aura 末参调用/定义 15 行、2 UI 文件；`use-bigvm` 两处均在一个 Cargo.toml。动态/非跟踪/仓外消费者与类型聚合导入仍需逐项核对。
- `CARGO_TARGET_DIR=.../ui-dialect-consumer/target cargo test --offline --manifest-path test/fixtures/modularity/native-provider-consumer/Cargo.toml --test provider_vm -- --test-threads=1`：6 passed、0 failed/ignored。default-features=false 的外部真实 VM：无 provider 0/0，显式 A=17/31、B=43/59，两活实例各维持自己的快照；新产生的 fixture Cargo.lock SHA-256 `3f7d4c5c8c0a0dd0df26312a3292f504d344dff302ae2585262f0a6548c01fc8`。该 no-UI 形态不能代替 UI 默认列。
- 已编译的 UI 默认外部 fixture 临时加入测试，在 `run_with_capture` 入口：空槽真实运行后 provider Some、frame stdout 0/0；显式 A、B 分别真实输出 17/31、43/59，1/1 通过，探针撤除。
- 区分性反例：另在 UI 默认外部 fixture 的进程中直接调用 SDK 再导出 `auto_lang::run_with_capture_and_bytecode`，清空 provider 后真实运行并检查进程默认 frame provider：原实现 1/1 通过；仅临时禁用 `inventory::submit!(auto_lang_core::UiCompatInstaller(...))`，同一测试 exit 101，`UI default not installed on direct SDK path`。源码和临时测试已恢复。结论：兜底不是当前可直接删除的无用项，先迁移/显式接线所有直达调用、保默认/显式/缺席/多实例/线程及退出，再永久复测；受控探针不是将来永久回归。


## r2 最终安装修复及进一步候选（2026-10-27）

T-00 最终源码 UI `49aa54d6e18e8d893be69b9430022aea14624ce7`、Man `7cc37fa364a2f59bedc2c920638f8007cd0075ce`；lang/down 未变。[最终真实安装正反控](t00-scoped-verification/report.md) 通过。当前三仓受管路径候选 [tracked-inventory-r2-final.json](tracked-inventory-r2-final.json) 已刷新，新增 841 中期交接的 module-resolution/生成入口/缓存 setter；它不是完整未知外部用户清单。841 最终 handoff 与 Spec 已明确其空 `assemble_ui_grammar` 桩被 841 删除，不能按中期清单重复认领。明确设置无效源码根时，最终安装 CLI 实际生成仍成功且 manifest 显示回落邻接 SDK；复现与控制见 [矩阵](retirement-matrix-draft.md) R10。


- 最终候选索引 `tracked-inventory-r2-final.json` 按受管路径 Git 搜索：旧 AST/parser/session/error 相关分别语言仓 28/19/16/16 文件，UI facade 语言 56、UI 9、Man 4 文件（每组含注释、测试和示例）；其中 UI 工具 Rust 模板与 Man 命令是生产消费。当前真实安装生成包的 main.rs 确有新 runtime 及旧 auto-lang UI 路径混用，见矩阵；该包锁定构建通过不能代替旧引用退出。
- 未知 schema 的真实升级准备反例：Core 安装 schema 临时改 999，正式 installer `--fail-point prepare` 返回注入故障，未提前拒绝该版本；旧可执行文件未变、恢复清单后完整 verify 通过。该故障注入未执行替换，T-05 仍需真实旧产物保护与恢复覆盖。
