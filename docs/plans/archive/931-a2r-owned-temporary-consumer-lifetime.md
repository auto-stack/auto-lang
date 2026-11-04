---
plan_id: PLAN-931
status: archived
feature_name: Rust HIR转译owned临时值消费者生命周期收口
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-a2x/checked-resource-facts.md, docs/specs/auto-a2x/architecture.md, docs/specs/auto-a2x/incremental-and-cache.md]
new_spec_components: [docs/specs/auto-a2x/temporary-consumer-lifetime.md]
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-test-case]
depends_on: [PLAN-902, PLAN-925]
current_step: 10
total_steps: 10
---

# [PLAN-931] Rust HIR转译owned临时值消费者生命周期收口

## 0. 变更摘要

关闭D-925-2：Rust lowering在新pin前全量drain，使先求值的owned临时值在真正多实参消费者前释放。以既有受检资源事实为唯一规则，明确消费者/嵌套表达式作用域，保源码序、模式转移、失败原子性与尽早清理。C925已修同形scoped drain，可作差分参照，不能作为唯一oracle。

依据[本波调查与排期](reports/ace-next-wave-correctness-and-admission-2026-11-04.md)。drafting r2，0/10；本轮只规划，不建树/改代码/运行测试，不替代完整M1签章。

## 1. 目标

- 同一调用/构造/方法/复合表达式的所有实参在真正消费前保持合法生命周期，源码顺序求值且恰一次。
- view临时在其消费者后清理；move/return转移的拥有值不被原作用域再释放；嵌套消费者只能清其责任域，不伤外层待用临时。
- 失败/预算/trap/短路/每出口不泄漏、不重复free，不用延到函数结束规避提前free。
- 修复接入真实Rust编译运行、函数级增量与缓存语义版本，旧错误片段不能命中冒正确。

非目标：不重做ownership/infer，不改Auto源码语法/参数模式/公开ABI；不重写C runtime、不做map/filter/reduce，不改原source、行为expected、NA/accepted或M1分母。

## 2. 架构方案

权威链：CheckedFunction/BodyResourcePlan/param mode→SemanticView→Rust lowering→RStmt/print/runtime→独立rustc认证。pending临时清理区分“外层未消费”与“当前子消费者已消费”；可采用scope mark/栈或等价可证机制，不盲搬C代码或只删drain。静态资源计划与动态临时责任仍单源，Rust后端只实现该计划。

修复覆盖`lower_pure/lower_move_value/lower_view_value`与实际调用它们的实参、构造、方法、比较/concat/索引等消费者；T-01追全同因路径，`lower_coll`若复用同一错误机制须补，若无关明确记录不改。branch/短路各路径责任分开，禁止跨未执行分支分配/释放。

## 3. 技术栈

现有ace-a2x-rust/core/store/worker、受检text/text-record/collections资源事实、Rust runtime计量和TestCase双轴。真实rustc与独立手算/参考实现认证，不跑旧AST Trans代新HIR腿。

## 4. 需求分析与背景调查

**r2复核（2026-11-04）**：本轮用户只授权复核/修订规划，不施工、派发或运行验证。初读基面 `ffc59e505`，终检 `50555f3d3`（928由3/12推进4/12，源码调查身份未漂移）；D927=`5d5054d5d` 已交付归档并清树，928 executing r2 4/12，929 executing r2 1/14且按用户裁定挂起等待D928；930 drafting r1 0/9。下列 aa6791649/927未合入等为初订调查历史，由本条与[本次复核](reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)取代其状态/写窗描述；不改在途计划进度/授权，未知D不以分支代替。

用户要求规划下一步；本轮只规划、不施工/派发，未指定模型/预算。调查main `aa6791649`。D925=`a37245cd6`、D926=`3fce61b9a`、D923=`ddd42ef2b`、D924=`48c2d8066`已为main祖先；初订时927未合入，最新状态按本节r2复核。

