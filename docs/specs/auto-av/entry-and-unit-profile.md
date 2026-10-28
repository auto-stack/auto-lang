# av.entry-unit/1 — 入口/Unit 执行 profile

> **Status**: experimental；执行 profile `av.entry-unit/1`；MIR schema
> `auto.mir.av-entry-unit/1`；ABC schema `auto.abc.av-entry-unit/1`（新 magic 家族
> `AVAU`，wire version 1，ABC profile id **7**）；producer id
> `plan862/av-entry-unit`。
> 实现：`experimental/auto-mir-lower`（HIR→MIR）、`experimental/ac-av-probe/core`
> （MIR/ABC/AV）、`probe`（worker 会话）；源码侧见
> [auto-ac/source-entry-unit.md](../auto-ac/source-entry-unit.md)。
> 来源：[PLAN-862](../../plans/archive/862-av-source-entry-unit-and-tail-semantics.md)
> 与 T-02 冻结合同（`docs/plans/attachments/862/work/entry-contract.md`）。

本 profile 是 AV 的第七个独立执行面：它不改动 `av.core-i32/1`（profile id 1 /
AVAB）与 `av.data|owned|collections|dynamic|closures`（AVAD v1–v5）的任何字节或
语义，而是给出**入口类别 + Unit 结果通道**的新域与独立身份三元组。

## 结果通道（互不替代）

| 通道 | 表示 | 判别 |
|---|---|---|
| 值 | `Success { value: Value::I32(v) }` | 与 Unit 不同载荷 |
| Unit | `Success { value: Value::Unit }` | **语义上不等于 `I32(0)`**；无 i32 形 |
| 语言 trap | `Trap { kind, span }` | 从不折叠为值或 Err |
| 资源 / 取消 / infra | `Resource` / `Cancelled` / `Infra` | 各自独立，语义与既有 profile 相同 |

- `Value::Unit` 是**真值**（无载荷、零运行时宽度）：不能与 `I32(0)` 互相比较、
  参与算术/相等判定；交叉使用即拒绝或 trap，永不隐式零化。
- 通道是**可观察的**：probe 的 run 步上报 `channel`（`value` / `unit`），父侧
  以 `av_channel` 期望判定；消费者侧退出码映射（Unit→0，且必须同时打印通道名）
  属**消费者合同**，不写进 AV 语义。
- 入口实参通道：入口零参数；驱动传入实参 → `Infra`（既有行为），不是语言类型错。

## 入口类别（wire 合同）

- `EntryKind::{Int, Unit}` 写入 MIR 模块头与新 wire 家族头部。
- 校验 `mir.entry-kind`：kind 必须与入口签名一致（`Int`↔i32 结果、`Unit`↔Unit
  结果），**双向**检查；kind 缺失、kind 与签名不符、kind 与结果皆非法 → 受控
  拒绝，不允许按签名猜成另一 kind。
- 源码形态不进入 wire：带 i32 尾值的脚本与 `fn main() int` 在 wire 上同为
  "零参 i32 入口"，运行期行为相同；脚本/Unit 的差异只在源码域体现。

## Unit 的 IR 表示

- MIR：`MirType::Unit`（wire tag 18，仅该家族）**零运行时宽度**，不占 frame
  slot 字节；`mir.unit-placement` 只允许它作 unit 入口的 `sig.result` 与该函数
  唯一 `Return` 的操作数（slot 若为 Param 或出现在非 unit 入口即拒绝）。
- ABC：`TY_UNIT` 只在新家族合法；旧家族编码 Unit → `abc.encode.profile`，新
  家族读到未知 tag → `abc.decode.type`；`AVAU` 与 AVAB/AVAD 家族**互不可读**，
  旧 owner schema 会先被 `mir.schema-profile` 拒绝。

## 恢复与缓存准入

- 新进程恢复（`av_probe_worker restore-run`）：每个 envelope 重 decode + 重
  verify，入口 kind 判定必须复现；缓存/存储 envelope 身份必须含 profile id 与
  三轴语义版本，"字节可解析"不能替代准入。
- cache 命名空间按 profile 独立：同一 store 中的 AVAB/AVAD 行对本 profile 是
  确定性 miss，绝不重解释。
- 增量：只重建受影响 unit（未改函数为 store 命中），AV 在命中路径仍执行（不缓存
  PASS）。
- 执行预算（fuel/取消/资源上限）与既有 AV 执行器一致，本 profile 不提高上限，
  也不新增分配通道（Unit 无载荷、无存储）。

## 验证与证据

- `experimental/ac-av-probe/probe/tests/av_entry_862.rs`（13 例）：三形态通道、
  Unit≠`I32(0)`、trap 独立、篡改 kind 被 `mir.entry-kind` 拒绝、`mir.unit-placement`
  负例、scalar 控制组、store fresh→hit、真新进程恢复、body-edit 只重建被改 unit、
  请求峰值内存采样、`AVAU` 被旧 owner schema 拒绝。
- 语料：`crates/auto-lang/test/cases/ace-m1/entry-862/`（U01–U10）；
  门 `scripts/verify-av-862.ps1 -Scope entry`（12 检查 + 判别力反控）。
