---
plan_id: PLAN-999
status: archived
feature_name: ACE M1残余正确性与准入收口
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-ac/source-core-i32.md, docs/specs/auto-av/cache-and-conformance.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-syntax/source-frontdoor.md, docs/specs/auto-semantics/source-query-session.md, docs/specs/auto-test-case/ace-source-only-replay.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-hir, auto-syntax, auto-semantics, auto-av, auto-ac, auto-a2x, auto-test-case]
depends_on: [PLAN-929, PLAN-934, PLAN-950, PLAN-998]
current_step: 13
total_steps: 13
---

# [PLAN-999] ACE M1残余正确性与准入收口

## 0. 变更摘要

在原M1范围内关闭五份原源对应的八格失败、F-941-13签名坐标污染，以及D-950-1当前仍可达的旧事实页重复发布问题。复用统一源码前门、事实查询、共享MIR与ManagedStore，交付真实反例/合法近邻、消费者与版本闭包、原预算下增量证据及941接收材料。

**本轮仅规划**：drafting r1、0/13。只写本计划与中央取号簿记，不施工/构建/测试，不建worktree或材料scope，不修改941计划/其工作树/current Specs，不创建、接管或联系其他会话。后续交付本包不等于M1签章，最终组合判定归941原owner或明确的后继验收责任。

## 1. 目标

1. 原八格经真实AV/native正式recipe通过；source/expected/oracle不改，旧case/check有一对一映射，不以新程序或手工MIR代原格。
2. F-941-13连续跨owner编辑不因来源span漂移误报签名变化；真签名/mode/名义类型变化仍失效，源码位置与诊断仍准确。
3. 当前可达的`QueryHost<ManagedStoreSink>`重复发布同语义事实不因请求revision变化发生Conflict；真语义/依赖变化、同键异内容、迟到/取消、旧页与坏恢复保持正确分类。
4. 向941交付源/工具/协议身份、逐格关闭表、版本/证据失效清单和预算/增量收据，由其原owner接收重读；不直接改941在途文件。

范围仅auto-lang仓，触面限§3和§5列出的共享前门/事实查询/目录签名/页存储及真实消费者、测试与固定组。属于既有架构中的L1有界修复。

非目标：文件I/O、公共标准库扩面、通用闭包/模块global cell、PortableTy文本/record扩面、旧A2R嵌套捕获、完整JIT/block增量、自举、M2产品迁移、全仓警告与无关治理债。313 required、批准/NA/accepted/future、原数值/trap语义、样本/规模/资源/栈/截止/P90门均不变；不靠改expected、缩源、skip或fallback凑绿。

## 2. 架构方案

Core/Scalar共享域准入 → 统一CST/AST → 同一source-query事实通道 → 对应profile验证得到Checked HIR → 唯一共享MIR → AV/native。A2R/C继续直接消费受检HIR，作为共同语义及增量读者回归，不改走MIR/AV。

目录签名只表达非坐标语义，来源图单独更新；持久事实与活请求revision分离，复用同一ManagedStore并版本化地址/解释，不重解释旧bare S934或`sq-module-page/1`。T-02先证明整链可行性；需要新架构/改变语言承诺或预算时needs_replan，不把探针成功当完整交付。

## 3. 技术栈

| 现存路径 / 符号 | 当前职责 / 已观察事实 | 本包拟触面 |
|---|---|---|
| `experimental/ace/crates/av/src/query_entry.rs`：`compile_scalar_source_product`、`compile_scalar_source_via_facts` | 普通scalar事实入口 | 拟新增Core兼容共享选择门，保显式Scalar旧义；名称T-02冻结 |
| `experimental/ace/crates/source-query/src/consumer.rs`：`compile_scalar_product`、`compile_profile_product`、`bind_with_facts` | 产品成功分支目前接Scalar/TextRecord，缺Core验证分支 | 接同一事实producer与Core verifier，不转回旧类型推导 |
| `experimental/ace/crates/av/src/driver.rs`：`Driver::request_inner`、`front_door_admit` | scalar请求和128深度代理门 | 正式source域与真实统一前门预算；Atom入口另保原合同 |
| `experimental/ace/crates/native/src/driver.rs`：`compile_escalating`、`Domain` | Scalar→Record→TextRecord→Numeric | 协调Core准入与profile传递，保原域优先级/终态诊断 |
| `experimental/ace/crates/core/src/source/cst/units.rs`：`signature_of`、`UnitCatalog::{signature_snapshot,edit_owner}`、manifest | 签名、声明事件、持久目录 | span-free签名与全读者/恢复/版本闭包 |
| `experimental/ace/crates/source-query/src/{host,session,store,keys,facts}.rs`：`QueryHost::{prepare,commit}`、`ManagedStoreSink::{put,get_by_key,get_checked}` | 生命周期revision、bare S934持久页 | 重复发布与完整依赖/旧页读取合同修复 |
| `experimental/ace/crates/source-query/src/modules/persistence.rs` | 模块页`sq-module-page/1`，revision=0且read-versions入键 | 保持此独立地址/缓存合同，不能借模块绿核销bare路径 |
| `experimental/ace/crates/core/tests/{conformance.rs,cases/source_units_928_t09.rs}`、`source-query/src/tests.rs` | 正式目录与页读写/恢复测试 | 拟新增`core/tests/cases/m1_residual_999.rs`等永久反例模块，T-08注册 |
| `crates/auto-lang/test/cases/`、`scripts/test-groups.toml`、现case-parent/strict reader | 同源、工具身份、固定职责组 | 拟新增`ace-m1-residual-999/`跟踪fixture/suite与唯一固定组，复用父判 |

生产范围不预设修改av-core执行器/全局资源常量。若T-02发现真实必要触面，须列原合同与全部读者、原栈门后有界认领；不得顺手改成运行时优化或协议重构。

## 4. 需求分析与背景调查

### 授权与基面

2026-11-08用户明确授权“正式规划M1残余正确性与准入收口计划，覆盖8红项、F-941-13及确认仍可达缓存问题，明确顺序、验收、941交接；只规划、不施工、不改941、不启动其他会话”。本次只规划，后续work需要实施指令，不借941原goal授权开工。

**实施授权（2026-11-09）**：用户经 `/goal 完成计划999` + /auto-plan-goal 授权整链交付——在本计划唯一树内施工、独立复审、有界修复与merge（含归档与清理）。本节即该授权的落账；后续阶段继承，不再重复请求。不改变目标/验收/范围/预算（越出即停）。

主检出基面=`03eb625ba72f7d574767db4fd98af3a1f6b44889`；941只读基面=`745f4e9aa366bcf583276a4be8641d7afe950d06`，第119轮。929/934/950/998已archived；998交付`fff9eedf0`与补证`413dae3ee`在master，旧cert-gap=0；941尚未吸收998，不据其gap=6倒退主线。941 F-941-15 AV修复属于该owner，曾通过1059项正式门，当前再做新身份闭包；本包不接管。

已读`docs/specs/overview.md`、`docs/specs/README.md`、仓库AGENTS及下列模块权威：`auto-ac/source-core-i32.md`、`auto-ac/source-scalar-i32.md`、`auto-ac/function-incremental-source.md`、`auto-av/cache-and-conformance.md`、`auto-syntax/source-frontdoor.md`、`auto-semantics/source-query-session.md`、`auto-test-case/ace-source-only-replay.md`。Core明定while和无人工深度排除；Scalar仍排除while，不能反改这些规则来迁就错误路由。

八格来自941既存`f94136-current-source-cases/capture-audit.json`：466 fresh checks，原8 FAIL，actual_gate_passed=false。f94134原始report交叉核得六格`source.unsupported`、两格`av.resource.frontend-nesting`。这些在途材料只用于本次调查；未来测试不得依赖其目录存在，T-01在当前基面重新复现。

