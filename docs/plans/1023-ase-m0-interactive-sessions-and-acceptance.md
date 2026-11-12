---
plan_id: PLAN-1023
status: drafting
feature_name: ASE M0交互会话原型与联合验收
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 0
total_steps: 10
depends_on: [PLAN-1022, PLAN-968, PLAN-969, PLAN-970, PLAN-980]
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/m0-interactive-sessions.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1023] ASE M0交互会话原型与联合验收

## 0. 变更摘要

r1，drafting，0/10，仅规划；消费1022正式frame及980/969/970/所用968能力，交付M0-SESSION/M0-CLOSE并联合核M0-BASE/LOAD/TEXT。M0定义为真实ACE交互会话闭环，不是Resolver/单text演示。路线：[ASE M0](../design/strategy/ase-m0-delivery-roadmap.md)。

## 1. 目标

单原生宿主支持两个存活应用会话；Counter A点击3次、B点击7次，切回各保留原值。同app双会话模型独立，关闭重开为0，旧事件不写新会话；取消/关闭后受管资源收敛。汇总各上游交付来源与证据并作M0实际裁决。

不含Clock/后台任务、checkpoint、Todo/服务/持久化、2D/3D、完整中文IME/复杂弹层、网络沙箱、A2R/Native发行或全Gallery。限制为研究profile，后续目标仍保留。

## 2. 架构方案

真实Auto源model/msg/on与text/button→980受检UI行为→969/970公共保留会话/执行实例→1022公共frame和输入适配→ACE更新模型→呈现结果。ASE只管理tab/session选择、代次和owner；不在Rust缓存/代算counter。共享制品与独立状态分离；不以重复编译+手工恢复数字冒保留会话。

## 3. 技术栈

复用1021/1022实验consumer与UI薄适配，消费980所交基本状态/按钮事件、969/970会话隔离/保留/关闭、968所用调度/取消接口；不要求970全部REPL/daemon/MCP。新增拟`auto-ui/crates/auto-ui-runtime/tests/ase_m0_sessions.rs`与ASE普通Counter源测试；原生输入/有效resize由现有UI宿主负责。

## 4. 需求分析与背景调查

用户2026-11-12明确授权制定ASE M0 strategy、第一批Plan及与ACE/AGE M2的实施次序，并确定Auto Scape Engine（ASE）为框架总名、AutoScape为浏览器产品。本轮授权为规划/文档，不含代码、fixture、worktree、构建、GUI/探针运行、提交/合入/标签或自动续跑；未指定模型、工期或token预算。既有其他会话在途/暂停决定保留。

取材[Specs overview](../specs/overview.md)、[语言SDK架构](../specs/language-sdk/architecture.md)、[UI仓界](../specs/auto-ui/project.md)和[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)。overview与部分UI旧测试别名为历史快照，现行AGENTS、固定组及UI本仓配置优先。当前ASE没有canonical现状规范，本包拟议增量仅在实际review/merge后沉淀，不将设计写成已支持。

读取身份：auto-lang HEAD `7b2f2e95967cb3434429093988fae429c05e865f`、auto-ui HEAD `ad9c05da65c9f48ec8a5ab0ef8815cff1ded0fb4`、auto-os HEAD `868c80d3e697ad6eaa9cce27596430a9abe08d5a`；工作区有其它会话文档WIP，内容可与HEAD不同，开工重核实际闭包与祖先。960模块Spec SHA256前16=`be25b0c69f2f8cc0`、公开packages AV实现=`fe9a2a1a36fdc7bf`；970草案=`70c5b6c98c680bac`、980草案=`fcc41243921c0fe7`。这些是只读来源标记，不是本包运行证据。

[组件/Store实例规范](../../../auto-ui/docs/specs/runtime/component-and-store-instances.md)要求实例所有权/跨App隔离/晚回调不重定向，为目标语义输入；不直接使用其旧VmRef执行实现。970是会话迁移草案，980 HELLO明确没有guest事件/状态，所以Hello交付不能关闭本包接口门。1022正式frame来源同样需核实际支持，而非从原路线推断。

## 5. 详细设计

### 最小正式进入门

