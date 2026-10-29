---
plan_id: PLAN-873
status: archived
feature_name: 构建成本测量与 workspace member 拆分决策（THR-D1）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-29
plan_revision: 3
current_step: 8
total_steps: 8
supersedes_spec_components: [docs/specs/language-sdk/project.md]
new_spec_components: [docs/specs/repository-boundaries/build-cost-baseline.md]
touched_goals: []              # 有意留空：只测量与决策，不改 GOAL
affects: [language-sdk, auto-ui, auto-man, repository-boundaries, test-infrastructure]
depends_on: [PLAN-867]
---

# [PLAN-873] 构建成本测量与 workspace member 拆分决策（THR-D1）

> **整Plan启动门G873=U867**；所有实施含建树/盘点/测量均等867完整交付。每次实际测量另须M873：已预约独占构建测试窗口、三仓输入冻结、隔离target/cache与污染检测成立；不把逻辑可启动误写成可与其他构建并跑。
> **W873等待门（2026-10-29用户裁定）**：T-02起的实际测量须等PLAN-864/866/880全部merged（计划文件入archive且worktree组折叠）后才开工；T-01准备（协议/初始pin/隔离目录）不受该门约束，已于2026-10-29完成（见§9）。
> 本计划是 [KNOWN-DEBT THR-D1](KNOWN-DEBT-AND-RISKS.md) 的落地：**"先测量，再决定是否按 crate 拆 workspace member"**，不承诺拆分本身。

## 0. 变更摘要

拆分把三仓分开后，**crate 粒度并没有变细**：语言侧仍由 `auto-lang` + `auto-lang-core` 两个根包承载 parser/codegen/engine/native/stdlib/http_server/trans 全部家族（843 只做了文件级分族），UI 侧由 5 包承载（`renderer.rs` 42,337 行仍在 `auto-ui-runtime`），Man 侧 9 成员 + `core/` 子 workspace。THR-D1 的原始观察是：任何改动都会触发整 crate 重编 + 测试二进制重链，2026-10-02 样点为 sccache 79% 命中下 `cargo check -p auto-lang` 57s、单个 worktree `target` 17G、non-cacheable 801 次；当时结论是**"不测不做"**。

本计划交付一份可复跑的构建成本测量协议与基线（冷/热、增量改一处、链接、nextest 启动、全档墙钟），据此建立"member 拆分收益 vs 成本"的决策模型，并给出**拆/不拆/试点**的明确结论与（若拆）下一份实施计划的范围。

## 1. 目标

- **G1 协议可复跑**：冷态/热态、sccache 命中/失效、并发/单实例、重复次数与统计量的定义固定，任何人按记录命令复跑得到同量级结果。
- **G2 基线数字**：三仓各包check冷/热、真实函数体增量、测试二进制编译/链接与枚举/启动分别记录。完整档墙钟只复用同输入已执行的867/后续merge收据；无可比样本明确记不可比/缺测，不为测量重复跑整档或制造绿色。所有数字绑命令/机器/输入/hash。
- **G3 变更站点测量**：在语言侧取 ≥4 个代表改点（parser / codegen / engine / trans 各一），UI 取 ≥2（compiler / runtime），Man 取 ≥1，分别测增量成本，说明"改动爆炸半径"。
- **G4 决策模型**：把"member 拆分"的预期收益（受影响编译单元缩小、测试二进制减少）与成本（manifest/feature 并集、跨 member 循环、CI 矩阵、构建缓存命中性、维护面）量化成同一张表，给出结论。
- **G5 结论可用**：结论含"若拆，先拆哪个 member、验收指标是什么、风险是什么"；若结论为不拆，须给出替代手段（增量/缓存/测试分组）与实测依据。

**非目标**：不执行member拆分、不改现网jobs/linker/栈等配置、不提交产品行为变更、不承诺提速比例。只在隔离测量树做可恢复的临时源变化探针，复位核hash；不在主检出touch/改源/清缓存。测量不是测试门禁豁免：本Plan不新跑裸t、tf/ta/taa/t3或其他整档，完整档墙钟复用有效收据并披露缺测限制。

## 2. 架构方案

1. T-01冻结协议：机器/工具链/三仓commit与锁、配置/features、target与缓存、样本重复次数和统计。只禁止测量窗口内其他构建/测试/重负载，不禁止无关纯文档工作。各次测量前后记录污染检查；锁仅约束合作入口，不能自称能阻止所有外部进程。
2. T-02/03在同一测量树区分三类：无改动热跑；仅时间戳变化（诊断控制，不代表真实编辑）；实际函数体文本变化（≥7站点，源码内容hash变化且实际rustc/编译缓存状态可观测）。公开接口变化只有T-01证明确需时另列受控变体，不混入函数体样本。
3. 冷态采用独立空target与隔离空缓存或仅测量进程关闭wrapper，热态保持同target/cache；不得清空共享sccache。每组从已知内容/缓存状态复位，至少3个有效样本取中位+极差，配置不同的结果分列。
4. 测试二进制构建/链接、nextest list枚举、真实定向测试进程启动和registry初始化分开观测，list不自动证明registry实际运行。完整档墙钟仅引用已合规执行且输入可比的merge收据；样本不足/不兼容如实记缺测，不借测量触发整档。
5. T-04给不拆/部分拆/全拆对照，编译单元缩小与链接收益属估算，未经拆分试点不得冒称实测收益。T-05结论可以是暂不拆、需试点或应拆；真实member实施须另外授权。
6. G873不依赖U869，可先测，但所有样本固定版本；869清单/872变化若尚未落地，结论标输入版本与适用范围。后续结构工作包设计冻结前必须U873并核与最终U869/U872的差额；影响结论时只复测受影响站点，不混算不同三仓快照。


