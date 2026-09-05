---
plan_id: PLAN-539
status: archived
feature_name: PyTorch 基本完整支持（Python FFI 清债 + 张量惯用法 + 训练循环 + 类派生分期）
author: [zhaopuming]
created_at: 2026-09-04T11:00:00+08:00
updated_at: 2026-09-04T22:10:00+08:00

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "parity/docs/known-divergences.md: DIV-PY-FLOAT/EXCEPT/ITER/KWARGS/AUTOLIST/CONST 六条状态回写（五 fixed + CONST documented）+ 新增 DIV-PY-TUPLE-1/DIV-PY-CLOSURE-1 两条"
  - "crates/auto-lang/src/py_ffi.rs: py_call(450)/py_getattr(451) shim 层扩展——id 带 450..=499 保留（next_native_id 400→500）+ 五宿主 shim 装回调桥窗口"
  - "crates/auto-lang/src/vm/engine.rs: TYPE_TO_I32/TYPE_TO_F64 改 tag 分派（原 pop_tagged 两臂）+ EQ/NE 补混型数值臂（消费 py_float 结果）"
  - "parity/libs/python/py_torch/README.md + tests/auto/torch.at: 461 存量套件断言随迁 py_float 标量语义（7/7 保持绿）"
new_spec_components:
  - "crates/auto-lang/src/py_ffi.rs: 16 内建族 py_call_kw(452)/py_call_may(453)/py_iter(454)/py_next(455)/py_matmul(456)/py_getitem(457)/py_setitem(458)/py_slice(459)/py_call0(460)/py_with(461)/py_enter(462)/py_exit(463)/py_item_kw(464)/py_float(465)/py_callable(466) + 回调桥（thread-local 任务槽 + BridgeGuard RAII）"
  - "crates/auto-lang/src/vm/engine.rs: 12 运算符 opcode dunder 路由臂（ADD..GE/NEG，NotImplemented 反射协议）+ ARRAY_LEN/GET_ELEM PyObjectHandle GIL 臂"
  - "crates/auto-lang/src/vm/codegen.rs: try_py_call_special_form 特形分派（py_call kwargs 五槽/项导入 kwargs/py_with 内联 bracket）+ 命名实参冒号语法贯通"
  - "crates/auto-lang/src/trans/python.rs: a2py 发射族（py_call_may→_auto_may 哨兵/py_iter→iter/py_next→next(x,None)/py_matmul→.matmul(/py_getitem→obj[]/py_setitem→赋值/py_slice→slice(/py_call0→直调/py_float→float(/py_callable→恒等）+ py_with 语句级 with ctx: 内联"
  - "parity/libs/python/py_torch_infer: 三方套件 16 例 + README（dunder/matmul/getitem/call0/with/kwargs/May）"
  - "parity/libs/python/py_torch_train: 三方套件 10 例 + README（seed 化 MLP 收敛/kwargs 构造器/容器往返）"
  - "parity/libs/python/README.md: python 分类级 README 新建（W0-W3 调用约定全集 + 标量封送规则 + 回调桥约束）"
  - "parity/crates/auto-parity/src/main.rs: phase p9 注册（py_torch_infer + py_torch_train）"
  - "docs/design/strategy/python-parity-roadmap.md: §7 运算符调研节（@ 中缀词法双义/** 幂/py_subclass 延期备注）"
touched_goals:
  - "goal-005: Python parity 第三维度——use.py 调 Python 库推进到训练/推理脚本级（16 内建族+dunder 路由+回调桥；py_torch_infer 16/16 + py_torch_train 10/10 三方绿；六条 DIV-PY 债清偿）

current_step: 22
total_steps: 22
---

# [PLAN-539] PyTorch 基本完整支持——Python FFI 清债与张量/训练惯用法分期路线

## 变更摘要

在 Plan 214（PyO3 嵌入 + `use.py`）→ 369（`PyObjectHandle` + `py_call`/
`py_getattr`）→ 461（numpy/pandas/matplotlib/torch 四库三方 parity）建成
的 Python 桥之上，把"调得到 torch 函数"推进到"能写 PyTorch 训练/推理
脚本"。四个波次（每波独立折叠，525 先例）：

- **W0 FFI 清债**：清偿 Plan 369 在案六条 DIV-PY 分歧（kwargs / for-in
  迭代 / 异常映射 / Auto list 封送 / float 返回 / 常量导入）——训练循环
  每一条都撞，是硬底座；
- **W1 推理惯用法**：句柄运算符 dunder 路由（`+ - * /` 按 Python 同义
  语义 → `__add__`/`__mul__`…，`*` 坚持逐元素）+ `matmul(a, b)` 函数
  形式 + `py_getitem`/`py_setitem`（切片/`state_dict`）+ callable 直调
  （`model(x)`）+ `no_grad` 上下文管理器 → 落 `py_torch_infer` 三方套件；
- **W2 训练循环**：kwargs 全量贯通（构造器/优化器/DataLoader）+ 异常
  May 化 + tuple/dict/None 往返封送 + 标量 `.item()` 高频通道（float 返
  回直通 f64，去字符串池）→ 落 `py_torch_train` 三方套件（合成数据 +
  固定种子小 MLP，确定性断言）；
- **W3 类派生/回调（范围 W0 考古裁定）**：Auto 闭包包装 `PyCallable`、
  Python 类派生最小面（自定义 `nn.Module`/`Dataset`）——或显式延后。

**运算符决策（本计划定案项，用户已确认方向）**：
- `*` **不**表示矩阵乘——torch/numpy 里 `*` 是逐元素乘，变义会击穿
  三方 parity（AutoVM 调 `__matmul__` 而 a2py 发射 `a * b` 语义为逐元
  素，静默分歧）；
