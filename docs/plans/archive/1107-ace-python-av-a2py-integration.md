---
plan_id: PLAN-1107
status: archived
feature_name: ACE Python AV与A2PY同源互操作及工程分发集成
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-20
plan_revision: 2
current_step: 8
total_steps: 8
depends_on: [PLAN-1026, PLAN-1042, PLAN-977, PLAN-978, PLAN-967, PLAN-1094]
supersedes_spec_components:
  - docs/specs/auto-av/python-interop.md
  - docs/specs/auto-a2x/python-stdlib-readiness.md
  - docs/specs/ace-migration/m2-build.md
new_spec_components:
  - docs/specs/auto-a2x/python-ecosystem-integration.md
touched_goals: [GOAL-002]
affects: [ace/source, ace/av, ace/a2x-python, ace/m2-build]
---

# [PLAN-1107] ACE Python：AV＋A2PY 同源互操作与工程分发

## 0. 变更摘要

用户2026-11-19明确Python后续能力（含978相关构建/装配/分发）直接在新版ACE做，采用AV＋A2PY。1026/1042/977/978已经归档，本计划是消费其成果的后继，不重开历史计划、不重新实现Python后端或SDK。

AV沿已交PySession/967 provider调用CPython，A2PY沿已有Checked HIR→共享A2X→ace-a2x-python生成Python并直接使用当前Python环境的库；两条路径共用源语义与独立oracle。978的工程缓存/身份/SDK在Python维度接通这两条路径；原native/AC/A2R/A2C职责保留。当前只规划，不实施/构建/运行探针。

## 1. 目标

- 同一份标准Auto Python程序经新ACE AV和新ACE A2PY均真实运行，与独立原生Python参考一致；以§5 PyTorch训练为首个完整工程。
- Python成员、调用、kwargs、索引、运算、显式转换、对象身份/生命周期及错误经共享受检语义消费，不回旧AutoVM/旧PythonTrans。
- 基于既有978 m2-build完成Python源构建/文件集/预编译消费/可重定位SDK，构建身份、运行兼容、工件选择分账保持。
- 精确登记已交、待接线、待验证、缺能力/环境；不将已有A2PY基础设施当作Python库互操作已全面交付。

非目标：重做Python执行器/生成器/FFI/store/loader，重开归档计划，取消1094及1078–1096的AV-only阶段策略，GPU/图形Python后端、全部Python包或Python语言替代、任意动态反射静态推断、无边界零复制buffer、模型热迁移。AV其他语言齐平、完整UI/产品迁移及新平台认证继续归原owner。

## 2. 架构方案

目标设计为[Design58当前路线](../design/58-python-standard-syntax-interop.md)。共同Auto源→ACE统一前门/绑定/Checked HIR及受检Python操作身份，分两条消费链：

1. AV：唯一共享HIR→MIR→AV foreign stub→967 shim/Rust adapter→PyO3→CPython。复用1026 PySession/资源账，不复活旧VM。
2. A2PY：共享A2X语义视图→已有PythonTargetProvider→生成Python/runtime helper→声明的OS Python→原生import/getattr/call等Python协议。**Python库操作默认直接运行于生成程序的解释器，不绕回PyO3再次嵌入CPython**。真实C/Rust外库仍消费967既有ctypes/C ABI路径。

本计划只补共同语义投影、目标接线和工程消费，不造第二parser/typechecker或按torch名字/源文本正则替换业务。当前A2PY foreign_bridge只支持其冻结C/Rust域，不能把ctypes链已存在当作use.py训练已可运行。

## 3. 技术栈

已有`ace-core/source-query/lowering/av/abi`、`ace-a2x-core/a2x-python/a2x-store/a2x-worker`、`ace-m2-build`、TestCase/AutoDown及独立Python oracle。Python/torch环境按实际支持矩阵冻结；AV既有1026环境为Windows x64、CPython3.14标准GIL、PyO30.29.3、torch2.14 CPU，仅作为有来源基线，不伪称本轮重跑。

## 4. 需求分析与背景调查

### 授权与现状

- 用户本轮要求Python工作统一ACE AV＋A2PY，含978的Python工程能力。沿本聊天此前约定，仅先做设计/计划，不推定本轮work/安装/发布授权。无额外模型、并行agent、预算或工期指派。
- 主检出master，读取HEAD=`b067fc0603540118b792c7b7171df6c0bae88d59`。存在大量其他会话文档WIP/未跟踪Rust文件，均保留，不清理、不接管。已读auto-plan-new当前技能（§8用独立checkbox任务），Specs overview及相关模块。
- 已确认1026/978/1042/977归档终态，不改其状态/冻结验收。978 r14已依赖1042且包含A2PY；无需把整个978“迁移一次”，只认领Python生态工程缺口。
- 1094 r3是AV-only脚本/Python协议补齐，尚待实施；其源码/病例共享，非AV检查按阶段保留，不以本计划反向卡其AV出口。

| 当前来源 | 已观察事实 | 本计划消费/差额 |
|---|---|---|
| `docs/specs/auto-av/python-interop.md`；`abi/src/py_session.rs`；`core/src/source/py.rs` | AV标准use.py、CPU训练和17帧操作已交；受限显示/浮点及已知前端边界 | 复用产品入口和受检身份，真正源面差额交1094最小切片，不复制一套AV协议 |
| `docs/specs/auto-a2x/python-stdlib-readiness.md`；`a2x-python/src/{provider,lower,print,runtime,ports}.rs` | 新PythonTargetProvider、AXPY1..3、真实run-py及资源helper已交 | 增Python对象操作的目标投影/实际运行，不能回旧AST PythonTrans |
| `a2x-python/src/foreign_bridge.rs` | 生成ctypes调用冻结Rust/C provider，超域具名拒绝 | Python包直接导入是另一语义分支，不扩大C ABI猜签名，不套双重解释器 |
| `docs/specs/ace-migration/m2-build.md`；`m2-build/src/{driver,identity,compat,selection,recipe}.rs` | 五目标工程层已有av/a2py，三类身份分开，SDK带声明解释器 | 核use.py工程是否实际接到这些入口，补Python依赖/运行与精确失效，不重造构建系统 |
| 1026既有`pytorch_train.as`及README | 已交版本记录五处表面调整；T-01 实测：真正需要的只有“条件循环拼写 + 入口标注”两处，浮点后缀与两处 hoist 在当前 HEAD 均**不必要**，“动态浮点关键字实参已拒”已过期 | 作为历史基线；本计划要求§5.1收敛后的正文，不把过期边界写成现状 |
| `1094-av-parity-script-and-python-protocols.md` | 消费1026并补完整脚本/对象协议，当前仅验AV | 本计划消费所需正式前门/协议切片；A2PY对应检查只由本Plan按任务接手后打开 |

本轮只读源码/Spec/归档记录，无当前运行证据。语义冲突以规范/已批准合同核定，不能把旧bug或v1工程限制固化为全语言规则。T-01补来源SHA/具体接口版本与consumer矩阵，未知不填PASS。

### T-01 冻结结果（2026-11-20，work 树 `D:/autostack/.wt/lang-1107/auto-lang`，branch `plan-1107-dev`，base `5089d3de0`）

**基线（真实执行）**：`cargo t ace-python`（先 `python scripts/fixed-tests.py ace-python --prepare-only`）PASS，10/10 用例执行、0 排除；其中 `p1026_v01_acceptance_training_via_public_session` 2.98s、`py_bridge_1026::p1026_py_bridge_torch_slice` 2.69s，整组 wall 4.12s。环境：Windows x64、`C:\Python314`（CPython 3.14.2）、torch 2.14.0+cpu、`PYO3_PYTHON` 已设、sccache 命中。证据输出在 `D:/autostack/plan-artifacts/plan-1107/logs/ace-python-baseline.log` 与 worktree `target/test-groups/run-1xr_jhb7/results.json`（本地临时件，收尾删除）。

**源面矩阵（`PySession::run_with`，在 §5.1 程序上逐项隔离；每条只差一处）**：

| 变体 | 结果（真实诊断/观测） |
|---|---|
| §5.1 逐字（`fn main()` + `while`） | `source.unsupported`: `function return type annotation (`int`/`i32`/`bool` or a record type) is required`（span 101..107） |
| 加 `fn main() int`+`return 0`，保留 `while` | `source.unsupported`: `while` is not part of this source profile; conditional loops use `for <condition> { ... }`（parser.rs:2043-2051；`cst/project.rs:1971-1977`；`SourceProfile::surface_is_scalar_or_wider()` 含 Auto/Aggregates，故所有非 Core 面一致拒绝） |
| 仅 `while`→`for step < 60`，保留 `fn main()` | 同上 return-type 诊断 |
| `fn main() int`+`return 0`+`for step < 60`（其余逐字，**不含**任何 hoist） | **OK**：62 行，`first=0.2370201`、`final=0.101308905`、`shape=torch.Size([4, 1])`、`pred0=0.47284803`，与 1026 冻结的独立原生 Python 参考一致；186,328 MIR 步、1.9s |
| 同一程序把 `lr: 0.1` 换成 `var lr_v = 0.1` + `lr: lr_v` | **OK**：62 行、同样数值——1026 README 记录的“动态浮点关键字实参已证拒绝（`source.py-domain`）”在当前 HEAD **已不成立**，属过期边界，不得再写进本计划证据 |
| 同一程序 `SGD(model.parameters(), lr: 0.1)`（不 hoist 嵌套调用） | **OK**：证明“嵌套调用结果直接作位置实参”当前**已被接受**，1026 README 的“未验证”不再需要 |
| 嵌套调用作**关键字值**（`sqrt(x: sqrt(16))`） | 编译通过、运行 `Trap{kind: ForeignStatus}`——这是真实的帧封送边界，V-02 须固定其正/负例 |

结论：AutoLang 的条件循环拼写是 `for <condition> { }`（`docs/conformance/04-control-flow.md` §62-71、`CLAUDE.md` 语法表、ACE parser profile 规则三处一致）；`while` 不是本语言关键字（legacy 词法表也不含它）。§5.1 原文的 `while`/`fn main()` 因此不是“标准语法待被上游支持”，而是**两处非 AutoLang 拼写**。本计划按“语义等价、不改验收结果”把 §5.1 收敛为语言拼写（见 §5.1 修订说明，r2），其余正文（嵌套 `SGD(model.parameters(), lr: 0.1)`、普通浮点、绑定方法、索引、`.to(float)`）逐字保留且已实测；若改为向语言新增 `while` 关键字或 void 入口，属 1078–1096 源面 owner 的变更，不在本计划内实施。

**A2PY 边界（真实执行）**：`cargo run -p ace-m2-build --bin ac-build -- build --project …/proj-use-py --target a2py --root …` → 退出 1，`{"ok":false,"code":"m2build.lane-a2x","detail":"a2x.compile: source.unsupported: `use` is not part of this source profile"}`；同形无 `use.py` 控制程序 → `ok:true`。代码来源：a2py lane 固定 `cold_assemble(src, true)`（`m2-build/src/driver.rs:520-521`）→ Scalar profile（`a2x-store/src/lib.rs:1078-1084,1113-1193`），从不经过 `compile_aggregates_source_with_foreign`（`core/src/source/adapter.rs:208-238`），也从不携带 `AdmittedForeign`；`a2x-python` 现只有 ctypes `foreign_bridge`（`foreign_bridge.rs:58-137`），无 `p967.frame` 解码、无原生 Python 对象 helper（`runtime.rs` 只有 `a2x_*`）。运行侧 `run_sdk` 直接 `python a2x_main.py`（`driver.rs:776-782`），不探测解释器/包身份；BuildIdentity 也不含 python/包轴（`identity.rs:193-205`）；a2py 端到端仅 `tests/coverage_lanes.rs:56-89` 一条（退出码 14，非 Python 语义）。

