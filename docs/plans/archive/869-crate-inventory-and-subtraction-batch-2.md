---
plan_id: PLAN-869
status: archived
feature_name: 三仓按 crate 检验清单与净瘦身第二批（含 80 万目标口径冻结）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 4
current_step: 10
total_steps: 10
supersedes_spec_components: [docs/specs/repository-boundaries/net-slimming.md, docs/specs/repository-boundaries/project.md]
new_spec_components: [docs/specs/repository-boundaries/crate-inventory.md]
touched_goals: []              # 有意留空：不改任何 GOAL 的定义/达成状态；80 万目标是 roadmap 的争取目标，本计划只冻结其分母与披露口径
affects: [repository-boundaries, test-infrastructure, language-sdk, auto-ui, auto-man]
depends_on: [PLAN-867]
---

# [PLAN-869] 三仓按 crate 检验清单与净瘦身第二批（含 80 万目标口径冻结）

> **整 Plan 启动门 G869 = U867**（867 全部 AC／final review／冻结门／三仓合入／Spec 与 ledger／归档收尾闭环）。867 未归档前只允许只读核查与规划簿记；不建实施树、不跑整档、不改三仓实现。
> 本计划承接 **M2 之后的路线下半部**：[roadmap §2.1](../roadmap-v0.6.md) "拆分后按 crate 建立检验清单……有证据地去重、精简和重构" 与 §2.2 "约 100 万行 → 80 万行以下（同脚本、分列披露）"。

## 0. 变更摘要

844 已把 M2 收口（U844 成立，M2 四份全 archived，ledger v114），但其生产净删**终值只有 −1,221 行**（lang NET-PROD 308,909→307,688），真重复清偿 5 族。这里−1,221是B（NET-PROD）；A（约定范围全部.rs桶）历史约100万、距80万约20万，二者不是同一口径。不得用B批净删线性推算A需要多少批；T-01须按同组三仓commit复算A/B差额，原估算只作历史。

同时，844 留下的"残留候选"（error-reply 采样族 ×10、`time_now`、`math` 20+ 族、`str.len`、`nv_to_value`）与 843 留下的结构残留（`call_emit::call_inner` 7,439 行 if 链、`stdlib.rs`/`http_server.rs`/`trans/rust.rs` 三入口、`lib.rs` async glue）**都没有 owner**；UI 面（NET-PROD 296,190，最大文件 `renderer.rs` 42,337）从未做过结构或减法审视。

本计划做三件事：**① 按已裁定口径（A 对外披露/达标 + B 不反弹约束 + 三分列）复算三种并列数字并写回规范；② 产出三仓全部 workspace 成员 × 七栏的 crate 检验清单；③ 按清单执行第二批减法（含经用户按批签核的结构性减法）并用同口径收据证明真实生产净删**，同时把结构热点排成可执行的结构计划包（不在本计划做手术）。

## 1. 目标

- **G1 口径已定并可复算**：用户 2026-10-28 裁定 —— **A（全部 `.rs`）作对外披露与达标口径，B（生产 NET-PROD）作"不得反弹"约束，测试/生成/第三方分列披露**；C（旧"单 crate 行数"76.6 万）仅作历史对照、不作当前分母。三种口径的当前值与差额仍须用 844 的 `measure.py` 同脚本并列复算，供对照与复核。
- **G2 清单齐全**：三仓全部 workspace 成员 × 七栏（职责／公开出口／重复实现／无消费者旧路径／依赖方向／测试与档位／文档与规范），缺栏即红；该清单成为后续减法批次与结构立项的单一事实源。
- **G3 批次真减法**：按清单冻结候选（真重复族／无消费者退役／过渡层／**已授权的结构性减法项**），实施后以 NET-PROD 收据证明**真实生产净删 ≥ 2,000行** 且至少 1 个真重复族或退役项闭合；沿用 844 的四类处置与 V-04 负控（搬移=0、helper 入账、测试/生成/第三方分列）。结构性减法项必须逐项满足 §5.3 的**四要件**（替代路径/承诺迁移/门禁绿/用户按批签核）。
- **G4 测试联动**：任何删除导致的测试 ID 消失，必须在 867 闭合账本里落为 **RETIRED 且有证据**；867 的三仓完整档在本计划冻结组合上复跑通过。
- **G5 结构立项就绪**：把已登记热点（语言 `call_inner`、三入口、`lib.rs`；UI `renderer.rs`/`vue.rs`/`aura_view_builder.rs`；Man `auto/main.rs`）排成结构计划包，每个热点给出规模/职责/拆分假设/触面档/前置/风险/审查面，**本计划不做结构手术**。