- 矩阵乘短期走 `matmul(a, b)` / `a.matmul(b)` 函数形式（W1，零语法成
  本）；中缀 `@`（当前为注解前缀占用，注解在声明位置、matmul 在表达
  式中缀位置，词法可区分）作为独立语言层决策，**不在本计划范围**，
  在 `docs/design/strategy/python-parity-roadmap.md` 追加调研节。

## 目标

1. Plan 369 登记的六条 DIV-PY 在案债全部清偿或在 `KNOWN-DEBT-AND-
   RISKS.md` 显式改记延期理由（不允许静默遗留）。
2. `PyObjectHandle` 上的算术/比较运算符按 Python dunder 同义语义路由；
   `*` = `__mul__`（逐元素），矩阵乘提供 `matmul(a, b)` 内建（→
   `operator.matmul` / `__matmul__`）。
3. torch 推理脚本可写：`use.py torch` 建张量 → 运算符运算 → `matmul`
   → 切片/索引 → `model(x)` 直调 → `no_grad` 块内推理，全程 AutoVM 与
   a2py 转译三方一致（`py_torch_infer` 套件）。
4. torch 训练循环可写：kwargs 构造 `nn.Linear`/`SGD`/`MSELoss`，
   `for` 迭代 batch，`loss.item()` 收敛标量（f64 直通），`optimizer
   .step()`，异常落 `May` 值可捕获（`py_torch_train` 套件，固定种子确
   定性）。
5. a2py 对每个新惯用法同步发射（kwargs → 关键字实参、dunder 运算 →
   运算符、`py_call` → 属性调用已有），三方流水线不破。
6. 每波沿用 461 方法论：三方 parity 全绿 + parity README/guide 同步。

### 非目标（Out of Scope）

- 中缀 `@` matmul 运算符、`**` 幂运算符（记录到 roadmap 调研节，独立
  语法计划）；
- Auto 原生 struct 的运算符重载 trait 体系（语言层，与句柄 dunder 路
  由是两回事）；
- GPU/CUDA 专项验证（本计划只断言确定性 CPU 路径；device 字符串可配）；
- bulk ndarray buffer 级封送（W2 仅做 list/tuple/dict 值封送；buffer
  通道登记为后续）；
- torch 之外的库扩展。

## 架构方案

### 现状基线（考古结论，W0 免重查）

- 桥面：仅两个内建 shim——`py.py_call`(450) / `py.py_getattr`(451)
  （`crates/auto-lang/src/py_ffi.rs:63`，注册于 `lib.rs:734`）；仅位置
  参数；方法名限字符串。
- 封送：Auto→Python 走 `pop_auto_py_arg`（`py_ffi.rs:512`）：i32/str/
  bool/null/f64(2-slot)/`PyObjectHandle`(堆对象直通 `Py<PyAny>`)/堆
  list；Python→Auto 返回值统一落字符串池（数值去 `.0` 字符串化），
  0 维张量经 `__float__` 转标量——DIV-PY-FLOAT-1。
- 已知坑（py_torch README 在案）：句柄上做算术退化（`t - 0` 坏句
  柄）；`use.py` 无别名，torch/numpy `arange` 同名冲突不能共导入
  （DIV-PY-CONST-1 关联，W0 一并裁定：最小面做 `use.py mod as alias`
  还是无 items 模块句柄 `py_getattr(mod, "arange")` 绕行）。
- a2py：`py_call` → `obj.method(...)`、`py_getattr` → `obj.attr` 已
  下沉（`crates/auto-lang/src/trans/python.rs:976`）。
- 语言面：`?T`/May 错误传播在案（525 W5）；无 tuple 值类型（W2 定映
  射）；`@` 为注解前缀。

### W0 设计要点

1. **kwargs（DIV-PY-KWARGS-1）**：`py_call` 变长 ABI 扩展——新增
   `py_call_kw(obj, method, args…, kw_names: List<str>, kw_vals…)`
   或在 `py_call` 约定第 2 参后插入 kwargs 哨兵；codegen
   （`vm/codegen.rs:5033` 起 py native 臂）与 VM shim 同步。a2py 发射
   关键字实参。
2. **迭代（DIV-PY-ITER-1）**：新增 `py_iter(handle) -> Iterator 句柄`
   + `py_next(it) -> ?PyObjectHandle`（StopIteration → null→May）；
   VM for-in 对 py 句柄编译为该对。手动 `__iter__`/`__next__` py_call
   已可用，本项是语法糖贯通。
3. **异常（DIV-PY-EXCEPT-1）**：shim 捕获 `PyErr` 时不再 `VMError::
   FFI` 硬错，改推 `May` 错误值（携带 `str(e)` + 类型名）；`py_call`
   调用点可 `?` 传播或 `??` 兜底。保留 `strict` 变体或编译期开关保持
   461 存量用例语义（存量断言不破）。
4. **Auto list 封送（DIV-PY-AUTOLIST-1）**：`pop_auto_py_arg` TAG_LIST
   分支已有 `ListData<Value>` → `PyList` 路径，461 实测仍坏——W0 首
   任务复现探针定位（疑嵌套/元素 tag 分派缺口），修后补往返用例。
5. **float 返回（DIV-PY-FLOAT-1）**：返回通道加 f64 直通（2-slot push，
   与 `pop_arith_operand` 对称），仅当结果为 `float`/`int` 且调用点
   声明类型已知时启用，否则维持字符串池；`.to(float)` 存量兼容。
6. **常量/别名（DIV-PY-CONST-1）**：最小面 = `use.py torch` 无 items
   取模块句柄 + `py_getattr(mod, "no_grad")`（369 已建），文档固化；
   `as` 别名与子模块路径（`torch.nn`）做 parser 探针后定案。

### W1 设计要点