## 3. 技术栈

cargo / rustc / sccache（`--show-stats`）/ `cargo nextest`（list 与 run）/ `cargo build --timings`（可选，用于编译单元拆分观察）/ PowerShell 计时与日志采集 / Python 汇总 / 机器级单实例锁。

## 4. 需求分析与背景调查

本次修订授权（2026-10-28）：用户明确要求按本会话核查建议修订867/869/872/873及总路线，只改主检出计划文档；不执行产品/验证设施改动、不发布canonical/ledger、不建树/agent、不分配874/875或其他新号、不推送。已给的A/B、2,000行批粒度、六族DISTINGUISH裁定继续有效；具体结构性减法批仍沿原签核规则。

### 4.1 授权与轮次

- 用户 2026-10-28 要求把"867 之后的后续计划"写成文档；本计划是其中一份（THR-D1）。
- 本轮只写合同：未跑任何测量、未改配置、未建树。
- **2026-10-29 goal 授权（/auto-plan-goal 完成计划 873）**：用户单条 goal 指令授权整链——
  work（含 T-03 起全部测量与建模）、review、限额内修复、merge（canonical/ledger/归档/清理）；
  各阶段继承授权不再重复请示；不改目标/验收阈值/仓库范围/预算，merge 须绑定当前
  plan_revision 与复审 pass 的提交。T-03 测量会话按此授权入场（§9 2026-10-29 T-03 记录）。

### 4.2 已观察事实（2026-10-28）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | THR-D1 原始观察：单 crate 76.6 万行/1,650 文件（2026-10-02）；sccache 79% 命中下 `cargo check -p auto-lang` 57s；单 worktree `target` 17G；non-cacheable 801 次；结论"不测不做" | [KNOWN-DEBT THR-D1](KNOWN-DEBT-AND-RISKS.md) |
| F-2 | 拆分后 crate 粒度未变细：语言侧 `auto-lang` + `auto-lang-core` 承载 parser/codegen/engine/native/stdlib/http_server/trans 全家族（843 为文件级分族） | [843 handoff](attachments/843/handoff.md) |
| F-3 | UI 侧 5 包 + `shell-pack`；`renderer.rs` 42,337 行（P total）在 `auto-ui-runtime`；生成物 19,687 行 | 844 T-01 基线（`net-slimming.md` 桶表） |
| F-4 | Man 侧根 9 成员 + `core/` 子 workspace（独立 lock/target） | [man-three-repository-delivery.md](../specs/repository-boundaries/man-three-repository-delivery.md) |
| F-5 | 现网构建设置：`.cargo/config.toml` `jobs=12`、`rust-lld`、`RUST_MIN_STACK=16MB`（与其注释一致） | 三仓 `.cargo/config.toml` |
| F-6 | 重型档（gallery 围栏 ~800s 冷、1M churn）与 nextest 注册表税（`ui_gen` 每进程 ~1.2s）是既有已知成本项 | `AGENTS.md` 全档资源表；`cargo tu` 注释 |
| F-7 | 867 将提供三仓完整档与档位口径；869 将提供 per-crate 行数/文件/测试归属（可作为收益模型输入） | [867](867-split-equivalence-test-closure.md)、[869](869-crate-inventory-and-subtraction-batch-2.md) |

### 4.3 规范依据

- [language-sdk/project.md](../specs/language-sdk/project.md)：语言包组与依赖闭包（本计划 SD 落点）。
- [repository-boundaries/project.md](../specs/repository-boundaries/project.md)：三仓身份与依赖方向。
- [net-slimming.md](../specs/repository-boundaries/net-slimming.md)：per-crate 行数口径（收益模型输入）。

### 4.4 假设与不声称

- **A1**：可为本测量进程提供独立target与隔离缓存/关闭wrapper；T-01先验证实际缓存位置/进程身份，无法隔离则阻塞冷态样本，绝不清共享缓存。
- **A2**：`cargo timings` 能给出编译单元级视角；若不可用，退化到 `check`/`build` 墙钟与 target 增量大小。
- **不声称**：本轮未测任何数字；F-1 的 2026-10-02 样点只作历史对照，不是当前基线。

## 5. 详细设计

### 5.1 测量矩阵

| 层 | 对象 | 真实入口与输入 | 观测/关闭条件 |
|---|---|---|---|
| L1冷/热check | 三仓各包 | 隔离空target/cache的check；同输入热跑 | 每组≥3有效样本、中位/极差、真实命令/配置/缓存状态 |
| L2增量三控制 | parser/codegen/engine/trans、UI compiler/runtime、Man各站点 | 无改动、仅mtime、真实函数体文本变化后check | ≥7真实改点；前后源码hash、被重编crate/rustc/cache命中、下游范围；mtime单列 |
| L3测试成本分层 | 各包适用测试目标 | 测试二进制仅编译/链接；nextest list；既有相关定向测试实际启动 | 编译/链接/list/运行/registry成本分开，不用list替代真实运行 |
| L4完整档墙钟 | 867或后继已执行merge收据 | 读取可比完整档命令/冻结输入/机器数据 | 不新跑整档；无可比样本记缺测并限制模型；旧收据重复次数据实 |

每次样本含源/锁/工具链/配置/缓存/机器身份与开始结束时间；污染样本拒收并预约重测。临时变体复位后核整个受控源码集合hash与Git diff；用户WIP不进入测量树，不改主检出。首次协议同时规定变体修改/复位、超时/取消后清理和测量拥有的缓存清理范围，不触共享目录。


### 5.2 决策模型