实际缺口见KNOWN-DEBT D-925-2；`a2x-rust/src/lower.rs`在三个pin入口全量`drain_frees`，`a2x-c/src/lower_rt.rs`已有`drain_since`；现C测试`a2x-worker/tests/cases/a2c_rt_restore.rs::SRC`的Wrap.mk("ab","cde")期望head/tail不丢失、span=5。这里是源码/既有收据调查，未新复跑，不把假设当修复证据。

已读auto-a2x/checked-resource-facts、architecture、incremental-and-cache，auto-a2c/record-text-hir-backend及Rust HIR认证合同。实施树拟`D:/autostack/.wt/lang-931/auto-lang` / `plan-931-dev`（未建）；材料开工后仅`D:/autostack/plan-artifacts/plan-931/`，登记owner，merge摘要后全删，核全部输出路径。Spec current-state只在交付时沉淀。

**执行授权（2026-11-04）**：用户目标 `/goal 完成计划931` 经 /auto-plan-goal 授权整链（work 建树施工 → 有界 repair → 独立 review → merge 沉淀/归档/清树），取代 r2 复核轮"本轮只规划"的执行窗限制；不改目标/验收门/范围/预算。

## 5. 详细设计

### 5.1 可观察合同

- 首例用原925同字节Wrap源码，在Rust目标分别核head="ab"、tail="cde"和span=5；另设异质长度、命名实参反序、三个临时、函数/方法/record构造/嵌套调用与短路正反例，值由手算/独立参考，不能仅A2R与C互相同意。
- 延迟清理只到实际消费者结束。内层释放自身已消费临时，外层先前实参继续有效；move值移交后由目标负责，已移出源不复活；别名/排他仍服从verify计划，非法程序仍原阶段拒绝。
- 观察真实生成RStmt/runtime责任：每次分配→最后消费→释放顺序，scope结束live回到预期；自然OOM/注入失败/预算/取消按既有runtime能力及其原失败语义，不为测试新增catch继续运行。fault若只能观测终止，则配同原语受控probe证明中断责任，不拿重跑绿推断故障原子性。
- `RUST_PRINT_VERSION`或等价资源lowering语义轴必须换键，实际producer/store/证书全部consumer核对；错误旧片段同源也不能复用。优先保持作用域版本，若全Rust共享版本导致一次性较宽miss，明确登记而非删缓存；C版本/内容不受无因影响。
- fresh/hit/restart/body-edit与恢复后首编辑、失败暖编辑→修复在真实A2xSession观察。未改函数lower/print/persist为0；恢复必要准入R、模块装配/源码hash/I/O分列。929仍负责去掉一般来源恢复后的全源ensure_env，本包不能借此冒其SYN关闭，也不能隐藏实际parse/resolve成本。

### 5.2 资源、金样与写面

保A2X各族原工作集≤48MiB、prepare/store/IO/源上限及正式P90≤2000ms适用样本；实际同时存活的多实参/外层与内层临时、候选/历史pin、解码与克隆峰值先预留后物化。测试以真实分配/消费/释放轨迹及存活峰值界区分“正确消费后释放”与“直到函数结束释放”；rt.rs目前没有累计存活字节配额，不能伪称有此预算拒绝，详5.3。不缩样本/改错误分类/把语法当资源错。

P面主要`a2x-rust/src/{lower,lower_coll,provider,print,rt}.rs`按同因需要修改；S只读core受检资源事实，不新增推导。H为worker测试/证书/固定组/必要来源清单。可与928 grammar/closure规则并行；929若改a2x-store/worker版本接线，同文件/符号单写并核所有reader/construct/match、base/handoff SHA、schema/有效-STALE-pending与释放。其他会话930现drafting r1、0/9，主体为生成UI交付/L1-L2边界，与本包代码独立；公共test-groups/infra等H仍按实际范围单写，不替其承诺窗口。

933先补904独立manifest材料；本包不等其整包才能写Rust私有逻辑，但最终受影响固定组的缺材料状态不能长期豁免。若本修复确改受影响打印金样，逐文件说明源码语义/正确清理顺序与独立运行结果，仅更新证实受影响的目标表示或版本字段；不改行为oracle、不复制actual批量bless、不删expect。无关Rust/C原字节应保持。