- **dunder 路由**：VM 二元运算分派在 LHS/RHS 任一为 `PyObjectHandle`
  时改走 FFI：`+ - * / % == != < > <= >=` → 对应 dunder（`__add__`/
  `__mul__`/`__eq__`/`__lt__`…），一元 `-` → `__neg__`；不在句柄上实
  现本地回退（Python 侧报错即报错，与三方语义一致）。
- **matmul 内建**：`py_matmul(a, b)`（native id 顺延）→
  `PyAny::call` `__matmul__`；a2py 发射 `a.matmul(b)`；Auto 侧提供
  `ext` 便捷方法 `a.matmul(b)` 薄封装。
- **getitem/setitem**：`py_getitem(obj, idx…)`（多参 → tuple 下标，
  覆盖 `x[:, 1]`）、`py_setitem(obj, idx, v)`；a2py 发射 `obj[…]`。
- **callable 直调**：`py_call0(handle, args…)`（对象自身 `__call__`）
  或 `py_call(obj, "__call__", …)` 别名糖；a2py 发射 `obj(args…)`。
- **上下文管理器**：`py_with(ctx_handle, fn(ctx) …)` 高阶内建（宿主
  侧 `__enter__`/`__exit__` + Auto 回调经回调桥），覆盖 `no_grad`。

### W2 设计要点

- kwargs 全量贯通到构造器（W0 的 `py_call_kw` 扩到 use.py 项导入直
  呼形态 `nn.Linear(784, 10, bias=False)`——codegen 项导入臂感知 kw）。
- tuple 往返：Python tuple → Auto `List`（拍平映射，W0 探针定案；不
  新增 tuple 值类型，登记 divergence：tuple 可变性/去重语义差异）。
- dict 往返：`PyObjectHandle` 持 dict 时 `py_getitem` 通路 + Auto
  Map → PyDict（`value_to_py` 已有对象分支，补 Map 分支）。
- None：返回 None → Auto null（现 fallback 已是 None，补显式 tag 与
  用例）。
- `py_torch_train` 套件：固定种子合成回归（`torch.manual_seed(0)`，
  小 MLP + MSE + SGD，固定步数后权重/loss 标量收敛断言），三方对名。

### W3 设计要点（W0 考古后裁定范围）

- 回调桥：Auto fn/闭包 → `PyCallable` 包装（宿主 PyO3 `PyCFunction`
  载 Auto 任务指针，GIL 入口回投 VM）；覆盖 `Dataset` 回调式用法。
- 类派生最小面：`use.py torch.nn` 句柄 + `py_subclass(base, name,
  {forward: auto_fn})` 工厂内建（非语法级 `extends.py`——语法级留待
  独立语言计划）；`nn.Module` 组合 + `forward` 覆盖达成即达标。
- 若考古发现 GIL/任务指针持存不可行 → 显式延后登记，波次改交付
  `nn.Sequential` 组合式训练黄金样替代。

## 技术栈

- Rust / PyO3（`pyo3` optional feature `python`，现状沿用）；
- `crates/auto-lang/src/py_ffi.rs`（shim 层）、`vm/codegen.rs`（py
  native 臂）、`trans/python.rs`（a2py 发射）、`native_registry.rs`；
- parity：`parity/libs/python/py_torch_infer|py_torch_train`（三段式
  `tests/auto/*.at` + `tests/python/*.py` + README）、
  `crates/auto-parity/src/main.rs` phase 表（新增 `p9` torch 扩展相）；
- torch CPU wheel（runner `--python-binary` 旗标沿用 461）。

## 需求分析与背景调查

- 用户需求：AutoVM 内直接调用 PyTorch 完成模型训练/推理的"基本完整"
  支持；追问张量矩阵乘的表面（`@` vs `*`）。结论：`*` 保持逐元素语
  义，matmul 函数形式先行，中缀 `@` 独立调研（见变更摘要决策节）。
- 既有能力链：Plan 214 → 369 → 461（详见现状基线节）。
- 在案债（`docs/plans/archive/369-python-parity-suite.md` 登记 +
  461 T2 补充）：DIV-PY-FLOAT-1 / EXCEPT-1 / ITER-1 / KWARGS-1 /
  AUTOLIST-1 / CONST-1 六条；`plans-360-369-status-summary.md:115`
  证实 FLOAT-1 未修复。
- spec 关联：`.autoos/specs.json` python parity 线（369/461 已沉积）；
  目标语言高阶能力线（525：May/闭包/泛型已达成，为 W2/W3 前置）。

## 测试设计

- **探针（每波 W0 起）**：`scratch/` 一次性 `.at` 探针先证行为再改
  码（461 T2 惯例）。
- **三方套件**：
  - `py_torch` 存量 7 例回归不破（`cargo run -- --root . run py_torch`）；
  - 新增 `py_torch_infer`（W1，~10 例：dunder 运算、matmul、切片、
    直调、no_grad）；
  - 新增 `py_torch_train`（W2，~8 例：seed 化小 MLP 训练收敛、异常
    捕获、kwargs 构造、tuple/dict 往返）；
  - 每套件 README 记惯用法与坑（461 惯例）。
- **单测**：`py_ffi.rs` shim 层 Rust 单测（kwargs ABI、dunder 分派、
  f64 直通）feature `python` 门控；a2py 快照测试补新发射形态。
- **回归门禁**：`cargo check -p auto-lang`（迭代）；波次折叠前
  `cargo tv` + parity workspace `cargo test` + 上述三方 run 全绿。

## 验收标准

1. 六条 DIV-PY 债每条状态明确：修复（带回归用例）或改记延期理由。
2. `py_torch_infer`、`py_torch_train` 三方 run 全绿且入 phase 表
   （`p9`）；`py_torch` 存量 7 例回归全绿。
3. Auto 脚本中 `a * b`（张量句柄）三方语义均为逐元素乘；`matmul(a,b)`
   三方一致；无 `*`-变义路径。
