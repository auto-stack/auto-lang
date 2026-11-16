---
plan_id: PLAN-1032
status: execution_done
feature_name: ACE typed native批量编辑失败原子性
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-16
plan_revision: 3
current_step: 3
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-mir/native-typed-entry.md, docs/specs/auto-ac/mir-native-cache.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1032] ACE typed native批量编辑失败原子性

## 0. 变更摘要

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F24，优先级**P1**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r3阶段裁定：F24全部A-required，先关闭原NTX-01/02与对应独立oracle再供985/A；虽然是native typed批编辑，但真实mn-worker/typed_entry消费者属于A的AC基础/测试接受态；不能因AC桌面在B而整体延期。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

edit_units的doc/residents/history/generation作为同一接受事务；任一prepare/admit/emit/store失败不能部分安装，后续run/同批重试可用。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。typed入口与source入口分层，不扩大typed公开能力；目标物理表示/错误码保持。

## 2. 架构方案

先全部候选构建并验证，再统一切换；尝试观察可增加，接受态保持。保签名依赖重准备和A→B→A真实COFF复用，不拿PE文件发布事务代会话事务。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。唯一树 `D:/autostack/.wt/lang-1032/auto-lang`，分支 `plan-1032-dev`（2026-11-16 建，基面 master `16a58402e`；仓库约定分支名，取代 §3 原“建议 codex/plan-1032-dev”）；work/review/merge 顺序使用同树，禁止链接。材料唯一 `D:/autostack/plan-artifacts/plan-1032/`，已在 `.retention/policy.json` 登记（owner/用途/状态 in-flight，merge 收尾删除）。实现提交 `6d3e638d7`。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/native/src/typed_session.rs](../../experimental/ace/crates/native/src/typed_session.rs) | 29186796e89fe450877d7f09079cfc629d7c18c85c9c04866a45a663cb750c38 | 静态取材，非运行证据 |
| [experimental/ace/crates/native/src/typed_fixture.rs](../../experimental/ace/crates/native/src/typed_fixture.rs) | e4d65e4d80f8d66f3efc11d6c8828830b77b56e50fea8b9fca549f27de5005a0 | 静态取材，非运行证据 |
| [experimental/ace/crates/native/tests/typed_entry_incremental_tier.rs](../../experimental/ace/crates/native/tests/typed_entry_incremental_tier.rs) | b7508d6348a4da79786aa984d537560c3f9649e117cc7029d23d326f94389e5c | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-mir/native-typed-entry.md](../../docs/specs/auto-mir/native-typed-entry.md) | f2de377e354771e9bee7b906d9cf482c08724d78d565c410af61ec4156a7a4da | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-ac/mir-native-cache.md](../../docs/specs/auto-ac/mir-native-cache.md) | e0dbd77304c7cfe8d1e99c4a8d2eb35c38c4e1833e61ca25f5a690870fe2a395 | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1笼统写invalid替换/emit/store失败，没有精确FixtureDoc、失败顺序、私有history与attempt计量边界，可能只覆盖未知unit早拒。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/native/tests/typed_entry_incremental_tier.rs | b7508d6348a4da79786aa984d537560c3f9649e117cc7029d23d326f94389e5c | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F24。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：edit_units的doc/residents/history/generation作为同一接受事务；任一prepare/admit/emit/store失败不能部分安装，后续run/同批重试可用。 先全部候选构建并验证，再统一切换；尝试观察可增加，接受态保持。保签名依赖重准备和A→B→A真实COFF复用，不拿PE文件发布事务代会话事务。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** 复用typed_entry_incremental_tier.rs的BASE（mn-typed-fixture/1，helper identity、disjoint const1、main调用helper，entry main）。输入i64 bits=0000000000000010，正常结果同bits；entry_plan、residents()/fp/COFF与每unit generation可公开观察，doc/history/全局generation由typed_session.rs内部cfg(test)快照核，不增生产public访问器。

