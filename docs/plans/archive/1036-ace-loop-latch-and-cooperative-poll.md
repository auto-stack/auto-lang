---
plan_id: PLAN-1036
status: archived
feature_name: ACE循环continue推进、闭区间终点与协作检查
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-17
plan_revision: 4
current_step: 6
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-ac/source-collections-variants.md, docs/specs/auto-mir/collection-place-profile.md, docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/auto-a2c/collections-hir-backend.md, docs/specs/auto-a2r/collections-hir-backend.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1036] ACE循环continue推进、闭区间终点与协作检查

## 0. 变更摘要

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F28/F29/F35，优先级**P1**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r3阶段裁定：F28/F29/F35全部A-required，先关闭原LOOP-01/02/03与对应独立oracle再供985/A；continue推进、闭区间MAX与回边poll是A普通控制流/预算/取消语义，不因图形只AV而延期。原6任务/0完成/既定设计与历史保留，仅规划。

2026-11-17 r4实施（work）：用户以“完成计划1036”目标授权整链（work→review→有界修复→merge）。同一目标下建树 `D:/autostack/.wt/lang-1036/auto-lang`（分支 plan-1036-dev，基面 6de19aa3918813908f9094a93da856983a623c04），T-01/T-02/T-03完成、代码提交 d766ce811；实现在树、计划在主检出。T-01实测到r2冻结源漂移已按现码重冻（见§4 r4表），三案均有“修前红→修后绿”双向证据（见§9 work收据）。

## 1. 目标

continue执行本循环应有推进与回边poll，break/return不误推进，嵌套loop各有正确目标；i32闭区间MAX最后迭代不合成越域加一。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。F28值/迭代、F29宽度、F35预算/取消三个oracle分别关闭；不把所有目标未支持面计为PASS，不新增无限规模承诺。

## 2. 架构方案

共享latch思想但保source降糖与C/Rust emitter不同机制；cleanup/survivors按现权威事实，不在backend重推ownership。保持数组/range迭代支持域和正常预算口径。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1036/auto-lang，建议分支codex/plan-1036-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1036/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/core/src/source/resolve.rs](../../experimental/ace/crates/core/src/source/resolve.rs) | 80eb12467732dd657008918c90bf36158d52d6bbea63bc87e7c511a698263c10 | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/source/adapter.rs](../../experimental/ace/crates/core/src/source/adapter.rs) | 64fb2234a0c39875e4100fb873f6b1ef696f3d180d585072c2060931553d4300 | 静态取材，非运行证据 |
| [experimental/ace/crates/lowering/src/hir_adapter.rs](../../experimental/ace/crates/lowering/src/hir_adapter.rs) | fa54e8c60ac15ff33c6663638f8c000ae951af0d19ddde9f598b5b4758f51761 | 静态取材，非运行证据 |
| [experimental/ace/crates/a2x-rust/src/lower_coll.rs](../../experimental/ace/crates/a2x-rust/src/lower_coll.rs) | ba1e0c6c163f99d853c8a26adb630929195d870965f42c69eef8f8f8b8f5ed77 | 静态取材，非运行证据 |
| [experimental/ace/crates/a2x-c/src/lower_coll.rs](../../experimental/ace/crates/a2x-c/src/lower_coll.rs) | 3f4f3c42731ec50b56171453a6a12a41d2d9d3d1b4e41a765e3deba9948c48ec | 静态取材，非运行证据 |
| [experimental/ace/crates/a2x-rust/src/coll.rs](../../experimental/ace/crates/a2x-rust/src/coll.rs) | 2af537d6991ab128f964442a74a50a40350f185e23c15d7f24c2d6381e858e13 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-syntax/composite-source-projection.md](../../docs/specs/auto-syntax/composite-source-projection.md) | 529bc7eccd6f9028d4ad0224d5db86f66d36b54689e4e11ad1a50a44fa8bb891 | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-a2x/checked-resource-facts.md](../../docs/specs/auto-a2x/checked-resource-facts.md) | 38da0f0d66e92992b408741320616820856aed8111ec73dd3ec3d5f8249e31da | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-a2x/architecture.md](../../docs/specs/auto-a2x/architecture.md) | 9984e8854b3bd7e9fbd00a195154a0c62e78861c2d46a3121b64b188c350cfdd | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1 LOOP-01只写伪代码n++，未给可运行源；poll断言没有独立次数、预算边界、break/return及嵌套loop归属，可能值正确但仍绕预算。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs | 9103e290ef8891f41d8acc7300c50e23daf145c47a1ad489d7d23b398bbe4552 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/a2x-worker/tests/cases/cert_coll_904.rs | 7fa77ce37e70b3b0f4ff741abcc95b6b3fc35d8ec7add89fe2fcb9bbe5ade909 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_cert.rs | 43a9fdfae88925888d4ce9335bd57d1c7415df3c02ce14e8efa04bf90e60d81d | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/a2x-rust/src/coll.rs | 2af537d6991ab128f964442a74a50a40350f185e23c15d7f24c2d6381e858e13 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/a2x-c/src/coll_runtime.rs | a20c4367283912a3531a7090f84a7f85396485f6d29f130de28ad461921074d4 | C原poll走charge(1)，独立于Rust coll:poll marker；T-01据实际trace冻结预算 |\n
1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

### r4开工重冻（2026-11-17，T-01 实测，非静态引用）

开工基线 HEAD=6de19aa3918813908f9094a93da856983a623c04（与r3取材不同，全部重算）。**r2冻结hash与实际源码有差额**，本包一律以r4实测hash为准：

| 已核源路径 | r2冻结SHA256 | r4实测SHA256 | 差额 |
|---|---|---|---|
| experimental/ace/crates/core/src/source/resolve.rs | 80eb12467732…263c10 | 488d376df544f0d561da3720282b688be597bd4b2abc33f0b9e3cdd73a1fb8fd | 有（15167→15425行） |
| experimental/ace/crates/core/src/source/adapter.rs | 64fb2234a0c3…d4300 | 1ac3cb79fd69feeb933113bdf8dc39a34a2662f97e4553e0b29290dde880445d | 有 |
| experimental/ace/crates/lowering/src/hir_adapter.rs | fa54e8c60ac1…51761 | 40873354bc3a1257ace19ce7135cc0f0b216b01d347ab3df2cefb7f004b6078a | 有 |
| experimental/ace/crates/a2x-rust/src/lower_coll.rs | ba1e0c6c163f…5ed77 | dcc3dbf5f4ebcbbdb0f26302e2adce24e61f1d57dfbca337e7d6a9bcb3716e9b | 有（1867→1914行） |
| experimental/ace/crates/a2x-c/src/lower_coll.rs | 3f4f3c42731e…c48ec | a3d0c3cc176bf286a13aee2e87298f97b71a4ddf54eb46e380462c565966e8f8 | 有（1666→1711行） |
| experimental/ace/crates/a2x-rust/src/coll.rs | 2af537d6991a…58e13 | 7054bbef5bf171f6eeb288c964dcf0624497f01716460762a4ce0700f7d4cd2e | 有 |
| experimental/ace/crates/a2x-c/src/coll_runtime.rs | a20c43672839…104d4 | 0c84a2a6594ffc92d8579eac1b7340f05872c5979e3869cd47f4217f568ff3d4 | 有 |
| experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs | 9103e290ef88…4552 | 898cefee9c38141734548dfead8fed56cbf027306ff0cf0fb547b089dae03058 | 有（721行→769行） |
| experimental/ace/crates/a2x-worker/tests/cases/cert_coll_904.rs | 7fa77ce37e70…e909 | 7fa77ce37e70b3b0f4ff741abcc95b6b3fc35d8ec7add89fe2fcb9bbe5ade909 | 无 |
| experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_cert.rs | 43a9fdfae889…d81d | 293b862df59dde7b28c30c70638b82662e8b7c5de9928451d48150fedc5ed99b | 有 |
| experimental/ace/crates/a2x-worker/tests/cases/a2c_coll.rs（r4新增落点） | — | a2c_coll.rs 41,619B（r4 见§6实施表） | 既有文件，扩写 |