- **收益**：受影响 crate 的生产行/编译单元/测试 ID 在单点改动后的重编范围缩小比例；测试二进制数量下降带来的链接节省。
- **成本**：member 数量与 manifest 维护、feature 并集变化、跨 member 依赖可能引入的循环（需要拆 trait/接口）、CI 组合数、缓存键粒度变化、发布/安装清单变化。
- **方案对照**：① 不拆（保留现状 + 优化缓存/分组）；② 部分拆（只拆最热 crate 的稳定子域，如 parser 与 trans）；③ 全拆（per-family crate）。
- 结论必须给出推荐方案 + 触发条件（例如"若 L2 中 trans 单点改动 > X 秒且占全档 > Y%，则拆 trans"）。

### 5.3 结论与交接

- work先给build-cost-baseline.md的proposal：协议、基线、模型、缺测限制、结论和复跑命令；merge再发布canonical。U873要求决策依据足够，数据不足以判断则结论“需试点/补测”并明确owner，不输出确定收益。后续语言/UI-Man结构工作包均消费本结论，暂不取874/875号。
- 若结论为"拆"：产出实施计划范围（成员划分、依赖改造点、里程碑、验收指标：冷 check P50、单点增量 P50、裸 t 墙钟），交下一份计划；本计划不实施。

### 5.4 执行合同

| 字段 | 内容 |
|---|---|
| 规则与边界 | G1–G5；只测量与决策；不动现网设置；单实例 |
| 实际路径 | 协议 → 测量 → 原始日志 → 汇总表 → 模型 → 结论 → 规范页/交接 |
| 所有权与状态 | 基线表为验收输入（改需原因+版本）；测量 owner=本计划；结论 owner=用户确认（若涉及拆 member） |
| 区分性用例 | V-01..V-05：协议复跑一致、冷热区分、并发污染拒绝、历史数字隔离、结论可追溯 |
| Oracle 与关闭 | 每 AC 绑命令与期望；关闭需原始日志路径 + 机器状态记录 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/build-cost-baseline.md | 无构建成本基线 → 测量协议 + 三仓基线 + member 拆分决策模型与结论 | THR-D1 要求"数据支撑再立项" | AC-01..AC-04 |
| SD-02 | modify | docs/specs/language-sdk/project.md | 语言包组说明补"crate 粒度与增量成本现状 + 若拆的结论指向" | 现有页只说成员职责 | AC-02, AC-04 |

## 6. 测试设计

`owner_stage` 均为 `work`；本轮未执行。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 协议可复跑 | 同机冻结输入、独立target/cache | L1重复协议 | ≥3有效样本，中位/极差；差异>10%须定位并重测或判不稳定，不强制制造倍率 | 原始本地日志hash/摘要 | 三仓各一包 |
| V-02 | AC-01/03 状态可区分 | 冷/热+无改动/mtime/函数体变化 | L1/L2实际check | 各状态实际缓存/内容hash/编译调用可区分；mtime未重编不能冒充函数体成本；冷未必固定倍数更慢 | 命令/源hash/rustc/cache观测 | ≥7真实函数体站点，API变体另列 |
| V-03 | AC-03 资源隔离 | 并发构建污染/误指共享cache/中断 | M873入场/结束检查与探针复位 | 污染样本拒收；共享cache不清；取消后临时源码复位hash一致，仅清本Plan拥有目录 | 锁/进程/源hash与退出摘要 | 构建测试独占，文档工作可继续 |
| V-04 | AC-04 模型可追溯 | 历史/当前/不同commit及无可比完整档收据 | 汇总模型 | 不混算版本；估算/实测/缺测分列；无收据不能新跑整档充数 | 表/来源hash/限制 | 所有模型行 |
| V-05 | AC-05 结论可用 | 模型输出 | 结论页 | 三方案对照 + 推荐 + 触发条件；若拆，范围可被独立执行 | 结论页 | member 决策 |

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 测量协议冻结且可复跑（V-01/V-02 两态区分成立） |
| AC-02 | work | 三仓check/函数体增量/测试编译链接/枚举启动各层数据或明确适用性齐备，日志路径/hash/命令/机器/有效样本数齐；完整档墙钟仅引用可比既有收据，无样本披露缺测及模型限制 |
| AC-03 | work | ≥7真实函数体改点及无改动/mtime对照，源hash与实际编译调用/缓存观测齐；M873污染拒绝、隔离缓存和复位控制有效 |
| AC-04 | work | 决策模型三方案对照 + 明确推荐 + 触发条件；历史/当前分列、估算标注 |
| AC-05 | work | build-cost-baseline proposal与THR-D1闭合/转阶段依据齐，结论含适用三仓版本及缺测限制；不拆/试点/拆均可，不擅自执行member拆分；交两个未编号结构工作包 |
| AC-06 | review | 独立终审复核协议、数字、模型与结论（含抽点复跑）；merge 门 pending |
| AC-07 | merge | 抽点复审及输入一致性通过、SD/ledger/归档/guard闭环才U873；不因docs测量计划跑整档；实际member实施另行授权 |

## 8. 执行步骤

