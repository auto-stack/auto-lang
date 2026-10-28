---
plan_id: PLAN-869
status: drafting
feature_name: 三仓按 crate 检验清单与净瘦身第二批（含 80 万目标口径冻结）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1
current_step: 0
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

844 已把 M2 收口（U844 成立，M2 四份全 archived，ledger v114），但其生产净删**终值只有 −1,221 行**（lang NET-PROD 308,909→307,688），真重复清偿 5 族。按同一口径，到"80 万行"还差约 **20 万行**；以 844 的效率推算需要约 165 个同规模批次——**说明 80 万目标不能靠一次性去重达成，必须先冻结分母、再按 crate 逐项清账**。

同时，844 留下的"残留候选"（error-reply 采样族 ×10、`time_now`、`math` 20+ 族、`str.len`、`nv_to_value`）与 843 留下的结构残留（`call_emit::call_inner` 7,439 行 if 链、`stdlib.rs`/`http_server.rs`/`trans/rust.rs` 三入口、`lib.rs` async glue）**都没有 owner**；UI 面（NET-PROD 296,190，最大文件 `renderer.rs` 42,337）从未做过结构或减法审视。

本计划做三件事：**① 冻结"80 万"的三种并列口径并交付点选材料；② 产出三仓全部 workspace 成员 × 七栏的 crate 检验清单；③ 按清单执行第二批减法并用同口径收据证明真实生产净删**，同时把结构热点排成可执行的结构计划包（不在本计划做手术）。

## 1. 目标

- **G1 口径可判定**：把"约 100 万 → 80 万"落成三种并列口径（A 全部 `.rs`／B 生产 NET-PROD／C 旧"单 crate 行数"），各自当前值与差额用 844 的 `measure.py` 同脚本复算；**达标口径由用户点选**；未点选前只披露、不宣布达标/未达标。
- **G2 清单齐全**：三仓全部 workspace 成员 × 七栏（职责／公开出口／重复实现／无消费者旧路径／依赖方向／测试与档位／文档与规范），缺栏即红；该清单成为后续减法批次与结构立项的单一事实源。
- **G3 批次真减法**：按清单冻结候选（真重复族／无消费者退役／过渡层），实施后以 NET-PROD 收据证明**真实生产净删 > 0** 且至少 1 个真重复族或退役项闭合；沿用 844 的四类处置与 V-04 负控（搬移=0、helper 入账、测试/生成/第三方分列）。
- **G4 测试联动**：任何删除导致的测试 ID 消失，必须在 867 闭合账本里落为 **RETIRED 且有证据**；867 的三仓完整档在本计划冻结组合上复跑通过。
- **G5 结构立项就绪**：把已登记热点（语言 `call_inner`、三入口、`lib.rs`；UI `renderer.rs`/`vue.rs`/`aura_view_builder.rs`；Man `auto/main.rs`）排成结构计划包，每个热点给出规模/职责/拆分假设/触面档/前置/风险/审查面，**本计划不做结构手术**。

**非目标**：不在本计划拆 `call_inner` 或 UI 巨文件（只立项）；不改产品语义（`math`/`str.len`/`time_now`/`error-reply`/`nv_to_value` 归后续语义统一计划，需用户裁定方向）；不动 844 的"保留有证"项（`lib-legacy`、`type_registry` REPL 面、aavm 全家、`str.len`/`math` 双轨、auto-api-gen 跨仓单源、auto-man 兼容门面）；不做 workspace member 拆分（THR-D1 归测量计划）；不做数据域/运行时域独立仓；不重定 80 万这个目标数值本身。

## 2. 架构方案

1. **口径层**：在 `net-slimming.md` 现有桶定义上增加"三种分母口径"，用 `measure.py` 一次复算出三组当前值/差额，形成点选材料（含每口径的语义、可得性、是否包含测试/生成/第三方）。点选结果写入规范，之后所有批次按该口径出收据。
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
| F-4 | 若"80 万"按 F-3 口径，缺口 **≈201,786**；按 844 的 1,221/批需 ~165 批 → 该目标必须换杠杆或重定分母 | F-1/F-3 算术 |
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
| A 全 `.rs` | F-3 全部桶（生产+文件测试+测试根+生成+第三方；不含 examples/experimental/website） | ≈1,003,007 | ≈201,786 | 覆盖面最广；但删测试/生成不算真瘦身（roadmap 明令禁止以删验收达标） |
| B 生产 NET-PROD | `net-slimming.md` 主指标三仓合计 | 628,408 | 已低于 80 万 | 只反映生产手写代码；与"100 万"基准不同分母 |
| C 旧"单 crate" | THR-D1 记载的 2026-10-02 单 crate 76.6 万行/1,650 文件（拆分前形态） | 拆分后该口径已不成立 | — | 仅历史对照，不可作为当前目标分母 |

