---
plan_id: PLAN-902
status: reviewed
feature_name: AX-B Checked HIR到Rust的record/text、参数模式与资源闭环
author: [Codex]
created_at: 2026-10-31
updated_at: 2026-11-01
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-a2x/project.md, docs/specs/auto-a2x/architecture.md, docs/specs/auto-a2x/incremental-and-cache.md, docs/specs/auto-a2r/project.md, docs/specs/auto-test-case/a2r-hir-certification.md]
new_spec_components: [docs/specs/auto-a2r/record-text-hir-backend.md, docs/specs/auto-a2x/checked-resource-facts.md]
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-a2r, auto-ac, auto-test-case]
depends_on: [PLAN-892, PLAN-881, PLAN-894]
current_step: 12
total_steps: 14
---

# [PLAN-902] AX-B：Checked HIR→Rust record/text、参数模式与资源闭环

## 0. 变更摘要

在已交付AX-A上增加显式record/text/text-record目标域，消费受检HIR而非MIR：具名record、Str/str/char、字段/构造、static/view方法、合法view/mut/move/clone/拥有返回，真实生成Rust、独立认证、函数级增量及资源/恢复/失败闭环。原R07/R08=42、T03=23、T05=42均为required原源码，不以typed夹具或AV实际值替代oracle。

依据[Design51 §5.3](../design/51-shared-a2x-and-hir-to-rust.md)、[A2X战略](../design/strategy/shared-a2x-and-hir-to-rust-roadmap.md)。只语言仓；复用experimental/ace中现有a2x-core/rust/store/worker四crate，不造第二前端/AV/store/runner，不新增workspace。当前只立项，不建树、不编译/运行探针、不接管899/901或修改其合同。

## 1. 目标

- G1：原record/text/text-record合法源码通过共享语义view/结构化prepare→Rust provider→真实rustc→实际程序，与独立手写reference/闭式oracle一致，类型/模式/资源事实均可追溯到权威验证器。
- G2：正确保名义类型、receiver/实参求值序、view/mut/move、显式深clone、拥有返回/借用来源、Unicode/NUL及重赋清理；不把Rust默认行为冒充Auto语义。
- G3：以函数/方法为增量单位，未改单元真实生产0及居民身份保持，依赖/模式/类型/资源事实变更正确失效；required-data、候选、完整文件集、配额、恢复与close成立。
- G4：日常emit/golden与显式compile/run认证分轴，新身份与892及旧a2r并列；首例纳入文件化、资源/故障、正式计量/敏感性、固定组和版本来源。

非目标：List/Variant（AX-C）、闭包/HOF、mut实例方法/ext/trait/任意重载、用户泛型/meta/模块、借str字段/递归堆record/部分move、宽源码/891私有ABI、Unit入口、console/host/FFI、第二目标、旧A2X切默认或全M1签章。旧core/scalar功能、artifact/golden/recipe/证书行为保持；不借新域自动放宽旧域。

## 2. 架构方案

`原.at+显式profile → 现有ace-core source/verify/受检资源事实 → SemanticView/FunctionPlan/依赖摘要 → Rust typed IR/合法化/runtime/printer → fileset/manifest → 原父侧golden或显式独立认证`。

共享层增加中立名义type/field/method身份、receiver/ParamMode、源码求值/落位、借用来源/资源分类与受检清理计划引用；Rust表示、生命周期拼写、helper、文件组织/卫生/打印归Rust provider。S只补缺失的验证器事实出口/单函数凭证，不能在a2x-core重做ownership/infer，也不从MIR/AV结果恢复语义。候选parts必须是本次已验证的body，不用env旧body发射。

现有`Ty={I32,Bool}`、`ParamFacts`无mode、prepare/fingerprint/session仅core/scalar，需要显式扩展/版本隔离，不能只给Rust printer加分支。目标runtime是已生成Rust的typed数据/资源helper，不是执行HIR的解释器。

## 3. 技术栈

Rust现有ACE四crate、core受检source接口与783 ManagedStore；Rust edition 2021、首期沿892实际Windows x64认证工具链（跨平台不冒认）及现有受控auto-tool-execution/Job、独立手写Rust reference、AutoDown/TestCase、固定cargo t组。生成资源helper属于目标源码，不是native no_std runtime或新Cargo成员。无新增第三方框架、root→实验编译器静态依赖、legacy AST/Trans执行旁路。

## 4. 需求分析与背景调查

### 4.1 授权及真实交付

用户明确授权AX-B正式立项并核892/881/894、参数/资源/增量/认证与899/901边界；本轮仅规划、不施工、不发跨会话消息或创建agent/chat。读取main=`03797707b0e86cbb3bd48e88a9282de5c5c0cab2`，他方docs/号源/WIP保留。current Spec为现状，Design为目标；旧战略里的“892/894草案”是历史快照。
2026-11-01 `/goal 完成计划902`（auto-plan-goal）：用户授权全链执行（work→独立review→bounded repair→merge），goal授权记录于本节与§9收据；后续stage继承授权不再重问。

| 依赖 | 可消费基线与核验 | 限制 |
|---|---|---|
| 892 AX-A | archived；历史收据含实际合入/修复与最终11/11，header 6/11是簿记残留；当前可达清理提交`d6ef24227`，is-ancestor HEAD exit0。现有SemanticView、plan/fingerprint、A2xSession、Rust provider/四文件集、严格scalar recipe与独立cert | 仅core/scalar；原58源码认证与warm预算是已有基线，不等record/text支持；源名键不能直接覆盖方法owner身份 |
| 881 方法 | archived r2、9/9；历史delivery db0773130；当前可达cleanup `c41c57890`，ancestor exit0。source-record/text-record及record-method-lowering Spec冻结static/view、receiver先求值一次、模式/资源及单体方法事实 | 不含mut实例/ext/trait；资源record与纯Copy行为区分；MIR MutRet不作为Rust ABI |
| 894 文本收口 | archived13/13；历史delivery 0a54a3b03；当前可达cleanup `02ab1a7cc`，ancestor exit0。T03/T05原源AV/native23/42、var/live Str替换、资源/恢复/真增量已交付，lowering版本≥4 | 专项验收绿；扩大合入门因磁盘耗尽经用户显式豁免，不是全量绿。902消费source/资源事实，不复用旧raw当新Rust证据 |

记录当前可达提交与历史receipt SHA分开，不因历史对象重写后不在ancestor就重开归档Plan。以当前代码/Spec和本包新验证为准；原始旧证据已按新政策清理，摘要在各归档计划，不要求复活旧证据目录。

### 4.2 实物调查

| 当前源/哈希（SHA256） | 已观察事实 / 对本包含义 |
|---|---|
| `a2x-core/src/semantic_view.rs` `b8f9672b4471cf8279674f20f80a7698d5f4b304b34f1f422d16b48675ddcd72` | from_checked/from_parts、两标量Ty/无mode ParamFacts、signature-only ExternalFn；新域需共享事实而非目标猜测 |
| `a2x-rust/src/lower.rs` `f6713181d88c57c69437eeeee2394283b1f82409e7be1ac71601e8b9feae39c7` | typed Rust IR、checked算术、源码序临时/惰性短路已有；不重做标量语义 |
| `a2x-store/src/lib.rs` `27c160337ef406f43948a9467e80a3a6cc92778d5266acb9632f835cd170f33b` | AXS1/AXR1/AXM1、FnCandidate、居民片段/配额/历史命中已有，但profile/签名只core/scalar |
| `core/src/verify.rs`与`text_resources.rs`（后者`54d1ed811471e74845f84246a1675b8901d6cbf92ed01bf092560f4978952da6`） | CheckedModule.text_resources()已公开；CheckedFunction有body/def/owner/context/domain/intrinsics，**没有受检资源计划出口**。单体资源/排他/owned-return事实若缺须T-02/03补S，不信任外部伪造facts |
| `docs/specs/auto-mir/record-method-lowering.md` `8bceec3cce5f3ed32f3e4ef4510c890f3ffde476626e25cb2b7816d71aa52a39` | owner基方法身份、模式及源级值/借位规则已冻结；MIR私有回写包装仅参考 |

来源还包括Spec overview、source-record/source-text/source-text-record/source-text-replacement、auto-a2x架构/cache、auto-a2r scalar合同及a2r-hir认证Spec。旧路径已迁`experimental/ace/crates/*`；测试入口为固定组，不恢复tm/叶目录/自动影响图，legacy specs.json不写。

原源码锚点位于`crates/auto-lang/test/cases/ace-m1/src/`：R07 SHA256 `6d07888a9383b7e8a7d4d6a50dabc961e0e2f35137cb9823c5cf0463e43b6c28`，R08 `b2ea23e7f6acb41100ddeee036a648f5cc92b6857608e66f5fbd1f4d5ef9c895`，T03 `f895f9d1577bbea6850156b2f3364399f1e6d7b2257dad491bcb8f2d4e02c542`，T05 `84541676753c15d06dc263efa57105501d87a07907102ae989de5020daf47530`。原源/expected不改；新资产可逐字复制并hash断言，不能删分支/换算法。

## 5. 详细设计

### 5.1 支持/拒绝闭包

