---
plan_id: PLAN-1117
status: archived
feature_name: "ACE A2R 接管 v0.5 入口与替换验收"
author: [Codex]
created_at: 2026-11-20
updated_at: 2026-11-22
plan_revision: 1
current_step: 5
total_steps: 6
depends_on: [PLAN-1114, PLAN-1115, PLAN-1116]
supersedes_spec_components: [docs/specs/auto-a2r/project.md, docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-cli/project.md, docs/specs/ace-migration/m2-build.md]
new_spec_components: [docs/specs/auto-a2r/legacy-replacement.md]
touched_goals: [GOAL-003]
affects: [auto-a2r, auto-cli, ace-migration, auto-test-case]
---

# [PLAN-1117] ACE A2R 接管 v0.5 入口与替换验收

## 0. 变更摘要

交 `IF-ACE-A2R-V05-REPLACEMENT/1`，用原用户操作在候选分发选择 ACE A2R。产品默认/安装及旧实现退役分别由 986/987 执行。 各批依赖和交接见 [Design 60](../design/60-ace-rust-route.md)。

## 1. 目标

冻结 v0.5 已支持的 A2R 用户入口、产物和依赖行为，实现原操作映射到 ACE A2R，验收构建、运行、升级与回滚。沿 985/B → 986 → 987 完成正式默认切换和退役。

## 2. 架构方案

旧公开操作 → 现役 CLI/AutoMan 适配层 → 显式选择 ACE A2R → 1115/1116 → Rust/Cargo 产物或预编译 SDK。每个操作绑定原规范/源码、当前工件和独立 oracle，旧版对照与回滚单独发行。

## 3. 技术栈

`crates/auto-lang-core/src/trans/rust.rs` 及 `trans/rust/project_fixups.rs`（历史来源）、`crates/a2r-std`、`crates/auto-lang-core/src/test_cases/a2r.rs`、ACE `ac-build`、现役 CLI 和 AutoMan 构建入口（跨仓由原 owner 维护）。

## 4. 需求分析与背景调查

[242](242-a2r-feature-gap-tracker.md)提供历史盘点线索。真实 CLI/包导出、[旧 A2R 金样](../specs/auto-test-case/a2r-golden-certification.md)、历史行为认证 [992](archive/992-legacy-a2r-seventeen-value-certification.md)/[994](archive/994-legacy-a2r-text-certification-repair.md)/[996](archive/996-legacy-a2r-finite-list-certification.md)/[997](archive/997-legacy-a2r-checked-i32-trap-certification.md)/[998](archive/998-legacy-a2r-resource-scenario-certification.md) 与 [迁移 Spec](../specs/ace-migration/replacement-scope-and-evidence.md) 共同取材。[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)汇总三端。[985](985-ace-m2-certification.md)/[986](986-ace-m2-release.md)/[987](987-ace-m2-retirement.md)分别负责候选、安装默认和整体退役。

本轮取材基线为主检出 `4438d4933baa09ee4c6727111e2de07ff6cabfa0` 及活动 Plan 工作区文本。用户授权检查 AV 路线并规划 Rust 路线，顺序为 AV/RS（含 `*.rs.at`）→ A2R 替换 v0.5。本次授权仅含语言仓路线和计划文档。实施前按 AGENTS.md 确认计划与实际依赖版本。

**执行授权（2026-11-22）**：用户 `/goal 完成计划1117` 授权整链交付（work→review→merge），解除 2026-11-20 §9 的 work 未授权阻塞。worktree `D:/autostack/.wt/lang-1117/auto-lang`、分支 `plan-1117-dev`、基提交 `821fa2424`（1116 归档后 master）；依赖 1114/1115/1116 均已归档。§10 待决项按 §5 边界处理：本包只交语言仓可选新路由与候选证据，公开承诺破坏与兄弟仓接线不在此授权内。

## 5. 详细设计

按命令/选项、包和依赖锁、外部 Rust API、生成 Cargo 布局、可运行产物、失败行为和领域消费者冻结分母。逐项登记旧请求→新请求、可观察输出/诊断、目标入口、独立 expected、适用平台和恢复方式。既有命令在兼容范围内沿原拼写，全文金样用于追溯旧实现；公开 API/产物布局属已承诺接口时，用真实 Rust consumer 验证兼容。

