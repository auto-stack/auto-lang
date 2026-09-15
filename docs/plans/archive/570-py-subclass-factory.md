---
plan_id: PLAN-570
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: py-subclass-factory
author: [zhaopuming]
created_at: 2026-09-06
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/trans]   # 受影响的 specs 路径
current_step: 0
total_steps: 14
---

# [PLAN-570] P539-D4 清偿：py_subclass 类工厂——Auto 闭包定义 Python 类（自定义 nn.Module/Dataset）

## 变更摘要

清偿 [KNOWN-DEBT P539-D4](KNOWN-DEBT-AND-RISKS.md)（roadmap §7.3 延期案）：Auto 脚本
定义**自定义 Python 类**——`nn.Module` 子类（自定义 forward）、`Dataset` 子类
（`__len__`/`__getitem__`）——Python 侧实例化/训练循环调用时方法体回到 Auto 闭包执行。
这是"复刻真实 PyTorch 项目"能力面的最后一块大缺口（组合式替代 `nn.Sequential`/
裸 Linear 栈已是金样，但真实项目就是自定义类）。

地基已全量在位（539 T21）：py_callable(466) 回调桥（thread-local 任务槽 +
PyCFunction::new_closure + call_closure 同任务多帧重入，map/apply_ 双探针实证）；
`type()` 三参类工厂在 pyo3 层**已单测实证可行**（`type('KwBox', (), {...})()` +
kwargs 方法调用）。本计划补三个真缺口：**变参回调实参**（现只喂 args[0]，
forward(self, x) 需全参）、**py_subclass 桥**（Auto 对象字面量 {方法名: 闭包} →
PyCFunction dict → type() 造类 → 返回类句柄）、**闭包生存期纪律**（capture 槽按
创建帧 bp 引用——跨帧存活的约束审查与守卫）；外加 a2py 类发射回译与
py_torch_custom 三方套件。

## 目标

- `.as` 内可写：

```auto
use.py importlib: import_module
use.py builtins: len

fn main() {
    var nn = import_module("torch.nn")
    var torch = import_module("torch")
    py_call0(py_getattr(torch, "manual_seed"), 0)

    var Net = py_subclass(py_getattr(nn, "Module"), "Net", {
        forward: (self, x) => {
            return py_call0(py_getattr(self, "fc"), x)
        }
    })
    var m = py_call0(Net)                       // 实例化
    py_call(m, "__setattr__", "fc", py_call0(py_getattr(nn, "Linear"), 2, 1))
    // 训练循环照常：py_call0(m, x) 触发 forward 回调回 Auto 闭包
}
```

  ——forward 回调在 Auto 侧执行（含 self 通道），训练收敛与 Python 原生自定义类
  三方一致（AutoVM ≡ a2py ≡ Python oracle）。
- Dataset 形态：`__len__`/`__getitem__` 回调 + 手动批切（DataLoader 探针为条件项）。
- `self` 约定：方法闭包首参即 self（Python 绑定方法语义自然成立——type-dict 函数
  被实例调用时 Python 自动传 self）。
- 约束成文：闭包捕获帧必须存活于回调窗口（num_workers=0、同线程、闭包定义域
  不先死）——README + 运行期守卫消息。

### 非目标

- 不做 Auto 侧 `class X(Base) {}` 原生语法（桥函数形态先行，语法糖归后续需求批）。
- 不做跨线程回调（num_workers>0/GPU 异步——T02 硬约束维持，线程外回调=响亮报错）。
- 不做继承链解析/多继承/MRO 特判（type() 原生语义照单全收）。
- DataLoader 正式支持为**条件项**（探针先行：main-thread 迭代回调若通则纳入，
  不通则登记 divergence 并以手动批切金样收口）。

## 架构方案

