---
plan_id: PLAN-1024
status: archived
feature_name: ACE M2 已实现代码质量、架构与复用审计
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-17
plan_revision: 2
current_step: 5
total_steps: 5
depends_on: []
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [ACE, auto-test-case]
---

# [PLAN-1024] ACE M2 已实现代码质量、架构与复用审计

## 0. 变更摘要

2026-11-17当前路线簿记：1027–1038已有9包交付归档，仅1030/1037/1038未开工；当前建议1037→1030→1038，1038只读研究可提前，1024 T-05独立收尾。逐包revision/正式提交/旧进度例外见§5最新路线表；本轮不修改审计结论、原任务/发现分母或实现，不签985/A。

2026-11-17 T-05合入归档（/goal 完成计划1024，auto-plan-merge）：r2审计文档落主 f046f4ae4，T-05/AC-05关闭（5/5），SD-00无规范增量（r2 final review认可），内链按归档位置重写并全量解析通过；无实施树/无plan-artifacts scope可清。本归档交付的是审计来源与归属，不批准任何后继重构代码，F01–35仍由各承接Plan关闭。

2026-11-12 r2：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

归属 **M2-A工程质量支撑**，允许在M1最后测试期间提前做只读分析。本包交付代码审计与可执行重构建议，不直接重构编译器，不接管M1测试/签章。2026-11-12已完成规划所需的首轮结构扫描与热点抽查，结果见§4/5；不是逐文件、逐符号审计完成声明，后续深审仍按§8执行。

2026-11-12第二轮：用户授权逐项深入审核F01–15，先正确性、再架构/复用，明确暂不改实现。已完成15项的定向静态复核，当前结论见§5“第二轮逐项审核”，方法与实际结果见§6/7。本轮不是整个workspace逐符号审计或独立终审。

2026-11-12第三轮：按Cargo全部35成员建立MC01–35独立记录，补语言父侧MC36，逐项补核入口/消费者、状态与拒绝、复用及测试归属；新增F16–20。已完成成员清单与定向静态补审，仍保留每成员未核分支、实际执行证据与完整语义/测试义务闭包，不能称全模块深审完成。

2026-11-12第四轮：按“完整分支深审”要求，将COV-A..E展开为57个分支族，新增F21–27，逐项写入证据、结论、建议和反例入口；覆盖恢复解码、提交依赖、会话事务、CT观察、父侧等待及清理。所有35成员及MC36均有归属，内部子分支/测试义务的未闭合项仍明确保留。只读静态审核，不把分支族清单当穷尽所有内部条件或运行覆盖。期间源码主线推进到7833ee65、Plan被另一会话细化为r2，已分别核对并保留增补。

2026-11-13第五轮：继续补B13源码构造、A05分页读取、C09 CT遍历及E07测试义务。逐一对账25个AST表达式/14个语句的分派及拒绝职责，新增F28–33：for-in continue推进、闭区间极值、名不副实的失败用例、分页目录溢出、CT命名实参遗漏、owned元素投影命名实参panic。补全部35成员测试入口/职责组、20个包的显式test target路径及4个平铺测试根引用核对；仍不冒实际测试发现/执行或全部case/check语义覆盖。当前来源42874419，生产源码与前轮新增证据相关文件无差额；只更新本Plan。

2026-11-13第六轮（当前）：完成HIR全部17类型、42表达式、9语句的资源/授信分区，以及181份case文件、71份suite、770个case、1116个check的逐项保留映射；35种注册recipe、原511检查/668固定输入、legacy313个cell和48架构recipe均已对账。新增F34资源record分类差额、F35 A2X continue绕回边poll。**只读深审工作闭合，AC-01..03、T-01..03完成；status=execution_done，current_step=3/5。** 当前源码固定81d8fc03，结论和后继切片见§5，实际自检见§6；没有执行ACE或修改实现。AC-04独立复审、AC-05合入归档仍待进行，35项发现的真实反例、修复和运行验证分别交后继有界实施，不能把审计闭合解释成风险消除或所有分支已测。


2026-11-13第七轮（当前）：用户要求先复审再分配。对r2提交文档SHA256=fa81b05a9c63c22b641eec91c21e58e9c3c7a8adb8ebd66710835f4665977bbf重新核源码/Spec/全部映射并做8个对账负控；只读审计交付final review=pass，同一执行上下文的限制见§9。F01–35逐项分给已有4份Plan与新1027–1038，优先级/任务/case/1002依赖入§5；风险不冒修复。AC-01..04/T-01..04完成，status=reviewed、4/5；T-05合入归档未执行，未修改实现/运行ACE。

发现区分：已观察的实现事实、有静态反例的正确性风险、待实测的架构/性能假设。文件长、unsafe多、profile多不能单独判定缺陷。M2已落地增量与M1遗留分别标识。

## 1. 目标

1. 覆盖ACE全部workspace成员及语言侧测试接缝，核查职责/依赖、重复语义、复用、提交/恢复、资源生命周期、诊断和测试可维护性。
2. 每项建议给出生产者/消费者、代码位置、风险、迁移边界、保留义务、验证和关闭方法。
3. 配合1002文档式测试试点、985覆盖更新、1017成本治理，给出实施顺序与归属，不另造执行器。

非目标：扩大语言集、改M1出口、执行945–948前探、运行正式性能/压力套件、改外仓代码、默认切换/退役/发布。代码修复和重构由后续有界实施合同承接；跨核心合同的大改先形成设计，不能把本审计包变成无界重写。

## 2. 架构方案

按真实链路：CST/AST→名称与类型事实→Checked HIR→共享Typed MIR→AV/AC/A2X；并行检查artifact/query/CTE、runtime、worker/父侧裁判、测试配置。

采用全量清点、分层人工追踪、反例区分、分批建议。清点不等于理解；最终覆盖表必须明确已核入口/消费者、未读面及理由。重复检测只作导航，逐项判断重复机制、独立oracle或有意冻结的兼容面。

## 3. 技术栈

本轮只读Rust/Cargo/TOML/Specs/计划，无需构建。仅改本Plan和路线导航，在主检出作文档簿记。若后续需要可执行探针/测试，固定实施树为 `D:/autostack/.wt/lang-1024/auto-lang`，分支 `plan-1024-dev`，先核实际基面；不得在master写探针/构建，不创建链接。

本轮无独立材料文件或scope。后续若产原始材料，唯一目录为 `D:/autostack/plan-artifacts/plan-1024/`，创建时登记owner、用途、路径、在途状态；结论折入Plan。收尾核所有输出路径并清理，不新建永久reports附件。

## 4. 需求分析与背景调查

### 授权与来源

用户2026-11-12要求自行建M2计划，允许M1最终测试期间提前分析已实现代码，关注不合理实现/架构、冗余和复用。授权覆盖提前只读调查，不推定实现/合入授权，未指定预算；只读工作不需重复许可。

主检出大量其他会话文档WIP保持原样。取号前`.next-id=1024`；`scripts/new-plan.sh ace-m2-code-quality-and-refactoring-audit`创建1024、推进1025。采用本仓脚本和已有四位Plan编号约定，不截断编号。

扫描开始HEAD=`7b2f2e95967cb3434429093988fae429c05e865f`；过程中其他会话推进主线，二次清点HEAD=`4252adf46e02557222f5521d98f472e58f6aa51c`。两者之间ACE有真实变更（含962），**本报告不是冻结M1 tag审计**。下列规模以二次清点为准；下一阶段必须冻结来源并核漂移。该次`git diff --name-only -- experimental/ace crates/auto-lang-core/src/test_cases scripts/test-groups.toml`为空，只表示当时这些路径无未提交差异。

### 规范与现有工作

- `docs/specs/overview.md`只作导航，旧crate名/别名不覆盖当前Cargo/固定组。
- `docs/specs/auto-mir/architecture.md`、`auto-av/architecture.md`：共享语义、恢复不恢复trust、真实生产计量。
- `docs/specs/auto-ac/native-backend-and-retirement.md`：共享MIR唯一native源，重构不恢复旧HIR→CLIF路径。
- `docs/specs/auto-test-case/architecture.md`、`ace-test-policy.md`、`ace-source-only-replay.md`：父侧裁判、独立oracle、正式性能分轴、跟踪输入自足。
- `docs/design/55-ace-m2-complete-language-and-product-replacement.md`及`strategy/ace-m2-delivery-roadmap.md`：M2-A/B边界，945–948原M1-TAG门保持。
- PLAN-1002管文档式作者格式四例七检查试点，985管M2覆盖认证，1017管成本专项。均不是本包只读扫描前置，也不因本报告视为已完成。

### 第二轮固定来源（历史轮次，第三轮继续核对）

- 用户本轮授权：对F01–15逐项深入审核，证据/结论/建议写入本Plan；**不修改实现代码**。状态由drafting进入executing，合同范围/AC不变，plan_revision仍为1。
- 本轮冻结Git对象：`f9fb3854beb3886338156cfa61d16f7f84893cc9`。关键读取采用`git show <SHA>:<path>`，未改变checkout；相邻rg/本地读取的路径以`git diff`核与冻结源一致。主检出无相关源码WIP，其他会话文档未纳入本包。
- `git diff --stat 4252adf46e02557222f5521d98f472e58f6aa51c f9fb3854beb3886338156cfa61d16f7f84893cc9 -- experimental/ace`为空：首轮后二次清点以来ACE跟踪代码无漂移，15项证据仍可复用。
- `git ls-tree -r --name-only <SHA> experimental/ace`重算：35成员、741个Rust文件、432648行不变。按Git blob原始字节计算排序的`路径:SHA256`清单摘要为`682881c435b89f82e3f75d4accd191a510c89dcda6adeff257f68ccd03cadcae`。首轮`58e9…`来自工作区字节（换行形式可不同），两种口径不混用；可复核身份以本轮Git blob口径为准。
- `git tag --list '*ace*'`未列出本地匹配tag。这里只证明本地无匹配名字，不能证明远端未签M1或所有tag均不存在；最终M1标签身份仍unknown。本轮明确审计冻结的当前实现，不以此阻塞已授权静态分析。
- 工作方式：仅更新主检出Plan簿记，无1024实施代码/构建/探针源码写入，无额外worktree、原始材料或输出文件；内存算法/抽象状态重放不冒真实ACE执行。

### 第三轮来源与覆盖口径（历史轮次，第四轮继续核对）

用户要求按模块建立覆盖清单并逐项补齐，继续只读、不改实现。HEAD仍为`f9fb3854beb3886338156cfa61d16f7f84893cc9`，ACE/语言父侧/固定组无未提交差异，沿用第二轮Git blob身份。新增核查单元MC01–35与Cargo成员一一对应，另列语言父侧横切接缝MC36；不再用12个领域合并行代替成员清单。

每行记录职责/真实消费者、已核入口和状态路径、测试依据及下一项未核工作。**S=已做定向静态补审，P=仍有具名剩余检查，R=运行证据未取得**。35成员本轮均为S/P/R：每个成员都核了具体入口或合同及其测试归属，但没有一个因列入表就被判全分支通过。源码函数/测试文件名是静态导航，不是实际执行数。无状态的合同crate明确记录“不安装/不执行”，不硬套恢复/运行生命周期。

本轮可复核的改进：从热点转向成员边界，新增F16–20；后续以MC行的剩余项驱动深审，不再仅用F列表驱动。原清点统计保留为历史，未发生代码规模漂移。

### 清点与覆盖边界（首轮历史）

通过`git ls-files experimental/ace`选择跟踪`.rs`，排除target/生成残留：**35个workspace成员，741个Rust文件，432648行；其中372个文件路径含`/src/`**。含注释/空行/内联测试，不称生产LOC。初始直接递归磁盘结果混入target，已弃用。

清点身份：排序的`路径:文件SHA256`行以LF连接后SHA256=`58e9bf66597b8f73c91d82771416a12f01d59c352b6e68bbc053a9ebdc8cb817`。本次不是逐行全读；下表保留未审面。

| 责任域（全部35成员） | 首轮观察 | 深审待核 |
|---|---|---|
| core | Cargo、source/profile、resolve/typecheck/adapter、计量hook、link | 各语义/拒绝分支 |
| syntax-runtime/grammar/core/artifacts | 依赖、CST入口、旧parser门控、投影热点 | 每规则投影/编辑/恢复 |
| semantics-contract/semantics/semantics-store/source-query | 事实入口、host/store、旧推导边界 | 正负依赖、失败/权限/恢复 |
| lowering/av-core | 单源lowering、MIR/codec/verify/执行分层、instruction spec | 每操作/资源态跨层一致性 |
| av | 并行session、worker、fixture/formal公开面、差异算法 | 各会话事务/取消/观察 |
| native/case-worker | 平行lowering、profile选择、增量编辑、IPC | ABI差异与全失败状态 |
| a2x-core/rust/c/store/worker | TargetProvider、分层key与依赖 | emitter重复、目标差异、oracle独立性 |
| artifacts | CAS/锁/GC/发布入口、subproc/sysinfo | 崩溃/并发状态机及测试 |
| comptime-contract/core/comptime/source-cte | 合同/服务/宿主依赖、公共入口 | 两条CTE通路重叠与消费者 |
| abi/foreign-contract | 合同及native/A2X/store依赖 | 布局/资源/权限及桥接oracle |
| data/text/collections/typed-entry/trace-runtime | no_std边界、历史同源、库依赖 | 导出ABI/单线程/fault/预算 |
| tests/conformance/stdlib/data-runtime/text-runtime | 跟踪清点、固定组导航、1002试点 | 原义务完整映射、遗漏/重复执行 |

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：已实现ACE架构/重复/状态与测试接缝只读审计。当前知识以当地Specs/本Plan所引现行合同为准。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/native/src/closures_incremental.rs](../../../experimental/ace/crates/native/src/closures_incremental.rs)：F01 UTF-8编辑/原实现；读取SHA256前16 `dd5a2747a9ca4a01`

- [experimental/ace/crates/av/src/closures_fn_session.rs](../../../experimental/ace/crates/av/src/closures_fn_session.rs)：既有边界算法复用比较；读取SHA256前16 `c325ca50dd9059fd`

- [experimental/ace/crates/core/src/lib.rs](../../../experimental/ace/crates/core/src/lib.rs)：嵌套/panic观察作用域候选；读取SHA256前16 `906442f4d5f2ab0f`

## 5. 详细设计

### 首轮发现（保留调查历史；当前裁定见下方第二轮）

P1先核正确性/观测可信度，P2实施前厘清边界，P3后置清理。表中路径均相对`experimental/ace/crates/`；行号须在正式执行重定位符号/hash。候选不等于批准重构，不把未实测假设升级成缺陷。

| ID / 优先级 / 分类 | 代码依据、问题及建议 | 保留边界与验证 |
|---|---|---|
| F01 / P1 / 静态反例 | `native/src/closures_incremental.rs:647`的diff只将旧range退至字符边界，替换仍截`b[start..end_b]`再lossy解码；`:299`传入`edit_body`。AV的`av/src/closures_fn_session.rs:839`已有双端UTF-8边界算法，值得复用。 | 内存逐句重放`é→ê`得range `0..2`、replacement `�`，重建不等于新串。拟测真实闭包源码`apply_source`与fresh一致性，覆盖中文/emoji/插删/相同串/公共后缀。尚未跑Rust/PE，不宣称证明错PE；M1处置归原owner。 |
| F02 / P1 / 观察状态风险 | `core/src/lib.rs:127`的capture_body_work直接覆盖COUNTS，不恢复外层；`:283`stage、`:338`type capture仅正常返回恢复。相对地`core/src/source/parser.rs:77`已有Drop恢复的capture_syntax_work。 | 统一作用域恢复规则，先定嵌套隔离/累加语义；验证嵌套和catch_unwind后外层/后续窗口计量准确。未跑panic反例，不称既有签章无效。 |
| F03 / P2 / 脆弱分派（含M2） | `native/src/driver.rs:50`按message.contains的unknown character、found `[`、逗号等文案决定扩域；961/962继续加字符串条件。宜用结构化surface/unsupported原因。 | 保错误stage/code和原profile身份；只改文案不得改选路。真语法/类型失败与窄域不支持须区分，不能遇错一律升级。 |
| F04 / P2 / 确认机制重复 | `native/src/collections_lower.rs:895`与`closures_lower.rs:948`的binary重复比较、除零、MIN/-1、溢出CLIF逻辑；20行归一块扫描也命中frame建立。宜抽同ABI/语义的标量发射、checked call。 | closure捕获/调用/准入保持专有；正常、溢出、除零两条真实PE oracle保留，不把独立oracle也共用被测实现。 |
| F05 / P2 / 历史会话分叉 | AV公开session/data/owned/collections/864/closures/closures_fn/query_entry等路径。864源码会话whole compile/lower再split，carry不等于前端增量。宜建立通路职责表，再选M2主通路和共用prepare/commit/rollback。 | 列输入、支持域、接受态、真实生产、恢复、消费者。体/签名/坏编辑、恢复后首改分别观察；不能合成万能session或直接删旧profile。 |
| F06 / P1待核 / 提交与指纹 | `av/src/collections_864_session.rs:82,109`以Debug文本FNV作指纹，inner.apply_revision成功前已更新fps；warmup先clear。失败后外层与内层是否一致需核。 | 有效→注入失败→同源重试/修复，核fps、accepted、root、输出原子性及删函数；先查历史入口真实消费者。候选map成功后swap、规范wire身份为建议，尚未实测失败链。 |
| F07 / P2 / 成本假设 | `core/src/source/adapter.rs`把resolved发Atom文本再descriptor bind，按顺序/片段重锚source span。宜测emit/bind/remap后评估typed builder，避免重复结构转换。 | 这本是有效接缝，不是已证无用代码。保Atom导入/descriptor验证/源位置；同进程不等于免verify。无实测，不承诺倍数。 |
| F08 / P2 / 事实与旧推导并存 | `source-query/src/consumer.rs:232`走bind_with_facts；core保TypeSupply/旧推导/typecheck及profile级入口。宜列真实入口→事实生产者→绑定验证→后端，区分独立检查与重复综合。 | 扩共享事实能力，不在各backend补类型规则；未覆盖profile不强删旧路。不将所有verify视为冗余。 |
| F09 / P2 / 职责混合 | core导出link并依赖Windows进程/Job APIs；artifacts导出subproc/sysinfo；av正常lib公开bench/formal/fixture，native公开typed_fixture/workload；abi依赖native/A2X/store。 | 按调用图评估中立模型/执行宿主/test support，优先复用auto-tool-execution；cargo tree与编译成本测后决定拆crate，不按名字搬、不破正式recipe和资源观测。 |
| F10 / P2 / IPC机制可复用 | `case-worker/src/main.rs:173`bounded reader与`native/src/bin/mn-collections-worker.rs:63`的take(cap+1) reader在EOF/IO/UTF-8/LF处理不同，abi/sema另有reader。 | 可共用有界读取机制，保协议适配层及原cap/会话处置。真子进程测长行/截断/错编码/多行/IO错/超时；不得统一成更宽松wire。 |
| F11 / P2 / runtime同源 | text-runtime文件头明确为data-runtime探针shim同构改名裁剪，两者有ledger/计数/budget/Win32 heap；collections已经依赖text。宜比较后抽no_std资源原语。 | 先对照导出ABI/返回码/trace/计量。保静态库独立导出、panic-abort、单线程和无CRT合同；探针/产品不同面不直接别名。保双free、ledger/分配失败、UTF-8/NUL原oracle。 |
| F12 / P2 / 现有良好复用基础 | a2x-store已有TargetProvider及semantic/target key分层，ManagedStore共享CAS；query的PageSink/QueryHost、source-cte的CtExecutor均可扩展。 | 优先扩已有机制，不建第二存储/runner/求解器；核span无关key、目标/runtime身份、恢复重验和能力重授。C/Rust printer、native ABI与AV执行保目标差异。 |
| F13 / P2 / 大模块维护压力 | resolve 10395行，HIR verify 7997，AV execute 7058，CST project 6769，HIR adapter 6315，MIR verify 5272（含注释/测试）。 | 按scope/事实、资源/CFG、投影/lowering家族拆内部模块，保单一规则owner与public入口；不按固定行数切，不去掉独立授信边界检查。 |
| F14 / P2 / 测试组织 | 历史Plan命名/fixture/formal入口并存，1002只验证四例七检查，不代表全ACE迁移。应做case/check/native test→不变量→consumer→新位置映射。 | F01/F02/F06作候选反例；长期语义名保旧身份映射。保独立期望、负控、恢复/在线编辑/资源原义务，不能删例/ignore/减样本降成本。 |
| F15 / P3 / 注释事实漂移 | core lib头仍称HIR直达Cranelift和原型隔离，但实际已退役；av-core头称零外部依赖，Cargo已有foreign-contract。 | 随模块清理，以实际图和Spec为准；不把陈旧注释误报成循环依赖。hash工具重复也先区分std/no_std/用途，不能为几行工具引入大反向依赖。 |

### 第二轮逐项审核（历史轮次结论，固定来源f9fb3854；当前复审见第七轮）

本节每项分别记录证据、结论、建议。`confirmed`表示所述静态事实/不变量缺口已确认，**不表示已修复或通过集成复现**；`candidate`表示收益/方案尚待测量；`rejected`仅否定本节指出的过度解释，不删除原观察。F01/02/06优先，继而核其余项。下列短路径仍相对`experimental/ace/crates/`。

#### F01：UTF-8增量差异 — confirmed（算法保真缺口，P1）

**证据。** `native/src/closures_incremental.rs:647–665`先在原始字节求共同前后缀，随后只调整旧range，replacement仍用原切点并lossy解码。`:299–308`的实际消费者把它送到`ClosuresFunctionFrontend::edit_body`。`core/src/source/closures_fn.rs:521–540`只验base SHA及旧range是否合法；`:666–680`把replacement拼成new_source再解析，不知道调用者请求的完整新源码，因此无法检查“拼出来的源码是否等于请求”。AV对应函数`:839`对新旧两侧共同调整前后缀。

**结论。** 11组内存逐语句算法重放中，native有6组重建不等于新串，AV对应算法0组；失败含`é→ê`、`中→串`、`😀→😁`、带公共后缀以及完整函数源码内的字符串替换。ASCII、同串、完整多字节插入/删除5组作为合法对照成功。`é`的`c3 a9`与`ê`的`c3 aa`使prefix停在1，旧range变为0..2，而替换只剩`aa`，得到U+FFFD。该范围校验会通过，故“下游旧range检查已经保护它”的解释被排除。是否在某真实支持程序上被后续拒绝、成功编译了替换字符或最终执行差异，仍需真实测试，不能从模型宣称错PE。

**建议。** 共用AV的双端边界机制，额外守恒断言`splice(old, range, replacement) == requested`；复用内部算法，不让native因此依赖整个AV宿主。后续在真实session测试上述三类字符、共同后缀、字符串/标识符/注释及fresh/warm语义；保现有路由和预算。修复owner为native增量/共享源码编辑维护者，当前仅记录，不改M1。

#### F02：计量作用域 — confirmed（嵌套丢失与unwind恢复缺口，P1）

**证据。** `core/src/lib.rs:127–131`没有保存旧COUNTS。外层记录a→内层记录b并take→外层记录c时，外层旧map已被覆盖，内层结束又设None，外层最终得到空map。`:283–288`、`:338–343`保存旧值但在`f()`后才恢复，panic unwind会跳过恢复。`core/src/source/parser.rs:77–91`的Drop恢复及`core/src/lib.rs`的observer Scope已有可参考方式。`core/tests/cases/closures_fn_912.rs:98`实际同时嵌套stage/body两个**不同**窗口；native冷启动也调用body capture。

**结论。** 同类body窗口嵌套不保真、stage/type窗口panic恢复不完整是控制流确定的缺口；普通Result::Err会正常返回，不属于这里的异常漏洞。现有测试的“stage包body”不等于“body包body”，不能据此说同类嵌套已覆盖。未证明当前正式验收实际触发同类嵌套或panic后继续计量，因此不判既有验收全部失效。

**建议。** 统一RAII恢复，并先明确内层数据隔离还是并入外层（现有stage/type正常路径是隔离）。永久反例测同类嵌套、异类嵌套、panic被catch后继续记录、无窗口零开销/无输出；不能只加catch把panic变绿。owner为core观察接缝维护者。

#### F06：864会话外层状态 — confirmed（条件性失败重试缺陷，P1；测试消费者边界）

**证据。** `av/src/collections_864_session.rs:104–121`在inner返回前把新fp写入self.fps；warmup`:79–94`亦如此。内层`collections_session.rs:255–274`会生产/验证/持久化候选，`:294–327`会link和执行，任何`?`或非Success均可返回；只有`:328–331`才切revision/units/identity。内层原子提交本身存在，并未遗漏。仓内符号搜索只找到`av/tests/cases/av_collections_session.rs`三项直接测试消费者，未找到worker调用该外层类型；当前范围不包括外仓/外部动态消费者。

**结论。** 数据流可构造：接受A→请求B，外层fp=B，内层生产/执行失败仍为A→重试B，外层判相同不提交，内层carry A。纯抽象状态重放得到第二次submitted=[]、fp=B/inner=A。无需依赖hash碰撞即可发生；但本轮未在ACE触发真实错误，失败场景的实际源码/IO故障测试待补。原三测试只验fresh/body-edit/hit/helper/interface成功面，没有失败后相同请求重试。另，外层仅遍历新units，内层order含旧units，两层均无删除输入：不能把这个body-edit API当支持完整拓扑删除；删除是否应支持须核合同后另定，不能直接加成新承诺。Debug文本指纹是维护/不必要失效风险，未证明碰撞故障。

**建议。** 外层生成candidate fps，inner成功后整体swap；失败留旧fp，验证执行trap/持久化失败后重试与修复。warmup同样处理。先保历史测试入口的支持域，不把它升为M2主会话；owner为AV集合session维护者。

#### F03：诊断文案决定profile — confirmed（耦合）；candidate（替代设计）

**证据。** `native/src/driver.rs:50–81`用code加message子串分类；`:95,124,140,159`决定是否继续编译梯。实际次序Scalar→Core旁路→Record→TextRecord→Numeric→Aggregates；Core旁路失败有意不终止（`:102–119`），并非严格单调超集梯。961/962新增字符/构造确实扩充文案匹配。

**结论。** 相同code但文本不同会改变分类，为静态可判的控制流耦合；本轮没有证明某条现有合法源码当前被错分。不能把所有失败归为surface，也不能将Core旁路改成首错终止。

**建议。** 引入结构化reject reason/token或明确profile能力判定，message只负责展示；保窄profile优先、Core旁路、terminal diagnostics、attempted/surface_rejections报告身份。后续正反控必须含同码不同原因、只改文案、真正parse/type错误和组合语法；owner为共享源码入口/native分派维护者。

#### F04：native算术发射重复 — confirmed（可切出的52行机制）

**证据。** 从冻结源码分别提取两个`fn binary`的完整花括号体，**52行逐字相同**，SHA256=`058843c622dfa5398afd9ea3ce317223f58bc30ae88088110e017c00063efbe6`。内容包括6种比较、Div/Rem除零、MIN/-1、i64中间计算、i32上下界与-30/-31状态。两个Lower结构及closure参数/布局准入则不同。

**结论。** 这是确定的维护重复，不是两个独立oracle，也不是整份lowerer相同。frame建立的块相似尚不足以证明完整生命周期可合并。

**建议。** 首片仅抽共享CLIF标量运算（返回值/guard由调用方接入），保operand load/store、捕获/布局准入与资源ABI分开。检查正常值、边界溢出、除零、MIN/-1的原trap/cleanup；不要连带合并布局、cache和wrapper。

#### F05：会话分叉 — confirmed（职责分散）；rejected（全是可删除重复）

**证据。** `av/src/session.rs`有PreparedEdit及独立prepare/commit/cancel/restore；`data_session.rs`有单体、批量与cap编辑；`owned_session.rs`处理拥有资源及env发布；`collections_session.rs`接Typed MIR增量单元；`closures_session.rs`还有admitted unit、restore_install和root rollback；`closures_fn_session.rs`在其上承担源码/创建点/接受态恢复。实际worker直接用DataIncrementalSession（`:654`）、ClosuresFunctionSession（`:1635/1754`）与FunctionIncrementalSession（`:2794`）；864外层只找到测试消费者。

**结论。** Source host、typed安装层、owned/closure生命周期、历史测试外层不是同一抽象。源码whole重编译再split的864路径确实不能代表函数前端增量；但不能据多个session名断言每个都不合理。

**建议。** 共用候选/接受态和观察术语、身份/恢复/失败协议，先画各入口→状态→安装层→worker表，再确定M2统一host facade；保内部domain策略与typed入口，先收敛重复机制再评退役。任何删除先有消费者映射与原测试去向。

#### F07：Atom往返 — confirmed（重复转换）；candidate（性能收益）

**证据。** `core/src/source/adapter.rs:435–474`走parse/compile_program；`:1037/1056`和`:1151–1178`分别在旧编译及共享emit中descriptor bind并重锚，`:616/631`是函数/批量scoped接缝。因此事实入口并未免掉Atom生成/解析，warm路径也有对应转换；但函数scoped接缝避免重走所有body。

**结论。** 往返与重锚是实际工作，耗时占比/是否瓶颈未知；“全部是无用序列化”及“warm必whole重算”均不成立。文本还承担descriptor和跨入口行为一致性。

**建议。** 分别记录parse/resolve/emit/bind/reanchor/verify的分配和时间，再评内部typed HIR builder。保外部Atom入口和descriptor规则；typed构造不得绕开CheckedModule minting。与1017共享归因方法，不提前承诺加速。

#### F08：类型权威分裂 — confirmed（profile差异）；rejected（事实失败静默成功fallback）

**证据。** `source-query/src/apply.rs:588`按事实绑定；`:603–619`明确TextRecord外库路径没有solver facts，仅用foreign supply解析。`consumer.rs:376–421`在特定语言拒绝时调用旧compile_scalar_source只取诊断：旧路若成功也返回source.fact-diagnostic拒绝，不能取得fallback成功产物。A2X store`:936–1004`的Scalar走共享事实/CT，Core仍compile_source，Record/Text/TextRecord/Collections/Numeric/Aggregates走各自现有入口。native梯同样是scalar/core事实与更宽旧路径混合。

**结论。** 不能把带product/facts名字的所有profile宣称统一事实推导；也不能把诊断oracle当静默类型生产回退。只读核到的差异是有限支持域与兼容诊断机制，尚未证明同一成功请求重复综合两遍类型。

**建议。** 以profile×consumer列事实覆盖/旧综合/独立verify/诊断oracle，随M2语言能力扩面收敛共享权威。保支持域外显式拒绝和诊断兼容；不直接删除TextRecord foreign解析或独立verify。

#### F09：依赖层与测试宿主混合 — confirmed；candidate（crate拆分收益）

**证据。** Cargo正常path图：native→av→source-cte；abi→native及A2X/store；artifacts→core/auto-cache/auto-tool-execution。av的bench/formal/fixture模块多数无cfg(test)，只有部分batch观测受feature门；native同样公开typed_fixture/workload。`artifacts/src/subproc.rs:1–28`已经委托auto-tool-execution，**不是第二套独立Job执行器**；但`core/src/link.rs:152–520`仍有本地Job/reader/capture实现。

**结论。** 宿主/支持代码进入正常依赖和core职责偏宽为事实；未测编译时间/产物链接占用，不能断言所有测试代码都进入最终PE。已有subproc委托应保留，不重复提取。core与owned执行器是否可等价替换需比较其caps、buffer复用、Job失败与故障注入口。

**建议。** 先将模型依赖和执行宿主/test support接口分开，再评拆crate；优先适配现有执行器而非新库，保持无反向循环、观察与执行边界。不一次迁移所有模块或借feature隐藏失败测试。

#### F10：worker读行 — confirmed（机制重复与差异）；candidate（协议缺陷程度）

**证据。** case-worker`:173–211`与sema `semantics-store/src/ipc.rs:267–305`同属fill_buf预分配有界机制，IO Err会映射None/已有部分行，UTF-8使用lossy；case-worker主循环`:43–45`把None当正常EOF。native collections worker`:63–78`使用take(cap+1)，要求末尾LF并用serde_json::from_slice严格解码。case/sema cap不含LF，native的bytes.len包含LF。

**结论。** “所有reader都无界/均不安全”不成立，两类均有限界；EOF、IO、编码、cap计量差异确定存在。IO被折成EOF、字符串字段内坏字节被替换是需要独立核验的错误分类/字节保真风险，但尚未证明可绕过nonce/hash/父侧验证，不报协议攻击成功。不得以统一实现默改现有wire。

**建议。** 共用返回bytes+明确EOF/TooLong/IO/InvalidEncoding的低层机制，上层按版本化协议适配LF/末行/空白及cap；先补有界内存、错编码、Reader IO错和父侧拒绝负控，再决定历史兼容行为是否需要版本升级。

#### F11：运行时同源 — confirmed（共性）；rejected（同构即可整体合并）

**证据。** 冻结data-runtime有64个函数、39个pub extern导出，text-runtime有58个函数、30个pub extern导出（静态签名提取，含自供memcpy等，不是链接实际导出计量）。二者共有ledger/try_alloc/owned_free/UTF-8机制，但data保p774记录/handle探针；text在`:61–143`新增allocator/free/poll hooks、metadata释放、4096字节分块取消和ERR_CANCELLED=-12，data只有原-1..-11状态面。

**结论。** 历史同源并不等于当前同合同；若把text替回旧探针版本，会丢取消/元数据/可注入分配能力。直接把两库一起链接还须考虑双方自供memcpy/memmove/memset/memcmp符号，不能承诺原样并库。

**建议。** 候选仅限no_std内部ledger/UTF-8原语，外部符号、状态、hook、预算、trace由各适配层保持。先核独立静态库及panic-abort构建、失败回滚/双free/取消与元数据清理，再评共享。禁止以增加libc依赖解决复用。

#### F12：已有复用接缝 — confirmed（应沿用）；rejected（需要再造统一基础层）

**证据。** `a2x-core/src/target.rs:111`的TargetProvider约束target身份、runtime/prelude/fileset，a2x-store消费此trait，semantic键与target键分层。`source-query/src/host.rs:260–319`在cancel/base/key/deps重验和sink.put成功后才切CurrentRecord。`source-cte/src/executor.rs:150–185`有CtExecutor，ColdAvExecutor沿共享lower→MIR/ABC验入→真实AV；旧comptime可消费该执行接缝。ManagedStore已有liveness/GC/CAS/锁合同。

**结论。** 系统已有多处有效单源机制。为重构再造store/query runner/第二VM会扩大分叉，而不是收敛。本轮没有做并发/损坏恢复运行认证，不对全部边界无缺陷背书。

**建议。** 新目标扩provider、新CT宿主注入executor、新页面适配sink，保现有key/version/权限与恢复重验；先用这些接缝容纳M2变化，再评是否有真实抽象缺口。

#### F13：大模块 — confirmed（维护热点）；candidate（拆分边界）

**证据。** Git blob计数与首轮行数一致；resolve 10395行/158处fn匹配，HIR verify 7997/110，AV execute 7058/135，CST project 6769/132，HIR adapter 6315/101，MIR verify 5272/70。fn为文本匹配含方法/可能的注释，不称精确语义函数数。HIR CheckedModule/CheckedFunction和MIR CheckedMir分别有不同minting边界，verify文件还公开多种资源/函数验证入口。

**结论。** 可以按规则族缩小维护单元，但文件长并不能证明运行慢或检查冗余；不同IR/信任阶段verify不能合并成一个万能pass。

**建议。** 优先在同crate拆私有模块（scope/事实、CFG/init、owned资源、CST投影家族、表达式/控制流lower），维持public API和凭证构造私有性；与功能扩面分开提交，先保逐语义反例再移动。暂不提出新crate/新公共接口。

#### F14：测试更新 — confirmed（需守恒映射）；unknown（全量覆盖率）

**证据。** 语言侧真实父门`crates/auto-lang-core/src/test_cases/ac.rs:411/941/1048`负责response准入、manifest持久化、流比较；ac_av也复用流比较。固定组区分ace-source/av/native/lowering等职责，ACE policy冻结13慢项身份并把正式性能分轴。1002合同仍是四case/七check试点。抽查native closures tier未见Unicode编辑反例，864三测试只有成功态；AV测试有非ASCII文字，不能因native样本无该词便说全仓无Unicode测试。

**结论。** 新格式成功不代表检查义务/故障态迁移完成；当前没有全量case/check→不变量/consumer→新入口总表，覆盖率unknown。未运行测试发现，因此不报告实际执行总数或全部漏测项。

**建议。** 保父侧裁判及独立预期，将F01编辑保真、F02异常恢复、F06失败重试列为具名永久反例；1002只改变作者表示，985核完整义务，1017治理成本，重构包维护真实身份映射。准备/执行/清理成本分开，不能用删除、ignore或降样本获得优化。

#### F15：注释/名字漂移 — confirmed（文档）；rejected（据此断言依赖违规）

**证据。** core lib头仍写HIR→Cranelift，而Cargo及native退役Spec已把lowering移出；av-core lib头写零外部依赖，但Cargo正常依赖foreign-contract和serde_json。core经syntax-grammar间接用auto-atom/val，因此旧“禁止这些依赖”注释与实际图/模块规范有差额。

**结论。** 注释陈旧确定，实际依赖图没有因此自动构成循环/违规；是否保原隔离约束应按当前canonical Spec及历史决策核定，而不是删依赖迎合旧注释。

**建议。** 随对应模块小步修正架构说明/名字，单独保可追溯旧Plan/协议身份；通用hash函数按std/no_std、密码学/版本检测用途分组后再决定复用，不为短函数引入宿主重依赖。

### 第三轮按成员覆盖清单（历史轮次，MC01–35；当前闭合见第六轮）

下表成员路径相对`experimental/ace/`，源码/测试名相对该成员目录。全表状态S/P/R，含义见§4。消费者指本仓已定位的静态调用/依赖或外部worker/静态库消费；不能因为没有Cargo正常依赖边就判定runtime或worker无人使用。表中“保留”是本次有依据的建议，不是全模块无问题证明。

| ID / 成员 | 职责、入口及实际消费者 | 本轮补核的状态/拒绝或复用依据 | 测试依据（未运行）及具名剩余检查 |
|---|---|---|---|
| MC01 `crates/a2x-c` | C目标provider；`src/provider.rs`的descriptor/runtime_source/produce_fragments/assemble_fileset；worker、ABI C侧消费 | AXC1/2/3与Rust命名空间分开，Scalar/RecordText/Collections按family分发；保C runtime/ABI目标差异，不合并printer | worker `tests/cases/a2c_rt_cert.rs`、`a2c_coll_cert.rs`及inline测试；`ace-a2c-abi`；余：逐运算UB/溢出、别名/错误清理和C库符号闭包 |
| MC02 `crates/a2x-core` | Checked HIR语义视图/函数计划；`src/prepare.rs::plan_function`和target trait；C/Rust/store共同消费 | 工作栈遍历与MAX_STRUCTURE_DEPTH=400有具名a2x.too-deep拒绝；语义指纹与target配置分开；保共享语义层 | `tests/conformance.rs`、`tests/cases/sem_fingerprint_902.rs`；`ace-a2x`；余：每种新聚合/深类型walk/print/drop边界与指纹不变量 |
| MC03 `crates/a2x-rust` | Rust provider、lower/print/runtime；worker、stdlib及AV/ABI dev测试消费 | `src/provider.rs`明确foreign-unsupported，AXR族与C分开；RUST_PRINT_VERSION=3负责语义缓存失效；保拒绝面 | worker `tests/cases/cert_coll_904.rs`等；`ace-a2x`；余：每语法分支、借用/提前释放、fresh/restored片段等价及跨target拒绝 |
| MC04 `crates/a2x-store` | A2xSession、anchor、模块装配；`src/lib.rs`与`src/modules.rs::cold/read_fragments`；worker/ABI/stdlib消费 | provider委托、resident target隔离、source/env恢复接缝；模块requirements进入root；不另建缓存 | worker `tests/cases/store_cache.rs`、`store_coll_933.rs`、`a2x_decl_env_929.rs`；`ace-a2x`；余：完整root/manifest发布失败矩阵、其他target居民保全 |
| MC05 `crates/a2x-worker` | daily-emit与显式certify两轴，`src/bin/a2x-worker.rs::main/read_source`；语言ac_a2r_hir/ac_a2c_hir父侧消费 | 找到源码两次读入与stdin读到EOF无前置cap（F18/19）；正式compiler/process在crun/独立formal路径，保日常spawn-zero | `tests/conformance.rs`连48个测试源文件，含boundaries/cert/store族；`ace-a2x`；余：源变更/二读失败、坏编码/长输入真实父子反控与输出发布 |
| MC06 `crates/abi` | bridge、layout、cache、source_foreign及resource worker；A2X worker及显式协议父侧消费 | emitter identity含版本；source restore接收当前capabilities/environment，C object read有资源账户；不能持久化函数地址当授信 | `tests/conformance.rs`、`source_resources.rs`；`ace-abi`；余：所有物理lane与平台ABI、重授/替换provider、故障回滚的逐路径映射 |
| MC07 `crates/artifacts` | ManagedStore/CAS、codec、GC/lease/root；大多数ACE宿主消费 | `store/mod.rs::put_artifact/put_artifact_leased/get_artifact_bounded`及gc liveness互锁、chunk cap64MiB；subproc已委托公共executor | `tests/conformance.rs`、`store_resources.rs`；`ace-artifacts`；余：逐崩溃点/并发锁序、坏内部计数解码和目录链接拒绝；外框限长不能代内部预算 |
| MC08 `crates/av` | 源码/typed宿主、会话、cache/restore/worker；native/comptime/语言协议消费 | 在R2基础上补worker→Data/ClosuresFunction/FunctionIncremental入口、864测试外层边界；各会话非同一层，保typed与source分层 | `tests/conformance.rs`连会话、恢复、资源、语言族；`ace-av`；余：每session的prepare/cancel/late commit/坏编辑/恢复后首改状态表，不以F01/06代整库审查 |
| MC09 `crates/av-core` | MIR/ABC verify、LoadedModule/Program、真实执行；lowering/AV/native/CTE/ABI消费 | `src/abc_verify.rs`重贴envelope身份并deep verify，区分可重验unit与可执行closure；loaded_program IndexBudget保8MiB与RAII Charge | `tests/conformance.rs`、foreign_resources、public_closures/public_dynamic；`ace-av`；余：逐opcode的codec/verify/fuel/trap/drop四层一致性及宽类型/断点状态 |
| MC10 `crates/case-worker` | check/build/run/syntax IPC适配；语言`test_cases/ac.rs`父门消费 | `src/exec.rs::execute`先req.validate/平台/逐请求backend，product运行仍受contained executor约束；读行差异见F10 | `tests/cases/protocol.rs`等及conformance；`ace-native-worker`；余：每请求kind/输入类型/超限/IO错的真实终态，拒绝后无产物和reap |
| MC11 `crates/collections-runtime` | 原生集合/closure的Cell/Frame/arena运行时；native通过静态库链接消费 | `src/arena.rs::allocate/hold_external/tighten_limit`以checked_add、页与external共用预算；lib frame和fault/观察机制；正常Cargo无人依赖不等于无consumer | native collections/closures/resource tiers通过真实PE观察；余：每Cell所有权/retain/release/取消、碎片和外部frame字节边界；不将arena计数当OS全内存 |
| MC12 `crates/comptime` | 旧recipe deck/driver/store/worker，语言CTE probe消费 | `src/driver.rs`有ephemeral/store、累积request wall、cancel独立终态；store恢复要全身份/freshness；预热与请求时钟分开记录 | conformance的cte_admit/cache/freeze/resources/schedule/snapshot族；`ace-comptime`；余：旧recipe与ordinary-source两通路实际职责差额、根预算/取消后再请求 |
| MC13 `crates/comptime-contract` | request/frozen/admission数据合同；comptime/core消费，无执行安装态 | `src/request.rs`recipe与Closure/Snapshot lane显式，frozen envelope含闭包依赖/meta/caps，admission区分language与resource/cancel/infra | inline与comptime的cte_admit/freeze测试；`ace-comptime`；余：所有字段未知值/错误tag相容性及schema边界；保817与936身份，不能同名替换 |
| MC14 `crates/comptime-core` | 旧recipe绑定/替换/依赖/调度；comptime消费，无独立VM | `dependencies.rs::manifest_fresh`按当前input/caps重建比较，含负依赖；schedule先前引用/环拒绝；consumer backfill后仍verify，裁剪失败回FULL源 | inline及comptime的cte_schedule/first/admit族；`ace-comptime`；余：所有绑定位置和循环/重名/回灌类型相邻反例、裁剪fallback成本 |
| MC15 `crates/core` | AST/HIR/descriptor/源码绑定、资源verify与link；多数编译消费者 | R2计量/resolve/adapter外，补`source/packages/binding.rs`源段边界/诊断定位；CheckedModule/Function负责不同凭证；手写parser门控保持 | conformance的source/candidates/packages/diagnostic等族；`ace-source`；余：语言构造×profile表、模块/包可见性、语义拒绝与资源拒绝、全部verify分支 |
| MC16 `crates/data-runtime` | p774 ABI/资源探针no_std静态库；data-conformance机器调用消费 | ledger/budget/fault和p774记录/handle家族，单线程进程heap合同；与text新增hooks不同，保独立导出 | `tests/data-runtime`的v02–v08；运行库工具准备并非自身test target；余：每导出签名、fault序号/trace/清理与平台边界 |
| MC17 `crates/foreign-contract` | typed decl→facts→物理lane/plan/严格codec；core/av-core/native/ABI/C/source-query共享 | `plan.rs::project`先facts.validate；resource FieldKind表示域宽于source_ok，不能按枚举变体推支持；typed.validate具名拒绝，非执行授权 | `tests/conformance.rs`Windows typed→facts→plan/tamper；`ace-foreign-contract`；余：每FnForm、layout、codec字段及source/typed适用性全矩阵 |
| MC18 `crates/lowering` | Checked HIR→共享Typed MIR；AV/native/CTE/ABI消费 | `hir_adapter.rs`whole及function collections/closures入口，错误/源映射和MIR版本保单owner；不能让backend各补语义 | `tests/conformance.rs`、closures_fn_912等；`ace-lowering`；余：每Expr/Stmt/owned place/闭包和宽数值映射及不支持型反例 |
| MC19 `crates/native` | MIR准入→CLIF/COFF→PE，cache/typed与source会话；worker/ABI/comptime消费 | 复用F01/03/04，补cache CoffMeta绑定runtime/layout和typed_session接受态接口；旧HIR直达native不得恢复 | conformance及typed_entry(_incremental)_tier；`ace-native`；余：所有ABI/布局/恢复/重链接失败及codegen分支，frame cleanup与真实资源观测 |
| MC20 `crates/semantics` | 有限双向类型求解/反射/query；source-query/comptime/sema-store消费 | `infer.rs`Ok/Diags可有summary，ResourceExhausted/Cancelled不当程序裁决；budget.charge检查steps/deadline；query含正负依赖freshness | src inline测试与sema-store parity/meta/budget测试；`ace-semantics`；余：所有constraint合流、模板实例/递归与负查询失效，不把I32/Bool有限Ty当完整M2类型 |
| MC21 `crates/semantics-contract` | 逻辑身份、可移植元数据页/assembly引用；众多共享消费者，无执行态 | identity.domain_hash长度前缀隔离；codec cap512KiB/向量上限/嵌套8192与尾字节拒绝；assembly deny_unknown_fields，声明要求进版本 | conformance的assembly_944/sema_identity_contract及inline codec；`ace-semantics`；余：全部nested字段和非法tag、producer/schema/version跨consumer恢复 |
| MC22 `crates/semantics-store` | S796语义页/worker/query处置；语言sema adapter经IPC消费 | `store.rs`fetch区分HitStaleProvenance与Stale/Miss，run_query_flow使用互斥Disposition；kind/target/facts owner同时核；IPC见F10 | conformance的sema_boundary/meta_query_cache_deps/cache_r4/budget；`ace-semantics`；余：每处置→父侧阶段映射、GC引用与source位置刷新完整分支 |
| MC23 `crates/source-query` | 普通源码fact页/实例化/host/store/restore；AV/native/A2X/ABI/CTE消费 | `restore.rs::restore_verify/restore_bind`拒缺页/错body/覆盖，不重solver当验证；session cache root与resources/cancel分开；F08不能泛称全profile facts | tests的architecture_facts_941/restore_identity_941/instances_restore/scalar_parity；`ace-semantics`；余：每正负依赖/实例host事务、能力/签名/来源漂移与事实完整性 |
| MC24 `crates/source-cte` | ordinary-source CT planner/service/executor/store；AV/A2X/comptime消费 | schedule RootBudget=10s，per-call min(2s,remaining)，调用递归≠CT环；service.execute_ct hit重验expect、miss真实执行；meta能力在产品入口先核 | tests的meta_capability_941/architecture_restore/meta/child及inline；`ace-comptime`；余：所有nested CT、同根single-flight、失败后冻结/重授与恢复首改 |
| MC25 `crates/syntax-artifacts` | A01语法、A02源码/token、A03分页CST持久化；source-query/syntax宿主消费 | `get_grammar(_by_id)`先decode后grammar_id比较，暴露F16；根按会话revision、view lease与required闭包，CST_HOT_BUDGET=8MiB | conformance的syntax_admission/budget/cache/depth_persisted；`ace-syntax`；余：坏页计数预分配、每子树跨度/归属/字段拒绝、迟到根与GC并发 |
| MC26 `crates/syntax-core` | 共享CST门面和旧有限strict投影；AV/语法对照消费 | `src/lib.rs::project_strict`先拒error/missing，ProjectionError分LiteralRange/Unsupported/TooDeep；生产前门re-export与历史投影分清 | frontdoor_reexport_927/core_projection/scalar_projection/generation/incremental；`ace-syntax`；余：旧投影全部消费者与退役价值、每literal/range/坐标边界 |
| MC27 `crates/syntax-grammar` | Atom语法配置→GrammarProgram；core/syntax/artifacts/case-worker消费 | compile_profile显式裁剪且影响canonical内容；declared_profiles防未声明门控；parse_charset明确受限ASCII，不当Unicode支持证明 | grammar_reject/profiles_927/depth/lossless/recovery；`ace-syntax`；余：循环/nullable规则拒绝、所有grammar DSL错误和profile组合相容性 |
| MC28 `crates/syntax-runtime` | scanner/CST/增量/handle/codec/recovery，无前端宿主依赖 | edit按NoConvergence/Unterminated/ComplexEdits显式fresh fallback；recovery必须前进；handle跨generation/lease校验；新增F16/17 | program_codec/handle/recovery inline及syntax-core增量测试；`ace-syntax`；余：坏程序预分配、lease churn、checkpoint/页重用和每恢复路径预算 |
| MC29 `crates/text-runtime` | 文本owned/UTF-8/资源no_std静态库；collections-runtime和native链接消费 | `ac_rt_install_allocator`只在无活buffer/ledger时设hooks，copy_poll分块取消并释放新buffer；metadata release与旧probe不同 | text-conformance contract/fault/lifecycle/textops/trap及native资源负控；余：所有导出取消/别名/部分失败、hook更换后旧资源/长度保真 |
| MC30 `crates/trace-runtime` | 两个test-only mark_a/b机器端，core trace capability/native PE消费 | 全62行读取：写stderr单字节、返回1/2，非生产stdlib；WriteFile结果未核、panic spin，不能提供独立可靠输出成功证明 | core调用顺序和host native tests，以父侧原字节为oracle；余：输出句柄失败/部分写的分类与是否应版本化，不把故意最小探针当通用I/O |
| MC31 `crates/typed-entry-runtime` | one-shot typed native帧machine端；native wrapper/typed_entry消费 | mn_typed_read校magic/version/reserved、checked total≤256KiB及cap再读payload，hash/nonce锚；mn_typed_check校sig/argc/长度，status互斥 | native typed_entry_tier/typed_entry_incremental_tier；余：每错误reason、半帧/尾随输入的父子合同及静态anchor单次调用前置；不能推广成可重入多请求runtime |
| MC32 `tests/conformance` | 跨worker/父裁判/生命周期/批处理集成，语言测试case由其消费 | conformance显式接batch/console/host/governance/native_lifecycle；governance调用预构建auto-case/worker，不是自身crate静态dep图 | `ace-host`/`ace-batch`/`ace-console`按原expression分轴；余：每host族→实际不变量/独立oracle/工具准备/忽略条件总表 |
| MC33 `tests/stdlib` | 标准引用/装配身份的AV与Rust真实consumer测试 | 唯一conformance测试明示“不声称callable support”；核真实声明/requirements、shadow控制、manifest tamper及rustc输出 | `ace-stdlib`；余：别把reference已入manifest冒新stdlib功能就绪，完整库行为属945–948/972后续，tag前不执行其前探 |
| MC34 `tests/data-runtime` | Cranelift生成真实ABI probe并调用p774；非普通语言语义引擎 | src规则/ProbeCx构建机器调用，conformance接ABI/lifecycle/reject/fault/trap/text/scale；值、trace、ledger是不同oracle | `ace-runtime`及ace-workspace原选择；余：逐case原输入规模/故障序号/条件/期望独立性，不按测试名推已测 |
| MC35 `tests/text-runtime` | 真实文本静态库ABI/资源测试，与data probe独立 | ProbeCx及contract/fault/lifecycle/smoke/textops/trap入口；保字节/码点/错误码和资源清理多轴 | `ace-runtime`；余：全部导出到case映射、取消hooks新义务的完整覆盖、真实工具/平台条件 |

### 横切消费者补审（MC36，不计第36个workspace成员）

语言父侧`crates/auto-lang-core/src/test_cases/`：ac.rs负责response/manifest/流准入，ac_a2r_hir.rs与ac_a2c_hir.rs把源码写到各case scratch后向预构建worker传source_path，daily不把golden交worker。当前父侧会核身份/产物，不应因worker自报成功直接PASS；原各recipe/batch/证书路径仍须分别核。MC36下一项是每adapter/action/recipe→worker→父oracle→固定组映射，包含raw/typed/source、emit/compile-run/资源与恢复轴，不能用一个ac.rs替全部接缝。

### 模块补审新增发现（F16–20；不改实现）

#### F16：语法程序内部计数驱动预分配 — confirmed（资源拒绝前分配，P1）

**证据。** `syntax-runtime/src/program_codec.rs:150–168`读取codec version/schema/name/revision后，直接将u32计数转usize并`Vec::with_capacity(n)`，尚未验证剩余输入或预算；规则、token、alt/steps、Pratt等还有同类点。Reader::take只在后续读元素时拒truncated。`syntax-artifacts/src/lib.rs:153–194`的get_grammar与get_grammar_by_id均先decode再比grammar_id，故后验身份比较不能保护预分配。

**结论。** 22字节头（version=3、schema u32、空name长度、revision u64、mode计数u32::MAX）足以走到首次为4294967295个String预留的请求；本轮只构造字节/算偏移，不调用decoder、不申请该内存、不进行压力测试。CAS外框64MiB即便生效，也不能约束22字节内编码的巨大数量。普通随机篡改可能先被CAS拒，但公开decode入口与被接受frame内的畸形payload仍应受控失败，不能借hash把内部数据视为已验证。真实allocator终态/可达坏store构造未运行，不声称已OOM或存在远程利用。

**建议。** 先按剩余字节/元素最小编码长度、总节点/字节预算验count，checked算术和可失败预留，按协议区分malformed与resource；审同族decoder而非只修第一处。持久负控用小输入/可观测预留账户证明“拒绝发生在大分配前”，保合法边界/roundtrip/grammar_id重验。owner=syntax-runtime/grammar恢复维护者，关联MC25/28。

#### F17：租约索引历史零计数不回收 — confirmed（元数据持续增长，P2）

**证据。** `syntax-runtime/src/handle.rs:67–83`publish仅retain树entries，keep从lease_counts正计数生成；lease`:89–100`为每个tree插入key，Drop`:145–151`只saturating_sub，不remove。全文件无其他lease_counts.remove/retain清零回收点。现有三项handle测试与syntax-core generation测试验证活性/跨session拒绝，没有索引收敛断言；仓内当前使用集中在这些测试，外部使用未知。

**结论。** 依代码顺序做1000轮publish→lease→drop抽象状态重放，entries=1、lease索引keys=1000、正计数=0。缺口是长会话元数据与publish遍历成本，**不是1000棵CST未释放**；不把测试使用面泛化成产品已泄漏。

**建议。** 最后租约释放时删除零项或在publish回收，保同时多lease及current/old句柄拒绝行为。新增lease churn测entries/lease索引收敛、旧树不可解析以及多lease最后一次释放边界；不靠强制清空活lease优化。

#### F18：A2X编译源码与报告SHA分两次读取 — confirmed（身份一致性缺口，P1）

**证据。** `a2x-worker/src/bin/a2x-worker.rs:251–265`的source_path分支先read_to_string得到编译src，再fs::read同一路径求source_sha256，第二读失败还unwrap_or_default；source_text分支则对同一字符串求hash。emit把第一份src用于编译、第二份hash用于EmitOutcome和诊断身份。父侧ac_a2r_hir.rs:172–189、ac_a2c_hir.rs:205–219实际发送source_path，因此不是完全无consumer的分支。

**结论。** 两读之间文件变化可形成compiled=A/hash=B，二读失败可形成compiled=A/hash(empty)。父侧独立身份检查可能拒绝该报告，未证明错误结果可PASS，不能称已绕过认证。即使fail-safe拒绝，身份来源仍不应取不同快照；本轮只核数据流，无并发文件写入实验。

**建议。** 一次有界读取原始bytes，从同份bytes进行UTF-8解码、编译与SHA；读错显式infra，不对空bytes补hash。后续通过可控Reader/文件替换反控与父侧错身份拒绝核验；不把失败消音或删source_path兼容面。

#### F19：A2X one-shot请求/源码无读入前界限 — confirmed（机制）；candidate（支持/预算策略，P2）

**证据。** 同worker`:114–125`在parse/dispatch前read_to_string到EOF，source_path分支亦先整体读入；与MC10的bounded-line worker不同。这是one-shot整JSON协议，不可以直接改成一行reader而改变格式。

**结论。** 下游结构深度/编译预算不能限制已经发生的输入读入分配。当前为受父侧构造请求的可信工具，尚未证明外部不可信暴露或现有输入超限，保持范围说明；“所有ACE worker均有相同预读cap”的假设不成立。

**建议。** 由recipe合同明确请求/source最大字节，take(cap+1)及超限显式拒绝，并保EOF framing/多行JSON；与F18共享同次读取而不是新加第三读。压力实测与新门限裁定后置，不在本轮擅减合法输入规模。

#### F20：ACE聚合包选择与当前成员不完全一致 — confirmed（选择差额）；candidate（治理修订，P2）

**证据。** `scripts/test-groups.toml`的sets.ace固定为ace-workspace/ace-adapters/ace-batch；ace-workspace与ace-performance的-p显式清单均只列当前35成员中的26项。遗漏的9项中，4个有自己的显式职责组：ace-a2x-c→ace-a2c-abi、ace-foreign-contract→ace-foreign-contract、ace-source-query→ace-semantics、ace-source-cte→ace-comptime；其余5个为no_std静态runtime，不能机械加--lib --tests。Cargo编依赖不执行依赖包自身测试。test_fixed_groups.py现有ace测试钉选择3个组，不对当前workspace各package职责做这项对账。

**结论。** `cargo t ace`不能据名称充所有当前成员独立测试的全集；但4个职责组存在，且M1签章可另行显式选它们，所以本差额不等于“这些测试从未运行”或“M1签章无效”。runtime有native/ABI conformance消费者，不能报无测试。静态选择差额已核，实际case执行数/平台排除未测。

**建议。** 测试框架更新时维护具名固定包/职责映射与聚合用途合同，决定ace聚合是否补4个包或明确所需并集；保持ignored/平台、正式性能和静态库构建专档，不重建动态全仓发现系统。本轮只记差额，不改组、不运行套件。关联MC01/17/23/24、MC32–36、1002/985/1017。

### 下一轮补齐清单（按模块闭包驱动）

这些是T-02/03的当前剩余工作，不是新实施任务或另一个计划状态机。每行关闭必须把具体证据和未核项回填对应MC，不以“无新发现”当通过。

| 次序 / 单元 | 必须补齐的可检查产物 | 关闭条件 |
|---|---|---|
| COV-A / MC25/28、MC05/36 | F16所有计数读点的预分配顺序表、F17租约状态表、F18/19源码身份/预算和父侧拒绝链 | 每个同类入口有合法/畸形/失败对照及静态结论；动态部分仍分账，任何真实探针须专属树 |
| COV-B / MC15/18/09/08/19 | 语言构造→profile→HIR/MIR→AV/AC操作表，逐列check/trap/resource/清理与source span | 各未覆盖构造有路径/owner和反例入口；不同verify边界不误删，不仅核F01–15 |
| COV-C / MC20–24、MC06/17 | 身份/正负依赖/能力→命中/失效/恢复→prepare/commit/cancel表 | 每种页/缓存/foreign form有生产者、当前授信与拒绝状态；CT调用递归与需求环分开 |
| COV-D / MC01–04、MC07、MC11/16/29–31 | emitter规则/target差异、CAS根/lease/GC、runtime导出→ABI→fault/cleanup矩阵 | 全部接口去向及必要差异可查，复制机制和独立oracle分清，测量收益仍不假定 |
| COV-E / MC10/12–14/26/27/32–36 | recipe/测试/固定组→上述不变量的职责表，补F20差额与条件 | 不同作者格式下case/check守恒，所有静态库具名测试消费者，覆盖率unknown有明确缺项而非虚称全审 |

### 第四轮分支深审（历史轮次，承接r2；当前闭合见第六轮）

用户本轮要求“完整分支深审”。本轮将COV-A..E展开为57个具名分支族（A10/B16/C13/D11/E07），实际追踪成功/拒绝/失败/恢复/取消/提交/清理；不把已有MC入口抽查当全分支已完成。每行S=已静态追踪、G=发现缺口、U=仍缺运行或内部子分支证据；**S/G不是测试PASS**。原MC剩余项、r2具体输入/职责/旧用例映射/V-DETAIL-01..04全部保留。

**来源对账。** 开始HEAD仍为f9fb3854，期间其他会话将主线推进至`7833ee6536bee222bee33b13e769f885dc64ff37`：ACE有34个路径变化，主要为性能分档，另有store目录计量/CTE观察改动。本轮新增F21–27所指7份ACE关键文件及两个A2X父adapter，在两版本间无差额。它们的结论仍绑定f9fb3854；store/CTE观察/测试组另核7833ee65，不拼成一个冻结运行结果。相关源码无WIP，本轮没有写代码。

计划文件也由其他会话从r1增为r2；首次写入的hash防护发现变化即未写入，重新读取r2具体输入/类型/职责/旧新映射后才追加本节，保留该版字段、任务和全部增补。r2新增的拟类型AuditFinding仍非实现API。

**全源分支导航。** 从f9fb3854 Git对象批量读取372个src Rust文件，274383行；词法出现if=9803、match=3726、问号=9015、unsafe=314，包含注释/字符串/inline测试，**不是CFG分支数，也不是人工全读或测试覆盖率**。作用仅是防止导航漏成员；人工结论以本节分支族/F项为准，不能将批量读入等同274383行已深审。

#### COV-A：解码、编辑、租约和父侧等待

路径相对`experimental/ace/crates/`，语言父侧另注明；行号绑定上述来源，后续执行需按符号/hash重定位。

| 分支 / MC | 正常、拒绝、失败/恢复分叉与证据 | 裁定、对照输入和测试边界 |
|---|---|---|
| A01 / MC28 | program_codec::Reader短头/错版本/错UTF/截断/尾字节；Atom tag0..2、Repeat0..4、Assoc0..1之外拒绝 | S：roundtrip、trailing在inline测试；非零bool/option现按真解释，不擅加新严格格式承诺 |
| A02 / MC28/25 | decode的全部12处with_capacity：mode_names:163/rules:168/tokens:185/grammar_rules:191/alts:196/steps:199/pratt:236/ops:240/field_names:283/stmt_end:289/sep_consume:294/sync:300 | G/F16：皆先按内部计数预留，后验元素/身份不挡大分配；必须统一预算而非补第一处 |
| A03 / MC28/27 | decoder核scanner token/mode和start；FirstSets核Tok/Rule/Pratt编号，但合法Pratt编号下的primary直接索引first/nullable | G/F21：仅1条rule时primary=1越界；合法primary=0为邻例，配置编译检查不代恢复结构准入 |
| A04 / MC25 | decode_tokens_v2先核n×25字节，decode_page先核8+n×32+e×8及manifest node_count，再预留并核edge | S：Windows x64下u32计数两路径不属F16短包巨量预留；保坏page/ref负控，32位乘法适用性不冒已验证 |
| A05 / MC25 | validate_manifest核page连续、decl page/node下界、origin/source、root edge、decl_count；decl_subtree进一步核可达结构 | S：decl_range单点检查不能代完整子树；目录分页/缺required/循环/跨度全组合仍U |
| A06 / MC28/26 | TreeSession当前/显式lease旧树成功；未知/跨session/未lease旧代/越界node拒绝；Drop与下次publish分别影响可读/回收 | S保代际；G/F17只指零计数索引，不是所有树泄漏；多lease释放一份与最后一份必须分测 |
| A07 / MC28/27/26 | incremental_lex/reparse空编辑/收敛复用；三种FallbackReason→显式fresh；strict投影拒error/missing，recovery须前进 | S：具名回退是合同但不充性能绿；fresh树/诊断/源码等价与预算的实际运行仍U |
| A08 / MC05 | source_text同份bytes hash；source_path读错/UTF错拒绝，二读变更/失败产生F18；stdin到EOF先整体分配 | G/F18/19：分清source身份与infra，不用空hash补错误，不把one-shot改line framing |
| A09 / MC36 | ac_a2r_hir/ac_a2c_hir：缺工具/child非零/response非ok→ERROR；再验protocol/recipe、spawn-zero、manifest hash和golden文件/路径集合 | S：infra不满足语言负例，golden不传child；manifest rehash不等于nonce/source全身份独立复核已做 |
| A10 / MC36 | 两daily adapter直接Command::output，返回后才finish exchange和判响应；dispatch_check同步调用 | G/F27：本地未见deadline/stream cap/kill-tree/reap，后置oracle无法处理永不返回；外层timeout不代局部ERROR收敛 |

#### COV-B：语法/类型/指令分支与会话事务

静态枚举提取并核消费者：源码AST 25个ExprKind/14个Stmt，HIR 17个TypeKind/42个ExprKind/9个Stmt，MIR 22个MirType/54个MirInstr/3个Terminator。**声明存在不等于profile支持**。11个SourceProfile不是全部单调超集：Core是旁路；Numeric虽有closures-superset旧注释，`docs/specs/ace-migration/m2-numbers.md`明确ctor-expr不在numeric原子面、聚合归962，故不把is_record未列Numeric直接报bug。

54个指令在下方前11行互斥完整归类；AVAD codec/MIR verifier/AV dispatch/InstructionSpec各能定位全部54个显式引用，**引用出现不证明每分支语义正确**。compact ABC只显式引用7种操作，属受限profile；native主lower与collections/closures专用lower按域准入，后两者显式引用28/16种并有unsupported拒绝，不能机械按数量判漏实现。

| 分支 / 操作家族 | 成功/错误/资源与清理分叉 | 结论与oracle义务 |
|---|---|---|
| B01 Const/Copy/Bin/Un/Conv（5） | eval_bin/un/num和conversion：checked整数、div0、MIN/-1、shift、float→int、同型条件 | S：DivOverflow/RemOverflow分态，混型防御Infra，不静默晋升；同型边界/NaN/-0/窄化用独立位/陷阱期望；numeric测试9个属性不冒运行9例 |
| B02 Call/Trace（2） | 先注册签名后lower、callee/arity/mode、frame/fuel/trace cap和返回清理 | S：trace需能力，递归资源态独立；VM显式帧不证明前端所有walk栈安全 |
| B03 GlobalGet/GlobalSet（2） | global型/当前宿主需求和会话状态，snapshot隔离 | S：状态不进纯常量缓存；权限/跨会话/资源子态仍U，不能以match引用充验证 |
| B04 ForeignCall/HostCall（2） | facts/plan/capability→目标准入；console输出cap；AV/native支持域不同 | S：公开ABI与私有载体不别名，恢复不授地址/权限；bool/char/float位与provider更换保反控 |
| B05 FieldGet/FieldSet/Aggregate/Clone/Move/View/MutRef/BorrowEnd/Drop/FieldMove（10） | 资源计划→init/alias/move/borrow→cleanup；root/field替换与借用结束 | S：非法静态用法、动态invalid handle、资源失败分开；self-alias/半构造/克隆/移动后读/借用冲突仍需逐case |
| B06 StrLen/StrLenBytes/StrCharAt/StrSlice/StrConcat/StrToOwned/StrCmp/CharToCodepoint/CharFromCodepoint（9） | 字节/码点、OOB/逆序/非法scalar、allocation/ledger/cancel和清理 | S：长度保真含NUL；F01是编辑保真缺口，不能由正常text range trap覆盖；各目标fault序列仍U |
| B07 ListNew/ListLen/ListCap/ListGet/ListPush/ListInsert/ListSet/ListRemove/ListFieldGet/ListFieldSet（10） | 元素/索引/容量、owned变更、generation和扩容失败 | S：native参数/返回域有限，不能由MIR有指令就放宽；保旧dst/旧元素及边界拒绝 |
| B08 VariantPack/VariantDisc/VariantGuard/VariantProject/VariantSwitch（5） | tag/字段型、guard→projection、branch trap及CFG合流 | S：typed例不代source模式；错误branch/移动后投影/合流init各独立 |
| B09 ListPlace/PlaceGet/PlaceSet（3） | list generation/索引/元素证据，mutation后stale，aggregates owned能力门 | S：StalePlace和ListIndex分态，临时alias不独立free；保能力边界 |
| B10 CreateClosure/CallClosure/EnvGet（3） | creation-site捕获/owned env/签名/拓扑及旧帧保活 | S：拓扑变化non-fast不冒局部生产；共享lowering不允许consumer各补捕获规则 |
| B11 DynPack/DynIs/DynTake（3） | 动态型节点/codec形状、is/type mismatch、take/move与节点预算 | S：DynTypeMismatch/DynTypeNodes不混语法/静态型拒绝；portable不恢复runtime handle |
| B12 Jump/Branch/Return（3终结符） | CFG目标、bool条件、返回型、唯一terminator、unreachable/init合流 | S：HIR/MIR/执行防御是不同授信边界；Success/Trap/Resource/Cancelled/Infra均有清理义务 |
| B13 / MC15/18 | source11 profile→parse/resolve或facts→Atom bind/remap→Checked HIR→shared MIR；非法entry/module/method/domain受控拒绝 | S：源码25+14变体resolve可定位；完整profile×构造×错误组合和全部HIR body内部仍U，不能以词出现冒全读 |
| B14 / MC08 | Function/Data/Owned/Collections/Closures prepare→verify→link/run→commit；closed/foreign/stale/cancel/gate | S：collections内层build-aside存在，F06在外层；owned的env_step.publish和closure外层front.commit位于部分接受态更新之后，是否可失败待前置证明，不直接判已触发 |
| B15 / MC19 | TypedSession edit_units先改doc再逐单元prepare，验证/emit/store任一步Err | G/F24：不是全candidate事务；不能用PE文件publish回滚测试替typed session状态保旧 |
| B16 / MC19 | collections PreparedEdit owner/revision/cancel/close；COFF fresh/reuse/read-only，恢复重验runtime/manifest/linker | S：hit不代run，候选拒绝应保旧文件；完整OS链接/发布失败的运行证据仍U |

#### COV-C：事实、依赖、CTE与FFI分支

| 分支 / MC | 成功/失效/拒绝/发布状态 | 裁定与验证去向 |
|---|---|---|
| C01 / MC20/21 | 逻辑id/方法owner、producer/schema/codec；solver Ok/Diags、Resource/Cancelled；正负依赖与来源位置 | S：页512KiB、vec/depth/尾字节门存在，预算/取消不缓存语言裁决；保sema身份/cache反控 |
| C02 / MC22 | Hit/HitStaleProvenance/Stale/Miss、kind/owner/target拒绝；互斥Disposition和IPC | S：内容复用不等于坐标新鲜，IO分类沿F10；资源/取消不冒语言错误 |
| C03 / MC23 | key含owner/sig/body/caps；同key还核positive signature与negative candidate-set；solver禁用miss拒绝 | S：sq06缺helper→新增helper必须miss；同callee body改不该让仅签名依赖失效 |
| C04 / MC23 | QueryHost prepare不安装；commit核cancel/base/key/positive/sink后换record/target | G/F22：cache-hit查negative，commit没查；sq27正依赖变化不覆盖负依赖翻转 |
| C05 / MC23 | restore_verify缺页/错owner/name/body/shape/coverage/envelope拒；restore_bind各函数需页，无silent solver | S：页完整性与当前foreign/caps分层；restore_identity/architecture_facts为导航 |
| C06 / MC23 | InstanceHost prepare_set；commit重实例化/比较set_fp，cancel/late/changed/sink Err；成功最后换集合 | S：current保旧不等于无中途soft-root/produced缓存；需区分逻辑安装与可回收副产物 |
| C07 / MC23 | InstancePage magic/version/envelope/字符串截断拒，但ty原字节+demands后直接返回 | G/F23：无总size/尾字节/合法tag门，不能以注释“decode即验证”替实现 |
| C08 / MC24 | CT hit核expect不触AV，bad env拒，miss真实执行，forbid/cancel/wall分态 | S：缓存值不是cached PASS，当前capability在product先验，不把key自报caps当授权 |
| C09 / MC24 | 普通递归≠CT环；root single-flight、累计10s、per-call min(2s,remaining)，nested/meta阶段规则 | S：每调用不重置root预算；全部nested source/恢复/取消组合仍U |
| C10 / MC24 | call/expr实际成功且ct_execute++后publish失败 | G/F26：返回av_reached=false和真实事实矛盾；仍是freeze错误，不报语言假PASS |
| C11 / MC12/13/14 | 817 Literal/QueryRef/Call/Meta、重复/前向/自引用/节点/深度/record仅meta拒绝；回灌type/placeholder；Closure/Snapshot分key | S：裁剪失败回FULL源不缩义务，ordinary CT与旧recipe不直接互换；保累计预算和独立错误类 |
| C12 / MC17/06 | strict codec先1MiB/错版/未知字段；Pure/Fallible/context，公开lane/位置制寄存器，bool/char/float位保真 | S：公开无f32，source_ok比typed窄，私有ABI非public别名；plan存在不等于执行授权 |
| C13 / MC06 | source/foreign candidate owner/base/validate→commit，rebind_provider先admit；恢复先root hash/budget再current环境/target/cap/代际 | S：恢复不恢复地址/授权；所有resource/foreign form的真实机器负控仍U |

#### COV-D：A2X、CAS及runtime失败与清理

| 分支 / MC | 已追踪成功/拒绝/失败/清理 | 裁定及需保差异 |
|---|---|---|
| D01 / MC01/02/03 | SemanticView凭证/plan深度→provider family/target→runtime/fragment/fileset；foreign C支持/Rust拒绝 | S：共享语义与目标键分层，C/Rust ABI/printer差异保留，不能为复用整套合并 |
| D02 / MC04 | resident hit、store有效UTF hit、坏缓存/miss→produce；read_only不put；range核base/UTF/单声明/name | S：read_only不等于不改session；显式恢复env与source身份不同轴 |
| D03 / MC04 | resident先换再assemble；ranged临时SHA，后续origin/anchor publish仍fallible | G/F25：只恢复SHA不代完整事务；晚失败不被parse早拒绝用例覆盖 |
| D04 / MC07 | put溢出/staging/namespace预算、锁忙、幂等/冲突、required死亡、SQL/FS失败→事务提交；cleanup忽略返回 | S：同key异值不覆盖；失败可能留可回收残留，不声称磁盘零副作用；锁序liveness→budget→namespace→chunk→SQL |
| D05 / MC07 | 无row/tombstone/missing为miss；hash/frame/kind坏为error；bounded cap、root替换、lease获取/失效/drop和读锁 | S：损坏/IO不全部当miss；新lease/GC互锁，跨进程竞态/OS拒删运行仍U |
| D06 / MC07 | GC below-high、exclusive live闭包、floor backoff、orphan/tombstone/staging/trash/reparse处理 | S：不为low删活数据；7833目录计量从is_reparse改file_type.is_symlink，Windows类型等价须核，不能按注释下安全结论 |
| D07 / MC11 | arena checked size/align/limit、free-list拆/合、VirtualAlloc失败、release非法/双free、尾部VirtualFree失败 | S：live=0不保证OS committed=0；OS释放失败计invalid，不能用allocator计数冒总内存收敛 |
| D08 / MC11 | frame_open预检/hold_external/allocate/enter失败按序回收；cell自move/share no-op，literal失败释放node并分cancel/resource | S：扩容/半构造须保旧dst；F04共享算术不得移除这些guard/cleanup |
| D09 / MC16/29 | ledger先验再free、负length/UTF/scalar/range、Nth alloc/ledger fail；text hooks/copy_poll/metadata release | S：text取消和provider seam不同于data探针；每导出/OS free失败原子性及ABI实测仍U |
| D10 / MC31 | header IO/magic/version/reserved/checked cap/hash→anchor→sig/argc/len；4种互斥response，write失败exit1 | S：one-shot pointer合同，不推广成并发/重入；半帧/尾输入父子适用性仍待核 |
| D11 / MC30 | mark_a/b stderr字节/返回值；WriteFile结果忽略，panic spin；父字节oracle | S：测试最小trace非通用I/O，缺字节会红但不证明写入成功，不与runtime整库合并 |

#### COV-E：分派、父裁判、测试与固定入口

| 分支 / MC | 真实分派/判定链 | 静态结论与条件 |
|---|---|---|
| E01 / MC10/36 | WorkerRequest schema/kind/profile/budget/path/lib/input/edits拒绝；平台/每请求backend retired-invalid；check/build/run/product/syntax | S：拒绝前置，父response/nonce/hash/cap、manifest、stream各一层；support路径不是授权 |
| E02 / MC32/36 | dispatch_check按adapter/action/注册recipe分派到ac/AV/CTE/ABI/MN/A2X/syntax，其他组合拒绝 | S：工具身份/child成功不代父oracle，新格式降到同一IR，禁止第二runner |
| E03 / MC32 | host/batch/console/governance/native_lifecycle显式mod，feature/target/tool身份与ignore/平台条件 | S：Cargo依赖不自动扩测试；父gate终态与内部每check退出分账 |
| E04 / MC33 | stdlib reference/requirements/同名shadow/schema/hash篡改及真实consumer结果 | S：引用进manifest不代callable stdlib，不解除945–948 tag前门 |
| E05 / MC34/35 | ABI/runtime contract/smoke/ops/lifecycle/reject/fault/trap/scale，机器调用/字节/ledger多oracle | S：静态库无普通Rust依赖不等于无人测；原规模/故障序号/条件和正式性能分档保留 |
| E06 / MC01/17/23/24/32–36 | 7833普通ace-workspace 26包，另4职责组；ace-perf 27包且串行，regular排除(^或::)performance:: | S修正F20历史：不是所有ACE独立测试全集；性能从regular排除不等于义务删除 |
| E07 / 全MC | test源码/声明/断言/固定组导航不是实际测试发现或运行覆盖 | U：本轮无nextest发现/全case-check迁移总表/执行覆盖，禁止“所有分支已测”或“全源码已逐行读完” |

#### 分支补核：提交前提、profile门与runtime导出

**B14/B16提交前提。** `core/src/source/adapter.rs::FnCandidate::{validate,validate_base,commit}`的拒绝全集是closed、owner、generation、slot、foreign context/slots；prepare成功和emit失败均将临时graft换回。native collections在提交前显式validate，之后独占env，runtime/link/run不修改env，故“最后commit带Result”本身不是可触发晚失败的证明。AV owned候选由本会话私有同步生成，闭合env未向外借出，候选间无generation/foreign/slot发布；closures的commit_candidate仅有env_id/base_generation两个Err守卫，apply_fast中的update只持另一session。当前正常Result控制流未找到这些前提变化路径，**不将两个迟置commit新增为原子性缺陷**；panic/进程崩溃恢复与可注入外部故障须另核，不能与F24/25确定的fallible晚失败混列。

**B13 profile准入真值（`core/src/source/mod.rs`，1表示对应谓词为真）。** 这是门的实际实现，不是完整语言支持表；构造的具体位置/类型/能力还需后续层校验。尤其is_scalar不含Record而surface_is_scalar_or_wider包含Record，二者职责不同，不能为统一代码机械合并。Numeric/Aggregates的entry谓词为0也不能仅凭“superset”注释判bug或自行扩承诺。

| profile | scalar | scalar-or-wider | text | record | entry | collections | closures | numeric | aggregates |
|---|---|---|---|---|---|---|---|---|---|
| Core | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Scalar | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| Text | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 0 |
| Record | 0 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |
| TextRecord | 1 | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 |
| EntryUnit | 1 | 1 | 0 | 0 | 1 | 0 | 0 | 0 | 0 |
| HostConsole | 1 | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 |
| Collections | 1 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 0 |
| Closures | 1 | 1 | 0 | 1 | 1 | 1 | 1 | 0 | 0 |
| Numeric | 1 | 1 | 0 | 0 | 0 | 1 | 1 | 1 | 0 |
| Aggregates | 1 | 1 | 0 | 1 | 0 | 1 | 1 | 1 | 1 |

进一步读`resolve_body/expr`：GridLit/ArrayLit仅初始化位置降糖，其他位置具名拒绝；Index必须i32且List/owned elem域有别；IndexAssign核local/List/元素型、owned move及iterate-mutate；LetDestruct核Tuple/元数/重复绑定，普通值与已有local采用不同投影方式。这些都不能由“AST存在变体”代替位置/资源语义。尚未逐一闭合全部39个AST变体与11个profile的合法/错误组合，保B13-U。

**D09导出守恒。** 按两个lib.rs的显式`pub extern`/`pub unsafe extern`声明分组，data共39、text共30；含4个CRT支持符号，不冒最终链接符号集。每组均保独立消费/拒绝/清理职责，不能仅因同源把probe并入产品runtime。

| 导出族 | data / text数量 | 分支与消费者义务 |
|---|---|---|
| memcpy/memmove/memset/memcmp | 4 / 4 | C指针/长度和重叠合同；不是接受任意无效指针的安全API |
| allocator install/metadata/release | 0 / 3 | alloc/free必须成对，live或保留ledger拒重装；metadata free失败保指针，不报释放成功 |
| 寄存器/borrow/record/handle探针 | 14 / 0 | mix/many 2、borrow 2、record 5、handle 5；ABI位置、借用与owned/handle代际不能互代 |
| make/free/clone/concat/from_lit | 5 / 5 | 负长/零长/溢出/预算/Nth alloc/ledger失败；失败输出与原owner保全、null free |
| text/char/length与eq/ne | 8 / 10 | byte/Unicode scalar、UTF-8、index/range、text分块copy/poll回滚；eq/ne仅text多出 |
| budget/fault/error/summary/invariant/trap/hang | 8 / 8 | fault序号、invalid_frees、live/peak与父侧终态分别核；hang必须父进程收敛 |

`owned_free`两版均先ledger_take，再调用OS/provider free；free返回0后invalid_frees增加、live/live_bytes不减，且该ledger项已移除。故现有实现能报异常，但没有“失败后仍可重试同指针并恢复完整ledger”的证明；不把它报成无声释放成功或未经合同核对直接要求重试。text私有FreeHook可让该分支可控，data原HeapFree失败与text provider故障必须分开验证；D09-U保留具名未闭合项。`ac_rt_release_idle_metadata`失败分支保旧ledger指针，与owned_free不可混为同一行为。

**D05 lease补核。** acquire先持唯一lease文件锁，在shared liveness内核target仍live再写row；无target lease也受同一锁区保护。lease_live缺row/released/可取得锁为false，仍持锁即便过TTL为true，锁IO错误不伪装false；sweep只释放可证明已死者。Lease::release写row失败虽被忽略，self消费后OS guard仍drop，后续lease_live/sweep可识别死亡，不据row尚未改直接报永久活租约。F17的内存handle索引与本CAS lease是不同实现。真实跨进程/异常终止/拒删仍未执行。

### 第四轮新增发现F21–27

#### F21：恢复Pratt.primary缺少范围检查 — confirmed，P1

**证据。** `syntax-runtime/src/program_codec.rs:318–331`只核scanner/start；`parse/mod.rs:180–197`只核Pratt编号就把primary用于first[prim]/nullable[prim]。公开FirstSets::compute返回Result，却存在未受控索引路径。

**结论。** 内存按v3布局构造117字节：1 token/1 rule，首步Pratt(0)，1张Pratt表primary=1，start=0；SHA256=`83fecb67ea320eef2abe1ec5dd15bccfe5fd2b2479a9ba7209a8c72436539d8d`。源码控制流会到first[1][0]；只构造字节/推理，未调用Rust parser或实际panic，不报远端利用。CAS随机坏字节拒绝不能代公开decoder语义准入。

**建议。** 恢复执行前完整核Pratt primary/op、step、field、newline/recovery等引用；合法primary=0、错primary、错表编号、不可达坏引用分测，不能用默认值吞错误。owner=syntax-runtime/grammar恢复，MC27/28。

#### F22：QueryHost提交没重验负依赖 — confirmed，P1

**证据。** session.rs:338–357的body_key只有owner/sig/body/caps；page_deps_fresh:723–740还查negative候选集。host.rs:222–248只保存positive sig_reads，commit:260–319核cancel/base/key/positive/sink，无negative；MemorySink.put不补此验证。sq06测缓存负依赖，sq27只测提交正依赖。

**结论。** 沿sq06真实源，prepare缺helper的main事实后新增helper，main签名/body、host base与旧正依赖均可不变，现有commit guard全过，安装的诊断/事实可能已过期。只读数据流/抽象guard重放，未跑Rust；不声称错误target已执行。QueryHost直接使用主要在本库测试，产品面待核；F12的完整重验正面表述必须限定。

**建议。** candidate携完整读集/候选集版本，commit复用依赖freshness规则；不用全模块hash破坏局部性。负页→新增/歧义翻转拒提交，无关声明/同callee body变化作合法对照。owner=source-query host，MC23。

#### F23：实例页decoder不是完整严格reader — confirmed，P2

**证据。** instance_host.rs有512KiB常量，encoder检查输出；decode:163–223不先核总长，ty取任意u8，demands后不验EOF即Ok。From<InstanceRecord>只产ty0/1/2。get_str会先验剩余字节，不能误报F16预留问题。

**结论。** 159字节按该布局的页加1字节仍可同路返回，ty255无显式拒绝，超限尾部亦无总长门。仅小字节/控制流证明，未跑Rust；上层页等价/实例集重推导可挡部分坏值，不报告绕过执行授权。现codec测试覆盖magic/version/envelope/truncated，未见该入口尾字节/tag断言。

**建议。** 输入前置size、tag合法性、完整消费和checked offset；合法三tag、截断/错tag/suffix/上限邻例分测，不重新encode丢尾部当验证。owner=source-query实例页，MC23。

#### F24：typed native批量编辑不保失败原子性 — confirmed，P1

**证据。** typed_session.rs:164–238先写全部replacement到self.doc，再prepare_one。后者build_verified_unit/admit/emit/store均可Err，成功单元即replace_resident/history/generation；无全候选/rollback。typed_fixture.rs:769确为单元build/verify，非一次全doc校验。现incremental tier三项测试主要成功面。

**结论。** 单项非法替换即可doc=B而resident=A；多项可先安装前项，再因后项失败返回Err。与Atomic multi-unit edit注释冲突。未实际Rust/PE重放；文件publish已有transaction测试不能替会话状态测试。范围限定typed驻留入口。

**建议。** doc/units/history/generation整体build-aside，全部成功再swap，尝试观察与接受态分开。首/中项verify/emit/store失败、同批重试/后续run、旧entry_plan/COFF身份为永久反例。owner=native typed session，MC19。

#### F25：A2X暖编辑晚失败未恢复全部状态 — confirmed，P1

**证据。** a2x-store/lib.rs:2348–2406先retain/extend resident再fallible assemble；ranged:1665–1680失败只恢复source_sha，后续origin/anchor publish:1720–1779仍可Err，此时新SHA/resident已变而anchor可能旧。早parse拒绝不是晚publish拒绝。

**结论。** source/resident/anchor三轴缺少一个提交点；公开TargetProvider返回Result与anchor IO/预算失败是真实故障点，未执行故障注入。read_only不写store也不意味着session不变；不把任何副产物入CAS都误称接受态提交。

**建议。** fileset/anchor/root/resident/source统一prepare→validate→publish→swap；失败保旧，允许可回收无根副产物。用错误provider/anchor publish失败负控，不只改语法造早拒绝。owner=A2X session，MC04。

#### F26：CT发布失败返回了错误的AV到达事实 — confirmed，P2

**证据。** source-cte/service.rs:234–265与383–414分别在call/expr AV成功、ct_execute++/日志已写后，sink.publish Err却返回av_reached=false；executor对该字段定义为实际尝试是否达到AV。service在两基线间未改。

**结论。** 返回错误事实与真实计数矛盾，影响归因，不是语言假PASS或错误值成功发布。本轮未跑sink失败。

**建议。** freeze失败code和av_reached=true分开；call/expr的publish失败、执行前拒绝、trap/cancel邻例同时核日志/计数/字段，不能为补事实重跑AV。owner=source-cte，MC24。

#### F27：A2X日常父adapter缺少局部有界执行 — confirmed（本地路径），P1待集成核验

**证据。** 语言ac_a2r_hir.rs:202–208、ac_a2c_hir.rs:234–240直接Command::output，dispatch同步调用；无该层deadline/输出cap/kill-tree/reap。WorkerExchange在返回后记录，不是执行控制器。已有auto-tool-execution和ac worker_session可复用。

**结论。** child挂起/无界输出时后置protocol/manifest/golden检查无法提供受控终态。上层超时可能终止整个测试，不等于case内清理；实际Job继承/孤儿进程后果未运行核验，不称必然。F19管读入、F27管父等待/输出，不能互代。

**建议。** 沿公共owned executor接deadline/caps/进程树/reader终态，保单次仅spawn预构建worker、不建rustc/cargo。挂起、双流超量、子孙持管道、IO错及正常对照分别测，不能只timeout不reap。owner=语言A2X adapter，MC36。

### 第五轮内部条件与测试义务补审（2026-11-13，历史轮次）

**来源。** 本轮固定Git对象`42874419aa1054bd6a97cfce40826dc9f68103c9`。相对7833ee65，相关范围只有7个跟踪路径变化：ac_mn接收aggregates诊断profile、syntax-grammar profile测试接收aggregates，以及ABI/AV/source-cte的既有测试调整；test-groups和下列新增发现生产者未变。逐项核过diff，不把接收侧已修的profile漏项再次报成当前问题。ABI/AV/source-cte部分多样本原命名oracle从performance回到日常，有1025归档T-04/F-1025-06的明确有界接收及canonical Spec背书，不仅据P90/耗时重报违反分档；原正式campaign仍归ace-perf。计划从主线已跟踪版本读回，保留r2合同与前四轮记录；无源码WIP、无实现/构建/运行。

**正式M1来源更新。** 本轮已出现可读正式收据：`archive/1025-ace-m1-mainline-identity-and-final-signoff.md` T-05..08/§9及`docs/specs/auto-test-case/ace-m1-expanded-acceptance.md`的1025节记录H=`40b51f809d5be10dddf8a8c56891b0d62527faf8`、落地C=`b923362f47bc8861407e9a470d63ddfdb505982b`，C正式读回exit0、1006P/0F/0U、tag_ready=true。`git diff --quiet C 42874419 -- experimental/ace crates/auto-lang-core/src/test_cases scripts/test-groups.toml`退出0，故本次审计源码范围与该C相同；收据的原件已由1025清理，本轮没有重新运行/重新准入其终验。`git tag --list '*ace*'`本地仍无匹配，1025明确实际创建/推送另需授权；**已知正式验收C不等于tag已存在**。前几轮“正式来源unknown”保留为当时历史，当前不再把整个M1来源列unknown，也不创建tag。

#### B13：全部AST变体分派与内部条件

本轮分母是`source/ast.rs::ExprKind`的25个变体与`Stmt`的14个变体，按variant集合逐项对账；下表合并同行变体，既不重复计数，也不把profile声明当支持证明。主定位：`resolve.rs::expr:7291`、`expr_iterative:8346`、`resolve_body:5703`，随后`typecheck::{check_block,check_expr,check_arg_modes}`、`adapter::plan_expr/plan_block`和共享`lowering::hir_adapter`。这是分派/条件追踪，**不是25×11或14×11的全组合执行证据**。

| 表达式变体（合计25） | 已核内部条件与下游义务 | 现有静态验证入口 / 剩余 |
|---|---|---|
| IntLit / TypedLit / BoolLit | i32、字宽/位型、bool分别node_ty；TypedLit先投影验字面量范围，typecheck精确同型，不隐式转换 | numeric_source/numeric_hir、scalar_source；负unsigned/字宽/溢出保持各层拒绝 |
| Var | local→view/闭包移动/函数回调/全局等按上下文；undefined保诊断，不把usize::MAX哨兵送进成功发射 | source_frontend、closures_surface；错误结果必须在emit前拒绝 |
| StrLit / CharLit | 字符串受admits_text_literals；CT selector字面量在meta入口截取，散落scalar字符串拒；char保scalar值 | text_source/text_binding/source_ct_936；byte/scalar/NUL及能力分开 |
| TypeWordArg | 仅.to与类型级方法消费；游离标记由typecheck拒绝 | text_source/numeric_source；不可当普通值 |
| TupleLit | 元素走iterative，VT序列intern shape，adapter按位置物化；参数view与owned临时不同 | aggregates_t01_probe、aggregates_v_rows；深层资源型仍按verify/cleanup |
| GridLit / ArrayLit | 仅Let初始化降糖；grid列不能为空、行宽一致、元素型匹配；array空数组无推断拒绝；其他位置具名拒绝 | aggregates_t01_probe、v05_grid_column_ledger_closes；正常资源账不是故障注入 |
| Index | i32索引、List基底；Copy元素ListGet，owned元素ElemView，冻结域不许whole view逃逸 | aggregates_t01_probe、collections_rejects；不能把类型通过当index运行时OOB通过 |
| Range | 值位拒绝；仅ForIn消费为i32界，半开/闭区间两种条件 | range_for_in_counts_and_executes；新增F28/29 |
| HashExpr | ct_enabled门→inner绑定→未服务需求typecheck拒绝；生产CT须先发现/服务/结构替换再facts绑定 | source_ct_936、source-cte tests；新增F32，失败不能静默转运行时 |
| GenericType | 仅receiver位置路由new/variant等；游离类型不是值 | collections/source_instances_935；模板TyVar不当已具体化凭证 |
| Closure | resolve_closure捕获顺序、move/borrow、签名、逃逸及creation-site，之后凭证和env由共享lowering消费 | closures_surface/closures_fn_912/closures_context_913；scope不是复制所有session |
| CallValue | expr_iterative先callee后实参，再CallValueOut验Function/签名；leaf expr拒绝是防御分叉 | closures_surface/av_closures_source；不因leaf拒绝就误报间接调用全不支持 |
| Call | iterative实参按源码顺序保存names，再映射形参；命名参数不是文本调序；foreign/meta/普通函数各自准入 | source_frontend::named_args_preserve_source_order；CT的同类walk遗漏见F32 |
| Method / MethodCall | receiver后args，文本/record/静态/列表按真实VT分类；类型级char处理shadow，owned get.field有融合前置路径 | text_record_source/collections_rejects/numeric_source；融合遗漏Named检查见F33 |
| Binary / Unary | iterative子树；精确型/运算矩阵；logical保短路，不能把未取RHS变eager；trap与静态拒绝分离 | scalar_source/numeric_source/av_numeric_semantics；边界与跨目标观察保持 |
| CtorNamed / Field | ctor按源码值求值再映射声明字段，核字段存在/重复/缺项/型；field按record链或owned view投影 | record_source/text_record_source、named_ctor_out_of_order_constructs；F33是融合特例 |
| ViewArg / MutArg | lookup须record local；view走readonly默认借用、mut标记BorrowMut；check_arg_modes双向核形参模式，HIR再核alias | record_source/text_record_source；显式标记不是绕开可变/借用验证 |

| 语句变体（合计14） | 已核条件、控制流和拒绝职责 | 证据 / 裁定 |
|---|---|---|
| Let | duplicate、annotation同型、字面量owned提升、grid/array初始化、borrow closure受限local及cell标记 | resolve:6068–6525、typecheck Let；后续init/alias独立verify |
| Assign | view escape、closure moved/borrow escape、字段路径/最终型；immutable由共享verify拒绝 | resolve:6526–6659；不能把本层不查immutable报成完全无检查 |
| IndexAssign | lookup/List/i32/元素型、owned存入move和迭代修改拒绝 | resolve:5984–6067；RHS先求值/旧目标失败保全仍归资源测试 |
| LetDestruct | Tuple/arity/重复绑定；已有local直接TupleGet，其余合成临时 | resolve:5887–5983；保持view不物化为owner |
| Hold | lookup须record；alias作用域进入/退出，不新造owner；当前合同只承诺whole-local | resolve:5839–5886；不擅把旧MVP扩成完整borrow checker |
| Return | fn_result期望、literal retag、禁止whole-view和borrow closure逃逸 | resolve:6660–6680、typecheck Return；资源退出LIFO另核 |
| ExprStmt | script尾值/Unit effect/ListRemove消费各归属；不得任意丢值 | resolve:6681、entry矩阵、adapter:1980附近；Ast旧注释非当前全合同 |
| If | cond必须bool；then/else独立scope，结构frame合流 | resolve:6689、typecheck If；CFG/init/resource合流不可因复用省略 |
| While | cond bool、loop target、退出/继续及scope | resolve:6711、adapter WhileArm；与for-in合成更新语义不同 |
| Loop | loop target、body、header/exit映射 | resolve:6733、hir_adapter:6132；无隐藏的for迭代latch |
| Break | 最近loop target；无loop由phantom目标使verify受控拒绝 | resolve:6751、adapter phantom_target、HIR→MIR exit |
| Continue | 最近loop target→MIR header；不经过当前body尾部 | resolve:6762、hir_adapter:6169；证实F28，不能用While的正确性代证ForIn |
| IsMatch | variant local、qualified arm、unknown/重复/穷尽、payload绑定隔离；每arm guard后才project | resolve:6779–6951；filler只在静态穷尽保证下不可达，不能当无条件default语义 |
| ForIn | range/List分别路由；界型/List local、Copy绑定/owned view、禁止结构修改；隐藏index在body尾递增 | resolve:6952–7288；F28/29，list与range共因但合法邻例不同 |

#### A05/C09/C13/B03：剩余链路的定向闭合

- **A05：** 重读validate_manifest→decl_entry→decl_range/decl_subtree→page。内联目录核page/node/node_lo/origin；子树核同decl槽、kind/field、child范围、trivia右界与重复child/环，再remap。分页目录只核frame kind/JSON/条目存在，未复用内联条目准入，F31确认此差异；不再笼统写“目录尚未检查”。`syntax_admission`现反例构造器皆`decl_index: []`，已有子树反例不能替分页反例。
- **C09：** serve_site同owner+span查环（普通函数递归不当环），栈Result返回后pop；nested→callee closure→本地meta→substitute→真实service，literal/meta-only不冒AV计数，root wall按remaining收紧。公开scalar-CT产品的Named实参是当前可达遗漏，F32；Hold/tuple等扩域walker是否完整单列未来扩面风险，不从scalar-CT里不存在的构造直接报产品bug。普通Err后栈回收已静态核，panic/unwind、真实超时仍未运行。
- **C13：** source_foreign validate核owner/base及front候选，provider先admit/资源准入再rebind；restore先root限长/hash/schema/target/current contract/provider/generation/runtime，再required逐页hash/累计预算、当前HostAuthority重验、Frontend恢复。source_publication最终pointer lock/expected identity、cancel、current authority重复核，先register候选root→atomic_replace后才安装，切换前错误清理临时/root。切换后的旧根retire失败记录PublicationMaintenance，避免把已接受态冒普通拒绝；这与F24/25不同，保现机制。
- **B03/B04：** execute_module_impl_with_host无显式console provider时host imports拒绝，non-executable unit拒执行；HostCall核声明/签名、frame/cancel、每call/累计字节、slot/fuel/wall再write。全局初始化按generation，失败reset cells并cleanup frames，成功才mark；GlobalGet missing/GlobalSet host写失败是Infra，不能统称语言trap。程序运行中的global效果不因一次Resource就被推定为整次run事务，未从无该承诺处报新回滚bug。
- **D05：** 重读get_artifact/read_chunk_gated及bounded两入口：缺row/deleted/缺blob为miss，tombstone到artifact层转miss，坏state/kind/frame/address/IO仍是错误。bounded先实文件metadata限长，按real_len一次分配/read_exact/hash，不依赖DB自报长度。bounded在open前已drop gate，不能声称整个读取都持共享gate；并发删文件可变IO，不在未运行竞态前报告所有读者互锁已认证。普通read明确无该cap，不能把bounded的保证推广给所有消费者。
- **D07：** 全读collections-runtime::arena的allocate/release/hold_external/tighten_limit：checked size+alignment、固定32MiB ceiling、free-list split/前后合并/尾回退、OS reserve/commit失败不增live；release先核null/范围/对齐/块形状/available，更新live后VirtualFree失败记invalid并保committed事实。physical::finish独立报告OS指标/健康，arena live=0不能充OS提交为0；原API是私有单线程unsafe指针合同，不擅承诺任意伪造内指针安全。真OS失败/进程峰值尚未执行。
- **D09：** 再核text allocator/free/poll配对、live/ledger非空拒重装，release_idle_metadata只在无live时释放，失败保旧指针；owned_free失败丢ledger项与该路径不同，仍按§4轮保具名待议，不把invalid计数存在当可重试恢复证明。取消copy回收新buffer，合法原owner不得被当成新buffer释放；ABI/故障序列运行证据仍未取得。
- **D10：** 全读typed-entry-runtime的read_exact/write_all、read/check/response：头magic/version/reserved→checked total/256KiB/caller cap→精确payload+checksum→anchor，wrapper再核sig/argc/exact expected length；44字节response四互斥status，写失败ExitProcess(1)。请求按头声明长度读，不在helper等EOF；响应decoder另拒尾字节（typed_entry_tier:443–445），不能把response严格性误当request EOF承诺，也不以one-shot私有合同要求并发/重入。坏输入构造/父侧超时真实运行仍未做。

#### E07：成员→test target→职责组及新旧义务

重新解析35份Cargo.toml及固定组，**20个包显式声明的25个test target路径全部存在**；不含有名test表的包仍可能有src内联单测/自动目标，no_std支持静态库也不机械加普通测试。下表列所有成员测试职责（MC路径身份与§5原表对应），不能把依赖编译当包自身测试执行。

| 成员 | 已有显式test target / 运行职责组（均未运行） |
|---|---|
| MC01 a2x-c | 自动/src测试面；ace-a2c-abi |
| MC02 a2x-core、MC03 a2x-rust、MC04 a2x-store | 自动/src测试面；ace-a2x |
| MC05 a2x-worker | conformance；ace-a2x |
| MC06 abi | conformance、source_resources；ace-abi |
| MC07 artifacts | conformance、store_resources；ace-artifacts |
| MC08 av | conformance；ace-av（batch feature由正式组保留） |
| MC09 av-core | conformance、foreign_resources；ace-av |
| MC10 case-worker | conformance；ace-native-worker |
| MC11 collections-runtime | 支持静态库；native collections_resources_tier/fixtures消费，不伪造本包单测PASS |
| MC12 comptime | conformance；ace-comptime |
| MC13 comptime-contract、MC14 comptime-core | src/合同职责；ace-comptime，不因词法0属性报无人验证 |
| MC15 core | conformance；ace-source，source-syntax-927为具名子面 |
| MC16 data-runtime | 支持静态库；MC34 data-conformance ABI/fault/lifecycle消费 |
| MC17 foreign-contract | conformance及src；ace-foreign-contract |
| MC18 lowering | conformance；ace-lowering与实际消费者共同保共享语义 |
| MC19 native | conformance、typed_entry_tier、typed_entry_incremental_tier；ace-native |
| MC20 semantics | src自动测试面；ace-semantics |
| MC21 semantics-contract、MC22 semantics-store | conformance；ace-semantics |
| MC23 source-query | src及自动tests目标；ace-semantics |
| MC24 source-cte | src自动测试面；ace-comptime |
| MC25 syntax-artifacts、MC26 syntax-core、MC27 syntax-grammar | conformance；ace-syntax |
| MC28 syntax-runtime | src自动测试面；ace-syntax |
| MC29 text-runtime | 支持静态库；MC35 text-conformance及native资源链消费 |
| MC30 trace-runtime | 支持静态库；core source-frontends的真实机器ABI/字节oracle消费 |
| MC31 typed-entry-runtime | 支持静态库；native typed_entry_tier/typed_entry_incremental_tier消费 |
| MC32 conformance | conformance；ace-host，ace-console子面，ace-batch按feature选择，rust-lowering子面保现表达式 |
| MC33 stdlib | conformance；ace-stdlib |
| MC34 data-runtime测试包、MC35 text-runtime测试包 | conformance；ace-runtime |
| MC36 语言父侧 | auto-lang-core lib；ace-adapters；只在显式ACE组被选，不把普通语言组扩成ACE构建 |

静态定位：35成员的tests树共365份Rust文件，词法`#[test]`/带路径test属性共2728处，位置清单按`路径:行`排序/LF连接SHA256=`dabd4ea6c8b0cab794980f5ec5616ea293a671875be7c232450657cc9028eaf5`。**含辅助文件/fixture/字符串/宏与cfg，非编译后用例或执行数量，不含所有src内联测试，不称完整测试总数。** 另外直接核平铺root：core 40/40、av 92/92、native 38/38、syntax-artifacts 4/4的cases/*.rs均被本包conformance的显式path引用；不报告这四处存在孤儿文件，也不把它推广成全仓模块可达性证明。

| 发现/义务 | 已有入口/合法邻例与实际缺口 | 格式更新/后续修复必须保留的独立判据 |
|---|---|---|
| F01/F02/F06 | native closures_native_tier、core closures_fn_912、AV av_collections_session；分别是正常编辑、异类观察嵌套和成功session邻例 | UTF-8重建=请求源；同类嵌套/unwind后数据；失败后同请求重试必须生产并接受，不能以相邻成功例充覆盖 |
| F03/F08 | numeric_source/profile isolation、各profile source/HIR、architecture_facts_941 | 支持域/诊断/事实权威分别判；message变化不暗改调度，不去掉诊断oracle和独立verify |
| F04/F05/F07/F09/F11/F12/F13/F15 | 现MC对应语义/ABI/session/restore/consumer测试，既有provider/executor/sink不迁成第二实现 | 代码移动/机制抽取保独立oracle和原入口；编译/性能收益无实测仍candidate；注释更新不用新产品case造分母 |
| F10/F18/F19/F27 | case/sema/A2X worker及父adapter现protocol/nonce/manifest/golden邻例 | 原始字节/源码单次身份、IO/EOF分类、cap/deadline/reader/进程树终态各自判；infra不充语言负例 |
| F14/F20/F30 | 固定组及根模块、1002作者格式、V03实测行为重新核对 | 原case/check逐行守恒、原平台/ignore/性能档保留；正常构造资源账不能关闭中途失败/旧目标保全 |
| F16/F21/F31 | program_codec_roundtrip、syntax_admission内联子树反例、现正常分页生产 | 12个计数门/Pratt primary/分页目录准入各独立；小畸形bytes与合法邻例，不用decoder前置配置编译代恢复验证 |
| F17 | syntax handle/session现lease/代际拒绝入口；与CAS文件锁lease分开 | 活代可读、最后lease释放后的树/索引回收分别观察，不能仅看Drop无panic |
| F22/F23 | source-query sq06负依赖cache、sq27提交正依赖；instance_host_935::page_codec_roundtrip_and_envelope_refusals | 新增callee/歧义翻转在commit拒绝；实例页size/tag/EOF独立邻例，上层拒绝不充decoder拒绝 |
| F24/F25 | typed_entry_incremental_tier三成功用例；A2X store_cache/ranged接口现成功/早拒绝邻例 | 首/中/晚失败后doc/resident/history/source/anchor及旧产物身份；同批重试；readonly与接受态不混 |
| F26/F32 | source-cte现执行/cycle/nested/single-flight；core source_ct_936前门 | freeze失败事实保持AV已到达；Named/Pos同值都发现并替换需求，服务计数与最终checked值分账 |
| F28/F29/F33 | aggregates_t01_probe::range_for_in_counts_and_executes、collections_rejects、owned字段正常投影 | continue有限反例、闭区间MAX单次结束、Named get.field受控结果；正常/非法位置对照，不能仅改expected迁就当前路径 |

这张表给F01–33全部去向；它是已核的义务差额与反例入口表，**尚非每个原case/check/recipe的逐行迁移账**。没有以2728个属性或新作者格式四例七检查关闭985的完整覆盖认证；1002管格式、985管覆盖、1017管成本、本Plan管代码审计，仍沿原分工。

### 第五轮新增发现F28–33

#### F28：for-in的continue跳过隐式推进 — confirmed（控制流），P1

**证据。** `core/src/source/resolve.rs:6762–6770`将continue绑定最近loop；range:7003–7040与List:7170–7240均在用户body解析后把index增量追加到body尾。`adapter.rs:2005`保continue target；`lowering/src/hir_adapter.rs:6132–6172`对Loop把continue目标设header，continue直接跳header并终止当前块，无for专用latch。因此continue绕过追加的增量；不是AV运行器遗漏某条指令。现range测试只有小界正常累加/非法值位，没有该控制分叉。

**结论。** 具名有限反例：`fn main() int { var n = 0; for i in 0 .. 3 { n = n + 1; if n == 4 { return i }; if n < 4 { continue } }; return 99 }`（正式fixture按普通换行格式）。普通迭代语义为3次后99，现降糖抽象路径第4次return 0；不依赖挂死或耗尽fuel。特意用条件continue：adapter:1825–1845会把无条件终结后的合成增量放入不可达block，verify可能先报block-structure，不能拿那个更早拒绝的输入声称已到错值执行；条件分叉保有语法上可达的增量路径。List同因可重复首元素，不能仅修range。仅内存控制流重放得99/0，未运行真实ACE；源是否在其他前置层另行拒绝仍须真实链核，不能说已观察错误PE。

**建议。** 对每个for-in明确推进continuation/latch，continue应经过本循环推进，break/return不推进，内层循环continue不误推进外层。range/List、owned view、嵌套循环、正常/continue/break/return分别核；不可通过禁止原合法continue隐藏问题。owner=core源码控制流/共享lowering，MC15/18，关联B12/B13。

#### F29：i32闭区间终点MAX后仍合成加一 — confirmed（边界路径），P1

**证据。** 同一range分支用Le比较闭界，body正常结束后无条件合成i32 Add(1)，无“到终点即结束”守卫。AV `av.rs:5684–5695`的AddI32为checked_add→IntOverflow；共享MIR使AV/native/A2X拿到同一合成算术，四端相等不能证明区间本身正确。

**结论。** `for i in 2147483647 ..= 2147483647 { n = n + 1 }`按区间成员应执行一次并正常结束，当前静态路径执行body后求2147483648，发生合成溢出；这不是用户body请求溢出。合法上界范围合同未见排除MAX，现正常范围测试1..5与1..=3无法区分。未运行Rust/PE，实际诊断/trap仍待真实链证据。F28修continue不自动修该边界。

**建议。** 在闭区间最后一次迭代的推进点显式结束，避免通过扩大用户值型或取消checked overflow改变语言算术。MAX单点/MAX-1到MAX、空/反向、MIN、普通半开、continue/return/break邻例核独立预期；owner=core range降糖，MC15/18。

#### F30：聚合V03现用例未触发所称中途失败 — confirmed（该测试证据缺口），P2

**证据。** `av/tests/cases/aggregates_v_rows.rs:38–47::v03_midfail_recovery_two_frames`只调用一次compile_and_execute_aggregates，断言42/live0/allocs=frees。所含`v03_midfail_recovery.at`是main正常调用build(40)、build(2)各创建/扩容列表后相加，无触发失败输入。`av/src/collections_864_support.rs:39–75`用默认ExecLimits、未取消标记，且只在Success返回CollectionsRun，无故障注入或失败后同session重试。该test的“two frames”不能解释为两次独立失败/恢复实验。

**结论。** 该测试能证明普通函数frame退出回收，不能证明“中途分配失败逆序回收且旧目标不变”。这是具体case/check与义务不对应，不能据此断言整个workspace无故障测试或962历史验收失效；现AV collections_resources等有真正fault/事务测试，仍需逐一核其能否承接聚合新嵌套形态。只审代码，未运行该case。

**建议。** 保留正常释放用例并准确命名；另用真实聚合源的半构造、已接受旧值和可控Nth分配失败，核原值/资源事件/失败后可继续及合法对照，优先复用已有fault seam。1002迁作者格式时不能因标题含recovery就标迁移完成；owner=聚合能力/测试映射，MC15/08、985，关联F14。

#### F31：分页CST目录漏条目准入，可溢出跨度算术 — confirmed（读取路径），P1

**证据。** `syntax-artifacts/src/lib.rs:1149–1214::validate_manifest`只遍历m.decls验证origin等，m.decl_index只参与计数。`decl_entry:1309–1343`分页读frame/JSON后直接clone条目，未核origin/node_lo/索引页声明数；`decl_range:1361–1390`与`decl_subtree:1402–1500`随后直接算`d.origin + nd.start`，再算len，未checked_add。view_from_manifest身份/源长验证在读取分页条目前完成，故不补此缺口。当前syntax_admission的畸形构造器使用内联decls、空decl_index。

**结论。** 取一个普通合法CST page和分页条目，把origin置u64::MAX、根start=1/len=1、manifest源长=1，其他page/node/frame/hash均可保持有效：manifest阶段无inline条目可拒，延迟读取时先发生无检查加法。checked溢出构建可panic；无溢出检查构建可回绕为[0,1)而通过后置源界。此为静态算术/控制流反例，未运行Rust reader，不宣称已绕过完整产品恢复或远端攻击成功。

**建议。** 内联/分页共用条目验证，延迟读取后先核page/node/node_lo/origin，再checked span算术；分页shape/count与mixed inline/paged语义也须明确。同一合法/畸形条目在两布局结果一致，保正常非零origin/trivia/epsilon邻例；不靠关闭debug溢出检查解决。owner=syntax-artifacts恢复准入，MC25，关联A05/F16/F21。

#### F32：CT需求发现/替换漏Named实参 — confirmed（当前scalar-CT可达数据形状），P1

**证据。** `source-cte/src/product.rs:83–115`产品明确Scalar+CT能力；原Scalar支持命名参数，core的p15-named-eval-order-x2和named_args_preserve_source_order证明该作者面；source-query的sq18_named_arguments_produce_clean_facts_and_a_distinct_key亦有事实车道合法邻例。`schedule.rs:501–530`在Call/Method/MethodCall参数只遍历Arg::Pos/MovePos；find_hash_expr:1090附近、replace_at:825附近、substitute:777附近与product::substitute_all的repl同样跳Named。core AST的Named真实value字段没有消失。故同值改写为命名实参会漏掉HashExpr，而typecheck:295–305明确拒未服务CT。

**结论。** `fn id(x int) int { return x }`配`fn main() int { return id(x: #{ 1 + 2 }) }`应沿已有scalar named/CT合同得到3；当前需求发现走不到该节点，整体替换亦不走Named，最终仍带未服务HashExpr，不能当普通runtime执行。Pos邻例`id(#{ 1 + 2 })`走该子树。仅源码/抽象遍历证明，实际前门与最终错误尚未执行；Hash内部命名调用、nested/meta场景须分别核，而非只补一个root搜索。

**建议。** 在现有CT模块共用显式遍历/重建Arg.value规则，至少覆盖Pos/MovePos/Named及source-order，发现、call closure、meta、替换、find使用一致子树面；不再造CT executor。相同语义Named/Pos、参数倒序、nested、meta、能力拒绝和普通同名函数分别核独立值/真实执行计数；owner=source-cte/共享AST遍历，MC24/15，关联C09。

#### F33：owned元素字段融合对Named索引用unreachable — confirmed（源码到panic分支），P1

**证据。** `core/src/source/cst/project.rs:5794–5802`普通call实参把named_arg组装为Arg::Named，method同用push_args。`resolve.rs:8618–8740`在Field的collections融合前置路径识别`local_list.get(single_arg).field`，若元素owned且字段存在，则`ma[0]`仅接受Pos/MovePos，其余`unreachable!("get takes one positional index")`，在通用实参规则处理前执行。普通resolve_list_method:5512附近按rargs长度/值处理，不能保护该更早的融合分叉。

**结论。** `type Item { text Str; code int }`、`List<Item>`持有一个合法Item后，`return xs.get(index: 0).code`形成满足该前置路径的Named参数；依当前源码进入panic而非受控诊断。即使builtin的合同最终规定只接受位置参数，也应受控拒绝，不能以此证明unreachable。`xs.get(0).code`为合法邻例；不带字段的普通调用走不同路径，不可代证。未执行真实前门/Rust/PE，影响范围限定本融合路径。

**建议。** 在融合前复用并核实builtin实参准入，不用panic表达用户错误；Named若允许须统一参数映射，若不允许返回具名诊断。owned/Copy元素、存在/不存在字段、Pos/Named/多参/错型、嵌套字段分别核，保source-order和owned view零物化；owner=core collections字段投影，MC15，关联B05/B13。

### 第六轮：HIR资源细分与逐项映射闭合（2026-11-13，当前）

**闭合口径。** 承接用户明确要求“直到只读深审闭合”，本轮关闭的是固定源码上的调查/分类/迁移义务清单，不把静态审查升级为“所有分支测试通过”或风险已修。生产/测试/配置均不修改、不构建、不运行ACE；发现的真实Rust/PE/fault/OS反例归具名后续修复与验证，不能把它们既作本轮未授权运行前置、又无限保留为未读事项。T-04独立复审、T-05归档另有门禁，均不因本轮闭合提前完成。

固定源沿`81d8fc03f10f4a7d179b0bc3a917602406ed7b91`；开读时相关源码无WIP，1024上轮未提交簿记读回保留。下列路径相对`experimental/ace/crates/`；全文映射中的路径相对仓根。只读提取在内存中完成，没有另写报告/脚本/附件。

#### HIR类型、凭证与资源边界

读取`core/src/hir.rs`全部17个TypeKind、42个ExprKind、9个Stmt；跟踪`verify.rs::{check_profile_ops,check_body,check_collections_rules,check_closure_borrow_rules,check_closures_function_rules}`→TextResourcePlan/CollectionsBodyFacts/ClosureBodyFacts/ForeignResourcePlan→共享HIR→MIR lower→MIR ownership→AV/native及A2X消费者。核对不同域/物理表示，不强迫所有层的owned布尔值字面相等，也不把不同凭证合并成一次verify。

| 类型分区（17个TypeKind全覆盖） | 资源/授信结论 | 保留的差异 |
|---|---|---|
| I32/Bool/Char/I8/U8/U32/I64/U64/F32/F64（10） | 值与位型，无拥有资源；窄化/非法scalar仍各自有trap/静态型边界 | char不按任意u32放行；float NaN/负零不可文本近似 |
| Str | 借用视图/来源生命周期，不因含文本字节就当新owner | 静态literal与外部param、caller-owned view分别保持 |
| StrOwned | owner；clone/concat/promotion成功才接受，失败归还新建责任 | data/text hooks/取消/ledger合同不整体混并 |
| Function | 名字/签名，不是可随意存放的运行时closure/FFI地址 | CallbackRef另需foreign lane凭证 |
| Closure | captured env由closure facts及MIR/runtime管理；borrow/Move/Shared分别核 | CollectionsBodyFacts把Closure记None不单独报漏free：A2X resolve_ty拒此值型，AV/native走closure机制 |
| Record | 结构字段、同名owner/布局、借用或拥有；递归经不透明边与直接内联环不同 | Str-only历史谓词不足覆盖List字段，新增F34；不据Rc表示就取消source move纪律 |
| List | backing/元素/代际/Place；结构变化与字段写区分 | nested/opaque边按aggregates准入；冻结collections原拒绝保持 |
| Variant | tag/guard/payload转移；不同表示可有不同物理owner责任 | 无payload/Copy payload与owned payload分开，不把所有投影都当deep copy |

#### 42个HIR表达式资源矩阵

T=TextResourcePlan，C=CollectionsBodyFacts，K=ClosureBodyFacts+MIR/runtime，F=ForeignResourcePlan；它们都是已验证域的后续消费机制，不是可恢复的执行授权。表中的“拒”保原域/目标准入；“现测试”列导航由下方逐case/check/recipe映射及native引用表落实，不冒执行结果。

| ID / ExprKind | 借用、转移、临时值和失败条件 | producer→consumer / 保留义务 |
|---|---|---|
| HR01 ReadLocal | 初始化/未移动/未丢弃/活borrow；参数view不成为新owner，shared cell读当前值 | HIR init/alias→MIR MUST init/MAY moved/dropped；typed/src分层测试保留 |
| HR02 Constant | i32/bool值无拥有责任；依声明型解释 | shared Const/codec/执行，不用显示文本当位oracle |
| HR03 Unit | 无payload，不在collections effect结果位置造owned slot | Eval/Return域规则；不可存成普通local/operand |
| HR04 NumericConst | 字宽/符号/float bits恒等，无资源转移 | NumConst→MirConst；numeric_hir/source及av_numeric_semantics |
| HR05 Cast | 单operand；范围/float-int/shift等按矩阵；失败不产生新owner | shared Conv，原trap/fuel/源span保持 |
| HR06 Binary | 左右求值/精确型；text eq/ne和numeric不同；checked算术失败清理当前frame | lower的真实MIR型分派，F04抽机制不删清理 |
| HR07 Unary | 单operand，不额外制造拥有值；neg边界与bool not不同 | shared Un/Num，错误类不并成泛错误 |
| HR08 Logical | RHS按极性/常量可达性，T区分Normal/Conditional/Dead moves；不能eager化 | CFG join及MUST/MAY状态；shortcircuit-latent原case保留 |
| HR09 Call | eval_args保源码顺序、bindings才映射形参；View/MutBorrow/MoveOwn、owned返回/错误责任不同 | T/C/K；foreign调用还要F、当前caps/provider；不能把Call存在当授权 |
| HR10 CallClosure | callee先于args；签名、env generation、borrow存活；返回及trap清理 | K+MIR CallClosure；A2X未支持的closure值明确拒，不由C遗漏推定泄漏 |
| HR11 CallbackRef | 仅F同步noescape lane消费的命名free fn；不持任意函数地址 | F callback def/sig/ctx，公开ABI与内部carrier分开 |
| HR12 CreateClosure | Move capture杀source、Borrow pin owner、Shared cell共享；env半构造与失败归还 | verifier K→MIR CreateClosure/env prepare/commit；不重造每consumer捕获推导 |
| HR13 StrLit | 借用literal/static，拥有位置才promotion；空/非空与取消/fault分开 | T/C的owned_value_child，AV Const按目的型，native runtime hooks |
| HR14 CharLit | 校验Unicode scalar；无需owned清理 | CharConst；非法scalar不能因u32可携带就放行 |
| HR15 TextLen | 借用operand、结果为码点数；不转移owner | StrLen；C限制面与完整T面分别准入 |
| HR16 TextLenBytes | UTF-8字节长度，与HR15不互代 | StrLenBytes；A2X Collections未支持的宽文本操作具名拒 |
| HR17 TextCharAt | operand/index次序、scalar index/OOB；错误清理pending+live | T→StrCharAt，各目标独立trap期望 |
| HR18 TextSlice | 借用view、界/逆序/来源extent；结果不是新独立buffer owner | T provenance→StrSlice；元数据/索引工作照预算，不承诺零heap |
| HR19 TextConcat | 两operand、fresh owner；失败释放已建临时，原owner保持 | T/C temp→StrConcat；字符串长度/复制工作与分配均守预算 |
| HR20 TextClone | fresh deep copy，原值保留；Nth fault不冒成功 | T/C temp→Clone/StrToOwned；保av_owned_v03_clone_fault等真实入口 |
| HR21 ClosureClone | owned env deep clone；borrow closure拒克隆，半env失败按K清理 | K→MIR Clone；A2X值型拒绝边界保留 |
| HR22 TextToOwned | borrowed→fresh owned；promotion失败不消费原view | T/C temp→StrToOwned；取消/Nth alloc/旧dst保全 |
| HR23 TextToBorrowed | view来源集合，不延长owner寿命、不准owner提前move/free | T alias roots→View；caller/external source区别 |
| HR24 CharToCodepoint | scalar→int，无owner转移 | CharToCodepoint；NUL/非ASCII保持精确值 |
| HR25 CharFromCodepoint | int→scalar，非法码点是运行trap而非静态型错 | CharFromCodepoint；不能把负/代理区作成功字符 |
| HR26 RecordCtor | eval顺序先求值再字段排列；字段恰一次，拥有operand在接受时转移 | T/C pending→Aggregate；半构造失败与旧目标不变分账，F34 |
| HR27 FieldGet | 真实owner路径/field型；owned record/view、ListGet融合、Copy/owned末端分别处理 | FieldGet/FieldMove/ListFieldGet/Place；F33 Named前置panic、F34资源分类保反例 |
| HR28 BorrowView | base一次求值，view不free owner，escape必须拒绝 | T/C provenance→View/pass-through；独立verify不去掉 |
| HR29 BorrowMut | mutable根、独占及跨函数exclusion fixpoint；copyable与owned写回路径不同 | T+F shadow/commit或MIR MutRef；错误保caller原字节 |
| HR30 RecordClone | 逐资源字段deep copy，原record保持；view来源owned字段受限 | T/C→Clone或FieldGet+StrToOwned+Aggregate；List字段分类见F34 |
| HR31 Move | whole-root，非partial；kill once；同语句view冲突、use-after-move保持拒绝 | T/C/K→MIR Move/ownership；不能仅改owned判定放宽旧域 |
| HR32 ListNew | backing allocation，容量/元素闭包/预算；成功才成为pending owner | C→ListNew→native/A2X runtime；空列表也有表示责任 |
| HR33 ListLen | 借用list，不消费；不同于cap、物理bytes | C→ListLen；无效handle/代际另核 |
| HR34 ListPush | list和值按源码顺序；owned rvalue/move-source，扩容失败不消费旧list/值 | C err_frees→ListPush/runtime；List结构变更让Place stale |
| HR35 ListInsert | index/value/容量；OOB或分配失败保旧顺序/责任 | C→ListInsert；guest/resource/trap/infra分态 |
| HR36 ListGet | Copy元素可取值，owned为投影/view；whole materialization拒绝 | C+HIR view-escape→ListGet/ListPlace/PlaceGet；裸get不代融合分支 |
| HR37 ListSet | RHS先物化/pin，再替旧元素、旧owner恰释放一次；失败旧dst保留 | C replace/err_frees→ListSet；av_collections_resources_listset_transaction_both_sides |
| HR38 ListRemove | Copy copy-out或owned transfer-out，更新generation；不能把两者都当clone | C temp/kill→ListRemove；真正资源事务用例保留 |
| HR39 ListClear | 不是新opcode；尾remove+必要Drop，最终frame释放空容器 | C→ListLen/ListRemove/Drop；opaque资源record分类见F34 |
| HR40 VariantCtor | tag/payload型，payload有无、Copy/owned责任；接受时才转移 | C→VariantPack，形状/失败/清理分别核 |
| HR41 VariantGuardTest | root须whole local；生成正guard事实，无payload消费 | HIR guard→VariantGuard，非选中arm不应trap |
| HR42 VariantProject | 活guard、当前branch、owned move-out或Copy；错误branch/后续重投影拒绝 | C→VariantProject/MIR ownership/runtime；保variantproject事务反例 |

#### 9个HIR语句及跨层失败状态

| ID / Stmt | 必须保留的资源状态分叉 | 已核消费路径 |
|---|---|---|
| HS01 Let | 接受前pending，接受后local；View/MutBorrow参数不进frame owner，MoveOwn进入 | C::block Let / T live+provenance / MIR init |
| HS02 Return | 返回值转出不free；其余live与pending逆序；borrow返回须外部/静态源 | C exit_frees/T returned_owned/MIR RETURN+frame cleanup |
| HS03 If | 终结臂不参与fallthrough交集；未终结臂保common；T dead/backing/gen合流 | C::block四种then/else终结组合；MIR MUST init/MAY moved/borrow |
| HS04 Loop | kill dry-scan、survivors、每迭代scoped death；back-edge资源/取消 | C loop_survivors、MIR worklist；A2X continue绕poll见F35 |
| HS05 Assign | RHS成功才释放旧值/安装新值；field-path与whole local不同；immutable独立拒绝 | C replace_frees/T replace/MIR FieldSet；C/Rust collections field-place域外显式拒 |
| HS06 Break | 清理离开scope的责任，保loop后survivors，不执行继续推进 | C exit_frees→MIR exit或目标break；F28合成latch修复须保此义务 |
| HS07 Continue | 清理当前迭代责任，保survivors，再执行正确latch/预算poll | C exit_frees→MIR header/A2X continue；F28/F35 |
| HS08 Drop | owned local一次释放，重复/未初始化/use-after-free受控拒绝 | MIR DROP及runtime ledger；A2X collections不接受显式Drop，不能假称该目标支持所有HIR |
| HS09 Eval | 只消费Unit effect/已明确Copy remove，不能丢失owned返回值 | verify_collections_rules/C pending truncation→NO_SLOT；各target准入仍保留 |

**细分结论与建议。** 已核42+9分派与资源状态、17类型的支持/拒绝及域差异，不再把未运行当“还没读”。MIR ownership在分配前按blocks×slot/token矩阵计预算，init取交/MAY moved/drop/borrow取并并在loop求不动点；T跨callee参数exclusion独立求不动点；C事实由已验证body派生并以digest/owner/context绑定；K补捕获/跨body规则；F绑定current provider/handle代际/同步callback/noescape与mut/out shadow commit。故不能为复用把五者合成“一个owned bool/一个verify”。建议只抽领域明确的类型资源谓词、child枚举和cleanup/latch机制，保各授信门与静态库ABI。F30已有真fault入口可复用，但聚合新形态仍须具名新增反例，不能把旧Str故障例按名字复制覆盖。

#### F34：资源record分类仍只沿Str边，漏List等不透明资源 — confirmed（跨层事实不一致），P1

**证据。** `core/src/collections_facts.rs:106–156`的TypeClassifier对Record遍历字段并把List记OwnedKind::List；相反`core/src/source/resolve.rs:945–965`、`lowering/src/hir_adapter.rs:2447–2470`、`av-core/src/ownership.rs:49–78`的record分类只沿StrOwned/record边，不把List字段当资源。MIR Move在ownership:1055附近明确拒is_owned=false。aggregates合同已允许record的List/Box字段、tuple物化record；现composite_record_list_field_executes用`Node { name Str; kids List<int> }`，name Str恰使旧谓词为true，不能验证“只有List字段”的邻例。

**结论。** 最小类型形状`Record{items:List<i32>}`：C清理事实为owned，源码/上述MIR record谓词为false；`Record{items:List<i32>,name:StrOwned}`又为true。这不是所有不同表示owned标记都必须相同的主张：具体问题是已经承诺的List-field record/tuple在move、owned-element判定、FieldMove/cleanup策略处仍复用Str-only谓词。合法`.move`被拒与复制/投影责任错误属于需真实反例核的下游风险；本轮未运行，不报告已观察双free/错误机器结果。

**建议。** 先冻结各域的语义owner/物理drop/可借用定义，再扩aggregate资源闭包（包含opaque List/Box、合法Variant payload，closure仅限实际准入域），所有消费者沿同一明确语义消费；不能简单把所有record/借用str都改成owned。只含List、含Str+List、纯标量、嵌套record、tuple-of-list、Box递归配move/clone/get/clear/参数/返回/fault对照。owner=core资源分类/共享lowering/MIR ownership，MC15/18/09；AC-02风险登记，修复不在本轮。

#### F35：A2X collections的continue绕过回边poll — confirmed（生成控制流），P1风险待运行量化

**证据。** `a2x-rust/src/lower_coll.rs:384–434`把scoped frees和a2x_coll_poll追加在Loop body尾；`:440–446`的Continue只emit exit_frees+RStmt::Continue，print.rs:345直接打印continue。C的lower_coll.rs:583–636同样在用户body后插poll，Continue直接打印continue。Rust coll.rs:341明确poll每回边1WU、扣work_remaining并查cancelled。源/typed HIR的条件Continue能在任意多次回边跳过该尾部poll，而AV/MIR执行预算按实际dispatch另计。

**结论。** 该生成路径漏本应覆盖continue回边的协作预算/取消点；不是缺case文件，也不是F28隐藏index推进的同一断言。其他helper或父截止可能仍终止程序，故不声称所有链路永不取消或已实测挂死；但不能用父kill/normal fallthrough poll代证guest每回边预算。范围限定C/Rust collections生成，未运行编译器/生成程序。

**建议。** 统一continue→本循环cleanup/latch/poll，break/return及内层continue各守自己的责任；同源合法control和有条件continue循环，分别核WU/poll trace、预算边界、cancel、LIFO/原值和父终态。沿现runtime poll与owned executor，不造新runner。owner=A2X C/Rust lowering/runtime，MC01/03/18；可与F28同设计切片，但保两种独立反例。


### 第六轮逐项映射附录（只读保留账，不是执行报告）

**证据与算法边界。** 对跟踪的181份case文件、71份suite，按围栏/字面对象在内存读取；770个case/1116个物理check、1133个suite绑定全部消费，无无法解析的片段。提取器只用于定位/比对，不替代AutoDown/Rust合同reader，不输出lint/执行PASS。fixture base按test_cases::run_case和ir::fetch_payload实际规则取suite.fixture_root；每个source/files、expect artifact/bindings sidecar均找到并读hash。51份inline source与719份file source按声明区分；source combined hash沿ir.rs的虚拟path+NUL+文件SHA+NUL排序规则复算。suite defaults单独绑定，禁止用本附录的审计声明hash冒runner的effective_check/oracle hash。

完整旧目录511项全部按(suite,case,check)逐行对回当前物理check，adapter/action/recipe/source combined hash/case文件hash一致，旧668份pin输入逐文件吻合；7个registry内嵌inventory的69项、expanded inventory、332个legacy入口及313个legacy cell另作下面对应。35个registered recipe全有实际case使用且可分派，另外7种adapter/action保旧VM/旧A2R/AC/syntax角色，不冒新ACE支持。70个suite有固定case组，a2r-hir-892保原具名手动入口；无缺失include/源/expect文件、无未知recipe，不把手动入口悄悄计入自动聚合。

**身份注意。** `ace.m1.r08/{ac-native,av}`在两个不同suite中复用裸case/check名，属不同完整身份；本表保file+suite，不按裸pair去重。283个native身份另核package/target/module完整路径（词法屏蔽Rust字符串/注释再展开mod/path），不是仅搜到同名fn；依旧不冒nextest编译发现。

**结论/建议。** 每条M行的迁移结论都是“保留当前完整身份/原件/expected/条件/原入口”；本轮没有授权搬家、删case、缩check或修改expected。R行只映射同一真实dispatch及对应资源义务；1002新作者格式若替换旧表示必须按此账一一回填，未替换者继续留原入口，不能复制成第二runner或让新语义自证。expect顶层只有completion的209条只承诺声明终态及其中显式结构化拒绝，不能按名字扩大成值/资源oracle；typed/result/stream/artifact/probe通道各保原声明与字节。所有M/N/L/A行执行状态统一UNOBSERVED（本轮未运行），这里的静态对应完成不冒产品PASS、M1重签或覆盖率。

#### R：42条真实分派路线（35个registered recipe + 7种无recipe组合）

| ID | adapter/action | recipe | 真实父入口 | 源码定位 | 物理check数 | 保留义务 |
|---|---|---|---|---|---|---|
| R01 | a2r/emit | - | a2r::run_check | crates/auto-lang-core/src/test_cases/a2r.rs:45 | 99 | 旧A2R emit/golden；不冒新A2X compile-run认证 |
| R02 | ac/build | - | ac::run_check | crates/auto-lang-core/src/test_cases/ac.rs:487 | 4 | MC15/19/25–28/36；源码/语法或AC产物；当前请求授信 |
| R03 | ac/check | - | ac::run_check | crates/auto-lang-core/src/test_cases/ac.rs:487 | 16 | MC15/19/25–28/36；源码/语法或AC产物；当前请求授信 |
| R04 | ac/run | - | ac::run_check | crates/auto-lang-core/src/test_cases/ac.rs:487 | 19 | MC15/19/25–28/36；源码/语法或AC产物；当前请求授信 |
| R05 | avm/run | - | avm::run_check | crates/auto-lang-core/src/test_cases/avm.rs:70 | 73 | 旧AutoVM对照；不能作为新ACE默认实现 |
| R06 | probe/run | abi/ca-d1-bridge-v1 | abi::run_check | crates/auto-lang-core/src/test_cases/abi.rs:792 | 11 | MC06/17；当前foreign ABI/provider/callback/handle/atomic |
| R07 | probe/run | abi/interop-v1 | abi::run_check | crates/auto-lang-core/src/test_cases/abi.rs:792 | 4 | MC06/17；当前foreign ABI/provider/callback/handle/atomic |
| R08 | probe/run | abi/resource-foreign-certify-v1 | abi::run_check | crates/auto-lang-core/src/test_cases/abi.rs:792 | 57 | MC06/17；当前foreign ABI/provider/callback/handle/atomic |
| R09 | probe/run | abi/source-foreign-certify-v1 | abi::run_check | crates/auto-lang-core/src/test_cases/abi.rs:792 | 31 | MC06/17；当前foreign ABI/provider/callback/handle/atomic |
| R10 | probe/run | ac-a2c/hir-collections-emit-v1 | ac_a2c_hir::run_check | crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs:114 | 20 | MC01–05；目标family/manifest/四文件golden，日常spawn-zero |
| R11 | probe/run | ac-a2c/hir-record-text-emit-v1 | ac_a2c_hir::run_check | crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs:114 | 5 | MC01–05；目标family/manifest/四文件golden，日常spawn-zero |
| R12 | probe/run | ac-a2c/hir-scalar-emit-v1 | ac_a2c_hir::run_check | crates/auto-lang-core/src/test_cases/ac_a2c_hir.rs:114 | 5 | MC01–05；目标family/manifest/四文件golden，日常spawn-zero |
| R13 | probe/run | ac-a2r/hir-collections-emit-v1 | ac_a2r_hir::run_check | crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs:107 | 20 | MC01–05；目标family/manifest/四文件golden，日常spawn-zero |
| R14 | probe/run | ac-a2r/hir-record-text-emit-v1 | ac_a2r_hir::run_check | crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs:107 | 9 | MC01–05；目标family/manifest/四文件golden，日常spawn-zero |
| R15 | probe/run | ac-a2r/hir-scalar-emit-v1 | ac_a2r_hir::run_check | crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs:107 | 7 | MC01–05；目标family/manifest/四文件golden，日常spawn-zero |
| R16 | probe/run | ac-av/closures-fn-run-v1 | ac_av::run_closures_fn_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1557 | 13 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R17 | probe/run | ac-av/closures-recovery-run-v1 | ac_av::run_closures_recovery_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1579 | 5 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R18 | probe/run | ac-av/closures-run-v1 | ac_av::run_closures_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1514 | 1 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R19 | probe/run | ac-av/closures-source-run-v1 | ac_av::run_closures_source_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1536 | 24 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R20 | probe/run | ac-av/collections-run-v1 | ac_av::run_collections_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1677 | 1 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R21 | probe/run | ac-av/collections-source-run-v1 | ac_av::run_collections_source_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1658 | 44 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R22 | probe/run | ac-av/compact-run-v1 | ac_av::run_compact_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1310 | 1 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R23 | probe/run | ac-av/console-run-v1 | ac_av::run_console_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1329 | 15 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R24 | probe/run | ac-av/core-run-v1 | ac_av::run_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1270 | 153 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R25 | probe/run | ac-av/data-incremental-v1 | ac_av::run_data_incremental_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1698 | 4 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R26 | probe/run | ac-av/data-run-v1 | ac_av::run_data_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1365 | 44 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R27 | probe/run | ac-av/dynamic-run-v1 | ac_av::run_dynamic_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1717 | 2 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R28 | probe/run | ac-av/loaded-run-v1 | ac_av::run_loaded_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1289 | 10 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R29 | probe/run | ac-av/owned-run-v1 | ac_av::run_owned_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1736 | 52 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R30 | probe/run | ac-av/resident-run-v1 | ac_av::run_resident_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1635 | 3 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R31 | probe/run | ac-av/resource-events-v1 | ac_av::run_resource_events_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1777 | 5 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R32 | probe/run | ac-av/resource-run-v1 | ac_av::run_resource_check | crates/auto-lang-core/src/test_cases/ac_av.rs:1756 | 10 | MC08/09；共享HIR/MIR、profile、frame/arena、父值/trace/stream |
| R33 | probe/run | ac/mir-native-closures-resource-v1 | ac_mn::run_check | crates/auto-lang-core/src/test_cases/ac_mn.rs:1690 | 9 | MC19/31；共享MIR/native、真实PE、typed bits或资源账 |
| R34 | probe/run | ac/mir-native-collections-v1 | ac_mn::run_check | crates/auto-lang-core/src/test_cases/ac_mn.rs:1690 | 43 | MC19/31；共享MIR/native、真实PE、typed bits或资源账 |
| R35 | probe/run | ac/mir-native-resource-events-v1 | ac_mn::run_check | crates/auto-lang-core/src/test_cases/ac_mn.rs:1690 | 5 | MC19/31；共享MIR/native、真实PE、typed bits或资源账 |
| R36 | probe/run | ac/mir-native-resource-guards-v1 | ac_mn::run_check | crates/auto-lang-core/src/test_cases/ac_mn.rs:1690 | 9 | MC19/31；共享MIR/native、真实PE、typed bits或资源账 |
| R37 | probe/run | ac/mir-native-scalar-v1 | ac_mn::run_check | crates/auto-lang-core/src/test_cases/ac_mn.rs:1690 | 233 | MC19/31；共享MIR/native、真实PE、typed bits或资源账 |
| R38 | probe/run | ac/mir-native-typed-v1 | ac_mn::run_check | crates/auto-lang-core/src/test_cases/ac_mn.rs:1690 | 12 | MC19/31；共享MIR/native、真实PE、typed bits或资源账 |
| R39 | probe/run | cte/scalar-query-v1 | ac_cte::run_check | crates/auto-lang-core/src/test_cases/ac_cte.rs:397 | 6 | MC12–14/24；需求/key/实际执行/冻结，资源与语言错误分态 |
| R40 | probe/run | sema/query-v1 | sema::run_check | crates/auto-lang-core/src/test_cases/sema.rs:841 | 5 | MC20–23；事实/deps/restore/commit与父裁判 |
| R41 | syntax/edit | - | ac::run_check | crates/auto-lang-core/src/test_cases/ac.rs:487 | 1 | MC15/19/25–28/36；源码/语法或AC产物；当前请求授信 |
| R42 | syntax/parse | - | ac::run_check | crates/auto-lang-core/src/test_cases/ac.rs:487 | 26 | MC15/19/25–28/36；源码/语法或AC产物；当前请求授信 |

<details>
<summary>S：71个suite的fixture/default/固定入口身份</summary>

| ID / suite | suite文件（case根之下） | fixture_root | 固定入口 | suite SHA256 / defaults审计SHA256 |
|---|---|---|---|---|
| S01 a2c-hir-collections-926 | a2c-hir-collections-926/a2c-hir-collections-926.suite.ad | . | case-a2c-hir-collections-926 | eeee0720fe818067fc143dd2b8c012ff50a72013cf0205e9e52f4fd6872fa5fc / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S02 a2c-hir-record-text-925 | a2c-hir-record-text-925/a2c-hir-record-text-925.suite.ad | . | case-a2c-hir-record-text-925 | 837e831eb11e44ee540ab4f157c81f9d7a4cb73a2ae3049c5a99621c81790725 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S03 a2c-hir-scalar-921 | a2c-hir-scalar-921/a2c-hir-scalar-921.suite.ad | . | case-a2c-hir-scalar-921 | 552d8be4bb2273fb2bea0c05062044a363bee3730769ff18f098e6fcaef2ba43 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S04 a2r-hir-892 | a2r-hir-892/a2r-hir-892.suite.ad | . | 原具名手动入口 | 2dbc8f2fecc0d7955441c64eeca79e3a2a65646a9cbd5e4c637b639f467bbeb9 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S05 a2r-hir-collections-904 | a2r-hir-collections-904/a2r-hir-collections-904.suite.ad | . | case-a2r-hir-collections-904 | d8c6644c2defef5f5f3ecf3a96d730a05855f89851999f2f331674d758461e1a / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S06 a2r-hir-record-text-902 | a2r-hir-record-text-902/a2r-hir-record-text-902.suite.ad | . | case-a2r-hir-record-text-902 | 310fe3944d5840d56ec201d805388369653263e915f59a3e7bef47757104e6d3 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S07 ac-av | ac-av/ac-av.suite.ad | . | case-ac-av | 01006362d1d17c6877d3e45b7742c85c74365d46adcfc461c2c202962adcecbd / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S08 ac-cte | ac-cte/ac-cte.suite.ad | . | case-ac-cte | f863d9cf80b840663d7d8d821a63848fd37c51cbc519cab9adac22fa4afce24b / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S09 ac-mn | ac-mn/ac-mn.suite.ad | . | case-ac-mn | 7911d29aec83bce78b69477a74c92a9772c5a7bd43461d37f351cc1dc41375e5 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S10 ac-native | ac-native/ac-native.suite.ad | . | case-ac-native | 1165ade5554f7d91caf37c91e0b02657b65af70b5faddfaec9412db4064df430 / eae1fb88f4bb425e0fe42a4a673af3aa3439f697f3ead3d3b9117e186ddd9197 |
| S11 ac-semantics | ac-semantics/ac-semantics.suite.ad | . | case-ac-semantics | 627ed254e8b0c761c70d41bd7bf507b108892d1f7f1bb1e5fe943f5bf4671fa2 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S12 ace-ca-d-resources-939 | ace-ca-d-resources-939/ace-ca-d-resources-939.suite.ad | . | case-ace-ca-d-resources-939 | 67cf7ee40070a84bf20582914619ddd3673a408799bb16c2d7111043b69ae7d9 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S13 ace-ca-d-scalar-937 | ace-ca-d-scalar-937/ace-ca-d-scalar-937.suite.ad | . | case-ace-ca-d-scalar-937 | 0372dbfbd0446b8bc5e3accdfaa8e902dfa9b6531999c7371d256b01ef2932da / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S14 ace-ca-d-source-938 | ace-ca-d-source-938/ace-ca-d-source-938.suite.ad | . | case-ace-ca-d-source-938 | bb0869b9dac3f7ae7ed38fcf881b43ba8b5cd885f462e6b6baafbfeba448910b / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S15 ace-m1-expanded-941 | ace-m1-expanded/ace-m1-expanded.suite.ad | . | case-ace-m1-q4-941 | 98d83748f3128c213aca0555a71f599fbf8214ace713c87ffef828dce8595895 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S16 ace-m1-collections-q4-941 | ace-m1-expanded/collections-q4.suite.ad | .. | case-ace-m1-collections-q4-941 | f3cd8762e31d32e9d32c6cb857f59c0e7f9e85f518353af44b5f90a0699a826e / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S17 ace-m1-legacy-coverage-941 | ace-m1-expanded/legacy-coverage.suite.ad | . | case-ace-m1-legacy-941 | 0710de1688d947f843595d50f458e26863f871dcb9a63032738545f670a956eb / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S18 ace-m1-original-data-diagnostics-941 | ace-m1-expanded/original-data-diagnostics.suite.ad | .. | case-ace-m1-original-data-diagnostics-941 | cb5c12105a7bebb92b9d39fd20b7ae0a824fd4b098b4a6ec8ac009f5f821b21a / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S19 ace-m1-original-data-q4-941 | ace-m1-expanded/original-data-q4.suite.ad | .. | case-ace-m1-original-data-q4-941 | 7178c9ae81ade6ae2b6b6a7fbb2b0b46b1e3ae020388b0e01026d91aee8d4d32 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S20 ace-m1-record-text-q4-941 | ace-m1-expanded/record-text-q4.suite.ad | .. | case-ace-m1-record-text-q4-941 | cf105ef7b6d59a9685fd40f17f88f7be6b9eac10fee7d54b716ae1afb8db916c / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S21 ace-m1-scalar-q4-941 | ace-m1-expanded/scalar-q4.suite.ad | . | case-ace-m1-scalar-q4-941 | c275414c23838210c7ee5b8c08293a1af570b698eb86e55c52eb77b9d072854c / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S22 ace-m1-source-supplement-941 | ace-m1-expanded/source-supplement.suite.ad | .. | case-ace-m1-source-supplement-941 | e12b3f6822f8d7fb6b356cc90ae7d3c682279a1ca4af8f2fb9bd6e6f23af3135 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S23 ace-m1-text-q4-941 | ace-m1-expanded/text-q4.suite.ad | . | case-ace-m1-text-q4-941 | 7cec1e4888c06c46032584dfa5b57676c83c671edf740124905be15d20ef9775 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S24 ace-m1-residual-999 | ace-m1-residual-999/ace-m1-residual-999.suite.ad | . | case-ace-m1-residual-999 | a98d9ee96903eaa85907bcd575f14b236caf2305fb9660841677a38c2d978537 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S25 case-ace-m1-a2r-emit-917 | ace-m1/a2r-emit-917/a2r-emit-917.suite.ad | .. | case-ace-m1-a2r-emit-917 | 81b7143d33c982e9347e7e18458959ce4517251216984c6640469256d177d6e7 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S26 a2r-owned-temporaries-931 | ace-m1/a2r-owned-temporaries-931/a2r-owned-temporaries-931.suite.ad | . | case-a2r-owned-temporaries-931 | 83c93e64a215f2c71b7767aec21af78ad6a69a11d3f43c62e240fd38e616b342 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S27 case-ace-m1-a2r-semantics-916 | ace-m1/a2r-semantics-916/a2r-semantics-916.suite.ad | .. | case-ace-m1-a2r-semantics-916 | 9f38f179e7828044d3bbbaf46169ddff291b60343f10481178dc3e165a1d2e69 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S28 ace-m1-base | ace-m1/ace-m1-base.suite.ad | . | case-ace-m1-base | 7fd00b8d1c79d45be1f4d5ad90083780a8acb67798c6c36b9d29b8f53d26ae63 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S29 ace-m1-tr-native | ace-m1/ace-m1-tr-native.suite.ad | . | case-ace-m1-tr-native | 4e2b320d1b44f6c317ff1c69e29706efb4fc1332b04864f955f85aa5208d7fb3 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S30 ace-m1-closures-fn-912 | ace-m1/closures-fn-912/closures-fn-912.suite.ad | . | case-ace-m1-closures-fn-912 | 310ea668d14732fef309d3c24dc182f594047c2f969b907b267d6fe3cf25f917 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S31 ace-m1-closures-recovery-913 | ace-m1/closures-recovery-913/closures-recovery-913.suite.ad | . | case-ace-m1-closures-recovery-913 | 22ec80d6c6c727c0fb75f489dba2a27a7b6ea0a79a05f0f922a1a8f22680a3e5 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S32 ace-m1-closures-865-source | ace-m1/closures-source-865.suite.ad | . | case-ace-m1-closures-865-source | 3b50bbc97890dd244f2d25805a21dd1efc4b1ed04dfdee1e3d7025a971276f98 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S33 ace-m1-collections-864 | ace-m1/collections-source-864.suite.ad | . | case-ace-m1-collections-864 | f100e0083ad1511a4f2962223072d014854cdcebdd70fa31e6c670bb53a70a41 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S34 ace-m1-console-863 | ace-m1/console-863/console-863.suite.ad | . | case-ace-m1-console-863 | d88920b15c9a7c04bb6bce793d814c94d3ad2c30d96efad7984660c7f8e11c6b / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S35 ace-m1-entry-862 | ace-m1/entry-862/entry-unit-862.suite.ad | . | case-ace-m1-entry-862 | af0f77004930570714cb4a75111b69a9eb4db39fc1e381169812200f94851657 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S36 ace-m1-immutable-avm-920 | ace-m1/immutable-avm-920.suite.ad | . | case-ace-m1-immutable-avm-920 | be2909add47887fb2a5880daeb29a2fe6c7c7f76088994519781c047f74b1374 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S37 legacy-a2r-cert-992 | ace-m1/legacy-a2r-cert-992.suite.ad | . | case-legacy-a2r-cert-992 | 92fd0fb0fbf3ceaf897eddb9910da9c7504c6c364bad27685c7c622ab2f78b30 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S38 legacy-a2r-list-996 | ace-m1/legacy-a2r-list-996.suite.ad | . | case-legacy-a2r-list-996 | ee87c899f93d32747722c7b3af4d1ea08a38b26a6b611fb9d73e6cf540f4c040 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S39 legacy-a2r-resources-998 | ace-m1/legacy-a2r-resources-998.suite.ad | . | case-legacy-a2r-resources-998 | 2caf6b332c34ae0d875258bb77372b547f72599ba64514cedca7aef995779e9e / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S40 legacy-a2r-text-994 | ace-m1/legacy-a2r-text-994.suite.ad | . | case-legacy-a2r-text-994 | 1ac3620958cc8df6e3ceeb93df8e7fcdc98328afa18b199f36102f775b49e8d8 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S41 legacy-a2r-trap-997 | ace-m1/legacy-a2r-trap-997.suite.ad | . | case-legacy-a2r-trap-997 | 032e0a643bb68c129f67e99d4706f44a8f944364949256c82d6a662fd4afcddb / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S42 ace-m1-method-881 | ace-m1/method-881.suite.ad | . | case-ace-m1-method-881 | fa96026e1187c555d8cae2edc39609d8199bd467ed1604ca0378661a3a97b9cc / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S43 ace-m1-native-closures-924 | ace-m1/native-closures-924/native-closures-924.suite.ad | . | case-ace-m1-native-closures-924 | e9400afdbddb9d5164832eeeedfde20c6ec6cfad1f8a6f0909ef0b1374e81f19 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S44 ace-m1-native-collections-889 | ace-m1/native-collections-889.suite.ad | . | case-ace-m1-native-collections-889 | d9e9069086d3f9e1db9026388acaac3fa29b2d1b55939cf724a875ed50d0bc65 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S45 ace-m1-native-diags-911 | ace-m1/native-diags-911.suite.ad | . | case-ace-m1-native-diags-911 | 69646bf2ee2d22b160089c70aa295369bf0ecd10a8bf5ea843cde53e1b634844 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S46 ace-m1-native-resource-923 | ace-m1/native-resource-923/native-resource-923.suite.ad | . | case-ace-m1-native-resource-923 | f5c17dd21c05bc08d5944c8fb5a3473776c66915af5a15fab90d7c9dba760be3 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S47 native-warm-admission-932 | ace-m1/native-warm-admission-932/native-warm-admission-932.suite.ad | . | case-ace-native-warm-admission-932 | acef03af6ed3442c52888c9af993057b46b64d8ea05de6f57161731042db86a0 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S48 ace-m1-nb-diags-908 | ace-m1/nb-diags-908.suite.ad | . | case-ace-m1-nb-diags-908 | fa483e564ba28cf33de81ab65f94c914414bd5a1f007ebc57e60019e279bd532 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S49 ace-m1-order-layout-914 | ace-m1/order-layout-914/order-layout-914.suite.ad | . | case-ace-m1-order-layout-914 | 6cf8f0960ad31e3cc1fcb0330f2537395df970ffa6d7ccded8db818c2f33bbe4 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S50 ace-m1-resident-866 | ace-m1/resident-online-866.suite.ad | . | case-ace-m1-resident-866 | f4d8ebb9c439102110ba6190b4a7cf56f70be784d45a698357337e18a3e59bb4 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S51 ace-m1-resource-915 | ace-m1/resource-915/resource-915.suite.ad | . | case-ace-m1-resource-915 | 19c8e93a5fe9dfd691d18f82f3b93664ed83d4b87378c8676162b21606075bc8 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S52 ace-m1-resource-events-919 | ace-m1/resource-events-919/resource-events-919.suite.ad | . | case-ace-m1-resource-events-919 | d63242b0cbc3b191fa7d0d7852536d6c9c4a3798fabd57a53808f1a649c422dc / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S53 ace-m1-shared-mutable-909 | ace-m1/shared-mutable-909/shared-mutable-909.suite.ad | . | case-ace-m1-shared-mutable-909 | ad6be54aa0bd6cb0e61eb9d8a18433cf051f176d152b8ecfb0546c3d2271e0e4 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S54 ace-m1-source-composite-928 | ace-m1/source-composite-syntax-928/source-composite-syntax-928.suite.ad | . | case-ace-source-composite-928 | a7802f8ee9d66718f86910d5f8e3ca7397562027f31b47ecb7ece0c591068514 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S55 ace-m1-source-cte-936 | ace-m1/source-cte-936/source-cte-936.suite.ad | . | case-ace-source-cte-936 | 636be13658126c53b54304ac3c91c9eac07abd5d860a6030901cf6cfe438a0e6 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S56 ace-m1-source-instances-935 | ace-m1/source-instances-935/source-instances-935.suite.ad | . | case-ace-source-instances-935 | 1134232647d6f3daa97c00246459a4488eb27cbcb94b4f8248c0e6dadb953fc4 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S57 ace-m1-source-query-934 | ace-m1/source-query-934/source-query-934.suite.ad | . | case-ace-source-query-934 | 25ab8d5bc5e04020d8f58e960be842dea5adb78377f6c165ae7449176e789a6c / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S58 ace-m1-source-syntax-927 | ace-m1/source-syntax-927/source-syntax-927.suite.ad | . | case-ace-source-syntax-927 | 976dcd1f0ef7530006a4f56ccbf5b774a0306a26cc64d9b7693c8831776cd6fa / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S59 ace-m1-text-source-894 | ace-m1/text-source-894.suite.ad | . | case-ace-m1-text-source-894 | f01c231e9db1ded80f84af7c8a24ed5303a083b5b58ace3fd08c6fe354a91562 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S60 ace-m1-typed-native-891 | ace-m1/typed-native-891/typed-native-891.suite.ad | . | case-ace-m1-typed-native-891 | b34a5872ce2b4498f17fed4874fa75f468b8ea5a88d8ef23c81c771f12c59dbf / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S61 ace-m1-unified-source-929 | ace-m1/unified-source-syntax-929/unified-source-syntax-929.suite.ad | . | case-ace-unified-source-929 | 165bffd91e994f49ad843f6d7db254f6ba47021f3afe8b2c999f00b627731647 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S62 autoabi | autoabi/autoabi.suite.ad | . | case-autoabi | 2116e20447ac7acf045e8c674880be0fcef1d840745da839cd4f1088fa00a328 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S63 av-abc-825 | av-abc/825/av-abc-825.suite.ad | . | case-av-abc-825 | d0432246f745eecedda46b0327db9590718d0e1464ddb972ef9be19ca0b25d1d / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S64 av-abc-826 | av-abc/826/av-abc-826.suite.ad | . | case-av-abc-826 | 8372ebc79dacaf92fc461156d24cb59d5c9adc0fb436291c6e855186a93ee2e2 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S65 av-data-821 | av-data/821/av-data-821.suite.ad | . | case-av-data-821 | 196a3b76a10fcfa300b22b077b7d6b8289d4ae4b30b67286b119e4d7a953ead4 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S66 av-data-822 | av-data/822/av-data-822.suite.ad | . | case-av-data-822 | 8e8f1392cb6242a375eb24a5b95252a58f4bafc3786033553e47aff08720e513 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S67 av-data-823 | av-data/823/av-data-823.suite.ad | . | case-av-data-823 | 73b516ee1e0cfa42c22e1892dbf423ca4fbbb14b2c6136e56ecdaf88cccc4696 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S68 av-data-824 | av-data/824/av-data-824.suite.ad | . | case-av-data-824 | 2bb64d54ca2e7b382741eb41a60eba8d46dad9e0c888c8f6205b27d6f0f22864 / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S69 av-source-827 | av-source-827/av-source-827.suite.ad | . | case-av-source-827 | 1ea66b47f0fc1b0b68682a184f75219070defb848001b6e375832869a35d345f / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S70 governance-887-first-wave | governance-887/first-wave.suite.ad | . | case-governance-887-first-wave | ebd6455948289bd66214bed8b725e0463df8da58f807bbe62730790392d9803d / 44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a |
| S71 pilot | pilot/pilot.suite.ad | ../.. | case-pilot | e8000bc50dd24730496473f720cfc0edde28565f6980b60244bad11737787665 / 358a942e2e7306c509d744f07dcaa026a32d059145e62ac66c48c72051b4cd2f |

</details>

<details>
<summary>D：181个case文件原件身份</summary>

| ID | case根之下的跟踪路径 | 文件SHA256 |
|---|---|---|
| D001 | a2c-hir-collections-926/FIN864A.case.ad | 2ecddc564bd77621396090dc082bdae2fe356744177f215f3ecb0e807a6dd9fe |
| D002 | a2c-hir-collections-926/FIN864B.case.ad | 392e0af07ab463ff65ed117cb4de35c00b89fc3ff987ca5970b944c0dcd3dfaf |
| D003 | a2c-hir-collections-926/L01.case.ad | 188ce77bc54f173781a2758ed07d50420ac14acf6b49b194997c9a2a0802adff |
| D004 | a2c-hir-collections-926/L02.case.ad | aa275d719e893ae971923c6feb08b37da768786f629f98b311ecd7dcf1f279b6 |
| D005 | a2c-hir-collections-926/L03.case.ad | 56b2e9d6932b032f8c6d5af3ff6efde9efa8a399ed6a8da6efa1f5cb32f6c697 |
| D006 | a2c-hir-collections-926/L04.case.ad | d3065b50d38cfb13293b15be5fbb041ddd77a5ee861937257fe9571d29267031 |
| D007 | a2c-hir-collections-926/L05.case.ad | 83a33990443c1f6f7d8719d06b0d7ad32952a86decbff129f15afc0fb59f17c9 |
| D008 | a2c-hir-collections-926/L06.case.ad | 7acc1c622ec1ab7cc0df34b16a9036870f6b7ed08426bed683d8782ce42a6dfd |
| D009 | a2c-hir-collections-926/L07.case.ad | 48652d79d7ea1148feeb8493fcbbf3d1612234b25eb1782cc88a2d7face0d5f6 |
| D010 | a2c-hir-collections-926/LSTR864.case.ad | a189795c16200a10af9149e04c5dc66c46dbe89b392030c624235f7c76d3fa38 |
| D011 | a2c-hir-collections-926/N889-list-variant.case.ad | 71175f3639720c689a0c8747cc9450df8aabdbd658bc87eb2f79cda56a309183 |
| D012 | a2c-hir-collections-926/N889-nested-variant.case.ad | 7966564d0878dee1622a39c3a1adbf96a480bb78d34ba9cc0f0ea4054d46f748 |
| D013 | a2c-hir-collections-926/N889-optional-variant.case.ad | 15002212df625067bf5967779c84e5c17c074ede659ae8c8ef53ac968c16392c |
| D014 | a2c-hir-collections-926/V01.case.ad | ceeed0518500f9aa22e41869b3770e38578d41d1351e8c3eb0ad505c7fb9569e |
| D015 | a2c-hir-collections-926/V02.case.ad | 352c7d2b9197c3fa72114a31beaeb3e7542553e17fa674b41cfd3be5bbc85169 |
| D016 | a2c-hir-collections-926/V03.case.ad | ef5764ed75dc2da120c7ed846fbc507983801713b9b7aa05e45145f7a1cfcacc |
| D017 | a2c-hir-collections-926/V04.case.ad | 74417b4ee0d42591df4d65c06c170154efb097019aa9e59f80182ec4df8796e3 |
| D018 | a2c-hir-collections-926/V05.case.ad | 1c913c13d8a76912f5da09aa92b1866482112876cf3dff921e4a1224f68d4359 |
| D019 | a2c-hir-collections-926/V06.case.ad | de972d9de44ad9f21b6d2e1cf6fc803c467f68857f7f92955bd0d47c46f3f8a5 |
| D020 | a2c-hir-collections-926/V07.case.ad | 08d8cc9805a8b893630c083a494456ce8d5ea689c499da00b13fe25ef67d5b62 |
| D021 | a2c-hir-record-text-925/C04.case.ad | f566123e60705d39c6ccbd4f4472eada66aeaf703b272e3bb4f7f5f2cd43639a |
| D022 | a2c-hir-record-text-925/R07.case.ad | 7349611dbf454d9155b71cff815cf12eaeedf3e24d83daea2849e7f098876e1a |
| D023 | a2c-hir-record-text-925/R08.case.ad | f61d38b46efad187e0ec84b492c69657dfd18d888a52cdd1e4ce1b785d37f0cc |
| D024 | a2c-hir-record-text-925/T03.case.ad | f0b9eaf435bc20337ff24804b01f442513f787bb0f2a333e75b801003c2d9a34 |
| D025 | a2c-hir-record-text-925/T05.case.ad | ed64be722f88cd30e0da01947063fc7dd913a731be9bc9a04076126cfc8b51df |
| D026 | a2c-hir-scalar-921/C01.case.ad | 3b23d404d38d337e6103921b86d3e595b9e341eeef58d47a983f84a5b4df6d7d |
| D027 | a2c-hir-scalar-921/C02.case.ad | 89d38847333ddae88e8afd28bfb14b510087da40e24fec69f9035f0948c5038b |
| D028 | a2c-hir-scalar-921/C03.case.ad | 8cd86e577ad0f6bf119ac2a18ad097ac37ca8fcc0b62614349d8905dd3c51610 |
| D029 | a2r-hir-892/B01.case.ad | 0b5921d33e4b3d76d7b7df1c528d4a58beb9a320a12bc05a29fc23aa8b0dc76a |
| D030 | a2r-hir-892/B04.case.ad | 3cbfa91b4436be41b2d0d67a7b745139b298b35e10e8414c36cdf2c67510c19b |
| D031 | a2r-hir-892/B13.case.ad | ecbc485a08d88a92d63d25c86a93e930bbb473f66a8d33790789fbcf88f927de |
| D032 | a2r-hir-892/B14.case.ad | ae8f36ffc0335a444f3ab22394ec1a1463a56518f5e0e9c19435e05b428b1b51 |
| D033 | a2r-hir-892/NAMED-01.case.ad | c24d0d73f6a6fe19418e2b2e8908d663fd6ad0e1b736bf0fa1b617cd02f1c749 |
| D034 | a2r-hir-collections-904/FIN864A.case.ad | 694bf7bce02ceca59478c9a70713c5ceb3a79684e41d1794e4c4984226cad8cc |
| D035 | a2r-hir-collections-904/FIN864B.case.ad | 9d5cfcd4c28665bb949ccedc6983c86167b4d2952599d4c59dd84b1b95209e04 |
| D036 | a2r-hir-collections-904/L01.case.ad | b2f09d9f739e8405ae5557c70c4aa216cd5ef17ba29f8c3f83c5ac45c50fc9b4 |
| D037 | a2r-hir-collections-904/L02.case.ad | 5e74d58dc8beb15877f009166a0ef6037ec7a358467c13465677851c4e3bbb48 |
| D038 | a2r-hir-collections-904/L03.case.ad | b56ba07bf8fbb1c2c326a63216e5486eea26e66f0fe8b8235b499826298805cf |
| D039 | a2r-hir-collections-904/L04.case.ad | 809e271828a1dd7312c3259b926155bf068627c78de2d289edb124085698569c |
| D040 | a2r-hir-collections-904/L05.case.ad | b2e900d16a99b64b99860ed91cc905087bb92746f4d11190b70ff07625a6b216 |
| D041 | a2r-hir-collections-904/L06.case.ad | b7be49be35b9e464fde830db8d70c2b1da35ad81212bb4bdc72dc812bce86814 |
| D042 | a2r-hir-collections-904/L07.case.ad | 5cd836a09ec92a1973e31af6bac09585f3ae32cb6b8cb218903bdc2160f9a6fc |
| D043 | a2r-hir-collections-904/LSTR864.case.ad | 892c2607d7d571fb7fc77f6f8d0e47f075c9f691f346836cc0b0dfc5a651524b |
| D044 | a2r-hir-collections-904/N889-list-variant.case.ad | 927839d7dace7be4ee97177837e27f61698460d5000b83e2dfadcbb371b3d39c |
| D045 | a2r-hir-collections-904/N889-nested-variant.case.ad | bc976926a8ca0f1bc1671c64fdb0285e6bec3cb25f9ddc6f8627814042a35828 |
| D046 | a2r-hir-collections-904/N889-optional-variant.case.ad | 20b90c04e75d7a0f880fae82300b7b2a8ae10958ce7448b61a32dca3217d4795 |
| D047 | a2r-hir-collections-904/V01.case.ad | 34032ac75ea23d88adc60cd4c99759180ed463c9e0cd4257ecfe7b27505cc24c |
| D048 | a2r-hir-collections-904/V02.case.ad | 852fbabfb0b68146c67072a29cd9b401726d178fbce1bf1dbfa67af931a00fbc |
| D049 | a2r-hir-collections-904/V03.case.ad | b482936b1a50ed0891f38d7ab73289a2bc37dc6ee1e4801e0a2ecfd14aafe810 |
| D050 | a2r-hir-collections-904/V04.case.ad | 121922b0c24b736f87a390da1ee26ebc1aaf7997e557e0dcad7d583afb44e4af |
| D051 | a2r-hir-collections-904/V05.case.ad | da97e0f3ef4e37bcb76089284fbdcf0d0de5a7990bf5eeecee409d0fd344d944 |
| D052 | a2r-hir-collections-904/V06.case.ad | 51780a5f87c90fc64d2abb8ac5cfdb4a68ecf32aa18df47af566bbe0a0c31df9 |
| D053 | a2r-hir-collections-904/V07.case.ad | 9e7f00c361f8495079269ee2de0590f7b0a2710c8836c7bd992ce87746f9292a |
| D054 | a2r-hir-record-text-902/R07.case.ad | e153ca50b91b58edabd11373633080b46b093a561b1baaa050177ec2ac7274fc |
| D055 | a2r-hir-record-text-902/R08.case.ad | 1ce23de1dc275b098586e6301e8be70d1c38b821925bff4d66209238213b34d3 |
| D056 | a2r-hir-record-text-902/T03.case.ad | 9c895ce0a0794ba82a27699ff796e9353782572ac8299abbf04067c0dcdbc1a5 |
| D057 | a2r-hir-record-text-902/T05.case.ad | 7011b54d9f3eb49100f67834aeb57a2c80e67b1075003bc9229f63a062adad10 |
| D058 | ac-av/av-core-kind.case.ad | e7c7aeeb17d5bc96f610917d99d41b605bdd0dc862865e109b7e2b26759a9e03 |
| D059 | ac-av/av-core-semantics.case.ad | 466b0ec91ce145a74fa52ef522eb4639567b5555137cbb7a2ba2feea4e9767eb |
| D060 | ac-av/av-core-trace.case.ad | bede3e6604a684744ed3df47ccada71fbbe6419f01e833b1205793570acdfa02 |
| D061 | ac-av/av-core-traps.case.ad | 8695f42c13e9187d8e63b754ffde9a31f36ffe8697013b19d5ee458fca1dfbe0 |
| D062 | ac-av/av-core-vertical.case.ad | 9d8adb49ffca0098321077fd28e31f55d14971c89c17a61dcad056de81c5e3ed |
| D063 | ac-av/av-data-bool.case.ad | 9893d62c8828e34fbae91f66651a6afa4cb1269299a195ffd64f18fd5da18ca5 |
| D064 | ac-av/av-data-first.case.ad | 0b1cda199e204f35e2382d59773fa5677bbf2f6dba0a38c26aed6e012b1d5f34 |
| D065 | ac-av/av-data-int-zero.case.ad | e005bd2568b3426e8bd97e8f5dea7207e56d2f6afa4e8b9c220ac326e8c30b29 |
| D066 | ac-av/av-data-typed.case.ad | b5b78cc0f4de868d451bc9d521ba3b34e3835797d2ef0966d4de6f4b89a6919c |
| D067 | ac-av/av-resources-front-door.case.ad | eeae93e4d83cd094a2f79a4e8e90a024c9c6e8a1744b3a32849c929b8bc2f5f2 |
| D068 | ac-cte/cte-first-bool-branch.case.ad | 21d9d794518ae4d9f599fb02cad36077d5e80621a8c47cc68bcdfa305d782525 |
| D069 | ac-cte/cte-first-sum42.case.ad | aa2bfb53355918418ee2cfd92a4335c9174cbad8de953ee01c1ea52647506f25 |
| D070 | ac-cte/cte-meta-arg.case.ad | 1ef0132a07eceec04529a48574742c9811946bdcbc2faaa0cfea2005ec060085 |
| D071 | ac-cte/cte-restart-cross-process.case.ad | b8814e32ca56ace04460a3ef858c84f47c023ed9d0294b1814ef2dbe35d00d33 |
| D072 | ac-cte/cte-trap-diags.case.ad | 78d37a2fa737b56da1e71a2e52c8fd5d11e10070fcd3d80dcc5452ce8817751f |
| D073 | ac-cte/cte-warm-session-edit.case.ad | b59686e27a209cdc357a13981b5c311cf1f239dd9466a627b0b8da4ce54a5d4e |
| D074 | ac-mn/mn-div-trap.case.ad | 9d46251861b34f2020b35fe32885f4992cb6d7afb279cd358dc1d5eec7d0153f |
| D075 | ac-mn/mn-first-trace.case.ad | 6e0466987c85562927afb45345a25266dff68bef7145eeb6e1b480bcd3c88937 |
| D076 | ac-mn/mn-value.case.ad | c60328be22c0df2957d5b2019293205822c112c39fca86d072fb48aaa254389a |
| D077 | ac-native/core-run.case.ad | fbefcd485ea2b37d7d7e36bea1af44dddda73a2c5804aa653cdf9fc64bbbfeee |
| D078 | ac-native/hir-invalid.case.ad | 3301e646df3f48a4920f76c797bad247b38a3b405021b3bc90a060febb2a7cea |
| D079 | ac-native/hir-valid.case.ad | bd1a46a5c54768e00b297dda0da750f62567695208e30754a26add89ffae8948 |
| D080 | ac-native/red-controls.case.ad | 65044d8d4cc081afaf2e5787a87af30f0f763621379844920e52c2584a55e6dc |
| D081 | ac-native/scalar-run.case.ad | 1927fd209a92c90698c1bfbece3b309935209502ae2d16415dd22bbc3f7076fb |
| D082 | ac-native/src-invalid.case.ad | 263596fccc6772096c58fc0f4b851d16162083140c0207af2a460bfc9e2a6ad9 |
| D083 | ac-native/syntax.case.ad | df93e0b83ce90882204d440129121a6e293da7aec53fe62b607bb5f9f9d9d7d5 |
| D084 | ac-semantics/cache-roundtrip.case.ad | a6101c231b486ffea6219885882542f8d7dc43de657f4dff904e1e70048dea7c |
| D085 | ac-semantics/callee-sig-change.case.ad | 0f2b3e62b8ee46ad489605598bc53c5ca4992833fee0cb7525eff251ac29a431 |
| D086 | ac-semantics/infer-bindings.case.ad | ccb3562934830d1191e3656c59855c316cd3135f16d05a964ef94198fd399b2a |
| D087 | ac-semantics/parse-error.case.ad | 9d57d39ae6cebf147850b56e038e14feaa57fed8c7cc1882943c1919451581d3 |
| D088 | ac-semantics/turnaround.case.ad | c7074be32e1581e14e60d301ee18ec969c93233cd8cda5082008d5c72999bf30 |
| D089 | ace-ca-d-resources-939/resource-export.case.ad | 50afebabbe7c7ad52325e508ee001df0d1ec87bdfacb2776c85286bd9cdcf7e2 |
| D090 | ace-ca-d-resources-939/resource-import.case.ad | a1b779674b979600c23eadd03a430d0d7ef86552d84040562df30331b0765234 |
| D091 | ace-ca-d-resources-939/resource-recovery.case.ad | b30f141ce7ab97e4b38df544a3421d12ce59b98c321632d45a2eb5598491675a |
| D092 | ace-ca-d-scalar-937/bridge-legs.case.ad | 105da9938bdd54610f8c8ef6108f3d9522fd046e15c428bbdb430b43409f7150 |
| D093 | ace-ca-d-source-938/scalar-import.case.ad | 8878ebc8cd94753a40dd952089a461248f3f765038d56db981d7ca739e537b7b |
| D094 | ace-ca-d-source-938/source-import.case.ad | 04dce85fbc8b72b69f6e70fff327d0063ea1abc6ec43bf935869d6b17138fe44 |
| D095 | ace-m1-expanded/collections-q4.case.ad | f2c73aa5825d550555616b855fb59ff532d37f4ad634a398a534f669e86445ec |
| D096 | ace-m1-expanded/legacy-coverage.case.ad | d27785f50ca6a9338af993f92f20db6e7b4334c1ce1a86ec22bf6d9bd288c2bb |
| D097 | ace-m1-expanded/original-data-diagnostics.case.ad | 9d8c0daf594f3da8a2e3fd352e95f76abda4a1fdc88fda95bdf77bb52c06b547 |
| D098 | ace-m1-expanded/original-data-q4.case.ad | bd4bf0519eff0ec38a7dfb4e87980b8d6d3b020f66a32c901861d921597dd44e |
| D099 | ace-m1-expanded/q4-001.case.ad | 5cc3222b837738a09cf01071159249cf9ef0d2f6c7d89b42477c620b9be75fc0 |
| D100 | ace-m1-expanded/record-text-q4.case.ad | 2391fef28922a0a5ff8e720e11c862324c9828b42854f2f4154a1735d297ea68 |
| D101 | ace-m1-expanded/scalar-q4.case.ad | d22824714d5812c1272db9fa341d8d53d2e186ff61eb3778fac02f99f93ba75f |
| D102 | ace-m1-expanded/source-supplement.case.ad | e6cd3ebbb8bed778d68c77a4a9e8be9210de421aad0c79258a996aebd593ee69 |
| D103 | ace-m1-expanded/text-q4.case.ad | 0e3d71917ad2ecff056eb5168a964590c1fe47ab3a63ee8e71fb7c38017a603c |
| D104 | ace-m1-residual-999/ace-m1-residual-999.case.ad | 61b09fdc4137efc36c823d2dc3ec3d267761d485cc3a8ed257238690ef4bdb34 |
| D105 | ace-m1/a2r-emit-917/a2r-emit-917.case.ad | 398d9b518234143049d9526b85a9644c287f516e546beaf4af84de02e7bebf86 |
| D106 | ace-m1/a2r-owned-temporaries-931/derived-view-931.case.ad | 6747e28f09c73b924d20d624b4933a35d81c22113816084a2bc1c7cbf4335580 |
| D107 | ace-m1/a2r-owned-temporaries-931/modes-931.case.ad | 99a1000303abd2912a99d42485b3c0c18e8a6d78f81661dfacad20efa8d1978e |
| D108 | ace-m1/a2r-owned-temporaries-931/ownership-931.case.ad | fd718e3d66f1b03186c0afaec912747b16bdec79c04554c732732171d2f9c383 |
| D109 | ace-m1/a2r-owned-temporaries-931/short-circuit-931.case.ad | 6abb2f276972854891bdc926ba6a45aa86c8fb860456275d6ea8cb734efb6366 |
| D110 | ace-m1/a2r-owned-temporaries-931/wrap-931.case.ad | facf6192e85bff7d19ff0a0b2263a3c1b4ba2e6a6993c642997f06217715d7ba |
| D111 | ace-m1/a2r-semantics-916/a2r-semantics-916.case.ad | 8fc543a22c604f7e3a57acf98236edff6e1449a3f5714c9592b4bc2918308365 |
| D112 | ace-m1/b-diags.case.ad | 97552180ef19899273f5614c427d023247d6e1577f5b5842529e701578073308 |
| D113 | ace-m1/b-scalar.case.ad | 758863818ad3c5b501fbda65cecbbb6949e424bc9eb27eafa0c52df243f927ee |
| D114 | ace-m1/b-traps.case.ad | c3865c08e794f22cb2634a04bfb978ae2c2c978e1cd3cf39887ea76b4bc0bf87 |
| D115 | ace-m1/closures-fn-912/closures-fn-912.case.ad | ee0e4b19d72e62bbd9f7e65681d9a7db42e0916a610a908ac03af026e2d55118 |
| D116 | ace-m1/closures-recovery-913/closures-recovery-913.case.ad | 51fce8baf7da74ec3b67d99a9819532e0a52f20944e3ec73b7de5e1ccbe01c0d |
| D117 | ace-m1/closures-source-865.case.ad | 78cb9d50971401ff8293fe958be0c59f6a0f9e353e77d0ae2701e8e5404413f5 |
| D118 | ace-m1/collections-source-864.case.ad | 8880b437ed092c5dd2c1b90668433ccab8221aacb6ee3d33b65bf3558419522f |
| D119 | ace-m1/console-863/console-863.case.ad | aa519502ec2f563e6148f94a327aa4cbf8162e4b8ef127c826df934120aa78e9 |
| D120 | ace-m1/entry-862/entry-unit-862.case.ad | 013240d15cfd3977268050abbdbaac29dda658aaa8500856716168cc2bdd1476 |
| D121 | ace-m1/immutable-avm-920.case.ad | 96ef60067b3c1dbab1580ffa2cdb5d0433221011ca0d4ca0207b588f787e1e42 |
| D122 | ace-m1/legacy-a2r-cert-992.case.ad | c62673f31390044d92f2831a7994700d61d31686562cd22ca9f4797654e09631 |
| D123 | ace-m1/legacy-a2r-list-996.case.ad | cc9bcc7dcf911361111171e2eeaad78e99be7d3feae1e3dfcb96d033e03e6194 |
| D124 | ace-m1/legacy-a2r-resources-998.case.ad | ad9f0a06cbe77755af7e842e6116e37fb04f1f627fade7024154b515737296d7 |
| D125 | ace-m1/legacy-a2r-text-994.case.ad | 6b2853fcd946186aff4e59382ccfc356b796c7a72513e23c72caf6ebd6a2b6e8 |
| D126 | ace-m1/legacy-a2r-trap-997.case.ad | 029b9ef6944fa0347e0d8ec80e886ad4590c8d661b4326ac294567039e2e34c1 |
| D127 | ace-m1/legacy-composite.case.ad | c53da5c0b3ccf2942242841e2d862642563ce409fb60e64df63deef665846f44 |
| D128 | ace-m1/legacy-negative.case.ad | fc65ec71d387aa67b1dba3637cb80e3ff82a3c65fd6999c0710190273a1db403 |
| D129 | ace-m1/method-881.case.ad | 7577482cc84c64349668a96eca232c61b41e6a5eb32ebb1617915c99bd2afacb |
| D130 | ace-m1/native-closures-924/native-closures-924.case.ad | 7fc27081f57012ccb2eaae970f901ec618bad953172f063ec69815624058a815 |
| D131 | ace-m1/native-collections-889.case.ad | 9c3f8fff067919a9f7d04c282aeaf02b2b903ef93547d7412140ad68c93511d7 |
| D132 | ace-m1/native-diags-911.case.ad | be01ca54fe6602a0f2a9ea00112d9fa3a206a24ac6a9b7371715ddfe23ae44dd |
| D133 | ace-m1/native-resource-923/native-resource-923.case.ad | bd0f8f8a915714e702b4b8e3acd1ca5a3c925f9ce3070f783e7e156a4752587e |
| D134 | ace-m1/native-warm-admission-932/native-warm-admission-932.case.ad | b5c56798aa1a61b45f2c65301a4c4ef09935d888cdcc1f6682c9e3b75ec4ae1a |
| D135 | ace-m1/nb-diags-908.case.ad | dcf7401aa1cb0892d9eda86009b727e01ca1fabfb7e6d30a6e64cc0454d5cdfb |
| D136 | ace-m1/order-layout-914/E01-E02.case.ad | 67216c727df29313af754bd12c27d2f13956ad7fdf49b8f880106f8ff0acf7fb |
| D137 | ace-m1/resident-online-866.case.ad | 4c65887d4da9f4499f6b0e1b71878f236d0bee810882d305dfc47ed8eb2b144b |
| D138 | ace-m1/resource-915/resource-915.case.ad | 4d9e43bebf0f5992463a1adf17259c6990d097fa3f5a250f531fd58db3d5f28a |
| D139 | ace-m1/resource-events-919/resource-events.case.ad | 0ca6fdbd5fa0d9e32ad4514f4e3f4e8864bc32ad3f064f66e46b858736c01ae3 |
| D140 | ace-m1/session-lifecycle.case.ad | 786d105a339d03dd7e474b0c875b422ef6545e8b3994d256cdda694f9d9f3db5 |
| D141 | ace-m1/shared-mutable-909/shared-mutable-909.case.ad | 35c804a5c9208eb8c7c12828441333e7b20a1be48e4bb163e999ae4c745e2a01 |
| D142 | ace-m1/source-composite-syntax-928/source-composite-syntax-928.case.ad | 9fae4b239ae85554b9d393cc5ee272eb374cbc6bf068f218af64168e213ac1de |
| D143 | ace-m1/source-cte-936/source-cte-936.case.ad | 068afa3fb4bca2c80ff5a08cd7b7564572b0c090c8f744ca3da0c8f4509a5ea7 |
| D144 | ace-m1/source-instances-935/source-instances-935.case.ad | 006bdeae4b809cc87468de671de6c09cd2fd8f05e69d5c2e4a3a305a99d03dad |
| D145 | ace-m1/source-query-934/source-query-934.case.ad | a1520f65825474bb96d6b73d4cfd0c3973a4735b317a7809175d8fa3f27d8dc1 |
| D146 | ace-m1/source-syntax-927/source-syntax-927.case.ad | 826a507ddd41ee5ba70dc7eb7613377c8ec92cb3a75c2b3fcb1e4dd92a4f4f91 |
| D147 | ace-m1/t-r-native.case.ad | 80a532ee3890d37f0d3a7154e850b97f300a70e4a077925fd15d8cdc5b97f63f |
| D148 | ace-m1/text-source-894.case.ad | 9bc45d99308c8f4f5ee8ccc0103b709d87c5fa7cbe227de307c2146a4b595851 |
| D149 | ace-m1/typed-native-891/typed-native-891.case.ad | a0c6e26f87dd6fb7baac8e232423e7736102ebf4283cc42c8c20f0c1ad57dd25 |
| D150 | ace-m1/unified-source-syntax-929/unified-source-syntax-929.case.ad | ea84675aa212625dc6e86035f92cca7a29bfb7bed59704f66cc08b852ad0a0bd |
| D151 | autoabi/interop-legs.case.ad | 21e29ba8b34b9febef8343ce785b367986fdb9cf6c80b1ab5dbe43e5a52064e5 |
| D152 | av-abc/825/av-loaded-b01.case.ad | e0a4f756a505beba9add11bc828a0c56e7779424054dc5ed7ce9e0ed8970ecf6 |
| D153 | av-abc/825/av-loaded-b04.case.ad | f187e6171cf080da232358746c36bfc6831a9853367c96eb0b093da8e109937e |
| D154 | av-abc/825/av-loaded-b09.case.ad | c841cd622679808420968a2510626d8fb1d7e5b84a6693d2d87ed9cbc759a620 |
| D155 | av-abc/825/av-loaded-b13.case.ad | df074ebd791f408183ea1f4c0112cbb9a3346191bf30f07012885182869000fe |
| D156 | av-abc/825/av-loaded-b14.case.ad | c2d705ccc8367f1e9c0182c2eddd41693c5037957948e30c78368ec4c9a99db2 |
| D157 | av-abc/825/av-loaded-first-case.case.ad | a60c5c70f1c74e91baf3730b1624656548798adc2ce50086208f947ae761a055 |
| D158 | av-abc/825/av-loaded-lifecycle.case.ad | cf847d96eb6c8cb58aedbc6ffaf56b54e02899671a76f9ad06cb0e778a7d14b0 |
| D159 | av-abc/826/av-compact-first-case.case.ad | 8152c61897274495354b7a00e5f3cfba71d2e77acedf867954d6a7e56dd08f5f |
| D160 | av-data/821/av-owned-admission.case.ad | e28f20deac1df31479d69324386883d44e0aac4726a31aecc85d52d5b675a93e |
| D161 | av-data/821/av-owned-borrowed.case.ad | 96a90a2991089ed90f82798d8fa7d2d7ce1b6bd06710dba8d60a4b4ddc3fd8d3 |
| D162 | av-data/821/av-owned-first-case.case.ad | eb76ed65af4b63a18a193fe83b568210fe0222609f65f2eaa9e5a3c16cd04b57 |
| D163 | av-data/821/av-owned-first-twin.case.ad | ba3217720cd3b88be1a7f10a633bfc2ae9a2cd00c638cd40d065439c509346c8 |
| D164 | av-data/821/av-owned-mut-writeback.case.ad | b5ef415f31b636fffead5e42483ffbfec08fb9c3fbc46972fcfb9cc9be47ada1 |
| D165 | av-data/822/av-collections-first-case.case.ad | 9a30de9a1bb064acdb08d4262abba3f8309d7c18476a1a0784c4df08e029456f |
| D166 | av-data/823/av-closures-first-case.case.ad | 8bfa9109189cf00eb8265343a59d45664ae218c28688948cbcc519aee7b0e2be |
| D167 | av-data/824/av-dynamic-first-case.case.ad | 85f946cf1ac995fc56a065d129794ca96a7cd22154040d60923af9cea65ce47d |
| D168 | av-data/824/av-dynamic-take-probe.case.ad | 5323cfd3acb4eb2c797e995a880498699043f76958998ef999bd18eda5c039a0 |
| D169 | av-source-827/av-source-cache.case.ad | 86500bf9f68934983e69ed8f2bef8f10e0c797f405cb95b0083401791162c2ac |
| D170 | av-source-827/av-source-first-case.case.ad | 5902910fe5152990f89f872591f685c1d2d2505e33d69e49db896d333e3bb4a1 |
| D171 | av-source-827/av-source-semantics.case.ad | d56172f494152ebebd59b20ea837971ac74e6fefd69838b4c8f6605eb1c797cf |
| D172 | av-source-827/av-source-unsupported.case.ad | 74750ae1d09cc7ccd28bee04a9fbf6854f67f1f6e2f1eae28fc568289439866a |
| D173 | governance-887/d-arithmetic.case.ad | f524dfb547f4d67fc831b58048cf5a3157ba5c58a8c04d1c94a514b0a460d91e |
| D174 | governance-887/d-group.case.ad | f03138aa0c2377ec084e90551b6a97fb60e43ef93d66f1f647255652c5d51ac5 |
| D175 | governance-887/d-unary.case.ad | 042c1a4df90e480374ef18111ad5097913b1ec6be73f54e81ce1dd27fa468f4f |
| D176 | pilot/a2r-basics.case.ad | 867d4d3f005faec1c7d442820e8baa9109e56c14662d775116346b437cc73b1b |
| D177 | pilot/a2r-cabi.case.ad | 0873c8d99c77e5e728d936da76544cea9ee629f82697e60c8b2703be95b81acd |
| D178 | pilot/a2r-project.case.ad | 79d88be47a482f30a7fbb8b901a5015d8b393dcd61c3f8246642635704614d63 |
| D179 | pilot/vm-basics.case.ad | 6c9e294010c109e44b123d05aadb84e1df9ca05314eaa148ed2177e0c1a3a19a |
| D180 | pilot/vm-script.case.ad | ea55de17bd8b29c80f9e4980eafd8de39267a44aaffcf009016c0a4194d9ae92 |
| D181 | pilot/vm-shortcircuit.case.ad | 77ebeb190c10d15913a976ae54c4312f87ebd8fb25b562eae047455c7fd7e1c0 |

</details>

<details>
<summary>M：1116个case/check逐项保留映射（含全部511项原目录）</summary>

| ID | 文件:围栏行 / suite | case/check（原样保留） | R路线 | 独立预期通道（摘要） | 复算source SHA256 | check声明审计SHA256 | 旧目录归属 |
|---|---|---|---|---|---|---|---|
| M0001 | D001:10 S01 | a2c-hir-collections-926.fin864a/a2x-emit | R10 | "success"; artifacts=4 | cb7d63e1356a05909eaf9a32b069192ce0fa5590da6c34f07b872475ef8474f4 | 852e63248d7f6ff87948aaac1587a0c9518202c207bc49037d06beea76f87ba8 | 原511 |
| M0002 | D002:10 S01 | a2c-hir-collections-926.fin864b/a2x-emit | R10 | "success"; artifacts=4 | 6dc3d3af1778bb4f228ed354202756fc65a9aec8fbc1bce06c015e33f5b2be97 | f6ae450be91e98ccda31a1a1d4bc441e14d7177c4ff64710b5550b764399f994 | 原511 |
| M0003 | D003:10 S01 | a2c-hir-collections-926.l01/a2x-emit | R10 | "success"; artifacts=4 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | 9d75078db60993aeceac39f5526246ae0ed48361d1076ea4a640e987430aa21a | 原511 |
| M0004 | D004:10 S01 | a2c-hir-collections-926.l02/a2x-emit | R10 | "success"; artifacts=4 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | 0e35e80e0937c19a9394445392ffc8f8743148dbd3eac772ec8915a0baf0796e | 原511 |
| M0005 | D005:10 S01 | a2c-hir-collections-926.l03/a2x-emit | R10 | "success"; artifacts=4 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | 51ace7ca8bf0aa0102059ed049e8e9ab3d7a28e13aea630bfaba971f9ab41b04 | 原511 |
| M0006 | D006:10 S01 | a2c-hir-collections-926.l04/a2x-emit | R10 | "success"; artifacts=4 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | e015469f3cddb6909bc0467a56c9ec964e3f857c9673c3862dfd9092c99fc882 | 原511 |
| M0007 | D007:10 S01 | a2c-hir-collections-926.l05/a2x-emit | R10 | "success"; artifacts=4 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 8dda0b23324123eca2acd1b8cf0ba06aad8b633c8bccc95b8952e6e1c3786188 | 原511 |
| M0008 | D008:10 S01 | a2c-hir-collections-926.l06/a2x-emit | R10 | "success"; artifacts=4 | de3a85627019b586ef8b9ccb377e60ef57a0686f15076d57e93910f366b5a884 | ad107fa55343d3e005678c3f7e7fa70992179cacb021066f1b917ed487626c95 | 原511 |
| M0009 | D009:10 S01 | a2c-hir-collections-926.l07/a2x-emit | R10 | "success"; artifacts=4 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | 7ee08cd83239dd1ef511f0e19b365c0718847252090a82cf6b3e5eaac36a25a0 | 原511 |
| M0010 | D010:10 S01 | a2c-hir-collections-926.lstr864/a2x-emit | R10 | "success"; artifacts=4 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | adab0b21d744789358fde7bc4df65f736050ad272406d1a90d1a4f669009ca2d | 原511 |
| M0011 | D011:10 S01 | a2c-hir-collections-926.n889-list-variant/a2x-emit | R10 | "success"; artifacts=4 | 6f3db730b45eb6535914a3d7de9dfe7d9fc5ac7ca022889e37fd57b4eb810192 | 6dca520a2892f3d1184686ac51467322b0209698f78f3c1905c85de0ec75090e | 原511 |
| M0012 | D012:10 S01 | a2c-hir-collections-926.n889-nested-variant/a2x-emit | R10 | "success"; artifacts=4 | 47389d8fd49b7317503b021b10d9158062cb61f0e6f1c7d0f8de1774fa4a7851 | e2a509b526809aa22d478f808ce885bee0314d9fd0a7b343e6ecf38ee5bb06ed | 原511 |
| M0013 | D013:10 S01 | a2c-hir-collections-926.n889-optional-variant/a2x-emit | R10 | "success"; artifacts=4 | 357cdbe78fee58152ef74221bfbb5493f78b6ea1e7e7194538640d71b807574e | 500c534a0671cd2e3a779ed7ac7e7886235037c0ae6e0160e3e0a1b841145f3e | 原511 |
| M0014 | D014:10 S01 | a2c-hir-collections-926.v01/a2x-emit | R10 | "success"; artifacts=4 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | 13b57a8cd595d522434591216234d6d8c23ea49c1d76d5486c314084e94e5792 | 原511 |
| M0015 | D015:10 S01 | a2c-hir-collections-926.v02/a2x-emit | R10 | "success"; artifacts=4 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | 41c858d998e43211b4235ecb939768586a8caaa9584ecf65e1c31119ad606064 | 原511 |
| M0016 | D016:10 S01 | a2c-hir-collections-926.v03/a2x-emit | R10 | "success"; artifacts=4 | 41767e7e7046efc75d06d790e6392a4da4260baa6b95e8e244964f1883383f97 | 8809947e78520ea461d40cbad180b2ba31f05fea5d6f1ce29815104ba1162c01 | 原511 |
| M0017 | D017:10 S01 | a2c-hir-collections-926.v04/a2x-emit | R10 | "success"; artifacts=4 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | 9327defd17472b4488d05096692570c106b31c525d6f6a0feab86c7d500e502a | 原511 |
| M0018 | D018:10 S01 | a2c-hir-collections-926.v05/a2x-emit | R10 | "success"; artifacts=4 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 58cea0f3223bcddcb99fd6faed518352a680862168cf8e8cf1c37cc61abcd9eb | 原511 |
| M0019 | D019:10 S01 | a2c-hir-collections-926.v06/a2x-emit | R10 | "success"; artifacts=4 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | 64ee45370a046dde6eb14df584c514060e68a323e10a1cfd258658600011e072 | 原511 |
| M0020 | D020:10 S01 | a2c-hir-collections-926.v07/a2x-emit | R10 | "success"; artifacts=4 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | 71766a2da03ae3aac37313c14fd3a63123b02264650bb01d3c8e7cc8bdbf2558 | 原511 |
| M0021 | D021:8 S02 | a2c-hir-record-text-925.c04/a2x-emit | R11 | "success"; artifacts=4 | 05c593750e21779409dc23b0a20f6aa5b7ff320092f890831c9a16fc8d7ef89b | 21f8421593e3a951c2d6fd3b28cb2bf73f13b7ace7da30ad9c4863495923d85a | 原511 |
| M0022 | D022:8 S02 | a2c-hir-record-text-925.r07/a2x-emit | R11 | "success"; artifacts=4 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | 4ba14fdfb315b406cde5d2abd5a680e5c19d812430dfb88c8ace8712b0ad1cc4 | 原511 |
| M0023 | D023:8 S02 | a2c-hir-record-text-925.r08/a2x-emit | R11 | "success"; artifacts=4 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | c329c8353eb95fcc4dbd55d70649262a097f1dfccbf8f940d46013b9c83dfd75 | 原511 |
| M0024 | D024:8 S02 | a2c-hir-record-text-925.t03/a2x-emit | R11 | "success"; artifacts=4 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | f8c0e2a5f30ab6f594c67528a72ba75ad83823139dc173713e1d0b2f6320b57c | 原511 |
| M0025 | D025:8 S02 | a2c-hir-record-text-925.t05/a2x-emit | R11 | "success"; artifacts=4 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | a31d46decf2742f19ce0fce804e3a5cb2bbf67b9122523a3473c9f42d78620ff | 原511 |
| M0026 | D026:9 S03 | a2c-hir-scalar-921.c01/a2x-emit | R12 | "success"; artifacts=4 | d0b8340d655dbabd00acb92b0c9540c7b173a3d0ecb11058050b674b573be2eb | ea7dcfbacd95ac79da856b4bfe85dd81463dde569a1a94af28e4ceaa0cdd6369 | 原511 |
| M0027 | D027:7 S03 | a2c-hir-scalar-921.c02/a2x-emit | R12 | "success"; artifacts=4 | a7ee74538df550ad40fb570a0fef5975ad126a04f9c6132e4d57755a7214a647 | a4ac50846b03425d689d2c9a102849a0c7738655acac4cc5a2cc4e2e37ebdab0 | 原511 |
| M0028 | D028:5 S03 | a2c-hir-scalar-921.c03/a2x-emit | R12 | "success"; artifacts=4 | e9aa2f0821e031b4af32718e28f94329c0bcb037335fc8f1e497373ccfcdac2d | a29bb2f0c31e12454ecd3373510bcafdaf55650983a28ca8644a4b34d8700cac | 原511 |
| M0029 | D029:11 S04 | a2r-hir-892.b01/a2x-emit | R15 | "success"; artifacts=4 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | a7cd18d429220a9511aaa9ec38bc0b01377fe92e347d348e79511bbc1031eaae | 原511 |
| M0030 | D030:6 S04 | a2r-hir-892.b04-shortcircuit/a2x-emit | R15 | "success"; artifacts=4 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | 229ea5bec4ff24c1341d3db6519db0a5e014e018f361a32a42c18e920e4dc827 | 原511 |
| M0031 | D031:6 S04 | a2r-hir-892.b13-overflow/a2x-emit | R15 | "success"; artifacts=4 | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 5810ec67dfd161de61b64cf6b45ab16731b17e8884f5aff69c86359db95eedb5 | 原511 |
| M0032 | D032:6 S04 | a2r-hir-892.b14-div-zero/a2x-emit | R15 | "success"; artifacts=4 | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | 8152a290a195752af65adc95da0dfca6f22404f8b12facf38d25b9544766c162 | 原511 |
| M0033 | D033:6 S04 | a2r-hir-892.named-01-source-order/a2x-emit | R15 | "success"; artifacts=4 | d2dfe05a18ca97dfa1736a9af7ae7a4220c7f0f137b2f5b610ed67f4cd791230 | f1233ac946058a750a387fe75d477ccee4ff8cff802049c019bafc2d098fff4f | 原511 |
| M0034 | D034:6 S05 | a2r-hir-collections-904.fin864a/a2x-emit | R13 | "success"; artifacts=4 | cb7d63e1356a05909eaf9a32b069192ce0fa5590da6c34f07b872475ef8474f4 | d235c66fb315eb4ba915e149ee80602183623374a65d003a5fa93b03008289a8 | 原511 |
| M0035 | D035:6 S05 | a2r-hir-collections-904.fin864b/a2x-emit | R13 | "success"; artifacts=4 | 6dc3d3af1778bb4f228ed354202756fc65a9aec8fbc1bce06c015e33f5b2be97 | 823a2cf608858b12302b73f8d4bd0e10648a3a6a1c231e234d6d59d0007a890e | 原511 |
| M0036 | D036:6 S05 | a2r-hir-collections-904.l01/a2x-emit | R13 | "success"; artifacts=4 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | c36567724808d39a307f222e12c10c6eb353035447000fa50af9b6ceeb734061 | 原511 |
| M0037 | D037:6 S05 | a2r-hir-collections-904.l02/a2x-emit | R13 | "success"; artifacts=4 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | 05c8dce8afbc784acd0358c8316ae1089b5af264e96e0b91f272a4541afb45f9 | 原511 |
| M0038 | D038:6 S05 | a2r-hir-collections-904.l03/a2x-emit | R13 | "success"; artifacts=4 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | e1e2f863693accc5b4420e15b0dde7ba8669494b8bf410d4bd222d5eb3a3f47a | 原511 |
| M0039 | D039:6 S05 | a2r-hir-collections-904.l04/a2x-emit | R13 | "success"; artifacts=4 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 446e98546a4ffa00508ea7f63b6eab00fd7b14b4e0d1b2785c6ea9f625547450 | 原511 |
| M0040 | D040:6 S05 | a2r-hir-collections-904.l05/a2x-emit | R13 | "success"; artifacts=4 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | dd929cfba11afe2df806d7a2f2a881bdc08734d4b76adcf4a24c54ad2b88d36d | 原511 |
| M0041 | D041:6 S05 | a2r-hir-collections-904.l06/a2x-emit | R13 | "success"; artifacts=4 | de3a85627019b586ef8b9ccb377e60ef57a0686f15076d57e93910f366b5a884 | 6c0e866b5a3f731bd04c5eb1c9c5c3e0fb853755ad097d00d990bff1916f8ebb | 原511 |
| M0042 | D042:6 S05 | a2r-hir-collections-904.l07/a2x-emit | R13 | "success"; artifacts=4 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | 6a0038d68d27ca3c30e0b81754ea917ee7d73f40a7c776bf23fce33fafde0aa5 | 原511 |
| M0043 | D043:6 S05 | a2r-hir-collections-904.lstr864/a2x-emit | R13 | "success"; artifacts=4 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | ed05552d21b5b5f23a5ca2edfa66e19e379127fe9c389b60fb64ea103ce8a7dd | 原511 |
| M0044 | D044:6 S05 | a2r-hir-collections-904.n889-list-variant/a2x-emit | R13 | "success"; artifacts=4 | 6f3db730b45eb6535914a3d7de9dfe7d9fc5ac7ca022889e37fd57b4eb810192 | cf6233bcbe54cddaf91ec804df04770a28f43df821c849b3d17bd968f8b349ca | 原511 |
| M0045 | D045:6 S05 | a2r-hir-collections-904.n889-nested-variant/a2x-emit | R13 | "success"; artifacts=4 | 47389d8fd49b7317503b021b10d9158062cb61f0e6f1c7d0f8de1774fa4a7851 | 2ea255f599508263a7ccac34166db00a199640995e75043785e132eda04ea587 | 原511 |
| M0046 | D046:6 S05 | a2r-hir-collections-904.n889-optional-variant/a2x-emit | R13 | "success"; artifacts=4 | 357cdbe78fee58152ef74221bfbb5493f78b6ea1e7e7194538640d71b807574e | ff42cd555224e80262652d0cecdd7a45b1c9d5a7833e88b9da9c6a13fdd4efdb | 原511 |
| M0047 | D047:6 S05 | a2r-hir-collections-904.v01/a2x-emit | R13 | "success"; artifacts=4 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | 5a08e39f2825e541f2b5660fe739ef165927ac8d9c1443880a1e310801a1dae7 | 原511 |
| M0048 | D048:6 S05 | a2r-hir-collections-904.v02/a2x-emit | R13 | "success"; artifacts=4 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | a048d1ecd942fbe0aaa44fa763c637d1aa18478cc85ec87f84986e92e9781547 | 原511 |
| M0049 | D049:6 S05 | a2r-hir-collections-904.v03/a2x-emit | R13 | "success"; artifacts=4 | 41767e7e7046efc75d06d790e6392a4da4260baa6b95e8e244964f1883383f97 | 2c97d66c3faa4862a23c8d30f7ca20f6c7ecf7305f7a2e31a54c114f78f5933b | 原511 |
| M0050 | D050:6 S05 | a2r-hir-collections-904.v04/a2x-emit | R13 | "success"; artifacts=4 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | 6bb2380899f3a745ca51839b65354ba8563308a585bc3a4098da0d6a04e73147 | 原511 |
| M0051 | D051:6 S05 | a2r-hir-collections-904.v05/a2x-emit | R13 | "success"; artifacts=4 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | a0c30382860d4a213b60616686e500b02e2106d1844280215710f4965c51a6ba | 原511 |
| M0052 | D052:6 S05 | a2r-hir-collections-904.v06/a2x-emit | R13 | "success"; artifacts=4 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | eb7813e8d7383c20987eb35c6092b511918fce064e3bf5fcea64206a09fcea35 | 原511 |
| M0053 | D053:6 S05 | a2r-hir-collections-904.v07/a2x-emit | R13 | "success"; artifacts=4 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | 50904a9d4e1beb0b308c5bfcb85f7bef0c0622396260448a8e24f18766888f8e | 原511 |
| M0054 | D054:10 S06 | a2r-hir-record-text-902.r07/a2x-emit | R14 | "success"; artifacts=4 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | cf8b0f156c62fb974bd60cd60a91a1a69b3bc2be97f4d5724b1505806d95bb4e | 原511 |
| M0055 | D055:7 S06 | a2r-hir-record-text-902.r08/a2x-emit | R14 | "success"; artifacts=4 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 701274c784e5cb656fd2daee35fa12da59c30602024d3e1864115f0b9a3369eb | 原511 |
| M0056 | D056:9 S06 | a2r-hir-record-text-902.t03/a2x-emit | R14 | "success"; artifacts=4 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | 3c25a18600af57250683306b16477e77a40ae451e7bd65ea4ea527284000ccec | 原511 |
| M0057 | D057:7 S06 | a2r-hir-record-text-902.t05/a2x-emit | R14 | "success"; artifacts=4 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | ac8c91041751b65442fae7a432af97f4f580f66c28ed34f460a8f1eeb0050cf3 | 原511 |
| M0058 | D058:9 S07 | av.core.kind-selfcall-module/run-kind-fresh-then-hit | R24 | "success"; exit=42 | ee3a75ce508eca57b2bcee1d9b3ea15ecc3a00a95d8f74c6ea223dedab2c0222 | edcbc29759aaa1fa06b043ee76a2f15e880199c9d62b3f7c721384bbec249e3f | 扩展/其他现用 |
| M0059 | D059:4 S07 | av.core.shortcircuit-latent/run-shortcircuit | R24 | "success"; exit=0 | 2c6337fd7ff920a4368949bcd699f8ebb5a630dbd19a0b9bdc33065bf032385b | 9752574f9b07706a74f4a35d9fd335e492865cf846419a8cb0556968a0bc5d15 | 扩展/其他现用 |
| M0060 | D060:6 S07 | av.core.trace-order/run-trace | R24 | "success"; exit=1 | 75ebaf9e5c09580f798548055a51a7a5ad1914d3761d9e338e62cc1eb6242b10 | 234af9b0dcd5b3bd606ed4d99fa0755190544c7efd241ce904ac0ef5dc38a10c | 扩展/其他现用 |
| M0061 | D061:4 S07 | av.core.trap-rem-overflow/run-trap-rem | R24 | {"stage":"run","message":{"contains":"av.trap.rem-overflow"}} | c3aeb5adfdf829c2e543ef03ebd8fd4e50b0f38ee681f9427a4f5bb07140a877 | 4de0abf01b23274ce5dd691919d5af458746776a51e7b77b5e6dace6d99b37b9 | 扩展/其他现用 |
| M0062 | D061:24 S07 | av.core.trap-div-zero/run-trap-div | R24 | {"stage":"run","message":{"contains":"av.trap.div-by-zero"}} | 53319a9c1e6076134ed4e6620406d8f3c0e1de4f502381d47f4ff3722fd19bde | cb92dac47b08067622c358e3a0e43db207cdcc876ddcbd5bce2155844c9e5dc3 | 扩展/其他现用 |
| M0063 | D062:5 S07 | av.core.vertical/run-vertical | R24 | "success"; exit=0 | 3347c7713fc298f1afefb0e2974c7c2ed7e97361da3e116b36af3601fcd74c1a | 5c509713167b7057a2ae9c80aa27d5a09a3f6ceecc91bd99eae443e7b9cee8d1 | 扩展/其他现用 |
| M0064 | D063:6 S07 | av.data.bool/run-data-bool | R26 | "success" | 7cdd79570e8b1e27f69a185524a5a6d643855778d7641f640893f8b2f90a1a32 | b6a8bac7a3deba7e212ecf6d8b435df9abf851114a350094d8844c10042d25a8 | 扩展/其他现用 |
| M0065 | D064:8 S07 | av.data.first/run-data-first | R26 | "success"; exit=42 | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | 27bedc0e9609741311e1402d389a66cf1816a61809d141dd98fc1a20d82b53e0 | 扩展/其他现用 |
| M0066 | D065:6 S07 | av.data.int.zero/run-int-zero | R26 | "success"; exit=0 | 3c85805d127fa923bd40c9490fad2e58ae7c9c3eefbb5895d9e91aa66911bec2 | 1cbfa12442cf0e8d850a95e17b179cf86feefffe8aa439f31291ea66cbedfc78 | 扩展/其他现用 |
| M0067 | D066:9 S07 | av.data.typed/typed-bool-true | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | b3e65b53ee8bbe44ccc85fe208687a59c50da29ad28279a7ed7252ffddb151ee | 扩展/其他现用 |
| M0068 | D066:9 S07 | av.data.typed/typed-f32-nan-payload | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | 382c31d8617f490dd88d1c200265048e8289801c8bdd6728d7513240b56a245d | 扩展/其他现用 |
| M0069 | D066:9 S07 | av.data.typed/typed-f32-one | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | c3a742f6463ba89a9007d5b393bb6493af255c727163294dbdab4db513d7b29a | 扩展/其他现用 |
| M0070 | D066:9 S07 | av.data.typed/typed-fneg-nan | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | 59710a1203faed45b7e607894baef41f81ef3274839662752040245226b46650 | 扩展/其他现用 |
| M0071 | D066:9 S07 | av.data.typed/typed-i32-42 | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | 1fcfa00af1709eb0e397a43e8b3413c65593f89ab38886c58c9eb03596616fe2 | 扩展/其他现用 |
| M0072 | D066:9 S07 | av.data.typed/typed-i64-min | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | 931c8e8796d71b1461e0153e824973aa5892c7c729defc294905cc5857a2b581 | 扩展/其他现用 |
| M0073 | D066:9 S07 | av.data.typed/typed-neg-zero | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | 464a942881131930fe89f0b14cde60e440ac9f71209da68fbd0f6c5860d87b9c | 扩展/其他现用 |
| M0074 | D066:9 S07 | av.data.typed/typed-u64-max | R26 | "success" | 21a6cd6e3b917ac626bffc4d6b279b1004ebc1ed25edf7ffb2749534b9151a27 | dc68b623f2bbfcf8e46951cafc1ff1d497bdd95f1d74744b1a25b1496d513d97 | 扩展/其他现用 |
| M0075 | D067:7 S07 | av.core.front-door-nesting/run-nesting | R24 | "success"; exit=1 | 0a184c9ee61cb65c7f14c8baa89ed7404851710f3a5efb4b7427d540c598184f | 8fb08f0977083d2dec0283f455a4e030cb3f38a96702d3e7af71a0a615803729 | 扩展/其他现用 |
| M0076 | D068:6 S08 | cte.first.bool-branch/query-bool-branch | R39 | "success"; exit=1; probe=1 | 7004ae5bd70ff872f184d248d52adb1aacfcb4a0e87b4ce1e542f8bdcda299eb | ef79ac697bf038987b4c989a27c6d4a9b068c0c56f3e2c0876df9bc1bb09e54c | 扩展/其他现用 |
| M0077 | D069:6 S08 | cte.first.sum42/query-sum42 | R39 | "success"; exit=42; probe=1 | 60af9409e305d0adb5202506e965fb2b2aff912862077c6cd4a11c37baa4cec1 | 77b661714de8c23f53971b4ed5a36741f3f98d8fc0127470bbf0b16f2be028a5 | 扩展/其他现用 |
| M0078 | D070:8 S08 | cte.meta.arg/query-meta-arg | R39 | "success"; exit=300; probe=1 | 3941216eb743d560016297f4bca09cc56e8e3f5fe318c114f256df552923283f | 14cc3820e831ace73a1010bcc6b8647280da03e61ed897b2b67212cda5c209ea | 扩展/其他现用 |
| M0079 | D071:6 S08 | cte.restart.cross-process/query-restart-hit | R39 | "success"; exit=42; probe=1 | 60af9409e305d0adb5202506e965fb2b2aff912862077c6cd4a11c37baa4cec1 | 84f39094e2c0c6b2de15ba429febbc28d48327810603c682bfc9eede03c24b1b | 扩展/其他现用 |
| M0080 | D072:5 S08 | cte.trap.overflow-diag/query-trap-diag | R39 | {"stage":"run","message":{"contains":"cte.trap.int-overflow"}}; probe=1 | b66400ff66b48b1898ef1afd952254d9f3ac77a7a6ee5b0e0d26a19504bdcd4e | 4db6ed2d5c4a6e1b15f4038f85a66446ed66942e7bfa12f3d0757931f8ec5830 | 扩展/其他现用 |
| M0081 | D073:6 S08 | cte.warm.session-edit/query-warm-edit | R39 | "success"; exit=43; probe=1 | 60af9409e305d0adb5202506e965fb2b2aff912862077c6cd4a11c37baa4cec1 | de568c71785d83ff93e1d623d730f1dac279746b30e0417caa242d025c3b9472 | 扩展/其他现用 |
| M0082 | D074:5 S09 | mn.trap.div-overflow/run-trap | R37 | "success"; exit=70 | 010b1f8a3df17e4fec3e42c0c6ba4fb54c10fee82d242c2c2bc3cdba012297cb | 2e29391672395d67d90606a5a2c53fa0b81f8b479a37f9e5fa180cd376735cdf | 扩展/其他现用 |
| M0083 | D075:8 S09 | mn.first.trace/run-first | R37 | "success"; exit=0 | 00e9ff62832b8d5db11d33124a2ef0924b9b1e08004424abee744f999aeca402 | 052754b0f561023a79040c1629b358ed0a53007bdd5b345320f69c6f3f9bd374 | 扩展/其他现用 |
| M0084 | D076:4 S09 | mn.value.seven/run-value | R37 | "success"; exit=7 | 67bccba958c42839d3649d9e083bc87118d5a8857c471a02229702294b65a3d8 | ba659671dcaa88fc24bdfc144a079e06e68fc063cc37ae974efcdc42a197a8ab | 扩展/其他现用 |
| M0085 | D077:5 S10 | ac.core.precedence/run | R04 | "success"; exit=14 | e9f06c6db735783c309eb67d6bc2d801d5f8d613d0b50d39670a1d5dc66f1188 | 369ef0e434a928da2190278de5e55768cc1c0799208d371a987af395533adbbb | 扩展/其他现用 |
| M0086 | D077:8 S10 | ac.core.loop-retest/run | R04 | "success"; exit=3 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | 78f79a7c4ba22807a79de4eb2edf60c868cb5c3219a2009549c837af216239b6 | 扩展/其他现用 |
| M0087 | D077:11 S10 | ac.core.while-zero/run | R04 | "success"; exit=1 | 4f558c850d1bef74ff12adb01884b2ec8570eb3ed5edf344065fd61a3cf0fa19 | 594e0aced1226a9bcac71eb639e9d3f64018cf36ffa420db4efc41cd8ff3b605 | 扩展/其他现用 |
| M0088 | D077:14 S10 | ac.core.unexecuted-trap/run | R04 | "success"; exit=0 | 1e9a66abf0d874fb7e4398d5230bfccc856009cc6e4cbaf57fe558f5ea3740fe | 8da65f532fe689d928caa057f0d53ef1cd03cd62dfe3f2912a00c60b260e41b6 | 扩展/其他现用 |
| M0089 | D077:17 S10 | ac.core.add-overflow/run | R04 | "success"; exit=70 | d3a0e872e6b62cf8a119d2df92d98758a4cef1e0965e16b17e6611746fca41af | ef44329c274bef5b6b5d6dc8ed5bbd08ac8e6e40dabca178079ac5693d63454b | 扩展/其他现用 |
| M0090 | D077:20 S10 | ac.core.mul-overflow/run | R04 | "success"; exit=70 | 6db3d4b0cad6c96ee853679c84619186aefd02a1b0af2da1d98642961eb877ab | ef44329c274bef5b6b5d6dc8ed5bbd08ac8e6e40dabca178079ac5693d63454b | 扩展/其他现用 |
| M0091 | D077:23 S10 | ac.core.bool-branch/run | R04 | "success"; exit=4 | 5fb17b26f7fc43eff8ad77c89e1d06874d4df5adecf1efb9e478297599c1f3fb | 3743d1f9781097617794ac80478f59239c60d6c7b561855e8944ba4967e72477 | 扩展/其他现用 |
| M0092 | D077:27 S10 | ac.core.named-arg-binding/run | R04 | "success"; exit=73 | 2ca5739b1d6890e9585cb8ec933a0798cec4020cd754db0b7e735e84554b0def | 7e2840d3b8d01ee4b7f34e97bf1cc05032444656eb5e00b13b81ced70016b7fe | 扩展/其他现用 |
| M0093 | D077:34 S10 | ac.core.named-eval-order-marks/run | R04 | "success"; exit=12; stderr#d96dff4f88b8 | 1beac57b2e5fdeff6049addf2221ec3c65213f6e62132094b5c494140b054b3b | 9b0f266597a9fd3407cb1ea6de81b62eb125197d96de2512f1a20c40720b88d1 | 扩展/其他现用 |
| M0094 | D077:37 S10 | ac.core.observable-before-trap/run | R04 | "success"; exit=70; stderr#6193c97585a0 | 36be0b6b962af80b2d2f4229973fbf5cbc8b2b8ba0d08e8b0978df3d21c6583f | ddea88bc48c16af73d44e756323c832733f3296ae10529a84e2b4c64bf01753e | 扩展/其他现用 |
| M0095 | D077:40 S10 | ac.core.trace-without-capability/run | R04 | {"stage":"resolve","code":"source.undefined"} | 36be0b6b962af80b2d2f4229973fbf5cbc8b2b8ba0d08e8b0978df3d21c6583f | cf4cee27ff523c6e610cf1ea5bdc37ed7c683bc3e1c4cc9b0558206d91fdbd86 | 扩展/其他现用 |
| M0096 | D078:3 S10 | ac.hir.ref-dangling-expr/chk | R03 | {"stage":"bind","code":"bind.dangling-ref"} | 07df73501a68687118dfef78b53567aa92da3099c67af203240eafd8099dc349 | da58a423bb8f5ed0e64914b947e4a4b85c1dde93d0112083f77df1e941dbb2a8 | 扩展/其他现用 |
| M0097 | D078:61 S10 | ac.hir.ref-dangling-type/chk | R03 | {"stage":"bind","code":"bind.dangling-type-ref"} | 8cbb5f8d89f22163fec8117a772688f33c5b91d576defc296ce2abb20a8c8b1d | 5e09e66d2e1f1d450829f8b4556f7a25cb195c4c8a9a5fe34f212953a8a65f1f | 扩展/其他现用 |
| M0098 | D078:119 S10 | ac.hir.struct-dup-arg/chk | R03 | {"stage":"verify","code":"verify.binding-invalid"} | 2aa23499da6f9b45be076a2d0dc9888a6fa6d38aee9ee9524944179e14bf2de0 | 15087ca45c7520985683ef89d1f2c85cbb3b3d2d6cb2070506d8f57d0f1c639a | 扩展/其他现用 |
| M0099 | D078:185 S10 | ac.hir.struct-missing-field/chk | R03 | {"stage":"bind","code":"bind.missing-field"} | 0525a83f6cbc21faa8efb9ebc8e88228fd6367efa1a4410d1cd24858780b6c06 | 3a761b9029b7926b895f27b2a31c93f66e45e7468a9763a5bfc0ac2b8e57ad89 | 扩展/其他现用 |
| M0100 | D078:242 S10 | ac.hir.type-swap-mismatch/chk | R03 | {"stage":"verify","code":"verify.type-mismatch"} | 6517aa4cbe6114c61df06be63d5c9882b08b5f6a77312d489dff4055b6d528d8 | 73817ffa8e6cb43e7d93629e697c86fb97d5970a9312df64d3da9f9e991384fd | 扩展/其他现用 |
| M0101 | D078:312 S10 | ac.hir.graph-self-cycle/chk | R03 | {"stage":"verify","code":"verify.block-structure"} | 75330452d9b3d0bdb49355ef702c581bbf66e1d96adbc64f11695dbfc40d4115 | 320db104674927d8c72a9b130acf86ca8ad1a121b9a2a727f3c83d2b29fd1fe0 | 扩展/其他现用 |
| M0102 | D078:316 S10 | ac.hir.type-param-mismatch/chk | R03 | {"stage":"verify","code":"verify.type-mismatch"} | 67325188b356c7b9c50e2fb777952642bcfe06e66c415f65c85f8a25d04152b6 | 73817ffa8e6cb43e7d93629e697c86fb97d5970a9312df64d3da9f9e991384fd | 扩展/其他现用 |
| M0103 | D079:3 S10 | ac.hir.wrapper-run/chk | R03 | "success" | ab58ef25c7657c2af200b6a88f9b799f9e02239c86e22d5416b2b335efa7aa54 | f7a86d949881cce5aa766ba05417c46bc2cb68a832bdaf280503df80b5fb8cd1 | 扩展/其他现用 |
| M0104 | D079:3 S10 | ac.hir.wrapper-run/run | R04 | "success"; exit=5 | ab58ef25c7657c2af200b6a88f9b799f9e02239c86e22d5416b2b335efa7aa54 | ca4738ff7e2656db178660e099ac7f25c446c115b76a82f1b747742c033c18e0 | 扩展/其他现用 |
| M0105 | D079:91 S10 | ac.hir.wrapper-build/bld | R02 | "success" | ab58ef25c7657c2af200b6a88f9b799f9e02239c86e22d5416b2b335efa7aa54 | d93219db78d7ae2888619dfa9d772187a3b893e0f51144a670245a91c4ff8e1a | 扩展/其他现用 |
| M0106 | D080:3 S10 | ac.red.build-golden-fail-product-still-runs/bld-bad-golden | R02 | "success"; artifacts=1 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | 7721353e76c7e98132864f7bcdfdd52194bc7e802642c226cd1fa5389c60cdbc | 扩展/其他现用 |
| M0107 | D080:3 S10 | ac.red.build-golden-fail-product-still-runs/run-consumer | R04 | "success"; exit=3 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | b19be231698622d405f704eeb8d8589f2a441ea804c811a16dc5dcd540744460 | 扩展/其他现用 |
| M0108 | D080:6 S10 | ac.red.wrong-exit-fails/run-wrong-exit | R04 | "success"; exit=4 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | a611cd7f8f160df0ed75d69c02dffcbd04c46cce3f56c65796ffcbf0226be4de | 扩展/其他现用 |
| M0109 | D081:3 S10 | ac.scalar.short-circuit-trap/run | R04 | "success"; exit=70 | d727ebe7cb23602d20392db4ca93886dc10ecd9d5bdb081007253b487dd40cb8 | 6b9188fd6a0b251c784f28f4be3a800d69b724d80138e0d463ebcc982f9ccaf4 | 扩展/其他现用 |
| M0110 | D081:14 S10 | ac.scalar.div-zero/run | R04 | "success"; exit=70 | 16b82e5cc1c1167f15f2954fcfd2e79a10a8e816edd1983b26cf50dd21a78644 | 6b9188fd6a0b251c784f28f4be3a800d69b724d80138e0d463ebcc982f9ccaf4 | 扩展/其他现用 |
| M0111 | D081:17 S10 | ac.scalar.cond-for/run | R04 | "success"; exit=0 | 73531c1db5fb130bc2e40bb7d84d25abe4845e43d6a01c1e121bb37814f09dfe | e223d2071947057e69ea7f55ab774482105e97702345166680bb823cfea5ea4d | 扩展/其他现用 |
| M0112 | D082:3 S10 | ac.src.unclosed-brace/chk | R03 | {"stage":"parse","code":"source.syntax"} | bd0a329b1b55fcddc7a7f783f0c065fdbb96d00e72d83e992b81f7237abe54f8 | 17a75a2dc9fcde9310143bf59bd3a6b4f2c29a201ce9f995d6ced25554b62834 | 扩展/其他现用 |
| M0113 | D082:6 S10 | ac.src.unclosed-scalar-for/chk | R03 | {"stage":"parse","code":"source.syntax"} | c7e16f19b38df73dcf25d83363041e9e31e0a078202b092f56cc66106f394654 | 3a67965e85e42e07625bd4d5e232ab42c8d494742c7486e5695d450e129be4a2 | 扩展/其他现用 |
| M0114 | D082:13 S10 | ac.src.undefined-local/chk | R03 | {"stage":"resolve","code":"source.undefined"} | 9cc4bb5c39b5c26fcd1fcfa050d6a7f32d5c88d3d2e6c97ba78d9985781b27c1 | 622095db720524277253738500e99e5882c42bc2d6950149ad0ea8d3a20836f9 | 扩展/其他现用 |
| M0115 | D082:16 S10 | ac.src.undefined-fn/chk | R03 | {"stage":"resolve","code":"source.undefined"} | 81a560cbf2bfc85cd6b8b093a461d0752d17ad3762510fec0fb119970ae60fcd | 622095db720524277253738500e99e5882c42bc2d6950149ad0ea8d3a20836f9 | 扩展/其他现用 |
| M0116 | D082:19 S10 | ac.src.type-not-on-int/chk | R03 | {"stage":"typecheck","code":"source.type-mismatch"} | f8fc48643a4f74cbcd04d1251007a560b2e9866f864bcb2885d5a95f87662817 | 7a73987cc623f1582d561fb561c3af791087764931ad1f793e70b76d11350ab3 | 扩展/其他现用 |
| M0117 | D082:26 S10 | ac.src.type-neg-on-bool/chk | R03 | {"stage":"typecheck","code":"source.type-mismatch"} | e5bf923be929232ecb03e1b0e6d895debe877403cf6fdada5a8a979d4378097a | 7a73987cc623f1582d561fb561c3af791087764931ad1f793e70b76d11350ab3 | 扩展/其他现用 |
| M0118 | D082:33 S10 | ac.src.newline-rhs/chk | R03 | {"stage":"parse","code":"source.syntax"} | 8d174a38fdd99564bec43b436320a8f13173cce120a65ee8223127e08defa34c | 17a75a2dc9fcde9310143bf59bd3a6b4f2c29a201ce9f995d6ced25554b62834 | 扩展/其他现用 |
| M0119 | D082:40 S10 | ac.src.newline-call/chk | R03 | {"stage":"parse","code":"source.syntax"} | 310fefcf721297dc8f8c8eede66aa9630a3ad034c666ab1426f0e695bfa616ac | 17a75a2dc9fcde9310143bf59bd3a6b4f2c29a201ce9f995d6ced25554b62834 | 扩展/其他现用 |
| M0120 | D082:48 S10 | ac.src.undefined-build/bld | R02 | {"stage":"resolve","code":"source.undefined"} | 9cc4bb5c39b5c26fcd1fcfa050d6a7f32d5c88d3d2e6c97ba78d9985781b27c1 | 83aaa6330de56f88abd46e4d0a11534e4b82aa1a368264f042c19f888c8080c1 | 扩展/其他现用 |
| M0121 | D082:51 S10 | ac.src.undefined-run/run | R04 | {"stage":"resolve","code":"source.undefined"} | 9cc4bb5c39b5c26fcd1fcfa050d6a7f32d5c88d3d2e6c97ba78d9985781b27c1 | cf4cee27ff523c6e610cf1ea5bdc37ed7c683bc3e1c4cc9b0558206d91fdbd86 | 扩展/其他现用 |
| M0122 | D082:54 S10 | ac.src.wrong-stage-build-red/bld-red | R02 | {"stage":"parse","code":"source.syntax"} | 9cc4bb5c39b5c26fcd1fcfa050d6a7f32d5c88d3d2e6c97ba78d9985781b27c1 | f07b24aa2b1f4f1e93f20104133b094e4d843b149581ff4122a764defcdc133d | 扩展/其他现用 |
| M0123 | D082:57 S10 | ac.src.entry-missing-backend/run | R04 | {"stage":"backend","code":"entry.not-found"} | 17f23ec5eea689cd4c911ddecd7bdd08212a81e0134d7d7da5050a4c9cbe073b | 36031ae33a9e70a3308f517a4b42075f6e46c7a1998317eeeb213ccfb8660c21 | 扩展/其他现用 |
| M0124 | D082:62 S10 | ac.syntax.core-invalid/parse | R42 | {"stage":"parse","code":"syntax.parse"} | 0bd59956c32bdf9f6dcde51e2b93e12b4cc9a374aa2680875a0ebf8d275c55e6 | d74521c08c8742a94686be4e5d815df36d01a596b4fd072f6c7243e8b73b43c6 | 扩展/其他现用 |
| M0125 | D082:65 S10 | ac.syntax.mini-invalid/parse | R42 | {"stage":"parse","code":"syntax.parse"} | f48428905709096065533bb5a72304185be47e26d8f84f795f47eb587ae994e5 | 319379ddde774bf96ce0d11eec809c61c733a99d3db8d01ef6ea362563c1ad5e | 扩展/其他现用 |
| M0126 | D083:3 S10 | ac.syntax.core-lossless/parse | R42 | "success"; artifacts=1 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | bb141e9e32e071f5f8f629791761c295178c699c8dac27089cfa365e60101b9e | 扩展/其他现用 |
| M0127 | D083:6 S10 | ac.syntax.mini-valid/parse | R42 | "success" | 709793a320b0ef029b8c0096d12ad80fab9128836c1bfdbc418a40229d3b0efd | 807739ae53be746c1f6257100b713e08ea76ba5895cec04639248e0d470e9213 | 扩展/其他现用 |
| M0128 | D083:9 S10 | ac.syntax.edit-equivalence/edit | R41 | "success"; artifacts=1 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | a3fc9110b880d1253ce3919b8aac6bb853e311364d6d303ed4503a56c37ab3f5 | 扩展/其他现用 |
| M0129 | D084:9 S11 | ac.sema.cache-roundtrip/roundtrip | R40 | "success"; probe=3 | 47bc51e3b584f9985fe58a1b023bb749bdf01b70f56c075e4134b55e5b9704c6 | 4dd6eb58577a53abeb918ed8e733a9ab71d77df10475305c52bb4f6134d18689 | 扩展/其他现用 |
| M0130 | D085:9 S11 | ac.sema.callee-sig-change/callee-sig | R40 | "success"; probe=2 | 867cabf81c9cd0f7a5a2309d0c6aec32ee0432a7ea78a87ec212e107164eafc1 | fb45ea87dbf7691c648a7501d5cf14ec63ffde5f6b701cc481819c03e22cf715 | 扩展/其他现用 |
| M0131 | D086:11 S11 | ac.sema.infer-bindings/bindings | R40 | "success"; probe=1 | f89b0666a0fcf4ca1c7f903bf5e23113c56efea5643cd2ecc4ae69ae2dca798c | e7514e4f2dbfd4443ef158617c35ae0427b81c30b8181a7ab335af8a6ad0db63 | 扩展/其他现用 |
| M0132 | D087:8 S11 | ac.sema.parse-error/parse | R40 | {"stage":"parse","code":"source.syntax"}; probe=1 | be365c6e02805206901e252c6cbca146ac74e528fe2c994c920974128e83a635 | 913824ae2a4161b9799b627318f281a05a0f03a6823d3cecf3bcb154ec83c232 | 扩展/其他现用 |
| M0133 | D088:8 S11 | ac.sema.turnaround-stages/stages | R40 | "success"; probe=1 | 40130465dc85fcbaecc4a1b507bea7febb59d12c9416a3b9c37fe3265bee4e85 | e832c7b9d4d041110634d823227c9bcb38d5a664bc29ad349867c95e9a928725 | 扩展/其他现用 |
| M0134 | D089:12 S12 | ace-ca-d3.ce-text-view/certify | R06 | "success"; exit=0; probe=2 | 81581cfb49d5bb52a7df4a2b446e4183523aa2d065eb2d47cf7e8bf5177ed1be | bb2d64c9e92b6f15bb81c358355fc78c6ac7af30bd663c28c73116502a38b35a | 扩展/其他现用 |
| M0135 | D089:37 S12 | ace-ca-d3.ce-record/certify | R06 | "success"; exit=0; probe=2 | d2e4017e7c65b58d9d3a438596a76202d6f9b211e33adf745a1428674b931842 | bb2d64c9e92b6f15bb81c358355fc78c6ac7af30bd663c28c73116502a38b35a | 扩展/其他现用 |
| M0136 | D090:4 S12 | ace-ca-d3.utf8-bytes-native/certify | R08 | "success"; exit=8; probe=1 | d74db265e0b3d9761919cbbd16f000bb7c844ba5eab1cb5b3ce462b2a2fceb29 | 27904821621ffbb23d1fcddfc3d86cb4c6a7ecf2971c9fa99e3f208aa0ce5d4b | 扩展/其他现用 |
| M0137 | D090:11 S12 | ace-ca-d3.utf8-bytes-c/certify | R08 | "success"; exit=8; probe=1 | d74db265e0b3d9761919cbbd16f000bb7c844ba5eab1cb5b3ce462b2a2fceb29 | 9ef6b4ef51c4db0a1ddcaf8daaf4b0767f3278791842ee98f15d037cb8201048 | 扩展/其他现用 |
| M0138 | D090:18 S12 | ace-ca-d3.utf8-scalars-native/certify | R08 | "success"; exit=3; probe=1 | ff3d7a9cc9fbb4e32157d762958a75680b0206bb0d56b6b2e29a5953caf84922 | 64bf2991839192c0135dbec87faf3618995fcf0524ad6177d32618413e098323 | 扩展/其他现用 |
| M0139 | D090:25 S12 | ace-ca-d3.utf8-scalars-c/certify | R08 | "success"; exit=3; probe=1 | ff3d7a9cc9fbb4e32157d762958a75680b0206bb0d56b6b2e29a5953caf84922 | 94b6dbbbdc2cc290ee5be182d35997f0f14a809aaa497de6ea62609a631dc36d | 扩展/其他现用 |
| M0140 | D090:32 S12 | ace-ca-d3.record-mut-native/certify | R08 | "success"; exit=42; probe=1 | a5329c00230a727490ee9021fa7af4bc6c68d878a3c7b1df6e4159217dd04561 | 74a90075199c53f9e750e6686f644f888742d605640d99238189f4efd406486a | 扩展/其他现用 |
| M0141 | D090:39 S12 | ace-ca-d3.record-mut-c/certify | R08 | "success"; exit=42; probe=1 | a5329c00230a727490ee9021fa7af4bc6c68d878a3c7b1df6e4159217dd04561 | f335d1f511e14e45a7ffcb8285508553e1b6b7851139e0d9cc659b330ddfbf70 | 扩展/其他现用 |
| M0142 | D090:46 S12 | ace-ca-d3.record-mut-fail-native/certify | R08 | "success"; exit=70; probe=1 | a85ac86bf2bce8b12d8025fcdb85baebb3e154f04119f8ac4789b524b8e88daa | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0143 | D090:53 S12 | ace-ca-d3.record-mut-fail-c/certify | R08 | "success"; exit=70; probe=1 | a85ac86bf2bce8b12d8025fcdb85baebb3e154f04119f8ac4789b524b8e88daa | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0144 | D090:60 S12 | ace-ca-d3.handle-lifecycle-native/certify | R08 | "success"; exit=5; probe=1 | 7847fa1e0b25bfacfb4c683cbb2ff1d540b7a21b9b49dc2cf3497019f642842d | 056050948622222c3ae5ab113255266bc5468612b4e7a4e4cc07fe5e4871a316 | 扩展/其他现用 |
| M0145 | D090:67 S12 | ace-ca-d3.handle-lifecycle-c/certify | R08 | "success"; exit=5; probe=1 | 7847fa1e0b25bfacfb4c683cbb2ff1d540b7a21b9b49dc2cf3497019f642842d | bd24779adaa930ee42256f66a561ec8dd33f1b088ca4c37f2a46626e200e0ee1 | 扩展/其他现用 |
| M0146 | D090:74 S12 | ace-ca-d3.handle-stale-native/certify | R08 | "success"; exit=70; probe=1 | 2d3165b13cde20346495cfe8aa28f65f6835544d7efef9792c396c4c78d51c5d | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0147 | D090:81 S12 | ace-ca-d3.handle-stale-c/certify | R08 | "success"; exit=70; probe=1 | 2d3165b13cde20346495cfe8aa28f65f6835544d7efef9792c396c4c78d51c5d | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0148 | D090:88 S12 | ace-ca-d3.txn-commit-native/certify | R08 | "success"; exit=21; probe=1 | 4518ee42be97df7c4fd9fda4d0a0a27814ea325f10c9c84eab4afbc27b614157 | 7e26fb5335743a261091b7c285e5c8d83ab7ba924df89f2ab717a890a6e9f59e | 扩展/其他现用 |
| M0149 | D090:95 S12 | ace-ca-d3.txn-commit-c/certify | R08 | "success"; exit=21; probe=1 | 4518ee42be97df7c4fd9fda4d0a0a27814ea325f10c9c84eab4afbc27b614157 | f068338168e58fdeabd14921f86b2b41bc531c053243360ce16b605614ab649a | 扩展/其他现用 |
| M0150 | D090:102 S12 | ace-ca-d3.txn-seed-fail-native/certify | R08 | "success"; exit=70; probe=1 | f2fcf55d24a18b15bffe0a07f3138c925e1cfbe0fd195d75eb8a22254f0e42b5 | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0151 | D090:109 S12 | ace-ca-d3.txn-seed-fail-c/certify | R08 | "success"; exit=70; probe=1 | f2fcf55d24a18b15bffe0a07f3138c925e1cfbe0fd195d75eb8a22254f0e42b5 | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0152 | D090:116 S12 | ace-ca-d3.txn-laststep-fail-native/certify | R08 | "success"; exit=70; probe=1 | 161de218e508ca0a00dba73a33a13ccc45461530a155f425a898abf741782ac5 | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0153 | D090:123 S12 | ace-ca-d3.txn-laststep-fail-c/certify | R08 | "success"; exit=70; probe=1 | 161de218e508ca0a00dba73a33a13ccc45461530a155f425a898abf741782ac5 | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0154 | D090:137 S12 | ace-ca-d3.utf8-owned-native/certify | R08 | "success"; exit=7; probe=1 | 5898ced156d999fdbd3d94ebdf60f1e52370375702e03180b268d6412d0bc5a3 | 5984c49ad5a53e81e3b036b6018c74f7d5f3e4de17146280de00f0418bf51ae6 | 扩展/其他现用 |
| M0155 | D090:144 S12 | ace-ca-d3.utf8-owned-c/certify | R08 | "success"; exit=7; probe=1 | 5898ced156d999fdbd3d94ebdf60f1e52370375702e03180b268d6412d0bc5a3 | d5ba8c36b1c4fc3686ca6abf1a8426ea187f5b4ab5052d34ce5c6a73f6025173 | 扩展/其他现用 |
| M0156 | D090:154 S12 | ace-ca-d3.pairv-temp-native/certify | R08 | "success"; exit=47; probe=1 | b878ddcbdd9c6e6edda9e570851723b136e7bff03d4f6f40f6268c890347f57d | 6186702bf0029e5f9875899ee5171c1dcf866972e7deb3e189ec3b613a31cbde | 扩展/其他现用 |
| M0157 | D090:161 S12 | ace-ca-d3.pairv-temp-c/certify | R08 | "success"; exit=47; probe=1 | b878ddcbdd9c6e6edda9e570851723b136e7bff03d4f6f40f6268c890347f57d | 40fbecfde1db26ced478d9beae8167173ddf52f2d7b6eb00ed93497557f792a6 | 扩展/其他现用 |
| M0158 | D090:175 S12 | ace-ca-d3.increment-body-edit-native/certify | R08 | "success"; exit=21; probe=4 | ee30635b55962d97f39c2f867bac763c506e286b5abbb35e93eb7e591f061524 | df06351a1b27562af01c878e6cdff1b28f769be6675d9090df46011a2454c249 | 扩展/其他现用 |
| M0159 | D090:182 S12 | ace-ca-d3.increment-body-edit-c/certify | R08 | "success"; exit=21; probe=4 | ee30635b55962d97f39c2f867bac763c506e286b5abbb35e93eb7e591f061524 | cf2fe3fd47c7c55a185027055d9bebbe67392d9a4b6b8ad820678bdf3fddd3d6 | 扩展/其他现用 |
| M0160 | D090:198 S12 | ace-ca-d3.recover-first-error-native/certify | R08 | "success"; exit=70; probe=3 | a85ac86bf2bce8b12d8025fcdb85baebb3e154f04119f8ac4789b524b8e88daa | 0e48e0545a9846d003ead7e1bbc9a685e5848cf51534781cbd620df6f5cf6ebd | 扩展/其他现用 |
| M0161 | D090:205 S12 | ace-ca-d3.recover-first-error-c/certify | R08 | "success"; exit=70; probe=3 | a85ac86bf2bce8b12d8025fcdb85baebb3e154f04119f8ac4789b524b8e88daa | 3e2e0dc5925105ba42041dac4e8d09b6fa1e7d1ab614f66bc96fa4bb059c0d1e | 扩展/其他现用 |
| M0162 | D090:215 S12 | ace-ca-d3.callback-reentrant-native/certify | R08 | "success"; exit=21; probe=1 | ee30635b55962d97f39c2f867bac763c506e286b5abbb35e93eb7e591f061524 | 7e26fb5335743a261091b7c285e5c8d83ab7ba924df89f2ab717a890a6e9f59e | 扩展/其他现用 |
| M0163 | D090:222 S12 | ace-ca-d3.callback-reentrant-c/certify | R08 | "success"; exit=21; probe=1 | ee30635b55962d97f39c2f867bac763c506e286b5abbb35e93eb7e591f061524 | f068338168e58fdeabd14921f86b2b41bc531c053243360ce16b605614ab649a | 扩展/其他现用 |
| M0164 | D090:237 S12 | ace-ca-d3.provider-transition-native/certify | R08 | "success"; exit=3504; probe=2 | bdf4e4d14d0dd11a81abf4b5761a0688d9fa94abe2350e85d87caaeb967dd83b | a912435fea7edabf9b9c02b963812a5796ac14c9f04ad964ffdf3262904cdd55 | 扩展/其他现用 |
| M0165 | D090:244 S12 | ace-ca-d3.provider-transition-c/certify | R08 | "success"; exit=3504; probe=2 | bdf4e4d14d0dd11a81abf4b5761a0688d9fa94abe2350e85d87caaeb967dd83b | 84f05f06bdd45a2bc600614c0ec3b7059e5a1428a3f2dbc1686aa5bc678961a7 | 扩展/其他现用 |
| M0166 | D090:255 S12 | ace-ca-d3.txn3-commit-native/certify | R08 | "success"; exit=3504; probe=1 | bdf4e4d14d0dd11a81abf4b5761a0688d9fa94abe2350e85d87caaeb967dd83b | d7df9ad67fe115560594e2ab7612ad980a3ee3df23ae23137546147d2cd8a422 | 扩展/其他现用 |
| M0167 | D090:262 S12 | ace-ca-d3.txn3-commit-c/certify | R08 | "success"; exit=3504; probe=1 | bdf4e4d14d0dd11a81abf4b5761a0688d9fa94abe2350e85d87caaeb967dd83b | b74627c116ac1efe92c7dc14fcea65588a9ed117857ae1ff6a1c4b78575472db | 扩展/其他现用 |
| M0168 | D090:276 S12 | ace-ca-d3.txn3-lastfail-native/certify | R08 | "success"; exit=70; probe=1 | e60fd979ad066ec598aaeef2a9bcdc30b3c51a92113808730b3b7ca9f1b3dda8 | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0169 | D090:283 S12 | ace-ca-d3.txn3-lastfail-c/certify | R08 | "success"; exit=70; probe=1 | e60fd979ad066ec598aaeef2a9bcdc30b3c51a92113808730b3b7ca9f1b3dda8 | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0170 | D090:293 S12 | ace-ca-d3.clone-owners-native/certify | R08 | "success"; exit=10; probe=1 | a2b81071c223ce69224d45e2db4c856ccf538421b18dd82620e85fbc1ed657f5 | f43be35e08ab011b613c53053d2b2cda2a9b40f4d7a4d7b8dd8f8dd42b4ea69b | 扩展/其他现用 |
| M0171 | D090:300 S12 | ace-ca-d3.clone-owners-c/certify | R08 | "success"; exit=10; probe=1 | a2b81071c223ce69224d45e2db4c856ccf538421b18dd82620e85fbc1ed657f5 | 2447411e484abfe227762aa367d78531e32639553fbd34cc547035120a626c87 | 扩展/其他现用 |
| M0172 | D090:310 S12 | ace-ca-d3.forged-handle-native/certify | R08 | "success"; exit=70; probe=1 | b663571e1b39c674d38ae1cfdea1a28d3dfcef408961ca1e69b8e12983e1fe9c | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0173 | D090:317 S12 | ace-ca-d3.forged-handle-c/certify | R08 | "success"; exit=70; probe=1 | b663571e1b39c674d38ae1cfdea1a28d3dfcef408961ca1e69b8e12983e1fe9c | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0174 | D090:326 S12 | ace-ca-d3.stored-cb-native/certify | R08 | "success"; exit=180; probe=1 | a42e8e302e1832c75d9c62cb45220f67aa2821504a46d2222623edeaaf5b5d41 | 356d93946730ee80481b351c39eee1c82929d4d350fe30901fcd4c5b79dbe8d5 | 扩展/其他现用 |
| M0175 | D090:333 S12 | ace-ca-d3.stored-cb-c/certify | R08 | "success"; exit=180; probe=1 | a42e8e302e1832c75d9c62cb45220f67aa2821504a46d2222623edeaaf5b5d41 | da645b80d6f0fdde5b446d385e60e54aa4802459558cd1ff91e626489df7fb62 | 扩展/其他现用 |
| M0176 | D090:344 S12 | ace-ca-d3.utf8-restart-edit-native/certify | R08 | "success"; exit=8; probe=2 | d74db265e0b3d9761919cbbd16f000bb7c844ba5eab1cb5b3ce462b2a2fceb29 | 66c01dc8cfa25a7ac397137bfcec29d882ca9646b97070751d5374150e99208a | 扩展/其他现用 |
| M0177 | D090:351 S12 | ace-ca-d3.utf8-restart-edit-c/certify | R08 | "success"; exit=8; probe=2 | d74db265e0b3d9761919cbbd16f000bb7c844ba5eab1cb5b3ce462b2a2fceb29 | 707e17868d224a510d4b72a50c417a4c263cef5f077c6e298e098acb6f32b27e | 扩展/其他现用 |
| M0178 | D090:362 S12 | ace-ca-d3.callback-native/certify | R08 | "success"; exit=18; probe=1 | dd81bae8177c3f484bd167166474f372015161f11da61b003c523bf8373b161b | 40928587fc063b17ad47b7cc15891f5dfc28268b3b65b8a49057e2b4008085ff | 扩展/其他现用 |
| M0179 | D090:369 S12 | ace-ca-d3.callback-c/certify | R08 | "success"; exit=18; probe=1 | dd81bae8177c3f484bd167166474f372015161f11da61b003c523bf8373b161b | dd6682ce84063bab72339bf7675068b81ac515305fa1d4a31642500c76f3848b | 扩展/其他现用 |
| M0180 | D090:376 S12 | ace-ca-d3.callback-fail-native/certify | R08 | "success"; exit=70; probe=1 | 0616a067c57d0387595e399b7cefc843c2a0f1a2cdc2b0c781875f86a54d8e9e | d2aa1d7b2b504490c03942924edf9a5f24e74ed2a92c103da4116407efb194c7 | 扩展/其他现用 |
| M0181 | D090:383 S12 | ace-ca-d3.callback-fail-c/certify | R08 | "success"; exit=70; probe=1 | 0616a067c57d0387595e399b7cefc843c2a0f1a2cdc2b0c781875f86a54d8e9e | 0fbaf071f3598f1d7d4d0df21655a6ff708eac05b26c19298f319735e082c1fd | 扩展/其他现用 |
| M0182 | D090:394 S12 | ace-ca-d3.alias-pair-ok-native/certify | R08 | "success"; exit=53; probe=1 | 61a5a916e6b81919afad52af604476734492e3dd33d1fef160cb7982a1699330 | 290752402d4b1e2dea11e23af4478704a25893e9d74da0fe3f33a10df8f71745 | 扩展/其他现用 |
| M0183 | D090:401 S12 | ace-ca-d3.alias-pair-ok-c/certify | R08 | "success"; exit=53; probe=1 | 61a5a916e6b81919afad52af604476734492e3dd33d1fef160cb7982a1699330 | f1381a26b488ac2e30ddfa58edbb45ca9d5460860860d5dbf41c159211e33012 | 扩展/其他现用 |
| M0184 | D090:408 S12 | ace-ca-d3.alias-pairv-ok-native/certify | R08 | "success"; exit=16; probe=1 | 2d848dab6e9e23c15b7b87790a236809102a8c686ebe858f8da6b6943450617a | ccea8e2c10c579e7b05b23d169f9948cd0447de788975e3296fd6ca0877a4303 | 扩展/其他现用 |
| M0185 | D090:415 S12 | ace-ca-d3.alias-pairv-ok-c/certify | R08 | "success"; exit=16; probe=1 | 2d848dab6e9e23c15b7b87790a236809102a8c686ebe858f8da6b6943450617a | 346a91bc2ce72da5a4ea3ddd650d88765716be8ae352c14a147b557f9763a90c | 扩展/其他现用 |
| M0186 | D090:422 S12 | ace-ca-d3.alias-pair-reject-native/certify | R08 | {"stage":"verify","code":"verify.mut-conflict"}; probe=1 | 82f22f8274bf9d50c37b59c6f8f2393e9a4ff0d0d467db35c34e9aafcc044ebe | 8214bd96f6dc5cc225314259bae5634f9fe9960e05524e2779e22d0e7bc47fb0 | 扩展/其他现用 |
| M0187 | D090:429 S12 | ace-ca-d3.alias-pair-reject-c/certify | R08 | {"stage":"verify","code":"verify.mut-conflict"}; probe=1 | 82f22f8274bf9d50c37b59c6f8f2393e9a4ff0d0d467db35c34e9aafcc044ebe | b9c3caea1ac908f28d53b2f7be9e5968a5816f8138e987c8c05665cc8a1c16f6 | 扩展/其他现用 |
| M0188 | D090:436 S12 | ace-ca-d3.alias-pairv-reject-native/certify | R08 | {"stage":"verify","code":"verify.mut-conflict"}; probe=1 | f4e7e86a7044c670f98b6fb99c380246852245617ccad572007fe73883cef64c | 8214bd96f6dc5cc225314259bae5634f9fe9960e05524e2779e22d0e7bc47fb0 | 扩展/其他现用 |
| M0189 | D090:443 S12 | ace-ca-d3.alias-pairv-reject-c/certify | R08 | {"stage":"verify","code":"verify.mut-conflict"}; probe=1 | f4e7e86a7044c670f98b6fb99c380246852245617ccad572007fe73883cef64c | b9c3caea1ac908f28d53b2f7be9e5968a5816f8138e987c8c05665cc8a1c16f6 | 扩展/其他现用 |
| M0190 | D090:451 S12 | ace-ca-d3.wrong-oracle/certify | R08 | "success"; exit=9; probe=1 | d74db265e0b3d9761919cbbd16f000bb7c844ba5eab1cb5b3ce462b2a2fceb29 | 026c8fda965f6abf68515d3c6d15669f7587f3c5ede52dea4eca16afc353368c | 扩展/其他现用 |
| M0191 | D090:464 S12 | ace-ca-d3.wide-self/wide-self | R06 | "success"; probe=2 | 5aecc0badab601eeb684c33de99ae44ee9e9543e35c49471624620fa6e65647d | 9394af0be3cf8327a8d072cd975f8a83c94f2d47ef2a8d95fc79223e8cb2e175 | 扩展/其他现用 |
| M0192 | D090:471 S12 | ace-ca-d3.wide-n2c/wide-n2c | R06 | "success"; probe=2 | 1b435e5dd25999b7c01a46aa85596da32d5e33e4faa92de519b23254eca7066c | 1d47dd94e98ad1ff4caa0c38c8479e90a3148360db65cc9c940a35121e851894 | 扩展/其他现用 |
| M0193 | D090:478 S12 | ace-ca-d3.wide-c2n/wide-c2n | R06 | "success"; probe=2 | 8e9332ad3032ef0064b0fb932e49b1d31c8265f9d6561ccb10d92b4a8df5cec9 | 3410e547b0caf98c0c6fa76075584446dd7b32ee2d0b257830ed43f3acc36a90 | 扩展/其他现用 |
| M0194 | D091:48 S12 | ace-ca-d3.recover-record-mut-native/certify | R08 | "success"; exit=42; probe=3 | a5329c00230a727490ee9021fa7af4bc6c68d878a3c7b1df6e4159217dd04561 | 43d418e89a285e1259fd515f7156ae9ca02e9b9d5af8f9513a7d462f9acdde83 | 扩展/其他现用 |
| M0195 | D091:55 S12 | ace-ca-d3.recover-record-mut-c/certify | R08 | "success"; exit=42; probe=3 | a5329c00230a727490ee9021fa7af4bc6c68d878a3c7b1df6e4159217dd04561 | 228c6b3de020e58bd548f21e48a55638fa826479becb21df16d610b171ce5342 | 扩展/其他现用 |
| M0196 | D092:9 S13 | ace-ca-d1.source-export/source-export | R06 | "success"; probe=2 | cb1b755055054531f88bc42e8a3d47ca00e0c860a6b261ce74ecb146a85f22b4 | 555426852279c5fe2b048e1e7bdb2422da9f0f9817921bf44047cd8d82b633c2 | 原511 |
| M0197 | D092:35 S13 | ace-ca-d1.source-body-edit/source-body-edit | R06 | {"stage":"run","code":"pe-exit"}; probe=2 | cb1b755055054531f88bc42e8a3d47ca00e0c860a6b261ce74ecb146a85f22b4 | 9cb87c3351865f95062ad053355887829467e670fea4bc12e34c4c3015aab8b6 | 原511 |
| M0198 | D092:64 S13 | ace-ca-d1.typed-self/typed-self | R06 | "success"; probe=2 | 776ac4a08bbe6b2a4362739ba52e891b7c48359286be9e4f3aefa7eb97c95c5e | 9533a410994350c8818625ff70d3cc465e4c2e2e2988170e1dfb36d564edf221 | 原511 |
| M0199 | D092:86 S13 | ace-ca-d1.typed-n2c/typed-n2c | R06 | "success"; probe=2 | a172d13f1f94f466597cb3dd935d8d9e641282d130bc1ced38afc92fbf40e8f7 | 79d33281c4255bb63c1a77c51bb90a714b640c9974c2b9771b33242e6a48c53f | 原511 |
| M0200 | D092:108 S13 | ace-ca-d1.typed-c2n/typed-c2n | R06 | "success"; probe=2 | 81e7354fdac4524653f6785f00d0798a91dc07903a1a26e431cf94ccb1f93d9d | ff884199edf1b1058e49e0015b0423860ed203d76530502bd839c802f2eeb5fa | 原511 |
| M0201 | D092:132 S13 | ace-ca-d1.bad-leg/bad-leg | R06 | {"stage":"parse","code":"deck"}; probe=1 | cb1b755055054531f88bc42e8a3d47ca00e0c860a6b261ce74ecb146a85f22b4 | 1d1ddefbfd8f7abd280c6ab9b86b235904320217e84836fc859116d58965554e | 原511 |
| M0202 | D093:3 S14 | ace-ca-d2.bool-false-native/certify | R09 | "success"; exit=42; probe=1 | 20f5baae952aaf5337c8ebe20ecc9cedc92acdfc8591f9e8bcda5f1ac0f64a43 | 18f286e79dc829f791ab18990034e4fdc2393661306dce045da4ee6381f6b22e | 扩展/其他现用 |
| M0203 | D093:11 S14 | ace-ca-d2.bool-false-c/certify | R09 | "success"; exit=42; probe=1 | 20f5baae952aaf5337c8ebe20ecc9cedc92acdfc8591f9e8bcda5f1ac0f64a43 | 33bbff34bfac05e0098387e761ac041bcd54c0974de684fe3b66805caeccbb21 | 扩展/其他现用 |
| M0204 | D093:19 S14 | ace-ca-d2.bool-true-native/certify | R09 | "success"; exit=42; probe=1 | 6bdd4a3efb2fea7b30147c98d92051a00d2b905f356bd9774abc5177af83ab80 | 18f286e79dc829f791ab18990034e4fdc2393661306dce045da4ee6381f6b22e | 扩展/其他现用 |
| M0205 | D093:27 S14 | ace-ca-d2.bool-true-c/certify | R09 | "success"; exit=42; probe=1 | 6bdd4a3efb2fea7b30147c98d92051a00d2b905f356bd9774abc5177af83ab80 | 33bbff34bfac05e0098387e761ac041bcd54c0974de684fe3b66805caeccbb21 | 扩展/其他现用 |
| M0206 | D093:35 S14 | ace-ca-d2.fallible-native/certify | R09 | "success"; exit=42; probe=1 | 79202b3211386b667bfd38c0714f6754991716bb8269db8f5113758384f3ce23 | 18f286e79dc829f791ab18990034e4fdc2393661306dce045da4ee6381f6b22e | 扩展/其他现用 |
| M0207 | D093:43 S14 | ace-ca-d2.fallible-c/certify | R09 | "success"; exit=42; probe=1 | 79202b3211386b667bfd38c0714f6754991716bb8269db8f5113758384f3ce23 | 33bbff34bfac05e0098387e761ac041bcd54c0974de684fe3b66805caeccbb21 | 扩展/其他现用 |
| M0208 | D093:51 S14 | ace-ca-d2.zero-native/certify | R09 | "success"; exit=70; probe=1 | 63ad7763837a24d513272897c9125c51d488e9e923f668250e6c7f975877412b | df9ac2e403c86afc99bb678870db7c15b97d9dee5267969a9299e7f0bb7005ea | 扩展/其他现用 |
| M0209 | D093:59 S14 | ace-ca-d2.zero-c/certify | R09 | "success"; exit=70; probe=1 | 63ad7763837a24d513272897c9125c51d488e9e923f668250e6c7f975877412b | c78affffefd5cce6e358634f18bb8670b7238c035f295ec3db7b5d088f303425 | 扩展/其他现用 |
| M0210 | D093:67 S14 | ace-ca-d2.overflow-native/certify | R09 | "success"; exit=70; probe=1 | b59d18287dfe06660a8f11d41c3d4d589ad9602b8c7909875cd4a6a0a0b30b83 | df9ac2e403c86afc99bb678870db7c15b97d9dee5267969a9299e7f0bb7005ea | 扩展/其他现用 |
| M0211 | D093:75 S14 | ace-ca-d2.overflow-c/certify | R09 | "success"; exit=70; probe=1 | b59d18287dfe06660a8f11d41c3d4d589ad9602b8c7909875cd4a6a0a0b30b83 | c78affffefd5cce6e358634f18bb8670b7238c035f295ec3db7b5d088f303425 | 扩展/其他现用 |
| M0212 | D093:83 S14 | ace-ca-d2.order-native/certify | R09 | "success"; exit=5235; probe=1 | 8c3d76126036573490a0b378f7325a9887d1bf341c98c3993d27603c9cd9c9ac | 2466c00f83bc528cc859f17d11fa6420218e236ea5dc36cb5eebe5398256123d | 扩展/其他现用 |
| M0213 | D093:91 S14 | ace-ca-d2.order-c/certify | R09 | "success"; exit=5235; probe=1 | 8c3d76126036573490a0b378f7325a9887d1bf341c98c3993d27603c9cd9c9ac | 80b75eaa2c84cac7b7e7fb816f92a5c7c3f7444cc83720d674c355e442456894 | 扩展/其他现用 |
| M0214 | D093:99 S14 | ace-ca-d2.and-native/certify | R09 | "success"; exit=1; probe=1 | 53f5449e185379c0ffadf26c66abf3a5ed2bfd1aff0b01177a4d89e69cc55c2b | 32e4b1417cbd7a084c069c4fa23c130f37b56647a8c62265ec7357d2cfc0a68b | 扩展/其他现用 |
| M0215 | D093:107 S14 | ace-ca-d2.and-c/certify | R09 | "success"; exit=1; probe=1 | 53f5449e185379c0ffadf26c66abf3a5ed2bfd1aff0b01177a4d89e69cc55c2b | 11e90ab7c0170567559285df65922d589c73eafdb32260343e547bf1e987dab4 | 扩展/其他现用 |
| M0216 | D093:115 S14 | ace-ca-d2.or-native/certify | R09 | "success"; exit=1; probe=1 | 88b92a80674a0fe3289172585371043caef5b9822fa845eca3cd832b3a0fa185 | 32e4b1417cbd7a084c069c4fa23c130f37b56647a8c62265ec7357d2cfc0a68b | 扩展/其他现用 |
| M0217 | D093:123 S14 | ace-ca-d2.or-c/certify | R09 | "success"; exit=1; probe=1 | 88b92a80674a0fe3289172585371043caef5b9822fa845eca3cd832b3a0fa185 | 11e90ab7c0170567559285df65922d589c73eafdb32260343e547bf1e987dab4 | 扩展/其他现用 |
| M0218 | D093:131 S14 | ace-ca-d2.mixed-native/certify | R09 | "success"; exit=30211; probe=1 | ded50a8cf557ea51cf17b24897c2e72c7259257264abbb77251258af5efceefc | 0b1b046caea0eae24afda1d2d95fd42c5f27d17e4d4eacba3b7c5c3fef2ccde0 | 扩展/其他现用 |
| M0219 | D093:139 S14 | ace-ca-d2.mixed-c/certify | R09 | "success"; exit=30211; probe=1 | ded50a8cf557ea51cf17b24897c2e72c7259257264abbb77251258af5efceefc | 45a12dafe6d3982b1a6529014b7ab7fd1af11cfeeaa128b01dfa9b6a0a3ad450 | 扩展/其他现用 |
| M0220 | D093:147 S14 | ace-ca-d2.named-order-native/certify | R09 | "success"; exit=325; probe=1 | 7fb82de7218d10f6c0bc8649bc78f18e692760e63b918ba281b66ed3b29bcf71 | 22b3346bade61a118f7953b70f4f10dce94de1f371d62cd251b2cdf2d9041a6f | 扩展/其他现用 |
| M0221 | D093:155 S14 | ace-ca-d2.named-order-c/certify | R09 | "success"; exit=325; probe=1 | 7fb82de7218d10f6c0bc8649bc78f18e692760e63b918ba281b66ed3b29bcf71 | d25afe970023eb2786bd63c416dcf9e9b861f3c1d9da8f19a146bdc193d9fcf9 | 扩展/其他现用 |
| M0222 | D094:4 S14 | ace-ca-d2.add-native/certify | R09 | "success"; exit=42; probe=2 | dd52c149f82e707c08ad2932462f397aab9217b430f50ccb80a25a45978904f2 | 204a6682b97dc50a4947c8c85b77dd89bc6b25782db738d1fe1fdf903d12660e | 扩展/其他现用 |
| M0223 | D094:12 S14 | ace-ca-d2.provider-body-native/certify | R09 | "success"; exit=43; probe=1 | f04d2b14cae96eff3f4d164f281c23c48f5c1beb632dc9950727efbbfb8e8433 | a5ea75a3c3fc0ae37df5de5053f29b3f561165e02c647b6a254eb718e4335f43 | 扩展/其他现用 |
| M0224 | D094:20 S14 | ace-ca-d2.denied-native/certify | R09 | {"stage":"resolve","code":"foreign.capability"}; probe=1 | 697fbe3ec25f03e78accbd911e24f8ec8e10372b186b2d287fc2a55d2e138c8d | a67f5491cd34a5ba841be82732ee4bc632c65f939ac01d0ac2c0a6abcfe6542c | 扩展/其他现用 |
| M0225 | D094:28 S14 | ace-ca-d2.unreviewed-native/certify | R09 | {"stage":"resolve","code":"foreign.authority"}; probe=1 | f21e0457fd1110fb2a12687c86a3df0595ef76a2cb26cc93586a0d110d8ebbbe | b388488837d0da7eb1d15af84b37c30be3e4f0406d1e25df718481ba9582f0f7 | 扩展/其他现用 |
| M0226 | D094:36 S14 | ace-ca-d2.add-c/certify | R09 | "success"; exit=42; probe=2 | dd52c149f82e707c08ad2932462f397aab9217b430f50ccb80a25a45978904f2 | 9def75942a1fd88c0e692b68c6a59cc881aa43cc9c317c88765c662a1f5cc421 | 扩展/其他现用 |
| M0227 | D094:44 S14 | ace-ca-d2.provider-body-c/certify | R09 | "success"; exit=43; probe=1 | f04d2b14cae96eff3f4d164f281c23c48f5c1beb632dc9950727efbbfb8e8433 | 21ab4a3fc1aea6e0dcc4e51a64f72c0b3d44cc9d95dc5ca7e3091c117d173228 | 扩展/其他现用 |
| M0228 | D094:52 S14 | ace-ca-d2.denied-c/certify | R09 | {"stage":"resolve","code":"foreign.capability"}; probe=1 | 697fbe3ec25f03e78accbd911e24f8ec8e10372b186b2d287fc2a55d2e138c8d | 30312281a9fff236006b3b06097ca7806c86817693dfc8af234700764be90a74 | 扩展/其他现用 |
| M0229 | D094:60 S14 | ace-ca-d2.unreviewed-c/certify | R09 | {"stage":"resolve","code":"foreign.authority"}; probe=1 | f21e0457fd1110fb2a12687c86a3df0595ef76a2cb26cc93586a0d110d8ebbbe | 0d87a4cbd9d5185f3926f3c567f4c4adc5e98d6f02cb96850770e18be0cc8046 | 扩展/其他现用 |
| M0230 | D094:68 S14 | ace-ca-d2.wrong-type-native/certify | R09 | {"stage":"resolve","code":"source.foreign.type-mismatch"}; probe=1 | 5e0a74cbeb9f0c96459daee84d91dc43bc1525a118f1f9a791723db1346c68ab | e00f047090337166652ee498bf557dce20aece3a3e59b2b87b50dacc0c0bbd9e | 扩展/其他现用 |
| M0231 | D094:72 S14 | ace-ca-d2.wrong-type-c/certify | R09 | {"stage":"resolve","code":"source.foreign.type-mismatch"}; probe=1 | 5e0a74cbeb9f0c96459daee84d91dc43bc1525a118f1f9a791723db1346c68ab | 991a09f6dd6411748681c3ce46a0f54f9be1e596be8bea5ee2ce47a9eb8f7f32 | 扩展/其他现用 |
| M0232 | D094:77 S14 | ace-ca-d2.wrong-oracle/certify | R09 | "success"; exit=43; probe=1 | dd52c149f82e707c08ad2932462f397aab9217b430f50ccb80a25a45978904f2 | 93d9256cd6a2fdfd1fb3bbc0a7b221aeb497bc5fa2bb8e6e8398e70515e8dbd0 | 扩展/其他现用 |
| M0233 | D095:3 S16 | ace.m1.collq4.l01/ac-native | R34 | "success"; exit=42 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0234 | D095:3 S16 | ace.m1.collq4.l01/av | R21 | "success"; exit=42 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0235 | D095:18 S16 | ace.m1.collq4.l02/ac-native | R34 | "success"; exit=42 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0236 | D095:18 S16 | ace.m1.collq4.l02/av | R21 | "success"; exit=42 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0237 | D095:33 S16 | ace.m1.collq4.l03/ac-native | R34 | "success"; exit=0 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | 3dc852ca17409f8d3629f02a196f2720808c7b86b553c07487d13a6cc19df7c7 | 扩展/其他现用 |
| M0238 | D095:33 S16 | ace.m1.collq4.l03/av | R21 | "success"; exit=0 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | 49034f30f2d4e53d97a4d2a1ab1c049034996af94910f631803b280c2d5e56ef | 扩展/其他现用 |
| M0239 | D095:48 S16 | ace.m1.collq4.l04/ac-native | R34 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0240 | D095:48 S16 | ace.m1.collq4.l04/av | R21 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0241 | D095:63 S16 | ace.m1.collq4.l05/ac-native | R34 | "success"; exit=42 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0242 | D095:63 S16 | ace.m1.collq4.l05/av | R21 | "success"; exit=42 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0243 | D095:78 S16 | ace.m1.collq4.l06/ac-native | R34 | {"stage":"run","code":"mn.trap.list-index"} | de3a85627019b586ef8b9ccb377e60ef57a0686f15076d57e93910f366b5a884 | 470c6eff37a5381692e425b1b0d23b67870b017600d53e068b03219871a9d88d | 扩展/其他现用 |
| M0244 | D095:78 S16 | ace.m1.collq4.l06/av | R21 | {"stage":"run","code":"av.trap.list-index"} | de3a85627019b586ef8b9ccb377e60ef57a0686f15076d57e93910f366b5a884 | b05349e8f3d7bb108626f4747caab82c5fc144215a83e81778576234d75018bb | 扩展/其他现用 |
| M0245 | D095:93 S16 | ace.m1.collq4.l07/ac-native | R34 | "success"; exit=42 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0246 | D095:93 S16 | ace.m1.collq4.l07/av | R21 | "success"; exit=42 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0247 | D095:108 S16 | ace.m1.collq4.v01/ac-native | R34 | "success"; exit=42 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0248 | D095:108 S16 | ace.m1.collq4.v01/av | R21 | "success"; exit=42 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0249 | D095:123 S16 | ace.m1.collq4.v02/ac-native | R34 | "success"; exit=42 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0250 | D095:123 S16 | ace.m1.collq4.v02/av | R21 | "success"; exit=42 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0251 | D095:138 S16 | ace.m1.collq4.v03/ac-native | R34 | "success"; exit=3 | 41767e7e7046efc75d06d790e6392a4da4260baa6b95e8e244964f1883383f97 | faebef089624d502e0b5970da594f3f2f3fce0c2381098f9363bf59424372462 | 扩展/其他现用 |
| M0252 | D095:138 S16 | ace.m1.collq4.v03/av | R21 | "success"; exit=3 | 41767e7e7046efc75d06d790e6392a4da4260baa6b95e8e244964f1883383f97 | 87e7bf3dfb9b8855e6de7614cc8a3a7610f0d44732a269e353870a4a126fda5a | 扩展/其他现用 |
| M0253 | D095:153 S16 | ace.m1.collq4.v04/ac-native | R34 | "success"; exit=42 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0254 | D095:153 S16 | ace.m1.collq4.v04/av | R21 | "success"; exit=42 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0255 | D095:168 S16 | ace.m1.collq4.v05/ac-native | R34 | "success"; exit=42 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0256 | D095:168 S16 | ace.m1.collq4.v05/av | R21 | "success"; exit=42 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0257 | D095:183 S16 | ace.m1.collq4.v06/ac-native | R34 | "success"; exit=42 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0258 | D095:183 S16 | ace.m1.collq4.v06/av | R21 | "success"; exit=42 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0259 | D095:198 S16 | ace.m1.collq4.v07/ac-native | R34 | "success"; exit=42 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0260 | D095:198 S16 | ace.m1.collq4.v07/av | R21 | "success"; exit=42 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0261 | D095:213 S16 | ace.m1.collq4.lstr864/ac-native | R34 | "success"; exit=51 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | da8ca191701ff16c1ef3aefce45b8e785fd4095e297e039e9171ae70af40ff6d | 扩展/其他现用 |
| M0262 | D095:213 S16 | ace.m1.collq4.lstr864/av | R21 | "success"; exit=51 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 2deff781874e1bc519d6550bb848061e69fd12f9f2fa6e37f54bca641b51167d | 扩展/其他现用 |
| M0263 | D095:228 S16 | ace.m1.collq4.fin864a/ac-native | R34 | "success"; exit=42 | cb7d63e1356a05909eaf9a32b069192ce0fa5590da6c34f07b872475ef8474f4 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0264 | D095:228 S16 | ace.m1.collq4.fin864a/av | R21 | "success"; exit=42 | cb7d63e1356a05909eaf9a32b069192ce0fa5590da6c34f07b872475ef8474f4 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0265 | D095:243 S16 | ace.m1.collq4.fin864b/ac-native | R34 | "success"; exit=53 | 6dc3d3af1778bb4f228ed354202756fc65a9aec8fbc1bce06c015e33f5b2be97 | 79037a361639fef2402fdcc70dfca1005f09e541b4032c1a012541720ec94909 | 扩展/其他现用 |
| M0266 | D095:243 S16 | ace.m1.collq4.fin864b/av | R21 | "success"; exit=53 | 6dc3d3af1778bb4f228ed354202756fc65a9aec8fbc1bce06c015e33f5b2be97 | af35e73d6d4d05f37b94a0abe7819b1fb6ee02a85954d9fa7148328af8f09dd0 | 扩展/其他现用 |
| M0267 | D095:258 S16 | ace.m1.collq4.n889-list-variant/ac-native | R34 | "success"; exit=42 | 6f3db730b45eb6535914a3d7de9dfe7d9fc5ac7ca022889e37fd57b4eb810192 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0268 | D095:258 S16 | ace.m1.collq4.n889-list-variant/av | R21 | "success"; exit=42 | 6f3db730b45eb6535914a3d7de9dfe7d9fc5ac7ca022889e37fd57b4eb810192 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0269 | D095:273 S16 | ace.m1.collq4.n889-nested-variant/ac-native | R34 | "success"; exit=42 | 47389d8fd49b7317503b021b10d9158062cb61f0e6f1c7d0f8de1774fa4a7851 | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0270 | D095:273 S16 | ace.m1.collq4.n889-nested-variant/av | R21 | "success"; exit=42 | 47389d8fd49b7317503b021b10d9158062cb61f0e6f1c7d0f8de1774fa4a7851 | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0271 | D095:288 S16 | ace.m1.collq4.n889-optional-variant/ac-native | R34 | "success"; exit=42 | 357cdbe78fee58152ef74221bfbb5493f78b6ea1e7e7194538640d71b807574e | 4612e88a73b8ddecad146cd6856b850386706c76a9a2baf277b3fdb75e41ec9e | 扩展/其他现用 |
| M0272 | D095:288 S16 | ace.m1.collq4.n889-optional-variant/av | R21 | "success"; exit=42 | 357cdbe78fee58152ef74221bfbb5493f78b6ea1e7e7194538640d71b807574e | bf79a636e20ab87f79f727d8fff75ddadde75b80c7684082e91bd381e1462226 | 扩展/其他现用 |
| M0273 | D096:5 S17 | ace.m1.legacy941.b08/ac-native | R37 | "success"; exit=7 | fbf777f2b3174edea44f8d65057fc1f178a93e269f241d61dd0d1d0579ca8145 | c4489e780690c26bc6b927c76a862653858929b34923888c0a2a867cc690df31 | 扩展/其他现用 |
| M0274 | D096:5 S17 | ace.m1.legacy941.b08/av | R24 | "success"; exit=7 | fbf777f2b3174edea44f8d65057fc1f178a93e269f241d61dd0d1d0579ca8145 | 822145fafe319fffb12dedec6bd0d3024d01c13e8d4d9d3c049a77b43150b160 | 扩展/其他现用 |
| M0275 | D096:24 S17 | ace.m1.legacy941.b11/ac-native | R37 | "success"; exit=7 | cb0094b90d2db6537d059b910fc99dbcd3daf1e20e2c3c1cca7f1849e82b6dd2 | c4489e780690c26bc6b927c76a862653858929b34923888c0a2a867cc690df31 | 扩展/其他现用 |
| M0276 | D096:24 S17 | ace.m1.legacy941.b11/av | R24 | "success"; exit=7 | cb0094b90d2db6537d059b910fc99dbcd3daf1e20e2c3c1cca7f1849e82b6dd2 | 822145fafe319fffb12dedec6bd0d3024d01c13e8d4d9d3c049a77b43150b160 | 扩展/其他现用 |
| M0277 | D097:3 S18 | ace.m1.original-data-diag.m09/ac-native | R37 | {"stage":"resolve","code":"source.not-callable"} | c36ba34bb5fc6f3bc35b6affbd194637f46f47f2cc5c107b87e5dc11015621b1 | f135ebe33366296eee264049e99dc30eb82494da047b330150c64394d5756686 | 扩展/其他现用 |
| M0278 | D097:3 S18 | ace.m1.original-data-diag.m09/av | R26 | {"stage":"parse","code":"source.not-callable"} | c36ba34bb5fc6f3bc35b6affbd194637f46f47f2cc5c107b87e5dc11015621b1 | 4296035f561d6419258cdad0a47e59598241b7b57aa1ea22567ac24fc0aee62e | 扩展/其他现用 |
| M0279 | D097:18 S18 | ace.m1.original-data-diag.m10/ac-native | R37 | {"stage":"typecheck","code":"verify.immutable-write"} | c9da6e0549d40a8fb11f900b50a76090bc04b618be543ec6dd00041c0a99f942 | d1567696678cd6ab511f6558ec5e0406eda59a2c38430c6f776ce72e2b2aec26 | 扩展/其他现用 |
| M0280 | D097:18 S18 | ace.m1.original-data-diag.m10/av | R26 | {"stage":"parse","code":"verify.immutable-write"} | c9da6e0549d40a8fb11f900b50a76090bc04b618be543ec6dd00041c0a99f942 | 1afdd17c6152860780d4a85282d7dc9d2124609d5fbf015384691316010352e8 | 扩展/其他现用 |
| M0281 | D097:33 S18 | ace.m1.original-data-diag.m11/ac-native | R37 | {"stage":"typecheck","code":"verify.mut-conflict"} | 2059c671b802c4f83f99ac88e61e1081de10c1d6154a0a7964bf989d4e9a539e | ccf0ce88e7791e6b040e7f9a0f4d3f03874ee440a3098b52783e356a9de2c5fd | 扩展/其他现用 |
| M0282 | D097:33 S18 | ace.m1.original-data-diag.m11/av | R26 | {"stage":"parse","code":"verify.mut-conflict"} | 2059c671b802c4f83f99ac88e61e1081de10c1d6154a0a7964bf989d4e9a539e | d156b391f1a7135384e46da3c8b3be7f6775453ef94264b0ca6f748ea57df4fa | 扩展/其他现用 |
| M0283 | D097:48 S18 | ace.m1.original-data-diag.m12/ac-native | R37 | {"stage":"resolve","code":"source.duplicate"} | 145e3179e2fe81d0ea80af7739e9fbbff4603f63d7287cd1bda67735bbd4af7a | 4c1e44ee91f5556608b8e7bc4f0d33079ba20d8772f0172784cfdbf4920db670 | 扩展/其他现用 |
| M0284 | D097:48 S18 | ace.m1.original-data-diag.m12/av | R29 | {"stage":"parse","code":"source.duplicate"} | 145e3179e2fe81d0ea80af7739e9fbbff4603f63d7287cd1bda67735bbd4af7a | 5b6a0a5cd5a3e6fb3e9a04b94dd5046a614c056c3051e1a04d35b995c552572e | 扩展/其他现用 |
| M0285 | D097:63 S18 | ace.m1.original-data-diag.r09/ac-native | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 5c615ee205db29e23ca75db12dd2ac067bdf2056ba558ed2acda13c50164ad8b | 6015560617cb7f3b70617a4e3457d4db37b0127757b62a5abdc6a5e84798fc97 | 扩展/其他现用 |
| M0286 | D097:63 S18 | ace.m1.original-data-diag.r09/av | R26 | {"stage":"parse","code":"source.type-mismatch"} | 5c615ee205db29e23ca75db12dd2ac067bdf2056ba558ed2acda13c50164ad8b | 5f1c47560168719e112e3b512a0452c0aa896c146dca5f91798d4f99ccf796f2 | 扩展/其他现用 |
| M0287 | D097:78 S18 | ace.m1.original-data-diag.r10/ac-native | R37 | {"stage":"typecheck","code":"verify.use-after-move"} | de19cbac46fe4b4c8e21a3c8442efb5ad2746598f7435ac6818ff86fdf29c7fc | 59d1e9145e8c3a32cac21a17ebcde6d7948232807efa39be9186b23935fe7e84 | 扩展/其他现用 |
| M0288 | D097:78 S18 | ace.m1.original-data-diag.r10/av | R29 | {"stage":"parse","code":"verify.use-after-move"} | de19cbac46fe4b4c8e21a3c8442efb5ad2746598f7435ac6818ff86fdf29c7fc | 81c3bf9edfd495df9fbdb46708bb40544dc47f89346bd8d16220449d65c1f552 | 扩展/其他现用 |
| M0289 | D097:93 S18 | ace.m1.original-data-diag.t06/ac-native | R37 | {"stage":"typecheck","code":"verify.borrow-escape"} | ea10e8f81483653e87517702b06693102f1705db235ca1261cf7f12a81222bb9 | 17347a48d3835dc808493ec7c2bfe5d4bada9644388ef2c4a0bccaa79756b2a9 | 扩展/其他现用 |
| M0290 | D097:93 S18 | ace.m1.original-data-diag.t06/av | R29 | {"stage":"parse","code":"verify.borrow-escape"} | ea10e8f81483653e87517702b06693102f1705db235ca1261cf7f12a81222bb9 | 021623862b541879a6e9b03064629746be0338f86284e100bba16ca855ecce25 | 扩展/其他现用 |
| M0291 | D097:108 S18 | ace.m1.original-data-diag.t07/ac-native | R37 | {"stage":"parse","code":"text.invalid-scalar"} | c0bbf1cb39be98b262013e094f7c557f0de5d1676e5b7426b470517563542c05 | 2dbe0dea93b1dbc42dce8ae742f01f229eb852a3af31ce97aa93d880bb6b8473 | 扩展/其他现用 |
| M0292 | D097:108 S18 | ace.m1.original-data-diag.t07/av | R29 | {"stage":"parse","code":"text.invalid-scalar"} | c0bbf1cb39be98b262013e094f7c557f0de5d1676e5b7426b470517563542c05 | ea7ecf3131ca13a3adf06c7da14bdf6f760554bc9ea6154f63a8eb091f8258df | 扩展/其他现用 |
| M0293 | D098:3 S19 | ace.m1.original-data.m01/ac-native | R37 | "success"; exit=0 | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | 1d66cd1ef52fd747a702605678fe56f4d9eb4e924f1fe3a8aa20048e9083faea | 扩展/其他现用 |
| M0294 | D098:3 S19 | ace.m1.original-data.m01/av | R26 | "success"; exit=0 | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | f9a8d7b83c5b33df430844f3f910b5e37270a5d090fb77d4db41e1ed97e924c5 | 扩展/其他现用 |
| M0295 | D098:18 S19 | ace.m1.original-data.m02/ac-native | R37 | "success"; exit=57 | b34860a4d3b3b9bd4f1f173f9f5e62f3cfeaf8a2ade85ab90778c2b8db654e5c | b39a758026e5e2a56b01c71c026caa6c1a85d8e79620a2a25521e90732b910cc | 扩展/其他现用 |
| M0296 | D098:18 S19 | ace.m1.original-data.m02/av | R29 | "success"; exit=57 | b34860a4d3b3b9bd4f1f173f9f5e62f3cfeaf8a2ade85ab90778c2b8db654e5c | 42a29f90942c47bbd6b1b552d115477c7b2f287fe721fde919918b52a21dc6ff | 扩展/其他现用 |
| M0297 | D098:33 S19 | ace.m1.original-data.m03/ac-native | R37 | "success"; exit=75 | 0f520150182614da12206b6d3430fa506c908cdb9a346ea661d520a03ac01d5b | f446731f88a355f88ae5ebde5e4835314fb30e387c14b60527a75b8f2c8f896f | 扩展/其他现用 |
| M0298 | D098:33 S19 | ace.m1.original-data.m03/av | R26 | "success"; exit=75 | 0f520150182614da12206b6d3430fa506c908cdb9a346ea661d520a03ac01d5b | d4be712763f5c18aacdfc748d12fcb3043cab63ff4ceacbe38f8b6e7a50986cb | 扩展/其他现用 |
| M0299 | D098:48 S19 | ace.m1.original-data.m04/ac-native | R37 | "success"; exit=135 | a893d67b33efbe6f0ca1f36c66c91e532dfe8dc129a6fdb191881d070c50b601 | e7257011db9fe2fc09cf57aca5189dceef7ff4b9dba417fa3f21400d27e3a004 | 扩展/其他现用 |
| M0300 | D098:48 S19 | ace.m1.original-data.m04/av | R26 | "success"; exit=135 | a893d67b33efbe6f0ca1f36c66c91e532dfe8dc129a6fdb191881d070c50b601 | 38fe888ad01d03433fa733ca59f3eb10b7e0c94230fc445089215564651e8251 | 扩展/其他现用 |
| M0301 | D098:63 S19 | ace.m1.original-data.m05/ac-native | R37 | "success"; exit=42 | ba1d9861a18f6d47c6ebaf3487e8958f8db3d7d61cadfa7b2cfff4b7c2cf247a | 97b74dfaeb278934c01673ec75641218f0453deaaabf1df40421ee1be2df9b67 | 扩展/其他现用 |
| M0302 | D098:63 S19 | ace.m1.original-data.m05/av | R29 | "success"; exit=42 | ba1d9861a18f6d47c6ebaf3487e8958f8db3d7d61cadfa7b2cfff4b7c2cf247a | e16dac9288eff7771e266ebe2ed58769944d85a18a49b08286727f4129ea79a8 | 扩展/其他现用 |
| M0303 | D098:78 S19 | ace.m1.original-data.m06/ac-native | R37 | "success"; exit=52 | 06b83e19d1981be3b01db77ede9344012f9aa04a41ee9ac7a998d50807c6bbe1 | 05812085bcae210b672595ef3236358acb94a220762d22ac60c443839064eaa5 | 扩展/其他现用 |
| M0304 | D098:78 S19 | ace.m1.original-data.m06/av | R29 | "success"; exit=52 | 06b83e19d1981be3b01db77ede9344012f9aa04a41ee9ac7a998d50807c6bbe1 | 8249196dc153aab79458565d2849845ba66e0c5172aedb70a77e164484b1f84b | 扩展/其他现用 |
| M0305 | D098:93 S19 | ace.m1.original-data.m07/ac-native | R37 | "success"; exit=84 | 49f20dea97ffd736acb7ee0ba4a95c029265c2cdf5c495c5c08c6c40abac0951 | 1b854b8537ed121c5268df4942278f80f9e5922d6c15e24e4b1cacca2ee6da61 | 扩展/其他现用 |
| M0306 | D098:93 S19 | ace.m1.original-data.m07/av | R26 | "success"; exit=84 | 49f20dea97ffd736acb7ee0ba4a95c029265c2cdf5c495c5c08c6c40abac0951 | 823cd7c4996b7b82489f5592a5ac4570b2f42edbf863a7fca2d4d3a5b12492e8 | 扩展/其他现用 |
| M0307 | D098:108 S19 | ace.m1.original-data.m08/ac-native | R37 | "success"; exit=42 | 23043d246afbd8e0187424ed13c2146e8286d780cac16ac978ab52c8c817bbda | 91973e74553982db689a937d610728edd3a5cdd1d14dbe14d209c610a829d17e | 扩展/其他现用 |
| M0308 | D098:108 S19 | ace.m1.original-data.m08/av | R26 | "success"; exit=42 | 23043d246afbd8e0187424ed13c2146e8286d780cac16ac978ab52c8c817bbda | 8f5b672982a723d0e36f2c1755d87747d007372e38f298e9165ebc201251981d | 扩展/其他现用 |
| M0309 | D098:123 S19 | ace.m1.original-data.r01/ac-native | R37 | "success"; exit=19 | 9126803ce99c46d12066c468b56189704386f5c77acbcf8a564acb76d8e3fdc6 | 8c297d6f7200797349cdee8d6a0e4ad40b70288efdc3936dc9a1a7e80dccd066 | 扩展/其他现用 |
| M0310 | D098:123 S19 | ace.m1.original-data.r01/av | R26 | "success"; exit=19 | 9126803ce99c46d12066c468b56189704386f5c77acbcf8a564acb76d8e3fdc6 | 7cf54ccc68b90d4119793a15155a8d55252e593a28740c8fed4eeb8e951f1f3f | 扩展/其他现用 |
| M0311 | D098:138 S19 | ace.m1.original-data.r02/ac-native | R37 | "success"; exit=9 | 51fe7459ffc8913137c2bd392909d7143b68c2a6aebd47b1066e34f7c16abca0 | c6b99ef22640259330c4ab86bc0bc281af5ccedb07ea3af700a3f3e61894bafb | 扩展/其他现用 |
| M0312 | D098:138 S19 | ace.m1.original-data.r02/av | R26 | "success"; exit=9 | 51fe7459ffc8913137c2bd392909d7143b68c2a6aebd47b1066e34f7c16abca0 | 27c2bfb5bd696cd9fb8074587cbf3105c1b76d8cadb7bbb1beca12bf3c0371d9 | 扩展/其他现用 |
| M0313 | D098:153 S19 | ace.m1.original-data.r03/ac-native | R37 | "success"; exit=23 | 10630e3d095fcd27adc5471137ca6b28e4d36a45ce63d53f7262bf6d8ed5296b | 869f2e7f52df59b05243226311a19349c1d8a96515f03d6ae8c06c06ebb560f0 | 扩展/其他现用 |
| M0314 | D098:153 S19 | ace.m1.original-data.r03/av | R26 | "success"; exit=23 | 10630e3d095fcd27adc5471137ca6b28e4d36a45ce63d53f7262bf6d8ed5296b | fa9830b79607ceb499123bf0da4ad0e6d2d9758a4a4594f6f3afa8e7e111efbb | 扩展/其他现用 |
| M0315 | D098:168 S19 | ace.m1.original-data.r04/ac-native | R37 | "success"; exit=51 | 1382b4c3a35d443e5ebf21ece67d4a6c9b017e443eec9346b5e0ece12d3b7b59 | 9a91bcd69c3001bfe1e3b17e7111e70be255e5efe433fccb26a82f0672e1a522 | 扩展/其他现用 |
| M0316 | D098:168 S19 | ace.m1.original-data.r04/av | R29 | "success"; exit=51 | 1382b4c3a35d443e5ebf21ece67d4a6c9b017e443eec9346b5e0ece12d3b7b59 | 18f8d57e33adf43aee2ce362672ced436b7b985fe333dfd95c44d9391858f836 | 扩展/其他现用 |
| M0317 | D098:183 S19 | ace.m1.original-data.r05/ac-native | R37 | "success"; exit=42 | c971be343e1a6ddcf4fc71474060077c30720d70cb6ec4f81855167b9ef71f4b | 97b74dfaeb278934c01673ec75641218f0453deaaabf1df40421ee1be2df9b67 | 扩展/其他现用 |
| M0318 | D098:183 S19 | ace.m1.original-data.r05/av | R29 | "success"; exit=42 | c971be343e1a6ddcf4fc71474060077c30720d70cb6ec4f81855167b9ef71f4b | e16dac9288eff7771e266ebe2ed58769944d85a18a49b08286727f4129ea79a8 | 扩展/其他现用 |
| M0319 | D098:198 S19 | ace.m1.original-data.r06/ac-native | R37 | "success"; exit=21 | 9851f21afb89ad0053d2578faf3a89843bd44bdaaea3f7e82fcca9412a058f1f | deb6d4a802795031675d787aefa4aa90953d9d034898209a369415bb84fa1ab2 | 扩展/其他现用 |
| M0320 | D098:198 S19 | ace.m1.original-data.r06/av | R26 | "success"; exit=21 | 9851f21afb89ad0053d2578faf3a89843bd44bdaaea3f7e82fcca9412a058f1f | 0cbd6c9381ea740fcc3ccc2fb7f37494545a02172eb05048bf96711803d04c2c | 扩展/其他现用 |
| M0321 | D098:213 S19 | ace.m1.original-data.t01/ac-native | R37 | "success"; exit=38 | e58f7c63892c43ad128e02569c28cbcc0b0f251ee3dd7e5999a7f8204b402c79 | 5d9a14c4ef073ec1075b16a25b18c8c7a86de9bec21fa4a0706c2438dbab46e6 | 扩展/其他现用 |
| M0322 | D098:213 S19 | ace.m1.original-data.t01/av | R29 | "success"; exit=38 | e58f7c63892c43ad128e02569c28cbcc0b0f251ee3dd7e5999a7f8204b402c79 | 38ea2c371f4246421c33b067101edd6123f605869fccc961b849c189c9770d24 | 扩展/其他现用 |
| M0323 | D098:228 S19 | ace.m1.original-data.t02/ac-native | R37 | "success"; exit=27 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 567bf7d38fab934e5bdd582df4c155f08cc8c40f5497740989874fa1bf978df5 | 扩展/其他现用 |
| M0324 | D098:228 S19 | ace.m1.original-data.t02/av | R29 | "success"; exit=27 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 6e6b6eb4d54a2afd91e08113d0b313b69a6801343a24594635bdcd6d8951f2aa | 扩展/其他现用 |
| M0325 | D098:243 S19 | ace.m1.original-data.t04/ac-native | R37 | "success"; exit=128512 | 2935380215cf39547566d92c8ae51586cc3497427f016b8393a82b9508fa1b19 | 489ff386c390bfb0b7535d88e5e8a7d92ea8ccba7c19d306746d5e270dd08417 | 扩展/其他现用 |
| M0326 | D098:243 S19 | ace.m1.original-data.t04/av | R29 | "success"; exit=128512 | 2935380215cf39547566d92c8ae51586cc3497427f016b8393a82b9508fa1b19 | 8371495d1e8f9b6fcad78ebe1493b159abf3a3eb404f29a4f44fc1d705120a2d | 扩展/其他现用 |
| M0327 | D098:258 S19 | ace.m1.original-data.t08/ac-native | R37 | "success"; exit=70 | c6406d69b553bb29c61b0c9805f587f7a7656e46e349e17eeaa94ca66ca4ecf3 | 492e229303e654b3e4b5dc93e5dc6faadabeb4007be68ae283cce4fa545b1a2a | 扩展/其他现用 |
| M0328 | D098:258 S19 | ace.m1.original-data.t08/av | R29 | {"stage":"run","code":"av.trap.str-index"} | c6406d69b553bb29c61b0c9805f587f7a7656e46e349e17eeaa94ca66ca4ecf3 | 4c5d2f50d1fe3d711679c0d767151a9d2ac58551b75a19a4adbdb536d4d20c94 | 扩展/其他现用 |
| M0329 | D099:5 S15 | ace.m1.expanded.q4-001/a2c-emit | R12 | "success"; artifacts=4 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | 037b1cd9280933619f66388fbe9810a3a4ee8d999c0f407461786b5eb6c6b175 | 原511 |
| M0330 | D099:5 S15 | ace.m1.expanded.q4-001/a2r-emit | R15 | "success"; artifacts=4 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | d95f3ed638ddb22ee29dab160dea4378cb476fc566db99b1ab2fed3e7a5bbdc6 | 原511 |
| M0331 | D099:5 S15 | ace.m1.expanded.q4-001/ac-native | R37 | "success"; exit=42 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0332 | D099:5 S15 | ace.m1.expanded.q4-001/av | R24 | "success"; exit=42 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | 96f7c7bf3b73335dc523a423cd08e0821a9fdaee6e52e0207f8fd83d971e1d64 | 原511 |
| M0333 | D100:4 S20 | ace.m1.rtq4.record-text-cert-925.m_copy_snap/ac-native | R37 | "success"; exit=221 | e00dab2c021446636515fe0c0d8f6a08a1877395a968e0842fb1a694f9f6a64d | ce991b8bb856fa9f19cd9dce27bdcf751695ce4eb0df19c5216b4199a3da2b3a | 扩展/其他现用 |
| M0334 | D100:4 S20 | ace.m1.rtq4.record-text-cert-925.m_copy_snap/av | R26 | "success"; exit=221 | e00dab2c021446636515fe0c0d8f6a08a1877395a968e0842fb1a694f9f6a64d | fc0a2c9bd2715854b9b41aa107819717dc26ac7b0978917bb3c909a3957c7a8a | 扩展/其他现用 |
| M0335 | D100:23 S20 | ace.m1.rtq4.record-text-cert-925.m_multi_mut/ac-native | R37 | "success"; exit=12 | 50e7f2a719363b2065b6d3baebbcdbabaacfdfb4dc898e885613433b58f58e94 | cde551f6a17d8fc20e4688bff44fae6fc97d381295084e414c7439453c898fcb | 扩展/其他现用 |
| M0336 | D100:23 S20 | ace.m1.rtq4.record-text-cert-925.m_multi_mut/av | R26 | "success"; exit=12 | 50e7f2a719363b2065b6d3baebbcdbabaacfdfb4dc898e885613433b58f58e94 | b0600e2d3c61da0aa8250a5aacf6fed090a3417de960561975649d3b36cc70f7 | 扩展/其他现用 |
| M0337 | D100:42 S20 | ace.m1.rtq4.record-text-cert-925.m_view_record/ac-native | R37 | "success"; exit=42 | dbae7f11855b814d0aabe49041c8594d374f72140474eaec21c3a4b18fb2496e | 91973e74553982db689a937d610728edd3a5cdd1d14dbe14d209c610a829d17e | 扩展/其他现用 |
| M0338 | D100:42 S20 | ace.m1.rtq4.record-text-cert-925.m_view_record/av | R26 | "success"; exit=42 | dbae7f11855b814d0aabe49041c8594d374f72140474eaec21c3a4b18fb2496e | 8f5b672982a723d0e36f2c1755d87747d007372e38f298e9165ebc201251981d | 扩展/其他现用 |
| M0339 | D100:61 S20 | ace.m1.rtq4.record-text-cert-925.m_mut_text/ac-native | R37 | "success"; exit=4 | 97785e4bd96c5e1d079f53d1591a92ba3addc2eb4e1557cb8a1a2a52a00eea06 | 331c3c37bb2fd4cc43de397eefd1b3d461090f23f01dae39a879a4caabc740bd | 扩展/其他现用 |
| M0340 | D100:61 S20 | ace.m1.rtq4.record-text-cert-925.m_mut_text/av | R29 | "success"; exit=4 | 97785e4bd96c5e1d079f53d1591a92ba3addc2eb4e1557cb8a1a2a52a00eea06 | 64a885d9039297210d5d7481598ea194766e8319fd4ff0586f3f86e3dd7dfa78 | 扩展/其他现用 |
| M0341 | D100:80 S20 | ace.m1.rtq4.record-text-cert-925.m_borrow_return/ac-native | R37 | "success"; exit=2 | 19ed3fb99650139c78fcb81d1da3a5fba853562553bc140386a1467f9e94d281 | 09ab2751a13449c0d0c8e22720012c6fd797118022ed1299ca1fd90f5eb95db4 | 扩展/其他现用 |
| M0342 | D100:80 S20 | ace.m1.rtq4.record-text-cert-925.m_borrow_return/av | R29 | "success"; exit=2 | 19ed3fb99650139c78fcb81d1da3a5fba853562553bc140386a1467f9e94d281 | aa9ae4901291b0d52e521b197cf43b4419366cad2895537947a82d518c717d87 | 扩展/其他现用 |
| M0343 | D100:99 S20 | ace.m1.rtq4.record-text-cert-925.m_owned_return/ac-native | R37 | "success"; exit=22 | 30967409630bfd561b6e7831bd214f8c2bc5f29a1544f96a4d4a00af7234b5af | cff6649cee66f7f33506e2302a22a3703808599b6b06b309891f4f8c6943ed7a | 扩展/其他现用 |
| M0344 | D100:99 S20 | ace.m1.rtq4.record-text-cert-925.m_owned_return/av | R29 | "success"; exit=22 | 30967409630bfd561b6e7831bd214f8c2bc5f29a1544f96a4d4a00af7234b5af | 751b9e485ddec08b79e082377aea61a8a9bbd6ffbd551b661f7cf65caffc0da1 | 扩展/其他现用 |
| M0345 | D100:118 S20 | ace.m1.rtq4.record-text-cert-925.m_named_ctor/ac-native | R37 | "success"; exit=12 | a13fae4235411f21b6d488f1cde5a90b387e8015c780c23e81ee091d56f3213f | cde551f6a17d8fc20e4688bff44fae6fc97d381295084e414c7439453c898fcb | 扩展/其他现用 |
| M0346 | D100:118 S20 | ace.m1.rtq4.record-text-cert-925.m_named_ctor/av | R26 | "success"; exit=12 | a13fae4235411f21b6d488f1cde5a90b387e8015c780c23e81ee091d56f3213f | b0600e2d3c61da0aa8250a5aacf6fed090a3417de960561975649d3b36cc70f7 | 扩展/其他现用 |
| M0347 | D100:137 S20 | ace.m1.rtq4.record-text-cert-925.m_name_collision/ac-native | R37 | "success"; exit=137 | e4ba5b82ed38f1afcb0864772d8b364870808bd4f7afd7294d3d3e73d5253fdb | 66ef7df859a6424b7fcd4e8f85db2582b45c50d73db530d8b8de52c1f15b700c | 扩展/其他现用 |
| M0348 | D100:137 S20 | ace.m1.rtq4.record-text-cert-925.m_name_collision/av | R26 | "success"; exit=137 | e4ba5b82ed38f1afcb0864772d8b364870808bd4f7afd7294d3d3e73d5253fdb | ca0bc5e804ab29b6cd136a00eba52e109941c6d0681d2c6b580a50df173fbb5e | 扩展/其他现用 |
| M0349 | D100:156 S20 | ace.m1.rtq4.record-text-cert-925.t_char_at_neg/ac-native | R37 | "success"; exit=70; stderr#9bc5683d8052 | 22bad6ca714d78855bc7f1bc428c9fcf1f046e62e6cfb38180225cb332748278 | 23359a28033c9d0e6d0ba9f490580c9d7f214e818bce0113530d13b56e0ed4f2 | 扩展/其他现用 |
| M0350 | D100:156 S20 | ace.m1.rtq4.record-text-cert-925.t_char_at_neg/av | R29 | {"stage":"run","code":"av.trap.str-index"} | 22bad6ca714d78855bc7f1bc428c9fcf1f046e62e6cfb38180225cb332748278 | 4c5d2f50d1fe3d711679c0d767151a9d2ac58551b75a19a4adbdb536d4d20c94 | 扩展/其他现用 |
| M0351 | D100:175 S20 | ace.m1.rtq4.record-text-cert-925.t_slice_oob/ac-native | R37 | "success"; exit=70; stderr#d8135c5c1d51 | 738655d8927856098e1cc2a4c24d1e53ab83c4e2ae7f238d38af4765389f967d | 5b97b567a7352eb45fc4c6e55ef036aabbcbb800b398135c75ee9c152f538a17 | 扩展/其他现用 |
| M0352 | D100:175 S20 | ace.m1.rtq4.record-text-cert-925.t_slice_oob/av | R29 | {"stage":"run","code":"av.trap.str-index"} | 738655d8927856098e1cc2a4c24d1e53ab83c4e2ae7f238d38af4765389f967d | 4c5d2f50d1fe3d711679c0d767151a9d2ac58551b75a19a4adbdb536d4d20c94 | 扩展/其他现用 |
| M0353 | D100:194 S20 | ace.m1.rtq4.record-text-cert-925.t_surrogate/ac-native | R37 | "success"; exit=70; stderr#85460ca96ec8 | e8861447a97bdcb3a947ccd9bd2545054cfddf860656017471f047a923eb312e | 469193e2299eb37c63229d0ae16d5b564e50947fb07b326ae462f98a0ed4aed8 | 扩展/其他现用 |
| M0354 | D100:194 S20 | ace.m1.rtq4.record-text-cert-925.t_surrogate/av | R29 | {"stage":"run","code":"av.trap.char-invalid"} | e8861447a97bdcb3a947ccd9bd2545054cfddf860656017471f047a923eb312e | 1284bb2de55cf43bfe616cb0ed72baa3e6a5d745cc761c070d8a8d07100802e3 | 扩展/其他现用 |
| M0355 | D100:213 S20 | ace.m1.rtq4.record-text-base.r07/ac-native | R37 | "success"; exit=42 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | 91973e74553982db689a937d610728edd3a5cdd1d14dbe14d209c610a829d17e | 扩展/其他现用 |
| M0356 | D100:213 S20 | ace.m1.rtq4.record-text-base.r07/av | R26 | "success"; exit=42 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | 8f5b672982a723d0e36f2c1755d87747d007372e38f298e9165ebc201251981d | 扩展/其他现用 |
| M0357 | D100:232 S20 | ace.m1.rtq4.record-text-base.r08/ac-native | R37 | "success"; exit=42 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 91973e74553982db689a937d610728edd3a5cdd1d14dbe14d209c610a829d17e | 扩展/其他现用 |
| M0358 | D100:232 S20 | ace.m1.rtq4.record-text-base.r08/av | R26 | "success"; exit=42 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 8f5b672982a723d0e36f2c1755d87747d007372e38f298e9165ebc201251981d | 扩展/其他现用 |
| M0359 | D100:251 S20 | ace.m1.rtq4.record-text-base.t03/ac-native | R37 | "success"; exit=23 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | de3f210db57220eb3b5ec1f2864ba711d34269040b0ad4a31380c81ddfb238d2 | 扩展/其他现用 |
| M0360 | D100:251 S20 | ace.m1.rtq4.record-text-base.t03/av | R29 | "success"; exit=23 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | 4f4ed9eba4c67c4848596e3a13b0151ceb84b98cec047ea6e4444ba46552dab7 | 扩展/其他现用 |
| M0361 | D100:270 S20 | ace.m1.rtq4.record-text-base.t05/ac-native | R37 | "success"; exit=42 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | 97b74dfaeb278934c01673ec75641218f0453deaaabf1df40421ee1be2df9b67 | 扩展/其他现用 |
| M0362 | D100:270 S20 | ace.m1.rtq4.record-text-base.t05/av | R29 | "success"; exit=42 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | e16dac9288eff7771e266ebe2ed58769944d85a18a49b08286727f4129ea79a8 | 扩展/其他现用 |
| M0363 | D100:289 S20 | ace.m1.rtq4.record-text-base.c04/ac-native | R37 | "success"; exit=42 | 05c593750e21779409dc23b0a20f6aa5b7ff320092f890831c9a16fc8d7ef89b | 97b74dfaeb278934c01673ec75641218f0453deaaabf1df40421ee1be2df9b67 | 扩展/其他现用 |
| M0364 | D100:289 S20 | ace.m1.rtq4.record-text-base.c04/av | R29 | "success"; exit=42 | 05c593750e21779409dc23b0a20f6aa5b7ff320092f890831c9a16fc8d7ef89b | e16dac9288eff7771e266ebe2ed58769944d85a18a49b08286727f4129ea79a8 | 扩展/其他现用 |
| M0365 | D101:5 S21 | ace.m1.scalarq4.core-c02-paren-newline/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | 239af14fbe6b6a359b3e64d1a72c4852295017f4974ccfbce9822feef55f4fa1 | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0366 | D101:5 S21 | ace.m1.scalarq4.core-c02-paren-newline/av | R24 | "success"; exit=3 | 239af14fbe6b6a359b3e64d1a72c4852295017f4974ccfbce9822feef55f4fa1 | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0367 | D101:24 S21 | ace.m1.scalarq4.core-c05-ac-start-name/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | d5594e9c97a4906718d27ecf04d1d673aa8e6428d1f9a324643f4ddbb83dbecb | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0368 | D101:24 S21 | ace.m1.scalarq4.core-c05-ac-start-name/av | R24 | "success"; exit=3 | d5594e9c97a4906718d27ecf04d1d673aa8e6428d1f9a324643f4ddbb83dbecb | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0369 | D101:43 S21 | ace.m1.scalarq4.core-c06-exitprocess-name/ac-native | R37 | "success"; exit=4; stderr#7df92993a715 | d624762ef53cdaf4940aebfccb6d0a7c0dc1d16d1cb68b638c6b5112bdfcc7d1 | 19551233abc7ce3610d2cf3d8cf75a97e97f657bdb8d32d517511c57db0b38df | 扩展/其他现用 |
| M0370 | D101:43 S21 | ace.m1.scalarq4.core-c06-exitprocess-name/av | R24 | "success"; exit=4 | d624762ef53cdaf4940aebfccb6d0a7c0dc1d16d1cb68b638c6b5112bdfcc7d1 | 7767de78e0dcc28da3094fc34ea8173fafd1f472364a016e38ee1ea3f203dd12 | 扩展/其他现用 |
| M0371 | D101:62 S21 | ace.m1.scalarq4.core-d01-call-multiline/ac-native | R37 | "success"; exit=5; stderr#7df92993a715 | 0ea653b3004edd35c84631d0650a5516eb8d4a7559b9090ca121e91a3662dde2 | 6bfa5c8f5515af46b5611c3e2f50a2a53c31def79f9f3916c8d4c2fd868d8470 | 扩展/其他现用 |
| M0372 | D101:62 S21 | ace.m1.scalarq4.core-d01-call-multiline/av | R24 | "success"; exit=5 | 0ea653b3004edd35c84631d0650a5516eb8d4a7559b9090ca121e91a3662dde2 | 54382647d6870b5dbd6d93a3f6222be4d70e4a4cb397ccd18f369cda4094ed5e | 扩展/其他现用 |
| M0373 | D101:81 S21 | ace.m1.scalarq4.core-d02-paren-multiline/ac-native | R37 | "success"; exit=6; stderr#7df92993a715 | 2857b4e28c99e31af7e8a7576d5e887d13fe5538977f0090280d7b23cb83bfce | 8fa6a6a3fe6d86f176d5b022a5885572bf69ba7cf66c1b8d11e24e46cfc9532f | 扩展/其他现用 |
| M0374 | D101:81 S21 | ace.m1.scalarq4.core-d02-paren-multiline/av | R24 | "success"; exit=6 | 2857b4e28c99e31af7e8a7576d5e887d13fe5538977f0090280d7b23cb83bfce | 7d8f1759e39aa6f1f8dc23fed9eb46c73bda04181554a01c1a8162a1f0437c0b | 扩展/其他现用 |
| M0375 | D101:100 S21 | ace.m1.scalarq4.core-d03-negative-multiline/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 353d1d55e88f376d3ee6e4e685e4ea6c5c975542a739ff34d2ee174a6282b559 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0376 | D101:100 S21 | ace.m1.scalarq4.core-d03-negative-multiline/av | R24 | "success"; exit=1 | 353d1d55e88f376d3ee6e4e685e4ea6c5c975542a739ff34d2ee174a6282b559 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0377 | D101:119 S21 | ace.m1.scalarq4.core-d04-nested-paren/ac-native | R37 | "success"; exit=6; stderr#7df92993a715 | 514e32f95575e8f2372c5c8751ab003336c4fab8ae1617046ce66a3a0393f979 | 8fa6a6a3fe6d86f176d5b022a5885572bf69ba7cf66c1b8d11e24e46cfc9532f | 扩展/其他现用 |
| M0378 | D101:119 S21 | ace.m1.scalarq4.core-d04-nested-paren/av | R24 | "success"; exit=6 | 514e32f95575e8f2372c5c8751ab003336c4fab8ae1617046ce66a3a0393f979 | 7d8f1759e39aa6f1f8dc23fed9eb46c73bda04181554a01c1a8162a1f0437c0b | 扩展/其他现用 |
| M0379 | D101:138 S21 | ace.m1.scalarq4.core-d05-call-no-tail/ac-native | R37 | "success"; exit=2; stderr#7df92993a715 | 52c3bacadbb63b9a614cc54e67b3cbfd568a5da0014d3ed6c8a0d4f4f8328ce1 | f09b5db2979dfd7a45869f698ce2dda45209e9aec1147ad4636f7e16156d34a9 | 扩展/其他现用 |
| M0380 | D101:138 S21 | ace.m1.scalarq4.core-d05-call-no-tail/av | R24 | "success"; exit=2 | 52c3bacadbb63b9a614cc54e67b3cbfd568a5da0014d3ed6c8a0d4f4f8328ce1 | a6494ec419566d22f4759c29aed077a57a4be54ee255022fcac6d32abaf5a57e | 扩展/其他现用 |
| M0381 | D101:157 S21 | ace.m1.scalarq4.core-d07-colliding-prefix/ac-native | R37 | "success"; exit=7; stderr#7df92993a715 | 4abb9053b687d88b92159966879f8c1121ab9c56254ec3f51209ed42504d415e | 44040ac73ad252afe8ecde0da3a1b29d9dd6208b513d718d7cc474b66a850517 | 扩展/其他现用 |
| M0382 | D101:157 S21 | ace.m1.scalarq4.core-d07-colliding-prefix/av | R24 | "success"; exit=7 | 4abb9053b687d88b92159966879f8c1121ab9c56254ec3f51209ed42504d415e | 2427f51bbe8eca305828d3f45326a94b859febcbf89bd0f98a4d66b49cb6a29f | 扩展/其他现用 |
| M0383 | D101:176 S21 | ace.m1.scalarq4.core-e01-nesting-16/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 0ad362d7ae196c57f3c9f3d58526aeded20608d055c8798f4cf9d4a0d1cf4ae4 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0384 | D101:176 S21 | ace.m1.scalarq4.core-e01-nesting-16/av | R24 | "success"; exit=1 | 0ad362d7ae196c57f3c9f3d58526aeded20608d055c8798f4cf9d4a0d1cf4ae4 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0385 | D101:195 S21 | ace.m1.scalarq4.core-f04-parens-99/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 9856940c603b5adc614c55c31123224b42c4ec56d124b8ca6c9180094b8b0128 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0386 | D101:195 S21 | ace.m1.scalarq4.core-f04-parens-99/av | R24 | "success"; exit=1 | 9856940c603b5adc614c55c31123224b42c4ec56d124b8ca6c9180094b8b0128 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0387 | D101:214 S21 | ace.m1.scalarq4.core-f11-add-chain-100/ac-native | R37 | "success"; exit=300; stderr#7df92993a715 | 1c3eae4c1c7400e89fc8e8d332b366b359e73e920e07b0cfa3f255bdeb960640 | 93c9e3fa23f5f41488c5f0eb3bc199373afb921909987ab3e2ff69a54a18906d | 扩展/其他现用 |
| M0388 | D101:214 S21 | ace.m1.scalarq4.core-f11-add-chain-100/av | R24 | "success"; exit=300 | 1c3eae4c1c7400e89fc8e8d332b366b359e73e920e07b0cfa3f255bdeb960640 | d16c15d3309e460f0fdc7e578be98c9223014e8ef8868b47629ce513fa5254db | 扩展/其他现用 |
| M0389 | D101:233 S21 | ace.m1.scalarq4.core-g02-parens-200/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 77875b4f2b0790405788e04d9c55fd3e89fdecce35683d9e13cabcce65b60847 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0390 | D101:233 S21 | ace.m1.scalarq4.core-g02-parens-200/av | R24 | "success"; exit=1 | 77875b4f2b0790405788e04d9c55fd3e89fdecce35683d9e13cabcce65b60847 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0391 | D101:252 S21 | ace.m1.scalarq4.core-m01-comma-args/ac-native | R37 | "success"; exit=42; stderr#7df92993a715 | 328dab128d17d9aa28059a5860a1ef01c6ebd5ec090e0412db52533637892ca2 | 78a9962c3d69eb04dca4ead5c033edfe0e1ccce887de1e02ec29ece9fdca7c1b | 扩展/其他现用 |
| M0392 | D101:252 S21 | ace.m1.scalarq4.core-m01-comma-args/av | R24 | "success"; exit=42 | 328dab128d17d9aa28059a5860a1ef01c6ebd5ec090e0412db52533637892ca2 | a31900d8b0a32397dee22f50ef870cbe8c924facec1c7a8275bbd37025717c5f | 扩展/其他现用 |
| M0393 | D101:271 S21 | ace.m1.scalarq4.core-m02-named-binop/ac-native | R37 | "success"; exit=42; stderr#7df92993a715 | 1ff889874cf40d0c9dfa081b16ce78426f787cf15d224c506d039c543f36e73f | 78a9962c3d69eb04dca4ead5c033edfe0e1ccce887de1e02ec29ece9fdca7c1b | 扩展/其他现用 |
| M0394 | D101:271 S21 | ace.m1.scalarq4.core-m02-named-binop/av | R24 | "success"; exit=42 | 1ff889874cf40d0c9dfa081b16ce78426f787cf15d224c506d039c543f36e73f | a31900d8b0a32397dee22f50ef870cbe8c924facec1c7a8275bbd37025717c5f | 扩展/其他现用 |
| M0395 | D101:290 S21 | ace.m1.scalarq4.core-m06-nested-call/ac-native | R37 | "success"; exit=192; stderr#7df92993a715 | 40e8e5607ae6a49e3fc630c0ca5511f8c84c3763d80c2b8dddf8195ca405ded7 | c485a0988560874b31bb4166b73a5e7dae874167098bdbb8671bfeb5a3b840e0 | 扩展/其他现用 |
| M0396 | D101:290 S21 | ace.m1.scalarq4.core-m06-nested-call/av | R24 | "success"; exit=192 | 40e8e5607ae6a49e3fc630c0ca5511f8c84c3763d80c2b8dddf8195ca405ded7 | 73859dc784d028134fddf47986bf121d07eb97ffb163f89fe2fe606149de1310 | 扩展/其他现用 |
| M0397 | D101:309 S21 | ace.m1.scalarq4.core-m07-bool-arg/ac-native | R37 | "success"; exit=5; stderr#7df92993a715 | 3cc544e5a14d1c8e6a900785076acc4cd7582e3ed8ddde6603de4784d11de878 | 6bfa5c8f5515af46b5611c3e2f50a2a53c31def79f9f3916c8d4c2fd868d8470 | 扩展/其他现用 |
| M0398 | D101:309 S21 | ace.m1.scalarq4.core-m07-bool-arg/av | R24 | "success"; exit=5 | 3cc544e5a14d1c8e6a900785076acc4cd7582e3ed8ddde6603de4784d11de878 | 54382647d6870b5dbd6d93a3f6222be4d70e4a4cb397ccd18f369cda4094ed5e | 扩展/其他现用 |
| M0399 | D101:328 S21 | ace.m1.scalarq4.core-m10-paren-args/ac-native | R37 | "success"; exit=42; stderr#7df92993a715 | f924190fe7ab35d3e17fc6cad895629e9716d5996e6c36ec9bff047d340b0896 | 78a9962c3d69eb04dca4ead5c033edfe0e1ccce887de1e02ec29ece9fdca7c1b | 扩展/其他现用 |
| M0400 | D101:328 S21 | ace.m1.scalarq4.core-m10-paren-args/av | R24 | "success"; exit=42 | f924190fe7ab35d3e17fc6cad895629e9716d5996e6c36ec9bff047d340b0896 | a31900d8b0a32397dee22f50ef870cbe8c924facec1c7a8275bbd37025717c5f | 扩展/其他现用 |
| M0401 | D101:347 S21 | ace.m1.scalarq4.core-m22-named-newline/ac-native | R37 | "success"; exit=33; stderr#7df92993a715 | 9dc06edaf30cf8e72de359930652a838d46139676a3ac57ea92529b7b000b704 | ce888840b0470a8ca298835e9c3b27820e24c22645436f49f9e06288b3c0c868 | 扩展/其他现用 |
| M0402 | D101:347 S21 | ace.m1.scalarq4.core-m22-named-newline/av | R24 | "success"; exit=33 | 9dc06edaf30cf8e72de359930652a838d46139676a3ac57ea92529b7b000b704 | 35cae0c81e5665874fb2c292592f9c2794dea6cff7da4abdba358878ea2322e4 | 扩展/其他现用 |
| M0403 | D101:366 S21 | ace.m1.scalarq4.core-p01-precedence/ac-native | R37 | "success"; exit=14; stderr#7df92993a715 | e9f06c6db735783c309eb67d6bc2d801d5f8d613d0b50d39670a1d5dc66f1188 | dc2653c798b46d530d4701ded836e523cfdceeb04da86f32b7e7dcb1c46893bb | 扩展/其他现用 |
| M0404 | D101:366 S21 | ace.m1.scalarq4.core-p01-precedence/av | R24 | "success"; exit=14 | e9f06c6db735783c309eb67d6bc2d801d5f8d613d0b50d39670a1d5dc66f1188 | fa88806312beebbaaae58feb420b0949e08b54fc123ee390a8df591cbb9abd23 | 扩展/其他现用 |
| M0405 | D101:385 S21 | ace.m1.scalarq4.core-p02-signed-pos/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 7394378b33012c635f5631ce67ccf6381cf8c781aaa9929210c6cfe7b0820940 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0406 | D101:385 S21 | ace.m1.scalarq4.core-p02-signed-pos/av | R24 | "success"; exit=1 | 7394378b33012c635f5631ce67ccf6381cf8c781aaa9929210c6cfe7b0820940 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0407 | D101:404 S21 | ace.m1.scalarq4.core-p03-bool-param/ac-native | R37 | "success"; exit=7; stderr#7df92993a715 | d1276376ad0acf217633bcd195ab113a8b2c5da7b3614a121e8de157e41ae769 | 44040ac73ad252afe8ecde0da3a1b29d9dd6208b513d718d7cc474b66a850517 | 扩展/其他现用 |
| M0408 | D101:404 S21 | ace.m1.scalarq4.core-p03-bool-param/av | R24 | "success"; exit=7 | d1276376ad0acf217633bcd195ab113a8b2c5da7b3614a121e8de157e41ae769 | 2427f51bbe8eca305828d3f45326a94b859febcbf89bd0f98a4d66b49cb6a29f | 扩展/其他现用 |
| M0409 | D101:423 S21 | ace.m1.scalarq4.core-p04-while/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0410 | D101:423 S21 | ace.m1.scalarq4.core-p04-while/av | R24 | "success"; exit=3 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0411 | D101:442 S21 | ace.m1.scalarq4.core-p05-while-zero/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 4f558c850d1bef74ff12adb01884b2ec8570eb3ed5edf344065fd61a3cf0fa19 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0412 | D101:442 S21 | ace.m1.scalarq4.core-p05-while-zero/av | R24 | "success"; exit=1 | 4f558c850d1bef74ff12adb01884b2ec8570eb3ed5edf344065fd61a3cf0fa19 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0413 | D101:461 S21 | ace.m1.scalarq4.core-p06-continue/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | bbd902d18bc49b96bf6a7e44fdf2fc4947143c99d888828e87c13d3d5493b94b | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0414 | D101:461 S21 | ace.m1.scalarq4.core-p06-continue/av | R24 | "success"; exit=3 | bbd902d18bc49b96bf6a7e44fdf2fc4947143c99d888828e87c13d3d5493b94b | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0415 | D101:480 S21 | ace.m1.scalarq4.core-p07-break-nearest/ac-native | R37 | "success"; exit=5; stderr#7df92993a715 | 4ad2088c87e5f4f650b9d3205639dfca1efd12e40a286d3042b99dd011ba5c6f | 6bfa5c8f5515af46b5611c3e2f50a2a53c31def79f9f3916c8d4c2fd868d8470 | 扩展/其他现用 |
| M0416 | D101:480 S21 | ace.m1.scalarq4.core-p07-break-nearest/av | R24 | "success"; exit=5 | 4ad2088c87e5f4f650b9d3205639dfca1efd12e40a286d3042b99dd011ba5c6f | 54382647d6870b5dbd6d93a3f6222be4d70e4a4cb397ccd18f369cda4094ed5e | 扩展/其他现用 |
| M0417 | D101:499 S21 | ace.m1.scalarq4.core-p08-recursion/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | c84d27674f1567b69e9625bffe27eae2d25efa7fcff60e701a3e4aac5286655c | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0418 | D101:499 S21 | ace.m1.scalarq4.core-p08-recursion/av | R24 | "success"; exit=3 | c84d27674f1567b69e9625bffe27eae2d25efa7fcff60e701a3e4aac5286655c | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0419 | D101:518 S21 | ace.m1.scalarq4.core-p09-forward-sig/ac-native | R37 | "success"; exit=8; stderr#7df92993a715 | c8eec3480883db68d99a604e13949a4f468b90e7d36575baf4bb71f461393480 | af94ce52310f9bd4d1e8ae46b3a30c2b49c4db82ea33753bf24adb6f2240429f | 扩展/其他现用 |
| M0420 | D101:518 S21 | ace.m1.scalarq4.core-p09-forward-sig/av | R24 | "success"; exit=8 | c8eec3480883db68d99a604e13949a4f468b90e7d36575baf4bb71f461393480 | e2eb7fef4c7241b978aa0db96f4073ce599d5222bb5ce4f71fd3778ff821c81c | 扩展/其他现用 |
| M0421 | D101:537 S21 | ace.m1.scalarq4.core-p10-shadow/ac-native | R37 | "success"; exit=5; stderr#7df92993a715 | 270653a6725975a7d90a882c2f81561fa9862deda9343c3360832c1b281490d0 | 6bfa5c8f5515af46b5611c3e2f50a2a53c31def79f9f3916c8d4c2fd868d8470 | 扩展/其他现用 |
| M0422 | D101:537 S21 | ace.m1.scalarq4.core-p10-shadow/av | R24 | "success"; exit=5 | 270653a6725975a7d90a882c2f81561fa9862deda9343c3360832c1b281490d0 | 54382647d6870b5dbd6d93a3f6222be4d70e4a4cb397ccd18f369cda4094ed5e | 扩展/其他现用 |
| M0423 | D101:556 S21 | ace.m1.scalarq4.core-p11-min-max/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | c95645354fbe620f96c1a5f21e339667f636c6105fd2948c9d0d100752324280 | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0424 | D101:556 S21 | ace.m1.scalarq4.core-p11-min-max/av | R24 | "success"; exit=0 | c95645354fbe620f96c1a5f21e339667f636c6105fd2948c9d0d100752324280 | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0425 | D101:575 S21 | ace.m1.scalarq4.core-p12-unexecuted/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | 1e9a66abf0d874fb7e4398d5230bfccc856009cc6e4cbaf57fe558f5ea3740fe | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0426 | D101:575 S21 | ace.m1.scalarq4.core-p12-unexecuted/av | R24 | "success"; exit=0 | 1e9a66abf0d874fb7e4398d5230bfccc856009cc6e4cbaf57fe558f5ea3740fe | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0427 | D101:594 S21 | ace.m1.scalarq4.core-p13-add-overflow/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | d3a0e872e6b62cf8a119d2df92d98758a4cef1e0965e16b17e6611746fca41af | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0428 | D101:594 S21 | ace.m1.scalarq4.core-p13-add-overflow/av | R24 | {"stage":"run","code":"av.trap.int-overflow"} | d3a0e872e6b62cf8a119d2df92d98758a4cef1e0965e16b17e6611746fca41af | 21c769caf9ef3ac7ace95361b3fca998f96422b9ecacb4916d343e410be34610 | 扩展/其他现用 |
| M0429 | D101:613 S21 | ace.m1.scalarq4.core-p14-mul-overflow/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | 6db3d4b0cad6c96ee853679c84619186aefd02a1b0af2da1d98642961eb877ab | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0430 | D101:613 S21 | ace.m1.scalarq4.core-p14-mul-overflow/av | R24 | {"stage":"run","code":"av.trap.int-overflow"} | 6db3d4b0cad6c96ee853679c84619186aefd02a1b0af2da1d98642961eb877ab | 21c769caf9ef3ac7ace95361b3fca998f96422b9ecacb4916d343e410be34610 | 扩展/其他现用 |
| M0431 | D101:632 S21 | ace.m1.scalarq4.core-p17-hetero-named/ac-native | R37 | "success"; exit=6; stderr#7df92993a715 | 814c2aba33674cdc0d07b7c773cd36ddac69226151a7650fc2949fcd5ad62ddc | 8fa6a6a3fe6d86f176d5b022a5885572bf69ba7cf66c1b8d11e24e46cfc9532f | 扩展/其他现用 |
| M0432 | D101:632 S21 | ace.m1.scalarq4.core-p17-hetero-named/av | R24 | "success"; exit=6 | 814c2aba33674cdc0d07b7c773cd36ddac69226151a7650fc2949fcd5ad62ddc | 7d8f1759e39aa6f1f8dc23fed9eb46c73bda04181554a01c1a8162a1f0437c0b | 扩展/其他现用 |
| M0433 | D101:651 S21 | ace.m1.scalarq4.core-p18-bool-return/ac-native | R37 | "success"; exit=4; stderr#7df92993a715 | 5fb17b26f7fc43eff8ad77c89e1d06874d4df5adecf1efb9e478297599c1f3fb | 19551233abc7ce3610d2cf3d8cf75a97e97f657bdb8d32d517511c57db0b38df | 扩展/其他现用 |
| M0434 | D101:651 S21 | ace.m1.scalarq4.core-p18-bool-return/av | R24 | "success"; exit=4 | 5fb17b26f7fc43eff8ad77c89e1d06874d4df5adecf1efb9e478297599c1f3fb | 7767de78e0dcc28da3094fc34ea8173fafd1f472364a016e38ee1ea3f203dd12 | 扩展/其他现用 |
| M0435 | D101:670 S21 | ace.m1.scalarq4.core-p19-user-mark-a/ac-native | R37 | "success"; exit=6; stderr#7df92993a715 | b7a1ef8c6bbad04f6e909b0da8e6519bcc46028ac01abc1787ea03af1db38a20 | 8fa6a6a3fe6d86f176d5b022a5885572bf69ba7cf66c1b8d11e24e46cfc9532f | 扩展/其他现用 |
| M0436 | D101:670 S21 | ace.m1.scalarq4.core-p19-user-mark-a/av | R24 | "success"; exit=6 | b7a1ef8c6bbad04f6e909b0da8e6519bcc46028ac01abc1787ea03af1db38a20 | 7d8f1759e39aa6f1f8dc23fed9eb46c73bda04181554a01c1a8162a1f0437c0b | 扩展/其他现用 |
| M0437 | D101:689 S21 | ace.m1.scalarq4.core-p20-utf8-crlf-min/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | 1a39a04f58f180351e858425a15571f719fe6187d5208d32a84db0dc931dd9b0 | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0438 | D101:689 S21 | ace.m1.scalarq4.core-p20-utf8-crlf-min/av | R24 | "success"; exit=0 | 1a39a04f58f180351e858425a15571f719fe6187d5208d32a84db0dc931dd9b0 | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0439 | D101:708 S21 | ace.m1.scalarq4.sc-a-minus-1/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | b7927e4199ab8bb362c52214f4e34c99d2c07c7a7497784520a34fb7954dc40d | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0440 | D101:708 S21 | ace.m1.scalarq4.sc-a-minus-1/av | R24 | "success"; exit=0 | b7927e4199ab8bb362c52214f4e34c99d2c07c7a7497784520a34fb7954dc40d | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0441 | D101:727 S21 | ace.m1.scalarq4.sc-binop-args/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | cf58bab15a1025ce62fd01e8a8d11d6d6e78b884df8f24f51fe524b748c29acb | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0442 | D101:727 S21 | ace.m1.scalarq4.sc-binop-args/av | R24 | "success"; exit=0 | cf58bab15a1025ce62fd01e8a8d11d6d6e78b884df8f24f51fe524b748c29acb | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0443 | D101:746 S21 | ace.m1.scalarq4.sc-cmp-matrix/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | c8a083f10857a8d524675670955efd363924ab5e3ba4d3409acfc477b4ca7414 | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0444 | D101:746 S21 | ace.m1.scalarq4.sc-cmp-matrix/av | R24 | "success"; exit=0 | c8a083f10857a8d524675670955efd363924ab5e3ba4d3409acfc477b4ca7414 | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0445 | D101:765 S21 | ace.m1.scalarq4.sc-count-digits/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | d1886b9b8c89982176cd3c2c13b97996caeeb57d86a5c10b2ebe1eee1da8a213 | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0446 | D101:765 S21 | ace.m1.scalarq4.sc-count-digits/av | R24 | "success"; exit=0 | d1886b9b8c89982176cd3c2c13b97996caeeb57d86a5c10b2ebe1eee1da8a213 | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0447 | D101:784 S21 | ace.m1.scalarq4.sc-crlf/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | 8c665d6602b3d28a4a16cef8dba32fe98b1275604eaebc715ee8c05a71a582cf | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0448 | D101:784 S21 | ace.m1.scalarq4.sc-crlf/av | R24 | "success"; exit=0 | 8c665d6602b3d28a4a16cef8dba32fe98b1275604eaebc715ee8c05a71a582cf | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0449 | D101:803 S21 | ace.m1.scalarq4.sc-for-loops/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | 73531c1db5fb130bc2e40bb7d84d25abe4845e43d6a01c1e121bb37814f09dfe | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0450 | D101:803 S21 | ace.m1.scalarq4.sc-for-loops/av | R24 | "success"; exit=0 | 73531c1db5fb130bc2e40bb7d84d25abe4845e43d6a01c1e121bb37814f09dfe | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0451 | D101:822 S21 | ace.m1.scalarq4.sc-gcd-euclid/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | 5f2e203f5551269f8a16ace32cc1d7614bcbb2808e562cfda397fc327bfe2cf3 | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0452 | D101:822 S21 | ace.m1.scalarq4.sc-gcd-euclid/av | R24 | "success"; exit=0 | 5f2e203f5551269f8a16ace32cc1d7614bcbb2808e562cfda397fc327bfe2cf3 | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0453 | D101:841 S21 | ace.m1.scalarq4.sc-is-digit/ac-native | R37 | "success"; exit=0; stderr#7df92993a715 | c6c3ab6076c34f29bfcead2e36921f3b5a55c31ce55e5687b9079faa908d87a2 | f096b0141fa704070da739dad586116b5206adce65367d792bf18e55675d97d2 | 扩展/其他现用 |
| M0454 | D101:841 S21 | ace.m1.scalarq4.sc-is-digit/av | R24 | "success"; exit=0 | c6c3ab6076c34f29bfcead2e36921f3b5a55c31ce55e5687b9079faa908d87a2 | 6bdef44e1184cbf8ae3cc6dd735b5752c9ac8ff5cc5cf6201a229260f2fd5091 | 扩展/其他现用 |
| M0455 | D101:860 S21 | ace.m1.scalarq4.sc-short-circuit-contexts/ac-native | R37 | "success"; exit=0; stderr#f006b9bb540b | e225157370442fe02670adafe849ea117b889cb38e12ce3360fcce897284ee55 | b3e9bffa532f94cf2a6fd931701d778c2096e6099dab01e2712e45fbf9fe1680 | 扩展/其他现用 |
| M0456 | D101:860 S21 | ace.m1.scalarq4.sc-short-circuit-contexts/av | R24 | "success"; exit=0 | e225157370442fe02670adafe849ea117b889cb38e12ce3360fcce897284ee55 | 37a379f0b290aa072449bd65d3b595940fadd41014a1bc3b08f9a3be7ccff2f4 | 扩展/其他现用 |
| M0457 | D101:879 S21 | ace.m1.scalarq4.sc-short-circuit-trace/ac-native | R37 | "success"; exit=0; stderr#b974b97d59ca | ff78ea933c247346f944bc92a923302e67be0121b91cabb4bcc3562f101bcf66 | 8cc7a7f8e6a777c7d05513f3a06c297584967d26fdc8c06d4195e195902a7062 | 扩展/其他现用 |
| M0458 | D101:879 S21 | ace.m1.scalarq4.sc-short-circuit-trace/av | R24 | "success"; exit=0 | ff78ea933c247346f944bc92a923302e67be0121b91cabb4bcc3562f101bcf66 | 37a379f0b290aa072449bd65d3b595940fadd41014a1bc3b08f9a3be7ccff2f4 | 扩展/其他现用 |
| M0459 | D101:898 S21 | ace.m1.scalarq4.sc-short-circuit-trap/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | d727ebe7cb23602d20392db4ca93886dc10ecd9d5bdb081007253b487dd40cb8 | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0460 | D101:898 S21 | ace.m1.scalarq4.sc-short-circuit-trap/av | R24 | {"stage":"run","code":"av.trap.div-by-zero"} | d727ebe7cb23602d20392db4ca93886dc10ecd9d5bdb081007253b487dd40cb8 | 54a344dd2245255024c3a8dd40dbabbfa676b167f10ea1b4787eb506086f753c | 扩展/其他现用 |
| M0461 | D101:917 S21 | ace.m1.scalarq4.ace-b01/ac-native | R37 | "success"; exit=14; stderr#7df92993a715 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | dc2653c798b46d530d4701ded836e523cfdceeb04da86f32b7e7dcb1c46893bb | 扩展/其他现用 |
| M0462 | D101:917 S21 | ace.m1.scalarq4.ace-b01/av | R24 | "success"; exit=14 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | fa88806312beebbaaae58feb420b0949e08b54fc123ee390a8df591cbb9abd23 | 扩展/其他现用 |
| M0463 | D101:936 S21 | ace.m1.scalarq4.ace-b02-signed-div-rem/ac-native | R37 | "success"; exit=-21; stderr#7df92993a715 | 9919d86da0f3937e2637b0bd68c40a6f20be60c1b1f54964888c5aa49a8eaccc | b9170639ac3fcba742ac3db0a05f56b20cedcc617aec03e1a6df37fb49b7beff | 扩展/其他现用 |
| M0464 | D101:936 S21 | ace.m1.scalarq4.ace-b02-signed-div-rem/av | R24 | "success"; exit=-21 | 9919d86da0f3937e2637b0bd68c40a6f20be60c1b1f54964888c5aa49a8eaccc | 6b9f5513a7b3707cfe9865d939d2029a89d9f921e3c214b6a884e107b318aa46 | 扩展/其他现用 |
| M0465 | D101:955 S21 | ace.m1.scalarq4.ace-b04-shortcircuit/ac-native | R37 | "success"; exit=42; stderr#7df92993a715 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | 78a9962c3d69eb04dca4ead5c033edfe0e1ccce887de1e02ec29ece9fdca7c1b | 扩展/其他现用 |
| M0466 | D101:955 S21 | ace.m1.scalarq4.ace-b04-shortcircuit/av | R24 | "success"; exit=42 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | a31900d8b0a32397dee22f50ef870cbe8c924facec1c7a8275bbd37025717c5f | 扩展/其他现用 |
| M0467 | D101:974 S21 | ace.m1.scalarq4.ace-b09-recursion/ac-native | R37 | "success"; exit=120; stderr#7df92993a715 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | e79d69bac3494ffb807e241bf111e53232e9a98b022dbeabb287b72b5a368f12 | 扩展/其他现用 |
| M0468 | D101:974 S21 | ace.m1.scalarq4.ace-b09-recursion/av | R24 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 49d7bc394db003a95ab37b079f075ffe64c5067d4054d957b0d084b4657fba7b | 扩展/其他现用 |
| M0469 | D101:993 S21 | ace.m1.scalarq4.ace-b13-overflow/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0470 | D101:993 S21 | ace.m1.scalarq4.ace-b13-overflow/av | R24 | {"stage":"run","code":"av.trap.int-overflow"} | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 21c769caf9ef3ac7ace95361b3fca998f96422b9ecacb4916d343e410be34610 | 扩展/其他现用 |
| M0471 | D101:1012 S21 | ace.m1.scalarq4.ace-b14-div-zero/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0472 | D101:1012 S21 | ace.m1.scalarq4.ace-b14-div-zero/av | R24 | {"stage":"run","code":"av.trap.div-by-zero"} | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | 54a344dd2245255024c3a8dd40dbabbfa676b167f10ea1b4787eb506086f753c | 扩展/其他现用 |
| M0473 | D101:1031 S21 | ace.m1.scalarq4.bd-min-literal/ac-native | R37 | "success"; exit=-2147483648; stderr#7df92993a715 | 3b5a953d47740d8d870a26c2e4089505f110496d0f33f7c8f672486b479923eb | e4106982d1c69390989ebce03391f3bf2f09a5314ca814e78462bf02d1976293 | 扩展/其他现用 |
| M0474 | D101:1031 S21 | ace.m1.scalarq4.bd-min-literal/av | R24 | "success"; exit=-2147483648 | 3b5a953d47740d8d870a26c2e4089505f110496d0f33f7c8f672486b479923eb | 9483f474c32dcffeb578630d3e8c894909edaba5ace7d047440a8926331b0caf | 扩展/其他现用 |
| M0475 | D101:1050 S21 | ace.m1.scalarq4.bd-min-div-minus-1/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | 8954b0c3b390f896ee8063a7c38417d584a8249b0727680ba176f4dc8dfcd233 | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0476 | D101:1050 S21 | ace.m1.scalarq4.bd-min-div-minus-1/av | R24 | {"stage":"run","code":"av.trap.div-overflow"} | 8954b0c3b390f896ee8063a7c38417d584a8249b0727680ba176f4dc8dfcd233 | 905ec3512157ddd5a695edf5f90f51d27b6597da04b646b15731b1dcc167afcc | 扩展/其他现用 |
| M0477 | D101:1069 S21 | ace.m1.scalarq4.bd-min-rem-minus-1/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | c4387433178d5a233d27bcfa52b99a2cb311c6f45c3641f3eac5b9514467859b | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0478 | D101:1069 S21 | ace.m1.scalarq4.bd-min-rem-minus-1/av | R24 | {"stage":"run","code":"av.trap.rem-overflow"} | c4387433178d5a233d27bcfa52b99a2cb311c6f45c3641f3eac5b9514467859b | e4122e02e58dc4af7720036275124725dba4ab490a0728b53ba97973f43c70d4 | 扩展/其他现用 |
| M0479 | D101:1088 S21 | ace.m1.scalarq4.bd-rem-by-zero/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | 2bf84a5b0dc7c7e2d4ccc72bb01b82afd9b2a03f45218127cf02d83052838adf | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0480 | D101:1088 S21 | ace.m1.scalarq4.bd-rem-by-zero/av | R24 | {"stage":"run","code":"av.trap.div-by-zero"} | 2bf84a5b0dc7c7e2d4ccc72bb01b82afd9b2a03f45218127cf02d83052838adf | 54a344dd2245255024c3a8dd40dbabbfa676b167f10ea1b4787eb506086f753c | 扩展/其他现用 |
| M0481 | D101:1107 S21 | ace.m1.scalarq4.bd-value70/ac-native | R37 | "success"; exit=70; stderr#7df92993a715 | ced05e6debb460b7a711a1723139ffdca36e9551ed107456b3467f5dc23407db | 840208619873cce4109274f7ae6c59bddbea5878f7e9609258f0761e96282d36 | 扩展/其他现用 |
| M0482 | D101:1107 S21 | ace.m1.scalarq4.bd-value70/av | R24 | "success"; exit=70 | ced05e6debb460b7a711a1723139ffdca36e9551ed107456b3467f5dc23407db | 81d5b34e952f7123f75fd0643ed3c5539cec698c4ea1b6baa84eaaf43737170b | 扩展/其他现用 |
| M0483 | D101:1126 S21 | ace.m1.scalarq4.bd-named-source-order/ac-native | R37 | "success"; exit=12; stderr#29c3f568e70a | de335c6dc7de0c6a83bd7315b27a176af9c949d91445ea8201070892b91f717f | f6661550e0877d74fbe41f3d55f280a2fe48785ebf6c31749e9393804b67aa56 | 扩展/其他现用 |
| M0484 | D101:1126 S21 | ace.m1.scalarq4.bd-named-source-order/av | R24 | "success"; exit=12 | de335c6dc7de0c6a83bd7315b27a176af9c949d91445ea8201070892b91f717f | 75204f7ed63ace1bfd7c83c611984a7613b17d7b01d06499a7142ae3980cfa56 | 扩展/其他现用 |
| M0485 | D101:1145 S21 | ace.m1.scalarq4.bd-or-skips-rhs/ac-native | R37 | "success"; exit=8; stderr#7df92993a715 | 296008ee6b22537a1280dcf1fdd59f6b05d1a280d13cbf57ec99b314ba985c38 | b911bb83d090c0b5f2af618bb0dd6aa5a5e25c66e77fbf94012326e143768373 | 扩展/其他现用 |
| M0486 | D101:1145 S21 | ace.m1.scalarq4.bd-or-skips-rhs/av | R24 | "success"; exit=8 | 296008ee6b22537a1280dcf1fdd59f6b05d1a280d13cbf57ec99b314ba985c38 | 085e664cdaef1417ff498664fb23223401b72bafb6bdc861dda50ec7fcc5d007 | 扩展/其他现用 |
| M0487 | D102:3 S22 | ace.m1.original-scalar.c01/ac-native | R37 | "success"; exit=42 | d0b8340d655dbabd00acb92b0c9540c7b173a3d0ecb11058050b674b573be2eb | 91973e74553982db689a937d610728edd3a5cdd1d14dbe14d209c610a829d17e | 扩展/其他现用 |
| M0488 | D102:3 S22 | ace.m1.original-scalar.c01/av | R24 | "success"; exit=42 | d0b8340d655dbabd00acb92b0c9540c7b173a3d0ecb11058050b674b573be2eb | a31900d8b0a32397dee22f50ef870cbe8c924facec1c7a8275bbd37025717c5f | 扩展/其他现用 |
| M0489 | D102:18 S22 | ace.m1.original-scalar.c02/ac-native | R37 | "success"; exit=70 | a7ee74538df550ad40fb570a0fef5975ad126a04f9c6132e4d57755a7214a647 | 5ac197e1baf14f974d235eeb99a0f434819df3997ed148f2df0675df10c801ca | 扩展/其他现用 |
| M0490 | D102:18 S22 | ace.m1.original-scalar.c02/av | R24 | {"stage":"run","code":"av.trap.div-overflow"} | a7ee74538df550ad40fb570a0fef5975ad126a04f9c6132e4d57755a7214a647 | 905ec3512157ddd5a695edf5f90f51d27b6597da04b646b15731b1dcc167afcc | 扩展/其他现用 |
| M0491 | D102:33 S22 | ace.m1.original-scalar.c03/ac-native | R37 | "success"; exit=61 | e9aa2f0821e031b4af32718e28f94329c0bcb037335fc8f1e497373ccfcdac2d | 5cf0a0366db2e5cd5132a233396c681ff5beeef34c975f3438e0100e0b32c1f6 | 扩展/其他现用 |
| M0492 | D102:33 S22 | ace.m1.original-scalar.c03/av | R24 | "success"; exit=61 | e9aa2f0821e031b4af32718e28f94329c0bcb037335fc8f1e497373ccfcdac2d | 1d90688d0b39bef7e9471c033f38e4c5272fe4e3585395a1524fa05f3b8aab91 | 扩展/其他现用 |
| M0493 | D102:48 S22 | ace.m1.original-scalar.named-01/ac-native | R37 | "success"; exit=23; stderr#d96dff4f88b8 | d2dfe05a18ca97dfa1736a9af7ae7a4220c7f0f137b2f5b610ed67f4cd791230 | 6e17bce11b349794da96dc57ec0dc3b5c0d9efc115a6b7ebc063d7eaf3098c17 | 扩展/其他现用 |
| M0494 | D102:48 S22 | ace.m1.original-scalar.named-01/av | R24 | "success"; exit=23; trace#d96dff4f88b8 | d2dfe05a18ca97dfa1736a9af7ae7a4220c7f0f137b2f5b610ed67f4cd791230 | 9a3ca937b433b520e034c6f19f1f33f1097cc84abe915df86e1596f7a4213332 | 扩展/其他现用 |
| M0495 | D103:5 S23 | ace.m1.textq4.v03_len_scalars_bytes/ac-native | R37 | "success"; exit=506 | 2f83a2067975db04f441fe3d4e5bafeaa603a4b3e0afa45f2accd6af4ac749bc | c9b1e106d8ca9617d20dc8f100648cf012f49f3026759b6c93b1c45f295de1ac | 扩展/其他现用 |
| M0496 | D103:5 S23 | ace.m1.textq4.v03_len_scalars_bytes/av | R29 | "success"; exit=506 | 2f83a2067975db04f441fe3d4e5bafeaa603a4b3e0afa45f2accd6af4ac749bc | 09befc9900a28f759e4738bcb0349c19370434d383ae2215c78efff7c8bd2293 | 扩展/其他现用 |
| M0497 | D103:24 S23 | ace.m1.textq4.v03_nul_data/ac-native | R37 | "success"; exit=33 | 2f1614fb94c783d15ff6d168a93bea4175fc16d9ae3ae97482873bead83989da | aadd8e84fd5cb6de26e2626c023f29717458a99809d401f91b11c923f4a833a3 | 扩展/其他现用 |
| M0498 | D103:24 S23 | ace.m1.textq4.v03_nul_data/av | R29 | "success"; exit=33 | 2f1614fb94c783d15ff6d168a93bea4175fc16d9ae3ae97482873bead83989da | de9f0560775a17c83f29f929e287765ec9b803b9aa9872c0b579f52cd0f68da7 | 扩展/其他现用 |
| M0499 | D103:43 S23 | ace.m1.textq4.v03_eq_full_bytes/ac-native | R37 | "success"; exit=1 | b8f06344378f2a5970024fe0c6ec09c5c0ea59f25d58bc4d50fdf0e14b12acce | 2ead393ef7a6d89f2ca92a05edcfaeda29f1fe83399587d966b2430beb23d074 | 扩展/其他现用 |
| M0500 | D103:43 S23 | ace.m1.textq4.v03_eq_full_bytes/av | R29 | "success"; exit=1 | b8f06344378f2a5970024fe0c6ec09c5c0ea59f25d58bc4d50fdf0e14b12acce | 0e130738ae603a1665a5f086483b2a36741aa96b2f026ab77153987cc1e94502 | 扩展/其他现用 |
| M0501 | D103:62 S23 | ace.m1.textq4.v03_char_at/ac-native | R37 | "success"; exit=233 | da0987d8b36a1dc7ccc0eed680566e57fe23ec90fb58fbdaa335575e4c36921d | 6daf2bbdfda94283bffd4fa513c32f6900c2c60a4e043190f574af00af7795ad | 扩展/其他现用 |
| M0502 | D103:62 S23 | ace.m1.textq4.v03_char_at/av | R29 | "success"; exit=233 | da0987d8b36a1dc7ccc0eed680566e57fe23ec90fb58fbdaa335575e4c36921d | 12becd6240e8070a01bff216f718b298c229dbeb524038dbc3e12b891883ee99 | 扩展/其他现用 |
| M0503 | D103:81 S23 | ace.m1.textq4.v03_char_at_negative/ac-native | R37 | "success"; exit=70; stderr#9bc5683d8052 | 22bad6ca714d78855bc7f1bc428c9fcf1f046e62e6cfb38180225cb332748278 | b7d20febcfd532fe8bbce2f10d65a816cd71d2af75c04451f52d3ebe50a402c1 | 扩展/其他现用 |
| M0504 | D103:81 S23 | ace.m1.textq4.v03_char_at_negative/av | R29 | {"stage":"run","code":"av.trap.str-index"} | 22bad6ca714d78855bc7f1bc428c9fcf1f046e62e6cfb38180225cb332748278 | cd9bedcc4ebb14a70eff9aa7c881f0f02fda4635b76f67aab1b7584a0e5738d0 | 扩展/其他现用 |
| M0505 | D103:100 S23 | ace.m1.textq4.v03_slice_oob/ac-native | R37 | "success"; exit=70; stderr#d8135c5c1d51 | 738655d8927856098e1cc2a4c24d1e53ab83c4e2ae7f238d38af4765389f967d | 6c63be88f9ff8e4ca198c62a9f2b419d41ddfcce75ed167538fec3c423a86024 | 扩展/其他现用 |
| M0506 | D103:100 S23 | ace.m1.textq4.v03_slice_oob/av | R29 | {"stage":"run","code":"av.trap.str-index"} | 738655d8927856098e1cc2a4c24d1e53ab83c4e2ae7f238d38af4765389f967d | cd9bedcc4ebb14a70eff9aa7c881f0f02fda4635b76f67aab1b7584a0e5738d0 | 扩展/其他现用 |
| M0507 | D103:119 S23 | ace.m1.textq4.v03_slice_scalars/ac-native | R37 | "success"; exit=3 | 5143d7e118920854e4720ea31b93d64140ad928e770312275e8c3d6de6b4feec | a831ccedfd1cd60df280a2b12e9b70162ea3e09e8f7bf67af949da0082ed26a9 | 扩展/其他现用 |
| M0508 | D103:119 S23 | ace.m1.textq4.v03_slice_scalars/av | R29 | "success"; exit=3 | 5143d7e118920854e4720ea31b93d64140ad928e770312275e8c3d6de6b4feec | 58bb850650ccb40c3e40ed4d15870d7070f0ae869914168c71859860534e34c0 | 扩展/其他现用 |
| M0509 | D103:138 S23 | ace.m1.textq4.v03_codepoint_ok/ac-native | R37 | "success"; exit=20013 | f4f46ab727ab2559fbdfaf06539df18ab0a515892e95395ec62a408d0222d57b | 34b744d45f0a32374f1b67ceaa8cf3baf64a5f30f95b30417c31a356891b5de3 | 扩展/其他现用 |
| M0510 | D103:138 S23 | ace.m1.textq4.v03_codepoint_ok/av | R29 | "success"; exit=20013 | f4f46ab727ab2559fbdfaf06539df18ab0a515892e95395ec62a408d0222d57b | 2d5b32f95269e1141b77c686474785620a0e057752dac56e756a2b6382dcdf39 | 扩展/其他现用 |
| M0511 | D103:157 S23 | ace.m1.textq4.v03_codepoint_surrogate/ac-native | R37 | "success"; exit=70; stderr#85460ca96ec8 | e8861447a97bdcb3a947ccd9bd2545054cfddf860656017471f047a923eb312e | 976ff567513fa89320e266df8a12e0c87d5e0fb182bf353ceb09a79a53a2a6b0 | 扩展/其他现用 |
| M0512 | D103:157 S23 | ace.m1.textq4.v03_codepoint_surrogate/av | R29 | {"stage":"run","code":"av.trap.char-invalid"} | e8861447a97bdcb3a947ccd9bd2545054cfddf860656017471f047a923eb312e | 037241abd45f50e95b5e4b329c8d5de76e96483edcfe2ce6672a02692791d505 | 扩展/其他现用 |
| M0513 | D103:176 S23 | ace.m1.textq4.v03_concat_clone/ac-native | R37 | "success"; exit=112 | 03f913e1743f7772a7f24e5e63195904d0aa139e7332799f049a4b5f94e83f6c | 277956a45f8b7e8a9b09b750302db1af26f34feb22528949d1800be8d7d43ab4 | 扩展/其他现用 |
| M0514 | D103:176 S23 | ace.m1.textq4.v03_concat_clone/av | R29 | "success"; exit=112 | 03f913e1743f7772a7f24e5e63195904d0aa139e7332799f049a4b5f94e83f6c | c2110673356bc9a4f2a1f85a6375767670e7571109ac483a0fbb6285e45676b9 | 扩展/其他现用 |
| M0515 | D104:24 S24 | ace.m1.res999.core-p04-while/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0516 | D104:24 S24 | ace.m1.res999.core-p04-while/av | R24 | "success"; exit=3 | 20535c1ae71b1fc8e5414682bbb38ce50fe5dad7242a596f57b53124779edb18 | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0517 | D104:45 S24 | ace.m1.res999.core-p05-while-zero/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 4f558c850d1bef74ff12adb01884b2ec8570eb3ed5edf344065fd61a3cf0fa19 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0518 | D104:45 S24 | ace.m1.res999.core-p05-while-zero/av | R24 | "success"; exit=1 | 4f558c850d1bef74ff12adb01884b2ec8570eb3ed5edf344065fd61a3cf0fa19 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0519 | D104:66 S24 | ace.m1.res999.core-p06-continue/ac-native | R37 | "success"; exit=3; stderr#7df92993a715 | bbd902d18bc49b96bf6a7e44fdf2fc4947143c99d888828e87c13d3d5493b94b | e7bde25eeaa529d7d65c1f0aee6a9c69d1970e0d031227deb3a27bf88a26f0c3 | 扩展/其他现用 |
| M0520 | D104:66 S24 | ace.m1.res999.core-p06-continue/av | R24 | "success"; exit=3 | bbd902d18bc49b96bf6a7e44fdf2fc4947143c99d888828e87c13d3d5493b94b | a65105e56d40b9b0af2c1389756379c7e592283c59b4abc014b51c6a4b0f46c1 | 扩展/其他现用 |
| M0521 | D104:89 S24 | ace.m1.res999.core-f11-add-chain-100/ac-native | R37 | "success"; exit=300; stderr#7df92993a715 | 1c3eae4c1c7400e89fc8e8d332b366b359e73e920e07b0cfa3f255bdeb960640 | 93c9e3fa23f5f41488c5f0eb3bc199373afb921909987ab3e2ff69a54a18906d | 扩展/其他现用 |
| M0522 | D104:89 S24 | ace.m1.res999.core-f11-add-chain-100/av | R24 | "success"; exit=300 | 1c3eae4c1c7400e89fc8e8d332b366b359e73e920e07b0cfa3f255bdeb960640 | d16c15d3309e460f0fdc7e578be98c9223014e8ef8868b47629ce513fa5254db | 扩展/其他现用 |
| M0523 | D104:111 S24 | ace.m1.res999.core-g02-parens-200/ac-native | R37 | "success"; exit=1; stderr#7df92993a715 | 77875b4f2b0790405788e04d9c55fd3e89fdecce35683d9e13cabcce65b60847 | 17729a6df1b617e4cc7698f359699a4d0f340b898d86b67594c189534f0289eb | 扩展/其他现用 |
| M0524 | D104:111 S24 | ace.m1.res999.core-g02-parens-200/av | R24 | "success"; exit=1 | 77875b4f2b0790405788e04d9c55fd3e89fdecce35683d9e13cabcce65b60847 | 6dc99bfb275bd2a0ce3fd0bbe9aa8deedfdea68ebe501d179e551de56b729837 | 扩展/其他现用 |
| M0525 | D104:138 S24 | ace.m1.res999.neg-syntax-before-while/av | R24 | {"stage":"parse","code":"source.syntax"} | 58a703d7a23f0813e125faa2db67f47689d4ba27abb2f4d717c0187fe18b9394 | 274f0881a5fd656c5cf8ced20023aeff3239a572af16ba77c1d7ddee18c6f0d3 | 扩展/其他现用 |
| M0526 | D104:154 S24 | ace.m1.res999.neg-over-ceiling-chain/av | R24 | {"stage":"verify","code":"mir.lower.depth"} | 9d8cc9f446789bd6f978602faea289afeacf8265a268b855ad554d6070675d04 | 3310fe5ea4e81acd7682f66a88e352bade6be6cbef491f01438b378ba26453e4 | 扩展/其他现用 |
| M0527 | D105:14 S25 | ace.m1.a2r917.r07.single/emit | R01 | "success"; artifacts=1 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | b2c65540ff7f1dffe551f49c708ee643360b950834789866af8e814b26069598 | 原511 |
| M0528 | D105:26 S25 | ace.m1.a2r917.r07.project/emit | R01 | "success"; artifacts=2 | 711272fe471930de9f52c6cf3139897af3919ad772a6b7f408285e04f379c957 | 7db00c004781cf11b2f2d768bb5619a27c7485327f6e68f01ab28dfaf65b19e6 | 原511 |
| M0529 | D105:42 S25 | ace.m1.a2r917.t08.single/emit | R01 | "success"; artifacts=1 | c6406d69b553bb29c61b0c9805f587f7a7656e46e349e17eeaa94ca66ca4ecf3 | f45b82f8a2cc2418d663ab278f51f870d97693e644b02736d0cd430155e1b59f | 原511 |
| M0530 | D105:54 S25 | ace.m1.a2r917.t08.project/emit | R01 | "success"; artifacts=2 | 1282afb9f3cba90f2515e56241e6f68e668be3fa1440e3e74c0302f5e33e61f9 | 889b9aafcff292d2206e968bb48612b85f58f8ac84040e4ff9f62daa15a9e5f1 | 原511 |
| M0531 | D105:70 S25 | ace.m1.a2r917.nb_now21/emit | R01 | "success"; artifacts=1 | bcdbfaea782cf417565a2b04170504f3d311708c2a4a8e093c9614de3bec2fcb | 255b75dfa1e99b71f660a83aeeff5fc2b38037380a545cdd3a38deeca702d243 | 原511 |
| M0532 | D105:82 S25 | ace.m1.a2r917.nb_caller21/emit | R01 | "success"; artifacts=1 | 5c156c8b152327daf8aeba3972fc9897cd86e643742c5799f83ae5057d14ad84 | 3dcec181957b4f8c33b86916eefc283691592c5a463e273a0e7743dba6ef540d | 原511 |
| M0533 | D105:94 S25 | ace.m1.a2r917.nb_view/emit | R01 | "success"; artifacts=1 | dc39416422922e95cc8b924ce6cc326c9ffc46c3116d5f4dde5f8b217024c352 | 83ef1993de0a692c9a15e79c85b2dbfe3588bdeeae7f064c56d6b8375d0b9016 | 原511 |
| M0534 | D105:106 S25 | ace.m1.a2r917.nb_move/emit | R01 | "success"; artifacts=1 | 9a6eba5afcca0dc20088f5416d7326ee1e51de330595fe039c7c0f8076faeb01 | 6a1b698ba3fc9266edaeacb8ee893bd2548c88476757b96fb42b9a120b80fc19 | 原511 |
| M0535 | D105:118 S25 | ace.m1.a2r917.nb_two_roots/emit | R01 | "success"; artifacts=1 | 7cd50bc07e6d51e4fd302f04ad94525024b3a11750d11944b2ba6c703ebbe476 | e4eb9eba1bd93e5f368d486a16880f0c5913c0e13474a4b8d6516670fee2ce1b | 原511 |
| M0536 | D105:130 S25 | ace.m1.a2r917.nb_nested/emit | R01 | "success"; artifacts=1 | ba71291e59cdc725a607593d98f773cb9efafb9d54e3808d8198ef7d922d764c | d512d30b83613698e5788820d0b764ab17540164397798f311c94724dd85fd21 | 原511 |
| M0537 | D105:142 S25 | ace.m1.a2r917.nb_slice_ok/emit | R01 | "success"; artifacts=1 | 6703714f07a2c861650f6b6c0b212fd3ea8fc521f9dad0c415495852a9a61e90 | 7698dcba8201677d706fb3b3d924f7e89465e4a1c55b3fd21b06e6ca364f966d | 原511 |
| M0538 | D106:7 S26 | a2r-owned-temporaries-931.derived-view/a2x-emit | R14 | "success"; artifacts=4 | c050625f5eef9ed1c42013f65942e7e66346b695b0a22de3784d14c379fb2cb3 | 27cb5388707fd2e710c8ba01013135d27108e9bb55271153f3851dcf735a15fd | 扩展/其他现用 |
| M0539 | D107:9 S26 | a2r-owned-temporaries-931.modes/a2x-emit | R14 | "success"; artifacts=4 | 53c383fe03671674b843d0790461a71d9c2bac147d1fe81fc247d5e58f15763d | cd70dfdcaa319fb7580b37570a8eba00cbf8792b6bb968530eb28016b54cb09c | 扩展/其他现用 |
| M0540 | D108:8 S26 | a2r-owned-temporaries-931.ownership/a2x-emit | R14 | "success"; artifacts=4 | 36e9d60816da4fe59ae0d42d50b4f474a3401acaa6c6d7ec66c47bfef1671237 | a40b3ac88def0c8848791e82727ed18a8f1a57c461e1df4aca34382e78d14f95 | 扩展/其他现用 |
| M0541 | D109:7 S26 | a2r-owned-temporaries-931.short-circuit/a2x-emit | R14 | "success"; artifacts=4 | 2f4610d6f8c24aa43805882d20ca1a0d08ded043bf7c874844f8da89e528dc97 | c5c857e7f560571782e039d5c93ce6d87d6a664876f6b96e253346279dad33e2 | 扩展/其他现用 |
| M0542 | D110:8 S26 | a2r-owned-temporaries-931.wrap/a2x-emit | R14 | "success"; artifacts=4 | 7979ba398453af1a5c4569665563e4b8a3bde6aac804ba95bbc829e30d491ed6 | ebd50f2f2baa81ea9ad85c430a7d5b069134dfb2e1b0290b09479cb59885fc26 | 扩展/其他现用 |
| M0543 | D111:3 S27 | ace.m1.a2r916.b16.single/emit | R01 | {"stage":"type","code":"auto_name_E0204"} | ae2430c6a6ce13ac34eae04473412e09ece883954eadb655cd75bc64d8604a06 | cc9e00367f842b9f98aee98a422082eb310ddad60e61d30b65a9e5f3dae2995f | 原511 |
| M0544 | D111:15 S27 | ace.m1.a2r916.b16.project/emit | R01 | {"stage":"type","code":"auto_name_E0204"} | fdf639c8bb4e19ccdbdbb63837a04729eedfc178e9b00d5a189b31a245ded6f3 | 5c19f6e26d40c2d89243a6a3734b1e58cfe7cfd96c2ddd5f26c1236b3e61aaa0 | 原511 |
| M0545 | D111:28 S27 | ace.m1.a2r916.b17.single/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | d0630cc078fd1f8a8b23a2e32f403e1f3523284729ce3a273050739e2f98dcc5 | 原511 |
| M0546 | D111:40 S27 | ace.m1.a2r916.b17.project/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | 48c5eea3367300cae37102b9351206d122467ee4d6372b96806460580df45908 | dc36c49cce7b6f8e25bfce9e6310bbf8914f5a4595888d3ea8ea15a27c590421 | 原511 |
| M0547 | D111:53 S27 | ace.m1.a2r916.b18.single/emit | R01 | {"stage":"type","code":"auto_name_E0203"} | 28f3cd22052664270d5884102c90ee8e87e39757c41f8ec7293ff9328d026cc0 | 019721c439533e19ccd3b37c160dea26d0c96c4c0d0dacbea2ea457c60837655 | 原511 |
| M0548 | D111:65 S27 | ace.m1.a2r916.b18.project/emit | R01 | {"stage":"type","code":"auto_name_E0203"} | c7c14be000e901adb46ffb70fbf27373ef30e44d3780837aadd5a3a235575c95 | 9714262f7051eb8940103abb675ab668622f92bb76d4a70c5ecc7c1f1fd6673a | 原511 |
| M0549 | D111:78 S27 | ace.m1.a2r916.b19.single/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | c0774f9a0a18b7a3254b37d0b326686cac07ebc872a97da6fcde8618c4debf39 | 09002b56b8c0d458b011f024b9164718b087a43426e840e2017e052ee51f21c1 | 原511 |
| M0550 | D111:90 S27 | ace.m1.a2r916.b19.project/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | 20083d717416fbc473b5da51b9b0f5876ef879077f72fcea906515ac1c442f16 | dc36c49cce7b6f8e25bfce9e6310bbf8914f5a4595888d3ea8ea15a27c590421 | 原511 |
| M0551 | D111:103 S27 | ace.m1.a2r916.t06.single/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | ea10e8f81483653e87517702b06693102f1705db235ca1261cf7f12a81222bb9 | 941c1a5b66f11b6827fcd5ab5d7b0ac9808984b99300beac128213924d9845e4 | 原511 |
| M0552 | D111:115 S27 | ace.m1.a2r916.t06.project/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | d0c1130d4fa4f4249b363d0b555250ba4e23364f383c3196347e7bb0b22000a9 | dc36c49cce7b6f8e25bfce9e6310bbf8914f5a4595888d3ea8ea15a27c590421 | 原511 |
| M0553 | D111:128 S27 | ace.m1.a2r916.r09.single/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | 5c615ee205db29e23ca75db12dd2ac067bdf2056ba558ed2acda13c50164ad8b | 987e8d2977ea663d6fe76163122b0b70de06a647760a311849e086c569c88ca9 | 原511 |
| M0554 | D111:140 S27 | ace.m1.a2r916.r09.project/emit | R01 | {"stage":"type","code":"auto_type_E0101"} | 873a910073a2302c66aae6441a16d740f7ba42801de092ae5a1c24d1b9edd936 | dc36c49cce7b6f8e25bfce9e6310bbf8914f5a4595888d3ea8ea15a27c590421 | 原511 |
| M0555 | D111:153 S27 | ace.m1.a2r916.r10.single/emit | R01 | {"stage":"type","code":"auto_type_E0102"} | de19cbac46fe4b4c8e21a3c8442efb5ad2746598f7435ac6818ff86fdf29c7fc | 4ada1727dee280b3737b9d256f8fa29512abfbca8ffb1f4b1aea435952fedb31 | 原511 |
| M0556 | D111:165 S27 | ace.m1.a2r916.r10.project/emit | R01 | {"stage":"type","code":"auto_type_E0102"} | 17b85e4081e6be1a497b56f97a047adc3200aa3add147a28a9aa38444b1cff7d | 2b05711e4f3344ae29f640864b670c6988ed545175ceb1eed9563cb8f80e570f | 原511 |
| M0557 | D111:178 S27 | ace.m1.a2r916.nb17/emit | R01 | "success"; artifacts=1 | e1f02665846dd554fba2c945c63202d6eaa7ffc6e0fc94d24aa5fbad5754b175 | 9b71182a33e73208764b5eda228a83442d28886ce9148d7efa0fa7ac2ccfc07e | 原511 |
| M0558 | D111:190 S27 | ace.m1.a2r916.nb18/emit | R01 | "success"; artifacts=1 | 084d0c470ba5bfd7698909a06db5c4762db80c0a19a593a755f0c5ed75fef5d1 | d1edcf138f74478e30fea18c07e181c9d99c59eff98a4cf309cfa15c2c84bb0c | 原511 |
| M0559 | D111:202 S27 | ace.m1.a2r916.nb19/emit | R01 | "success"; artifacts=1 | acea5ca5e801999b06e0c8e0d986af24535cd57d25d4660528b432f33225953b | 097c1244ea6d0b333820edb3169fe66b57e3808b2fcd4f6d6d392f22eb538f90 | 原511 |
| M0560 | D112:7 S28 | ace.m1.b16/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | ae2430c6a6ce13ac34eae04473412e09ece883954eadb655cd75bc64d8604a06 | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M0561 | D112:7 S28 | ace.m1.b16/autovm | R05 | {"stage":"type","code":"auto_link_E0401"} | ae2430c6a6ce13ac34eae04473412e09ece883954eadb655cd75bc64d8604a06 | 57abd414562e4e7a9ebd669b5e1c82d4dd1cf03b4bdd0c6bed06a4a219181911 | 原511 |
| M0562 | D112:7 S28 | ace.m1.b16/av | R24 | {"stage":"parse","code":"source.undefined"} | ae2430c6a6ce13ac34eae04473412e09ece883954eadb655cd75bc64d8604a06 | 89bcf5dbb3357050e62bc25fa9570e4e7da0e2069fcc414a838e1b621b2dd47a | 原511 |
| M0563 | D112:28 S28 | ace.m1.b17/ac-native | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | d60c06a072006718eeb66ce59c3ddaaf277aa7493141094499c55045c96c1727 | 原511 |
| M0564 | D112:28 S28 | ace.m1.b17/av | R24 | {"stage":"parse","code":"source.type-mismatch"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | b17cc20c638a0f0318f517f7793b35ddea9f957e0523601f0fc9c3eda5947854 | 原511 |
| M0565 | D112:47 S28 | ace.m1.b18/ac-native | R37 | {"stage":"typecheck","code":"verify.immutable-write"} | 28f3cd22052664270d5884102c90ee8e87e39757c41f8ec7293ff9328d026cc0 | eb426bcefb532b70cbd8e5f100a6a257adb3addc59df6df11af84d6f49f08ba9 | 原511 |
| M0566 | D112:47 S28 | ace.m1.b18/av | R24 | {"stage":"parse","code":"verify.immutable-write"} | 28f3cd22052664270d5884102c90ee8e87e39757c41f8ec7293ff9328d026cc0 | 1f95d3d097c1b48c69433e92c8647280bbbe49fda550a297fd54522cfe0823a4 | 原511 |
| M0567 | D112:66 S28 | ace.m1.b19/ac-native | R37 | {"stage":"typecheck","code":"verify.no-return-path"} | c0774f9a0a18b7a3254b37d0b326686cac07ebc872a97da6fcde8618c4debf39 | 6afdc38031d4587cdf756596182129425c45a3e2f8d4063eb8a7ea5378b4da6b | 原511 |
| M0568 | D112:66 S28 | ace.m1.b19/av | R24 | {"stage":"parse","code":"verify.no-return-path"} | c0774f9a0a18b7a3254b37d0b326686cac07ebc872a97da6fcde8618c4debf39 | 5c6be9ba04f0649df615ddfda866c75b5d624f435447a8854c0d7732a054bce6 | 原511 |
| M0569 | D113:8 S28 | ace.m1.b01/a2r-snapshot | R01 | "success"; artifacts=1 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 058bbbe22ff4725de4a2de25ad2fae20375241e963b455e0573ed064908f0a61 | 原511 |
| M0570 | D113:8 S28 | ace.m1.b01/ac-native | R37 | "success"; exit=14 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 178702efd90cd52bae247ac7a5400d979f906a2e0613d64c690a6b13630a8abc | 原511 |
| M0571 | D113:8 S28 | ace.m1.b01/autovm | R05 | "success"; result={"format":"legacy-display","text":"14"} | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 6c11d0cf899f64bebcd097f766a3d9130495759a7bb925bc33f9a55fca8b7aab | 原511 |
| M0572 | D113:8 S28 | ace.m1.b01/av | R24 | "success"; exit=14 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 65ac2524fc32565472977ac2a188c045383bb5bee814028cb9e7b278177db9c5 | 原511 |
| M0573 | D113:32 S28 | ace.m1.b02/a2r-snapshot | R01 | "success"; artifacts=1 | 9919d86da0f3937e2637b0bd68c40a6f20be60c1b1f54964888c5aa49a8eaccc | 7330bdc0ccbb78cafd6d997cd7406d57ea41890d3f0d1bb17fc295fff3c1b012 | 原511 |
| M0574 | D113:32 S28 | ace.m1.b02/ac-native | R37 | "success"; exit=-21 | 9919d86da0f3937e2637b0bd68c40a6f20be60c1b1f54964888c5aa49a8eaccc | 382aaa854669629ee8481e6b17cbe0d20207cc46f2b0b135036bdadecf4a530e | 原511 |
| M0575 | D113:32 S28 | ace.m1.b02/autovm | R05 | "success"; result={"format":"legacy-display","text":"-21"} | 9919d86da0f3937e2637b0bd68c40a6f20be60c1b1f54964888c5aa49a8eaccc | 280d44f3e544ad62053894fe1ff1266e1fbc247136850e4d1b12166046c04a46 | 原511 |
| M0576 | D113:32 S28 | ace.m1.b02/av | R24 | "success"; exit=-21 | 9919d86da0f3937e2637b0bd68c40a6f20be60c1b1f54964888c5aa49a8eaccc | 2e9b55bad4dfff7f3ab71a126414fc97cc0575a24c7ff2e495bc3dda07c8cb30 | 原511 |
| M0577 | D113:56 S28 | ace.m1.b03/a2r-snapshot | R01 | "success"; artifacts=1 | af16940b3c1ccb76033bdb6d119c8ec0bcd80bc73a55c01fbda649adaef5303a | 67b281c3805d183dfc295811cbbffd3e560ac286f11f9217495a3087a3b166cf | 原511 |
| M0578 | D113:56 S28 | ace.m1.b03/ac-native | R37 | "success"; exit=42 | af16940b3c1ccb76033bdb6d119c8ec0bcd80bc73a55c01fbda649adaef5303a | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0579 | D113:56 S28 | ace.m1.b03/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | af16940b3c1ccb76033bdb6d119c8ec0bcd80bc73a55c01fbda649adaef5303a | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0580 | D113:56 S28 | ace.m1.b03/av | R24 | "success"; exit=42 | af16940b3c1ccb76033bdb6d119c8ec0bcd80bc73a55c01fbda649adaef5303a | 96f7c7bf3b73335dc523a423cd08e0821a9fdaee6e52e0207f8fd83d971e1d64 | 原511 |
| M0581 | D113:80 S28 | ace.m1.b04/a2r-snapshot | R01 | "success"; artifacts=1 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | 07cb8d418a946b78fe8c3ca532b38b57569cb5830a972a3f4ab7dd0f1fc32d09 | 原511 |
| M0582 | D113:80 S28 | ace.m1.b04/ac-native | R37 | "success"; exit=42 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0583 | D113:80 S28 | ace.m1.b04/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0584 | D113:80 S28 | ace.m1.b04/av | R24 | "success"; exit=42 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | 96f7c7bf3b73335dc523a423cd08e0821a9fdaee6e52e0207f8fd83d971e1d64 | 原511 |
| M0585 | D113:104 S28 | ace.m1.b05/a2r-snapshot | R01 | "success"; artifacts=1 | f2d1644a66b5ca1b9097e43c782c73bdead1eb8a610a3928f6e67c2af2937c23 | 358f15041bb48463336c0194b34631694f15bd553d7c0d6fe4079e23b5406f35 | 原511 |
| M0586 | D113:104 S28 | ace.m1.b05/ac-native | R37 | "success"; exit=42 | f2d1644a66b5ca1b9097e43c782c73bdead1eb8a610a3928f6e67c2af2937c23 | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0587 | D113:104 S28 | ace.m1.b05/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | f2d1644a66b5ca1b9097e43c782c73bdead1eb8a610a3928f6e67c2af2937c23 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0588 | D113:104 S28 | ace.m1.b05/av | R24 | "success"; exit=42 | f2d1644a66b5ca1b9097e43c782c73bdead1eb8a610a3928f6e67c2af2937c23 | 96f7c7bf3b73335dc523a423cd08e0821a9fdaee6e52e0207f8fd83d971e1d64 | 原511 |
| M0589 | D113:128 S28 | ace.m1.b06/a2r-snapshot | R01 | "success"; artifacts=1 | 824e3c30780ad3b0b6e4c5e7d295fea7fe643362fa530a97cea3aee520c77c94 | 432e1b104b2ee966f2fc21e97f37bb05de2840abd45ea6bc9d4a92dbc3e19896 | 原511 |
| M0590 | D113:128 S28 | ace.m1.b06/ac-native | R37 | "success"; exit=0 | 824e3c30780ad3b0b6e4c5e7d295fea7fe643362fa530a97cea3aee520c77c94 | fa3b2f5ec7fc9be0812f7ca7d56af138529e6ca34f9630db9f99accd66bd5768 | 原511 |
| M0591 | D113:128 S28 | ace.m1.b06/autovm | R05 | "success"; result={"format":"legacy-display","text":"0"} | 824e3c30780ad3b0b6e4c5e7d295fea7fe643362fa530a97cea3aee520c77c94 | f83c147653bfd134d348bfc09d3ea2abf41bd00420febb856979bf7efc03398d | 原511 |
| M0592 | D113:128 S28 | ace.m1.b06/av | R24 | "success"; exit=0 | 824e3c30780ad3b0b6e4c5e7d295fea7fe643362fa530a97cea3aee520c77c94 | 0a8ddf242292e153c2acbc0008cccbb592e5dbf55371f8a240b16059d0809064 | 原511 |
| M0593 | D113:152 S28 | ace.m1.b07/a2r-snapshot | R01 | "success"; artifacts=1 | 017189855e365981414d9cd092ac6da6b2bd147ea1a3d71f5b56c61745e947a5 | d3eae85457060278e63e4239b078d30a93360ef85b86d8bf000d4a54cef910b3 | 原511 |
| M0594 | D113:152 S28 | ace.m1.b07/ac-native | R37 | "success"; exit=30 | 017189855e365981414d9cd092ac6da6b2bd147ea1a3d71f5b56c61745e947a5 | ce2e95ecafec0d83e0cfd86e7e1bfd8149bb8bad9b0878431182e924b7b556d5 | 原511 |
| M0595 | D113:152 S28 | ace.m1.b07/autovm | R05 | "success"; result={"format":"legacy-display","text":"30"} | 017189855e365981414d9cd092ac6da6b2bd147ea1a3d71f5b56c61745e947a5 | 0f1d14f5a2c5f33c0df2d6bb75b967d1dadb6195262049c00a8b6c85f8392bb2 | 原511 |
| M0596 | D113:152 S28 | ace.m1.b07/av | R24 | "success"; exit=30 | 017189855e365981414d9cd092ac6da6b2bd147ea1a3d71f5b56c61745e947a5 | 5173b348a757d0c20d84f692744610e3022255ad6aa8f40d920167976a29febf | 原511 |
| M0597 | D113:176 S28 | ace.m1.b09/a2r-snapshot | R01 | "success"; artifacts=1 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 7e4fb7fa4a3e135ba3c372a4db5ca35dc1af390b945b1c6591e9f8194d8d8663 | 原511 |
| M0598 | D113:176 S28 | ace.m1.b09/ac-native | R37 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | c4c4fb46652372ea31c33925e88fb756a31c2fac3c7238c69173e29b409ca161 | 原511 |
| M0599 | D113:176 S28 | ace.m1.b09/autovm | R05 | "success"; result={"format":"legacy-display","text":"120"} | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | cb669b098f24d910c1d83958dc8c0613262e2eb466c43d87231d5de537644b8c | 原511 |
| M0600 | D113:176 S28 | ace.m1.b09/av | R24 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 54f4c10a15f2d2b8b4f061ff85e5789859d4a82aa278afe8604daee564a61600 | 原511 |
| M0601 | D113:200 S28 | ace.m1.b10/a2r-snapshot | R01 | "success"; artifacts=1 | 86d73485d913c72d352c1fcfdb0696ed3b02cc41629f0decd94c2a15e5309a65 | 927d6918503283e26c2ac4ebea3d79eeaebe2679bdbd4198dda9466a1d5fa843 | 原511 |
| M0602 | D113:200 S28 | ace.m1.b10/ac-native | R37 | "success"; exit=42 | 86d73485d913c72d352c1fcfdb0696ed3b02cc41629f0decd94c2a15e5309a65 | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0603 | D113:200 S28 | ace.m1.b10/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 86d73485d913c72d352c1fcfdb0696ed3b02cc41629f0decd94c2a15e5309a65 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0604 | D113:200 S28 | ace.m1.b10/av | R24 | "success"; exit=42 | 86d73485d913c72d352c1fcfdb0696ed3b02cc41629f0decd94c2a15e5309a65 | 96f7c7bf3b73335dc523a423cd08e0821a9fdaee6e52e0207f8fd83d971e1d64 | 原511 |
| M0605 | D113:224 S28 | ace.m1.b12/a2r-snapshot | R01 | "success"; artifacts=1 | 1f0cec8300218b8b75ff9fd7259f7d7abe006ab9ccea020f51855f80c69c10b2 | fb523ac5a4e0222166c7d0a4c359158c34b2c1d0b980f9d796867291749838f6 | 原511 |
| M0606 | D113:224 S28 | ace.m1.b12/ac-native | R37 | "success"; exit=-2 | 1f0cec8300218b8b75ff9fd7259f7d7abe006ab9ccea020f51855f80c69c10b2 | e84fead509497096bfb86fa5aef2636f90d6f39535aa0d0468064e5fbdca176e | 原511 |
| M0607 | D113:224 S28 | ace.m1.b12/autovm | R05 | "success"; result={"format":"legacy-display","text":"-2"} | 1f0cec8300218b8b75ff9fd7259f7d7abe006ab9ccea020f51855f80c69c10b2 | a260f7f6dec30eb00edaae3457ce3d5e8d71e31e4313b88f2c02d68ffc604c62 | 原511 |
| M0608 | D113:224 S28 | ace.m1.b12/av | R24 | "success"; exit=-2 | 1f0cec8300218b8b75ff9fd7259f7d7abe006ab9ccea020f51855f80c69c10b2 | 09902099615b5d22808c1d8bd10f6bda5861959508f2e9abed8410f9125a1616 | 原511 |
| M0609 | D114:6 S28 | ace.m1.b13/ac-native | R37 | "success"; exit=70 | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | c0c2f064d8ac40de0a24d7bfa0f30f3cca69a9d8a348eba991ed6748070ec3e6 | 原511 |
| M0610 | D114:6 S28 | ace.m1.b13/av | R24 | {"stage":"run","code":"av.trap.int-overflow"} | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 0e5da218405a3be0f8e3704f76cfd721ca073a1fd3914f2dba68a4720196f0d4 | 原511 |
| M0611 | D114:25 S28 | ace.m1.b14/ac-native | R37 | "success"; exit=70 | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | c0c2f064d8ac40de0a24d7bfa0f30f3cca69a9d8a348eba991ed6748070ec3e6 | 原511 |
| M0612 | D114:25 S28 | ace.m1.b14/autovm | R05 | {"stage":"run","code":"DivisionByZero"} | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | 1c27e2a5782568a153a77333cd916b53abde2e6bf4e24105d94cca8fb4b517a2 | 原511 |
| M0613 | D114:25 S28 | ace.m1.b14/av | R24 | {"stage":"run","code":"av.trap.div-by-zero"} | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | 9804398fef534f538f79bf26c51970daf601d1c38a4559492c6a06d63d332915 | 原511 |
| M0614 | D114:46 S28 | ace.m1.b15/ac-native | R37 | "success"; exit=70 | 0c05c05f1f13f7096554b76989eec6c51989d3f2389cd3ecb4a3cf17aa8c5e2e | c0c2f064d8ac40de0a24d7bfa0f30f3cca69a9d8a348eba991ed6748070ec3e6 | 原511 |
| M0615 | D114:46 S28 | ace.m1.b15/av | R24 | {"stage":"run","code":"av.trap.rem-overflow"} | 0c05c05f1f13f7096554b76989eec6c51989d3f2389cd3ecb4a3cf17aa8c5e2e | ad536776f1a5fec5c14a8c5cb19720580f26fe8132ef66bc8bb5e87c3bfb0d9f | 原511 |
| M0616 | D114:65 S28 | ace.m1.b20/ac-native | R37 | "success"; exit=70 | b0a37a65580c6e7e7b973ce119dc1c2551a8a5addfe172c21b56ad65bb3708ac | c0c2f064d8ac40de0a24d7bfa0f30f3cca69a9d8a348eba991ed6748070ec3e6 | 原511 |
| M0617 | D114:65 S28 | ace.m1.b20/autovm | R05 | {"stage":"run","code":"DivisionByZero"} | b0a37a65580c6e7e7b973ce119dc1c2551a8a5addfe172c21b56ad65bb3708ac | 1c27e2a5782568a153a77333cd916b53abde2e6bf4e24105d94cca8fb4b517a2 | 原511 |
| M0618 | D114:65 S28 | ace.m1.b20/av | R24 | {"stage":"run","code":"av.trap.div-by-zero"} | b0a37a65580c6e7e7b973ce119dc1c2551a8a5addfe172c21b56ad65bb3708ac | 9804398fef534f538f79bf26c51970daf601d1c38a4559492c6a06d63d332915 | 原511 |
| M0619 | D115:8 S30 | ace.m1.f01-av912/av-f01-closures-fn-fresh | R16 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 4112be564aa18abd98f2a0c1cd8457db003604590746451dd702f29dce3ae66c | 原511 |
| M0620 | D115:8 S30 | ace.m1.f01-av912/av-f01-closures-fn-hit | R16 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 16c1897e0e1ddbfbfebbbb9cc869bb146760fbdefe0a4ce786560415a919c1aa | 原511 |
| M0621 | D115:8 S30 | ace.m1.f01-av912/av-f01-closures-fn-lambda-body-edit | R16 | "success"; exit=48 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 1ab7e35b300258170f1e04cf42270850ef72c720231c7512584dd85496a242df | 原511 |
| M0622 | D115:8 S30 | ace.m1.f01-av912/av-f01-closures-fn-restart | R16 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 2b0bf7f97ebd0dc9197cada793524cbb2c0c2f1101580cf11c32f4d41b92452b | 原511 |
| M0623 | D115:40 S30 | ace.m1.f02-av912/av-f02-closures-fn-fresh | R16 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | a73f7161808bad168abeb7a3dc3ab17de31477d9d87da9dac18b9a93d3a93897 | 原511 |
| M0624 | D115:40 S30 | ace.m1.f02-av912/av-f02-closures-fn-hit | R16 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 40fcde88a6117cc7cfc33caa413319734ab76ef285f17cc9c7ef616dbd1d7b50 | 原511 |
| M0625 | D115:40 S30 | ace.m1.f02-av912/av-f02-closures-fn-owner-body-edit | R16 | "success"; exit=61 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | b2719d29fc43dbe29b88d53bb3d51d0911372dd89e74a043a8e72473f9a388a9 | 原511 |
| M0626 | D115:69 S30 | ace.m1.k06fn-av912/av-k06-closures-fn-fresh | R16 | "success"; exit=42 | b93fd5be95ddabd36cadd047e3e4654a2f56c90ccf08f48361e996d150f1674e | e606fbd7e48f448cb0cb20c644f92061748adcd1522f49a2286643bf35c77cfe | 原511 |
| M0627 | D115:69 S30 | ace.m1.k06fn-av912/av-k06-closures-fn-restart | R16 | "success"; exit=42 | b93fd5be95ddabd36cadd047e3e4654a2f56c90ccf08f48361e996d150f1674e | 2c1f7008330086a8894c74aa3d08ad7ec879c5d8bf3b889042381c9557582740 | 原511 |
| M0628 | D115:88 S30 | ace.m1.k01fn-av912/av-k01-closures-fn-body-edit | R16 | "success"; exit=43 | 3cbd619464b23d68ac6e4223abf97112180cae662845c5251b6781ceb7cdeb1f | 2e99e43427d5e501f211524d5a38784ae88a5327522d1fc7c37dbbe69f6d0e60 | 原511 |
| M0629 | D115:108 S30 | ace.m1.s02fn-av912/av-s02-closures-fn-body-edit | R16 | "success"; exit=44 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | 26f2bd0182c30dbab2910d725dca461887ed1242a01e6116796aa16906906d4e | 原511 |
| M0630 | D115:108 S30 | ace.m1.s02fn-av912/av-s02-closures-fn-fresh | R16 | "success"; exit=42 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | 2a906e37561d85fc5492748766ad6736ab4255aa9406ff221eac42ccd6cae8a7 | 原511 |
| M0631 | D115:108 S30 | ace.m1.s02fn-av912/av-s02-closures-fn-restart | R16 | "success"; exit=42 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | 9f5ca4b297f74790e563312f38c78a0a0aeb3a07e3b3becc79e0c23c23172823 | 原511 |
| M0632 | D116:10 S31 | ace.m1.f01rcv-av913/av-f01-recovery-publish | R17 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 8c1f88e9038986d90ca459441d51cae7a79f2487a47622c501eb8b86de6a078f | 原511 |
| M0633 | D116:10 S31 | ace.m1.f01rcv-av913/av-f01-recovery-restore | R17 | "success"; exit=51 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 6c66fa2157bf68577bf5beb0446441c60e75c1389e1473c4ae3e3f9c22babafe | 原511 |
| M0634 | D116:10 S31 | ace.m1.f01rcv-av913/av-f01-recovery-restore-edit | R17 | "success"; exit=48 | c6f5de8e11d28528c420820646ee05df14f7f61833c6c9ca526a1817f70f2c1a | 00127b84fb0919f2fe31ebe792d4c19a69463e7b99b1c780ec730ed21a6aef50 | 原511 |
| M0635 | D116:36 S31 | ace.m1.k06rcv-av913/av-k06-recovery-publish | R17 | "success"; exit=42 | b93fd5be95ddabd36cadd047e3e4654a2f56c90ccf08f48361e996d150f1674e | 9de51f25da6c697403549d12af617d3e45451661a80299cd11c0e7da67cc5123 | 原511 |
| M0636 | D116:36 S31 | ace.m1.k06rcv-av913/av-k06-recovery-restore | R17 | "success"; exit=42 | b93fd5be95ddabd36cadd047e3e4654a2f56c90ccf08f48361e996d150f1674e | fa144d54820003a0142ad4c8cfad52ebea60d78b213c4f9ffbea68bc51ae6d6f | 原511 |
| M0637 | D117:5 S32 | ace.m1.k01-av865/av-k01-closures-source-fresh | R19 | "success"; exit=42 | 3cbd619464b23d68ac6e4223abf97112180cae662845c5251b6781ceb7cdeb1f | a7c5df0a9f8cc32205cfa62a462f741b5b2f61232ae0319d78b38f18d6a20009 | 原511 |
| M0638 | D117:5 S32 | ace.m1.k01-av865/av-k01-closures-source-restart | R19 | "success"; exit=42 | 3cbd619464b23d68ac6e4223abf97112180cae662845c5251b6781ceb7cdeb1f | 0df711829111507a35ed4bc1155562a236db79794c896a63f72d13b1cdb88ac3 | 原511 |
| M0639 | D117:24 S32 | ace.m1.k01-body-edit-av865/av-k01-closures-source-body-edit | R19 | "success"; exit=43 | 3cbd619464b23d68ac6e4223abf97112180cae662845c5251b6781ceb7cdeb1f | 101d670acfb146e02bfb7792a3d302f4afaa8cdb6e6eb1d24573283ba4855607 | 原511 |
| M0640 | D117:44 S32 | ace.m1.k02-av865/av-k02-closures-source | R19 | "success"; exit=42 | ae0e48715462947dbd3885e4e3e3fdea2b674cd8589bc7052971787b95ecdffd | e5c626448d969e079f936ec00060b918295387a8e8c2bc9c7e1811f65b1b8158 | 原511 |
| M0641 | D117:60 S32 | ace.m1.k03-av865/av-k03-closures-source | R19 | "success"; exit=42 | e5999647a610cb928a8cfef3d93e726b0b4ad4bb6aaafba4303201b4b9ceb8b9 | 74d508642e48681c77b4f2c3300923242b0aaa6bb62b88f6b22c1e6536dd58f0 | 原511 |
| M0642 | D117:76 S32 | ace.m1.k04-av865/av-k04-closures-source | R19 | {"stage":"resolve","code":"source.closure-borrow-escape"} | 848819225d0a65669e3ad5c1e29f723b88c1c7c756fe930fedf09079b05eac2e | b21c65803cc0614a9e63e4aaf735b56257a385777ca98f0308083397624f8f8a | 原511 |
| M0643 | D117:92 S32 | ace.m1.k05-av865/av-k05-closures-source | R19 | "success"; exit=42 | 5ed0981cd258e0a1c7c87c713d712180cd1fa5cac6808035756942fa303d3b6f | 89498d2324781b2c12ad1be25dac9691542ed5ba701465cf85072dfac5a26fc2 | 原511 |
| M0644 | D117:108 S32 | ace.m1.k06-av865/av-k06-closures-source | R19 | "success"; exit=42 | b93fd5be95ddabd36cadd047e3e4654a2f56c90ccf08f48361e996d150f1674e | 9815c3a0103938375ee666507777a58845daa84dd67ab9d8761dd1b96b9963ec | 原511 |
| M0645 | D118:31 S33,S44 | ace.m1.l01-av864/av-l01-collections-source | R21 | "success"; exit=42 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | c65b3d98490e158a54eff2cd1f5c15396fe9822875f965dee0d601a6873545f2 | 原511 |
| M0646 | D118:47 S33,S44 | ace.m1.l02-av864/av-l02-collections-source | R21 | "success"; exit=42 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | 9e446923ecb3663b8475e3eb5d7da10521cf7af2eab56fb8cf1bf93d027cd285 | 原511 |
| M0647 | D118:63 S33,S44 | ace.m1.l03-av864/av-l03-collections-source | R21 | "success"; exit=0 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | bbcf3c4964bc39af72aa341e0ad59db7a89f69ba098d32fb639d62d3bf6001a5 | 原511 |
| M0648 | D118:79 S33,S44 | ace.m1.l04-av864/av-l04-collections-source | R21 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 0a56f6fd4b095a978b26f3ea6ee5d33f5848bfd248d90376fa6d4cf4932ee411 | 原511 |
| M0649 | D118:95 S33,S44 | ace.m1.l05-av864/av-l05-collections-source | R21 | "success"; exit=42 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | cea0b0210f89133424dd0283cc1bb0d6775b5cc36e3bb6f39bc5ef359634a315 | 原511 |
| M0650 | D118:111 S33,S44 | ace.m1.l06-av864/av-l06-collections-source | R21 | {"stage":"run","code":"av.trap.list-index"} | de3a85627019b586ef8b9ccb377e60ef57a0686f15076d57e93910f366b5a884 | 98b99273546e586a80d79b9ef37ed345b1365fcd1be8cd6925cc85ded146e041 | 原511 |
| M0651 | D118:127 S33,S44 | ace.m1.l07-av864/av-l07-collections-source | R21 | "success"; exit=42 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | 9eff49baf4968a2b7e6482c965101d1b116727f412d070690f09b86c3f49b809 | 原511 |
| M0652 | D118:143 S33,S44 | ace.m1.v01-av864/av-v01-collections-source | R21 | "success"; exit=42 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | e6bee14157c48ffaf2e1e46a418f84eedc22beb92db237965485cb74f0e7018d | 原511 |
| M0653 | D118:159 S33,S44 | ace.m1.v02-av864/av-v02-collections-source | R21 | "success"; exit=42 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | 46cef3786286159d28e675074171174b3b409cf2ca593cf353e05ee413c79d69 | 原511 |
| M0654 | D118:175 S33,S44 | ace.m1.v03-av864/av-v03-collections-source | R21 | "success"; exit=3 | 41767e7e7046efc75d06d790e6392a4da4260baa6b95e8e244964f1883383f97 | b84bff0662158242766a4d606a3761924b49fccc67f1481c3af4f7eb2a72bd0d | 原511 |
| M0655 | D118:191 S33,S44 | ace.m1.v04-av864/av-v04-collections-source | R21 | "success"; exit=42 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | e5ad0f7e06af1d091871027bb29985ecc73e119cb7768bcb6d113fdf0631a0aa | 原511 |
| M0656 | D118:207 S33,S44 | ace.m1.v05-av864/av-v05-collections-source | R21 | "success"; exit=42 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | e4d10192da97815907ed6797fc7a3e84a7d5ff1449d7409835a79189f398161c | 原511 |
| M0657 | D118:223 S33,S44 | ace.m1.v06-av864/av-v06-collections-source | R21 | "success"; exit=42 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | 4b01a9b318854f7c96b9668ad19c213eb59490252061140d3d8687481e98a7c2 | 原511 |
| M0658 | D118:239 S33,S44 | ace.m1.v07-av864/av-v07-collections-source | R21 | "success"; exit=42 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | 477b1557b6126c2fac27c7785d14013e9056e805c0906fdcb81556b5cbd95317 | 原511 |
| M0659 | D118:255 S33,S44 | ace.m1.n01-av864/av-n01-collections-source | R21 | "success"; exit=51 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | b198c79eb064ffe0351bb714b3d8e812e29fd4594481aea185dd3fd3854cf60d | 原511 |
| M0660 | D118:271 S33,S44 | ace.m1.n02-av864/av-n02-collections-source | R21 | "success"; exit=42 | cb7d63e1356a05909eaf9a32b069192ce0fa5590da6c34f07b872475ef8474f4 | 7ea50bdd036a0d0c861d0b121e6211f7cb0f3a3faabc0da7158d97031dcbcd4f | 原511 |
| M0661 | D118:287 S33,S44 | ace.m1.n03-av864/av-n03-collections-source | R21 | "success"; exit=53 | 6dc3d3af1778bb4f228ed354202756fc65a9aec8fbc1bce06c015e33f5b2be97 | 7859f337a1713ad5b1a2370309a7e377a80d9f008f037e0428d94a41fb9ecfbf | 原511 |
| M0662 | D119:4 S34 | ace.m1.o01/av-console | R23 | "success"; exit=7; stdout#d37578f6441a; stderr#7df92993a715 | f5d29f54d58d4af33dfb9e3f7a6315b8c30dd7913f79a378e6db0096f546cd44 | a0dd8ab85c3dcc96eca817f20c5ac908aea66fb16fbbaf4d73bee4fb34f5f6cd | 原511 |
| M0663 | D119:14 S34 | ace.m1.o02/av-console | R23 | "success"; exit=0; stdout#fde4b71e34bd; stderr#fde4b71e34bd | 6cb87ba732387d45578c6a474e262cb83a0c7d1101701ebe1f8b6874aaa7bdab | 57392bc69204da71db7faffd12390b8e71f1014b679e1746af75340efc48db1b | 原511 |
| M0664 | D119:24 S34 | ace.m1.o03/av-console | R23 | "success"; exit=0; stdout#e4cd2381da75; stderr#1c8eac6b6088 | ff647408458ed7ba66ebff1ffca415b909dbb8a720286f14f91d244f31e9f008 | 43e7c70e3ed8954f9cd26b97a794610a1c0362b8489471d2685464edc193973f | 原511 |
| M0665 | D119:34 S34 | ace.m1.o04/av-console | R23 | "success"; exit=0; stdout#748357fcb9c5; stderr#6e332d643da1 | 637ecec5894877329a1b6323d91b1d17d0e4d42858f37048c540668894f41a15 | a22c51f96f975c85b92c62618bbc0daffbdf30402e5cfda09d4c85ce3e4502ca | 原511 |
| M0666 | D119:44 S34 | ace.m1.o05/av-console | R23 | "success"; exit=4; stdout#0128b4e2ca6d; stderr#0128b4e2ca6d | 70a6f251701128e47dc43d427636c8d0212e3f5c66e2becfe1637e25f6214742 | f27594c29bd31b673f9e786cce22d0811cb0e47576052a5559a79dc4febb65bb | 原511 |
| M0667 | D119:54 S34 | ace.m1.o06/av-console | R23 | "success"; exit=0; stdout#5041afafef9b; stderr#482d2d35c1b5 | 76bba2ef496eaeb8b1b6694d254625e5a22d0ac3563a2a5b24da59dfb8e09314 | 812e2a7ac843cfb855bf505bc19d223957794f0dfd09733a7da73c221df44f6c | 原511 |
| M0668 | D119:64 S34 | ace.m1.o07/av-console | R23 | "success"; av_channel="unit"; stdout#8f3bc2541467; stderr#1b02263251ae | 0261fe094ca6b06b2d9841f90101f8c7e43c290cd6f1727bf79240c90a9cca07 | 028752b0fb8139a803a279f6847ce277ece8fed076c26301524dae5bfaaddcb0 | 原511 |
| M0669 | D119:74 S34 | ace.m1.o08/av-console | R23 | "success"; exit=42; av_channel="value"; stdout#5b51da32bed1; stderr#0e8a159b7128 | e30bf9d45bcd8798d6e2600bbef45162df067940bfccf88449dc5257388e1e8f | 011168c9df300bdb082e3a37abca9a1a273ac9083cc2945b094694b420cad866 | 原511 |
| M0670 | D119:84 S34 | ace.m1.o09/av-console | R23 | "success"; av_channel="unit"; stdout#7df92993a715; stderr#e17408451ad8 | da5010485cc2443f76cd4c8ed1ba165617bca8b00b5afa53ef7d0f1635726508 | 8f312abb9ad15440dead8859b68ab72bf81f4b296a625be379649f303da85044 | 原511 |
| M0671 | D119:94 S34 | ace.m1.o10/av-console | R23 | {"stage":"run","code":"av.trap.div-by-zero"}; stdout#9217406b6a8d; stderr#767af3aa45f5 | 43caa9c56d2f007448e352ce400bc226c8298b596402ef6551da2ccc61f45b47 | 96776cbfcb298998e394290e9feee1cc6850521e52da8bae7b48e63d9d148583 | 原511 |
| M0672 | D119:104 S34 | ace.m1.o11/av-console | R23 | {"stage":"resolve","code":"source.console-argument"}; stdout#7df92993a715; stderr#7df92993a715 | e3cbc554e52a114453be1f6a84c1b503a7c7beb71134ad04daceaced3d18722c | e64a17e17af740966ef45fb2a27a4c3d8e1415ab9088622845ad4ec837f1164d | 原511 |
| M0673 | D119:114 S34 | ace.m1.o12/av-console | R23 | {"stage":"parse","code":"host.unauthorized"}; stdout#7df92993a715; stderr#7df92993a715 | 48b3e5f2858cf0b230dda0afff87255f4afbf6b3b87bee2c29deb1108f2f7d71 | db9305a78fed1f7f932e7c015f554e576813dce5224087ab0ad9086ccfde4215 | 原511 |
| M0674 | D119:124 S34 | ace.m1.o13/av-console | R23 | {"stage":"resolve","code":"source.console-argument"}; stdout#7df92993a715; stderr#7df92993a715 | 4981d1eb161f47dec26416d7f17404bc9b31a29b43b8d05250e7d9888c436f70 | e64a17e17af740966ef45fb2a27a4c3d8e1415ab9088622845ad4ec837f1164d | 原511 |
| M0675 | D119:134 S34 | ace.m1.o14/av-console | R23 | {"stage":"parse","code":"source.console-reserved"}; stdout#7df92993a715; stderr#7df92993a715 | 6c9bf770dd5e67234f9fd7ef56e5a9d99583a039a61ac586db01f40f5b20eb9f | 22733a4f952a2baebc88a9e9335ffa0b7c795aff4d26887db233dbcb8f9858dc | 原511 |
| M0676 | D119:144 S34 | ace.m1.o15/av-console | R23 | "success"; av_channel="unit"; stdout#7fc4d6ff5d26; stderr#7fc4d6ff5d26 | fee9e33db6de2b7f093f000333ecb7c980f8903d8513250db4ae018a3a78d3a1 | faebe0037a59b4d0f9e5e71e7578954ed501cd7e8f9b56860e77f6a55668ae74 | 原511 |
| M0677 | D120:25 S35 | ace.m1.u01/av | R24 | "success"; exit=42; av_channel="value" | 88ba885c22b9fc4b9198a310db2b04d8ac22465764f6346ed997303ed6e41907 | a158ce2987c6305c46c7af0003869abda4983151e5b17bb343f98e25710e11f4 | 原511 |
| M0678 | D120:25 S35 | ace.m1.u01/av-scalar-control | R24 | "success"; exit=42 | 88ba885c22b9fc4b9198a310db2b04d8ac22465764f6346ed997303ed6e41907 | 77f5bf8b9837c900425fa809afa44418cf67ac87e410abb19d1f703e4d9a13c8 | 原511 |
| M0679 | D120:44 S35 | ace.m1.u02/av | R24 | "success"; av_channel="unit" | d151ed5a1ea0aaae2a037e83222013c3dd8009e9b891fda4c42da0ed8f37c4e8 | 4f3db28fe2c910ee69884e4525446a5a7c612efaf86d0d04f684a16d342b72b0 | 原511 |
| M0680 | D120:44 S35 | ace.m1.u02/av-scalar-rejects | R24 | {"stage":"parse","code":"source.unsupported"} | d151ed5a1ea0aaae2a037e83222013c3dd8009e9b891fda4c42da0ed8f37c4e8 | 05fc320f11dadc879f0134f7a2de0542938d3424f622ed470ee25d5532bab7e5 | 原511 |
| M0681 | D120:63 S35 | ace.m1.u03/av | R24 | "success"; exit=42; av_channel="value" | 303ea59fe73998917387a391574ab7901878bb7834ad963eb4081b936a530692 | a158ce2987c6305c46c7af0003869abda4983151e5b17bb343f98e25710e11f4 | 原511 |
| M0682 | D120:79 S35 | ace.m1.u04/av | R24 | "success"; av_channel="unit" | d0a321b722dd497bb8d33d0ecd731f67fa7e6f27eb78c78300c5eb49a2d9b529 | 4f3db28fe2c910ee69884e4525446a5a7c612efaf86d0d04f684a16d342b72b0 | 原511 |
| M0683 | D120:95 S35 | ace.m1.u05/av | R24 | {"stage":"run","code":"av.trap.div-by-zero"} | a51236296bbe096cfe9d742b49f88f926246d9644236769ec0e63b0c6fc5f1a2 | 949968bbd97556453866f1645dc4a7b44116c7aba7da9e64a0b44bd6a9f56fef | 原511 |
| M0684 | D120:111 S35 | ace.m1.u06/av | R24 | {"stage":"parse","code":"source.entry-missing"} | 16833ae6224a48c96882734f369a2f8746f7536b8a0b77c4c3213cb01e78d312 | ac4d679766c574f0283d00e6e16b69a9b55a662b861ae136ecbd498c27e76d65 | 原511 |
| M0685 | D120:127 S35 | ace.m1.u07/av | R24 | {"stage":"parse","code":"source.entry-params"} | e2e07dc8f08e955aa3aa85db5be8bf799547cbfaa145e26de0abdfcc02de1f3e | c8b38f4c03f8089a1fb6cadc743e45e010badcc7cc16888c27eb4227a8e69232 | 原511 |
| M0686 | D120:143 S35 | ace.m1.u08/av | R24 | {"stage":"parse","code":"source.unit-return-value"} | 6b97b3941d4b05b0617d7d89cbd1e88d568de6b21a45d94039180ce5308fc594 | c8bbeed4493b3772912f9ee3a081eeb32f34c8dc812543f20b97fe744e867031 | 原511 |
| M0687 | D120:159 S35 | ace.m1.u09/av | R24 | {"stage":"parse","code":"source.entry-tail"} | 1dd954ee1b461cebbd1a91ce17505f2c809bee09babb626ff1f0b9b0472427b2 | f963dfecd3b3410b624a74862cb4a31f9461a5a7202dc82cc62febc0125b5009 | 原511 |
| M0688 | D120:175 S35 | ace.m1.u10/av | R24 | {"stage":"parse","code":"source.entry-duplicate"} | bdbfdb53c4364f05e0468a5e135e78841c3f4f4a40f9291a91eb7fb5ea4e7775 | e7ce1af3b4e682364810c328ba6543007383f0247133568597b4e1a22d850986 | 原511 |
| M0689 | D121:29 S36 | ace.m1.ia920.b18/autovm | R05 | {"stage":"type","code":"auto_name_E0203"} | 28f3cd22052664270d5884102c90ee8e87e39757c41f8ec7293ff9328d026cc0 | d74f3c899f236d2cc36d11c1790ce19ca8850dfb0b0f7cc45ae61dfa520f531e | 原511 |
| M0690 | D121:44 S36 | ace.m1.ia920.b18c/autovm | R05 | {"stage":"type","code":"auto_name_E0203"} | a8b4d3bf0a7e39ca66782fab4b64991af21c25bd075da26f4ab02fd813745e92 | d74f3c899f236d2cc36d11c1790ce19ca8850dfb0b0f7cc45ae61dfa520f531e | 原511 |
| M0691 | D121:59 S36 | ace.m1.ia920.top/autovm | R05 | {"stage":"type","code":"auto_name_E0203"} | 57782f8a2ac858c88e9c9ef3b68759328e3807f7d05703e384b03ad7ae11ec08 | d74f3c899f236d2cc36d11c1790ce19ca8850dfb0b0f7cc45ae61dfa520f531e | 原511 |
| M0692 | D121:74 S36 | ace.m1.ia920.nb18/autovm | R05 | "success"; result={"format":"legacy-display","text":"2"} | 084d0c470ba5bfd7698909a06db5c4762db80c0a19a593a755f0c5ed75fef5d1 | e60db4b7e5433ec092bcf78db646a485ad9110e53919ef9fa8f0c42baf7530ca | 原511 |
| M0693 | D121:89 S36 | ace.m1.ia920.nb18s/autovm | R05 | "success"; result={"format":"legacy-display","text":"6"} | 54534ad3676e7c960936524a2d99952e243cb203a32262c7b536d29f237a92a0 | a02cac218cc49e2ee4d40d1752a7931c0afb95c032ceaa1eec93652b78232be5 | 原511 |
| M0694 | D122:10 S37 | ace.m1.b08/a2r-snapshot | R01 | "success"; artifacts=1 | fbf777f2b3174edea44f8d65057fc1f178a93e269f241d61dd0d1d0579ca8145 | 47373a83d194a35c6f5530492e9905c1783df10dadb7719e8c1e2ddad4fe9a44 | 扩展/其他现用 |
| M0695 | D122:26 S37 | ace.m1.b11/a2r-snapshot | R01 | "success"; artifacts=1 | cb0094b90d2db6537d059b910fc99dbcd3daf1e20e2c3c1cca7f1849e82b6dd2 | 6f26e4b8696c2247d5970c8e8985c68fba6ac7e0b71fafdfed1d7eb0eb65fa53 | 扩展/其他现用 |
| M0696 | D122:42 S37 | ace.m1.r01/a2r-snapshot | R01 | "success"; artifacts=1 | 9126803ce99c46d12066c468b56189704386f5c77acbcf8a564acb76d8e3fdc6 | 5213db2075dccc2e433100d4687933da0bda4ef34a19c7add3487ba8a260429f | 扩展/其他现用 |
| M0697 | D122:58 S37 | ace.m1.r03/a2r-snapshot | R01 | "success"; artifacts=1 | 10630e3d095fcd27adc5471137ca6b28e4d36a45ce63d53f7262bf6d8ed5296b | ae0418b5706bb33706499f993e202f8a062d26b70fdfc8d08fbf22f90e7633ab | 扩展/其他现用 |
| M0698 | D122:74 S37 | ace.m1.r04/a2r-snapshot | R01 | "success"; artifacts=1 | 1382b4c3a35d443e5ebf21ece67d4a6c9b017e443eec9346b5e0ece12d3b7b59 | b8b45b77bcea5585485c4297a82dd2b357d07aff38672f8bd278c800c5d1485d | 扩展/其他现用 |
| M0699 | D123:15 S38 | ace.m1.a2rt.l01/a2r-snapshot | R01 | "success"; artifacts=1 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | 8309c754dc277165d18561b2d23eb78a01d6fdd3098f0cf687418a233277326e | 扩展/其他现用 |
| M0700 | D123:31 S38 | ace.m1.a2rt.l02/a2r-snapshot | R01 | "success"; artifacts=1 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | 31c8d4103342bbf16eadb6f71217f2572d490f4d4e84cee69971f061df60cb04 | 扩展/其他现用 |
| M0701 | D123:47 S38 | ace.m1.a2rt.l03/a2r-snapshot | R01 | "success"; artifacts=1 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | 141e9eddb2414b5368676ab01bb17a21fdc9d23ce02f7fd2956e370353da3ee9 | 扩展/其他现用 |
| M0702 | D123:63 S38 | ace.m1.a2rt.l04/a2r-snapshot | R01 | "success"; artifacts=1 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | e9ea8c71dddc86203d3ae15e8894ae8ae2c3faf4161b308e960e25749a3b7a1e | 扩展/其他现用 |
| M0703 | D123:79 S38 | ace.m1.a2rt.l05/a2r-snapshot | R01 | "success"; artifacts=1 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 82d02957cc4c3ba04723f3b6832742c9cc16a8b4d0dee0c9952c2c08999b1e6b | 扩展/其他现用 |
| M0704 | D123:95 S38 | ace.m1.a2rt.l07/a2r-snapshot | R01 | "success"; artifacts=1 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | 910550673eaedba3b57d00489943184f6d61405242e287f1070036c2caa350eb | 扩展/其他现用 |
| M0705 | D124:14 S39 | ace.m1.a2rscene.r01/a2r-scene-emit | R01 | "success"; artifacts=1 | cc367c3514064f0e9141ae7a56b08934c77ac8892bbcb5bd6cde4b29da0e5bcb | 45c2551f59e5723ed771ffe9aed53b59cd78054b17fde021486b157203541b03 | 扩展/其他现用 |
| M0706 | D124:30 S39 | ace.m1.a2rscene.r02_l04/a2r-scene-emit | R01 | "success"; artifacts=1 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | e892be39e6b2fa4f2f037dca8153239d832ecb4fa8673909f7ce3b1913e9353a | 扩展/其他现用 |
| M0707 | D124:46 S39 | ace.m1.a2rscene.r02_l05/a2r-scene-emit | R01 | "success"; artifacts=1 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | b396d199cd544f5592814cb10c60010434070778820d3fac2648504f4a52d89d | 扩展/其他现用 |
| M0708 | D124:62 S39 | ace.m1.a2rscene.r02_v05/a2r-scene-emit | R01 | "success"; artifacts=1 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 20e06e6fa329c65f7ded760ff02c69ef85a9ae2dbf20bc112fa38080c317719e | 扩展/其他现用 |
| M0709 | D124:78 S39 | ace.m1.a2rscene.r02_lstr/a2r-scene-emit | R01 | "success"; artifacts=1 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 254e443b53ebc8ce3519858118401a6e9228d7bfc96fd93c8a78c851e084718d | 扩展/其他现用 |
| M0710 | D124:94 S39 | ace.m1.a2rscene.r02_k02/a2r-scene-emit | R01 | "success"; artifacts=1 | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | 3998de02e52b9333863f08026383db7b03d55dee2bc610819bdfe4acab3f1c4a | 扩展/其他现用 |
| M0711 | D124:110 S39 | ace.m1.a2rscene.r03_legal/a2r-scene-emit | R01 | "success"; artifacts=1 | 8a6df4cbfcd688a0c8b798b690df476004815afc3b6e3309f190039cb6f797ba | 24e4be3111cb03b7fa2c8e0ca46bff633a4168018550366effc6c7a1b30f02bf | 扩展/其他现用 |
| M0712 | D124:126 S39 | ace.m1.a2rscene.r03_static/a2r-scene-emit | R01 | "success"; artifacts=1 | 72ddd2fedf331feb54b5d76c499fcc6aa0156d8725c84ca2fafe57f658b28a44 | 9ce3cb465f034f78769bd5403dcfa175c8238bc2bb4767ee25f4e74677f6a556 | 扩展/其他现用 |
| M0713 | D124:142 S39 | ace.m1.a2rscene.r04/a2r-scene-emit | R01 | "success"; artifacts=1 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | 12974016cdce28fef86f982bc2353789c041d559d701fc2698499eb28915a185 | 扩展/其他现用 |
| M0714 | D124:158 S39 | ace.m1.a2rscene.r05/a2r-scene-emit | R01 | "success"; artifacts=1 | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | f560f1394ba907e43f51f007f8f8d2814b59fa52be6ed2b1c501dd02e2191114 | 扩展/其他现用 |
| M0715 | D124:174 S39 | ace.m1.a2rscene.r06_deep/a2r-scene-emit | R01 | "success"; artifacts=1 | ea9701a4684c9018e5ba836f0e875f0522b63682992eacc80cce8868363c86cb | 0412ddc9d7cfa17e3f8a8b6f3850e1ada08e7ad642c78051bb16d30384844616 | 扩展/其他现用 |
| M0716 | D124:190 S39 | ace.m1.a2rscene.r06_trap/a2r-scene-emit | R01 | "success"; artifacts=1 | 470a91024762aeee811ba1e7d6dbffbc56902b960d9482db3b6ac7d51cea62d0 | c468b25931464fbbc38cebab7b5cb100d0b849d89e7e8821dd24440947b778ec | 扩展/其他现用 |
| M0717 | D125:14 S40 | ace.m1.a2rt.t01/a2r-snapshot | R01 | "success"; artifacts=1 | e58f7c63892c43ad128e02569c28cbcc0b0f251ee3dd7e5999a7f8204b402c79 | 533ba6abaa7b640acfb60a148721328418701feb1b0adb0df173423803547c7c | 扩展/其他现用 |
| M0718 | D125:30 S40 | ace.m1.a2rt.t02/a2r-snapshot | R01 | "success"; artifacts=1 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 4c0e661dbee93a3cee6a12a0f5598ecca4c25c1a9348d08fa4adcde0b9e3047c | 扩展/其他现用 |
| M0719 | D125:46 S40 | ace.m1.a2rt.t03/a2r-snapshot | R01 | "success"; artifacts=1 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | cf66ea6e6a8bb2652496107389eae536df7c0112a9e2a02454c2a9b72c83e593 | 扩展/其他现用 |
| M0720 | D125:62 S40 | ace.m1.a2rt.t04/a2r-snapshot | R01 | "success"; artifacts=1 | 2935380215cf39547566d92c8ae51586cc3497427f016b8393a82b9508fa1b19 | 18e4073a69c994955f9a00a9edf850efe7a4c6e3b4c305551326c846e6d27518 | 扩展/其他现用 |
| M0721 | D126:12 S41 | ace.m1.a2rtrap.b13/a2r-trap-emit | R01 | "success"; artifacts=1 | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 2f23536c760b8d8c9afc746113860e2a5e7ec37f0b58dd9e08badf2b173e5227 | 扩展/其他现用 |
| M0722 | D126:28 S41 | ace.m1.a2rtrap.b14/a2r-trap-emit | R01 | "success"; artifacts=1 | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | 6ac065b5a6ca2eff905229b23a2270041b013696fb70c20332c074107de8d43d | 扩展/其他现用 |
| M0723 | D126:44 S41 | ace.m1.a2rtrap.b15/a2r-trap-emit | R01 | "success"; artifacts=1 | 0c05c05f1f13f7096554b76989eec6c51989d3f2389cd3ecb4a3cf17aa8c5e2e | ce5a2a8e4f3a9e2cc332e85cae022f8ed324b5198d592cab6564b22b4178f3ea | 扩展/其他现用 |
| M0724 | D126:60 S41 | ace.m1.a2rtrap.b20/a2r-trap-emit | R01 | "success"; artifacts=1 | b0a37a65580c6e7e7b973ce119dc1c2551a8a5addfe172c21b56ad65bb3708ac | e8c198ebb4795d0e0cde7363bf7e479001d66547430014f0a5890440dcd7ccc9 | 扩展/其他现用 |
| M0725 | D127:9 S28 | ace.m1.k01/a2r-snapshot | R01 | "success"; artifacts=1 | 3c62ff4cd2f7907bfe810d371ccb2b50c85af9822dddbae82c868f45dce89204 | 7afd59ed2b8977aa364cf3290a4b87de63c29efbd232d9f0a692834b86caa757 | 原511 |
| M0726 | D127:9 S28 | ace.m1.k01/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 3c62ff4cd2f7907bfe810d371ccb2b50c85af9822dddbae82c868f45dce89204 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0727 | D127:27 S28 | ace.m1.k03/a2r-snapshot | R01 | "success"; artifacts=1 | d2c3c9c04af812a636f9714037fd63579273687dadac87b1efc0e2c9190c0a18 | ef11a91658c2aea1a6df6ad35d228c642a00d7f633ab1f2d777b55bd3125bae3 | 原511 |
| M0728 | D127:27 S28 | ace.m1.k03/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | d2c3c9c04af812a636f9714037fd63579273687dadac87b1efc0e2c9190c0a18 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0729 | D127:45 S28 | ace.m1.k06/a2r-snapshot | R01 | "success"; artifacts=1 | 64f9641748567f776e63db3063049e01f8b8a4f5c88b55fd23f11e1d5de88ec4 | 5feeb342c793e1c472daf65cd8b3ca6c0e316e78363a2be33edec30a78291de0 | 原511 |
| M0730 | D127:45 S28 | ace.m1.k06/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 64f9641748567f776e63db3063049e01f8b8a4f5c88b55fd23f11e1d5de88ec4 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0731 | D127:63 S28 | ace.m1.r02/a2r-snapshot | R01 | "success"; artifacts=1 | 51fe7459ffc8913137c2bd392909d7143b68c2a6aebd47b1066e34f7c16abca0 | 0dbc173473efb4f94cd626e52b57fcd210a87bf47d6cb11e1240463472e7b3cd | 原511 |
| M0732 | D127:63 S28 | ace.m1.r02/autovm | R05 | "success"; result={"format":"legacy-display","text":"9"} | 51fe7459ffc8913137c2bd392909d7143b68c2a6aebd47b1066e34f7c16abca0 | dd933e5c1e7e5336a824416a03787f0241785ddb3bb8c4f1cc61c9e3127601f3 | 原511 |
| M0733 | D127:81 S28 | ace.m1.r05/a2r-snapshot | R01 | "success"; artifacts=1 | c971be343e1a6ddcf4fc71474060077c30720d70cb6ec4f81855167b9ef71f4b | 34ff0a87e462e8bf2dcbfd5edd7a727f8a928cd3d963ae9efb595a5862bbecc6 | 原511 |
| M0734 | D127:81 S28 | ace.m1.r05/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | c971be343e1a6ddcf4fc71474060077c30720d70cb6ec4f81855167b9ef71f4b | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0735 | D127:99 S28 | ace.m1.r06/a2r-snapshot | R01 | "success"; artifacts=1 | 9851f21afb89ad0053d2578faf3a89843bd44bdaaea3f7e82fcca9412a058f1f | 00db2839c9dc79be63c682cbc2b2dcf365c82ea5dc0df743bdb02c2820c17497 | 原511 |
| M0736 | D127:99 S28 | ace.m1.r06/autovm | R05 | "success"; result={"format":"legacy-display","text":"21"} | 9851f21afb89ad0053d2578faf3a89843bd44bdaaea3f7e82fcca9412a058f1f | e4b29e0436fbb4dd9ee2deaf28f324001a852b8ec03f3a5ed548c09c3c4656c4 | 原511 |
| M0737 | D127:117 S28 | ace.m1.r08/a2r-snapshot | R01 | "success"; artifacts=1 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | eb379611a92ea31d65ceaaca9df6072be869d3e029c5086024739e3a09ca3764 | 原511 |
| M0738 | D127:117 S28 | ace.m1.r08/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0739 | D127:135 S28 | ace.m1.t05/a2r-snapshot | R01 | "success"; artifacts=1 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | c6edaed3f5f86eaf742a553482a5eb9515674ffb2692b171714b53b74e9fbc31 | 原511 |
| M0740 | D127:135 S28 | ace.m1.t05/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0741 | D127:153 S28 | ace.m1.v01/a2r-snapshot | R01 | "success"; artifacts=1 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | 217534942322aaec72652df8fcb429886a82d0ace0b1ca33455114becc62b53f | 原511 |
| M0742 | D127:153 S28 | ace.m1.v01/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0743 | D127:171 S28 | ace.m1.v02/a2r-snapshot | R01 | "success"; artifacts=1 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | a0df5b208db06a45a5dc65eabde2c9fed6d8ea3ad765c124259ddfebbd28628e | 原511 |
| M0744 | D127:171 S28 | ace.m1.v02/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0745 | D127:189 S28 | ace.m1.v04/a2r-snapshot | R01 | "success"; artifacts=1 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | b65102a55c9f49da9a0f4da44aa8b5514078a2744e90da2b4ff6e75daedb6e0a | 原511 |
| M0746 | D127:189 S28 | ace.m1.v04/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0747 | D127:207 S28 | ace.m1.v05/a2r-snapshot | R01 | "success"; artifacts=1 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 69f8a2a04c0f56624dcccae2d239cd97261582d3f5da7fad82345557d324a94f | 原511 |
| M0748 | D127:207 S28 | ace.m1.v05/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0749 | D127:225 S28 | ace.m1.v06/a2r-snapshot | R01 | "success"; artifacts=1 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | 3e67fe07183513a90b3b5d0cb7e178612a51d548d474f30df108656bf32a1ea2 | 原511 |
| M0750 | D127:225 S28 | ace.m1.v06/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0751 | D127:243 S28 | ace.m1.v07/a2r-snapshot | R01 | "success"; artifacts=1 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | 1be95a33be18c54c0138ba7182301a2991cb173b5a1120ce0021effddbbe4e2e | 原511 |
| M0752 | D127:243 S28 | ace.m1.v07/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 原511 |
| M0753 | D128:15 S28 | ace.m1.t07/a2r-snapshot | R01 | {"stage":"parse","code":"auto_lexer_E0001"} | c0bbf1cb39be98b262013e094f7c557f0de5d1676e5b7426b470517563542c05 | aa9fa5a102c46ba60dc01ad0ac38f2e5c6feacaab57ec08a809020d3df3f4dd8 | 原511 |
| M0754 | D128:15 S28 | ace.m1.t07/autovm | R05 | {"stage":"parse","code":"auto_lexer_E0001"} | c0bbf1cb39be98b262013e094f7c557f0de5d1676e5b7426b470517563542c05 | 7c71c7d05d46f5d9ea94455889601bd01f26649dd379d7bc5ec308805e4ed0c8 | 原511 |
| M0755 | D128:32 S28 | ace.m1.k04/a2r-snapshot | R01 | {"stage":"type","code":"auto_type_E0101"} | 4f2b4f3d75cf9edc8f8efa31c0eff33d8c43acf4cdbe64c45581a567e2616f67 | 9ac2141adbf6600196e8433022ce6cf08e31f3b7c149328c5b5510f40cbd4828 | 原511 |
| M0756 | D128:32 S28 | ace.m1.k04/autovm | R05 | {"stage":"run","code":"RuntimeError"} | 4f2b4f3d75cf9edc8f8efa31c0eff33d8c43acf4cdbe64c45581a567e2616f67 | e3e2fd1e6cea42bdea989485930e3aa7394d883f5446520d2b0dcf622a809b87 | 原511 |
| M0757 | D129:10 S42 | ace.m1.m08/ac-native | R37 | "success"; exit=42 | 23043d246afbd8e0187424ed13c2146e8286d780cac16ac978ab52c8c817bbda | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0758 | D129:10 S42 | ace.m1.m08/av | R26 | "success"; exit=42 | 23043d246afbd8e0187424ed13c2146e8286d780cac16ac978ab52c8c817bbda | 66f47c1fc4e6e0c12d76184371613dbd02929fd446770a1a0b040b9971ef2c18 | 原511 |
| M0759 | D129:29 S42 | ace.m1.m01/ac-native | R37 | "success"; exit=0 | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | fa3b2f5ec7fc9be0812f7ca7d56af138529e6ca34f9630db9f99accd66bd5768 | 原511 |
| M0760 | D129:29 S42 | ace.m1.m01/av | R26 | "success"; exit=0 | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | 9deac897d41558b3114f01dfcea14dddc7c33de54ea06a91a48c31fe8ef245d2 | 原511 |
| M0761 | D129:48 S42 | ace.m1.m02/ac-native | R37 | "success"; exit=57 | b34860a4d3b3b9bd4f1f173f9f5e62f3cfeaf8a2ade85ab90778c2b8db654e5c | 8e3b08c5cf4adfbc25055259849322cbe10cc1c62a8e718deb81d2e976f81223 | 原511 |
| M0762 | D129:48 S42 | ace.m1.m02/av | R29 | "success"; exit=57 | b34860a4d3b3b9bd4f1f173f9f5e62f3cfeaf8a2ade85ab90778c2b8db654e5c | 157df6dcba687cd6659a0fe9a32b47777a81c62068d7297a9e8b05fa50504859 | 原511 |
| M0763 | D129:67 S42 | ace.m1.m03/ac-native | R37 | "success"; exit=75 | 0f520150182614da12206b6d3430fa506c908cdb9a346ea661d520a03ac01d5b | 29fd9c555f2ed79145fb90c4aad4e8f4a95bd5671b29b1199ea06f958931f401 | 原511 |
| M0764 | D129:67 S42 | ace.m1.m03/av | R26 | "success"; exit=75 | 0f520150182614da12206b6d3430fa506c908cdb9a346ea661d520a03ac01d5b | d237a86001de818c7535e14294f03646c1e11a41c5ecb5e115177d94c6c6f2f1 | 原511 |
| M0765 | D129:86 S42 | ace.m1.m04/ac-native | R37 | "success"; exit=135 | a893d67b33efbe6f0ca1f36c66c91e532dfe8dc129a6fdb191881d070c50b601 | 407176225150e1eeed5c429c01cd1768f45ae1764fdf459350003646f9d505d3 | 原511 |
| M0766 | D129:86 S42 | ace.m1.m04/av | R26 | "success"; exit=135 | a893d67b33efbe6f0ca1f36c66c91e532dfe8dc129a6fdb191881d070c50b601 | fb5229933a8b9007f6156ab81b2142f748a6419997600ef3489e87112e9c9feb | 原511 |
| M0767 | D129:105 S42 | ace.m1.m05/ac-native | R37 | "success"; exit=42 | ba1d9861a18f6d47c6ebaf3487e8958f8db3d7d61cadfa7b2cfff4b7c2cf247a | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0768 | D129:105 S42 | ace.m1.m05/av | R29 | "success"; exit=42 | ba1d9861a18f6d47c6ebaf3487e8958f8db3d7d61cadfa7b2cfff4b7c2cf247a | 7304961f5149af4fe6883cf5b7501e4524cecefee2b53a25f8a9b274b9d243a2 | 原511 |
| M0769 | D129:124 S42 | ace.m1.m06/ac-native | R37 | "success"; exit=52 | 06b83e19d1981be3b01db77ede9344012f9aa04a41ee9ac7a998d50807c6bbe1 | 36e75e67786fb748beb82c89560ce2f6654787c4760f1f2015c0a783555dfb17 | 原511 |
| M0770 | D129:124 S42 | ace.m1.m06/av | R29 | "success"; exit=52 | 06b83e19d1981be3b01db77ede9344012f9aa04a41ee9ac7a998d50807c6bbe1 | 3fdbadf0cd505d4f4380995b3583f5bafcae035e9284ed52b8bea8814804f061 | 原511 |
| M0771 | D129:143 S42 | ace.m1.m07/ac-native | R37 | "success"; exit=84 | 49f20dea97ffd736acb7ee0ba4a95c029265c2cdf5c495c5c08c6c40abac0951 | c0748ffb9ed23611060936257a5cc6162b72d49bf1c2a97efd4ecaaf476787d7 | 原511 |
| M0772 | D129:143 S42 | ace.m1.m07/av | R26 | "success"; exit=84 | 49f20dea97ffd736acb7ee0ba4a95c029265c2cdf5c495c5c08c6c40abac0951 | afb7012e78e84a3055cd2171a194b85b8d2d667dda5639f4867df4294d4c27f7 | 原511 |
| M0773 | D129:162 S42 | ace.m1.r08/ac-native | R37 | "success"; exit=42 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0774 | D129:162 S42 | ace.m1.r08/av | R26 | "success"; exit=42 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 66f47c1fc4e6e0c12d76184371613dbd02929fd446770a1a0b040b9971ef2c18 | 原511 |
| M0775 | D129:181 S42 | ace.m1.m09/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | c36ba34bb5fc6f3bc35b6affbd194637f46f47f2cc5c107b87e5dc11015621b1 | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M0776 | D129:197 S42 | ace.m1.m10/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | c9da6e0549d40a8fb11f900b50a76090bc04b618be543ec6dd00041c0a99f942 | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M0777 | D129:213 S42 | ace.m1.m11/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | 2059c671b802c4f83f99ac88e61e1081de10c1d6154a0a7964bf989d4e9a539e | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M0778 | D129:229 S42 | ace.m1.m12/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | 145e3179e2fe81d0ea80af7739e9fbbff4603f63d7287cd1bda67735bbd4af7a | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M0779 | D130:18 S43 | ace.m1.r02r05.k02/base | R33 | "success"; exit=42 | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | fcc1aa97afd52954792a553408c7e45f1e654af79bc0bd9fb41202c848ba6c5e | 原511 |
| M0780 | D130:18 S43 | ace.m1.r02r05.k02/fault-s1 | R33 | {"stage":"run","code":"mn.resource.allocation-injected"} | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | 03e41d948b1056d1356bcb796e5a8454cb53944080a061f55f98beaccc8dd212 | 原511 |
| M0781 | D130:18 S43 | ace.m1.r02r05.k02/fault-s4 | R33 | {"stage":"run","code":"mn.resource.execution-budget"} | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | 1d81027fb84848fbcee81f0b80a8d995f1193dbaca96195b80dbe69e61873d79 | 原511 |
| M0782 | D130:18 S43 | ace.m1.r02r05.k02/fault-s9 | R33 | {"stage":"run","code":"mn.resource.allocation-injected"} | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | 0253bc287006b6e1c01e7a89752b43d962aeda9d79688ca638fd14ae7a83b5f8 | 原511 |
| M0783 | D130:18 S43 | ace.m1.r02r05.k02/fault-s9-n2 | R33 | "success"; exit=42 | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | b3a8fdd2ae0c57f89792775e82d4b67719d701ada3529629d594262efd0e8657 | 原511 |
| M0784 | D130:46 S43 | ace.m1.r02r05.k05/base | R33 | "success"; exit=42 | 6fe5880599cf32cd27ac5b73f4959468f1221a3c988984f5d74e64acd9cfc4c5 | fcc1aa97afd52954792a553408c7e45f1e654af79bc0bd9fb41202c848ba6c5e | 原511 |
| M0785 | D130:46 S43 | ace.m1.r02r05.k05/fault-s9-n3 | R33 | "success"; exit=42 | 6fe5880599cf32cd27ac5b73f4959468f1221a3c988984f5d74e64acd9cfc4c5 | 4b0125989f9d3d87cc1c8727e27b661a971a7af8e7e9a63b6ef710baaf70ee2b | 原511 |
| M0786 | D130:46 S43 | ace.m1.r02r05.k05/fault-s9a | R33 | {"stage":"run","code":"mn.resource.allocation-injected"} | 6fe5880599cf32cd27ac5b73f4959468f1221a3c988984f5d74e64acd9cfc4c5 | 12ecd7bc3e708b3b06c826b1276bf0253b3d69ed130b4a5b21bd129a00ccd64f | 原511 |
| M0787 | D130:46 S43 | ace.m1.r02r05.k05/fault-s9b | R33 | {"stage":"run","code":"mn.resource.allocation-injected"} | 6fe5880599cf32cd27ac5b73f4959468f1221a3c988984f5d74e64acd9cfc4c5 | 2c6f80e795e816313dad89f3b6ba322195fc9dfa318951de8850ebc125825738 | 原511 |
| M0788 | D130:71 S43 | ace.m1.r05.avm/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 扩展/其他现用 |
| M0789 | D131:2 S44 | ace.m1.l01-mn889/mn-l01-collections-source | R34 | "success"; exit=42 | ef7e1d5b9144863923a6f3d3df934b241524962a3adad58c4f0f1dcf6c907482 | bbcee1988d866330e175e272f0a43c6bac3bb9bcd27cd9c17d0d555735564b7a | 原511 |
| M0790 | D131:18 S44 | ace.m1.l02-mn889/mn-l02-collections-source | R34 | "success"; exit=42 | 3660bb84506947ab955f98e9d57dca2f4ed469c14858e27ed81da7864c828ca6 | 3c273c5a81f0b70fbd906b5584fa618a395ab16fd2c3804617d8356ab0dc649a | 原511 |
| M0791 | D131:34 S44 | ace.m1.l03-mn889/mn-l03-collections-source | R34 | "success"; exit=0 | a027594739605a6f8e496f41fe8a4e119573bc5674a812b25e3281df8f2eedde | 993c156bcfb8f76ea886f13ce37ef8ecfd9ba7266404a51182314dccfc5727da | 原511 |
| M0792 | D131:50 S44 | ace.m1.l04-mn889/mn-l04-collections-source | R34 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | faa47c160532fe84565459358b82e06a72ca5476e4dc7eab48d86e4b4c7d0b30 | 原511 |
| M0793 | D131:66 S44 | ace.m1.l05-mn889/mn-l05-collections-source | R34 | "success"; exit=42 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 8d1a3f46e8b2e6ad68a3076e22c2857afcd968069516ead7100d4c5035aeb088 | 原511 |
| M0794 | D131:82 S44 | ace.m1.l06-mn889/mn-l06-collections-source | R34 | {"stage":"run","code":"mn.trap.list-index"} | de3a85627019b586ef8b9ccb377e60ef57a0686f15076d57e93910f366b5a884 | 826dbf02f9c0d7b5139a6098c6af2db25810f296929fd24be4a2b4896706058d | 原511 |
| M0795 | D131:98 S44 | ace.m1.l07-mn889/mn-l07-collections-source | R34 | "success"; exit=42 | 067bf2968f2f550565501995f03a4d85ae2903d7c5b1810c7e3ab8da38c634a2 | de4ad079f5fa43497fed116b6db25c2ef84e747406966f9d95808351bbb36f18 | 原511 |
| M0796 | D131:114 S44 | ace.m1.v01-mn889/mn-v01-collections-source | R34 | "success"; exit=42 | bbcf44eb65ebe52b7b584878e552d019ad621b19d7adaa451647addeb460d1a7 | 50c1d5aff5282ba312e0a1df0e9b4b4be6c805d3a62184338bcf26ed58fb402b | 原511 |
| M0797 | D131:130 S44 | ace.m1.v02-mn889/mn-v02-collections-source | R34 | "success"; exit=42 | 3deaff075b327d89167bc97b34ff5a9db95ed29d197253b9ed93565ab4f4eb11 | 8880047d7eed265b22f775d2fded48be2a310800d1c04ccc7c1c23a9c44bc85e | 原511 |
| M0798 | D131:146 S44 | ace.m1.v03-mn889/mn-v03-collections-source | R34 | "success"; exit=3 | 41767e7e7046efc75d06d790e6392a4da4260baa6b95e8e244964f1883383f97 | 32c4e674609283fc053fe759f861c3113b3f7bf9c51c85f42233fb84fb811be0 | 原511 |
| M0799 | D131:162 S44 | ace.m1.v04-mn889/mn-v04-collections-source | R34 | "success"; exit=42 | f56d74ae8ff290bf6251fbb08a7ac8aa799bd2db4b41bc975c5f8a84e8f21a6c | 6978ec111435748297d224e4e18e2673988617826409607f120c3318ed5d5d71 | 原511 |
| M0800 | D131:178 S44 | ace.m1.v05-mn889/mn-v05-collections-source | R34 | "success"; exit=42 | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | ddf86ab81bde5283fed795e430153b4c6727f64970c71db1388286f10f33d353 | 原511 |
| M0801 | D131:194 S44 | ace.m1.v06-mn889/mn-v06-collections-source | R34 | "success"; exit=42 | 1505aee2d7149ba48ef9c172e2032048f3f0832febfe2061bbfe2fc8c2316864 | 7489f9b886398885f9508e1eabe252560407330399593042492e95b16dc93d2f | 原511 |
| M0802 | D131:210 S44 | ace.m1.v07-mn889/mn-v07-collections-source | R34 | "success"; exit=42 | 9c60982a8827f6ddd11db4c75a4778bfb86c825261d86277c920bc31cb55e168 | 053e1c9e0c57abdcaac3818c9a74595041e6e2e417fbe7db41ba5fac84322912 | 原511 |
| M0803 | D131:226 S44 | ace.m1.n01-mn889/mn-n01-collections-source | R34 | "success"; exit=51 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 64666c7857be47888a2e10a90f48355a5f6d405d2d6a457ed69bdd0052a67ca7 | 原511 |
| M0804 | D131:242 S44 | ace.m1.n02-mn889/mn-n02-collections-source | R34 | "success"; exit=42 | cb7d63e1356a05909eaf9a32b069192ce0fa5590da6c34f07b872475ef8474f4 | 1e3036eb4b1a00481f9730adb9774459928504883cead76b86fbf4c5fdb6ced6 | 原511 |
| M0805 | D131:258 S44 | ace.m1.n03-mn889/mn-n03-collections-source | R34 | "success"; exit=53 | 6dc3d3af1778bb4f228ed354202756fc65a9aec8fbc1bce06c015e33f5b2be97 | a5382d8311e1db6993a1c1306c0125335af39ea226e690633fbddbafbf923c9d | 原511 |
| M0806 | D131:274 S44 | ace.m1.n889-list-variant/av | R21 | "success"; exit=42 | 6f3db730b45eb6535914a3d7de9dfe7d9fc5ac7ca022889e37fd57b4eb810192 | 28522a48c3b5a9f86368925b4612f87637081d18035e6cae8719357320facb5c | 原511 |
| M0807 | D131:274 S44 | ace.m1.n889-list-variant/mn | R34 | "success"; exit=42 | 6f3db730b45eb6535914a3d7de9dfe7d9fc5ac7ca022889e37fd57b4eb810192 | 7e8298df520369a937201f55cc31a283590ee0010b18921c4ffe1d20ba925442 | 原511 |
| M0808 | D131:286 S44 | ace.m1.n889-nested-variant/av | R21 | "success"; exit=42 | 47389d8fd49b7317503b021b10d9158062cb61f0e6f1c7d0f8de1774fa4a7851 | 28522a48c3b5a9f86368925b4612f87637081d18035e6cae8719357320facb5c | 原511 |
| M0809 | D131:286 S44 | ace.m1.n889-nested-variant/mn | R34 | "success"; exit=42 | 47389d8fd49b7317503b021b10d9158062cb61f0e6f1c7d0f8de1774fa4a7851 | 7e8298df520369a937201f55cc31a283590ee0010b18921c4ffe1d20ba925442 | 原511 |
| M0810 | D131:298 S44 | ace.m1.n889-optional-variant/av | R21 | "success"; exit=42 | 357cdbe78fee58152ef74221bfbb5493f78b6ea1e7e7194538640d71b807574e | 28522a48c3b5a9f86368925b4612f87637081d18035e6cae8719357320facb5c | 原511 |
| M0811 | D131:298 S44 | ace.m1.n889-optional-variant/mn | R34 | "success"; exit=42 | 357cdbe78fee58152ef74221bfbb5493f78b6ea1e7e7194538640d71b807574e | 7e8298df520369a937201f55cc31a283590ee0010b18921c4ffe1d20ba925442 | 原511 |
| M0812 | D132:31 S45 | ace.m1.nd911.b17/ac-native-strict | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | f5cccdefe89b75e86bc67e48c1e27aeada2660549e82d0d088aee29b430e274a | 原511 |
| M0813 | D132:47 S45 | ace.m1.nd911.b18/ac-native-strict | R37 | {"stage":"typecheck","code":"verify.immutable-write"} | 28f3cd22052664270d5884102c90ee8e87e39757c41f8ec7293ff9328d026cc0 | 8037a2ad0de736b43956ee56c036d4a4129ae35f0021c6f5068899352d5eb9fc | 原511 |
| M0814 | D132:63 S45 | ace.m1.nd911.b19/ac-native-strict | R37 | {"stage":"typecheck","code":"verify.no-return-path"} | c0774f9a0a18b7a3254b37d0b326686cac07ebc872a97da6fcde8618c4debf39 | 6d7996b3e79892193d87401beac7077e555b7ab0d1e17a62a67967eeb4a41c22 | 原511 |
| M0815 | D132:79 S45 | ace.m1.nd911.nb17/ac-native-strict | R37 | "success"; exit=2 | e1f02665846dd554fba2c945c63202d6eaa7ffc6e0fc94d24aa5fbad5754b175 | a5353760d9395ab4dc2f84c5e1d022fbf5f469955bdf834070998969dd7073d3 | 原511 |
| M0816 | D132:95 S45 | ace.m1.nd911.nb18/ac-native-strict | R37 | "success"; exit=2 | 084d0c470ba5bfd7698909a06db5c4762db80c0a19a593a755f0c5ed75fef5d1 | a5353760d9395ab4dc2f84c5e1d022fbf5f469955bdf834070998969dd7073d3 | 原511 |
| M0817 | D132:111 S45 | ace.m1.nd911.nb19/ac-native-strict | R37 | "success"; exit=0 | acea5ca5e801999b06e0c8e0d986af24535cd57d25d4660528b432f33225953b | e00e4b025a5ed3d0dbc5dba2f2e517860720a0d37d8d90f2b3f3dc9c7f89138b | 原511 |
| M0818 | D132:127 S45 | ace.m1.nd911.b17v/ac-native-strict | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 5f5ecdf309553a608e037d928838aa24e586c1e8e8e91716f92b6823ebd848b2 | f5cccdefe89b75e86bc67e48c1e27aeada2660549e82d0d088aee29b430e274a | 原511 |
| M0819 | D132:143 S45 | ace.m1.nd911.b18v/ac-native-strict | R37 | {"stage":"typecheck","code":"verify.immutable-write"} | 78d7c2355632bdb6106dc0c331548d254efd68f8e6c2f91d21f7d48647ab15ec | 8037a2ad0de736b43956ee56c036d4a4129ae35f0021c6f5068899352d5eb9fc | 原511 |
| M0820 | D132:159 S45 | ace.m1.nd911.b19v/ac-native-strict | R37 | {"stage":"typecheck","code":"verify.no-return-path"} | 23cd1b3829cdccb3b5906dea5bd75147fc0d712c53208426082bb0a42c013753 | 6d7996b3e79892193d87401beac7077e555b7ab0d1e17a62a67967eeb4a41c22 | 原511 |
| M0821 | D132:175 S45 | ace.m1.nd911.multi/ac-native-strict | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 988898aa8dadf5a7ec722e46cfc50ad36d4e80f3f33462b7a6f1041bd8236991 | f5cccdefe89b75e86bc67e48c1e27aeada2660549e82d0d088aee29b430e274a | 原511 |
| M0822 | D132:191 S45 | ace.m1.nd911.other-stage-syntax/ac-native-strict | R37 | {"stage":"parse","code":"source.unsupported"} | f1fe7df9e403f02ca7a2584b72a42cf9518b1e1c922a08014ba4c6165f8d1fd1 | e6988c54ec1f013644ea68a53445e3c41a3976bd5aef4bf3e50b1aa8de7c3c38 | 原511 |
| M0823 | D132:207 S45 | ace.m1.nd911.other-stage-bind/ac-native-strict | R37 | {"stage":"resolve","code":"source.undefined"} | a359819d8ad3a8a84cad4b72e671578e192b782fe1d33b4d1799b0463c1f737d | 9283a8c8b0eed36071c12dd1722ebd4ce14131e3c79939475f17b0fa85bf5a91 | 原511 |
| M0824 | D132:223 S45 | ace.m1.nd911.multibyte/ac-native-strict | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 2f59f4fb0d3a71067530c8336f4104dcca2acaeaa2733833918f89a07d3395de | f5cccdefe89b75e86bc67e48c1e27aeada2660549e82d0d088aee29b430e274a | 原511 |
| M0825 | D132:243 S45 | ace.m1.nd911.red.wrong-family/ac-native-strict-wrong-family | R37 | {"stage":"typecheck","code":"verify.immutable-write"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | 12b84e547d0c67848dad71eb86ce9a61cf3f61ffdb9eeff55763fd6ec0c47d92 | 原511 |
| M0826 | D132:259 S45 | ace.m1.nd911.red.wrong-stage/ac-native-strict-wrong-stage | R37 | {"stage":"parse","code":"source.type-mismatch"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | e19000e392cb29fac2420a51b119a4e00c3fb47009bbf70f3d19f3847518071d | 原511 |
| M0827 | D132:275 S45 | ace.m1.nd911.red.generic-shell/ac-native-strict-generic-shell | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | 1347ee38884128f1fdc889e471e499670806715f39ec7b232f3b29b4a4fed9e8 | 29ebaaa7e478fa66ba8e4886b854431d4c65b360e96faf4c01ed4e53d384e6fb | 原511 |
| M0828 | D132:296 S45 | ace.m1.nd911.warm-reject-stops/ac-native-strict-warm | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | eb7096c4ecc436870463096bf11a345a69f923c145fa5f2b052adab62a05df9e | d90b24460f098e3d405b09c6aa421f060a74d55f3318cb19fdd357cff9cad273 | 原511 |
| M0829 | D132:318 S45 | ace.m1.red.i12-avm/autovm-wrong-value | R05 | "success"; result={"format":"legacy-display","text":"15"} | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 07945e022f62d837e04587e43f4dbab55d917847e35ef17cb85f7bf8dfcc80d0 | 扩展/其他现用 |
| M0830 | D132:333 S45 | ace.m1.red.i12-a2r/a2r-wrong-golden | R01 | "success"; artifacts=1 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 297efbb0937e06b01def26e0426a35700ba25bc9887fce4acd88f4870a7b910d | 扩展/其他现用 |
| M0831 | D133:19 S46 | ace.m1.r03.l04/base | R36 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | e33382151a2394d553414c7a0cdd15da0ad4007eb50d3c3470ba2000a74885fe | 原511 |
| M0832 | D133:19 S46 | ace.m1.r03.l04/fault-s1 | R36 | {"stage":"run","code":"mn.resource.allocation-injected"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 050027468b6284dabc738a88dff26bd0914b840092c9d982171375ad85c021b9 | 原511 |
| M0833 | D133:19 S46 | ace.m1.r03.l04/fault-s2 | R36 | {"stage":"run","code":"mn.resource.allocation-injected"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | f3fc730f49932a19a0923af93deb643eba9dc5ac5b005507106e10531e1f2b59 | 原511 |
| M0834 | D133:19 S46 | ace.m1.r03.l04/fault-s4 | R36 | {"stage":"run","code":"mn.resource.execution-budget"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | d82889383b624eed59a946db93a1f9f851c67a31084bef785d5b2703323f58b8 | 原511 |
| M0835 | D133:44 S46 | ace.m1.r03.grow/base | R36 | "success"; exit=24 | 882a9e3643610168df7098c897fc5fb5da0f43aabb27d0c7d6bfe1740cbb857c | 370ab4f62f170693a8e64d8dee6cc56f81726ac9a9814d162f0d465d3cab926f | 原511 |
| M0836 | D133:44 S46 | ace.m1.r03.grow/fault-s3 | R36 | {"stage":"run","code":"mn.resource.allocation-injected"} | 882a9e3643610168df7098c897fc5fb5da0f43aabb27d0c7d6bfe1740cbb857c | 89246517373a869b2949433937972a1f6960b77f625dd5e8ec8810cd8c9cc232 | 原511 |
| M0837 | D133:63 S46 | ace.m1.r03.lstr/base | R36 | "success"; exit=51 | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 15cd018c7219de7e089a53b43f1c17ee7b0a6c6b4fed32709ea5798becb7c57b | 原511 |
| M0838 | D133:63 S46 | ace.m1.r03.lstr/fault-s6 | R36 | {"stage":"run","code":"mn.resource.execution-budget"} | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 2a321ed2b8713eee4e71284c324fbbf1c496409205c95bad098b3e33ba8137ae | 原511 |
| M0839 | D133:63 S46 | ace.m1.r03.lstr/fault-s7 | R36 | {"stage":"run","code":"mn.resource.execution-budget"} | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 2cb874bbe75a18222c807266654194c6e6ffd26420412ef8a69628a5c9694501 | 原511 |
| M0840 | D134:39 S47 | ace.m1.nw932.body-edit.success/body-edit | R37 | "success"; exit=64 | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | 1357705d2b9436dd6d65b7d4e28b50cd77cd857fd2e4f625c9f80eeb19c92088 | 扩展/其他现用 |
| M0841 | D134:58 S47 | ace.m1.nw932.body-edit.language/language-refusal | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | 4949faf86048ecd397c88deb32adb50d60087e1dfd3259df22bdef9f5f52f0c2 | 扩展/其他现用 |
| M0842 | D134:77 S47 | ace.m1.nw932.body-edit.repair/repair-edit | R37 | "success"; exit=85 | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | a25f15a7556305b9a906442cdcdb39e31d20711a253e834eae736f0108eddc69 | 扩展/其他现用 |
| M0843 | D134:96 S47 | ace.m1.nw932.sig-edit.refused/sig-refusal | R37 | {"stage":"verify","code":"mn.incremental.refused.signature-or-context-drift"} | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | b192b7b7c8f230f678d56937bd173db11e0037a4f5454e68a727af344f517122 | 扩展/其他现用 |
| M0844 | D134:116 S47 | ace.m1.nw932.sig-edit.survives/survives | R37 | "success"; exit=43 | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | 1ae959e37fe43bf023d565ccef68aa792655678a62a3fe8024e47b6fa67eeff8 | 扩展/其他现用 |
| M0845 | D134:132 S47 | ace.m1.nw932.multi-decl.invalid/multi-decl-refusal | R37 | {"stage":"verify","code":"mn.incremental.refused.invalid-edit"} | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | 734771fa3852697700ad5fce5062fee0f883d8fdb4499f1558f8a468c5554c2c | 扩展/其他现用 |
| M0846 | D134:151 S47 | ace.m1.nw932.history.a-b-a/history | R37 | "success"; exit=43 | 0983c9bbdb8487a7b95367ac22cd9e17990ffb9011d508e88cbd7e48069ba872 | f658d4a1761fdc103a275ffbadb877886422ddf17bea8c7f9e825bf63b4d9424 | 扩展/其他现用 |
| M0847 | D135:17 S48 | ace.m1.nb17/ac-native | R37 | "success"; exit=2 | e1f02665846dd554fba2c945c63202d6eaa7ffc6e0fc94d24aa5fbad5754b175 | 3da4c1a4e0518bc00f9e570336da02631dea376bb194d6ee2ff87ac0d5c50a18 | 原511 |
| M0848 | D135:17 S48 | ace.m1.nb17/autovm | R05 | "success"; result={"format":"legacy-display","text":"2"} | e1f02665846dd554fba2c945c63202d6eaa7ffc6e0fc94d24aa5fbad5754b175 | e60db4b7e5433ec092bcf78db646a485ad9110e53919ef9fa8f0c42baf7530ca | 原511 |
| M0849 | D135:17 S48 | ace.m1.nb17/av | R24 | "success"; exit=2 | e1f02665846dd554fba2c945c63202d6eaa7ffc6e0fc94d24aa5fbad5754b175 | 68cda7e6e78e11f404859c80fe9af331ae91183005b41cdd3b75e22823c033a3 | 原511 |
| M0850 | D135:38 S48 | ace.m1.nb18/ac-native | R37 | "success"; exit=2 | 084d0c470ba5bfd7698909a06db5c4762db80c0a19a593a755f0c5ed75fef5d1 | 3da4c1a4e0518bc00f9e570336da02631dea376bb194d6ee2ff87ac0d5c50a18 | 原511 |
| M0851 | D135:38 S48 | ace.m1.nb18/autovm | R05 | "success"; result={"format":"legacy-display","text":"2"} | 084d0c470ba5bfd7698909a06db5c4762db80c0a19a593a755f0c5ed75fef5d1 | e60db4b7e5433ec092bcf78db646a485ad9110e53919ef9fa8f0c42baf7530ca | 原511 |
| M0852 | D135:38 S48 | ace.m1.nb18/av | R24 | "success"; exit=2 | 084d0c470ba5bfd7698909a06db5c4762db80c0a19a593a755f0c5ed75fef5d1 | 68cda7e6e78e11f404859c80fe9af331ae91183005b41cdd3b75e22823c033a3 | 原511 |
| M0853 | D135:59 S48 | ace.m1.nb19/ac-native | R37 | "success"; exit=0 | acea5ca5e801999b06e0c8e0d986af24535cd57d25d4660528b432f33225953b | fa3b2f5ec7fc9be0812f7ca7d56af138529e6ca34f9630db9f99accd66bd5768 | 原511 |
| M0854 | D135:59 S48 | ace.m1.nb19/autovm | R05 | "success"; result={"format":"legacy-display","text":"0"} | acea5ca5e801999b06e0c8e0d986af24535cd57d25d4660528b432f33225953b | f83c147653bfd134d348bfc09d3ea2abf41bd00420febb856979bf7efc03398d | 原511 |
| M0855 | D135:59 S48 | ace.m1.nb19/av | R24 | "success"; exit=0 | acea5ca5e801999b06e0c8e0d986af24535cd57d25d4660528b432f33225953b | 0a8ddf242292e153c2acbc0008cccbb592e5dbf55371f8a240b16059d0809064 | 原511 |
| M0856 | D136:13 S49 | ace.m1.e01.ace/ac-native | R37 | "success"; exit=7; stderr#d96dff4f88b8 | ff1194276bdccfa0a30dd3fe6d99ad93fc4c28d9acd75679c0a05647ab47bc3e | fc2f073f48c445641d23f6333ccb90280a3be759387422c5c5570c28280735b2 | 原511 |
| M0857 | D136:13 S49 | ace.m1.e01.ace/av | R24 | "success"; exit=7; trace#d96dff4f88b8 | ff1194276bdccfa0a30dd3fe6d99ad93fc4c28d9acd75679c0a05647ab47bc3e | b4add745d0b455eeb7950c46b41d49f677d9879870708df7f0c523f02efd8927 | 原511 |
| M0858 | D136:32 S49 | ace.m1.e01.legacy/a2r-snapshot | R01 | "success"; artifacts=1 | ed385af9f0ae39c7d0f47851286201d9be0f14c6c2c88384d33f987ce0e61a0b | 1570cbafc9edfea772a429f550da4be1f58c552985020dcbe1a3e276e5b43535 | 原511 |
| M0859 | D136:32 S49 | ace.m1.e01.legacy/autovm | R05 | "success"; result={"format":"legacy-display","text":"7"}; stdout#29c3f568e70a | ed385af9f0ae39c7d0f47851286201d9be0f14c6c2c88384d33f987ce0e61a0b | c1ada993acbbbbb0080bd0a146ea0427e25a06b93d2dadca144dcbf87b165d9d | 原511 |
| M0860 | D136:50 S49 | ace.m1.e02.ace/ac-native | R37 | "success"; exit=103; stderr#d96dff4f88b8 | 0af17830cf88ef169353f6fa66074cfba007c68f22cbfca5174b99777aff8af7 | 0dde3f0fd3f5cbcc150d50babe6bb149840f6e764dd0ddd37f78cfbce5e96d92 | 原511 |
| M0861 | D136:50 S49 | ace.m1.e02.ace/av-data | R26 | "success"; exit=103; trace#d96dff4f88b8 | 0af17830cf88ef169353f6fa66074cfba007c68f22cbfca5174b99777aff8af7 | 4b3ce192b9ec0f5e40f3f3277a6cf155245638919dbfdc75ddcf2295f6eafdfa | 原511 |
| M0862 | D136:69 S49 | ace.m1.e02.legacy/a2r-snapshot | R01 | "success"; artifacts=1 | 3d8d92edb651bcbd9b497e15dc5f82fcf58a7f71835c729066c5a2246274fe6f | b1a9d1623ba53604b28907ff7c97972fb66e8489e0f62266c2af9af2f5cdff7a | 原511 |
| M0863 | D136:69 S49 | ace.m1.e02.legacy/autovm | R05 | "success"; result={"format":"legacy-display","text":"103"}; stdout#29c3f568e70a | 3d8d92edb651bcbd9b497e15dc5f82fcf58a7f71835c729066c5a2246274fe6f | cfde476e0fdf6cd6f2e0d8fad7cf8cf2aeff8bc26a4bbf167a571ea2182b217d | 原511 |
| M0864 | D136:87 S49 | ace.m1.i05.ace/history-revisit-run | R37 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 2b8640de76e8a0c03f6bbd6b0cbdeb8bb9af9bcaef67c4ec1525dbfbe8c16f7c | 扩展/其他现用 |
| M0865 | D136:109 S49 | ace.m1.i11.ace/edit-turnaround-p90 | R37 | "success"; exit=37 | 971e0e5477c48ae66bb0de099e28f939395bd636ff2085d7989024cf4367defd | 85b40b3beab34e95fd1d3a4014216898aa06392f48a4283df5720a204e798f75 | 扩展/其他现用 |
| M0866 | D136:147 S49 | ace.m1.i02.ace/callee-body-edit | R37 | "success"; exit=10 | 971e0e5477c48ae66bb0de099e28f939395bd636ff2085d7989024cf4367defd | f43fce9e3e6c014e699ce8e29fa67949b3f4e98af78af57a1b6c07a254f2a750 | 扩展/其他现用 |
| M0867 | D136:167 S49 | ace.m1.i09.ace/refused | R37 | {"stage":"verify","code":"mn.incremental.refused.signature-or-context-drift"} | 971e0e5477c48ae66bb0de099e28f939395bd636ff2085d7989024cf4367defd | 3a2ea4440fbabe722d32d23d5681057f59b1cea4f3107d1b6246e22c104b9a0b | 原511 |
| M0868 | D136:167 S49 | ace.m1.i09.ace/survives | R37 | "success"; exit=7 | 971e0e5477c48ae66bb0de099e28f939395bd636ff2085d7989024cf4367defd | e0760985e1f5522925f4f7bbd576c5e23855f11c214ea6a9b250fe6e3de40e4b | 原511 |
| M0869 | D136:190 S49 | ace.m1.i04.ace/declaration | R34 | "success"; exit=103 | 30efe99fe37f9227562318f4bbabdd827da7db1b7d3bbc4c8ee1d84519b7fc63 | ba7eabfd6390fb5d1508d1fe512ee87794d012df134a4f147437a63cd3128063 | 原511 |
| M0870 | D136:210 S49 | ace.m1.i04.av/body-edit-precision | R25 | "success"; exit=105 | 388ac945d4cef26b4c9dcacf4df1ae3e77251b3832e21629940682bd0ee6576f | 5e70ae0788ca3bf0f337957091e9ea644c7c19059ad5f282faaf3cca00bf498f | 原511 |
| M0871 | D136:210 S49 | ace.m1.i04.av/layout-change-rebuild | R25 | "success"; exit=104 | 388ac945d4cef26b4c9dcacf4df1ae3e77251b3832e21629940682bd0ee6576f | d3861e34d1f67c857475998007e741e4224f382063eb0ecb8b0fd27c8284ec70 | 原511 |
| M0872 | D136:210 S49 | ace.m1.i04.av/unused-declaration-carry | R25 | "success"; exit=104 | 388ac945d4cef26b4c9dcacf4df1ae3e77251b3832e21629940682bd0ee6576f | 38c30eba066f52778ee2e6cb56f6dfc0a723f626674c62f3c1ff1bf12df72c0e | 原511 |
| M0873 | D136:244 S49 | ace.m1.i09.av/partial-rebuild-refused | R25 | "success"; exit=104 | 388ac945d4cef26b4c9dcacf4df1ae3e77251b3832e21629940682bd0ee6576f | 03c16ec9f927afbdbd80353b4d79b87ec81068283b4f36c2cb2427cc4e0a2c5b | 原511 |
| M0874 | D137:23 S50 | ace.m1.res-r01-av866/av-res-r01-continuity | R30 | "success"; exit=2100 | fab164aa38170493d9fcf630f1cdf14502fa60ad35b4085073f1664248ab00da | 5789d5b47a4dc2f0d259050af72bdbb754e7b122f19763adf79cd8af90d16615 | 原511 |
| M0875 | D137:40 S50 | ace.m1.res-r02-av866/av-res-r02-online-swap | R30 | "success"; exit=2608 | fab164aa38170493d9fcf630f1cdf14502fa60ad35b4085073f1664248ab00da | 2891699f903c36900be606b94bed65abdacfe098c3b76fda3f85df0c0e483c28 | 原511 |
| M0876 | D137:60 S50 | ace.m1.res-r03-av866/av-res-r03-churn-two-swaps | R30 | "success"; exit=3070 | fab164aa38170493d9fcf630f1cdf14502fa60ad35b4085073f1664248ab00da | 554f74b352c9013e16a78cbaeb73045b7560e5c162491bd066f0088320e8623d | 原511 |
| M0877 | D138:26 S51 | ace.m1.r02.l04/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 扩展/其他现用 |
| M0878 | D138:26 S51 | ace.m1.r02.l04/autovm-fault-1 | R05 | {"stage":"run","code":"AllocFault"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 20cb2c9539479f6a39088290cae629ff18d453f6065843e031e5cf14a4640712 | 扩展/其他现用 |
| M0879 | D138:26 S51 | ace.m1.r02.l04/autovm-fault-2 | R05 | {"stage":"run","code":"AllocFault"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 646d8ef74e62ee262bbd4a0ed23ae5db009235f1b90e364c8eea6f7f2156323f | 扩展/其他现用 |
| M0880 | D138:26 S51 | ace.m1.r02.l04/autovm-fault-3 | R05 | "success"; result={"format":"legacy-display","text":"42"} | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 386b6a767c899035c1df9a95c5a126053333db5b2b3b0e9285b3623d0ce10177 | 扩展/其他现用 |
| M0881 | D138:26 S51 | ace.m1.r02.l04/av-r02-l04-sweep | R32 | "success" | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | d57d0aaebf05bdc9b6b1ec229e9c55f8ca0fbae4f9543b8e89f15e89299719df | 原511 |
| M0882 | D138:53 S51 | ace.m1.r02.l05/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 扩展/其他现用 |
| M0883 | D138:53 S51 | ace.m1.r02.l05/autovm-fault-1 | R05 | {"stage":"run","code":"AllocFault"} | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 20cb2c9539479f6a39088290cae629ff18d453f6065843e031e5cf14a4640712 | 扩展/其他现用 |
| M0884 | D138:53 S51 | ace.m1.r02.l05/autovm-fault-2 | R05 | {"stage":"run","code":"AllocFault"} | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 646d8ef74e62ee262bbd4a0ed23ae5db009235f1b90e364c8eea6f7f2156323f | 扩展/其他现用 |
| M0885 | D138:53 S51 | ace.m1.r02.l05/autovm-fault-3 | R05 | "success"; result={"format":"legacy-display","text":"42"} | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 386b6a767c899035c1df9a95c5a126053333db5b2b3b0e9285b3623d0ce10177 | 扩展/其他现用 |
| M0886 | D138:53 S51 | ace.m1.r02.l05/av-r02-l05-sweep | R32 | "success" | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 1578068d30481560365f45cd0a2780ea14ff3973e3bd2c0f8f308d9812bd571c | 原511 |
| M0887 | D138:80 S51 | ace.m1.r02.v05/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 扩展/其他现用 |
| M0888 | D138:80 S51 | ace.m1.r02.v05/autovm-fault-1 | R05 | {"stage":"run","code":"AllocFault"} | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 20cb2c9539479f6a39088290cae629ff18d453f6065843e031e5cf14a4640712 | 扩展/其他现用 |
| M0889 | D138:80 S51 | ace.m1.r02.v05/autovm-fault-2 | R05 | {"stage":"run","code":"AllocFault"} | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 646d8ef74e62ee262bbd4a0ed23ae5db009235f1b90e364c8eea6f7f2156323f | 扩展/其他现用 |
| M0890 | D138:80 S51 | ace.m1.r02.v05/autovm-fault-3 | R05 | "success"; result={"format":"legacy-display","text":"42"} | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | 386b6a767c899035c1df9a95c5a126053333db5b2b3b0e9285b3623d0ce10177 | 扩展/其他现用 |
| M0891 | D138:80 S51 | ace.m1.r02.v05/av-r02-v05-sweep | R32 | "success" | d3a29b69a26181aa5c4868b4598ff47cd03ad177f209a63a0f35d3df9d41a3d1 | ff375650e7f85d1156af75cd5929d76244cbfee85b1f1f305749aed2217a4684 | 原511 |
| M0892 | D138:107 S51 | ace.m1.r02.lstr/autovm | R05 | "success"; result={"format":"legacy-display","text":"51"} | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 83b0a0394d0568da51daa18d8fb1cb72ff3cc698bd8d7e938219f194dd559a2a | 扩展/其他现用 |
| M0893 | D138:107 S51 | ace.m1.r02.lstr/autovm-fault-1 | R05 | {"stage":"run","code":"AllocFault"} | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 20cb2c9539479f6a39088290cae629ff18d453f6065843e031e5cf14a4640712 | 扩展/其他现用 |
| M0894 | D138:107 S51 | ace.m1.r02.lstr/autovm-fault-2 | R05 | "success"; result={"format":"legacy-display","text":"51"} | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | e6fac610e2919e7344b27e904f4760c0d57d575ce8954ad560a68f429f5ffc7e | 扩展/其他现用 |
| M0895 | D138:107 S51 | ace.m1.r02.lstr/av-r02-lstr-sweep | R32 | "success" | 971fae38abe80696af6f3bf2298b2d533545351bfcfeaeff9e93b07ad4a11ffd | 3da39abee589fae8e0ad21fb98cc91362463c20a003bd907dabdd1a675cd278c | 原511 |
| M0896 | D138:131 S51 | ace.m1.r02.capture/av-r02-capture-clone | R32 | "success" | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | 5dc89b1a4764b1eff903a682a798ca05d9abd2b1cdd50bc97f153d9d82b901ce | 原511 |
| M0897 | D138:147 S51 | ace.m1.r02.k02/autovm | R05 | "success"; result={"format":"legacy-display","text":"42"} | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | ac1738c741d52419fe511218f2e058333d23ac1d0d5ceee716c441d17526c79f | 扩展/其他现用 |
| M0898 | D138:147 S51 | ace.m1.r02.k02/autovm-fault-none | R05 | "success"; result={"format":"legacy-display","text":"42"} | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | cff58f088fe2e8958fe4e1460bf593481f6b188052ef94681cd8b297538464df | 扩展/其他现用 |
| M0899 | D138:147 S51 | ace.m1.r02.k02/av-r02-k02-sweep | R32 | "success" | 11218c51b3a1aeba2dd9b1fa25164a34c1d0f25087925050b2f9d8e4006f5c1a | b370729ace694408e9faedc8d1978b5042c473de8817cf800ff016eb5171df01 | 原511 |
| M0900 | D138:168 S51 | ace.m1.r03.stale/av-r03-legal-neighbour | R21 | "success"; exit=130 | 8a6df4cbfcd688a0c8b798b690df476004815afc3b6e3309f190039cb6f797ba | fac192cb95179fd04f657496ead88870d071c44bc8da82a784f5e59eaaebe158 | 原511 |
| M0901 | D138:168 S51 | ace.m1.r03.stale/av-r03-stale-insert | R32 | "success" | 8a6df4cbfcd688a0c8b798b690df476004815afc3b6e3309f190039cb6f797ba | edfeac869cce69b67270d913c76ea01dce6291b4fc35b9a2e207e8751f103211 | 原511 |
| M0902 | D138:168 S51 | ace.m1.r03.stale/av-r03-stale-push | R32 | "success" | 8a6df4cbfcd688a0c8b798b690df476004815afc3b6e3309f190039cb6f797ba | 0a1b7a1e06b3642bce8912fbe5999e08493e602bf80b3e6977d9d95038049646 | 原511 |
| M0903 | D138:168 S51 | ace.m1.r03.stale/av-r03-stale-remove | R32 | "success" | 8a6df4cbfcd688a0c8b798b690df476004815afc3b6e3309f190039cb6f797ba | 37f7501f948b2802c964ac20d0f82448e2beeacb5f6d748fd27fba5f78d02f3d | 原511 |
| M0904 | D138:168 S51 | ace.m1.r03.stale/av-r03-stale-replace | R32 | "success" | 8a6df4cbfcd688a0c8b798b690df476004815afc3b6e3309f190039cb6f797ba | 6156b7b81b228415f5f9aed2608d781b9758ce792dfc1c1167748ac6eadf8c79 | 原511 |
| M0905 | D138:196 S51 | ace.m1.r03.static-reject/av-r03-static-reject | R21 | {"stage":"resolve","code":"bind/source.iterate-mutate"} | 72ddd2fedf331feb54b5d76c499fcc6aa0156d8725c84ca2fafe57f658b28a44 | 34e697d5601c9c5558a86fad4b7e9cf6ace2c7ebc38823f1c1ed7bcae8e5b9a8 | 原511 |
| M0906 | D139:13 S52 | ace.m1.r01.av/av-owned | R31 | "success"; exit=732 | d186612a6bc0a49df874bd8fc1c7ef4364fc290734262585ce2afbcd162046d6 | cfa32e77eca3b6e743bf4a049d373cfbd890077bc43d05d22e98f6c48fc2a44e | 原511 |
| M0907 | D139:29 S52 | ace.m1.r01.mn/ac-native | R35 | "success"; exit=969 | d37211e4eebe81adc499a12154f0520ba9dd1c5bbbe61a3d5c99a98348a48566 | 349ae545be8c1a8a137916018fb56433f83cf1f66f12c36f41bc5c93a4d55f91 | 原511 |
| M0908 | D139:45 S52 | ace.m1.r04.av/av | R31 | "success"; exit=42 | 0cc7b5238965c8817131e17c79e7148872595c32363f7d36d9f0dd5c692dfb70 | 6fe1b1e4d1931920956ce10a6c22d315ad03dac1eacfd66e25073ee0e5b874a4 | 原511 |
| M0909 | D139:61 S52 | ace.m1.r04.mn/ac-native | R35 | "success"; exit=42 | 0cc7b5238965c8817131e17c79e7148872595c32363f7d36d9f0dd5c692dfb70 | 11a06e0be813efd3d7fe614c50d4692c1e4e3c530a011e9e1045a716c4d46038 | 原511 |
| M0910 | D139:77 S52 | ace.m1.r06-budget.av/av | R31 | {"stage":"run","code":"av.resource.instr-steps"} | ea9701a4684c9018e5ba836f0e875f0522b63682992eacc80cce8868363c86cb | da4795d6c39a6a05c19660fb2fc88fcd2d55f8636b97a531bccbe9f4a3a71749 | 原511 |
| M0911 | D139:93 S52 | ace.m1.r06-budget.mn/ac-native | R35 | {"stage":"run","code":"mn.resource.execution-budget"} | ea9701a4684c9018e5ba836f0e875f0522b63682992eacc80cce8868363c86cb | 695e4fa325fefaec54b9c4f0e0b13295c552f1f8ac8491acbeea6973456a466d | 原511 |
| M0912 | D139:109 S52 | ace.m1.r06-cancel.av/av | R31 | {"stage":"run","code":"av.cancelled"} | ea9701a4684c9018e5ba836f0e875f0522b63682992eacc80cce8868363c86cb | 8817ce646a62cbe22d773e0ce1655e50cb181c497024376309a4653e20d30a6c | 原511 |
| M0913 | D139:125 S52 | ace.m1.r06-cancel.mn/ac-native | R35 | {"stage":"run","code":"mn.resource.cancelled"} | ea9701a4684c9018e5ba836f0e875f0522b63682992eacc80cce8868363c86cb | 7a222432f140977f7a66a2f4a2c8b31c7a0c1e7f2e10ef007b77cca20cf5a865 | 原511 |
| M0914 | D139:141 S52 | ace.m1.r06.avm/autovm | R05 | {"stage":"run","code":"DivisionByZero"} | 5d7d1f937870fea6a35c179492dc0ada0600b61eb1fb96e28257e56c904eefb6 | 1c27e2a5782568a153a77333cd916b53abde2e6bf4e24105d94cca8fb4b517a2 | 扩展/其他现用 |
| M0915 | D139:156 S52 | ace.m1.r06-trap.av/av | R31 | {"stage":"run","code":"av.trap.div-by-zero"} | 5d7d1f937870fea6a35c179492dc0ada0600b61eb1fb96e28257e56c904eefb6 | 14d620c6fb710dbe6b00849a94c8cfe1b26ee4090c5b74374f4eb0eb0e46110f | 原511 |
| M0916 | D139:172 S52 | ace.m1.r06-trap.mn/ac-native | R35 | {"stage":"run","code":"mn.trap.overflow"} | 5d7d1f937870fea6a35c179492dc0ada0600b61eb1fb96e28257e56c904eefb6 | 8e32ccda2f5ea7a1e9da6bee8e0086318fbc6c57f07aa1291fd67f06acb16b23 | 原511 |
| M0917 | D140:15 S28 | ace.m1.session.av/body-edit-run | R24 | "success"; exit=720 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | b262386a901b4d2ade4f34939ff3f3463e9da6cce9597e93bb558b00a2d1d008 | 原511 |
| M0918 | D140:15 S28 | ace.m1.session.av/edit-persist-across-workers | R24 | "success"; exit=720 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | b9ed6699593860bd48c346cce345fdecf9e405a86b2245c321525972ef37e044 | 原511 |
| M0919 | D140:15 S28 | ace.m1.session.av/hot-run | R24 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 31f33b931b713fc340adbc15b47e03ea8f029cbef1844fb9f9d4f31e48707fcc | 原511 |
| M0920 | D140:15 S28 | ace.m1.session.av/warm-run | R24 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | ad44b39fe3db81d3c269896799aea48eac6dcf38c1dfba29a0d99e18e55622d5 | 原511 |
| M0921 | D140:62 S28 | ace.m1.session.mn/edit-then-restart-run | R37 | "success"; exit=720 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | bbda4c0360aa6693bbe571e1fc12fc83e8055081d4bc3943fef0b646296d7549 | 原511 |
| M0922 | D140:62 S28 | ace.m1.session.mn/fresh-run | R37 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 050b5ae5cf38c8913d691755f76c784d1747ce0b263457ad9f520bc348f5e911 | 原511 |
| M0923 | D141:5 S53 | ace.m1.s01-av909/av-s01-closures-source-body-edit | R19 | "success"; exit=46 | 54e5952b7c7152d10188101b41f3e26210604ce33a2e45071f242eb550830f07 | 376842d890f0d1251ec1e6128d70b6bb32d21543831ccccadab9b07c499ee81a | 原511 |
| M0924 | D141:5 S53 | ace.m1.s01-av909/av-s01-closures-source-fresh | R19 | "success"; exit=43 | 54e5952b7c7152d10188101b41f3e26210604ce33a2e45071f242eb550830f07 | 5f840b3ffc709bc0b99476dd63ed13f7e7676393b68250675d17c783e4c86ffc | 原511 |
| M0925 | D141:28 S53 | ace.m1.s02-av909/av-s02-closures-source-body-edit | R19 | "success"; exit=44 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | 9fbf26cb2f87f00d9c68f60df4e08ec0acea9e4b83b0d3f0a2b9565845e53735 | 原511 |
| M0926 | D141:28 S53 | ace.m1.s02-av909/av-s02-closures-source-fresh | R19 | "success"; exit=42 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | d1420e4e60f3182a5057f8f75115d1397eea62305f16053b4ecbc0d58722934e | 原511 |
| M0927 | D141:28 S53 | ace.m1.s02-av909/av-s02-closures-source-restart | R19 | "success"; exit=42 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | dba45cf8cb87510a9d2fda1616c14c98a5a9f4efdc53409b2a1e39fcb247b7ff | 原511 |
| M0928 | D141:54 S53 | ace.m1.s03-av909/av-s03-closures-source-body-edit | R19 | "success"; exit=45 | aaaae95383eeb089971b6cc2f4083eb4bbcabdceb504d365655a82efc5f12500 | 4acc3aa1c4e7f3826965326db540132904ec637da364ef976ecd0cb2ba2e5e7c | 原511 |
| M0929 | D141:54 S53 | ace.m1.s03-av909/av-s03-closures-source-fresh | R19 | "success"; exit=42 | aaaae95383eeb089971b6cc2f4083eb4bbcabdceb504d365655a82efc5f12500 | 2dfe51b8c0c00e7c53e4ddbaea65a5180424861077f1c6bb68e2301e6eece9e5 | 原511 |
| M0930 | D141:77 S53 | ace.m1.s04-av909/av-s04-closures-source-fresh | R19 | "success"; exit=44 | e9e1a43bc48fc88993f3b1beaa319b47f6a6a598da0fb364c919d78669795044 | c42143a321de6bd6d9c6609a652b79e7fed890241e2a1169a7d85dca16ad867a | 原511 |
| M0931 | D141:93 S53 | ace.m1.s05-av909/av-s05-closures-source-fresh | R19 | "success"; exit=82 | 06bea500b43786297d866a9fd8d28bc36a93f41ef60e0f273366ce0d932921d9 | 1fd9a23001ad60ba91f91c81473bf6c99a36e1a14c6b50c4c2f96b3399b59d3d | 原511 |
| M0932 | D141:109 S53 | ace.m1.s06-av909/av-s06-closures-source-fresh | R19 | "success"; exit=84 | 127be75c4899d77d43c1f1d1dcbb24152a9ac5b6ce4d634ace05b510a35df68b | 7c984b7b8fa4f26065e284ab6a753b795d83b9ecd90958bcb0fb6646cd60fa7b | 原511 |
| M0933 | D141:125 S53 | ace.m1.s07-av909/av-s07-closures-source-reject | R19 | {"stage":"resolve","code":"source.closure-shared-mutable"} | fbbfc773ca1f4598efba8d6b6bd5038e453c6ae18d30319706b26878e1e3853b | d4d8cba2ebcdc1539289159840340654eda9e00644fec1dc838cb134b4ca0c78 | 原511 |
| M0934 | D141:141 S53 | ace.m1.s08-av909/av-s08-closures-source-reject | R19 | {"stage":"verify","code":"verify.immutable-write"} | 57ba689912f3a46b0a7e65e852e04c568fd245dc323a6203e61415b3a2ada8e8 | ba9abf0299ee2bc09661a35880c4794c1e2df9a847f37441c6089d1ba6c370ab | 原511 |
| M0935 | D142:24 S54 | ace.m1.source-composite-928.collections-parse/parse | R42 | "success"; artifacts=1 | a6a91b8e8313a48e1a21aaf79301aed6b360f4485d70c78d64d5d9b9fa124597 | 60f950391034eec15622267d2f540c8e892861ef4d146f366ba61663a0d2d256 | 扩展/其他现用 |
| M0936 | D142:40 S54 | ace.m1.source-composite-928.closures-parse/parse | R42 | "success"; artifacts=1 | 99722d41944eaac75a88f55cb5f0e9c7c42e2ddc126c7429ba8388fce499213e | 7917a25e04505b6bc2cd08e4b4e078bb83b5bde48b5a67737cfbefebae65b877 | 扩展/其他现用 |
| M0937 | D142:56 S54 | ace.m1.source-composite-928.same-text-parse/parse | R42 | "success"; artifacts=1 | 80846978291f7b9bb5a02ee79649fc2e340e470412dc68f2ff75eddda3a20839 | 73373a3f8a33d9e7459d97f2a9fdfbed60736e40bfbeb0c4c54e9bcaa0fcc518 | 扩展/其他现用 |
| M0938 | D142:72 S54 | ace.m1.source-composite-928.gate-collections-under-closures/parse-gate | R42 | {"stage":"parse","code":"syntax.parse"} | a6a91b8e8313a48e1a21aaf79301aed6b360f4485d70c78d64d5d9b9fa124597 | 2df5d7366459d609f4ea652cb748fedfafd89c2e63231d83fbd7c35e3dfa4273 | 扩展/其他现用 |
| M0939 | D142:88 S54 | ace.m1.source-composite-928.gate-closures-under-collections/parse-gate | R42 | {"stage":"parse","code":"syntax.parse"} | 99722d41944eaac75a88f55cb5f0e9c7c42e2ddc126c7429ba8388fce499213e | 95cb1fc33f5c1900a5c4a63ec3010b4548490ee8e3ba060f4ec0a4ab2f0eb594 | 扩展/其他现用 |
| M0940 | D142:104 S54 | ace.m1.source-composite-928.av-l04/av-l04 | R21 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 52966e436e29e9b6a81c7e7c48262fcde1b5ba94946881b6b2a5f116ced378c1 | 扩展/其他现用 |
| M0941 | D142:104 S54 | ace.m1.source-composite-928.av-l04/mn-l04 | R34 | "success"; exit=42 | f4f27ee5ce2401353cc9a69fc446ca26baf153361c03b31a22645eeca2878f97 | 66362cc44ec5a5f24b55c3e5063c1f6baac902b0fe340c47d60e42d3eda8539f | 扩展/其他现用 |
| M0942 | D142:123 S54 | ace.m1.source-composite-928.av-l05/av-l05 | R21 | "success"; exit=42 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 680868a3a57c06757442fedc28ff80ada5e04b550f39f24be9680ebf1bc6d7ed | 扩展/其他现用 |
| M0943 | D142:123 S54 | ace.m1.source-composite-928.av-l05/mn-l05 | R34 | "success"; exit=42 | 5d48215241c6debb3ec8121dacda9706ed1cc383d549e3b7927b72eb4212dc11 | 9f727feada21d15e92ed6f6749c02303e8e1618a3aee212940e8063c1df0e813 | 扩展/其他现用 |
| M0944 | D142:142 S54 | ace.m1.source-composite-928.av-k02/av-k02 | R19 | "success"; exit=42 | ae0e48715462947dbd3885e4e3e3fdea2b674cd8589bc7052971787b95ecdffd | 4bb3f13b10c4977363f5a1c5990953c965be3f16101987035accb55ff4a85e34 | 扩展/其他现用 |
| M0945 | D142:158 S54 | ace.m1.source-composite-928.av-k06/av-k06 | R19 | "success"; exit=42 | b93fd5be95ddabd36cadd047e3e4654a2f56c90ccf08f48361e996d150f1674e | bbeb1c2ba0e1e251dc7d9e56e44925aa0107555e29fb6af6a09ecf6509882c09 | 扩展/其他现用 |
| M0946 | D142:174 S54 | ace.m1.source-composite-928.av-s02/av-s02 | R19 | "success"; exit=42 | c18a3b0b57f5735da699b5d43027580d7ee72357491384547c3a55e4e41e180c | 16be818bf8e9b3ed7a2125abe30f4f0956e99db76b40f4e535164e04963a58c8 | 扩展/其他现用 |
| M0947 | D142:190 S54 | ace.m1.source-composite-928.av-same-text-owners/av-u1-same-text | R19 | "success"; exit=1112 | f5465602de8238af26af68c3c1ad657aa0b8ee8e545863659fda09915e2cb3e1 | c42d4b69eb1c37521348fe780d23a2ca5fdb818db4ac10245863e2c62aad1f57 | 扩展/其他现用 |
| M0948 | D143:27 S55 | ace.m1.cte936.sum42/ac-native | R37 | "success"; exit=42 | f8a7c9b6bf730faaee228cb492fd3ceec84740e795a39fd6d6a1ff4e1137135e | 19857b130a3cd9dc8966e0b82bb8e2ef7dffa8958ea33c7758031c104efb5863 | 扩展/其他现用 |
| M0949 | D143:27 S55 | ace.m1.cte936.sum42/av | R24 | "success"; exit=42 | f8a7c9b6bf730faaee228cb492fd3ceec84740e795a39fd6d6a1ff4e1137135e | 397cd7269586ef30b002178d369234375cbfb5bc8f93840b5a409a96dbc931c2 | 扩展/其他现用 |
| M0950 | D143:46 S55 | ace.m1.cte936.sum43/ac-native | R37 | "success"; exit=43 | 7250146ccd8303c7179457b34aad55a348d63205b2c6e61f2b1c7e26e9f868f7 | 9478e6980465b9e3a56d1a1d8750cbcd662d26f4dd8a5aafb42aa24076c911f4 | 扩展/其他现用 |
| M0951 | D143:46 S55 | ace.m1.cte936.sum43/av | R24 | "success"; exit=43 | 7250146ccd8303c7179457b34aad55a348d63205b2c6e61f2b1c7e26e9f868f7 | 5172bfbcd0cbe51ff7233ae9deb129cbc5751ce34ab81f4ff74ddf70a2f1cced | 扩展/其他现用 |
| M0952 | D143:65 S55 | ace.m1.cte936.boollit/ac-native | R37 | "success"; exit=7 | 1e4da798f2eae44a408c3092f8db21fe6ddbb77324eed315091a703f23eb33ad | 79c1e5f46cf6ac004c22b435c5a1b3f626d2771b0cc150d534dbe4e7df3193e2 | 扩展/其他现用 |
| M0953 | D143:65 S55 | ace.m1.cte936.boollit/av | R24 | "success"; exit=7 | 1e4da798f2eae44a408c3092f8db21fe6ddbb77324eed315091a703f23eb33ad | 417cdcf1d625d76a054da1c400e066db8a031e27adf9d146bfd2714b4d07c9f3 | 扩展/其他现用 |
| M0954 | D143:84 S55 | ace.m1.cte936.meta2/ac-native | R37 | "success"; exit=42 | 8225ff2a2be7064c57d34c395ccec90cd86963678629614968b8495b2ef09c04 | 094540735d8a4454c0baf48f2e3564bc39d2148867c6a27ded550480b11c8ec1 | 扩展/其他现用 |
| M0955 | D143:84 S55 | ace.m1.cte936.meta2/av | R24 | "success"; exit=42 | 8225ff2a2be7064c57d34c395ccec90cd86963678629614968b8495b2ef09c04 | 5888eb8f970dfba9b278274bf2f77fd9c49a28700a90e7bfc9c5336c1b2fd53f | 扩展/其他现用 |
| M0956 | D143:103 S55 | ace.m1.cte936.meta3/ac-native | R37 | "success"; exit=43 | 60af7d5dade32de1cabb045b35bc053a6206d6a589f581168343c5e4b69216f6 | 72f979d9b7b0629ab5790ca73ee0151a6df42d38010a26f6903e8df524be23cc | 扩展/其他现用 |
| M0957 | D143:103 S55 | ace.m1.cte936.meta3/av | R24 | "success"; exit=43 | 60af7d5dade32de1cabb045b35bc053a6206d6a589f581168343c5e4b69216f6 | 713c319b49c7f2b1b0657bcd9e8ef7d73552988ce415caf122a47fd3b3bc2ca2 | 扩展/其他现用 |
| M0958 | D143:122 S55 | ace.m1.cte936.choose42/ac-native | R37 | "success"; exit=42 | 3850f0daa5af90a4c2146e31ea0104577253e69d2618afc0d7abb796d6e55298 | 59227681bdac88459c1a62292f6edbc25c90e5e2a191956786261f1eabfecd1e | 扩展/其他现用 |
| M0959 | D143:122 S55 | ace.m1.cte936.choose42/av | R24 | "success"; exit=42 | 3850f0daa5af90a4c2146e31ea0104577253e69d2618afc0d7abb796d6e55298 | 84475d321d1359ae6dcdc206334946f38b4a3a8181d78c6c6bcf59484fa3aeda | 扩展/其他现用 |
| M0960 | D143:141 S55 | ace.m1.cte936.pick40/ac-native | R37 | "success"; exit=40 | 03f48764a48d599ba13fd944081ac5a3d1866657bec1c8703aed41672b42dfec | 42bc8c775a869ed4d374072334bb473e2789665c89bacf8572acd7d3a2e8b7d1 | 扩展/其他现用 |
| M0961 | D143:141 S55 | ace.m1.cte936.pick40/av | R24 | "success"; exit=40 | 03f48764a48d599ba13fd944081ac5a3d1866657bec1c8703aed41672b42dfec | 4e119d87d58bb7a9c29b7d66128216d1ea122f7137364774ae99f8d5a834cce7 | 扩展/其他现用 |
| M0962 | D144:28 S56 | ace.m1.inst935.choose42/ac-native | R37 | "success"; exit=42 | cd56041f110610845db4b693591f98e4462bb8dd73f1357f720f6645e9b91848 | 6666cb2555fe8a8053e516eae4628be2e4910aa29709e4f9a40e26db347df4a9 | 原511 |
| M0963 | D144:28 S56 | ace.m1.inst935.choose42/av | R24 | "success"; exit=42 | cd56041f110610845db4b693591f98e4462bb8dd73f1357f720f6645e9b91848 | 6bd2334053cd2c5a6666e1853d397e890367e37ac108e595206c6da464837a90 | 原511 |
| M0964 | D144:47 S56 | ace.m1.inst935.choose0/ac-native | R37 | "success"; exit=0 | bb4bc907a93a395636cea983cf8f4afffa728ae0552b47106279a4c9958afc76 | bb688f2226614c9e6fd207d1a85cc6a16302c74158ae1561fb4a267bda307e77 | 原511 |
| M0965 | D144:47 S56 | ace.m1.inst935.choose0/av | R24 | "success"; exit=0 | bb4bc907a93a395636cea983cf8f4afffa728ae0552b47106279a4c9958afc76 | 151c020e0f1fe52c49e39373aed0557baf0cfd5929eec72b7f5be4a0e3d42756 | 原511 |
| M0966 | D144:66 S56 | ace.m1.inst935.expected-type/ac-native | R37 | "success"; exit=42 | 74ce851db2e482b56c1f601931de9b33f5d573643b555dab5ef937f65b317e2c | 6666cb2555fe8a8053e516eae4628be2e4910aa29709e4f9a40e26db347df4a9 | 原511 |
| M0967 | D144:66 S56 | ace.m1.inst935.expected-type/av | R24 | "success"; exit=42 | 74ce851db2e482b56c1f601931de9b33f5d573643b555dab5ef937f65b317e2c | 6bd2334053cd2c5a6666e1853d397e890367e37ac108e595206c6da464837a90 | 原511 |
| M0968 | D144:85 S56 | ace.m1.inst935.cross-wrap/ac-native | R37 | "success"; exit=40 | aa81a7cb1d984f9f1458b934703cef76b472d8ce913a128323aaebc947c91b18 | 8d4e2b1e0ece1d319b07c3ab97325e2bc42e53d048037550f39a85e6c360aaaa | 原511 |
| M0969 | D144:85 S56 | ace.m1.inst935.cross-wrap/av | R24 | "success"; exit=40 | aa81a7cb1d984f9f1458b934703cef76b472d8ce913a128323aaebc947c91b18 | 340d1abee25aae37391845bda6256ee8f73bd00b135e3366ea13b12eef921c73 | 原511 |
| M0970 | D144:104 S56 | ace.m1.inst935.recursion/ac-native | R37 | "success"; exit=40 | 04cc7f382be4ca28dce84730b3235d9af520beb5cde5e05a729bca6a095b554c | 8d4e2b1e0ece1d319b07c3ab97325e2bc42e53d048037550f39a85e6c360aaaa | 原511 |
| M0971 | D144:104 S56 | ace.m1.inst935.recursion/av | R24 | "success"; exit=40 | 04cc7f382be4ca28dce84730b3235d9af520beb5cde5e05a729bca6a095b554c | 340d1abee25aae37391845bda6256ee8f73bd00b135e3366ea13b12eef921c73 | 原511 |
| M0972 | D144:123 S56 | ace.m1.inst935.neg-hetero/ac-native | R37 | {"stage":"resolve","code":"type-mismatch"} | 8994694e7638b06f944b49b97f8f90b92f11c3dbfd87ffb446e5c716196e603c | 7304768adffcdf3df9edeb7d1fde243f2c60d3f9c4ecb4f67e1e526910f190dc | 原511 |
| M0973 | D144:123 S56 | ace.m1.inst935.neg-hetero/av | R24 | {"stage":"parse","code":"type-mismatch"} | 8994694e7638b06f944b49b97f8f90b92f11c3dbfd87ffb446e5c716196e603c | 8ea0227a7f46388443c92d2fc8244303b7600559216feb8619de278e2daebd6e | 原511 |
| M0974 | D145:25 S57 | ace.m1.sq934.leaf42/a2c-emit | R12 | "success"; artifacts=4 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | 037b1cd9280933619f66388fbe9810a3a4ee8d999c0f407461786b5eb6c6b175 | 原511 |
| M0975 | D145:25 S57 | ace.m1.sq934.leaf42/a2r-emit | R15 | "success"; artifacts=4 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | d95f3ed638ddb22ee29dab160dea4378cb476fc566db99b1ab2fed3e7a5bbdc6 | 原511 |
| M0976 | D145:25 S57 | ace.m1.sq934.leaf42/ac-native | R37 | "success"; exit=42 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M0977 | D145:25 S57 | ace.m1.sq934.leaf42/av | R24 | "success"; exit=42 | 7d7bff9c0fc4f821154ac1cbdb4aedb065e2f0d2e59ac5aec9ad8a75d7aff08b | 96f7c7bf3b73335dc523a423cd08e0821a9fdaee6e52e0207f8fd83d971e1d64 | 原511 |
| M0978 | D145:60 S57 | ace.m1.sq934.leaf43/ac-native | R37 | "success"; exit=43 | 3d45fe7ac0fd3b41e6aeea7ff0a8af6ba6121dd380eebde70fda07a5ab5c60f8 | 40e661ed44486beb3133b3a945a0a2f6e5b114f30b12b5949aa5c880ca13177c | 原511 |
| M0979 | D145:60 S57 | ace.m1.sq934.leaf43/av | R24 | "success"; exit=43 | 3d45fe7ac0fd3b41e6aeea7ff0a8af6ba6121dd380eebde70fda07a5ab5c60f8 | 82ab2f61690de7d6d2717373f7c8945027c37cc16fcf3e4579884614093776c3 | 原511 |
| M0980 | D145:79 S57 | ace.m1.sq934.named7/ac-native | R37 | "success"; exit=7 | d509360af6129e2438d693d44dce5259e1618e1e24d6fbad1f008f1f0eb24b9d | c4489e780690c26bc6b927c76a862653858929b34923888c0a2a867cc690df31 | 原511 |
| M0981 | D145:79 S57 | ace.m1.sq934.named7/av | R24 | "success"; exit=7 | d509360af6129e2438d693d44dce5259e1618e1e24d6fbad1f008f1f0eb24b9d | 822145fafe319fffb12dedec6bd0d3024d01c13e8d4d9d3c049a77b43150b160 | 原511 |
| M0982 | D145:98 S57 | ace.m1.sq934.trace-ab7/ac-native | R37 | "success"; exit=7; stderr#f9039ad27ef4 | 1e3db4c9677a9637cbb2b402480de50f2710e69d5c7cc4ffd8b315b9473be606 | 5ba60598ce8cf0121068807fe492a7d089794bc9e6fd68d4c6d9803aa6645441 | 原511 |
| M0983 | D145:98 S57 | ace.m1.sq934.trace-ab7/av | R24 | "success"; exit=7; trace#f9039ad27ef4 | 1e3db4c9677a9637cbb2b402480de50f2710e69d5c7cc4ffd8b315b9473be606 | 8450697a83df0be97c39f8e02509928098a557c21e06b1c516a93218a95087c2 | 原511 |
| M0984 | D145:117 S57 | ace.m1.sq934.neg-bool-plus/ac-native | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | 8a5d5336e01d7cad1edb1dcef4fb019319fae16b9aa9f0723af7b543df84b6da | d60c06a072006718eeb66ce59c3ddaaf277aa7493141094499c55045c96c1727 | 原511 |
| M0985 | D145:117 S57 | ace.m1.sq934.neg-bool-plus/av | R24 | {"stage":"parse","code":"source.type-mismatch"} | 8a5d5336e01d7cad1edb1dcef4fb019319fae16b9aa9f0723af7b543df84b6da | b17cc20c638a0f0318f517f7793b35ddea9f957e0523601f0fc9c3eda5947854 | 原511 |
| M0986 | D145:136 S57 | ace.m1.sq934.neg-not-int/ac-native | R37 | {"stage":"typecheck","code":"source.type-mismatch"} | f8fc48643a4f74cbcd04d1251007a560b2e9866f864bcb2885d5a95f87662817 | d60c06a072006718eeb66ce59c3ddaaf277aa7493141094499c55045c96c1727 | 原511 |
| M0987 | D145:136 S57 | ace.m1.sq934.neg-not-int/av | R24 | {"stage":"parse","code":"source.type-mismatch"} | f8fc48643a4f74cbcd04d1251007a560b2e9866f864bcb2885d5a95f87662817 | b17cc20c638a0f0318f517f7793b35ddea9f957e0523601f0fc9c3eda5947854 | 原511 |
| M0988 | D146:13 S58 | ace.m1.source-syntax-927.core/parse | R42 | "success"; artifacts=1 | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 24cf63259719a04f9a2aa74d11901a4d42403d0f3b141d9a0b9abe39cb1ee469 | 扩展/其他现用 |
| M0989 | D146:29 S58 | ace.m1.source-syntax-927.scalar/parse | R42 | "success"; artifacts=1 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 7993d0277bb453d9e4d083e03c966f3801fc50fdb4162d2232eef79ab48d8a83 | 扩展/其他现用 |
| M0990 | D146:45 S58 | ace.m1.source-syntax-927.text/parse | R42 | "success"; artifacts=1 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 13b60da1ae2002f1cceeb2c4bdedb2b8e78d77c046b9f40de1c578f25b7161d7 | 扩展/其他现用 |
| M0991 | D146:61 S58 | ace.m1.source-syntax-927.record/parse | R42 | "success"; artifacts=1 | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | 33946b679f13156efb1eadc4d848923ddd68f2dbd2664e088b8d102d5b279e44 | 扩展/其他现用 |
| M0992 | D146:77 S58 | ace.m1.source-syntax-927.text_record/parse | R42 | "success"; artifacts=1 | 145e3179e2fe81d0ea80af7739e9fbbff4603f63d7287cd1bda67735bbd4af7a | b7c655ce717e1a2e7d8f7f73bf32663bf0849a6d596b66a2a0b62d4f11ed770c | 扩展/其他现用 |
| M0993 | D146:93 S58 | ace.m1.source-syntax-927.entry_unit/parse | R42 | "success"; artifacts=1 | f89e2b6d31712d95c21a188ea2eaed2048cefd1c2bc1df7b94905e876fe3dbf0 | 4bd0987387429b17deba9fec4d9e481087225f90bd4916c5d6bdc4a37187ea5e | 扩展/其他现用 |
| M0994 | D146:109 S58 | ace.m1.source-syntax-927.host_console/parse | R42 | "success"; artifacts=1 | 3e67fdccb44be32838ad75b2e61f283bed406055bbfa672d4146784b2d7f322c | 6f1d7986ad7e0e4836cdc484fb89c77ec02913526908b525f31d4a9078f6dcb4 | 扩展/其他现用 |
| M0995 | D147:15 S29 | ace.m1.t01/ac-native | R37 | "success"; exit=38 | e58f7c63892c43ad128e02569c28cbcc0b0f251ee3dd7e5999a7f8204b402c79 | 84238ded21a75aae55d3d1ccab8de4e4d05110351b854a75e8caa327af4b9308 | 原511 |
| M0996 | D147:15 S29 | ace.m1.t01/av | R29 | "success"; exit=38 | e58f7c63892c43ad128e02569c28cbcc0b0f251ee3dd7e5999a7f8204b402c79 | c5051916fe059b97a20bac61505006d100c2f05c007b9c28c77fa50f0949bf8f | 原511 |
| M0997 | D147:34 S29 | ace.m1.t02/ac-native | R37 | "success"; exit=27 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 443e1ef8935a82f72e026730c3ae5fc7f09ca1464c83d1df08541c431959457e | 原511 |
| M0998 | D147:34 S29 | ace.m1.t02/av | R29 | "success"; exit=27 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 2d38c8cd25cf870c480ea452b8b5524abdb5eb91f45b6077eb1b438ae3a393bc | 原511 |
| M0999 | D147:61 S29 | ace.m1.t03/ac-native | R37 | "success"; exit=23 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | e1d614346b9412d30818d8ad723ba6e366fd16b3e4457ee5f0b92158fdd4f8f9 | 原511 |
| M1000 | D147:77 S29 | ace.m1.t04/ac-native | R37 | "success"; exit=128512 | 2935380215cf39547566d92c8ae51586cc3497427f016b8393a82b9508fa1b19 | b3f65d54ad6656a0b082104fc9db91c2e03d12b67679f6a12996a6c9ed2a3ebf | 原511 |
| M1001 | D147:77 S29 | ace.m1.t04/av | R29 | "success"; exit=128512 | 2935380215cf39547566d92c8ae51586cc3497427f016b8393a82b9508fa1b19 | 9d4a3df57e27ef4a80755781dfd601645fc66680e79d7365b2e6fad259603087 | 原511 |
| M1002 | D147:96 S29 | ace.m1.t06/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | ea10e8f81483653e87517702b06693102f1705db235ca1261cf7f12a81222bb9 | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M1003 | D147:96 S29 | ace.m1.t06/av | R29 | {"stage":"parse","code":"verify.borrow-escape"} | ea10e8f81483653e87517702b06693102f1705db235ca1261cf7f12a81222bb9 | 39ecba7e061929e61f8c7f9759427f1ba0f9cdb4244ed1612c325426002f4408 | 原511 |
| M1004 | D147:115 S29 | ace.m1.t07/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | c0bbf1cb39be98b262013e094f7c557f0de5d1676e5b7426b470517563542c05 | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M1005 | D147:115 S29 | ace.m1.t07/av | R29 | {"stage":"parse","code":"text.invalid-scalar"} | c0bbf1cb39be98b262013e094f7c557f0de5d1676e5b7426b470517563542c05 | 888486c68f0e6e9559171fe82f28068c972f627b3f648ac1eb9f01408372fb5a | 原511 |
| M1006 | D147:134 S29 | ace.m1.t08/ac-native | R37 | "success"; exit=70 | c6406d69b553bb29c61b0c9805f587f7a7656e46e349e17eeaa94ca66ca4ecf3 | c0c2f064d8ac40de0a24d7bfa0f30f3cca69a9d8a348eba991ed6748070ec3e6 | 原511 |
| M1007 | D147:134 S29 | ace.m1.t08/autovm | R05 | "success"; result={"format":"legacy-display","text":"0"} | c6406d69b553bb29c61b0c9805f587f7a7656e46e349e17eeaa94ca66ca4ecf3 | f83c147653bfd134d348bfc09d3ea2abf41bd00420febb856979bf7efc03398d | 扩展/其他现用 |
| M1008 | D147:134 S29 | ace.m1.t08/av | R29 | {"stage":"run","code":"av.trap.str-index"} | c6406d69b553bb29c61b0c9805f587f7a7656e46e349e17eeaa94ca66ca4ecf3 | cd9bedcc4ebb14a70eff9aa7c881f0f02fda4635b76f67aab1b7584a0e5738d0 | 原511 |
| M1009 | D147:155 S29 | ace.m1.r01/ac-native | R37 | "success"; exit=19 | 9126803ce99c46d12066c468b56189704386f5c77acbcf8a564acb76d8e3fdc6 | 45f0ac470ac18ca0f9897542a37cab983f3a725b9dff0178b579ae2467769d65 | 原511 |
| M1010 | D147:155 S29 | ace.m1.r01/av | R26 | "success"; exit=19 | 9126803ce99c46d12066c468b56189704386f5c77acbcf8a564acb76d8e3fdc6 | 023041f47f7e5d7cc44c0f38965df29cd5e3336609401a174587faa41d382053 | 原511 |
| M1011 | D147:174 S29 | ace.m1.r02/ac-native | R37 | "success"; exit=9 | 51fe7459ffc8913137c2bd392909d7143b68c2a6aebd47b1066e34f7c16abca0 | e987d4192e1f54faad48b6e48fd00d527e8387925ae18d3c50397d75557f3381 | 原511 |
| M1012 | D147:174 S29 | ace.m1.r02/av | R26 | "success"; exit=9 | 51fe7459ffc8913137c2bd392909d7143b68c2a6aebd47b1066e34f7c16abca0 | b7e6bdbca4d3d4f24d58b43d214baff5467b05b0a32f1873f1a441161e33329d | 原511 |
| M1013 | D147:193 S29 | ace.m1.r03/ac-native | R37 | "success"; exit=23 | 10630e3d095fcd27adc5471137ca6b28e4d36a45ce63d53f7262bf6d8ed5296b | e1d614346b9412d30818d8ad723ba6e366fd16b3e4457ee5f0b92158fdd4f8f9 | 原511 |
| M1014 | D147:193 S29 | ace.m1.r03/av | R26 | "success"; exit=23 | 10630e3d095fcd27adc5471137ca6b28e4d36a45ce63d53f7262bf6d8ed5296b | 99f1f97260fef4718af4a9076d0e58323b30821386cacecaa512b247d3bf2664 | 原511 |
| M1015 | D147:212 S29 | ace.m1.r04/ac-native | R37 | "success"; exit=51 | 1382b4c3a35d443e5ebf21ece67d4a6c9b017e443eec9346b5e0ece12d3b7b59 | ddf1ed2f47ed2a527da4f2ec3a5b4169e55e412a1937ba38a987b60d01707e31 | 原511 |
| M1016 | D147:212 S29 | ace.m1.r04/av | R29 | "success"; exit=51 | 1382b4c3a35d443e5ebf21ece67d4a6c9b017e443eec9346b5e0ece12d3b7b59 | 19b91ca571269ed754d3a79a805e755f60bf01b4a2b425e263822ce2f1df6ff9 | 原511 |
| M1017 | D147:231 S29 | ace.m1.r05/ac-native | R37 | "success"; exit=42 | c971be343e1a6ddcf4fc71474060077c30720d70cb6ec4f81855167b9ef71f4b | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M1018 | D147:231 S29 | ace.m1.r05/av | R29 | "success"; exit=42 | c971be343e1a6ddcf4fc71474060077c30720d70cb6ec4f81855167b9ef71f4b | 7304961f5149af4fe6883cf5b7501e4524cecefee2b53a25f8a9b274b9d243a2 | 原511 |
| M1019 | D147:250 S29 | ace.m1.r06/ac-native | R37 | "success"; exit=21 | 9851f21afb89ad0053d2578faf3a89843bd44bdaaea3f7e82fcca9412a058f1f | 30440a098675c6dfa46e702a1eab542bc86b92ab739c887acf6729035f74bf43 | 原511 |
| M1020 | D147:250 S29 | ace.m1.r06/av | R26 | "success"; exit=21 | 9851f21afb89ad0053d2578faf3a89843bd44bdaaea3f7e82fcca9412a058f1f | 2d9457bfc86d9a912a3e3939e78203adc4d6b5a1c6ffd333fd76d418435affe9 | 原511 |
| M1021 | D147:269 S29 | ace.m1.r07/ac-native | R37 | "success"; exit=42 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M1022 | D147:269 S29 | ace.m1.r07/av | R26 | "success"; exit=42 | 8f53aa607013917aeda83b599bb87d3d34155239bb2b2ba6ece3d6b1e3923b90 | 66f47c1fc4e6e0c12d76184371613dbd02929fd446770a1a0b040b9971ef2c18 | 原511 |
| M1023 | D147:288 S29 | ace.m1.r08/ac-native | R37 | "success"; exit=42 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M1024 | D147:288 S29 | ace.m1.r08/av | R26 | "success"; exit=42 | d01a5a76651ec8ee0f3bdf7d648724215f2c4de398e5bd1796a2c41c0f71baab | 66f47c1fc4e6e0c12d76184371613dbd02929fd446770a1a0b040b9971ef2c18 | 原511 |
| M1025 | D147:307 S29 | ace.m1.r09/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | 5c615ee205db29e23ca75db12dd2ac067bdf2056ba558ed2acda13c50164ad8b | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M1026 | D147:307 S29 | ace.m1.r09/av | R29 | {"stage":"parse","code":"source.type-mismatch"} | 5c615ee205db29e23ca75db12dd2ac067bdf2056ba558ed2acda13c50164ad8b | d1b50ad164394062b766c1e45521b4f439c8b6950934f3e99aaec3ba97c7ba2c | 原511 |
| M1027 | D147:326 S29 | ace.m1.r10/ac-native | R37 | {"stage":"parse","code":"mn.unsupported.frontend"} | de19cbac46fe4b4c8e21a3c8442efb5ad2746598f7435ac6818ff86fdf29c7fc | c171d8d1fc61e3bf977d069c3a94e57e151574d97db02867f6520f9ddfb4e752 | 原511 |
| M1028 | D147:326 S29 | ace.m1.r10/av | R29 | {"stage":"parse","code":"verify.use-after-move"} | de19cbac46fe4b4c8e21a3c8442efb5ad2746598f7435ac6818ff86fdf29c7fc | e886c33cd86c0283804bf6b325c15310055f826f021491abd071c3773c58b6fd | 原511 |
| M1029 | D148:10 S59 | ace.m1.t03.value/ac-native | R37 | "success"; exit=23 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | e1d614346b9412d30818d8ad723ba6e366fd16b3e4457ee5f0b92158fdd4f8f9 | 原511 |
| M1030 | D148:10 S59 | ace.m1.t03.value/av | R29 | "success"; exit=23 | 938b270163bc95e0b2a124686a0e43bf4ab8f0bf9c678696f6f69c7bd120cad1 | f4818b3a773b66512f257a43ea42d17e4fb08e28dc2f24a302dc6349a1ad49e2 | 原511 |
| M1031 | D148:29 S59 | ace.m1.t05.value/ac-native | R37 | "success"; exit=42 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | 16236030cec8e126a4d08c7eaa50f8f829f8e18f06a2de105907ffe535b7d0ce | 原511 |
| M1032 | D148:29 S59 | ace.m1.t05.value/av | R29 | "success"; exit=42 | 5ef6d95944286b8c2fb45056f7ce81870661a998150b460be6005fdcf63efaa7 | 7304961f5149af4fe6883cf5b7501e4524cecefee2b53a25f8a9b274b9d243a2 | 原511 |
| M1033 | D149:5 S60 | ace.m1.w01-mn891/mn-w01-typed | R38 | "success"; typed={"kind":"i64","bits":"8000000000000000"} | a0c2c044f596099921b828806cd6cd75afa26d5f474f7af364827c508b04d80a | d0e77dbccc20f9e72f5765625cd47dad20ee466466bc0d3390a5fd9035930ac3 | 原511 |
| M1034 | D149:21 S60 | ace.m1.w02-mn891/mn-w02-typed | R38 | "success"; typed={"kind":"u64","bits":"ffffffffffffffff"} | c29782267d0861c7d8e825b9d0571c8c1855fedfa9b5ace53c20d6385b9a3d13 | d0c10261a5841280dc0606d92d0b95e2d097435284eee6d247a32f7adc6c2735 | 原511 |
| M1035 | D149:37 S60 | ace.m1.w03-mn891/mn-w03-typed | R38 | "success"; typed={"kind":"f64","bits":"8000000000000000"} | 2051a3f7bea760946463a22c7a0435599a6325a33fa380bc8a7ae2d44ad020e2 | f4f199d72d25aa745bb131064a4739f7f25e7bfde839a591d155fc309154f62e | 原511 |
| M1036 | D149:53 S60 | ace.m1.w04-mn891/mn-w04-typed | R38 | "success"; typed={"kind":"f64","bits":"fff80000deadbeef"} | 96304f57f0d5fdd6a77dcb30f17f687fd66e8211874ae1b576c729841980c0ce | 2f18e9995f0f7a95b01f66d46404d26299765af4ec08c7351ea762d6bbc0ec6b | 原511 |
| M1037 | D149:69 S60 | ace.m1.w05-mn891/mn-w05-typed | R38 | "success"; typed={"kind":"f32","bits":"7f80ad00"} | cf6d215ec3b1f413cde4368c1aa07d4bc780eaa76d4e01fea4276e23f7308d9d | 216e92dacdabc534a47f66fc23c5fdf3ed8e65f36bcac1b11882dbc58bf3815e | 原511 |
| M1038 | D149:85 S60 | ace.m1.t891-value70/mn-typed-value70 | R38 | "success"; typed={"kind":"i32","bits":"46"} | 1fe353cb2b775894b0b906bb8e7a1238689e780b17bdb21876359135840bccb2 | 521895122492530350e109cbf408e7cc46902361747bfc1f182db32d36bd8161 | 原511 |
| M1039 | D149:101 S60 | ace.m1.t891-div-trap/mn-typed-div-zero | R38 | {"stage":"run","code":"mn.typed.trap"} | eea111e93761c20328bcde3e23a2e040f7dbe029299a749b4d71a9e1b055ded5 | fe08214af10855e9a499e85ec2923622b31d3a5c73edcf23b96625592ef78cb6 | 原511 |
| M1040 | D149:117 S60 | ace.m1.t891-ident-i64/mn-typed-ident-i64 | R38 | "success"; typed={"kind":"i64","bits":"0000000100000001"} | dd8f7f809776009bbead436a7852be0f72c035d8142f149cb5fe9c18041414e3 | 59185fe6a30ca02d6b3520f1a43ebf13c79d1c5db2433ad2851bf8b4d6e0d07a | 原511 |
| M1041 | D149:133 S60 | ace.m1.t891-mix-select/mn-typed-mix-select | R38 | "success"; typed={"kind":"u64","bits":"ffffffffffffffff"} | afa3c2b1f3a869a758e1c3f8e1ff0eedcf695970cdb793c0e75e3f4cd9af1dd5 | e89858c7fabee7f8f54848772c31b35c08f04e0d4312e7a011c175bf8ac0ed24 | 原511 |
| M1042 | D149:149 S60 | ace.m1.t891-record-field/mn-typed-record-field | R38 | "success"; typed={"kind":"f64","bits":"fff80000deadbeef"} | f571f2b505bf3aee59cc334c8e99cf0232b6345129d2d159789fce116d4cf3a1 | 2d41a16e0f841b5f7e736cb710c8451d61d7a76b6367892e717a90ee5565962b | 原511 |
| M1043 | D149:165 S60 | ace.m1.t891-call-forward/mn-typed-call-forward | R38 | "success"; typed={"kind":"i64","bits":"8000000000000000"} | 026275b649a0e40624abc6194b5759e5918201ac5bd7a087ab5873139b0f3e0b | 7dd5c06b9935d915f9f9b51c462bcec6f88f905f484ed382435cfb0de3dd2ee2 | 原511 |
| M1044 | D149:181 S60 | ace.m1.t891-f32-branch-subnormal/mn-typed-f32-branch | R38 | "success"; typed={"kind":"f32","bits":"00000001"} | a9dc1fa5cd81cde7683ee460cdbb25d0b86a38cd948abf6502760b314471cdc2 | eb4347582952918c5808d751eddd759976db2564b82a9b9e830b41fca13a9b29 | 原511 |
| M1045 | D150:20 S61 | ace.m1.unified-source-929.parse-core/parse-core | R42 | "success"; artifacts=1 | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | 5ef652073d999dda4c92d5612d2e1009040f575379e9c02f1093a900af51dccb | 原511 |
| M1046 | D150:35 S61 | ace.m1.unified-source-929.parse-scalar/parse-scalar | R42 | "success"; artifacts=1 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | e30d02290e74ee33709b02b9d8f8a7281121778a35112bdc4c0c5e1d6d15cb2a | 原511 |
| M1047 | D150:50 S61 | ace.m1.unified-source-929.parse-text/parse-text | R42 | "success"; artifacts=1 | 2e3c1f3775e0afba298e2549d6c383f7640ee05738b618d8a2e065089bd398b7 | 3bba3fd322b58fdaba7065a5310ed86a4a63f386cd4dc8e0a377e7fbc961fecb | 原511 |
| M1048 | D150:65 S61 | ace.m1.unified-source-929.parse-record/parse-record | R42 | "success"; artifacts=1 | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | d7321561f02ad53d42060818ee937c8f87668e58bd5b4254e1ced687f2f09ee4 | 原511 |
| M1049 | D150:80 S61 | ace.m1.unified-source-929.parse-text-record/parse-text-record | R42 | "success"; artifacts=1 | 145e3179e2fe81d0ea80af7739e9fbbff4603f63d7287cd1bda67735bbd4af7a | d69ebbb9667bbe3d70ca7154dec704fba61f7d6cefecf3e9c4ecc6a39b2d3d43 | 原511 |
| M1050 | D150:95 S61 | ace.m1.unified-source-929.parse-entry-unit/parse-entry-unit | R42 | "success"; artifacts=1 | f89e2b6d31712d95c21a188ea2eaed2048cefd1c2bc1df7b94905e876fe3dbf0 | d1f34984edf7ecf786524778aec43d72afa803976684d0f6b940a07cca85ad12 | 原511 |
| M1051 | D150:110 S61 | ace.m1.unified-source-929.parse-host-console/parse-host-console | R42 | "success"; artifacts=1 | 3e67fdccb44be32838ad75b2e61f283bed406055bbfa672d4146784b2d7f322c | 5ef737bd70cd74ba7e84da44dae3ad1d090d2e8e7e0aa59d679b8f3711de903d | 原511 |
| M1052 | D150:125 S61 | ace.m1.unified-source-929.parse-closures/parse-closures | R42 | "success"; artifacts=1 | 99722d41944eaac75a88f55cb5f0e9c7c42e2ddc126c7429ba8388fce499213e | 0080d7081605ab6971536f5986cb27e61e9b1fdefd3ba27046770362e02cdcb5 | 原511 |
| M1053 | D150:140 S61 | ace.m1.unified-source-929.parse-collections/parse-collections | R42 | "success"; artifacts=1 | a6a91b8e8313a48e1a21aaf79301aed6b360f4485d70c78d64d5d9b9fa124597 | 8c329698deb9b843075686b65d492dffaa5e6bcf7712615ffc6cd24f331e25a9 | 原511 |
| M1054 | D150:156 S61 | ace.m1.unified-source-929.profile-gate-negative/parse-record-under-scalar-grammar-refused | R42 | {"stage":"parse","code":"syntax.parse"} | 1fe9728c4469fde9684de9779f8efc1013fe7f355d82d866e8162345bf86555b | f482b4fca689727f17dcbb87f05b6508c2b4f1163b8c6f0883fff84a0500b836 | 原511 |
| M1055 | D150:172 S61 | ace.m1.unified-source-929.scalar-run/av-scalar-fresh-run | R24 | "success"; exit=14 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | 334ac01477636c968bbeb11b841760d5e9713c48be4a16daeb6e49d8a6ef9a54 | 原511 |
| M1056 | D151:10 S62 | autoabi.leg-a/leg-a | R07 | "success"; probe=5 | 5f8d3b199527eb6367aa045e77a89c4b4e62e2bec668f5891672e7fcd663763f | 78511917383ba7474bfd9da1a15bb577f669cfb7b456d9eda7d1cf87fb031a01 | 扩展/其他现用 |
| M1057 | D151:45 S62 | autoabi.leg-b/leg-b | R07 | "success"; probe=2 | 5f8d3b199527eb6367aa045e77a89c4b4e62e2bec668f5891672e7fcd663763f | e5dd23f8b92248b3407bbb73cc69ec0f2030f4dbb236545cb8ae9d324e0c284e | 扩展/其他现用 |
| M1058 | D151:68 S62 | autoabi.leg-c/leg-c | R07 | "success"; probe=2 | 5f8d3b199527eb6367aa045e77a89c4b4e62e2bec668f5891672e7fcd663763f | a21032f731c8d8f42b47f42a305ae5769605d18e4184160d39ed51c69f0eded3 | 扩展/其他现用 |
| M1059 | D151:93 S62 | autoabi.bad-sig/bad-sig | R07 | {"stage":"parse","code":"deck"}; probe=1 | 5f8d3b199527eb6367aa045e77a89c4b4e62e2bec668f5891672e7fcd663763f | a58099be8c33edd34776175e0fad40543d08ac5f04db04fa872f6682cd16fe8c | 扩展/其他现用 |
| M1060 | D152:2 S63 | av.loaded.b01/run-loaded-b01 | R28 | "success"; exit=14 | 92007b382ae9540074bd817e6d160b677d0a3d53d5c97609c15ca4ea2562d4d3 | dc3776b990b7eb82fd21ea9d402650ab4bf4c63ad9cdf024e318426c5bd3c3e3 | 扩展/其他现用 |
| M1061 | D153:2 S63 | av.loaded.b04/run-loaded-b04 | R28 | "success"; exit=42 | c466f5ac75a014c71012f1678e72ee3946eadfa27d1578c4a53418826f7b55c9 | f60ea518397078b17ab6afb6c49cd57f61f388925c42f28bb45f1bcc7665926a | 扩展/其他现用 |
| M1062 | D154:2 S63 | av.loaded.b09/run-loaded-b09 | R28 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 988af977b5bd7e81ab085b3558266344762d727e411bf44fb6a53065ecc22a27 | 扩展/其他现用 |
| M1063 | D155:2 S63 | av.loaded.b13/run-loaded-b13 | R28 | {"stage":"run","code":"av.trap.int-overflow"} | c972dd096fbb5f1bcb1ad63df00ee7debf714c1a6d4560302f634f2c111ce356 | ff44cdeddd479a6ea5b422d6f7f3c74762e1f18027c26aa4f20cfdf4767e1494 | 扩展/其他现用 |
| M1064 | D156:2 S63 | av.loaded.b14/run-loaded-b14 | R28 | {"stage":"run","code":"av.trap.div-by-zero"} | 40f88a40f6abab0060fa573f9a7909fd118ec0153f33c7c5c9fe6f1b12d0a8c0 | c5fd12828259835c34b045f4eab7cd320feb23886f820eccb8c9b959b16425ba | 扩展/其他现用 |
| M1065 | D157:5 S63 | av.loaded.first/run-loaded-first | R28 | "success"; exit=42 | 6dd0d84123051642918d3b23f1c09100f9b01ac404b6dbd16b01d0b3257e06ff | 6d9ed13634b33bba1722090ad87675ee1a750126ddbb3cdb89602a5f16f1c59b | 扩展/其他现用 |
| M1066 | D158:3 S63 | av.loaded.lifecycle/loaded-body-edit-run | R28 | "success"; exit=720 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 2108a6bba7a7bb5bc6e381dedcc1de14ba925b86a5da95783093189fc5baafbd | 扩展/其他现用 |
| M1067 | D158:3 S63 | av.loaded.lifecycle/loaded-edit-persist-across-workers | R28 | "success"; exit=720 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | cffd4f07c3417dc96d9e0b71bfe041bd028d2052c7b7e392f6b07abe994e45f8 | 扩展/其他现用 |
| M1068 | D158:3 S63 | av.loaded.lifecycle/loaded-hot-run | R28 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | 44b466ace0a34c17ea370992880206a65d8babb28fb798716f8f555f4aef0eee | 扩展/其他现用 |
| M1069 | D158:3 S63 | av.loaded.lifecycle/loaded-warm-run | R28 | "success"; exit=120 | ce4ad8beedf13796dbf6662b39f3534895825819b745f00e9d6a41f407983d5b | f979cd6e6630fe4c3b2fcb1f6082202f448f563a4febbc5f98380f3e943313bd | 扩展/其他现用 |
| M1070 | D159:8 S64 | av.compact.first/run-compact-first | R22 | "success"; exit=42 | 6dd0d84123051642918d3b23f1c09100f9b01ac404b6dbd16b01d0b3257e06ff | 5c8a8eb58a5a5fa3a6dd94497a443692bb6645aea407edb8faa10e3436b40fa3 | 扩展/其他现用 |
| M1071 | D160:7 S65 | av.owned.admission-partial-move/run-owned-rejected | R29 | {"stage":"parse","code":"verify.partial-move"} | 7ab2a60ec5b5d1e47f357277030cdfa5619de0b553ba7e6ac6c0213f864a7db5 | 7b0beb4d5ee17ade9e0deb34531f34148b14d1116d13822e65d27cb6560523c8 | 扩展/其他现用 |
| M1072 | D161:7 S65 | av.owned.borrowed-record/run-owned-borrowed | R29 | "success"; exit=15 | c089afd5b7c062f314a90edf0f2cf345f5ece1081a762bf757a180127c855fcb | 4b963560751019628552ca9768673a981cf4d89a5feaf8c25cb481d6cd87ef70 | 扩展/其他现用 |
| M1073 | D162:8 S65 | av.owned.first/run-owned-fixture | R29 | "success" | eae1ccbaba03e560ac55fc72433bc14f35e7fc2bc0d5c632b4e9795cf2f9a470 | a5173bc4e35ef530079a6da093192ff67e882f08a1d4fe616facfe63337977e6 | 扩展/其他现用 |
| M1074 | D163:7 S65 | av.owned.first-twin/run-owned-source | R29 | "success"; exit=404 | 3b8116e2e594def3d164546a8716246aff121aaf02a4f6738d82dea5be79fc3b | e7bffa5e4206ea42d0155a674e89911cebe9f84859d8361818bd9ffd030c6e7e | 扩展/其他现用 |
| M1075 | D164:5 S65 | av.owned.mut-record-writeback/run-owned-mut-writeback | R29 | "success"; exit=808 | 8b3f0ca09e708ad442ea2195d391fa9c9dfaf6713de875279fa288c8e8f21496 | a7e44b5b82eb7ddb724aa9fecf8126b904dc8f21cc78402267c77500e40ebb5e | 扩展/其他现用 |
| M1076 | D165:8 S66 | av.collections.first/run-collections-fixture | R20 | "success" | 94f38c2179596153a861fd589806f69e897fe33d3d6cb22a1436f8541bb1a7de | 978bc6233b1fc24b66a8aad859d30d7a694c0329cdd3dc28efff65a5ce52b367 | 扩展/其他现用 |
| M1077 | D166:2 S67 | av.closures.first/run-closures-fixture | R18 | "success" | 94f38c2179596153a861fd589806f69e897fe33d3d6cb22a1436f8541bb1a7de | 3effac600582c6e8dec36a534f367461447d16381531f84d88b5c5d245b2d799 | 扩展/其他现用 |
| M1078 | D167:8 S68 | av.dynamic.first/run-dynamic-fixture | R27 | "success" | 94f38c2179596153a861fd589806f69e897fe33d3d6cb22a1436f8541bb1a7de | 3d37043cd3b039e075ce906f9967af7c0342b82d98a32ede5f49c9db9b7e6c6d | 扩展/其他现用 |
| M1079 | D168:8 S68 | av.dynamic.take_probe/run-dynamic-take-probe | R27 | "success" | 94f38c2179596153a861fd589806f69e897fe33d3d6cb22a1436f8541bb1a7de | a80b877cf8125e42b2d8ea5701cb73ed3ae0d1819a7b819c3f672bb6ac6434ae | 扩展/其他现用 |
| M1080 | D169:11 S69 | av.source.session-overlay/overlay-edit-run | R24 | "success"; exit=9 | 5f2e203f5551269f8a16ace32cc1d7614bcbb2808e562cfda397fc327bfe2cf3 | 4320aeab62e28c7e1b60b4f8e6f3407564393e4b06b061736e4f906785a0e46b | 扩展/其他现用 |
| M1081 | D169:11 S69 | av.source.session-overlay/warm-run | R24 | "success"; exit=0 | 5f2e203f5551269f8a16ace32cc1d7614bcbb2808e562cfda397fc327bfe2cf3 | ae12068c447fcf400f39cf52e13255f39f240e599319f3c777b73ac562952ac1 | 扩展/其他现用 |
| M1082 | D170:8 S69 | av.source.first/run-first | R24 | "success"; exit=0 | 5f2e203f5551269f8a16ace32cc1d7614bcbb2808e562cfda397fc327bfe2cf3 | 8c368f299f64bbf97c81c691a398ebbe6a4203b14410393322dfac9125e6bcbe | 扩展/其他现用 |
| M1083 | D171:7 S69 | av.source.binop-args/run-binop-args | R24 | "success"; exit=0 | cf58bab15a1025ce62fd01e8a8d11d6d6e78b884df8f24f51fe524b748c29acb | eec7a0a9c2356fbec362d7e39b66215e959fe5cbba143d5e03d86bdeca7f2124 | 扩展/其他现用 |
| M1084 | D171:32 S69 | av.source.short-circuit-trap/run-trap-div-zero | R24 | {"stage":"run","message":{"contains":"av.trap.div-by-zero"}} | d727ebe7cb23602d20392db4ca93886dc10ecd9d5bdb081007253b487dd40cb8 | 0c70e2d4af53d241d27d346ef3de1efec1fd23f6a5e667c5a741b5f1337e2591 | 扩展/其他现用 |
| M1085 | D172:13 S69 | av.source.unsupported-print/run-unsupported-print | R24 | {"stage":"parse","code":"source.unsupported"} | 0eb16391f6c86b6f1d0c6cbb02ae588d86934df55458908f44a250669b47ef9e | 99e3436d96db46f0482fe7b7e3594b072734f0ec0633cedf2d2e29eaa8488e32 | 扩展/其他现用 |
| M1086 | D172:33 S69 | av.source.unsupported-list/run-unsupported-list | R24 | {"stage":"parse","code":"source.syntax"} | ca4ed3a9dfedadee030401cc8d727ed6aa170ba45dc140330e79bb87a0095795 | c5495fb56185a2381a2bda1264e72b4a847b1086d42f283768802ce0ed54b239 | 扩展/其他现用 |
| M1087 | D173:21 S70 | gov.d.arithmetic/av | R24 | "success"; exit=3; av_channel="value" | 0dc1db3f7c124e43c0e138525686ce4921bb60f5b920c7841b4772805980d0fa | 92c696edcf19ab9fc275675a30802118d6f52c4871001e9759d6cf7c9e5f9a6d | 扩展/其他现用 |
| M1088 | D173:21 S70 | gov.d.arithmetic/av-scalar-control | R24 | "success"; exit=3 | 0dc1db3f7c124e43c0e138525686ce4921bb60f5b920c7841b4772805980d0fa | ec4aba7d1bec0790c4b5dee5f6f59ab9b69f26a0c1b87ef3848e4eaf5727bde9 | 扩展/其他现用 |
| M1089 | D174:17 S70 | gov.d.group/av | R24 | "success"; exit=9; av_channel="value" | 75efe77c489ba3281b0ef3f37f299889efb73ddff8647110ed9e2ee1acca2236 | 6ea8b2e05a3fa50d6e59543ef232dba6f38da007dbfa3d1d399b2529938d2116 | 扩展/其他现用 |
| M1090 | D174:17 S70 | gov.d.group/av-scalar-control | R24 | "success"; exit=9 | 75efe77c489ba3281b0ef3f37f299889efb73ddff8647110ed9e2ee1acca2236 | 052d77043d99216171e59449452ae7580aef6e2adf61093dfb44a796e8993a52 | 扩展/其他现用 |
| M1091 | D175:17 S70 | gov.d.unary/av | R24 | "success"; exit=-6; av_channel="value" | ecada3511081daa01eef7a33f10d9f10e0d07676c3010aee07c4a47ab1434c1e | ca97f0285be6f5c1a4dd6119e03028ea8ef3482be28dd2e92bf89efd5d0511f8 | 扩展/其他现用 |
| M1092 | D175:17 S70 | gov.d.unary/av-scalar-control | R24 | "success"; exit=-6 | ecada3511081daa01eef7a33f10d9f10e0d07676c3010aee07c4a47ab1434c1e | d997d54cc828671df016c2492bbaff97e4d0cf3b8ffe5427763066f0ee7eee89 | 扩展/其他现用 |
| M1093 | D176:8 S71 | trans.basics.func/emit | R01 | "success"; artifacts=1 | 8c4cc00f31730a92af6e49b3396ccbe94d8709bf9a93ef01ff649c1a2eaf08d2 | 7ed0fb12d680e7e916a00258508461a50885a27192ac580e45dfde2bacdc12f5 | 扩展/其他现用 |
| M1094 | D176:24 S71 | trans.basics.doc-comments/emit | R01 | "success"; artifacts=1 | 543247c1d546a87141e441616a51c05a91c2b43af31b332f368e6a5c78bf402d | aeeae7e6b32eec4d0b6834a12c90ba9603e882b8fe008a282a7da466167b18ca | 扩展/其他现用 |
| M1095 | D176:40 S71 | trans.basics.struct/emit | R01 | "success"; artifacts=1 | 5c129a046eebafee6222dbeba8272b0db206e7af491ec51dd92a6ff87a776fa7 | 167863b741209c1cfb791c431616d81ddd81b43032518da6c2a166acf68a2524 | 扩展/其他现用 |
| M1096 | D176:56 S71 | trans.basics.for-range/emit | R01 | "success"; artifacts=1 | 4270d3644c4c445039587fee65d1ae6895337e987b6742b26134671e070c1723 | bb9751ecf47968f7a4f7f25cdb606fc8b53dc45198b328bfee268e3ca9749e0c | 扩展/其他现用 |
| M1097 | D176:72 S71 | trans.basics.arithmetic/emit | R01 | "success"; artifacts=1 | 25daf3e47d22ad545e51c352e5bb80c64b87274f21b4ba3b308661bcb6f0f97d | b829ffcf6fa2b55a63395e21aba8c580eb02c2412c67a3b32e0d6c44af77ff68 | 扩展/其他现用 |
| M1098 | D177:8 S71 | trans.cabi.static/emit | R01 | "success"; artifacts=1 | dc1e97ebf51600c4b711eee98f73b580a20d1b4511d80008d90c39fc7828ca9f | b132c9f9166cbcdf5bb366af35ef14bf5b9992a810f75b73e2b137140a645197 | 扩展/其他现用 |
| M1099 | D177:25 S71 | trans.cabi.dynamic/emit | R01 | "success"; artifacts=1 | fd3d7ec300e9b06f5bea77a5a49b950c20d23c185b4d23f5f9ab534040262194 | a9d0456030e5d6fb3bc05ac7327b52f6034a4b4602d42e81816ee31bb55b41e1 | 扩展/其他现用 |
| M1100 | D178:11 S71 | trans.project.multi-file/emit | R01 | "success"; artifacts=5 | d29b16f21cbeb481d1f395d7f4ff3398a5e0d8d7ffe2097a5f5780a0a88b2f72 | f3862229b83ae67bda029f1619fc8156bfb2413af6e686424103ae585e55e486 | 扩展/其他现用 |
| M1101 | D179:7 S71 | lang.basic.hello/rust | R01 | "success"; artifacts=1 | 69f96eae22f6fd138cab5f8b8c3d87ed823785105c6f70ff478000f8186cc202 | c742ca7a79b638d306158a22fd4ef53627c9447cf34543ed750894c7a0a906fe | 扩展/其他现用 |
| M1102 | D179:7 S71 | lang.basic.hello/vm | R05 | "success"; stdout#e55de9d2e278 | 69f96eae22f6fd138cab5f8b8c3d87ed823785105c6f70ff478000f8186cc202 | d253b977cdf1cb1f52798e924b3c22f569b1a2446378d9689c8f7bb5ab2ecd38 | 扩展/其他现用 |
| M1103 | D179:25 S71 | lang.basic.str-upper/rust | R01 | "success"; artifacts=1 | 87c0dc1312db3cab26c1c7755e9f5d1c670c2f9c8803547d69c9c75ec7120dc4 | 5276617a17bc6d62bc3388ca5d86d8bed43ba3ecf60479c7ade3ec057437d871 | 扩展/其他现用 |
| M1104 | D179:25 S71 | lang.basic.str-upper/vm | R05 | "success"; stdout#e481023bdd58 | 87c0dc1312db3cab26c1c7755e9f5d1c670c2f9c8803547d69c9c75ec7120dc4 | 148ab2aa53c843459a283d43301d5f62027ff6ca6911ea9cb60c89dad4dd7c63 | 扩展/其他现用 |
| M1105 | D179:43 S71 | lang.basic.eq-false/rust | R01 | "success"; artifacts=1 | 3cca38ad6cc3135ee4261cc6f428ac4cfd90603a966560c8f3973cf3f63fa24a | 5fc4bcad2551e26fe6016479275f629f928abf1a54464a35614db632d38d692a | 扩展/其他现用 |
| M1106 | D179:43 S71 | lang.basic.eq-false/vm | R05 | "success"; result={"format":"legacy-display","text":"false"} | 3cca38ad6cc3135ee4261cc6f428ac4cfd90603a966560c8f3973cf3f63fa24a | eacc845f44e5c4b020943251766963570467a159d75c3d863e7e6028c8690f9b | 扩展/其他现用 |
| M1107 | D179:61 S71 | lang.basic.arithmetic/vm | R05 | "success"; result={"format":"legacy-display","text":"3"} | a3bf3a7dabef25b578a87d59f2c742a3bcf4b6ced9aa9b69cd8c7a56beb86fb7 | 91ffb344ed9f95301b7f4779177e07ea33376bbc242494e39811e1cd9c7d2c92 | 扩展/其他现用 |
| M1108 | D179:75 S71 | lang.basic.uint/vm | R05 | "success"; result={"format":"legacy-display","text":"3u"} | ab99e2bd6f1aeae28fe6f9cb35d393a08bfc42cd24ee9cd6c5fc77cae48e984c | be0d124f34ac7304808c1009b33bd25400d3402bfdfb78d64768c8585f693bd0 | 扩展/其他现用 |
| M1109 | D179:89 S71 | lang.basic.byte/vm | R05 | "success"; result={"format":"legacy-display","text":"0xFF"} | 6960706b4320a9d90a18350021f05f4ea64326baea7acdc7652715575727132b | b126a3de1c29b37859c07a37f6e20053a986219785b04d84844c38b08caab3dc | 扩展/其他现用 |
| M1110 | D179:103 S71 | lang.basic.unary/vm | R05 | "success"; result={"format":"legacy-display","text":"-6"} | f9b724840fb8b83128e3c4ec735cf7445faf8ca98f65e5f277db903e56fd6175 | c2734c0647cab0b3dea8b39427e2a2cd347c1484a9c0d83401bfa4000382f3bb | 扩展/其他现用 |
| M1111 | D179:117 S71 | lang.basic.group/vm | R05 | "success"; result={"format":"legacy-display","text":"9"} | 22b75292ac6517786dd515f3e9f0ba93af14f8a72139021a192fcb80014d8393 | ba3f94eed2b16eda5f3281c665dbdc4a5ee24c3f87e00cfc69932367dd5e1f74 | 扩展/其他现用 |
| M1112 | D179:131 S71 | lang.basic.comp/vm | R05 | "success"; result={"format":"legacy-display","text":"true"} | f79b2fe28b4c816b947e157c51d720ed4d55d9d02694ed793ee2bdfc06333e91 | 122eaf9687b00e6fb1278c82bf9fda97ff9db70da21ef2bb2d4b1318859510a3 | 扩展/其他现用 |
| M1113 | D180:10 S71 | lang.script.json-is-null/vm | R05 | "success"; result={"format":"legacy-display","text":"true"} | 97011c85b8ff265a2e3ea9dccce7e1cb7d00a39e5e46aff55f8761cc485b3c48 | 122eaf9687b00e6fb1278c82bf9fda97ff9db70da21ef2bb2d4b1318859510a3 | 扩展/其他现用 |
| M1114 | D181:8 S71 | lang.shortcircuit.and-or/vm | R05 | "success"; stdout#93da39a32efa | c24b5af4b96aa6cff3876a0a92c1184985317803b298c03ec0756cdc05871c7f | cca3503bcbbdd9b8ca3fc517e54ae9c8fc9ba0059ae990b27e75e092b4f7b063 | 扩展/其他现用 |
| M1115 | D181:22 S71 | lang.shortcircuit.negative-index/vm | R05 | "success"; stdout#4a7648e0a8c5 | a1225e217fedac10bad6365c1adcebe2e7bf154861e8dc41d37a3f51fd69b3eb | ac50c0a3cfd5036d34b52872387ec6705b38df29377cc19b1ea4b75fe74da734 | 扩展/其他现用 |
| M1116 | D181:36 S71 | lang.error.out-of-bounds/vm | R05 | {"stage":"run","message":{"contains":"out of bounds"}} | ced8a32592422500cf052e53e87473f19db793c7b5ca24d3fb045a1f81336e2d | 3893fcda7128b35b4a362a4cfe93cdeec0d7de7e11b62ddcfc4ed75d6f081ec0 | 扩展/其他现用 |

</details>

<details>
<summary>L：313个legacy cell到332个原入口/M项的逐项映射</summary>

| ID | 原cell | 原target | binding独立oracle | 原入口所对应M项 | 原owner | 审计结论 |
|---|---|---|---|---|---|---|
| L001 | corpus:B01:a2r | legacy-A2R | {"kind":"value","expected":"14"} | M0569 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L002 | corpus:B01:ac | AC-native | {"kind":"value","expected":"14"} | M0570 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L003 | corpus:B01:av | AV | {"kind":"value","expected":"14"} | M0572 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L004 | corpus:B01:avm | legacy-AVM | {"kind":"value","expected":"14"} | M0571 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L005 | corpus:B02:a2r | legacy-A2R | {"kind":"value","expected":"-21"} | M0573 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L006 | corpus:B02:ac | AC-native | {"kind":"value","expected":"-21"} | M0574 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L007 | corpus:B02:av | AV | {"kind":"value","expected":"-21"} | M0576 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L008 | corpus:B02:avm | legacy-AVM | {"kind":"value","expected":"-21"} | M0575 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L009 | corpus:B03:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0577 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L010 | corpus:B03:ac | AC-native | {"kind":"value","expected":"42"} | M0578 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L011 | corpus:B03:av | AV | {"kind":"value","expected":"42"} | M0580 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L012 | corpus:B03:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0579 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L013 | corpus:B04:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0581 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L014 | corpus:B04:ac | AC-native | {"kind":"value","expected":"42"} | M0582 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L015 | corpus:B04:av | AV | {"kind":"value","expected":"42"} | M0584 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L016 | corpus:B04:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0583 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L017 | corpus:B05:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0585 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L018 | corpus:B05:ac | AC-native | {"kind":"value","expected":"42"} | M0586 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L019 | corpus:B05:av | AV | {"kind":"value","expected":"42"} | M0588 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L020 | corpus:B05:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0587 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L021 | corpus:B06:a2r | legacy-A2R | {"kind":"value","expected":"0"} | M0589 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L022 | corpus:B06:ac | AC-native | {"kind":"value","expected":"0"} | M0590 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L023 | corpus:B06:av | AV | {"kind":"value","expected":"0"} | M0592 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L024 | corpus:B06:avm | legacy-AVM | {"kind":"value","expected":"0"} | M0591 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L025 | corpus:B07:a2r | legacy-A2R | {"kind":"value","expected":"30"} | M0593 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L026 | corpus:B07:ac | AC-native | {"kind":"value","expected":"30"} | M0594 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L027 | corpus:B07:av | AV | {"kind":"value","expected":"30"} | M0596 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L028 | corpus:B07:avm | legacy-AVM | {"kind":"value","expected":"30"} | M0595 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L029 | corpus:B08:a2r | legacy-A2R | {"kind":"value","expected":"7"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L030 | corpus:B08:ac | AC-native | {"kind":"value","expected":"7"} | M0273 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L031 | corpus:B08:av | AV | {"kind":"value","expected":"7"} | M0274 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L032 | corpus:B09:a2r | legacy-A2R | {"kind":"value","expected":"120"} | M0597 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L033 | corpus:B09:ac | AC-native | {"kind":"value","expected":"120"} | M0598 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L034 | corpus:B09:av | AV | {"kind":"value","expected":"120"} | M0600 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L035 | corpus:B09:avm | legacy-AVM | {"kind":"value","expected":"120"} | M0599 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L036 | corpus:B10:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0601 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L037 | corpus:B10:ac | AC-native | {"kind":"value","expected":"42"} | M0602 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L038 | corpus:B10:av | AV | {"kind":"value","expected":"42"} | M0604 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L039 | corpus:B10:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0603 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L040 | corpus:B11:a2r | legacy-A2R | {"kind":"value","expected":"7"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L041 | corpus:B11:ac | AC-native | {"kind":"value","expected":"7"} | M0275 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L042 | corpus:B11:av | AV | {"kind":"value","expected":"7"} | M0276 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L043 | corpus:B12:a2r | legacy-A2R | {"kind":"value","expected":"-2"} | M0605 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L044 | corpus:B12:ac | AC-native | {"kind":"value","expected":"-2"} | M0606 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L045 | corpus:B12:av | AV | {"kind":"value","expected":"-2"} | M0608 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L046 | corpus:B12:avm | legacy-AVM | {"kind":"value","expected":"-2"} | M0607 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L047 | corpus:B13:a2r | legacy-A2R | {"kind":"trap","expected":"overflow"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L048 | corpus:B13:ac | AC-native | {"kind":"trap","expected":"overflow"} | M0609 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L049 | corpus:B13:av | AV | {"kind":"trap","expected":"overflow"} | M0610 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L050 | corpus:B14:a2r | legacy-A2R | {"kind":"trap","expected":"div-by-zero"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L051 | corpus:B14:ac | AC-native | {"kind":"trap","expected":"div-by-zero"} | M0611 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L052 | corpus:B14:av | AV | {"kind":"trap","expected":"div-by-zero"} | M0613 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L053 | corpus:B14:avm | legacy-AVM | {"kind":"trap","expected":"div-by-zero"} | M0612 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L054 | corpus:B15:a2r | legacy-A2R | {"kind":"trap","expected":"rem-overflow"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L055 | corpus:B15:ac | AC-native | {"kind":"trap","expected":"rem-overflow"} | M0614 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L056 | corpus:B15:av | AV | {"kind":"trap","expected":"rem-overflow"} | M0615 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L057 | corpus:B16:a2r | legacy-A2R | {"kind":"diagnostic","expected":"resolve"} | M0544,M0543 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L058 | corpus:B16:ac | AC-native | {"kind":"diagnostic","expected":"resolve"} | M0560 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L059 | corpus:B16:av | AV | {"kind":"diagnostic","expected":"resolve"} | M0562 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L060 | corpus:B16:avm | legacy-AVM | {"kind":"diagnostic","expected":"resolve"} | M0561 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L061 | corpus:B17:a2r | legacy-A2R | {"kind":"diagnostic","expected":"type"} | M0546,M0545 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L062 | corpus:B17:ac | AC-native | {"kind":"diagnostic","expected":"type"} | M0563,M0812 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L063 | corpus:B17:av | AV | {"kind":"diagnostic","expected":"type"} | M0564 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L064 | corpus:B18:a2r | legacy-A2R | {"kind":"diagnostic","expected":"assign-immutable"} | M0548,M0547 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L065 | corpus:B18:ac | AC-native | {"kind":"diagnostic","expected":"assign-immutable"} | M0565,M0813 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L066 | corpus:B18:av | AV | {"kind":"diagnostic","expected":"assign-immutable"} | M0566 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L067 | corpus:B18:avm | legacy-AVM | {"kind":"diagnostic","expected":"assign-immutable"} | M0689 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L068 | corpus:B19:a2r | legacy-A2R | {"kind":"diagnostic","expected":"missing-return"} | M0550,M0549 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L069 | corpus:B19:ac | AC-native | {"kind":"diagnostic","expected":"missing-return"} | M0567,M0814 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L070 | corpus:B19:av | AV | {"kind":"diagnostic","expected":"missing-return"} | M0568 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L071 | corpus:B20:a2r | legacy-A2R | {"kind":"trap","expected":"div-by-zero"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L072 | corpus:B20:ac | AC-native | {"kind":"trap","expected":"div-by-zero"} | M0616 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L073 | corpus:B20:av | AV | {"kind":"trap","expected":"div-by-zero"} | M0618 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L074 | corpus:B20:avm | legacy-AVM | {"kind":"trap","expected":"div-by-zero"} | M0617 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L075 | corpus:H01:av | AV | {"kind":"value","expected":"2100"} | M0874 | PLAN-866 AV resident | 保原件/原预算/原适用性；不把同源当执行 |
| L076 | corpus:H02:av | AV | {"kind":"value","expected":"2608"} | M0875 | PLAN-866 AV resident | 保原件/原预算/原适用性；不把同源当执行 |
| L077 | corpus:H03:av | AV | {"kind":"value","expected":"3070"} | M0876 | PLAN-866 AV resident | 保原件/原预算/原适用性；不把同源当执行 |
| L078 | corpus:K01:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0725 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L079 | corpus:K01:av | AV | {"kind":"value","expected":"42"} | M0637,M0638 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L080 | corpus:K01:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0726 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L081 | corpus:K02:av | AV | {"kind":"value","expected":"42"} | M0640 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L082 | corpus:K03:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0727 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L083 | corpus:K03:av | AV | {"kind":"value","expected":"42"} | M0641 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L084 | corpus:K03:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0728 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L085 | corpus:K04:a2r | legacy-A2R | {"kind":"diagnostic","expected":"borrow-escape"} | M0755 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L086 | corpus:K04:av | AV | {"kind":"diagnostic","expected":"borrow-escape"} | M0642 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L087 | corpus:K04:avm | legacy-AVM | {"kind":"diagnostic","expected":"borrow-escape"} | M0756 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L088 | corpus:K05:av | AV | {"kind":"value","expected":"42"} | M0643 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L089 | corpus:K06:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0729 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L090 | corpus:K06:av | AV | {"kind":"value","expected":"42"} | M0644 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L091 | corpus:K06:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0730 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L092 | corpus:L01:a2r | legacy-A2R | {"kind":"value","expected":"42"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L093 | corpus:L01:ac | AC-native | {"kind":"value","expected":"42"} | M0789 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L094 | corpus:L01:av | AV | {"kind":"value","expected":"42"} | M0645,M0645 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L095 | corpus:L02:a2r | legacy-A2R | {"kind":"value","expected":"42"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L096 | corpus:L02:ac | AC-native | {"kind":"value","expected":"42"} | M0790 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L097 | corpus:L02:av | AV | {"kind":"value","expected":"42"} | M0646,M0646 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L098 | corpus:L03:a2r | legacy-A2R | {"kind":"value","expected":"0"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L099 | corpus:L03:ac | AC-native | {"kind":"value","expected":"0"} | M0791 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L100 | corpus:L03:av | AV | {"kind":"value","expected":"0"} | M0647,M0647 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L101 | corpus:L04:a2r | legacy-A2R | {"kind":"value","expected":"42"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L102 | corpus:L04:ac | AC-native | {"kind":"value","expected":"42"} | M0792,M0831 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L103 | corpus:L04:av | AV | {"kind":"value","expected":"42"} | M0648,M0648 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L104 | corpus:L05:a2r | legacy-A2R | {"kind":"value","expected":"42"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L105 | corpus:L05:ac | AC-native | {"kind":"value","expected":"42"} | M0793 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L106 | corpus:L05:av | AV | {"kind":"value","expected":"42"} | M0649,M0649 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L107 | corpus:L06:a2r | legacy-A2R | {"kind":"trap","expected":"list-oob"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L108 | corpus:L06:ac | AC-native | {"kind":"trap","expected":"list-oob"} | M0794 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L109 | corpus:L06:av | AV | {"kind":"trap","expected":"list-oob"} | M0650,M0650 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L110 | corpus:L07:a2r | legacy-A2R | {"kind":"value","expected":"42"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L111 | corpus:L07:ac | AC-native | {"kind":"value","expected":"42"} | M0795 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L112 | corpus:L07:av | AV | {"kind":"value","expected":"42"} | M0651,M0651 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L113 | corpus:O01:av | AV | {"kind":"value","expected":"7"} | M0662 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L114 | corpus:O02:av | AV | {"kind":"value","expected":"0"} | M0663 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L115 | corpus:O03:av | AV | {"kind":"value","expected":"0"} | M0664 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L116 | corpus:O04:av | AV | {"kind":"value","expected":"0"} | M0665 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L117 | corpus:O05:av | AV | {"kind":"value","expected":"4"} | M0666 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L118 | corpus:O06:av | AV | {"kind":"value","expected":"0"} | M0667 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L119 | corpus:O07:av | AV | {"kind":"value","expected":"unit"} | M0668 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L120 | corpus:O08:av | AV | {"kind":"value","expected":"42"} | M0669 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L121 | corpus:O09:av | AV | {"kind":"value","expected":"unit"} | M0670 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L122 | corpus:O10:av | AV | {"kind":"trap","expected":"av.trap.div-by-zero"} | M0671 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L123 | corpus:O11:av | AV | {"kind":"diagnostic","expected":"source.console-argument"} | M0672 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L124 | corpus:O12:av | AV | {"kind":"diagnostic","expected":"host.unauthorized"} | M0673 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L125 | corpus:O13:av | AV | {"kind":"diagnostic","expected":"source.console-argument"} | M0674 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L126 | corpus:O14:av | AV | {"kind":"diagnostic","expected":"source.console-reserved"} | M0675 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L127 | corpus:O15:av | AV | {"kind":"value","expected":"unit"} | M0676 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L128 | corpus:R01:a2r | legacy-A2R | {"kind":"value","expected":"19"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L129 | corpus:R01:ac | AC-native | {"kind":"value","expected":"19"} | M0309,M1009 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L130 | corpus:R01:av | AV | {"kind":"value","expected":"19"} | M0310,M1010 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L131 | corpus:R02:a2r | legacy-A2R | {"kind":"value","expected":"9"} | M0731 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L132 | corpus:R02:ac | AC-native | {"kind":"value","expected":"9"} | M0311,M1011 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L133 | corpus:R02:av | AV | {"kind":"value","expected":"9"} | M0312,M1012 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L134 | corpus:R02:avm | legacy-AVM | {"kind":"value","expected":"9"} | M0732 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L135 | corpus:R03:a2r | legacy-A2R | {"kind":"value","expected":"23"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L136 | corpus:R03:ac | AC-native | {"kind":"value","expected":"23"} | M0313,M1013 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L137 | corpus:R03:av | AV | {"kind":"value","expected":"23"} | M0314,M1014 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L138 | corpus:R04:a2r | legacy-A2R | {"kind":"value","expected":"51"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L139 | corpus:R04:ac | AC-native | {"kind":"value","expected":"51"} | M0315,M1015 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L140 | corpus:R04:av | AV | {"kind":"value","expected":"51"} | M0316,M1016 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L141 | corpus:R05:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0733 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L142 | corpus:R05:ac | AC-native | {"kind":"value","expected":"42"} | M0317,M1017 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L143 | corpus:R05:av | AV | {"kind":"value","expected":"42"} | M0318,M1018 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L144 | corpus:R05:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0734 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L145 | corpus:R06:a2r | legacy-A2R | {"kind":"value","expected":"21"} | M0735 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L146 | corpus:R06:ac | AC-native | {"kind":"value","expected":"21"} | M0319,M1019 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L147 | corpus:R06:av | AV | {"kind":"value","expected":"21"} | M0320,M1020 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L148 | corpus:R06:avm | legacy-AVM | {"kind":"value","expected":"21"} | M0736 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L149 | corpus:R07:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0528,M0527 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L150 | corpus:R07:ac | AC-native | {"kind":"value","expected":"42"} | M1021 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L151 | corpus:R07:av | AV | {"kind":"value","expected":"42"} | M1022 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L152 | corpus:R08:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0737 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L153 | corpus:R08:ac | AC-native | {"kind":"value","expected":"42"} | M0773,M1023 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L154 | corpus:R08:av | AV | {"kind":"value","expected":"42"} | M0774,M1024 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L155 | corpus:R08:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0738 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L156 | corpus:R09:a2r | legacy-A2R | {"kind":"diagnostic","expected":"nominal-type"} | M0554,M0553 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L157 | corpus:R09:ac | AC-native | {"kind":"diagnostic","expected":"nominal-type"} | M0285,M1025 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L158 | corpus:R09:av | AV | {"kind":"diagnostic","expected":"nominal-type"} | M0286,M1026 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L159 | corpus:R10:a2r | legacy-A2R | {"kind":"diagnostic","expected":"use-after-move"} | M0556,M0555 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L160 | corpus:R10:ac | AC-native | {"kind":"diagnostic","expected":"use-after-move"} | M0287,M1027 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L161 | corpus:R10:av | AV | {"kind":"diagnostic","expected":"use-after-move"} | M0288,M1028 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L162 | corpus:T01:a2r | legacy-A2R | {"kind":"value","expected":"38"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L163 | corpus:T01:ac | AC-native | {"kind":"value","expected":"38"} | M0321,M0995 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L164 | corpus:T01:av | AV | {"kind":"value","expected":"38"} | M0322,M0996 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L165 | corpus:T02:a2r | legacy-A2R | {"kind":"value","expected":"27"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L166 | corpus:T02:ac | AC-native | {"kind":"value","expected":"27"} | M0323,M0997 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L167 | corpus:T02:av | AV | {"kind":"value","expected":"27"} | M0324,M0998 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L168 | corpus:T03:a2r | legacy-A2R | {"kind":"value","expected":"23"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L169 | corpus:T03:ac | AC-native | {"kind":"value","expected":"23"} | M1029,M0999 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L170 | corpus:T03:av | AV | {"kind":"value","expected":"23"} | M1030 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L171 | corpus:T04:a2r | legacy-A2R | {"kind":"value","expected":"128512"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L172 | corpus:T04:ac | AC-native | {"kind":"value","expected":"128512"} | M0325,M1000 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L173 | corpus:T04:av | AV | {"kind":"value","expected":"128512"} | M0326,M1001 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L174 | corpus:T05:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0739 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L175 | corpus:T05:ac | AC-native | {"kind":"value","expected":"42"} | M1031 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L176 | corpus:T05:av | AV | {"kind":"value","expected":"42"} | M1032 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L177 | corpus:T05:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0740 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L178 | corpus:T06:a2r | legacy-A2R | {"kind":"diagnostic","expected":"borrow-escape"} | M0552,M0551 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L179 | corpus:T06:ac | AC-native | {"kind":"diagnostic","expected":"borrow-escape"} | M0289,M1002 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L180 | corpus:T06:av | AV | {"kind":"diagnostic","expected":"borrow-escape"} | M0290,M1003 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L181 | corpus:T07:a2r | legacy-A2R | {"kind":"diagnostic","expected":"invalid-scalar"} | M0753 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L182 | corpus:T07:ac | AC-native | {"kind":"diagnostic","expected":"invalid-scalar"} | M0291,M1004 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L183 | corpus:T07:av | AV | {"kind":"diagnostic","expected":"invalid-scalar"} | M0292,M1005 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L184 | corpus:T07:avm | legacy-AVM | {"kind":"diagnostic","expected":"invalid-scalar"} | M0754 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L185 | corpus:T08:a2r | legacy-A2R | {"kind":"trap","expected":"slice-oob"} | M0530,M0529 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L186 | corpus:T08:ac | AC-native | {"kind":"trap","expected":"slice-oob"} | M0327,M1006 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L187 | corpus:T08:av | AV | {"kind":"trap","expected":"slice-oob"} | M0328,M1008 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L188 | corpus:T08:avm | legacy-AVM | {"kind":"trap","expected":"slice-oob"} |  | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L189 | corpus:U01:av | AV | {"kind":"value","expected":"42"} | M0677 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L190 | corpus:U02:av | AV | {"kind":"value","expected":"unit"} | M0679 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L191 | corpus:U03:av | AV | {"kind":"value","expected":"42"} | M0681 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L192 | corpus:U04:av | AV | {"kind":"value","expected":"unit"} | M0682 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L193 | corpus:U05:av | AV | {"kind":"trap","expected":"div-by-zero"} | M0683 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L194 | corpus:U06:av | AV | {"kind":"diagnostic","expected":"entry-missing"} | M0684 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L195 | corpus:U07:av | AV | {"kind":"diagnostic","expected":"entry-params"} | M0685 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L196 | corpus:U08:av | AV | {"kind":"diagnostic","expected":"unit-return-value"} | M0686 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L197 | corpus:U09:av | AV | {"kind":"diagnostic","expected":"entry-tail"} | M0687 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L198 | corpus:U10:av | AV | {"kind":"diagnostic","expected":"entry-duplicate"} | M0688 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L199 | corpus:V01:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0741 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L200 | corpus:V01:ac | AC-native | {"kind":"value","expected":"42"} | M0796 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L201 | corpus:V01:av | AV | {"kind":"value","expected":"42"} | M0652,M0652 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L202 | corpus:V01:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0742 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L203 | corpus:V02:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0743 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L204 | corpus:V02:ac | AC-native | {"kind":"value","expected":"42"} | M0797 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L205 | corpus:V02:av | AV | {"kind":"value","expected":"42"} | M0653,M0653 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L206 | corpus:V02:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0744 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L207 | corpus:V03:a2r | legacy-A2R | {"kind":"value","expected":"3"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L208 | corpus:V03:ac | AC-native | {"kind":"value","expected":"3"} | M0798 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L209 | corpus:V03:av | AV | {"kind":"value","expected":"3"} | M0654,M0654 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L210 | corpus:V04:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0745 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L211 | corpus:V04:ac | AC-native | {"kind":"value","expected":"42"} | M0799 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L212 | corpus:V04:av | AV | {"kind":"value","expected":"42"} | M0655,M0655 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L213 | corpus:V04:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0746 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L214 | corpus:V05:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0747 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L215 | corpus:V05:ac | AC-native | {"kind":"value","expected":"42"} | M0800 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L216 | corpus:V05:av | AV | {"kind":"value","expected":"42"} | M0656,M0656 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L217 | corpus:V05:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0748 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L218 | corpus:V06:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0749 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L219 | corpus:V06:ac | AC-native | {"kind":"value","expected":"42"} | M0801 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L220 | corpus:V06:av | AV | {"kind":"value","expected":"42"} | M0657,M0657 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L221 | corpus:V06:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0750 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L222 | corpus:V07:a2r | legacy-A2R | {"kind":"value","expected":"42"} | M0751 | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L223 | corpus:V07:ac | AC-native | {"kind":"value","expected":"42"} | M0802 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L224 | corpus:V07:av | AV | {"kind":"value","expected":"42"} | M0658,M0658 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L225 | corpus:V07:avm | legacy-AVM | {"kind":"value","expected":"42"} | M0752 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L226 | corpus:W01:ac | AC-native | {"kind":"value","expected":"8000000000000000"} | M1033 | PLAN-891 native typed | 保原件/原预算/原适用性；不把同源当执行 |
| L227 | corpus:W01:av | AV | {"kind":"value","expected":"8000000000000000"} | M0072 | AV typed data parent | 保原件/原预算/原适用性；不把同源当执行 |
| L228 | corpus:W02:ac | AC-native | {"kind":"value","expected":"ffffffffffffffff"} | M1034 | PLAN-891 native typed | 保原件/原预算/原适用性；不把同源当执行 |
| L229 | corpus:W02:av | AV | {"kind":"value","expected":"ffffffffffffffff"} | M0074 | AV typed data parent | 保原件/原预算/原适用性；不把同源当执行 |
| L230 | corpus:W03:ac | AC-native | {"kind":"value","expected":"8000000000000000"} | M1035 | PLAN-891 native typed | 保原件/原预算/原适用性；不把同源当执行 |
| L231 | corpus:W03:av | AV | {"kind":"value","expected":"8000000000000000"} | M0073 | AV typed data parent | 保原件/原预算/原适用性；不把同源当执行 |
| L232 | corpus:W04:ac | AC-native | {"kind":"value","expected":"fff80000deadbeef"} | M1036 | PLAN-891 native typed | 保原件/原预算/原适用性；不把同源当执行 |
| L233 | corpus:W04:av | AV | {"kind":"value","expected":"fff80000deadbeef"} | M0070 | AV typed data parent | 保原件/原预算/原适用性；不把同源当执行 |
| L234 | corpus:W05:ac | AC-native | {"kind":"value","expected":"7f80ad00"} | M1037 | PLAN-891 native typed | 保原件/原预算/原适用性；不把同源当执行 |
| L235 | corpus:W05:av | AV | {"kind":"value","expected":"7f80ad00"} | M0068 | AV typed data parent | 保原件/原预算/原适用性；不把同源当执行 |
| L236 | scenario:C01:ac | AC-native | {"kind":"value","expected":"42"} |  | PLAN-817 CTE | 保原件/原预算/原适用性；不把同源当执行 |
| L237 | scenario:C01:av | AV | {"kind":"value","expected":"42"} |  | PLAN-817 CTE | 保原件/原预算/原适用性；不把同源当执行 |
| L238 | scenario:C02:ac | AC-native | {"kind":"value","expected":"-"} |  | PLAN-817 CTE | 保原件/原预算/原适用性；不把同源当执行 |
| L239 | scenario:C02:av | AV | {"kind":"value","expected":"-"} |  | PLAN-817 CTE | 保原件/原预算/原适用性；不把同源当执行 |
| L240 | scenario:D01:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L241 | scenario:D02:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L242 | scenario:D03:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L243 | scenario:D04:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L244 | scenario:D05:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L245 | scenario:D06:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L246 | scenario:D07:av | AV | {"kind":"hash","expected":"-"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L247 | scenario:D08:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L248 | scenario:D09:av | AV | {"kind":"value","expected":"42"} |  | PLAN-824 AV dynamic | 保原件/原预算/原适用性；不把同源当执行 |
| L249 | scenario:E01:a2r | legacy-A2R | {"kind":"trace","expected":"ba"} | M0858 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L250 | scenario:E01:ac | AC-native | {"kind":"trace","expected":"ba"} | M0856 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L251 | scenario:E01:av | AV | {"kind":"trace","expected":"ba"} | M0857 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L252 | scenario:E01:avm | legacy-AVM | {"kind":"trace","expected":"ba"} | M0859 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L253 | scenario:E02:a2r | legacy-A2R | {"kind":"trace","expected":"ba"} | M0862 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L254 | scenario:E02:ac | AC-native | {"kind":"trace","expected":"ba"} | M0860 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L255 | scenario:E02:av | AV | {"kind":"trace","expected":"ba"} | M0861 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L256 | scenario:E02:avm | legacy-AVM | {"kind":"trace","expected":"ba"} | M0863 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L257 | scenario:E03:a2r | legacy-A2R | {"kind":"group","expected":"-"} | M0581,M0544,M0543 | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L258 | scenario:E03:ac | AC-native | {"kind":"group","expected":"-"} | M0582,M0560,M0616 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L259 | scenario:E03:av | AV | {"kind":"group","expected":"-"} | M0584,M0562,M0618 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L260 | scenario:E03:avm | legacy-AVM | {"kind":"group","expected":"-"} | M0583,M0561,M0617 | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L261 | scenario:G01:a2r | legacy-A2R | {"kind":"cert","expected":"receipt"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L262 | scenario:G02:a2r | legacy-A2R | {"kind":"spawn","expected":"0"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L263 | scenario:G03:a2r | legacy-A2R | {"kind":"cert","expected":"reuse"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L264 | scenario:G04:a2r | legacy-A2R | {"kind":"proposal","expected":"only"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L265 | scenario:G05:a2r | legacy-A2R | {"kind":"cert","expected":"stale"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L266 | scenario:G06:a2r | legacy-A2R | {"kind":"report","expected":"gap"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L267 | scenario:G07:a2r | legacy-A2R | {"kind":"report","expected":"non-green"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L268 | scenario:G08:a2r | legacy-A2R | {"kind":"cert","expected":"reuse"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L269 | scenario:I01:ac | AC-native | {"kind":"value","expected":"edits"} | M0921,M0922 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L270 | scenario:I01:av | AV | {"kind":"value","expected":"edits"} | M0917,M0918,M0919,M0920 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L271 | scenario:I02:ac | AC-native | {"kind":"value","expected":"edits"} | M0921,M0922 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L272 | scenario:I02:av | AV | {"kind":"value","expected":"edits"} | M0917,M0918,M0919,M0920 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L273 | scenario:I03:ac | AC-native | {"kind":"value","expected":"edits"} | M0921,M0922 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L274 | scenario:I03:av | AV | {"kind":"value","expected":"edits"} | M0917,M0918,M0919,M0920 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L275 | scenario:I04:ac | AC-native | {"kind":"value","expected":"edits"} | M0869 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L276 | scenario:I04:av | AV | {"kind":"value","expected":"edits"} | M0870,M0871,M0872 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L277 | scenario:I05:ac | AC-native | {"kind":"value","expected":"hit"} | M0921,M0922 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L278 | scenario:I05:av | AV | {"kind":"value","expected":"hit"} | M0917,M0918,M0919,M0920 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L279 | scenario:I06:ac | AC-native | {"kind":"value","expected":"same"} | M0921,M0922 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L280 | scenario:I06:av | AV | {"kind":"value","expected":"same"} | M0917,M0918,M0919,M0920 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L281 | scenario:I07:av | AV | {"kind":"value","expected":"pins"} |  | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L282 | scenario:I08:av | AV | {"kind":"value","expected":"hot-swap"} | M0874,M0875,M0876 | PLAN-866 AV resident | 保原件/原预算/原适用性；不把同源当执行 |
| L283 | scenario:I09:ac | AC-native | {"kind":"value","expected":"reject"} | M0867,M0868 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L284 | scenario:I09:av | AV | {"kind":"value","expected":"reject"} | M0873 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L285 | scenario:I10:ac | AC-native | {"kind":"value","expected":"bounded"} |  | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L286 | scenario:I10:av | AV | {"kind":"value","expected":"bounded"} |  | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L287 | scenario:I11:ac | AC-native | {"kind":"value","expected":"<1000ms"} |  | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L288 | scenario:I11:av | AV | {"kind":"value","expected":"<1000ms"} |  | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L289 | scenario:I12:a2r | legacy-A2R | {"kind":"group","expected":"non-green"} |  | PLAN-852/908 legacy-A2R golden/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L290 | scenario:I12:ac | AC-native | {"kind":"group","expected":"non-green"} |  | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L291 | scenario:I12:av | AV | {"kind":"group","expected":"non-green"} |  | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L292 | scenario:I12:avm | legacy-AVM | {"kind":"group","expected":"non-green"} |  | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L293 | scenario:R01:a2r | legacy-A2R | {"kind":"events","expected":"lifo"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L294 | scenario:R01:ac | AC-native | {"kind":"events","expected":"lifo"} | M0907 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L295 | scenario:R01:av | AV | {"kind":"events","expected":"lifo"} | M0906 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L296 | scenario:R02:a2r | legacy-A2R | {"kind":"events","expected":"-"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L297 | scenario:R02:ac | AC-native | {"kind":"events","expected":"-"} | M0779,M0780,M0781,M0782,M0783,M0784,M0785,M0786,M0787 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L298 | scenario:R02:av | AV | {"kind":"events","expected":"-"} | M0896,M0899,M0881,M0886,M0895,M0891 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L299 | scenario:R02:avm | legacy-AVM | {"kind":"events","expected":"-"} |  | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L300 | scenario:R03:a2r | legacy-A2R | {"kind":"events","expected":"-"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L301 | scenario:R03:ac | AC-native | {"kind":"events","expected":"-"} | M0835,M0836,M0831,M0832,M0833,M0834,M0837,M0838,M0839 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L302 | scenario:R03:av | AV | {"kind":"events","expected":"-"} | M0900,M0901,M0902,M0903,M0904,M0905 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L303 | scenario:R04:a2r | legacy-A2R | {"kind":"events","expected":"once"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L304 | scenario:R04:ac | AC-native | {"kind":"events","expected":"once"} | M0909 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L305 | scenario:R04:av | AV | {"kind":"events","expected":"once"} | M0908 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L306 | scenario:R05:a2r | legacy-A2R | {"kind":"events","expected":"42"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L307 | scenario:R05:ac | AC-native | {"kind":"events","expected":"42"} | M0779,M0780,M0781,M0782,M0783,M0784,M0785,M0786,M0787 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L308 | scenario:R05:av | AV | {"kind":"events","expected":"42"} |  | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L309 | scenario:R05:avm | legacy-AVM | {"kind":"events","expected":"42"} |  | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |
| L310 | scenario:R06:a2r | legacy-A2R | {"kind":"events","expected":"-"} |  | legacy-A2R composite lowering/certification | 保原件/原预算/原适用性；不把同源当执行 |
| L311 | scenario:R06:ac | AC-native | {"kind":"events","expected":"-"} | M0911,M0913,M0916 | PLAN-852 legacy-required integration; original native producer | 保原件/原预算/原适用性；不把同源当执行 |
| L312 | scenario:R06:av | AV | {"kind":"events","expected":"-"} | M0910,M0912,M0915 | PLAN-852 legacy-required integration; original AV producer | 保原件/原预算/原适用性；不把同源当执行 |
| L313 | scenario:R06:avm | legacy-AVM | {"kind":"events","expected":"-"} |  | legacy-AVM adapter (PLAN-852/920 admission) | 保原件/原预算/原适用性；不把同源当执行 |

</details>

<details>
<summary>A：48条架构recipe及逐fact引用（独立review行为第49条，未冒签）</summary>

| ID | 原架构row | 每个fact的原entry与M/N去向 | composer facts（非产品执行） |
|---|---|---|---|
| A01 | ARCH-01/all-grammars | syn:M1045,M1046,M1047,M1048,M1049,M1050,M1051,M1052,M1053,M1054,M1055 | - |
| A02 | ARCH-01/single-frontdoor | syn:M1045,M1046,M1047,M1048,M1049,M1050,M1051,M1052,M1053,M1054,M1055; syn-local:N277,N278,N279 | - |
| A03 | ARCH-01/nested-body-reuse | syn-local:N277,N278,N279 | - |
| A04 | ARCH-01/source-position-freshness | syn-replay:N280,N281,N282,N283 | - |
| A05 | ARCH-01/public-first-edit | sq-restore:N275,N276; syn-replay:N280,N281,N282,N283 | - |
| A06 | ARCH-02/solver-authority | sq:M0977,M0976,M0975,M0974,M0979,M0978,M0981,M0980,M0983,M0982,M0985,M0984,M0987,M0986; sq-core:N224,N225,N226,N227,N228,N229,N230,N231,N232,N233,N234,N235,N236,N237,N238,N239,N240,N241,N242,N243,N244,N245,N246,N247,N248,N249,N250,N251,N252,N253,N254,N255,N256,N257,N258,N259,N260,N261,N262,N263,N264 | - |
| A07 | ARCH-02/facts-before-hir | sq-core:N224,N225,N226,N227,N228,N229,N230,N231,N232,N233,N234,N235,N236,N237,N238,N239,N240,N241,N242,N243,N244,N245,N246,N247,N248,N249,N250,N251,N252,N253,N254,N255,N256,N257,N258,N259,N260,N261,N262,N263,N264 | - |
| A08 | ARCH-02/positive-negative-deps | sq-core:N224,N225,N226,N227,N228,N229,N230,N231,N232,N233,N234,N235,N236,N237,N238,N239,N240,N241,N242,N243,N244,N245,N246,N247,N248,N249,N250,N251,N252,N253,N254,N255,N256,N257,N258,N259,N260,N261,N262,N263,N264; inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972 | - |
| A09 | ARCH-02/real-instances | inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972 | - |
| A10 | ARCH-02/effective-implementation | inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972; inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274 | - |
| A11 | ARCH-02/required-instance-pages | inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274 | - |
| A12 | ARCH-03/all-declared-q4 | q4:M0332,M0331,M0330,M0329; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003; sq:M0977,M0976,M0975,M0974,M0979,M0978,M0981,M0980,M0983,M0982,M0985,M0984,M0987,M0986; inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A13 | ARCH-03/source-order-and-traps | order:M0856,M0857,M0859,M0858,M0860,M0861,M0863,M0862,M0864,M0865,M0866,M0867,M0868,M0869,M0872,M0871,M0870,M0873 | - |
| A14 | ARCH-03/shared-ir-boundary | q4:M0332,M0331,M0330,M0329; sq:M0977,M0976,M0975,M0974,M0979,M0978,M0981,M0980,M0983,M0982,M0985,M0984,M0987,M0986; inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972 | - |
| A15 | ARCH-03/source-vs-typed | q4:M0332,M0331,M0330,M0329 | - |
| A16 | ARCH-03/certification-two-legs | scalar-cert:N019; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003 | - |
| A17 | ARCH-04/cold-real-av-and-hit | ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A18 | ARCH-04/callee-body-change | ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A19 | ARCH-04/meta-args-and-negative | ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A20 | ARCH-04/instance-child-ct-version | ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223; inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972 | - |
| A21 | ARCH-04/frozen-hir-four-targets | ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A22 | ARCH-04/unsupported-and-resources | ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A23 | ARCH-05/producer-and-io-zero | sq-restore:N275,N276; inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223; rt-restore:N015,N016,N017,N018; coll-restore:N004,N005,N006,N007 | - |
| A24 | ARCH-05/method-closure-body-edits | syn-local:N277,N278,N279; recovery:M0632,M0633,M0634,M0635,M0636; rt-restore:N015,N016,N017,N018 | - |
| A25 | ARCH-05/signature-layout-meta | order:M0856,M0857,M0859,M0858,M0860,M0861,M0863,M0862,M0864,M0865,M0866,M0867,M0868,M0869,M0872,M0871,M0870,M0873; sq-core:N224,N225,N226,N227,N228,N229,N230,N231,N232,N233,N234,N235,N236,N237,N238,N239,N240,N241,N242,N243,N244,N245,N246,N247,N248,N249,N250,N251,N252,N253,N254,N255,N256,N257,N258,N259,N260,N261,N262,N263,N264; inst:M0963,M0962,M0965,M0964,M0967,M0966,M0969,M0968,M0971,M0970,M0973,M0972 | - |
| A26 | ARCH-05/restore-immediate-good-edit | sq-restore:N275,N276; inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223; rt-restore:N015,N016,N017,N018; coll-restore:N004,N005,N006,N007; foreign:N020,N021,N092,N093,N094,N095,N096,N097,N098,N099,N100,N101,N102,N103,N104,N105,N106,N107,N108,N109,N110,N111,N112,N113,N114,N115,N116,N117,N118,N119,N120,N121,N122,N123,N124,N125,N126,N127,N128,N129,N130,N131,N132,N133,N134,N135,N136,N137,N138,N139,N140,N141,N142,N143,N144,N145,N146,N147,N148,N149,N150,N151,N152,N153,N154,N155,N156,N157,N158,N159,N160,N161,N162,N163,N164,N165,N166,N167,N168,N169,N170,N171,N172,N173,N174,N175,N176; foreign-resource:N078,N079,N080,N081,N082,N083,N084,N085,N086,N087,N088,N089,N090,N091 | - |
| A27 | ARCH-05/bad-first-repair-second-restore | recovery:M0632,M0633,M0634,M0635,M0636; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223; rt-restore:N015,N016,N017,N018; coll-restore:N004,N005,N006,N007; foreign:N020,N021,N092,N093,N094,N095,N096,N097,N098,N099,N100,N101,N102,N103,N104,N105,N106,N107,N108,N109,N110,N111,N112,N113,N114,N115,N116,N117,N118,N119,N120,N121,N122,N123,N124,N125,N126,N127,N128,N129,N130,N131,N132,N133,N134,N135,N136,N137,N138,N139,N140,N141,N142,N143,N144,N145,N146,N147,N148,N149,N150,N151,N152,N153,N154,N155,N156,N157,N158,N159,N160,N161,N162,N163,N164,N165,N166,N167,N168,N169,N170,N171,N172,N173,N174,N175,N176; foreign-resource:N078,N079,N080,N081,N082,N083,N084,N085,N086,N087,N088,N089,N090,N091 | - |
| A28 | ARCH-05/all-required-pages | sq-restore:N275,N276; inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223; rt-restore:N015,N016,N017,N018; coll-restore:N004,N005,N006,N007; foreign:N020,N021,N092,N093,N094,N095,N096,N097,N098,N099,N100,N101,N102,N103,N104,N105,N106,N107,N108,N109,N110,N111,N112,N113,N114,N115,N116,N117,N118,N119,N120,N121,N122,N123,N124,N125,N126,N127,N128,N129,N130,N131,N132,N133,N134,N135,N136,N137,N138,N139,N140,N141,N142,N143,N144,N145,N146,N147,N148,N149,N150,N151,N152,N153,N154,N155,N156,N157,N158,N159,N160,N161,N162,N163,N164,N165,N166,N167,N168,N169,N170,N171,N172,N173,N174,N175,N176; foreign-resource:N078,N079,N080,N081,N082,N083,N084,N085,N086,N087,N088,N089,N090,N091 | - |
| A29 | ARCH-05/atomic-publication-and-late | sq-restore:N275,N276; inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223; rt-restore:N015,N016,N017,N018; coll-restore:N004,N005,N006,N007; foreign:N020,N021,N092,N093,N094,N095,N096,N097,N098,N099,N100,N101,N102,N103,N104,N105,N106,N107,N108,N109,N110,N111,N112,N113,N114,N115,N116,N117,N118,N119,N120,N121,N122,N123,N124,N125,N126,N127,N128,N129,N130,N131,N132,N133,N134,N135,N136,N137,N138,N139,N140,N141,N142,N143,N144,N145,N146,N147,N148,N149,N150,N151,N152,N153,N154,N155,N156,N157,N158,N159,N160,N161,N162,N163,N164,N165,N166,N167,N168,N169,N170,N171,N172,N173,N174,N175,N176; foreign-resource:N078,N079,N080,N081,N082,N083,N084,N085,N086,N087,N088,N089,N090,N091 | - |
| A30 | ARCH-05/historical-cross-project-ownership | sq-restore:N275,N276; inst-restore:N265,N266,N267,N268,N269,N270,N271,N272,N273,N274; ct:M0949,M0948,M0951,M0950,M0953,M0952,M0955,M0954,M0957,M0956,M0959,M0958,M0961,M0960; ct-service:N191,N192,N193,N194,N195,N196,N197,N198,N199,N200,N201,N202,N203,N204,N205,N206,N207,N208,N209,N210,N211,N212,N213,N214,N215,N216,N217,N218,N219,N220,N221,N222,N223 | - |
| A31 | ARCH-06/resource-facts-authority | events:M0906,M0907,M0908,M0909,M0910,M0911,M0912,M0913,M0914,M0915,M0916; native-fault:M0831,M0832,M0833,M0834,M0835,M0836,M0837,M0838,M0839; closure-fault:M0779,M0782,M0783,M0780,M0781,M0784,M0786,M0787,M0785,M0788; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003 | - |
| A32 | ARCH-06/every-allocation-fault-and-retry | av-fault:M0881,M0877,M0878,M0879,M0880,M0886,M0882,M0883,M0884,M0885,M0891,M0887,M0888,M0889,M0890,M0895,M0892,M0893,M0894,M0896,M0899,M0897,M0898,M0902,M0901,M0903,M0904,M0900,M0905; native-fault:M0831,M0832,M0833,M0834,M0835,M0836,M0837,M0838,M0839; closure-fault:M0779,M0782,M0783,M0780,M0781,M0784,M0786,M0787,M0785,M0788; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003 | - |
| A33 | ARCH-06/failed-source-mut-out-atomicity | native-fault:M0831,M0832,M0833,M0834,M0835,M0836,M0837,M0838,M0839; closure-fault:M0779,M0782,M0783,M0780,M0781,M0784,M0786,M0787,M0785,M0788; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003; foreign-resource:N078,N079,N080,N081,N082,N083,N084,N085,N086,N087,N088,N089,N090,N091 | - |
| A34 | ARCH-06/all-domain-budgets | native-fault:M0831,M0832,M0833,M0834,M0835,M0836,M0837,M0838,M0839; av-fault:M0881,M0877,M0878,M0879,M0880,M0886,M0882,M0883,M0884,M0885,M0891,M0887,M0888,M0889,M0890,M0895,M0892,M0893,M0894,M0896,M0899,M0897,M0898,M0902,M0901,M0903,M0904,M0900,M0905; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003; abi-layout:N054,N055,N056,N057,N058,N059,N060,N061,N062,N063,N064,N065,N066,N067,N068,N069,N070,N071,N072,N073,N074,N075,N076,N077 | - |
| A35 | ARCH-06/abi-original-three-directions | abi-three:N047,N048,N049,N050,N051,N052,N053; abi-layout:N054,N055,N056,N057,N058,N059,N060,N061,N062,N063,N064,N065,N066,N067,N068,N069,N070,N071,N072,N073,N074,N075,N076,N077; bridge:M0196,M0197,M0198,M0199,M0200,M0201 | - |
| A36 | ARCH-06/d1-source-vs-typed | bridge:M0196,M0197,M0198,M0199,M0200,M0201 | - |
| A37 | ARCH-06/d2-real-source-imports | foreign:N020,N021,N092,N093,N094,N095,N096,N097,N098,N099,N100,N101,N102,N103,N104,N105,N106,N107,N108,N109,N110,N111,N112,N113,N114,N115,N116,N117,N118,N119,N120,N121,N122,N123,N124,N125,N126,N127,N128,N129,N130,N131,N132,N133,N134,N135,N136,N137,N138,N139,N140,N141,N142,N143,N144,N145,N146,N147,N148,N149,N150,N151,N152,N153,N154,N155,N156,N157,N158,N159,N160,N161,N162,N163,N164,N165,N166,N167,N168,N169,N170,N171,N172,N173,N174,N175,N176 | - |
| A38 | ARCH-06/d3-resources-and-callback | foreign-resource:N078,N079,N080,N081,N082,N083,N084,N085,N086,N087,N088,N089,N090,N091 | - |
| A39 | ARCH-07/one-pid-one-entry | resident:M0874,M0875,M0876; av-resident:N177,N178,N179,N180,N181,N182,N183,N184,N185,N186,N187,N188,N189,N190 | - |
| A40 | ARCH-07/last-owner-gc | resident:M0874,M0875,M0876; av-resident:N177,N178,N179,N180,N181,N182,N183,N184,N185,N186,N187,N188,N189,N190 | - |
| A41 | ARCH-07/original-churn-and-samples | resident:M0874,M0875,M0876; av-resident:N177,N178,N179,N180,N181,N182,N183,N184,N185,N186,N187,N188,N189,N190; rt-restore:N015,N016,N017,N018; coll-restore:N004,N005,N006,N007; abi-cache:N022,N023,N024,N025,N026,N027,N028,N029,N030,N031,N032,N033,N034,N035,N036,N037,N038,N039,N040,N041,N042,N043,N044,N045,N046 | - |
| A42 | ARCH-08/approved-old-conservation | q4:M0332,M0331,M0330,M0329 | - |
| A43 | ARCH-08/old38-cert-separation | scalar-cert:N019 | - |
| A44 | ARCH-08/strict-all-fields-and-paths | q4:M0332,M0331,M0330,M0329 | document-byte-caps-enforced |
| A45 | ARCH-08/original-execution-terminal | q4:M0332,M0331,M0330,M0329 | raw-terminal-bound-execution |
| A46 | ARCH-08/daily-cert-axis | scalar-cert:N019; rt-cert:N008,N009,N010,N011,N012,N013,N014; coll-cert:N001,N002,N003 | - |
| A47 | ARCH-08/required-and-no-empty | q4:M0332,M0331,M0330,M0329 | denominator-and-honest-three-state |
| A48 | ARCH-08/cost-memory-and-phase | q4:M0332,M0331,M0330,M0329 | admission-peak-within-live-cap |

review_row=ARCH-08/fresh-union-spec-review：仅有合法独立凭据可关闭，本轮不生成/复用R冒独立审查。

</details>

<details>
<summary>N：283个架构原生测试身份到实际源码声明</summary>

| ID | 原package/target$完整测试名 | 声明源码:行 | 本轮状态 |
|---|---|---|---|
| N001 | ace-a2x-worker::conformance$a2c_coll_cert::a2c_coll_certify_all_suite_cases | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_cert.rs:130 | package/target/完整module路径静态相符；未运行 |
| N002 | ace-a2x-worker::conformance$a2c_coll_cert::a2c_coll_certify_wrong_legs_rejected | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_cert.rs:157 | package/target/完整module路径静态相符；未运行 |
| N003 | ace-a2x-worker::conformance$a2c_coll_cert::a2c_coll_daily_emit_spawn_zero | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_cert.rs:197 | package/target/完整module路径静态相符；未运行 |
| N004 | ace-a2x-worker::conformance$a2c_coll_restore::coll_readonly_lease_gc_liveness | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_restore.rs:189 | package/target/完整module路径静态相符；未运行 |
| N005 | ace-a2x-worker::conformance$a2c_coll_restore::coll_restore_cross_target_isolation | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_restore.rs:221 | package/target/完整module路径静态相符；未运行 |
| N006 | ace-a2x-worker::conformance$a2c_coll_restore::coll_restore_pure_p0_then_next_edit_then_restore_again | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_restore.rs:54 | package/target/完整module路径静态相符；未运行 |
| N007 | ace-a2x-worker::conformance$a2c_coll_restore::coll_restore_rejects_and_repairs | experimental/ace/crates/a2x-worker/tests/cases/a2c_coll_restore.rs:132 | package/target/完整module路径静态相符；未运行 |
| N008 | ace-a2x-worker::conformance$a2c_rt_cert::rt_cert_draft_strict_controls | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:182 | package/target/完整module路径静态相符；未运行 |
| N009 | ace-a2x-worker::conformance$a2c_rt_cert::rt_certify_all_suite_cases | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:263 | package/target/完整module路径静态相符；未运行 |
| N010 | ace-a2x-worker::conformance$a2c_rt_cert::rt_certify_bad_references_refused | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:500 | package/target/完整module路径静态相符；未运行 |
| N011 | ace-a2x-worker::conformance$a2c_rt_cert::rt_certify_modes_matrix | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:408 | package/target/完整module路径静态相符；未运行 |
| N012 | ace-a2x-worker::conformance$a2c_rt_cert::rt_certify_trap_axis | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:463 | package/target/完整module路径静态相符；未运行 |
| N013 | ace-a2x-worker::conformance$a2c_rt_cert::rt_certify_two_leg_pass_and_stable_bytes | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:142 | package/target/完整module路径静态相符；未运行 |
| N014 | ace-a2x-worker::conformance$a2c_rt_cert::rt_certify_wrong_oracle_refused | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_cert.rs:166 | package/target/完整module路径静态相符；未运行 |
| N015 | ace-a2x-worker::conformance$a2c_rt_restore::rt_readonly_lease_gc_liveness | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_restore.rs:201 | package/target/完整module路径静态相符；未运行 |
| N016 | ace-a2x-worker::conformance$a2c_rt_restore::rt_restore_cross_target_and_family_isolation | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_restore.rs:234 | package/target/完整module路径静态相符；未运行 |
| N017 | ace-a2x-worker::conformance$a2c_rt_restore::rt_restore_pure_p0_then_method_edit_then_restore_again | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_restore.rs:52 | package/target/完整module路径静态相符；未运行 |
| N018 | ace-a2x-worker::conformance$a2c_rt_restore::rt_restore_rejects_and_repairs | experimental/ace/crates/a2x-worker/tests/cases/a2c_rt_restore.rs:135 | package/target/完整module路径静态相符；未运行 |
| N019 | ace-a2x-worker::conformance$contract_certification::certify_dry_run_b01 | experimental/ace/crates/a2x-worker/tests/cases/contract_certification.rs:79 | package/target/完整module路径静态相符；未运行 |
| N020 | ace-abi::ace_abi$source_foreign::function_cache::resource_tests::target_cache_put_reserves_payload_copies_before_writing_and_repairs_after_release | experimental/ace/crates/abi/src/source_function_cache.rs:99 | package/target/完整module路径静态相符；未运行 |
| N021 | ace-abi::ace_abi$source_foreign::publication_scratch::tests::pointer_encoder_keeps_its_ticket_and_refuses_missing_repeated_or_failed_records | experimental/ace/crates/abi/src/source_publication_scratch.rs:282 | package/target/完整module路径静态相符；未运行 |
| N022 | ace-abi::conformance$cache_turnaround::abi_cache_artifact_path_not_basename | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1696 | package/target/完整module路径静态相符；未运行 |
| N023 | ace-abi::conformance$cache_turnaround::abi_cache_bare_source_path_resolves_cwd | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1752 | package/target/完整module路径静态相符；未运行 |
| N024 | ace-abi::conformance$cache_turnaround::abi_cache_cross_project_same_content_different_deps | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:734 | package/target/完整module路径静态相符；未运行 |
| N025 | ace-abi::conformance$cache_turnaround::abi_cache_cross_project_sharing | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:223 | package/target/完整module路径静态相符；未运行 |
| N026 | ace-abi::conformance$cache_turnaround::abi_cache_extensionless_header_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1033 | package/target/完整module路径静态相符；未运行 |
| N027 | ace-abi::conformance$cache_turnaround::abi_cache_foreign_toolchain_generation_not_served | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:816 | package/target/完整module路径静态相符；未运行 |
| N028 | ace-abi::conformance$cache_turnaround::abi_cache_include_real_deps_invalidated_and_reused | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:544 | package/target/完整module路径静态相符；未运行 |
| N029 | ace-abi::conformance$cache_turnaround::abi_cache_include_records_system_header_closure | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:657 | package/target/完整module路径静态相符；未运行 |
| N030 | ace-abi::conformance$cache_turnaround::abi_cache_invalidation_by_real_inputs | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:150 | package/target/完整module路径静态相符；未运行 |
| N031 | ace-abi::conformance$cache_turnaround::abi_cache_legacy_v3_envelope_never_served | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1453 | package/target/完整module路径静态相符；未运行 |
| N032 | ace-abi::conformance$cache_turnaround::abi_cache_missing_block_is_miss_then_recompute | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:300 | package/target/完整module路径静态相符；未运行 |
| N033 | ace-abi::conformance$cache_turnaround::abi_cache_nearby_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:888 | package/target/完整module路径静态相符；未运行 |
| N034 | ace-abi::conformance$cache_turnaround::abi_cache_nested_artifact_cleanup_reuses | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1925 | package/target/完整module路径静态相符；未运行 |
| N035 | ace-abi::conformance$cache_turnaround::abi_cache_nested_subpath_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:985 | package/target/完整module路径静态相符；未运行 |
| N036 | ace-abi::conformance$cache_turnaround::abi_cache_no_include_cross_project_both_succeed | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1141 | package/target/完整module路径静态相符；未运行 |
| N037 | ace-abi::conformance$cache_turnaround::abi_cache_object_hit_never_calls_producer | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:105 | package/target/完整module路径静态相符；未运行 |
| N038 | ace-abi::conformance$cache_turnaround::abi_cache_parent_path_escape_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1319 | package/target/完整module路径静态相符；未运行 |
| N039 | ace-abi::conformance$cache_turnaround::abi_cache_relative_paths_bind_cwd | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1612 | package/target/完整module路径静态相符；未运行 |
| N040 | ace-abi::conformance$cache_turnaround::abi_cache_tampered_block_is_controlled_error | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:244 | package/target/完整module路径静态相符；未运行 |
| N041 | ace-abi::conformance$cache_turnaround::abi_cache_tmp_named_header_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1391 | package/target/完整module路径静态相符；未运行 |
| N042 | ace-abi::conformance$cache_turnaround::abi_cache_transitive_header_dir_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1178 | package/target/完整module路径静态相符；未运行 |
| N043 | ace-abi::conformance$cache_turnaround::abi_cache_transitive_subpath_shadow_invalidates | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1243 | package/target/完整module路径静态相符；未运行 |
| N044 | ace-abi::conformance$cache_turnaround::abi_cache_unused_nearby_file_legal_recompile | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1088 | package/target/完整module路径静态相符；未运行 |
| N045 | ace-abi::conformance$cache_turnaround::abi_cache_v3_same_materials_upgrade_publishes | experimental/ace/crates/abi/tests/cases/cache_turnaround.rs:1800 | package/target/完整module路径静态相符；未运行 |
| N046 | ace-abi::conformance$cache_turnaround::ace_latency::nearest_rank_boundaries | experimental/ace/crates/abi/tests/common/ace_latency.rs:11 | package/target/完整module路径静态相符；未运行 |
| N047 | ace-abi::conformance$calls::abi_calls_consumer_rejects_tampered_boundary | experimental/ace/crates/abi/tests/cases/calls.rs:336 | package/target/完整module路径静态相符；未运行 |
| N048 | ace-abi::conformance$calls::abi_calls_leg_a_full_suite | experimental/ace/crates/abi/tests/cases/calls.rs:164 | package/target/完整module路径静态相符；未运行 |
| N049 | ace-abi::conformance$calls::abi_calls_leg_b_full_suite | experimental/ace/crates/abi/tests/cases/calls.rs:192 | package/target/完整module路径静态相符；未运行 |
| N050 | ace-abi::conformance$calls::abi_calls_leg_c_full_suite_nested | experimental/ace/crates/abi/tests/cases/calls.rs:218 | package/target/完整module路径静态相符；未运行 |
| N051 | ace-abi::conformance$calls::abi_calls_missed_call_control_exits_1 | experimental/ace/crates/abi/tests/cases/calls.rs:304 | package/target/完整module路径静态相符；未运行 |
| N052 | ace-abi::conformance$calls::abi_calls_swapped_args_control_exits_1 | experimental/ace/crates/abi/tests/cases/calls.rs:241 | package/target/完整module路径静态相符；未运行 |
| N053 | ace-abi::conformance$calls::abi_calls_wrong_record_offset_control_exits_1 | experimental/ace/crates/abi/tests/cases/calls.rs:272 | package/target/完整module路径静态相符；未运行 |
| N054 | ace-abi::conformance$contract_layout_admission::abi_admission_deep_chain_bounded_and_working | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:799 | package/target/完整module路径静态相符；未运行 |
| N055 | ace-abi::conformance$contract_layout_admission::abi_admission_flatten_phase_peak_budgeted | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:1229 | package/target/完整module路径静态相符；未运行 |
| N056 | ace-abi::conformance$contract_layout_admission::abi_admission_index_budget_counts_memo_state | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:1110 | package/target/完整module路径静态相符；未运行 |
| N057 | ace-abi::conformance$contract_layout_admission::abi_admission_missing_semantic_ref_rejected | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:782 | package/target/完整module路径静态相符；未运行 |
| N058 | ace-abi::conformance$contract_layout_admission::abi_admission_oversize_record_is_resource_rejection | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:435 | package/target/完整module路径静态相符；未运行 |
| N059 | ace-abi::conformance$contract_layout_admission::abi_admission_recursive_record_rejected | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:382 | package/target/完整module路径静态相符；未运行 |
| N060 | ace-abi::conformance$contract_layout_admission::abi_admission_same_width_respell_is_semantic_reject | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:511 | package/target/完整module路径静态相符；未运行 |
| N061 | ace-abi::conformance$contract_layout_admission::abi_admission_semref_gate_end_to_end | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:843 | package/target/完整module路径静态相符；未运行 |
| N062 | ace-abi::conformance$contract_layout_admission::abi_admission_shared_subgraph_flatten_path_budgeted | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:1287 | package/target/完整module路径静态相符；未运行 |
| N063 | ace-abi::conformance$contract_layout_admission::abi_admission_staging_gate_counts_representation_bytes | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:672 | package/target/完整module路径静态相符；未运行 |
| N064 | ace-abi::conformance$contract_layout_admission::abi_admission_staging_gate_original_long_name_input_refused | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:735 | package/target/完整module路径静态相符；未运行 |
| N065 | ace-abi::conformance$contract_layout_admission::abi_admission_tampered_payload_original_label_rejected | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:603 | package/target/完整module路径静态相符；未运行 |
| N066 | ace-abi::conformance$contract_layout_admission::abi_admission_tiny_graph_with_long_name_computes | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:1170 | package/target/完整module路径静态相符；未运行 |
| N067 | ace-abi::conformance$contract_layout_admission::abi_admission_truthful_label_tampered_payload_also_rejected | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:639 | package/target/完整module路径静态相符；未运行 |
| N068 | ace-abi::conformance$contract_layout_admission::abi_admission_unknown_record_rejected | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:428 | package/target/完整module路径静态相符；未运行 |
| N069 | ace-abi::conformance$contract_layout_admission::abi_contract_absent_capability_symbol_rejected | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:1089 | package/target/完整module路径静态相符；未运行 |
| N070 | ace-abi::conformance$contract_layout_admission::abi_contract_admission_rejects_each_dimension | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:30 | package/target/完整module路径静态相符；未运行 |
| N071 | ace-abi::conformance$contract_layout_admission::abi_contract_c_provider_descriptor_matches_independent_header | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:931 | package/target/完整module路径静态相符；未运行 |
| N072 | ace-abi::conformance$contract_layout_admission::abi_contract_descriptor_roundtrip_and_fingerprint | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:134 | package/target/完整module路径静态相符；未运行 |
| N073 | ace-abi::conformance$contract_layout_admission::abi_contract_fallible_form_spells_ctx | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:888 | package/target/完整module路径静态相符；未运行 |
| N074 | ace-abi::conformance$contract_layout_admission::abi_contract_production_builder_spells_recview | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:865 | package/target/完整module路径静态相符；未运行 |
| N075 | ace-abi::conformance$contract_layout_admission::abi_layout_field_order_is_load_bearing | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:367 | package/target/完整module路径静态相符；未运行 |
| N076 | ace-abi::conformance$contract_layout_admission::abi_layout_matches_c_oracle | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:309 | package/target/完整module路径静态相符；未运行 |
| N077 | ace-abi::conformance$contract_layout_admission::abi_layout_wrong_offset_control_fails | experimental/ace/crates/abi/tests/cases/contract_layout_admission.rs:349 | package/target/完整module路径静态相符；未运行 |
| N078 | ace-abi::conformance$resource::abi_resource939_fault_sweep_allocation_domain | experimental/ace/crates/abi/tests/cases/resource.rs:442 | package/target/完整module路径静态相符；未运行 |
| N079 | ace-abi::conformance$resource::abi_resource939_provider_generations_lease_and_gc | experimental/ace/crates/abi/tests/cases/resource.rs:319 | package/target/完整module路径静态相符；未运行 |
| N080 | ace-abi::conformance$resource::abi_resource_autoabi_consumer_full_protocol | experimental/ace/crates/abi/tests/cases/resource.rs:200 | package/target/完整module路径静态相符；未运行 |
| N081 | ace-abi::conformance$resource::abi_resource_c_consumer_full_protocol | experimental/ace/crates/abi/tests/cases/resource.rs:225 | package/target/完整module路径静态相符；未运行 |
| N082 | ace-abi::conformance$resource::abi_resource_edges_independent_c_control | experimental/ace/crates/abi/tests/cases/resource.rs:289 | package/target/完整module路径静态相符；未运行 |
| N083 | ace-abi::conformance$resource::abi_resource_record_mutout_c_control | experimental/ace/crates/abi/tests/cases/resource.rs:467 | package/target/完整module路径静态相符；未运行 |
| N084 | ace-abi::conformance$resource::abi_resource_rollback_control_discriminates | experimental/ace/crates/abi/tests/cases/resource.rs:251 | package/target/完整module路径静态相符；未运行 |
| N085 | ace-abi::conformance$resource::abi_resource_utf8_and_recvalid_autoabi_consumer | experimental/ace/crates/abi/tests/cases/resource.rs:516 | package/target/完整module路径静态相符；未运行 |
| N086 | ace-abi::conformance$resource_recovery::abi_resource939_body_edit_reproduces_only_the_edited_function | experimental/ace/crates/abi/tests/cases/resource_recovery.rs:298 | package/target/完整module路径静态相符；未运行 |
| N087 | ace-abi::conformance$resource_recovery::abi_resource939_missing_or_corrupt_required_pages_refuse_restore_never_cold_start | experimental/ace/crates/abi/tests/cases/resource_recovery.rs:166 | package/target/完整module路径静态相符；未运行 |
| N088 | ace-abi::conformance$resource_recovery::abi_resource939_over_budget_cancellation_preserves_root_and_cleans_without_quota | experimental/ace/crates/abi/tests/cases/resource_recovery.rs:424 | package/target/完整module路径静态相符；未运行 |
| N089 | ace-abi::conformance$resource_recovery::abi_resource939_pointer_replace_failure_keeps_accepted_root | experimental/ace/crates/abi/tests/cases/resource_recovery.rs:496 | package/target/完整module路径静态相符；未运行 |
| N090 | ace-abi::conformance$resource_recovery::abi_resource939_restore_first_edit_fails_publishes_nothing_then_fix_and_recover | experimental/ace/crates/abi/tests/cases/resource_recovery.rs:93 | package/target/完整module路径静态相符；未运行 |
| N091 | ace-abi::conformance$resource_recovery::abi_resource939_revision_churn_gc_convergence_and_lease | experimental/ace/crates/abi/tests/cases/resource_recovery.rs:568 | package/target/完整module路径静态相符；未运行 |
| N092 | ace-abi::conformance$source_foreign::source_foreign_actual_provider_symbols_are_checked_before_source_or_target_production | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1427 | package/target/完整module路径静态相符；未运行 |
| N093 | ace-abi::conformance$source_foreign::source_foreign_authorized_capability_edit_after_restore_only_rechecks_actual_readers | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1847 | package/target/完整module路径静态相符；未运行 |
| N094 | ace-abi::conformance$source_foreign::source_foreign_both_targets_preserve_status_bool_order_short_circuit_and_mixed_slots | experimental/ace/crates/abi/tests/cases/source_foreign.rs:861 | package/target/完整module路径静态相符；未运行 |
| N095 | ace-abi::conformance$source_foreign::source_foreign_c_consumes_checked_hir_and_calls_independently_compiled_provider | experimental/ace/crates/abi/tests/cases/source_foreign.rs:701 | package/target/完整module路径静态相符；未运行 |
| N096 | ace-abi::conformance$source_foreign::source_foreign_context_binding_requires_actual_data_symbol_and_keeps_nonzero_context_legal | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1477 | package/target/完整module路径静态相符；未运行 |
| N097 | ace-abi::conformance$source_foreign::source_foreign_daily_emit_and_metadata_spawn_no_tools_with_an_existing_provider_input | experimental/ace/crates/abi/tests/cases/source_foreign.rs:20 | package/target/完整module路径静态相符；未运行 |
| N098 | ace-abi::conformance$source_foreign::source_foreign_declaration_readers_atomic_after_restore | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1612 | package/target/完整module路径静态相符；未运行 |
| N099 | ace-abi::conformance$source_foreign::source_foreign_external_compiler_output_refusal_preserves_current_and_next_request_repairs | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3076 | package/target/完整module路径静态相符；未运行 |
| N100 | ace-abi::conformance$source_foreign::source_foreign_formal_cache_discovery_budget_keeps_pointer_and_repairs_without_production | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3417 | package/target/完整module路径静态相符；未运行 |
| N101 | ace-abi::conformance$source_foreign::source_foreign_formal_early_diagnostics_recover_owned_provider_residue_without_publishing | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3330 | package/target/完整module路径静态相符；未运行 |
| N102 | ace-abi::conformance$source_foreign::source_foreign_formal_request_metadata_budget_preserves_pointer_and_repairs | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3503 | package/target/完整module路径静态相符；未运行 |
| N103 | ace-abi::conformance$source_foreign::source_foreign_formal_restart_first_error_repair_preserves_structured_diagnostics_and_pointer | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3549 | package/target/完整module路径静态相符；未运行 |
| N104 | ace-abi::conformance$source_foreign::source_foreign_formal_scalar_review_transition_restores_without_hidden_interface_production | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3149 | package/target/完整module路径静态相符；未运行 |
| N105 | ace-abi::conformance$source_foreign::source_foreign_formal_tool_io_failure_keeps_pointer_and_recovers_owned_scratch | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3237 | package/target/完整module路径静态相符；未运行 |
| N106 | ace-abi::conformance$source_foreign::source_foreign_formal_worker_rebinds_provider_without_any_caller_production | experimental/ace/crates/abi/tests/cases/source_foreign.rs:2108 | package/target/完整module路径静态相符；未运行 |
| N107 | ace-abi::conformance$source_foreign::source_foreign_history_hits_before_lowering_and_uses_original_a2x_cache_after_restart | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3661 | package/target/完整module路径静态相符；未运行 |
| N108 | ace-abi::conformance$source_foreign::source_foreign_hot_collection_reclaims_revisions_and_preserves_current_and_reader_closures | experimental/ace/crates/abi/tests/cases/source_foreign.rs:303 | package/target/完整module路径静态相符；未运行 |
| N109 | ace-abi::conformance$source_foreign::source_foreign_name_changes_keep_nonreader_slots | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1990 | package/target/完整module路径静态相符；未运行 |
| N110 | ace-abi::conformance$source_foreign::source_foreign_native_analysis_uses_shared_staging_and_repairs_without_codegen_on_refusal | experimental/ace/crates/abi/tests/cases/source_foreign.rs:258 | package/target/完整module路径静态相符；未运行 |
| N111 | ace-abi::conformance$source_foreign::source_foreign_native_calls_real_c_provider_and_body_edit_keeps_caller_identity | experimental/ace/crates/abi/tests/cases/source_foreign.rs:639 | package/target/完整module路径静态相符；未运行 |
| N112 | ace-abi::conformance$source_foreign::source_foreign_native_output_reservation_is_live_until_cache_handoff | experimental/ace/crates/abi/tests/cases/source_foreign.rs:917 | package/target/完整module路径静态相符；未运行 |
| N113 | ace-abi::conformance$source_foreign::source_foreign_protocol_rejects_unknown_fields_and_empty_selection_before_any_store_or_tool | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1536 | package/target/完整module路径静态相符；未运行 |
| N114 | ace-abi::conformance$source_foreign::source_foreign_provider_symbol_index_uses_shared_budget_before_rebind_and_repairs | experimental/ace/crates/abi/tests/cases/source_foreign.rs:180 | package/target/完整module路径静态相符；未运行 |
| N115 | ace-abi::conformance$source_foreign::source_foreign_publication_faults_and_revocation_preserve_the_accepted_root | experimental/ace/crates/abi/tests/cases/source_foreign.rs:2227 | package/target/完整module路径静态相符；未运行 |
| N116 | ace-abi::conformance$source_foreign::source_foreign_publication_recovers_a_crashed_registration_and_gc_preserves_live_readers | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3858 | package/target/完整module路径静态相符；未运行 |
| N117 | ace-abi::conformance$source_foreign::source_foreign_real_process_restart_uses_required_managed_pages_before_first_edit | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1338 | package/target/完整module路径静态相符；未运行 |
| N118 | ace-abi::conformance$source_foreign::source_foreign_real_signature_and_source_repair_publish_together_after_restore | experimental/ace/crates/abi/tests/cases/source_foreign.rs:2557 | package/target/完整module路径静态相符；未运行 |
| N119 | ace-abi::conformance$source_foreign::source_foreign_request_output_quota_refuses_real_first_edit_and_next_request_repairs | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3044 | package/target/完整module路径静态相符；未运行 |
| N120 | ace-abi::conformance$source_foreign::source_foreign_required_page_output_refusal_precedes_first_contract_edit_and_repairs | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3112 | package/target/完整module路径静态相符；未运行 |
| N121 | ace-abi::conformance$source_foreign::source_foreign_required_restore_rejects_each_missing_corrupt_or_misbound_page | experimental/ace/crates/abi/tests/cases/source_foreign.rs:2799 | package/target/完整module路径静态相符；未运行 |
| N122 | ace-abi::conformance$source_foreign::source_foreign_restore_shares_remaining_mir_budget_and_repairs_after_release | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1095 | package/target/完整module路径静态相符；未运行 |
| N123 | ace-abi::conformance$source_foreign::source_foreign_runtime_policy_pin_matches_current_sdk_material | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3778 | package/target/完整module路径静态相符；未运行 |
| N124 | ace-abi::conformance$source_foreign::source_foreign_selected_targets_restore_then_immediate_edit_and_failed_first_edit | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1194 | package/target/完整module路径静态相符；未运行 |
| N125 | ace-abi::conformance$source_foreign::source_foreign_target_source_caps_reject_before_production_and_preserve_current | experimental/ace/crates/abi/tests/cases/source_foreign.rs:1023 | package/target/完整module路径静态相符；未运行 |
| N126 | ace-abi::conformance$source_foreign::source_foreign_warm_body_edit_p90_gate | experimental/ace/crates/abi/tests/cases/source_foreign.rs:4256 | package/target/完整module路径静态相符；未运行 |
| N127 | ace-abi::conformance$source_foreign::source_foreign_work_tracks_target_discard_accept_and_unchanged_publication_payload_io | experimental/ace/crates/abi/tests/cases/source_foreign.rs:3792 | package/target/完整module路径静态相符；未运行 |
| N128 | ace-abi::source_resources$source_foreign::actual_provider_rebind_heap_fits_its_precharged_symbol_index_window | experimental/ace/crates/abi/tests/source_resources.rs:4764 | package/target/完整module路径静态相符；未运行 |
| N129 | ace-abi::source_resources$source_foreign::allocator_window_counts_new_live_buffers_after_older_frees_and_real_reallocation | experimental/ace/crates/abi/tests/source_resources.rs:1169 | package/target/完整module路径静态相符；未运行 |
| N130 | ace-abi::source_resources$source_foreign::c_cache_canonical_text_admission_precedes_store_publication_and_repairs | experimental/ace/crates/abi/tests/source_resources.rs:2667 | package/target/完整module路径静态相符；未运行 |
| N131 | ace-abi::source_resources$source_foreign::c_cache_conflict_releases_finished_encoding_before_rediscovering_the_canonical_object | experimental/ace/crates/abi/tests/source_resources.rs:4073 | package/target/完整module路径静态相符；未运行 |
| N132 | ace-abi::source_resources$source_foreign::c_cache_decoded_object_is_charged_at_its_real_allocation_and_pinned_by_aliases | experimental/ace/crates/abi/tests/source_resources.rs:4197 | package/target/完整module路径静态相符；未运行 |
| N133 | ace-abi::source_resources$source_foreign::c_cache_dependency_metadata_precharges_its_array_and_retains_paths_through_encoding | experimental/ace/crates/abi/tests/source_resources.rs:3719 | package/target/完整module路径静态相符；未运行 |
| N134 | ace-abi::source_resources$source_foreign::c_cache_discovery_ids_refuse_before_copy_and_repair_without_calling_the_producer | experimental/ace/crates/abi/tests/source_resources.rs:4470 | package/target/完整module路径静态相符；未运行 |
| N135 | ace-abi::source_resources$source_foreign::c_cache_encoded_payload_precharges_coexistence_and_preserves_legacy_wire_bytes | experimental/ace/crates/abi/tests/source_resources.rs:3918 | package/target/完整module路径静态相符；未运行 |
| N136 | ace-abi::source_resources$source_foreign::c_cache_entry_decode_reserves_shared_index_before_owning_typed_metadata | experimental/ace/crates/abi/tests/source_resources.rs:4399 | package/target/完整module路径静态相符；未运行 |
| N137 | ace-abi::source_resources$source_foreign::c_cache_entry_read_reserves_shared_io_before_loading_the_original_store_frame | experimental/ace/crates/abi/tests/source_resources.rs:3862 | package/target/完整module路径静态相符；未运行 |
| N138 | ace-abi::source_resources$source_foreign::c_cache_generation_precharges_borrowed_lines_and_preserves_the_original_digest | experimental/ace/crates/abi/tests/source_resources.rs:3561 | package/target/完整module路径静态相符；未运行 |
| N139 | ace-abi::source_resources$source_foreign::c_cache_invalid_base64_releases_its_output_reservation_and_remains_a_miss | experimental/ace/crates/abi/tests/source_resources.rs:4258 | package/target/完整module路径静态相符；未运行 |
| N140 | ace-abi::source_resources$source_foreign::c_cache_malformed_entry_remains_a_miss_in_the_shared_resource_path | experimental/ace/crates/abi/tests/source_resources.rs:4333 | package/target/完整module路径静态相符；未运行 |
| N141 | ace-abi::source_resources$source_foreign::c_cache_recursive_listing_precharges_directory_growth_and_keeps_original_applicability | experimental/ace/crates/abi/tests/source_resources.rs:3388 | package/target/完整module路径静态相符；未运行 |
| N142 | ace-abi::source_resources$source_foreign::c_cache_source_identity_does_not_copy_the_whole_input_to_hash_it | experimental/ace/crates/abi/tests/source_resources.rs:4646 | package/target/完整module路径静态相符；未运行 |
| N143 | ace-abi::source_resources$source_foreign::c_fragment_outputs_share_the_request_account_and_store_hits_produce_nothing | experimental/ace/crates/abi/tests/source_resources.rs:5135 | package/target/完整module路径静态相符；未运行 |
| N144 | ace-abi::source_resources$source_foreign::c_generation_key_reserves_its_array_and_all_owned_fields_until_consumed | experimental/ace/crates/abi/tests/source_resources.rs:2774 | package/target/完整module路径静态相符；未运行 |
| N145 | ace-abi::source_resources$source_foreign::c_runtime_output_is_admitted_before_its_allocation_and_paid_exactly_once | experimental/ace/crates/abi/tests/source_resources.rs:4957 | package/target/完整module路径静态相符；未运行 |
| N146 | ace-abi::source_resources$source_foreign::candidate_directory_growth_and_path_copies_are_precharged_and_released_after_dedup | experimental/ace/crates/abi/tests/source_resources.rs:1247 | package/target/完整module路径静态相符；未运行 |
| N147 | ace-abi::source_resources$source_foreign::canonical_and_artifact_paths_precharge_real_long_junction_targets_and_release_temporaries | experimental/ace/crates/abi/tests/source_resources.rs:1350 | package/target/完整module路径静态相符；未运行 |
| N148 | ace-abi::source_resources$source_foreign::cold_interface_admission_reserves_before_production_and_retains_until_release | experimental/ace/crates/abi/tests/source_resources.rs:5376 | package/target/完整module路径静态相符；未运行 |
| N149 | ace-abi::source_resources$source_foreign::cold_toolchain_discovery_does_not_copy_the_compiler_image_to_hash_it | experimental/ace/crates/abi/tests/source_resources.rs:1607 | package/target/完整module路径静态相符；未运行 |
| N150 | ace-abi::source_resources$source_foreign::compiler_dependency_paths_reserve_native_probes_array_growth_and_unique_owned_text | experimental/ace/crates/abi/tests/source_resources.rs:3195 | package/target/完整module路径静态相符；未运行 |
| N151 | ace-abi::source_resources$source_foreign::compiler_object_file_read_reserves_before_allocation_and_retains_only_the_shared_output | experimental/ace/crates/abi/tests/source_resources.rs:4129 | package/target/完整module路径静态相符；未运行 |
| N152 | ace-abi::source_resources$source_foreign::compiler_output_text_borrows_valid_bytes_and_precharges_exact_lossy_storage | experimental/ace/crates/abi/tests/source_resources.rs:2884 | package/target/完整module路径静态相符；未运行 |
| N153 | ace-abi::source_resources$source_foreign::compiler_raw_capture_reserves_both_buffers_before_allocating_or_spawning | experimental/ace/crates/abi/tests/source_resources.rs:3118 | package/target/完整module路径静态相符；未运行 |
| N154 | ace-abi::source_resources$source_foreign::fileset_assembly_charges_true_products_only_and_keeps_legacy_bytes | experimental/ace/crates/abi/tests/source_resources.rs:5231 | package/target/完整module路径静态相符；未运行 |
| N155 | ace-abi::source_resources$source_foreign::frontend_session_metadata_is_admitted_once_and_covers_real_row_growth | experimental/ace/crates/abi/tests/source_resources.rs:5453 | package/target/完整module路径静态相符；未运行 |
| N156 | ace-abi::source_resources$source_foreign::late_mir_and_target_output_refusals_keep_native_bytes_charged_until_their_real_drop | experimental/ace/crates/abi/tests/source_resources.rs:977 | package/target/完整module路径静态相符；未运行 |
| N157 | ace-abi::source_resources$source_foreign::lexical_path_output_is_admitted_at_its_real_allocation_before_copying | experimental/ace/crates/abi/tests/source_resources.rs:1192 | package/target/完整module路径静态相符；未运行 |
| N158 | ace-abi::source_resources$source_foreign::native_codegen_workspace_is_admitted_across_the_cranelift_emit_window | experimental/ace/crates/abi/tests/source_resources.rs:5568 | package/target/完整module路径静态相符；未运行 |
| N159 | ace-abi::source_resources$source_foreign::native_foreign_call_reserves_declaration_and_arguments_and_releases_evaluation_scratch | experimental/ace/crates/abi/tests/source_resources.rs:2189 | package/target/完整module路径静态相符；未运行 |
| N160 | ace-abi::source_resources$source_foreign::native_lower_container_admission_covers_blocks_stack_slots_and_final_copy | experimental/ace/crates/abi/tests/source_resources.rs:1852 | package/target/完整module路径静态相符；未运行 |
| N161 | ace-abi::source_resources$source_foreign::native_lower_emission_reserves_instruction_and_span_capacity_before_allocation | experimental/ace/crates/abi/tests/source_resources.rs:1678 | package/target/完整module路径静态相符；未运行 |
| N162 | ace-abi::source_resources$source_foreign::native_runtime_copy_is_admitted_as_the_requests_first_owned_page | experimental/ace/crates/abi/tests/source_resources.rs:5056 | package/target/完整module路径静态相符；未运行 |
| N163 | ace-abi::source_resources$source_foreign::native_scalar_constant_pool_admits_initial_and_coexisting_growth_before_allocation | experimental/ace/crates/abi/tests/source_resources.rs:2066 | package/target/完整module路径静态相符；未运行 |
| N164 | ace-abi::source_resources$source_foreign::native_source_emission_quota_rejects_before_codegen_and_repairs_current_target | experimental/ace/crates/abi/tests/source_resources.rs:2592 | package/target/完整module路径静态相符；未运行 |
| N165 | ace-abi::source_resources$source_foreign::native_unit_metadata_admits_capability_and_signature_before_ownership_and_releases_indices | experimental/ace/crates/abi/tests/source_resources.rs:2327 | package/target/完整module路径静态相符；未运行 |
| N166 | ace-abi::source_resources$source_foreign::owned_payload_handoff_moves_bytes_and_keeps_aliases_charged_once | experimental/ace/crates/abi/tests/source_resources.rs:4590 | package/target/完整module路径静态相符；未运行 |
| N167 | ace-abi::source_resources$source_foreign::pointer_encoding_reserves_before_output_allocation_and_preserves_current_on_refusal | experimental/ace/crates/abi/tests/source_resources.rs:4808 | package/target/完整module路径静态相符；未运行 |
| N168 | ace-abi::source_resources$source_foreign::provider_binding_reserves_shared_budget_and_keeps_page_and_metadata_aliases_charged | experimental/ace/crates/abi/tests/source_resources.rs:4679 | package/target/完整module路径静态相符；未运行 |
| N169 | ace-abi::source_resources$source_foreign::provider_rebind_keeps_current_facts_without_allocating_whole_declaration_copies | experimental/ace/crates/abi/tests/source_resources.rs:3280 | package/target/完整module路径静态相符；未运行 |
| N170 | ace-abi::source_resources$source_foreign::rejected_shared_owner_keeps_its_fee_until_the_real_payload_drops_and_can_retry_attachment | experimental/ace/crates/abi/tests/source_resources.rs:1117 | package/target/完整module路径静态相符；未运行 |
| N171 | ace-abi::source_resources$source_foreign::source_crt_compile_uses_the_shared_capture_budget_and_releases_it_after_real_execution | experimental/ace/crates/abi/tests/source_resources.rs:2968 | package/target/完整module路径静态相符；未运行 |
| N172 | ace-abi::source_resources$source_foreign::source_identity_reserves_digest_and_final_text_before_their_actual_allocations | experimental/ace/crates/abi/tests/source_resources.rs:1300 | package/target/完整module路径静态相符；未运行 |
| N173 | ace-abi::source_resources$source_foreign::source_startup_wrapper_quota_stops_before_link_tools_and_preserves_accepted_payloads | experimental/ace/crates/abi/tests/source_resources.rs:1055 | package/target/完整module路径静态相符；未运行 |
| N174 | ace-abi::source_resources$source_foreign::tool_discovery_caps_both_streams_before_accepting_a_compiler_identity | experimental/ace/crates/abi/tests/source_resources.rs:1624 | package/target/完整module路径静态相符；未运行 |
| N175 | ace-abi::source_resources$source_foreign::tool_environment_snapshots_reserve_before_ownership_and_keep_utf16_values_lossless | experimental/ace/crates/abi/tests/source_resources.rs:3039 | package/target/完整module路径静态相符；未运行 |
| N176 | ace-abi::source_resources$source_foreign::toolchain_identity_and_sql_needle_precharge_streamed_text_and_preserve_original_escaping | experimental/ace/crates/abi/tests/source_resources.rs:1529 | package/target/完整module路径静态相符；未运行 |
| N177 | ace-av::conformance$av_resident::av_resident_cancel_surfaces_as_outcome | experimental/ace/crates/av/tests/cases/av_resident.rs:154 | package/target/完整module路径静态相符；未运行 |
| N178 | ace-av::conformance$av_resident::av_resident_candidate_queue_never_disturbs_the_run | experimental/ace/crates/av/tests/cases/av_resident.rs:236 | package/target/完整module路径静态相符；未运行 |
| N179 | ace-av::conformance$av_resident::av_resident_churn_retired_generations_release | experimental/ace/crates/av/tests/cases/av_resident.rs:557 | package/target/完整module路径静态相符；未运行 |
| N180 | ace-av::conformance$av_resident::av_resident_inflight_ancestor_defers_and_next_run_sees_new | experimental/ace/crates/av/tests/cases/av_resident.rs:653 | package/target/完整module路径静态相符；未运行 |
| N181 | ace-av::conformance$av_resident::av_resident_inflight_frame_stays_old_callee_resolves_new | experimental/ace/crates/av/tests/cases/av_resident.rs:444 | package/target/完整module路径静态相符；未运行 |
| N182 | ace-av::conformance$av_resident::av_resident_inflight_layout_change_never_panics | experimental/ace/crates/av/tests/cases/av_resident.rs:730 | package/target/完整module路径静态相符；未运行 |
| N183 | ace-av::conformance$av_resident::av_resident_no_swap_resume_stays_old | experimental/ace/crates/av/tests/cases/av_resident.rs:405 | package/target/完整module路径静态相符；未运行 |
| N184 | ace-av::conformance$av_resident::av_resident_online_swap_effective_at_call_boundary | experimental/ace/crates/av/tests/cases/av_resident.rs:359 | package/target/完整module路径静态相符；未运行 |
| N185 | ace-av::conformance$av_resident::av_resident_reload_visibility_v2_calibre | experimental/ace/crates/av/tests/cases/av_resident.rs:773 | package/target/完整module路径静态相符；未运行 |
| N186 | ace-av::conformance$av_resident::av_resident_restart_variant_is_red | experimental/ace/crates/av/tests/cases/av_resident.rs:99 | package/target/完整module路径静态相符；未运行 |
| N187 | ace-av::conformance$av_resident::av_resident_slice_invariance_and_chain_continuity | experimental/ace/crates/av/tests/cases/av_resident.rs:67 | package/target/完整module路径静态相符；未运行 |
| N188 | ace-av::conformance$av_resident::av_resident_stale_and_foreign_refused_at_safety_point | experimental/ace/crates/av/tests/cases/av_resident.rs:482 | package/target/完整module路径静态相符；未运行 |
| N189 | ace-av::conformance$av_resident::av_resident_stop_between_slices | experimental/ace/crates/av/tests/cases/av_resident.rs:181 | package/target/完整module路径静态相符；未运行 |
| N190 | ace-av::conformance$av_resident::av_resident_zero_fuel_slice_parks_without_progress | experimental/ace/crates/av/tests/cases/av_resident.rs:213 | package/target/完整module路径静态相符；未运行 |
| N191 | ace-source-cte::ace_source_cte$tests::t02_01_first_chain_real_av_freeze_backfill_av_end_42 | experimental/ace/crates/source-cte/src/tests.rs:100 | package/target/完整module路径静态相符；未运行 |
| N192 | ace-source-cte::ace_source_cte$tests::t02_02_callee_body_edit_invalidates_and_yields_43 | experimental/ace/crates/source-cte/src/tests.rs:134 | package/target/完整module路径静态相符；未运行 |
| N193 | ace-source-cte::ace_source_cte$tests::t02_03_hit_never_executes_and_forbid_av_proves_it | experimental/ace/crates/source-cte/src/tests.rs:170 | package/target/完整module路径静态相符；未运行 |
| N194 | ace-source-cte::ace_source_cte$tests::t02_04_bool_flip_is_strict | experimental/ace/crates/source-cte/src/tests.rs:221 | package/target/完整module路径静态相符；未运行 |
| N195 | ace-source-cte::ace_source_cte$tests::t02_05_helper_body_edit_invalidates_the_outer_ct | experimental/ace/crates/source-cte/src/tests.rs:264 | package/target/完整module路径静态相符；未运行 |
| N196 | ace-source-cte::ace_source_cte$tests::t02_06_restore_hits_with_zero_execution_over_the_real_store | experimental/ace/crates/source-cte/src/tests.rs:296 | package/target/完整module路径静态相符；未运行 |
| N197 | ace-source-cte::ace_source_cte$tests::t02_07_deck_free_lane_and_acyclic_dependencies | experimental/ace/crates/source-cte/src/tests.rs:436 | package/target/完整module路径静态相符；未运行 |
| N198 | ace-source-cte::ace_source_cte$tests::t02_08_unknown_and_off_surface_callees_are_controlled_rejections | experimental/ace/crates/source-cte/src/tests.rs:479 | package/target/完整module路径静态相符；未运行 |
| N199 | ace-source-cte::ace_source_cte$tests::t02_09_foreign_package_never_addresses_another_owners_page | experimental/ace/crates/source-cte/src/tests.rs:358 | package/target/完整module路径静态相符；未运行 |
| N200 | ace-source-cte::ace_source_cte$tests::t05_expect_inference_helper | experimental/ace/crates/source-cte/src/tests.rs:796 | package/target/完整module路径静态相符；未运行 |
| N201 | ace-source-cte::ace_source_cte$tests::t05_meta_evaluates_locally_and_becomes_real_av_args | experimental/ace/crates/source-cte/src/tests.rs:700 | package/target/完整module路径静态相符；未运行 |
| N202 | ace-source-cte::ace_source_cte$tests::t05_meta_only_demand_never_runs_the_av | experimental/ace/crates/source-cte/src/tests.rs:744 | package/target/完整module路径静态相符；未运行 |
| N203 | ace-source-cte::ace_source_cte$tests::t05_nested_hashes_are_demands_inner_first_single_flight | experimental/ace/crates/source-cte/src/tests.rs:582 | package/target/完整module路径静态相符；未运行 |
| N204 | ace-source-cte::ace_source_cte$tests::t05_plain_recursion_is_not_a_cycle | experimental/ace/crates/source-cte/src/tests.rs:675 | package/target/完整module路径静态相符；未运行 |
| N205 | ace-source-cte::ace_source_cte$tests::t05_query_cycle_is_named_with_the_chain | experimental/ace/crates/source-cte/src/tests.rs:631 | package/target/完整module路径静态相符；未运行 |
| N206 | ace-source-cte::ace_source_cte$tests::t05_shared_root_deadline_is_a_resource_state | experimental/ace/crates/source-cte/src/tests.rs:769 | package/target/完整module路径静态相符；未运行 |
| N207 | ace-source-cte::ace_source_cte$tests::t05_two_identical_sites_execute_once | experimental/ace/crates/source-cte/src/tests.rs:612 | package/target/完整module路径静态相符；未运行 |
| N208 | ace-source-cte::ace_source_cte$tests::t06_actual_trap_counts_the_attempt_and_publishes_nothing | experimental/ace/crates/source-cte/src/tests.rs:881 | package/target/完整module路径静态相符；未运行 |
| N209 | ace-source-cte::ace_source_cte$tests::t06_exec_log_correlates_keys_and_outcomes | experimental/ace/crates/source-cte/src/tests.rs:923 | package/target/完整module路径静态相符；未运行 |
| N210 | ace-source-cte::ace_source_cte$tests::t06_general_expression_strict_tag | experimental/ace/crates/source-cte/src/tests.rs:949 | package/target/完整module路径静态相符；未运行 |
| N211 | ace-source-cte::ace_source_cte$tests::t06_general_expression_wrapper_executes_for_real | experimental/ace/crates/source-cte/src/tests.rs:822 | package/target/完整module路径静态相符；未运行 |
| N212 | ace-source-cte::ace_source_cte$tests::t06_unexecuted_branch_trap_stays_latent | experimental/ace/crates/source-cte/src/tests.rs:859 | package/target/完整module路径静态相符；未运行 |
| N213 | ace-source-cte::ace_source_cte$tests::t07_backfill_product_compiles_and_av_end_observes_42_then_43 | experimental/ace/crates/source-cte/src/tests.rs:980 | package/target/完整module路径静态相符；未运行 |
| N214 | ace-source-cte::ace_source_cte$tests::t07_bool_hash_backfills_into_the_branch | experimental/ace/crates/source-cte/src/tests.rs:1049 | package/target/完整module路径静态相符；未运行 |
| N215 | ace-source-cte::ace_source_cte$tests::t07_second_compile_hits_the_frozen_page | experimental/ace/crates/source-cte/src/tests.rs:1073 | package/target/完整module路径静态相符；未运行 |
| N216 | ace-source-cte::ace_source_cte$tests::t08_chain_edits_miss_on_the_shared_store | experimental/ace/crates/source-cte/src/tests.rs:1248 | package/target/完整module路径静态相符；未运行 |
| N217 | ace-source-cte::ace_source_cte$tests::t08_inner_ct_flip_stales_the_instance_and_reexecutes_the_outer | experimental/ace/crates/source-cte/src/tests.rs:1175 | package/target/完整module路径静态相符；未运行 |
| N218 | ace-source-cte::ace_source_cte$tests::t08_instance_composition_choose_flips_with_the_template_body | experimental/ace/crates/source-cte/src/tests.rs:1119 | package/target/完整module路径静态相符；未运行 |
| N219 | ace-source-cte::ace_source_cte$tests::t09_bad_envelope_is_a_controlled_rejection_not_a_miss | experimental/ace/crates/source-cte/src/tests.rs:1391 | package/target/完整module路径静态相符；未运行 |
| N220 | ace-source-cte::ace_source_cte$tests::t09_missing_pages_fall_back_to_real_execution | experimental/ace/crates/source-cte/src/tests.rs:1449 | package/target/完整module路径静态相符；未运行 |
| N221 | ace-source-cte::ace_source_cte$tests::t09_r_window_metering_and_first_bad_then_repair | experimental/ace/crates/source-cte/src/tests.rs:1468 | package/target/完整module路径静态相符；未运行 |
| N222 | ace-source-cte::ace_source_cte$tests::t09_restore_then_first_edits | experimental/ace/crates/source-cte/src/tests.rs:1336 | package/target/完整module路径静态相符；未运行 |
| N223 | ace-source-cte::ace_source_cte$tests::t10_original_scale_baseline_with_ct_demand | experimental/ace/crates/source-cte/src/tests.rs:1537 | package/target/完整module路径静态相符；未运行 |
| N224 | ace-source-query::ace_source_query$tests::fact_page_preflight_refuses_oversized_payload_and_count_before_materialization | experimental/ace/crates/source-query/src/tests.rs:1048 | package/target/完整module路径静态相符；未运行 |
| N225 | ace-source-query::ace_source_query$tests::solver_attempts_include_cancel_and_resource_but_exclude_preflight_rejects | experimental/ace/crates/source-query/src/tests.rs:44 | package/target/完整module路径静态相符；未运行 |
| N226 | ace-source-query::ace_source_query$tests::sq01_one_body_yields_complete_structural_facts | experimental/ace/crates/source-query/src/tests.rs:86 | package/target/完整module路径静态相符；未运行 |
| N227 | ace-source-query::ace_source_query$tests::sq02_bindings_are_keyed_by_declaration_node | experimental/ace/crates/source-query/src/tests.rs:125 | package/target/完整module路径静态相符；未运行 |
| N228 | ace-source-query::ace_source_query$tests::sq03_body_only_edit_keeps_the_caller_warm | experimental/ace/crates/source-query/src/tests.rs:140 | package/target/完整module路径静态相符；未运行 |
| N229 | ace-source-query::ace_source_query$tests::sq04_callee_signature_change_invalidates_the_reader | experimental/ace/crates/source-query/src/tests.rs:160 | package/target/完整module路径静态相符；未运行 |
| N230 | ace-source-query::ace_source_query$tests::sq05_whitespace_drift_keeps_the_key_and_refreshes_spans | experimental/ace/crates/source-query/src/tests.rs:180 | package/target/完整module路径静态相符；未运行 |
| N231 | ace-source-query::ace_source_query$tests::sq06_negative_dependency_flips_when_the_name_appears | experimental/ace/crates/source-query/src/tests.rs:199 | package/target/完整module路径静态相符；未运行 |
| N232 | ace-source-query::ace_source_query$tests::sq07_stale_or_tampered_pages_are_refused | experimental/ace/crates/source-query/src/tests.rs:228 | package/target/完整module路径静态相符；未运行 |
| N233 | ace-source-query::ace_source_query$tests::sq08_cold_miss_solves_and_warm_hit_does_not | experimental/ace/crates/source-query/src/tests.rs:282 | package/target/完整module路径静态相符；未运行 |
| N234 | ace-source-query::ace_source_query$tests::sq09_outside_the_finite_surface_is_a_controlled_rejection | experimental/ace/crates/source-query/src/tests.rs:302 | package/target/完整module路径静态相符；未运行 |
| N235 | ace-source-query::ace_source_query$tests::sq10_signature_query_never_reads_the_body | experimental/ace/crates/source-query/src/tests.rs:324 | package/target/完整module路径静态相符；未运行 |
| N236 | ace-source-query::ace_source_query$tests::sq11_walk_ids_and_versions_are_deterministic | experimental/ace/crates/source-query/src/tests.rs:340 | package/target/完整module路径静态相符；未运行 |
| N237 | ace-source-query::ace_source_query$tests::sq12_apply_facts_drives_emission_and_verification | experimental/ace/crates/source-query/src/tests.rs:363 | package/target/完整module路径静态相符；未运行 |
| N238 | ace-source-query::ace_source_query$tests::sq13_apply_rejects_another_owners_page | experimental/ace/crates/source-query/src/tests.rs:394 | package/target/完整module路径静态相符；未运行 |
| N239 | ace-source-query::ace_source_query$tests::sq14_body_scoped_projection_materialises_only_the_target_body | experimental/ace/crates/source-query/src/tests.rs:413 | package/target/完整module路径静态相符；未运行 |
| N240 | ace-source-query::ace_source_query$tests::sq15_signature_catalog_is_declaration_only | experimental/ace/crates/source-query/src/tests.rs:450 | package/target/完整module路径静态相符；未运行 |
| N241 | ace-source-query::ace_source_query$tests::sq16_foreign_namespace_pages_are_never_reinterpreted | experimental/ace/crates/source-query/src/tests.rs:481 | package/target/完整module路径静态相符；未运行 |
| N242 | ace-source-query::ace_source_query$tests::sq17_session_signature_query_matches_the_catalog | experimental/ace/crates/source-query/src/tests.rs:538 | package/target/完整module路径静态相符；未运行 |
| N243 | ace-source-query::ace_source_query$tests::sq18_named_arguments_produce_clean_facts_and_a_distinct_key | experimental/ace/crates/source-query/src/tests.rs:555 | package/target/完整module路径静态相符；未运行 |
| N244 | ace-source-query::ace_source_query$tests::sq19_fact_supplied_bind_installs_query_types_and_verifies | experimental/ace/crates/source-query/src/tests.rs:585 | package/target/完整module路径静态相符；未运行 |
| N245 | ace-source-query::ace_source_query$tests::sq20_fact_supplied_bind_rejects_a_missing_fact | experimental/ace/crates/source-query/src/tests.rs:607 | package/target/完整module路径静态相符；未运行 |
| N246 | ace-source-query::ace_source_query$tests::sq21_complete_key_covers_every_axis_and_capabilities | experimental/ace/crates/source-query/src/tests.rs:630 | package/target/完整module路径静态相符；未运行 |
| N247 | ace-source-query::ace_source_query$tests::sq22_capability_change_invalidates_warm_pages | experimental/ace/crates/source-query/src/tests.rs:672 | package/target/完整module路径静态相符；未运行 |
| N248 | ace-source-query::ace_source_query$tests::sq23_mutual_recursion_is_not_a_query_cycle | experimental/ace/crates/source-query/src/tests.rs:690 | package/target/完整module路径静态相符；未运行 |
| N249 | ace-source-query::ace_source_query$tests::sq24_audit_reports_each_dependency_axis | experimental/ace/crates/source-query/src/tests.rs:729 | package/target/完整module路径静态相符；未运行 |
| N250 | ace-source-query::ace_source_query$tests::sq25_source_map_refresh_keeps_the_key_and_the_bind_works | experimental/ace/crates/source-query/src/tests.rs:764 | package/target/完整module路径静态相符；未运行 |
| N251 | ace-source-query::ace_source_query$tests::sq26_host_prepare_then_commit_switches_once | experimental/ace/crates/source-query/src/tests.rs:792 | package/target/完整module路径静态相符；未运行 |
| N252 | ace-source-query::ace_source_query$tests::sq27_pre_commit_failures_keep_the_old_record | experimental/ace/crates/source-query/src/tests.rs:822 | package/target/完整module路径静态相符；未运行 |
| N253 | ace-source-query::ace_source_query$tests::sq28_target_ready_is_attached_at_the_same_switch | experimental/ace/crates/source-query/src/tests.rs:872 | package/target/完整module路径静态相符；未运行 |
| N254 | ace-source-query::ace_source_query$tests::sq29_single_flight_shares_one_computation_per_key | experimental/ace/crates/source-query/src/tests.rs:884 | package/target/完整module路径静态相符；未运行 |
| N255 | ace-source-query::ace_source_query$tests::sq30_read_only_view_never_installs | experimental/ace/crates/source-query/src/tests.rs:936 | package/target/完整module路径静态相符；未运行 |
| N256 | ace-source-query::ace_source_query$tests::sq31_managed_store_sink_roundtrip_and_key_admission | experimental/ace/crates/source-query/src/tests.rs:961 | package/target/完整module路径静态相符；未运行 |
| N257 | ace-source-query::ace_source_query$tests::sq32_page_codec_rejects_foreign_truncated_and_tampered_pages | experimental/ace/crates/source-query/src/tests.rs:996 | package/target/完整module路径静态相符；未运行 |
| N258 | ace-source-query::ace_source_query$tests::sq33_published_page_is_a_project_root | experimental/ace/crates/source-query/src/tests.rs:1070 | package/target/完整module路径静态相符；未运行 |
| N259 | ace-source-query::ace_source_query$tests::sq34_restore_verify_refuses_missing_stale_or_partial_pages | experimental/ace/crates/source-query/src/tests.rs:1094 | package/target/完整module路径静态相符；未运行 |
| N260 | ace-source-query::ace_source_query$tests::sq35_restore_bind_uses_pages_only_and_needs_every_page | experimental/ace/crates/source-query/src/tests.rs:1128 | package/target/完整module路径静态相符；未运行 |
| N261 | ace-source-query::ace_source_query$tests::sq36_restore_then_first_edit_legs | experimental/ace/crates/source-query/src/tests.rs:1155 | package/target/完整module路径静态相符；未运行 |
| N262 | ace-source-query::ace_source_query$tests::sq37_fact_entry_produces_the_same_hir_as_the_legacy_entry | experimental/ace/crates/source-query/src/tests.rs:1211 | package/target/完整module路径静态相符；未运行 |
| N263 | ace-source-query::ace_source_query$tests::sq98_sink_duplicate_put_and_real_conflict_preserved | experimental/ace/crates/source-query/src/tests.rs:1302 | package/target/完整module路径静态相符；未运行 |
| N264 | ace-source-query::ace_source_query$tests::sq99_host_republish_same_semantics_is_idempotent | experimental/ace/crates/source-query/src/tests.rs:1254 | package/target/完整module路径静态相符；未运行 |
| N265 | ace-source-query::instances_restore_controls_935$anti_cold_miss_with_solver_disabled_is_red | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:203 | package/target/完整module路径静态相符；未运行 |
| N266 | ace-source-query::instances_restore_controls_935$anti_foreign_owner_page_refused | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:303 | package/target/完整module路径静态相符；未运行 |
| N267 | ace-source-query::instances_restore_controls_935$anti_tampered_instance_page_type_fails_real_bind | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:236 | package/target/完整module路径静态相符；未运行 |
| N268 | ace-source-query::instances_restore_controls_935$anti_warm_hit_with_solver_disabled_is_green | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:218 | package/target/完整module路径静态相符；未运行 |
| N269 | ace-source-query::instances_restore_controls_935$instance_pages_fit_the_frozen_budget_anchors | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:280 | package/target/完整module路径静态相符；未运行 |
| N270 | ace-source-query::instances_restore_controls_935$instances_restore_pages_only_bind_and_first_edits | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:107 | package/target/完整module路径静态相符；未运行 |
| N271 | ace-source-query::instances_restore_controls_935$t07_corrupted_instance_page_refused | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:367 | package/target/完整module路径静态相符；未运行 |
| N272 | ace-source-query::instances_restore_controls_935$t07_delete_last_caller_shrinks_instance_set | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:322 | package/target/完整module路径静态相符；未运行 |
| N273 | ace-source-query::instances_restore_controls_935$t08_multi_reader_instance_load_bounded_solves | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:408 | package/target/完整module路径静态相符；未运行 |
| N274 | ace-source-query::instances_restore_controls_935$t08_zero_budget_is_resource_state | experimental/ace/crates/source-query/tests/instances_restore_controls_935.rs:386 | package/target/完整module路径静态相符；未运行 |
| N275 | ace-source-query::restore_seam_929$facts_repair_after_the_bad_edit_restores_a_clean_page | experimental/ace/crates/source-query/tests/restore_seam_929.rs:162 | package/target/完整module路径静态相符；未运行 |
| N276 | ace-source-query::restore_seam_929$facts_restore_lane_runs_on_the_929_restore_seam | experimental/ace/crates/source-query/tests/restore_seam_929.rs:55 | package/target/完整module路径静态相符；未运行 |
| N277 | ace-syntax-core::conformance$incremental_local::incremental_local_body_edit_bounded_rescan_and_reparse | experimental/ace/crates/syntax-core/tests/cases/incremental_local.rs:27 | package/target/完整module路径静态相符；未运行 |
| N278 | ace-syntax-core::conformance$incremental_local::incremental_local_insert_at_doc_start_shifts_only_directory | experimental/ace/crates/syntax-core/tests/cases/incremental_local.rs:146 | package/target/完整module路径静态相符；未运行 |
| N279 | ace-syntax-core::conformance$incremental_local::incremental_local_lexer_real_reads_are_bounded | experimental/ace/crates/syntax-core/tests/cases/incremental_local.rs:203 | package/target/完整module路径静态相符；未运行 |
| N280 | ace-syntax-core::conformance$incremental_replay::incremental_replay_core_200_deterministic_mixed_edits | experimental/ace/crates/syntax-core/tests/cases/incremental_replay.rs:156 | package/target/完整module路径静态相符；未运行 |
| N281 | ace-syntax-core::conformance$incremental_replay::incremental_replay_explicit_fallback_reason_on_deletion_into_string | experimental/ace/crates/syntax-core/tests/cases/incremental_replay.rs:177 | package/target/完整module路径静态相符；未运行 |
| N282 | ace-syntax-core::conformance$incremental_replay::incremental_replay_late_results_never_overwrite_current_root | experimental/ace/crates/syntax-core/tests/cases/incremental_replay.rs:220 | package/target/完整module路径静态相符；未运行 |
| N283 | ace-syntax-core::conformance$incremental_replay::incremental_replay_mini_200_with_unterminated_fallback_visible | experimental/ace/crates/syntax-core/tests/cases/incremental_replay.rs:164 | package/target/完整module路径静态相符；未运行 |

</details>

<details>
<summary>U：342条scalar/Q4/composite/独立参考义务逐项对账</summary>

| 原TSV:行 | 原family/case | profile | 独立oracle | 源文件 | 源SHA256 | 原target/入口导航 | 结论 |
|---|---|---|---|---|---|---|---|
| scalar-obligations.tsv:2 | core-c02-paren-newline | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c02-paren-newline.at | 61a3581b2aa284b77cc130b49fc6f248570f366f255ae632cefb62d90660cf4e | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:3 | core-c05-ac-start-name | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c05-ac-start-name.at | 2af56576b302e3f514a2fedd1158cef90dd350b3b8d8c80eefb0c192c90860b3 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:4 | core-c06-exitprocess-name | core | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c06-exitprocess-name.at | a0c634f621b23c8d31d07a7cd0146f1e8be8be3b150ddbb5f19790c21be121a4 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:5 | core-d01-call-multiline | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d01-call-multiline.at | 050e612c075994a32672bd20ef3d639d986af7b848962c764b084aeac3b07fb0 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:6 | core-d02-paren-multiline | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d02-paren-multiline.at | b313a7731f13fae4c9aa81d9a4bae5da1f3e39c283e656906dcbf8822e80d7df | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:7 | core-d03-negative-multiline | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d03-negative-multiline.at | 3eced0b5aca2bd1f9f326d094fd83403b1a5581d7c8658ae2d7ef4ee5c05d4cc | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:8 | core-d04-nested-paren | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d04-nested-paren.at | 68a69974ce66d92588e5bd7c0766c080097dc3f740c64004a00dac3610ca4a83 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:9 | core-d05-call-no-tail | core | value:2 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d05-call-no-tail.at | c993eb16ce37098e11042da2f17fee346d85733b936937f27cb5ca9eb556849f | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:10 | core-d07-colliding-prefix | core | value:7 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d07-colliding-prefix.at | 1ed45f90bae1b7a235f49e8e2781da55344fc56126a060ca99b434d47029a4fd | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:11 | core-e01-nesting-16 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-e01-nesting-16.at | d687528fc35f12783b0ccd13d3fe09fc77f46a4095f136bdfdf0df34d707a608 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:12 | core-f04-parens-99 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-f04-parens-99.at | d035c0f22af60f44e278062e7bf364c1535e47abf63e5f332c0450612eeb07ca | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:13 | core-f11-add-chain-100 | core | value:300 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-f11-add-chain-100.at | 58f8576f878d2ebc443d072b2c3f4523da0f7234a80d9a8cdf165e0493465659 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:14 | core-g02-parens-200 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-g02-parens-200.at | b3e140b35957443297852ca93ea5c6c153d124f821e1e12e51861d798b35cfed | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:15 | core-m01-comma-args | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m01-comma-args.at | b0e649f7515bd664e59397cd6124c7d75a6bb3b62d709f21dd5fd13e04b5c32e | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:16 | core-m02-named-binop | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m02-named-binop.at | 6de19f12338f4aa65888a078b63ab31425c3b33c4f7f4df30bd4f9d2dec0576a | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:17 | core-m06-nested-call | core | value:192 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m06-nested-call.at | 26b1af73d665fe6b74930a514529791246cfc7a8aa39be4c7a0c925b583735d9 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:18 | core-m07-bool-arg | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m07-bool-arg.at | 56c32c40262409dc09c991fe8275b6b0b8ae0f1bef2791ad8c175c82cd94294a | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:19 | core-m10-paren-args | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m10-paren-args.at | c49cd0daac746223abe242d01760d37055d67002f1bddce7682c9dbb78e5f90b | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:20 | core-m22-named-newline | core | value:33 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m22-named-newline.at | f0ef5d6f2f48867e7c6e6c6f8fa6d505576a4d0a42ab26ba8c7c81470782ccb0 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:21 | core-p01-precedence | core | value:14 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p01-precedence.at | a09b91f416483d65d8d75a46a20d3e28645844f268496ef45207d2fba16d32d2 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:22 | core-p02-signed-pos | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p02-signed-pos.at | b6606ec99ca3b038da755a09fdd4cdb3ce8a8ac4002f5b8d6b2f5cabe1b463a4 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:23 | core-p03-bool-param | core | value:7 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p03-bool-param.at | aefe99819ee76bc82ae51933ea42dbc957c92d963cdfa9fd58d68ced8ea5a374 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:24 | core-p04-while | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p04-while.at | 556dcc51e11bf7b723e3b061db1c2d614e845770f41617857dfd0bd1425189bf | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:25 | core-p05-while-zero | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p05-while-zero.at | 1407ca813c5aeaff33017aafd342faaf6ad67a11418e554536097bfc14b3759d | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:26 | core-p06-continue | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p06-continue.at | 91435499a6b92ff818d280349c0d9b780078c4cf7faa7af074849c7cd81270c8 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:27 | core-p07-break-nearest | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p07-break-nearest.at | 6506162d22ce0d398111d739fe3815c017a5aebdd7709c631f0bff84ae856528 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:28 | core-p08-recursion | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p08-recursion.at | dcda62893983d3ba20e17760b11258cf3eb797816e3531781a93515832944a85 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:29 | core-p09-forward-sig | core | value:8 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p09-forward-sig.at | 6491415ef21f2a0204141fa9a637e38d6d5f240715d777eb0ddc9676e2e8fb62 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:30 | core-p10-shadow | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p10-shadow.at | 3fa87ea52dea38eb0ed750c0e4c87f03e966244ae2f17e5f235f198419309bea | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:31 | core-p11-min-max | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p11-min-max.at | 25bc370a6fcebac19fb5ea49d1a48f103f3da6c0f0ad82d8ba6b6fb8f49ad70d | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:32 | core-p12-unexecuted | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p12-unexecuted.at | 490f0a01093a4ca34d8fa039f1bb14b59f499c88dde60f9acdee571f7ac5b201 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:33 | core-p13-add-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p13-add-overflow.at | 3c8688fdacd28eecf38afb69647da79926dd95d998dfa0a9a7f5717cfee9d0f9 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:34 | core-p14-mul-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p14-mul-overflow.at | 8ce02fbc3c958173f08d1b1ed3015475b14ee6565cc84cfed001502bb59596c0 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:35 | core-p17-hetero-named | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p17-hetero-named.at | d47441320bbb727380548d50954d9d7fdc0c5e5cd9b67de34f4c9209d129ad8f | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:36 | core-p18-bool-return | core | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p18-bool-return.at | 8426728f973314cfab2c8709a69be072f649d1ff09514bae8218d2c9096bc63f | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:37 | core-p19-user-mark-a | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p19-user-mark-a.at | 1fd8da912590117a42d5825c0340e0f33d587cea8a7e6c8b4afec45c4c06b290 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:38 | core-p20-utf8-crlf-min | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p20-utf8-crlf-min.at | 08e75dceb6aea83f48a91d92e8a45e3afeedef3aea0a9d8cb8940187ce088e01 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:39 | sc-a-minus-1 | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-a-minus-1.at | 2f4496793e6a4db42d53f377a30031441843a0a8a08aaefb895995e9370dc641 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:40 | sc-binop-args | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-binop-args.at | 46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:41 | sc-cmp-matrix | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-cmp-matrix.at | 290961bc12288fd5e1523320bac43aeace7a1a770c941449aa3223cdbba50101 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:42 | sc-count-digits | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-count-digits.at | 716d9e896d8eac777d6b14d121b11929fe77972d82adef19ce79f88b576ce3c9 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:43 | sc-crlf | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-crlf.at | d1064b0a16aec0cbeab5a669353109e131d6ec304fb560a4dd449443d3674e3f | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:44 | sc-for-loops | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-for-loops.at | fa358a467646a370567d2c1fceb16dc6ea8b631d38e75b1c5199132b7ff1e38d | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:45 | sc-gcd-euclid | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-gcd-euclid.at | 6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:46 | sc-is-digit | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-is-digit.at | 6e6797b67aca64c6c1b4d44fc0f323d54585974cce6bab1288bdb20b22943923 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:47 | sc-short-circuit-contexts | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-contexts.at | b2cbf94dfd30be472f8362cae1efb68a8485e33550ba3bda35b978128be3a2ad | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:48 | sc-short-circuit-trace | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-trace.at | aa4baf2b834d37538c9ecb4000e080043035d09c0d2c594740e8448a0436bc49 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:49 | sc-short-circuit-trap | scalar | trap:DivByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-trap.at | d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:50 | ace-B01 | core | value:14 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B01.at | b7e5f25e46902be22c9bd3db35a334cb56e130e11cae090b6927314fc2d980c6 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:51 | ace-B02-signed-div-rem | scalar | value:-21 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B02-signed-div-rem.at | 7995c2cd010370655c76350afbaae9ba4cb814eec6b8946dfc600d2b9121b28d | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:52 | ace-B04-shortcircuit | scalar | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B04-shortcircuit.at | 4487dff4b270520faf0619f8aa0097573d51e5b5ee812ffca02488ab26415c12 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:53 | ace-B09-recursion | scalar | value:120 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B09-recursion.at | 66cfe6d5ec89f26568ad16359e6564fe4ef2bc8edc53b0f4175e5c5962fad5b6 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:54 | ace-B13-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B13-overflow.at | 54943ad037b05733996d41d88161225d7bcd0bdd0cb5a7ec9ba50941e3fd481e | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:55 | ace-B14-div-zero | scalar | trap:DivByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B14-div-zero.at | 1330a237463208d9cdaf958f174cbad20933842de12b86ecb685368f17677ffe | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:56 | bd-min-literal | scalar | value:-2147483648 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-literal.at | 4bf2b0a9f4395500fdb09405dd62a6125fab15add96280d1bc833e941fbbc90c | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:57 | bd-min-div-minus-1 | scalar | trap:DivOverflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-div-minus-1.at | c299f6864ca383aa6d98ff869f8517dd9a2d46017f281bf05f56b4a5c15174e1 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:58 | bd-min-rem-minus-1 | scalar | trap:RemOverflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-rem-minus-1.at | ce7a15de35446d1c375d47374adb52509287a313e32e6e67e33449a5fbca83a9 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:59 | bd-rem-by-zero | scalar | trap:RemByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-rem-by-zero.at | 8017f260721d0e494b87b6110b8448869bdb0daf071598756ba2fd4f919ba890 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:60 | bd-value70 | core | value:70 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-value70.at | aa8bda9f9973f904be028a6ed8ea71b6eecc9a3c55d74e5fd531f0fad3fa8e59 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:61 | bd-named-source-order | scalar | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-named-source-order.at | 94a9724690289d3143345d808530261a3564f5dba758c3dc1f1047039f167052 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| scalar-obligations.tsv:62 | bd-or-skips-rhs | scalar | value:8 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-or-skips-rhs.at | fef0c532b71ba0b21853b33a2415f35ad2a1cb2c6f8d3b83f8e41283e46b7876 | ace-a2x-worker::conformance::scalar_semantics::full_corpus_true_chain | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:2 | scalar/core-c02-paren-newline | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c02-paren-newline.at | 61a3581b2aa284b77cc130b49fc6f248570f366f255ae632cefb62d90660cf4e | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:3 | scalar/core-c05-ac-start-name | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c05-ac-start-name.at | 2af56576b302e3f514a2fedd1158cef90dd350b3b8d8c80eefb0c192c90860b3 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:4 | scalar/core-c06-exitprocess-name | core | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c06-exitprocess-name.at | a0c634f621b23c8d31d07a7cd0146f1e8be8be3b150ddbb5f19790c21be121a4 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:5 | scalar/core-d01-call-multiline | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d01-call-multiline.at | 050e612c075994a32672bd20ef3d639d986af7b848962c764b084aeac3b07fb0 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:6 | scalar/core-d02-paren-multiline | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d02-paren-multiline.at | b313a7731f13fae4c9aa81d9a4bae5da1f3e39c283e656906dcbf8822e80d7df | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:7 | scalar/core-d03-negative-multiline | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d03-negative-multiline.at | 3eced0b5aca2bd1f9f326d094fd83403b1a5581d7c8658ae2d7ef4ee5c05d4cc | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:8 | scalar/core-d04-nested-paren | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d04-nested-paren.at | 68a69974ce66d92588e5bd7c0766c080097dc3f740c64004a00dac3610ca4a83 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:9 | scalar/core-d05-call-no-tail | core | value:2 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d05-call-no-tail.at | c993eb16ce37098e11042da2f17fee346d85733b936937f27cb5ca9eb556849f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:10 | scalar/core-d07-colliding-prefix | core | value:7 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d07-colliding-prefix.at | 1ed45f90bae1b7a235f49e8e2781da55344fc56126a060ca99b434d47029a4fd | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:11 | scalar/core-e01-nesting-16 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-e01-nesting-16.at | d687528fc35f12783b0ccd13d3fe09fc77f46a4095f136bdfdf0df34d707a608 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:12 | scalar/core-f04-parens-99 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-f04-parens-99.at | d035c0f22af60f44e278062e7bf364c1535e47abf63e5f332c0450612eeb07ca | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:13 | scalar/core-f11-add-chain-100 | core | value:300 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-f11-add-chain-100.at | 58f8576f878d2ebc443d072b2c3f4523da0f7234a80d9a8cdf165e0493465659 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:14 | scalar/core-g02-parens-200 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-g02-parens-200.at | b3e140b35957443297852ca93ea5c6c153d124f821e1e12e51861d798b35cfed | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:15 | scalar/core-m01-comma-args | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m01-comma-args.at | b0e649f7515bd664e59397cd6124c7d75a6bb3b62d709f21dd5fd13e04b5c32e | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:16 | scalar/core-m02-named-binop | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m02-named-binop.at | 6de19f12338f4aa65888a078b63ab31425c3b33c4f7f4df30bd4f9d2dec0576a | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:17 | scalar/core-m06-nested-call | core | value:192 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m06-nested-call.at | 26b1af73d665fe6b74930a514529791246cfc7a8aa39be4c7a0c925b583735d9 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:18 | scalar/core-m07-bool-arg | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m07-bool-arg.at | 56c32c40262409dc09c991fe8275b6b0b8ae0f1bef2791ad8c175c82cd94294a | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:19 | scalar/core-m10-paren-args | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m10-paren-args.at | c49cd0daac746223abe242d01760d37055d67002f1bddce7682c9dbb78e5f90b | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:20 | scalar/core-m22-named-newline | core | value:33 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m22-named-newline.at | f0ef5d6f2f48867e7c6e6c6f8fa6d505576a4d0a42ab26ba8c7c81470782ccb0 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:21 | scalar/core-p01-precedence | core | value:14 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p01-precedence.at | a09b91f416483d65d8d75a46a20d3e28645844f268496ef45207d2fba16d32d2 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:22 | scalar/core-p02-signed-pos | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p02-signed-pos.at | b6606ec99ca3b038da755a09fdd4cdb3ce8a8ac4002f5b8d6b2f5cabe1b463a4 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:23 | scalar/core-p03-bool-param | core | value:7 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p03-bool-param.at | aefe99819ee76bc82ae51933ea42dbc957c92d963cdfa9fd58d68ced8ea5a374 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:24 | scalar/core-p04-while | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p04-while.at | 556dcc51e11bf7b723e3b061db1c2d614e845770f41617857dfd0bd1425189bf | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:25 | scalar/core-p05-while-zero | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p05-while-zero.at | 1407ca813c5aeaff33017aafd342faaf6ad67a11418e554536097bfc14b3759d | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:26 | scalar/core-p06-continue | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p06-continue.at | 91435499a6b92ff818d280349c0d9b780078c4cf7faa7af074849c7cd81270c8 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:27 | scalar/core-p07-break-nearest | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p07-break-nearest.at | 6506162d22ce0d398111d739fe3815c017a5aebdd7709c631f0bff84ae856528 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:28 | scalar/core-p08-recursion | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p08-recursion.at | dcda62893983d3ba20e17760b11258cf3eb797816e3531781a93515832944a85 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:29 | scalar/core-p09-forward-sig | core | value:8 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p09-forward-sig.at | 6491415ef21f2a0204141fa9a637e38d6d5f240715d777eb0ddc9676e2e8fb62 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:30 | scalar/core-p10-shadow | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p10-shadow.at | 3fa87ea52dea38eb0ed750c0e4c87f03e966244ae2f17e5f235f198419309bea | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:31 | scalar/core-p11-min-max | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p11-min-max.at | 25bc370a6fcebac19fb5ea49d1a48f103f3da6c0f0ad82d8ba6b6fb8f49ad70d | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:32 | scalar/core-p12-unexecuted | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p12-unexecuted.at | 490f0a01093a4ca34d8fa039f1bb14b59f499c88dde60f9acdee571f7ac5b201 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:33 | scalar/core-p13-add-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p13-add-overflow.at | 3c8688fdacd28eecf38afb69647da79926dd95d998dfa0a9a7f5717cfee9d0f9 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:34 | scalar/core-p14-mul-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p14-mul-overflow.at | 8ce02fbc3c958173f08d1b1ed3015475b14ee6565cc84cfed001502bb59596c0 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:35 | scalar/core-p17-hetero-named | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p17-hetero-named.at | d47441320bbb727380548d50954d9d7fdc0c5e5cd9b67de34f4c9209d129ad8f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:36 | scalar/core-p18-bool-return | core | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p18-bool-return.at | 8426728f973314cfab2c8709a69be072f649d1ff09514bae8218d2c9096bc63f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:37 | scalar/core-p19-user-mark-a | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p19-user-mark-a.at | 1fd8da912590117a42d5825c0340e0f33d587cea8a7e6c8b4afec45c4c06b290 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:38 | scalar/core-p20-utf8-crlf-min | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p20-utf8-crlf-min.at | 08e75dceb6aea83f48a91d92e8a45e3afeedef3aea0a9d8cb8940187ce088e01 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:39 | scalar/sc-a-minus-1 | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-a-minus-1.at | 2f4496793e6a4db42d53f377a30031441843a0a8a08aaefb895995e9370dc641 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:40 | scalar/sc-binop-args | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-binop-args.at | 46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:41 | scalar/sc-cmp-matrix | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-cmp-matrix.at | 290961bc12288fd5e1523320bac43aeace7a1a770c941449aa3223cdbba50101 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:42 | scalar/sc-count-digits | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-count-digits.at | 716d9e896d8eac777d6b14d121b11929fe77972d82adef19ce79f88b576ce3c9 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:43 | scalar/sc-crlf | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-crlf.at | d1064b0a16aec0cbeab5a669353109e131d6ec304fb560a4dd449443d3674e3f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:44 | scalar/sc-for-loops | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-for-loops.at | fa358a467646a370567d2c1fceb16dc6ea8b631d38e75b1c5199132b7ff1e38d | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:45 | scalar/sc-gcd-euclid | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-gcd-euclid.at | 6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:46 | scalar/sc-is-digit | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-is-digit.at | 6e6797b67aca64c6c1b4d44fc0f323d54585974cce6bab1288bdb20b22943923 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:47 | scalar/sc-short-circuit-contexts | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-contexts.at | b2cbf94dfd30be472f8362cae1efb68a8485e33550ba3bda35b978128be3a2ad | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:48 | scalar/sc-short-circuit-trace | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-trace.at | aa4baf2b834d37538c9ecb4000e080043035d09c0d2c594740e8448a0436bc49 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:49 | scalar/sc-short-circuit-trap | scalar | trap:DivByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-trap.at | d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:50 | scalar/ace-B01 | core | value:14 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B01.at | b7e5f25e46902be22c9bd3db35a334cb56e130e11cae090b6927314fc2d980c6 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:51 | scalar/ace-B02-signed-div-rem | scalar | value:-21 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B02-signed-div-rem.at | 7995c2cd010370655c76350afbaae9ba4cb814eec6b8946dfc600d2b9121b28d | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:52 | scalar/ace-B04-shortcircuit | scalar | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B04-shortcircuit.at | 4487dff4b270520faf0619f8aa0097573d51e5b5ee812ffca02488ab26415c12 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:53 | scalar/ace-B09-recursion | scalar | value:120 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B09-recursion.at | 66cfe6d5ec89f26568ad16359e6564fe4ef2bc8edc53b0f4175e5c5962fad5b6 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:54 | scalar/ace-B13-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B13-overflow.at | 54943ad037b05733996d41d88161225d7bcd0bdd0cb5a7ec9ba50941e3fd481e | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:55 | scalar/ace-B14-div-zero | scalar | trap:DivByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B14-div-zero.at | 1330a237463208d9cdaf958f174cbad20933842de12b86ecb685368f17677ffe | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:56 | scalar/bd-min-literal | scalar | value:-2147483648 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-literal.at | 4bf2b0a9f4395500fdb09405dd62a6125fab15add96280d1bc833e941fbbc90c | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:57 | scalar/bd-min-div-minus-1 | scalar | trap:DivOverflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-div-minus-1.at | c299f6864ca383aa6d98ff869f8517dd9a2d46017f281bf05f56b4a5c15174e1 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:58 | scalar/bd-min-rem-minus-1 | scalar | trap:RemOverflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-rem-minus-1.at | ce7a15de35446d1c375d47374adb52509287a313e32e6e67e33449a5fbca83a9 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:59 | scalar/bd-rem-by-zero | scalar | trap:RemByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-rem-by-zero.at | 8017f260721d0e494b87b6110b8448869bdb0daf071598756ba2fd4f919ba890 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:60 | scalar/bd-value70 | core | value:70 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-value70.at | aa8bda9f9973f904be028a6ed8ea71b6eecc9a3c55d74e5fd531f0fad3fa8e59 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:61 | scalar/bd-named-source-order | scalar | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-named-source-order.at | 94a9724690289d3143345d808530261a3564f5dba758c3dc1f1047039f167052 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:62 | scalar/bd-or-skips-rhs | scalar | value:8 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-or-skips-rhs.at | fef0c532b71ba0b21853b33a2415f35ad2a1cb2c6f8d3b83f8e41283e46b7876 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:63 | record-text-cert-925/M_COPY_SNAP | record | value:221 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_COPY_SNAP.at | 2db804e9f664ab1fa1afa49bf8e9d72ee0ac503e33850804656a52f211a4a09a | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:64 | record-text-cert-925/M_MULTI_MUT | record | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_MULTI_MUT.at | 6e5b0eba1f87742d2fb7c693025d4321a1bcb0447e795e98f0fc3bf144d61844 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:65 | record-text-cert-925/M_VIEW_RECORD | record | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_VIEW_RECORD.at | 18d46146b19f52b49aaee46a9e139901431c48e9b40b52fc6fbc0959600b6926 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:66 | record-text-cert-925/M_MUT_TEXT | text-record | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_MUT_TEXT.at | 2ab16069ee964db74e810b0183043a8c4d69b35b17ff0c79953a3fe0b7939a7f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:67 | record-text-cert-925/M_BORROW_RETURN | text | value:2 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_BORROW_RETURN.at | 70c6bb77ef2cf101498a5beadcb68d131fc7df948ecef948226a805064c45a08 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:68 | record-text-cert-925/M_OWNED_RETURN | text | value:22 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_OWNED_RETURN.at | 39559d922d3f2cd402d5ba875a3c5b75d739e91ee983668971c110ad21a88477 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:69 | record-text-cert-925/M_NAMED_CTOR | record | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_NAMED_CTOR.at | adf0ad863a47130c9d393120dc60c78695a102c6b6a6d9c4af4c854923f93d0c | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:70 | record-text-cert-925/M_NAME_COLLISION | record | value:137 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_NAME_COLLISION.at | 3d51468a071500924786c6034624df4a98f96a1c2bdd41a217a85ebd19f87bc4 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:71 | record-text-cert-925/T_CHAR_AT_NEG | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_CHAR_AT_NEG.at | af394c415404d5650fd4665fc9164f29eb8082b67fcd51f0ccf9d51a6ed1e164 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:72 | record-text-cert-925/T_SLICE_OOB | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_SLICE_OOB.at | 0954832b3bc4f2d166b2ce40da91495c70596fc48cd1c85c1e94440187b32763 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:73 | record-text-cert-925/T_SURROGATE | text | trap:InvalidScalar | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_SURROGATE.at | 4f9a85fa9398ca0034499ece2ef13bae8d2d403af833dad019a3642723b9c7bf | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:74 | text-char-925/v03_len_scalars_bytes | text | value:506 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_len_scalars_bytes.at | a3b5e245f557a16c76c604ebcc7d2e36dc8cc4ec801e0be13d0c0ad14ac89e65 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:75 | text-char-925/v03_nul_data | text | value:33 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_nul_data.at | 8ba399015a10bf8d46152ed3a24b808e2beb82b5de051ba03081281780514877 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:76 | text-char-925/v03_eq_full_bytes | text | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_eq_full_bytes.at | c88f8407c7235a1042d062575aa52c68a258eca8752e49844620ab3dfd1358c7 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:77 | text-char-925/v03_char_at | text | value:233 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_char_at.at | 80c4571a88581aad8837c46b98aa52b70eb9b5582182e761cfa4f7e780d15285 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:78 | text-char-925/v03_char_at_negative | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_char_at_negative.at | af394c415404d5650fd4665fc9164f29eb8082b67fcd51f0ccf9d51a6ed1e164 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:79 | text-char-925/v03_slice_oob | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_slice_oob.at | 0954832b3bc4f2d166b2ce40da91495c70596fc48cd1c85c1e94440187b32763 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:80 | text-char-925/v03_slice_scalars | text | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_slice_scalars.at | 6b76f6f5f7788a973513e4695e9a1ab6a445a797d844d7d5ea124120be091c71 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:81 | text-char-925/v03_codepoint_ok | text | value:20013 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_codepoint_ok.at | c285dbc9906d68c47533f9dfa6b78162cb6c33ff5cf786af0af77f3998ca6a0a | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:82 | text-char-925/v03_codepoint_surrogate | text | trap:InvalidScalar | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_codepoint_surrogate.at | 4f9a85fa9398ca0034499ece2ef13bae8d2d403af833dad019a3642723b9c7bf | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:83 | text-char-925/v03_concat_clone | text | value:112 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_concat_clone.at | e30f0a07b816273899687fe5fdb67d2482ac76cea1d3349d96aa38e063d68d2a | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:84 | collections-926/L01 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L01.at | 8247b7e7688e1931a36bc09eb4ec2cc789a3b099237d20d5dc8095a71e76a4f5 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:85 | collections-926/L02 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L02.at | 8276953bd3fe700fe44593092117d2b93b691aff0747bfb0f1099cc2008432e2 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:86 | collections-926/L03 | collections | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L03.at | 299034a89f81f97a0ab9ffa71f278d05262db5747a77ec5a460dbdbfce070339 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:87 | collections-926/L04 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L04.at | a79b03e53e6e82bda9000a1b6cd1bcfae695d4cc427928261e9d13bda19b0b99 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:88 | collections-926/L05 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L05.at | 3c55fc65f45724c072aaf7a772cfdc7d52da0d4f117ec6f23841795551629149 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:89 | collections-926/L06 | collections | trap:ListIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L06.at | 9b4aaa8834835283629077af431d7864d9c88a732060cebbd50fe24921c21be8 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:90 | collections-926/L07 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L07.at | 8aa9b61d53b3f0ce715a98d3c6ff1b0a7fd5f75ede2ea1dc132c650e97577aec | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:91 | collections-926/V01 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V01.at | 900acc88f43f9505b91fa6c4096642589fa7e3ca770833106bd552c481857dee | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:92 | collections-926/V02 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V02.at | 281396b1a3d5ea129a5c4e34672287975bac87ed7b543af7fbc828ce7869259f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:93 | collections-926/V03 | collections | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V03.at | 99bc231454ba7426c61828525e58dfd2514c6be54249ccbcab43d9c1e0ebf532 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:94 | collections-926/V04 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V04.at | c9303cad597133e0a517b70d5f0693a8a4961199a3148b82abffe039e22839cc | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:95 | collections-926/V05 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V05.at | 734c211d1ea0e62d52f14617369a77c975c0a23c915e3ed9dff91fa1f4c7c0c3 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:96 | collections-926/V06 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V06.at | 7145928b2c570c848f6ca7746545d1ca3d31790045855080d05db4c3d4112383 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:97 | collections-926/V07 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V07.at | 04844f208d458a043dd6ef2d2202a8ffb09789010d750d075cc164ee59135478 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:98 | collections-926/LSTR864 | collections | value:51 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--LSTR864.at | f36e92cfa92d4bc4240995c75e312fa84076e7bb7d983b75141dadb7e70524ff | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:99 | collections-926/FIN864A | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--FIN864A.at | 39524c0208966b1aca826afbc1b6c3898526f9373275fb62d7e8dd13662ceb81 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:100 | collections-926/FIN864B | collections | value:53 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--FIN864B.at | d0ce9a71534a37a289eea7ff22307d05414042420548263b3d1a4b4c431286b1 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:101 | collections-926/N889-list-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-list-variant.at | b90d64918e20ea8bb7a702871067cf177064adab604cad4af1205a3f6fb33a7e | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:102 | collections-926/N889-nested-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-nested-variant.at | 4b6433b03528d986ef67edf9d7420409f8e7d64ebccafceadf9fc4d23da6e61f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:103 | collections-926/N889-optional-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-optional-variant.at | e800cab21196d7e848e3ad19bc7af5903f8534e55d52d064695ab38c5c781b84 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:104 | record-text-base/r07 | record | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/R07.at | 6d07888a9383b7e8a7d4d6a50dabc961e0e2f35137cb9823c5cf0463e43b6c28 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:105 | record-text-base/r08 | record | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/R08.at | b2ea23e7f6acb41100ddeee036a648f5cc92b6857608e66f5fbd1f4d5ef9c895 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:106 | record-text-base/t03 | text-record | value:23 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/T03.at | f895f9d1577bbea6850156b2f3364399f1e6d7b2257dad491bcb8f2d4e02c542 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:107 | record-text-base/t05 | text | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/T05.at | 84541676753c15d06dc263efa57105501d87a07907102ae989de5020daf47530 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:108 | record-text-base/c04 | text-record | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/C04.at | 56f052c7192c531eb118e3732059bda0260ef5c2b747d62bb0953d1f566d3558 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:109 | source-query/leaf42 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-query-934/src/leaf42.at | 827926839c09c494915fa161b7e4689294aaf4171af82fb24325df413d4f7b9d | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:110 | source-query/leaf43 | scalar | value:43 | crates/auto-lang/test/cases/ace-m1/source-query-934/src/leaf43.at | bf0fbcaa66ae6254b408e8aa68b04e5db81c96bbf8140cfa5e555ce34ee05aa6 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:111 | source-query/named7 | scalar | value:7 | crates/auto-lang/test/cases/ace-m1/source-query-934/src/named7.at | f4f3c0db215ae3df1de1e2c66021d6c9c9a80d538ece1b33c9121a96f9ab080e | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:112 | source-query/trace-ab7 | scalar | value:7 | crates/auto-lang/test/cases/ace-m1/source-query-934/src/trace-ab7.at | ab28e4cbde917e8a3b4dd2e36761337191fd4cea2a1734d8a04395f345e680d4 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:113 | source-query/neg-bool-plus | scalar | diagnostic:source.type-mismatch | crates/auto-lang/test/cases/ace-m1/source-query-934/src/neg-bool-plus.at | a28683d11a70fa19dfd34136e1201f01bce63303d7e7db1c348c2f2f42cb7f77 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:114 | source-query/neg-not-int | scalar | diagnostic:source.type-mismatch | crates/auto-lang/test/cases/ace-m1/source-query-934/src/neg-not-int.at | b94d7445035790edf181010cd33d2a7dc7dc6808edb2c983078a05d569e27860 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:115 | source-instances/choose42 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-instances-935/src/choose42.at | 2feecf61cb0733144f02de65104d3fe6ee98261ab6c857e2448680794cdf8234 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:116 | source-instances/choose0 | scalar | value:0 | crates/auto-lang/test/cases/ace-m1/source-instances-935/src/choose0.at | 286e40bb5540cae9e532ce44c2f8ad689be03f82429bebf16584ad0759b315f9 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:117 | source-instances/expected-type | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-instances-935/src/expected-type.at | cc8aa9ebf566f26e2fb881ace116de4cae124e1e5e7d24daaae2b0128a644695 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:118 | source-instances/cross-wrap | scalar | value:40 | crates/auto-lang/test/cases/ace-m1/source-instances-935/src/cross-wrap.at | 42ec99665f6841b6a9f9abf38e7413062ab9466f131dbc3f60edcbe1b816b896 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:119 | source-instances/recursion | scalar | value:40 | crates/auto-lang/test/cases/ace-m1/source-instances-935/src/recursion.at | 48beb764c78028745ed54ec3f3cc7497fc363b5730938dde98192f7aee16b282 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:120 | source-instances/neg-hetero | scalar | diagnostic:type-mismatch | crates/auto-lang/test/cases/ace-m1/source-instances-935/src/neg-hetero.at | 8ef1a995b76c1446b43c2ca0435c7f5fdee00e1973481df5822226b7de066f14 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:121 | source-cte/sum42 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum42.at | 97d6cc23e97c6c349a5b5ffe55ea2aa24a6e4af6c1b8f9bb4a9676a04f90a6d4 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:122 | source-cte/callee-body43 | scalar | value:43 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum43.at | 9864bc9e5fbdc420df8912b65b4b0229ff6642077b00c6790d15515136ae9f92 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:123 | source-cte/bool | scalar | value:7 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/boollit.at | 799da4a2a4ac5104e08ea433b57eaeb600f7374b752ea50cf56f34a7a7ff652f | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:124 | source-cte/meta-fields2 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta2.at | 0485f6ed33a0c20b0e3576bc11e9a58957d1784354ce04c2aaed1f92ba323adf | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:125 | source-cte/meta-fields3 | scalar | value:43 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta3.at | 257a85a7583606441e0dc85515a90c0b93d1cf9352c8d93b24d10ae76a6f9010 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:126 | source-cte/choose-instance42 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/choose42.at | ad48036cd25b00947da60879ec90bd1cc6a70605f5a48f6fbec728b876b587f4 | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| q4-source-contracts.tsv:127 | source-cte/child-ct-chain40 | scalar | value:40 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/pick40.at | 7e4cc347a7d62c10758caf0de161ceecd614d6c97a957df6f826b17540b5742d | ["AC-native","AV","ACE-A2R-HIR","ACE-A2C-HIR"] | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:2 | record-text-cert-925/M_COPY_SNAP | record | value:221 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_COPY_SNAP.at | 2db804e9f664ab1fa1afa49bf8e9d72ee0ac503e33850804656a52f211a4a09a | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:3 | record-text-cert-925/M_MULTI_MUT | record | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_MULTI_MUT.at | 6e5b0eba1f87742d2fb7c693025d4321a1bcb0447e795e98f0fc3bf144d61844 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:4 | record-text-cert-925/M_VIEW_RECORD | record | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_VIEW_RECORD.at | 18d46146b19f52b49aaee46a9e139901431c48e9b40b52fc6fbc0959600b6926 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:5 | record-text-cert-925/M_MUT_TEXT | text-record | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_MUT_TEXT.at | 2ab16069ee964db74e810b0183043a8c4d69b35b17ff0c79953a3fe0b7939a7f | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:6 | record-text-cert-925/M_BORROW_RETURN | text | value:2 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_BORROW_RETURN.at | 70c6bb77ef2cf101498a5beadcb68d131fc7df948ecef948226a805064c45a08 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:7 | record-text-cert-925/M_OWNED_RETURN | text | value:22 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_OWNED_RETURN.at | 39559d922d3f2cd402d5ba875a3c5b75d739e91ee983668971c110ad21a88477 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:8 | record-text-cert-925/M_NAMED_CTOR | record | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_NAMED_CTOR.at | adf0ad863a47130c9d393120dc60c78695a102c6b6a6d9c4af4c854923f93d0c | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:9 | record-text-cert-925/M_NAME_COLLISION | record | value:137 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_NAME_COLLISION.at | 3d51468a071500924786c6034624df4a98f96a1c2bdd41a217a85ebd19f87bc4 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:10 | record-text-cert-925/T_CHAR_AT_NEG | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_CHAR_AT_NEG.at | af394c415404d5650fd4665fc9164f29eb8082b67fcd51f0ccf9d51a6ed1e164 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:11 | record-text-cert-925/T_SLICE_OOB | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_SLICE_OOB.at | 0954832b3bc4f2d166b2ce40da91495c70596fc48cd1c85c1e94440187b32763 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:12 | record-text-cert-925/T_SURROGATE | text | trap:InvalidScalar | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_SURROGATE.at | 4f9a85fa9398ca0034499ece2ef13bae8d2d403af833dad019a3642723b9c7bf | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:13 | text-char-925/v03_len_scalars_bytes | text | value:506 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_len_scalars_bytes.at | a3b5e245f557a16c76c604ebcc7d2e36dc8cc4ec801e0be13d0c0ad14ac89e65 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:14 | text-char-925/v03_nul_data | text | value:33 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_nul_data.at | 8ba399015a10bf8d46152ed3a24b808e2beb82b5de051ba03081281780514877 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:15 | text-char-925/v03_eq_full_bytes | text | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_eq_full_bytes.at | c88f8407c7235a1042d062575aa52c68a258eca8752e49844620ab3dfd1358c7 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:16 | text-char-925/v03_char_at | text | value:233 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_char_at.at | 80c4571a88581aad8837c46b98aa52b70eb9b5582182e761cfa4f7e780d15285 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:17 | text-char-925/v03_char_at_negative | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_char_at_negative.at | af394c415404d5650fd4665fc9164f29eb8082b67fcd51f0ccf9d51a6ed1e164 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:18 | text-char-925/v03_slice_oob | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_slice_oob.at | 0954832b3bc4f2d166b2ce40da91495c70596fc48cd1c85c1e94440187b32763 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:19 | text-char-925/v03_slice_scalars | text | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_slice_scalars.at | 6b76f6f5f7788a973513e4695e9a1ab6a445a797d844d7d5ea124120be091c71 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:20 | text-char-925/v03_codepoint_ok | text | value:20013 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_codepoint_ok.at | c285dbc9906d68c47533f9dfa6b78162cb6c33ff5cf786af0af77f3998ca6a0a | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:21 | text-char-925/v03_codepoint_surrogate | text | trap:InvalidScalar | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_codepoint_surrogate.at | 4f9a85fa9398ca0034499ece2ef13bae8d2d403af833dad019a3642723b9c7bf | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:22 | text-char-925/v03_concat_clone | text | value:112 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_concat_clone.at | e30f0a07b816273899687fe5fdb67d2482ac76cea1d3349d96aa38e063d68d2a | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:23 | collections-926/L01 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L01.at | 8247b7e7688e1931a36bc09eb4ec2cc789a3b099237d20d5dc8095a71e76a4f5 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:24 | collections-926/L02 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L02.at | 8276953bd3fe700fe44593092117d2b93b691aff0747bfb0f1099cc2008432e2 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:25 | collections-926/L03 | collections | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L03.at | 299034a89f81f97a0ab9ffa71f278d05262db5747a77ec5a460dbdbfce070339 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:26 | collections-926/L04 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L04.at | a79b03e53e6e82bda9000a1b6cd1bcfae695d4cc427928261e9d13bda19b0b99 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:27 | collections-926/L05 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L05.at | 3c55fc65f45724c072aaf7a772cfdc7d52da0d4f117ec6f23841795551629149 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:28 | collections-926/L06 | collections | trap:ListIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L06.at | 9b4aaa8834835283629077af431d7864d9c88a732060cebbd50fe24921c21be8 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:29 | collections-926/L07 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L07.at | 8aa9b61d53b3f0ce715a98d3c6ff1b0a7fd5f75ede2ea1dc132c650e97577aec | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:30 | collections-926/V01 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V01.at | 900acc88f43f9505b91fa6c4096642589fa7e3ca770833106bd552c481857dee | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:31 | collections-926/V02 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V02.at | 281396b1a3d5ea129a5c4e34672287975bac87ed7b543af7fbc828ce7869259f | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:32 | collections-926/V03 | collections | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V03.at | 99bc231454ba7426c61828525e58dfd2514c6be54249ccbcab43d9c1e0ebf532 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:33 | collections-926/V04 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V04.at | c9303cad597133e0a517b70d5f0693a8a4961199a3148b82abffe039e22839cc | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:34 | collections-926/V05 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V05.at | 734c211d1ea0e62d52f14617369a77c975c0a23c915e3ed9dff91fa1f4c7c0c3 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:35 | collections-926/V06 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V06.at | 7145928b2c570c848f6ca7746545d1ca3d31790045855080d05db4c3d4112383 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:36 | collections-926/V07 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V07.at | 04844f208d458a043dd6ef2d2202a8ffb09789010d750d075cc164ee59135478 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:37 | collections-926/LSTR864 | collections | value:51 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--LSTR864.at | f36e92cfa92d4bc4240995c75e312fa84076e7bb7d983b75141dadb7e70524ff | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:38 | collections-926/FIN864A | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--FIN864A.at | 39524c0208966b1aca826afbc1b6c3898526f9373275fb62d7e8dd13662ceb81 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:39 | collections-926/FIN864B | collections | value:53 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--FIN864B.at | d0ce9a71534a37a289eea7ff22307d05414042420548263b3d1a4b4c431286b1 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:40 | collections-926/N889-list-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-list-variant.at | b90d64918e20ea8bb7a702871067cf177064adab604cad4af1205a3f6fb33a7e | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:41 | collections-926/N889-nested-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-nested-variant.at | 4b6433b03528d986ef67edf9d7420409f8e7d64ebccafceadf9fc4d23da6e61f | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| composite-obligations.tsv:42 | collections-926/N889-optional-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-optional-variant.at | e800cab21196d7e848e3ad19bc7af5903f8534e55d52d064695ab38c5c781b84 | ace-a2x | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:2 | v03_len_scalars_bytes | text | value:506 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_len_scalars_bytes.at | a3b5e245f557a16c76c604ebcc7d2e36dc8cc4ec801e0be13d0c0ad14ac89e65 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:3 | v03_nul_data | text | value:33 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_nul_data.at | 8ba399015a10bf8d46152ed3a24b808e2beb82b5de051ba03081281780514877 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:4 | v03_eq_full_bytes | text | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_eq_full_bytes.at | c88f8407c7235a1042d062575aa52c68a258eca8752e49844620ab3dfd1358c7 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:5 | v03_char_at | text | value:233 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_char_at.at | 80c4571a88581aad8837c46b98aa52b70eb9b5582182e761cfa4f7e780d15285 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:6 | v03_char_at_negative | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_char_at_negative.at | af394c415404d5650fd4665fc9164f29eb8082b67fcd51f0ccf9d51a6ed1e164 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:7 | v03_slice_oob | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_slice_oob.at | 0954832b3bc4f2d166b2ce40da91495c70596fc48cd1c85c1e94440187b32763 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:8 | v03_slice_scalars | text | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_slice_scalars.at | 6b76f6f5f7788a973513e4695e9a1ab6a445a797d844d7d5ea124120be091c71 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:9 | v03_codepoint_ok | text | value:20013 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_codepoint_ok.at | c285dbc9906d68c47533f9dfa6b78162cb6c33ff5cf786af0af77f3998ca6a0a | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:10 | v03_codepoint_surrogate | text | trap:InvalidScalar | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_codepoint_surrogate.at | 4f9a85fa9398ca0034499ece2ef13bae8d2d403af833dad019a3642723b9c7bf | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| text-reference-contracts.tsv:11 | v03_concat_clone | text | value:112 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/text-char-925--v03_concat_clone.at | e30f0a07b816273899687fe5fdb67d2482ac76cea1d3349d96aa38e063d68d2a | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:2 | sum42 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum42.at | 97d6cc23e97c6c349a5b5ffe55ea2aa24a6e4af6c1b8f9bb4a9676a04f90a6d4 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:3 | callee-body43 | scalar | value:43 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/sum43.at | 9864bc9e5fbdc420df8912b65b4b0229ff6642077b00c6790d15515136ae9f92 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:4 | bool | scalar | value:7 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/boollit.at | 799da4a2a4ac5104e08ea433b57eaeb600f7374b752ea50cf56f34a7a7ff652f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:5 | meta-fields2 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta2.at | 0485f6ed33a0c20b0e3576bc11e9a58957d1784354ce04c2aaed1f92ba323adf | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:6 | meta-fields3 | scalar | value:43 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/meta3.at | 257a85a7583606441e0dc85515a90c0b93d1cf9352c8d93b24d10ae76a6f9010 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:7 | choose-instance42 | scalar | value:42 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/choose42.at | ad48036cd25b00947da60879ec90bd1cc6a70605f5a48f6fbec728b876b587f4 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| cte-reference-contracts.tsv:8 | child-ct-chain40 | scalar | value:40 | crates/auto-lang/test/cases/ace-m1/source-cte-936/src/pick40.at | 7e4cc347a7d62c10758caf0de161ceecd614d6c97a957df6f826b17540b5742d | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:2 | core-c02-paren-newline | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c02-paren-newline.at | 61a3581b2aa284b77cc130b49fc6f248570f366f255ae632cefb62d90660cf4e | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:3 | core-c05-ac-start-name | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c05-ac-start-name.at | 2af56576b302e3f514a2fedd1158cef90dd350b3b8d8c80eefb0c192c90860b3 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:4 | core-c06-exitprocess-name | core | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-c06-exitprocess-name.at | a0c634f621b23c8d31d07a7cd0146f1e8be8be3b150ddbb5f19790c21be121a4 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:5 | core-d01-call-multiline | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d01-call-multiline.at | 050e612c075994a32672bd20ef3d639d986af7b848962c764b084aeac3b07fb0 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:6 | core-d02-paren-multiline | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d02-paren-multiline.at | b313a7731f13fae4c9aa81d9a4bae5da1f3e39c283e656906dcbf8822e80d7df | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:7 | core-d03-negative-multiline | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d03-negative-multiline.at | 3eced0b5aca2bd1f9f326d094fd83403b1a5581d7c8658ae2d7ef4ee5c05d4cc | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:8 | core-d04-nested-paren | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d04-nested-paren.at | 68a69974ce66d92588e5bd7c0766c080097dc3f740c64004a00dac3610ca4a83 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:9 | core-d05-call-no-tail | core | value:2 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d05-call-no-tail.at | c993eb16ce37098e11042da2f17fee346d85733b936937f27cb5ca9eb556849f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:10 | core-d07-colliding-prefix | core | value:7 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-d07-colliding-prefix.at | 1ed45f90bae1b7a235f49e8e2781da55344fc56126a060ca99b434d47029a4fd | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:11 | core-e01-nesting-16 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-e01-nesting-16.at | d687528fc35f12783b0ccd13d3fe09fc77f46a4095f136bdfdf0df34d707a608 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:12 | core-f04-parens-99 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-f04-parens-99.at | d035c0f22af60f44e278062e7bf364c1535e47abf63e5f332c0450612eeb07ca | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:13 | core-f11-add-chain-100 | core | value:300 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-f11-add-chain-100.at | 58f8576f878d2ebc443d072b2c3f4523da0f7234a80d9a8cdf165e0493465659 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:14 | core-g02-parens-200 | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-g02-parens-200.at | b3e140b35957443297852ca93ea5c6c153d124f821e1e12e51861d798b35cfed | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:15 | core-m01-comma-args | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m01-comma-args.at | b0e649f7515bd664e59397cd6124c7d75a6bb3b62d709f21dd5fd13e04b5c32e | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:16 | core-m02-named-binop | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m02-named-binop.at | 6de19f12338f4aa65888a078b63ab31425c3b33c4f7f4df30bd4f9d2dec0576a | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:17 | core-m06-nested-call | core | value:192 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m06-nested-call.at | 26b1af73d665fe6b74930a514529791246cfc7a8aa39be4c7a0c925b583735d9 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:18 | core-m07-bool-arg | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m07-bool-arg.at | 56c32c40262409dc09c991fe8275b6b0b8ae0f1bef2791ad8c175c82cd94294a | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:19 | core-m10-paren-args | core | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m10-paren-args.at | c49cd0daac746223abe242d01760d37055d67002f1bddce7682c9dbb78e5f90b | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:20 | core-m22-named-newline | core | value:33 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-m22-named-newline.at | f0ef5d6f2f48867e7c6e6c6f8fa6d505576a4d0a42ab26ba8c7c81470782ccb0 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:21 | core-p01-precedence | core | value:14 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p01-precedence.at | a09b91f416483d65d8d75a46a20d3e28645844f268496ef45207d2fba16d32d2 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:22 | core-p02-signed-pos | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p02-signed-pos.at | b6606ec99ca3b038da755a09fdd4cdb3ce8a8ac4002f5b8d6b2f5cabe1b463a4 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:23 | core-p03-bool-param | core | value:7 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p03-bool-param.at | aefe99819ee76bc82ae51933ea42dbc957c92d963cdfa9fd58d68ced8ea5a374 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:24 | core-p04-while | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p04-while.at | 556dcc51e11bf7b723e3b061db1c2d614e845770f41617857dfd0bd1425189bf | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:25 | core-p05-while-zero | core | value:1 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p05-while-zero.at | 1407ca813c5aeaff33017aafd342faaf6ad67a11418e554536097bfc14b3759d | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:26 | core-p06-continue | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p06-continue.at | 91435499a6b92ff818d280349c0d9b780078c4cf7faa7af074849c7cd81270c8 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:27 | core-p07-break-nearest | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p07-break-nearest.at | 6506162d22ce0d398111d739fe3815c017a5aebdd7709c631f0bff84ae856528 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:28 | core-p08-recursion | core | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p08-recursion.at | dcda62893983d3ba20e17760b11258cf3eb797816e3531781a93515832944a85 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:29 | core-p09-forward-sig | core | value:8 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p09-forward-sig.at | 6491415ef21f2a0204141fa9a637e38d6d5f240715d777eb0ddc9676e2e8fb62 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:30 | core-p10-shadow | core | value:5 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p10-shadow.at | 3fa87ea52dea38eb0ed750c0e4c87f03e966244ae2f17e5f235f198419309bea | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:31 | core-p11-min-max | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p11-min-max.at | 25bc370a6fcebac19fb5ea49d1a48f103f3da6c0f0ad82d8ba6b6fb8f49ad70d | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:32 | core-p12-unexecuted | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p12-unexecuted.at | 490f0a01093a4ca34d8fa039f1bb14b59f499c88dde60f9acdee571f7ac5b201 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:33 | core-p13-add-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p13-add-overflow.at | 3c8688fdacd28eecf38afb69647da79926dd95d998dfa0a9a7f5717cfee9d0f9 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:34 | core-p14-mul-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p14-mul-overflow.at | 8ce02fbc3c958173f08d1b1ed3015475b14ee6565cc84cfed001502bb59596c0 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:35 | core-p17-hetero-named | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p17-hetero-named.at | d47441320bbb727380548d50954d9d7fdc0c5e5cd9b67de34f4c9209d129ad8f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:36 | core-p18-bool-return | core | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p18-bool-return.at | 8426728f973314cfab2c8709a69be072f649d1ff09514bae8218d2c9096bc63f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:37 | core-p19-user-mark-a | core | value:6 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p19-user-mark-a.at | 1fd8da912590117a42d5825c0340e0f33d587cea8a7e6c8b4afec45c4c06b290 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:38 | core-p20-utf8-crlf-min | core | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/core-p20-utf8-crlf-min.at | 08e75dceb6aea83f48a91d92e8a45e3afeedef3aea0a9d8cb8940187ce088e01 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:39 | sc-a-minus-1 | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-a-minus-1.at | 2f4496793e6a4db42d53f377a30031441843a0a8a08aaefb895995e9370dc641 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:40 | sc-binop-args | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-binop-args.at | 46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:41 | sc-cmp-matrix | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-cmp-matrix.at | 290961bc12288fd5e1523320bac43aeace7a1a770c941449aa3223cdbba50101 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:42 | sc-count-digits | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-count-digits.at | 716d9e896d8eac777d6b14d121b11929fe77972d82adef19ce79f88b576ce3c9 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:43 | sc-crlf | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-crlf.at | d1064b0a16aec0cbeab5a669353109e131d6ec304fb560a4dd449443d3674e3f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:44 | sc-for-loops | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-for-loops.at | fa358a467646a370567d2c1fceb16dc6ea8b631d38e75b1c5199132b7ff1e38d | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:45 | sc-gcd-euclid | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-gcd-euclid.at | 6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:46 | sc-is-digit | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-is-digit.at | 6e6797b67aca64c6c1b4d44fc0f323d54585974cce6bab1288bdb20b22943923 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:47 | sc-short-circuit-contexts | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-contexts.at | b2cbf94dfd30be472f8362cae1efb68a8485e33550ba3bda35b978128be3a2ad | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:48 | sc-short-circuit-trace | scalar | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-trace.at | aa4baf2b834d37538c9ecb4000e080043035d09c0d2c594740e8448a0436bc49 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:49 | sc-short-circuit-trap | scalar | trap:DivByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/sc-short-circuit-trap.at | d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:50 | ace-B01 | core | value:14 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B01.at | b7e5f25e46902be22c9bd3db35a334cb56e130e11cae090b6927314fc2d980c6 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:51 | ace-B02-signed-div-rem | scalar | value:-21 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B02-signed-div-rem.at | 7995c2cd010370655c76350afbaae9ba4cb814eec6b8946dfc600d2b9121b28d | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:52 | ace-B04-shortcircuit | scalar | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B04-shortcircuit.at | 4487dff4b270520faf0619f8aa0097573d51e5b5ee812ffca02488ab26415c12 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:53 | ace-B09-recursion | scalar | value:120 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B09-recursion.at | 66cfe6d5ec89f26568ad16359e6564fe4ef2bc8edc53b0f4175e5c5962fad5b6 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:54 | ace-B13-overflow | core | trap:Overflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B13-overflow.at | 54943ad037b05733996d41d88161225d7bcd0bdd0cb5a7ec9ba50941e3fd481e | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:55 | ace-B14-div-zero | scalar | trap:DivByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/ace-B14-div-zero.at | 1330a237463208d9cdaf958f174cbad20933842de12b86ecb685368f17677ffe | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:56 | bd-min-literal | scalar | value:-2147483648 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-literal.at | 4bf2b0a9f4395500fdb09405dd62a6125fab15add96280d1bc833e941fbbc90c | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:57 | bd-min-div-minus-1 | scalar | trap:DivOverflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-div-minus-1.at | c299f6864ca383aa6d98ff869f8517dd9a2d46017f281bf05f56b4a5c15174e1 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:58 | bd-min-rem-minus-1 | scalar | trap:RemOverflow | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-min-rem-minus-1.at | ce7a15de35446d1c375d47374adb52509287a313e32e6e67e33449a5fbca83a9 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:59 | bd-rem-by-zero | scalar | trap:RemByZero | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-rem-by-zero.at | 8017f260721d0e494b87b6110b8448869bdb0daf071598756ba2fd4f919ba890 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:60 | bd-value70 | core | value:70 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-value70.at | aa8bda9f9973f904be028a6ed8ea71b6eecc9a3c55d74e5fd531f0fad3fa8e59 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:61 | bd-named-source-order | scalar | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-named-source-order.at | 94a9724690289d3143345d808530261a3564f5dba758c3dc1f1047039f167052 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| scalar-reference-contracts.tsv:62 | bd-or-skips-rhs | scalar | value:8 | crates/auto-lang/test/cases/ace-m1-expanded/scalar-src/bd-or-skips-rhs.at | fef0c532b71ba0b21853b33a2415f35ad2a1cb2c6f8d3b83f8e41283e46b7876 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:2 | record-text-cert-925/M_COPY_SNAP | record | value:221 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_COPY_SNAP.at | 2db804e9f664ab1fa1afa49bf8e9d72ee0ac503e33850804656a52f211a4a09a | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:3 | record-text-cert-925/M_MULTI_MUT | record | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_MULTI_MUT.at | 6e5b0eba1f87742d2fb7c693025d4321a1bcb0447e795e98f0fc3bf144d61844 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:4 | record-text-cert-925/M_VIEW_RECORD | record | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_VIEW_RECORD.at | 18d46146b19f52b49aaee46a9e139901431c48e9b40b52fc6fbc0959600b6926 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:5 | record-text-cert-925/M_MUT_TEXT | text-record | value:4 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_MUT_TEXT.at | 2ab16069ee964db74e810b0183043a8c4d69b35b17ff0c79953a3fe0b7939a7f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:6 | record-text-cert-925/M_BORROW_RETURN | text | value:2 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_BORROW_RETURN.at | 70c6bb77ef2cf101498a5beadcb68d131fc7df948ecef948226a805064c45a08 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:7 | record-text-cert-925/M_OWNED_RETURN | text | value:22 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_OWNED_RETURN.at | 39559d922d3f2cd402d5ba875a3c5b75d739e91ee983668971c110ad21a88477 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:8 | record-text-cert-925/M_NAMED_CTOR | record | value:12 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_NAMED_CTOR.at | adf0ad863a47130c9d393120dc60c78695a102c6b6a6d9c4af4c854923f93d0c | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:9 | record-text-cert-925/M_NAME_COLLISION | record | value:137 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--M_NAME_COLLISION.at | 3d51468a071500924786c6034624df4a98f96a1c2bdd41a217a85ebd19f87bc4 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:10 | record-text-cert-925/T_CHAR_AT_NEG | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_CHAR_AT_NEG.at | af394c415404d5650fd4665fc9164f29eb8082b67fcd51f0ccf9d51a6ed1e164 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:11 | record-text-cert-925/T_SLICE_OOB | text | trap:TextIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_SLICE_OOB.at | 0954832b3bc4f2d166b2ce40da91495c70596fc48cd1c85c1e94440187b32763 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:12 | record-text-cert-925/T_SURROGATE | text | trap:InvalidScalar | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/record-text-cert-925--T_SURROGATE.at | 4f9a85fa9398ca0034499ece2ef13bae8d2d403af833dad019a3642723b9c7bf | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:13 | record-text-base/r07 | record | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/R07.at | 6d07888a9383b7e8a7d4d6a50dabc961e0e2f35137cb9823c5cf0463e43b6c28 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:14 | record-text-base/r08 | record | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/R08.at | b2ea23e7f6acb41100ddeee036a648f5cc92b6857608e66f5fbd1f4d5ef9c895 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:15 | record-text-base/t03 | text-record | value:23 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/T03.at | f895f9d1577bbea6850156b2f3364399f1e6d7b2257dad491bcb8f2d4e02c542 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:16 | record-text-base/t05 | text | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/T05.at | 84541676753c15d06dc263efa57105501d87a07907102ae989de5020daf47530 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| record-text-reference-contracts.tsv:17 | record-text-base/c04 | text-record | value:42 | crates/auto-lang/test/cases/a2c-hir-record-text-925/src/C04.at | 56f052c7192c531eb118e3732059bda0260ef5c2b747d62bb0953d1f566d3558 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:2 | collections-926/L01 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L01.at | 8247b7e7688e1931a36bc09eb4ec2cc789a3b099237d20d5dc8095a71e76a4f5 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:3 | collections-926/L02 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L02.at | 8276953bd3fe700fe44593092117d2b93b691aff0747bfb0f1099cc2008432e2 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:4 | collections-926/L03 | collections | value:0 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L03.at | 299034a89f81f97a0ab9ffa71f278d05262db5747a77ec5a460dbdbfce070339 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:5 | collections-926/L04 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L04.at | a79b03e53e6e82bda9000a1b6cd1bcfae695d4cc427928261e9d13bda19b0b99 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:6 | collections-926/L05 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L05.at | 3c55fc65f45724c072aaf7a772cfdc7d52da0d4f117ec6f23841795551629149 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:7 | collections-926/L06 | collections | trap:ListIndex | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L06.at | 9b4aaa8834835283629077af431d7864d9c88a732060cebbd50fe24921c21be8 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:8 | collections-926/L07 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--L07.at | 8aa9b61d53b3f0ce715a98d3c6ff1b0a7fd5f75ede2ea1dc132c650e97577aec | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:9 | collections-926/V01 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V01.at | 900acc88f43f9505b91fa6c4096642589fa7e3ca770833106bd552c481857dee | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:10 | collections-926/V02 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V02.at | 281396b1a3d5ea129a5c4e34672287975bac87ed7b543af7fbc828ce7869259f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:11 | collections-926/V03 | collections | value:3 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V03.at | 99bc231454ba7426c61828525e58dfd2514c6be54249ccbcab43d9c1e0ebf532 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:12 | collections-926/V04 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V04.at | c9303cad597133e0a517b70d5f0693a8a4961199a3148b82abffe039e22839cc | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:13 | collections-926/V05 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V05.at | 734c211d1ea0e62d52f14617369a77c975c0a23c915e3ed9dff91fa1f4c7c0c3 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:14 | collections-926/V06 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V06.at | 7145928b2c570c848f6ca7746545d1ca3d31790045855080d05db4c3d4112383 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:15 | collections-926/V07 | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--V07.at | 04844f208d458a043dd6ef2d2202a8ffb09789010d750d075cc164ee59135478 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:16 | collections-926/LSTR864 | collections | value:51 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--LSTR864.at | f36e92cfa92d4bc4240995c75e312fa84076e7bb7d983b75141dadb7e70524ff | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:17 | collections-926/FIN864A | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--FIN864A.at | 39524c0208966b1aca826afbc1b6c3898526f9373275fb62d7e8dd13662ceb81 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:18 | collections-926/FIN864B | collections | value:53 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--FIN864B.at | d0ce9a71534a37a289eea7ff22307d05414042420548263b3d1a4b4c431286b1 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:19 | collections-926/N889-list-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-list-variant.at | b90d64918e20ea8bb7a702871067cf177064adab604cad4af1205a3f6fb33a7e | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:20 | collections-926/N889-nested-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-nested-variant.at | 4b6433b03528d986ef67edf9d7420409f8e7d64ebccafceadf9fc4d23da6e61f | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |
| collections-reference-contracts.tsv:21 | collections-926/N889-optional-variant | collections | value:42 | crates/auto-lang/test/cases/ace-m1-expanded/composite-src/collections-926--N889-optional-variant.at | e800cab21196d7e848e3ad19bc7af5903f8534e55d52d064695ab38c5c781b84 | 原手动引用 | 保规模/平台/原执行轴；不冒运行 |

</details>

**逐项对账结果。** M=1116、S=71、D=181、L=313、A=48、N=283、U=342；原511/7内嵌inventory的69/legacy332入口/313cell/48架构recipe的564处case引用及283个原生身份经10个Cargo target根/显式path/inline mod静态展开均落到具名行，无待归类项。35个registered recipe=已用recipe，未知/无分派=0。U分母={"scalar-obligations.tsv":61,"q4-source-contracts.tsv":126,"composite-obligations.tsv":41,"text-reference-contracts.tsv":10,"cte-reference-contracts.tsv":7,"scalar-reference-contracts.tsv":61,"record-text-reference-contracts.tsv":16,"collections-reference-contracts.tsv":20}。完整审计表示SHA256=b85dd7d3cfdc1b0c015f6dd76b077692aa246305506ceff943947079d0f6c542（canonical JSON审计表示，非Rust runner指纹），原件及defaults仍以S/D/输入路径和固定Git对象为权威。原正式目录668个文件pin均已核对；source/oracle侧354个补充来源逐一核hash。零映射缺口不代表零实现风险，F01–35和原明确不适用/缺独立reference的合同继续保留。


附加终检：expanded inventory 397项逐条metadata/source/case哈希一致，49架构义务=48个recipe+独立review行；668个原目录固定文件与读取期间全部材料再次核hash不变。此处仍为静态映射自检，不产生正式验收凭据。

### 建议实施次序与交接

| 顺序 | 有界切片 | 与现有工作关系 |
|---|---|---|
| 1 | F01/F02/F06正确性与观察：先真实反例，再定修复 | M1风险交原owner，不在本轮顺手改代码或改变出口 |
| 1a | F16/F21恢复准入、F22负依赖、F24/F25候选事务；各自独立修复合同 | 先保合法邻例/当前旧态，再补晚失败反例；不为统一机制扩大profile或恢复授信 |
| 1b | F18/F19/F27父子身份与有界终态；F23/F26严格reader与事实归因 | 保one-shot/预构建worker/父oracle，复用已有executor；不把infra归语言负例 |
| 1c | F28/F29共享源码循环、F31分页准入、F32 CT遍历、F33字段融合 | 在各自原owner先固化有限反例；共享walk/推进点只消重复机制，不扩大语言/ABI/恢复授信 |
| 1d | F30具体测试义务纠正 | 保正常释放用例，再补真实聚合失败/旧值保全；与F14/985映射，不借改名冒新增执行证据 |
| 2 | F04/F10/F11共用机制 | 先冻结保留合同；避免与1002同时大改作者格式和执行器 |
| 3 | F03/F05/F08/F09/F12主通路 | 与962–978能力owner核接口，不接管WIP；重大决定先设计 |
| 4 | F07/F13/F15往返/模块/注释清理 | 小步保持身份；1017测真实成本，本包不报加速收益 |
| 全程 | F14义务守恒 | 1002管格式，985管覆盖，1017管成本，本包管架构审计与候选，不互代验收 |

### 第六轮覆盖闭合与有界后继合同

下表是对前轮P/U的最终分流，不重写历史轮次的“当时未完成”。S-closed表示本包已交付固定源上的职责、状态分区、证据/裁定和保留义务；它不表示所有CFG路径或条件笛卡尔积已证明、每个源码行已人工读完或动态覆盖率100%。本包不存在仍待调查/归类的HIR项或原case/check/recipe行；具名运行、修复和收益未知项继续作为后继义务公开保留，不计运行PASS。

| 原闭包 / 当前状态 | 已完成的只读证据与归属 | 保留的后继验证，不作本轮未读事项 |
|---|---|---|
| COV-A / S-closed | A01–10；12处计数预留、Pratt准入、分页/跨度、lease及父子身份/终态；F16–19/21/27/31 | 原decoder/OS/真实worker反控；按合法邻例、坏页、资源错误与infra分开，见切片S04/S05/S07 |
| COV-B / S-closed | B01–16；54指令/3终结符/11 profile、AST25/14、HIR17/42/9；T/C/K/F→MIR→AV/native/A2X消费及拒绝；F01–09/24/28–29/33–35 | profile×构造×资源/fault的动态矩阵保持unknown；42+9逐项为分区，不伪造穷尽输入证明，见S01–03/S06/S08/S10–12 |
| COV-C / S-closed | C01–13；正负依赖、恢复/当前权限、prepare/commit、两条CT通路/Named遍历、foreign lane与shadow commit；F08/22–23/26/32 | 真晚失败/恢复首改、ABI/provider/能力变更、single-flight/取消运行；见S03/S04/S08/S12 |
| COV-D / S-closed | D01–11；两个target、CAS/GC/lease锁序、arena/frame、data/text/typed/trace各ABI与故障/清理分界；R5补OS拒绝后的状态，R6补loop poll/资源分类 | OS/provider失败后的ledger收敛、reparse实机差异、真实链接/调用及测量收益未取得；见S03–07/S11 |
| COV-E / S-closed | E01–07；M1116/S71/D181、42 dispatch route、L313/A48/N283及U342逐行对账；原511/668、69和397 inventory检查全对齐 | nextest编译发现/执行数/原条件排除与缺独立reference维持原状态；1002未迁移者继续原入口，见S09 |
| MC01–35 + MC36 / S-closed | 前轮成员表的入口/consumer/拒绝/测试归属保留；现以COV、HIR表、逐项映射覆盖其具名剩余面；不以F列表代成员分母 | 静态库由真实消费者验证，manual/平台/ignore/正式规模保持；M1收据是原证据，不是本轮重跑 |

**第六轮时的后继切片建议（历史，以下已由第七轮立项；当前交付见2026-11-17表）。** 每片须沿原owner在专属树冻结来源、补真实可区分反例，再实施和独立复审；不让1024变成修复所有ACE的无界合同。下表覆盖F01–35；原断言/输入/规模/条件不得因复用被删除，动态结果仍未取得。

| 依赖顺序 / 切片 | 发现与生产者→消费者 / 有界改面 | 保留与关闭oracle / Spec候选及冲突边界 |
|---|---|---|
| S01，独立 | F01；native闭包session差异→apply_source，沿AV已有UTF边界机制 | é→ê、ASCII/空/前后缀、fresh/warm源码与结果/诊断；只改编辑保真，auto-ac会话Spec候选，不接管M1签章 |
| S02，独立 | F02；core capture→AV/native/CT计量consumer；scope guard，明确嵌套计数 | 正常/Err/panic后再capture，外层恢复、无幽灵值；观察合同候选，与1017计量字段对齐 |
| S03，各子片独立 | F06/F22/F24/F25；分别864外层、QueryHost负依赖、typed批编辑、A2X resident/anchor事务；每个owner单独合同 | 接受A→prepare B→晚失败→仍A→同批重试可成功；正负依赖、其他target居民、旧文件/结果分账；auto-av/auto-semantics/auto-ac/auto-a2x候选，不能用一个事务测试替四片 |
| S04，各decoder独立 | F16/F21/F23/F31；syntax程序/FirstSets、实例页、CST分页reader→恢复消费者；限量再读/分配、形状/范围门 | 小坏包不巨分配/不panic，合法邻例及当前格式/错类；恢复不授信；syntax/source-query Spec候选，禁止静默升级旧wire严格性 |
| S05，独立 | F17；TreeSession lease计数→旧树访问/索引回收 | 多lease、最后Drop、publish、跨代拒绝、churn后零项有界；syntax生命周期候选，不冒所有树泄漏 |
| S06，独立或同设计多子片 | F28/F29/F35；source for-in合成→MIR；C/Rust collections Loop/Continue→runtime poll；共享latch思想，各自有界改面 | 条件continue有限循环/闭区间MAX/break/return/嵌套loop；F35另核WU、cancel和cleanup，不能以值正确替预算正确；core/A2X控制流Spec候选 |
| S07，各入口独立 | F10/F18/F19/F27；worker reader/源快照→语言父adapter及现owned executor | framing/EOF/编码/IO保持；编译输入SHA同源、合法上限、迟到/超时/输出cap/kill-reap；auto-test-case/worker Spec候选，复用现executor，不造runner |
| S08，先S02约定观察字段 | F26/F32；ordinary CT发现/回灌→服务执行/发布事实，child walk共性与事实字段分别切片 | Named hash、正常位置、嵌套/预算拒绝；真实执行后publish失败必须仍观察AV到达，不冒语言PASS；auto-cte候选，保817/ordinary通路边界 |
| S09，贯穿全程 | F14/F20/F30；固定组选择/1002作者格式/985覆盖→本附录M/S/D/L/A/N/U和真实fault入口 | 原check、expected、case/source身份、原条件逐项守恒；正常释放例保留，再补真正聚合中途失败与旧值保全；auto-test-case候选，保1016/1025正式性能分轴，不补跑压力门 |
| S10，独立 | F33；source Named get.field→adapter融合分支 | Named/positional等价，合法/越界为受控结果，真入口无panic；core/lowering候选，不放宽owned元素whole escape |
| S11，先设计冻结定义 | F34；core资源分类→shared lowering/MIR ownership→AV/native及已有聚合消费者 | List-only/Str+List/纯标量/tuple/Box等合法域；move/clone/字段投影/clear/调用返回/fault分别断言；跨核心语义owner/物理drop/借用合同先设计，不用一个布尔统一全部域 |
| S12，先冻结保留合同 | F03/F05/F08/F15；结构诊断/profile、会话职责、类型权威、旧名字注释→当前consumer | 保当前受控拒绝及source span、恢复/候选边界；962–978在途owner逐一协调，不把不同层会话或失败路径删作重复；各模块Spec候选 |
| S13，先S09守恒；收益先测 | F04/F07/F09/F11/F12/F13；算术发射、Atom往返、crate宿主、data/text机制、现provider/executor及大模块→原consumer | 优先52行机制/纯child枚举等小片；保runtime符号/ABI/预算/取消与独立oracle；1017测真实成本才报收益，1002只管格式；模块拆分收益仍candidate |

当前Spec影响仍为SD-00：本包没有交付行为变化，supersedes/new/touched维持空。上述候选不是现行Spec增量，也不要求本轮改canonical Specs、其他Plan或债务表；确认项/候选项的独立核验与正式债务登记归T-04，后继实现分别声明真正增量。真实反例尚未执行是明示的证据边界，不是删除/批准延期原义务。


### 第七轮：复审后逐发现分配（2026-11-13历史快照；当前状态见2026-11-17路线表）

本表是F01–35的唯一主承接表，S01–13仍保为机制/依赖研究分组，不与本表争owner。**35/35有主计划、任务、独立关闭case、优先级和1002关系**：6项进已有969/970/978/985，29项进新1027–1038（11修复Plan+1研究Plan）。全部实现仍未授权/未开始；分配不等于缺陷已修、已执行或原M2验收通过。

优先级P1=先真实反例再修正确性/准入/事务/预算风险，非声称已有生产事故；F06保P1但当前历史测试consumer紧急度低于同级生产入口。P2=策略/架构/测试治理与收益取证，P3=注释。F12非缺陷；F14静态保留账已完成，后续是实际迁移/认证，不重做本轮映射。

G1002=1002 T-03四例七checks及负控真实通过，T-04独立复审且接口正式合入、绑定SHA。核心修复/现有Rust或旧case反例不依赖1002；只有新作者格式迁移或document/IR/adapter重构任务等待G1002，不等全语料搬迁。1002也不反向等待35项全修完；实际影响其七行基线的真故障按受影响行协调重跑。

| 发现 | 优先级 | 复审保留分类/边界 | 唯一主计划 | 承接任务/AC | 具名关闭case（拟实施） | 与1002关系 | 当前状态 |
|---|---|---|---|---|---|---|---|
| F01 | P1 | 算法保真缺口；真实PE后果未测 | [1027](1027-ace-native-utf8-edit-fidelity.md) | T-02/03→T-04；AC-02/03/04 | UTF-01/02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F02 | P1 | 同类嵌套/unwind恢复缺口 | [1028](1028-ace-scoped-observation-restoration.md) | T-02/03→T-04；AC-02/03/04 | OBS-01/02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F03 | P2 | 文案耦合确认；替代方案候选 | [969](969-ace-m2-facade.md) | T-01..04→T-05；AC-AUDIT | AUDIT-03 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F04 | P2 | 52行机制重复；复用候选 | [1038](../1038-ace-reuse-candidates-and-cost-decisions.md) | T-02/03→T-04；AC-02/03/04 | REUSE-01 | 研究不依赖；后继涉及作者/执行接线的重构等G1002 | 已承接，未修复/未运行 |
| F05 | P2 | 职责分散；不可整套删除 | [970](970-ace-m2-sessions.md) | T-01..04→T-05；AC-AUDIT | AUDIT-05 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F06 | P1 | 条件性重试缺口；仅历史测试consumer | [1030](../1030-ace-av-864-failed-edit-transaction.md) | T-02/03→T-04；AC-02/03/04 | AVTX-01/02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F07 | P2 | 转换事实；收益未知 | [1038](../1038-ace-reuse-candidates-and-cost-decisions.md) | T-02/03→T-04；AC-02/03/04 | REUSE-02 | 研究不依赖；后继涉及作者/执行接线的重构等G1002 | 已承接，未修复/未运行 |
| F08 | P2 | profile权威差额；无成功fallback证据 | [969](969-ace-m2-facade.md) | T-01..04→T-05；AC-AUDIT | AUDIT-08 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F09 | P2 | 依赖/宿主混合事实；成本未知 | [1038](../1038-ace-reuse-candidates-and-cost-decisions.md) | T-02/03→T-04；AC-02/03/04 | REUSE-02 | 研究不依赖；后继涉及作者/执行接线的重构等G1002 | 已承接，未修复/未运行 |
| F10 | P2 | reader有界且差异确认；协议后果待核 | [1034](1034-ace-worker-source-and-parent-bounds.md) | T-02/03→T-04；AC-02/03/04 | IPC-02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F11 | P2 | 同源不等同合同；共享候选 | [1038](../1038-ace-reuse-candidates-and-cost-decisions.md) | T-02/03→T-04；AC-02/03/04 | REUSE-02 | 研究不依赖；后继涉及作者/执行接线的重构等G1002 | 已承接，未修复/未运行 |
| F12 | 保留约束 | 正向复用基础；非缺陷 | [978](978-ace-m2-build.md) | T-01..04→T-05；AC-AUDIT | AUDIT-12 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 保留实践，不计修复数 |
| F13 | P2 | 维护热点；拆分候选 | [1038](../1038-ace-reuse-candidates-and-cost-decisions.md) | T-02/03→T-04；AC-02/03/04 | REUSE-03 | 研究不依赖；后继涉及作者/执行接线的重构等G1002 | 已承接，未修复/未运行 |
| F14 | P2 | 静态保留账已闭；迁移/运行覆盖待认证 | [985](../985-ace-m2-certification.md) | T-A01/A02/A03→T-A04；AC-AUDIT | AUDIT-14 | 旧账/职责先做；新格式批迁才等G1002，不能反向堵1002四例 | 已承接，未修复/未运行 |
| F15 | P3 | 陈旧注释；非依赖违规 | [1038](../1038-ace-reuse-candidates-and-cost-decisions.md) | T-02/03→T-04；AC-02/03/04 | REUSE-03 | 研究不依赖；后继涉及作者/执行接线的重构等G1002 | 已承接，未修复/未运行 |
| F16 | P1 | 内部计数先预留；真实allocator后果未测 | [1029](1029-ace-restored-input-and-lease-bounds.md) | T-02/03→T-04；AC-02/03/04 | ADM-01 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F17 | P2 | 零租约索引增长；非整树泄漏 | [1029](1029-ace-restored-input-and-lease-bounds.md) | T-02/03→T-04；AC-02/03/04 | ADM-05 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F18 | P1 | 源与SHA两读；未证明父侧误PASS | [1034](1034-ace-worker-source-and-parent-bounds.md) | T-02/03→T-04；AC-02/03/04 | IPC-01 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F19 | P2 | 预读无cap事实；合法上限需决定 | [1034](1034-ace-worker-source-and-parent-bounds.md) | T-02/03→T-04；AC-02/03/04 | IPC-02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F20 | P2 | 固定选择差额；不否定M1原验收 | [985](../985-ace-m2-certification.md) | T-A01/A02/A03→T-A04；AC-AUDIT | AUDIT-20 | 旧账/职责先做；新格式批迁才等G1002，不能反向堵1002四例 | 已承接，未修复/未运行 |
| F21 | P1 | Pratt引用未验；panic未运行 | [1029](1029-ace-restored-input-and-lease-bounds.md) | T-02/03→T-04；AC-02/03/04 | ADM-02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F22 | P1 | commit缺负依赖；错误target未执行 | [1031](1031-ace-query-negative-dependency-commit.md) | T-02/03→T-04；AC-02/03/04 | NEG-01/02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F23 | P2 | size/tag/EOF缺口；上层仍可能拒 | [1029](1029-ace-restored-input-and-lease-bounds.md) | T-02/03→T-04；AC-02/03/04 | ADM-03 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F24 | P1 | typed接受态部分改写；未运行 | [1032](1032-ace-native-typed-batch-atomicity.md) | T-02/03→T-04；AC-02/03/04 | NTX-01/02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F25 | P1 | resident/SHA/anchor缺单一接受点 | [1033](1033-ace-a2x-late-publication-atomicity.md) | T-02/03→T-04；AC-02/03/04 | AXTX-01/02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F26 | P2 | AV到达字段错误；非语言假PASS | [1037](1037-ace-cte-named-walk-and-attempt-facts.md) | T-02/03→T-04；AC-02/03/04 | CT-02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F27 | P1 | 局部父等待无界；进程树后果未测 | [1034](1034-ace-worker-source-and-parent-bounds.md) | T-02/03→T-04；AC-02/03/04 | IPC-03 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F28 | P1 | continue绕隐式推进；有限模型区分 | [1036](1036-ace-loop-latch-and-cooperative-poll.md) | T-02/03→T-04；AC-02/03/04 | LOOP-01 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F29 | P1 | 闭区间MAX后多加一；未运行 | [1036](1036-ace-loop-latch-and-cooperative-poll.md) | T-02/03→T-04；AC-02/03/04 | LOOP-02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F30 | P2 | 正常例冒故障标题；真实fault未测 | [1035](1035-ace-aggregate-resource-classification.md) | T-02/03→T-04；AC-02/03/04 | RES-03 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F31 | P1 | paged条目/跨度准入缺口 | [1029](1029-ace-restored-input-and-lease-bounds.md) | T-02/03→T-04；AC-02/03/04 | ADM-04 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F32 | P1 | Named CT子树遗漏；真入口未运行 | [1037](1037-ace-cte-named-walk-and-attempt-facts.md) | T-02/03→T-04；AC-02/03/04 | CT-01 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F33 | P1 | Named字段融合有panic分支；未运行 | [1035](1035-ace-aggregate-resource-classification.md) | T-02/03→T-04；AC-02/03/04 | RES-02 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F34 | P1 | 资源分类差额；不强求各表示owned相同 | [1035](1035-ace-aggregate-resource-classification.md) | T-02/03→T-04；AC-02/03/04 | RES-01 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |
| F35 | P1 | continue绕回边poll；运行影响待量化 | [1036](1036-ace-loop-latch-and-cooperative-poll.md) | T-02/03→T-04；AC-02/03/04 | LOOP-03 | 核心/原入口不依赖；新格式或document/adapter重构才等G1002 | 已承接，未修复/未运行 |

**2026-11-13分配时次序和冲突（历史）。** 第一批优先1027/1028、1029的F16/21/31、1031–1037高风险项；它们不存在互为整包语义前置，选择同文件不重叠者推进。1029与1031及在途963共用source-query，1035/1036及963/965共用core/resolve/lowering，1033/1034/1037与966及A2X在途共用路径，必须单写/串行重核基线，不混合WIP。1030历史外层单独验证后修。第二批处理1029的P2项、1034协议/上限决定、已有M2结构职责与985治理；1038先交六项go/no-go/保留决定，不能直接实施大重构。

1038交付研究而非修复，F04/07/09/11/13/15各有决定与首个可实施合同；只有有证据的go项再建/修有界实施Plan。1017不承接这些项的全部修改，只协调同路径成本基线。已有M2 Plan的最终能力依赖保持，不能因承接登记越过其原前置。所有原case/check/规模/条件/独立oracle保留。

逐项分配表示SHA256=0f7e524f15bb0defc6565e9c96dda007920d8aeb5fe6d9af6f681e85efed6abb（仅本表canonical JSON），用于防漏/重复/误owner，不是运行凭据。后继真正关闭须写源提交、修前/修后、合法邻例和独立review，不因本表有owner就关闭风险。


<a id="quality-route-20261117"></a>

### 修复优化路线状态与执行顺序（2026-11-17，当前）

本轮核对主线HEAD=e85096628c468376c0c3632d832420a2e7f6d439。本节更新1027–1038十二包及审计总计划的状态/调度，不重开已归档计划，不修改实现，不把本次簿记算成任何实施任务完成。**9/12已交付归档；3/12未开工，默认新增实施顺序为1037→1030→1038。** 1024自身仍reviewed r2、4/5，T-05审计来源合入/归档可独立安排，不等待三份后继全部完成。

| 计划 / 职责 | 当前状态 / revision | 正式主线交付或下一动作 | 阶段与当前顺序 |
|---|---|---|---|
| [1027 native UTF-8编辑保真](1027-ace-native-utf8-edit-fidelity.md) | 已交付归档，6/6；r3 | 交付a425e9adf，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1028 计量窗口嵌套/unwind](1028-ace-scoped-observation-restoration.md) | 已交付归档，6/6；r1 | 交付7880f2a42，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1029 恢复输入准入与租约](1029-ace-restored-input-and-lease-bounds.md) | 已交付归档，6/6；r6 | 交付1ea540a19，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1030 864历史会话失败重试](../1030-ace-av-864-failed-edit-transaction.md) | drafting，0/6；未开工；r3 | 后续T-01核历史consumer是否仍适用；保失败后旧接受态、同批重试原oracle | B后续；第2（P1历史入口，不降成已关闭） |
| [1031 QueryHost负依赖提交](1031-ace-query-negative-dependency-commit.md) | 已交付归档，6/6；r3 | 交付4e6d28522，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1032 typed native批编辑原子性](1032-ace-native-typed-batch-atomicity.md) | 已交付归档，6/6；r3 | 交付45fcd50f4，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1033 A2X暖编辑/晚发布事务](1033-ace-a2x-late-publication-atomicity.md) | 已交付归档，6/6；r3 | 交付305bbd0c1，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1034 worker快照、限额与父子终态](1034-ace-worker-source-and-parent-bounds.md) | 已交付归档；末merge收据完成（头部仍3/6）；r5 | 交付140c1a626，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1035 聚合资源分类与故障回归](1035-ace-aggregate-resource-classification.md) | 已交付归档，6/6；r5 | 交付d2847bf4c，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1036 continue推进、终点与poll](1036-ace-loop-latch-and-cooperative-poll.md) | 已交付归档，6/6；r4 | 交付3b734996b，已核为主线祖先 | 既有已交来源，不再占新开工队列 |
| [1037 CT命名实参与发布失败事实](1037-ace-cte-named-walk-and-attempt-facts.md) | drafting，0/6；未开工；r3 | 下一包：T-01重核CT-01/02真实入口、源漂移与故障事实；已有等效修复须反证/回归关闭，不能重复改实现 | A必闭正确性；第1（F26虽P2亦属A） |
| [1038 复用候选、成本与模块边界研究](../1038-ace-reuse-candidates-and-cost-decisions.md) | drafting，0/6；未开工；r3 | 后续T-01冻结六项研究输入，交go/no-go/保留决定和有界后继合同 | B有界研究；第3（P2/F15 P3）；只读可提前 |

交付判定取末次review/merge/cleanup收据，并以git merge-base --is-ancestor逐一核九个交付提交；相关语言worktree均不在当前登记中。1034 frontmatter的3/6为归档簿记滞后，末merge/清理收据及当前主线140c1a626证明已交，不误报仍在执行；本轮不回写归档文件。1031保留外部retention登记表具名未决，1033/1034保留各自已归因的预存红/环境项；归档不等于全套件/全平台在今日基线重新全绿。

执行规则：

1. **先1037**：原CT-01/02、F26/F32全部A-required；先T-01核当前入口/作者格式/依赖与publish故障seam，再T-02/03修复或充分反证+永久回归、T-04独立复审、T-05/06正式来源/收尾，交985 T-A03/A04接收；不等待1030/1038或完整1066迁移。未复现/组绿不能替已修反证。
2. **1024 T-05并行文档收尾**：原只读审计已复审，合入归档审计来源不等于签后继修复或签985/A。本次只更新簿记，未执行T-05。
3. **再1030**：保P1但当前属B历史测试消费者，先核可达调用与事务合法邻例，再修失败重试。若最新A真实消费链使用此缺口，须按985原约上调受影响A义务，不能拿B标签放行。
4. **1038最后交研究结论**：纯只读盘点可与前项并行，不占实现写窗；默认不抢1037的正确性资源。六项均须给go/no-go/保留理由与成本证据，unknown如实；go项另立有界实施合同，不能以本研究授权直接拆模块/改ABI。1030→1038是建议资源次序，不新增整包硬依赖。

1002试点已以70a680dba正式落主，1017成本专项已以357d68471落主（本轮均核祖先）；不再称其在途或要求等待整个测试新框架完成。G1002仍按实际作者/document/adapter接线触面消费正式接口；1037原入口回归不等待1066全部格式迁移。1038的成本/重复实现假设要在1028/1029/1034/1035/1036等新源上重核，不能复用旧数字宣称收益。Windows固定组排除Linux/WSL腿并记原条件，Linux另在专用环境；不据此取消义务。正式ace-perf仍显式独立，不附每Plan日常门。

当前三包均无登记的实施树；启动时以当时master重核共享source-cte/AST/core/lowering/worker及文档用例迁移触面，同文件单写/串行重核，不能混其他在途WIP。排序不自动授权开工，也不新增全仓/正式性能回归；冻结完整受影响固定组及真实消费者由原T-01确认。原35项静态审计表与2026-11-13分配hash保留为历史来源，本节不把“有承接人”或“9包归档”扩写成全部发现/985-A已通过。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r2当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：B01只作跨层公共行为例，不给审计新增运行要求。F01源编辑é→ê的range/replacement重建必须等于新源；算法静态反例、真实Rust/PE结果分账。 本轮未运行；这些断言是实施目标，不是已观察PASS。

```text
old = "é"  (UTF-8 c3 a9)
new = "ê"  (UTF-8 c3 aa)
required: splice(old, old_range, replacement) == new
```

这是F01的算法保真反例/邻例输入；本包只审计，不因该展示创建Rust探针或宣称真实PE已错。保留原F01–20/MC01–36已核与未核分类。

**类型/代码设计。** 已实现源/Spec/既有测试→生产者消费者与不变量追踪→反例/误报排除→confirmed/candidate/rejected/unknown→有界后继设计。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct AuditFinding { rule: RuleRef, source: SourceIdentity, path: ProducerConsumerPath, status: FindingStatus, counterexample: CounterexampleRef, proposed_owner: PlanRef }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/native/src/closures_incremental.rs](../../../experimental/ace/crates/native/src/closures_incremental.rs) | F01 UTF-8编辑/原实现 |
| [experimental/ace/crates/av/src/closures_fn_session.rs](../../../experimental/ace/crates/av/src/closures_fn_session.rs) | 既有边界算法复用比较 |
| [experimental/ace/crates/core/src/lib.rs](../../../experimental/ace/crates/core/src/lib.rs) | 嵌套/panic观察作用域候选 |

**状态、错误与资源。** 仅大文件行数/相似字符串/旧注释/自报PASS不直接定缺陷；只按旧range退UTF-8边界但replacement仍断字须展示错误重建，合法ASCII/原样源作邻例。 保当前F01–20、MC01–36的已有发现/未核面和原风险分类；不在此修代码/加测试/改变M1出口，未测Rust/PE仍标未测，不把候选当已批准重构。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [experimental/ace/crates/core/tests/cases/source_instances_935.rs](../../../experimental/ace/crates/core/tests/cases/source_instances_935.rs) | 现职责/实例不变量与测试接缝 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `0b559ecd6575bd90` | 沿F/MC原审计ID核来源/反例：V-DETAIL-01..04；不注册运行case；原任务 T-01/T-02/T-03 |
| [crates/auto-lang/src/tests/trait_vm_tests.rs](../../../crates/auto-lang/src/tests/trait_vm_tests.rs) | 公开语义与内部实现不同，旧VM不是审计全局oracle | Rust断言来源：test_spec_default_method_inherited_and_overridden, test_abstract_spec_method_still_enforced, test_bounded_generic_fn_dispatches_on_runtime_type；SHA256前16 `a1f3924261667a1c` | 沿F/MC原审计ID核来源/反例：V-DETAIL-01..04；不注册运行case；原任务 T-01/T-02/T-03 |
| [docs/plans/1002-autodown-document-cases.md](1002-autodown-document-cases.md) | 格式试点只四例七checks，不据此声称全迁 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `504e82209cd0e423` | 沿F/MC原审计ID核来源/反例：V-DETAIL-01..04；不注册运行case；原任务 T-01/T-02/T-03 |
| [docs/plans/1017-ace-perf-m2-cost-and-workload-optimization.md](1017-ace-perf-m2-cost-and-workload-optimization.md) | 性能归实测，审计只交定位/候选 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `858881bc8a02af4e` | 沿F/MC原审计ID核来源/反例：V-DETAIL-01..04；不注册运行case；原任务 T-01/T-02/T-03 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

本包仍纯分析：上面的源与草图用于审计职责/反例，不新增Rust测试或运行产品。原F01–15及V01–06全部保留，真实动态验证交后续具名实施合同；canonical Spec不因建议而改写。

**原任务内的决定和交接。** 每项给真实源/受影响consumer/保留不变量/反例或误报排除、归属与测试守恒；动态验证需要另行实施合同，本计划保持纯分析。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-01`、`T-02`、`T-03`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs目标 | before/after | 理由 / AC |
|---|---|---|---|---|
| SD-00 | 无 | 无 | 当前合同不变，交付审计/拟议改进 | 未交付实现不将建议写成当前Spec；后续实施包分别声明增量 / AC-03 |

## 6. 测试设计

本包纯分析，验证来源/覆盖/推理，不跑cargo t ace/正式性能，不把未测报绿。后续实施在专属树选择完整职责组和必要consumer，原样本、条件、golden/oracle保持；不用本表代替真实结果。

| 验证 / 不变量 | 实际入口、区分错误实现的oracle | 固定组导航 / 状态 |
|---|---|---|
| V01 编辑保真 | native/AV闭包session apply_source；range/replacement重建精确等于新源码，fresh/warm结果/诊断一致 | ace-native ace-av及实际closures suite；仅F01算法内存重放已做，Rust/PE未做 |
| V02 观察作用域 | core capture APIs嵌套和panic后捕获，外层恢复、无幽灵计数，明确内外计数规则 | ace-source及观察consumer；待真实测试 |
| V03 失败原子性 | 864会话有效→失败→同源重试/修复，accepted/fps/root/结果一致 | ace-av、case-ace-m1-collections-864；先核真实消费者和故障点 |
| V04 分派/算术 | 正常、窄域拒绝、真语法/类型错误；改文案不改选择；两个native家族保原算术trap | ace-native ace-lowering ace-source和实际受影响case；待测 |
| V05 IPC/资源 | 各真实worker错编码、EOF/长行、取消/超时/迟到输出；父侧拒绝准确、资源收敛 | 按触面选ace-native-worker/ace-abi/ace-semantics/ace-artifacts；待测 |
| V06 测试迁移 | 原case/check守恒；改一期望只红目标，删检查被对账拒绝，新旧观测一致 | auto-test-case及1002/985实际注册组；不虚构新组已存在 |

准备/构建/运行/清理分账；正式性能仍由1017与原合同沿现入口ace-perf显式执行（旧ace-performance已改名）。用户未指定独立reviewer模型，本包不自行开子agent/chat；最终独立复审需另行按review范式执行。

### 第二轮实际检查与后续反例入口

| 检查 | 已执行结果 | 未执行边界 / 对应发现 |
|---|---|---|
| R2-V01 固定源与漂移 | git show/ls-tree/正常path依赖解析；741文件/35成员清点；R1后基线→本轮ACE diff为空；命令退出0 | 未检远端tag，不将本地无匹配tag当M1未交付 / T-01 |
| R2-V02 UTF-8算法与合法对照 | 内存按两份Rust控制流重放11组：native 6个重建不等价、5个等价；AV 11个均等价；每组以重建==新串为oracle，命令退出0 | 不是Rust/真实编译执行；尚未观察PE或source诊断实际终态 / F01、V01 |
| R2-V03 计量与提交数据流 | 逐句追capture保存/恢复及inner build-aside→execute→commit；F06内存抽象状态两步重放第二次无提交但接受态仍A，命令退出0 | 不是线程局部Rust测试或真实fault run；Result::Err正常返回与panic分开 / F02、F06、V02/03 |
| R2-V04 重复与差异 | 两binary完整52行逐字相同、SHA见F04；runtime公开extern静态提取data 39/text 30；依赖及API职责核对完成 | 不代表整lowerer/两runtime相同，不代表最终链接符号或编译成本 / F04/05/09/11/13 |
| R2-V05 事实/父裁判/复用 | 核事实成功与诊断拒绝两分支、provider/query/CTE接缝、worker读行差异、父门3公共函数和固定组导航 | 未运行nextest发现、协议进程、格式试点或恢复/并发套件 / F03/07/08/10/12/14/15 |

上述反例模型只提供机制可反驳性；后续真实V01–06仍为待验证，不能将本轮命令exit0记为ACE功能/资源PASS。本轮无测试资产更改，无构建，不新增材料目录。

### 第三轮实际检查（成员覆盖与新增反例）

| 检查 / 范围 | 实际结果与方法 | 未验证边界 |
|---|---|---|
| R3-V01 Cargo成员→MC清单 | TOML读取全部35个members，逐包取Cargo/source/测试入口/本仓消费者；MC01–35集合与members严格相等，无重复/缺项；语言父侧另列MC36 | 这是成员/记录守恒，不是测试执行覆盖率；间接/目标平台/feature Cargo完整解析尚未cargo tree认证 |
| R3-V02 分层补审 | 补查语法codec/lease/edit/recovery、语义身份/codec/求解/query/store、CT两车道、foreign plan、A2X provider/读源、runtime和conformance职责；证据落各MC | 静态局部入口审阅，不冒每个文件/函数已全读；每行剩余项维持P |
| R3-V03 F16小输入反例 | 仅内存构造22字节header：`<HIIQI>(3,1,0,0,0xffffffff)`，mode count在偏移18，值4294967295；SHA256=`829a498dcaf6acddf41b8043a187df90ed91c6cc53d737ad462d3c8ea9581847`；确认decoder在首次读元素前预留 | 未调用Rust decoder，未申请大内存，无OOM/压力实验；输入SHA只标反例身份，不是执行PASS |
| R3-V04 F17生命周期 | 内存按publish/lease/drop顺序模型1000轮：entries=1、lease keys=1000、positive=0；核Drop没有remove和测试断言边界 | 未运行Rust/OS内存采样，不称1000棵树未释放或生产泄漏 |
| R3-V05 F18/19源码与协议 | 核source_path两读、source_text一次hash、父侧实际传path；SHA(A)/SHA(B)及二读失败空hash的可区分性作内存对照；核stdin read_to_string在dispatch前 | 未并发修改文件，未运行worker长输入；父侧是否拒绝真实混合身份仍待测 |
| R3-V06 F20包选择对账 | 静态解析sets/group：ace-workspace的-p为26/35；9个差额=4有显式职责组+5 no_std runtime；确认ace alias仅三组且Cargo依赖不自动选择依赖包测试 | 未执行nextest发现，不给实际用例数/平台排除数，不据差额判签章无效 |

本轮同时对照`docs/specs/auto-syntax/snapshot-cst.md`、`auto-semantics/semantic-contract.md`、`auto-cte/project.md`、`auto-abi/shared-interface-and-scalar-bridge.md`、`auto-av/loaded-program-runtime.md`：保UTF/lease/授信、身份与负依赖、CT预算、ABI lane与装载预算合同，不将实现观察改写成新规范。所有检查仅内存计算/只读源码，未创建探针文件或材料目录。

### 第四轮实际检查（分支静态证据）

| 检查 | 实际操作与结果 | 限制 / 关联 |
|---|---|---|
| R4-V01 分支导航与归属 | 对冻结Git源372个src文件生成内存导航；COV-A10/B16/C13/D11/E07共57行，归属覆盖MC01–35及父侧MC36 | 词法数量不代CFG、人工全读或动态覆盖；仍缺的内部条件与case/check闭包具名保留 |
| R4-V02 反序列化 | 列出program_codec全部12处预留，追decoder→FirstSets；按格式构造内存117字节Pratt反例及159/160字节实例页邻例 | 是布局/控制流证据；未调用真实Rust decoder、未观察实际panic或资源峰值 / F16/21/23 |
| R4-V03 提交与观察 | 对照cache-hit负依赖guard与host commit；追typed/A2X首次修改→全部fallible返回；追CT实际执行→publish失败→返回字段；补核FnCandidate/closure提交守卫 | F22/24/25/26为静态结论；无Rust/PE/IO注入。B14/B16没有因Result或注释就报新缺陷 |
| R4-V04 语义与资源 | 核54个MIR指令互斥分组、3终结符、11个profile谓词；读source位置/类型/资源拒绝，data/text显式导出39/30分组 | 操作引用/导出守恒不代每条语义通过；全部构造×profile、fault序列与ABI实测仍U |
| R4-V05 CAS与父侧 | 核put/GC/root/lease守卫及清理失败，核A2X两个父adapter的同步等待和后置裁判 | Windows reparse语义、OS/并发、完整父子终态未运行；不宣称必然孤儿或零残留 / F27 |
| R4-V06 来源与并发 | 检查f9fb3854→7833ee65源码差额；F21–27关键7份ACE文件及2份父adapter不变；store/CTE观察与regular/perf组变化另外核。读回另一会话r2后追加，保留具体输入/类型/旧新映射和V-DETAIL-01..04 | 非单一冻结运行报告；F20旧26/26只作历史，当前regular26/perf27。M1正式tag未知不被其他SHA代替 |
| R4-V07 文档与源集对账 | Cargo members与MC01–35路径集合严格相等，无缺项/重复；57个分支ID唯一且与A10/B16/C13/D11/E07严格相等；27个发现均有证据/结论/建议；0–10章节各一、围栏成对、4条V-DETAIL及r2状态/任务保留。补核提交/profile/runtime所引9份源码在两基线间也无差额；结束HEAD仍7833ee65，相关源码/父adapter/测试组无WIP | 只证明记录结构及来源一致，不是独立review或产品测试结果；其他会话文档WIP原样保留 |

以上不新增r2的运行义务或替换V-DETAIL-01..04。真实反例应在后续专属树保留合法邻例，日志、尝试观察与接受态分别比较；本轮没有生成测试、原始报告或材料目录。

### 第五轮实际检查（内部条件与测试义务）

| 检查 | 实际结果 | 未取得的证据 |
|---|---|---|
| R5-V01 来源差额 | 7833ee65→42874419的7个变化逐项读回；新发现生产源码、aggregates_v_rows均无差额；当前ACE源/父adapter/固定组无WIP；核1025正式验收H/C及Spec收据，C→审计源范围diff退出0；有界oracle分档恢复有原owner裁定，不报新F | 非tag存在证明；未重放1025已清理原件；旧轮本Plan所有运行仍未做 |
| R5-V02 变体守恒 | 对ExprKind25/Stmt14逐个分派并跟typecheck/adapter/shared lowering，分别17/14行覆盖；具名条件/错误近邻及现测试入口落§5 | 不是11个profile×全部构造×资源子态的执行笛卡尔积；HIR更细子态仍不得冒PASS |
| R5-V03 控制流反例 | Python内存重放F28有限条件continue例：独立迭代结果99，现合成header路径0；补查adapter死代码分块，排除把无条件尾continue先拒绝当错值证据；F29单点MAX最后合成值2147483648不属i32 | 没有调用ACE前门/compiler/AV/PE，不报告实际错误产物或trap |
| R5-V04 延迟目录/CT子树/融合 | F31 paged origin=u64::MAX、start1/len1：checked加法越界，wrapping为[0,1)；F32 Named hash在发现/替换子树面均缺；F33 Named get.field命中unreachable路径 | 均为结构/算术静态反例，真实reader/前门/Rust panic未执行 |
| R5-V05 测试导航和实际义务 | 35份Cargo/职责组及20包25个显式target路径对账；365份tests源码/2728词法属性口径与hash落§5；四平铺根40/92/38/4全部直接引用；F01–33有不变量/合法邻例/缺口去向表 | 无nextest发现/实际运行/原条件排除数；2728不作编译后分母；逐原case/check/recipe迁移仍未完成 |
| R5-V06 失败与清理 | 补C09/C13/B03、D05/D07/D09/D10的具体状态/准入/失败/回收事实；F30现测试只正常执行，真正资源fault入口另保留 | OS并发/拒删/VirtualFree/provider失败序列及完整ABI运行未做，不用无新发现冒通过 |
| R5-V07 收尾自检 | 0–10章节/围栏/33个F三字段、57个分支ID、35个MC与Cargo集合、AST25/14表、4条V-DETAIL/r2合同字段均核对通过；25个target路径存在，365份tests源码均为跟踪文件；本Plan git diff --check通过。结束HEAD推进到81d8fc03，42874419→HEAD审计源范围diff退出0、相关源码无WIP，其他文档WIP保留 | 自检不是独立review；只有本Plan属于本会话修改，其他会话在途文档未纳入本包 |

本轮未创建探针/材料文件或worktree，未修改实现/测试/配置，未构建/运行ACE。上表内存重放只展示源码控制流的区分性，不能替真实Rust负控。V-DETAIL-01..04、原正式组/规模/平台/ignore及性能分档全部保留。

### 第六轮实际检查（只读工作闭合）

| 检查 | 本轮实际方法与结果 | 保留边界 |
|---|---|---|
| R6-V01 来源与权限 | 冻结81d8fc03f10f4a7d179b0bc3a917602406ed7b91；重核ACE、语言父adapter、auto-test-case、固定组和case目录无源码WIP/来源漂移；本Plan此前各轮WIP及并发其他文档保留 | M1正式验收C及1025收据沿原记录；不创建tag/重签，不修改实现/测试/配置 |
| R6-V02 HIR资源细分 | 17类型、42表达式HR01–42、9语句HS01–09逐个核T/C/K/F、shared MIR ownership及目标消费/拒绝；F34/F35补源码位置、最小形状、结论和建议 | 静态反例/控制流，不称真实错误执行；Box等source物化形状与HIR enum不混计 |
| R6-V03 当前完整case分母 | 内存解析181份case/71份suite全部770个case、1116个check、1133个suite绑定；719file/51inline，12个case带额外files；source/files/expected sidecar和defaults逐项绑定；35注册recipe+7非recipe route均找到实际dispatch | 只读提取不冒Rust reader验收/运行；181个case文件均无CRLF，inline source无换行归一化差额；两个跨suite裸pair复用保完整身份 |
| R6-V04 历史守恒 | 原正式511检查逐项metadata/source/case一致；668固定文件逐一SHA一致；registry69和expanded397检查逐一匹配；legacy332入口/313cell/381引用均落M行 | 原invalid/unsupported/manual/无独立reference条件原样保留，不合并AV/旧VM/A2X/真实机器轴，不以映射改写正式M1凭据 |
| R6-V05 架构与原生身份 | 48个architecture recipe+独立review行共49义务；564处case/check引用全匹配；283个native身份按10个Cargo target根、path及inline mod静态展开到完整包/target/module/fn；U342行/354个来源逐一读hash | 不以同名fn或源码属性数冒编译后nextest发现；没有编译/运行/测试排除数 |
| R6-V06 读取稳定性和表内守恒 | 所有已读取材料在Plan写前再次hash不变；M1116/S71/D181/L313/A48/N283/U342和42 route无缺项/未知分派；完整审计表示SHA256=b85dd7d3cfdc1b0c015f6dd76b077692aa246305506ceff943947079d0f6c542；35发现均有证据/结论/建议，全部落S01–13后继切片 | 审计表示hash不是runner的effective_check/oracle指纹；内容与原文件/Git身份一起核，不以exit0冒功能PASS |
| R6-V07 文档与交接 | 0–10章节、原T-01..05/AC-01..05/V-DETAIL-01..04、MC35+父侧、57分支族和历史记录保留；AC-01..03/T-01..03闭合，AC-04/05不勾；git diff --check与目标文档结构核对 | 执行者自检，不是独立review；仅本Plan为本会话写入，无worktree/材料scope/探针、构建/运行或提交/合入/归档 |

本轮命令只有只读Git/搜索/源码读取、内存解析/比对和Plan定点写入；内存自检成功均不计ACE用例PASS。运行发现/执行/原条件排除=未取得，准备/冷编译/产品运行/清理耗时=不适用；没有首次运行红或重放绿可报告。静态核查的缺陷/风险与真实验证债务已分别记录，用户“暂不修改实现代码”持续有效。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | B01只作跨层公共行为例，不给审计新增运行要求。F01源编辑é→ê的range/replacement重建必须等于新源；算法静态反例、真实Rust/PE结果分账。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | 仅大文件行数/相似字符串/旧注释/自报PASS不直接定缺陷；只按旧range退UTF-8边界但replacement仍断字须展示错误重建，合法ASCII/原样源作邻例。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | 保当前F01–20、MC01–36的已有发现/未核面和原风险分类；不在此修代码/加测试/改变M1出口，未测Rust/PE仍标未测，不把候选当已批准重构。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

- [x] AC-01 / work：冻结来源，覆盖35成员和语言侧父执行器/固定组；源集、人工追踪面、未核面/理由可查，清点不冒全读。
- [x] AC-02 / work：逐项复核F01–15及新增项，标confirmed/candidate/rejected/unknown，提供反例/误报排除；保拒绝理由，unknown不计PASS。
- [x] AC-03 / work：形成按依赖排序的有界重构切片、消费者/测试守恒表，明确1002/985/1017接口、Spec影响、在途冲突，不擅减义务。
- [x] AC-04 / review：独立核源码/高风险反例/遗漏延后绕行/来源漂移；确认债务追加KNOWN-DEBT，不覆盖其他WIP；假设不冒确认缺陷。
- [x] AC-05 / merge：经review归档分析结论，落路线导航、核所有输出路径并清理材料；未review不称完成。（2026-11-17 T-05执行：r2 final pass后落主f046f4ae4并归档；输出路径对账=无worktree/无plan-artifacts scope/无散落目录，收据见§9）

首轮已执行：取号成功；跟踪Rust源清点、Cargo正常path依赖提取、20行归一块比较、关键入口人工核查；Python仅内存逐句重放F01得reconstructed='�' != 'ê'。这是算法反例，不是Rust集成测试。未改生产/测试源码/canonical Specs；未编译或运行产品/性能。首轮证据直接折入本Plan，没有附件。

### 第二轮验收进度

AC-02本轮完成的含义是**15项全部有定向静态证据、裁定、建议和未验证边界**，不是15项均为缺陷、更不是修复完成。F01/02/06缺口确认（动态影响边界保留），F03/04/05/07/08/09/10/11/13/14/15确认各自描述的结构事实；收益、替代方案及全量覆盖率仍candidate/unknown；F12确认既有复用基础。各项被否定的过度解释在§5保留。

AC-01仍未完成：35成员全量清点已做，但非每成员的语义/失败分支、全语言侧consumer及测试义务已穷尽。AC-03仍未完成：已形成小步方向与owner建议，完整影响/迁移表、后续实施合同和成本基线尚缺。AC-04/05保持未做，本轮不冒独立终审/归档。

### 第三轮验收进度（历史轮次）

MC记录覆盖为35/35，外加MC36；每成员入口/职责、实际消费者或链接消费者、状态/拒绝路径、测试导航、剩余项均已补记。新增F16–20也按证据/结论/建议和未运行边界完成本轮静态裁定，AC-02保持完成。

AC-01维持未勾：成员清单已齐，但COV-A..E列出的逐语义/故障/恢复和测试义务闭包尚未完成，不能用记录齐全冒全深审。AC-03维持未勾：目标复用切片已有候选顺序，但原case/check/recipe到新位置的完整迁移表和有界实施合同仍缺；AC-04/05不变。未通过发现数量增加来宣称覆盖提升，覆盖证据是逐成员具名行和剩余项。

### 第四轮验收进度（历史轮次）

完成本轮57个分支族静态深审记录和F21–27证据/裁定/建议；MC35/35+父侧MC36保持，AC-02保持勾选。确定缺口与候选收益分账：F21/22/24/25先列正确性风险，F27列父侧本地控制缺口并保集成后果待核，F23/26分别列reader严格性和观察事实错误；没有将7项冒真实运行失败。

**不宣称“ACE所有内部代码分支已经深审完成”。** B13所有AST/HIR子分支与profile组合、B03/05/06资源/权限子态、A05全部分页/子树畸形组合，以及C09/C13、D05/07/09/10的深层失败条件仍有具名静态或运行缺口；E07完整原case/check/recipe去向表未闭合。57是审查单元数量，不是穷尽的CFG分母。AC-01/03继续未勾，T-02/03继续进行；AC-04独立复审与AC-05归档尚未执行，status/current_step/total_steps维持executing/1/5，plan_revision保另一会话的2。

### 第五轮验收进度（历史轮次）

新增F28–33六项均有证据/结论/建议，AC-02保持。B13不再只有“源码39个变体可定位”：每个分派与内部条件有表；A05已定位分页与内联的准入差异；C09已定位Named子树遗漏；C13、D05/07/09/10已补具体失败与清理状态。E07补35成员→既有target/组、4个平铺root与F01–33义务去向，保留词法/编译发现/执行/原条件排除的区别。

静态阅读、发现确认、真实执行、修复关闭是四种状态：F28/29/31/32/33为静态机制风险，F30为该test的证据缺口；均未动态复现/修复。仍有HIR/资源细分组合、完整原case/check/recipe的逐行去向与真实反例执行待闭合，不能因AST表/测试target表齐全宣布所有代码分支通过。AC-01/03仍未勾，T-02/03进行；AC-04/05未做，status=executing、revision=2、current_step=1/5维持。

### 第六轮验收进度（历史轮次）

AC-01完成：固定81d8fc03及无漂移源码，MC01–35/父侧MC36、57分支族、AST与HIR全部分派、各域资源/授信和失败清理均有可查静态分区；原P/U按§5闭合表转为具体动态/后继义务。AC-02完成：F01–35均有证据/裁定/建议和未运行边界，未把候选收益升级成缺陷。AC-03完成：完整逐case/check/recipe保留账、S01–13依赖/owner/consumer/独立关闭oracle和Spec候选，以及1002/985/1017接口已补齐。

**用户所要求的只读深审已闭合**，不再留“HIR待细分/原check待映射”作本包未完事项。未知的动态后果/覆盖率、真实反例和修复继续按S片登记，不计运行通过、不声称所有CFG条件组合已经证明。AC-04独立复审和AC-05合入归档仍未执行；因此工作阶段execution_done、3/5，整个Plan尚未reviewed/archived。语义范围/AC/任务分母不变，revision仍2。


### 第七轮验收进度（当前）

AC-01..03重新核对通过：成员/源、HIR17/42/9及资源域、M1116/S71/D181/全部旧义务静态守恒；AC-04已按§9原始来源重建并登记债务候选。审计交付通过，不代表35项产品风险已修或动态PASS。状态reviewed、4/5；T-05/AC-05仍未执行。

“1024所有深入审查已完成”的准确范围：本Plan约定的静态模块/分支族/资源分区/逐项迁移义务审查已完成；不是每个Rust符号逐行全读、不是所有CFG/条件笛卡尔积证明、不是编译后的全测试发现/执行认证。真实Rust/PE、fault/OS/并发、性能收益及修复由承接Plan逐个关闭。同一上下文重新核证不能冒不同agent独立外审；本次没有另起agent或chat。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 文件、操作、产物、验证 / AC |
|---|---|---|
| T-01 / work | [x] 固定源清点完成；R5补正式验收C=b923362f4，实际tag本地未发现/1025未创建 | 本Plan§4/5固定SHA及1025/Spec收据；C→42874419审计源范围diff为0；git ls-files/Cargo解析/git diff重核成员/源集/父侧并记hash；不把tag-ready冒实际tag；AC-01 |
| T-02 / work | [x] R6只读闭合；HIR17/42/9资源/授信及跨consumer补齐，F34/35归类 | core/syntax/query/lowering/av-core/av/native逐组生产者→消费者；F01–09/13及F21–24/28–29/31–35有证据/裁定/建议；动态反例明确后继、未运行；AC-01/02 |
| T-03 / work | [x] R6只读闭合；逐原case/check/recipe账及有界后继S01–13完成 | artifacts/CTE/ABI/A2X/runtime/worker/语言父侧、全部MC与原511/668/legacy/architecture/当前1116checks守恒，1002/985/1017/Spec及在途边界明确；AC-01..03 |
| T-04 / review | [x] 第七轮按原件重建复审通过；同执行上下文限制明示 | 独立按真实代码/反例复核覆盖、拒绝项、未核项和漂移，按auto-plan-review写§9及债务，核SD-00；AC-04 |
| T-05 / merge | [x] 2026-11-17完成：落主f046f4ae4+归档本提交，收据见§9 | 分析折入本Plan、路线导航、归档archive；有树/材料先路径对账与guard清理，归档不等于批准重构代码；AC-05 |

当前T-04复审已完成，下一门为T-05合入/规范沉淀/归档；本次用户仅要求复审与分配，未执行T-05。实际代码修复依§5逐项主计划承接，仍未获本轮实施授权。后继开工先核本轮冻结源之后漂移，代码/探针/测试均在各自唯一专属树，不在主检出顺手修改。


r2规划时交接（历史）：原任务ID/阶段/进度保持；T-01须在依赖实现前把具体输入/类型/旧新check与未决来源冻结；T-01/T-02/T-03分别落实对应V-DETAIL-01..04与原所有验证。第六轮已交付上述只读部分，当前状态以本节任务表为准。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-1024 r2：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原5任务/全部AC与V/依赖/阶段/预算和executing 1/5；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12：stage=new，revision=1，outcome=pass，next=work。合同与首扫F01–15可审阅，T-01..05/AC尚未全部完成。这是规划自检和背景调查，**不是独立实现review、全量审计通过或M1签章**。草案保drafting，首扫作为背景事实，不把未来work/review/merge勾完。

### 第二轮执行记录

2026-11-12：stage=work，revision=1，outcome=partial，status=executing，current_step=1/5。用户授权已落实为只读审计；固定源f9fb3854，完成F01–15逐项定向复核（先F01/02/06），更新§4来源、§5证据/裁定/建议、§6方法与结果、§7验收状态、§8任务进度。没有改变目标/阈值/支持承诺，故不增语义revision。

关键纠正：F06限定测试外层，内层原子提交保留；F08否定事实失败静默成功fallback；F09确认artifacts/subproc已有公共executor委托；F11否定同源即同合同；F14不以静态词搜冒全仓缺例/覆盖统计。未授权延期/删除原义务，没有新实现workaround；确认风险作为本Plan债务候选保留，正式债务表登记归T-04，不提前修改其他会话的债务WIP。

本记录为执行者深入核查和文档自检，不是独立review verdict。T-04独立复审、T-05归档/清理未执行；不宣称整个Plan execution_done或M1验证失败。下一责任为分析执行者补T-02/03覆盖闭包；实际代码修复仍不在本轮。

### 第三轮执行记录

2026-11-12：stage=work，revision=1，outcome=partial，status=executing，current_step=1/5。沿用f9fb3854固定源，完成35成员与MC表一一对账，逐项补记入口/状态/消费者/测试，另列MC36；新增F16–20，并把剩余工作改为COV-A..E按模块闭包推进。目标/AC未改，语义revision不增加，不新取Plan号。

新增明确风险是语法decoder按内部计数提前预留、handle零租约索引残留、A2X编译与SHA分次读；A2X请求预读预算与ACE聚合选择差额分别保当前工具/职责边界。正向保留：CT资源/cancel不混语言裁决、正负依赖/恢复重验、strict语法投影、typed runtime先限长再读、已有provider/store/executor；没有以“优化”删除这些机制。

本轮只更新本Plan，未修改实现、测试、配置、canonical Specs或其他计划；未建worktree/材料scope，未构建/运行ACE，不提交/合并/归档。独立复审T-04、所有动态反例和原正式门均未执行。每MC剩余项及COV-A..E是当前交接，不把本轮记录解释成全模块深审通过。

### 第四轮执行记录

2026-11-12：stage=work，revision=2，outcome=partial，status=executing，current_step=1/5。用户授权“完整分支深审”，已将COV-A..E展开57个具名分支族，补提交前提/profile/runtime导出/lease状态，新增F21–27并按证据/结论/建议分别落§5，实际检查见§6，剩余内部条件及覆盖闭包见§7/8。scope/AC未缩小，未新增语义revision，保留并发r2规划。

执行期间源码由f9fb3854推进7833ee65：新发现关键文件无变化，真实变化分别核对，不把不同源拼成一次PASS。Plan首次写入hash防护发现r2变化即中止该次写入，读回r2后仅追加/定点修改；其他会话的具体输入、类型/职责、旧新映射、V-DETAIL、任务/阶段/进度均保留。没有修改实现/测试/配置/其他文档，没有探针文件、构建/运行、材料scope、worktree、提交/合并/归档；本记录仍非独立review verdict。

### 第五轮执行记录

2026-11-13：stage=work，revision=2，outcome=partial，status=executing，current_step=1/5。承接用户指出的内部条件/测试义务未闭合项，固定42874419并与前轮核差额；完成AST25表达式/14语句的逐分派记录、分页/CT/foreign/CAS/runtime失败状态补审、35成员测试target/组及F01–33义务去向。新增F28–33，其中5项为静态正确性/受控拒绝风险、1项为实际测试证据缺口。内存反例没有运行ACE，不将“待补真实反例”写成已PASS/已观察panic。

仍只改本Plan；其他计划/Specs/源码/测试/配置原样保留，无worktree/材料scope、构建/运行、提交/合并/归档。保持r2全部合同、AC/V、任务ID/owner_stage/状态/预算和既有阶段记录；本记录不冒独立review。另核1025/Spec正式验收C来源，与本轮审计源差额为0；实际tag本地未发现、不创建不推送，旧“整个正式来源unknown”已纠正。剩余HIR/资源细分、逐原check迁移、真实运行和后续来源漂移在§7/8/10具名保留。

### 第六轮执行记录（只读工作交付，非独立review）

2026-11-13：stage=work，revision=2，outcome=execution_done，current_step=3/5，next=T-04 review。按用户授权把剩余HIR资源细分和逐case/check/recipe守恒补至闭合；§5记录17/42/9矩阵、F34/35三字段证据、完整映射和13组有界后继建议；§6记录实际内存核对，§7按原AC验收只读工作。F01–35后继修复/动态oracle保留，未把未运行项勾成PASS，没有删减原T/AC/V/规模/条件或跨阶段冒完成。

来源固定81d8fc03f10f4a7d179b0bc3a917602406ed7b91；原正式C→上轮→本轮已核范围无差额。完整审计表示SHA256=b85dd7d3cfdc1b0c015f6dd76b077692aa246305506ceff943947079d0f6c542，用于本附录自检；原文件/S/D/固定Git源仍为权威。独立review须核源码与表，不能将此摘要hash当运行结果或复审签章。

本会话只写本Plan，主检出其他会话文档WIP保留，相关实现/测试/配置无WIP；所有提取与验证均内存完成，没有探针/原始日志/报告/fixture或其他输出目录、worktree或plan-artifacts scope，故无本轮材料迁移/删除。未构建、未运行ACE/性能、未创建tag、未提交/合入/改canonical Specs/债务表/归档。T-04独立复审与T-05合入归档仍待执行，本记录不能作为它们的pass；只读工作闭合不授权代码修复。


### 第七轮独立核证记录（final，只读审计交付）

- stage=review；review_scope=final；plan_id=PLAN-1024；plan_revision=2；outcome=pass；next=T-05（本次未执行）。
- reviewer_identity=本会话Codex；limitations=与执行相同会话/上下文，无另起review agent/chat。逐原文件、代码和Spec重建结论，未将执行者摘要/勾选作证；不冒不同审查者独立上下文。
- worktree_path=D:/autostack/auto-lang；branch=master；reviewed_commit=base_commit=81d8fc03f10f4a7d179b0bc3a917602406ed7b91。git worktree list未有1024实施树：本包仅主检出Plan簿记，无未落地实现树或产品diff，故不据此虚构缺树阻塞。
- 受审文档为未提交只读分析交付，提交内容SHA256=fa81b05a9c63c22b641eec91c21e58e9c3c7a8adb8ebd66710835f4665977bbf，不以HEAD替代文档身份；产品/父adapter/auto-test-case/组/case源均已提交且无WIP。后加逐项分配是用户本轮授权的规划簿记，表身份=0f7e524f15bb0defc6565e9c96dda007920d8aeb5fe6d9af6f681e85efed6abb。
- dependency_revisions：review时1002 r7、985 r8、1017 r3；分配后969 r7/970 r6/978 r9/985 r9及新1027–1038 r1、1002 r7联络注记，均非实现完成证据。Spec源见下表；supersedes/new/touched保持空，SD-00原行SHA256=b48788401727b72089ba77e9b51c7bacc30d5571338effbbc11ff15e2c0ab70c，无规范行为增量。

| AC / task | verdict | 原件核验及实际结果 |
|---|---|---|
| AC-01 / T-01/02 | pass（静态范围） | Cargo35+父侧MC36、57分支族与HIR17/42/9逐枚举；核恢复/授信/失败/清理及消费边界；未读面/动态未知明确，不称逐行全读 |
| AC-02 / T-02/03 | pass（分类交付） | F01–35各证据/结论/建议与原源核对；UTF双端、capture恢复、负依赖commit、四类reader、三会话事务、Named/loop/record分类及poll重点回读；机制事实与动态后果/收益分开 |
| AC-03 / T-03 | pass（建议/守恒） | 从原件重算770case/1116check/71suite、35recipe/42route；668输入pin、原511及expanded397/registry69/legacy313/architecture48/N283对齐；有界切片、consumer/保留义务和1002/985/1017边界完整；本轮逐项分配补具体承接 |
| AC-04 / T-04 | pass（限本审计） | 扫遗漏/延后/绕行；8个内存负控=缺项/重复/换ID及source/case/adapter/action/recipe五种错身份，全部被拒；正控通过；KNOWN-DEBT登记分类和主owner；仅文档健康检查适用，无新增编译告警/debug产物可查 |
| AC-05 / T-05 | pending merge-owned | 未合入、沉淀、归档或清树；没有把真实代码正确性留在merge验收 |

实际命令/结果：只读Git/source/Spec读取、内存解析/重算及8个mutated-copy对照均exit0；两份native binary完整体相同，SHA256=058843c622dfa5398afd9ea3ce317223f58bc30ae88088110e017c00063efbe6。完整映射重算SHA256=b85dd7d3cfdc1b0c015f6dd76b077692aa246305506ceff943947079d0f6c542，与受审附录逐字一致。UTF/U64/continue独立内存模型仍仅静态区分；没有ACE运行/大内存申请/产品测试/正式压力，八个负控不是八个产品case。

新增审计交付阻塞finding=0；原F01–35是后继产品风险/结构事实/候选与保留实践，不因审计pass关闭。F14原“当前缺全量表”为第二轮历史时点，第六轮已补；F20的26/26为旧轮，不覆盖当前regular26/perf27与1025分档。已将旧“当前结论”标题标明历史，未更改AC/语义revision。未来若产品/Spec/输入漂移，受影响证据失效并重新核查。

| spec_inputs | SHA256 |
|---|---|
| docs/specs/auto-ac/native-backend-and-retirement.md | 77ba8a3c2fa19128b7bbc958d21c5d928de788157daa39b6978b9d8fb0731a78 |
| docs/specs/auto-ac/closures-function-production.md | 8e29772c3dea9f138e055912b134ca957267e35496dde958b7b4ef8bdbb4321f |
| docs/specs/auto-syntax/snapshot-cst.md | 8b158de818761fa468e3b95e1efe84f48f8857fc30b55d8c34b9fcf819cf416c |
| docs/specs/auto-semantics/source-query-session.md | 1a5375476e35f39015136c89f9e1a868e73c8b9448789c87d8675d441e90e8b9 |
| docs/specs/auto-a2x/incremental-and-cache.md | c8bdda7ae802b12e2d4f1cb16801104f359247b256ce8abd67f025a42bc8657e |
| docs/specs/auto-a2x/checked-resource-facts.md | d79eb509e7cb8242297320638601ffbc7c4b5df0626a22911e0adb9acd733704 |
| docs/specs/auto-cte/source-query-integration.md | a605853533d7f70a0f9ccce5d825d1cf461741910e434800d65323bbd312c6ae |
| docs/specs/auto-test-case/architecture.md | 2e37d29c0df993f95809bf262af6a53b4b53acebf353453dcfc3e1626bc89fe4 |
| docs/specs/auto-test-case/ace-test-policy.md | 8750821af761ab59d38da01a819b7b298da0822f1cd12a8d6c9241530736d46f |

证据全部内存处理并折入Plan，无额外脚本/日志/报告/fixture、worktree或plan-artifacts。KNOWN-DEBT逐项登记不批准延期、也不把缺陷转为完成；所有代码修改保持未执行。未提交/合入/改canonical Specs或legacy JSON/tag/归档。

2026-11-13 分配后目标文档终检：35项主承接/关闭case唯一且均可回到计划正文，已有计划6项/新计划29项；1027–1038共12个唯一编号、72个未执行任务，均drafting 0/6。新文件0–10章节/AC/任务/源与Spec路径及本地链接核对通过；969/970/978/985原任务ID/顺序/全部原AC与0进度保留，1002仍r7四例七检查/0 of 4。受影响计划依赖递归177节点无缺失/环，兼读历史930实际存放的archived路径作取材，不新增该旧目录或重开归档计划；12份新计划均无1002整包硬依赖。目标git diff --check及新文件空白检查通过；81d8fc03→当前HEAD审计源范围diff=0、相关源码无WIP。上述检查命令exit0，只证明规划结构/来源，不是产品测试。并发其他会话的964/967/947/1017及Python规划文档保留，未纳入本次写入。

2026-11-16 stage:planning-handoff / 1024审计后继阶段归属簿记：原r2只读审计/reviewed 4-of-5、35发现/原历史记录及T-05未执行事实保持。用户本轮在[985 r23 A-FREEZE](../985-ace-m2-certification.md)重冻A/B分母：F01/02复用1027/28已交来源；1029、1031–1037为A正确性；1030历史入口、1038研究B；1066正确性A片段与格式迁移B分开。此指针不是重新作审计、修复或merge，也不把原发现表中审计时“未修”倒填成已修。1024 T-05可交审计来源而不等待全部修复；A最后只收已审正式快照及各实际修复/反证来源。

本轮阶段归属簿记静态核对：985 r23的162父义务与35发现映射完整，13份Plan原92任务/状态/完成数保持，来源/阶段图无环。仅更新此交接指针与规划记录；不重开原审计review、不新增实现/产品结果，原T-05尚未执行。


2026-11-17 stage:planning-status-sync | scope:1024→1027..1038修复优化路线 | source_HEAD:e85096628c468376c0c3632d832420a2e7f6d439 | 方法：逐Plan元数据/末review-merge清理收据、九个正式交付提交祖先及worktree登记只读核对 | 结论：9包交付归档、1030/1037/1038仍drafting0/6；当前建议1037→1030→1038，1024 T-05文档收尾独立；1038只读研究可提前，无新增整包硬依赖。1034头部3/6与终态收据差额按收据解释，不重开归档。原Task/AC/Case/接受范围及进度保持；未建树、修改实现/测试/配置/canonical Spec、运行ACE/构建/性能或提交/合入/清理。本轮仅状态/导航簿记，不签新的实现/独立终审PASS。


2026-11-17本轮状态同步静态核对通过：九个交付提交祖先、17份来源Plan（本轮四份计划变更前冻结；其余13份保持字节不变）、12包状态9已交/3未开工、原35项发现分配、三草案18任务/18AC/原Case、章节/围栏/新增链接与quality-route锚点一致。只修改五份计划/路线文档；未改归档/实现、未运行ACE或构建/性能；此检查不是新产品验收。


### 2026-11-17 T-05 merge 收据（PLAN-1024:r2，/goal auto-plan-merge）

- prepared：复核基线=r2 final review pass（2026-11-13 §9第七轮，绑定审计源81d8fc03/文档SHA256 fa81b05a…，AC-04 pass、AC-05 pending merge-owned）；其后增量仅为2026-11-16 planning-handoff与2026-11-17 planning-status-sync两笔§9簿记+§0/§5路线表（各轮自证不修改审计结论/分母/实现），属文档性后代，实现/依赖无变化（本包无实现），按merge技能文档性后代规则准予直接交付；依赖空。
- landed：delivery_commit=f046f4ae45a1a50ae97b83898625cdd9a211a873（docs(plan) 落主，+3004/−22，master tip=delivery commit）。本包为只读审计、全程无实施worktree/分支（r2 review收据已核worktree list无1024树），无ff合并对象，文档直接落默认分支。
- canonical_verified：批准的no-impact结论——supersedes/new/touched均空，SD-00原行SHA256=b4878840…无规范行为增量（r2 final review核证）；审计知识的持久载体=本归档Plan+§5逐项分配到1027–1038各承接Plan，无需docs/specs/写入。9份spec_inputs仅为审计输入（r2收据有SHA表），非本包delta。
- archived：git mv至docs/plans/archive/1024-ace-m2-code-quality-and-refactoring-audit.md，status=archived、current_step=5/5，AC-05/T-05勾闭；内链按归档位置重写（仓库根 ../../→../../../、活动计划→../、归档同侪去archive/前缀，含1002/1017实为已归档的纠正），全部25个相对链接从新父目录解析exit0；外部锚点#quality-route-20261117随文件保留。
- cleaned：无实施树（worktree list核对，从未存在，无需wt-guard）；plan-artifacts/plan-1024/从未创建（磁盘核对无该scope），登记表无1024条目→无可删/可刷新；散落输出对账：D:/autostack顶层与plan-artifacts无1024命名物，D:/无autostack-build，各轮均内存处理（§3/§6声明相符）。
- 批量回归due检查：收据2026-11-16T10:04Z、last_covered=1064；其后合入仅1037/982/983/984/1066簿记，无id%5==0，收据龄<48h → 不到期，未触发cargo t all。
- 着陆检查/陈旧产物观察：本交付为纯文档，不影响任何生产面产物（后端二进制/web bundle），无需重建/重启（不在授权范围）。
- 共享导航簿记：KNOWN-DEBT/ace-route-plan-map/ace-m2-delivery-roadmap/1030/1038中指向本文件的链接路径已在工作树机械修正（1037归档同款archive/前缀），保留为其owner会话未提交WIP一并落主；归档计划内 ../1024- 旧式链接按仓内既有惯例保留（archive/973→../1064先例）。
- stage: goal | plan_id: PLAN-1024 | plan_revision: r2 | round: 1 | from: review | to: merge | status: archived | outcome: pass | worktree: 无（只读审计，主检出文档交付） | commit: f046f4ae4+本归档提交 | next: done

## 10. 待澄清事项

无阻碍只读调查的用户决策。当前未决技术事实：

- 正式M1来源本轮已核：1025/Spec收据的C=b923362f4，与42874419审计源范围无差额；收据不是本轮重跑，实际tag本地仍无匹配且1025未创建/推送。未来tag存在/目标或后续源码漂移须据新事实对账，不再把当前正式验收C列unknown。
- F01真实支持源码的fresh/warm、诊断/PE实际表现；F02正式测量是否触发同类嵌套或unwind后继续；F06真实trap/持久化失败后重试。机制缺口已有证据，动态复现未做，不再笼统称所有内容“尚未核查”。
- 864仓外消费者、会话拓扑删除是否承诺；F03结构化错误的相容方案；F10旧wire EOF/编码/IO行为是否需升级。
- typed HIR builder成本收益、crate边界真实构建成本、runtime共享抽取的符号/取消/预算相容性仍须后继测量；全量测试保留映射已在第六轮闭合，实际格式搬迁不属于本轮。

- 新增F16内部计数预算/分类、F17活租约与索引回收、F18真实两读错身份的父侧结果、F19one-shot请求/source合法上限、F20聚合用途与职责包选择，均须保留现有相容承诺再定方案。
- MC01–35、MC36的COV-A..E静态审查及case/check/recipe闭包已齐，见第六轮闭合表；真实编译发现/动态覆盖率仍unknown，不能由静态表推成“所有分支通过”。
- F21真实decoder拒绝/FirstSets结果、F22负依赖翻转提交、F23超限/tag/尾字节、F24批量晚失败/同批重试、F25provider/anchor失败保旧、F26sink失败事实、F27deadline/输出/进程树终态：静态证据已落，真实执行全部未做。
- D09 owned_free在OS/provider拒绝后失去ledger项的合同与收敛策略、D06目录计量reparse变更的Windows实机边界，静态状态已读、实际终态未测，归S04–07相关owner和独立复审的风险归类；不得拿正常释放或源码导航统计代替。
- F28/29的for-in continue/latch与MAX结束、F31分页条目准入/溢出、F32 Named需求发现/回灌、F33 Named get.field受控结果均须真实原入口反例与合法邻例；F30须核现fault用例对聚合嵌套形态的实际覆盖，不能只依据标题迁移。

- F34 List-only资源record的真入口move/clone/投影与fault影响；F35条件continue的真实poll/WU/cancel轨迹及清理。两者静态证据已核，动态严重度/后果待S11/S06有界合同，不声称已观察双free或挂死。

以上为已完成只读审查公开保留的技术未知/后继实施验证，分别落§5的S01–13，非T-02/03仍有未读或未映射行。T-04核证及债务分类登记已完成，下一门是T-05合入归档；不放宽945–948门、不修改其他计划状态，也不把本轮静态审计当代码修复授权。