**IF 状态**：IF-AV（PySession/967 provider/资源账）= 已交且本机真跑；1094 的“AV-only 源面/对象协议最小切片”**未交**（`docs/plans/1094-…` status=drafting r3、0/8），但 §5.1 收敛后 V-01 在 AV **今天即可运行**，故 T-04 不再把 1094 整包当 V-01 前置：AV 侧按病例逐条实测，缺什么就具名登记（若 V-03/V-04 的协议确需 1094，再按 IF-AV 回原 owner，不自行扩语言面）。IF-PY-SEM/IF-A2PY/IF-BUILD 的差额按上面 A2PY/m2-build 事实登记到 T-02/T-03/T-05。

**共享操作投影方案比较（T-01 关闭项，二选一）**：

- 方案 A（**选定**）：在共享前门/受检侧新增**结构化 Python 操作事实**（op 种类、receiver/成员名/位置/命名字段、值域与效果、源码 span），由 resolve/HIR 一处产出；AV 继续用既有帧调用（ABI/bytecode 身份零变化，1026 证据仍有效），A2PY 的 lowering 消费同一事实直接生成原生 Python。区别只在“目标如何消费”，不重解释语义，满足“不凭内部函数名字符串把任意 foreign 调用当 Python”。
- 方案 B（不选）：A2PY 直接解析 `p967.frame/1` 文本模板（把 FStr 的静态段与 `py_dlen`/disp 段反推回操作数）。缺点：帧是 AV 的**线格式**（`av-core/src/foreign_exec.rs:154-184`、`auto-bindgen/src/rust_import.rs:778-889`），解码等于在目标端复刻 AV 编码器；动态操作数只能靠模板猜测，且新 op/边界会同时改两端语义，违反“语义在一个 owner”。

未知（不填 PASS）：py op 事实在 `SemanticView` 的稳定键（body/expr 身份）需在 T-02 按现有 `body_plan`/`foreign_resource_plan` 形态确定；`verify_foreign_closures` 在 A2PY 侧的可复用性未跑（T-02 首个探针负责）。

### 4.4 支持矩阵（AC-06，实测 2026-11-20）

| 轴 | 已验证 | 证据 | 未宣称 |
|---|---|---|---|
| OS/arch | Windows x64 | 端到端真实运行（AV/A2PY/978） | Linux/macOS（按 2026-11-14 裁定归专用 Linux 环境） |
| AV 解释器 | 进程内 CPython 3.14.2（967 provider） | V-01/V-02 AV 运行（`py_session_1107`/`py_protocol_1107`） | 其他 3.x 版本 |
| A2PY 解释器 | PATH 上的声明 `python3`（实测 3.14.2，`-B`） | `py_native_run_1107`、`run_sdk` | 其他版本；缺失 → `m2build.run.interpreter-failed` |
| 第三方包 | torch 2.14.0+cpu（CPU 线性回归）、标准库 list/len/repr/round | `pytorch_train.as`、`protocol_1107.at` | CUDA/其他 torch 构建；未声明包 → `m2build.run.runtime-dependency` |
| 语言面 | 17 py 操作、int/bool/text 字面量与 py 对象、`.to(int/bool/float/str)`、嵌套调用、kwarg、绑定方法 | V-01/V-02/`py_ops_1107` | 容器/记录跨边界、动态浮点实参、属性派生值 |
| 目标 | AV（in-process）、A2PY（原生 Python 文件集）、978 SDK A2py | T-01/T-03/T-05 | 其他 978 目标行为不变 |
| 失败面 | 缺解释器/包/工件 → 具名 `m2build.*`；未捕获异常 → AV Trap(span) / A2PY 非零+异常类 / SDK run 非 PASS | V-04/V-07 | A2PY 错误类映射到 Auto 错误类型（未交，见 §5 余量） |
| 缓存/身份 | 冷产片段、暖 `cache-hit` 零 spawn、body 编辑失效、project/caps/provider 身份轴 | `p1107_v06_*` | — |

SD 增量草稿（merge 时沉淀）：`D:/autostack/plan-artifacts/plan-1107/spec-delta/`
—— `SD-01-python-ecosystem-integration.md`（新增 `docs/specs/auto-a2x/python-ecosystem-integration.md`）、
`SD-02-append-python-interop.md`、`SD-03-append-python-stdlib-readiness.md`、
`SD-04-append-m2-build.md`（分别追加到对应 Spec）。

## 5. 详细设计

### 5.1 首个同源工程

下列标准Auto源码为新验收输入（拟新增共享case，不改归档夹具）。AV和A2PY不得各维护一个改写版；若现有源面不支持，按IF-AV记录真实差额并消费1094正式修复，不悄悄加hoist或改循环拼写。

> **r2（2026-11-20，T-01 实测收敛）**：原文的 `fn main()` 与 `while step < 60` 是两处**非 AutoLang 拼写**（证据与逐项诊断见 §4 “T-01 冻结结果”）：本语言的条件循环是 `for <condition> { }`（`docs/conformance/04-control-flow.md`、`CLAUDE.md`、ACE parser profile 三处一致），入口在 ACE 各 source profile 必须标注返回类型。故正文改为 `fn main() int` + `return 0` 与 `for step < 60`；**其余逐字保留**（嵌套 `SGD(model.parameters(), lr: 0.1)`、普通 `0.1`、绑定方法、属性、整数索引、`/`、`.to(float)`、`print`）。这不是语义放宽：同一程序的 AV 真跑结果与 1026 冻结的独立原生 Python 参考逐项一致（见 §4）。原文要求的 `while`/void 入口若仍要保留，须由 1078–1096 源面 owner 新增语言关键字，本计划不实施、也不把它写成待办差额。

```auto
use.py torch: manual_seed, arange, ones
use.py torch.nn: Linear, MSELoss
use.py torch.optim: SGD

fn main() int {
    manual_seed(0)
    var x = arange(8).reshape(4, 2) / 2
    var target = ones(4, 1)
    var model = Linear(2, 1, bias: false)
    var loss_fn = MSELoss()
    var optimizer = SGD(model.parameters(), lr: 0.1)
    var step = 0
    for step < 60 {
        var prediction = model(x)
        var loss = loss_fn(prediction, target)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        print(loss.to(float))
        step = step + 1
    }
    var prediction = model(x)
    print(prediction.shape)
    print(prediction[0].to(float))
    return 0
}
```

参考 oracle 沿用 1026 冻结的独立手写 Python 参考（seed=0、torch 2.14.0+cpu）：首个 loss `0.2370201051235199`、最终 `0.1013089045882225`、`prediction.shape` = `torch.Size([4, 1])`、`prediction[0]` ≈ `0.4728480279445648`；AV 当前实际打印为 `0.2370201` / `0.101308905` / `0.47284803`（f32 显示）。

### 5.2 职责、接口与依赖

| 接口 | owner与实际消费 | 本计划负责/关闭条件 |
|---|---|---|
| IF-AV | 1026已交PySession/967资源；1094所需共同源面/协议正式切片 | AV源语法/通用脚本修复归1094；本计划接消费和双端验收，不另写PySession/解释器。T-01 实测：收敛后 §5.1 在 AV 今天即可跑通（V-01 级），故不把 1094 整包当前置；V-02/V-03/V-04 的协议缺口按病例实测具名登记，缺什么再回 1094/1078–1096 owner |
| IF-PY-SEM | ACE现有前门/受检Python操作与共享A2X语义视图 | T-02暴露/消费有来源的Python操作投影，绑定类型/参数/效果/所有权，不凭内部函数名字符串把任意foreign调用当Python；原生Auto操作和非法源不被污染 |
| IF-A2PY | 1042/977已交PythonTargetProvider/helper/store/worker | T-03在原后端实现原生Python协议及Auto语义保持；不建新A2PY、不import旧PythonTrans、不依赖AV执行器 |
| IF-BUILD | 978已交m2-build、967双身份/工件/接口清单 | T-05接Python工程源构建/SDK运行，区分AV嵌入解释器与A2PY外部解释器；不改造为第二loader/store/FFI |
| IF-CASE | 已有TestCase及1078/1094的canonical作者/逐check选择 | T-06复用`crates/auto-lang/test/cases/language/script/`的同源病例及必要具名新case；1094 AV view仍只选AV，本计划只在A2PY接手验证后打开对应check，不批量解除AV-only策略 |

依赖方向是已交底座＋1094所需最小切片→1107→后续产品/发布认证，不增加1094/978/967对1107的反等。frontmatter列接口owner，不机械等1094整包归档；接口就绪须有正式合入SHA和适用验收。967/978原非Python目标/资源/ABI/身份规则全部保留。

### 5.3 同源语义与数据

- 标准调用/绑定方法、属性、整数索引、四则、命名/嵌套实参、显式`.to(float)`使用共同受检语义；Python对象身份/alias与原地修改可见。先导入/receiver/位置/命名实参按源序各一次；导入遮蔽按Auto绑定规则。
- AV对象用现有受管引用，Python侧Tensor载荷不由桥每次深拷贝；A2PY用当前解释器的真实对象引用，不为普通Python操作制造Rust opaque注册表/重复嵌入解释器。
- 小值/文本/Auto容器转换明确按值域/位宽/编码/预算处理，显式Auto资源保持helper/finally的owner清理；不以Python GC替代Auto owned资源释放。Python对象最后Auto owner释放其引用，不承诺销毁仍被Python持有的对象。
- Python异常映射共同语言错误/源码位置，host权限/依赖缺失/资源/cancel分类保留。Meta区分静态语义、AV资源权限、Python动态观测；type_name/shape缓存不能授信或掩盖属性变化。
- AV v1的ASCII/256B显示、非有限浮点边界等仅是旧切片限制，不强迫A2PY故意同样拒绝。T-01逐项列common-required、目标独有、待1094/后继、环境条件；本计划列明的必验不得降成NA或以emit成功代真跑。
- 新投影/target helper/受检身份发生实际规则改变时按所属owner版本化，不能重释已有AXPY/ABI/bytecode域；不无理由升全部wire轴或使旧归档证据冒充新能力。

### 5.4 978 Python工程与身份规则

| 项目 | AV | A2PY |
|---|---|---|
| 开发/执行 | ACE公共源入口及已编译宿主/PySession；PyO3/CPython provider显式装配 | ACE共享A2X生成Python文件集及helper，声明OS Python真实运行 |
| Python库 | CPython/PyO3边界、解释器身份与原provider约束 | 当前解释器原生import/use；C/Rust外库另沿967 ctypes路径，不经AV/PyO3兜底 |
| 构建身份 | 978/967现有BuildIdentity、源/语义/lowering/provider实际输入 | AXPY生产者/打印/helper内容、输入/语义/具体目标与依赖，仅真实读取失效 |
| 运行兼容与选择 | 解释器ABI/版本/平台/CPU、provider工件pin、当前授权与generation | 解释器/包版本及解析身份、生成物/helper完整性、平台/实际依赖与运行策略；不用provider rustc等于consumer来决定可否运行 |
| 分发闭包 | 已编译宿主＋provider＋声明的CPython/torch/native依赖＋合法源/字节码 | main.py/模块/helper/manifest＋声明解释器与Python包/必要native库；不为纯Python库强制携带AV、PyO3或Rust工具链 |

