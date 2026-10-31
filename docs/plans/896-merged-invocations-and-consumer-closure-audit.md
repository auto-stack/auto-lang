---
plan_id: PLAN-896
status: execution_done
feature_name: 基于Cargo包与Rust模块的固定测试分组及构建减量
plan_revision: 7
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-31
current_step: 15
total_steps: 17
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/test-purpose-and-admission.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/auto-test-case/project.md, docs/specs/aavm/project.md, docs/specs/repository-boundaries/build-cost-baseline.md, docs/specs/auto-av/project.md, docs/specs/auto-cte/project.md, docs/specs/tool-execution/project.md, docs/specs/auto-cache/project.md, docs/specs/auto-ac/mir-native-scalar.md, docs/specs/auto-ac/data-resource-contract.md]
new_spec_components: [docs/specs/auto-test-case/build-storage-lifecycle.md]
touched_goals: []
affects: [test-infrastructure, experimental-workspaces]
depends_on: [PLAN-888, PLAN-895]
---

# [PLAN-896] 基于Cargo包与Rust模块的固定测试分组及构建减量

## 0. 变更摘要

**r6继承已授权r4/r5，并将ACE统一为独立workspace、保留多个职责明确crate，与测试整编和probe历史命名清理一起交付；r4已替换r1..r3的tm演进方案并吸收897全部未实施需求，896为唯一执行合同。**放弃运行时“义务→叶→顶级模块→自动消费者图”的选测体系。测试分组直接对应Cargo package/target和实际Rust模块命名空间，以一份小型静态配置写死；正常用例继续由Cargo/nextest发现和并行运行，不预建全仓逐用例目录。已有TestCase语料、oracle及结果校验保留。

新流程：计划根据修改选择固定组 → 直接构建所需Cargo目标 → 同配置合并为一次nextest运行 → 记录真实结果。全仓覆盖对账用于本次迁移及明确周期治理，不是每次运行前置。r4已获用户授权实施→独立复审→合入→清理；本轮开始实施。

## 1. 目标

- G1：按Cargo包＋Rust模块固定分组，先落lexer/parser/infer/ownership/vm/a2r/a2c/a2py/a2ts及实际必要职责；不沿用47/48个旧顶级或1207细叶作为约束，不为数量/60秒继续机械拆分。
- G2：恢复简单入口，拟`cargo t parser`、`cargo t parser vm`；兼容配置一次并行调度，必要HTTP/性能/内存资源约束保持。分组只影响选测，不承诺同crate按模块独立编译。
- G3：停止自动测试依赖图扩张；计划明确修改组及少量必要集成/消费组，review核影响范围。真实共享接口修改不能仅跑parser就认证所有下游，也不因Cargo依赖关系把所有消费者单测都拉起。
- G4：无关ACE不发现/编译/补建worker；必须构建的experimental普通dev/test使用debug=0、同树兼容target复用并安全回收旧缓存，保显式完整调试及正式性能合同。
- G5：AAVM/AA2R实验测试全部默认ignore，保代码/测试/夹具/oracle与显式手动入口，正常VM/Rust实现A2R及真实产品消费者不误关。
- G6：保留长期回归能力及原判定；r5允许有依据合并重复测试、删除已完成使命的临时测试，逐项记录旧测试去向与理由，不以减少用例数冒充性能优化。旧tm执行路径退出，仓规/脚本/周期/在途入口有明确承接。
- G8：ACE统一迁入`experimental/ace/`独立workspace，统一Cargo.lock和同树target；内部保留职责明确的库与验证工具包，普通cargo t不得隐式解析、编译或准备ACE测试/worker。
- G7：ACE测试按真实行为重组为少量Cargo测试入口及语义模块，去掉测试文件/模块/函数名中的计划号；库、验证工具、实验原型分别按真实职责命名。

非目标：把ACE合成单一大crate、无职责依据拆新产品crate、继续拆VM内部层次、优化产品算法、重新建设分片目录/全局指纹/动态影响图/新裁判、跨树共享可写target、关闭全部probe、擅清890/894活动缓存、改金样/原压力/样本/正式门。固定分组首层落地后，特别大的VM组另据实测讨论。

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

拟`cargo t <组> [组...]`由既有standalone launcher或薄脚本展开静态Cargo参数和nextest过滤式，不再进入module_catalog/refresh。`cargo t --list`只读静态组名/职责/配置，不构建。新增显式`cargo t all`运行语言仓全部固定组（不改变ignore/平台/里程碑条件，不等于外仓UI/Man批次）。`cargo t`无组时保持原日常t义务的固定配置并行运行（仅AAVM/ACE隔离等已批准变化另列），不会静默跑全部新组；是否采用更大固定默认包不在本轮自动决定。真实零选、未知组、错误过滤/工具准备和非零退出必须失败。

计划在开工/冻结前记录修改了哪些固定组、为何补哪些集成/消费组、实际命令和适用条件；不运行自动消费者图或`tm --changes`。例如parser私有算法变更以parser完整组及其已知集成验证为主；AST/语法/共享IR接口改动按真实影响明确补vm/a2r等，不建立无限递归依赖推导。未知触面先review并解释，必要时选原固定日常包，不能静默运行全部experimental。

### 2.3 原生发现与并行

同manifest/package/features/profile的多个组以OR过滤并集一次nextest调用，不先逐叶list/run。Rust/nextest对选中目标的正常列测试仍允许，用于实际完整集合、零选及结果校验；禁止为了先“知道总共多少测试”而枚举全部工程。新增同命名空间测试自然被纳入，宏/cfg/build.rs仍由Cargo正确重新编译，不维护另一个测试新鲜度系统。

子模块测试过滤只限制运行；同crate测试二进制仍可能整体编译，源码改变仍由Cargo指纹决定增量成本。本轮不承诺parser独立二进制。HTTP串行、nextest资源组/heavy_gate/原截止、Windows有界Job和输出限制继续生效。结果来自真实测试终态；包装内部原check继续核原trace/比较器，不能用父exit0替代子项。

