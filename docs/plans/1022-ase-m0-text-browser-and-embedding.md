---
plan_id: PLAN-1022
status: drafting
feature_name: ASE M0两页文字浏览器与公共嵌入
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 0
total_steps: 9
depends_on: [PLAN-1021, PLAN-967, PLAN-969, PLAN-978, PLAN-979, PLAN-980]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/m0-text-browser.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1022] ASE M0两页文字浏览器与公共嵌入

## 0. 变更摘要

r1，drafting，0/9，仅规划；交付M0-TEXT/M0-T。1021及F0/H0/E0/B0/980 HELLO正式来源齐后，消费公共接口验证两份独立AutoUI普通源的真实文字输出与视口切换；不等待完整M2或AGE混合图形。路线：[ASE M0](../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

一个原生验证窗口，提供打开A/B、重新打开、关闭、错误/当前地址；A/B内容由独立普通AutoUI源经ACE AV产生。验证导航代次、改源、失败重试与真正frame嵌入。

不复制980 Hello的FFI/Schema执行器，不使用1021低层路径代替E0，不建设完整旧引擎浏览器；不包含业务按钮/保留状态、多窗口、3D、后台服务、全量Gallery或默认切换。

## 2. 架构方案

ASE目录/导航状态→969/E0及UI owner的受检独立应用入口→UI公共视图/关闭接口→同一个宿主frame。Rust/Iced仅可提供测试控制与现有投影/窗口服务；子页面中文/ASCII与普通函数返回值来自AV真实dispatch。Hello的单窗与可切换frame是不同事实，后者在T-01单独先证。

## 3. 技术栈

复用语言仓`experimental/ase-m0/`消费者与UI已有`crates/auto-ui-runtime/src/ui/iced/renderer/{host,projection}.rs`、`dynamic_build.rs`及980交付接口；新增UI薄适配/测试目标，具体路径T-01按正式API冻结。UI依赖ASE共享消费者时保持单向，无Core→UI/ASE反依赖。

## 4. 需求分析与背景调查

用户2026-11-12明确授权制定ASE M0 strategy、第一批Plan及与ACE/AGE M2的实施次序，并确定Auto Scape Engine（ASE）为框架总名、AutoScape为浏览器产品。本轮授权为规划/文档，不含代码、fixture、worktree、构建、GUI/探针运行、提交/合入/标签或自动续跑；未指定模型、工期或token预算。既有其他会话在途/暂停决定保留。

取材[Specs overview](../specs/overview.md)、[语言SDK架构](../specs/language-sdk/architecture.md)、[UI仓界](../specs/auto-ui/project.md)和[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)。overview与部分UI旧测试别名为历史快照，现行AGENTS、固定组及UI本仓配置优先。当前ASE没有canonical现状规范，本包拟议增量仅在实际review/merge后沉淀，不将设计写成已支持。

读取身份：auto-lang HEAD `7b2f2e95967cb3434429093988fae429c05e865f`、auto-ui HEAD `ad9c05da65c9f48ec8a5ab0ef8815cff1ded0fb4`、auto-os HEAD `868c80d3e697ad6eaa9cce27596430a9abe08d5a`；工作区有其它会话文档WIP，内容可与HEAD不同，开工重核实际闭包与祖先。960模块Spec SHA256前16=`be25b0c69f2f8cc0`、公开packages AV实现=`fe9a2a1a36fdc7bf`；970草案=`70c5b6c98c680bac`、980草案=`fcc41243921c0fe7`。这些是只读来源标记，不是本包运行证据。

已核980 §5 HELLO定义最小单widget/view/text与函数文本，使用F0/H0/E0/B0，不含guest事件/状态。已核现有`vm_bridge.rs`、`handler_codegen.rs`与`dynamic_build.rs`路径和旧执行耦合说明；不得创建假旧VmBridge过类型门。F0/H0/E0/B0/HELLO目前为规划，未有本包可消费的正式证据。

## 5. 详细设计

### 接口门与有界先证

frontmatter为相关Plan并集；本包不等待这些Plan全包结束。严格进入门：1021正式交付；967/F0-D、979/H0-D、969/E0-D、978/B0-D、980/HELLO-D各记录repo/SHA/接口/profile/验证范围。A2R首窗可并行，不是本包AV门。未齐只作合同准备，不开窗口实现/低层替代。

T-01最多比较两种公共Component/视图消费机制，在同一真实宿主frame上先显示A文字、关闭、显示B文字。必须复用UI投影/窗口与ACE公共执行；若只能合并源码、伪造旧桥或重写渲染器，具名needs_replan并停后续。修复宿主呈现接缝可落本包UI薄适配；FFI/语言/状态语义缺口归原owner，不重复派工。

两份页面包含中文/ASCII不同文本与普通函数返回文本，任意合法根/函数名及修改后仍工作。T-02创建真实AutoUI源，放拟新增`experimental/ase-m0/fixtures/text-{a,b}/`；UI可经显式路径消费，不复制一份另一权威。T-03新增拟`auto-ui/crates/auto-ui-runtime/tests/ase_m0_text.rs`及同owner原生验证入口；源码/配置随正常工程跟踪，暂存scope外不存实验产物。

导航世代属于ASE控制面，每次打开/重开/关闭推进；受理当前代次结果才挂载，旧完成被拒绝并释放owner。无guest异步时可在宿主控制面延迟一次真实装载结果验证陈旧拒绝，不以此冒通用异步执行。关闭后空frame，重开重执行；不承诺保留AppSession或缓存零编译。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ase/m0-text-browser.md | headless→正式E0/UI消费、导航/关闭与支持边界 | M0-T可复核 | AC-01..05 |
| SD-02 | add | auto-ui: docs/specs/runtime/ase-m0-frame-consumption.md | 无ASE公共消费约定→实际薄适配/资源与输入owner，仅记录已验范围 | 公共frame接缝归UI | AC-02/04/05 |

### 实施树与材料

唯一组`D:/autostack/.wt/lang-1022/{auto-lang,auto-ui}`，主分支`codex/plan-1022-dev`，每仓唯一树与显式锁/SDK身份；先读UI当地规约。正式源码不落master，Plan簿记留主检出。scope `D:/autostack/plan-artifacts/plan-1022/<repo>/`建立即登记owner/用途，截图/日志/构建副本不散落。

## 6. 测试设计

测试与oracle以下均为拟议、未执行；新增测试/CLI入口在对应实施任务创建，不能在创建前报告PASS。实现与内部检查由执行者承担，最终独立review采用独立上下文、同一权威树；本轮未指定模型/人员，不创建agent/chat。

语言仓只按真实触面选择完整固定组及必要消费者，参考现存`ace-av`、`ace-source`、`ace-artifacts`，不为单一外部消费者机械跑`cargo t ace`全并集或`cargo t all`。若改ACE的源码/共享语义须补相应完整组与下游理由；纯ASE消费者不修改ACE时以新增ASE独立目标为主。UI只跑新增/受影响命名目标与公共接缝必要集成，按本仓入口，不沿用语言仓退役别名；原生/GPU/焦点单实例，自动注入与真实用户输入分账。工作迭代按反例/完整职责面，最终review后冻结提交验证受影响并集，身份未变证据可复用。

| Case ID | AC / invariant | 输入/入口 | oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 正式来源 | T-01核6项交付及HELLO接口 | 草案/分支tip不算D；AV通过不要求A2R齐，不误等整包 |
| V-02 | AC-02 独立真实UI | 新原生验证入口打开A/B、函数文本/换名 | 实际AV dispatch文本字节、UI session接收与画面对应；Rust硬画/旧VM替代不能通过来源审计 |
| V-03 | AC-03 独立改源/恢复 | 改A不改B，非法A→修复，未知app | A变化/B不变；非法不半挂载，修复后真实重开；未知身份零执行 |
| V-04 | AC-04 导航代次 | A迟到→B、关窗后迟到、A→B→A | 旧结果拒绝不覆盖新frame且owner释放；受控宿主延迟不冒guest async |
| V-05 | AC-05 生命周期/呈现 | 有效resize/焦点、打开关闭20轮 | 当前内容按新frame呈现、关闭空frame；存活owner/目标不随轮次无界增长，所计资源实源，不假称驱动显存归零 |

拟创建命令：在UI唯一树`cargo test -p auto-ui-runtime --test ase_m0_text -- --test-threads=1`（T-06创建目标）；真实窗口入口参数与定向集成并集在T-01按980正式接口冻结，T-05记录可重放原生命令，不虚构现有`--engine ace`。零新增图形实验源码不跑完整AGE套件。真实原生交互缺环境则pending。

## 7. 验收标准

| ID | owner_stage | 验收与方法 |
|---|---|---|
| AC-01 | work | 精确接口正式来源齐，T-01公共frame先证通过，V-01 |
| AC-02 | work | 两独立普通AutoUI源/函数文本经真实AV至frame，无legacy/硬编码，V-02 |
| AC-03 | work | 单源修改与非法/修复/未知app处理，V-03 |
| AC-04 | work | 打开/重开/关闭代次及迟到拒绝不覆盖新页，V-04 |
| AC-05 | work | 有效resize/焦点、20轮关闭释放与真实窗口结果，V-05 |
| AC-R | review | 独审源→受检执行→UI owner→实际画面与清理；不拿Hello代嵌入 |
| AC-D | merge | 两仓正式来源/SD-01/02及冻结定向并集齐，归档与路径对账清理 |

## 8. 执行步骤

- [ ] T-01｜work｜1021+F0/H0/E0/B0/HELLO正式D｜核接口/跨仓树/scope，最多两机制frame先证并冻结公共消费路径/原生命令；AC-01。no-go时停后续。
- [ ] T-02｜work｜T-01｜在ASE实验工程加两独立普通AutoUI文字源/描述，任意根名与函数文本；AC-02/03，核源码闭包。
- [ ] T-03｜work｜T-02｜UI owner薄适配与单原生验证宿主，控制按钮只发导航动作，接E0/公共frame；AC-02，V-02。
- [ ] T-04｜work｜T-03｜导航代次/取消/错误/修复与关闭owner；AC-03、AC-04，V-03/04。
- [ ] T-05｜work｜T-04｜实际窗口有效resize/焦点/20轮生命周期，核文本与dispatch来源；AC-05，V-05。人工原生与自动注入分列。
- [ ] T-06｜work｜T-05｜创建并运行完整ase_m0_text目标与必要公共接缝集成；V-01..05；未跑格不PASS。
- [ ] T-07｜work｜T-06｜告警/格式/调试输出检查，折§9摘要、Spec delta与1023交接；AC-01..05。
- [ ] T-08｜review｜T-07｜独立核原源/AV/公共嵌入、陈旧释放及上下游边界；AC-R，不重复接管980。
- [ ] T-09｜merge｜独审pass｜冻结相关完整目标/必要组，两仓SD正式落地、归档；全输出目录对账折摘要清scope，guard清树；AC-D。

## 9. 复审记录

stage: new；PLAN-1022 r1；outcome: blocked（1021及F0/H0/E0/B0/HELLO正式来源未齐）；next: 按原owner交付接口后T-01公共frame先证。0/9，所有运行/测试/review/merge未执行。

本轮静态预检：章节/任务数、AC覆盖、相关活动依赖无环及本地链接通过；归档前置作为已交终点，不重新审计其历史依赖。仅规划检查，未运行测试或产品。

## 10. 待澄清事项

T-01负责公共嵌入是否可行及adapter位置，最多两机制并以具名结果结束；不为缺接口另写旧引擎版。此Plan不自动要求980新增复杂语法；若必须改变原owner语义，先形成对应修订再依赖实施。
