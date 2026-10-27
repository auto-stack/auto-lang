# PLAN-842 实施交接（T-09，审查稿）

> 独立复审 R6 的问题已修复并完成真实时序及多来源对照，见 [R6 修复证据](repair-r6/report.md)；仍待独立最终复审，不能当作 pass。

> 独立复审 R5 问题已修复并以真实正反控验证，见 [R5 修复证据](repair-r5/report.md)；本文仍须新一轮独立最终复审，不能作为 pass。

> 独立复审 R4 的问题已修复并完成真实反例，见 [R4 修复证据](repair-r4/report.md)；本文尚须独立最终复审，不能作为 pass。

> 独立复审 R3 的完整 SDK 与复制期来源问题已修复并完成真实反例，见 [R3 修复证据](repair-r3/report.md)；本文尚须独立复审，不能作为 pass。

> 独立复审 R2 的问题与随后发现的批量测试选择缺口已完成修复和定向验证，见 [R2 修复证据](repair-r2/report.md)；本文尚须独立复审，不能当作 pass。

## 工作期冻结与边界

计划 r2 的 T-00..T-08 已完成；来源为 lang `a8b4a5bcf53d9be484cc38f028453a4cebbc947f`、UI `bd0e0f38253c5fd0676c8ef729c8c030d372ae82`、Man `9ac33ad871c4e0278b322e9712930ed059029e9c`、down `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`。上述不是已合入或已发布声明。最终合流可能改变 Git 身份，必须检查内容/依赖/配置/输入并重审受影响证据，合入前不得放行后续计划。

三个破坏性接口包实际版本 0.2.0：`auto-lang`、`auto-ui-dialect`、`auto-ui-runtime`。旧四个语言模块、两处 `UiParseContext` 别名、旧 Aura builder 和 `use-bigvm` 已退出；[迁移与旧版恢复](migration-0.2.md) 列出实际替代和旧固定源。正式 `auto_lang::Parser`、UI/back_proxy/host_adapters、UI build、find/resolve 薄 API、非 UI 外部方言及 SDK hook 继续支持。`DynamicComponent::with_registry_and_imports` 仍有真实导入语义并保留。

特别区分两种 inventory：841 的 **UI 文法登记已退出**；宿主 `UiCompatInstaller` 登记仍承载已链接 UI 语言包但直接进入 SDK 的默认 provider 语义。T-06 实际删登记即红、恢复即绿，按照冻结 R08「只退役已证明不必要的兜底」保留现有单份实现；显式 hook/installer 与显式 provider 优先照常。不得在落规范时把这一必要保留描述为已删除或默许未来删除而改变默认。

## 精确 Spec delta

