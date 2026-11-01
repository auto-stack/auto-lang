---
plan_id: PLAN-906
status: archived
feature_name: 语言仓独立构建与Web轻合同/组合依赖边界收口
author: [Codex]
created_at: 2026-11-01
updated_at: 2026-11-01
plan_revision: 1
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/language-sdk/project.md, docs/specs/language-sdk/http-host-port.md, docs/specs/repository-boundaries/web-four-repository-delivery.md, docs/specs/auto-api-gen/project.md]
new_spec_components: [docs/specs/repository-boundaries/standalone-language-build.md]
touched_goals: []
affects: [auto-lang-core, auto-lang, auto-api-gen, auto-generation-core, auto-web-contract, sdk-delivery]
depends_on: [PLAN-890, PLAN-900, PLAN-901]
---

# [PLAN-906] 语言仓独立构建：Web轻合同与组合依赖边界收口

## 0. 变更摘要

将900-N7从债务登记转为正式收口Plan，架构依据[Design53](../design/53-independent-build-and-generated-cargo-governance.md)。目标是Core独立消费及完整语言workspace在无auto-web兄弟检出时可从固定、可获得来源解析/构建/测试，不借全局旧二进制、其他checkout或隐式环境兜底；Web服务继续由Web owner显式装配。

本次发现范围不仅Core→auto-web-contract：facade auto-lang和auto-api-gen也直接依赖auto-web。前两阶段必须核完整闭包及公开兼容并冻结方案，不能仅改Core一条path就关闭全仓独立问题。候选中涉及公共API/类型/域owner迁移或新依赖发行渠道的具体决定，必须先展示可审阅合同；不预定把整个Web实现/政策搬回语言。

当前drafting0/8，只立约；未建树/编译/测试/改清单源码/拷依赖/发布/改环境或删除任何仓。用户本轮没有实施授权。

## 1. 目标

- G1：L1 Core SDK独立消费和L2语言workspace根完整构建分开验收，消除对环境中Web兄弟路径的要求，保语言→UI/Man/Web实现禁止边与单一类型身份。
- G2：Web严格配置/错误/hash/预算/策略、HTTP宿主端口及普通语言/服务装配的现行行为不降级，域算法只归实际owner。
- G3：新来源/版本/资源/锁/安装/发行合同可fresh/locked、重定位及错误恢复；原测试义务与消费者支持面无未批准丢失。
- G4：不靠删workspace成员、隐藏dev/optional依赖、复制实现、删links守卫或旧编译物实现“独立”；把900-N7及更广依赖差额诚实关闭。

非目标：重开890全部Web搬迁、Web新功能/版本升级、重新迁UI或实现UI014/007/AG020、完成905 Cargo缓存治理、改配置schema/策略门限/运行引擎、建立新仓/registry/远端发布、复制Store/AST/VM或第二HTTP实现、改变默认服务支持面、清他人缓存/源码。

## 2. 架构方案

当前schema数据、配置解析/纯政策与运行实现分层保留；Core只依赖真实必要的中立类型/端口或可独立获取的固定轻合同，不得反向依赖Web实现。Web继续消费唯一合同并通过已交付host port装配；UI/Man/生成工程选择Web能力时依实际owner公开入口，不借语言SDK隐式启动第二实现。

T-02比较并在原约束内冻结候选（不是全部都实施）：

1. 语言真正需要的中立宿主数据/端口归合适基础owner，Web政策留Web；旧Core http_service_config再导出类型与配置调用需逐项兼容/批准迁移。
2. 固定轻合同来源通过已有可用的受管依赖渠道消费，验证完整源码和离线输入；若需受管vendor快照必须唯一publisher/版本/来源hash，不形成可编辑第二套实现。未发布registry版本或任意绝对path不算方案。
3. facade/generator的剩余Web实现调用由Web owner/显式组合适配承接；必要compat门面按真实版本/调用者迁移，不悄悄删除功能或把Web实现复制回语言。

方案若需改变已批准公开兼容、迁移crate/owner、增加远端动作或扩大scope，T-02输出具体设计差额/AC/迁移和恢复方案供用户确认后才进入T-03；若无合法可行方案则记录needs_replan，不能将“仅Core能构建”改成整个任务通过。一个主Plan贯穿调查、选择和已批准实施，不为探针另拆号。

## 3. 技术栈

Rust/Cargo和现有source_owner/rust_workspace/SDK投影/安装/四仓fixture、HTTP严格配置与host port、Git来源/锁。Cargo解析包括normal/build/dev/optional与工作区清单，不能只看normal tree。官方Cargo配置/patch依据见Design53；实施核本机版本与实际argv，无远端发布默认权限。

## 4. 需求分析与背景调查

### 4.1 授权和已交付来源

2026-11-01用户明确要求把Cargo治理和Web兄弟检出问题分别做成计划文件，当前只授权规划。900/901已归档、018已交付，899等独立路线不追加全局前置；018的SDK消费与901完整发行闭包成果保留，不能恢复旧test-*或破坏语言零UI依赖。905研究独立于本计划，身份变化向其提供正式来源，不混算收益。

