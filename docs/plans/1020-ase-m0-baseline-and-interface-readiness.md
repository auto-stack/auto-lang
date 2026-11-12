---
plan_id: PLAN-1020
status: drafting
feature_name: ASE M0基线与公共接口就绪调查
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 0
total_steps: 8
depends_on: []
supersedes_spec_components: []
new_spec_components: [docs/specs/ase/m0-readiness.md]
touched_goals: [GOAL-002, GOAL-007, GOAL-010]
---

# [PLAN-1020] ASE M0基线与公共接口就绪调查

## 0. 变更摘要

r1，drafting，0/8，仅规划。交付M0-BASE：现有Gallery真实链路/成本基线，以及M0-H/T/I各切片可消费与缺失的接口合同。此包是调查，不用它的完成宣称浏览器/新引擎产品运行成功。权威：[ASE M0路线](../design/strategy/ase-m0-delivery-roadmap.md)、[Gallery设计](../design/strategy/autoscape-native-gallery.md)。

## 1. 目标

确认用户体验中的开销实际来自哪些阶段，明确URI/独立源码、ACE公共执行、AutoUI嵌入、事件/状态/关闭分别由谁交付。给出1021的无UI执行入口与1022/1023的具名进入门。

不改现有Gallery代码、不治理全仓性能、不实现新runtime、不安装SDK或重跑ACE/AGE全里程碑。测量所需构建仅在后续本包授权实施时在专属树进行。

## 2. 架构方案

读取Spec→核对实际启动入口→冻结小规模可比操作与环境→分项测量→核验公共接缝→形成可执行就绪记录。以源/进程/监听与实际运行观测相互印证，不从tab数量推断后台拓扑。读取外仓但不写其源码或配置；UI原生窗口错峰。

## 3. 技术栈

现有PowerShell/Python/系统进程观测、Gallery现有Vue/VM入口、现有ACE包模块公开链。只需研究工具与现有构建器，不新增通用监控服务/测试runner。

## 4. 需求分析与背景调查

用户2026-11-12明确授权制定ASE M0 strategy、第一批Plan及与ACE/AGE M2的实施次序，并确定Auto Scape Engine（ASE）为框架总名、AutoScape为浏览器产品。本轮授权为规划/文档，不含代码、fixture、worktree、构建、GUI/探针运行、提交/合入/标签或自动续跑；未指定模型、工期或token预算。既有其他会话在途/暂停决定保留。

取材[Specs overview](../specs/overview.md)、[语言SDK架构](../specs/language-sdk/architecture.md)、[UI仓界](../specs/auto-ui/project.md)和[源码重放规范](../specs/auto-test-case/ace-source-only-replay.md)。overview与部分UI旧测试别名为历史快照，现行AGENTS、固定组及UI本仓配置优先。当前ASE没有canonical现状规范，本包拟议增量仅在实际review/merge后沉淀，不将设计写成已支持。

读取身份：auto-lang HEAD `7b2f2e95967cb3434429093988fae429c05e865f`、auto-ui HEAD `ad9c05da65c9f48ec8a5ab0ef8815cff1ded0fb4`、auto-os HEAD `868c80d3e697ad6eaa9cce27596430a9abe08d5a`；工作区有其它会话文档WIP，内容可与HEAD不同，开工重核实际闭包与祖先。960模块Spec SHA256前16=`be25b0c69f2f8cc0`、公开packages AV实现=`fe9a2a1a36fdc7bf`；970草案=`70c5b6c98c680bac`、980草案=`fcc41243921c0fe7`。这些是只读来源标记，不是本包运行证据。

具体依据：`auto-os/ui-gallery/src/gallery/AppViewport.vue`、`AppViewport.vm.at`、`src/demos-registry.ts`、`src/front/registry.at`、`auto-ui/crates/auto-ui-runtime/src/back_proxy.rs`为已核存在路径；后者支持lazy_sessions而非每tab独立listener。`experimental/ace/crates/av/src/modules/packages.rs`有PackageAvTarget::cold/cold_with_entry/run_value；源码注释明确当前包链无warm body-edit路径。980/HELLO尚为草案，不能冒可用API。

## 5. 详细设计

### 范围、入口与决策

T-01先重核来源、环境与当前运行命令；T-02固定Counter类纯前台及一个现有全栈示例的操作清单。冷应用制品、同进程重复访问、重启、两不同示例顺序访问、隐藏空闲与关闭各测3次；首次冷构建单列不混入应用切换。记录样本数与失败，3次只用于机制基线，不宣称统计P95或正式性能认证。

T-03分别核Vue和旧VM两条真实可用路径，无法运行的格给可重复原因并标pending，不估数。原命令从项目README/实际工具帮助确认后写入§9；所有命令来自专属树，不在master构建/测试，需生成物也不得回写主检出。启动前核原服务归属，只关闭本次owner创建的进程。

T-04只读核验F0/H0/E0/B0/HELLO及事件/状态/会话/公共frame：记录仓/Plan/切片、源SHA/ABI、入口、已验范围、缺口及阻塞consumer。正式落地与规划/WIP分别标记；1021先用960现有公开链，不把它误称969完整facade。最多比较两种frame机制，不运行GPU接缝新实现。

