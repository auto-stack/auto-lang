---
plan_id: PLAN-896
status: drafting
feature_name: 基于Cargo包与Rust模块的固定测试分组及构建减量
plan_revision: 4
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-31
current_step: 0
total_steps: 13
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/test-purpose-and-admission.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/auto-test-case/project.md, docs/specs/aavm/project.md, docs/specs/repository-boundaries/build-cost-baseline.md]
new_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
touched_goals: []
affects: [test-infrastructure, experimental-workspaces]
depends_on: [PLAN-888, PLAN-895]
---

# [PLAN-896] 基于Cargo包与Rust模块的固定测试分组及构建减量

## 0. 变更摘要

**r4替换r1..r3的tm演进方案，并吸收897全部未实施需求，896为唯一执行合同。**放弃运行时“义务→叶→顶级模块→自动消费者图”的选测体系。测试分组直接对应Cargo package/target和实际Rust模块命名空间，以一份小型静态配置写死；正常用例继续由Cargo/nextest发现和并行运行，不预建全仓逐用例目录。已有TestCase语料、oracle及结果校验保留。

新流程：计划根据修改选择固定组 → 直接构建所需Cargo目标 → 同配置合并为一次nextest运行 → 记录真实结果。全仓覆盖对账用于本次迁移及明确周期治理，不是每次运行前置。当前只起草，待确认r4；没有建实施树、改测试或清缓存。

## 1. 目标

- G1：按Cargo包＋Rust模块固定分组，先落lexer/parser/infer/ownership/vm/a2r/a2c/a2py/a2ts及实际必要职责；不沿用47/48个旧顶级或1207细叶作为约束，不为数量/60秒继续机械拆分。
- G2：恢复简单入口，拟`cargo t parser`、`cargo t parser vm`；兼容配置一次并行调度，必要HTTP/性能/内存资源约束保持。分组只影响选测，不承诺同crate按模块独立编译。
- G3：停止自动测试依赖图扩张；计划明确修改组及少量必要集成/消费组，review核影响范围。真实共享接口修改不能仅跑parser就认证所有下游，也不因Cargo依赖关系把所有消费者单测都拉起。
- G4：无关ACE不发现/编译/补建worker；必须构建的experimental普通dev/test使用debug=0、同树兼容target复用并安全回收旧缓存，保显式完整调试及正式性能合同。
- G5：AAVM/AA2R实验测试全部默认ignore，保代码/测试/夹具/oracle与显式手动入口，正常VM/Rust实现A2R及真实产品消费者不误关。
- G6：迁移不丢原测试，沿用Rust/nextest与TestCase原判定；除具名批准的实验ignore外不因用途/耗时停用。旧tm执行路径退出，仓规/脚本/周期/在途入口有明确承接。

非目标：物理拆新产品crate、继续拆VM内部层次、优化产品算法、重新建设分片目录/全局指纹/动态影响图/新裁判、跨树共享可写target、关闭全部probe、擅清890/894活动缓存、改金样/原压力/样本/正式门。固定分组首层落地后，特别大的VM组另据实测讨论。

## 2. 架构方案

### 2.1 固定分组，不再从叶生成

技术名称：Cargo package/crate是第一层；其内`mod parser`、`mod vm`等Rust模块是第二层，二者不是新的Cargo package。配置拟为`scripts/test-groups.toml`（新增），每组只列workspace/manifest、package、明确lib/test/bin目标、features/profile、Rust测试命名空间或现成suite/治理文件及必要工具。不存逐用例ID/叶分片/每源文件hash/自动消费者图；不依据计量结果自动生成分组。

| 拟组 | 已核实真实实现/测试位置 | 归组原则 |
|---|---|---|
| lexer | `crates/auto-lex/src/lexer.rs::tests`；core lexer.rs已是再导出 | 优先整个auto-lex包；宿主词法集成单列已知命名空间 |
| parser | `crates/auto-lang-core/src/parser.rs`及parser/{tests,decls,expressions,statements,type_syntax,scenario} | 包限定后的`parser::`完整子树，不仅名为tests的单个子mod；宿主解析集成显式补充 |
| infer / ownership | core的`infer/`、`ownership/`及宿主对应tests | 对应源码模块；不把所有消费者当作本组成员 |
| vm | core的`vm/`及auto-lang/src/tests/vm_*、相关行为测试 | 先一个固定职责组，可由多个明确命名空间组成；不按20例切叶 |
| a2r / a2c / a2py / a2ts | core `trans::{rust,c,python,typescript}`；宿主`tests::a2r_tests`等 | 后端命名空间＋原语料包装完整调用；Python/TS共文件的测试需准确分类，不能用a2py字符串猜 |
| 其他crate/ACE工程 | 各Cargo.toml及真实lib/tests/worker入口 | 小crate整体一组；ACE按当前包/target，非相关任务不选 |