已读docs/specs/overview.md、language-sdk相关现有合同、repository-boundaries/web-four-repository-delivery.md、auto-api-gen/project.md及Web docs/specs/project.md。Design50/52为边界目标，900-N7不是已实现结论；本计划纳入更完整反例，不修改旧归档。main b1a4871aa6a8d960130fc2e1aed8bf6dc4183c5a；Web ec9c4bac90fed62e6b651df7789153149e8a9473；UI6ab7f2def391eca584ebbbbbf375ef75a7f64ec9；Man cbbe6573e2d40e3237f52718dd02a3cd065d5cbd，开工重绑。他方文档/号源WIP保全。

### 4.2 实物闭包与未决范围

| 实际路径/调用 | 本次观察（只读） | 本Plan必核 |
|---|---|---|
| crates/auto-lang-core/Cargo.toml | auto-web-contract path=../../../auto-web/crates/auto-web-contract，version=0.1.0 | 无兄弟检出即清单解析风险；Core独立消费不等全workspace独立 |
| Core src/http_service_config.rs | pub use auto_web_contract::*；自身只保ACTIVE_SERVICE_CONFIG/set/take | 导出面不只一个DTO；不能简单改名或复制同名Rust类型 |
| Web crates/auto-web-contract | Cargo继承Web workspace.package，links=auto_web_contract；依赖serde/serde_json | 轻包独立来源还要处理workspace继承/许可证/来源及links单源，不删冲突守卫冒解决 |
| 合同src/lib.rs | HttpServiceConfig/ServiceLimits/CorsPolicy/resolve_service_config/CorsOutcome/EffectiveClient等，含严格解析、默认值、hash、CORS/Host/可信代理纯策略 | 哪些数据是Core必须、哪些政策归Web逐项审；不把整个纯策略包当语言基础简单搬回 |
| facade crates/auto-lang | Cargo直接auto-web；src/lib.rs含Web generators/公开转发；bin/auto-case.rs装配install_http_service_host_if_absent | 普通语言、旧生成API、测试工具实际服务需求需具名接收；不能只把optional=false改true（Cargo仍要解析） |
| crates/auto-api-gen | Cargo直接auto-web；src/api_gen.rs兼容再导出并委托generate_api；同名feature转发Web | 若目标完整语言workspace无Web检出，它也必须闭合；跨仓Man/UI消费者和锁/发行源身份必须接续 |
| source_owner/发行 | auto-generation-core/source_owner.rs验证Web包source_repo/路径/hash；UI rust_ui.rs与901引入Web包完整闭包 | 新轻合同/组合来源改变需要同步受管manifest、锁、投影与来源拒绝；不改JSON签名伪验身份 |

本轮未运行cargo metadata/tree/build，未移动真实auto-web检出制造无依赖环境。通过rg发现的直接三边不是完整依赖证明；T-01必须连传递/可选/dev/target/build、工具/fixture消费者一起冻结。

## 5. 详细设计

### 5.1 两层验收与方案门

L1：隔离消费Core与语言基础的公开API、VM及必要HTTP host port；无Web/UI/Man checkout可见，不含Web实现/renderer/管理实现依赖。L2：保当前合法语言workspace完整成员/普通语言职责，根metadata与build及必要测试能在无Web兄弟检出中完成；若依赖固定外部源码，必须在固定声明/锁和明确获得流程内，不允许查主检出fallback。两层必须同时通过才关闭整个独立构建项。

T-01产出逐边表：请求者/包身份/normal-build-dev-optional-target/真实符号与测试/owner/支持版本/来源和替代候选。T-02用小型自有完整消费探针比较最多两个有据方案，冻结选中类型owner、公共路径/serde/hash、source获得/lock规则、facade与generator接收、保留/迁移的测试ID、支持矩阵和回退。只要方案涉及兼容承诺变化，必须有具体用户决定，不能把一次可行性probe称产品交付。

### 5.2 实施与类型/政策保护

T-03只实现已确认方案，改动路径以§4实物为依据：Core清单/http_service_config、Web合同/宿主适配、facade/bin/auto-api-gen、source_owner和SDK投影/fixture。新增基础承载包/文件仅T-02冻结后建，当前不虚构它已存在。不复制整个Web实现或两套policy，不放宽严格JSON未知字段/坏值/非法组合诊断，不降低CORS/代理/Host/限流语义；配置hash和协议/类型身份变化若必要必须版本化并获确认。

Web运行仍显式adapter，缺provider时保持具名失败、普通无Web调用保持可用，!Send VM线程/owned消息及取消/关闭约束不变。Core/API公开Rust路径和签名、TypeId/trait实现跨crate身份按T-02承诺验证。旧版本固定消费者不被悄悄指向新类型；不得移除links来允许同名双source，故意混源控制须拒绝。

### 5.3 可获得性、发行与隔离实验

隔离环境从冻结Git来源导出/专用检出生成，不包含或访问Web/UI/Man兄弟目录；明确清除AUTO_WEB_SOURCE_ROOT/AUTO_LANG_ROOT/相关delivery override与隐式patch路径，记录实际resolved manifest/source树及编译argv。不能重命名/移动用户主检出。保留已声明且可获得的registry/toolchain输入；warm --offline仅证明预取后离线，不冒首次fresh已能下载。