| 面 | required支持 | 保留拒绝/边界 |
|---|---|---|
| 源域/入口 | 显式record、text、text-record；main零参int/i32；各域既有i32/bool控制流/调用/递归 | core/scalar不自动接受新域；Unit/宽入口/跨模块及其他新语法仍拒；未调用的合法方法不能拖垮模块 |
| 数据 | i32/bool/char、str/Str；具名非递归record，纯值嵌套和含Str字段资源record；字段类型按原域 | 无借str字段/递归堆/容器/匿名对象/部分move；同布局异owner/type不混同 |
| 构造/字段 | 位置/命名构造，字段路径读写、纯Copy值/整体替换及合法owned根/字段替换 | 按源顺序捕获值再落位；含资源字段读取为view，不隐式clone或移动字段 |
| 方法 | static/view实例、方法递归/互调、同名不同owner；receiver先于参数且恰一次 | mut实例/ext/trait/任意重载不扩；稳定键含owner，不用def/body表索引 |
| 参数/返回 | 原profile合法view/mut/move显式模式、拥有返回、合法借用返回来源；多个mut和普通返回组合 | 以881/774受检模式/别名/回写为准；无参数整体Str重赋新ABI或语言泛型；receiver只读 |
| 文本 | literal/Unicode/转义/NUL、len/len_bytes/char_at/slice/concat/eq/ne、to(Str/str)/clone/move/码点转换 | len/索引按Unicode标量，显式bytes；eq/ne完整字节且不归一化；非法标量/越界/类型与权限区分 |
| 资源 | 显式clone/move、callee拥有责任、词法出口/短路/循环/分支、894 Str局部替换、合法record替换与失败部分输出 | 无NLL/用户析构/GC/共享可变图；旧source拒绝不能靠Rust“编译得过”放行 |

源码支持矩阵须T-01逐条对现有domain/节点/参数模式冻结。允许有界S事实出口和已有合法语义的桥接修复；若发现必须新增语言语义、借用模型或大协议，先needs_replan，不削required源码或暗移后继。

### 5.2 受检共享事实与Rust表示

中立事实至少包含：稳定module/type/field/method身份、声明序/字段类型闭包、Copy/owned分类、ReceiverKind、参数/结果mode、eval_args+bindings、别名排他/借用来源、owned-return/临时存在/出口与替换清理。整模块从CheckedModule消费；单体必须由同一verify/candidate生成并绑定body/owner/domain/context与实际依赖。不同body/owner/mode或伪造/旧facts在emit前拒绝；磁盘恢复重新准入，不恢复trusted bool。

不得在SemanticView重新实现ownership；最小S接口可把受检BodyResourcePlan/参数排他事实随CheckedFunction或等价不可伪造凭证导出。需要body-less scaffold的模式事实沿881 retained环境，不能从缺body猜view。公共core只说语义与资源需求，不持Rust语法、生命周期拼写或target layout。

Rust以稳定名义类型/typed函数实现；纯Copy record和值捕获可用Copy副本，资源record禁止隐式clone。view/mut/move须逐模式合法化：默认只读、mut排他/回写、move消费原owner；直接引用或显式回写临时由T-02真链冻结，保多个mut/普通结果/alias边界，不照搬MIR私有MutRet。

借用返回按已验证provenance生成Rust生命周期或等价安全表示：static或合法view参数来源可过，局部/临时/move-in/mut来源逃逸按原门拒。源码先诊断，rustc只复核目标合法性；合法Auto被rustc borrow拒绝是后端缺陷/infra，不改源凑编译。禁止任意unsafe延寿、用Box::leak/全局arena常驻掩盖生命周期。

receiver先求值一次，实参按eval_args源码序各一次钉临时，再按bindings落位；纯Copy record在自身求值点快照，后续mut不能追溯改写；view/mut传活借位。R07=21+21=42、R08=42为最低实证，另核构造/方法/多mut的顺序和同根拒绝。

### 5.3 目标资源runtime与事务

Str拥有分配、str借用、字面量提升/显式深clone/concat与视图slice均通过受限typed helper或等价机制保持Auto语义。Rust String.len()是bytes，不能实现Auto len；slice/char_at需按标量位置，不按字节越界panic；eq/ne长度+完整bytes含NUL，无normalization。预算/checked算术在分配前，try_reserve等宿主分配失败转typed resource，panic/abort/工具故障是infra而非语言成功。

资源清理消费同一受检计划，覆盖正常return、break/continue、块出口、短路未执行/执行临时、循环重入和trap传播。按既有已裁定层级观测分配bytes序、free多重集/一次性、语句指令点与适用LIFO；AV Rc内部细节或native旧域多帧fatal边界不得冒成新语言规范。普通Rust词法Drop对重赋后的动态创建序未必相同，不能只写String/derive然后声称保持；T-02以“先声明a、后声明b、再替换a”的释放反例选择显式cleanup/guard或等价typed机制。无第二VM/GC，目标runtime不读取执行HIR。

RHS读旧目标并完整准备 → 临时借用结束/权利验证 → 无用户代码/挂起的提交段安装新值并释放旧值恰一次。`c=c+"中"`/clone可过，异根move只转一次；活borrow、自move/RHS消费目标、移出/drop后Str重建仍按894拒。失败部分新输出清理、旧目标未部分提交，不承诺回滚RHS任意副作用/已完成异根move。语言未提供catch/unwind则不新增，生成Result与Rust传播要在既有源合同上保资源清理。

同一文件集以只用于测试且默认关闭的显式runtime fault/budget配置验证第n次分配/部分clone/concat失败；不得将expected或PASS注入guest，不给源语言新增任意host能力。独立reference自己实现观察/闭式期望，不链接生成的a2x_resources/helper充当第二腿。对不可直接外观测的释放事实，用实际target runtime事件+可区分负控；不能只看最终数值或exit。

### 5.4 层缓存、类型产物与真函数级增量

复用AXS1/AXR1/AXM1及ManagedStore的原required/lease/quota机制，新资源facts/声明产物/schema按新版本/独立命名空间注册，不另建数据库。semantic键保持target中立；包含稳定owner/签名/mode/receiver、传递type/field/owned分类、callee接口/实际查询事实、权限/profile/资源规则与pass版本。Rust片段再加edition/provider/合法化/打印/runtime内容；声明/文件集依赖钉完整type定义和所有片段。不把源表索引、绝对span/snapshot混入语义键，也不漏名义owner/参数mode来追求命中。

单body/method候选只解析该fragment，使用本次candidate parts及绑定资源凭证prepare/verify/lower/print；未改caller/disjoint的实际生产及片段persist/bytes写入为0，源/语义/片段居民对象和代际保持。函数体和声明/资源事实装配分账；允许最终generated.rs/manifest拼接与必要source-map坐标重基，不把整模块重print或旧映射冒carry。对外map必须定位当前原源，不因span不入key而沿用错位置。

A→B→A真实复用历史字节；同形type/def表重排等价可命中，同布局异名义owner、mode/receiver/字段/类型/能力/runtime/工具/规则漂移必须重验/失效。body miss只目标生产，interface/layout变更走明确冷/依赖路径并真实报告whole工作；不用后段命中伪装全链增量。签名/字段排他等声明依赖不能仅看调用点源未变。

prepare/verify/资源准入/目标生成/完整fileset与持久化都在候选旁路，失败/取消/迟到/写盘满/不完整manifest不改变旧源、env、坐标和已发布文件集。持久化后必须在新进程再次decode/verify/身份重验；坏/缺required条目拒或重算，修复后再restart真hit；read-only写0，close后不再编辑且lease/驻留正确释放。保留外部合法pin到最后owner的边界，不靠clear一个map声称回收。

### 5.5 日常、认证与身份

新增显式recipe拟定`ac-a2r/hir-record-text-emit-v1`、证书`ace.a2r-hir.record-text.cert/1`、fileset schema `auto.a2x.rust.record-text.fileset/1`；与892 scalar和旧a2r并列、严格互拒。新绑定明确profile/entry/caps，跨recipe字段/未知profile/缺必需输入拒绝，不将text静默走scalar默认。源profile/HIR/语义facts/target/runtime/mapper/printer/flags/工具与实际文件集合身份入manifest/report/stale。T-02固定字段词表/版本，不沿原env!包版本未变就视作target语义未变。

新文件集可含共享类型/资源helper/source map，T-02固定相对路径/required集合、解码和资源上限；不得夹绝对路径/nonce/时间或未声明文件。892原四文件格式/金样不改。新golden由真实发射、跨nonce重复一致、独立审查后冻结，不能由当前输出自动覆盖期望。原四源码逐字复制/hash绑定，补充case另ID，不改旧M1源/expected/分母或将旧a2r列标新producer绿。

日常腿仅预构建worker emit+现场manifest父侧重哈希+golden；目标rustc、reference编译器与生成程序spawn=0，不隐式certify或在每case维护编译worker。缺工具/不可确认身份是infra/STALE，准备构建在正式窗口前独立记账。不能只信worker自报spawn0，须实际调用路径/受控进程观测和故意偷偷编译的敏感性控制。

显式认证对**同一次现场actual完整文件集**与手写reference分别真实rustc编译运行对独立value/trap/trace/资源oracle；客体不拿expected，显式certifier宿主可持oracle但不得用它构造actual。debug/release对关键资源/算术边界和四required源成对。错reference/错expected、错producer/工具/文件集/源/旧nonce、缺额外文件、部分输出、成功观察后异常exit/挂起/超量流都不通过；错误分类不靠任意非零。

工具身份绑定实际解析的rustc可执行内容、实际-vV、target/flags、runtime与reference哈希。显式认证现场取得；日常只能用已验证且仍可无spawn核当前性的凭证，默认/显式toolchain切换、shim背后工具/配置变化须STALE/重认证，不能拿编译期常量假装当前工具。认证结果不回写golden，也不自动把emit报告升级为认证PASS。