**漂移结论。** F28/F29/F35三个机制在r4源码中**仍然存在且位置已核**（非仅凭r2叙述）：range降糖在 resolve.rs 的 `Stmt::ForIn` Range分支（r4:10826–10961）把隐藏index增量追加在body尾；List分支（r4:11013–11189）同构；`continue`在 aggregates profile 经 `emit_break_or_continue_to_out`（6724/6760）直接跳loop头，非aggregates面（10618–10628）同理；A2X C/Rust的 `hir::Stmt::Loop` 把poll追加在body尾而 `Continue` 只打印 `continue`（a2x-rust r4:511–576、a2x-c r4:777–836）。三个标量静态机制因此按现码重述成立，动态后果由r4反例界定（见§9）。

**r4实测到的额外边界（非F28/F29/F35新增义务）。** `for i in <ident> ..= <ident>`（非字面量端点）在现语法层被具名拒绝（`source.syntax`：`expected {, found ..=`），即动态bound不在现支持域；本包不扩该支持域，LOOP-02改用字面量端点并在测试内保留该拒绝断言（未支持面不计PASS）。`for i in a..b` 无空格形式仍是既有affix边界（既有测试已记录）。


## 5. 详细设计

唯一范围=F28/F29/F35。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：continue执行本循环应有推进与回边poll，break/return不误推进，嵌套loop各有正确目标；i32闭区间MAX最后迭代不合成越域加一。 共享latch思想但保source降糖与C/Rust emitter不同机制；cleanup/survivors按现权威事实，不在backend重推ownership。保持数组/range迭代支持域和正常预算口径。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** source for-in问题沿当前Aggregates，先AV共享MIR，再native及真正支持该构造的A2X入口；A2X Collections poll另用typed HIR或原Collections source，避免未支持range语法挡在前门。C/Rust生成结果各实际编译运行一次，不以生成文本包含poll代执行。

**状态、故障与责任。** 预算Wbase须按当前runtime计费点表独立冻结，不用故障执行结果反推expected；A2X_RT_BUDGET/RT_CANCEL/EVENTS按每个新进程设置，不能靠线程环境竞争。poll失败时LIFO/local survivors清理恰一次；break/return不误poll，F28/F29/F35三种oracle分别关闭，父deadline只兜底不证明guest取消。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F28/F29/F35 → 原LOOP-01/02/03 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：continue推进、闭区间MAX与回边poll是A普通控制流/预算/取消语义，不因图形只AV而延期。关闭谓词：有限continue循环终止且独立值正确，i32 MAX无合成溢出；本层/嵌套break-return正确，实际poll/预算/取消/清理按原计数；已有C/Rust支持腿独立执行一次非UI例，不要求UI三端爬梯。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01a | modify（review F-R1-04 重定向） | docs/specs/auto-ac/source-collections-variants.md | before（C6/C7 §is/for-in）：`for x in <List 局部>` 只写 len 循环前读一次、Copy/owned 元素绑定、结构修改编译拒、break/continue 合法。after：补源级控制流合同——**continue 执行本循环的隐式推进**（range/List 同源；嵌套的非计数 `loop`/`for <cond>` 的 continue 只走内层目标；break/return 不推进）；具名 `for i in a .. b` / `a ..= b` 形，**闭区间末成员即终点**（i32 上界不合成 hi+1），非字面量端点仍为 `source.syntax` 拒绝。 | 源级规则的真实承载文档（F28/F29 属源控制流，不属语法单元目录） | AC-02/03/04 |
| SD-01b | modify（review F-R1-04 新增） | docs/specs/auto-mir/collection-place-profile.md | before（源码 lowering 约束）：`for-in` 降为显式索引循环（len 循环前读一次；LIST_LEN + LIST_GET；无迭代器对象）。after：补"计数循环的回边推进=latch；continue 亦先经latch再测条件；闭区间以端点守卫结束；不新增指令（仍是既有比较/分支/Add）"。 | MIR 降糖形状的承载文档 | AC-02/03/04 |
| SD-02 | modify（保留，正文按实际交付收紧） | docs/specs/auto-a2x/checked-resource-facts.md | before：exit_frees 覆盖 return/break/continue/块贯穿/回边；`loop_survivors`=回边轮询错误路径的存活集；"释放幂等（take 置空/memset）使回边 poll 错误路径重列安全"。after：明确"**每实际回边（含 continue 驱动的回边）消费同一合作 poll**；poll 错误路径=该迭代 scoped 集+survivors；幂等是**前提**——Rust list free 后 cap=0 为真 no-op（不重发释放事件、不二次 drop），与 C `data=NULL` 同规则"。 | 事实/清理权威文档，且本次修复的两个消费者都受此约束 | AC-02/03/04 |
| SD-03a | modify（review F-R1-04 重定向） | docs/specs/auto-a2c/collections-hir-backend.md | before（每出口/回边段）：`for-in 脱糖为 loop{ if(i<n){体;i++} else break }`；回边 poll 错误路径=每迭代 scoped 集+幸存者。after：补"continue 同为本循环推进与同一 poll；**无构造/无分配简单循环每实际回边恰 1 条 `coll:charge:1`**（call 是独立的 4 WU 行）；释放幂等（data=NULL）使重列安全"。 | C 后端的真实承载文档 | AC-02/03/04 |
| SD-03b | modify（review F-R1-04 重定向） | docs/specs/auto-a2r/collections-hir-backend.md | before（Billing / cancellation / fault 冻结表）：`1 WU per loop back-edge poll`；`Cooperative cancellation at polling points only`。after：补"每**实际回边**（含 continue 驱动）＝1 WU＋取消检查；Rust 释放幂等（take 后 cap=0 为真 no-op，事件不重发）"。 | Rust 后端的真实承载文档（WU/取消冻结表所在） | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-source ace-lowering ace-av ace-native ace-a2x ace-a2c-abi。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| LOOP-01 | AC-02 / T-02 | §6完整有限条件continue例：n=n+1，n==4 return i，n<4 continue，range0..3；List对应 | 真实aggregates source→共享MIR→AV/native及已支持A2X | 正确有限结果99；现静态错误路径0可区分；不以timeout作为唯一oracle | 拟实施，未运行 |
| LOOP-02 | AC-02 / T-02 | 单点MAX..=MAX、空/反序/普通端点，含break/return | range降糖及真实compiler/runner | 最后一项完成后正常终止，无额外MAX+1；原合法值/迭代数一致 | 拟实施，未运行 |
| LOOP-03 | AC-02 / T-02 | 条件continue为主的typed/source collections循环；嵌套loop | C/Rust generated code→a2x_coll_poll，原events/WU/cancel接口 | 每实际回边按合同扣WU/查cancel；LIFO与旧值正确；父kill不得代guest poll证据 | 拟实施，未运行 |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