### 2.4 无关ACE不构建与存储

将目前挂在通用test-cases的AV/MN/CTE/ACE宿主验证迁到独立ACE workspace的conformance包；必要的根侧桥接若必须保留，用默认关闭feature或独立测试target隔离，不在普通日常配置启用；普通VM/legacy TestCase仍可使用test-cases。静态ACE组显式准备其工具，非ACE组若请求ACE动态worker须在构建前失败并解释；真实修改影响ACE时由计划明确选ACE组，不按计划标题永久豁免。

继承897：必要experimental普通dev/test明确debug=0，不改opt-level/debug-assertions/overflow-checks/panic；完整调试debug=2保显式入口，正式release/benchmark不改。同树兼容依赖target复用，保regular/batch命名空间、triple/profile/features/锁与worker/lib来源绑定；不跨树共享可写缓存、不硬链活动产物、不建junction/symlink。硬编码shim消费者一并承接，不能只改构建脚本。

清理先预览，核owner、实际解析路径、活动/测量预留、证据/源码及reparse；仅自有闲置可重建缓存，保必要原始证据并遵守14天/hold/预算，不把target搬入plan-artifacts。旧缓存逐工程迁移，避免新旧同时扩大；890/894和其他会话的缓存未经其owner交接不动。profile/target身份变更不得复用旧绿冒当前证据。

### 2.5 tm退出、原义务与AAVM

888/895逐用例清单与分区只作**迁移基线/历史审计输入**，不再是新runner消费依赖。T-04用旧基面真实枚举与新固定组正常枚举对账：原测试、包装输入/check、oracle、profile、平台/ignored/手动条件逐项说明去向；r5有据合并/删除单列后验证有效覆盖守恒，不强求职责组完全不相交。未注册测试也列归属/现行入口，新增注册须具名说明，不借移除tm停掉现行必需门。

旧tm不再保平行执行器；替换所有本仓活跃调用与tf/taa/ta/t3周期路径。迁移后旧tm命令给出明确迁移提示并非零退出，不默默转全量；旧细叶ID不再是公共执行接口。适用的截至11月30日旧入口兼容承诺，在新组/固定旧选面中保真实测试义务，放弃旧叶导航本身是本r4拟变更契约。外仓UI/Man保owner入口，不复制其框架；在途890/894/893须登记入口影响与交接，不擅改其计划验收。

AAVM/AA2R按真实实验源码/注册逐IDignore，保手动`--ignored`与原feature/平台/T3约束；不是忽略全部parity或全部A2R。历史lang/vm-aavm含23叶/138义务、120已ignored且混入两个plan492命名空间的7条非实验测试，禁止顶级黑名单。冻结合同仅记录此次批准的适用性差集，不删测试。原本真实消费packages/aavm/lib-legacy等代码的产品验证仍保留。其他一次性调研/专项逐ID审用途，未经明确批准不改变启用状态。

### 2.6 ACE测试整编与库命名（r5用户追加）

范围为experimental内ACE相关syntax、semantics、artifacts、core、MIR lowering、AV、native、CTE、ABI、case-worker、A2X及根crate的ACE宿主测试；不包含AutoGraphics等无关工程。T-14先核当前实际测试/调用关系及890/891/894/893写窗，按包分批实施，不改他方WIP。

- **入口合并**：各包按运行条件保留少量集成入口，例如`tests/conformance.rs`、`tests/resources.rs`，内部用`tests/conformance/records.rs`等子模块挂载；私有实现单元测试放回对应模块的`#[cfg(test)] mod tests`。不为每项能力或每个计划生成独立exe。不能为减少exe而让普通ACE小例必须编译全部跨后端/压力依赖；进程边界、features、平台或资源要求不同的测试可独立入口，逐包说明理由。不承诺全ACE合为一个exe或固定目标数。
- **语义命名**：用`records::nested_field_write_returns_value`、`cache::restore_revalidates_dependencies`等行为名代替`av_data_820_*`。历史Plan引用放注释/迁移摘要。核nextest过滤、资源组、脚本、env/CARGO_BIN_EXE、include路径、fixture定位及固定组配置，改名后仍选择原完整职责。版本化wire/profile/schema、原fixture字节、金样和对外协议标识不随清除计划号而改写。
- **去重与删除**：先按“保护的不变量＋输入/边界＋oracle＋执行路径/条件”比对。只合并等价覆盖，共用fixture/参数表时保留可诊断的场景名及断言；不同后端、fresh/restart、非法输入、生命周期/故障和性能边界不因看起来相似而删除。一次性方案比较、临时调试输出、已退役路径且无当前不变量的测试可以删除，记录理由；已发现过真实缺陷的反例优先改造成稳定回归。目的不明者先保留并注明缺什么事实，不能按文件含probe/计划号/耗时直接删除。有效压力/性能例属于正规专项，保原规模和阈值，不放普通日常池。
- **名字与边界**：Cargo.toml带`[package]`的目录是包，可提供lib crate和bin/test targets；仅`[workspace]`的根目录是组织多个包的工作区，不是Rust mod。`ac-av-probe`根当前是core/probe workspace，`ac-cte-probe`是contract/core/probe workspace；`ac-artifact-probe`、`auto-abi-probe`是带lib的独立包。对已被实际消费的库，拟按职责考虑`ac-av`/`auto-av`、`ac-artifacts`、`ac-cte`/`auto-cte`等名称；验证专用工具采用`*-conformance`/`*-test-support`等名称，真正未定型研究才保留probe。最终名字由T-14的包/API/消费者盘点确定并在T-16统一实施，不因存在lib.rs就升级为生产稳定接口、不更改既定依赖方向。
- **验收口径**：逐包记录前后顶层测试target数、语义case去向及实际构建exe/PDB字节；测试合并减少重复链接，慢例分层解决运行耗时，两者分别报告。普通日常整命令在明确参考机器/常规已构建环境下以≤60秒为目标，准备/编译/执行总时间分列；首次冷构建另列，超标说明具体慢例/准备成本，不能以单组≤60秒替代整命令目标或通过删有效测试凑数。是否达标据真实测量报告。

