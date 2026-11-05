---
plan_id: PLAN-937
status: archived
feature_name: CA-D1 共享接口事实与标量ABI桥底座
author: [Codex]
created_at: 2026-11-05
updated_at: 2026-11-05
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-abi/project.md, docs/specs/auto-abi/windows-x64-v01.md, docs/specs/auto-abi/conformance-cache.md, docs/specs/auto-a2c/project.md, docs/specs/auto-test-case/architecture.md]
new_spec_components: [docs/specs/auto-abi/shared-interface-and-scalar-bridge.md, docs/specs/auto-test-case/ace-ca-d-bridge-certification.md]
touched_goals: [GOAL-002]
affects: [auto-abi, auto-a2c, auto-a2x, auto-test-case]
depends_on: [PLAN-797, PLAN-921]
current_step: 12
total_steps: 12
---

# [PLAN-937] CA-D1 共享接口事实与标量ABI桥底座

## 0. 变更摘要

将CA-D拆为937底座→938源码foreign标量→939资源桥。本包在当前797 Windows x64 candidate/descriptor3上交付backend-free接口事实/物理投影、真实native ABI consumer、新HIR→C标量导出和独立C认证；不把无人消费的schema当交付，不把797声明IR冒普通源FFI。架构见[Design41 §12](../design/41-interface-metadata-and-foreign-abi.md#12-ca-d正式拆包与可独立交付边界2026-11-05target-state)，当前来源见[CA-D基线/排期](reports/ace-ca-d-baseline-and-schedule-2026-11-05.md)。

## 1. 目标

1. 同一受检逻辑签名/名义身份/模式/effect/error/capability与ABI物理计划由797 native发射/准入和新A2C wrapper实际消费，A2C不反依赖ace-abi的Cranelift栈。
2. 当前统一前门冷编译Auto标量函数→既有C provider完整业务片段→公开fallible wrapper→独立C caller实际调用；另用受检typed declarations覆盖公开宽标量，清楚区分输入身份。
3. 当前797原三腿真实回归；新桥实际native ABI peer↔A2C scalar export及独立C caller→A2C source export，完整准入/独立oracle/失败out不变。
4. 逐接口/函数桥片段的真实增量、受限桥artifact恢复、依赖失效、lease与原资源/时间门独立关闭。

非目标：源foreign import、统一source会话恢复首编辑、mn源码公开导出、AV/A2R foreign调用、资源桥、通用header importer、任意use.c、公开f32、动态loader/热卸载、跨平台。前两项由938/939关闭；本包绿不是整个CA-D/M1绿。无需等929或934整包，但不得修改其S/恢复接缝。

## 2. 架构方案

受检Auto事实或显式typed候选→校验/归一→逻辑Interface Facts→target/domain physical plan→native ABI或C薄adapter。拟新增`experimental/ace/crates/foreign-contract/`（T-02冻结窄边界）；不含core/AV/Cranelift/旧AST/进程/store依赖，可依赖现semantics-contract与受控codec。797 descriptor/layout/validate中可复用的纯模型保持原wire语义并re-export；abi emit/ctool/worker留原owner。新C侧拟`a2x-c/src/abi_scalar.rs`只做纯生成；host组合拟`abi/src/bridge.rs`接现core/artifacts/tool runner，既有TargetProvider无副作用约束不变。

797 native consumer是真实Cranelift机器码/准入，不是静态解析证明；其业务体本包仍来自decl::BodyKind，不扩共享HIR→MIR以外的native源码lowering。新A2C业务体则必须来自实际Checked HIR/provider，不能按fixture名拼body。C私有a2x_result不可公开：wrapper显式转换ctx/status/out；公开pure仅在有实际总性/无失败契约证据的有限typed声明中使用，普通checked运算走fallible。不得靠重新实现业务或捕获进程ExitProcess来伪造错误通道。

## 3. 技术栈

当前ACE单workspace：abi/{descriptor,layout,validate,semref,emit,cache,ctool,worker}.rs；core受检源入口；a2x-c/{lower,provider}.rs与a2x-core::TargetProvider；artifacts ManagedStore；语言侧AutoDown/TestCase adapter。新crate/模块/recipe/固定组在T-02/09新增，当前不存在。

## 4. 需求分析与背景调查

授权：用户“OK，那么做CA-D的计划”，承前只做规划；允许调查/设计/计划文档，不实施、建树、构建测试、启动goal/agent或干预929。main基面`122725e9a`；797 archived9/9历史delivery1799f86ca非当前祖先，不能复制历史PASS；921 D3c760865c、925/926已交付，933已关闭sema ignored附件/904独立来源余账。929 executing r2 10/14，由934会话获授权收尾T-11..14；934 header drafting r2 1/13不代表未获执行授权，本会话不占其公共面/采样。

执行授权追加（2026-11-05）：用户 `/goal 完成计划937`（/auto-plan-goal 链式授权：work→review→bounded repair→merge；本节取代"仅规划"限制）。唯一树 `D:/autostack/.wt/lang-937/auto-lang`（plan-937-dev，基面 master@122725e9a），材料 scope `D:/autostack/plan-artifacts/plan-937/`（已登记 owner/用途/在途）。

实际差额：abi emitter仍是固定BodyKind；core DefKind只有Function/Method/Intrinsic；a2x ExternalFn仅同模块兄弟签名；native HostCall拒绝，均非通用FFI。private mn bool=I32/Str ptr+I32 len，891 typed宽浮点为整数位载体，公开ABI bool=u8/f64为目标浮点lane，必须转换。a2x-c目前仅依赖a2x-core/core，保持无Cranelift；当前auto-abi Spec是candidate，不发布新ABI版本。

唯一拟树`D:/autostack/.wt/lang-937/auto-lang`、分支`plan-937-dev`（未建）。未来材料唯一scope`D:/autostack/plan-artifacts/plan-937/`，开工登记owner/用途/在途，外部C消费根/target/log显式归scope；merge折叠摘要入计划、全输出对账、删除scope、guard后清树。用户未指定额外人员/模型，不创建其他chat/agent。

## 5. 详细设计

### 5.1 范围与信任边界

| 数据/owner | 本包支持与实际consumer | 禁止推断 |
|---|---|---|
| 逻辑接口 | DeclRef/owner/content版、参数/结果逻辑类型与mode、borrow/alias、effects/error/caps、来源typed或checked-source | 字符串同名=同名义；候选或persisted checked=true即可调用 |
| 物理计划 | autoabi.win-x64.v0.1-candidate、descriptor/3、Windows x64、自然packing、标量lane/ctx/多out/服务版本；保现资源形态codec兼容 | 私有mn/A2C ABI与public别名；integer carrier=f64物理ABI |
| source导出 | 既有i32/bool标量业务，真实源/def/签名凭证绑定，显式导出清单；例`sum(20,22)`=42、除零=失败且out哨兵不变 | 宽源码/foreign import、任意函数无错误pure导出 |
| typed标量桥 | i32/u32/i64/u64/f64/bool/char受检声明、异质非零实参、目标位型结果；固定body只走标明的typed lane | f32公开ABI、宽Auto源数值语法已成熟 |
| 认证/绑定 | 先descriptor+symbol/provider/runtime/target/cap准入，再link/run；C独立header/provider/caller | hash/PASS代本次准入；进程地址/Checked凭证持久化 |

### 5.2 T-02有界可行性冻结

在专属树证明一条最小全链：独立C调用真实C provider生成的Auto sum和fallible div；native ABI consumer调用同一标量接口并验证异质公开签名；候选坏target拒绝且callee计数0；修改一个桥输入仅生产其片段。必须证明新依赖图无环且a2x-c不依赖abi/Cranelift，descriptor/3原canonical字节/未知格式拒绝不变，C生成业务/所需runtime完整保留而不剪语义依赖或重写业务。公开库导出fileset应由正式assembler显式生成无进程main的library形态，不能认证时临时删main；旧exe fileset保持原样。两轮局部方案仍无法满足核心合同就needs_replan，不能保留“后续补真consumer”的schema交付或改预算。提取layout应保797两阶段精确计费/深图合法控制；不得把不可信描述转成safe标志。

### 5.3 数据流与错误语义

输入declaration/checked source→当前名义/凭证/来源校验→逻辑事实→physical plan→生产adapter→绑定实际symbol/object/tool/header/libs→process-local admission→调用→成功才提交out。public多个out按声明序，输入与out别名按契约拒绝或先暂存，清理后一次提交；非法bool/Unicode char、缺caps、未知服务、错provider在调用前拒。raw候选缺ownership/error事实不能安全调用。源trap映射显式status表并保原因，不按退出码/字符串猜；caller有效指针/容量/声明契约的责任清楚，ABI admission不是任意地址安全沙箱。

### 5.4 增量、恢复与资源

逻辑key含声明来源/名义版本/signature/mode/effects/实际读取依赖；physical key另含domain/version/target/layout/runtime；adapter/object按实际读取含backend/内容/工具flags/头闭包；完整binding/link/证书闭包另含实际库/provider实现。库body未被adapter读取时不把整个库hash混入每个adapter key。复用797 cl -showIncludes +适用域v4及候选负依赖，不能重建简化扫描器。接口不变的callee body编辑只重产该body/object；不读body的caller metadata/adapter P=0；实际link/完整fileset装配单列，不冒全部IO0。不同typed/source、native/C namespace互拒。

桥恢复是已生成桥artifact/接口事实的有界解码、内容/semref/依赖/目标/provider重验，producer探针提前装到真实入口；R不重做source编译、类型推导、adapter或对象生产。缺页拒该P0 restore，显式cold另计；恢复后立即编辑桥输入、首坏→修复→再恢复，对未改接口/函数桥P=0、payload额外读写0。**这只证明桥输入层，不签source session恢复/首编辑；该项938必须从真实Auto源重验，不可引用本包替代929。** source export冷链和既有live session body-edit另测真实生成，无全源first-edit承诺偷渡。

沿统一ManagedStore/lease，不新建FFI数据库/进程runner；512真实修订/GC收据和用户文件哨兵、旧在用objects/root存活、最后lease释放后收敛。原797 build60s/run5s/total70s、IPC4MiB/每流1MiB、hot8MiB+暂存索引8MiB、单产物64MiB/请求128MiB，分配前预留、失败清理与取消进程树回收不变。A2X原48MiB总工作集/128MiB store及原更严限制仍适用，不能用C认证120/30/360覆盖ABI上限。daily emit/admit spawn=0；explicit certify实际compile/link/run且不缓存PASS。

### 5.5 写面与接续

| 面 | 本包写入/交接 | owner/释放规则 |
|---|---|---|
| P/ABI | 新foreign-contract、abi纯模型兼容投影/bridge、abi私有test/cside、新C abi_scalar模块 | 937主面，可独立于929；写原abi descriptor/validate需全consumer枚举和冻结兼容，不假借新文件并写旧模型 |
| C | 只读现FnFacts/provider生成结果，以新薄导出组合入口调用；必要lib.rs新模块注册 | a2x-c/lib.rs全文件短窗；不改provider接口、core/source、a2x-core/store/session、原source restore |
| S/N | 不改core HIR/AST/verifier、shared lowering、native编译器或AV | 929/934等原owner；若可行性要求该面即停其依赖任务，修订归938，不以本包暗占 |
| H | ACE workspace/lock、新包完整测试注册、abi worker协议/父decoder、TestCase模块/fixture/manifests/SDK | 929→934及937按具体文件稳定SHA短窗单写；无追加并写。私有实现不持H窗等待；929正式采样优先，错峰 |
| 后继 | 938消费已交付逻辑/physical/绑定/错误/计数API；939消费资源codec布局 | 以D937与接口hash+consumer清单交接；不要求934..936整波完成 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-abi/shared-interface-and-scalar-bridge.md | 原仅797声明IR→中立事实/双consumer/source-C导出与typed边界 | 限定真实交付 | AC-01,AC-02,AC-03 |
| SD-02 | modify | docs/specs/auto-abi/project.md | 新底座/桥入口，797 candidate/原拒绝面不变 | 不误称完整FFI | AC-01,AC-08 |
| SD-03 | modify | docs/specs/auto-abi/windows-x64-v01.md | 增薄adapter转换/ctx/status/out实际支持矩阵，不默改descriptor3 | 私有/公开分域 | AC-02,AC-04 |
| SD-04 | modify | docs/specs/auto-abi/conformance-cache.md | 增桥key/P-R计数/恢复/原限额/独立认证规则 | 可复核增量 | AC-05,AC-06,AC-07 |
| SD-05 | modify | docs/specs/auto-a2c/project.md | 已有内部C语义→新增有限source export，不冒source import | 实际C consumer | AC-03,AC-08 |
| SD-06 | add | docs/specs/auto-test-case/ace-ca-d-bridge-certification.md | 新ABI桥recipe/输入profile/library文件集/严格父读者/独立认证，旧recipe不冒新source | 首例可审定、非旧PASS替代 | AC-07 |
| SD-07 | modify | docs/specs/auto-test-case/architecture.md | 新ABI桥正式adapter/严格协议与原probe来源隔离 | 注册不是认证PASS | AC-07 |

以上仅提议，review后merge沉淀；本轮不写current Spec。

### 首例协议/文件集身份与完整测试注册

原正式父路径`crates/auto-lang-core/src/test_cases/abi.rs`、`mod.rs`与`crates/auto-test-case/src/contract.rs`实际只认原`abi/interop-v1`/`abiworker/1`，新链不能借该recipe冒已支持source。T-02冻结本包新recipe、输入profile/协议schema、source/typed来源、ABI domain/profile/target/工具完整身份、required页清单、实际方向与certificate精确字段集；首例接线任务同步迁移全部父/worker/SDK构造与reader，旧ABI/C exe/Rust证书互拒，unknown字段/错版/丢方向/空选择不PASS。可以复用正式runner，但不复用旧PASS判定。

C库导出采用正式新增library fileset/assembler而非认证时剪旧exe main.c；完整业务片段/runtime/公开header/adapter/manifest/必要CRT依赖都入身份，不能只编可通过的小段。source输入与typed adapter输入分别具名，后者不得核销普通source验收。存在`autotests=false`的包新增test须登记对应Cargo `[[test]]`，固定组必须实际选到完整包/新target并记录selected/executed/条件排除，不以写了测试文件即算覆盖；拟组在本包H任务正式注册前均未执行。

## 6. 测试设计

下列V均为**拟新增/未执行**，work owner见T，真实entry拟`abi/tests/scalar_bridge.rs`、`abi/cside/bridge_*`、`a2x-c/tests/abi_scalar.rs`；T-09使首例`crates/auto-lang/test/cases/ace-ca-d-scalar-937/`与语言父adapter可运行。独立手写C头/原型表/调用者/布局和手算常量入跟踪，生成器不读expected；reference不由actual生成。implementation/internal-check/final-independent-review依次同一专属树，无用户指定人员，不自动派agent。

| Case ID | AC / invariant | setup / entry / work owner | observable oracle / distinguishing assertion | coverage boundary |
|---|---|---|---|---|
| V-01 | AC-01/02 真实双consumer | T-03/04，neutral事实→797 emit/validate与C wrapper | 两方物理签名/真正执行与独立原型一致；故意交换异质arg负控失败 | native声明IR与C adapter，不冒mn source |
| V-02 | AC-03 source出口 | T-05，统一前门编译sum/div→完整C业务→独立caller | 20+22=42；body+1=43；div0保out哨兵、status原因正确；改实际body应翻结果 | i32/bool source；不得硬编码42 |
| V-03 | AC-02/04 typed宽度 | T-04/05，native/C独立混合8实参含栈参数、i64/u64极值、f64负零与安静NaN payload、bool/char边界 | 整数/返回位型与独立C memcpy取位一致；窄化/float载体混用负控红 | typed公开lane，无f32承诺；sNaN只按原支持无运算位传递规则 |
| V-04 | AC-04 严格准入 | T-06，改domain/target/packing/signature/semref/service/cap/provider及非法bool/char | 明确拒绝stage/code，callee counter0；合法非零ctx/有序multiout仍通过 | 不对已知不兼容实际盲调 |
| V-05 | AC-05 key/真实P0 | T-07，fresh/hit/body/接口/目标/工具/头及更高优先级候选变化、只读非读者 | changed reader精确miss；未改桥producer/对象编译=0、payload额外IO0；link单列 | 继承797负依赖全集/原适用域 |
| V-06 | AC-05/06 恢复/失败 | T-07/08，bridge restart→立即首编辑/首坏修复→再恢复，删页/伪造/错来源/旧schema | R深验，无parse/solve/emit；旧root/交付文件字节不变；合法小深图可用 | 不是Auto source恢复首编辑，938另验 |
| V-07 | AC-06/07 资源/计时 | T-08/10，原6场景×30、512修订、超限/timeout/cancel/GC/并发owner | body-edit与descriptor-reject P90≤2000ms；hot/temp/产物帽，Job回收/GC真实收据、lease与哨兵 | 原样本、两阶段预算/父整请求计时不降 |
| V-08 | AC-07 正式三腿/新桥认证 | T-09/10，原abi/interop-v1 +新首例，实际独立cl与native对象 | 原三腿+新source出口/native-peer分别对独立oracle；丢调用/篡改输出/cert/nonce/晚失败/零选择不得PASS | 缺工具infra；daily不spawn；现SHA重验 |
| V-09 | AC-08 边界/完整组 | T-10/11，依赖图/拒绝矩阵/归档delta/冻结职责门 | 不引Cranelift到C后端、不冒源foreign；SD对应实际能力，独立review逐AC | 仅CA-D1，不签M1 |

完整职责组基线`cargo t ace-abi ace-a2x ace-artifacts case-a2c-hir-scalar-921 auto-test-case ace-adapters infra-fixed`；修改core消费者仅限现只读调用不自动扩大source组，实际共享修改需明列追加且遵§5.5禁止触面。新增contract crate和a2x-c完整tests须在固定组覆盖，拟`ace-foreign-contract`、`ace-a2c-abi`、`case-ace-ca-d-scalar-937`（T-09正式注册前不可执行）。新source export含CRT服务时T-02冻结完整对象/CRT/link闭包，不机械套用原freestanding probe flags，不剪依赖过编译；原60/5/70限额仍保。原797正式AutoDown/probe adapter `abi/interop-v1`、`abiworker/1`及性能入口沿当前dispatcher，不以Rust单测代正式采样；原ignored/平台/resource条件不减。性能与929正式采样互斥，`ace-performance`仅触及原适用专项；不按例机械跑全experimental/all。

## 7. 验收标准

| ID | owner_stage | 通过条件 / closing evidence |
|---|---|---|
| AC-01 | work | V-01/09中立依赖图、真实native+C消费、797 canonical/严格拒绝兼容；无孤立schema交付 |
| AC-02 | work | V-01/03实际物理转换含栈参/符号/ctx/out/精确宽度，公开candidate不冒私有ABI |
| AC-03 | work | V-02真实Auto→Checked HIR→C业务→public wrapper→独立C caller，成功/失败和body变化可观察 |
| AC-04 | work | V-04错误在call前拒且计数0，合法控制不被全拒，原out仅成功提交 |
| AC-05 | work | V-05/06每接口/函数桥真实P0、依赖失效与桥恢复首编辑；source恢复明确不在本包核销 |
| AC-06 | work | V-06/07失败原子、取消/资源/lease/GC与原更严预算全部兑现 |
| AC-07 | work | V-07/08文件化首例、独立oracle/正式认证/原三腿与原计时样本全部真实，daily spawn0 |
| AC-08 | review | 独立逐AC/反控/省略扫描与固定职责门、Spec delta核齐，原债不隐匿；pass绑定revision+SHA |
| AC-09 | merge | 冻结职责并集/漂移核对、SD沉淀读回/交付归档、所有输出对账及scope/guard清理收据 |

### 7.1 验证记录（2026-11-05 work，绑定 worktree code HEAD c5d5fa497）

命令/退出码/关键数字（全部真实执行，串行 `--test-threads=1`，worktree `D:/autostack/.wt/lang-937/auto-lang`）：

- `cargo test -p ace-foreign-contract` → ok：22 lib + 3 conformance（含位置制分类实证、tamper矩阵、链路roundtrip）。
- `cargo test -p ace-a2x-c --lib` → ok：abi_scalar 5 测试（wrapper表/库fileset形状+确定性/宽lane拒/状态表有序）。
- `cargo test -p ace-abi -- --test-threads=1` → ok：17 lib + 75 conformance=92；其中 scalar_bridge 12：typed三腿exit0+负控exit≠0、源导出链exit0+stdout ok、body+1翻转（冻结oracle失败=判别）、准入六维命名拒绝、fresh=5/重跑=0/编辑=1新片段+P0(plan/div)、restore深验0生产+旧页不可变+三负控、预算穷尽命名拒、64MiB帽、512churn GC≥500+root/lease存活+收敛≤128KiB+哨兵不变。
- 桥回转门 `bridge_turnaround_six_scenarios_30_samples` → ok（178s串行）：6场景×30，body-edit P90 1770ms≤2000、descriptor-reject P90 0ms≤2000；fresh p90 1448ms / library-hot 3416ms（每跑真实CRT编译链接主导）/ typed-object-hot 152ms / restart-disk-hot 1277ms；JSONL+summary → target/bridge-937/turnaround/。
- 冻结职责组并集 `cargo t ace-foreign-contract ace-a2c-abi ace-abi ace-a2x ace-artifacts auto-test-case case-a2c-hir-scalar-921 case-ace-ca-d-scalar-937 ace-adapters infra-fixed` → 全 PASS（ace-a2x 177/177 于边界许可面按计划扩 foreign-contract 后；case-ace-ca-d-scalar-937 6/6）。
- 797兼容回归：abi 全套（binding/calls/feasibility/contract_layout_admission/resource/cache_turnaround 含原6场景×30与512）在上述 ace-abi 内绿；case-a2c-hir-scalar-921（原A2C首例）绿。

关键实证事实（设计纠偏记录）：Windows x64 混合参数为**位置制**（MSVC 14.44 探针 f1(int×4,double)第5参走栈、f2(int,double,int)的double在XMM1；Cranelift 0.126 get_intreg/get_fltreg 按 arg_idx 选寄存器）——foreign-contract plan 分类按位置制实现并钉测试；797 emit.rs 文档注释中"2 fp args in XMM0/XMM1"措辞与实际（位置5-8全栈传）不符但不改其冻结文件。

修带的预先存在破损（H面附带，非ABI语义改动）：797 binding 测试读取 `docs/plans/attachments/797/semref-retirement-receipt.md`——attachments 已按 2026-10-31 裁定整目录去跟踪，新鲜 worktree 必然缺文件；收据事实已折叠在归档797计划§9（含 autoabi-semref-snapshot/0 与 15f81fd0），测试读目标改为归档计划。a2x-worker 边界许可面按计划（两消费者共绑中立模型）扩 ace-foreign-contract。

材料scope：`D:/autostack/plan-artifacts/plan-937/`（REGISTER.md+probe探针，merge时折摘要后整删）。

## 8. 执行步骤

| ID | owner_stage | 依赖 | 修改路径/可验证成果；对应AC/验证 | 状态 |
|---|---|---|---|---|
| T-01 | work | 用户后续执行指令 | 重核本§4/基线报告/原797三腿、main/tree/实际owner/依赖全consumer；材料scope登记、冻结原样本与oracle；AC-01/07 | [x] [✅ 已完成] worktree组建（基面122725e9a+auto-down兄弟）；基线报告10项SHA256逐一复核一致（descriptor/decl/emit/cache/ctool/hir/native/target/provider等）；929树停在T-10提交无活动写入，未触碰；scope登记REGISTER.md。 | |
| T-02 | work | T-01 | §5.2全链可行性/模型与依赖图/错误转换/恢复预算；以V-01/02/06缩小证据冻结，不成立needs_replan；AC-01/02/03/06 | [x] [✅ 已完成] 最小全链真实执行：独立C调用真实源导出库（V-02绿）+ native ABI consumer三腿（V-01绿）+ 坏期望拒绝且计数0 + 单输入编辑仅产其片段（V-05绿）。依赖图无环实证：a2x-c依赖集={a2x-core,core,foreign-contract}（无abi/Cranelift，boundary测试钉住）；abi→{foreign-contract,a2x-c,a2x-core,...}单向。位置制Windows x64实证（cl探针f1/f2/f3 + Cranelift get_intreg/get_fltreg源码互证）。未触797 descriptor wire（descriptor/3经投影构造、canonical/未知格式拒绝原测试绿）。库fileset为assembler显式library形态（无main、自有schema），旧exe fileset未动。 | |
| T-03 | work | T-02 | 新foreign-contract与abi descriptor/layout/validate兼容投影、re-export；V-01/04；AC-01/04 | [x] [✅ 已完成] crates/foreign-contract（lane/facts/typed/plan/codec，deny_unknown_fields、错版/超限/篡改拒）；位置制ARG_SLOTS=4分类+bool→u8/char→u32投影+无f32规则；abi re-export + bridge::descriptor_from_plan投影（validate::admit_descriptor实测）。25测试绿（22 lib+3 conformance）。 | |
| T-04 | work | T-03 | abi emit/bridge真实typed scalar consumer、物理calls/ctx/out/位型与独立header；V-01/03/04；AC-02/04 | [x] [✅ 已完成] typed_decl.rs冻结族（ChecksumV1位折叠含f64原始位/NaN payload/-0.0、FallibleDivV1）+typed_emit.rs Cranelift callee/caller（栈槽位重解释保NaN payload）+cside/bridge_typed.{h,callee.c,caller.c}独立三实现。三腿PE真实执行exit 0；错期望负控exit≠0；手算常量独立Python折叠核对（修正i64::MIN+123笔误后=5791786002463203090，与Rust oracle/机器一致）。 | |
| T-05 | work | T-03/04 | 新a2x-c abi_scalar+abi host组合；真实source C exporter、独立caller/peer；V-02/03；AC-02/03 | [x] [✅ 已完成] a2x-c/abi_scalar.rs纯生成（公开fallible wrapper：显式trap枚举→status表-101..-105、reason保留原始码、extern原型、library fileset=library.c+abi_export.h+generated.c+runtime+manifest，schema auto.a2x.c.library.fileset/1、recipe ac-a2c/hir-scalar-library-export-v1，无main.c、字节确定性5测试）+abi/bridge.rs host组合（统一前门compile_scalar_source→真实SemanticView/provider fragments→库fileset→admission→CRT闭包/M-T编译→独立bridge_source_caller.c真实调用）。V-02全绿含body+1翻转（冻结caller oracle失败=判别oracle）。 | |
| T-06 | work | T-04/05 | pre-call binding准入/失败commit与正反控制；V-04；AC-04 | [x] [✅ 已完成] admission_rejects_before_any_production：wrong target/domain/packing/signature(同宽替换)/unknown symbol/semref六维命名拒绝；合法控制（非零ctx值、有序outs=值+reason、混合lane值预检）通过；char代理对/布尔非法值在调用前拒（Unicode scalar命名错误）。 | |
| T-07 | work | T-06 | abi cache/bridge逐函数key/P与桥恢复、环境负依赖；V-05/06；AC-05 | [x] [✅ 已完成] ABI-BR1层四键（facts/plan/fragment/library）+逐键producer计数：fresh=5产品、同源重跑=0、body编辑=恰sum片段新键+div键P0+plan键P0（按plan自身digest键控）、库重装1。restore深验（strict codec+digest重核+source身份匹配，0生产、真实payload读），旧handle旧manifest字节不变（不可变历史）；负控：错源身份/缺页/非桥键均命名拒绝。 | |
| T-08 | work | T-07 | 资源预留/失败取消/lease/GC/512修订，V-06/07；AC-06 | [x] [✅ 已完成] 总预算穷尽=命名拒绝（no partial green）；单产物64MiB帽拒绝（produce_counted实测）；512修订churn：GC删除≥500、root+lease存活、managed收敛≤128KiB、用户哨兵未触、lease释放翻转liveness。原60/5/70预算、1MiB流帽、contained executor沿用未放宽。 | |
| T-09 | work | T-05..08，H短窗 | abi worker/语言父adapter/TestCase fixture/独立oracle/完整包固定组；所有生产数据schema严格，V-08；AC-07 | [x] [✅ 已完成] worker_bridge.rs（abi/ca-d1-bridge-v1，同abiworker/1信封、deck ace-ca-d1/1严格语法、797退出码纪律：正PE退出=pe-exit diags）、父adapter双recipe分发+contract注册（source/typed-self/typed-n2c/typed-c2n腿校验、拒旧recipe互借）+stale轴族、首例fixture ace-ca-d-scalar-937（6/6绿：源导出/body-edit负控/typed三腿/bad-deck负控）、固定组ace-foreign-contract/ace-a2c-abi/case-ace-ca-d-scalar-937注册、a2x-worker CRT flags漂移钉。 | |
| T-10 | work | T-01..09，正式采样窗 | 本§6完整职责自查+原正式三腿/新桥/6×30/P90/预算，SD提案与938/939稳定接口交接；AC-01..07 | [x] [✅ 已完成] 冻结职责组并集全绿：ace-foreign-contract/ace-a2c-abi/ace-abi(92串行,含12 scalar_bridge+回转门)/ace-a2x(177/177,边界许可面按计划扩foreign-contract)/ace-artifacts/auto-test-case/case-a2c-hir-scalar-921(原三腿)/case-ace-ca-d-scalar-937/ace-adapters/infra-fixed。桥回转门6场景×30：body-edit P90=1770ms≤2000、descriptor-reject P90=0ms≤2000、fresh 1448/library-hot 3416(编译主导)/typed-hot 152/restart-hot 1277；JSONL+summary落target/bridge-937/turnaround。稳定接口交接：938消费foreign-contract facts/plan+bridge片段键+TypedDecls codec；939消费lane投影+status表。 | |
| T-11 | review | T-10 execution_done | 同一树独立上下文逐条核V/AC、边界/债/差异、warnings/fmt和Spec delta；AC-08 | [x] [✅ 已完成] 复审两轮（needs_fix→修复→final pass）：AC-01..07逐条工件重构+代表性重跑（FC25/scalar_bridge14含回转门/case组6/6/并集9/9）；3项P3卫生缺陷闭包；债候选D-01/D-02在案非缺陷；SD-01..07路径/内容核齐。pass绑定8b52a7157（其后仅测试oracle真实性修复9ef14b80f+组workspace修复4fb28f73e，均为复审记录在案的受影响复验项）。 |
| T-12 | merge | T-11 pass，后续合入指令 | 冻结受影响组/重验触面漂移、SD沉淀/实际合入读回/归档/材料和worktree全路径清理；AC-09 | [x] [✅ 已完成] 见§9合并回执（delivery fa35943ac，ff-only，7 spec哈希读回，主检出冒烟绿，归档+清理收据）。 |

下一可执行入口仅T-01→T-02；稳定接口D门均已有，H与正式采样仍须实际owner释放，不自动启动work。

## 9. 复审记录

2026-11-05：stage:new；PLAN-937 r1；静态规划预检，未执行验证。已把CA-D拆包、native probe/source出口/typed恢复三种身份隔离，源foreign及source首次恢复编辑明确归938、资源归939。最终静态校验结果见CA-D报告；stage:new pass只指可审阅规划，不是work/review/merge通过。

静态规划校验已完成：各节/稳定ID及交叉引用、当前固定组/新组标注、新链接、来源hash、编号唯一/next-id=940和文档范围/空白检查通过；全部实施与复审验收仍未执行。

2026-11-05 stage:work | PLAN-937 | plan_revision r1 | outcome:pass | worktree `D:/autostack/.wt/lang-937/auto-lang` | branch plan-937-dev | base 122725e9a | code HEAD c5d5fa497（15a29b9b6 基座 + 68382861a T-09/10 + c5d5fa497 边界许可，树 clean）| deps: master基面、MSVC cl 14.44.35207、rust-lld(sysroot)、SDK 10.0.26100 | task_ids T-01..T-10 [x]（current_step 10/12；T-11 review/T-12 merge 未动）| evidence: §7.1（92+25+5+6测试、回转门6×30 P90达标、冻结组并集全绿、797/A2C回归绿）| blockers: 无 | next: review（T-11，同树独立上下文）

Work-owned AC 结果：AC-01 ✓（三腿真实执行+797兼容+依赖图无环实证）、AC-02 ✓（位置制物理投影+位型一致+公开candidate不冒私有ABI）、AC-03 ✓（真实源→前门→C库→独立caller执行，body编辑可观察）、AC-04 ✓（六维pre-call拒+计数0+合法控制过）、AC-05 ✓（逐函数P0+恢复深验；source恢复明确不在本包——938）、AC-06 ✓（预算/原子失败/lease/GC/512全兑现）、AC-07 ✓（首例6/6+独立oracle+回转门+daily spawn纪律沿worker协议）、AC-08/AC-09 归 review/merge。统一回归门（冻结并集）已在review前跑绿；review按需复跑触面。

stage: goal | plan_id: PLAN-937 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-937/auto-lang | commit: c5d5fa497 | next: review

stage: review | review_scope: final | reviewer: 同会话自审（计划§6约定不派agent——独立限制：判决由工件重构，未采信执行摘要；代表性证据全部重跑） | PLAN-937 | r1 | outcome:needs_fix | worktree D:/autostack/.wt/lang-937/auto-lang | plan-937-dev | reviewed c5d5fa497 | base 122725e9a | deps: cl 14.44.35207/lld(sysroot)/SDK 10.0.26100 | spec_inputs: SD-01..07目标路径5现存2新建已核 | acceptance_results: AC-01..07 pass（FC 25绿、scalar_bridge 14绿含192s回转门、case组6/6绿、冻结组并集9/9[ace-a2x组于c5d5fa497复绿,其余组输入在68382861a..c5d5fa497间未变]、diff范围41文件全在声明写面、无调试打印/TODO、SD路径齐） | findings: RV-01 P3 fmt漂移(a2x-c/abi_scalar.rs行宽,cargo fmt --check红)；RV-02 P3 未用导入self(worker_bridge.rs:21)；RV-03 P3 未用变量plan_key_canonical(bridge.rs:758,typed链副本;源导出副本入handle已用) | debt候选(非缺陷): D-01 空steps空过为797协议族既有属性(零选择),938+可加固; D-02 797 emit.rs文档注释"fp in XMM0/1"与位置制实际不符(冻结文件未动,§7.1已记) | evidence: 本行内嵌+§7.1 | next: work修复RV-01..03(最小范围:fmt+两处清理+受影响crate编译复验),其余证据按输入未变复用 | 复审修复范围: cargo fmt + 删除未用导入/变量 + cargo check -p 受影响crate零警告 + scalar_bridge抽样复跑

2026-11-05 stage:work（复审修复轮）| PLAN-937 r1 | outcome:pass | worktree D:/autostack/.wt/lang-937/auto-lang | plan-937-dev | code HEAD 见下条复审回执 | 修复 RV-01 cargo fmt（三crate全量，fmt --check 绿）；RV-02 删未用导入；RV-03 删未用变量；测试代码未用解构/导入/mut 同轮清零（lib+tests 零警告）；受影响复验：scalar_bridge 14/14（含回转门160s重跑）于修复后修订全绿，其余证据输入未变复用。| next: review 复验

stage: review | review_scope: final（复验轮）| reviewer: 同会话自审（限制同前） | PLAN-937 | r1 | outcome:pass | worktree D:/autostack/.wt/lang-937/auto-lang | plan-937-dev | reviewed 8b52a7157 | base 122725e9a | deps 同前轮 | findings闭包: RV-01 fmt --check绿；RV-02/03 删除且 lib+tests 零警告；受影响复验 scalar_bridge 14/14（含回转门）于8b52a7157全绿；其余AC证据输入未变复用（复验轮未重复冻结组并集——修复仅fmt+未用代码，组面输入未变；merge冻结门再核）。spec delta: SD-01..07 经复审可沉淀（新增2文件路径就绪、5修改目标现存、before/after与实现一致、无旧账隐匿）。| verdict: final pass → merge（T-12）。目标不再有未决用户问题（§10原条款已由goal授权覆盖）。

stage: merge | PLAN-937:r1 | outcome:pass | prepared: 基线reviewed 8b52a7157+SD冻结+worktree树净 | landed: rebase 7/7干净→master ff-only=delivery fa35943ac（其上叠加记录在案微修复：9ef14b80f测试oracle真实性、4fb28f73e组workspace；range-diff等价+受影响复验）| canonical_verified: docs/specs/auto-abi/shared-interface-and-scalar-bridge.md(c15d5ee6566bc8e0..)、auto-test-case/ace-ca-d-bridge-certification.md(3c5353b78f06c654..)、auto-abi/project.md(7c135f6e876f426b..)、windows-x64-v01.md(b45152cc9937daa1..)、conformance-cache.md(dc709ca6e305a6b2..)、auto-a2c/project.md(c9ac15b0a41ce7cb..)、auto-test-case/architecture.md(a81e243511443345..) 全部于master读回 | 冻结门: 快组三连(foreign-contract/a2c-abi/case-937)于终基线绿+主检出冒烟case组绿；重组并集证据复用（68382861a/c5d5fa497跑，修复轮diff=fmt/未用码/测试oracle，机制/输入/配置未变，按896规则复用记录在案）| archived: docs/plans/archive/937-shared-foreign-interface-and-scalar-abi-bridge.md status=archived 本提交 | cleaned: 见cleaned行（wt-guard后树/分支/组目录+plan-artifacts整删+登记表刷新）| 生产工件观察: auto-lang为编译器仓无部署进程；ACE为experimental无生产二进制/网页包需重建（显式观察项，非跳过）| 批量回归: 937%5≠0；.last-batch-regression按48h规则核对（见cleaned行后记）

cleaned: wt-guard clean×2（auto-lang/auto-down）→ worktree移除、plan-937-dev删除、组目录rmdir；D:/autostack/plan-artifacts/plan-937/ 整目录删除（证据已折§7.1/§9；探针conv.c为一次性位型实证，结论已入§7.1与spec）；.retention/policy.json 刷新（auto-lang/PLAN-937 archived/deleted/no holds）。批量回归到期核对：.last-batch-regression.json last_covered=925@2026-11-03T14:11Z——937%5≠0 且回执<48h，未到期（下次到期由后续%5计划或48h+merge触发）。生产工件：本仓无部署进程/发行二进制/网页包需重建（ACE为experimental；显式观察，非遗漏）。

stage: goal | plan_id: PLAN-937 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed, cleaned) | commit: fa35943ac | next: done

## 10. 待澄清事项

没有需再次批准的旧ABI扩容或预算放宽。crate提取/re-export及完整错误转换由T-02证明，不能先默认成功；若必须修改929的共享source/session才能做D1，停依赖任务并修订归包，保937可独立边界，不能把未交付主目标塞后继。原797候选profile不升版。后续执行/合入由用户另行指定。