### 5.3 r2临时责任与验证边界（T-01..06，AC-01..05）

临时生命周期覆盖“表达式已生成”到“真实RStmt/RExpr求值完成”的全链；不能把lower_expr返回或准备下一个pin当消费点。T-01冻结每类consumer实际生成顺序，T-02先以整链反例证明局部作用域方案：

| 必须核实的责任 | 关闭条件/错误实现反控 |
|---|---|
| 借用结果延续owner | `TextSlice/TextToBorrowed/FieldGet`等派生view如果在原verify规则下合法，其底层临时须活到最后真实consumer；非法逃逸保持原诊断，不靠隐式clone/提升生命周期绕过verify。支持合法形态由T-01核源码，不承诺新增借用语法。 |
| 语句根和条件 | let/assign/return/if条件、record字段构造在最终RStmt真正求值前不得drain其依赖。替换先完整物化RHS，成功才提交并释放旧值；失败仍保原有旧值，禁止先free旧值再做可失败计算。 |
| 嵌套/短路隔离 | `Logical`当前用 `mem::take(pending_frees)` 放入右支，T-02核外层pending是否被偷走；未执行右支alloc/consumer/free均0，右支只能取走自身责任，结束后外层仍完整。左右两种短路与真正执行右支均须测。 |
| mode与片段调用 | `ExternalFn.modes`已有受检事实，但lower.rs片段external分支当前填View；本包在T-03消费原modes，验证长度/绑定完整性而非默认View。同一程序cold与单函数片段的move/mut/view调用须等价，不能后端再推导或把全部view强转move。 |
| 失败/重复释放 | 在首参成功、后参失败、callee失败、嵌套成功后外层失败等适用点注入原fault；已转移值由新owner清理，未转移值由当前责任域清理。显式free的调用次数/目标身份也检查，不能依赖take空值的幂等或进程退出回收掩盖错调度。若原路径不能满足原失败合同，T-02先给有界全链方案；未证明不得将该AC标通过。 |

资源观测分三域：编译/会话A2X管理工作集沿≤48MiB及原适用门；生成程序沿原分配算术/try_reserve/fault/错误清理合同；live-byte峰值是由独立事件观察核对的生命周期断言，**不是新产品runtime配额**。现rt.rs的`A2X_RT_FAULT_ALLOC`按分配序号触发，不能用它证明存活峰值拒绝或预留先于真实try_reserve。T-02核实际reservation与event时序，T-04在原fault能力上实现可观测反例，纯观测探针不改变产品语义。缺event的分配/隐式clone亦需核实；结果正确但先free后读空、隐式clone补救、延到函数尾、重复free调用均不能通过。

### 5.4 r2恢复与并行关闭条件（T-05..08，AC-04..07）

AXP1/2/3目前是C来源恢复锚，Rust rt/collections没有等价锚；931不得在原Rust restart重装配上写“已证明Rust source-bound纯恢复”。T-05先逐族列实际入口及counter，旧Rust片段/语义键、fileset、来源与证书消费形成完整失效链；同源旧错误fragment/错误新版本字段/坏required边/同名异owner不得命中。一次真实版本迁移miss与其后hit/P0分开观察，不能清整个store掩盖版本轴未入键。

新增V-09的无预热恢复首编辑全链由929提供共同接缝：931可先完成私有修复/原restart路径验证，并在稳定SHA提前交付代码供929消费；最终AC-05/06须有**真实929恢复接口或等价已正式释放并验证的同能力接缝**。若该接缝尚无，记录acceptance pending而非豁免；不必等待D929整包后才写Rust，也不把本包私改通用source环境当并行方案。相同case前后绑定来源/producer/grammar/目标/schema/工具hash，接口漂移使关联证据STALE。恢复成功→首坏编辑→修复→有效编辑及close/最后pin各自核值/状态/生产计数。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2x/temporary-consumer-lifetime.md | 明确跨实参/嵌套消费者临时责任及失败/缓存版本 | 关闭提前释放 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-a2x/checked-resource-facts.md | 补Rust消费原资源事实的临时锚点，不改ownership规则 | 单源事实 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-a2x/architecture.md | 修复适用域/真实认证/版本/余账 | 现状不冒全M1 | AC-04/05/06/07 |
| SD-04 | modify | docs/specs/auto-a2x/incremental-and-cache.md | Rust临时责任语义轴、external原modes依赖和恢复首编辑实际接口/证据边界 | 旧错误片段不能命中，新SYN不冒现状 | AC-04/05/06 |

