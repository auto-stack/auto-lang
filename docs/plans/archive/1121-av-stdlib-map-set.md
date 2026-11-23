---
plan_id: PLAN-1121
status: archived
completion_kind: delivered
feature_name: "AV 标准库 HashMap 与 HashSet 迁移"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-23
plan_revision: 1
current_step: 5
total_steps: 5
depends_on: [PLAN-972, PLAN-1084, PLAN-1088, PLAN-1096]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/stdlib/design/ace-collections.md]
new_spec_components: []
touched_goals: [GOAL-001, GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1121] AV 标准库 HashMap 与 HashSet 迁移

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](1096-av-parity-full-language-certification.md)。

## 1. 目标

在 Windows AV 普通工程交付受检 Map/Set 操作和旧 `HashMap`、`HashSet` 的可用迁移入口。首批覆盖旧用例实际使用的字符串键、字符串/整数值、构造、查询、覆盖/去重、数量、删除和清空。

## 2. 架构方案

公共 Auto 容器接口 → 共享类型实例与所有权检查 → AV 集合存储。先核现有整数 map 和普通泛型记录、拥有值可支撑的底座，复用公共语义。键的相等、散列、缺键和引用规则由公共容器合同给出。

## 3. 技术栈

`stdlib/ace/asm/map.at`、`stdlib/auto/{hashmap,hashset}.at`、`experimental/ace/crates/core/src/source/resolve.rs`、普通集合 AutoDown 语料。

拟实施树为 `D:/autostack/.wt/lang-1121/auto-lang`，分支 `plan-1121-dev`。

## 4. 需求分析与背景调查

[核心标准库](../../specs/ace-migration/m2-stdlib-core.md)当前为 `List<int>` 上的整数 map，[1084](1084-av-parity-dynamic-objects-and-null.md)把 typed 容器留给标准库。原 [迁移表](../../../crates/auto-lang/test/cases/language/migration/legacy-map.tsv)中 `legacy.13_collections.001–013` 和 `legacy.21_conv.003–004` 为首批；已交付的整数 map、Dyn 对象和其各自行为继续以原用例观察。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

