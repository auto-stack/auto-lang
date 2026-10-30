---
plan_id: PLAN-890
status: executing
feature_name: AutoWeb独立仓完整搬迁与四仓交付
author: [Codex]
created_at: 2026-10-29
updated_at: 2026-10-31

plan_revision: 6
supersedes_spec_components: [docs/specs/auto-media-core/project.md, docs/specs/auto-api-gen/project.md, docs/specs/language-sdk/project.md, docs/specs/language-sdk/architecture.md, docs/specs/stdlib/design/http-server.md, docs/specs/stdlib/design/http-service-deployment.md, docs/specs/stdlib/design/backend-assembly.md, docs/specs/repository-boundaries/project.md, docs/specs/auto-man/project.md, docs/specs/auto-ui-tools/project.md, docs/specs/auto-ui-compiler/project.md, docs/specs/auto-test-case/governance-and-inventory.md]
new_spec_components: [docs/specs/auto-web/project.md, docs/specs/repository-boundaries/web-four-repository-delivery.md, docs/specs/language-sdk/http-host-port.md]
touched_goals: []

affects: [repository-boundaries, language-sdk, auto-api-gen, auto-web, auto-man, auto-ui-tools, auto-ui-compiler, auto-test-case]
depends_on: [PLAN-884, PLAN-885, PLAN-887]
current_step: 8
total_steps: 12
---

# [PLAN-890] AutoWeb独立仓完整搬迁与四仓交付

> **当前授权（2026-10-30，最新 `/goal 完成计划890`）**：恢复全链交付，从原五仓同树 T-07 继续到独立终审、冻结门、各仓合入与安全收尾；覆盖下文历史“逐单元另行启动/末尾暂停”安排。目标、AC、仓集合及禁止远端发布保持。实施不创建代理；T-11 独立上下文复审按原合同执行。第2次独立终审 needs_fix，executing8/12；仅重开T-08及依赖T-10，R2已闭。完整tm门 NOT_PASSED，不提前认证后续任务。
>
> **当前进度（独立终审第2次）**：executing8/12。R2配置传递及R1原A/B/C反例已闭；R1-D仍阻断：公开验收端接受一致删去必需UI用例或低于Man原floor的凭据。重开T-08及依赖T-10，保留其他已完成任务和有界证据。下一轮Work须先核验连续两次失败的原因诊断与oracle调整，再实施第2修复周期；同树有界修复后受影响复审。[独立复审](attachments/890/review/t12-affected-review-r6.md)。最终tm/四仓/当前组合发行、合入/规范沉淀/归档/guard清理均待执行。以下旧停止点保留为历史，以本条及§9最新收据为准。

> r6 / executing / 6/12；2026-10-30。用户已授权890完整交付，并明确选择“显式安装 Web adapter”。885收尾已由用户确认；当前按下述显式SDK装配迁移合同执行。r1的仅规划限制与等待记录为历史，不覆盖当前授权。
> **当前停止点（2026-10-30）**：用户“继续后续阶段”启动r4单元2；T-05～T-06运行时/生成器迁移及定向验收已完成并提交，暂停在该单元末。语言`1b177ebab`，Web`d8c889c78`；见[本阶段交接](attachments/890/work/t05-t06-handoff-r6.md)。**组合产品尚未构建通过**：根check暴露UI back-proxy三项旧Core引用错误，Vue另有旧HTTP生成调用，均由下一单元T-07消费迁移关闭；不冒baseline红或整包可用。**完整tm闭包仍NOT_PASSED**，保留旧3个AAVM红及未执行项。整体executing6/12，T-07待启动，未合入/归档/清理。
>
> **上一阶段停止点（2026-10-30；本轮已恢复）**：T-03～T-04基础实现与定向验收完成，阶段提交后暂停；不进入T-05。语言代码冻结`1a253950810de67f16f487c7eb7a2745f1061bb0`、交接`e45399a633a1dcf74f70943cf2d2c663791589a6`，Web冻结`26259642f56656547db7bc07a68c35a2bd225edd`。**完整tm消费者闭包未通过**：750模块选择的执行中55通过、3个既存红、23原条件不适用，旧HTTP测试装配挂起后中止，剩余未执行；修复后直接23项和HTTP4项重放通过，不能替代新冻结组合的完整门。见[本阶段交接](attachments/890/work/t03-t04-handoff-r5.md)。整体仍executing；独立终审、最终门及完整AC均保留。
> 依据：[Design50](../design/50-autoweb-service-framework-and-repo-boundaries.md)、[当前拆分路线](reports/v06-extraction-execution-roadmap.md)。一个完整工作包，内部可行性探针、迁移和恢复不另取号；本Plan为跨仓唯一合同，未来Web仓留指针，不再重复分配迁移Plan。

> **2026-10-30 历史T-02停止点（r5已获本单元启动授权）**：T-02 r3 有界修订与修复已通过阶段验收，executing / 2 of 12；本次到此暂停。代码/契约冻结 `f56211c12`，阶段交接冻结 `4ec696a63`。保全部树/分支，不进入 T-03、不合入、不归档、不清理。阶段审查由实施会话重建证据，明确不冒独立终审；AC-13 与 merge-owned 门仍 pending。后续按 [r4串行及正式同步安排](attachments/890/work/stage-sequencing-r4.md)；每启动单元结束提交/更新计划/交接后暂停。现行 [T-02 r3 契约](attachments/890/work/t02-contract-r3.md) / [交接与证据](attachments/890/work/t02-handoff-r3.md)；r2 needs_replan 收据保留为历史。

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
| [884](archive/884-language-structure-closure-without-crate-split.md) | archived10/10；delivery `fa0282c85`，有guard清理收据；后继保单运行桥/注册/主管线职责，不重构旧引擎全体 |
| [885](archive/885-blog-generated-rust-backend-parity.md) | archived7/7；delivery `dacd00184`，Spec/ledger已落；历史尾部cleaned仍记待guard且原始日志未恢复。用户已确认885收尾，启动缺口由该确认关闭，不伪造日志或代改885收据 |
| [887](archive/887-test-governance-and-ace-first-corpus.md) | archived r2、10/10；delivery `dd916c184`，有清树收据。盘点/守恒/首批ACE消费已交；性能以用户批准机制级负结果关闭，未实现显著加速。B17–19语义、D生命周期、ui-tn手工/Windows等限制保留 |
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

### r5 当前授权与tm规则对齐（T-03～T-04）

用户已明确启动本单元：先核实888正式交付，合并master到原分支，定向复核后执行T-03～04；完成提交/更新计划/交接后暂停，不进入T-05。已核实888交付`5a6c23afa`、归档`107148fcd`及收尾`129ef6057`都为master祖先，归档状态10/10且最终门/guard收尾明确通过；未联系或操作888会话。同步前890=`ed44fd0bd`、master=`129ef605763e59ca13bf312fd4a391eded10c13a`，原树clean；已合并为`29ed9d7bde334ae14e21223c935a4f3974581f71`，无人工冲突。双方实现与测试义务保留。

本次合同有界修订原因：888正式规约已将旧裸t+触面档叠跑改为tm完整叶及消费者/集成闭包。r3中旧档位命令仅保留历史，不再作为当前验证处方；阶段执行按有意义反例、必要check及实际受影响完整tm模块；最终统一合入门仍归T-12，在冻结提交选择受影响完整模块去重并集，周期批次独立且不重复叠跑语言旧档。原ID/原子oracle/ignored/平台/慢模块条件不减，不因目录过期而重生floor接受漏项。T-08四仓治理须扩正式tm所用模块/来源/消费者/执行收据，保持r3 required owners/真实执行/失败拒绝义务；具体实现仍T-08，不提前改共享runner。

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

888由原会话优先独立复审、必要验证、合入和收尾，不等待890；本次不联系或操作该会话。890避让共享脚本/注册/锁文件/最终合入窗口，不使用888未交付WIP。每单元失败均保留未完成状态和具体关闭条件；实质设计/合同变动记录原因并先修订，不无限追加修补。阶段提交不代表整个计划完成。完整规则与当前冻结身份见 [r4阶段交接](attachments/890/work/stage-sequencing-r4.md)。

### T-02 r3 技术契约（r4保留；阶段安排以下文为准）

2026-10-30 用户交接恢复 T-02 修订与修复。仅继承原目标/AC/仓集合及 T-02 窗口；不进入 T-03、合入、归档或清树。基线语言 `76f2069d5`（代码 `e2ed6ba76`）、UI `7b323bc`、Man `e6cc06f`、Down `c3464d0`。语言树 clean，主检出产品路径 clean；主检出已有其他文档 WIP 保留。

| finding / 决策 | 不变量与实施入口（owner_stage=work） | 精确关闭条件 / 反例 |
|---|---|---|
| R1 / D-T02-01 | 中立 `HttpServiceLaunch` 在 invocation 执行前显式绑定，独立于编译 metadata；配置由领域 owner 以不可变 typed payload 传递，Core 不解析 Web 部署政策。sync SDK 桥跨线程携同一快照，async 调用逐 poll 恢复上下文，VM 构造后持自身快照；host 仍在激活时选择。旧 active-config 仅作现有入口兼容转换，不成为新 SDK 权威 | configured SDK 实际业务端口/config hash/port0 bound 正确；嵌套、交错调用与失败后新实例不串配置；普通程序无额外装配要求。旧 audit 的 env 端口精确业务反例必须转绿 |
| R2 / D-T02-02 | `serve_with`/`serve_async*` 返回真实 `AutoResult<()>`，bind/runtime/join 错误逐层传播；ready、身份和停止属于 `ServiceRuntime`，进程 fatal/bound 仅历史 facade，不能裁决新调用 | 占口 SDK 非零且不 ready；同进程 A 失败后 B 成功/独立关闭。保持原 Result trait；不由 stderr 字符串替代错误传播 |
| R3 | owner 成功交出普通 parked reply 后，由网络桥写状态/字节并终结；发送失败由 owner 取消，不抢先 complete；错误/超时/取消也恰一次回收 | audit 原 wire200/25 对应唯一 event200/2/completed；补同源发送失败、async 错误/取消与资源归零，不能只检查响应 |
| R4 / D-T02-02 | runtime session 持网络与日志线程句柄；停止发射→关 sender→排空→join；join 在阻塞执行器/回收线程，不能阻塞业务 reactor。启动失败与 future Drop 也必须触发取消/回收；VM/parked 清理留原线程，网络只持 owned Send 消息 | 正常/启动失败/取消都有真实线程终结证据；日志写/线程错误显式报告，队满仍丢弃计数且不阻塞；取消只触发清理，完成 receipt 必须等待实际 join，不能用 future Drop 本身冒清理成功 |
| R5/R6 / D-T02-03 | 关闭 R1..R4 后冻结候选契约的包/API/消费者映射及正式887兼容四仓字段/拒绝规则；实际建包、runner/发行仍各原任务负责 | 指明已执行与待实施命令、required owner/leg/ID/source/hash；不将 source probe 当完整迁仓；不修改888共享窗口 |