此表是已核路径下的首层设计，完整映射由T-01/02列出。散落的`planNNN`测试按真实能力收进某组；确有必要时仅移动测试mod归属/新增统一`mod tests`挂载，不搬产品实现。跨能力集成明确单列，命名不明确的测试不能悄悄掉出运行面。重叠选择交nextest集合并集去重，同名不同binary不误去重。

### 2.2 简单入口与范围选择

拟`cargo t <组> [组...]`由既有standalone launcher或薄脚本展开静态Cargo参数和nextest过滤式，不再进入module_catalog/refresh。`cargo t --list`只读静态组名/职责/配置，不构建。`cargo t`无组时保持原日常t义务的固定配置并行运行（仅AAVM/ACE隔离等已批准变化另列），不会静默跑全部新组；是否采用更大固定默认包不在本轮自动决定。真实零选、未知组、错误过滤/工具准备和非零退出必须失败。

计划在开工/冻结前记录修改了哪些固定组、为何补哪些集成/消费组、实际命令和适用条件；不运行自动消费者图或`tm --changes`。例如parser私有算法变更以parser完整组及其已知集成验证为主；AST/语法/共享IR接口改动按真实影响明确补vm/a2r等，不建立无限递归依赖推导。未知触面先review并解释，必要时选原固定日常包，不能静默运行全部experimental。

### 2.3 原生发现与并行

同manifest/package/features/profile的多个组以OR过滤并集一次nextest调用，不先逐叶list/run。Rust/nextest对选中目标的正常列测试仍允许，用于实际完整集合、零选及结果校验；禁止为了先“知道总共多少测试”而枚举全部工程。新增同命名空间测试自然被纳入，宏/cfg/build.rs仍由Cargo正确重新编译，不维护另一个测试新鲜度系统。

子模块测试过滤只限制运行；同crate测试二进制仍可能整体编译，源码改变仍由Cargo指纹决定增量成本。本轮不承诺parser独立二进制。HTTP串行、nextest资源组/heavy_gate/原截止、Windows有界Job和输出限制继续生效。结果来自真实测试终态；包装内部原check继续核原trace/比较器，不能用父exit0替代子项。

### 2.4 无关ACE不构建与存储

将目前挂在通用test-cases的AV/MN/CTE/ACE宿主接线，用默认关闭feature或独立测试target隔离；普通VM/legacy TestCase仍可使用test-cases。静态ACE组显式准备其工具，非ACE组若请求ACE动态worker须在构建前失败并解释；真实修改影响ACE时由计划明确选ACE组，不按计划标题永久豁免。

继承897：必要experimental普通dev/test明确debug=0，不改opt-level/debug-assertions/overflow-checks/panic；完整调试debug=2保显式入口，正式release/benchmark不改。同树兼容依赖target复用，保regular/batch命名空间、triple/profile/features/锁与worker/lib来源绑定；不跨树共享可写缓存、不硬链活动产物、不建junction/symlink。硬编码shim消费者一并承接，不能只改构建脚本。

清理先预览，核owner、实际解析路径、活动/测量预留、证据/源码及reparse；仅自有闲置可重建缓存，保必要原始证据并遵守14天/hold/预算，不把target搬入plan-artifacts。旧缓存逐工程迁移，避免新旧同时扩大；890/894和其他会话的缓存未经其owner交接不动。profile/target身份变更不得复用旧绿冒当前证据。

### 2.5 tm退出、原义务与AAVM

888/895逐用例清单与分区只作**迁移基线/历史审计输入**，不再是新runner消费依赖。T-04用旧基面真实枚举与新固定组正常枚举对账：原测试、包装输入/check、oracle、profile、平台/ignored/手动条件逐项说明去向；集合并集守恒，不强求职责组完全不相交。未注册测试也列归属/现行入口，新增注册须具名说明，不借移除tm停掉现行必需门。

