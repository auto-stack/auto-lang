---
plan_id: PLAN-959
status: archived
feature_name: M2 全语言与全产品兼容分母冻结
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-07
plan_revision: 2
current_step: 6
total_steps: 7
depends_on: []
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-denominator.md]
touched_goals: [GOAL-002]
---

# [PLAN-959] M2 全语言与全产品兼容分母冻结

## 0. 变更摘要

以922的A–G 68行、REP-01..11为种子，逐项连接语法/公共API/标准库符号/工程/入口/目标/平台/原oracle/owner；全量扫描兄弟产品、子模块、安装SDK及冻结副本。分语言规范、真实支持、已知旧bug、未决四类；不得把旧AST枚举存在当运行支持。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付可追溯的完整替换分母和后继责任表。范围为规范/公共API/真实consumer/安装物的双向调查，不实现任何语言/后端/产品迁移，不要求未来ACE producer已存在。unknown可保留并有owner/关闭任务；缺行或无owner则本盘点不完成，unknown仍阻M2终签。

## 2. 架构方案

依赖：无实施依赖；先核当前基线。依赖指正式交付来源及适用接口，不把WIP/reviewed当delivered；不以最终认证尚未交付阻塞独立能力建设。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
**2026-11-07 实施授权**：用户 `/goal 完成计划959`（auto-plan-goal 范式）授权本 Plan 全链——
在专属 worktree 执行 work（含建树）、review、合同内有界修复与 merge（Spec 沉淀/归档/清树）；
不授权改目标/验收/范围/仓集，不授权未绑定本修订与提交的 merge。原 §9 "blocked 待实施指令"
由此解除。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `crates/auto-ast/src/ast.rs`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/core/src/source/ast.rs`（本轮确认存在；开工重核符号/版本）。
- `docs/specs/ace-migration/replacement-scope-and-evidence.md`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

## 5. 详细设计

分母行使用稳定ID，至少记录repo+revision、入口/符号、语法/类型/效果、旧producer、consumer、目标/平台/feature、source/oracle、状态、后继Plan/AC、owner与证据适用身份。沿922口径增量核查，禁止只数目录或用不存在的源码证明支持。

T-01对规范→AST/入口与consumer→入口双向建表；T-02追真实调用链及安装/冻结SDK副本；T-03将每项义务分配给945–948及960–987，并核分包依赖。语法FStr/union/tag/grid/Hold/Box/Arc、默认/命名参数、global/store初始化、C/Rust/Python bridge等均须行级归属；只登记不提高实现状态。

分母冻结带源身份；冻结后新增/变更以delta行追加并使受影响旧证据陈旧，不永久把今天仓列表当封顶。不可读/未安装保unknown；失效旧功能/废弃API须有正式退出依据，不能直接删行。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/ace-migration/m2-denominator.md | 建立范围/身份/owner口径及当前未闭义务，绝不声明实现 | 替换分母 | AC-01..03 |
| SD-02 | modify | docs/specs/ace-migration/replacement-scope-and-evidence.md | 关联新分母与后继责任，保原分类与REP门 | 防漏项 | AC-01..03 |

## 6. 测试设计

本包只做只读溯源与文档/清单一致性验证，不因通用模板运行cargo或实现新checker。若后来确需持久可执行checker，须显式修订实现范围；本包不以新生产功能为前置。

| Case | AC | 输入/操作 | oracle | owner |
|---|---|---|---|---|
| V-01 | AC-01 | 对照AST/API正向清单与TS/AGE/安装反向consumer清单 | 两侧每个可达项均有行；unknown不得当不存在 | work/T-04 |
| V-02 | AC-02 | 在TEMP审计副本删除一个TS consumer、冻结SDK或unknown行 | 对照来源发现缺行；恢复行只恢复完整性，不提高实现状态 | work/T-04 |
| V-03 | AC-03 | 将某缺口后继改为不存在编号或形成循环前置 | 责任/依赖校验失败；真实后继及AC恢复才通过 | work/T-04 |
| V-04 | AC-01/03 | consumer源码变更但沿用旧证据SHA | 标陈旧并重核受影响行，未变行不重复全仓盘点 | work/T-04 |

独立review核实际来源/完整性/后继可执行性；不把本轮计划作者自查称独立复审。原始审计材料统一本Plan scope，最终摘要折入Plan。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 规范/API/所有真实消费者与安装物双向覆盖，含unknown，来源身份完整 | T-01/02/04、V-01/04 |
| AC-02 | work | 五类迁移状态有真实依据，缺行/假实现控制可被识别；不要求实现未来能力 | T-02/04、V-02 |
| AC-03 | work | 每项required与unknown有owner、后继Plan/AC/输入/oracle，依赖可执行；全M2缺口不因盘点完成关闭 | T-03/04、V-03 |
| AC-04 | review | 独立按实际来源复核AC和遗漏/延期，不信表格勾选 | T-05 |
| AC-05 | merge | 口径Spec沉淀、归档、来源与scope/树清理对账 | T-06/07 |

## 8. 执行步骤

预定树`D:/autostack/.wt/lang-959/auto-lang`，分支`plan-959-dev`（已建，基线 master
141a57ccd→ff 至 92bbf298b；交付提交 3b13bf42e）。跨仓只读，计划簿记在主检出。材料仅
`D:/autostack/plan-artifacts/plan-959/`（SCOPE.md、t01-baseline.md、audit/{check_denominator.py,
check-results.md,v0-real.md,m1..m5 突变副本}；merge 整删）。

- [x] **T-01｜owner_stage: work｜AC-01**：冻结规范/API/仓/子模块/安装基线及双向枚举来源。
  已完成（spec §1/§6/§7：8 仓 branch@SHA + 24 个 auto-os 子模块 + apps.manifest 28 id 逐项
  + autographics 冻结 SDK ×2（.sdk-source==sdk/frozen 681 文件字节等同，两钉 932ad487/7b987ca8
  不一致→UK-06）+ man-install/web-install/musk install.sh 三安装器 + ~/.auto 现存安装物 +
  本机引擎二进制；正向源：`auto-ast/src/ast.rs` 全枚举（Stmt 30/Expr ~60/Type ~45/Op 39）、
  facade 24 直定义 pub fn + 50 re-export + entry/script_entry 全表、auto-man CLI
  42 个顶层子命令（cli_spec.rs Commands 枚举复算；另无子命令 REPL 臂+位置文件臂；
  含 block/old-repl/parse/eval/config 隐藏臂）、stdlib 188 .at/427 目录项/859 BigVM/
  366 #[vm]/17 操作合同、trans 栈 11 目标；反向源：44 仓 Cargo 依赖+Rust API 引用+脚本级
  `auto run -r vm` 消费+pac.at/api.at/#[test] 清单（6 个只读 Explore agent 并行+主会话
  抽查 binding.tsv 131 行/ast.rs 8 锚/operations.toml 17/vm_bridge 路径漂移/Lambda 桩
  expr_fallback.rs:394-400/grid 关键字 token.rs:412 直证）。
- [x] **T-02｜owner_stage: work｜AC-01/02**：逐行追真实旧producer→consumer，区分实证/未接线/
  缺能力/陈旧/unknown，不实现producer。已完成（spec §3 L 表 49 行：每行 parser+codegen
  file:line 双证 + 四类分账——parser-only/AST-only 10（union/tag/grid/Box/Arc/alias/
  type-alias/默认参数/Lambda 桩/NavCall/OnEvents-SceneDecl/byte 字面量）、已知旧bug
  或差异 14 行（真实支持侧 13：binding diff-legacy 28 格+B13/B15 int=i64+复杂 LHS/
  捕获复合赋值/动态调用 unimplemented+负索引 SEM-02；另 L-23 Lambda 桩为 AST-only
  兼旧bug）、未决 1（L-36 平台条件）、无此形 3 行 5 构造（L-04 i128-f32 后缀；
  L-16 set 字面量；L-46 spread/when-match/global）；新 AST 在场列只记在场不记支持；
  【2026-11-07 修复 F-959-R1：尾注行集原误含 L-03 漏 L-10、L-18 未统一标记、
  parser-only 误计 11——已修并经 grep 行标记重算 oracle 复核=14/10/3/1 逐项相等】；
  §4 A/B 刷新：22 个 922 锚复验（vm_bridge 移 ui/ 子目录同行、tool_ops :334→:353，余原位），
  old-repl 实测启 AutoVM REPL 修正 A-06；§5 S 表 25 域（redis/clipboard/dialog/i18n 仅 natives
  面不漏记；D-956-2 旧 bug 作 oracle 输入）；§6 C–F 刷新+F-13 auto-shell/F-14 20 应用仓破损钉
  4 处/F-15 零引用账；§7 K 表 8 行（K-03 修正 922 "auto-notes vendor=冻结副本"表述——实为
  autodown JS 子模块+jade-garden server 副本引活 auto-lang）；未实现任何 producer。
- [x] **T-03｜owner_stage: work｜AC-03**：每行映射正式后继任务/AC与oracle，核语言/库/产品/
  退役责任及语义依赖无环。已完成（spec §8：945–948/960–987 33 包 + auto-os/064 逐包喂给
  行集，每行后继列带 Plan/批次/T-08/AC-06；无主项行级补齐 4 项——平台条件→L-36（966 T-01
  +978 T-08）、TaskDef/on=>→L-43（968）、Hash 各形态→L-32（966，Design55 §5 归属落行）、
  vscode 模式→C-06（971+986）+Lambda 桩→L-23（964）；DAG 由 frontmatter depends_on 重建
  复验无环（在途前置仅 941/938/939，已 archived 前置不阻塞；OS064↔984 无互等）；§9 UK 13 行
  全 owner+下一步（UK-09 闭、UK-11/12/13 新增：launch-vm.cmd:33 二进制缺失、web vendored
  installer 滞后 cb4ab3dd、平台条件谓词）；§8 末条明文"全 M2 缺口不因盘点关闭"）。
- [x] **T-04｜owner_stage: work｜AC-01..03**：执行V-01..04对照审核，汇总来源/缺口/owner，
  不运行未来产品测试。已完成（plan-artifacts/plan-959/audit/check-results.md；TEMP 审计
  脚本 check_denominator.py 不入产品：表行连续性/交叉引用/后继存在/逐包喂给/DAG/基线 SHA
  六项——**REAL PASS**（L49/S25/K8/UK13、33 包全有喂给、无环、8 仓 SHA 新鲜）；**V-02**
  三突变副本全 FAIL 被识别：删 K-02 冻结 SDK 行→"K missing [2]+3 dangling+§8 981/987
  dangling"、删 UK-06→"UK missing [6]+dangling"、清空 976 TS consumer 喂给→"976 fed by
  ZERO rows"，恢复行即复 PASS 且行无实现状态字段可提升；**V-03** 后继改 999→"non-existent
  plan+977 zero-fed"、注入 959→984 环→"cycle 959->984->959"+18 派生环；**V-04** 回拨
  auto-ui SHA→STALE 识别，且审计期间实捕三轮他方落地漂移（auto-lang 141a57ccd→92bbf298b
  含 8 crates 文件、auto-ui 四前移至 b4094acd8、auto-os→07646ddd7 含 host 代码+8 gitlink），
  每次只重核受影响锚（L-01/L-25/S-16/C 五锚/driver Cargo:11/host main.rs:46-49/apps.manifest
  28 同集）全部原位，未重盘全仓；**V-01** 正向×反向双向对账：AST→L、facade/CLI→A/D、
  stdlib→S、trans→B、TS/AGE/安装→C-F/K、28 manifest→OS064 表，每侧可达项有行，unknown 13
  行不当不存在；validator 还抓出本台账两处真缺陷（UK-09 漏行、948 喂给无编号引用）并修复；
  健康=纯文档零 cargo、链接 6/6 解析、无游离 TODO）。
- [x] **T-05｜owner_stage: review｜AC-04**：独立复核分母和后继责任，具名遗漏回工作阶段修订。
  已完成（两轮：round2 needs_fix F-959-R1（P3 §3 尾注行集错+L-18 未标记）/F-959-R2（P4 CLI 计数）
  → work 同树修复 05a61f900（diff 仅 L-18+尾注 7+/4−）→ round3 受影响条目复核全闭：R1
  oracle=grep 行标记重算 14/10/3/1 与尾注逐项相等（markers L-01/02/06/10/17/18/20/22/23/
  25/26/33/47/48，含 L-23 桩另计）；S 尾注 25 行=实际行数；validator 结构 PASS@05a61f900；
  R2=42 顶层子命令复算一致；同会话独立复审限制如实声明，裁决由工件重建——独立抽查 10/10
  锚（work 未查过）+计数独立重算+DAG 独立重建，未采信 work 摘要）。
- [x] **T-06｜owner_stage: merge｜AC-05**：落口径Spec/合入/归档；不将能力状态提升。
  已完成（prepared=reviewed 05a61f900+冻结 delta 两件；rebase master 8a6b0af46 后
  range-diff 全等 3b13bf42e→14b7d674b/05a61f900→6dc23772c；landed=master --ff-only
  tip 6dc23772c 线性零 merge 提交；canonical_verified=m2-denominator.md blob
  bf86494ab/sha256 c3dd5491ec1395aa…（=reviewed 内容）、replacement-scope… blob
  85cbded0a（SD-03 +15 行，SD-01/02 零删改）；纯文档零 cargo 固定组门=内容/链接检查
  （6/6 链接解析、validator 结构 PASS）；无 legacy ledger 写入、无能力状态提升）
- [ ] **T-07｜owner_stage: merge｜AC-05**：摘要折入本Plan、所有输出路径对账与scope清理，guard clean后清树/分支。

T-01→02→03→04→05→06→07；调查不等待下游实现，所有阶段真实完成才记账。

## 9. 复审记录

2026-11-06第二轮复核r2：修复通用模板误要求producer实现的问题，重写§1/5–8为只读盘点交付；原7任务ID保持、0/7。此处修正手段，不减少全M2替换义务。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

2026-11-07 | stage: work | plan_id: PLAN-959 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-959/auto-lang | branch: plan-959-dev | base_commit: 92bbf298b（建树于 141a57ccd，期间 master 前移经 ff-only 跟进） | code_commit: 3b13bf42e | dependency_revisions: 跨仓全只读@冻结读取（ui b4094acd8/man 2a9b16e50/web 2d698413b/os 07646ddd7/down 58ea33f69/musk 721ecfa07/gen 593b4eecd；完整 44 仓见 spec §1/§6）；auto-ui 他方会话在途合并致四轮前移，均按 delta 规程只重核受影响锚 | task_ids: T-01..T-04 | evidence: 交付两件@3b13bf42e——①新 Spec docs/specs/ace-migration/m2-denominator.md（344 行：§1 基线/§2 口径/§3 L 49 行/§4 A-B 刷新+22 锚漂移/§5 S 25 行/§6 C-F 刷新+OS064 28/§7 K 8 行/§8 后继喂给 33 包+OS064+无主项 4 补齐+DAG 无环/§9 UK 13/§10 delta 规程）②replacement-scope-and-evidence.md 增 SD-03（挂接台账与行级后继，不改 SD-01/02 与 REP/V06 门）；V-01..04 对账=plan-artifacts/plan-959/audit/check-results.md（REAL PASS、5 突变副本全 FAIL 识别、3 轮实漂移捕获、2 处台账真缺陷自抓自修）；spec sha256 前缀 a2f3688effd01739；健康=纯文档零 cargo（§6 口径）、链接 6/6、无游离 TODO | blockers: 无合同内未决（UK-10 auto-gen 复活/归档沿用用户裁定门，不阻盘点完整性；UK-13 平台条件为语言侧未决已具 owner 966/978） | next_responsible_role: review（T-05 独立冻结复审@3b13bf42e：按实际来源抽查 L/S/K 行证据、无主项归属、UK owner、SD-01/03 对照 §5 规范增量表；不信表格勾选）

stage: goal | plan_id: PLAN-959 | plan_revision: r2 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-959/auto-lang | commit: 3b13bf42e | next: review

2026-11-07 | stage: review | review_scope: final | reviewer: 同会话独立复审（限制如实声明：与实施同会话/同上下文，计划未授权另建 reviewer agent；裁决由工件重建——抽查选取 work 未查过的锚，计数由规范正文重算，不采信 work 摘要） | plan_id: PLAN-959 | plan_revision: r2 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-959/auto-lang | branch: plan-959-dev | reviewed_commit: 3b13bf42e | base_commit: 92bbf298b | dependency_revisions: 跨仓只读@spec §1（ui b4094acd8/man 2a9b16e50/web 2d698413b/os 07646ddd7/down 58ea33f69/musk 721ecfa07/gen 593b4eecd） | spec_inputs: SD-01 新 m2-denominator.md（344 行）+ SD-02 replacement-scope SD-03 节（+15 行，SD-01/02 零删改）vs 计划 §5 规范增量表（add/modify 目标与 frontmatter supersedes/new_spec_components/touched_goals GOAL-002 一致）；无 implemented 越权声明（通读 §2 口径与各表"后继"列仅登记）；无 legacy ledger 写入（树 clean，.autoos 未触） | acceptance_results: AC-01 pass（独立抽查 10/10 锚——L-07 expr_async.rs:257、L-27 stmt_emit.rs:116、L-14 scenario.rs:974、L-24 decls.rs:1599-1604、L-43 stmt_emit.rs:3275、新 AST ParamMode:86/Enum:169/ForIn:450、F-13 backend.rs:5/shell.rs:10 全部原位；计数独立重算 stdlib .at=188、http.at fn=79、.sdk-source=681、L 行=49；OS064 A05/A10/A28 映射与 064 §5 一致；validator REAL PASS 复跑） | AC-02 **partial→F-959-R1**（五类/四类分账有真实依据、V-02 突变识别成立；但 §3 尾注"已知旧bug/差异 14 处"所列行集错：含 L-03（无标记，实为 parser-only）漏 L-10（有 **已知差异** R01/R03/R04/R07）；L-18 实载 SEM-02 负索引差异却无统一 **已知差异** 标记；"parser-only/AST-only 合计 11 行"实为 10（L-03/08/12/13/14/23/24/38/44/45），L-04 是无此形非 parser-only；无此形实为 3 行 L-04/16/46——摘要与行不符即台账完整性缺陷） | AC-03 pass（§8 喂给 33 包+OS064 逐项有行；无主项 4 项行级归属核实；DAG 由 frontmatter 独立重建无环；UK 13 行全 owner+下一步） | pending_closeout_gates: merge-owned——SD-01/03 读回、archive、输出路径对账、guard 清树/分支/组/scope、.retention 刷新；纯文档零 cargo 按 §6 口径 | findings: **F-959-R1**（P3，AC-02/T-02，原要求缺口类）§3 尾注行集与行标记不一致——修正：L-18 补统一 **已知差异** 标记；尾注改为"已知旧bug/差异 14 行（L-01/02/06/10/17/18/20/22/23/25/26/33/47/48）、parser-only/AST-only 10 行（L-03/08/12/13/14/23/24/38/44/45）、无此形 3 行（L-04/16/46）、未决 1（L-36）"；oracle=grep 行标记重算与尾注逐项相等；**F-959-R2**（P4，T-01 收据簿记）"auto-man CLI 40 命令"实为 Commands 枚举 42 个顶层子命令（+无子命令 REPL 臂+位置文件臂）——修正收据数字；T-02 收据同步 R1 数字 | evidence: 本记录内嵌命令/结果即持久证据；抽查与重算命令见 plan-artifacts/plan-959/audit/（merge 折叠后整删） | next: work（T-02 尾注/L-18 修复 + T-01/T-02 收据数字修正，同树提交后回 review 复核受影响条目；修复周期 1/3）

stage: goal | plan_id: PLAN-959 | plan_revision: r2 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-959/auto-lang | commit: 3b13bf42e | next: work

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。

2026-11-07 | stage: review | review_scope: final | reviewer: 同会话独立复审（限制同 round2：与实施同会话，工件重建裁决；round3 仅复核修复受影响条目，未动 round2 已验证据） | plan_id: PLAN-959 | plan_revision: r2 | outcome: pass | worktree_path: D:/autostack/.wt/lang-959/auto-lang | branch: plan-959-dev | reviewed_commit: 05a61f900 | base_commit: 92bbf298b | dependency_revisions: 跨仓只读@spec §1；auto-lang master 前移 92bbf298b→8a6b0af46 纯 docs（0 非 docs 文件）判读=无锚失效，auto-ui 稳定 b4094acd8 | spec_inputs: 同 round2 + 修复 diff 核对（3b13bf42e→05a61f900 仅 L-18 标记+§3 尾注，无越权改动） | acceptance_results: AC-04 pass（round2 全量 + round3 修复复核）；AC-01 pass（10/10 独立锚+计数重算 188/79/681/49+OS064 映射）；AC-02 pass（R1 闭：oracle 重算 14/10/3/1 逐项相等、L-18 已标记、S 尾注 25=实际）；AC-03 pass（喂给 33 包+OS064、无主项 4、DAG 无环独立重建、UK 13 全 owner） | pending_closeout_gates: merge-owned——SD-01/SD-03 读回于落地提交、archive、全输出路径对账、guard 后清树/分支/组、plan-artifacts scope 整删、.retention 刷新；纯文档零 cargo（§6 口径），无统一回归门需要 | findings: 无未决（R1/R2 已闭于 05a61f900） | evidence: 本记录内嵌命令/结果即持久证据；v0-real/v1-repaired sha256 前缀 a2f3688effd01739/c3dd5491ec1395aa（后者=05a61f900 内容）及 5 突变副本在 plan-artifacts/plan-959/audit/ | next: merge

stage: goal | plan_id: PLAN-959 | plan_revision: r2 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-959/auto-lang | commit: 05a61f900 | next: merge

2026-11-07 | stage: merge | PLAN-959:r2 | prepared: reviewed 05a61f900 + 冻结 delta（m2-denominator.md 新建 353 行 + replacement-scope SD-03）→ rebase master 8a6b0af46（docs-only 前移），range-diff 全等（3b13bf42e→14b7d674b、05a61f900→6dc23772c） | landed: master `--ff-only`，tip=6dc23772c 线性无 merge 提交 | canonical_verified: docs/specs/ace-migration/m2-denominator.md blob bf86494ab/sha256 c3dd5491ec1395aa847f…（=复审 v1-repaired 内容）、docs/specs/ace-migration/replacement-scope-and-evidence.md blob 85cbded0a（SD-01/SD-02 零删改，+SD-03） | gates: 纯文档计划——固定组门=内容/结构/链接检查（6/6 解析、validator 六项 PASS）零 cargo（§6 口径）；无统一回归门需要 | archived: docs/plans/archive/959-ace-m2-denominator.md（status: archived，随本簿记提交） | batch-regression: **due**——PLAN-990（990%5=0）在 last_covered_plan_id=956 之后落地（receipt@2026-11-06T01:12Z gate_passed=false 含已知红+957 编译器崩溃在途）；本 merge 收尾后按 regress 技能主检出单实例执行 | cleaned: pending（本提交后执行：guard→树/分支/组/scope 清理→.retention 刷新→cleaned 收据） | next: cleanup