r2“最终 API 尚未冻结”及 needs_replan 收据保留为历史。r3 已关闭 R1..R4 并冻结 R5/R6 的执行目的地与 schema，现行证据见 [r3交接](attachments/890/work/t02-handoff-r3.md)。T-02 计2/12；所有实际迁仓/原ID运行/四仓发行、独立终审及 merge-owned 门仍 pending。

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

修复1已完成并冻结，工作T-01～T-10共10/12；R1/R2按实际反例与合法对照关闭实施缺口，待T-11受影响独立复审。未变代码/输入的原证据有界复用；当前新SDK/CLI已真实执行，不以旧发行收据冒新组合证书。T-12最终完整模块/消费者并集、四仓/当前组合发行门、到期批次、各仓合入、规范/账本/index、归档/guard仍pending，全链授权继续。

| task_id | owner_stage / 依赖 | 涉及路径、产物与关闭条件 | AC / 验证入口 |
|---|---|---|---|
| T-01 | work / G890 | [✅ T-01] 已完成入口清单：唯一语言树/只读依赖、用户确认U885及原始证据边界、Web本地身份、76源码文件/4120实际声明的唯一逻辑owner、6档24869历史行/4683原ID保留及17资源hash；未冒充运行分母或最终迁出结果。提交`b3c74b165`，摘要`attachments/890/work/intake-r2.md`；原许可/来源与888/889实际diff窗口见原入口摘要。T-02仍须冻结物理包图与全路径端口，T-08实际ID迁移/执行账不得以此盘点替代。 | AC-01入口部分；V-01，Git对象/语法树/源hash/原清单核对；parser及admission控制各1/1，原证据本地hash见摘要 |
| T-02 | work / T-01 | **[✅ T-02] r3有界修订/修复与阶段验收通过；到此暂停。** launch与编译声明分离、VM前绑定；真实Result、独立网络关闭、parked事件移交桥、日志/网络join与取消cleanup回执闭合。原audit3/3、session5/5、交换配置反控拒绝、file/SSE终态/资源、5个命名单测及check/fmt通过；闭包/共享生成未触面证据有界复用。现行 [r3契约](attachments/890/work/t02-contract-r3.md) / [冻结交接](attachments/890/work/t02-handoff-r3.md)；代码`f56211c12`、交接`4ec696a63`。R5/R6冻结的是后续包/API/schema执行合同，未冒实际建Web或四仓发行。阶段自查同会话限制明示，独立终审仍T-11 | AC-02/03/07仅T-02部分；V-02..04及V-05/07最小兄弟控制；完整迁仓/CLI/UI/发行仍各原任务，T-03需用户另行授权 |
| T-03 | work / T-02 | **[✅ T-03]** 用户已启动本单元；888正式交付/归档/收尾祖先已核，clean原分支已合入master为`29ed9d7bd`。本地Web普通仓`D:/autostack/auto-web` bootstrap `8ff4966`，唯一树`D:/autostack/.wt/lang-890/auto-web`、`codex/plan-890-dev`，基础包/锁/文档及CI接线契约骨架提交`2625964`。合同/框架基础已真实check及metadata闭包验证，来源/MIT/原Git锚保留，无远端/registry/CI部署。基础包不冒runtime交付；T-04定向验证及完整门限制见下一项 | AC-01/02的T-03部分；基础包check、482包normal/build/required-dev闭包无UI/Man/Tauri/混合facade，单Core/合同/共享载体来源，完整独立发行留T-09 |
| T-04 | work / T-03 | **[✅ T-04；基础实现/定向验收完成，完整tm门未通过]** 基础代码`84779b2b1`、测试装配修复`1a2539508`：纯合同实际迁Web、四共享模块迁auto-generation-core，旧路径薄转发；Core真实Cargo来源入SDK身份清单。实际类型/Arc消费、严格配置、原15单测、生成companion正反例及T-02 audit3/session5通过；tm4个直接叶扩大750模块/5483义务；执行55通过/3既存红/23原条件不适用后，旧HTTP测试装配挂起中止，剩余未执行。已显式修复旧SDK测试装配并完整重放直接23项及HTTP4项，均通过；不冒完整闭包通过，T-12在最终新源身份重选并执行，原ID/断言/条件保留。Core `lib/entry/run_bridge/host_capabilities/http_server*`按冻结端口接线，纯合同/语言值转换与最小VM桥、必要共享生成承载/SDK源码闭包；按r2完成SDK服务显式安装迁移、普通语言兼容和明确缺席诊断 | AC-02/03；V-02..04，实际包check/命名单测及外部SDK消费 |
| T-05 | work / T-04 | **[✅ T-05；R2文件入口配置修复，原transport/资源证据保留]** 本轮修复与当前输入见[修复1交接](attachments/890/work/repair-1-handoff.md)；下列原阶段数据按原输入保留。[r6交接](attachments/890/work/t05-t06-handoff-r6.md)。Web真实host/transport/scopes/file/upload/policy/session归新owner；Core临时桥退出；runtime隔离nextest109/109、实际包TCP/关闭和上传快照正反控2/2、audit3/session5及文件/SSE/12MiB/RST/双实例/legacy通过。根组合check三项UI旧引用失败保留，非全AC通过。Web `src/`接管HTTP领域runtime/配置/观测/路由与两轨装配；Core旧领域逻辑逐消费者迁出，handler/scope/file/upload/许可完整终态保持 | AC-03/06/07；V-04/08..10，真实TCP正反/资源控制 |
| T-06 | work / T-04 | **[✅ T-06；迁移/定向验收，CLI/UI消费待T-07]** 代码语言`1b177ebab`、Web`d8c889c78`；HTTP生成/targets/模板归Web，IPC留原owner/shared四模块单源；生成48通过/1原ignore、targets18通过、实际cold生成/编译/TCP/恢复1通过、原file/upload/JSON-SSE wire3通过，Tauri具名2通过。原按需ignore和完整ID/门仍pending；见[r6交接](attachments/890/work/t05-t06-handoff-r6.md)。拆`auto-api-gen/src/{api_gen,rust_workspace,sidecar,source_owner}.rs`真实归属，HTTP生成/客户端/模板切Web，885控制与非HTTP/共享规则不复制，必要兼容门面无环 | AC-04；V-05/07，约简与实际生成Rust编译/执行 |
| T-07 | work / T-05/06 | **[✅ T-07；消费迁移/定向验收，整应用T-10/最终门T-12保留]** Man/UI正式依赖与直接HTTP调用切Web，显式安装provider并跨线程保存typed launch；Core形态闭包534包、禁止UI/大门面/Tauri边0。真实CLI VM和Rust两轨精确响应/port0配置哈希/关闭退出/端口释放，UI真实VM worker1、proxy4、Vue流生成1及Man命名5均通过；根/UI/Man check通过。原daemon命令、非HTTP/共享单源保留。语言`163d342a6`、UI`672955d`、Man`9f8ffc2`，[收据/限制](attachments/890/work/t07-handoff.md)；冻结完整组合及Blog双UI不冒提前认证 | AC-05的受影响消费接线；V-06/07定向实证；整应用和完整闭包仍T-10/12 |
| T-08 | work / T-07 | **[待修 T-08；R1-D必需UI原ID/Man floor在验收端未守恒；原A/B/C反例已闭]** 本轮修复与当前输入见[修复1交接](attachments/890/work/repair-1-handoff.md)；下列原阶段数据按原输入保留。117迁移ID及696历史行/116唯一ID、17资源核；10原UI照片测试迁至media且原测试子树hash相同。46 recipes/7320义务/1211叶已真实发现；Web217过+2原ignore，保留语言28过、媒体54过、当前治理反例9过。真实四仓清单全部ID/floor核通过，partial/list-only不认证交付的实际负控exit2；不降floor、不退役原义务。语言`38f68d8e9`、Web`f4cc93c`，[收据/原红/限制](attachments/890/work/t08-handoff.md) | AC-08/11的T-08部分；V-11/16；最终冻结实际批次T-12仍NOT_PASSED |
| T-09 | work / T-08 | **[✅ T-09；R1C/R2有界修复与新SDK/CLI验证完成，最终组合收据T-12保留]** 本轮修复与当前输入见[修复1交接](attachments/890/work/repair-1-handoff.md)；下列原阶段数据按原输入保留。19包纯SDK自包含源/Windows包已真实构建、测试、fresh/locked/A→B及故障恢复；冻结发行45/45、历史Core恢复14/14，SDK217过+2原ignore。Man事务唯一源/不可编辑工具快照、完整stdlib/版本/来源/哈希与严格Core/Full边界；公开反例4拒绝且旧workspace不改。真实Git独立检出构建/库测试及Windows目录限制/LEAK分列保留；治理10与用途19反例通过，未降旧floor/原条件。[包/哈希/范围](attachments/890/work/t09-handoff.md)；语言`f739c3751`、Web`9ddb7fe`、Man`cb4ab3d` | AC-09/11；V-12/13；旧架构仅Core合法形态，未冒旧UI/Full运行认证；T-10/11/12继续 |
| T-10 | work / T-09 | **[待修 T-10；依赖T-08重开，须完成第2次失败诊断、oracle调整与新冻结交接]** 本轮修复与当前输入见[修复1交接](attachments/890/work/repair-1-handoff.md)；下列原阶段数据按原输入保留。固定Blog原314文件未改，真实冷Rust构建/API13及完整Vue15/15、原生22/22；新增UI局部快照/同名状态修复`9aad12c`，原ui_gen模块866过+2原ignore、4反例。语言`ede598998`刷新46 recipes/7329义务/1213叶，75职责组、19守恒用途控制；新增warning差0、旧debug/TODO来源核与SD01..09回交完成。原红/路径/缓存/控制器清理限制具名保留，[冻结交接](attachments/890/work/t10-handoff.md) | AC-10..12；V-14..17，工作AC自查不冒独立终审/整档门 |
| T-11 | review / T-10 | 同权威组树独立final review AC-01..12、V/SD及新反例，绑定各HEAD/依赖/资源；有finding回work同树修复并重审；pass后reviewed11/12且最终门pending | AC-13；V-18，auto-plan-review流程，独立证据重建 |
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

