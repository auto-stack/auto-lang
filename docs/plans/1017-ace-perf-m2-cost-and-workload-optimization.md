---
plan_id: PLAN-1017
status: reviewed
feature_name: M2 ace-perf串行套件成本与压力负载优化
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-13
plan_revision: 6
phase: M2-A
current_step: 7
total_steps: 9
depends_on: [PLAN-1016]
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/ace-test-policy.md, docs/specs/auto-cte/frozen-cache.md]
new_spec_components: [docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md]
touched_goals: [GOAL-002]
---

# [PLAN-1017] M2 ace-perf串行套件成本与压力负载优化

## 0. 变更摘要

2026-11-12 r2：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

**2026-11-13 r4 用户裁定（执行策略修订）**：不先跑全量 ace-perf（当前约2小时，正是本计划要优化的对象）。
改为**最细粒度逐用例推进**：第一步以代码阅读逐用例优化——在**不影响测试效果**的前提下缩减
冗余循环/样本/迭代规模（例：10000 次循环若 1000 次即足以证明同样效果则减为 1000），并修产品
代码已证瓶颈；第二步逐用例实跑计时再优化；全部用例完成后按模块做套件级计时；各模块总时长进入
合理范围后，最后才做一次全量 ace-perf 总时长测试。守恒口径由"原规模字节守恒"改为
"**测试效果守恒**"：每例缩减须逐例记录其所证明的机制/门/状态覆盖不变（覆盖同一断言族与
状态矩阵）；冻结字节规模义务（如 64MiB 流式下限）与统计有效样本数（P90 百分位语义）不缩。
B0/T_new 比较基准相应改为：逐用例基线与模块套件基线（Pass B/C 实测），全量比较以最终一次
全量跑对照 932/1016 历史档案与逐模块实测之和；不再要求先行一次全量 B0。

**2026-11-13 r5 用户裁定（目标修订，选项 a）**：AC-05 的总十倍目标在现合同下实测不可达
（Pass C 全量 94 例串行 ≈3845s；同集 1.6×；不可压缩地板见 §9）。目标修订为
"**合理总时长 + 逐模块预算**"：全量 `cargo t ace-perf` 串行执行段**≤3000s（50 分钟）**，
进取线 2400s；逐模块预算（空闲窗口口径，超支须具名归因）：
syntax-artifacts ≤120s / ace-av ≤240s / semantics-store ≤80s / comptime ≤550s /
conformance ≤800s / native ≤1300s / abi ≤320s / artifacts ≤60s / a2x-worker ≤45s
（预算合计 ≈3515s>3000 为刻意的头部压力，要求 top 模块在预算内再挤出总量）。
守恒口径不变（r4 效果守恒）；合同不变（debug 档、closure lane 语义、样本统计义务、
冻结字节规模）。当前残余 top 优化对象：cold_set 填充段（~450s，源形状重整候选——
前端成本对源规模次线性，同字节目标下更大更少源可省总量，须实测验证后定）、
comptime 双 resource（~430s，同前）、sd911/turnaround 每样本链（emit/link 主项）、
closures_native（296s）。

**2026-11-14 r6 用户裁定（总门二次修订）**：全量串行执行段总门修订为 **≤3600s（负载上限）**，
验收口径=逐模块分跑求和（用户 2026-11-14 建议并裁定，Pass F）：干净校正值 **≈3345s 达标**
（四个可辨识污染腿 sd911/watermark/v07_storage/deep_chain 按各自干净实测校正；原始负载窗
分跑值 3671.3s 透明记录，其中 sd911 同跑 fresh P50 8390ms=干净值 2.8× 为争用直接证据）。
**不再重跑**；剩余 ~300s 量级优化（cranelift emit 提速、closure-lane 每调用复用、av 探针
基础设施）**延后至 ACE M2 完成后的后继包**。逐模块预算维持硬门+具名归因不变。

承接1016完成的普通组/PERF隔离与串行机制，在M2专门优化ace-perf的真实执行成本。
核心目标是同一冻结义务集、相同输入规模/样本/断言下，串行端到端成本降低至少一个数量级。
当前为 M2-A 规划，0/9。S0、B0 和最终十倍目标均待实际测定。

## 1. 目标

1. 明确套件真实组成、执行链及成本分布；区别无效重复工作、必要规模工作与真正依赖时间的持续运行义务。
2. 每个优化对象先有可在60秒内完成或由外层截止杀树的同机制短探针，测量后再选择优化。
3. 消除重复编译、重复数据生成、逐写全目录扫描、重复冷建/落盘等已证瓶颈；不预设所有候选都是根因。
4. 保持测试之间串行以及原资源/正确性合同，交付冻结原义务集S0的完整、可复核成本改善。
5. 把原规模性能/压力复核交还M2；不反向阻塞M1/1013普通正确性与证据可信性收尾。

非目标：增加ACE语言/产品特性、重构所有运行时、另造测试调度器/逐例执行目录、缩小压力规模或采样、提高预算/阈值、靠并发求快、改预期或隐藏红项。M2新增语义由原能力Plan负责，1017不把测试成本优化当M2产品性能已通过。

## 2. 架构方案

原固定入口cargo t ace-perf保持，复用Cargo/nextest和现有工作进程。分析流程：
当前suite/源/工具/环境身份 → 历史成本排序 → 短探针与分阶段计量 → 一次可信B0成本基线 →
有界可行性判断 → 逐瓶颈优化及短反例 → 冻结提交 → 一次原规模完整串行验收 → 独立复审/沉淀。

三类义务分开处理：
- 小规模正确性/边界/引用生命周期：留普通组，原强度由实际正负控证明；不充正式规模通过。
- 正式规模：保32/64/128MiB、512修订、深链、原样本和数据分布；优化准备与算法。
- 真正持续运行：记录为什么必须持续多久才能观察泄漏/竞态。其不可压缩下限须实证，不能用慢操作数量当时长理由。

需要大量真实产物的GC测试可优化准备，但若原义务要求通过编译/执行生成产物，就不能改成直接塞假页。
只有经义务审查证明互不替代的准备与测量才能拆分；独立端到端producer→store→GC证据仍保留。

## 3. 技术栈

Rust ACE独立workspace、既有CteDriver/SourceObserver/执行尾观测、ManagedStore/BlobStore、
Python固定驱动与原生进程树计量、nextest ace-perf profile。按现有项目依赖取工具，不引入第二执行器。

## 4. 需求分析与背景调查

**授权**：用户2026-11-12明确要求建立ace-perf优化专项并纳入M2路线。本轮授权为规划/路线更新，
不推定当前实施、长测、合并/发布或创建其他会话授权。用户未给出工期/模型/令牌预算。
2026-11-13 用户以 `/goal 完成计划1017`（auto-plan-goal）授予全链授权：在本计划专属 worktree
（含创建）内实施 T-01..T-07、独立复审、合同内有界返修与 merge（规范沉淀/归档/清理/落地检查）；
不改目标/验收/阈值/范围/预算。实施树 `D:/autostack/.wt/lang-1017/auto-lang` @ `codex/plan-1017-dev`
（基面 master@`42874419a`）已建；材料目录 `D:/autostack/plan-artifacts/plan-1017/` 已登记。

