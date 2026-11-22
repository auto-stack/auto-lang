---
plan_id: PLAN-1126
status: archived
feature_name: ACE Python NumPy 与 json 同源验收
author: [Codex]
created_at: 2026-11-22
updated_at: 2026-11-23
plan_revision: 1
depends_on: [PLAN-1107, PLAN-1094, PLAN-1110]
supersedes_spec_components:
  - docs/specs/auto-a2x/python-ecosystem-integration.md
  - docs/specs/auto-av/python-interop.md
  - docs/specs/auto-test-case/feature-organized-corpus.md
new_spec_components: []
touched_goals: [GOAL-002]
affects: [ace/source, ace/av, ace/a2x-python, auto-test-case]
current_step: 5
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

主检出基线 `b2a616c779aaafb1c97029787c88ef4343392785`。用户授权新增计划文档，示例与验证均待实施。[Specs 总览](../../specs/overview.md)

2026-11-23 实施授权：用户以 `/goal 完成计划1126` 授权全链（专属树内实施、独立复审、合同内有界修复、合入与归档清理），不含变更目标/验收/兼容承诺/范围/预算。开工基面 `d4bc60a89`。

| 来源 | 现状与职责 |
|---|---|
| [1107 实测规范](../../specs/auto-a2x/python-ecosystem-integration.md) | 已交 PyTorch 同源训练、双端 Python 操作和 SDK 基线。保留为回归。 |
| [1094 AV 规范](../../specs/auto-av/python-interop.md)、[NumPy 现有语料](../../../crates/auto-lang/test/cases/language/script/python-context.case.ad) | 1094 已归档。AV 已有 `arange(6).len()` 的实跑证据。完整数组和 JSON 流水线需要另验。 |
| [1110](1110-ace-a2py-python-exception-mapping.md) | 当前 `execution_done` 4/6，专属树实现提交 `ece92f65b`，独立复审和合入待完成。本包实施消费正式落主的异常、原位置及双端检查接口。 |
| [共享前门](../../../experimental/ace/crates/core/src/source/resolve.rs)、[AV provider](../../../experimental/ace/crates/foreign-contract/py/py_bridge_provider.rs)、[A2PY lowering](../../../experimental/ace/crates/a2x-python/src/lower.rs) | 已有 `GetItem` 受检操作。AV 将带 tag 的索引值传给 Python get-item，A2PY 生成原生索引 helper。字符串键、`slice` 对象和属性派生对象的组合需双端实证。 |
| [PLAN-461](461-python-sci-parity.md)、[PLAN-1124](../1124-av-stdlib-json-and-csv.md) | 461 的旧科学库语料可作输入参考。1124 负责 Auto 自带 `std.json` 和 CSV，1126 负责调用 Python 标准库 `json`，各自验收。 |

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
| SD-02 | modify | `docs/specs/auto-av/python-interop.md` | 实测未新增索引形态或受检操作；改为追加 AV 支持边界的实测事实 | AC-01/02/04 |
| SD-03 | modify | `docs/specs/auto-test-case/feature-organized-corpus.md` | 实施中新增：登记 `python-libraries.case.ad` 唯一作者及其反控（work 阶段按实际改动补入） | AC-04 |

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

- [x] AC-01（owner_stage=work）NumPy 结果在两端各自通过 V-01，独立参考与固定值一致。
  - 证据（§9「T-02/T-03/T-04 交付记录」）：`numpy-json` 与 `numpy-shape-error-caught` 的 av/a2py 四行在固定组内 PASS；独立参考按值等于冻结 oracle。A2PY 两行以声明解释器（自带 NumPy）运行，持久环境已按 §10 Q-01 裁定 (a) 就位。
- [x] AC-02（owner_stage=work）json 结果在两端各自通过 V-02，固定序列化及往返结构成立。
  - 证据：`numpy-json` 的 json 部分与 `json-decode-error-uncaught` 双端 PASS；参考另断言解码树与原输入结构、类型相等。
- [x] AC-03（owner_stage=work）V-03 有两种异常的三方原类证据及两端实际分类、处理结果和原位置。
  - 证据：`ValueError`→`python.5`/`76..99`/handler 恰一次；`JSONDecodeError`→`python.10`/`50..60`/未捕获；参考侧为 NumPy 原始异常文本与解释器 traceback。
- [x] AC-04（owner_stage=work）所有必验 Case 有相同源身份和双端真运行，V-04 反例、回归与支持矩阵有证据。
  - 证据：每 Case 两端 `identity.source` 相同；测试层 M1..M8、运行器层 C1..C10 负控全部按预期；基线回归仅 2 条既有 1093 红；支持矩阵写入 SD-01/SD-02。
- [x] AC-05（owner_stage=review）独立核提交、验收和规范增量，检查同源、oracle 独立性及运行身份。
- [ ] AC-06（owner_stage=merge）执行冻结职责组与必要消费者门，沉淀实测规范、归档与清理。

## 8. 执行步骤

实施前在可用的 TODO 工具登记任务。拟用专属树 `D:/autostack/.wt/lang-1126/auto-lang`，分支 `plan-1126-dev`。开工登记唯一材料目录 `D:/autostack/plan-artifacts/plan-1126/`。

- [x] T-01 冻结环境、接口与预期
  - owner_stage: work。核 1107/1094，并在 1110 正式合入后核异常与执行路径。用 §5 正例的最小片段核已交范围和差额。冻结 NumPy 版本、索引形态、错误类别和实际验证命令。关联 AC-01..04。
  - 证据（2026-11-23，§9「T-01 冻结记录」）：依赖 1094@r5/1107@r2/1110@r2 均已归档落主；语言面零差额；A2PY 隔离腿的 NumPy 来源为环境差额，已由用户裁定 (a)（§10 Q-01）。
- [x] T-02 验证 NumPy 对象操作
  - owner_stage: work。前置 T-01。先验已有对象操作，确有缺口时沿共享前门、provider 和 A2PY lowering 扩通用形态。关闭 V-01 和 NumPy 负例。关联 AC-01/03。
  - 证据：已有对象操作无缺口，未扩任何形态。V-01 与 NumPy 负例双端 PASS（§9）。
- [x] T-03 验证 json 对象与 Unicode
  - owner_stage: work。前置 T-01。核键访问、嵌套值及 Unicode，按实际差额接通。关闭 V-02 和 json 负例。关联 AC-02/03。
  - 证据：键访问、嵌套值与 Unicode 无差额。V-02 与 json 负例双端 PASS（§9）。