旧tm不再保平行执行器；替换所有本仓活跃调用与tf/taa/ta/t3周期路径。迁移后旧tm命令给出明确迁移提示并非零退出，不默默转全量；旧细叶ID不再是公共执行接口。适用的截至11月30日旧入口兼容承诺，在新组/固定旧选面中保真实测试义务，放弃旧叶导航本身是本r4拟变更契约。外仓UI/Man保owner入口，不复制其框架；在途890/894/893须登记入口影响与交接，不擅改其计划验收。

AAVM/AA2R按真实实验源码/注册逐IDignore，保手动`--ignored`与原feature/平台/T3约束；不是忽略全部parity或全部A2R。历史lang/vm-aavm含23叶/138义务、120已ignored且混入两条plan492非实验测试，禁止顶级黑名单。冻结合同仅记录此次批准的适用性差集，不删测试。原本真实消费packages/aavm/lib-legacy等代码的产品验证仍保留。其他一次性调研/专项逐ID审用途，未经明确批准不改变启用状态。

## 3. 技术栈

Cargo原生package/target/features/profile，Rust模块测试命名空间，nextest过滤/并行/资源组，既有TestCase oracle/trace、Windows Job及standalone launcher。仅一份固定分组配置和薄分发，不新建运行器/全仓枚举缓存服务。

## 4. 需求分析与背景调查

### 授权及现状

2026-10-31用户明确提出合并896/897、放弃tm叶到大模块组织，按crate及lexer/parser/vm/a2r/a2c/a2py等固定Rust模块选测；先解决首层，VM进一步细拆以后讨论。之前已批准AAVM/AA2R保代码且测试默认ignore。用户当前要求讨论新方案，故本轮整合为可确认草案，未授权实施为已开始。无新增预算/自动续行期限。

原896 r1..r3见Git历史/本节交接，原897 r3完整合同在897文档历史区保留；897不独立建树/执行。当前Specs仍描述888/895旧现状，不能把本草案写成已交付。设计见`docs/design/testing/crate-and-rust-module-tests.md`（本轮新增target-state）。

| 来源 | 已观察事实/用途 |
|---|---|
| `docs/specs/overview.md`、auto-test-case的module-selection/test-purpose-and-admission/governance-and-inventory | 当前唯一叶/消费者守恒是旧已交付规则；本r4显式提出替换，不悄悄绕过 |
| `.cargo/config.toml`、`.config/nextest*.toml`、`scripts/full-regression-driver/src/main.rs` | t及旧别名目前转tm；旧日常过滤/资源/超时仍在；可复用launcher但需改路由 |
| `crates/auto-lex/src/lexer.rs`、`crates/auto-lang-core/src/{parser.rs,lexer.rs,trans.rs,lib.rs}`、auto-lang/src/tests.rs | lexer已独立；parser/vm/trans在core；宿主含大量散落集成和ACE接线，不能只过滤parser::tests就声称全覆盖 |
| `module_command.py/module_catalog.py/impact_graph.py/run_selected.py` | 全局身份失效先刷新全recipe；动态消费闭包及逐叶串行是退役对象，不再优化分片缓存 |
| 888观测表、895摘要 | 旧计量为逐叶单轮，895重建预算；不是新模块当前实跑基线，不继承“48均达标”结论 |
| `897-experimental-build-storage-lifecycle.md` r3 | 三树历史experimental唯一125.06GiB/PDB72.92GiB、源码约10MB/树；原始清单路径/hash见897，未重测当前容量，不用库存减空目录声称收益 |
| `docs/specs/repository-boundaries/build-cost-baseline.md`、toolchains.json/prepare_tools.py | Cargo构建边界、已有同树regular/batch工具复用和shim路径承接，不能忽略存量 |

### 897吸收对账（不隐含丢弃要求）