```
.as 源                          Python 侧                       VM
──────────                     ──────────                     ────
py_subclass(base, "Net", {  ──桥──▶ type("Net", (base,), {      ──▶ 类句柄入堆
  forward: (self,x)=>{...}         "forward": PyCFunction(
})                                    new_closure(closure_id)
                                    )})
py_call0(Net) ─────实例化─▶ Net() 实例
py_call0(m, x) ──训练循环─▶ Net.forward(m, x) ──PyCFunction 闭包─▶ run_closure_bridged
                                                                        │ 弹 GIL 内重入
                                    ◀── 返回值 value_to_py ────────────┘ call_closure(
                                                                        task, id, N 实参)
```

- **单一真相源不变**：类语义全在 Python type() 机器上（桥只组装）；Auto 侧语义
  = 闭包体（与 py_with/py_callable 同构——回调桥复用，不新建通道）。
- **变参回调是唯一引擎级改动**：py_callable 现只喂 args[0]（py_ffi.rs:1402-1408），
  扩为全实参 marshal + `call_closure(task, id, n)`（call_closure 本就按实参数压帧，
  engine.rs:1742-1828）。
- **a2py 回译**：`py_subclass(base, "Name", {m: (self, ...) => {...}})` 规范形态 →
  真 Python 类定义（`class Name(base):` + `def m(self, ...):` + 闭包体语句发射，
  语句体发射机件 py_with 已有——DIV-PY-CLOSURE-1 的 set 降级不触达此路径）。

## 技术栈

Rust（crates/auto-lang/src/py_ffi.rs——变参回调 + py_subclass 桥(481) + 生存期守卫；
vm/engine.rs 零改动（call_closure 现成）；trans/python.rs——类发射回译）；
parity py_torch_custom 新套件（三方方法论沿用 461/539）。

## 需求分析与背景调查

（取材：KNOWN-DEBT P539-D4、roadmap §7.3、539 档案 T02/T21、2026-09-06 代码调研）

- **回调桥现状（全就绪件）**：py_callable(466) shim py_ffi.rs:1380-1426——
  `pop_arg_i32(closure_id)` → `PyCFunction::new_closure` → 槽守卫 →
  `run_closure_bridged`（:2020-2052）弹实参 → `vm.call_closure(task, id, 1)`
  → 返回值 `nv_to_value_local`+`value_to_py` 回 Python。thread-local 槽
  :1969-1972（BRIDGE_TASK/BRIDGE_VM）+ BridgeGuard RAII :2000-2015；空槽守卫单测
  :3152-3170。**约束**：GIL 线程==VM 线程（T02；:296/:1967-1968/:2029 三处成文），
  num_workers=0。
- **call_closure 可重入性**：engine.rs:1742-1828——同 task 压新帧（saved
  ip/bp/closure-id 五项 :1755-1759），run_one_instruction 循环至 bp==saved_bp，
  预算 1M 条（:1775），错误经 intercept_error（try/catch 可拦）。py_with 先例
  （py_ffi.rs:1157 调用）+ native.rs 14+ 处先例——嵌套回调（forward 里再 py_call）
  天然支持。
- **type() 工厂已实证**：单测 py_ffi.rs:3030-3046（type('KwBox', (), {...})() +
  kwargs 方法调用）——运行期 exec/类工厂代码为零新增风险面，纯组合。
- **基类句柄通道**：importlib 金样（train.as:22,34-35）或 py_module(471)
  （py_ffi.rs:1536-1564）+ py_getattr(451) 链式取 torch.nn.Module。
- **三个真缺口**：
  ① **变参实参**：现回调只 marshal args[0]（:1402-1408）——`forward(self, x)`
    的绑定调用 PyCFunction 收到 (self, x)，需全参透传（call_closure 第三参本就
    是实参数）；
  ② **dict 值封送限制**：Auto ObjectData→PyDict 值封送有 Plan-300 字符串化限制
    （train README:32-33）——闭包值（栈上 Int(id)，engine.rs:240）不能走通用
    dict 封送，需 py_subclass 桥**自读字段**逐个包 PyCFunction 后组 dict；
  ③ **闭包生存期**：Closure.env/capture_slots 按创建帧 bp 引用读值，创建帧消亡
    后无意义（engine.rs:232-237 TimerCallback 注释警告）；py_with 曾因句柄跨
    回调 RC-fragile（py_ffi.rs:1148-1154）——nn.Module 用法（闭包定义于 main、
    回调发于同帧训练循环）天然安全，但纪律须成文 + 守卫（回调时帧存活探针）。