先L1真实API/VM/host测试，再L2根cargo metadata --locked、cargo build --workspace --locked（若存在平台不适用成员按原平台规则具名，不能为门临时删成员）与受影响固定组。已批准域owner迁出的原成员/测试必须有正式去向与原子义务映射，不以缩workspace面/关闭测试凑目标；若不能保原支持需用户修订后才执行。

组合面用固定新Lang/Web/UI/Man输入重放实际受影响Web服务、Core/full、SDK独立消费和生成发行fresh/locked/A→B，含坏来源/错版本/缺包/断安装→恢复再消费；不重跑890所有应用或全AG，代表场景不替代完整受影响原子义务。旧方案回退按跨仓提交组合逆序，来源/配置/旧有效输出保持，不reset其他WIP。

### 5.4 与其他路线、材料规则

唯一lang-906组D:/autostack/.wt/lang-906/{auto-lang,auto-web,auto-ui,auto-man}，语言分支plan-906-dev，实际修改的其他仓各一树/分支plan-906-<repo>-dev；只读依赖detached，必要Down同组。实施/内部核查/独立review同一权威树顺序，禁止junction/symlink。用户只批准规划前，不建树。

UI014/AG020可能共用source_owner/工具/SDK接口，不是无关全局等待；T-01核当前正式diff/来源和写入窗口，消费合同先行、最终合入短串行。905在本计划依赖输入改变后记录新样本版本，不能混旧新作空间A/B；本计划不负责其缓存算法。

开工登记D:/autostack/plan-artifacts/plan-906/（跨仓按身份子目录），探针/SDK/消费副本/发行包/输出/外置target只放该scope；正常构建在专用树target。merge必要命令/退出码/数字/节选折本计划，再按实际所有命令路径对账删整个scope和工作树；只具名未解决问题持有并登记。attachments禁止入库，legacy specs.json只读，不清其他目录或全局cache。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/standalone-language-build.md（新） | Web兄弟path前提→L1/L2固定来源独立消费、支持矩阵、禁止隐式fallback与单源 | 闭900-N7及完整边界 | AC-01/03/04 |
| SD-02 | modify | docs/specs/language-sdk/{project,http-host-port}.md | 按T-02已确认方案更新实际轻合同/端口owner与公开路径，保运行语义 | 不将候选写成现状 | AC-02/03 |
| SD-03 | modify | docs/specs/repository-boundaries/web-four-repository-delivery.md、auto-api-gen/project.md | 真实新来源/适配/生成兼容/发行闭包，原能力与义务保持 | 组合一致 | AC-02/04/07 |
| SD-04 | modify | Web/UI/Man现有owner Spec对应章节（T-01按真实受影响页冻结） | 必要跨仓接收/版本/测试与恢复指针；仅各仓canonical | 同一份合同，不跨仓复制新实现 | AC-04/07 |

### 5.5 T-02 方案冻结（C1：候选1+3组合；探针 P-A/P-B/P-C 全绿）

**A 合同包单源迁移（L1）**：`auto-web-contract` 包逐字迁 lang `crates/auto-web-contract`（src 1069 行+build.rs 原样；Cargo.toml 仅 workspace 继承改显式 version=0.1.0/edition=2021/license=MIT；links=auto_web_contract 与包名不变）。Web workspace members 去该项；auto-web 依赖 path 翻转 `../../../auto-lang/crates/auto-web-contract`（=0.1.0 保留）；Core path → `../auto-web-contract`。类型身份单一包迁移不变（foundation-consumer Arc::ptr_eq 钉保持）；RateLimiter/RateDecision/ServiceRuntime 留 Web（有状态运行时）；Core build.rs DEP_AUTO_WEB_CONTRACT_ROOT 指纹机制 Cargo-选择无关。**P-A 证**：逐字复制+显式字段独立消费 2/2（严格解析/hash 重放/host 策略/槽 set-take/links 守卫随行）。

**B facade Web 边清零（公开兼容变化→需用户确认）**：删 auto-web 依赖；api 删 `auto_web::api::{AxumGenerator,TypeScriptGenerator}` 再导出，facade 自设 Target 枚举改再导出 core Tauri-only Target（`auto_lang::api::Target` 路径存续、Axum/TypeScript 变体移除；四仓零外部消费者，Man/UI 直连 auto_web::api）；`http_service_config` 再导出源改 auto_lang_core（合同+槽不变；RateLimiter/ServiceRuntime/RateDecision 不再经 facade 路径）；删 http_file_service/http_service_observability/http_upload_service 与 vm::ffi::{http_server,http_transport} 再导出（零外部消费者，Web 侧原文不动）；bin/auto-case 删 install 调用（无 .at 用例用 serve，缺席按 host-port 规范具名拒绝）。

**C api-gen Web 边清零**：删 auto-web 依赖、4 个 web api_gen 再导出（四仓零消费者）与 test-http-e2e feature 转发；generate_api 非 tauri 腿改具名错误指路 auto_web::generate_api 直连（UI vue 已直连；tauri 腿不变）。

**D http e2e 家族迁移（义务映射→需确认）**：8 文件 ~235 fn 迁 `test/fixtures/modularity/lang-web-e2e/`（独立探针工程、非 workspace 成员；deps=auto-lang+auto-web 组内相对路径，web-port-probe 先例）；`crate::`→`auto_lang::`，install 改直连 auto_web；plan736 contract 部分留 facade、RateLimiter/ServiceRuntime 部分随迁；`http` 固定组表达式收紧排除已迁家族，迁移目标由四仓 fixture 门驱动（T-04 接线）；fn 名（原 ID）全保留。