| 897原目标/任务 | 本计划归属与变化 |
|---|---|
| G6/T-11 ACE域隔离 | G4、T-12、V-05；直接静态组+宿主feature/target隔离，不建域图 |
| G2/T-09先选后发现 | G1/2、T-02/05；放弃分片新鲜度方案，直接Cargo目标/命名空间 |
| G0/G3/T-10用途与雪藏 | T-01/11/12；活引擎保留，其他研究逐ID建议，不批量停用 |
| G1/T-02 profile与G4/T-03 target/shim | T-12、AC-10；普通debug=0/完整debug=2、同树复用保持 |
| T-04安全清理 | T-13、AC-11；不触他方活动现场，收据与证据保护保持 |
| G5/T-06稳态容量及50%目标 | T-07/AC-06真实同工作量测量；50%是旧草案未批准的拟门，本r4不把它变成已授权硬门；报告实际比例/未达原因，用户确认本r4即以此口径替换 |
| T-05/07/08验证/复审/合入 | T-06/09/10；最终门改固定组，不再tm并集 |

## 5. 详细设计与实施入口

拟唯一树`D:/autostack/.wt/lang-896/auto-lang`、分支`plan-896-dev`，必要兄弟由分组脚本准备；897不再创建另一树。主检出只计划/设计簿记。T-01核真实HEAD及890/894/893写窗，T-02用parser+lexer最小真实链先验证固定范围/零选拒绝/无全域构建，再迁移完整分组。review在同权威树独立上下文，期间实施停写，无自动新chat/代理。仍需保留的准备/Job/trace机制复用，不能把退役tm理解为移除隔离/oracle。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/module-selection.md | 动态叶/顶级/自动闭包→固定Cargo包+Rust mod；计划具名集成组、原生发现与并行 | 移除复杂运行前置 | AC-01/02/04/05 |
| SD-02 | modify | docs/specs/auto-test-case/test-purpose-and-admission.md | 48建议/候选不再成为新分组结构；实验ignore已批准，其他用途不自动筛选 | 保用途决定与执行分离 | AC-09/04 |
| SD-03 | modify | docs/specs/auto-test-case/governance-and-inventory.md、project.md | tm目录强制消费→迁移对账及明确周期审计；既有TestCase/trace继续核原比较 | 不以简化丢测试/冒绿 | AC-03/05 |
| SD-04 | modify | docs/specs/aavm/project.md | 触面自动自举门→保源码/手动入口、全部实验默认ignore | 用户裁定 | AC-09 |
| SD-05 | add | docs/specs/auto-test-case/build-storage-lifecycle.md | 规定必要构建、普通/完整调试、同树缓存/来源/安全清理 | 吸收897存储目标 | AC-10/11 |
| SD-06 | modify | docs/specs/repository-boundaries/build-cost-baseline.md | 补固定组实际Cargo编译边界/时空成本，不宣称mod过滤使crate独立编译 | 防错误收益口径 | AC-06/10 |

## 6. 测试设计

均为拟执行，未实跑。work持真实实现/反例，review独立逐AC及SD，merge持冻结最终固定组门。脚本单测沿用现有治理测试并修改真实消费者，不新建一套通用检查器。

| case_id / owner_stage | AC | 真实入口/区分性输入 | 预期oracle与边界 |
|---|---|---|---|
| V-01 / work | AC-01/02 | 新cargo t入口parser、parser+vm、未知名/空匹配/相似命名空间 | 精确package+mod子树，含parser的decls等；兼容组同调用、跨组时间区间重叠；未知/零选失败，不靠子串误选 |
| V-02 / work | AC-01/05 | 同namespace新增/删除/改名、宏展开/cfg、同名不同binary；正常nextest枚举 | 由Cargo真实发现自然纳入/移除，迁移全集差集明确；无分片目录或全局源码扫描前置 |
| V-03 / work | AC-03 | 一绿一红/崩溃/超时/缺child/check/错oracle/准备失败 | 原终态/首次红不覆盖，缺项不冒PASS，未完NOT_RUN，原包装oracle拒绝；重放可追 |
| V-04 / work | AC-04/05 | parser私有改动与共享AST改动两份真实影响清单；固定旧入口/周期 | 计划具名组及必要集成依据可review，无自动图扩773叶；原日常/周期义务有去向，专项按触面 |
| V-05 / work | AC-10 | 干净普通VM/legacy TestCase树、list组名、显式ACE组、伪装普通组补建ACE worker | 前三非ACE场景实际ACE构建=0；必要ACE正常准备，未声明补建构建前失败；核真实argv/产物而非只看测试是否运行 |
| V-06 / work | AC-09 | AAVM root/宏/生成/独立parity入口、普通A2R/VM/两条plan492正控、显式ignored小例 | 实验默认忽略不充绿、重生成不复活；原代码/oracle保留；普通组不误关；手工可运行，原T3限制保留 |
| V-07 / work | AC-06 | 固定parser/vm/a2r和适用旧日常选面，同源/配置/负载记录 | 原单次nextest与新入口同选面对照，分别计冷构建/执行/清理/整命令、CPU/内存/磁盘；ignore减少单列，不承诺50秒/P90 |
| V-08 / work | AC-10 | AV+MN+真实shim消费者；普通debug=0、显式debug=2、错profile/hash | 实际配置生效、target复用且原行为/正式门不变；错来源消费失败，regular/batch仍隔离 |
| V-09 / work | AC-11 | 自有闲置缓存预览/执行、活动/预留/证据/源码/未知路径/reparse拒绝与中途失败 | 实际释放唯一字节可核；不穿透/不动他方、不把半删报成功；原始证据保护 |