本包交语言仓可选的新路由和候选证据；AutoMan 等兄弟仓按现有 owner 接线。1117 最终逐项核对 1096 的适用正式结果、1068 矩阵的适用输入及各领域正式来源。UI/AGE 接 1074、1103、1076/1077 的真编译、真业务、宿主/包。1068 从 1115/1116 和领域生产者独立接收，可与本包并行。985/B 接 1068 三端门和本包替换证据，986/987 在正式发行件完成默认与退役。

### 规范增量

| ID | 操作 | 规范目标 | 交付后规则与理由 | 验收 |
|---|---|---|---|---|
| SD-01 | 新增 | `docs/specs/auto-a2r/legacy-replacement.md` | 新增旧→新公开操作与支持、回滚和逐行证据 | AC-01/02/03 |
| SD-02 | 修改 | `docs/specs/auto-a2r/project.md`、`docs/specs/auto-cli/project.md` | 记录可选接管的实际命令与能力边界 | AC-02 |
| SD-03 | 修改 | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 登记发行衔接、差额和验收身份 | AC-01/03 |
| SD-04 | 修改 | `docs/specs/ace-migration/m2-build.md` | a2r 车道语言通道 delta 注记（复审 F-1117-R2 补登记） | AC-02 |

## 6. 测试设计

以下为拟执行验证。各任务先定位现有同义 Case 或实现正式入口和必要反例，再记录准确命令、独立预期、实际阶段和产物身份。公共语料采用 1078 的唯一特性源。平台及条件执行范围见 [Design 60](../design/60-ace-rust-route.md)。

| Case | 验收 / work 任务 | 真实入口和区别性断言 |
|---|---|---|
| V-01 | AC-01/02 / T-01/02 | 同一老项目走现役构建/导出动作并选候选，真实生成、运行并由仓外 Cargo consumer 实编实调；核固定值、错误与数据，实际 legacy emitter/AutoVM dispatch 为零。 |
| V-02 | AC-02/03 / T-03/04 | 干净候选目录中生成/编译及运行预编译产物，按 manifest 验工具依赖。错工件/升级失败时原包和用户数据保持，独立 v0.5 回滚可重演；再选候选走新 A2R。 |
| V-03 | AC-03 / T-04 | 删 required 行、用 emit-only 替代真实运行、错工件、标 ACE 实际 dispatch legacy，接收门均拒绝。 |

`cargo t ace-a2x ace-m2-build a2r a2r-compile a2r-std`，补实际受影响 legacy Case 固定组和 CLI/工程组。显式 build-lane 核真实产物，领域按原 owner 收有效证据。986/987 用各自冻结发行件验证安装默认和生产旧链不可达。 完整受影响固定组合并入最终复审后的冻结验证，记录应跑、实跑、原条件排除和未跑数，以及准备/执行/清理时间。独立 review 查实际源码、Case 和真实生产入口。

## 7. 验收标准