**状态、故障与责任。** 全部fallible产出在候选侧完成，再一次接受doc/居民/history/generation。ManagedStore中无根可回收产物不要求物理零写，但失败不能改变本会话可执行接受态。T-01必须定位现obtain_object/cache写的可控故障；现接口不足时只新增局部cfg(test)一次性failpoint，不能通过skip缺工具或把failpoint放在事务前证明晚失败。fixture/PE文件publish测试不能替代会话事务。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F24 → 原NTX-01/02 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：虽然是native typed批编辑，但真实mn-worker/typed_entry消费者属于A的AC基础/测试接受态；不能因AC桌面在B而整体延期。关闭谓词：helper先合法、后项晚失败时doc/residents/history/generation/target保持旧，实际run旧0x10且同批修复重试0xff；真实COFF/合法单项邻例按原r2。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-mir/native-typed-entry.md | 现规则/支持域保持；补充以下修复后的明确合同：typed batch与source接口分别保同一接受事务；明确失败history不消耗、旧entry_plan/COFF可用和attempt观察分账。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-ac/mir-native-cache.md | 现规则/支持域保持；补充以下修复后的明确合同：缓存无根副产物可回收不等于部分接受；保当前runtime身份与A→B→A字节复用。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-native。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| NTX-01 | AC-02 / T-02 | 单项invalid替换；多项首项合法、后项verify失败 | TypedSession::edit_units→prepare_one→后续entry运行 | Err后doc/居民/历史/generation及旧entry_plan/COFF全保旧 | 已实施并运行：`audit1032_ntx_01_failed_batch_keeps_previous_entry_runnable` PASS（1.065s/1.424s 复跑，见 §9） |
| NTX-02 | AC-02 / T-02 | emit/store故障及被签名变化影响的dependent失败；同批重试 | 同入口及原typed incremental tier | 无部分提交；清故障重试成功；A→B→A复用与真实值保持 | 已实施并运行：`audit1032_ntx_02_batch_failure_is_atomic` + `..._signature_change_pulls_real_dependents` PASS（1.42s/0.63s，见 §9） |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

