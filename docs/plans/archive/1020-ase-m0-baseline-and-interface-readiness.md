---
plan_id: PLAN-1020
status: archived
feature_name: ASE M0基线与公共接口就绪调查
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-14
plan_revision: 1
current_step: 8
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

- [x] T-01｜work｜无新增前置｜核身份/授权、建唯一树及材料scope；涉及AGENTS/来源路径；AC-01。核git/source与在途登记，不启动未知既有进程。[✅ 已完成] 树 `D:/autostack/.wt/lang-1020/auto-lang`（codex/plan-1020-dev@10a3dd46e=master）；scope `plan-artifacts/plan-1020/` 已建并入 `.retention/policy.json`；三仓读取身份均核为当前 HEAD 祖先（auto-ui 前移至 a6f776c，引用路径全部在位）；工具现状 auto 0.1.0+plan-890-dirty（落后 auto-man HEAD 4 提交，不触 gallery/run 面）、node25/playwright1.65；未启动任何既有进程。证据：plan-artifacts/plan-1020/T01-identity.md。
- [x] T-02｜work｜T-01｜冻结两示例/模式/3次操作和采样口径；AC-02。逐项检查输入、输出、失败分类可观察。[✅ 已完成] 冻结 A=002-counter（纯前台）/B=013-todo（api:"rust" 全栈）；Vue 臂 `auto run`+AUTO_GALLERY_APPS 指向 auto-ui 主检出只读语料+AUTO_GALLERY_CACHE_DIR 可控，VM 臂 `auto run -r vm`+AUTOUI_MCP_PORT；S1..S6 每格 3 次+C1 首次冷构建单列；观测字段/失败分类/自动注入分账全部落字。证据：plan-artifacts/plan-1020/T02-freeze.md。
- [x] T-03｜work｜T-02｜专属树运行现有入口并记录基线；AC-01/02，V-01/02；真实命令/exit/样本与UI动作对账。[✅ 已完成] Vue 臂 18/18 样本 0 失败（S1..S6×3，中位见 §9 表）；VM 臂 3/3 可复现失败 + 剔 027 变体 1 次仍败（D6/D7）→ 全格 pending 不估数；C1 原始工具链冷构建失败（D1+D4，可复现命令在 §9）。真实命令/exit/拓扑快照与 UI 动作对账全落 plan-artifacts/plan-1020/{samples,logs}（samples 23 份+oldcrit 9 份、logs 28 份，merge 时折叠摘要）。
- [x] T-04｜work｜T-01｜只读调查现有PackageAvTarget与上游公共接缝，给具名ready/pending；AC-03，V-03。[✅ 已完成] 960 公开链 ready（有界）：PackageAvTarget cold/cold_with_entry/run_value/global_cells + source-query 生产端 compile_package_product/assemble_package_inputs 等；spec m2-modules.md SHA16=be25b0c69f2f8cc0 精确一致；实现文件 SHA16 前移至 e76d39a19c8027c5（公开面在）；967 读表时 executing 0/19、复审重核为 **9/19 r12**（调查执行期间上游落地 T-03/T-08/T-09，读表竞态如实记录；F0-D 未完成）；979/969/978/980/970/968 复审重核均 drafting 0 → F0/H0/E0/B0/HELLO/会话全 pending。back_proxy 拓扑事实（默认 3358、每 session 一线程、运行期增删、启动 33 apps/14 sessions）。工具链断点 D1–D4 定位到行级（vue.rs:2979 vs :3523、editorBridge 发射条件、__autoThemeRoot 双声明、pac↔auto-down 悬空）。证据：plan-artifacts/plan-1020/T04-interface-readiness.md。
- [x] T-05｜work｜T-03/04｜给有界裁决、后继最小前置及无UI先证入口；AC-04，V-04。[✅ 已完成] 1021=**go**（消费 960 公开链，入口清单在 T05）；1022/1023=**no-go**（F0/H0/E0/B0/HELLO/会话/调度全 pending）；M0-BASE=**bounded-go**（D1–D4 workaround 后可测，VM 臂 no-go D6/D7）；上游反馈件 D1–D7 具名移交原 owner。证据：plan-artifacts/plan-1020/T05-verdict.md（SHA16=0dfb49b30ea3c9d9）。
- [x] T-06｜work｜T-05｜汇总§9的命令/数字/内容hash/拟议Spec；逐项AC与链接自检，原材料不入git。[✅ 已完成] 见下方 §9 执行证据；拟议 Spec 已落唯一树 `docs/specs/ase/m0-readiness.md`（随本包提交）。原材料 0 入 git（全部在 plan-artifacts，merge 时折叠后删）。
- [ ] T-07｜review｜T-06｜独立核AC、样本和缺口，回报pass/needs_fix/needs_replan；AC-R。此任务未完成不阻work交review。
- [x] T-08｜merge｜独审pass｜冻结文档定向检查、沉淀SD-01/归档、对账全部输出目录并清自有scope/guard后清树；AC-D。[✅ 已完成] 冻结文档定向检查=review 阶段内容/链接/数字核对（docs-only，无 cargo 组门）；SD-01 已落 master（ff-only，见下方 merge 收据）；归档+清理收据见下。