**非目标**：不在本计划拆 `call_inner` 或 UI 巨文件（只立项）；不改产品语义（`math`/`str.len`/`time_now`/`error-reply`/`nv_to_value` 归 872 语义双轨计划，且已裁定全部 DISTINGUISH、零行为变更）；**不无条件退役** 844 的"保留有证"项（`lib-legacy`、`type_registry` REPL 面、aavm 全家、`str.len`/`math` 双轨、auto-api-gen 跨仓单源、auto-man 兼容门面）——它们只能作为结构性减法候选按 §5.3 四要件取证并经用户按批签核；不做 workspace member 拆分（THR-D1 归 873 测量计划）；不做数据域/运行时域独立仓；不重定 80 万这个目标数值本身。

## 2. 架构方案

1. **口径层**：在 `net-slimming.md` 现有桶定义上登记已裁定口径（**A 对外披露/达标 + B 不反弹约束 + 测试/生成/第三方分列**），用 `measure.py` 一次复算出三组当前值/差额供对照。该口径对所有批次生效，收据按此出。
2. **清单层**：新规范页 `crate-inventory.md` 承载"三仓成员 × 七栏"，逐 crate 一行；数据来源=各仓 `Cargo.toml` 成员、`cargo metadata`、`measure.py` 的 per-crate 切片、867 的档位归属、777 的依赖边。清单必须机器可复现（生成脚本 + 差异检查），避免成为手工台账。
3. **减法是执行层**：候选 → 负控（cfg/宏/字符串注册/脚本/直接 manifest/外部公开消费）→ 实施 → 收据（NET-PROD + 重复族/退役项 + 测试处置）→ 867 门复跑。判据沿用 844，不新造弱判据。
4. **结构立项层**：每个热点一个"立项包"（规模、职责边界、拆分假设、是否需要 probe、触面档、前置、风险、审查面），交后续计划执行；本计划只保证"可被独立执行"，不预写实现。

判定原则：

- 清单与口径都是**验收输入**：不得为让某 crate 显得"瘦"而改分类（把生产标 generated、把测试算生产、把移仓算净删一律禁止）。
- 删除必须**先证明无消费者**（cfg/target/feature、宏/字符串注册、脚本、直接 manifest、公开承诺五面全查），DEPRECATED 标签不等于无消费者。
- 结构热点"规模"只用于排序定位，不能作为拆分必要性的唯一理由；拆分必要性要给职责/耦合/变更频率/测试可行性证据。

## 3. 技术栈

`measure.py`（844 冻结口径）、`cargo metadata`（成员/依赖/features）、git（`log`/`blame` 佐证活跃度与公开承诺）、Python 3（生成清单与差异检查）、nextest/libtest（借用 867 的档位归属）、Markdown 规范页 + JSON 机器可读清单。

## 4. 需求分析与背景调查

本次修订授权（2026-10-28）：用户明确要求按本会话核查建议修订867/869/872/873及总路线，只改主检出计划文档；不执行产品/验证设施改动、不发布canonical/ledger、不建树/agent、不分配874/875或其他新号、不推送。已给的A/B、2,000行批粒度、六族DISTINGUISH裁定继续有效；具体结构性减法批仍沿原签核规则。

### 4.1 授权与轮次

- 用户 2026-10-28 在 867 规划后询问"867 之后下一步做什么"，并确认把该"下一步"写成计划文档。
- 本轮授权：只在语言主检出写合同与只读核查；不建树、不改三仓实现、不改 canonical Specs/ledger。
- 未来实施需 G869（=U867）满足后建组树；减法删除属产品源码改动（Category B），需按 §8 门禁执行。