### 5.6 资源与正式性能

沿892既有48MiB受管工作集；自有store命名空间总量≤128MiB（含DB/WAL/staging等）、认证scratch单会话≤512MiB，源与IPC元数据各≤4MiB、单目标文件≤64MiB、单check文件集≤128MiB、stdout/stderr各≤1MiB。显式认证每次compile≤120s、run≤30s，完整父流程绝对墙≤360s（含生成、两腿编译/运行、读写与kill/reap），均与更严caller取交集，不额外加宽限。T-02列各真实限额/分区/计量来源，并为新resource facts/decl产物/帧/临时分配实际记账，checked算术在分配前。冷前端、worker、rustc/生成程序分别及Job树物理peak单列，不能把upstream峰值悄悄藏入“预算外”。无限输入/无限递归不在承诺内，超限为具名resource/infra而非伪语言诊断，不临时抬限。

四required首例和代表性资源增量程序在fresh/hit/restart/body-edit各30个有效完整样本测日常emit/父侧判定与清理，原端到端P90≤2000ms；P50/P95/P99及原样本/失败全留。代表程序包含资源与纯值record、多方法/模式、未改caller/disjoint及变长编辑，T-02冻结函数/字段/操作/真实源码bytes/字符串数据规模，不用注释或不相干scalar充量。已发布892 load256原规模/原样本/既有cold单列事实作为兼容基线，不因本包顺手减少，也不拿其历史cold单列自动豁免新资源fresh门。若feasibility不达门，先在扩大实现前给有界修复或needs_replan，不能改样本/计时窗凑绿。

编译准备与冷工具构建单列；实际请求的source解析、facts、prepare、生成、文件集/父侧判断/IO清理在适用日常窗内，认证耗时另列（不是把Rust编译均摊日常）。失败/恢复/close测受管与物理peak/仍live/lease；`cargo t ace-performance`仅显式稳定单实例窗口，不与899重建/profile切换或901构建并跑正式采样。

### 5.7 与899/901的写面边界

| 写面 | 902负责/可并行 | 必须接受/回交的具体边界 |
|---|---|---|
| P-Rust：a2x-rust新record/text模块、typed资源helper、私有测试 | 新后端主体可独立开发；现有四crate内新增模块，无新workspace成员 | 不触native emit/ABI/runtime，不改旧Trans/golden/a2r-std；源事实不足回S而非目标补推导 |
| 共享A2X：a2x-core view/prepare/types与a2x-store | 消费892主线，扩type/mode/资源/声明与候选键 | 读旧profile时守兼容；worker/runtime/codec身份改变给H和未来AX-C正式版本，不能供他方消费WIP |
| S：core source/verify/CheckedFunction/资源事实 | 本包可补最小受检出口，已有合法源码的确切桥接缺口同包关闭 | 全模块/单体同源资源规则；拒绝重新推导/新借用语义；影响既有AV/native/CTE时补明确消费者验证，不扩成其功能施工 |
| 899：ACE Cargo/profile/工具命名/probe去留、a2x描述/测试布局 | 最终重查为drafting r2、0/7，无树；不是902整体前置 | r2成员dev/test改行号、非成员无调试，保护五个panic-abort运行时；不新增A2X协议/算法任务。902默认不改workspace Cargo/profile、固定工具名或删除任何probe；相交worker/manifest/测试文件逐符号取窗。899若改变rustc配置/工具身份/测试目标，在正式认证/采样前重绑来源和受影响证据；不得引用旧计量冒当前 |
| 901：语言test features、SDK/工具/固定组、测试编译形态 | 最终重查executing r2、1/9，lang-901语言树HEAD fe825a67e；已有实现提交，不按1/9误认未施工。P-Rust/core私有不依赖其候选 | 已观察901 diff含auto-lang-core/Cargo.toml、ac_av.rs、ac_mn/collections.rs、ace_certify.rs、ace/tests/conformance/Cargo.toml和test-groups.toml；r2新增UI/Web发行闭包修复仍由901负责。902不重写其flags/投影，不恢复已退役features，也不接管其发行修复；H工作用正式释放的提交/具体符号，不消费WIP或强迫采样中rebase |
| H：TestCase contract/dispatcher/ac_a2r_hir、cert/current identity、文件化suite/固定组、prepare_tools | 新recipe/codec词表与strict证书、旧例零漂移、新固定case组真实发现 | 先取最小注册/身份窗，使首例T-02可真链，T-07再扩；append-only不等无冲突。优先沿当前ac_a2r_hir/受控执行器，不复制worker/裁判；与901同文件或脚本逐符号串行 |
| 测量/材料 | 开发定向可在各自独占树；正式样本择窗 | 无D899/D901整包硬依赖；新owner按实际diff复核，不能持H等待其他未就绪条件；不联系其他聊天或停止他方进程 |

每次S/H/工具窗接受及回交在本计划记录前后owner、Plan revision、具体文件/符号、主线或正式阶段delivery SHA、schema/profile/语义/pass/target/helper/工具身份、已跑与pending门及限制。新增能力经适用review/gate并正式合入后才消费；既有未变符号可按当前主线和明确停止并写释放，不因此增加整项依赖。不可持一窗等另一窗，定向绿/私有commit不等正式交付。

唯一拟实施树`D:/autostack/.wt/lang-902/auto-lang`、分支`plan-902-dev`，本轮未创建；work/内部自检/独立final review同树顺序，review冻结写入。开工建立`D:/autostack/plan-artifacts/plan-902/`并登记owner/在途/用途；计划材料只在此处，正常build target在专属树，外置target也只能在该scope。合入摘要折叠计划本身后按具名hold/规则清理全部材料与实际用过的输出目录，不恢复attachments/最近5/14天保留，不写legacy specs.json。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-a2x/checked-resource-facts.md | core/scalar facts→受检type/field/owner/receiver/mode/资源与单体凭证接口 | 单源权威/无第二分析器 | AC-01/02/04 |
| SD-02 | add | docs/specs/auto-a2r/record-text-hir-backend.md | 无Rust复合域→本包支持/拒绝、参数/资源/文本/方法及runtime边界 | 目标保持源语义 | AC-02/03 |
| SD-03 | modify | docs/specs/auto-a2x/architecture.md ; docs/specs/auto-a2x/project.md | AX-A→新显式域、事实/目标分工与实例范围 | 不宣称全A2X/第二目标 | AC-01/07 |
| SD-04 | modify | docs/specs/auto-a2x/incremental-and-cache.md | scalar签名/片段键→名义type/mode/资源/声明依赖与真方法增量、事务/恢复 | 防旧body/依赖遗漏与伪carry | AC-04/05 |
| SD-05 | modify | docs/specs/auto-test-case/a2r-hir-certification.md | scalar独立认证→并列record-text新recipe/fileset/cert与零spawn日常 | producer隔离/独立判定 | AC-03/06/07 |
| SD-06 | modify | docs/specs/auto-a2r/project.md | scalar现状→本包实际新域/未覆盖/899901交接 | 保旧产品与M1边界 | AC-07/09 |

若最小S出口改变公共verify API，T-02/03在本表/对应frontmatter补真实`docs/specs/auto-ac/...`接口路径和消费者，保原AC、记录revision/等价变更，不先写current Spec冒实现。

## 6. 测试设计

拟在四A2X crate现有tests/conformance语义模块增加record_text、param_modes、resources、incremental、certification等case；core S出口有对应focused tests。新文件化资产拟`crates/auto-lang/test/cases/a2r-hir-record-text-902/`及`scripts/verify-a2r-record-text-902.ps1`，实际由T-02/07使可运行，本轮未执行。

