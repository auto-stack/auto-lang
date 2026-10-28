---
plan_id: PLAN-878
status: reviewed
feature_name: AutoGraphics跨端边界与依赖审计（XP-0）
author: [Codex]
created_at: 2026-10-28
updated_at: 2026-10-28

plan_revision: 1
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-graphics/platform-boundary-audit.md
touched_goals: [GOAL-008]
affects: [auto-graphics, repository-boundaries]
depends_on: [PLAN-861]
current_step: 9
total_steps: 9
---

# [PLAN-878] 跨端边界与依赖审计（XP-0）

## 0. 变更摘要

对861冻结AG实现、固定旧SDK及相关依赖做有界只读审计，给出桌面/WebGPU/移动边界、可共享部分/平台特有部分、执行环境缺口和XP-1进入建议；附纯仓库迁移与生产融合的分离评估。可与874/875及[877 Gallery](877-autographics-gallery-catalog-and-basic-six.md)并行，不等它们完成，不实现适配/迁仓/依赖抽取。

## 1. 目标

- 从实际manifest/锁/source引用与cfg给出依赖及native调用清单，区分已观察事实、目标解析结果和未测假设。
- 审计共享语义/renderer/宿主/执行adapter与DOM/Canvas边界，形成后续最小探针的具体输入、触面、失败/可用判断。
- 给出移动原生与移动Web各自缺口；评估AG在auto-ui实验目录承接的纯搬仓前置，明确与ACE/HIR统一分离。

非目标：XP-1..5探针、cargo build/check/test、WASM编译、浏览器/GPU/手机运行、移植/抽crate/替依赖、全AutoUI审计、迁仓/生产接线、修改874/875、Gallery内容制作、远端操作。XP-0完成仅表示有可核查审计，不等于任一新平台可运行。

## 2. 架构方案

审计按“固定输入/用途→真实路径/依赖→平台服务/共享语义→证据与不确定性→后继门”组织，非只搜索关键词给出结论。直接依赖、传递依赖、target/feature条件、构建/运行/测试/证据工具用途分别记录。静态source审计与目标metadata解析不能替代目标编译和运行。

书面产物建议位于语言仓新路径`docs/reports/autographics-platform-audit-878/{inputs,dependency-map,platform-seams,next-probes,migration-readiness}.md`及`README.md`索引；原始metadata/命令输出在本地`D:/autostack/plan-artifacts/plan-878/`，不将非Markdown证据加入attachments。没有生产代码产物。

## 3. 技术栈

Git对象、manifest/Cargo.lock、Rust cfg/源码/已有SDK身份工具、Python/PowerShell文本/JSON/TOML读取。允许在隔离导出包使用`cargo metadata --locked --offline --format-version 1`（可加`--no-deps`/`--filter-platform <target>`），仅作不运行build script的依赖解析；不得联网更新/改锁/为解析创建链接。

目标profile：Windows已验证参考、`wasm32-unknown-unknown`浏览器、`aarch64-linux-android`移动原生、`aarch64-apple-ios`移动原生；移动Web/WebView为浏览器宿主约束另列，不用target triple替代。元数据可解析不代表已安装对应SDK/toolchain、可编译或有设备支持；缺缓存/不可解析如实列局限，不自动下载。

## 4. 需求分析与背景调查

授权：用户只要求正式规划XP-0并保留874/875、暂不实施。未来work只按本合同读取固定来源并写审计文档；本轮不创建实现树/运行metadata或审计程序。未授权迁仓/改auto-ui/远端操作。

依据[全局overview](../specs/overview.md)、[AG现状](../specs/auto-graphics/overview.md)、[integration](../specs/auto-graphics/integration-contract.md)、[prototype](../specs/auto-graphics/prototype-profile.md)、[2D角色](../specs/auto-graphics/game-actions-and-character2d.md)、[独立auto-ui](../specs/auto-ui/project.md)、[平台设计](../design/autographics/platform-architecture-and-research.md)与[迁仓候选讨论](../design/autographics/repository-move-and-integration-gates.md)。

固定AG `8b12632d7b5b1b0df72fbb4eef84297a054de31d`（861已交付）；SDK逻辑pin`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`与681文件/17本地包内容锁。旧Git对象不可达时只用已验证保留副本，已观察860消费包SDK路径；不得拿当前auto-lang代替。AGP/driver目前各自`[workspace]`及Cargo.lock、driver指向自身`.sdk-source`和兄弟AGP。

