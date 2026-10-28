---
plan_id: PLAN-860
status: archived
feature_name: AutoGraphics真实Auto角色运动与运行预算可行性
author: [Codex]
created_at: 2026-10-27
updated_at: 2026-10-28
plan_revision: 1
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-graphics/game-runtime-feasibility.md
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 9
total_steps: 9
---

# [PLAN-860] AutoGraphics真实Auto角色运动与运行预算可行性

## 0. 变更摘要

M2/S2第一片：消费858已冻结的G2/G3关卡，以真实Auto/固定SDK证明角色运动、地面/墙顶判定和目标路线在有限资源下是否可行，给861公共输入/角色接口提供实际机制和预算依据。857仍在review/fix；本轮只规划，未冻结其最终源码，不写857树。

## 1. 目标

- 把G2-01..03、G3-01..03及角色可抵达收集物/终点的必要条件落实为有界运行探针，不用理想抛物线或Rust执行玩法代替。
- 识别旧v5“最多两接触、只输出最后法线”对最终grounded、地面+墙角、平台边缘的具体缺口，最多比较两个有限接法。
- 实测128节点wide/deep、真实样本和失败恢复下的Auto/managed/CPU/工作集/owner及宿主收集成本。
- 输出明确go或requires_replan决策，绑定实际源码/配置/输入和继承门；不把负面可行性结论写成公共引擎已实现。

非目标：正式公共新API、动态对象/触发服务、游戏相机/精灵/材质动画/声音/HUD、完整小游戏/独立仓/发行、增加SDK/依赖/锁/预算、修改旧API算法或生产UI/HIR/ACE。运行分层见[专题设计](../design/autographics/m2-runtime-and-character-control.md)，保持面/交接见[共同合同](attachments/860/planning/runtime-contract.md)。

## 2. 架构方案

保留公开SDK Session、原预算和原Host/Play所有权。先以现有behavior/5分层组合得到事实，再允许在**独立probe-only受信Auto单元**里试验有界接触摘要/最终支撑判定，不改原`.at`/assembler或声称新API已公开。每逻辑步仅一次角色更新调用，Rust只送输入、校验/发布完整输出、计量和作独立测试oracle，不能执行碰撞/跳跃业务。

probe-only候选明确来源区域、长度、hash、固定输入/输出和状态上限；其原始Session路径与现行用户准入路径分别记录，不能以受信探针运行成功证明任意用户脚本已支持。后继861必须补齐公开入口、准入、真实OS输入和错误规则。

## 3. 技术栈

现有driver/AGP、固定旧AutoVM SDK、Rust与Python标准工具。新增拟定`src/research/game_runtime.rs`、`src/bin/autographics-game-probe.rs`、`research/game-runtime/*.at`、`tests/game_runtime_feasibility.rs`及`tools/check_game_feasibility.py`。这些是新路径，不是现有工具。不扩根workspace或Cargo依赖。

## 4. 需求分析与背景调查

用户2026-10-27要求规划下一阶段并估算M2剩余工作；本轮仅new，未授权当前实施/新agent/仓库/推送，无额外时间或token预算。进入依据为[M2设计](../design/autographics/m2-editor-and-public-game-prototypes.md)、[858研究Spec](../specs/auto-graphics/m2-game-sample-research.md)与[冻结G表](../design/autographics/m2-game-sample-matrix.md)。