| ID | 存放目标 | 应写入的现状、退出和验证边界 |
|---|---|---|
| SD-01 | `docs/specs/auto-lang/project.md`、`docs/specs/language-sdk/project.md` | 0.2.0 退出 `auto_lang::{ast,parser,session,error}` 模块和 `use-bigvm`；分别指向 auto-ast、SDK parser、parser-api、diagnostics。根 `Parser`、正式 UI/构建/resolve 薄接口和非 UI 外部方言保留。SDK 纯 Core 无 UI 时 hook 缺席/no-op；已链接 UI 语言包的直接 SDK 默认需现有 inventory，显式 provider 优先，双实例/线程/退出边界及 T-06 消融证据列明。 |
| SD-02 | `docs/specs/auto-parser-api/project.md`、`docs/specs/auto-ui-dialect/project.md` | UI dialect 0.2.0 移除两处 `UiParseContext` 名称，使用 `DomainParseContext` 显式组合；`ExternalDialect`、Override/Append 非 UI 合同不删。UI grammar 的旧 `UiGrammarSource`/`assemble_ui_grammar` 仍退出；codec schema 1 合法、旧/未来 schema、错误 domain/tag 与尾随非空数据拒绝。 |
| SD-03 | `docs/specs/auto-ui-compiler/project.md`、`docs/specs/auto-ui-runtime/project.md` | runtime 0.2.0 退出 `AuraViewBuilder::with_registry_and_imports`，改为 `with_registry`；`DynamicComponent` 同名不同方法仍处理导入。生成器及 29 示例直连 runtime/compiler/SDK；宿主入口安装 SDK hook 与 runtime provider；保默认、显式、缺席、实例/线程/退出。旧 cache identity 失效并重建；MPV-native 显式含 libloading，真实 DLL/视频播放未在本轮实测。 |
| SD-04 | `docs/specs/auto-man/project.md`、`docs/specs/auto-cli/project.md`、`docs/specs/repository-boundaries/generated-ui-distribution.md` | Man/生成后端使用 canonical owner；指定语言/UI 根权威、错误不回退；非受管语言/UI 源验证本地 Cargo 依赖图、每个包自身版本、包身份及约束，允许合法 dev 环和重命名；受管安装以文件系统身份识别同一 SDK 的路径别名，并冻结各个实际选中语言/UI 来源自己的收据；staging UI 原字节按所选 UI 收据校验、提交前复核，来源版本记录各自收据；开发 UI owner 与受管 Full 安装按 owner/path/hash 校验并记录各来源；生成 Rust 应用显式装配宿主、无 auto-lang 直接依赖。`generated-ui-delivery/1` 仍支持，未知版本/坏 producer 拒绝、旧产物及未知版本备份保护；fresh/locked、资源失败和恢复是实测门。 |
| SD-05 | 新增 `docs/specs/repository-boundaries/compatibility-retirement.md` | 用 [冻结矩阵](retirement-matrix.md) 精确记录退出、正式保留、0.2.0 边界、旧固定源与迁移/回退、必要宿主登记；不把历史脚本/负向 probe 当作生产消费，也不声称调查了未知仓外消费者。 |
| SD-06 | `docs/specs/repository-boundaries/generated-ui-distribution.md`、`docs/specs/repository-boundaries/man-three-repository-delivery.md` | `man-install/1` Full SDK 的合法中间投影与源码/已投影态均可交付；按 TOML 表作用域识别、异常混态拒绝。Core 不含 UI；安装器在准备替换之前验证已有 schema/profile/文件，未知或损坏拒绝并保留。0.1 真实旧安装升级、backup 恢复及 prepare/swap 反例已有证据；清单 hash 是一致性而非签名。 |

## 接收与待完成

- 退役/保留逐行、测试 ID/别名/锁和 777/792 边界见 [T-08 审计](t08-audit/report.md)；原 T-02..07 证据分别位于对应附件。七份本次生成的 fixture lock 已逐字节保存在审计目录，按 manifest 可恢复；新宿主 fixture lock 已受管。
- 实际旧/新安装、Rust/Vue/SDK 外部消费和失败恢复见 [T-07](t07-progress/report.md)，含审计补测后重新绑定及前版记录。部分旧语料测试 1 项依原规则 ignored；本机无 MPV DLL/视频实机播放证据；已报告编译 warning 未冒称清零。
- T-10 独立 review 逐 AC-01..08、SD-01..06、源码/consumer/反例、遗漏/延期/绕过、格式和 warning 核验，结果必须绑定实际三仓提交及配置；若发现问题必须修复并重新审受影响范围。
- review pass 仅表示验收与规范审查；裸 `cargo t` 和适用触面档、实际有序合入、来源读回、ledger/索引/归档与 fresh wt-guard 是 T-11/T-12，当前均 pending。不得将此交接当作最终交付或放行 PLAN-843。

## Work 验收就绪记录

| 验收 | 工作阶段证据与限界 |
|---|---|
| AC-01 | [R01–R16 逐行](t08-audit/report.md) 与 [外部旧入口反例](t04-verification/report.md)；必要宿主登记按冻结条件具名保留。 |
| AC-02 | [T-03 生产与源](t03-progress/report.md)、[T-07 最终安装/外部消费](t07-progress/report.md)；历史脚本及负向 probe 不视作生产调用。 |
| AC-03 | [T-06](t06-progress/report.md)：纯 SDK/无 UI、默认、显式、两个 VM、线程/退出与消融；[T-07](t07-progress/report.md) 生成应用宿主。 |
| AC-04 | [T-05](t05-progress/report.md)：0.1→0.2 精确版本、未知版本/坏输入、真实 Core/Full 升级与备份恢复、cache codec。 |
| AC-05 | [T-08](t08-audit/report.md) 测试 ID、alias、锁及边界；[迁移说明](migration-0.2.md) 与上述 SD。 |
| AC-08 | [T-00](t00-scoped-verification/report.md) 原始合法安装投影反例修复；[T-07](t07-progress/report.md) 以审计后来源再次安装、fresh/locked 与坏源保旧恢复。 |

