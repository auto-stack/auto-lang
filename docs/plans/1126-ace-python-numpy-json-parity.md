---
plan_id: PLAN-1126
status: drafting
feature_name: ACE Python NumPy 与 json 同源验收
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-22
plan_revision: 1
depends_on: [PLAN-1107, PLAN-1094, PLAN-1110]
supersedes_spec_components:
  - docs/specs/auto-a2x/python-ecosystem-integration.md
  - docs/specs/auto-av/python-interop.md
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace/source, ace/av, ace/a2x-python, auto-test-case]
current_step: 0
total_steps: 6
---

# [PLAN-1126] ACE Python NumPy 与 json 同源验收

## 0. 变更摘要

2026-11-22，用户要求在 1110 后验证 NumPy 与 Python 标准库 `json`。正例用一份 Auto 源同时调用两库，AV 和 A2PY 各自真实执行，再与手写原生 Python 参考比较。两种异常分别用独立的同源 Case 验收。本轮交付计划文档。

## 1. 目标

- NumPy 覆盖 `arange`、`reshape`、广播、索引、带步长切片、形状读取和聚合。数组留在 Python 端，Auto 显式提取验收所需的小值。
- `json` 覆盖 `loads`、字符串键和数组索引、嵌套叶子读取，以及 Unicode 序列化与结构往返。容器沿现有 Python 对象语义传递。
- 分别用真实 NumPy 和 json 异常验收类型、捕获或未捕获结果，以及原始 Auto 失败位置。
- 首批范围为 Windows x64 和固定的兼容 CPython/NumPy 组合。通用 Auto 容器转换、任意维度切片语法、零复制数组存储与跨平台矩阵另行规划。

## 2. 架构方案

一份 Auto 源 → ACE 共享受检前门和 `PyOpFact` → AV 的 CPython provider 或 A2PY 生成的原生 Python 文件集 → 实际运行结果。父侧以手写 Python 参考核值、文本及异常原类，用原 Auto 源核错误位置。两库由 `use.py` 导入，数组计算与 JSON 编解码交给实际库。先验证现有对象操作，发现差额时补通用协议。

## 3. 技术栈

ACE 共享前门、967 Python provider、`ace-a2x-python`、1110 runner、AutoDown TestCase。AV 使用装配的 CPython，A2PY 和独立参考使用声明的 OS Python。先从 1107 已验 CPython 3.14.2 基线盘点，NumPy 和双方 Python 版本、构建及包来源由 T-01 按实际环境冻结。

## 4. 需求分析与背景调查

主检出基线 `b2a616c779aaafb1c97029787c88ef4343392785`。用户授权新增计划文档，示例与验证均待实施。[Specs 总览](../specs/overview.md)

| 来源 | 现状与职责 |
|---|---|
| [1107 实测规范](../specs/auto-a2x/python-ecosystem-integration.md) | 已交 PyTorch 同源训练、双端 Python 操作和 SDK 基线。保留为回归。 |
| [1094 AV 规范](../specs/auto-av/python-interop.md)、[NumPy 现有语料](../../crates/auto-lang/test/cases/language/script/python-context.case.ad) | 1094 已归档。AV 已有 `arange(6).len()` 的实跑证据。完整数组和 JSON 流水线需要另验。 |
| [1110](1110-ace-a2py-python-exception-mapping.md) | 当前 `execution_done` 4/6，专属树实现提交 `ece92f65b`，独立复审和合入待完成。本包实施消费正式落主的异常、原位置及双端检查接口。 |
| [共享前门](../../experimental/ace/crates/core/src/source/resolve.rs)、[AV provider](../../experimental/ace/crates/foreign-contract/py/py_bridge_provider.rs)、[A2PY lowering](../../experimental/ace/crates/a2x-python/src/lower.rs) | 已有 `GetItem` 受检操作。AV 将带 tag 的索引值传给 Python get-item，A2PY 生成原生索引 helper。字符串键、`slice` 对象和属性派生对象的组合需双端实证。 |
| [PLAN-461](archive/461-python-sci-parity.md)、[PLAN-1124](1124-av-stdlib-json-and-csv.md) | 461 的旧科学库语料可作输入参考。1124 负责 Auto 自带 `std.json` 和 CSV，1126 负责调用 Python 标准库 `json`，各自验收。 |