**E 接线**：source_owner web 类别拆分（contract→lang 树源、auto-web 留 web 类）；rust_ui DELIVERY_VENDOR/PROJECTION contract 改 lang 树投影；web-install.py contract owner 翻转；5 fixture 探针 path 更新；modularity-audit 基线边更新（core→contract 内部化，facade/api-gen→web 边消失）。

**回退**：跨仓提交组合逆序（lang manifest/源码/测试迁移 + web workspace/依赖行），无远端动作/不可逆迁移，lock 再生即回旧形态。

**C2（有据否决）**：合同留 Web+固定渠道——registry 未发布、auto-web 无 git remote（新建=远端动作）、vendor 复制=可编辑第二套实现+links 双源冲突；且不解决 E2/E3（auto-web 全实现仍需 Web 兄弟）。P-B 负控（未打补丁清单树：`failed to read ...\auto-web\crates\auto-web-contract\Cargo.toml`，exit 101）冻结债务实证；正控（C1 补丁）全 workspace `cargo metadata --offline` exit 0 无 Web 兄弟。P-C：facade 外独立 crate 显式装配 Web host 真 TCP e2e 1/1 绿（迁移通道证）。

## 6. 测试设计

当前全部未执行。work实施并跑反例，review独立上下文在同组冻结树核；不以实现者checked box/自查冒独立审查，本规划不创建agent/聊天。

| ID / task | AC | 输入/实际入口 | 可区分错误实现的oracle |
|---|---|---|---|
| V-01 / T-01/02 | AC-01 | 冻结normal/build/dev/optional/target Cargo metadata+源码路径；L1/L2分别隔离 | 没有ambient Web根依赖；可选path仍解析反例被发现；facade/api-gen漏边必须令L2失败，不能Core成功就绿 |
| V-02 / T-03/05 | AC-02 | 原严格配置正常/坏字段/零限额/非法组合、CORS/Host/可信代理、effective hash；旧新公开API/trait跨crate消费 | 值/字节/hash/错误阶段/策略保持，单一类型来源；混同名不同source/版本必须拒绝，不删links逃避 |
| V-03 / T-03/05 | AC-02/03 | Core VM及HTTP port，显式adapter、缺席/重复/实例隔离、真实服务JSON/file/upload/取消/关闭受影响原子 | 普通语言不需要Web安装，实际服务由Web提供；原线程/预算/owned资源终态与诊断保持；不靠直接API伪UI事件 |
| V-04 / T-04/05 | AC-03 | 无兄弟目录/env/ambient patch的新Core消费与原完整workspace metadata/build --locked、必要固定组；缺新依赖源负控 | 实际resolved源/编译路径/运行与固定声明一致，坏/缺依赖明确失败，不能借原主检出/旧exe或缩成员面过门 |
| V-05 / T-04/05 | AC-04 | 新跨仓公开Web/UI/Man/生成消费者，fresh/locked/A→B，错source/版本/资源、安装中断、恢复再用 | 新旧支持矩阵和原子义务保持；错源拒绝、旧有效产物保全、恢复可用；不得重新生成golden或降floor |

候选语言职责组：auto-lang-core、auto-lang、auto-api-gen、auto-generation-core、auto-test-case、http、integration及真正受影响SDK/CLI/fixture；T-01以静态配置和实物调用冻结完整并集，work定向迭代，review后冻结并集或同输入有效证据复用。不自动跑全ACE/AG；若共用HTTP host/类型变更触及ACE adapter，则明确补必要ace-adapters/宿主消费者。Web/UI/Man各用原owner受影响完整测试族与fresh消费，不用语言包绿替代。原ignore/platform/manual/资源预算/样本不变，首次红/修复/环境重放分别记；无未解释丢失或新增红。

## 7. 验收标准

- [x] AC-01（owner_stage: work）：完整依赖/符号/owner/测试账及T-02方案冻结；L1/L2定义清楚，有必要的公开兼容/域归属/渠道决定，不以单path补丁代全仓。
- [x] AC-02（work）：严格配置/政策/公开类型与host port单源，原语义与显式装配保持；V-02/03有效正常/反例，旧新版本支持或已批准迁移有证。
- [x] AC-03（work）：隔离L1+L2均实际解析/构建/必要测试通过，无ambient Web/UI/Man源码或旧产物依赖；全部当前合法成员/能力/原子义务保持或有批准接收，缺包/错源不伪成功。
- [x] AC-04（work）：跨仓新组合SDK/发行/生成实际消费fresh/locked/重定位/坏件恢复通过，来源/锁/类型不双份，回退可执行；900-N7只有真实两层门齐才关闭。
- [x] AC-05（owner_stage: review）：独立上下文逐AC/负控/遗漏/延期/健康/SD审查通过，无本包新增未处理warning/debug/格式问题，既存限制具名。
- [x] AC-06（owner_stage: merge）：冻结必要门/有效输入证据及各实际修改仓按依赖顺序落地，canonical SD及900-N7真实关闭说明/来源接受齐；不写legacy ledger。
- [x] AC-07（merge）：归档、本计划所有实际输出路径/统一scope/各树guard清理完成，只有具名未解决持有；不冒整个Cargo治理或所有产品已完成。

