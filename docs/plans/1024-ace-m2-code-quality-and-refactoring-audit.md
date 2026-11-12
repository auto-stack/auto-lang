---
plan_id: PLAN-1024
status: executing
feature_name: ACE M2 已实现代码质量、架构与复用审计
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 2
current_step: 1
total_steps: 5
depends_on: []
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [ACE, auto-test-case]
---

# [PLAN-1024] ACE M2 已实现代码质量、架构与复用审计

## 0. 变更摘要

2026-11-12 r2：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

归属 **M2-A工程质量支撑**，允许在M1最后测试期间提前做只读分析。本包交付代码审计与可执行重构建议，不直接重构编译器，不接管M1测试/签章。2026-11-12已完成规划所需的首轮结构扫描与热点抽查，结果见§4/5；不是逐文件、逐符号审计完成声明，后续深审仍按§8执行。

2026-11-12第二轮：用户授权逐项深入审核F01–15，先正确性、再架构/复用，明确暂不改实现。已完成15项的定向静态复核，当前结论见§5“第二轮逐项审核”，方法与实际结果见§6/7。本轮不是整个workspace逐符号审计或独立终审。

2026-11-12第三轮：按Cargo全部35成员建立MC01–35独立记录，补语言父侧MC36，逐项补核入口/消费者、状态与拒绝、复用及测试归属；新增F16–20。已完成成员清单与定向静态补审，仍保留每成员未核分支、实际执行证据与完整语义/测试义务闭包，不能称全模块深审完成。

2026-11-12第四轮：按“完整分支深审”要求，将COV-A..E展开为57个分支族，新增F21–27，逐项写入证据、结论、建议和反例入口；覆盖恢复解码、提交依赖、会话事务、CT观察、父侧等待及清理。所有35成员及MC36均有归属，内部子分支/测试义务的未闭合项仍明确保留。只读静态审核，不把分支族清单当穷尽所有内部条件或运行覆盖。期间源码主线推进到7833ee65、Plan被另一会话细化为r2，已分别核对并保留增补。

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

- [experimental/ace/crates/native/src/closures_incremental.rs](../../experimental/ace/crates/native/src/closures_incremental.rs)：F01 UTF-8编辑/原实现；读取SHA256前16 `dd5a2747a9ca4a01`

- [experimental/ace/crates/av/src/closures_fn_session.rs](../../experimental/ace/crates/av/src/closures_fn_session.rs)：既有边界算法复用比较；读取SHA256前16 `c325ca50dd9059fd`

- [experimental/ace/crates/core/src/lib.rs](../../experimental/ace/crates/core/src/lib.rs)：嵌套/panic观察作用域候选；读取SHA256前16 `906442f4d5f2ab0f`

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

### 第二轮逐项审核（当前结论，固定来源f9fb3854）

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

### 第三轮按成员覆盖清单（当前，MC01–35）

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

### 第四轮分支深审（当前，承接r2）

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

### 建议实施次序与交接

| 顺序 | 有界切片 | 与现有工作关系 |
|---|---|---|
| 1 | F01/F02/F06正确性与观察：先真实反例，再定修复 | M1风险交原owner，不在本轮顺手改代码或改变出口 |
| 1a | F16/F21恢复准入、F22负依赖、F24/F25候选事务；各自独立修复合同 | 先保合法邻例/当前旧态，再补晚失败反例；不为统一机制扩大profile或恢复授信 |
| 1b | F18/F19/F27父子身份与有界终态；F23/F26严格reader与事实归因 | 保one-shot/预构建worker/父oracle，复用已有executor；不把infra归语言负例 |
| 2 | F04/F10/F11共用机制 | 先冻结保留合同；避免与1002同时大改作者格式和执行器 |
| 3 | F03/F05/F08/F09/F12主通路 | 与962–978能力owner核接口，不接管WIP；重大决定先设计 |
| 4 | F07/F13/F15往返/模块/注释清理 | 小步保持身份；1017测真实成本，本包不报加速收益 |
| 全程 | F14义务守恒 | 1002管格式，985管覆盖，1017管成本，本包管架构审计与候选，不互代验收 |

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r2当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

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
| [experimental/ace/crates/native/src/closures_incremental.rs](../../experimental/ace/crates/native/src/closures_incremental.rs) | F01 UTF-8编辑/原实现 |
| [experimental/ace/crates/av/src/closures_fn_session.rs](../../experimental/ace/crates/av/src/closures_fn_session.rs) | 既有边界算法复用比较 |
| [experimental/ace/crates/core/src/lib.rs](../../experimental/ace/crates/core/src/lib.rs) | 嵌套/panic观察作用域候选 |