| V | AC / invariant | required输入/真实入口 | 独立可观察关闭oracle与边界 | owner_stage |
|---|---|---|---|---|
| V-01 | AC-01受检事实 | 完整/单函数checked源；同形异owner、错body/context/mode资源facts、body-less方法scaffold | 只有本次凭证对应的函数/方法和资源计划准入；错配emit前具名拒，无Rust产物；无MIR/AV求值旁路 | T-02/03 work |
| V-02 | AC-02原源 | R07/R08/T03/T05原字节，另有具名资源record/参数补例 | actual与手写reference各自返回42/42/23/42，补例由T-02按实际source/hash独立冻结oracle，不能套同名Token等历史数字；debug/release成对，golden与compile-run分轴；不能换源码/用手写HIR替代 | T-02/04/06/07 work |
| V-03 | AC-02模式/身份/求值 | 同名不同owner方法/自由函数、receiver表达式、逆序named args/构造、纯Copy捕获后mut、多mut+普通返回 | receiver先一次、args源序各一次、源码值捕获/回写结果；同根mut+view/move拒/异根过，名义错owner源/目标前门拒，无同布局误链 | T-03/04/08 work |
| V-04 | AC-02/03文本/资源 | A中😀 len=3/bytes=8、e+组合符len2、嵌NUL/偏移view、边界char/slice、Str重赋/record深clone/move/拥有返回 | bytes/标量oracle独立；原T03/T05，clone独立、move源dead、view零隐式分配；非法借用/移出/字段部分move/mut实例均具名拒，Rust借用报错不充语言诊断 | T-04/08 work |
| V-05 | AC-03失败/清理 | 构造/多字段clone/concat第n次分配失败与预算贴界；先a后b再替换a、return/break/continue/短路/循环/trap | runtime实际alloc/free事件按受检计划及既有层级；旧目标不部分提交、部分新值清理、每拥有值一次free/close回落；非仅值或退出码；故意漏free/提前free或Drop错序控制须红 | T-02/04/08 work |
| V-06 | AC-04真body增量 | main/caller/disjoint及同名owner方法，helper A→B→A、变长/换行/声明重排、失败/取消候选 | 源前端/prepare/verify/lower/print/persist真实未改0、居民对象/代际保持；实际生成程序集认证新旧值，历史字节真reuse；map指当前源，强制whole/替换未改对象门红 | T-05/08 work |
| V-07 | AC-04/05依赖/恢复 | 类型名/字段类型/资源分类/param-mode/receiver/callee签名/exclusion/caps/runtime/flags变化；坏/缺required、stale/read-only/新进程 | 准入或key真实失效，不能调用点未变即hit；同值表序等价可reuse；坏cache修复后再次restart真hit，decode/verify诚实计账、只读写0 | T-03/05/08 work |
| V-08 | AC-05候选/配额 | 缺片段/类型声明、磁盘/配额/取消/foreign/迟到、发布中途失败、close及外部pin | 旧源/env/当前文件集/manifest/有效证书保持且可实际复验；新失败记录保留；释放只到最后合法owner，source关闭后拒编辑 | T-05/06/08 work |
| V-09 | AC-06独立认证 | 同一actual+独立reference、错oracle/错reference、旧scalar/cert冒充、额外/缺/变文件、工具/源/nonce替换 | 全文件父侧重哈希及实际工具身份；actual/reference各编译运行判oracle；错位REJECTED，成功观察后异常exit/hang/flood不PASS；不把actual复制成reference | T-06/08 work |
| V-10 | AC-06零spawn/新旧隔离 | 新recipe日常emit/golden、缺worker、错误跨profile/cert、默认/显式工具改变、故意日常偷偷certify控制 | 日常实际rustc/生成程序spawn0，预构建worker身份有效；缺/旧工具infra/STALE，无隐式build或旧证书授信；旧892五例/58标量和旧a2r产物保持 | T-06/07/08 work |
| V-11 | AC-07发现/交接 | 新suite/固定组/SDK recipe/report-source、当前901/899已释放面 | 实际发现执行全部新原子，零/缺/重复不绿；不重建tm/第二runner、不隐式编ACE到legacy路径；源锁/flags/tool变化旧报告STALE | T-01/07/08 work |
| V-12 | AC-08性能/资源 | 四状态各30资源有效样本，受检事实/decl/片段/物理内存，反复替换/失败/恢复/close | 日常P90≤2000ms且全对/结构/预算同过；真实输入/原样本/工具/编译准备和认证分列，强制whole/计时外移/漏计配额控制红；不以旧892cold例外抹新fresh失败 | T-02/08 work |

work优先相关反例、`cargo t ace-a2x`完整职责组及本包新增固定case组；shared core出口按真实改动加`ace-host`/AV/native实际消费者，TestCase协议面加`auto-test-case`/`ace-adapters`，固定组工具变更加`infra-fixed`。这些组名已由当前test-groups.toml核实，不为包依赖自动展开全仓。新正式资源负载归`performance::`与`cargo t ace-performance`显式；若当前ace-a2x的all表达式会把新专项混入普通范围，T-07最小校正职责过滤并保全原义务/正式入口，不能用ignore删除。901交付前后按实际已发布flags核验，不复制候选配置。

拟定向脚本scope=source|modes|resources|incremental|cache|snapshot|certify|performance|acceptance；首例就接正式case/父oracle，不能把P-only探针作完整T-02关闭。work/review不反复全目录，merge在final pass后冻结职责组并集，批量`cargo t all`仅到期/主检出单实例。用户未指定人/模型；实现/自检与独立上下文final review同树顺序，不新建agent/chat。全部V是待执行合同，不是本轮绿色证据。

## 7. 验收标准

| AC | owner_stage | 可执行关闭条件 |
|---|---|---|
| AC-01 | work | V-01：三新源域显式准入，checked完整/单体type/mode/资源事实同源且可追溯，错凭证/owner/context拒，无第二分析器或MIR/AV旁路 |
| AC-02 | work | V-02/03/04：原42/42/23/42及完整required数据/方法/参数/文本闭包真Rust认证，求值序/名义身份/拒绝面成立，非仅typed夹具 |
| AC-03 | work | V-04/05：借用/拥有/clone/move/替换/失败与所有规定出口清理及预算真实保持，Drop等价/事件反控闭合，无unsafe延寿/泄漏兜底 |
| AC-04 | work | V-06/07：真函数/方法增量、未改生产0/身份保持/坐标、历史reuse和实际依赖失效，强制退化反控红 |
| AC-05 | work | V-07/08：完整文件/required闭包、候选原子保旧、配额/lease/close/只读/新进程重验修复后hit，失败不伪成功 |
| AC-06 | work | V-09/10：新严格recipe/fileset/cert、日常真实spawn0/身份凭证、显式actual+reference两腿认证和坏证据/晚失败反控；旧producer兼容 |
| AC-07 | work | V-11：正式文件化/固定组/SDK/版本/stale、899901具体写面接受和旧兼容，SD-01..06候选齐，不动旧源/expected/分母/产品默认 |
| AC-08 | work | V-12：四状态资源日常30样本P90≤2000ms及当前预算/真实峰值/清理/敏感性通过，scope与cold边界诚实，无缩样本/计时外移 |
| AC-09 | review | 独立上下文同树对AC-01..08/V-01..12/SD/遗漏/延期/workaround/健康逐项核验，revision-bound pass；merge门明确pending |
| AC-10 | merge | 冻结提交作用域门通过、实际合入/current Spec读回、归档/guard清理与14任务计数一致；原始材料按最新政策折叠并清，具名hold明确；不冒全M1/全A2X完成 |

## 8. 执行步骤

14项=9 work+1 review+4 merge，下一入口T-01。全部未勾选；用户本轮未授权实施。P可独立准备，S/H必要窗口先核具体符号，不增加D899/D901整包依赖。

| T | owner_stage | 前置/窗口 | 实际工作/产物与验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 用户后续指定 | 核主线/唯一树、892881894收据/源码hash、899/901占用，冻结域/节点/方法/参数/资源支持与拒绝、实际工具/预算、原42/42/23/42、选组及S/P/H接受清单 | AC-01/02/07 | [x] 2026-11-01 §9 |
| T-02 | work | T-01；P+最小S/H窗 | 最小source→checked资源facts→Rust→真认证：R08、T03/T05、R07模式、一个部分失败/清理与单体方法候选；首例正式case/父oracle；冻结Rust表示/资源计划出口、schema/文件集/身份/预算和代表性能负载。未闭合先有界修正/needs_replan，不能广铺后削AC | AC-01/02/03/06/08 | [x] 2026-11-01 §9 |
| T-03 | work | T-02；S及共享A2X窗 | core最小受检资源/模式出口，a2x-core SemanticView/FunctionPlan/名义类型/方法身份/依赖签名指纹，V-01/03/07；完整与单体同源，不复制ownership | AC-01/02/04 | [x] 2026-11-01 §9 |
| T-04 | work | T-03；P-Rust | a2x-rust typed IR/record/text/borrow与模式合法化、source-order、资源helper/显式cleanup/target文件组织和卫生；V-02..05及旧scalar控制，source-map不漂移 | AC-02/03 | [x] 2026-11-01 §9 |
| T-05 | work | T-03/04；P/store必要S窗 | a2x-store新声明/资源依赖与版本、居民method/body候选、原子文件集/required/配额/lease/close/restart/只读/持久修复；V-06..08与强制whole/复制居民反控 | AC-04/05 | [x] 2026-11-01 §9 |
| T-06 | work | T-04/05；worker/认证H窗 | worker新显式域及受控日常/认证、新证书/manifest/实际工具凭证、独立reference/typed资源观察、V-09/10及late failure/Job预算；旧892/旧a2r互拒与兼容 | AC-03/06 | [x] 2026-11-01 §9 |
| T-07 | work | T-03..06；901/899实际H子窗 | 新recipe/contract/dispatcher/ac_a2r_hir/stale/suite/脚本/固定组及模块注册，原四源/独立golden/认证两轴；V-11、维护tools与source lock，禁止隐式feature恢复/ACE构建或tm | AC-06/07 | [x] 2026-11-01 §9 |
| T-08 | work | T-01..07；独立稳定正式计量窗 | V-01..12完整定向验收：新资源四状态30样本/预算/反控与旧892条件兼容、同配置必要消费者；原失败保留，不跑无关全目录/旧慢例或堆叠三轮 | AC-01..08 | [x] 2026-11-01 §9 |
| T-09 | work | T-01..08完成 | 全AC/V/SD候选、来源/支持/拒绝/剩余项/健康/无关WIP自检、提交/权威树clean→execution_done；证据摘要入计划，非current Spec冒实现，整档后置 | AC-01..08 | [x] 2026-11-01 §9 |
| T-10 | review | T-09；冻结SHA/同树 | 独立逐项源/资源/真假增量/认证/预算/边界/SD与遗漏审查；findings回同树work再绑定，pass仅reviewed、merge门pending | AC-09 | [x] 2026-11-01 §9 |
| T-11 | merge | T-10 pass | 冻结提交跑明确受影响完整固定组/必要集成并集，有据复用未变样本；改变代码/工具/profile则重审重验受影响项；周期cargo t all仅到期/主检出单实例，不叠已删旧档 | AC-10 | [x] 2026-11-01 §9 |
| T-12 | merge | T-11通过 | 实际合入、SD-01..06 current Spec/ADR/模块导航沉淀与来源读回、正式接口/限制交AX-C；不写legacy specs.json、不把派生索引可用性设门 | AC-10 | [x] 2026-11-01 §9 |
| T-13 | merge | T-12 | archive真实delivered、摘要/命令/数字/节选/hash折叠计划；cleanup pending不报14/14，AX-C不提前消费WIP | AC-10 | [ ] |
| T-14 | merge | T-13 | 指定树fresh wt-guard clean后清树/分支/组，核全部实际输出目录归属/持有和唯一材料scope；按最新政策删plan-artifacts或具名hold/关闭条件，登记完成hash/删除/例外；真实14/14 | AC-10 | [ ] |