frontmatter是相关Plan并集；T-01逐条核最小切片，不以整包完成替代实际接口。需1022正式来源、980提供model/msg/on＋text/button事件与更新、969/970提供独立保留会话/模型归属/关闭、所用968提供事件调度及实际采用的取消语义；分别记录版本/ABI/SHA/测试来源。无对应早期任务ID时由原owner明确交接，不虚构READY或要求全部M2标签。不得私建持久执行器绕过970。

本profile最多两个活Session，第三个请求具名`session.limit`并保持现有两个不变；不自动淘汰、不承诺休眠。顺序测试A/B以及同app双上下文，每轮最多两个；关闭后释放名额。一次关闭必须取消当前输入捕获、撤呈现、释放会话租约与owned结果；持久数据不在本包。

A=3/B=7通过真实按钮事件达到；切换保留同一会话模型，不合并源命名空间。关闭A后重新打开种子0；同app A1/A2分别3/7。事件身份包含session及generation，旧队列事件在关闭/重开后明确拒绝、不能写入新0模型；测试从真实adapter队列注入迟到事件，不用自造布尔标志代表生产拒绝。

T-05记录20轮开/切/关后受管Session/任务/图形目标真实创建/释放计数与owner走势；共享字体/宿主缓存单列允许有界驻留，不要求RSS/物理显存归零。原生事件和宿主计量不能侵入ACE私有字段；公共观察缺失交原owner限定补接口。无界guest/强进程故障隔离不属于该最小profile，不借取消按钮声称已具备。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ase/m0-interactive-sessions.md | 两页文字→双会话状态/代次/上限/关闭与M0已验格 | M0完成边界真实 | AC-01..06 |
| SD-02 | modify | auto-ui: docs/specs/runtime/ase-m0-frame-consumption.md（1022创建） | text消费→已验证基本按钮事件/resize/owner与陈旧拒绝边界 | UI适配单源 | AC-02..05 |

### 首次入口与统一树

唯一组`D:/autostack/.wt/lang-1023/{auto-lang,auto-ui}`，分支`codex/plan-1023-dev`，每仓唯一树，显式来源锁与依赖解析，不建链接。材料`D:/autostack/plan-artifacts/plan-1023/<repo>/`开工即登记；原生输入/GPU同机单实例。不修改OS/Gallery产品源码与默认入口。

## 6. 测试设计

测试与oracle以下均为拟议、未执行；新增测试/CLI入口在对应实施任务创建，不能在创建前报告PASS。实现与内部检查由执行者承担，最终独立review采用独立上下文、同一权威树；本轮未指定模型/人员，不创建agent/chat。

语言仓只按真实触面选择完整固定组及必要消费者，参考现存`ace-av`、`ace-source`、`ace-artifacts`，不为单一外部消费者机械跑`cargo t ace`全并集或`cargo t all`。若改ACE的源码/共享语义须补相应完整组与下游理由；纯ASE消费者不修改ACE时以新增ASE独立目标为主。UI只跑新增/受影响命名目标与公共接缝必要集成，按本仓入口，不沿用语言仓退役别名；原生/GPU/焦点单实例，自动注入与真实用户输入分账。工作迭代按反例/完整职责面，最终review后冻结提交验证受影响并集，身份未变证据可复用。

| Case ID | AC / invariant | 输入/真实入口 | 独立oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 接口准入 | T-01核状态/事件/会话/关闭/所用调度 | 只有text或草案时拒绝启动依赖实现；不虚构970整包可用 |
| V-02 | AC-02 状态保留 | 原生按钮A×3、B×7→切回 | 显示3/7且ACE模型对应；换宿主顺序与合法应用名仍成立，无Rust代算 |
| V-03 | AC-03 同app隔离 | A1×3/A2×7；关闭A1→重开 | 3/7独立，A1重开0而A2仍7；真实会话身份/模型owner核齐 |
| V-04 | AC-04 陈旧与限额 | 关闭后实际旧代次事件；第三会话；释放再开 | 旧事件拒绝/新0不变；第三具名拒绝且现有状态不变；释放名额后可打开 |
| V-05 | AC-05 呈现/释放 | 有效resize/焦点转移/20轮开关 | 命中当前frame、旧输入不串；owned资源计数收敛，共享缓存单列 |
| V-06 | AC-06 联合边界 | 1020–1022收据/当前身份＋本包结果 | M0-BASE/LOAD/TEXT/SESSION/CLOSE每门实际来源；缺证据pending，不从plan勾选反推完成 |