auto-ui已读事实：独立workspace`crates/*`含dialect/compiler/runtime/tools/shell-pack，831 archived 12/12，842 archived 17/17，工作副本clean、master `7095f536747e5180338a470a7d73452c6b550ed1`；runtime显式依赖语言SDK与UI-owned dialect，Iced依赖仍0.14系。README/crates README仍有过时目录，不能代替manifest/实际源码/交付Spec。本轮未运行其构建/测试，不声明所有发行/平台面无缺口。规划指纹见[本批基线](attachments/877/planning/baseline.md)。

## 5. 详细设计

### 5.1 唯一审计树、输入与只读面

实施树`D:/autostack/.wt/lang-878/auto-lang`，分支`plan-878-dev`，从实施时语言主检出已提交基线创建并记录完整SHA；AG审计输入始终为上述8b12632d7的Git导出，不能因主树缺experimental目录而误报缺失。保留SDK在自己的隔离目录逐文件验证；auto-ui只读固定7095f53对象/manifest，不借其WIP。读与写根分离，不移动/编辑原AG/SDK/auto-ui目录，不改当前锁或其他计划证据。

同树顺序work/独立review，review暂停写入；主检出仅Plan簿记。可与874/875/877并行，不共用target/metadata输出/consumer准备目录；不需要GPU或窗口焦点。若固定源/必需SDK身份无法验证，T-01不能关闭；metadata离线不足不冒称闭包可移植，按下文记录未解析边。

### 5.2 有界覆盖集与证据规则

必须覆盖AG两个workspace全部受管manifest/build入口/锁、实际source文件清单；固定SDK的17个本地包及其引用边；外部依赖按锁定版本/feature/target用途形成可追溯图。auto-ui只读取workspace/五包manifest、相关版本/patch/发行/测试触面及831/842现行Spec；不扩成全UI实现复审。

每条结论最少记录：来源repo/完整commit或内容SHA、文件/符号、方向/用途、feature/target/cfg、平台分类、观察方法及限制；推测API可用性单列hypothesis。直接/间接/仅测试/仅工具/native运行必需不混为一类；锁里有包不等于该目标可达，源码写`cfg`也不证明依赖边已经隔离。

T-02先冻结`attachments/878/work/audit-contract.md`的root集合、输出列、默认/具名feature与target选择及判定词汇。允许metadata得出完整/部分/不可解析；已锁定第三方源码在本机可用时抽查相应平台关键边，缺源码/缓存如实标unknown。审计结果至少分类shared-candidate、host-only、adapter-needed、unresolved；不得以native-only关键字一次搜索代替全文件/依赖清单核销。

### 5.3 审计触面与判别对照

| 触面 | 实际观察入口/候选 | 必须区分的结论 |
|---|---|---|
| 场景/身份/时钟/事件 | AGP `src/core/{scene,id,time,events}.rs`、driver document/play/source admission | 纯语义候选与宿主owner/同步调度混入；u64/JS Number、f32/Run/tick/代次精确桥接要求 |
| renderer/资产 | AGP `src/render2d/`、`src/render3d/`、`src/renderer/`、`src/assets/glb.rs`、着色器/纹理格式 | wgpu可移植路线与当前资源/feature/同步readback/目标句柄假设，静态GLB支持集不扩张 |
| 窗口/UI/输入/IME | AGP host/iced adapter；driver `src/bin/autographics-authoring.rs`、`src/bin/authoring/cli.rs`、authoring host/input | winit/桌面逻辑属于宿主；真实控件/焦点/投影与平台回调区分，DOM非直接纹理来源 |
| OS/文件/线程 | native working_set/OS输入、Storage/SDK读取与Session执行/超时/worker | 仅测量/探针使用的Windows API与正常路径必需项分开；同步等待/线程/文件不能由cfg或WASM目标名自动消失 |
| 音频/生命周期 | Kira/codec配置、surface/设备owner、Play/Stop | 音频依赖/用户激活/挂起是未测平台要求；device失效保逻辑状态不等于保像素，重建/旧代次隔离缺口 |
| 构建/消费/工具 | 两Cargo.toml/lock、prepare_sdk/package_consumer/current verifier/根栈配置 | 产品路径与Python/Git/证据工具分离；当前源闭包补齐/路径假设、迁仓后身份/导出影响 |

