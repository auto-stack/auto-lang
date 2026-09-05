# 脚本模式互操作设计（Script-Mode Interop）

> **Status**: Draft（裁决已闭环，待立项实施）
> 路径：本文件 | 上游：[python-parity-roadmap](python-parity-roadmap.md) §7、[auto-as-rust-script-strategy](auto-as-rust-script-strategy.md)
> 起点：Plan 539（PyTorch FFI）沉淀的桥接面 + 设计讨论（2026-09-04/05，与用户逐条裁决）

## 0. 目标与非目标

**目标**：让 Auto 脚本模式（AutoScript，`.as`）达到"作为 Python/JS/GDScript/ArkTS
替代"的表面体验——直接写 `t.sum(dim: 0)`、`model(x)`、`with torch.no_grad() {}`，
编译器自动 lowering 到 Plan 539 建成的 py_xxx 桥接函数族。

**非目标**：不改变桥接层语义（py_xxx 是语义本体，糖只是改写）；不在本期适配
JS/GDScript/ArkTS（只预留 ForeignRef 协议位）；不提速（易用性/可读性 >> 效率）。

## 1. 核心架构：糖 → 桥 lowering

```
.as 源码（糖）                    正常模式 Auto（桥）           VM
─────────────────────           ─────────────────────       ────────
t.sum(dim: 0)        ──改写──▶   py_call_kw(t, "sum", …)  ──▶  452
model(x)             ──改写──▶   py_call0(model, x)       ──▶  460
with ng { … }        ──改写──▶   py_enter(ng); try{…}finally{py_exit(ng)}
x.foo / x[i] / x[a..b] ─改写──▶  py_getattr / py_getitem / py_slice
```

- **单一真相源**：语义全在桥上（450-466 + 待新增 setattr/contains 等）。
  糖有 bug 看改写产物即知；py 行为分歧修在桥上，糖自动继承。
- **改写器做成独立 source-to-source 工具先行**（Auto 糖 → 正常模式 .at），
  像 Babel 独立演进、产出直接跑现有门禁；稳定后折进编译器模式管线
  （multi_mode.rs），**不**塞进 codegen 热臂（Plan 539 三次栈溢出教训）。
- **分发组合子**：`send/get/set/call/len/iter` 等约十个正常模式函数组成
  通用对象协议（内部查运行期 tag：ForeignRef → py 桥，Auto 值 → 原生方法表）。
  静态已知 py 的调用点（use.py 导入、py 调用返回值流型）窥孔直达 py_xxx，
  零分发开销。这组组合子即未来 JS/ArkTS 适配的承接点（换 ForeignObject 实现）。
- **语句级糖**：with/for-in 工作在语句层（表达式级改写覆盖不了控制流）。
- **可调试**：`--dump-lowered` 展示改写产物（AI 生成-验证循环的审查面）。

## 2. 模式体系（裁决：`.as` 扩展名）

| 信号 | 效果 |
|---|---|
| `.at` 文件 | 正常模式（存量语料默认）：静态类型、null 禁令、显式 `.?()` |
| `.as` 文件 | AutoScript：动态分派糖、隐式 Err 传播、null 是值 |
| `#[script]` / `#[rust]` pragma | 显式覆盖（罕见场景） |
| `.at` 含 `use.py`/null 字面量 | **诊断错误**（"疑似脚本内容，改名 .as 或加 #[script]"）——推断只做提示不做语义 |

- 助记：`.as` = AutoScript。与 ActionScript 撞扩展名（已死语言，可接受）。
- 先例：`.ts`/`.js` 方言对 + 渐进关系。
- 跨模式 import 双向允许：`.as`→`.at` 按签名检查；`.at`→`.as` 导出函数
  自动物化为显式 `!T`（隐式可失败跨类型边界变显式）。
- 迁移：含 `use.py` 的 `.at`（py parity 套件等）一次性改名 `.as`，
  parity runner glob 兼容两者。

## 3. 边界目录（lowering 规则表）

### A. 调用
| # | 糖 | lowering | 状态 |
|---|---|---|---|
| A1 | `t.sum(a)` | `py_call` | 539 ✅ |
| A2 | `t.sum(dim: 0)` | `py_call_kw` | 539 ✅ |
| A3 | `model(x)` | `py_call0` | 539 ✅ |
| A4 | `Linear(2,3, bias: false)` | `py_item_kw` | 539 ✅ |
| A5 | 闭包作 py 实参 | 自动包 `py_callable` | 539 手动，糖自动 |

