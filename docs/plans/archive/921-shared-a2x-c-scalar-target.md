---
plan_id: PLAN-921
status: archived
feature_name: CA-A共享A2X目标合同与HIR到C标量闭环
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-02
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-a2x/project.md, docs/specs/auto-a2x/architecture.md, docs/specs/auto-a2x/incremental-and-cache.md]
new_spec_components: [docs/specs/auto-a2c/project.md, docs/specs/auto-a2c/scalar-hir-backend.md, docs/specs/auto-test-case/a2c-hir-scalar-certification.md]
touched_goals: [GOAL-002]
affects: [auto-a2x, auto-a2c, auto-test-case]
depends_on: [PLAN-892, PLAN-902, PLAN-904, PLAN-913]
current_step: 13
total_steps: 14
---

# [PLAN-921] CA-A共享A2X目标合同与HIR到C标量闭环

## 0. 变更摘要

将已批准CA-A正式立项：共享目标描述/provider/session/store去Rust耦合，以真实Checked HIR→C core/scalar消费者闭环；首例带文件化TestCase、独立actual/reference认证、真函数暖增量、来源绑定纯恢复和预算。drafting r1、0/14，本轮只规划。

依据：[C路线](../design/strategy/shared-a2x-and-hir-to-c-roadmap.md)、[Design51 §13](../design/51-shared-a2x-and-hir-to-rust.md#13-a2c第二目标与m1范围增补2026-11-02target-state)、[Design43 §20](../design/43-ac-av-typed-mir-and-abc.md#20-915917在途时的下一波2026-11-02target-state)。不是旧AST a2c扩面，不把标量通过当C/M1全完成。

## 1. 目标

G1：共享层不拥有Rust私有TargetProfile/lower_functions，C/Rust使用同一语义事实、受控store服务和目标隔离。G2：原892 core/scalar全部58冻结用例及负控有真实C文件集/编译/运行独立oracle；检查C未定义行为、源码序和命名卫生。G3：受支持body暖编辑未改函数P/IO0及实际对象/代际保持；新进程来源绑定恢复P0/R>0，失败后修复再恢复。G4：新C recipe/cert/domain严格，与旧C/旧Rust/新HIR Rust身份互拒，Rust既有输出/命中/证书不被无因改变。

非目标：CA-B record/text/方法资源、CA-C集合/变体、CA-D有限public ABI、CA-E四端/M1总签章；i64/u64/f32/f64宽源码、Unit/script入口、闭包、多模块/stdlib/泛型/meta、通用FFI、全平台/C++。797的公开ABI不从内部C helper签名推导；本包不改其descriptor。

## 2. 架构方案

受检HIR/单函数凭证→SemanticView/FunctionPlan→目标能力准入→目标C结构/控制流合法化→确定性C文件集→原受控工具执行→独立父判。共享语义键无target；C/Rust片段、文件集、runtime、工具/证书均有完整目标身份。同一ManagedStore及配额/lease/GC，不复制DB/前端/执行器/测试runner。新增拟成员experimental/ace/crates/a2x-c（包ace-a2x-c，实施前T-01确认无同名在途）。

共享TargetDescriptor/窄provider接缝移入a2x-core或中立模块；Family继续描述语义域，不以Family兼作target。Rust专有类型/输出留a2x-rust；工具驱动与证书各目标保类型清晰。具体trait形状由T-02 whole-path feasibility选择，不先做巨型逐节点抽象；禁止“生成Rust再翻成C”或调用旧Trans。

## 3. 技术栈

Rust实现ACE；C11，首认证平台Windows x86_64-pc-windows-msvc，真实MSVC cl /TC /std:c11独立编译。沿auto-tool-execution的CREATE_SUSPENDED/Job/超时/reader清理；不新造裸Command壳。C runtime先作为provider内版本化文本资产，不另开workspace/产品依赖，是否多文件由有界fileset冻结。

## 4. 需求分析与背景调查

授权：A2C已批准为M1必需目标，本轮用户授权提前规划下一波；只规划/取号，不授权构建探针/认证/产品切换或跨仓写入。当前main5a0ef7609，913/914/916已交付；915/917实施树在途。先前CA-*未占号，本包只正式化CA-A，不提前为B..E取号。

| 来源/实读符号 | 当前事实/内容SHA256 |
|---|---|
| docs/specs/auto-a2x/architecture.md | d8d3698589eeddd36e393017fe1365cb37fb2075bc34faac7784eeef3b1219dc；共享方向明确但现实现只有Rust真消费者 |
| docs/specs/auto-a2x/incremental-and-cache.md | 271c66eb046ec33b3abc8d70c1b38990c724d88948692bb46cff239e9db9bb6c；AXS/AXR分层、原预算/真实计数 |
| experimental/ace/crates/a2x-store/src/lib.rs | 220cade6557e2e08d2d8190cb502999561142e205d77f4665212cf59e3ffaa0f；直接import Rust lower_functions/TargetProfile；fragment_key/build_fileset/producers耦合Rust |
| 同A2xSession::restart_assemble/profile | 实际调用cold_assemble/profile；不是新来源绑定纯恢复已交付的证据 |
| experimental/ace/crates/a2x-core/src/{semantic_view,prepare}.rs | 现有SemanticView/FnFacts/FunctionPlan/语义fp；不可复制类型/ownership分析 |
| experimental/ace/crates/a2x-rust/src/{target,artifacts,cert}.rs | 现Rust schema/recipe/edition与证书，strict字段/文件集身份须保持 |
| experimental/ace/crates/a2x-worker/tests/cases/{scalar_semantics,hir_shape,incremental_cache,store_cache,contract_certification,resource_budget,boundaries}.rs | 原58语料/负控/预算/load256/boundaries可复用义务，改语言不等于自动继承结果 |
| docs/specs/auto-abi/{project,windows-x64-v01,conformance-cache}.md | 797仅Windows候选声明IR三腿，不是C源码后端/本包公开ABI；当前代码为后继CA-D基面 |

892/797历史delivery不是当前祖先，不拿旧SHA直接当本次集成证据；按当前code/Spec内容与冻结新SHA验证。902/904现交付和913实际来源/有界read可用，但AV专用root/schema/活cell不能照搬作通用A2X平台。T-01重核全部consumer与新路径不存在情况。

## 5. 详细设计

### 5.1 支持域、C整数语义与输出

只支持原core/scalar profile/实际类型及零参i32入口，按既有a2x域准入明确拒绝资源/宽位/闭包等。core与scalar算子差异按原Spec，不因C易表达就放宽；所有非法原源须在Auto前端/视图拒绝，cl失败只能infra。

i32 checked add/sub/mul/neg先在有足够位宽的中间表示计算并检查，再转换，不能执行C signed overflow后检查。div/rem先排零和MIN/-1；保-7/3=-2、-7%3=-1及原陷阱分类。未取短路/分支不得求值或提升陷阱；所有调用按Call.eval_args源序各求一次、临时固定后bindings落位，不赌C函数参数求值顺序。循环break/continue目标、return/trap传播、布尔规范化、名字/关键字/临时卫生、文本/路径转义均与既有语义一致。C helper是内部ABI，状态/错误通道显式，无unwind/longjmp跨FFI，不暴露新AutoABI承诺。

文件集至少包含完整用户函数、所需helper、头/入口及manifest所列全部依赖；不裁main/只编reference。运行结果用值/具名trap的确定性协议，编译器error、访问冲突、host exit、timeout不可充Auto trap。源位置信息是观测不是语义key；源/owner/entry/deps工具身份完整绑定。

### 5.2 共享接缝、隔离与兼容

共享SemanticView/prepare/原受检资源事实只读；目标descriptor含lang/standard/target/backend/version/runtime/printer等适用身份，认证另绑工具实际identity和flags。AXS语义层不因切目标重生产；目标片段/fileset/cert不得跨C/Rust或语义族命中。mixed-target同一服务会话与同store互测，不只是两个完全隔离程序各自绿。

Rust旧文件字节、recipe/schema及有效证书身份保持；中立接口调整不能静默改Rust输出或把旧Rust cert升级成C。新增codec/目标身份显式版本，不擅改旧schema字段来兼容；边界测试新增ace-a2x-c白名单须按真实依赖精确扩，不放行旧AST/AV/Cranelift/旧Trans。store可有兼容facade，但共享路径不再静态拥有Rust专有lower/target；是否保旧Rust构造入口由T-02记录，不改变已有调用语义。

### 5.3 来源绑定恢复和函数级增量

原restart_assemble重走cold不是纯恢复。本包以C标量最小闭环新增有界来源恢复：caller提供完整来源/声明上下文/目标/工具/实际依赖身份与可验证锚；store索引/manifest/语义/片段均未授信，decode后重建并验证必要HIR/函数凭证/依赖一致性。不得先parse/resolve/typecheck/prepare/lower全源来计算命中key，再将恢复写成P0；不得直接反序列化Checked/PASS/进程地址授信。完整src hash/上下文核对等公共管理工作记M；真实decode/结构/语义/资源再验证记R，确切生产点P逐项记录，不靠阶段改名藏重编译。

T-02必须从新进程真实入口跑来源→store→重验→装配→后续编辑可行性后再冻结实现。最小修复限A2X来源/有界codec/凭证消费，不借机建第二全局增量平台；若当前公共前端缺不可替代出口，具名列不满足的AC、触面与修订方案，依赖实施不能以冷fallback当完成，也不静默延期到CA-E。

至少3个未改函数、callee body/签名对照、A→B→A、坏编辑→修复→下一restart、readonly hit/miss、跨工程/target/权限：暖body只改目标及实际依赖者；未改parse/resolve/typecheck/HIR/prepare/lower/print/持久IO0，真正resident fragment/payload/代际保持。callee只改body不重编普通caller；读body/接口依赖者按实际边失效。source fragment凭证与完整源base/range/next sha绑定；cold/声明变化如实计量，不扩本包承诺全部语言声明增量。

发布是候选prepare→validate→commit原子，失败/取消/迟到旧成功root/bytes/代际可执行，临时无半提交；修复后下一纯恢复仍P0/R>0。current/history/lease/pin最后释放后物理GC；未知他方namespace/哨兵不动。测试强制whole/换对象/只留hash必须红。

### 5.4 首例登记、认证与预算

拟独立recipe ac-a2c/hir-scalar-emit-v1、cert ace.a2c-hir.scalar.cert/1、独立C fileset族；名字在T-01核冲突后冻结，不复用ace.m1.cert/1或Rust cert。新固定组/adapter/协议/consumer由首例同时接线，CA-E只收组合，不承接本包延期测试。旧binding/批准hash/required313不改；新增C ledger放独立目标登记草案/本包cases，后续CA-E明确AND旧门+新增C门，总签章此时仍不可签。

actual=本次现场producer完整原字节；reference独立手写且与actual分别编译运行、各自对同一独立oracle。证书严格绑完整路径/bytes/fileset/deps、源/profile/entry/target/runtime/printer、cl完整identity/flags、wrapper/reference/oracle/受控执行收据；旧证书/缺依赖/错工具/编译only/只跑reference/裁产物/输出篡改/未知字段/旧operation均非绿。fresh认证与日常快照分开：日常只emit/bytes/身份核查，cl/link/rustc/cargo实际spawn0；漂移只proposal/STALE，不自动覆盖golden或隐式编译。

A2X同会话合计WS≤48MiB（Rust+C resident+暂存一起计）、同owner store≤128MiB、cert scratch≤512MiB、单源/元数据4MiB、每流1MiB、值8MiB/managed32MiB、父private增量128MiB及原更严门不放宽；语义结构深度400按既有a2x.too-deep明确，不扩大成新语言限制。原load256/58语料/原负控及条件保留。compile≤120s/run≤30s/认证wall≤360s，受控Job/子孙/reader清理完整；缺工具=infra未执行不是SKIP/PASS。正式暖hit/restart/body-edit原规模至少30样本P90≤2000ms；fresh/冷准备/认证另列。

### 5.5 与915/917/919/920/922写面

P=a2x-c新provider/runtime/tests/手写reference先做；A2X S=core/store/rust/worker，由本包主写，公共core source/schema/单函数凭证仅缺正式出口时取短窗，不能重做915资源分析或917旧Rust修复。913已合入通用有界读取按实SHA消费，不写AV专有恢复root/lease/schema。

H=workspace manifests/lock、test_cases/{mod,ac_protocol,ac_a2r_hir}.rs、新C adapter、auto-test-case、SDK/fixed组/新登记文件，与915/917/919/920逐文件单写；921不能以“新目标”直接改他方shared枚举而不核全部construct/match/producer/consumer。每窗owner/revision/实际SHA/协议字段/所有reader/验证/释放，漂移STALE；无D915/D917/919整包前置。922只读本包基线/缺口，不抢A2X写面。正式认证/慢测错峰，不持H等待工具。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-a2x/project.md | 单Rust实现→真实第二目标及边界 | 实证共享而非接口命名 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-a2x/architecture.md | Rust耦合session→目标接缝/所有consumer | 不复制前端/DB | AC-01/08 |
| SD-03 | modify | docs/specs/auto-a2x/incremental-and-cache.md | 新C语义/片段隔离、来源恢复/P/R/M/原子与配额 | 不把cold冒restore | AC-03/04/05 |
| SD-04 | add | docs/specs/auto-a2c/project.md | 无新C→有限core/scalar现状 | 不宣称资源/全M1 | AC-02/08 |
| SD-05 | add | docs/specs/auto-a2c/scalar-hir-backend.md | C语义/域/拒绝/命名/错误/target合同 | 无C UB且源序一致 | AC-02 |
| SD-06 | add | docs/specs/auto-test-case/a2c-hir-scalar-certification.md | 新C actual/reference/strict证书/日常零编译 | 目标身份独立 | AC-06/07 |

## 6. 测试设计

现有完整职责组ace-a2x、ace-source、ace-semantics、ace-artifacts、ace-adapters、auto-test-case、infra-fixed、infra-ace-m1；必要原Rust consumer组case-a2r-hir-record-text-902、case-a2r-hir-collections-904。实际不改source/semantics时按冻结diff说明保留基础消费者或复用身份未变证据；不机械运行全部experimental。新增拟case-a2c-hir-scalar-921及C包接ace-a2x由T-09注册，未来正式性能命名门沿原ace-performance条件。新长期cases拟crates/auto-lang/test/cases/a2c-hir-scalar-921/，原892冻结语料不改。

| Case ID | AC | 输入/真实入口/使能任务 | oracle与区分性反控 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01 | Rust/C共享prepare/session/mixed store，T-03/08 | 同语义不重prepare、目标片段不同、无旧Trans调用；跨目标键/只改lang负控红 | core/scalar |
| V-02 | AC-02 | 原58例→C实际fileset→cl→运行，T-04/05/10 | actual/reference分别对独立值/trap；MIN/MAX、负div/rem不触C UB | 冻结core/scalar域 |
| V-03 | AC-02 | 命名逆序/短路/未达trap/循环及卫生，T-04 | 源序一次eval/惰性；故意C参数直接调用/先算RHS反控红 | 当前合法输入 |
| V-04 | AC-02/06 | 非法源/不支持域/缺工具/编译错误/运行crash，T-04/09/10 | 前端具名拒/infra严格分态；cl拒不能充Auto诊断/trap | 无新资源域 |
| V-05 | AC-03 | >=3未改函数、body/callee、历史回改，T-07 | 未改各P/IO0、真正fragment恒等/代际；whole/换对象反控红 | 暖body支持路径 |
| V-06 | AC-04 | 新进程纯恢复、坏源/上下文/依赖/好hash坏payload，T-06 | P0/R>0、错误不安装、合法非零ID准入；cold算key反控红 | C最小来源链 |
| V-07 | AC-04/05 | 只读、失败发布/取消/迟到→修复→再restart、pin/GC，T-06/07 | 旧root/bytes保留、无半写、再恢复P0、最后owner回收/哨兵不动 | 本session/store域 |
| V-08 | AC-06 | 双侧真认证、完整fileset/真实工具，T-10 | 只参考/裁main/编译only/未知字段/源工具deps错配均拒，能证明反控命中 | 新C cert独立域 |
| V-09 | AC-06/07 | 日常snapshot/漂移/STALE，T-09..11 | 工具进程spawn0；只proposal不改golden；强制编译负控红 | 非显式认证 |
| V-10 | AC-05/07 | load256/原cap边界/正式30暖样本，T-11 | 合计预算与P90≤2000ms，先限额后物化，原条件/规模不缩 | fresh/认证单列 |
| V-11 | AC-08 | Rust scalar/rt/collections原fileset/golden/store/cert，T-03/08/11 | bytes及有效身份不变/新target不污染；公用接口所有consumer通过 | 不追认旧历史非祖先证据 |
| V-12 | AC-07/08 | 首例AutoDown/新ledger/严格父判，T-09 | 新C身份不可被旧A2C/新旧Rust结果冒充；旧313守恒，新总M1不可签 | CA-E仍后继 |

实施/自查同一唯一树；final review为独立上下文，从冻结SHA重建判定，未授权不创建agent/chat。review时暂停写，修复回同树，最终merge按真实漂移重跑明确完整职责并集；证据不变可复用。周期cargo t all仅due主检出单实例，不为本包安排三轮全局门。全部V/AC均待实施。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01真实双目标/去Rust耦合/中立共享key与严格目标隔离，边界机械控制通过 |
| AC-02 | work | V-02..04原58语义/拒绝/源序/无UB/完整fileset/严格错误分类通过 |
| AC-03 | work | V-05真函数暖编辑未改P/IO0与payload/代际、实际依赖失效，非wrapper记数 |
| AC-04 | work | V-06/07新进程来源恢复P0/R>0、坏数据拒绝/修复后再读与原子事务通过 |
| AC-05 | work | V-07/10有界工作集/配额/GC/最后pin、只读/哨兵和原压力通过 |
| AC-06 | work | V-08/09真实actual/reference独立认证与strict身份、日常零目标编译通过 |
| AC-07 | work | V-09/10/12首例登记/原规模正式P90/全部原cap通过，不将总验收延期到CA-E |
| AC-08 | work | V-11/12原Rust合同/bytes/命中/证书兼容、所有公用consumer、M1旧新账分离 |
| AC-09 | review | 独立冻结review逐项及SD通过，未决必需来源/增量/认证不可只记后继 |
| AC-10 | merge | 正式主线交付/SD读回/archive、全部实际输出对账/guard/树分支组scope清理收据 |

## 8. 执行步骤

| T | owner_stage | 前置/写面 | 结果及验证入口 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前main/实际owner窗 | 核58基线/797边界/工具/新path冲突/所有consumer/窗口，冻结profile与新身份 | AC-01..08 | [x] 已完成（lang-921 树@9c81ee6d7（master基）。**冻结身份**：crate `experimental/ace/crates/a2x-c`/包`ace-a2x-c`、backend `auto-a2x-c`、descriptor lang=c standard=c11、recipe `ac-a2c/hir-scalar-emit-v1`、cert `ace.a2c-hir.scalar.cert/1`、fileset `auto.a2x.c.fileset/1`、print `a2x-c-print/1`、新层 AXC1 `a2921/axc-c-fn/v1` + AXP1 `a2921/axp-source-anchor/v1`（语义层 AXS1 共享复用，key 不动，registry consumers 增列 auto-a2x-c）。**冲突核查**：a2x-c/ace-a2x-c/ac-a2c/ 全repo零命中（"a2c"裸名仅主仓旧转译器历史，无路径冲突）；Rust 侧 ac-a2r/*、ace.a2r-hir.*、auto.a2x.rust.* 全部不动。**58基线**：scalar_semantics.rs 现值 61 条语料（37 core+11 sc+6 ace+7 bd，下限断言>=55）+hir_shape 4 测试=当前冻结全集，V-02 按当前 61 条全量执行；cargo t ace-a2x 基线绿（见 T-03 证据）。**797边界**：abi crate ctool.rs=IR三腿非C源码后端；本包复用其 cl 定位序（AUTOABI_MSVC_CL env→vswhere→最高版本；实机 BuildTools 2022 cl 14.44.35207 可解析，PATH 无 cl；缺工具=infra 非 SKIP）。**consumer 全图**：a2x-store（fragment_key/produce/build_fileset 耦合点 store:27,417-433,560-836,1163-1259,1399-1408）、a2x-worker 5 bins+17 test files、a2x-rust、boundaries.rs 5 处注册点（sanctioned/members/dirs/单目标钉住 bound:130-137）、ac_a2r_hir.rs（子进程消费）；根 workspace exclude experimental 无 crate 级依赖。**窗口**：920 已并发合并归档（06a6f8c72，vm/specs 面，无重叠）；919 在 merge 航班中（重叠仅 append-only：test_cases/mod.rs dispatch、contract.rs recipe 注册、test-groups.toml），921 基于其后 merge 时按漂移重验；922 只读本包。MSVC cl 无需 vcvars（ctool 定位序直接全路径调用），绕开 legacy vcvars 硬编码 Community 路径缺陷） |
| T-02 | work | T-01，私有P及S短窗 | 最小C标量+双目标+新进程来源恢复+下一body编辑whole-path feasibility；必需不可行具名修订，不cold凑绿 | AC-01/03/04 | [x] 已完成（以实施代探针、全链真实验证：最小C标量=T-04语料61/61真cl绿；双目标=T-03 seam+V-01 混合会话/store 绿；新进程来源恢复+下一body编辑=a2c-formal 子进程见证（--child-restore：跨进程 restore P0 断言+恢复后 ranged 编辑+真cl oracle 通过）+ a2c_restore 5 测试绿；无cold凑绿路径（无锚=命名拒绝）。D-01/D-02 由实施冻结：TargetProvider 窄trait 六方法；恢复最小凭证链=AXP1锚（源sha→逐函数origin/slice-sha/sem_fp/signature+文件集回执+runtime文本）） |
| T-03 | work | T-02，A2X S | 目标描述/provider/session中立接缝，旧Rust兼容/依赖边界V-01/11；cargo t ace-a2x | AC-01/08 | [x] 已完成（提交 b983607d（lang-921 树）：a2x-core 新增 target.rs 中立契约（TargetDescriptor/Family/FileSet/FilesetRequest/TargetProvider：fragment_key_inputs/fragment_layer_id/runtime_source/produce_fragments 批量生产/capture_prelude_seed/assemble_fileset）；a2x-rust 新增 RustTargetProvider（原 store 逻辑逐字迁移：AXR1/2/3 key 字段序 lang/edition/backend/backend_version/runtime_sha256/print_version 字节同一；RT/COLL prelude 走无损行式 seed codec（records/variants 往返），无新依赖）；a2x-store 删除 ace-a2x-rust 依赖（真去耦）、session 持 Box<dyn TargetProvider>、open 增 provider 参数（48 处 caller 更新）、resident 键 (backend,name)、set_target/current_backend 混合目标支持、AXC1/AXP1 层注册 + static_layer 扩、cold 入口 ensure_target_supports 命名拒绝、miss 批量生产保序（miss_slots 保 loop 位置=冻结字节序）；AXS1 consumers 增列 auto-a2x-c（registry 元数据，key/bytes 不动）。**验证**：cargo t ace-a2x 全组 95/95 PASS（字节同一 oracle：store_cache/incremental_cache/902/904 全绿）；rustfmt 本包文件 clean。V-01/11 完整证据在 T-08 补） |
| T-04 | work | T-02/03，新a2x-c | C域/checked helper/源序/短路/控制流/命名，V-02..04；新增C单测并入ace-a2x | AC-02 | [x] 已完成（提交 56ba586f：新 crate experimental/ace/crates/a2x-c（包 ace-a2x-c，deps 仅 a2x-core+core）：lower.rs 语句式 C11 生成（i32/bool 域，checked a2x_add/sub/mul/neg/div/rem——int64 中间量先检查后收窄，div/rem 先排零与 MIN/-1，C 截断保 -7/3=-2、-7%3=-1；Logical 短路=分支守卫临时量，RHS 语句仅在取侧执行；Call 逐参 eval_args 源序 pin 临时量后按 bindings 落位（不赌 C 参数求值序）；for(;;)+break/continue；fn_/mt_/lN_ ASCII 卫生命名，INT32_MIN 用常量表达式 -2147483647-1）；runtime.rs 版本化 C 运行时文本（单一 static inline 头，陷阱枚举 5 类与 Rust 同名分类，marks 参数化→内容 hash 进 fragment key）；boundaries.rs 白名单/members/dirs 精确扩 + 单目标钉改双目标真实断言（Rust AXR1/C AXC1 层隔离、C 非标量族命名拒绝、store 无目标 crate 依赖断言）。ace-a2x 组增 -p ace-a2x-c。**V-02 语料 61/61 全绿**（a2c_scalar_semantics::c_full_corpus_true_chain：复用 scalar_semantics 冻结表（含短路极性/陷阱反控/trace marks 序），独立 oracle 执行） |
| T-05 | work | T-04，C provider | 完整确定性fileset/runtime/入口/manifest与真实受管cl运行；V-02/03 | AC-02 | [x] 已完成（提交 56ba586f：provider.rs 确定性 3 文件集 main.c/a2x_c_runtime.h/generated.c + manifest（auto.a2x.c.fileset/1，含 per-file sha256/entry/functions origins）；generated.c 前向声明块=扫描 fragment 原型行（同 rt prelude 扫描文本模式）；crun.rs 受管 cl 腿：cl 定位复用 abi::ctool 冻结序（env→vswhere→最高版本，缺工具=infra 非 skip），vcvars64 从 cl 路径推导（BuildTools/Community 通用，无硬编码版本），cl -nologo -std:c11 -TC -W4 -WX -O2 -MT 经 cmd raw_arg 单命令行 + ace_core::link 受管执行（CREATE_SUSPENDED/Job/120s 编译/30s 运行/1MiB 流帽）；stdout/stderr _setmode 二进制模式对齐 Rust value:/trap:/marks 字节协议。**61 语料全部真实编译+运行+oracle 通过**（-W4 -WX 零警告，含 unreachable/常量条件陷阱逐例修复：防御性 return 删除、if 条件与 Or 守卫经临时量物化）；确定性测试 c_fileset_identity_and_determinism 绿（双新鲜会话字节同一） |
| T-06 | work | T-02/03/05，store/artifacts有界窗 | 来源anchor/codec/重验/纯恢复、坏件与修复再恢复V-06/07；cargo t ace-artifacts ace-a2x | AC-04/05 | [x] 已完成（提交 a47cc7f：AXP1 锚（a2921/axp-source-anchor/v1，行式严格 codec+往返单测；键 owner/profile/source_sha256/backend/entry；发布=提交点：RequiredData 边 put 时活性检查；幂等重发布不耗 quota）；restore_source_bound：P0 断言（lower/print/persist/parse 计数器零增量）+R>0（binding/payload/fileset 计数）+锚 runtime 文本 hash 重验+恢复 manifest 经 origins 图与冷装配字节同一；坏源/上下文/entry 命名拒绝且零安装；同 key 异内容=store 冲突（锚不可变，结构性防篡改）；repair 腿 re-cold 幂等+恢复仍纯。cargo t ace-a2x 含全部恢复测试绿） |
| T-07 | work | T-03..06，store/frontend必要窗 | 函数暖编辑/事务/lease/只读/GC V-05/07，原counter/payload恒等 | AC-03/05 | [x] 已完成（ranged 授权扩展到标量族（C 链暖编辑走 base+range+fragment，next_sha 锚重绑：origin 平移+edited slice/sem_fp 重绑+回执刷新；env 缺失经 ensure_env 重建（M 类 env_rebuilds 计数）；V-05 计数器证据：仅被编辑函数 lower/print/persist（+锚），A→B→A store 真历史命中零重降；readonly 冷零持久零锚；lease 绑锚 id，lease+GC 后恢复仍活（RequiredData 边保持片段集存活）；quota 门在幂等锚下不再误触（原 892 quota 测试原样绿）） |
| T-08 | work | T-03..07，双target consumer | 同会话C/Rust/同store隔离和原Rust bytes/命中/cert兼容V-01/11 | AC-01/08 | [x] 已完成（c_and_rust_same_session_store_isolation：同 session set_target 双向、per-backend resident、各自锚各自恢复（双后端均 P0）、文件集名隔离；Rust 兼容：迁移后 cargo t ace-a2x 全组绿=字节同一 oracle（892/902/904 store_cache/incremental/cert 全过，persist 计数仅因锚+1 且幂等重发布不再计）；Rust 旧 recipe/cert/fileset 常量零改动） |
| T-09 | work | T-05..08，H短窗 | worker recipe/协议/父adapter/AutoDown/固定组/新ledger草案首例，V-04/09/12；cargo t ace-adapters auto-test-case infra-fixed infra-ace-m1及新组 | AC-07/08 | [x] 已完成（提交 10c3b33：worker daily-emit-c op（spawned_rustc=0&spawned_cl=0、nonce 确定emit、文件 sha 清单）+identity 增列 C recipe/targets；契约四点注册（REGISTERED_PROBE_RECIPES/a2x_recipe 匹配/步汇 ac-a2c→run/a2x_hir profile 集 core|scalar）；父适配器 ac_a2c_hir.rs（单 worker 进程+双 spawn 零断言+manifest 重哈希+路径域审查+4 文件 golden 字节判+多余产物拒）；套件 a2c-hir-scalar-921（C01/C02/C03 + golden（worker 实发字节冻结）+ 手写 reference-C01.c）+ .gitignore manifest 豁免 + 固定组 + tools 前缀 (ac-a2c/,a2x)。**cargo t case-a2c-hir-scalar-921 PASS 3/3**；auto-test-case 111 PASS、ace-adapters 48 PASS。新 C ledger=以本合同 spec（a2c-hir-scalar-certification.md）+套件为首例登记，不动 required313（总 M1 仍不可签）） |
| T-10 | work | T-09，错峰认证 | C实际/独立reference真认证、golden审阅/strict坏证据控制V-02/08，不覆盖原expected | AC-02/06 | [x] 已完成（提交 10c3b33：certify-c op=actual+手写 reference 双腿真 cl（同受管执行器/冻结 flags）对同一闭式 oracle；CCertDraft strict（ace.a2c-hir.scalar.cert/1：cl banner/hash+vcvars 内容 hash+flags+全文件绑定+reference hash；字节稳定序列化）；worker crun 增 toolchain_identity+compile_c_standalone。a2c_cert 3 测试绿：双腿通过+字节稳定、错 oracle 双腿拒、11 项坏证据控制（旧 schema/recipe 冒充、裁剪、verdict/oracle_kind/backend/standard/工具身份/文件绑定全拒）；原 expected/golden 未覆盖（golden_sha256 为可选绑定）） |
| T-11 | work | T-01..10，错峰 | V-09..12/正式暖30样本/load256/完整职责自查与SD提案，记录所有残余 | AC-05..08 | [x] 已完成（提交 10c3b33+9c15e6f：a2c-formal 形式采样（load256 原样 273,575B/115 函数；31 版本全部真 cl 预认证对独立闭式 oracle；5 态×30 样本最近秩：hit p90=1499ms/restart=1593ms/body-edit=457ms/restore=150ms 全≤2000ms，fresh p90=8644ms 另列；子进程恢复见证 P0+恢复后编辑真 cl oracle 通过；收据 plan-artifacts/plan-921/a2c-formal-receipt.json，exit0）；V-10 预算：a2x-mem 混合目标相（Rust+C 同 session/store resident 并存 30 暖编辑）+ resource_budget 双测试绿（warm 48MiB+mixed 48MiB@load256 规模）；V-09 日常零编译=适配器 spawn0 断言；SD-01..06 六份 spec 已写（auto-a2x 三改+auto-a2c 两新+认证合同一新）；残余：904 golden-manifest 既有缺口（非本包，见 KNOWN-DEBT）、ACE 里程碑档全量/ace-performance 错峰档按原治理不变） |
| T-12 | review | work完成，冻结SHA | 独立全AC/mandatory cases与SD复审，未批延期不得pass | AC-09 | [ ] |
| T-13 | merge | review pass | 冻结职责/消费者并集与漂移重验、合入/SD-01..06读回/archive | AC-10 | [x] 已完成（rebase 无冲突+range-diff 全等价；冻结并集 rebased tip 全 PASS；ff-only 合入 master=3c760865c；SD 六文件 landed 读回；archive 见 §9 merge 收据） |
| T-14 | merge | T-13 | 全输出目录对账/摘要折计划，guard后删树分支组与整个scope、刷新登记 | AC-10 | [x] 已完成（输出对账：探针/emit/cert/receipt 均在 plan-artifacts/plan-921（唯一暂存区，无仓内散落/无第二证据区/无链接）；摘要+关键数字折叠入本文件 §9；wt-guard clean 后双 worktree+分支+组目录删除；plan-artifacts/plan-921 整目录删除；.retention/policy.json 登记） |

拟唯一树D:/autostack/.wt/lang-921/auto-lang、分支plan-921-dev，本轮未建。所有探针/生成C/对象/exe/日志/认证SDK副本/外置target只放D:/autostack/plan-artifacts/plan-921/，开工登记owner/用途/在途；通常构建target在专属树。禁止attachments/散落安装根/第二证据区/链接；merge全删，具名未解决hold才保留，不清他方产物。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-921 r1 | outcome:pass（规划静态准入，非C实现/认证/最终review） | 0..10节、T14/AC10/V12/SD6定义引用/本地链接锚点/固定组与拟新增组边界已核；next:work T-01→T-02 whole-path feasibility，必需来源/目标接口事实闭合后才依赖实施。CA-B..E不取号、不承担本包延期门；未建921树/scope/测试认证，旧批准/登记/expected/current Spec未改。

2026-11-02 | stage:new | PLAN-921 r1 | CA-A正式取号。当前事实是store依赖Rust且restart走cold，故source-bound恢复纳本包必需T-02/06，不能冒已交付或推给CA-E。915/917不设整包前置，S/H按真实owner。只规划、未实施/构建/认证/current Spec或旧登记变更；静态预检另记，所有功能V/AC待work。

2026-11-03 | stage: work | PLAN-921 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-921/auto-lang | branch: plan-921-dev | base_commit: 9c81ee6d7 | code_commit: 9c15e6fa3（b983607d→56ba586f→a47cc7f5→fff3c124→10c3b33c→9c15e6fa3） | dependency_revisions: auto-down 组内 detached 7c4d5cb（未消费）；并发会话完成 920 合并归档/919 merge 航班（重叠仅 append-only 文件，基面 9c81ee6d，merge 时按漂移重验） | task_ids: T-01..T-11 | evidence: cargo t ace-a2x 全组 PASS（105 测试：原 95 字节同一全绿+C 语料 61/61 真cl+C 恢复/暖编辑/混合目标/预算/证书全绿）；cargo t case-a2c-hir-scalar-921 PASS 3/3（首例 golden 字节判+spawn0）；auto-test-case 111/ace-adapters 48 PASS；a2c-formal exit0（5 态×30 样本全暖门 P90≤2000ms，restore p90=150ms，31 版本真 cl 预认证，子进程 P0 见证）；a2x-c/a2x-store/a2x-worker 零警告；rustfmt 本包文件 clean；SD-01..06 六份 spec 已入 worktree 提交 | blockers: 无合同内未决；KNOWN-DEBT 新登记 904 golden-manifest 既有缺口（观察项，非本包写面）；里程碑档（ACE 全量/ace-performance）按原治理错峰 pending，非本包每轮门 | next_responsible_role: review（T-12 独立冻结复审）

stage: goal | plan_id: PLAN-921 | plan_revision: r1 | round: 3 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-921/auto-lang | commit: 9c15e6fa3 | next: review

2026-11-03 | stage: review | review_scope: final | reviewer: 同会话独立复审（自冻结 9c15e6fa3 重跑重建判定，未采信执行摘要；实施会话内复审的既有限制如实声明） | PLAN-921 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-921/auto-lang | branch: plan-921-dev | reviewed_commit: 9c15e6fa3 | base_commit: 9c81ee6d7 | dependency_revisions: auto-down detached 7c4d5cb 未消费；master 并发前进（920 已归档、919 merge 航班中，重叠仅 append-only） | spec_inputs: SD-01..06 六份全文本 vs 代码核对（层 schema a2921/axc-c-fn/v1、a2921/axp-source-anchor/v1、fileset/recipe/cert 常量、flags 冻结表、路径/边界声明逐一 grep 对码；范围审计 71 文件无计划外改动、required313/binding/applicability 零 diff） | acceptance_results: AC-01..08 全 pass（独立重跑于 9c15e6fa3：cargo t ace-a2x 105 全绿[61 语料真 cl+恢复/暖编辑/混合目标/证书/预算/边界]、case-a2c-hir-scalar-921 3/3、auto-test-case/ace-adapters/infra-ace-m1 PASS、a2c-formal exit0[hit 1527/restart 1655/body-edit 951/restore 219ms P90 全≤2000，31 版本真 cl 预认证，子进程恢复见证 lower=0]、500 深度源经 C 路径实证 a2x.too-deep 命名拒绝、健康 0 警告 0 fmt 漂移） | pending_closeout_gates: merge 拥有的冻结受影响并集（ace-a2x+case-a2c-hir-scalar-921+auto-test-case+ace-adapters+infra-ace-m1，按真实漂移重验）、SD 读回、archive、树/分支/组/scope 清理 | findings: 无阻断。观察项：O-1 904 golden-manifest 既有缺口（已记 KNOWN-DEBT，非本包写面）；O-2 cert-scratch 512MiB 由构造满足（1MiB 流帽+KB 级产出，未放宽未单列断言）；O-3 C 冷 fresh p90≈11.4s 按合同另列非门 | evidence: 本 §9 记录即持久证据（命令+数字如上）；formal 收据 plan-artifacts/plan-921/a2c-formal-receipt-head.json（exit0） | next: merge

stage: goal | plan_id: PLAN-921 | plan_revision: r1 | round: 4 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-921/auto-lang | commit: 9c15e6fa3 | next: merge

2026-11-03 | stage: merge | PLAN-921 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-921/auto-lang | branch: plan-921-dev | base: 9c81ee6d7 | reviewed_commit: 9c15e6fa3 | delivery_commit: 3c760865c | checkpoints: prepared（reviewed 基线+SD 冻结+worktree 内 canonical 编辑）→ landed（master 前进[918/920 并发归档]后 rebase 6/6 无冲突，git range-diff 全 `=` 补丁等价，**ff-only 合入 master=3c760865c 线性历史**）→ canonical_verified（landed 提交含 SD-01..06：docs/specs/auto-a2x/{project,architecture,incremental-and-cache}.md 修改+auto-a2c/{project,scalar-hir-backend}.md+auto-test-case/a2c-hir-scalar-certification.md 新建；范围审计 71 文件零计划外改动、required313/binding 零 diff）→ archived（本文件 git mv 至 docs/plans/archive/，status: archived，簿记随收据提交默认分支）→ cleaned（见下） | merge_gate: 冻结受影响并集在 rebased tip 3c760865c 全 PASS——cargo t ace-a2x（105：61 语料真cl/恢复P0/暖编辑/混合目标/证书/预算/边界）+ case-a2c-hir-scalar-921（3/3 golden 字节判）+ auto-test-case + ace-adapters + infra-ace-m1；formal 采样在 reviewed HEAD exit0（hit 1527/restart 1655/body-edit 951/restore 219ms P90≤2000，31 版本真 cl 预认证，子进程恢复见证 lower=0；收据已折叠上文） | cleanup: wt-guard 双树 clean（auto-lang+auto-down 兄弟）→ worktree remove×2 + 分支删除 + 组目录删除；plan-artifacts/plan-921 全删（2026-10-31 裁定；无具名持有）；.retention/policy.json 登记刷新 | observations: 生产面产物检查——本包为 experimental/ace 语言仓内交付，无后端二进制/web bundle/守护进程部署面；主检出 experimental/ace/target/release/a2x-worker.exe 为陈旧基（merged 后需重构建，非部署阻塞）；batch regression 到期核查：921%5=1 非号到期 | evidence: 本 §9 即持久证据（命令+退出码+关键数字全部内联） | next: done

stage: goal | plan_id: PLAN-921 | plan_revision: r1 | round: 5 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 3c760865c | next: done

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 provider形状 | T-02以真实C/Rust/consumer冻结窄接口；不先造无消费者大框架 |
| D-02 恢复最小凭证链 | T-02证来源→有界store重验→纯恢复→下一body编辑；不可行给AC/真实缺出口修订，不cold冒P0 |
| D-03 工具identity | T-01/10固定MSVC C11与完整target/flags/依赖身份，缺工具如实infra |
| D-04 资源/后继门 | CA-B/C/D/E未分号，921不给其PASS/NA；T-11留接口/owner/可执行交接 |
| D-05 写面窗口 | T-01逐文件核915/917/919/920与manifest/lock/父consumer，未释放只挡对应任务 |