**状态、错误与资源。** 仅大文件行数/相似字符串/旧注释/自报PASS不直接定缺陷；只按旧range退UTF-8边界但replacement仍断字须展示错误重建，合法ASCII/原样源作邻例。 保当前F01–20、MC01–36的已有发现/未核面和原风险分类；不在此修代码/加测试/改变M1出口，未测Rust/PE仍标未测，不把候选当已批准重构。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [experimental/ace/crates/core/tests/cases/source_instances_935.rs](../../experimental/ace/crates/core/tests/cases/source_instances_935.rs) | 现职责/实例不变量与测试接缝 | Rust断言来源，具体test/check由原测试模块核；SHA256前16 `0b559ecd6575bd90` | 沿F/MC原审计ID核来源/反例：V-DETAIL-01..04；不注册运行case；原任务 T-01/T-02/T-03 |
| [crates/auto-lang/src/tests/trait_vm_tests.rs](../../crates/auto-lang/src/tests/trait_vm_tests.rs) | 公开语义与内部实现不同，旧VM不是审计全局oracle | Rust断言来源：test_spec_default_method_inherited_and_overridden, test_abstract_spec_method_still_enforced, test_bounded_generic_fn_dispatches_on_runtime_type；SHA256前16 `a1f3924261667a1c` | 沿F/MC原审计ID核来源/反例：V-DETAIL-01..04；不注册运行case；原任务 T-01/T-02/T-03 |
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

准备/构建/运行/清理分账；正式性能仍由1017与原ace-performance合同显式执行。用户未指定独立reviewer模型，本包不自行开子agent/chat；最终独立复审需另行按review范式执行。

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

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | B01只作跨层公共行为例，不给审计新增运行要求。F01源编辑é→ê的range/replacement重建必须等于新源；算法静态反例、真实Rust/PE结果分账。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | 仅大文件行数/相似字符串/旧注释/自报PASS不直接定缺陷；只按旧range退UTF-8边界但replacement仍断字须展示错误重建，合法ASCII/原样源作邻例。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | 保当前F01–20、MC01–36的已有发现/未核面和原风险分类；不在此修代码/加测试/改变M1出口，未测Rust/PE仍标未测，不把候选当已批准重构。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03；T-01/T-02/T-03 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

- [ ] AC-01 / work：冻结来源，覆盖35成员和语言侧父执行器/固定组；源集、人工追踪面、未核面/理由可查，清点不冒全读。
- [x] AC-02 / work：逐项复核F01–15及新增项，标confirmed/candidate/rejected/unknown，提供反例/误报排除；保拒绝理由，unknown不计PASS。
- [ ] AC-03 / work：形成按依赖排序的有界重构切片、消费者/测试守恒表，明确1002/985/1017接口、Spec影响、在途冲突，不擅减义务。
- [ ] AC-04 / review：独立核源码/高风险反例/遗漏延后绕行/来源漂移；确认债务追加KNOWN-DEBT，不覆盖其他WIP；假设不冒确认缺陷。
- [ ] AC-05 / merge：经review归档分析结论，落路线导航、核所有输出路径并清理材料；未review不称完成。

首轮已执行：取号成功；跟踪Rust源清点、Cargo正常path依赖提取、20行归一块比较、关键入口人工核查；Python仅内存逐句重放F01得reconstructed='�' != 'ê'。这是算法反例，不是Rust集成测试。未改生产/测试源码/canonical Specs；未编译或运行产品/性能。首轮证据直接折入本Plan，没有附件。

### 第二轮验收进度

AC-02本轮完成的含义是**15项全部有定向静态证据、裁定、建议和未验证边界**，不是15项均为缺陷、更不是修复完成。F01/02/06缺口确认（动态影响边界保留），F03/04/05/07/08/09/10/11/13/14/15确认各自描述的结构事实；收益、替代方案及全量覆盖率仍candidate/unknown；F12确认既有复用基础。各项被否定的过度解释在§5保留。

AC-01仍未完成：35成员全量清点已做，但非每成员的语义/失败分支、全语言侧consumer及测试义务已穷尽。AC-03仍未完成：已形成小步方向与owner建议，完整影响/迁移表、后续实施合同和成本基线尚缺。AC-04/05保持未做，本轮不冒独立终审/归档。

### 第三轮验收进度（历史轮次）

MC记录覆盖为35/35，外加MC36；每成员入口/职责、实际消费者或链接消费者、状态/拒绝路径、测试导航、剩余项均已补记。新增F16–20也按证据/结论/建议和未运行边界完成本轮静态裁定，AC-02保持完成。

AC-01维持未勾：成员清单已齐，但COV-A..E列出的逐语义/故障/恢复和测试义务闭包尚未完成，不能用记录齐全冒全深审。AC-03维持未勾：目标复用切片已有候选顺序，但原case/check/recipe到新位置的完整迁移表和有界实施合同仍缺；AC-04/05不变。未通过发现数量增加来宣称覆盖提升，覆盖证据是逐成员具名行和剩余项。