### 4.2 已观察事实（2026-10-28，可复核）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | 844 终值：lang NET-PROD 308,909→307,688（**−1,221**）；man 资产类 −5 文件；ui 零触面 | [net-slimming.md](../specs/repository-boundaries/net-slimming.md)、[844 归档 §9](archive/844-cross-repository-dedup-and-net-slimming.md) |
| F-2 | 三仓 NET-PROD 现值：lang 307,688 / ui 296,190 / man 24,530，合计 **628,408** | 同上（ui/man 未受 844 影响） |
| F-3 | 三仓 `.rs` 桶合计约 **1,003,007** 行（P 801,621 + T2 81,422 + XT 63,183 + G 20,347 + VEND 36,434；未含 XO/AT/docs） | 844 T-01 `probe/baseline-stats.txt`（本地工件）、`net-slimming.md` 桶表 |
| F-4 | 旧估算201,786/165批混用A/B及不同时间点，不作当前事实或排期依据；若A=1,003,007则距800,000为203,007，当前差额由T-01同源复算 | r4纠正F-1/F-3推算，旧值仅历史 |
| F-5 | UI 面从未做结构或减法审视：NET-PROD 296,190、P 369 文件、inline test 114,485、生成 19,687；最大文件 `renderer.rs` 42,337、`vue.rs` 34,451、`aura_view_builder.rs` 28,044 | 844 T-01 基线（P total 口径） |
| F-6 | 语言最大文件（P total）：`auto-api-gen/api_gen.rs` 9,772、`stdlib.rs` 8,187、`trans/rust.rs` 7,825、`call_emit.rs` 7,553、`http_server.rs` 5,153 | 同上 |
| F-7 | 843 结构残留（入口生产行口径）：`stdlib.rs` 5,679、`http_server.rs` 4,156、`trans/rust.rs` 5,960，均超 843 自设 ≤3000/3500；`call_emit.rs::call_inner` 7,439 行 if 链需逐块 early-return 审计；`lib.rs` 9 个 async run-glue 留根 | [843 handoff §4](attachments/843/handoff.md)、[KNOWN-DEBT P843 段](KNOWN-DEBT-AND-RISKS.md) |
| F-8 | Man：根 9 成员 + `core/` 子 workspace；最大文件 `crates/auto/src/main.rs` 3,222 | [man-three-repository-delivery.md](../specs/repository-boundaries/man-three-repository-delivery.md)、844 基线 |
| F-9 | 844 保留有证（勿删）：`lib-legacy`（playground 活跃）、`type_registry` REPL 面（API pin）、aavm 全家（taa 语料）、`str.len`/`math` 双轨（语义不同）、`auto-api-gen`（跨仓单源）、auto-man 兼容门面 | net-slimming.md 末节 |
| F-10 | 844 残留候选（超出其减法授权）：error-reply 采样族 ×10、`time_now` 双版、`math` 20+ 族双轨、`str.len` 字符/字节、`nv_to_value` 双签名 | [844 归档 §2 保留表/§6](archive/844-cross-repository-dedup-and-net-slimming.md) |
| F-11 | 867（进行中）将交付：三仓完整档（含 man `t/tc/tcore/ti` 与 UI `tu/ti/tn`）、未解释缺口=0 闭合账本、红集对齐、档位选择下限守卫 | [PLAN-867](867-split-equivalence-test-closure.md) |

### 4.3 规范依据

- [net-slimming.md](../specs/repository-boundaries/net-slimming.md)：桶定义、NET-PROD 主指标、分列与负控规则、844 收据。
- [repository-boundaries/project.md](../specs/repository-boundaries/project.md)：三仓身份与旧实现保全。
- [roadmap-v0.6.md](../roadmap-v0.6.md) §2.1/§2.2：按 crate 检验清单 + 100 万→80 万目标（同脚本、分列披露、未达须报告）。
- [PLAN-867](867-split-equivalence-test-closure.md)：测试等价出口（本计划的验收基础）。
- [PLAN-843](archive/843-language-core-structure-and-module-ownership.md)、[PLAN-844](archive/844-cross-repository-dedup-and-net-slimming.md)：结构热点与减法先例。

### 4.4 假设与不声称

- **A1**：`measure.py` 可复用（844 冻结版），但 per-crate 切片能力需 T-01 验证；若不能直接切片，先补受控扩展再出数。
- **A2**：三仓 `Cargo.toml` 成员可由 `cargo metadata` 稳定枚举（867 已用同方法做档位归属）。
- **不声称**：本轮未跑 `measure.py` 复算、未生成清单、未做任何候选判定；F-1..F-11 全部引自既有收据/规范/只读观察。差额数字（≈201,786）是按既有基线的算术，不是新测量。

## 5. 详细设计

### 5.1 80 万口径三案（T-01）

| 口径 | 定义 | 当前值 | 到 80 万的差额 | 语义/风险 |
|---|---|---|---|---|
| A 约定范围全部 `.rs` | F-3约定桶P/T2/XT/G/VEND；XO的examples/experimental/website另列 | 历史≈1,003,007（非当前实测） | 若为1,003,007则差203,007；201,786属另一时点，须同源核对 | 范围/分类冻结，不靠删测试达标 |
| B 生产 NET-PROD | `net-slimming.md` 主指标三仓合计 | 628,408 | 已低于 80 万 | 只反映生产手写代码；与"100 万"基准不同分母 |
| C 旧"单 crate" | THR-D1 记载的 2026-10-02 单 crate 76.6 万行/1,650 文件（拆分前形态） | 拆分后该口径已不成立 | — | 仅历史对照，不可作为当前目标分母 |

- T-01 产出：三案复算输出 + 已裁定口径的落地记录（形成net-slimming proposal，merge再写canonical）；所有报告必须并列披露三案数字（A 为达标口径、B 为不反弹约束、C 仅历史对照）。
- 对外表述规则：达标判定只看 A；B 用于证明"生产代码不反弹"；测试/生成/第三方单独列行，不得合并进主指标。

- T-01冻结A覆盖P/T2/XT/G/VEND及排除XO，报告写明“约定范围全部.rs”；若原用户A裁定要求全仓含XO而与既有表冲突，具体needs_replan，不静默缩分母。A/B/C绑定同组三仓commit、脚本hash及分类版本，历史值不混作当前差额。
- 每批2,000仅是已裁定粒度。安全候选不足须报告并等合批/重规划，不扩大删源凑数，不自动追加计划。80万未达按roadmap披露，不改变867出口，也不隐含无限减法承诺。
- T-01把本地attachments/844/probe/measure.py的经审查等价实现纳入新scripts/measure-source-size.py长期受管，留同输入复算与hash；后续不得依赖未入库探针，不顺带提交原始日志。