## 5. 详细设计

以下候选源描述用例目标，开工按 T-01 核实可执行语法和差额。切片用已有索引表达式形态，索引值由 Python `slice` 构造。

```auto
use.py numpy: arange
use.py builtins: slice
use.py json: loads, dumps

fn main() int {
    var base = arange(6).reshape(2, 3)
    var a = base + arange(3)
    var shape = a.shape
    var row = a[1]
    var selected = row[slice(0, 3, 2)]
    print(shape[0].to(int))
    print(shape[1].to(int))
    print(row[2].to(int))
    print(a.sum().to(int))
    print(selected.sum().to(int))
    print(a.mean().to(float))

    var data = loads("{\"values\":[2,3,5],\"label\":\"雪\",\"meta\":{\"enabled\":true}}")
    var values = data["values"]
    var meta = data["meta"]
    print(values[2].to(int))
    print(meta["enabled"].to(bool))
    var encoded = dumps(data, sort_keys: true, ensure_ascii: false)
    var decoded = loads(encoded)
    print(decoded["label"].to(str))
    print(encoded.to(str))
    print(dumps(decoded, sort_keys: true, ensure_ascii: false).to(str))
    return 0
}
```

数组形状、切片和 JSON 对象按 Python 协议读取。若出现类型差额，沿受检对象索引扩整数、文本或 Python 对象索引值，保持求值次数和源序。结果对象使用 AV 现有句柄或 A2PY 原生引用，Auto 仅显式转换所需叶子。JSON 固定编码选项可比精确文本，解析后的树按语义比较。

NumPy 形状不相容与坏 JSON 分别核真实 `ValueError`、`JSONDecodeError`。T-01 从 1110 合入代码确认其 Auto 类别，保留真实 Python 原类。如现有按类名映射把 `JSONDecodeError` 归为 `other`，两端按该现行规则断言。解释器、包和授权缺失沿现有准入路径，运行资源错误保持原类别。

### 规范增量

| delta_id | 操作 | 目标 | 规则增量 | 验收 |
|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-a2x/python-ecosystem-integration.md` | 追加实测 NumPy/json 对象操作、原生参考和运行环境 | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-av/python-interop.md` | 若实际新增索引形态，追加受检操作和 AV 支持边界 | AC-01/02/04 |

## 6. 测试设计

拟新增 `crates/auto-lang/test/cases/language/script/python-libraries.case.ad`。每个 Case 只有一份 Auto 作者源，两个运行检查使用同一逻辑路径和原始字节。原生参考直接调用 NumPy 和 `json` API，由独立 OS Python 运行，输入与参数固定。父侧分别用两端的实值和参考比对，并核源码指纹、解释器/包身份与实际执行记录。原生参考和预期均独立于 ACE 产物。

| 用例 | 三方预期与区分性 | 对应任务 |
|---|---|---|
| V-01 NumPy 正例 | 广播后 `[[0,2,4],[3,5,7]]`。形状 `2,3`、元素 `7`、全量和 `21`、切片和 `10` 精确相同，均值 `3.5` 采用 `rtol=1e-6, atol=1e-6`。原生参考用 NumPy 原生切片独立计算。 | T-02/04，AC-01/04 |
| V-02 json 正例 | 同一个正例读取 `5`、`true`、`雪`。首次编码和解码后再编码都得到固定 UTF-8 输出 `{"label": "雪", "meta": {"enabled": true}, "values": [2, 3, 5]}`。原生参考另外断言解码树与原输入结构相等。 | T-03/04，AC-02/04 |
| V-03 真实异常 | `arange(6).reshape(4, 2)` 触发 `ValueError`，Auto catch 一次并继续。坏 JSON `loads("{")` 触发 `JSONDecodeError`，Auto 报未捕获诊断，A2PY 实际退出非零。原生 Python 核原类，AV/A2PY 同时核 Auto 分类与原源中失败操作的范围。替换错误类别或范围必须使验收失败。 | T-02..04，AC-03/04 |
| V-04 入口与回归 | 改错切片和、JSON 字段或源指纹均令验收失败。缺 NumPy、未运行或只发射不会满足执行门。保留 1107 PyTorch、1110 异常以及纯 Auto 索引和文本回归。 | T-04，AC-04 |