拟完整入口（T-06创建）：在UI唯一树`cargo test -p auto-ui-runtime --test ase_m0_sessions -- --test-threads=1`，并复跑受影响ase_m0_text/ASE headless完整目标；实际GUI原生命令复用1022并扩case，由T-01冻结。保留既有测试规模，20轮是新资源profile非缩减旧样本；原生人工与自动分列。共享SDK/ABI变化按触面补固定组，不能仅用新测试自证。

## 7. 验收标准

| ID | owner_stage | 验收与方法 |
|---|---|---|
| AC-01 | work | 1022及精确状态/事件/会话/取消接口正式来源齐，V-01 |
| AC-02 | work | 两Counter经ACE模型/真实按钮达到并保留3/7，V-02 |
| AC-03 | work | 同app双会话隔离、关开重置0且不动另一会话，V-03 |
| AC-04 | work | 旧事件真实拒绝、第三请求拒绝不破旧态、释放名额可重开，V-04 |
| AC-05 | work | 有效resize/焦点/20轮受管资源收敛、原生交互与自动分别验，V-05 |
| AC-06 | work | 联合门的来源/范围/未支持项对齐，无旧引擎fallback；V-06。不提前声称正式M0已落地 |
| AC-R | review | 独审模型/输入/会话owner/计量及M0全部必需门，不以三个前包勾选代证据 |
| AC-D | merge | 本包受审来源落地、Spec/归档/清理齐后才标M0正式完成；不签ACE/AGE里程碑 |

## 8. 执行步骤

- [ ] T-01｜work｜1022及最小正式UI/会话/调度接口｜核来源/唯一树/scope，冻结真实输入与观察口；AC-01，不足则停依赖实现。
- [ ] T-02｜work｜T-01｜新增普通Counter源与A/B描述，模型/事件经980/ACE，UI宿主不代算；AC-02。
- [ ] T-03｜work｜T-02｜接公共保留会话与tab绑定，同app双会话/关闭重开；AC-02、AC-03，V-02/03。
- [ ] T-04｜work｜T-03｜代次队列/关闭清理/两会话上限，禁止旧事件重定向；AC-04，V-04。
- [ ] T-05｜work｜T-04｜有效resize/焦点、真实输入和20轮资源计量；AC-05，V-05，不冒完整IME/DPI/沙箱。
- [ ] T-06｜work｜T-05｜新增并运行完整ase_m0_sessions与受影响text/headless目标；V-01..05，必要共享职责组选测理由登记。
- [ ] T-07｜work｜T-06｜核1020–1022正式来源与本包联合矩阵，给M0候选裁决/缺项；AC-06，V-06，不标里程碑正式完成。
- [ ] T-08｜work｜T-07｜告警/格式/无调试输出，§9折命令/结果/资源/hash与拟议Spec交接；AC-01..06。
- [ ] T-09｜review｜T-08｜独立重建M0 required与实际链路、反例/资源、遗漏扫描；AC-R。
- [ ] T-10｜merge｜独审pass｜冻结相关完整目标/必要组，SD-01/02正式落地、归档、全输出对账折摘要删scope/guard清树；AC-D，齐后才记录M0正式完成。

## 9. 复审记录

stage: new；PLAN-1023 r1；outcome: blocked（1022及状态/事件/保留会话接口未交）；next: 等所需切片正式交接后T-01。0/10，所有V-*、联合认证、review/merge未执行；本轮不签M0。

本轮静态预检：章节/任务数、AC覆盖、相关活动依赖无环及本地链接通过；归档前置作为已交终点，不重新审计其历史依赖。仅规划检查，未运行测试或产品。

## 10. 待澄清事项

原owner须明确最小状态/事件/会话交付切片，T-01核正式来源和语义；没有相应接口则保持pending，不假设Hello已含。若基本frame消费需要完整AGE新增能力，由T-01举证修订依赖，不预先等待AGE整个M2。