五模式真实 probe 均 exit0：注解 handler 精确随机 JSON/缺参400/非法整数400/404；直接原语安装后404/受控停止/同线程/handler task无增量；直接原语无安装、仅异步 provider、注解无安装三反控 child1/明确诊断/端口仍可绑定。`cargo check -p auto-lang -j4` exit0；原3项定向测试各1passed/0ignored（host选择、metadata Pending/nested/unwind、媒体wire断言）；locked外部fixture构建exit0，Core141/root3/external17等存量warnings不冒最终健康通过。未跑整档或GUI。命令、原始log SHA256与边界见 [blocking摘要](attachments/890/work/blocking-listener-port-r2.md)。

初始语言源码 `use auto.http; server.listen(...)` 在到达native前发生既有 Server.listen 名称解析错误，未记通过；直接原语反控使用既有公开 SDK alias，不以其替代最终语言消费验收。语言级直接调用、direct返回后的post-main协调、不同语义legacy stdnet入口、实际back-proxy消费、剩余实例状态/producer/deadline、完整共享生成与physical包/API/SD/四仓schema冻结仍T-02，未经批准未延后。T-01历史原子账需T-08按最终owner逐项对账；未创建Web仓，未动888共享注册/runner/锁/最终合入窗口，UI/Man仍等完整U890。独立final review/merge-owned门与交付收尾未执行。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 16 | from: work | to: work | status: executing | outcome: pass（有界实现与真实正反证据，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: d861642a5 | next: work（goal active；仍1/12；继续T-02）

### Goal round 17 — T-02 shared generation mechanism cut（2026-10-30）

代码提交 `7c5a98804`：混合 api_gen 的25个共享函数收敛到实际生产 `shared_generation.rs` 单源模块，旧公开路径re-export、私有调用保私有。范围包括锁/装配identity纯规则、full/lenient抽取、带source_dir/proofs转译、runtime源码限定、companion/db后处理和公共类型提升；HTTP模板/服务装配与Tauri IPC装配不混入。按父Git对象精确切片重建、只施加具名可见性及rustfmt后全文byte比较通过，函数体零变化。选定语言侧具名受限载体proposal `auto-generation-core`0.1.0（四模块：shared_generation/rust_workspace/sidecar/source_owner），完整T-02冻结通过后才T-04物理迁包，不能据此提前建Web。

独立source probe的17项原单元通过；新增真实外部Cargo消费1/1：实际转译导入的两模块、精确环境值/leaf运行、去掉type promotion产生E0308 types::Work/works::Work、去掉source_dir产生repository.rs &str/String E0308、恢复原生成物后再次实际运行。完整metadata480 packages无auto-api-gen/UI/Man/Tauri，含normal/build/dev；只属本地闭包证据，非fresh发行或跨OS执行。原APIgen定向ID12passed/1父版本复现失败/0ignored，包含字符串/借用编译oracle及Tauri mock dispatcher；root/APIgen cargo check分别exit0，存量warnings保留。初始fixture对Env lowering必产非空proof的错误假设已移除，不以空proof冒引用闭包认证。

为读取原Tauri fixture建立Man唯一同寿命只读detached组树 `D:/autostack/.wt/lang-890/auto-man`，准确pin `e6cc06fbc496f366edde7929abfd49254ec13647`；main/组树clean，无AGENTS/链接/产品改动/分支，后续如接线则在同树转Plan分支，最终需guard清理。不是UI/Man结构后继启动。原fixture比较先因缺组树失败，补齐后发现generate_handler宏纯换行byte差异。临时恢复api_gen/lib为父提交d861642a5的原始代码运行同原测试，得到相同fixture drift失败，并finally逐byte恢复候选。未改fixture/golden/断言/ID/skip；此原ID按明确基线失败继续入T-08/final accounting，不冒全绿。证据及hash见 [共享生成摘要](attachments/890/work/shared-language-generation-r2.md)。

888权威树读回eb80a4b89，仅有其诊断md未跟踪。未动共享脚本/注册/锁/合入窗，无agent/消息、整档、GUI、push。T-02剩余监听/legacy/producer/deadline与完整消费者/HTTP生成闭包/physical包API命令SD四仓schema仍待闭；Web未建，独立终审、冻结门、合入沉淀与清理均pending。UI/Man结构包继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 17 | from: work | to: work | status: executing | outcome: pass（共享生成实际单源改动与强正反证据，含具名父版本红；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 7c5a98804 | next: work（goal active；仍1/12，继续T-02）

### Goal round 18 — T-02 actual generated-service neutral closure（2026-10-30）

提交组合：lang `d31081334ddfce369a494849bccb3c324449c715` + UI `7b323bc02e4a8153a086ca3f88c5f1a95b47c446`。为关闭真实服务对UI的依赖，把已验证renderer-neutral照片实现及十个原测试整文件LF逐byte迁入既有auto-media-core/image-codec（源UI d53f946、blob ebeadbb360dee925c6fd137fc0a6c21fca0c707c，SHA256 738751c6bfa88c9ea5c313d620d008528f7e64a9e4ab28fa78ed0bdcef12558f）；UI唯一同寿命树转codex/plan-890-dev，旧feature/public路径只re-export，未开展UI结构后继。新增source/test补充映射保十原ID/条件/新owner，T-08仍需入完整执行账，不替换更小floor。SD-04补既有media Spec及模块index影响，新增component仅属proposal，不提前改canonical。

实际生产生成器photo模板改用media公开源，生成Cargo不再选择UI runtime；四处生成/成员/lock/新鲜度装配统一Service workspace。开发态只验证实际Core/a2r/media依赖，未选择的facade可有缺席可选UI路径；选中Core的坏可选路径仍绑定前拒绝，旧manifest保持。managed来源receipt校验与UI调用全闭包保持。原canonical-owner/media-route测试只更新真实owner期望并新增禁止UI断言，路由/GET HEAD/两状态义务未删。

已验：media十原例10pass/0ignored（private TEMP/TMP避免借共享缩略图cache）；codec-off check0；UI默认runtime locked check0；root/APIgen check0；原workspace4+owner1+stateful/full-cover1+image1定向全过；增强service来源/已选Core拒绝probe1/1。存量Core141/root3/APIgen3/UI215及最小生成物4warnings不冒全局清零。真实external production generator新项目→未手修Rust build+locked→metadata483packages，所选闭包0 UI/Man/Tauri/旧语言facade；精确generation ready fingerprint/随机业务值、3x2 PNG scan/full原字节/JPEG thumbnail真实尺寸、越界400/未知token404、自身shutdown202/draining true/child0/真实stdout STOPPED、源码变更stale1/恢复fresh0全过。源码/生成/构建工作目录均在890树，复用显式target；完整fresh SDK/分发/四仓档仍T-09后续。证据：[中立生成摘要](attachments/890/work/neutral-generated-service-r2.md)。

验证流程修正：早期fixture错读stderr的STOPPED流；另一次把临时生成工程放主检出attachments违反worktree纪律，均未计入认可证据。最终在本树target/runs新工程完整重放通过，main仅保随后复制raw日志。此后补preflight同时拒绝树外source/build/evidence落点；树外target负控exit2、未启动生成，log SHA256 ef927a8989ef4cd29ec8cc44c48eab700801c94354a9981559f10e5afbaf08c8，候选合法路径未变，复用已过完整重放。脚本/fixture不注册全局档或锁。

所有本轮子进程均已结束，lang/UI树clean；Man只读树仍e6cc06f。共享治理脚本/测试注册/锁/888 helper/最终合入窗未写，未跑整档/docs_gen/GUI或push。T-02剩余直接语言入口与返回协调、legacy/state/producer/rate/deadline终态、完整VM及实际消费者/physical包API命令SD四仓schema冻结仍开放；Web未建，不以本次单源接缝/实际生成成功冒完整技术门。原Tauri byte fixture父版本红继续具名保留。UI/Man结构包继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 18 | from: work | to: work | status: executing | outcome: pass（实际生成无UI闭包及中立单源，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: d31081334 + UI 7b323bc | next: work（goal active；仍1/12，继续T-02）

### Goal round 19 — T-02 real language direct activation（2026-10-30）

提交 `4a266bb30b16e19508559982113d15916ce328ae`。实际语言入口已核实为 `use auto.http` + `http.server_listen(http.server(), address)`（wildcard自由函数同样可达真实native），不是只能手工SDK栈调用。方法式server.listen仍在现基面发生link未定义；type-qualified探测exit0但未到检查过的adapter不计服务正例；未为迁移改编译器/loader/注册或新增语言特性。此前“直接语言入口未核实”在此自由函数支持级关闭，不冒全部拼写/轨道可用。

修复直接监听完成后post-main注解再次启动：AutoVM持私有per-owner Cell，只有blocking host成功返回才置位；失败不置位、其他新VM不受影响，自动owned handoff对已完成owner直接返回，无需再次选host或绑定监听。仅新增2个crate-private访问器，不扩大公开VM/私有模块面、不复制VM或跨线程发送，lib自动入口和888共享面未改。fixture自己的stdin关闭controller现在保存并实际join；生产logger/deadline等完整线程终态仍未冒关闭。

真实语言程序language-installed：精确随机JSON/缺参400/非法整数400/未知404/stop正常返回、handler task无增量、原owner thread、controller join，1次真实external blocking activation且0次post-main async activation，child0。language-missing与language-unsupported均wrapper0/child1，真实native RuntimeError分别明确缺安装/不支持blocking且端口仍可绑定；不是跑到post-main才假装缺host。普通installed自动入口重放也全过，fresh自动消费者保持。原host选择单测1/1扩per-VM正反：完成owner后清默认仍可返回；fresh缺host拒绝；host拒绝不claim；fresh安装后仍触发auto，记录准确owner ID。locked外部构建与root check0；Core141/root3/external17存量warnings保留。未跑整档/docs_gen/GUI。详见 [direct语言摘要](attachments/890/work/direct-language-activation-r2.md)。

888读回eb80a4b89，其相对基面对engine.rs/Core lib无改动、仅自身诊断md未跟踪；本轮无共享注册/runner/锁/最终合入窗、无agent/消息。所有执行输入/构建在本树，raw日志仅本地；lang/UI仍同寿命树，UI7b323bc与Man e6cc06f未改。T-02仍需不同语义legacy stdnet、configured direct选项/active cfg、控制/producer/rate/deadline及完整owned terminal/consumer、physical包API/commands/SD/四仓schema冻结；Web未建，独立终审和merge-owned最终门/合入沉淀/清理均pending。UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 19 | from: work | to: work | status: executing | outcome: pass（真实语言直接入口与owner恰一次装配，非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 4a266bb30 | next: work（goal active；仍1/12，继续T-02）