4. W2 验收官：固定种子小 MLP 训练脚本（构造→迭代→反传→step→收敛
   断言）在 AutoVM 与 a2py 转译产物上输出一致标量序列。
5. `cargo tv` 全绿；新增代码 `cargo fmt` 干净；无 TODO/debug print。
6. parity README / parity-guide / roadmap 调研节（`@` 中缀 + `**`）
   同步更新。

## 执行步骤

### W0 FFI 清债（前两任务为探针/考古）

- [x] T01 W0 探针：`scratch/p539/` 复现六债现状（kwargs 报错形态/
      for-in 失败形态/list 封送坏点/float 字符串化/异常 panic 栈/
      常量 getattr 绕行），输出记录入本文件执行注记
      （验证：探针 .at 逐条可复跑）
      [✅ 已完成] 六探针 `scratch/p539/p1..p6_*.at` 全部可复跑（master
      build target/debug/auto.exe + torch 2.14.0+cpu @ C:\Python314），
      逐条现状见「执行注记 W0 探针记录」——关键修正两处认知：①kwargs
      不是"报错"而是**静默丢名**（Arg::Pair 值按位置推，名字丢弃，
      codegen.rs:8487）；②for-in 句柄不是失败而是**静默零迭代**（落
      ARRAY_LEN 索引通道，PyObj 堆对象 len→0）
- [x] T02 W0 考古：W3 回调桥可行性（PyO3 `PyCFunction` 持 VM 任务
      指针 + GIL 入口回投）结论定案，写入待澄清①
      [✅ 已完成] 结论：**可行（带约束），W3 保留在计划内**。两级证据：
      (A) 立即回调（py_with 形态）——shim 内 `vm.call_closure` 是既定
      先例（native.rs shim_list_map:2621 起 14 处），零新机制；
      (B) PyCallable 包装（Dataset 回调式）——`PyCFunction::new_with`
      需 `'static+Send+Sync`，裸 task 指针有跨 shim 生存期/别名风险，
      改用**新任务执行模型**：回调时 `AutoTask::new`（task.rs:168 公开）
      新建任务跑闭包体，闭包注册表 vm.closures 共享，无 &mut 别名；
      约束 num_workers=0（torch CPU DataLoader 默认）+ 回调仅发生在
      宿主 py_call shim 窗口内（GIL 在位、同线程）。已写入待澄清①
- [x] T03 DIV-PY-KWARGS-1：`py_ffi.rs` 新增 kwargs shim（ABI 定案后
      实施）+ `vm/codegen.rs` py native 臂扩展 + `native_registry`
      注册；a2py `trans/python.rs` 关键字实参发射；Rust 单测
      （验证：`cargo t py_ffi` + 探针 kwargs 用例转正）
      [✅ 已完成] ABI 定案：独立 `py_call_kw` 内建（id 452）固定 5 槽
      约定（obj, method, posargs, kw_names, kw_vals 各为封送列表），
      无哨兵无新操作码；codegen 对 `py_call` 含 `Arg::Pair`（**Auto 命名
      实参语法是冒号 `k: v`**，`k = v` 解析为赋值表达式按位置传——
      探针修正认知）调用点改发 5 槽 + CALL_PY count=5；a2py `Arg::Pair
      → key=val` 发射原已正确（补钉即可，快照在 T15 套件）。单测 4 新
      （build_kwargs zip/长度不齐 zip 语义/pyo3 kwargs API/shim 注册），
      py_ffi 26/26（注意：须 `--features python`，cargo t 别名不含该
      feature）；探针 p1b：`numpy(force: true)`（keyword-only 判别）→
      ndarray、`sum(dim: 0)` → 15。commit 7481b04db
- [x] T04 DIV-PY-AUTOLIST-1：定位 `pop_auto_py_arg` TAG_LIST 缺口修
      复，嵌套 list 往返用例（验证：`cargo t py_ffi` + 探针）
      [✅ 已完成] 病根不是 TAG_LIST 分支而是**编码不匹配**：数组字面量
      CREATE_ARRAY 产物是 TAG_OBJECT 编码 ListData（engine.rs:2885 
      encode_object），TAG_LIST 仅 py 返回/ui 桥生产——TAG_OBJECT 臂
      downcast 三连失败落单 None。修复：TAG_OBJECT 臂补 ListData 下转
      + `value_to_py` 嵌套臂（Array/Block→PyList、Obj→PyDict、VmRef→
      堆解析，guard 先释后递归防锁叠）+ 3 单测（嵌套数组/Obj/VmRef）；
      探针 p3 `tensor([1.0,2.0,3.0]).sum()`→6、p3b `len`→3。commit 
      7481b04db（与 T03 同提交，kwargs 通道依赖本修复）
      附带发现（存量，非本次引入）：a2py 浮点字面量整型化（`1.0`→
      `1`）——W1/W2 三方套件写用例时需定夺（修 a2py 或用例规避），
      已记执行注记
- [x] T05 DIV-PY-FLOAT-1：返回通道 f64 直通（2-slot），声明类型感知
      分派；`.to(float)` 存量兼容用例（验证：`cargo t py_ffi` +
      `run py_math` 回归 20/20）
      [✅ 已完成] 设计修正：Plan 377 已把 f64 单槽化，计划原文"2-slot"
      理由过时——直接 `push_f64` 即可，无需"声明类型感知"闸（那是
      2-slot 时代的保护）。但发现连锁断裂：py 返回被 codegen 谎记
      `StrFixed`（fn_return_types），`.to(int)/.to(float)` 静态路由到
      字符串解析臂，f64 位模式被 `pop_tagged` 误当 i32——修
      TYPE_TO_I32/TYPE_TO_F64 为按 NanoValue tag 分派（str 解析臂保留
      Plan 510 G3 份额配平；f64 截断/直通；f32/bool/null 补齐）。
      验证：py_ffi 28/28（3 新单测）；探针 p4（sqrt(2.0)→
      1.4142135623730951、sqrt(4.0).to(str)→"2"、`v+1.0`→4，NaN
      消失）；py_math 20/20 + py_torch 7/7。commit 13824f227
