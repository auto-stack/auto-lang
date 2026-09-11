---
plan_id: PLAN-602
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: py-subclass-class-factory
author: [ZCode]
created_at: 2026-09-09
updated_at: 2026-09-11
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals:
  - "GOAL-006"                # Consumer parity：py 子类派生 = torch 全面支持的主台阶（暂定，review 定稿）

affects: [auto-lang/vm, auto-lang/trans, parity]
current_step: 0
total_steps: 10
---

# [PLAN-602] py-subclass-class-factory：Python 类派生工厂——nn.Module/Dataset 子类的 Auto 回调绑定

> python-parity-roadmap §7.3 / §7.4 队列④-关联的 **W3 类派生延期件**
> （P539-D4，Plan 539 显式延期项）的立项补课。539 T21 回调桥（单回调通道）
> 与 PLAN-598（闭包内 let/句柄修复）已清掉两块前置；本计划补上最后一块：
> **多参回调封送 + exec 类工厂桥**，让 Auto 闭包能作为自定义 nn.Module/
> Dataset 的方法被 Python 侧调用。**这是 torch 线从"组合式 Sequential 金样"
> 到"可移植真实模型代码"的主台阶。**

## 0. 变更摘要

1. **多参回调封送（前置件）**：`run_closure_bridged` 现固定单参
   （py_ffi.rs L2020：`arg: &Bound<PyAny>` 单值 + `call_closure(id, 1)`）。
   泛化为 n 参——Python 侧调用参数按 tuple 顺序逐一封送上栈，闭包形参数
   决定 arity，不匹配 → Python TypeError。
2. **`py_subclass(name, base, methods)` 类工厂桥（核心件，新 native）**：
   - `methods` = Auto 对象字面量；值两类——**Closure**（Auto 回调方法，
     经 PyCFunction::new_closure 包装为类方法，Python 隐式 self 作为首参
     封送为句柄）与 **Str**（Python 源码方法，exec 模板内联，如
     `__init__`/纯 Python 辅助方法）；
   - 实现 = 工厂 shim 内 exec 类模板 + callback 方法 setattr 挂载，
     返回类句柄；实例化走既有 `py_call0(cls, args...)`（Python type call，
     `__init__` 正常执行）；
   - 双轨：VM native 臂（py_ffi 新桥，py 桥 480→481+）+ a2py
     `_auto_subclass` helper 注入与发射臂。
3. **语料**：`parity/libs/python/py_torch_subclass/` 新套件（三方 parity）：
   自定义 nn.Module（Python `__init__` 建层 + Auto `forward` 回调）、
   Dataset（Auto `__len__`/`__getitem__` 回调）、Auto 驱动训练小循环；
   新 phase `p12` 注册。
4. **审计与账面**：GIL/单线程/窗口纪律成文（回调仅在宿主 shim 窗口内
   合法——窗口外触发显式 RuntimeError，既有守卫转文档契约）；重入深度
   探针；KNOWN-DEBT P539-D4 销账拆面；路线图 §7.3/⑦/⑧ 注记更新。

**非目标**：多线程/多 worker 回调（T02 约束 num_workers=0 不动摇）；GPU
（队列 ⑧ 条件立项）；`use.py class` 声明式语法糖（bridge API 先行，语法面
需求驱动另立）；pickling/多进程迁移语义；Auto struct 运算符重载（队列④）。

## 1. 目标

- 自定义 nn.Module/Dataset 子类：Python 侧类 + Auto 回调方法，三轨
  （AutoVM ≡ a2py ≡ 原生 Python）行为一致。
- 回调 ABI 从单参泛化为 n 参，self 以句柄封送（Auto 侧可经
  py_getattr/py_call 继续操作 Python 状态）。
- GIL/单线程/重入纪律从隐式约束升格为成文契约 + 探针钉死。
- 账面：P539-D4 销账拆面；路线图同步。

## 2. 架构方案