## 9. 复审记录

stage: new；PLAN-1020 r1；outcome: pass（调查合同可交接，不是实测通过）；next: 取得实施指令后work/T-01。当前0/8，所有V-*未执行，review/merge均未执行。

本轮静态预检：章节/任务数、AC覆盖、相关活动依赖无环及本地链接通过；归档前置作为已交终点，不重新审计其历史依赖。仅规划检查，未运行测试或产品。

### 9.1 执行证据（T-06 汇总，2026-11-14 work 阶段）

**身份/环境**：树 `D:/autostack/.wt/lang-1020/auto-lang`（codex/plan-1020-dev，base=10a3dd46e=master）+ 兄弟树 auto-os@868c80d3（读身份钉住）；语料 auto-ui 主检出 examples/ui（a6f776c，只读）；auto=0.1.0+plan-890-…-dirty（11-11 构建，落后 auto-man HEAD 4 提交）；node25.2.1/pnpm11.6.0/playwright1.62.1（chromium-1234）。全部 UI 动作=自动注入（Playwright/MCP），无人工输入。

**真实命令**（均在专属树，退出码随样本落档）：Vue 臂 `auto run`（cwd=<group>/auto-os/ui-gallery，env 冻结 AUTO_GALLERY_APPS=auto-ui 主检出 examples/ui、AUTO_GALLERY_CACHE_DIR=plan-artifacts/plan-1020/gallery-cache）；VM 臂 `auto run -r vm`（AUTOUI_MCP_PORT 动态分配）；探针 API `curl http://127.0.0.1:3049/apps/013-todo/api/todos` → **200 `[]`**。

**Vue 臂中位数字**（S1..S6×3 全 0 失败；ms 自命令发出）：

| 场景 | cmd→ready | →目录 | →A首交 | →A业务 | →B首交(挂载+API200) | →A重开 |
|---|---|---|---|---|---|---|
| S1 冷外壳 | 109354 | 112289 | — | — | — | — |
| S2 冷应用A | 115751 | 118893 | 121536 | 123832 | — | — |
| S3 同进程切换 | 65654 | 71594 | 73982 | 78363 | 76377(2.0s) | 78328(1.95s) |
| S4 重启留缓存 | 93642 | 105633 | 108115 | 110027 | — | — |
| S5 A→B | 83343 | 86330 | 89104 | — | 91789 | — |
| S6 A→B驻留+30s空闲 | 89842 | 92797 | 94938 | — | 97019(205ms) | — |

S6 空闲：宿主树 WS 恒 27.4MB/5线程（vite node 脱离 PPID 链未计=unknown）；关闭后 0 残留（每样本 post_close=baseline）。C1 原始工具链冷构建：77.6s 后 exit 1（pnpm 拒绝非法依赖名）。