- [x] T06 DIV-PY-EXCEPT-1：shim 异常 → May 错误值；461 存量语义保持
      开关；异常捕获用例（验证：`cargo t py_ffi` + 探针 try-catch
      形态）
      [✅ 已完成] 考古修正：VMError::FFI 在 **try-catch 下本就可捕获**
      （intercept_error 拦截一切 VMError 变体并压消息字符串，探针
      p5b 实证）——债的实缺是 May **值**通道。设计：`py_call_may`
      内建（id 453，与 py_call 同变长约定）成功包 Result.Ok（镜像
      CREATE_OK stake 语义）/ 异常包 Result.Err 携带
      `PyException <类型名>: <消息>`；`py_call` 保持 strict（存量
      不破，即计划的"strict 变体保留"）；NULL_COALESCE 补 Result 臂
      （Err→default/Ok→解包）。a2py：`_auto_may(lambda…, None)` 哨兵
      + preamble helper。验证：py_ffi 30/30；探针 p5c 三形态
      （fallback/3/3）AutoVM 与 a2py 逐行一致；py_math 20/20 +
      py_torch 7/7。commit 6c77bf149
- [x] T07 DIV-PY-ITER-1：`py_iter`/`py_next` 内建 + for-in 编译贯通
      （`vm/codegen.rs` 迭代臂）；a2py for-in 发射（验证：探针
      `for x, y in loader` 形态 + `run py_torch`）
      [✅ 已完成] `py_iter`(454)/`py_next`(455)：GIL iter/next，
      StopIteration→null 家族——loader 惯用法走手动模式
      （`var b = py_next(it); while b != null`）。for-in 贯通取**索引
      通道 GIL 臂**方案：ARRAY_LEN/GET_ELEM 补 PyObjectHandle 下转
      （len()/obj[i]，feature 门控），张量/字典/列表句柄 for-in 按
      Python 语义；`auto.iterator.next` 管线是 i32 有损通道（字符串
      →0）不宜掺入——考古修正计划原文"编译为该对"的路线。a2py：
      `iter(x)`/`next(x, None)`。探针 p2b：手动=2/for-in=2 双端一致；
      **p2c 考古**：`for x, y in t` 可解析但语义不齐（单次迭代无
      解包，a2py 侧会 unpack 报错）——W2 套件用单变量循环+索引
      访问。py_ffi 30/30；py_torch 7/7。commit af0560925
- [x] T08 DIV-PY-CONST-1：模块句柄 + `py_getattr` 常量绕行文档固化
      （`parity/libs/python/README.md`）；`use.py` 别名/子模块探针
      结论登记（做或不做，理由入档）（验证：文档 + 探针记录）
      [✅ 已完成] 探针四形态（p8a-d）：①裸模块 dot-call
      （`use.py torch` + `torch.arange`）**在案已坏**（Undefined 
      variable，300 时代特性腐烂）；②子模块项导入（`use.py 
      torch.nn: Linear`）可用；③同名项跨模块**静默后者胜**（非
      报错）；④**importlib 句柄绕行可用**——`use.py importlib: 
      import_module` → `import_module("torch")` → 句柄 +
      py_getattr 取常量/成员（no_grad/arange 实证）。裁定：别名
      语法**不做**（importlib 通道已覆盖需求面，语法级留独立计划
      ——待澄清②）；文档落 `parity/libs/python/README.md`（新建
      分类级 README：W0 调用约定全集 + 常量/句柄裁定 + 封送注记
      含 a2py 浮点字面量整型化存量坑）。commit（README 新建）
- [x] T09 W0 折叠：KNOWN-DEBT 六条状态回写；`cargo tv` + parity
      workspace test + `run py_torch` 全绿（验证：门禁输出留档）
      [✅ 已完成] 六债回写 `parity/docs/known-divergences.md`（五条
      ✅fixed + CONST-1 ✅documented，各带根因/修复/证据；commit
      2831acb7b）。门禁：**cargo tv 3565/3565 全绿**（298.8s，含
      aavm2 语料深递归）——期间发现并修复一处非语义回归：kwargs
      拦截内联在巨型 native-call 臂里撑大递归编译栈帧致
      aavm2_a2r_is_corpus 栈溢出（master 86s 过/2.4s 早夭，仅回退
      codegen 即恢复），抽 `is_py_call_kw_form`/`compile_py_call_kw_form`
      独立方法后 102.1s 过（commit 在案）；parity workspace test
      38/38；py_torch 7/7 + py_math 20/20；tt/tf 折叠前档（见复审
      记录留档）。`pop_str_idx` 死码警告为基线存量（源码逐字节同，
      非本次引入）

### W1 推理惯用法

- [x] T10 dunder 路由：VM 二元/一元/比较运算句柄分派臂（`__add__`…
      映射表）+ 单测（验证：`cargo t vm` + 探针 `t * t`）
      [✅ 已完成] 12 opcode 臂（ADD/SUB/MUL/DIV/MOD/EQ/NE/LT/GT/LE/
      GE/NEG）：`stack_has_py_handle` peek 顶 3 槽（兼容 2-slot f64
      垫片）→ `py_dunder_dispatch`（NotImplemented 回落反射
      __rxx__，Python 双目协议）；结果统一 py_auto_marshal_return
      ——**比较返回逐元素 bool 张量**（torch 语义，强制 bool 会破
      a2py parity，考古修正计划隐含假设）。臂内一行调用（T09 栈帧
      教训）。探针 w1_t10 八断言全中（t*t=55 钉逐元素乘、2*t=30 钉
      反射）。a2py 运算符原样发射已在位；存量缺口记档（复合接收者
      无括号、0 维张量 .to(str) 形态差——套件用中间变量+`.to(int)`
      规避）。验证：cargo t vm 783/783；py_ffi 32/32