8 项 = 6 work + 1 review + 1 merge；G873=U867。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [✅] | T-01 | work | G873 | 冻结M873窗口、三仓输入/工具链、隔离target/cache、变体/复位/污染/预算协议；不清共享缓存（协议=[m873-protocol.md](attachments/873/m873-protocol.md) r1；组/pin/隔离目录已建，见§9 2026-10-29） | AC-01 |
| [✅] | T-02 | work | G873；T-01；**W873（864/866/880 merged）**；M873每次入场 | L1与L3构建/链接/list/定向启动测量；完整档只读可比merge收据，无样本记限制，不新跑整档 | AC-02 |
|   |   |   |   | ↳ [✅ 2026-10-29 W873成立（864/866/880均archive）后执行：re-pin（仅auto-lang a5d8a4fa8→a748017da，三仓Cargo.lock不变）；L1冷check中位 lang/-p auto-lang **80.0s**（4有效样本，污染风暴3样本拒收/存疑）、lang-core **55.5s**、ui workspace **84.2s**、man workspace **105.9s**（方差大下界90–95）；热重check全仓 **0.4–0.9s**；L3 no-run冷 lang **196.6/226.6s**、ui-runtime **276.7s**、man/auto **181.1s**；nextest list 1.2–2.9s（枚举数 2026-10-29 终审 F-1 修正为日志实数：lang **1559** 测/22 二进制、ui-runtime **1889** 测/2 二进制、man **34** 测/4 二进制；原记 511/1/10 为 T-02 提取错误）；定向启动 lang 3.2s/ui 1.1s/man 1.4s；L4只读收据 880批量回归 tf 84.0s/tt 512.4s/tb 61.0s@03f851505（输入差866落地，按近似参考标注）。证据=.measure/logs/index.md+35样本log+污染记录；运行器 scripts/measure-build-cost-873.sh（commit 63cb3e1dd，plan-873-dev）] | AC-02 |
| [✅] | T-03 | work | G873；T-02；M873每次入场 | ≥7真实函数体站点+无改动/mtime对照，编译调用/缓存观测，逐站复位hash和污染拒绝。↳ [✅ 2026-10-29：L2 三控制 7 站点 ×3 样本 + L2e 编辑→测试二进制 3 站点 ×3；驱动=scripts/measure-l2-873.sh（c310e296f/07d9578b6）；lang 四站点 body 中位 6.9s/重编 4 crate 跨 2 仓、mtime 5.9s 同链、无改动 0.7s；ui-compiler 4.6s/4 crate、ui-runtime 4.6s/3 crate、man 1.2s/单 crate；L2e no-run：lang 27.4s、ui-runtime 22.9s、man 4.9s；首批样本跨仓 mtime 级联已归因单列（O5）；证据=.measure/logs/index.md T-03 节 + L2-*/L2e-* 样本 log] | AC-03 |
| [✅] | T-04 | work | G873；T-03 | 汇总表 + 三方案决策模型（收益/成本/触发条件）。↳ [✅ 2026-10-29：build-cost-baseline.md §2 基线数字 + §3 三方案模型（收益上界全标注估算：冷≈0/<2s check/10–15s 测试二进制；成本=迁移面/结构风险/CI 面/链地板反压）+ 触发条件 T-A..T-D] | AC-04 |
| [✅] | T-05 | work | G873；T-04 | 结论页 + 若拆的范围草案；THR-D1 闭合或转阶段。↳ [✅ 2026-10-29：结论=**暂不拆**（不立项 member 拆分）；THR-D1 以"已测量+结论闭合"处理；替代手段 A1(ui-iced 默认特性门控，建议独立签核)/A2(sccache 常态)/A3(测试档位已交付)/A4(mtime 级联观察项)；若拆范围草案=先拆 trans（§4.3，含验收指标与风险清单）；适用版本+缺测限制披露（§4.1/§5）] | AC-05 |
| [✅] | T-06 | work | G873；T-05 | SD-01/02 proposal及handoff冻结，写后续未编号结构包适用版本/差额重测门；canonical仅merge。↳ [✅ r1 冻结 601aeb95a（SD-01 新增 + SD-02 增节）；🔄 终审 needs_fix F-1/F-2/F-3 后修复重冻 **41b474e03**：F-1 nextest 枚举实数 lang 1559/22 二进制、ui 1889/2、man **34**/4（复审人原判 man=10 亦沿用旧错，work 从日志重数修正为 34，三样本一致）；F-2 ui-compiler mtime 中位 3.8；F-3 n=2 格与 >10% 极差组披露入 §2.2；同步修 index.md 与 §8 T-02 行] | AC-02,AC-04 |
| [✅] | T-07 | review | G873；T-06冻结提交；抽测须M873 | 独立复审协议/数据/估算与限制，必要独占抽点复测，不触发整档。↳ [✅ 2026-10-29 独立子代理终审（final）：首轮 needs_fix（F-1 nextest 计数/F-2 中位/F-3 统计披露，全文档级）→ 修复重冻 41b474e03 → 受限复审 **pass** 绑定 41b474e03；测量/模型/结论独立复算成立，抽点复测复现（no-change 0.7s 精确、mtime 6.0s +1.7%）；复审人独立确认 man=34（其首轮"10"为正则漏 / 错误，已自纠）；AC-01..AC-06 全达成；merge 门 pending 见 §9] | AC-06 |
| [✅] | T-08 | merge | G873；T-07 pass | 输入差额核验、SD/ledger/归档/逐树guard清理；输出U873和后续工作包设计输入，不取号不实施member拆分。↳ [✅ 2026-10-29：输入差额核验（master 仅 docs 前进，三仓 pin 不变）；SD-01/02 canonical 随交付提交落地 master **54f4ae6e9**（rebase 后等价 range-diff 全 =；reviewed 41b474e03→d670e47fc）；ledger v127 守卫式离线投影（designs P873-DESIGN-BUILD-COST-BASELINE + P840-SDK 溯源、reviews P873-R、reports P873-M）；repository-boundaries/project.md 索引 + language-sdk/plans.md 登记；THR-D1 行闭合改写；归档+清理回执见 §9 merge 记录] | AC-07 |

**实施树**：G873后按需建立 `D:/autostack/.wt/lang-873/{auto-lang,auto-ui,auto-man}`，每个测量仓一个权威树，分支 `codex/plan-873-dev`；同组固定输入，禁止junction/symlink。全部check、编译、定向运行与临时源探针只在组内，设置专属target/cache，不借主检出target、不清共享sccache。每树移除前fresh wt-guard clean；只读原始收据可在主检出核对。