- [x] **AC-01** · owner_stage=`work`。原 required 入口、产物和行为一一对账，差异、归属与平台明确；本版适用 required 行无未验。（SD-01 `docs/specs/auto-a2r/legacy-replacement.md` §1 E-01..E-10 + §3 逐项映射：差异逐条具名（while/scrutinee/i64-checked/包车道），归属与平台（Windows/MSVC 首平台，Linux 沿 1063）明确。）
- [x] **AC-02** · owner_stage=`work`。现役用户入口可选择候选并由新 A2R 真实生成、编译和运行，兼容性由实际消费者验证。（`AUTO_ACE_RUST=ace` 绑定的渲染器（`transpile_scalar_source`）与 m2-build A2R 车道改走 1115/1116 语言通道，提交 0daef20f9；V-01：同老项目 legacy=39 / 候选 build→SDK→run=value:39 / 仓外 Cargo consumer `#[path]` 原样编入实调 fn_main()=consumer:39；trap:Overflow 错误腿。）
- [x] **AC-03** · owner_stage=`work`。V-02/V-03 证实恢复/回滚和接收门敏感性，三项发行计划可消费版本绑定的接口。（V-02：错工件/失败升级下原包+用户数据保持、restore 读回旧根、再选候选真实重建 value:40 走语言通道；V-03：接收门四负控（删 required 行/emit-only/错工件/标 ACE 实 legacy）具名拒，提交 451cdb0be；`IF-ACE-A2R-V05-REPLACEMENT/1` 回执 schema `auto.a2r.v05-replacement/1` + 签发样本已验。）
- [x] **AC-04** · owner_stage=`review`。逐项独立复验 AC-01–03、缺口归属及 Spec 增量，结论和证据绑定合同版本及提交。（独立复审代理 final pass @5a04adee6，修复后绑定 0789cdfac；§9 收据。）
- [x] **AC-05** · owner_stage=`merge`。冻结门通过后落主、沉淀已审规范与接口、归档及清理；核发布给实际消费者的来源身份。（§9 merge 收据；消费者来源身份=SDK manifest 工具轴（rustc 身份/msvc-crt）+语言通道 schema+回执哈希绑定；兄弟仓 CLI 消费归 auto-man owner=D-1117-1。）

## 8. 执行步骤

实施入口为 `D:/autostack/.wt/lang-1117/auto-lang`、分支 `plan-1117-dev`，基于实施时主检出的正式输入。按需在当前工具中登记 T 项再开工；首任务 T-01 的输入和待决项见 §4、§5、§10。材料统一放 `D:/autostack/plan-artifacts/plan-1117/` 并登记。

- [x] T-01 冻结旧 A2R 公开分母
  - owner_stage=`work`。核实际 auto/man/SDK 调用链、旧用例与差异，登记用户入口、版本承诺、目标及 owner（AC-01）。
  - **[✅ 2026-11-22]** 分母冻结于 SD-01 `docs/specs/auto-a2r/legacy-replacement.md`（worktree 451cdb0be）：入口 E-01..E-10（CLI 命令面在 auto-man、API/产物面在本仓）、产物/依赖承诺（.a2r.rs/Cargo 布局/a2r-std/四 profile id）、逐项旧→新映射与具名差异；v0.5=发布名（代码侧唯一版本身份=legacy-a2r.*/1）。多模块包车道与兄弟仓绕过孔登记为 D-1117-1..4。
- [x] T-02 接入现役命令的 ACE A2R 选择
  - owner_stage=`work`。让真实用户入口消费 1116；V-01 验现役工程、Cargo 导出和仓外消费者（AC-02）。
  - **[✅ 2026-11-22，0daef20f9]** 宿主桥 `transpile_scalar_source`（982 开关绑定的渲染器）与 m2-build A2R 车道改走语言通道；V-01 三观察同值 39（legacy emit+852 wrapper consumer / 候选 build→SDK→run / 仓外 consumer 实调 fn_main）+ trap:Overflow；证据 plan-artifacts/plan-1117/v01/V-01-RECEIPT.md。新增固定用例：`p1117_rust_language_route`（4 例 <0.1s）、`a2r_language_lane_1117`（0.79s 真 rustc）。
- [x] T-03 验证兼容、数据与回滚
  - owner_stage=`work`。V-02 重演现役动作与失败恢复；收所需 1074/1103/1076/1077 等正式证据（AC-02/03）。
  - **[✅ 2026-11-22]** V-02 全链绿（plan-artifacts/plan-1117/v02/V-02-RECEIPT.md）：干净候选目录/manifest 工具依赖（msvc-crt+rustc）/错工件具名拒/失败升级下原包+用户数据保持/restore 回滚重演（CURRENT live）/再选候选真实重建 value:40 语言通道。领域行（UI/AGE/web）按原 owner 既有正式证据登记（983 B 批、980 系列已落；1074 W3/1103/1076/1077 以各自归档件为准），本包不重复执行其套件。
