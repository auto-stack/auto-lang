---
plan_id: PLAN-1017
status: drafting
feature_name: M2 ace-perf串行套件成本与压力负载优化
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
phase: M2-A
current_step: 0
total_steps: 9
depends_on: [PLAN-1016]
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/ace-test-policy.md, docs/specs/auto-cte/frozen-cache.md]
new_spec_components: [docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md]
touched_goals: [GOAL-002]
---

# [PLAN-1017] M2 ace-perf串行套件成本与压力负载优化

## 0. 变更摘要

承接1016完成的普通组/PERF隔离与串行机制，在M2专门优化ace-perf的真实执行成本。
核心目标是同一冻结义务集、相同输入规模/样本/断言下，串行端到端成本降低至少一个数量级。
本轮仅建计划并挂路线：drafting r1、0/9；未建实施树、写优化代码、启动编译/长测或声明性能达标。

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

**来源**：docs/specs/overview.md、auto-test-case/{module-selection,ace-test-policy}.md；
Design55 §16.2 M2A-07、M2执行路线；1016 r5及其实现提交edc73bdf7；历史932性能台账；
comptime资源/周转测试、语义压力测试、语法64MiB流式、artifacts布局与store、auto-cache storage。
1016当前是execution_done而非已落主，主树规范/命令仍可能保旧名ace-performance；开工先核1016审查落地主身份，
不得把其worktree能力冒当前master。当前未发现与本需求相同的其他活动优化Plan。

**已知事实与不确定性**：
- 1016已经把普通/PERF隔离、串行profile和短用例交付到独立树；其触及8包发现87项=86 active+1原ignored，
  这是局部实际发现，不能充全ACE正式suite总数或全套绿。
- 932历史61项逐测试耗时合计约4900秒（含失败记录）；1013/1005有1201秒GC与1503.8秒语法流式记录。
  这些跨提交、配置和负载的历史数仅用于排序，不是本计划B0，也不能把普通组总耗时冒PERF总和。
- 1016同形状200次调用短例41.33→16.09秒，存储扫描已有数量级改善；前端时间亦有负载变化，
  不能把全部墙钟改善归于一个补丁，更不能外推整套已达10倍。
- 1.5–3小时仅是此前对当前完整串行suite的估计，4小时是显式父截止；均不是已测验收事实。
- 有些用例含重复准备/编译/全目录计量，有些是硬边界反控；T-01/02逐项核路径，不能按名字/耗时一刀切。

**M1交接**：1017不作为1013前置。1013仍负责其原件真实性、冻结提交、反伪造、独立审查与签前结论。
其旧recipe还引用已移入PERF的原规模CT、外部P90、资源churn、居民可见性等事实，须由1013显式修订M1口径，
保普通正确性来源并把正式压力结论归M2；不能删除测试后沿用旧分母/凭据假称全PASS。
本计划不修改1013的实现、recipe、分母或签章，不重开archived1005。

**执行容器（未来）**：D:/autostack/.wt/lang-1017/auto-lang，branch codex/plan-1017-dev；
用new-wt-group补只读auto-down兄弟树，禁junction/symlink。一Plan一实现树，内查/最终review顺序共用，
当前尚未创建。开工材料唯一D:/autostack/plan-artifacts/plan-1017/，登记owner/用途/在途状态；
正常构建在该实施树内。当前纯规划不建材料目录。

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

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/ace-perf-cost-and-reproduction.md | 无整套成本合同→冻结S0、端到端计量、短复现、原规模守恒和改善结果 | 可复核成本优化 | AC-01..06 |
| SD-02 | modify | docs/specs/auto-test-case/module-selection.md, ace-test-policy.md | 保1016显式串行隔离，补优化/原规模验收职责与成本证据；不成为每Plan门 | 防止退回日常长测 | AC-04..07 |
| SD-03 | modify | docs/specs/auto-cte/frozen-cache.md | 随实际优化补复用/存储计量规则和反控；预算与身份不减 | 记录真实闭环知识 | AC-03..06 |

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

## 7. 验收标准

- [ ] AC-01 (work) 完整S0真实发现与身份/条件/义务/历史成本清单，不能拿1016局部87项充全套。
- [ ] AC-02 (work) 主耗时均有≤60秒有界同机制探针、合法控制与可辨退化/失败反例，阶段成本足以说明优化依据。
- [ ] AC-03 (work) 完成已证主耗时的有界优化及必要同源路径修复，关键身份/预算/生命周期正负控通过。
- [ ] AC-04 (work) 原输入规模/状态/分布/样本/断言/资源门/平台ignore守恒，所有外移准备与清理回计，串行不变。
- [ ] AC-05 (work) 同S0可信B0与最终完整串行门可比，端到端T_new≤0.1T_B0；不成立则needs_replan，不假报达标。
- [ ] AC-06 (work) 完整当前suite的真实终态、成本/峰值/准备运行清理分账与M2新增集报告齐全，原规模正确性通过。
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

## 9. 复审记录

2026-11-12 stage:new | PLAN-1017 r1 | outcome:pass | next:work（未来M2阶段授权后） | 0/9。
当前合同已绑定1016依赖、短探针→有界基线→优化→单次原规模验收；十倍为待验证硬目标，尚无达标结论。
本轮仅规划和路线挂接、静态文档核查；没有实施/测试/正式性能复审/merge/tag。

## 10. 待澄清事项

- 无阻塞本轮建计划的问题。开工前须确认1016已审落地主身份与M2实施授权；当前不启动工作。
- 当前完整suite的B0、不可压缩持续时长与10倍可行性未知，由T-01..03负责，不预填测量数或默认放宽目标。
- M1验收配方/分母与签章调整由1013负责；本计划与M2路线不自动使旧M1条件通过。

### 本轮取材身份

- docs/specs/overview.md SHA256 bfd17555d7fa0f0c933785fbe505724b1c4a285df046717756eb2d5761f66e69
- docs/specs/auto-test-case/module-selection.md SHA256 6d92d00e617ed3a27e7edf6ff50fdc8368b5ea1856c1218cd93c4a6dd0080bd1
- docs/specs/auto-test-case/ace-test-policy.md SHA256 9b977b100b544f59ea3c2d73e45957647d42c98dacccf6dae2952148dadbf762
- docs/design/55-ace-m2-complete-language-and-product-replacement.md SHA256 e281cb63aae58258da60b0469917e30670cf07af131ab7a758cb19592f1cecc6
- docs/plans/1016-ace-comptime-short-repro-and-cost-repair.md SHA256 5228557dd01c160777b25f540e36ddb4e2d5bb3f2cd82ea03e1f00dd44140409