T-01 用字符串键、局部 owned 字符串值与取值后删除验证现有存储和所有权可行性，确定构造名及 Type/方法回写、键域、alias 规则、缺键返回值和兼容层拼写。原用例中两个删除失败（13.006、13.011）须定位失败原因，保留原记录，按获准后的容器合同分别断言成功删除和缺键。迭代顺序只在规格已承诺时固定。库以纯 Auto 可行路线优先，新增通用容器机制须经 T-01 举证。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-core.md`](../../specs/ace-migration/m2-stdlib-core.md) | 记录首批公共操作、兼容拼写、已验键值域 | AC-01/02 |
| SD-02 | modify | [`docs/specs/stdlib/design/ace-collections.md`](../../specs/stdlib/design/ace-collections.md) | 补充容器资源、键相等、替换/清空和缺键行为 | AC-01/02 |

## 6. 测试设计

用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。执行结果见 §8/§9。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 旧 `legacy.21_conv.003/004` + Map/Set 普通工程 | 输出按原可比预期分别为 `age=30 score=100 has=1 size=1` 与 `apple=1 cherry=0 size=1`，只按显式兼容规则处理展示差异。 |
| V-02 | AC-01/02 · T-02 | 相同字符串键覆盖；不同键发生散列冲突时保留独立键；Set 重复插入 | 覆盖不增加大小，冲突键各得原值，集合重复只计一次，错键值类型在适用的静态入口具名拒绝。 |
| V-03 | AC-02 · T-03 | 删除、清空、缺键、最后 owner 与旧删除红例 | 大小变化、缺键协议、未删另一条、原 owner 存活及最后释放可观察；旧原例按迁移表分类保留。 |

冻结验证命令为 `cargo t ace-stdlib ace-source ace-semantics ace-lowering ace-av case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](../1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [x] AC-01 · owner_stage=work。在普通 AV 包中运行 V-01 和容器近邻，旧 Map/Set 检查逐行承接。（2026-11-23：av.parity.1121.map-set 26 例；旧 13 行 + 2 conv 行逐行绑定，详见 §8 证据）
- [x] AC-02 · owner_stage=work。V-02/03 的替换、冲突/去重、缺键、生命周期有独立观察，整数 map 和 Dyn 不回归。（2026-11-23：覆盖 9922/去重 2/缺键 71/释放对账/错型 verify.type-mismatch/面冲突与未知面绑定拒；整数 List graft + Dyn 近邻 302）
- [x] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。见 §9 T-04 复审及会话限制记录。
- [x] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。见 §9 T-05 合入及清理记录。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [x] T-01 核定 Map/Set 接口与存储可行性
  - owner_stage=work。调查 15 条原用例和现有 ACE 可表达容器。在普通包中跑字符串键和 owned 值探针，先确定接口、容器语义、底座及旧失败归类。
  - [✅ 已完成 2026-11-23] 纯 Auto 路线可行：非泛型记录 + 并行 List 底座（keys/tags/ivals/svals；items），for-条件扫描，put=owned Str+.move，get_str=元素 clone，缺键/错面=fail 受检通道。原例 002/003/21_conv.003/004 期望逐字复现（conv 用 bool→int shim，`has=1`）。关键具名拒：auto 面无用户泛型/无方法间调用/无 while；单元方法须显式 void；错值型在 Verify:verify.type-mismatch 拒；FStr bool 具名拒。var 赋值记录=move，形参共享变更可见。证据 D:/autostack/plan-artifacts/plan-1121/t01-probe-evidence.md。
- [x] T-02 实现容器和普通工程入口
  - owner_stage=work。依赖 T-01。交付公共库、必要的受检接线与旧兼容入口，运行 V-01/02。
  - [✅ 已完成 2026-11-23] 交付 `stdlib/ace/asm/map_str.at`（纯 Auto 记录容器，非装配集）+ adapter 普通面嫁接（用户自声明同名类型优先）+ resolve 构造回退与拼写回写（insert*/get* 按接收者局部变量静态值面分派，int 默认面；drop=使用结束标记）。`av.parity.1121.map-set` 26 例全绿（整套件 0.74s，单例均 <10s）；V-01 旧行逐字承接（002 `[Alice, Wonderland]`、003 `67`、conv003/004 原 oracle 逐字），006/011 两条旧删除红例按容器合同重归类为成功删除。
- [x] T-03 补旧用例映射和生命周期证据
  - owner_stage=work。依赖 T-02。运行 V-03、实际整数 map 与 Dyn 近邻，登记各旧行。
  - [✅ 已完成 2026-11-23] V-03 证据：缺键 get=受检 KeyError（catch 后 owner 可续用 71）；remove 未命中=false 且不波及其他（13）；错面 get=绑定期 source.map-face-mismatch（静态拒，graft 内 ValueError 为纵深）；释放对账（arena alloc=release、live=0）；整数 List graft 与 Dyn 对象近邻不回归（302）。15 条旧行 + 2 条 conv 行已登记 legacy-map.tsv（PLAN-1121 行组，canonical=av.parity.1121.map-set）。实际整数 map（asm/map.at 装配线）与 StrBldr 属 972 装配面，由 ace-stdlib 固定组在 T-05 冻结并集覆盖（本面不重复建入口）。SD-01/SD-02 规范增量已写入 worktree 两 spec。
- [x] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [x] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1121 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

2026-11-23 · stage: work | plan_id: PLAN-1121 | plan_revision: r1 | outcome: pass（T-01 完成）| worktree: D:/autostack/.wt/lang-1121/auto-lang | branch: plan-1121-dev | base_commit: 6d3bd9967 | code_commit: （未提交，T-02 一并提交）| task_ids: T-01 | evidence: D:/autostack/plan-artifacts/plan-1121/t01-probe-evidence.md | blockers: 无 | next: T-02

stage: goal | plan_id: PLAN-1121 | plan_revision: r1 | round: 1 | from: work | to: work | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1121/auto-lang | commit: 6d3bd9967 | next: work

## 10. 待澄清事项

StrBldr 仍沿现有文本和容器台账。T-01 的容器底座决策先于新增共享布局。

2026-11-23 · stage: work | plan_id: PLAN-1121 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-1121/auto-lang | branch: plan-1121-dev | base_commit: 6d3bd9967 | code_commit: （见下） | dependency_revisions: auto-down 组内 detached@456c86a（未改动） | task_ids: T-01/T-02/T-03 | evidence: D:/autostack/plan-artifacts/plan-1121/t01-probe-evidence.md + 本 §9 | blockers: 无 | next: T-04 review

stage: goal | plan_id: PLAN-1121 | plan_revision: r1 | round: 2 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1121/auto-lang | commit: （T-02/T-03 checkpoint 提交后无新代码改动） | next: review

### work 交接就绪记录（2026-11-23）

- **work-owned AC 结果**：AC-01/AC-02 全项通过（§8 各任务证据行）；AC-03=review、AC-04=merge，未由 work 标记。
- **交付提交**：`feat(auto-av): AV stdlib HashMap/HashSet migration face over grafted record containers (Plan 1121)`（10 文件，+1282/-35；worktree plan-1121-dev）。
- **新增用例**：`av.parity.1121.map-set` 26 例（plan1121_map_set.rs），整套件 0.74s、单例均 <1s（≤10s 纪律）；行为与计时：V-01 旧行承接 8 例、V-02 5 例、V-03/近邻/负控 13 例。
- **受影响固定组并集**（worktree 冻结提交，prepared manifest 26656016f9d8478e，run-8_3o0bip）：
  - ace-av：1627 执行 / 1614 过 / 13 红 / 23 跳（67.6s）。13 红与 PLAN-1055 D-1055-BASE-01 继承清单及 D-1088-LAYOUT-01 **逐名一致**（av_method_owned×4、av_method_modes、av_owned_first_case、av_owned_recipe、av_test_batch、component_event_sessions_1106、resource_915×3、layout::tests），1092/1093/1094/1120 收据均记"master 基面同名同败、非后包引入"；首跑另有 py_lane×7 红为 **av-providers 准备工件缺失**（foreign.prepared），`cargo run -p ace-abi --bin prepare-av-providers` 后定向重放 7/7 绿。
  - ace-source：990 执行 / 5 红——text_record_source×5，同为 D-1055/D-1088 已登记基面红（1093/1094 收据逐字记录）；含本包新增 adapter 单测（mapset 嫁接触发/恒等 + 1082 字节稳定回归）全绿。
  - 本包 26 例在两组并集内全绿；无新增红。
- **未跑/延后**：§6 冻结并集的其余组（ace-stdlib ace-semantics ace-lowering case-av-language-parity）归 T-05/merge 在冻结提交执行（本改面为普通面 graft + resolver 接线，语义/ lowering/装配面零行为变更，依据：嫁接仅由 HashMap/HashSet 触发、拦截仅以嫁接类型名 gated、失败用例源零 HashMap 提及）；整数 map（asm/map.at 装配线）与 StrBldr 属 972 装配面，由 ace-stdlib 组覆盖。
- **不变证据复用**：1120 收据的 12 例原红同名单 + master 基面同败记录直接承接（本 run 同名同败复现）。
- **代码身份**：worktree clean @ plan-1121-dev（交付提交后无改动）；主检出无本计划代码 WIP。
- **健康**：ace-core check 无新增警告（既有 http_ops/ConfigPairValue 等警告与本包无关）；新代码零 println/调试残留。

### 独立复审记录（2026-11-23 · T-04 · AC-03）

stage: review | review_scope: final（本计划未分设终审角色；**限制声明**：在实现会话内复审，无独立评审上下文可用，结论以提交物与独立重放重建，非执行者摘要） | reviewer_identity: goal-chain session（同实现会话，限制已声明） | plan_id: PLAN-1121 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-1121/auto-lang | branch: plan-1121-dev | reviewed_commit: df4d37940740a384858902e489287e49c7e6106e | base_commit: 6d3bd996740f600b91ece1ef6b77058025566515 | dependency_revisions: auto-down 组内 detached@456c86a（未改动） | spec_inputs: docs/specs/ace-migration/m2-stdlib-core.md sha256 c0baf9f8…4861（worktree 冻结副本）；docs/specs/stdlib/design/ace-collections.md sha256 b379313b…066a（worktree 冻结副本） | next: merge（T-05）

**AC 证据图**：
- AC-01 pass——独立重放 `cargo test -p ace-av --test conformance plan1121 -- --test-threads=1`：26/26 绿（0.83s），冻结提交 df4d37940；V-01 旧行承接含 conv003/004 原 oracle 逐字。
- AC-02 pass——同套件重放覆盖：覆盖 9922、去重 2、缺键 71（受检 KeyError+catch 后续用）、错面静态拒、错值型 verify.type-mismatch、整数 List graft 与 Dyn 近邻 302、用户自声明 HashMap 遮蔽 5。
- AC-03 pass（本记录）——清单审计：diff 全量自读（嫁接/拦截/分派/drop 归一）；零 println/dbg/todo 新增；unwrap 均在内部保证位；15 条迁移行 12 列齐、oracle/SHA16 守恒、文件 diff 恰 16 ins/15 del（多行单元格原样）；SD-01/SD-02 文本逐条对实测行为核对一致；supersedes 双目标、new_spec_components 空（增量写入既有规范，理由充分）。
- AC-04 merge 待办：§6 冻结并集在落地提交执行（merge 所有）。

**独立反例探针（复审临时，已删除）4/4 绿**：owned-Str 局部键移入+字面覆盖=42；int 面键上 insert_str 覆写 → source.map-face-mismatch；迁移类型作形参注解 → source.undefined（**命名边界**，登记 D-1121-MAP-PARAM-ANNOTATION，非阻塞、不在首批合同）；Set 字面重复插入 size=1。

**红账归因（两组并集 run-8_3o0bip，复用 work 阶段有效证据，未重跑全量）**：ace-av 13 红与 D-1055-BASE-01 继承清单及 D-1088-LAYOUT-01 逐名一致（master 基面同名同败：1092/1093/1094/1120 收据链）；ace-source 5 红 = text_record_source 同清单；首跑 py_lane×7 为 av-providers 工件缺失（prepare-av-providers 后定向重放 7/7 绿）。**本包零新增红**。

**测试成本（2026-11-14 裁定）**：新增 26 例，套件墙钟 0.83s，单例均 ≪10s；无经 helper/子进程编译前提（conformance 走 prepared manifest；provider 经显式 prepare lane）。

**计时表**：plan1121 套件 0.83s（26 例）；复审探针 0.09s（4 例）；两组并集复用 work 证据（ace-av 67.6s/1627，ace-source 990 执行）。

**债务**：D-1121-MAP-PARAM-ANNOTATION（形参注解边界，open，非阻塞）。

stage: goal | plan_id: PLAN-1121 | plan_revision: r1 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1121/auto-lang | commit: df4d37940740a384858902e489287e49c7e6106e | next: merge

### merge 收尾记录（2026-11-23 · T-05 · AC-04）

**PLAN-1121:r1 · prepared** — 复审基线 df4d37940740a384858902e489287e49c7e6106e（plan-1121-dev，基于 6d3bd9967）；规范增量 SD-01/SD-02（worktree 冻结副本 sha256 c0baf9f8…4861 / b379313b…066a）；交付提交 = 复审提交（无文档后代）。

**冻结固定组并集（§6，worktree 冻结提交执行）**——`cargo t ace-stdlib ace-source ace-semantics ace-lowering ace-av case-av-language-parity`：

| 组 | 结果 | 归因 |
|---|---|---|
| ace-stdlib | PASS | — |
| ace-semantics | PASS（295） | — |
| ace-lowering | PASS（9） | — |
| ace-source | 红 5 | 原红：text_record_source 五例（D-1055/D-1088 继承清单，1092/1093/1094/1120 收据"master 基面同名同败"） |
| ace-av | 红 13 | 原红：D-1055-BASE-01 清单 12 例 + D-1088-LAYOUT-01（1120 收据同清单同败）；首跑 py_lane×7 为 av-providers 工件缺失（foreign.prepared），prepare-av-providers 后定向重放 7/7 绿 |
| case-av-language-parity | 红 2 | 原红：av.parity.1093 explicit-c-library / c-header-spelling-rejected（1120 归档收据在案同两名；C provider 工件跳过，D-1109 家族） |

零新增可归因红；本包 26 例（av.parity.1121.map-set）在并集内全绿。计时：本轮 prepare 183s + 执行 110s；work 轮 prepare 806s + ace-av 67.6s/1627 + ace-source 990 执行；新增 26 例套件 0.83s（单例 ≪10s）。

**landed**：master 未动（仍 6d3bd9967），无需 rebase；主检出 `git merge --ff-only plan-1121-dev` → master tip = df4d37940740a384858902e489287e49c7e6106e（无 merge commit；10 文件 +1282/-35）。落地冒烟：master 上 plan1121 套件复跑 26/26（0.83s）。

**canonical_verified**：落地提交内 `docs/specs/ace-migration/m2-stdlib-core.md`（PLAN-1121 增量节，sha256 c0baf9f8…4861）与 `docs/specs/stdlib/design/ace-collections.md`（b379313b…066a）与复审冻结副本逐字节一致；`stdlib/ace/asm/map_str.at` 在库（非装配集，`stdlib_assembled_files()` 不列）。无索引/ledger 发布步骤。

**债务移交**：D-1121-MAP-PARAM-ANNOTATION（形参注解边界，open，owner=963 全泛型切片线，非阻塞）。

stage: merge | plan_id: PLAN-1121 | plan_revision: r1 | outcome: pass | delivery_commit: df4d37940740a384858902e489287e49c7e6106e | canonical_specs: docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/stdlib/design/ace-collections.md | archive_path: docs/plans/archive/1121-av-stdlib-map-set.md | cleanup: 见下（cleaned 记录于归档后）

stage: goal | plan_id: PLAN-1121 | plan_revision: r1 | round: 4 | from: merge | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1121/auto-lang | commit: df4d37940740a384858902e489287e49c7e6106e | next: done

**cleaned（2026-11-23 收尾核验）**：`wt-guard.ps1` 双树 clean（auto-lang/auto-down，pwsh）；`git worktree remove` + `git branch -d plan-1121-dev`（was df4d37940）+ 组目录 `D:/autostack/.wt/lang-1121` 已删（0 残留）；plan-artifacts/plan-1121 全目录删除（证据已折叠于本文件 §8/§9 与 work 交接记录；原 t01-probe-evidence.md 要点=本文件 T-01 证据行）；`.retention/policy.json` 已登记 archived/删除/具名持有（D-1121-MAP-PARAM-ANNOTATION）。构建中间产物随 worktree 删除、未回收。任务计数终核：T-01..T-03 完成、T-04 复审 pass、T-05 本记录；current_step 3/3 执行步 + 复审/归档步完成。

**部署观察**：本计划交付物为编译器/stdlib 源码与测试，无生产面制品（backend 发行二进制/前端 bundle）在本范围内；未触发 release 构建或重启——记为 stale-artifact 观察（无受影响制品）。

**批量回归到期核**：`.last-batch-regression.json` last_covered=1118；1121 % 5 ≠ 0 且收据龄 <48h → 不到期，不触发。

**归档补记（2026-11-23）**：依据 `df4d37940` 的 r1 复审和 `9d3cd9e63` 的收尾记录补齐 AC-03/04、T-04/05 和 5/5 进度，核实本页 10 处相对链接。迁移表实数为 13 条集合用例和 2 条转换用例，共 15 条。核验交付提交与原收尾提交均在 `master` 的历史中，两份规范与交付提交的 Git blob 一致，计划分支、工作树组及暂存目录均已移除，现存材料登记表有归档记录。债务 [D-1121-MAP-PARAM-ANNOTATION](../KNOWN-DEBT-AND-RISKS.md) 随归档补记提交。此次只核验文档和 Git/磁盘状态，复用原功能验证记录。

stage: goal | plan_id: PLAN-1121 | plan_revision: r1 | round: 5 | from: merge | to: done | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1121/auto-lang（已移除） | commit: df4d37940740a384858902e489287e49c7e6106e | next: done