### 5.2 crate 检验清单（T-02）

- 范围：三仓全部 workspace 成员（lang 现成员 + ui 五包 + shell-pack + man 9 成员 + man `core/` 子 workspace），逐 crate 一行。
- 七栏定义：
  1. **职责**（一句话；与规范页 owner 对齐）
  2. **公开出口**（`pub` 面 + 是否 re-export 门面 + 是否 777 边界白名单）
  3. **重复实现**（与哪些 crate 有疑似重复；引用 844 已清偿族与残留候选）
  4. **无消费者旧路径**（DEPRECATED/兼容/过渡层，附消费者证据）
  5. **依赖方向**（上游/下游 + 是否违反禁向边）
  6. **测试与档位**（867 的档位归属 + 选择数下限 + ignore 面）
  7. **文档与规范**（spec 页、是否滞后于代码）
- 产物：`docs/specs/repository-boundaries/crate-inventory.md`（人读）+ `scripts/crate-inventory.json`（机读，含生成命令与 `cargo metadata` 指纹）+ 差异检查脚本（清单与仓库不一致即红）。
- 排序输出：按"可减行数 × 风险倒数 × 证据完备度"给出候选队列，交 T-03 冻结。

### 5.3 第二批减法（T-03/T-04）

- 候选来源：① 844 残留候选（F-10）；② 清单第 3/4 栏；③ 新发现（只在有消费者证据时进入）。
- 每项按 844 四类处置：MIGRATED／RENAMED／RETIRED／LOST；本批预期以"真重复族"与"无消费者退役"为主。
- 负控五面（缺一即不得删）：cfg/target/feature 矩阵、宏与字符串注册、脚本/CI、直接 manifest 引用、公开承诺（文档/版本说明/外部 consumer）。
- **结构性减法（已授权，2026-10-28）**：退役"活跃但有替代"的面。逐项四要件：① 替代路径真实存在（消费者已切或切点已实测）；② 公开承诺可迁移（版本/迁移说明/拒绝行为具名）；③ 删除后的真实消费者有定向验证，最终三仓完整档按867口径在final review后由merge核；④ 用户按批签核记录。T-03先提交候选证据/替代与迁移/定向验证方案供批签核，T-04实施后补定向正控，最终整档是合入门，不要求签核前跑整档或把删后验证当删前条件。四要件全部闭合前不得交付。注意：`.at` 资产（`lib-legacy`、aavm）不影响 `.rs` 指标，不能用来"凑"A 口径；真正能压 A/B 的是 `.rs` 子系统。
- 收据：`attachments/869/subtraction-receipt.md`（NET-PROD 前后、族/项清单、测试处置、复算命令、负控输出、**每批净删 ≥ 2,000 行的校验结果**）。
- **可恢复性（强制）**：记删除提交SHA、父提交/保留ref、删除前hash与恢复配方（git log --diff-filter=D、git show父提交路径、git log -S）；从本地保留提交或bundle实际恢复核hash。本地未推送提交也可恢复，但须有持久ref/可读对象或验证过的bundle；未跟踪字节另保全，不假称Git已保护。推送状态据实记未授权/未推送/已推送；推送仅为额外副本，本Plan不授权推送/历史过滤/强推，不设为U869条件，不绕过全部在途worktree合并后的证据历史过滤安排。

### 5.4 结构立项包（T-05）

两个后续结构工作包暂不编号（874/875仅讨论暂称，不占号不建文件）；逐项区分843原目标/实际交付/旧残项或验收差额/新增治理，避免重复计功。语言包等U869、U872及U873结论；UI/Man包保守后继。868/870/871按真实文件/符号/接口交接，不因native同名全局等待。每个热点一页：现状规模（P total 与生产行两口径并列）／职责与耦合／拆分假设（含是否需要 probe）／触面档（t/tv/tt/tu/tb/taa/man/ui）／前置（867 基线）／主要风险（例如 `call_inner` 的早期 return 语义、UI renderer 的 GPU 状态机）／审查面（是否需要独立 GPU/像素证据）。

### 5.5 执行合同（work/review/merge 共用）