```
Auto 侧（corpus）
  let MyModule = py_subclass("MyModule", nn.Module, {
      __init__: "def __init__(self, dim):\n        self.lin = nn.Linear(dim, 1)",
      forward: (self, x) => py_call0(py_getattr(self, "lin"), x)
  })
  let m = py_call0(MyModule, 4)                    ← Python type call，__init__ 正常
  let y = py_call(m, "__call__", batch)            ← Auto 驱动；窗口内回调合法

VM 轨（py_ffi.rs）
  py_subclass arm（新 native，py 桥 481）:
    methods 遍历 → Closure 值: PyCFunction::new_closure 包装（n 参回调，见下）
                → 收集为 callbacks: Vec<(name, closure_id, n_args)>
                → Str 值: 收集为 src_methods: Vec<(name, source)>
    exec 模板（工厂 shim 内，GIL 在手）:
      namespace = {__base__: base_handle}; exec("class {name}(__base__):\n" + src_methods)
    callbacks setattr(cls, name, pycfunc) → 返回 cls 句柄
  run_closure_bridged 泛化: args tuple 逐元素 py_auto_marshal_return 上栈
    → call_closure(id, n)（n = closure.params.len()；不匹配 → TypeError）

a2py 轨（trans/python.rs）
  "py_subclass" 臂 → 注入 _auto_subclass helper（exec+setattr 同构实现）+
  发射 _auto_subclass(name, base, {m: lambda…/str…})；对象字面量值中
  Closure 经既有 lambda 发射（598 后语句体也安全）

窗口纪律（成文，非新代码）
  回调仅在宿主 py_call 族 shim 的 BridgeGuard 窗口内合法（T02：num_workers=0，
  单线程 GIL 在 VM 线程）；窗口外触发 → 既有 RuntimeError 守卫（转契约）。
  Auto 驱动循环 = 每次迭代经 py_call 族进入 → 窗口覆盖全程回调链。
```

门禁归属：py_ffi.rs → `cargo tv`；trans/python.rs → `cargo tt`；双触 →
fold 前 `cargo tf`（review 阶段）。py 套件回归走 parity 相位。

## 3. 技术栈

Rust（py_ffi.rs 回调桥 + 新 native 臂；trans/python.rs a2py helper）、
PyO3（PyCFunction::new_closure / exec）、parity py 套件（torch 环境，本机
已验可用）。无新 crate 依赖。

## 4. 需求分析与背景调查

**授权**：用户确认"⑦′ 先行，W3 紧随"的路线推荐（2026-09-09），⑦′ 已交付
（PLAN-598 归档），本计划即"W3 紧随"件。预算：中型计划（539 W3 补课件），
未设自动续跑上限。

**债务与延期原文**：
- KNOWN-DEBT P539-D4：自定义 nn.Module/Dataset 需 Python 侧类工厂（exec
  生成类 + 方法绑回 Auto 回调）；回调桥 T21 已通（thread-local 任务槽，
  map/apply_ 双探针实证）；类工厂的方法绑定面 + GIL/生存期约束审查超 W3
  预算。组合式替代金样 = py_torch_train（Linear 裸栈 + seed 化收敛）。
- roadmap §7.3：W3 类派生延期备注（同上）；§7.4 队列④（Auto struct 运算符
  重载——非本批）、⑧（GPU/上层生态——条件立项）。

**T21 桥现状实勘（2026-09-09，py_ffi.rs）**：
- 窗口 = thread-local `BRIDGE_TASK`/`BRIDGE_VM` 裸指针（L1969-1972），
  BridgeGuard 于宿主 shim 入口安装/出口清除（L2000-2015）——窗口语义 =
  "回调只能在 py_call 族 shim 持 GIL 期间触发"（T02 约束，num_workers=0）。
- `run_closure_bridged`（L2020-2052）：**单参**封送
  （`py_auto_marshal_return(arg)` ×1 → `call_closure(task, id, 1)` →
  `pop_nv` 回转）；窗口外触发 → RuntimeError 守卫。
- `nv_to_value_local`（L1977）：对象/list 回转 `Value::VmRef`（句柄）——
  self 封送为句柄的通道已存在。
- `py_callable`（L293/L1381）：单闭包 → PyCFunction 包装（identity 于
  a2py 侧）。
- 无任何 py_subclass/py_class 桥（grep 0 hits）——真新路径。

**关联计划**：539（W1 dunder 路由/T14 py_with/T21 回调桥）、560（T07 中缀）、
567（Err 通道/may 家族/py_anno T20）、569（py 分派动态化）、598（闭包内
let/句柄修复——本计划的前置清障件）。

## 5. 详细设计

### D1 多参回调封送（前置件）

`run_closure_bridged` 泛化：签名改收 `&Bound<'py, pyo3::types::PyTuple>`
（PyCFunction 回调天然收 args tuple），arity = 闭包 params.len()；
0 参（`__len__`）→ 不上栈直呼；逐元素 `py_auto_marshal_return` 上栈后
`call_closure(task, id, n)`；调用方 tuple 长度 ≠ n → Python TypeError
（消息含期望/实际）。`py_callable`/map 既有单参探针保持绿（单参 = n=1
特例）。