```auto
fn main() int {
    var n = 0
    for i in 0 .. 3 {
        n = n + 1
        if n == 4 { return i }
        if n < 4 { continue }
    }
    return 99
}
```
LOOP-01独立结果99；静态旧路径第4次return 0，选有限反例避免靠挂死。List近邻[10,20,30]同控制，正确仍99。

```auto
fn main() int {
    var n = 0
    for i in 2147483647 ..= 2147483647 { n = n + 1 }
    return n
}
```
LOOP-02期望1；2..=1为空期望0；正常0..3期望3，配break/return不多推进。

LOOP-03独立poll例为普通loop计数n从0加到4，n==4 break，其余条件continue；实际回边数3，正常fallthrough同形控制也3。令A2X_RT_EVENTS=1：Rust应恰有3条coll:poll；C原runtime没有该marker，简单无构造/无分配loop应恰有3条coll:charge:1（另有call的4WU，不能混计）。break那次不poll，内层continue只走内层latch。带owned构造的其他例须按实际charge来源区分，不把所有1WU事件都叫poll。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| LOOP-01 | range/List条件continue；嵌套for/loop；break/return | 推进恰一次；inner continue不推进outer；退出清理/迭代view与原iterate-mutate拒绝保持 |
| LOOP-02 | MAX单点、普通/空/反序；最后一次break/return | 无最后MAX+1；低界负值/动态bound按既有i32与求值顺序 |
| LOOP-03 | C/Rust运行、normal/continue对照、预算边界、cancel、owned survivor | Rust poll marker/C无构造例的1WU charge分别核回边3；独立按现call/alloc charge表算Wbase，Wbase+3可完成，少1预算拒；cancel在应达poll呈Cancelled，free路径不扣/不poll |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| LOOP-01 | audit1036_loop_01 | [experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs](../../experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| LOOP-02 | audit1036_loop_02 | [experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs](../../experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| LOOP-03 | audit1036_loop_03 | [experimental/ace/crates/a2x-worker/tests/cases/cert_coll_904.rs](../../experimental/ace/crates/a2x-worker/tests/cases/cert_coll_904.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

### r4实施落点与实测（work 完成，2026-11-17，提交 d766ce811）

原方案表保留为合同；下表是**实际交付身份**（名字/文件/结果），逐条对应原LOOP行。

| 原Case | 实际测试锚点（已存在且被执行） | 落点文件 | 独立oracle | 修前实测 | 修后实测 |
|---|---|---|---|---|---|
| LOOP-01 | `aggregates_t01_probe::audit1036_loop_01_continue_advances_this_loop_only` | experimental/ace/crates/av/tests/cases/aggregates_t01_probe.rs | range 99 / List 99 / 嵌套24 / break 33 / return 302 / owned-view 4 / try路由 30；ledger闭合且 `iterate-mutate` 仍拒绝 | range 得 **0**（≠99）：`assertion left: 0, right: 99` | 全绿（0.048s） |
| LOOP-02 | `aggregates_t01_probe::audit1036_loop_02_inclusive_endpoint_no_synthetic_overflow` | 同上 | MAX单点1、空/反序0、半开3、闭3、MAX-1..=MAX 2、MIN..=MIN+1 2、末点continue 1、末点break 1；非字面量端点=`source.syntax`拒绝 | `trap: av.trap.int-overflow`（合成MAX+1） | 全绿（0.043s） |
| LOOP-03（Rust腿） | `cert_coll_904::audit1036_loop_03_continue_backedge_polls` | experimental/ace/crates/a2x-worker/tests/cases/cert_coll_904.rs | 3条 `coll:poll`（continue 3次、break不poll）；fallthrough对照2条；Wbase=7（call 4+poll 3）完成/6即`trap:Budget`；cancel首个poll即`trap:Cancelled` | 0 条 poll（`left: 0, right: 3`），预算/取消两闸均不触发 | 全绿（1.60–1.87s） |
| LOOP-03（C腿） | `a2c_coll::audit1036_loop_03_c_continue_backedge_polls` | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll.rs | 无构造/无分配loop：3条 `coll:charge:1`（poll）+1条 `coll:charge:4`（call）且分配账为空；fallthrough对照2条；budget 7完成/6 `trap:Budget`；cancel `trap:Cancelled` | 0 条 `coll:charge:1`（`left: 0, right: 3`） | 全绿（2.80s） |

新增用例均为≤3s（见§9逐例计时），未改任何既有expected/oracle/样本/规模/平台/ignore条件；原`range_for_in_counts_and_executes`等既有例随组全绿。

### r4复审修复落点（2026-11-17，提交 a628ad594，对应 review F-R1-01..04）

| finding | 实际断言/交付 | 修前（red） | 修后（green） |
|---|---|---|---|
| F-R1-01 | `audit1036_loop_01` 追加**嵌套非计数 loop**（纯 `loop`=36）与 **`for <cond>`（while）**（=24）两段；并做受控变异（把latch查找改为“任一外层latch”）复验：裸loop例立即变 24≠36 → 证明该例能区分按target/按任意外层 | 断言不存在（覆盖缺口） | 通过；变异实验 red=`left: 24` |
| F-R1-02 | `audit1036_loop_02` 追加末成员 `return`（单点MAX返回2147483647）与“正常推进到末成员后return”（27） | 断言不存在（覆盖缺口） | 通过 |
| F-R1-03 | **实现修复**：`a2x-rust/src/coll.rs::a2x_coll_list_free` 先 `mem::replace` 取出 Vec、cap==0 直接返回（真幂等 no-op，不二次drop、不重发释放事件）——对齐 `checked-resource-facts` 已冻结的“list free 后 data=NULL/cap=0”规则（C 端本已 guard）。**新用例**：`cert_coll_904::audit1036_loop_03_owned_cleanup`（Rust腿，continue回边+fallthrough两条形 × clean/cancel/budget）与 `a2c_coll::audit1036_loop_03_c_owned_cleanup`（C腿同形） | Rust腿 continue回边poll触发时 **0xC0000374 堆损坏**（`Failure(-1073740940)`，双重释放）；C腿本就平衡 | Rust腿 3.24s 绿（值17、cancel/budget trap且alloc/free字节与次数闭合）；C腿 4.64s 绿 |
| F-R1-04 | §5 规范增量重定向为 SD-01a/01b/02/03a/03b 五条，目标改为真实承载文档（auto-ac source-collections-variants / auto-mir collection-place-profile / auto-a2x checked-resource-facts / auto-a2c collections-hir-backend / auto-a2r collections-hir-backend），frontmatter `supersedes_spec_components` 同步为这5份 | 原目标=语法单元目录与A2X架构ADR，均不承载该规则 | 待T-05按新目标正文沉淀与落地核对 |

本轮A阶段交接：LOOP-01/02/03的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。 — 2026-11-17：r4按现码重冻（§4 r4表），10份入口文件hash实测，§6三案在原入口（AV aggregates链 / A2X Rust coll链 / A2X C coll链）全部可运行；非字面量range端点按现语法具名拒绝，未扩支持域。
- [x] AC-02 / work（review F-R1-01..03 已闭，2026-11-17 a628ad594）：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。 — 2026-11-17：三案修前红/修后绿双向证据见§6 r4表与§9；修复点在 resolve.rs(前三continue路径+latch)与 a2x-rust/a2x-c lower_coll(continue回边poll)，未删/未放宽任何原断言。
- [x] AC-03 / work（review F-R1-04 已闭：SD重定向+frontmatter，2026-11-17）：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。 — 2026-11-17：冻结并集组实测（§9组表）：ace-source 821 / ace-lowering 5 / ace-av 1340 / ace-a2x 306 / ace-native 299 全PASS；ace-a2c-abi 8/9，唯一红为本仓已登记master预存红（runtime pin bd260a8b vs 0a4f68e3，KNOWN-DEBT 4588），非本计划引入。SD-01/02/03 正文待T-05按实际交付收敛。
- [x] AC-04 / review（2026-11-17 final 复审 pass @a628ad594）：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [x] AC-05 / merge（2026-11-17 ff-only 落 master @3b734996b）：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [x] AC-06 / merge（2026-11-17 cleaned：guard clean、树/分支/组目录与 plan-artifacts 删除；登记表因他人损坏 fail-closed 留 pending 条目原文）：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 把两个完整source与独立typed/source poll例启用，冻结continue目标/每条回边/poll次数与Wbase计费，确认各目标准入。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 — 2026-11-17：r4重冻（§4）+三案在原入口跑通；Wbase=7（call 4+poll 3）由clean运行自证，非故障反推。 |
| T-02 / work | [x] T-01决定与可区分输入（F-R1-01/02 已补，见§6 r4复审修复表） | 按本loop latch保推进/终点/poll，嵌套归属和cleanup分开；分别新增三类永久反例，不用单一值绿替代预算绿。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 — 2026-11-17：resolve.rs 增 `for_latches` latch 与 `rblock_can_fall_through`（range闭区间端点守卫=Break；List/半开=增量），三条continue路径（直跳/路由dispatch/collections面）同源注入；a2x-rust/a2x-c 增 `back_edge_poll(lb)` 并在 Loop 体尾与每个 Continue 前注入同一 poll（break不poll）。提交 d766ce811。 |
| T-03 / work | [x] T-02（F-R1-03 已闭：幂等free修复+两腿cleanup用例） | 运行range/List/MAX及C/Rust真实poll/cancel/预算边界/owned清理；保共享MIR与目标原拒绝，完成职责并集。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 — 2026-11-17：冻结并集组5绿1具名预存红（§9组表）；三个新案修前红/修后绿，逐例≤2.8s。 |
| T-04 / review | [x] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [x] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [x] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1 LOOP-01只写伪代码n++，未给可运行源；poll断言没有独立次数、预算边界、break/return及嵌套loop归属，可能值正确但仍绕预算。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

| 路径 | r1字节SHA256 | r2字节SHA256 |
|---|---|---|
| experimental/ace/crates/core/src/source/resolve.rs | 24441ead81f35f8f3928183edc3e371dd478162b7ba6ee531dffb782d7e3d0df | 80eb12467732dd657008918c90bf36158d52d6bbea63bc87e7c511a698263c10 |
| docs/specs/auto-a2x/checked-resource-facts.md | d79eb509e7cb8242297320638601ffbc7c4b5df0626a22911e0adb9acd733704 | 38da0f0d66e92992b408741320616820856aed8111ec73dd3ec3d5f8249e31da |

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原3个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-16 stage:new/revision | PLAN-1036 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F28/F29/F35/原LOOP-01/02/03明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

2026-11-17 stage: work | PLAN-1036 | r4 | outcome: pass | status: execution_done | worktree: D:/autostack/.wt/lang-1036/auto-lang | branch: plan-1036-dev | base_commit: 6de19aa3918813908f9094a93da856983a623c04 | code_commit: d766ce811 | dependency_revisions: none (语言单仓, 无兄弟树) | task_ids: T-01, T-02, T-03 | blockers: none | next: review

**授权与写所有权。** 用户本目标“完成计划1036”授权整链：建树→work→review→有界修复→merge。实现在唯一树，计划与进度写主检出；主检出开工时携带他会话 docs-only WIP（docs/design、docs/specs、他计划文件），按§4保留不动，本包未纳入、未提交、未改动其中任何文件。

**材料 scope。** `D:/autostack/plan-artifacts/plan-1036/`（owner=agent/plan-1036，用途=本计划探针/日志/回滚副本，在途）内含 `groups-t03.log`（首轮六组驱动输出）与三份修前备份 `lower_coll_fixed.rs.bak`/`lower_coll_c_fixed.rs.bak`/`resolve_fixed.rs.bak`（红检时的回滚副本，非运行证据本体）。全部证据以本§摘要为durable形态，merge收尾删除该scope。

**修前红（动态，非静态推断）**：① LOOP-01 `assertion left: 0, right: 99`（range条件continue重复i=0）；② LOOP-02 `source must execute: trap: av.trap.int-overflow`（单点MAX合成2147483648）；③ LOOP-03 Rust腿 `left: 0, right: 3`（`coll:poll`计数），C腿 `left: 0, right: 3`（`coll:charge:1`计数）。④ 路由路径红检：临时关掉dispatch侧latch后 LOOP-01 得 `left: 50, right: 30`，证明该注入是承重的、非装饰。所有红检用`git show HEAD:<path>`回滚单文件、跑完即恢复，最终树与提交一致（`git status` clean）。

**修后绿与冻结门（提交 d766ce811，第二/三轮同形复跑）**：

| 固定组 | 结果 | 执行数 | 组wall(s) |
|---|---|---|---|
| ace-source | PASS | 821 | 5.6 |
| ace-lowering | PASS | 5 | 0.7 |
| ace-av | PASS | 1340 | 56.5 |
| ace-a2x | PASS | 306 | 95.5 |
| ace-native | PASS | 299 | 56.9 |
| ace-a2c-abi | FAIL(8/9) | 9 | 0.6 |

六组整命令 wall=486s（首轮含工具准备=708s；准备/发现/执行/清理由驱动分别计量，工具=case-parent/av/ac/mn/cte/abi/sema/a2x/runtime libs，均由本树构建后按sha绑定）。ace-a2c-abi唯一红=`admitted_text::tests::runtime_admission_preserves_bytes_and_the_certification_pin`，实测 `0a4f68e3…0299` vs 钉 `bd260a8b…e12d`，与 KNOWN-DEBT（4588行）登记的 master 预存红逐字一致（Plan 972 改 runtime.rs 未重钉），其输入文件 runtime.rs 在本树未被本计划修改（git status 无该文件）→ **具名排除、不计本计划红、也不计PASS**。Linux/WSL腿按2026-11-14裁定不在Windows执行。

**新增用例计时（含setup/子进程/清理，均≤10s）**：`audit1036_loop_01` 0.048s、`audit1036_loop_02` 0.043s、`audit1036_loop_03`(Rust腿,真实rustc编译+运行) 1.60–1.87s、`audit1036_loop_03_c`(C腿,真实MSVC cl编译+运行) 2.80s。测试行为/断言见§6 r4表；未新增任何超过10s的用例，未调用ACE性能/campaign。

**健康检查**：`cargo build -p ace-core / ace-a2x-rust / ace-a2x-c` 零新增warning；6个改动文件用 `rustfmt --edition 2021` 单文件格式化（整crate fmt会改到无关既有漂移 a2x-rust/coll.rs、print.rs、plan981_*，故未整crate执行）；无调试打印残留；无新增ignore/platform条件；未改任何既有expected/样本/规模。

**遗漏/延后（具名，交review裁定是否需另立计划）**：① `a2x-python/src/lower.rs:620-635` 与 `a2x-ts/src/lower/mod.rs:427-436` 的Collections emitter与C/Rust**同形**（poll在loop体尾、`continue`直接跳头），即F35同类缺口在两个非C/Rust后端仍在。本计划§5把F35范围限定“C/Rust collections生成”，§2禁止“因共用文件扩大范围”，故本包**不修**且**不声称全后端闭合**；已登记为债候选，owner=其后端计划，关闭条件=同源的continue回边poll/per回边WU计数例在各自真实链上红→绿。② `emit_post_frame_dispatch` 的Try分支（resolve.rs:6974）把越界break/continue以`Break{try_loop}`外推，本计划只在Loop|ForIter分支注入latch；try内continue到外层计数for-in的组合由§6 try例覆盖（值30），非try嵌套外推的组合仍属未观测面（无现用例）。③ 非字面量range端点仍是语法拒绝（未扩支持域），若产品要支持属新支持决策，不属本包。

2026-11-17 stage: goal | plan_id: PLAN-1036 | plan_revision: r4 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1036/auto-lang | commit: d766ce811 | next: review
2026-11-17 stage: review | review_scope: final | reviewer_identity: 实施会话内复审（本计划未指定独立reviewer/model，§6“review独立核证（无指定模型/另起agent授权）”）——独立性限于“重放冻结提交+重读代码/合同”，不采信work叙述；limitations=同一上下文，未另起agent | plan_id: PLAN-1036 | plan_revision: 4 | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1036/auto-lang | branch: plan-1036-dev | reviewed_commit: d766ce811 | base_commit: 6de19aa3918813908f9094a93da856983a623c04 | dependency_revisions: none | spec_inputs: auto-syntax/composite-source-projection.md(529bc7ec…), auto-a2x/checked-resource-facts.md(38da0f0d…), auto-a2x/architecture.md(9984e885…), plus 复核增量命中 auto-ac/source-collections-variants.md、auto-mir/collection-place-profile.md、auto-a2c/collections-hir-backend.md、auto-a2r/collections-hir-backend.md | next: work（有界修复 F-R1-01..04）

**基线核对。** 实现已提交且树干净（`git status` 空）；diff=6文件 +680/−119（两个lowerer+resolver+3个测试文件）。主检出在复审时已前移30个提交（6de19aa39→a0e9066c9，含1029/981等他人代码+测试落main），**本计划实现未漂移**；`git merge-tree --write-tree master plan-1036-dev` 返回单树、exit 0（无冲突），因此merge需在合并基线上重跑受影响并集（已列 pending_closeout_gates）。

**复审自身重放（本会话，冻结提交 d766ce811，非work原文）**：`cargo nextest -p ace-av --test conformance -E 'test(audit1036)|test(range_for_in_counts_and_executes)|test(for_in_nested_list_view_binding_reads)|test(chain_health_and_recovery)'` = 5/5 PASS（0.033–0.051s）；`cargo nextest -p ace-a2x-worker --test conformance -E 'test(audit1036)|test(a2c_coll_loop_poll_cancel)|test(a2c_coll_exit_cleanup_matrix)|test(cert_coll_two_leg_pass_and_wrong_oracle_rejected)'` = 5/5 PASS（2.180–3.392s）。新增用例实测：loop_01 0.051s、loop_02 0.045s、loop_03(Rust) 2.180s、loop_03_c(C) 3.392s——**均≤10s**；无helper/子进程隐式构建被测产品（`rustc`/`cl` 编译的是被测目标的生成程序，属被测行为本身，与同组既有例同形）；健康检查：三crate `cargo build` 零warning、diff 无 dbg!/println!/eprintln! 残留、6文件已单文件 rustfmt。

**AC 结果表**：AC-01 pass（r4重冻+入口可运行，三案红/绿可复现）；AC-02 **partial**（F-R1-01..03：§6声明状态中有3项无断言）；AC-03 **partial**（F-R1-04：Spec准备的目标文档错位，规范增量未落到承载该规则的文档）；AC-04 本行；AC-05/06 未执行（merge）。

**发现（4项，均in-contract，均为“已声明义务未落实/未定向”，不是外围改进）**

- **F-R1-01（AC-02；T-02；P2，覆盖缺口）** §6 r2 LOOP-01 行声明“range/List条件continue；**嵌套for/loop**；break/return”，但 `aggregates_t01_probe.rs:785` 只覆盖“嵌套**for-in**”。缺：计数for-in 内嵌**裸 `loop`（或 while）**、其 `continue` 只推进内层（外层计数latch**不得**执行）。违反的既有规则=§6该行状态断言 + 本计划改动机制（`for_latch_of` 按target查找必须对非计数loop不命中）。关闭oracle：内层continue计数=预期值且外层迭代数不变、ledger闭合；最小重放=该test内追加一段source。
- **F-R1-02（AC-02；T-02；P2，覆盖缺口）** §6 r2 LOOP-02 行声明“MAX单点、普通/空/反序；**最后一次break/return**”，现 `:868` 只断言最后一次 `break`。缺：`for i in 2147483647 ..= 2147483647 { … return n }`（末成员 `return` 绕过latch、不得合成MAX+1）。关闭oracle：返回值为该成员对应值、无IntOverflow。
- **F-R1-03（AC-02；T-03；P1，新增发射点的错误路径未验）** §6 r2 LOOP-03 行声明“C/Rust运行、normal/continue对照、预算边界、cancel、**owned survivor**”，而两个新poll例故意使用**无owned局部**的loop（为C端`coll:charge:1`计数纯净）。修复在 `continue` 处**新增了poll发射点**，其错误路径free清单=`exit_frees(lb,None)+loop_survivors(lb)`，但该路径无任何断言；既有 `a2c_coll_exit_cleanup_matrix` 的cancel命中**fall-through** poll，不能替代。关闭oracle：含owned每迭代局部+owned循环幸存者的loop、只用continue构成回边，在`A2X_RT_CANCEL=1`/`A2X_RT_BUDGET=<Wbase>`下呈`trap:Cancelled`/`trap:Budget`且 `coll:alloc`与`coll:free-backing`**字节与次数闭合**、值路径逐次恰一次释放（C腿+Rust腿各一）。
- **F-R1-04（AC-03；AC-05；P1，规范增量定向错误）** §5 `规范增量` 的 SD-01/SD-03 目标文档与规则无对应：`auto-syntax/composite-source-projection.md` 只讲语法单元目录/身份/pin（无 for-in/continue/range 规则）；`auto-a2x/architecture.md` 只讲数据流/ADR/依赖方向（无 poll/WU 规则）。该两份为错误落点，merge 会把控制流/计费规则沉淀到无关文档，而**真正承载该规则的canonical文档仍是旧文本**：`auto-a2c/collections-hir-backend.md:33-34`（“for-in 脱糖为 `loop{ if(i<n){体;i++} else break }`；回边 poll 错误路径=每迭代 scoped 集+幸存者”——continue 回边现已同样推进并同样poll，闭区间端点变体未含）、`auto-a2r/collections-hir-backend.md:27`（“1 WU per loop back-edge poll”未说continue回边）、`auto-ac/source-collections-variants.md` C6/C7 §for-in（未说continue推进/闭区间终点/range形）、`auto-mir/collection-place-profile.md:30`（“for-in 降为显式索引循环”未含latch/端点规则）；SD-02 目标 `auto-a2x/checked-resource-facts.md` 方向正确但正文须写明“每实际回边（含continue）消费同一poll”。关闭oracle：SD-01..03 的目标路径逐一命中上述真实承载文档（或给出“该文档不承载→不列”的理由），before/after 规则与代码行为逐字一致；`auto-a2c` L33 的脱糖形状更新为含continue推进与闭区间端点两种形态。

**未发现**：实现无行为性缺陷证据（三案红/绿+replay一致，merge-tree无冲突）；无范围削减、无绕过（Python/TS同形缺口已按F35原范围具名登记，非本包in-contract项，但merge须把它写入债务登记而非当PASS）。

**pending_closeout_gates（merge所有）**：① 主检出自base前移30提交，合入后须在合并基线重跑受影响并集 `cargo t ace-source ace-lowering ace-av ace-a2x ace-native ace-a2c-abi`（同身份证据可复用者仅限未变输入）；② `ace-a2c-abi` 的 `admitted_text::…certification_pin` 既有红按KNOWN-DEBT(4588)具名排除，不充PASS；③ SD目标修正后按§5正文沉淀，并在归档前核对落地文本；④ Python/TS同形缺口入KNOWN-DEBT具名持有。

2026-11-17 stage: goal | plan_id: PLAN-1036 | plan_revision: r4 | round: 2 | from: execution_done | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1036/auto-lang | commit: d766ce811 | next: work
2026-11-17 stage: work | PLAN-1036 | r4 | outcome: pass（复审修复轮） | status: execution_done | worktree: D:/autostack/.wt/lang-1036/auto-lang | branch: plan-1036-dev | base_commit: 6de19aa3918813908f9094a93da856983a623c04 | code_commit: a628ad594 | dependency_revisions: none | task_ids: T-02, T-03 | blockers: none | next: review

**修复对象**：review F-R1-01..04（全部 in-contract）。逐条闭合见§6「r4复审修复落点」表；本条只记身份、门与计时。

**实现改动（1处生产代码）**：`experimental/ace/crates/a2x-rust/src/coll.rs::a2x_coll_list_free` 改为真幂等（`mem::replace` 取出 Vec；cap==0 直接返回，不二次 drop、不重发释放事件）。依据=已冻结规则 `checked-resource-facts` “释放幂等（take置空/memset）… list free 后 data=NULL/cap=0”（C 端本就 guard）。这是 F-R1-03 oracle 抓出的**真实缺陷**：修复前，continue 回边（或 fallthrough 回边）的 poll 一旦 trap，错误路径重列的每迭代局部被二次释放 → 生成程序 `0xC0000374`（STATUS_HEAP_CORRUPTION，实测 `Failure(-1073740940)`）。修复按“机制不是样例”落地在运行时释放原语，覆盖两条回边形与所有 owned 元素（Str/deep 本就 take 幂等）。

**测试改动（3文件，全为既有锚点内追加/新锚点）**：`aggregates_t01_probe.rs`（F-R1-01 嵌套非计数 loop/while、F-R1-02 末成员 return）、`cert_coll_904.rs`（新 `audit1036_loop_03_owned_cleanup`，Rust腿两形×clean/cancel/budget）、`a2c_coll.rs`（新 `audit1036_loop_03_c_owned_cleanup`，C腿同形）。未改任何既有expected/oracle/样本/规模/条件。

**本会话执行与计时（含setup/编译/运行/清理，均≤10s）**：`audit1036_loop_01` 0.05s、`audit1036_loop_02` 0.11s、`audit1036_loop_03`(Rust) 1.84s、`audit1036_loop_03_c`(C) 3.21s、`audit1036_loop_03_owned_cleanup`(Rust) 3.24s、`audit1036_loop_03_c_owned_cleanup`(C) 4.64s。诊断期一次红线：期望值写错导致 46s（无限loop烧预算），已改正为有限形并复测，最终无>10s用例。

**受影响固定组（修复提交 a628ad594 上，`cargo t ace-av ace-a2x`）**：ace-av 1340 executed / 0 failed / 45.4s；ace-a2x 308 executed / 0 failed / 78.4s；整命令 wall=400s，gate_passed=true。本轮未重跑 ace-source/ace-lowering/ace-native/ace-a2c-abi（本修复未触其输入：resolve/lowering/C-runtime 与 ABI 文本未变；上轮 d766ce811 上它们的 PASS 证据在同身份输入下仍有效），其中 ace-a2c-abi 的既有 pin 红仍按 KNOWN-DEBT(4588) 具名排除。

**受控变异（证明新断言可区分错误实现）**：把 `for_latch_of` 临时改成“任一外层latch”（忽略target）→ `audit1036_loop_01` 的裸loop例给 `left: 24`(≠36) 立即转红；恢复后转绿。证据留于本§，变异未入库。

**未闭/交下一阶段**：T-04 re-review 只核 F-R1-01..04 与受影响AC；T-05 merge 须①按§5新目标正文沉淀SD-01a/01b/02/03a/03b并核对落地；②在合并基线（main 已前移30提交）重跑受影响并集；③把 Python/TS 同形缺口写入 KNOWN-DEBT 具名持有。

2026-11-17 stage: goal | plan_id: PLAN-1036 | plan_revision: r4 | round: 3 | from: executing | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1036/auto-lang | commit: a628ad594 | next: review
2026-11-17 stage: review | review_scope: final（复审，范围限定 F-R1-01..04 + AC-02/03） | reviewer_identity: 实施会话内复审（无独立reviewer/model；独立性=重放冻结提交+重读代码/规范原文，不采信work叙述；limitations=同一上下文） | plan_id: PLAN-1036 | plan_revision: 4 | outcome: pass | worktree: D:/autostack/.wt/lang-1036/auto-lang | branch: plan-1036-dev | reviewed_commit: a628ad594（修复轮增量 diff vs d766ce811=4文件 +250/−6） | base_commit: 6de19aa3918813908f9094a93da856983a623c04 | dependency_revisions: none | spec_inputs: 5份承载文档（auto-ac/source-collections-variants L45/47、auto-mir/collection-place-profile L30、auto-a2x/checked-resource-facts L47/66/68/121、auto-a2c/collections-hir-backend L33/34/36、auto-a2r/collections-hir-backend L27）逐一核在，SD-01a/01b/02/03a/03b 的 before 片段全部命中原文 | next: merge

**逐发现闭合（check → invariant → 结果）**

| finding | 不变量 | 复审实测 |
|---|---|---|
| F-R1-01 | 计数for-in的latch按**target**命中；非计数 loop/while 的 continue 不得推进外层 | 代码：`for_latch_of` 以 `*t == target` 精确匹配（resolve.rs:6569）；用例：裸loop=36、while=24 常驻；受控变异（改为“任一外层”）时该例转红 `left: 24`——接受该变异证据（本会话执行，未入库），并以当前代码复核匹配条件 |
| F-R1-02 | `return` 绕过latch；i32 上界不合成 MAX+1 | 用例：单点MAX `return i`=2147483647；正常推进到末成员后return=27；`rblock_can_fall_through` 对体末终结语句不追加latch（代码复核） |
| F-R1-03 | 回边poll错误路径重列 = 释放必须真幂等（不二次drop/不重发事件） | 实现：`mem::replace` 取出 + `cap==0` 早退（coll.rs:786-800）；**结构性复核**：Rust 全部释放原语（str take / list 新take / deep take / 生成的record·variant free 逐字段委派 / i32·bool no-op）均幂等→重列不可能二次释放；红→绿：修复前 `Failure(-1073740940)`（0xC0000374），修复后 continue/fallthrough × clean/cancel/budget 六路实测通过且 alloc/free 字节与次数闭合（本轮复跑 Rust 3.78s、C 5.73s） |
| F-R1-04 | delta 必须落在真实承载文档 | 5份目标存在、before 片段逐条命中（见上 spec_inputs）；SD-02 保留并收紧；frontmatter `supersedes_spec_components` 与表格5行一致 |

**AC 结果**：AC-01 pass（前轮）；AC-02 **pass**（F-R1-01..03 已闭，§6 各行子状态均有断言）；AC-03 **pass**（消费者/原测试/门保持：修复提交上 `cargo t ace-av ace-a2x` = 1340/1340 与 308/308，gate_passed=true，wall 400s；SD准备已按真实承载重定向）；AC-04 本行 pass；AC-05/06 未执行（merge）。

**覆盖边界（非阻断，明示）**：`free路径不扣/不poll` 由**结构+观察**关闭而非专门断言——唯一释放路径（`a2x_str_free`/`a2x_coll_list_free`/`a2x_coll_deep_free`/生成的 free 助手）内不含任何 charge/poll 调用（代码核对），且 cancel/budget 的 stderr 事件序在 trap poll 之后仅有 `coll:free-backing` 行。若后续要求每条冻结规则都有常驻断言，可作增量；不影响本计划关闭谓词。

**本会话复跑与计时（a628ad594）**：AV `audit1036_loop_01` 0.116s、`_loop_02` 0.118s（+ 既有 `range_for_in_counts_and_executes` 0.107s、`for_in_nested_list_view_binding_reads` 0.103s）；A2X `loop_03`(Rust) 2.14s、`loop_03_c`(C) 4.00s、`loop_03_owned_cleanup`(Rust) 3.78s、`loop_03_c_owned_cleanup`(C) 5.73s（+ 既有 `a2c_coll_loop_poll_cancel` 2.89s、`a2c_coll_exit_cleanup_matrix` 3.39s）。全部 ≤10s；无新增helper隐式构建（编译对象=被测生成程序本身）；健康：4文件 rustfmt --check 干净、`cargo build -p ace-a2x-rust` 零warning、diff 无调试打印。

**pending_closeout_gates（merge所有，同前轮）**：① 主检出仍领先 base 30+ 提交（含他人代码/测试落 main），merge 后须在合并基线重跑受影响并集 `cargo t ace-source ace-lowering ace-av ace-a2x ace-native ace-a2c-abi`；② `ace-a2c-abi::admitted_text…certification_pin` 既有红按 KNOWN-DEBT(4588) 具名排除；③ 按 §5 新目标把 SD-01a/01b/02/03a/03b 正文沉淀并核对落地文本；④ Python/TS 同形缺口入 KNOWN-DEBT 具名持有；⑤ 归档前 guard clean + 清树/分支/scope。
2026-11-17 stage: goal | plan_id: PLAN-1036 | plan_revision: r4 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1036/auto-lang | commit: a628ad594 | next: merge
2026-11-17 stage: merge | PLAN-1036:r4 | outcome: pass | checkpoint: prepared+landed+canonical_verified+archived（cleaned 待执行） | delivery_commit: 3b734996b | reviewed_commit(before rebase): a628ad594 → rebased 68924c748 | base_commit: 6de19aa39 | canonical: 5份（auto-ac/source-collections-variants sha256 cc52d3ae…、auto-mir/collection-place-profile f37ff295…、auto-a2x/checked-resource-facts ae5bb206…、auto-a2c/collections-hir-backend 6f0f22a9…、auto-a2r/collections-hir-backend 7d18ff1f…） | archive: docs/plans/archive/1036-ace-loop-latch-and-cooperative-poll.md

**准备**：review pass 绑 r4+reviewed_commit a628ad594+5份已审 delta（SD-01a/01b/02/03a/03b），T-01..T-04 全勾、AC-01..04 全勾。主检出已领先 base 33 提交（含1029/981等他人代码与测试）。

**重放/落标**：worktree 内 `git rebase master` → 2 个提交重写（d766ce811→b9713e10f、a628ad594→68924c748）；`git range-diff 6de19aa39..a628ad594 9b73b70d4..68924c748` 输出两条 `=`（补丁等价证明）。随后在**重放基线**上跑合并自有并集门：`cargo t ace-source ace-lowering ace-av ace-a2x ace-native ace-a2c-abi` → ace-source 821 PASS/8.4s、ace-lowering 5 PASS/0.7s、ace-av 1341 PASS/46.5s、ace-a2x 308 PASS/78.7s、ace-native 299 PASS/43.5s、ace-a2c-abi 8/9（唯一红=`admitted_text::…certification_pin`，`0a4f68e3…` vs 钉 `bd260a8b…`，**与 KNOWN-DEBT(4588) 登记的 master 预存红逐字一致**，非本计划引入，具名排除不计PASS）；整命令 wall=519s、gate_passed=false（仅该具名预存红）。规范增量提交前复核：本增量**纯文档**，代码/依赖/配置与已审提交一致 → delivery_commit=3b734996b。

**落地与规范核对**：`git merge --ff-only plan-1036-dev` → master tip=3b734996b（线性、无merge提交）。5份 canonical 文档在 master 上逐一核到本计划规则文本（PLAN-1036 标记计数 1/1/2/1/1）并记录 sha256（见上）。未写任何 ledger/specs.json、未发布索引。

**归档**：`git mv` 至 `docs/plans/archive/`，frontmatter `status: archived`；本次提交即归档簿记（另见 commit）。AC-05/T-05 勾选；AC-06/T-06 留待清理完成。

**未闭（T-06）**：worktree/branch/组目录移除、`plan-artifacts/plan-1036` 摘要折叠后删除、registry 刷新、landing 制品观察、批量回归 due 检查。
2026-11-17 stage: merge | PLAN-1036:r4 | outcome: pass | completion_kind: delivered | checkpoint: cleaned | delivery_commit: 3b734996b | worktree: D:/autostack/.wt/lang-1036/auto-lang（已移除） | branch: plan-1036-dev（已删，was 3b734996b） | archive: docs/plans/archive/1036-ace-loop-latch-and-cooperative-poll.md

**清理（逐项实测）**：① `wt-guard.ps1` 移除前 fresh 运行=clean（无 reparse point）；② `git worktree remove` 成功、`git worktree list` 已无 lang-1036；③ `git branch -d plan-1036-dev` 成功（was 3b734996b=交付提交，证明已落主）；④ 组目录 `D:/autostack/.wt/lang-1036` 空后删除（`Test-Path`=False）；⑤ `plan-artifacts/plan-1036` 整 scope 删除：6文件 1,458,039B（`coll_rs_fixed.bak` 29,607B sha256 b6cb088c…、`groups-t03.log` 2,597B d678cf2c…、`lower_coll_c_fixed.rs.bak` 75,578B 7f8c6e57…、`lower_coll_fixed.rs.bak` 80,843B 829a7bed…、`resolve_fixed.rs.bak` 634,732B ad30b52b…、`resolve_mut_base.bak` 634,682B b3f9f3d8…），内容已折叠为本计划§6/§9摘要；⑥ 无具名财产持有（named holds: none）。

**登记表刷新：pending（他人数据损坏，fail-closed 未改写）**。`D:/autostack/plan-artifacts/.retention/policy.json` 当前**不能解析**：`plans[34].holds[1]`（~L314/315）之后缺 `]`/`}` 直接接入下一对象，尾部同类断裂再次出现；mtime 2026-11-17 01:08（他人会话合并 1029/981 期间）。按"不覆盖他人记录"原则本包未改写该文件；本计划应插入的条目原文如下，owner=下次 merge/批量回归 triage，关闭条件=该 JSON 可解析且此条目插入：

```json
{"plan":"auto-lang/PLAN-1036","status":"deleted-at-merge (no holds)","deleted_at":"2026-11-17","completion":"delivered r4 @3b734996b (master ff-only via plan-1036-dev; reviewed a628ad594 rebase→68924c748, range-diff two '='). AUDIT-1024 F28/F29/F35: for-in continue latch (resolve for_latches on the direct/dispatch/collections continue paths), inclusive-range endpoint guard (no synthesized hi+1), per-actual-back-edge poll in C/Rust collections incl. continue; surfaced + fixed non-idempotent Rust list release (0xC0000374 heap corruption on the poll error path -> take + cap==0 no-op). 6 permanent cases <=5.8s; 5 canonical specs (SD-01a/01b/02/03a/03b). Union on rebased baseline: ace-source 821, ace-lowering 5, ace-av 1341, ace-a2x 308, ace-native 299 PASS; ace-a2c-abi 8/9 with the named pre-existing runtime-pin red (KNOWN-DEBT 4588).","scope":"plan-artifacts/plan-1036 (deleted whole; 6 files 1,458,039B; folded into docs/plans/archive/1036-ace-loop-latch-and-cooperative-poll.md)","named_leftovers":"none held; two non-blocking boundaries recorded in the archived plan (a2x-python/a2x-ts same-shape continue poll gap = D-1036-PYTS-POLL in KNOWN-DEBT; 'cleanup never bill/poll' closed by inspection of the sole release path)"}
```

**landing / 制品观察（不部署）**：本计划只改 `experimental/ace`（resolve/lowering/a2x C+Rust/runtime/测试）与 docs/specs；**未触 `crates/**`**，故 PATH 上的 `auto` CLI 与 web 产物均不涉及，无需重建/重启。ACE worker 的 `experimental/ace/target/release/*.exe` 是测试工具、按组需要现建（本轮并集门在 worktree 内自建并按 sha 绑定），随 worktree 一并消失；主检出的同类副本是更早构建的测试工具，不构成生产制品陈旧问题。无 release/打包/守护进程动作被触发。

**批量回归 due 检查（auto-lang）**：`docs/plans/.last-batch-regression.json` 存在，`last_covered_plan_id=1064`、`timestamp=2026-11-16T10:04:09Z`。本计划 1036 `%5=1`（非 0），receipt 年龄 <48h，窗口内无 `%5==0` 的计划在此 receipt 之后归档 → **当前 not due**；owner=周期批量回归（下次 %5 计划归档或 >48h 有合并时再触发）。

**最终计数**：6/6 任务、AC-01..06 全部勾选（AC-05/06 为本条落地）；`git status` clean（仅他人 docs WIP 保留、未纳入）。
2026-11-17 stage: goal | plan_id: PLAN-1036 | plan_revision: r4 | round: 5 | from: merge | to: done | status: archived | outcome: pass | worktree: removed | commit: 3b734996b | next: done

**GOAL 完成收据（PLAN-1036 交付闭环）**：plan=PLAN-1036 r4；delivery_commit=`3b734996b`（master ff-only，线性无 merge 提交；reviewed `a628ad594` → rebase `68924c748`，`git range-diff` 两条 `=`）；canonical Specs 5 份已落并逐份核文本+sha256（auto-ac/source-collections-variants cc52d3ae…、auto-mir/collection-place-profile f37ff295…、auto-a2x/checked-resource-facts ae5bb206…、auto-a2c/collections-hir-backend 6f0f22a9…、auto-a2r/collections-hir-backend 7d18ff1f…）；archive=`docs/plans/archive/1036-ace-loop-latch-and-cooperative-poll.md`（status: archived，docs 已提交）；cleanup=guard clean + 树/分支/组目录/plan-artifacts scope 全删（6文件 1,458,039B 折叠后删）；剩余 merge 自有项=登记表条目待他人修复后插入（其 JSON 已被他人写坏，条目原文留本§9）、批量回归 not due（owner=周期 gate）。产品面无生产制品重建需求（本计划只触 experimental/ace 与 docs）。

**交付实质（对用户可见）**：AUDIT-1024 的 F28（for-in `continue` 跳过隐藏推进 → 重复当前元素）、F29（闭区间 i32 上界合成 `hi+1` → IntOverflow）、F35（A2X C/Rust collections `continue` 绕过回边 poll → 预算/取消点丢失）三项全部修复并有红→绿证据；修复过程中发现并修掉一个**真实内存安全缺陷**（Rust list 释放非幂等 → 回边 poll 错误路径二次释放 → 0xC0000374 堆损坏）。6 个永久回归用例常驻，全部 ≤5.8s。
## 10. 待澄清事项

F28值/迭代、F29宽度、F35预算/取消三个oracle分别关闭；不把所有目标未支持面计为PASS，不新增无限规模承诺。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。

2026-11-17 r4 work后更新：**当前无待用户澄清项**（未改目标/AC/范围/仓集合/预算）。两项具名未观测面已记§9 outlier：① a2x-python/a2x-ts 的Collections continue回边poll（F35同形、非本包C/Rust范围，属债候选/其后端计划owner）；② try外推与计数for-in组合的非覆盖情形（现无用例）。二者均**不计PASS也不计本计划红**，由T-04复审裁定是否需要另立计划或本包有界补测。

2026-11-17 r4 review后（needs_fix）：**无需用户裁定的问题**（F-R1-01..04 均为本计划已声明义务的落实/定向修复，不改目标/AC口径/范围/仓集合/预算）。下一轮 work 只做有界修复：①在既有锚点test内补3段断言（嵌套裸loop、末成员return、continue回边poll的owned survivor清理）；②按实际承载文档重定向 SD-01/SD-03 正文与 `supersedes_spec_components`；③重定向后重跑受影响用例，不重跑整组；随后re-review只核这四项与受影响AC。

2026-11-17 r4 复审修复后：**无需用户裁定**（F-R1-01..04 均为已声明义务的落实，目标/范围/预算不变）。下一轮 re-review 只核这四项与受影响AC-02/03；修复未改语义合同，故 `plan_revision` 仍为 r4（review 的 r5 升版规则仅在合同变更时触发）。

2026-11-17 r4 复审 pass（@a628ad594）：F-R1-01..04 全部闭合，AC-01..04 已勾，status=`reviewed`，4/6。merge（T-05/T-06）承接：合并基线上重跑受影响并集、按§5新目标沉淀SD-01a/01b/02/03a/03b、Python/TS 缺口入债务登记、归档/清理与输出路径对账。无需用户裁定。