R1/R2/R3 问题及批量测试选择缺口已修复并以[最终提交重验](repair-r3/report.md)，待独立复审确认；当前无已确认的未修复 work-owned finding 或待用户决定的合同事项。真实 MPV DLL/视频播放未实测，原规则下仅编译和缺库降级有证据；原 grammar 长等待用例 1 项依既定隔离规则 ignored，不计作通过。现存 warnings 及完整遗漏审查交独立 review 具体核验，不能以本记录宣称零 warning。前代 T-05 升级/回退所需安装器、模型及输入未被后续来源验证或测试驱动变更修改，按具体证据复用；最终源码安装、生成和消费已在 T-07 实际重新绑定。

统一裸 `cargo t`、适用触面档与批量门到期判断由 merge 在最终审查通过后执行，现为 pending；实际三仓合入、来源/Spec/ledger/索引、归档和 guard 清理均未执行。

## r3 增补（2026-10-27，T-13..16 + r11 重绑）

- **SD-07**（`docs/specs/auto-man/project.md` 新节「merged API 客户端臂语义」）：臂优先级 ①db 吸收 ②route A（标量契约+无 db+转译成功，`[]str/[]int` 属标量面）③声明类型臂 ④viewer ⑤CRUD 脚手架；兜底按 PLAN-710 D-7 契约收口（`[]str → Vec<String>`、列表缺省 `Vec::new()`）；PLAN-648 `Vec<Value>` 全量列表形态退役；route A 转译失败生成物留 `⚠ route A inactive` 标记（不静默）。
- **SD-08**（`docs/specs/auto-ui-compiler/project.md`「准入与解析失败双轨」+ `auto-man/project.md`「解析失败与受信准入双轨」）：前置预校验只硬拒受信准入失败（domain/module，任何模式、先于缓存/输出）与严格模式；语法解析失败普通模式保持逐文件 warn-and-skip、严格失败、fresh/增量一致；双轨不得互相吞并。
- **AC-09..12** 证据：[t13-t16-r3 报告](t13-t16-r3/report.md)（含 T-15 真实生成/编译/GET/POST 行为差异 oracle 与 T-16 双轨反例）。
- **r11 源码/安装重绑**（r3 修复后在 lang `6f71646ac`、UI `bd228b0`、Man `4cc912e`、down `c39ac7f` 上）：Full 实装 6813 文件 verify 通过（清单 SHA-256 `955FA96E3BF0349953FE6258D9E128901387CECC5176CE8651F095F3C293F0C8`）；安装 CLI 真实 Rust 交付生成 5434 文件 + fresh lock + `cargo build --locked` + 资源校验 PASS；真实 Vue 生成（生成物含 8 个 status token 双面正确值）+ 生产构建 dist；Core 实装 636 文件 verify + SDK locked check + 独立外部消费者 fresh lock 3/3 + 安装 exe 真实程序 `plan842-core-ok`。失败恢复/staging 保护与 relocate/upgrade 证据按输入身份复用：保护逻辑（man-install.py/交付预检）自 r10 后未改（Man 4cc912e 仅测试文件、UI r3 改动不触交付保护码），耐久反例 `delivery_redelivery_failure_preserves_old` 在新代码 1/1 绿。
- 测试 ID 增量（T-08 审计延续）：UI lib 新增 3 项（`merged_api_client_route_a_wins_for_scalar_contract`、`merged_api_client_route_a_transpile_failure_leaves_marker`、T-15/16 扩展断言并入既有具名测试），无删除/改名；Man 冒烟测试原名单保留。
- 前节 r2 记录对 AC-01..08/SD-01..06 继续有效；本节不重开既有结论。裸 `cargo t`/触面档/批量门/合入/归档仍 merge-owned pending。