上述路径已确认存在；具体可移植性是待审计结论。对照必须含“平台特有仅测量代码不能据此判整个core不可移植”“无Windows关键词但通过传递依赖/同步文件仍受限”“真实已隔离纯数据路径”三类，避免全红或全绿的分类器。

### 5.4 后续研究与纯迁仓评估

next-probes输出XP-1具体最小场景/输入来源（可引用877未交付内容为未来候选，但**不依赖877完成**，也可用当前冻结scene/模型）、渲染/执行/UI三种结论边界、依赖缺口、触面owner、正负oracle与资源规模建议。不是实现浏览器宿主，不以静态审计输出“WASM已通过”。XP-2/3/4只做研究接口/风险清单，不创建探针。

migration-readiness根据auto-ui实际独立状态评估：在`auto-ui/experimental/autographics-{prototype,auto-driver}`保留嵌套独立workspace/锁/旧SDK，是否可保持AG只读消费、默认UI零新增依赖/构建负担、无语言→UI→语言重环；列生产源码/测试/工具/小夹具/文档/旧收据路径与当前身份manifest的迁移触面。纯仓归属切换、公开运行/声明融合、ACE替换、切默认/退役各有独立门，不因auto-ui已可用自动成立。

迁仓时间只给候选：建议874/875实际交付冻结后、下一批共享运行协议开发前设置短暂停写窗口，核XP-0/目标owner/恢复与两端门后另立纯搬仓Plan；它在技术上可以早于ACE成熟。此前“归仓与融合一并等ACE”是现行历史排期，本审计不得自行取消它或启动迁移；只提交调整建议供后续决策。迁仓研究不是本计划交付强制实现，不额外分第三个计划号。

### 5.5 审计结果和失败界

最终输出`audit_complete`（覆盖集是否逐项分类）和`next_probe_readiness`（ready/limited/requires_replan，及每项阻塞），将两者分开。必要输入验证失败、root漏项、事实无来源、关键字段/未测伪装支持使AC失败。若已完整诚实记录静态可见/未解析边，limited或requires_replan可以交付审计，但不得放行未解决的XP-1相关前置、宣称跨端支持或要求当前M2等待。

Markdown包含命令/退出码/计数/路径/hash和文件/符号引用；原始metadata在本地保留。交付文档为真正知识产物可入`docs/reports/.../*.md`，不是将大量原始日志换扩展名入库。遵守attachments非md证据不入库政策。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/platform-boundary-audit.md | 跨端设计假设→固定版本静态边界/依赖审计、已知限制/未解析边、XP前置与非支持声明 | 后续探针有事实起点 | AC-01..05 |

Spec只沉淀实际审计事实与边界，不能新增已实现平台能力；不改canonical auto-ui/AG接口或现行ACE/迁移合同。

## 6. 测试设计

本计划为文档/只读审计，无功能测试/编译/浏览器/设备门。以下是审计可重复性与分类区分检查，实施与同树独立最终review顺序执行，无用户指定模型/agent。

| Case ID | AC/不变量 | 实际入口/输入 | 可观察oracle/错误对照 | work任务/覆盖 |
|---|---|---|---|---|
| V-01 | AC-01 输入身份 | Git树/SDK内容锁/两锁/auto-ui固定commit | 清单逐项存在/内容一致；当前master或错误SDK不得冒充pin；缺输入不计完成 | T-01/02；冻结范围非全仓 |
| V-02 | AC-02 覆盖/依赖 | manifests/source集、锁/metadata、17SDK包root | 全root有分类/出处；直接、传递、feature/cfg、仅测试工具分别核销；metadata失败不写目标可达已证明 | T-02/03；静态非编译 |
| V-03 | AC-03 真假平台边界 | §5.3三个分类对照；窗口/存储/线程/GPU/音频入口 | 仅测量native不污染core判断；隐藏传递边不误标shared；合法纯数据候选不过度判native | T-03/04；需实际文件符号反证 |
| V-04 | AC-04 DOM/执行/生命周期 | 真实平台设计/SDK/发布栈与文档引用 | DOM overlay与可采样UI、renderer可用与Auto执行、device失效与保旧逻辑/像素分别列明；无未运行pass | T-04；只形成后继决策 |
| V-05 | AC-05 收口/迁仓 | README/next-probes/migration-readiness、auto-ui实际manifest与831/842事实 | 覆盖计数与局限一致；XP-1输入不依赖未交付877；纯迁仓与融合/ACE门分开，既定排期未被静默修改 | T-05；无迁仓执行 |