### D2 py_subclass 工厂桥（VM native 臂）

签名 `py_subclass(name: Str, base: Any, methods: Object) -> Any`（类句柄）。
处理序（单 shim 内完成，GIL 在手）：
1. 遍历 methods：Closure 值 → 闭包 closure_id + params.len() →
   生成 `PyCFunction::new_closure`（回调内调 `run_closure_bridged_n`）→
   暂存 callbacks；Str 值 → 暂存 src_methods（Python 源码方法体）。
2. exec 模板：`namespace = {"__base__": base, ...}`；源码 =
   `class {name}(__base__):\n` + src_methods 原文（缩进归一）；
   `py.run(&src, Some(ns), None)`。
3. callback 方法 `setattr(cls, m_name, pycfunc)`（PyCFunction 作为类属性
   即未绑定函数，Python 调用时 self 自然作首参传入）。
4. 返回 cls 句柄。
Python 隐式 self 约定：**回调闭包首参 = self 句柄**（Auto 侧用
py_getattr(self, ...) / py_call(self, ...) 继续 Python 状态；不需要 self
的方法（如 Dataset `__len__`）照常声明首参但不使用）。

### D3 a2py 轨（trans/python.rs）

`"py_subclass"` 发射臂：改写为 `_auto_subclass(name, base, {…})` + 注入
Python helper（与 `_auto_may` 同机制，`needs_*` 旗标族）：
```python
def _auto_subclass(name, base, methods):
    ns = {"__base__": base}
    src = f"class {name}(__base__):\n"
    cbs = {}
    for m, v in methods.items():
        if callable(v): cbs[m] = v
        else: src += v if v.startswith("    ") else _indent(v) + "\n"
    exec(src, ns); cls = ns[name]
    for m, f in cbs.items(): setattr(cls, m, f)
    return cls
```
（a2py 侧 closure 值经既有 lambda 发射即为 Python callable，598 后语句体
亦安全——helper 内 callable() 判定自然分流。）对象字面量 emission 顺序
确定性 = 现有 Expr::Object 臂（保序）。

### D4 语料 py_torch_subclass（三方 parity）

`parity/libs/python/py_torch_subclass/`：
- `tests/auto/subclass.as`：TAP 风格（镜像 tests/python/test_subclass.py
  测试名）：
  1. custom Linear 子类（Python `__init__` 建层 + Auto `forward` 回调，
     seed 固定）→ 实例化 + 前向标量断言；
  2. Dataset 子类（Auto `__len__`/`__getitem__` 回调）→ len + 索引取值
     断言；
  3. Auto 驱动小训练环（Dataset + 手动 loop + loss 标量断言；num_workers=0
     纪律注释）；
  4. 窗口外回调负面：Auto 侧先把方法句柄存表、延迟到非 shim 上下文触发
     → 断言 RuntimeError 文案（负面 golden）。
- `tests/python/test_subclass.py`：原生 Python 同语义 oracle（同名测试）。
- `README.md`：scope/版本/惯用法（self 首参约定、窗口纪律、num_workers=0、
  torch CPU seed 固定）。
- 注册：`discover_libraries_by_phase` 新 phase `p12`（py_torch_subclass）；
  dashboard phases 不含（无网络门控需求，视 T-06 实测定）。

### D5 审计成文（GIL/重入/生存期）

`docs/design/strategy/python-parity-roadmap.md` §7.3 扩写为契约节（或独立
小节）：
- 窗口纪律（回调仅 shim 窗口内合法；num_workers=0；单线程 GIL 在 VM 线程）；
- 重入深度：Auto→py→回调→py→… 的嵌套 shim 链实测探针（深度 ≥3 钉死；
  无硬限则记录实测上限与栈预算）；
- 生存期：self/参数句柄走既有 VmRef/RC 纪律（无新增机制，审计确认）；
- 已知边界：Python 侧主动回调（非 Auto 驱动）不支持 → W3 后续/多线程泵。

### 规范增量