**门禁与证据**：测量M873独占构建测试资源；无Rust交付改动，不跑Cargo整档/docs_gen。原始日志/探针/转储本地保留，md摘要含命令/退出码/数字/本地路径+hash；长期可复跑协议/工具需受管在scripts，不依赖attachments未入库脚本。

## 9. 复审记录

2026-10-29 / stage: merge / plan_id: PLAN-873 / plan_revision: 3 / outcome: **pass（delivered）** / delivery_commit: **54f4ae6e9**（master tip，ff-only） / reviewed_commit: 41b474e03 → rebase 映射 d670e47fc（`git range-diff` 六提交全 `=` 补丁等价） / channel: ledger=守卫式离线投影（无 store writer；备份+脚本改写+diff 零扰动证明+schema/读回验证）：

- **prepared**：reviewed 基线核验（r3 pass 绑 41b474e03，依赖 867 archived）；worktree 内补 repository-boundaries/project.md 索引引用 + ledger v127 投影（f115fee45，docs/projection-only 后裔 → 合法 delivery 候选）；输入差额核验：master a748017da→4cf7bfdb1 仅 docs/plans 簿记，三仓构建输入与测量 pin 一致，无需复测。
- **landed**：rebase plan-873-dev（a748017da 基）到 master 后 `git merge --ff-only` 于主检出，master tip=54f4ae6e9=分支 tip（uniq=1）；落地内容=scripts/measure-build-cost-873.sh、scripts/measure-l2-873.sh、docs/specs/repository-boundaries/build-cost-baseline.md（SD-01）、docs/specs/language-sdk/project.md（SD-02）、.autoos/specs.json v127、repository-boundaries/project.md 索引；零 Rust 产品代码。
- **ledger_refreshed**：v126→127；designs +P873-DESIGN-BUILD-COST-BASELINE（file=build-cost-baseline.md）、P840-SDK 追加 873 溯源 tags；reviews +P873-R；reports +P873-M；全条目带 file/tags；schema 六节校验+读回 ID/内容核验通过；diff 68+/2− 零扰动。
- **回归门禁归属**：AGENTS Category A（未改 crates/ 下 Rust 源码——diff 仅 scripts/shell+docs/specs/ledger）→ 裸 cargo t/docs_gen 按规禁跑，终审双轮确认无 Cargo 门禁到期；批量回归到期判定=**未到期**（收据 .last-batch-regression.json last_covered_plan_id=880 @2026-10-28T18:55Z≈6.5h 前 <48h；873%5=3 非 %5==0；其后无 %5==0 合并与 L0 fix-* 合并）。
- **落地产物观察（landing≠deployment 豁免核验）**：本计划零代码/零配置/零生成物变更——backend release 二进制、依赖仓 release、gen/front/vue/dist 均不受影响，无 stale 产物需要重建/重启。
- **archived**：本文件 git mv 至 docs/plans/archive/，status: archived，current_step=8/8；KNOWN-DEBT THR-D1 行闭合改写（已测+已决：暂不拆，A1 跟进项独立签核）；language-sdk/plans.md 登记 873 行。
- **cleaned**：四树 wt-guard 全 clean（auto-lang/auto-ui/auto-man/auto-down）后移除 worktree 与分支（lang plan-873-dev=54f4ae6e9 已合入；ui/man plan-873-dev=各自 master；另删 lang 侧 rebase 前备份分支）；证据保全=D:/autostack/plan-artifacts/plan-873/（measure-logs 155 文件 34M + measure-pins 双 pin）；.measure 专属 target/cache（~48G）回收；组目录 .wt/lang-873 已移除；plan-artifacts 下无其他 873 中间物（测量均在 .measure，无 .pdb/.exe 产物）。
- **U873**：决策依据充分（协议/三仓基线/模型/触发条件/范围草案，终审独立复算+抽点复现）；THR-D1 闭合；两个未编号结构工作包的设计输入=build-cost-baseline.md §3.3 触发条件 + §4.3 范围草案（设计冻结前核 pin/inventory 差额，§4.1 适用版本门）。

`stage: goal | plan_id: PLAN-873 | plan_revision: r3 | round: 4 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: 54f4ae6e9 | next: done`

2026-10-29 / stage: review（受限复审，final）/ review_scope: final / reviewer: 同一独立子代理上下文（agent_1a32efd1；修复轮同一复审人，仅核 F-1/F-2/F-3 闭合）/ plan_id: PLAN-873 / plan_revision: 3 / reviewed_commit: **41b474e03**（基 601aeb95a）/ outcome: **pass（final）→ reviewed，merge 绑定 41b474e03** / worktree: D:/autostack/.wt/lang-873/auto-lang @ plan-873-dev（树 clean 已核）/ next: merge（T-08）：

- F-1/F-2/F-3 全部闭合：计数 lang 1559/22、ui 1889/2、**man 34/4**（复审人独立重验三样本一致，并确认其首轮"man=10"判定为正则字符类漏 `/` 所致——`auto::bin/auto` 24 行被漏；与 T-02 原错同源，修复正确）；中位 3.8；统计披露段六项全覆盖。簿记同步（index.md/§8 行）核验 ✓。
- 修复 diff 601aeb95a..41b474e03 = 仅 build-cost-baseline.md（+8/−2），零代码/脚本/测试改动，无新到期门禁。
- 非阻塞备注：man-auto mtime 表值 1.2（严格中位 1.15）已由 n=2 披露段覆盖。
- **终审链**：needs_fix（F-1/F-2/F-3）→ 修复 41b474e03 → pass。AC-01..AC-06 全达成；AC-07 归 merge。
- **merge-owned pending**：SD-01/SD-02 canonical 发布 + specs.json/README/overview 回填；KNOWN-DEBT THR-D1 行闭合改写；归档+status 翻转；逐树 wt-guard + worktree/分支/组清理（含 .measure 48G 处置）；无 Cargo 门禁到期（measurement-only，diff 零 Rust 代码——终审双轮确认）。