- [x] T-04 交付共享用例与参考
  - owner_stage: work。前置 T-02/03。落唯一作者源、两端真实检查和手写 Python 参考。关闭 V-01..04，记录输入身份和各用例耗时。关联 AC-01..04。
  - 证据：实现提交 `9039eb7c8`、规范草稿 `6ca8c483e`；V-01..04、输入身份、计时表与负控见 §9。
- [x] T-05 独立复审
  - owner_stage: review。前置 T-04。冻结提交核验 AC-01..04、负控、资源及规范增量。关联 AC-05。
- [ ] T-06 合入归档
  - owner_stage: merge。前置 T-05 pass。冻结门、规范沉淀、归档和材料对账。移树前用 `pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <worktree>` 核对。关联 AC-06。

## 9. 复审记录

2026-11-23 开工（goal 授权，见 §4）。base `d4bc60a89`，专属树 `D:/autostack/.wt/lang-1126/auto-lang`，分支 `plan-1126-dev`，auto-down 兄弟 detached `456c86a`。材料目录 `D:/autostack/plan-artifacts/plan-1126/`（已登记 `.retention/policy.json`）。主检出存在他人未提交的 docs 改动（design/strategy、plan 1054、plan 1119 删除）及未注册目录 `.wt/lang-1095`，均不在代码路径，本计划不触碰。

**T-01 冻结记录**（探针源、驱动脚本与原始输出在材料目录 `t01/`）：

- 环境：Windows x64。AV 用进程内 CPython 3.14.2（PyO3，`PYO3_PYTHON=C:\Python314\python.exe`），NumPy 2.5.1 取自用户 site。A2PY `run-py` 按 1042 冻结合同以 `python -I -B a2x_main.py` 隔离运行（见 `docs/specs/auto-a2x/python-stdlib-readiness.md`），默认解析到 `C:\Python314\python.exe`（3.14.2，exe sha256 `fda7026477256845afab371e354c4d512896665f1761939cb5887d0a9dec257a`）。隔离模式下 `utf8_mode=0`，但本机 ANSI 代码页为 cp65001，stdout 仍为 UTF-8（`雪` → `E9 9B AA`）。
- 工具准备（全 debug）：`cargo t case-av-language-parity case-a2py-language-parity --prepare-only` 共 217 s（case-parent 125.2 / av 30.2 / av-providers 34.7 + warm 0.7 / a2x-python-debug 24.5）。开工前核查全机无其他构建/测试作业。
- 基线（`d4bc60a89`，既有语料）：`cargo t case-av-language-parity case-a2py-language-parity` 64.9 s。`case-a2py-language-parity` PASS 4/4；`case-av-language-parity` 529 PASS / 2 FAIL / 190 NOT_RUN / 161 SKIP。2 红与 1110 merge 记录同 ID 同原因（`av.parity.1093.explicit-c-library`、`av.parity.1093.c-header-spelling-rejected`，`foreign.prepared`，1093 既有债务），非本计划所致。
- 语言面差额：零。§5 候选源（878 B，sha256 `10f1afcfe24554b5801d972e4c77bcbf05d405b4034e7f636099d96c63fd7cd6`）经普通 AV 源码入口一次跑通（157 B / 11 行，`host_dispatches=11`）；json 部分经 A2PY `run-py` 一次跑通。int 键、文本键、`slice` 对象键与属性派生对象（`a.shape`）上的索引都由既有 `GetItem` 覆盖（AV `py_getitem` → `obj.get_item`；A2PY `a2x_py_getitem` → `recv[idx]`），未新增操作或索引值类别，故 SD-02 不新增受检操作。
- 环境面差额：隔离模式排除用户 site，本机系统 site 为空（全部第三方包在用户 site）。默认解释器下 A2PY 的 NumPy 行以 `ModuleNotFoundError` / `python.1` / `14..20`（`use.py` 行的 `arange`）退出 1，stdout 为空。以既有声明解释器入口 `A2PY_PYTHON` 指向带 NumPy 的 venv 后，同一 worker 在隔离模式下把候选源与两条异常源全部跑通，输出、类别与位置与 AV 逐项一致。venv 与用户 site 是同一 wheel（`numpy-2.5.1-cp314-cp314-win_amd64.whl`；`_core/_multiarray_umath.cp314-win_amd64.pyd` sha256 前缀 `ab5cea76a0fad4fd` 两处相同）。持久环境已按用户裁定 (a) 就位（§10 Q-01）。
- 错误类别（两端一致，按类型 qualname 精确匹配）：`ValueError`→`python.5`；`JSONDecodeError`→`python.10`（现行规则归 other，不按 `ValueError` 子类归 5）；`ModuleNotFoundError`→`python.1`。失败位置为失败调用表达式整体，已按原源字节复核：`[76,99)` = `arange(6).reshape(4, 2)`，`[50,60)` = `loads("{")`。
- 验证命令：`cargo t case-av-language-parity case-a2py-language-parity --prepare-only` 后 `cargo t … --out <scope>`；单例 `auto-case run --suite … --target av` / `--check a2py`；`auto-case module-case`（负控副本）。

**T-02/T-03/T-04 交付记录**（实现提交 `9039eb7c8`，规范草稿 `6ca8c483e`，均在 `plan-1126-dev`，树干净）：

- 交付物：唯一作者 `crates/auto-lang/test/cases/language/script/python-libraries.case.ad`（6866 B，sha256 前缀 `7478a71493e722dd`；`language.suite.ad` 增一行引用）；独立参考 `crates/auto-lang/test/cases/ace-m2/python-1126/`（README + 3 份手写原生 Python，无 Auto 源副本）；`experimental/ace/crates/a2x-python/tests/py_libraries_1126.rs`（5 例，编译期内嵌作者文件）；规范草稿 SD-01/02/03。编译器、共享前门、provider、A2PY lowering **零改动**。
- 语言差额：T-02/T-03 均无缺口，未扩任何形态。A2PY 程序输出在 Windows 走解释器文本流为 CRLF（`value:` 观察行为原始字节 LF），故 A2PY 流 oracle 用既有的 `compare: "lf"`，AV 流 oracle 保持字节精确。