AV 用 `case-av-language-parity` 与普通源码执行入口。A2PY 消费 1110 正式交付的检查和 `run-py` 路径，T-01 记录实际 profile、解释器和命令。改动面候选固定组为 `ace-python`、`ace-a2x`、`ace-adapters`，共享改动按实际加 `ace-source`、`ace-lowering`。新增用例单独计时。检查只消费已显式准备的 worker、解释器与 NumPy，Windows 按现有规则排除 Linux/WSL。

## 7. 验收标准

- [ ] AC-01（owner_stage=work）NumPy 结果在两端各自通过 V-01，独立参考与固定值一致。
- [ ] AC-02（owner_stage=work）json 结果在两端各自通过 V-02，固定序列化及往返结构成立。
- [ ] AC-03（owner_stage=work）V-03 有两种异常的三方原类证据及两端实际分类、处理结果和原位置。
- [ ] AC-04（owner_stage=work）所有必验 Case 有相同源身份和双端真运行，V-04 反例、回归与支持矩阵有证据。
- [ ] AC-05（owner_stage=review）独立核提交、验收和规范增量，检查同源、oracle 独立性及运行身份。
- [ ] AC-06（owner_stage=merge）执行冻结职责组与必要消费者门，沉淀实测规范、归档与清理。

## 8. 执行步骤

实施前在可用的 TODO 工具登记任务。拟用专属树 `D:/autostack/.wt/lang-1126/auto-lang`，分支 `plan-1126-dev`。开工登记唯一材料目录 `D:/autostack/plan-artifacts/plan-1126/`。

- [ ] T-01 冻结环境、接口与预期
  - owner_stage: work。核 1107/1094，并在 1110 正式合入后核异常与执行路径。用 §5 正例的最小片段核已交范围和差额。冻结 NumPy 版本、索引形态、错误类别和实际验证命令。关联 AC-01..04。
- [ ] T-02 验证 NumPy 对象操作
  - owner_stage: work。前置 T-01。先验已有对象操作，确有缺口时沿共享前门、provider 和 A2PY lowering 扩通用形态。关闭 V-01 和 NumPy 负例。关联 AC-01/03。
- [ ] T-03 验证 json 对象与 Unicode
  - owner_stage: work。前置 T-01。核键访问、嵌套值及 Unicode，按实际差额接通。关闭 V-02 和 json 负例。关联 AC-02/03。
- [ ] T-04 交付共享用例与参考
  - owner_stage: work。前置 T-02/03。落唯一作者源、两端真实检查和手写 Python 参考。关闭 V-01..04，记录输入身份和各用例耗时。关联 AC-01..04。
- [ ] T-05 独立复审
  - owner_stage: review。前置 T-04。冻结提交核验 AC-01..04、负控、资源及规范增量。关联 AC-05。
- [ ] T-06 合入归档
  - owner_stage: merge。前置 T-05 pass。冻结门、规范沉淀、归档和材料对账。移树前用 `pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <worktree>` 核对。关联 AC-06。

## 9. 复审记录

2026-11-22，`stage: new | PLAN-1126 r1 | status: drafting | progress: 0/6`。下一阶段入口为实施授权后的 T-01，所依赖的双端异常接口由 1110 正式交付。本轮没有产品运行证据。

## 10. 待澄清事项

T-01 核准实际 Python/NumPy 组合、现有字符串和对象索引的覆盖、精确错误分类及两端命令。候选源码为验收合同。源语法差额、共享语义或宿主边界变化按实际证据写回。