| 字段 | 内容 |
|---|---|
| 规则与边界 | G1–G5；减法与清单；不改语义；不动保留有证项；三仓各一树 |
| 实际路径 | `measure.py` → 三口径数 → 清单 → 候选 → 负控 → 删除 → NET-PROD 收据 → 867 门复跑 → 立项包 |
| 所有权与状态 | 口径与清单为验收输入（改需原因+哈希）；各 crate owner=其仓；收据 owner=语言仓编排 |
| 区分性用例 | V-01..V-06：口径复算、清单缺栏、负控拦截、搬移=0 负控、删除测试无据、立项包缺项 |
| Oracle 与关闭 | 每 AC 绑命令与期望值；关闭需实际输出与哈希；review/merge pending 显式在档 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/repository-boundaries/net-slimming.md | 增"80 万目标已定口径（A 达标 / B 不反弹 / 三分列）+ 批次收据格式 + 对外表述规则" | 目标原先无可判定分母 | AC-01 |
| SD-02 | add | docs/specs/repository-boundaries/crate-inventory.md | 无统一 crate 检验清单 → 三仓成员 × 七栏 + 生成/差异检查规则 | roadmap §2.1 的下半部无承载 | AC-02 |
| SD-03 | modify | docs/specs/repository-boundaries/project.md | 拆分完成后的常态 = 按 crate 检验清单 + 批次减法 + 结构立项（指向 net-slimming / crate-inventory） | 旧页只写到 M2 出口 | AC-02, AC-06 |

> `touched_goals` 留空：80 万是 roadmap 的争取目标，本计划只定分母与披露方式，不改 GOAL。

## 6. 测试设计

`owner_stage` 均为 `work`；本轮未执行。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 口径可复算 | 同组三仓commit、分类版本；历史脚本及新受管等价工具 | T-01交付的scripts/measure-source-size.py（新） | 同输入两次一致，旧新工具差额全归因；A/B/C同源且覆盖/排除显式，搬移0净删 | 摘要/hash/差异表 | P/T2/XT/G/VEND，XO单列不暗改范围 |
| V-02 | AC-02 清单完备 | 三仓 `cargo metadata` | 清单生成 + 差异检查 | 成员集合与 metadata 完全一致；任一成员缺栏即非零 | 清单 JSON + 退出码 | 全部 workspace 成员 |
| V-03 | AC-03 负控有效 | 构造 3 类误删：cfg 门后消费者、字符串注册消费者、脚本/manifest 引用 | 清单负控脚本 | 三类均被拦；真正无消费者项放行 | 负控输出 | 三仓各一类 |
| V-04 | AC-03 收据口径 | 搬移一个文件（不改内容）、新增一个 helper、删除一个测试 | `measure.py` 前后 + 分列 | 搬移=0 净删；helper 自抵；测试/生成/第三方分列不计主指标 | 收据前后表 | 同 844 V-04 |
| V-05 | AC-04 测试处置 | 有证退役与无证删除各一控制 | work定向867闭合账本；merge完整档 | 有证RETIRED可接受；无证LOST阻塞；编译红/未执行不能伪装退役 | 账本/定向输出/最终收据 | 867全部义务 |
| V-06 | AC-06 立项包完备 | 6 个热点 | 立项包检查脚本 | 每包七要素齐全（规模/职责/假设/触面档/前置/风险/审查面）；缺项即红 | 检查输出 | 已登记热点 |
| V-07 | AC-03 可恢复性 | 从本地保留ref或bundle恢复被删文件，核未推送状态 | git show父提交路径 / bundle verify及恢复 | hash一致；未推送不判不可恢复，未跟踪未保全必须红；无需网络写入 | SHA/ref/bundle hash摘要 | 每个实际删源仓至少一例 |

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | A/B/C同源复算、范围与时点明确，分类/脚本指纹齐；net-slimming口径proposal按A达标/B不反弹/分列，canonical仅merge；不要求本Plan达到80万 |
| AC-02 | work | `crate-inventory.md` + `scripts/crate-inventory.json` 覆盖三仓全部成员 × 七栏，零缺项零多余；差异检查可复跑 |
| AC-03 | work | NET-PROD净删≥2,000，至少1真实重复/退役项，负控五面和具体结构性减法签核/替代/承诺迁移/定向消费者证据齐；恢复SHA/配方/状态齐，完整档最终证明归AC-08 |
| AC-04 | work | 删除涉及的测试 ID 全部在 867 账本内为 RETIRED 有据；无 LOST |
| AC-05 | work | 删除后867总账/基线、定向消费者与账本反控通过；未解释缺口=0；完整档配置就绪，显式merge pending，不在work重复整档 |
| AC-06 | work | 结构立项包覆盖 6 热点，各七要素齐全、可被独立执行 |
| AC-07 | review | 独立终审绑定三仓commit/依赖/输入，核V-01..V-07、SD-01..03、口径、恢复演练及删除合理性；merge门pending |
| AC-08 | merge | final review后冻结三仓统一实际完整档+必要触面档，同输入同选面可复用不重复；继承867既存红认证/零新增红/必需未运行阻塞；SD/ledger/归档/guard闭环才U869，不以推送或80万达标为出口 |

## 8. 执行步骤