验证命令以只读Git/SDK verify、允许的离线metadata、文档链接/引用与diff检查为主，T-02冻结具体命令和输入；不新增通用审计框架/分类器或写运行探针。独立review从固定对象抽查关键结论和代表性反例，不仅核对复述。merge只做最终文档/清单一致性门与知识/归档，不跑根cargo t/tf/tv/tu/docs_gen或AG完整档。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 唯一审计树、AG/SDK/auto-ui固定输入及写面核实，V-01 |
| AC-02 | work | roots/source/manifest/依赖分类覆盖闭合，target/feature/用途/未解析边可追溯，V-02 |
| AC-03 | work | 共享/宿主/需adapter/未知触面有实际来源及区分对照，V-03 |
| AC-04 | work | DOM/Canvas/3D、执行与渲染、输入/资源/生命周期/移动两路线边界与未测项明确，V-04 |
| AC-05 | work | audit_complete/readiness、XP-1具体入口/阻塞、纯迁仓候选与保持面完整，V-05，不宣称适配完成 |
| AC-06 | review | 同树独立核AC/V/SD、覆盖与原始来源、代表性反证/未测分类、零实现越界；merge门pending |
| AC-07 | merge | 冻结文档/清单/引用/计数一致，实际输入/并行写面与受保护计划未变 |
| AC-08 | merge | 报告/Spec/ledger/索引沉淀，归档/guard清理收据，XP后继不被自动启动 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 861交付（已满足） | master基面唯一lang-878树，固定AG对象/SDK副本/auto-ui对象只读核验 | AC-01 | [x] 12/12 AG blob、681/681 SDK 文件、6/6 auto-ui blob、基线8ae51d6af |
| T-02 | work | T-01 | audit-contract冻结roots/feature/target/输出列/命令，输入/清单基线与raw证据目录 | AC-01/02 | [x] attachments/878/work/audit-contract.md |
| T-03 | work | T-02 | dependency-map与完整source分类，离线metadata可用则解析，三类反证核对 | AC-02/03 | [x] 10/10 metadata complete；222文件全分类；三对照§6 |
| T-04 | work | T-03 | platform-seams：DOM/执行/输入/资源/生命周期/移动与共享候选/阻塞，V-03/04 | AC-03/04 | [x] 六触面+DOM边界+移动两路线+U-01..08/H-01..03 |
| T-05 | work | T-04 | next-probes/migration-readiness/README、计数/限制与readiness收口，提交报告/SD草案 | AC-01..05 | [x] audit_complete+limited(R-01..04)；commit 6f41dfacf |
| T-06 | review | T-05 | 同树独立从固定源核审计与代表性反证，merge文档门pending | AC-06 | [x] final pass→F-01/02修复→d2ef25469复核pass |
| T-07 | merge | T-06 pass | 冻结输入/文档/清单/链接/无源码修改门，不运行Cargo档 | AC-07 | [x] 独立复核冻结文档一致；md-only commits 零源码/零锁改动；Category A 零 Cargo 门（AGENTS.md 纯验证/文档任务禁跑 cargo t/docs_gen） |
| T-08 | merge | T-07 | 仅报告/知识合主检出，Spec/ledger/索引与归档，不合AG源码或迁仓 | AC-08 | [x] delivery a313bf0ca ff-only 落 master；ledger v118(+3, 离线投影)；overview/plans.md 索引；本归档提交 |
| T-09 | merge | T-08 | guard clean后清树/分支/组目录，保留本地raw与实际9/9收据 | AC-08 | [x] wt-guard clean；树/分支/组目录已清（branch was a313bf0ca）；本地raw 54MB 保留无 target/.pdb 可回收 |

## 9. 复审记录