T-05给go/bounded-go/no-go及未执行格。调查可以交付明确no-go，1022/1023所需真实运行门不因此关闭；新机制影响上游时反馈原owner，不直接修改其合同。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ase/m0-readiness.md | 无ASE现状→实测路径、测量边界及接口准入证据，未交能力仍pending | 防基线/设计冒完成 | AC-01..04 |

### 首次执行入口

无新增M2能力前置，但仍需本包实施指令。唯一树`D:/autostack/.wt/lang-1020/auto-lang`，分支`codex/plan-1020-dev`；需要构建外仓则同组真实sibling worktree，禁止主检出测试/链接。T-01登记owner和`D:/autostack/plan-artifacts/plan-1020/`，同步`.retention/policy.json`在途scope；工具/SDK当前若缺只记录，不擅自安装。

## 6. 测试设计

测试与oracle以下均为拟议、未执行；新增测试/CLI入口在对应实施任务创建，不能在创建前报告PASS。实现与内部检查由执行者承担，最终独立review采用独立上下文、同一权威树；本轮未指定模型/人员，不创建agent/chat。

语言仓只按真实触面选择完整固定组及必要消费者，参考现存`ace-av`、`ace-source`、`ace-artifacts`，不为单一外部消费者机械跑`cargo t ace`全并集或`cargo t all`。若改ACE的源码/共享语义须补相应完整组与下游理由；纯ASE消费者不修改ACE时以新增ASE独立目标为主。UI只跑新增/受影响命名目标与公共接缝必要集成，按本仓入口，不沿用语言仓退役别名；原生/GPU/焦点单实例，自动注入与真实用户输入分账。工作迭代按反例/完整职责面，最终review后冻结提交验证受影响并集，身份未变证据可复用。

| Case ID | AC / invariant | 输入/入口 | oracle与边界 |
|---|---|---|---|
| V-01 | AC-01 真实拓扑 | Vue/VM各实际启动/选择/关闭 | 命令、源/环境、进程/线程/listener与UI动作对齐；无法运行明确pending，不填零 |
| V-02 | AC-02 阶段计量 | T-02冻结操作，3次样本 | 准备/编译/装载/服务/首交互/空闲/清理分列；无观测字段unknown |
| V-03 | AC-03 接口可信 | 每个F0/H0/E0/B0/HELLO与会话/frame来源 | 草案/未落主不能判ready；正常公开960入口不能因UI缺失误判不可用 |
| V-04 | AC-04 可执行交接 | 1021/1022/1023进入表 | 每个缺口有owner/consumer/关闭条件；readonly研究不被写成新产品验证 |

验证用实际启动/观测命令及文档链接/结构检查，不运行无关Cargo suite。

## 7. 验收标准

| ID | owner_stage | 验收与方法 |
|---|---|---|
| AC-01 | work | 真实命令/版本/进程监听拓扑可复核，区分proxy lazy与独立进程；V-01 |
| AC-02 | work | 3次冻结场景原始观测和摘要齐，首次红/未执行/环境边界诚实；V-02 |
| AC-03 | work | 现有公开入口与未交接口分开，依赖表有身份/owner/适用范围；V-03 |
| AC-04 | work | go/bounded-go/no-go及后继首次入口成立，所有缺口具名；V-04 |
| AC-R | review | 独立核命令/样本/现状与遗漏；未执行不PASS，无假性能结论 |
| AC-D | merge | 受审知识落Spec、计划归档，原始材料折摘要并按策略清理；不升级上游状态 |

## 8. 执行步骤

- [ ] T-01｜work｜无新增前置｜核身份/授权、建唯一树及材料scope；涉及AGENTS/来源路径；AC-01。核git/source与在途登记，不启动未知既有进程。
- [ ] T-02｜work｜T-01｜冻结两示例/模式/3次操作和采样口径；AC-02。逐项检查输入、输出、失败分类可观察。
- [ ] T-03｜work｜T-02｜专属树运行现有入口并记录基线；AC-01/02，V-01/02；真实命令/exit/样本与UI动作对账。
- [ ] T-04｜work｜T-01｜只读调查现有PackageAvTarget与上游公共接缝，给具名ready/pending；AC-03，V-03。
- [ ] T-05｜work｜T-03/04｜给有界裁决、后继最小前置及无UI先证入口；AC-04，V-04。
- [ ] T-06｜work｜T-05｜汇总§9的命令/数字/内容hash/拟议Spec；逐项AC与链接自检，原材料不入git。
- [ ] T-07｜review｜T-06｜独立核AC、样本和缺口，回报pass/needs_fix/needs_replan；AC-R。此任务未完成不阻work交review。
- [ ] T-08｜merge｜独审pass｜冻结文档定向检查、沉淀SD-01/归档、对账全部输出目录并清自有scope/guard后清树；AC-D。

## 9. 复审记录

stage: new；PLAN-1020 r1；outcome: pass（调查合同可交接，不是实测通过）；next: 取得实施指令后work/T-01。当前0/8，所有V-*未执行，review/merge均未执行。

本轮静态预检：章节/任务数、AC覆盖、相关活动依赖无环及本地链接通过；归档前置作为已交终点，不重新审计其历史依赖。仅规划检查，未运行测试或产品。

## 10. 待澄清事项

T-01核当前工具与基线可运行性，T-04负责接口可消费性，T-05裁决研究结论；这些是有主责的调查任务，不要求用户先回答实现机制。若要扩大安装/产品修复/平台或运行不可信代码，先另定范围。
