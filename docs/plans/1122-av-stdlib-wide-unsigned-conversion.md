---
plan_id: PLAN-1122
status: drafting
feature_name: "AV 标准库宽无符号数值转换与旧方法兼容"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
current_step: 0
total_steps: 5
depends_on: [PLAN-972, PLAN-1079, PLAN-1096]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-core.md, docs/specs/auto-av/language-parity.md]
new_spec_components: []
touched_goals: [GOAL-001, GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1122] AV 标准库宽无符号数值转换与旧方法兼容

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

交付 Windows AV 普通源码上有明确失败结果的 `Str → u64` 转换及必要的旧方法迁移，实际值贯穿局部、参数、全局、运算及插值。

## 2. 架构方案

用已有宽整数、受检运算及所有权通道承载结果。优先扩展共享 `stdlib/ace/asm/conv.at` 的核心算法，方法接面复用现有绑定。以可区分失败的公共返回形保留有效零。

## 3. 技术栈

`stdlib/ace/asm/conv.at`、`experimental/ace/crates/core/src/source/resolve.rs`、旧 `crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs` 及标量 AutoDown 语料。

拟实施树为 `D:/autostack/.wt/lang-1122/auto-lang`，分支 `plan-1122-dev`。

## 4. 需求分析与背景调查

[核心标准库](../specs/ace-migration/m2-stdlib-core.md)当前 `to_int(Str) -> ?int`。[旧 shim](../../crates/auto-lang-core/src/vm/ffi/stdlib/strings_json.rs)的 `Str.to_uint` 签名返回 `i64`，实现为 `trim().parse::<i64>().unwrap_or(0)`。旧 `legacy.25_method_u64.001–011` 还涉及宽结果推导、混合数值、全局和插值，由 [旧用例表](../../crates/auto-lang/test/cases/language/migration/legacy-map.tsv)逐行实证。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

新受检十进制解析承载完整 u64 值域，非法字符、负号和超范围输入显式失败。T-01 固定空白、正号、旧零哨兵和旧隐式提升的兼容表，并决定是否保留 `Str.to_uint()` 的兼容方法或提供显式迁移。旧运行结果与公开合同按行对照，使用标准已有规则处理异常值。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-core.md`](../specs/ace-migration/m2-stdlib-core.md) | 增加受检宽无符号解析/格式化及公开错误合同 | AC-01/02 |
| SD-02 | modify | [`docs/specs/auto-av/language-parity.md`](../specs/auto-av/language-parity.md) | 记录旧方法拼写、返回值和混用的迁移落点 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01 · T-02 | 受检解析 0、5000000000、9007199254740993、18446744073709551615 | 逐值精确。最大值经格式化往返，超过最大值 1、负号、垃圾和空串显式失败。 |
| V-02 | AC-01/02 · T-03 | 旧 25.001–011 的局部、参数、全局读写、比较、插值与混合算术 | 有效 64 位结果及可比输出逐行保持。给旧失败/不兼容形保留原源和精确诊断。 |
| V-03 | AC-02 · T-03 | 旧 `to_int` i32 正负边界和新增 u64 再进入窄类型 | 旧解析结果保持；窄化溢出受检，禁止以截断或浮点近似取得相同显示。 |

冻结验证命令为 `cargo t ace-stdlib ace-source ace-semantics ace-lowering ace-av case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [ ] AC-01 · owner_stage=work。u64 边界、失败和往返经普通 AV 实跑，满足 V-01。
- [ ] AC-02 · owner_stage=work。旧方法族及既有 i32 兼容逐行归档，V-02/03 有独立 oracle。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 冻结旧解析语义与公开迁移形
  - owner_stage=work。运行旧关键输入和当前 AV 近邻，确认方法返回、词法和异常值差异。先固定迁移表，再定公共签名。
- [ ] T-02 实现受检宽解析与输出
  - owner_stage=work。依赖 T-01。沿已有宽数字能力实现精确解析/格式化和必要方法绑定，运行 V-01。
- [ ] T-03 承接旧方法族与窄化反例
  - owner_stage=work。依赖 T-02。接入共享标量语料和原映射，运行 V-02/03。
- [ ] T-04 独立复审
  - owner_stage=review。依赖 T-03。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-05 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1122 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

## 10. 待澄清事项

T-01 关闭 `Str.to_uint()` 的兼容签名与旧哨兵处置。若与已冻结语义冲突，具名定案后再实施该兼容形。