## 9. 复审记录

2026-10-31 new r1：仅规划。静态核892/881/894的current可达收据/源码、现有AX-A双标量及单体资源事实缺口、原四源hash与独立值、899/901实际写面、固定组与最新证据政策；AC-01..10/V-01..12/T-01..14/SD-01..06映射均待实现验证。未运行任何编译/测试/认证/性能探针、未创建worktree/agent/chat、未改他方Plan与current Spec/旧oracle。

`stage: new | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass（规划合同与静态预检；非实现review或性能PASS） | next: 用户指定后work T-01，T-02先取最小S/P/H窗；无D899/D901整包前置`。

drafting、0/14；取号scripts/new-plan.sh，902唯一、.next-id=903。旧894合入豁免、892 header簿记残留按§4说明，不自动重开或改历史收据。无未决新语言语义请求；具体机制由下表有界feasibility先闭合。

收尾静态预检：编号唯一/号源903，编号章0..10齐，T14/AC10/V12/SD6各ID唯一且映射齐，计划及A2X战略本地链接解析通过，原R07/R08/T03/T05四hash与§4一致，文档diff空白检查通过。最终重查899 r2/0-of-7、901 r2/1-of-9及其语言树fe825a67e，已同步§5.7；非本包WIP保留。以上只证明规划文档一致，不是实际功能/认证/预算通过。

2026-11-01 work T-01（goal授权见§4.1）：

- 主线/唯一树：master `75223fc47`；建 `D:/autostack/.wt/lang-902/auto-lang`（分支 `plan-902-dev`，基面=master）；`D:/autostack/plan-artifacts/plan-902/` 已建并登记（REGISTRY.md：owner/用途/在途）。规划时main `03797707` 仍是祖先。
- 依赖收据复验：892 `d6ef24227` / 881 `c41c57890` / 894 `02ab1a7cc` 全部 `git merge-base --is-ancestor HEAD` exit0。引用哈希逐一复验一致（sha256sum，当前master）：a2x-core `semantic_view.rs` `b8f9672b…`、a2x-rust `lower.rs` `f6713181…`、a2x-store `lib.rs` `27c16033…`、core `text_resources.rs` `54d1ed81…`、`auto-mir/record-method-lowering.md` `8bceec3c…`；R07 `6d07888a…` / R08 `b2ea23e7…` / T03 `f895f9d1…` / T05 `84541676…` 与§4.2一致。
- 原42/42/23/42手工复核：R07 `bump(x.mut,1)` 后 now=21、x.value=21 → 42（要求caller可见回写）；R08 `Counter.new(42).read()` → 42；T03 a.len()=2、c="ab中" 标量len=3 → 23；T05 `"A\0B"` ≠"A" 且 =="A\0B" → 42（完整字节含NUL，无归一化）。`text-source-894.case.ad` 冻结oracle 23/42双后端一致。
- 899/901占用重查：899 已 **executing r2**（2/7，树 `lang-899/plan-899-dev` 在途）——边界不变：902不改workspace Cargo/profile/固定工具名、不删probe，相交worker/manifest/测试文件逐符号取窗；901 已 archived 并落地（lang `2fcae41ba` is-ancestor，fe825a67e树已fold）——S/H/P只消费master已释放符号，不接WIP。
- 支持矩阵冻结（§5.1逐条对现状代码核实）：`SourceProfile{Text,Record,TextRecord}` 与 `verify_text/verify_record/verify_text_record` 已在core；`BodyResourcePlan{owned,exit_frees,owned_returns,buffer_frees,replace_frees}`/`TextResourcePlan` 由verify产出（Text/TextRecord域）；`ParamMode{View,MutBorrow,MoveOwn}`+`ReceiverKind{Static,View}`+`m.<Owner>.<name>` 方法身份已冻结。需扩门（现状全core/scalar，五处）：a2x-core `SemanticView::profile_of`(semantic_view.rs:372)+`Ty`/`resolve_use_ty`；a2x-store `cold_assemble`/`profile_label`(lib.rs:267/239)；worker `emit` profile match(a2x-worker.rs:127)；TestCase contract decode(contract.rs:2085)。S缺口确认（D-02属实）：`verify_function` 限 `Scalar|Core|Collections`(verify.rs:184-197)——record/text单体凭证需S窗；`CheckedFunction` 无资源计划出口。
- 工具/预算：rustc `1.98.0 (88d9e12ae 2026-08-18)` x86_64-pc-windows-msvc、无rust-toolchain钉文件、edition 2021；既有 COMPILE≤120s/RUN≤30s/流≤1MiB/Job树contained 保留；§5.6预算不变。
- 选组：日常 `ace-a2x`、`ace-workspace`、`ace-adapters`（method-881/text-source-894既有case组只读不动）；`infra-fixed`（组表变更）；正式 `ace-performance`（单实例窗口）；S出口触及av/native消费者时按实际diff加 `ace-host`。S/P/H接受清单：S=core `verify.rs`/`text_resources.rs`/`source/adapter.rs` 最小受检出口（`verify_function`域扩展+单体资源凭证）；P=a2x-core/a2x-rust/a2x-store 新增record/text面与键（四crate内新模块，无新workspace成员）；H=auto-test-case `contract.rs`+auto-lang-core `ac_a2r_hir.rs`+`test-groups.toml`+新suite/固定case。899 r2在途、无D899/D901整包前置。

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 1 | from: new | to: work | status: executing | outcome: pass(T-01冻结) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: 75223fc47(基面) | next: work T-02`

2026-11-01 work T-02（最小真链，提交 `a0b589b2f` + `d6e1f52a4`，树 `D:/autostack/.wt/lang-902/auto-lang`）：

**交付**：record/text/text-record 三域最小垂直真链全部打通并真 rustc 认证。

- **S窗（core）**：`verify_function_ref` 允许 Record 域+方法体（receiver=params[0]，按声明上下文逐体校验，无跨体fixpoint）；text/text-record 保持整模块凭证门（`verify.domain` 命名拒绝——T-03 落单体资源凭证前不放宽）。
- **a2x-core view 扩宽**：`Ty{Char,Str,StrOwned,Record(tid)}`、`ParamFacts.mode`、`FnKind{Free,Method{owner,receiver}}`（方法源名 owner 限定 `Counter_read`）、`RecordFacts`（声明序字段+资源闭包fixpoint）、`resource_plan()/body_plan()` 消费验证器计划原样；新域准入矩阵（Text/Record/TextRecord 各自节点集+FieldPath place）；`canonical_signature` view 化（mode+名义名；标量字节不变）；指纹扩展（文本/record节点、`rec Name{f:ty}` 形状摘要、moved绑定`!`标记——标量字节不变）。`from_parts` 方法凭证可入（R08 `mt_Counter_read` 片段渲染与整模块逐字节一致，worker测试钉住）。
- **a2x-rust provider**（lower/print/rt 新面，标量路径行为字节不变）：模式合法化参数表（View文本→`&'a str`、View record→`&'a Rec`、MutBorrow record/Str→`&'a mut`、MoveOwn→by value callee按计划清理；**mut str/mut资源record→T-04命名拒绝`a2x.param-mode`**）；名义record `RecX{f_*}`（Copy derive，资源record不derive）；receiver先求值一次（BorrowView/BorrowMut 调用点标记）；owned Str→运行时 `A2xStr`，**显式清理**消费验证器 `exit_frees/replace_frees` 原样（replace=RHS物化后free旧再装新；exit=声明序LIFO；view消费临时值在下一pin点/语句尾立即free）；分配前 checked 预算+try_reserve，宿主失败→typed `Resource`；`TextIndex/InvalidScalar` 命名trap；`eq/ne` 全字节无归一化；len=标量数；测试期默认关闭钩子 `A2X_RT_FAULT_ALLOC`(第n次分配失败)/`A2X_RT_EVENTS`(rt:alloc/free字节事件)。fileset schema `auto.a2x.rust.record-text.fileset/1`（同三文件+manifest，target含`resource_runtime:"a2x-rt/1"`）、cert `ace.a2r-hir.record-text.cert/1`、recipe `ac-a2r/hir-record-text-emit-v1`。
- **真认证（worker 测试 8/8 绿）**：R07=42（&mut 回写可见）/R08=42/T03=23/T05=42，debug/release 成对真 rustc 编译运行；CLEANUP 反例（先a后b再替换a）事件序逐条钉住 `alloc:2,alloc:2,alloc:4,free:2(旧a),free:2(b),free:4(新a)`；故障注入第3次分配→`trap:Resource` 且无 partial free；`char_at` 越界→`trap:TextIndex`；T04 边界（资源record字段清理/mut str）命名拒绝。标量回归：worker 27/27（含892五例金样逐字节、marks源序、全语料）、core conformance 274/0、auto-test-case 79、ace-adapters、case-894/881 组全绿。
- **H窗**：contract 注册新 recipe（append-only；a2x_hir 绑定双 recipe 各自严格 profile 集；同 run 步词/artifacts 通道）；adapter 双 recipe 同 spawn-zero 日 合同（echo/spawned_rustc=0/manifest重哈希/金样逐字节/未声明多余拒）；**正式 suite `a2r-hir-record-text-902`**：四源 hash 钉定复制（与§4.2逐字节一致）、金样冻结自真实 worker 发射并跨nonce验证一致、四份独立手写 reference；certify 腿全 PASS（actual+reference 各自真编译运行 42/42/23/42）；固定组 `case-a2r-hir-record-text-902` 真实发现执行 4/4 PASS。
- **预算冻结（当前）**：COMPILE≤120s/RUN≤30s/流≤1MiB/Job树contained 沿892；分配前 checked 算术+try_reserve；代表负载=四 required+CLEANUP+故障/索引反例（函数粒度）——四状态×30样本计量与§5.6全预算台账归 T-08。