`stage: goal | plan_id: PLAN-873 | plan_revision: r3 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-873/auto-lang | commit: 41b474e03 | next: merge`

2026-10-29 / stage: work（修复轮，终审 needs_fix 回路由）/ plan_id: PLAN-873 / plan_revision: 3 / outcome: F-1/F-2/F-3 修复完成，重新冻结 / worktree: D:/autostack/.wt/lang-873/auto-lang @ plan-873-dev / code_commit: **41b474e03**（基 601aeb95a，仅 docs/specs/repository-boundaries/build-cost-baseline.md 一个文件）：

- **F-1**（medium）：nextest 枚举计数修正为日志实数——从 L3-list-* 原始日志按条目格式（`<二进制> <测试名>`）重数：lang 1559/22 二进制、ui-runtime 1889/2、man **34**/4（复审报告判 man=10 沿用了 T-02 旧错，本修复以三样本一致的重数为准）；baseline §2.4、index.md L3 节、§8 T-02 行三处同步。时序结论（list 1.2–2.9s 非瓶颈）不变。
- **F-2**（minor）：ui-compiler mtime 干净中位 3.8/3.8/4.2 → **3.8**（原 4.0 系均值误入）；baseline §2.2 表与脚注、index.md 同步。
- **F-3**（minor）：统计披露补入 baseline §2.2——O5 剔除后三格 n=2（man-auto mtime、L2e ui-runtime/man no-run）按披露中位；极差>10% 三组（ui-compiler-restore、man-restore、L1 lang 冷组）归因负载方差/多会话时段；不补测（复审裁定）。
- §9 T-02 历史回执中的 511/1/10 按追加纪律保留原文，以本条与前述修正为准。
- next: review（受限复审 F-1/F-2/F-3 闭合，冻结提交 41b474e03）。

`stage: goal | plan_id: PLAN-873 | plan_revision: r3 | round: 2 | from: work | to: review | status: execution_done | outcome: needs_fix_repaired | worktree: D:/autostack/.wt/lang-873/auto-lang | commit: 41b474e03 | next: review`

2026-10-29 / stage: review / review_scope: final / reviewer: 独立子代理上下文（general-purpose agent，与执行会话无共享历史；实现会话内派发为既有限制）/ plan_id: PLAN-873 / plan_revision: 3 / reviewed_commit: 601aeb95a / outcome: **needs_fix**（F-1 medium/F-2/F-3 minor，全部文档级；测量/模型/结论/抽点复测全部独立复算成立——AC-01/03/04/05 pass，AC-02 partial）：

- 独立复算：151 份原始 log 重算 L1/L2/L2e/L3/L4 吻合；规模数字独立重数一致（301/250,441/32,852、vm 106,479/trans 52,058/parser 12,221、renderer 42,337）；O1 特性链实证（Cargo.toml L83 default 含 auto-ui-runtime/ui-iced）。
- 抽点复测（E）：复测 no-change 第二样 0.7s/0 单元精确复现；mtime 复测 6.0s/4 crate 链复现（+1.7% vs 中位 5.9）；首样 7.0s/4 单元为间隔后链刷新（O2 现象重演）。原始 L2-LANG-base-S1.log 已备份 .orig-pre-review-backup.log。
- findings：F-1 nextest 计数（511/1/10 → 实数；**注：复审报告判 man=10，work 重数修正为 34**）；F-2 ui-compiler mtime 中位 4.0→3.8；F-3 n=2 格与 >10% 极差组披露。证据=.measure/logs/L3-list-*.log。
- 规范增量：SD-01 修 F-1/F-2 后可发布；SD-02 无需改动可发布；frontmatter supersedes/new 与交付一致。
- 冻结 diff 复核：a748017da..601aeb95a = 4 文件（两脚本+两 docs），零产品代码/crates/test/Cargo 变更，无越界。
- merge-owned pending（复审人列出）：文档修正→新冻结（本轮已完成 41b474e03）、canonical 发布+ledger、THR-D1 行闭合、归档、guard 清理、无 Cargo 门禁到期（measurement-only）。

2026-10-29 / stage: work / plan_id: PLAN-873 / plan_revision: 3 / outcome: **pass（T-03..T-06 完成，全部 work 任务交付，execution_done → review）** / worktree: D:/autostack/.wt/lang-873/auto-lang @ plan-873-dev / base: 63cb3e1dd（T-02 冻结）/ code_commit: **601aeb95a**（含 c310e296f L2 驱动、07d9578b6 L2e 扩展）/ dependency_revisions: 组内 auto-ui 4adf2d03、auto-man e6cc06f、auto-down c3464d0（detached）= measurement pin 不变；master a748017da→4cf7bfdb1 仅 docs 簿记（crates/test/Cargo.lock 零变化，未 re-pin，L1/L2/L3 输入可比）/ authorization: 用户 2026-10-29 goal 指令（§4.1）：