| Case（源 sha256 前 16） | AV | A2PY | 断言 |
|---|---|---|---|
| `py.libraries.1126.numpy-json`（`10f1afcfe24554b5`） | PASS | PASS | 11 行冻结输出；A2PY 多一行 `value:0`，无语言错误 |
| `py.libraries.1126.numpy-shape-error-caught`（`78e343f1c2ebb276`） | PASS | PASS | 3 行 + `python.5` + `76..99`；A2PY 另核 `ValueError`、`handled=1` |
| `py.libraries.1126.json-decode-error-uncaught`（`e824ea8e110edcbc`） | PASS | PASS | AV：`lang.error` + 原消息 + `python.10` + `50..60`、无输出；A2PY：`JSONDecodeError`、退出 1、stderr 原文 |

- 同源身份：每 Case 的 av/a2py 两行 `identity.source` 相同（`196bf283…` / `36e11fbe…` / `d1ad6a97…`，即 sha256(`main.as\0` + sha256(源字节) + `\0`)）；A2PY 父侧另核 worker 回报的 `source_sha256` 等于暂存字节。
- 三方比对：参考输出与冻结 oracle 按值一致（整数/文本精确，均值 3.5 在 `rtol=atol=1e-6` 内，1e-5 偏差判不一致）。NumPy 原始异常文本 `ValueError: cannot reshape array of size 6 into shape (4,2)` 等于 Auto 行去掉 `caught: PyException ` 前缀；未捕获参考的 traceback 末行 `json.decoder.JSONDecodeError: Expecting property name enclosed in double quotes: line 1 column 2 (char 1)` 等于 AV 消息与 A2PY stderr 去掉 `PyException ` 前缀。
- 负控（原始报告在材料目录 `t04/`）：
  - 测试层变异（`mutation-controls-committed.json`，sha256 前缀 `a5e5d2a02ad604b4`，绑定 `9039eb7c8`；语料每次恢复且 sha256 一致）：M1 错切片和、M2 错 JSON 字段、M3 均值越容差、M8 A2PY oracle 漂移 → 参考比对红；M4 源码一字之差 → 指纹钉住红；M5 错位置 → json 参考 + 指钉红；M6 错类别 → json 参考红；M7 错 error-type → NumPy 参考红。M0 未变异对照 5/5 绿。
  - 运行器层（`runner-controls.json`，`7cdce664b6894944`，真实 AV worker + A2PY worker 裁决）：C1 错切片和、C3 错位置、C4 错类别 → 两端 FAIL；C2 源改 `decoded["labels"]` → 两端 FAIL（`KeyError` / `python.7` / `738..755`）；C5 错 error-type → A2PY FAIL；C7 缺 NumPy（A2PY 默认解释器）→ NumPy 两行 A2PY FAIL（`ModuleNotFoundError` / `python.1` / `14..20`），json 行不受影响；C8 缺 NumPy（AV 设 `PYTHONNOUSERSITE=1`）→ NumPy 两行 AV FAIL；C9 emit-only、VM 兜底、旧 PythonTrans、自备预期、运行配方下的发射操作均被 worker 以 `status: error` 退出 2 拒绝（`c9-emit-only.json`）；C10 未选中的腿为 NOT_RUN。
- 健康检查：新测试文件零编译警告（8 条警告均为 `ace-core` 库 base 存量）、rustfmt 干净、无调试输出；新增文件均为 LF、无 BOM。未新增依赖（boundaries 白名单未动）。
- 组选择：`case-av-language-parity`、`case-a2py-language-parity`（语料的完整职责组与适配器集成）、`ace-a2x`（新测试所在 crate 的完整职责组）。未选 `ace-adapters`/`ace-python`/`ace-source`/`ace-lowering`：本计划不改其下任何代码。最终冻结并集归 review/merge。

本会话实际运行的测试（资源等待 0 s：每次启动前核查全机，均无其他构建/测试作业；全 debug profile，唯 `ace-a2x` 组既有的 release 工具 `a2x` 除外）：

| 模块 / 实际范围 | 结果与执行/排除数 | 准备 | 发现 | 运行 | 总墙钟 |
|---|---|---|---|---|---|
| `cargo nextest -p ace-a2x-python --test py_libraries_1126`（提交后直跑） | 5 run / 5 passed / 0 skipped | 热 0.6 s | 未单计 | 0.264 s | 1.4 s |
| 同上，变异控制 M0..M8（9 次调用） | M0 5/5 绿；M1..M8 各恰好预期测试红 | 增量重编含在内 | — | — | 16.5 s |
| `auto-case module-case` ×3（C0，材料目录副本） | 6/6 PASS | 工具已备 | 含在总墙钟 | 行 0.52–1.51 s | 每 Case 1.5–2.0 s |
| 运行器层负控 C1..C8、C10（17 次调用） | 16 项全部按预期判定 | — | 含在内 | — | 26.6 s |
| `cargo t case-av-language-parity case-a2py-language-parity ace-a2x --prepare-only` | 3 个调用 | 202.4 s（`a2x` 工具 163.3 s） | — | — | 3m23s |
| `--run-prepared`：`case-av-language-parity` | 532 PASS / 2 FAIL（既有红）/ 193 NOT_RUN / 161 SKIP | 复用 | 含在 52.6 s | 52.6 s | — |
| 同上：`case-a2py-language-parity` | 7 PASS / 881 NOT_RUN | 复用 | 含在 10.7 s | 10.7 s | — |
| 同上：`ace-a2x` | 354 run / 354 passed / 7 skipped / 0 failed | 复用 | 未单计 | 83.1 s（计量 92.3 s） | — |
| 三组合计（一次 `--run-prepared`） | `gate_passed=false`，仅因 2 条既有 1093 红 | — | — | — | 171.0 s（墙钟 2m52s） |
| 基线（改动前，同两组 case） | a2py 4/4 PASS；av 529 PASS / 2 FAIL | 217.1 s（首次冷准备） | — | — | 64.9 s |