2026-10-28 new r1：`stage: new | plan_id: PLAN-878 | plan_revision: 1 | outcome: pass | next: 等用户启动work后T-01/02`。861已交付；不依赖874/875/877完成。全部T/AC/V/SD未执行，本轮只核计划输入与文档，不运行审计/metadata/构建/浏览器/GPU/真机、无新树/代码/迁仓/远端操作。

2026-10-28 work r1：`stage: work | plan_id: PLAN-878 | plan_revision: 1 | outcome: pass | worktree: D:/autostack/.wt/lang-878/auto-lang | branch: plan-878-dev | base_commit: 8ae51d6afa5c2b2cc3bc72833c6408d212f5680b | code_commit: 6f41dfacf65b7057f352dbc642c04f1e80c3d625 | dependency_revisions: AG=auto-lang@8b12632d（git archive 导出 plan-878/ag-export）、SDK=plan-860保留副本（681文件逐SHA-256核验，pin 7b987ca8）、auto-ui=7095f53（只读） | task_ids: T-01..T-05 | evidence: 报告6篇+SD草案（docs/reports/autographics-platform-audit-878/、docs/specs/auto-graphics/platform-boundary-audit.md，8文件827行，仅md入库）；本地raw=plan-artifacts/plan-878/（meta-*.json ×10全exit0、verify_sdk_copy.py PASS、src-classify.json 222行） | blockers: 无 | next_responsible_role: review（T-06 同树独立，AC-06）`

- AC-01✅ 输入身份全核验（12 AG blob+12 hash-object、681 SDK 文件、6 auto-ui blob、唯一树/分支、主检出零代码WIP）。
- AC-02✅ roots闭合：AGP 478/driver 787 锁条目、10/10 metadata 解析 complete（5 target×2 workspace）、SDK 闭包 15/17（auto-test-case+autodown-test-parser 为测试包非产品闭包）、222 文件逐文件分类（22 shared-candidate/16 adapter-needed/73 host-only/106 evidence-tool/5 其他）。
- AC-03✅ 三强制对照有实际文件/符号反证（gpu_foundation 仅测量路径、SDK 栈无关键词但全 target 无门控、output.rs/composition 纯数据候选）。
- AC-04✅ 六触面+DOM/Canvas/3D 边界+移动两路线缺口+生命周期未测项显式列表（U-01..U-08、H-01..H-03），无未运行 pass 冒称。
- AC-05✅ audit_complete=complete、next_probe_readiness=limited（R-01..R-04 各附绕行）；XP-1 输入不依赖 877（P-1 可用冻结 scene/模型）；纯迁仓四门分离且"归仓可先于 ACE"仅作确认不取消现行排期；无适配完成宣称。
- 备注：spec 草案内指向本 Plan 文件的链接在合并提交计划簿记后在主检出可解析（本 Plan md 当前为 master 未跟踪文件——877/878 规划批次文件均未入 git，属批主所有，work 未代为提交）。
- 禁令遵守：零 cargo build/check/test、零网络、零链接、零改锁、零改 874/875/860/861/877 证据、worktree 内无 junction/symlink；merge 门（裸 cargo t 等）不适用——本计划无 crates 改动（Category A 纯验证/文档）。

`stage: goal | plan_id: PLAN-878 | plan_revision: 1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-878/auto-lang | commit: 6f41dfacf65b7057f352dbc642c04f1e80c3d625 | next: review`