### Goal round 20 — T-02 explicit legacy stdnet / identity / terminal（2026-10-30）

提交 `79a6a411a211fb905a30bf58867ad1793cd8c3d6`。阻塞宿主端口显式区分ServerHandle/LegacyStdnet；旧公开serve_blocking_stdnet成为按kind选host的薄桥，保unit-return+指名诊断，缺安装/仅async provider均绑定前拒绝不回退。旧loop单源移到server_legacy，由真实外部source crate执行；仍未宣称Core实现退出，T-05必须移走。修掉SSE/file早退漏task、持DashMap guard移除/异步owner blocking_lock风险；request/iterator/next-task guards在错误/断连完成，资源通过请求local Send context登记/终态与late注册同锁；新具名primitive cleanup dispatcher与既有RequestScope late分支共享，不扩大VM/private模块pub面。

按现行http-return-resources §2修旧legacy裸int查表误判：只由真实Sse凭据赋予流身份。新probe经公开VM SDK创建真实lazy generator，实际key0保留为caller-owned；HTTP普通int0必须是精确JSON字节0且不得拉取/回收该generator。两个真正SSE形式（原~Iter声明、int声明但返回真实generator）各精确1/2/3帧保持；handler/request返回值的实际rc_count/凭据类型检查在VM drop前证明只有caller seed仍持有，seed显式释放后持有0，避免仅看进程退出掩泄漏。

真实SDK wrapper→显式external LegacyStdnet：随机精确JSON、缺参/非法参数400、404、CORS*、只发头承诺1MB body的OPTIONS204、两种SSE、连续3次file显式500、观察实际无限generator帧后RST并恢复精确业务响应、stop/controller join全过。VM drop前handler task delta0/service iterator delta0/descriptors0/request credential owners0；owner返回后actual credentials/uploads0，caller iterator保未启动且独立释放。missing/unsupported反控保旧child unit-return0但必须明确诊断且无监听，非服务健康绿。fixture grade-swap替换错误runtime即使JSON精确也被原CORS oracle拒绝（期望exit1），不改生产逻辑；server-handle语言重放仍过。

本轮主动命中并修复自己Round16引入的源布局回归：原Plan705默认调用图围栏红（非默认server-handle loop误放进默认HTTP扫描目录）。不改测试/断言/required count，把该loop逐byte移回非默认职责文件ffi/http_server_handle，APIalias不变，SHA256 3184a2af5cccc2ef4ffbd359dd7adfd7d1dce4f5189ca9a084e2008081f472c8；此前原本在stdlib/http_client，同样不属默认图。Core只有定义、唯一调用在explicit外部probe；默认async装配未转调用该loop。相同原gate在最终source上1pass：默认未标记busy0、原marked legacy1。这是本Plan修复，未转历史债或缩floor，T-08仍应完整迁原义务。

Root check0，lockedSDK build0；原Core late upload context/host selection/media wire三项各1pass/0ignored；root原callgraph门1pass，fmt/diff/Python编译过。现存Core141/root3/external17/root lib-test42 warnings不冒全局健康清零。无整档/docs_gen/GUI/push，未动共享治理script/测试注册/锁/888 helper/最终合入窗；源码与构建均在本树，只md摘要入库。证据：[legacy端口摘要](attachments/890/work/legacy-host-port-r2.md)。所有进程结束，lang树clean，UI7b323bc/Man e6cc06f不变。

T-02仍开放configured launch/程序级legacy policy与producer所属、default runtime完整deadline/control终态、实际consumer及physical包/API/commands/SD/四仓schema最终冻结；Web未建，独立终审/最终门/合入沉淀/清理pending。不得因已打通legacy和生成闭包而跳完整门；UI/Man结构后继继续等U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 20 | from: work | to: work | status: executing | outcome: pass（legacy真实端口、来源身份与终态/修复本Plan围栏回归；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 79a6a411a | next: work（goal active；仍1/12，继续T-02）

### Goal round 21 — T-02 native policy / owner rate windows（2026-10-30）

提交 `e32f4d678`。真实反例确认语言http.rate_limit此前写Core framework全局，而external source runtime读取另一份：四个精确nonce成功后第五次仍200、未按要求429（exit1）。先修fixture header大小写读取错误，再取得此认可red；未以错误oracle充反例。native改存VM私有纯设置，未设置/显式关闭/正配额区分；具名VmHttpBridge只在原VM线程读取，未公开私有字段或跨线程传VM。旧SDK set/clear公开面保一个中立process default数据源，语言显式设置包括0只覆盖自身VM；固定窗口算法和每个serving lifetime bucket仍属framework，每poll选择/恢复实例状态，standalone dispatch旧共享bucket不被实际owned服务消费，后续T-05仍须完整单源迁出。

最终真实TCP：Core旧SDK default1、A native4、B native0后handler动态2；A耗尽后B仍可多次精确响应，B开启后仅两次200再429，A仍429。校验429 JSON/retry_after_ms/Retry-After/request-id、旧环境CORS精确值、两owner drain/registry0；config-only普通程序无adapter仍返回42。SDK default/clear及VM显式disable/window最小值/fresh VM单测1pass/0ignored；locked外部双bin build0、root check0；受poll wrapper影响的configured service state原探针重放0，identity/counters/CORS/shutdown隔离保持。Core normal warnings140（原141的unused native vm现已使用）、root3/external17、Core lib-test160，未冒全局清零。最终唯一native-call折行为格式调整，未改变已编译行为；helper fmt/Python编译/diff过。详见 [native policy摘要](attachments/890/work/native-policy-owner-r2.md)，六份raw日志hash已记。

888读回70a80fead：本轮相关重叠仅其原spawn_outcome_reporter cfg变动，不改该helper/共享注册/runner/锁/最终合入窗口，不发消息或启实施agent。主检出只有既有docs簿记/他方设计WIP，未基于它做实现；lang/UI/Man复用同寿命树，UI7b323bc/Man e6cc06f保持clean未改。所有子进程已结束，源码/生成/构建在890树，raw本地保留，只有md摘要入库。无整档/docs_gen/GUI/push；原HTTP e2e ID未冒执行，T-08完整迁移floor保持。

T-02仍开放producer编译/activation前元数据归属、configured launch、完整deadline/control/线程终态、actual consumer与physical包/API/commands/SD/四仓schema冻结；Web未建，独立终审/最终门/合入沉淀/guard pending。UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 21 | from: work | to: work | status: executing | outcome: pass（真实native限流外部传递/实例隔离与SDK兼容；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: e32f4d678 | next: work（goal active；仍1/12，继续T-02）

### Goal round 22 — T-02 network thread join / drain terminal（2026-10-30）

提交 `9fc909552`。上一轮为实际实现进展。本轮owner保留并异步join自己的真实网络线程，失败启动分支同样回收；只有Send线程句柄进入blocking executor，VM仍原线程。接收队列先drop，网络线程完成后才报Stopped。owner drain deadline只设置一次并在继续parked工作前检查，不随请求/完成事件刷新。两侧等待真实watch true（含激活前已true），忽略false更新，sender关闭仍退出。连接清理改为spawn前捕获RAII，未poll或被强制取消也abort自己的reset-shadow、取消自己scope并归还该实例active计数。

两个来源约束反例：只恢复父e32f4d678的owner源码，实际完整transport返回后同一真实网络线程fixture尾部延迟800ms，旧owner提前返回，terminal实测network_tails1、probe exit1；只恢复父transport而保新join，partial-head idle跨drain后实际connections_active仍1，adapter断言触发child panic/外部probe exit1，后续connection-refused是次生输出。两个临时恢复均在同树finally逐byte还原候选，随后最终build与正例；没有替代网络实现或生产延迟旋钮。early system-temp源码运行未计认可证据，已把两个受影响probe源/资源scratch固定本树target，并在正确位置重放两red和四positive。

最终locked external双bin build0、root check0；watch初始true/false更新/sender close单测1pass/0ignored；tail installed精确随机业务+实际线程完成先于owner终态、pre-stopped正常返回+端口重绑、configured双owner带partial-head连接跨drain强制清理、16MiB精确file跨关闭完整送达+ready非200+B独立+两端口重绑四probe全exit0。每实例return后actual active计数0，全owner完成后registry0。deadline固定由代码实证，不冒称跨全部CPU/native负载有限关闭。Core140/external17/root3现有warnings保留。helper fmt/Python编译/diff过。详见 [network终态摘要](attachments/890/work/network-terminal-r2.md)，九份raw日志hash、父源码还原hash及边界已记。

新file日志显示完整wire送达后request事件bytes_sent仍0，此项列为T-02待查response terminal observation，未冒字节观测通过或擅自延后。其余producer activation前归属/并发编译、configured launch/实例failure传播/logger退出、实际consumer/physical包API命令SD四仓schema冻结仍开放。Web未建，仍executing1/12；独立终审/最终门/合入沉淀/guard pending，UI/Man结构后继继续等完整U890。

888读回643f2b375，transport/dispatch相对共用基面无diff；未改helper/共享注册/runner/锁/合入窗、不发消息或启实施agent。主检出既有docs WIP保留，lang/UI/Man唯一同寿命树，UI7b323bc/Man e6cc06f仍clean未改。本轮进程全部终结，只有md摘要入库；无整档/docs_gen/GUI/push。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 22 | from: work | to: work | status: executing | outcome: pass（真实网络join/连接取消终态正反证据；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 9fc909552 | next: work（goal active；仍1/12，继续T-02）

### Goal round 23 — T-02 actual file/SSE terminal observation（2026-10-30）

提交 `2dbde2ab5`。上一轮为实际进展。本轮关闭已观察的file bytes_sent0问题：原producer finish hook在HTTP缓冲体未读时就complete scope，file/SSE未按payload poll累计字节，实际file206/304/416/404亦未替换占位200。真实7项wire/event multiset反例exit1；fixture初期误以为自动ETag，改为明确descriptor ETag后才取得认可red。新framework body guard只计交给HTTP的payload字节（不冒TCP ACK），保原scope单点恰一次事件，成功/早Drop/错误分开，支持精确长度最后帧后直接Drop而不额外poll EOF。producer结束只记结果，不提前归还未读request；实际file状态/inline错误长度在终态前写回。未扩大VM/private端口字段或复制语言对象。