### 输入指纹（SHA256，work重核）

| 路径 | SHA256 |
|---|---|
| `experimental/ace/crates/av/src/driver.rs` | `c4d6cb1da0b0b04fbe8a78197003f9aafdd2d1857f5bf368445103a963785789` |
| `experimental/ace/crates/av/src/query_entry.rs` | `0c08b2fa85b2e4fe9e271563650ef0e69762b300d788ea01b13de5ede631be26` |
| `experimental/ace/crates/native/src/driver.rs` | `7c96dab22dc0aba936daceaa914dbee6e57ccb9b7c731a30b32f41a2fae6759e` |
| `experimental/ace/crates/source-query/src/consumer.rs` | `d9e02040e8ad4160a0cabf2bb97f1c5c8ab42207b88f270f5808ffab6588e5b9` |
| `experimental/ace/crates/source-query/src/store.rs` | `8cc2e8d0a933b0965a01fffd51777c6fa78e0a737a880cbc9ef56288d3159977` |
| `experimental/ace/crates/source-query/src/host.rs` | `81e930720f18b45d6b44de2991a774c6e44b3754efac77144c1d16c4a90ed9bd` |
| `experimental/ace/crates/source-query/src/modules/persistence.rs` | `33c2aa581f0b019e423c731d7424b78b0d018ff032600d8973cff61832d4b87c` |
| `experimental/ace/crates/core/src/source/cst/units.rs` | `1fe09a51b44ecfb26abee04e88873f8d444710dd7b27e54fa3c90e7e5df21653` |
| `docs/specs/auto-ac/source-core-i32.md` | `ac79fda3cef96354b46a1fd66bd103bff18058f8fddbeb7bb03927d49c0327af` |
| `docs/specs/auto-semantics/source-query-session.md` | `d05be82cdc865cc77f6a375f42cd066ddbe4ff2f4c86a1b3c39b21eddd130c57` |

### 缓存可达性调查

D-950-1不只依据旧债务纳入。当前`source-query/src/lib.rs`公开导出`QueryHost`、`PageSink`和`ManagedStoreSink`；`QueryHost::prepare`经`infer_body`取得facts，命中仍赋新`revision`；`commit`调用`sink.put(key,facts)`；sink将含revision的`encode_page`写到只有sq-key的bare S934地址。这个公开API组合静态可达，**不代表默认AV/AC每次都会走到**；仓内其精确组合主要在测试缝，模块实际路径另有自己的配方。

模块发布直接访问同一sink的store，以`sq-module-page/1`+read-versions及revision=0规避冲突，不能证明bare路径已修。T-01用当前公开API做同owner同body连续两次prepare/commit和直接sink重复put，冻结hit/revision/真实Conflict/旧current事实。若期间已被正式交付修好，核实D与永久回归后核销而不重复修；若判不可达，必须给完整路径和接口裁定，不能以“CLI没用”当关闭。所有动态复现仍未执行。

## 5. 详细设计

### 5.1 五原源、八原格及期望

源路径相对于941的`crates/auto-lang/test/cases/ace-m1-expanded/`，文件=`scalar-src/<case后缀>.at`；原case完整前缀=`ace.m1.scalarq4.`。T-01将五源相同字节冻结到本包独立跟踪fixture；允许只读冻结测试输入，不复制941 WIP生产实现/汇总器。

| 闭包ID | 原case后缀 / 原check | 原expected | 当前失败 | source SHA256 |
|---|---|---:|---|---|
| R-01 | `core-p04-while` / av | value 3 | source.unsupported | `556dcc51e11bf7b723e3b061db1c2d614e845770f41617857dfd0bd1425189bf` |
| R-02 | `core-p04-while` / ac-native | value 3 | source.unsupported | 同R-01 |
| R-03 | `core-p05-while-zero` / av | value 1 | source.unsupported | `1407ca813c5aeaff33017aafd342faaf6ad67a11418e554536097bfc14b3759d` |
| R-04 | `core-p05-while-zero` / ac-native | value 1 | source.unsupported | 同R-03 |
| R-05 | `core-p06-continue` / av | value 3 | source.unsupported | `91435499a6b92ff818d280349c0d9b780078c4cf7faa7af074849c7cd81270c8` |
| R-06 | `core-p06-continue` / ac-native | value 3 | source.unsupported | 同R-05 |
| R-07 | `core-f11-add-chain-100` / av | value **300** | av.resource.frontend-nesting | `58f8576f878d2ebc443d072b2c3f4523da0f7234a80d9a8cdf165e0493465659` |
| R-08 | `core-g02-parens-200` / av | value 1 | av.resource.frontend-nesting | `b3e140b35957443297852ca93ea5c6c153d124f821e1e12e51861d798b35cfed` |

R-07文件名虽含100，期望为300，不能按名字推导oracle。两深源的native、五源的Rust/C是正对照，不冒AV成功。新增suite可用本包ID，但保原namespace/case/check/profile/recipe/source/oracle映射；“本包8/8”与“941原8格接纳转绿”分别报告。

### 5.2 Core准入：T-02→T-03

原AV `ac-av/core-run-v1`、native `ac/mir-native-scalar-v1`的既有原源请求必须成功，不要求941改case options来迁就修复。共享选择门按真实受检profile能力选择，不按ID/路径/expected/字符串改写。Scalar成功仍沿原身份/字节；只允许具名表面不支持继续尝试Core，名字/类型/资源/取消/损坏/真实语法错误不得被吞后改道。

显式`compile_scalar_product`保持Scalar拒while；Core经同一事实producer和对应verifier，不能靠旧推导生产类型或旧VM运行。T-02核`compile_profile_product`的Core分支、produce/walk对While覆盖、HIR/lowering域、保留env与warm session的profile传递，先做R-01两端最小链及坏源/普通Scalar对照。拟新增API/路由版本由T-02冻结全producer/reader；各backend不另写解析或推导器。真实域/规则变化必须进相关键，普通Scalar产物不得无故失效。

### 5.3 AV source深度准入：T-02→T-04

旧`front_door_admit`把token/表达式构造路径同128常量比较。对Auto source车道消除过期代理语法门，沿统一前门的真实资源预算准入。不能仅删if或把128改到201：核词法/CST、投影、事实walk/solver、HIR验证、lower/ABC、释放全链，原资源内两深源成功，代表性非连续嵌套/flat链也安全，畸形输入保持准确诊断。

独立Atom入口`front_door_admit_atom`的冻结递归保护不随本包放宽；若source内部仍经Atom表示，不能绕未修复的危险解析。不得笼统提高`av-core/src/contract.rs::MAX_FRONTEND_NESTING`影响所有入口。沿现source前门64MiB档及各实际请求输入/执行fuel/调用深度/进程/栈预算，不扩大原帽；超资源=resource态，畸形=syntax/bind态，拒绝后下一合法请求仍可用。T-02证明真实全链及沿用既有迭代/资源设施的可行性，无无限规模性能承诺。

### 5.4 F-941-13签名摘要：T-05

`signature_of`现用`Debug(TypeRef)`，Record/TyVar/Generic内Ident及嵌套TypeAnn会携来源span。改为明确的、无来源坐标的结构化签名投影，只消除坐标，保留原签名全部非坐标信息：名字、参数名/序/mode、名义名称与嵌套类型、返回类型；owner/receiver/模板等已有上下文身份不得丢。禁止常量签名、只比参数数目、删除名义区别或屏蔽真失效。