| 新用例 | 行为与 oracle | profile | 实测 | 结果 / ≤10s |
|---|---|---|---|---|
| `numpy-json/av` | 11 行输出字节精确（广播、形状、切片和、均值、json 往返） | debug worker | 0.93 s | PASS |
| `numpy-json/a2py` | 同 11 行（`compare: lf`）+ `value:0`，无语言错误 | debug worker + 隔离 Python | 1.51 s | PASS |
| `numpy-shape-error-caught/av` | `python.5` + `76..99` + 3 行 | debug worker | 0.93 s | PASS |
| `numpy-shape-error-caught/a2py` | `ValueError` / `python.5` / `76..99` / `handled=1` / 退出 0 | 同上 | 1.08 s | PASS |
| `json-decode-error-uncaught/av` | `lang.error` + 原消息 + `python.10` + `50..60`，无输出 | debug worker | 0.87 s | PASS |
| `json-decode-error-uncaught/a2py` | `JSONDecodeError` / `python.10` / `50..60` / 退出 1 / stderr 原文 | 同上 | 1.01 s | PASS |
| `p1126_v01_v02_native_reference_matches_the_frozen_oracle` | 参考输出按值等于冻结 oracle；两端 oracle 互相一致 | test | 0.145 s | PASS |
| `p1126_v03_native_reference_confirms_the_numpy_valueerror` | NumPy 原始类名与消息等于 Auto 文本；捕获恰一次 | test | 0.271 s | PASS |
| `p1126_v03_native_reference_confirms_the_json_decode_error` | traceback 末行等于 AV 消息与 A2PY stderr | test | 0.210 s | PASS |
| `p1126_v04_sources_are_pinned_and_spans_match_the_authored_bytes` | 3 个源指纹对一字之差敏感；位置等于原源字节范围 | test | 0.029 s | PASS |
| `p1126_v04_the_oracle_rejects_wrong_values` | 7 种错行、错字段名、行数增减、错类型均判不一致 | test | 0.164 s | PASS |

固定组门说明：三组由同一次 `--run-prepared` 运行（`gate-run-01a/results.json`，sha256 前缀 `2f305f1f1b0a1f9d`），环境含 `A2PY_PYTHON` 指向材料目录内的临时 venv。`case-a2py-language-parity` 的 NumPy 两行依赖该声明解释器；用户默认环境下这两行为 FAIL（见 C7）；持久环境已按 Q-01 (a) 就位，两组 case 门在该环境下的复跑待目标恢复后进行。`ace-a2x` 不依赖 `A2PY_PYTHON`：其 5 个新测试以 `AUTO_PYTHON_HOME` 或 `C:\Python314\python.exe`（非隔离，`-B`）运行参考脚本，需要该解释器能导入 NumPy（本机经用户 site 可见；缺包是具名失败，不跳过）。

2026-11-23，`stage: work | plan_id: PLAN-1126 | plan_revision: r1 | outcome: blocked（T-01..04 已交付并提交；等待 Q-01 裁定）| worktree_path: D:/autostack/.wt/lang-1126/auto-lang | branch: plan-1126-dev | base_commit: d4bc60a89 | code_commit: 9039eb7c8（实现）/ 6ca8c483e（规范草稿，docs-only）| dependency_revisions: 1094@r5 / 1107@r2 / 1110@r2（均为 base 祖先）| task_ids: T-01..04 | evidence: 见上「T-01 冻结记录」「T-02/T-03/T-04 交付记录」| blockers: Q-01——A2PY 隔离腿的 NumPy 来源；用户默认环境下 `case-a2py-language-parity` 的 NumPy 两行 FAIL | next_responsible_role: user（裁定 Q-01）→ work（在持久环境下复跑两组 case 门后交 review）`。

未执行/待后续：review 与 merge 所属的冻结并集在冻结提交上的最终运行；`ace-adapters` 等未选组的理由见上。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 1 | from: drafting | to: executing | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e（实现 9039eb7c8）| next: user（Q-01 裁定后 → work 复跑两组 case 门 → review）`。本轮未对机器环境做任何改动。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。等待轮：未做阶段工作、未跑检查。重读结果——§10 无答复；用户级/系统级 `A2PY_PYTHON` 均为空，系统 site 无 NumPy，建议的 venv 目录不存在（即 (a)/(b) 未获授权、(c) 未发生）；专属树在 `6ca8c483e` 且干净。连续等待第 2 轮。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。等待轮：未做阶段工作、未跑检查；重读结果同第 2 轮（§10 无答复、环境未配置、专属树 `6ca8c483e` 干净）。连续等待 3 轮，按规则记 blocked；确切问题与解锁操作见 §10 Q-01，回复 (a)/(b)/(e) 即为授权，或自行配置后告知（(c)）。保留的检查点：专属树与分支 `plan-1126-dev`（实现 `9039eb7c8`、规范草稿 `6ca8c483e`）、材料目录 `D:/autostack/plan-artifacts/plan-1126/`（含作为声明解释器证据的临时 venv `t01/scratch/venv-numpy` 与固定组门收据 `t04/gate-run-01a`）、主检出上的计划簿记（未提交）。解锁后由 work 在持久环境下复跑 `case-av-language-parity` 与 `case-a2py-language-parity`，以 `pass` 交 review。路径对账：本目标在登记范围外无输出；`fixed-tests.py` 自身在系统临时目录生成的 `fixed-admission-*` 诊断目录（本目标两次：03:09、03:38，由门禁结果的 `output_admission.diagnostics` 引用）属驱动所有，留 merge 收尾对账。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：§10 仍无答复，环境仍未配置（用户级/系统级 `A2PY_PYTHON` 为空、系统 site 无 NumPy、建议 venv 目录不存在）。本轮未做阶段推进、未改机器环境、未新增提交；仅做只读重读与 work 内自审：①六个检查的 stderr 均有断言（显式空围栏按作者层语义为零字节预期，`numpy-shape-error-caught/av` 由 `ac-av/language-source-run-v1` 的省略即零字节规则覆盖），无需改动；②README 所写命令形式 `cargo nextest run --manifest-path experimental/ace/Cargo.toml -p ace-a2x-python --test py_libraries_1126` 实测 5/5 PASS（2.5 s）；③仓库内没有登记 Python 环境变量的自有脚本，`A2PY_PYTHON` 惯例为用户 shell 环境，不存在被遗漏的合规做法。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读，无阶段工作、无提交、未改机器环境。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 10 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 11 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。本轮只做只读重读。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 12 | from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。维持 blocked：仅自动续跑提醒，非用户裁定；§10 无答复，环境仍未配置，专属树 `6ca8c483e` 干净。到达默认轮数上限（12），按规则停止并报告最后已核实阶段：work（T-01..04 已交付并提交，`outcome: blocked` 待 Q-01）；review 与 merge 未开始。同一目标可在用户回复 Q-01 选项后恢复。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 13（超出默认上限 12，由宿主自动续跑触发）| from: work | to: work | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | next: user`。仍无用户裁定：§10 无答复，环境未配置，专属树 `6ca8c483e` 干净。未自行延长轮数上限做阶段工作，仅只读重读；后续同类自动轮次不再逐轮追加收据，状态以本行为准，直至用户回复 Q-01。

