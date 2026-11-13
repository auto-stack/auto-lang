# ACE正式性能套件成本与复现（PLAN-1017）

> PLAN-1017 SD-01/SD-DETAIL：`cargo t ace-perf` 串行正式套件的成本合同——冻结义务集
> （S0）、端到端计量、同机制短复现、原规模守恒与 r5 修订后的总时长/逐模块预算验收。
> 承接 PLAN-1016 的普通组/PERF 隔离与串行 profile；本文件只辖 ace-perf 正式套件。

## 入口与身份（S0 冻结口径）

唯一正式入口 `cargo t ace-perf`（固定组，scripts/test-groups.toml）：`::performance::`
命名空间、nextest `ace-perf` profile（串行）、父截止 14400s。S0 = 该入口的真实发现
（T-01 冻结：94 例 active/0 ignored，平台/ignore/输入身份齐全；普通 ACE 组排除
`::performance::` 不变）。套件组成变化须重新冻结并记录增量集 S_new（单列覆盖与成本，
不得筛快例/删慢例凑数）。

## 计量与可比性

- 指标 T = 每轮真实需要的夹具生成、准备、IPC、前端/降级/执行、发布/扫描、GC、报告
  准入与清理；一次性 Cargo 冷编译单列，warm 准备仍计入。
- 同机/同工具链/同 profile/同适用条件；正式采样独立时段、单实例，不与其他重活重叠。
  长串行跑中的墙钟门/P90/exe Deadline 膨胀按环境归因记录（隔离重放绿为准），不冒代码
  失败；无统一三 warm/P90 矩阵要求。
- 比较基线 = 932/1016 历史档案 + 本包 Pass B/C 实测台账（r4 裁定：不先行全量 B0）。
- 历史跨提交/跨负载数仅用于排序，不冒当前有效证据；证据复用须同源/同依赖/同配置/同
  输入身份。

## r5 验收目标（用户裁定 2026-11-13，取代十倍目标）

全量 `cargo t ace-perf` 串行执行段 **≤3000s**（进取 2400s）＋逐模块预算（空闲窗口口径，
超支须具名归因）：syntax-artifacts ≤120 / ace-av ≤240 / semantics-store ≤80 /
comptime ≤550 / conformance ≤800 / native ≤1300 / abi ≤320 / artifacts ≤60 /
a2x-worker ≤45（预算合计 ≈3515s>3000 为刻意的头部压力）。

已证不可压缩地板（具名归因候选）：注入延迟（release_gate ~33s）、真实 GC/物理删除、
30 样本 P90 统计义务、冻结字节规模（32/64MiB 水位）、debug 档每样本真实编译/emit/link
（cold_set 填充段 emit ≈3.5s/MiB 实测、250–2000 语句/源每字节成本平坦——源形状重整
不省，PLAN-1017 探针证伪）、closure lane 每调用全源重编译语义（r4 裁定复用=产品契约
变更，留后续设计）。

## 短复现（≤60s 有界探针）

主耗时机制须有常驻 ignored 探针（显式 `--ignored --nocapture` 运行）：同机制缩小输入、
合法控制与可辨退化反例、分阶段计时；外层 60s 预算超界先保阶段进度再减规模。当前常驻：
`resources_short_cost_probe`（调用形状×副牌）、`resources_revision_cost_probe`
（修订循环/直连 put/space 分解）、`cold_fill_shape_probe`（填充源形状）、
`turnaround_hot_phase_probe`（全热请求相位分解）。短例只归因，不代原规模通过。

## 效果守恒（r4 口径，逐例记录）

每例裁剪须记录其所证明的机制/门/状态覆盖不变（同一断言族与状态矩阵）；冻结字节规模
义务与统计有效样本数（P90 百分位语义）不缩；平台/ignore/预算/串行不变。已裁定裁剪：
cold_set churn 512→128、closures_native churn 512→128（逐轮不变量，轮数非冻结规模）、
injected_spike 30→10 样本+仅 body-edit 行、resident_degrade/gate_ms_zero 定向断言行。

## 已交付成本修复（产品面，合同保持）

- 批量发布：`put_artifact_batch`（一批一次 staging/预算/命名空间真实计量/SQLite 提交，
  逐 chunk CAS/锁/活性语义保留）；CTE `publish_frozen_payload_batch` 按请求批发布，
  失败先 flush 已成功产品再传播（逐调用 publish-on-success 契约）。
- 流式记忆化：`stream_snapshot_component` 每 chunk 记忆化（字节恒等）；core `FieldSet`
  排序 Vec（迭代序与诊断恒等）。
- **A17 source-memo**（native）：内容寻址 source-sha×entry×caps×pipeline 轴 →
  link-manifest 指针；命中按 worker-restore 纪律逐 unit 重接纳（symbol 重算+sha/len
  绑定+封套准入）后真链接真运行；一切 miss 信号清洁回落全管线。memo 持久化随清单
  有序发布 supersede（同轴旧 memo 软根即落，被取代链保持可回收）；Fresh 不读、
  ReadOnly 不写、persist 失败计数不失败请求。**语义 pipeline 变更必须 bump
  pipeline 轴常量之一再重键**（同 CTE engine identity 纪律）。

## 证据与复现纪律

正式验收 = 冻结提交上一次完整 `cargo t ace-perf`（真实终态、逐例计时、条件排除/首红/
隔离重放分列）；未执行、首红、环境归因、重放绿分别保留。计划证据不入 git：原始输出在
`plan-artifacts/plan-<NNN>/`，耐久摘要（命令、退出码、关键数字）折回计划文件。
