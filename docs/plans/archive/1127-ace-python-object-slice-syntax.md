---
plan_id: PLAN-1127
status: archived
completion_kind: delivered
feature_name: ACE Python 对象的冒号切片语法
author: [codex]
created_at: 2026-11-23
updated_at: 2026-11-23
plan_revision: 1
depends_on: [PLAN-1126, PLAN-1110, PLAN-1107]
supersedes_spec_components:
  - docs/specs/auto-syntax/grammar-contract.md
  - docs/specs/auto-av/python-interop.md
  - docs/specs/auto-a2x/python-ecosystem-integration.md
  - docs/specs/auto-test-case/feature-organized-corpus.md
new_spec_components: []
touched_goals: []
affects: [auto-syntax, auto-av, auto-a2x, auto-test-case]
current_step: 5
total_steps: 6
---

# [PLAN-1127] ACE Python 对象的冒号切片语法

## 0. 变更摘要

2026-11-23，承接 PLAN-1126，在同一 ACE 受检源面支持 Python 对象索引 `row[0:3:2]`。用一份 Auto 正例同时调用 NumPy 和 CPU PyTorch，另以同源异常 Case 核对捕获、未捕获和原始源码范围。交付范围由本文定义，目标设计见 [Design 58](../../design/58-python-standard-syntax-interop.md)。

## 1. 目标

- ACE 普通 Auto 源支持经受检绑定确认为 Python 对象的单个冒号切片键，支持两个或三个槽位及省略槽位。结果仍可链式索引、调用和显式提取标量。
- 保留 Auto 原有 `s[start..end]` 和 `base[key]`。保留显式 `row[slice(...)]` 的语义及已有 Python 对象协议。
- 覆盖双端真实运行、独立原生 Python 对照、源序与单次求值、Python 原异常类及精确位置。切片赋值、逗号分隔的多维键、`.../None` 特殊键的新语法、Auto 集合冒号切片、旧后端扩展和通用容器过桥另属各自范围。允许通过普通索引再单轴切片，例如 `matrix[0][::2]`。

## 2. 架构方案

`base[start:stop:step]` → ACE 共享 Grammar/CST 后缀与可选槽位 → 共享受检 Python 调用事实 → Python `slice` 键 → 既有 `GetItem`。AV 由 967 provider/CPython 消费，A2PY 由生成的 Python 程序消费。两端复用既有异常观测和源身份绑定。设计优先在共享受检模型中归一化，避免将语法字节串交由各后端重新解释。

## 3. 技术栈

ACE 声明式 `auto-source.at`、CST 投影与 bind/resolve、Checked HIR / `PyOpFact`、AV 既有 `py_getitem`/PySession、A2PY 既有 `a2x_py_getitem` 和 1110 运行观测、AutoDown TestCase。以 1107/1126 已验 Windows x64、CPython 3.14.2、NumPy 2.5.1、torch 2.14.0+cpu 为环境基线，实施第一步核实际解释器、包身份及版本。冻结同组版本供 AV、A2PY 和独立原生参考使用。

## 4. 需求分析与背景调查

本轮授权为 auto-lang 仓内的新计划、相关设计和路线图。2026-11-23 用户以 `/goal 完成计划1127` 授权执行、复审、合同内修复和合入归档。实施基面 `dca52e196a64f123ca057e1442c71c8f3e95ab30`，专属树 `D:/autostack/.wt/lang-1127/auto-lang`（`plan-1127-dev`）；只读 auto-down 同组检出 `456c86ac886f9781e56f6153ce9aa41cd59e5c8e`。静态核对基线为 `9c0f5e0fc2d6`，从 [Specs 总览](../../specs/overview.md) 进入相关模块。