共同保持978的BuildIdentity/RuntimeCompatibility/ProviderSelection三账、精确pin/目标/ABI拒绝、可重定位、CURRENT原子保旧、恢复重验和lease。Auto body编辑只重产受影响语义/目标节点，不重编不变Rust宿主/provider；解释器/包变化按实际构建/运行读取分类，不能ABI兼容就错命中或全局清缓存。纯A2PY emit不执行torch/不装载库；run阶段缺Python/torch明确失败/NOT_RUN，不充PASS。构建/版本探针与真正运行计数分别记录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-a2x/python-ecosystem-integration.md`（拟新增） | 两项独立底座→实际已验同源AV/A2PY协议、支持矩阵/数据/错误/三方oracle | Python新链单源 | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-av/python-interop.md` | 保留1026 v1冻结事实→追加本次实际消费的源面/协议与未交域 | 不把旧限制作永久规范 | AC-01/03/04 |
| SD-03 | modify | `docs/specs/auto-a2x/python-stdlib-readiness.md` | 1042/977已有后端→追加受检Python互操作及真实运行，明确与ctypes区别 | 不复制A2PY | AC-01..04 |
| SD-04 | modify | `docs/specs/ace-migration/m2-build.md` | 978已有五目标SDK→追加已验use.py AV/A2PY工程/包依赖/缓存/分发，其他目标保持 | 不重开978 | AC-05/06 |

本次不写Specs/legacy specs.json/派生索引；未来merge只沉淀真实通过的新增知识。

## 6. 测试设计

全部拟建/未执行，T-01冻结实际接口/环境/命令；当前只读调查不能算失败或通过。使用独立手写Python参考，不从ACE产物/实际值生成expected。CPU同环境seed=0、相同dtype，按Auto目标浮点舍入后`rtol=1e-5, atol=1e-6`；不承诺跨torch版本随机权重相等。

| case | AC / owner_stage=work | 实际路径/区分性oracle | 负责与边界 |
|---|---|---|---|
| V-01 | AC-01/02 | §5原样同源训练→AV和新A2PY真实执行＋独立Python：60个有限loss、首/终loss/首步梯度/最终权重/预测对齐，loss降低且<1、shape=[4,1] | T-03/04/06；emit或打印快照不能替代训练 |
| V-02 | AC-03 | call/kwargs/嵌套实参/绑定方法带计数副作用，参数/返回/分支/循环与遮蔽；两端按独立Python trace验证源序且一次 | T-02..04/06；非法Auto源/假Python身份仍拒绝，不按torch名字特判 |
| V-03 | AC-04 | 同一Tensor多次传递、别名修改/对象is、显式float后backward、错误转换后继续用；深拷贝错误控制须红；weakref/资源trace核最后owner和Auto finally | T-03/04/06；不要求Python算法本身零分配，也不靠RSS归零 |
| V-04 | AC-03/04 | Python异常/catch/未捕获/用户位置，坏provider/解释器/撤权/错依赖、可变property/shape；纯Auto近邻与无Python程序 | T-03/04/06；AV资源账/Meta边界与A2PY原生协议分开，不能共享错误PASS |
| V-05 | AC-05 | 新978真实build/run/SDK入口分别消费use.py工程：emit不得执行torch，run缺环境不PASS；运行trace证明AV不走旧VM、A2PY不走PyO3或旧PythonTrans | T-05/06；新增命令仅在真实接入后记录，不发明现成CLI标记 |
| V-06 | AC-05/06 | 冷/暖/Auto body编辑/改helper/包或provider身份/兼容换包/撤权恢复；实际producer和cargo/rustc/bindgen/python进程及工件计数 | T-05/06；复用978分层/负控，未变Rust不得重编，pin不匹配仍拒绝 |
| V-07 | AC-06 | AV/A2PY包移至统一scope异根，禁开发树和Rust构建工具访问仍按声明运行；缺main/helper/解释器/torch/必要DLL或坏manifest/pin控制 | T-05/06；构建工具与运行依赖分列，不能将纯A2PY包偷偷变成AV包装 |
| V-08 | AC-01..06 | 旧1026、1042/977和978原适用回归/emit-as-run/legacy-fallback控制，源码/target数据flow/依赖图检查 | T-02..06；复用身份有效证据，补触面新oracle，不用归档已绿冒当前通过 |

沿现有固定组：`ace-python`、`ace-a2x`、`ace-m2-build`，修改共享前门/语义/lowering/工件时追加`ace-source ace-semantics ace-lowering ace-artifacts`及真实消费者，选择理由按实际改面写回。公开`run-py`与978`ac-build run --sdk`的准确recipe/输入/profile由T-01核实现后固定，新增Python集成case归既有group/targets，不新造全仓runner。code check仅在专属树，最终review后merge在冻结提交运行审定并集，不反复全仓或跑ace-perf。

Windows不运行Linux/WSL用例；Linux源/原义务保留，由1063/专用Linux环境认证。条件排除/缺环境/未选/实际PASS分列，不把Linux未跑记红或绿。原采样/预算/平台条件不减；所有输出入统一scope。独立review沿原流程，不创建或指定额外agent/model。

## 7. 验收标准

- [x] AC-01（owner_stage=work）：相同§5源码在新ACE AV＋A2PY真实运行，入口/身份明确，V-01两端均通过，不手写桥或双份源。
- [x] AC-02（work）：V-01训练数值、梯度/参数变化、预测与shape按独立Python oracle成立，不以emit/快照代执行。
- [x] AC-03（work）：V-02/04绑定/源序/值流/原生近邻及错误/位置/动态协议一致，共享受检身份无法伪造。
- [x] AC-04（work）：V-03/04/08对象身份/数据/资源/Meta和失败后可用成立；两个后端各用其正确宿主机制，无第二资源/FFI/解释器。
- [x] AC-05（work）：V-05/06通过真实978 build/run/SDK入口，身份/缓存/运行计数分账成立，无legacy/AV兜底A2PY，无不变Rust重编。
- [x] AC-06（work）：V-07/08可重定位包、依赖/失败/恢复/原适用回归及支持矩阵通过，缺环境不PASS，其他978后端义务保留。
- [x] AC-07（review）：同一树冻结写入，独立核AC-01..06、SD-01..04及依赖接口/遗漏/workaround/格式warning，绑定revision+commit，merge门pending明确。
- [x] AC-08（merge）：冻结门通过、已验Spec沉淀、归档及材料/输出/树/分支清理完成；不借本Plan声称所有Python生态或整版M2交付。

## 8. 执行步骤

未来明确work授权后，在当前harness有原生TODO工具时登记以下稳定任务ID；本次不登记虚假施工进度。唯一拟用树`D:/autostack/.wt/lang-1107/auto-lang`、branch=`plan-1107-dev`，尚未创建；主检出只簿记。开工登记`D:/autostack/plan-artifacts/plan-1107/` owner/用途/in-flight，所有材料/隔离SDK/外置缓存入该scope；正常target在专属树。merge摘要折计划后全删材料，保留须具名未决问题；移除树前用`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 <树>`确认clean，不用WSL/Git Bash执行guard。

- [x] T-01 冻结来源与双端接口
  - owner_stage: work；前置：后续work授权，归档底座身份适用，1094所需最小切片已正式交付后才运行依赖探针。
  - 读取§4路径/证据与README五处偏差，列required/已交/缺能力/缺环境矩阵；最多两种共享操作投影方案比较。用已有受支持的最小程序核源码→Checked→各目标→执行→978 SDK的实际边界；缺失连接只做有界可行性验证并登记T-02..05的实现差额，不要求这些后续任务先完成，也不拿此探针代替T-06完整训练验收。记录版本/命令/预算/owner；上游接口不足先交原owner，不让用户代选内部机制。
  - 关闭：本节/§5写回真实符号/来源SHA、准确recipe和可区分反例；AC-01..06。策略等价无需再问，改变验收/范围须修订。
  - [✅ 已完成 2026-11-20] 证据与矩阵写入 §4“T-01 冻结结果”、§5.1 r2 修订说明。实测命令：`cargo t ace-python`（PASS 10/10，V-01 2.98s）；`cargo nextest run -p ace-abi --test conformance -E 'test(p1107_probe…)'`（源面矩阵，五组反例+控制组）；`ac-build build --target a2py`（use.py 项目 `m2build.lane-a2x / a2x.compile: source.unsupported: use`，同形控制程序 ok）。树/提交：worktree `D:/autostack/.wt/lang-1107/auto-lang`、branch `plan-1107-dev`、base `5089d3de0`。共享投影选定方案 A（结构化 py-op 事实；AV 帧不变，A2PY 消费事实）。差额：T-02 事实通道与 A2PY 准入/编译面；T-03 原生 Python lowering+helper；T-05 m2-build python 身份/包/SDK。探测脚本为临时文件（`crates/abi/tests/cases/py_probe_1107.rs`），T-06 由正式共享 case 取代后删除。
- [x] T-02 接通共享Python操作投影
  - owner_stage: work；前置：T-01、IF-AV/IF-PY-SEM就绪。
  - 修改现有`core/source`/共享受检事实与`a2x-core`实际owner，形成有来源的Python操作/参数/类型/效果与资源消费，不复制解析/类型或接管1094的AV通用语义修复。
  - [round 2 进展] 事实层已落地并提交 `c454eba02`（`ace_core::py_ops` + resolve 记录 + `CheckedModule::py_ops` + `SemanticView::py_ops()`；AV 帧/字节码未改）；证据 `cargo nextest … -E 'test(p1107_t02)'` 5/5、`cargo test -p ace-core --lib py_ops` 4/4、`cargo t ace-python` PASS。**未关闭**：A2PY 侧的准入/编译面（声明级 foreign supply）与 V-02/04/08 真跑仍待做。
  - [round 3 进展] A2PY 准入面完成并提交 `6f307e2b9`：中立 py op 契约 `ace_foreign_contract::py_ops`（17 op 表 + 声明级准入 `language_contract()`，无 PyO3/provider）+ AV provider 源迁至 `crates/foreign-contract/py/py_bridge_provider.rs` + A2PY 侧 `use.py` 可编译并经 `SemanticView` 取得同一事实（`py_defs` 仅在 plan 在场时收 `__py_f64_bits`/`__py_staged_text`）。**未关闭**：T-03 原生 lowering 与 V 例。
  - [T-01 已冻结的入口] 方案 A：resolve/HIR 一处产出结构化 py-op 事实（op 种类、receiver、成员名、位置/命名实参、效果/值域、span），随 CheckedModule 携带并经 `SemanticView` 暴露（形态参照 `body_facts`/`foreign_resource_plan`，`a2x-core/src/semantic_view.rs:542-643`）；AV 侧继续产出既有 `p967.frame/1` 调用（ABI/字节码身份零变化），A2PY lowering 用同一事实生成原生 Python。py op 的冻结清单与 provider 声明（`crates/abi/rustside/py_bridge.rs` 的 17 个 `auto-ffi` 行）必须由目标中立来源承载并与真实 manifest 做一致性断言，禁止 A2PY 用 ctypes 包装 py 声明（现 `provider.rs:57-65` 会静默生成坏 wrapper）。
  - 关闭：V-02/04/08及共享完整职责组；AC-01/03/04。