## 8. 执行步骤

| ID / owner_stage | 前置 | 动作/交付 | AC / 关闭证据 |
|---|---|---|---|
| T-01 / work | 用户明确批准本合同实施 | 建唯一lang-906组/scope，冻结完整依赖/符号/能力/测试/来源/当前共享面；核可用固定依赖渠道、明确资源窗口 | AC-01，V-01 |
| T-02 / work | T-01 | 最多两个有据候选的有界完整路径探针；在Plan与Design53冻结L1/L2方案、API/类型/政策/owner/来源/版本迁移/回退和精确门。需要变更兼容或新渠道先呈用户决定 | AC-01，V-01/02探针；未达go不进T-03，不把no-go当问题关闭 |
| T-03 / work | T-02 go及必要具体决定已确认 | Core/合同/端口与facade/api-gen实际调用按方案实施；Web接收，单owner/协议/原严格政策保护；不复制整实现 | AC-02，V-02/03 |
| T-04 / work | T-03 | source_owner/SDK投影/manifest/锁/必要UI/Man/Web消费者接线；构造隔离L1/L2及新组合fresh/locked/恢复入口 | AC-03/04，V-04/05 |
| T-05 / work | T-04 | 冻结完整受影响组/消费/反控，原ID/条件/计数对账与源树证明；回退/SD准备，无源码fallback | AC-01..04，V-01..05 |
| T-06 / review | T-05 | 同树独立上下文总审AC、类型/政策/来源/负控及SD/实际范围 | AC-05 |
| T-07 / merge | T-06 pass | 冻结门及必要消费、各实际仓串行合入、canonical知识和900-N7真实关闭；906版本回交905/014 | AC-06 |
| T-08 / merge | T-07 | 摘要折计划，逐命令实际输出路径对账删scope、guard清树/分支/组，归档 | AC-07 |

## 9. 复审记录

2026-11-01 / stage: new / PLAN-906 r1 / outcome: pass（规划自查，非实施/独立review） / 实物核发现三条Web边、Core整包再导出和合同workspace继承/links；已将“只修一条path”误判置V-01反例。T01..08覆盖AC01..07/V01..05/SD01..04；T-02是真实方案决策门，未选具体搬迁/发布渠道。next：用户批准后T-01/02，必要公开兼容/owner决定必须在实施前确认。当前未建树/构建/测试/源码改动/发布。

2026-11-01 / stage: work / PLAN-906 r1 / T-01 完成 / 用户 /goal 授权全链（goal grant 记录：work/review/有界修复/merge，本条即授权登记）。建组 D:/autostack/.wt/lang-906/{auto-lang(plan-906-dev@174652b5c),auto-web(@ec9c4ba),auto-ui(@d43425c),auto-man(@1eeac78),auto-down(@7c4d5cb)}，scope D:/autostack/plan-artifacts/plan-906/（.retention/policy.json 登记 in-flight）。闭包冻结（V-01 输入）：生产边 E1 core→contract（生产仅类型化槽，launch 类型擦除）、E2 facade→auto-web（9 处再导出，四仓零外部生产消费者，全走 auto_web 直连）、E3 api-gen→auto-web（4 再导出符号零消费者+非 tauri 委托）；反向边 web→lang 四 crate 既有；fixture 边 5 个独立探针；http e2e 家族 8 文件 ~235 fn 的 Web 触点唯一=http_host_test_support::install()；渠道现状：registry 未发布、auto-web 无 remote（无冷 Git 源）、PLAN-901 投影+source_owner web 类为既有受管渠道；Core build.rs 经 DEP_AUTO_WEB_CONTRACT_ROOT 指纹（Cargo-选择无关）。证据：plan-artifacts/plan-906/shared/t01-closure-freeze.md + lang/metadata-{nodeps,full}.json。当前 L1/L2 均不独立（三边 path 解析依赖组内/主检出 web）。next：T-02 候选探针与方案冻结。

2026-11-01 / stage: work / PLAN-906 r1 / T-02 完成（方案冻结，实施门待用户） / 探针三绿：P-A 合同包逐字重定位（显式字段+links/build.rs 随行）独立消费 2/2（严格解析/hash 重放/host 策略/槽 set-take 全同）；P-B 负控=未打补丁清单树 `cargo metadata` 红于 `auto-web\crates\auto-web-contract\Cargo.toml` 缺失（900-N7 实证），正控=C1 补丁（contract 入 lang 成员+core path 翻转+facade/api-gen 删 web 行）全 workspace 无 Web 兄弟 `--offline` metadata exit 0（仓内 vendor/autodown-test-parser 补齐后）；P-C=plan729 代表用例迁 facade 外独立 crate（auto-lang+auto-web 直连、显式 install host）真 TCP e2e 1/1 绿。C1 冻结于 §5.5 + Design53 §3；C2 有据否决。**阻塞：C1 含公开兼容变化/owner 迁移/测试义务通道变化（§10.1），按本计划 §2 门与 goal 授权边界须用户逐项确认后才进 T-03。** 证据：plan-artifacts/plan-906/shared/{t01-closure-freeze.md,probe-a,probe-b,probe-c}。next：等待用户对 §10.1 (a)(b)(c)(d) 的决定。