2026-10-28 review r1（final）：`stage: review | review_scope: final | reviewer_identity: 独立 general-purpose 子代理上下文（同会话宿主、零执行上下文交接、从工件重建裁定；非第二检出） | plan_id: PLAN-878 | plan_revision: 1 | outcome: pass（6f41dfacf 首轮 → F-01/F-02 修复 d2ef25469 复核 pass） | worktree: D:/autostack/.wt/lang-878/auto-lang | branch: plan-878-dev | reviewed_commit: d2ef2546987669c51bbb820ac99fb72c3c2f7fd1 | base_commit: 8ae51d6afa5c2b2cc3bc72833c6408d212f5680b | dependency_revisions: AG=8b12632d、SDK=681文件锁（pin 7b987ca8，复核 PASS）、auto-ui=7095f53 | spec_inputs: docs/specs/auto-graphics/platform-boundary-audit.md（SD-01 草案，链接除待合并 Plan 文件外全部可解析） | acceptance_results: AC-01..05 pass、AC-06 pass（独立复核：blob/锁/计数/三对照/反例全部重放）、AC-07/08 pending merge 门 | pending_closeout_gates: 无 Cargo 档（Category A 纯文档）；T-07/T-08 文档/归档/沉淀门由 merge 执行 | findings: F-01(P2,幽灵H-04/H-05区间串,已修d2ef25469)、F-02(P2,合同R1/R2计数80/140→65+14/51+89,已修d2ef25469)、F-03(P2注记,driver main.rs evidence-tool/host-only边界属判断,接受不改) | evidence: 复核命令与结果见子代理记录（blob 5+2抽查、verify_sdk_copy.py重跑PASS 681、presence矩阵逐值重算、222计数重分类、链接全检、无越权非md文件、无锁改动、worktree零reparse point） | next: merge`

裁定要点：独立复核重放了 12 blob 中的 5、auto-ui blob 中的 2（其余 12/12 与 6/6 为 work 首轮全量核验）、SDK 681 文件全量重验 PASS、metadata 8 项断言逐值命中、三对照文件/符号级命中、222 文件分类零 sdk-source 泄漏、无夸大支持声明、XP-1 不依赖 877、四门未捆绑。修复未改任何证据/数字结论，仅两处区间串与两处计数措辞。

`stage: goal | plan_id: PLAN-878 | plan_revision: 1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-878/auto-lang | commit: d2ef2546987669c51bbb820ac99fb72c3c2f7fd1 | next: merge`

## 10. 待澄清事项

XP-0静态审计范围已明确，实际可移植性/XP-1前置是输出，不预设通过。若必要固定输入不可取证，保留阻塞而非用新SDK/分支替换。用户仅授权规划，禁止立即T-01；纯仓迁移是独立候选建议，不以本Plan代替迁仓实施决策或提前集成ACE。

2026-10-28 merge r1：`stage: merge | plan_id: PLAN-878:r1 | outcome: pass | delivery_commit: a313bf0ca99dfddba10ef0c0afd3fc63f0ac8de9（=reviewed d2ef25469 的 projection-only 后代：仅 overview/plans.md/ledger 三文件索引沉淀，无实现/依赖变更，免再复审） | land: master ff-only 8ae51d6af→a313bf0ca，无 merge commit，线性历史 | canonical_specs: docs/specs/auto-graphics/platform-boundary-audit.md（SD-01 add；sha256 36ed3c2fe250f5a6…）+ overview.md/plans.md 索引回写 | ledger: .autoos/specs.json v117→v118（guarded offline projection——同日 PLAN-867 先例，store writer 不可达；+designs P878-PLATFORM-BOUNDARY-AUDIT、+reviews P878-REVIEW-FINAL、+reports P878-MERGE，既有条目零变更，备份 plan-artifacts/plan-878/specs.backup.v117.json） | archive: docs/plans/archive/878-autographics-platform-boundary-and-dependency-audit.md（status: archived，随本提交首次入 git，K13 满足） | gates: Category A 纯验证/文档——零 Cargo 门（AGENTS.md 禁跑）；batch regression 不到期（.last-batch-regression.json last_covered=860 当日 09:33Z，878%5=3≠0，<48h） | landing-not-deployment: 纯文档落地，无生产二进制/web bundle 触面，无 stale 项 | cleaned: wt-guard clean→worktree/plan-878-dev 分支/组目录已清；plan-artifacts/plan-878 54MB raw 保留（ag-export/metadata JSON×10/src 分类/备份），无 target 树/.pdb 可回收 | next: done`

completion_kind: delivered。XP-0 审计交付完成：audit_complete=complete、next_probe_readiness=limited（R-01..R-04）。XP-1..5 与迁仓均未被本 Plan 启动，各需独立合同；874/875/877 触面零改动（874 SHA 8e8d2b60…/875 4b88df0d… 未验证改动——本 Plan 全程未触碰其文件）。

`stage: goal | plan_id: PLAN-878 | plan_revision: 1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: （已清） | commit: a313bf0ca99dfddba10ef0c0afd3fc63f0ac8de9 | next: done`