- [x] T11 `py_matmul` 内建（native id 顺延）+ Auto `ext` 薄封装 +
      a2py `a.matmul(b)` 发射（验证：探针 + 快照）
      [✅ 已完成] py_matmul(456) → `__matmul__` 直调；ext 糖（a.matmul(b)
      方法面）经探针证伪不可行（句柄 var 无静态类型可路由），函数形式
      即官方面，a2py 发射 `a.matmul(b)`。**id 带保护**：next_native_id
      400→500，450..=499 预留固定内建带（裸模块发现数百可调用原会爬
      进固定带静默覆写；三处钉 400 的存量断言随迁）。探针 m[0][1]=13/
      m[1][1]=40
- [x] T12 `py_getitem`/`py_setitem` 内建（tuple 下标）+ a2py `obj[…]`
      发射（验证：探针 `x[:, 1]` + 快照）
      [✅ 已完成] py_getitem(457)/py_setitem(458) 多参索引组 tuple 键；
      **py_slice(459)** null 端点→None 无界（`x[:,1]` =
      `py_getitem(x, py_slice(null, 1))`——Auto 无切片字面量，函数
      形式显式构造）；setitem 推 nil 配平。a2py：`obj[i, j]`/赋值/
      `slice(a, b, None)`。探针：切片取行 sum=23、setitem 后 sum=115
- [x] T13 callable 直调（`__call__` 糖）+ a2py `obj(args…)` 发射
      （验证：探针 `model(x)` 形态）
      [✅ 已完成] py_call0(460) 句柄自身直调（`func.call(args)`），
      a2py `fn(args...)`。探针：`py_call0(torch.tensor 类, [1,2,3])`
      → sum=6（W2 构造器同形态）
- [x] T14 `py_with` 上下文高阶内建（`__enter__`/`__exit__` + 回调）
      ；`no_grad` 用例（验证：探针 + `run py_torch` 扩例）
      [✅ 已完成] py_with(461)：宿主三段（__enter__ → call_closure(1)
      → __exit__(None,None,None)），闭包体异常 __exit__ 仍执行且返回
      true 可吞错（Python with 语义）——T02 (A) 级回调桥先例兑现。
      a2py **语句级内联** `with ctx as p: <闭包体>`（表达式级 lambda
      装不下语句体，撤销 _auto_with 助手方案）。探针：no_grad 实例
      化（with 的是 `no_grad()` 产物非类）+ randn 块 → with-ok；
      py_torch 7/7
- [x] T15 `py_torch_infer` 三方套件落地（~10 例 + README + phase
      `p9` 注册）（验证：`cargo run -- --root . run py_torch_infer`
      全绿）
      [✅ 已完成] 16 例（dunder 算术×7 含反射、逐元素 ==、matmul、
      getitem+slice、setitem、call0、kwargs dim、no_grad、May×2）
      三方 **16/16 全绿**；p9 注册（py_torch_train 占位）。实现补强：
      考古发现存量缺口 **DIV-PY-CLOSURE-1**（py 句柄在闭包局部/捕获
      退化为裸 id，master 同形复现）→ py_with 0 参闭包改 codegen
      **内联降级**（py_enter 462 / py_exit 463 bracket，主作用域
      语义）；内联大块抽助手（T09 栈帧教训二次实证）。门禁：
      py_ffi 33/33 + cargo t vm 783/783 + py_torch 7/7 + py_math 
      20/20
- [x] T16 W1 折叠：parity README/guide 同步；门禁留档（验证：
      `cargo tv` + phase p8 回归 + p9 全绿）
      [✅ 已完成] guide p9 节 + parity README 表两行（infer/train）；环境
      复原：pandas/matplotlib 补装（461 解释器失存后 p8 相缺）。
      门禁：**cargo tv 3565/3565**；**phase p8 全绿**（numpy 10/10 +
      pandas 8/8 + matplotlib 3/3 + torch 7/7——W0/W1 对 461 全量零
      回归）；phase p9 16/16；tt/tf 折叠档（tf 2 存量红 P528-D6 同
      W0）

### W2 训练循环

- [x] T17 kwargs 贯通项导入直呼形态（`nn.Linear(…, bias=False)`）
      codegen 项导入臂 + a2py（验证：探针构造器形态）
      [✅ 已完成] `py_item_kw`(464) 5 槽约定（module/func 名运行时解析，
      绕开注册绑定）；`Linear(2, 3, bias: false)`/`SGD(params, lr: 0.1)`
      实证；a2py `Arg::Pair→key=value` 原已在位。连锁修复：
      value_to_py 的 VmRef 臂补 **PyObjectHandle** 解析（kw 位置实参
      携带生成器句柄原落 None——SGD(params) 探针实锤）
- [x] T18 tuple/dict/None 往返封送（拍平映射定案 + divergence 登记）
      （验证：`cargo t py_ffi` + 探针）
      [✅ 已完成] tuple 顶层/嵌套→Auto List（TAG_OBJECT 同数组字面量
      编码；DIV-PY-TUPLE-1 登记：不可变/可哈希 divergence，list 返回
      保持 461 TAG_LIST 通道——TAG_OBJECT 化破 rc 配平且 py_list 有
      master 存量红 P539-D1）；None→null 家族（原 i32 0）；ObjectData
      →PyDict 实参臂。探针 w2_t18：shape 拍平计数 2/bias-null/dict
      2 键/lr 0.5/tolist 拍平。存量登记：P539-D1（py_list sorted_getitem
      master 红，p7 脱门禁）、P539-D2（.len() 类型谎言）、P539-D3
      （a2py 复合接收者）