### 2.7 单一独立ACE workspace（r6用户明确决定）

**公开入口（r7用户明确）**：`cargo t ace` 显式运行ACE常规自动测试集合，直接定位独立ACE workspace的固定包/测试目标，不通过全仓发现，也不默认叠加普通语言日常集合；包内可并行的测试沿nextest执行，保资源约束。ACE压力/性能/手动/平台及里程碑条件继续按各自专项规则，不因该入口无条件全跑；实际未执行条件明确报告。普通`cargo t`仍不编译ACE，`cargo t --list`静态列出ace组且不构建。

目标根为`experimental/ace/Cargo.toml`（虚拟workspace），内部`crates/<职责名>/Cargo.toml`保留syntax、semantics、artifacts、前端core、共享MIR/lowering、AV、native、CTE、ABI、case-worker、A2X等实际独立职责与原依赖方向，跨库验收拟`tests/conformance/`独立包。此为目录/构建组织调整，不把不同IR/后端实现合为一个crate，也不因清理名字把尚未交付能力标作正式产品。最终包清单/新旧目录名/依赖映射由T-14先核实，不机械照搬示意名称。

- **workspace迁移**：由T-17建立唯一ACE workspace，移除被吸收包的嵌套`[workspace]`，调整继承字段、`workspace.dependencies`、manifest相对路径及根语言workspace排除配置；合并为一个跟踪的Cargo.lock。冻结原依赖版本/来源，锁文件冲突逐项解释并验证，不顺手全量升级。真正为验证Cargo/外部消费者而必须独立的fixture工程可保独立并具名说明，不能把常用实现继续散落成历史probe workspace。
- **构建与选择**：ACE内部同树共用自己的target，普通dev/test debug=0，完整调试显式选择；原各workspace不同release/benchmark有效配置逐项承接，不因统一根profile悄悄改变正式性能条件。统一workspace不等于全部成员都构建：固定组明确manifest/package/target/features，ACE根default-members明确为有限常用成员，压力/跨后端工具不默认加入。日常脚本不得使用`--workspace --tests`替代包选择；显式全ACE验收才允许构建完整面。
- **普通cargo t边界**：语言workspace与ACE workspace保持分开；无参数普通cargo t、普通具名组及`--list`不先发现ACE、不构建ACE库/测试/bin、不补建ACE worker。库依赖、build.rs、dev-dependency、测试包装及动态工具准备同时核查；只把测试标ignore不能通过此门。显式ACE组才进入新workspace。现有显式`cargo t all`仍为全部固定组并集，若包含ACE应清楚列出；它不改变普通cargo t默认范围。
- **整体迁移顺序**：T-14职责/测试/消费者映射 → T-17统一workspace并与T-16一次搬迁改名 → T-15在新路径整编测试（可按已稳定包分批） → T-12工具/profile/宿主隔离及T-13闲置旧缓存回收。T-15/T-16原包级局部先后约束由此替代，避免先搬一遍改名、再搬一遍workspace。必须核`CARGO_MANIFEST_DIR`、include/fixture定位、worker查找、shim lib硬编码路径、脚本/固定组及外部真实消费者；原正式协议/fixture内容保持。
- **产物与收尾**：仅一个ACE workspace/锁文件不保证所有feature/profile都共用同一产物；依旧按实际配置核复用与字节数。不跨worktree共享可写target。迁移验证通过后只清理自有、闲置旧target，活动计划在写窗交接前不移动/清理其源码和缓存，保恢复与证据。新旧target并存的迁移峰值和稳态空间分列。

## 3. 技术栈

Cargo原生package/target/features/profile，Rust模块测试命名空间，nextest过滤/并行/资源组，既有TestCase oracle/trace、Windows Job及standalone launcher。仅一份固定分组配置和薄分发，不新建运行器/全仓枚举缓存服务。

## 4. 需求分析与背景调查

### 授权及现状

2026-10-31 r6追加授权：用户明确决定ACE统一为独立workspace、内部保留多个职责清晰crate，并与测试整编/probe改名一起更新896；普通cargo t不得隐式编译ACE。本轮按要求更新计划合同，未执行目录搬迁、锁文件合并或构建。原全链路授权保持。

2026-10-31 r5追加授权：用户明确要求重组全部ACE测试为少量入口，按真实意图去掉测试计划号，合并相似覆盖、删除临时测试，仅保留可守护未来修改的正规用例，并核查probe目录是否应改为库名。此前“不丢任何原测试”收窄为“不丢有效验证能力”；有据删除临时/等价重复测试属于本次范围。r4已有实施、独立复审、合入与清理授权保持，不新建897。

2026-10-31用户明确提出合并896/897、放弃tm叶到大模块组织，按crate及lexer/parser/vm/a2r/a2c/a2py等固定Rust模块选测；先解决首层，VM进一步细拆以后讨论。之前已批准AAVM/AA2R保代码且测试默认ignore。用户当前要求讨论新方案，故本轮整合为可确认草案，未授权实施为已开始。无新增预算/自动续行期限。