## 6. 测试设计

T-06使能新文件化目录`crates/auto-lang/test/cases/ace-m1/a2r-owned-temporaries-931/`和独立reference，所有新例未执行。

| case | AC | 输入/入口与work任务 | 关闭oracle与反控 |
|---|---|---|---|
| V-01 | AC-01/02 | T-01/02/03/06，原Wrap源码+非对称2/3实参/命名反序，真实Rust编译运行 | head/tail原字节和5等独立结果；强制旧全drain必失败，C正确仅作peer证据 |
| V-02 | AC-02 | T-02/03/06，函数/方法/构造/嵌套与比较/concat、短路未执行分支 | 源码顺序/各次一次，内层消费不杀外层，未执行分支alloc/free=0 |
| V-03 | AC-02/03 | T-03/04/06，view/move/return与拥有record/Str，成功/每出口/原可用fault | 消费前alive、消费后恰一次清理、move目标责任，失败部分责任可证；无leak/double-free/旧值破坏 |
| V-04 | AC-03 | T-02/04，真实event/多层临时/clone与存活峰值界、close/最后pin | 正确消费后释放在独立峰值界内，延到函数结束反控违反峰值断言；编译预算/生成runtime原合同分列，不虚构累计字节配额 |
| V-05 | AC-04/05 | T-05/06，同源旧缓存/错producer版本与合法历史、fresh/hit/restart/body-edit/失败修复 | 旧错误片段不命中，证书/来源重验；未改函数lower/print/write0，M/R/原ensure_env如实非零，不冒929关闭 |
| V-06 | AC-06/07 | T-06/07，原902/904及C921/925/926/资源source适用集 | 原source/行为oracle不改，真实父侧bytes与独立认证分轴；无关目标字节/身份不回退，材料缺失交933不改判绿 |
| V-07 | AC-02/03 | T-02/03/04/06，合法slice/field派生view、let/assign/return/条件，失败RHS | 底层owner活至最后consumer；原非法逃逸仍拒；旧目标直到RHS成功才替换，free调用身份/次数独立核对 |
| V-08 | AC-02/04/05 | T-03/05/06，同源cold与external scaffold片段，view/mut/move与命名重排，外层pending跨短路 | 受检modes原样传播，值/写回/移交/trace一致，错误默认View及短路偷取外层责任反控红 |
| V-09 | AC-04/05/06 | T-05/06/07，真实929或等价稳定来源恢复接口，新进程restore→立即编辑及restore→坏编辑→修复 | 无dummy/ensure_env预热；必要R>0、未改各P/单元payload生产读写0、实际模块装配另计；接缝未交则pending，不称Rust已有AXP锚 |

职责组`cargo t ace-a2x ace-source ace-adapters auto-test-case infra-fixed`；必要集成`case-a2r-hir-record-text-902 case-a2r-hir-collections-904 case-a2c-hir-scalar-921 case-a2c-hir-record-text-925 case-a2c-hir-collections-926 case-ace-m1-text-source-894`，若改共享事实补`ace-semantics ace-lowering ace-av ace-native`。拟新增`case-a2r-owned-temporaries-931`由T-06注册。正式性能沿`ace-performance`受影响A2X原命名样本/条件错峰，原规模不减，不为低影响迭代重复跑全仓。测试准备/冷编译、运行、认证/清理分列；未执行/条件排除不算PASS。