stage: goal | plan_id: PLAN-906 | plan_revision: r1 | round: 1 | from: work | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-906/auto-lang | commit: 174652b5c（基面，代码未动） | next: user（§10.1 四项决定后恢复 T-03）

2026-11-01 / stage: work / PLAN-906 r1 / T-03+T-04+T-05 完成 / 提交：lang@0cb025e7b（实施）+611368b32（SD）plan-906-dev；web@a2d8d0e（翻转）+847527e（SD）plan-906-web-dev；ui@42bfda7 plan-906-ui-dev（组 D:/autostack/.wt/lang-906/）。实施=§5.5 C1：(a) 合同包逐字迁 crates/auto-web-contract（links/build.rs 随行），双侧清单/成员/锁翻转；(b) facade 9 处 Web 再导出退役+Target 回归 core+auto-case 去隐式 host；(c) api-gen 4 再导出退役+非 tauri 具名错误+死 vue 分支移除；(d) e2e 家族 10 模块迁 lang-web-e2e 独立探针（plan705=19/707e2e=3/707wait=2/729=23/730=23/734=7/736=11/776=59/781+840=11/349cors=8，共 162=161 pass+1 skip 原条件，ID/断言保留；golden 路径与 plan705 源扫描路径按组内 5 级修正）；(e) source_owner lang 类别翻转+receipt owner、web-install/冻结侧位置派生、rust_ui web 类别仅 auto-web、四仓门 lang-web-e2e 固定腿+物理源断言翻转、5 个 fixture 探针中 2 个合同 path 更新。测试（本轮回执命令）：cargo t http（50 例 gate PASS）/auto-api-gen/auto-generation-core PASS（后者修一处自改测试的缺源负控选错 crate）；web -p auto-web --lib 175/175；ui web_consumers 1/1；fixture nextest（串行配置同源）161/161。隔离实验（V-04）：L1 新消费者（指隔离导出树）2/2 绿；L2 全 workspace --locked 构建 exit=0（无 Web 兄弟/清除 env，1m39s）；缺源负控=抽走合同包 metadata 红于具名路径、恢复后绿；负控 P-B（T-02）红实证 900-N7。既有红归因：modularity-audit strict=fail 2 违例（auto-cache→auto-tool-execution 新增禁向边 + stale_baseline）在 master 主检出自跑同判——非本计划新增，登记为既有治理债（基线重采样属维护任务）。merge-owned pending gate：四仓 fixture 全门（external legs+发行 fresh/locked/A→B）与受影响固定组冻结并集在冻结提交执行。证据：plan-artifacts/plan-906/{shared/t01-closure-freeze.md,probe-a,probe-b,probe-c,iso-lang,iso-l1-consumer},lang/{e2e-nextest-run3.log,iso-l2-build.log,iso-metadata.log}。next：T-06 独立复审。

2026-11-01 / stage: work / PLAN-906 r1 / T-02 实施门解除 / 用户确认 §10.1 (a)(b)(c)(d) 全部批准（目标续跑消息原文确认四项；授权补记：C1 公开兼容变化与 owner 迁移经用户逐项确认，纳入 goal 授权范围）。next：T-03 按冻结方案实施。

## 10. 待澄清事项

1. **【已确认 2026-11-01】C1 设计合同用户决定（§5.5）**：(a) 合同包 owner 迁移 Web→Lang——**批准**；(b) facade 公开兼容变化（Target Axum/TypeScript 变体移除、Web 再导出面删除、http_service_config 改指 core）——**批准**；(c) api-gen 非 tauri 委托改具名错误（调用方 auto_web 直连）——**批准**；(d) http e2e 家族（8 文件 ~235 fn）迁组内独立探针工程 test/fixtures/modularity/lang-web-e2e/——**批准**。T-03 实施门解除。
2. ~~T-02选择轻合同owner与获取方案~~（已冻结 C1/C2 见 §5.5；渠道证据：registry 未发布、无 git remote）。
3. 独立构建的冷来源、锁、工具链和当前支持平台由T-01冻结；offline预热验证与cold获取区别记录，不承诺无任何registry缓存/网络即可冷编。（T-01 补充：vendor/autodown-test-parser 为仓内固定源；registry 依赖按锁离线解析已证（P-B --offline 绿）。）
4. 905研究与014/020等source-owner/工具接线交集按实际diff协调，不加无关全线完成前置；本计划不自行向其他会话发消息或清其材料。

2026-11-01 / stage: work / PLAN-906 r1 / needs_fix 修复（复审周期 1） / F1(P2)：ui 树 Cargo.lock 未随 rust_ui 提交（组内清单图已变：auto-api-gen 去 auto-web、a2r-std 增 getrandom rider）→ 已提交 816b614，三树全 clean。F2(P3)：test-groups 日常默认排除式中已迁家族死模式（plan707_stream_e2e/729::http_e2e|lifecycle|r1_fixes/730::http_e2e/734::http_e2e/776::http_e2e_r*/t0*/781::budget_e2e）清除，toml 105 组解析+http 组选择复核绿。修复提交：lang@0ba038f32、ui@816b614；无代码语义变化（锁同步+死模式清理）。next：受影响发现重审。