- **T-03 L2/L2e 测量**（M873 窗口 08:26–08:55，29min ≤4h）：入场/出场检查全绿（四树 clean、HEAD=pin、rustc 进程 0、.measure 48G≤60G、共享 sccache 零接触）。L2 基线（无改动）三仓 0.6–0.7s/0 单元；7 站点 ×[mtime/body/restore]×3 全部有效（lang 四站点：mtime 5.9–6.0 / body 6.8–7.5 中位 6.9 / 重编 core+auto-lang+ui-compiler+ui-runtime 跨 2 仓 4 crate；ui-compiler body 4.6s/4 crate 仓内；ui-runtime body 4.6s/3 crate；man-auto body 1.2s/单 crate）；L2e 编辑→no-run：lang 27.4s、ui-runtime 22.9s、man 4.9s。**跨仓 mtime 级联归因**（O5）：前序电池 git checkout 复位推新 core/auto-ui 源 mtime → 晚跑仓首样本多编 6–11 单元（ui-compiler-mtime-S1/man-mtime-S1/ui-runtime-norun-S1/man-norun-S1），按机制归因单列、干净样本另行中位，未按污染拒收（非并发重负载）。逐站复位 sha256 核验 + restore 构建全过；探针 diff 断言 1 插入/0 删除全过。
- **关键观测 O1–O5**（建模输入）：O1 `-p auto-lang` 默认特性拖 UI 图形栈（ui-iced→iced/wgpu，冷差 ~25s）；O2 单行改动爆炸半径=4 crate 跨 2 仓且 mtime 链地板（5.9s）主导、内容边际 ~1s，四家族不可区分；O3 man 9 成员冷 check 反最慢（105.9s）——成员数非冷成本杠杆；O4 温态编辑周期秒级 vs 冷态分钟级；O5 跨仓 mtime 级联实测。
- **T-04/T-05**：决策模型三方案对照+收益上界（冷≈0 / check <2s / 测试二进制 10–15s，全标注估算）+成本面（迁移 301 文件/250k 行/32,852 inline 测试、结构风险、CI/发布面、链地板反压）+触发条件 T-A..T-D；**结论=暂不拆**，THR-D1 以"已测量+结论闭合"处理，替代手段 A1（ui-iced 出默认特性，独立签核）为唯一建议跟进项；若拆范围草案=先拆 trans（验收指标+风险清单）；适用版本绑定 pin+inventory 指纹差额披露；缺测限制 5 项入册（per-binary 链接分解/warm-sccache 变体/完整档近似/man 方差/inventory 差额）。
- **T-06**：SD-01=docs/specs/repository-boundaries/build-cost-baseline.md（新增 proposal）、SD-02=language-sdk/project.md 增节；handoff 冻结提交 601aeb95a（树 clean）。
- **AC/V 对照**：AC-01..AC-05 全部 work-owned 达成（V-01 L2 组极差<10% 一致；V-02 三态可区分 0.7/5.9/6.9；V-03 rustc-only 守卫+复位核验+O5 归因；V-04 全数字带 log/pin、历史(2026-10-02 样点)/当前分列、估算标注；V-05 三方案+推荐+触发条件+可独立执行范围草案）。无未闭合 in-contract finding。
- **测试面**：本计划无 Rust 交付改动，按合同 §8 未跑 Cargo 整档/docs_gen；证据=.measure/logs/（L2-*/L2e-* 44 样本 log + index.md T-03 节，本地保留不入库）。
- **merge-owned pending**（不阻塞 review）：T-07 独立复审（AC-06）、T-08 merge（AC-07：输入差额核验、SD-01/02 canonical 发布、ledger upsert、KNOWN-DEBT THR-D1 行闭合改写、归档、逐树 guard 清理）；无 Cargo 门禁到期（docs/measurement-only 计划）。
- next: review（T-07 独立终审，冻结提交 601aeb95a）。

`stage: goal | plan_id: PLAN-873 | plan_revision: r3 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-873/auto-lang | commit: 601aeb95a | next: review`

2026-10-29 / stage: work / plan_id: PLAN-873 / plan_revision: 3 / outcome: T-02完成（W873成立后L1/L3/L4测量），T-03待续 / authorization: 用户goal指令（推进并等待864/866/880 merged后测量）：

- **W873成立过程**：监视期间864/880由各自会话完成review+merge+archive（880另附批量回归收据三腿10263测全绿）；866会话一度停摆90分钟，本会话按/auto-plan:work范式完成接手评估（读合同/核对worktree状态）时会话自续（merge master→T-04→T-05→needs_fix修复→终审pass→merge门禁→archive a748017da），未实际冲突——本会话零写入866树。
- **re-pin**（M873入场04:41）：仅auto-lang前进 a5d8a4fa8→a748017da（97文件+11226/-494=864/866落地+880/873簿记），**三仓Cargo.lock哈希均不变**（等待期零依赖变更，冷态第三方编译成本可比）；auto-ui/auto-man/auto-down原地不动。两份pin并存：T-01基线+measurement pin（.measure/pins/）。
- **L1冷check**（空target、无wrapper、incr=0、≥3有效样本取中位）：lang/-p auto-lang **80.0s**（71.6/80.8/84.3/79.2）；lang/-p auto-lang-core **55.5s**（58.5/55.2/55.5）；ui/--workspace **84.2s**（85.9/80.3/84.2）；man/--workspace **105.9s**（90.2/95.1/131.7/109.9/105.9，方差大，下界90–95，多会话时段如实记录）。**热同输入重check**：0.4–0.9s（四组）。
- **L3分层**：no-run冷（编译+链接）lang 196.6/226.6s、ui-runtime 276.7s、man/auto 181.1s；nextest list稳态1.2–2.9s（枚举数：lang默认面511、ui-runtime 1、man/auto 10）；定向单测真实启动lang 3.2s（S2=18s离群另记）/ui 1.1s/man 1.4s。两轮启动样本因测试名提取错误exit≠0（0.1–1.3s），已按有效名重跑覆盖，失败样本留档不计数。
- **L4只读收据**：880批量回归@03f851505（本机主检出单实例）：tf-language 3202=**84.0s**、tt 3569=**512.4s**、tb 3092=**61.0s**；可比性限制=输入差866落地，按近似参考标注。未新跑任何整档。
- **污染处置（M873 §6）**：04:52–05:09外来构建风暴（三主检出并发rustc/cargo，2.28GB rustc实证）→lang冷S4/S5拒收、S3存疑、man S1拒收；05:43长驻cargo run（auto-todo dev server）卡死双进程守卫→守卫适配为rustc-only并记录追记；补样后各组≥3有效。出场检查：四树clean+HEAD=pin、.measure 20.7G≤60G、共享sccache零接触（全程wrapper unset）。
- **交付物**：运行器scripts/measure-build-cost-873.sh（63cb3e1dd@plan-873-dev，冷/热守卫+隔离target+证据日志头尾）；.measure/logs/35样本log+index.md+pollution记录+双pin。
- 初步观察（待T-03/T-04建模确认）：单包冷check里依赖闭包占大头（core 55.5s vs lang 80.0s，根crate仅+25s）；热增量极小（<1s）；测试二进制no-run是check的2.5–3.5倍；man仓9成员已拆但workspace冷check反而最慢（105.9s）——对"member拆分收益"假设构成反向数据点，建模时须计入。
- next: T-03（≥7真实函数体站点增量测量，M873新窗口）→T-04建模→T-05结论。