- **组合式金样对标**：py_torch_train train.as——`py_call0(layer, x)` 直调、
  manual_seed(0) + 60 tick 收敛断言（lf < first_loss && lf < 1，:68-93）、TAP
  三方对照、`.len()` 规避注记（:17-18——**PLAN-569 落地后可直接用 .len()**，
  执行序：569 先、570 后，新套件用干净惯用法书写）。
- **DataLoader**：无先例（全仓唯一符号是无关 Rust 侧 data.rs:21）；约束注释在
  parity README:24-29。探针先行。
- **a2py 空白**：python.rs 只有 @dataclass class 发射（:1693-1761），无 Python
  基类派生面——本计划新增 py_subclass 匹配器（规范形态窄匹配，非规范形态
  emit_plain_call 兜底报 NameError 响亮）。

## 详细设计

### A. 变参回调（py_callable 通道升级，466 行为扩展）

`new_closure` 闭包体（py_ffi.rs:1395-1413）：`args` 全量遍历——每实参
`py_any_to_value` 入 `Vec<Value>`；`run_closure_bridged` 扩签
`(values: Vec<Value>)` 逐个压栈后 `call_closure(task, id, values.len())`。
单实参行为不变（map/apply_ 探针回归）。**kwargs 首批不传**（`_kwargs` 丢弃并
注释——Auto 闭包无 kwargs 形参面，Python 侧以 kwargs 调方法时 TypeError 照单全收）。

### B. py_subclass 桥（新 native 481）

`NATIVE_PY_SUBCLASS: u16 = 481`；shim 签名
`py_subclass(base, name: str, methods: ObjectData)`：
1. BridgeGuard 进入；弹 methods（TAG_OBJECT ObjectData）——**自读字段**：
   `fields: Vec<(String, Value)>`，每个 `Value::Int(closure_id)` 包
   `PyCFunction::new_closure`（与 py_callable 同机柄，抽出共享 helper
   `make_pycallback(closure_id) -> Py<PyAny>`）；
2. 组 `PyDict`（方法名 → PyCFunction）；`builtins.type` 句柄 call3
   `(name, (base,), dict)`——base 经 pop_auto_py_arg（句柄解引用 type 对象）；
3. 返回类对象包 PyObjectHandle 入堆。三处注册镜像（lib.rs/codegen 常量+id+
   py_native_map）+ a2py 臂。
对象字面量闭包字段：codegen 对 `{ m: (x) => {...} }` 的 ObjectData 字段存闭包
值——**验证点**（T02 探针）：字段是否存 Int(closure_id)（栈闭包值形态
engine.rs:240）；若发射为别的形态（如立即 py_callable 包裹），shim 兼容两种
（Int → 包；PyObjectHandle(PyCFunction) → 直用）。

### C. self 约定与生存期纪律

- **self**：type-dict 函数被实例调用 → Python 自动绑 self 首参 → 变参回调后
  `(self, x)` 全参入 Auto 闭包——闭包形参表写 `(self, x)` 即可，无桥侧特判。
- **生存期守卫**：make_pycallback 捕获 `closure_id`；回调执行时 run_closure_bridged
  现有空槽守卫之外，加**帧存活探针**（捕获创建时 task.bp 下界，回调时 task 栈深
  不小于该界——越界即 "closure creator frame gone (lifetime violation)" 响亮报错）。
  探针为防御面；纪律成文：闭包定义域必须存活于全部回调窗口（README Callback
  bridge 段扩写）。
- **num_workers=0**：维持 T02 约束三处成文不动；线程外回调现行空槽守卫已拦。

### D. a2py 类发射