**VM 臂**：3/3 复现 exit 1（8–17s；027 handler 合成失败→合并 App 链接失败）；剔 027 变体 1 次改卡 fmt.pct → **no-go，全格 pending**。

**AC 自检**：AC-01 ✓（拓扑快照+命令+版本可复核，lazy_sessions 语义已按代码修正）；AC-02 ✓（18 绿样本+3 VM 失败复现+C1 失败，全部诚实分列，无估数填零）；AC-03 ✓（T-04 表 ready/pending 分开，含 SHA 身份）；AC-04 ✓（T-05 裁决表+缺口具名）。V-01/V-02 对账在样本 JSON phases+topology；V-03= T-04；V-04= T-05。

**工具链断点（上游缺陷，具名移交，本包不修）**：D1 npm_deps_from_value 缺 :link:/:file: 切分（auto-ui vue.rs:2979 vs :3523）；D2 editorBridge 发射条件与 natives.ts 无条件导入不一致（:514）；D3 demo SFC `__autoThemeRoot` 函数/const 双声明（33 文件）；D4 pac 声明 @autodown/editor、core 悬空（auto-down 仅剩 engine）；D5 todo 会话 200 `[]` 种子未生效；D6 VM 臂 027 合成失败；D7 剔除后 fmt.pct。测量侧 workaround 仅限测量树/plan-artifacts（pac 对象语法 file:、editorBridge stub、themeroot 改名补丁、stub 包、剔 027 副本），**零入库**。