另捕获并修复两个实际问题：configured file在net线程泄漏旧默认CORS，反控exit1后用每poll服务policy上下文覆盖net request/file组装/拒绝路径；中间修复造成still-connected stalled file在watchdog后不终结，此为本轮未提交候选的回归，不甩父基面。具名per-connection watch只中止该失败/超时active file连接，让真实body Drop完成同一scope清理，准备失败仍走HTTP inline错误，stream激活竞态有双侧检查；未建全局abort map/生产延迟旋钮。

最终真实external probe exit0：完整binary GET、range206、HEAD200、conditional304、range416、missing404、finite SSE七项method/route/status/bytes/outcome精确一一匹配；file/SSE RST各一次canceled且正字节、未读file不得计满长，源文件实际截断一次canceled；200ms file idle budget下客户端保持连接不再读，3s内实际body终结事件已到且仅一次，再主动close；B counters不随A流量变化；owners/registry0。real file pump单测1pass证明producer完成而body未读时request permit仍0可用，最后帧后不poll EOF Drop一次归还、exact bytes/completed1/canceled0；原full_queue503断言不改、内部新参数None，1pass。locked external build/root check0；受影响16MiB across-drain+配置隔离/两端口重绑与legacy native/CORS/ordinary-no-host两probe重放均0。Core140/external17/root3 baseline warnings保留，fmt/Python编译/diff过，最终仅注释更正不改已验行为。详见 [响应终态摘要](attachments/890/work/response-terminal-observation-r2.md)，十份raw日志hash及边界已记。

888仍643f2b375，transport相对共用基面无diff；无共享注册/runner/锁/合入窗/agent/消息，主检出既有docs WIP保留；UI7b323bc/Man e6cc06f clean未改。全部进程终结，源码/资产/构建在890树，仅md摘要入库；无整档/docs_gen/GUI/push。

T-02继续核compiler producer activation前归属/并发编译/async-return元数据、configured launch交接/实例failure/logger退出、ordinary parked/async实际观测、完整consumer/physical包API命令SD四仓schema冻结。未把file/SSE局部通过扩大为all-response或全技术门；Web未建、原ID/floor仍待T-08逐项迁移核账。仍executing1/12，独立终审/冻结最终门/合入沉淀/guard pending；UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 23 | from: work | to: work | status: executing | outcome: pass（file/SSE真实终态/字节/状态/许可与CORS、watchdog闭合；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2dbde2ab5 | next: work（goal active；仍1/12，继续T-02）

### Goal round 24 — T-02 compiler/main/activation metadata ownership（2026-10-30）

提交 `e2ed6ba761b1082af34e759012f53d292cd46346`。上一轮为实际进展。真实反例：A编译后main真正park，B在同一owner线程编译并激活，再激活A，其自身route404；原来在main后才读process tables/routes。初期fixture错误prefix await改语言实际`.await`、非JSON错误解析改妥后才取得认可404 red。新增language-owned线程local CompilationMetadataGuard，仅收owned声明数据；原publisher保legacy mirror并经具名bridge写当前collector。lib execute/test、entry source VM、script debug四条正常构建路径在module-init/main可await前freeze并绑定VM私有immutable Arc，服务/borrowed启动及自动路由使用该VM声明；DTO补async_returns，实际external reader按当前owner判断，不再读另一份source crate私有async表。未改变opcode/字节码生成/native注册，未公开VM字段、AST/Store或跨线程传VM。

新真实gap probe最终exit0：invocation-specific精确A/B值、same-name int/str参数/返回、A async25/B普通240、非法整数400、external实际async metadata reader A真/B假，owners/registries0；ordinary-first正例在B激活后清default host，A仍返回42、无listener/provider调用、B仍可用。fixture按provider身份选地址而非激活顺序，stdin shutdown后controller实际join。两新Core定向测试2pass/0ignored：nested真实编译+错误/unwind恢复outer params/return/shadow/async；两真实compiler并发四轮八VM，双方结束才验各自snapshot，cross-thread等待有时限，只Arc metadata过线程。compile-fail反控expected101，CompilationMetadataGuard因Rc<()>不能Send，原VM/context反控保持。旧metadata roundtrip/Pending-nested-unwind/host-selection三原ID各1pass/0ignored；direct真实语言listener与file/SSE终态全路径受影响probe重放exit0。locked三bin external build/root check0；Core140/external16（实际async reader现被调用，较原17少1）/root3/lib-test160 baseline warnings保留。详见 [compiler metadata摘要](attachments/890/work/compiler-metadata-owner-r2.md)，12份raw日志hash及命令/边界齐。

初次external build拒绝private collector访问，已改具名纯数据bridge操作解决，未扩大私有module可见性；失败/旧binary重放不计正例。custom SDK compiler有明确begin/finish/bind-before-share路径；旧unbound raw VM缓存fallback保留，不把它冒作multi-program并发保证，最终owner/facade随T-05退出scaffold整理。helper fmt/Python编译/diff通过；源码/资产/构建在本树，仅md摘要入库。所有进程结束、lang clean；UI7b323bc/Man e6cc06f仍clean未改。888643f2b375仅不同entry函数改动，本轮source VM构建之外未改共享test注册/runner/script/锁/合入窗，无agent/消息/整档/docs_gen/GUI/push。

T-02剩configured launch交接、实例failure/logger终态、ordinary parked/async响应观测、完整实际consumer/physical包API命令SD四仓schema最终冻结。Web未建，原test-ID迁移/floor核账仍T-08；不以新probe替代原required执行。仍executing1/12，独立终审/冻结最终门/合入沉淀/guard pending；UI/Man结构后继继续等完整U890。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 24 | from: work | to: work | status: executing | outcome: pass（compiler-owned metadata与实际main gap/并发/ordinary隔离；非完整交付） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: e2ed6ba76 | next: work（goal active；仍1/12，继续T-02）

### Goal round 25 — 用户限定T-02的阶段收口与暂停（2026-10-30）

最新用户指令收窄本次执行到T-02后停止，不自动T-03；要求代码提交、保树保分支、主计划/风险/契约/交接齐，不合入master、不归档、不清理。该边界覆盖先前自动全链执行授权，但整体目标/12项任务/AC不缩减。本轮不再延伸生产补丁，先列R1..R6关闭条件，再作有界实际审查。

**技术门结论：needs_replan，T-02未完整通过，仍executing/current_step1/12；T-03禁止准入。** 三项实际required反例全部未关闭：①配置端口5077却精确服务于env5078，active config未消费；②真实占口已有bind诊断，自动服务SDK仍child0；③真实parked async wire200/25，但唯一completed事件status=null/bytes_sent0。另源码显示logger仅存tx、丢弃spawn句柄，没有已验证join。R1当前owner端口缺launch/config，R2结果/ready仍依全局诊断，R4日志session所有权需设计；R3可在现有模型内修但仍未实施，不冒“待决策”或后移T-05通过。须先作有界T-02方案修订，不能用持续修补代替阶段结论。

生产代码最后变更 `e2ed6ba761b1082af34e759012f53d292cd46346`；本轮仅探针/文档变更。**代码/候选契约冻结 `33a467a98fd2929640050550b3faf00d5a7824ed`；最终交接封存提交 `76f2069d5d17bb5a591097712c2673458829016f`（仅新增交接Markdown）。** build probe locked exit0，T02 closeout audit exit1/三项fail；后者是未通过证据，不因定位问题改记green。Python编译/helper fmt/diff checks通过；本轮无crates生产改动，不跑根cargo测试档/docs_gen/GUI/无关回归。build日志SHA256 d93bffbf512205b34bdc3708d0c51e2c895e025ea48d6be7c43b252cf775c942；audit日志SHA256 be1c501931841e073411e841d1640e86a45bf4fa915764b232681dc3f0dab625，本地路径均 `D:/autostack/auto-lang/docs/plans/attachments/890/work/`。此前各接缝正例只按原范围/pin复用，不扩成完整部署/发行/全ID绿；父Tauri金样red和存量warnings仍保留。

已形成 [T-02端口/包图候选与阶段结论](attachments/890/work/feasibility-and-contract.md) 及 [可直接使用的交接](attachments/890/work/t02-handoff.md)：公开签名/时序/消息与线程边界、受限25函数载体、真实SDK/Man/UI/生成服务consumer映射、source→owner包图候选、迁移顺序、当前命令/失败观察、887四仓schema约束与待决D-T02-01..03。包/API/schema未最终批准，不能把文档形成冒作T-02完整freeze。共享runner与fixture脚本实物与正式887 blob一致，未写888 WIP schema/共享注册/锁/合入窗口。

语言权威树 `D:/autostack/.wt/lang-890/auto-lang` / `plan-890-dev` 保留；UI同组 `codex/plan-890-dev` /7b323bc、Man detached e6cc06f、Down detached c3464d0同寿命保留，均无本轮产品改动。所有子进程结束，语言最终提交后及UI/Man/Down树clean（ignored target/本地证据保留）；主检出只保本计划簿记和既有他方docs WIP，未提交/混入他方内容。Web主仓及组树均不存在，未创建；未merge/push/归档/guard-remove，未发消息或启实施agent，UI/Man结构后继仍等完整U890。

下一入口为**同Plan、同树的T-02有界方案修订/修复**：先确定per-invocation launch与session结果/日志终态，修R1..R4，再确认R5/R6并真实记录通过。即使之后T-02通过，也必须用户另行允许才进入T-03。本次按用户指令阶段暂停，不标整体goal complete。

stage: goal | plan_id: PLAN-890 | plan_revision: r2 | round: 25 | from: work | to: work | status: executing | outcome: needs_replan（T-02技术门未过；用户要求阶段暂停） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 76f2069d5（code freeze33a467a98） | next: user（阶段暂停；不自动new/work/T-03）

### T-02 r3 有界修订、修复及阶段暂停（2026-10-30）

stage: new / plan_revision: 3 / outcome: pass / changed: T-02、D-T02-01..03、SD-03/05 / authorization: 当前用户交接承接T-02修订与修复，目标/AC/仓集合不变；不恢复T-03全链。现行§5与r3契约取代r2待决机制，r2失败收据原样保留。

stage: work / scope: T-02 / outcome: pass / implementation: `88e604ba0`（launch/Result/观测/cleanup）、`28d5c41c1`（真实main-gap与交换配置反控）、`f56211c12`（独立网络外部关闭修复）。R1实际configured/explicit/hash/port0/交错实例通过；R2占口child1且同进程失败不污染B；R3 wire200/25对应event200/2/completed、parked error500/87及cancel恰一次；R4服务正常/失败/取消后真实network/log join及资源0，cleanup回执与服务成功分离。补充T02-R3-SHUTDOWN1先红101再绿0：owner不再poll时网络仍独立释放端口；其余原audit3/3、session5/5、file/SSE七项精确事件+RST/截断/watchdog通过。无整档/GUI/docs_gen；rootcheck0、5个命名单测各1/1、fmt/diffcheck0；Core139/root3/external16/lib-test159存量warning未冒清零。原Tauri父版本红保留。