| delta_id | 操作 | 目标 | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/overview.md | py 桥 480 → +py_subclass（exec 类工厂：Closure 方法 n 参回调绑定、Str 方法 exec 内联、self 首参句柄约定）；回调封送单参 → n 参（arity 不匹配 TypeError） | 类派生 = torch 全面支持主台阶；回调 ABI 泛化是工厂的前置件 | AC-01..04 |
| SD-02 | modify | docs/specs/auto-lang/frontend/overview.md | a2py：py_subclass 发射臂 + `_auto_subclass` helper 注入（exec+setattr 同构） | 双轨一致性（VM ≡ a2py） | AC-01..03 |
| SD-03 | add | docs/design/strategy/python-parity-roadmap.md | §7.3 扩写为窗口/GIL/重入/生存期契约节（审计成文）；§7.4 队列⑦⑧注记同步 | 隐式约束升格为成文契约 | AC-04 |

（SD-03 目标为策略文档——它同时是债务账面的上游客体；review 定稿。）

## 6. 测试设计

| 用例 | 层 | 断言 |
|---|---|---|
| n 参回调 | 单测（py_ffi）+ 语料 | tuple 逐元素封送；arity 不匹配 TypeError |
| custom Module forward | 三轨 | seed 固定前向标量三轨一致（golden） |
| Dataset len/getitem | 三轨 | 回调值三轨一致 |
| Auto 驱动训练环 | 三轨 | loss 标量 golden 一致（seed 化收敛） |
| 窗口外回调负面 | VM+a2py | RuntimeError 文案断言（负面 golden） |
| 重入深度探针 | VM | 嵌定 ≥3 层 shim 链实测（钉死记录） |
| 既有单参回调 | parity | py_callable/map 既有探针零回归 |
| 相位回归 | parity | p5-p9 + p12 全绿；tt/tv/tf 三档唯预存红 |

## 7. 验收标准

- **AC-01**：n 参回调封送——多参闭包经类方法被 Python 调用，参数序与
  arity 语义正确（含 0 参 `__len__`）；单参既有探针零回归。
  验证：py_ffi 单测 + 语料 1。
- **AC-02**：`py_subclass` 工厂——custom nn.Module（Python `__init__` +
  Auto `forward`）三轨实例化/前向一致；类句柄实例化走 py_call0 正常触发
  `__init__`。验证：语料 1 + parity 三轨。
- **AC-03**：Dataset 面——`__len__`/`__getitem__` 回调 + Auto 驱动循环
  三轨一致。验证：语料 2/3 + parity 三轨。
- **AC-04**：窗口纪律成文且被负面语料钉死——窗口外回调 RuntimeError
  （双轨）；审计节（D5）落 roadmap。验证：语料 4 + 文档 diff。
- **AC-05**：回归零漂移——cargo tv/tt 唯预存红；p5-p9 零回归；
  **cargo tf 全量门**（review 阶段）。验证：门禁命令。
- **AC-06**：账面——P539-D4 销账拆面（类工厂已交付/多线程泵仍 open 归
  长期）；路线图 §7.3 契约化 + §7.4 ⑦ 核销（引用本计划）+ ⑧ 保持条件。
  验证：三文件 diff 人工核对。

## 8. 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成]
证据。执行于 worktree `D:/autostack/.wt/lang-602/auto-lang`，开工前 master
提交本骨架 + .next-id；worktree 禁 junction/symlink；分组需 auto-down
detached 兄弟仓（nextest autodown-core 解析，同 592/598 先例）。）

- [ ] **T-01** 基线：master 提交骨架；建 worktree + auto-down 兄弟；
  构建 auto；记录基线（cargo tt/tv 现状、py_torch_train 单跑快照）。
  验证：构建零错误；两基线输出留痕。
- [ ] **T-02** 多参回调封送（D1）：run_closure_bridged 泛化 n 参 +
  py_callable 单参路径回归探针。验证：`cargo tv`；单参探针绿。
- [ ] **T-03** py_subclass VM 臂（D2）：py_ffi 新 native + exec 工厂 +
  callbacks setattr；最小 .as 探针（工厂 + 实例化 + forward 标量）。
  验证：`cargo tv`；探针输出正确。
- [ ] **T-04** a2py 轨（D3）：_auto_subclass helper + 发射臂；探针双轨
  对齐（同 corpus VM vs a2py stdout）。
  验证：`cargo tt`；探针双轨一致。
- [ ] **T-05** 语料 py_torch_subclass（D4）：subclass.as +
  test_subclass.py + README；首跑三轨对齐（golden 定稿）。
  验证：`run py_torch_subclass` 三轨全绿。