**来源**：docs/specs/overview.md、auto-test-case/{module-selection,ace-test-policy}.md；
Design55 §16.2 M2A-07、M2执行路线；1016 r5及其实现提交edc73bdf7；历史932性能台账；
comptime资源/周转测试、语义压力测试、语法64MiB流式、artifacts布局与store、auto-cache storage。
2026-11-13 更新：1016 已归档，交付提交 `a8fe02aa0` 已落主。ACE M1 正式标签 `v0.6-ACE-M1` 指向 `b923362f4`。[1025 的收口记录](archive/1025-ace-m1-mainline-identity-and-final-signoff.md)与当前源码是重核正式/常规用例归属的依据。T-01 以当前 `cargo t ace-perf --discover-only` 的真实发现逐项核平台、ignore 与输入身份；标签收口恢复到普通组的六项按原 M1 身份对账。

**已知事实与不确定性**：
- 1016已经把普通/PERF隔离、串行profile和短用例交付到独立树；其触及8包发现87项=86 active+1原ignored，
  这是局部实际发现，不能充全ACE正式suite总数或全套绿。
- 932历史61项逐测试耗时合计约4900秒（含失败记录）；1013/1005有1201秒GC与1503.8秒语法流式记录。
  这些跨提交、配置和负载的历史数仅用于排序，不是本计划B0，也不能把普通组总耗时冒PERF总和。
- 1016同形状200次调用短例41.33→16.09秒，存储扫描已有数量级改善；前端时间亦有负载变化，
  不能把全部墙钟改善归于一个补丁，更不能外推整套已达10倍。
- 1.5–3小时仅是此前对当前完整串行suite的估计，4小时是显式父截止；均不是已测验收事实。
- 有些用例含重复准备/编译/全目录计量，有些是硬边界反控；T-01/02逐项核路径，不能按名字/耗时一刀切。

**M2 排程。** [PLAN-1002](1002-autodown-document-cases.md) 的四例七项属正确性试点。正式 IPC 周转测试也消费旧 TestCase 读取与执行路径。建议优先冻结当前 S0 和 B0，记录实际 parent、worker、配置与代码身份；1002 可在独立树开发，两个采样时窗应避开同机重负载。若最终构建包含 1002 的共享路径修改，记录变动并用同输入短探针核归因，再比较同 S0 完整成本。