2026-10-29 / stage: work / plan_id: PLAN-873 / plan_revision: 3 / outcome: T-01完成（准备阶段），测量待命W873 / authorization: 用户2026-10-29目标指令（先准备协议/固定版本/隔离目录；864/866/880 merged后再测量）：

- **W873等待门入册**：T-02起的实际测量须等PLAN-864/866/880全部merged（archive+组折叠）；T-01准备不受限。
- **隔离组已建**：`D:/autostack/.wt/lang-873/{auto-lang,auto-ui,auto-man,auto-down}`——前三仓`plan-873-dev`分支（基各自master），auto-down detached依赖只读（auto-ui的`../../../auto-down`与auto-man的`../auto-lang` iced_widget patch均落组内兄弟解析）；四树建后`status -s`干净。分支名从r2草案`codex/plan-873-dev`改用本仓AGENTS.md规范的`plan-873-dev`。
- **初始pin（T-01基线）**：auto-lang `a5d8a4fa8` / auto-ui `4adf2d03` / auto-man `e6cc06f` / auto-down `c3464d0`，含三仓Cargo.lock sha256（全表见协议§1.1）。**re-pin门**：测量入场时三仓ff-only快进post-merge master并重记pin，两份pin差额在结论页披露。
- **隔离目录**：`.measure/{target-lang,target-ui,target-man,cache,logs,pins}`；共享sccache（`D:\autostack\.sccache`，用户级RUSTC_WRAPPER+SCCACHE_DIR环境启用）零接触——wrapper控制为每条测量命令强制前缀（协议§2/§5）。
- **M873协议冻结**：`docs/plans/attachments/873/m873-protocol.md` revision 1（窗口纪律/≥3样本中位+极差/L1-L4态定义/L2三控制与复位污染/预算红线/复跑命令骨架/等待门状态）。
- 本轮零测量、零构建、零Cargo命令（preparation-only）；等待门快照：864 drafting 0/7、866 executing round1（T-02自身等864 S/H wiring）、880 drafting 2/10。
- next: W873成立后T-02入场——re-pin、预约M873窗口、L1/L3测量。



本轮合同预检（非实现复审/测试）：章节0–10、单一任务表、任务DAG无环、编号唯一、AC/V/SD引用及frontmatter依赖检查通过；本Plan 8任务/7 AC/5 V/2 SD。来源内容hash：`docs/specs/language-sdk/project.md` sha256 `d678cdd89dabac7a1f5b5537698ced5a3b8cc9e98a510e6add45579051ce888e`。仅文档校验，未运行Cargo/docs_gen。

2026-10-28 / stage: new / plan_id: PLAN-873 / plan_revision: 2 / outcome: blocked（合同就绪，实施等U867；实际测量还需M873） / authorization: 用户本会话修订授权 / input_sha256: `0e54b88e40569c0457578af8602c429035b0caadc5a74c5dd38290f2121f7334` / changed: G873/M873、§2/5测量分层、V-01..04、AC-02/03/05/07、T-01..03/06..08，专用树/cache及整档不重复 / progress: drafting 0/8保持 / next: U867后T-01预约窗口；873不等U869，但测量输入固定，后续结构设计冻结前核U869/U872最终差额并必要定向复测。旧touch代表编辑、清共享缓存、主检出测量、测量期所有agent停工和新跑裸t要求只作历史，已被本r2替代。未测数字/改实现/建树/发布canonical/ledger/推送。

2026-10-28 / stage: new / plan_id: PLAN-873 / revision: 1 / outcome: **pass（合同可执行；实施等 U867）** / next: 门成立后 work T-01 / changed: 新建 T-01..T-08、AC-01..AC-07、V-01..V-05、SD-01..SD-02。

- 取号：`scripts/new-plan.sh build-cost-and-member-split-decision`（873 → 874）。
- 本轮只写合同：未测任何数字、未改配置、未建树。
- 说明：本计划不承诺拆分；结论可能是"不拆 + 优化缓存/分组"，届时 THR-D1 按结论闭合并把替代手段登记。

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | 冷态缓存隔离 | 测量真实性及他方缓存 | r2已定：专属空target/cache或仅测量进程关闭wrapper，禁止清共享sccache；T-01验证隔离，否则阻塞样本 |
| D-2 | 是否把 CI 矩阵/发布清单成本纳入模型 | 结论保守度 | 纳入但标注为估算 |
| D-3 | 若结论为"拆"，是否立即立项 | 排期 | 交用户决定；本计划只给范围草案 |