stage: review / review_scope: phase(T-02) / reviewer_limitations: 实施会话重读冻结source/diff及输入/断言重建；没有独立agent或final上下文，不抵AC-13 / plan_revision: 3 / reviewed_commit: `f56211c12f6cc8062f0cbfb5a4f0d03c8e4a958c` / base: `76f2069d5` / outcome: pass（仅T-02技术可行性/执行合同） / findings: T02-R3-COV1真实交错配置oracle缺口已补且swap反控拒绝；T02-R3-SHUTDOWN1独立关闭回归已修且反例先红后绿；无剩余T-02必需缺口，无未批准延后/workaround或新增非合同债候选 / spec_delta: r3 launch/session规则与现行SDK/部署Spec一致，目标清单不变；规范未发布 / pending_closeout: 后续物理迁仓/原ID/消费者/发行/应用、T-11独立终审和所有merge-owned门。

阶段交接提交 `4ec696a631058a92c4f282fdab054a86dbadc2ff`；主计划只作默认检出簿记，不混入其其他docs WIP。语言权威树`D:/autostack/.wt/lang-890/auto-lang`分支`plan-890-dev` clean；UI`7b323bc`、Man`e6cc06f`、Down`c3464d0`原树未改；Web未建。原887共享脚本blob保持`e8458b27e32521dff62ee611eb1814193c86b445` / `cbff4c581eadbf763eeda278c40c1167b7b9053d`；未写共享runner/注册/锁/888 WIP、未发消息、未合入/push/归档/清理。完整命令/结果/本地证据SHA256、Spec输入hash与适用边界见 [r3交接](attachments/890/work/t02-handoff-r3.md)。

当前状态 **executing / 2 of 12**；next: user（按原边界阶段暂停；明确允许后才T-03）。所有树/分支/本地证据保留。

### r4 用户指定阶段串行及888正式基线门（2026-10-30）

stage: new / plan_revision: 4 / outcome: pass / changed: §5执行顺序、T-03前置同步、各启动单元暂停边界 / authority: 用户本次明确指令 / unchanged: 目标、完整AC、12任务、T-02技术冻结`f56211c12`及其r3证据 / documentation_commit: `ed44fd0bd6ea063e8e4977454bd5ef04455aa762` / worktree: 原`plan-890-dev`，文档提交后clean / evidence: [阶段规则及交接](attachments/890/work/stage-sequencing-r4.md) / verification: 只核文档、状态/任务数、链接及限定diff；无生产改动，未构建/运行测试 / next: user（保持executing2/12、阶段暂停；未核实或宣称888已经合入，未联系或操作888会话，未合并master，未启动T-03）。

### r5 T-03～T-04阶段收口（2026-10-30）

stage: work / scope: T-03,T-04 / outcome: foundation-and-targeted-acceptance-complete / status: executing4/12 / full_tm_gate: NOT_PASSED / review: same-session phase audit only, independent final AC-13 pending / next: user（阶段暂停，不进入T-05）。

888正式交付`5a6c23afa`、归档`107148fcd`、收尾`129ef6057`均为master祖先，归档合同10/10及复审/门禁/guard收据已核。原clean890分支`ed44fd0bd`合入正式master`129ef605763e59ca13bf312fd4a391eded10c13a`后为`29ed9d7bde334ae14e21223c935a4f3974581f71`；无需手工冲突处理，888 gated tracing/reporter与890 launch/session实现均保留。未联系或操作888会话，未使用WIP；共享scripts、.cargo/config.toml及AGENTS与该正式master保持一致。

实现冻结：语言基础`84779b2b1`、最终测试装配`1a253950810de67f16f487c7eb7a2745f1061bb0`、交接`e45399a633a1dcf74f70943cf2d2c663791589a6`；新Web本地bootstrap`8ff4966`，唯一阶段树基础`26259642f56656547db7bc07a68c35a2bd225edd`。纯配置单源移至auto-web-contract，Core同类型re-export；实际Cargo选中来源进入SDK身份清单，重复物理合同负控被links命名空间拒绝。四受限生成模块迁auto-generation-core，旧路径薄委托；原15测试体/名保留，正式ID登记仍T-08。Web runtime/生成器算法未提前迁移。

定向证据：Web workspace check、语言locked check、SDK边界check、482包真实依赖闭包均通过；无UI/Man/Tauri/混合facade依赖及Core→Web实现反向边。真实consumer验证同类型/Arc、严格配置、普通SDK42及旧新生成路径；原载体15单测、合同消费者及真实companion正反例通过。T-02新包源重建后audit3/3与session5/5通过，缺adapter child1/无监听、普通42 child0/资源0保持；未变transport/body/compiler部分仅按交接声明的代码/输入边界复用，不能扩为完整迁仓证明。源码机械守恒与fmt/diffcheck通过，存量warning未宣称清零。

完整tm结果不得误读为pass：7183义务/1186叶目录中4直接叶扩750模块/5483义务；58已执行模块55通过、3失败，另23原条件NOT_APPLICABLE。三失败与888归档P763-R1相符（aavm compile corpus、compile use corpus临时Rust缺auto_lang；a2r golden INVALID_BYTECODE:60），相关产品源本轮未变，保留原红而非删除。旧plan734 HTTP测试缺显式SDK host且等待无界，定向装配修复后原4测试完整重放通过，4个直接Core叶原23项重放通过；两重放均为诊断/定向验收，**不是修复后750模块新源身份完整认证**。未重跑大闭包冒通过；未完成项及已有红须在T-12最终冻结组合依新tm规则处置/执行，不能按本次55pass豁免。工具当前未覆盖新Web源身份，T-08须扩owner/来源/consumer与实际收据，现以锁/源码hash补充记录。测试生成6个既有报告原件先本地存hash后按冻结Git恢复，未提交报告；完整本地日志/hash见交接。

阶段审查仅确认本单元AC-01..04涉及的基础/类型/SDK/共享承载部分。未做独立终审，不设置reviewed，不宣称AC-03运行时、AC-04完整生成或整体交付。拟Spec delta：纯合同新owner、四模块载体、Core实际源身份及显式SDK测试装配；canonical Specs/ledger留最终沉淀。T-05须用真实Web installer替换临时测试adapter并退出Core runtime桥；T-06完整迁生成器；T-07消费、T-08逐ID/四仓tm、T-09/10发行/应用、T-11独立审查、T-12最终门/合入/沉淀/安全收尾均保留。

[阶段基线与有界合同修订](attachments/890/work/t03-t04-baseline-r5.md) / [冻结交接、命令、退出码与本地SHA256](attachments/890/work/t03-t04-handoff-r5.md)。所有树/分支保留；未合入890至master、未远端发布、未归档或清理。 收口只读观察到master已前进至`60bda3276a0781a5da60452eb317fa399428980a`；本次同步/证据基面仍固定于正式888收尾`129ef6057`，未在阶段末静默二次合并或扩大验证范围。

### T-07 全链恢复与消费迁移（最新授权）

stage: work | plan_id: PLAN-890 | plan_revision: r6 | scope: T-07 | outcome: stage-owned-implementation-and-targeted-acceptance-complete | status: executing7/12 | worktrees: original lang-890 five-repository group | code_commits: language 163d342a6, UI 672955d, Man 9f8ffc2 | evidence: [T-07实际命令/数字/原失败与限制](attachments/890/work/t07-handoff.md) | final_review: pending T-11 | frozen_union: NOT_PASSED, owner T-12 | next: work T-08（本次完整goal授权覆盖旧逐单元暂停，不等待再次启动）。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 26 | renewed_session_round: 1 | authorization: latest /goal 完成计划890 renews the full chain; prior r2 round25 pause is historical | from: work | to: work | status: executing | outcome: pass（T-07阶段；非work-complete/非完整计划） | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 163d342a6 | next: work（T-08执行中，T-09/10/11/12仍保留）

2026-10-31 / stage: work / plan_id: PLAN-890 / plan_revision: 6 / outcome: continuing（T-08 complete; executing8/12） / worktree_path: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commit: 38f68d8e9 / dependency_revisions: Web f4cc93c, UI 672955d, Man 9f8ffc2, Down c3464d0 / task_ids: T-08 / evidence: attachments/890/work/t08-handoff.md; 46 recipes/7320 obligations/1211 leaves, Web217 pass+2原ignore，语言保留28 pass、media54 pass、governance9 pass；四仓真实列举守恒与floor均过，partial/list-only负控exit2，不冒最终执行。原Plan736缺显式adapter实际TIMEOUT120.046s修复后双profile约2.7s及六兄弟消费均过；原失败不覆写。全局证据预算预览exit2/64.88GB未达标，无删除；Plan本身低于500MiB / blockers: none for next implementation; final gate NOT_PASSED / next_responsible_role: work T-09 → T-10 → independent T-11 → frozen gates/merge T-12, authorization inherited.

2026-10-31 / stage: work / plan_id: PLAN-890 / plan_revision: 6 / outcome: continuing（T-09 complete; executing9/12） / worktree_path: D:/autostack/.wt/lang-890/auto-lang / branch: plan-890-dev / code_commit: f739c3751 / dependency_revisions: Web 9ddb7fe, Man cb4ab3d, UI 672955d, Down c3464d0 / task_ids: T-09 / evidence: attachments/890/work/t09-handoff.md; 冻结发行45过、旧新Core历史还原/失败后真实消费14过、SDK库217过+2原ignore；Source/WIN zip 3.59/19.40MB；严格真实收据/源码/输出hash验证通过，missing/duplicate/zero/not-run/错hash等治理10通过，用途守恒19通过。真实Git检出库214过/3长路径环境失败/2原ignore，短路径5项重放均过；1次LEAK原始结果保留，命名隔离18.466s通过且三端口已释放。D盘曾满而失败的诊断收据保留，自动审查两次拒绝缓存删除，未删除；转C盘继续，后续D盘空间恢复非本会话删除所致。语言运行SDK源87a09af94与当前f739 SDK代码/锁/stdlib输入相同，治理/验证脚本差异另记。 / blockers: none for T-10; final four-owner actual gate NOT_PASSED / next_responsible_role: work T-10 fixed supported Blog source + affected Vue/native, health and SD freeze; then independent T-11 and T-12 gates/landing under inherited grant.

### 2026-10-31 T-10 final work handoff — r6 / execution_done10/12