10 项 = 8 work + 1 review + 1 merge；G869=U867 对全部行生效。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | G869 | 同源A/B/C复算，迁移等价度量至新scripts/measure-source-size.py并验证；冻结口径proposal与2,000/恢复收据模板，canonical仅merge | AC-01 |
| [x] | T-02 | work | G869；T-01 | 清单生成器 + `crate-inventory.md` + `crate-inventory.json` + 差异检查；输出候选队列 | AC-02 |
| [x] | T-03 | work | G869；T-02 | 第二批候选冻结（族/项/负控五面证据；结构性减法项另备四要件），**送用户按批签核**（不实施） | AC-03 |
| [x] | T-04 | work | G869；T-03 | 实施减法：删除/收敛 + 测试处置 + NET-PROD 收据 | AC-03, AC-04 |
| [x] | T-05 | work | G869；T-04 | 两个未编号后续结构工作包立项材料；区分843旧残项/新目标、872接口与873决策输入；不占号不建874/875 | AC-06 |
| [x] | T-06 | work | G869；T-04,T-05 | 867义务总账/退役证据、相关定向反例与真实consumer；完整档merge pending，不在work跑整档 | AC-05 |
| [x] | T-07 | work | G869；T-06 | SD-01..03 proposal与handoff冻结送审；work不发布canonical | AC-01,AC-02,AC-06 |
| [x] | T-08 | work | G869；T-07 | 全 AC/V 自检与遗漏扫描；未闭项列 finding/pending | AC-01..AC-06 |
| [x] | T-09 | review | G869；T-08 冻结提交 | 独立复审（含负控重放、口径复算、清单差异、删除合理性） | AC-07 |
| [x] | T-10 | merge | G869；T-09 pass | 冻结输入上三仓完整档 + 触面档；SD/ledger、归档、清理 | AC-08 |

**实施树**：`D:/autostack/.wt/lang-869/{auto-lang,auto-ui,auto-man}`，分支 `plan-869-dev`；auto-down 仅在组合消费时 detached 固定版本。禁止 junction/symlink；清理前逐树 `wt-guard.sh` clean。

**门禁**：work/review仅必要check、定向测试与反例；final review后merge冻结统一实际完整档+适用触面档。语言tu随867退役后不得跑空别名，生成面由auto-ui/tu承接；tf批量仍语言主检出单实例。同输入同选面覆盖只跑一次，变动须复审重冻结。

## 9. 复审记录



本轮合同预检（非实现复审/测试）：章节0–10、单一任务表、任务DAG无环、编号唯一、AC/V/SD引用及frontmatter依赖检查通过；本Plan 10任务/8 AC/7 V/3 SD。来源内容hash：`docs/specs/repository-boundaries/net-slimming.md` sha256 `1efb2d1ea1a43e633cc97d7fb61bbc2936aff04519b34548672e720147cd60dc`。仅文档校验，未运行Cargo/docs_gen。

2026-10-28 / stage: new / plan_id: PLAN-869 / plan_revision: 4 / outcome: blocked（合同就绪，实施等U867） / authorization: 用户本会话修订授权 / input_sha256: `c9ac84bb54ccf6736571c0fd36f52dc3820eab401b7c786a820dad52bb0311f7` / changed: A/B范围时点、2,000批粒度、恢复/推送边界、AC-05/07/08、V-07、T-05/06/07及未编号工作包门 / progress: drafting 0/10保持 / next: U867后T-01；已裁定口径/2,000不再询问，具体结构性减法批仍继承T-03签核。旧“同口径165批”“必须推送”仅历史。未实施/跑Cargo/改canonical/ledger/建树/推送；原始证据本地留存，md摘要含命令/退出码/数字/路径+hash。

2026-10-28 / stage: new / plan_id: PLAN-869 / revision: 1 / outcome: **pass（合同可执行；实施等 U867）** / next: 867 归档后 work T-01（口径点选需用户在 T-01 结果上确认）/ changed: 新建 T-01..T-10、AC-01..AC-08、V-01..V-06、SD-01..SD-03。

- 取号：`scripts/new-plan.sh crate-inventory-and-subtraction-batch-2`（master 原子取号 869 → 870；原提议的"868"已被另一会话的 `868-hir-mir-lowering-shared-crate` 占用，故顺延）。
- 本轮只写合同：未跑 `measure.py`、未生成清单、未做候选判定、未建树、未改实现/规范/ledger。
- 阻塞项：G869（=U867）未满足；D-1 已裁定（见下条 r2 记录），不再是阻塞项。

2026-10-28 / stage: new（r2 语义修订） / plan_id: PLAN-869 / plan_revision: 2 / outcome: **pass（口径与结构性减法授权已裁定，仍等 U867）** / ruling: ① 80 万口径 = **A 对外披露/达标 + B 不得反弹 + 三分列**（D-1 关闭）；② **结构性减法获授权**：逐项四要件（替代路径/承诺迁移/门禁绿/用户按批签核，D-3 关闭）；③ D-2（每批最小净删额）仍待 T-01 给建议值 / changed: §0、§1 G1/G3、非目标、§2 口径层、§5.1、§5.3、AC-01/AC-03、T-01/T-03、§10 D-1/D-3/D-4 / next: U867 后 work T-01。