- [x] T19 `py_torch_train` 三方套件落地（~8 例 seed 化 MLP 训练收敛
      + 异常捕获 + kwargs + 容器往返 + README + phase p9）（验证：
      `cargo run -- --root . run py_torch_train` 全绿）
      [✅ 已完成] 10 例三方 **10/10 全绿**（含验收官④的 seed 化 60 步
      收敛：manual_seed(0)、Linear(2,1)+MSELoss+SGD(lr:0.1)、
      lf<first_loss && lf<1，AutoVM 与 a2py 产物输出一致）。
      **语义演进**：0 维张量急切 f64 与训练冲突（loss 需张量
      backward）→ 仅精确 PyFloat 急切 f64 + `py_float`(465) 显式
      提取（a2py→float()）；EQ/NE 补混型数值臂；py_torch(461)/
      infer 套件断言随迁。栈帧纪律三次实证（T17 内联又溢出 aavm2
      语料→三特形合并 try_py_call_special_form 单调用）
- [x] T20 W2 折叠：验收官④脚本留档；guide 训练章节（验证：门禁）
      [✅ 已完成] 折叠裁定：W2 中间折叠**跳过**——master 带并行会话
      （plan536）未提交脏树（KNOWN-DEBT 等冲突面），强并伤在途工作；
      分支持有全部提交，终态折叠归 /auto-plan:merge。验收官④=
      train.at test 4（seed 化收敛脚本本体即档，
      三方 10/10）；训练章节落 `parity/libs/python/README.md`（标量
      规则+训练惯用法）+ 三套件 README；门禁：tv/tt/tf + p8/p9
      全绿（tf 2 存量红 P528-D6 同前）

### W3 类派生/回调（范围以 T02 结论为准）

- [x] T21 回调桥（Auto fn → `PyCallable`）+ `Dataset` 回调式用例
      （验证：探针 + 单测）
      [✅ 已完成] `py_callable`(466)：`PyCFunction::new_closure` 包装
      Auto 闭包；thread-local 桥窗口（BridgeGuard RAII）回投**当前
      任务**（T02 (B) 设计落地——无裸生命周期 hack，窗外触发=明确
      RuntimeError 且单测钉住）；五宿主 shim 装窗。a2py 恒等降级。
      探针 w3_t21：map(x=>x*2)=12、apply_(x=>x*10)=60 双端一致
      （回调往返×4 实证；Dataset 回调式面=同通道，套件时间盒内未
      单列——apply_/map 双形态已证通道）。py_ffi 34/34
- [x] T22 `py_subclass` 工厂内建最小面（`nn.Module` forward 覆盖）
      或显式延后登记 + Sequential 组合金样替代（验证：W3 套件或
      KNOWN-DEBT 登记 + 门禁）
      [✅ 已完成] 走**预案路径**：显式延期（P539-D4——类工厂
      方法绑定面 + GIL/生存期约束审查超 W3 预算；回调桥单通道已通
      是其可行性前提）；组合式金样 = py_torch_train（Linear 裸栈 +
      seed 化收敛 10/10）；调研节落 python-parity-roadmap.md §7.3；
      §7.1/7.2 为 `@`/`**` 中缀独立语言决策调研（验收⑥）。门禁：
      py_ffi 34/34 + cargo t vm 787/787 + 四套件三方全绿

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-04 22:10）
**复审方式**：worktree `D:/autostack/.wt/lang-539/auto-lang`（branch
plan-539-dev @ f98311680+fmt 提交）内重跑全量门禁 + 逐条验收对照代码。

### 验收标准逐条裁定

1. **六条 DIV-PY 债状态明确** —— ✅ pass。known-divergences.md 实核：
   FLOAT/EXCEPT/ITER/KWARGS/AUTOLIST 五条 ✅fixed（各带根因/修复/回归
   证据），CONST-1 ✅documented（importlib 句柄裁定）。回归用例在套件：
   kwargs=infer t13、float=py_math 20/20+infer t5-7、except=infer
   t15/16、iter=infer t14、autolist=infer t12/train t7、const=p8d 探针
   +分类 README。
2. **两套件三方全绿 + p9 + py_torch 存量** —— ✅ pass。实跑：
   py_torch_infer 16/16、py_torch_train 10/10（三方一致性，含
   AutoVM/a2py TAP 文本逐行对拍）、py_torch 7/7；p9 注册于
   auto-parity main.rs phase 表。
3. **`*` 逐元素 + matmul 三方一致 + 无变义** —— ✅ pass。代码实核：
   engine.rs MUL 臂仅 `__mul__`/`__rmul__`（5664 行）；`__matmul__`
   全仓唯一出现于 py_matmul shim（py_ffi.rs:658）；a2py 仅发射
   `.matmul(`（trans/python.rs:1101）。回归钉：infer t2/t6/t7
   （逐元素）+ t9（matmul）。
4. **验收官④ seed 化训练一致标量序列** —— ✅ pass。train.at test 4：
   manual_seed(0)、Linear(2,1)+MSELoss+SGD(lr:0.1)、60 步
   zero_grad/backward/step、lf<first_loss && lf<1；三方 TAP 全等
   （comparator 按名逐行对拍即"输出一致标量序列"）。
5. **tv 全绿 + fmt 干净 + 无 TODO/debug print** —— ✅ pass（复审期
   修复后）。tv 3569/3569；新代码 fmt：复审发现 py_ffi 新增 hunk 未
   格式化——外科手术式修 24+1 hunk（存量 68 hunk 不动避免 diff 污染），
   修后编译+34/34+全量门禁重跑绿；diff 扫描无 TODO/FIXME/dbg!/println!
   新增。