已读[全局概览](../specs/overview.md)、[AG概览](../specs/auto-graphics/overview.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[运动](../specs/auto-graphics/bounded-motion-profile.md)、[Play](../specs/auto-graphics/scene-play-session.md)、[测量基线](../specs/auto-graphics/research-quality-baseline.md)。858已交付研究，但角色/资源转换/游戏预算均未实测；不能把13项203825B原始候选输入当最终游戏包。

[观察文件SHA](attachments/860/planning/source-observation.json.txt)绑定当前已交付图形`4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`，SDK=`7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`。`Session::budget`每slice1000/cumulative100000，单调用120resumes/250ms；Play保原30s init/2s tick/8ms pump。`ag_motion_normal`返回最后接触，不能直接推断最终地面；原Input只含x/y/toggle，没有跳跃动作边沿。857当前work引用只用于定位，不代替其review/merge。

## 5. 详细设计

### 5.1 前置与写面

硬前置857独立review、冻结合入门及实际交付。然后从该autographics-dev提交创建唯一`D:/autostack/.wt/lang-860/auto-lang`/`plan-860-dev`，只合图形分支；main仅簿记/沉淀。T-01核真实提交、旧接口/生成物、SDK/锁、857新AGP基线及测量/consumer工具。当前无dependency-ready实施任务。

允许写新增probe单元/fixture/检查器/当前来源清单、README及必要模块声明；生产输入/Host/Play/旧assembler/旧`.at`/SDK/锁/原资产不改。需要提取原测试fixture可采用只读复用，不复制第二套长期运行框架。不修改857错误比较器、测试或状态。

### 5.2 固定研究分母与候选

使用858原关卡/角色框、速度4、重力20、跳速8、dt=1/60；G2/G3坐标、三物品和终点位置保持。墙顶等抽象边界须转成明确静态碰撞体，T-02记录，不移动平台/物品或缩短坑距来过门。角色朝向/相机输入在本probe按固定数值输入，不认领相机或正式旋转writer。

最多两候选：A=现有v5调用组合；B=probe-only有限角色接触/支撑摘要，可把必要判定融合进原扫描结构，但不改v5文件/公共身份。必须测试“先碰地再碰墙”“切向离开平台后最终已无支撑”“顶碰撞”“两接触终止/舍入停止”，不能仅记上一帧grounded或只读最后法线。T-02写`attachments/860/work/probe-contract.md`，固定积分顺序、支撑/skin/容差、回调/输出边界、所有状态及上限，再跑结果；不能事后改expected。

原场景128节点、原CPU/调用时限、源码256KiB用户/64KiB managed/320KiB combined、32credits均不静默提高。B若使用不同内部摘要而不直接调用公共query，必须逐项报告扫描/状态/输出成本及与旧query的差异，不能“未调用query”就宣称已满足未来公共credit语义。候选go需在现有硬CPU/源码/场景界内，并形成可直接由861消费的明确有限资源合同；确需新profile预算/不同solver/SDK能力则requires_replan，不能自动放行。

### 5.3 有界运行与真实发布

负载：G2/G3代表路线、128wide/deep（两维静态场景，含最近阻挡体在末序/大量候选目标）、空输入、地面/墙/顶/边缘和坏参数/预算/取消。每路线最多1800逻辑步，保存可达/不可达实际轨迹、数值、状态和失败原因。用角色框与原半径触发区的独立几何判定确认曾到达每个目标范围；这只是路线可达证据，没有金币事件/胜负游戏实现。

头部采样先做真实Auto Session与完整输出准入；至少一条2D和3D正常轨迹接实际Host/既有模型呈现，核初始化/起跳/落地或受阻/Stop的最终合成像素与数值。探针输入回放是本阶段限定，不冒称OS键盘支持。CPU逻辑执行不可拆成多次函数调用绕过单callback预算；getters/准入/prepare/整步另行计量，完整帧不发布半份输出。坏形状/非有限/初始穿插与正常接触/PrecisionStop分别分类，不能让Fault伪装grounded。

采样使用853同条件方法，至少每类5次新进程、预热后30有效步，保存实际runner、输入/环境、CPU P50/P95、resumes/预算失败、源码字节、逻辑/实际状态内存、工作集、上传和owner；yield不等于精确steps，不能虚构GPU耗时或60Hz保证。轨迹记录每case至多1800条、总trace内存≤5MiB，和被测运行状态分开计量；20轮创建/运行/故障/Stop/关闭证明真实owner释放。

### 5.4 工具、决策和交接

T-03/04建立新工具约定`python tools/check_game_feasibility.py <fresh-output> --executable <retained-exe> --controls`，分头部数学/预算与Host/GPU验证，输出各case所走真实路径。真实变异至少包含stale-grounded造成空中跳和角落只认最后接触，及丢完整输出/漏实际画面；同oracle具名红、恢复正例绿。缺报告/partial/坏runner/0selected不能绿，沿用857实际修复后的证据设施，不复制已废弃比较器。

新增plan860 current identity/verifier并更新source/consumer入口，旧manifest不改；fresh own-SDK consumer编译并实际运行probe，无生产者路径/target。原生成物与profile/默认视图保持，受影响原门按共同合同验证。

最终`decision.json`记录选择/否决、原始证据、支持/拒绝集、积分/接触/grounded语义、资源上限、输入/输出/发布/owner、残余不可测项及861入口。研究可诚实交付requires_replan结论，但这不满足861前置；只在明确go且必要正负/资源证据齐备时允许861冻结实施基线。本Plan不修改M2分母来制造go。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-graphics/game-runtime-feasibility.md | 858提出未测缺口→绑定真实Auto探针/有限负载/可达与预算结果、选型和限制；明确非公共API交付 | 861有证据前置 | AC-02/03/04/05 |

## 6. 测试设计

实施自查与独立最终review同一树顺序进行，用户未指定模型或新agent；merge-owned完整面在最终review通过后执行。共同合同规定当前分母/consumer/产物恢复，不能把探针绿替代旧门。

| V | AC/不变量 | 输入/入口 | 区别错误实现的oracle | work任务/覆盖 |
|---|---|---|---|---|
| V-01 | AC-02/03 实际Auto | 新game_runtime_feasibility；dt/速度/g/跳速和G2/G3静态几何 | 实际Session一次角色更新；独立小场景f32/几何oracle及源hash；Rust替代执行不能算通过 | T-02/03；角色可行性非完整游戏 |
| V-02 | AC-02 接触/边缘 | 地+墙/顶、边缘离开、空中再次跳、初始重叠、PrecisionStop | 最终支撑与接触分类正确；错误stale-grounded/只认最后法线在同门红，Fault不变正常状态 | T-03/04；静态AABB、无坡/移动平台 |
| V-03 | AC-03 资源/路线 | 128wide/deep、末序最近体、原目标路线与1800步界、预算超限 | 同输入实际轨迹/目标距离、原hard limits、原始统计/所有者；不可达明确失败不改地图 | T-03/04；采样不承诺任意负载实时性 |
| V-04 | AC-04 发布/退出 | 正常Host2D/3D、丢输出/错误像素、partial、Fault/Stop/再Run20轮 | 数值AND合成实际内容，失败保last-good，真实弱owner/最后释放及fresh upload负控 | T-04；现有占位几何不是正式美术 |
| V-05 | AC-01/05 身份/交接 | current verifier/consumer、漏新文件/坏SDK/runner、恢复、decision | 错输入拒绝、fresh consumer实际运行；go所有必要证据闭合；requires_replan不会让861开工 | T-04/05；不证明新公共API准入 |

定向driver命令：`cargo check --locked`；新增`cargo test --locked --test game_runtime_feasibility -- --test-threads=1`；受影响原motion_host/geometry与新check工具（T-03/04创建）。不运行根整档；最终两端独立workspace完整测试、原/新native-owner-content与复用审计归merge。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件 |
|---|---|---|
| AC-01 | work | 857实际交付/唯一树/依赖/允许写面核对；source/consumer闭包无漏项 |
| AC-02 | work | 两候选以内、合同先冻结，最终支撑/墙顶/角落/边缘/空中跳语义与原错误分类有实测；V-01/02 |
| AC-03 | work | 原地图路线/128负载与支持界真实运行，原始性能/源码/内存和失败理由齐备；V-01/03 |
| AC-04 | work | Host/GPU数值与内容正反、预算/取消/last-good和最后owner/恢复成立；V-04 |
| AC-05 | work | fresh consumer实际运行，完整go/requires_replan决定可审；无虚构API/性能通过；V-05 |
| AC-06 | review | 独立核work AC/V/SD、路径/阈值/decision与warnings/fmt/遗漏；merge门pending |
| AC-07 | merge | 冻结两端完整/原新native-owner-content及受影响面通过，允许复用逐项证明 |
| AC-08 | merge | 源码只合autographics-dev，知识/ledger/归档/guard清理有实际收据，861只消费明确go |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 857交付 | 唯一树/实际提交/SDK/锁/857分母及测量工具核对，保留新阶段原始基线 | AC-01 | [✅] |
| T-02 | work | T-01 | probe-contract：G2/G3碰撞体/输入/积分/接触/状态与资源界；最多两候选和case oracle先冻结 | AC-02 | [✅] |
| T-03 | work | T-02 | 新research Auto/Rust probe/bin/命名tests；实际Session接触/路线/128/预算，V-01..03 | AC-02/03 | [✅] |
| T-04 | work | T-03 | 新Host/GPU check/真实变异/退出恢复/current identity/consumer，V-04/05 | AC-01/04/05 | [✅] |
| T-05 | work | T-04 | 五次进程/暖步采样、完整decision、定向收口/SD草案与提交，不跑完整档 | AC-01..05 | [✅] |
| T-06 | review | T-05 | 独立最终review含选择/否决理由；完整门pending | AC-06 | [✅] |
| T-07 | merge | T-06 pass | 冻结完整两端/原新native-owner-content及输入/产物复用审计 | AC-07 | [ ] |
| T-08 | merge | T-07 | 源码合autographics-dev，main沉淀SD/overview/plans/ADR/ledger/索引并归档 | AC-08 | [ ] |
| T-09 | merge | T-08 | guard clean清树/临时分支，保留原始产物/实际9/9；给861真实提交和decision | AC-08 | [✅] |

## 9. 复审记录

2026-10-27 new r1：stage=new；outcome=blocked（857尚在review/fix、未实际交付）；next=857正式交付后T-01核基线。全AC/V/T/SD未执行，本轮只读源码/研究与文档检查，无Cargo/native/性能、无新API实现。支持结论留给真实探针，不用研究输入数或理想公式代替。

2026-10-28 work T-01/T-02：用户授权实施860。857已于2026-10-28完成merge/交付/归档10/10（autographics-dev `4150a8bea`→`81b0974ac` ff-only，producer/consumer/fresh-consumer全量331+163+331与runner注册门全过，worktree已清理），前置满足。唯一树`D:/autostack/.wt/lang-860/auto-lang`、分支`plan-860-dev`自`81b0974ac`建立，`git rev-parse --show-toplevel`/`--show-current`核过；主检出WIP仅docs/plan簿记面（未修改crates/test）。SDK钉`7b987ca8b`在对象库验证、prepare_sdk PASS 681文件/17 local；两锁`2b131d3dd3`/`c3df168e`与858 source-observation逐字节一致；857基线工具（verify_plan857/plan857-identity/m2_evidence/runner_provenance/check_m2_baseline/m2-baseline bin）在树；driver `cargo check --locked` exit 0（RUST_MIN_STACK=16777216、RUSTFLAGS=/STACK:33554432）。v5接口事实核对：`ag_move_and_slide`两阶段/仅最后法线、status0-4、credits每tick 32（`_ag_cast_charge`，move 24/getter 1，超限fault 10）、Session 1000步/slice·100k累计·120resumes/250ms。[probe-contract](attachments/860/work/probe-contract.md)已冻结（含2026-10-28 r2修订：候选B=原样v5 move+同调用内几何支撑判定，放弃自管6迭代循环——任何结果运行前修订，expected/输入表/oracle不变；G2/G3显式碰撞体、共享控制器积分序、路线输入表及跳步{78,127,180,237}/{112,261}、SC-01..10/RT/128/HT/MUT-1..4与853采样约定）。executing 2/9；T-03未跑任何结果。

## 10. 待澄清事项

前置857交付。T-02局部机制选择在已授权研究边界内；若两候选都不能在原hard limits下满足必要语义，保留失败证据交付requires_replan并提出范围/资源/机制选择，不实施861或调高预算。新增正式物理库、SDK/执行环境或M2必需分母变化须先由用户决定。

2026-10-28 work T-03..05：`stage: work | plan_id: PLAN-860 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-860/auto-lang | branch: plan-860-dev | base_commit: 81b0974ac2512126790b0ef2af79235edf45b485 | code_commit: 83a503c92（4 提交：5ff0b9c6e probe+决策、9525c76c2/8c93270f4 provenance 参数化、83a503c92 最终证据绑定） | dependency_revisions: SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 不变，两锁不变 | task_ids: T-03,T-04,T-05 | evidence: 命名测试 20/20（tests/game_runtime_feasibility --test-threads=1，覆盖 V-01..05：G2/G3 双候选路线可达+敌/险/坑 oracle、SC-01..10 语义分类、MUT-1..4 真实源码变异具名红恢复绿、128wide/deep 末序阻挡体、预算/credits/fault 分类、Host/Play/GPU 发布与 Session 轨迹逐位绑定、Stop 像素恢复、20 轮 owner 释放、fresh spawn）；受控构建收据 origin_verified @8c93270f4（plan-artifacts/plan-860/controlled-build/）；40 进程采样矩阵（4 类×2 候选×5 进程，外部 10ms 工作集）全部过 check_game_feasibility（--controls 负控 PASS）；fresh consumer 自有 SDK 离线编译并实跑 g2-a/g3-b 检查器过；verify_plan860 407 文件 --controls PASS；decision=go（research/game-runtime/decision.json，含成本表/限制/861 入口）；SD-01 草案 docs/specs/auto-graphics/game-runtime-feasibility.md 已备。合同修订 r2/r3/r4 均在任何被接受结果之前（见 attachments/860/work/probe-contract.md 修订记录） | blockers: 无 work 内阻塞 | next_responsible_role: 独立最终 review（T-06）`。merge-owned pending：两端独立 workspace 完整 `cargo test --locked -- --test-threads=1`、AGP 受影响面与原新 native-owner-content 复用审计、`/auto-plan:regress` 批量回归档、spec/ledger 沉淀与归档 guard 清理——不能以本记录代替。G3 默认 3D 相机下像素哈希不随检查点变化（3D 发布以数值逐位+恢复严格性绑定，可见运动像素判别在 G2 2D 面证明）已记入 decision 限制；工作树 clean，主检出 WIP 仅本簿记。

2026-10-28 independent final R1：`stage: review | review_scope: final | reviewer_identity: 实施同会话/同模型（局限已声明），结论从工件与冻结提交上的新执行重建，不采信实施摘要 | plan_id: PLAN-860 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-860/auto-lang | branch: plan-860-dev | reviewed_commit: 7144526c3 | base_commit: 81b0974ac | dependency_revisions: SDK 7b987ca8b 不变、两锁不变 | spec_inputs: SD-01 草案 docs/specs/auto-graphics/game-runtime-feasibility.md（worktree 准备，merge 发布）+ probe-contract r4 冻结文本（attachments/860/work/probe-contract.md） | acceptance_results: AC-01..05 全 PASS——写面审计 21 文件无任何 api/authoring/AGP/SDK/预算改动、lib.rs 纯增量只读访问器；24 项合同符合性检查（跳步表/相位/clearance/出生点/冻结规则逐字/B 单元 charge 与状态界）；20/20 命名测试在冻结提交重跑；verify_plan860 407 文件 --controls PASS；checker --controls PASS；m2_evidence --self-check PASS；40 进程采样报告全 PASS 且绑定受控构建收据（origin_verified @83a503c92，runner 4276a91c）；fresh consumer 报告 PASS；独立运动学 oracle（复审自建：连续性/速度增量守恒/跳步上升/合同落点接地/敌/险/坑正确盒膨胀/触发圆独立重推）对全新 g2-a/g3-b 原始轨迹 PASS；变异断言抽查 13/13（6 处初判 MISSING 经查为 rustfmt 换行伪差异）；decision 一致性 12/12（go 判据=合同§9） | pending_closeout_gates: 两端独立 workspace 完整 cargo test、AGP 受影响面与原新 native-owner-content 复用审计、批量回归 tf 到期判定、spec/ledger 沉淀、归档与 guard 清理（均 merge-owned，T-07..09） | findings: F-860-R1-01（已闭合）runner-receipt.json 曾被重采样清理误删致 decision 引用悬空——复审在冻结提交重建受控证明、以证明过的 runner 重跑 40 进程采样并重记严格收据，提交 7144526c3 重新绑定；独立 oracle 三轮迭代均为复审探针自身错误（自由落体界/盒膨胀侧/种子），按规程保留并分类，非产品缺陷。N-1（非阻塞，合同文本对齐）合同§8 MUT-3 原文"Host 发布丢最终位移写回"与实现形式（用户源积分输出丢弃，被冻结轨迹 numeric oracle 捕获）不同——计划§5.4 原文"丢完整输出"已满足，Host 级写回抑制需改生产代码故不可达；建议 work 侧补合同 r5 修订注记（纯 md）。N-2（非阻塞，覆盖边界）"20轮…故障…"的故障腿在 Session 级证明（sc09 关闭 VM + lib terminal_policy_tests 弱引用故障释放），Host 20 轮为 run/stop 干净循环——组合覆盖成立，循环内注入故障的变体未实现 | evidence: plan-artifacts/plan-860/{sampling,review,work-t04,controlled-build}（原始报告/收据/证明/日志；sampling-summary 与 runner-receipt 为复核锚点） | next: merge（T-07..09）；在 merge 冻结落地提交上按 857 T-08 先例执行完整门

2026-10-28 merge T-07..08 delivered：`stage: merge | plan_id: PLAN-860 | plan_revision: 1 | completion_kind: delivered | reviewed_commit: 7144526c3 | delivery_commit: 7144526c3 | autographics-dev: 81b0974ac -> 7144526c3 (fetch ff-only，零 rebase，哈希与复审证据逐位一致) | gates: producer 全量 351/351 exit 0（plan-artifacts/plan-860/merge/producer-full.log）；consumer 全量 351/351 exit 0（merge/consumer-full.log）——首轮红两次均归因留痕：①consumer 导出缺 git 跟踪二进制/非身份夹具（two-parts.glb、authoring-probe/main.at、research 白名单 .txt；package_consumer 历史缺口，证据侧补齐逐字节一致副本）②play839-files-<pid> 临时目录跨运行残留+Windows PID 复用致 count==1 断言环境偶发，隔离重放 7/7 绿后清残留单次全绿；AGP 零改动无受影响面；root 工作区零改动无根档门 | canonical_spec: docs/specs/auto-graphics/game-runtime-feasibility.md + overview/plans 索引（503181cfe） | ledger: .autoos/specs.json v111->112，P860-GAME-RUNTIME-FEASIBILITY/P860-TESTS/P860-REVIEW-R1/P860-MERGE 四条目（file/tags 齐）；首次投影意外全文件重排（indent 1+CRLF/LF）当场发现并 amend 修正，最终提交 diff +99/−5 外科式（52f1184 主检出现 tip），通道=guarded offline projection（无 store writer 可达；备份 specs.json.bak-p860-merge） | archive: 本文件 status: archived | next: T-09 清理与到期判定`。

2026-10-28 merge T-09 cleaned：`stage: merge | outcome: pass | completion_kind: delivered | production_artifacts: 本计划为 experimental/autographics 研究线，root 工作区零改动，无生产后端/发布包消费面，无 stale 产物义务（观察项） | batch_regression: 860%5==0 晚于 859 收据（last_covered=859 @2026-10-28T05:55Z）-> 到期，任务 9/9；清理实绩：wt-guard clean（无 reparse point）后 worktree 移除、plan-860-dev 删除（tip 7144526c3 经 --is-ancestor 验证已在 autographics-dev 后 -D）、组目录 lang-860 移除；plan-artifacts/plan-860 回收 4 棵 CACHEDIR 确认的 target 树与散落 .pdb，26610MB->398MB，保留 runners/（4276a91c/436d18b1 等锚点）与全部日志/收据/证明/冻结源`。