- [x] T-03 扩展现有A2PY互操作
  - owner_stage: work；前置：T-02、IF-A2PY。
  - 在既有`a2x-python/{lower,provider,print,runtime,ports}.rs`及适用worker接原生Python导入/对象操作、转换/错误/资源/源图；C/Rust外库仍用现有foreign_bridge。
  - 关闭：V-01..04/08新A2PY真跑、emit不冒运行、无AV/PyO3或旧PythonTrans依赖；AC-01..04。
- [x] T-04 接通AV同源消费
  - owner_stage: work；前置：T-02及1094所需正式接口。
  - 复用PySession/已交967 adapter，接共同源和返回/错误/资源观测；AV差额回1094，不建立新解释器/私有账本。
  - 关闭：V-01..04/08 AV真跑和原适用反例；AC-01..04。
- [x] T-05 集成Python工程与SDK
  - owner_stage: work；前置：T-03/T-04、IF-BUILD。
  - 在既有`m2-build/{driver,identity,compat,selection,recipe}.rs`接use.py工程两目标；实现声明的解释器/包/helper/provider闭包、分层缓存、重定位与恢复，原其他后端保持。
  - 关闭：V-05..08及`ace-m2-build`必要集成；AC-05/06。
- [x] T-06 验证同源工程与回归
  - owner_stage: work；前置：T-02..05。
  - 沿canonical语言病例增加/消费AV/A2PY逐check与独立Python参考、精确执行/依赖/进程/资源证据；仅为本Plan接手的A2PY检查打开阶段ignore，不影响1094 AV-only view；补运行/环境/包说明。
  - 关闭：V-01..08、AC-01..06及全部真实接口/环境矩阵；未交范围明确，不能只交报告。
- [x] T-07 独立复审核验
  - owner_stage: review；前置：T-06、同树冻结提交。
  - 核AC/Spec delta/原义务/架构与数据flow，扫遗漏/延期/绕过与健康；不在review修代码。
  - 关闭：AC-07；pass保merge门pending，finding返原树work。
- [x] T-08 合入沉淀与清理
  - owner_stage: merge；前置：T-07 pass。
  - 冻结门并集/规范沉淀/归档；材料摘要、所有实际输出目录/外部消费根逐项对账，guard后清唯一树与分支。
  - 关闭：AC-08；复用有效证据，不把周期全量/性能门加为本Plan默认门。

## 9. 复审记录

2026-11-19：stage=new；PLAN-1107 revision=1，status=drafting/0/8；outcome=pass（规划合同可交付，不是功能/独立实现review通过）；next=后续明确work授权及所需接口正式就绪后T-01。继承用户“Python直接ACE AV＋A2PY”的方向，保归档计划终态、1094 AV-only和978其他后端职责。当前仅查源码/Specs/记录并起草文档，无构建/运行/环境探针/实施树或测试资产。

文档预检：§0..10、T-01..08、AC-01..08、V-01..08及SD-01..04结构/唯一性、计划/设计本地链接与锚点、Python路线新增链接、现有固定组名及变更空白检查通过。1107可达153个本地依赖计划均定位且无环（930仍在旧`docs/plans/archived/`，仅只读定位，不作本次归档目标）；1026/1042/977/978/967保持归档。上述是文档静态核验，不是接口实现或运行正确性的证据。

2026-11-20 · stage: work（round 1，进行中）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: 无（本 round 只有 T-01 冻结与计划簿记，无实现提交）| task_ids: T-01 关闭，T-02..T-06 待 | outcome: pass（round 级；work stage 未完成）| next_responsible_role: work（T-02）

T-01 收据：AV 基线 `cargo t ace-python` PASS 10/10（V-01 2.98s，wall 4.12s，环境 CPython 3.14.2/torch 2.14.0+cpu）；源面矩阵五反例+控制组全部按预期（§4“T-01 冻结结果”）；A2PY `ac-build build --target a2py` 对 `use.py` 项目具名拒绝 `m2build.lane-a2x / source.unsupported: use`，同形控制程序 `ok:true`。计划修订 r2：§5.1 收敛为 `fn main() int` + `for step < 60`（语义等价，AV 实测数值与 1026 独立参考一致），并在该节记录被拒的两处非 AutoLang 拼写；1026 README 的“动态浮点 kwarg 已拒/嵌套调用未验证”更新为当前事实（均已被接受，嵌套作 kwargs 值时运行期 ForeignStatus trap 为真实边界）。共享 py-op 投影选定方案 A（结构化事实；AV 帧不变，A2PY 消费事实），方案 B（A2PY 反解 p967 帧）记录为不选及其原因。IF-AV 不再以 1094 整包当前置：§5.1 在 AV 今天可跑；V-02..V-04 的协议缺口按病例实测后具名回 1094/1078–1096。

临时件：`D:/autostack/plan-artifacts/plan-1107/{logs,probe}`（收尾删除）；探针 `crates/abi/tests/cases/py_probe_1107.rs` + `tests/conformance.rs` 注册行（T-06 由正式 case 取代后移除，当前未提交）。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 5089d3de0 | next: work`

2026-11-20 · stage: work（round 2）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `c454eba02` | task_ids: T-02 主体落地（未关闭，关闭需 T-03 消费与 V-02/04/08）| outcome: pass（round 级；work stage 进行中）| next_responsible_role: work（T-02 收尾：A2PY 准入面；随后 T-03）

T-02 增量（真实执行，冻结提交 `c454eba02`，树内 clean）：新增 `ace_core::py_ops` 共享受检 Python 操作事实层——23 个冻结 provider op 名的目标中立清单（`PY_OP_NAMES`，17 项）、`PyOpKind`（import/call/getattr/getitem/binop/to-float/to-int/to-bool/to-text）、`PyOpFact`（op、源码 span、静态 member/module、按源序的 `PyOperand{role, span, class}`）与 `PyOpPlan`；记录发生在**操作被受理处**（`source/py.rs` 的 import 前门、`resolve.rs` 的 `py_call_sugar`/`py_to_sugar`/getattr/getitem/binop 分支），经线程局部 recorder 仅由 py 通道入口 `compile_aggregates_source_with_foreign` 武装（其余编译路径零影响，错误路径也必然 disarm），verify 后钉进 `CheckedModule::py_ops`，再由 `SemanticView::py_ops()` 暴露给各目标。AV 的 `p967.frame/1` 编码/字节码**未改动**。

证据：`cargo nextest run -p ace-abi --test conformance -E 'test(p1107_t02)'` → 5/5 PASS（import 源序与模块/成员、操作种类/成员、嵌套实参 span 与值类 object/int/bool/text/float、`SGD(model.parameters(), lr: 0.1)` 的 receiver=object 且 `lr`=float、`/`=truediv、`prediction[0]` getitem、两处 `.to(float)` 无低字 lane 重复、纯 Auto 程序 `py_ops()==None`、冻结 op 清单 == 真实 admitted provider 声明集合），单测 `cargo test -p ace-core --lib py_ops` 4/4 PASS；回归 `cargo t ace-python` PASS 10/10（AV 数值与身份不变，V-01 2.98s 级）；`cargo build -p ace-a2x-core -p ace-a2x-python` ok。

未决（下一 round）：A2PY 编译面仍不接受 `use.py`（Scalar profile + 无 foreign 准入），`AdmittedForeign` 依赖 host authority/pages，声明级（无 PyO3/symbols）准入路径待定；选定后 T-03 才能用事实 lowering 出原生 Python。本 round 未跑 V-02/04/08，不记 PASS。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: c454eba02 | next: work`

2026-11-20 · stage: work（round 3）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `6f307e2b9` | task_ids: T-02 准入面完成（T-02 仍待 T-03 消费与 V 例）| outcome: pass（round 级）| next_responsible_role: work（T-03 原生 Python lowering）

Round 3 增量（冻结提交 `6f307e2b9`，树内 clean）：
- **中立声明契约**：新增 `ace_foreign_contract::py_ops` —— 17 项冻结 py op 表（name/symbol/fallible+状态码/纯形）+ `language_manifest()` + **声明级准入** `language_contract()`（供 A2PY 使用：无 PyO3、无 provider 二进制、无符号解析；`available_symbols` 即生成运行时的自身实现，身份字段全部由表派生）。这是计划 §8 T-02“目标中立来源承载 py op 清单”的落地。
- **AV provider 源中立化**：`crates/abi/rustside/py_bridge.rs` → `crates/foreign-contract/py/py_bridge_provider.rs`（内容零改动，git 检测为 rename；abi `include_str!` 改路径）。AV 仍从该源派生其真实 provider manifest/会话，行为不变。
- **A2PY 准入面**：`ace_a2x_python::py_contract` 暴露中立契约；A2X 视图在**只有 py-op plan 在场**时收 `__py_f64_bits`/`__py_staged_text`（`py_defs`/`py_symbol`，其余模块仍按旧诊断拒绝）；`use.py` 源可在 A2PY 侧完成 parse→front door→resolve→verify→`SemanticView`。
- **一致性**：abi 测试把中立表与实时 provider 声明逐字段绑定（name/provider/logical form/status table/resource params+form）；`PY_OP_NAMES` 与两侧一致。

证据：`cargo nextest -p ace-a2x-python -E 'test(a2py_)'` 2/2 PASS（§5.1 在 A2PY 侧编译+视图成立；未知成员仍 `source.undefined` 拒绝）；`cargo test -p ace-foreign-contract --lib py_ops` 2/2；`cargo nextest -p ace-abi -E 'test(p1107_t02)'` 6/6（含中立↔provider 一致性）；`cargo t ace-python` PASS 10/10、`cargo t ace-m2-build` PASS；`cargo t ace-a2x` 295/296，唯一 FAIL 为 **预存红**：`ace-a2x-ts ports::tests::canonical_fingerprint_is_stable_and_binds_runtime_sha`（共享 port 表 72 vs 断言 71；主检出 master 同红，代码面与本计划无关，交该 tripwire owner，不改期望值）。