- [x] T-04 签发替换候选接口
  - owner_stage=`work`。V-03 核实际生产和接收负控；给 985/B、986、987 交 IF-ACE-A2R-V05-REPLACEMENT/1，三端结果入 1068 原矩阵（AC-01–03）。
  - **[✅ 2026-11-22，451cdb0be]** 接收门 `ace-m2-build::receipt`（schema `auto.a2r.v05-replacement/1`，四 required 行哈希绑定候选 SDK）；`a2r_replacement_receipt_1117`（1.16s）真回执过 + 四负控具名拒；签发样本 v04/v05-replacement-receipt.sample.json（门内验证后落盘）。接口=spec §6+门+样本，985/B、986、987 可消费；1068 三端行的收录归 1068 owner（本包以 SD-03 挂接登记）。
- [x] T-05 独立复审
  - owner_stage=`review`。核 AC-01–04，回放关键正反例、实际入口和结果，记录遗漏、延期及替代实现，绑定 revision 和提交。
  - **[✅ 2026-11-22]** 独立上下文复审代理（新会话、零实施参与，从工件重建：git diff 全读、锚点核码、测试重放、plan-artifacts 核对）final **pass**：AC-01..04 全 pass；3 个 minor 发现 F-1117-R1（spec 模块计数 21→19 pub mod）、F-1117-R2（计划元数据补 m2-build.md，SD-04 行）、F-1117-R3（V-01 profile id 观察通道注记）已修复并定向复验（0789cdfac，代码零漂移）。新增用例计时 0.08–1.44s（≤10s 门全过）；候选路由依赖级零 legacy 可达（grep+依赖图核）；无新增告警。
- [x] T-06 验证冻结交付与归档
  - owner_stage=`merge`。执行冻结受影响组，核 AC-05，落主并交接，折叠证据后按 guard 清理并完成路径对账。
  - **[✅ 2026-11-22]** 冻结并集门（worktree）：准备=3 invocations（fixed-tests prepare-only，exit 0）→ 执行 `ace-a2x ace-m2-build a2r a2r-compile a2r-std ace-facade`（5m52.9s 墙钟）：ace-a2x PASS 347/347；ace-m2-build 首轮红（identity_cache a2r 断言=车道 session-less 真实后果，**merge 修复** cdb7d9ce2 重锚后新 tip 复跑 PASS）；a2r-std PASS 115/115；a2r-compile=条件排除（单例 #[ignore] 显式门未强选，PLAN-1066 owner）；a2r=预存 infra 红（vm-cache-inputs 发现失败，PLAN-1108/DEBT-1098 族，当日批量收据已载）；ace-facade 99/100（唯一红 p969_t02_console=**预存** D-1098-01 具名债，master 基线同红复验）；另 p978_av_warm_build_p90_bounded 为负载敏感计时 flake（串行双绿）。落主：rebase master（他仓 Plan 108 spec 已前移）range-diff 5/5 全等（0daef20f9→bd5182483、451cdb0be→af9b272a4、5a04adee6→de4103977、0789cdfac→35590c89a、cdb7d9ce2→937851ead），ff-only 落主，master tip=交付提交 **937851ead**。canonical_verified：legacy-replacement.md 7401e7ce16b3ea35、auto-a2r/project.md 737434049eddd11f、auto-cli/project.md eb4353f03ecb2319、replacement-scope-and-evidence.md d8504be0e8f79828、m2-build.md 315b3ecfc9af6073（主检出与审阅 delta 一致）。批量回归 DUE（1115%5==0 落主 > last_covered 1114）→ regress 技能交接（主检出单实例，非本 goal 内执行）。

## 9. 复审记录

2026-11-20，stage=new，PLAN-1117 r1。计划已起草，实施确认和 §4–5 所列前置交付待办。outcome=blocked（work 尚未授权），next=确认后进入 T-01。T-01–06、AC-01–05 均待执行。

2026-11-22，stage=work（goal 整链授权 `/goal 完成计划1117`，§4 已记录）。T-01..T-04 全部交付：worktree `D:/autostack/.wt/lang-1117/auto-lang` 分支 plan-1117-dev，base 821fa2424，commits 0daef20f9（候选路由语言通道：宿主桥+m2-build 车道+两测试文件）→ 451cdb0be（接收门 receipt 模块+负控测试+SD-01 spec）→ 5a04adee6（SD-02/SD-03 spec 增量）。AC-01..03（work-owned）完成，证据见 §8 各任务行与 plan-artifacts/plan-1117/{v01,v02,v04} 收据。