2026-11-01 / stage: work / PLAN-906 r1 / needs_fix 修复（复审周期 2） / 勘误：周期 1 的 F2 修复为坏替换（16 个残留 tests:: 前缀连缀 + plan781_budget_http_tests::budget_e2e 漏删 + TOML 终结引号错位），中间提交 0ba038f32/4c56fa431/dbc78d38f 为坏态，03ca6cb7d 从冻结 alternative 清单整行重写修复：13 alternatives（test_cases::ac_av 全族/ac_mn::collections::controls/ace_certify::tests/tests::a2r_tests/a2ts_tests/book_listing_tests/plan724::e2e/plan727::e2e），tomllib 105 组解析 OK、cargo t --list 正常、http --select-only 选择 OK、树 clean。周期 1 回执中"死模式清除"表述按此勘误。next：F2 三验。

stage: review | review_scope: final | reviewer: 独立子代理上下文（general-purpose agent_0fff0a9d，同冻结树独立会话线索；局限：未重跑 web/ui 全套与四仓全门——收据日志+两项许可抽 Replay 复核） | plan_id: PLAN-906 | plan_revision: r1 | outcome: pass（2 修复周期后） | worktree: D:/autostack/.wt/lang-906/{auto-lang,auto-web,auto-ui} | branch: plan-906-dev@03ca6cb7d（base 174652b5c）/ plan-906-web-dev@847527e（base ec9c4ba）/ plan-906-ui-dev@816b614（base d43425c） | dependency_revisions: down@7c4d5cb detached | spec_inputs: SD-01..04 已核与代码一致（合同家=lang、RateLimiter 留 web、facade/api-gen 零 web、具名拒绝、owner 翻转）；lib.rs/build.rs sha256 与 Web 删除版逐字节一致 | acceptance_results: AC-01..05 pass（10 区独立核验：树/逐字迁移/残留引用 0/零 Web 边/10 模块迁移完整+编译/隔离抽 Replay L1 2/2+L2 metadata exit0+build exit0+负控红/组表达式诚实+四仓腿/SD 一致/健康 0 新警告 0 调试打印/无延后伪装；e2e 161+1skip 对账） | findings: F1(P2 ui 锁)周期1闭；F2(P3 死模式)周期1坏修复→周期2整行重写闭（03ca6cb7d，105 组解析/选择 OK；坏中间提交 0ba038f32/4c56fa431/dbc78d38f 留史） | pending_closeout_gates: 四仓 fixture 全门（external legs+发行 fresh/locked/A→B）与受影响固定组冻结并集=merge-owned；modularity-audit strict 既有红（master 同判，auto-cache→auto-tool-execution+stale_baseline）为治理债非本计划 | next: merge (T-07/08)

2026-11-01 / stage: merge / PLAN-906 r1 / prepared + rebase 等价证明 / master 漂移=Plan905 docs-only（174652b5c→ed4286792，3 文件全 docs/，代码零变化）。lang rebase 6/6 range-diff 全等（0cb025e7b→206e26156、611368b32→86609540c、0ba038f32→bfc1391b1、4c56fa431→c66024d08、dbc78d38f→707f53618、03ca6cb7d→71f922720=交付尖）；web master 未动（ec9c4ba=基）；ui rebase（42bfda7+816b614 → 新尖）docs-only 位移。既有证据（http/e2e161/隔离 L1L2/web175/ui consumer）绑定的代码内容在 rebase 后逐字节不变，按 PLAN-896 复用规则继续有效；四仓全门在交付尖执行。

2026-11-01 / stage: merge / PLAN-906 r1 / 冻结门修复与归因（四仓全门 web-four-repository.py，交付尖累进）/ 跑1：integration 发现期红 "ACE host tests leaked into a legacy group"——3 例 `auto-lang-core::test_cases::ac_a2r_hir::path_domain_tests::*`（**Plan 904** RV-4 新增，legacy 排除式未随更）；`ace-adapters` 组正向选择 `test_cases::ac*` 故排除无覆盖损失→ integration/auto-lang-core 两 legacy 组排除式补 `test_cases::ac_a2r_hir`（9373a084f）。跑2：integration 3193/3194，`plan738 t02_inventory full_scan` 基线 38→39 漂移——第 39 = `stdlib/auto/rand.rs.at`（**Plan 907** 新增 .rs.at 镜像未更基线）；逐名核对后按测试自述指示更新基线（36da18088）。跑3：语言 6 组全 PASS（integration 3194/generation-core 18/api-gen 3/media-core 54/a2r-std 108/test-case 94）+ lang-web-e2e 腿 161/161；我方腿校验 bug（读 stdout 而 Summary 在 stderr + failed 段缺省不匹配）→ 修 four_repository 腿读 stderr/可选 failed（fba55b497）。跑4：media-core `shutdown_interleaved_with_concurrent_submit_and_open` 123s TIMEOUT（跑3 同组 54/54 绿；隔离重放 0.46s 绿）=环境归因 flake。跑5：语言+e2e 腿全绿；web 腿 "native test cwd escapes selected checkout"——web owner 合同（890 冻结）仍列 `-p auto-web-contract`，包已迁 lang 树且零测试→从 web 腿移除（0adae7b3b）。跑6：integration `plan707_spike_cancel_gap_active_job_future_not_stopped` 红（跑3/5 绿；隔离首次红、重试绿）=既有时序 flaky，非本计划面。跑7 进行中。既有红/flake 归因均有命令+日志（plan-artifacts/plan-906/lang/fourrepo-gate.log 与各跑 language/0/run.stderr）。