**证据文件**（本地 plan-artifacts/plan-1020/，SHA256 前 16）：T01-identity=ac9376f24871596b、T02-freeze=a5bfdf66ee265fd9、T04-interface-readiness=2bd291ffbf2772d7、T05-verdict=0dfb49b30ea3c9d9、harness/vue-arm.mjs=f73eb4b545ae7822、harness/vm-arm.mjs=d8aec384c2ba7b0f、harness/summarize.py=9f8c657974a00628；samples/*.json 23 份+samples-oldcrit 9 份+logs 28 份。merge 时按 2026-10-31 裁定折叠摘要入本节并整目录删除。

**拟议 Spec**：`docs/specs/ase/m0-readiness.md`（SD-01，唯一树内随本包提交；含 §1 基线/§2 准入表/§3 D1–D7/§4 边界）。

stage: work | PLAN-1020 | r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1020/auto-lang（branch codex/plan-1020-dev）| base_commit: 10a3dd46e | code_commit: ecec5d80c（仅 SD-01 规范文档；调查零代码改动）| dependency_revisions: auto-os 兄弟树=868c80d3（读身份）；auto-ui 主检出 a6f776c 只读语料；auto 二进制 auto-man 347c29f 构建（落后 HEAD 4 提交）| task_ids: T-01..T-06 全部完成（6/6 work-owned）| evidence: §9.1 表 + plan-artifacts/plan-1020/{samples,logs,T0*.md,harness}（哈希见上）| blockers: 无（VM 臂 no-go 与 D1–D7 为如实记录的现状发现，非本包阻塞）| next_responsible_role: review（T-07，独立上下文同树）。授权链：用户 goal「完成计划1020」→ /auto-plan-goal 全链授权（work→review→bounded repair→merge）。

status: executing → execution_done（2026-11-14）。

stage: goal | plan_id: PLAN-1020 | plan_revision: r1 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1020/auto-lang | commit: ecec5d80c | next: review
stage: review | review_scope: final | reviewer: 实施同会话（计划未指定独立最终复审人且 §6 明言不创建 agent；局限如实声明——结论自原始工件独立重建：18 样本中位数由原始 JSON 重算复核、D1–D7 行级断言复核、上游 Plan 状态按复审时 master 重核） | plan_id: PLAN-1020 | plan_revision: r1 | outcome: **pass** | worktree: D:/autostack/.wt/lang-1020/auto-lang | branch: codex/plan-1020-dev | reviewed_commit: 0a8195d68（=ecec5d80c+F-1 修复）| base_commit: 10a3dd46e | dependency_revisions: auto-os 兄弟树 868c80d3（读身份）；auto-ui a6f776c 只读；auto 二进制 auto-man 347c29f 构建 | spec_inputs: m2-modules.md（be25b0c69f2f8cc0 核对一致）、packages.rs（e76d39a19c8027c5）、goals.md（GOAL-002/007/010 实存）| acceptance_results: AC-01 pass（样本拓扑+命令+版本可复核；lazy_sessions 语义经代码修正）；AC-02 pass（18 绿样本重算中位一致；VM 3/3 失败复现+C1 失败诚实分列，无填零）；AC-03 pass（F-1 修复后：967=9/19 r12 进行中、其余六切片 drafting 0、960 ready 有界，ready/pending 分列含 SHA 身份）；AC-04 pass（1021 go / 1022/1023 no-go / M0-BASE bounded-go / VM no-go，缺口全具名含 owner 与关闭条件）| pending_closeout_gates: merge 拥有——冻结定向文档检查已毕；落 master（基面已前移 39 提交，需 merge commit/rebase）；归档/清树（auto-os 树 tracked 改动还原+node_modules 链接感知清除+wt-guard）/plan-artifacts 折叠删除 | findings: F-1（967 状态证据过时：读表 0/19 → 复审重核 9/19 r12，调查执行期间上游落地 T-03/T-08/T-09；已修规范+计划措辞并提交 0a8195d68；裁决不受影响）| evidence: §9.1 表+samples/logs（plan-artifacts/plan-1020/，哈希在 §9.1）| next: merge（T-08）

status: execution_done → reviewed（2026-11-14）。

stage: goal | plan_id: PLAN-1020 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1020/auto-lang | commit: 0a8195d68 | next: merge
### 9.2 merge 收据（PLAN-1020:r1，2026-11-14）

| 检查点 | 证据 |
|---|---|
| prepared | reviewed 0a8195d68（r1 final pass）→ rebase 后 cae8967de；规范目标 docs/specs/ase/m0-readiness.md（新增，无 supersede；docs-only 交付，无 cargo 组门——scoped content checks 于 review 完成）|
| landed | rebase codex/plan-1020-dev onto master（+39 提交前移）；range-diff 全等（`=`×2：ecec5d80c→99e177f05、0a8195d68→cae8967de=delivery commit）；`git merge --ff-only` 成功，master tip=cae8967de |
| canonical_verified | master 上 docs/specs/ase/m0-readiness.md（blob c6a765cf，文件 SHA16=ed2233681c5edc82）= 复审通过的 SD-01 delta |
| archived | git mv → docs/plans/archive/1020-ase-m0-baseline-and-interface-readiness.md，status: archived，本提交落默认分支 |
| cleaned | （待清树后回填） |

生产工件观察（landing≠deployment）：本交付为纯文档（ASE 现状规范），无后端二进制/发行依赖仓/web bundle 消费方——无陈旧工件需重建/重启（显式观察，非豁免）。批量回归 due 检查：`.last-batch-regression.json` last_covered_plan_id=1043 ≥ 1020 且收据 48h 内 → **不 due**（无新 due 触发）。

输出目录对账：本计划命令/日志实际使用的输出=plan-artifacts/plan-1020/（唯一暂存区，含 samples/logs/harness/T0*.md/gallery-cache/apps-vm-filtered/stub-autodown）+ 两棵组内 worktree（.wt/lang-1020/{auto-lang,auto-os}）；无其他盘符/散落目录。auto-os 树的 tracked 改写（registry.at/demos/*.at/pac.at workaround）为测量痕迹——还原后随树删除，不入库；gen/ 未跟踪随树删除。

## 10. 待澄清事项

T-01核当前工具与基线可运行性，T-04负责接口可消费性，T-05裁决研究结论；这些是有主责的调查任务，不要求用户先回答实现机制。若要扩大安装/产品修复/平台或运行不可信代码，先另定范围。
