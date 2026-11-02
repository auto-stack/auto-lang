---
plan_id: PLAN-913
status: drafting
feature_name: 闭包源码来源绑定恢复与前置生产收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-ac/artifact-contract.md, docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-ac/source-closures.md, docs/specs/auto-av/cache-and-conformance.md, docs/specs/auto-av/resident-function-install.md, docs/specs/auto-av/closure-environments.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-av/project.md]
new_spec_components: [docs/specs/auto-av/closures-source-recovery.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-av, auto-test-case, artifact-cache]
depends_on: [PLAN-912, PLAN-909, PLAN-783]
current_step: 2
total_steps: 12
---

# [PLAN-913] 闭包源码来源绑定恢复与前置生产收口

## 0. 变更摘要

承909已有A23读取/ABC重验，消费[912单函数生产](912-closures-function-production-closure.md)正式交付的accepted source state，关闭R3：新进程不再先source compile/lower/encode求flat内容键，而从调用方已授权的来源manifest绑定进入，完整重验后执行；恢复出的安全上下文还须支持下一次真实单body编辑。

本轮拆包决策见[Design43 §16](../design/43-ac-av-typed-mir-and-abc.md#16-909后继闭包源码生产粒度与恢复入口2026-11-02target-state)。912关闭R1/R2/R4，913独立关闭恢复入口/R3。drafting r1、0/12，**本轮只规划**：不建树/材料、不构建/测试/探针/认证，不改production/current Spec/源/expected/binding/911或912合同。codec/反控可先设计，实际集成与完整验收以D912为前置，不能消费WIP或只占接口名。

非目标：完整source compiler/cache平台、层级号重排、新数据库/远端签名信任平台、活env/cell/帧序列化、持久闭包值、跨线程/循环GC、native/A2R恢复、默认迁移/完整AJ、M1签章、D-909-1 native旧红修复。数据命中不缓存PASS，不用来源hash自报代替授信。

## 1. 目标

- G1：同source/context的授权来源恢复P源生产=0，真正read/decode/结构/类型/资源/权限重验、link/admit并执行原source语义；不重lower/encode再称hit。
- G2：source→code关系由调用方保留的上次真实成功生产/commit根绑定授权，manifest/index/payload皆未授信；错/坏/漏/跨域/陈旧上下文不能执行。
- G3：恢复后安全frontend context由真实验证路径重建，下一次合法单body编辑沿912强门，未改函数P/工件IO0、同新进程内对象/代际；不能跑一次恢复后再whole重建。
- G4：required闭包、发布/租约/GC/只读/取消/并发与最后owner、预算/时间/父证据均可验，失败修复后下一次独立恢复真命中。
- G5：保持909/912语义及既有A23/其他profile合同，和911按真实S/H窗口；恢复绑定不带oracle/guest value或受信Checked对象。

## 2. 架构方案

`912已接受source+函数/code/deps只读state → 暂存所有required数据 → 原子发布source recovery manifest/root → parent持有强摘要绑定 → 新进程校验调用方anchor+source/context → 未受信metadata/HIR/MIR/ABC深验/重建凭证 → 安装/执行 → 继续单body候选`。

首选直接按调用方expected manifest强摘要读取，不必先造可写source索引；若有source-only索引，它只是hint，不能自授权。无可信绑定的原apply_source保持明确cold行为，不冒纯恢复；显式require-restore遇缺/坏/旧数据明确MISS/STALE/REJECT，不自动启动compiler掩盖。冷重建必须独立选择/记录，再验证下一次纯恢复确实免生产。

## 3. 技术栈

既有ace-artifacts ManagedStore/codec/required-edge/lease/root、av closures session/A23 envelope/ABC验证、912受检context/data view、AutoDown/TestCase及固定组。source recovery recipe/schema在本域注册，**不抢全局cache层号、不改A00–A23旧义务**；无新store/通用调度器。拟新私有模块`av/src/closures_source_recovery.rs`及codec/store测试由T-02..05启用。

## 4. 需求分析与背景调查

### 4.1 授权与实际前置

2026-11-02用户授权909后继正式规划及拆包，要求真实零生产/恢复/预算/时间/911边界。依本轮实际链拆912、913，两者均必要执行合同，不预占额外编号；本轮不施工、不联系/创建会话或代理、不清理他方材料，不改已归档Plan。

main `5a4eb4e257c4f5f4305f670bb52db482dc7e0917`；D909 `59b8107eb`为HEAD祖先(exit0)、12/12已归档，K06原源42/Shared tag4/cell/clone语义与现有restart重验已交付。R3仍需编译source/lower/encode求key。912目前同轮drafting0/12，**其目标不能当已实现API/Spec**；913依赖D912或明确发布并有验证的稳定接口阶段，但完整交付/最终门须接受正式D912基面。911实际树存在且用户确认实施，header滞后不改；910构建治理非语义前置。

### 4.2 来源与缺口

| 当前来源 | SHA256 / 事实 |
|---|---|
| auto-av/cache-and-conformance.md | `8d895b127203b125874302a64f5ec1eef6a528be233042ca48de757849f946e4`；restart有A23命中/深验零写，未免前置lower/encode |
| auto-ac/function-incremental-source.md | `25afaa44f2bfa65f76bb8cd818c6a387a08d44c220d076d1d0dab863a6233de0`；当前owner cache，912才将补单函数凭证/生产边界 |
| auto-mir/closure-capture-profile.md | `a9cb0b08a1c5cc7d1fa3c507fb3613399da4cf78cb2cb71d39bebfc40c7b852d`；Shared/env/间接调用/链接/所有权必须深验 |
| av/src/closures_865_support.rs | `e6ab4e46f946717cd8c98c00dc2014a478907ac634bf1aa34616da1b6c7f2da1`；new丢frontend/current_wires，apply_source先编译/逐单元encode后调用update |
| av/src/closures_session.rs | `9797e09aa8ab3257951efeeae642dc38b88a34b2b5bcc2a4d35d5a71211ba4c0`；closures_unit_key的unit-content=domain_hash(flat)，另核producer/owner/profile/source-snapshot/codec×sem，当前恢复不能在flat未产时算该键 |
| auto-ac/artifact-contract.md、av/cache.rs及ManagedStore | 统一carrier/required与provenance、root/租约/GC已存在，来源lookup不能另造DB/永久root或把provenance假充required |
| 912拟accepted source state | 本轮未实现；实际类型/身份/所有权/阶段/预算与消费者合同由912 T-02/05/merge冻结，913 T-01重核，不据草稿反向要求912额外语言面 |

本轮未重跑909验收或恢复探针，原raw材料已清，后续用当时冻结代码真实生成，不恢复历史附件。已有R5坏载荷/只读例不能单独证明source→code来源或无前置生产，必须新入口端到端验证。

## 5. 详细设计

### 5.1 来源绑定与信任边界

第一次由实际编译/verify/admit/commit形成912 accepted state，913发布对应manifest。调用方/parent从这次真实生产保留**预期manifest ID/强摘要与逻辑owner/source/config**；绑定是不含结果的来源授权，不是oracle或cached PASS。后续新进程带此binding+实际源/配置；source全文SHA、owner/module/entry、profile/语义/schema、caps/config、compiler/producer与实际读取依赖必须匹配。`source_path`仅既定逻辑来源/诊断映射，不凭路径未变、FNV或payload自报source_sha信任。

source-only lookup/index/旁路JSON即使自洽也不提供锚点。攻击控制重写索引/manifest/payload全部自描述hash，但不改变parent已保留预期root digest，仍必须拒。不能让worker先读一份未知manifest，再把它的hash自报为“expected”。无预期绑定的兼容入口明确cold；require-restore缺绑定/缺数据为可定位MISS/REJECT，不偷偷重编译或执行未知code。首次producer有权产生代码的边界沿现有编译器/受控worker，不引入全局密钥/远端签名服务。

新context跨进程重新mint env/session/候选授权，不能把持久env_id/generation/checked=true当活进程授权。语义logical ID稳定，host pointer/Arc地址不是持久身份；旧进程候选恢复后foreign拒绝。实际读的type/meta/实例/CT/effect/权限版本变化即使源未变也STALE；未读事实变化不无谓失效。需要重算CT/源语义才能重新建立关系时显式cold，不把compiler工作改名为恢复。

编译语义配置与run/restore/read-only等传输/存储策略分开：后者不能令原本合法的同一source-code binding天然不相等，但授权仍逐请求核。manifest记录权限需求，不持久化grant本身；当前host授权撤销必须拒。producer/工具身份按实际有效语义/版本和父观察校验，不以固定family字符串代替完整身份；版本/真实worker变化依原STALE规则重核，不伪装同一构建。

### 5.2 有界manifest与required闭包

T-02冻结本域schema（拟`ace.closures.source-origin/1`）、版本化codec/query/授信边界与cap。至少含：source/逻辑owner/entry/profile/config与producer版本；root生成代/accepted state摘要；完整函数/decl/type/binding/lambda目录与单位code/HIR/MIR/ABC digest/identity；所读依赖和授权需求；source/mint/current坐标/映射；恢复下一次编辑所需的安全元数据。所有引用有精确kind/owner/role/hash，名字/类型图用语义身份，不让VT::Function内部索引跨表误用。

source/语法或编辑定位元数据、函数context/body、必要IR/ABC unit及其类型/env/cleanup数据是required；运行/下一编辑必需数据不能标provenance后任GC。benchmark/旧执行日志/可重建展示可为provenance，不被运行读取。不能保存trusted CheckedFunction/CheckedModule、live cell/env/borrow/host地址或guest value/PASS；解码数据从未授信状态进入A的真实验证构造，不能`deserialize(...verified=true)`或重切旧凭证。

manifest单件≤4MiB，source≤4MiB，自己管理的恢复namespace≤128MiB（含其required backing按统一去重/保守账，不清其他项目/其他plan store）。字节/条目/图遍历/字符串检查先于无界物化；有限结构/显式栈、unknown kind/schema/重复逻辑ID/类型图环/非法range/越界引用/错role/同context异body等fail closed。合法非零ID和深链正控保留，不能靠拒所有新ID过门。

### 5.3 发布、崩溃与GC

prepare期间所有新required数据有临时OS租约，校验完后原子发布完整manifest/root；任何时候可见的active recovery root都能闭包读取。不能先写root再补chunks。取消/进程退出/写盘失败不改上一有效恢复root，不遗留永久candidate pin；下一次读旧root、冷重建/重试发布再独立恢复都必须真验。

**source/code commit与recovery publish阶段分别记状态**：若A已接受新源后B发布失败，不能谎称A也回滚；旧recovery binding仍对应旧源，新源不得借旧binding命中。错误结果明确source已提交/新恢复root未发布，不给新有效ticket；可以从同一不可变accepted state重试发布而不重跑source/lower/encode。活项目root更新须检查accepted generation/CAS，慢旧发布不能覆盖更近源；需要历史ticket时显式历史授权，不偷改active root。可选hint写失败不让自洽hint覆盖authoritative root，当前ticket是否成立以完整root为准。

root/required边与租约保护当前/候选/正在恢复的reader/授权pin；GC并发只回收真正无引用且未租约的对象。close释放本session/root/临时引用，外部授权pin到最后owner后才计释放，不靠history长度推断内存/磁盘无存活。保原code8MiB/每unit32及cell8MiB等cap，不做全局清仓/按时间盲删root。只读恢复不得为MISS/坏件重建/改root/metadata；需lease的控制动作与artifact数据写分账，严格FS只读正例用预先授权root/安全read lease，不能把无安全lease的拒绝计作正例通过。

binding是来源授权/身份，不承诺永久保留全部历史数据；当前durable project root及显式保留的root/lease才保护required闭包，已无引用历史被合法GC后明确MISS，不以“ticket仍在”执行缺件code。512修订试验释放未保留旧租约并核真实watermark；需要保留的外pin计入预算，不能为通过128MiB门清除仍授权的pin。增量发布只序列化/写改变的body/context/code数据，原node引用复用；root目录/坐标delta可列M轴，不能把全部未改body重编码/写盘藏成metadata工作。

### 5.4 新进程重验与下一次编辑

新进程按expected root取manifest→核source/config/tool/实际依赖→遍历required→检查hash/envelope与source/body/context关联→decode HIR/必要MIR/ABC→真实结构/类型/资源/capture/env/授权验证→新process凭证/FunctionCode/安装→实际执行。错owner/profile/codec/语义版本、Shared tag/env形状、内部payload正确但绑定错误、权限撤销、丢失/截断/回滚数据在执行前拒、危险访问0；bad envelope不等cache miss，更不能silent cold后写PASS。

保存的代码/源字面量和合法编译常量不是“缓存运行结果”；禁止使用上次guest观测value/oracle/PASS跳过这次执行。正例同时观察真实AV dispatch/cell-env分配/释放或对应trace，强制返回旧value、不进execute及忽略本次budget/cancel的控制均须红，不能仅凭两次都42当恢复闭环。

恢复未信任HIR的函数验证/ABC准入是R轴，可非零；source compiler parse/resolve/typecheck/Atom plan/bind/HIR构造、MIR lower/ABC encode属于P轴，require-restore正例全部P=0。不能通过先调用lower/encode计算旧flat key、先编译取body digest，或把compiler函数改标签为revalidate过门；实际生产dispatch与R decode/verify输入角色分别计量。

必须恢复912下一编辑所需的安全frontend context/源索引/依赖与身份，**下一次**改一个lambda后同owner兄弟/其他函数全部P/IO0，同新进程内HIR/MIR/code对象与代际保持。跨进程仅比较logical identity/content，不能拿不同进程Arc指针相等作假oracle；旧进程live cell不复活，新执行从原初始化值开始。恢复失败/依赖过期后选择cold修复，再全新进程用新binding恢复，也要真实P0，不能只证明当前fallback能跑。

### 5.5 API、文件化与正式验收

复用既有closures source recipe/worker/adapter及source-state data，T-02冻结显式restore binding输入与返回shape；若当前协议无法承载，则新增本域可选字段/明确版本并同步三缝consumer，不造runner/通用report。原未带binding source入口保cold兼容；任意mode/版本或错recipe拒，old/new schemas不互授信。请求只有源/配置/来源绑定，不含oracle/期望value/parent PASS；从第一次真实生产的返回artifact获得binding，parent独立hash后进入另一进程，不预置一份“必过ticket”。

parent在同一check的有限状态中取得/核binding并转发下一真实worker进程即可，不引入跨case变量语言或通用工作流DSL。T-02必须用实际现行decoder/adapter/worker证明能完成两进程握手，新增字段/step由T-06启用；不能用手写脚本先编译出一份有利ticket再声称正式入口已闭。

拟长期`ace-m1/closures-recovery-913.case.ad`/suite与独立reference/反控由T-06启用，引用原K06及909共享/clone源、912同owner函数编辑语料。必须实际不同PID新进程、原source手算值/trace/trap/资源oracle；current repeated run不冒restart，模型codec测试不冒真实restore。root/producer/source/schema/tool等父证据hash复核，零选/重复/漏项/SKIP/旧counter不绿，case/manifest身份变只重核受影响。不改旧原源/expected/binding/NA/认证分母，不生成a2r金样/证书，不签M1。

### 5.6 资源、时间与观测

继承912 retained前端合计≤16MiB（current/candidate/history/pin/intern/索引全部）、source/单manifest≤4MiB、恢复namespace≤128MiB；原code current+history8MiB/每unit32、value8MiB、verify/decode各16MiB更严门不加大。恢复decoder/required遍历/索引/lease/原子发布临时数据也计账；父private/working-set按原更严门且相对冷基线增量≤128MiB（拟合同），最后owner与GC释放单列。T-02最小链先证资源可行，不能靠隐藏table/全量载入或提高cap。

P源生产/R持久重验/M源hash/元数据/坐标三轴按实际函数/whole和失败出口记attempt，IO分read/write/root/lease；readonly数据/root写0不冒整个控制层无任何写动作。fresh=实际冷生产，hit=驻留复用，restart=真正新process来源绑定重验，body-edit=restore后真实单体生产；各30原规模样本，P90≤2000ms与独立语义/零P/资源/下一编辑同过，原更严门保留、长期亚秒目标不变，准备/冷编译/发现/执行/清理/整命令分列。

512修订/历史+GC/多caller租约、不同时source/config/旧根/取消后恢复/损坏修复/严格只读/无anchor cold控制都要区分。强制旧source compile/lower/encode放进lookup前、隐藏到R标签、只比较domain_hash等错误路径使门真红；不得为取恢复证据多跑一次compiler或缓存运行结果。原深链/预算/受控进程规则保留，不全跑13慢例或统一三warm矩阵。

正式新规模接受912冻结的至少64顶层函数/每被测owner3兄弟lambda及同源手写reference/hash，恢复后编辑首/中/末lambda；保原909样本/资源机制，不能为新进程P90换小源。T-02记录实际source/unit/required字节/读取与验证次数，T-07四状态各30样本沿同一规模；512 churn/深链累计墙钟另列，不当一次恢复时间。

### 5.7 912/911与公共面交接

| 面 | 913主责 / 前置与释放 |
|---|---|
| D912源契约 | 接受正式source-state/凭证/context/逻辑ID/依赖/阶段/资源合同与SHA；不反向消费drafting目标，变更先和912作者核接口，不默复制第二套frontend |
| P-recovery | closures来源manifest/query/codec/required/root/lease/restore路径；尽量独立私有模块；不改统一ManagedStore全局语义或破旧A23 envelope |
| S-shared | core untrusted→checked context构造、verify计量、closures_session恢复/只读入口；912先释放正式接口，911若改失败事实/attempt则同源符号短窗，不改其三族/首失败/原默认合同 |
| H | ac_av/contract/worker模式/工具版本、groups/SDK、Spec/manifest；911拥有native诊断及B17–19，913只占闭包恢复相关字段，整个文件单写，不能认为不同键可并写 |
| runtime | 既有Shared/private/borrow/clone/cleanup/活env语义只读兼容；持久没有live cell，恢复不能冒866在线HMR或帮native旧红造PASS |

窗口收据含owner/revision/正式SHA/字段/类型/构造与全部consumer、版本/根/source/config/工具身份、已跑/pending/释放。实际T-01接受D912，独立codec/负控可先但不算完整链通过；无需D911/910整包，相关S/H未释放只等实际任务。跨接口变化STALE受影响证据重核，最终review绑定实际整合SHA；实际采样错峰、不持源码窗等机器，不联系/停他方进程。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/closures-source-recovery.md | 无→host来源绑定/manifest/深验/下一编辑/预算与阶段 | source-only hint不授信 | AC-01..05 |
| SD-02 | modify | docs/specs/auto-ac/artifact-contract.md ; docs/specs/auto-av/cache-and-conformance.md | 按flat求key前先生产→来源root/required闭包入口与原cache兼容 | 关闭R3、不另造cache层号 | AC-01/02/03 |
| SD-03 | modify | docs/specs/auto-ac/function-incremental-source.md ; docs/specs/auto-ac/source-closures.md | 912进程内状态→未授信恢复/新凭证/下次编辑 | 不反序列化authority或whole重建 | AC-02/04 |
| SD-04 | modify | docs/specs/auto-av/resident-function-install.md ; docs/specs/auto-av/closure-environments.md | 原code安装→来源binding/last-owner/无live env恢复与phase分账 | code与运行值/活pin分离 | AC-03/04/05 |
| SD-05 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | 原恢复收据→首例/实际新PID/P0/重验及反控，原oracle不改 | 独立关闭门 | AC-01/05 |
| SD-06 | modify | docs/specs/auto-av/project.md | 当前导航→来源恢复接口/信任边界/未关项 | 可独立续接 | AC-05/08 |

## 6. 测试设计

拟新source恢复suite与case由T-06接入既有TestCase；Rust在现有artifacts/av conformance目标新增私有cases，不拼不存在的独立target。真实变异在唯一树/材料副本隔离、证明门红后恢复，不改原oracle。以下全部待执行。

| V | 实际输入/入口 → 独立oracle与区分性反控 | AC / owner_stage |
|---|---|---|
| V-01 | 第一次真实912生产→parent持root摘要→新PID恢复K06/共享/clone | 同源42/82/44等原oracle，P全0/R实际>0、AV dispatch与资源/trace真实；旧lower/encode求key、不execute返旧value或忽略本次budget/cancel反控红 | AC-01/05 work T-02/06/07 |
| V-02 | 同源config/owner/entry/profile/producer变、read type/meta/CT/effect/cap变、未读事实变 | 实际读者失效/STALE，未读不误失效；同调用点源码不充hit，合法identity正控过 | AC-02 work T-02/03/05 |
| V-03 | 索引/manifest/payload全改自描述、host expected digest不变；source-only无anchor | 拒绝伪造绑定；无anchor明确cold/require-restore拒，不能自报expected或算恢复命中 | AC-02 work T-02/03 |
| V-04 | 漏required/截断/错kind/跨profile/Shared env/合法payload错source/body/context | 准入前拒/危险访问0；合法非零IDs/未知版本反控分态，不当miss重写PASS | AC-02 work T-03/05 |
| V-05 | decoded前端/context/HIR/MIR/ABC深验、伪checked=true/旧env_id/foreign候选 | 数据先未信任，凭证真实重新mint；旧候选拒绝；R非零如实，source compiler改名R反控门红 | AC-01/02 work T-02/05 |
| V-06 | 新进程恢复→立刻单lambda edit，另一同owner与其他fn未变 | 原预期结果变化，未改P/IO0且该进程内对象/代际恒等；丢frontend state/下一改whole反控红 | AC-04 work T-02/05/07 |
| V-07 | 每发布缝fault/取消/进程退出、A已commit但B publish失败、旧代慢发布 | active恢复root无半闭包；不假称A回滚/不给新ticket；旧根可读、重试不再编译、CAS不覆盖新源 | AC-03 work T-02/04/07 |
| V-08 | reader/OS lease/active root与GC并发；close及外pin最后释放 | 读者/required不被删，最后owner后可回收；死candidate无永久pin；不清history向量假记0 | AC-03/05 work T-04/07 |
| V-09 | 只读store正例、miss/坏件、read lease/control动作 | 数据/root/修复写0，命中真实执行；缺安全租约明确错误不充正例；不在readonly silent cold/修复 | AC-01/03 work T-04/07 |
| V-10 | 坏件拒→明确cold重建/修复→再新PID来源恢复 | 后一次真P0/R深验且执行对oracle，不能只证明当次fallback可跑 | AC-01/03/04 work T-05/07 |
| V-11 | guest多次修改cell后close/restart、合法borrow/clone/退出 | 重启从原初值、live state/PASS未持久；env/cell恰清理，原Str Borrow/owned门保留 | AC-04/05 work T-05/07 |
| V-12 | full manifest/namespace/16MiB context/旧8MiB code等边界，512修订/深链与GC | 峰值/bytes真实有界，超额Resource保旧root/源；不旁路大map/全量载入或删其他scope | AC-03/05 work T-03/07 |
| V-13 | parent source/anchor/artifact/tool/schema哈希、零选/重复/旧report、mode错recipe | 正式证据读回准入，缺/伪/STALE不冒PASS；默认source cold兼容、请求无oracle | AC-02/05 work T-06/07 |
| V-14 | fresh/hit/restart/restore后edit各30样本，原规模/对照与父峰值 | P90≤2000ms与P0/R实计/语义/资源同时过，准备/生产/重验/清理分账，长期亚秒保留 | AC-01/05 work T-07 |
| V-15 | 912正式接口、911失败/计量与SDK/其他producer默认consumer | actual SHA/阶段/只读数据契约与全consumer齐；接口再变STALE受影响重核，无重复编译/另造runner | AC-04/05 work T-01/07 |
| V-16 | publication read/报告artifact写失败/部分绑定、下一次重新读取 | 不交不完整binding，parent实际rehash与source/工具关联，重试后真实restore可用；compiler与报告材料不跨scope | AC-03/05 work T-04/06/07 |

完整职责初选`cargo t ace-artifacts ace-av ace-source ace-lowering`（实际context授信触面具名补`ace-semantics`），宿主/decoder触面补`ace-host ace-adapters auto-test-case`，registry选`infra-fixed infra-ace-m1`；原`case-ace-m1-closures-865-source case-ace-m1-shared-mutable-909`与正式912 source组/拟`case-ace-m1-closures-recovery-913`（all=false、显式ACE，T-06注册）。共享凭证/HIR有真实下游变化补`ace-native ace-a2x`完整消费者，未触不自动全仓。正式计量沿`ace-performance`本包命名case/原条件，保Job/内存/ignore/平台样本，不重跑13慢例；最终并集以actual diff冻结。

内部自查与同树独立上下文final review分开，未指定人员/模型不创建agent/chat；相同输入/身份有效证据有据复用，修复/接口/配置变仅受影响重验。首红/环境/重放/未执行不合并成绿。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01/05/09/10/14：授权root新PID恢复无任何P源编译/lower/encode，R真实深验/执行/readonly成立，old路径/改名R反控真红，后一次恢复仍真P0 |
| AC-02 | work | V-02..05/13：anchor/source/owner/config/producer/实际读依赖与required全核，无自授权index/同context异body/旧凭证/越域执行；unknown与合法正控分明 |
| AC-03 | work | V-07..10/12/16：source commit与recovery publish状态诚实、root原子/旧代CAS/lease/GC/只读/取消/预算与最后owner，坏后修复再读可用 |
| AC-04 | work | V-05/06/10/11/15：安全context重新授信、下一body改按912强门，未改P/IO0和对象恒等；不恢复live env/PASS、原闭包语义不变 |
| AC-05 | work | V-01/08/11..16：文件化首例/独立oracle/父证据/全部职责，30样本P90与预算/阶段同过，912/911接口/H/SDK与SD-01..06/残余owner齐，不签M1 |
| AC-06 | review | 同树独立逐AC-01..05/V-01..16/SD/原源/身份/反控/遗漏延期/健康核，revision+冻结SHA pass，不以模型codec/序列化成功冒真实恢复 |
| AC-07 | merge | 冻结职责/必要consumer门或有效证据、真实合入/Spec读回/归档、所有材料/guard清理、12/12真实 |
| AC-08 | merge | A/B关闭点分别读回：R1/R2/R4是D912，R3由本包实际证据闭；既有C-19/911/旧a2r/D909-1另保，不重开旧Plan或写旧ledger |

## 8. 执行步骤

8 work+1 review+3 merge=12项，全未勾。接口/信任/预算在最小真实源消费后冻结；不能先造source-only信任的cache平台再加安全性。若需通用签名/远端平台或扩大前端语言则needs_replan，不删R3与下一编辑门。

| T | owner_stage | 前置/窗 | 工作与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令；D912实际接口；911具体S/H | 冻结正式SHA/数据/阶段/cap/所有consumer、owner根与无anchor策略、唯一树/材料及窗口；可先codec/控制设计但不冒完整前置已齐 | AC-01..05 | [x] 2026-11-02 树`D:/autostack/.wt/lang-913/auto-lang`@`plan-913-dev`基`master@aceed7176`（组内 auto-down detached `7c4d5cb` 只读）；材料`D:/autostack/plan-artifacts/plan-913/`（README 登记 owner/用途 + `t01-freeze.md`）。**D912 接受**：交付`13a465336`为HEAD祖先(`git merge-base --is-ancestor` exit0)、912 archived 12/12；实读并冻结`AcceptedSourceState`（`ace.closures.accepted-source/1` 字段全集/数据视图≠凭证）、`edit_body`+`WarmGateReport`、`ClosuresFunctionFrontend`、`ClosuresUpdateSession`（真实门序 verify_mir→encode_abc→store lookup→verify_abc_bytes→admit；R3 根因=`closures_unit_key` 的 `unit-content`=domain_hash(flat)，任何命中前必先 encode）、`ManagedStore`（put/get_by_id/get_artifact_bounded/register_root/root_artifacts/remove_root/acquire_lease 的"先注册新 root 再摘旧"语义）、`ClosuresVerifyContext`（from_checked/lambda/sites_of/overlay_from_batch）。**SHA 冻结**：17 份源/Spec 实读（freeze §2）；`av/closures_session.rs` 实值`f9465d25…`≠计划§4.2旧值`9797e09a…`（912 已加 `artifact_of`/`revision`/accepted-state 支撑，差异如实记录），`av/closures_865_support.rs` `e6ab4e46…` 与§4.2 一致。**机制冻结**：manifest schema`ace.closures.source-origin/1`（本域私有层 id，不抢全局 cache 层号、不改 A00–A23）；发布=prepare 期租约→原子 `register_root(project,manifest,"source-recovery-current")`→摘旧 root；恢复=父 expected root artifact id+强摘要→按 id 直读 manifest→逐单元 envelope→`verify_abc_bytes`→`admit_unit`→新会话 `restore_install()`（不 verify_mir/encode_abc，P=0/R>0）→link/execute；下一次编辑=core `ClosuresContextDump`+`restore_from_dump`（未改单元 signature-only 空 body 仅作脚手架，dump 只作未授信输入，须与解码已验单元事实交叉核对），等价 oracle=同源 spanless wire 对冷整模块字节相等。**消费者初选**：`ace-av ace-core ace-source ace-lowering`，用例面 av/core 的 913 相邻新增 + 正式 `case-ace-m1-closures-recovery-913`。**建组插曲**：`bash` 实为 WSL bash，worktree 管理文件写成 `/mnt/d/...` 致 `prunable`/`not a git repository`，已改写为 Windows 路径修复，后续脚本（含 merge 的 wt-guard）改用 Git Bash。主检出预检：无 `crates|experimental|test|tests|auto|stdlib|scripts` 改动（仅 docs 簿记）。开放点 4 项（脚手架字节等价/空 body 下类型判定/registry 由 MIR 还原的 cell+mutable 位/同进程对象代际）列 freeze §7，由 T-02 实证，不当豁免。 |
| T-02 | work | T-01/D912；最小P/S | 原K06真实A生产→parent anchor→新PID无P深验运行→下次lambda edit，含错anchor/旧lower负控、readonly与发布失败最小链；冻结schema/required/授权/资源/提交边界 | AC-01..05 | [x] 2026-11-02 commit `453dd7a36`+`cef3012b5`+`4489fc97b`（树 plan-913-dev）。**链已通**：`av/src/closures_source_recovery.rs`（schema `ace.closures.source-origin/1`、私有层`A23-R`、root kind `source-recovery-current`；caps source/manifest/context 4MiB、wire 8MiB、units 4096）发布=put 单元/上下文/manifest→OS 租约→**先 register 新 root 再摘旧**；manifest 以 `RefKind::RequiredData` 闭包每单元 ABC + 声明上下文 artifact；每单元另存 **spanless identity wire**（base64+fnv，强校验==accepted `mir_identity`）。恢复=`ClosuresRestoredSession`/`restore_source_session`：caller expected id+sha256 → 按 id 直读 manifest/context/units → 请求侧 envelope+digest → 逐单元 `verify_abc_bytes`+`admit_unit`（R 轴记 `RestartDeepVerify`）→ 由已验 MIR 造 `UnitEvidence`（lambda env arity、creator 站点 callee+mode 序列）→ `restore_context` 重建安全上下文 → `ClosuresUpdateSession::new(None)`+`restore_install`（无 verify_mir/encode、不写 store、close 不碰 store）。**实测 10/10 PASS**（`cargo test -p ace-av --test conformance closures_recovery_913`）：不同 PID 恢复执行 51 且 `p_axis_attempts=0`/R 计数真实（含 context_reads=1）；**同新进程内紧跟 g body 编辑**=48、route fast-unit、submitted=1、whole=0、warm gate pass、body work 仅该单元、`unchanged_production_attempts=0`、未改 4 单元 Arc 同对象、改者新对象、store_writes=0、live=0、allocs==frees；**旧 lower 负控**（无 anchor 重启=verified/encoded≥5、cache_hits 只在 verify+encode 之后）；**错 anchor/缺 root/异源/异 owner/空 artifact/缺 required 数据(purge A23-R→miss)** 均具名拒；**发布失败保旧 root**（失败尝试后仅 1 root 且仍恢复 51）；二次发布仅 1 current root 且新旧 manifest 各恢复自己绑定源。`cargo t ace-source ace-lowering ace-av ace-artifacts` 见 §9 收据。**边界冻结**：恢复上下文 owner 路由=具名非快`RestoredContext`→整模块 truth；恢复会话不持久化（持久化/再发布属 T-04/T-06）；dump 的 cell/mutable 位仅经站点/mode/arity 交叉核对 |
| T-03 | work | T-02；私有codec/store | manifest/query/稳定身份/实际依赖/有界解码、lookup仅hint与parent预期绑定/required闭包验证；V-02..05/12 | AC-01/02/05 | [ ] **进行中**（2026-11-02 commit `99509dfba`+`90a10f873`）：**可移植声明上下文**已落 `core/src/source/closures_context.rs`（schema `ace.closures.context-dump/1`，caps units/fns 4096、表 65536、串 8192；serde 仅在新模块镜像 DTO 上，core 现有 AST/HIR/resolver 类型无序列化形态）。`dump_context` 出 units（wire 身份 `d_<name>`、kind/logical/站点事实/span/body_digest/mint_span）+每函数签名+resolver 五表（records/lists/variants/function_types/sigs/rec_index）+registry 声明与 creator→sites；`restore_context` 把 dump 当**未授信输入**：schema/source SHA/module/caps/bounds/空/catalog↔fn 配对/registry 重复均具名拒（`restored.stale|source-mismatch|context-mismatch|capability|bounds|empty|duplicate`），并**与已解码单元的 evidence 交叉核对**（lambda 的 MIR env arity==registry env 长、owner 的 decoded CreateClosure 站点 callee 顺序+capture mode 序列==registry sites_of+site_modes、owner 不得带 env），registry 由新 `ClosuresVerifyContext::from_declarations` 重建并重跑冷构造不变量。**等价 oracle 实测**：恢复上下文与冷态对同一 lambda 编辑产出**逐字节相同** scoped Atom 文档（含 receipt/def id 集合）。route 诚实性：新 `NonFastReason::RestoredContext`，restored 上下文对 owner 路由（需保留 body 复用）具名拒→调用方走整模块 truth。**实测**：`cargo test -p ace-core --test conformance closures_context_913` 4/4 PASS；`cargo t ace-source ace-lowering ace-av ace-artifacts` 4 组全 PASS（run-e9siguy2）。**未完**：manifest/query 侧 lookup-only-hint 与 required 闭包（units+context+identity wire 的 required 边）、V-02..05/12 的负控/边界用例、av 侧 dump 落盘与恢复接线（下一轮） |
| T-04 | work | T-03；root/lease P | 暂存/原子root/CAS/GC/并发reader/readonly/关闭、A commit与B publish失败分账，原受控故障与重试；V-07..09/16 | AC-03/05 | [ ] |
| T-05 | work | T-02..04；912稳定验证构造S/AV恢复P | 零P新进程admit/执行、真实context新mint与下一编辑、坏后cold修复再新恢复，V-01/05/06/10/11/15 | AC-01/02/04 | [ ] **进行中**（2026-11-02 commit `cef3012b5`）：零P新进程 admit/执行 + 真实 context 重建 + **下一 lambda 编辑**已实测（10/10，见 T-02 行与 `plan-artifacts/plan-913/t05-restore-then-edit.md`）；恢复上下文由 `dump_context`/`restore_context` 经已验单元 evidence 交叉核对后重建（core 4/4 等价 oracle：同编辑逐字节同文档）。**未完**：坏件后 cold 修复再全新进程恢复真 P0（V-10）、V-11 重启从原初值/活 state 不持久、V-15 912/911 接口消费者齐 |
| T-06 | work | T-03..05；worker/adapter/注册H | 显式来源binding输入/返回/parent hash与请求无oracle；首例/反控/独立reference/suite、原消费者/manifest hash保护，V-01/13/16 | AC-01/05 | [ ] |
| T-07 | work | T-01..06；正式运行窗 | V-01..16/真实强制红、完整组/必要consumer、四状态30样本/512/原深链/资源及readonly/pin/下一恢复；与911错峰 | AC-01..05 | [ ] |
| T-08 | work | T-07 | 逐AC/V/SD/来源hash/窗口/残余owner/健康自查，命令退出/关键数字折计划，冻结SD/提交/树clean→execution_done | AC-01..05 | [ ] |
| T-09 | review | T-08同树冻结SHA | 独立跟source→binding→root→untrusted→deep验证→下一编辑，审反控/信任/阶段/完整门与SD，修复重绑pass→reviewed | AC-06 | [ ] |
| T-10 | merge | T-09 pass | 冻结完整组/必要consumer门或有效同身份证据；batch仅due主检出单实例/既有用户择时，不额外全仓 | AC-07 | [ ] |
| T-11 | merge | T-10 | 真合入/SD-01..06读回、R3实际关闭与A/B/911接口/剩余owner交接，archive delivered，不签M1 | AC-07/08 | [ ] |
| T-12 | merge | T-11 | 摘要/hash/全部实际输出与hold对账，fresh wt-guard clean后清树/分支/组/整个scope及登记，真实12/12 | AC-07/08 | [ ] |

拟唯一树`D:/autostack/.wt/lang-913/auto-lang`、分支`plan-913-dev`，未建；work/自查/独立review同树顺序，不建第二树同步证据。开工唯一材料`D:/autostack/plan-artifacts/plan-913/`登记owner/用途/在途；实际parent/child测试store、探针/报告/崩溃副本/外部target全在此，长期src/oracle才入职责目录。禁止清全机TEMP/session store凑fresh、禁junction/跨他方store写。merge摘要折本计划，核全部实际输出后删scope/树中间物，具名未解问题才hold，不写旧JSON/attachments。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-913 r1 | outcome:pass（规划合同，非前置已齐/实施/最终review PASS） | 用户授权909后继并判断拆包，B专门关闭恢复前置生产而非A重复做前端。D90959b8107eb祖先0，实际key依赖flat、恢复前lower/encode已核；D912未交付，实际集成以正式接口为前置，codec/反控可先设计。

Design43 §16与索引先更新，再在allocation.lock排他下由new-plan.sh真实分配912/913，下一号914，无第三号。本轮只设计/计划/路线，不改producer/current Spec/原源/expected/binding/911/912/910合同，不建实施树/材料/跑构建测试探针认证/agent/chat。

规划静态预检（2026-11-02）：本计划0..10节、T12/AC8/V16/SD6定义及引用、12项全未勾、drafting r1/0/12、局部链接/架构anchor、现有Spec路径与未来Spec尚未建立均通过；912对应T12/AC8/V14/SD6通过。五份设计/战略/交接文档新增链接与15个现有固定组名称通过，新912/913组不是已存在入口。912/913各唯一、next=914，无新树/材料；binding SHA256保持8dea9b9fd98ad379d62dff51cce9074b7e4b434c2537cdeae331d072ea343422，current Specs/code/scripts无tracked diff。历史扫号遇既有archive/013重号，未改历史，已单独核新两号无冲突。只验证规划文档，不产生恢复/认证/P0/运行PASS。

2026-11-02 | stage: work | PLAN-913 r1 | outcome: in-progress（T-01 done 1/12；T-02 进行中） | worktree `D:/autostack/.wt/lang-913/auto-lang` branch `plan-913-dev` | base `aceed7176` | code `453dd7a36` | dependency: auto-down detached `7c4d5cb` 只读 | tasks: T-01 | evidence: `plan-artifacts/plan-913/t01-freeze.md`（身份/基面、17 份源 SHA、D912 接受面、机制与轴/caps/负控冻结）、`t02-recovery-entry.md`（发布/恢复实现与 6 例实测） | checks: `cargo test -p ace-av --test conformance closures_recovery_913` = 6 passed/0 failed（837 filtered）；`cargo t ace-av` 固定组 PASS（run-80f4d_7a，exit0） | notes: 建组经 WSL bash 生成 /mnt 管理路径已改写修复（后续脚本用 Git Bash）；912 交付面现有 3 条既有 warning（`closures_fn_session.rs` 的 env_layout 死赋值与未用 `mix_u64`、`owned_session.rs` 字段）非本包引入，健康项记录 | blockers: 无 | next_responsible_role: work（T-02 余项：恢复后下一次 lambda 编辑 → 需 core `ClosuresContextDump`/`restore_context`）

2026-11-02 | stage: goal | plan_id: PLAN-913 | plan_revision: r1 | round: 1 | from: drafting | to: work | status: drafting | outcome: pass | worktree: D:/autostack/.wt/lang-913/auto-lang | commit: aceed7176 | next: work

2026-11-02 | stage: work | PLAN-913 r1 | outcome: in-progress（T-01 done 1/12；T-02/T-03 进行中） | worktree `D:/autostack/.wt/lang-913/auto-lang` branch `plan-913-dev` | code `90a10f873`（T-02 `453dd7a36`；T-03 `99509dfba`+`90a10f873`） | tasks: T-01 | evidence: `plan-artifacts/plan-913/t03-context-dump.md`（设计依据=实读 resolve.rs:2267-2400/adapter.rs:516-549,3290-3367；dump/restore 语义、拒绝码、等价 oracle） | checks: `cargo test -p ace-core --test conformance closures_context_913` = 4 passed/0 failed；`cargo t ace-source ace-lowering ace-av ace-artifacts` 4 组全 PASS（run-e9siguy2，exit0）；T-02 的 `cargo t ace-av` PASS（run-80f4d_7a）与 av 6 例仍绿 | notes: core 增 serde derive（仅新模块镜像 DTO）与 dev-dep serde_json；core 现有类型未获序列化形态；恢复上下文对 owner 路由具名非快（不静默用空 body 复用）；worktree 已清 plan827 测试副产物（按精确路径，未误删跟踪文件） | blockers: 无 | next_responsible_role: work（av 侧接线：context dump + spanless identity wire 作为 required 数据随 manifest 发布；restore 由已验 MIR 生成 evidence → `restore_context` → 新会话入口 → 端到端"恢复后立刻单 lambda 编辑"）

2026-11-02 | stage: goal | plan_id: PLAN-913 | plan_revision: r1 | round: 2 | from: work | to: work | status: drafting | outcome: pass | worktree: D:/autostack/.wt/lang-913/auto-lang | commit: 90a10f873 | next: work

2026-11-02 | stage: work | PLAN-913 r1 | outcome: in-progress（T-01/T-02 done 2/12；T-03/T-05 进行中） | worktree `D:/autostack/.wt/lang-913/auto-lang` branch `plan-913-dev` | code `4489fc97b`（本轮 `cef3012b5` 恢复链 + `4489fc97b` 失败发布/旧路/缺件控制） | tasks: T-02 | evidence: `plan-artifacts/plan-913/t05-restore-then-edit.md`（required 声明上下文+identity wire 的发布/恢复语义、10 例断言、边界） | checks: `cargo test -p ace-av --test conformance closures_recovery_913` = **10 passed/0 failed**（含 restore→下一 lambda 编辑：P=0/whole=0/warm gate pass/`unchanged_production_attempts=0`/未改 4 单元同对象/改者新对象/store_writes=0；旧路负控 verified+encoded≥5；purge `A23-R`→miss；失败发布保旧 root 且仍恢复 51）；`cargo t ace-source ace-lowering ace-av ace-artifacts` **4 组全 PASS（run-s9bhz4nj，exit0，clean tree）**；core `closures_context_913` 4/4 仍绿 | notes: 上一轮同集运行 exit1 系运行期间工作树被我编辑（脏树），清树后同码 exit0，本轮全程未在运行中改树；R 轴改用 `Stage::RestartDeepVerify`（不占 P 轴 DecodeAdmit）；恢复会话不持久化故编辑产物不落盘（T-04/T-06） | blockers: 无 | next_responsible_role: work（T-04 根/租约/GC/严格只读/CAS；T-03 余项 lookup-only-hint 与 required 闭包边界；T-05 坏后 cold 修复再恢复 V-10）

2026-11-02 | stage: goal | plan_id: PLAN-913 | plan_revision: r1 | round: 3 | from: work | to: work | status: drafting | outcome: pass | worktree: D:/autostack/.wt/lang-913/auto-lang | commit: 4489fc97b | next: work

next: 后续实施指令后T-01，接受D912数据/阶段/预算与consumer收据，再T-02最小来源绑定→新PID真实重验→下一编辑；不以无anchor自洽缓存过门。所有V/AC待执行。

## 10. 待澄清事项

| D | owner/入口 | 有界关闭条件 |
|---|---|---|
| D-01 D912与来源authority | T-01/02，913 | A正式接口/父expected root授权与无anchorcold策略冻结，不消费草稿/自报expected，不造全局签名平台 |
| D-02 codec/required与重验 | T-02/03/05 | plain数据未授信、全部角色/版本/实际依赖与上下文重建，禁止deserialize Checked权威、源重编译改名R |
| D-03 发布/只读/GC | T-02/04 | A commit和B publish分别诚实、root原子/CAS/lease/只读正例真实，无失败后半ticket或永久pin |
| D-04 时间/资源 | T-02/07 | 16MiB front/4MiB manifest/128MiB自有namespace及原更严cap、30样本P90与P0/R/M同时证，放宽先修订 |
| D-05 911/公共H与残余 | T-01/07/merge | typed失败/attempt与字段consumer具名窗口、接口漂移STALE；C-19/911/旧a2r/native红另owner，不冒M1 |