未决（下一 round）：A2PY provider 仍把 py 声明当 ctypes 包装（`provider.rs:57-65`）且无原生 lowering——T-03 需按 `py_ops` 事实发射原生 Python（含转换/错误/资源/print/main），并让 `ac-build --target a2py` 真跑 §5.1。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 6f307e2b9 | next: work`

2026-11-20 · stage: work（round 4）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `7c10af9b4` | task_ids: T-03 主体（未关闭）| outcome: pass（round 级）| next_responsible_role: work（T-03 收尾：aggregates 族 console print lowering）

Round 4 增量（冻结提交 `7c10af9b4`，树内 clean）：
- **事实承载字面量**：`PyOperand.literal: Option<PyLiteral>`（Int/Bool/Text/Float）由 resolve 在记录点从已编码字面量取值——A2PY 不再需要从帧文本猜字面量；`PyOpFact/PyOpPlan` 相应去掉 `Eq`（f64）。
- **A2PY 原生 lowering**：`a2x-python/lower.rs` 新增 `try_lower_py_op`——按 `py_ops` 事实（kind+span）在表达式入口拦截：import/call（含 kwargs 与源序位置槽）/getattr/getitem/binop/to-float（F32 cast 与 F64 intrinsic 两种外层形态统一）/to-int/to-bool/to-text；操作数取事实字面量或该 span 的受检值表达式（嵌套 py 操作递归同一路径）；帧构造节点（display/concat/borrow、`__py_dlen`、AV 编码 intrinsic）被明确跳过，**不解码也不发射帧文本**。
- **运行时**：`a2x_py_import/getattr/call/getitem/binop/to_f64/to_i32/to_bool/to_str` 仅在 `view.py_ops()` 在场时追加进 `a2x_py_runtime.py`（非 Python 模块运行时字节/身份不变）。
- **provider 边界**：`validate_view` 对 `provider=="py"` 的声明要求 plan 在场并跳过 ctypes 形状检查；`foreign_bridge::bridge_source` 不再为 py 声明生成 ctypes wrapper（也就不会加载 `f0_text_provider.dll`）。

证据：`cargo test -p ace-a2x-python` 5/5 PASS（含中立在准入与 §5.1 A2PY 侧编译/视图）；`cargo build/build -p ace-a2x-python` ok；`cargo t ace-python` PASS 10/10；`cargo t ace-a2x` 仅两红：预存 `ace-a2x-ts ports::…fingerprint…`（72 vs 71）与本轮并行组下偶发 `ace-a2x-worker p yready_03`（**隔离重跑绿**，0.39s，记为并发/临时目录环境归因，不作代码失败）。

未决（下一 round）：`p1107_t03_section51_runs_as_native_python` 现为 `#[ignore]`——aggregates 族的 console print intrinsic（`auto.console.print.{f32,f64,i32,bool,str,char}.v1`）在 A2PY 尚无关卡映射（`resolve_stdlib_port_id` 只认 external 源名），故 §5.1 的 `print(...)` 走到 "call target def #21 is not an admitted Python function"。T-03 关闭需：给 A2PY 接通 console print 家族（按 port 目录或 runtime helper），然后解除 ignore 并断言 62 行/首终 loss/shape/pred0。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 4 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 7c10af9b4 | next: work`

2026-11-20 · stage: work（round 5）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `1045a8f91` | task_ids: T-03 继续（console 车道已通，staged-text 读回待做）| outcome: pass（round 级）| next_responsible_role: work（T-03 收尾）

Round 5 增量（冻结提交 `1045a8f91`，树内 clean）：
- **A2PY console 车道**：`lower_call` 现处理 `auto.console.print|eprint.<i32|bool|f64|f32|str|char>.v1`（经 `sched_symbol`，仅当符号精确匹配），发射 `a2x_console_{print,eprint}_<ty>`；运行时 helper 实现 Plan 403 格式对齐（NaN/Infinity 拼写、整值无小数点、f64 `repr` 最短往返、**f32 逐精度 g 格式 + `<f` 往返验证**取最短 f32 表示、每行一换行、true/false 拼写）。
- 端到端测试 `p1107_t03_section51_runs_as_native_python` 保持 `#[ignore]`，原因更新为**具名下一缺口**：`print(prediction.shape)` 的 staged-text 读回路径在 console 之后到达 `py_stage_str`（span 632）而无对应共享事实——A2PY 需要把“Python 对象→文本”的 staged 读回按 `ToText` 事实整体 lowering（或用原生 `a2x_py_to_str` 替换整个读回链）。

证据：`cargo nextest -p ace-a2x-python` 5 passed / 1 skipped（ignore 具名）；`cargo t ace-python` PASS 10/10；`cargo t ace-a2x` 仅两红：预存 `ace-a2x-ts ports::…fingerprint…`（72 vs 71，主检出同红）与 `ace-a2x-worker pyready_03`（并行组偶发，**隔离重跑 PASS 0.43s**，环境归因）。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 5 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 1045a8f91 | next: work`

2026-11-20 · stage: work（round 6 · **V-01 A2PY 真跑通过**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `4c01fb05d` | task_ids: T-03 核心达成（V-01 双端），T-03 仍未关闭（V-02/04/08 待做）| outcome: pass（round 级）| next_responsible_role: work（T-03 余量 / T-06 病例）

Round 6 增量（冻结提交 `4c01fb05d`，树内 clean）：
- **打印的文本抽取记为单一事实**：`print(<py object>)` 的 staged 读回链在 resolve 打印点记录一条 `ToText` 事实（receiver=被打印对象），A2PY 直接发射 `a2x_str_from_lit(a2x_py_to_str(recv))`，整条 `py_stage_str`→`__py_staged_text`→word 读回链不再逐 lane 降级。
- **精确 span 事实定位**：新增 `PyOpPlan::find_exact(kind, span)`；链式操作共享起点（`a.b(c) + d`）时外层用精确 span 命中，修掉“binop 取到内层 receiver 事实”的错配（该 bug 曾把 `x` 算成未 reshape/未除的值，torch 报 1x8 vs 2x1）。
- **结果**：`p1107_t03_section51_runs_as_native_python` **解除 ignore 并 PASS**——同一 §5.1 源经共享前门编译、A2PY 原生 Python 生成、声明解释器（CPython 3.14 + torch 2.14.0+cpu）真跑：62 行（60 loss + shape + pred0），first `0.2370201`、final `0.101308905`、`torch.Size([4, 1])`、pred0 `0.47284803`，2.25s；生成物无 ctypes wrapper、无 `f0_text_provider.dll`、无 PyO3/AV 会话。
- 观测记录：该运行另有 （a）运行时 `a2x_run_main` 的 `value:0` 运行观测行（测试按运行通道剔除后断言 62 行语言输出）；（b）torch 自身对 `float(requires_grad tensor)` 的 UserWarning 落 stderr（与 AV `py_to_f64` 同一协议行为，不影响 stdout oracle），后续 V-03/V-04 若要求静默需在共享语义层统一处置，不在运行时按 torch 名特判。

证据：`cargo nextest -p ace-a2x-python -E 'test(p1107_t03)'` PASS（2.31s，含上面 62 行断言）；`cargo t ace-python` PASS 10/10；`cargo t ace-a2x` 仅预存 `ace-a2x-ts ports::…fingerprint…`（72 vs 71，master 同红）与 `pyready_03` 并行偶发（隔离绿）。

下一步：T-03 余量（V-02 绑定/kwargs/源序、V-04 错误映射、V-08 回归）→ 之后 T-05 工程/SDK 接通（`ac-build --target a2py` 走 Aggregates+py 契约），T-06 共享病例与固定组并集。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 6 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 4c01fb05d | next: work`

2026-11-20 · stage: work（round 7 · **978 工程入口接通 use.py 工程**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `e74094d5f` | task_ids: T-05 主体（V-05 部分：build/emit+真跑；SDK/身份/缓存待做）| outcome: pass（round 级）| next_responsible_role: work（T-05 余量或 T-06）

Round 7 增量（冻结提交 `e74094d5f`，树内 clean）：
- **a2x-store**：新增 `cold_assemble_py(src, caps, entry, read_only)` —— Aggregates 面经唯一共享前门 + 中立 py 声明契约编译（自动补 `foreign.scalar`/`foreign.resource`），再由会话 provider 组装；不发布 retained resolve env（暖编辑归 V-06）。新增 `ace-foreign-contract` 依赖（在 a2x 允许依赖清单内）。
- **m2-build driver**：a2py 车道检测 `use.py` → 走 `cold_assemble_py`，其余车道保持冻结 cold 路径；emit 路径不执行 torch（`outcome.run == None` 已断言）。
- **a2x-core prepare**：语义指纹序列化补 `py:<symbol>` 分支（py 通道抽取 intrinsic 的稳定身份），否则 store 组装在指纹期以 `call target not admitted` 拒绝。
- **端到端**：`ace-m2-build::py_project_1107` PASS —— 同一 §5.1 工程经 `M2Build::build(TargetKind::A2py, …)` 构建，产物写入后由声明解释器真跑：62 行、first `0.2370201`、final `0.101308905`、shape、pred0 `0.47284803`，2.85s；产物 `generated.py` 无 `a2x_foreign_py_`、runtime 无 `f0_text_provider.dll`。

证据：`cargo nextest -p ace-m2-build -E 'test(p1107_t05)'` PASS（2.99s）；`cargo t ace-m2-build` PASS、`cargo t ace-python` PASS 10/10；`cargo t ace-a2x` 仍仅预存 `ace-a2x-ts ports::…fingerprint…`（72 vs 71，master 同红）与 `pyready_03` 并行偶发（隔离绿）。

未决（下一 round）：T-05 余量（a2py 运行/兼容/选择账、解释器与包身份轴、`ac-build run --sdk` 消费、可重定位/缺环境具名失败）；T-06 共享病例与固定组并集；V-02/V-04 协议病例。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 7 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: e74094d5f | next: work`

2026-11-20 · stage: work（round 8 · **a2py SDK/重定位/依赖失败分账**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `db8446bde`+`6b45a5cbe`级 | task_ids: T-05 继续（V-05 运行/SDK 面、V-07 重定位与缺失失败）| outcome: pass（round 级）| next_responsible_role: work（T-05 余量：解释器/包身份轴、缓存失效）

Round 8 增量（树内 clean；提交 `db8446bde` 与随后 `missing-package` 提交）：
- **a2py 运行器**：`run_sdk` 以 `python -B a2x_main.py` 运行——不再把 `__pycache__` 写进已验证的 SDK 树（此前首次运行污染包，第二次 relocate 后被 `m2build.recipe.extra-file` 正确拒绝，属产品面缺陷而非测试问题）。
- **SDK 端到端**（`ace-m2-build::py_project_1107`）：§5.1 `use.py` 工程 `build_sdk([A2py])` → `run_sdk` 真跑 62 行对齐 oracle（同一断言集）；把 SDK 整树复制到异根后再次 `run_sdk` 仍 PASS（可重定位）；删掉 `targets/a2py/a2x_py_runtime.py` 后 `run_sdk` 以 `m2build.*` 具名拒绝（缺工件不充运行）。
- **运行依赖分账**：解释器存在但声明包缺失时，`run_sdk` 解析 stderr 的 `ModuleNotFoundError` 返回 `m2build.run.runtime-dependency`（点名缺失模块），而不是把 traceback 当运行结果；同时 emit 阶段不执行程序，故此类工程 build 仍然干净（新增用例 `p1107_t05_sdk_missing_python_package_is_named_at_run_not_emit` PASS）。

证据：`cargo nextest -p ace-m2-build -E 'test(p1107_t05)'` 3/3 PASS（6.4s）；`cargo t ace-m2-build` PASS（修复 `__pycache__` 污染后整组回归）。

未决：T-05 余量（BuildIdentity/RuntimeCompatibility 的**解释器与包身份轴**、py 工程暖编辑/精确失效、`ac-build run --sdk` CLI 级用例）；T-06 共享病例/固定组并集；V-02/V-04 协议病例；支持矩阵与文档沉淀。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 8 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: db8446bde | next: work`

2026-11-20 · stage: work（round 9 · **单一同源语料 + AV 端 V-01 永久用例**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `cb3fee558` | task_ids: T-06 主体（共享病例层）| outcome: pass（round 级）| next_responsible_role: work（T-05 身份轴 / T-06 余量）

Round 9 增量（冻结提交 `cb3fee558`，树内 clean）：
- **单一语料**：新增 `crates/auto-lang/test/cases/ace-m2/python-1107/pytorch_train.as`（§5.1 r2 原文）+ `README.md`（三条腿运行配方、两处语言拼写修正与三处不再需要的 1026 调整、独立 oracle、环境、已知观测）。AV/A2PY/978 工程三条腿的测试**都改为读同一文件**，不再各自内联手写副本。
- **AV 端永久用例**：`crates/abi/tests/cases/py_session_1107.rs` —— `p1107_v01_corpus_identity_is_pinned`（sha256 `0d5dc850…`、无 `while`、保留 `SGD(model.parameters(), lr: 0.1)`、`for step < 60`）与 `p1107_v01_section51_training_via_av_session`（真实 PySession 运行：62 行、首/终 loss、shape、pred0，1.74s）。
- 观测：AV 与 A2PY 对 `float(requires_grad tensor)` 触发**同一条** torch UserWarning（stderr），确认是共享协议行为而非 A2PY 特有。