原896 r1..r3见Git历史/本节交接，被撤回897 r3合同保留于Git历史（779988aca），当前文件已按用户要求删除并回收897编号；897不独立建树/执行。当前Specs仍描述888/895旧现状，不能把本草案写成已交付。设计见`docs/design/testing/crate-and-rust-module-tests.md`（本轮新增target-state）。

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
| SD-07 | modify | docs/specs/auto-test-case/test-purpose-and-admission.md | ACE按行为组织单元/集成/专项，去重与临时测试删除有据、语义改名可追 | 长期回归优先 | AC-12/13 |
| SD-08 | modify | docs/specs/auto-av/project.md、docs/specs/auto-cte/project.md及实际改名库项目卡 | 更新到单一ACE workspace下实际package/库与验证工具名称、消费者路径，实验状态不因改名改变 | 消除probe历史命名误导 | AC-14/15 |
| SD-09 | modify | docs/specs/repository-boundaries/build-cost-baseline.md、docs/specs/auto-test-case/build-storage-lifecycle.md | 多个ACE独立workspace→单一独立ACE workspace、多crate/单锁/同树target；默认普通测试零ACE构建 | 构建与存储统一且按需 | AC-10/15 |
| SD-10 | modify | docs/specs/tool-execution/project.md、docs/specs/auto-cache/project.md | 有界修复：复用已有std-only owned进程原语保Command环境/partial/清理，私有FFI target不受外层覆盖污染；不新增runner或工作负载豁免 | 实际编译消费者与冻结条件 | AC-03/10 |

## 6. 测试设计

均为拟执行，未实跑。work持真实实现/反例，review独立逐AC及SD，merge持冻结最终固定组门。脚本单测沿用现有治理测试并修改真实消费者，不新建一套通用检查器。

| case_id / owner_stage | AC | 真实入口/区分性输入 | 预期oracle与边界 |
|---|---|---|---|
| V-01 / work | AC-01/02 | 新cargo t入口parser、parser+vm、未知名/空匹配/相似命名空间 | 精确package+mod子树，含parser的decls等；兼容组同调用、跨组时间区间重叠；未知/零选失败，不靠子串误选 |
| V-02 / work | AC-01/05 | 同namespace新增/删除/改名、宏展开/cfg、同名不同binary；正常nextest枚举 | 由Cargo真实发现自然纳入/移除，迁移全集差集及r5有据合并/删除明确；无分片目录或全局源码扫描前置 |
| V-03 / work | AC-03 | 一绿一红/崩溃/超时/缺child/check/错oracle/准备失败 | 原终态/首次红不覆盖，缺项不冒PASS，未完NOT_RUN，原包装oracle拒绝；重放可追 |
| V-04 / work | AC-04/05 | parser私有改动与共享AST改动两份真实影响清单；固定旧入口/周期 | 计划具名组及必要集成依据可review，无自动图扩773叶；原日常/周期义务有去向，专项按触面 |
| V-05 / work | AC-10 | 干净普通VM/legacy TestCase树、list组名、显式ACE组、伪装普通组补建ACE worker | 前三非ACE场景实际ACE构建=0；必要ACE正常准备，未声明补建构建前失败；核真实argv/产物而非只看测试是否运行 |
| V-06 / work | AC-09 | AAVM root/宏/生成/独立parity入口、普通A2R/VM/两条plan492正控、显式ignored小例 | 实验默认忽略不充绿、重生成不复活；原代码/oracle保留；普通组不误关；手工可运行，原T3限制保留 |
| V-07 / work | AC-06 | 固定parser/vm/a2r和适用旧日常选面，同源/配置/负载记录 | 原单次nextest与新入口同选面对照，分别计冷构建/执行/清理/整命令、CPU/内存/磁盘；ignore减少单列，不承诺50秒/P90 |
| V-08 / work | AC-10 | AV+MN+真实shim消费者；普通debug=0、显式debug=2、错profile/hash | 实际配置生效、target复用且原行为/正式门不变；错来源消费失败，regular/batch仍隔离 |
| V-09 / work | AC-11 | 自有闲置缓存预览/执行、活动/预留/证据/源码/未知路径/reparse拒绝与中途失败 | 实际释放唯一字节可核；不穿透/不动他方、不把半删报成功；原始证据保护 |
| V-10 / work | AC-12 | ACE各包新入口正常枚举、命名case定向重放；核nextest资源/平台/feature/进程条件 | 语义模块可选、少量入口及例外有据，无计划号测试身份，关键反例仍红；不额外建立运行时目录 |
| V-11 / work | AC-13 | 旧测试→保留/改名/合并/删除清单，按各能力选正例和区分性反例 | 每项合并有等价依据、删除有临时用途证据；有效oracle/边界/样本不削弱，不以数量守恒拒绝有据清理 |
| V-12 / work | AC-14/15 | 改名后的manifest解析、原真实消费者/worker/shim/固定组构建与代表性测试 | 无悬空path/旧活动过滤、库职责与依赖方向不变；文档历史引用无需抹除，协议/fixture不变 |
| V-13 / work | AC-15/10 | 新ACE根cargo metadata --no-deps --locked、逐包及跨库代表性验证；干净普通测试target执行默认cargo t与普通组、只列组名，再显式cargo t ace | 实现成员同属新workspace_root/lock/target；普通三种入口实际ACE编译和worker准备均为0、ACE产物无新增；cargo t ace能构建执行ACE常规完整集合且不隐式叠加普通日常集合，原专项条件保留，非仅不运行测试；profile与依赖变化可核 |

本次迁移允许一次实际全范围**发现对账**，不等于重跑全部用例；移除旧逐义务目录后仍要证明每项有新去向。执行验证只跑受影响固定完整组及具名集成/真实消费者，冻结门记录package/target/features/profile/过滤/工具身份与真实结果。旧tm/目录生成的负控按退役责任调整，不能删失败断言来隐藏漏测。最终新架构未落地前不得把现行仓规门视为已通过；T-08同步AGENTS/文档/工具入口，技能外仓维护列具体交接，不扩大本仓写范围。