代表序列：有名义record类型参数的函数B前存在A；先改长A body使B注解坐标漂移，再改B body。第二改不得仅因span触发`OwnerSignatureChanged`，未改函数/方法/闭包payload对象/代际及P/IO保持，实际新代码可执行，诊断位置仍跟当前源走。覆盖中文注释/CRLF、受支持的Generic/Function嵌套注解；不能用没有内层Ident的纯int反例自证。

真参数类型/mode/返回/名义变化仍产生对应事件与真实依赖集。`signature_snapshot`、直路`edit_owner`、候选提交/恢复目录用同一规则。持久摘要含义若变，版本化或按旧规则核验后重建当前摘要；不得把旧span摘要直接解释成新语义摘要。来源坐标移动不能通过重建未改body来伪装修复。

### 5.5 可达缓存：T-01确认→T-06

同语义、同实际依赖的重复请求可增加会话/host revision，不能制造不同不可变语义页；真正语义/依赖变化必须换身份或拒旧页。禁止清库、覆盖put、吞Conflict或部分payload相等绕过冲突。

采用版本化发布配方，复用PageSink/ManagedStore区分持久事实与活请求视图，优先审查模块已用的“规范生命周期+read-versions”规则，但不改模块现有地址。至少核实际正签名读取、负名字/候选集合、能力/owner/producer/schema/profile/算法轴；不能只清revision继续把不同callee版本挤入bare sq-key。若需要扩充PageSink发布/读取上下文，作为拟新增接口由T-02冻结put→lookup/get_checked→host current→恢复完整读者，不改全局store的同键异内容拒绝规则。

host base revision和迟到/取消门保持。持久规范值不得当当前授权或当前请求revision，活视图从当前会话绑定并验证。旧bare页按旧codec/地址可读且不被重写；缺当前完整依赖时受控拒绝/显式miss，不能挑“最新同owner页”。新页新进程真实读取/准入、第三次查询复用闭环；真同完整键异语义仍Conflict，失败保旧current/target/有效页。模块`sq-module-page/1`、944根、935实例与936冻结页各保原域。

沿原事实页≤512KiB、managed 16MiB及求解5秒/1,000,000 steps等实际适用帽；新依赖摘要/索引/编码暂存须入原账，不能在规范化前分配无限副本。project soft root、required引用和lease沿原store设施；重复发布不得累积无法回收的重复页，仍被合法旧current/外部owner持有的页不提前删除，最后owner释放后按原GC规则回收。

### 5.6 与941交接及公共窗口

本包不把整包941写成depends_on，避免互等环。T-01可先做当前master私有复现与输入冻结；生产修改前按真实文件/符号取得941对应接口稳定交付或释放窗口，不复制WIP生产实现，不从其树拼运行时。

允许两种合流：941先交付诚实保留本域红的验收系统，999基于其D继续；或999先取得实际公共窗口交付，941原owner接收D999。均不强制整包互等。grammar/query/driver/units/store及固定组全文件以真实SHA、版本与所有reader交接；未释放时只做独立测试材料，不并发改同一语义合同。不自动发送消息/启动会话。

T-10形成以下可审材料，merge后填真实D；941原owner自行接收，本包不写其Plan/在途树：

| 材料 | 必需内容 | 941后续责任 |
|---|---|---|
| 八格关闭表 | R-01..08→原case/check/source SHA/expected、实际profile/recipe/终态/独立参考、parent/工具身份 | 最终基面重跑原suite，八格FAIL→PASS，其余逐键守恒 |
| 缺陷关闭表 | F13与D-950-1修前/合法近邻/修复/永久回归/版本/残余 | 核真实关闭，不因文字closed自动填PASS |
| 身份/失效清单 | base、reviewed、D、触码、grammar/目录/页配方、producer/reader/SDK镜像与工具hash | 重prepare受影响工具，失配source/cert/daily/机制/伴读/性能证据重采 |
| 预算/增量收据 | 原负载/30样本/P90、真实P/M/R/IO、恢复首编辑/坏编辑修复/源hash | 同当前身份并入ARCH与预算门；缺页/旧schema/迟到不绿 |
| 规范/余账 | SD-01..06实际落地身份、未闭项owner/关闭条件、接受前后逐键差分 | 保未知/其他红，最终组合和M1签章另验 |