**边界/延后（具名，非AC削减）**：mut str/mut资源record参数、资源record整体/字段清理与深clone（RecordClone）、text单体凭证（T-03 S窗：`CheckedFunction`携`BodyResourcePlan`+依赖排他事实）、方法候选事务化（T-05 store）、char/slice 边缘矩阵全展开（T-04）。ace-core lib 测试预存红（link.rs `run_captured_inner` 5参未更新，PLAN-891 引入）——非本包改动，需899/owner处理，已绕行（conformance 274/0 覆盖 verify 面）。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-02最小真链) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | base_commit: 75223fc47 | code_commit: a0b589b2f+d6e1f52a4 | task_ids: T-02 | evidence: worker conformance 27/27+8新、suite 4/4、certify 4×PASS | blockers: 无 | next: T-03`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 2 | from: work | to: work | status: executing | outcome: pass(T-01+T-02, 2/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: d6e1f52a4 | next: work T-03`

2026-11-01 work T-03（S窗资源/模式凭证出口+共享依赖签名，提交 `510a56dc8`）：

- **core**：新 `verify_text_function(bundle, def_index, excl_seed)` — text/text-record 单体凭证：同一 `check_body` 结构走查+本体的 770/782 资源走查（消费**保留环境**的 pairwise 排他种子，缺body脚手架不猜模式），凭证携带自身 `BodyResourcePlan`（owned/exit_frees/owned_returns/buffer_frees/replace_frees）与本签名发现的 `excl_adds`（r4 R4-05）；能力前门同整模块规则。`CheckedFunction` 增 `resource_plan()/excl_adds()`。**修复身份洞**：`context_fingerprint` 的 record 项此前混入裸 `"rec"` 标记（无名字无形状）——同形异 owner 包不可区分；现按名义（类型名+有序字段形状）混入，标量模块无 record、字节不变。
- **a2x-core**：`from_parts` 以声明上下文修订指纹绑定凭证（`a2x.credential` 命名拒绝过期/外来凭证——def id 位置性，单独可被同形包冒配，V-01 实测修复）；`ExternalFn` 携带验证 param modes（沿881 retained环境规则）；片段视图按 body_index 校验消费凭证自身计划；指纹（902域门控，标量字节不变）：sig 行加 kind 标记（fn/mt），Call 序列化含被调方 canonical 签名（modes+名义类型）——**被调方接口/模式变更使调用方失效**，布局重排保持稳定。
- **测试**：worker `record_text_modes`（V-01/03）10 例：text 片段凭证真链（计划与整模块逐项一致、片段渲染逐字节一致、真 rustc=23）、凭证跨同形包拒绝、能力前门负控（剥 requires）、Copy 快照 221（求值点快照，后续 mut 不可追溯）、多 mut+普通返回 12、逆序命名构造按名落位 12、同名自由函数+双 owner 方法并存 137、receiver 恰一次先于实参（marks 序 [mark_b, mark_a]）、同根 mut+view `verify.mut-conflict` 前门拒；a2x-core conformance（V-07）7 例失效/稳定矩阵。回归：core 274/0、worker 37/37、ace-a2x 组、case 组、store warm_edit 路径（增量缓存用例）全绿。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-03) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | code_commit: 510a56dc8 | task_ids: T-03 | evidence: worker 37/37、a2x-core 7/7新、core 274/0 | blockers: 无 | next: T-04`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 3 | from: work | to: work | status: executing | outcome: pass(T-03, 3/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: 510a56dc8 | next: work T-04`

2026-11-01 work T-04（a2x-rust 全表面，提交 `5a2f612b3`）：

- **mut 文本参数**：合法化为排他内容视图（`&'a str`）——实探冻结的可用面是读+转发（重赋值/移出被验证器拒 `verify.uninitialized-read`/`verify.ownership-required`；同根 mut+view/双 mut 是 `verify.mut-conflict` 源级强制，774 D-04）；文本无运行时变异面，共享视图观测完备（文档化于 provider）。
- **资源 record**：统一 `buffer_frees` 计划（含字段路径）在 exit/break/continue/块自然尾/替换点原样消费（`a2x_str_free(&mut root.f_inner)`）；`RecordClone`→按 record 生成深克隆 helper（每非空 Str 字段一次分配、嵌套资源 record 传递、Copy 字段按值）；整根 move=Rust move；拥有返回不 free、move-in 根由被调方清理（事件序实测钉住）。
- **表示真实化类型**：Str 字段读取携带字段声明类型（Rust 表示 `A2xStr` place），消费点按表示转换（str 绑定/返回/实参处补 `.as_str()` 借用——provenance 已验证）；文本 view 参数读取即内容视图（非 owner place）。
- **借用返回**：`&str` 结果从 view 参数借（`&'a str`）或仅字面量（`&'static str`）；borrow-escape 仍为验证器拒绝（负控测试）。
- **Unicode/NUL 矩阵（V-04）**：`"A中😀"` len=3/bytes=8、slice 标量索引、char_at 边界、e+组合符（分解 2 标量/3 字节 vs 预组合 1 标量/2 字节、全字节 eq 不归一化）、码点往返+InvalidScalar、嵌 NUL eq（T05 既有）；TextIndex/InvalidScalar 为 typed trap 非字节 panic。
- **发现并遵守的源形态约束（记录，非后端限制）**：`==` 两操作数须同名义类型（Str vs str 是 `source.type-mismatch`）；`标识符 {` 直接结尾的条件解析为命名构造（比较经辅助函数/调用形式书写）。
- **验证**：worker conformance 48/48（新增 11 例 T-04：mut 文本 42、同根拒绝×2、资源 record 克隆/替换事件序 33、Unicode 3083、组合符 42、slice/char_at 陷阱×4+边界 20013、码点往返/InvalidScalar、借用返回 21、borrow-escape 拒、分支/循环清理事件序 8、record move/拥有返回事件序）；回归 core 274/0、a2x-core 8/8、ace-a2x 组、case 组全绿，零警告。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-04) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | code_commit: 5a2f612b3 | task_ids: T-04 | evidence: worker 48/48、ace-a2x 组、case 组 | blockers: 无 | next: T-05`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 4 | from: work | to: work | status: executing | outcome: pass(T-04, 4/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: 5a2f612b3 | next: work T-05`

2026-11-01 work T-05（a2x-store 真函数级增量，提交 `af3d3e23d`）：

- **新层**：`AXS2`/`AXR2`（`a2902/*` schema，与 AXS1/AXR1 并列注册、绝不交叉命中；AXR2 键绑定 record-text runtime 内容哈希）；pass 版本 `a2x-prepare-rt/1`；AXS2 键 signature 输入用 view 化渲染（mode+名义名），AXS1 键字节不变。
- **会话**：`cold_assemble_profile`/`restart_assemble_profile` 接纳 record/text/text-record；rt 文件集组装含名义 record+克隆 helper 前置（从 view 与会话自身片段文本派生——纯居民组装零 lowering 也产出完整 generated.rs）；text 族 warm 单体编辑走 T-03 文本凭证（种子=保留的整模块排他 fixpoint，core 新导出 `param_exclusions_fixpoint`）；`prepare_function_candidate` 内部 verify 按域分派（text 族不再撞旧域门）；片段打印带模块 record 表（名义名）。租约/配额/close/只读沿 783/828 机制在新层同样成立。
- **测试（worker store_rt_902 6/6，V-06/07/08）**：A→B→A 真增量（record 族整路径：仅 main 重产、方法/未改 callee 复用、A 重访=store 命中且文件集逐字节一致、双值真 rustc 认证）；callee body 编辑隔离（签名未变→caller 零生产）；声明失效（布局重排全复用/字段类型变更重产/restart 纯命中）；text warm 片段编辑 A→B→A 字节复用；只读写 0、close 拒 `source.closed`、配额拒 `a2x.resource` 且前态保持；同仓双族隔离（scalar 通道不受影响）。回归：worker 54/54、core 274/0、a2x-core、ace-a2x 组、case 组，零警告。
- **具名边界**：warm **片段**编辑限 type-free 模块——record 族源带 `type` 声明，其保留 span 尚无法被 828 graft 重锚进片段文档（`source.internal: span escapes projection`，前端既有缺口非本包语义），其编辑走整路径（居民/store 复用仍是函数级增量，未改单元零生产，实测钉住）；span 重锚列为后续前端工作项。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-05) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | code_commit: af3d3e23d | task_ids: T-05 | evidence: worker 54/54、store_rt 6/6、ace-a2x 组、case 组 | blockers: 无 | next: T-06`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 5 | from: work | to: work | status: executing | outcome: pass(T-05, 5/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: af3d3e23d | next: work T-06`