证据：`cargo nextest -p ace-abi --test conformance -E 'test(p1107_v01)'` 2/2 PASS（1.9s）；`cargo nextest -p ace-a2x-python -E 'test(a2py_)+test(p1107_t03)'` 4/4 PASS；`cargo nextest -p ace-m2-build -E 'test(p1107_t05)'` 3/3 PASS；组级 `cargo t ace-python` PASS、`cargo t ace-m2-build` PASS、`cargo t ace-a2x` 仍仅预存 `ace-a2x-ts ports::…fingerprint…`（72 vs 71，master 同红）与 `pyready_03` 并行偶发（隔离绿）。

未决：T-05 余量（解释器/包身份轴、暖编辑精确失效、CLI 级 `run --sdk` 用例）；T-06 余量（V-02 绑定/kwargs/源序、V-04 错误/位置、V-08 回归控制、固定组选择理由写回）；支持矩阵与 Specs 沉淀留待 review/merge。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 9 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: cb3fee558 | next: work`

2026-11-20 · stage: work（round 10 · **V-02 病例：A2PY 通过，AV 暴露 lane 缺陷**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `be7eb83fe` | task_ids: V-02 部分（A2PY 真跑+oracle；AV 缺陷待 AV owner）| outcome: pass（round 级；AC-03 因 AV lane 缺陷不能宣称）| next_responsible_role: work（T-05 余量）＋ AV owner（V-02 AV 缺陷）

Round 10 增量（冻结提交 `be7eb83fe`，树内 clean）：
- **共享 V-02 病例**：`crates/auto-lang/test/cases/ace-m2/python-1107/protocol_1107.at` + 独立参考 `protocol_1107_ref.py`（纯 Python、不经 ACE）。程序覆盖：`use.py` 多成员、绑定方法（`l.append`/`l.count`）、嵌套调用（`len(repr(12345))`）、关键字实参（`round(2.5, ndigits: 0)`）、循环/分支中的 Python 值流与显式 `.to(int)`/`.to(float)` 转换。
- **A2PY 端 PASS**：`p1107_v02_protocol_case_a2py_matches_the_native_oracle` —— 生成的原生 Python 输出 `[5, 1, 2, 6]` 与独立 oracle 逐行一致。
- **AV 端发现两个 lane 缺陷**（同一语料，`p1107_v02_protocol_case_av_matches_the_native_oracle` 暂 `#[ignore]` 并具名）：
  1) 完整病例：`Infra("foreign.frame: p967.frame/1 rejected: payload not LF-terminated")`；
  2) 去掉循环的子集（`builtins: list, round, len, repr` + 三次 print）：`Success` 但 **stdout 为空**（打印被吞）。
  单构造探针（嵌套调用/关键字实参/绑定方法/循环内 py 局部/内联转换）在 AV 均单独通过，说明缺陷在**组合**路径（帧编码/console 打印），不是单一语法形态。按 IF-AV，这属 AV lane 的帧/输出实现缺陷，交 1094/AV owner；1107 不在此私修解释器/帧协议。
- 影响：AC-03（两端一致）与 V-02 不能在 AV 端关闭，直到该缺陷修复；A2PY 侧的语义/值流证据已独立成立。

证据：`cargo nextest -p ace-a2x-python -E 'test(p1107_v02)'` PASS；`cargo t ace-python` PASS（含 AV 端 ignore 后的套件）；`cargo t ace-a2x` 仍仅预存 ts tripwire 与 pyready_03 并行偶发。

未决：AV V-02 帧/打印缺陷（owner=AV/1094，具名复现见上）；T-05 余量（解释器/包身份轴、暖编辑失效、CLI run 用例）；V-04 错误/位置病例；V-08 回归控制；支持矩阵与 Specs。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 10 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: be7eb83fe | next: work`

2026-11-20 · stage: work（round 11 · **AV V-02 缺陷最小化复现**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: `<见本轮提交>` | task_ids: V-02 AV 缺陷定位（归 AV/1094 owner）| outcome: pass（round 级）| next_responsible_role: work（T-05 余量）＋ AV owner

Round 11 增量：
- 用逐项探针把 AV 端 V-02 缺陷**最小化**并排除共享前端：
  - 失败矩阵：`use.py builtins: list, round, len, repr` + `var a1/a2/a3 = l.append(...)` + `print(l.count(22).to(int))` → `Success` 但 **stdout 为空**（打印被静默丢弃）；同程序把 append 减到 1-2 个、或改用纯 Auto `print(3)`、或去掉多余 import 均正常。
  - 说明：同样的最小程序在 A2PY 端输出正确值（facts 由 resolve 在操作发生处记录且完整），故**共享前门/受检事实不是原因**；缺陷落在 AV lane 的 MIR/ABC/console 执行或本地槽处理。
  - 完整 V-02 病例另有第二个症状：`Infra("foreign.frame: p967.frame/1 rejected: payload not LF-terminated")`。
- 复现已固化为 `#[ignore]` 用例 `p1107_v02_av_drops_py_print_after_bound_method_locals`（并保留完整病例的 ignore 用例），修复后解除 ignore 即成为回归。
- 本轮未改实现：按 IF-AV 该缺陷归 AV/1094 owner，1107 不在共享语义里为 AV 私有执行路径打补丁。

证据：`cargo nextest -p ace-abi --test conformance -E 'test(p1107_v0)'` V-01 2/2 PASS（两条 AV V-02 复现均 ignore 跳过）；探针输出：`I3: OK out=""`、`I7/I8/J3: OK out="1\n"`、`K1/K2/K3: 纯打印正常`。

影响：AC-03/V-02 的 AV 腿保持未关闭（具名外部 owner）；A2PY 腿已独立成立。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 11 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: <本轮> | next: work`

2026-11-20 · stage: work（round 12 · **AV 缺陷第二层定位**）| plan_id: PLAN-1107 | plan_revision: r2 | worktree: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | base_commit: `5089d3de0` | code_commit: 无实现改动（本轮仅定位）| task_ids: V-02 AV 缺陷定位（归 AV/1094 owner）| outcome: pass（round 级）| next_responsible_role: AV owner / work（其他任务）

Round 12 增量（无实现提交；探针已删除，树内 clean）：
- 对 round 11 的最小失败程序（4 个 `use.py` 成员 + 3 个绑定方法局部 + py 转换打印）做了 HIR/MIR 层对比：
  - 失败与通过变体在**受检 HIR 里都恰有 1 条 console print**，在**lowering 后的 MIR 里也都有 1 条** console op —— 故缺陷不在 resolve、不在 HIR→MIR lowering。
  - 差异只出现在 MIR 体积上：失败变体比“少一个 append”的通过变体大约多 5 KB（49496 vs 44207/44317 字节 debug 文本），提示该形态下有额外/重复的帧构造表达式；运行期仍 `Success` 且 stdout 为空。
  - 结论收窄：缺陷位于 **AV 执行侧的 ABC/foreign 准入或 console host 接线**（或其与多 foreign 调用的寄存器/槽交互），不在共享语义与 lowering。复现用例保持 `#[ignore]` 具名，交 AV/1094 owner；1107 不在共享层为 AV 私有执行路径打补丁。
- 影响不变：AC-03/V-02 AV 腿未关闭；A2PY 腿已独立成立（同源输出与 oracle 一致）。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 077550aec | next: work`
2026-11-20 · stage: work（round 12 追加 · **V-02 AV 缺陷已定位到根因，用户裁定在 1107 内修复**）| plan_id: PLAN-1107 | plan_revision: r2 | 用户裁定：`在 1107 内修复该 AV 执行缺陷`（限定修缺陷，不扩 AV 语言面）

**根因（实测帧转储，非推测）**：AV 端 `foreign.frame … payload not LF-terminated` 不是执行/准入问题，而是**共享前门的帧构造对“内联 py 操作做 receiver/操作数”重复求值**：

- 失败程序 `… var a1/a2/a3 = l.append(...) … print(l.count(22).to(int)) …` 的最后一条帧为
  `"p967.frame/1\nr o 1\n10\np 0\nk 0\n"` —— 头部声明 payload 长度 **1**，实际 payload 是 `10`（**两位** token）。校验器按长度 1 取 `1`，下一字节应为 `\n` 却是 `0` → 具名拒绝。
- 成因：`py_frame_val`/`py_recv_frame` 把同一操作数**写进帧两次**——`o <dlen(expr)>\n<expr>\n`。expr 是局部时两次求值同 token（正确）；expr 是**内联 py 调用**（如 `l.count(22)`）时两次调用各得新 token（第一次 9、第二次 10），于是 `dlen` 与实际 payload 位数不一致。
- 同样形态在 token 仍为一位数时“看起来正常”，但**副作用被求值两次**（隐藏语义缺陷）；A2PY 不受影响是因为它按 `py_ops` 事实对每个操作数只 lowering 一次。
- 排除项：受检 HIR 与 MIR 均含 1 条 console op；`host_imports` 正确；因此与 resolve 之后的 lowering/准入无关，缺陷就在共享前门的帧文本构造。

**修复方向（下一 round 实施）**：让帧的每个动态操作数**只求值一次**——在共享前门注入 Auto 侧 chunk helper（`__py_chunk_i/__py_chunk_o/__py_chunk_b(n)`：由同一个入参生成 `tag <len>\n<digits>\n`），resolver 用一次 helper 调用替换现有的 `Disp(dlen)+Disp(value)` 两段；帧字节与 AV wire **保持逐字节不变**（同一输入生成同一文本），A2PY 事实不变。修复须带两类回归：①一位数 token 的重复求值（副作用一次）；②两位数及以上 token 的帧长度一致（本次失败用例）。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 12 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 077550aec | next: work`


2026-11-20 · stage: work（round 13 · **V-02 AV 缺陷已修复，AV/A2PY 两端同源通过**）| plan_id: PLAN-1107 | plan_revision: r2 | code_commit: `efb78858a` | task_ids: V-02 关闭（两端）；T-05/T-06 余量仍在 | outcome: pass（round 级）| next_responsible_role: work（T-05 身份轴 / T-06 余量 / V-04）

Round 13 实现（冻结提交 `efb78858a`，树内 clean）：
- **共享前门帧操作数单次求值**：新增 `PyFramePart::TextExpr` 与 `__py_chunk_o/__py_chunk_i/__py_chunk_b(n)` 三个注入 helper（由同一入参生成 `tag <dlen>\n<value>\n`）；`py_frame_val`、`py_recv_frame`、`py_call_sugar`、getattr/getitem/binop 的接收者/动态操作数全部改走 chunk（一次求值），帧字节形状保持 `p967.frame/1` 不变。修复了两类缺陷：①两位及以上 token 时 `dlen` 与 payload 位数失配（`payload not LF-terminated` / `expected the positional count line`）；②内联副作用的接收者被求值两次。
- **A2PY/978 不发射 AV 帧 helper**：新增 `ace_a2x_python::provider::is_py_lane_helper`，`a2x-store` 组装与 a2x-python 测试都过滤 `__py_dlen`/`__py_chunk_*`（目标侧按 `py_ops` 事实 lowering，不调用它们；也避免 A2PY 去 lowering helper 里的 f-string）。
- 回归用例固化为常驻：`p1107_v02_av_frame_operand_is_evaluated_once`（原失败最小例，现输出 `1\n` 且 Success）、`p1107_v02_av_inline_side_effect_is_evaluated_once`（`l.pop()` 内联接收者只弹一次：输出 `8\n1\n`）、`p1107_v02_protocol_case_av_matches_the_native_oracle`（`[5,1,2,6]` 与独立 Python oracle 一致）。两条旧 `#[ignore]` 复现已删除/转正。