**work handoff receipt**：stage: work | plan_id: PLAN-1117 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1117/auto-lang | branch: plan-1117-dev | base_commit: 821fa2424 | code_commit: 5a04adee6 | dependency_revisions: 1114/1115/1116 archived（语言通道+build lane+SDK 按 1116 冻结身份消费）| task_ids: T-01..T-04 | evidence: V-01 三观察 39+trap:Overflow；V-02 失败隔离/回滚重演/再选候选 40；V-03 四负控具名拒+签发样本；桥 crate 31 测试文件绿；m2-build a2r/coverage/cli 绿 | blockers: 无 | 遗留具名债: D-1117-1 兄弟仓绕过孔（auto-man owner）、D-1117-2 多模块语言面包车道、D-1117-3 语言发射 session-less、D-1117-4 legacy 义务不减（987 退役前）| 本会话测试台账: 见 §7 验证记录表 | 冻结并集门（`ace-a2x ace-m2-build a2r a2r-compile a2r-std case-a2r-language-parity` 及 CLI/工程组）: pending，owner=final review/merge | next_responsible_role: review（T-05）

2026-11-22，stage=goal round 1。work 阶段完成（上条收据）。outcome=pass，next=review（T-05 独立复审）。

stage: goal | plan_id: PLAN-1117 | plan_revision: r1 | round: 1 | from: work | to: review | status: executing→execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1117/auto-lang | commit: 5a04adee6 | next: review(T-05)

2026-11-22，stage=review（T-05 final）。reviewer=独立复审代理（新上下文/零实施参与；结论全部从工件重建）。AC-01 pass（E-01..E-10 锚点逐条核码、具名差异核到 parser/resolve 行号）；AC-02 pass（语言臂/钩子/装配全读，a2c/a2py 不动，1116 车道回归绿）；AC-03 pass（接收门 8 具名拒实装、四负控各断言具体 v05.* 码）；AC-04 本审。findings=F-1117-R1/R2/R3（全 minor/文档面）→ 同日有界修复（0789cdfac：spec 计数；计划元数据 SD-04；收据注记）→ 定向复验闭合，代码零漂移。新增用例计时：p1117×4（0.08–0.14s）、a2r_language_lane_1117（1.07s）、a2r_replacement_receipt_1117（1.33s）、a2r_project_1116 复用+重放（1.44s）——≤10s 门全过；无新告警；worktree clean。冻结并集门（ace-a2x ace-m2-build a2r a2r-compile a2r-std case-a2r-language-parity 及 CLI/工程组）= pending，owner=merge（T-06）。outcome=**pass（final）**，status=reviewed，reviewed_commit=0789cdfac。

stage: goal | plan_id: PLAN-1117 | plan_revision: r1 | round: 1 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1117/auto-lang | commit: 0789cdfac | next: merge(T-06)

stage: merge | plan_id: PLAN-1117 | plan_revision: r1 | outcome: pass | completion_kind: delivered | prepared: reviewed pass（0789cdfac，r1）+ 冻结 Spec delta | 冻结门: 上述 T-06 记录（ace-m2-build 修复后 PASS、两个预存红具名不充 PASS）| landed: ff-only，master tip=937851ead=delivery commit（rebase 旧→新映射与 range-diff 见 T-06）| canonical_verified: 五文件哈希见 T-06（主检出=审阅 delta）| batch_regression: DUE → regress 交接 | cleaned: 见下条收据 | delivery_commit: 937851ead

## 10. 待澄清事项

（2026-11-22 更新）T-01 原"识别 v0.5 生成 API/布局与依赖行为"已闭（SD-01 §1/§3：v0.5=发布名，代码身份=四 profile id + 候选三件套 fileset）。公开承诺破坏：无——候选为增量可选路由，旧布局/行为逐字节不变（986 发行默认切换时再裁定）。兄弟仓入口：D-1117-1（auto-man `auto trans -t rust` 无 merge 臂绕过开关）由 auto-man owner 接线出证；985/B、986、987 消费 §6 接收门与签发样本。