`match_py_subclass_form(call)`：窄匹配 `py_subclass(base, "Name", ObjectLit)`——
发射：

```python
class Name(<base>):
    def m(self, ...):      # 闭包形参直映（首参 self 语义同）
        <闭包体语句发射>    # 复用 py_with 语句体发射机柄
```

非规范形态（非 ObjectLit/非字面量名）→ emit_plain_call 兜底（Python 侧
NameError: py_subclass——响亮）。快照单测 ×3（单方法 forward / 双方法
Module+Dataset 形态 / 非规范兜底）。

### E. py_torch_custom 三方套件

`parity/libs/python/py_torch_custom/`（README + tests/python/test_custom.py +
tests/auto/custom.as）：
1. 自定义 `Net(nn.Module)`：`__init__` 用 Python 侧 setattr 组装（fc=Linear(2,1)，
   Auto 侧 py_call m "__setattr__"）或组合 `nn.Sequential` 参数——**探针定形**
   （__init__ 闭包回调 vs 实例后 setattr，取简）；forward 闭包（self 通道）；
   seed(0) 训练 60 tick 收敛断言（对标 train.as 形态）。
2. 自定义 `DS`：`__len__` 返回定数、`__getitem__` 返回 `[x, y]` 对；Auto 侧
   for-in 或索引消费（569 后 `.len()` 可用）。
3. DataLoader 探针（条件项 T12）：`DataLoader(ds, batch_size=4)` main-thread
   迭代——回调同线程应通；不通（异步路径）→ 登记 divergence + 手动批切金样。
4. phase 注册 p9（torch 族）。

### F. 债务核销与回写

P539-D4 ✅（套件实证）；roadmap §7.3/§7.4 状态回写；parity README Callback
bridge 段扩写（类工厂面 + 生存期纪律）。

## 测试设计

- **py_ffi 单测**：make_pycallback 共享 helper；py_subclass 桥——纯 Python 基类
  （type 直建）+ 方法回调值往返（不含 torch，快）；变参回调（双参闭包
  `(a,b) => a+b` 经 Python 调用）；空槽守卫回归；帧存活探针触发例。
- **tv .as 语料**（99_script_err 族或 99_py_dispatch 类目共用）：py_subclass 纯
  Python 形态端到端（无 torch 依赖——type 直建基类 + 方法回调 + 实例调用打印）。
- **a2py**：py_subclass 类发射快照 ×3。
- **parity**：py_torch_custom 三方绿；p5-p9 全量复跑零回归（torch 导入慢——p9
  相位本就含）。
- **合入前**：`cargo tf` 一次 + `cargo tt`。

## 验收标准

1. 变参回调：双参 Auto 闭包经 Python 侧调用返回正确值；map/apply_ 单参探针
   行为零变化。
2. `.as` 内 py_subclass 组自定义 nn.Module：实例化 + forward 回调（self 通道）
   + seed 化训练收敛断言三方一致（AutoVM ≡ a2py 类发射 ≡ Python 原生类）。
3. Dataset 形态 `__len__`/`__getitem__` 回调可用；DataLoader 探针结论成文
   （通→纳入用例；不通→divergence 登记 + 手动批切金样）。
4. 生存期守卫：创建帧消亡后回调触发 → 响亮错误（单测）；空槽守卫回归绿。
5. p5-p9 全量零回归；`cargo t`/`tv`/`tt` 零新增红（对照基线）；合入前
   `cargo tf` 绿。
6. KNOWN-DEBT P539-D4 ✅（附链接）；roadmap §7.3/§7.4 回写；parity README
   Callback bridge 段扩写。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [ ] **T01** 变参回调：`crates/auto-lang/src/py_ffi.rs` new_closure 闭包体全参
  marshal + run_closure_bridged 扩签（Vec<Value>）+ 单测（双参往返/单参回归）。
  验证：`cargo test -p auto-lang --lib --features python py_callable`。