原日志/探针/截图本地保留不入库，摘要记命令/退出码/源与工具身份/首次失败/本地路径/hash；14天及hold/预算规则保持。纯规划不运行Rust/tests/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可核完成条件 |
|---|---|---|
| AC-01 | work | 一份固定配置对应真实package/target/mod，首层完整职责及散落测试有归属；不需tm/list/全仓目录/动态图；VM不再机械细拆；V-01/02 |
| AC-02 | work | cargo t具名组及多组一次兼容nextest并行调度，原资源约束/HTTP串行/截止保持；无参数保固定日常范围，编译限制明确；V-01 |
| AC-03 | work | 原真实结果/包装check/oracle/源码身份与失败/生命周期门继续有效，父exit0不代子项；V-03 |
| AC-04 | work | 每计划能直接写固定完整组命令和必要集成理由，无自动依赖图拉全仓；真实共享变化仍有明确下游验证；不按用途批量删测；V-04 |
| AC-05 | work | 旧实际枚举到新组并集守恒，除已批准ignore及r5有据合并/删除外，每个原身份/条件/oracle可追；tm退出/旧别名/周期/治理消费者承接且零静默退全量；V-02/04 |
| AC-06 | work | 新固定组完整单轮及代表旧日常同选面对照，冷编译/运行/整命令/CPU/内存与稳态唯一存储分列，超时慢例具名；不以忽略/空目录或估算报收益；V-07 |
| AC-07 | review | 冻结SHA独立逐AC/SD/V、遗漏/延后/workaround及健康审核通过；债务具名不掩盖合同缺口 |
| AC-08 | merge | 最终审查后固定完整组冻结门、真实合入/规范沉淀、896归档、897按用户要求删除并回收序号、留存预算、fresh guard clean清树；周期仅到期触发 |
| AC-09 | work | AAVM/AA2R精确全范围默认ignore、代码/手动重放保留，普通A2R/VM/消费者不误关；AGENTS/Specs一致；V-06 |
| AC-10 | work | 无关ACE零构建；必要ACE宿主/动态工具显式隔离；debug=0/完整调试、同树复用、shim来源与正式参数保持；V-05/08 |
| AC-11 | work | 旧缓存迁移与实际回收有owner/预览/执行/失败收据，活动/测量/证据/源码/他方保护；不把target转存证据，V-09 |
| AC-12 | work | ACE按行为分模块及少量Cargo测试入口；测试文件/mod/fn去计划号且定向选择、资源隔离不退化；逐包target/exe/PDB前后数据，V-10 |
| AC-13 | work | 全ACE测试有可审计去向；相似测试按覆盖等价合并、临时测试有据删除，长期反例/专项/oracle/原规模保留；V-11 |
| AC-14 | work | probe目录/包按真实库或验证职责整理，所有真实消费者/工具/路径和当前Spec承接；真研究可留probe并说明，crate/mod/workspace区别明确；V-12 |
| AC-15 | work | ACE实现在experimental/ace单一独立workspace，多个职责清晰crate、单Cargo.lock/同树target、继承profile和真实消费者完整迁移；无常用实现残留独立probe workspace；普通cargo t零隐式ACE编译，cargo t ace显式运行ACE常规自动测试，专项条件保持，V-13及V-05/08/12 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 具体路径/交付及验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 用户确认r4 | 唯一896树核HEAD/890/894/893接口；冻结旧真实选面和897空间/用途/active owner基线；盘点所有源模块/散落测试及当前入口，不沿用tm计数作新架构 | AC-01/04/05/10/11 |
| [x] | T-02 | work | T-01 | 拟新增scripts/test-groups.toml；先lexer/parser最小组与现有launcher薄分发真链；V-01/02；固定完整职责映射，明确namespace/target/features，非零选面校验 | AC-01/02 |
| [x] | T-03 | work | T-02 | 在固定组文档/配置中写明集成组及选择指导；两种parser改动范围实例，取消自动消费者图；散落历史测试归组必要时只调整test mod挂载 | AC-01/04 |
| [x] | T-04 | work | T-02/03 | 对比旧真实枚举与新组正常枚举，并集/条件/oracle/包装绑定逐项去向；差集和授权例外摘要，机器清单本地；V-02/04 | AC-05 |
| [x] | T-05 | work | T-02 | `.cargo/config.toml`、launcher与薄脚本完成cargo t多组兼容合并/并行；原Job/结果/trace机制复用不经module_catalog，V-01/03；无第二runner | AC-02/03 |
| [x] | T-11 | work | T-01 | aavm*.rs、宏/生成和parity实验逐ID默认ignore，保代码/手动入口、plan492（7条）及普通A2R；V-06及授权差集 | AC-09 |
| [x] | T-12 | work | T-02/17及T-01写窗 | tests.rs/必要feature或target、prepare_tools/toolchains/真实shim接线；ACE隔离、debug=0/2、同树兼容target与来源，V-05/08 | AC-10 |
| [x] | T-13 | work | T-01/12/16/17 | 复用现有工具或最小build_storage.py新入口：owner预览/受保护拒绝/闲置旧缓存回收，不清890/894；V-09，实际唯一字节收据 | AC-11 |
| [x] | T-06 | work | T-03/04/05/11/12/15/16/17 | `run-full-regression.py`及全部本仓活跃tm消费者/治理脚本迁新组，tm明确退役提示，旧义务/周期承接，V-02..06；跨仓仅owner交接不擅改 | AC-01..05/09/10 |
| [x] | T-07 | work | T-06/13 | 代表固定组/旧日常同选面实跑及空间/成本对照V-07，吸收897库存/减量结论；不重跑全目录/收益矩阵，实际背景与慢例分列 | AC-06/10/11 |
| [x] | T-08 | work | T-07 | SD-01..10、AGENTS与本仓入口文档同步，原叶规则明确替换；列需维护技能/在途计划的准确接口交接；健康及逐AC证据，冻结execution_done | AC-01..06/09..11 |
| [ ] | T-09 | review | T-08 | 同树独立上下文auto-plan-review逐AC/V/SD及897吸收对账、遗漏/健康/债务；绑定代码SHA，有finding回work | AC-07 |
| [ ] | T-10 | merge | T-09 pass | auto-plan-merge按新固定组冻结门、真实合入/规范沉淀；896归档；897已按用户指示删除，不再归档或双份完成；留存/周期及guard清唯一树/分支/空组 | AC-08 |
| [x] | T-14 | work | T-01；核ACE在途写窗 | 盘点本节范围所有测试及库/工具消费者；给出保留/合并/删除理由、语义名称与逐包入口/改名表，本地明细+Markdown摘要，不新建运行时测试目录 | AC-12/13/14 |
| [x] | T-15 | work | T-14/17及相关包新路径稳定、文件释放 | 按能力逐包合并测试入口、命名/fixture复用与有据删临时case，更新过滤/资源配置，V-10/11；不借机改协议或削压力 | AC-12/13 |
| [x] | T-16 | work | T-14；与T-17同批迁移、真实消费者释放 | 按已核实职责改库/验证工具目录及包名，迁Cargo path/工具/shim/脚本与固定组，V-12及逐包存储对照；与T-17一次搬迁并与T-12/13共用路径决策避免重复迁移 | AC-14/15/10/11 |
| [x] | T-17 | work | T-14及ACE相关在途写面释放；与T-16同批 | 建立experimental/ace统一workspace/成员/单锁/同树target与根exclude，承接继承配置、依赖版本、release有效profile和真实消费者；迁宿主到conformance，注册cargo t ace固定入口并按包选择；V-13/05/08/12，冷/稳态存储分列 | AC-15/10/14 |