实现/内部自查/独立review按角色在唯一树顺序进行，用户未指定模型/人，不自动派发。有效未变证据复用，语义版本变化标受影响STALE。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01原缺陷可区分，独立值/字节oracle与真Rust执行正确 |
| AC-02 | work | V-01..03/07/08完整同因消费者/派生view/语句根/片段模式/源码序/分支不变量，受检事实单源 |
| AC-03 | work | V-03/04原预算、失败原子/每出口/最后清理，不用延迟到函数尾规避 |
| AC-04 | work | V-05缓存/证书/来源语义轴变更真实生效，旧错误片段不服务 |
| AC-05 | work | V-05/08/09函数级增量/恢复首编辑/失败修复及真实P/M/R可观察，真实接缝未交则pending，原时间门保持 |
| AC-06 | work | V-06原source/行为oracle/目标兼容与正式文件case/认证齐，933材料及929接口交接明确 |
| AC-07 | work | 冻结完整职责/必要消费者证据、SD与D-925-2余账准确，健康无本包新warning/debug |
| AC-08 | review | 独立逐AC/SD/遗漏与反控复核，revision-bound pass，不能遗留本包问题冒债出包 |
| AC-09 | merge | 交付/Spec读回/归档、所有输出路径对账/材料摘要及删除、guard清树完成，未冒M1总签章 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 产出与验证/AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 当前main/审计§7输入/私有P | 同因调用点/资源事实/原例/版本/预算与933/929/H窗口冻结，AC-01/02/07 | [✅ 2026-11-04 冻结记录 plan-artifacts/t01-freeze.md：三pin入口全量drain+语句根drain+Logical mem::take偷取+external硬编码View四同因点；lower_coll不同因记录不改；rt.rs take()-free静默空串oracle；fault冻结语义=失败零free；C侧drain_since参照] |
| T-02 | work | T-01，私有最小整链 | Wrap/多实参+派生view/语句根/短路隔离/失败清理与真实event峰值可行性；失败needs_replan不减门，AC-01/02/03 | [✅ 2026-11-04 红证据 plan-artifacts/t02-red-evidence.md：925同字节Wrap与语句根两形均"先free后用"且pin不可变致E0596编译红（3探针红）；修复机制可行性由事件账本证实] |
| T-03 | work | T-02，Rust lowering | 真消费者作用域/pin/move/branch/external原modes，V-01..03/07/08，AC-01/02 | [✅ 2026-11-04 提交 ed4c53a26：drain_since作用域机制+消费者锚点后释放+view临时pin mut+Logical侧块自责+external受检modes+MoveOwn实参按值转移（探针暴露的第三缺陷E0308一并修复）；worker conformance 154/154 绿，探针7/7 绿] |
| T-04 | work | T-03，Rust原runtime/probe必要窗 | 失败/早清理/所有权/最后owner峰值，V-03/04，AC-03 | [✅ 2026-11-04 提交 b3a6a17c6：move 由 callee 恰一次清理（账本 1/1）、return 转 caller 责任（1/1）、分支出口替换序（RHS 先物化）、fault 第2分配零free、事件账本独立峰值界=单消费者足迹8字节（延到函数尾反控16）；reservation 先于 try_reserve 事件时序已核（rt.rs alloc_buffer 顺序）] |
| T-05 | work | T-03/04，provider/store证书必要窗 | 语义键/旧缓存拒绝/来源与增量，V-05，AC-04/05 | [✅ 2026-11-04 提交 b3a6a17c6：RUST_PRINT_VERSION 1→2（共享轴一次性较宽 miss 已登记 D-931-2）；旧版本键铸 store 三代重开实测（print/1 片段在 print/2 下必 miss 重产→正确值→第三代全 hit）；Rust record-text 既有 warm_edit_method 生命周期（仅 span 重lower、mk 片段字节不变、坏编辑命名拒绝、修复值23）；V-09 纯恢复接缝缺失按 §10 记 pending（D-931-1，owner=929）] |
| T-06 | work | T-03..05，case/worker/注册H | 使能V-01..09、独立oracle/反控/认证与固定组，AC-01..06 | [✅ 2026-11-04 提交 0d162752d+fmt修复 d7d0aa22c（F-931-R1 关闭：fmt 0 差异、13 探针绿）：固定组 case-a2r-owned-temporaries-931 注册并 PASS（5 case 金样字节比对 + 5 独立手写 reference 实跑 5/22/5/22/43）；worker conformance 13 探针（值/事件/峰值/fault/版本轴/生命周期）；金样为修复后发射的首次捕获（无覆盖既有期望）；bool-local if 仅 text profile（冻结语法面不改，源按语料风格改写避开 if/else-both-return 尾巴）] |
| T-07 | work | T-06，933材料稳定提交/929恢复接缝稳定SHA/适用consumer | 必要受影响金样独立核验、兼容/SD/债务与consumer记录，AC-06/07 | [✅ 2026-11-04：既有金样兼容独立核验——902/921/925/926/894 组原字节全 PASS（冻结语料无该形状，Rust/C 字节未变）；904 组 20 manifest 金样预存缺口（*.json 母规则吞掉，任何检出必红）同通道再生补齐（tracked 三文件 20/20 字节一致后 force-add）记 D-931-3；债务四条入 KNOWN-DEBT（D-925-2 关闭、D-931-1/2/3 登记）；生成码 rustc 警告形状经 902 冻结金样对照确认全部预存（mt_ 命名、AndSC 括号渲染），本包代码零新warning；933 材料/929 接缝交接状态在 D-931-1/3 明确] |
| T-08 | work | T-07，冻结提交/采样错峰 | 完整职责/必要消费者及正式资源/时间门，AC-01..07 | [✅ 2026-11-04 冻结提交 0d162752d（树 clean）：职责组 ace-a2x/ace-source/ace-adapters/auto-test-case/infra-fixed 5/5 PASS；必要集成 case-a2r-owned-temporaries-931/902/904/921/925/926/894 7/7 PASS；资源门经 ace-a2x 内 resource_budget 等随组绿；正式性能：a2x crates 无 ::performance:: 命名样本（实测过滤空），mn/av perf 腿非本包触面不重跑全仓（D-911-4 治理）] |
| T-09 | review | T-01..08完成，停止代码写入 | 独立逐AC/SD/遗漏/健康复审，AC-08 | [✅ 2026-11-04 R1 needs_fix（F-931-R1 测试文件fmt）→ work 修复 d7d0aa22c → R2 pass（同会话工件重建，身份限制已记录；SD-01 目标目录确认新建、SD-02..04 目标存在；merge-owned 冻结并集待跑）] |
| T-10 | merge | T-09 pass及后续实施/合入授权 | canonical沉淀/交付/归档/输出清理，AC-09 | [✅ 2026-11-04 见 §9 merge 收据 checkpoints：prepared(a9220262e 冻结SD)→landed(84c894b92 ff-only, rebase×2 range-diff 5/5 等价)→canonical_verified(4路径+哈希)→archived(本提交)→cleaned(后续提交) ] |