2026-10-28 / stage: new（r3 修订） / plan_id: PLAN-869 / plan_revision: 3 / outcome: **pass（D-2 已裁定；仍等 U867）** / ruling: 每批 **NET-PROD 净删 ≥ 2,000 行**（口径=net-slimming 主指标；不足则合并到下一批，不得用删测试/生成物/改分类凑数；结构性减法批同套用）；收据强制含**删除提交 SHA + 恢复配方 + 推送状态**，并新增 V-07 恢复演练 / changed: §5.3、AC-03、T-01、V-07、§10 D-2 / next: U867 后 work T-01。


2026-10-28 / stage: work / plan_id: PLAN-869 / plan_revision: 4 / outcome: **pass（work 全 8 任务完成，execution_done → review）**
- worktree：`D:/autostack/.wt/lang-869/{auto-lang,auto-ui,auto-man}`（分支均 plan-869-dev；auto-down detached c3464d0 未动）
- 提交链（auto-lang）：a9c5af389（T-01 度量脚本+口径）→ 42e235405（T-02 清单）→ 70bff36fb（T-04 删减）→ 5105243a7（T-04 收据/账本/下限）→ e0e72e772（T-05 立项包）→ 691e3f3a9（T-06/07）→ 668297c8f（债务/交接）；auto-ui 4779236；auto-man e6cc06f
- **AC-01..AC-06 全过**（证据）：A/B/C 复算 A=1,001,334→998,497、B=628,573→**625,753（净删 −2,820 ≥2,000）**、C 仅历史；47 crate×七栏清单+差异检查（缺栏/多余双负控红）；负控五面（websocket 宏注册消费者实证拦截、cosmic 文档承诺面拦、V-03 类1 防护链说明）；16 测试 ID RETIRED（lang-t 3198→3182 恰 −16，LOST=0，ui-tu 863/man-t 281 不变）；档位守卫 11 档全绿；立项包 7×7+检查 oracle（负控红）；V-01/02/04/05/06/07 全绿（V-03 两类实证一类防护链）
- 三仓 cargo check 零错、零新增警告（stash 基线归一化对照）；提交内文件 fmt-clean；零新增调试打印
- merge-owned pending（显式）：冻结提交上裸 `cargo t` + 触面档（tt 触 trans / tb 不涉 / tv 语料 / ui t+tu / man t+tc+ti / taa 仅当 aavm 触发线——本批未触 packages/aavm，预计免）；SD-01..03 canonical 化；ledger/归档/清理
- 未决用户决策（不阻塞 review）：auto-cosmic 结构性退役候选四要件已备（subtraction-freeze §4），待按批签核（D-3）
- next_responsible_role: review（T-09）

2026-10-28 / stage: goal / plan_id: PLAN-869 / plan_revision: r4 / round: 1 / from: drafting / to: review / status: archived / outcome: pass / worktree: D:/autostack/.wt/lang-869/auto-lang / commit: 668297c8f / next: review

2026-10-28 / stage: review / review_scope: final / reviewer: 同会话复审（独立性限制：与实施同 session，结论按工件重建非采信执行摘要——oracle 全部重跑或复算）/ plan_id: PLAN-869 / plan_revision: 4 / outcome: **pass（final；merge-owned 门显式 pending）**
- worktree：D:/autostack/.wt/lang-869/auto-lang @plan-869-dev；reviewed_commit `6882292cd`（修复后）；base `a27435014`；deps：auto-ui `4779236`、auto-man `e6cc06f`、auto-down c3464d0（detached，零触面）
- **发现与处置**：
  - F-869-R1-01（已修）：清单 md/json 绑定删减前基线指纹，--check 冻结态红 → work 修复 `6882292cd` 重生成（CHECK PASS 复验）。
  - F-869-R1-02（预存红归因，非本批）：auto-lang-core lib 12 失败族（plan729×4/plan730×7/plan698×1/plan738×1，两次全量运行 15↔12 时序浮动）——**基线 a27435014 同族对照复跑 13 失败（超集）**，环境绑定预存红，与批 2 删除无关（本批未改其运行时行为；删除项均零消费者编译期退场）。记录在案，merge 整档若再现按预存红归因。
  - 观察（非 finding）：ui ui_gen 族 861 pass + 2 ignored = 863（与下限一致；867 记 0 ignored 为 nextest 口径差）。