stage: work | plan_id: PLAN-890 | plan_revision: r6 | scope: final implementation T-01..T-10 | outcome: pass | status: execution_done | current_step: 10/12 | worktree: D:/autostack/.wt/lang-890/{auto-lang,auto-web,auto-ui,auto-man}, Down same fixed sibling | implementation_commits: language `ede598998566ea890ff67170bb9744f3bc537a58`, Web `9ddb7fe5bb1886166073b5805c38895f3da195ff`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466` | evidence: [T-10 self-audit, exact command/profile/result/hash/limitations](attachments/890/work/t10-handoff.md), [SD proposal](attachments/890/work/t10-spec-delta-proposal.md) | actual_results: fixed source314 unchanged; default-profile generated Rust cold build2m31; complete Vue15/15 and native22/22; UI module866 pass/2 original ignore/4 new controls; source-bound discovery46 recipes/7329 obligations/1213 leaves; purpose/conservation19 pass | source_identity: `0bb28657a28e2405d21bf42da70bd3b47ab99c45d773aae8617ad0d5ce981bb1` | health: all four candidate whitespace pass; owned fmt pass; changed-source warning baseline4/candidate4/new0, not whole-tree warning clean | limitations: original failures retained; runtime profiles after explicit cold compilation; SDK history Core valid shape only; controller preview tree required explicit owned cleanup (3 exact children, 3 ports v4/v6 rebound), native own-child assertions independently pass | independent_review: NOT_RUN, T-11 on the same trees and independent context | final_frozen_tm_and_four_owner_gate: NOT_PASSED, T-12; old AAVM reds remain named, no list-only/discovery/purpose classification substituted for execution | merge_specs_archive_guard: NOT_RUN | next: review.

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 27 | renewed_session_round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: ede598998566ea890ff67170bb9744f3bc537a58 (implementation/governance; docs-only handoff follows) | next: review | authorization: current full-chain /goal remains active; historical r2/r4 stage pauses do not apply; no implementation agent or cross-chat messages.

### 2026-10-31 T-11 independent final review — needs_fix（第1轮）

stage: review | review_scope: final | reviewer_identity: `/root/plan890_final_review`（独立上下文、原权威五仓同树，无第二检出/实施代理） | plan_id: PLAN-890 | plan_revision: r6 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | reviewed_commit: `e1ece5d45138040bcb67c4ecf5060cca53099019`（实现`ede598998566ea890ff67170bb9744f3bc537a58`） | base_commit: `6e48fe11c0be28159a950413a14f7964b3659389` | dependency_revisions: Web `9ddb7fe5bb1886166073b5805c38895f3da195ff`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | spec_delta_sha256: `b2a47e3a5891e79b25bcf842462bdbc33aa222117c373355c9b192aed47e7e9e` | evidence: [完整14AC/18V/9SD映射、风险核查及最小关闭条件](attachments/890/review/t11-final-review-r6.md), SHA256 `6ec2e2666d46504c9963cf09bd8625f1e4b162315f4ae8e472c62c889f39377b`; 本地`t11-final-baseline.json`含逐Spec/锁/来源/原始反例hash | pending_closeout_gates: 全部T-12仍未执行 | next: work（同树修复R1/R2，不合入）。

- **890-R1 / HIGH / AC-08/09/12**：真实公开fixture接受零必需执行的四仓收据，exit0/pass/11过0失败0pending；实际目录无条件20用例缩19仍被collector接受；真实发行45收据删去5类必需command/exit/log/hash后逐项仍被接受。缺口为必需执行证明可选/自报适用集合未守恒，不是新签名或安全产品要求。原真实45收据及完整20-ID控制仍为合法对照。关闭须真实消费者拒绝遗漏、重复、无依据排除及不匹配输出，原条件/合法runtime与reuse形态保持；按复审包做有界治理检查，不追加全目录门。
- **890-R2 / HIGH / AC-03/06/09/12**：真实已安装Web CLI，五相关源文件与候选逐字相同；`run_file`所走`script_entry::run_with_path`新线程未传TLS launch。无配置env5453对照正确；有效development配置5457却在env5458精确响应；有效proxy配置5462却在env5463/0.0.0.0响应，非法Host应400实际200。三次均shutdown202/exit0/端口释放。原T09/HTTP测试把env端口设成config端口，遮住配置丢失。关闭须不同端口、loopback/有效profile/配置身份及Host真实拒绝，保留无配置和普通无服务文件行为；修复文件入口共享线程交接并重建受影响SDK/发行身份。
- 独立新增检查：原ownership10/10、Man transaction6/6通过；R1三类反控与R2真实wire分列为失败。Parent核16份T10原日志/产物大小与hash全相符。初始无效proxy夹具、任意启动等待超时和UI独立探针setup/已知边界不冒新产品缺陷；原红/ignore/限制不覆盖为绿。
- AC-01/02/04/05/07/10/11保留有界pass；AC-03/06/08/09/12 fail，AC-13未获pass，AC-14 merge-owned partial。SD-02/03/05/07/08待缺口关闭，不通过软化规范代修复。重开T-05公开配置入口、T-08、T-09及T-10冻结，完成数10→6；T-02/04已验证端口/类型/DAG保留，修复时补文件入口交接反例。所有必需缺口按在途阻断追踪，不转非阻塞债。
- 五候选HEAD和clean状态最终读回均未变；review环境hash `aa2ef3ba...`与discovery `a92a86a7...`不同，逐仓源码输入相同，不冒combined identity相同。主分支同期正式交付894（最终观察`f0898d6af`）；后续合入前须同树协调正式改动、刷新受影响目录/来源/消费者身份，原复审不覆盖未来组合。没有全局测试/GUI重放、代码修复、canonical/账本写入或合入/归档/guard移除。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 28 | renewed_session_round: 3 | from: review | to: work | status: executing | outcome: needs_fix | current_step: 6/12 | final_review_attempt: 1 | repair_cycles_started: 1/3 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: e1ece5d45138040bcb67c4ecf5060cca53099019（review baseline；后续仅复审簿记不覆盖代码） | next: work | authorization: 当前全链授权继承，原树有界修复后受影响复审；不请求再次许可，不执行T-12门或合入。

复审阻断按AGENTS登记在同树`KNOWN-DEBT-AND-RISKS.md`，明确OPEN/必修而非延后债；仅该文档提交`46547a0f898cfe29745e7bfcee62dc874f9cfb64`，候选实现不变、提交后clean。主检出只提交本计划与两份MD复审/冻结摘要；所有raw探针/JSON/log本地保留，未暂存。他方894及其他文档WIP未混入；本轮仅文档目标校验，无新增Rust测试或全局门。

### 2026-10-31 work repair cycle 1 — execution_done10/12

stage: work | plan_id: PLAN-890 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | base_commit: `46547a0f898cfe29745e7bfcee62dc874f9cfb64`（复审后只读簿记），正式894/主计划同步`c73e88f86` | code_commit: `2506549321ad8f4699089a7b0dce53d74f78b046`（实现74e13580a、证明分支0fdca25c0、原格式恢复b446256b5、最终目录250654932） | dependency_revisions: Web `e01232b3d449ea590a85d9458d553ab17b9456a3`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | task_ids: T-05/T-08/T-09/T-10 | status: execution_done | current_step: 10/12 | findings: R1/R2 implementation repaired, independent affected re-review pending | evidence: [修复交接/关闭表/复用边界/命令及本地索引](attachments/890/work/repair-1-handoff.md), SHA256 `d72f61ea3d6409605dfc56ec6a92d496276c4c79fbe56c69d5e278531a6eb628`; [当前SD增量](attachments/890/work/repair-1-spec-delta.md), SHA256 `3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172` | source_identity: `815c5287416833e52b7656e0c07764600a53ca532b883bd13168b728abf1ce8f` | blockers/user_decisions: none for affected review | next_responsible_role: independent final review（同权威树、原findings+合法对照+相关分支；未变AC证据保留）。

R1真实关闭：公开零执行收据exit2拒绝，完整20原ID实际执行且证明被接受，原遗漏ID的20→19与同步短日志仍拒绝；14治理控制含nextest/libtest/Python及原负Case元oracle、缺command/退出/输出/hash/排除/重复/过滤/旧source组合。原真实45按原输入仍有效，5项删证明反例全部拒绝，当前组合不能冒借它认证新SDK字节。模块消费者49/11.119s、用途守恒19/30.132s；真实47 recipe发现后7367义务/1220叶，76职责组全覆盖。894正式新增32观察/7叶仅用途pending，不变注册/样本/预算/断言/条件；两周期清单完整47。API20真实墙钟9.570742s/工作CPU8171.875ms/作业峰334266368bytes，非P90或整目录计量。

R2真实关闭：Core host模块6/0.26s、Web HTTP文件3/5.85s；新不可变19包源candidate=false、locked冷构建2m13/安装720文件，真实CLI fresh3+relocated3精确nonce、配置端口≠环境端口、loopback/profile/hash、Host400、shutdown202/exit0/端口回收；A已不存在且移动前全树无reparse。当前SDK库217/203.198s+2原ignore，源码/安装清单测试后重核未变；必要Rust生成运行兄弟未改代码也以新不同端口driver真实重放。两新ZIP共22986983bytes并读回完整唯一清单/CRC；raw/缓存未入库。原Core128/Web13/测试140 warnings保留，无新warning/debug或必需延后；受改Rust fmt、五树whitespace/clean均通过。

当前SDK实际生产者Lang74e13580a/Webe01232b；后续语言变更是未打包治理/目录，不篡改其原来源。SDK输入hash `3bc6c24bea6ac48d236d41f4af87c2412816714f930f886ae4fa83e38a094a10`/config `d595de8766d93c16257a43fcc83b1d229c3f63d8f98466d84c28b59946037066`与原始日志hash见本地`repair-1-independent-tests.json`（SHA256 `5a88ff4552d9c6a827008894e54f8d3fb4c4a07a34beabcbbbe821edee2c8f72`）。SDK当前执行不是复用旧SDK测试数。旧45/历史14、未变HTTP算法/生成器/Man事务/UI与固定Blog15/22仅在各自原输入/未变路径复用；**新完整当前组合发行/四仓/完整tm门仍NOT_PASSED，既定T-12冻结统一门前不认证新组合**。原3个AAVM红与原ignore/manual/环境限制保留。无新GUI/全目录测试body/周期批次/远端/合入/规范/账本写入/归档/guard移除。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 29 | renewed_session_round: 4 | from: work | to: review | status: execution_done | outcome: pass | current_step: 10/12 | repair_cycle: 1/3 | previous_final_reviews: 1 needs_fix | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2506549321ad8f4699089a7b0dce53d74f78b046 | next: review | authorization: 当前完整goal继承；无必需新决定，不重复请求许可；所有T-11/12仍保留。

### 2026-10-31 T-11 affected independent final review — needs_fix（第2次）

stage: review | review_scope: final | reviewer_identity: `/root/plan890_final_review`（独立上下文，同权威五仓，实施冻结） | plan_id: PLAN-890 | plan_revision: r6 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-890/auto-lang | branch: plan-890-dev | reviewed_commit: `2506549321ad8f4699089a7b0dce53d74f78b046` | dependency_revisions: Web `e01232b3d449ea590a85d9458d553ab17b9456a3`, UI `9aad12ccbc5f01d5241eb95f657666a943667ff2`, Man `cb4ab3dd5d411314cce10b129eeb3b433a11c466`, Down `c3464d0a0d36e4f8a7988b0c0c26448fc52361b4` | source_identity: `815c5287416833e52b7656e0c07764600a53ca532b883bd13168b728abf1ce8f` | spec_delta_sha256: `3ed4dfec05c803c569308d1e503b8cb2b0d44008942b11c5177f560710047172` | evidence: [受影响AC/V/SD、实际公开入口反例与关闭门](attachments/890/review/t12-affected-review-r6.md), SHA256 `e3f46655d363335ec055d64eee25ba94f64cfcf74bc2b7c276c422021b9a62b6`; 同目录`t12-final-baseline.json`保存来源/锁/Spec/输入及raw哈希 | pending_closeout_gates: 全部T-12未执行 | next: work。

- **R2已关闭**：重建安装工具原公开CLI独立3/3过，不同config/env端口、loopback/profile/hash、proxy非法Host400，shutdown202/exit0/rebind；无配置/ordinary控制保留。当前SDK217/2及未变生成Rust配置证据限原输入复用；不重开T-05/T-09。
- **R1原A/B/C反例已闭，R1-D / HIGH / AC-08/12仍OPEN**：实际UI源清单3128列举/3110选中，删除原必需`auto-ui-compiler\ta2ui::export::tests::test_export_button_action`并同步listing/run/selected/count/hash→3109，生产端原规则拒绝missing1/suppressed1，验收端及公开four-owner检查仍PASS；实际Man原floor281→280同样被生产端拒绝、验收端放行。公开整包因特意缺发行腿为partial/exit2，未冒整包PASS；隔离协议夹具仅替换外部fingerprint，不声称执行测试体。缺口是原必需分母未在最终消费端复核，不是新增签名或防篡改要求。
- 原14ownership独立通过，真实20证明和遗漏/重复/错cwd/无依据排除/错实际ID反控通过，合法原ignore N/A接受；真实旧45及5个命令缺失反控、当前组合拒旧SDK、16份repair raw哈希均核。五HEAD最终clean不变，12个canonical目标hash未变；SDK打包源码与实际74e/e012生产者相同。无全目录/GUI/周期批次、代码修复、canonical/ledger或合入操作。
- 仅重开**T-08及依赖T-10**，实际完成数10→8；AC-08/12 fail、AC-13未获pass，其余已闭/原输入有界证据保留；SD-02/07/08 partial，不软化规范。R1-D为在途必修阻断，不能转为非阻塞债。
- **连续两次final needs_fix诊断门**：下一轮Work必须先记录并核验“原分母为何只在producer核、repair1一用例正控为何遮蔽一致删项”的原因与路径矩阵，按实际完整源清单建立正控、一致删除必需UI ID/Man低于floor反控；核实设计/oracle调整后才提交修复并再次回交。第2修复周期不重置自动修复上限3。

stage: goal | plan_id: PLAN-890 | plan_revision: r6 | round: 30 | renewed_session_round: 5 | from: review | to: work | status: executing | outcome: needs_fix | current_step: 8/12 | final_review_attempt: 2 | consecutive_failed_final_reviews: 2 | repair_cycles_completed: 1/3 | next_repair_cycle: 2/3 | worktree: D:/autostack/.wt/lang-890/auto-lang | commit: 2506549321ad8f4699089a7b0dce53d74f78b046（review freeze，后续仅阻断簿记） | next: work | authorization: 当前全链继承；必需诊断/oracle调整门先行，无新用户决定，不重复许可。

本轮阻断已在同树`KNOWN-DEBT-AND-RISKS.md`登记，文档提交`e6f15ec01`，实现源码仍为复审冻结版本；代码未修，提交后clean。主检出仅提交本计划和两份复审/SD摘要，raw证据不暂存。文档任务数/状态/摘要hash及限定diffcheck通过；首次任务计数探针误将§10历史T-05/06行计入而失败，限定§8任务表后确认12项/8已完成，不涉及产品测试失败。

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

旧885历史收尾等待已由用户“885已经收尾了。”关闭。本次实际Core-only外部消费者无需Web装配即自动提供HTTP，已编译、真实精确JSON/关闭验证并提交。与r1无Core→Web实现边+全部Web实现迁出同时保持该消费的静态包图冲突，需选择具体版本迁移方案或兼容门面包图修订；原合同§2.2/§10 D-04要求实质决定，不自动削减支持。两方案、实际调用链、限制和日志hash见[决策说明](attachments/890/work/sdk-default-decision.md)，来源与共享窗见[入口摘要](attachments/890/work/source-and-window.md)。等待用户输入工具实际答复；当前全部生产代码未改，正式Web仓未创建，T-01/02均未冒完成。

2026-10-30 D-04最新裁定：用户明确“显式安装 Web adapter”。以r2当前合同为准，前述未答复/等待/blocked是历史。下一步work，完整交付范围与所有其他门保持；0/12不冒完成。


### 2026-10-30 T-05～T-06 恢复入口

用户“继续后续阶段”启动r4单元2：按T-05运行时→T-06生成器串行；本单元结束提交/更新计划/交接后暂停，T-07及合入/归档/清理不在本单元。原语言树`e45399a63`、Web`26259642`、UI`7b323bc`、Man`e6cc06f`、Down`c3464d0`均核实clean，复用全部原树；主检出仅既有文档WIP，保全不纳入。完整tm旧收据仍NOT_PASSED。

| 规则/入口 | 实际链路与所有权 | 区分性控制与关闭条件 |
|---|---|---|
| T-05 / AC-03/06/07 | Core编译元数据/VM本线程→HttpVmOwner/owned消息→Web host→Web transport/dispatch/scope/file/upload→真实body终态与network/logger join。现有external-runtime探针已验证端口，当前仍按路径编译Core原件；本轮改为正式Web源，删除Core临时run_existing桥及领域运行时 | 重放audit3/session5、file/SSE/upload/RST/两实例和缺provider/plain控制；编译包图无Core→Web实现或Web→UI/Man/Tauri，检查无旧运行时副本；阶段新增destination测试不替代T-08逐ID登记及T-12冻结门 |
| T-06 / AC-04 | 现有api_gen入口→共享四模块单源→HTTP服务/客户端生成切Web；Tauri/非HTTP留原owner薄装配 | 885约简及实际生成Rust编译/运行、非法兄弟/非HTTP消费；源/模板/锁与真实依赖相符，不手修生成产物；独立发行仍T-09 |

stage: work / scope: T-05,T-06 / outcome: executing / next: work / current_step: 4/12 / independent_review: pending T-11 / full_tm_gate: NOT_PASSED, final frozen union T-12。


### 2026-10-30 T-05～T-06 r6阶段收口

stage: work / plan_id: PLAN-890 / plan_revision: 6 / scope: T-05,T-06 / outcome: stage-owned-implementation-and-targeted-acceptance-complete / status: executing6/12 / worktree: 原五仓组树 / code_commits: language `1b177ebab1dab4203588abd77047026aaa366eb9`, Web `d8c889c786d83778f4b46efd7e73d6a8fb2935bf` / evidence: [r6交接及命令/数字/本地SHA256](attachments/890/work/t05-t06-handoff-r6.md) / phase_audit: 实施上下文重建源码/图/原断言与控制，不冒独立终审 / full_product_check: FAILED（UI back-proxy旧Core引用，T-07既定消费迁移；不可冒baseline红） / full_tm_gate: NOT_PASSED（旧750并集的3红/中断与未执行原样保留，新身份最终门未跑） / remaining: T-07真实Man/UI装配及消费、T-08逐ID/四仓模块来源与执行账、T-09发行恢复、T-10应用/full自检、T-11独立审查、T-12冻结门/合入/规范/归档/guard清理 / next: user启动r4单元3/T-07。本单元完成后暂停；无远端/合入/归档/清理，无共享runner/注册/888脚本改写。

本轮发现并在原合同内关闭上传端口的环境重读漂移：添加中立executor的默认phase入口，Web转原captured-limits单源，ordinary facade与父版逐字相同。正确wire上限变异拒绝、还原后scoped413/ordinary接受并清理；初版raw max-file错误oracle、时间戳还原复用旧变异binary、libtest错误隔离与中断、stale probe lock初拒绝、首次编译/路径/命令失败均在交接具名保留，未覆盖为绿。最终nextest隔离109/109，源/许可/断言/条件未削减。metadata Web482/Core475/contract16闭包及禁止边负向检查通过；四整文件同formatter机械相等，Core无run_existing残余。规范仅提议实际owner/API/上传phase端口；canonical/账本留T-12。既存Core128/Web13/facade1警告明示，不宣称全库warning clean。


### 2026-10-31 T-10 bounded consumer repair (r6; execution continues)

固定Blog `b42a660` 的实际Vue输入得到保存PUT 422；独立API13项通过，真实请求缺body/expected_rev。已定位既有Vue store发射器将同名局部快照误当state ref，错误发射`snap_body.value`/`snap_rev.value`。T-10在原UI owner补有界正确性修复：按块追踪局部绑定；局部保持原名，冲突状态仅在实际使用时发射内部别名，保留委托函数/lambda捕获及旧公开getter。无新AST/运行时协议、产品目标、AC、仓集合或预算变更；SD-09增补现行`auto-ui-compiler/project.md`触面及plans索引。原ui_gen整模块866通过/2原ignore，旧断言未改；新增4反例覆盖快照字段、块/循环/闭包、同名初始化与委托捕获。首次0选、夹具解析红、初版源码形态控制红与缺失SQLite生成缓存输出均保留；不得视为完整应用绿。初版工具真实UI7项任务通过但重启有Windows esbuild目录访问失败，整profile13/15，保留失败；最终修订工具仍须重建并复跑Vue/原生完整profile。完整四仓门、独立终审、合入/归档/guard均pending。