下一入口T-01可独立盘点Rust私有面，不等待928整包；公共H按实际owner领窗。本轮不授权work。

## 9. 复审记录

2026-11-04 stage:merge / outcome:pass / delivery:84c894b92（master tip 前后 e79e4f4e4 簿记提交；ff-only 无 merge commit）/ Spec:docs/specs/auto-a2x/{temporary-consumer-lifetime,architecture,checked-resource-facts,incremental-and-cache}.md / archive:docs/plans/archive/931-a2r-owned-temporary-consumer-lifetime.md（10/10，status: archived）/ cleanup:双树 guard clean 移除、分支/组目录删、plan-artifacts/plan-931 整删、登记刷新、无具名持有 / landing:主检出 release worker 重建（Nov4 19:15）+ 931 case 组主检出复跑 PASS；批量回归未到期（925 后合入 927/928/931 均非 5 倍数、收据<48h） / pending:V-09 接缝联验归 929（D-931-1）；未冒 M1 总签章

2026-11-04 stage:goal / plan_id:PLAN-931 / plan_revision:r2 / round:4 / from:merge / to:done / status:archived / outcome:pass / worktree:(已清) / commit:e79e4f4e4 / next:done

2026-11-04 stage:merge / plan_id:PLAN-931:r2 / prepared:基线 d7d0aa22c（R2 pass）+SD-01..04 于 worktree 冻结为 a9220262e / landed:master 两次前移（37503a3f5=928合入、053cdb9b4=AGE docs）→ rebase×2，range-diff 5/5 全等（旧→新：ed4c53a26→75149c025、b3a6a17c6→b2e0581c9、0d162752d→6c4fc09d5、d7d0aa22c→d23393cd7、a9220262e→ca437dce5→84c894b92），`git merge --ff-only plan-931-dev` → master tip=84c894b92（无 merge commit）/ 冻结并集门禁：12/12 PASS 于 ca437dce5（职责 5+集成 7）；053cdb9b4 为纯 AGE docs/928归档簿记（ace 触面零重叠，证据复用）+ case-a2r-owned-temporaries-931 于 84c894b92 复跑 PASS / canonical_verified:docs/specs/auto-a2x/{temporary-consumer-lifetime(新增,sha16 8cfdc8e5833a2777),architecture,checked-resource-facts,incremental-and-cache}.md 均在 84c894b92 / archived:本提交（计划移入 archive/，status: archived；KNOWN-DEBT 外科暂存仅含 D-925-2 关闭+D-931-1/2/3 登记，其余会话 WIP 未携带） / cleaned:wt-guard 双树 clean→worktree auto-lang+auto-down 移除、分支 plan-931-dev 删除（was 84c894b92）、组目录 .wt/lang-931 移除；plan-artifacts/plan-931 整目录删除（证据摘要已折叠本计划；无具名持有，D-931-1 为债务记录非磁盘持有）；登记表 scope 移除 / merge outcome:pass

