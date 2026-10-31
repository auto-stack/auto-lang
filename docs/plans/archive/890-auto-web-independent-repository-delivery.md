---
plan_id: PLAN-890
status: archived
completion_kind: delivered
feature_name: AutoWeb独立仓完整搬迁与四仓交付
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-31

plan_revision: 7
supersedes_spec_components: [docs/specs/auto-media-core/project.md, docs/specs/auto-api-gen/project.md, docs/specs/language-sdk/project.md, docs/specs/language-sdk/architecture.md, docs/specs/stdlib/design/http-server.md, docs/specs/stdlib/design/http-service-deployment.md, docs/specs/stdlib/design/backend-assembly.md, docs/specs/repository-boundaries/project.md, docs/specs/auto-man/project.md, docs/specs/auto-ui-tools/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-test-case/governance-and-inventory.md]
new_spec_components: [docs/specs/auto-web/project.md, docs/specs/repository-boundaries/web-four-repository-delivery.md, docs/specs/language-sdk/http-host-port.md]
touched_goals: []

affects: [repository-boundaries, language-sdk, auto-api-gen, auto-web, auto-man, auto-ui-tools, auto-ui-compiler, auto-test-case]
depends_on: [PLAN-884, PLAN-885, PLAN-887]
current_step: 11
total_steps: 12
---

# [PLAN-890] AutoWeb独立仓完整搬迁与四仓交付

> **当前授权（最终）**：用户明确“不需要再次测试了”，要求更新主分支后安全合入。896新cargo t与四仓冻结门已有实际通过证据；898及后续主干整合只做独立静态核查，不追加构建、测试或周期批次，不把旧结果认证为新基线已执行。禁止远端发布保持。
>
> **当前进度**：四仓已ff-only合入、canonical已核；r7独立终审与恢复复审通过。以下各早期停止点/pending均属历史。归档与五工作树清理已完成；附件本地副本和空组目录删除被自动策略拒绝，T-12保留该清理尾项。

> r6 / executing / 6/12；2026-10-30。用户已授权890完整交付，并明确选择“显式安装 Web adapter”。885收尾已由用户确认；当前按下述显式SDK装配迁移合同执行。r1的仅规划限制与等待记录为历史，不覆盖当前授权。
> **当前停止点（2026-10-30）**：用户“继续后续阶段”启动r4单元2；T-05～T-06运行时/生成器迁移及定向验收已完成并提交，暂停在该单元末。语言`1b177ebab`，Web`d8c889c78`；见[本阶段交接](#e890-ca5b663d4f4f80ee)。**组合产品尚未构建通过**：根check暴露UI back-proxy三项旧Core引用错误，Vue另有旧HTTP生成调用，均由下一单元T-07消费迁移关闭；不冒baseline红或整包可用。**完整tm闭包仍NOT_PASSED**，保留旧3个AAVM红及未执行项。整体executing6/12，T-07待启动，未合入/归档/清理。
>
> **上一阶段停止点（2026-10-30；本轮已恢复）**：T-03～T-04基础实现与定向验收完成，阶段提交后暂停；不进入T-05。语言代码冻结`1a253950810de67f16f487c7eb7a2745f1061bb0`、交接`e45399a633a1dcf74f70943cf2d2c663791589a6`，Web冻结`26259642f56656547db7bc07a68c35a2bd225edd`。**完整tm消费者闭包未通过**：750模块选择的执行中55通过、3个既存红、23原条件不适用，旧HTTP测试装配挂起后中止，剩余未执行；修复后直接23项和HTTP4项重放通过，不能替代新冻结组合的完整门。见[本阶段交接](#e890-18ff84e0bd8a968f)。整体仍executing；独立终审、最终门及完整AC均保留。
> 依据：[Design50](../../design/50-autoweb-service-framework-and-repo-boundaries.md)、[当前拆分路线](../reports/v06-extraction-execution-roadmap.md)。一个完整工作包，内部可行性探针、迁移和恢复不另取号；本Plan为跨仓唯一合同，未来Web仓留指针，不再重复分配迁移Plan。

> **2026-10-30 历史T-02停止点（r5已获本单元启动授权）**：T-02 r3 有界修订与修复已通过阶段验收，executing / 2 of 12；本次到此暂停。代码/契约冻结 `f56211c12`，阶段交接冻结 `4ec696a63`。保全部树/分支，不进入 T-03、不合入、不归档、不清理。阶段审查由实施会话重建证据，明确不冒独立终审；AC-13 与 merge-owned 门仍 pending。后续按 [r4串行及正式同步安排](#e890-69842b508b36ad9b)；每启动单元结束提交/更新计划/交接后暂停。现行 [T-02 r3 契约](#e890-1a20e94b922d0662) / [交接与证据](#e890-8caf4976a4ad8156)；r2 needs_replan 收据保留为历史。

## 0. 变更摘要

将既有Web服务领域从语言/Man/UI的混合实现中迁至与auto-ui、auto-man同级的auto-web独立仓：服务契约、路由/绑定/错误、部署政策、VM/Rust服务适配、HTTP服务及客户端生成、资源与测试、独立发行、真实消费者切换和原件退出一起交付。语言继续拥有语义/转译/VM，Man继续拥有CLI/工程/依赖与通用进程编排，UI继续拥有界面与非HTTP适配。

“完整”指本期既有服务框架职责完整搬迁和独立交付，不是实现Design31全部网络生态，也不是把整个auto-api-gen或全部HTTP标准库搬走。基线支持面、历史红/skip/未执行项分列；不因迁移宣称全应用、全平台或测试全集全绿，不承诺提速或全生态净删数量。

## 1. 目标

- G1：Web领域有唯一生产owner，真实调用切入新仓；语言基础不反依赖Web实现，基础Web构建/必需测试不要求UI/Man/Tauri。
- G2：VM与生成Rust服务保持现行支持行为、部署/预算/安全/健康/关闭合同，885生成修复与非HTTP兄弟消费者保持。SDK服务宿主按用户批准的新版本合同显式安装Web adapter；普通语言调用不增加装配要求，CLI/UI现有服务入口内部完成装配。
- G3：测试资产逐ID和原子断言迁移，新增Web独立入口及四仓组合执行；887守恒/强oracle不退化。
- G4：固定版本可独立构建、测试、打包并在外部工程fresh/locked/重定位使用，Blog及受影响双UI消费闭环。
- G5：旧重复实现退出，来源/历史/许可证与恢复可验证；独立终审、冻结门、各实际修改仓合入、Spec/账本和guard收尾齐备才U890。

非目标：ACE能力/默认引擎改动、AutoVM/A2R广泛重写、细拆编译器crate、UI状态/Feature/HMR重设计、数据/runtime另建仓。现有HTTP客户端及通用运行能力按真实归属保留语言，不笼统外迁。新增OpenAPI/SDK产品、认证系统、可观测平台、内置TLS、公网/跨OS生产认证、浏览器/沙箱/远程运行均另议；已有安全/观测能力必须保留。无远端push、建远端仓、改默认分支或公开registry发布授权。

## 2. 架构方案

### 2.1 目标职责与依赖

| 所有者 | 目标内容 | 依赖/兼容约束 |
|---|---|---|
| auto-web合同层（候选`auto-web-contract`） | 路由/参数/响应及严格服务配置的纯数据/政策，必要轻合同 | 不依赖Core/VM/Axum/Man/UI；Core如必须引用，仅允许固定单源的轻合同，不能拉Web实现。具体包数T-02冻结，不以多crate计功 |
| auto-web框架/工具（候选`auto-web`） | HTTP装配、政策执行、服务与HTTP客户端生成、资源、语言adapter、独立CLI/库入口 | 可依赖公开语言SDK及轻合同；normal/build/必需dev闭包无UI/Man/Tauri，无Core→Web实现反向边 |
| auto-lang SDK与语言adapter | 元数据/类型/通用转译/VM执行、值转换、线程亲和与最小宿主端口；既有非Web生成适配 | 不把Store/AST/VM复制到Web，不扩大私有pub面代替端口。HTTP标准库shim保名字/ID/语言语义，领域实现通过已验证端口装配 |
| auto-man | `auto service`、`auto run`等命令、项目/配置来源、安装构建、通用子进程管理 | 调用Web公开能力；管理核心不引完整VM/renderer。`auto serve`仍为daemon，不能被服务命令占用 |
| auto-ui | Vue/native VM界面、Rust UI/Tauri和服务消费 | HTTP客户端生成用Web正式出口；Tauri IPC、sidecar声明及共享Rust工程规则保原单源，不随HTTP机械整搬 |

### 2.2 两个先决技术门

**端口与兼容门（r2用户已裁定）**：Core当前直接启动HTTP，既有host provider没有完整HTTP端口；`server_model::ApiRequest.raw_upload_body`含Axum Body，不能改名后假称中立DTO。用户明确选择“显式安装 Web adapter”：新版本Core只持中立端口/语言桥，Web拥有实现并向宿主提供显式安装入口，不引Core→Web实现边，也不保留第二套旧服务实现。直接SDK服务消费者须添加固定Web依赖并在执行前安装adapter；这是批准的装配兼容迁移，不宣称旧零装配行为未变。普通无服务路由的语言调用保持原行为。Man/UI现有服务命令在宿主组合点安装，外部命令/flags/stdio/exit/ready/关闭语义保持。缺provider且实际需要Web服务时返回明确装配诊断，不伪成功、不隐式旧回退、不静默加载动态库或外部工具。T-02仍须列全部实际消费者，并用真实VM→handler→JSON/SSE/file/upload→body终态验证owned消息/流、!Send VM原线程、背压/预算/取消及安装/缺席/重复安装/显式覆盖/实例隔离；安装API、版本与快照时序在T-02冻结，用户方向裁定不等于全路径技术门通过。

**共享生成闭包门**：auto-api-gen兼容门面若委托Web，Web不得再反向依赖整个auto-api-gen取得rust_workspace/sidecar/source_owner。优先复用已有适当owner；确无合适承载，T-02可提出一个具名、函数清单受限的语言侧共享生成载体（候选`auto-service-gen-core`），仅解决现有共享职责与DAG，不建common巨包。不复制SidecarDecl、类型提升/源码新鲜度/共享工程和锁规则；每个函数/状态只有一份实现。

两门允许在Plan内部有界调查，T-02未通过不得大规模搬迁。探针不是正式交付，也不新增ACE协议/引擎或改变语义。

## 3. 技术栈

复用Rust/Cargo、当前Tokio/Axum两服务轨、Python现有发行/回归设施、TestCase/nextest/libtest和公开工具协议。不同轨现有Axum版本对象不跨边界混用，不以搬迁顺手升级框架。优先扩展`scripts/ui-repository-{fixture,distribution,switch}.py`、`three-repository-fixture.py`及`run-full-regression.py`的既有机制；新增Web专用入口只能是有正反消费验收的适配，不能另造弱事务/测试裁判。

## 4. 需求分析与背景调查

### 4.1 授权、状态与版本

2026-10-30 用户交接恢复同一 Plan 的 T-02 有界修订/修复，未扩仓/目标/AC/预算。本次 r3 T-02 已通过阶段验收并暂停；先前全链授权不自动恢复 T-03 或后续合入/归档/清理。整体目标、12项任务与验收标准保持，阶段收口不等于计划交付。r4记录串行安排；r5用户已明确启动正式888基线同步和T-03～T-04，现已执行并在该单元末暂停。后续单元仍须用户另行启动。

历史规划授权（2026-10-29）仅限本Plan、设计/路线簿记。当前用户已通过`/goal 完成计划890`授权work→独立review→merge→清理全链，并确认“885已经收尾了。”；2026-10-30进一步明确“显式安装 Web adapter”，批准直接SDK服务消费者的新版本显式装配迁移。完整Web迁移和四仓交付范围保留，不新增远端发布或UI/Man结构后继实施授权。890与888无交集工作可并行，共享脚本/注册/锁/最终合入窗口串行；不发跨聊天消息，不创建实施agent。独立final review仍按合同使用同树独立上下文。

本轮只读基线：lang `674871e2a3bcc1a9cf594ac7446f3c1d1129ecbb`，UI `bce23b0f43a8dfa1d46360a47d3e98121061b56d`，Man `e6cc06fbc496f366edde7929abfd49254ec13647`。它们是调查快照，不是未来冻结组合。

| 输入 | 核实结果与使用限制 |
|---|---|
| [884](884-language-structure-closure-without-crate-split.md) | archived10/10；delivery `fa0282c85`，有guard清理收据；后继保单运行桥/注册/主管线职责，不重构旧引擎全体 |
| [885](885-blog-generated-rust-backend-parity.md) | archived7/7；delivery `dacd00184`，Spec/ledger已落；历史尾部cleaned仍记待guard且原始日志未恢复。用户已确认885收尾，启动缺口由该确认关闭，不伪造日志或代改885收据 |
| [887](887-test-governance-and-ace-first-corpus.md) | archived r2、10/10；delivery `dd916c184`，有清树收据。盘点/守恒/首批ACE消费已交；性能以用户批准机制级负结果关闭，未实现显著加速。B17–19语义、D生命周期、ui-tn手工/Windows等限制保留 |
| [888](888-module-test-selection-and-ace-batch-pilot.md)、[889](889-native-collections-variants-source-closure.md) | 收尾读回888 executing r1、0/10，尚无实施树；原全链授权被后续“先核查，暂不交接共享写入”限制，worker case_serve/parent run_check_impl等X888及M888仍待实际接受。889 drafting r1、0/10，为ACE集合/变体包。两者均不构成Web全局U门，T-01按真实合同/正式交付核交集，不代写或接管它们 |
| 本地目标身份 | 本轮`D:/autostack/auto-web`不存在，仅此本地事实；未查询远端，不代表无既有仓。T-01核已有身份/refs/历史/WIP/许可证，存在则沿用保全，不清空/orphan/强推或创建替代仓 |

取号使用独占文件锁并重新扫描活跃/归档文件：最大实际文件887，号源却仍887，888/889已有用户/受管设计引用，故保留两号不占用，纠正号源后由`scripts/new-plan.sh`正式分配890，下一号891；没有重建888/889或改其合同。后继只认中央号源实际状态。

### 4.2 Spec与源码证据

现行来源：`docs/specs/overview.md`、`auto-api-gen/project.md`、`language-sdk/{project,architecture}.md`、`stdlib/design/{http-server,http-service-deployment,http-return-resources,http-file-transfer,backend-assembly}.md`、`repository-boundaries/{project,ui-repository-delivery,man-three-repository-delivery,split-equivalence-acceptance,generated-ui-distribution}.md`、`auto-test-case/governance-and-inventory.md`、`auto-man/project.md`及`auto-ui-tools/project.md`。overview/HTTP目标设计的历史路径、数量或愿景不能覆盖已交付章节；新增归属只在实际交付后改canonical。

观察实物：语言`crates/auto-api-gen/{Cargo.toml,src/{lib,api_gen,rust_workspace,sidecar,source_owner}.rs}`；Core `src/{entry.rs,entry/run_bridge.rs,lib.rs,http_service_config.rs,http_service_observability.rs,vm/host_capabilities.rs,vm/ffi/{http_server.rs,http_transport.rs,http_server/server_{model,route,argbind,dispatch,scope}.rs}}`；Man `crates/auto/src/main.rs`与`auto-man-runtime/src/http_service.rs`；UI `crates/auto-ui-tools/src/{run_vm,run_vue,vue,rust_ui,tauri}.rs`。源清单只读观察，不等于逐函数搬迁审计。auto-api-gen dev依赖Tauri，Web基础必需测试须按HTTP/非HTTP真实拆闭包。

| SHA256输入 | 内容hash |
|---|---|
| API生成Spec | `a96ff4aea2476deabd2ea356851bc18f71d0473214308b46254b28326c743541` |
| HTTP部署Spec | `cea4cd929f9b2001cb4a4318f249a743b2074feb666a7ee55899d29381c0f5f0` |
| 887治理Spec | `ed9f46fad6b30b17585d469e15b9250252695111e14d981a9225470305019949` |
| api_gen.rs | `8fe8257e64684901c3de17d6d103f1766dbac7d69bff4eef46875ea81ebd70d8` |
| http_transport.rs | `781fa869e7c62b849556a89230ba46dee0a116e2c45b3c3f8ad1d4f1f69b467a` |

## 5. 详细设计

### 当前r7测试框架合同（用户要求整合896）

用户已明确启动本单元：先核实888正式交付，合并master到原分支，定向复核后执行T-03～04；完成提交/更新计划/交接后暂停，不进入T-05。已核实888交付`5a6c23afa`、归档`107148fcd`及收尾`129ef6057`都为master祖先，归档状态10/10且最终门/guard收尾明确通过；未联系或操作888会话。同步前890=`ed44fd0bd`、master=`129ef605763e59ca13bf312fd4a391eded10c13a`，原树clean；已合并为`29ed9d7bde334ae14e21223c935a4f3974581f71`，无人工冲突。双方实现与测试义务保留。

896已正式交付且用户明确要求整合后用新cargo t重编译测试。当前正式规则取docs/specs/auto-test-case/module-selection.md与scripts/test-groups.toml：固定Cargo包/target/Rust namespace组，同配置OR并集一次nextest，不预枚举全仓、不执行旧叶目录/自动消费者图。旧tm目录/收据仅作历史审计。日常重测先cargo t；按890真实改动明确补vm/http/stdlib/ffi/a2r/必要生成消费组及Web、UI、Man各自原生owner入口，具名说明选择理由，不用cargo t all或旧全目录替代影响分析。Web/shared-generation新包及四仓真实收据须在T-08适配固定入口；原零选、重复、漏执行、错来源、坏组合拒绝及原比较器保持。896批准的AAVM/AA2R默认ignore与ACE迁移/默认隔离原样继承，不能把未执行实验认证为通过；其他原oracle、样本、压力、平台/里程碑条件不减。独立复审后T-12执行冻结固定组/必要集成并集及真实外仓腿，周期批次仍按到期主检出单实例，不每计划叠跑。

本单元边界：T-03仅本地普通Web仓治理/bootstrap与唯一Web实施树、可编译基础包/锁；T-04迁出严格纯配置单源到auto-web-contract、Core保同类型re-export与中立host/launch/消息桥，迁出四个受限共享生成模块到语言侧auto-generation-core并保旧消费路径薄委托，核SDK源码闭包。Web runtime/生成器算法和旧临时runtime桥退出仍T-05/06；不得用stub安装器或隐式回退冒已迁运行时。建仓与基础接线不改变业务语义。

### r4 阶段串行与正式基线同步（当前执行顺序）

用户本次明确：本次仅T-02，现已收口并暂停；完整AC、12任务及技术义务不减。
后续按以下启动单元分别取得用户指令、串行执行：

| 启动单元 | 允许任务及顺序 | 完成后的动作 |
|---|---|---|
| 1 | T-03～T-04：先正式基线同步，再建仓、基础包与Core接线 | 提交、更新主计划、简明交接，然后暂停 |
| 2 | T-05～T-06：先运行时迁移，再生成器迁移 | 同上，等待下一单元启动 |
| 3 | T-07：Man/UI消费迁移 | 同上 |
| 4 | T-08：测试资产、逐ID守恒、四仓治理 | 同上 |
| 5 | T-09～T-10：独立发行、恢复、应用消费验证 | 同上 |
| 6 | T-11：独立终审 | 记录明确结论及冻结证据，然后暂停 |
| 7 | T-12：冻结门、合入、规范沉淀、安全清理 | 全合同收尾通过才完整交付 |

**T-03前置顺序**：用户明确确认继续 → 核实888已正式合入master并记录正式交付/复审/验证/收尾锚 → 核实890成果已提交且原`plan-890-dev`/worktree干净 → 记录同步前890 HEAD、master HEAD、888正式提交 → 将包含888正式交付的master合并进现有`plan-890-dev` → 保双方实现/测试义务，处理冲突并提交、记录同步后HEAD/冲突处理依据 → 定向复核受影响接口/工具/T-02证据；未受影响项说明实现/依赖/配置/输入不变后复用 → 按888正式工具核对后续治理，若与890合同实质不一致先有界修订，不能静默替换验收。满足以上条件后才实施T-03。该同步不是将890合入master，本次不执行。

888由原会话优先独立复审、必要验证、合入和收尾，不等待890；本次不联系或操作该会话。890避让共享脚本/注册/锁文件/最终合入窗口，不使用888未交付WIP。每单元失败均保留未完成状态和具体关闭条件；实质设计/合同变动记录原因并先修订，不无限追加修补。阶段提交不代表整个计划完成。完整规则与当前冻结身份见 [r4阶段交接](#e890-69842b508b36ad9b)。

### T-02 r3 技术契约（r4保留；阶段安排以下文为准）

2026-10-30 用户交接恢复 T-02 修订与修复。仅继承原目标/AC/仓集合及 T-02 窗口；不进入 T-03、合入、归档或清树。基线语言 `76f2069d5`（代码 `e2ed6ba76`）、UI `7b323bc`、Man `e6cc06f`、Down `c3464d0`。语言树 clean，主检出产品路径 clean；主检出已有其他文档 WIP 保留。

| finding / 决策 | 不变量与实施入口（owner_stage=work） | 精确关闭条件 / 反例 |
|---|---|---|
| R1 / D-T02-01 | 中立 `HttpServiceLaunch` 在 invocation 执行前显式绑定，独立于编译 metadata；配置由领域 owner 以不可变 typed payload 传递，Core 不解析 Web 部署政策。sync SDK 桥跨线程携同一快照，async 调用逐 poll 恢复上下文，VM 构造后持自身快照；host 仍在激活时选择。旧 active-config 仅作现有入口兼容转换，不成为新 SDK 权威 | configured SDK 实际业务端口/config hash/port0 bound 正确；嵌套、交错调用与失败后新实例不串配置；普通程序无额外装配要求。旧 audit 的 env 端口精确业务反例必须转绿 |
| R2 / D-T02-02 | `serve_with`/`serve_async*` 返回真实 `AutoResult<()>`，bind/runtime/join 错误逐层传播；ready、身份和停止属于 `ServiceRuntime`，进程 fatal/bound 仅历史 facade，不能裁决新调用 | 占口 SDK 非零且不 ready；同进程 A 失败后 B 成功/独立关闭。保持原 Result trait；不由 stderr 字符串替代错误传播 |
| R3 | owner 成功交出普通 parked reply 后，由网络桥写状态/字节并终结；发送失败由 owner 取消，不抢先 complete；错误/超时/取消也恰一次回收 | audit 原 wire200/25 对应唯一 event200/2/completed；补同源发送失败、async 错误/取消与资源归零，不能只检查响应 |
| R4 / D-T02-02 | runtime session 持网络与日志线程句柄；停止发射→关 sender→排空→join；join 在阻塞执行器/回收线程，不能阻塞业务 reactor。启动失败与 future Drop 也必须触发取消/回收；VM/parked 清理留原线程，网络只持 owned Send 消息 | 正常/启动失败/取消都有真实线程终结证据；日志写/线程错误显式报告，队满仍丢弃计数且不阻塞；取消只触发清理，完成 receipt 必须等待实际 join，不能用 future Drop 本身冒清理成功 |
| R5/R6 / D-T02-03 | 关闭 R1..R4 后冻结候选契约的包/API/消费者映射及正式887兼容四仓字段/拒绝规则；实际建包、runner/发行仍各原任务负责 | 指明已执行与待实施命令、required owner/leg/ID/source/hash；不将 source probe 当完整迁仓；不修改888共享窗口 |

r2“最终 API 尚未冻结”及 needs_replan 收据保留为历史。r3 已关闭 R1..R4 并冻结 R5/R6 的执行目的地与 schema，现行证据见 [r3交接](#e890-8caf4976a4ad8156)。T-02 计2/12；所有实际迁仓/原ID运行/四仓发行、独立终审及 merge-owned 门仍 pending。

### 5.1 启动门、工作树与交接

G890 = 已有完整执行授权 ∧ U884 ∧ U885相关完整交付/收尾（用户已确认） ∧ U887。U包括完整验收、独立终审、冻结门、实际合入、规范/账本、归档和guard收尾。当前G已满足，T-01核实际身份和X，后续只写已释放范围。U885收尾文本差额不由本Plan代填。T-01/T-02无U865/U888/U889全局门；r4新增T-03阶段启动门：用户确认继续且888正式合入，并先按上述顺序同步现有分支。888不反向等待890，也不要求等待ACE整条路线。

未来一Plan一仓一树：`D:/autostack/.wt/lang-890/{auto-lang,auto-web,auto-ui,auto-man}`；语言分支`plan-890-dev`，其他仓默认`codex/plan-890-dev`，若各仓明确规则不同则从其规则。T-01重读各仓AGENTS及实际路径/HEAD；auto-down等仅固定必要只读依赖。不得创建链接；不动现有主检出WIP，不借未交付变更。独立review在同一组树冻结只读，不另复制一套实施树。

T-01可先在语言权威树核边界，T-02探针在该树内拟新增`test/fixtures/modularity/web-port-probe/`进行有限源码/包图试验，暂不创建目标仓；不把探针复制体作为长期实现。通过后T-03承接已有目标仓或新建本地普通Git仓；新仓先最小治理/bootstrap提交，再建其唯一实施树，所有产品修改在实施树内完成。Web默认分支不是清理对象；最终只清Plan树/分支，不删正式仓。确有远端身份/操作需要时另列具体对象，未授权不得执行。

| 写面/合同 | X890接受与回交 |
|---|---|
| 885生成/转译/facade/共享工程 | 只接正式已合入的885修复和原控制；迁出HTTP算法后给旧/新路径映射、新工具链pin及消费者回执。转译/语言根因仍回语言owner，不搬成Web私有修补 |
| 887清单/runner/退役/floor | 复用已交付守恒规则；Web自己登记新owner与四仓编排。迁移不是新减测理由，保留既存红与未执行项，不重生成旧baseline接受缺项 |
| 888共享scripts/test-governance、注册与公共test接口 | 888优先由原会话独立收尾，本次不联系或操作。890避让共享脚本/注册/锁/最终合入窗口；T-03前核实888正式合入并同步master，之后以其正式工具核对890合同，差异先有界修订，不借WIP或默减原887测试义务 |
| ACE865/889等Core/host/注册/指纹 | 实际diff到符号的X交接；worker/session/profile/cache/资源算法只读，不为Web改ACE。记录前owner、释放范围、正式SHA、接受SHA、接口/锁/资源、受影响case和回交 |
| UI/Man结构后继（未编号） | 实施排U890后；交公开Web服务/生成出口、CLI/客户端调用图、固定版本、测试owner与未决债；不将Web算法再搬一次，不借本Plan启动UI架构重设计 |
| BLOG-004和应用 | 原合同可继续，不增加Web硬前置；Web真实消费用固定应用源及独立数据/产物目录，不改业务WIP，不代勾应用验收 |

### 5.2 迁移单与行为范围

T-01/02形成一份逐符号/资源/状态清单：源路径/commit→职责→唯一目标→调用消费者→测试ID/原子义务→来源/许可证→删除/兼容/恢复条件。保留项必须注明为何仍属语言、Man、UI或中立规则，不能按文件名全搬；未知required项阻塞后续。首期Web独立仓范围含：

1. HTTP路由/绑定/参数校验/返回和错误、Web配置/纯政策及服务装配、已有HTTP客户端生成和服务模板。VM桥只保语言数据转换/执行/线程与句柄端口；框架不能任意读Core私有全局状态。
2. VM/Rust两轨现行部署支持矩阵：development/proxy_service、地址与端口优先级、严格配置、Host/CORS/可信单层代理/限流、health/ready/受控关闭与有界观测。保既有支持等级和已知差异，不统一重写Axum transport。
3. JSON/普通响应、SSE、文件、上传、handler scope与资源许可的现有支持路径；ready不能只看TCP连通。请求/连接/body终态、失败/取消/期限/drain/断连的owner清楚且恰一次释放。
4. 885的source_dir导入、runtime time facade、字符串/借用/参数及同形类型身份；生成新鲜度/producer/resource/锁一致，真实生成Rust必须编译运行。
5. CLI与UI消费接线和必要资源/测试/文档发行。Tauri IPC与通用sidecar/shared workspace/语言转译仍各有原单源；允许有限兼容门面委托，但无第二份生产Web算法、隐式旧路径回退或永远待迁的活跃消费者。

### 5.3 测试迁移、四仓入口与发行

逐ID保原来源、父入口/聚合子case、oracle、条件、档位与适用后端；移动/改名/退出/新增分账，重复wrapper数量不能冒充覆盖。新Web独立档位T-02冻结，必含真实HTTP/生成及失败控制。原`tf --batch`三仓历史组合可保回放；新增明确四仓模式（候选`--with-web`，**尚未实现**）必须将Web列required owner和执行腿，既有三仓pass不能认证新四仓。缺Web/缺ID/重复/0选择/坏hash/未执行required/过滤后冒充完整均非零；使用现有测试裁判，不新增“PASS”协议。只列举不算执行。

发行至少含源码/manifest/锁、模板/stdlib引用、工具producer/资源hash和许可证；提供Web独立库/工具公开入口、CI配方和可重放本地发行包。公开registry/远端CI不是本轮默认动作；若未在远端执行，明确本地CI等价入口结果，不能标远端绿。独立消费者仅固定正式依赖/候选发行集合，关闭隐式D:/autostack/main-WIP/旧target/rlib兜底，无UI/Man/Tauri检出也能完成Web基础build/test。跨仓组合可以显式带这些消费者。

优先复用既有admission/staging/恢复设施，拟新增Web适配入口`scripts/web-repository-{fixture,distribution,switch}.py`和`test/fixtures/modularity/web-repository-consumer/`（T-09落地，具体等价命名可由T-02冻结）。实际fresh解析、locked重放、A→B重定位、升级/失败恢复后再真实读回使用。缺模板/错源/hash/版本/断生成等失败必须保旧可用产物及用户文件，或按预先定义的恢复配方恢复并验证；不要求Git四仓“原子合并”，而要求明确依赖顺序、候选组合、失败停止和恢复点。

### 5.4 真实消费者与最终退出

迁移HTTP对拍用独立fixture让相同`.at`服务业务分别经VM/Rust执行，独立预期值比较，不把旧引擎输出当唯一oracle。Blog固定源经新工具链冷生成/构建/真实HTTP创建、读回、条件保存、陈旧rev拒绝及重启回放；不手改生成物。BLOG-004的Vue+Rust后端和native VM UI+Rust后端是**双UI**，不是VM/Rust双服务证据。分别核Web变更影响的Vue与native VM消费；受影响链必须实际重放，无影响复用须给源码/依赖/配置/输入未变证明。UI原主线程栈策略、OS876桌面入口与051脚本能力不因依赖切换退回；非触面可用来源/diff证明，不追加无关全桌面测试。

切换后语言/Man/UI只留具名桥、通用规则及必要兼容门面。删除旧Web原件/重复模板/过时路径和隐式fallback，源→目标可追溯。纯移仓不算净删，生产/测试/生成/第三方口径分列。未决历史红按基面与逐项证据归因；新增回归不能转为历史债，必需环境/消费者缺口不能靠“稍后”宣布交付。

### 规范增量

以下均为proposal，当前不写canonical。跨仓目标标为拟新增，frontmatter只填本仓真实相对路径，不把`auto-web:`伪装backend可解析路径。

| delta_id | add/modify/retire | target | before/after rule | rationale / AC |
|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-web/project.md` | 无现行Web仓→语言侧仅保已交付Web owner/版本/入口与canonical指针 | 不在语言仓复制Web正文；AC-01/12/14 |
| SD-02 | add | `docs/specs/repository-boundaries/web-four-repository-delivery.md` | 三仓现行交付→新增Web来源、四仓支持组合/测试/发行/恢复与有限退出 | AC-08..12/14 |
| SD-03 | add | `docs/specs/language-sdk/http-host-port.md` | Core内部HTTP直连→已验证最小端口、值/线程/生命周期；SDK服务显式安装/缺席诊断/版本迁移，普通语言调用不变；r3增加独立typed launch的调用前快照/跨线程/逐poll恢复、VM绑定与取消cleanup回执 | AC-02/03/06/07 |
| SD-04 | modify | `docs/specs/auto-api-gen/project.md`、`docs/specs/language-sdk/{project,architecture}.md`、`docs/specs/auto-media-core/project.md`（附module plans索引） | 混合语言adapter→Web/通用/非HTTP真实分工及旧入口迁移，保885合同；renderer-neutral photo单源归media基础、旧UI路径薄门面 | AC-02..05/11/12 |
| SD-05 | modify | `docs/specs/stdlib/design/{http-server,http-service-deployment,backend-assembly}.md` | 更新实际owner/入口指针，r3实例配置/真实Result/独立网络关闭与日志排空join合同，已支持语言语义/部署等级/预算/安全不变；return/file等关联Spec只在确有归属变更时补指针 | AC-03/06/07/12 |
| SD-06 | modify | `docs/specs/repository-boundaries/project.md`、`docs/specs/auto-man/project.md`、`docs/specs/auto-ui-tools/project.md` | 三仓现状增加Web正式消费与禁止反向边，Man/UI仅留各自职责 | AC-01/05/09/11/12 |
| SD-07 | modify | `docs/specs/auto-test-case/governance-and-inventory.md` | 三仓治理基线→Web迁移/四仓执行与ID守恒，887负结果与未执行限制保留 | AC-08/12 |
| SD-08 | add | 目标Web仓`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`（拟新增） | Web正式规范归Web，公开端口/行为/支持与发行、来源和测试义务可追溯 | AC-01..12/14 |
| SD-09 | add/modify | UI/Man各自`docs/web-integration.md`；现行`docs/specs/auto-ui-compiler/{project,plans}.md` | 消费方记录固定版本/兼容/恢复与测试，仅链接Web规范；补T-10局部快照绑定与按需内部状态别名，保公开getter、原局部名/委托捕获与生成指纹 | AC-05/09/10/12/14；原UI owner有界修复，无新AST/运行时协议 |

## 6. 测试设计

下表是完整验收合同。T-02 r3 已完成其可行性/端口/生命周期部分，具体运行命令、反控、冻结包/API与后续新建目的地见现行 r3 契约及交接；不能据此将整体 AC-02/03/07 或四仓交付计通过。所有拟建 Web 包/命令明确标待实现，实际迁仓后的验收仍归 T-03..12。

| ID | AC / work owner | 输入、入口与必需观察 | 边界/关闭证据 |
|---|---|---|---|
| V-01 | AC-01 / T-01 | Git refs/WIP/许可证/版本、源→目标/consumer清单、实际共享diff | 身份保全/恢复载荷可读；无未知required归属；状态头不代替交付收据 |
| V-02 | AC-02 / T-02/04 | 真实Cargo metadata/tree及独立SDK/合同消费者，合法跨包类型同一source；漏依赖/同名双source控制 | normal/build/必要dev/feature全闭包无环，无Core→Web实现、Web→UI/Man/Tauri；受管SDK源码清单同步 |
| V-03 | AC-02/03 / T-02/04 | 外部SDK与CLI；V-03a无路由无provider语言调用，V-03b有服务路由缺provider，V-03c显式安装后真实TCP/关闭，V-03d重复安装/显式覆盖与两实例隔离 | a原值/输出不变；b明确装配错误且不ready/不留下监听或任务；c精确响应及生命周期通过；d按T-02冻结的快照/覆盖规则观察实际服务，不能只查注册槽。Core闭包无Web实现；新SDK固定依赖和迁移文档与发行包一致，Man/UI命令无需用户额外操作 |
| V-04 | AC-03/07 / T-02/05 | 真实VM线程→owned请求/上传body→handler→JSON/SSE/file→终态、取消/断连/错误控制 | VM !Send不跨线程；值/句柄无悬挂，背压与许可覆盖body终态；仅JSON探针不足放行整域 |
| V-05 | AC-04 / T-06 | 885约简fixture及完整back cold generate→实际Rust编译/运行；导入、time、借用/同形类型和非法兄弟控制 | 值/路径/类型/诊断符合现行Spec；字符串/注释/custom模块不误改，无手修产物 |
| V-06 | AC-05 / T-07 | `auto service --server vm|rust`、旧`auto run`及配置/端口，UI HTTP调用 | 命令/flags/stdio/exit/ready/关闭保持，`auto serve`仍daemon；新Web生产owner可追溯 |
| V-07 | AC-04/05 / T-06/07 | Tauri IPC、sidecar、Rust UI共享workspace/source owner现有真实fixture/消费者 | 非HTTP仍可构建/消费，声明/锁/算法无副本，不能只用HTTP测试证明未迁兄弟安全 |
| V-08 | AC-06 / T-05 | 独立VM/Rust服务相同路由/参数/文本/JSON/None/错误/流/file/upload合法与拒绝输入 | 状态/体/headers/错误阶段按共同合同及具名既有差异；独立oracle，真实TCP，不以文本生成assert代替运行 |
| V-09 | AC-06 / T-05 | strict config坏字段/范围/冲突；Host/CORS/XFF/限流、health、stale生成、占口 | 既有拒绝码/阶段和支持等级保持；拒绝先于body/FS；失败不ready、不误杀用户进程；两轨已支持部署面重放 |
| V-10 | AC-07 / T-05 | 许可占满、慢体、断连、handler失败/取消、上传/文件/流、drain/期限 | owned资源/reader/子进程/线程实际结束或准确恢复；恰一次回收/计量，不能只看函数返回/置标志 |
| V-11 | AC-08 / T-08 | 原ID/子case账、Web独立真实执行与四仓模式；缺Web/0选/重复/缺ID/过滤/坏hash故障控制 | 逐ID守恒与真实执行账相合；坏组合非零，三仓历史结果不误认证四仓；既存红/skip/NOT_RUN具名 |
| V-12 | AC-09 / T-09 | Web固定正式依赖独立检出/新target build/test；外部生成工程fresh/locked/A→B重定位 | 无原工作树/绝对根/旧target隐式源；源码/锁/模板/producer/许可证完整，重定位后真实运行 |
| V-13 | AC-09/11 / T-09 | 合法旧三仓/新四仓组合；缺资源/错hash或版本/生成中断/发布失败，再恢复 | 旧产物及用户文件hash保持，或规定恢复成功且再次真实消费；恢复日志本身不足 |
| V-14 | AC-10 / T-10 | 固定Blog源码冷生成Rust→真实HTTP创建/读/保存/陈旧rev拒绝/重启 | 数据内容、版本和失败状态精确正确，工具与产物hash绑定，业务WIP不修改 |
| V-15 | AC-10 / T-10 | Blog Vue/native VM UI到新Rust后端的受影响消费；独立VM/Rust后端对拍另见V-08 | 改动链实际重放；无影响复用给四类不变证明。双UI不冒充双服务，不代替BLOG-004最终验收 |
| V-16 | AC-11 / T-08/10 | 代码/依赖/资源/consumer扫描并真实构建无旧实现组合；恢复历史包再消费 | 原Web算法/模板/隐式回退退出；具名桥和非HTTP保留有owner，移仓/净删分列 |
| V-17 | AC-12 / T-10 | fmt、适用check/定向test及新warning/debug/遗漏延后workaround扫描，SD/交接逐条核 | 新增未处理warning/必需缺口0；已知红按原基面对照，不用改golden/floor洗绿 |
| V-18 | AC-13/14 / T-11/12 | 同树独立final review冻结SHA→各仓门→实际合入→规范/账本/index/归档/guard读回 | final pass只表审查通过，merge-owned pending显式保留；四仓实际提交/来源可消费后才U890 |

角色：当前执行会话负责本次 T-02 修订/work/阶段自查；本次不创建或联系代理/聊天。阶段审查从冻结实物重建并明示同会话限制，不冒独立终审。T-11 仍由独立上下文在同树 final review；merge角色持有最终门与合入收尾。

时序：work/review仅必要`cargo check -p <实际包>`、命名单测/HTTP/生成/消费fixture和反控；全部finding修完、提交并final pass后，merge在冻结提交按各仓规则统一裸`cargo t`及实际触面`tv/tt/UI tu/Man相关档`。完整四仓批次只在指定单实例窗口按新required矩阵执行一次有效组合，复用须明确输入不变；失败归因/改码后复审受影响内容再跑受影响门，不反复无差别整档。tf到期仍由主检出单实例批量技能处理，不因本Plan四个树各跑一次；新四仓认证不能用旧三仓结果替代。Web新仓采用等价分层与确切档位，T-02冻结。

原始证据本地保存，仅摘要/命令/退出码/逐ID执行结果/版本/路径+hash沉淀；附件非md不入库。只管理本Plan创建的临时目录/端口/PID；不清共享缓存、不终止他方任务。GUI/部署/机器级测量只占必要预约窗口，不追加全套AutoGraphics或无关桌面验收。

## 7. 验收标准

| ID | owner_stage | 可观察完成条件 / 验证 |
|---|---|---|
| AC-01 | work | 完整来源/目标/consumer/测试/状态清单及身份保全、上游U/X收据齐，无未解释required项；V-01 |
| AC-02 | work | 最小合同/公开端口和单一类型身份可实际编译消费，DAG全闭包无禁向边，SDK服务按批准的显式adapter版本迁移、普通语言调用兼容及资源清单成立；V-02/03 |
| AC-03 | work | Web生产实现归新owner，Core仅保语言桥/合法标准库职责，真实VM/owned body全路径安全可用，显式安装和缺席诊断符合r2合同，CLI/UI宿主完整装配；V-03/04 |
| AC-04 | work | HTTP生成/客户端归Web，885合同及非HTTP Tauri/sidecar/Rust UI单源保持；V-05/07 |
| AC-05 | work | Man/UI经公开出口真实消费，命令/端口/stdio/exit/ready/取消兼容，Man核心无VM/renderer反向负担；V-06/07 |
| AC-06 | work | VM/Rust服务现有支持矩阵及部署/政策/错误强正反例通过，不新增支持承诺或默减原支持；V-08/09 |
| AC-07 | work | scope/预算/许可/线程/进程/流/file/upload在成功/失败/取消/断连/关闭实际终结正确；V-04/10 |
| AC-08 | work | 原子测试义务迁移无未知丢失/减弱，Web独立及四仓真实选择/执行有可信账，缺项与伪完整负控有效；V-11 |
| AC-09 | work | 目标Web仓独立闭包/可重放发行、外部fresh/locked/重定位与升级失败恢复后真实使用成立；V-12/13 |
| AC-10 | work | Blog冷生成Rust真实HTTP闭环及受影响双UI消费有版本约束实证，双服务对拍独立成立，不修改应用业务掩盖迁移问题；V-08/14/15 |
| AC-11 | work | 旧Web重复实现/模板/回退退出，兼容/桥具名且有退出或长期归属，来源历史保全/恢复可消费，净删不混移仓；V-13/16 |
| AC-12 | work | 新增warning/调试残留/未批准遗漏0，SD proposal与接口/测试/版本/限制回交完整，全部工作AC证据冻结；V-17 |
| AC-13 | review | 独立final review绑定代码/依赖/资源/配置/SD，逐工作AC与反例通过，必需缺口不转债；明确merge-owned pending；V-18 |
| AC-14 | merge | 冻结门、各实际修改仓合入与正式来源、Spec/账本/index读回、归档和逐树guard清理全闭，固定四仓新组合可消费，最终12/12才U890；V-18 |

Design50映射：WEB-AC-01→AC-01..04/11；02→AC-02..06；03→AC-06/07；04→AC-06/07；05→AC-10；06→AC-02/09；07→AC-08；08→AC-01/09/11；09→AC-13；10→AC-14。无设计必需项被“完整搬迁”名义省略。

## 8. 执行步骤

r7按用户要求整合896并重编译测试，现8/12。T-08适配新固定组与四仓实际证据，T-10重测/集成/健康核查，T-11新来源受影响独立复审；原领域实现与历史有界证据保留，不能拿旧PASS认证新来源。

| task_id | owner_stage / 依赖 | 涉及路径、产物与关闭条件 | AC / 验证入口 |
|---|---|---|---|
| T-01 | work / G890 | [✅ T-01] 已完成入口清单：唯一语言树/只读依赖、用户确认U885及原始证据边界、Web本地身份、76源码文件/4120实际声明的唯一逻辑owner、6档24869历史行/4683原ID保留及17资源hash；未冒充运行分母或最终迁出结果。提交`b3c74b165`，摘要`attachments/890/work/intake-r2.md`；原许可/来源与888/889实际diff窗口见原入口摘要。T-02仍须冻结物理包图与全路径端口，T-08实际ID迁移/执行账不得以此盘点替代。 | AC-01入口部分；V-01，Git对象/语法树/源hash/原清单核对；parser及admission控制各1/1，原证据本地hash见摘要 |
| T-02 | work / T-01 | **[✅ T-02] r3有界修订/修复与阶段验收通过；到此暂停。** launch与编译声明分离、VM前绑定；真实Result、独立网络关闭、parked事件移交桥、日志/网络join与取消cleanup回执闭合。原audit3/3、session5/5、交换配置反控拒绝、file/SSE终态/资源、5个命名单测及check/fmt通过；闭包/共享生成未触面证据有界复用。现行 [r3契约](#e890-1a20e94b922d0662) / [冻结交接](#e890-8caf4976a4ad8156)；代码`f56211c12`、交接`4ec696a63`。R5/R6冻结的是后续包/API/schema执行合同，未冒实际建Web或四仓发行。阶段自查同会话限制明示，独立终审仍T-11 | AC-02/03/07仅T-02部分；V-02..04及V-05/07最小兄弟控制；完整迁仓/CLI/UI/发行仍各原任务，T-03需用户另行授权 |
| T-03 | work / T-02 | **[✅ T-03]** 用户已启动本单元；888正式交付/归档/收尾祖先已核，clean原分支已合入master为`29ed9d7bd`。本地Web普通仓`D:/autostack/auto-web` bootstrap `8ff4966`，唯一树`D:/autostack/.wt/lang-890/auto-web`、`codex/plan-890-dev`，基础包/锁/文档及CI接线契约骨架提交`2625964`。合同/框架基础已真实check及metadata闭包验证，来源/MIT/原Git锚保留，无远端/registry/CI部署。基础包不冒runtime交付；T-04定向验证及完整门限制见下一项 | AC-01/02的T-03部分；基础包check、482包normal/build/required-dev闭包无UI/Man/Tauri/混合facade，单Core/合同/共享载体来源，完整独立发行留T-09 |
| T-04 | work / T-03 | **[✅ T-04；基础实现/定向验收完成，完整tm门未通过]** 基础代码`84779b2b1`、测试装配修复`1a2539508`：纯合同实际迁Web、四共享模块迁auto-generation-core，旧路径薄转发；Core真实Cargo来源入SDK身份清单。实际类型/Arc消费、严格配置、原15单测、生成companion正反例及T-02 audit3/session5通过；tm4个直接叶扩大750模块/5483义务；执行55通过/3既存红/23原条件不适用后，旧HTTP测试装配挂起中止，剩余未执行。已显式修复旧SDK测试装配并完整重放直接23项及HTTP4项，均通过；不冒完整闭包通过，T-12在最终新源身份重选并执行，原ID/断言/条件保留。Core `lib/entry/run_bridge/host_capabilities/http_server*`按冻结端口接线，纯合同/语言值转换与最小VM桥、必要共享生成承载/SDK源码闭包；按r2完成SDK服务显式安装迁移、普通语言兼容和明确缺席诊断 | AC-02/03；V-02..04，实际包check/命名单测及外部SDK消费 |
| T-05 | work / T-04 | **[✅ T-05；R2文件入口配置修复，原transport/资源证据保留]** 最新修复与当前输入见[修复2交接](#e890-2709e784f3aa9c94)；下列原阶段数据按原输入保留。[r6交接](#e890-ca5b663d4f4f80ee)。Web真实host/transport/scopes/file/upload/policy/session归新owner；Core临时桥退出；runtime隔离nextest109/109、实际包TCP/关闭和上传快照正反控2/2、audit3/session5及文件/SSE/12MiB/RST/双实例/legacy通过。根组合check三项UI旧引用失败保留，非全AC通过。Web `src/`接管HTTP领域runtime/配置/观测/路由与两轨装配；Core旧领域逻辑逐消费者迁出，handler/scope/file/upload/许可完整终态保持 | AC-03/06/07；V-04/08..10，真实TCP正反/资源控制 |
| T-06 | work / T-04 | **[✅ T-06；迁移/定向验收，CLI/UI消费待T-07]** 代码语言`1b177ebab`、Web`d8c889c78`；HTTP生成/targets/模板归Web，IPC留原owner/shared四模块单源；生成48通过/1原ignore、targets18通过、实际cold生成/编译/TCP/恢复1通过、原file/upload/JSON-SSE wire3通过，Tauri具名2通过。原按需ignore和完整ID/门仍pending；见[r6交接](#e890-ca5b663d4f4f80ee)。拆`auto-api-gen/src/{api_gen,rust_workspace,sidecar,source_owner}.rs`真实归属，HTTP生成/客户端/模板切Web，885控制与非HTTP/共享规则不复制，必要兼容门面无环 | AC-04；V-05/07，约简与实际生成Rust编译/执行 |
| T-07 | work / T-05/06 | **[✅ T-07；消费迁移/定向验收，整应用T-10/最终门T-12保留]** Man/UI正式依赖与直接HTTP调用切Web，显式安装provider并跨线程保存typed launch；Core形态闭包534包、禁止UI/大门面/Tauri边0。真实CLI VM和Rust两轨精确响应/port0配置哈希/关闭退出/端口释放，UI真实VM worker1、proxy4、Vue流生成1及Man命名5均通过；根/UI/Man check通过。原daemon命令、非HTTP/共享单源保留。语言`163d342a6`、UI`672955d`、Man`9f8ffc2`，[收据/限制](#e890-1f9b138c564b0993)；冻结完整组合及Blog双UI不冒提前认证 | AC-05的受影响消费接线；V-06/07定向实证；整应用和完整闭包仍T-10/12 |
| T-08 | work / T-07 | **[✅ T-08；r7固定组入口及可信准入已实现/定向验证，最终当前组合门T-12 pending；以下旧证据为历史；修复3 managed worker/目录/原ID守恒已验证，见修复3交接；修复2共享原ID/floor/迁移执行守恒与合法ignore已验证；待独立复审]** 最新修复与当前输入见[修复2交接](#e890-2709e784f3aa9c94)；下列原阶段数据按原输入保留。117迁移ID及696历史行/116唯一ID、17资源核；10原UI照片测试迁至media且原测试子树hash相同。46 recipes/7320义务/1211叶已真实发现；Web217过+2原ignore，保留语言28过、媒体54过、当前治理反例9过。真实四仓清单全部ID/floor核通过，partial/list-only不认证交付的实际负控exit2；不降floor、不退役原义务。语言`38f68d8e9`、Web`f4cc93c`，[收据/原红/限制](#e890-6f447d9485de823a) | AC-08/11的T-08部分；V-11/16；最终冻结实际批次T-12仍NOT_PASSED |
| T-09 | work / T-08 | **[✅ T-09；R1C/R2有界修复与新SDK/CLI验证完成，最终组合收据T-12保留]** 最新修复与当前输入见[修复2交接](#e890-2709e784f3aa9c94)；下列原阶段数据按原输入保留。19包纯SDK自包含源/Windows包已真实构建、测试、fresh/locked/A→B及故障恢复；冻结发行45/45、历史Core恢复14/14，SDK217过+2原ignore。Man事务唯一源/不可编辑工具快照、完整stdlib/版本/来源/哈希与严格Core/Full边界；公开反例4拒绝且旧workspace不改。真实Git独立检出构建/库测试及Windows目录限制/LEAK分列保留；治理10与用途19反例通过，未降旧floor/原条件。[包/哈希/范围](#e890-4d314850cd8b40e6)；语言`f739c3751`、Web`9ddb7fe`、Man`cb4ab3d` | AC-09/11；V-12/13；旧架构仅Core合法形态，未冒旧UI/Full运行认证；T-10/11/12继续 |
| T-10 | work / T-09 | **[✅ T-10；r7整合/首次统一门修复已定向关闭，当前组合最终门T-12仍pending；以下旧证据为历史；修复3诊断/原生闭包/独立guest及健康检查完成，见修复3交接；修复2原因诊断/oracle调整及41+19验证/目录/SD新冻结完成]** 最新修复与当前输入见[修复2交接](#e890-2709e784f3aa9c94)；下列原阶段数据按原输入保留。固定Blog原314文件未改，真实冷Rust构建/API13及完整Vue15/15、原生22/22；新增UI局部快照/同名状态修复`9aad12c`，原ui_gen模块866过+2原ignore、4反例。语言`ede598998`刷新46 recipes/7329义务/1213叶，75职责组、19守恒用途控制；新增warning差0、旧debug/TODO来源核与SD01..09回交完成。原红/路径/缓存/控制器清理限制具名保留，[冻结交接](#e890-6aafd921a0a45c59) | AC-10..12；V-14..17，工作AC自查不冒独立终审/整档门 |
| T-11 | review / T-01..10 | **[✅ T-11] T19最终受影响独立PASS**；原媒体断言保全、Windows owned build、非Windows信号组保留及来源受限恢复脚本通过。报告[最终复审](#e890-d0511e724d4bee1e) SHA2568c695e50599d6df83e228e98461162fb2c23d54da1268e98eff1cd852a9c054f；冻结54b5/e144/4da/d2b/c346。完整实际执行/SDK/landing/archive/cleanup仍T-12 | AC-01..14；V-18审查部分 |
| T-12 | merge / T-11 | 再核冻结组合/写入窗口，按T-02依赖顺序和恢复点跑各仓统一门+四仓完整认证，实际合入各修改仓；Web canonical、语言指针、UI/Man消费文档及各自ledger/index沉淀；主合同归档后逐树guard clean再安全清理，最终来源/产物/ID/状态读回 | AC-14；V-18，auto-plan-merge流程；批次到期按现行单实例规则，收尾后12/12 |

进度记法：每任务实际完成才登记`[✅ T-NN]`与提交/命令/退出码/结果/证据hash，不因任务表存在打勾。T-10/T-11不提前要求merge-owned实际合入/归档；T-12在归档时若清理未完仍11/12，guard收尾读回后才12/12。各仓实际合入不可能作为同一Git原子事务；若中途失败，保候选/原有效组合和恢复路径，修复复核后继续，不宣称U890。

## 9. 复审记录

2026-10-29 / stage: new / plan_id: PLAN-890 / plan_revision: 1 / outcome: pass（合同静态预检，非实施验收） / scope: 0–10章节唯一有序、12任务/14AC/18V/9SD唯一连续且引用有效、14AC均有任务承接、任务依赖无环、Design50十项验收映射、65个当前本地链接存在、现行Spec目标及5个源码/Spec hash一致、890唯一且号源891、围栏/行末空白/限定diff检查通过；历史折叠正文和未改历史段落的失效链接不冒称本轮已修复 / implementation: 全部未执行 / next: 用户审阅；后续明确执行且G/X齐备才work T-01。885最终收尾锚/端口兼容/共享生成可行性仍由具名任务关闭，静态pass不放行迁移。未创建目标仓/实施树/agent、未运行项目测试或构建、未联系他方、未提交或push；其他会话865归档簿记WIP保持原样，未加入本Plan改动。

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 1 / outcome: blocked / scope: T-01启动核查，任务未关闭 / authorization: 用户 `/goal 完成计划890` 授权原合同内work→独立review→merge→清理全链；890与888无交集面可并行，共享脚本/注册/锁/最终合入短窗串行，UI/Man结构包继续等U890 / base_commit: eda05a86f2f6bd29d3b70479917360d64a908a66 / code_commit: none / worktree: 未创建（G890未闭） / evidence: `git status --short`主仓无产品代码WIP，既有文档WIP保留；`git worktree list --porcelain`确认888唯一树已存在、885树未注册；`git branch --list '*885*'`无分支，`Test-Path D:/autostack/.wt/lang-885`与子树均False；885归档frontmatter archived7/7但T-07正文仍未勾，末尾merge收据仍为`cleaned: 待 worktree guard 后移除`。本地attachments/885不存在；所查路线/原规划会话仍仅记录此差额，未取到guard实际退出码/输出锚 / blockers: G890中的U885完整收尾证据未闭（非888全局等待，不是技术否定结果）；不以目录消失伪造历史guard通过，不代填885收据 / next: 提供885实际guard/清理原始记录锚后继续T-01；若原始证据不存在，需用户明确裁定如何处置这项历史证据缺口。未改产品/共享脚本、未建仓树、未构建测试、未发消息或建agent。

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 1 | from: work | to: user | status: drafting | outcome: blocked | worktree: 未建 | commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | next: user（G890/U885历史收尾锚；授权已接受，不重复请求执行授权）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 2 | from: user | to: user | status: drafting | outcome: waiting | worktree: 未建 | commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | next: user（2026-10-30续轮重读§9/§10，未见885 guard/清理锚或历史缺口裁定；主仓HEAD/工作树注册不变。上一轮分类progress：完成首次入口核查并形成具体阻断证据；本轮无live process，不属于verified wait。相同G890/U885缺口连续第二轮，未执行阶段工作/构建/测试，未创建仓树或更改其他Plan；继续等待具体证据或裁定。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 3 | from: user | to: user | status: drafting | outcome: blocked | worktree: 未建 | commit: eda05a86f2f6bd29d3b70479917360d64a908a66 | next: user（2026-10-30第三轮重读890§9/§10与885末尾收据：仍未有实际guard/清理锚或历史缺口裁定，原收据仍cleaned pending。上一轮分类no progress：仅状态重核与等待簿记，无live handle。相同G890/U885阻断连续三轮，满足goal blocked审计阈值；停止自动续跑。恢复需提供原始证据锚，或用户明确裁定允许登记历史证据缺口后继续；890未完成，0/12不变，未执行阶段工作/测试或修改他方Plan。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 4 | resumed_blocked_audit: 1/3 | from: user | to: user | status: drafting | outcome: waiting | worktree: 未建 | commit: b5556e1696d029d0b4deca8748f985018be5740c | next: user（2026-10-30恢复轮：goal实际active，按新blocked审计计数。重读890/885仍无缺口裁定，885原收据cleaned pending。追加只读恢复调查：rg扫描本地Codex sessions/archived_sessions中lang-885来源，除本会话外为规划/其他计划引用；具名guard-clean模式未找到885实际执行记录；Claude sessions亦无lang-885。未用“无匹配”证明历史未执行，仅确认仍未恢复所需证据。上一轮分类no progress：阻塞审计与簿记未改变启动门；本轮产生新的恢复调查结果但G890仍未闭。888已推进独立提交，890未触其写面；不运行阶段构建/测试、不建仓树、不代补885历史记录。待提供原始guard/清理锚或明确历史缺口处置裁定。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 5 | resumed_blocked_audit: 2/3 | from: user | to: user | status: drafting | outcome: waiting | worktree: 未建 | commit: b5556e1696d029d0b4deca8748f985018be5740c（沿用上轮观察，不新增代码身份断言） | next: user（2026-10-30续轮仅重读890§9/§10与885末尾收据，缺失的guard/清理证据和处置裁定仍未补入；同一G890/U885阻断恢复后连续第二轮。上一轮恢复调查未改变下一步且未解门，本轮按no progress处理，无live process可等待。未执行阶段工作或检查，不再重复已做的全局证据搜索；仍需实际原始记录或用户对历史缺口的明确裁定。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 6 | resumed_blocked_audit: 3/3 | from: user | to: user | status: drafting | outcome: blocked | worktree: 未建 | commit: b5556e1696d029d0b4deca8748f985018be5740c（上次观察锚） | next: user（2026-10-30恢复后第三轮重读890待澄清项及885清理相关行，T-07仍未勾、cleaned仍pending，无新增证据或用户裁定。上一轮no progress，无live handle；同一启动门缺口连续三轮，满足再次blocked阈值。未执行阶段工作/构建/测试；目标未交付，0/12不变。恢复条件仍为885实际guard/清理证据锚，或对历史证据缺口的明确处置裁定；停止重复自动续跑。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 7 | resumed_blocked_audit: 1/3（第二次恢复） | from: user | to: user | status: drafting | outcome: waiting | worktree: 未建 | commit: 无新增观察 | next: user（goal重新active；只读重核890§10及885 cleaned行，无新增证据或裁定。上一轮no progress；本轮不重复搜索/阶段检查。已通过用户输入工具提出具体二选一处置问题，等待真实提交答复，预选或自动续跑均不算同意。G890未解，0/12不变。）

2026-10-30 / stage: work-resume / plan_id: PLAN-890 / plan_revision: 1 / authorization: 用户明确答复“885已经收尾了。” / outcome: pass（解除G890中的885收尾阻塞，非T-01完成） / interpretation: 接受用户对上游实际完成状态的确认，原始guard输出未找到仍诚实保留为历史证据边界，不伪造通过记录、不改885历史收据；不再以此阻断890或重复询问 / base_commit: b3f43a7c4a392a6652b70c9b79db5cedb5589047 / next: work T-01 来源与X窗口冻结、创建唯一语言树；888独立工作继续，共享脚本/注册/锁/合入串行，UI/Man后继等U890。

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 1 / outcome: blocked（新发现的D-04兼容决策，非885历史阻塞） / worktree_path: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / base_commit: b3f43a7c4a392a6652b70c9b79db5cedb5589047 / code_commit: 6d70b2267（真实Core-only HTTP消费者基线） / evidence_commit: afea065b203e7c5f842cc874b8e67c72c8130f35 / dependency_revisions: auto-down c3464d0；UI f7c83c6f/Man e6cc06fb/Blog 7f9b6755为只读观察锚、非最终冻结组合 / task_ids: T-01进行中；未关闭T-01/T-02或任何AC / evidence: 新建唯一语言树+只读down兄弟，建树脚本WSL登记以Windows git worktree repair修复，两树未建链接；root/branch/status读回正确。`cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --locked -j 4` exit0；实际TCP探针exit0、HTTP200精确随机token、shutdown delivered=true、子进程exit0；normal/build/dev依赖树无Web/UI/Man/Tauri；fixture fmt与diff检查exit0。原样Core编译报告147个既有warning，探针0新增；不声称已清全部warning。日志/依赖树hash见 attachments/890/work/sdk-default-decision.md；本地governed-inventory.json SHA256 8a30f4dccddb4a22bc27b268a22e7339aedf9a9f5948454d90da88f843a81415 为既有盘点输出（executed=0，不是本轮测试通过）；历史六档候选1161行保tier未去重，不冒原子分母或最终owner，web-obligation-candidates.json SHA256 79021aeebed36341fac162ae7190042ef6088fb8fd9e1107c99034c31f83acbd。 / findings: 当前仅Core依赖的公开run_with_capture自动提供HTTP，r1又禁止Core→Web实现边并要求全部实现迁出；仅让Man/UI安装provider不覆盖这个已验证SDK消费。详见已提交具体两方案决策说明；未以缺provider默退化或旧实现副本解决。 / shared_window: 已查888实diff含entry、HTTP支持函数/stream、注册/公共脚本；本轮仅新fixture与890自身摘要，零共享文件写入，未借888 WIP，未发跨聊天消息/建agent / blockers: 需用户选择显式SDK版本迁移，或允许旧Core兼容门面委托Web并独立保持底层语言owner；问题已通过用户输入工具提出，等待实际答复；不自行修改兼容承诺 / next: 答复后按auto-plan-new受控修订D-04/包图与受影响验收，再回work；源码/测试逐义务清单和完整owned-body端口仍待完成，非T-02 pass。merge-owned统一门及完整四仓门全部pending。

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 8 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: afea065b203e7c5f842cc874b8e67c72c8130f35 | next: user（上一轮no progress；本轮用户确认885后已实际建树/提交/真实HTTP验证，属于progress。旧历史阻塞已解除；现为新D-04公开兼容决策的首次阻塞，不能沿用旧阻塞计数。0/12保持，所有必需交付仍原范围，UI/Man结构后继继续等U890。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 9 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: afea065b203e7c5f842cc874b8e67c72c8130f35 | next: user（2026-10-30重读§9/§10与权威树，D-04未有实际答复，树clean且提交未变；上一轮为progress：旧885阻塞已解除且已产生真实探针/提交/具体兼容证据。本轮no progress，无live process；新D-04同一兼容决策阻塞连续第二轮。未执行阶段工作/构建/测试，未创建Web仓或改写生产支持承诺。簿记纠错：恢复r1第1–7轮原始status:drafting，修正恢复实施时全局字符串替换误及历史收据的笔误，未改当时结论；当前frontmatter仍executing。）

stage: goal | plan_id: PLAN-890 | plan_revision: r1 | round: 10 | from: user | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: afea065b203e7c5f842cc874b8e67c72c8130f35 | next: user（2026-10-30第三轮核实D-04兼容方案未答复；权威树clean、HEAD不变。上一轮no progress，无live handle；本轮未执行阶段工作或测试。同一D-04阻塞在第8–10轮连续三轮，满足blocked审计；885历史阻塞已关闭，不计入本次三轮。保留基线探针与具体两方案说明，尚未交付890，不修改兼容承诺、不合入/归档/清树。恢复需用户选择显式SDK迁移或兼容门面包图方案，再受控修订并继续原完整交付。）

2026-10-30 / stage: new（bounded revision） / plan_id: PLAN-890 / plan_revision: 1→2 / outcome: pass / authorization: 用户明确“显式安装 Web adapter” / changes: §1 G2、§2.2、§4.1、§5.1/SD-03、§6 V-03a..d、AC-02/03、T-02/04按显式装配更新；Core继续零Web实现反向边；SDK服务添加固定Web依赖并显式安装，缺provider明确拒绝且无伪ready，普通无服务路由语言调用不变，Man/UI原服务命令在宿主组合点安装。只批准这项装配兼容迁移，未改变其他服务行为/资源预算/支持等级，完整Web迁移及四仓交付义务保留 / evidence: 6d70b2267历史Core-only真实TCP默认基线与afea065b2决策说明保留为before证据，不复用为r2安装后验收；源码/完整生命周期/测试逐义务清单仍须T-01/02关闭，不能以决策替代实现 / worktree: D:/autostack/.wt/lang-890/auto-lang，plan-890-dev，HEAD afea065b203e7c5f842cc874b8e67c72c8130f35（clean） / next: work继续T-01/02，冻结安装API/快照/覆盖规则及共享生成DAG后做全路径可行性；不采用兼容门面反向边、不重复请求该选择的批准。宿主get_goal仍返回blocked，须由用户恢复goal续跑；这与D-04语义阻塞已解除分列。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 11 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: afea065b203e7c5f842cc874b8e67c72c8130f35 | next: work（用户实际裁定已到，D-04兼容阻塞解除；本轮修订当前执行合同，未宣称端口实现或任何整任务完成，历史收据保持原revision。）

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 2 / outcome: pass（T-01入口冻结；work整体未完成） / worktree: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / commit: b3c74b165 / task_ids: T-01 / evidence: intake-r2.md；Rust语法树区分生成字符串与真实声明，保scope/attrs及具名macro边界；admission合法/重复manifest/未知source控制实际通过；76文件4120声明全部有唯一逻辑owner，原六档全部24869行保持（4683原ID，executed=0），新旧测试与物理迁移仍由T-02/T-08验证。精确hash见摘要，非md证据未入库。 / dependency_update: 根workspace解析要求额外只读UI兄弟树，已用Windows Git建立D:/autostack/.wt/lang-890/auto-ui detached d53f946ab89ce6bb3cbbb2ea929f094ff2600be6，status clean，冻结UItools源hash与盘点一致0漂移；不包含主检出WIP、不改UI产品，后续若需修改沿用此唯一树。 / next: T-02 有限owned-body探针与host安装/生成DAG；Core新改三文件不在观察到的888/889实际diff中，共享注册/入口/脚本尚未写入；统一门与最终四仓认证pending。

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 2 / outcome: pass（T-02有界实验单元；work整体与T-02均未完成） / worktree: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commits: b80902eb2（owned上传流）、b4bb6ffc3（纯配置合同）、c8be230f011896901e737dfbf9ffa6a6b5d360e1（显式local host原型） / task_ids: T-01已闭，T-02进行中，current_step=1/12 / evidence: owned_upload_命名2/2（跨真实线程、队列满拒绝、惰性读取、错误与恰一次Drop）；真实原730程序12MiB上传/精确下载/409保原件/422业务拒绝/headers-only401及staging清理和关闭通过；独立纯合同消费者1/1、原Core配置模块14/14（含限速TTL与严格配置）通过；普通SDK无host42/exit0、有路由无host明确错误/exit1、显式安装JSON200精确token/关闭exit0及上传矩阵通过；实际serve_registered future稳定选择/重复安装保护与不同local owner1/1；HttpVmOwner:Send编译反控按E0277/Rc<AutoVM>实际拒绝exit101。13份本地log hash与摘要读回一致，非md证据未暂存；详情见三个r2实验摘要。 / scope_boundaries: 核心服务仍有临时run_existing_runtime_for_probe单源桥，T-05必须移除，不是假装框架已迁出；完整metadata/resource双实例与全部终态、直接listen/back-proxy消费者、生成共享DAG/非HTTP正反控、四仓schema与实际独立新Web闭包仍pending。当前Core产品142既有warning（比原147少5），未宣称全仓无warning。 / shared_window: 写前确认lib.rs/host_capabilities.rs及三个上传seam文件不在观察到的888/889实际diff；未写其entry/注册/aliases/governance/HTTP测试支持面。UI兄弟仍只读，UI/Man结构包未启动。 / next: 继续T-02窄VM执行/metadata/资源端口与共享生成DAG，再全路径证明通过后才T-03建Web仓；无新用户阻塞、不重复请求显式adapter授权；所有merge-owned统一门与四仓认证pending。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 12 | from: work | to: work | status: executing | outcome: pass（真实实现与定向验证进展，非work-complete） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: c8be230f011896901e737dfbf9ffa6a6b5d360e1 | next: work（目标actual active；T-01 1/12，T-02继续。已解除的885/D-04阻塞不再等待，无live build进程。未review/merge/归档、未宣称U890。）
2026-10-30 / stage: work-resume / plan_id: PLAN-890 / plan_revision: 2 / authorization: 用户再次明确发送 `/goal resume`，恢复此前全链目标；885收尾与显式adapter选择均已解决，不再询问 / reconciliation: get_goal仍显示blocked（宿主元数据观察，不能由update_goal自行改为active），不将它误记为新的技术阻塞，也不再重复要求同一恢复指令；按本次直接恢复指令在既有唯一树继续work。默认12轮之后的继续来自用户本次明确恢复，非自行增加合同范围；既有计数与历史收据不重置。

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 2 / outcome: pass（3个有界实现单元；T-02与work整体仍未完成） / worktree: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commits: 78ada233b（共享生成DAG seam）、9ee280cba（owned消息/上传资源）、5403d0fa3（活动VM元数据隔离） / task_ids: T-02进行中，仍1/12 / evidence: 共享原始三模块独立消费者初始16/16，后续workspace触面5/5（服务无UI、保留已有UI成员、缺显式源不覆盖原manifest）；原UI photo源码脱离UI crate独立10/10、私有TEMP/TMP缓存；消息原源码独立线程消费1/1且闭包无Core/VM/Axum/UI/Man；实际TCP同步/异步文件/HEAD/Range/ETag/404/500/SSE顺序EOF通过；更强终态oracle发现原730 early401遗留上传句柄1，注入前绑定scope后原12MiB/409/422/headers-only401及新合法路径绑定400均通过，scope/file/upload/stream/live-op全归零；实际双VM先复现第二程序覆盖同名参数后第一200→400，按实例快照并逐poll恢复上下文后JSON/file/SSE/headers-only上传保持，第一独立关闭、第二继续200，最终两future join且登记归零；上下文Pending/嵌套/受控unwind命名测试1/1。SDK fixture最终build exit0，Core原有142 warning保持，非新增；未跑整档。三个新摘要含命令、退出码、数字、原件与本地log SHA256，已同步主树attachments/890/work/*.md，所有非md证据保持local-only。 / shared_window: 主树仍仅文档WIP，888/889现有变更未纳入；root http_server.rs仅现有serve_with的crate内可见性、metadata传递及模块注册，未写888的spawn_outcome_reporter/测试注册/共享脚本与锁；只读UI树仍clean。 / remaining: 编译producer全局scratch、axum/CORS/rate/观测及准入许可隔离、完整取消/断连/期限、direct listen/back-proxy、最终窄VM执行端口/真实物理包DAG/API/四仓schema仍待T-02闭合；临时runtime probe入口T-05必须退出，不能据局部绿建Web仓或标execution_done。 / next: work继续T-02；独立review、所有merge-owned冻结门与四仓认证/合入/沉淀/清理均pending；UI/Man结构后继仍等待U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 13（用户明确恢复后继续，历史不重置） | from: work | to: work | status: executing | outcome: pass（定向实现进展，非交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 5403d0fa3 | next: work（既定T-02继续；无新的用户决策阻塞，仍1/12，不宣称目标完成）

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 2 / outcome: pass（2个有界实现单元；T-02及work整体仍未完成） / previous_turn_classification: progress（上一轮3个实际提交与修复前后证据改变后续行动，非状态重述） / reconciliation: 本轮get_goal实际active，上一轮blocked元数据观察已由宿主恢复，不再询问或建立新目标 / worktree: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commits: 6cba1e6f2（每服务生命期许可）、bcf661295（真实外部包运行端口） / task_ids: T-02，仍1/12 / evidence: 真实双VM一名额下64MiB未读文件先复现A=503且B=503；每服务持有TransportConfig预算后A=503、B=200，RST后A恢复200、独立关闭及资源归零；原队列满503/Retry-After命名测试1/1。外部包引用原服务源码的边界核查先记录16个实际访问/路径错误，再以既有公开VM SDK、单源owned metadata、11个具名异步/资源操作及host-owned RequestContext打通；未扩大VM字段/方法可见性或复制VM/AST/Store，未公开原私有模块/注册表。external-runtime模式不再调用Core临时启动桥，真实TCP同步/异步文件/HEAD/Range/ETag/404/500/SSE、原12MiB上传及409/422/headers-only401/绑定400均过；新增上传取消必须实际出现非空暂存再RST，清理无残留且不发布截断内容，等待时marker仍响应。双VM实际验证重复安装保护、显式替换provider、清空默认槽后活动服务继续、metadata/file/SSE/upload分类和许可隔离及独立关闭；VM/上下文guard两项Send编译反控均以E0277实际拒绝，missing-host child1/plain42 child0保持。原生资源嵌套context/恢复parent/取消后迟到上传即时Drop命名测试1/1，4 Drop/0 poll且许可恢复；最终外部模式build exit0、最后本地guard类型上重跑三类TCP均exit0，外部scope表及实际Core file/upload/stream/live-op登记归零。Core既有142 warnings不变，外部源码探针18项既有源unused/dead警告未冒称生产健康完成；18项警告与尚未消费的配置/直接监听等路径留待实际迁移收束。两份摘要已复制主树，命令/退出码/数量/本地hash见instance-admission-r2.md与external-runtime-port-r2.md，非md证据未暂存。 / shared_window: 无主树产品WIP，UI兄弟树clean；未写888/889产品变更、共享测试runner/注册/锁、UI/Man结构后继；本轮Core原有子模块仅显式标路径以供同源外部核查，未接管888测试支持helper。 / remaining: T-02尚待配置/观测/control、axum builder/legacy rate/直接listen和back-proxy、生成器HTTP/非HTTP完整函数切口、最终包/API/命令/SD与四仓schema冻结；Core仍编译旧框架/原executors且临时入口T-05必须移除。外部源码probe不是Web独立仓交付，也不替代T-08原ID/dev闭包迁移或T-09独立发行。 / next: work继续T-02冻结剩余合同，再通过技术门后T-03；全部review/merge-owned门及实际交付/归档/guard收尾pending，无新用户决策阻塞。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 14 | from: work | to: work | status: executing | outcome: pass（实际边界与资源闭合进展，非T-02/整体交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: bcf661295 | next: work（目标actual active；仍1/12；无live build或未收割子进程，无新增阻塞）

2026-10-30 / stage: work / plan_id: PLAN-890 / plan_revision: 2 / outcome: pass（配置服务状态单元，非T-02整体关闭） / previous_turn_classification: progress（已提交真实跨包运行端口及独立许可，改变本轮行动） / worktree: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commit: 38b6a8178 / task_ids: T-02，仍1/12 / evidence: ServiceRuntime独立持有observation与一次绑定的shutdown，身份/状态/计数/sink单源实例化，旧free函数为具名进程兼容facade；PendingRequestEvent持有自己的observation，完成/取消/bytes共用恰一次闸；逐poll服务策略恢复防legacy wildcard串入配置服务。外部包真实双配置/随机值/端口证明身份四元组与hash稳定、A流量不改B计数、CORS按现行Spec、headers-only shutdown202关闭A且B仍ready，两个owner join且资源归零。共享observation工厂变异实际被相同身份断言拒绝，仍正常排水；未把双监听当隔离证据。观测模块5例及两个新配置命名例共7个唯一unit通过，后续ID/两种终态顺序复验为其中1例不重复计数；旧外部file/SSE/upload/RST/许可隔离三类定向回放均过。Core warning142→141（删已触文件的unused import），外部源probe18→17，不冒称最终生产健康。探针最初sink16/实际GET期待403分别属于非法输入/错误oracle，按既有合法范围和Spec §4改为64/实际GET无allow头+预检403，没有改产品政策洗绿。摘要configured-instance-state-r2.md已同步主树，含精确命令/退出码/本地SHA256/复用边界；非md证据未入库。 / coordination: 主树产品路径clean；主HEAD独立推进b0a6c869d且889树已由owner移除，实际inventory重核后未重建/接管；888树独立，未写共享runner/测试注册/锁或UI/Man结构包，未rebase/merge/push。 / remaining: legacy进程诊断/关停facade与多legacy实例、直接listen/back-proxy、legacy rate/axum builder/producer scratch、精确deadline、生成器与最终包/API/SD/四仓schema合同仍属T-02；logger线程终结不以本轮owner返回冒证，框架/执行器实际迁出T-05、测试迁移T-08、独立发行T-09仍pending。 / next: work继续T-02，优先直接服务入口及剩余实例上下文，完成技术门/冻结后才T-03；无新增用户决定阻塞，全部review/merge-owned门与交付收尾未执行。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 15 | from: work | to: work | status: executing | outcome: pass（有界实现与真实正反证据，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 38b6a8178 | next: work（goal active；仍1/12；工作树clean，无live build/子进程，继续T-02）

### Goal round 16 — T-02 borrowed blocking listener port（2026-10-30）

代码提交 `d861642a5`：给显式 HttpServiceHost 增加借用同线程 VM 的 blocking port，缺席/仅异步 provider 均绑定前明确诊断。实际语言原语 shim 与旧 SDK unit-return 入口调用同一选择逻辑；两份同义监听循环收敛为一份、媒体 wire writer 随之单源，通用媒体 lookup 留 Core。新任务先脱离 DashMap guard 再 try_lock，兼容 Tokio 所在线程并避免错误移除时自锁；无扩大 VM/private module 可见性。外部 crate source probe 实际执行这份循环，不调用 Core 临时 startup bridge。

五模式真实 probe 均 exit0：注解 handler 精确随机 JSON/缺参400/非法整数400/404；直接原语安装后404/受控停止/同线程/handler task无增量；直接原语无安装、仅异步 provider、注解无安装三反控 child1/明确诊断/端口仍可绑定。`cargo check -p auto-lang -j4` exit0；原3项定向测试各1passed/0ignored（host选择、metadata Pending/nested/unwind、媒体wire断言）；locked外部fixture构建exit0，Core141/root3/external17等存量warnings不冒最终健康通过。未跑整档或GUI。命令、原始log SHA256与边界见 [blocking摘要](#e890-c98ba41c7d4b5e99)。

初始语言源码 `use auto.http; server.listen(...)` 在到达native前发生既有 Server.listen 名称解析错误，未记通过；直接原语反控使用既有公开 SDK alias，不以其替代最终语言消费验收。语言级直接调用、direct返回后的post-main协调、不同语义legacy stdnet入口、实际back-proxy消费、剩余实例状态/producer/deadline、完整共享生成与physical包/API/SD/四仓schema冻结仍T-02，未经批准未延后。T-01历史原子账需T-08按最终owner逐项对账；未创建Web仓，未动888共享注册/runner/锁/最终合入窗口，UI/Man仍等完整U890。独立final review/merge-owned门与交付收尾未执行。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 16 | from: work | to: work | status: executing | outcome: pass（有界实现与真实正反证据，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: d861642a5 | next: work（goal active；仍1/12；继续T-02）

### Goal round 17 — T-02 shared generation mechanism cut（2026-10-30）

代码提交 `7c5a98804`：混合 api_gen 的25个共享函数收敛到实际生产 `shared_generation.rs` 单源模块，旧公开路径re-export、私有调用保私有。范围包括锁/装配identity纯规则、full/lenient抽取、带source_dir/proofs转译、runtime源码限定、companion/db后处理和公共类型提升；HTTP模板/服务装配与Tauri IPC装配不混入。按父Git对象精确切片重建、只施加具名可见性及rustfmt后全文byte比较通过，函数体零变化。选定语言侧具名受限载体proposal `auto-generation-core`0.1.0（四模块：shared_generation/rust_workspace/sidecar/source_owner），完整T-02冻结通过后才T-04物理迁包，不能据此提前建Web。

独立source probe的17项原单元通过；新增真实外部Cargo消费1/1：实际转译导入的两模块、精确环境值/leaf运行、去掉type promotion产生E0308 types::Work/works::Work、去掉source_dir产生repository.rs &str/String E0308、恢复原生成物后再次实际运行。完整metadata480 packages无auto-api-gen/UI/Man/Tauri，含normal/build/dev；只属本地闭包证据，非fresh发行或跨OS执行。原APIgen定向ID12passed/1父版本复现失败/0ignored，包含字符串/借用编译oracle及Tauri mock dispatcher；root/APIgen cargo check分别exit0，存量warnings保留。初始fixture对Env lowering必产非空proof的错误假设已移除，不以空proof冒引用闭包认证。

为读取原Tauri fixture建立Man唯一同寿命只读detached组树 `D:/autostack/.wt/lang-890/auto-man`，准确pin `e6cc06fbc496f366edde7929abfd49254ec13647`；main/组树clean，无AGENTS/链接/产品改动/分支，后续如接线则在同树转Plan分支，最终需guard清理。不是UI/Man结构后继启动。原fixture比较先因缺组树失败，补齐后发现generate_handler宏纯换行byte差异。临时恢复api_gen/lib为父提交d861642a5的原始代码运行同原测试，得到相同fixture drift失败，并finally逐byte恢复候选。未改fixture/golden/断言/ID/skip；此原ID按明确基线失败继续入T-08/final accounting，不冒全绿。证据及hash见 [共享生成摘要](#e890-03d1ec7a02eaf7e5)。

888权威树读回eb80a4b89，仅有其诊断md未跟踪。未动共享脚本/注册/锁/合入窗，无agent/消息、整档、GUI、push。T-02剩余监听/legacy/producer/deadline与完整消费者/HTTP生成闭包/physical包API命令SD四仓schema仍待闭；Web未建，独立终审、冻结门、合入沉淀与清理均pending。UI/Man结构包继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 17 | from: work | to: work | status: executing | outcome: pass（共享生成实际单源改动与强正反证据，含具名父版本红；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 7c5a98804 | next: work（goal active；仍1/12，继续T-02）

### Goal round 18 — T-02 actual generated-service neutral closure（2026-10-30）

提交组合：lang `d31081334ddfce369a494849bccb3c324449c715` + UI `7b323bc02e4a8153a086ca3f88c5f1a95b47c446`。为关闭真实服务对UI的依赖，把已验证renderer-neutral照片实现及十个原测试整文件LF逐byte迁入既有auto-media-core/image-codec（源UI d53f946、blob ebeadbb360dee925c6fd137fc0a6c21fca0c707c，SHA256 738751c6bfa88c9ea5c313d620d008528f7e64a9e4ab28fa78ed0bdcef12558f）；UI唯一同寿命树转codex/plan-890-dev，旧feature/public路径只re-export，未开展UI结构后继。新增source/test补充映射保十原ID/条件/新owner，T-08仍需入完整执行账，不替换更小floor。SD-04补既有media Spec及模块index影响，新增component仅属proposal，不提前改canonical。

实际生产生成器photo模板改用media公开源，生成Cargo不再选择UI runtime；四处生成/成员/lock/新鲜度装配统一Service workspace。开发态只验证实际Core/a2r/media依赖，未选择的facade可有缺席可选UI路径；选中Core的坏可选路径仍绑定前拒绝，旧manifest保持。managed来源receipt校验与UI调用全闭包保持。原canonical-owner/media-route测试只更新真实owner期望并新增禁止UI断言，路由/GET HEAD/两状态义务未删。

已验：media十原例10pass/0ignored（private TEMP/TMP避免借共享缩略图cache）；codec-off check0；UI默认runtime locked check0；root/APIgen check0；原workspace4+owner1+stateful/full-cover1+image1定向全过；增强service来源/已选Core拒绝probe1/1。存量Core141/root3/APIgen3/UI215及最小生成物4warnings不冒全局清零。真实external production generator新项目→未手修Rust build+locked→metadata483packages，所选闭包0 UI/Man/Tauri/旧语言facade；精确generation ready fingerprint/随机业务值、3x2 PNG scan/full原字节/JPEG thumbnail真实尺寸、越界400/未知token404、自身shutdown202/draining true/child0/真实stdout STOPPED、源码变更stale1/恢复fresh0全过。源码/生成/构建工作目录均在890树，复用显式target；完整fresh SDK/分发/四仓档仍T-09后续。证据：[中立生成摘要](#e890-6bb22aeec7e5419d)。

验证流程修正：早期fixture错读stderr的STOPPED流；另一次把临时生成工程放主检出attachments违反worktree纪律，均未计入认可证据。最终在本树target/runs新工程完整重放通过，main仅保随后复制raw日志。此后补preflight同时拒绝树外source/build/evidence落点；树外target负控exit2、未启动生成，log SHA256 ef927a8989ef4cd29ec8cc44c48eab700801c94354a9981559f10e5afbaf08c8，候选合法路径未变，复用已过完整重放。脚本/fixture不注册全局档或锁。

所有本轮子进程均已结束，lang/UI树clean；Man只读树仍e6cc06f。共享治理脚本/测试注册/锁/888 helper/最终合入窗未写，未跑整档/docs_gen/GUI或push。T-02剩余直接语言入口与返回协调、legacy/state/producer/rate/deadline终态、完整VM及实际消费者/physical包API命令SD四仓schema冻结仍开放；Web未建，不以本次单源接缝/实际生成成功冒完整技术门。原Tauri byte fixture父版本红继续具名保留。UI/Man结构包继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 18 | from: work | to: work | status: executing | outcome: pass（实际生成无UI闭包及中立单源，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: d31081334 + UI 7b323bc | next: work（goal active；仍1/12，继续T-02）

### Goal round 19 — T-02 real language direct activation（2026-10-30）

提交 `4a266bb30b16e19508559982113d15916ce328ae`。实际语言入口已核实为 `use auto.http` + `http.server_listen(http.server(), address)`（wildcard自由函数同样可达真实native），不是只能手工SDK栈调用。方法式server.listen仍在现基面发生link未定义；type-qualified探测exit0但未到检查过的adapter不计服务正例；未为迁移改编译器/loader/注册或新增语言特性。此前“直接语言入口未核实”在此自由函数支持级关闭，不冒全部拼写/轨道可用。

修复直接监听完成后post-main注解再次启动：AutoVM持私有per-owner Cell，只有blocking host成功返回才置位；失败不置位、其他新VM不受影响，自动owned handoff对已完成owner直接返回，无需再次选host或绑定监听。仅新增2个crate-private访问器，不扩大公开VM/私有模块面、不复制VM或跨线程发送，lib自动入口和888共享面未改。fixture自己的stdin关闭controller现在保存并实际join；生产logger/deadline等完整线程终态仍未冒关闭。

真实语言程序language-installed：精确随机JSON/缺参400/非法整数400/未知404/stop正常返回、handler task无增量、原owner thread、controller join，1次真实external blocking activation且0次post-main async activation，child0。language-missing与language-unsupported均wrapper0/child1，真实native RuntimeError分别明确缺安装/不支持blocking且端口仍可绑定；不是跑到post-main才假装缺host。普通installed自动入口重放也全过，fresh自动消费者保持。原host选择单测1/1扩per-VM正反：完成owner后清默认仍可返回；fresh缺host拒绝；host拒绝不claim；fresh安装后仍触发auto，记录准确owner ID。locked外部构建与root check0；Core141/root3/external17存量warnings保留。未跑整档/docs_gen/GUI。详见 [direct语言摘要](#e890-5ee6016527cb6366)。

888读回eb80a4b89，其相对基面对engine.rs/Core lib无改动、仅自身诊断md未跟踪；本轮无共享注册/runner/锁/最终合入窗、无agent/消息。所有执行输入/构建在本树，raw日志仅本地；lang/UI仍同寿命树，UI7b323bc与Man e6cc06f未改。T-02仍需不同语义legacy stdnet、configured direct选项/active cfg、控制/producer/rate/deadline及完整owned terminal/consumer、physical包API/commands/SD/四仓schema冻结；Web未建，独立终审和merge-owned最终门/合入沉淀/清理均pending。UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 19 | from: work | to: work | status: executing | outcome: pass（真实语言直接入口与owner恰一次装配，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 4a266bb30 | next: work（goal active；仍1/12，继续T-02）

### Goal round 20 — T-02 explicit legacy stdnet / identity / terminal（2026-10-30）

提交 `79a6a411a211fb905a30bf58867ad1793cd8c3d6`。阻塞宿主端口显式区分ServerHandle/LegacyStdnet；旧公开serve_blocking_stdnet成为按kind选host的薄桥，保unit-return+指名诊断，缺安装/仅async provider均绑定前拒绝不回退。旧loop单源移到server_legacy，由真实外部source crate执行；仍未宣称Core实现退出，T-05必须移走。修掉SSE/file早退漏task、持DashMap guard移除/异步owner blocking_lock风险；request/iterator/next-task guards在错误/断连完成，资源通过请求local Send context登记/终态与late注册同锁；新具名primitive cleanup dispatcher与既有RequestScope late分支共享，不扩大VM/private模块pub面。

按现行http-return-resources §2修旧legacy裸int查表误判：只由真实Sse凭据赋予流身份。新probe经公开VM SDK创建真实lazy generator，实际key0保留为caller-owned；HTTP普通int0必须是精确JSON字节0且不得拉取/回收该generator。两个真正SSE形式（原~Iter声明、int声明但返回真实generator）各精确1/2/3帧保持；handler/request返回值的实际rc_count/凭据类型检查在VM drop前证明只有caller seed仍持有，seed显式释放后持有0，避免仅看进程退出掩泄漏。

真实SDK wrapper→显式external LegacyStdnet：随机精确JSON、缺参/非法参数400、404、CORS*、只发头承诺1MB body的OPTIONS204、两种SSE、连续3次file显式500、观察实际无限generator帧后RST并恢复精确业务响应、stop/controller join全过。VM drop前handler task delta0/service iterator delta0/descriptors0/request credential owners0；owner返回后actual credentials/uploads0，caller iterator保未启动且独立释放。missing/unsupported反控保旧child unit-return0但必须明确诊断且无监听，非服务健康绿。fixture grade-swap替换错误runtime即使JSON精确也被原CORS oracle拒绝（期望exit1），不改生产逻辑；server-handle语言重放仍过。

本轮主动命中并修复自己Round16引入的源布局回归：原Plan705默认调用图围栏红（非默认server-handle loop误放进默认HTTP扫描目录）。不改测试/断言/required count，把该loop逐byte移回非默认职责文件ffi/http_server_handle，APIalias不变，SHA256 3184a2af5cccc2ef4ffbd359dd7adfd7d1dce4f5189ca9a084e2008081f472c8；此前原本在stdlib/http_client，同样不属默认图。Core只有定义、唯一调用在explicit外部probe；默认async装配未转调用该loop。相同原gate在最终source上1pass：默认未标记busy0、原marked legacy1。这是本Plan修复，未转历史债或缩floor，T-08仍应完整迁原义务。

Root check0，lockedSDK build0；原Core late upload context/host selection/media wire三项各1pass/0ignored；root原callgraph门1pass，fmt/diff/Python编译过。现存Core141/root3/external17/root lib-test42 warnings不冒全局健康清零。无整档/docs_gen/GUI/push，未动共享治理script/测试注册/锁/888 helper/最终合入窗；源码与构建均在本树，只md摘要入库。证据：[legacy端口摘要](#e890-a6385bebc8c0e3d0)。所有进程结束，lang树clean，UI7b323bc/Man e6cc06f不变。

T-02仍开放configured launch/程序级legacy policy与producer所属、default runtime完整deadline/control终态、实际consumer及physical包/API/commands/SD/四仓schema最终冻结；Web未建，独立终审/最终门/合入沉淀/清理pending。不得因已打通legacy和生成闭包而跳完整门；UI/Man结构后继继续等U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 20 | from: work | to: work | status: executing | outcome: pass（legacy真实端口、来源身份与终态/修复本Plan围栏回归；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 79a6a411a | next: work（goal active；仍1/12，继续T-02）

### Goal round 21 — T-02 native policy / owner rate windows（2026-10-30）

提交 `e32f4d678`。真实反例确认语言http.rate_limit此前写Core framework全局，而external source runtime读取另一份：四个精确nonce成功后第五次仍200、未按要求429（exit1）。先修fixture header大小写读取错误，再取得此认可red；未以错误oracle充反例。native改存VM私有纯设置，未设置/显式关闭/正配额区分；具名VmHttpBridge只在原VM线程读取，未公开私有字段或跨线程传VM。旧SDK set/clear公开面保一个中立process default数据源，语言显式设置包括0只覆盖自身VM；固定窗口算法和每个serving lifetime bucket仍属framework，每poll选择/恢复实例状态，standalone dispatch旧共享bucket不被实际owned服务消费，后续T-05仍须完整单源迁出。

最终真实TCP：Core旧SDK default1、A native4、B native0后handler动态2；A耗尽后B仍可多次精确响应，B开启后仅两次200再429，A仍429。校验429 JSON/retry_after_ms/Retry-After/request-id、旧环境CORS精确值、两owner drain/registry0；config-only普通程序无adapter仍返回42。SDK default/clear及VM显式disable/window最小值/fresh VM单测1pass/0ignored；locked外部双bin build0、root check0；受poll wrapper影响的configured service state原探针重放0，identity/counters/CORS/shutdown隔离保持。Core normal warnings140（原141的unused native vm现已使用）、root3/external17、Core lib-test160，未冒全局清零。最终唯一native-call折行为格式调整，未改变已编译行为；helper fmt/Python编译/diff过。详见 [native policy摘要](#e890-87d5a3dc4d1a4214)，六份raw日志hash已记。

888读回70a80fead：本轮相关重叠仅其原spawn_outcome_reporter cfg变动，不改该helper/共享注册/runner/锁/最终合入窗口，不发消息或启实施agent。主检出只有既有docs簿记/他方设计WIP，未基于它做实现；lang/UI/Man复用同寿命树，UI7b323bc/Man e6cc06f保持clean未改。所有子进程已结束，源码/生成/构建在890树，raw本地保留，只有md摘要入库。无整档/docs_gen/GUI/push；原HTTP e2e ID未冒执行，T-08完整迁移floor保持。

T-02仍开放producer编译/activation前元数据归属、configured launch、完整deadline/control/线程终态、actual consumer与physical包/API/commands/SD/四仓schema冻结；Web未建，独立终审/最终门/合入沉淀/guard pending。UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 21 | from: work | to: work | status: executing | outcome: pass（真实native限流外部传递/实例隔离与SDK兼容；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: e32f4d678 | next: work（goal active；仍1/12，继续T-02）

### Goal round 22 — T-02 network thread join / drain terminal（2026-10-30）

提交 `9fc909552`。上一轮为实际实现进展。本轮owner保留并异步join自己的真实网络线程，失败启动分支同样回收；只有Send线程句柄进入blocking executor，VM仍原线程。接收队列先drop，网络线程完成后才报Stopped。owner drain deadline只设置一次并在继续parked工作前检查，不随请求/完成事件刷新。两侧等待真实watch true（含激活前已true），忽略false更新，sender关闭仍退出。连接清理改为spawn前捕获RAII，未poll或被强制取消也abort自己的reset-shadow、取消自己scope并归还该实例active计数。

两个来源约束反例：只恢复父e32f4d678的owner源码，实际完整transport返回后同一真实网络线程fixture尾部延迟800ms，旧owner提前返回，terminal实测network_tails1、probe exit1；只恢复父transport而保新join，partial-head idle跨drain后实际connections_active仍1，adapter断言触发child panic/外部probe exit1，后续connection-refused是次生输出。两个临时恢复均在同树finally逐byte还原候选，随后最终build与正例；没有替代网络实现或生产延迟旋钮。early system-temp源码运行未计认可证据，已把两个受影响probe源/资源scratch固定本树target，并在正确位置重放两red和四positive。

最终locked external双bin build0、root check0；watch初始true/false更新/sender close单测1pass/0ignored；tail installed精确随机业务+实际线程完成先于owner终态、pre-stopped正常返回+端口重绑、configured双owner带partial-head连接跨drain强制清理、16MiB精确file跨关闭完整送达+ready非200+B独立+两端口重绑四probe全exit0。每实例return后actual active计数0，全owner完成后registry0。deadline固定由代码实证，不冒称跨全部CPU/native负载有限关闭。Core140/external17/root3现有warnings保留。helper fmt/Python编译/diff过。详见 [network终态摘要](#e890-5621da983b0d3f3f)，九份raw日志hash、父源码还原hash及边界已记。

新file日志显示完整wire送达后request事件bytes_sent仍0，此项列为T-02待查response terminal observation，未冒字节观测通过或擅自延后。其余producer activation前归属/并发编译、configured launch/实例failure传播/logger退出、实际consumer/physical包API命令SD四仓schema冻结仍开放。Web未建，仍executing1/12；独立终审/最终门/合入沉淀/guard pending，UI/Man结构后继继续等完整U890。

888读回643f2b375，transport/dispatch相对共用基面无diff；未改helper/共享注册/runner/锁/合入窗、不发消息或启实施agent。主检出既有docs WIP保留，lang/UI/Man唯一同寿命树，UI7b323bc/Man e6cc06f仍clean未改。本轮进程全部终结，只有md摘要入库；无整档/docs_gen/GUI/push。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 22 | from: work | to: work | status: executing | outcome: pass（真实网络join/连接取消终态正反证据；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 9fc909552 | next: work（goal active；仍1/12，继续T-02）

### Goal round 23 — T-02 actual file/SSE terminal observation（2026-10-30）

提交 `2dbde2ab5`。上一轮为实际进展。本轮关闭已观察的file bytes_sent0问题：原producer finish hook在HTTP缓冲体未读时就complete scope，file/SSE未按payload poll累计字节，实际file206/304/416/404亦未替换占位200。真实7项wire/event multiset反例exit1；fixture初期误以为自动ETag，改为明确descriptor ETag后才取得认可red。新framework body guard只计交给HTTP的payload字节（不冒TCP ACK），保原scope单点恰一次事件，成功/早Drop/错误分开，支持精确长度最后帧后直接Drop而不额外poll EOF。producer结束只记结果，不提前归还未读request；实际file状态/inline错误长度在终态前写回。未扩大VM/private端口字段或复制语言对象。

另捕获并修复两个实际问题：configured file在net线程泄漏旧默认CORS，反控exit1后用每poll服务policy上下文覆盖net request/file组装/拒绝路径；中间修复造成still-connected stalled file在watchdog后不终结，此为本轮未提交候选的回归，不甩父基面。具名per-connection watch只中止该失败/超时active file连接，让真实body Drop完成同一scope清理，准备失败仍走HTTP inline错误，stream激活竞态有双侧检查；未建全局abort map/生产延迟旋钮。

最终真实external probe exit0：完整binary GET、range206、HEAD200、conditional304、range416、missing404、finite SSE七项method/route/status/bytes/outcome精确一一匹配；file/SSE RST各一次canceled且正字节、未读file不得计满长，源文件实际截断一次canceled；200ms file idle budget下客户端保持连接不再读，3s内实际body终结事件已到且仅一次，再主动close；B counters不随A流量变化；owners/registry0。real file pump单测1pass证明producer完成而body未读时request permit仍0可用，最后帧后不poll EOF Drop一次归还、exact bytes/completed1/canceled0；原full_queue503断言不改、内部新参数None，1pass。locked external build/root check0；受影响16MiB across-drain+配置隔离/两端口重绑与legacy native/CORS/ordinary-no-host两probe重放均0。Core140/external17/root3 baseline warnings保留，fmt/Python编译/diff过，最终仅注释更正不改已验行为。详见 [响应终态摘要](#e890-5ba82b8d9f848b23)，十份raw日志hash及边界已记。

888仍643f2b375，transport相对共用基面无diff；无共享注册/runner/锁/合入窗/agent/消息，主检出既有docs WIP保留；UI7b323bc/Man e6cc06f clean未改。全部进程终结，源码/资产/构建在890树，仅md摘要入库；无整档/docs_gen/GUI/push。

T-02继续核compiler producer activation前归属/并发编译/async-return元数据、configured launch交接/实例failure/logger退出、ordinary parked/async实际观测、完整consumer/physical包API命令SD四仓schema冻结。未把file/SSE局部通过扩大为all-response或全技术门；Web未建、原ID/floor仍待T-08逐项迁移核账。仍executing1/12，独立终审/冻结最终门/合入沉淀/guard pending；UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 23 | from: work | to: work | status: executing | outcome: pass（file/SSE真实终态/字节/状态/许可与CORS、watchdog闭合；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2dbde2ab5 | next: work（goal active；仍1/12，继续T-02）

### Goal round 24 — T-02 compiler/main/activation metadata ownership（2026-10-30）

提交 `e2ed6ba761b1082af34e759012f53d292cd46346`。上一轮为实际进展。真实反例：A编译后main真正park，B在同一owner线程编译并激活，再激活A，其自身route404；原来在main后才读process tables/routes。初期fixture错误prefix await改语言实际`.await`、非JSON错误解析改妥后才取得认可404 red。新增language-owned线程local CompilationMetadataGuard，仅收owned声明数据；原publisher保legacy mirror并经具名bridge写当前collector。lib execute/test、entry source VM、script debug四条正常构建路径在module-init/main可await前freeze并绑定VM私有immutable Arc，服务/borrowed启动及自动路由使用该VM声明；DTO补async_returns，实际external reader按当前owner判断，不再读另一份source crate私有async表。未改变opcode/字节码生成/native注册，未公开VM字段、AST/Store或跨线程传VM。

新真实gap probe最终exit0：invocation-specific精确A/B值、same-name int/str参数/返回、A async25/B普通240、非法整数400、external实际async metadata reader A真/B假，owners/registries0；ordinary-first正例在B激活后清default host，A仍返回42、无listener/provider调用、B仍可用。fixture按provider身份选地址而非激活顺序，stdin shutdown后controller实际join。两新Core定向测试2pass/0ignored：nested真实编译+错误/unwind恢复outer params/return/shadow/async；两真实compiler并发四轮八VM，双方结束才验各自snapshot，cross-thread等待有时限，只Arc metadata过线程。compile-fail反控expected101，CompilationMetadataGuard因Rc<()>不能Send，原VM/context反控保持。旧metadata roundtrip/Pending-nested-unwind/host-selection三原ID各1pass/0ignored；direct真实语言listener与file/SSE终态全路径受影响probe重放exit0。locked三bin external build/root check0；Core140/external16（实际async reader现被调用，较原17少1）/root3/lib-test160 baseline warnings保留。详见 [compiler metadata摘要](#e890-8e23a6b7df3766e4)，12份raw日志hash及命令/边界齐。

初次external build拒绝private collector访问，已改具名纯数据bridge操作解决，未扩大私有module可见性；失败/旧binary重放不计正例。custom SDK compiler有明确begin/finish/bind-before-share路径；旧unbound raw VM缓存fallback保留，不把它冒作multi-program并发保证，最终owner/facade随T-05退出scaffold整理。helper fmt/Python编译/diff通过；源码/资产/构建在本树，仅md摘要入库。所有进程结束、lang clean；UI7b323bc/Man e6cc06f仍clean未改。888643f2b375仅不同entry函数改动，本轮source VM构建之外未改共享test注册/runner/script/锁/合入窗，无agent/消息/整档/docs_gen/GUI/push。

T-02剩configured launch交接、实例failure/logger终态、ordinary parked/async响应观测、完整实际consumer/physical包API命令SD四仓schema最终冻结。Web未建，原test-ID迁移/floor核账仍T-08；不以新probe替代原required执行。仍executing1/12，独立终审/冻结最终门/合入沉淀/guard pending；UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 24 | from: work | to: work | status: executing | outcome: pass（compiler-owned metadata与实际main gap/并发/ordinary隔离；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: e2ed6ba76 | next: work（goal active；仍1/12，继续T-02）

### Goal round 25 — 用户限定T-02的阶段收口与暂停（2026-10-30）

最新用户指令收窄本次执行到T-02后停止，不自动T-03；要求代码提交、保树保分支、主计划/风险/契约/交接齐，不合入master、不归档、不清理。该边界覆盖先前自动全链执行授权，但整体目标/12项任务/AC不缩减。本轮不再延伸生产补丁，先列R1..R6关闭条件，再作有界实际审查。

**技术门结论：needs_replan，T-02未完整通过，仍executing/current_step1/12；T-03禁止准入。** 三项实际required反例全部未关闭：①配置端口5077却精确服务于env5078，active config未消费；②真实占口已有bind诊断，自动服务SDK仍child0；③真实parked async wire200/25，但唯一completed事件status=null/bytes_sent0。另源码显示logger仅存tx、丢弃spawn句柄，没有已验证join。R1当前owner端口缺launch/config，R2结果/ready仍依全局诊断，R4日志session所有权需设计；R3可在现有模型内修但仍未实施，不冒“待决策”或后移T-05通过。须先作有界T-02方案修订，不能用持续修补代替阶段结论。

生产代码最后变更 `e2ed6ba761b1082af34e759012f53d292cd46346`；本轮仅探针/文档变更。**代码/候选契约冻结 `33a467a98fd2929640050550b3faf00d5a7824ed`；最终交接封存提交 `76f2069d5d17bb5a591097712c2673458829016f`（仅新增交接Markdown）。** build probe locked exit0，T02 closeout audit exit1/三项fail；后者是未通过证据，不因定位问题改记green。Python编译/helper fmt/diff checks通过；本轮无crates生产改动，不跑根cargo测试档/docs_gen/GUI/无关回归。build日志SHA256 d93bffbf512205b34bdc3708d0c51e2c895e025ea48d6be7c43b252cf775c942；audit日志SHA256 be1c501931841e073411e841d1640e86a45bf4fa915764b232681dc3f0dab625，本地路径均 `D:/autostack/auto-lang/docs/plans/attachments/890/work/`。此前各接缝正例只按原范围/pin复用，不扩成完整部署/发行/全ID绿；父Tauri金样red和存量warnings仍保留。

已形成 [T-02端口/包图候选与阶段结论](#e890-87d69d695303f960) 及 [可直接使用的交接](#e890-41796130e16d9105)：公开签名/时序/消息与线程边界、受限25函数载体、真实SDK/Man/UI/生成服务consumer映射、source→owner包图候选、迁移顺序、当前命令/失败观察、887四仓schema约束与待决D-T02-01..03。包/API/schema未最终批准，不能把文档形成冒作T-02完整freeze。共享runner与fixture脚本实物与正式887 blob一致，未写888 WIP schema/共享注册/锁/合入窗口。

语言权威树 `D:/autostack/.wt/lang-890/auto-lang` / `plan-890-dev` 保留；UI同组 `codex/plan-890-dev` /7b323bc、Man detached e6cc06f、Down detached c3464d0同寿命保留，均无本轮产品改动。所有子进程结束，语言最终提交后及UI/Man/Down树clean（ignored target/本地证据保留）；主检出只保本计划簿记和既有他方docs WIP，未提交/混入他方内容。Web主仓及组树均不存在，未创建；未merge/push/归档/guard-remove，未发消息或启实施agent，UI/Man结构后继仍等完整U890。

下一入口为**同Plan、同树的T-02有界方案修订/修复**：先确定per-invocation launch与session结果/日志终态，修R1..R4，再确认R5/R6并真实记录通过。即使之后T-02通过，也必须用户另行允许才进入T-03。本次按用户指令阶段暂停，不标整体goal complete。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 25 | from: work | to: work | status: executing | outcome: needs_replan（T-02技术门未过；用户要求阶段暂停） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 76f2069d5（code freeze33a467a98） | next: user（阶段暂停；不自动new/work/T-03）

### T-02 r3 有界修订、修复及阶段暂停（2026-10-30）

stage: new / plan_revision: 3 / outcome: pass / changed: T-02、D-T02-01..03、SD-03/05 / authorization: 当前用户交接承接T-02修订与修复，目标/AC/仓集合不变；不恢复T-03全链。现行§5与r3契约取代r2待决机制，r2失败收据原样保留。

stage: work / scope: T-02 / outcome: pass / implementation: `88e604ba0`（launch/Result/观测/cleanup）、`28d5c41c1`（真实main-gap与交换配置反控）、`f56211c12`（独立网络外部关闭修复）。R1实际configured/explicit/hash/port0/交错实例通过；R2占口child1且同进程失败不污染B；R3 wire200/25对应event200/2/completed、parked error500/87及cancel恰一次；R4服务正常/失败/取消后真实network/log join及资源0，cleanup回执与服务成功分离。补充T02-R3-SHUTDOWN1先红101再绿0：owner不再poll时网络仍独立释放端口；其余原audit3/3、session5/5、file/SSE七项精确事件+RST/截断/watchdog通过。无整档/GUI/docs_gen；rootcheck0、5个命名单测各1/1、fmt/diffcheck0；Core139/root3/external16/lib-test159存量warning未冒清零。原Tauri父版本红保留。

stage: review / review_scope: phase(T-02) / reviewer_limitations: 实施会话重读冻结source/diff及输入/断言重建；没有独立agent或final上下文，不抵AC-13 / plan_revision: 3 / reviewed_commit: `f56211c12f6cc8062f0cbfb5a4f0d03c8e4a958c` / base: `76f2069d5` / outcome: pass（仅T-02技术可行性/执行合同） / findings: T02-R3-COV1真实交错配置oracle缺口已补且swap反控拒绝；T02-R3-SHUTDOWN1独立关闭回归已修且反例先红后绿；无剩余T-02必需缺口，无未批准延后/workaround或新增非合同债候选 / spec_delta: r3 launch/session规则与现行SDK/部署Spec一致，目标清单不变；规范未发布 / pending_closeout: 后续物理迁仓/原ID/消费者/发行/应用、T-11独立终审和所有merge-owned门。

阶段交接提交 `4ec696a631058a92c4f282fdab054a86dbadc2ff`；主计划只作默认检出簿记，不混入其其他docs WIP。语言权威树`D:/autostack/.wt/lang-890/auto-lang`分支`plan-890-dev` clean；UI`7b323bc`、Man`e6cc06f`、Down`c3464d0`原树未改；Web未建。原887共享脚本blob保持`e8458b27e32521dff62ee611eb1814193c86b445` / `cbff4c581eadbf763eeda278c40c1167b7b9053d`；未写共享runner/注册/锁/888 WIP、未发消息、未合入/push/归档/清理。完整命令/结果/本地证据SHA256、Spec输入hash与适用边界见 [r3交接](#e890-8caf4976a4ad8156)。

当前状态 **executing / 2 of 12**；next: user（按原边界阶段暂停；明确允许后才T-03）。所有树/分支/本地证据保留。

### r4 用户指定阶段串行及888正式基线门（2026-10-30）

stage: new / plan_revision: 4 / outcome: pass / changed: §5执行顺序、T-03前置同步、各启动单元暂停边界 / authority: 用户本次明确指令 / unchanged: 目标、完整AC、12任务、T-02技术冻结`f56211c12`及其r3证据 / documentation_commit: `ed44fd0bd6ea063e8e4977454bd5ef04455aa762` / worktree: 原`plan-890-dev`，文档提交后clean / evidence: [阶段规则及交接](#e890-69842b508b36ad9b) / verification: 只核文档、状态/任务数、链接及限定diff；无生产改动，未构建/运行测试 / next: user（保持executing2/12、阶段暂停；未核实或宣称888已经合入，未联系或操作888会话，未合并master，未启动T-03）。

### r5 T-03～T-04阶段收口（2026-10-30）

stage: work / scope: T-03,T-04 / outcome: foundation-and-targeted-acceptance-complete / status: executing4/12 / full_tm_gate: NOT_PASSED / review: same-session phase audit only, independent final AC-13 pending / next: user（阶段暂停，不进入T-05）。

888正式交付`5a6c23afa`、归档`107148fcd`、收尾`129ef6057`均为master祖先，归档合同10/10及复审/门禁/guard收据已核。原clean890分支`ed44fd0bd`合入正式master`129ef605763e59ca13bf312fd4a391eded10c13a`后为`29ed9d7bde334ae14e21223c935a4f3974581f71`；无需手工冲突处理，888 gated tracing/reporter与890 launch/session实现均保留。未联系或操作888会话，未使用WIP；共享scripts、.cargo/config.toml及AGENTS与该正式master保持一致。

实现冻结：语言基础`84779b2b1`、最终测试装配`1a253950810de67f16f487c7eb7a2745f1061bb0`、交接`e45399a633a1dcf74f70943cf2d2c663791589a6`；新Web本地bootstrap`8ff4966`，唯一阶段树基础`26259642f56656547db7bc07a68c35a2bd225edd`。纯配置单源移至auto-web-contract，Core同类型re-export；实际Cargo选中来源进入SDK身份清单，重复物理合同负控被links命名空间拒绝。四受限生成模块迁auto-generation-core，旧路径薄委托；原15测试体/名保留，正式ID登记仍T-08。Web runtime/生成器算法未提前迁移。

定向证据：Web workspace check、语言locked check、SDK边界check、482包真实依赖闭包均通过；无UI/Man/Tauri/混合facade依赖及Core→Web实现反向边。真实consumer验证同类型/Arc、严格配置、普通SDK42及旧新生成路径；原载体15单测、合同消费者及真实companion正反例通过。T-02新包源重建后audit3/3与session5/5通过，缺adapter child1/无监听、普通42 child0/资源0保持；未变transport/body/compiler部分仅按交接声明的代码/输入边界复用，不能扩为完整迁仓证明。源码机械守恒与fmt/diffcheck通过，存量warning未宣称清零。

完整tm结果不得误读为pass：7183义务/1186叶目录中4直接叶扩750模块/5483义务；58已执行模块55通过、3失败，另23原条件NOT_APPLICABLE。三失败与888归档P763-R1相符（aavm compile corpus、compile use corpus临时Rust缺auto_lang；a2r golden INVALID_BYTECODE:60），相关产品源本轮未变，保留原红而非删除。旧plan734 HTTP测试缺显式SDK host且等待无界，定向装配修复后原4测试完整重放通过，4个直接Core叶原23项重放通过；两重放均为诊断/定向验收，**不是修复后750模块新源身份完整认证**。未重跑大闭包冒通过；未完成项及已有红须在T-12最终冻结组合依新tm规则处置/执行，不能按本次55pass豁免。工具当前未覆盖新Web源身份，T-08须扩owner/来源/consumer与实际收据，现以锁/源码hash补充记录。测试生成6个既有报告原件先本地存hash后按冻结Git恢复，未提交报告；完整本地日志/hash见交接。

阶段审查仅确认本单元AC-01..04涉及的基础/类型/SDK/共享承载部分。未做独立终审，不设置reviewed，不宣称AC-03运行时、AC-04完整生成或整体交付。拟Spec delta：纯合同新owner、四模块载体、Core实际源身份及显式SDK测试装配；canonical Specs/ledger留最终沉淀。T-05须用真实Web installer替换临时测试adapter并退出Core runtime桥；T-06完整迁生成器；T-07消费、T-08逐ID/四仓tm、T-09/10发行/应用、T-11独立审查、T-12最终门/合入/沉淀/安全收尾均保留。

[阶段基线与有界合同修订](#e890-adbb1a260bdcd577) / [冻结交接、命令、退出码与本地SHA256](#e890-18ff84e0bd8a968f)。所有树/分支保留；未合入890至master、未远端发布、未归档或清理。 收口只读观察到master已前进至`60bda3276a0781a5da60452eb317fa399428980a`；本次同步/证据基面仍固定于正式888收尾`129ef6057`，未在阶段末静默二次合并或扩大验证范围。

### T-07 全链恢复与消费迁移（最新授权）

stage: work | plan_id: PLAN-890 | plan_revision: r6 | scope: T-07 | outcome: stage-owned-implementation-and-targeted-acceptance-complete | status: executing7/12 | worktrees: original lang-890 five-repository group | code_commits: language 163d342a6, UI 672955d, Man 9f8ffc2 | evidence: [T-07实际命令/数字/原失败与限制](#e890-1f9b138c564b0993) | final_review: pending T-11 | frozen_union: NOT_PASSED, owner T-12 | next: work T-08（本次完整goal授权覆盖旧逐单元暂停，不等待再次启动）。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 26 | renewed_session_round: 1 | authorization: latest /goal 完成计划890 renews the full chain; prior r2 round25 pause is historical | from: work | to: work | status: executing | outcome: pass（T-07阶段；非work-complete/非完整计划） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 163d342a6 | next: work（T-08执行中，T-09/10/11/12仍保留）

2026-10-31 / stage: work / plan_id: PLAN-890 / plan_revision: 6 / outcome: continuing（T-08 complete; executing8/12） / worktree_path: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commit: 38f68d8e9 / dependency_revisions: Web f4cc93c, UI 672955d, Man 9f8ffc2, Down c3464d0 / task_ids: T-08 / evidence: attachments/890/work/t08-handoff.md; 46 recipes/7320 obligations/1211 leaves, Web217 pass+2原ignore，语言保留28 pass、media54 pass、governance9 pass；四仓真实列举守恒与floor均过，partial/list-only负控exit2，不冒最终执行。原Plan736缺显式adapter实际TIMEOUT120.046s修复后双profile约2.7s及六兄弟消费均过；原失败不覆写。全局证据预算预览exit2/64.88GB未达标，无删除；Plan本身低于500MiB / blockers: none for next implementation; final gate NOT_PASSED / next_responsible_role: work T-09 → T-10 → independent T-11 → frozen gates/merge T-12, authorization inherited.

2026-10-31 / stage: work / plan_id: PLAN-890 / plan_revision: 6 / outcome: continuing（T-09 complete; executing9/12） / worktree_path: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commit: f739c3751 / dependency_revisions: Web 9ddb7fe, Man cb4ab3d, UI 672955d, Down c3464d0 / task_ids: T-09 / evidence: attachments/890/work/t09-handoff.md; 冻结发行45过、旧新Core历史还原/失败后真实消费14过、SDK库217过+2原ignore；Source/WIN zip 3.59/19.40MB；严格真实收据/源码/输出hash验证通过，missing/duplicate/zero/not-run/错hash等治理10通过，用途守恒19通过。真实Git检出库214过/3长路径环境失败/2原ignore，短路径5项重放均过；1次LEAK原始结果保留，命名隔离18.466s通过且三端口已释放。D盘曾满而失败的诊断收据保留，自动审查两次拒绝缓存删除，未删除；转C盘继续，后续D盘空间恢复非本会话删除所致。语言运行SDK源87a09af94与当前f739 SDK代码/锁/stdlib输入相同，治理/验证脚本差异另记。 / blockers: none for T-10; final four-owner actual gate NOT_PASSED / next_responsible_role: work T-10 fixed supported Blog source + affected Vue/native, health and SD freeze; then independent T-11 and T-12 gates/landing under inherited grant.

### 2026-10-31 T-10 final work handoff — r6 / execution_done10/12

stage: work | plan_id: PLAN-890 | plan_revision: r6 | scope: final implementation T-01..T-10 | outcome: pass | status: execution_done | current_step: 10/12 | worktree: D:/autostack/.wt/lang-890/{auto-lang,auto-web,auto-ui,auto-man}, Down same fixed sibling | implementation_commits: language `ede598998566ea890ff67170bb9744f3bc537a58`, Web `9ddb7fe5bb1886166073b5805c38895f3da195ff`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466` | evidence: [T-10 self-audit, exact command/profile/result/hash/limitations](#e890-6aafd921a0a45c59), [SD proposal](#e890-eba7e5bd9981c181) | actual_results: fixed source314 unchanged; default-profile generated Rust cold build2m31; complete Vue15/15 and native22/22; UI module866 pass/2 original ignore/4 new controls; source-bound discovery46 recipes/7329 obligations/1213 leaves; purpose/conservation19 pass | source_identity: `0bb28657a28e2405d21bf42da70bd3b47ab99c45d773aae8617ad0d5ce981bb1` | health: all four candidate whitespace pass; owned fmt pass; changed-source warning baseline4/candidate4/new0, not whole-tree warning clean | limitations: original failures retained; runtime profiles after explicit cold compilation; SDK history Core valid shape only; controller preview tree required explicit owned cleanup (3 exact children, 3 ports v4/v6 rebound), native own-child assertions independently pass | independent_review: NOT_RUN, T-11 on the same trees and independent context | final_frozen_tm_and_four_owner_gate: NOT_PASSED, T-12; old AAVM reds remain named, no list-only/discovery/purpose classification substituted for execution | merge_specs_archive_guard: NOT_RUN | next: review.

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 27 | renewed_session_round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: ede598998566ea890ff67170bb9744f3bc537a58 (implementation/governance; docs-only handoff follows) | next: review | authorization: current full-chain /goal remains active; historical r2/r4 stage pauses do not apply; no implementation agent or cross-chat messages.

### 2026-10-31 T-11 independent final review — needs_fix（第1轮）

stage: review | review_scope: final | reviewer_identity: `/root/plan890_final_review`（独立上下文、原权威五仓同树，无第二检出/实施代理） | plan_id: PLAN-890 | plan_revision: r6 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | reviewed_commit: `e1ece5d45138040bcb67c4ecf5060cca53099019`（实现`ede598998566ea890ff67170bb9744f3bc537a58`） | base_commit: `6e48fe11c0be28159a950413a14f7964b3659389` | dependency_revisions: Web `9ddb7fe5bb1886166073b5805c38895f3da195ff`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | spec_delta_sha256: `b2a47e3a5891e79b25bcf842462bdbc33aa222117c373355c9b192aed47e7e9e` | evidence: [完整14AC/18V/9SD映射、风险核查及最小关闭条件](#e890-f0fee9ee4a1d7f26), SHA256 `6ec2e2666d46504c9963cf09bd8625f1e4b162315f4ae8e472c62c889f39377b`; 本地`t11-final-baseline.json`含逐Spec/锁/来源/原始反例hash | pending_closeout_gates: 全部T-12仍未执行 | next: work（同树修复R1/R2，不合入）。

- **890-R1 / HIGH / AC-08/09/12**：真实公开fixture接受零必需执行的四仓收据，exit0/pass/11过0失败0pending；实际目录无条件20用例缩19仍被collector接受；真实发行45收据删去5类必需command/exit/log/hash后逐项仍被接受。缺口为必需执行证明可选/自报适用集合未守恒，不是新签名或安全产品要求。原真实45收据及完整20-ID控制仍为合法对照。关闭须真实消费者拒绝遗漏、重复、无依据排除及不匹配输出，原条件/合法runtime与reuse形态保持；按复审包做有界治理检查，不追加全目录门。
- **890-R2 / HIGH / AC-03/06/09/12**：真实已安装Web CLI，五相关源文件与候选逐字相同；`run_file`所走`script_entry::run_with_path`新线程未传TLS launch。无配置env5453对照正确；有效development配置5457却在env5458精确响应；有效proxy配置5462却在env5463/0.0.0.0响应，非法Host应400实际200。三次均shutdown202/exit0/端口释放。原T09/HTTP测试把env端口设成config端口，遮住配置丢失。关闭须不同端口、loopback/有效profile/配置身份及Host真实拒绝，保留无配置和普通无服务文件行为；修复文件入口共享线程交接并重建受影响SDK/发行身份。
- 独立新增检查：原ownership10/10、Man transaction6/6通过；R1三类反控与R2真实wire分列为失败。Parent核16份T10原日志/产物大小与hash全相符。初始无效proxy夹具、任意启动等待超时和UI独立探针setup/已知边界不冒新产品缺陷；原红/ignore/限制不覆盖为绿。
- AC-01/02/04/05/07/10/11保留有界pass；AC-03/06/08/09/12 fail，AC-13未获pass，AC-14 merge-owned partial。SD-02/03/05/07/08待缺口关闭，不通过软化规范代修复。重开T-05公开配置入口、T-08、T-09及T-10冻结，完成数10→6；T-02/04已验证端口/类型/DAG保留，修复时补文件入口交接反例。所有必需缺口按在途阻断追踪，不转非阻塞债。
- 五候选HEAD和clean状态最终读回均未变；review环境hash `aa2ef3ba...`与discovery `a92a86a7...`不同，逐仓源码输入相同，不冒combined identity相同。主分支同期正式交付894（最终观察`f0898d6af`）；后续合入前须同树协调正式改动、刷新受影响目录/来源/消费者身份，原复审不覆盖未来组合。没有全局测试/GUI重放、代码修复、canonical/账本写入或合入/归档/guard移除。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 28 | renewed_session_round: 3 | from: review | to: work | status: executing | outcome: needs_fix | current_step: 6/12 | final_review_attempt: 1 | repair_cycles_started: 1/3 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: e1ece5d45138040bcb67c4ecf5060cca53099019（review baseline；后续仅复审簿记不覆盖代码） | next: work | authorization: 当前全链授权继承，原树有界修复后受影响复审；不请求再次许可，不执行T-12门或合入。

复审阻断按AGENTS登记在同树`KNOWN-DEBT-AND-RISKS.md`，明确OPEN/必修而非延后债；仅该文档提交`46547a0f898cfe29745e7bfcee62dc874f9cfb64`，候选实现不变、提交后clean。主检出只提交本计划与两份MD复审/冻结摘要；所有raw探针/JSON/log本地保留，未暂存。他方894及其他文档WIP未混入；本轮仅文档目标校验，无新增Rust测试或全局门。

### 2026-10-31 work repair cycle 1 — execution_done10/12

stage: work | plan_id: PLAN-890 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | base_commit: `46547a0f898cfe29745e7bfcee62dc874f9cfb64`（复审后只读簿记），正式894/主计划同步`c73e88f86` | code_commit: `2506549321ad8f4699089a7b0dce53d74f78b046`（实现74e13580a、证明分支0fdca25c0、原格式恢复b446256b5、最终目录250654932） | dependency_revisions: Web `e01232b3d449ea590a85d9458d553ab17b9456a3`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | task_ids: T-05/T-08/T-09/T-10 | status: execution_done | current_step: 10/12 | findings: R1/R2 implementation repaired, independent affected re-review pending | evidence: [修复交接/关闭表/复用边界/命令及本地索引](#e890-0d9015432e61f0d0), SHA256 `d72f61ea3d6409605dfc56ec6a92d496276c4c79fbe56c69d5e278531a6eb628`; [当前SD增量](#e890-b6c1120878575113), SHA256 `3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172` | source_identity: `815c5287416833e52b7656e0c07764600a53ca532b883bd13168b728abf1ce8f` | blockers/user_decisions: none for affected review | next_responsible_role: independent final review（同权威树、原findings+合法对照+相关分支；未变AC证据保留）。

R1真实关闭：公开零执行收据exit2拒绝，完整20原ID实际执行且证明被接受，原遗漏ID的20→19与同步短日志仍拒绝；14治理控制含nextest/libtest/Python及原负Case元oracle、缺command/退出/输出/hash/排除/重复/过滤/旧source组合。原真实45按原输入仍有效，5项删证明反例全部拒绝，当前组合不能冒借它认证新SDK字节。模块消费者49/11.119s、用途守恒19/30.132s；真实47 recipe发现后7367义务/1220叶，76职责组全覆盖。894正式新增32观察/7叶仅用途pending，不变注册/样本/预算/断言/条件；两周期清单完整47。API20真实墙钟9.570742s/工作CPU8171.875ms/作业峰334266368bytes，非P90或整目录计量。

R2真实关闭：Core host模块6/0.26s、Web HTTP文件3/5.85s；新不可变19包源candidate=false、locked冷构建2m13/安装720文件，真实CLI fresh3+relocated3精确nonce、配置端口≠环境端口、loopback/profile/hash、Host400、shutdown202/exit0/端口回收；A已不存在且移动前全树无reparse。当前SDK库217/203.198s+2原ignore，源码/安装清单测试后重核未变；必要Rust生成运行兄弟未改代码也以新不同端口driver真实重放。两新ZIP共22986983bytes并读回完整唯一清单/CRC；raw/缓存未入库。原Core128/Web13/测试140 warnings保留，无新warning/debug或必需延后；受改Rust fmt、五树whitespace/clean均通过。

当前SDK实际生产者Lang74e13580a/Webe01232b；后续语言变更是未打包治理/目录，不篡改其原来源。SDK输入hash `3bc6c24bea6ac48d236d41f4af87c2412816714f930f886ae4fa83e38a094a10`/config `d595de8766d93c16257a43fcc83b1d229c3f63d8f98466d84c28b59946037066`与原始日志hash见本地`repair-1-independent-tests.json`（SHA256 `5a88ff4552d9c6a827008894e54f8d3fb4c4a07a34beabcbbbe821edee2c8f72`）。SDK当前执行不是复用旧SDK测试数。旧45/历史14、未变HTTP算法/生成器/Man事务/UI与固定Blog15/22仅在各自原输入/未变路径复用；**新完整当前组合发行/四仓/完整tm门仍NOT_PASSED，既定T-12冻结统一门前不认证新组合**。原3个AAVM红与原ignore/manual/环境限制保留。无新GUI/全目录测试body/周期批次/远端/合入/规范/账本写入/归档/guard移除。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 29 | renewed_session_round: 4 | from: work | to: review | status: execution_done | outcome: pass | current_step: 10/12 | repair_cycle: 1/3 | previous_final_reviews: 1 needs_fix | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2506549321ad8f4699089a7b0dce53d74f78b046 | next: review | authorization: 当前完整goal继承；无必需新决定，不重复请求许可；所有T-11/12仍保留。

### 2026-10-31 T-11 affected independent final review — needs_fix（第2次）

stage: review | review_scope: final | reviewer_identity: `/root/plan890_final_review`（独立上下文，同权威五仓，实施冻结） | plan_id: PLAN-890 | plan_revision: r6 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | reviewed_commit: `2506549321ad8f4699089a7b0dce53d74f78b046` | dependency_revisions: Web `e01232b3d449ea590a85d9458d553ab17b9456a3`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | source_identity: `815c5287416833e52b7656e0c07764600a53ca532b883bd13168b728abf1ce8f` | spec_delta_sha256: `3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172` | evidence: [受影响AC/V/SD、实际公开入口反例与关闭门](#e890-1ce952cb28cda1c4), SHA256 `e3f46655d363335ec055d64eee25ba94f64cfcf74bc2b7c276c422021b9a62b6`; 同目录`t12-final-baseline.json`保存来源/锁/Spec/输入及raw哈希 | pending_closeout_gates: 全部T-12未执行 | next: work。

- **R2已关闭**：重建安装工具原公开CLI独立3/3过，不同config/env端口、loopback/profile/hash、proxy非法Host400，shutdown202/exit0/rebind；无配置/ordinary控制保留。当前SDK217/2及未变生成Rust配置证据限原输入复用；不重开T-05/T-09。
- **R1原A/B/C反例已闭，R1-D / HIGH / AC-08/12仍OPEN**：实际UI源清单3128列举/3110选中，删除原必需`auto-ui-compiler\ta2ui::export::tests::test_export_button_action`并同步listing/run/selected/count/hash→3109，生产端原规则拒绝missing1/suppressed1，验收端及公开four-owner检查仍PASS；实际Man原floor281→280同样被生产端拒绝、验收端放行。公开整包因特意缺发行腿为partial/exit2，未冒整包PASS；隔离协议夹具仅替换外部fingerprint，不声称执行测试体。缺口是原必需分母未在最终消费端复核，不是新增签名或防篡改要求。
- 原14ownership独立通过，真实20证明和遗漏/重复/错cwd/无依据排除/错实际ID反控通过，合法原ignore N/A接受；真实旧45及5个命令缺失反控、当前组合拒旧SDK、16份repair raw哈希均核。五HEAD最终clean不变，12个canonical目标hash未变；SDK打包源码与实际74e/e012生产者相同。无全目录/GUI/周期批次、代码修复、canonical/ledger或合入操作。
- 仅重开**T-08及依赖T-10**，实际完成数10→8；AC-08/12 fail、AC-13未获pass，其余已闭/原输入有界证据保留；SD-02/07/08 partial，不软化规范。R1-D为在途必修阻断，不能转为非阻塞债。
- **连续两次final needs_fix诊断门**：下一轮Work必须先记录并核验“原分母为何只在producer核、repair1一用例正控为何遮蔽一致删项”的原因与路径矩阵，按实际完整源清单建立正控、一致删除必需UI ID/Man低于floor反控；核实设计/oracle调整后才提交修复并再次回交。第2修复周期不重置自动修复上限3。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 30 | renewed_session_round: 5 | from: review | to: work | status: executing | outcome: needs_fix | current_step: 8/12 | final_review_attempt: 2 | consecutive_failed_final_reviews: 2 | repair_cycles_completed: 1/3 | next_repair_cycle: 2/3 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2506549321ad8f4699089a7b0dce53d74f78b046（review freeze，后续仅阻断簿记） | next: work | authorization: 当前全链继承；必需诊断/oracle调整门先行，无新用户决定，不重复许可。

本轮阻断已在同树`KNOWN-DEBT-AND-RISKS.md`登记，文档提交`e6f15ec01`，实现源码仍为复审冻结版本；代码未修，提交后clean。主检出仅提交本计划和两份复审/SD摘要，raw证据不暂存。文档任务数/状态/摘要hash及限定diffcheck通过；首次任务计数探针误将§10历史T-05/06行计入而失败，限定§8任务表后确认12项/8已完成，不涉及产品测试失败。

### 2026-10-31 repair cycle 2 entry — mandatory two-failure diagnosis

Before substantive edits, current producer/consumer path and source baseline/floor/migration rules inspected; original R1-D reproduced at250654932+docs e6f15ec01 with valid controls and actual public owner check (UI3110→3109, Man281→280 stillPASS). [Verified diagnosis and path/oracle adjustment](#e890-746a7a518ef25bca) records why prior single-test positive fixture missed the invariant, shared-rule design, migrated-target execution credit and permanent positive/negative matrix. Implement adjustment and verify before readiness; status executing8/12, repair cycle2/3, no new user decision.

### 2026-10-31 work repair cycle 2 — execution_done10/12

stage: work | plan_id: PLAN-890 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | base_commit: `e6f15ec019ad74cf06ea94c74e8230624d058073` | code_commit: `29cf444809e9166c6cfb98ff741b31ae6bc112cf`（共享守恒027e8b89c、合法ignore/preflight331e7b9a7、最终目录/阻断状态29cf44480） | dependency_revisions: Web `e01232b3d449ea590a85d9458d553ab17b9456a3`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | task_ids: T-08/T-10 | status: execution_done | current_step: 10/12 | finding: R1-D work-repaired, independent acceptance pending; R2 unchanged and independently closed | evidence: [修复2交接/AC/关闭表/复用及命令结果](#e890-2709e784f3aa9c94), SHA256 `b4a73dd262592082bf3e178077dd3555e9ff40f9428e1711ca6e8e121bf70f17`; [当前SD](#e890-cd5db04e23fa0dc0), SHA256 `40791c32bd48a6e30473d67748d286a7005ec428fcb149cfb7267f6e9893b681`; [必需诊断及已验证调整](#e890-746a7a518ef25bca), SHA256 `af6b63853780daee8f477048b4c2190d26b0f1992deb56edaf9d357467cc2987` | source_identity: `03f08f193042eb7670a487baf4202b97404b5c04e075dabfe9fdc2691b36bf33` | blockers/user_decisions: none for affected review | next_responsible_role: independent final affected review attempt3，同权威树，实施暂停。

两次失败诊断在代码前完成并链接§9，原UI/Man有效控制及一致删项已先复现再修复；共享producer/consumer原baseline/retirement/migration/floor，迁移credit须来自真实PASS目标，不信receipt自报。最新真实公开UI3110→3109和Man281→280反例均FAIL，合法控制PASS；永久正控保持原language/UI分母，新增一致删除/ignore/filter、伪造credit、UI libtest和任意排除反控。libtest清单不含ignore标记的兼容分支以同批ui-full原标记守恒，原T10实际866PASS/2ignored与当前active866相符，原rust.rs未变，floor863不变；未把ignored算PASS。

最终受影响owner+driver41/41、74.572s通过；用途19/19、29.481s，policy1220叶/76组/7368义务/23source rows通过。最终47配方重发现/build均exit0，源前后稳定；发现累计23.426s只是build/list口径，不是测试体计量。原真实20-ID证明在当前目录被接受，一致19反例拒绝，未重跑body。原49模块消费者及未变产品/SDK/R2/Blog证据仅按原输入复用；当前SDK包装输入对实际74e/e012逐路径diff为空，12个canonical Spec hash不变。原20条mock错误、后续9条输出夹具失败及CRLF未发生变异的失败日志原样保留；收窄mock边界、补合成输出并断言实际少1，原断言未删。初版ownership123.446s与最终41总74.572s分别归属各冻结源，不推算P90。最终五树clean，raw index位于本地`repair-2-evidence-index.json`；仅MD摘要入库。

完整tm/消费者/集成、真实四仓及当前组合发行历史恢复、到期main周期批次仍T-12 NOT_PASSED；原三个AAVM红/原条件及警告边界保留。未运行全目录体/GUI/周期批次、未发布canonical/ledger、未合入/远端/归档/guard移除。工作ready不冒独立pass。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 31 | renewed_session_round: 6 | from: work | to: review | status: execution_done | outcome: pass | current_step: 10/12 | repair_cycle: 2/3 | previous_final_reviews: 2 consecutive needs_fix | diagnosis_and_verified_adjustment: complete | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 29cf444809e9166c6cfb98ff741b31ae6bc112cf | next: review（attempt3，非重置计数） | authorization: 当前完整goal继承，无新决定，不重复许可。

### 2026-10-31 T-11 independent final affected review — PASS（第3次）

stage: review | review_scope: final | reviewer_identity: `/root/plan890_final_review`（独立上下文，同权威五仓，实施冻结） | plan_id: PLAN-890 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | reviewed_commit: `29cf444809e9166c6cfb98ff741b31ae6bc112cf` | base_commit: `2506549321ad8f4699089a7b0dce53d74f78b046`（attempt2冻结） | dependency_revisions: Web `e01232b3d449ea590a85d9458d553ab17b9456a3`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | source_identity: `03f08f193042eb7670a487baf4202b97404b5c04e075dabfe9fdc2691b36bf33`（原冻结target/env，环境hash a92a86a7...） | spec_delta_sha256: `40791c32bd48a6e30473d67748d286a7005ec428fcb149cfb7267f6e9893b681`，冻结副本[t13-frozen-spec-delta.md](#e890-55a4a012f744207b) | evidence: [独立第3次报告、受影响AC/V/SD及实际正反控](#e890-c4a9850272f2d870), SHA256 `9b642a8adf4ba70bf0fbc9ec0fbba6b016194d215b225a3d14f128280d24c028`; 本地`t13-final-baseline.json`含来源/依赖/Spec/输入及raw hashes | acceptance_results: 全部work-owned AC通过，AC-08/12及独立AC-13更新为pass；AC-14 merge-owned pending | pending_closeout_gates: 全部T-12 NOT_PASSED | next: merge。

- 两次失败后的原因诊断、原反例敏感性及已实现共享oracle调整先经独立核验，才接受ready；不是以41pass总数代替不变量检查。R1-D关闭：真实原清单UI3128列举/3110选中合法，原必需ID一致删至3109即missing1/suppressed1且公开owner检查FAIL；真实Man281合法/280低于原floor公开FAIL，未改输入或降低断言。正控公开整包仍因故意缺发行腿为partial/exit2，未冒整包绿。
- 独立源规则/兄弟控制核验language115/UI10原迁移credit；缺一个真实已过目标、空执行集或以无关目标填充均拒绝。真实UI868清单/866PASS/2原N/A与原flag、未变rust.rs、floor863相符；任意外加ID不能获原ignore身份。原四个队列退出/最低数量/ignore/零Man检查独立4/4、1.111s通过。首个源控制探针正控仅用required而未补原floor，实际3185<3189被正确拒绝；纠正为实际完整源清单后通过，保留首红，不改产品阈值。
- 独立全部350项/17,310,478bytes的size/SHA均匹配，41pass与原84诊断红分列；R1 A/B/C及R2在未变真实产品输入下保持关闭。最终五HEAD/clean及完整03f08...身份读回相同，12个canonical目标hash及实际SDK包装输入均未变。AC-08/12/13和SD-02/07/08现pass，其他AC/V/SD仅在记录的原输入/支持边界保留；无必需未完项、未批准延后或新语义决定。
- 本轮未执行全目录/独立SDK全套/GUI/周期批次，未修实现、未写canonical/ledger、未合入/归档/guard/retention。T-12须执行实际冻结tm叶/消费者/集成并集、真实四仓及当前发行历史恢复、到期main单实例周期批次、所有修改仓实际landing及规范/显式ledger/index/module-plans回读、归档和各树guard；原AAVM红/条件/警告边界保留，禁止预认证未来门。

独立关闭已登记同树`KNOWN-DEBT-AND-RISKS.md`，仅文档提交`4dbe6f6670f86432ff6e32d8f47d7563be3ec806`，实现仍复审冻结源，后续merge重核代码身份；主检出仅本Plan及两份MD复审摘要入库，raw证据不暂存。他方文档WIP保留。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 32 | renewed_session_round: 7 | from: review | to: merge | status: reviewed | outcome: pass | current_step: 11/12 | final_review_attempt: 3 | repair_cycles_completed: 2/3 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 29cf444809e9166c6cfb98ff741b31ae6bc112cf（review freeze；4dbe6f667仅阻断关闭簿记） | next: merge | authorization: 当前全链继承，不重复许可；当前pass不覆盖未来实现变更，全部T-12实际门及交付待完成。

### 2026-10-31 T-12 merge prepared checkpoint — actual gates running

stage: merge | checkpoint: prepared | plan_id: PLAN-890 | plan_revision: r6 | status: reviewed11/12 | reviewed_freeze: Lang29cf44480 + fixed peers, independent final pass9b642a8a.../SD40791c32... | current_delivery_candidate: Lang `3c44ec3264f7f536d5bc57a1f6d8b1f9764f2f70`（rebase e1600601f + approved canonical/projection f5c7b47f4 + actual refreshed catalog3c44）, Webde1783f, UId347dd3, Man b9f0c02, Downc3464d0 | evidence: [准备/重写等价/规范检查](#e890-a115ccd89e8ef3a2), 本地pre-merge-baseline/rebase-source-equivalence/language-range-diff/prepared-canonical/canonical-preparation-check/frozen-union-contract JSON/TXT | actual_gate: RUNNING, not pass | next: complete or diagnose actual frozen gate before landing。

语言55个unique提交线性化，52个replay项中重复上游簿记丢弃，当前共享Plan保留main最新状态；最终非Plan Git blobs与原候选相同，无Rust产品冲突处理。47配方rerere合并内容核等于已审定最终配方；checkout物理换行造成raw来源身份变化，未冒原03f08。原Git来源/17资源契约通过。已审SD01..09准备到各权威树，用户要求的兼容ledger/module plans/overview与实际index generator完成；41份MD新增链接检查0错误，归档路径仍是计划目标，实际归档门后核。各仓未landed，未归档或guard清理。

受影响277路径选出1220完整叶/7368义务，保守未知触面扩大保留；实际四仓language schedule与选测modules及来源身份完全相同，`7bbb7a3bf6dc12e8e271f06c69a30bcf2d68288933c077beff8e51d4d507cf44`，一次并集同时满足受影响与必需组合语言腿，不额外重复旧档。Cargo启动环境与direct Python的PATH/有效环境身份不同而逐仓source-input digest一致，分别记录不混认。使用明确`cargo run --manifest-path scripts/full-regression-driver/Cargo.toml -- tf --batch --with-web --frozen-trees`，不是在worktree调用main-only周期别名。

首次自有门启动前发现他方nextest，但未等检查返回便启动；仅按唯一frozen-output参数+PID/父进程/birth核实后终止自有8652进程树，保留language-list0字节等partial原件，未认证body或停止他方。确认后重启fresh `four-owner-final-retry-1`；自有exec session39093仍运行，已开始真实模块执行，最终门未过。迁移清单永久引用原e45399a6且rebase后非祖先，已保留本地tag `plan-890-source-e45399a6`供长期来源校验；仅临时backup分支待清理。周期/发行历史/保留预算及所有landing收尾仍pending。


### 2026-10-31 T-12 actual frozen gate needs_fix — terminal checkpoint

stage: merge | plan_id: PLAN-890 | plan_revision: r6 | outcome: needs_fix | status: executing | current_step: 8/12 | frozen_candidate: Lang3c44ec326/Webde1783f/UId347dd3/Manb9f0c02/Downc3464d0 | evidence: [失败/停止/原报告恢复及关闭oracle](#e890-0d80b9b7314a384d), SHA256 `9f665665262f450a173cc395e94dcb907197c753851440506e906052df215cf6` | next: work。

上一prepared收据的RUNNING状态现已终止：实际1220叶/7368义务门只有60模块结果，25PASS/4FAIL/31原条件NOT_APPLICABLE，executed29，complete=false/gate_passed=false。缺a2x-worker导致六项中五红一过；该模块workload100、module124来自vctip残留后代（已清理），不能误写为测试超时。两个AAVM编译叶E0433缺auto_lang路径，goldens参考执行INVALID_BYTECODE60；具名旧红仍须关闭，不能以债豁免。只终止经参数/父进程/时间核实的自有40192树，session39093 exit1；余1160叶无完成结果，外仓/最终发行恢复未完成，无根完整收据。十个原跟踪报告从启动快照原字节恢复，五树clean；未改实现、未动他方进程或清理缓存。独立第3次PASS不被改写为失败，但新修复输入须重审。M1/M1b/M2/M3在阻断簿登记，语言文档提交`d4158137e`。

重开T-08/T-10/T-11，8/12；T-01..07及T-09原有界完成保留。下一Work先核实际原因/区分性oracle，再第3/3修复、定向完整叶及消费者验证、受影响独立复审；不得静默开启第4周期、减少义务或预认证未来门。未合入各默认分支、未归档/guard清理；到期main周期批次及最终landing/readback均pending。既有全链授权继承，无新用户决定。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 33 | renewed_session_round: 8 | from: merge | to: work | status: executing | outcome: needs_fix | current_step: 8/12 | final_review_attempt: 3 | repair_cycles_completed: 2/3 | next_repair_cycle: 3/3 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 3c44ec3264f7f536d5bc57a1f6d8b1f9764f2f70 (frozen gate; d4158137e blocking bookkeeping only) | next: work | authorization: current full-chain grant inherited; no new approval required.


### 2026-10-31 repair cycle 3 completion — affected independent review handoff

stage: work | plan_id: PLAN-890 | plan_revision: r6 | outcome: pass | status: execution_done | current_step: 10/12 | task_ids: T-08,T-10 | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | base_commit: d4158137e5244e8ef352ad70bf5bf904f9ad8cee | code_commit: 2ef7bece2fa6b2509ed04506789059b997728093 | dependency_revisions: Webde1783f1c94dbc4d656958faca74ab61a69831ba/UId347dd3d888545f6f07fe67b7f4c7f1be28741c8/Manb9f0c0259ae8b167a635c978dab551e382dbc894/Downc3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | evidence: [修复3交接](#e890-a013784219215e74), SHA256 `50ed90bf46a1f517c785c0ebf26184d72fca99e8792ec975e862040a5765bc5b`; [诊断/原红/设计调整](#e890-e0613fdbf61b84e5), SHA256 `0305691854ef1b678f288e30f37b7d055250a79d53e9ba3fc27e1fb8ef77397d`; [SD增量](#e890-f45a42b3a0d8b00b), SHA256 `a77d62978dfe65230a12e993adcc20c3688cc81f1925924db33191fe8acd816c` | blockers: none work-owned | next_responsible_role: independent affected final review。

M1/M1b在既有工具准备边界构建a2x并传明确worker，原六项控制全过、workload meter无后代错误，不增加测试体豁免；bare维护固定target。M2原guest发射契约保留，用薄命名空间转发原File实现并让独立Cargo target/缓存发布一致；最终原compile-corpus62.081s/use-corpus24.777s均过，meter94.576s/CPU76s/Job858910720/error=null。通用发射器的临时调整在调用方审计后全部撤回，未保留改变其他merged工程的方案。M3真因是CALL_CLOSURE一字节argc被漏读，参考闭包走查改用同一decoder的future四字节capture规则；原goldens及全部reference7控制均过，包括两个新合法/非法后继反控。第一轮13项11过/2编译红和脚本NameError（仅列举后、体执行前）均保留，最终消费者2过覆盖其新输入；18项不同Rust、Python53过，原ignored VM用例不冒过。原生引用/签名/拒绝门、断言/金样/样本/条件未削减。

真实47配方discovery全部0；首次catalog拒绝新leaf无top-owner，补入既有core/stdlib-assembly（aaa7307f7）后真实公开tm选择exit0，目录7371义务/1221叶，旧ID无删除，仅3新反例。原5完整seed经消费者/集成展开778叶/5626义务，本轮仅select-only，不在Work重跑全目录。发布exact实际runtime catalog2ef7bece2；Cargo公开来源`5a02e7b8ce6c89e5d1b8b5f18e4839406d9e3ad383eb485f3e1da11b3d5ee9aa`、direct检查`983c36ac26c1f0bf458a54438950ec40c94ce850ab0bf96a9bce4c824a582198`环境口径分列，逐仓byte输入digest等同。46份raw/6352734bytes、5HEAD/clean/SD与源身份在本地repair-3-final-baseline.json存hash，raw不入库。

Core改动rustfmt/限定diffcheck过，原完整build+nextest与当前均288种warning消息、无新增类别；不冒全库零warning。独立Web目录位置和受限generation载体导航已按SD准备；另见阻断簿的非本闭包ABT历史候选，不将其宣称本计划修复或用其延后M3。工作范围无未完finding/语义决定；旧R1/R2受影响行为来源规则未改，已有独立证据限相应输入复用。新SDK来源的真实发行/全四仓、冻结完整闭包、各仓landing、规范/显式ledger/index回读、实际归档与guard、到期main周期等仍T-12，均NOT_PASSED，未完成U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 34 | renewed_session_round: 9 | from: work | to: review | status: execution_done | outcome: pass | current_step: 10/12 | final_review_attempts_completed: 3 | repair_cycles_completed: 3/3 | next_repair_cycle: none_without_user_decision | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2ef7bece2fa6b2509ed04506789059b997728093 | next: review | authorization: current full-chain grant inherited; no new approval required for independent review and current merge-owned gates。


### 2026-10-31 independent final affected review attempt 4 — PASS

stage: review | review_scope: final affected re-review | reviewer_identity: /root/plan890_final_review（独立上下文，同权威五仓） | plan_id: PLAN-890 | plan_revision: r6 | outcome: pass | status: reviewed | current_step: 11/12 | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | reviewed_commit: 2ef7bece2fa6b2509ed04506789059b997728093 | base_commit: d4158137e5244e8ef352ad70bf5bf904f9ad8cee | dependency_revisions: Webde1783f1c94dbc4d656958faca74ab61a69831ba/UId347dd3d888545f6f07fe67b7f1be28741c8/Manb9f0c0259ae8b167a635c978dab551e382dbc894/Downc3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | evidence: [独立报告](#e890-782acb3b7e32d925), SHA256 `b935eab66852812842c9d20d8cfd2d7a851cd4d77487c4de01d7f1c418d19658`; [冻结SD](#e890-15513e7611658e98), SHA256 `406de25245818691de6d48b33951302297af59e6d0280642c3d7e8617b163b48`; 本地t14-final-baseline.json SHA256 `01054326d50bd917b8af7d9f405bf05c8711b87ac9b69434990015cb779a19fe` | pending_closeout_gates: 全部T-12实际门NOT_PASSED | next: merge。

M1/M1b/M2/M3独立复审关闭，无新增work-owned finding；AC01..13与SD01..09按现行源码/先前未变机制及明确支持边界通过，AC14 partial。实际prepare a2x exit0产出相同worker77d5...，正常/坏golden/缺worker三原控全过；真实精确哈希/已回收辅助进程反例及严格workload泄漏拒绝通过。M2实际原两腿当前最终receipt经原始SHA/断言核验，独立提取当前原prelude，六条Unicode/普通/EOF/零limit/usizeMAX/缺文件转发对照编译执行过；临时通用emitter方案已完全撤回，不借临时绿。M3真实reference7/7过，独立构造argc0/1/127/255、future0/2/3、截断与生产native1902/未证明拒绝共18断言过，实际指令边界/native proof仍strict。首次错误unittest类loader失败保留，纠正后的实际控制通过，不冒产品红。

独立核46份raw/6352734bytes全hash，47recipes/7371义务/1221叶守恒，旧ID无删只3新增；原5reference与新2boundary所属leaf和已有owner均核。4dbe→e160非Plan Git blobs无差异；准备的canonical/ledger/index/module-plans/overview逐项核，67个输入SHA冻结，父随后复核同67/67。用户显式ledger/index规约高于skill新默认；实际交付report/归档链接与当前SDK认证仍未来门。原两次诊断及provisional/脚本/目录首红历史均保留，不豁免原AAVM失败。ABT其他表确属本native-reference调用路径外，不宣称整体VM格式重构、不借其延后M3。

来源身份分列：公开Cargo目录`5a02e7b8ce6c89e5d1b8b5f18e4839406d9e3ad383eb485f3e1da11b3d5ee9aa`、Work direct `983c36ac26c1f0bf458a54438950ec40c94ce850ab0bf96a9bce4c824a582198`、review direct `182f6ed1c7d5ffd421b08b4422e921ee1f9e89cb1b64793514a1541bbeba8d0d`，环境不同而五repo输入bytes摘要相同，未混认物理来源。所有HEAD/clean在独立末尾及父读回保持；新增格式/diffcheck过，原288类warning不冒零。无实现/配置更改、全目录/GUI/周期批次、canonical/ledger写入或任何landing/archive/清理。

独立闭合在原树阻断簿登记，仅文档提交`d7e1ffdda775d55aeddcea2af068a2d3aef6e752`；与review freeze唯一Git差异为KNOWN-DEBT-AND-RISKS.md，代码/目录/Spec输入不变。主检出只回写本Plan和两份MD复审摘要，原始JSON/日志不入库。他方WIP保全。下一T-12实际冻结完整tm叶/消费者/集成并集、真实四owner与当前SDK发行历史、所有修改仓landing和来源读回、规范/显式ledger/index/report沉淀、实际归档门/各树guard清理与保留预算、到期main单实例周期批次，全部仍NOT_PASSED，不能用本PASS代替。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 35 | renewed_session_round: 10 | from: review | to: merge | status: reviewed | outcome: pass | current_step: 11/12 | final_review_attempt: 4 | repair_cycles_completed: 3/3 | next_repair_cycle: none_without_user_decision | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2ef7bece2fa6b2509ed04506789059b997728093（review freeze；d7e1ffdda仅关闭簿记） | next: merge | authorization: existing full-chain grant inherited; no repeated permission required; no future code/gate PASS is inferred。


### 2026-10-31 T-12 final frozen gate retry — running checkpoint

stage: merge | checkpoint: prepared/actual_gate_running | plan_id: PLAN-890 | plan_revision: r6 | reviewed_freeze: 2ef7bece2 + fixed peers, final t14 PASS b935eab6.../frozen SD406de252... | pre_rebase: d7e1ffdda (review closure only) | rebased: c171399c3435cdf7a771104c90c39501c4b52e96 | frozen_candidate: eb817ad81fb262f0d6c64d8f21cfd0df7f67fe8b / Webde1783f / UId347dd3 / Manb9f0c02 / Downc3464d0 | next: wait exact active gate handle63703; no duplicate restart。

主检出4608d216d以来无新的上游代码：语言49提交rebase到该主分支，实际range-diff49/49全等，无非Plan Git blob改变；仅带入main十份Plan/摘要MD，物理checkout来源与旧审查身份分列。真实公开cargo tm --changes final-2/changed-paths.txt --select-only exit0，47配方枚举、1221完整叶/7371义务；该选择与four-owner显式schedule全并集相等，旧义务ID未变。新实际runtime catalog仅3行来源元数据刷新并提交eb817ad81；公开code身份6c9f6fb1446b81e2c3e080c2ca16a91a50b21e01bac3509529f5857268782681，不冒旧5a02或旧SDK来源。

原五树clean，启动前无其他cargo/nextest/rustc/auto-case进程；D余181760057344/G余29641084928bytes。实际启动 `cargo run --quiet --manifest-path scripts/full-regression-driver/Cargo.toml -- tf --batch --with-web --frozen-trees --output D:/autostack/auto-lang/docs/plans/attachments/890/merge/final-2/four-owner`，原五AUTO_*_ROOT、G:/autostack-build/plan890-frozen-target、RUSTC_WRAPPER空。自有Cargo根PID2516、父37576、创建09:27:03，子Cargo64388/driver40008；exec session63703尚在运行。此显式worktree组合门不是main-only周期别名，也不额外叠跑旧语言档。尚无整门结果，不认证后续外仓腿/当前发行/历史恢复。

本地final-2保存entry-baseline/rebase-equivalence/range-diff/changed-paths/frozen-union-contract/gate-launch JSON/TXT/日志；原中断结果不覆盖。首次登记890活动证据hold（精确主attachments和既有source路径，owner auto-lang/PLAN-890 merge、2026-11-14复核/14天口径），预计1221叶原始清单/日志或超500MiB，临时具名1536MiB上限/理由/到期登记；实际最终量及其他外部scope仍须核，不宣称全局预算绿。未运行保留删除，用户14天优先于安装helper最新五计划默认。此checkpoint仅在main未提交簿记，避免正在冻结门时为簿记再改默认分支；实际landing前复核main未发生上游变化。


### 2026-10-31 user-directed PLAN-896 integration (r7)

User explicitly directed replacing final tm execution with the delivered896 cargo t, integrating896 and rebuilding/testing. Main c2126b8e9 contains896 delivery a9bd4f948 and archive/cleanup receipts. Old frozen gate session63703 was already terminal exit1, receipt error language module tests failed2,230 records and6 FAIL leaves; original red remains, not waived by new method. No owned old process remains; original reports restored and candidate clean before rebase.

Integration baseline eb817ad81 retained as codex/plan-890-pre-896; rebase onto c2126b8e9 completed as07246f6e585d007065f39ff528daaaf2ca698116. This is a real upstream code/framework integration, not a pure equal rewrite claim. Shared regression driver/tests kept upstream fixed-group implementation; movedACE rust_lowering kept896 owned compiler/target behavior plus890 explicit worker resolution. New upstream ACE workspace/toolchains retained; no resurrection of retired paths. Remaining four-owner/source admission integration is a known work-owned task before readiness, not silently dropped.

Actual cargo t --out attachments/890/work/integration-896/daily launched session26831 with five sibling roots/RUSTC_WRAPPER empty/CARGO_TARGET_DIR inside890 worktree. First attempt usingG target was correctly rejected before execution by896 ownership guard; log integration-896-target-preflight.log preserved. No test PASS yet. Detailed source/conflict/command entry in local integration-896/entry.json. T-08/T-10/T-11 reopened; status executing8/12, next continue user-authorized integration/retest, no implementation agents or remote actions. Historical automatic repairs3/3 are not reset or relabeled; new work follows the explicit user instruction.


### 2026-10-31 r7 integration daily verification checkpoint

stage: work | scope: user-directed896 integration/rebuild | plan_id: PLAN-890 | plan_revision: r7 | outcome: executing (daily integration passed; remaining T08 adapter work) | current_step: 8/12 | code_commit: d171e3632f7d41e2bd965b94af6f3688c0d4c685 | upstream_ancestor: c2126b8e9 including896 deliverya9bd4f948 | evidence: [整合/冲突/编译/真实终态](#e890-17874c445132c12f), SHA256 `89058660a70f69d43885b2a162be25e84f7106e3276a53bce1fb5a5cc5775ceb` | next: finish fixed-group four-owner integration and affected validation before independent review。

首轮实际cargo t完整日常2970项中2964过6红，编译/发现203.616秒、执行116.875秒、整体320.635秒；五个FFI oracle子构建误继承外层target、缓存marker未核产物，一项705静态调用图仍只扫Core旧owner。只修测试接线：子工程显式自有target且缓存必须有真实exe，调用图覆盖Core中立桥和Web实际owner，原legacy恰一处/禁止未标记同步调用与VM/oracle/golden断言全保留。新cargo t按原日常配置定向5+1均过。修复提交后再一次完整新日常组2970/2970过，meter.error=null、准备2.145秒/执行75.767秒/整体78.032秒；nextest skipped667含过滤，正式receipt另列106原ignored身份，不把过滤或未执行实验记通过。最终results.json SHA256 `50f0e4a6e8e159f37584277bd61eea07dda2af20f22c8f593e55b25a3d1f975f`，raw本地保留。

本次确实重编译并执行了896新入口，没有用tm或all全实验/全仓图。890四仓runtime接入尚待把旧模块证明改为固定组实际证明，旧tm管线不能恢复为新runner前置；Web/UI/Man及当前SDK/历史恢复等仍要实际覆盖，新来源受影响独立复审与最终合入收尾未过。T08/T10/T11保持重开，不因日常组绿提前交最终审查。Root/同组代码无未提交WIP，main仅本Plan/摘要及外来文档WIP，未merge890到默认分支、未远端/归档/guard清理。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 36 | from: merge | to: work | status: executing | outcome: needs_fix (user-directed upstream/framework integration; daily passed, fixed four-owner adapter still open) | current_step: 8/12 | historical_repair_cycles_completed: 3/3 | final_review_attempts_historical: 4 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: d171e3632f7d41e2bd965b94af6f3688c0d4c685 | next: work | authorization: explicit user request integrates delivered896 and uses new cargo t; prior full890 objective retained, historical caps not silently reset。


### 2026-10-31 r7固定组四仓入口工作交接

用户再次明确按最新AGENTS完成890全链。冻结实现87f7058dd，Web16d8378、UI5cfb5c8、Man6633feb、Downc3464d0。显式语言组为integration/auto-generation-core/auto-api-gen/auto-media-core/a2r-std/auto-test-case；Web、UI和Man原生八腿保留。未恢复tm、全叶目录或自动消费者图；未用all作为每计划门。原迁移必需ID/原floor/ignore/真实media PASS信用、696原历史行/116身份、wrapper child oracle和来源/命令/终态逐项校验。新增真实Cargo路径来源核、tracked/untracked漂移拒绝、无started的伪ok终态拒绝。

定向验证：`python -m unittest scripts/test_four_repository_fixed.py scripts/tests/test_fixed_groups.py`45/45通过（随后新增两项来源负控由正式cargo t执行）；`python -m unittest scripts/test_full_regression.py`27/27通过；`cargo t web-delivery-controls --out .../integration-896/fixed-owner-controls`10/10通过，exit0；实际四仓locked/offline metadata路径核通过。三外仓锁仅更新auto-cache的auto-tool-execution依赖，无版本漂移。Spec受影响两页改述896固定组。完整六组/八外腿、当前SDK发行与历史恢复在冻结版本独立代码/验收/Spec审查通过后由T-12执行；此处不提前认证。独立角色plan890_final_review现做受影响复审，未合入/归档。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 37 | from: work | to: review | status: execution_done | outcome: pass (implementation and targeted controls; final frozen unified gates pending T-12) | current_step: 10/12 | historical_repair_cycles_completed: 3/3 | final_review_attempts_historical: 4 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 87f7058dd | next: review | authorization: latest explicit full-chain renewal; no historical cap reset.


### 2026-10-31 r7独立复审F15-1与定向修复

独立上下文plan890_final_review发现新负控以next(iter(selected))选择victim，可能误删允许的extra；实际seed1复现测试失败，生产准入guard正确。记录needs_fix，不用原10/10覆盖该反例。04c369567固定从原required-minus-moved选择删除对象，并新增删合法extra仍接受正控；seed1/seed2各10/10通过。Web85afa90与语言Spec仅将实际SDK闭包19改20，新增依赖来自896。

新cargo t ace-workspace筛rust_lowering首轮在工具准备阶段失败，ACE锁仍含Core旧axum而缺纯Webcontract；测试体未执行。原FAIL保留在integration-896/rust-lowering-current，103.114秒，未冒通过。6879f71ec更新ACE锁为实际纯contract闭包，无第三方版本升级；新固定ace-rust-lowering组覆盖迁移后完整host::rust_lowering::命名空间，仅准备其真实a2x worker，不为六控制准备其余AV/native工具。当前新cargo t定向运行中，独立受影响复核/最终组合门待核。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 38 | from: review | to: work | status: executing | outcome: needs_fix (F15-1 new deterministic negative-control fixture) | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 5c7e449af9858e6f3619e2a9785ad98d2efe7b88 | next: work | historical_repair_cycles_completed: 3/3 preserved | authorization: explicit latest896 integration/full delivery; this finding belongs to new r7 adaptation, not reopening historical repairs.


### 2026-10-31 r7有界修复交接（f79331598）

F15-1确定性负控已关闭：seed1/2各10/10，合法extra删除正控接受；生产guard未削弱。当前Lang f79331598786849a9b610151a03a2171af127c55、Web85afa90、UI5cfb5c8、Man6633feb、Downc3464d0均冻结。ACE真实发现173项，最初按物理host目录猜命名空间选0，exit1拒绝保留；改为真实Rust tests::rust_lowering::完整命名空间后，新cargo t ace-rust-lowering实际6/6通过（167未选，不认证ACE全集），涵盖golden负控/缺worker/凭据和certify路径。原case-parent工具准备编译100秒、a2x准备19.61秒及conformance冷编译64秒分别保留，不拿warm5.819秒冒充整命令成本。37 fixed控制串行全过；中途并行自测一项触发真实lease冲突、隔离重放37/37，按测试环境并发记录，不改guard。SDK20包Spec已对应真实新增依赖。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 39 | from: work | to: review | status: execution_done | outcome: pass (F15-1 fixed; moved ACE6 controls actual pass; final unified gates pending) | current_step: 10/12 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: f79331598786849a9b610151a03a2171af127c55 | next: review | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 1.


### 2026-10-31 r7发行/原integration选择补齐交接（8372bb8ef）

自查关闭不能交给未来门豁免的旧假设：Web四仓选择从lib扩为lib+tests，真实nextest列举221活跃+2原ignore，包含http_runtime三项和http_generation一项；不拿217库测试替代这4项。Web testing/delivery canonical改新固定入口。发行consumer严格20包且必含auto-tool-execution，原45步/217库/2ignore/历史恢复oracle不变。immutable Man installer提交仅表脚本来源，当前Core CLI提交通过consumer_sources独立记录，来源必须与selected Man core/crates/scripts/locks一致，原Git恢复材料和来源不放松；新增错CLI pin/变锁/一致删第20依赖负控，实际consumer方法通过。Man core独立lock补auto-cache→auto-tool-execution，locked/offline metadata通过，根与Core各源身份分列。

当前冻结Lang8372bb8ef75b202a1bd2760c7107e1268f141c9a、Web7ccfa17ec99177e2dc7617c95d6ff43822d6d98d、UI5cfb5c8、Man8b6e334a03fac52a237bcd0a20ef49453dc86b73、Downc3464d0。新cargo t web-delivery-controls11/11过，raw integration-896/delivery-consumer-controls/results.json；包含原真实发行consumer所有变异，不运行旧tm包装fixture。ACE六项f793证据有界复用：之后未变其源码/固定组/工具/样本/编译配置，后续仅Python准入与外仓Core锁/规范变更，不能按旧HEAD冒当前整仓执行。原首红全部保留。阶段仍work→独立受影响复审；最终八外腿/六组、当前20包SDK及历史恢复完整门均待T-12，不提前认证交付。


### 2026-10-31 最终主干同步与唯一冻结来源

并行897已正式落地，主干后续zip whitelist两行也必须保全；先整合9fa721947，再整合0e2bf7b2b，均无冲突。当前唯一Lang代码freeze93ddf4a05501e0792ae0f6d664a4070a6d739999，Webab09fd8842dbfb4a1062d32886609f33dd664bb4、UId1424556b167b225c260635f30d9e941cf508c34、Man43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6、Downc3464d0。五个真实consumer锁同步ZIP闭包，原所有name/version/checksum保持（removed=[]），Web/ACE各新增20外部包、UI/Man/Core各17；所有locked/offline metadata通过。离线首轮缺derive_arbitrary1.5下载后再次离线核通过，首失败保留。上游897仅进入现行基面，不纳入890功能宣称。其最新archive主干3fa75dd9c为Plan纯文档，后续ff映射需证明无运行输入变化。

最新实际新cargo t：ace-rust-lowering6/6，整命令15.991秒，results SHA256cc576c32d7f5e485500ffb22eea5c2aa949e35a6f8c53843ce801c9bbe2f9f87；web-delivery-controls11/11，1.004秒，SHAce879d448d53ccb1c43a91d5c22f36c6e5e3998e64d332bf49347a5814836a6f。两者source_state56197c4a3a03c32dace9d9bfc8aa85458548816ea7936ac0aa408150458c1285与当前精确一致；raw分别integration-896/rust-lowering-final-master与controls-final-master。未因旧主干ACE绿复用到变化后的Core；独立上下文将核最新来源后出最终结论。八外腿/六组/当前SDK发行恢复/合入/归档/guard仍NOT_PASSED。


### 2026-10-31 r7独立最终受影响复审通过

独立上下文/root/plan890_final_review完成t17最终PASS，冻结Lang93ddf4a05501e0792ae0f6d664a4070a6d739999、Webab09fd8842dbfb4a1062d32886609f33dd664bb4、UId1424556b167b225c260635f30d9e941cf508c34、Man43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6、Downc3464d0。报告[独立r7复审](#e890-74707f31f0dcaf7a)，SHA256d68aaab1cb88030183a6bd14289866b6cdd41c0f4d1c625720a1c6d85385b0f7。F15-1、旧无started/混仓/untracked准入、新ACE6控制、20包严格闭包/Man CLI独立来源/Web真实四integration targets均已关闭；未触面AC按报告精确范围复用。T-11完成，T-12仍包含六显式语言组、八外腿、当前20SDK发行/历史恢复、各仓实际landing、canonical/ledger/index、真实archive gate及各guard清理，均NOT_PASSED，不能凭review PASS合入。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 40 | from: review | to: merge | status: reviewed | outcome: pass (independent final affected t17; unified gates still pending) | current_step: 11/12 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 93ddf4a05501e0792ae0f6d664a4070a6d739999 | next: merge | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: explicit renewed authorization and recorded F15 closures; no erased history.


### 2026-10-31 r7最终门首次实际失败（不覆盖为绿）

冻结93dd统一语言六组已实际完成，695.533秒；integration3682项3674过8红，auto-api-gen3项2过1红；auto-generation16、media54、a2r-std104、auto-test-case93均过。首原始results SHAa0c2a13147e7e903ce2893c8bc733d8cfc9db14b39957431b29ea5be1f12f5e8；四仓receipt outcome fail SHA6ceb07e6acdb3910fc5f392c36dbcf30bb0bb8c6c50ecec16c7892c9a6f3bc09，外腿未启动。root下49包装调用/436子比较校验valid，不等于全部测试绿。实际integration执行347.095秒，meter发现vctip后代且已收尸，strict出口124保留，未豁免测试体后代泄漏。原条件排除750不计PASS。

具名失败：back_proxy真实031 native names输出负哨兵数组、018旧book_id/template路径；872派生binding表漂移；884新增zip_io未入provider指纹；724 sync/async与727 sync/async原C临时构建缓存的libsqlite3-sys/out/bindgen.rs缺失；734 Tauri generator多行registration与832冻结单行fixture不符。四个缺失缓存均保留旧build marker而out目录03:10被改，不能归代码编译不兼容；需精确cache诊断/定向重放。进入Work有界修复机制，不改golden/减少assert/降低floor，不以旧r6或新11控冒门通过。t17独立PASS保留其原冻结边界，修复后受影响复核与新冻结统一门仍必须。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 41 | from: merge | to: work | status: executing | outcome: needs_fix (actual unified gate9 atomic failures plus strict compiler-descendant cleanup) | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 93ddf4a05501e0792ae0f6d664a4070a6d739999 | next: work | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 2 (first real frozen unified run diagnosis).


### 2026-10-31 首次最终门有界修复交接

原9个原子失败已定位并定向关闭，原golden/阈值/条件/原子断言未弱化；原vctip strict失败不豁免，724/727改既有owned Cargo生命周期后3+4实际过且meter.error=null。当前冻结Lang4992de5fbcf01725673d15c03b34e9690442f291、Webe144ec0d9febe74f8cf734744d91ade47fe57bcf、UI4da2201fa46bd8a88bc5c2d661b21433b7ac84b5、Man43a8f1e、Downc3464d0；同步master3fa后全部实际源码字节hash未变，range-diff/原快照保留。当前源上的typed/真实031各1过、新准入12过；其他7项与Tauri消费者3的实际scope/hash/复用边界见[修复交接](#e890-12e120acc8ca8d27)。真实Web两profile网络+upload3/3清洁通过，完整当前组合仍待最终门。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 42 | from: work | to: review | status: execution_done | outcome: pass (scoped repair evidence; no final unified gate credit) | current_step: 10/12 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 4992de5fbcf01725673d15c03b34e9690442f291 | next: review | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 2.


### 2026-10-31 r7修复最终独立复审通过并移交合入门

T18独立最终PASS，报告[修复复审](#e890-a9be80225212b9a0) SHA2564b50b42a80d3d1aca15d70b3a5b26e4af3b556fd0d0f6ded6fc050763125f098。冻结Lang54b5d7c92f0244faecde929353b7e292ff6b3579/Webe144ec0d9febe74f8cf734744d91ade47fe57bcf/UI4da2201fa46bd8a88bc5c2d661b21433b7ac84b5/Man43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6/Downc3464d0。最后Schema guard只变Python两文件；13控/source_state9154精确且独立Git反例通过；原Rust/TCP证据限实际字节未变范围复用。SDK新导出20包candidate=false，source_sha2568c55a2d281e9d96355799d958e2793fba0048091d16b49cd21fa8605065359db。六语言组/八外腿/当前SDK发行与历史恢复/各仓landing/canonical/ledger/index/archive/guard仍未通过，开始T-12。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 43 | from: review | to: merge | status: reviewed | outcome: pass (independent final affected T18; merge-owned gates pending) | current_step: 11/12 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 54b5d7c92f0244faecde929353b7e292ff6b3579 | next: merge | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 2.


### 2026-10-31 第二轮统一门真实失败（Man原消费者漏适配）

原冻结54b5上六语言职责组3954全部过；Web221过/2原ignore、UI-full3121过/18原ignore，Man-t281过/4ignore；原生腿实际出口[('web', 0, None), ('ui-full', 0, None), ('man-t', 0, None), ('man-tcore', 0, None), ('man-tc', 0, None), ('man-ti', 124, 'descendants live after workload exit')]。原统一receipt SHA2569c2e7ecbab4b3789b498dd251b94ad89147c97c12bb5fd589e4ff3db70a934bd，outcome=fail。Man-ti43项42过1红：plan811_media_consumers生成源码观察仍include Lang auto-api-gen薄forwarder，真实实现已迁Web；原media公开入口断言保留，需按真实owner观察修复。meter同时发现自有vctip在根退出后仍存活（随后已清理remaining0），严格失败保留，不能作为准备辅助豁免。先单独计量复现定位，再修复原builder owned生命周期，独立核查后重跑受影响门；不因此重复已通过六语言组/不运行cargo t all。UI-tu/UI-ti未启动，当前SDK发行/旧新恢复及landing/归档/清理未通过。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 44 | from: merge | to: work | status: executing | outcome: needs_fix (Man original generator source observation and strict workload descendant cleanup) | current_step: 10/12 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 54b5d7c92f0244faecde929353b7e292ff6b3579 | next: work | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 3 (second actual unified-gate diagnosis; bounded repair, no historical cap reset).


### 2026-10-31 Man原消费者与构建生命周期有界修复交接

Man d2b2664fea33a93def31ef4cb04be7110c97e807冻结，其他四仓未动、无新依赖/锁/配置/金样变化。媒体原ID/全部断言保留，改读真实Web实现并加Lang facade委托断言。原v05热态8.954秒strict124复现后，Cargo/Ninja生产build用已有owned设施保持stdin/stdout/stderr/env/cwd/argv及原ExitStatus；Windows显式清理自有Job，非Windows保原终端组。原媒体3+真实构建1+新退出码/NotFound2实际6过且meter.error=null。前次FAIL及精确hash、未变字节与本地恢复编排设计见[修复交接](#e890-49c991d9b3da6137)。待T19独立复核，原六语言组/Web/UI实际结果仅按准确未变输入复用；Man四腿/未执行UI两腿及其余交付收尾仍pending。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 45 | from: work | to: review | status: execution_done | outcome: pass (Man bounded source/lifecycle repair and6actual directed controls; final affected review/gates pending) | current_step: 10/12 | worktree: original five-owner group | commit: Lang54b5d7c92; Man d2b2664fea33a93def31ef4cb04be7110c97e807 | next: review | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 3.


### 2026-10-31 T19独立PASS，恢复冻结四仓门

独立/root/plan890_final_review已发布[T19](#e890-d0511e724d4bee1e)，SHA2568c695e50599d6df83e228e98461162fb2c23d54da1268e98eff1cd852a9c054f；绑定Lang54b5d7c92/Webe144ec0d/UI4da2201f/Man d2b2664fea33a93def31ef4cb04be7110c97e807/Downc3464d0。Man5文件修复、原ID/断言保全和local恢复脚本2a2f4c8825851a1a1205220c1013a9c5b126b7694ffaafad81d48a176b8ca574均通过。原实际调度ID归一仅用于比较：完整环境hash与原8fe46精确相等，新执行保留新真实身份；工具固定与历史时间/缓存推断边界详见工作交接。开始恢复门final-896-3：仅复用原Lang3954/Web221/UI-full3121实际结果及原source/log身份，重跑全部Man四腿与未执行UI-tu/ti，最后仍完整public validate/fresh Cargo分母/children/terminal/source checks。旧FAIL永远不改，新SDK20/发行45/历史14与合入归档清理仍pending。

stage: goal | plan_id: PLAN-890 | plan_revision: r7 | round: 46 | from: review | to: merge | status: reviewed | outcome: pass (T19 independent final affected; frozen recovery and delivery gates pending) | current_step: 11/12 | worktree: original five-owner group | commit: Lang54b5d7c92; Man d2b2664fea33a93def31ef4cb04be7110c97e807 | next: merge | historical_repair_cycles_completed: 3/3 preserved | r7_framework_adaptation_repair: 3.


### 2026-10-31 冻结四仓组合门实际通过

最终组合receipt SHA256734410720b8026390401da1fe21826654c421ca8a212d77b21e541367084df40，路径attachments/890/merge/final-896-4/four-owner/receipt.json。语言六组3954、Web221、UI-full3121输入未变实证复用，Man283/24/254/43与UI-tu866/UI-ti28新实际通过，原条件排除单列。实际正常执行exec-8ebe exit0，完整public consumer及6+8实际新分母/子比较/终态/源码检查通过；原FAIL与中断不覆盖。独立[恢复核查补充](#e890-2adbc3f91dba4d5f) SHAe9dccc506f1430e8641b8b3e57e245451f6fc048d7328890e8573ce646ffdc47确认实际通过并关闭本地finalizer F19-R1。当前SDK20发行/独立测试、真实旧新Core14、landing/canonical/ledger/index/archive/guards/retention仍pending；不能凭本门提前归档。完整来源/工具变体/中断边界见[合入收据](#e890-a69851c1725399a4)。



<a id="e890-final-summary"></a>

### 最终整合收据 PLAN-890:r7（2026-10-31）

- 用户最终授权：不再测试，安全rebase/合入/归档/清理。未再次构建或执行测试；周期批次亦未执行，不认证为绿。
- 独立终审T19及恢复addendum PASS；冻结producer Lang54b5d7c92/Webe144ec0/UI4da2201/Man d2b2664/Downc3464d0。
- 实际冻结验证：语言六组3954 PASS；Web221/UI-full3121/Man283+24+254+43/UI-tu866/UI-ti28 PASS，原ignore/平台条件保留。四仓正式receipt SHA256 734410720b8026390401da1fe21826654c421ca8a212d77b21e541367084df40。
- 独立20包SDK217 PASS+2原ignore；发行45/45 SHA256 cd47a550c94d0bcc96c4bb85c07683b2ec88d920e6ff8234282f3a52b02c60b4；历史Core恢复14/14 SHA256 da108f44a6944be77ffd1094134bce335a49e1d93798902389cead31ffa5ea5a；完整fixture11/11 SHA256 b5ec61e434ec0ec8fff23d49e4f65da8911a5bfa4c8c4dd3e909b01618e3e9ce。原首次失败/中断不被后续绿覆盖，细节见下方折叠原摘要。
- 合入更新：语言先整合896，再整合898退役旧测试引擎，最终rebase至11ec691dc（含6d1b10fb2既存编译修复）。176acaba9栈51项到7493e4635 range-diff全部等价；原54b5至898整合存在具名冲突适配，未冒称全等。UI rebase至537b873，原锁同步两提交被上游吸收，其余5提交正常重放。Man锁既存WIP SHA25652a3069bb97ff98e4ec5fb8c32f44bd3e92b328f39cfc56d7186c95ae354992e与备份一致，所有包/边已包含于候选，清除重复WIP后ff-only合入，未丢依赖。
- 898兼容修正：不恢复已退役tm/catalog；原八腿命令、UI基线/迁移、floors及45发行ID移入正式web_native_owners/contract；四仓与发行调用切新helper。独立静态核查确认10个helper和发行三个校验函数/负控原AST保持、13固定控制保留；配置路径用Path.joinpath规范化Windows/Unix分隔符。复审建议已落实，仅静态核查，未测试此整合基线。
- canonical：docs/specs/auto-web/project.md、repository-boundaries/web-four-repository-delivery.md、language-sdk/http-host-port.md及相关模块；已核主干具备原reviewed delta。模块plans与P890 ledger已更新；python scripts/spec-index.py exit0，65projects，原未分组项目警告保留。
- 部署观察：主检出未见四仓release目录；PATH auto.exe为auto-man/target/debug/auto.exe，139275264bytes，mtime2026-10-31T11:37:01.8459266+08:00，PID53568在12:12启动，早于最终修复，明确stale/unverified。未重建/重启用户进程，未见安装auto-web；合入不冒称生产部署。
- 证据收拢：下方收纳全部附件MD摘要/合同/复审/交接（逐文件SHA256）；原探针/日志/截图/JSON为短期材料，attachments/890整目录在归档前删除。plan-artifacts/plan-890按最新五计划保留政策另管，不永久承诺原文件可重放。

| 仓库 | 实际已落主分支交付提交 |
|---|---|
| auto-lang | `65ed69dab35c7fda2a9c5ab1e98340dc18a6711e` |
| auto-web | `f29c7d1439e2dd202860dfd3ba300da07354f486` |
| auto-ui | `d29e36c55c8bb74e672a44ce6aa654c10318da7f` |
| auto-man | `c4e5aca3d9adaba0f37ad2c035956426458e27f1` |

归档时间：2026-10-31T07:27:08.343414+00:00；状态archived，completion_kind:delivered。cleanup待实际完成后追加。

### 关闭时收拢的原始摘要记录

以下为各历史阶段在原输入下的原摘要，后来的最终收据覆盖其pending状态，绝不覆盖首次失败事实。

<a id="e890-5e609c087c02fb20"></a>

<details>
<summary>merge\entry.md · SHA256 0ac3646f2bf40ef1f2d45638d6b22e60a68ab6f97cb22941b1dbd461b1d7d40b</summary>

#### PLAN-890 r6 merge entry

Reviewed11/12, independent attempt3PASS at language29cf44480 and fixed Webe012/UI9aad/ManCB4/Downc346; current language4dbe6f667 is debt-only, source03f08... verified unchanged. Main ae31f23cc, unrelated language/UI docs WIP preserved; no main code WIP. Previous goal turn was progress: independent final acceptance and routing committed.

T-12 ownership: reconcile/linearize candidate history, prepare only reviewed canonical delta plus user-required module/ledger/index, verify unchanged implementation and Spec scope, inspect affected tm module closure and execute real required four-owner union once (reuse its actual language union to cover the affected selection), refresh genuine current-cohort release/history proof, then land each modified repository, verify canonical readback, archive, guard-clean and perform user14-day budget/retention/accounting plus actual due check. Nothing is yet marked passed.

Main receipt currently last_covered_plan_id895 with a user waiver scoped to895 before any execution; this is not a general890 waiver. Exact due determination will use actual landing time and standing conditions. No new scope, remote push or publication. No implementation agents. No deletion/reclamation yet.


</details>

<a id="e890-0d80b9b7314a384d"></a>

<details>
<summary>merge\gate-needs-fix.md · SHA256 9f665665262f450a173cc395e94dcb907197c753851440506e906052df215cf6</summary>

#### PLAN-890 r6 frozen merge gate — needs_fix (2026-10-31)

The T-12 gate is NOT_PASSED. No modified repository has landed, no Plan archive or worktree cleanup occurred. Next stage: Work, repair cycle 3/3 (two completed repair cycles; final independent review attempt 3 remains a historical PASS on its recorded inputs). No fourth repair cycle is authorized by the goal workflow.

Frozen candidate: Lang `3c44ec3264f7f536d5bc57a1f6d8b1f9764f2f70`, Web `de1783f1c94dbc4d656958faca74ab61a69831ba`, UI `d347dd3d888545f6f07fe67b7f4c7f1be28741c8`, Man `b9f0c0259ae8b167a635c978dab551e382dbc894`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. Five trees clean after original report restoration. Source identity and union contract are in `frozen-union-contract.json`; raw evidence is local only.

Actual command (Lang authoritative worktree): `cargo run --quiet --manifest-path scripts/full-regression-driver/Cargo.toml -- tf --batch --with-web --frozen-trees --output D:/autostack/auto-lang/docs/plans/attachments/890/merge/four-owner-final-retry-1`. Frozen five-owner root environment and G:/autostack-build/plan890-frozen-target are retained in prepared checkpoint evidence. This consumes the same 1220-leaf / 7368-obligation language union as the actual public affected selector; no duplicate legacy language gate was stacked.

Partial results: 60 module records, 25 PASS, 4 FAIL, 31 original-condition NOT_APPLICABLE, zero awaiting-human. `executed_modules=29`, `complete=false`, `passed=false`, `gate_passed=false`. Remaining 1160 modules have no completed result; the next may have started when stopped. External owner legs and final current-cohort distribution/history gates were not completed. No root receipt.json exists. Outer session39093 ended exit1 following deliberate termination of verified own PID40192 tree, after mandatory reds were established; this is not an observed whole-gate natural completion. Other owners' processes were not terminated.

| Finding | Actual evidence and required closing oracle |
|---|---|
| 890-M1 — worker preparation | `language/auto-lang/a2r-hir-892/direct/001`: six tests, one PASS (missing-worker negative), five FAIL. Three credential/certify tests require absent `experimental/auto-a2x/target/release/a2x-worker.exe`; tampered-golden expected Fail receives infra Error; first-case-green reports five missing-worker errors. Trace actual source/preparation contract; prepare the unchanged worker or repair the proper managed preparation owner, then execute the complete six-test leaf and its original negative controls. Do not suppress the leaf or weaken the error/failure distinction. |
| 890-M1b — descendant accounting | Same leaf's workload exit100; module exit124 is specifically meter error `descendants live after workload exit`, not a measured test timeout. Meter identifies MSVC `vctip.exe` PID41968; cleanup_complete=true, remaining_active=0. Diagnose source/runner boundary and retain resource/exit distinctions; do not misattribute 124 solely to missing worker or waive resource accounting. |
| 890-M2 — generated Rust namespace | `language/auto-lang/aavm-runner/t3-ta-taa/001` and `/002`, exit100: original `test_aavm2_compile_corpus` / `test_aavm2_compile_use_corpus` generated Rust cannot resolve `auto_lang::a2r_std::fs::read_text_range` (E0433). Trace actual generated namespace/dependency owner before edits; complete both original leaves with unchanged inputs/assertions and relevant negative controls. |
| 890-M3 — reference evaluator | `language/auto-lang/aavm2-a2r/t3-ta-taa/001`, exit100: `test_aavm2_goldens_check` fails with RuntimeError `STDASSEMBLY.INVALID_BYTECODE: 60`; sibling main-dump test PASS. Trace real reference/native/host assembly path, then replay the complete original leaf. No golden or assertion reduction. |

M2/M3 resemble the three original AAVM failures recorded in earlier stages/P763-R1; historical attribution does not waive this mandatory gate. These are blocking closure findings, not nonblocking debt. This merge round changes no product implementation. Reopen T-08/T-10/T-11 for bounded diagnosis, repair and affected independent re-review. Prior R1/R2 closure remains historical and is invalidated only where actual changes affect its inputs. Current source export is preparation (19 packages/candidate:false), not final distribution certification.

All ten tracked reports moved by the active producer were restored byte-for-byte from original snapshots following cancellation; verification receipt `four-owner-final-retry-1/language-module-reports/preservation-after-cancel.json`. Raw backups remain local. Initial separate own-run cancellation before test bodies is recorded in `early-gate-cancellation.json` and must not be confused with these actual test failures. No cache deletion or retention apply was performed in this round. User's 14-day evidence policy remains authoritative over conflicting installed retention defaults.

Next Work must first establish bounded diagnosis and discriminating oracles for M1/M1b/M2/M3 before repair commits, preserve thresholds/conditions/test identities, fix the approved canonical overview navigation noted in `post-gate-document-checks.md`, run affected complete leaves/consumers as justified, and hand changed inputs to independent review. Fresh invalidated T-12 gates follow a PASS; no immediate full-catalog retry before diagnosis.

##### Local evidence identity

- `four-owner-final-retry-1/language-modules/execution/results.json`: SHA256 `0f21abcace0c61861cb760683f08988da6650c32301901572022d7c52afac30f`.
- `four-owner-final-retry-1/language-modules/execution/30-run.log`: SHA256 `d75833d96346914eccecbb7fa6dc8d20d8ea3cecdf4354289521c250aa1b91db`.
- `four-owner-final-retry-1/language-modules/execution/46-run.log`: SHA256 `5949f90954ff4f0498acae0834f16da5e84ce34c372b1594aa780d5e6f22d380`.
- `four-owner-final-retry-1/language-modules/execution/47-run.log`: SHA256 `103318f0dbe465cabd4ac1b1b89b549349e3d4e3803269ff05bb61c8cdba7aa2`.
- `four-owner-final-retry-1/language-modules/execution/49-run.log`: SHA256 `7aa300c5b35d87a4f07e26f75860b2a118a270e7e66709e8fa5069afbcb48c96`.
- `four-owner-final-retry-1/language-module-reports/preservation-after-cancel.json`: SHA256 `5ee6aee2f4a8006022a710f4cc8f7939d853e65dd45d632fb3e303c5665790c3`.
- `frozen-union-contract.json`: SHA256 `c26dae873cad1c89485001237e09b0973d0aeb66b66594a88933bf865b1b7628`.


</details>

<a id="e890-6bb2492a2655d861"></a>

<details>
<summary>merge\post-gate-document-checks.md · SHA256 f26f6a67dcb07d8118e49dc614370ce6efc674d72fc9821646be5264e683ac2b</summary>

#### Post-gate documentation readback (PLAN-890)

Keep all frozen source files unchanged while session39093 executes. Before landing, correct the global overview navigation placement: the new independent auto-web row belongs in the UI/Web ecosystem table, not the crates/workspace-members table; add the restricted auto-generation-core carrier row to the language crate map using the already reviewed auto-api-gen ownership contract pointer. This is navigation/current-state deposition within SD01/04/08, no implementation or test eligibility change.

After actual gate results exist, replace preparation/pending status in the Web delivery canonical page and owned compatibility report pointers with the actual recorded outcome; do not anticipate a pass. Read actual current crate/source manifests before changing historical numeric closure text. Preserve the central archive link as planned until actual archive. Record every derived docs/projection-only descendant and its exact path/hash delta from the executed freeze; never relabel the original execution source identity or producer commits.

Any product/config/assertion/input behavior change must return through work/affected re-review; these document corrections do not permit bypassing a failed gate or manufacturing fresh execution.


</details>

<a id="e890-a115ccd89e8ef3a2"></a>

<details>
<summary>merge\prepared.md · SHA256 0cd4a7c25ce6a7f64a0f40dc9718457863c5c0b8ea330786c35bd1b8d14af727</summary>

#### PLAN-890:r6 prepared checkpoint (not landed)

Independent final pass9b642a8a... and frozen SD40791c32... checked. Language development history rebased onto ae31f23cc:4dbe6f667→e1600601f (52 replay candidates from55 unique commits, upstream documentation duplicates dropped). Documentation conflicts retain current shared main Plan; original contract snapshots with equal main/candidate Git blobs retain main. rerere recipe union was verified equal to all47 approved final recipes. No Rust/product conflict resolution was needed. Actual final non-plan Git blobs were equal to pre-rebase candidate; range-diff and source-equivalence receipts retained. Physical checkout line endings changed the raw source fingerprint; the old03f08 identity is not claimed for the rebased delivery. Original Git source/resource contracts independently pass.

Canonical descendants prepared: Lang f5c7b47f4, Webde1783f, UId347dd3, Man b9f0c02. Reviewed SD01..09 applied to the twelve original/new language targets, restricted Web five-page canonical set, consumer integration pointers and module plans/overview. Language spec-index grouping adds only the new Web navigation source; actual index generator exit0,65projects, existing unrelated ungrouped warnings retained. User-required compatibility ledger upserts preserve all old unrelated items; new external ledgers contain six sections and only five PLAN890 pointers each, no invented canonical goals. Delivery report items remainpending.

Scoped preparation check:41changed/new Markdown files, zero invalid added links; central archive links are planned targets and will be verified against actual archived state. Existing Man project card remains its pre-existing437lines with current-scope edits only; no unrelated knowledge migration. Compiler card remains150lines through paragraph wrapping only, existing content preserved. All four preparation commits clean; master user docs WIP untouched.

Final affected selection and actual four-owner/current release/history gates remain NOT_PASSED. Nothing landed/archived/cleaned. Raw JSON/path/hash proofs local under this directory; only concise MD enters Git.


</details>

<a id="e890-a69851c1725399a4"></a>

<details>
<summary>merge\receipt-r7.md · SHA256 4f7fe35a7f7873bf033f0e7dd8da414dcee1ec8a8f3ae5717ea40929a242ce31</summary>

#### PLAN-890:r7 consolidation receipt

Stage: merge; outcome: pending. This document records actual checkpoints only. Independent T18 final affected review passed on Lang54b5d7c92f0244faecde929353b7e292ff6b3579/Webe144ec0d9febe74f8cf734744d91ade47fe57bcf/UI4da2201fa46bd8a88bc5c2d661b21433b7ac84b5/Man43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6/Downc3464d0. Review SHA2564b50b42a80d3d1aca15d70b3a5b26e4af3b556fd0d0f6ded6fc050763125f098. Main Lang base3fa75dd9c. No remote publication.

- Prepared: current canonical changes independently reviewed; runtime schema guard13/13 and real Git staged/untracked negative controls passed. Historical failed full run remains failed, not relabeled.
- Frozen final four-owner gate: currently running scripts/web-four-repository.py --out docs/plans/attachments/890/merge/final-896-2/four-owner, from the actual Lang890 worktree. Explicit language groups integration, auto-generation-core, auto-api-gen, auto-media-core, a2r-std, auto-test-case; eight native owner legs web, ui-full, man-t, man-tcore, man-tc, man-ti, ui-tu, ui-ti. NOT_PASSED until complete actual receipt validation.
- Fresh source export: plan-artifacts/plan-890/final-896/source-54b5d7c92,20 packages,candidate=false; source_sha2568c55a2d281e9d96355799d958e2793fba0048091d16b49cd21fa8605065359db. Current independent SDK tests/distribution/legacy14 still NOT_PASSED.
- Landing, canonical readback/ledger/index, archival/count/link check, guarded cleanup, latest-five maintenance and batch due check are pending.

Deployment observation (before landing): main repositories auto-lang/auto-web/auto-ui/auto-man have no target/release directory. No current production release artifact is certified by local debug/SDK validation. No user process has been restarted; this delivery does not claim an installed production upgrade. Final closeout must record current installed consumer artifacts or their explicit missing/stale status.

Installed consumer observation at2026-10-31T13:10+08:00: PATH resolves auto.exe to D:/autostack/auto-man/target/debug/auto.exe,139275264 bytes,mtime2026-10-31T11:37:01.8459266+08:00. Active PID53568 started12:12:05.481593+08:00 from that same path. It predates final Lang/UI repair commits12:40+08:00 and is explicitly stale/unverified for this delivery. No installed auto-web was resolved; no release directory for any of the four mains. Do not substitute this old running consumer for the actual frozen worktree/SDK gate products. Process left untouched.

Frozen language gate completed PASS: final-896-2/four-owner/language/results.json SHA256006e4519c927bb69a68ee4c71ace0147b9b9ae702e7cc9d456ec8c4d53b646fc,source_state9154eb92b9ba992a2d7e3eeabaea8bb4a209c25d9124511c3874617cbf34941e,585.596s whole command. Actual integration3684/3684 (750 original excluded),generation16/16,API3/3,media54/54,a2r-std104/104,TestCase93/93,total3954. Integration preparation2.341s/run451.918s; generation16.729/3.201; API12.590/1.592; media2.560/45.413; std3.544/9.026; TestCase1.640/2.262. Every execution exit0/meter.error=null. Compiler warnings in raw outputs are existing baseline diagnostics, not a claimed zero-warning repository. External bootstrap/legs and complete source admission still pending; this language PASS is not the full four-owner result.

External progress (not yet full admission): Web nextest native command exit0,meter.error=null,400.929s,221ok/2original ignored. Includes original217 library and4HTTP runtime/generation targets. UI-full exit0,meter.error=null,66.855s,3121ok/18original ignored; preparation163.940s separate. Man four legs,UI-tu/UI-ti and final fresh-denominator/source/child admission remain pending. Actual background during Web execution included unrelated UI-016 Rust compilation; no foreign process was stopped and timings are observed wall time, not a formal performance claim.

Frozen four-owner gate now actual PASS: final-896-4/four-owner/receipt.json SHA256734410720b8026390401da1fe21826654c421ca8a212d77b21e541367084df40. Actual normal execution exec-8ebec7ba-f381-43e4-8c63-2d0cbabd09fd completed/exit0 with explicit final pass after six language and eight owner fresh source-denominator/child/terminal checks. Original final-896-2 FAIL and interrupted final-896-3 remain unchanged. Language3954/Web221/UI-full3121 reuse exact unchanged source/dependency/config/condition inputs with full environment admission; new Man283/24/254/43,UI-tu866,UI-ti28 actual pass, original exclusions retained. No global/ignored/environment-wide claim.

Independent recovery addendum t19-t12-recovery-addendum-r7.md SHAe9dccc506f1430e8641b8b3e57e245451f6fc048d7328890e8573ce646ffdc47 accepts actual normal completion and closes F19-R1. Actual executed local script a08c remains preserved as complete_interrupted_four_owner.executed-v1.py; finalizer-only safety descendant12b268077ed69ea0292d030ce9b2ae97a79bbf5aecdf82285f6d4172f953ab66 is verified by4 isolated exception controls, no body/admission rerun. Soft interrupts now persist fail, not the temporary consumer pass.

Interruption is unknown, not attributed to product or another actor. UI-ti preparation rebuilt the derived UI tool9a6→5c5 at unchanged source. Old rows keep their actual9a6 input history; new UI-ti actual28 binds5c5 and freezes it during workload. Exact final executed UItool/te-child preserved in plan-artifacts/plan-890/final-896/final-execution-tools; manifest SHA56ed5e0eadedb410ab67db5e319913ae32c0e0bea75909f4374b48487fee90d4. Later cache rebuilds do not claim old rows tested new executable bytes.

Current legacy14 now started from genuine Git old Lang674871e2/Man e6cc and current Lang54b5/Webe144/Man d2b. Dedicated TEMP/TMP is plan-artifacts/plan-890/final-896/tmp-legacy; target is owned build-legacy-d2b2664. This fresh SDK/legacy environment is separate from the four-owner environment and cannot rewrite its provenance. Legacy/distribution current results remain pending until actual completion.

Legacy actual14/14 PASS, receipt SHA256da108f44a6944be77ffd1094134bce335a49e1d93798902389cead31ffa5ea5a. Current producer pins54b5/e144/Man d2b; true old Lang674871e2/Man e6cc; old UI pin is recorded, not an old Full/UI execution claim. Fresh independent20-package SDK/distribution45 now running, TEMP/TMP tmp-distribution, never skip-independent-tests.

Main preflight: Lang advanced3fa→297a by documentation/design/planning-only commits (Design52,898/899,Reader renumber); no runtime/code/lock diff. UI/Man mains each carry17-package upstream896/897 Cargo lock drift. Semantic package/version/checksum/edge comparison proves every delta already in reviewed candidates, with no removed/replaced edges or checksums. Exact bytes and binary patches protected under merge/preexisting-main-locks; no restore yet. Other authored main docs remain unrelated and preserved.


</details>

<a id="e890-f0fee9ee4a1d7f26"></a>

<details>
<summary>review\t11-final-review-r6.md · SHA256 6ec2e2666d46504c9963cf09bd8625f1e4b162315f4ae8e472c62c889f39377b</summary>

#### PLAN-890 r6 independent final review — needs_fix

stage: review | review_scope: final | outcome: needs_fix | plan_id: PLAN-890 | plan_revision: 6

Actual independent context: `/root/plan890_final_review`. This is the separately assigned review context, operating on the implementation's same five authoritative sibling trees. No second checkout, implementation agent, code repair, canonical Spec/ledger write, staging, commit, merge or computer control occurred. Evidence writes are local to this main-checkout review directory. The parent owns Plan/status/debt bookkeeping after this receipt.

Two blocking findings remain: **890-R1**, acceptance receipt admission does not enforce execution proof; **890-R2**, the public Web CLI loses its explicit launch configuration across Core's file-execution thread. Passing historical invocations remain evidence for their actual inputs, not proof of the failed invariants.

##### Frozen baseline and instruction identity

All five trees were clean both at entry and final readback. `git worktree list --porcelain`, `rev-parse HEAD`, branch, merge-base and status were inspected in each exact tree. Root: `D:/autostack/.wt/lang-890/`.

| Repository | Branch | Reviewed HEAD | Diff base |
|---|---|---|---|
| auto-lang | plan-890-dev | e1ece5d45138040bcb67c4ecf5060cca53099019 | 6e48fe11c0be28159a950413a14f7964b3659389 |
| auto-web | codex/plan-890-dev | 9ddb7fe5bb1886166073b5805c38895f3da195ff | 8ff49669a819e39a166d52f69781bba902e4db4a |
| auto-ui | codex/plan-890-dev | 9aad12ccbc5f01d5241eb95f657666a943667ff2 | 69690dfb8e0b691a9af8d9e1eacc240511d2e83e |
| auto-man | codex/plan-890-dev | cb4ab3dd5d411314cce10b129eeb3b433a11c466 | e6cc06fbc496f366edde7929abfd49254ec13647 |
| auto-down | detached | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | same |

Language's implementation source commit is `ede598998566ea890ff67170bb9744f3bc537a58`; following handoff commits do not alter product inputs. `t11-final-baseline.json` records full binary-diff hashes, actual locks, Spec hashes, Plan/proposal identity and local replay file hashes. It is the machine-readable companion to this receipt.

Main Plan SHA-256: `c594f2e148876b13cb3c62757f642329129a9ae4bd33474f0a3a8803547c3c4b`.
Frozen proposed delta (`../work/t10-spec-delta-proposal.md`) SHA-256: `b2a47e3a5891e79b25bcf842462bdbc33aa222117c373355c9b192aed47e7e9e`.

Fully read instruction sources: main `AGENTS.md`, `C:/Users/zhaop/.codex/skills/auto-plan-review/SKILL.md` (SHA-256 `36d7987f095a5766b6a62d7d8c6d5a6c5f1f48514b0606608832dd86cf92c94e`) and the new skill's `references/verification-contract.md` (`73ea4ef36cd8b9010f17f648201a685338272d85d1eef5c8862b0f1ddc14e0bf`). Repository/user rules govern merge deposition. No retention operation was performed; historical handoff retention prose must not substitute for the user's applicable current rule.

Independent `source_identity` readback confirms **every repository input hash equals T-10/catalog**: language `9221ff9d...`, UI `4f45cffe...`, Down `c0cf5cca...`, Man `26981967...`, Web `48181f86...`. This context's environment hash is `aa2ef3ba911c3e023806664da55440f837b78389fd926c8af27ea349e1cc8c2f`, whereas discovery used `a92a86a788c826d4bc7bd0e7b02da4cce96952501b3df5e775a492bb884ff220`. Thus the combined identity here is `7ca72e87...`, **not** discovery's `0bb28657...`; source bytes match, effective environments are not asserted identical. The final gate must freeze its real environment. Counts remain 7,329 obligations / 1,213 leaves; listing is not execution.

During review, main advanced to `0a54a3b03` with delivered PLAN-894 changes. None of the candidate trees changed. Synchronizing that official baseline and refreshing affected source/catalog/consumer identities is later work before landing, not an additional defect in this frozen candidate and not covered by this review.

##### Findings and minimal closing scope

###### 890-R1 — HIGH: incomplete execution evidence is admitted as complete delivery

Existing rule: AC-08/09/12, V-11/12/13/17, T-08/T-09/T-10; T-02 r3's receipt contract requires exact IDs, list/run command, cwd/package/features, output path/hash and actual execution. Missing runs, filtering after listing and unexecuted required checks must reject. This is an original requirement failure, not a new anti-forgery/security-signature requirement.

Established cause: `scripts/test-governance/web_delivery.py:197` checks only required leg names, `run_exit == 0` and truthy `selected` after checking outer source identities. It never validates the actual execution records. The public `scripts/web-repository-fixture.py` calls this validator and compares current identities, but performs no additional execution proof reconciliation.

| Subcase / real boundary | Actual mutation and observed result | Required result |
|---|---|---|
| A, public complete fixture | `t11-forged-four-owner.py` constructs **current real five-repository identities** and all required leg names, with each result containing only `{leg, run_exit:0, selected:1}`. No required test legs execute. With the unchanged genuine T-09 distribution receipt, actual public `run --require-complete` returns **0, pass, 11 passed / 0 failed / 0 pending**. | Reject missing IDs/commands/cwd/results/log hashes; never certify complete execution. |
| B, real-catalog Web collector | `t11-short-execution.py` first admits a complete test-only protocol control using the actual loaded catalog. It then changes unconditional `web-unit/auto-web/api-gen/direct/001` from **20 applicable IDs to 19** and changes its summary log to 19/19, keeping the declared/selected 20 obligations. `collect_web_legs` still returns all four complete legs. Omitted ID: `web-unit|auto-web|api_gen::tests::test_inline_meta_body_binds_extractors_and_wraps_early_returns`. | Reject unexplained loss of an unconditional obligation. Original ignored/milestone exclusions must remain legitimate. |
| C, distribution validator | `t11-distribution-missing-command.py` validates the genuine original 45-row receipt, then independently deletes `command`, `exit`, `expected_exit`, `log`, `log_sha256` from each of `external-locked-build`, `fresh-generated-build`, `missing-resource-rejected`, `failed-swap`, `successful-upgrade`. Every mutated receipt still returns True. | Required command-driven steps cannot become optional `{id,status:PASS}` banners. |

B's established cause is comparing summary counts only with self-declared `row['applicable']` (`web_delivery.py:176`), without reconciling actual applicable IDs and exclusions to the source-bound obligations. C's cause is `if 'command' in row` (`:48`): a missing required command skips its entire proof check. The genuine producer writes these fields through `command(...)`; legitimate runtime/detail/reuse rows have different schemas and must remain accepted.

Replays, from the authoritative language tree:

```text
python D:/autostack/auto-lang/docs/plans/attachments/890/review/t11-forged-four-owner.py
python D:/autostack/auto-lang/docs/plans/attachments/890/review/t11-short-execution.py
python D:/autostack/auto-lang/docs/plans/attachments/890/review/t11-distribution-missing-command.py
```

The first two review scripts intentionally return 1 when the forbidden acceptance occurs; the last prints each accepted mutation. See same-stem `.log` files, `t11-no-executions-gate.json`, `t11-public-forged-receipt-result.json`, `t11-collector-control/`, and the five `t11-distribution-*.json` mutations. These synthetic negative-control receipts are **not** claimed as executed tests. Subcase A reaches the actual public certification consumer.

Closing oracle: permanent tests must prove all three negative families reject through their real consumers; the actual original distribution45 receipt remains accepted, the complete 20-ID collector control remains accepted, and all existing 10 ownership controls remain valid. Check sibling omission/duplicate/unexplained-exclusion/stale-log cases under the same invariant, including required legacy command-driven build/install evidence where applicable. Do not add signatures or new support promises. Repair is bounded to receipt admission/producer metadata as necessary; no global test suite is attached to this finding. Future real four-owner execution stays merge-owned.

###### 890-R2 — HIGH: explicit Web CLI launch configuration is lost across the file-entry thread

Existing rule: AC-03/06/09/12, V-03/06/09/12/17, T-05/T-09/T-10 and SD-03/05/08 require the explicit invocation snapshot to travel across worker threads and preserve configured listening/policy. Configured service execution must not silently use legacy environment/default settings.

Actual path: Web `crates/auto-web/src/main.rs:18` installs `with_http_service_launch(launch, ...)` and calls `auto_lang_core::run_file`. Core `script_entry.rs:25` → `run_file_with_args` → `run_with_path:75` → `.spawn(move || ...)` at `:80` does **not** copy/bind the caller's TLS launch. The new thread's invocation has no launch; `WebHttpServiceHost::serve_activated` consequently takes its legacy `AUTO_HTTP_PORT` / `0.0.0.0` branch and never applies the parsed configuration. The separately repaired `entry/run_bridge.rs` does propagate launch, and Man/UI's named worker paths have their own explicit propagation; their passes do not cover this file entry.

Replay: `python D:/autostack/auto-lang/docs/plans/attachments/890/review/t11-cli-launch-loss.py` (intentionally exit 1 on the defect). This drives the real published local `auto-web.exe` from T-09's relocated B/tool, not a source wrapper. Binary SHA-256 `e50b844ea86a063282a9031b021e17f718c44acc588aed5b5ff1c9d141c1623a` equals the install's runtime producer. The script asserts byte equality between installed SDK and current candidate for Web main/host, Core script_entry/lib/launch; exact SHA-256 values and source pins are in `t11-cli-launch-loss-result.json`. No candidate-only fix is being overlooked.

| Case | Expected | Observed wire / exit |
|---|---|---|
| No-config legitimate control | Legacy env port 5453 | Exact unique nonce at 5453; shutdown202, exit0, port released. |
| Valid development config | Loopback declared5457, different env5458 | Exact unique nonce at **5458**; diagnostics **0.0.0.0:5458**; declared5457 unused. Shutdown202, exit0, port released. |
| Valid proxy_service, `allowed_hosts=[only.example]`, `auth.responsibility=app` | Declared5462; untrusted Host rejected400 | Exact nonce at **env5463**, diagnostics **0.0.0.0:5463**; `Host: untrusted.example` returns **200**, not400. Shutdown202, exit0, port released. |

Ports are selected afresh on replay. Exact random response tokens distinguish these owned services from any unrelated listener. Only owned children were terminated; all final cases shut down normally and their ports rebound. This is a configuration/policy failure, not a startup-performance finding.

Why earlier controls missed it: `auto-web/tests/http_runtime.rs:39` and distribution `runtime(...)` deliberately set `AUTO_HTTP_PORT` equal to their configuration's port. Exact JSON/shutdown passes in that setup even when launch is lost. The existing configuration/isolation probes exercise other entrypoints. A parse-only bad-config check also cannot detect a correctly parsed but discarded config.

Closing oracle: add a permanent real `auto-web run --config` case with **different** declared and legacy environment ports, exact response at the declared port, loopback bind, effective profile/config identity and actual Host policy rejection. Keep no-config legacy behavior and ordinary non-service file execution accepted. Exercise `run_file` / `run_file_with_args` and representative existing launch entrypoints to close the shared thread-handoff invariant; do not just mirror the env port to make the original test pass. Rebuild/retest affected SDK/CLI/distribution inputs and bind the repaired source/artifact identities. No whole runtime/catalog rerun is required merely for review; final unified gates follow the repaired freeze.

##### Acceptance evidence map

`pass` below is bounded acceptance on the named frozen inputs and inspected runtime evidence, not a whole-environment or future merged-baseline certificate. `partial` retains applicable evidence while a listed invariant remains open.

| AC | Task(s) | Result | Code / artifacts and reviewed evidence |
|---|---|---|---|
| AC-01 | 01/03 | pass | Inventory/provenance and bootstrap history, actual five Git registrations; original source/resource validation executed successfully by public fixture; original 885 user-confirmed history boundary preserved. |
| AC-02 | 02/04 | pass | Core neutral `HttpServiceHost`, Rc-owned !Send future, messages/metadata/request hooks; Web contract and Cargo manifests; actual public fixture metadata closure and source ownership check passed. One Core/contract/shared-generation identity, no forbidden Web→UI/Man/Tauri or Core→Web implementation edge. SDK source inventory inspected. |
| AC-03 | 02/04/05/07 | fail | New runtime owner and explicit installer exist; old Core domain implementations removed; Man/UI composition and owned transport evidence retained. **R2** breaks actual Web CLI/file-entry configured service assembly. |
| AC-04 | 06 | pass | Web HTTP `api_gen` / Axum/TypeScript targets; Core Tauri and restricted shared-generation single source, thin old facade. Original885 fixture/resource bytes and original generator/assertion migration; actual cold-generated Rust/wire receipts inspected and source ownership validated. |
| AC-05 | 07 | pass | Man service/runtime language bridge and UI worker/proxy/Vue/Rust paths explicitly consume Web; typed launch wiring inspected. T07 actual VM/Rust CLI, proxy/worker/generation evidence plus T10 actual dual UI retained; daemon/non-HTTP path preserved. Independent Web CLI R2 is separately failed. |
| AC-06 | 05 | fail | Original strict config and wire matrices/source remain; VM/Rust file/upload/SSE/error, parser/security-budget evidence retained. **R2** proves a current public configured path bypasses both listen and Host policy. No new platform promise inferred. |
| AC-07 | 02/05 | pass | Owned request/body DTOs, scope permit through body terminal, late-resource registration and finalization, connection lifetime Drop, network/logger joins inspected; actual file/SSE/RST/upload/isolation/drain receipts distinguish resource termination from flags. Final R2 replay also confirms owned shutdown/exit/port release. No full lifecycle certification outside declared profile. |
| AC-08 | 08 | fail | Original117 migration identities, 696 history rows/116 IDs, 17 resources and media10 source assertions conserved; 10 existing ownership controls independently pass. **R1A/B** show actual certification can still admit no execution or missing unconditional execution. |
| AC-09 | 09 | fail | Real source19 SDK, original217/2, fresh/locked/relocation/distribution45 and history14 receipts remain applicable to actual invocations; immutable installer artifact and 6 independently passing transaction controls. **R1C** weakens proof admission; **R2** breaks independently shipped CLI configuration. |
| AC-10 | 10 | pass | Fixed b42a660 Git consumer314 unchanged; original driver only binds roots/tool/archive identity, does not edit scenario assertions. Actual API/cold-build and Vue15/native22 raw outcomes read, restart/conflict/content checks examined. Separate T05/T07 service wire evidence retained; dual UI is not inferred dual service. Parent's file/hash readback corroborates16 raw artifacts. |
| AC-11 | 04/06/08/09 | pass | Old runtime/HTTP target originals removed; named Core language bridges and Tauri/shared facade have long-term owners. Public original-source/resource checks pass, immutable Man tool provenance and older/new Core archive/runtime recovery evidence inspected. Pure moves are not net deletion. |
| AC-12 | 10 | fail | Changed-warning comparison baseline4/candidate4/new0 and inherited TODO provenance inspected; candidate whitespace checked. **R1/R2 are mandatory work gaps**, not debt. Delta remains proposal and needs truthfulness correction after repair. |
| AC-13 | 11 | fail | Independent context, frozen inputs, all AC/V/SD and high-risk areas examined; final pass cannot be issued with R1/R2. |
| AC-14 | 12 | partial | Deliberately unexecuted merge-owned final gates/landings/deposition/archive/guard cleanup. No U890 or delivered status. |

| V | Result | Evidence / distinguishing boundary |
|---|---|---|
| V-01 | pass | Actual Git identity/cleanliness, provenance/resource/source validators; preserved original receipts. |
| V-02 | pass | Actual metadata closure and package/source checks in public fixture; neutral contract/DTO/trait and shared carrier inspected. |
| V-03 | partial | Original plain/missing/custom/override/isolation evidence and source selection retained; R2 fails configured file entry. |
| V-04 | pass | Owned upload stream/message conversion; VM !Send remains Rc-local; body/scope/resource branches and original actual wire/terminal assertions inspected. |
| V-05 | pass | HTTP generator/885 counterpart and input/borrow/runtime controls preserved; actual generated build/results and original source fixtures. |
| V-06 | partial | Existing Man/UI commands and actual receipts pass; public standalone Web CLI config fails R2. |
| V-07 | pass | Tauri non-HTTP target, single SidecarDecl/shared_workspace/source owner, real retained consumer fixtures. |
| V-08 | pass | Original independent VM/Rust wire/JSON/SSE/file/upload matrices and actual result oracles retained; does not imply all configured entrypoints pass. |
| V-09 | fail | Strict config parser remains strict, but valid configuration/policy lost at actual CLI; R2 authorized/untrusted Host contrast. |
| V-10 | pass | Owned scopes/connection cancellation and network/logger cleanup; original terminal/resource assertions plus independent normal shutdown/rebind. |
| V-11 | fail | R1 public omitted-execution receipt and real-catalog20→19 counterexamples. Existing10 tests pass but miss invariant. |
| V-12 | partial | Real fixed standalone SDK/distribution valid for actual inputs; R2 independently shipped CLI and R1 proof admission remain. |
| V-13 | partial | Actual6 transaction controls independently pass and real45/history14 receipt validates; R1 missing required command evidence still admitted. |
| V-14 | pass | Actual fixed Blog cold compile/API/restart source and result identity. |
| V-15 | pass | Actual original Vue15/native22; ordinary root/tool/resource materialization scope, native owned close/reopen, unchanged314 original files. |
| V-16 | pass | Source/consumer migration and original resource validation, old domain exits, thin adapters, valid old/new Core recovery. |
| V-17 | fail | No new warning/debug claim contradicted; actual in-contract R1/R2 omissions prevent completion. |
| V-18 | partial | Independent final review complete with needs_fix; final gate/merge/deposition/cleanup remain unexecuted. |

##### Risk coverage and independent executions

Inspected the full declared high-risk matrix by responsibility: provider selection/override/missing/active owner, compiler metadata versus invocation settings, local VM versus Send network messages, policy admission before body/FS, queue/stream/file/upload terminal ownership, network/logger/reader cleanup, separate Axum tracks, HTTP versus IPC/shared generation, installed source/hash/version selection, transaction prepare/verify/swap/post-swap recovery, migrated tests/resources/conditional exclusions, trusted receipt consumption, fixed Blog dual UI and source identity. Counterexamples independently chosen here cover trusted receipts and a real worker-boundary configuration loss; no first-defect early termination.

Independent commands completed:

- `python -m unittest discover -s scripts/tests -p test_web_test_ownership.py -v` from language tree: **10 passed**, 0.978s.
- `python scripts/test_install_transaction.py` from Man tree: **6 passed**, 0.151s; live/unknown staging owners retained as designed.
- R1A real public consumer: **forbidden pass**, 11/0/0 with zero required leg executions.
- R1B actual-catalog collector control: complete valid protocol accepted, then unconditional20→19 mutation also incorrectly accepted.
- R1C genuine original distribution45 accepted; five one-at-a-time required-command proof deletions incorrectly accepted.
- R2 real immutable installed Web CLI: no-config legitimate control passes; valid configured development and proxy policy fail with exact wire assertions; every final owned process shuts down202/exit0 and releases its port.
- `git diff --check master...HEAD` language and prior candidate owned-format/whitespace receipts: no whitespace errors observed. No whole-tree warning-clean claim.

Supplemental external Rust holdout (`t11-ui-holdout.rs`) linked the actual current cached UI compiler and confirmed local/nested/closure variable emission. Initial direct-link missing Windows library was a probe setup error, corrected by its actual native import-library path. The standalone public `AuraTsContext::new` intentionally lacks Vue store's private alias configuration, so its explicit `.count` output is not a production store alias defect. The typed-local shadow observation falls under the pre-existing explicitly documented flat `int_locals` limitation and is not counted as a new in-scope snapshot repair failure. Raw output is retained, no assertion was weakened.

Initial runtime scratch probes timed out at arbitrary15/60s while waiting for the configured occupied-port failure. Their raw logs remain `t11-runtime-holdout-first.log` / `t11-runtime-holdout.log`; they are **not a timing requirement or independent product verdict**. Diagnostics exposed the unrelated default port. R2 replaced that inference with the direct different-port exact-nonce/policy oracle above. The first proxy input missed the mandatory auth responsibility, was correctly rejected, and remains `t11-cli-launch-proxy-invalid-setup.log`; final R2 uses a valid `auth.responsibility=app` config.

No unrelated full library/catalog regression, UI computer-use rerun, formal performance matrix, final affected `tm` union or periodic batch was run. Prior raw failures, original ignored/manual/platform/milestone conditions, three historical AAVM reds, arbitrary Windows long-path limitations and build-cache environment failures remain named, not overwritten with new green results.

##### Proposed Spec delta review

Read canonical main Specs for all listed module/boundary targets; exact current source hashes are in `t11-final-baseline.json`. The proposal is frozen above and no canonical text was published. Existing module/frontmatter paths are real; new local paths are correctly prospective, and Web/UI/Man cross-repository targets are not disguised as language ledger paths. Empty `touched_goals` adds no invented canonical goal ID; G1..G5 are this Plan's local goals.

| SD | Result | Decision |
|---|---|---|
| SD-01 | pass proposal | Language Web pointer/new sibling identity/0.1.0 and canonical Web owner, no duplicate normative Web body; no remote publish claim. |
| SD-02 | partial | Four-owner/source/distribution/transaction boundary is correct intent; **R1** means current wording that complete receipt certification enforces actual execution cannot yet be deposited as delivered. |
| SD-03 | partial | Neutral ownership, !Send owner and explicit installer accurate; **R2** contradicts universal invocation-thread snapshot propagation. Repair file entry, then verify wording. |
| SD-04 | pass proposal | HTTP/non-HTTP generation split, four restricted shared modules, photo owner/media10 source consistency and retained885 semantics match code. |
| SD-05 | partial | Supported tracks and strict configuration/policy contract remain correct; **R2** prevents claiming actual public entrypoints preserve these settings. No wider platform/TLS/auth product promise added. |
| SD-06 | pass proposal | Real repository/Man/UI consumer ownership and fixed source boundary match current wiring; landed cohort still must be actual merge result. |
| SD-07 | partial | Migration/conditions/purpose advice correct; **R1** prevents claiming missing/filtered execution is rejected by all certification boundaries. |
| SD-08 | partial | Five new canonical Web pages appropriate; publish delivered CLI/config and trustworthy test/distribution claims only after R1/R2 close and actual gates run. Staging docs must then be reconciled. |
| SD-09 | pass proposal | Consumer pages and bounded existing UI compiler correction fit actual owners; preserve original local names, explicit state aliases/public getters and source fingerprint. No AST/runtime protocol extension. |

Frontmatter proposed components accurately identify the affected existing and new language-side files. Merge must preserve concurrent canonical changes and perform explicit actual ledger/index/module-plans deposition required by repository rules. This review does not authorize replacing failed acceptance with softened prose.

##### Routing and still-pending closeout

Return to work **in the same trees**, retain stable finding IDs and all current raw evidence. Reopen T-08 for R1 four-owner/collector admission, T-09 for R1 distribution admission and R2 shipped Web CLI, and T-10 for affected evidence/health/freeze and Spec claims. T-05's public runtime/CLI/config portion is also affected by R2; reopen that bounded portion (or explicitly track it as the existing T-09 repair dependency), retaining unchanged transport/resource/VM-Rust wire evidence. T-02/T-04's launch requirement must be checked on the file entry during repair, without discarding their already verified neutral port/type/DAG work. Parent recomputes actual completed-task count; do not leave execution_done while repairing.

After committed repairs, re-review R1 A/B/C and R2 with legitimate controls and representative sibling paths. Reuse unaffected source-bound evidence with explicit input/config justification. Do not request another full final-review loop solely to re-prove unrelated unchanged families.

Only after acceptance passes: reconcile delivered main894 changes in the same candidate, refresh its actual catalog/source/config/consumer identities, run the frozen affected complete `cargo tm` leaf/consumer/integration union and actual required four-owner gate once under its designated owner, and any due main-only periodic batch. No old raw t/tv/tt stack is added. Required owner gates, actual cross-repository landings/recovery points, canonical Specs and required ledger/index readback, archive/counts and each mandatory guard/cleanup remain T-12-owned. There is no merge/U890 authorization from this needs_fix receipt.


</details>

<a id="e890-bb5cc7621f95d087"></a>

<details>
<summary>review\t11-frozen-spec-delta.md · SHA256 b2a47e3a5891e79b25bcf842462bdbc33aa222117c373355c9b192aed47e7e9e</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


</details>

<a id="e890-1ce952cb28cda1c4"></a>

<details>
<summary>review\t12-affected-review-r6.md · SHA256 e3f46655d363335ec055d64eee25ba94f64cfcf74bc2b7c276c422021b9a62b6</summary>

#### PLAN-890 r6 — final affected re-review, attempt 2

stage: review | review_scope: final | outcome: **needs_fix** | reviewer_context: `/root/plan890_final_review`

This filename is a follow-up review receipt, **not** execution of Plan task T-12. The original independent review is [t11-final-review-r6.md](#e890-f0fee9ee4a1d7f26). Its unaffected acceptance evidence is retained. R2 is closed on the repaired baseline. R1's original A/B/C examples are closed, but the original complete-required-ID invariant remains open for external UI/Man owner legs (**R1-D**). There is no final pass, merge authorization or U890.

The reviewer used the same five authoritative implementation trees, with implementation paused. No code, Plan, canonical Spec, ledger or index was edited; no staging, commit, cleanup, second checkout, implementation agent or GUI operation occurred. Only local review evidence was written. Parent owns status/count/debt routing.

##### Reviewed freeze

Root: `D:/autostack/.wt/lang-890/`. Git registrations, resolved trees, branch, exact HEAD and clean status were independently checked at entry and final readback.

| Repository | Branch | Reviewed HEAD |
|---|---|---|
| auto-lang | plan-890-dev | 2506549321ad8f4699089a7b0dce53d74f78b046 |
| auto-web | codex/plan-890-dev | e01232b3d449ea590a85d9458d553ab17b9456a3 |
| auto-ui | codex/plan-890-dev | 9aad12ccbc5f01d5241eb95f657666a943667ff2 |
| auto-man | codex/plan-890-dev | cb4ab3dd5d411314cce10b129eeb3b433a11c466 |
| auto-down | detached | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 |

Main bookkeeping HEAD: `670d8e4b9b80163fc093d44b9d36bf014731c1f4`. Main Plan remains revision6, execution_done10/12 as submitted; its SHA-256 is `dd46f77bca1d26374f6b6d89edb7182417f89e9ac9457d8290dbd2112cb06f81`. Parent must change the state after this needs_fix receipt.

Handoff SHA-256 independently matches `d72f61ea3d6409605dfc56ec6a92d496276c4c79fbe56c69d5e278531a6eb628`. The proposed SD is copied byte-exact to [t12-frozen-spec-delta.md](#e890-f614e876595a8318), SHA-256 **`3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172`**. No proposal content drift was observed. All twelve canonical Spec targets hashed in the first review remain byte-identical; no canonical write occurred.

Under `CARGO_TARGET_DIR=G:/autostack-build/plan890-frozen-target` and an empty `RUSTC_WRAPPER`, independent source identity is exactly **`815c5287416833e52b7656e0c07764600a53ca532b883bd13168b728abf1ce8f`**, matching the submitted catalog. Its actual effective environment hash is `a92a86a788c826d4bc7bd0e7b02da4cce96952501b3df5e775a492bb884ff220`. The current catalog contains 47 recipes / 7,367 obligations / 1,220 leaves; this is discovery, not execution of those bodies.

[t12-final-baseline.json](#e890-final-summary) records exact roots/branches, prior-review and current SHA, repair-diff hashes, locks, all canonical Spec hashes, source/environment identity and local evidence hashes. It also records an independent Git-path comparison: **no current packaged SDK/crate/owner-tool input changed** relative to immutable language `74e13580adff15a861ca46efb5d0ab0ce89c168d` / Web `e01232b3d449ea590a85d9458d553ab17b9456a3`. Later governance/docs changes are not relabeled as artifact producer commits. Official894 synchronization is present in the candidate; the earlier pending synchronization itself is closed, while final combined gates remain pending.

The review skill and verification-contract source read in attempt1 remain applicable. This is an independent affected re-review, preserving the prior full AC/V/SD coverage and checking changed invariants and representative sibling paths rather than re-running unrelated suites.

##### Finding status

###### 890-R2 — closed on this exact baseline

Core `script_entry::run_with_path` now captures `invocation_launch()` before spawning its existing execution thread and binds it around the worker's existing LocalSet. The repair does not add a Web/framework dependency, alter its stack/join contract or change the transport algorithms. The permanent Core test covers distinct snapshots through `run_file_with_args`, explicit absent launch through `run_file`, ordinary42 with a selected context, and caller restoration. Existing provider/metadata/interleaved-context tests remain intact.

The first review's actual public CLI holdout was independently replayed against `D:/autostack-build/plan890-repair1/installed-B/tool/auto-web.exe`. Binary SHA-256 **`7220b7bf635ab2057587ec1684648200051d7ea7f78c1a0005f6b855651c3640`** equals its install manifest. The replay asserts byte equality of installed/current Web main/host and Core script_entry/lib/launch.

| Actual independent case | Expected and observed |
|---|---|
| No-config legitimate control | Exact nonce at legacy env4425; shutdown202, exit0, port released. |
| Explicit development config | Declared4430, different env4431; exact nonce at **4430**. Diagnostics: **127.0.0.1:4430**, development, config hash `c2b95f4e959d2b16`. Shutdown202/exit0/rebind. |
| Valid proxy_service / allowed_hosts only.example / auth app | Declared4437, different env4438; exact nonce at **4437**; **untrusted Host400**. Loopback, proxy_service and hash `24332aada9a3cd4f` observed. Shutdown202/exit0/rebind. |

Command: `python D:/autostack/auto-lang/docs/plans/attachments/890/review/t12-cli-launch-loss.py`, exit0. Raw `.log` / `-result.json` and per-case logs use the `t12-cli-launch-` prefix. Ports are freshly selected on replay, and exact unique response tokens distinguish owned processes from unrelated listeners.

Reviewed changed permanent Web tests now keep config and environment ports different, reject a fallback listener, test Host400, inspect effective profile/hash and release the port. The submitted genuine Core6 and Web HTTP3 outputs were read and independently hash-verified. Fresh3/relocated3 results, SDK217 unique actual PASS IDs plus two original skips, and the unchanged Rust generated-runtime divergent-port sibling receipt are applicable bounded evidence. R2 is not being closed by a mirrored-port or parse-only proxy.

###### 890-R1 — original examples closed; HIGH residual R1-D remains

Original rule remains AC-08/12, V-11/17, T-08/T-10 and the T-02 r3 complete-owner contract: missing required IDs or selection below the frozen owner floor cannot certify a full owner leg. This is the same original obligation, not a requirement for signatures or arbitrary tamper-proof logs.

What is now correct:

- A: actual public fixture rejects the original no-execution `{leg,run_exit:0,selected:1}` receipt. Permanent14 controls independently pass, including this real public-consumer negative.
- B: the real20-ID module proof is admitted against the final source catalog. Original20→19, duplicate applicable ID, foreign cwd, unexplained skip and a different actual PASS ID at the same total all reject. A legitimate original ignored module remains NOT_APPLICABLE, not executed/green.
- C: the genuine historical45 receipt remains accepted for its actual original inputs. All five original required-command deletions reject. Its old packaged SDK cannot certify the changed current cohort at the combined consumer. Current immutable SDK raw output has217 unique PASS IDs and the original217/2 summary. No new aggregate/current-cohort distribution PASS is inferred.

**R1-D actual residual:** `validate_four_receipt` reconciles each external owner's list/run command, cwd, listing/output bytes and hash, selected IDs and actual PASS IDs **with each other**, but does not reapply the producer's authoritative original-ID/migration checks and frozen selection floor. A consistent omission from listing + run output + selected IDs/count + recomputed hashes is still admitted.

Independent replay: `python D:/autostack/auto-lang/docs/plans/attachments/890/review/t12-owner-missing-id.py` under the frozen environment above. See [t12-owner-missing-id.log](#e890-final-summary), its result JSON, and `t12-public-{owner-valid,ui-short,man-short}-result.json`.

| Source-bound control / mutation | Existing real producer rule | Real receipt consumer / public gate check |
|---|---|---|
| Original actual UI listing, 3,128 listed / 3,110 selected | `baseline_for('tf','ui-full')` + `validate_ids` accepts the control. | `validate_four_receipt` accepts; public `four-owner-actual-execution` = PASS. |
| Delete `auto-ui-compiler\ta2ui::export::tests::test_export_button_action`; consistently regenerate listing, PASS output, selected IDs/count and both hashes: 3,110→**3,109** | Same original source baseline rejects **missing=1, suppressed=1**. This is not an ignored or migrated test. | **Still accepted; public gate check PASS.** |
| Actual original Man `man-t` listing, **281 selected**, frozen floor281 | `check_minimum` accepts legitimate control. | Receipt consumer accepts. |
| Consistently remove one actual Man ID from listing/run/IDs/count/hashes: 281→**280** | Actual producer rejects **280 < 281**. | **Still accepted; public gate check PASS.** |

The original UI/Man listings are the retained actual `t08-four-owner-listing-fixed` artifacts. The replay first checks them against the current producer's real baseline/floor functions. It exercises the production receipt validator and the actual public `web-repository-fixture.run` entry with stored receipt files. All source/resource/Cargo and proof validators remain real and unpatched. As in the permanent accepted complete-protocol fixture, **only the external Git/environment fingerprint is substituted** so the isolated proof protocol can be tested without executing a full owner batch. This boundary is explicit: synthetic run-output protocol files do not claim test-body execution.

Public positive and both negative runs have overall **partial / exit2 only because the independent distribution leg is deliberately pending**; their individual `four-owner-actual-execution` checks incorrectly remain **PASS**. This is not reported as an actual whole-delivery PASS in this round. The public entry adds current-identity reconciliation after `validate_four_receipt`, but has no supplementary original-ID/floor guard that would reject these shortened owner proofs. The residual therefore reaches the same public acceptance boundary, rather than being an unused helper concern.

Cause is established in source: the producer `scripts/run-full-regression.py` calls `check_minimum` and `validate_ids` before recording owner execution. The repaired consumer instead uses `listed_ids(...)`, set/count equality and `execution_ids(...)`; it never consumes those original required-ID/floor obligations for external owners. Web/media complete module/consumer/condition checks are separately enforced and pass their affected controls.

**Minimal closure:** make public receipt admission preserve the existing source-bound owner baseline/migration and floor rules as well as list/run equality. Required original UI IDs must not disappear consistently from all proof fields; Man's original selection floors must not disappear at consumption. Preserve migration counts only when their complete target obligations execute in the same union. Add permanent positive full-owner protocol controls and consistent-delete negatives for the demonstrated UI and Man paths. Reuse the authoritative existing contracts rather than inventing a weaker replacement denominator. Re-run the affected ownership/public-consumer checks and this holdout; the final real four-owner batch remains the later frozen gate, not a per-finding repair test.

##### Second failed final review: diagnosis and oracle adjustment required before more repair

This is the second consecutive final needs_fix of PLAN-890. Work must first record and verify the remaining-invariant diagnosis and oracle adjustment, then make its bounded repair; a new implementation commit alone does not justify another ready handoff.

| Remaining invariant | Why repair1's oracle missed it | Affected states | Required adjustment / closing oracle |
|---|---|---|---|
| External owner keeps original required IDs and floors at public consumption | Tests delete a selected-ID field or change log/hash independently; their accepted owner protocol uses one self-declared test per external leg. Those tests prove internal consistency and field presence, not preservation of the authoritative source denominator. | UI original baseline/migration IDs; Man owner floors; consistent listing/output/count/hash shrink. Legitimate original ignores/manual conditions and fully executed migration targets must stay valid. | Start with an actual source-bound complete owner listing accepted by the original producer. Consistently remove an original required UI ID and take Man below its frozen floor, updating all proof fields. Both production consumer and public gate check must FAIL; unmodified controls must PASS. |

No third independent product defect was established. R1-D is residual invariant closure of R1, not a new scope extension. R2 and unaffected transport/generator/installer/UI evidence should not be reopened to compensate for this governance gap.

##### Current acceptance and verification map

| AC | Result | Current decision |
|---|---|---|
| AC-01 | pass, retained | Original provenance/source/resource/history inventory; current five authoritative identities and submitted official894 synchronization verified. |
| AC-02 | pass, retained | Neutral type/DAG/!Send port unchanged; current immutable SDK source comparison and original closure evidence applicable. |
| AC-03 | pass, updated | R2 file-thread handoff closes the configured standalone CLI gap; explicit installer/owned-body/Man/UI source paths retain their earlier evidence. |
| AC-04 | pass, retained | HTTP/shared/non-HTTP generation ownership unchanged; real20-ID proof admitted, prior885/compiler/runtime evidence remains bounded to actual inputs. |
| AC-05 | pass, retained | UI/Man pins and composition code unchanged; earlier worker/proxy/command evidence retained. |
| AC-06 | pass, updated | Actual divergent-port development/proxy configuration and Host400 now pass; previous supported wire/policy matrices retained without enlarging support. |
| AC-07 | pass, retained | Resource/transport algorithms unchanged; fresh owned shutdown202/exit0/port rebind and original lifecycle evidence remain applicable. |
| AC-08 | **fail** | R1 A/B/C examples repaired, but R1-D still admits missing original UI ID / below-floor Man owner proof at public consumption. |
| AC-09 | pass, bounded work evidence | Current fixed SDK/CLI actual fresh/relocated behavior and217/2 inspected; old45/history14 apply only to unchanged recorded mechanisms and inputs. Full current-cohort release certification remains explicitly merge-owned. No independent distribution-mechanism defect remains established here. |
| AC-10 | pass, retained | Original fixed Blog314, cold build/API13 and Vue15/native22 evidence retains unchanged consumer/UI/generator scope. R2's relevant SDK thread repair is now separately exercised. No GUI rerun was needed. |
| AC-11 | pass, retained | Domain exits, named bridges/shared owner/media source, immutable installer and old/new valid Core history unchanged. |
| AC-12 | **fail** | The mandatory governance invariant R1-D remains; it cannot be converted to debt. Existing scoped health/whitespace/warning evidence is retained. |
| AC-13 | **fail** | This independent affected review is complete, with needs_fix; no final pass. |
| AC-14 | partial / pending merge | Actual final gates/landings/deposition/archive/guard cleanup deliberately unexecuted. |

Affected V results: V-03 and V-06 now pass for the repaired file/CLI entry; V-09 passes the actual configured port/profile/Host control; V-11 remains fail for R1-D; V-12/13 retain bounded work evidence with current aggregate release certification pending; V-17 remains fail because the mandatory acceptance gap persists; V-18 records this independent needs_fix and pending closeout. Original V-01/02/04/05/07/08/10/14/15/16 coverage is retained from attempt1 with the concrete unchanged-input comparisons above. No unchecked risk family was silently turned green by the repair totals.

##### Independent executions, reuse and scope limits

- Ownership14 was independently run once: **14 passed /14.029s**, actual public missing-proof rejection included; log `t12-ownership-controls.log`.
- Original CLI distinct-port/Host holdout on rebuilt7220 executable: **3 passed**, normal shutdown and owned port release; `t12-cli-launch-loss.log` / result.
- Current real20-ID raw execution proof accepted; shortened/duplicate applicable IDs, foreign cwd, unexplained skip and wrong actual PASS ID at unchanged total reject. Original ignored NOT_APPLICABLE control accepted. No bodies were executed for that N/A control.
- Genuine old45 validates only for its original recorded cohort; original five missing-command controls reject; old package/current SDK mismatch rejects. Current immutable SDK raw217 unique IDs/summary verified. Raw evidence and code/source hashes, not a banner, were inspected.
- All16 entries in `repair-1-evidence-index.json` independently match their exact bytes/SHA-256. Core6, Web HTTP3, module-consumers49, purpose19 and actual SDK217/2 are reused only within their source/config/assertion scope. Actual20 execution differs from final total identity solely in advisory registry indentation; its leaf fields, checker bytes and source inputs remain compatible and were independently consumed against the final catalog.
- Both periodic schedules retain all47 current recipes; the official894 seven leaves and original conditions/floors remain registered. No new eligibility exclusion or sample/budget reduction is accepted.
- Full source/config identity matches the given freeze; all five final trees remain clean. No code was repaired during review.

No full catalog test-body gate, independent SDK suite, UI/Man full run, GUI, GC/scale/performance matrix or periodic batch was repeated for this re-review. Required later operations remain pending, not green. Original failures, historical AAVM reds, two original ignores, manual/platform/milestone conditions and Windows/environment limits remain named in the original receipts.

##### Spec proposal status

The repair clarification preserves the original r6 contract; it does not authorize weakening it. Canonical target hashes are unchanged from attempt1 and the current proposal is frozen at3ed4dfec... above.

| SD | Status after affected re-review |
|---|---|
| SD-01 | pass proposal, unchanged Web pointer/owner/version and no remote-release claim. |
| SD-02 | **partial**: intended actual owner/source/distribution proof contract remains correct, but required-ID/floor admission claim is not yet true for R1-D. |
| SD-03 | pass proposal after R2: actual neutral file-entry snapshot propagation and explicit absent/ordinary contexts agree with code and current wire controls. |
| SD-04 | pass proposal, unchanged HTTP/shared/non-HTTP/media split and885 semantics. |
| SD-05 | pass proposal after R2: supported profiles/listener/policy now apply at the public file CLI; no new platform/TLS/auth product claim. |
| SD-06 | pass proposal, unchanged Man/UI consumer rules; final landed cohort must still be the actual merge result. |
| SD-07 | **partial**: original conditions/purpose/migrations remain valid, but public full-owner proof must also conserve source-bound original IDs/floors. |
| SD-08 | **partial**: publish canonical Web facts only after R1-D closes and actual final release/four-owner gates complete; no new aggregate PASS yet. |
| SD-09 | pass proposal, unchanged consumer knowledge and bounded UI compiler snapshot correction. |

No canonical knowledge or live ledger/index was written. Repository-required explicit deposition remains merge-owned; do not replace unmet invariants with softer Spec prose. No retention operation occurred.

##### Route and minimal task reopening

Return to work in the same trees. **Reopen T-08 and dependent T-10 only** for this established residual and the affected readiness/freeze/Spec claims; parent recomputes completed-task count/state. R2 is closed, and T-09's independent artifact/distribution-mechanism work can retain its bounded valid evidence. T-01..07 and unchanged T-09 need not be repeated merely because a second final review failed. If the repair actually changes their implementation inputs, assess that real diff rather than preemptively reopening them.

Before another repair/readiness handoff, fulfill the second-failure diagnosis/oracle adjustment above. After committed repair, re-review R1-D with its legitimate controls and the affected receipt/owner branches; preserve R1 A/B/C and R2 closure unless their actual source/config inputs change.

After acceptance passes only: T-12 owns the frozen affected complete `tm` module/consumer/integration union, actual full required four-owner/current-cohort release gate and any due main-only single-instance periodic batch. It then owns actual repository landings/recovery points, canonical Specs and explicit required ledger/index/module-plans readback, archive/counts and every mandatory guard/cleanup. This needs_fix receipt certifies none of those future operations and covers no future implementation.


</details>

<a id="e890-f614e876595a8318"></a>

<details>
<summary>review\t12-frozen-spec-delta.md · SHA256 3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


##### r6 repair clarification (R1/R2; same existing contract)

SD-02/07/08: the final four-owner consumer requires the full explicit registered
recipe/module union, source/dependency/effective-configuration identity and
original conditional applicability. Required owner cwd, list/run command,
listing/output path and hash, selected IDs and actual per-ID results must agree.
A self-reported success banner/count cannot replace execution proof. Public tm
may refresh into an ephemeral catalog; its source contracts are revalidated,
not weakened or rejected merely because published metadata bytes differ.
Original negative Case oracles can legitimately produce a raw exit1/FAIL while
their explicit meta-oracle passes; those source/oracle identities remain checked.

Distribution command-driven checks require their actual command/expected and
actual exit/output identity. Runtime/detail and exact-input SDK reuse retain
their own explicit schemas; they are not excuses to omit a required command.
Current-owner packaged SDK/crate/installer inputs are reconciled with the
distribution's frozen source cohort at the public combined consumer. A valid
historical receipt does not certify changed SDK bytes. Both periodic manifest
consumers keep every current registered recipe; responsibility/purpose advice
never changes original registration, conditions, samples, budgets or floors.

SD-03/05/08: Core's file entry (`run_file`/`run_file_with_args` through
`script_entry::run_with_path`) captures the neutral invocation launch before
spawning its execution thread and rebinds it on that worker, like the existing
run bridge. An explicit absent launch remains absent; ordinary no-service files
need no Web provider. Web `run --config` consumes the declared loopback/address,
port, profile/config identity and policy even when legacy environment values
differ. Existing no-config legacy behavior remains. No new protocol, network
product, platform promise, package or dependency edge is introduced.

Evidence remains source-bound and scoped: current rebuilt SDK217/2, fresh and
relocated CLI configuration/Host controls, original unaffected generator/Man/UI
proofs, and explicit final current-cohort release/four-owner gates pending under
T-12. Do not deposit a new aggregate PASS before that frozen combination actually
executes. The known first failures and original manual/ignored conditions remain.


</details>

<a id="e890-c4a9850272f2d870"></a>

<details>
<summary>review\t13-affected-review-r6.md · SHA256 9b642a8adf4ba70bf0fbc9ec0fbba6b016194d215b225a3d14f128280d24c028</summary>

#### PLAN-890 r6 — final affected re-review, attempt 3

stage: review | review_scope: final | outcome: **pass** | reviewer_context: `/root/plan890_final_review`

This is independent T-11 acceptance on the exact frozen candidate below. **R1, including residual R1-D, is closed; R2 remains closed.** All work-owned acceptance criteria and the proposed Spec delta pass within the recorded support/evidence boundaries. T-12's actual frozen module/four-owner/current-cohort release gates, due periodic gate, repository landings, deposition, archive and guarded cleanup remain **pending / NOT_PASSED**. This receipt is not U890 or a claim that those future operations passed.

The same independent review context used the original five authoritative implementation trees with implementation paused. No code, Plan, canonical Spec, ledger or index was modified; no staging, commit, second checkout, implementation agent, GUI or retention operation occurred. Local evidence writes stayed in this main-checkout review directory. Parent owns Plan/status/count routing.

##### Exact reviewed baseline

Group: `D:/autostack/.wt/lang-890/`. Git registrations, paths, branch, exact HEAD and clean status were independently checked at entry and final readback.

| Repository | Branch | Reviewed HEAD |
|---|---|---|
| auto-lang | plan-890-dev | 29cf444809e9166c6cfb98ff741b31ae6bc112cf |
| auto-web | codex/plan-890-dev | e01232b3d449ea590a85d9458d553ab17b9456a3 |
| auto-ui | codex/plan-890-dev | 9aad12ccbc5f01d5241eb95f657666a943667ff2 |
| auto-man | codex/plan-890-dev | cb4ab3dd5d411314cce10b129eeb3b433a11c466 |
| auto-down | detached | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 |

Main bookkeeping HEAD: `105a469656f2a1a2dc8d4ec5119b6fe2fcdf59e2`. Main Plan r6 was submitted as execution_done10/12; its SHA-256 is `5f5cf79efb130ffe48d49c7d68b364d4cc7061357d61bb16835a2d9ba8fc3580`.

Independently verified submission hashes:

- `repair-2-diagnosis.md`: `af6b63853780daee8f477048b4c2190d26b0f1992deb56edaf9d357467cc2987`.
- `repair-2-handoff.md`: `b4a73dd262592082bf3e178077dd3555e9ff40f9428e1711ca6e8e121bf70f17`.
- Proposed delta and byte-exact LF [t13-frozen-spec-delta.md](#e890-55a4a012f744207b): **`40791c32bd48a6e30473d67748d286a7005ec428fcb149cfb7267f6e9893b681`**.

Under `CARGO_TARGET_DIR=G:/autostack-build/plan890-frozen-target` and an empty `RUSTC_WRAPPER`, independent code/dependency/effective-environment identity is exactly **`03f08f193042eb7670a487baf4202b97404b5c04e075dabfe9fdc2691b36bf33`**, matching the submitted catalog. Effective environment hash: `a92a86a788c826d4bc7bd0e7b02da4cce96952501b3df5e775a492bb884ff220`. Current discovery has47recipes /7368obligations /1220leaves; discovery is not execution.

[t13-final-baseline.json](#e890-final-summary) records exact roots/branches, prior-review/current revisions, repair-diff hashes, lock hashes, source/environment identity, canonical Spec hashes and local evidence hashes. All twelve canonical target Specs remain byte-identical to prior reviews. Actual scoped Git comparison against immutable SDK producers language `74e13580adff15a861ca46efb5d0ab0ce89c168d` / Web `e01232b3d449ea590a85d9458d553ab17b9456a3` shows no changed packaged SDK/crate/stdlib/owner-tool/fixture input. UI/Man/Web/Down production pins and R2 implementation inputs did not change in repair2.

The previously read review skill and verification-contract source remain applicable. This is an affected re-review of [attempt1](#e890-f0fee9ee4a1d7f26) and [attempt2](#e890-1ce952cb28cda1c4), retaining their applicable full AC/V/SD evidence rather than restarting unrelated acceptance work.

##### Two-failure readiness audit

The required diagnosis/oracle adjustment was checked **before accepting readiness**, not inferred from41passing tests. The diagnosis accurately states the independently established cause: repair1 compared supplied listing/run/count/hash fields but did not consume the source-bound original-ID/floor denominator. Its single arbitrary-test positive owner fixture could not detect consistent deletion.

The retained pre-fix replay reproduces the exact earlier failure on its named pre-repair source: original UI3110→3109 and Man281→280 were admitted by the public owner check despite the producer's rejection. Original positive controls were accepted. The pre-fix raw record and original reviewer inputs remain preserved.

Actual repair and permanent oracle now match that diagnosis:

| Required adjustment | Verified implementation/oracle |
|---|---|
| Producer and public consumer share the authoritative denominator | `run-full-regression.py::validate_owner_selection` is called by primary/extra producer legs and `web_delivery.py::validate_four_receipt`. It consumes the existing baseline/migration/retirement/floor rules; the minimum files and original IDs were not weakened. |
| Positive fixtures keep original required source identities | Language/UI protocol positives preserve the actual source baseline instead of one arbitrary owner ID. Man/UI extra legs preserve their original minimums. |
| Consistent deletion and suppression fail at public consumption | Permanent controls and the independent original holdout update listing/output/IDs/count/hashes together. The public gate rejects original UI ID loss and Man below-floor proof. |
| Migration credit means actual target execution | Consumer derives targets from original required-source mappings and checks them against IDs from already validated PASS/applicable module execution. Catalog/selection presence, a supplied count or unrelated PASS IDs cannot earn credit. |
| Original ignore applicability stays legal | ui-tu's unchanged single-process command uses the already required same-source ui-full nextest ignore flags, with exact `condition_excluded` IDs. Genuine866PASS/2original-ignore evidence reconciles correctly; arbitrary exclusion does not become original N/A. |
| Denominator failure propagates before expensive proof consumption | Consumer performs all owner denominator checks before its execution-proof loop. The actual public invalid cases fail with the baseline/floor diagnostic; producer queue controls retain failure/zero-selection propagation. |

This satisfies the mandatory post-two-failures diagnosis and verified oracle adjustment. It is the same approved r6 invariant, not a new cryptographic authenticity, eligibility, performance or platform requirement.

##### R1-D closure: actual independent public holdout

Command from the authoritative language tree, with the frozen environment above:

```text
python D:/autostack/auto-lang/docs/plans/attachments/890/review/t13-owner-missing-id.py
```

The durable original independent input was retained. Comparing the work replay with the prior independent replay shows only stronger assertions of the required rejection/public FAIL outcomes; no negative input or positive oracle was weakened. This review copied it to the t13 evidence prefix and independently ran it, exit0.

| Actual source-derived control/mutation | Producer guard | Repaired receipt consumer and actual public entry |
|---|---|---|
| Original UI3128listed /3110selected, complete original baseline | Accepted | Accepted; `four-owner-actual-execution` PASS. |
| Consistently delete `auto-ui-compiler\ta2ui::export::tests::test_export_button_action`, including listing/run/selected IDs/count and recomputed hashes:3110→3109 | Missing1/suppressed1 rejection | Same rejection; public owner check **FAIL**. |
| Real Man281 selected, original floor281 | Accepted | Accepted. |
| Consistently delete one actual Man ID and update all proof fields:281→280 | Below frozen minimum280<281 | Same rejection; public owner check **FAIL**. |

Raw evidence: [t13-owner-missing-id.log](#e890-final-summary), its result JSON, and `t13-public-{owner-valid,ui-short,man-short}-result.json` / corresponding gate inputs. The unchanged real public `web-repository-fixture.run` consumes the stored proof. As in the permanent accepted protocol fixture, only the external Git/environment fingerprint is substituted; source/resource/Cargo and all receipt/atomic/execution validators remain real and unpatched. Synthetic protocol run text is **not** claimed to execute UI/Man bodies.

The positive public aggregate remains partial/exit2 solely because independent distribution is deliberately omitted. The invalid aggregates are fail/exit2 and their specific owner checks FAIL. This distinction is preserved; no actual full delivery certificate was manufactured for the review.

##### Independent sibling/applicability and failure controls

`t13-source-controls.py` independently checks the source guard and genuine applicability inputs, exit0:

- Original source language and UI controls are accepted with exactly115 and10 source-derived migrated-target credits respectively. Removing **one** target from the purported actual passed set rejects, even after adding an unrelated target; an empty execution set also rejects. Source/catalog membership cannot replace execution. Dropping an original language/UI required ID consistently rejects missing1/suppressed1.
- Actual Man281→280 rejects through the same shared guard.
- Genuine T10 raw866 unique PASS IDs and two original ignored IDs reconstruct libtest's actual868 `: test` listing shape. The same-source nextest flags yield exactly those two N/A identities and866 active IDs. Floor863 is unchanged. The actual `rust.rs` ignore-declaration source is unchanged from T08 to current UI9aad. Ignored bodies are not counted as executed or passed.
- An arbitrary new/non-source identity cannot acquire original-ignore status from this reconciliation.
- Independent current source identity/catalog counts match the freeze. All350 evidence-index entries,17,310,478bytes, independently match exact sizes/SHA-256.

The first source-control setup used only the required baseline as the language positive and correctly failed its frozen floor3185<3189. That was an incomplete probe setup, not a product failure. The first raw log is preserved as `t13-source-controls-first.log`; the corrected positive uses the actual retained source listing, satisfying both original IDs and the unchanged floor. No product assertion or denominator was relaxed.

Four specific queue/ignore controls were independently run once, **4passed /1.111s**, exit0: first-child failure stops/receipts the queue; frozen batch minimum rejects; original libtest ignore parsing remains correct; zero Man selection rejects. Log: [t13-exit-ignore-controls.log](#e890-final-summary). These checks exercise error propagation and applicability, not a full regression tier.

The submitted current41 result was separately read and verified at `repair-2-owner-and-driver-final.log`:26driver +15ownership,41passed/74.572s. Permanent source assertions include ignored/filtered/omitted required UI IDs, missing language IDs, Man/ui-tu floors, fabricated migration counts, missing actual migration execution, legal original UI N/A and arbitrary-exclusion rejection. The old84-case diagnostic20-error/9-failure logs are preserved and not presented as41pass. Scoped queue mock changes supply the files their real boundary owns and isolate Cargo mocking from real source-contract Git reads; original assertions were retained. The final41 is reused on its exact unchanged inputs rather than rerun merely for ownership change.

##### Findings and reused evidence

| Finding | Current status / scope |
|---|---|
| R1-A, absent execution proof | Closed in repair1; actual public missing-proof negative and permanent controls remain applicable. |
| R1-B,20→19 unconditional module execution | Closed in repair1; current module validator/execution proof unchanged. The original real20-ID evidence and shortened19 rejection retain their inspected source/input scope. |
| R1-C, optional distribution command proof | Closed in repair1; required command/exit/log/hash and current-cohort source checks remain unchanged. Genuine original45/history14 are historical/input-bound evidence, not a new aggregate current release PASS. |
| R1-D, external owner original-ID/floor loss | **Closed by this re-review**: complete source positives accepted, consistent UI/Man shrink rejected at the actual public boundary, shared producer/consumer guard and actual migration-target execution verified. |
| R2, CLI file-thread configuration loss | Remains **closed** at unchanged production inputs. Attempt2 independently observed the rebuilt7220 executable's actual declared port/loopback/profile/hash/Host400 and old no-config control, shutdown202/exit0/rebind. No unrelated CLI/Rust/GUI body replay is needed for this Python-only repair. |

No open work-owned finding, mandatory oracle gap, unapproved deferral or new semantic decision was established. No third/fourth product defect is inferred from fixture setup failures. Rust/lock/transport/generator/installer/UI/Man product inputs are unchanged by repair2; prior accepted evidence remains bounded to its recorded inputs. Existing warning totals, historical AAVM failures, platform/manual/milestone/ignored conditions, Windows limitations and original raw reds are retained, not reclassified green.

##### Current AC/V acceptance map

| AC | Result | Evidence decision |
|---|---|---|
| AC-01 | pass, retained | Original provenance/history/source/resource inventory and current five exact identities. |
| AC-02 | pass, retained | Neutral port/type/DAG/!Send ownership and immutable packaged SDK source comparison unchanged. |
| AC-03 | pass, retained | R2 file-entry assembly is independently closed; explicit host and owned-body/Man/UI composition remain as verified. |
| AC-04 | pass, retained | HTTP/non-HTTP/shared generation split and885 semantics unchanged; original compile/wire and real20-ID proof apply. |
| AC-05 | pass, retained | UI/Man consumer pins, commands/worker/proxy/composition code unchanged. |
| AC-06 | pass, retained | Supported VM/Rust policy/wire matrices and repaired actual configured CLI/Host controls; no enlarged support promise. |
| AC-07 | pass, retained | Owned resource/terminal/network/logger algorithms and actual closure evidence unchanged. |
| AC-08 | **pass, updated** | Shared original-ID/retirement/migration/floor admission; independent public shrink negatives, valid source controls, actual migration-credit and original N/A controls close R1-D. |
| AC-09 | pass, bounded work evidence | Current immutable SDK217/2/fresh-relocated actual proofs retained; original45/history14 only their recorded inputs/mechanisms. Final combined current-cohort certification is explicitly merge-owned. |
| AC-10 | pass, retained | Fixed Blog314/cold compile/API13/Vue15/native22 and UI866/2 evidence remain within unchanged consumer/product inputs. No GUI rerun. |
| AC-11 | pass, retained | Domain exits/named bridges/shared/media source and valid historical Core recovery unchanged. |
| AC-12 | **pass, updated** | No mandatory work finding remains; scoped source/health/whitespace/tests and frozen SD are coherent. Prior failures/limits are explicit. |
| AC-13 | **pass** | This independent final affected acceptance review binds all work-owned criteria and the SD to the exact freeze, retaining earlier applicable coverage. |
| AC-14 | **pending merge** | Actual frozen final gates, all repository landings, canonical deposition/readback, archive and every guard/cleanup remain unexecuted by this review. |

V-11 and V-17 now pass for the repaired source-denominator/oracle/readiness invariant; V-18's independent-review portion passes and its merge portion remains pending. V-01..10 and V-12..16 retain prior accepted scoped evidence, with V-03/06/09's R2 closure and V-12/13's explicit current-cohort gate distinction preserved. The first review's full risk matrix was not replaced by test totals: provider/VM/thread/body/resource/policy/generation/SDK/transaction/consumer/history risks retain source-bound proof, and all changed receipt/baseline/migration/ignore/error paths received this affected review.

##### SD decision

All nine proposed operations pass **as a proposal for the verified implementation plus explicit later gate results**. No canonical publication occurred here. Canonical source hashes are unchanged, and the frozen proposal above is LF/byte-exact.

| SD | Verdict |
|---|---|
| SD-01 | pass: language pointer, distinct Web owner/version/canonical source; no duplicated normative domain or remote-release claim. |
| SD-02 | pass after R1-D: original source-bound IDs/floors and actual migration targets are now required at public receipt consumption; future full gate results must still be actual. |
| SD-03 | pass: neutral local VM ownership, explicit host selection and repaired file-thread launch snapshot agree with code and bounded actual runtime evidence. |
| SD-04 | pass: unchanged HTTP/shared/non-HTTP/media ownership and inherited885 contracts. |
| SD-05 | pass: actual supported configuration/policy/listener paths, separate Axum tracks and explicit limits; no new platform/TLS/auth product promise. |
| SD-06 | pass: unchanged consumer/repository boundary; record the actual landed cohort at merge. |
| SD-07 | pass after R1-D: shared source denominator, original conditions/floors and execution-derived migration credit; purpose advice does not alter eligibility. |
| SD-08 | pass proposal: canonical Web facts are grounded; actual final release/four-owner receipt claims remain conditional on T-12 execution, not anticipated green text. |
| SD-09 | pass: unchanged consumer knowledge and bounded existing UI emitter correction, with no new AST/runtime protocol. |

The libtest applicability clarification describes actual original ignore preservation, not a new exclusion or floor reduction. Component targets and empty canonical-goal impact metadata remain as reviewed; no new semantic Plan revision is needed. Merge must preserve concurrent canonical work and perform the repository-required explicit ledger/index/module-plans deposition and readback.

##### Route and explicit pending gates

T-08 and dependent T-10's repair scope is complete; no work task needs reopening. Parent may record **reviewed11/12** for this exact candidate and route to T-12.

Before landing, the merge owner must freeze/check the actual combined inputs/environment and execute the affected complete `cargo tm` leaf/consumer/integration union, the real required four-owner/current-cohort release/history gate and any due main-only single-instance periodic batch. Preserve original conditions and named historical reds; do not waive a required failure or turn list/discovery/N/A into execution. Reuse valid unchanged evidence only with its recorded input/config justification.

Then perform actual landings/recovery-point readback in every modified repository, canonical Specs and required ledger/index/module-plans deposition/readback, archive/count verification and each mandatory worktree guard/cleanup. Those operations remain NOT_PASSED/pending here. User-authorized evidence accounting/retention belongs to closeout; none was performed in review. Final acceptance on this freeze does not cover future implementation edits or authorize remote push/publication.


</details>

<a id="e890-55a4a012f744207b"></a>

<details>
<summary>review\t13-frozen-spec-delta.md · SHA256 40791c32bd48a6e30473d67748d286a7005ec428fcb149cfb7267f6e9893b681</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


##### r6 repair clarification (R1/R2; same existing contract)

SD-02/07/08: the final four-owner consumer requires the full explicit registered
recipe/module union, source/dependency/effective-configuration identity and
original conditional applicability. Required owner cwd, list/run command,
listing/output path and hash, selected IDs and actual per-ID results must agree.
A self-reported success banner/count cannot replace execution proof. Public tm
may refresh into an ephemeral catalog; its source contracts are revalidated,
not weakened or rejected merely because published metadata bytes differ.
Original negative Case oracles can legitimately produce a raw exit1/FAIL while
their explicit meta-oracle passes; those source/oracle identities remain checked.

Distribution command-driven checks require their actual command/expected and
actual exit/output identity. Runtime/detail and exact-input SDK reuse retain
their own explicit schemas; they are not excuses to omit a required command.
Current-owner packaged SDK/crate/installer inputs are reconciled with the
distribution's frozen source cohort at the public combined consumer. A valid
historical receipt does not certify changed SDK bytes. Both periodic manifest
consumers keep every current registered recipe; responsibility/purpose advice
never changes original registration, conditions, samples, budgets or floors.

SD-03/05/08: Core's file entry (`run_file`/`run_file_with_args` through
`script_entry::run_with_path`) captures the neutral invocation launch before
spawning its execution thread and rebinds it on that worker, like the existing
run bridge. An explicit absent launch remains absent; ordinary no-service files
need no Web provider. Web `run --config` consumes the declared loopback/address,
port, profile/config identity and policy even when legacy environment values
differ. Existing no-config legacy behavior remains. No new protocol, network
product, platform promise, package or dependency edge is introduced.

Evidence remains source-bound and scoped: current rebuilt SDK217/2, fresh and
relocated CLI configuration/Host controls, original unaffected generator/Man/UI
proofs, and explicit final current-cohort release/four-owner gates pending under
T-12. Do not deposit a new aggregate PASS before that frozen combination actually
executes. The known first failures and original manual/ignored conditions remain.

##### Repair cycle 2 clarification (r6 unchanged contract)

SD-02 / SD-07 / AC-08/12: source-bound full-owner receipt admission and the execution producer share the same baseline/migration/retirement/frozen-floor selection guard. Matching list/run/IDs/count/hash alone cannot remove an original required ID or lower an owner below its original floor. Consumer migration credit comes from original required-source mappings and is legal only when the exact target IDs actually passed in the already validated complete module union, not merely appeared in the catalog/selection or a receipt count. Language/UI source baselines and Man/UI nextest/libtest floors preserve their original applicability and eligibility. No floor, original ignore, condition, sample or milestone is relaxed.

Implementation: scripts/run-full-regression.py::validate_owner_selection is shared by primary and extra execution legs and scripts/test-governance/web_delivery.py::validate_four_receipt. Required original UI/language IDs and Man/UI libtest floor controls include internally consistent list/output/count/hash mutations. The public fixture verifies failure propagation. This is the original r6 proof contract; no new cryptographic/log authenticity, performance, platform or product promise.

SD-03/05 and all unaffected owner, lifecycle, generator, artifact and Blog clauses retain the repair1 proposal and their explicit bounded evidence. SD-08 publication remains after independent acceptance and final required current-cohort/module/four-owner gates. Current canonical documents and live ledger/index remain unwritten at work stage.

UI single-process applicability detail: ui-tu keeps the original cargo-test single-process command. Because libtest --list omits ignore flags, its original ignored identities are reconciled against the already proved same-source ui-full nextest listing and recorded as condition_excluded. Exact selected IDs and actual passed output exclude those identities; arbitrary self-reported exclusions reject. The existing ui-tu floor stays863, and the genuine current UI866-pass/two-original-ignore result remains consistent. Ignored bodies are not counted as executed or passed.


</details>

<a id="e890-782acb3b7e32d925"></a>

<details>
<summary>review\t14-affected-review-r6.md · SHA256 b935eab66852812842c9d20d8cfd2d7a851cd4d77487c4de01d7f1c418d19658</summary>

#### PLAN-890:r6 independent final affected review, attempt 4

Outcome: **PASS** for the frozen work-owned acceptance and proposed/prepared Spec delta below. This is the follow-up review receipt, not Plan task T-14 or a completed merge. No new work-owned finding was established. Actual unified merge gates remain **NOT_PASSED**.

`stage=review; review_scope=final (affected re-review); reviewer_context=/root/plan890_final_review; plan_revision=r6; current main Plan=execution_done10/12; diff_base=d4158137e; implementation paused`.

The reviewer is the assigned independent agent context, separate from implementation. No invented model/session identity is asserted. Same authoritative sibling trees were inspected; no additional checkout, implementation agent, code repair, canonical publication, Plan edit, staging, commit, retention operation or other-chat message occurred. Parent's independent consistency audit is corroboration, not this verdict's basis. The current main Plan, review skill and repository workflow govern this review; user-directed ledger/index compatibility takes precedence over the skill's generic cache wording. Evidence maintenance belongs to merge; none occurred here.

##### Frozen baseline

All paths below are under `D:/autostack/.wt/lang-890/`. `git worktree list --porcelain`, repository/branch/HEAD and `git status --porcelain` were read independently at entry and final readback. Every candidate tree is clean.

| Repository | Branch | Exact HEAD |
|---|---|---|
| auto-lang | plan-890-dev | 2ef7bece2fa6b2509ed04506789059b997728093 |
| auto-web | codex/plan-890-dev | de1783f1c94dbc4d656958faca74ab61a69831ba |
| auto-ui | codex/plan-890-dev | d347dd3d888545f6f07fe67b7f4c7f1be28741c8 |
| auto-man | codex/plan-890-dev | b9f0c0259ae8b167a635c978dab551e382dbc894 |
| auto-down | detached | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 |

Repair implementation is75af80a911eff5052527a9bf46675e446b0f5c85, required owner registration aaa7307f788659897fdaaf2dae8aa553b9512157, and catalog/debt checkpoint2ef7bece2. Independently comparing old4dbe6f667 to rebased e1600601f Git blobs found **zero non-Plan differences**. Rebase preparation is not permission to reuse a different physical-source fingerprint. Prepared canonical changes were inspected separately from product equivalence.

Public Cargo catalog identity is5a02e7b8ce6c89e5d1b8b5f18e4839406d9e3ad383eb485f3e1da11b3d5ee9aa (environment6c03fdc40a96d0d2c77abef00a74977aaeaae79b1d13275c82689771b3c8d013). Handoff direct identity983c36ac26c1f0bf458a54438950ec40c94ce850ab0bf96a9bce4c824a582198 has environment18304cc5ef66d5b6010c97f0d54768b24bf1a92de470269d520012acb23f21ad. Review shell direct identity is182f6ed1c7d5ffd421b08b4422e921ee1f9e89cb1b64793514a1541bbeba8d0d/environment aa2ef3ba911c3e023806664da55440f837b78389fd926c8af27ea349e1cc8c2f. These are **distinct environment-bearing identities**, with identical five repository input-byte digests, not interchangeable HEAD labels.

Final per-repository source digests, independently recomputed:

| Repo | Input SHA-256 |
|---|---|
| Lang | e69f6f7ea511501a51a5556afdc7636e1f2a19732e7fe6ff9002c85ebb2b131f |
| Web | 768a6ab276a602457f85ecd89fe74d5d63bba24c477883d11c6936e869e8afc0 |
| UI | 6a7b8fc4def23198bef502d0c5fa5a726b2eadaca240c47eb6d29877055577ad |
| Man | 5de79fdc1e916e4c946e5e4b1043a74dd73a4548c2adb63b95bff14a209f6f72 |
| Down | c0cf5ccad14e7aea049ca3f560c5da84877c0fcfb622a0889bb830f7e9057864 |

Work's frozen target is `G:/autostack-build/plan890-frozen-target`, `RUSTC_WRAPPER=''`, Cargo-configured stack/nextest conditions unchanged. Review's direct cached-binary and rustc probes did not rebuild the workspace; the actual preparation replay inherited the review shell (no CARGO_TARGET_DIR, sccache wrapper) but explicitly selected the owned tool target and produced the exact same worker bytes. Source identity with target/wrapper explicitly set still differs by environment/PATH; no aggregate identity equality is invented.

The exact handoff/diagnosis/SD hashes are respectively50ed90bf46a1f517c785c0ebf26184d72fca99e8792ec975e862040a5765bc5b,0305691854ef1b678f288e30f37b7d055250a79d53e9ba3fc27e1fb8ef77397d,a77d62978dfe65230a12e993adcc20c3688cc81f1925924db33191fe8acd816c. The LF-normalized frozen proposal [t14-frozen-spec-delta.md](#e890-15513e7611658e98) hashes **406de25245818691de6d48b33951302297af59e6d0280642c3d7e8617b163b48**; only CRLF→LF differs from the supplied SD bytes.

Local [t14-final-baseline.json](#e890-final-summary), SHA-256 **01054326d50bd917b8af7d9f405bf05c8711b87ac9b69434990015cb779a19fe**, binds67 canonical/source/ledger/index input hashes, exact tree registrations, raw input index, independent commands/results and the full new counterexample input source. Temporary compiled probes were isolated and removed; their input/output remains in this receipt. No original red or final raw file was overwritten.

##### Diagnosis, readiness and conservation

Read both prior needs_fix review histories and repair1/2 closure, the historical t13 PASS, actual interrupted merge `gate-needs-fix.md`, repair3 diagnosis, source diff and original/final results. The historical PASS is not certified as current by its name. Merge's actual red receipts remain red: partial25PASS/4FAIL/31N/A, complete=false, outer exit1; external owners/current distribution were not completed. M1, M1b, M2 and M3 were treated as in-contract blockers and repaired, not waived as historical debt.

The diagnosis records the first11PASS/2FAIL AAVM usize-conversion failure, provisional emitter green, abandoned generic-emitter workaround, failed local replay setup before execution, missing top-level owner rejection and final targeted results separately. Final source restores generic Rust emission byte-for-byte to d4158137e; the provisional green is not the final-source evidence. This is the required diagnosis/coverage adjustment after earlier failed reviews.

Independent stored-catalog comparison:7368→7371 obligations, **removed=[]**, exactly two Core instruction-boundary controls plus one preparation ownership control added. `module_catalog.load/validate/validate_top` passed on the actual current catalog: **47recipes /7371obligations /1221leaves**. The new two-test leaf is explicitly owned by existing Core/stdlib-assembly; original reference leaf retains its five tests. No old ID, original eligibility, baseline/floor, sample, golden or milestone was retired. Source-only catalog enumeration is not execution.

##### Independent checks and closure

| Invariant / actual check | Result and coverage |
|---|---|
| M1 preparation reaches real consumers | Called real `prepare_tools.prepare(root,['a2x'],temporary_output,set())`: Cargo `build --locked --offline -p a2x-probe --target-dir <Lang>/target/test-tools/regular --release --bin a2x-worker`, cwd experimental/auto-a2x, exit0/0.304s. Exact AC_A2X_HIR_WORKER points to regular/release/a2x-worker.exe, SHA77d5b767b33e538dba98b5d7b72bd4c186e8f7f1713775fe747ebdb459bc1d91. |
| M1 valid and negative actual body controls | Real cached auto_lang-ed57d7b1a13926d0.exe (SHA99c087b1adc0d53f5d5a9457293b098b088482030fe869457373cd4b97b9e4ea), exact first_case_green, tampered_golden_is_red and missing_worker_is_infra_error, each exit0; actual5-case source comparison accepts, one-byte golden mutation is FAIL, absent explicit worker is ERROR with missing-file diagnosis. Direct and ac-a2r case tool dispatch/source override and mixed-selection rejection inspected. No reliance on execution order or fixed external target. |
| M1b preparation exception cannot certify workload leakage | Real `python -m unittest test_native_meter.ProcessCapture.test_preparation_exception_requires_exact_image_hash_and_completed_reap`, exit0/4.380s. Its actual own-child leak is rejected under default workload policy; only explicit preparation flag plus exact pinned image bytes and completed Job reap qualifies. Unknown/image-error/incomplete/unreaped/nonzero/hash mutation rejects. Native meter/process_capture unchanged; workload execution never opts into preparation exception. |
| M2 thin namespace and owned cache | Actual diff adds only a forwarding namespace to prior File implementation and removes inherited CARGO_TARGET_DIR from both standalone guest Cargo builds. Inspect staging target→release exe→source-addressed publication; final runtime/emitter/Cargo dependencies otherwise unchanged. |
| M2 distinguishing forwarding holdout | Extracted exact current prelude, rustc-compiled without dependencies, six controls compare emitted namespace call to existing File result for Unicode split, normal range, EOF, zero limit, usize::MAX and missing file. Compile0/run0,6equal results; prelude SHA31d608134b93bf6fcb4e58c23cdf7a2fdccb52fe143b41c7511151f5c361bf56. This asserts forwarding, not a new production file API. |
| M2 original actual consumer legs | Read/hash/assertions of repair-3-aavm-consumer-final, original compile_corpus62.081s +compile_use_corpus24.777s PASS, nextest2/2 total86.875s; meter94.576s/error=null/111processes. Actual original corpus membership/oracles remain unchanged. Provisional aavm-final green is retained as provisional only; original first red preserved. No unrelated AAVM expansion. |
| M3 actual engine/decoder format | Engine consumes CALL_CLOSURE one-byte argc and CREATE_FUTURE u32 body/u8 count/u32 capture names; codegen agrees. Disassembler now consumes argc and reference closure delegates future width to that same decoder. Native proof before business instruction remains strict; no fake proof or bypass. |
| M3 original five +new boundary two | Actual auto_lang_core-402e36a205fa6fab.exe (SHAa49c73f33815156e0a5ea910f9293a882820e5dbe575e8ccfbca0c09a3155eda), filter `stdlib_assembly::reference:: --nocapture --test-threads=1`, exit0:7PASS/0ignore. Original final-interface, signature-cache invalidation, legacy-wire exclusion, unproved-TCP refusal and selected-JSON real heap proof retain assertions. |
| M3 independent boundary/combination holdout | rustc linked exact current Core rlib SHA0a74568c7cc33c0685d127f0467985a80ee88084a26bcfafa5374c75e6b7fe65. argc0/1/127/255 valid and invalid-successor pairs; future0/2/3 capture names containing invalid-opcode bytes; two truncations; post-closure CALL_NAT1902 production proof accepted and untyped interface SIGNATURE_UNVERIFIED rejected:18assertions, compile0/run0. Full source SHA c95c7391e65466dbb2ccf3952af13f960bc784a466c3a742d704263cfdf77139 in baseline. Truncated Flash emitted the existing OOB diagnostic and was rejected; no assertion was weakened. |

The first native-meter command used the wrong unittest class and returned loader AttributeError before executing any test. Correcting the class to ProcessCapture produced the actual result above; both outputs are retained. This is probe setup, not a product red. Full input source/results for both new holdouts are embedded in baseline for direct reuse.

ABT textual assembler/disassembler and legacy `OpCode::operand_size`/generator-classification tables are distinct historical consumers. This repair's actual preflight closure calls `vm::disasm::Disassembler::instruction_operands`, not those tables. Their whole-table format inconsistencies are outside the changed M3 native-reference path; no claim of general ABT/whole-VM format migration or all generator correctness is made. The affected shared Disassembler path was inspected and tested with valid/data/invalid/truncated/native-proof combinations. No in-contract M3 sub-item is deferred on that basis.

Independent raw verification: all **46files /6,352,734bytes** in repair-3-final-baseline.json match exact sizes/SHA-256, including final real logs/meters. Examples: final AAVM run26c945e9e3b07f579865726256007b77e86515a70b724e90ee22ebe2acb3c887; native7 run6538afe060e590e5425e0ec3d0bb93fd5f560c7d9272bdec0f090d273b441b7f; original targeted mixed red e57569e20766d38540453d2d942b82784378bee5ab199c43e60934a595260b05. This verifies bytes, not an aggregate new PASS. Work's53Python/18Rust assertion scope was inspected; this reviewer did not use that total as its sole oracle or repeat unrelated bodies.

`git diff --check d4158137e..HEAD` exits0. Actual warning comparison reads the prior build plus same-discovery/run logs,288 distinct categories before/after, added=[]; inherited warnings are not claimed absent. No added debug print/workaround or unapproved product promise was found. Diagnostic error context was improved without altering golden content/comparisons.

##### Acceptance map

Full initial risk/AC/V/SD mappings in t11/t12 and accepted t13 remain the coverage baseline; changed source, dependencies, configuration, preparation, gate evidence and prepared Specs were reconciled here. Reuse is bounded by same relevant byte inputs/mechanisms, not merely prior PASS. Original R1-A/B/C/D receipt denominator/execution/distribution defenses and R2 worker-launch configuration mechanism are unchanged by repair3; retain their original actual positives/negative/public-boundary evidence. The prepared documentation/rebase does not repin an old installer executable to a new producer HEAD.

| AC | Result on this freeze | Tasks / evidence |
|---|---|---|
| AC-01 | pass, retained | T01/03 provenance/resources/history; rebase non-Plan equivalence and exact five sources. |
| AC-02 | pass, retained | T02/04 neutral local SDK/!Send/provider/DTO/DAG and unchanged type/source single owner. |
| AC-03 | pass, retained | T02/04/05/07 explicit host and repaired file-worker launch propagation; R2 real configured/no-config wire proof retained. |
| AC-04 | pass, updated consumer check | T06 HTTP/nonHTTP/restricted shared generation and885 semantics unchanged; generic merged emitter byte-equivalent, M2 standalone shim fixed without widening production contract. |
| AC-05 | pass, retained | T07 Man/UI actual composition/worker/proxy/commands and frozen consumer source conditions. |
| AC-06 | pass, retained | T05 VM/Rust wire/policy/error/file/upload/SSE and configured CLI bounds; M3 native preflight remains strict. |
| AC-07 | pass, retained | T02/05 owned resource/body/terminal/network/logger lifecycle; strict workload descendant policy retained and independently challenged. |
| AC-08 | pass, updated | T08 shared original-ID/floor/migration actual-PASS defenses unchanged; exact catalog conservation/new leaf owner, managed source preparation, M1/M1b/M2/M3 original oracle closure. |
| AC-09 | pass, bounded work evidence | T09 immutable package/source/distribution/rollback/original history evidence retained on its actual source cohort. Newly rebased/repaired current SDK full release certificate remains a real T12 gate, not inferred green. |
| AC-10 | pass, retained | T10 original314 Blog/API/cold/Vue15/native22 and UI866/2 actual outcomes scoped to unchanged inputs; no new GUI run or new combined-cohort certificate. |
| AC-11 | pass, retained | T04/06/08/09 domain exits/named Core bridges/media/shared ownership and source/resource contracts unchanged. |
| AC-12 | pass, updated | T10 no mandatory work-owned gap remains; diagnosis, targeted negative/positive oracles, health, canonical preparation and frozen proposal coherent. |
| AC-13 | pass | T11 this independent affected review binds current committed clean sources, all affected invariants and SD; earlier risk coverage retained. |
| AC-14 | partial / pending merge | T12 actual final gates, SDK distribution, landings, deposition/readback, archive/guards/cleanup not executed here. |

| V IDs | Current result / bounds |
|---|---|
| V-01, V-02 | pass; original provenance/metadata/package/source plus exact current/rebase identities. |
| V-03, V-04 | pass; explicit/local host, !Send, owned requests/body/source and prior actual R2 closure unchanged. |
| V-05, V-07 | pass; HTTP/shared/nonHTTP generation preserved; M2 generic emitter restoration/actual original corpus legs. |
| V-06, V-08, V-09, V-10 | pass; original consumers/wire/policy/resource proofs reused on unchanged product mechanisms; M1b workload policy remains strict. |
| V-11 | pass, updated; real preparation consumers/negative oracles, catalog owner/conservation and original receipt execution guards. |
| V-12, V-13 | pass for bounded work oracles; original SDK/transaction/history proofs retained; actual current combined release remains NOT_PASSED T12. |
| V-14, V-15, V-16 | pass, retained; fixed original Blog/UI/consumer/migration/resource/history boundaries unchanged. |
| V-17 | pass, updated; M1/M1b/M2/M3 actual failure closure, warning/format/omission/workaround review. |
| V-18 | pass review portion; partial overall because actual merge/deposition/archive/cleanup gates remain pending. |

##### Spec delta and prepared knowledge

Read main canonical sources and all changed prepared canonical/navigation/ledger/index paths. Their hashes are frozen in baseline. SD01..09 remain within approved r6 intent, with repair3 SD07/08 clarification; no new component, dependency edge, host capability, platform, protocol, golden/floor/eligibility change or invented goal is introduced.

| SD | Decision / affected sources |
|---|---|
| SD-01 | pass; Lang auto-web pointer/project and actual independent five-page Web canonical owner/version. Overview correctly puts sibling Web outside workspace crates. |
| SD-02 | pass; repository-boundaries web-four-repository-delivery retains original source-bound IDs/floors/migration actual execution and real future certification gates. |
| SD-03 | pass; language-sdk overview/architecture/project/http-host-port/plans match local VM ownership, explicit installation, typed launch/thread binding and R2's accepted real behavior. |
| SD-04 | pass; API generation/shared carrier/media/Man/compiler/tools owner edits preserve existing885 semantics and restrict HTTP/nonHTTP split; auto-generation-core navigation is explicit, distinct from auto-gen-core. |
| SD-05 | pass; stdlib backend/server/deployment mapping preserves old supported tracks/budgets and points to Web; no universal TLS/WS/platform or new wire promise. |
| SD-06 | pass; four-repository consumer pointers, current landing remains conditional. |
| SD-07 | pass, updated; governance-and-inventory adds source-bound preparation, no workload-leak waiver, original guest File forwarder/owned target and exact native instruction boundaries. Conditions/original baselines remain unchanged. |
| SD-08 | pass, updated; Web's five pages and language overview/index source/group/module-plans prepared consistently; final current release/four-owner receipt wording stays pending. |
| SD-09 | pass; UI/Man integration pages and scoped original UI state alias fix retain actual source cohorts/commands and clearly distinguish later descendant mapping from binary provenance. |

Language compatibility-ledger comparison with e1600601f found no removed unrelated item; only the five P890 pointers are added. External Web/UI/Man ledgers have six sections and five scoped P890 items, no invented canonical goals; DELIVERY is pending. REVIEW's historical third-review text describes that actual receipt, not the final merge result. Merge must refresh actual review/landing records after its own gates. Future archive links remain planned targets requiring landing readback. Prepared Man card's prior437lines and compiler content are preserved; no unrelated migration/semantic deletion was found. `scripts/spec-index.py` adds only auto-web group navigation; existing unrelated ungrouped warnings are retained. Explicit user-required ledger/index/module-plans deposition is not waived by the installed skill's generic cache text.

No canonical source or ledger was written by this reviewer. Frozen SD is a reviewed proposal/prepared-state contract; it is not a report of already completed merge-owned operations.

##### Route and remaining limits

M1/M1b/M2/M3 are **closed for this exact candidate**. No task reopening or new debt is required by this affected review. Parent may record T11 complete/reviewed11/12 and route to T12. Repair3/3 is the completed repair cycle; this review does not authorize another repair or cover future implementation.

The following remain **NOT_PASSED, merge-owned**: actual frozen affected complete tm leaf/consumer/integration union; real four-owner/current SDK release/history gates on the current combined cohort; due main-only single-instance periodic batch; every actual repository landing and recovery-point/source readback; canonical Specs/explicit ledger/index/module-plans deposition and archive-link readback; Plan archive/count finalization, each mandatory guard/worktree cleanup, and user-policy evidence accounting. Earlier partial reds cannot be replaced by this acceptance PASS or historical45/history14/UI totals. Main due-batch rules and original ignored/manual/platform/milestone conditions remain unchanged. No global campaign, GUI, retention deletion or remote publication was run during this review.


</details>

<a id="e890-15513e7611658e98"></a>

<details>
<summary>review\t14-frozen-spec-delta.md · SHA256 406de25245818691de6d48b33951302297af59e6d0280642c3d7e8617b163b48</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


##### r6 repair clarification (R1/R2; same existing contract)

SD-02/07/08: the final four-owner consumer requires the full explicit registered
recipe/module union, source/dependency/effective-configuration identity and
original conditional applicability. Required owner cwd, list/run command,
listing/output path and hash, selected IDs and actual per-ID results must agree.
A self-reported success banner/count cannot replace execution proof. Public tm
may refresh into an ephemeral catalog; its source contracts are revalidated,
not weakened or rejected merely because published metadata bytes differ.
Original negative Case oracles can legitimately produce a raw exit1/FAIL while
their explicit meta-oracle passes; those source/oracle identities remain checked.

Distribution command-driven checks require their actual command/expected and
actual exit/output identity. Runtime/detail and exact-input SDK reuse retain
their own explicit schemas; they are not excuses to omit a required command.
Current-owner packaged SDK/crate/installer inputs are reconciled with the
distribution's frozen source cohort at the public combined consumer. A valid
historical receipt does not certify changed SDK bytes. Both periodic manifest
consumers keep every current registered recipe; responsibility/purpose advice
never changes original registration, conditions, samples, budgets or floors.

SD-03/05/08: Core's file entry (`run_file`/`run_file_with_args` through
`script_entry::run_with_path`) captures the neutral invocation launch before
spawning its execution thread and rebinds it on that worker, like the existing
run bridge. An explicit absent launch remains absent; ordinary no-service files
need no Web provider. Web `run --config` consumes the declared loopback/address,
port, profile/config identity and policy even when legacy environment values
differ. Existing no-config legacy behavior remains. No new protocol, network
product, platform promise, package or dependency edge is introduced.

Evidence remains source-bound and scoped: current rebuilt SDK217/2, fresh and
relocated CLI configuration/Host controls, original unaffected generator/Man/UI
proofs, and explicit final current-cohort release/four-owner gates pending under
T-12. Do not deposit a new aggregate PASS before that frozen combination actually
executes. The known first failures and original manual/ignored conditions remain.

##### Repair cycle 2 clarification (r6 unchanged contract)

SD-02 / SD-07 / AC-08/12: source-bound full-owner receipt admission and the execution producer share the same baseline/migration/retirement/frozen-floor selection guard. Matching list/run/IDs/count/hash alone cannot remove an original required ID or lower an owner below its original floor. Consumer migration credit comes from original required-source mappings and is legal only when the exact target IDs actually passed in the already validated complete module union, not merely appeared in the catalog/selection or a receipt count. Language/UI source baselines and Man/UI nextest/libtest floors preserve their original applicability and eligibility. No floor, original ignore, condition, sample or milestone is relaxed.

Implementation: scripts/run-full-regression.py::validate_owner_selection is shared by primary and extra execution legs and scripts/test-governance/web_delivery.py::validate_four_receipt. Required original UI/language IDs and Man/UI libtest floor controls include internally consistent list/output/count/hash mutations. The public fixture verifies failure propagation. This is the original r6 proof contract; no new cryptographic/log authenticity, performance, platform or product promise.

SD-03/05 and all unaffected owner, lifecycle, generator, artifact and Blog clauses retain the repair1 proposal and their explicit bounded evidence. SD-08 publication remains after independent acceptance and final required current-cohort/module/four-owner gates. Current canonical documents and live ledger/index remain unwritten at work stage.

UI single-process applicability detail: ui-tu keeps the original cargo-test single-process command. Because libtest --list omits ignore flags, its original ignored identities are reconciled against the already proved same-source ui-full nextest listing and recorded as condition_excluded. Exact selected IDs and actual passed output exclude those identities; arbitrary self-reported exclusions reject. The existing ui-tu floor stays863, and the genuine current UI866-pass/two-original-ignore result remains consistent. Ignored bodies are not counted as executed or passed.

##### Repair cycle 3 addendum (r6, existing AC/task scope)

Keep SD01..09 and repair2 acceptance unchanged. Canonical source preparation already exists in the four authoritative worktrees from the interrupted merge stage; it has not landed. This repair clarifies SD07/08 in existing governance-and-inventory.md: managed a2x worker preparation/source identity and explicit path; no workload leak exceptions; existing AAVM guest runtime shim forwards to its prior implementation under the emitted public namespace; owned guest target/cache; real CALL_CLOSURE/future instruction boundaries preserve strict native reference verification. No generic transpiler change, new runtime API, provider capability, dependency edge, test eligibility, baseline/floor, golden, sample, milestone or platform promise.

Core disassembler consumes the existing engine's one-byte closure argc; reference closure uses the same decoder for four-byte future names. Two permanent controls prove both valid acceptance and invalid-successor rejection. Original golden reference error now identifies the case and invalid byte. This repairs existing verification behavior rather than adding a new public Spec component. Canonical global overview places independent Web outside workspace crates and adds the restricted existing auto-generation-core map entry (SD01/04/08 navigation only). Final gate/ledger report pointers stay pending until actual merge-owned execution and landing readback.


</details>

<a id="e890-c5df8552bc6c8084"></a>

<details>
<summary>review\t15-fixed-groups-r7.md · SHA256 7fa47e2031a4d33f18e49ff763bde1b6bb0f414d7df290de797ba633c0ca0e03</summary>

#### PLAN-890:r7 fixed-group affected independent review

Outcome: **needs_fix**. One new work-owned finding F15-1 is established on the frozen candidate below. No final acceptance PASS; no current four-owner/SDK gate is certified. This is an independent affected re-review after the user-authorized896 integration, not another implementation repair or a reset of historical review/repair counts.

`stage=review; review_scope=final affected; reviewer_context=/root/plan890_final_review; plan_revision=7; reviewed_commit=5c7e449af9858e6f3619e2a9785ad98d2efe7b88; upstream_base=323a7b3a4; outcome=needs_fix`.

##### Authority and revision binding

Read the main checkout's r7 Plan (execution_done10/12 at review entry), new fixed adapter, native producer/consumer receipt paths, fixed-runner diff, changed test/config/Spec sources,896 integration checkpoint and actual evidence. Main Plan input SHA256 at this review:129364944f12824bea5ce2876f7efcf678481da7d4c121df4b2c8741f9097a49. The user explicitly superseded old tm selection with896 cargo t; retired catalog execution is not a review or merge prerequisite.

The initial request named87f7058dd. Actual readback found a subsequent no-conflict rebase to5c7e449af; the parent explicitly established the latter as authoritative. Comparing87f→5c7 showed only Plan/checkpoint docs and upstream Cargo.toml's standalone-fixture exclusion plus non-workspace dev debug=false. These build settings invalidate any assertion that the old daily receipt covers the final full environment; final unified runs must use the new freeze. Implementation sources otherwise retain that adapter patch. Range-diff is preserved at work/integration-896/landing-base-range-diff.txt.

All trees are the existing `D:/autostack/.wt/lang-890/<repo>`, independently read as clean before the finding was routed:

| Repo | Exact HEAD | Branch |
|---|---|---|
| Lang | 5c7e449af9858e6f3619e2a9785ad98d2efe7b88 | plan-890-dev |
| Web | 16d8378297f3a1c1fcf9b39894e842e3251742ec | codex/plan-890-dev |
| UI | 5cfb5c8ebfd98f3b4222f692652c371cd159eee7 | codex/plan-890-dev |
| Man | 6633feb4817a3bb526d44219899b8e93864d523b | codex/plan-890-dev |
| Down | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | detached |

External diffs from t14's pins are lock-only (Web5lines, UI1, Man1), incorporating auto-cache's auto-tool-execution edge. They are real dependency/config changes, not new runtime behavior or unchanged-source release claims. New source SDK closure has20 packages; historical19-package evidence cannot certify this cohort. Work acknowledged the stale19 documentation needs correction and current SDK execution belongs to final merge gates.

Frozen Git-blob SHA256 inputs (avoids confusing CRLF worktree bytes with Git bytes):

| Source | SHA256 |
|---|---|
| four_repository.py | e012e5fb3cb831cbb51a072078a2b0a020570d6c114f9de189c99e68d129cea9 |
| fixed-tests.py | 0a58b7c681961dca9dc1a5870322eb0efdfc0cea3f1919f0de41796aca5ab34a |
| test_four_repository_fixed.py | f9bb1a54eb3670bfa371214d4ec87370474f1a63c9a6cf3c93ab3ff081f582f2 |
| test-groups.toml | 9b078d30f4448087efe169c2163ff8ac8bc653f0743eb82094a9b82630e56cfe |
| web-repository-fixture.py | f13f27f8c7edf8649778602cd16e18864d4d1680d330c4ef2d583d78cb18c960 |
| governance-and-inventory.md | 03715dcd9e30090efbea9127ad62a80dd28f628447025ab049ff6837be0e47db |
| web-four-repository-delivery.md | 15ce6a4f194e214c1e635906bffdd01d206b583000fbd66b0757a88fa4b6473c |

No implementation, canonical, Plan, staging or commit changes were made by this reviewer. Only this Markdown receipt is written. Work is explicitly resuming after the finding; that does not change this receipt's frozen-source verdict. No independent model/session identity is fabricated.

##### F15-1: negative oracle randomly removes a legal extra ID

**P2; affected T08 and dependent T10; AC08/12/13, V11/17/18.** `scripts/test_four_repository_fixed.py:34` chooses `victim=next(iter(selected))` after unioning100 arbitrary legal extra IDs with original required UI IDs. A legal extra can be chosen. Deleting it correctly remains acceptable to the real owner-selection guard, but the test incorrectly requires a baseline-loss error. This makes a required permanent control nondeterministically fail under Python hash randomization and can break `cargo t web-delivery-controls`/future all selection without a product regression.

Concrete original setup: full source baseline/required minus media moves,100 additional `current\textra<N>` identities, complete actual media credit. With `PYTHONHASHSEED=1`, victim is **current\textra26**, outside the original required denominator. The valid initial selection passes; the bad-oracle deletion is accepted by the actual guard, as it should be.

Exact replay from the authoritative Lang tree, no compilation:

```python
import os, subprocess, sys
r = subprocess.run(
    [sys.executable, "scripts/test_four_repository_fixed.py",
     "FourOwnerControls.test_ui_required_and_actual_media_credit_are_both_required"],
    env={**os.environ, "PYTHONHASHSEED": "1"})
print(r.returncode)
```

Actual: exit1, `AssertionError: ValueError not raised`, at the `assertRaisesRegex(ValueError,'baseline coverage lost')` context. One test ran0.077s. This is a faulty negative fixture, not evidence that `owner_selection` permits dropping an original required ID. The preceding ordinary10-test run passed0.072s; one passing random process does not close the invariant.

Closing oracle: choose a deterministic victim from **required-minus-moved**; deleting it must reject through real owner_selection. Separately delete one known extra identity and require acceptance. Re-run the actual test/10 adapter controls with seed1 and representative other seeds, retaining migration-credit and valid-source positives. No full owner/SDK/ACE/global campaign is needed to repair this fixture. Review the resulting committed source and any changed source/Spec metadata before issuing PASS.

Parent accepted this finding and is returning to Work only for the negative-oracle repair. Preserve unaffected tasks; reopen T08 and dependent T10/T11 readiness as necessary. This review does not edit shared Plan state.

##### Verified affected invariants and evidence

- Actual `python scripts/test_four_repository_fixed.py`:10passed/0.072s before the independent seed mutation. `python -m unittest scripts.tests.test_fixed_groups`:37passed as part of the first combined command. The attempted combined import `scripts.test_four_repository_fixed` produced a loader ImportError before that file ran; corrected direct file invocation above ran its10 actual controls. The import setup error is not a product red.
- Independently exercised real `require_clean` in a temporary committed Git repository: clean source accepted; adding untracked scripts/new_runtime.py rejected with `frozen source contains tracked or untracked WIP`. This closes the dirty-development probe where snapshot and the former full.require_clean_tree alone accepted it. New guard is called at run entry, completion and public admission; snapshot covers all tracked non-document inputs, including Man's non-crates paths.
- Called real `validate_layout` against all four Cargo workspaces with `cargo metadata --locked --offline --no-deps`; selected group accepted. Its permanent wrong-owner negative rejects an ambient dependency path. Actual Cargo metadata confirms Lang/UI/Man auto-web and Web/Core dependencies resolve the selected sibling trees. Down's resource owner exists and is separately snapshotted; it is not invented as a Cargo workspace.
- Independently called new native_terminals with valid suite/test started→ok protocol: accepted. Bare ok, absent suite end, missing terminal, extra terminal and failed terminal all reject. The adapter adds these receipt-admission checks without changing896's general fixed terminal parser. No test-body success is inferred from this protocol fixture.
- Source inspection follows fixed producer→stored listing/native command and stdout→fresh selected-source enumeration→receipt admission→public web-repository-fixture. Command/cwd/exit/process-error checks, duplicate/missing legs, nonzero/ignored/extra terminal checks, source_state/config/invocation equality, local output bounds, wrapper bound-input rederivation and actual child trace validation remain real. No receipt consumer consults the old leaf catalog on this path.
- Original Web115 active migration destinations cannot disappear consistently; original two ignores remain excluded. UI original baseline/retirement/migration and original floors are preserved. Media credit comes from actual validated fixed-language PASS IDs, not counts/catalog presence. UI libtest original ignore reconciliation uses the same-source full listing. Man original floor remains shared by producer and consumer. Historical116ID/696row conservation is checked against actual enumerated target identities and original Git manifests. The fixture's independent source/resource checks retain17 original resources and original provenance; they do not fabricate execution.
- Selected fixed language groups are explicitly integration, auto-generation-core, auto-api-gen, auto-media-core, a2r-std, auto-test-case. Original Web/UI/Man commands remain owner legs; no blanket ACE or new global consumer graph is inferred. Shared-generation-core's new full package group is present. Native HTTP sequencing/resource config remains original. Current four-owner execution has not yet occurred.
- Independently hash-checked all270 integration-index entries/12,854,771bytes: zero missing/size/hash mismatches. Daily-final2970PASS, its106 original exclusions, prior2964PASS/6FAIL, targeted5+1 closures and initial owned-target refusal remain source-bound distinct receipts. The daily green predates the final rebase profile/config and cannot certify the new combined freeze.
- FFI parity helper repair preserves original VM/oracle/golden comparisons, explicitly owns target and requires a real executable before cache reuse. HTTP source audit now covers actual Web owner plus neutral Core and retains the original exact one legacy point/unmarked-call rejection. Generic Rust emitter and HTTP runtime mechanisms are not weakened to repair these test paths. `git diff --check 323a7b3a4..5c7e449af` passed.

##### Remaining affected verification and AC/V/Spec map

New moved ACE rust_lowering source preserves890's explicit prepared-worker selection at two integration points in the896-owned conformance path. Old r6 a2x actual runs do not automatically certify this new owner/compiler composition. Work is running a targeted new-owner check (`cargo t ace-workspace --test-name rust_lowering`, session76961 at notification); no result was independently available when this receipt closed. Its original output must remain, including source-state invalidation if the planned fixture repair overlaps. This is **pending targeted Work evidence**, not a passed milestone or a demand to execute all ACE. Re-review should inspect that actual targeted result and current worker/compiler/source identity.

| IDs | Current result and scope |
|---|---|
| AC01/02/03/04/05/06/07/10/11 | Retain applicable t14/r6 mechanism evidence where product/source/input paths are unchanged; review896 source/conflict and lock/config deltas explicitly. No current-cohort binary/SDK/GUI certificate is inferred. New full configured runtime/owner execution remains merge-owned. |
| AC08 | **fail F15-1**; new source/command/terminal/migration guards inspected and their meaningful controls pass, but the permanent original-ID loss oracle is unstable. |
| AC09 | partial/current release pending;20-package source closure and changed dependencies require actual current SDK/distribution/history gate, not old19/217/45 receipt relabeling. |
| AC12 | **fail F15-1**; readiness cannot claim all work-owned oracles complete; moved ACE targeted evidence also pending. |
| AC13 | independent review completed **needs_fix**; not final accepted. |
| AC14 | partial, merge-owned operations unperformed. |
| V01..10, V14..16 | Retain previously accepted unchanged mechanism coverage plus inspected current integration deltas; runtime/source-cohort limitations above apply. |
| V11/V17 | **fail F15-1**; public fixed adapter/negative-oracle stability needs repair; changed ACE owner evidence pending. |
| V12/V13 | partial/current combined SDK gate pending; original mechanism evidence retained on its real historical input. |
| V18 | review complete needs_fix; merge portion pending. |
| SD01/03/04/05/06/09 | Prior enduring owner/local-host/generation/config/consumer semantics retained within unchanged mechanism scope. No new runtime API or platform promise found. |
| SD02/07/08 | Current fixed-group/native producer/consumer proposal is consistent with the user-directed896 switch, but **partial pending F15-1/ACE evidence and20-package documentation correction**; no new aggregate delivered claim may be deposited. |

Changed canonical governance-and-inventory and four-repository-delivery texts correctly replace tm/runtime-catalog claims with six fixed language groups/native owners, actual execution-derived credit, original baseline/floor/ignore preservation, and distinct future gates. Upstream standalone-fixture exclusion/dev debug configuration is a real source/build input, not reason to relax acceptance. No new product scope, canonical goals or permission to delete original source/test obligations is introduced. This report freezes the two changed Spec input hashes above; their post-repair proposals must be rechecked, not silently treated as covered by this failed review.

##### Route

Return to Work for F15-1, complete the targeted changed ACE-owner evidence, reconcile20-package current source documentation, commit and request bounded affected re-review. Do not repeat unaffected bodies or a global suite merely because this receipt is needs_fix. Parent owns Plan/status/debt bookkeeping.

The actual frozen six-language-group union, Web/UI/Man four-owner run, current20-package SDK/release/history recovery, due main-only periodic batch, every repository landing and canonical/ledger/index/readback, archive and all guards/cleanup remain **NOT_PASSED merge-owned**. They cannot repair a work-owned oracle defect by themselves, and this failed receipt never covers future source changes. No implementation repair, compilation/global execution, GUI, retention deletion, canonical publication or remote operation was performed by this reviewer.


</details>

<a id="e890-1ace4ed8791579cb"></a>

<details>
<summary>review\t16-fixed-groups-r7.md · SHA256 1539501b39e6a9493f8bff05fb596593e48b0f757ffa6537e8c9c34486a3a97f</summary>

#### PLAN-890:r7 bounded independent closure review

Outcome: **needs_fix (interim)** on38de49426. F15-1, changed ACE-owner verification and Web integration selection are closed, but Work then identified three mandatory r7 distribution/provenance gaps before this draft was published. The preliminary PASS is withdrawn; the final frozen revision and bounded closure review are pending. Actual complete six-group/eight-leg/current-SDK gates remain NOT_PASSED, merge-owned.

`stage=review; review_scope=final affected closure; reviewer_context=/root/plan890_final_review; plan_revision=7; reviewed_commit=38de494266324e49afd19f9c187990f428b26d05; affected_base=5c7e449af9858e6f3619e2a9785ad98d2efe7b88; outcome=needs_fix`.

This is the assigned independent context, not the implementation context. No model/session identity is invented. Same authoritative trees were used; no new checkout/agent, source fix, Plan/canonical write, staging, commit, compilation/global campaign, GUI, retention operation or remote action was performed by the reviewer. Only this Markdown receipt is written. Main Plan r7 at entry is execution_done10/12; current Plan input SHA256 is2a34e2648a4da49797501db505a731db7821e241cb3a16e9beccfe90abfd4443. User-directed896 cargo t supersedes old tm selection requirements. Historical repair/review counts are not reset.

##### Final revision and source boundary

Independently read exact HEAD/clean state and checked the new all-input clean guard. All candidate paths are `D:/autostack/.wt/lang-890/<repo>`:

| Repository | Exact HEAD | Branch |
|---|---|---|
| Lang | 38de494266324e49afd19f9c187990f428b26d05 | plan-890-dev |
| Web | 07f54658b308e9784fd20101a560758f4ae9fe4b | codex/plan-890-dev |
| UI | 5cfb5c8ebfd98f3b4222f692652c371cd159eee7 | codex/plan-890-dev |
| Man | 6633feb4817a3bb526d44219899b8e93864d523b | codex/plan-890-dev |
| Down | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | detached |

The requested intermediate f79331598786849a9b610151a03a2171af127c55 did not receive PASS: Work identified that Web --lib omitted http_runtime/http_generation integration targets. Current38de adds --tests to that exact owner configuration; Web07f updates only testing/delivery wording. No previous receipt is relabeled to certify this later source. All five trees were independently clean at final readback. Actual `validate_layout` with four `cargo metadata --locked --offline --no-deps` calls passes against these selected roots, with Down separately checked as a resource owner.

Current fixed configuration SHA256:8fc772f5f6180bb8ed5cae8e635fe2d67d9fd60b40b13fc6b140ca94ac144690. Current fixed checkout_state:3c1acdd732dc52981ec461e7933cbe3eddc4935ebcb690f6b4381e0f6b424ae6. The final current10-control receipt matches both exactly. The earlier ACE six-case receipt matches f793 state5d828523277fed2ec409d290c73c3be2cdea0cd3d8982b3ce2c3b976d1b6231b; only the Python Web target option changed from f793 to38de. ACE source/lock/tool configuration/target/selected input/profile are byte-unchanged, so that scoped ACE result is reused; its older global state is not asserted equal to current.

##### F15-1 closure and fixed-runner controls

Actual diff replaces nondeterministic `next(iter(selected))` with `sorted(required-moved)[0]` and separately requires legal deletion of current\textra0 to remain accepted. It tests the existing rule, without changing the product guard or weakening required IDs/media execution credit. This directly fixes t15's seed1/current\textra26 counterexample.

Independent actual replays, from the authoritative Lang tree:

```python
import os, subprocess, sys
for seed in ("1", "2", "0", "17"):
    subprocess.run([sys.executable, "scripts/test_four_repository_fixed.py"],
                   env={**os.environ, "PYTHONHASHSEED": seed}, check=True)
```

Each process passed10/10, respectively0.094s,0.091s,0.079s,0.082s. An additional ordinary process passed10/10/0.087s. Required source-ID deletion rejects, legal extra deletion accepts, missing actual media credit rejects, original Man floor and original ignore/selection/command/source/terminal controls remain. The unchanged product guard is not incorrectly blamed for the original negative fixture's mistake.

Independent `python -m unittest scripts.tests.test_fixed_groups`: **37passed/0.988s**, exit0, against the new fixed configuration. No build or owner body was invoked by these controls. Work's initial concurrent lease conflict remains an environment/ownership record, not a product red or a substituted PASS.

Work's actual `cargo t web-delivery-controls` final current receipt is `work/integration-896/fixed-owner-controls-integrations/results.json`, SHA256 **449a8866237294fe873f0291ba0b5abf6788fbfdaa6e62f2d9c6270d1cb447eb**, gate_passed=true/executed10. Its source_state and config exactly match current values above. Earlier fixed-owner-controls-final receipt SHA8f31739c8228b5bd2d3040cd62660d82af248ae59998f3ffc58d1e1c74cceae0 binds f793; both were read as distinct receipts.

##### Changed ACE owner, lock and actual six-case evidence

The ACE lock is reconciled to Core's current pure-Web-contract dependency. Parsing old/new locks independently finds only auto-web-contract0.1.0 added and unreachable axum0.8.9, axum-core0.5.6, httpdate1.0.3, matchit0.8.4 and serde_path_to_error0.1.20 removed. Every retained package version/source/checksum is unchanged; dependency feature-edge pruning is consistent. No registry upgrade or alternative runtime implementation was introduced.

New fixed group ace-rust-lowering explicitly selects ace-conformance's single conformance target, **complete `tests::rust_lowering::` namespace**, with tools=[a2x], original nextest resource config and owned experimental/ace/target. It does not alter sample/oracle/ignore eligibility or weaken assertions. The filter names the actual Rust namespace, not the physical file's host directory; unrelated AV/native tools are not prepared to execute these six tests.

Read actual `work/integration-896/rust-lowering-final/results.json`, SHA256 **0fe7659e614c2b52d03b1a6066b92b88bf2f889840a2624c558e9ec678c4d952**: gate_passed=true,6 selected/executed/PASS,0 excluded, total8.809516s. Original nextest execution has exit0/meter.error=null, wall7.032321s, workloadCPU9.375s, Job peak455831552bytes,129processes. Its effective environment fingerprint is2ac8663a9d0c4190211e96cf6f017c671769d5bdf702c551226d11cd31a6bfb4. Actual worker path is `<Lang>/experimental/ace/target/release/a2x-worker.exe`; independently rehashed SHA256 **460b32dd86c4c63f0235d2adfb9397fdd62dd2eaafe48e681c9686240243efe9**, matching its preparation record. The adapter still sets explicit AC_A2X_HIR_WORKER; generic compiler ownership/target policy remains896's.

Exact selected IDs under `ace-conformance::conformance`:

- tests::rust_lowering::a2r_hir_certify_leg_all_cases_pass
- tests::rust_lowering::a2r_hir_certify_negative_controls_reject
- tests::rust_lowering::a2r_hir_first_case_green
- tests::rust_lowering::a2r_hir_missing_worker_is_infra_error
- tests::rust_lowering::a2r_hir_tampered_golden_is_red
- tests::rust_lowering::a2r_hir_worker_credential_pin

Assertions retain actual/reference compiled execution and closed-form values/traps, wrong oracle/reference rejection, original golden-byte comparison, missing worker infra ERROR and matching/mismatching worker credential control. This verifies the new owner/worker composition rather than merely reusing r6's old package name.

Independently parsed the actual discovery and raw native events through the real adapter protocol/terminal checker: exactly6 completed PASS/error=[], with3 known filtered synthetic started→ignored pairs correctly excluded. An initial reviewer parser invocation omitted the known-unselected set and correctly refused one filtered event; using the actual discovery-derived known-unselected identities restores the production admission contract without relaxing any assertion. Native stdout SHA256 **67122638067120a9ff32d2c9ab52f48a6144d5467d7aee4e688bbe2aff27ec89**, stderr **296a8a5a2812b25269592ccdd437302479ca571a0a93bc9716017bc32353a0ce**. No suite total is used in place of these actual six identities.

The first new-owner preparation failed on stale ACE Cargo.lock; the next wrong physical host namespace selected zero and failed. These records remain failures before the corrected full namespace, not ignored/N/A or overwritten green. No unrelated ACE/global body was rerun by this reviewer.

##### Final Web selection closure

Independently derived actual owner list/run commands from the frozen adapter. Both contain `-p auto-web -p auto-web-contract --lib --tests --features auto-web/test-http-e2e` and the original Web nextest config; execution retains --no-fail-fast/--no-tests fail and structured native protocol. Cargo metadata confirms the real integration targets **http_runtime** and **http_generation**, with test=true, alongside Web/contract libraries. Current source contains the original3 runtime and1 cold-generation integration checks; --tests includes them rather than relying on separately shipped SDK execution to replace owner obligations.

Producer and receipt consumer share these exact options, fresh-source enumeration, terminal proofs and original migration/floor rules. The single later option change is fully inspected; current control receipt binds it. This is selection/admission closure, **not** a claim that the final complete Web library/integration run has occurred. That real run is required in T12.

T15's other independently established closures remain applicable: tracked+untracked clean checks at entry/completion/admission, five-owner actual Cargo path binding, strict started/terminal/suite protocol, source-derived current denominator reconciliation, original115 active Web destinations/two ignores, source baseline/retirement/floor admission, actual media PASS credit, wrapper input+child proof,17 resources and116ID/696row provenance. No repair alters those mechanisms or fabricates a current full receipt.

##### Spec/source closure and frozen delta

The updated proposal is source-bound to the prepared canonical texts below. Enduring rules are unchanged: local VM/explicit Web host, actual source/command/output/ID/child proof, original eligibility and conditional final gates. R7 replaces tm wording with explicit896 fixed groups/native owners; it adds no runtime API, platform promise, invented canonical goal or permission to drop original obligations.

Reviewed clarification: current SDK first-party closure is **20 packages**, including896's auto-cache→auto-tool-execution dependency; original19-package receipts remain historical. Read actual `D:/autostack/plan-artifacts/plan-890/final-896/source-5c7e449af/source-manifest.json`, SHA256 **990800f3d5550ef1ca3216cb69b5a4d47a28f953addff0ed6f265933250fc996**. Its source_closure.packages has20 entries matching20 first-party sdk/crates manifests and includes auto-tool-execution. The vendor autodown-test-parser helper manifest is separately vendor-owned, not an extra first-party SDK package. Actual auto-cache manifest uses the included ../auto-tool-execution path. This inspection confirms the count/dependency, not a new current-HEAD binary/source release PASS; the exporter must regenerate/rebind the final current cohort before real SDK gates.

The changed language boundary project/delivery pages and Web delivery page say20; Web testing explicitly names both real integration targets under --lib --tests, and testing/delivery replace retired module-union wording with fixed cargo t selection. Delivery and final landed-cohort claims stay conditional. No canonical publication or ledger/index operation was performed here.

Frozen actual worktree-byte SHA256 inputs (LF Markdown receipt; no separate raw artifact added):

| Input | SHA256 |
|---|---|
| scripts/test_four_repository_fixed.py | fc4a2d62afa769f8f9abe6ddbebf082cd8b87c4ce3adcf2357a281e0ae8bc279 |
| scripts/test-groups.toml | 8fc772f5f6180bb8ed5cae8e635fe2d67d9fd60b40b13fc6b140ca94ac144690 |
| scripts/test-governance/four_repository.py | 612c32bd57471fe9d3c692f5915a25bef350d1c75bd46002b406a665cf6e70c2 |
| experimental/ace/Cargo.lock | 01e7f9b559c409b40a11a0a08e09ba15fa46f5328ba28506ddd42a071d5e83dc |
| Lang docs/specs/repository-boundaries/project.md | 0c3dae0710ba0af6ffa323a8eae258d9e9faf5f0fb9770f02e61c4799952ceec |
| Lang docs/specs/repository-boundaries/web-four-repository-delivery.md | 9aedb08a41d3c25daf2c60ab98724f0d9ee03028292069c31f27c4df39c38837 |
| Lang docs/specs/auto-test-case/governance-and-inventory.md | 03715dcd9e30090efbea9127ad62a80dd28f628447025ab049ff6837be0e47db |
| Web docs/specs/testing.md | 6cb16ff165ac7488907cffed435e9abd4e588351aa6406a19ef55181ad8b5627 |
| Web docs/specs/delivery.md | 1361b381fd3a06f9a66c09ee5dd2711936debfca9421e9dca79f02db988da57f |

`git diff --check 5c7e449af..38de49426` passed. These are prepared Spec/source hashes; Git canonical publication/ledger/index/readback remains a later operation.

##### Current acceptance and route

| IDs | Verdict / evidence boundary |
|---|---|
| AC01/02/03/04/05/06/07/10/11 | pass, retained within unchanged product/mechanism/input scope from t14 plus r7 source/conflict/lock/config review in t15; no old binary recertification. |
| AC08 | **pass, updated**: F15-1 deterministic original-required negative and legal extra positive; real original-ID/floor/credit guards retained; full ACE namespace actual six checks; Web integration selection corrected. |
| AC09 | **fail / Work adaptation pending**: current20-package source count is verified, but consumer20-package validation, Man core locked closure and independent consumer-vs-installer provenance still require repair. |
| AC12 | **fail**: admitted distribution/provenance work gaps remain. |
| AC13 | review ongoing needs_fix; no final PASS on38de. |
| AC14 | partial, merge-owned execution/landing/closeout pending. |
| V01..10, V14..16 | retain accepted mechanism coverage and t15 r7 integration review; changed Web selection and ACE owner input scope verified above. |
| V11/V17 | **pass, updated**: repaired permanent oracle, actual changed-owner six-case proof, fixed-group/native selection and source/Spec consistency. |
| V12/V13 | pass for bounded work mechanisms/source closure; actual current-cohort SDK/distribution/history gate still pending. |
| V18 | pass independent-review portion; merge portion remains partial/pending. |
| SD01/03/04/05/06/09 | pass proposal, retained scope/ownership/host/generation/config/consumer decisions; actual cohort recorded only after gates/landing. |
| SD02/07/08 | **pass, updated**: fixed-group/native receipts, original denominators/conditions,20-package count and both Web integration targets agree with current source; final delivered receipt claims remain conditional. |

F15-1 is closed. Return the distribution/provenance adaptations below to Work and dependent readiness/review; no reviewed11/12 may be recorded for38de on the basis of this draft. T15 remains historical.

Before landing, the merge owner must actually execute the **six language fixed groups plus all eight external native legs**, including both Web integration targets, on the frozen final inputs; regenerate/current-bind and execute the **20-package SDK/distribution/history recovery** gate; run any due main-only periodic batch; then complete all repository landings, canonical/explicit compatibility-ledger/index/module-plans readback, archive/count bookkeeping and mandatory worktree guards/cleanup. Every one remains **NOT_PASSED merge-owned** here. Historical2970 daily,6ACE,10controls,37fixed,45distribution/history14 or UI counts do not collectively substitute for that unified current execution. No old tm campaign is required or authorized by this receipt. Future implementation changes invalidate affected acceptance and require appropriate review.

##### Subsequent Work-owned gaps — preliminary PASS withdrawn

Before the draft conclusion was published, Work identified three r7 adaptations that cannot be deferred to merely running the final SDK gate: (1) distribution receipt admission still required19 packages despite the actual20-package closure; it must strictly require20 and auto-tool-execution; (2) Man core/Cargo.lock remained stale for the new cache/tool dependency and rejected locked metadata; (3) the old recovery consumer conflated the immutable cb4ab3d installer snapshot with the current complete Man Core CLI source. The latter requires separately source-bound consumer_sources.auto-man, exact consumer Git archive/current core/crates/scripts/lock reconciliation, while preserving the installer's immutable artifact pin.

Work is repairing these with real consumer negative controls and will provide a new frozen revision. These are current work-owned acceptance gaps, not reasons to waive source binding or certify the wrong runtime from future gate totals. This receipt is interim needs_fix until those changes are independently reviewed; earlier closed checks and raw evidence above remain reusable only within their stated unchanged scope. No source repair was performed by the reviewer.


</details>

<a id="e890-74707f31f0dcaf7a"></a>

<details>
<summary>review\t17-fixed-groups-r7.md · SHA256 d68aaab1cb88030183a6bd14289866b6cdd41c0f4d1c625720a1c6d85385b0f7</summary>

#### PLAN-890:r7 final affected closure review

Outcome: **PASS** for work-owned acceptance and the affected Spec delta on the exact final freeze below. F15-1 and the subsequently admitted distribution/consumer/lock/selection gaps are closed. T15 and T16 remain historical needs_fix records. This PASS does not certify the pending complete current-cohort execution or future implementation.

`stage=review; review_scope=final affected closure; reviewer_context=/root/plan890_final_review; plan_revision=7; reviewed_commit=93ddf4a05501e0792ae0f6d664a4070a6d739999; upstream_base=e9bfa55dc0d8484e2576d569500c65a728b63ef2; outcome=pass`.

The assigned independent context is separate from implementation; no model/session identity is invented. Read the current main Plan, prior review history, actual source/diffs/receipts and prepared canonical changes. Main Plan input SHA256:db3739ad0ef56bccdb5a139117cbc332c6d29f9b5cdcfe0e2d64fc863763f354, r7 execution_done10/12. User-directed896 fixed cargo t supersedes the old tm contract. Historical repair/review counts are retained. No implementation, Plan/canonical, staging/commit, new checkout/agent, compilation/global campaign, GUI, retention or remote operation was performed by the reviewer. Only this LF Markdown report is written.

##### Exact final freeze

All existing paths are `D:/autostack/.wt/lang-890/<repo>`. Independently read HEAD/status and ran the adapter's current all-input clean guard; every tree is clean:

| Repository | Exact HEAD | Branch |
|---|---|---|
| Lang | 93ddf4a05501e0792ae0f6d664a4070a6d739999 | plan-890-dev |
| Web | ab09fd8842dbfb4a1062d32886609f33dd664bb4 | codex/plan-890-dev |
| UI | d1424556b167b225c260635f30d9e941cf508c34 | codex/plan-890-dev |
| Man | 43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6 | codex/plan-890-dev |
| Down | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | detached |

Earlier8372/477180 revisions were not silently equated to this freeze. Upstream897 added real Core/stdlib/ZIP dependencies and later namespace-whitelist corrections; Work reconciled locks and reran the affected actual ACE bodies on the final code. This review checks the resulting source/dependency/gate interface; it is not a second functional review of separately delivered897. The actual rebase base also includes upstream e9bfa55dc's probe expectation update. Later main f940/3fa contains Plan documentation plus restoration of12 existing Cargo fixture-exclusion/dev-dependency-debug lines already present in this candidate. Parent reports the candidate/main Cargo difference is only890's generation-core member. Before any final rebase/landing, verify the full runtime-input byte mapping is empty; do not assume docs-only from commit labels. No new profile is covered by this review. It is not necessary to rebase merely to obtain this acceptance verdict.

Current fixed checkout_state is **56197c4a3a03c32dace9d9bfc8aa85458548816ea7936ac0aa408150458c1285**; fixed configuration SHA256 is **8fc772f5f6180bb8ed5cae8e635fe2d67d9fd60b40b13fc6b140ca94ac144690**. Both latest actual receipts below independently match these exact current values. No prior global source_state is relabeled current.

Independent lock parsing against pre897 pins: Web/ACE add20 dependency identities each; UI/Man/ManCore add17 each; **removed=[]**, with all original name/version/source/checksum tuples retained. Thus these are actual dependency-closure additions, not a hidden upgrade/replacement. Independently ran locked/offline/no-deps Cargo metadata on the four selected workspaces, Man core and ACE: all pass. The actual cross-repository path layout is checked against all selected roots. No compilation was triggered by these metadata reads.

| Final lock | Actual SHA256 |
|---|---|
| Web Cargo.lock | 838e2bcb1b190838ad36b38e4f4f85577c579666e8c9afd087d69f1f4fa2264d |
| UI Cargo.lock | aed59c7856fe00f2d81e6a9f135bd4d28631f4d9a37c56a1600b943bbbdf328a |
| Man Cargo.lock | c18d7636a8d6b97a426d5208c24dbe9d2a003fe6d5ba2f55385ecf371cce7e20 |
| Man core/Cargo.lock | 204e83f5b842a6932d127091a17789213bc4622649eea854b56cfa282fb4841e |
| ACE Cargo.lock | 3ecb96e956564351633513b6d07697986a34eedf5ec6d2ced365815e596123b2 |

##### Work-owned closures

**F15-1 is closed.** The negative chooses `sorted(required-moved)[0]`; legal deletion of current\textra0 is separately accepted. Independent seed1/2/0/17 runs each passed10 controls (0.094/0.091/0.079/0.082s). The selected-source/floor/media guards were not changed to accommodate the bad old oracle. Source bytes are unchanged by subsequent upstream rebases. The reviewer also reran37 fixed controls against the new configuration:37passed/0.988s. Original t15 seed1/current\textra26 failure remains valid historical evidence.

**Web integration omission is closed.** Frozen list/run contracts use `-p auto-web -p auto-web-contract --lib --tests --features auto-web/test-http-e2e` and the original nextest configuration/native protocol. Actual `work/integration-896/web-target-admission/list.stdout` SHA256 **3af0517914bf66d4f60129c012e4ab99c8f94b5b5f4c5a61d4b553f789c29c63** was independently parsed:221active/2original ignores, original115 active migration IDs admitted, and all four integration cases selected:

- http_runtime::configured_proxy_file_entry_enforces_host_policy
- http_runtime::installed_host_serves_exact_json_and_control_shutdown_joins
- http_runtime::phase_port_retains_installed_upload_limit_snapshot
- http_generation::cold_generated_rust_executes_and_missing_web_preserves_workspace

Those are actual Rust target identities, not inferred from separate SDK execution. Later changes are scripts/Specs/dependency locks, not Web Rust targets or selection options; exact selection remains bounded reusable evidence. Complete execution of those targets still belongs to the final four-owner gate.

**Changed ACE owner is actually verified on the final code.** New ace-rust-lowering fixed group selects the entire real `tests::rust_lowering::` namespace of ace-conformance's conformance target, tools=[a2x], original resource config and owned experimental/ace/target. It does not sample cases or change original goldens/oracles/conditions. The final actual receipt is:

`work/integration-896/rust-lowering-final-master/results.json`, SHA256 **cc576c32d7f5e485500ffb22eea5c2aa949e35a6f8c53843ce801c9bbe2f9f87**.

It has gate_passed=true, exactly6selected/executed/PASS, total15.991097s; actual execution exit0/meter.error=null, wall6.898128s, workloadCPU10.09375s, peakJob460128256bytes/129processes. Its effective environment hash isab47dc6f59757946ca960ae25b291695b7cd9b2657476e9282124c0ca7f24dbc. Independently parsed the source listing and actual native event stream through the real adapter:6completed/error=[], including all six original cases: certify positive, wrong-oracle/reference rejection, first original golden, missing worker ERROR, tampered golden FAIL, and credential mismatch/matching pin. Raw native stdout SHA256 **ca2bd79a6f2d7c9df3b5dbce8ec1c5c4f3b9657c2ffb7760c0d0cb676b308df4**, stderr **b204f9f618975e9bda3d3a5aa324810fd4dc693fd1d0c7e84472b07a9bee503b**.

This final actual replay supersedes older-source green only for the changed inputs, without overwriting the stale-lock/zero-selection/earlier results. No all-ACE or unrelated runtime campaign is required by this review.

**20-package distribution admission is closed.** The real consumer now requires exactly20 unique first-party package names and requires auto-tool-execution in addition to the original critical owners; the installed closure's observed count must agree. Count-preserving substitution of auto-tool-execution by a placeholder is rejected, not only a19-package count. The actual20-package source manifest and auto-cache path dependency were inspected in t16; that old5c7 source is evidence of count/dependency only, not a new current release certificate. The vendor autodown-test-parser helper remains separately vendor-owned, not a21st first-party package. Final source/third-party inventories must be regenerated for current897-integrated inputs before SDK execution.

**Current CLI consumer source is distinct from the immutable installer.** Producer records `consumer_sources.auto-man` from actual new-core-install source evidence. Consumer requires it, reconciles new Core install/snapshot cohorts and revision-bound archive proofs, and compares selected Man's Cargo manifests/locks/core/crates/scripts/LICENSE plus untracked files against that source. The public fixture now passes selected Man explicitly. The generic installer's cb4ab3dd5d411314cce10b129eeb3b433a11c466 pin remains unchanged; independent hash of its Git source, embedded file and declared manifest is exactly **7172bccb641e8c298b7595df8b913704375f75c7889e1e538bcf2fe303fb1066** for all three. No copied mutable installer or provenance weakening was introduced.

**Man Core lock closure is closed.** The initial missing auto-cache→auto-tool-execution edge was added, then897 transitive additions were synchronized as listed above. Actual current Man core locked/offline metadata passes. No required lock failure is deferred to the future runtime gate.

##### Actual admission tests and independent counterexamples

The latest registered `cargo t web-delivery-controls` receipt is `work/integration-896/controls-final-master/results.json`, SHA256 **ce879d448d53ccb1c43a91d5c22f36c6e5e3998e64d332bf49347a5814836a6f**. It reports actual11selected/executed/PASS, total1.004458s, exact current source_state/config, execution exit0/meter.error=null. Its effective environment hash is191228b15b184b6db254134d065d2497c7c911315b21331f69fe4c20eb99579a. New eleventh control invokes the existing real distribution consumer test method directly; it does not execute retired tm fixtures or certify synthetic command text as real SDK bodies.

Reviewer independently ran `python scripts/test_four_repository_fixed.py`:11passed/0.207s. Reviewed the actual consumer assertions, then extended that same temporary valid protocol fixture through the real `validate_distribution_receipt` with four independently chosen mutations:

| Input / mutation | Actual result |
|---|---|
| Complete20-package protocol fixture with separate CLI/installer pins | ACCEPT before and after mutations. |
| Replace required auto-tool-execution with sdk-placeholder, preserving20 entries/count | REJECT: installed source closure missing/duplicated/different. |
| Remove consumer_sources entirely | REJECT: current Core CLI consumer source missing. |
| Mock actual selected-Man added-files response as core/new_runtime.rs, leaving all other source responses empty | REJECT: current Core CLI consumer inputs differ. |
| Change actual retained old-source tar bytes, retaining original archive hash | REJECT: distribution output identity differs. |

This extension reuses the existing test's fixture construction and original assertions; only its outer function call is wrapped to add mutations. Each mutated input is restored before the valid control is rerun. No product assertion is mocked; the selected-Man source mutation substitutes only the external Git boundary response. The original permanent controls additionally reject installer-pin-as-current-CLI, changed core/Cargo.lock, consistent19-package deletion, missing commands/exits/logs, partial totals, archive identity errors and source changes. These are **protocol/admission controls**, not a fabricated45-check current runtime certificate.

T15's previously independently tested all-input clean guard, actual Cargo path ownership, started/test/suite protocol, self-consistent listing omission rejection, real migration credit, original UI/Man floors and ignores, wrapper input/child traces,115active Web moves/two ignores,17resources and116ID/696row provenance remain applicable on unchanged mechanism bytes. T14's R1/R2 runtime/source mechanisms are reused within their unchanged inputs; old global source identities/binaries are never repinned to this cohort.

##### Frozen source and Spec delta

Reviewed actual prepared boundary/delivery/testing text. The invariant is now explicit:20 first-party SDK packages; installer source pin identifies the immutable transaction script; the current rebuilt CLI has an independent source/archive/current-input proof; fixed cargo t six-language groups plus native Web/UI/Man owners; both HTTP integration targets remain mandatory. Original scope, floors/ignored/manual/platform/milestone conditions, expected runtime values and recovery obligations are retained. No new public API/platform promise or invented goal is introduced.

| Final input | Actual worktree-byte SHA256 |
|---|---|
| Lang scripts/test-governance/web_delivery.py | 1cf0e9600910c24a9ba8b9798c63d493a2670f61c3f98291de082730e71564cd |
| Lang scripts/test_four_repository_fixed.py | 3b68656a9aa611df8fc902cc330d6bc6f903705118c7eebcc87f872d87f73f7f |
| Lang scripts/web-repository-fixture.py | bde4f60b9dfd7aabe08b63f681fdd2e30392e676f91adcf02d5716f7a6be5716 |
| Lang scripts/test-governance/four_repository.py | 612c32bd57471fe9d3c692f5915a25bef350d1c75bd46002b406a665cf6e70c2 |
| Lang scripts/test-groups.toml | 8fc772f5f6180bb8ed5cae8e635fe2d67d9fd60b40b13fc6b140ca94ac144690 |
| Lang docs/specs/repository-boundaries/web-four-repository-delivery.md | 0cbfbc0da7955c8c7be0696be73c41d148974f5810ccd6ac9bf9537bef7264b4 |
| Web scripts/web-repository-distribution.py | 6032c24421e54143e9a883e6a8b93efad82688553f272bb1d3afd079ad940b20 |
| Web docs/specs/delivery.md | f3f3546375e9bfb70dac01564d5bbdb2aad487fc577591551d7634ffd049ecdb |
| Web docs/specs/testing.md | 6cb16ff165ac7488907cffed435e9abd4e588351aa6406a19ef55181ad8b5627 |

T16 freezes the unchanged20-package project and governance clarifications. This table freezes the later producer/consumer provenance addition and final native-selection text; those prepared sources describe current implementation with actual future gates conditional. `git diff --check` passed for the reviewed language/Web/Man deltas. No canonical publication or live ledger/index operation occurred here. Existing user-required compatibility deposition/readback is not waived.

##### AC/V/SD decision and route

| IDs | Current result / boundary |
|---|---|
| AC01/02/03/04/05/06/07/10/11 | pass, retained: t14's full risk/acceptance map plus t15/r7 source-conflict/config review; unchanged mechanism/input scope only. Upstream897 integration and new dependency identities are explicit, not an assertion of old binary equivalence. |
| AC08 | **pass, updated**: deterministic required-ID negative/legitimate-extra positive, original media execution credit/floors, complete ACE namespace actual6 and both Web integration targets. |
| AC09 | **pass for completed work-owned source/admission/recovery mechanisms**: strict20-package closure, independent CLI source, actual selected-Man/locked dependency binding. Complete current SDK/runtime/distribution/history certification remains T12 NOT_PASSED. |
| AC12 | **pass, updated**: no admitted work-owned gap remains; real targeted source/state/lock/oracle/Spec checks complete. |
| AC13 | **pass**: independent final affected acceptance binds this exact clean freeze. |
| AC14 | partial, merge-owned actual gates/landings/deposition/archive/guards/cleanup not executed. |
| V01..10, V14..16 | retain prior scoped accepted mechanisms and r7 integration inspection; changed source/lock/owner/selection evidence above resolves affected interfaces. |
| V11/V17 | **pass, updated**: current actual6/11 receipts, independent mutated-admission controls, original conditions and final source/Spec readiness. |
| V12/V13 | pass work-owned mechanisms/strict current-source admission; current complete20-package SDK and legacy/runtime aggregate remain pending. |
| V18 | pass review portion; merge portion remains partial. |
| SD01/03/04/05/06/09 | pass proposal, prior enduring owner/host/generation/config/consumer semantics retained within unchanged scope. Actual final landed cohort is recorded only after gates/landing. |
| SD02/07/08 | **pass, updated**: fixed native receipts, original denominator/condition preservation,20-package count, independent immutable-installer/current-CLI provenance and both Web integration targets match code. No delivered aggregate is anticipated. |

No work task needs reopening from this final affected review. Parent may record T11/reviewed11/12 for this freeze and route to T12. Historical needs_fix and initial stale-lock/zero-selection/lease/first-red evidence remain; they are not replaced by this acceptance PASS. The main docs-only897 descendant must be reconciled by source-range readback at landing; any future product change invalidates affected acceptance.

Still **NOT_PASSED, merge-owned**: execute the complete **six language fixed groups and all eight external native legs**, including both Web HTTP integration targets; regenerate/export current source/third-party inventories and execute the **current20-package SDK, release, transaction/recovery and old/new Core history/runtime gates**; run any due main-only periodic batch under current rules; then complete actual repository landings, canonical/compatibility-ledger/index/module-plans/readback, archive/count checks, each mandatory worktree guard and cleanup, and user-policy evidence accounting. Historical2970daily,6ACE,11admission,37fixed,45distribution/history14 or UI counts do not collectively substitute for that current unified execution. No retired tm campaign is required by this receipt. No remote publication is authorized or performed.


</details>

<a id="e890-a9be80225212b9a0"></a>

<details>
<summary>review\t18-final-gate-repair-r7.md · SHA256 4b50b42a80d3d1aca15d70b3a5b26e4af3b556fd0d0f6ded6fc050763125f098</summary>

#### PLAN-890:r7 independent review of first unified-gate repair

Outcome: **PASS**, for the affected work-owned repair and Spec delta on the final54b5 freeze below. The nine original failed atoms and the subsequently identified runtime-Schema source-guard gap are closed. The original failed unified gate remains failed. Complete current six-group/eight-leg/SDK/history execution and all landing/closeout operations remain **NOT_PASSED**, merge-owned. T17's previous PASS is historical; it did not certify this later repair.

`stage=review; review_scope=final affected; reviewer_context=/root/plan890_final_review; plan_revision=7; reviewed_commit=54b5d7c92f0244faecde929353b7e292ff6b3579; runtime_evidence_commit=4992de5fbcf01725673d15c03b34e9690442f291; repair_base=93ddf4a05501e0792ae0f6d664a4070a6d739999; integrated_main=3fa75dd9c; outcome=pass`.

Assigned independent context is separate from implementation. No invented model/session ID. Read main's current r7 Plan (frontmatter execution_done10/12, latest round42 readiness), handoff, original failed atoms, actual repair/consumer/Spec diffs and raw targeted results. Main Plan SHA256:70fe51646d8193d99e7012b1eb92fb3d5956d27be6788a2daa532dbfa3e394b2. Handoff repair-final-gate-r7.md SHA256:80b619ccd1f17feca66f6ce3c08fb7c70c925972abbad42a42f1c60ef6cf3d34.

No implementation/Plan/canonical changes, staging/commit, new checkout/agent, compiler invocation, global campaign, GUI, retention or remote operation was performed by this reviewer. Only this LF Markdown report is written. One existing compiled original assertion was run without a build, as documented below. User-directed896 fixed cargo t rules govern; no retired tm campaign is required.

##### Final input identity

Existing authoritative paths: `D:/autostack/.wt/lang-890/<repo>`. Independent HEAD/status/clean-guard reads confirm:

| Repository | Exact HEAD | Branch |
|---|---|---|
| Lang | 54b5d7c92f0244faecde929353b7e292ff6b3579 | plan-890-dev |
| Web | e144ec0d9febe74f8cf734744d91ade47fe57bcf | codex/plan-890-dev |
| UI | 4da2201fa46bd8a88bc5c2d661b21433b7ac84b5 | codex/plan-890-dev |
| Man | 43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6 | codex/plan-890-dev |
| Down | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | detached |

All are clean. Current fixed checkout_state **9154eb92b9ba992a2d7e3eeabaea8bb4a209c25d9124511c3874617cbf34941e** matches the latest13-control schema receipt exactly; configuration SHA256 remains8fc772f5f6180bb8ed5cae8e635fe2d67d9fd60b40b13fc6b140ca94ac144690. The earlier frozen-typed/proxy31/pins receipts bind the actual4992 runtime freeze state37485f39bb671bf5e4c7e16789f2111a5a34c038f14671ea6a66a332e6e6252a. The real Web runtime before/after identities were independently equal to the4992 five-source snapshots. Only the two Python source-admission/control files change from4992 to54b5; Rust/TCP/SDK inputs, actual schema bytes, locks, runner configuration and build profiles are unchanged. Those scoped runtime results are reused on that proven input boundary, not asserted to have the new global state.

Independent `git diff 7ad103ad2..4992de5fb -- . :(exclude)docs/plans` is empty. Precommit/frozen source records independently have equal raw source_sha256 for all five repositories. This establishes actual implementation/input equivalence across the final main rebase, rather than asserting equivalence from commit labels. Earlier dirty/precommit scoped checks are reused only where their tested helper/generator/oracle inputs stayed unchanged.

| Repo | Actual raw source SHA256 |
|---|---|
| Lang | e16ca2704fc88e06e7227d9e20be6ff8a926e836181117c3eee9b43604042a4c |
| Web | 88a18de2e4e18b3c61ddc31bef5ba34a1b3cf7ca0967b47a322076cca0a8edd3 |
| UI | 5d4a4d3b5b2aa0c05b9ef07be8e03d25276eecd41e6e84ddfb538b50a488808c |
| Man | e4f53a7ad5a30bc1eb1f2da785979eefa1f1e5705a6646322cbf88b2830eeabb |
| Down | a3678d75e2c7fcdd3068900b8a568e380ce68a1c8dbf035c5176dd4bce67b422 |

The current Lang raw digest now includes actual runtime Schema inputs and the new guard. The following older input records establish4992/rebase equivalence, not the new guard digest. Local input-record hashes: precommit-inputs.json84c9acc135cc2144a49fd2b654434860815db0c29de55e7eb2bd046eaf21fe1d; frozen-inputs.json05a805ec8032cfb95236450c4e349010b6d121f09c9953a1e438fff3884a5f10; rebase-range-diff.txt19a9253acd4dfefacfed58cc535965b7618ee78a1374f3e45ea2177155db0acf. All paths are under work/integration-896/repair-gate-1/.

##### Actual failed gate and bounded closure

The original gate is not waived: six language groups ran695.533s; integration3674PASS/8FAIL of3682, auto-api-gen2PASS/1FAIL of3; shared generation16/media54/a2r104/TestCase93 passed on their actual old inputs. External legs had not started. Original wrapper49/child436 comparisons did not turn these failed atoms or strict vctip124 into a PASS. Original result hashes under merge/final-896/four-owner/language/:0/result.json83a48036f46f20366582330d300a3a9601d762739f83e70de060dcbd5da6cac9;2/result.jsonb939c87343669e624acc122d3a004f182305e1898d4bf9e3c344418d01d0cc2b.

| Failure / invariant | Actual repair and review conclusion |
|---|---|
| ZIP source closure | Register existing zip_io.rs in Vm provider inputs. Directory-coverage assertion retained. No source exclusion added. |
|872 derived binding table | Original compiled generator updates only native_id_entries879→882 for the three already delivered ZIP natives. Original byte-equality comparison retained; independently re-executed with emit flag absent. |
|724/727 generated-project failures | Missing libsqlite3-sys/out/bindgen.rs with stale Cargo markers is the observed cache failure. Timestamps do not identify a remover. Work rebuilt only verified owned temporary sys-package caches. Original source/products/goldens/value assertions stay intact. |
| Compiler descendants | Both test helpers use existing auto_tool_execution::run_command_output with owned cwd/env/target and ProcessTree lifecycle, preserving original exit/stdout/stderr/error behavior and actually closing descendants. This does not disable strict outer workload accounting. |
|018 corpus/client request | Current corpus uses id; actual TS generator interpolates /books/${id}/chapters. Test now sends /books/1/chapters and retains200/entity assertions. No invalid :id fallback or product argument weakening. |
|031 native names JSON | Type-directed language-owned marshalling fixes the API boundary while preserving original Vec<String>/list/iterator/RC ABI. Original native image API still executes; result now explicitly parses Vec<String>. |
| Tauri fixture | Generator emits the existing fixture's compact handler registration, preserves all handlers/order, adds empty/multiple controls. Original fixed fixture and equality/dispatcher assertions remain unchanged. |
| Preparation receipt false rejection | Only explicit preparation may accept the original exact-pinned compiler auxiliary after real completed reap/remaining0. Consumer recomputes policy; workload errors remain fatal. |

The rejected broad typed-container patch is absent: no repair diff in convert.rs, ListData/RC definitions or native collection/iterator implementation. No unsupported filesystem feature was added to make a new probe pass. Initial fs.dir_names resolution failure, wrong diagnostic Type Display, RC-grace/canary test mistakes and mixed build/run vctip failure remain recorded; the corrected controls reach the existing Str.split/native-string-list path and original031 image path.

##### Type-directed boundary and high-risk paths

Inspected Core nv_to_typed_json in full. Only exact supported canonical []str/List<str> declarations (including existing ~ / canonical Future wrapping) enable legacy string-sentinel decoding. Unknown/integer declarations delegate to unchanged generic numeric JSON. Physical ListData<i32> uses checked arithmetic and bounded pool lookup; bad indices return None rather than falling back to numbers. Physical typed Value lists accept Str/CStr only: Value::Int(-1) is rejected even when pool[0] exists. Non-list nominal records retain ordinary object serialization.

The Core test proves actual native Vec<String> production, empty/duplicate/Unicode/escaped strings, Future spellings, typed-list compatibility, wrong typed integer rejection, unknown/[]int negative numbers, nominal record compatibility, old Vec<String> roundtrip, container RC ownership, unchanged pool length/counts after actual repeated marshalling and after failure/other branches, and quiescent teardown. It follows real dying-grace/reap rules instead of dereferencing freed canary objects. This helper reads VM values; it does not materialize new VM strings or alter collection/iterator ownership.

All four actual handler reply paths use it: Web json_value_reply (sync and final Future values), ServerHandle, LegacyStdnet, and UI backproxy. Web reads active immutable owner metadata; UI stores per-session f.ret.unique_name separately from diagnostic Type Display. The two middleware-short-circuit paths deliberately do not borrow the terminal handler's declaration. File/upload/stream/resource dispatch retains priority. Typed failure maps to500; UI releases task stack before early error, and both direct HTTP paths retain task cleanup and500 mapping. Unknown legacy metadata remains the old generic behavior, not a new promise to infer string types.

Current real Web runtime3 tests exercise both public configured CLI profiles, native Str.split string arrays, negative integer arrays, existing exact nonce/config/Host/port/shutdown/rebind and upload-limit snapshot. They are not mocked calls to the new serializer.

##### Actual scoped results and independent replay

Raw prefix: main `docs/plans/attachments/890/work/integration-896/repair-gate-1/`.

| Actual receipt | Result | SHA256 |
|---|---|---|
| frozen-typed/results.json |1PASS, exact4992 runtime state, clean meter | fb9bd2fea03b45f207259e91bd5d7fc4b677cd7808ea7f532dcc6fec5e228f23 |
| frozen-proxy31/results.json |1PASS, exact4992 runtime state, clean meter | 3c833f324cf9bf644476787c6ba8a825a19a55430e0012779f8c7c61da3fd197 |
| frozen-pins/results.json |12PASS, exact4992 runtime state, clean meter | c1754d6ef4bd70336649c02723c72ab088b2509c57ce4e8179bcaa16d74f0166 |
| frozen-schema-controls/results.json |13PASS, exact current54b5 state, clean meter |99ced027305796ee0822da11bb336ede719647f470e17ddd7f41155e54f98511 |
| frozen-web-runtime/results.json |3PASS, before/after4992 sources equal, workload meter clean | a5530f08d88185e29a0bf96cc0ac6410cebb28136f592f0e8a167670160bb04f |
| back-proxy-routes/results.json |1PASS | e4bf78f9a624bff7ab11ff71035d8d54b64fa9ca0473aba389b06ae00d5e06e5 |
| client724/results.json |3PASS, including both original e2e failures | e33a48cc66b0b135baeec75d9f93a43b618371bd8bcdf898c13cefb8f561c410 |
| transfer727/results.json |4PASS, original transfer oracles intact | 01ea76825a1b9954be9370d8620522f19da038b4a0ee2a43e90fb9dd0834da04 |
| binding-table/results.json |1PASS | 103e3e65dd080548942db5e714388e65c521a5805749b011cd7832a99e74762d |
| provider-inputs/results.json |1PASS | 63dc841fc8df345337cc3700d64f3dff205b525f61d385270f41c8cbd3d224d0 |
| tauri-registration/results.json |1PASS | 57848e40a8548a423ad9699991ba2e9281ccab5db5bfad193df22423c592ad39 |
| tauri-consumer/results.json |3PASS, original fixture/dispatcher assertions intact | 66e4c017af1248fa73fd356a5f45205c578e33fada31854a5f0c03384250bf31 |

All listed receipt bytes/hashes and actual selections/results were independently read. Non-frozen earlier checks retain their actual inputs; their relevant helper/generator/corpus/oracle sources are unchanged by the final narrow typed-Value/record refinements and equal-source rebase. Discovery_complete=false/filter receipts are targeted checks, never a complete integration/group PASS.

Reviewer independently ran `python scripts/test_four_repository_fixed.py`:12passed/0.204s. Also used the **actual stored successful MSVC preparation record**, not only synthetic policy data, in real command_proof:

- Exact path+current SHA, completed reap, remaining_active0: ACCEPT.
- Remove accepted_compiler_auxiliary_cleanup flag: still ACCEPT, proving the flag is not trusted.
- Same record as workload, remaining1, incomplete reap, original meter exit1, unknown image, timeout error, empty process list: all seven REJECT, even with accepted=true.

The actual frozen Web discovery preserves its original descendants-live error and pinned-vctip reap evidence; actual Web workload exit0/error=null is distinct. Revalidated the raw native stream through real native_terminals:3completed/error=[], no missing execution. Raw stdout SHA256729dd767f4c81f9f66780c71c84ac7620bee4d96bccf89e60578b542f6c752dc; stderr302490ba3165d07a36085608080e01548e906b68829f8d98909296e63f0d3aa3. These checks do not grant any general workload exception.

To distinguish real872 assertion closure from emit-mode rewrite, independently ran the current discovered binary `target/debug/deps/auto_lang-6f9e9cd4f26c95b1.exe` (SHA256704c553c959819c2da628d4038dc210b9d50c37dfc3d0bdd26ead107d3d88328), exact test `tests::plan872_builtin_binding_probes::plan872_binding_table_json_matches_regeneration --exact --nocapture`, with AUTO_LANG_872_EMIT_TABLE removed from the child environment: exit0,1passed/0.02s. No compiler or file rewrite was invoked.

##### Frozen Spec delta and source hashes

The three prepared additions describe the existing typed API marshalling boundary, not a new VM container ABI or transport capability. Language host-port specifies canonical string-list declarations, unknown/int numeric preservation and invalid-index refusal. Web HTTP contracts cover all declared handler paths, frozen owner metadata and resource priority; middleware remains independent. UI integration records per-session canonical types and unchanged collection/iterator/RC behavior. This is consistent with the inspected implementation and real wire proof, with no golden/budget/eligibility reduction, new goal, platform promise or dependency reversal.

| Affected input | Actual worktree-byte SHA256 |
|---|---|
| Core value_json.rs |394cf3306a0111365aeb2b9470ba04cf4b9d0a197f09469952c9f1684eb74f8a |
| Core api/targets/tauri.rs |3f64ec32dd03f149ea9584bbe790e9a90f2770b3633b763304a60797aabea857 |
| Core stdlib_assembly/manifest.rs |09cc0d479519243bf22cf0be377b0115073575d528ecb7d6e0299cbbc42fee81 |
| four_repository.py |92a26fefed2888766cf8e0020edc0ab29a3f955fd9ede8f85fe57bbea1e4d2fa |
| test_four_repository_fixed.py |fc44d929ab3a838cb73a784e19cf53708c617d6afcaf7f9584828d37d83fb456 |
| builtin-binding-table.json |a943d7604cbf24ba64c6973c2f7ed97061a0d7d4c178b70d4fe6c84999a39657 |
| Lang docs/specs/language-sdk/http-host-port.md |b79e7f252170c2d8ac8bbf9ef51944ed8dfe473da0b1ef4a1dc84ff3f4189f40 |
| Web docs/specs/http-contracts.md |3e1af8b5e1a4a9f8d2cd9c64327c8143eb5dfccc97573e48e1afdc5148bd69ee |
| UI docs/web-integration.md |be959b54ac69a0ba4b2f8e71a332320c76fcdcf3fcdfde912f3ba0e61984374e |

This table plus the explicit before/after rules freezes the affected SD in this Markdown receipt; no canonical file was written by the reviewer. Existing component paths/empty touched_goals remain justified; no new semantic Plan revision is required for these corrections. Scoped diff whitespace checks passed in language/Web/UI. Compiler output/first failure limits remain recorded; this receipt does not claim the repository has no inherited warnings.

##### Acceptance, Spec decision and route

| IDs | Current result / evidence boundary |
|---|---|
| AC01/02/05/09/10/11 | pass, retained within t17's source/ownership/provenance/mechanism scope, with current identities and the explicitly affected consumer changes above. Old SDK/UI/binary cohorts are not repinned to this source. |
| AC03/06 | **pass, updated**: all four handler reply paths, canonical owner/session types, real native strings/negative numbers/config/Host/shutdown, resource priority and typed failure cleanup. |
| AC04 | **pass, updated**: Tauri generator now matches unchanged original fixture and dispatcher; full handler/order controls retained. HTTP/nonHTTP/shared-generation ownership unchanged. |
| AC07 | **pass, updated**: VM/list/iterator ABI unchanged, actual post-marshalling RC/pool checks and cleanup; owned compiler children close without waiving workload accounting. |
| AC08 | **pass, updated**: ZIP/provider/derived source closure, preparation-policy revalidation and runtime-Schema source/clean guard all close their actual input invariants. Original ID/floor/child proofs remain intact. |
| AC12 | **pass, updated**: nine failed atoms and the final Schema guard have bounded closure; no mandatory work-owned gap remains. Actual aggregate is still pending. |
| AC13 | **pass**: final affected acceptance binds54b5 and the unchanged runtime-input evidence boundary. |
| AC14 | partial, merge-owned unified execution/landing/deposition/archive/guards/cleanup pending. |
| V01/02/07/12..16 | retain applicable t17 accepted mechanisms and cohort limits; current runtime/source/provenance changes are explicit above. |
| V03/04/06/08/09/10 | **pass, updated**: direct/default/UI reply paths, type/resource/RC rules and actual wire/cleanup evidence; no widened platform/legacy claim. |
| V05 | **pass, updated**: original fixed Tauri bytes, dispatcher and new full-registration controls. |
| V11/17 | **pass, updated**: original failure oracles, genuine emit-disabled comparison, source coverage and strict workload/preparation distinction. |
| V18 | pass review portion; merge portion partial/pending. |
| SD01/02/04/06/07/08 | pass proposal, retained t17 ownership/source20/fixed-group/current-CLI and eligibility rules; original gate failure remains explicit, not delivered. |
| SD03/05/09 | **pass, updated**: typed language boundary, Web frozen handler paths and UI canonical session metadata match code/controls without VM ABI expansion. |

No work-owned finding remains. Parent may record reviewed11/12 for this exact freeze and route to T12. The old failed aggregate, temporary test-input errors and abandoned broader ABI change remain preserved. This PASS never covers future product edits.

Still **NOT_PASSED, merge-owned**: rerun the complete current **six language fixed groups plus all eight external owner legs**, current20-package SDK/source/third-party export and actual SDK tests/distribution/transaction/legacy old-new Core recovery, due main-only periodic batch, every repository landing and source recovery-point readback, canonical/compatibility-ledger/index/module-plans deposition/readback, archive/count verification and each mandatory worktree guard/cleanup/user-policy evidence accounting. The earlier four passing language groups and scoped3/12/1 results do not substitute for the new aggregate. Do not waive a subsequent real failure or run a retired tm campaign in its place. No remote publication is authorized by this review.

##### Final runtime-Schema admission closure

Before the final verdict, Work identified that docs/schema/aura.at is an explicit SDK runtime root_input and must not inherit the general documentation exception. No PASS on4992 was published; the interim draft was held pending this repair. Final54b5 adds a shared source_input predicate used by snapshot and initial/final/public-admission clean checks: runtime Schema .at/JSON inputs are included, while ordinary Markdown/Plan/report bookkeeping keeps its prior exception. Current code/Schema were clean; this is a source-admission gap closure, not a claim that an actual mixed-source build had occurred.

Independently read4992→54b5: exactly four_repository.py and test_four_repository_fixed.py changed; no Rust, SDK runtime source, actual Schema, lock, test configuration, sample, golden, build profile or transport byte changed. Latest actual cargo t web-delivery-controls frozen-schema-controls receipt has13PASS, clean meter, SHA99ced027305796ee0822da11bb336ede719647f470e17ddd7f41155e54f98511 and source_state9154eb92b9ba992a2d7e3eeabaea8bb4a209c25d9124511c3874617cbf34941e, independently equal to current.

Reviewer independently ran the final13 controls:13passed/1.602s. Added an independent real temporary-Git holdout: commit original docs/schema/aura.at and docs/plans/progress.json; stage changed Schema content and verify **both source and resource hashes change** and clean admission rejects; restore/stage original Schema, change Plan JSON and verify clean admission accepts and the **entire snapshot remains equal**; add untracked docs/schema/runtime.json and verify clean admission rejects. All three outcomes matched. No compiler or authoritative-tree mutation occurred.

This closes the last source-guard gap and updates SD02/07/08's existing full-runtime-input invariant; it creates no new product scope. T17 and the interim T18 draft remain historical boundaries. The final PASS above binds54b5 only, with earlier Rust/TCP results explicitly reused on unchanged input bytes. The complete new aggregate and all T12 operations remain NOT_PASSED.


</details>

<a id="e890-d0511e724d4bee1e"></a>

<details>
<summary>review\t19-second-final-gate-repair-r7.md · SHA256 8c695e50599d6df83e228e98461162fb2c23d54da1268e98eff1cd852a9c054f</summary>

#### PLAN-890 r7 — T19 independent affected review

**Outcome: PASS — final affected acceptance of this exact frozen repair.** Remaining actual merge gates are not passed.

Reviewer context: `/root/plan890_final_review`; final affected review in the existing authoritative sibling trees, independent of Work. Implementation remained paused. This receipt updates T18 after the actual second unified-gate failure; it does not overwrite either historical receipt or certify a future gate.

##### Frozen scope

| Repository | Inspected HEAD | Branch |
|---|---|---|
| Lang | 54b5d7c92f0244faecde929353b7e292ff6b3579 | plan-890-dev |
| Web | e144ec0d9febe74f8cf734744d91ade47fe57bcf | codex/plan-890-dev |
| UI | 4da2201fa46bd8a88bc5c2d661b21433b7ac84b5 | codex/plan-890-dev |
| Man | d2b2664fea33a93def31ef4cb04be7110c97e807 | codex/plan-890-dev |
| Down | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | detached |

Paths: `D:/autostack/.wt/lang-890/<repository>`. HEAD/branch/status and registered worktrees were independently read; all five implementation trees were clean. Main Plan was read from `D:/autostack/auto-lang/docs/plans/890-auto-web-independent-repository-delivery.md`: revision7, execution_done10/12, round45. The worktree Plan was not used as the status authority. The review skill and verification-contract were consulted. No implementation, Plan, canonical source, ledger or index was written by the reviewer; no compilation or full campaign was run.

Man comparison base:43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6. Exactly four Rust files and docs/web-integration.md changed; no manifest, dependency, lock, configuration, golden, floor or original test ID changed. `git diff --check <base> HEAD` passed. Work handoff `work/repair-second-final-gate-r7.md` SHA256:7bf820d7dcf985a17301704f6752ba54b4d0a50a75dd92248ef2683daf18bd61.

##### Mechanism and actual evidence

The original media test now observes the actual Web generator instead of the Lang forwarding facade. Every original assertion remains, and the facade delegation assertion is additional. The original three media IDs and actual consumer/runtime comparisons remain intact. This corrects source ownership observation without changing a golden or accepting different product output.

The private build helper is used by the actual CargoBuilder build and NinjaBuilder build paths. Source inspection confirms original argv, Cargo cwd, Ninja inherited cwd plus its original -C argument, inherited environment and all three console streams. Memory-only early-return paths remain before process launch. Windows selects the existing ProcessTree facility, collects the root's original ExitStatus, explicitly terminates the owned Job, and propagates spawn/wait/cleanup failure. Root nonzero remains nonzero and Cargo's original status diagnostic remains. Unix retains DirectChildOnly and the foreground process group; this is source-level review, not a cross-platform execution claim. Build/run/clean APIs and unrelated run/clean paths were not changed. No image-name kill, workload whitelist, new deadline, captured-output substitution or test-body process wrapper was introduced.

Spawn preserves the original io source/kind; wait and terminate errors propagate with ?. Ninja identifies the command but combines execution-stage wording. More specific cleanup-stage wording would improve diagnosis; the current implementation does not swallow or convert a cleanup error into success, so this is not a blocking acceptance defect.

The warm original v05 reproduction has its assertions passing but strict descendants-live rejection, wall8954.462ms; retained vctip PID20880 is from the Community MSVC14.43 toolchain. This is workload accounting, not a preparation exception. The real replacement receipt selects the original media3, original v05 real Cargo/Ninja1 and new helper2. Raw protocol contains six active successful atoms plus four original ignored synthetic events; ignored events are not credited as executed tests. Both preparation and execution meters have error=null. Preparation14419.543ms; execution21408.495ms, root exit0. The nonzero23 and missing-program NotFound controls exercise failure preservation. Reviewer independently replayed the actual stored discovery/terminal stream through production fixed.enumerate_rust (selected Man root) and native_terminals:6 selected,6 successful terminals,errors=[]. An initial readback mistakenly used the default Lang root and correctly failed the checkout-boundary guard; correcting the root gave the result above. No reviewer rerun of these Rust bodies was necessary.

All following raw paths are relative to main `docs/plans/attachments/890/work/integration-896/repair-gate-2/`:

| Raw input/result | SHA256 |
|---|---|
| diagnosis-v05-warm/run.command.json | 74e1b60cafa4d9668ab6f4727ee331ebca60e0de4013e2f9b6cc0ed9c6ab3e2e |
| targeted-owned/run.command.json | c67e11c860e179cb01a88f4de1e1774b2eb9087f269de824766248680c3536ef |
| targeted-owned/run.stdout | e44152dd6be45e33f757529d3dabdc4b3989c39a77e170317ba2292b05c2c91e |
| targeted-owned/discovery.command.json | a3e76e8e00b760f5bce3b753051b01bf699089ecdfbab6a34de68feaadd9d25b |

##### Recovery composition review

The local recovery script is evidence orchestration, not a public runner replacement. It was read completely and was not executed. Initial inspected SHA256:96497ad263591a51564a020e20773bf690aea6d84dfcbce19321867bf5670c9b.

The original failed receipt at `merge/final-896-2/four-owner/receipt.json` independently matches SHA2569c2e7ecbab4b3789b498dd251b94ad89147c97c12bb5fd589e4ff3db70a934bd and error `man-ti: execution failed`. Its actual language3954, Web221 and UI-full3121 success records are retained at their original paths and producer identities. Current complete Lang/Web/UI/Down snapshots independently equal their original producer snapshots. Man alone differs; its diff is exactly the five approved files. Actual locked/offline/no-deps metadata of Lang/Web/UI succeeded and showed no path dependency into Man. Source searches additionally checked changed-file references; the Lang Tauri fixture dependency points to a different unchanged Man file.

The script preserves the failed aggregate and copies only language/Web/UI-full successful evidence. All four Man legs must execute again, including the previously passing legs, because their production builder changed; UI-tu/ti were never executed and must run. It uses production fixed_proof, source discovery, owner_selection and final validate for complete fresh denominators, original active/ignored conditions, atomic terminal protocols, child comparisons, clean source readback and the full eight-leg union. Any exception after receipt creation records FAIL and returns nonzero; the new output directory refuses reuse. No old Man-ti partial success is reused or relabelled. The prior hash is checked again before success. No fresh run is claimed by this review.

The initial recovery draft did not separately establish external environment/tool applicability. This was resolved before verdict by an additional local admission, without product code changes or replaying unchanged test bodies. Final resume_four_owner.py SHA256:2a2f4c8825851a1a1205220c1013a9c5b126b7694ffaafad81d48a176b8ca574. Its immutable recovery-provenance.json SHA256:302f00202b413fc1837b89df61005fe773d89d2a6d48964051bd6d729302b965.

Reviewer independently located the actual CommandExecution item exec-9395695e-51c6-4989-b0e5-c9a46f9c8d27 for the original final-896-2 command in the recorded main session JSONL. In Work's actual primary execution environment, normalizing only CODEX_TOOL_CALL_ID in a comparison copy reproduces the original complete external environment hash8fe46af2c354f86270d4e81e9fd2868f973e6054297b296d19d43327d0ee532c; flags, wrappers, PATH, roots and applicability conditions are therefore covered. The new real execution keeps its own dispatch ID and records its actual new environment hash. The six recorded primary controls (valid input, four environment mutations, scheduler-nonce-only acceptance) were read back; raw SHA256011bb9f6e6426db90299483403c46ff494964af5e0ac22f0a3635d01715d0cc1.

Reviewer independently verified all six global tool and two bootstrap binary current file hashes/mtimes against the pinned provenance, then exercised the admission function alone with an isolated reviewer-environment valid control: accepted with actual environment unchanged; RUSTFLAGS, T3_MILESTONE and RUSTC_WRAPPER mutations rejected; deliberately wrong Cargo SHA rejected. This isolated control tests the shared guard, not a claim that the reviewer's different process environment reproduces the executor's historical hash. No orchestration or compiler was launched. The actual-script admission runs before gate work, after bootstrap and after final public validation. Any change rejects rather than silently blessing reuse.

Historical global-tool file hashes were not independently captured during the old run. Their pre-dispatch mtimes, matching cached rustc1.98.0 detailed fingerprints/version, exact source/config/environment and present pinned files support the unchanged-toolchain inference; this is not a fabricated historical hash. Bootstrap pins are the retained original bootstrap outputs, not proof of an earlier language-phase binary hash. The bounded reuse is acceptable with these explicit limits, while actual new tool/source/condition readbacks and the public final consumer remain mandatory.

##### Acceptance and specification scope

T18's complete AC/V/SD evidence map is retained only for its explicitly unchanged inputs. The new Man delta does not change Core/Web/UI Rust, SDK source20, locks or runtime profiles. Original R1/R2 and later typed marshalling/Schema controls remain within their prior byte and mechanism boundaries.

| IDs | Affected result and boundary |
|---|---|
| AC01..04/06/09..11 | retain applicable T18 pass; no new implementation claim or SDK aggregate certification |
| AC05/07 | pass affected Man build lifecycle: inherited command semantics, original status, owned Windows cleanup; non-Windows code inspection only |
| AC08 | pass affected original media ID/assertion preservation and recovery selection structure; composed actual gate remains pending |
| AC12 | pass bounded source/health/omission scan: no dropped assertions, golden/floor weakening, workload exemption or unrelated rewrite |
| AC13 | pass: affected independent review recorded on exact five-repository freeze; final route below |
| AC14 | partial: all remaining merge-owned gates and landing operations are pending |
| V01..05/08/09/12..16 | retain T18's applicable source/input-bound evidence, not a claim of new full execution |
| V06/07/10 | updated Man status/stdio/cwd/build-owner/lifetime review and actual6-atom strict evidence |
| V11/17 | updated source-owner oracle and recovery denominator/failure/old-FAIL preservation review |
| V18 | this review portion only; actual merge/deposition/archive portion pending |
| SD01..05/07/08 | retain unchanged accepted proposal boundaries |
| SD06/09 | Man docs/web-integration.md correctly records Windows owned build cleanup, inherited CLI semantics and non-Windows terminal boundary; no runtime support expansion |

Frozen affected canonical proposal is Man docs/web-integration.md at d2b, SHA256e7d25da8a72860388a37fda843f7c0d54f6bf59543dee78c54ef1cc85ae3c805. No new component or goal is introduced. Main spec-impact metadata remains applicable. Reviewer did not publish canonical/ledger/index changes. Evidence retention is not performed during review.

No work-owned finding remains in this affected repair. Parent may restore T11 reviewed11/12 on the exact frozen cohort and route to T12. The reviewed local recovery script may compose unchanged successful execution evidence only under its explicit source/environment/tool admission, rerun the four Man legs and missing UI-tu/ti, and require the final public consumer. This review never labels that future composed receipt PASS in advance. Current SDK20/distribution45/legacy14, composed four-owner receipt, any due main-only batch, actual repository landing/source recovery, canonical/compatibility-ledger/index readback, archive/guard/cleanup remain NOT_PASSED. Historical FAILs remain immutable. No remote publication is authorized.

Final readback: five HEADs/cleanliness remained as listed. Man four-Rust/canonical-doc delta and local evidence-script SHA were frozen at verdict. No implementation edits, full gate, retention operation, ledger write, staging or commit occurred during this review. PASS does not cover later implementation changes.


</details>

<a id="e890-2adbc3f91dba4d5f"></a>

<details>
<summary>review\t19-t12-recovery-addendum-r7.md · SHA256 e9dccc506f1430e8641b8b3e57e245451f6fc048d7328890e8573ce646ffdc47</summary>

#### PLAN-890 r7 — T19/T12 interrupted-gate evidence addendum

Context: `/root/plan890_final_review`, independent read-only recovery review. This supplements the frozen T19 product acceptance; it neither changes its source SHA nor relabels an old runner as the new recovery script. No Rust, specification or public runner change was requested or made during this review. No compilation or test body was run by the reviewer.

##### Actual execution and recovery boundary

Frozen sources remain Lang54b5d7c92f0244faecde929353b7e292ff6b3579, Webe144ec0d9febe74f8cf734744d91ade47fe57bcf, UI4da2201fa46bd8a88bc5c2d661b21433b7ac84b5, Mand2b2664fea33a93def31ef4cb04be7110c97e807, Downc3464d0a0d36e4f8a7988b0c0c26448fc52361b4. The final receipt before/after snapshots independently match all five current sources; clean admission remained unchanged.

The final-896-3 process interruption has unknown cause. Its ui-ti directory is empty and no top-level receipt exists. It provides no UI-ti execution result or aggregate PASS. Its completed Man283/24/254/43 and UI-tu866 rows were independently replayed through the production command/selection/terminal consumers: exact required commands, successful atoms and strict meter error=null. Source/configuration reuse remains within T19's declared Man-only repair boundary; original successful Lang3954/Web221/UI-full3121 execution records remain at their original paths and identities. No failed atom or ignored condition is converted into a successful execution.

UI-ti was actually replayed as a new producer:28 active PASS,15 original condition-excluded, strict meter error=null. The reviewer used the real stored listing and run output through native_selection/native_terminals and command_proof, obtaining28 selected, no terminal errors. Started/ignored protocol markers are not credited as active execution. phase.json is completed/exit0, five-source before==after; actual-command.ps1 and execution-source.json identify the new real CommandExecution exec-4c0aa40d-b46c-4b1e-968c-b15d5a8af764, its start/end and exit0.

The derived UI binary legitimately has a separate execution boundary: earlier rows retain their original9a6 bootstrap evidence; the new UI-ti phase records actual5c5d94249d48ed7001f044ed13c2fee507218cba958a0a2fca02b5021ef8e895 before/after preparation and workload, with source/flags and real execution proof. The binary was independently read at14:07:11 mtime. Preserved exact te-child/UI-tool bytes independently match the final-execution-tools manifest and phase pins. Old results do not claim to execute the new binary. External static-tool and whole-environment admission remain strict; this distinction does not waive workload descendant accounting.

##### Completed public admission

The separately reviewed local complete_interrupted_four_owner.py at SHA256a08c8a221b23d9b0841827ead854d2c206cd60fb1a467f14e0ef604e20c41714 reconstructs completed rows from original raw files and records original producer/environment identities. It initially persists pending_admission. The independent reviewer located its actual CommandExecution exec-8ebec7ba-f381-43e4-8c63-2d0cbabd09fd: completed/exit0, output first says checkpointed/public admission starts, then Final four-owner admission pass. Thus the current success is supported by normal actual completion, not merely by a JSON outcome string.

The resulting receipt is `docs/plans/attachments/890/merge/final-896-4/four-owner/receipt.json`, SHA256734410720b8026390401da1fe21826654c421ca8a212d77b21e541367084df40. All six language and eight external owner fresh denominator directories exist. Each external source-list command is successful with meter.error=null; selected sizes remain221/3121/283/24/254/43/866/28. Public full validation also checks original atomic logs, fixed source bindings, actual child comparison traces, migrated obligations, original ignored conditions and source/layout identity. The reviewer did not rerun this discovery campaign: its actual normal invocation and raw readback are reused.

The actual composed four-owner gate evidence is accepted for this source cohort. This is not overall PLAN-890 delivery or archive acceptance. SDK/distribution/recovery/landing and other T12 obligations retain their own status until actually verified.

##### Local finalizer finding F19-R1

A distinct local-script interruption hazard was found, without evidence that it occurred in this successful invocation. The script sets its in-memory outcome=pass before calling f.validate, then catches Exception only. KeyboardInterrupt/SystemExit are BaseException. Its finally writes the current receipt even if admission did not complete.

Independent minimal control used the exact actual AST's last try/finally block, an in-memory receipt initially pending_admission, a validate substitute raising KeyboardInterrupt, a null machine gate and a write substitute recording outcomes. Actual recorded write was ['pass']; expected is no persisted PASS before all admission/source/static/original-hash checks complete. No filesystem receipt, original evidence, compiler or real gate was touched by this control.

Scope: recovery orchestration only, not a new Man/Rust product finding. Fix before retaining/reusing it as interruption-safe: a completion flag set only after all checks, with finally refusing PASS unless complete, or equivalent BaseException handling. Minimal closing oracle: normal completion accepts; ordinary exception, KeyboardInterrupt and SystemExit during validation/final checks persist non-PASS. No test-body replay is needed. This defect does not negate the independently confirmed normal completed/exit0 public validation above.

##### Raw identity map

Paths relative to main `docs/plans/attachments/890/merge/final-896-4/four-owner/`:

| Artifact | SHA256 |
|---|---|
| ui-ti-replay/phase.json |92b7eba711b82b4a5eadd5b9d7f4b0b531e82b395100592d1e08c3bf3a77f350 |
| ui-ti-replay/row.json |f67314d44bd454d36d1c5230b96e7af072135d817fde2d47334c0fe49906183d |
| ui-ti-replay/execution-source.json |6f93e3be92d7eb0c2190316ed9a6774e4ec204d8096be4a3d4e8b3dec7619bfe |
| ui-ti-replay/actual-command.ps1 |672c97917bddaf6c9aa99514c12471c813efebb190efa381821258a72508a61a |

Preserved binary manifest: `D:/autostack/plan-artifacts/plan-890/final-896/final-execution-tools/manifest.json`, SHA25656ed5e0eadedb410ab67db5e319913ae32c0e0bea75909f4374b48487fee90d4.

AC08/V11 updates: composed actual execution/public admission is evidenced, exclusions and failed/interrupted history preserved. AC12/13 retains T19 implementation/SD acceptance; local finalizer finding above is separately scoped. AC14/V18 remains partial until all other actual T12 delivery/landing/knowledge/archive/guard operations close. No product repair or unrelated full-suite repeat is justified by the unattributed interruption alone. This addendum does not authorize future changed scripts or code automatically.

##### F19-R1 closure

Closed before this addendum's final routing. The exact normally executed v1 script was preserved as complete_interrupted_four_owner.executed-v1.py, SHA256a08c8a221b23d9b0841827ead854d2c206cd60fb1a467f14e0ef604e20c41714; its actual completed execution and original receipt producer remain unchanged. The corrected local script SHA25612b268077ed69ea0292d030ce9b2ae97a79bbf5aecdf82285f6d4172f953ab66 adds admission_completed=False, sets it True only after public validation and all final source/static/original-hash checks, and forces a non-PASS write in finally if incomplete. It preserves KeyboardInterrupt/SystemExit propagation.

Reviewer independently replayed the corrected script's actual AST try/finally with isolated substitutes: normal completion persisted pass; KeyboardInterrupt and SystemExit persisted fail and propagated; ordinary ValueError persisted fail. All four matched the closing oracle. Work's corresponding finalizer-controls.json independently matches SHA2567becb7b1928e5201cf327c4246c125ba0127a7f7586365b89442d75f695c03fe. These are control executions of the finalizer, not fake native gate results. No body or complete admission was repeated for this pure finalizer correction.

Final bounded conclusion: the actual completed composed four-owner gate evidence is accepted, and F19-R1 is closed. T19 product acceptance stands at its exact frozen cohort. The new local-script hash is certified only for the inspected recovery/finalization mechanism; it is not relabelled as the producer of the already completed v1 gate. Remaining SDK/legacy/recovery/landing/deposition/archive/guard operations still require actual T12 evidence. No overall Plan completion or future implementation PASS is issued here.


</details>

<a id="e890-c98ba41c7d4b5e99"></a>

<details>
<summary>work\blocking-listener-port-r2.md · SHA256 548e8e224ed595fc79aafc2e4166eee39569ff6df8108dc179978dfddb7c737f</summary>

#### PLAN-890 r2 — explicit borrowed blocking listener seam

Status: bounded T-02 progress; executing 1/12. Not production Web delivery.
Parent code revision: `38b6a8178c517613fd531694930f9a3c4879c1c2`.

##### Change and ownership

`HttpServiceHost::listen_blocking` receives the existing borrowed `&AutoVM`,
address and owned immutable compiler-metadata snapshot. Selection is shared
with the async entry, before binding a socket. Missing providers give the
agreed explicit-install diagnostic. Async-only custom hosts get an explicit
unsupported-blocking error rather than an implicit Core fallback. The language
native `shim_http_server_listen` and legacy SDK `run_http_server_blocking` both
select that host. The latter preserves its unit-return/log-on-error signature.
No VM field or private module visibility was widened.

The two equivalent server-handle loops are now one source,
`http_server/server_blocking.rs`. The external boundary fixture compiles that
actual source and runs it through the borrowed port; Core still compiles it
until T-05. Media wire serialization moves to that source; renderer-neutral
media lookup stays in the language primitive. Its original test accesses the
same writer through a test-only alias, without modifying test registration.
Fresh handler tasks clone out of DashMap before locking, so error cleanup
cannot hold a table read guard while removing its task. `try_lock` handles the
owner-thread call within a current-thread Tokio runtime without blocking_lock's
runtime panic. Both busy-error and bind-error paths remove the fresh task.
The metadata context helper restores nested/Pending/unwinding owners and is
shared by synchronous and async selection.

##### Actual validation

- `cargo check -p auto-lang -j 4`: exit 0; default dependency surface including
  existing UI consumers checks. Existing warnings remain (Core 141; root 3;
  dependency warnings); this is not a final warning-clean assertion.
- SDK fixture `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin blocking --locked -j 4`:
  exit 0; external boundary 17 existing source/dead warnings.
- `blocking_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/blocking.exe --mode installed`:
  exit 0, real annotated language handler via borrowed external runtime;
  exact per-run JSON, missing parameter 400, invalid integer 400, unknown route
  404. Controlled accept wake returns owner; handler-task count unchanged and
  owner thread preserved.
- Modes `direct-installed`, `direct-missing`, `unsupported`, `missing`: wrapper
  exit 0 each. Direct-installed invokes the actual language native from an SDK
  consumer inside Tokio, returns exact 404 wire and stops/returns with unchanged
  task count. Direct-missing invokes that same native with no provider and gets
  child exit 1/explicit-install diagnostic; unsupported uses an async-only
  provider and gets child exit 1/unsupported-blocking diagnostic; missing tests
  the annotated post-main activation without a provider. All three negatives
  verify the requested port is still bindable. No fake server/forced-exit green.
- Scoped Core tests (three distinct original IDs, each 1 passed, 0 ignored):
  `explicit_http_host_selection_is_stable_for_active_local_owners`,
  `owner_metadata_restores_after_pending_nested_poll_and_unwind`,
  `image_media_route_has_priority_without_changing_user_routes`.
  Selection test now also asserts missing/unsupported borrowed calls.
- `git diff --check`: exit 0. Changed/new isolated Rust sources formatted;
  no whole-tier tests or docs_gen.

Raw local-only logs are in this directory; no non-md evidence staged.

| Local log | SHA256 |
|---|---|
| `blocking-port-build.log` | `38f471f5ccb4a6f3b2205ad7c056728b3e0e1c09e30d5aea7165465db98b64fd` |
| `blocking-root-check.log` | `de97e059842ac24611b7447ae8a3ad6f107a9154a074e4d1d5849841604d4850` |
| `blocking-host-selection-test.log` | `10db92443e886b9ddfe9d7e795edc831485bc7f0c9e815be981b697aed586b3a` |
| `blocking-metadata-context-test.log` | `e489f80da82f5cbf3e74d1e29ea0deaaa5fd48bf0d2a7cc3ace4d2e6452cff60` |
| `blocking-media-test.log` | `8eeb98fe96a23ca6bf83145438340530b4e8a40d32b339831871fbcb72875249` |
| `blocking-installed.log` | `e8ac2ff1185cef4f83978aa16481f42db3cc62d1ee7501717fc1ad402a860b9d` |
| `blocking-direct-installed.log` | `95c713b8440d969efd1ede9e80f673a2df7b1ced285230311583ebab71112061` |
| `blocking-direct-missing.log` | `8d3a6b40eee338871957007a13ca9f273ddf03843ca526f402428cd295f54eda` |
| `blocking-unsupported.log` | `3cbca589e032c95a122625a62206bbb80450e0107c448273bfc7c35a6a54b8a5` |
| `blocking-missing.log` | `532635beb782e3d5dc3fdf5bff8f46f14e80fbb324c8b140d57e13200e7c8fac` |

##### Limits and remaining T-02 work

An initial source spelling `use auto.http; server.listen(...)` failed with
`Undefined symbol: Server.listen` before reaching the native. It is not counted
as passing language-level direct-listen evidence. The direct-native control
uses the public SDK `VMConvertible` alias and actual shim, without publishing
private modules. Production language spelling/native binding remains an
explicit T-02 obligation, not an accepted deferral.

The stop flag is a bounded embedding control checked after accept, woken by
one owned controller socket. It makes no slow-body cancellation or drain
promise. The source is still serial stdnet behavior; legacy
`serve_blocking_stdnet` has different SSE/CORS behavior and is not silently
replaced. That entry, possible post-main reactivation after a direct listener
returns, actual back-proxy consumers, unconfigured mutable service state,
producer scratch, exact deadline terminal behavior, complete shared-generation
helpers/DAG, physical package/API/SD/command/four-repository schema freeze all
remain T-02. T-03 Web repository has not been created. T-05 must remove Core's
framework implementation and temporary adapters; this source fixture alone is
not that migration. T-08 must reconcile all original IDs/resources against the
immutable T-01 inventory, including the moved writer assertion.

Shared registration/runner/locks and final merge window remain untouched;
888's `spawn_outcome_reporter` helper is not changed. UI/Man structural
successors still wait for full U890. Merge-owned final gates pending.


</details>

<a id="e890-8e23a6b7df3766e4"></a>

<details>
<summary>work\compiler-metadata-owner-r2.md · SHA256 bb4e2181617e71a16842552d1e3e347bf3792a248cc454bf52cecb52a98f0e80</summary>

#### PLAN-890 r2 — compiler-to-VM metadata ownership

Round24 / work / T-02; parent `2dbde2ab53faa4d86d5713f03a91fedac1535d69`.
The preceding round made progress. This closes the observed compilation/main/
activation ownership gap; it is not full T-02 or final delivery.

##### Cause and ownership

Previously the host captured process metadata and routes only after main.
Actual red: compile A, genuinely park A's main on an existing delay future,
compile/activate B with conflicting handler names/types/routes on the same local
thread, then resume/activate A. A's own route returned 404. Initial fixture
prefix-await syntax was invalid and was corrected to the language's `.await`;
non-JSON error parsing was also corrected before recording the accepted 404 red.

A language-owned, thread-local `CompilationMetadataGuard` now collects only
owned declaration data. Existing compiler publishers still maintain their
legacy caches, and additionally publish into the current collector through
named language bridge operations. The four normal construction paths in
lib execute/test, entry source construction, and script debug freeze and bind
the result before module initialization/main can await. Nested compilation,
error and panic restore the prior collector; the guard is !Send.

The VM stores a private immutable Arc, with named bridge getter/binder. Binding
requires exclusive `&mut AutoVM`; no private field/AST/Store is exposed, copied,
or sent across a network thread. Automatic activation and borrowed listeners
use that VM's declarations/routes; ordinary programs with an empty frozen route
set cannot inherit another program's routes. Async-return declarations are now
included in the neutral DTO and selected by the actual external reader, instead
of reading another source crate's private async table. No opcode/bytecode
generation or native registration changed.

Custom SDK compilers can begin the same guard, compile their modules, then call
`VmHttpBridge::bind_program_metadata(&mut vm, guard.finish(&routes))` before
sharing/running the VM. Existing unbound raw VM/record/read APIs keep their
legacy fallback; that process cache is not a new multi-program concurrency
guarantee. T-05 must resolve its final owner/facade and remove runtime scaffolds.

##### Scoped evidence

All source/build/scratch work stays in `D:/autostack/.wt/lang-890/auto-lang`.
Raw logs below are local-only in
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Result | Log SHA256 |
|---|---|---|
| parent `metadata_gap_probe.py --exe .../isolation.exe` | expected1: A's own route404 after B compile/activation | `metadata-gap-before.log` `d0c6f55d0db345606d9ee1aa40be17673f1a90d94c8f238da3e7a9e26d0eaa03` |
| final same probe, invocation-specific values | 0; real A main parked; A/B own routes and int/str params, actual external async reader true for A/false for B, future result25 vs ordinary int240, invalid int400; both drain/registries0 | `metadata-gap-after.log` `9688739ca375d79606ff2fe32b8e04efae7ffc9331148838730ca19b14cb3f53` |
| same `--ordinary-first` | 0; A resumes with default host cleared, returns42 and no listener/provider call; B stays independently usable | `metadata-ordinary-gap.log` `d6e98ffb7baeeaf3bb31da52ad206cdac5e2e27352d1918ec5a0b0f7628c9263` |
| `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin isolation --bin blocking --bin web-port-probe --locked -j 4` | 0 | `metadata-owner-build.log` `d1243dadf9d17cc1745aca86a7dca4e007b7ccb61d5a65051c74d8c073739189` |
| `cargo test -p auto-lang-core program_metadata::tests:: --lib --locked -j 4 -- --test-threads=1` | 0, 2pass/0ignored; nested real compile plus error/unwind preserve outer params/return/shadow/async data; two real concurrent compilers, four rounds/eight VMs, both finish before snapshots checked, bounded cross-thread waits, only metadata Arc crosses threads | `metadata-owner-tests.log` `31898e29cbad19018de64728680f26f0b59467ec06a5c5d8e271242ee122ed7c` |
| `cargo check -p auto-lang --locked -j 4` | 0 | `metadata-owner-root-check.log` `45e25720960ec8ba6921f67b90c2f70a3bafdc4c1aa9d14de68715b50bf99c9b` |
| probe `cargo check ... --features compile-fail-owner-send --bin web-port-probe --locked -j 4` | expected101; compiler rejects CompilationMetadataGuard because Rc<()> is !Send, alongside existing VM/context negatives | `metadata-send-negative.log` `e305dd53c2cbaa13189826ed9a41ed03eda77e23a549d64307595eeb8d723826` |
| original exact-name `plan669_api_param_sigs_roundtrip` | 0, 1pass/0ignored; legacy cache facade preserved | `metadata-legacy-roundtrip-test.log` `b777ca0e855c7348fdc43a9b37894d51fdc800c6dc8106b41780045b898291ca` |
| original `owner_metadata_restores_after_pending_nested_poll_and_unwind` | 0, 1pass/0ignored | `metadata-poll-restore-test.log` `3f9880bebbbd4d156d84f945feec5d61eb9d118275701a4f10fd0c64ce7ca286` |
| original `explicit_http_host_selection_is_stable_for_active_local_owners` | 0, 1pass/0ignored; unbound embedding fallback/host selection preserved | `metadata-host-selection-test.log` `762b19e347768946661060f54d1bb113123151d8d4aab8aee18a7cd9124292b4` |
| real `blocking_probe.py ... --mode language-installed` | 0; correct JSON/404/same owner/task delta0/controller joined/one explicit activation | `metadata-direct-language.log` `8ab6e86a991580b397fcb06c545b680b9737550e2fa8ca468542ecafa30bd6db` |
| affected `response_observation_probe.py ...` | 0; seven exact file/SSE terminal events plus RST/truncation/connected watchdog cancellation and B isolation, actual registries0 | `metadata-response-observation.log` `6208320276fd3c9d7b15dc0a349709d8fb1d5b2ed508223560b35f54b3a029d4` |

An initial external build correctly rejected access to the private collector
module. It was fixed with named bridge operations, not a visibility widening;
failed/stale-binary runs are not positive evidence. The final normal build
reports Core140/external16/root3 warnings (external async reader is now used),
Core lib-test160; no global warning-clean claim. Helper rustfmt/Python/diff checks
pass. Later changes only strengthen test assertions/readiness and documentation,
without changing verified production behavior.

The same isolation fixture now chooses addresses by provider identity rather
than activation order, and its stdin controller breaks on shutdown and joins.
888 head643f2b375 touches different entry functions; our entry change is only
source VM construction, with no shared test registry/runner/script/lock edits.
UI7b323bc/Man e6cc06f remain clean/unchanged. All processes ended; only Markdown
evidence is staged. No whole tier/docs_gen/GUI, agent/message, push or merge.

T-02 remaining: configured launch handoff, per-instance failure/logger terminal,
ordinary parked/async response observation, full actual consumer and physical
package/API/command/SD/four-repository schema freeze. Original test-ID migration
and full execution accounting stay T-08; local probes do not replace that floor.
Web is not created; Plan remains executing1/12. Final independent review,
merge-owned gates/deposit/archive/guard are pending. UI/Man successors still
await full U890.


</details>

<a id="e890-8e4ec6f52e774248"></a>

<details>
<summary>work\configured-instance-state-r2.md · SHA256 8f18b0a8ab46f715ed8ca2b584f01ba6eeb4c2d90ed627d6b695841cf99a850f</summary>

#### PLAN-890 r2 — configured instance state and control

T-02 partial, still executing 1/12. Baseline `bcf661295`. Previous goal turn
was progress: actual committed external runtime/VM port and lifetime budgets.
This unit closes configured-service observation/control sharing on the tested
paths. It does not close the complete T-02 contract or Web relocation.

##### Single-source instance state

`ServiceObservation` owns identity, state, counters and bounded log sink.
Existing free functions are legacy process facades over that one implementation;
configured `ServiceRuntime` allocates a distinct observation and binds its
shutdown sender once. A rebinding attempt is rejected. Transport, owner state
transitions, health and shutdown consume the runtime's actual instance. Pending
events retain their owning observation across threads and terminal callbacks.
Complete/cancel and byte counters now share the same exactly-once gate.

Instance IDs retain the existing `svc-<pid>-<hex>` shape, with a monotonic atomic
stamp ensuring two observations cannot receive the same stamp. No schema field
or profile/limit/CORS support was added. Per-poll policy selection restores the
previous state on Pending and nested calls, so configured dispatch does not
emit legacy wildcard headers. The Core temporary startup bridge was adapted
to the same observation instead of losing its startup/error events; it still
must be removed when T-05 moves the framework.

##### Strong oracle and accepted controls

The external adapter runs two real programs with distinct strict JSON configs,
origins and ports. The controller requires exact per-program random values,
distinct identities, correct bound address and stable config/identity after
the second startup. Ten A requests must not change B completions or add to B
request totals (the one intervening B snapshot accounts for exactly one).
Allowed CORS origins are exact; no Origin receives no allow header. Actual
GET with an unapproved origin still executes without authorization headers;
an unapproved preflight returns 403. Headers-only A shutdown returns 202/true,
closes A, and leaves B ready with its original identity. Both futures join and
real scope/file/upload/stream/live-op registries are empty.

The fixture-only shared-observation mutation must fail on shared identity while
both services still drain. It exercises the actual runtime factory/health
path, not a dummy counter. Initial probe inputs incorrectly requested sink
capacity 16 (minimum is 64) and initially expected CORS to reject an actual
GET. Those setup/oracle errors supplied no product-failure credit. They were
corrected against the existing contract and deployment Spec §4, which explicitly
forbids using CORS in place of authentication; product CORS policy was not
weakened or changed to satisfy the probe.

##### Evidence

All logs local-only under
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Actual result | Log SHA256 |
|---|---|---|
| `service_state_probe.py --exe .../isolation.exe` | exit 0; identity/config/counters/CORS/own headers-only shutdown; B ready; both owners drain and resource tables zero | configured-service-isolation-final.log `55b1cf4451dfd7478a45c72d49e7f600716b10f532f358fdee63f1f525591d5d` |
| Same oracle with `--shared-observation-negative` | expected exit 1 on shared instance identity; both owners drained; wrapper exit 0 | configured-service-observation-negative.log `b01c914f14002b291723b946d4efb2fa9ca75842a52e59507392dc40b08614e6` |
| `cargo test -p auto-lang-core http_service_observability::tests -j 4 -- --test-threads=1` | exit 0; 5 passed, 0 ignored; legacy facade plus real emit-path overflow, owned event/counter isolation | instance-observation-module-tests.log `54d928f5fc43737b56824c06c9de5a2248b9059fb7ff055de6648f47c7a18b98` |
| Final named observation test after ID stamp change and both terminal orderings | exit 0; 1 passed, 1630 filtered; complete-first/cancel-first each count once, correct owning byte totals | instance-observation-final-id-test.log `577ecead43f2c70e52cbc802ba6cc9837d4d2206744df01d8a7e33fb722fec3b` |
| `instance_runtime_shutdown_is_bound_once_to_its_own_receiver` | exit 0; 1/1; duplicate binding refused and each watch changes only its own receiver | instance-shutdown-binding-test.log `757de8756abba7b7cea1c7d1bdf16499f7f6472288f0b973fb3c5f67a5ecdaa6` |
| `instance_policy_context_restores_after_pending_and_nested_owner` | exit 0; 1/1; Pending and nested policy state restore | instance-policy-context-test.log `9280e616fe55bea3fcfeb1c4c9f4f7b1a321defd6930f3cba3aed35db8a15846` |
| SDK probe `cargo build --features external-runtime --locked -j 4` | exit 0; Core 141 warnings (142 baseline minus unused import); external probe 17 source unused/dead warnings (previous 18), no production-clean claim | configured-service-isolation-build-final.log `e310ae40de2c304328bc083cdf07b20bc9454be9bb890c30a28105314b2259ab` |
| External reply replay | exit 0; exact sync/async file, HEAD/Range/ETag/errors/SSE and terminal registries zero | instance-observation-reply-replay.log `94f5617adaa4632d647eb16b5f19ffa69af7b0b7d6f7a1b00cca99ae2f62bede` |
| External upload replay with cancellation | exit 0; original 12MiB, 409/422/401/400, real active receive/RST cleanup and zero registries | instance-observation-upload-replay.log `31414d48546e665b72d4582ea7e390c66cd87e2abe4489ae83c345a615cfc57e` |
| External live-owner replay with admission pressure | exit 0; metadata/owned bodies/default replacement+clear, A=503/B=200, RST recovery, independent shutdown and zero registries | instance-observation-owner-replay.log `ddbc6cc0fd980a2ee7d2a1432a7a9b357582c1b1dfeca3b0603c0405e4b186e6` |

Seven distinct scoped unit cases were verified (5 observation + 2 config).
The later observation replay is one of those five, not extra coverage. Other
four module results are reused after checking their implementation/inputs were
unchanged by the ID stamp and test-only terminal-order extension. The SDK build
precedes only block-local line wrapping; no behavior/dependency/config change.
The negative's ID equality condition is unaffected by the opaque stamp format
correction; its shared Arc factory mutation and inputs are unchanged.

##### Remaining work and coordination

Legacy process diagnostic/shutdown facades, direct listen/back-proxy, legacy
rate and axum-builder mutable state, producer scratch, exact explicit deadline
and complete generation/non-HTTP/package/API/SD/four-repository freeze remain
T-02. In particular this unit does not certify multiple unconfigured legacy
services' diagnostic control isolation or logger thread teardown. Framework
and executor relocation remains T-05; no Web repository created yet. UI/Man
structural successors still wait for U890. No whole-tier gates, GUI, shared
runner/registration/lock writes, merge or push.

Main advanced independently to `b0a6c869da6d92307a0afd7a8ce0216eaaa4dd33` and
889's tree was removed by its owner during this turn. Actual worktree inventory
was reread; no missing 889 tree was recreated or repurposed. Main product paths
remain clean; 888's tree remains distinct. Candidate inputs will be reconciled
again in the required final serial merge window, not silently rebased here.


</details>

<a id="e890-5ee6016527cb6366"></a>

<details>
<summary>work\direct-language-activation-r2.md · SHA256 359bf40b14f7b00a546a2f15bca4a070c79b7657b13f6456ca8c807d4c3f7837</summary>

#### PLAN-890 r2 — real language direct listener and one activation

Bounded T-02 progress from language parent `d31081334`; still executing 1/12.
No Web repository or final delivery claim.

##### Actual language entry and owner completion

The standard VM declaration is callable as
`use auto.http` + `http.server_listen(http.server(), address)`. Both the qualified
and wildcard-import free function spellings reached the real native and the
explicit missing-host diagnostic. This closes the earlier uncertainty about
whether a complete language program can reach the borrowed port: it can,
without private SDK APIs or a compiler/linker change.

The advertised method spelling `server.listen(...)` remains an unresolved
Server.listen symbol in the current baseline; this is not a newly working
method API. The loader, HTTP declarations and corresponding code-generation
algorithm were not changed. The exploratory type-qualified spelling exited 0
but did not reach the checked adapter and is not credited as a service entry.
Only an exact nonce HTTP response plus the real external runtime callback can
pass the accepted positive oracle. No generic compiler/stdlib registration fix
or unrelated new language feature was added.

A direct call with annotated handlers would otherwise return to the post-main
annotation hook and start a second listener. Core now records successful
blocking host completion in a **private per-AutoVM Cell**. The automatic host
handoff returns without activating again for that owner. The marker is written
only after a successful direct host return; failure leaves it false. It is not
a process-global route/host flag, does not block another explicit listener call,
and does not suppress a fresh VM. The VM remains !Send on its owner thread;
no raw pointer, VM clone or public field/method visibility expansion. Only two
crate-private accessors were added. Core lib's automatic hook itself was not
edited; the decision is inside the existing owned host handoff.

The bounded external fixture now retains and joins its own shutdown controller
thread. The controller signals its stop flag and wakes its owned accept socket;
real listener return, handler task count, owner thread and controller join are
all required. Production slow-body cancellation/logger-thread lifetime is not
certified by joining this fixture controller.

##### Actual outcomes

- Locked external SDK binary build exit 0 (`external-runtime`, `--bin blocking`,
  -j4). Core 141 and external source probe 17 existing warnings remain.
- `blocking_probe.py --mode language-installed` exit 0: real language source,
  main calls the qualified native, real annotated handler gives the exact random
  JSON value, missing argument/invalid integer return 400, unknown route 404.
  Shutdown wakes and returns normally; fresh handler tasks have zero delta;
  original VM thread preserved; controller joined. Exactly one external blocking
  activation, zero post-main async activations; child exit 0 within deadline.
- `language-missing` exit 0 wrapper / child 1: the real native fails with
  RuntimeError containing explicit-install diagnostic and the requested port
  remains bindable. This is before post-main auto activation.
- `language-unsupported` exit 0 wrapper / child 1: an installed async-only custom
  provider fails at native execution with the unsupported-blocking diagnostic;
  port remains bindable. An empty/stub native cannot pass this control by falling
  through to a successful async-only post-main provider.
- Existing `installed` annotated automatic-entry replay exit 0, exact JSON and
  argument/404 checks, real bounded return/tasks/owner thread/controller join.
  Fresh automatic consumers still activate normally.
- Existing Core host-selection unit ID passes 1/1, zero ignored. Its original
  active snapshot/replacement/missing/unsupported assertions remain. Added
  checks complete one owner, clear the process host, accept that owner's
  already-completed auto handoff, reject a fresh owner without a host, and then
  run a fresh owner's automatic callback with a reinstalled host. A controlled
  host rejection leaves its marker false. Event identities/counts prove the
  distinction is per owner; no new global-state test race was introduced.
- `cargo check -p auto-lang -j 4` exit 0, root's 3 existing warnings. The final
  binary rebuild adds only fixture controller-join code / cfg(test) assertions;
  production checked source is unchanged. Changed isolated Rust files formatted,
  git diff check exit 0. No whole-tier tests, GUI or docs_gen.

All source/input/build operations were inside the dedicated worktree. The
Python source fixture now lives under its ignored target/direct-language folder;
its logs below remain local-only in main attachments. No evidence JSON/log is
staged.

| Local evidence | SHA256 |
|---|---|
| `direct-language-spelling-audit.json` | `d784effb6c1316dcd4bbb3896a3fd1f92ce9fd429c463b645319155abc2d5b87` |
| `direct-language-build.log` | `a5b55cd7fc58ed53aac87355062f5fc1223126fd4a590896ce40bb687a647cae` |
| `direct-language-root-check.log` | `cd45a58845831acc9a0d746eb065434e82c5554dd4e9ca279916458a7f4486c5` |
| `direct-owner-activation-test.log` | `843a33f8ebf79a3b99d336e2fd60114eadfcfd532073c8622bc17341d9ab49aa` |
| `direct-language-installed.log` | `8ab6e86a991580b397fcb06c545b680b9737550e2fa8ca468542ecafa30bd6db` |
| `direct-language-missing.log` | `0ecac49703887f8db063a27fd8861e0af9c6dd734f694bf72bc0fa949264f52c` |
| `direct-language-unsupported.log` | `ecd91ebdc66044c4241f9ce39d66d8af31d840f0ed15904421b7833570340c71` |
| `direct-installed.log` | `a73c83218b903d294cc60beaa07304a2482b93e3b7b3c8fe377e3d989b124091` |

##### Remaining T-02 and coordination

This proves the existing VM free-function direct-listen grade and its successful
return coordination, not every legacy listener's response capabilities. The
synchronous server-handle source still has its original JSON/media semantics;
legacy `serve_blocking_stdnet` has distinct SSE/CORS behavior and remains a
separate migration obligation. Method-spelling cleanup/new capabilities were
not substituted for the real requested Web migration. Configured direct launch
options / active config ownership, legacy controls/producer/rate/deadline state,
full actual consumer and owned-terminal controls, and complete physical package/
public API/command/SD/four-repository receipt freeze remain T-02. UI/Man assembly,
framework exit, final independent review and merge-owned gates still pending.

888 was checked at eb80a4b89: no engine.rs/Core lib changes relative to the
language base and only its own diagnosis Markdown untracked. Only the private
VM bookkeeping and 890's existing host/fixtures were changed. No shared test
registration/runner/lock/final merge window or message/agent was used. UI remains
7b323bc and its structural successor, like Man's, still waits for U890.


</details>

<a id="e890-1bb5ac746ed89fd6"></a>

<details>
<summary>work\explicit-host-prototype-r2.md · SHA256 667bbbed01e76442de2b18558edd6003041401c07de83d475458c2fe66139441</summary>

#### PLAN-890 r2 — explicit local host prototype

2026-10-30. Partial T-02 evidence; not the complete Web port or delivery.

##### Actual handoff

Core's post-program API-service path now takes a snapshot of an explicitly
registered `HttpServiceHost`. There is no implicit provider installation.
No service routes means ordinary execution does not consult the HTTP host.
Routes with no provider return a named missing-adapter error, rather than
silently skipping service startup or retaining an automatic old fallback.

The registry offers explicit replacement and atomic install-if-absent. The
snapshot point is service activation, not VM construction; frame/clipboard
provider behavior is unchanged. An already activated local owner keeps its
chosen future and route snapshot after registry replacement. `HttpVmOwner`
holds the VM privately as Rc; the service future is local, not Send. The host
object is Send+Sync, which does not make its VM owner movable across threads.

This is a bounded source experiment. Existing startup/configuration logic was
moved once from the root pipeline into the explicitly named temporary
`run_existing_runtime_for_probe` bridge. Only the external fixture installs
the prototype adapter and calls that bridge. The HTTP implementation still
exists in Core; this is **not** evidence of a finished Core/Web package DAG.
T-05 must remove the temporary bridge and Core's framework implementation.
Leaving or renaming this bridge as a permanent fallback cannot pass V-16.

The broader direct-listen/back-proxy/native service consumers, real instance
resource isolation, metadata ownership and all shared generation interfaces
still need their required migration. The prototype does not certify those
unmodified consumers or the original whole HTTP suite.

##### Executed evidence

- External SDK plain mode: result 42, child exit 0, no host activation or
  listener startup, no installation.
- External SDK missing mode: child exit 1 and the explicit missing-adapter
  diagnostic, no listener startup. The same public `run_with_capture` entry
  is used; a silent-success fallback would fail this control.
- External SDK installed mode: the fixture explicitly installs the host;
  actual TCP returns HTTP 200 and the exact random token, shutdown is
  delivered and the process exits 0.
- Installed host upload replay: original 12 MiB accepted body, exact file
  download, conflict preservation, 422 business rejection/staging cleanup,
  headers-only 401 and shutdown all remain passing.
- Named Core selection test: 1 passed / 1,624 filtered out, exit 0. The actual
  `serve_registered` futures are invoked with distinct local VM owners. A is
  activated and parked; global host becomes B; a later owner dispatches to B;
  A then completes with its original host and one-route snapshot, B with its
  own two-route snapshot, on each owner thread. Repeat install preserves the
  custom selection. This test proves handoff selection, **not** simultaneous
  real-network scope/metadata isolation.
- Opt-in compiler negative: requesting `HttpVmOwner: Send` exits 101 with
  E0277, specifically `Rc<auto_lang_core::vm::engine::AutoVM>` not Send. The
  check wrapper initially expected an unqualified type spelling; it was fixed
  to accept the actual qualified compiler diagnostic and rerun successfully.
  This is an expected rejection, not a product build failure or a skipped case.
- Normal probe build exit 0; Core product warnings are 142, down from the
  original 147 after inert warning cleanup in the pure contract source cut.
  Remaining baseline warnings are not represented as fixed.

All evidence is local under main `docs/plans/attachments/890/work/`:

| File | SHA256 |
|---|---|
| `explicit-host-plain.log` | `a430f7c05e6f65bb26c33219af3b7d15752d9cbb221f50a8f8dac1587726313d` |
| `explicit-host-missing.log` | `471911e6054b9caaa9f2b2bfd7ae67c99e90126f0c46fbe74daeabf8e3466f4f` |
| `explicit-host-installed.log` | `5d3c4ba33d65970dfbe5d6e2d8ac9ca5d4766199626990e8dcde9773b6cbbfcb` |
| `explicit-host-upload.log` | `cffbb28ade114f1baa28ed91504627a435fab8e3006483353a1087c978e12f2b` |
| `explicit-host-selection.log` | `4b256f64d901d8dc3e4614669307d90b9c6eb54006a0280c53381c8d4d1e5bfb` |
| `explicit-host-owner-send-negative.log` | `2d974e746ed64a728e401ecd72d88038aceddf253c8eda669220433370633817` |

The files were checked against 888/889 diffs before this write unit: neither
owned the modified root `lib.rs` or `host_capabilities.rs`. Their occupied
entry/test-registration/governance/HTTP-support functions remain untouched.
No full tier, final review, merge or Web bootstrap was performed.


</details>

<a id="e890-55e3797d8bb79cf1"></a>

<details>
<summary>work\external-runtime-port-r2.md · SHA256 19f9615559355cedb0ddadc76daca55be7f1eb81e4e4df40eb22e287758a01ca</summary>

#### PLAN-890 r2 — actual external runtime/VM boundary

T-02 partial; executing 1/12. Baseline after admission repair `6cba1e6f2`.
This closes the reproduced crate-boundary/lifetime coupling for the executed
paths, not the complete T-02 freeze or framework relocation.

##### Implemented candidate SDK port

- `HttpVmOwner::into_local_parts()` consumes its local owner and supplies the
  existing public `Rc<AutoVM>` plus `Arc<HttpProgramMetadata>`. No VM field or
  method visibility changed, no VM/AST/Store clone, no raw pointer, no Send
  coercion. The metadata and route/parameter declarations have one owned source
  in `http_service/messages.rs`; previous paths re-export the same types.
- `VmHttpBridge` has eleven named operations: completion notification/wakeup,
  discard async result, wait current/legacy stream readiness, close legacy
  stream, insert upload capability, take upload receipt, and finalize transfer,
  file-response or upload resource IDs. Each delegates to the existing native
  implementation; maps/notifier mutation are not exported. The one conversion
  call now uses its already-public SDK re-export instead of a private module.
- `RequestContext` reports four typed resource kinds, cooperative cancellation
  and deadline extension to the serving host. Its Arc payload is Send+Sync;
  its TLS restoration guard is !Send, nesting/unwind safe, and never moves the
  VM. Late registration checks terminal state under the resource-group lock
  and immediately reclaims a resource created after cancellation.
- Existing source's `ScopeGuard` now uses that generic hook. Native file,
  upload, transfer and stream operations report to the host context, including
  an external framework's scope. Upload phase deadlines capture the host Arc,
  not a lookup into Core's old framework table. Core's duplicate TLS scope ID
  was removed. Existing CPU cancellation calls resolve through the hook.

##### Actual cross-package execution

The external package includes the original service/transport/child sources by
path. The first audit had module-resolution setup errors; explicit existing
child paths fixed that tooling issue (not semantic failure credit). The next
audit produced 16 errors (13 privacy, 3 unresolved private-path/alias uses).
Named SDK operations and existing public paths removed those couplings without
publishing the private modules. Required-feature external check passes.

`external-runtime` adapters now hand the actual VM/metadata to the external
crate's own `serve_with`/transport/dispatch/scope. They do not invoke Core's
temporary `run_existing*` methods. Runtime marker checks reject an executable
built in the old mode. The scope count is read from the external crate; native
resource counts are read from Core's actual single language registries.

Real TCP covers JSON, exact sync/async binary file, HEAD/Range/ETag/404/500,
ordered SSE EOF, original 12MiB upload/exact download, occupied-target 409,
business 422, headers-only 401, and path binding 400. A partial raw upload must
create a nonempty staged file before RST, then clean staging without publishing
partial bytes while an unrelated marker request remains responsive. RST is
actual socket reset, not a forced child kill or a timeout-as-success.

The two live VMs additionally verify install-if-absent, explicit provider
replacement, clearing the default after activation, same-named metadata,
file/SSE/upload classification, one-owner body pressure (A=503, B=200), RST
permit recovery, and independent first shutdown while B stays 200. Both owner
futures join; scope/file/upload/stream/live-op registries return to zero.

##### Evidence

Local-only directory:
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Command/check | Actual result | Log SHA256 |
|---|---|---|
| External required-feature audit after module resolution, before SDK bridge | expected compiler exit 101, 16 errors; concrete private boundary inventory | external-runtime-boundary-resolved.jsonl `4a9edc8e339e1b3036724d7bd41326c3ac761a9f71630bc8f5d86e5a2b7828d5` |
| Same external lib audit after named bridge | exit 0; no private module re-export | external-runtime-boundary-bridge.jsonl `5612eb12d258967b45ab1a370f13827c5b78d8da5b9c17dbfd1b5e131b0629fc` |
| External lib with actual owned handoff function | exit 0 | external-runtime-owned-port-check.jsonl `6cc7f26d71127356e554f2d78472199725e441d8ce58f85dce2f3d9c724269a9` |
| `cargo test -p auto-lang-core host_context_restores_parent_and_reclaims_late_native_uploads -j 4 -- --test-threads=1` | exit 0; 1 passed, 1626 filtered; real native capabilities assigned to nested scopes, parent restored, late upload immediately dropped, four drops/zero polls, permits restored | host-request-context-native-control.log `5bb0c9db208de58107ef214850397067656bd7b6df6d4acf9e1cda696387ef0f` |
| SDK fixture `cargo build --features external-runtime --locked -j 4` on final local guard type | exit 0; Core 142 existing warnings unchanged; external source probe 18 existing-source unused/dead warnings, not declared warning-clean production | external-runtime-sdk-build-final-local-guard.log `a258352362465c27db4e57426b7fc31aa8aa6aae934bc8d7a6cd457fb468f7e6` |
| `response_probe.py --expect-external` | exit 0; exact file/SSE/protocol branches and terminal registries zero | external-runtime-reply-final.log `c8a7290be44bbd82318a6b51647749603494d3585803cee2e32a7ec67b444738` |
| `upload_probe.py --expect-external --cancellation` | exit 0; original 12MiB + rejects + actual receive/RST/staging cleanup; registries zero | external-runtime-upload-final.log `033896cd403abaedf93f4808d5b7ade11500942e5d21067361d11689801ffd06` |
| `isolation_probe.py --expect-external --admission` | exit 0; actual two providers/VMs, metadata/resources/budget/default replacement+clear/independent shutdown; registries zero | external-runtime-isolation-final.log `50b3327755e0a12bd6d465d0a9740206a0ec688c837ee8dd4603cfd7410413d2` |
| Fixture check with `--features external-runtime,compile-fail-owner-send --locked` | expected compiler exit 101/E0277 for both Rc<AutoVM> owner and TLS guard; controller exit 0 | external-runtime-local-ownership-negative-final.log `7eceef6f8ad4e42301edc7d28f0d9fd9bb95a81724bea3ef5e6e86d1ee2ccb1a` |
| Missing host (`probe.py --mode missing`) | controller 0, child 1; explicit diagnostic/no listener | external-runtime-missing-final.log `471911e6054b9caaa9f2b2bfd7ae67c99e90126f0c46fbe74daeabf8e3466f4f` |
| Ordinary language (`probe.py --mode plain`) | exit 0, value 42/no host/listener | external-runtime-plain-final.log `28f423fb79989a13a25941e6de5f5f69445b8d90861c85c8e4657723b4d13232` |
| Fixture tree `--features external-runtime --locked --prefix none` | exit 0; no UI/Man/Tauri/auto-api-gen in closure | external-runtime-tree.log `296a6119107a52b00dce106f9c0b84c2195ee678a092593c05c33b0816528c97` |

The final normal build checks the !Send guard change; final TCP checks were
then rerun on that executable. The context unit precedes only the zero-sized
thread-affinity marker/tuple-to-named-field change; its drop/register behavior,
inputs and configuration are unchanged, and actual final native cleanup is
also covered by the external TCP controls. The upload wait log's existing
"after 30s" text is the configured budget, not measured RST latency or deadline
evidence. Required explicit deadline branch remains pending.

##### Remaining gate and implementation ownership

The external fixture is not a production Web package. Core still compiles the
original framework and owns the original executors until T-05. Its old startup
probe methods must be removed there. Configuration/control/observer and legacy
rate state, axum-builder state, direct listen/back-proxy, compiler scratch
concurrency limits, shared generator/non-HTTP full cut, precise package/API/
command/SD/four-repository schema freeze remain T-02 obligations. Required
inline test migration/dev closure stays T-08, full independent distribution
T-09. No historical suite execution credit, no T-02/whole-work pass, no whole
tiers or GUI. No shared runner/registration/lock changes, merge or push.


</details>

<a id="e890-87d69d695303f960"></a>

<details>
<summary>work\feasibility-and-contract.md · SHA256 cfdc162d585eea28a0321c5232dd42b96061abb8b1cac4aed438938ced91d1b5</summary>

#### PLAN-890 T-02 端口、包图与阶段结论

2026-10-30；合同基准 r2。**阶段结论：needs_replan，T-02 未完整通过，T-03 不准入。**
用户最新边界为只收口 T-02 后暂停；本文件不授权建 Web 仓、迁移下一阶段、
合入 master、归档或清理工作树。当前 Plan 仍 executing / 1 of 12。
下面区分已验证契约和待决候选，不能将候选包图冒作实际四仓交付。

##### 1. 已验证、可复用的端口基线

生产/语言代码基线：`e2ed6ba761b1082af34e759012f53d292cd46346`。
当前主入口位于 `crates/auto-lang-core/src/vm/host_capabilities/http_service.rs`。

| 契约 | 当前实物与已验边界 |
|---|---|
| 明确安装 | `install_process_http_service_host_if_absent(Arc<dyn HttpServiceHost>) -> bool`；重复安装不覆盖；`set_process_default_http_service_host` 显式覆盖；`clear_process_default_http_service_host` 清默认 |
| 快照时序 | 真正服务激活时选择 host，已激活服务保自身 Arc；之后替换/清默认不改变它。普通无路由程序不选择 host。有路由且无 host 明确失败，无隐式旧实现回退 |
| owner | `HttpVmOwner` 持本地 Rc<AutoVM> 与 immutable metadata；`into_local_parts` 只交既有 VM SDK 和 owned 声明；owner/future !Send，网络仅接 owned Send 消息 |
| 现行签名 | `HttpServiceHost::serve(owner) -> HttpServiceFuture`，其 Output=`AutoResult<()>`；`listen_blocking(&AutoVM, address, Arc<HttpProgramMetadata>, BlockingListenerKind) -> AutoResult<()>` |
| 阻塞等级 | `ServerHandle` / `LegacyStdnet` 必须显式区分；async-only host 对 blocking 明确不支持。LegacyStdnet 旧 unit-return wrapper 的“有诊断且不监听”不能作为服务健康通过 |
| 编译声明 | `CompilationMetadataGuard::begin/finish(routes)` 在执行前冻结，`VmHttpBridge::bind_program_metadata(&mut vm, metadata)` 绑定 VM 私有 Arc；params/returns/shadowed/async_returns/routes 都归本程序。正常 execute/test/source-VM/debug 四入口已接；nested/error/unwind/并发编译、main 等待后另一程序编译均有证据 |
| owned 消息 | `messages::{ApiRequest, ApiReply, ApiBody, FileReplySeed, HttpProgramMetadata}` 单一类型身份；upload body 是 owned Send IO 流，不跨包传 Axum Body、AST、Store、VM 指针或借用 |
| 语言资源桥 | `VmHttpBridge` 的具名 future/stream/file/transfer/upload 清理和声明操作；request-context guard !Send，late register 清理与终态协调；不得扩大私有 VM 字段或私有模块可见性来迁仓 |
| body / 线程 | 实际 JSON/SSE/file/upload、背压/准入、RST/错误/drain/typed return identity 已有分项证据。文件/SSE 实际 body poll 计 payload bytes、终态一次；文件 watchdog 中止自身连接。网络 thread join 后才报 Stopped |
| 仍属过渡 | Core 仍编译旧框架，外部 source probe 不等于最终 owner 退出；raw/unbound VM 的 legacy 元数据 cache 不是多实例隔离保证；临时 run_existing_* 探针桥必须在 T-05 删除 |

已验证命令、原始输出 hash 及适用源码见同目录：
`compiler-metadata-owner-r2.md`、`response-terminal-observation-r2.md`、
`network-terminal-r2.md`、`native-policy-owner-r2.md`、`legacy-host-port-r2.md`、
`owned-messages-and-resource-repair-r2.md`、`configured-instance-state-r2.md`。
这里不将这些局部通过扩大为启动/退出/所有普通异步响应全通过。

##### 2. 本次剩余项、实际结论与关闭条件

| ID | 本次结论与具体证据 | 关闭条件 / 后续边界 |
|---|---|---|
| R1 配置启动交接 | **失败**。Core active config 指定端口5077，实际精确业务响应来自 env 5078；active config 未消费。`HttpVmOwner` 不携 launch/config，外部适配器只能另外取得部署输入；当前 Man 恰使用 set_active_service_config + run_language_file | 确定独立于编译 metadata 的 per-invocation launch 输入，保既有配置/CLI优先级；真实 configured SDK/Man 配置、port0 bound、hash、ready/关闭正确，且实例不串用配置。先修订端口设计，不通过读取更多 Core 私有全局来补洞 |
| R2 失败/ready归属 | **失败**。真实占口且有 bind-failed 诊断，自动服务 SDK child 仍 exit0；source runtime 返回 () 后适配器返回 Ok(())，fatal/bound 仍为进程全局面 | runtime 真正传播 Result，ready/failure 属同一 invocation；A 启动失败不能污染 B；SDK错误非零、Man ready 不伪成功。现有 trait 的 Result<()> 可保留，但实现与 ready/control 通道尚未闭合 |
| R3 普通 parked 响应 | **失败**。wire 为200/25，唯一事件却 status=null、bytes_sent=0、outcome=completed；owner 在 bridge 填状态/字节之前提前 complete scope | 在当前模型内可修：普通 parked reply 的 scope/event 移交 bridge，保发送失败、超时、取消及资源回收。实际 async 成功/错误/取消事件一一匹配。不是必须新用户语义决策；仍属未完成 T-02，不后移冒通过 |
| R4 日志线程 | **设计/实施缺口，未验终结**。SinkState 仅存 tx，install_log_sink 丢弃 spawn JoinHandle；已有 request 输出不证明线程 join | 决定 Web runtime-owned 日志 session、停止发射/关闭 sender/排空与 join 顺序及错误策略；不能在业务 reactor 上阻塞等待，也不能只等进程退出 |
| R5 物理包/API/consumer 冻结 | 下文已形成可审查候选和具名消费映射；启动模型未定，故最终 API/物理 DAG **未批准**。Web/共享载体尚未创建，只有真实 source/metadata probes | R1/R2/R4 决定后补齐函数→包映射及公开 API；闭包不得反向依赖；用既有独立源/生成消费者核所需接缝，不提前做 T-03/04 大搬迁 |
| R6 命令/四仓 schema | 当前可运行命令和887基线约束在下文；新增四仓 runner/包命令 **尚未实现/执行** | T-02 先确定扩展字段及拒绝规则；T-08才在串行窗实施共享 runner/注册，T-09做完整发行。三仓历史 pass、list-only/filtered receipt 不能认证新四仓 |

本次不再修改生产代码。R1 使现行端口不能作为完整迁移冻结合同；
R2/R4 还缺启动/退出所有权。按原 T-02“失败给 needs_replan、不大搬”处理。
R3 是明确的工程未完成项，不以“需要决策”掩盖，也不转为历史债。

##### 3. 待决设计与建议（未冒充批准）

1. **D-T02-01 launch 归属**：建议独立中立 `HttpServiceLaunch`（名称候选），
   在 VM 执行前与 invocation 绑定，带已解析不可变配置及独立控制/ready 关联。
   Core 如需类型依赖只指向轻合同，不指向 Web runtime。另一方案是在 VM 构建时
   固定带配置的 provider；该方案改变当前“激活时选默认 host”时序，不能静默采用。
   不把部署配置混进 compiler metadata，也不继续依赖 ambient process config。
2. **D-T02-02 生命周期与结果**：建议保现有 `AutoResult<()>`，由 Web session
   拥有 ready/control/worker/日志，runtime 返回真实 Err，完成排空与 join 后才成功返回；
   进程全局 fatal/bound 不再作新调用权威。需定义 SDK service future 被丢弃时的
   取消/回收责任；目前不能从正常 shutdown 证据推断该边界已支持。
3. **D-T02-03 freeze**：前两项确定后才给最终函数签名和包图放行。
   这些建议保持原目标/仓集合/验收标准，不授权新增产品或降低门槛；
   下次执行先走有界方案修订，再在原树修 T-02，不直接开 T-03。

##### 4. 包图候选与职责边界

下列首版名/版本 `0.1.0` 仅为本地候选，不是已创建/发布的包。

| 包 / owner | 源与职责 | 依赖方向 |
|---|---|---|
| `auto-lang-core`（现有） | 语义/转译/VM、编译声明、值/句柄/资源桥、普通HTTP标准库与显式host端口 | 不依赖 Web 实现；若 launch 类型必需，仅允许固定单源轻合同 |
| `auto-web-contract`（拟 Web 仓） | 当前 http_service_config/contract.rs 的严格配置及纯 Host/CORS/单代理政策等轻合同 | 无 Core/VM/Axum/UI/Man；owned VM消息仍在语言端口，不把含VM依赖的消息机械塞进合同包 |
| `auto-web`（拟 Web 仓） | 服务框架、VM/Rust服务装配、HTTP生成及客户端、资源/CLI库出口 | →Core公开SDK、轻合同、受限共享生成、a2r/media等基础；normal/build/必需dev不依赖UI/Man/Tauri；不得依赖混合auto-api-gen |
| `auto-generation-core`（拟语言仓） | shared_generation/rust_workspace/sidecar/source_owner 四模块，限既有职责 | →语言公开API/project-model等原基础；不引Web/Man/UI/Tauri，不成为common大包 |
| `auto-api-gen`（现有兼容/非HTTP） | Tauri IPC装配、非HTTP入口；原HTTP入口具名委托Web | →Web+共享载体，Web不得反向依赖它；不能保第二份HTTP实现 |
| `auto-media-core`（现有） | renderer-neutral photo实现；UI旧路径已薄re-export | 无Web/UI反向边；保持image-codec条件与原十例身份 |
| Man/UI（消费方） | Man CLI/工程/通用子进程；UI界面及非HTTP适配 | HTTP→Web，通用生成→共享载体；本次未启动其结构后继 |

共享载体受限函数清单（包括必要私有支持函数，共25个，原机械保守证明不变）：
bind_workspace_lock、lock_freshness、assembly_freshness、try_full_parse、
transpile_db_to_rs、transpile_back_module_to_rs、transpile_back_module_to_rs_with_proofs、
qualify_a2r_std、replace_in_code_regions、code_region_mask_text、post_process_companion_rs、
promote_api_types_in_companion、post_process_db_rs、append_clone_for_borrowed_fields、
add_mut_to_let_collections、append_tostring_for_str_fields、strip_collection_new、utf8_len、
balance_paren、fix_borrowed_slice_returns、primary_type_name_pub、extract_db_fn_names、
extract_api_lenient、parse_fields、parse_params。
SidecarDecl仍由auto-project-model单源。完整映射基于T-01不可变语法树清单，不能
据文件名整搬TypeScript/Tauri共用发射器，也不能重生成更小floor。

##### 5. 实际消费者与拟公开出口

| 当前消费点 | 下一阶段归属/出口及未关闭处 |
|---|---|
| Core lib自动hook、entry source VM、script debug、custom SDK | Core显式host桥；Web提供安装器；configured launch与结果尚待D-T02-01/02 |
| stdlib server_listen、旧serve_blocking_stdnet | 同一桥分ServerHandle/LegacyStdnet；保已验等级差异与明确缺安装诊断 |
| Man auto-man-runtime/http_service.rs::VM轨 | 当前 set_active_service_config + language::run_language_file + global bound/fatal；必须迁为Web公开session消费，R1/R2阻止把当前链当已接线 |
| Man同模块Rust轨 | 当前 api_gen::generate_api(project,"rust")、shared workspace与真实子进程；HTTP生成转Web，工程/进程编排留Man |
| UI run_vm.rs:315 / run_vue.rs:321 | 当前 generate_api(...,"rust"/"vue")；HTTP生成/客户端转Web正式入口 |
| UI tauri.rs:102,171 / rust_ui.rs | Tauri装配留非HTTP；原try_full_parse/transpile/post-process/workspace/sidecar走共享载体；保旧入口兼容与单源 |
| 外部生成Rust服务 | 当前生产generate_api(...,"rust")已做无UI实际编译/HTTP；最终producer/manifest/源码receipt须切Web pin后在T-09 fresh/locked/重定位重验 |
| Blog固定源 | `7f9b675587509c7b22c3a2f02088ea4d8a47d4ba`观察pin；T-10独立数据/产物和双UI触面验证，业务WIP不动，不代勾BLOG-004 |

拟Web generation出口沿用 `generate_api(project, mode)` 和
`backend_generation_is_fresh(project)` 的可追溯HTTP职责；Tauri mode仍由原owner处理。
最终模块路径/CLI flags随D-T02-03冻结；没有把尚不存在的`cargo -p auto-web`命令记作可执行证据。

##### 6. 现在可运行的检查与候选四仓收据

所有下列命令 cwd=`D:/autostack/.wt/lang-890/auto-lang`：

```text
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin web-port-probe --bin isolation --locked -j 4
python test/fixtures/modularity/web-port-probe/t02_closeout_audit.py --sdk-exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --isolation-exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe
```

本次build exit0；audit **exit1，3项失败**。这是未通过收据，不是“反控通过所以阶段绿”。
raw：`D:/autostack/auto-lang/docs/plans/attachments/890/work/t02-closeout-audit.log`
SHA256 `be1c501931841e073411e841d1640e86a45bf4fa915764b232681dc3f0dab625`；
build log SHA256 `d93bffbf512205b34bdc3708d0c51e2c895e025ea48d6be7c43b252cf775c942`。
日志sink缺句柄是源码证据（http_service_observability.rs::SinkState/install_log_sink），
不冒作已执行的线程终态测试。新探针不改生产实现/注册/共享runner/锁。

四仓扩展必须建立在正式887，而非888 WIP：本树与`dd916c184`下
`scripts/run-full-regression.py` blob均 `e8458b27e32521dff62ee611eb1814193c86b445`，
`three-repository-fixture.py`均 `cbff4c581eadbf763eeda278c40c1167b7b9053d`。
候选`--with-web`模式必须：

- required owners=language/web/ui/man；down仅固定基础依赖，不替代Web；记录各root、
  commit/source/lock/resource身份与运行前后身份，必须含Web实际执行腿。
- 保旧三仓receipt解释；若新增`four-repository-fixture/1`，复用原checks的
  id/status/detail、pending/summary，不创第二个测试PASS裁判。
- 扩现有regression roots/identities/results，明确four-repository模式与required legs；
  保选中ID、list/run exit、日志/hash、原子义务映射；不能只报聚合数量。
- 缺owner/源/hash/required ID、重复/0选择、漏执行、身份漂移必须非零；
  list-only、filter、manual/ignored/pending或三仓历史pass不得冒完整四仓pass。
- 原6档24869历史行/4683原ID及696 Web行、55宏边界、17资源义务按原清单守恒；
  列举执行数仍0的条目不能因已盘点计作运行。新增SDK/探针用例单列，不冲抵迁移缺项。

上述为待实施schema约束，未写共享脚本、未跑全档、未声称四仓验收通过。
T-02重开应最终确认字段和精确包/leg命令；T-08串行窗口才实施注册/runner。

另保留已具名的父版本基线：`test_plan734_tauri_fixture_matches_generator` 的宏换行
byte差异已在父源码复现；同组非HTTP定向为12 pass/1父版red，不能表述为全部兄弟全绿。
Core140/external16/root3及Core lib-test160存量warnings未清，不冒最终健康门通过。
本次只改探针/文档，未运行根cargo测试档或docs_gen，也未追加无关GUI/全量回归。

##### 7. 迁移顺序与停止点

本次停在T-02 needs_replan。下一入口是同一Plan/同一组树的有界T-02方案修订与修复，
先关闭R1..R4，再确认R5/R6；不新建Plan/替代树，不直接开Web仓。
只有T-02真实通过且用户另行授权下一阶段，才T-03 bootstrap→T-04语言桥/共享载体→
T-05/06运行时/生成→T-07消费者→T-08守恒→T-09发行/恢复→T-10应用触面→
T-11独立终审→T-12冻结门/各仓合入/规范/归档/guard。
本次不执行其中任何后续步骤；保全部现有树、分支、未关闭风险和历史基线红。


</details>

<a id="e890-7ba5e2e071eb3811"></a>

<details>
<summary>work\instance-admission-r2.md · SHA256 1feab90f5d52a7e6136ddb572a43fe128f5a8e19c1a357c31dfc2b1ecdc3ee18</summary>

#### PLAN-890 r2 — per-instance lifetime admission

T-02 partial, not a task pass. Baseline `5403d0fa3`; one authoritative language
tree. No Web repository or mass migration yet.

##### Actual failure and fix

The existing network called `create_scope`, which used a process-global
OnceLock semaphore initialized by the first caller. With a child-only request
budget of one, an unread large file from service A exhausted service B too.
The real two-VM oracle observed A=503 and B=503 despite B being idle.

Network service instances now allocate their own request lifetime semaphore
from the effective TransportConfig queue capacity. Each scope takes an owned
permit from that instance and retains the existing body-terminal cleanup.
The old public scope factory remains for existing embedding/tests and is not
on the production network path. Its long-term SDK disposition is part of the
T-02 port freeze; it is not a second framework fallback.

The permanent `isolation_probe.py --admission` control runs two live VMs with
a child-only one-request budget. A real 64 MiB file (written in 1 MiB blocks,
private temp root) plus a 4 KiB receive buffer holds A's response open. A's
next request must receive 503 while B returns its exact JSON value. RST of
that owned socket must reclaim A's permit and restore A's 200. The existing
JSON/file/SSE/headers-only-upload metadata checks and independent shutdown
checks also run; both owner futures join and five real registries are zero.
No global env mutation, shared cache removal, GUI or performance claim.

##### Evidence

Local-only directory:
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Command / invariant | Result | Log SHA256 |
|---|---|---|
| `python .../isolation_probe.py --exe .../isolation.exe --admission` before repair | expected exit 1; A=503, B=503; both owners drained; proves shared budget defect | two-owner-admission-before.log `7efeddfc9c990e1cacffffcb56b02f06624746a3adcf064fe3f0d808aba35022` |
| Same command after repair | exit 0; A=503, B=200/second-independent; RST then A=200/1007; first shutdown leaves B=200, both drain, registries zero | two-owner-admission-final.log `fc87d7a7e93ca025dcf0ac854172f32c8ae47a2eb85abb320c1d75acbb1bc6ad` |
| `cargo test -p auto-lang-core full_queue_rejects_with_503 -j 4 -- --test-threads=1` | exit 0; original named bridge case 1/1; unrelated integration 0 selected; preserves fast full-queue rejection and Retry-After | admission-full-queue-control.log `3e92df48229e38d0de530fd064479c674f27e4c849c09013e896f3eaa29fccd4` |
| `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --bin isolation --locked -j 4` | exit 0; existing Core 142 warnings, no new warning | admission-sdk-build.log `4272851d3e828b31471dbd21f2a9f23a7a718fd5b466df215a5be0d70a071b04` |

No whole-tier tests, registration/shared-script/lock writes, merge or push.
Final VM port and physical production DAG remain T-02 work. Other mutable
service state is not certified by this request-admission counterexample.


</details>

<a id="e890-ba3ea599a80967c7"></a>

<details>
<summary>work\intake-r2.md · SHA256 da4bcda103d21def5782b96cf22f8b4f36d85294c172ecd22bfdb170fb836722</summary>

#### PLAN-890 r2 — T-01 frozen intake

2026-10-30. Source/consumer intake complete; this is not migration acceptance.

##### Authority, identity and ownership

The user authorized complete 890 delivery, confirmed 885 closeout, and selected
explicit installation of the Web adapter. Core has no Web implementation
dependency. SDK service hosts must migrate explicitly; ordinary language calls
remain usable without Web. Existing CLI/UI service commands install the adapter
at their composition point. UI/Man structural successors remain behind U890.

One language tree: `D:/autostack/.wt/lang-890/auto-lang`, `plan-890-dev`, base
`b3f43a7c4a392a6652b70c9b79db5cedb5589047`, intake implementation begins from
`afea065b203e7c5f842cc874b8e67c72c8130f35`. AutoDown sibling is read-only
`c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`. The earlier identity/license and
U884/U885/U887 intake is in [source-and-window.md](#e890-ab8af97a394bf500).
The 885 historical guard-log gap remains disclosed; the user's closeout
confirmation resolves the start condition without fabricating that log.

Web local target remains absent. No identity, history, remote or publication is
created during this intake. T-03 must recheck that fact before bootstrap. UI,
Man and Blog are observed consumers, not editable business-source worktrees.
Their existing documentation WIP is preserved. Final candidate pins are frozen
later, rather than representing old observations as final versions.

##### Frozen source and obligation maps

`web-migration-inventory` parses real Rust items using syn. Generated Rust in
strings, comments and templates is inert. It preserves nested module/method
identity and attributes, and names macro inputs as whole preserved units
instead of pretending to have expanded them or enumerated runtime tests.

Local-only `intake-map-r2.json` contains 76 source files / 4,120 actual source
items. Logical target assignment is unique: language 2,317; shared language
generation 141; Web 770; pure Web contract 75; language VM adapter 108; Man
consumer glue 172; UI consumer glue 537. Each source item carries original
source SHA256, kind, qualified name, attributes and source ordinal. Package
names and final physical cuts are T-02 outputs; these are source responsibilities,
not assertions that relocation already happened.

The same artifact freezes all six historical manifest inputs (language, tt,
tb, taa, ta, t3): 24,869 tier rows / 4,683 distinct historical identities.
696 tier rows are assigned to Web service control families, 24,173 remain with
language obligations. Tier repetition is retained and is not new coverage.
The manifests' byte hashes are preserved per row. This intake does not count
these rows as executed, compare a refreshed smaller floor, or certify complete
current runtime discovery. T-08 must reconcile them with actual selected/executed
IDs, test sources/atomic assertions and other required governed obligations.

55 macro boundaries are named, including lazy_static/thread_local state and
existing registration/stdlib macros. They inherit a concrete source owner and
must be retained or migrated as a whole until their expansion is explicitly
accounted for. They are not unknown source files or erased obligations. The
tool rejects an unowned source path and duplicate/empty historical manifests.
Generic language macros are retained with their language source.

17 resource inputs (HTTP stdlib declarations, 885 fixtures and affected corpus
assets found in the bounded scope) are separately byte-hashed in
`resource-intake-r2.json`. This is a source input ledger, not the final number
of all inline/aggregated HTTP test cases. Inline test source remains bound to
its Rust source hash; no suite is reduced to these 17 files.

| Artifact, local under main `docs/plans/attachments/890/work/` | SHA256 |
|---|---|
| `intake-map-r2.json` | `09d1d3001f26bc0f7c35fd7954eddb8257e8df527394a0afd692c0563e227a81` |
| `resource-intake-r2.json` | `9b590f0053cd347c4adfe7bbfbc1d2fe00b29f6e0498f6912d2a5a8fdb15aaa5` |

##### Important source cuts and actual consumers

- Web owns HTTP transport, service file/upload executors, observability,
  routing policy, HTTP Rust/TypeScript generation and HTTP templates. The
  Core `api/targets/{axum,typescript}` generators are included, not overlooked
  because the larger `auto-api-gen` file also emits HTTP code.
- Core retains actual language execution, metadata publication, value/resource
  conversion and the minimal VM/extractor bridge. `axum_adapter.rs` implements
  language extern/heap/extractor shapes; it cannot be moved as an arbitrary
  framework copy. Domain policy still leaves Core through narrow ports.
- Pure configuration/validation/policy may be a shared lightweight Web contract.
  Rate limiting, active service state and service runtime remain Web framework
  responsibilities; the whole config file is not blindly classified as a DTO.
- Tauri, shared transpile/type promotion and source/workspace/sidecar rules
  remain a single lower generation owner. Web cannot consume an auto-api-gen
  facade that delegates back to Web. The restricted shared carrier, if needed,
  is frozen in T-02, not a new common package for unrelated code.
- Actual callers include Man `cmd_service`, `http_service`, `target`, runtime
  UI operations and CLI composition; UI `run_vm`, `run_vue`, `rust_ui`, `tauri`
  and `vue`; Core SDK capture/file/script entries; codegen metadata producers;
  ordinary HTTP clients/back-proxy/stdlib callers; generated Rust services.
  Blog is a fixed-source generation/HTTP consumer and affected dual-UI consumer,
  not permission to modify its business source.
- General HTTP client/async/transfer/return-resource and standard-library
  value/facade contracts remain language-owned except explicitly identified
  service couplings. Keeping a shim name/ID does not permit retaining a second
  framework implementation. All split paths have a logical owner in the map.

##### Execution entry for T-02

The first independent source seam is the upload request body. The current
`ApiRequest.raw_upload_body` and `PendingUpload.body` still expose Axum Body.
The existing `a2r_std::http::UploadBodyStream` is an owned Send stream with the
same chunk/error values. Moving the lazy conversion to the network edge must
not poll the body before authorization, collect it eagerly, or change wire
budgets, deadline/permit ownership or terminal cleanup. This is a bounded
source experiment, not approval to declare the entire port feasible.

Next checks: actual JSON baseline and upload accept/reject controls; typed
owner-channel/backpressure/drop behavior; real stream/file/upload handler
terminal paths; proposed explicit host installation and missing-provider
diagnostics; actual shared generation DAG and non-HTTP controls. Installation
API/snapshot/override rules, exact V commands and four-repository schema remain
T-02 responsibilities. No Web repository or mass migration precedes its pass.

The existing lifecycle inventory includes request queued/running/parked,
authorization rejection, slow/oversized body, handler failure, disconnect,
stream/file completion and shutdown/drain. Every later source cut must retain
the original strong controls for these states; a successful JSON exchange is
not a substitute.

##### Serial windows and verification

888's current diff occupies entry/test registration, HTTP outcome-reporting
support, HTTP stream test support, wrappers, Cargo aliases and governance/full
regression scripts. These are not accepted from unlanded WIP. This intake
changes only 890's new tools/summary and its own Plan. The first body experiment
uses `http_transport.rs`, `server_model.rs`, `server_dispatch.rs`, outside the
observed 888/889 changed files; recheck before writing. Shared files, registration,
locks, schema integration and final landing remain serial windows.

Verification: Rust parser sensitivity/control test 1/1; Python intake positive
control plus actual duplicate-manifest and unknown-source rejection 1/1;
inventory runs and diff checks exit 0. No root Cargo test tier or docs generator
was run. Historical baselines and all future final/merge-owned gates remain
pending, with zero execution credit assigned by these maps.


</details>

<a id="e890-17874c445132c12f"></a>

<details>
<summary>work\integration-896-checkpoint.md · SHA256 89058660a70f69d43885b2a162be25e84f7106e3276a53bce1fb5a5cc5775ceb</summary>

#### PLAN-890 r7: integrate delivered PLAN-896 and rerun cargo t

The user explicitly directed integrating896 and replacing final tm execution with the new cargo t. Language890 now contains master c2126b8e9 (896 delivery a9bd4f948) via rebase integration07246f6e5; current committed candidate d171e3632f7d41e2bd965b94af6f3688c0d4c685. Branch plan-890-dev remains in the original D:/autostack/.wt/lang-890/auto-lang; no second tree, implementation agent or default-branch890 code landing. Prior branch eb817ad81 is retained as codex/plan-890-pre-896 for conflict/source audit. Rebase has real semantic resolutions, not an asserted patch-equivalent rewrite.

896 fixed-groups launcher, no global pre-discovery/consumer expansion, ACE workspace/ordinary isolation, default AAVM ignore and owned compiler policy are retained. Shared regression driver/tests chose upstream896; four-owner admission must still be ported to its fixed-group/native receipts before T08 completion. Toolchain paths retain new ACE owners; moved rust_lowering consumes the explicit prepared worker and keeps896 owned compiler execution. No old tm test was restarted.

Old gate63703 had already exited1 before this instruction, leaving230 records/six failed leaves and restored tracked reports; its FAIL remains historical. First new cargo t invocation rejected an external G target before running bodies; integration-896-target-preflight.log preserves it. New commands use CARGO_TARGET_DIR inside890 checkout, five explicit sibling roots and empty RUSTC_WRAPPER.

Actual initial cargo t daily: compilation/discovery succeeded203.616s, measured execution116.875s, whole320.635s;2970 executed,2964PASS/sixFAIL,nextest667 skipped (including filtering; the fixed receipt separately records106 ignored/condition-excluded identities), meter error=null. Five FFI dep parity failures built into inherited outer target then looked for their local oracle binary; the helper now explicitly owns dir/target and requires the expected executable before cache reuse/publication. One HTTP callgraph audit still scanned only the former Core owner; it now scans neutral Core bridge plus actual Web HTTP owner/family, keeping exactly one marked legacy synchronous point and rejecting unmarked calls. Original VM/oracle/golden/count assertions unchanged. Source fixes committed d171e3632; ACE moved-file delta after integration is rustfmt only.

New cargo t targeted replays: ffi_dep_parity_tests5/5PASS (44.664s) and plan705_gate_default_callgraph_no_busy_wait1/1PASS (14.036s), each meter error=null. After all related fixes were committed, one final complete daily-group run verifies the normal parallel composition, rather than treating isolated six-case success as a whole-run certificate.

Final `cargo t --out <integration-896/daily-final>`: **2970PASS /0FAIL**, 106 explicit ignored/condition exclusions remain recorded separately from filtered tests, gate_passed=true. Preparation/discovery 2.145s; execution 75.767s; whole 78.032s; meter.error=null. This is an observed current daily group, not all fixed groups, all experimental tests, all repositories or an A/B performance claim. Receipt SHA256 `50f0e4a6e8e159f37584277bd61eea07dda2af20f22c8f593e55b25a3d1f975f`. Original first red and all raw commands/structured events remain local in separate directories.

Plan r7 is still executing8/12. Known remaining work: adapt890's four-owner/source completeness guards to new fixed-group/native evidence without reactivating retired runtime catalogs, declare and validate genuinely affected groups/owner legs, validate moved ACE controls where actually changed, update canonical proposal and obtain affected independent review, then current SDK/distribution/history and real landings/archive/guards. Prior3 repair cycles and four reviews remain historical, not reset or claimed to certify this new upstream cohort. No remote push, archive or cleanup. No new user decision needed for the directed integration.

Local integration-896/entry.json records conflict choices and active/terminal commands; results-summary.json retains the first red/replays; evidence-index.json binds all actual raw files. Only this concise Markdown checkpoint and shared Plan bookkeeping are staged. Other main-checkout WIP remains untouched.


</details>

<a id="e890-a6385bebc8c0e3d0"></a>

<details>
<summary>work\legacy-host-port-r2.md · SHA256 6b7c937da18f8cb3b9bf0f1c94509dac63658ff8cda68412782b302fe13a8b6b</summary>

#### PLAN-890 r2 — explicit legacy stdnet grade and owned cleanup

Bounded T-02 progress from parent `4a266bb30`. Still executing 1/12; no Web
repository, production framework exit, final review or final delivery claim.

##### Explicit grade, one source and lifetime repair

The borrowed host port now receives `BlockingListenerKind::{ServerHandle,
LegacyStdnet}`. These are distinct existing grades, not interchangeable aliases.
`listen_registered` selects ServerHandle; `listen_legacy_registered` selects
LegacyStdnet. Old public SDK `http_server::serve_blocking_stdnet` is a thin
explicit-host wrapper with its original **unit-return / diagnostic** signature.
Missing or async-only providers fail before binding. No implicit Core fallback.
Successful borrowed completion uses the per-VM marker already introduced.

The old legacy loop moved once from server_dispatch into server_legacy. The
external source fixture compiles that actual implementation and is reached via
the real public compatibility SDK wrapper, not a hand-made VM response. Core
still compiles the framework until T-05; an external source fixture is not the
final independent Web package. No VM/private module field visibility was widened.

Three real cleanup defects were addressed at the owner boundary: fresh task
Arcs clone out of DashMap before locking/removing; try_lock supports the existing
Tokio owner thread; request/temp-task/iterator guards also run on SSE/file early
continues. Native resources register in a request-local Send context (no VM in
that Arc). Its terminal mark and ID drain share a lock, so late constructors
immediately finalize. Existing named primitive cleanup is reused through one
`VmHttpBridge::finalize_resources(kind, ids)` dispatcher; the old RequestScope
late-resource branch now delegates to that same dispatcher. No new deadline
promise is added to this synchronous legacy grade. Stream write/flush failure
ends pulling and releases the iterator/task instead of continuing indefinitely.

The old int-table-hit SSE guess contradicted the **existing** return-resource
Spec §2 (declaration/table hit/value is not resource identity). Streaming now
requires the real owned Sse credential. A normal integer, even when equal to a
live iterator key, stays JSON. Real generators remain streaming even through an
old int-declared handler; this repairs the legacy loop to the current producer
protocol rather than dropping source compatibility. Popped credential slots
are cleared and their ownership is consumed once through the existing StakeGuard.
File-return routes remain explicitly unsupported 500 with the original reason;
they are not silently upgraded to the default file transport.

##### Default-callgraph finding and unchanged original gate

The original named Plan705 source gate first failed: round16 had placed the
separate server-handle loop (originally in stdlib/http_client, outside the default
HTTP family) into the scanned default-family directory. This was our introduced
layout regression, not a historical red and not deferred. The separate source
now lives at ffi/http_server_handle, retaining its existing module alias and
whole LF-byte-identical body (SHA256
`3184a2af5cccc2ef4ffbd359dd7adfd7d1dce4f5189ca9a084e2008081f472c8`).
Only the external explicit-grade fixture calls serve_server_handle; the Core
production tree has its definition, no default-path call. Default async assembly
was not changed to call it. The old legacy stdnet loop stays in the scanned
HTTP family with its original marked synchronous call. Original gate source,
assertions and required count remain unchanged: default unmarked busy calls 0,
marked legacy call exactly 1. The same original test now passes. T-08 must move
that full obligation with the final Web owner, not narrow the ledger/floor.

##### Actual verification and explicit limits

- Locked external SDK build exit 0; root `cargo check -p auto-lang -j4` exit 0.
  Core 141 / root 3 / external source 17 existing warnings remain; root's named
  lib-test build reports 42 existing test warnings, not a clean-global claim.
- `legacy_probe.py --mode installed` exit 0 with child autovm explicitly selected:
  exact per-run JSON, 400 missing/invalid argument, 404 unknown route, wildcard
  CORS, header-only OPTIONS 204 without reading its promised 1 MB body. The
  unchanged original counter generator emits exactly 1/2/3 frames; an int-declared
  endpoint returning that real generator emits the same frames. Three repeated
  file requests give exact explicit 500. An actual infinite generator is observed
  yielding, then its owned peer is reset; the serial owner serves the exact
  recovery value afterwards. Its controller stops and joins normally.
- Collision control uses a **real VM SDK-produced lazy generator**, obtains its
  credential and actual first iterator key 0, and retains it outside the request.
  HTTP integer 0 must be exact JSON bytes `0` / application/json, never SSE. After
  all requests the caller iterator is still unstarted and taskless. It is then
  explicitly released by its caller, not consumed by the server.
- Before VM destruction: handler task delta 0, service iterator delta 0, descriptor
  table 0, retained request credential owners 0. The one unrelated caller seed
  remains until its explicit release. Actual rc_count plus owned credential type
  inspection distinguishes live ownership from the bounded zero-RC grace queue;
  after caller release retained ownership is 0 before VM drop. After owner return:
  actual live credentials and upload tables 0. A process exit alone is not the oracle.
- `missing` / `unsupported`: wrapper exit 0, old compatibility child exit 0 with
  its preserved unit return, **required diagnostic/no listener**. These are
  negative compatibility controls, not a healthy service result.
- `grade-swap`: fixture deliberately selects ServerHandle for a LegacyStdnet
  request; real exact JSON readiness occurs but the original CORS expectation
  rejects it (expected oracle exit 1). Failure cleanup asks its owned controller
  to stop; forced termination can never count green. No production mutation flag.
- Server-handle language replay stays passing. Original named unit cases:
  host_context_restores_parent_and_reclaims_late_native_uploads,
  explicit_http_host_selection_is_stable_for_active_local_owners,
  image_media_route_has_priority_without_changing_user_routes: each 1 pass,
  zero ignored. Original Plan705 callgraph gate: red-before / 1 pass-after.
  Its final replay includes the credential-only source. Root check includes the
  final runtime; later edits only strengthen fixture assertions/documentation.
- New/changed isolated sources pass rustfmt, diff check and Python compilation.
  No whole tier, docs_gen, GUI, push or shared registration/runner/lock change.

Raw local-only evidence below; only Markdown staged. Inputs/source/builds stay
in the Plan tree. No unrelated process/cache was used as cleanup.

| Local evidence | SHA256 |
|---|---|
| `legacy-port-build.log` | `51781fa99336cd32bf95afb04ab0a09fdf2c9255a56c0a46bcbba888e083c747` |
| `legacy-root-check.log` | `9ddd7ea76a057bdffdc135cfd58f375b0f96304521876e3772baa3a4c9816e20` |
| `legacy-installed.log` | `33ca6d85e38a169df8794092b94a1500c8913c4b8e377d4db94e7dcab0e92648` |
| `legacy-missing.log` | `fd4b9903da7c2c1333e97d9c66e2d21c6025332cc71d54745ab7096d1c4aefc4` |
| `legacy-unsupported.log` | `b95ec40c58006f3221191068e2c97cb3b00eb8b590f910070544fb7d1a18b38e` |
| `legacy-grade-swap-negative.log` | `1a8cde9b1f272fbc769d9b59a6b2ed2fc3bb5a9bf127afee17538828593fee69` |
| `legacy-server-handle-replay.log` | `8ab6e86a991580b397fcb06c545b680b9737550e2fa8ca468542ecafa30bd6db` |
| `legacy-callgraph-before.log` | `97145e8bd2e8aef7170880f0c9b49e6ae7752931def3f3137ecda9c4150dbbb8` |
| `legacy-callgraph-after.log` | `69f85cfa3c0d9e08ab0bcead7cf6167339d84052e8f39faef4fa76cfa8340536` |
| `legacy-callgraph-scope-map.json` | `7215aaa49803873712bfc4b66d3077f76a84d286473c9d867a85070d9ede386e` |
| `legacy-host_context_restores_parent_and_reclaims_late_native_uploads.log` | `7b11c60065336ac6838c0999811f4d178aaa4715ecbe93fe7fd81792bf3bf78b` |
| `legacy-explicit_http_host_selection_is_stable_for_active_local_owners.log` | `e6262fab179bdce2e009abd1c3978780b252c2776a988a8bbf7c9bf47bed3c75` |
| `legacy-image_media_route_has_priority_without_changing_user_routes.log` | `9208a4b8fa9a0b182b635cb871d778c8c7c5414fd3805a93ce2c7469aed526a4` |

T-02 still needs configured-launch/legacy program-policy and producer ownership,
full default-runtime deadline/control terminal cases and all actual consumer /
physical package / API / command / SD / four-repository schema freeze. This is
not permission to create Web before that full pass. All real framework originals
and these temporary Core exports must exit to Web at T-05, preserving the two
explicit legacy grades and current source/resource identity contract. The
full T-08 original-ID/atomic obligations, independent final review, frozen gates,
all-repository land/spec/archive/guard remain pending. UI/Man successors still
wait for full U890; their trees were not edited in this round.


</details>

<a id="e890-87d5a3dc4d1a4214"></a>

<details>
<summary>work\native-policy-owner-r2.md · SHA256 bbfe9577810f37d7a18ce5a9002ef0caf59b322316390ad0febd7c14559dce0e</summary>

#### PLAN-890 r2 — native policy handoff and owner rate buckets

Round 21, work/T-02, parent `79a6a411a211fb905a30bf58867ad1793cd8c3d6`.
This closes the observed native rate-limit handoff gap; it is not the full
T-02 port/freeze gate, final review, or Web repository delivery.

##### Cause and change

The real `http.rate_limit` native wrote Core framework-global configuration;
the external source crate owned another copy and served without that setting.
A pre-fix actual TCP probe required four successful requests then 429 and
instead received a fifth exact nonce response with 200 (exit 1). Its initial
header-name capitalization error was corrected before recording this red.

The native now records a private per-VM pure setting. Unset, explicitly disabled,
and a positive quota remain distinct. `VmHttpBridge::rate_limit` reads the named
setting on the existing VM thread; no private VM field becomes public and no
VM crosses a network thread. The old embedding `set_rate_limit`/clear facade
keeps one neutral process-default source through the bridge, rather than a
second framework-global copy. An explicit language setting, including zero,
overrides that default for its VM. Normalizing negative inputs and a zero
window follows the existing native behavior.

Rate-window execution and buckets remain framework-owned. Each actual serving
lifetime now owns its own buckets, selected only while polling its local
future, with nested/pending/unwind restoration. The existing fixed-window
algorithm is shared with the standalone dispatch compatibility path. That
standalone path retains its legacy shared buckets; actual owned services do
not use them. No route/registration/runner/lock changes are involved.

Old VM CORS comes from `AUTO_CORS_ORIGIN`, not a language setter. Its exact
environment value crosses the external service boundary unchanged. Configured
service CORS continues to use its own existing instance configuration.

##### Scoped checks

All commands run in `D:/autostack/.wt/lang-890/auto-lang`.
Raw logs below are local-only in
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Exit / distinguishing result | Log SHA256 |
|---|---|---|
| pre-fix `policy_probe.py --exe .../isolation.exe` | 1, fifth exact nonce request was 200 instead of 429 | `4c126e40c82b29cff52657f6c697500b19302a784bb021f12e114beb607c2485` (`native-policy-before.log`) |
| `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin isolation --bin web-port-probe --locked -j 4` | 0, actual external source runtime | `a9beaf648d0f4f7a93be72180a647a96facd6853800a97e2331d9330b0716270` (`native-policy-build.log`) |
| final `policy_probe.py --exe .../isolation.exe --plain-exe .../web-port-probe.exe` | 0; real SDK default 1, VM A native quota 4, VM B native disabled then handler-enabled quota 2; A remains exhausted while B serves, B changes do not reset A; exact nonce/CORS/429 payload/Retry-After/request-id; both owners drain and registries empty | `a955f1c299c819f54a2f56d68d59993112c40c90e79694e0a0c0d177705b5e28` (`native-policy-after.log`) |
| config-only program inside same final probe | 0, `http.rate_limit` then ordinary expression yields 42 without installing any adapter or listener | same final probe log |
| `cargo test -p auto-lang-core rate_settings_preserve_sdk_default_and_per_vm_explicit_disable --lib --locked -j 4 -- --exact vm::host_capabilities::http_service::bridge::tests::rate_settings_preserve_sdk_default_and_per_vm_explicit_disable --test-threads=1` | 0, 1 pass / 0 ignored; public SDK default and clear, positive and zero VM overrides, minimum window, fresh VM | `494a57068c23bdf0ebdb9f9101936c7f31e6b9f41c95df2c0d499b7d56bf979e` (`native-policy-settings-test.log`) |
| `cargo check -p auto-lang --locked -j 4` | 0 | `7260803512aeba5c492fd7a39e38869cfa0660f2175d2fb1ace71079b362c045` (`native-policy-root-check.log`) |
| `service_state_probe.py --exe .../isolation.exe` | 0, configured identities/bound config/counters/CORS remain independent; headers-only own shutdown leaves second ready | `aaf6ac2f52e64c37fd14cbd9ea1e6a2bc4b705b51e548c7cea3680b25b059a13` (`native-policy-service-state.log`) |

Final policy script SHA256:
`8c0662f8dab0d1e4794f49bb77f6544da939f08590c94c156c8c370b3a8fbeae`.
Generated source/scratch and build outputs stay under the fixture's worktree
target. `py_compile`, touched helper rustfmt checks, and `git diff --check`
pass. A final native-call line wrapping edit is formatting only; it does not
invalidate the compiled native behavior above.

Core normal build has 140 existing warnings (previous 141, the previously
unused native VM argument is now used), root 3, external runtime 17; the Core
lib-test build reports 160 warnings. This is not a global warning-clean claim.
No whole test tier, docs_gen, GUI, push, or shared script/registration/lock
execution occurred. Original HTTP e2e IDs are still pending T-08 harness
migration; this new probe does not claim their execution or shrink their floor.

##### Remaining gate

T-02 remains open for producer metadata ownership before activation/concurrent
compilation, configured launch and complete runtime control/deadline/thread
termination, actual consumers and the final physical packages/APIs/commands/
Spec delta/four-repository receipt schema freeze. Framework implementation
still exists in Core as source-probe scaffolding and must exit in T-05.
The standalone compatibility buckets and process-default facade must be
included in the owner/consumer inventory and eventual single-source relocation.
Web has not been created; independent final review and merge-owned gates are
pending. UI and Man structural successors continue waiting for full U890.


</details>

<a id="e890-5621da983b0d3f3f"></a>

<details>
<summary>work\network-terminal-r2.md · SHA256 a80026849d499bd287a1ac6ffeedae12d4b8fb638a3246c312ea4da32681f184</summary>

#### PLAN-890 r2 — network join and drain termination

Round 22 / work / T-02. Parent `e32f4d6786d850c7595df354ff18d7ecce92cd06`.
The preceding round made implementation progress, not a wait. This round
closes specific network-thread/connection cleanup gaps; full T-02 remains open.

The actual owner discarded its network JoinHandle. Its drain deadline was
also recomputed after every event. The owner now retains and asynchronously
joins that exact owned thread (including failed startup), drops its queue
receiver, and marks Stopped after join. Only the Send thread handle enters
spawn_blocking; the local VM stays on its original thread. Its drain deadline
is set once, checked before further parked work, and reused across events.
Network and owner wait on the actual true watch value, honoring an already
true value and ignoring false updates; sender closure still stops service.

Connection cleanup now uses a captured RAII guard, created before task spawn,
so canceled/unpolled connection futures also abort their reset shadow, cancel
their own scopes and decrement the same instance's active count. The permit
and actual resource body lifetimes remain tied to their existing owners.

Two source-bound counterexamples distinguish these repairs:

- Restore only the original owner file from parent e32f4d678, keep the current
  fixture and actual transport: installed probe exits 1 with a real network
  thread still alive (`network_tails=1`) when the old owner returns. The
  fixture wrapper executes the original transport in full, then keeps that
  same real thread alive 800 ms. This is a fixture-only post-transport delay,
  not a production setting or replacement network implementation.
- Restore only the original transport file while retaining the new owner join:
  the partial-head idle-connection probe exits 1, with the instance's actual
  `connections_active=1` after its serving future returns. Secondary Python
  connection-refused output follows the child's counter assertion/panic; it
  is not the distinguishing failure.

Both temporary restores run in this same worktree, save candidate bytes, and
restore those exact bytes in finally. Candidate owner SHA256
`b00acedbba787fbed4d790c127e39abbb5bab915460a452d11b5104fbb549982`;
candidate transport SHA256
`26a93adde489f502205c38346a2731e668b9787ae5fc257b13bad4cbbd48b1c3`.
The final build is after both restores. Probe/source scratch now explicitly
uses the worktree fixture target. Earlier runs with system-temp source were
excluded; both reds and the final four positives were replayed with the
correct scratch location. Raw logs are local-only under
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Command/check in the Plan worktree | Exit/result | Log and SHA256 |
|---|---|---|
| parent-owner installed probe with `PLAN890_NETWORK_TAIL_MS=800` | expected 1, live actual network tail 1 | `network-tail-before.log` `5e867c9883614e8ce2387a1a847e6705037b0199ec72c6b9ae5511e016225788` |
| parent-transport `service_state_probe.py --idle-on-shutdown` | expected 1, actual instance active counter 1 | `network-idle-before.log` `16d67df72723af53ed81fb4443e144e2df8d3ce9d0296e8a11217e7a6a15d9d8` |
| `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin web-port-probe --bin isolation --locked -j 4` | 0 | `network-terminal-build.log` `064845bd11eee2ea49126f5b5bd66e3139ca48b0b866110d5c0748deed530517` |
| `probe.py --exe .../web-port-probe.exe --mode installed`, tail=800 | 0, exact nonce, thread tail completes before owner terminal, all registries 0 | `network-tail-after.log` `a6f1a6987a79db81e717567dcbc332d373c541fb5f1544a17bdd9ab47a284cd4` |
| same `--mode pre-stopped`, tail=800 | 0, initial true watch returns, actual thread joined and port rebinds | `network-prestopped.log` `7ad60f0943f5de006342be6212e8e0653bbd94119dc6288b14c2d45552f8b926` |
| `service_state_probe.py --exe .../isolation.exe --idle-on-shutdown`, tail=800 | 0, force-drained partial-head connections, per-instance active count 0 after return, B independent, both ports rebind | `network-idle-state.log` `cb250c0c5531260708de96d4dc6477c1fe6c02e6f7aa73a1c40e2c0d40d6bcbd` |
| same `--file-during-drain`, tail=800 | 0, complete exact 16 MiB file across shutdown, ready no longer 200, B independent, counters 0 and both ports rebind | `network-file-drain.log` `307d0c757d5bb6624d399f420842278f4a6fd0f31c5ae4fbaea13b58739c9339` |
| `cargo test -p auto-lang-core shutdown_wait_honors_initial_true_false_updates_and_sender_close --lib --locked -j 4 -- --exact vm::ffi::http_server::server_dispatch::shutdown_tests::shutdown_wait_honors_initial_true_false_updates_and_sender_close --test-threads=1` | 0, 1 pass / 0 ignored | `network-terminal-watch-test.log` `4a17e10823fb7b3757dd1f6a88e687b0950ce803d703db661497d2d8cbbdfddb` |
| `cargo check -p auto-lang --locked -j 4` | 0 | `network-terminal-root-check.log` `7638280a3b2535392ec3e211440b81b3a30f133cfe22e7d2bac502b768a4af54` |

Touched helper rustfmt, Python compilation and diff whitespace checks pass.
Core normal warnings140 / external17 / root3 remain baseline, not global health
clearance. No whole tier, docs_gen, GUI, shared registration/runner/lock writes,
agent, cross-thread message, remote operation or final merge window.
888 head readback 643f2b375; its transport/dispatch files have no diff against
the shared base. UI 7b323bc and Man e6cc06f stay clean and unchanged.

Remaining T-02: producer ownership before activation/concurrent compilation,
configured launch and per-instance failure propagation/logger termination,
complete response terminal observation, actual consumers and final physical
package/API/command/SD/four-repository schema freeze. The new file-drain log
shows a file request event with bytes_sent=0 despite complete wire delivery;
this is an unresolved terminal-observation finding to investigate, not a
bytes-accounting pass. No claim of original HTTP e2e ID execution or floor
reduction. T-05 must still remove original Core framework implementations.
Web is not created and the Plan remains executing 1/12; final independent
review, merge-owned gates, canonical deposit and guarded cleanup are pending.


</details>

<a id="e890-6bb22aeec7e5419d"></a>

<details>
<summary>work\neutral-generated-service-r2.md · SHA256 083c06bd7da6a88c850d211b31d71a3588cae72a4b3a0d1e37a48a396cded4bf</summary>

#### PLAN-890 r2 — generated Rust service without a UI owner dependency

Bounded T-02 source/closure progress. Language parent `7c5a98804`; UI parent
`d53f946ab89ce6bb3cbbb2ea929f094ff2600be6`. Still executing 1/12; no Web
repository, final review, frozen merge gates or complete delivery.

##### One neutral photo owner, compatible UI path

Moved `auto-ui-runtime/src/ui/photo_service.rs` into existing language-side
`auto-media-core/src/photo_service.rs`, under its existing `image-codec` feature.
The **whole file**, including ten original test bodies, is LF-byte-identical to
UI parent blob `ebeadbb360dee925c6fd137fc0a6c21fca0c707c`, SHA256
`738751c6bfa88c9ea5c313d620d008528f7e64a9e4ab28fa78ed0bdcef12558f`.
No photo algorithm, registry, cache, decode/EXIF limit, root/token policy or
thumbnail behavior was copied or rewritten. Both repositories retain MIT
license/source lineage. This is a move, not net source deletion.

The existing UI feature gate stays at `ui && image-pipeline`; the old public
module now only re-exports `auto_media_core::photo_service::*`. Those public
paths now name the neutral owner's same types/functions, with no second UI
photo registry. UI's codec feature already forwards `auto-media-core/image-codec`.
No UI architecture successor, renderer/HMR redesign or new feature is launched.
UI same-lifetime tree is now on `codex/plan-890-dev`; only this facade is changed.
Its exact final commit is recorded in the central Plan after both source commits.

Original test names preserved at the neutral owner (10/10 actual passes):
`root_listing_is_non_recursive_with_dirs`, `subdir_listing_navigates_one_level`,
`sanitize_rel_dir_rejects_traversal`, `tokens_resolve_per_root_and_never_leak_paths`,
`photo_from_rel_rebuilds_entry`, `content_type_map`, `date_label_converts_epoch_days`,
`thumb_cache_key_includes_mtime_and_width`, `render_thumbnail_roundtrip_and_cache`,
`missing_root_is_an_error_not_a_panic`. Old package/module is auto-ui-runtime /
ui::photo_service::tests; new is auto-media-core / photo_service::tests, selected
by image-codec (default). This additive source mapping supplements the immutable
intake; it is not a smaller replacement floor or a complete T-08 ledger receipt.
The old neutral source probe now consumes the public media package, not UI
source by path; its no-local-test library is documented as check-only, not a
zero-selection passing test channel.

##### Production generator and service workspace

Actual generated photo handlers call `auto_media_core::photo_service`; generated
Cargo no longer selects auto-ui-runtime. Existing canonical-owner/media-route
assertions retain their original IDs and endpoint obligations and now require
the neutral path; the canonical-owner test additionally forbids UI dependencies
and UI import paths. Stateful/full-cover emission and GET/HEAD/media/photo
routes are retained. This is not a hand-edit of generated consumers.

All four production service workspace calls (initial layout, final membership,
lock receipt and reuse freshness) use the already-established Service mode.
Existing members requiring UI still select UI mode through the same manifest
scan; target-specific renamed UI dependencies cannot be discarded. Unmanaged
service source validation follows **every selected Core/a2r/media dependency**,
without recursively validating the unused facade's optional renderer paths.
UI callers still validate the full original facade/UI source closure. Managed
SDK receipts remain strictly checked; no fallback or weakening was added there.
A strengthened child-process control proves a service can use good selected
packages while the unused facade has a missing optional UI path, then proves
that a missing optional dependency of **selected Core** fails before rewriting
the previous workspace. Missing explicit UI and existing renamed UI members
also fail without changing the old manifest.

##### Actual commands and outcomes

- `cargo test -p auto-media-core photo_service::tests -j 4 -- --test-threads=1`:
  10 passed, 0 ignored, private child TEMP/TMP under the local evidence directory
  so a real decode/cache check does not borrow the global thumbnail cache.
- `cargo check -p auto-media-core --no-default-features -j 4`: exit 0; codec-off
  media file service remains available without the photo module/codecs.
- External neutral package consumer check: exit 0. Local ignored fixture lock
  updated offline after replacing path-included UI source with the real package;
  this is not a final SDK distribution lock.
- `cargo check -p auto-ui-runtime --locked --target-dir ../auto-lang/target -j 4`
  from its dedicated UI tree: exit 0, default UI pipeline and the compatibility
  path compile. UI 215 existing warnings remain, not a global clean claim.
- `cargo check -p auto-lang -j 4`: exit 0, root 3 existing warnings; API generator
  check also exit 0, 3 existing warnings. Core retains 141 baseline warnings.
- Original targeted generation/workspace tests: 4 workspace ownership/source
  cases + 1 canonical owner + 1 stateful/full-cover routes + 1 image runtime
  shape all pass, zero ignored. Strengthened isolated service facade/selected-Core
  rejection control passes 1/1. No whole-tier tests or docs_gen.
- New external `web-generated-service-probe` is the actual production generator
  consumer. Its locked build passes. `probe.py --exe .../web-generated-service-probe.exe --target test/fixtures/modularity/web-port-probe/target --evidence test/fixtures/modularity/web-generated-service-probe/target/runs`
  passes. All generated project/source/build working directories are inside the
  Plan worktree; the script now rejects an evidence/scratch directory outside it.
  It uses a new source/project and an explicit reusable target, not final T-09's
  clean-target/relocation/source-distribution certification.

Accepted run directory: `D:\autostack\.wt\lang-890\auto-lang\test\fixtures\modularity\web-generated-service-probe\target\runs\generated-neutral-4hhq92op`.
The actual generated binary was compiled unchanged, then built locked; complete
Cargo metadata has **483 packages**, zero UI/Man/Tauri packages and no unused
auto-lang facade in the selected graph. `AUTO_UI_SOURCE_ROOT` points to an absent
path. It becomes ready with the exact generation.json assembly fingerprint,
returns the exact per-run business string, scans a real 3x2 PNG, streams its
byte-identical original, decodes an actual JPEG thumbnail with 3x2 frame size,
rejects traversal (400) and an unknown token (404), and returns 202/draining=true
from its own shutdown endpoint. The child exits 0 and its stdout contains the
real AUTO_SERVICE_STOPPED receipt. Source mutation then gives stale/nonzero;
restoring exact API bytes restores freshness. Photo SHA256:
`6da272b1a92ec2637f6e33b58c3efd6fe963d2589e7f913f5d942d1842caf348`.

Earlier fixture runs are not counted: first read the shutdown receipt from
stderr rather than production stdout; another placed temporary generated source
under the main checkout's evidence directory, violating the worktree execution
rule. The final accepted replay fixes both, completes all assertions from a
new worktree-local project and leaves only copied raw logs/receipts in main
attachments. Existing generated-code unused imports/variables remain (4 in
this minimal contract), not asserted as globally warning-clean. No forced kill
counts as a successful stop. General slow-body/drain deadline and logger-thread
termination are still independent lifecycle obligations.

##### Evidence (raw local-only, only Markdown staged)

Files below are local under main `docs/plans/attachments/890/work`; accepted
run raw logs/JSON were copied after the worktree-local process ended. Generated
source and build were not moved to or rerun from the main checkout.

| Local evidence | SHA256 |
|---|---|
| `photo-neutral-owner-tests.log` | `c395d9d1a7537a91649702012fbdb1bd0f42254c55b301a8c65b72f4c93f9157` |
| `photo-ui-compat-check.log` | `e80a6f4f9c76390e149e605cd919e0260091c379e442f43737cffaf366418784` |
| `neutral-generation-root-check.log` | `455858d6edc6c1e4beab62835dc08b123c26b20fc09d80fc5b39b4d1e91e135c` |
| `neutral-generation-check.log` | `a043ff1a54449a80c6eb78c18401baba711400dbe1f71947898654703832daf9` |
| `photo-codec-off-check.log` | `fef245648b4c0304d383a9e138f76315e827d49953428dea71eae8a643631ce3` |
| `photo-neutral-consumer-check.log` | `46bd4b0295d53dd08915c0f181292a9efae2d43a853ba84ecbb7ce017c534d9c` |
| `service-facade-closure-control.log` | `1702bcf9bc2f4580e61a4f7c4574b733dba406d01db32455b47c36949ff84bb9` |
| `neutral-generation-rust_workspace-plan842_tests.log` | `d9f3017552e95e7fd7d3b76c9037a94c9a51d8e5314c1a8757dd22d4ce50c13a` |
| `neutral-generation-plan842_generated_backend_uses_canonical_owners.log` | `4316d3fa771796efe456cbb6adf0586debab9718f94a646035d2528936d125dd` |
| `neutral-generation-media_route_is_emitted_for_stateful_and_full_cover_servers.log` | `590188eac3e3973920fd60b2f3beec02d5bfe8259f7fe068cb75235c0ed2e95d` |
| `neutral-generation-image_stdlib_codegen_uses_shared_runtime_feature.log` | `5428e44d317028039b5629435e35b8d73cbc406fdfb131650c7495e47006368f` |
| `neutral-producer-build.log` | `087f1c843d94cb58981a7fc7ce56d80da6013a1e9ce6f5aff4ae388087392ad7` |
| `generated-neutral-oracle.log` | `41b0cf6a7172ae45ba47881512c289538b9af42ba2ca820b84472e07571e8427` |
| `photo-neutral-source-map.json` | `388af0615cf5d66ebd0c457e4d144523dca329c639fba087c65f74a02a0dafc9` |
| `neutral-produced-generation.log` | `19b00117020401e5c60a7176c46ce39fa654c5bf2b86453629b29e5c7dfc255a` |
| `neutral-produced-build.log` | `3f0395d004e8980cfc7cce18abaae34edd6d0592ee9ca6bb46b6f6a5904f0ef4` |
| `neutral-produced-locked-build.log` | `b338704c651876428c9fbae2da619d3d7643cc876f916d1e3cfb30057a7bb4bd` |
| `neutral-produced-metadata.json` | `31cb6a0359c81b746fc4b69f6e67808d2e28f18a7bb52f073c2a30832f231175` |
| `neutral-produced-fresh.log` | `6ea1013c12f4fb4a77e48fefa4d018d3771cfac214f387d97fc85ed65a5bbf2e` |
| `neutral-produced-stdout.log` | `8890d137f08a6b8456a40a0975929e721d66720e6aa98bc713e02d4a7b60dfb7` |
| `neutral-produced-stderr.log` | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `neutral-produced-stale.log` | `2287f41e6f98d31fbdb72247a9f86a2cba5d6e078f06ecfbede677dfb6da8a76` |
| `neutral-produced-restored-fresh.log` | `6ea1013c12f4fb4a77e48fefa4d018d3771cfac214f387d97fc85ed65a5bbf2e` |
| `neutral-produced-result.json` | `843087a9ad9f514b11c4d63f35962d21f12871e35b842ed37b185df48b192782` |

##### Remaining scope and specification proposal

SD-04 must also update the existing `docs/specs/auto-media-core/project.md` and
module plans index on merge: neutral photo source/API, codec gate, old UI facade,
original test mapping and production HTTP use now belong with that foundation.
This adds an actual touched component to the proposal, not a prematurely edited
canonical rule or a new Plan/goal. Existing 9 SD entries retain their IDs.

T-02 remains open: actual SDK language spelling/direct-return coordination,
legacy listeners/control/producer/rate/deadline/owned terminal paths and the
complete physical Web/shared-package API/command/SD/four-repository schema freeze.
This successful generated-service closure is not T-03 bootstrap permission on
its own. T-04/06/07 still must relocate framework/generation once and wire all
real Man/UI/SDK consumers; T-08 must reconcile the full original ID/atomic
ledger plus this photo supplement; T-09/10 must prove distribution/recovery,
Blog and affected dual UI. Independent review/frozen gates/merge/spec/archive/
guard remain pending. The parent-reproduced Tauri formatting fixture red from
the previous round remains visible; not rebaselined or counted green here.
No shared runner/test registration/lock/final integration window was taken.
UI/Man structural successors still wait for complete U890.


</details>

<a id="e890-a4b16f20afcf5a21"></a>

<details>
<summary>work\owned-messages-and-resource-repair-r2.md · SHA256 085662c97025bb46b797fa688aa021d838b4a83c4da4b47f55e216e8ac3b35d2</summary>

#### PLAN-890 r2 — owned messages and upload resource closure

T-02 partial; 1/12 remains executing. Previous commit
`78ada233b403e3b92a2f60025d9c1f8dbacb352c`. No final port/package/instance-isolation
claim and no mass migration permitted yet.

##### Implemented boundary and found invariant

The original ApiRequest/ApiBody/ApiReply/FileReplySeed definitions now have one
source in `vm/host_capabilities/http_service/messages.rs`. The old server-model
path re-exports the exact types. They contain owned bytes, bounded channels,
version-independent a2r upload streams/file descriptors, and no VM/Axum object.
The header accessor is public on the owned message. No VM private field surface
was expanded. Standalone compilation imports this actual source without Core,
VM, Axum, UI or Man; it is not a completed production contract package.

Stronger terminal checks found a real leak: the existing upload program's
headers-only `/api/early` response was 401 and staging was empty, but after
owner shutdown `vm_upload_counts()` was `(1,0,0)`. Injection happened before
the handler-segment scope guard, so insert_upload_request registered no owner.
The repair binds the request scope around capability insertion. It covers
early returns and argument binding failures, not only consumed uploads.
The counterexample now passes with all five real registries zero. A legitimate
path-param int binding failure after injection also returns 400 and reclaims
its capability; its handler would return 500 if incorrectly executed.

Two initial formulations of that extra negative input were rejected at compile
time (body-bound int forbidden on upload routes; UploadRequest must return
UploadReceipt). They supplied no runtime credit. The final input uses a path
int and `~UploadReceipt`, retaining compile-time upload contract validation.

##### Evidence

All logs local-only at
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Result | Log and SHA256 |
|---|---|---|
| Actual message source standalone named test | exit 0; 1 passed; real thread transfer, binary/source error/body drop/SSE bounded queue+EOF/file identity | owned-messages-probe-final.log `7f73e099955b04aafafb26f588757df58632b5c75ac7fa18355102484ca271b3` |
| Reply TCP replay | exit 0; exact sync/async 256KiB binary file; HEAD/206 range/304 etag/404/500; ordered SSE 1,2,3 EOF; all registries zero after return | owned-reply-tcp-probe-final.log `9cdd230430e6d50d067fb40f547da819a8a904faf7e26cb49bf55879cb5b4b3a` |
| Pre-repair original upload replay with stronger terminal assertion | expected exit 1; 401 correct but upload requests 1, other registries 0 | owned-messages-upload-replay.log `f9afdda694fac9c19dcf7a91ab92f10c62f6d8b33e98fccb4e7626ad11e3ec23` |
| Repaired upload replay + binding sibling | exit 0; original 12MiB exact download, 409 preserves destination, 422/staging empty, headers-only 401, binding 400; scopes/files/upload tuple/streams/live ops all zero; drain exit 0 | owned-upload-resource-repair-final.log `7900648d8a6565a451c2e2ad51a62ab1ca8a4130d02813eaecff32ad36332f80` |
| SDK fixture build after repair (`--locked -j 4`) | exit 0; Core existing 142 warnings unchanged | owned-messages-sdk-build-repair.log `7b321ba6fccb23fd1e066937fa5dc162f43bf8d395175f3af60b1397bdceffcf` |
| Missing host | controller exit 0; child exit 1, explicit adapter diagnostic, no listener start | owned-messages-missing-replay.log `471911e6054b9caaa9f2b2bfd7ae67c99e90126f0c46fbe74daeabf8e3466f4f` |
| Plain language | exit 0; value 42, no adapter/listener, registries zero | owned-messages-plain-replay.log `28f423fb79989a13a25941e6de5f5f69445b8d90861c85c8e4657723b4d13232` |
| Message fixture tree (`--locked --prefix none`) | exit 0; no Core/UI/Man/Axum/Tauri | owned-messages-tree.log `0885784b4b77973b984e5bb54d8183c166a90714e819653a3515b70c87a4f510` |

Reply replay consumes the original PLAN-729 program (SHA256
`2a4391ff331061602bd470f6b021d1d83a44ae6304f21b6071e4ce3361b23be8`)
and original Core SSE generator (root source SHA256
`b0a4de8f2ed6dc14b9852243f2934f98c286581f0094b0a40f6e35dbe70db45f`).
Upload source remains the previously frozen PLAN-730 SHA256
`1b4d78ac32af8528c76babff0b9f7e36edc1542b525d615caddd457992cebe38`.
Only temp-root substitutions, identity marker, and the named binding negative
are added. No original full-suite execution credit or real simultaneous-instance
isolation claimed. Reply evidence remains valid after the upload-only insertion
guard change: its program declares no upload parameter and never enters that
branch. No whole-tier gates, shared scripts/registration/locks, GUI, merge or push.


</details>

<a id="e890-607654585f44cd1c"></a>

<details>
<summary>work\owned-upload-seam-r2.md · SHA256 3031f61fa6df5e0ebad2877edbbc4b3167e302d7a056361d7b86b8999cb22ceb</summary>

#### PLAN-890 r2 — owned upload seam experiment

2026-10-30. T-02 partial evidence, not a complete feasibility pass.

##### Changed boundary

`ApiRequest.raw_upload_body` and `PendingUpload.body` now carry the existing
`a2r_std::http::UploadBodyStream`, rather than Axum Body. The same lazy
chunk/error conversion previously performed by `start_handler` is performed
at the network edge. Neither constructing the stream nor enqueueing it polls
the source. Handler receive still starts consumption after authorization.

There is no second stream contract or body collection buffer. Chunk values,
error mapping, wire limits, upload budgets and existing single-use request
ownership remain unchanged. Framework code still lives in Core at this bounded
experiment stage; the complete explicit host port and production move remain
pending. This does not certify a whole framework-neutral SDK or final package
DAG merely because two body fields no longer expose Axum.

The three changed production files were checked against actual 888/889 changed
paths before writing: `http_transport.rs`, `http_server/server_model.rs`,
`http_server/server_dispatch.rs` had no observed overlap. Shared entry,
registration, scripts, aliases and HTTP test-support functions were untouched.

##### Executed controls

1. `cargo test -p auto-lang-core --lib owned_upload_ -j 4 -- --test-threads=1`:
   exit 0, **2 passed / 1,622 filtered out**. The tests exercise the actual
   network-edge conversion and actual ApiRequest/PendingUpload types:
   zero polls at projection and bounded enqueue; queue-full rejection drops
   the rejected body exactly once without polling; an actual OS owner thread
   receives the request, obtains exact chunk bytes and the source error;
   dropping the last body owner cleans once. Authorization rejection retains
   the body until pending ownership is dropped, with zero polls and one drop.
2. Standalone probe build with `--locked -j 4`: exit 0. Core product compilation
   reports the same 147 pre-existing warnings as the unchanged baseline; no
   warning cleanup or claim of repository-wide warning-free health is made.
3. Existing JSON/default-assembly control: exit 0, HTTP 200, exact random token,
   actual shutdown delivered and process exit 0.
4. `upload_probe.py`: exit 0. Reuses the actual PLAN-730 upload handler program
   with only existing temporary-root substitutions and an added random fixture
   readiness marker. Original source SHA256:
   `1b4d78ac32af8528c76babff0b9f7e36edc1542b525d615caddd457992cebe38`.
   Accepted multipart input remains **12,582,912 bytes**, above the ordinary
   buffered-body limit. Exact published/download bytes match; same-target retry
   returns 409 and preserves bytes; business rejection returns 422 and empties
   staging; headers-only `Expect: 100-continue` returns 401 without body input
   or staging; shutdown is delivered and the process exits 0.

The last replay is a scoped assertion set, not whole PLAN-730 suite execution
credit. It proves affected value/authorization/file paths, not every existing
cancel/deadline/SSE/VM lifecycle state required before T-02 can pass.

The fixture workspace cannot directly test a dependency package's dev tests;
that attempted command exited 101 before execution. The root workspace then
required an actual read-only UI sibling manifest. Its tree is pinned at
`d53f946ab89ce6bb3cbbb2ea929f094ff2600be6`, clean; all inventoried UI tool source
hashes match the intake. Only the named Core tests ran, no root test tier.

##### Local-only evidence

Files under main `docs/plans/attachments/890/work/`, never staged:

| File | SHA256 |
|---|---|
| `owned-upload-tests.log` | `5f4a115a1bbf2666cbe2178ed46aa8a3f0d23d7279bdd64953f8234c6f95f83c` |
| `owned-upload-probe-build.log` | `129e43be6df22f2c1fc39bde2a872963fb79269e8800fc514035c526852ed730` |
| `owned-upload-json-control.log` | `198d851c6d6761a9ee7d55d5c4081d6fa77e0d6279d3370db5c25a83b00040b0` |
| `owned-upload-wire.log` | `22938201188d9e7c1f0c8b1a9276a95f2e59c03d45efe943cbb52234a4f7996a` |

Remaining T-02 work: explicit host installation/missing-provider behavior,
per-instance metadata/resource ownership, the rest of the real body terminal
matrix, shared generation functions/package DAG and non-HTTP controls, exact
four-repository fixture/runner contract. No mass move, Web bootstrap, final
review, merge gate or delivery claim is authorized by this partial result.


</details>

<a id="e890-f5e34c46a937cfd1"></a>

<details>
<summary>work\owner-metadata-isolation-r2.md · SHA256 6e99dbc5332029d32c82edd31aaf6d881a5a55f9316d56d66a095d10667c4de7</summary>

#### PLAN-890 r2 — live owner metadata isolation

T-02 partial, status executing 1/12; previous unit
`9ee280cba`. No Web repository created, no T-02 completion or full private-port
claim. Temporary single-runtime experiment methods must leave Core in T-05.

##### Counterexample and repair

Two actual VMs serve concurrently on one LocalSet owner thread, with independent
network threads, ephemeral listeners and shutdown watches. The first program
declares `shared(n int)`; the second declares `shared(word str)`. Before repair,
the first returns 200/1007, the second 200/second-hello, then the first changes
to 400/missing word after the second compilation. Both owners still drain;
the nonzero oracle is a metadata failure, not a timeout/environment label.

Each activated HttpVmOwner now captures immutable route/signature/return/shadow
metadata. Every poll selects that snapshot and restores the previous context
on Ready, Pending and unwind. The network's request future receives the same
snapshot, including deferred upload classification; the context is not left
installed globally while a future waits. POST broadcast return lookup also
uses the selected snapshot. No raw VM pointer, Send coercion or VM field export.

Final oracle additionally uses reused handler names with differing file/SSE/
upload signatures. Exact binary file, ordered SSE EOF and headers-only 418
upload rejection survive the second program. Closing the first releases its
listener while the second still returns 200/second-still-active. Both owner
futures join and scope/file/upload/stream/live-op registries are zero.

##### Verification

Logs local-only under
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Result | Log SHA256 |
|---|---|---|
| Pre-repair actual two-VM `isolation_probe.py` | expected exit 1; first response incorrectly 400 after second compile; both drained | two-owner-metadata-before.log `98997370cff42fdfa5e325522497eb69b13b3306c507204fd08621b22e7aacbf` |
| Final named live-resource oracle | exit 0; JSON/file/SSE/headers-only upload survive, first shutdown does not stop second, both owners join, registries zero | two-owner-resource-metadata-final.log `f5f811430bad1a86e71afccaca3d0b96c2a83efb471d54d829526c8353cd8b5b` |
| `cargo test -p auto-lang-core owner_metadata_restores_after_pending_nested_poll_and_unwind -j 4 -- --test-threads=1` | exit 0; 1 passed, 1625 filtered; unrelated console integration 0 selected; nested owner context, Pending restoration and deliberate caught unwind | owner-metadata-context-control.log `f533896d43a2fa55e696c740db21d0f89bc833cc90a727bee52729e5a0d3a172` |
| SDK fixture `cargo build --locked -j 4` | exit 0; existing Core 142 warnings, no new warning | metadata-sdk-build-final.log `7595749ffce027003ac594ac4021bc667d54de091679d18aa4e003509fa85a86` |
| Reply TCP replay after metadata selection | exit 0; sync/async binary, HEAD/Range/ETag/404/500, ordered SSE/EOF, terminal registries zero | metadata-reply-replay-final.log `5a9f3b58d13017ef72435e10382e18f45a24827d9d349186865880ba216aef00` |
| Upload replay after metadata selection | exit 0; original 12MiB, preserved occupied file, 422/401/400, staging empty, terminal registries zero | metadata-upload-replay-final.log `25d9db0df2699ab153e6b20e949140684cc863e41a50df49c50a01d517549f82` |

The final transport formatting edit only wraps the same call arguments; build
and runtime evidence precede that whitespace-only edit. No behavior change.

##### Remaining T-02 gate

This closes the reproduced active-owner metadata overwrite, not every global
state. Producer compilation scratch tables remain global; simultaneous
compilation, axum-router state, legacy CORS/rate state, service observability,
admission/permit isolation, all cancel/disconnect/deadline branches, direct
listen/back-proxy consumers, the final narrow language execution port and
physical production DAG/API/schema freeze still need closure. No partial
runtime bridge is accepted as final framework relocation. No whole-tier tests,
shared scripts/test registration/locks, UI/Man structural work, merge or push.


</details>

<a id="e890-4afef97856c8cca7"></a>

<details>
<summary>work\pure-contract-cut-r2.md · SHA256 4f1b55404e1e2fe5eefe2a01d8691f0d1e15a130938a4007f35b446c881ffe1d</summary>

#### PLAN-890 r2 — pure configuration source cut

2026-10-30. Bounded T-02 evidence; the full port gate remains pending.

The actual schema, strict resolution, immutable configuration, effective hash,
CORS/Host/single-proxy policies now have one source in
`http_service_config/contract.rs`. The existing public
`http_service_config::*` paths re-export those same types. Mutable rate-limit
state, ServiceRuntime, active configuration and process policy state stay in
the runtime facade for the later Web move. No configuration implementation was
copied into a second production module, and this source movement is not net
production deletion or completed repository extraction.

An independent `web-contract-probe` compiles that actual source with only
serde/serde_json dependencies, no Core/VM/Axum/UI/Man dependency. It executes
accepted proxy-service configuration, permitted/hostile Host, a legitimate
single trusted rewrite and rejected multi-hop XFF, exact effective-hash
round-trip, unknown-field rejection and invalid service-profile rejection.
Result: **1 passed, exit 0, no warnings**.

The affected original Core configuration module was replayed on the actual
facade/re-export path: **14 passed / 1,610 filtered out, exit 0**. This includes
default/proxy profiles, strict unknown/range/zero rejection, CORS and Host,
single trusted proxy, bounded rate-limit TTL, port precedence, effective-hash
sensitivity/round-trip and active-config single take. No whole root tier ran.

Four inert `mut` bindings and one unused private health-range constant were
removed while cutting the source. The actual fixed health budget remains 8.
The existing CORS-default assertion now compares the explicit original method
list rather than the implementation's moved private constant; the expected
policy is unchanged and the oracle is independent of that constant.

Local-only logs under main `docs/plans/attachments/890/work/`:

| File | SHA256 |
|---|---|
| `pure-contract-probe.log` | `7285327b9ee89a825d902240dbe5d1b23d7c0571021f36d04be0580e9eeaad88` |
| `pure-contract-core-controls.log` | `3d09557800a1c1ba820d74fea720cbd680e2802b53a13ca9ffa40b36b17d6244` |
| `pure-contract-tree.log` | `07a34b6751b05406d370a9197de98e24273bd39bc620f45fd9fbfb1aee2deff2` |

The probe deliberately compiles a source file in an isolated test package;
it does not claim that production cross-repository package/type distribution
has been delivered. T-03/T-04 must establish one real contract package source
and all consuming manifests/locks. Before-images in the T-01 intake remain
immutable; later source/target mappings must append this move rather than
regenerate the original inventory to hide it.


</details>

<a id="e890-0e4632e6544f4103"></a>

<details>
<summary>work\repair-1-entry.md · SHA256 782534650b2504f6f55e7506000d0cc6d22551fc0fc4781008a359bd038695f2</summary>

#### PLAN-890 r6 repair cycle 1: execution entry

Previous round made progress: independent final review found two reproducible HIGH defects and reopened T-05/08/09/10; it was not a wait or no-progress round. Authorization remains the full-chain goal. No implementation agents.

Entry language HEAD46547a0f8 was clean; synchronized official master74f204884 (including completed894) in the same tree to c73e88f86. Plan add/add uses authoritative main; recipe conflict is the union of46 existing and one delivered894 recipe (47). No overlapping recipe value changed. Other owner pins remain Web9ddb7fe/UI9aad12c/Man cb4ab3d/Down c3464d0, clean. Main has docs-only unrelated WIP, preserved.

| Finding | Actual path and states | Distinguishing oracle / scope |
|---|---|---|
| R1 | full/module/distribution producers -> stored IDs/commands/outputs -> Web complete fixture; valid, omitted, duplicate, excluded, changed output | Preserve genuine45 and full20 controls; reject public zero-execution, unconditional20-to19, required command proof deletion. Reconcile module conditions from original source and execution environment; validate actual commands/list/run/log identity, not self-reported count. Permanent real-consumer and sibling controls; Python governance scoped checks, no global suite. |
| R2 | explicit Web config -> caller TLS launch -> Core run_file[_with_args] -> worker -> captured VM launch -> Web policy/listener | Capture and rebind domain-owned launch at file worker like existing run bridge, preserving thread stack/join/error semantics; actual differing config/env ports, loopback/profile/config identity and Host400, no-config legacy and ordinary language controls. Run scoped Core/HTTP test module and installed rebuilt CLI replay, no generated business edits. |

Reproduce review scripts in a separate repair evidence directory so original review artifacts remain immutable. Exact raw review source/input/result hashes are in t11-final-baseline.json. Fix both invariants before final affected checks; preserve unaffected port/type/DAG, transport/resource, generator, Man/UI and fixed Blog evidence with source/input boundaries. Repaired shipped SDK/CLI inputs require fresh artifact identities and affected distribution recovery checks; old45 is a valid-control receipt only, not certification of changed code.

Final affected complete tm union, real four-owner gate, due main-only batch, all actual landings/Specs/ledger/index/archive/guard remain T-12 after acceptance. Catalog must refresh for actual47 recipes plus new repair tests and consumer closure. No new goal/AC/compatibility/repo/budget change; no waived existing red.


</details>

<a id="e890-0d9015432e61f0d0"></a>

<details>
<summary>work\repair-1-handoff.md · SHA256 d72f61ea3d6409605dfc56ec6a92d496276c4c79fbe56c69d5e278531a6eb628</summary>

#### PLAN-890 r6 repair cycle 1 handoff

Work-owned repairs are ready for independent affected re-review. This is an
implementation/self-audit receipt, not final review, a unified regression PASS,
U890, landing, canonical deposition, archive or cleanup. The previous goal turn
made progress by finding R1/R2; this round repaired and froze them in the same
five authoritative sibling trees. No implementation agents or remote actions.

##### Authoritative freeze

Root `D:/autostack/.wt/lang-890/`:

| Repository | Current committed HEAD | Branch |
|---|---|---|
| language | 2506549321ad8f4699089a7b0dce53d74f78b046 | plan-890-dev |
| Web | e01232b3d449ea590a85d9458d553ab17b9456a3 | codex/plan-890-dev |
| UI | 9aad12ccbc5f01d5241eb95f657666a943667ff2 | codex/plan-890-dev |
| Man | cb4ab3dd5d411314cce10b129eeb3b433a11c466 | codex/plan-890-dev |
| Down | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | detached, unchanged |

All five HEADs/cleanliness and candidate whitespace were read back. Main carries
only unrelated documentation WIP and this Plan's bookkeeping; none was mixed
into implementation. Official delivered894 and shared bookkeeping main74f204884
were synchronized atc73e88f86; conflicts retained authoritative main Plan and the
union of46+1 recipes. Core file/receipt repair74e13580a; proof branch and
responsibility reconciliation0fdca25c0; original registry indentation restored
atb446256b5; final catalog/debt receipt250654932. The final net registry diff is
only42 added lines, no unrelated entry/value change.

Final source/dependency/effective-environment identity:
`815c5287416833e52b7656e0c07764600a53ca532b883bd13168b728abf1ce8f`.
Use `CARGO_TARGET_DIR=G:/autostack-build/plan890-frozen-target` and an empty
RUSTC_WRAPPER for this discovery identity; do not substitute another environment
and claim identity equality. The catalog was actually refreshed on all47
recipes: **7367 obligations /1220 disjoint leaves**, covered once by76 curated
responsibilities. Added894's7 leaves contain32 original observations; no prior
leaf was retired. Source/condition/legacy/atomic/resource/consumer checks passed.
Both periodic manifests name all47 recipes; purpose advice remains separate
from execution eligibility. Mixed owned-text functionality/lifetimes/session/
formal120-sample/four-path uses are pending purpose advice, not disabled tests.

##### Finding closure and exact scope

| Finding | Actual invariant repair | Permanent/replayed closing evidence |
|---|---|---|
| R1 /AC08/09/12 | Required execution is no longer an optional banner: fixed owner cwd/list/run recipes, raw list/output identities, exact selected and per-ID passed results, original conditional applicability and child/case oracles are reconciled. Full scheduled union, source/config identity and consumer closure remain mandatory. Ephemeral catalogs are source-contract revalidated. Required distribution and legacy command steps cannot delete command/exit/log/hash; fresh/reused SDK logs require actual217 IDs. Public combined validation rejects changed packaged inputs instead of certifying old distribution bytes. | Ownership module14pass; original module consumers49pass; actual API20-ID leaf executed and its real proof accepted, exact original20-to19 omitted-ID mutation rejected even with a matching19/19 log; original genuine45 accepted directly and5 required-command deletions reject; current public no-execution fixture rejects. Python and original negative-Case proof controls preserve raw exit1/FAIL when the explicit original meta-oracle requires it. |
| R2 /AC03/06/09/12 | Core `script_entry::run_with_path` captures the neutral invocation launch before its worker and rebinds it around the existing LocalSet. Stack/join/error behavior unchanged; no host/framework dependency added. Web CLI config no longer falls through to env/default listening or policy. | Current Core host module6pass; Web HTTP complete affected file3pass; actual new fixed19-package installed CLI fresh3/relocated3 allpass with different declared/env ports, exact nonce, loopback/profile/hash, Host400, shutdown202/exit0/port release. No-config legacy and ordinary42 accepted. SDK217pass/2original skips; actual unchanged Rust generated-runtime sibling also honors differing config/env port. |

The Core6 cover provider active-owner stability, rate default/explicit disable,
launch nested unwind/interleaved polls, real concurrent compiler/declaration
isolation and new compiled file entries (two different launch snapshots,
run_file_with_args/run_file/no-config/ordinary-file and caller restoration).
The new Web profile checks derive the exact config hash from the real public
resolver, assert loopback diagnostics and reject the legacy env listener. They
do not mirror config and env ports as the old fixture did.

R1's14 controls expand the original10 and add public real-consumer rejection,
ephemeral source-contract validation, both schedule consumers, and Python/
negative-case schema branches. Protocol fixtures use synthetic controlled logs
and are explicitly not claimed as actual test bodies. The separate actual
20-case producer/consumer probe uses real nextest list/run and original IDs:
wall9.570742s, workload CPU8171.875ms, peak process176943104bytes /job334266368bytes;
whole probe14.403972s. It certifies that leaf/protocol only, not a full closure.
Its overall source identity differs from the final freeze solely because two
advisory JSON registries returned from2-space to original1-space indentation:
`repair-1-identity-equivalence.json` reconstructs the exact old input hash,
proves all other input bytes and peer repo hashes equal, and records the actual
checker SHA. No test rerun or arbitrary identity repinning is substituted.

The first trace fixture used[] where the real producer emits a dict; that setup
failure remains in `repair-1-ownership-final.log`. The corrected fixture matches
the real schema. Case/Python branch controls were added before this final code
freeze; all14 pass. Original assertions/IDs, floors, samples, ignores, conditions,
first failures, three historical AAVM reds and Windows path limitations remain.
No weakened golden, budget, default eligibility or timeout was used.

##### Current standalone artifact evidence

New external consumer root `D:/autostack-build/plan890-repair1`, not a second
editable Plan tree. Source SDK is candidate=false with19 actual local packages,
source SHA `b2e0d9831840430fe2408f8e910c20855dadbdac31144f1bc3f195df5a2131bc`.
Producer refs are Lang74e13580a/Webe01232b: later Lang changes are governance,
not packaged SDK input changes. Do not rewrite these actual provenance refs.

Public install cold build: **2m13s**, default dev configuration, locked Cargo,
720 owned files. Fresh and relocated profiles use the rebuilt real executable:
SHA `7220b7bf635ab2057587ec1684648200051d7ea7f78c1a0005f6b855651c3640`.
The original A directory is absent after relocation. Resolved move source/dest
were verified inside the owned D root; all reparse points were refused before
the move. Five relevant installed/current code files are byte-exact. Every
service probe returned shutdown202/exit0 and released its owned port.

The independently installed/source SDK mandatory library leg was run once for
this changed immutable SDK cohort: **217passed/2original skips**,203.198s actual
execution. Its source inventory and installed720-file manifest were reverified
unchanged afterward. Exact reusable SDK input hash:
`3bc6c24bea6ac48d236d41f4af87c2412816714f930f886ae4fa83e38a094a10`;
nextest config SHA `d595de8766d93c16257a43fcc83b1d229c3f63d8f98466d84c28b59946037066`.
The receipt is `repair-1-independent-tests.json`, not a repinned old SDK test.

| New local package | Bytes | SHA256 |
|---|---:|---|
| source ZIP,728 exact entries |3591536|d4f225053d69fd6ef0c2f967f2e82cd5b69db0f8e34433b61d3d9599075af786|
| Windows ZIP,721 exact entries |19395447|d9d9cfe33dcbbe5c88ca7f5d3b5b4ed4b0a5e8dd209ceadb9e296b79de89c2ca|

Both archives were read back for exact unique entry inventory and ZIP CRC; all
inputs matched manifests before packaging. Their total22986983bytes is below
the500MiB release-payload budget. Build/cache/source-consumer trees are temporary
rebuildable workspaces, not promoted durable payloads. No global evidence-budget
green claim or deletion is made; final retention/accounting is merge closeout.

The old T09 genuine45/history14 remain valid only for their recorded actual
inputs and unchanged installer/generation/history mechanisms. Man's immutable
transaction artifact, license/source identity and six independent transaction
controls are unchanged. Their bytes and applicable behavior were not repaired.
**A complete new aggregate distribution/current-cohort/four-owner PASS is not
claimed here.** Final combined artifact/receipt refresh is still the explicitly
merge-owned frozen gate, alongside its actual owner/module executions. The
public fixture now prevents using those older packaged inputs as that future
new-combination certificate. Current feature repairs are actually executed;
this is not deferral of their correctness to a global suite.

##### Work acceptance and reused evidence

| AC | Current bounded work decision |
|---|---|
|01|Original intake/source/history and actual sibling identities preserved; official894 synchronized without taking other WIP.|
|02|Neutral port/type/DAG unchanged; new provider test stays Core-only, actual current SDK19 has no UI/Man implementation.|
|03|R2 file entry repaired and actual configured/missing/ordinary/provider controls close the new gap; original owned port evidence retained.|
|04|HTTP/shared/non-HTTP ownership unchanged; SDK/API20 and real Rust config sibling supplement original T06 compile/wire proof.|
|05|Man/UI code/pins and their actual worker/proxy/caller paths unchanged; T07/T10 proofs reused at those paths.|
|06|Current valid dev/proxy configs drive actual port/loopback/hash/Host400; existing policy/wire tests retained and current SDK217 passes.|
|07|Original owned transport/body/resource algorithms unchanged; current Core6 and six real shutdown/exit/rebind controls pass.|
|08|R1 admission closed; original identity/resource/floor/condition obligations preserved, actual refreshed source partition and19 controls pass. Final complete union remains T12.|
|09|Current immutable SDK fresh locked build,217/2, real relocation/config/policy use and exact source/Windows packages pass; unaffected transaction/history/generation receipts retained with their inputs. Final combined current-cohort certificate remains T12.|
|10|Fixed Blog314 and original Vue15/native22/cold build/API13 source-bound evidence retained; no GUI rerun or business-source patch, unrelated UI emitter unchanged.|
|11|Original domain exits, thin facades, shared carrier/media owner and legacy valid Core shapes retained; no old domain copy restored.|
|12|Changed Rust fmt and all five candidate whitespace checks pass; baseline Core128/Web13 normal and Core140 test warnings remain, no new warning/debug/defer claim. SD proposal is clarified, no new contract/goal/repo/budget.|

The source comparison supporting reuse is concrete: Web production main/host/
generators/runtime and Man/UI code are unchanged by this repair (Web diff only
HTTP tests and distribution driver); language product change is the file-entry
launch handoff and its permanent Core test. Governance/helper/registry/compiled
catalog changes are separately checked. The original GUI/transport/generation/
transaction tests do not substitute for the new R1/R2 proofs, and vice versa.

Work-owned open findings, required repair oracles and user decisions: **none**
after this frozen scoped verification. Independent affected final re-review is
still required; do not infer pass from this table. Canonical Specs/ledger/index
remain unwritten. New proposal: [repair delta](#e890-b6c1120878575113), SHA
`3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172`.

##### Evidence and next owner

`repair-1-evidence-index.json` records16 key raw files with exact bytes/SHA256,
final source identity and counts. Additional raw before/after probes are in
`repair-1-pre/`, `repair-1-replay/`, `repair-1-real-module/` and CLI profile dirs.
Only deliberate Markdown summaries are committed. New actual totals:
Core6; Web HTTP3; ownership14/10.867s; module consumers49/11.119s;
purpose19/30.132s; actual API20; SDK217/2; CLI fresh3+relocated3; Rust sibling1.
No Plan-wide/four-owner/full-catalog test-body gate or periodic batch was run.

Next: independent affected review on these same committed trees, starting from
T11's R1 A/B/C and R2 findings, updated source/Spec hashes, legitimate controls
and sibling branches. Preserve applicable initial-review AC/V/SD coverage.
After a final pass only: T12 frozen affected complete tm closure, actual full
four-owner/current-cohort release gate, any due main-only single-instance batch,
all actual repository landings, canonical knowledge and required ledger/index,
archive and guarded cleanup. Do not waive named old reds or certify original
ignored/manual/unexecuted conditions. Goal remains active until those are real.


</details>

<a id="e890-97599dbf9acc9108"></a>

<details>
<summary>work\repair-1-progress.md · SHA256 0653d9d58a0f02e111bffefad335e91dc87c075d52dfbc500c11e977f37e3eba</summary>

#### PLAN-890 r6 repair cycle 1: verified progress (not readiness)

Language source/governance commit74e13580adff15a861ca46efb5d0ab0ce89c168d; Web test/distribution commit e01232b3d449ea590a85d9458d553ab17b9456a3. Before those, sync official main74f204884 including delivered894 atc73e88f86; original known-debt review commit46547a0f8 preserved. UI9aad12c/Man cb4ab3d/Down c3464d0 unchanged. No code on main, no implementation agents, no remote.

R2 file-entry worker now captures/rebinds the caller's neutral launch, retaining32MiB stack and join/error behavior. Permanent real configured Web tests use differing ports, exact response, loopback/profile/config hash, Host400 and shutdown/rebind; original development/shutdown and upload assertions retained. Core real compiled-file test exercises run_file_with_args two distinct launch snapshots, run_file no-config and ordinary42 without host call, caller context restoration.

Actual checks:
- Web http_runtime complete affected file:3pass (47.74s preparation/build,5.85s execution), default profile, G target.
- Core named file-entry case:1pass0ignored0.10s (50.39s build); unchanged baseline140test warnings, Core128/Web13 normal warnings retained.
- Ownership/receipt module:13pass9.923s, expanded prior10 cases and3 new protocol/public/manifest cases; actual public missing-execution receipt rejects2 with genuine original45 distribution control. Original shortened20-to19 and5 missing-command failures reproduced before edits; after fix legitimate original45 still validates directly with217actual per-ID successes and2original skips, all5 proof deletions reject.
- Real module-selection consumer tests49pass11.119s, including public ephemeral refresh/source preserving behavior.
- New independent source export candidate=false,19packages, sourceSHA b2e0d9831840430fe2408f8e910c20855dadbdac31144f1bc3f195df5a2131bc. Fresh public install cold2m13s,720owned files, source pins74e/e012.
- Real rebuilt installed CLI fresh3/3 and relocated3/3: no-config control, differing configured port, valid proxy Host rejection400; exact nonces, startup loopback/profile/hash, allshutdown202/exit0/portsreleased. Five relevant current-vs-installed source files byte-exact, executable agrees with installer manifest. Move within owned D verification root was preceded by resolved source/destination checks and full reparse-point rejection; oldA absent, newB live.
- Actual unchanged generated-Rust runtime sibling consumed changed divergent-port distribution helper: exact890/shutdown202/exit0/release; binarySHA bc290e19d85bf2cf55d61a5b528970ea04b4510ddaa977c8ee47098c6674fb1d, declared11840/env11841. It is explicitly old unchanged generator/runtime source, not a claim of a newly rebuilt generated artifact.

R1 now requires source/selection/config-bound module report, original applicability/env/exclusions, owner cwd, exact list/run commands, output hashes and per-ID actual success reconciliation. Public four-owner admission consumes actual full scheduled module union and fixed owner recipes, with libtest/nextest listing/run evidence; missing/shortened/duplicate/filter/stale records fail. Runtime ephemeral catalogs are source-contract revalidated, never blindly compared only to published catalog bytes. Distribution command-driven rows require command/exit/expected-exit/log/hash, legacy build/archive siblings require proof, SDK fresh/reuse logs require217actual per-ID successes; direct historical45 remains valid for its actual inputs. Public final combination additionally checks current selected package/SDK inputs against the distribution source cohort; current changed code cannot borrow old45 as its own delivery pass.

Both periodic manifests reconciled to47actual registered recipes, including delivered894 and Web/shared/media consumers, without changing original conditions/floors/default eligibility. Actual47-recipe discovery and current immutable SDK library leg are running; catalog/top-level/purpose refresh and real runner-protocol consumer check still pending. Therefore no work-ready or finding-closure verdict yet; all final tm/four-owner/current-cohort release/land/spec/ledger/index/archive/guard gates remain pending under their existing owners. Do not repin old45 to newsource or call a controlled synthetic unit protocol actual test execution.

Local proof paths share this directory: repair-1-entry.md; repair-1-pre/{short,distribution}.log; repair-1-public-replay.log; repair-1-ownership-final-5.log; repair-1-module-consumers.log; repair-1-web-http.log; repair-1-core-file-entry.log; repair-1-source-export.log; repair-1-fresh-install.log; repair-1-cli-{fresh,relocated}/t11-cli-launch-loss-result.json; repair-1-generated-config.{log,json}. First fixture-shape failure for traces[] vs actualdict is retained in repair-1-ownership-final.log; corrected fixture now matches real producer schema. No product assertion was relaxed.

Rebuildable SDK verification root D:/autostack-build/plan890-repair1 is external consumer output, not another editable Plan worktree or durable evidence-cache promotion. Actual final evidence budgets/retention remain merge closeout work; user-freed D space is not attributed to our deletion.


</details>

<a id="e890-b6c1120878575113"></a>

<details>
<summary>work\repair-1-spec-delta.md · SHA256 3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


##### r6 repair clarification (R1/R2; same existing contract)

SD-02/07/08: the final four-owner consumer requires the full explicit registered
recipe/module union, source/dependency/effective-configuration identity and
original conditional applicability. Required owner cwd, list/run command,
listing/output path and hash, selected IDs and actual per-ID results must agree.
A self-reported success banner/count cannot replace execution proof. Public tm
may refresh into an ephemeral catalog; its source contracts are revalidated,
not weakened or rejected merely because published metadata bytes differ.
Original negative Case oracles can legitimately produce a raw exit1/FAIL while
their explicit meta-oracle passes; those source/oracle identities remain checked.

Distribution command-driven checks require their actual command/expected and
actual exit/output identity. Runtime/detail and exact-input SDK reuse retain
their own explicit schemas; they are not excuses to omit a required command.
Current-owner packaged SDK/crate/installer inputs are reconciled with the
distribution's frozen source cohort at the public combined consumer. A valid
historical receipt does not certify changed SDK bytes. Both periodic manifest
consumers keep every current registered recipe; responsibility/purpose advice
never changes original registration, conditions, samples, budgets or floors.

SD-03/05/08: Core's file entry (`run_file`/`run_file_with_args` through
`script_entry::run_with_path`) captures the neutral invocation launch before
spawning its execution thread and rebinds it on that worker, like the existing
run bridge. An explicit absent launch remains absent; ordinary no-service files
need no Web provider. Web `run --config` consumes the declared loopback/address,
port, profile/config identity and policy even when legacy environment values
differ. Existing no-config legacy behavior remains. No new protocol, network
product, platform promise, package or dependency edge is introduced.

Evidence remains source-bound and scoped: current rebuilt SDK217/2, fresh and
relocated CLI configuration/Host controls, original unaffected generator/Man/UI
proofs, and explicit final current-cohort release/four-owner gates pending under
T-12. Do not deposit a new aggregate PASS before that frozen combination actually
executes. The known first failures and original manual/ignored conditions remain.


</details>

<a id="e890-746a7a518ef25bca"></a>

<details>
<summary>work\repair-2-diagnosis.md · SHA256 af6b63853780daee8f477048b4c2190d26b0f1992deb56edaf9d357467cc2987</summary>

#### PLAN-890 r6 repair cycle 2: verified diagnosis before implementation

Entry: main d43cc9437, authoritative language e6f15ec019ad74cf06ea94c74e8230624d058073 (reviewed implementation250654932 plus debt-only commit); Web e01232b, UI9aad12c, Man cb4ab3d, Downc3464d0. All implementation trees retained; main carries only unrelated docs WIP. No implementation agents. Prior goal turn was progress: independent finding R1-D established and state routed to executing8/12.

##### Remaining invariant and verified cause

AC-08/12, T-08 and dependent T-10: complete public owner admission must keep original source-bound IDs and frozen floors. The actual producer calls baseline_for / validate_ids / check_minimum; consumer checks command/cwd/hash and listing==selected==actual-PASS only. Repair1 accepted owner protocol used a single arbitrary test. Its independent-field deletion controls therefore could not distinguish consistent deletion of an original required test. This is a denominator gap, not crypto scope or a new test eligibility rule.

Pre-fix reproduction: copied durable independent t12-owner-missing-id.py into repair-2-pre (original review evidence unchanged). With original frozen target/env, `python docs/plans/attachments/890/work/repair-2-pre/t12-owner-missing-id.py` exit0 reproduces UI3110→3109 (producer missing1/suppressed1; public owner checkPASS) and Man281→280 (producer below-floor; public checkPASS). Valid controlsPASS. Public totals10PASS/0FAIL/1pending and exit2 deliberately omit distribution; synthetic protocol output does not claim test bodies ran. Log repair-2-pre/replay.log. This verifies sensitivity before code edits.

| State/path | Diagnosis and design/oracle adjustment | Closing observation |
|---|---|---|
| UI original baseline IDs, including suppressed/ignored/filter changes | Reuse producer's exact original baseline/migration/retirement guard at receipt consumption, and share its guard function rather than copying a weaker rule. Positive fixtures retain the real source denominator instead of one arbitrary ID. | Complete source baseline accepted; consistently deleted or suppressed required ID rejects in validator and public gate. |
| Man nextest legs / UI integration nextest and libtest | Apply the same frozen per-leg minimum checks to all required external owner legs, using original kind/commands. | Full positive floor controls accepted; consistent below-floor listing/output rejects for representative Man nextest and UI libtest, with expected rejection propagation. |
| Language source baseline and migrated UI/Web/media tests | Apply shared baseline/floor rules to language too. Migration credit derives from authoritative original required IDs, not receipt's count, and at consumer only when all target IDs actually executed in already-validated module legs. | Original source set preserved; missing/suppressed original ID or missing migration execution cannot earn credit; legitimate migrated complete targets accepted. Original ignored N/A remains N/A. |
| Existing module closure, release and CLI behavior | Keep original A/B/C negative and full module contract tests; R2 source/SDK/UI/Man transport and artifact inputs untouched. | Ownership and actual prior R1-D replay + related full-driver/unit consumers pass; existing unaffected raw/source hashes retain bounded reuse. |

Changes remain within approved r6 contract and repository set. No acceptance/floor/ignore/sample/condition change; no architecture revision is necessary. Implementation readiness requires permanent controls for shared producer/consumer guard and public admission, not merely replaying the two reported examples. Final complete affected tm union, real four-owner/current release and due main periodic gate remain T-12 pending after acceptance review.

##### Verified adjustment and implementation results

Shared baseline/floor guard implemented at027e8b89c; producer primary and extra legs and the public receipt consumer call it. Consumer derives credit from exact passed targets, not receipt counts. Before expensive execution-proof verification it preflights every required owner denominator, then validates every execution and actual migration target. Follow-up331e7b9a7 preserves legitimate UI libtest ignores: real --list lacks ignore attributes, so the already required same-source ui-full nextest list supplies the two original flags. The genuine T10 log has866 actual PASS and2 ignored; rust.rs ignore declarations are unchanged from T08 to current9aad. Source guard computes866 active, both original N/A IDs, and retains floor863. This is original applicability preservation, not a new exclusion or floor reduction.

Permanent tests now retain original language/UI baseline IDs; nextest/libtest floors; consistent omitted/ignored/filtered UI IDs, omitted language IDs, Man/ui-tu below-floor proofs and fabricated migration credit; exact public failure propagation; required migration target execution; legal original ignored identities and arbitrary-exclusion rejection. Final owner+driver41/41 passed74.572s, including ownership15 and full-driver26. The earlier123.446s ownership run preceded denominator preflight; its source belongs to027e, not the final timing. No P90 or module-wide standalone timing claim. Original independent R1-D positive control passes and both negatives reject; final repeat is recorded separately.

Two test-fixture failures were corrected without weakening assertions: Windows CRLF made a libtest text deletion a no-op (now splits lines and asserts selected count decreased exactly1); historical runner tests globally mocked subprocess.Popen and omitted real boundary output files, conflicting with source-Git checks and repair1 output-proof requirements (scoped runner subprocess proxy and explicit synthetic command output). Original20-error,9-failure and no-op-control logs retained. No product check or original test assertion was removed.


</details>

<a id="e890-2709e784f3aa9c94"></a>

<details>
<summary>work\repair-2-handoff.md · SHA256 b4a73dd262592082bf3e178077dd3555e9ff40f9428e1711ca6e8e121bf70f17</summary>

#### PLAN-890 r6 repair cycle 2 — work-complete handoff

Outcome: **pass for work readiness; independent final affected review pending**. Current shared Plan becomes execution_done10/12. T-11/T-12 remain incomplete. No final acceptance, current-cohort release/four-owner/module gate, canonical publication, merge, archive or cleanup is certified here.

##### Frozen authoritative inputs

Group D:/autostack/.wt/lang-890; original trees retained, no links or implementation agents. Main bookkeeping base d43cc9437988d2addd9ff35fd08879c7dd7ef8ee carries unrelated docs WIP; none was staged. Work entry e6f15ec019ad74cf06ea94c74e8230624d058073 is reviewed250654932 plus debt-only tracking. Current five trees are clean:

| Repo | Branch | Handoff HEAD |
|---|---|---|
| auto-lang | plan-890-dev | 29cf444809e9166c6cfb98ff741b31ae6bc112cf |
| auto-web | codex/plan-890-dev | e01232b3d449ea590a85d9458d553ab17b9456a3 |
| auto-ui | codex/plan-890-dev | 9aad12ccbc5f01d5241eb95f657666a943667ff2 |
| auto-man | codex/plan-890-dev | cb4ab3dd5d411314cce10b129eeb3b433a11c466 |
| auto-down | detached | c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 |

Implementation027e8b89c shares source-denominator admission; follow-up331e7b9a7 preserves original libtest ignore applicability and performs denominator preflight before expensive execution proofs;29cf44480 freezes catalog and debt readiness. Only four Python implementation/test files plus the changed infra-web catalog shard/metadata and in-Plan debt summary changed. No Rust, lock, UI/Man/Web/Down production source, gate minimum, original ignore, condition, sample, profile or eligibility change.

Source identity **03f08f193042eb7670a487baf4202b97404b5c04e075dabfe9fdc2691b36bf33**, effective environment a92a86a788c826d4bc7bd0e7b02da4cce96952501b3df5e775a492bb884ff220. Reproduce with CARGO_TARGET_DIR=G:/autostack-build/plan890-frozen-target and empty RUSTC_WRAPPER. Catalog discovery provenance records331e; final29cf only changes excluded catalog/docs, independently verified identical code identity. Actual current catalog47recipes/7368obligations/1220leaves/76responsibilities; one new permanent ownership test ID, all former IDs retained.

##### Mandatory two-failure diagnosis and invariant closure

[repair-2-diagnosis.md](#e890-746a7a518ef25bca) was recorded and linked in Plan§9 **before implementation**. Pre-fix durable R1-D replay at250654932+docs e6f15ec01 reproduced both UI3110→3109 and Man281→280 admitted by public owner check, with producer rejection and valid controls. Original review files remain unchanged; copied pre-replay and logs live in repair-2-pre.

Cause: repair1 reconciled listing/run fields but did not consume the producer's original baseline/migration/retirement/floor denominator. Its arbitrary one-test positive owner fixture could not detect consistent deletion. Adjustment is implemented and verified: same validate_owner_selection in producer primary/extra legs and actual stored-receipt admission; complete original-baseline positives; internally consistent deletion/ignore/filter/output/count/hash negatives; actual public failure propagation. This is the same r6 contract, not new log-authentication or crypto scope.

| Finding / existing invariant | Repair and permanent oracle | Current evidence / remaining gap |
|---|---|---|
| R1-D: UI/language original required IDs survive receipt consumption | Shared baseline_for/validate_ids/retirement/migration guard; no self-declared denominator. Required ID omitted, ignored or filtered while all proof fields match rejects. | Owner controls and original independent public UI holdout reject missing1/suppressed1. Original source controls accepted. Work repaired; independent re-review pending. |
| R1-D: Man/UI nextest and libtest original floors survive receipt consumption | Shared check_minimum; consistent Man281→280 and ui-tu below-floor proof reject, including fabricated migrated_original_required1000000. | Current owner controls and real original Man public holdout reject280<281; valid controls pass. No floor was lowered. |
| R1-D: migrated original required tests only earn execution credit when targets actually passed | Derive exact targets from authoritative mappings. Receipt consumption checks targets against already validated module PASS/applicable IDs, not selected/catalog membership or a receipt count. | Both language/UI valid targets accepted; absent actual targets reject. Earlier original complete module/migration guards remain. |
| Legitimate original UI ignore applicability | libtest --list prints ': test' even for ignored tests; ui-tu uses the already required, same-source ui-full nextest flags and exact condition_excluded IDs. Keep single-process cargo-test run. | Original two ignored identities accepted as N/A, arbitrary exclusion rejects; source-bound genuine T10 log866PASS/2ignored matches active set866 with unchanged floor863. Original rust.rs ignore file is unchanged from T08 to current9aad. No bodies rerun for this replay. |
| R1 A/B/C previously closed examples | Existing no-execution/public, exact complete module IDs, command/output/legacy/SDK proof controls retained. | Current ownership15 includes these controls. Existing actual20-ID proof accepted on current catalog, exact consistent19-ID mutation rejects; not a fresh execution or complete-union pass. |
| R2 configuration handoff | No affected production input changed. | Independently closed in attempt2; old7220 rebuilt SDK actual CLI3/3 and scoped evidence retained. No new CLI body run necessary for governance-only repair. |

No open in-contract implementation finding, required work-owned oracle gap or semantic decision remains in this handoff. Independent acceptance remains necessary; this internal readiness claim does not close R1-D on behalf of the reviewer.

##### Actual checks and raw identity

Run from authoritative language tree unless noted. Logs/JSON/probes remain local; repair-2-evidence-index.json records actual paths, sizes and SHA-256.

| Check | Actual result and scope |
|---|---|
| python -m unittest scripts.test_full_regression scripts.tests.test_web_test_ownership -v | **41passed /74.572s**, exit0 from unittest (shell completes after printing log); full-driver26 + ownership15. Real public positive owner checkPASS, UI/Man consistent-drop checksFAIL. Deliberately absent distribution keeps positive public aggregatepartial/exit2; no whole-delivery PASS claimed. Synthetic protocol logs/child observations exercise admission, not actual owner test bodies. |
| python repair-2-replay/owner-missing-id.py | Final original independent holdout adaptation, expected outcomes updated only: real retained UI3128listed/3110selected positivePASS; original required ID3110→3109 rejects; real Man281 positivePASS and281→280 rejects. Actual production validator and public owner checks, only external fingerprint substitution as permanent protocol; no validators patched. |
| python repair-2-ui-ignore-compat.py | Genuine stored T10 actual866PASS/2ignored output and T08 source-bound nextest flags reconcile868 listed→866 active; exact N/A names and unchanged rust.rs verified, floor863 preserved. It is a lossless run-derived protocol transform, not a new raw list-command or full-owner certificate. |
| consume existing actual20-ID row using validate_module_result(current_catalog,...) | Accepted20; exact shortened19-ID proof rejects. No bodies rerun; underlying module validator/source/options/inputs unchanged. |
| discover_modules.py, all47 explicit --recipe arguments; build_modules.py with all47 RECIPE=JSON inputs | Both exit0. Final discovery sum23.426s (build/list/discovery only, not execution timing). Catalog complete7368/1220, before/after source identities stable. Logs repair-2-final-discovery.log / repair-2-catalog-build.log. |
| audit_test_purposes.py --check; python -m unittest scripts.tests.test_test_purpose_policy -v | Check exit0,1220leaves→76groups/7368obligations/23ledger rows. Controls **19passed /29.481s**, exit0; advice-only original conditions preserved. |
| git diff --check and exact source/Spec/readiness hash checks | Passed; all five trees clean at final readback; twelve canonical Spec inputs byte-identical to attempt2. No new production/compiler warning source; no whole-tree warning-clean claim. |

Original repair1 module-selection consumers49/11.119s retain their unchanged code/assertion/input scope. A repair2 combined84-case diagnostic also actually passed the49 module-selection and9 governance cases, but its overall result failed on historical queue-fixture incompatibilities; it is **not** reported as84pass or a final gate.

Preserved failures: first new libtest deletion was a Windows CRLF no-op (assertion caught it; corrected splitlines and explicit count-minus1 assertion). Driver diagnostic84 had20 mock-boundary errors; after narrowing subprocess mock scope,9 output-file fixture failures remained. Both are retained; queue fixtures now produce their synthetic boundary files and allow actual source-contract Git reads, with original assertions preserved. Final41 passes include every affected full-driver case. Earlier123.446s ownership run is on preflight predecessor code; final41 total74.572s is the current measured check, no P90 or inferred per-module timing promise.

##### Evidence reuse and acceptance map

SDK equivalence is verified by actual Git comparison against actual producer language74e13580adff15a861ca46efb5d0ab0ce89c168d / Webe01232b3d449ea590a85d9458d553ab17b9456a3 over every previously frozen packaged crate/root/stdlib/tool/fixture input. Governance scripts in language are not packaged inputs. repair-2-final-identity.json records current code identity, five source pins, exact twelve canonical Spec hashes and empty scoped SDK diff. Installed binary/source receipts retain their original producer commits and are not repinned to current governance HEAD.

| AC | Work readiness / evidence retained or updated |
|---|---|
| AC-01/02 | Retain attempt2 provenance/type/DAG/owner mapping and unchanged actual SDK source comparisons. |
| AC-03/06/07 | Retain independently closed R2, actual configured ports/profile/hash/Host, lifecycle/owned closure within recorded inputs. |
| AC-04/05 | Retain unchanged generators/shared ports and UI/Man consumer pins; current actual20 proof consumption adds no new product claim. |
| AC-08 | Work repaired by shared source denominator, original-ID/floor and migration-execution oracles above; independent affected re-review required. |
| AC-09 | Retain current fixed SDK217/2/fresh-relocated scoped proofs; historical45/history14 only original recorded mechanisms/inputs. Final current-cohort aggregate release remains T-12. |
| AC-10 | Retain fixed Blog314, Vue15/native22 and original UI866/2 proofs at unchanged pins; no GUI replay. |
| AC-11 | Retain physical owner exits, original test/resource migration contracts and historical valid Core shape; no broader historical UI/Full claim. |
| AC-12 | Work repaired, current41+19/purpose/source health and SD freeze; original failures/ignores/conditions/warnings remain explicit. |
| AC-13 | Pending independent review on current five commits; no self-granted pass. |
| AC-14 | Pending merge-owned actual final gates, all repo landings, canonical Specs/required ledger/index/module-plans, archive and every guarded cleanup. |

Updated proposed SD: [repair-2-spec-delta.md](#e890-cd5db04e23fa0dc0), retains nine operations and exact metadata targets; clarifies the same original SD02/07 invariant and legitimate ignore handling. Canonical/ledger/index untouched. No semantic Plan revision or new user decision needed.

##### Handoff and future gate ownership

T-08 and dependentT-10 are work-complete; restore10/12. Next is independent affected final review, attempt3, same authoritative trees/context separate from implementation. Repair cycle2 of maximum3; diagnosis and verified oracle adjustment required after two failed final reviews are fulfilled, not reset by this internal pass.

After acceptance/SD pass only, T-12 owns frozen complete affected tm leaves+consumers/integration union, actual required four-owner and current-cohort distribution/history certification, and due main-only single-instance cargo tf --batch. All remain **NOT_PASSED**. Preserve original three named AAVM reds (temporary Rust missing auto_lang and INVALID_BYTECODE:60), original manual/platform/milestone/ignored distinctions and original failed outputs; no waiver or list-only certificate. Due 890 periodic batch has not run here. No remote push/publication authorized.

Only concise MD summaries are staged. Local attachment bytes were142.61MiB before final discovery additions; this is not full Plan/global-budget certification. Rebuildable external SDK/Cargo caches are not durable payloads; merge must perform user-required14-day retention/budget accounting and guarded cleanup without touching unrelated scopes. Prior global over-budget receipt remains historical, not claimed fixed. Main unrelated docs WIP and other worktrees/processes are preserved.


</details>

<a id="e890-cd5db04e23fa0dc0"></a>

<details>
<summary>work\repair-2-spec-delta.md · SHA256 40791c32bd48a6e30473d67748d286a7005ec428fcb149cfb7267f6e9893b681</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


##### r6 repair clarification (R1/R2; same existing contract)

SD-02/07/08: the final four-owner consumer requires the full explicit registered
recipe/module union, source/dependency/effective-configuration identity and
original conditional applicability. Required owner cwd, list/run command,
listing/output path and hash, selected IDs and actual per-ID results must agree.
A self-reported success banner/count cannot replace execution proof. Public tm
may refresh into an ephemeral catalog; its source contracts are revalidated,
not weakened or rejected merely because published metadata bytes differ.
Original negative Case oracles can legitimately produce a raw exit1/FAIL while
their explicit meta-oracle passes; those source/oracle identities remain checked.

Distribution command-driven checks require their actual command/expected and
actual exit/output identity. Runtime/detail and exact-input SDK reuse retain
their own explicit schemas; they are not excuses to omit a required command.
Current-owner packaged SDK/crate/installer inputs are reconciled with the
distribution's frozen source cohort at the public combined consumer. A valid
historical receipt does not certify changed SDK bytes. Both periodic manifest
consumers keep every current registered recipe; responsibility/purpose advice
never changes original registration, conditions, samples, budgets or floors.

SD-03/05/08: Core's file entry (`run_file`/`run_file_with_args` through
`script_entry::run_with_path`) captures the neutral invocation launch before
spawning its execution thread and rebinds it on that worker, like the existing
run bridge. An explicit absent launch remains absent; ordinary no-service files
need no Web provider. Web `run --config` consumes the declared loopback/address,
port, profile/config identity and policy even when legacy environment values
differ. Existing no-config legacy behavior remains. No new protocol, network
product, platform promise, package or dependency edge is introduced.

Evidence remains source-bound and scoped: current rebuilt SDK217/2, fresh and
relocated CLI configuration/Host controls, original unaffected generator/Man/UI
proofs, and explicit final current-cohort release/four-owner gates pending under
T-12. Do not deposit a new aggregate PASS before that frozen combination actually
executes. The known first failures and original manual/ignored conditions remain.

##### Repair cycle 2 clarification (r6 unchanged contract)

SD-02 / SD-07 / AC-08/12: source-bound full-owner receipt admission and the execution producer share the same baseline/migration/retirement/frozen-floor selection guard. Matching list/run/IDs/count/hash alone cannot remove an original required ID or lower an owner below its original floor. Consumer migration credit comes from original required-source mappings and is legal only when the exact target IDs actually passed in the already validated complete module union, not merely appeared in the catalog/selection or a receipt count. Language/UI source baselines and Man/UI nextest/libtest floors preserve their original applicability and eligibility. No floor, original ignore, condition, sample or milestone is relaxed.

Implementation: scripts/run-full-regression.py::validate_owner_selection is shared by primary and extra execution legs and scripts/test-governance/web_delivery.py::validate_four_receipt. Required original UI/language IDs and Man/UI libtest floor controls include internally consistent list/output/count/hash mutations. The public fixture verifies failure propagation. This is the original r6 proof contract; no new cryptographic/log authenticity, performance, platform or product promise.

SD-03/05 and all unaffected owner, lifecycle, generator, artifact and Blog clauses retain the repair1 proposal and their explicit bounded evidence. SD-08 publication remains after independent acceptance and final required current-cohort/module/four-owner gates. Current canonical documents and live ledger/index remain unwritten at work stage.

UI single-process applicability detail: ui-tu keeps the original cargo-test single-process command. Because libtest --list omits ignore flags, its original ignored identities are reconciled against the already proved same-source ui-full nextest listing and recorded as condition_excluded. Exact selected IDs and actual passed output exclude those identities; arbitrary self-reported exclusions reject. The existing ui-tu floor stays863, and the genuine current UI866-pass/two-original-ignore result remains consistent. Ignored bodies are not counted as executed or passed.


</details>

<a id="e890-e0613fdbf61b84e5"></a>

<details>
<summary>work\repair-3-diagnosis.md · SHA256 0305691854ef1b678f288e30f37b7d055250a79d53e9ba3fc27e1fb8ef77397d</summary>

#### PLAN-890 repair cycle 3/3 — diagnosis entry (2026-10-31)

Previous goal turn made progress: actual frozen gate failures, partial results and original report restoration were verified, blockers recorded, and Plan routed to Work8/12. No implementation agent. This stage resumes authoritative Lang d4158137e, Webde1783f, UId347dd3, Manb9f0c02, Downc3464d0; main289420c89. Main code paths clean, five worktrees clean; no Cargo/rustc/nextest process at entry. D free182389932032, G free31035420672 bytes. Original bound negative inputs/log hashes in ../merge/gate-needs-fix.md remain the pre-fix replay evidence.

| Finding / existing invariant | Verified actual path and cause / next discriminating oracle |
|---|---|
| M1, T08/T10 / real preparation before consumer | a2r_hir_892_tests maintenance_build_worker inherits CARGO_TARGET_DIR; actual frozen gate redirects it to G, while worker_exe and case adapter look at experimental/auto-a2x/target/release. Other nextest-isolated tests assume a prebuilt worker; test-order cannot supply preparation. tools recipe omits a2x entirely. Repair managed tool declaration/selection and consume its explicit worker env in test helper; separate observable preparation from test timing. Full six-test leaf must pass including tampered-golden FAIL and bogus-worker ERROR controls; assertions/goldens unchanged. |
| M1b / real owned-process accounting | first-case test builds Cargo inside workload, bringing MSVC vctip descendant into workload job; code100 and meter124 were both genuine. Moving this producer to existing preparation boundary permits only existing hash-pinned compiler auxiliary cleanup in prepare(), never exemptions for test workload. Complete leaf must show actual cleanup/no meter error; original meter evidence retained. |
| M2 / merged zero-dependency AAVM consumer | call_module_methods File.read_text_range unconditionally emits auto_lang::a2r_std::fs, despite merge_mode skipping external runtime setup. AAVM prelude already owns File::read_text_range and zero-dependency Cargo manifest. Ordinary output needs runtime; merged output needs the existing prelude contract. Both original compile leaves and applicable ordinary file-range consumer must remain valid; no manifest dependency workaround, duplicate new runtime, corpus reduction or golden updates. |
| M3 / linked reference follows actual bytecode | verify_linked_native_closure uses Disassembler instruction operands, except stale CREATE_FUTURE two-byte captures while producer writes u32 names. This is an inspected sibling inconsistency, not yet established cause of error60. Need identify original golden case and actual opcode/boundary before repair. Preserve native admission and invalid-bytecode rejection; no bypass. |

Before substantive fixes the actual path/oracle adjustments above are recorded. First next step improves failure context on original golden reference assertion and obtains exact runtime bytecode evidence. Related fixes and sibling controls will then be checked once on final repair inputs. Unified union, four-owner, final distribution/history and periodic gates remain merge-owned and NOT_PASSED. Prior review remains historical; changes require affected independent review. User14-day retention and explicit ledger/index policy override conflicting installed skill defaults; no retention deletion.

##### Established M3 cause and sibling inspection

Diagnostic-only replay `cargo nextest run -p auto-lang --lib --features test-aavm --config-file .config/nextest-full.toml --ignore-default-filter -E test(=tests::aavm2_a2r::test_aavm2_goldens_check) --no-tests fail` exited100; original g30_closure_nested fails at byte60=0x0a, preceding bytes end CALL_CLOSURE(0x94), argc2, CALL_NAT(0x72). Actual engine consumes one-byte argc; Disassembler incorrectly declared CALL_CLOSURE zero-width, so linked verifier reads argc2 as POP_N then skips part of successor native instruction. This can both reject valid bytecode and miss invalid successors. Fix decoder width, with direct valid/invalid-successor regression; keep original golden oracle unchanged. Sibling CREATE_FUTURE u32 capture producer/engine matches disassembler, so remove verifier's stale special two-byte override and test capture bytes that would be invalid if executed plus actual invalid successor rejection. No admission bypass.

M2 sibling inspection also found standalone guest Cargo inherits CARGO_TARGET_DIR while cache/publication expects staging/target/release. Remove only this target override for the two owned standalone guest builders, retaining source-addressed staging/publication and original dependency-free manifest. Ordinary project build environment is unchanged. This is the same producer/output-owner boundary mismatch as M1, not a new product feature.

Rust diagnostic error now identifies the invalid byte; original golden assertion includes the case name. No raw debug print remains. Minimal check union: all six original a2r-hir tests; both original AAVM compile corpus leaves; both original aavm2-a2r direct leaf tests; two new linked-boundary unit controls; original file-range VM and runtime parity siblings. Managed worker preparation runs once outside workload timing, retaining exact source-built artifact/hash/meter receipt. Existing module-selection and native-meter preparation controls cover changed tool dispatch and preparation boundary. No full catalog execution during repair.

##### First targeted run and adjustment

Original managed worker preparation succeeded13.13s (hash/meter in repair-3-targeted/tools). Actual complete original four-leaf union plus two new boundary controls and runtime parity selected13 nonignored tests:11PASS/2FAIL, workload100, meter.error=null (no leaked descendant). All six worker tests, original two-test goldens/main-dump leaf, two boundary controls and runtime parity pass. Original VM File-range test is ignored and remained NOT_APPLICABLE; it was not one of the13 executed tests. Existing Python module-selection plus three genuine preparation/reap controls53PASS22.086s.

Both AAVM compile leaves now reach generated File prelude but reject the emitter's usize conversion (Into<i128> not implemented for usize). Retain signed language integer arguments specifically in merge mode so existing prelude owns negative/zero-range rejection; ordinary runtime emission/casts unchanged. This is a same-cycle refinement from actual compiler evidence, not a new repair cycle. Repeat only these two invalidated complete leaves; reuse the unchanged eleven passing checks. Original logs retained, no replacement green overwrites the first red.

##### Consumer audit refinement before readiness

The first two AAVM leaves pass with the provisional merge-mode emitter adjustment (repair-3-aavm-final, exit0), but source audit found script_entry::trans_rust_merged is public and serves arbitrary merged projects. Routing every merged File call into an AAVM-specific File prelude would change unrelated consumers. Therefore discard that provisional production-emitter edit in full (Git blob restored to HEAD). Keep the emitter's original public runtime namespace and add a thin namespaced forwarding function in the existing dependency-free AAVM guest prelude, delegating to its original File::read_text_range implementation (no new runtime implementation or Cargo dependency). This repairs the actual incomplete consumer shim contract and preserves generic merged emission byte-for-byte. The existing usize emission is unchanged; the fixture forwarder converts to u64 for its original Into<i128> implementation. Repeat only the two affected original AAVM compile leaves on this final consumer fix. Earlier provisional green is not final-source evidence.

One local replay script initially raised NameError after successful listing/build and before test execution (repair-3-aavm-retry); no body result is claimed. Its subsequent run used a new output directory. Original raw evidence is retained.

##### Final consumer result and scope boundary

Final source75af80a91: original AAVM compile-corpus and compile-use-corpus both PASS (62.081s/24.777s, nextest total86.875s; native meter94.576s, CPU76.000s, Job peak858910720, error=null). Final actual emit source blob is exactly the pre-repair public emitter; no provisional merge-mode path/cast change remains. Initial eleven passing original worker/golden/runtime/boundary tests retain unchanged relevant source/configuration; two changed AAVM tests supersede their first failure only for final inputs, without overwriting original failures. Python53 tests passed; no full union rerun in Work.

M3 scope is the actual linked native-reference walk and the shared diagnostic Disassembler it calls. Other ABT textual format/legacy operand_size tables were inspected and contain separate historical width inconsistencies; no claim is made to have repaired all bytecode tools. Those paths are not changed or consumed by this repair's actual closure walk; a general ABT/whole-opcode migration is outside PLAN-890. The original instruction engine format is preserved. This is not a deferral of M3 native-closure acceptance.

Public cargo tm selection is now refreshing actual47-recipe discovery after the three new permanent controls, with no test-body execution. Result/catalog publication and final current-source handoff remain to be checked before readiness. Other Plan896 process activity appeared during the final two-case measurement and discovery; timing is a single observed run with actual background, not a comparative or percentile claim.

##### Catalog ownership repair (same cycle)

First actual source discovery completed47/47 exit0, but persistent catalog validation correctly rejected the new direct leaf `language/auto-lang-core/stdlib-assembly-reference-instruction-boundary/direct/001` as missing a top-level owner. Register that exact new leaf under existing core/stdlib-assembly responsibility (aaa7307f7); all previous leaves and classifications remain unchanged. This is required registration of permanent controls, not bypassing conservation or retiring tests. Fresh actual discovery/selection uses repair-3-selection-final; no test bodies execute in either discovery. Warning comparison now includes the original build plus actual original nextest discovery/run logs:288 distinct messages both sides, zero added message categories, existing warnings retained rather than declared absent. Initial comparison against build-only source was incomplete and was replaced by this complete baseline comparison; no product warning was hidden.

Current main attachment890 scope contains3584files, unique inode data221363851bytes (211.11MiB), below500MiB for that scope. This is a scoped observation, not a global budget PASS; other external source/build/artifact scopes and current global registry remain final-closeout audit work. Installed v2 registry has no890 plan/scope entry and conflicts with explicit user14-day retention, so no apply/deletion was attempted. Raw cache/evidence remains local.


</details>

<a id="e890-a013784219215e74"></a>

<details>
<summary>work\repair-3-handoff.md · SHA256 50ed90bf46a1f517c785c0ebf26184d72fca99e8792ec975e862040a5765bc5b</summary>

#### PLAN-890 r6 repair3 readiness handoff (2026-10-31)

Work outcome: PASS for the third/final bounded repair cycle; next owner is independent affected final review. This is not T-12 delivery certification. Plan becomes execution_done10/12, T-11/T-12 remain open. No implementation agent, landing, remote push, archive, worktree removal or retention deletion occurred.

Frozen authoritative trees: Lang `2ef7bece2fa6b2509ed04506789059b997728093`, Web `de1783f1c94dbc4d656958faca74ab61a69831ba`, UI `d347dd3d888545f6f07fe67b7f4c7f1be28741c8`, Man `b9f0c0259ae8b167a635c978dab551e382dbc894`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`; all five clean. Reuse these exact trees, no copies. Implementation75af80a91, control owneraaa7307f7, published catalog/blocker bookkeeping2ef7bece2. Proposed Spec delta a77d62978dfe65230a12e993adcc20c3688cc81f1925924db33191fe8acd816c; prior SD01..09/repair2 remain within their explicit support boundaries, with repair3 addendum.

| Finding → invariant / actual fix | Current closing evidence / remaining gap |
|---|---|
| M1 → real source-built worker before consumer | Existing prepare_tools/toolchains + actual language tool override build a2x and supply AC_A2X_HIR_WORKER. Six original a2r-hir controls all PASS including missing-worker ERROR, tampered golden FAIL, wrong/matching credential and certify rejected/accepted paths. No cases/assertions/goldens removed. New actual-policy Python control covers both suite and direct consumers, mixed-module rejection and unchanged unrelated tools. |
| M1b → real process accounting | Nested first-case maintenance build is bypassed only when explicit worker is supplied; bare maintenance target is made explicit. Existing preparation-only exact-image/hash/completed-reap rule unchanged; actual13-test workload meter error=null. Three original leaking/clean producer and hash/reap controls PASS. No workload leak exemption added. |
| M2 → unchanged emitted contract, valid independent guest | Thin namespace forwarder in original AAVM prelude delegates to existing File implementation. Standalone guest Cargo removes inherited CARGO_TARGET_DIR so target/cache publication agree. Final original compile-corpus and compile-use-corpus both PASS62.081s/24.777s; nextest86.875s, meter94.576s/CPU76.000s/Job858910720bytes/error=null. Temporary generic emitter changes were fully discarded; final emitter Git blob equals pre-repair HEAD. No Cargo dependency or duplicate new runtime implementation. |
| M3 → walk actual native-reference instruction boundaries | CALL_CLOSURE decoder consumes its engine's one-byte argc; verifier uses common Disassembler for four-byte future capture names. Original goldens PASS; both new accepted/invalid-successor controls PASS. Entire reference area7/7 PASS (including all five original selected-interface/signature/TCP-refusal/JSON override controls); meter2.670s/CPU2.688s/Job201547776bytes/error=null. Native admission and invalid-bytecode rejection remain active. |

Verification details: first repair-3-targeted run selected13 nonignored tests,11PASS/2 compiler failures (unresolved usize-to-i128 after provisional adjustment). Final M2 consumer fix supersedes those two failures for new inputs, never overwrites the originals. Original11 passing checks have unchanged relevant implementation/inputs; subsequent reference7 includes two repeated boundary controls plus five existing native admission checks. Thus18 distinct Rust tests have scoped passing evidence, plus53 Python tests22.086s. The originally ignored VM File-range test remains NOT_APPLICABLE and is not counted as a pass. Diagnostic original golden replay failed before fixes; provisional two-case green and replay-script NameError before execution remain separately recorded. There is no whole-suite PASS claim.

Actual public `cargo tm --test-name a2r_hir_892_tests --test-name aavm2_compile_corpus --test-name aavm2_compile_use_corpus --test-name aavm2_goldens_check --test-name instruction_boundary_tests --select-only --out <repair-3-selection-final>` exited0. Real47/47 discovery; final catalog7371 obligations/1221 leaves (previous7368/1220). Exact old ID set retained, only two native-boundary Rust controls and one Python preparation control added. New leaf belongs to existing core/stdlib-assembly; first missing-owner refusal was repaired, not bypassed. Public selector expands five complete seed leaves to778 leaves/5626 obligations through consumers/integration; this selection is discovery-only. Work executed the scoped original five seed leaves and justified siblings through nextest isolation, not the778-body/global campaign. Merge owns the final frozen affected closure/required slow modules and full four-owner combination.

Public catalog source identity `5a02e7b8ce6c89e5d1b8b5f18e4839406d9e3ad383eb485f3e1da11b3d5ee9aa`; direct inspection identity `983c36ac26c1f0bf458a54438950ec40c94ce850ab0bf96a9bce4c824a582198`. Effective Cargo-launch environment/PATH hashes differ; all five repository byte-input digests were independently checked equal before publishing the exact real public runtime catalog. Keep both provenance records, do not relabel one as the other. Raw baseline/index includes46 files6352734bytes plus the real final discovery receipt, exact hashes and five clean HEAD readbacks.

Health/scope audit: modified Core files rustfmt checked, changed diff whitespace clean; no added debug prints/TODO workarounds. Complete previous build+nextest warnings and current discovery contain288 distinct warning messages each, zero added categories; existing warnings remain explicit. Global overview placement corrected under approved SD01/04/08. Separate legacy ABT/operand_size format inconsistencies discovered during sibling inspection are recorded as outside-path debt candidates, not claimed repaired or used to defer native-closure acceptance.

Work-owned AC01..12 retain prior independently verified owner/host/generator/SDK/UI/Man/Blog support boundaries where relevant source is unchanged; AC08/12 current tool/decoder/catalog findings are repaired above. R1/R2 behavior code and original source rules/floors/ignore conditions remain unchanged, but no prior artifact receipt certifies a changed current SDK source fingerprint. AC13 awaits affected independent review. AC14 remains NOT_PASSED: final complete affected module union, actual four-owner legs, current SDK217/2/distribution45/history14 (or exact-input valid evidence), actual per-repository landings, canonical/explicit ledger/index/report readbacks, actual archive/link check, fresh guards/cleanup, landing artifact observations and due main periodic batch remain T-12.

Open in-contract Work findings: none. Unresolved semantic/authorization questions: none. Remaining coverage gaps are explicitly review/merge-owned above, not a work-owned deferral. Two prior repair cycles are completed; this is cycle3/3, no silent fourth cycle. Prior final review attempt3 PASS remains historical; new frozen inputs require affected independent review, not reuse of that PASS as current readiness.

Evidence: repair-3-diagnosis.md (0305691854ef1b678f288e30f37b7d055250a79d53e9ba3fc27e1fb8ef77397d), repair-3-spec-delta.md, repair-3-final-baseline.json, repair-3-targeted/, repair-3-aavm-consumer-final/, repair-3-native-reference/, repair-3-python.log, repair-3-selection-final/ and actual catalog publication receipt. Raw attachments remain local; only concise deliberate Markdown summaries are committed. Current attachment scope211.11MiB observation is not a global budget/retention certification. User14-day policy remains authoritative; final closeout must reconcile the conflicting installed helper safely.


</details>

<a id="e890-f45a42b3a0d8b00b"></a>

<details>
<summary>work\repair-3-spec-delta.md · SHA256 a77d62978dfe65230a12e993adcc20c3688cc81f1925924db33191fe8acd816c</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


##### r6 repair clarification (R1/R2; same existing contract)

SD-02/07/08: the final four-owner consumer requires the full explicit registered
recipe/module union, source/dependency/effective-configuration identity and
original conditional applicability. Required owner cwd, list/run command,
listing/output path and hash, selected IDs and actual per-ID results must agree.
A self-reported success banner/count cannot replace execution proof. Public tm
may refresh into an ephemeral catalog; its source contracts are revalidated,
not weakened or rejected merely because published metadata bytes differ.
Original negative Case oracles can legitimately produce a raw exit1/FAIL while
their explicit meta-oracle passes; those source/oracle identities remain checked.

Distribution command-driven checks require their actual command/expected and
actual exit/output identity. Runtime/detail and exact-input SDK reuse retain
their own explicit schemas; they are not excuses to omit a required command.
Current-owner packaged SDK/crate/installer inputs are reconciled with the
distribution's frozen source cohort at the public combined consumer. A valid
historical receipt does not certify changed SDK bytes. Both periodic manifest
consumers keep every current registered recipe; responsibility/purpose advice
never changes original registration, conditions, samples, budgets or floors.

SD-03/05/08: Core's file entry (`run_file`/`run_file_with_args` through
`script_entry::run_with_path`) captures the neutral invocation launch before
spawning its execution thread and rebinds it on that worker, like the existing
run bridge. An explicit absent launch remains absent; ordinary no-service files
need no Web provider. Web `run --config` consumes the declared loopback/address,
port, profile/config identity and policy even when legacy environment values
differ. Existing no-config legacy behavior remains. No new protocol, network
product, platform promise, package or dependency edge is introduced.

Evidence remains source-bound and scoped: current rebuilt SDK217/2, fresh and
relocated CLI configuration/Host controls, original unaffected generator/Man/UI
proofs, and explicit final current-cohort release/four-owner gates pending under
T-12. Do not deposit a new aggregate PASS before that frozen combination actually
executes. The known first failures and original manual/ignored conditions remain.

##### Repair cycle 2 clarification (r6 unchanged contract)

SD-02 / SD-07 / AC-08/12: source-bound full-owner receipt admission and the execution producer share the same baseline/migration/retirement/frozen-floor selection guard. Matching list/run/IDs/count/hash alone cannot remove an original required ID or lower an owner below its original floor. Consumer migration credit comes from original required-source mappings and is legal only when the exact target IDs actually passed in the already validated complete module union, not merely appeared in the catalog/selection or a receipt count. Language/UI source baselines and Man/UI nextest/libtest floors preserve their original applicability and eligibility. No floor, original ignore, condition, sample or milestone is relaxed.

Implementation: scripts/run-full-regression.py::validate_owner_selection is shared by primary and extra execution legs and scripts/test-governance/web_delivery.py::validate_four_receipt. Required original UI/language IDs and Man/UI libtest floor controls include internally consistent list/output/count/hash mutations. The public fixture verifies failure propagation. This is the original r6 proof contract; no new cryptographic/log authenticity, performance, platform or product promise.

SD-03/05 and all unaffected owner, lifecycle, generator, artifact and Blog clauses retain the repair1 proposal and their explicit bounded evidence. SD-08 publication remains after independent acceptance and final required current-cohort/module/four-owner gates. Current canonical documents and live ledger/index remain unwritten at work stage.

UI single-process applicability detail: ui-tu keeps the original cargo-test single-process command. Because libtest --list omits ignore flags, its original ignored identities are reconciled against the already proved same-source ui-full nextest listing and recorded as condition_excluded. Exact selected IDs and actual passed output exclude those identities; arbitrary self-reported exclusions reject. The existing ui-tu floor stays863, and the genuine current UI866-pass/two-original-ignore result remains consistent. Ignored bodies are not counted as executed or passed.

##### Repair cycle 3 addendum (r6, existing AC/task scope)

Keep SD01..09 and repair2 acceptance unchanged. Canonical source preparation already exists in the four authoritative worktrees from the interrupted merge stage; it has not landed. This repair clarifies SD07/08 in existing governance-and-inventory.md: managed a2x worker preparation/source identity and explicit path; no workload leak exceptions; existing AAVM guest runtime shim forwards to its prior implementation under the emitted public namespace; owned guest target/cache; real CALL_CLOSURE/future instruction boundaries preserve strict native reference verification. No generic transpiler change, new runtime API, provider capability, dependency edge, test eligibility, baseline/floor, golden, sample, milestone or platform promise.

Core disassembler consumes the existing engine's one-byte closure argc; reference closure uses the same decoder for four-byte future names. Two permanent controls prove both valid acceptance and invalid-successor rejection. Original golden reference error now identifies the case and invalid byte. This repairs existing verification behavior rather than adding a new public Spec component. Canonical global overview places independent Web outside workspace crates and adds the restricted existing auto-generation-core map entry (SD01/04/08 navigation only). Final gate/ledger report pointers stay pending until actual merge-owned execution and landing readback.


</details>

<a id="e890-12e120acc8ca8d27"></a>

<details>
<summary>work\repair-final-gate-r7.md · SHA256 80b619ccd1f17feca66f6ce3c08fb7c70c925972abbad42a42f1c60ef6cf3d34</summary>

#### PLAN-890 r7 first frozen-gate repair

Work handoff, not final delivery. Current frozen language4992de5fbcf01725673d15c03b34e9690442f291; Webe144ec0d9febe74f8cf734744d91ade47fe57bcf; UI4da2201fa46bd8a88bc5c2d661b21433b7ac84b5; Man43a8f1e6e2314ae334edab38d4dea2c3dc8e4db6; Downc3464d0. Rebase onto master3fa75dd9c was conflict-free; Git implementation diff from7ad103ad2 is empty, all five raw source_sha256 values before/after match. See integration-896/repair-gate-1/{precommit-inputs,frozen-inputs,rebase-range-diff}.

##### Failure and mechanism

The first explicit six-group gate on93dd ran695.533s and failed nine original atoms (8 integration,1 Tauri fixture) plus strict vctip descendant accounting. Other four groups actually passed; no external owner leg started. Root execution49 wrapper invocations/436 child comparisons were valid. Original failures and all early development negatives remain local; no result is relabeled.

- ZIP: include zip_io.rs in the existing Vm provider source closure; regenerate the original872 binding table through its exact compiled generator. The only table diff is native_id_entries879→882 for897's three primitives. No golden or comparison was changed.
- HTTP fixture cache: four known724/727 C-temp projects had stale successful Cargo markers but missing libsqlite3-sys/out/bindgen.rs. Filesystem timestamps identify the missing output and prior build, not its remover. Verify exact Temp/path/manifest ownership and clean only their sys package. Keep source and first failures. Both helpers now use the existing owned process executor and own target, closing compiler descendants rather than waiving workload meter errors. Root facade adds only a test dependency on the existing tool executor.
-018: current corpus712 names the path argument id; the real TS generator interpolates it into /books/1/chapters. The stale test request/comment used the former book_id/template wire. Change only the request to actual supported client wire, retaining status/entity assertions.
-031: keep the original Vec<String>/list/iterator/RC ABI. Decode legacy ListData<i32> sentinels only under an immutable canonical string-list declaration. Typed Value::Int remains an integer: under []str it is rejected; []int/unknown stays numeric. Str/CStr typed lists, empty/Unicode/escapes/duplicates, canonical Future wrapping, read-only pool/RC state and unrelated record serialization are controlled. No negative-value/string-pool heuristic for ordinary ints. UI records per-session unique_name separately from diagnostic Type Display and reports a typed marshalling failure as500 after task cleanup. Default Web replies, ServerHandle and LegacyStdnet use their active owner metadata; middleware never borrows the handler declaration.
- Tauri: emit the existing fixture's compact handler registration, preserving order/all handlers and adding empty/multiple controls. The existing byte equality and actual mock dispatcher assertions remain unchanged; no fixture/golden edit.
- Receipt consumption: preparation may admit only the existing path+SHA pinned compiler auxiliary after confirmed reap/remaining0. Recompute this policy, never trust accepted=true; workload errors remain fatal. New12-control suite rejects altered pin, live descendants, forged credit and missing source/case obligations.

##### Actual scoped evidence

Raw prefix: docs/plans/attachments/890/work/integration-896/repair-gate-1/.

| Scope | Actual result | results.json SHA256 |
|---|---|---|
| frozen-typed | [('PASS', 1)] | `fb9bd2fea03b45f207259e91bd5d7fc4b677cd7808ea7f532dcc6fec5e228f23` |
| frozen-proxy31 | [('PASS', 1)] | `3c833f324cf9bf644476787c6ba8a825a19a55430e0012779f8c7c61da3fd197` |
| frozen-pins | [('PASS', 12)] | `c1754d6ef4bd70336649c02723c72ab088b2509c57ce4e8179bcaa16d74f0166` |
| back-proxy-routes | [('PASS', 1)] | `e4bf78f9a624bff7ab11ff71035d8d54b64fa9ca0473aba389b06ae00d5e06e5` |
| client724 | [('PASS', 3)] | `e33a48cc66b0b135baeec75d9f93a43b618371bd8bcdf898c13cefb8f561c410` |
| transfer727 | [('PASS', 4)] | `01ea76825a1b9954be9370d8620522f19da038b4a0ee2a43e90fb9dd0834da04` |
| binding-table | [('PASS', 1)] | `103e3e65dd080548942db5e714388e65c521a5805749b011cd7832a99e74762d` |
| provider-inputs | [('PASS', 1)] | `63dc841fc8df345337cc3700d64f3dff205b525f61d385270f41c8cbd3d224d0` |
| tauri-registration | [('PASS', 1)] | `57848e40a8548a423ad9699991ba2e9281ccab5db5bfad193df22423c592ad39` |
| tauri-consumer | [('PASS', 3)] | `66e4c017af1248fa73fd356a5f45205c578e33fada31854a5f0c03384250bf31` |

Frozen typed/proxy/pin results bind source_state37485f39bb671bf5e4c7e16789f2111a5a34c038f14671ea6a66a332e6e6252a exactly to4992de. Earlier finding replays retain their actual precommit dirty baseline; relevant helper/generator/corpus inputs stayed byte-identical through commit/rebase. Web runtime-separated executed3/3 clean, with both actual CLI profiles asserting native string lists, numeric negative lists, original host/config/shutdown/release and upload snapshot. Later only explicit typed-Value rejection/pool controls and unrelated record fallback were refined; its final current-cohort unified execution remains mandatory.

Initial new test mistakes are preserved: missing direct test dependency, assuming immediate RC removal rather than the actual dying grace/reap protocol, then querying a freed object through the canary API. Final tests use real quiescent reap and inspect backing-map absence without dereferencing a freed object. An initial exploratory fs.dir_names setup failed native name resolution; it was not an original acceptance oracle and added an unrelated prerequisite. The focused wire test uses the existing Str.split Vec<String> producer, without changing filesystem implementation/enablement. A mixed build+run probe also retained strict vctip124; successful replay separates preparation from workload, never grants a workload exception.

Final independent affected review is still required. After pass: rerun current six language groups/eight native external legs, current20-package SDK tests/distribution/legacy recovery, then all landings/canonical/ledger/index/archive/guards/retention. None is certified by this Work summary.

Current frozen Web execution completed: integration-896/repair-gate-1/frozen-web-runtime/results.json SHA256a5530f08d88185e29a0bf96cc0ac6410cebb28136f592f0e8a167670160bb04f; actual3/3, execution meter.error=null, five-owner before/after identities equal and exact4992/e144/4da/43a/c346. This replaces the earlier bounded Web runtime reuse note with a current-cohort direct holdout, without certifying the future full four-owner gate.

Final source guard descendant54b5d7c92f0244faecde929353b7e292ff6b3579 changes only four_repository.py and its Python controls. Runtime docs/schema inputs are now included both in source snapshots and tracked/untracked clean admission; Plan/report Markdown output remains allowed. The actual Git fixture proves tracked schema drift and untracked schema injection reject while Plan output is accepted. New cargo t web-delivery-controls completed13/13, results SHA25699ced027305796ee0822da11bb336ede719647f470e17ddd7f41155e54f98511, source_state9154eb92b9ba992a2d7e3eeabaea8bb4a209c25d9124511c3874617cbf34941e. No Rust/SDK/lock/config bytes changed from4992; existing scoped runtime evidence retains only its documented input boundary. Unified final gate remains pending.


</details>

<a id="e890-49c991d9b3da6137"></a>

<details>
<summary>work\repair-second-final-gate-r7.md · SHA256 7bf820d7dcf985a17301704f6752ba54b4d0a50a75dd92248ef2683daf18bd61</summary>

#### PLAN-890:r7 second unified gate bounded repair

Work handoff, not final delivery. Lang54b5d7c92/Webe144ec0d/UI4da2201f/Downc3464d0 unchanged; Man d2b2664fea33a93def31ef4cb04be7110c97e807 replaces43a8f1e. Only four Man Rust files and its reviewed ownership document changed; no dependency/lock/config/golden/threshold change.

Original final-896-2 four-owner receipt remains FAIL, SHA2569c2e7ecbab4b3789b498dd251b94ad89147c97c12bb5fd589e4ff3db70a934bd. Six language groups3954 passed; Web221pass/2ignore,UI-full3121pass/18ignore,Man daily281pass/4ignore; Core/tc passed. Man-ti42/43 with stale implementation-source observation plus strict workload vctip; UI-tu/ti never started.

Media: original test ID and all original assertions retained, source observation points to the actual single Web generator. Extra facade delegation assertion preserves the compatibility bridge. No media/generator product code or oracle changed.

Compiler lifetime: original v05 real Cargo+Ninja test asserted PASS but strict meter rejected owned vctip both first compile/run and warm replay. Warm8.954s, not a preparation exception. Both actual production build adapters now use the existing process facility, with original argv/cwd/env/three inherited console streams and raw exit status. Windows owns a Job and explicitly terminates remaining descendants after root wait; cleanup failure propagates. Non-Windows keeps DirectChildOnly, retaining foreground group/Ctrl+C. No process-name kill, workload whitelist, added deadline, test-child wrapper or output-capture substitution. Original build/run/cleanup APIs remain.

New helper controls prove nonzero exit23 and missing-program NotFound. Actual directed native selection: original media3, original v05 real build1, helper2. Preparation and execution separately metered; all6 pass, both meters.error=null. Rust bytes stayed unchanged from this precommit replay to d2b; only ownership documentation was appended before commit. Do not label this a frozen full Man run. Final affected review then all4 frozen Man native legs and previously unexecuted UI-tu/ti are mandatory.

- diagnosis-v05/run.command.json: SHA256 d9f84dfc1a94854f0cd2d7f2a8528825e280030cc733fe1ff470fff558d079a6
- diagnosis-v05/run.stdout: SHA256 c779aca3aab9c15a857d51704c5623d98174bba6575b36582e450dcab170dc97
- diagnosis-v05-warm/run.command.json: SHA256 74e1b60cafa4d9668ab6f4727ee331ebca60e0de4013e2f9b6cc0ed9c6ab3e2e
- diagnosis-v05-warm/run.stdout: SHA256 dbdcc811a91f89ff6c0d04368e376bb5418801823d5aeaef6e233658745cccba
- targeted-owned/run.command.json: SHA256 c67e11c860e179cb01a88f4de1e1774b2eb9087f269de824766248680c3536ef
- targeted-owned/run.stdout: SHA256 e44152dd6be45e33f757529d3dabdc4b3989c39a77e170317ba2292b05c2c91e
- targeted-owned/discovery.command.json: SHA256 a3e76e8e00b760f5bce3b753051b01bf699089ecdfbab6a34de68feaadd9d25b

Proposed recovery: local resume_four_owner.py uses production fixed_proof/native source selection/validate, with exact original failed receipt hash, exact current unchanged owner snapshots, exact five-file Man delta and no Cargo path edge from unchanged Lang/Web/UI to Man. Reuses only actual language/Web/UI rows and their raw identities, reruns every Man leg and missing UI-tu/ti, then fresh source denominators/child comparisons/terminal protocol/clean identities through the public consumer. Original failed aggregate is immutable, reused producer identities and changed scope explicit. Review this script before execution; it is local raw evidence, never staged or a public runner change. SDK20/distribution45/legacy14,landing/canonical/archive/guards/retention remain NOT_PASSED.

Recovery provenance closure: the original actual CommandExecution in this thread is exec-9395695e-51c6-4989-b0e5-c9a46f9c8d27 at2026-10-31T05:05:57.070Z, with the actual final-896-2 command. Replacing only CODEX_TOOL_CALL_ID in a comparison copy of the current primary environment reproduces the original complete sorted environment SHA2568fe46af2c354f86270d4e81e9fd2868f973e6054297b296d19d43327d0ee532c exactly. No secret values are printed; no old identity is passed to new runs. Thus all other environment bytes, including flags/wrapper/profiles/conditions/PATH/roots, match. Local recovery script2a2f4c8825851a1a1205220c1013a9c5b126b7694ffaafad81d48a176b8ca574 checks this before gate/after bootstrap/after admission, plus pinned actual tools and bootstrap file SHA+mtime. recovery-provenance.json302f00202b413fc1837b89df61005fe773d89d2a6d48964051bd6d729302b965 records actual source log reference and versions; global binaries predate original dispatch, all three build caches have the same rustc1.98.0 fingerprint. Original global tool hashes were not individually recorded: chronology/cache/source/env is the explicit unchanged-toolchain inference, not invented historical SHA equality. Two bootstrap binaries are the original pre-Web/UI outputs, not a separately recorded earlier language-phase hash. Actual6 provenance controls passed: current primary accepted, RUSTFLAGS/wrapper/T3/root mutations rejected, changed scheduler nonce accepted while actual environment remains untouched. Control receipt SHA256011bb9f6e6426db90299483403c46ff494964af5e0ac22f0a3635d01715d0cc1. No Rust body rerun for this local provenance adjustment.


</details>

<a id="e890-5ba82b8d9f848b23"></a>

<details>
<summary>work\response-terminal-observation-r2.md · SHA256 68232796eab9da26eb3947e5b21c9206f8743fc4fa6ca3dc981ef08458e21a74</summary>

#### PLAN-890 r2 — response-body observation and lifetime

Round 23, work/T-02, parent `9fc9095520597dd7843d7ecedccaef965eea5f31`.
Prior round was progress. The file event bytes=0 finding is now repaired and
verified; this does not close the whole T-02 gate or deliver Web.

The original file producer finish hook completed the request before buffered
HTTP data was consumed. File/SSE payload polls never accumulated event bytes,
and the actual file decision status did not replace the placeholder 200.
Actual TCP red: seven completed wire responses instead emitted events with
zero bytes and status 200, including 206/304/416/404 wire results. The first
fixture assumption that ETag was automatic was corrected to an explicit
descriptor ETag before recording this valid red.

A framework-owned body guard now counts bytes handed to HTTP, finalizes the
same scope/event exactly once, distinguishes natural completion from early
drop/error, and handles an exact-length consumer dropping after its last frame
without another EOF poll. It does not claim TCP acknowledgement counts. File
producer completion records its outcome; it no longer completes an unread
request. Actual file status and inline error size are installed before scope
completion. SSE uses the same body guard and records cancellation on shutdown
or early drop. No private VM or neutral-port field was widened.

Two further actual checks prevented incomplete repairs:

- Configured file responses leaked legacy wildcard CORS on the network thread.
  The request's existing policy context is now selected per network-future poll,
  covering file assembly and transport rejection paths as well as owner work.
- The intermediate repair left a still-connected stalled body alive after its
  file watchdog fired. That was this round's intermediate regression, not a
  parent-baseline claim. A per-connection watch now aborts that exact connection
  on an active file-stream failure/timeout, dropping its actual body and letting
  the same RAII scope terminal release resources/permit. Preparation errors
  remain inline HTTP responses; a failure racing with stream activation is
  covered. No global connection-abort map or production timeout knob was added.

##### Scoped evidence

Commands run in `D:/autostack/.wt/lang-890/auto-lang`; source/assets/build scratch
stay in that worktree. Raw logs below are local-only under
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Result | Log SHA256 |
|---|---|---|
| initial actual response observation | expected exit1; zero-byte/placeholder-status event multiset differs from seven exact wire results | `response-observation-before.log` `72e7b4ad897890f4979280e251d26c4c784b808df65cfe35112ef240232b43d7` |
| configured no-Origin file oracle | expected exit1; legacy CORS header leak | `response-cors-before.log` `dfb4f82a090390f6514eef17ae8a0e67e2d9be62f542b56964081861a5171522` |
| intermediate stalled-file watchdog oracle | expected exit1; connected unread body lacks terminal within 3s with file idle budget 200ms | `response-watchdog-intermediate.log` `189e783cc11951f9a8ecf504e479485f270064c01368c958b74e6bab63b0de7b` |
| `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin isolation --locked -j 4` | 0; actual external source runtime | `response-observation-build.log` `f4825546ce4dd839b3ddc73750690c6f01e182310a20d3cc5cfbec44c5834e32` |
| `response_observation_probe.py --exe .../isolation.exe` | 0; exact method/route/status/bytes/outcome multiset for seven responses: full binary GET, range206, HEAD200, conditional304, range416, missing404, ordered finite SSE; file/SSE RST each canceled once with positive bytes, file count strictly less than unread full length; source truncation canceled once; connected stalled file watchdog canceled once before client close; second VM counters unchanged; all owners drain/registries0 | `response-observation-after.log` `5286114dce871155fd263acdfd2e77f092e275437e7a81afdb7c292709299798` |
| `cargo test -p auto-lang-core file_producer_eof_keeps_request_permit_until_body_terminal --lib --locked -j 4 -- --exact vm::ffi::http_transport::bridge_tests::file_producer_eof_keeps_request_permit_until_body_terminal --test-threads=1` | 0, 1pass/0ignored; actual file pump fully finishes while body unread, actual request semaphore stays occupied; last frame read then drop without EOF releases once with exact counters | `response-body-lifetime-test.log` `1c0afb490d94856067e2c7902cf758c021fea3c813c7cdac82e893629f3194b2` |
| original `full_queue_rejects_with_503` exact Core test | 0, 1pass/0ignored; added internal connection-control argument defaults None, original assertions unchanged | `response-full-queue-test.log` `3bc2ecc77bd026b4fdd6a531928eab54ac0e8ae8f2567e1e14a98bdf7412fa2f` |
| `cargo check -p auto-lang --locked -j 4` | 0 | `response-observation-root-check.log` `9e9f13b7c745cf1c3ab3fc9b06a602dc718e227cfd12401c8809f2daa9f335a1` |
| `service_state_probe.py --exe .../isolation.exe --file-during-drain` | 0, full exact16MiB file across shutdown, configured identity/counters/CORS/control isolation and both port rebinds | `response-observation-drain.log` `1f5c09517dfc585d9acf50f3bf44dd4d9420c3ca39c9aa37842df74cd135fe10` |
| `policy_probe.py --exe .../isolation.exe --plain-exe .../web-port-probe.exe` | 0; new network policy context preserves legacy exact environment CORS and native rate isolation; plain source uses unchanged previously built ordinary-program bin | `response-observation-legacy-policy.log` `e521509adcf5d88bf091d68454a3ffeff19f474c572c156b710eb7c40a143370` |

Final post-check Rust edit only corrects a comment. Touched-file rustfmt,
Python compilation and diff checks pass. Core140/external17/root3 baseline
warnings remain, not global health clearance. No whole tier/docs_gen/GUI,
shared registration/runner/lock changes, agent/message, push or merge window.
888 head 643f2b375 has no transport diff from the common base; UI7b323bc and
Man e6cc06f remain clean and unchanged. Raw files were not staged.

Remaining T-02: compiler producer ownership before activation/concurrent
compilation (including async-return metadata), configured launch handoff and
per-instance failure/logger termination, full actual consumer inventory and
physical package/API/command/SD/four-repository schema freeze. Ordinary parked
reply/async-return observation still needs its actual-path check; no general
all-response claim is inferred from this file/SSE probe. T-05 must remove the
Core framework scaffold, T-08 must migrate original IDs without reducing their
floor; these new tests do not count as execution of those unrun original IDs.
Web has not been created; Plan remains executing1/12. Final independent review,
merge-owned gates/deposit/archive/guard remain pending; UI/Man structural
successors still await full U890.


</details>

<a id="e890-1d38a6122d17860e"></a>

<details>
<summary>work\sdk-default-decision.md · SHA256 de596e66f9a0689be1552feefdeed7b7d72075f1e7c353571dfb35767dee36be</summary>

#### PLAN-890: default HTTP compatibility decision

2026-10-30, r1. Concrete pre-migration decision; no production code changed.

##### Observed supported consumer

`test/fixtures/modularity/web-port-probe` depends only on `auto-lang-core` and
calls `auto_lang_core::run_with_capture` on an API route. It registers no
provider and links no Web, UI, Man or Tauri package. Build and real TCP run
both returned exit 0 on Windows. The exact per-invocation JSON body was
returned with HTTP 200; the actual shutdown sender delivered true and the
SDK returned successfully after the VM owner loop exited.

Commands and local evidence (not committed):

- `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --locked -j 4`: exit 0. `core-probe-build.log`, SHA256 `c9c2df223e4f190cf22358bb5991df06c624ee0094a6b886faa8ef2b7ea29e19`. Core reports 147 existing warnings on unchanged production source; the new probe reports no compiler warnings.
- `python test/fixtures/modularity/web-port-probe/probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe`: exit 0, exact_body true, shutdown_delivered true, child exit 0. `core-probe-run.log`, SHA256 `86a761417c8dea798b1c368b2808d434ebb5dc8486bdfc96838af3e4577228a2`.
- `cargo tree --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --locked --edges normal,build,dev --prefix none`: exit 0, captured `core-consumer-tree.txt`, SHA256 `fa9478a3806870627fedb5aaccbf6cb1fe56ab2023d31e123ad6603c003f64e8`. No Web/UI/Man/Tauri package is in this closure.
- Fixture rustfmt and `git diff --check`: exit 0.

Actual path: `entry.rs::run_with_capture` → execution-engine capture →
dedicated VM thread / LocalSet → `lib.rs` after route registration → direct
`http_server::serve_async` / `serve_async_with` → `http_transport` network
thread and owner queue. The implementation is compiled inside Core today.

##### Why the proposed dependency boundary needs a decision

Under the proposed static package DAG, a program whose only dependency is
Core cannot execute an implementation that has been wholly moved into an
unlinked Web package. Registering Web from Man/UI preserves those composed
applications but does not preserve this independent SDK consumer. An
optional dependency with a default Web feature remains a Core→Web
implementation edge, expressly forbidden by r1. Keeping a second default
implementation would also violate the migration contract. Dynamic loading,
implicit external tools or hidden binary payloads would add distribution and
compatibility obligations, not prove the current consumer contract preserved.

This is the concrete AC-02/03 and D-04 compatibility question. It is not a
failure of the JSON baseline or a claim that the owned-body port is already
proven infeasible. No all-path lifecycle feasibility pass is claimed.

##### Reviewable alternatives

1. **Explicit version migration.** Keep Core free of Web implementation
   dependencies. Web exposes the service-host adapter, and Man/UI/other
   service hosts install it before execution. Ordinary language calls retain
   their behavior. Direct SDK consumers that execute API routes must add the
   Web dependency and explicit assembly in the new version; missing assembly
   must return a clear error rather than silently run without serving. This
   requires an explicit approved compatibility change, migration guidance and
   external-consumer regression tests. All current known consumers must be
   inventoried and moved; no unspecified old-path fallback remains.
2. **Compatibility facade with a pure lower language owner.** Keep the old
   Core API's default HTTP behavior in a facade that delegates to Web; move
   the one language implementation beneath both into a distinct package.
   Web depends on that lower language owner, not the facade. This requires
   revising r1's explicit Core dependency rule and package/source distribution
   design. Type identity must be re-exported from one lower owner; no AST/VM
   duplication is allowed. It is a larger package migration, not a free
   compatibility trick.

Historical decision state at r1: a user-input question was submitted with these
choices; its preselected state was not approval.

2026-10-30 update: the user explicitly selected **显式安装 Web adapter**. PLAN-890
r2 records alternative 1 as the current contract. The compatibility decision is
closed; no renewed approval is required. Prototype wiring and remaining
feasibility checks proceed in the same authoritative tree. The original baseline
above remains before-evidence, not a claim that the old implicit default is
preserved or that the candidate Web implementation is already delivered.


</details>

<a id="e890-a978158ca2cf6df4"></a>

<details>
<summary>work\shared-generation-seam-r2.md · SHA256 d465085e0e2515a3e9aa8f066c85ac08ba4267252e93a6b6d1e2f776a9ba73f7</summary>

#### PLAN-890 r2 — shared generation and neutral media feasibility

Status: T-02 partial, not execution_done. T-01 remains 1/12; no Web repository
created and no framework mass move. Source revision before this unit:
`c8be230f011896901e737dfbf9ffa6a6b5d360e1`.

##### Single source and producer boundaries

- Moved the exact FNV-1a algorithm into `source_owner`; `api_gen::fnv1a`
  re-exports it. Shared workspace lock identity no longer depends back on the
  mixed API generator. Empty, absent and unreadable lock distinctions remain.
- Added explicit service/UI requirements to shared workspace generation.
  The original public entry continues requesting UI. The new service entry
  avoids UI source discovery and UI/iced dependencies. Existing members with
  direct or renamed target-specific UI dependencies retain the UI requirement;
  missing selected sources fail before replacing the old manifest.
- The three original shared modules compile together with no `api_gen` module,
  UI, Man or Tauri in the dependency closure. Candidate later carrier remains
  the bounded `auto-service-gen-core`, not the whole generator or a copied
  SidecarDecl. Actual package migration is still pending T-02's full gate.
- Newly discovered transitive producer: generated photo handlers refer to
  UI's backend-neutral photo service. The unchanged source at UI revision
  `d53f946ab89ce6bb3cbbb2ea929f094ff2600be6` compiles with only image/EXIF/blake3,
  whose families already exist in `auto-media-core`. Proposed later single
  owner: media foundation, UI re-export, Web generated handler uses that owner.
  No implementation copy or source move occurred. Source SHA256:
  `738751c6bfa88c9ea5c313d620d008528f7e64a9e4ab28fa78ed0bdcef12558f`.
  Append this to final physical migration mapping; do not regenerate a smaller
  T-01 baseline or count it as completed migration.

##### Checks

All logs local-only under
`D:/autostack/auto-lang/docs/plans/attachments/890/work/`.

| Check | Actual result | Log SHA256 |
|---|---|---|
| Initial shared fixture, original three modules + lock control | exit 0; 16 passed, 0 ignored; preceding workspace-mode change | shared-generation-probe.log `488cc466d25f52b80a0890164e247171f6354950aec11d75895d232d333057ee` |
| Shared fixture `cargo test ... workspace -- --test-threads=1` after workspace change | exit 0; 5 passed, 12 filtered; new service/UI isolation control plus four original workspace cases | service-workspace-probe.log `bc19db2a9f13386b6c29b717e780ab6091a466b7d1eac5c65c4e6d76d72e6a33` |
| Photo fixture `cargo test ... photo_service::tests:: -- --test-threads=1` | exit 0; original 10 passed, 0 ignored; includes traversal/token/root/cache/thumbnail controls | photo-neutral-probe.log `d9f91804d55e1cfacc8c7d1b894cc61ddd26979e0288e4ed0b723eadf9c8d9e5` |
| Shared fixture `cargo tree --locked --prefix none` | exit 0; no auto-api-gen/auto-ui/auto-man/tauri package | shared-generation-tree.log `cd71632b29d9bbe30fe9fd03fad69922596b039389adbdb1e9464d3e551c110e` |
| `cargo check -p auto-api-gen --lib -j 4` on final code | exit 0; existing Core 142 and generator 3 warnings, no new warning | shared-generation-api-check-final.log `8acc54b81d4555ee51cc40f5a32993eb514ae676d397fb611b7a7b96b9a8e736` |

Photo test child TEMP/TMP were overridden in try/finally to private
`.../work/photo-probe-temp`; both parent values restored. UI tree remains clean.
No GUI, shared cache cleanup, aggregate tiers, shared registration/scripts/locks,
merge, archive or push. Required full runtime isolation/owned port/lifecycle
proof and frozen APIs/schema still remain before T-02 can pass.


</details>

<a id="e890-03d1ec7a02eaf7e5"></a>

<details>
<summary>work\shared-language-generation-r2.md · SHA256 21986b9f1500496b86085be5712f67cca67e3ab47d39bb60177d539fb537c592</summary>

#### PLAN-890 r2 — restricted shared language generation seam

Bounded T-02 progress, executing 1/12. Parent language revision
`d861642a59e7f03d37b49ffbd13179f311fd14e8`. No Web repository or physical new
shared crate has been created; full T-02 still gates that migration.

##### Single-source cut and proposed physical owner

`crates/auto-api-gen/src/shared_generation.rs` now owns the 25 functions below.
The mixed `api_gen` module re-exports its existing public paths and imports its
private implementation helpers. Existing tests retain original names/IDs and
bodies. Public proof-returning transpilation exists on the restricted module
for the future cross-package caller; its old mixed-module path stays private.
No Core private fields/modules were exposed. These are actual one-source
implementations compiled by production callers and the external source probe,
not copied probe algorithms or a second SidecarDecl.

| Functions (all originals retained) | Responsibility / consumers |
|---|---|
| `bind_workspace_lock`, `lock_freshness`, `assembly_freshness` | Receipt update and conservative identity comparisons; HTTP generation/reuse and UI Rust workspace consumers |
| `try_full_parse`, `extract_api_lenient`, internal `parse_fields`, `parse_params`, `primary_type_name_pub` | Existing language API/type extraction shared by HTTP, IPC and Rust UI; no network runtime or IPC assembly |
| `transpile_db_to_rs`, `transpile_back_module_to_rs`, `transpile_back_module_to_rs_with_proofs` | Existing public language transpiler / 16 MiB parse thread, source_dir import signatures and owned reference-proof output; HTTP companions, UI Rust companions and db |
| `qualify_a2r_std`, internal `replace_in_code_regions`, `code_region_mask_text` | Existing embedded runtime qualification and code-region scanning, preserving strings/comments/custom module boundaries |
| `post_process_companion_rs`, `promote_api_types_in_companion`, `post_process_db_rs` | Single postprocessing/type-identity mechanism; 885 imported signature and canonical type repairs preserved |
| internal `append_clone_for_borrowed_fields`, `add_mut_to_let_collections`, `append_tostring_for_str_fields`, `strip_collection_new`, `utf8_len`, `balance_paren`, `fix_borrowed_slice_returns` | Existing supporting rewrites, private to the mechanism |

A mechanical source-conservation check reconstructs the exact slices from the
parent Git object, applies only the named export visibility changes and rustfmt,
and compares the entire resulting module bytes. All 25 function bodies match;
canonical module SHA256 `bfffc557d23f8cfa47680a9606b472575e4bfdd108b675a4ba6627158f8c8a91`.
The previously dead `replace_in_code_regions` helper remains private with its
original allow/comment; no unapproved history pruning is mixed into this cut.
No mutable module static/thread-local was introduced.

Selected carrier proposal for the final T-02 freeze: language-side
`auto-generation-core` 0.1.0, restricted to `shared_generation`, `rust_workspace`,
`sidecar`, `source_owner`. This name reflects its existing HTTP and non-HTTP
consumers. Normal/build closure is the current language SDK / project-model /
value support plus existing regex/toml/semver/sha2/serde_json utilities; necessary
dev closure uses tempfile. No dependency on Web, mixed auto-api-gen, UI, Man or
Tauri. Existing project-model remains the SidecarDecl owner. Source-owner and
workspace modules reference one another by local names and never call back to
`crate::api_gen`. Physical relocation and SDK source-list/delivery-resource
registration belong to T-04 after the full T-02 pass, not this probe.

Future direction: Web generation -> auto-generation-core -> language SDK;
retained auto-api-gen facade/non-HTTP assembly -> auto-generation-core and
(for its HTTP compatibility calls) Web. Web must never depend back on the facade.
The actual mixed generator still exists pending T-06. Its backend freshness
calculation (depends on generated HTTP assembly/source snapshots) remains Web
owned; only pure lock/identity helpers move to the restricted mechanism.
HTTP routes/client/server/media/photo templates stay outside this carrier.
Tauri IPC command assembly stays outside Web. Those remaining cuts are not
certified by this narrower source check.

##### Actual verification

- Shared source probe compiles the actual four modules by source path, with no
  auto-api-gen dependency. After adding direct regex use its local ignored lock
  was updated offline, then all subsequent consumer build/test/metadata commands
  used `--locked`. Full Cargo metadata contains 480 resolved packages, zero
  auto-api-gen/auto-ui*/auto-man*/tauri* packages, including available normal,
  build and dev package graph. This is dependency evidence, not cross-OS runtime
  certification. Metadata initially offline missed target-specific cached crates;
  normal metadata downloaded those locked entries, without version changes.
- `cargo test --manifest-path test/fixtures/modularity/web-shared-generation-probe/Cargo.toml --target-dir test/fixtures/modularity/web-port-probe/target --locked -j 4 -- --test-threads=1`:
  17 existing unit cases passed, zero ignored. This ran before the new integration
  target was added; unchanged unit inputs/results are reused, not counted again.
- The explicit new `--test shared_companion -- --test-threads=1 --nocapture`
  command passes 1/1. Actual source_dir-aware language transpilation generates
  two modules, postprocessing/public type promotion feeds an external Cargo
  consumer with the real Core dependency. It compiles/runs exact per-invocation
  environment value + `/leaf`; removing only promotion gives actual E0308
  `types::Work` vs `works::Work`; removing only source_dir gives actual E0308
  in repository.rs, expected &str / found String. Restoring the exact original
  generated sources then compiles/runs again. No hand-edit of generated Rust,
  fake runtime, old binary invocation or UI dependency. Uses explicit shared
  target for this bounded check; fresh-target/relocation delivery remains T-09.
  Initial fixture-only assertion that Env lowering must produce a nonempty
  reference-proof vector was removed: that source legitimately lowers without
  a tracked assembly reference. No proof completeness credit is claimed here.
- Original auto-api-gen tests retained and run scoped: qualifier 1 pass;
  `test_extract_api_lenient` 4 pass; `test_parse_` 3 pass; `test_handler_body` 3
  pass (including actual compile/run string/borrow/type cases); `plan734` mock
  dispatcher 1 pass, fixture byte comparison 1 known parent failure below.
  Total distinct original IDs: 12 pass, 1 parent-reproduced failure, none ignored.
- `cargo check -p auto-api-gen -j 4`: exit 0, 3 existing unused-variable warnings;
  `cargo check -p auto-lang -j 4`: exit 0, root 3 existing warnings. Core retains
  141 baseline warnings. No new warning is claimed resolved globally.
- Isolated changed/new Rust sources pass rustfmt check; git diff check exit 0.
  No root whole tier, docs_gen, GUI, merge or push.

##### Non-HTTP fixture baseline and same-lifetime Man input

The original `test_plan734_tauri_fixture_matches_generator` first could not
read its hard-coded sibling fixture because this group had no Man checkout.
Man main is clean and remains exactly
`e6cc06fbc496f366edde7929abfd49254ec13647`; it has no root/nested AGENTS.md.
Created its one same-lifetime **read-only detached** tree at
`D:/autostack/.wt/lang-890/auto-man` from that exact commit. No product change,
branch, symlink/junction or copied substitute fixture. Its future Web integration
edits will use this same tree and only then attach its Plan branch; it must be
guarded/cleaned with the rest at final delivery. This is not a UI/Man structural
successor launch or authorization.

With the real fixture present, comparison fails solely because emitted
`generate_handler![` is multiline while committed Man fixture is rustfmt's
single line `generate_handler![echo,]`; preceding field/command/registration
content matches. The Tauri mock dispatcher successfully validates ok=true,
n=42, tag=hi. To attribute the byte drift, temporarily restored **both** actual
API generator and lib module files from parent d861642a5, reran that exact
original test, and restored candidate bytes in a finally block with byte-equality
assertions. Parent also fails the same generator-drift assertion. Its replay
log is separate and hashed. Candidate tests/fixture/generator output were not
normalized, weakened, ignored or rebaselined. This original failure remains a
visible qualified baseline, not a green non-HTTP fixture claim. It stays in
T-08/final review accounting; moving generic functions does not erase its ID.

##### Local evidence and remaining contract

All raw files below stay local under the main checkout's
`docs/plans/attachments/890/work`; only this Markdown summary is staged.

| Local evidence | SHA256 |
|---|---|
| `shared-language-helpers-check.log` | `b22a261b99c5faabc1c9992f4f8a06a3ab3b9e008ed9f68c3a7bc2118dc91a98` |
| `shared-language-facade-check.log` | `7a2d7c19f7dcfd815d65b873c52e0c7b6505f9f3b2a228a6fe012336018e2396` |
| `shared-language-root-check.log` | `a38a39ca6bfd8dae4cfe3985c3fcf0a91e6b649209ea608aa583bc0a7e093c2e` |
| `shared-language-source-tests.log` | `76cc2a4f3a5c13e3dcc1174b79dd342d81b3bd4523bdc3310be8decbbb7a77ba` |
| `shared-language-compile-controls.log` | `3b39fb74a06f7b959d31cbc834446e047e80129b712c7f227711ad54d2f530e7` |
| `shared-language-original-qualifier.log` | `5779c184a54b1dfa51ec9fff042e1dccf9b6b06c83eabdd1bfa84bf801e068e4` |
| `shared-language-original-test_extract_api_lenient.log` | `f918a945214db00b3e0fa1fb9dc1af474fec1df1244812da06c0376c18405927` |
| `shared-language-original-test_parse_.log` | `a2102c0a45e72562d9c3ffbe50d00ea45949ed0c17c4e277cbef92c8faf21c73` |
| `shared-language-original-test_handler_body.log` | `27ff5b886257bc5735e334a55c6df243775f8311aff5c2033f1214bd74645127` |
| `shared-language-original-plan734.log` | `85d3c6d58b5550caac9b580a6385726532a7b0fbfbc7477a135c6bc14f9b571a` |
| `shared-language-plan734-parent-replay.log` | `35d16087a7fb7b5b8a94b4d6018cb977921c99285f78bf64a9c863e705bfce7c` |
| `shared-language-conservation.json` | `87eb53596fdac2c7ecf2b8086b978f170a1820e91e8171d9ccd6414f81787036` |
| `shared-language-metadata.json` | `5a2a08d0f7e3847de7eddd3be57be49b3047bdda6cb0726ff6b4db7973bc5595` |

T-02 still needs the remaining actual listener/legacy/control/back-proxy paths,
producer/deadline state, full VM lifecycle controls, actual HTTP generation
standalone closure, physical service package/API/commands/SD/four-repository
receipt schema and full port/generation freeze. This carrier proposal is not
permission to skip those gates or create Web early. T-04/06 must wire the
service-only workspace, move renderer-neutral photo implementation once, retire
all old Web originals, and prove all real consumers. T-08 must account every
original source item/test/atomic obligation against the immutable intake.

888 was read at eb80a4b89, with only its own new diagnosis Markdown untracked.
No shared runner/script/registration/lock or final integration window was taken;
no message/agent was created. UI/Man structural successors still wait for U890.
T-11 independent review and T-12 frozen gates/land/spec/archive/guard pending.


</details>

<a id="e890-ab8af97a394bf500"></a>

<details>
<summary>work\source-and-window.md · SHA256 92de20a1b78c43397cb0a03521be74e7d8e90db70583887723ce756c697e713a</summary>

#### PLAN-890 r1: source and write-window intake

2026-10-30. Work in progress; this is not a completed T-01 or feasibility pass.

##### Authority and identity

- User authorized complete delivery and confirmed `885已经收尾了。` after the historical guard-output gap was reported. This closes that start blocker by user confirmation; no guard output is reconstructed and PLAN-885's historical receipt is untouched.
- Language authoritative tree: `D:/autostack/.wt/lang-890/auto-lang`, branch `plan-890-dev`, base `b3f43a7c4a392a6652b70c9b79db5cedb5589047`.
- Read-only AutoDown sibling: `D:/autostack/.wt/lang-890/auto-down`, detached `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`.
- Creation used `new-wt-group.sh`; its WSL Git registration required Windows `git worktree repair` for these two new trees. Absolute root and branch were then read back correctly, with clean status. No links were created.
- U884 `fa0282c85`, U885 `dacd00184`, U887 `dd916c184` are ancestors of the accepted language baseline (`merge-base --is-ancestor`, each exit 0).
- Observed UI: `f7c83c6fbe55b51b4c1e830a4175c94ec5c0d131`; Man: `e6cc06fbc496f366edde7929abfd49254ec13647`; Blog: `7f9b675587509c7b22c3a2f02088ea4d8a47d4ba`. These are observations, not the final candidate pins. UI has unrelated documentation WIP; Man status was clean. No top-level AGENTS.md exists in either UI or Man at inspection time.
- `D:/autostack/auto-web` does not exist. No remote identity was inferred, queried, or created. Do not create Web until T-02 passes.
- UI PLAN-014 exists as a drafting successor; no implementation is accepted under 890's authorization for that successor. UI/Man structural work continues to wait for U890.

##### Source responsibilities and consumers

| Source / state | Intended owner or constraint | Actual consumers / closing work |
|---|---|---|
| Core `http_service_config.rs` pure validated configuration vs ServiceRuntime/control TLS | Split pure contract from framework policy/runtime; do not blindly move whole file | Man service configuration, VM and generated Rust tracks; preserve strict resolution and effective hash |
| Core `http_service_observability.rs` | Web service observability, bounded sink | Core server/transport and generated service; preserve health/request accounting |
| `http_transport.rs` | Web network runtime | Separate network thread, bounded owner request queue, reply, drain; no VM reference sent to network |
| `http_server.rs` and `server_route/argbind/dispatch/scope` | Separate Web routing/policy from genuine language marshaling/VM execution | API metadata publication from codegen/stdlib; VM owner LocalSet, handler task/scope lifecycle |
| `server_model.rs` | Cannot export the current structure as a neutral DTO | `ApiRequest.raw_upload_body: Option<axum::body::Body>`; request body needs a real owned stream boundary and terminal ownership |
| `http_upload/http_server_file/http_return_resource/http_stream/http_transfer` | Split Web transport/service from language resource IDs, value conversion and genuine stdlib behavior | receive authorization, stream/file reply, permits, cancel/disconnect/drain; each path must retain exactly-once terminal cleanup |
| `stdlib/http_client.rs`, `stdlib/http_async.rs`, `async_http.rs`, ordinary HTTP/JSON/socket primitives | Language unless a demonstrated framework-specific portion belongs to Web | General language consumers remain; no whole HTTP-family mechanical migration |
| `lib.rs` automatic server path; `entry.rs`, `entry/run_bridge.rs` | Language execution stays; host assembly must preserve agreed compatibility | Core-only `run_with_capture` currently automatically serves API routes, without Web installation/registration; baseline probe added before any port design is called compatible |
| `auto-api-gen/api_gen.rs` HTTP server/Vue HTTP client/templates | Web | Man `http_service::serve_rust`; UI `run_vm`, `run_vue`; generated Rust cold compile/run and 885 controls |
| Same generator's Tauri branch and generic transpile/type-promotion helpers | Appropriate existing language/non-HTTP owner; shared mechanism remains single source | UI Tauri/Rust UI call `try_full_parse`, companion/db transpile/postprocess/type helpers; no HTTP-only relocation of the entire module |
| `rust_workspace.rs`, `sidecar.rs`, `source_owner.rs` | Existing owner preferred, otherwise one restricted shared language carrier per T-02 | UI Rust generation, Man Rust service, source receipts, lock/freshness; Web cannot depend back on a delegating auto-api-gen facade |
| Test wrappers, HTTP fixtures, governed IDs and atomic obligations | Follow real production owners, preserve source/oracle and execution conditions | Preliminary lexical file inventory is not runtime enumeration or an equivalence receipt; per-ID migration still pending |

Local lexical inventory: `source-inventory.json`, 37 files, SHA256 `e16d60b13d6ca20656253ca0fa31eb74d9c644696c8ed295c683130731207a0e`. It records source bytes/hash and candidate symbol names, including embedded snippets; it must not be used as a Rust parser or test-identity oracle. Raw evidence is local-only.

##### Serial write windows

888 authoritative tree remains `D:/autostack/.wt/lang-888/auto-lang`. Its actual diff/status includes `entry.rs`, Core test registration, `http_server.rs::spawn_outcome_reporter`, `http_stream.rs`, several HTTP wrappers, Cargo aliases/features, governance scripts and full-regression driver. These are real overlaps, even though its current purpose is testing.

- 890 currently writes only its new `test/fixtures/modularity/web-port-probe` and its own evidence/Plan.
- Do not modify the above shared files or borrow 888's unlanded schema/worker changes. Later acceptance must bind formal source/schema/configuration and reconcile actual symbols sequentially.
- Runtime/compiler/ACE algorithms and worker/profile/cache/resource mechanics outside the Web seam remain read-only.
- Shared locks, test registration, governance/runner and final merge windows remain pending explicit serial ownership. No chat messages were sent and no other Plan was edited.

##### Next evidence

Build and run the new Core-only external consumer against actual TCP, exact per-invocation JSON and observed graceful shutdown. Its Cargo normal/build/dev tree already resolves without UI/Man/Tauri (captured locally as `core-consumer-tree.txt`). This is baseline compatibility evidence only. T-01 still needs the complete migration/test obligation map; T-02 still needs the proposed port and generation DAG, owned-body/lifecycle checks, and all required controls. No task is marked complete from these preliminary observations.


</details>

<a id="e890-69842b508b36ad9b"></a>

<details>
<summary>work\stage-sequencing-r4.md · SHA256 5aefbb214b54b7c3ff5395c2dd60782db5d4b9b60a542536d13c054ded1476ae</summary>

#### PLAN-890 r4：用户指定的阶段串行与正式基线同步

2026-10-30。此文只修订执行顺序与启动条件，不改变目标、AC、测试义务或
T-02技术契约。现行主计划 r4；T-02技术证据仍由
[r3契约](#e890-1a20e94b922d0662)和[r3交接](#e890-8caf4976a4ad8156)约束。

##### 当前停止点

本次仅T-02：已提交的代码/技术契约冻结为
`f56211c12f6cc8062f0cbfb5a4f0d03c8e4a958c`，阶段交接提交
`4ec696a631058a92c4f282fdab054a86dbadc2ff`。Plan保持
`executing / 2 of 12`，按用户指令暂停。保留原唯一实施树及分支、UI/Man/Down
兄弟树和本地证据；不进入T-03、不合入master、不归档、不清理。
阶段审查的同会话限制仍有效，不抵独立终审或整个Plan完成。

888由原会话优先独立收尾，不等待890。890不联系、操作或代替该会话；避让
共享测试脚本、注册、锁文件及最终合入窗口，不消费888未交付WIP。
本次没有核实或宣称888已合入；核实发生在用户启动T-03时。

##### T-03启动前的必需同步

以下按顺序执行，不能因历史全链授权自动跳过：

1. 用户明确确认继续；核实888已正式合入master，保存正式提交及其独立复审、
   验证与收尾锚。分支存在、工作树消失或状态文字不能代替正式合入证据。
2. 核实890阶段成果已提交、现有`plan-890-dev`及实际受管实施树干净。
   不换树、不重建分支、不掺入其他会话WIP。
3. 记录同步前890 HEAD、master HEAD和888正式交付提交；将包含888正式交付
   的master **合并进现有`plan-890-dev`**。此操作不是将890合入master。
4. 保留双方实现、既有测试ID与原子验收义务；处理冲突并提交，记录同步后HEAD、
   冲突文件和处理依据。若无法在原合同内解决，保留明确缺口，不强行冒通过。
5. 定向复核受影响接口、测试工具及T-02证据。未受影响证据记录相关实现、依赖、
   配置和输入不变依据后复用，不重跑整个T-02探针或整档作为同步仪式。
6. 后续治理以888**正式交付**工具为基础，对照r3包/API/四仓schema与原887义务。
   若存在实质合同差异，先作有原因、影响项和关闭条件的有界修订，不静默替换AC、
   减少测试义务或接受WIP schema。完成同步及必要复核/修订后才实施T-03。

##### 后续启动单元

| 顺序 | 本次获启动指令后允许的任务 | 单元内顺序 |
|---|---|---|
| 1 | T-03～T-04 | 建仓、基础包、Core接线；先完成上述正式基线同步 |
| 2 | T-05～T-06 | 先运行时迁移，再生成器迁移 |
| 3 | T-07 | Man/UI真实消费迁移 |
| 4 | T-08 | 测试资产、逐ID守恒及四仓治理 |
| 5 | T-09～T-10 | 独立发行、恢复及应用消费验证 |
| 6 | T-11 | 同树独立终审 |
| 7 | T-12 | 冻结门、合入、规范沉淀与安全清理 |

每个启动单元串行执行；完成后提交、更新主计划并形成简明交接，然后暂停，
等待用户启动下一单元。未通过则保留未完成状态、具体剩余项及关闭条件。
需要实质改变设计或合同，先记录原因并作有界修订，不无限追加修补。
阶段提交不代表整个计划完成；完整AC、独立终审、merge-owned门及安全清理义务
原样保留。T-12才允许按合同实际合入890及清理其工作树。

##### 修订收据

`stage: new / plan_revision: 4 / outcome: pass / changed: stage sequencing,
T-03 prerequisites / next: user`。
依据是用户本次明确指令；不增加产品范围或减少验收要求。
本次为文档修订，无生产实现、构建、测试、代码合并、888会话操作或远端操作。


</details>

<a id="e890-1a20e94b922d0662"></a>

<details>
<summary>work\t02-contract-r3.md · SHA256 8fd21e1526f6f8ba20c720bd753af6a41d02647f1a6d6a2ad3405ab245e8760c</summary>

#### PLAN-890 T-02 r3 execution contract

2026-10-30. This supersedes the **current instructions** in the r2 feasibility
and handoff documents; their failures and receipts remain historical evidence.
Scope: T-02 only, same trees, no T-03, merge, archive, cleanup or remote writes.
Goals, AC IDs, original test-ID obligations and twelve tasks are unchanged.

##### D-T02-01: invocation input and activation

Language owns `vm::host_capabilities::http_service::{HttpServiceLaunch,
with_http_service_launch, with_http_service_launch_async,
current_http_service_launch}`. `HttpServiceLaunch::new(Arc<T>)` accepts only
`T: Any + Send + Sync`; `settings::<T>() -> Option<Arc<T>>` preserves the exact
domain-owned type/source identity. It is a typed immutable payload, not a map
of private Core callbacks or compiler declarations. Web's configuration type
remains single-source in the pure contract package. A host must reject an
unsupported payload type; it must not fall back to environment configuration.

The synchronous SDK bridge snapshots before spawning its worker, restores a
scoped context there, and binds the launch to the VM before main execution.
Async scope restoration happens on every poll, including Pending/error/unwind;
an explicit empty context does not acquire another invocation's legacy input.
`VmHttpBridge::{bind_launch,launch}` supports existing custom VM builders and
borrowed listeners without exporting VM fields. `HttpVmOwner::launch()` exposes
the same VM snapshot before `into_local_parts()` is consumed.

The old `set_active_service_config` seam is compatibility input only: captured
once at invocation entry, never read by a new runtime at service activation.
Its process-wide single-slot nature is **not** a concurrent SDK API; T-07 removes
Man's use in favor of the explicit scoped input. Existing sequential Man entry
keeps working during migration. No new ambient fallback was introduced.
Host installation, duplicate installation, explicit replacement and selection
still happen at activation; an active service retains its original host.

Configured automatic service uses its resolved config's address, port, budgets
and hash; environment is only the existing unconfigured legacy input. CLI and
project precedence stays with the current config resolver/Man assembly. Port=0
publishes the kernel address in instance identity. The direct borrowed grades
retain their existing ServerHandle / LegacyStdnet distinctions; this revision
does not claim those old blocking implementations gained async deployment policy.

##### D-T02-02: result, readiness and lifetime

`HttpServiceHost::serve(HttpVmOwner) -> HttpServiceFuture` remains local and
returns `AutoResult<()>`. `serve_async`, `serve_async_with` and `serve_with`
propagate actual spawn/bind/network-join/log errors; no caller manufactures Ok
after a unit-return runtime. Process fatal/bound slots remain historical
facades, never the result/readiness authority for the new session. Readiness,
identity, counters, shutdown and cleanup receipt belong to its `ServiceRuntime`.

`NetworkSession` owns an independent stop sender and starts a blocking join
worker when the actual network thread is created. Its Drop requests stop;
dropping the awaiter cannot discard the worker's responsibility to join.
External shutdown is relayed on the independent network reactor as well: it
must stop intake and start drain even while the VM owner cannot be polled.
`ParkedRequests` cleans local tasks on the VM owner thread on normal or cancelled
exit. Signal tasks are owned and aborted on return/Drop. No VM or borrowed
context enters a network/blocking thread.

After the network runtime and all its producers exit: join network thread,
emit its Stopped state, close the log sender, drain accepted log lines, join log
thread, then publish cleanup result. The service Result awaits this sequence.
Queue-full emission still drops/counts immediately. Log spawn/write/panic and
network panic/join errors fail the owned result; a detached cleanup failure also
gets a diagnostic. `ServiceRuntime::wait_cleanup() -> Result<(), String>` is an
independent receipt available even after serve-future cancellation. It proves
resource cleanup, **not** startup success. Normal service errors and cleanup
errors are distinct; dropping a future alone is never a completion receipt.
No infinite-resource or blocked-stderr progress guarantee is added.

R3: successful parked-reply delivery transfers completion to the network bridge.
The bridge sets status/bytes before terminal emission, including its late-reply
branch; failed delivery cancels. Error/timeout/disconnection/cancellation remain
exactly-once terminal paths. File/SSE retain actual body-driven terminal ownership.

##### D-T02-03: frozen physical/API destination (implementation still T-03 onward)

All new packages use initial local version `0.1.0`, edition/workspace policy as
their owning repository; no registry/remote publication is implied.

| Package / owner | Frozen source cut and public destination | Dependency rule |
|---|---|---|
| `auto-lang-core` / language | existing neutral host, launch, owned messages, `VmHttpBridge`, compilation metadata and language registries | zero Web runtime dependency; deployment types leave Core; no VM/AST/Store copy |
| `auto-web-contract` / Web | `http_service_config/contract.rs`: strict config resolver, hash and pure Host/CORS/proxy rules; root re-exports preserve one type identity | zero Core/VM/Axum/UI/Man |
| `auto-web` / Web | HTTP dispatch/transport/scopes/blocking grades, mutable `ServiceRuntime`, observability and session ownership, service/file/upload domain executors, HTTP generation/templates/assets/client generation | public Core SDK + contract + restricted generation carrier/media/a2r; normal/build/required-dev closure zero UI/Man/Tauri and zero mixed `auto-api-gen` |
| `auto-generation-core` / language | exactly existing `shared_generation`, `rust_workspace`, `sidecar`, `source_owner`; the 25-function list in r2 §4 is retained verbatim | existing language/project-model dependencies only; no HTTP policy or UI/Man/Tauri; SidecarDecl stays project-model |
| `auto-api-gen` / language | compatibility/non-HTTP generator facade; HTTP named delegation → Web; Tauri stays original owner | Web never depends back on this facade; no second HTTP implementation |
| `auto-media-core` / language | existing renderer-neutral photo implementation; UI only re-exports | no Web/UI reverse edge |

Web public root: `WebHttpServiceHost: HttpServiceHost`,
`install_http_service_host_if_absent() -> bool`,
`service_launch(Arc<auto_web_contract::HttpServiceConfig>) -> HttpServiceLaunch`,
`ServiceRuntime`, and HTTP `generate_api(project, mode)` /
`backend_generation_is_fresh(project)` with current argument/result types.
The installer delegates to Core's installation primitive, does not override a
custom provider. Explicit override uses Core's existing setter. Tauri mode is
not a Web generation API. New symbol locations are execution destinations,
not claims that these packages already exist.

Consumers remain exactly r2 §5: Core automatic/source/debug/custom SDK and
blocking primitives; Man VM/Rust service tracks; UI VM/Vue HTTP generation;
Tauri/Rust UI shared generation only; generated external Rust; fixed Blog source.
Man/UI combine the adapter internally; direct SDK service consumers install it
and pass launch explicitly. Old names remain thin compatibility delegates where
the original migration contract requires them. No retained old runtime fallback.

##### Four-repository receipt and command contract

887 production blobs remain the r2 §6 baseline. T-08 owns implementation of the
following extension, in its serialized shared-file window; T-02 does not write
the shared runner, registrations, lock or 888 WIP schema.

| Surface | Frozen extension |
|---|---|
| Fixture | `four-repository-fixture/1`, `plan=PLAN-890`, `phase`; existing checks `{id,status,detail}`, pending and summary preserved. `sources` retains `lang/man/ui/down`, adds `web`; `sources.lang` denotes owner `language` |
| Runner | existing `tier/filters/list_only/roots/identities/results/outcome`; add `mode=four-repository`, `required_owners=[language,web,ui,man]`, `required_legs`; roots/identities include Web and fixed Down; record before/after identities |
| Identity | each actual absolute root, HEAD, product/source diff hash, lock hash, resource manifest hash and runtime producer identity; missing identity/hash is failure. All required owners must match before/after; Down is a pinned dependency, never a replacement required owner |
| Legs | preserve current language/UI/Man legs and IDs; add Web `web-unit`, `web-http`, `web-generation`, all required. Each retains exact selected IDs, list/run command, cwd/package/features, exit, output path/hash and atomic old→new obligation map. Per-owner duplicate IDs, zero required selection, omitted run, hash/source drift or missing migrated ID fail |
| Full verdict | existing checks/test oracle remains the judge. Full four-repo pass requires zero failed/pending required checks and every required leg actually run on one unchanged frozen combination. `list_only`, filters, manual/ignored/pending entries can only describe partial evidence; old three-repo pass cannot certify four-repo |

T-03 creates Web manifest; T-05/06 implement exact migration test destinations:
`cargo test --manifest-path <web>/Cargo.toml -p auto-web-contract --lib`,
`cargo test --manifest-path <web>/Cargo.toml -p auto-web --lib`,
`cargo test --manifest-path <web>/Cargo.toml -p auto-web --test http_runtime`,
`cargo test --manifest-path <web>/Cargo.toml -p auto-web --test http_generation`.
These named integration files are **new destinations**, not presently runnable
evidence. Original IDs/assertions map atomically; new probe IDs cannot offset
any of 24869 history rows / 4683 original IDs / 696 Web rows / 55 macro / 17
resource obligations. T-08 implements `--with-web` on the existing runner and
`scripts/web-repository-fixture.py run --phase source-switched --require-complete`.
Full command at the assigned single-instance window is
`python scripts/run-full-regression.py tf --batch --with-web --frozen-trees --output <receipt-dir>`.
T-09 implements and runs fresh/locked/relocation/distribution/recovery entrypoints
already named in the main Plan. None of these future commands was executed here.

Order remains bootstrap → shared carrier/language bridge → Web runtime/generation
→ consumers → ID/runner migration → distribution/recovery → application checks
→ independent final review → frozen gates/landing/deposit/guard. T-02 ends before
bootstrap, with all current trees and branches retained. Final Plan review,
merge-owned gates and milestone/batch runs remain pending.


</details>

<a id="e890-8caf4976a4ad8156"></a>

<details>
<summary>work\t02-handoff-r3.md · SHA256 1d8f3773f2837ec930d938c83918ccb23f165e27cb9fe2b3b21163211f371ce9</summary>

#### PLAN-890 T-02 r3 stage handoff

2026-10-30. Current contract: [t02-contract-r3.md](#e890-1a20e94b922d0662).
This supersedes the r2 pause handoff's **next action**, without changing its
historical needs_replan verdict or deleting its counterexamples.

##### Stage result and boundary

T-02 bounded revision/repair is complete; stage acceptance check passes with the
same-session review limitation below. Plan stays **executing / 2 of 12**.
T-03 requires a new user instruction. No Web repository was created; no merge,
archive, worktree removal, branch removal, push, whole-tier test, docs_gen,
shared runner/registration/lock edit, agent or cross-chat message occurred.
This is neither AC-13 independent final review nor U890 delivery.

Production/source contract commit: `88e604ba0b80b54e59092fed27e58b9a2244e6df`.
Supplemental oracle commit: `28d5c41c1321b42bcda1efc0e1d85a400a20d455`.
Final implementation/contract freeze (including independent network-stop repair):
`f56211c12f6cc8062f0cbfb5a4f0d03c8e4a958c`, base `76f2069d5`.
The handoff documentation commit follows without changing implementation.

| Owner | Authoritative tree / branch / retained identity |
|---|---|
| language | `D:/autostack/.wt/lang-890/auto-lang`, `plan-890-dev`, freeze above |
| UI | `D:/autostack/.wt/lang-890/auto-ui`, `codex/plan-890-dev`, `7b323bc02e4a8153a086ca3f88c5f1a95b47c446` |
| Man | `D:/autostack/.wt/lang-890/auto-man`, detached `e6cc06fbc496f366edde7929abfd49254ec13647` |
| Down | `D:/autostack/.wt/lang-890/auto-down`, detached `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` |

UI/Man/Down unchanged. Main checkout only receives 890 Plan bookkeeping; its
existing other docs WIP remains uncommitted and untouched by this work.

##### Finding closure and actual evidence

All commands below run in the language tree. Raw evidence is LOCAL ONLY under
`docs/plans/attachments/890/work/`; only these Markdown summaries are committed.

| Finding | Actual outcome and distinguishing oracle |
|---|---|
| R1 | Old configured-launch audit now serves the declared port, active config consumed, child0. Explicit scoped launch ignores env fallback; ready bound/hash equal caller-resolved input; port0 reports nonzero kernel address. Two actual services interleave A's parked main with B's execution; each keeps caller-fixed config hash/port/instance, and closing first-activated B leaves A intact. Deliberately swapped launch payloads are rejected by the hash oracle |
| R2 | Occupied SDK port returns child1 with bind diagnostic. Within one process A fails, never gets identity/ready, logs join; B starts and serves exact business body, then independently shuts down/cancels. New runtime result no longer consults process fatal/bound authority |
| R3 | Original parked async wire200/25 now has exactly one event200/2/completed. A genuinely parked handler error has event500/87/completed; cancellation of an actual in-flight slow request emits one canceled event and resources clean. Ordinary completion is owned by bridge; delivery failure cancels. File/SSE body terminal replay also passes |
| R4 | Normal, failed startup and dropped serve task wait actual network/log joins. Cancellation awaits aborted owner-task destruction plus retained `wait_cleanup`, not a delay or process exit heuristic. A 150ms fixture tail runs on the actual network thread; cleanup waits past it. Log worker error and network panic are reported by owned results; reactor timer still progresses while join waits |
| R5/R6 | r3 contract freezes package/source/API/consumer/command and four-repository extension/rejection rules; legacy three-repo schema and 887 blobs preserved. New packages/commands remain explicit T-03..09 implementation destinations, not fictitious executed evidence |

Executed checks (all exit0):

- Build: `cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin web-port-probe --bin isolation --bin session --locked -j 4`.
- Original counterexample replay: `python test/fixtures/modularity/web-port-probe/t02_closeout_audit.py --sdk-exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --isolation-exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe` — 3/3 pass.
- `python test/fixtures/modularity/web-port-probe/t02_session_probe.py --sdk-exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --session-exe test/fixtures/modularity/web-port-probe/target/debug/session.exe --isolation-exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe` — 5/5: explicit priority, port0/hash/control shutdown, A-failure→B-shutdown, A-failure→B-cancel, interleaved-main launch. `python docs/plans/attachments/890/work/t02-r3-launch-negative.py` — caller-bound hash oracle rejects the actual swapped launch mutation; local-only driver hash below.
- `response_observation_probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe` — 7 exact terminal events, 2 RST canceled-once, truncation and connected stalled-file watchdog canceled-once, other-owner counters unchanged.
- `probe.py --exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --mode missing` / `--mode plain` — missing host child1/no listener; ordinary result42/child0/resources0.
- `cargo check -p auto-lang` — exit0. Existing warnings: Core139 / root3 / external16; Core lib-test159. No new unhandled warning; no global warning-clean claim.
- `cargo test -p auto-lang-core --lib <filter> -- --exact <full-name>`: `launch_context_restores_after_nested_unwind_and_interleaved_polls`, `owned_log_sink_drains_joins_and_reports_worker_errors`, `session_cleanup_error_is_owned_and_join_does_not_block_reactor`, `explicit_http_host_selection_is_stable_for_active_local_owners`, `external_shutdown_releases_network_without_polling_vm_owner` — each 1 pass / 0 fail / 0 ignored. Full paths are in corresponding logs and source.
- Changed Rust files `rustfmt --edition 2021 --config skip_children=true --check`; `git diff 76f2069d5..f56211c12 --check` — pass.

The first supplemental gap probe failed because its controller assumed
`stop-first` meant source order. With A's main parked, B actually activates first;
the controller was querying the already stopped B. This is a fixture error,
not a product failure. The corrected control checks surviving A and pre-fixes
expected hashes in the caller before launch; it rejects swapped settings.
Original failed log remains local (`t02-r3-launch-gap.log`).

Unaffected r2 compiler metadata, owned upload/resource, native policy, blocking
grade, shared-generation/non-HTTP and generated-Rust receipts remain bounded by
their original assertions. Their domain implementations/inputs/dependency
manifests were not changed here; new launch fields do not certify their final
physical migration. No original test floor/ID/golden was retired or regenerated.

##### Phase acceptance review

`stage: review / review_scope: phase(T-02) / plan_revision: 3 / outcome: pass`.
Reviewer limitation: the implementation session reconstructed the verdict from
committed source/diff and executable artifacts; there was no independent agent,
model or final-review context. AC-02/03/07 pass **only their T-02 feasibility
portion**; overall physical ownership, CLI/UI integration and all other work ACs
remain with T-03..10. AC-13 and merge-owned gates remain pending.

Inspection covers launch producer→worker→VM→host, both runtime result branches,
ready ownership, response send/bridge finalization, local parked Drop, actual
network/log join and cancellation receipt; package DAG and four-owner schema
are checked as execution contracts. Supplemental coverage finding
`T02-R3-COV1` (real interleaved launch identity) is closed by the positive and
swapped-input control. `T02-R3-SHUTDOWN1` exposed an actual regression: the new
internal stop channel initially depended on owner progress to relay external
shutdown. Its durable test fails before repair (exit101, 2.02s) and passes after
repair (exit0, 0.01s) while the owner executor is deliberately unpolled. The
network-thread relay now preserves independent external stop and owns/joins its
relay task. Final audit/session/body replays and cargo check were rerun after
this repair; unrelated launch/sink/selection checks are unchanged and reused. No remaining T-02 finding, unauthorized deferral or
workaround was found. No new out-of-contract debt candidate was introduced.
Historical Tauri fixture red, warning baseline and later migration/ID/consumer
obligations remain explicitly pending, not downgraded or marked fixed.

Spec inputs: language-sdk/project.md SHA256
`dfb7d5efa901179c7495902872e93d5b40e1d079d3ad7517f86521ed2c343656`;
architecture.md `30fe0cdb9b7d1933410483653f484d7d106b493397af5c77b8fb803f02a0195e`;
stdlib/design/http-service-deployment.md
`cea4cd929f9b2001cb4a4318f249a743b2074feb666a7ee55899d29381c0f5f0`.
SD-03/05 acquire the launch/session/cleanup rules; other SDs stay unchanged.
No canonical Spec, ledger or derived index was published.

##### Next entry

Pause after T-02, retain all trees/branches and local evidence. On explicit
user authorization for T-03: reread shared Plan r3 and this contract, recheck
these identities/cleanliness and actual shared window, then bootstrap the local
Web repository using the frozen package/API destinations. Do not repeat T-02's
entire probe campaign unless a relevant source/config/input changes. Final
independent review and all frozen merge gates occur in their original stages.

##### Local evidence hashes

Base directory: `D:/autostack/.wt/lang-890/auto-lang/docs/plans/attachments/890/work/`.

| File | SHA256 |
|---|---|
| `t02-r3-final-build.log` | `14b0a022fecbd2dface94ea85ba0aa815305d0bb6fe5fb36e35ae2a59a754118` |
| `t02-r3-final-check.log` | `c66ddfb944da21ed120b8a3ccd541ed605d18f017504344c78ec06ae11b7786b` |
| `t02-r3-final-audit.log` | `2d2355bd8ff6463644aa165046f7df4a5985c098bd5ccfa4d6e72ac1b7c7e6a8` |
| `t02-r3-final-session.log` | `b46613cdf377c8f958a2334f5cb33463ca32d44048ca4523dbbb9d68a36c227c` |
| `t02-r3-final-body.log` | `e0e79466e92259ca4aa5852292546875b4e031e907f7416b03b7c7c18a933f66` |
| `t02-r3-final-negative.log` | `9bf945b6283c2624e15b676eb42f2062698939307008d1d29903f8aee4c63f84` |
| `t02-r3-launch-negative.py` | `0908328b26f1a090bdba66d3e289828251c0db8f2d31bce83b20733beacf933e` |
| `t02-r3-independent-stop-before.log` | `9df7d397da4d29eef9d1612c1fb112811ce5a3c1c3963fe575499ef9d0481231` |
| `t02-r3-independent-stop-after.log` | `7da932467c768b3a75588e0b235aa1cd14b39b6e3bc6a6752d1c5596b7a31b47` |
| `t02-r3-launch-test.log` | `fae561011a33a68816f19aa845e4388dc4dfd90d0f2372435c38bf6775ce76d2` |
| `t02-r3-log-test.log` | `a48e37ea5898cf51d2cb1e58c0c4acd466a00f2a1dbbd91344768e34c15e5c49` |
| `t02-r3-session-error-test.log` | `acddc2ec47371acc831267090ed9ff33e13df76418dbbcaf2ca180b3a8605aa0` |
| `t02-r3-host-replay.log` | `b9a6fd728e004f83a569a2eed98f40e4349b5490f9c2142205c123395d14cd38` |
| `t02-r3-missing.log` | `471911e6054b9caaa9f2b2bfd7ae67c99e90126f0c46fbe74daeabf8e3466f4f` |
| `t02-r3-plain.log` | `dbcbc6c5c0c30f7bdf144c0509ec411f77b490934ad187b112fa477fc462d385` |

Frozen r3 contract SHA256: `8fd21e1526f6f8ba20c720bd753af6a41d02647f1a6d6a2ad3405ab245e8760c`.


</details>

<a id="e890-41796130e16d9105"></a>

<details>
<summary>work\t02-handoff.md · SHA256 b7e4e2634ed6f039b884c70b5ab34a495f3ebb9f83cd9cb9c48229fa823668de</summary>

#### PLAN-890 T-02 阶段暂停交接

2026-10-30；依据用户最新“仅完成T-02后停止，不自动T-03”指令。

**结论：needs_replan；T-02未完整通过；Plan仍executing、current_step=1/12。**
本次完成的是有界阶段审查与保存，不是T-02验收通过或整个890交付。
保留所有现有工作树/分支；未合入master、未归档、未清理、未创建Web仓。

##### 冻结与权威位置

| 项目 | 固定身份 / 状态 |
|---|---|
| 语言权威树 | `D:/autostack/.wt/lang-890/auto-lang`，`plan-890-dev` |
| 代码与契约冻结 | `33a467a98fd2929640050550b3faf00d5a7824ed`；本交接文档随后单独提交，不再改实现 |
| 生产实现最后变更 | `e2ed6ba761b1082af34e759012f53d292cd46346`；本次仅新增诊断探针与契约文本 |
| UI同寿命树 | `D:/autostack/.wt/lang-890/auto-ui`，`codex/plan-890-dev`，`7b323bc02e4a8153a086ca3f88c5f1a95b47c446`；本次未改 |
| Man同寿命树 | `D:/autostack/.wt/lang-890/auto-man`，detached `e6cc06fbc496f366edde7929abfd49254ec13647`；只读，未改 |
| Down只读树 | `D:/autostack/.wt/lang-890/auto-down`，detached `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4`；未改 |
| 主计划 | `D:/autostack/auto-lang/docs/plans/890-auto-web-independent-repository-delivery.md`；只更新890簿记，不提交/混入主检出其他WIP |
| 端口/包图候选 | [feasibility-and-contract.md](#e890-87d69d695303f960)，SHA256 `cfdc162d585eea28a0321c5232dd42b96061abb8b1cac4aed438938ced91d1b5` |
| Web身份 | `D:/autostack/auto-web`与组内`auto-web`均不存在；不能据此推断远端身份，也不得在本次创建 |

提交本文件后，语言/UI/Man/Down四树应保持tracked/untracked状态clean；ignored
target和本地证据保留。“clean”不等于清空构建产物。主检出既有其他docs WIP不属于
这份冻结，不作stash/rebase/merge或删除；本次无主检出代码改动。

##### 可复用完成项

- 显式host安装/重复安装/覆盖、缺安装诊断、普通语言兼容、原线程!Send owner、
  owned请求/上传流、JSON/SSE/file/upload的分项真实路径和具名资源桥。
- 每实例请求准入、配置观测上下文、native限流、直接语言监听与阻塞等级；
  网络线程正常关闭join、文件/SSE实际body终态/字节/状态、RST/截断/watchdog控制。
- compiler声明在main前归VM，params/returns/shadow/async/routes单一owned DTO；
  实际main等待+另一程序编译、并发编译、嵌套/错误/unwind、普通程序不继承路由。
- 共享生成25函数与四模块的受限载体接缝、真实companion类型/导入正反例；
  renderer-neutral photo已在现有media单源，UI仅兼容re-export；真实生成Rust无UI闭包。

以上均按各自收据的代码/依赖/输入及已列限制复用。不能扩大为完整部署/退出、
四仓发行、全部历史ID执行、CLI/UI集成或独立最终复审通过。

##### 本次必要验证与未关闭风险

只构建受影响SDK探针（locked，exit0），不运行根整档/GUI/docs_gen。
`t02_closeout_audit.py`实际exit1，三个required检查均失败：

| 项 | 实测 | 后续必做 |
|---|---|---|
| 配置交接 | 配置5077，精确业务在env5078，配置未消费，child0 | 决定并实现per-invocation launch/config传递；保端口/配置优先级、hash与实例隔离 |
| 启动失败 | 占口有bind诊断，SDK child仍0 | Web runtime真正传播Result，ready/failure归调用；停止依赖process-global fatal/bound作新调用权威 |
| 普通异步观测 | wire200/25；event status=null、bytes0、completed | 修owner提前complete与bridge填事件的竞态，补成功/错误/取消/资源回收定向例 |

源码额外确认日志sink丢弃线程句柄；其终态join未证明。需要runtime-owned日志
session及排空/join/错误策略。service future被丢弃的控制/回收语义也未冻结，不能
从正常shutdown通过推断安全。最终物理包/API/命令/四仓schema仍是未批准候选。

raw build/audit日志及hash见契约§6。阶段audit非零就是阶段未过，不因“发现了反例”
改记pass。父版本Tauri字节fixture红及存量warnings也继续保留，不退役/重基线冒绿。

##### 下一次直接入口

**当前入口是T-02有界方案修订/修复，不是T-03。**

1. 重读主计划最新用户边界、本交接与契约；读回上述树/分支/HEAD与dirty状态。
   当前交接提交相对代码冻结只能新增本Markdown；其他变化需重新核验受影响证据。
2. 在同一Plan明确D-T02-01 launch所有权与D-T02-02 session/结果/日志终态方案。
   推荐独立中立launch上下文，保持激活时host选择，保现有Result签名但落实错误传播；
   不把部署配置塞入compiler metadata、不扩大Core私有pub面、不加更多全局fallback。
   需要修订执行设计时用auto-plan:new有界修订，目标/AC/仓集合不缩减。
3. 获得下一次执行指令后，在这些原树仅修契约R1..R4，重放下面的实际失败命令，
   补配置hash/port0/实例失败隔离/取消/日志终态的精确关闭证据；R3是工程未完成项，
   不以“待决策”替代修复。随后确认R5/R6的包API与887兼容四仓收据设计。
4. T-02完整通过仍需显式记录；只有用户另行允许下一阶段，才进入T-03。
   共享脚本/测试注册/锁/最终合入窗口与888串行；不借其WIP schema，不发跨聊天消息。

cwd必须是语言890树；当前失败命令可直接重放：

```text
cargo build --manifest-path test/fixtures/modularity/web-port-probe/Cargo.toml --features external-runtime --bin web-port-probe --bin isolation --locked -j 4
python test/fixtures/modularity/web-port-probe/t02_closeout_audit.py --sdk-exe test/fixtures/modularity/web-port-probe/target/debug/web-port-probe.exe --isolation-exe test/fixtures/modularity/web-port-probe/target/debug/isolation.exe
```

后续T-03..12目标未取消，但均未在本次执行窗口内：Web bootstrap、物理迁包/原件退出、
Man/UI接线、逐ID守恒、fresh/locked/重定位/恢复、Blog及双UI、独立终审、冻结门、
合入/规范/归档/guard仍待完整授权链执行。UI/Man结构后继继续等待完整U890。


</details>

<a id="e890-adbb1a260bdcd577"></a>

<details>
<summary>work\t03-t04-baseline-r5.md · SHA256 4b6891b39d3a85d3dc4301fa39102555358214b0f90d06a5d40c6f31732c266c</summary>

#### PLAN-890 T-03/T-04 r5 baseline

User authorizes only sync + T-03/T-04, then commit/handoff/pause. No T-05, final merge, archive or cleanup.

888 delivered: 5a6c23afacd3c40412c5561a5ba2ae322a119c2e, archived 107148fcd, closed 129ef605763e59ca13bf312fd4a391eded10c13a. All are master ancestry; archive records 10/10, independent review/required gates and fresh guards. No conversation operations performed.

Sync: clean plan-890-dev ed44fd0bd6ea063e8e4977454bd5ef04455aa762 + master 129ef605763e59ca13bf312fd4a391eded10c13a -> merge 29ed9d7bde334ae14e21223c935a4f3974581f71. Ort auto-merged entry.rs/http_server.rs, no manual conflicts. Entry adds gated file-test trace wrappers; HTTP adds gated test reporter port. Stream adds gated liveness accessor. T-02 launch/session/metadata production bodies otherwise unchanged at sync; old SDK source manifests/features changed so build/affected HTTP evidence will be rebound. Shared-generation code and inputs unchanged at sync and reusable until its T-04 relocation checks.

Formal rule: AGENTS.md module selection section + docs/specs/auto-test-case/module-selection.md. Old t/tv/tt/tb/taa stacking is superseded by explicit tm complete modules and consumer/integration union; final merge-owned gate remains pending. No IDs/oracles/conditions reduced. Web four-owner integration will extend the delivered module-aware tools in T-08, not the old frozen 887 runner alone. Periodic recipes stay independent. Scope/AC unchanged; main Plan revision5 records this justified tooling-contract correction.


</details>

<a id="e890-18ff84e0bd8a968f"></a>

<details>
<summary>work\t03-t04-handoff-r5.md · SHA256 1c0fd30792cf7b0be21112e05ec25c5fcb1f05847b7b69e88068255135955a81</summary>

#### PLAN-890 T-03/T-04 r5 handoff

2026-10-30. Stage work result: T-03/T-04 foundation and targeted acceptance complete; Plan remains executing 4/12. **Expanded tm consumer gate is NOT passed** and remains merge-owned. This is the next-stage contract after r4 sequencing and r3 technical
freeze; historical receipts remain valid only within their recorded boundaries.

##### Authority and fixed candidate

User explicitly authorized delivered-888 verification, master→existing890 sync,
affected T-02 verification with formal tm, then T-03/T-04 only. Stop before T-05.
No landing of 890 to master, remote creation/push/publication, archive or cleanup.

888 delivery `5a6c23afa`, archive `107148fcd`, and final closeout `129ef6057` are
master ancestors. Its archived Plan records 10/10, review, gates and guarded
cleanup; no other conversation was contacted or operated.

Sync: clean `plan-890-dev` `ed44fd0bd` + master
`129ef605763e59ca13bf312fd4a391eded10c13a` →
`29ed9d7bde334ae14e21223c935a4f3974581f71`. No manual conflicts.
Entry and HTTP source auto-merges preserve 888 gated tracing/reporter accessors
and 890 launch/session bodies. Shared scripts/config/AGENTS remain byte-identical
to delivered master. No 888 WIP was consumed.

| Repository | Worktree / branch | Candidate source commit |
|---|---|---|
| language | `D:/autostack/.wt/lang-890/auto-lang`, `plan-890-dev` | foundation `84779b2b1`; final test-assembly repair `1a253950810de67f16f487c7eb7a2745f1061bb0` |
| Web | `D:/autostack/.wt/lang-890/auto-web`, `codex/plan-890-dev` | `26259642f56656547db7bc07a68c35a2bd225edd` |
| UI | retained `auto-ui` sibling, `codex/plan-890-dev` | unchanged `7b323bc` |
| Man | retained `auto-man` sibling, detached | unchanged `e6cc06f` |
| Down | retained `auto-down` sibling, detached | unchanged `c3464d0` |

Web is a new local ordinary Git repository at `D:/autostack/auto-web`, initial
governance/master bootstrap `8ff4966`. All product work is in its unique Plan
worktree. No junction/symlink or replacement language tree was created.

##### Foundation and actual ownership

- `auto-web-contract` 0.1.0 owns the exact original strict config/policy source.
  Core's old config path re-exports this same type. No VM/Axum/UI/Man dependency.
- Core's named SDK assembly inputs include the pure contract, using the actual
  Cargo-selected source root emitted by the contract's metadata build script.
  The Cargo metadata namespace also rejects two physical contract sources;
  this creates no native runtime library or implicit service implementation.
- `auto-generation-core` 0.1.0 owns exactly four existing modules:
  shared_generation/rust_workspace/sidecar/source_owner. The 25 shared helper
  list and the other three modules' existing mechanisms are retained. Two
  parser helpers gain hidden public visibility solely to preserve legacy test
  imports through the old facade. No private VM fields/modules were exposed.
- The four old API-generator paths are thin re-exports; no copied algorithms.
  All 15 existing module-test bodies/names are preserved with a recorded owner
  map. Formal registration/ID migration still belongs to T-08.
- `auto-web` 0.1.0 currently exposes the pure contract and real `service_launch`
  construction. It has no fake installer, success stub or runtime fallback.
  Runtime/generator ownership transfer, temporary Core runtime bridge removal,
  Man/UI consumer migration, formal distribution and four-owner module
  registration remain T-05..10. This stage does not certify those obligations.
- Web lock, source/MIT provenance, architecture/testing and CI integration
  contract skeletons are committed. No external CI was configured or claimed.

##### Evidence and affected-source reasoning

Actual accepted checks:

- Web workspace build/check: exit0. Actual normal/build/required-dev graph:
  482 packages; one Core/contract/carrier identity; no UI/Man/Tauri, mixed API
  facade or root `auto-lang`; no contract→Core/Axum/runtime edge.
- Real foundation consumer: same Core/Web config type and `Arc`, strict unknown
  input rejection, ordinary SDK result42, old/new generation facade behavior.
- Deliberately duplicated physical contract source is rejected by Cargo;
  accepted control is the actual foundation consumer. Initial negative fixture
  needed standalone workspace fields; its original parse failure is retained,
  not misclassified as the intended source-identity rejection.
- Moved carrier lib: 15 pass / 0 fail / 0 ignored, original tests and assertions.
- Actual pure-contract consumer: 1 pass; actual generated-companion import/type
  accepted and invalid compiler controls: 1 parent test passes.
- Mechanical-source proof: four modules exactly match normalized parent source
  after the two declared helper-visibility changes and rustfmt; pure policy
  source equals original normalized Git source. No SidecarDecl/VM/Store copy.
- `cargo check -p auto-lang --locked`: exit0. Core139/root3 and mixed facade3
  warnings are inherited; external boundary restored to original16 after
  declaring/forwarding the newly delivered test-only HTTP feature gate.
- T-02 rebuilt against real contract ownership: audit3/3, session5/5 pass,
  covering launch/hash/port0/occupied failure, async terminal status/bytes,
  failure isolation, main-gap config ownership, actual cancellation/log joins.
  Unchanged transport/body/resource and compiler declaration logic retain their
  original bounded evidence; no whole-Plan pass is inferred.
- Changed Rust formatting and diff whitespace checks pass. The first formatting
  command included the intentionally deleted contract path; corrected check
  uses non-deleted files, without altering product source to silence the check.

##### Formal tm selection and execution

r5 follows delivered AGENTS/module-selection Spec. Old bare tier stacking is
superseded; original IDs/oracles/conditions are not reduced. Final merge-owned
affected-module gate and periodic scheduling remain distinct from stage checks.

The first stale-directory discovery is **not** acceptance evidence: initial
lock resolution plus final source/config completion changed its input identity;
the tool correctly rejected freezing it. A new discovery on the frozen sources
produced 7183 obligations / 1186 leaves, with no tracked catalog/floor rewrite.

The formal selection uses:

```text
cargo tm --test-name http_service_config --test-name host_capabilities::http_service --test-name declared_path_deps_all_in_identity_sources --select-only --out docs/plans/attachments/890/work/t04-tm-frozen
```

Four directly selected leaves expand to 750 complete modules / 5483 obligations.
The actual execution uses the same navigation and the emitted runtime catalog,
without `--select-only`, at `t04-tm-execution`. No consumers were manually removed.
Web commit/lock/source identities supplement the current four-repository source
identity protocol; automatic Web-owner extension remains T-08.

###### Actual expanded-run outcome (not green)

The expanded run was intentionally stopped at the blocked legacy HTTP leaf;
`complete=false`: **58 modules executed = 55 PASS + 3 FAIL**, 23 original-condition
NOT_APPLICABLE; 81 of 750 module results recorded. The in-flight HTTP module has
no completed result and the remainder is NOT_RUN. The owner runner and its own
descendants were reaped; no other session/process was stopped. Exit1 is retained.

Three AAVM failures match the delivered888 archive's P763-R1 descriptions:
`test_aavm2_compile_corpus` and `test_aavm2_compile_use_corpus` fail E0433 missing
`auto_lang` in temporary Rust projects; `test_aavm2_goldens_check` fails
`STDASSEMBLY.INVALID_BYTECODE:60`. Related runner/VM/trans source is unchanged by
T-04. They remain original failures, not PASS, not silently retired or fixed here.

The blocked `language/auto-lang/api-contract-http-e2e/th/001` leaf starts services
through `crate::run`, discards its Result, and waits on a fixed port. It had no
explicit host installation under the approved new SDK contract. A bounded repair
adds a test-only explicit host in one support module, forwarding to the existing
single Core runtime bridge, and makes this helper surface early owner failure
and bound its readiness wait. It changes no production default, runtime policy,
assertion, ID, ignore or comparator. This is SDK test assembly, not T-05 migration.
T-05 must replace this adapter with the real Web installer when deleting the
existing temporary Core bridge. No old Core runtime is silently selected.

On repaired source `1a2539508`, diagnostic replay of the **entire original HTTP
leaf** passes 4/4. Complete members of the four direct Core leaves pass 23/23,
including config, host selection, compiler metadata and SDK source-identity
boundary controls. Replay uses original catalog identities/recipe arguments
against current source, without sampling a leaf. It is explicitly diagnostic
replay, **not** a fresh formal full-consumer receipt or a PASS for the interrupted
750-module selection. No global rerun was started after the bounded repair.

Six tracked plan818 reports produced by the module tests were copied to local
`t04-generated-reports/` with hashes and then restored from the frozen HEAD;
these owned generated artifacts are not implementation changes or new source.

##### Phase acceptance and remaining obligations

Phase code/acceptance inspection reconstructed from committed artifacts in the
implementation session: T-03/T-04 direct task portions of AC-01/02/03/04 pass.
This is **not independent final review**, not AC-13 and not whole-Plan approval.
T-04's package/port/source closure is supported by the actual builds, type/Arc
consumer, strict/duplicate-source controls, moved-source/test preservation,
real generated companion, 23 complete direct-leaf tests, HTTP leaf4 and T-02
replays. The broader consumer union is preserved as an explicitly **unclosed**
verification gate, not converted into stage success or omitted obligations.

Remaining risks/next-owner work:

- T-05 owns actual HTTP runtime/executor/observability migration and removal of
  temporary Core runtime bridges, including the new single test-host support
  consumer. T-06 owns HTTP generators; the four-module shared carrier is already
  single-source. Neither stage has been entered.
- T-07 owns real Man/UI composition and manifests; these repositories remain
  untouched. Source distribution/fresh/locked/relocation/restore is still T-09;
  grouped package checks are not independent installed-delivery certification.
- T-08 owns actual Web/module registration and original ID/atomic obligations;
  the 15 moved unit bodies have an owner map, not a fake migrated execution floor.
  Formal source_identity currently covers language/UI/Down/Man; Web source/lock
  pins are separately bound here until the four-owner extension is implemented.
- T-11 independent final review and T-12 frozen affected-complete-module union
  (all consumers/integration/necessary slow cases) remain pending. The expanded
  gate has no PASS receipt; baseline reds need explicit disposition and current
  source changes require a new/fresh source-bound selection before that gate.
  Prior partial consumer passes do not certify the repaired final tree as a whole.
- Existing Tauri fixture red, inherited warnings and other historical scope
  boundaries remain; no goldens, samples, scales, original conditions or test
  assertions were removed or weakened.

Next: user starts T-05/T-06 explicitly. Keep all five grouped trees/branches and
local evidence. No master landing, archive, cleanup, remote publishing or T-05
implementation occurred. Stage progress records work-owned foundation completion,
not completion of the deferred merge gate or overall AC-02/03 ownership migration.


##### Local evidence hashes

All paths below are relative to `D:/autostack/.wt/lang-890/auto-lang/docs/plans/attachments/890/work/`. Raw artifacts are local-only.

| File | SHA256 |
|---|---|
| `t04-foundation-check.log` | `3c1424e491648bca01cebb6b2a1335df93487f21211075eb1faa0da3c8afbe3e` |
| `t04-foundation-consumer.log` | `63a7f6f828980b5182d04ab51d04366be922344d31462c363c9f7a19e488094b` |
| `t04-package-graph.log` | `918029c38ebe0d13ed8b24462f09528a52226efe16092e1b85c98c855757de88` |
| `t04-contract-consumer.log` | `cdaf4f937401beabc3ff5c420d6426fa9f7034d0999031d189098ca4099d534a` |
| `t04-generation-unit.log` | `b914d91c324e4d0972f7cf8c0336977676821c4f55a41321f09f9b13bd17d8e9` |
| `t04-shared-companion.log` | `75e46f6f07d214446d724dca5b51aaee43fc03dc9174e46f235d3dfc67aa142a` |
| `t04-mechanical.log` | `76ed2d072fab2211735bbfaae14f074b50d6da4495928189aec6b7022ef660aa` |
| `t04-unit-owner-map.json` | `680f68cd995bfb78ff4ba37c3c079f723057b9ffcd4bafeee618c8bfcb734a35` |
| `t04-root-check.log` | `fcf1acdcf95ddae7d2271efa84d8b7dd3b9b580864fd10fdf7011fcc1388b650` |
| `t04-boundary-check.log` | `75c3363423f322452aa89c60e2496f53388f730d46af6301269666908e6423a5` |
| `t04-duplicate-contract-fixed.log` | `22a5c0007f81cf42d2de983f938c450e4452c26fd26e6715d3a573530af28754` |
| `t04-duplicate-contract.py` | `ef8f46de180331745a6cb2157ebc8d84fa17d8b6946fea1e1ae7e5170b037ebc` |
| `t04-t02-audit.log` | `34da33e53e0edde9c2dc92b56b4b8c1d5d35a43a71208d9f00b5b714fb347123` |
| `t04-t02-session.log` | `5f3c37fe1b85206aaa82c964880e4de5b6ec820356bee8bdff449987bab68047` |
| `t04-tm-frozen/selection.json` | `75e6d04fb6529395c7294d0eea73f821ffdc73e02f1be3c6d2e300eeef1ce1ca` |
| `t04-tm-execution/selection.json` | `73859e262a175f4380d5ed06d8bda8138cf27287ffb2d08adf85a52777846810` |
| `t04-tm-execution/execution/results.json` | `3dba20922c7e46d79bdb213fd7413badf929285376fdfc6235aa34c57bfe5b7e` |
| `t04-tm-execution.log` | `1cf44d06a3376d0606ada20f7b048f3c2c20128aa5a5dc3510a8a526a847be45` |
| `t04-sdk-http-leaf-replay.log` | `7f06078f7ba47d17b59d79b14b64bf43386acda30c2be7ecf3cd6d8b641de92b` |
| `t04-direct-leaf-replay.log` | `644eb53d6b005c5d3a6abd5732f07ef5d5e9827f492824c5dd4029ca27e858c2` |
| `t04-missing-control.log` | `471911e6054b9caaa9f2b2bfd7ae67c99e90126f0c46fbe74daeabf8e3466f4f` |
| `t04-plain-control.log` | `dbcbc6c5c0c30f7bdf144c0509ec411f77b490934ad187b112fa477fc462d385` |
| `t04-generated-reports/restore-receipt.json` | `fa6e835d0d6b2e68064b6e4709230a57672875990f50f1ca9cb9dd69f260e3fc` |

Candidate lock hashes: language `563cce71cb4fba1ba1ab6f00afd47cf896b004735797ae988c58f39790b00622`; Web `09951e9d3592b6918ab6da722c37204e5944ad49b6c396e2333cc594acb12458`.

Formal tm Spec SHA256 `77a6c290602575a2256ee33717fc0b3eafca9e9b2d43f53f82b6fa8f66e2d4c0`; r5 rules align with this delivered source, without editing shared tools.


</details>

<a id="e890-ca5b663d4f4f80ee"></a>

<details>
<summary>work\t05-t06-handoff-r6.md · SHA256 2713dcd8ace43d6f3224a784dacc6d1c43a6a733eded35d4705d118b6ca0dbb1</summary>

#### PLAN-890 T-05/T-06 r6 handoff

2026-10-30. User's “继续后续阶段” starts r4 unit 2 only: runtime → generator,
then commit, update and pause. Result: stage-owned T-05/T-06 implementation and
targeted acceptance complete, Plan **executing 6/12**. **The combined product is
not yet buildable; the full tm consumer union remains NOT_PASSED.** This is not
independent final review or a whole-AC/whole-Plan pass.

##### Frozen candidate and ownership

| Repository | Existing authoritative tree / branch | Source commit |
|---|---|---|
| language | `D:/autostack/.wt/lang-890/auto-lang`, `plan-890-dev` | 1b177ebab1dab4203588abd77047026aaa366eb9 |
| Web | `D:/autostack/.wt/lang-890/auto-web`, `codex/plan-890-dev` | d8c889c786d83778f4b46efd7e73d6a8fb2935bf |
| UI | retained sibling / `codex/plan-890-dev` | unchanged `7b323bc` |
| Man | retained sibling / detached | unchanged `e6cc06f` |
| Down | retained sibling / detached | unchanged `c3464d0` |

Entry language `e45399a63`, Web `26259642`; all five trees were clean. No new
worktree, junction, agent, cross-chat message, remote write, master landing,
archive or cleanup. Main checkout's other documentation WIP was preserved.

- Web now owns actual transport, route matching/argument dispatch, scopes,
  blocking grades, runtime/policy/observation/network+log joins and file/upload
  executors. `WebHttpServiceHost` and its non-overriding installer are real.
  The CLI `auto-web run <source> [--config <json>]` composes this same provider.
- Core retains compiler declaration snapshots, language JSON/value conversion,
  native registries, launch/request/resource hooks and legacy invocation input.
  All `run_existing*` temporary runtime methods and Core's domain implementations
  are removed; no fallback implementation is retained. The legacy stdnet public
  wrapper remains a single Core host-selection bridge and is re-exported by Web.
- The upload neutral executor port preserves the VM's original host limit
  snapshot and phase hook; ordinary a2r upload_receive is literally unchanged.
  An additive trait method has a default for existing custom executors; Web
  delegates it to the existing single scoped receive algorithm.
- Web owns HTTP service/client generation, templates and Axum/TypeScript targets.
  Core owns extraction/contract/trait and Tauri target. `auto-api-gen` keeps IPC
  orchestration/algorithms and its three original IPC tests, delegating HTTP to
  Web. The upper auto-lang facade has a thin mixed target selector; direct Core
  HTTP generator consumers migrate to `auto_web::api`, as inventoried in T-01.
- The four shared generation modules remain single-source in the language
  carrier. Only existing relative-path/source-validation helpers gain hidden
  public visibility. Web's adapter validates its selected source before calling
  the shared workspace mechanism and adding its fixed `=0.1.0` dependency.
  Generated service manifests/paths now call the real Web runtime; VM Axum 0.8
  and generated Axum 0.7 retain their separate transport objects.
- Original source comments, MIT license, test names/bodies/assertions/conditions
  are retained. Formal old-ID/new-owner accounting is T-08, not certified by
  this move. Do not interpret language deletion counts as whole-system savings.

##### Actual targeted evidence

All accepted commands below finished successfully (exit 0). Test replay claims
are bounded to these invocations and their source/config inputs; no global run.

| Check | Actual result / oracle |
|---|---|
| `cargo check -p auto-lang-core -p auto-api-gen --locked -j 4` after formatting | pass; Core128/Web13/facade1 inherited warnings recorded, no repository warning-clean claim |
| actual Web Cargo metadata, locked | 482-package Web closure; Core475; pure contract16. No UI/Man/Tauri/auto-api-gen/auto-lang in Web closure; no Core→Web implementation edge; one contract identity |
| `cargo test --manifest-path ../auto-web/Cargo.toml -p auto-web --test http_runtime -- --test-threads=1` | 2/2: real installed crate/CLI exact JSON, control202, owned exit/port release; scoped upload snapshot + ordinary-facade accepted control and staging/session cleanup |
| original audit/session controllers against relocated source | audit3/3 and session5/5: config priority/hash/port0, occupied failure, parked status/bytes, failed A→B shutdown/cancel, main-gap isolation; real network/log joins |
| original response/upload/isolation/blocking controllers | exact sync/async file/HEAD/Range/ETag/404/500/SSE; 12MiB upload/download,409/422/headers-only401/bind400, actual staging-before-RST cleanup; two-owner metadata/admission/independent shutdown; borrowed listener real JSON and task/controller completion |
| final SDK build + upload/missing/plain/legacy | local ignored lock refreshed offline, then locked build0. Final upload TCP/RST and registries0; missing host child1/no listener, ordinary42/child0, legacy exact CORS/preflight/JSON/SSE/unsupported-file500/collision/RST and resource baselines |
| `cargo nextest run --manifest-path ../auto-web/Cargo.toml -p auto-web --lib -j 1 -E 'test(/^http_/)'` | all109 affected default runtime tests pass in isolated per-test processes;67 filtered other modules, not missing obligations |
| `cargo test ... --lib api_gen::tests -- --test-threads=1` | 48 pass,1 original ignored; includes companion/import/time/borrow/code-region and actual Rust compile controls |
| `cargo test ... --lib api_targets -- --test-threads=1` | 18 pass, original cross-target assertions retained |
| `cargo test ... --test http_generation -- --test-threads=1` | 1 pass: public cold generate, missing Web preserves exact old workspace bytes, restore/regenerate, unmodified Rust build, materialized lock/freshness, exact890 TCP response and shutdown/port release. Generated lock488 packages, forbidden owners0 |
| feature-enabled original generated wire matrices via nextest, -j1 | 3/3 pass: original729 file,730 upload,776 JSON/busSSE. Single-run102.641/11.983/70.718s including fixture preparation; no speed claim. Original on-demand ignored738 remains not executed;216 entries filtered/ignored, not a whole feature-suite pass |
| original non-HTTP owner named controls | Tauri upload-unsupported1/1 and real mock dispatcher1/1 pass; old fixture-format red remains original bounded evidence, not reclassified green |
| mechanical proof / format / whitespace | file executor, observation, transport and legacy-grade whole sources equal parent after same rustfmt; ordinary upload facade exact; no temporary Core bridge.48 stage Rust files formatted; staged diff whitespace clean |

The source-boundary fixture still compiles the **single relocated Web source**
with its original synthetic real-network-thread tail wrapper, and records actual
scope/VM registry and retained cleanup receipts. It is diagnostic, not a claim
that its wrapper is the production package. The separate real-package runtime
and generation integration tests exercise the actual installed Web crate.
Launch, metadata and resource registries are the existing Core SDK types; no
VM/AST/Store copy, public private-field expansion, pointer laundering or Send
coercion was introduced.

##### First failures and corrections (retained, not overwritten)

- Initial cut compilation found dangling source references/imports, cross-package
  test access and missing generator dependencies. Named test-only presence/bus
  helpers replaced private-map test access; no VM/private module was exposed.
  Initial source/fixture command errors are not credited as semantic controls.
- The scoped upload facade initially reparsed mutable environment limits. The
  first max-file/raw fixture was an invalid distinguishing oracle (old raw path
  does not enforce that separate cap); it is retained as fixture error evidence.
  Corrected wire-limit case actually rejects an environment-reparse mutation:
  old installed64, new env1024, requested256,100 bytes must be413 for the scoped
  path while ordinary facade admits/cleans100. Mutation fails this same assertion;
  restored code passes both controls. A timestamp-preserving copy first reused a
  stale mutated artifact; that failed replay was discarded, then a real rebuild
  passed2/2. No successful receipt relies on the stale artifact.
- A broad libtest-process runtime invocation was wrong for the original isolated
  global-limits tests: lease failed after an earlier test initialized the same
  static; the next total-timeout test could not use its own limits. That one owned
  test process was identified by exact binary/arguments+parent and stopped;
  its nonzero/partial log is retained. The correct serial nextest process-isolated
  run passed109/109 without changing assertions, ignores or deadlines.
- The first final probe `--locked` build correctly rejected its newly stale local
  ignored lock. It was refreshed offline, then locked build passed. No tracked
  lock was weakened or a stale binary counted after that rejection.

##### Required next-stage gaps

**Current combined root check FAILED**: `cargo check -p auto-lang -p auto-api-gen`
reported three UI-runtime back-proxy compile errors. They are newly exposed
consumer migration work, **not baseline reds**: old Core imports `match_route`,
`ApiArgBindError`, `build_meta_json`, `push_typed_string_arg` now belong to Web.
Read-only scan also finds Vue HTTP generation at `auto-ui-tools/src/vue.rs:9979`
still calling Core's moved TypeScriptGenerator. T-07 owns explicit Web dependency/
imports and host/launch composition in UI/Man. Do not restore Core runtime copies,
add a reverse dependency or claim the current full product builds. UI/Man were
not changed in this authorized T-05/T-06 unit. Their eventual real consumer tests
and full AC-03/04/05 remain pending in T-07 and later gates.

T-08 must register the actual Web owners/IDs/module/source identities and preserve
all original obligations/conditions. T-09 owns installed source packaging,
fresh/locked/relocation/recovery (current development-source checks are not an
installed-distribution certificate). T-10 application/full work self-check,
T-11 independent final review and T-12 frozen complete affected module/consumer
union, landing/Spec deposit/archive/guarded cleanup remain pending.

The earlier750-module union remains **NOT_PASSED**:55 pass,3 original AAVM reds,
23 original-condition NOT_APPLICABLE and interrupted/NOT_RUN items. No new union
receipt exists, no old red was fixed/retired here, and source/catalog identities
must be freshly selected at the later assigned gate. Existing warnings/Tauri
fixture red and platform/milestone boundaries remain explicitly bounded.

Next responsible role: user starts r4 unit3/T-07. Retain all five clean trees and
local evidence. This stage audit was reconstructed in the implementation context;
it does not satisfy independent AC-13 and does not set execution_done/reviewed.

##### Evidence hashes

Raw logs/probes/metadata remain local-only under this handoff's directory.

| Local file | SHA256 |
|---|---|
| `t06-core-facade-check-final.log` | `477e3ebfa460171eeb58ece4f461e1449d23042b08449430ba356687fdb414d6` |
| `t06-package-graph-final.log` | `0f18acaaf5c0714372a5d444a8054c54c88f045cc708e70f63844c9bab95534d` |
| `t06-web-metadata-final.json` | `c4632d7b74b85081ee27da297bcb13ea664b318c16db51c404bd1e80b45ce364` |
| `t05-upload-wire-snapshot-mutation.log` | `fd20bf8058366e705c4a6c6540caf516ef5a92f3173b2da627b1f088d1b6f61b` |
| `t05-upload-wire-snapshot-rebuilt.log` | `bb7cb54f73457fa74daf1edb3af32943fb7da7d642e3da553b5ff0e433f6021b` |
| `t05-audit.log` | `226f38cb92a063f88995c940456d133091aa243e609a64923ed990fdd713b4f4` |
| `t05-session.log` | `50f7ce3a7ef7f0a59bd4bb09a16e0443baa51756955335047df8e7311033b74c` |
| `t05-response3.log` | `7f7c6a130a84661a66058c0120f68cfcce5edd25fc2ce6c05016bb71d6fd4028` |
| `t05-upload-final.log` | `f0b6d4e519146e2a387941927474317f8443a58aba5c14a9436d168fcc7a0e1d` |
| `t05-isolation.log` | `a97cc8d53e3c961e84aab688f2f0098016d8517d909dc3122ae39b7e5c573a90` |
| `t05-blocking.log` | `a73c83218b903d294cc60beaa07304a2482b93e3b7b3c8fe377e3d989b124091` |
| `t05-legacy-final.log` | `33ca6d85e38a169df8794092b94a1500c8913c4b8e377d4db94e7dcab0e92648` |
| `t05-missing-final.log` | `471911e6054b9caaa9f2b2bfd7ae67c99e90126f0c46fbe74daeabf8e3466f4f` |
| `t05-plain-final.log` | `dbcbc6c5c0c30f7bdf144c0509ec411f77b490934ad187b112fa477fc462d385` |
| `t05-runtime-unit-complete.log` | `931d4b245a559dde3d3823df3b2d9a5ac64de3b148d6e49de423a38378fc4a8f` |
| `t05-runtime-nextest.log` | `d3df92bd16d6779ba9be7dadf13efd7df9e999c0b0e1e5d3033772c50c84163d` |
| `t06-generator-unit.log` | `b42076aa046d534ab8b198f5604ce3eb556126fdae260e7ac3f85cf3e7c43e2b` |
| `t06-target-unit.log` | `43732d6c24c5cfee8473befb7a48b11c6d2a320e6bf7bc805df19f25ccdab559` |
| `t06-real-generation.log` | `a150c3e0ac06fdacd6a446ca902a4953e791a3a7e01621f070cf045860bdcd44` |
| `t06-generated-wire.log` | `1412779b154df5b1cbce4002cff4f561a5a7441e099a475811c1331705ec9f82` |
| `t06-tauri-control.log` | `02985fe5fe479713b1e1aa9605d37a488bdebee46a116c20a5f59c7793f9511d` |
| `t06-tauri-dispatcher-final.log` | `190cf7fcf1f9c00039c5ab13987ccdc91730a38f1e2c55263fda58d9f9246754` |
| `t06-language-check.log` | `466fefaeb84bdcc5cdc4ec5102b5637979ba1c47dc0bdb8f045909d28cec53d3` |
| `t05-mechanical-final.json` | `212787b01f776f0b7d4b68f38d0a728d3111dbb33d277af2acbc7e1ea79ba788` |
| `t05-probe-final-build-locked.log` | `c4fd6373cb635c94229fa7cd6bb00d4c3696c2e82eb17aa121cdf31cd6b02f6c` |

Lock SHA256: language `7fc7a69192ff0f2d99295b24a64ae54cedfe7e877a4be2cad43459e02786cde7`; Web `6137ca7d9c1ddad2beec774c6419ff846702621c6017d1e5f62ee38fd804a215`.


</details>

<a id="e890-1f9b138c564b0993"></a>

<details>
<summary>work\t07-handoff.md · SHA256 ed6506c867c35575b3f6a4c590af3644f2949e1a64c6d986ffac87dd00ac87ef</summary>

#### PLAN-890 T-07 consumer migration receipt

User renewed full-chain authorization with /goal 完成计划890; this supersedes historical per-unit pauses. Original trees reused. Plan executing7/12; T-08 in progress, final affected module union NOT_PASSED. No landing/archive/cleanup/remote publishing.

Frozen source commits: language 163d342a6, UI 672955d, Man 9f8ffc2; Web implementation d8c889c78 (T-08 nextest configuration currently uncommitted). Down unchanged c3464d0. Main unrelated documentation/report WIP preserved.

##### Changes and actual results

- Man CLI (including Core manifest) explicitly installs Web; service launch follows the nested worker using typed invocation context rather than the legacy global active-config slot. Both real CLI service tracks return exact JSON, report the expected port0-derived bound address and config hash, accept control shutdown, exit0, and release the port. Rust executes the actual release generation/build; final cold run 298.77s, not a speed claim. Its new permanent test is explicitly heavy/ignored by default and was executed with --ignored --test-threads=1; this does not change any original condition. Its compiler cache is local rebuildable output, not retained evidence.

- UI runtime imports HTTP matching/binding from Web; metadata/VM values stay SDK-owned. UI tools directly use Web HTTP generation and carry typed launch into their execution worker. Real UI VM child exact TCP/shutdown/port release1, original back-proxy typed/missing/bad/query/session-state controls4, original Vue gallery SSE/client generation1 pass. Non-HTTP Tauri stays on original facade.

- cargo check -p auto-lang -p auto-api-gen, UI tools check and Man Core-profile check pass. Man targeted nextest5/5 includes original non-UI occupied-port rejection, workspace helper, argv reset and unchanged daemon roundtrip plus CLI VM service. Actual Core Cargo metadata normal/build closure534 packages; UI/iced/Tauri/legacy auto-lang forbidden packages0. Core workspace lock refreshed too.

##### Failed attempts retained

Initial Man check found five old consumer references (AxumGenerator and Web-owned serve-state helpers); direct owner imports fixed them. Initial Rust fixtures used tempfile dot-prefixed package names; adding pac name did not fix the directory-based member naming, then an alphabetic tempfile prefix corrected the invalid fixture. A later real Rust compile rejected private nonce; fixture now uses the public API required by the existing Rust generator. One cold attempt ended exit1 without a completed harness receipt and is not credited. Final successful source is committed; all original failed logs retained. Newly exposed errors are not called baseline reds. Existing Core/Web/UI/Man/facade compiler warnings remain recorded; no repository-wide warning-clean assertion.

##### Boundaries and next

This receipt verifies the changed composition and direct consumers. It is not the T-10 fixed Blog cold CRUD/restart or Vue/native application certificate, T-08 ID/atomic conservation/four-owner certification, final independent review, or frozen module union. T-06 shared/Tauri controls are reused only for unchanged algorithm/source/config inputs; UI tools still call the unchanged non-HTTP facade. Legacy full application/tool orchestration will be replayed through the affected Blog consumer in T-10. No required obligation is retired or converted into debt here.

##### Local evidence identities

| File | SHA-256 |
|---|---|
| t07-root-check.log | 3dd76a3a77dc380b1ee2d0fa872a15e450b0e4c54ec2d9b2629f063d90418702 |
| t07-man-check-fixed.log | 3dedf36bb036f2f9c1551a67b90c6bee82291d25d537cc1b214058d6e70c0abf |
| t07-ui-check.log | 87b92ee3ad97b090d94744db3efed55a9d9059165e4eb72e13967a77e7640137 |
| t07-man-targeted.log | 9d8a288978c0093776556cb167f0ab3fba68cb83963d3bc55d436e1a9fd18139 |
| t07-cli-rust-test-public.log | 57a317196f73ec97d252d9ea8bb5f7711cb26c45899c7fe540045223306c14c8 |
| t07-ui-vm-test.log | e4967328ee24257b6b68adfb9df80ae7bd951d3a4a3d6e9475b43718311fa276 |
| t07-proxy-targeted.log | 6237cab8a5affdf2bae5513a81da5dc74c21ce9991f85ab8882c19f14aea6502 |
| t07-ui-gallery-test.log | 6df583d92f361ddc3b7f55f512c4254b02055eaca092406824e236242aea5d71 |


</details>

<a id="e890-6f447d9485de823a"></a>

<details>
<summary>work\t08-handoff.md · SHA256 e371c90d3e9278a6974bf1899f2991c9787002e8b5ce08fc038af8901bfc11e1</summary>

#### PLAN-890 T-08 ownership and execution handoff

Status: T-08 implementation and bounded verification complete; final four-owner actual batch is T-12-owned and NOT_PASSED. Language code `38f68d8e9`; Web `f4cc93c`; UI `672955d`; Man `9f8ffc2`; Down unchanged `c3464d0`. Next: T-09 in the same authoritative group.

##### Preserved obligations and inputs

- The original six manifest hashes retain 696 historical Web-family rows / 116 distinct IDs: 88 moved to Web and 28 language ABI/lowering/compatibility consumers remain in language. The namespace migration contract preserves 117 IDs including the two originally ignored generation tests. No retirement, assertion reduction or changed ignore condition is authorized.
- Seventeen original input hashes are checked: three 885 fixture files move byte-for-byte to Web, fourteen language standard-library/compiler inputs remain. Old duplicate fixture paths are removed.
- Ten original UI photo tests moved with the previously implemented neutral photo service to auto-media-core. Their original UI Git blob/hash, exact target IDs and applicability now participate in catalog verification and actual media-unit execution. Frozen UI floor 3117 is unchanged; the old UI leg's 3110 plus ten conserved migrated obligations is 3120. Counting is conditional on the complete media leg being selected and executed in the same final union.
- Only language and the explicitly registered sibling Web recipes may contribute module source paths. Cargo source owner checks reject foreign valid packages, reverse implementation dependencies and an impure contract package.

##### Actual bounded verification

- Web standalone library: 217 passed, two original ignored tests not executed; 190.996 seconds. Source and original assertions remain unchanged by T-08.
- Actual module runner, Web runtime/generation and initial governance: three complete modules / nine passed; 115 seconds. Runtime and generated Rust checks use real TCP, exact responses and shutdown/port release.
- Retained language consumers plus governance: eight complete modules / 36 passed; 103.107 seconds. Of these, 28 are retained language tests and eight were the then-current governance controls.
- Neutral media library: 54 passed, zero skipped; 49.604 seconds, including all ten moved photo tests. t08-media-unit.log.
- The complete photo `#[cfg(test)]` subtree matches the original UI Git blob byte-for-byte after CRLF normalization, SHA-256 `208bb5e410f943c9b62ffcb93632e3500501aadf0822f809f9844819200a421b`; assertions/fixtures were not replaced by new equivalents.
- Current ownership controls: nine passed; t08-media-governance-controls-final.log. Includes missing/suppressed photo identity rejection. Earlier existing governance/runner controls: 49 and 26 passed respectively; later combined 34 passed before the added photo control.
- The ten changed Rust files pass scoped rustfmt checking (t08-scoped-fmt.log, exit 0); staged diff whitespace check passes. Whole auto-lang package formatting also inspects untouched examples/integration tests and reports pre-existing formatting differences (t08-lang-fmt.log); this is not presented as a package-wide clean result, and unrelated files were not reformatted.
- Retained Plan736 drain first timed out at its original 120.046-second deadline with a missing explicit Web adapter diagnostic. This was a migration regression, not baseline. The common test host installers were repaired at seven actual server fixtures; root test helper visibility was corrected after an E0603 compile failure. Drain passed in feature and default profiles in approximately 2.7 seconds; six representative sibling consumers passed. Production SDK explicit installation policy remains unchanged.
- First four-owner list-only receipt correctly failed: UI selected 3110 below unchanged 3117 floor. Exact baseline comparison found the ten photo tests already moved to media. The original failed receipt remains t08-four-owner-listing/receipt.json; a replacement listing must not overwrite it. No listing is actual execution or a complete delivery receipt.
- Final discovery covers 46 recipes / 7,320 obligations / 1,211 leaves, effective code identity `2eb1c71e5318d0392759fd0136014c29679271d6a9a6dd20dde74a74fc5f4536`. Final four-owner list-only completed with stable before/after identities and all original ID/floor checks passing. Language selected 3095+115 moved, UI 3110+10 moved, tt 3482+115, tb 2985+115; Web declares 219 library (two original ignores remain), 2 runtime, 1 generation; media 54; Man t/tcore/tc/ti 281/24/252/43; UI tu/ti/tn 864/28/2. The manual tn cases are enumerated, not executed.
- This real listing produces outcome `partial`; validating it as delivery raises `partial/failed receipt cannot certify four-repository delivery`. The public fixture negative returned exit 2 with 9 accepted source checks, one rejected gate and distribution pending. A partial/list-only receipt cannot count as the final gate.

##### Local evidence identities

Paths relative to this handoff directory; raw files remain local:

| Evidence | SHA-256 |
|---|---|
| t08-web-standalone-execution.log | d62713ce58948dc71bee90a05a3fdc14442743240dcf4c7d24ba3ffa81d00873 |
| t08-media-unit.log | 10a60633261c56652cd50892ff42223ac38a35e90fdcddbc2ad14b275bc7a1be |
| t08-media-governance-controls-final.log | f9ee57affd140ad8b1893b458281b659c9e88184061cf9632e95f334ced0502d |
| t08-four-owner-listing-fixed/receipt.json | 3988fe4dcd62a8e348085703cf69e71bd75149e15a2d5e425961d3f9d5e22d86 |

##### Evidence policy and limits

Raw logs/JSON/binaries remain local under this directory and are not staged. Formal source contracts and catalog shards are governed source files under scripts/test-governance. Final full module union, independent review, distribution, Blog/UI validation and landing remain pending. Existing AAVM failures and original conditional/manual omissions remain named in earlier receipts; no all-green claim.

Retention preview t08-retention-preview.log returned exit 2: global unique evidence data 64,883,446,496 bytes, above 10 GiB, with existing exceptions/holds. Nothing was deleted or claimed within budget. This Plan's then-current attachment scope was approximately 42.5 MB, below its 500 MiB budget; worktree targets are rebuildable caches, not permanent evidence.


</details>

<a id="e890-a64978ff42750607"></a>

<details>
<summary>work\t09-execution-entry.md · SHA256 db7a96bd74f12ecedb67eeb298d7a1b106f03718c899e81f384baa08ac4d103a</summary>

#### PLAN-890 T-09 execution entry

Scope remains AC-09/11 and V-12/13. No remote publication or acceptance change.

The existing Man installer owns install admission, unique marked staging, live-owner protection, backup/swap and rollback. Web packaging must call that same implementation rather than create a second transaction. For a standalone source distribution, the Man-owned tool may be supplied as a generated immutable tool artifact with original repository revision, path and content hash; it does not require the Man checkout or a Man Rust dependency. The source owner remains Man; regeneration must reject drift.

Web owns selection and staging of its tool, pure contract, actual public language SDK dependency closure, source inputs and licenses. The installed SDK may use the existing combined-crates layout. A service requirement must validate the minimal Core/media/shared generation/Web closure against its receipt, while UI requirements retain the current full SDK checks. A missing or damaged authoritative managed SDK cannot fall back to an ambient checkout. Versions and source/resource identities are checked before touching an existing generated workspace.

Required actual consumer states: independent source payload without UI/Man/Tauri; new target build/test; external generation through the public Web API; fresh Cargo resolution; locked replay; A-to-B relocation without A/old target; exact TCP result and shutdown; legal old three-owner package and new four-owner package; failed missing input/hash/version and interrupted generation; failed installer swap; recovery followed by another actual generation/runtime use. Previous generated/user bytes are hashed before and after each negative control. Logging a recovery action without subsequent use is insufficient.

The packaging runner must reject a partial result, changed source cohort, missing required scenario, zero actual execution or unmatched runtime producer. Build caches are not copied into evidence or release payloads. Final raw evidence remains local with the Plan retention budget; the permanent receipt records commands, exits, key data, artifact paths and hashes.

Known concrete consumer findings from preflight: Man's installer currently lacks Web source recognition/path projection, still copies retired language-owned autoui-skill instead of its canonical Man source, and does not stage the new shared Core CLI Web test target. Its old Core manifest has exactly two owner identities; the upgraded Core must add Web when Web packages exist while still accepting a genuinely old two-owner install for recovery. These are in-contract consumer adaptations, not a new installer architecture or changed compatibility promise.


</details>

<a id="e890-4d314850cd8b40e6"></a>

<details>
<summary>work\t09-handoff.md · SHA256 47c86816a5b8feddb284b00d5d3e629f47a81f27dd4f580e602d96afa0058a46</summary>

#### PLAN-890 T-09 distribution and recovery handoff

T-09 complete within its contract. Overall Plan remains executing; Blog/UI work, independent final review and frozen affected/four-owner gates remain pending. No remote publication.

##### Authoritative sources and package cohort

Implementation stays in `D:/autostack/.wt/lang-890/{auto-lang,auto-web,auto-man,auto-ui}`. SDK runtime inputs are language `87a09af94` (SDK fix `50a61a08d` plus delivered main synchronization), Web `9ddb7fe`, Man installer `cb4ab3d`; UI `672955d` and Down `c3464d0` unchanged. Later language changes in this unit are verification/governance scripts, not SDK crate/Cargo/stdlib input changes.

Web has 19 actual local SDK packages and no UI, Man, Tauri, mixed auto-api-gen or auto-lang facade dependency. Its self-contained source payload includes the fixed SDK source closure, full standard-library source, fixture inputs, Cargo lock, licenses, source ownership, registry package/license inventory and an immutable Man-owned installer artifact. That artifact comes from real committed Git blobs; its bytes, commit/path and license hash are checked. Generated Python caches are ignored and absent from the final Web commit.

Man remains the sole owner of install admission/staging/swap/recovery. Web invokes its immutable tool artifact with strict Web payload callbacks. Failed post-swap verification now restores the previous installation and preserves the failed new payload. Core/full packaging recognizes Web, canonical Man skill resources and every declared shared Core CLI test. Six real filesystem transaction controls pass. A staged 32-package Core SDK also compiled successfully.

Shared source validation accepts minimal managed service SDKs while retaining Full-only UI validation. It checks the complete owned SDK tree, Web/Core package identities and source revisions. Explicit/adjacent SDK stdlib selection is authoritative and missing inputs cannot borrow ambient stdlib. The isolated permanent source-root test covers language root, crate override, delivery root, adjacent SDK and missing-source cases. Shared generation library: 16 passed; source-root case: passed (six subprocess states).

##### Actual delivery verification

Frozen source payload: `candidate=false`, source identity `22310a57b4aa0a2bf608b038ba4efece8d8272f91da9197c06853b5c3662dc68`.

- Self-contained SDK mandatory library execution: 217 passed, two original ignored tests preserved; 207.962 seconds. Earlier repaired-source execution also passed 217/2 in 314.566 seconds.
- Frozen external distribution: **45 checks passed, zero failed/pending**. Actual new-target install, external fresh lock and locked build, public Rust generation, exact VM/generated-Rust HTTP results, shutdown 202/exit 0/port release, A-to-B relocation with A absent and a new target, controlled interruption after the public workspace phase plus recovery and real consumption, wrong/missing version, missing managed receipt, changed managed source hash, missing bundle resources/hash/version, failed swap preserving the exact old install, successful upgrade and another real generation/runtime use.
- History/compatibility: **14 passed**. Real old Git snapshots language `674871e2...` and Man `e6cc06fb...`, plus the current fixed cohort, were compiled. The valid older Core install has two owners; the upgraded Core has Web as a third owner. Prepare/verify/swap failures each preserved the exact old package and user content, then actually served again. UI pin `bce23b0...` is observed only: this is the old three-owner architecture's supported Core shape, not an old Full/UI runtime certification.
- The public fixture accepts ten current source/distribution checks and reports the final four-owner gate pending; `--require-complete` correctly returns 2. Distribution validation independently checks all 45 required steps, actual logs/hashes and exact runtime outcomes, SDK source cohort/files, and the revision-bound old/new archive/build/recovery proof. Ten ownership/distribution controls pass, including accepted protocol control and missing/duplicate/zero/unexecuted/wrong-runtime/wrong-hash/unfrozen counterexamples.
- Delivered responsibility metadata was synchronized from main. Eleven old top-level leaf references were reconciled with migrated source ownership; 74 responsibility groups cover the existing 1,211-leaf catalog exactly. Original Core count floor 1600 is retained by counting its 115 actual, source-cited Web destinations. Nineteen purpose/conservation controls pass. Purpose suggestions do not change selection/admission. The mixed generation group is pending only for future daily-admission advice; all 890 required tests still execute. Full discovery must refresh after all T-10 changes and newly delivered 892 tests; no current whole gate is claimed.

##### Independent Git checkout and retained failures

A real local Git clone at Web `9ddb7fe` consumes only a fixed language SDK export at `87a09af94`; actual metadata has exactly 19 local packages and UI/Man directories absent. CLI locked build passed. Its full library execution had 214 passed / three MSVC linker output-path failures / two original ignores. Source, assertions and thresholds were unchanged when moving this isolated fixture to the shorter normal temp path recorded in `standalone-git-short-location.json`. All five selected checks passed, covering the three failed wire matrices and two input/root consumers. That replay recorded one transient nextest LEAK for Plan776; isolated replay then passed in 18.466 seconds without LEAK and all three fixture ports were explicitly rebound. Retain both original results; this does not certify arbitrary Windows paths or all-environment absence of process leaks.

First source-payload execution had 205/12 failures/2 ignored: missing full stdlib, a virtual-facade source-selection assumption, and nested fixture Web paths. These were repaired rather than labeled baseline. The first Git-export setup also omitted its required language Cargo.lock; the corrected export preserves that input. Raw logs are never overwritten.

D: filled during an early diagnostic external build, which failed and remains a failed receipt. Automatic approval review rejected both bulk and single verified Cargo-cache deletion with `blocked by policy`; nothing was deleted. Validation continued in a unique C: workspace. D: later recovered space; do not attribute that recovery to this session's deletion. Build caches remain rebuildable and are not release/evidence payloads.

##### Artifact and receipt identities

All paths below are relative to `C:/Users/zhaop/.codex/.chatgpt-projects/g-p-6aa9047652808191a99340cce4efab3b/plan890-external-validation-20261031/`. Raw artifacts remain local. The two release ZIPs total 22,986,271 bytes, below the per-plan 500 MiB budget; temporary build/source caches are not copied into release payloads. Existing global evidence budget was already above 10 GiB; no budget-green claim or protected evidence deletion.

| Local artifact | SHA-256 |
|---|---|
| auto-web-0.1.0-source.zip (3,590,311 bytes) | 9987aaff9a6b4a78e782d6a06b15f1efe91b8a6bcab7e3a44b60d6e9e65106db |
| auto-web-0.1.0-windows.zip (19,395,960 bytes) | 2be25dc39faaa88c86963f28568b224f197462c8152acf0aed0ec61941543de6 |
| replay-frozen/distribution-receipt.json | 52d67f31bb98a9dc4cebe812548776dd1730759393185a5fb001a7348b690027 |
| replay-frozen/independent-tests.json | 89b7280e191c6ddd8e9e42aa580cffe1de1d76c630032bb94e0e3f81586d7a28 |
| legacy/legacy-receipt.json | db984742e1167f1f205135c9d8e8ac76b392e75dfed88f5e27a60c0ef0c279c1 |
| source-frozen/source-manifest.json | aa7e984487abd41c85eb085bd02cc89cb63210141821ac4da791348f4db8f1a7 |

Warnings remain inherited: Core 128, Web 13 in the relevant normal build; generated trivial-service imports produce four existing template warnings. No clean-all-warnings claim. Whole-package formatting had untouched pre-existing differences; changed Rust files and staged whitespace checks pass. T-10 owns the final warning/debug/defer scan and proposed Spec delta; T-11 owns independent acceptance review; T-12 owns refreshed frozen gates/landing/knowledge/archive/guarded cleanup.


</details>

<a id="e890-fb122a733a77831c"></a>

<details>
<summary>work\t10-execution-entry.md · SHA256 b498fd558e2eba1935543a9c58288022b6aed163c45e8f235aad05405503ed7c</summary>

#### PLAN-890 r6 T-10 execution entry

Authorization: the current full-chain goal; same five authoritative sibling
trees, no implementation agent and no business-WIP edits. Final review/landing
remain later stages.

##### Fixed consumer source

Blog source is the real Git archive of
`b42a660dadef253a85837a750365a6e93875be28`, containing the delivered BLOG-004
acceptance harness. It is exported to
`C:/Users/zhaop/AppData/Local/Temp/blog890-b42`; the primary business checkout
has advanced independently and its WIP is excluded. Archive SHA-256:
`ecac7f8ade1f380728fa4559b59bf3dd73b5a471e4f3a5fd06e841153a0195f5`.
The external driver only selects the actual Man/tool/source roots and replaces
the archive's unavailable Git HEAD observation with its verified source pin.
It does not alter scenarios/assertions, business source or generated Rust.

Fresh generated workspace and isolated `AUTO_BLOG_DATA` live under that export.
Man's real `auto service --server rust` cold-generated, release-compiled and
served the backend: 13 original API/storage checks passed, including Chinese,
quotes/newlines, 4,690-character content, revision conflict, malformed inputs,
isolated durable data and restart. That is a filtered backend result, not the
complete Vue profile. Raw driver/report: main checkout
`docs/plans/attachments/890/work/t10-blog-api*`.

##### Environment and next actions

The actual Man CLI initially exhausted D: at link-time PDB output. Relinking
the same compiled objects without debug symbol output succeeded; no business or
test profile/condition was reduced. The UI tool was built from the authoritative
UI source into a fresh C: target (development debug information disabled for
these implementation tool/unit builds). Generated backend uses its original release mode.
Formal module discovery/execution keeps its original build profiles/conditions;
the new G: target is only the Cargo artifact location.

Original full Vue starts failed before readiness because the configured npm
mirror returned 404 for the Windows Rollup 4.67.2 archive. A plain environment
registry override was ineffective in the hosted child. Explicitly installing
the same generated manifest with pnpm's official-registry option succeeded;
the original formal CLI/profile then reruns. First failures remain intact.

Before final freeze, synchronize the UI main branch's already delivered changes:
master `69690df` contains 31 newer commits, including PLAN-015 native page-store
dispatch/style fixes and PLAN-013 experimental HUD work. No new architecture or
experimental full-suite scope is added. Current preliminary discovery must not
be certified against a subsequently changed source cohort; refresh after the
actual final sync and preserve both receipts. Primary UI has docs-only WIP,
which must be preserved during eventual landing.

Health audit found one trailing blank line in the newly added migration
inventory manifest, now removed. Web's three generator TODO strings and two
test-input `todo!()` strings are present with equal counts in the original
language source at `1a253950810de67f16f487c7eb7a2745f1061bb0`; they are inherited,
not newly deferred Plan work. No new TODO/FIXME/debug marker was found in the
other three repository diffs. Full candidate health/SD/AC readback remains open.
Proposed SD-01..09 deposition is in `t10-spec-delta-proposal.md`.


T-10 repair is committed at UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`,
after synchronizing delivered UI master `69690df` through `b8fe063`. Final
implementation verification ran the original single-process `ui_gen::` owner
module: 866 passed, two original ignored, four new controls; existing assertions
were not changed. This implementation run used the stated C: development
debug-symbol override; it is not the later default-profile frozen owner gate.
The initial local-renaming diagnostic closed all seven pre-restart browser
tasks but its full profile had two restart failures with an actual esbuild
Windows directory-access error. Source was refined before freeze to preserve
local names and emit state aliases only on actual shadowed use; this also keeps
existing return-value output and delegated captures. No generated Rust or
original fixed Blog file was edited. Both final profile replays remain required.
Two profile-specific SQLite cache outputs lacked bindgen.rs: the exact 16-byte
Cargo build-script fingerprint markers were renamed and preserved, allowing
Cargo itself to recreate those missing outputs. No dependency/source/profile
change or broad cache deletion was used for that repair.


</details>

<a id="e890-6aafd921a0a45c59"></a>

<details>
<summary>work\t10-handoff.md · SHA256 c8fa25a6dbbf65502334149482214c52d3d39d1be8f478986b4879a47b9bd792</summary>

#### PLAN-890 r6 T-10 implementation handoff

Work tasks T-01..T-10 are complete; independent final review T-11 and frozen
execution/landing/deposition/archive/guard cleanup T-12 remain pending. This is
implementation self-audit, not independent review or U890. No remote publication.

##### Authoritative candidate and identities

One original sibling group: `D:/autostack/.wt/lang-890/`.

| Repository | Commit |
|---|---|
| language (implementation/governance) | `ede598998566ea890ff67170bb9744f3bc537a58` |
| Web | `9ddb7fe5bb1886166073b5805c38895f3da195ff` |
| UI | `9aad12ccbc5f01d5241eb95f657666a943667ff2` |
| Man | `cb4ab3dd5d411314cce10b129eeb3b433a11c466` |
| Down (fixed dependency, unchanged) | `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` |

UI synchronized already delivered main `69690df` through merge `b8fe063`, then
committed the necessary store-snapshot repair at `9aad12c`. Primary repository
business/docs WIP was preserved. The language docs handoff commit may follow
this source commit; it does not change implementation inputs.

Current code/config/environment identity:
`0bb28657a28e2405d21bf42da70bd3b47ab99c45d773aae8617ad0d5ce981bb1`.
Discovery enumerated all 46 recipes: **7,329 obligations / 1,213 leaves**.
The governance changes were present during discovery and then committed without
changing their bytes. The current identity was independently recomputed equal
to the catalog. Metadata HEAD provenance and implementation byte identity are
reported separately. Discovery does not mean those bodies executed.

##### Fixed consumer, real cold build and dual UI

Blog is the real Git archive at `b42a660dadef253a85837a750365a6e93875be28`, in
`C:/Users/zhaop/AppData/Local/Temp/blog890-b42`. Final readback verified **all 314
original files unchanged**. The business checkout and its WIP were not copied
into the consumer. No generated Rust was manually edited.

The real Man `auto service --server rust` cold-generated and release-compiled the
backend (2m25s); all 13 API/storage/restart checks passed. The final independent
E: cache cold-build used the original development profile and unmodified generated
Rust: `cargo build --locked --manifest-path rust-workspace/blog890-b42-back/Cargo.toml -j4`,
exit 0, **2m31s**. Full UI profiles then reused those compilation artifacts, as
the original harness permits. This is not a claim that a completely cold GUI
startup fits the unchanged 120-second internal provisioning deadline.

Final commands from the fixed consumer directory:
`python run-plan890-acceptance.py vue-rust` and
`python run-plan890-acceptance.py native-vm-rust`.
The external driver only supplies authoritative source/tool/data/build roots,
the verified archive identity observation and `BROWSER=none`; it does not change
scenario assertions or business source. Runtime `CARGO_PROFILE_DEV_DEBUG` was
unset for these final profiles; target is the fresh
`E:/autostack-build/plan890-blog-runtime-20261031`.

| Actual original profile | Result | Raw report / SHA-256 |
|---|---|---|
| Vue + generated Rust | **15/15**, zero failed/pending | `t10-blog-full/vue-rust-20261031-043454/report.json` / `7de6707cb66847ee16405a4a6119fec96189ac6cebf9270e575953edf94d0833` |
| Native VM UI + generated Rust | **22/22**, zero failed/pending | `t10-blog-full/native-vm-rust-20261031-043927/report.json` / `7801ea4da39104ebb04502a4ed635582bdb9e3749bce17baaae3092cbb25be30` |

Vue's author-task aggregate contains seven actual browser checks, plus the
separate restart readback: hard Chinese/quote/newline input, 3s autosave,
independent HTTP readback, title save, visible stale-revision conflict,
keep-local recovery, refresh/reopen and backend restart. Native checks additionally
observe exact durable body hashes/revisions, genuine owned windows/MCP, normal
q exit 0, owned-child exit, forced exit, closed MCP, reopen and another save.
Both profiles exercise Rust backends; independent VM/Rust service parity remains
the separate T-05/T-07 evidence, not inferred from dual UI.

| Final executable | Bytes | SHA-256 |
|---|---:|---|
| Man `auto.exe` | 139501056 | `a3b8d0c396f324c169ddf685b9fdf5fd9ae2329e18759cb91be0753c68a15d6b` |
| explicit UI tool | 130664960 | `604f592ed27446c6ab13575e7018c9266a7bfd63aa928c01a97452fdb4a03479` |
| generated Blog backend | 31456768 | `e1236139cc90c3013f42ac70245923c8e0912b5746f3d7e37224bd286d2f3d32` |

The UI tool and implementation unit runs used C: builds without development
debug symbols; assertions/features/conditions were unchanged. The final default
owner gates remain merge-owned. Man and the final generated runtime used their
original development configurations. File paths, hashes, reports and source
identity are indexed locally in `t10-evidence-index.json`.

##### Necessary UI owner repair and controls

Actual UI PUT requests returned 422 while independent API input passed. The
existing emitter treated `snap_body`/`snap_rev` locals as same-named model refs,
therefore omitted primitive body/revision fields. The repair stays in the UI
compiler's two existing owned files: lexical binding scopes distinguish locals;
user local names remain stable for delegated function/lambda captures; state
aliases are reserved and emitted only for actual shadowed uses. Same-name
initializers avoid the JavaScript temporal dead zone. Public getters and ordinary
non-shadowing return output remain compatible; no AST schema/runtime protocol or
new repository/dependency was introduced.

`cargo test --locked -p auto-ui-compiler --lib ui_gen:: -- --test-threads=1`:
**866 passed, 2 original ignored**, four added controls; existing assertions were
not edited. The initial nextest attempt correctly failed with zero selected tests
because ui_gen belongs to the owner's single-process tu lane. An early test-only
fixture parse failure was repaired. An all-state-alias prototype exposed the
existing return-value assertion; production emission was narrowed rather than
changing that old assertion. The local-renaming diagnostic passed all seven
browser author tasks, but its full profile retained two restart failures with an
actual Windows esbuild directory-access error. The committed final implementation
and the final 15/22 profiles supersede no raw red result; all originals remain.

SDK/source packaging, dependency/fixture materialization and cache failures were
kept distinct from product defects. The configured npm mirror returned 404 for
the same Windows Rollup archive; official-registry installation retained versions.
The Git Blog archive omitted its documented prebuilt engine dist: rebuild used
a real fixed Down `c3464d0` Git archive, **178 original engine source files exactly
equal**, original full build/assertion chain, then 154 built resources with stamp
`f46a2f188e86faab` (full stamp/file hashes in the resource manifest). Reka 2.11.0
had 10 missing installed files; all 3,081 files were checked against its exact
same-version official tarball, only missing bytes restored. Neither dependency
versions nor application source/expected values were changed.

Several C: Cargo build-script fingerprints outlived missing SQLite `bindgen.rs`
outputs. Exact fingerprint markers were renamed/preserved so Cargo regenerated
them; those original failures remain. Final runtime verification instead used a
fresh E: cache and original profile. No broad shared-cache deletion occurred.
D: Man-cache NTFS compression retained 32,605,127,461 logical bytes in
15,594,190,572 physical bytes; the executable hash observed before/after compression
was unchanged before the later intentional rebuild. Prior rejected cache deletion
attempts were not retried by another deletion method.

The original fixed harness leaves Windows Vite preview children when killing its
wrapper. After the tests, **three exclusively owned private-export children** were
identified by exact path/PID/birth time and reaped; ports 13275/4136/6975 rebound
on IPv4 and IPv6. No Blog backend process remained; native owned-child closure
was independently asserted in the 22 checks. This does not claim the unmodified
controller cleans its tree unaided. Source-level controller upkeep is recorded
as 890-N3 under the Blog owner, with the completed local cleanup receipt.

##### Test governance and health

Two actual new leaves are covered: selected-SDK stdlib authority and the already
delivered 892 HIR/Rust channel. Existing shared-generation and infra-Web leaves
absorbed their new cases. **75 responsibility groups** cover all 1,213 leaves.
The 892 leaf mixes registered daily and certification/worker controls, so its
purpose suggestion remains pending; this does not change execution eligibility,
profiles, original ignores, budgets, floors or any required Plan selection.
`python -m unittest discover -s scripts/tests -p test_test_purpose_policy.py -v`:
**19 passed in 61.580s**. The complete affected/four-owner execution gate is still
**NOT_PASSED**, not manufactured from discovery or purpose metadata.

Whole candidate whitespace checks pass in all four modified repositories.
Web package-owned fmt checks pass; the overbroad `cargo fmt --all` attempt also
visited untouched language dependencies and its pre-existing failures remain in
the raw log. Changed UI files were formatted before commit. Same normal tool
configuration warning comparison on both changed compiler files: baseline 4,
candidate 4, **new delta 0**. Existing Core/Web/UI/Man/dependency/generated-template
warnings are not described as a warning-clean whole tree. Man's interrupt-sent
warning is unchanged source from `041043bb`. No new debug/TODO/deferral markers
were added. Web's three generator TODO strings and two test-input todo stubs have
identical counts in the original source at `1a2539508`; they are inherited.

Proposed SD-01..09: [Spec delta](#e890-eba7e5bd9981c181), including the bounded
UI compiler existing-component/plans pointer addition. Canonical deposition is
T-12, not prematurely written here. The T-09 service SDK's 19-package code inputs
are unchanged by the UI repair; the whitespace-only inventory fixture is absent
from that payload. See [T-09](#e890-4d314850cd8b40e6) for the immutable 45/45 distribution,
14/14 historical Core recovery, 217/2 independent library evidence and ZIP hashes.

##### Work-AC self-audit and later gates

| AC | Evidence / implementation conclusion |
|---|---|
| 01 | T-01/T-02 source/consumer/history inventory and approved decisions; original 885 historical guard gap closed by explicit user confirmation, never a fabricated replay |
| 02 | T-02/T-04/T-07 real SDK/Cargo type/DAG and forbidden-edge controls; independent 19-package local SDK |
| 03 | T-02/T-05 explicit SDK host/missing-host/custom-provider/invocation-owner controls; ordinary calls remain provider-free |
| 04 | T-06 HTTP/shared/non-HTTP generation, real compile/run; T-09 external generation/recovery; T-10 real fixed consumer |
| 05 | T-07 genuine Man VM/Rust and UI worker/proxy/client wiring; T-10 complete profiles |
| 06 | T-05 wire/policy/strict-config/negative matrices and T-07 real service results; no enlarged platform/support promise |
| 07 | T-02/T-05 owned cancellation/body/reader/file/upload/network/logger completion; T-07 shutdown/exit/port; native genuine closure and separate test-preview cleanup |
| 08 | T-08 source-cited IDs/resources, independent execution and genuine list-only rejection; refreshed source-bound 46-recipe partition and 19 controls; full frozen owner gate remains T-12 |
| 09 | T-09 real independent SDK 217/2, distribution45, history14, fresh/locked/relocation/failure recovery and real reuse; Windows path/LEAK first results retained |
| 10 | Fixed source 314 unchanged, real cold compile/API13 and final Vue15/native22; actual UI defect repaired at its owner |
| 11 | T-04/T-06 single domain owner/original exit and named compatibility/non-HTTP/shared sources; T-09 recoverable old/new valid Core shapes |
| 12 | Owned health checks/new-warning delta0, all raw reds/limitations retained, actual SD proposal and version/input handoff; no new mandatory work item silently deferred |

This table is a claim to independently verify. T-11 must use an independent
context on these same authoritative trees, check every work AC/V/SD against
actual source/evidence and add distinguishing counterexamples. Do not make a
second checkout or run implementation agents. T-12 owns final affected complete
module/consumer union, real four-owner gate, any due main-only batch, actual
landing in each modified repository, canonical Specs/required ledger/index,
archive and per-tree clean guard/removal. Three historical AAVM reds and original
conditions stay named in the earlier receipts; no final gate PASS is claimed.
Raw evidence remains local, only deliberate Markdown summaries are committed.
Default retention after final review/landing is 14 days; hashes are identity
records, not a promise that all raw files remain replayable forever. Cache trees
are rebuildable workspaces, not promoted durable release payloads. The existing
global evidence budget overrun remains explicit and unclaimed as green.


</details>

<a id="e890-eba7e5bd9981c181"></a>

<details>
<summary>work\t10-spec-delta-proposal.md · SHA256 b2a47e3a5891e79b25bcf842462bdbc33aa222117c373355c9b192aed47e7e9e</summary>

#### PLAN-890 r6: final Spec delta proposal

Status: proposed for independent review, not deposited canonical knowledge.
The merge stage must read back every deposited path in each actual delivered
repository. A proposal or a local passing probe is not U890.

##### SD-01 / language auto-web pointer

Add `docs/specs/auto-web/project.md`: AutoWeb 0.1.0 is a separate sibling Git
repository. Canonical Web domain prose lives in that repository's
`docs/specs/{project,architecture,http-contracts,testing,delivery}.md`.
The language pointer describes the dependency boundary and links the central
archived Plan and four-repository delivery contract. It must not reproduce a
second normative Web contract. No remote repository, push or registry release
was performed.

##### SD-02 / four-repository delivery

Add `docs/specs/repository-boundaries/web-four-repository-delivery.md`:
language/Web/UI/Man are required owners; Down is a fixed dependency, not a
replacement owner. Source, locks, resources, producer identities and exact test
selections are bound before/after. The runner requires actual execution of all
required legs; a list-only, filtered, missing/duplicate/zero, stale-hash or
partially executed receipt cannot certify the full product. Source migration
contracts retain original identities, assertions, conditions and resources.

The staged managed SDK validates its receipt, complete owned file inventory,
versions and source cohort before replacing an existing workspace. Service-only
generation accepts the Web minimal or Man Core/Full shape; UI generation still
requires Full. Explicit selected or adjacent managed SDK failure cannot borrow
ambient sources. The source payload carries 19 actual packages, complete stdlib,
necessary fixtures and licenses. User content and the old valid installation
survive prepare/verify/swap failure through the Man-owned installer transaction.
Web carries a byte-exact, Git-provenanced artifact of that transaction owner;
it does not own a second editable installer algorithm.

##### SD-03 / language HTTP host port

Add `docs/specs/language-sdk/http-host-port.md`: Core owns compiler metadata,
values/conversions, VM and native registration, context/resource hooks and the
neutral local host interface. `HttpServiceHost: Send + Sync` selects a provider;
the returned `HttpServiceFuture` and `Rc<AutoVM>` remain local and !Send. Owned
request/reply/body messages cross to network ownership. Core starts no implicit
Web framework. Existing listener grades remain distinct.

New direct SDK service consumers add fixed Web dependency and call
`install_http_service_host_if_absent`; this preserves an already selected custom
provider. An explicit setter is the replacement operation. Ordinary language
calls need no host. Missing host for service calls rejects before listening.
Active VM owners preserve the provider and invocation configuration snapshot;
typed launch travels across worker threads, and each future poll restores its
own context. Cancellation/drop and network/logger drain retain actual resource
completion obligations. Man/UI install internally at their composition points.

##### SD-04 / generator, SDK and media ownership

Update auto-api-gen and language-sdk project/architecture: HTTP Rust/TypeScript
targets, service/client generation and HTTP templates live in Web. Tauri IPC
and non-HTTP adapters retain their original owner. The existing four shared
mechanisms (`rust_workspace`, `shared_generation`, `sidecar`, `source_owner`)
have one restricted source in auto-generation-core; there is no copied
declaration/type-lifting/lock/freshness algorithm or Web-to-facade reverse edge.
The inherited PLAN-885 runtime path, import/type/borrowing and epoch contracts
remain in force. Renderer-neutral photo implementation and its ten original
tests live in auto-media-core; the old UI path is only a re-export.

##### SD-05 / HTTP language contracts

Update the HTTP server/deployment/backend-assembly Specs with actual owner and
public entry points, explicit SDK assembly and typed launch snapshots. Preserve
the supported VM/Rust routes, response/error/stream/file/upload semantics,
strict config, security/budget and listener grades. VM Axum 0.8 and generated
Rust Axum 0.7 stay distinct tracks; no transport-specific types cross the port.
Do not imply new production platforms, TLS/auth/telemetry products or unchanged
arbitrary Windows long-path support.

##### SD-06 / repository, Man and UI consumer rules

Update repository-boundaries, auto-man and auto-ui-tools project Specs and add
each consumer repository's `docs/web-integration.md`. Man owns CLI/engineering,
dependency/process orchestration and generic installation; UI owns its renderer,
tool/worker/proxy and non-HTTP behavior. Both directly consume fixed Web source
and compose the adapter. Core has no Web implementation dependency; Web's normal,
build and required independent dev closure has no UI/Man/Tauri implementation.
Record the actual landed cohort, supported commands, recovery procedure and
test entries, linking Web prose rather than copying it.

##### SD-07 / governance

Update auto-test-case governance: old/new test identity and resource contracts,
Web runtime/library/generation recipes, media migration, four-owner full gate,
and their failure controls. Purpose classifications are advice and do not change
execution eligibility. Preserve frozen floors through source-cited migrated
obligations, original feature/ignored/platform/milestone/manual conditions and
the first red results. New catalog discovery and complete module/consumer union
belong to the frozen final gate, not a claim from old catalog counts.

##### SD-08 / Web canonical source

Create the five canonical Web pages listed in SD-01, grounded in actual exports,
ownership, package/test manifests, SDK/distribution scripts and final receipts.
Update the existing README/architecture/testing staging text to point to them.
Retain provenance/history pages. Separate delivered behavior from manual,
milestone, environment-limited and unexecuted support. The actual fixed Blog and
dual-UI results must be recorded from their receipts, not anticipated here.

##### SD-09 / consumer knowledge

Consumer integration pages record typed launch propagation, VM worker and Rust
generation/proxy paths, unchanged CLI service/run behavior and explicit tool
identity. Archive/ledger/module plans indexes are merge-owned; update only the
actual affected modules in the actual delivered repositories. Preserve unrelated
main-checkout documentation work.


SD-09 bounded correctness addition: update the existing language-side
`docs/specs/auto-ui-compiler/project.md` and its module plans pointer. Vue store
handler emission distinguishes actual lexical local bindings from module refs.
Local snapshot variables retain their names and primitive values; explicit
state references use a reserved internal module alias only where shadowed and
actually used. Block/loop/closure/catch scopes restore enclosing bindings,
same-name initializers preserve access to state, and delegated function/lambda
captures retain their original local names. Public store getters and ordinary
non-shadowing output remain compatible. Generator fingerprints already cover
these owned source changes; no AST schema or runtime protocol is added.

The fixed Blog exposed this through real PUT 422 despite independently legal
API calls. Final owner module verification: 866 passed / two original ignored;
four new controls supplement the existing assertions. Original failures and
application/source/binary identities are in the T-10 handoff; complete Vue and
native profile results must be deposited only after their actual execution.


</details>

## 10. 待澄清事项

> 最新全链启动与T-07交付见§9续记；历史暂停条款已被本次 `/goal 完成计划890` 覆盖。D-01旧885启动缺口及D-04语义选择已关闭，不重复询问。

| ID | 问题 / owner与关闭门 |
|---|---|
| D-01 | 885最终guard收尾锚、888/889当前权威合同/接受窗口及目标仓身份：T-01重核，不代补他方收据、不占用已用号，不据滞后状态头认定无WIP |
| D-02 | T-02 r3可行性部分已闭：owned端口/线程/资源与launch/session有界实证；实际迁仓/消费者和最终组合仍T-04/05/07，不把Axum Body当中立类型 |
| D-03 | T-02已冻结受限auto-generation-core四模块/25函数与consumer映射；T-04/06实际迁包，禁止循环/复制 |
| D-04 | **用户已裁定，语义阻塞关闭**：显式安装Web adapter，按r2完成直接SDK版本迁移、缺席诊断、普通语言兼容及Man/UI宿主装配；全路径技术门仍属T-02，不等于已验 |
| D-05 | **已关闭（T-10）**：固定Blog b42a660源码314原文件不改；真实冷构建及Vue15/15、原生22/22，有工具/产物hash和独立API/持久化oracle。额外固定引擎资源、缓存与预览控制器问题如实记录；不声明BLOG-004全局U或借业务WIP冒绿 |
| D-06 | T-02 r3已在887正式基线上冻结四仓schema/required字段/拒绝规则；T-08实际共享窗短串行实施，必含Web真实执行，不读888 WIP schema |
| D-07 | 远端仓/registry/CI发布渠道和新增Web产品：不在本轮默认授权；本地独立交付可完成，未来确需外部写入给明确对象与版本另议，不以“发布完成”伪称远端动作 |

### 2026-10-30 执行授权与启动门核查

本次goal已授权890全链，§4.1及顶部“仅规划”是2026-10-29历史授权快照，不再代表当前授权。合同范围、验收标准和前置U门不变。T-01核实885实际树/分支已不存在，但未取得移除前guard clean证据，归档末尾仍pending，故G890未成立、0/12保持drafting，未创建890实施树或Web仓。当前需要的是885原始guard/清理记录的具体路径或原会话工具输出；若记录无法恢复，需要对这项历史证据缺口作明确裁定。不能对已不存在路径重跑guard冒充历史通过，也不重建假885树补历史证据。888无关工作不受此阻塞影响，UI/Man结构包仍等U890。

2026-10-30更新：用户已明确确认885收尾，前述等待记录为历史；D-01中885启动缺口关闭（用户确认，原始guard日志未恢复），继续T-01。当前已有完整执行授权，无需重复许可。

### 2026-10-30 D-04实际兼容决策（历史等待，用户已答复）

旧885历史收尾等待已由用户“885已经收尾了。”关闭。本次实际Core-only外部消费者无需Web装配即自动提供HTTP，已编译、真实精确JSON/关闭验证并提交。与r1无Core→Web实现边+全部Web实现迁出同时保持该消费的静态包图冲突，需选择具体版本迁移方案或兼容门面包图修订；原合同§2.2/§10 D-04要求实质决定，不自动削减支持。两方案、实际调用链、限制和日志hash见[决策说明](#e890-1d38a6122d17860e)，来源与共享窗见[入口摘要](#e890-ab8af97a394bf500)。等待用户输入工具实际答复；当前全部生产代码未改，正式Web仓未创建，T-01/02均未冒完成。

2026-10-30 D-04最新裁定：用户明确“显式安装 Web adapter”。以r2当前合同为准，前述未答复/等待/blocked是历史。下一步work，完整交付范围与所有其他门保持；0/12不冒完成。


### 2026-10-30 T-05～T-06 恢复入口

用户“继续后续阶段”启动r4单元2：按T-05运行时→T-06生成器串行；本单元结束提交/更新计划/交接后暂停，T-07及合入/归档/清理不在本单元。原语言树`e45399a63`、Web`26259642`、UI`7b323bc`、Man`e6cc06f`、Down`c3464d0`均核实clean，复用全部原树；主检出仅既有文档WIP，保全不纳入。完整tm旧收据仍NOT_PASSED。

| 规则/入口 | 实际链路与所有权 | 区分性控制与关闭条件 |
|---|---|---|
| T-05 / AC-03/06/07 | Core编译元数据/VM本线程→HttpVmOwner/owned消息→Web host→Web transport/dispatch/scope/file/upload→真实body终态与network/logger join。现有external-runtime探针已验证端口，当前仍按路径编译Core原件；本轮改为正式Web源，删除Core临时run_existing桥及领域运行时 | 重放audit3/session5、file/SSE/upload/RST/两实例和缺provider/plain控制；编译包图无Core→Web实现或Web→UI/Man/Tauri，检查无旧运行时副本；阶段新增destination测试不替代T-08逐ID登记及T-12冻结门 |
| T-06 / AC-04 | 现有api_gen入口→共享四模块单源→HTTP服务/客户端生成切Web；Tauri/非HTTP留原owner薄装配 | 885约简及实际生成Rust编译/运行、非法兄弟/非HTTP消费；源/模板/锁与真实依赖相符，不手修生成产物；独立发行仍T-09 |

stage: work / scope: T-05,T-06 / outcome: executing / next: work / current_step: 4/12 / independent_review: pending T-11 / full_tm_gate: NOT_PASSED, final frozen union T-12。


### 2026-10-30 T-05～T-06 r6阶段收口

stage: work / plan_id: PLAN-890 / plan_revision: 6 / scope: T-05,T-06 / outcome: stage-owned-implementation-and-targeted-acceptance-complete / status: executing6/12 / worktree: 原五仓组树 / code_commits: language `1b177ebab1dab4203588abd77047026aaa366eb9`, Web `d8c889c786d83778f4b46efd7e73d6a8fb2935bf` / evidence: [r6交接及命令/数字/本地SHA256](#e890-ca5b663d4f4f80ee) / phase_audit: 实施上下文重建源码/图/原断言与控制，不冒独立终审 / full_product_check: FAILED（UI back-proxy旧Core引用，T-07既定消费迁移；不可冒baseline红） / full_tm_gate: NOT_PASSED（旧750并集的3红/中断与未执行原样保留，新身份最终门未跑） / remaining: T-07真实Man/UI装配及消费、T-08逐ID/四仓模块来源与执行账、T-09发行恢复、T-10应用/full自检、T-11独立审查、T-12冻结门/合入/规范/归档/guard清理 / next: user启动r4单元3/T-07。本单元完成后暂停；无远端/合入/归档/清理，无共享runner/注册/888脚本改写。

本轮发现并在原合同内关闭上传端口的环境重读漂移：添加中立executor的默认phase入口，Web转原captured-limits单源，ordinary facade与父版逐字相同。正确wire上限变异拒绝、还原后scoped413/ordinary接受并清理；初版raw max-file错误oracle、时间戳还原复用旧变异binary、libtest错误隔离与中断、stale probe lock初拒绝、首次编译/路径/命令失败均在交接具名保留，未覆盖为绿。最终nextest隔离109/109，源/许可/断言/条件未削减。metadata Web482/Core475/contract16闭包及禁止边负向检查通过；四整文件同formatter机械相等，Core无run_existing残余。规范仅提议实际owner/API/上传phase端口；canonical/账本留T-12。既存Core128/Web13/facade1警告明示，不宣称全库warning clean。


### 2026-10-31 T-10 bounded consumer repair (r6; execution continues)

固定Blog `b42a660` 的实际Vue输入得到保存PUT 422；独立API13项通过，真实请求缺body/expected_rev。已定位既有Vue store发射器将同名局部快照误当state ref，错误发射`snap_body.value`/`snap_rev.value`。T-10在原UI owner补有界正确性修复：按块追踪局部绑定；局部保持原名，冲突状态仅在实际使用时发射内部别名，保留委托函数/lambda捕获及旧公开getter。无新AST/运行时协议、产品目标、AC、仓集合或预算变更；SD-09增补现行`auto-ui-compiler/project.md`触面及plans索引。原ui_gen整模块866通过/2原ignore，旧断言未改；新增4反例覆盖快照字段、块/循环/闭包、同名初始化与委托捕获。首次0选、夹具解析红、初版源码形态控制红与缺失SQLite生成缓存输出均保留；不得视为完整应用绿。初版工具真实UI7项任务通过但重启有Windows esbuild目录访问失败，整profile13/15，保留失败；最终修订工具仍须重建并复跑Vue/原生完整profile。完整四仓门、独立终审、合入/归档/guard均pending。


### 归档后实际清理收据（2026-10-31）

- archived: `9207c0b49` 已提交实际归档文件；旧活跃路径不存在，68份MD已收拢，原59份受跟踪附件均已从Git删除。
- 五个权威工作树（Lang/Web/UI/Man/Down）逐仓确认clean、HEAD已入各master，再即时执行wt-guard全部clean，随后git worktree remove成功；四条开发分支均以branch -d成功删除。保留验证tag与历史恢复refs，不冒称它们是未清工作树。
- 局部清理阻塞：`D:/autostack/auto-lang/docs/plans/attachments/890`仍有10569文件/546目录/515080414bytes，无reparse point。自动批准审查拒绝整目录递归删除、先核查后逐文件删除和移除空组目录，均返回“blocked by policy”，未给更细原因。未继续绕过；临时hold owner为本PLAN-890收尾，原因仅为自动策略拒绝，不代表产品未解决缺陷。T-12整体清理仍pending，不能宣称merge全链pass或goal complete。空组`D:/autostack/.wt/lang-890`亦暂留。
- 用户明确不再测试，故没有落地主干后测试/构建或到期周期批次；原冻结通过证据的来源身份保持。其他会话的主检出文档WIP未混入提交。
- stage: goal | plan_id: PLAN-890 | plan_revision: r7 | from: merge | to: merge | status: archived | outcome: partial_delivery_cleanup_pending | next: delete_classified_local_scratch_when_policy_allows

- 证据保留维护已先preview再apply，退出0，最新五项为890/896/Blog003/UI013/888；删除旧Musk106证据等，实际释放1104410978bytes，无错误。剩余唯一文件数据38378036695bytes，仍超过10GiB监控预算，未冒称达标，受保护材料不为预算删减。收据：D:/autostack/plan-artifacts/.retention/receipts/last-five-20261031T072931628989Z.json。该维护没有删除上文被策略拒绝的本地附件或空组目录。