本次迁移允许一次实际全范围**发现对账**，不等于重跑全部用例；移除旧逐义务目录后仍要证明每项有新去向。执行验证只跑受影响固定完整组及具名集成/真实消费者，冻结门记录package/target/features/profile/过滤/工具身份与真实结果。旧tm/目录生成的负控按退役责任调整，不能删失败断言来隐藏漏测。最终新架构未落地前不得把现行仓规门视为已通过；T-08同步AGENTS/文档/工具入口，技能外仓维护列具体交接，不扩大本仓写范围。

原日志/探针/截图本地保留不入库，摘要记命令/退出码/源与工具身份/首次失败/本地路径/hash；14天及hold/预算规则保持。纯规划不运行Rust/tests/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可核完成条件 |
|---|---|---|
| AC-01 | work | 一份固定配置对应真实package/target/mod，首层完整职责及散落测试有归属；不需tm/list/全仓目录/动态图；VM不再机械细拆；V-01/02 |
| AC-02 | work | cargo t具名组及多组一次兼容nextest并行调度，原资源约束/HTTP串行/截止保持；无参数保固定日常范围，编译限制明确；V-01 |
| AC-03 | work | 原真实结果/包装check/oracle/源码身份与失败/生命周期门继续有效，父exit0不代子项；V-03 |
| AC-04 | work | 每计划能直接写固定完整组命令和必要集成理由，无自动依赖图拉全仓；真实共享变化仍有明确下游验证；不按用途批量删测；V-04 |
| AC-05 | work | 旧实际枚举到新组并集守恒，除已批准ignore每个原身份/条件/oracle可追；tm退出/旧别名/周期/治理消费者承接且零静默退全量；V-02/04 |
| AC-06 | work | 新固定组完整单轮及代表旧日常同选面对照，冷编译/运行/整命令/CPU/内存与稳态唯一存储分列，超时慢例具名；不以忽略/空目录或估算报收益；V-07 |
| AC-07 | review | 冻结SHA独立逐AC/SD/V、遗漏/延后/workaround及健康审核通过；债务具名不掩盖合同缺口 |
| AC-08 | merge | 最终审查后固定完整组冻结门、真实合入/规范沉淀、896及897按合并归档方式收尾、留存预算、fresh guard clean清树；周期仅到期触发 |
| AC-09 | work | AAVM/AA2R精确全范围默认ignore、代码/手动重放保留，普通A2R/VM/消费者不误关；AGENTS/Specs一致；V-06 |
| AC-10 | work | 无关ACE零构建；必要ACE宿主/动态工具显式隔离；debug=0/完整调试、同树复用、shim来源与正式参数保持；V-05/08 |
| AC-11 | work | 旧缓存迁移与实际回收有owner/预览/执行/失败收据，活动/测量/证据/源码/他方保护；不把target转存证据，V-09 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 具体路径/交付及验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户确认r4 | 唯一896树核HEAD/890/894/893接口；冻结旧真实选面和897空间/用途/active owner基线；盘点所有源模块/散落测试及当前入口，不沿用tm计数作新架构 | AC-01/04/05/10/11 |
| [ ] | T-02 | work | T-01 | 拟新增scripts/test-groups.toml；先lexer/parser最小组与现有launcher薄分发真链；V-01/02；固定完整职责映射，明确namespace/target/features，非零选面校验 | AC-01/02 |
| [ ] | T-03 | work | T-02 | 在固定组文档/配置中写明集成组及选择指导；两种parser改动范围实例，取消自动消费者图；散落历史测试归组必要时只调整test mod挂载 | AC-01/04 |
| [ ] | T-04 | work | T-02/03 | 对比旧真实枚举与新组正常枚举，并集/条件/oracle/包装绑定逐项去向；差集和授权例外摘要，机器清单本地；V-02/04 | AC-05 |
| [ ] | T-05 | work | T-02 | `.cargo/config.toml`、launcher与薄脚本完成cargo t多组兼容合并/并行；原Job/结果/trace机制复用不经module_catalog，V-01/03；无第二runner | AC-02/03 |
| [ ] | T-11 | work | T-01 | aavm*.rs、宏/生成和parity实验逐ID默认ignore，保代码/手动入口、plan492及普通A2R；V-06及授权差集 | AC-09 |
| [ ] | T-12 | work | T-02及T-01写窗 | tests.rs/必要feature或target、prepare_tools/toolchains/真实shim接线；ACE隔离、debug=0/2、同树兼容target与来源，V-05/08 | AC-10 |
| [ ] | T-13 | work | T-01/12 | 复用现有工具或最小build_storage.py新入口：owner预览/受保护拒绝/闲置旧缓存回收，不清890/894；V-09，实际唯一字节收据 | AC-11 |
| [ ] | T-06 | work | T-03/04/05/11/12 | `run-full-regression.py`及全部本仓活跃tm消费者/治理脚本迁新组，tm明确退役提示，旧义务/周期承接，V-02..06；跨仓仅owner交接不擅改 | AC-01..05/09/10 |
| [ ] | T-07 | work | T-06/13 | 代表固定组/旧日常同选面实跑及空间/成本对照V-07，吸收897库存/减量结论；不重跑全目录/收益矩阵，实际背景与慢例分列 | AC-06/10/11 |
| [ ] | T-08 | work | T-07 | SD-01..06、AGENTS与本仓入口文档同步，原叶规则明确替换；列需维护技能/在途计划的准确接口交接；健康及逐AC证据，冻结execution_done | AC-01..06/09..11 |
| [ ] | T-09 | review | T-08 | 同树独立上下文auto-plan-review逐AC/V/SD及897吸收对账、遗漏/健康/债务；绑定代码SHA，有finding回work | AC-07 |
| [ ] | T-10 | merge | T-09 pass | auto-plan-merge按新固定组冻结门、真实合入/规范沉淀；896归档，897记录由896吸收交付而非双份完成；留存/周期及guard清唯一树/分支/空组 | AC-08 |