PLAN-906:r1 consolidation receipt
- prepared：reviewed 基线 03ca6cb7d（独立复审 final pass，2 周期）；rebase 两次（master ed4286792→Plan908 后 ac4472f8d），11/11 range-diff 全等；交付尖 lang **4571f55e9**。merge 期门修复 6 项（9196c8676/894f32d1f/51bb984eb/8007a2450/4571f55e9 + man 7f30eb8）经复审代理目标重审 pass。
- gates（冻结门，plan-artifacts/plan-906/shared/fourrepo 跑8 + fourrepo-legs r2）：语言 6 固定组 PASS（integration 3194/generation-core 18/api-gen 3/media-core 54/a2r-std 108/test-case 94，gate_passed=true）；lang-web-e2e 腿 161/161（1 skip 原条件）；web 腿 exit 0；ui-full 3583/3586（3 红 `ui::iced::layout_tests::{desktop_surface_z_slot_window_covers_icons,p625_uigallery_sidebar_pills_visible,popover_modal_centers_within_clipped_host}` 于 UI master a3df9e8 同命令确定性复现，复审者独立复现=UI owner 既有红 D-906-2）；man-t 283/man-tc 254/man-ti 43/man-tcore 24 全 exit 0；ui-tu 866/0 失败、ui-ti 28/28。http 组重钉 PASS。环境 flake 两例（media-core shutdown_interleaved 123s 超时/plan707_spike_cancel_gap）隔离重放绿记环境归因。发行门 web-repository-distribution：源暂存于 vendor/autodown-test-parser 布局红——Plan 901 谱系既有（master web metadata --locked 本身红），登记 D-906-1（Web owner），906 发行面改动代码级验证、端到端待门修复。modularity-audit strict 既有红 D-906-3。
- landed：lang master 4571f55e9（ff-only，==plan-906-dev）；web master 847527e（ff-only）；ui master 98cbcd2（ff-only；主检出脏 Cargo.lock 为 cargo 自动再生 getrandom 行、我落地锁的子集，还原后 ff，非用户内容）；man master 7f30eb8（ff-only）。各仓 wt-guard clean 后合入。
- canonical_verified（master 读回 sha256 前12）：standalone-language-build.md e1b2678fb2a5（新）；language-sdk/http-host-port.md dfeb2562d598；language-sdk/project.md e0e62deba10a；repository-boundaries/web-four-repository-delivery.md a70dd398e139；generated-ui-distribution.md 240dd50a68c1；auto-web/project.md d1dbe1da9fe8；auto-api-gen/project.md 37c1af4a6aa0；web 仓 docs/specs/project.md 含 PLAN-906 家注。合同 crates/auto-web-contract/src/lib.rs 96ea1f0ebedd（与 Web 删除版逐字节同）、links 守卫在。Design53 §3 T-02 冻结文本经共享主树随 Plan905 提交 65f3b8c92 落地（内容正确，出处注记）。
- archived：docs/plans/archive/906-standalone-language-web-boundary.md，completion_kind: delivered，8/8 任务，AC-01..07 全勾；900-N7 真实关闭（L1+L2 两层门齐：隔离消费 2/2、全 workspace --locked 构建 exit 0、缺源具名红）。906 版本回交：905 已归档（其 Web path 闭包观测在新身份下需沿 4571f55e9 更新——登记为 905 后继合同输入，不改归档）；UI 014 的 source_owner/投影接线已由本计划 rust_ui 改动承接。
- 批量回归到期：895 后多笔合并且 >48h → due；按 standing 2026-10-30 用户豁免登记 user-pending（同 904/905/908 先例），不自跑。
- cleaned：见下一条回执。

- cleaned：五树（lang/web/ui/man/down）各 wt-guard clean + 已落地核验后 `git worktree remove`；分支 plan-906-dev/plan-906-web-dev/plan-906-ui-dev/plan-906-man-dev 删除；组目录 D:/autostack/.wt/lang-906 rmdir；plan-artifacts/plan-906 整删（14G/39769 文件，含 iso-lang 导出树、探针、门收据、fixture 副本）；登记表 plan-artifacts/.retention/policy.json 刷新（plans[auto-lang/PLAN-906] archived/cleaned，deletions 记 plan-906，holds=[]）；系统临时 /tmp/p906-* 清除；误建的 auto-lang/.retention/ 已删除。landing≠deployment 观察：语言仓无常驻生产进程；主检出 target/debug 既有二进制（auto-case.exe 等）早于本次落地，下次构建自然刷新——不另做发布动作。stale 观察登记：无需重启的生产面。
- final counts：8/8 任务（T-01..T-08），AC-01..07 全勾，completion_kind: delivered。

stage: goal | plan_id: PLAN-906 | plan_revision: r1 | round: 2 | from: merge | to: done | status: archived | outcome: pass | worktree: removed (D:/autostack/.wt/lang-906 gone) | commit: lang 4571f55e9 / web 847527e / ui 98cbcd2 / man 7f30eb8；归档簿记 c26e9e8de | next: done