### B. 访问
| # | 糖 | lowering | 状态 |
|---|---|---|---|
| B1 | `w.shape` | `py_getattr` | 539 ✅ |
| B2 | `m.weight = w` | 新增 `py_setattr` | 待建 |
| B3/B4 | `x[i]` / `x[i,j] = v` | `py_getitem`/`py_setitem` | 539 ✅ |
| B5 | `x[a..b]` / `x[..b]` / `x[a..]` / `x[a..=b]` / `x[a..b..c]` | `py_slice`（Nil 端↔None；`..=`→b+1）| 词法已在（含步长注释位），实现步长 |
| B6 | `len(x)` | GIL `len()`（动态分派） | 随组合子 |
| B7 | `k in d` | 新增 `py_contains`（`__contains__`） | 待建 |
| B8 | `var torch = use.py torch` | 裸模块绑句柄（重做 Plan 300 腐烂面） | 待建 |
| B9 | `torch.nn.Linear` | B1 递归 | 依赖 B8 |

### C. 运算符
| # | 糖 | lowering | 状态 |
|---|---|---|---|
| C1/C2 | `+ - * / %` 一元 `-` 比较 | dunder 臂（反射协议） | 539 ✅ 逐元素语义钉死 |
| C3/C4 | `if x:` / `and or not` | GIL `bool(x)`（多元素张量按 Python 抛） | 待建（JMP 臂） |
| C5 | `a @ b` | `a.matmul(b)` → `__matmul__` | 词法位置可分（`@T` 引用类型不动） |
| C6 | `a ** b` | `Math.pow` / 句柄 `__pow__`（dunder 表加 POW 臂） | 定案 |
| C7 | `a is b` | GIL `is` | 待建 |
| — | `^` | **保留给未来 XOR**（与 Python 分工一致） | 定案 |

### D. 值/封送
| # | 规则 | 状态 |
|---|---|---|
| D1-D3 | 标量/list/对象 → PyObject | 539 ✅ |
| D4 | **注解预言机驱动标量强制**：`use.py` 导入时读返回类型注解（`-> float` 等），有承诺的自动 `py_float` 强制；无注解走保守规则（比较/print 强制，赋值/传参不强制保 backward） | 定案（`typing.get_type_hints` 运行期可读；注解撒谎最坏退回 Python 行为，不做正确性地基） |
| D5 | list→TAG_LIST（py_call 面）/tuple→List(TAG_OBJECT)：维持双通道，统一需先清 P539-D1 | 在案 |
| D6 | `Py_None ↔ null`（T18） | 539 ✅ |
| D7 | 句柄 print/f-string → GIL `str()` | 待建 |
| D8 | `type_name(x)` 动态分派 | 待建 |

### E. 控制流
| # | 规则 | 状态 |
|---|---|---|
| E1 | **错误模型**（见 §4） | 定案 |
| E2 | `with expr { }` / `with expr as x { }` → `py_enter + try-finally + py_exit`；**异常抑制默认不做**（py_exit 固定传 None×3；需要时显式 try-catch） | 定案；`with` 关键字零冲突（仅 `#[with(...)]` 注解参数位，方括号封闭） |
| E3 | for-in 双通道：有 `__getitem__`+len 走索引（快），否则 `__iter__/__next__`；`py_iter()` 强制迭代器语义 | 定案 |

### F. 生命周期
F1 导入（B8 补裸模块）；F2 句柄 rc（债务区非语法面）；F3 闭包捕获句柄退化
（DIV-PY-CLOSURE-1 在案）；F4 回调窗口约束文档化（num_workers=0）。

## 4. 错误模型（定案）

**隐式 `!T` + Err 单通道 + 边界 catch：**

1. 脚本模式函数全部默认可失败（隐式 `!T`，不写签名）；调用点隐式短路传播
   （编译器自动补 ERROR_PROPAGATE——机器已在）；
2. **None/null 是值，永不传播**（Python 模型：`d.get(k)`→None 是值，
   `d[k]`→KeyError 是 Err；`None + 1`→TypeError 归 Err）；
3. **catch 拦值传播**（帧边界拦 Err，`catch e` 绑定结构化载荷；
   py 异常即 `PyException TypeError: ...`）——try-catch 已存在（Plan 010/012），
   值通道拦截是唯一新机制；
4. 隐式传播作用域：**调用 + 索引 + 属性 + 含 null 操作数的运算**；
5. main 边界：未捕获传播 → 脚本带错退出（= Python traceback）；
6. a2py 映射：隐式传播零代码（Python 原生）、`.?(d)`→`x if x is not None else d`、
   catch→except。

## 5. Null 体系（定案）

**运行期一个家族，模式两套纪律：**

- TAG_NULL 家族 = `?T` 的 None 的运行期表示（niche 优化视角）；
- **正字法 `null`**（JSON/JS/GDScript/ArkTS 3/4 对齐 + JSON 字面量零转换）；
  `nil` 退役（parser 别名 + 警告）；`None` 保留为 Option 构造器（Some/Ok/Err
  家族，`?T` 体系表面）；运行期三拼写 null-family 判等（P-053-2 现状）；
- **正常模式三层禁令**：①生产者门控（null/nil 字面量解析错误、`use.py`
  解析错误、未初始化 `var` 禁止）②类型纪律（裸 T 不可空、`?T` 未解包流入
  T 位 = 类型错误——强制度需盘存加固；泛型擦除处按非空不透明处理）
  ③运行期金丝雀（null 误用 → TypeError → 类型良好代码永不触发，触发即
  编译器漏检报警）；