## 9. 复审记录

r1..r3为本文件Git历史中的旧方案：先合并调用，后补AAVM ignore与897接口。无实施进度，不把旧方案pass自检视作新方案验收。旧任务/AC语义被本r4整体替换，897历史任务映射见§4；无已完成项被擦除。

2026-10-31 / stage:new / plan_revision:r4 / outcome:pass（规划交接自检，非实施） / authorization:用户提出合并896/897并放弃tm，改固定crate+Rust mod首层；AAVM ignore仍继承 / grounding:实际auto-lex/core parser/vm/trans、宿主tests.rs、Cargo/nextest旧入口及897 r3 / changed:整体架构/任务/AC/SD替换，固定组代动态叶及图；吸收profile/target/清理/用途全部目标，不承诺同crate编译隔离或50秒 / preflight:13任务/11AC/9V/6SD，897不独立执行；未编译/测试/清缓存/建实施树 / next:确认本r4后T-01。

## 10. 待澄清事项

r4已获全链路实施授权，r5追加范围由本轮用户要求授权；不另要求选择一套新复杂调度系统。T-01/02核完整首层组及实际namespace，不能凭名字漏测试；r6单一ACE workspace及普通cargo t零ACE构建是明确要求，不再以多个历史workspace为默认方案；更细VM拆分明确在本轮之外。897旧稿50%空间硬门未获批准，本r4改真实同工作量比例报告，确认本r4即采用新口径。固定日常无参数包保旧范围，扩大默认范围仍需明确决定；ACE临时/重复测试依r5逐项核实后处理，范围外停用其他研究仍需明确决定。执行中的等价实现不重复索要批准；若必须改变以上义务/边界则先呈具体差异。

2026-10-31 / stage:work-entry / plan_revision:r4 / authorization:用户明确同意896 r4并要求独立复审、合入、清理；897删除并释放序号 / clarification:无参数cargo t保日常，显式cargo t all为语言仓全部固定组，原ignore/条件保留；此为用户问及的命令口径具体化 / status:executing / next:T-01。

2026-10-31 / stage:new / plan_revision:r5 / outcome:pass（增量规划自检，非实施验收） / changed:G6/G7、§2.6、T-14..16、AC-12..14、V-10..12、SD-07/08；T-06依赖新增整编任务 / authorization:用户本轮四项ACE整编要求；原r4全链路授权保持 / grounding:实际Cargo.toml：AV core/probe两个包且probe下74顶层测试文件，CTE三包、artifact/ABI为lib包；仅静态读源未编译/删测试/改库名 / next:同896权威树按T-14→T-15/16执行，核在途写窗；独立复审逐项覆盖新增AC。

2026-10-31 / stage:new / plan_revision:r6 / outcome:pass（增量规划静态自检，非实施验收） / changed:G8、§2.4/2.7、T-17、AC-15、V-13、SD-09，T-15/16改为统一workspace与命名一次迁移；SD-08更新 / authorization:用户明确要求单独ACE workspace、多职责crate、测试整编/去probe名及普通cargo t不隐式编译ACE / grounding:实际syntax、semantics、AV、CTE、A2X为多个workspace，各自继承和profile需承接 / preflight:17任务、15AC、13V、9SD；无代码/目录/锁文件迁移或测试执行 / next:同896权威树T-14核在途写窗与迁移映射，再T-16/17，最后按新路径整编及验证。

2026-10-31 / stage:new / plan_revision:r7 / outcome:pass（命令合同静态自检，非实施验收） / authorization:用户指定ACE测试使用cargo t ace / changed:§2.7、T-17、AC-15、V-13明确公开ace入口；普通cargo t零ACE构建与原专项条件不变 / next:依r6任务顺序实施，不另建runner或发现机制。

2026-10-31 / stage:work / T-01 completed / worktree:D:/autostack/.wt/lang-896/auto-lang / branch:plan-896-dev / base:6e48fe11c / dependencies:auto-down c3464d0, auto-ui 69690df（只读兄弟源码，原依赖解析所需） / evidence:主检出零代码WIP；native git repair修复WSL建树gitdir后路径/分支核验；原目录7165义务保迁移基线，AAVM精确131测试（非整个138义务顶级组）；897已删除、next-id=897 / initial real checks:lexer74及parser149通过；parser+VM1044通过，原child绑定核验；AAVM130条件内测试全部ignore、1里程碑未触发，执行0非PASS。