- [ ] **T-06** 注册与审计（D5 + AC-04）：phase p12 表注册；
  roadmap §7.3 契约节扩写（窗口/GIL/重入深度探针/生存期审计）。
  验证：`list`/`phase p12` 发现；文档 diff 核对。
- [ ] **T-07** 账面收口（AC-06）：KNOWN-DEBT P539-D4 销账拆面；
  路线图 §7.4 ⑦ 核销注记（引用本计划）；⑧ 保持条件立项。
  验证：三文件 diff 人工核对。
- [ ] **T-08** 回归门禁：cargo tv + tt 全绿（唯预存红）；p5-p9 + p12
  相位全绿。
  验证：门禁命令输出留痕。
- [ ] **T-09** 收口健康检查：探针残留扫描；fmt 增量干净（新代码
  rustfmt 通过，不动既有 fmt 噪音）；库 README/parity-guide 抽查。
  验证：三件套输出留痕。
- [ ] **T-10** fold 前 `cargo tf` 全量门 → 复审记录留痕（review 阶段
  执行，此处置 [ ] 由 review 勾选）。

## 9. 复审记录

（起草交接：stage=new | plan_id=PLAN-602 | rev=1 | outcome=pass |
next=work。范围锚定：W3 类派生 MVP = bridge API + Auto 驱动循环 +
num_workers=0；声明式语法/多线程泵/GPU 显式非目标。）

**执行前复审审计（2026-09-11，stage=work | rev=2 | 基线 1ed892643..master
164 提交）**：计划起草于 2026-09-10 00:02，其后 master 前进 164 提交
（PLAN-591/596/597/598/600/602/603/606/607/609/610/611/612 等）。逐面核对：

- py_ffi.rs / trans/python.rs / parity/libs/python/ / roadmap §7.3-7.5：
  164 提交**零触碰**，计划引用的行号与结构原样成立
  （run_closure_bridged L2020 单参封送、BRIDGE_TASK/BridgeGuard L1970/2000、
  nv_to_value_local L1977、`_auto_may` needs_may_helper 旗标族）。
- py 桥计数：specs vm/overview.md 仍为 480（尾号 480=py_int），无新增
  native——"480→481+" 前提成立。
- KNOWN-DEBT P539-D4：条目原文未变。
- **唯一冲突（已修正）**：相位号——PLAN-591 T9（a6dfa7e1c，2026-09-09
  18:10，早于本计划骨架 6 小时）已注册 `p11 = uuid_real`，计划原案的
  p11 系起草疏漏。全文改号 **p11 → p12**（p12 空闲），plan_revision
  1→2。
- 607/612 等近期计划均为 UI 面（iced/VM style recipes、标题菜单），与本
  计划涉及面（py_ffi/trans/parity py 套件）零交集。
- worktree `D:/autostack/.wt/lang-602/auto-lang` 存在但停在起草基线
  （分支为 master 祖先，ff 快进重同步）；auto-down 兄弟仓按 T-01 重建。

## 10. 待澄清事项

1. **exec 模板机械细节**：PyO3 侧 `py.run` 命名空间装配、缩进归一、
   base 为句柄的解引用时机——T-03 内实钉；若 exec 路线遇 PyO3 限制
   （如 GIL 重入），备选 = `types.new_class` + exec 逐方法装配（决策
   工件落 T-03 注记）。
2. **重入深度上探**：Auto→py→回调→py→… 嵌套 shim 链的实测上限与栈
   预算未预勘（D5 探针钉死；≥3 层即满足本批语料，更高深度按实测记录）。
3. **self 约定的确认**：回调首参 = self 句柄（本草案选定）vs 无 self
   （工厂剥离首参）——选 self 句柄：Auto 侧可继续操作 Python 状态
   （nn.Linear 调用、参数读取），表达力上限高；若执行期发现句柄封送
   开销不可接受再复议（决策已定，此条留痕供 review 追认）。
4. **p12 vs 并入 p9**：新套件注册为独立 phase p12（torch 子类专题；原案 p11 与 PLAN-591 uuid_real 冲突，2026-09-11 执行前审计改号，见 §9 审计记录）；
   若 review 认为应并入 p9（torch 惯用法），表项迁移零成本。
5. **a2py helper 与 VM 臂的语义漂移风险**：两侧实现需逐语义对齐
   （exec 模板/setattr 顺序/错误文案）；T-04 探针双轨对齐是闸门，
   漂移即修复而非放宽断言。