998的scenario证书补交仍由941接收。本包通过、旧313/零cert-gap、扩大后M1签章三个结论分列；999不能设置`can_claim_expanded_m1`或撤守卫。941若已归档，接收由具名后继验收责任承接，不重开归档Plan；接收入口由后续指令确定，本包不预占号。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-ac/source-core-i32.md` | Core契约与共享recipe事实/验证准入一致；显式Scalar不扩 | 六格入口差异 | AC-01/06 |
| SD-02 | modify | `docs/specs/auto-av/cache-and-conformance.md` | Auto source消除旧128代理门，真实预算与Atom保护分列 | 深源成功与资源分类 | AC-02/06 |
| SD-03 | modify | `docs/specs/auto-ac/function-incremental-source.md` | 签名无span，连续跨owner编辑与真依赖变化可区分 | F13 | AC-03/06 |
| SD-04 | modify | `docs/specs/auto-syntax/source-frontdoor.md` | 目录签名/来源/版本、Core/Scalar统一前门职责明确 | 恢复和读者不分叉 | AC-01/02/03 |
| SD-05 | modify | `docs/specs/auto-semantics/source-query-session.md` | bare发布新配方与生命周期/真实依赖分离，旧页兼容、模块配方不变 | D-950-1 | AC-04/05/06 |
| SD-06 | modify | `docs/specs/auto-test-case/ace-source-only-replay.md` | 本包五源八格跟踪闭包、941接收映射，不依赖忽略/在途文件 | 新鲜检出可复验 | AC-01/07 |

六项为候选delta，本轮不修改canonical Specs。T-02若证实其他规范有真实触面，在本计划有界修订并列消费者，不留待merge暗改合同。

## 6. 测试设计

所有下述测试/新接口均未执行。后续work负责实现/自查，最终review在同一权威树独立上下文执行，期间停止实施写入；review不修实现，needs_fix返回原树。不自动创建agent/chat，不规定模型；复用身份未变的证据并明确依据。

| Case ID | owner_stage / AC / 任务 | 输入与真实入口 | 可观察oracle / 区分错误实现 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | work / AC-01/02 / T-01/03/04/08 | 五原源八格，经原AV/native recipe和case-parent；Rust/C独立参考 | 精确3/3/1/1/3/3/300/1，八项实际执行，错oracle红 | 不以新源/手工MIR代原源；941父判转绿待其接纳 |
| V-02 | work / AC-01 / T-02/03 | 显式Core/Scalar、条件for/while、未知调用/错bool/坏语法 | Scalar成功字节/键保持，显式Scalar仍拒while，共享recipe选Core；错误不被宽域吞 | 935实例/936CT原门保持，不自动扩Core模板/CT组合 |
| V-03 | work / AC-02 / T-02/04 | 两原深源、128两侧合法连续/非连续嵌套/flat链、调用/块交错、缺闭括号 | 原值成功、坏源准确首诊断无panic，原资源两侧按资源态拒；下一合法请求成功无泄漏 | 代表尺度沿原Core夹具T-02冻结，不新设通用语法上限 |
| V-04 | work / AC-02/06 / T-04/07 | Atom坏输入、原fuel/cancel、相关原256KiB栈深链合同 | 消除source代理门不绕Atom/解码/资源验证，不抬栈；拒绝/取消不冒回收 | 若触及AV核心准入链，原F15/3072深链栈门必须保持 |
| V-05 | work / AC-03 / T-05 | 名义record注解的A变长→B body-only；UTF-8/CRLF及受支持嵌套类型 | B无假签名事件、未改P/IO=0且Rc/代际保持；新代码运行、当前坐标正确 | 必须含内层Ident，纯int例不足 |
| V-06 | work / AC-03/06 / T-05/07 | 真参数类型/mode/返回/名义变更，恢复→首编辑，坏编辑→修复 | 真依赖仍失效，旧目录按版本处理；失败不发布，fresh与增量等价 | 不得屏蔽所有事件或仅比参数数目 |
| V-07 | work / AC-04 / T-01/06 | 同会话`QueryHost<ManagedStoreSink>`连续prepare/commit、直接sink重复put、MemorySink合法对照 | 修前hit+revision增加导致真实冲突；修后无无谓solver/重复页，host revision仍单调；新进程再读/第三次复用成功 | 精确bare链与默认CLI可达分列，不借模块绿核销 |
| V-08 | work / AC-05 / T-06/07 | callee/负名字/能力变化、异owner、真同键异内容；cancel/late/写失败 | 真依赖换身份/拒旧、真Conflict保留；旧current/target/有效页不变，下一成功和恢复正确 | 不清库/覆盖/吞错，不从页恢复grants |
| V-09 | work / AC-05 / T-06/07 | 旧bare金样、新配方、模块配方与944/935/936近邻；重复页/root/lease持有和释放 | 旧原义读与错版本拒、新旧不串，模块地址/字节稳定，缺依赖不选最新页；重复不漏页，旧活owner保活、最后释放可GC | key/codec变化须全reader/真实读/原预算闭环，不能只测encode或提前删除旧root |
| V-10 | work / AC-06 / T-07/09 | 原115函数/≥273575B负载、原30修订，body/依赖编辑、恢复首编辑及失败修复 | 原P90≤2000ms、资源/P/M/R/IO不减，未改生产/IO=0且真实身份保持；强制whole控制红 | 冷准备/rustc/cl认证分列，不新承诺冷编译2秒 |
| V-11 | work / AC-07 / T-08/10 | 缺一原格、错hash/recipe/profile、伪成功/旧工具/错误参考/删证 | 现strict父门拒，关闭表由真实结果重建 | 不造通用checker，不写941 registry/composer |
| V-12 | work+review / AC-06/07/08 / T-08..11 | 仅跟踪输入的新鲜检出、本包固定组、职责/必要集成 | fixture/oracle/manifest含JSON均在git，零依赖941磁盘残留；发现/执行/原条件排除分列 | raw在本包scope，哈希不能代语义成功 |

### 正式入口与职责组

名称已静态核对当前`scripts/test-groups.toml`；`cargo t --list`虽是静态清单入口，alias仍经Cargo driver，本轮未运行以避免构建。work在本计划树启动后再核，不能使用不存在的组。

- 完整职责候选：`cargo t ace-source ace-semantics ace-av ace-native ace-a2x ace-artifacts`。实际修改语法运行时/grammar另加`ace-syntax`。这些覆盖共享AST/类型事实、AV/AC/AX和页存储，不能只跑parser自证。
- 必需集成：`cargo t case-ace-source-query-934 case-ace-unified-source-929 case-ace-native-warm-admission-932 case-a2c-hir-scalar-921`。Rust标量原真链由`ace-a2x`完整包承担，当前表无独立892组，不猜新组名。实例/CT入口或key受影响加`case-ace-source-instances-935 case-ace-source-cte-936 ace-comptime`；模块页接口触码加`ace-stdlib`与模块原测试。
- 拟新增唯一组`case-ace-m1-residual-999`，suite拟为`crates/auto-lang/test/cases/ace-m1-residual-999/ace-m1-residual-999.suite.ad`；T-08注册，先最小链再扩八格。未注册前不是已有可运行入口。
- case-parent adapter/SDK镜像/治理触码时补`auto-test-case ace-adapters infra-ace-m1 infra-fixed`完整职责；否则说明证据复用依据，不无条件扩大宿主范围。
- 941私有`case-ace-m1-scalar-q4-941`、`infra-ace-m1-arch`仅在其实际D已落本树后运行；未落时先以本包原字节/原recipe/check语义证明并交941重放，不拼WIP生产实现。
- 原性能/压力按`ace-performance`所属的原具名入口、原样本/规模/ignore/平台条件定向验证，T-01列实际测试名与预算；不顺手整治全62测试超窗。最终review后，merge在冻结提交跑所选完整职责/必要集成并集；周期`cargo t all`仅原due主检出单实例，不叠全仓三轮。

每次记录源/工具/配置/输入身份、退出码、真实执行数、原条件排除、准备/运行/清理时间、资源峰与终态。原负控子项非零须父门明确判控制成功，父exit0不代子项完成；首红、环境归因、隔离重放分别保留，不缩样本/抬截止/ignore。

## 7. 验收标准

| AC | owner_stage | 可观察完成条件 | 验证 / 任务 |
|---|---|---|---|
| AC-01 | work | 六Core原格原源原值经正式AV/native recipe通过，事实权威/显式Scalar/语义拒绝保持 | V-01/02；T-01..03/08 |
| AC-02 | work | 两深源AV返回300/1，无旧128 source代理拒绝；整链深/坏/资源与Atom原合同保持 | V-01/03/04；T-02/04/07 |
| AC-03 | work | F13连续跨owner变长body编辑不假失效，真签名/名义/mode仍失效；对象、恢复、坐标、计量正确 | V-05/06；T-05/07 |
| AC-04 | work | 当前公开可达bare链修前复现或有实际关闭D；修后重复发布/再开/再查询正确，不借模块绿核销 | V-07；T-01/06 |
| AC-05 | work | 生命周期/语义/真实依赖分离，版本与全reader闭环，真Conflict/旧页/失败保旧不退 | V-08/09；T-06/07 |
| AC-06 | work | 受影响完整职责/必要集成、原增量结构/P90/资源门有有效身份结果，反控敏感，零未处置新增warning/fmt/debug输出 | V-02..10/12；T-07/09 |
| AC-07 | work | 五源八格跟踪自足、独立oracle/parent证据、版本失效清单/SD与941材料完整；不写941、不冒签M1 | V-11/12；T-08/10 |
| AC-08 | review | 同树独立上下文逐AC/V/SD、全部实际读写/恢复/遗漏/健康核验pass；in-contract未闭不得记债豁免 | T-11 |
| AC-09 | merge | 冻结并集/漂移复验、D/规范来源核实、归档及实际输出根/scope/树清理完成；941接收状态单列 | T-12/13 |

## 8. 执行步骤

拟唯一树=`D:/autostack/.wt/lang-999/auto-lang`，branch=`codex/plan-999-dev`，尚未创建。work从实际main建树并确认兄弟依赖，不建junction/symlink。材料唯一scope=`D:/autostack/plan-artifacts/plan-999/`，开工登记owner/用途/在途状态；纯规划检查用系统临时目录。

顺序：冻结复现 → 整链可行性/公共稳定窗口 → 六格Core准入 → 两格深源 → F13 → 可达缓存 → 联合恢复/回归 → 正式suite → 原门/性能 → 交接 → 独立review → 合入沉淀 → 清理。缓存可达性调查在T-01先行，修复按序；不启动并行会话。

| task_id | owner_stage | 前置 | 路径/动作、结果与验收映射 | 完成 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；master可读 | 核D929/934/950/998和941边界；冻结五源/八格/参考/正式入口，重现8红/F13及公开Host+sink；列实际可达/版本/完整读者/原性能入口；AC-01..07/V-01/05/07 | [x] |

T-01 证据（2026-11-09，树`D:/autostack/.wt/lang-999/auto-lang`@b4f1af0be，branch `codex/plan-999-dev`，scope `plan-artifacts/plan-999/`）：
- 指纹：§4表10文件SHA256全部一致（worktree实测）。依赖：929/934/950/998均在archive；941 active（其树35a21a381 clean，只读未触）。
- 冻结：五源逐字节复制到`crates/auto-lang/test/cases/ace-m1-residual-999/src/`，SHA256与§5.1一致；suite/case文件按原case语义1:1编写（新ID `ace.m1.res999.*`，recipe/entry/expect逐字段同原），inventory发现10 checks全RUN。
- 8红复现（`t01-repro/`，auto-case run + av-worker/mn-worker release）：R-01..06=`source.unsupported`(stage frontend)、R-07/R-08=`av.resource.frontend-nesting`(stage admission)；两深源native正对照PASS（exit 300/1）。总计10 checks=8 FAIL+2 PASS，与941审计一致。
- F13复现（`t01-f13-probe.txt`，scratch probe）：A加长使B span漂移+16后，B仅body编辑触发假`OwnerSignatureChanged{owner:"b"}`；纯int对照无假事件（record Ident是区分载体）。
- D-950-1复现（`t01-cache-probe.txt`，公开API）：`QueryHost<ManagedStoreSink>`同源连续prepare/commit，第二次revision=2同canonical key→`key conflict on S934/S934[sq-key=…]`（CommitError::Sink）；直接sink重复put同判。确认为当前可达真缺陷（T-06施工，非核销）。
- 可达/版本/读者/性能入口：`QueryHost`生产调用仅在source-query crate内（host/instance_host/lib导出），av/native driver不直调——bare S934链是公开API可达、测试缝演练；模块页读者`modules/persistence.rs:723`用`get_checked`（`sq-module-page/1`独立配方，不动）；bare `get_by_key`无非测试生产读者。页codec=PAGE_CODEC_VERSION 1；MAX_FRONTEND_NESTING在`av/src/driver.rs:1841/1887`消费（contract常量）。V-10原性能入口=`av_closures_incremental_warm_edit_p90`（`av/tests/cases/av_closures_incremental.rs:289`，D828基线273575B/115函数/30修订，P90=10样本至多1超限、限2000ms）。
| T-02 | work | T-01；公共稳定接口/释放窗口 | consumer/query_entry/driver/units/store有界整链探针，冻结Core与深输入资源方案、签名版本、缓存新旧配方和所有reader；不成立needs_replan；AC-01..06/V-02..09 | [x] |

T-02 冻结与证据（`t02-chain-probe.txt`，同树scratch探针，未改生产码）：
- **Core准入可行（T-03方案冻结）**：6红根因=`compile_profile_product`仅lower Scalar/TextRecord（consumer.rs:302-309显式拒其余）+parse用Scalar面（`while`仅Core面接受，project.rs:1573 wide=scalar_or_wider即拒）。P1：legacy `compile_source`(Core面)+`verify::verify`(Core域)→`lower_checked→verify_mir→LegacyAbc encode/decode/admit→execute`全链=3 ✓；P2：Core-parse后`SourceQuerySession::infer_body`对while body产出完整facts（8节点/0 missing/0 diags）——producer/walk已覆盖While。T-03实施=consumer加Core分支（同事实producer+Core verifier+Core descriptor），native侧同一product入口（native/driver.rs:71同源），无需后端各自解析。941窗口：master当前无Core消费分支，接缝文件consumer.rs/units.rs/store.rs均941未改（指纹=T-01核），可开工。
- **深输入（T-04方案冻结）**：g02(200括号)整链绿（compile 8ms，lower/ABC/admit/exec全过=1）——仅128代理门拦它；f11(300项加链)compile全绿但**`lower_checked`内`eval_expr`递归爆栈**（debug测试线程2MiB@299深；生产worker主线程Windows默认1MiB更险；arena是flat的，仅调用栈递归）。冻结：①source车道删除`front_door_admit`的128 token/表达式代理比较（保留MAX_INPUT_BYTES与`front_door_admit_atom`原合同）；②真实预算=统一前门64MiB/parse-depth 1M + lowering既有`MAX_EXPR_DEPTH=512`资源帽（不动）；③T-04把`eval_expr`按仓内纪律（resolve.rs:523/verify.rs显式工作栈先例）改显式工作栈，使129..512深在真实线程栈内可用；>512=`mir.lower.depth` resource态。
- **签名版本（T-05方案冻结）**：`signature_of`读者=units.rs内两处比较（edit_owner直路1151/候选提交1642）+`signature_snapshot`（无生产消费者，仅测试）；digest字符串不入CatalogManifest（manifest只有content_sha）。⇒改span-free投影是纯行为修复，**无需持久化版本迁移**；manifest/caps/grammar身份不动。
- **缓存新旧配方（T-06方案冻结）**：采**方案A——sink层语义去重**：`ManagedStoreSink::put`先`get_by_key`，若同键现存页与facts**除revision外逐字段相等**⇒返回现存artifact id（幂等重发布，不再put）；不等⇒照常put让真Conflict浮出。页codec v1不动（wire字节不变）、旧bare页可读不被重写、模块`sq-module-page/1`零触、store同键异内容拒绝规则不动；存储页内revision降为"首次发布时的请求视图戳"（非当前授权，host活视图自持revision——§5.5分离原则）。否决v2重编码方案（会改页身份并牵动全部reader/版本合同）。
- 控制项：P4坏源（缺括号）=`source.syntax`准确诊断非resource；P5超尺寸输入=admission/`av.resource.input-size`。
| T-03 | work | T-02 | consumer/query_entry共享Core事实门与native域选择；六格绿、Scalar/错误分类保持，永久近邻；AC-01/V-01/02 | [x] |
| T-04 | work | T-03 | av source前门消除旧128代理门，整链深/坏/资源/Atom与下一请求控制，两原格绿；AC-02/V-01/03/04 | [x] |

T-03/T-04 证据（commit `c2f60dc79`，树 codex/plan-999-dev；run证据`t03-run1/2`、`t04-run1`、`deep-probe-out`）：
- **10/10 PASS**（`t04-run1`，真recipe×release workers）：R-01..R-08全绿（3/3/1/1/3/3/300/1经case-parent strict判值），两native正对照保持。六格经共享选择门（scalar面surface-reject→Core事实面），native经ladder Core步（Domain::Core→scalar arm）。
- 实施面：consumer.rs（`compile_core_product`+Core验证臂）、resolve.rs（fact绑定放行Core面）、av/query_entry.rs（`compile_scalar_or_core_source_*`共享门：仅`source.unsupported`@Text全体才转Core，Core失败以其诊断为终态）、av/driver.rs（source车道换门）、native driver/sd（Core步+`Domain::Core`）、lowering（512深改Resource分类`mir.lower.depth`）、resources_tier重指真预算（14测试绿：≤512合法形执行得真值、>512=mir-lower/mir.lower.depth资源态、Atom门128原样、输入尺寸门原样）。
- 深裕度：release av-worker主线程跑511项链=511 ✓（`deep-probe-out` scratch case）；debug测试线程经`with_deep_stack`(64MiB显式栈)执行深形——声明512帽在生产release全可用，测试侧显式配栈不依赖runner默认。
- 受影响切片：ace-source-query 89✓、ace-core source_units 13✓+25✓、ace-native lib 12✓、av_resources 15✓（302s，含既有32MiB perf腿）。完整职责组并集归T-09冻结跑。
- 残余到T-08补永久反控：gate的坏语法不转道case、拒绝后下一合法请求成功case（V-02/03负控）。
| T-05 | work | T-04 | units.rs语义摘要和全部读者/持久版本；F13、真变更/当前坐标/恢复反控；AC-03/V-05/06 | [x] |
| T-06 | work | T-05；可达或已有关闭D | host/session/store版本化发布/读取，真实依赖与活revision分离；重复/再读/真冲突/旧页/模块不退；已修则核销不重复施工；AC-04/05/V-07..09 | [x] |

T-05 证据（commit `05f...见git log`=`fix(ace): span-free catalog signature digest`，core conformance 4/4）：
- `signature_of`→`type_digest`结构化无坐标投影：Record/TyVar/Optional/Generic/Function全覆盖（名字/参数名/序/mode/名义与嵌套类型/返回全保留，仅去span）。持久面核验：digest字符串不入任何清单/存储（manifest只带content_sha）⇒无版本迁移。
- 永久反例模块`core/tests/cases/m1_residual_999.rs`（conformance注册）：①record参数漂移+body编辑零假事件+冷重建digest相等；②真参数类型/mode/返回/参数名变化仍触发；③CRLF+中文注释稳定；④嵌套Generic(List<P>)内层Ident漂移不误报、List<P>→List<int>真变化触发。
- 变异验证：临时回退旧`{:?}`digest→3个漂移场景红（假事件复现）、真变化测试仍绿——oracle具区分性。邻接面：source_units 25✓、source-query 89✓、av closures_fn 14✓。

T-06 证据（commit `fix(ace): idempotent republish of same-semantics SQ pages`，source-query 91✓）：
- 方案A落地：`ManagedStoreSink::put`先按完整键读现存页，facts除`revision`外逐字段相等（`facts_eq_mod_revision`）⇒返回现存artifact id（幂等，不再put）；真异内容照常put保store硬Conflict。页codec v1字节不变、bare旧页原样可读、模块`sq-module-page/1`零触、store同键异内容拒绝规则不动；存储页内revision语义=首次发布视图戳（非当前授权）。
- 公开链复现→修复验证：`QueryHost<ManagedStoreSink>`同源连续prepare/commit，第二次hit带新revision同canonical key——修前`key conflict on S934/S934[sq-key=…]`（T-01实录+变异复现），修后commit2成功、artifact同一、host revision单调2、跨进程重读成功。
- 永久回归sq98/sq99（tests.rs）：幂等重发布/同artifact/单调revision/新进程读取；直接sink重复put幂等+真篡改同键仍Conflict。变异验证：comparator退回含revision比较→sq98红出原冲突串。
| T-07 | work | T-03..06 | AV/native及受影响A2R/C/模块读者fresh/hit/restore→首编辑、坏编辑修复、cancel/late/发布失败保旧；P/M/R/IO与身份守恒；AC-02..06/V-04..10 | [x] |

T-07 证据（同树，run目录`t07-934-repro`及各组results）：
- 必需集成三套（cargo t正式门）：`case-ace-native-warm-admission-932` PASS（ladder改后暖编辑/拒绝分类全绿）；`case-ace-unified-source-929` PASS（统一前门九profile腿不变）；`case-ace-source-query-934` **base红→修复→PASS**：a2r/a2c runtime金样在master即红（PLAN-961给runtime加`InvalidScalar`/`A2X_TRAP_INVALID`+numeric helpers后未刷新934金样；程序产物main.rs/generated.rs逐字节同、仅runtime头+manifest内sha差）——按逐字节delta证据刷新4个金样（commit`test(ace): refresh 934 a2r/a2c runtime goldens`），非999回归。
- `ace-a2x`组PASS（A2R/C直接消费受检HIR的读者回归）；`av_closures_incremental`19✓（暖编辑/P90预算族）；`ace-native` lib 12✓＋1个**继承红**`allocation_controls::exact_backing_alignment…`（master基线3/3复现，mn-collections-worker分配器域，与999触面无关，记为在途/归他域，不代修）。
- 模块/页读者：source-query 91✓（含模块persistence/host/store族）。
- 身份/预算计数：上述套件各自带P/M/R/IO计数器断言，全部随组通过；正式30样本P90数字归T-09冻结跑记录。
| T-08 | work | T-07；固定组全文件窗口 | 新跟踪fixture/suite/参考、唯一固定组及strict父门反控，八格原身份映射/新鲜检出闭包；不依赖941私有WIP；AC-01/07/V-01/11/12 | [x] |

T-08 证据（commit`test(ace): register case-ace-m1-residual-999…`，正式门run`run-0syyakf6`）：
- suite扩至12 checks：8冻结格+2 native正对照+2端到端负控——①真语法错误在while之前：选择门不吞错，parse/`source.syntax`判决保持（T-03/V-02负控落位）；②600项链超lowering真512帽：verify面/`mir.lower.depth`资源判决（父级map_diag_stage把mir-lower归verify面；T-04/V-03资源态负控落位）。拒绝后下一合法请求成功控制由resources_tier序列覆盖（同测试内拒绝后执行）。
- 唯一固定组`case-ace-m1-residual-999`注册（test-groups.toml，suite路径同§6拟案）；`cargo t case-ace-m1-residual-999`正式门PASS（12/12）。
- 新鲜检出闭包：fixture仅引用tracked `src/`文件，无941/外部路径依赖（"941"仅provenance注释）；strict父门/输出admission经正式cargo t自动执行（本轮未触case-parent/治理码——复用既有治理机制，依据：parent拒绝面由既有治理测试族钉住，本包未改其代码）。
- R-01..08→原case/check映射：case.ad头注释逐格登记（含R-07名100值300警示）。
| T-09 | work | T-08；冻结候选/采样窗口 | 按§6跑职责/必要集成与原负载30样本P90/资源/结构，健康与首红/重放/身份齐录；AC-06/V-10/12 | [x] |

T-09 证据（runs：`t09-union`、`t09-av-native-rerun`、`t09-native-final`、`t09-av-final`）：
- **完整职责组**：`ace-source`✓、`ace-semantics`✓、`ace-a2x`✓、`ace-artifacts`✓；`ace-av`=912/916绿+**3继承红**（collections small-stack族：base同红，见下）；`ace-native`最终冻结跑**PASS（230执行）**。首跑union揭5个native红（4×numeric_codegen+sd911 terminating-profile pin）=本包ladder初版把Core非表面失败当终态（数字字面量对Core是语法错、对更宽面是合法surface）——修复=Core为**侧脸**（仅成功返回，任何失败继续爬梯，record/TR/numeric终态规则原样，commit`fix(ace): native ladder Core step is a side face…`）；另`case_binding`nesting_oracle按新合同重指（201透明括号执行=1，R-08同类）；sd911 attempted-pin更新[Scalar,Core,Record]。修复后native组全绿、999/932 case组重验PASS。
- **必需集成**：`case-ace-source-query-934`✓（金样修复后）、`case-ace-unified-source-929`✓、`case-ace-native-warm-admission-932`✓、`case-a2c-hir-scalar-921`✓、`case-ace-m1-residual-999`✓（12/12）。
- **继承红登记（非999）**：①av collections 3例small-stack爆栈（master基线`cargo test`同红；nextest组跑时亦现）＝collections运行时深链/小栈域；②native `allocation_controls::exact_backing_alignment…`（基线3/3红；nextest组跑3次中1次现＝状态相关）＝mn-collections分配器域；③首rerun的mn P90红=与ace-av并跑抢占（隔离重跑绿，数字见④）。
- **④预算/增量收据**：AV `av_closures_incremental_warm_edit_p90`=10样本[0.2125,0.2053,0.179,0.1326,0.129,0.1338,0.1537,0.1336,0.1402,0.1518]，**P90=0.205ms**（门≤2000ms）；mn sd911 source-v1（30样本，debug档）：fresh P50/P90/P99=6057/6342/7221ms（报告不设门）、restart-warm P90=1673.9ms✓（≤2000）、body-edit P90=1792.2ms✓、signature-reject P90=326.4ms✓；ace-av组含原32MiB perf腿（原样保留，约5分钟长原子沿用原owner档）。身份：各组results.json记录config_sha256/selection/计数器；工具worker按树重建。
- **健康**：本包触面`cargo fmt`清（仓存留fmt漂移=base既有159处非本包文件）；新增warning=0（ace-av 3条=base既有）；无新增debug输出；测试残写`docs/reports/plan827/incremental-raw-*.json`已清。

### §5.6 交付材料（T-10；941原owner接收，本包不写其在途文件）

**① 八格关闭表**（基面=master`b4f1af0be`，交付树=codex/plan-999-dev@`38c93b966`，冻结fixture SHA=§5.1五源）：

| 闭包 | 原case/check | 修复前 | 修复后（`t04-run1`/`run-0syyakf6`，真recipe） |
|---|---|---|---|
| R-01 | scalarq4.core-p04-while/av | source.unsupported@frontend | PASS exit 3（AV经共享选择门→Core事实面） |
| R-02 | scalarq4.core-p04-while/ac-native | source.unsupported | PASS exit 3（native ladder Core步→scalar arm） |
| R-03 | scalarq4.core-p05-while-zero/av | source.unsupported | PASS exit 1 |
| R-04 | scalarq4.core-p05-while-zero/ac-native | source.unsupported | PASS exit 1 |
| R-05 | scalarq4.core-p06-continue/av | source.unsupported | PASS exit 3 |
| R-06 | scalarq4.core-p06-continue/ac-native | source.unsupported | PASS exit 3 |
| R-07 | scalarq4.core-f11-add-chain-100/av | av.resource.frontend-nesting | PASS exit 300（前门代理移除，lowering 512帽内299深） |
| R-08 | scalarq4.core-g02-parens-200/av | av.resource.frontend-nesting | PASS exit 1 |

parent/工具身份：case-parent strict判值；av-worker/mn-worker release（mn-worker-exe-sha256=`0cc66f60dbd5a0a45a0fcca138ca755315e31357f5795cabec24d054612b0c6a`@T-07组run）。

**② 缺陷关闭表**：

| 缺陷 | 修前（复现证据） | 修复 | 永久回归 |
|---|---|---|---|
| F-941-13 | A加长→B span漂移+16→B仅body编辑假`OwnerSignatureChanged`；纯int对照无假事件（`t01-f13-probe.txt`） | `signature_of`→`type_digest`无坐标结构化投影（commit`6dd05a17a`） | `core/tests/cases/m1_residual_999.rs` 4例（漂移/真变化/CRLF中文/嵌套Generic）；变异回退旧digest→3例红 |
| D-950-1 | 公开`QueryHost<ManagedStoreSink>`同源连续prepare/commit第二次`key conflict on S934/S934[sq-key=…]`（`t01-cache-probe.txt`） | sink按完整键预读，facts除revision外相等⇒幂等返回现存artifact（commit`981c8ea85`） | source-query `sq98/sq99`（幂等/同artifact/单调revision/跨进程重读/真Conflict保持）；变异→原冲突复现 |

版本/残余：digest无持久面（无迁移）；页codec v1字节不变、bare旧页原样、模块`sq-module-page/1`零触；残留=无（两缺陷以原反例+对照+变异三面关闭）。

**③ 身份/失效清单**（941重prepare受影响面）：
- base=`b4f1af0be`；999交付=5提交（`ef2b00007`fixture、`c2f60dc79`T-03/04、`6dd05a17a`T-05、`981c8ea85`T-06、`efce1ead9`934金样修复、`38c93b966`T-08）。
- 触码：`source-query/{consumer,lib,store,tests}`、`core/source/{resolve.rs,cst/units.rs}`、`av/{query_entry,driver}`、`native/{driver,source_diagnostics}`、`lowering/hir_adapter`、`av/tests/resources_tier`、`core/tests/{conformance,cases/m1_residual_999}`、`test-groups.toml`、934金样2组。
- 语法/目录/页配方：`FRONT_CAPS_VERSION=ac-source-front/3`未变（语法零触）；页codec=SQ934P v1未变；grammar-id不变；lowering身份面：`mir.lower.depth`由Unsupported改Resource分类（code不变）。
- 工具hash：av/mn/a2x worker随树重建（941按其流程重prepare/重采证据）；case-parent/auto-case未触码。
- 失效判定：941的AV/native源码执行证据若绑定旧128门或scalar-only面需重采；绑定通用recipe/门为绿的证据不受影响。

**④ 预算/增量收据**：T-09冻结跑（本节后补数字：原115函数/273575B/30修订P90≤2000ms、组运行时间/资源峰）。

**⑤ 规范/余账**：SD-01..06落地身份见merge沉淀；未闭项：①`allocation_controls::exact_backing_alignment…`继承红（master基线3/3，mn-collections分配器域，非999触面，owner=该域）；②961未刷新934金样一事已由999代修（`efce1ead9`），若961另有金样遗漏归其owner。
| T-10 | work | T-09 | 八格/F13/缓存关闭表、SD-01..06、精确版本/工具失效和941材料；KNOWN-DEBT只更新实证项，不假closed；AC-07/V-11/12 | [x] |

T-10 证据：§5.6五份材料①-⑤已按实际交付填入（①八格表②缺陷表③身份/失效清单④预算收据=T-09数字⑤规范/余账）。KNOWN-DEBT的D-950-1更新为"修复已交付PLAN-999，关闭待独立review"（未假closed）；继承红3项（av collections small-stack族/native allocation_controls/mn P90并发抢占）按base复现归档于T-09记录，不冒999内修复。SD-01..06候选delta维持§5表述、由merge按实测落地（FRONT_CAPS_VERSION/页codec/grammar-id零变——SD-02/SD-05的实际触面为driver前门与sink发布行为，规范文本在merge时沉淀）。
| T-11 | review | T-01..10完成冻结，暂停实现写入 | 同树独立上下文重建AC-01..07/V/SD，原case来源/旧页/遗漏/workaround/健康专项；AC-08 | [x] |

**T-11 复审记录（final，独立子代理上下文，2026-11-09）**：
`stage: review | review_scope: final | reviewer: 独立general-purpose子代理（非实施上下文；同权威树只读+实测） | PLAN-999 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-999/auto-lang | branch: codex/plan-999-dev | reviewed_commit: 99f43e6fe4322facfc1b4115da207076691d00bb | base: b4f1af0be | deps: master无WIP代码/941树只读未触 | spec_inputs: docs/specs六目标存在、SD-01..06与实际行为一致、frontmatter supersedes=6 SD目标、GOAL-002实存 | acceptance: AC-01..08全pass（AC-09 merge-owned pending）；执行面：亲自跑case-999正式门12/12+receipts值3/3/1/1/3/3/300/1+五源SHA直核、m1_residual 4/4、source-query全套123绿、scalar_source 7/7、av_resources 15/15（258s）、双基线warning逐字一致、13文件rustfmt基线对比、继承红master复现（stack overflow）、9提交diff逐行（含comparator 14字段审计、934金样delta审计：程序产物SHA不变仅runtime模板差）；复用：T-09的mn P90三态与宽组绿（同一冻结提交、树未变，可复用依据已记录） | findings: R9-01/R9-02（均P3注释级：driver.rs残留128门旧prose/LevelChain死链；sq99"fresh process"措辞）——已按非语义post-pass fold以comment-only提交修复（reviewed baseline 99f43e6fe不变） | pending_closeout_gates: merge冻结并集复用本commit证据（identity未变）、SD沉淀、归档、树/scope清理 | evidence: 见上+review scratch `plan-artifacts/plan-999/review/` | next: merge（T-12/13）`
| T-12 | merge | T-11 pass与后续合入授权、主线窗口 | 冻结并集/触面漂移复验，SD读回/D来源核实/归档，941接收分态；AC-09 | [x] |
| T-13 | merge | 已核实landed/规范来源 | 实际输出根对账，收据折Plan、scope整删或具名hold/登记刷新，wt-guard clean后删唯一树/分支/空组；AC-09 | [x] |


输出根包括实际命令用过的target/外部副本/临时源码，不能只看登记目录。merge按政策整删scope及worktree构建产物；具名持有必须有owner/理由/关闭条件，未知归属fail-closed。日志/探针/截图等不进仓；接口/身份漂移只重验受影响面，不继承失配PASS。

## 9. 复审记录

stage: goal | plan_id: PLAN-999 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-999/auto-lang | commit: 99f43e6fe (+comment-only fold for R9-01/02) | next: merge

stage: goal | plan_id: PLAN-999 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-999/auto-lang | commit: 99f43e6fe | next: review

2026-11-09 stage:work | PLAN-999 r1 | outcome:pass | worktree:D:/autostack/.wt/lang-999/auto-lang | branch:codex/plan-999-dev | base:b4f1af0be | code_commit:`99f43e6fe`(HEAD=8提交：ef2b00007→c2f60dc79→6dd05a17a→981c8ea85→efce1ead9→38c93b966→fb0a69788/a03ee522e→99f43e6fe，见§5.6③) | dependency_revisions:master检出无WIP代码，941树只读未触 | task_ids:T-01..T-10全[x]（current_step 10/13；T-11..13归review/merge） | evidence:10/10冻结格真recipe PASS、F13/D-950-1原反例+变异关闭、职责组ace-source/semantics/a2x/artifacts✓+av(1041/3继承红)+native(230✓)、必需集成5 case组✓、AV P90=0.205ms/mn三态P90✓、fmt/warning健康✓ | blockers:无（继承红3项归他域登记） | next_responsible_role:review（T-11，同树独立上下文，冻结`99f43e6fe`提交）

**work就绪声明（AC-01..07逐项）**：AC-01✓六格经共享选择门真recipe绿；AC-02✓两深源300/1、Atom门/输入门/512帽原样；AC-03✓F13假事件消除+真变化保持（变异验证）；AC-04✓公开bare链复现→幂等修复；AC-05✓生命周期分离、真Conflict/旧页/模块配方不退；AC-06✓职责/集成/预算组有效身份结果（继承红单列非本包）、健康零新增；AC-07✓五份941材料+fixture自足+不写941不冒签M1。统一回归门（merge冻结并集）已按T-09跑毕并记录，review可复用其身份不变的证据。

2026-11-08 stage:new / PLAN-999 r1 / outcome:pass（仅规划静态合同核验，不是实施/独立final review通过） / scope:仅规划 / current_step:0/13 / implementation_review:none / next:后续实施指令到达后T-01。

规划静态核验：0..10十一章齐全；T-01..13、AC-01..09、V-01..12、SD-01..06、R-01..08唯一连续、引用无外来ID，任务全未勾选；九项AC均有任务/可观察验证，四个依赖均已有归档来源，DAG不以941整包为前置；二十个既有固定组名与当前表一致，新999组与941私有组均明确标未落主线；六份Spec目标存在，候选delta与frontmatter一致。中央999在active/archive唯一，.next-id递增为1000；没有创建下一编号计划。已有八格输入hash/期望按独立参考表核对，R-07明确300。未来探针/测试未执行，不把静态检查充能力关闭。

改动范围对账：规划前后59份既存保护文件中，除预期递增的`.next-id`外内容hash均不变，含941计划与已有WIP/草案；新路径仅本计划。`git diff --check`通过。未创建本包worktree/branch/scope，未写源码/语料/证书/current Specs或调用其他会话。

本轮只读代码/规范与941既存材料，未跑产品/探针/性能或独立实施review。八格FAIL是原owner已有实测；F13有既存实证与当前源码对应；D-950-1静态公开API可达，当前动态复现明确归T-01。规划静态pass不得关闭缺陷、充产品PASS或签M1。

**T-12/T-13 merge 收据（PLAN-999:r1，2026-11-09）**：
- `prepared`：reviewed基线=99f43e6fe（final pass，R9-01/02为comment-only post-pass fold f95b4464e）；SD-01..06六份canonical delta在唯一树以docs-only后代提交（6d8ec265f），实现/依赖零变化 ⇒ delivery_commit=6d8ec265f；交付前identity重绑：case-999+case-932在6d8ec265f全PASS。
- `landed`：master无新提交（merge-base=b4f1af0be，rebase无操作、哈希不变）；主检出`git merge --ff-only codex/plan-999-dev`成功，master tip=6d8ec265f，无merge commit、线性历史。
- `canonical_verified`：六份docs/specs路径随6d8ec265f落在master（show --stat核对）；主检出smoke=`cargo t case-ace-m1-residual-999` PASS（主检出自带工具链重建后12/12）。无index/ledger写入。
- `archived`：计划文件git mv至docs/plans/archive/、status: archived、T-12/13勾选、13/13；本收据与完成摘要随归档提交入默认分支（仅本Plan文件——KNOWN-DEBT含他域在途编辑不代提交）。941接收分态：D999材料在§5.6，由941原owner或具名后继按其身份接收，本包未写其在途文件、未设can_claim_expanded_m1。
- `cleaned`：wt-guard clean（0 reparse point）后删除唯一树D:/autostack/.wt/lang-999/auto-lang与分支codex/plan-999-dev、空组目录；plan-artifacts/plan-999/整目录删除（证据已全量折叠进本Plan §5.6/§9/T记录；无具名未解决问题hold）；.retention/policy.json登记刷新。输出根对账：本计划命令实际使用过的输出根=plan-artifacts/plan-999/*（已删）、两仓target/*（随worktree/主检出常规构建缓存）、系统临时auto-case run目录（runner自管理）——无散落第二证据目录/外部盘符副本。生产工件观察项：auto-lang无daemon/web bundle；主检出release worker（av/mn/a2x）随smoke检查重建过debug档，release档如需由后续性能/发布任务重建——非本计划交付阻塞。
- 继承红与余账：av collections small-stack×3、native allocation_controls（状态相关）、934金样旧债（已由本包代修efce1ead9）——均master基线复现归档于T-09记录，归对应owner域。
- `completion_kind: delivered`；本goal链（work→review→merge→archive→cleanup）全部checkpoint有commit/磁盘实证。

## 10. 待澄清事项

1. 规划范围已明确，没有新增语言范围选择待问；后续work需要实施指令，本轮不推定施工授权。
2. Core事实整链/深输入预算可行性、签名持久版本与缓存依赖上下文由T-01/02有界证明；技术未知不自动延期八格/F13/可达缓存，失败保原目标needs_replan。
3. 941符号释放/稳定接口是生产修改接缝条件，不是整包依赖，也非本轮开会话/发消息授权；未来入场核实际SHA/owner，不接管其树。
4. 若新配方必须扩大兼容/消费者或预算，先具体修订并按既有授权判断；不得默删旧页、fallback旧推导、撤守卫。
5. 本域交付与941原八格组合已转绿分态；其余required/M2边界与最终M1结论均由真实组合决定。