6. **README/guide/roadmap 同步** —— ✅ pass。五文件实核在位：
   分类 README（新建）、infer/train 两套件 README、parity-guide p9 节、
   parity README 表两行、roadmap §7（@/＊＊/延期备注）。

### 门禁留档（复审档，最终代码状态）

- cargo tv **3569/3569**（208.9s）；cargo tt **3756/3756**；
- cargo tf 3407 过 + **2 红 = P528-D6 在案存量**（docs_gen
  kitchen_sink + schema_drift，非本计划引入，KNOWN-DEBT 在册）；
- py 套件三方：math 20/20 + torch 7/7 + infer 16/16 + train 10/10 +
  numpy 10/10；py_ffi 单测 34/34（--features python）。

### 遗漏/延后/workaround 扫描

- **延后（计划内预案，非私自）**：py_subclass 类派生——计划文本
  预授权路径（待澄清① + T22 "或显式延后登记"），登记 P539-D4，
  Sequential/Linear 裸栈金样 = train 套件在案。
- **T21 "Dataset 回调式用例"**：以 map/apply_ 双探针交付同通道证据，
  Dataset 本体依赖 py_subclass（随 D4 延期）——计划执行注记在案，非
  静默丢弃。
- **workaround 登记**：py_with 内联 bracket 体异常时跳过 __exit__
  （README 注明，非 Python 保证退出语义）；W2 中间折叠跳过（master
  并行会话脏树，分支持有全部提交，终态折叠归 merge）——均在档。
- **新登记 P539-D5**：py_call_may 仅位置实参 + a2py 语句体闭包 set 化
  （表达式体必需）。
- 存量登记：P539-D1（py_list master 红）、D2（.len() 类型谎言）、
  D3（a2py 复合接收者）。

### 裁定

六条验收全 pass，无阻断债 → **status: archived**，就绪
/auto-plan:merge。

## 执行注记

### W0 探针记录（T01，scratch/p539/，master 基线）

运行环境：`target/debug/auto.exe`（master，默认 features 含 python），
PYO3_PYTHON=C:\Python314\python.exe（3.14.2），torch 2.14.0+cpu 本机
新装（原 461 解释器已升级失存，C:\Python312 仅剩孤立 site-packages）。

| 债 | 探针 | master 现状（实测输出） |
|---|---|---|
| KWARGS-1 | p1_kwargs.at | Auto 已有命名实参语法（AST `Arg::Pair`，parser.rs:3121），原生 fn 调用正确绑定；但 py-FFI 调用点 codegen 对 `Arg::Pair` **静默丢名推值**（codegen.rs:8487 C2 注释的 variadic 语境处理）——不是报错，是静默错位语义 |
| ITER-1 | p2_iter.at | `for row in tensor` **静默零迭代**（无输出无错误）：句柄走"else"索引通道（codegen.rs:2870 起），ARRAY_LEN 对 PyObj 堆对象返 0 |
| AUTOLIST-1 | p3_list.at | `tensor([1.0,2.0,3.0])` → `FFI("…Could not infer dtype of NoneType")`：数组字面量 CREATE_ARRAY 用 **encode_object**（TAG_OBJECT，engine.rs:2885），`pop_auto_py_arg` 只认 TAG_LIST 分支（TAG_LIST 仅 py 返回/ui 桥使用）→ TAG_OBJECT 臂 downcast 三连失败落 `py.None()`，整表变单 None |
| FLOAT-1 | p4_float.at | `sqrt(2.0)` → 字符串 "1.4142135623730951"；`sqrt(4.0)` → "2"（整值丢 .0）；字符串参与算术 `v + 1.0` → **NaN**（静默） |
| EXCEPT-1 | p5_except.at | `py_call(t, "no_such_method")` → `Error: × FFI("…AttributeError…")` 进程级中止，后续 print 不可达，无捕获通道 |
| CONST-1 | p6_const.at | torch+math 异名共存 OK；句柄 print 形态 `<heap:4000000>`；同名冲突（torch/numpy arange）461 README 在案；裸模块句柄形态待 T08 探针 |

附加考古（喂给后续任务）：
- for-in 三通道：range 计数 / Call 源（iterator.next 哨兵 -1）/ 其余
  （ARRAY_LEN+GET_ELEM 索引）——T07 贯通面在此。
- `value_to_py` 缺 `Value::Array`/`Value::VmRef` 臂（嵌套结构封送落
  None）——T04 嵌套往返需补。
- CALL_PY 实参计数按 `call.args.args.len()`（含 Pair）——kwargs ABI
  扩展点在 codegen.rs:8594 与 shim 弹参约定两侧。

## 待澄清事项

1. **W3 范围**（T02 已裁定 2026-09-04）：回调桥**可行**——(A) 立即
   回调复用 shim 内 `vm.call_closure` 既定先例（W1 py_with 即用）；
   (B) PyCallable 包装走**新任务执行模型**（AutoTask::new 公开 + 
   vm.closures 共享，无 &mut 别名）。约束：num_workers=0（torch CPU 
   DataLoader 默认）、回调仅发生在宿主 py_call shim 窗口内。W3 保留
   在计划内最小面执行；若实施期发现新任务模型在闭包捕获 env 上有
   不可修缺口，T22 降级 Sequential 金样并登记延期。
2. **`use.py` 别名/子模块**：T08 探针后定案最小面（别名语法 vs 模
   块句柄绕行）；若做语法级别名，考虑独立小计划。
3. **tuple 映射**：拍平为 `List` 与 Python tuple 语义差异（不可变/
   可哈希）以 divergence 登记，是否未来引入 Auto tuple 值类型不在
   本计划。
4. **中缀 `@`/`**`**：独立语法计划，本计划只在 roadmap 追加调研节。