替换序列[helper_B,disjoint_BAD]：helper_B保持签名(a:i64)->i64，body为const t0 i64 bits=00000000000000ff，ret t0；disjoint_BAD保原已知unit名与i64结果，body=[]，ret="missing"。这样两者都通过fixture JSON结构读取，后项在真实build_verified_unit/verify失败，而不是“未知unit”前置检查。另将BAD放首位作单项/首项邻例。失败后run(0x10)仍0x10；修复BAD后同批重试才得到0xff，不能仅断言Err。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| NTX-01 | 后项verify失败、首项失败、签名变化dependent失败 | doc/residents/history/generation/entry_plan接受快照逐项不变，旧COFF字节/身份保持；run仍0x10 |
| NTX-02 | 历史hit后半批失败、emit/store失败、清故障同批重试、A→B→A | history不能被失败候选消耗；成功后0xff、返回A后0x10且原真实COFF复用；attempt计数允许增加 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| NTX-01 | audit1032_ntx_01（实际落地 `audit1032_ntx_01_failed_batch_keeps_previous_entry_runnable`，公开状态oracle） | [experimental/ace/crates/native/tests/typed_entry_incremental_tier.rs](../../experimental/ace/crates/native/tests/typed_entry_incremental_tier.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行（已核，见§9） |
| NTX-02 | audit1032_ntx_02（实际落地 `audit1032_ntx_02_batch_failure_is_atomic` + 邻例 `audit1032_ntx_02_signature_change_pulls_real_dependents`，内部接受快照oracle + cfg(test)一次性emit/store故障注入） | [experimental/ace/crates/native/src/typed_session.rs](../../experimental/ace/crates/native/src/typed_session.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行（已核，见§9） |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

本轮A阶段交接：NTX-01/02的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。证据：源/Spec 三文件字节哈希与 §4 表逐一相同（typed_session 29186796…、typed_fixture e4d65e4d…、tier 测试 b7508d63…、两 Spec f2de377e…/e0dbd773…）；新反例全部经真实 `TypedSession::edit_units`→`build_verified_unit`/`verify_mir`/`admit_typed_unit`→真实 PE 入口运行（非未知 unit 早拒），未改任何既有 expected。
- [x] AC-02 / work：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。证据：NTX-01/02 三锚点新增并实测 PASS；失败后 doc/residents/history/generation 与旧 entry_plan/COFF 全保旧的独立断言成立；partial-install 变异对照 3/3 定向红（§9），证明 oracle 能区分原缺陷。
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。证据：固定组 `cargo t ace-native` PASS（298/298 executed，0 excluded/not_run，gate_passed，执行期 checkout 未漂移）；原三例 warm/A→B→A/param-only、签名变更、冷恢复全绿；实质调用者仅同 crate（`typed_entry_tier.rs`、本 tier、`mn-worker` bin 无 `#[test]` 体），无跨仓消费者；SD-01/02 文本仍按 §5 供 merge 沉淀；无 1002 硬前置（未触作者格式/IR/adapter）。
- [ ] AC-04 / review：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 用原BASE和具体replacement接通后项失败、历史hit/依赖重准备及写入故障；冻结私有接受快照与现工具条件，避免未知unit早拒。 AC-01 ✅ 提交 `6d3e638d7`：`acceptance_snapshot`（cfg(test)：doc/residents/history/generation）与一次性 emit/store failpoint 落地；真实入口可达后项 verify 失败、签名依赖失败与写入晚失败。 |
| T-02 / work | [x] T-01决定与可区分输入 | doc/units/history/generation全部build-aside，在全部verify/admit/emit/cache成功后切换；补NTX永久断言，保真实COFF复用。 AC-02 ✅ 提交 `6d3e638d7`：`edit_units` 改为候选文档+`prepare_candidate` 暂存+`commit_staged` 单点切换；history 命中只读、提交时消费；A→B→A 字节等同原 COFF。 |
| T-03 / work | [x] T-02 | 对每种失败复核旧run与同批重试、A→B→A、param-only与signature-dependent；运行typed_entry_incremental_tier及native职责组，工具缺失不计成功。 AC-02/03 ✅ `cargo t ace-native` PASS：298 executed/298 selected、0 not_run/0 excluded、gate_passed、source_state `15ad563b…`、wall 679.93s（工具准备 577.8s/12项、发现 34.85s、执行 66.80s）；工具齐备（rust-lld/SDK 可用，无 infra-blocked）。 |
| T-04 / review | [ ] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1笼统写invalid替换/emit/store失败，没有精确FixtureDoc、失败顺序、私有history与attempt计量边界，可能只覆盖未知unit早拒。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。 SD-01归属从source function Spec校正为实际typed入口auto-mir/native-typed-entry，保原失败原子性义务。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原2个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-16 stage:new/revision | PLAN-1032 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F24/原NTX-01/02明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

2026-11-16 stage:work | PLAN-1032 | r3 | outcome:pass（T-01..T-03 完成，移交独立复审） | worktree `D:/autostack/.wt/lang-1032/auto-lang` | branch `plan-1032-dev` | base `16a58402e` | code `6d3e638d7` | deps 无（仅语言仓，无兄弟仓） | task_ids T-01/T-02/T-03 | blockers 无 | next: review。

**实现（提交 `6d3e638d7`，3 文件）**：`typed_session.rs` 的 `edit_units` 改为「候选文档 + 逐单元 build-aside（`prepare_candidate`）+ `commit_staged` 单点切换」：doc/residents/history/generation 只在整批全部 verify/admit/emit-or-reuse 成功后一次安装；history 命中在暂存期只读、提交时消费；unchanged 指纹仍不 bump 代际。`typed_fixture.rs` 六个声明加 `PartialEq/Eq`（供内部接受快照等值断言）。新增 cfg(test) 一次性 emit/store failpoint（thread-local、无生产表面对外 API）与 `acceptance_snapshot`（仅测试内可读，未增生产 public 访问器）。store 写入为内容寻址可回收副产物，晚失败不改变会话接受态（符合 §5 边界）。

**新增永久断言（3 锚点，均真实入口）**：
- `audit1032_ntx_01_failed_batch_keeps_previous_entry_runnable`（tests/typed_entry_incremental_tier.rs，公开态 oracle：resident(fp,generation,COFF hash)/entry_plan/实际 run）：单项非法、合法首项+后项晚失败、BAD 首位邻例、签名变更依赖失败四形态俱在失败后保持旧接受态并实际 run 仍 `0x10`；修好后同批重试 → `0x00ff`；合法单项邻例仍恰好 emit 1。
- `audit1032_ntx_02_batch_failure_is_atomic`（src/typed_session.rs 内 `cfg(test)`，内部接受快照 oracle）：A→B 后「history 命中+后项晚失败」不消耗 history、整快照逐项相等、旧闭包仍 run `0xff`；同批修复重试得 `0x10` 且 helper COFF 字节等于原 A 对象（真实复用非再 mint）；emit 第 2 次与 store 第 2 次注入失败均整快照相等、旧闭包仍 run `0x10`；故障清除后同批成功 `0xcd`，其中已持久化对象按 store 重准入（cache_hits 1/emit 1/persist 1），`whole_fallback` 恒 0。
- `audit1032_ntx_02_signature_change_pulls_real_dependents`（邻例）：callee 加宽而 caller 未跟上时依赖方 verify 失败，callee 不得被安装，旧闭包仍 `0x10`。

**失败→不变量对照（oracle 区分力，partial-install 变异对照）**：临时把提交点改为逐单元即时安装（模拟修前语义）后重建，3/3 定向红且各停在本缺陷断言上——`a dependent verify failure must not install the new callee`、`the already-prepared head must not be installed`、`a failed batch must not change doc/residents/history/generation`；还原后字节哈希 `0cce0a1d…` 与变异前一致并复跑全绿（先红后绿，非恒真断言）。

**本轮实际测试（本会话，debug/test 档）**：

| Test module / actual scope | Result and executed/excluded counts | Preparation/build | Discovery | Tests | Cleanup | Total wall time |
|---|---|---|---|---|---|---|
| `cargo nextest run -p ace-native --lib --tests -E 'test(/audit1032/)'`（3 锚点） | 3/3 PASS，0 excluded；`314 skipped` 为过滤未选 | 首跑增量编译 3.63s / 复跑 0.16s | 含在命令内，未单列 | 1.219–1.425s | 临时 workdir/store 由用例自清（临时目录） | 5.72s / 6.91s / 最终 1.25s 三次 |
| `cargo t ace-native`（固定组全量） | PASS 298 executed / 298 selected，0 not_run，0 excluded，`gate_passed=true`，`source_state=15ad563b…`（执行期变更为空） | 工具准备 577.8s（12 工具，release 档由 `toolchains.json` 既有配置决定）/ 组发现 34.85s | 34.85s（nextest list） | 66.80s（298 例，max_parallel 20，exec peak 1.09GB，job peak 6.61GB） | 无用例残留 | 679.93s（组总） |
| 变异对照（partial-install） | 3/3 FAIL（预期红，见上） | 增量 3.63s | — | 0.76s | 还原源后复跑绿 | ≈5s |

**新增用例行为/耗时（≤10s 门）**：

| New case ID | What behavior and oracle it tests | Profile | Actual elapsed seconds | Result |
|---|---|---|---|---|
| audit1032_ntx_01_failed_batch_keeps_previous_entry_runnable | 四种失败形态后公开接受态（resident fp/generation/COFF hash、entry_plan）不变且旧闭包实跑 `0x10`；修复后同批 `0xff` | debug/test | 1.065 / 0.884 / 0.773 | PASS ≤10s |
| audit1032_ntx_02_batch_failure_is_atomic | 内部接受快照（doc/residents/history/generation）等值 + history 不被消耗 + emit/store 晚失败不部分安装 + 同批重试 + 真实 COFF 复用 | debug/test | 1.497（变异红）/ 1.424 / 1.218 / 1.248 | PASS ≤10s |
| audit1032_ntx_02_signature_change_pulls_real_dependents | 依赖方 verify 失败不得安装已成功 callee，旧闭包实跑 `0x10` | debug/test | 0.631 / 0.502 / 0.588 | PASS ≤10s |

**证据位置**：`D:/autostack/plan-artifacts/plan-1032/`（`group-run/results.json`、`ace-native-run.log`、`typed_session.rs.pre-mutation`；scope 已在 `.retention/policy.json` 登记 in-flight，merge 收尾折叠摘要后整目录删除）。**未做**：未改 canonical Specs（SD-01/02 留 merge 沉淀）、未改测试期望/原样本/规模/平台/ignore、未跑 ace-perf 或全仓、未合入/归档/清树。**健康**：改动文件零新警告（`cargo build -p ace-native --lib --tests` 中无 typed_session/typed_fixture 告警），无调试打印残留。

2026-11-16 stage:goal | plan_id: PLAN-1032 | plan_revision: r3 | round: 1 | from: draft | to: work | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1032/auto-lang | commit: 6d3e638d7 | next: review

## 10. 待澄清事项

typed入口与source入口分层，不扩大typed公开能力；目标物理表示/错误码保持。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。