2026-11-22，`stage: new | PLAN-1126 r1 | status: drafting | progress: 0/6`。下一阶段入口为实施授权后的 T-01，所依赖的双端异常接口由 1110 正式交付。本轮没有产品运行证据。

2026-11-23，`stage: work | plan_id: PLAN-1126 | plan_revision: r1 | status: executing | outcome: waiting（宿主通知会话目标已暂停）| worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 6ca8c483e | blocker: 无——Q-01 已由用户裁定为 (a) 并执行，持久环境就位（§10）| next: work（目标恢复后在持久环境下复跑 `case-av-language-parity` 与 `case-a2py-language-parity`，记录后以 `pass` 交 review）`。本轮只执行了裁定 (a) 的环境步骤和一次环境冒烟（6/6 PASS），未做阶段推进、未新增提交。恢复时待办：①复跑前重新 `--prepare-only`（现有清单已过期）；②门并集的消费者核对——`language.suite.ad` 还被 `case-aac-gen1-language-parity`、`case-a2r-language-parity` 两个固定组引用，`crates/auto-lang-core/src/legacy_doc_for_trans.rs` 也读取该套件，review 时确认它们对本计划新增用例（未声明对应 probe）的处理；③规范草稿 `python-ecosystem-integration.md` §7「运行环境」补一句冻结环境的持久安排（专用 venv 经用户级 `A2PY_PYTHON` 声明），归档后把 `python-interop.md` 附录的「来源」改为归档链接。

**2026-11-23 恢复实施验收**。用户以 `/goal 继续完成计划1126` 恢复交付，沿用 §4 和 Q-01 的授权。暂停期间已准备工具并提交持久解释器规范补充 `f808fe3ea`。本轮以注册表声明的专用解释器实际执行，确认 CPython 3.14.2、NumPy 2.5.1、隔离模式 `ENABLE_USER_SITE=False`。提交 `f808fe3eaafcc63eb0e0687fc29f54d3ef1492ad`，clean，依赖未变。

| 固定组 | 结果 | 子运行 | 清理 |
|---|---|---:|---|
| `case-av-language-parity` | 532 PASS／2 FAIL／193 NOT_RUN／161 SKIP，534 执行。两条 1093 C provider 原有失败与开工基线同 ID 同诊断 `foreign.prepared`。 | 46.892 s，退出 1 | runtime 已回收，未分列时间 |
| `case-a2py-language-parity` | 固定门 PASS，7/7 PASS／881 NOT_RUN。普通父 CLI 汇总因未选行退出 1，固定门核对 7 条终态、无加载错误或覆盖缺口后判 PASS。 | 9.199 s | runtime 已回收，未分列时间 |

命令 `cargo t case-av-language-parity case-a2py-language-parity --run-prepared <prepared.json> --out <scope>/final/work-gate-02/run`，合计 63.351 s，退出 1（两条既有红）；复用本提交已完成的 debug 准备 5.166 s，发现耗时未单列。等待 PLAN-1121 的 `ace-source ace-av` 和 PLAN-117 的桌面示例编译，等待时间未精确计量，与门禁耗时分开。清单 source-state `36463dc67389823a…`；结果 SHA256 `02b948816a34e8aa70d519a7b9ef5d03016f25d18c59c7226c21542e573b4635`。

| 新 Case | AV / A2PY 实跑耗时（debug） | 同源身份前缀与终态 |
|---|---:|---|
| `py.libraries.1126.numpy-json` | 0.817 / 1.134 s | `196bf283`，双端 PASS，数组与 JSON 输出匹配 |
| `py.libraries.1126.numpy-shape-error-caught` | 0.724 / 1.003 s | `36e11fbe`，双端 PASS，AV 实测捕获计数 1 |
| `py.libraries.1126.json-decode-error-uncaught` | 0.707 / 0.975 s | `d1ad6a97`，双端 PASS，原类、位置与未捕获结果匹配 |

原生参考的五个测试与负控证据复用 §9 T-04（本轮仅补跑双端持久环境，参考源码和运行条件未变）。本轮无其他新增用例。

`stage: work | plan_id: PLAN-1126 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1126/auto-lang | branch: plan-1126-dev | base_commit: d4bc60a89 | code_commit: f808fe3ea | dependency_revisions: 1094@r5 / 1107@r2 / 1110@r2 / auto-down@456c86a | task_ids: T-01..04 | evidence: 恢复实施验收与 T-04 | blockers: 无 | next_responsible_role: review`。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 续跑 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: f808fe3ea | next: review`。

**首轮独立复审（2026-11-23）**。在独立续跑会话从提交 `f808fe3eaafcc63eb0e0687fc29f54d3ef1492ad`、原始 run report 和源码重建判定。AC-01/T-02 与 AC-02/T-03：三方正例核值通过，参考脚本直接读真实库，并断言数组/步长切片与解码树；AC-03/T-02、T-03：两原类、原位置、捕获计数与退出语义通过。AC-04/T-04：同源指纹、运行身份和原始负控核对通过；F1126-01 需要收齐运行说明。

| 本轮检查 | 结果 | 耗时 |
|---|---|---:|
| A2R、Gen1 逐个 `auto-case run --case <1126-id> --target <a2r/ gen1>` | 六组交叉选择均退出 2 并给出 `matches no declared check`，0 条选中。第一次探测按存在 report 读取，实际为匹配检查前拒绝，修正读取命令后六组完成；其他视图整套未重复执行。 | 15.134 s；准备复用，发现/清理未单列 |
| OS CPython 执行三个手写参考 | NumPy＋json 11 行、捕获 3 行、未捕获退出 1 且原始 traceback 类正确 | 0.122 / 0.114 / 0.059 s，含启动与输出采集 |
| `git diff --check`／单文件 `rustfmt --check` | 都退出 0，工作树干净 | 未单列 |

用例时限证据复用紧邻的「恢复实施验收」六条（0.707–1.134 s）及 T-04 五条 nextest（0.029–0.271 s），无新增测试。消费者证据 `final/review/consumer-summary.json` SHA256 `03d6c94a0b81dad392ca0968ce1b3a1d63083e84055c9d98e6a24c3baede4076`。旧 VM reader 仅按 `legacy-map.tsv` 中迁移 ID 投影旧 VM 检查，1126 无映射。固定组对 AV/A2PY 传入 source-bound worker，缺文件时显式报错；现存裸 AV CLI 的历史回退不在固定组路径，单例文档需写明预备工具环境。

- **F1126-01（T-04 / AC-04，运行说明偏差）**。`crates/auto-lang/test/cases/ace-m2/python-1126/README.md` 的 nextest 命令缺 `run`（T-04 已验证完整命令）；单 Case 示例未提醒使用准备清单环境；AV 解释器缺失时，`ac_av.rs` 的 `py-unavailable` 路径明确记录 NOT_RUN、固定组不通过，README 将解释器缺失与缺 NumPy 的实际 FAIL 合并表述。修复范围为 README 命令和三种准入终态说明。关闭条件为对照现有准备入口、`worker_binary`、`py-unavailable` 及 Q-01 缺包反控重新核文档，无需改变运行器和已通过的用例。

`stage: review | review_scope: final | reviewer_identity: 独立续跑会话 | plan_id: PLAN-1126 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1126/auto-lang | branch: plan-1126-dev | reviewed_commit: f808fe3ea | base_commit: d4bc60a89 | findings: F1126-01 | next: work`。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 续跑 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: f808fe3ea | next: work`。