- **None↔null 零转换** + 两条类型规则：null 可流入 `?T` 槽位（即 None）；
  `Some(null)` 非法（Some 载荷非空）；
- `*T` 裸指针：可空性显式（deref 前 `.ok()` 判定或 unsafe 块），与 `?T`
  None 不隐式互通（词法半存在，规则先行）。

## 6. W0 审计清单（null 家族正确性与术语统一）

| 项 | 现状 | 目标 |
|---|---|---|
| `null + 1` 等算术族 | 静默垃圾（实测 `-2147483646`） | TypeError（Python 格式消息，可 catch） |
| `"a" + null` | 垃圾数字入串（实测） | 同上 |
| `null[0]` / `null(x)` / `for x in null` | 未验证/疑垃圾 | TypeError（not subscriptable/callable/iterable） |
| `null.to(int)` | T05 加的 null→-1 臂（**翻案**：int(None) 应 TypeError） | 改错误；查 `?.` 内部依赖 |
| 数组越界 | 静默返回 0 哨兵 | Err（对标 IndexError） |
| `print(null)` | 待定 | `"None"`（a2py parity） |
| `nil` 关键字 | 与 null 并存 | 别名 + deprecated 警告 |
| 未初始化 `var x` | 需探针 | 正常模式禁止 |

验收用例 = 上述探针翻转（`null+1` 从 `-2147483646` 变可捕获 TypeError）。

## 7. 注解预言机（Python 元数据 = 免费的 d.ts）

`use.py` 导入时经 `inspect`/`typing.get_type_hints` 读参数/返回注解
（Plan 300 的 param_count 内省同钩子扩展）：

- `-> float/int` 承诺 → D4 自动标量强制获静态授权；
- `-> T | None` → nullability 知识（lint/微优化）；
- kwargs/参数类型知识同理；
- **定位是优化与糖的授权，不是正确性地基**（注解可撒谎，撒谎最坏退回
  Python 行为；覆盖率 torch 约半数，typeshed 补 stdlib）；
- 对标：TS 的 `.d.ts` / C# 程序集元数据——跨语言无缝感的来源是元数据。

## 8. 跨语言矩阵（预留）

| 维度 | Python（首） | JS | GDScript | ArkTS |
|---|---|---|---|---|
| 宿主 | PyO3 嵌入 ✅ | QuickJS/deno_core | 引擎绑定难 | ets 运行时 |
| 空值 | None（↔null） | null+undefined 双空 | null | undefined |
| 运算符 | dunder 全家 | 无 `@`/`**` 中缀 | 无重载 | 无 |
| 异步 | 同步为主 | Promise→future | 信号/await | Promise |

可移植层 = ForeignRef + send/get/set/call 组合子；差异落各自 ForeignObject
实现与强制规则表，不回渗语法。**/`**` 走函数/方法形式为跨语言默认。**

## 9. 实施波次（立项骨架）

1. **W0 null 家族审计与术语统一**（§6 全表 + 三层禁令的生产者门控；
   可独立小计划先行，收益立竿见影）；
2. **W1 动态分派地基**（ForeignRef 协议 + 分发组合子 + `.as` 模式管线
   + s2s 改写器骨架 + --dump-lowered）；
3. **W2 语法糖批**（A1-A5/B1-B9/C3-C7/D7/E2-E3 lowering 全表 + 迁移
   py 套件改名 .as）;
4. **W3 注解预言机**（D4 授权强制 + nullability lint）。

每波独立折叠（539 先例）；改写器每条规则配 source-to-source 单测，
产出跑现有三方门禁。

## 10. 裁决存档

| 项 | 终裁 |
|---|---|
| 切片 | `x[a..b]` 家族已在，纯 lowering；步长 `a..b..c` 实现 |
| `with` | 新关键字；lower 成 enter+try-finally+exit；异常抑制默认不做 |
| `@` | 脚本模式糖 → matmul；`@T` 引用类型不动 |
| `**` | 脚本模式糖 → `__pow__`/Math.pow；`^` 留 XOR |
| None 传播 | None 是值（无条件）；失败=Err 单通道；运行期类型错误归 Err |
| catch | 拦值传播，绑定结构化 Err；与隐式传播并存（Python 模型） |
| 传播作用域 | 调用+索引+属性+null 运算 |
| D4 | 注解预言机驱动 + 保守回退 |
| E3 | 索引快路径/迭代器双通道 |
| 模式边界 | `.as`/`.at` 扩展名为主 + pragma 覆盖 + 推断降级诊断 |
| null 正字 | `null`；`nil` 退役；`None`=Option 构造器；print "None" |
| 正常模式 | 三层 null 禁令（生产者/类型/金丝雀） |
| 越界 | 0 哨兵 → Err |
| 未初始化 var | 正常模式禁止 |
| 架构 | 糖→桥 lowering；组合子分派；s2s 工具先行；不进 codegen 热臂 |