### 第四轮验收进度（当前）

完成本轮57个分支族静态深审记录和F21–27证据/裁定/建议；MC35/35+父侧MC36保持，AC-02保持勾选。确定缺口与候选收益分账：F21/22/24/25先列正确性风险，F27列父侧本地控制缺口并保集成后果待核，F23/26分别列reader严格性和观察事实错误；没有将7项冒真实运行失败。

**不宣称“ACE所有内部代码分支已经深审完成”。** B13所有AST/HIR子分支与profile组合、B03/05/06资源/权限子态、A05全部分页/子树畸形组合，以及C09/C13、D05/07/09/10的深层失败条件仍有具名静态或运行缺口；E07完整原case/check/recipe去向表未闭合。57是审查单元数量，不是穷尽的CFG分母。AC-01/03继续未勾，T-02/03继续进行；AC-04独立复审与AC-05归档尚未执行，status/current_step/total_steps维持executing/1/5，plan_revision保另一会话的2。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 文件、操作、产物、验证 / AC |
|---|---|---|
| T-01 / work | [x] 本轮固定源清点完成，最终M1标签身份单列unknown | 本Plan§4固定SHA，优先单列最终M1来源与当前M2差额；git ls-files/Cargo解析/git diff重核成员/源集/父侧并记hash；AC-01 |
| T-02 / work | 进行中；R4分支族/源profile/提交守卫补核，剩余内部组合见§7 | core/syntax/query/lowering/av-core/av/native逐组追生产者→消费者，核F01–09/13及F21–24；可证反例与待探针分账；AC-01/02 |
| T-03 / work | 进行中；R4 CAS/runtime/父侧补核，完整迁移/义务表仍按COV-C/D/E | artifacts/CTE/ABI/A2X/runtime/worker/语言父侧，核F10–12/14–20及F25–27和状态/迁移守恒，覆盖全部成员，补切片依赖/保留表；AC-01..03 |
| T-04 / review | T-02/03完成 | 独立按真实代码/反例复核覆盖、拒绝项、未核项和漂移，按auto-plan-review写§9及债务，核SD-00；AC-04 |
| T-05 / merge | T-04通过 | 分析折入本Plan、路线导航、归档archive；有树/材料先路径对账与guard清理，归档不等于批准重构代码；AC-05 |

下一入口为T-02/03的内部闭包：从B13各构造的全部入口/拒绝及HIR→MIR、B03/05/06资源/权限开始，再补A05/C09/C13/D05/D07/D09/D10具名剩余，最后以E07原case/check/recipe逐条迁移表核守恒；不重复已核的57行来冒进度。复用F01–27及MC证据，开读前固定对应Git对象、核7833之后漂移；最终M1身份有正式来源后另做差额核验，不拿其他里程碑标签代替。用户本轮“暂不修改实现”保持：真实Rust/PE反例和修复未启动，后续需要执行探针须按专属树及材料规则安排，不能在master顺手加测试。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-01/T-02/T-03分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

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

## 10. 待澄清事项

无阻碍只读调查的用户决策。当前未决技术事实：

- 最终M1/tag精确身份：本地名字检索不能替代正式收据；T-01后续来源对账。
- F01真实支持源码的fresh/warm、诊断/PE实际表现；F02正式测量是否触发同类嵌套或unwind后继续；F06真实trap/持久化失败后重试。机制缺口已有证据，动态复现未做，不再笼统称所有内容“尚未核查”。
- 864仓外消费者、会话拓扑删除是否承诺；F03结构化错误的相容方案；F10旧wire EOF/编码/IO行为是否需升级。
- typed HIR builder成本收益、crate边界真实构建成本、runtime共享抽取的符号/取消/预算相容性，以及全量测试迁移总表。

- 新增F16内部计数预算/分类、F17活租约与索引回收、F18真实两读错身份的父侧结果、F19one-shot请求/source合法上限、F20聚合用途与职责包选择，均须保留现有相容承诺再定方案。
- MC01–35逐语义/失败路径、MC36 recipe链和case/check迁移闭包按COV-A..E补齐；每行已有具体未审项，不再只以旧15项驱动。
- F21真实decoder拒绝/FirstSets结果、F22负依赖翻转提交、F23超限/tag/尾字节、F24批量晚失败/同批重试、F25provider/anchor失败保旧、F26sink失败事实、F27deadline/输出/进程树终态：静态证据已落，真实执行全部未做。
- D09 owned_free在OS/provider拒绝后失去ledger项的合同与收敛策略、D06目录计量reparse变更的Windows边界，以及§7列出的内部子分支/原义务闭包，仍有明确归属；不得拿正常释放或源码导航统计代替。

以上归T-02/03与后续有界实施合同；未实测部分维持unknown/candidate。不放宽945–948门，不修改其他计划状态，不把本轮静态审计当代码修复授权。