2026-11-04 stage:goal / plan_id:PLAN-931 / plan_revision:r2 / round:3 / from:review / to:merge / status:reviewed / outcome:pass / worktree:D:/autostack/.wt/lang-931/auto-lang / commit:d7d0aa22c / next:merge

2026-11-04 stage:review / review_scope:final（R2 pass）/ reviewer:同实施会话工件重建（身份限制同 R1）/ plan_id:PLAN-931 / plan_revision:r2 / outcome:pass / worktree:D:/autostack/.wt/lang-931/auto-lang / branch:plan-931-dev / reviewed:d7d0aa22c / base:da884ab2b / deps:auto-down@7c4d5cb 只读 / spec_inputs:docs/specs/auto-a2x/{checked-resource-facts,architecture,incremental-and-cache}.md 现存（SD-02..04 目标），SD-01 temporary-consumer-lifetime.md 待 merge 新建（frontmatter supersedes/new/touched_goals=GOAL-002 已核，与 §5 规范增量表一致）/ acceptance_results:AC-01..07 pass（R1 全量核验+补充concat sibling代表例；R2 仅增量=F-931-R1 关闭：文件 fmt 0 差异、p931 探针 13/13 于 d7d0aa22c、范围纪律（a2c_coll_cert 预存漂移回退未动））；AC-05 V-09 按 AC 自身条款 pending（D-931-1，owner=929）；AC-08 本收据 / pending_closeout_gates:merge-owned——冻结 d7d0aa22c 上职责组∪集成组并集复跑（0d162752d 分组全绿+格式化后探针绿，格式化不涉语义但按冻结基线纪律重跑并集） / findings:F-931-R1 已关闭；无遗留 / evidence:见 R1 收据+上述 / next:merge