- T-01 产出：三案复算输出 + 点选材料（每案语义、是否可判定、达标需要什么手段）+ **用户点选记录**；点选前所有报告并列披露三案。
- 点选后写回 `net-slimming.md`；未点选不得在 issue/交付说明里宣布"达标"或"未达标"。

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
- 收据：`attachments/869/subtraction-receipt.md`（NET-PROD 前后、族/项清单、测试处置、复算命令、负控输出）。

### 5.4 结构立项包（T-05）

每个热点一页：现状规模（P total 与生产行两口径并列）／职责与耦合／拆分假设（含是否需要 probe）／触面档（t/tv/tt/tu/tb/taa/man/ui）／前置（867 基线）／主要风险（例如 `call_inner` 的早期 return 语义、UI renderer 的 GPU 状态机）／审查面（是否需要独立 GPU/像素证据）。

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
| SD-01 | modify | docs/specs/repository-boundaries/net-slimming.md | 增"80 万目标三种分母口径 + 点选规则 + 批次收据格式"；明确未点选只披露 | 目标当前无可判定分母 | AC-01 |
| SD-02 | add | docs/specs/repository-boundaries/crate-inventory.md | 无统一 crate 检验清单 → 三仓成员 × 七栏 + 生成/差异检查规则 | roadmap §2.1 的下半部无承载 | AC-02 |
| SD-03 | modify | docs/specs/repository-boundaries/project.md | 拆分完成后的常态 = 按 crate 检验清单 + 批次减法 + 结构立项（指向 net-slimming / crate-inventory） | 旧页只写到 M2 出口 | AC-02, AC-06 |

> `touched_goals` 留空：80 万是 roadmap 的争取目标，本计划只定分母与披露方式，不改 GOAL。

## 6. 测试设计

`owner_stage` 均为 `work`；本轮未执行。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 口径可复算 | 三仓冻结 commit；同脚本 | `python docs/plans/attachments/844/probe/measure.py`（或受控扩展） | 三案数值与 F-1..F-4 一致或差异逐项解释；两次运行一致 | 输出 JSON + 哈希 + 差异表 | 三仓；不含 examples/experimental（另有桶） |
| V-02 | AC-02 清单完备 | 三仓 `cargo metadata` | 清单生成 + 差异检查 | 成员集合与 metadata 完全一致；任一成员缺栏即非零 | 清单 JSON + 退出码 | 全部 workspace 成员 |
| V-03 | AC-03 负控有效 | 构造 3 类误删：cfg 门后消费者、字符串注册消费者、脚本/manifest 引用 | 清单负控脚本 | 三类均被拦；真正无消费者项放行 | 负控输出 | 三仓各一类 |
| V-04 | AC-03 收据口径 | 搬移一个文件（不改内容）、新增一个 helper、删除一个测试 | `measure.py` 前后 + 分列 | 搬移=0 净删；helper 自抵；测试/生成/第三方分列不计主指标 | 收据前后表 | 同 844 V-04 |
| V-05 | AC-04 测试处置 | 删除一个含测试的退役项 | 867 闭合账本 + 三仓门 | 该 ID 必须为 RETIRED 且有证据；无证据即 LOST 阻塞 | 账本条目 + 门输出 | 867 覆盖的档 |
| V-06 | AC-06 立项包完备 | 6 个热点 | 立项包检查脚本 | 每包七要素齐全（规模/职责/假设/触面档/前置/风险/审查面）；缺项即红 | 检查输出 | 已登记热点 |

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 三口径数值复算一致，点选材料齐备；`net-slimming.md` 记录口径与点选状态；未点选不宣布达标 |
| AC-02 | work | `crate-inventory.md` + `scripts/crate-inventory.json` 覆盖三仓全部成员 × 七栏，零缺项零多余；差异检查可复跑 |
| AC-03 | work | 第二批减法闭合：生产净删 > 0，至少 1 真重复族/退役项，负控五面留证；收据含 NET-PROD 前后 |
| AC-04 | work | 删除涉及的测试 ID 全部在 867 账本内为 RETIRED 有据；无 LOST |
| AC-05 | work | 冻结组合上 867 三仓完整档复跑通过；新增红=0 |
| AC-06 | work | 结构立项包覆盖 6 热点，各七要素齐全、可被独立执行 |
| AC-07 | review | 独立终审绑定三仓 commit/依赖/输入，核 V-01..V-06、SD-01..SD-03、清单/收据/负控/遗漏与健康面；merge 门 pending |
| AC-08 | merge | 并集影响复审通过；冻结输入上跑三仓完整档 + 触面档；SD/ledger 沉淀、归档与逐树 guard clean 清理 |