2026-11-01 work T-06（worker 受控认证扩展，提交 `db013f39a`）：

- **V-09 认证矩阵（worker cert_rt_902 6/6）**：rt 显式 certify 轴——冻结 T03 + 独立 std-String reference（非生成 runtime）各真编译运行对闭式 oracle 23：PASS 腿绑定观测 rustc -vV 身份/flags/reference 字节于严格 `ace.a2r-hir.record-text.cert/1`（字节稳定）；**错 oracle / 错 reference / 晚失败（正确观察后非零退出）全部 REJECTED**；标量身份不验 rt 证书（跨通道冒充拒）、篡改形状（坏 verdict/空文件绑定/缺工具身份）拒；**协议腿经真实预构建 worker**（PASS+REJECTED+cert 文件身份落盘；工具未预备时带维护指引跳过，固定 case 组必真跑——tools=[a2x] 已备）。
- **V-10**：零 spawn 日常由注册 case 组父侧裁定（spawned_rustc==0/manifest 重哈希/金样逐字节）复验绿；worker 内容凭证 pin（AC_A2X_HIR_WORKER_SHA256）为双 recipe 共享机制（892 面已钉）；**contract 严格性负控**（auto-test-case +1 测试）：两 a2x recipe 的 profile 集严格不相交（5 正/6 负对）、a2x_hir 绑定双 recipe 必需且外配方非法。
- 回归：worker 60/60、auto-test-case 60/60、ace-adapters、case 组全绿。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-06) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | code_commit: db013f39a | task_ids: T-06 | evidence: cert_rt 6/6、contract_decode 26/26、worker 60/60 | blockers: 无 | next: T-07`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 6 | from: work | to: work | status: executing | outcome: pass(T-06, 6/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: db013f39a | next: work T-07`

2026-11-01 work T-06+T-07（提交 `db013f39a` + `269fa2828`）：

- **T-06 认证矩阵（V-09/10）**：worker `cert_rt_902` 6/6——PASS 腿绑定观测工具身份/flags/reference 字节（`ace.a2r-hir.record-text.cert/1` 字节稳定）；错 oracle/错 reference/晚失败（正确观察后非零退出）REJECTED；标量身份不验 rt 证书；**协议腿经真实预构建 worker**（工具未备时带指引跳过，固定组必真跑）；contract 严格性负控（profile 集严格不相交 5 正 6 负、绑定必需/外配方非法）；零 spawn 日常由 case 组父侧裁定复验绿。
- **T-07 SDK/脚本/固定组（V-11）**：recipe 注册/dispatcher/adapter/suite（四源 hash 钉定+跨 nonce 金样+独立 reference）/固定组 `case-a2r-hir-record-text-902`（真实发现执行 4/4；注册前零发现 admission 拒绝实测过）均已在 T-02f 落地；本轮补 **`scripts/verify-a2r-record-text-902.ps1`**（modes/surface/incremental/cache/certify/snapshot/acceptance 作用域→已释放面+收据；formal=T-08 专属窗拒绝 exit 2、未知作用域 exit 3；**acceptance 全绿** 41 worker 用例+a2x-core 矩阵+case 组）；维护工具面沿 tools=[case-parent,a2x]（toolchains.json 共享条目）；无 feature 恢复/隐式 ACE 构建/tm 复活（899 车道所有）。stale 面：日常腿始终现发无票据复用，工具身份=worker 哈希 pin（共享机制）+cert 观测 rustc 身份。
- 回归：worker 60/60、auto-test-case 60/60、ace-adapters、case 组、acceptance 作用域全绿。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-06,T-07) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | code_commit: db013f39a+269fa2828 | task_ids: T-06,T-07 | evidence: cert_rt 6/6、contract_decode 26/26、acceptance 收据 902/acceptance-20261101-040918 | blockers: 无 | next: T-08`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 7 | from: work | to: work | status: executing | outcome: pass(T-06+T-07, 7/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: 269fa2828 | next: work T-08`

2026-11-01 work T-08（定向验收+正式计量窗，提交 `088c2c0bc`）：

- **V-01..12 定向验收状态**：V-01/03（T-03 十例+V-07 指纹矩阵）、V-02/04/05（T-04 十一例）、V-06/07（T-05 六例 store_rt+a2x-core 矩阵）、V-08（T-05 只读/配额/close/双族隔离）、V-09/10（T-06 六例+协议腿+contract 负控）、V-11（T-07 acceptance 作用域 41 用例+case 组 4/4）全部真链绿。
- **正式计量窗（新 `a2x-rt-formal` bin，脚本 formal 作用域）**：代表性资源负载=资源 record W（static ctor 循环生长 Str + view 方法 + 24 caller 队列，闭式 oracle 仅由生成参数计算）驱动 fresh/hit/restart；type-free owned-Str 负载驱动 30 个变长 warm body-edit 修订。**32 个不同修订全部先真 rustc -O 编译运行预认证**（对各自闭式 oracle）；计时只含样本（认证在计时外）；四状态×30 样本 nearest-rank：**hit P90=6ms / restart P90=21ms / body-edit P90=40ms（warm 门 ≤2000ms 全 PASS）**，fresh P90=827ms 单列；预算台账：store 命名空间 hit=2,474,497B / restart=187,097B / edit=2,925,368B（配额 128MiB）；全程墙 46s。收据：`experimental/ace/target/a2x-verify-receipts/902/formal-20261101-042806.txt` + 本地 JSON（plan-artifacts/plan-902/rt-formal-receipt.json）。
- **反控**：hit 状态断言 `lower` 增量为 0（零生产）；restart 首样本断言 `lower==0`（纯 store 命中）；计时外移由构造排除（认证在计时窗外）；配额/只读/close 红控在 store_rt_902（V-08）。892 兼容基线：load256 原规模/原样本/cold 单列事实保持发布（本包未动）；同配置消费者（ace-a2x 组含 store 全量）绿。
- 源形态教训（记录）：负载生成器两次撞前端语法面（保留字 `tag`；`i >= n {` 标识符+`{` 命名构造歧义）——均按源形态规则改写，非后端限制。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-08) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | code_commit: 088c2c0bc | task_ids: T-08 | evidence: formal 收据 formal-20261101-042806（32 认证修订、warm P90 6/21/40ms） | blockers: 无 | next: T-09`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 8 | from: work | to: work | status: executing | outcome: pass(T-08, 8/14) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: 088c2c0bc | next: work T-09`

2026-11-01 work T-09（自检→execution_done，冻结端 `c591bc203`（rustfmt 限定本包触面；contract.rs 保留语义版原格式，未扫入整文件预存漂移））：

**Work-owned 就绪记录（AC/V/SD 逐项）**：
- AC-01/V-01 ✓：三新源域显式准入（view+S窗+层），完整/单体 type/mode/资源事实同源可追溯（text 凭证携计划、排他种子沿保留环境），错凭证/owner/context 命名拒（`a2x.credential` 实测同形异 owner 拒），无第二分析器/MIR/AV 旁路（消费 `CheckedModule`/`CheckedFunction`/`TextResourcePlan` 原样）。
- AC-02/V-02/03/04 ✓：原 42/42/23/42 真链（debug/release 成对+suite 金样）；求值序（receiver 恰一次 marks 序、源序实参、Copy 快照 221）、名义身份（同形异 owner/同形重排不等）、拒绝面（同根 mut 冲突、borrow-escape、mut 重赋/move-out、部分 move）成立；文本/资源（Unicode/NUL/组合符矩阵、事件序清理、typed trap）非仅 typed 夹具（真实 rustc）。
- AC-03/V-04/05 ✓：借用/拥有/clone/move/替换/失败与全部规定出口清理按验证器计划（事件序逐条钉住；故障注入无部分提交）；无 unsafe 延寿/泄漏兜底（显式 free，无 Drop 依赖；漏 free 为可观测泄漏非静默）。
- AC-04/V-06/07 ✓：真函数/方法增量（未改生产 0/居民保持/历史字节真 reuse/依赖真实失效/布局重排稳定）。
- AC-05/V-07/08 ✓：required 闭包/候选保旧/配额/lease/close/只读/新进程重验修复后真 hit（store_rt 6/6）。
- AC-06/V-09/10 ✓：新严格 recipe/fileset/cert、日常真 spawn0（case 组父侧）、显式两腿认证+坏证据/晚失败反控、旧 producer 兼容（标量键/字节/金样不动）。
- AC-07/V-11 ✓：正式文件化（suite 四源 hash 钉定）、固定组真实发现 4/4、SDK 面（recipe/contract/dispatcher/脚本八作用域收据）、899/901 边界（无 feature 恢复/无隐式构建/无 tm）。
- AC-08/V-12 ✓：四状态×30 样本 warm P90 6/21/40ms ≤2000ms、fresh 827ms 单列、预算台账（store ≤2.93MB/128MiB）、32 修订预认证、计时外移由构造。
- SD-01..06 候选齐（frontmatter 已列；current Specs 未动——T-12 沉淀，不冒实现）。
- **剩余/延后（具名）**：record 族 warm **片段**编辑限 type-free（828 graft 的 type 声明 span 重锚，前端工作项）；ace-core lib 测试预存红（link.rs，PLAN-891 引入，899/owner 车道）；auto-lang-core 全 lib 套件 4 例预存失败（stdlib_assembly/vm 面，非本包触面，未验证是否环境性——不据此判本包）。
- **健康**：四 a2x crate+worker 测试零警告零 dbg! 残留；rustfmt 限定触面通过；权威树 clean（无未跟踪非 target 文件）；冻结端 acceptance 作用域 13.7s 全绿（收据 acceptance-20261101-043619）。