## 9. 复审记录

r1..r3为本文件Git历史中的旧方案：先合并调用，后补AAVM ignore与897接口。无实施进度，不把旧方案pass自检视作新方案验收。旧任务/AC语义被本r4整体替换，897历史任务映射见§4；无已完成项被擦除。

2026-10-31 / stage:new / plan_revision:r4 / outcome:pass（规划交接自检，非实施） / authorization:用户提出合并896/897并放弃tm，改固定crate+Rust mod首层；AAVM ignore仍继承 / grounding:实际auto-lex/core parser/vm/trans、宿主tests.rs、Cargo/nextest旧入口及897 r3 / changed:整体架构/任务/AC/SD替换，固定组代动态叶及图；吸收profile/target/清理/用途全部目标，不承诺同crate编译隔离或50秒 / preflight:13任务/11AC/9V/6SD，897不独立执行；未编译/测试/清缓存/建实施树 / next:确认本r4后T-01。

## 10. 待澄清事项

本r4为待确认的具体方案；不另要求选择一套新复杂调度系统。T-01/02核完整首层组及实际namespace，不能凭名字漏测试；更细VM拆分明确在本轮之外。897旧稿50%空间硬门未获批准，本r4改真实同工作量比例报告，确认本r4即采用新口径。固定日常无参数包保旧范围，扩大默认范围/停用其他研究仍需明确逐项决定。执行中的等价实现不重复索要批准；若必须改变以上义务/边界则先呈具体差异。