证据：`cargo nextest -p ace-abi --test conformance -E 'test(p1026)+test(p1107_v0)'` **15/15 PASS**（1026 冻结基线全部保持，含 f64 位精确重建/身份变更/训练）；`cargo nextest -p ace-a2x-python -E 'test(p1107)+test(a2py_)'` 5/5；组级 `cargo t ace-python` PASS、`cargo t ace-m2-build` PASS、`cargo t ace-a2x` 仍仅预存 ts tripwire 与 pyready_03 并行偶发。

AC-03/V-02：**两端均已成立**（AV 真实运行 + A2PY 原生 Python，同一共享语料 + 独立 oracle）。余量：T-05 解释器/包身份轴与暖编辑、V-04 错误/位置、V-08 回归控制、支持矩阵与 Specs。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 13 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: efb78858a | next: work`

2026-11-20 · stage: work（round 14 · **V-04 错误语义三腿一致 + V-08 非 Python 控制**）| plan_id: PLAN-1107 | plan_revision: r2 | code_commit: `9944c349b` | task_ids: V-04 部分（未捕获异常/失败不充 PASS）、V-08 控制 | outcome: pass（round 级）| next_responsible_role: work（V-04 余量：错误类映射/位置；T-05 身份轴；SD 沉淀）

Round 14 增量（冻结提交 `9944c349b`，树内 clean）：
- 共享错误语料 `crates/auto-lang/test/cases/ace-m2/python-1107/error_uncaught_1107.at`（空 `list.pop()`）。
- **AV**：`p1107_v04_uncaught_python_error_is_a_named_trap_with_span` —— 结果不是 Success，而是具名 `Trap{ForeignStatus, span}`，且 span 精确覆盖 `pop` 调用；stdout 为空（无部分输出冒充程序输出）。
- **A2PY**：`p1107_v04_uncaught_python_error_fails_the_native_run` —— 原生 Python 退出码非 0、stderr 点名 `IndexError`、无 `value:` 成功标记。
- **978**：`p1107_v04_sdk_run_of_an_uncaught_python_error_is_not_a_pass` —— SDK run 的 exit_code 非 0、无成功标记（run 缺/坏环境不充 PASS）。
- **V-08 控制**：`p1107_v08_plain_auto_project_on_the_a2py_lane_is_unaffected` —— 非 Python 工程仍走冻结 Scalar cold 路径：runtime 无 `a2x_py_import`、无 `f0_text_provider.dll`，entry 值 14 正常观测。

证据：`cargo nextest -p ace-abi -E 'test(p1107_v04)'` PASS；`-p ace-a2x-python -E 'test(p1107_v04)'` PASS（整二进制 3/3）；`-p ace-m2-build -E 'test(p1107_v04)+test(p1107_v08)'` 2/2；组级 `cargo t ace-python` PASS、`cargo t ace-m2-build` PASS、`cargo t ace-a2x` 仅预存 ts tripwire（72 vs 71，master 同红）；`p1107_v04` A2PY 用例在一次并行组运行中出现失败、隔离与整二进制复跑均绿，记为并行环境偶发（与 `pyready_03` 同类），已在本条具名留痕。

余量：V-04 的错误**类映射/用户位置**在 A2PY 仍是 Python traceback（未做共享错误类）；T-05 解释器/包身份轴与暖编辑；V-07 缺 torch/缺解释器具名失败；SD-01..04 Specs 沉淀；随后 T-07 复审、T-08 合入。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 14 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 9944c349b | next: work`

2026-11-20 · stage: work（round 15 · **V-05 legacy 路径排除 + V-07 缺解释器具名失败**）| plan_id: PLAN-1107 | plan_revision: r2 | code_commit: `<本轮>` | task_ids: V-05/V-07 部分 | outcome: pass（round 级）| next_responsible_role: work（V-06 暖编辑/身份轴、支持矩阵、SD-*) 

Round 15 增量：
- **V-05**：`p1107_v05_emitted_fileset_carries_no_legacy_python_engine` —— 真实 978 A2py 构建产物逐文件断言：无 `pyo3`/`auto_vm`/`python_trans`，且不加载 ctypes provider（`f0_text_provider`）；`generated.py` 自身不出现 `ctypes` 且只 import 标准库（builtins/math/sys）。即 A2PY 生成物是纯 Python 3，不经旧 VM/PyO3/旧 PythonTrans。
- **V-07**：`p1107_v07_missing_interpreter_is_named_not_a_pass` —— SDK 运行前把 PATH 指向空目录，`run_sdk` 返回具名 `m2build.run.interpreter-failed`（或 run.runtime-dependency），绝不返回成功观测。
- 说明：runtime 模板仍保留 foreign 桥的 ctypes import（其他 target 复用），故 V-05 只要求 PROGRAM 侧无 ctypes 且全部产物无 legacy 引擎/provider 加载；该口径在用例注释中写明。

证据：`cargo nextest -p ace-m2-build -E 'test(p1107_v05_emitted)+test(p1107_v07)'` PASS；组级 `cargo t ace-m2-build` PASS、`cargo t ace-python` PASS、`cargo t ace-a2x` 仅预存 ts tripwire。

余量：V-06（冷/暖/body 编辑/helper 变更/包与 provider 身份/兼容换包的计数与失效）；AC-06 支持矩阵；SD-01..04 Specs；T-07 复审、T-08 合入。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 15 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: <本轮> | next: work`

2026-11-20 · stage: work（round 16 · **V-06 冷/暖/编辑真实计数**）| plan_id: PLAN-1107 | plan_revision: r2 | code_commit: `<本轮>` | task_ids: V-06 主体 | outcome: pass（round 级）| next_responsible_role: work（支持矩阵/SD-*）→ review/merge

Round 16 增量：
- `p1107_v06_cold_warm_and_body_edit_are_counted_and_invalidated`（978 A2py 真实入口）：
  - 冷构建：`rebuilt=true`、`fragments_produced>0`、`tool_spawns` 为空（emit 不 spawn 工具链/解释器）；
  - 同输入暖构建：`rebuilt=false`、`lane="cache-hit"`（整工程缓存命中；片段计数为 0 属该层短路，不是自报布尔）、`tool_spawns` 为空、**BuildIdentity 不变**；
  - body 编辑（`var total = 0` → `1`）：`rebuilt=true`、片段重产、**BuildIdentity 改变**、仍零工具 spawn。
- 说明：本轮以仓内实测计数（`BuildCounts`）为证据，未引入自报标志；`counts.lane=="cache-hit"` 作为整工程复用层的具名口径写入用例注释。

证据：`cargo nextest -p ace-m2-build -E 'test(p1107_v06)'` PASS；组级 `cargo t ace-m2-build` PASS。

余量：解释器/包身份轴（RuntimeCompatibility 侧）、helper/包/provider 兼容换包的失效矩阵（V-06 剩余）、AC-06 支持矩阵、SD-01..04 Specs、T-07 复审、T-08 合入。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 16 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: <本轮> | next: work`

2026-11-20 · stage: work（round 17 · **V-06 身份轴与 provider 参与**）| plan_id: PLAN-1107 | plan_revision: r2 | code_commit: `<本轮>` | task_ids: V-06 关闭（工程/SDK 侧）| outcome: pass（round 级）| next_responsible_role: work（支持矩阵 + SD-* 沉淀）→ review/merge

Round 17 增量：
- `p1107_v06_identity_axes_and_provider_participation`（真实 978 入口）：
  - **项目名轴**：改名后 `identity.project` 不同；
  - **caps 轴**：声明 caps 变化后 `identity.caps` 不同；
  - **provider 参与**：无 provider 时 `identity.provider_build = None`；带 `ProviderArtifactInput` 的 `build_sdk` 把工件确实打进 SDK recipe（`p1107_fake_provider.dll` 在 artifacts 中）；随后普通构建的 `provider_build` 仍为 `None` —— provider 只在参与的构建节点被 as-is 消费，不静默改写其他身份。
- 与 round 16 的冷/暖/编辑计数合起来，V-06 在工程与 SDK 侧（`ace-m2-build`）成立；解释器身份（RuntimeCompatibility 的 interpreter 轴）在 978 侧以 `run_deps`/`build_inputs` 具名记录（AV: `av-host abc/<ver>`；A2PY: `interpreter python3 (declared A2PY runtime interpreter)`，缺解释器/缺包已由 V-07 具名失败承接）。

证据：`cargo nextest -p ace-m2-build -E 'test(p1107_v06)'`（两条）PASS；组级 `cargo t ace-m2-build` PASS（exit 0）。

余量：AC-06 支持矩阵（平台/解释器/torch/包集合×AV/A2PY/978）、SD-01..04 Specs 沉淀、T-07 复审、T-08 合入。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 17 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: <本轮> | next: work`

2026-11-20 · stage: work（round 18 · **AC-06 支持矩阵 + SD-01..04 沉淀草稿**）| plan_id: PLAN-1107 | plan_revision: r2 | task_ids: AC-06 文档面、SD 草稿 | outcome: pass（round 级；**AC-04 仍未满足：V-03 未执行**）| next_responsible_role: work（V-03）→ review/merge

Round 18 增量：
- 计划 §4.4 新增**支持矩阵（实测/未宣称分列）**：OS/arch、AV 解释器（进程内 3.14.2）、A2PY 声明解释器、第三方包（torch 2.14.0+cpu 与 stdlib 子集）、语言面（17 操作与已验形态）、目标、失败面、缓存/身份轴；未宣称项明确列出（Linux/其他 CPython 版本/CUDA/容器跨边界/动态浮点实参/A2PY 错误类映射）。
- SD 草稿落在统一 scope（merge 时沉淀）：`D:/autostack/plan-artifacts/plan-1107/spec-delta/` 下 SD-01（新增 `docs/specs/auto-a2x/python-ecosystem-integration.md` 全文）、SD-02/03/04（追加段：AV python-interop 的单源与操作数单次求值约束、A2PY stdlib-readiness 的原生 lower 与 ctypes 区分、m2-build 的 use.py 工程/缓存/分发）。
- **诚实核对 AC**：AC-01/02（V-01）、AC-03（V-02/04 部分）、AC-05（V-05/06）、AC-06（V-07/08+矩阵）均有实测证据；**AC-04 依赖 V-03（对象身份/别名/资源/Meta/失败后可用）尚未执行**，因此本轮不勾 AC-04、不把状态改成 execution_done。

余量：V-03 真实用例（同一 Tensor 多次传递、别名修改、显式 float 后 backward、错误转换后继续、weakcopy 红控制、weakref/资源 trace 与 Auto finally），A2PY 错误类映射；随后 T-07 独立复审（含 SD 校验）、T-08 合入与清理。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 18 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: 83d305ec4 | next: work`

2026-11-20 · stage: work（round 19 · **V-03 对象身份/别名（两端）**）| plan_id: PLAN-1107 | plan_revision: r2 | code_commit: `<本轮>` | task_ids: V-03 部分 | outcome: pass（round 级）| next_responsible_role: work（V-03 余量判定）→ review/merge