- [ ] **T02** 对象字面量闭包字段形态探针：scratch .as `{m: (x)=>{...}}` 存取 +
  dump——确定 ObjectData 字段携带闭包 id 形态（Int vs py_callable 包裹）。
  验证：探针输出（形态结论写入本计划执行注记）。
- [ ] **T03** make_pycallback 共享 helper 抽取：py_callable shim 内联机柄抽
  `fn make_pycallback(py, closure_id) -> PyResult<Py<PyAny>>`（含帧存活探针捕获）。
  验证：`cargo test ... py_callable` 回归。
- [ ] **T04** 帧存活探针：run_closure_bridged 加创建帧 bp 下界守卫 + 触发单测。
  验证：同 T01。
- [ ] **T05** py_subclass 桥：NATIVE_PY_SUBCLASS=481 常量 + shim（自读字段组
  PyCFunction dict + type() call3 + 类句柄入堆）+ 纯 Python 基类单测（type 直建，
  方法回调值往返）。
  验证：`cargo test ... py_subclass`。
- [ ] **T06** 三处注册镜像 + py_native_map：lib.rs / codegen.rs（常量+id+列表）。
  验证：`cargo check -p auto-lang`。
- [ ] **T07** tv 语料：99_py_dispatch（或新类目）py_subclass 纯 Python 形态端到端
  例（type 直建基类 + 实例调用打印回调返回值）。
  验证：`cargo test ... test-vm-files,python` 新例绿。
- [ ] **T08** a2py 类发射：trans/python.rs match_py_subclass_form + 快照 ×3
  （forward 单方法/双方法/非规范兜底）。
  验证：`cargo tt`。
- [ ] **T09** py_torch_custom 套件骨架：README + test_custom.py + custom.as
  （Net 形态：探针定 __init__ 组装法 → forward 闭包 → seed 训练收敛断言）。
  验证：`cd parity && cargo run -- run py_torch_custom`（首版三方对照输出）。
- [ ] **T10** Dataset 形态用例：DS 类（__len__/__getitem__）+ 消费路径（569 已并
  则 `.len()`，否则 for-in）。
  验证：套件增量用例三方绿。
- [ ] **T11** DataLoader 探针（条件项）：batch_size=4 main-thread 迭代；通→用例
  纳入，不通→known-divergences 登记 + 手动批切金样用例。
  验证：探针结论 + 套件绿。
- [ ] **T12** phase 注册 p9 + 全 parity 复跑（p5-p9 五相位零回归）。
- [ ] **T13** 债务核销 + 回写：KNOWN-DEBT P539-D4 ✅；roadmap §7.3/§7.4；
  parity README Callback bridge 段扩写（生存期纪律）。
- [ ] **T14** 门禁：`cargo t --no-fail-fast` 基线对照 + `cargo tv` + `cargo tt`
  + 合入前 `cargo tf`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

1. **__init__ 组装形态**：自定义 Module 的层组装——Python 侧 `__init__` 闭包
   回调（Auto 写构造逻辑）vs Auto 侧实例化后 `__setattr__` 逐层装配（桥更薄）。
   推荐**探针后取简**（T02/T09 定形；倾向 setattr 装配——__init__ 回调涉及
   super().__init__() 深水区）。
2. **kwargs 回调通道**：Python 以 kwargs 调 Auto 方法时首批丢弃（TypeError 照单
   全收）——若套件需要 kwargs 方法再议（桥面 kwargs→命名实参映射需闭包形参名
   内省，超首批预算）。
3. **闭包捕获语义**：Auto 闭包捕获创建帧槽位（按 bp 引用）——捕获可变局部并在
   回调中改写的行为（改的是帧槽）是否成文为"回调可观察创建帧状态"纪律？
   推荐 README 成文即可（与 W2 套件数据活 Python 侧约定互补）。
4. **执行序依赖**：本计划 T10 假定 PLAN-569 已并（`.len()` 干净惯用法）——若
   569 未并则 T10 用 for-in 规避（不阻塞，仅惯用法差异）。