2026-11-04 stage:review / review_scope:final（R1，needs_fix→R2 复审绑定修复提交）/ reviewer:同实施会话工件重建（用户未指定独立模型；判定自 diff/基线源码/复跑输出重建，非执行者叙述）/ plan_id:PLAN-931 / plan_revision:r2 / outcome:needs_fix / worktree:D:/autostack/.wt/lang-931/auto-lang / branch:plan-931-dev / reviewed:0d162752d / base:da884ab2b / deps:auto-down@7c4d5cb 只读 / findings:**F-931-R1**（P3 健康：新测试文件 a2r_owned_temporaries_931.rs 7 处 rustfmt 差异——AC-07 clean formatting；机械修复，不涉语义）/ 已核通过项：AC-01 红机制经基线源码证实（base lower.rs 三 pin 前 drain_frees + pin mutable:false → 先free后用+E0596；t02 生成码记录一致）；复审直接重跑 conformance p931 13/13 绿、case-a2r-owned-temporaries-931 PASS、RUST_PRINT_VERSION=a2x-print/2、a2x-c 零 diff（C 版本/内容无因不变）；AC-02 补充代表例 concat 双 pinned sibling（reviewer 亲跑：值4、零警告、t0/t1 在 concat 消费者后释放）；AC-03/04/05/06/07 证据如 work 收据（职责 5/5、集成 7/7、峰值 8<16、三代 miss/重产/hit、V-09 pending 按 AC 条款）；diff 无 debug 残留/无范围蔓延（+111 行 lower.rs 全在合同内；预存 fmt 漂移文件不在本包 diff）；a2x-rust cargo check 0 警告 / next:work（修 F-931-R1 后 R2 复审）

2026-11-04 stage:goal / plan_id:PLAN-931 / plan_revision:r2 / round:2 / from:work / to:review / status:execution_done / outcome:pass / worktree:D:/autostack/.wt/lang-931/auto-lang / commit:0d162752d / next:review

2026-11-04 stage:work / plan_id:PLAN-931 / plan_revision:r2 / outcome:pass / worktree:D:/autostack/.wt/lang-931/auto-lang / branch:plan-931-dev / base:da884ab2b / code:ed4c53a26→b3a6a17c6→0d162752d（冻结 0d162752d，树 clean）/ deps:auto-down detached@7c4d5cb（只读消费，未改）/ tasks:T-01..T-08 全 ✅ / evidence:（AC-01/02）探针红→绿 13/13 + 金样组 case-a2r-owned-temporaries-931 PASS + 独立 reference 实跑 5/22/5/22/43；（AC-03）fault 零free + 分支替换序 + 峰值界 8<16；（AC-04）print/1→2 三代 miss/重产/hit 实测；（AC-05）V-09 按 AC 自身条款记 pending（D-931-1，owner=929），既有 restart/warm-edit 腿绿；（AC-06）902/904/921/925/926/894 兼容全 PASS，904 manifest 预存缺口顺带关闭（20/20 字节核对）；（AC-07）职责组 5/5 + 集成 7/7 PASS，本包代码零新 warning（生成码警告形状经 902 冻结金样对照确认预存），债务 D-925-2 关闭 + D-931-1/2/3 登记 / blockers:无合同内阻塞；merge 后续门禁：冻结并集复跑归 merge、V-09 待 929 接缝 / next:review

2026-11-04 stage:goal / plan_id:PLAN-931 / plan_revision:r2 / round:1 / from:goal / to:work / status:drafting / outcome:pass / worktree:(未建，拟D:/autostack/.wt/lang-931/auto-lang) / commit:da884ab2b / next:work

2026-11-04 stage:new / revision:2 / outcome:pass（规划合同静态复核，不是实现复审）/ next:work（后续实施指令及各任务实际写窗就绪）。修改见[复核报告](reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)；保全部T/AC编号、总步骤及0进度，r1记录保留为历史；新增V与收口前置未执行，旧调查/历史运行不能为r2新增断言背书。

2026-11-04 stage:new / revision:1 / outcome:pass（规划静态核对）/ next:work（后续明确实施指令下）。未运行代码/测试/性能/认证，全部任务0；D-925-2仍open，由本包交付后按真实证据关闭。

## 10. 待澄清事项

无新增语义裁定。scope标记/分支责任与精确缓存轴由T-02/05证明；若修复需改变原资源上限或源语义，先needs_replan，不能放宽或改expected。完整ABI、SQ/CT不在本包扩写；SYN来源首次编辑的生产接缝仍归929，本包在其稳定接口上联验自己的AC-05/06，未有则验收pending，不改为范围外豁免。