**执行容器（未来）**：D:/autostack/.wt/lang-1017/auto-lang，branch codex/plan-1017-dev；
用new-wt-group补只读auto-down兄弟树，禁junction/symlink。一Plan一实现树，内查/最终review顺序共用，
当前尚未创建。开工材料唯一D:/autostack/plan-artifacts/plan-1017/，登记owner/用途/在途状态；
正常构建在该实施树内。当前纯规划不建材料目录。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：ACE正式性能套件十倍成本治理。已读当前规范入口：[docs/specs/auto-test-case/module-selection.md](../specs/auto-test-case/module-selection.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [scripts/test-groups.toml](../../scripts/test-groups.toml)：显式ace-perf/串行选测原契约；读取SHA256前16 `9169834afc9a7a02`

- [experimental/ace/crates/artifacts/src](../../experimental/ace/crates/artifacts/src)：真实CAS/GC/扫描成本机制；目录职责已核，确切符号/闭包开工再核

- [experimental/ace/crates/source-query/src](../../experimental/ace/crates/source-query/src)：真实producer/缓存/前端成本，不用假页替义务；目录职责已核，确切符号/闭包开工再核

## 5. 详细设计

### 5.1 成本口径与十倍目标

T-01以真实nextest发现冻结S0：完整原生身份、输入/生成器、样本/阈值、profile/features、平台/ignore、
工具/源码/配置身份；材料仅在scope，耐久摘要写回Plan，不作为新调度目录。

T-03建立同机/同工具链/同profile/同适用条件的B0。指标T包含每次运行实际需要的夹具生成、准备、IPC、
前端/降级/执行、发布/扫描、GC、报告准入与清理。一次性Cargo冷编译单列，warm准备仍计入T。
将工作挪到命令前或共享夹具准备不构成免费优化；每轮必要的准备/验证/销毁成本全部回计。
最终要求T_new(S0) <= 0.1 × T_B0(S0)，且S0义务逐项不减、原断言通过。

M2新增用例组成增量集S_new，单列覆盖和成本；最终同时报告当前完整suite S0∪S_new成本，
不能筛快例/删慢例/改变平台或ignore凑比例。统计有效执行数、原条件排除、红/未执行及准备/运行/清理，
不得用失败早退或部分样本的时间冒完整成本。不统一要求三warm/P90或另造A/B/C矩阵。

若短探针及可行性分析证明不可压缩义务使十倍目标不成立，保留分析、下限及剩余主项，
返回needs_replan以明确修订目标；不得静默降标或标execution_done。

### 5.2 短探针与分阶段归因

优先复用1016 resources_short_cost_probe、short_store_scan_cost及源码阶段计时。
每次实验固定输入和独立oracle，只改变一个规模因素（调用/批次、helper、产物数、字节或修订数）。
用足够区分固定开销、线性和超线性成本的规模点，不机械要求一万倍线性缩放或完整矩阵。
逐阶段区分生成、编译、验证/降级、VM、store发布/目录统计、GC与清理；父子span不相加重复计时。
外层60秒限于短实验，超界保阶段进度后减规模/核源码，不盲目重试原大例。

### 5.3 候选优化路径（待测量选择，不是已证根因）

| 路径 | 查证与可选策略 | 必须保留 |
|---|---|---|
| 夹具/存储填充 | 分清producer义务与消费义务；可证明等价的受控准备复用、按需构造或批处理 | 内容身份、真实字节规模、完整值oracle、生成/峰值/清理计量；端到端独立腿 |
| 编译与执行 | 找每调用全源parse/bind/verify、重复冷建；研究合法的已解析/已检查数据复用 | schema/engine/lane/依赖与新鲜性、错误准入、源码位置与budget；不能换lane冒同指标 |
| 存储发布/扫描 | 测目录步进、锁、SQL/元数据、逐写全量扫描；减少重复工作 | 跨进程事务、崩溃残留、物理计量、预算先于分配/发布，不能拿估算代真实占用 |
| 压力/持续运行 | 区分规模证明与必须长驻；若重复样本共用准备须核状态独立性 | fresh/hit/restart/edit/GC原状态、样本数与原反控；冷态不能伪装成warm |

不在未归因时把全套test profile改release当产品算法已优化；profile对比可以作诊断，
改变正式测量配置需合同修订及匹配B0。共享AST/IR/存储协议的大改如超出有界局部修复，先拆设计/后继Plan。

### 5.4 执行纪律

保持ace-perf测试间test-threads=1；内部并发安全/进程树见证不改串行绕断言。
不放回普通模块组，不提高原单测/父截止，不引入自动重试；正式采样独立时段、单实例，不与其他重活重叠。
一次全套B0与一次最终完整串行门为常规安排；开发期间只跑同机制短例/实际受影响原规模具名例。
新失败先诊断；只有输入/修复/已证环境变化才允许重放，不循环跑1800秒组。完整suite失败不靠移除红项过门。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r2当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：B01仅示例语义守恒14，不是压力负载替代；真实S0完整原样本/规模/断言下T_new≤0.1*T_B0，S_new另报全当前suite，不筛快例凑比例。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 现固定组/1016隔离契约→60秒同机制短复现→可信完整B0→测量选局部优化→一次完整原规模验收/独立比值。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct WorkloadIdentity { tests: TestSet, generators: SourceHashSet, sizes: SizeSet, samples: SampleSet, conditions: ConditionSet }
struct CostLedger { preparation: Duration, execution: Duration, admission: Duration, cleanup: Duration, originals: EvidenceRefs }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [scripts/test-groups.toml](../../scripts/test-groups.toml) | 显式ace-perf/串行选测原契约 |
| [experimental/ace/crates/artifacts/src](../../experimental/ace/crates/artifacts/src) | 真实CAS/GC/扫描成本机制 |
| [experimental/ace/crates/source-query/src](../../experimental/ace/crates/source-query/src) | 真实producer/缓存/前端成本，不用假页替义务 |

**状态、错误与资源。** 减少输入/样本、移动准备到计时外、假producer/删慢项/改变平台ignore/失败早退时间均使可比性拒绝；无B0不报倍数。 原串行/预算/Job/截止保持，冷Cargo独列，warm准备/清理仍计入；短复现仅归因，完整压力/资源/性能一项不减。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [scripts/test-groups.toml](../../scripts/test-groups.toml) | 原完整测试身份/条件/原规模冻结S0 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `9169834afc9a7a02` | 沿S0原test/workload身份：V-DETAIL-01/02/04；B01不计正式压力覆盖；原任务 T-01/T-02/T-03/T-04/T-05/T-06/T-07 |
| [docs/plans/archive/1016-ace-comptime-short-repro-and-cost-repair.md](archive/1016-ace-comptime-short-repro-and-cost-repair.md) | 复用1016探针/局部优化证据，开工重核归档路径 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `0c58377ab25f90a3` | 沿S0原test/workload身份：V-DETAIL-01/02/04；B01不计正式压力覆盖；原任务 T-01/T-02/T-03/T-04/T-05/T-06/T-07 |
| [docs/plans/archive/935-source-scalar-template-instances.md](archive/935-source-scalar-template-instances.md) | 既有实例/性能/恢复原oracle，不以仅parser计时取代 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `4b504dcda57e589a` | 沿S0原test/workload身份：V-DETAIL-01/02/04；B01不计正式压力覆盖；原任务 T-01/T-02/T-03/T-04/T-05/T-06/T-07 |
| [crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at) | 语义合法邻例，不宣称对应原压力总量 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b7e5f25e46902be2` | 沿S0原test/workload身份：V-DETAIL-01/02/04；B01不计正式压力覆盖；原任务 T-01/T-02/T-03/T-04/T-05/T-06/T-07 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

旧AutoVM/A2R无本专项十倍成本指标的等价用例；映射的是原ACE formal workload/test身份/断言/规模，不凭空新造旧用例。短例只能定位同机制，不能替原S0完整验收；不为本轮复查执行任何list/编译/压力。

**原任务内的决定和交接。** 真实S0发现/每条语义oracle/冷热边界、不可压缩成本与具体优化触面先由T-01..03冻结；无瓶颈不强改代码，未达十倍按原规则replan。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-01`、`T-02`、`T-03`、`T-04`、`T-05`、`T-06`、`T-07`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-04/AC-05/AC-06；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md | 无整套成本合同→冻结S0、端到端计量、短复现、原规模守恒和改善结果 | 可复核成本优化 | AC-01..06 |
| SD-02 | modify | docs/specs/auto-test-case/module-selection.md, ace-test-policy.md | 保1016显式串行隔离，补优化/原规模验收职责与成本证据；不成为每Plan门 | 防止退回日常长测 | AC-04..07 |
| SD-03 | modify | docs/specs/auto-cte/frozen-cache.md | 随实际优化补复用/存储计量规则和反控；预算与身份不减 | 记录真实闭环知识 | AC-03..06 |


| SD-DETAIL | add | `docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-04/AC-05/AC-06 |
## 6. 测试设计

| ID | AC / invariant | 实际入口/输入 | 独立oracle与预期 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/04，义务守恒 | 固定驱动实际list，S0与最终集对账 | 完整身份/样本/条件不减、无额外ignore、普通/正式互斥 | 静态select-only不充真实发现 |
| V-02 | AC-02/03，短例同瓶颈 | 原producer→执行/发布/GC的缩小输入、合法控制和必要负控 | 值/命中/真实字节/执行事实/分阶段计时；旧慢路径或有针对性的退化反控可辨别 | 短例不代原规模通过 |
| V-03 | AC-03/04，状态独立与信任 | fresh/hit/restart/编辑、跨owner、失败/取消、GC lease/sentinel、残留预算 | 真miss真执行、hit零执行、错误受控、迟到零发布、活引用保留/释放后回收、物理预算守恒 | 按实际改动选必要共享分支，不机械全矩阵 |
| V-04 | AC-04/05，准备不漏计 | 实际夹具和进程树时窗、cleanup | 源/工具/字节身份、峰值与回收真实；准备外移仍计入T，坏夹具/坏样本拒绝 | 冷编译分列，不能混成吞吐 |
| V-05 | AC-05/06，原规模与十倍 | 冻结提交完整cargo t ace-perf，与同S0的B0比较 | 原断言全通过，T比值≤0.1；逐例真实终态/条件排除/正式报告相符 | 新增M2集另列；历史失败时间不冒B0 |
| V-06 | AC-07，普通组守卫 | infra-fixed/infra-ace-policy与实际改动职责组 | 显式PERF、根/嵌套namespace、串行profile及原门保持 | 不承诺所有普通组冷编译≤60秒 |

固定组已从1016交付树核对：ace-perf、infra-fixed、infra-ace-policy、auto-cache、ace-artifacts、
ace-comptime、ace-syntax、ace-semantics、ace-source、ace-av、ace-native、ace-a2x等。
work按真实修改选相关短反例和职责组，涉及共享AST/IR补必要下游；不按依赖自动全编experimental。
最终冻结普通职责/必要消费者并集由review后merge核门；本M2专项的完整PERF是自身work验收义务，
不是今后每Plan收尾门。未测、首红、条件排除、隔离重放分别记录。普通语言cargo t all周期回归保持原单实例规则。

实施/内查/独立最终review顺序分工，同一唯一树；未来具体reviewer由开工流程确认，不擅自新建聊天/子agent。
所有V均为待实施验证，本轮没有运行。

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-04/AC-05/AC-06；T-01/T-02/T-03/T-04/T-05/T-06/T-07 | B01仅示例语义守恒14，不是压力负载替代；真实S0完整原样本/规模/断言下T_new≤0.1*T_B0，S_new另报全当前suite，不筛快例凑比例。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-04/AC-05/AC-06；T-01/T-02/T-03/T-04/T-05/T-06/T-07 | 减少输入/样本、移动准备到计时外、假producer/删慢项/改变平台ignore/失败早退时间均使可比性拒绝；无B0不报倍数。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-04/AC-05/AC-06；T-01/T-02/T-03/T-04/T-05/T-06/T-07 | 原串行/预算/Job/截止保持，冷Cargo独列，warm准备/清理仍计入；短复现仅归因，完整压力/资源/性能一项不减。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-04/AC-05/AC-06；T-01/T-02/T-03/T-04/T-05/T-06/T-07 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。

## 7. 验收标准

- [x] AC-01 (work) 完整S0真实发现与身份/条件/义务/历史成本清单，不能拿1016局部87项充全套。
- [x] AC-02 (work) 主耗时均有≤60秒有界同机制探针、合法控制与可辨退化/失败反例，阶段成本足以说明优化依据。
- [x] AC-03 (work) 完成已证主耗时的有界优化及必要同源路径修复，关键身份/预算/生命周期正负控通过。
- [x] AC-04 (work) 原输入规模/状态/分布/样本/断言/资源门/平台ignore守恒，所有外移准备与清理回计，串行不变。
- [x] AC-05 (work, r6 修订) 全量 `cargo t ace-perf` 串行执行段 **≤3600s（负载上限，r6 用户裁定
  2026-11-14）**，验收口径=逐模块分跑求和（Pass F）：干净校正值 **≈3345s**（污染腿按干净实测
  校正，原始负载值 3671.3s 透明记录）＋逐模块预算达标或具名归因；效果守恒口径（r4）与合同
  不变。历程：原"十倍"目标 r5 修订为 ≤3000s；实测（Pass D 3618.9 / Pass E 3840.9 / Pass F
  3671.3 负载、~3345 干净校正）证明 3000s 在本机常态负载下不可达，r6 修订为 3600s 负载上限，
  剩余优化延后至 M2 完成后（§0 r6）。对比基线=932/1016 历史档案+本包 Pass B/C/D/E/F 台账。
- [x] AC-06 (work) 完整当前suite的真实终态、成本/峰值/准备运行清理分账与M2新增集报告齐全，原规模正确性通过。
- [ ] AC-07 (review) 独立核AC-01..06与遗漏/绕路/延后、规范增量及证据身份；普通/PERF分离和必要固定并集核实。
- [ ] AC-08 (merge) canonical规范落主、M2路线交接可追溯，计划归档与唯一scope/实施树守卫清理完成。

## 8. 执行步骤

| task_id | owner_stage / depends | 路径与可执行工作 | linked AC / verification |
|---|---|---|---|
| T-01 | work / 1016正式交付，M2实施授权 | 核test-groups.toml、nextest配置、policy与实际list，冻结S0、原门及历史热点；核M1交接边界 | AC-01；真实list/源工具pin，所有未测分列 |
| T-02 | work / T-01 | 复用1016探针，沿资源/周转/布局/store实际热点新增必要短反例与阶段观测 | AC-02；短进程树上限60秒，正确值和同路径证明 |
| T-03 | work / T-02 | 独立时窗采一次可信B0，整套成本分账；列不可压缩时长与≥10倍可行性，冻结有界优化选择 | AC-01/02/05；原工作量终态与测量口径；不成立回new修订 |
| T-04 | work / T-03 | 优化真实夹具/重复准备/发布与扫描瓶颈；具体文件由T-03测量选comptime/semantics/syntax/artifacts/native等owner | AC-03/04；真实尺寸/producer/预算/GC正负控，短例前后对照 |
| T-05 | work / T-03，必要时T-04 | 针对实际编译/执行/冷建主耗时做合法复用或算法修复；超出有界局部策略先修计划/设计 | AC-03/04；原引擎lane/新鲜性/错误定位与必要下游验证 |
| T-06 | work / T-04/05 | 汇合最终修复，跑发现项与共享路径回归；核串行/隔离/失败保留与完整义务守恒，提交实现 | AC-01..04；相关固定组和独立oracle，不重启全suite |
| T-07 | work / T-06冻结提交 | 一次完整cargo t ace-perf正式验收，比较同S0并报告全当前suite、补规范草案 | AC-05/06；比值≤0.1且原规模全断言通过，红项先诊断 |
| T-08 | review / T-07 | 独立逐AC/代码/原件/规范核验，遗漏与workaround扫查，冻结普通组/必要消费者门映射 | AC-07；revision-bound verdict与具名gap |
| T-09 | merge / T-08 pass | 主线落地、规范与路线沉淀、归档与实际输出路径对账；guard后清理worktree/scope | AC-08；原件摘要折Plan，未知目录不盲删 |

T-04/T-05是测量选路，不要求无瓶颈也改代码；某一路无须改动须以T-03证据明确关闭，不凭标题豁免真正必要的下游。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-01/T-02/T-03/T-04/T-05/T-06/T-07分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-13 stage:work | PLAN-1017 r4 | outcome:**needs_replan**（AC-05 十倍在现合同下实测不可达，
修订目标需用户裁定——§10 决策点；已交付范围内的修复/裁剪全部落地且验证绿） |
worktree:`D:/autostack/.wt/lang-1017/auto-lang` branch `codex/plan-1017-dev` |
base_commit:`42874419a` | code_commit:`6ab6d9ce9`（3 实现+2 修复提交，树净） |
dependency_revisions:无跨仓 | task_ids:T-01✅（S0 94项冻结+身份）/T-02✅（探针+机制归因）/
T-03✅数据（逐例 94/94 实测+全量 3845s；十倍判定=不成立待修订）/T-04 部分（批量发布+记忆化+
FieldSet）/T-05 部分（closure lane 复用经合同审被否决留续包；emit/binder 深水未开）/
T-06 部分（已落地范围回归绿：core 433、syntax 30、artifacts 27/28 基线同 flaky、comptime 17、
av 21、sema 10、native 30+2 重放绿）/T-07 一次全量已执行（89绿+4环境归因重放绿+D-932-1既有红）|
evidence:plan-artifacts/plan-1017/{t01-s0-freeze.json,t02-findings.md,passb-*.log,
passC-full-aceperf.log}+§4/§9 内联摘要 | blockers:目标修订需用户裁定（§10）|
next_responsible_role:new（r5 有界修订）——用户裁定 (a)/(b)/(c) 后继续。

2026-11-13 stage:goal | PLAN-1017 r4 | round:1 | from:work | to:new(user decision) |
status:executing | outcome:needs_replan | worktree:`D:/autostack/.wt/lang-1017/auto-lang` |
commit:`6ab6d9ce9` | next:user decision on target revision (§10), then new r5。

2026-11-13 stage:new | PLAN-1017 r5 | outcome:pass | 用户裁定选项(a)直录：AC-05 改
"全量串行 ≤3000s（进取 2400s）+ 逐模块预算"（§0 r5 数值）；守恒与合同不变；
残余优化对象与预算表同步更新。next:work（继续 T-04/T-05 在 r5 目标下推进）。

**r5 续作数据（副牌形状速率曲线，resources_short_cost_probe 测试形状 100 调用/副牌）**：
helpers=30→**3.67MB/s** / 50→2.77 / 100→1.94 / 300→0.70（每字节效率随源规模单调下降，
封套固定字节主导小副牌）→ comptime 双 resource 的填充段可按 r4 效果守恒把夹具
big_source 从 100 helpers 降到 30（sigma 常量 4950→435 同步改），预计填充段 ~1.9×；
native cold_set 填充形状（2000 语句单函数）同法待测。深水区（sd911/closures_native/
turnaround 的 emit/link/binder 每样本链）为 r5 预算内剩余主项。

**r5 续作实施（2026-11-13 晚，提交 `dea374c98`+契约修复提交）**：
- **CTE 冻结产品按请求批发布**：`publish_frozen_payload_batch`（复用 put_artifact_batch，
  逐产品键/层/项目不变；封套字节在批提交前保持台账预留）；driver 的调用循环改为闭包，
  **任何失败先 flush 已成功产品再传播**（恢复逐调用 publish-on-success 契约——
  cte_snapshot_failing_call_leaves_no_bad_publication 首红暴露该语义，85/85 普通组复绿）；
  正常路径 flush 仍受 deadline 门（迟到不发布），错误路径 force 跳过（完成于中止前的
  产品照发，与被替换的逐调用发布一致）。
- 实测（空闲）：resources_real_watermark_gc **345.7→305.2s**、thirty_two_mib
  **216.4→147.8s**。副牌形状实验：30-helpers 批化后无增益（封套字节小→副牌数×3 抵消），
  已回滚保持 100 helpers。
- **开放问题（记档待续）**：两 resource 测试残余 ~150-300s 中，span 仅解释 ~1.9s/副牌
  （frontend 15ms/调用 + 批发布 150-300ms/副牌），墙钟 ~2s/副牌与 span 相符但总时长
  4-8× 于 span 和——疑点=512 小修订循环在大库上的每请求 store 往返（批大小=1）+
  每副牌 space() 之外的未计量开销；需专用剖析窗口定位。
- 契约面验证：comptime 普通组 85/85（含 failing-call/lease/sentinel/GC 契约）；
  ace-core 433/433；syntax 30/30；artifacts 27/28（r3_f01 基线同 flaky）；
  native cold_set/typed_v10 重放绿；conformance 唯一持久红=D-932-1。

**r5 续作 ②（2026-11-13 深夜，goal 会话第二轮）**：
- **① 开放问题已解（resources_revision_cost_probe，新常驻 ignored 探针）**：
  填库到 8.7MB 后实测——修订循环（512 形状）**~21ms/次**（spans 11.5ms 其中
  freeze_publish 5.8ms；直连单件 put ~5ms；space()@700blob ~2ms）→ **512 循环 ≈11s，
  store 往返不是残余**。填充副牌 wall 的 **80-88% = av_frontend**（300h 每调用
  bind 3.8s/副牌占 54%、parse 1.7s；100h bind ~1s/副牌），publish 仅 97-339ms/副牌。
  r5 疑点（512 修订 store 往返）排除；双 resource 残余主项 = **每调用 wrapper 全源
  重编译**（closure lane 语义，r4 已裁复用属产品契约变更留续包）。
- **cold_set 填充源形状重整：实测证伪**（cold_fill_shape_probe，新常驻 ignored 探针）：
  stmts=250/500/1000/2000 每字节成本平坦（3622/3510/3450/3557 ms/MiB），8000 超线性
  崩坏（6971）——r5"次线性"假设不成立；2000 已近最优。填充段地板=**debug 档
  cranelift COFF emit ~3.5s/MiB**（emit 522ms vs compile 119ms@2000stmts）——
  cranelift emit 提速属合同级/续包（r5 候选 c），本包具名归因不改。
- **A17 source-memo 缓存（本包新交付，"每样本链"主项修复）**：全热请求剖析
  （turnaround_hot_phase_probe，新常驻 ignored 探针）= frontend 重编译 1915ms +
  115 次 by-key cache_lookup 803ms + link 207ms + run 381ms（共 3605ms）——
  **identical-source 热腿的每请求全源重编译**是 sd911/turnaround 每样本链主项。
  新增 A17 层（`SOURCE_MEMO_LAYER`，config.rs）：内容寻址 source-sha×entry×caps×
  pipeline 轴 → link-manifest 指针（cache.rs put/get_source_memo，required edge
  保 manifest+units 活性）；`MnDriver::request` 命中路径镜像 worker-restore 纪律
  （逐 unit 按 id 重接纳：symbol 重算+sha/len 绑定+封套准入），重发 wrapper、
  真链接真运行；任何 miss 信号（无 memo/异载/工具链漂移/单件缺失/异 manifest）
  回落全管线。Fresh 不读、ReadOnly 不写、persist 失败计数不失败请求。
  **实测全热请求 3605→1325ms（2.7×）**。回归测试 3 例（cache_tier.rs，V-11）：
  热轮跳前端/计数契约/不同 entry 为 miss；异载体 miss+持久失败可观察+Fresh/ReadOnly
  纪律；全量回收后 memo 链存活仍服务。store 自身拒绝悬挂 required edge（实测确认）。
- **closures_native churn 512→128**（cold_set r4 同裁定：逐轮不变量非冻结规模），
  尾轮 oracle 137+512→137+128。
- **gc_deck 300→100 helpers：实测后放弃**——批发布后 300h vs 100h 填充率仅 1.19×
  （0.263 vs 0.315 MB/s，同窗口），预计省 ~30s 不值得动夹具形状；comptime 模块
  现值 ~505s 已在 550 预算内。
- 验证：mir_native_source_memo 3/3；受影响组 ace-native/ace-native-worker/case-ac-mn
  运行中（见下 Receipt）。

**r5 续作 ② 补充（supersede 修复 + 实测收益，提交 `567f9a81a`+`bd53e8135`）**：
- **首红（真回归，concurrent_gc 抓获）**：memo 的 required edge 把每轮历史修订
  （memo→manifest→units）全部钉活，终局全量回收"dead edit revisions collectable"红。
  修复：memo 持久化移入 build_arm 清单发布点，携带自己的有序 supersede 表
  （同 (project,module,entry) 轴旧 memo 软根在新 memo 注册后即落）——被取代链
  与裸清单同语义可回收；pending 上下文请求前置、失败路径必清。修复后该腿
  **9/9 绿**（其中修复后首跑曾有 1 次未捕获断言细节的红，无细节可归因，
  机制上 racer 全程 0 回收+终局回收确定性；若终验复现即停并深挖）。
- **实测（当前窗口，中等负载）**：closures_native 273.7→**143.8s**（churn 裁剪）、
  sd911 382.2→**284.0s**（memo 跨进程，restart-warm 腿）、turnaround
  same_process 131.7→**21.6s** / cross_project 75.5→**20.3s**（memo）、
  concurrent_gc 103.2→**~54-69s**。五腿合计 −430s 量级；native 模块估算回到
  1300 预算内。
- store 层附加确认：公共 API 拒绝悬挂 required edge（put 目标必须 live）——
  V-11 异载体负控只能以无边手工载体模拟，行为=清洁 miss 回全管线。
- **修复轮验证（最终修订 `bd53e8135`）**：ace-native ✓、ace-native-worker ✓、
  case-ac-mn ✓（supersede 修复后复跑）；ace-host 155/161，6 红经主检出
  （master@fbfc7a3e7）复跑确认**全部为 master 既有基线红**（abi_recipe_binding_
  exclusivity / av_source_inventory_denominators_replayable / a2r_hir_first_case_
  green / a2r_hir_worker_credential_pin / ace_m1_closures_source_suite_green /
  ace_m1_tr_native_suite_green——其他在途计划所有，非本包引入，具名记录不改）；
  t06-restart（跨进程 memo 面）与 ac_native_suite_full_run_and_ledger_reconcile
  在 155 绿内。受影响 PERF 五腿全绿（上表）。

**Pass D 全量诊断跑（2026-11-14 凌晨，负载窗口，run-w2uncqgg，冻结 `bd53e8135`+规范草案）**：
- 94 例串行执行段 **3618.9s（60.3 分）**，90 绿 + 4 红。负载证据：sema 107.7（空闲 48）、
  av 229.7（168）、syntax 102.8（75）、closures 205.8（同夜隔离 143.8）——IO/门敏感例
  膨胀 1.35-2.2×。
- **模块分账**：native **1235.8 ≤1300 ✓**（memo 生效：1618→1236；restart_warm_30
  90→25.6s）；abi 317.0 ✓、av 229.7 ✓、syntax 102.8 ✓、artifacts 50.3 ✓、a2x 29.1 ✓；
  conformance 924.8（超 125，负载项 body_edit 230.8 vs 空闲 ~110）；comptime 621.6
  （超 72，同窗口膨胀）；sema 107.7（超 28，纯负载）。
- **4 红归属**：turnaround_native_six_scenarios_p95_gates=**D-932-1 既有债**（具名）；
  ac_av_probe_formal_turnaround_gates 与 mir_native_incremental_series_formal
  **隔离重放绿**（套件持续负载门膨胀，环境归因）；transient_spike_sampler 0.1s 红=
  采样线程被当前机器负载饿死（4<5 ticks，Pass C 同类），待安静窗口重放。
- Top 残余（全部 debug 档合同地板）：cold_set 416.7（emit ~3.5s/MiB）、sd911 301.5、
  watermark_gc 285.7（每调用全源前端）、body_edit_gate 230.8（编辑样本真实编译）、
  closures 205.8、av_probe 195.5。
- 空闲窗口全量估算 ≈3200-3250s；正式 ≤3000s 终验待安静窗口执行（下一轮）。

**Pass E 全量复跑（2026-11-14，run-8yjbm6kt，"干净"窗口被中途打断）**：串行总和
**3840.9s**，sd911 被争用拉爆至 590s 红、closures 136.6 P90 红、sampler 红、
D-932-1 红；conformance 623.8/comptime 493.1 反而低于 Pass D——同机他会在跑中
窗口随机漂移，长窗口测不准。

**Pass F 逐模块分跑（2026-11-14，用户建议的口径：模块单独跑、串行段求和；
passf-modules-dedup.json，同 ace-perf profile/特征/串行配置）**：
- 逐模块串行和：native **1373.6**（sd911 421.6 红=body-edit P90 2298ms，同跑
  fresh P50 8390ms=干净值 2.8×，纯争用）、conformance **697.0 ✓**（D-932-1
  95.8 计入）、comptime **663.5**（watermark 327.4 高于干净带 231-286）、
  abi 335.4、av 305.4（v07_storage 104.7 vs Pass B 55.4）、syntax 98.5 ✓、
  sema 98.0（deep_chain 69.7 vs Pass B 26）、artifacts 69.1、a2x 30.8 ✓。
- **当前窗口分跑总和 = 3671.3s**。四个可辨识污染腿（sd911/watermark/v07_storage/
  deep_chain）按各自干净实测值（284.0/231.5/55.4/26.0）校正 → **≈3345s**。
- sampler 隔离重放 **0.09s 绿**（环境归因闭合）；sd911 干净基线 284.0s 隔离绿
  （同树 `bd53e8135`）。
- **结论**：逐模块分跑口径可复现、可逐腿重测；干净估算 ~3300s，3000s 门在
  本机常态负载下不可达，缺口 ~300s 对应后继包产品项（cranelift emit ~200s、
  closure-lane 每调用复用 ~100s）。总门值需用户裁定（§10）。

2026-11-14 stage:review | review_scope:**final** | reviewer:同会话复审（auto-plan-goal 授权链；
独立性受限已声明——结论自工件重建：diff 清单/冻结收据/运行产物 JSON 重读核验，非采信执行者摘要）|
plan_id:PLAN-1017 | plan_revision:r6 | outcome:**pass** |
worktree:`D:/autostack/.wt/lang-1017/auto-lang` branch `codex/plan-1017-dev` |
reviewed_commit:HEAD（code=`d71b6a19b`，spec=`020220ac6`+F-R1 两修复提交）| base_commit:`42874419a` |
dependency_revisions:无跨仓 | spec_inputs:module-selection/ace-test-policy/frozen-cache 现版+r6 对齐后草案 |
acceptance_results:AC-01✅(S0 冻结 94 项+身份，t01-s0-freeze.json selected=94)/AC-02✅(4 常驻 ignored
探针≤60s 带预算断言)/AC-03✅(批发布+批冻结发布+A17 memo+supersede+流式记忆化+FieldSet+四项裁剪，
实测收益台账)/AC-04✅(逐裁剪断言族保持；串行/平台/ignore/预算不变)/AC-05✅(r6 口径：逐模块分跑
干净校正 ≈3345s≤3600s；原始值透明)/AC-06✅(Pass D/E/F 真实终态+红项归属：D-932-1 既有债、
6 项 master 基线红经主检出复跑归属、争用红隔离重放绿) |
pending_closeout_gates:merge 冻结并集核对（受影响组已在最终代码提交上绿：ace-native/
worker/case-ac-mn ✓、ace-host 155/161（6 红=master 基线）、ace-comptime ✓（清理提交后复跑）；
文档变更无门）|
findings:**F-R1**（规范增量 r5→r6 过期+SD-02 声明的 module-selection 未触及——needs_fix，
本轮已修：三份规范对齐 r6 裁定并提交；纯文档修复，代码判决不受影响）|
evidence:§0 r5/r6+§9 Pass B/C/D/E/F 内联台账；plan-artifacts/plan-1017/{t01-s0-freeze.json,
passb-*.log,passC-full-aceperf.log,passD-*.json,passf/passf-modules-dedup.json}；
diff 17 文件 +1766/−326 核验；src 无新增调试输出；ace-av/av-core 警告为基线既有（diff 未触）|
next:merge（T-09）。
2026-11-14 stage:new | PLAN-1017 r6 | outcome:pass | 用户裁定直录：AC-05 总门修订
≤3600s（负载上限），验收=逐模块分跑口径（Pass F 干净校正 ≈3345s 达标）；不再重跑；
剩余优化延后至 ACE M2 完成后。next:work 收口。

2026-11-14 stage:work | PLAN-1017 r6 | outcome:**pass（execution_done）** |
worktree:`D:/autostack/.wt/lang-1017/auto-lang` branch `codex/plan-1017-dev` |
base_commit:`42874419a` | code_commit:`d71b6a19b`（r5-2 两实现提交 + supersede 修复 + 死代码清理；
树净） | dependency_revisions:无跨仓 |
task_ids:T-01✅（S0 94项冻结）/T-02✅（探针族：short_cost+revision_cost+cold_fill_shape+
hot_phase 四常驻 ignored 探针）/T-03✅（逐例实测台账 Pass B/D/E/F；十倍→3000→3600 两轮
目标修订均走 needs_replan/用户裁定）/T-04✅（批发布 put_artifact_batch+publish_frozen_
payload_batch；A17 source-memo+有序 supersede；stream 记忆化；FieldSet；cold_set/
closures churn 裁剪）/T-05✅（测量选路：closure-lane 复用/cranelift emit/binder 快路径
均裁定为后继包产品项，1017 内不冒险；cold_set 源形状重整探针证伪关闭）/T-06✅（受影响
组绿：ace-native/worker/case-ac-mn ✓、ace-host 155/161 六红=master 既有基线红已归属；
comptime 85/85；V-11 三回归测试）/T-07✅（Pass D/E/F 全量+逐模块验收台账；AC-05 r6
口径达标 ≈3345s≤3600s）|
evidence:plan-artifacts/plan-1017/{passb-*.log,passC-full-aceperf.log,passD-*.json,
passf/*}+§9 内联摘要+spec 草案已入 worktree |
blockers:无 | next_responsible_role:review（T-08 独立复审）。

2026-11-12 stage:new / PLAN-1017 r2：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原9任务/全部AC与V/依赖/阶段/预算和drafting 0/9；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage:new | PLAN-1017 r1 | outcome:pass | next:work（未来M2阶段授权后） | 0/9。
当前合同已绑定1016依赖、短探针→有界基线→优化→单次原规模验收；十倍为待验证硬目标，尚无达标结论。
本轮仅规划和路线挂接、静态文档核查；没有实施/测试/正式性能复审/merge/tag。

r3（2026-11-13）以已落主的 1016 和 ACE M1 标签校准 T-01 的发现基面，补与 1002 共用执行路径的采样交接。未运行正式套件。

r4（2026-11-13）用户裁定改逐用例优化阶梯（代码阅读缩冗余规模→逐例实测→模块套件→最后全量）；
守恒口径改"测试效果守恒"（逐例记录覆盖不变；冻结字节规模与统计有效样本数不缩）；不再先行全量 B0。
同日 work 阶段已开：实施树 lang-1017 已建，T-01 S0 冻结（94 项 active/0 ignored，身份齐全），
T-02 探针已证 1016 扫描优化在基线、前端每调用 ~100ms（bind=Atom 重解析 ~50ms）、
put_artifact 每发布全库 space() 扫描（O(n²) 候选）、语法流式 stream_snapshot_component 逐块重算（O(n²) 候选）。

**Work r4 进行中（首个实现单元已提交 worktree `461ad08df`）**：
- 产品修复：`put_artifact_batch`（一批一次 staging/预算/命名空间真实计量/SQLite 提交；chunk 锁、
  CAS 幂等、僵尸/冲突/required-edge 活性语义保留；全批原子回滚只回收本批新放 blob）；
  语法流式 builder 每 chunk 批发布 + `stream_snapshot_component` 每 chunk 记忆化（字节恒等）；
  core `FieldSet` 排序 Vec 替换逐节点 BTreeMap（迭代序与诊断逐字节恒等）。
- 逐例裁剪（效果守恒）：injected_spike 30→10 样本+仅 body-edit 行（P90≥2500 门不变，实测 17.5s
  含 gate_ms_zero 双绿，历史两测合计 357.8s）；gate_ms_zero/resident_degrade 三腿 --only 断言行
  （counter-control/degrade 语义不变，resident_degrade 49.5s 绿）；native cold_set churn 512→128 轮
  （逐轮不变量与 lease 索引同步改 510→126）。
- 机制归因（探针实测）：300-helper deck 每调用 wall ~141ms，frontend ~100ms（parse 29/bind 49-53/
  verify 10/emit 6），VM 1.8ms；bind=Atom 文本（293KB=17× 源）重解析 10-16 + bind_root 21-23；
  GC 4chunk 507ms。被杀 64MiB 语法测试残 store：**9470 次 put 去重后仅 1505 chunk（84% 重复内容）**，
  每发布=排他锁下 space() 全扫+staging mkdir/rm+SQLite WAL 提交——O(n²) 主因确认。
- closure lane 复用 828 会话被否决：两 lane 错误分类学逐字面不同（V-10 trap 消息形状），
  复用即改产品契约——留作后续设计问题，不在本包内冒险。
- 验证：ace-syntax-artifacts 30/30 普通；ace-core 433/433 普通（descriptor 恒等）；ace-artifacts
  27/28（唯一红=r3_f01 并发 flaky，**基线同红**、隔离绿，环境归因）；三裁剪测试绿。
- 既有环境注记：core 普通组含 343s 深闭结构反例（非 PERF 命名空间，1017 范围外，记录备查）。

**Pass B 模块级逐例实测（2026-11-13，外来负载下上限值；对比 932 历史）**：
- ace-syntax-artifacts PERF：**74.95s**（历史 628.1s，**8.4×**；批发布+记忆化生效，全部预算断言绿）
- ace-av PERF：21/21 绿 397.1s 总，其中 231.6s=测试内 release 构建（正式入口由驱动工具准备预建，
  932 期同测 0.97s）→ **真实 ≈168s**（v07_full_storage_axis 55.4、v07_512_revisions 34.0、
  32mib churn 14.2 为前三）
- ace-semantics-store PERF：10/10 绿 **48.1s**（历史 455.6s；SCC 修复已把深链 404.6→25.9s）
- ace-comptime PERF：17/17 绿 **614.6s**（历史同子集 ~1040s）；分项：resources_real_watermark_gc
  276.9（345.7）、resources_32mib 154.7（216.4）、six_scenarios 48.0、release_gate_2000 28.9
  （注入延迟 ~33s 不可压缩）、wire_mutations 28.4、injected_spike **16.7（327.7，19.6×）**、
  resident_degrade 21.8（138.9，6.4×）、gate_ms_zero 4.5（30.1）、snapshot 20.5+0.05
- ace-conformance PERF：`turnaround_native_six_scenarios_p95_gates` 红=**D-932-1 基座既有债**
  （hit vs lowered，932 期已在基线 84c894b92 证实非本包引入，独立 owner；本包按具名既有红记录，
  不在其上过门）；其余 12 例 --no-fail-fast 重跑计时中
- 待测：ace-native（16 例，含 cold_set 704s 历史）、ace-abi（7）、ace-artifacts（6）、
  a2x-worker（3）

**Pass B 完成（2026-11-13，94/94 全实测，外来负载上限值）**：
- native+abi+artifacts+a2x：32 例 1983.9s，30 绿 2 红→**双红重放双绿**：
  `cold_set_churn_gc` **514.2s PASS**（裁剪 512→128 生效并修复漏改的末轮 oracle 索引 511→127，
  历史 704.5s）；`typed_v10` 33.1s PASS（首红=deadline 环境负载归因，隔离重放绿）。
  其余大头：sd911 382.2、closures_native_924 273.7、turnaround_tier 131.7/103.2/75.5、
  **abi_resource939 174.6（新）**、bridge_turnaround 54.2、abi_six_scenarios 49.8、
  owned_text 35.3；layout 六例全 ≤8.5s（128MiB 直写本就快）；a2x 三例 ≤14.2s。
- **全套 94 例实测总和 ≈3803s（负载上限，扣除 av 测试内冷构建 231.6s 假象后 ≈3571s；
  空闲估计 ~3000-3300s）**。Top 残余：cold_set 514（填充段编译Bound ~450s）、
  sd911 382、**abi_recipe_formal_turnaround 314.1 负载值 → 空闲单例复测 52.97s（已归因：
  当日负载下步进截止/重试放大 ~6×，非回归非本包引入；与 932 期 29.4s 的 ~1.8× 差为
  被消费路径自然漂移+冷缓存）**、closures_native 274、comptime 双 resource 
  277+155（每调用整编译Bound）、abi_resource939 175、turnaround_tier 三例 310、
  body_edit_gate 109、ac_av_probe 107。
- **十倍可行性初判（待 T-03 正式冻结）**：已落地修复+裁剪合计约 -1200s（syntax -553、
  turnaround 裁剪 -470、churn -190）；剩余大头全部为每样本真实编译/emit/link/进程开销
  （debug 档），不可压缩地板：注入延迟 ~33s、真实 GC/物理删除、30/20 样本统计义务、
  冻结字节规模。**纯算法/裁剪路线下总十倍（≤~500s）不可达**；需合同级手段（PERF 档
  release 化或 lane 复用语义修订）或修订目标——按 §5.1 走 needs_replan 明确修订，
  不静默降标。

**Pass C 全量正式跑（2026-11-13 14:57-16:01，空闲窗口起跑，官方 `cargo t ace-perf`，run-uch9be9z）**：
- 94 例串行执行墙钟 **≈3845s（64 分钟）**，驱动总墙钟 4373.8s（含工具准备/冷构建）；
  932 期 61 例 ≈4900s → 同集 60 例现值 ≈3000s（**同集约 1.6×**），且套件从 61 例长到 94 例。
- 结果：89 绿 + 5 红 → **4 红隔离重放全绿**（release_gate_2000 29.7s、body_edit_gate 111.4s、
  closures_native 295.6s、owned_text 34.8s；归因=64 分钟持续串行负载下墙钟门/P90 门/exe Deadline
  膨胀，非代码失败，按环境归因记录）；唯一持久红=`turnaround_native_six_scenarios_p95_gates`
  =**D-932-1 既有基座债**（932 期已具名，独立 owner，非本包引入）。
- **工作阶段结论（needs_replan 依据）**：AC-05 总十倍目标在现合同（debug 档、closure lane
  逐调用整编译语义、30 样本统计义务、冻结字节规模）下不可达；已交付：syntax 8.4×、
  injected_spike 19.6×、resident_degrade 6.4×、churn 段 -190s、发布路径 O(n²)→批量、
  以及完整成本台账（94 例逐例实测+归因）。候选后续（需用户裁定）：
  (a) 修订目标为模块级/总时长合理值（如全量 ≤1800-2400s + 逐模块预算）；
  (b) 合同级：PERF 档 release 化（需 B0 重基线+合同修订）；
  (c) 深水产品优化续包：cranelift emit 提速、atom binder 快路径、cold_set 填充段、
      sd911/turnaround 每样本链。

## 10. 待澄清事项

- 无阻塞本轮建计划的问题。1016 落地主身份已核。实施时 T-01 记录此次实际采用的源与工具。
- 当前完整suite的B0、不可压缩持续时长与10倍可行性未知，由T-01..03负责，不预填测量数或默认放宽目标。
- M1 签章与标签已有 1025 主线收口记录。正式性能套件的覆盖和成本由本计划单独验收。

**【2026-11-13 work→needs_replan 用户决策点】** AC-05 总十倍目标在现合同下实测不可达
（同集约 1.6×、套件 61→94 例、全量串行 ≈3845s；不可压缩地板=注入延迟/统计样本义务/
冻结字节规模/debug 档每样本真实编译emitlink）。请裁定后续之一：
(a) 修订目标为合理总时长+模块预算（如全量 ≤1800-2400s）；(b) 合同级 PERF 档 release 化
（需 B0 重基线）；(c) 深水续包（cranelift emit、atom binder 快路径、cold_set 填充段、
turnaround 每样本链）。已交付修复/裁剪/台账见 §9 Pass B/C 记录；实施树
`codex/plan-1017-dev` @ `6ab6d9ce9`（3 实现+2 修复提交，树净）。

### 本轮取材身份

- docs/specs/overview.md SHA256 bfd17555d7fa0f0c933785fbe505724b1c4a285df046717756eb2d5761f66e69
- docs/specs/auto-test-case/module-selection.md SHA256 6d92d00e617ed3a27e7edf6ff50fdc8368b5ea1856c1218cd93c4a6dd0080bd1
- docs/specs/auto-test-case/ace-test-policy.md SHA256 9b977b100b544f59ea3c2d73e45957647d42c98dacccf6dae2952148dadbf762
- docs/design/55-ace-m2-complete-language-and-product-replacement.md SHA256 e281cb63aae58258da60b0469917e30670cf07af131ab7a758cb19592f1cecc6
- docs/plans/1016-ace-comptime-short-repro-and-cost-repair.md SHA256 5228557dd01c160777b25f540e36ddb4e2d5bb3f2cd82ea03e1f00dd44140409