- **AC 复核**（工件重建）：AC-01 ✓（measure 重跑 306,284/295,011/24,458、B=625,753、A=998,497 复现）；AC-02 ✓（47 成员、--check PASS、缺栏/多余负控红在案）；AC-03 ✓（−2,820 ≥2,000；五面负控含 websocket 实证拦截；恢复演练 sha256 在案）；AC-04 ✓（16 RETIRED、LOST=0、3198→3182 恰 −16）；AC-05 ✓（档位守卫 11 档绿 t06-tier-guard.txt；未解释缺口 0）；AC-06 ✓（7 包×7 要素 check PASS+负控红）。
- **行为面**：auto-val+auto-ast lib 190/190 绿；auto-lang-core lib 1,595+/12 预存红（归因如上）；ui ui_gen 861 绿。
- **规范增量**：SD-01..03 proposal 审核通过（目标路径真实、规则与实现一致、canonical 未动——merge 落）；SD-02 已随分支建页。frontmatter spec-impact 三字段齐备（touched_goals 空有书面理由）。
- **merge-owned pending（T-10）**：冻结提交裸 `cargo t`（lang）+ 触面档（tv/tt/tb/tu 归 ui 仓 t+tu/man t+tc+ti；taa 免——未触 aavm 触发线）+ 三仓档；SD canonical/ledger/归档/wt-guard 清理。
- next: merge

2026-10-28 / stage: goal / plan_id: PLAN-869 / plan_revision: r4 / round: 2 / from: review / to: merge / status: archived / outcome: pass (final; F-869-R1-01 fixed in 6882292cd; F-869-R1-02 pre-existing reds attributed to base) / worktree: D:/autostack/.wt/lang-869/auto-lang / commit: 6882292cd / next: merge

2026-10-28 / stage: merge / plan_id: PLAN-869 / plan_revision: r4 / outcome: **pass（delivered；U869 闭环）**
- prepared→landed→ledger_refreshed→archived→cleaned 全检查点在案
- **landed**：三仓 ff-only——auto-lang master=`fee385de2`（含 rebase onto 89c76fd8c，range-diff `=` 补丁等价证明）、auto-ui master=`154a396`、auto-man master=`e6cc06f`；无 merge commit
- **门禁终值**（冻结 tip）：lang 裸 t 3,185/3,185 + tv 163 + tt 3,556；ui t 2,443/2,443 + tu；man t/tc/ti 43/43（tb/taa 未触面免）；门禁修复 F-869-M1（a2r-std 平价面 19 行）、F-869-M2（rust_ui 夹具 8 行）均已修复+定向+全档复验
- **ledger_refreshed**：`.autoos/specs.json` v121→v122（guarded offline projection，备份+外科 diff 84+/1−+回读校验；designs×2/tests×1/reviews×1/reports×1，P869-*）
- **SD 落地**：net-slimming 800k 口径节 + crate-inventory.md（INDEX 已刷）+ project.md 常态治理节；canonical=merge 落（work 零改）
- **交付核数**：NET-PROD 合入视角净删 **−2,799**（lang −1,433/ui −1,294/man −72；组基线口径 −2,801 并在案）；A=998,960@合入基（批效果 −2,818 组基线口径）；16 测试 ID RETIRED、LOST=0、lang-t 下限 3182；47 crate 清单 + 7×7 结构立项包
- **cleaned**：三树 wt-guard clean 后移除（见下条目标记）
- 批量回归到期检查：869 % 5 ≠ 0；.last-batch-regression 回执 2026-10-28（<48h）→ **不到期**
- 陈旧产物观察：本批为库/工具面，无运行中生产进程；release 二进制/web bundle 未随本批重建（下次发布构建自然吸收，非本批出口）

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | ~~"80 万"用哪个分母~~ **已裁定（2026-10-28）**：A 对外披露/达标 + B 不得反弹 + 测试/生成/第三方分列；C 仅历史对照 | G1 与全批收据口径 | 关闭：T-01 按此复算并写回 `net-slimming.md` |
| D-2 | ~~批次是否设最小净删额~~ **已裁定（2026-10-28）**：每批 **NET-PROD 净删 ≥ 2,000 行**（口径=net-slimming 主指标，不含搬移/测试/生成/第三方/资产）；不足则**合并到下一批**，不得用删测试/删生成物/改分类凑数；结构性减法批同样套用该下限 | 批次拆分粒度与达标节奏 | 关闭：T-01 把 2,000 行下限写进收据模板与校验 |
| D-3 | ~~是否允许"结构性减法"~~ **已授权（2026-10-28）**：允许，逐项四要件 + 用户按批签核（见 §5.3） | 决定能否接近 A 口径 20 万缺口 | 关闭：候选入 T-03，签核后 T-04 实施 |
| D-4 | A 口径若在发行时仍未达 80 万，如何处置：按 roadmap 报告实际规模与剩余项／追加结构性减法批次／重定分母 | 目标可行性 | 发行前由用户裁定；本计划只保证口径与进度可披露，不承诺按期达标 |
| D-5 | 清单产物落规范页还是 `docs/reports/` | 可发现性与 ledger 影响 | 倾向 `docs/specs/repository-boundaries/crate-inventory.md`（SD-02），T-02 确认 |

G869 未满足前禁止建树/改实现；若实施中发现 80 万目标在当前授权内不可达（需退役活跃面），按 §10 D-3/D-4 上送，不在本计划内静默缩面或重定目标。