| 来源 | 已核对事实与本计划的交接 |
|---|---|
| [语法契约](../../specs/auto-syntax/grammar-contract.md)、[实际 Schema](../../../experimental/ace/grammars/auto-source.at)、[CST 投影](../../../experimental/ace/crates/core/src/source/cst/project.rs)、[PLAN-1086](1086-av-parity-borrows-moves-and-storage.md) | `postfix-step` 已有 `[expr]`，`auto/auto-packages` 另有 `[start..end]`。后者由 `slice_range` 投影为 `recv.slice(start[, end])`，省略起点时入参为 0。现有后缀没有冒号切片臂。`:` 已用于对象键、命名实参等语境，新臂需在方括号当前层识别分隔符。 |
| [AV 源码入口](../../../experimental/ace/crates/av/src/foreign_source.rs)、[A2PY worker](../../../experimental/ace/crates/a2x-worker/src/bin/a2x-worker.rs)、[共享前门入口](../../../experimental/ace/crates/core/src/source/adapter.rs) | 普通 Python 路径均有 `compile_language_source_with_foreign`，采用 `SourceProfile::Auto`。T-01 核查包模式及共享 Schema 消费者的准入，沿原 profile 合同开放，给不开放的消费者保留具名拒绝。 |
| [PLAN-1126](1126-ace-python-numpy-json-parity.md)、[同源语料](../../../crates/auto-lang/test/cases/language/script/python-libraries.case.ad)、[共享 resolve](../../../experimental/ace/crates/core/src/source/resolve.rs)、[AV provider](../../../experimental/ace/crates/foreign-contract/py/py_bridge_provider.rs)、[A2PY lowering](../../../experimental/ace/crates/a2x-python/src/lower.rs) | `row[slice(0, 3, 2)]` 已双端实跑。`GetItem` 已接受 Python 对象键，AV `obj.get_item` 和 A2PY `recv[idx]` 调用包原生协议。AV provider 已可将帧 `F0Val::None` 转为 Python `None`，省略槽位如何端到端送达仍需确认。 |
| [PLAN-1107](1107-ace-python-av-a2py-integration.md)、[PLAN-1110](1110-ace-a2py-python-exception-mapping.md)、[Python 生态规范](../../specs/auto-a2x/python-ecosystem-integration.md)、[AV 互操作规范](../../specs/auto-av/python-interop.md) | 1107 提供双端 PyTorch 和工程基线，1110 提供 A2PY 原类及源范围观测，1126 已用同一份 Auto 源核两端真实异常。三项均已归档；本计划消费其正式成果。 |
| [Python 切片语言规则](https://docs.python.org/3.14/reference/expressions.html#slicings)、[NumPy 索引文档](https://numpy.org/doc/stable/user/basics.indexing.html#slicing-and-striding)、[PyTorch 索引实现](https://github.com/pytorch/pytorch/blob/main/aten/src/ATen/TensorIndexing.h#L2348) | 原生 Python 切片将省略项传为 `None`。NumPy 支持步长小于零，PyTorch 上游实现要求大于零。实际启用的 PyTorch 构建的异常类在 T-01 用独立原生参考冻结。 |

### T-01 冻结结果

- 单模块普通 Auto 的 AV 与 A2PY 都经 `compile_language_source_with_foreign` 进入共享 Schema、投影及绑定。语法在 Auto / AutoPackages 保持同一后缀形状，包模式已有绑定门拒绝 `use.py`，窄 profile 不开放该后缀。
- 选定 `GetItem` 受检事实记录接收者及 start/stop/step 三槽。省略槽保留为 Python None。AV 在原 `p967.frame/1` 的独立零位置实参形状中传三槽，旧 provider 缺位置键时会拒绝；新 provider 以内建 PySlice 类型构造临时键，键在 getitem 调用域释放，不持有 PySession token。原位置实参形状保持。
- Windows x64，CPython 3.14.2、NumPy 2.5.1、PyTorch 2.14.0+cpu。AV 使用 `C:/Python314/python.exe` 的 CPython；A2PY 和原生参考用 PLAN-1127 暂存区内的隔离 venv，它显式读取同一组已安装包，未改全局环境。执行 `py314/Scripts/python.exe -I -B -c <原生冒号用例>` 退出 0、2.015 s（含 venv 准备）；NumPy 倒序为 `[5,4,3,2,1,0]`，两库零步长均 `ValueError: slice step cannot be zero`，PyTorch 负步长为 `ValueError: step must be greater than zero`。记录 `D:/autostack/plan-artifacts/plan-1127/t01-runtime.txt` SHA256 `eec45079ac89ee405af981517170c77c07104051d1c5fa7f20bb0a25e0f095a4`。

## 5. 详细设计

### 5.1 语法与共享语义

在同一个索引后缀里区分普通 `[expr]`、Auto `[start..end]`、Python `[start:stop]` 及 `[start:stop:step]`。候选产生式为 `"[" [expr] ":" [expr] [":" [expr]] "]"`。槽位采用普通表达式，以方括号当前层的冒号分隔。CST 保留标点、原槽位及范围，嵌套调用的命名实参或对象字面量中的冒号仍由内部表达式规则处理。

- `row[:]`、`row[:3]`、`row[2:]`、`row[1:4:]`、`row[::2]` 和 `row[::-1]` 均为候选合法形态。省略的 `start/stop/step` 一律保留为 Python `None`，终点排他性和默认端点由接收对象的 Python 索引协议决定。首批保证已有 Python 实参合同可传递的整数界限、整数运行时表达式，以及通过该合同的 Python 对象值。是否接受某个已传达的值适用 Python 对象协议，跨边界的值类型仍按当前准入合同。
- bind/resolve 只为已有 Python 对象来源的接收者产生受检语义。Auto 列表/文本或无已校验 Python 来源的值中的冒号切片出绑定诊断，普通索引、已支持的点点切片及原赋值入口保原义。语法多出冒号、逗号键属于当前语法拒绝，冒号切片写入应在源入口具名拒绝。
- 求值依次为接收者、存在的 start、stop、step，各一次。统一产生解释器内建 `slice` 键后走 `GetItem`。`slice` 不走用户名称查找，局部 `slice` 同名绑定不得改变语义。T-01 确认现有内建解析、可选值、操作事实/定位和临时键寿命能否复用，若需增补，以最小中立事实或受检运行时能力记清两端调用、兼容和失效。包名特判与新增容器转换不属本计划。
- AV 临时键、索引结果和外部对象沿 PySession 的资源与归属合同。A2PY 使用解释器原生引用。切片结果保留 Python 对象语义，可经原索引/调用继续使用，视图和拷贝由所调库决定。
- 零步长句式通过语法和绑定，求值后由索引协议报告原类。负步长如实保留包行为。真实索引失败的原 Auto 范围为从该接收表达式起点至本次右方括号的半开 UTF-8 字节范围。接收者或槽位求值先失败时保留该内层失败位置。捕获与未捕获沿 1094/1110 现有观测，A2PY 未捕获须同时观测实际非零退出。

### 5.2 候选 Auto 正例

同一个作者源引入两个包。示例采用现有 `use.py module: member` 拼写和不同构造器名称。完整源在 T-04 落地。

~~~auto
use.py builtins: range
use.py numpy: array
use.py torch: arange

var n = array(range(6))
var t = arange(6)
var n_stride = n[0:3:2]
var t_stride = t[0:3:2]
var head = n[:3]
var tail = t[2:]
var reversed = n[::-1]
~~~

### 规范增量

| ID | 动作 | 目标 | 现状与拟沉淀规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-syntax/grammar-contract.md` | 已有单索引和点点范围；追加冒号后缀、profile、CST 槽位及拒绝边界 | 固定语法兼容性 | AC-01 |
| SD-02 | modify | `docs/specs/auto-av/python-interop.md` | 从显式键扩为受检冒号键，记录规则、AV 资源与错误位置 | 固定 AV 支持面 | AC-02/03/04 |
| SD-03 | modify | `docs/specs/auto-a2x/python-ecosystem-integration.md` | 追加 A2PY 同源、运行身份、观测和 NumPy/torch 实测矩阵 | 固定两端一致的源合同 | AC-02/03/04 |
| SD-04 | modify | `docs/specs/auto-test-case/feature-organized-corpus.md` | 登记新增语料的唯一作者位置和双端 checks | 固定验收入口 | AC-04 |

## 6. 测试设计

拟新增 `crates/auto-lang/test/cases/language/script/python-slicing.case.ad`，每 Case 由一份 Auto 作者源构成，两个执行 check 使用同一原始字节、逻辑路径和源指纹。复用 [1126 的手写参考结构](../../../crates/auto-lang/test/cases/ace-m2/python-1126)，在新计划编号下编写独立原生 Python 参考，以原生冒号及库 API 直接表达。分别对照每个目标的真实输出或结构化异常，请求材料记录解释器、包版本、退出码和命令；固定期望独立核原生参考，避免三方同错。环境缺解释器、包或授权按现有准入失败或未执行记账，缺一条必验运行不关验收。

| 验证 | 输入与区分力 | 入口与期望 | 关联 |
|---|---|---|---|
| V-01 同源成功 | 两库各构造 `[0,1,2,3,4,5]`，同一正例涵盖 `[0:3:2]`、`[:3]`、`[2:]`、`[:]`、`[1:4:]`、`[::2]`、`[-3:]`。NumPy 再验 `[::-1]`、`[4:0:-2]` 和结果的 `[0]` 继续索引。 | 双端普通源运行及原生参考逐项比对应为 `[0,2]`、`[0,1,2]`、`[2,3,4,5]`、`[0,1,2,3,4,5]`、`[1,2,3]`、`[0,2,4]`、`[3,4,5]`。NumPy 倒序 `[5,4,3,2,1,0]`、逆向部分 `[4,2]`、首元素 5。逐元素及长度均应相等。 | AC-02/04 |
| V-02 零步长 | NumPy 切片用运行时整数 0，PyTorch 用字面量 `[::0]`，验证零不会被改为默认值或在 bind 阶段拒绝。 | 两库的原生参考预期均 `ValueError`，T-01 冻结实值。NumPy 在 Auto 中捕获一次并运行后续语句；PyTorch 用例未捕获，AV 有失败终态且 A2PY 实际非零。两端核原类、分类 `python.5` 和完整失败表达式范围。 | AC-03/04 |
| V-03 PyTorch 负步长 | torch `[::-1]` 单独的同源 Case。 | 原生参考预期 `ValueError`。AV/A2PY 观测原类、分类、原始下标位置及 Auto 捕获一次后继续。若所冻结版本与预期不同，T-01 将原生证据与矩阵写回，再推进 T-02/03。实现沿原生协议，不以 `flip` 改写语义。 | AC-03/04 |
| V-04 语法兼容 | `s[1..3]`、原 `xs[0]`、`j["key"]`、`row[slice(0,3,2)]`、绑定的命名参数包含冒号，及畸形 `row[:::]`、多轴 `row[:,1]`、非 Python 接收者和切片写入。 | Schema/CST、共享绑定及原有消费者。原有形态通过原有效 check，新非法形态在正确阶段具名拒绝，源位置与相关 token 匹配。`row[f(n: 0):3:2]` 等嵌套形态只识别外层冒号。 | AC-01/04 |
| V-05 顺序与定位 | 有副作用的 Python 操作或 Auto 表达式分别作为接收者、三槽，测试 `slice` 同名绑定。一个槽位内置失败操作早于切片失败，并在源码前移插入含 UTF-8 的前缀。 | AV/A2PY 按接收者→起点→终点→步长的单次 trace 与独立 Python 对照。失败后无后续槽位副作用，且位置锁定内层实际操作；含前缀重放必须用新位置。旧位置、错误原类、丢步长或源指纹篡改均需使比较失败。核临时键沿同 session 在成功和失败用例收尾。 | AC-02/03/04 |

AV 走 `case-av-language-parity` 普通源码入口，A2PY 走 `case-a2py-language-parity` 的 `run-py`。按 `scripts/test-groups.toml` 选 `ace-syntax ace-source ace-python ace-a2x` 及这两个真运行组，实际触及通用 lowering、语义、适配或工程装配则按职责增加相应 `ace-lowering`、`ace-semantics`、`ace-adapters`、`ace-m2-build` 等现有组。公共 Schema 和语料下游包括 A2R 与 AAC；T-01 核哪些入口真消费修改，给同套件的 `case-a2r-language-parity`、`case-aac-gen1-language-parity` 确定兼容验证并说明取舍。保 1107/1110/1126 及 1086 原相关回归。Windows 的 Linux/WSL 用例沿仓规具名排除。

## 7. 验收标准

- [x] AC-01（owner_stage=work）共享 Schema/CST 和绑定实现新后缀，确认已有 `[..]`、普通索引和命名实参语义保持，V-04 解析、投影和绑定反例区分正确阶段。
- [x] AC-02（owner_stage=work）V-01 双端逐值与原生 Python 及固定预期相等，V-05 证明源序单次求值、内建绑定和临时资源归属。
- [x] AC-03（owner_stage=work）V-02/03/05 记录原生参考的原类和双端实际分类、捕获次数或退出、原 Auto 逻辑路径与半开字节范围，含内层失败和位置位移反控。
- [x] AC-04（owner_stage=work）同源正例含两库，每个异常 Case 有同源 AV/A2PY 实跑、独立原生参考、实际运行身份与共享消费回归。语料及 oracle 有明确维护入口。
- [x] AC-05（owner_stage=review）独立核共享语义、语法兼容和错误观测，按冻结提交验证 AC-01..04，并审查规范增量。
- [ ] AC-06（owner_stage=merge）跑冻结职责及必要消费者门，核规范沉淀、正式合入、计划归档和保护式清理。

## 8. 执行步骤

开工在当前执行环境登记任务 ID（如果有原生 TODO）。唯一实施树为 `D:/autostack/.wt/lang-1127/auto-lang`，分支 `plan-1127-dev`。授权后从 T-01 开始，计划簿记留主检出。

- [x] T-01 冻结语法与运行契约
  - owner_stage: work。前置 PLAN-1126/1110/1107 已正式合入。以 §4 的 Schema/CST、resolve、普通源码入口及实际两包参考验证最小端到端可行性。记录 profile 与现有模式、槽位 `None`、内建构造、求值与错误位置在两链的数据流和最小兼容行为，冻结 Python/包版本、源与命令后进入 T-02。关联 AC-01..04。
- [x] T-02 交付共享冒号切片
  - owner_stage: work。前置 T-01。在 `auto-source.at` 与共享投影、绑定路径实现可选槽位及 Python 来源检查，保普通索引和 Auto 点点模式，闭合 V-04。关联 AC-01。
- [x] T-03 接通两端的键与观测
  - owner_stage: work。前置 T-02。两端按同一受检事实构造键并调用现有 GetItem。保临时归属、求值次序和 1110 原类/位置观测；在真实两库验证 V-01..03、V-05。关联 AC-02/03。
- [x] T-04 落地同源语料和独立参考
  - owner_stage: work。前置 T-02/03。落新语料及参考，复用 1126 的源身份和 1110 的诊断校验。完成 V-01..05 固定期望、三方结果、反控和复放；记录实测支持范围、完整职责与必要消费者组的选择。关联 AC-01..04。
- [x] T-05 独立复审
  - owner_stage: review。前置 T-04。锁提交，逐项核 AC-01..04 与规范增量，复查 parser/profile 兼容、跨边界生命周期及无绕过路径。关联 AC-05。
- [ ] T-06 合入归档
  - owner_stage: merge。前置 T-05 pass。冻结职责组和实际下游门、规范沉淀、归档、材料对账并按仓规保护式清理。关联 AC-06。

## 9. 复审记录

2026-11-23，`stage: new | PLAN-1127 r1 | status: executing | progress: 0/6 | outcome: pass | next: work T-01（待实施授权）`。草案已核 Schema/CST 现有两种后缀、正式依赖状态、真实入口和固定组；新增源码及产品验收均待实施。任务 T-01..06、验收 AC-01..06 为本版执行合同。

### 2026-11-23 实施验收

普通 Auto 与包裁剪共用切片 CST，语义沿已有 Python provider 准入。包模式受检反例为 `source.use-py.surface` 和 `source.py-slice-receiver`，窄 aggregates profile 在原语法入口拒绝。两个畸形下标的 Text 诊断定位方括号内首个冒号，切片写入 `source.py-slice-write` 定位被写的切片后缀。共享绑定记录接收者和三槽，源查询的受限 inference 入口以 `python-slice` 具名拒绝，包重写、泛型与 CT walk 遍历接收者及实际槽位。

实施发现原缓存只按固定目录判断 DLL 存在，导致新 provider 描述配上旧二进制（SHA256 `0c7909d4…`），对切片帧返回 `-105`。现按 provider 与 shim 内容摘要派生缓存目录。第一次拼接双摘要触发 Windows LNK1104 路径长度问题，改为组合摘要后显式准备生成新 DLL `72bf577bd285cfa7992f25e4c681c3e115a8af191f3f1c391323dacc66a4028e`。同进程实测 registry 计数 2→3（成功，仅结果加一）→3（零步长失败，类 5）→4（继续普通索引）→0（close 返回 4）。`t03-lifetime.txt` SHA256 `5b1cdb3cf130a92b8615b25e52487d6b5b395417e39ea464ecaa3fc32e9fa802`。

AV 冷链首次双库正例触及 16 MiB ownership 验证门，合并相邻静态帧片段，双库使用同一个形态循环，顺序和兼容检查单列同源 Case。保全部形态和 oracle。并行构建相撞的单例首跑超时、总墙 10.634 s（AV 冷导入撞 2 s 产品预算）；空闲环境隔离重放见下表，未改任何预算。Windows Linux/WSL 腿依裁定排除。

| 新 Case | 可观察行为 | AV / A2PY 检查 s | 同一 Case 含准备与收尾 s | 原生参考 s（dev/test） |
|---|---|---:|---:|---:|
| numpy-torch | 同一矩阵覆盖两库的正/省略/负端点，NumPy 负步长、逆切片与再索引 | 2.029 / 2.132 | 6.321 | 1.837 |
| order-compatibility | 命名实参、recv→start→stop→step 恰一次、本地 slice 遮蔽、显式 slice、Python 对象 bound、Auto 和 Python 普通索引、点点范围 | 0.972 / 1.095 | 4.257 | 0.119 |
| numpy-zero | 动态零步长，ValueError / python.5，117..128，捕获一次后成功 | 0.970 / 1.185 | 4.316 | 0.155 |
| torch-zero | 字面量零步长，ValueError / python.5，71..79，AV lang.error、A2PY 实际进程退出 1 | 1.944 / 2.143 | 6.243 | 1.837 |
| torch-negative | 负步长，ValueError / python.5，100..109，捕获一次后成功 | 2.062 / 2.219 | 6.460 | 1.839 |
| bound-error | 槽内 loads 抛 JSONDecodeError / python.10，219..229，步长未执行 | 0.992 / 1.055 | 4.214 | 0.144 |
| bound-error-shifted | UTF-8 前缀重放，240..250，捕获一次且继续执行 | 0.953 / 1.004 | 4.165 | 0.139 |

| 本轮测试/命令 | 结果／选择 | 准备/编译 | 发现 | 运行 | 清理 | 总墙钟 |
|---|---|---:|---|---:|---|---:|
| `cargo nextest run --offline -p ace-core --test conformance -E 'test(py_slicing_1127::)'` | 4/4 PASS，902 因定向选择未执行，前两轮保留 Text 定位反例 | 6.10 s | 未单列 | 0.089 s | 未单列 | 7.929 s |
| `cargo nextest run --offline -p ace-a2x-python --test py_slicing_1127` | 7/7 PASS | 14.90 s | 未单列 | 1.839 s | 未单列 | 17.309 s |
| `cargo t case-av-language-parity case-a2py-language-parity --prepare-only` 后 `auto-case module-case` ×7 | 14/14 PASS；Case-only 选择，非完整组 | 工具测量合计 31.823 s | 含总墙 | 上表 | 含总墙 | 36.157 s（仅 7 Case） |
| `python -I -B t03-lifetime.py` 同进程 ABI | 1/1 PASS（成功、失败、续用、close） | 复用准备 DLL | — | 2.527 s | 含总墙 | 2.527 s |
| `python -B controls/replay.py`，反控副本 | 错步长、旧偏移、错原类与分类，AV/A2PY 分别 FAIL；改 worker 源指纹为零，AV PASS / A2PY 具名 ERROR | Rust proxy 显式预编译 | — | 2.247、2.184、2.067、1.377 s | 含各总墙 | 分例如左 |

四个新增前端 Case 耗时 0.047（畸形/逗号拒绝）、0.053（包准入）、0.062（写入和非 Python 基值分阶段）、0.087 s（嵌套冒号/槽位位置）。启用 `AUTO_CASE_WIRE_EVIDENCE=1` 对比 AV 请求的实际 source_b64 和 A2PY 暂存字节，七条均为同一 `main.as`，源哈希与 A2PY 响应一致，例级 source identity 两视图一致。证据 `identity.json` SHA256 `0ddf6b1002bab42460df8e8793a891155e193421460eac934be24f377c712fbb`。CPython 子进程版本/程序哈希 `e725f520cfbb2e48…`，numpy/torch 与 AV 共享同一安装来源。编译日志为旧基线警告，包括未用 import/字段、可变绑定和 private_interfaces；本次移除了 py_context 的未读字段警告。

冻结职责选 `ace-syntax ace-source ace-python ace-a2x ace-semantics ace-comptime`，以及 `case-av-language-parity case-a2py-language-parity`。ABI 只改 py 构建缓存及 provider 的取项实现，Python 职责和真实 AV 运行覆盖；MIR lowering 和工程组装契约保持。A2R 和 AAC 同读语料，复审核 check 选择与代表性已有语法消费者，AAC 窄语法 profile 不包含切片新规则。总体固定组门由 merge 在冻结修订执行。

`stage: work | plan_id: PLAN-1127 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1127/auto-lang | branch: plan-1127-dev | base_commit: dca52e196a64f123ca057e1442c71c8f3e95ab30 | code_commit: 2a35bde448e876ab31158e30da9668462115ec2a | dependency_revisions: auto-down@456c86ac886f9781e56f6153ce9aa41cd59e5c8e / CPython 3.14.2 / NumPy 2.5.1 / PyTorch 2.14.0+cpu | task_ids: T-01..04 / AC-01..04 | evidence: 本节 | blockers: none | next_responsible_role: final review`。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: 2a35bde448e876ab31158e30da9668462115ec2a | authorization: /goal 完成计划1127 | next: review`。

### 复审 1 — needs_fix

`stage: review | review_scope: final | reviewer_identity/limitations: 同一目标会话分阶段复核，未分配第二 reviewer；以冻结 Git 提交、原始运行与新 holdout 重建判定 | plan_id: PLAN-1127 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1127/auto-lang | branch: plan-1127-dev | reviewed_commit: 2a35bde448e876ab31158e30da9668462115ec2a | base_commit: dca52e196a64f123ca057e1442c71c8f3e95ab30 | dependency_revisions: auto-down@456c86ac886f9781e56f6153ce9aa41cd59e5c8e | spec_inputs: SD-01..04 基面 Git blob 1851ee08 / bfa7dd0d / 5bde8e23 / c61f68d6 | pending_closeout_gates: 冻结职责并集 | findings: F-01..03 | next: work`。

| Finding / AC | 可重放证据和预期 | 影响与关闭条件 |
|---|---|---|
| F-01 / AC-01、02：被标记 Python 对象的可写绑定可接受裸整数 | `var row = arange(6); row = 2; print(row[:].tolist().to(str))`。普通 AV 实际成功打印 `[0, 1, 2, 3, 4, 5]`（2 复用了 live token），A2PY 以 TypeError/python.3 失败。本计划要求受检接收者；在绑定层阻止 Python 绑定写入非 Python 值。 | 检查直接和复合写入与控制流；用 Python→Python 的原变量重赋值作合法控制。 |
| F-02 / AC-03：括号开头未计入下标位置 | `print((arange(6))[::0])`，两端原类 ValueError、python.5；实测 `44..59`，原 `(arange(6))[::0]` 应为 `43..59`，含左括号。 | 检查括号接收者、括号后链式接收者和内部失败的范围；增加合法及错定位反控。 |
| F-03 / AC-04：Python ABI 日常固定组有可达的准备回退 | `ace-python` 工具声明为空，经测试 `open_py_bridge_session` 调 `py_bridge_shared_build`，新源码键冷 miss 触发 Cargo。 | 保留显式准备，在日常路径严格加载已准备且身份匹配的 provider，缺失须具名失败；用户 API 原入口保原用途。 |

两条独立 `auto-case module-case` holdout 使用 r1 冻结实现和准备工具，1.999 / 2.033 s，均退出 1；含拆解和收尾（阶段时间未单列）。第一次第一例误写作者文档 stage `bind`，改用合法 `resolve` 后测得上表。其他 AC-02/03 的已跑矩阵、AC-04 同源身份与反控复核原收据（当前仍受 F-01..03 限定），本轮未复跑职责并集。规范增量目标和原存文档已核，最终措辞待受检写约束与源范围修复后再冻。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: 2a35bde448e876ab31158e30da9668462115ec2a | next: work`。

### 修复 1

- F-01：已标 Python 对象的局部在直接赋值时要求 Python 对象值，拒绝裸标量、复合写和可写捕获为 Auto 单元格。合法 Python→Python 写仍通过，原双包样本依次替换接收对象并逐值复跑。AV 复跑原反例在 bind 返回 `source.py-domain`，范围 `64..65`。A2PY 编译子进程在源编译退出 2，未启动 Python。
- F-02：postfix 投影另保存接收者 CST 开始位置，切片本身范围包含括号，内部表达式原 span 保持。原 holdout 两端 `ValueError/python.5`、`main.as:43..59`。新增 py ABI 用例还钉住 `(row).reshape(3)[::0]` 整段受检事实并检查直接和复合非法写。
- F-03：`ace-python` 显式声明 `av-providers`，1026/1107 的 ABI 测试经 `load_prepared_py_bridge` 匹配源码与 shim stamp、DLL SHA256，缺 descriptor 直接给 `foreign.prepared`。产品 `PySession::open` 保显式原用途。新增缺 descriptor 用例回放无 Cargo 路径。

| 测试/命令 | 结果 | 准备/编译 | 发现 | 运行 | 清理 | 总墙钟 |
|---|---|---:|---|---:|---|---:|
| `cargo nextest run --offline -p ace-abi --test conformance -E 'test(p1026) \| test(p1107)' --config-file ../../.config/nextest-full.toml` | 26/26 PASS，204 未选 | 31.40 s，provider 已准备 | 未单列 | 5.943 s | 未单列 | 38.016 s |
| `auto-case module-case` 三条受影响原 Case + 两个 holdout | 正例、兼容、位移 6/6 PASS，括号异常 2/2 PASS，覆盖句柄 holdout 两端在绑定/共享编译止步 | case-parent、AV/provider/A2PY 调试预备 35.062 s | 含总墙 | 6.269 / 4.115 / 4.110 / 2.005 / 1.976 s | 含总墙 | 各例同左 |
| 本轮新增 2 native Case | F-01/02 direct/compound 与链式范围 0.133 s；F-03 加载器缺 descriptor 0.016 s | dev/test | 含上组 | 各例同左 | 含各例 | 均 ≤10 s |

本轮复用未受影响的 reference（源码、Python 解释器和 oracle 未变）、provider lifetime（provider bytes 及实际 DLL 未变）和 §9 初始反控；统一回归仍 pending。修复原始证据 `review/holdouts-repair.txt` SHA256 `35012be71de6803cf8174035efcab5fc502fa4899ad50c0d4c68e70246eacf04`、`repair-py-abi.log` SHA256 `de20edb719dad1f790b0b988d229c3abfbeb32cf5ab7859b6b2b1a3ff86e29c2`。

`stage: work | plan_id: PLAN-1127 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1127/auto-lang | branch: plan-1127-dev | base_commit: dca52e196a64f123ca057e1442c71c8f3e95ab30 | code_commit: 10047ade286efdb568befb3adc5bdd7a07aa3768 | dependency_revisions: auto-down@456c86ac886f9781e56f6153ce9aa41cd59e5c8e / CPython+NumPy+PyTorch 同前 | task_ids: T-02..04 F-01..03 | blockers: none | next_responsible_role: final review`。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: 10047ade286efdb568befb3adc5bdd7a07aa3768 | next: review`。

### 复审 2 — pass

| 验收 | 独立核验与范围 |
|---|---|
| AC-01 | PASS。冻结源码核查 Schema 的裁剪、CST 槽位、包和限定语义查询拒绝、运行域与可写 Python 绑定。4 条前端正反例，使用初始不受影响记录，F-01 的 raw 指纹按两端修复重放关闭。 |
| AC-02 | PASS。复核原 7 场逐行 oracle、单次源码序、遮蔽与 PySession 句柄关闭。Python→Python 改写在实际双库循环中保持，裸值/复合写在受检入口关闭。 |
| AC-03 | PASS。原始请求双端字节及路径相等，围绕零步长、负步长、槽内异常与 UTF-8 位移观测，错误类/范围/次数和 A2PY exit 均符合。括号反例重放 2/2，链式括号相关 span 在新 ABI 定向测试钉住。 |
| AC-04 | PASS。7 份单一作者源共 14 条实际检查，7 条独立原生参考。两引擎及原生见 §9 矩阵。新前端、参考、ABI 用例均在时限内。F-03 已改日常入口，追到描述符、stamp 及 DLL 哈希检查，缺准备没有编译回退。公共 suite inventory 中新 Case 只有 av/a2py 各 7 行，A2R/Gen1 不选新 Case，Gen1 窄 AAC 裁剪无切片新臂。A2R 共享普通入口的原有形态由冻结 source、A2X 组及 A2R 集成验证。 |

本轮直接 A2PY worker 重放错误输入，观测 `a2x.compile`、原因 `bind/source.py-domain 64..65`、退出 2，和 AV 拒绝一致（`review/overwrite-a2py-direct.json` SHA256 `602be7a3ec5dc66b69924582983d77b9358e363fe5b8488aaebE4e204af65a6e`）。`auto-case inventory` 1.164 s、退出 0，含发现及清理（未单列），结果 SHA256 `d080a1bc1585d9c0665da7b57aa5b3694d95864d3347a076e219569a9dabb352`。复核上一轮定向 26/26、两条 holdout 及原矩阵，未重复执行／添加用例。范围内无遗留 workaround 或延后项目，未引入新编译告警。完整冻结组、消费者源与落地走查由 merge 执行。

SD-01..04 仍是现有 canonical 路径增补（无新 module 或 goal 状态变化），review 冻结文本都在 `review/spec-delta`，SHA256 顺序为 `c552baa1d00d42ca…`、`1f87a164d697af11…`、`d510041187ac3ca0…`、`08d6d3746bf6d092…`。四项原目标 blob 与复审 1 相同。

`stage: review | review_scope: final | reviewer_identity/limitations: 同目标会话的独立复审阶段，按冻结代码、原始证据和复审反例重建 | plan_id: PLAN-1127 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1127/auto-lang | branch: plan-1127-dev | reviewed_commit: 10047ade286efdb568befb3adc5bdd7a07aa3768 | base_commit: dca52e196a64f123ca057e1442c71c8f3e95ab30 | dependency_revisions: auto-down@456c86ac886f9781e56f6153ce9aa41cd59e5c8e / CPython 3.14.2 | spec_inputs: SD-01..04 | acceptance_results: AC-01..05 pass | pending_closeout_gates: 冻结职责组、必要消费者、规范验证、归档和清理 | findings: F-01..03 closed | next: merge`。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: 10047ade286efdb568befb3adc5bdd7a07aa3768 | next: merge`。

`stage: merge | checkpoint: prepared | plan_id: PLAN-1127 | plan_revision: r1 | reviewed_commit: 10047ade286efdb568befb3adc5bdd7a07aa3768 | delivery_commit: 16118193d845dbf2410fdd6b86f754d6047a6c0e | base: dca52e196a64f123ca057e1442c71c8f3e95ab30 | canonical: SD-01..04 + auto-av/overview.md navigation | frozen_gates: pending`。

冻结选择配置 SHA256 `1e3c426e07a22d66605c59ac33cf8112ef57e234fdfde4cdfa7c05bdd36c8274`。计划源、自动生成 spec 索引及用户已有并行草稿分别保留。A2R 固定组的准备自动请求了 release worker，发现后终止本计划该准备分支，沿同一公开语料入口以已准备的 debug worker 验证原 1082 方括号读写。AAC Gen1 从选中检查及窄裁剪核不选择新增 Case，保留固定源组的 profile 兼容覆盖。

### 冻结合并门第一轮 — needs_fix

`cargo t` debug 固定门在 `16118193d845dbf2410fdd6b86f754d6047a6c0e` 的原始结果为 5/7 个 native 组通过。通过的组为 ace-syntax、ace-semantics、ace-comptime、ace-a2x 和 ace-python（独立 p1026 入口 12/12）；ace-source 994 执行有 6 红，ace-abi 342 执行有 51 红。具体源码/环境归因与基线重放继续记录在本节。

F-04（AC-04/06）为本计划的测试准备回归。共有 22 个 Python ABI 检查经新 `py_prepared::provider` 拒绝，而同一实现的独立 `ace-python` 准备录入 `av-providers` 后通过。准备工具审计确认 `av` 不会为 nextest 合并传递 `ACE_AV_PROVIDER_ROOT`，`ace-abi` 和 `ace-workspace` 也消费这些测试，必须显式声明同一工具。正式 PERF 选择只包含性能命名空间。

AV/A2PY 语料结果所有 14 个新检查 PASS。AV 原有两条 `foreign.prepared` 与 1126/1110 收据同签名。具体计数和时间在最终门记录保留。A2R debug 入口真实跑 `av.parity.1082.bracket-read-write / a2r` 1/1 PASS，2.627 s 总墙。

`stage: merge | plan_id: PLAN-1127 | plan_revision: r1 | outcome: needs_fix | frozen_delivery: 16118193d845dbf2410fdd6b86f754d6047a6c0e | finding: F-04 | next: work`。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 5 | from: merge | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: 16118193d845dbf2410fdd6b86f754d6047a6c0e | next: work`。

### 修复 2

冻结原生门原始结果 SHA256 `33d89d83a6f29e44d245e79169f1ab81fd97d6164dcf78aeeBFDD5AF37EF0C48`。在同一专属树临时检出开工 commit `dca52e196`，独立 debug 准备基线子进程并以原 nextest 配置重放命名失败。准备 100.445 s。source 6 选、5 红、深链 1 PASS（总墙 1.540 s）；ABI 的 29 个非 Python 原失败中 28 同红、并发 SQLite GC 重放绿（总墙 13.217 s）；发现/清理未单列。文本 5 条同为 Verify `verify.double-move`，`l6`，1444..1454；ABI 常量对照 C runtime SHA `0a4f68e3…` 与冻结 `bd260a8b…` 不一致及相关既有资源断言同复现。ABI 单次 `Sql("database is locked")` 与重放绿单列，等待复测。基线日志 SHA256 source `ca33c4e5689c2085a5909adb7ceb01aeb256142a483b1f09bea2a984a2716b05`、ABI `0ef64b8dacf33df9b52be901decfb094687a01d1bcef5b9bc19fef8a7d7f3ba4`。

F-04 在 `ace-abi` 和 `ace-workspace` 显式声明 `av-providers`，`cargo t ace-abi ace-workspace --select-only` 核两条准备合同，PERF 不消费 Python 日常用例。F-05（AC-01）是基线绿、实现组溢出的 16,384 项 Scalar `&&` 用例。共享闭包捕获预扫描原会递归二元子树，新增槽位遍历扩大了该递归路径。两个捕获 visitor 复用左→右显式二元栈，保存捕获顺序；原样本、栈预算和入口未变。定向 `cargo nextest run --offline -p ace-core --test conformance --config-file ../../.config/nextest-full.toml -E 'test(deep_logic_chain_reaches_checked_bundle) \| test(closures_context_913::) \| test(py_slicing_1127::)'` 9/9 PASS、897 未选，构建 11.42 s、运行 1.078 s、总墙 13.131 s（发现/清理未单列），深链 1.067 s。修复未新增测试。

`stage: work | plan_id: PLAN-1127 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1127/auto-lang | branch: plan-1127-dev | base_commit: dca52e196a64f123ca057e1442c71c8f3e95ab30 | code_commit: ffbe46b73d28da53d67ab104bb013974ce31f934 | findings: F-04/05 | dependency_revisions: 同前 | blockers: none | next: review`。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 6 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: ffbe46b73d28da53d67ab104bb013974ce31f934 | next: review`。

### 复审 3 — pass

复查 `git diff 16118193d..bafc880f1` 只有两个变更文件。Python ABI 测试严格读取准备工件，`ace-python`、`ace-abi`、`ace-workspace` 进入相同 loader；工具选择证明两个新增入口已声明。捕获遍历显式栈后推右、再推左，保持旧左→右 DFS 顺序和 scope，深 Scalar 原用例在同样预算下绿。核定向 9/9 原始结果和开工基线反例，无新增用例／单独产品执行。最后提交相对实施收据 `ffbe46b73` 仅移动一行原注释，因此重用定向验证。既有 F-01..03 路径、oracle 及审定规范原文字节均保持；AC-01..05 通过。合并必须重跑 `ace-source ace-abi` 和双端 Case 门，复用未被修复触达的语法、语义、CT、A2X 组及 A2R 控制。

`stage: review | review_scope: final | reviewer_identity/limitations: 同会话独立复核冻结证据及选组 | plan_id: PLAN-1127 | plan_revision: r1 | outcome: pass | reviewed_commit: bafc880f13e00a078b2ab24fa3da438757764bec | worktree_path: D:/autostack/.wt/lang-1127/auto-lang | branch: plan-1127-dev | base_commit: dca52e196a64f123ca057e1442c71c8f3e95ab30 | dependency_revisions: auto-down@456c86ac886f9781e56f6153ce9aa41cd59e5c8e | spec_inputs: 已冻结 SD-01..04，加 AV overview 导航 | findings: F-01..05 closed | pending_closeout_gates: 修复影响职责门、落地、归档、保护式清理 | next: merge`。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 7 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: bafc880f13e00a078b2ab24fa3da438757764bec | next: merge`。

### 最终冻结门原始记录

最终冻结提交 bafc880f13e00a078b2ab24fa3da438757764bec。AV/A2PY 的 source-state bc7d041fe3084daa845ea6f91f447c47579b5929d53a979c38bb5da664d3caa6，配置 SHA256 6cf3bf1a5e2766f25c60a0708c6d77434d0fd190274a196a910ad4e487e8f347。debug 准备退出 0（52.596 s），统一执行退出 1（123.202 s），待执行资源 364.069 s，已与门墙钟分列。AV 原始 548 PASS / 2 FAIL / 1 ERROR / 200 NOT_RUN / 161 SKIP，worker 87.145 s。两 FAIL 是已见的 1093 C 准备缺失。并行构建在执行中重新启动，新增双库正例 11.204 s 触发 av.resource.execution-wall（foreign call exceeded wall budget），首轮错误保留；其余 6 条新增 AV 检查通过。A2PY 固定组 PASS，14 PASS / 898 NOT_RUN，worker 26.023 s，7 条新增全绿；子报告因未选行返回 1。发现及清理均包含在统一墙钟，未单列。合并报告 SHA256 adfa760873af1a59e213a70753b06e253902156b9faa73d6a463f4e8f4ae0f4c，使用 PLAN-1127 的隔离解释器，用户级环境初查另见下段。

本机只读检查用户级 A2PY_PYTHON 所指 PLAN-1126 专用 venv。在 -I -B 下 NumPy 可见，Torch 不可见。PLAN-1126 的 Q-01 明确授权该 NumPy 环境，PLAN-1127 的既有双库验收使用临时解释器。已请求用户裁定持久环境调整，仓内验证继续进行。

### 最终冻结门对照和重放

| 冻结入口 | 结果 | 准备 | 发现 | 测试 | 总墙（各命令/子执行） |
|---|---|---:|---:|---:|---:|
| cargo t ace-source | 989 PASS / 5 基线红 / 1 SKIP；深链 1.414 s PASS | 同下组 | 0.473 s | 9.387 s | 9.865 s 子执行 |
| cargo t ace-abi | 314 PASS / 28 基线红 / 8 SKIP；Python ABI 定向用例和 SQLite 并发 GC 均 PASS | 120.912 s，两组统一准备，退出 0 | 1.404 s | 48.365 s | 49.657 s 子执行 |
| AV 双库正例隔离重放 | 1/1 PASS，退出 0，受检行 1.992 s | 复用冻结 debug manifest | 含总墙 | 同左 | 4.353 s |
| 复用 ace-syntax、ace-semantics、ace-comptime、ace-a2x、ace-python | 分别 159、295、136、361、12 PASS；条件排除 3、10、19、7、218 未选 | 第一轮 7 组统一准备 196.002 s | 未单列 | 2.468 / 3.378 / 11.764 / 65.556 / 1.978 s | 子执行 3.453 / 7.231 / 13.445 / 68.301 / 2.457 s |
| 复用 A2R 原方括号读写 | 1/1 PASS，实际入口、退出 0，其他 Case 未选 | 复用同源 debug worker | 含总墙 | 0.471 s 子项 | 2.627 s |

原生两组统一运行 63.065 s、退出 1，清理含总墙，准备／运行前资源轮询 0.186 / 0.158 s。最终报告 SHA256 2084884729456e1bf677f0423419f503dcbcc63c579df188bd936e7f12e54219，source-state 和配置与最终双端相同。开工提交按用例 ID 与终态比较，source 5/5、ABI 28/28 一致。基线有一例 SQLite 并发重放绿，最终整组该例 PASS（6.265 s）。ACE Python 的两个新增 ABI 检查 0.101 / 0.430 s。固定组保原来条件及已有较慢用例。

AV 以同一冻结 case-parent、AV worker 和 provider，运行 auto-case run --target av --case py.slicing.1127.numpy-torch，未改任何墙钟限制或断言。等待 0.177 s，报告 SHA256 4f5f975b2efc38f0a32a94f850427ba2d3fdd5fb3f4f67809c29c45183aa8cd0。AV 其余新增行用时 0.903 / 0.882 / 1.956 / 2.210 / 0.882 / 0.872 s；A2PY 依次 2.762 / 1.409 / 1.403 / 2.612 / 2.887 / 1.384 / 1.362 s。隔离重放另记，保留统一门首次的 11.204 s 超时。源码原样本、固定 oracle、工具输入范围不变，未复跑不受最终两个修复文件影响的通过组。Windows 的 Linux/WSL 腿依裁定排除，归专用环境。

默认环境反控：用最终冻结 Case 工件与用户级 A2PY_PYTHON，执行 auto-case run --check a2py --case py.slicing.1127.numpy-torch，退出 1，3.358 s 总墙，所选检查 1.094 s。实际 ModuleNotFoundError / python.1 / main.as:57..63（No module named 'torch'），Python 子进程退出 1。报告 SHA256 05602c04224a8ded008f4e4ec80839939e37edeba0a87508fa886e196a5ad9a6。较早用 --target a2py 收到选择器错误（退出 2），已用正确入口确认。D:/autostack/.tools/a2py-py314-numpy 和用户级变量均未修改。等待裁定期间保持 reviewed、专属树和证据；默认环境授权后只重放 A2PY。若用户明确接受单次隔离配置，按其裁定闭合默认环境差异。

`stage: goal | plan_id: PLAN-1127 | plan_revision: r1 | round: 8 | from: merge | to: merge | status: reviewed | outcome: waiting | worktree: D:/autostack/.wt/lang-1127/auto-lang | commit: bafc880f13e00a078b2ab24fa3da438757764bec | next: A2PY 持久环境裁定后合入`。


### 用户授权后复验默认 A2PY 环境

2026-11-23，用户授权既有 A2PY venv 复用本机已安装的 CPU PyTorch 与依赖，并要求复跑 A2PY 固定组后合入、归档和清理。用户级环境变量保持原值，在 PLAN-1126 既有 venv 的 site-packages 中新增 plan1127-shared-pytorch.pth，指向同一 CPython 3.14 的现有包目录。此配置对该 venv 可见。python -I -B 中 isolated=True、ENABLE_USER_SITE=False，NumPy 2.5.1 仍载自 venv，PyTorch 2.14.0+cpu 载自该显式目录，torch.version.cuda=None。两库 [::2] 均返回 [0, 2, 4]，退出 0，1.745 s。Torch CPU DLL SHA256 5a68d4a6e561fcc328eb16b2134aa6c69a141f9d8172e9ac2407d8f9ddf42c32，.pth SHA256 21d0f6c99853b809e141f38957175ad596255f0b3a69182190780b17a48860fd，smoke SHA256 c74df23fe47f950ee662e4fbbdd316037b59e9bb610371cec87bf1e09cf9a80c。

在最终冻结提交执行 cargo t case-a2py-language-parity --run-prepared，使用原冻结 debug 清单及用户级解释器。驱动采用清单中全部 invocation，实际同时跑了 AV 和 A2PY。A2PY 14 PASS / 898 NOT_RUN，含全部 7 条本计划新增检查，分别 2.606 / 1.510 / 1.507 / 2.695 / 2.617 / 1.507 / 1.473 s。A2PY 子执行 26.025 s，固定组 PASS，原子报告因未选行退出 1。AV 548 PASS / 2 既知 1093 C 准备红 / 1 双库墙钟 ERROR / 200 NOT_RUN / 161 SKIP，76.411 s；该错误与前次签名相同，发生时同机已有编译活动，原冻结隔离重放 1/1 PASS 证据有效。统一命令 110.656 s、退出 1，构建复用、发现/清理未单列，资源等待 0.161 s。共同 source-state 和配置同上，报告 SHA256 8f62d2e9c808e04684ea2490a0becb64ea42c8b5aabdb8c4c002776d9979cd01。持久 venv 是用户的机器环境，继承 PLAN-1126 的保留记录；本计划的临时 venv、输出和诊断在专属材料目录内。

`stage: merge | checkpoint: prepared | plan_id: PLAN-1127 | plan_revision: r1 | reviewed_commit: bafc880f13e00a078b2ab24fa3da438757764bec | delivery_commit: bafc880f13e00a078b2ab24fa3da438757764bec | frozen_gates: pass-known-reds-only | retained_raw: 两条 AV 已知准备红、AV 双库墙钟首跑和复跑、33 条基线源／ABI 红 | runtime: 默认 A2PY 14/14 PASS，AV 双库隔离重放 PASS | canonical: SD-01..04 + 导航 | next: land`。

`stage: merge | checkpoint: landed | plan_id: PLAN-1127 | delivery_commit: bafc880f13e00a078b2ab24fa3da438757764bec | base: dca52e196a64f123ca057e1442c71c8f3e95ab30 | default: master fast-forward`。

主检出核全部 37 个计划交付项与冻结提交一致。主检出的语言 suite 以冻结 debug case-parent/A2PY worker、默认 A2PY venv 执行新双库语料，所选检查 1/1 PASS（2.275 s，总墙 4.589 s，其他检查未选，原始退出 1），报告 SHA256 b58c16b803cc04f5c5f01d4adbb65863f38d7c324e1a5de90ed0b5ad7bcdfaae。

`stage: merge | checkpoint: canonical_verified | plan_id: PLAN-1127 | delivery_commit: bafc880f13e00a078b2ab24fa3da438757764bec | spec_delta: SD-01..04 + AV overview`。

在 master 直接核对规范文件与冻结待交付文件的 SHA256。

| 规范 | SHA256 |
|---|---|
| auto-syntax/grammar-contract.md | 95d9a2ec9929057558b56e4247f3037397f58508b353736cd322cff82211f30a |
| auto-av/python-interop.md | 22d3fcc6998fd6b5d313f4fcc9db43448b5129f15875cdd4931bd88d4f4068ea |
| auto-a2x/python-ecosystem-integration.md | 091f2a7d15576f9356b5e7989da9b14bb924c5b6289e79bf36ad501255d453ab |
| auto-test-case/feature-organized-corpus.md | df8e50bbf1555adf0202cec0db81659dc352fd7468e5abc7fcd53594726e0282 |
| auto-av/overview.md | 1011f2ec4362b0abb459656ccca71fa0c674fe9cffeb2e92f2704455b37d972e |

SD-01..04 保留 PLAN-1127 来源，AV 总览指向现有互操作规范。用户并行工作和遗留台账均保持原状。

`stage: merge | checkpoint: archived | plan_id: PLAN-1127 | status: archived | completion_kind: delivered | owned_tasks: 5/6 | cleanup: pending`。

实际归档态检查通过。已核 6 个任务、6 个验收项及尚未关闭的清理项，归档引用从新父目录解析，SD-01..04 回链到本计划且 AV 总览导航命中互操作规范。Design 58 的 §3.6 仍在主检出的并行草稿，归档链接定位已跟踪的 Design 58 文件。

## 10. 待澄清事项

开工调查及合入阶段的 A2PY 环境裁定均已关闭，见 §4 和 §9 的冻结记录。