## 8. 执行步骤

10 项 = 8 work + 1 review + 1 merge；G869=U867 对全部行生效。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G869 | 三口径复算与点选材料（含用户点选记录）；`measure.py` 若需 per-crate 切片，先补受控扩展并留负控 | AC-01 |
| [ ] | T-02 | work | G869；T-01 | 清单生成器 + `crate-inventory.md` + `crate-inventory.json` + 差异检查；输出候选队列 | AC-02 |
| [ ] | T-03 | work | G869；T-02 | 第二批候选冻结（族/项/负控五面证据），交 review 预检（不实施） | AC-03 |
| [ ] | T-04 | work | G869；T-03 | 实施减法：删除/收敛 + 测试处置 + NET-PROD 收据 | AC-03, AC-04 |
| [ ] | T-05 | work | G869；T-04 | 结构立项包（`call_inner`/三入口/`lib.rs`/UI 三巨文件/man main.rs），含 probe 定义与触面档 | AC-06 |
| [ ] | T-06 | work | G869；T-04,T-05 | 867 门复跑：三仓完整档 + 红集对齐；新增红=0 | AC-05 |
| [ ] | T-07 | work | G869；T-06 | SD-01..SD-03 落地 + `attachments/869/handoff.md` 冻结送审 | AC-01, AC-02, AC-06 |
| [ ] | T-08 | work | G869；T-07 | 全 AC/V 自检与遗漏扫描；未闭项列 finding/pending | AC-01..AC-06 |
| [ ] | T-09 | review | G869；T-08 冻结提交 | 独立复审（含负控重放、口径复算、清单差异、删除合理性） | AC-07 |
| [ ] | T-10 | merge | G869；T-09 pass | 冻结输入上三仓完整档 + 触面档；SD/ledger、归档、清理 | AC-08 |

**实施树**：`D:/autostack/.wt/lang-869/{auto-lang,auto-ui,auto-man}`，分支 `plan-869-dev`；auto-down 仅在组合消费时 detached 固定版本。禁止 junction/symlink；清理前逐树 `wt-guard.sh` clean。

**门禁**：含产品源码删除（Category B）→ 开发期 `cargo check -p <被改包>` + 定向测试；merge 在冻结组合上跑 867 三仓完整档 + 语言 `tv/tt/tu`/UI `tu/ti`/man `tc/ti` 触面档；批量回归按到期规则由主检出单实例执行。

## 9. 复审记录

2026-10-28 / stage: new / plan_id: PLAN-869 / revision: 1 / outcome: **pass（合同可执行；实施等 U867）** / next: 867 归档后 work T-01（口径点选需用户在 T-01 结果上确认）/ changed: 新建 T-01..T-10、AC-01..AC-08、V-01..V-06、SD-01..SD-03。

- 取号：`scripts/new-plan.sh crate-inventory-and-subtraction-batch-2`（master 原子取号 869 → 870；原提议的"868"已被另一会话的 `868-hir-mir-lowering-shared-crate` 占用，故顺延）。
- 本轮只写合同：未跑 `measure.py`、未生成清单、未做候选判定、未建树、未改实现/规范/ledger。
- 阻塞项：G869（=U867）未满足；D-1（80 万口径点选）需用户在 T-01 输出后决定。

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | "80 万"用哪个分母（A 全 `.rs`≈100.3 万／B 生产 NET-PROD≈62.8 万／C 旧单 crate 76.6 万） | 决定 G1 与全批收据口径 | T-01 出点选材料，用户点选后写回 net-slimming.md；未点选只披露 |
| D-2 | 批次是否设最小净删额（如每批 ≥2k） | 决定批次拆分粒度 | 建议 T-01 一并给建议值，用户确认 |
| D-3 | 是否允许"结构性减法"（退役活跃但有替代的面，如 `lib-legacy`/aavm/playground） | 决定能否接近 20 万缺口 | 需用户明确授权；未授权前只做无消费者退役与真重复清偿 |
| D-4 | 若 A 口径不可达（165 批不现实），是否改为"结构性减法 + 分母重定"组合 | 目标可行性 | 与 D-1/D-3 合并由用户裁定 |
| D-5 | 清单产物落规范页还是 `docs/reports/` | 可发现性与 ledger 影响 | 倾向 `docs/specs/repository-boundaries/crate-inventory.md`（SD-02），T-02 确认 |

G869 未满足前禁止建树/改实现；若实施中发现 80 万目标在当前授权内不可达（需退役活跃面），按 §10 D-3/D-4 上送，不在本计划内静默缩面或重定目标。