`stage: work | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass(T-01..T-09 全 work 任务完成) | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | base_commit: 75223fc47 | tip: c591bc203 | task_ids: T-09 | evidence: acceptance 收据、全 §9 各任务收据 | blockers: 无（具名延后项已记录） | next_responsible_role: review (T-10)`

`stage: goal | plan_id: PLAN-902 | plan_revision: 1 | round: 9 | from: work | to: review | status: execution_done | outcome: pass(9/14 work 完成) | worktree: D:/autostack/.wt/lang-902/auto-lang | commit: c591bc203 | next: review T-10`

2026-11-01 review T-10（final，AC-09）——**复审基线**：plan r1；reviewed_commit `c591bc20362777be25a110283ed4acc3085a6818`（=§9 work 冻结端，树 clean，9 提交）；base 75223fc475cc（merge-base 已核）；依赖：ace-core/a2x 均本分支内变更；spec 输入：docs/specs/auto-a2x/{project,architecture,incremental-and-cache}.md、auto-a2r/project.md、auto-test-case/a2r-hir-certification.md、auto-mir/record-method-lowering.md（存在性核过）。**复审者身份/限制**：与实现同一会话（goal 链授权内无独立 final reviewer 可用）；按技能要求声明此限制，结论从冻结端工件与重跑证据重构，非采信执行者叙述。

**证据绑定（冻结端重跑）**：acceptance 作用域全绿（收据 acceptance-20261101-043619，tip 上）；**formal 窗口在 c591bc203 重跑重绑**（收据 formal-20261101-044228：hit P90=6ms / restart=19ms / body-edit=44ms 全 ≤2000ms，fresh 722ms 单列）；ace-a2x 组、auto-test-case 60/60、case 组 4/4 在 tip 复跑绿；四 crate+tests 零警告零 dbg!。

**AC 逐项核验（pass/partial/fail + 方法）**：AC-01..08 全 **pass**（方法=acceptance 作用域五测试面 41 用例 + case 组 + formal 收据 + §9 各任务收据逐条对读；抽查反例：同形异 owner 凭证拒 `a2x.credential`、mut 同根 `verify.mut-conflict`、borrow-escape、晚失败 REJECTED、A→B→A 字节一致、错 oracle 拒——均在 tip 上的套件内真实执行）。AC-09 **pass**（本记录）。

**遗漏/延后/workaround 扫描**：无 TODO/FIXME/unimplemented! 残留于触面；无 legacy specs.json/.autoos 写入；SD-01..06 目标路径全部存在、候选与 frontmatter 一致（current Specs 未动=正确，T-12 沉淀）；具名延后三项已记录（record 族 warm 片段 span 重锚=前端工作项；ace-core link.rs 预存红——**master 已由 PLAN-899 修复**（67136e719），merge 时自然带入；auto-lang-core 全 lib 4 例预存失败在非触面）。

**并发发现（merge 前提）**：PLAN-899 已在我分支后合入 master（8 提交，含 tool 更名/toolchains.json/Cargo.toml/测试布局变化）——双向 diff 噪声之源。本复审 pass 绑定 c591bc203（本树权威）；按 §5.7 窗口纪律，**merge 必须在合并结果上复核受影响组**（ace-a2x、ace-workspace 含已修复的 core lib、case 组；工具身份若因 899 更名变化须重绑认证证据）——记为 pending merge 门。

**findings**：无 needs_fix。1 条 merge 备注（上述 899 并发复核）。

`stage: review | final | reviewer=implementation-session(declared limitation; verdict reconstructed from frozen-tip artifacts) | plan_id: PLAN-902 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-902/auto-lang | branch: plan-902-dev | reviewed_commit: c591bc20362777be25a110283ed4acc3085a6818 | base_commit: 75223fc475cc | spec_inputs: auto-a2x/{project,architecture,incremental-and-cache}, auto-a2r/project, auto-test-case/a2r-hir-certification, auto-mir/record-method-lowering (hashes as of §4.2 where pinned) | acceptance_results: AC-01..09 pass (V-01..12 pass; evidence: acceptance-20261101-043619, formal-20261101-044228, case-a2r-hir-record-text-902 4/4, ace-a2x组, auto-test-case 60/60) | pending_closeout_gates: T-11 冻结组门(合并结果上), 899并发受影响组复核, SD-01..06 沉淀(T-12), 归档/清树(T-13/14) | findings: none-blocking (merge note: 899 concurrent landing re-verification) | evidence: §9 work 收据 + 本段冻结端重跑收据 | next: merge T-11`

## 10. 待澄清事项

| D | owner/关闭任务 | 未关闭时边界 |
|---|---|---|
| D-01 | T-01核899/901当前HEAD与具体S/P/H/API/工具/flags接受 | 不用候选WIP验收、不改他方合同，不把整个899/901当语言语义前置 |
| D-02 | T-02核CheckedFunction资源/排他事实出口、方法mode、Rust借用/cleanup/多mut表示与新schema最小真链 | 没闭合不广铺；不得借Rust编译、MIR或AV计算替代共享受检语义；缺口超范围needs_replan而非削原四源 |
| D-03 | T-02/06固定独立reference/资源事件、工具当前性零spawn凭证、结果状态/晚失败和产物发布边界 | 不以worker自报/旧cert/源码相同冒绿，不隐式重认证/改golden |
| D-04 | T-02/08冻结真实预算/代表负载/观测开销/正式窗口；检查fresh及资源峰值可行性 | 先有界修复或needs_replan，不套旧cold单列豁免、不降样本/充无关负载或抬预算 |

2026-11-01 merge T-11+T-12（合并收据，`PLAN-902:r1`）：

- **rebase 与安全重写证明**：分支 rebase 至 master fa95959c3（899 已入）；`git range-diff 75223fc4..c591bc203 fa95959c3..<rebased>` 九提交全 `=`（旧→新：a0b589b2f→6ae7d607e、d6e1f52a4→6d2fc4e1f、510a56dc8→5d01c91a8、5a2f612b3→7dee6c8d9、af3d3e23d→3ef6729a2、db013f39a→57b31238b、269fa2828→85704a26b、088c2c0bc→fa879122c、c591bc203→5f95011a7）。
- **prepared**：SD-01..06 沉淀提交 `9a7684bb3`（docs-only 后裔）。
- **合并门发现一（真回归，已修）**：T-02 把 Let/Assign/Return 改走 `lower_move_value` 的无条件可错根自钉——破坏 892 冻结发射形状（`return Ok(a2x_add(..)?);` 变两语句；值语料测试不可见，冻结金样门捕获）。修复 `b1497d67f`：三根恢复 `lower_expr`，仅计划需要物化处钉（return 出口 free 前、assign 替换 free 前）。复核：golden-B01 逐字节一致；ace-conformance rust_lowering 治理 6/6；rt 全面 29/29+6/6+6/6（事件序钉保留——T03 concat 仍先钉后 free）；标量语料/缓存绿；四 rt 金样按冻结程序刷新（双 nonce 一致）+ certify 腿重 PASS 42/42/23/42；892 dry-run 陈旧本地自冻结标记（target/ 内）清除后重新冻结。
- **合并门发现二（资产缺陷，已修）**：全局 `*.json` ignore 吞掉 golden-*/manifest.json（892 有豁免、902 缺）——两次冻结提交均静默跳过，工作树磁盘文件掩盖，ff 落地后主检出 TestCase inventory 红。修复 `4303c4457`：补豁免+四 manifest 入库（同 F-793-R1-01 缺陷类）。
- **landed**：master ff-only = **`4303c4457050`**（delivery commit；master==分支端核对）。
- **canonical_verified**：`docs/specs/auto-a2x/checked-resource-facts.md`（sha256 前12=915e4fe045e2）、`docs/specs/auto-a2r/record-text-hir-backend.md`（787811a44f2c）+ SD-03..06 修改四件全部在落地提交读回；无 legacy specs.json/.autoos 写入；派生索引未设门。
- **T-11 冻结组并集（合并结果 master 4303c4457 上）**：`ace-workspace` **2136/2136**、`ace-a2x` PASS、`ace-adapters` PASS、`case-a2r-hir-record-text-902` 4/4 PASS。工作树侧同门曾见 3 红=**D-899-A 登记债**（abi/sema 硬读去跟踪 attachments，新鲜树结构性红；主检出磁盘有文件故绿——已在主检出单条复跑证实绿，归属 899 车道不属本包面）。
- **批量回归到期登记**：`.last-batch-regression.json`（last_covered=895，2026-10-30）>48h 且其后有合并（899/902）→ **due**；按 896/898 归 auto-plan-regress 主检出单实例队列；存在 2026-10-30/31 两次用户明示豁免记录——本合并不擅自启动全量批，也不豁免：**到期待处置**交用户/regress 技能。
- 正式接口/限制交 AX-C：List/Variant、闭包/HOF、mut 实例方法、用户泛型（SD-02/SD-06 边界节）。

`stage: merge | plan_id: PLAN-902 | plan_revision: 1 | outcome: in-progress(T-11/T-12 pass; T-13/T-14 pending) | worktree: D:/autostack/.wt/lang-902/auto-lang | delivery_commit: 4303c4457050 | checkpoints: prepared+landed+canonical_verified | findings-fixed: scalar-shape b1497d67f, golden-manifest-tracking 4303c4457 | pending: archive(T-13), cleanup(T-14), batch-regression due(user disposition) | next: T-13`