2026-10-31 / stage:work / incident:显式一次迁移发现（执行0测试）因ENOSPC中断；汇总旧结果文件写入被截断，原43组成功发现及4组编译失败的独立command/stdout/stderr仍在本地，已重建恢复清单 / corrective:新收据采用临时文件fsync+原子replace、独立每组result；Cargo准备前2GiB空间检查；没有停止他方进程 / owned cleanup:只删除896 target内763个PDB路径及debug/incremental，预先核精确绝对路径/无reparse/无896活跃构建；D空闲634880→24968552448字节，实际空闲增加24967917568字节；890/894/主检出缓存未动 / next:只补30个未完成组的发现，不重复已完成43组或以空目录报空间收益；四个未注册历史crate编译红保原输出待具名归因。

2026-10-31 / stage:work-scope-check / approved_execution:r4（用户本对话明确授权） / observed_shared_plan:r7新增ACE单workspace与测试整编，来源授权尚待本对话核对；已发出范围确认 / completed_r4_checks:原7165义务实际迁移发现0遗漏、4个新覆盖crate原编译红保留；parser+VM完整1194通过，兼容一次nextest并发峰20、执行60.977s；pilot24和ACE native3原check/source/oracle通过；固定组19/历史审计49/周期26控制通过 / discovered_and_repaired:日常首次运行漏入MN collections的2条ACE测试，cfg feature及worker_binary_for legacy guard补齐，增加执行前ACE归属拒绝；原运行保留，修复后daily3073适用声明中ACE=0，真实终态待本轮结束 / source_scope:不将r4代码宣称满足新增AC-12..15，不在范围未明时迁ACE目录或合入。

2026-10-31 / stage:work-checkpoint / code:9464b7263 / implemented_contract:r4（共享文件r7范围待用户确认，不冒称新T14..17完成） / final_r4_daily:修复MN collections漏feature后3073实际通过、ACE适用声明0、native执行78.813s、整命令105.010s、全流程CPU310.844s、最大Job峰值17759928320字节（准备/执行分列原meter可查，不声称历史50s或整仓低内存） / controls:固定组20、迁移审计49、周期驱动26、变更Rust格式通过 / independent_review:已向fresh-context子代理派发同树只读r4实现审查，绑定9464b7263；全现行r7未认证 / next:等待r4独立finding及新增范围确认，未合入/归档/清树。

2026-10-31 / stage:phase-review-and-repair / reviewed:9464b7263 / reviewer:独立上下文root/review896_r4 / verdict:needs_fix，F01原Case输出域审查丢失、F02五PS消费者仍tm、F03规范旧规则、F04同选面对照缺失 / repair:b8e2396b0，原owner最终域审查在任何结果写入前、PS显式固定组及真实终态必需ID、规范旧leaf/schedule移除；真实pilot保护域拒绝/合法24check、24固定/49审计/27周期控制通过 / cost:同源3073原生direct-nextest整78.140s/执行71.537s，固定warm整83.020s/执行80.887s；完整lexer74双新鲜target稳态292562845/292562327字节，几乎不变，debug0三例61.4%结果不外推全仓 / next:独立定向phase复核已派发，r7额外ACE结构范围仍待用户核对；未标execution_done/未合入/未归档/未清实施树。

2026-10-31 / stage:phase-repair-review / F01..04:独立复核b8e2396b0按限定关闭 / F05:混合parser+http全局串行，4444ea6d4移除多余threads1，经独立单项控制pass并关闭，HTTP资源组和legacy-th不变 / scope:phase only，AC06/10/11完整证据边界保留；本对话r4授权与共享r7新增范围待核，未进行ACE结构迁移、未授全计划reviewed / checkpoint:产品修改全部在896分支提交，报告md入库、raw本地保留；当前无896native构建/测试任务待运行，实施树留供继续工作，897已删除且next-id=897。

2026-10-31 / stage:work-resume / authorization:用户对异步范围问题明确回复“一并实施 r7 新增范围”，r7全部T01..17/AC01..15为本会话合同；实施→独立复审→合入→清理全链路授权保持，不再因r4/r7范围重复询问 / tree:复用唯一lang-896/auto-lang；已rebase master9c3db75da，r4四提交range-diff4/4全等，HEAD47cc768d6；旧phase review仅覆盖其原代码范围 / next:T14职责、包、测试、消费者与profile/lock冲突盘点→T16/17一次迁移→T15测试整编；保其他树源码/缓存。

2026-10-31 / stage:work-r7 / completed:T02/03/05（沿用已独立phase复核的固定分组、静态消费者指导、一次nextest并行与原结果核验） / implementation:29成员ACE单workspace/单锁、193→18集成targets、源/协议资产保真，no_std运行时保专用panic-abort profile；新增语义脚本适配的有效/零选/缺模块/list/错误传播7类控制通过，真实closures4+公开API12通过 / r7 gate:first ACE全组发现遇新增performance命名空间的super层级错误，已修复；SDK桥接16真实通过；当前重新执行ACE常规完整集合，不把发现/编译当测试通过 / remaining:最终ACE/普通语言零ACE及A2R实测、迁移身份对账、profile/hash反控、自有闲置缓存实际回收、完整独立复审与合入收尾。