**F1126-01 文档修复**。提交 `ec4a9ab05` 补齐 README 的 nextest `run`、固定组的准备/消费命令、单例预备环境与缺解释器/缺包的实际终态。核准备清单：AV 提供 `AC_AV_CASE_WORKER` / `ACE_AV_PROVIDER_ROOT`，A2PY 提供 `AC_A2X_HIR_WORKER`；复用已实跑的 nextest 完整命令与 C7/C8 缺包负控，和 AV `py-unavailable` 的原有代码路径对照。仅 README 文档变更，定向内容、diff 与 staging 检查通过；无新增测试或产品测试重跑。

专属树同步至 `master@6d3bd996740f600b91ece1ef6b77058025566515`（PLAN-1120）。`git range-diff` 四条补丁全部等价，无冲突：`9039eb7c8→e8992a71b`，`6ca8c483e→941aed7f9`，`f808fe3ea→986eedff8`，`ec4a9ab05→8d2e78b87`。新基面改了共享 resolve 与既有迁移映射，提交后冻结门仍须在同步后实跑。

`stage: work | plan_id: PLAN-1126 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1126/auto-lang | branch: plan-1126-dev | base_commit: 6d3bd9967 | code_commit: 8d2e78b87 | dependency_revisions: 1094@r5 / 1107@r2 / 1110@r2 / auto-down@456c86a | task_ids: T-04 / F1126-01 | evidence: README 命令及准入说明通过内容核对，冻结并集仍待 merge | blockers: 无 | next_responsible_role: review`。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 续跑 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 8d2e78b87 | next: review`。

**最终复审（2026-11-23，r1）**。完整冻结提交 `8d2e78b87d213b9f376ee596f7185afff1800b70`，基面 `6d3bd996740f600b91ece1ef6b77058025566515`，auto-down `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`，专属树干净。AC-01..04 证据沿用首轮复审的原始结果、正负控和时限；F1126-01 关闭，核完整命令、准备门环境变量与三个准入分支。对原提交逐路径 `git diff --exit-code` 确认语料、参考、测试和 SD-01..03 字节等价；`git diff --check` 通过，定向复审耗时 0.334 s；本次无新测试或产品测试执行。

- AC-01 / T-02：独立值比较含步长切片和均值容差，两条运行记录确认为真实库调用；错误切片 oracle 使 AV 与 A2PY 均红。
- AC-02 / T-03：原生 JSON 断言树与类型，冻结 oracle 核两次精确 Unicode 编码；错字段叶子和错键反控均成立。
- AC-03 / T-02、T-03：原类、原源范围、AV 捕获 1 次和 A2PY 退出 1 均有实测断言；改错范围或错误类失败。
- AC-04 / T-04：三对相同 `identity.source`，A2PY 核 `execution_claimed`、一次 Python spawn、worker 源哈希和进程出口，AV `py` provider 按真实完成状态裁决；旧 VM、A2R、Gen1 选择路径核对完成。缺包与未运行反控保留真实结果。日常入口通过预先装配的 debug worker；README 的单例复现现要求同一预备环境。
- SD-01/02/03：审核 Git blob 分别为 `258d2970a4a201328971cbda5d222c9c2fcbd12e`、`082669923b049628d2b5ce40d2c81b3e15ec05fe`、`c61f68d638a71d7887b78f3a83bf54a34e7c6b94`。描述既有对象路径的确认范围及隔离解释器要求，未扩操作或支持矩阵。遗漏、延期与 workaround 扫描未见新债；1093 的两条原有 C provider 失败已在 1110 归档及批次收据具名。

合入门：`cargo t case-av-language-parity case-a2py-language-parity ace-a2x`，已静态确认 3 个调用（两组 TestCase 与包含新增参考测试的完整 ACE A2X 职责组）。上游 1120 改了共享名字解析，本门在同步后最终交付提交运行。其他视图经选择检查确认不消费这三条用例；语言实现及适配器未改，其独立职责组不扩大。Linux/WSL 腿依平台裁定排除。

`stage: review | review_scope: final | reviewer_identity: 独立续跑会话 | plan_id: PLAN-1126 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1126/auto-lang | branch: plan-1126-dev | reviewed_commit: 8d2e78b87d213b9f376ee596f7185afff1800b70 | base_commit: 6d3bd9967 | dependency_revisions: 1094@r5 / 1107@r2 / 1110@r2 / auto-down@456c86a | acceptance_results: AC-01..05 pass | findings: F1126-01 closed | pending_closeout_gates: 冻结三组、规范核验、归档清理 | next: merge`。

`stage: goal | plan_id: PLAN-1126 | plan_revision: r1 | round: 续跑 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1126/auto-lang | commit: 8d2e78b87 | next: merge`。

**合入准备（2026-11-23）**。复审后仅补 canonical 出处、AV 模块导航及实测 stdout 编码说明，提交 `cebfe7872`，代码和测试输入与复审提交相同。PLAN-1121 在准备期合入并改动共享前门，同步 `master@c0441ea77ce4d69a94efe9c1d4fb05515f068a80`；`git range-diff` 证明本计划五条补丁等价（`e8992a71b→e4b038db7`、`941aed7f9→eceb37628`、`986eedff8→da7d086d0`、`8d2e78b87→01c15201a`、`cebfe7872→ec84ed4da`），审查范围逐路径无字节变化。基面共享适配、移位及 map/set 解析变化由同步后的冻结门覆盖。旧基面只准备未执行（case 38.592 s，ace-a2x 17.641 s），其清单不进入最终裁决。

`stage: merge | checkpoint: prepared | plan_id: PLAN-1126 | plan_revision: r1 | reviewed_commit: 01c15201a58584bf3ce6b80e4c2d2adf8f0fe464（等价映射） | delivery_commit: ec84ed4da20bf33ab158c8775fc347f445279cc3 | base: c0441ea77 | frozen_gates: pending`。

**冻结验证（2026-11-23）**。`ec84ed4da20bf33ab158c8775fc347f445279cc3`，CPython 3.14.2、NumPy 2.5.1，A2PY 使用 Q-01 持久 venv。工具和 nextest 为 debug/dev/test。因固定驱动的 `--debug-tools` 只接受 nextest，两组 Case 与 `ace-a2x` 分别做 `--prepare-only` 与 `--run-prepared`。共同 source-state `5bd88f840c92a18637be855e0696a3a166a1a1905d7e991ef9da2633217f3f7a`；固定配置 SHA256 `2d3cd7f908e494a618b6d97c2550f65dffa9326c6843f50b3b5b0af99be5167e`。每次统一开始前查 cargo、nextest、编译器和 worker 进程。本次启动时等待 PLAN-117 的测试编译 29.890 s；此前 PLAN-1121 和 PLAN-117 的构建等候未单列。

| 固定范围 | 终态及计数 | 准备／发现／执行／清理 |
|---|---|---|
| `case-av-language-parity` | 532 PASS／2 FAIL／193 NOT_RUN／161 SKIP，执行 534。仅 `av.parity.1093.explicit-c-library` 与 `av.parity.1093.c-header-spelling-rejected`，同 T-01 与 1110 既有 `foreign.prepared`；无新增红。 | 双 Case 共用准备 31.478 s，AV 子进程 45.636 s，发现与清理时间未单列，runtime 已回收 |
| `case-a2py-language-parity` | 固定门 7/7 PASS／881 NOT_RUN；部分选择父汇总退出 1，固定门按 7 条实际结果通过。 | A2PY 子进程 8.949 s，发现与清理时间未单列，runtime 已回收 |
| `ace-a2x` | 354/354 PASS，4 条原 ignored（941 两条 inventory，1116 两条 probe），子运行退出 0；包含 1107 PyTorch/协议与 1110 异常回归。 | 准备 17.124 s／发现 2.802 s／运行 65.696 s／清理未单列 |

两方案例一次 `cargo t case-av-language-parity case-a2py-language-parity --run-prepared … --out …` 合计 61.123 s，退出 1（保留两条 1093 原红），结果 SHA256 `9790da3594e5fb0313d76073a7e440507d6b478414baf18f04c352a7e6ebfad6`。`cargo t ace-a2x --run-prepared … --out …` 合计 69.348 s，退出 0，结果 SHA256 `cd1c439a90112408cf0ca92dd29f4c988fe12e557144d9356a31ea9afd7777e4`。平台外 Linux/WSL 腿依裁定排除。

| 新用例行为 | 冻结实测，debug/test，全部 PASS |
|---|---|
| NumPy＋json 正例，两端匹配参考小值与精确 Unicode 文本 | AV 0.836 s／A2PY 1.039 s；同源 `196bf2835a25` |
| NumPy 原类与位置、捕获一次并继续 | AV 0.741 s／A2PY 0.995 s；同源 `36e11fbe992e` |
| json 原类与位置、未捕获终态 | AV 0.682 s／A2PY 0.909 s；同源 `d1ad6a97a5d0` |
| `p1126_v01_v02_native_reference_matches_the_frozen_oracle`，独立正例参考 | 0.224005 s |
| `p1126_v03_native_reference_confirms_the_numpy_valueerror`，捕获原类 | 0.224032 s |
| `p1126_v03_native_reference_confirms_the_json_decode_error`，未捕获原类 | 0.143226 s |
| `p1126_v04_sources_are_pinned_and_spans_match_the_authored_bytes`，源与范围 | 0.011655 s |
| `p1126_v04_the_oracle_rejects_wrong_values`，错值、错类型、错行数 | 0.066877 s |

**landed**。`master` 的 `c0441ea77→9d3cd9e63` 仅变更 PLAN-1121 归档簿记。再次 rebase 的五条 `range-diff` 均等价（`e4b038db7→7f35400b1`、`eceb37628→b01815bc0`、`da7d086d0→9e39d21d6`、`01c15201a→33011eaed`、`ec84ed4da→6db6d7bf7`），测试代码、工具与输入未变。专属树 guard clean 后，`git merge --ff-only plan-1126-dev` 从 `9d3cd9e63` 快进至 `6db6d7bf7a552175a122fed78bd5a613eede2c27`。在主检出用已准备 `auto-case inventory` 核 3 源 6 条 RUN、每对指纹相同，退出 0、1.625 s；主检出生产/测试源码路径无 WIP，其他会话已有文档簿记及独立目录保留。

**canonical_verified**（`6db6d7bf7`，Git blob）：

- `docs/specs/auto-a2x/python-ecosystem-integration.md` — `5bde8e23866bd8e021d6ca87c0d129c7a34d1bd0`
- `docs/specs/auto-av/python-interop.md` — `bfa7dd0d4722a4263bbe4fe42ef9b1ebc1902eb5`
- `docs/specs/auto-test-case/feature-organized-corpus.md` — `c61f68d638a71d7887b78f3a83bf54a34e7c6b94`
- `docs/specs/auto-av/overview.md` — `d26a208b78ea6468e09bb83d946f7f5fba8f56da`（导航）

索引由应用派生。暂存路径核对：专属树内 target 为可删除缓存；统一材料范围经 guard clean，7346 文件、370133870 B；旧外置三个 `fixed-admission-*` 的 meter 均指向该专属树，共 2399 B；两个历史批次和本轮的临时 Case runtime 均已回收。Q-01 专用 venv 是持久机器环境，登记的路径和 wheel 哈希见 §10。

**archived**。completion_kind: delivered。主线 archive checkpoint：归档状态与 5/6 任务一致，T-06 的路径与登记清理待本阶段确认；§9 保留冻结验证、快进和规范来源。

## 10. 待澄清事项

T-01 核准实际 Python/NumPy 组合、现有字符串和对象索引的覆盖、精确错误分类及两端命令。候选源码为验收合同。源语法差额、共享语义或宿主边界变化按实际证据写回。

- **Q-01（待用户裁定，2026-11-23）A2PY 隔离腿的 NumPy 来源。** `run-py` 按 1042 冻结合同以 `python -I -B` 隔离运行，看不到用户 site；本机 NumPy 2.5.1 只在用户 site、系统 site 为空。默认解释器下 `case-a2py-language-parity` 的 NumPy 行必然失败（`ModuleNotFoundError` / `python.1`），V-01 与 NumPy 异常用例无法在 A2PY 真实执行。语言与合同均无需也不应为此改动。候选：
  - (a) 专用 venv（基座 `C:\Python314`，装 `numpy==2.5.1`）+ 用户级环境变量 `A2PY_PYTHON` 指向其 `python.exe`。走既有声明解释器入口，与 `PYO3_PYTHON` 同一惯例；不改合同、不改系统 site。需要指定 venv 的持久位置并登记。
  - (b) 往系统 site 装 NumPy（`pip install --ignore-installed numpy==2.5.1`）。与「系统 site 为空」的现状相悖，且与用户 site 副本并存（实测非隔离模式下用户 site 在 `sys.path` 中先于系统 site，用户副本优先；2026-11-23 更正，原文误写为「系统副本优先」）。
  - (c) 放宽 1042 隔离合同（改 `a2x-worker` 启动标志）。属于另一计划的冻结合同变更，不在本计划授权内。
  - (d) 调整验收范围（A2PY 的 NumPy 腿延期）。属于验收变更。
  - (e) 仓库内解决、不动机器环境：在测试工具链的准备阶段（`scripts/test-governance/toolchains.json` + `prepare_tools.py`）新增一种「Python 环境」工具，由 `--prepare-only` 在树内 target 目录建 venv 并装固定版本 NumPy，再经 `A2PY_PYTHON` 接线给运行。属于新增测试基础设施（准备阶段引入联网 pip），超出本计划声明的改动面，需你确认并入本计划（修订 r2）或另立计划。

  2026-11-23 已在 work 会话内以选项形式提请，其后多个自动轮次未获回答，期间未对机器环境做任何改动（未装包、未设环境变量）；随后用户裁定 (a)（见本节末）。各选项的确切解锁操作：

  - (a) `C:\Python314\python.exe -m venv D:\autostack\.tools\a2py-py314-numpy` → `D:\autostack\.tools\a2py-py314-numpy\Scripts\python.exe -m pip install numpy==2.5.1` → `setx A2PY_PYTHON D:\autostack\.tools\a2py-py314-numpy\Scripts\python.exe`，并在 `.retention/policy.json` 登记该目录。若选 (a) 但希望换位置，直接给出路径。回复 (a) 即视为授权由本会话执行以上三步。
  - (b) `C:\Python314\python.exe -m pip install --ignore-installed numpy==2.5.1`。回复 (b) 即视为授权由本会话执行。
  - 选 (a) 或 (b) 后：本会话在持久环境下复跑 `case-av-language-parity` 与 `case-a2py-language-parity`，记录后以 `pass` 交 review。

  **2026-11-23 用户裁定：(a)。** 已由本会话按上述三步执行并核实（脚本、原始输出在材料目录 `env-a/`）：

  - venv `D:\autostack\.tools\a2py-py314-numpy`：`C:\Python314\python.exe -m venv`（基座 3.14.2，`include-system-site-packages = false`，目录 68 MB）；`pip install --only-binary :all: numpy==2.5.1` 装入 venv 自身的 site-packages（`numpy-2.5.1-cp314-cp314-win_amd64.whl`，12.6 MB，取自 pip 缓存）。
  - 用户级环境变量 `A2PY_PYTHON=D:\autostack\.tools\a2py-py314-numpy\Scripts\python.exe`：设置前用户级、系统级均为空；`setx` 后注册表回读一致，路径存在。
  - 隔离模式核实（`-I -B`，与 `run-py` 的启动方式相同，中性工作目录）：`sys.flags.isolated=1`、`ENABLE_USER_SITE=False`，NumPy 2.5.1 来自 venv；`_core/_multiarray_umath.cp314-win_amd64.pyd` sha256 `ab5cea76a0fad4fd1bc58667aefba1382a7ce8c147b51cf3bff53eec4efb4ab2`，与 AV 所用的用户 site 副本逐字节相同；venv 的 `python.exe` sha256 `e725f520cfbb2e4844fe0a9fc8949236f20b0d67e470d1232618b99504a0691a`。
  - 环境冒烟（`env-a/env_smoke.py`：真实 AV worker 与 A2PY worker，工具环境取自 `b3ff3b45d928e7d2/prepared.json`）：三条用例的 av/a2py 六行全部 PASS、零 mismatch，同源身份与 §9 交付记录一致（`196bf283…`/`36e11fbe…`/`d1ad6a97…`），每例 3.2–3.5 s。它只证明环境就位，不替代两组固定组门的复跑。
  - 生效范围：注册表值只对之后新启动的进程生效；本会话既有进程的环境不含该变量，复跑时在命令内显式设置 `A2PY_PYTHON`。
  - `A2PY_PYTHON` 的读取方（代码核对）：`a2x-worker` 的 `resolve_python`（`run-py`，以及 `ace-a2x-worker`、`ace-av` 中把它当标准库工具的测试）与 `ace-a2x-python` 的 `py_runtime_and_provider.rs`。`m2-build` 的 `run_sdk`（PATH 上的 `python`）、1107 torch 原生运行与 1126 参考脚本（`AUTO_PYTHON_HOME` → `C:\Python314\python.exe`）都不读它，torch 腿不受影响。T-04 在临时 venv（同一轮子）解释器下已实跑的组：`ace-a2x`（354/354）、`case-a2py-language-parity`（7/7）；`ace-av` 组中 11 个调用 `resolve_python` 的测试目标未在该类解释器下实跑，仅静态核对其源码不引用 `sys.executable`、`sys.prefix` 或解释器路径。
  - 登记：`.retention/policy.json` 的 `holds` 增加该 venv（机器环境，非计划材料，不随 merge 收尾删除；该文件被 `.gitignore` 的 `*.json` 忽略，属本机索引）；材料目录 `env-a/` 登记在 plan-1126 scope 下。
  - 更正：(b) 原文「非隔离运行时系统副本优先」有误，已就地改正（实测基础解释器非隔离模式下 `sys.path` 里用户 site 下标 6、系统 site 下标 10，用户副本优先）。早期收据与聊天报告把「自行配置」记作 (c)，与本列表的 (c)（放宽 1042 合同）无关，以本列表字母为准。