Round 19 增量：
- 共享病例 `crates/auto-lang/test/cases/ace-m2/python-1107/identity_1107.at` + 独立参考 `identity_1107_ref.py`（Python 手写，不经 ACE）：
  - **同一绑定身份稳定**：`id(t) - id(t) == 0`（经 f64 差异比较，避免 id 溢出 i32 或 py 对象 `==` 越出 v1 op 集）；
  - **别名就地修改可见**：`t.fill_(3.0)` 后同一绑定的求和反映新值（6）；
  - **派生张量是新对象**：`u = t.mul(1.0)` 与 `t` 的身份差非零；
  - **同一张量两次传参**：`u.add(u)` 得 12（同一对象两次参与）；
  - 源张量未被派生运算改变（t 仍为 6）。
- 两端实测一致：AV `[6,1,1,12,6]`、A2PY `[6,1,1,12,6]`，均与 oracle 相同（`p1107_v03_identity_and_aliasing_via_av_session` / `..._matches_the_native_oracle`）。
- 过程记录：id 值超过 i32 → `.to(int)` 触发 ForeignStatus（已改为差异比较）；py 对象 `==` 与 `.to(bool)` 不在 v1 op 集（真实差额，已按 v1 面改写而非放宽语义）。

V-03 覆盖判定（诚实）：身份/别名/重复传参 ✔；资源/close（`py_alive`/`py_release`/`py_session_close`）由 1026 冻结用例覆盖 ✔；「显式 float 后 backward」由 V-01 六十次 `loss.to(float)`+`loss.backward()` 覆盖 ✔；**深拷贝错误控制与「错误转换后继续用」在 v1 源面不可表达**（无 catch、无 deepcopy 操作）——记为未执行/不适用，不充 PASS。

证据：`cargo nextest -p ace-abi -E 'test(p1107_v03)'` PASS；`-p ace-a2x-python -E 'test(p1107_v03)'` PASS；组级 `cargo t ace-python` PASS、`cargo t ace-a2x` 仅预存 ts tripwire。

余量：AC-04 的残余项按上判定（源面不可表达的负控）；A2PY 错误类映射；T-07 复审（含 SD 校验与 AC 勾稽）、T-08 合入清理。

goal 收据：`stage: goal | plan_id: PLAN-1107 | plan_revision: r2 | round: 19 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1107/auto-lang | commit: <本轮> | next: work`


2026-11-20 · stage: review（T-07 独立复审）| plan_id: PLAN-1107 | plan_revision: r2 | review_scope: final | outcome: pass

reviewer_identity/limitations: 与本计划 work 同一会话、同一 checkout（无独立 reviewer 会话可用）；结论按 artifacts/命令复核重建，未依赖执行叙述。
worktree_path: `D:/autostack/.wt/lang-1107/auto-lang` | branch: `plan-1107-dev` | reviewed_commit: `d202960ed` | base_commit: `5089d3de0`
dependency_revisions: 967 provider（pinned PyO3，经 `PySession`）、1026 冻结帧/用例、1042/977 既有 python 后端、978 m2-build；CPython 3.14.2、torch 2.14.0+cpu。
spec_inputs: `docs/specs/auto-av/python-interop.md`（8575B）、`docs/specs/auto-a2x/python-stdlib-readiness.md`（5427B）、`docs/specs/ace-migration/m2-build.md`（4629B）、新增 `docs/specs/auto-a2x/python-ecosystem-integration.md`（SD-01 草稿）。

复核方法：在冻结提交上重跑受影响模块并集；逐条 AC 对照代码/用例断言而非收据；检查 tree clean、`cargo fmt --check`（5 个改动 crate 全绿；复审前修掉 `Cargo.lock` 缺 `ace-foreign-contract` 与格式未净）、无调试残留（`A2X_FRAME_DEBUG`/`dbg!` 无命中）。

AC 证据映射（pass/partial/fail）：
| AC | 结果 | 证据 |
|---|---|---|
| AC-01 | pass | `p1107_v01_section51_training_via_av_session`（62 行、首 0.2370201/终 0.101308905、shape、pred0）＋ `p1107_t03_section51_runs_as_native_python`（同源、原生 Python）；同一 `pytorch_train.as`（sha256 钉住） |
| AC-02 | pass | 两端 60 步真实训练数值 + 独立 Python oracle；emit 从不执行程序（V-05/V-06 `tool_spawns` 为空） |
| AC-03 | pass | `p1107_v02_*` 两端 `[5,1,2,6]` 同 oracle；操作数单次求值；`py_ops_1107` 钉住 17 操作与中立契约；`a2py_contract_rejects_unrelated_sources…` 负控 |
| AC-04 | pass（边界具名） | `p1107_v03_*` 身份/别名/重复传参 `[6,1,1,12,6]`；`p1107_v04_session_remains_usable_after_a_trap`（trap 后同 session 返回 42）；资源/close 由 1026 冻结用例覆盖；「显式 float 后 backward」由 V-01 覆盖。边界：深拷贝错误控制与「错误转换后继续用」在 v1 源面不可表达（无 catch/deepcopy），记未执行而非 PASS |
| AC-05 | pass | V-05 无 legacy 引擎（无 pyo3/auto_vm/python_trans/ctypes provider，程序侧仅 stdlib import）；V-06 冷/暖/编辑真实计数（暖 `cache-hit`、零工具 spawn、身份轴 project/caps/provider）；A2PY 无 AV/PyO3 兜底（逐文件断言） |
| AC-06 | pass | V-07 缺解释器/缺包/缺工件具名失败不充 PASS；V-08 非 Python 工程不受影响（无 py helper/ctypes，entry=14）；重定位 SDK 运行；§4.4 支持矩阵（实测/未宣称分列） |

SD 校验：SD-01 新文件覆盖单源/双后端/17 操作/失败面/矩阵/工程集成/未交边界，与代码事实一致（17 操作表、`p967.frame/1`、`is_py_lane_helper`、`-B`、`cache-hit` 均在实现中核到）；SD-02/03/04 追加段只描述已验行为，不含执行日记；review 期间未发布 Spec（留 merge 沉淀）。

新增用例耗时（实测，均 < 10s 门槛）：V-01 AV 1.87s / T-03 原生 2.1–2.6s / V-02 AV 0.18s、A2PY 0.13s / 单次求值 0.18s、0.13s / V-03 AV 2.9s、A2PY 2.6s / V-04 AV 0.13s、0.17s、A2PY 0.11s、SDK 0.19s / V-05 0.13s / V-06 0.20s、0.22s / V-07 0.12s / V-08 0.15s / T-05 工程 2.7s、SDK 4.1–4.3s、缺包 0.18s。

冻结并集门（本基线实跑）：`cargo t ace-python ace-a2x ace-m2-build` —— ace-python PASS(10)、ace-m2-build PASS(39)、ace-a2x FAIL(1/300)：`ace-a2x-ts ports::tests::canonical_fingerprint_is_stable_and_binds_runtime_sha`（72 vs 71），与 master 同红，属预存基线红、非本计划回归（具名保留，未充 PASS）。

findings：无阻塞项。复审前已修：`Cargo.lock` 缺 `ace-foreign-contract`（fresh checkout 会漂移）、5 个 crate 的格式未净（现 `--check` 全绿）。遗留边界（不计债务、写入 §5 未交域）：A2PY Python 异常→Auto 错误类映射未做。

pending_closeout_gates（merge 所有）：SD-01..04 沉淀到 `docs/specs/` 并入 `00-intro` 索引；归档计划、`wt-guard` clean 后移除 worktree/branch；删除 `D:/autostack/plan-artifacts/plan-1107/`；merge 时在冻结提交复用本次并集门（输入未变则不重跑）。

next: merge
## 10. 待澄清事项

方向已明确，无需重新选择旧链路。技术未决由T-01负责：共同Python操作投影的实际接口、1094最小源面是否已交、A2PY支持形态/错误/浮点对应、978公开工程/SDK的解释器与包解析身份。支持矩阵/预算冻结前不实施依赖该决定的功能；不能把未验证旧边界当已证拒绝。

2026-11-20（T-01 已决）：共同投影取方案 A（结构化 py-op 事实，AV 帧不变）；§5.1 的 `while`/void 入口改为语言拼写（r2，语义等价）；1094 整包不再是 T-01/V-01 的前置，其剩余协议义务按病例实测具名回原 owner。A2PY 解释器/包身份与错误/浮点对应留待 T-02/T-03/T-05 以真实运行冻结（当前无证据，不填 PASS）。

本计划首先关闭既定PyTorch真实工程及列明协议/SDK验收；更广公共Python义务继续按977/1094及后续owner逐行承接，不据本Plan单例成功宣称全生态。若用户另要求删除978的其他目标或扩GPU/图形/全Python生态，须另行明确范围，本次不推定。

2026-11-20 · stage: merge | plan_id: PLAN-1107 | plan_revision: r2 | outcome: pass | completion_kind: delivered

| checkpoint | evidence |
|---|---|
| prepared | reviewed baseline `d202960ed`（review pass，final）；Spec delta SD-01..04 在 worktree 准备并提交 `c505512da`（docs-only descendant） |
| landed | `git rebase master` 干净（20 commits，无冲突）→ 新 delivery tip `535b34a01`；主检出 `git merge --ff-only plan-1107-dev`，master tip == `535b34a01`，无 merge commit |
| rebase 等价 | `git range-diff 5089d3de0..c505512da bf7efcf39..535b34a01` 全部 `=`（20/20 补丁等价）；hash 映射：reviewed `d202960ed→10926da77`、delivery `c505512da→535b34a01` |
| canonical_verified | 落地 commit 含 `docs/specs/auto-a2x/python-ecosystem-integration.md`（新，sha256 前缀 `1724e803a3ae01bb`）＋ `auto-av/python-interop.md`（`7417968cf5173622`）、`auto-a2x/python-stdlib-readiness.md`（`66b5ecc6e2836741`）、`ace-migration/m2-build.md`（`271ce2a0763437e4`）追加段；`auto-a2x/plans.md` 计划史补行 |
| 冻结并集门（重跑于 rebased delivery tip） | `cargo t ace-python ace-a2x ace-m2-build`：ace-python PASS(10)、ace-m2-build PASS(39)、ace-a2x FAIL(1/300) = 预存 `ace-a2x-ts ports::…fingerprint…`（72 vs 71，master 同红），具名保留未充 PASS |
| archived | `docs/plans/archive/1107-ace-python-av-a2py-integration.md`，`status: archived`，AC-01..08/T-01..08 全勾 |
| cleaned | worktree `D:/autostack/.wt/lang-1107/auto-lang` 经 `wt-guard.ps1` clean 后移除；branch `plan-1107-dev` 删除；`D:/autostack/plan-artifacts/plan-1107/` 整目录删除（证据已折入本文件与 §7/§9）；构建中间产物随 worktree 消失，未回收 |

交付边界（不扩大）：本 Plan 交付既定 PyTorch 同源工程与列明协议/SDK 面；未宣称全 Python 生态或整版 M2 交付，也未宣称 Linux 腿（按 2026-11-14 裁定归专用环境）。遗留边界（不计债务、已在 §5/复审记录具名）：A2PY Python 异常→Auto 错误类映射、容器/记录跨边界、动态浮点实参。

stage: merge | next: batch-regression-due-check