2026-10-31 / stage:work-r7-repair / frozen_migration:02340e449 / T14 completed:434 Rust源/1916测试函数全部保源码注册、1915 native身份+1真实shim状态探针；193→18集成target、有据同源去重、183资产原字节保持，新增Git属性防CRLF夹具入库归一化 / public real gate:ACE2031实际2030过/1采样零证据，桥接16过；旧失败和MSVC辅助残留仍严格红，未借准备期豁免冒绿 / independent_targeted_review:review-r7-process-and-sampling.md needs_fix + review-r7-executor-design.md设计可行，非最终复审 / bounded adaptation:T05/12内修同步首采样和join后取值，operator target仅外层显式arg，真实编译调用复用既有auto-tool-execution（无新runner/无ACE到语言依赖）；补SD10，不改冻结oracle/预算/正式规模 / actual controls:37 owned设施完整控制通过；冻结后FFI019、A2X真实生成/rustc、2采样反控、原4请求内存门、3 artifact timeout/partial/错误适配控制通过；修复前两种命名单例均真实重现exit0+vctip残留，修复后严格Job不再残留 / ordinary isolation:空target原日常3073实际枚举/3072过/1 FFI变体红，实际ACE包/测试/worker构建0；原019宽窄断言在outer target隔离修复后真实通过，首次红保留 / remaining:无效UTF8反控、最终正常日常/ACE门、A2R与完整AV/MN profile/空间对照、实际闲置缓存回收、最终独立复审及合入/归档/清树。897仍删除且next-id897。

2026-10-31 / stage:work-r7-freeze / code:e12dfff74 / T04/T11/T15:迁移真实枚举与原测试去向完成、AAVM安全手动ignored原例1PASS、语义整编与17个编号身份修复独立静态闭；183夹具gitblob逐字节同、479外部锁tuple无新增、29成员直接path全部存在 / actual:A2R完整386PASS（执行40.894s/整124.368s），ace batch12+桥接16PASS；f558 workspace2036中2033pass/3 trace PID重用撞名及1辅助残留仍红并保原结果 / bounded_repair:原事件create_new不放宽、持久claim分配身份，2原生控制PASS；真实ExitStatus内部携带、39设施控制PASS；剩余真实编译消费者承接owned原语，未扩大默认忽略或改oracle/样本 / independent:review-r7-static-f370与repair-f558、trace-and-output-design，有界修复静态复核中，最终r7尚未pass / pending:新冻结完整门、AV/MN全regular双profile/错profile实消费、实际闲置缓存回收、最终独立复审/合入清树。897保持删除/next-id897。

2026-10-31 / stage:work-final-evidence / frozen_code:7850efb84 / T12/T13:单ACE workspace实际1027×2 profile全regular同ID/源码未变，debug0 cache1620927405/full-debug4565256354（64.494%该范围）、准备34.257/46.019s、执行47.147/67.944s；真实runtime-dev库由原prepare消费者拒mn.admit.runtime-content且canonical admit，原hash反控保持；必要负控lib单独保，最终guard实回收其15558580字节只计控制 / production_cleanup:4精确旧根预览/idle/祖先owner与后验不存在，释放6225080204唯一文件数据，890 G盘worker保持；C01..04经独立反控修复、35轻控，未知/opaque/drive-relative拒绝 / ordinary:当前公开cargo t3075实际全PASS，nextest64.715s/完整run66.587s/整89.742s，未达≤60s目标，保旧冷target边界及FFI先红后修记录 / finalACE:87整门2065中2062pass，3功能时钟/协议反控未观察到原限定结果，Job干净，桥接16+batch12pass；同源码三项隔离真实绿，保首次红；仅精确3项本池num-test-threads排他，原700/900ms/全部budget/fault物证不改，785完整门正在跑；不宣称唯一原因或他方进程隔离 / remaining:785完整门、A2R trace新消费者复验与894 verifier、最终整体独立复审、主分支合入与归档/清树。897原稿仍删除/序号897。


### 2026-10-31 日常时长裁定与最终验证

用户明确接受“64.7也够了，后续再想办法优化”。日常3075项实际PASS、nextest64.715秒、整命令89.742秒作为本轮日常验收；不继续为凑50/60秒删测、降规模或扩优化范围。固定首层分组和独立ACE迁移继续按r7交付。

A2R最终一轮与890争用全局TEMP固定rmeta输出，386中385PASS/1真实compile_gate因os32失败，原红target/a2r-final-config保留。b2c4eedd3仅将该测试两类产物路径置于每调用TempDir，作用域覆盖全部工作线程，原输入/8worker上限/rustc参数/ledger/skip/断言保持。定向1和完整386实际PASS（target/a2r-private-temp-control、a2r-final-private-temp）。独立静态无finding；最终完整ACE与PS894消费者顺序验证后冻结总审。


### 2026-10-31 r7 execution_done 冻结交接

实现及规范提交216dfdc14（前置真实整门b2c4eedd3）；最终只删除新performance wrapper未用import，805个ACE-AV native身份/ignored前后相同且实际编译exit0，未修改测试输入/assert/配置。真实终态：日常3075、A2R386、ACE2065+16+12全部PASS；PS894 acceptance12步骤PASS（含33/4完整Case）。35固定/清理、49历史审计、9治理、PS7控制均PASS；设施40、采样4及原IPC1、手动ignored安全1等具名证据按不变闭包沿用。三腿strict Job均None。四项原时限精确预留pool，真实各窗口0交叠，其余并发峰21；保原首次红与隔离重放，不放宽时限。

V01..13/AC01..06、09..15工作项、SD01..10的逐项来源和限制见[最终验证摘要](attachments/896/r7-verification-summary.md)。同量1027×2、真实错profile拒绝、生产旧cache6225080204字节与验证控制15558580字节分列。日常64.715秒用户接受。472条既有去重有源warning具名核原源码；唯一新增unused已移除。30个M类Rust格式全0、19 Python编译无错、迁移保真/最终额外AV格式全0。D-896-BASELINE-01四处历史all编译红保留，不将all认证绿色。

本轮没有借用户接受时长豁免正确性；原红、repair、guard与实际来源明细在本地target，清树前转存精简证据包。最新直接human AGENTS要求14天/具名hold/500MiB计划预算；技能中的全局latest-five与之冲突时以该直接指令为准，本会话只维护896范围，不删除其他会话证据。AC07独立总审与AC08合入/归档/清理仍待实际执行，未预记完成。
