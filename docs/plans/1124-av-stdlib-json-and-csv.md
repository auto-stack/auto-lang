---
plan_id: PLAN-1124
status: drafting
feature_name: "AV 标准库动态 JSON、泛型编码与 CSV 迁移"
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
current_step: 0
total_steps: 6
depends_on: [PLAN-966, PLAN-975, PLAN-1064, PLAN-1088, PLAN-1096]
supersedes_spec_components: [docs/specs/ace-migration/m2-stdlib-data.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [stdlib, auto-av, auto-semantics]
---

# [PLAN-1124] AV 标准库动态 JSON、泛型编码与 CSV 迁移

## 0. 变更摘要

2026-11-22 起草。承接 [AV 对 AutoVM 的迁移差额](../design/strategy/av-autovm-language-parity.md)的本领域条目，使用 [PLAN-1096 已交语言基线](archive/1096-av-parity-full-language-certification.md)。

## 1. 目标

补齐 Windows AV 普通程序里的动态 JSON 树及常用访问/格式化 API、已准入记录的泛型 JSON 编码，以及旧 CSV 四个公开操作的迁移。

## 2. 架构方案

公共 Auto `std.json`、`std.csv`（拟） → 共享 Auto 算法与受检布局 → AV。重用 975 的语法验证和标量/字符串处理。动态树存储采用现有合适的受检值或有序项序列。泛型编码先核 966 字段事实和 975 的显式 schema/spec 接口。

## 3. 技术栈

`stdlib/ace/std/json.at`、`stdlib/auto/{json.at,encoding/csv.at}`、已验 975 数据组、普通工程数据语料。

拟实施树为 `D:/autostack/.wt/lang-1124/auto-lang`，分支 `plan-1124-dev`。

## 4. 需求分析与背景调查

[975 数据规范](../specs/ace-migration/m2-stdlib-data.md)已验 JSON 语法、路径取值、原语构造和 `decode<T>` 切片，并明确保留完整 `JsonValue`、泛型 encode 和 CSV 差额。旧 [CSV 公共声明](../../stdlib/auto/encoding/csv.at)有默认和自定义分隔符的解析/编码四形。

取材基线为主检出 `f0d9dcaf3` 及同一检出的活动计划。用户本轮授权在 auto-lang 分类立项，计划状态为草案。

## 5. 详细设计

动态值覆盖对象、数组、文本、数值、布尔、null；区分缺键、null、类型错误和坏文档。T-01 冻结键顺序、重复键、数字精度、递归深度/字节预算与失败语义，并用跨模块记录证明编码字段名和类型的实际来源。首批泛型编码需覆盖有代表性的记录、可选字段、嵌套记录及文本/数字；能力边界在 API 上明确。CSV 按旧公开声明的二维数据边界核实引号、分隔符、换行、空字段及错误；保留现有 975 低层和 typed decode API。

### 规范增量

| delta_id | 方式 | 目标 | 规则与理由 | 验收 |
| --- | --- | --- | --- | --- |
| SD-01 | modify | [`docs/specs/ace-migration/m2-stdlib-data.md`](../specs/ace-migration/m2-stdlib-data.md) | 增加动态 JSON、泛型编码和 CSV 的独立公共合同、旧 API 映射及受检范围 | AC-01/02 |

## 6. 测试设计

下列验收尚未执行。用例由对应 work 任务落到真实公开入口，沿 [共享 AutoDown 语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)保留唯一作者。旧结果作迁移参照，新增断言使用独立预期。

| 用例 | 验收与责任任务 | 输入与路径 | 独立预期 |
| --- | --- | --- | --- |
| V-01 | AC-01/02 · T-02 | 解析 `{"n":null,"a":[1.5,"中"]}`、未知键、畸形 JSON 和序列化 | 键/数组/非整数值可访问，区分 null 与缺键；紧凑/缩进输出按冻结精确预期检查，畸形输入明确失败。 |
| V-02 | AC-01/02 · T-03 | 两个跨模块、不同字段形的记录做泛型编码/再读 | 逐字段核独立文本或结构预期，字段重命名使实际输出改变；不满足编码合同的实例具名拒绝。 |
| V-03 | AC-01/02 · T-04 | 默认及自定义分隔符；被引号包住的逗号、换行、空字段、截断 | 解析得预定二维数据，反向输出按冻结字节合同，合法空字段和未结束引号不同；预算或解析错误后临时值可释放。 |

冻结验证命令为 `cargo t stdlib ace-stdlib ace-source ace-semantics ace-av case-av-language-parity`。按实际改面核对固定组和下游，分列实跑、原红、排除与未跑。Windows 在 Windows 环境验收，Linux 已有标准库义务进入 [1063](1063-ace-stdlib-linux-deferred-validation.md) 的独立执行台账。

## 7. 验收标准

- [ ] AC-01 · owner_stage=work。V-01–03 都从普通工程真实调用公共 API，结果与独立预期相符。
- [ ] AC-02 · owner_stage=work。旧接口义务和已验 JSON 近邻逐项保留，错误和资源预算可观察。
- [ ] AC-03 · owner_stage=review。独立复核所需用例、从源码到执行的真实接线、原 oracle 和规范增量，重放关键正反例。
- [ ] AC-04 · owner_stage=merge。冻结受影响固定组证据，按 review 认定的规范增量及交接记录归档。

## 8. 执行步骤

实施前于可用的现有任务跟踪入口登记任务 ID。T-01 可在已归档依赖上先核实基线和公开合同。

- [ ] T-01 核定数据接口、资源合同和编码事实
  - owner_stage=work。重放 975 与旧 CSV/JSON 代表输入。固定数据模型和歧义政策，验证真实字段事实或显式 schema/spec，再冻结兼容表。
- [ ] T-02 实现动态 JSON 读取与格式化
  - owner_stage=work。依赖 T-01。复用解析器已验语义交付动态树和访问方法，验 V-01。
- [ ] T-03 实现泛型 JSON 编码
  - owner_stage=work。依赖 T-01/T-02。经普通模块交独立实例与编码失败，验 V-02。
- [ ] T-04 迁移 CSV 公开操作
  - owner_stage=work。依赖 T-01。落实二维语义、分隔符与错误，验 V-03，汇入公共语料。
- [ ] T-05 独立复审
  - owner_stage=review。依赖 T-04。核 AC-03、SD 增量、实测结果和旧用例映射。
- [ ] T-06 冻结验证与归档
  - owner_stage=merge。依赖复审。执行 §6 固定组并集，核规范与主线结果，折叠证据摘要并按生命周期规则收尾。

## 9. 复审记录

2026-11-22 · stage: new · PLAN-1124 r1 · outcome: pass（草案结构与交接）。next: 计划确认后进入 work/T-01。

## 10. 待澄清事项

T-01 冻结 JSON 数字、对象顺序与重复键；记录 CSV 旧边界及规范化。若编码依赖更广反射，先以现有受检接口落定机制和必要共享差额。
