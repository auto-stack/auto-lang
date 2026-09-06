---
plan_id: PLAN-560
status: archived            # drafting → executing → execution_done → reviewed → archived
feature_name: script-mode-w2-sugar-batch
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/auto-lang/trans/overview.md: 修改 —— 现状节补 Plan 560：AST 帧形+链式规则（emit.rs 发射器/py_known 静态分析/s2s_rules A-C 族规则批）+ Str 再转义"
  - "specs/auto-lang/frontend/overview.md: 修改 —— 现状节补 Plan 560：with 关键字/Power(**) token/is 中缀糖/@ 中缀（解析层直产桥调用）+ #[with] 注解名撞位修复"
  - "specs/auto-lang/vm/overview.md: 修改 —— 现状节补 Plan 560：py 桥 470-475 六件（contains/module/str/pow/truthy/is）+ CALL_PY→CALL_NAT_COUNTED 改名 + CALL_PY 错误出口 RuntimeError 统一 + print shim 运行期 GIL str() 臂"
  - "specs/auto-cli/project.md: 修改 —— `--dump-lowered` 翻转真产物（W2 头）"
new_spec_components:
  - "specs/auto-lang/trans/design/s2s-lowering.md: 新增 —— s2s 改写契约：AST 帧形链式规则语义 + py_known 保守分析边界 + A/B/C 族规则表（含 py-known 门控设计修正）+ 步长嵌套 Range 位形 + 双星词法惯例（arm c 未消费）教训"
touched_goals:
  - "GOAL-005: W2 糖批主兑现——19 py 套件 .as 载体三方（17 全绿+2 master 既有红随迁实证）+ §3 边界目录糖全表探针绿"

affects: [auto-lang/vm, auto-lang/frontend, auto-lang/trans, auto-cli]   # 受影响的 specs 路径
current_step: 15
total_steps: 15
---

# [PLAN-560] 脚本模式 W2：语法糖批（lowering 全表 + Err 值通道 + py 套件 .as 迁移）

## 变更摘要

脚本模式实施波次第 3 波（W2；W0=550 null 地基、W1=555 分派地基均已归档）。
把 AutoScript（`.as`）的表面体验做实——设计源
[script-mode-interop.md](../design/strategy/script-mode-interop.md) §3
边界目录的 lowering 规则全表 + §4 错误模型落地 + py 五套件改名 `.as`
迁移。四件事主线：

1. **管线激活**：`.as` 执行从 passthrough 翻转为 **lower→compile**
   （s2s 改写器接入运行管线，`--dump-lowered` 展示真产物）——
   P555-D2 裁定随之落地（AST 发射器 + 链式规则语义）；
2. **糖批全表**：A 族调用（含 A5 闭包自动包 `py_callable`）/ B 族访问
   （含 B5 步长切片、B7 `contains` 桥、B8 裸模块句柄、B9 dotted 递归）/
   C 族运算符（`@` matmul、`**` pow、`is`、句柄真值）/ D7 句柄
   print/f-string / E 族控制流（`with` 新关键字、for-in 双通道）——
   每条规则配 source-to-source 单测（§9 纪律）；
3. **Err 值通道**（§4 定案全六条）：隐式 `!T` 传播（调用+索引+属性+
   null 运算四作用域，ERROR_PROPAGATE 机器已在）+ catch 拦值绑定结构化
   Err（py 异常即 `PyException TypeError: ...`）+ main 带错退出；
4. **迁移收口**：py 五套件改名 `.as` + parity runner glob 扩展 + a2py
   接受 `.as`；550 生产者门控从警告级**硬化为诊断错误**（迁移完成后，
   `#[script]`/`.as` 豁免）——P550-D4 的 CALL null 端到端探针面随之
   激活；窥孔直达优化 + P555-D5 CALL_PY 改名清理。

## 目标

1. `auto x.as` 直跑经 lowering 管线：`t.sum(a)`/`t.sum(dim: 0)`/`model(x)`/
   `Linear(2,3, bias: false)`/`w.shape`/`x[i]`/`x[a..b]`/`len(x)`/`with
   torch.no_grad() {}` 直接可写，lowering 到 539/555 桥族——探针逐条
   可跑（糖有 bug 看改写产物即知，`--dump-lowered` 审查面）；
2. C 族运算符糖：`a @ b` → matmul、`a ** b` → `__pow__`/Math.pow、
   `a is b` → GIL is、句柄在 `if`/`and or not` 的 GIL 真值（多元素
   张量按 Python 抛）；
3. Err 值通道按 §4 六条落地；`catch e` 绑定结构化载荷；
4. py 五套件三方门禁在改名 `.as` 后原样全绿（迁移零行为漂移）；
   550 门控硬化后 `.at` 含 use.py/null 为诊断错误、`.as`/`#[script]`
   豁免；
5. 窥孔直达：`use.py` 静态已知调用点绕组合子 tag 分派直达 `py_xxx`
   （零分发开销）+ CALL_PY → CALL_NAT_COUNTED 改名（P555-D5）。

### 非目标（Out of Scope）

- **D4 注解预言机**（`typing.get_type_hints` 授权强制 + nullability
  lint）——W3 专属，不抢入；
- **跨模式 import 物化**（`.at`→`.as` 导出自动 `!T`、`.as`→`.at`
  签名检查）——裁出本波（待澄清#1，建议迁移后按真实需求独立立项）；
- JS/GDScript/ArkTS 宿主（ForeignObject 协议位已有，仅不实现）；
- F2 句柄 rc 债务区、F3 闭包捕获退化（DIV-PY-CLOSURE-1）、F4 回调窗口
  约束——非语法面，维持文档化状态；
- `*T` 裸指针可空性（随指针特性落地，规则已在设计 §5 写死）。

## 架构方案

### 现状基线（W0/W1 沉淀）

- **桥族**：py natives 450-469（539 十七件 + 555 三桥 setattr/len/
  type_name）；**缺席**：py_contains（B7）、裸模块句柄（B8，Plan 300
  腐烂面重做）；
- **分派地基**（555）：组合子六件 1860-1865 双通道 + ForeignObject
  协议 + `ScriptMode` 八格矩阵——`.as` 直跑当前为 **passthrough**
  （信号已立、语义未激活）；s2s 骨架（token 粒度单遍首中 + identity
  发射，规则表空置）——**P555-D2 待裁定**（链式语义/AST 发射器）；
- **dunder 路由**（539 T10）：opcode 臂对 handle 操作数走 Python
  反射协议（`+ - * / %` 一元/比较逐元素语义已钉）——C1/C2 糖在
   `.at` 对 handle 已可用；**缺席**：POW 臂（C6）、`is`（C7）、
   句柄真值 JMP 臂（C3/C4）；
- **迭代**（E3 半成）：ARRAY_LEN 对 PyObjectHandle 走 GIL len（539），
  for-in over tensor 已工作（py_torch_infer 套件实证）；`__iter__` 强制
  迭代器语义的 `py_iter()` 通道在但糖未接；
- **门控**（550）：`.at` 含 use.py/null 警告级（硬化以迁移完成为前提）；
  **CALL null 守卫端到端探针面**（P550-D4）等动态分派语义激活。

### 设计要点

1. **管线激活在先**（批 0）：s2s 升级为 AST 发射器（token 面无法表达
   E2 语句展开与表达式重排——P555-D2 的裁定即此）+ 链式规则（规则表
   有序全过，产物再验证可解析）；`execute_autovm_with_path` 对
   `ScriptMode::Script` 的源走 `lower_source` 产物编译（W1 passthrough
   正式退役）。`--dump-lowered` 从"passthrough+头"翻转为真产物+头。
2. **糖=lowering 规则，语义=桥**（§1 单一真相源）：每条 A/B/C/D/E 规则
   是 AST→AST 改写（发射回正常模式 .at 源），产物跑现有三方门禁——
   py 行为分歧修在桥上，糖自动继承。静态已知 py（use.py 导入/桥返回值
   流型）在 W2 末加窥孔直达。
3. **with 新关键字**（§10 定案）：`with expr { }` / `with expr as x { }`
   → `py_enter(expr)` + try-finally + `py_exit(expr, null, null, null)`
   （异常抑制默认不做）；关键字零冲突（`#[with(...)]` 注解参数位方括号
   封闭）。语句级糖工作在 s2s 语句层。
4. **Err 值通道**（§4 六条按定案落地）：脚本模式函数隐式 `!T`（不写
   签名）；四作用域隐式传播发射 ERROR_PROPAGATE（机器已在，539）；
   catch 拦值=帧边界拦 Err 通道 + `catch e` 绑定结构化载荷
   （`PyException <type>: <msg>` 字符串形态，W2 最小面）；main 未捕获
   → 带错退出。**None/null 永不传播**（550 已把 null 运算翻为
   TypeError→Err，语义地基就位）。
5. **迁移纪律**：py 五套件 `.at`→`.as` 逐套件改名 + runner glob 兼容
   双扩展名 + a2py 面向 `.as` 接受；三方全绿一个套件一个套件过
   （迁移红=lowering 规则 bug，修规则不修套件）。全迁完后 550 门控
   硬化（警告→诊断错误）。
6. **栈帧纪律延续**（539/550/555 教训）：s2s 规则体小、发射器集中；
   新 dunder 臂（POW/is/bool）走 539 py_dunder 同型助手，不内联进
   热递归 match 臂。

## 技术栈

- `crates/auto-lang/src/trans/auto_s2s.rs`（AST 发射器 + 链式规则表 +
  A/B/C/D/E 规则实现）+ `trans/` 新增 emit 模块（AST→Auto 源打印器）；
- `crates/auto-lang/src/lib.rs`（execute_autovm_with_path 脚本模式走
  lowering 产物；dump_lowered 真产物）；
- `crates/auto-lang/src/lexer.rs` + `parser.rs`（`with` 关键字、`@`
  matmul 消歧、`**`/`is` token 面）；
- `crates/auto-lang/src/py_ffi.rs`（py_contains 桥 470 + 裸模块句柄
  B8 + POW/is/bool dunder 臂）；
- `crates/auto-lang/src/vm/engine.rs`（句柄真值 JMP 臂——若经 s2s
  lowering 到显式调用则免 engine 改动，按实现裁）；
- `crates/auto-lang/src/vm/interop.rs`（窥孔直达注册面）+
  `vm/codegen.rs`（CALL_PY→CALL_NAT_COUNTED 改名涉及发射/执行双端）；
- `parity/libs/python/py_*/tests/auto/*.at`→`*.as` 改名 + parity
  runner glob（`crates/auto-parity/src/`）+ a2py `.as` 接受；
- 探针：`scratch/p560/`；回归：`cargo tv` + `cargo tt` + py 五套件三方
  + `cargo t vm`。

## 需求分析与背景调查

（从 spec ledger overview 与设计源取材）

- **设计源**：`script-mode-interop.md` §3 边界目录全表（本计划的工作
  清单——A1-A5/B1-B9/C3-C7/D7/E2-E3 各行状态列即差距分析）、§4 错误
  模型（六条定案）、§9 波次骨架 W2 行、§10 裁决存档（with/@/**
  /None 传播/catch/传播作用域/架构）；
- **goals 关联**：GOAL-005（Python parity——W2 是"作为 Python 替代"
  表面体验的主兑现波；py 五套件迁移后 `.as` 成为 parity 载体）；
- **前波沉淀**：P550-1..6（null 家族守卫 + 门控 lint）、P555-1..6
  （分派地基 + s2s 骨架 + 模式管线）——本计划直接消费其全部通道；
- **债务联动（本波销号/激活）**：P555-D2（s2s 链式/AST 发射器裁定）
  → T02；P555-D5（CALL_PY 改名）→ T13；P550-D4（CALL null 端到端
  探针面）→ T14 动态分派语义激活后补探针；550 门控硬化前提=迁移完成；
- **先行考古**：py natives 450-469 在位（467-469 为 555 新增）；组合子
  1860-1865；dunder 路由 12 opcode（539）；`with` 词法零冲突；
  Plan 300 腐烂面=裸模块绑定的历史实现（B8 重做参照）。

## 详细设计

### lowering 规则表（W2 实现清单＝§3 全表展开）

| 规则 | 糖 | lowering 目标 | 实现位 |
|---|---|---|---|
| A1-A4 | `t.sum(a)` / `t.sum(dim: 0)` / `model(x)` / `Linear(2,3,bias:false)` | `py_call` / `py_call_kw` / `py_call0` / `py_item_kw` | s2s AST 规则（539 桥已在） |
| A5 | 闭包作 py 实参 | 自动包 `py_callable(closure)` | s2s 实参遍历规则 |
| B1/B9 | `w.shape` / `torch.nn.Linear` | `py_getattr`（dotted 递归） | s2s（B9 依赖 B8 模块句柄） |
| B2 | `m.weight = w` | `py_setattr`（467 ✅） | s2s 赋值规则 |
| B3/B4 | `x[i]` / `x[i,j] = v` | `py_getitem` / `py_setitem` | s2s |
| B5 | `x[a..b]`/`x[..b]`/`x[a..]`/`x[a..=b]`/`x[a..b..c]` | `py_slice`（Nil 端↔None；`..=`→b+1；**步长实现**） | s2s + 词法步长位 |
| B6 | `len(x)` | `obj_len` 组合子（1863 ✅） | s2s |
| B7 | `k in d` | 新桥 `py_contains`(470) + s2s | py_ffi + s2s |
| B8 | `var torch = use.py torch` | 裸模块绑句柄（Plan 300 重做） | py_ffi/import 通道 |
| C3/C4 | `if x:` / `and or not` | GIL `bool(x)`（多元素张量按 Python 抛） | s2s（lower 到显式真值调用） |
| C5 | `a @ b` | `py_matmul`（`@T` 引用类型不动——词法消歧） | lexer + s2s |
| C6 | `a ** b` | Math.pow / 句柄 `__pow__`（dunder 表加 POW 臂） | s2s + py_ffi POW 臂 |
| C7 | `a is b` | GIL `is`（新内建 `py_is` 471 或 dunder 臂） | py_ffi + s2s |
| D7 | 句柄 print/f-string | GIL `str()`（`py_str` 472 或复用 type_name 通道） | py_ffi + print shim 分派 |
| E2 | `with expr {}` / `with expr as x {}` | `py_enter` + try-finally + `py_exit(None×3)` | parser 关键字 + s2s 语句规则 |
| E3 | for-in over handle | `py_iter()` 强制迭代器 | s2s（array 通道已工作，双通道按 §E3） |

### Err 值通道（§4 六条 → 实现契约）

1. 脚本模式函数默认可失败（隐式 `!T`，不写签名）——s2s 在 `.as` 函数
   头/调用点统一补 ERROR_PROPAGATE 发射；
2. None/null 是值永不传播（550 守卫已翻 TypeError→Err 通道）；
3. catch 拦值：帧边界拦 Err + `catch e` 绑定 `PyException <type>: <msg>`
   字符串载荷（结构化对象形态 W2 最小面=带类型前缀字符串）；
4. 隐式传播四作用域：调用 + 索引 + 属性 + 含 null 操作数运算；
5. main 边界未捕获 → 脚本带错退出（退出码非 0 + stderr 消息）；
6. a2py 映射：隐式传播零代码（Python 原生）、`.?(d)`→`x if x is not
   None else d`、catch→except（a2py 面）。

### 迁移与硬化（批 4）

- py 五套件 `tests/auto/*.at` → `*.as`（probe 逐条过：AutoVM=lowering
  产物执行、a2py=转译、native Python=oracle——三方对拍载体即 `.as`）；
- parity runner glob 兼容 `.at|.as`；a2py 转译器入口接受 `.as`；
- 550 门控硬化：`.at`（无 pragma）含 use.py/null/nil → 诊断**错误**
  （E0xxx 级，替换现 stderr 警告）；`.as` 与 `#[script]`/`#[rust]`
  豁免矩阵不变；硬化后全仓 `.at` 语料零残留（`cargo tv` 裁决）。

## 测试设计

- **探针先行**（scratch/p560/）：§3 全表逐条糖形态现状（lowering 前
  `.as` 里的编译错/行为）+ Err 通道六条形态——基线入执行注记；
- **s2s 规则单测**：每条规则 source-to-source 断言（糖源→桥源产物
  字面量级比较 + 产物再解析通过）——§9 纪律"改写器每条规则配
  source-to-source 单测"；
- **端到端探针**：torch 句柄全糖矩阵（infer 套件语料子集在 `.as`
  下逐条跑）+ Err 通道 try-catch 拦值探针 + `--dump-lowered` 真产物
  留档；
- **三方门禁**：py 五套件迁移后全绿（每套件迁移即跑）；`cargo tv` +
  `cargo tt`（a2py `.as` 面）+ `cargo t vm`；
- **硬化回归**：`.at` 含 use.py 报错探针 + `.as` 豁免 + `#[script]`
  豁免 + 存量 `.at` 语料零误伤（tv 裁决）。

## 验收标准

1. `.as` 经 lowering 管线执行：§3 规则表全列糖形态探针绿（A1-A5/
   B1-B9/C3-C7/D7/E2-E3）；`--dump-lowered` 输出真桥源（抽查留档）；
   s2s 每规则 source-to-source 单测在案；
2. Err 通道六条探针绿：隐式传播四作用域短路、`catch e` 绑定
   `PyException TypeError: ...`、main 带错退出非 0、None 永不传播；
3. py 五套件三方全绿（`.as` 载体）；`cargo tv` + `cargo tt` +
   `cargo t vm` 全绿；
4. 550 门控硬化生效：`.at` 含 use.py/null → 诊断错误（编译期拒绝），
   `.as`/`#[script]`/`#[rust]` 豁免；存量 `.at` 语料零残留红；
   P550-D4 端到端探针面激活并补案；
5. 窥孔直达在案（静态已知 py 调用点产物直呼 py_xxx，`--dump-lowered`
   可见）+ CALL_NAT_COUNTED 改名全链一致；
6. P555-D2/D5 销号注记 + 本波债务登记（KNOWN-DEBT）。

## 执行步骤

（批 0 管线激活 → 批 1 B 族 → 批 2 C 族 → 批 3 E 族+Err → 批 4 迁移硬化）

- [x] T01 探针基线：`scratch/p560/` §3 全表糖形态 + Err 六条形态在
      现状 `.as`（passthrough）下的行为逐条记录（验证：探针可复跑，
      基线入执行注记）
      [✅ 已完成] 2026-09-05 六探针矩阵在案（p01-p06）：A 族方法糖=运行期 CALL_SPEC 错（PyObj 无 sum）；B1 属性=GET_FIELD 垃圾 `<obj:...>`；B3 handle 索引疑似已通（zeros(3)[0]→0）；B6 len=链接错 E0401；B5 切片可解析（len 先拦）；E3 for-in over tensor **今日已工作**（539 array 通道 GIL len+getitem，0/1/2）；Err：py IndexError 可 try-catch 但 e 绑定裸 FFI 串（非 PyException 载荷）、无隐式传播。详见执行注记 T01
- [x] T02 s2s 升级：AST 发射器（`trans/emit.rs` 新模块，AST→Auto 源
      打印）+ 链式规则语义（规则表有序全过+产物再解析）——P555-D2
      裁定落地（验证：identity round-trip 稳定单测 + 链式注入单测绿）
      [✅ 已完成] emit.rs（Op/Type 符号表直渲——两者 Display 均为 S-expr 调试形态，语料靶场实证；脚本子集+未覆盖响亮报错；Store 类型标注省略=推断回填；while=Iter::Cond/Destructured/Indexed 臂）；链式规则 LoweringRule{id,transform} 有序单遍（空表=规范化 identity）；四单测绿——含**五套件语料逐文件 parse→emit→re-parse→emit 幂等**（T12 迁移面实弹靶场：math/infer/train/numpy/torch 全过）
- [x] T03 管线激活：`lib.rs` execute_autovm_with_path 对
      `ScriptMode::Script` 源走 `lower_source` 产物编译；
      `--dump-lowered` 翻转真产物（验证：`.as` 空规则=行为与 W1 逐
      字节同回归探针 + dump 留档）
      [✅ 已完成] .as 扩展名预检 + #[rust] 行首压回（启发式预检，完整判定在 session 解析段——罕见面 P560 债）；p05 行为等价探针（0 1 2 同 W1）；dump 真产物（规范化源+W2 头）留档。**分歧注记**：原验证措词"逐字节同"系 W1 token 帧假设——AST 发射器规范化布局，等价判据修正为**行为等价**（输出逐字节同），源级为规范化重排（语料 round-trip 幂等已钉）
- [x] T04 B7/B8 桥补：`py_ffi.rs` py_contains(470) + 裸模块句柄
      （B8，use.py torch 绑模块对象）+ B9 dotted 递归（验证：桥单测
      + `use.py torch` 后 `torch.nn.Linear` 可达探针）
      [✅ 已完成] py_contains 470 + py_module 471（py.import 模块缓存，PyObjectHandle("module")）；注册单测 + p07 探针：py_module("torch")→"module"、getattr 链 nn/device、contains hit/miss 双态全绿。注：`var torch = use.py torch` 糖形态的 **s2s 规则**归 T05（本任务落桥半——计划文"绑模块对象"即 py_module 通道）；B9 递归经 py_getattr 链实证
- [x] T05 A/B 族规则（一）：A1-A4 调用 + B1/B9 属性 + B2/B3/B4 赋值
      索引 lowering 规则 + 单测（验证：source-to-source 单测 + torch
      探针 `t.sum(dim: 0)`/`w.shape`/`x[i]`）
      [✅ 已完成] py_known.rs 静态分析（items/裸模块/桥与组合子调用/导入项调用流+Ident 直传，分支不敏感保守单遍）+ rule_ab_family；**设计修正**：A1/A2 只对 py-known 接收者改写→py_call（盲改会让 obj_call Auto 臂拒绝 `s.len()`——Auto 方法分派保持糖态，实证钉）；B 族同门。考古减负：A4 item-kwargs 已由 codegen（539 T17）覆盖无需规则；A3 裸名调用流型归 T13。探针：p01 复跑翻绿（sum=15/kw=15——T01 基线病灶根除）+p08（shape/索引）；语料 round-trip 幂等保持（规则激活态）
- [x] T06 A/B 族规则（二）：B5 切片族（含步长 `a..b..c` 实现——词法
      步长位落地）+ B6 len + A5 闭包自动 py_callable + D7 句柄
      print/f-string GIL str()（验证：单测 + `x[1..3]`/`len(x)`/
      print(tensor) 探针）
      [✅ 已完成] 步长无需词法工作——`a..b..c` 已解析为嵌套 Range(a, Range(b,c))（实证），规则拆解三参 py_slice；B3 让位 B5（切片位形跳过）；D7 双通道=py_str 472 桥+print shim 运行期 handle 臂（嵌套形态兜底）+s2s 裸名包裹（防 obj_len/py_str 二遍重入收敛钉）；A5 防 py_callable 重入。六单测+端到端（10/3/4/tensor 全文）绿。f-string 面=print shim 已覆盖打印路径，f-string 插值语法本身语言无此形态（`"a" + x` 走 ADD dunder）——D7 以 print/拼接通道收口
- [x] T07 C 族（一）：`@` 词法消歧（`@T` 引用不动）+ `a @ b`→
      py_matmul lowering；`a ** b`→Math.pow/`__pow__`（py_ffi dunder
      表 POW 臂）（验证：单测 + `a @ b`/`a ** 2` 探针）
      [✅ 已完成] 实现位前移：lexer Power(**) token + parser infix 臂**直产桥调用**（a@b→py_matmul 456、a**b→py_pow 473 新桥=GIL operator.pow，数字/__pow__ 通用即"POW 臂"语义）——免新增 auto_val::Op 变量的全仓 exhaustive-match 波及（设计分歧注记：语义等价，发射形态从 s2s 规则前移到解析层，@T 类型位零影响实证）。探针 1024/134/8 绿；语料 round-trip 保持
- [x] T08 C 族（二）：`a is b`→GIL is（py_is 471）+ C3/C4 句柄真值
      （s2s lower 到显式 GIL bool 调用，多元素张量按 Python 抛）
      （验证：单测 + `if t:`/`a is None` 探针）
      [✅ 已完成] py_truthy 474 + py_is 475（占号顺移：py_str/py_pow 先占 472/473）；C3/C4=待澄清#4 裁定形态（s2s 显式调用零热臂）；parser Is 中缀臂（is 系既有保留 token——词位判别无需 Ident hack）；探针 falsy-0/truthy-2/is 同柄 true/异柄 false；语料 round-trip 保持
- [x] T09 E2 with：parser `with` 关键字（`with expr {}`/`with expr
      as x {}`）+ s2s 语句规则 → py_enter + try-finally + py_exit
      （异常抑制不做）（验证：单测 + no_grad 探针——infer 套件 14 例
      形态）
      [✅ 已完成] 无 as 形态直产 `py_with(ctx, () => { body })` 调用（全复用 539 内联通道；Stmt::Expr 形态避开 convert_last_block 尾块转对象坑）；**with-as 债案（P560-D2）**：`as` 系既有 Cast 中缀，parse_expr 整吞 `expr as name` 为 Cast、块体又被单元构造语法吞——绑定语法歧义响亮拒绝待后续裁定（py_enter/py_exit 显式形态可用）；顺修两枚执行期bug：lexer 双星探测（arm c 未消费惯例·peek 恒见 c·单星全误翻 Power——单测钉）+A5 豁免 py_with（内联要裸闭包）；no_grad 探针 gflag=0（= infer 套件 test_with_no_grad 同语义）
- [x] T10 E3 for-in 双通道：handle 源 `py_iter()` 强制迭代器 + Auto
      源 array 通道（现状）分派（验证：tensor for-in 探针 + dict
      items 迭代探针）
      [✅ 已完成] 双通道实证：sized handle（tensor）for-in 走 array 通道（GIL len+getitem，539 面——0 1 2/求和 3）零新码；unsized 语义走显式 py_iter 物化+py_next 拉取（耗尽→null 探针 true）。**裁定注记**：unsized 源的"自动"检测静态不可判（GIL len 运行期才知），双通道=编译期 sized 假设（array 通道）+ 显式 py_iter 覆盖 unsized——与 §E3 定案"双通道"的运行期形态一致；s2s 强制改写（for-in over py-known → py_iter 物化）会损失 sized 快路径，不取
- [x] T11 Err 值通道：隐式 `!T` 传播四作用域（ERROR_PROPAGATE 发射）+
      catch 拦值绑定 `PyException <type>: <msg>` + main 带错退出 +
      a2py 映射（验证：六条探针 + py 套件 15/16 例 May 通道回归）
      [✅ 已完成（最小面+双债案）】可观测契约全落地：CALL_PY 错误出口 FFI→RuntimeError 统一（catch e 绑定含 Python 类型字样载荷——p14 探针 caught+IndexError 全文）；None 永不传播（550 面观测 true）；main 未捕获 exit=1（双探针实测）；May 显式传播通道（py_call_may+.? = 四作用域语义的可表达形态，fallback 探针）。**P560-D3 债**：隐式传播自动化（.as 函数内 py 位点自动补 ERROR_PROPAGATE）——ERROR_PROPAGATE 是 May 值通道而 py 错误走异常通道，两通道汇合需桥出口产 Err 值（473+ shim 面的深集成），归 W3 前裁定；**P560-D4 债**：载荷严格 "PyException <Type>:" 前缀（现 RuntimeError 原文含类型字样，前缀精化随 D3 通道汇合同批）
- [x] T12 迁移：py 五套件 `tests/auto/*.at`→`*.as` 逐套件改名 +
      parity runner glob（`.at|.as`）+ a2py 接受 `.as`（验证：五套件
      三方逐套件全绿）
      [✅ 已完成] 五套件 git mv + runner 四发现门双扩展名；**顺修 trans_python pyname 原地覆写 bug**（path.replace(".at",".py") 对 .as 不命中→产物写回源文件，py_math 被覆写实证后从暂存区恢复+扩展名感知修正）；五套件三方 64/64 全绿（math 20+torch 7+infer 17+train 10+numpy 10）
- [x] T13 窥孔+改名：use.py 静态已知调用点 s2s 产物直呼 py_xxx +
      CALL_PY→CALL_NAT_COUNTED 全链改名（发射/执行/既有 450-469 面）
      （验证：`--dump-lowered` 直达产物抽查 + `cargo t vm` 全绿）
      [✅ 已完成] 窥孔由 A1 直呼形态满足（py-known 方法糖直接落 py_call——设计默认组合子+窥孔，实现取直呼优先，p09 dump 直证）；CALL_PY→CALL_NAT_COUNTED 31 处 8 文件（P555-D5 销号）；cargo t vm 801/801 + infer 17/17 复验
- [x] T14 硬化：550 门控 `.at` 含 use.py/null/nil 警告→诊断错误
      （`.as`/`#[script]`/`#[rust]` 豁免不变）+ P550-D4 CALL null
      端到端探针补案（验证：硬化探针矩阵 + `cargo tv` 存量零残留红）
      [✅ 已完成（2026-09-05 用户裁定后补完）] 硬化落地：警告→**诊断错误 auto_gate_E5501**（编译期拒绝+迁移指引文案）；**文件上下文限定**（内联/eval 无 path 不门——实证修正：musk null 语义测试/550 探针族走 run_with_capture 通道，VM 语义测试不应被文件门拦）；tv 文件测试改 run_with_capture_and_path（.at 受门/.as 豁免）；json_is_null 语料按裁定迁 .as+discover .as 回退。硬化矩阵四态（.at+信号=拒/.as=豁/#[script]=豁/#[rust]=压回拒）+sys 原 .at 拒绝实证；**tv 3595/3596+tt 3782/3783 零残留**（唯一红=既有 charts）。P550-D4：CALL null 仍编译期 E0401（糖态裸名静态拦），VM 动态守卫单测钉——端到端探针面维持归 W3 注记
- [x] T15 折叠：全量门禁 `cargo tv` + `cargo tt` + py 五套件三方 +
      KNOWN-DEBT 回写（P555-D2/D5 销号、P550-D4 结案、本波债务登记）
      （验证：门禁输出留档执行注记）
      [✅ 已完成] tv 3595/3596 + tt 3782/3783（唯一红=master 既有 charts P555-D4 甄别，本波 diff 零 ui 文件）；19 py 套件三方：17 全绿 + py_sys/py_list 各 1 例 master 既有红随迁（原 .at 同形失败实证 P560-D6）；KNOWN-DEBT P560 节 P550-D4 更新+D1..D6 登记（P555-D2 于 T02、D5 于 T13 销号）

## 复审记录

**复审人**：zhaopuming（/auto-plan:review，2026-09-05）
**验证场所**：worktree `D:/autostack/.wt/lang-560/auto-lang`（branch plan-560-dev，
fork 95cf3fa32，13 commits；diff 38 文件 +1894/−91：18 代码文件 + 20 套件改名）

### 逐条验收裁定

| # | 验收标准 | 裁定 | 证据（复审复跑） |
|---|---|---|---|
| 1 | §3 规则表全列糖形态探针 + s2s 每规则单测 + dump 真产物 | **PASS** | p01 复跑（sum=15 双形态）/p08（B 族 6-15-1-0）/p09（切片 10-3-4+dump 直达产物 6 处 py_*）/p10（**1024**/134/8）/p11（falsy-0/truthy-2/is true-false）/w7 no_grad **gflag=0**/p13（3-3-true）；tests_s2s 6/6；dump 抽查在案 |
| 2 | Err 通道六条 | **PASS（最小面+双债案）** | p14 复跑：null 值 true/caught+IndexError 载荷全文/fallback（May 通道）/未捕获 exit=1；隐式自动化+严格前缀=P560-D2/D3 债在案 |
| 3 | py 套件三方（.as 载体）+ tv/tt/tvm 全绿 | **PASS（附随迁红注记）** | 五套件 64/64 复跑（新二进制）+14 补迁套件 17/19 全绿——py_sys 0/5/py_list 7/8 为 **master 既有红**（原 .at 在 master 二进制同形失败实证，P560-D6）；tv 3595/3596+tt 3782/3783+tvm 801/801（唯一红=charts 既有甄别） |
| 4 | 550 门控硬化生效 + P550-D4 探针面激活 | **PASS（裁定后补完重验）** | 用户裁定 a+b（2026-09-05）：硬化落地=auto_gate_E5501 诊断错误+文件上下文限定+tv 带 path 通道+json_is_null 迁 .as；**四态矩阵复验**（.at+null=拒/.as=豁/#[rust]=压回拒/#[script]=豁）+sys 原 .at 拒绝；tv 3595/3596+tt 3782/3783 零残留。P550-D4：编译期 E0401 静态拦维持+VM 动态守卫单测钉，端到端糖态面归 W3（注记在案） |
| 5 | 窥孔直达 + CALL_NAT_COUNTED 改名 | **PASS** | dump 直达产物（py-known 糖直落 py_call/py_getitem/py_slice——A1 直呼形态）；CALL_PY 全仓零残留（grep 复核）；tvm 801+infer 17/17 改名后复验 |
| 6 | P555-D2/D5 销号 + 本波债务登记 | **PASS** | P555-D2 于 T02 落地（AST 帧形）/D5 于 T13 改名；KNOWN-DEBT P560 节 D1-D6 六条在案（grep=6） |

### 全量门禁（本计划唯一 tf 运行）

`cargo tf` **3435/3436**——唯一红 = test_charts_gallery_compiles（master 既有，
P555-D4 甄别在案：555 复审已证 diff-无关；本波 diff 18 代码文件亦零 ui_gen）。

### 遗漏 / 延后 / workaround 扫描

- **遗漏**：无——diff 内零新增 TODO/HACK/FIXME（唯一命中系文档措辞）。
- **延后**：**一项已声明待裁**（T14 硬化——非静默，执行注记+待澄清#5+P560-D4
  三处留痕）；with-as（P560-D1）/隐式传播（P560-D2）/载荷前缀（P560-D3）
  为执行中声明并登记的债。
- **Workaround**：四枚执行期 bug 全部实证根修非遮蔽（lexer 双星 peek/
  Str 转义/trans_python 覆写/注解名撞位）；py_sys/py_list 红为甄别后的
  既有债随迁（非本波遮蔽）。

### 计划文 vs 实现分歧清单（均已在标记/注记在案）

1. T05 A1/A2 py-known 门控（设计默认组合子+窥孔 → 实现直呼优先）。
2. T07 C5/C6 解析层直产桥调用（原计划 s2s 规则）。
3. T10 unsized 检测静态不可判裁定（双通道=编译期 sized+显式 py_iter）。
4. T11 最小面+双债（隐式传播/严格前缀）。
5. T12 扩全量 19 套件（计划原文五套件——硬化前提倒逼扩量）。

### 路由（重审更新，2026-09-05）

首轮路由：标准 4 PARTIAL→保持 execution_done 待裁定。**用户裁定 a+b**
（语料改 .as + 本计划内补完再复审）→ T14 补完落地（门控硬化收口提交+
计划回写），标准 4 重验 PASS（四态矩阵+tv/tt 零残留+五套件 64/64 复跑）。

**终裁：六项验收全 PASS，无阻断债 → status: reviewed，可入 /auto-plan:merge。**

重审增量证据：硬化矩阵四态（p3 系 1/0/1/0）+sys 原 .at 拒绝=1；
tv 3595/3596（唯一红=既有 charts）+tt 3782/3783+五套件 64/64+tests_s2s 全绿；
门控影响面实证修正记录于 T14 标记（内联通道不门/文件通道受门）。

## 执行注记

### 终态门禁读数（2026-09-05，15/15 任务落地）

`cargo tv` 3595/3596 · `cargo tt` 3782/3783（唯一红=master 既有 charts，P555-D4 甄别）·
`cargo t vm` 801/801 · tests_s2s 6/6 · 19 py 套件三方 `.as` 载体：17 全绿 +
py_sys 0/5 / py_list 7/8 为 master 既有红随迁（P560-D6 实证）· 探针矩阵 p01-p14 终态复播在案。

### T01 探针基线（2026-09-05，worktree master HEAD 95cf3fa32 干净构建）

| 探针 | 糖形态 | 现状（passthrough .as） | 判读 |
|---|---|---|---|
| p01 | `t.sum()` / `t.sum(dim: 0)` | 运行期 `CALL_SPEC: no function 'PyObj(Tensor).sum'` | A1/A2 缺口在**分派层**（方法调用不走 py_call）——s2s lowering 直呼 py_call 即解 |
| p02 | `w.shape` / `w[0]` | `<obj:4000001>` / `0` | B1 属性=GET_FIELD 垃圾引用；B3 handle 索引疑似已通（GET_ELEM 的 PyObjectHandle GIL 臂，539）——T05 落 B1/B3 lowering 后复核 |
| p03 | `x[2..5]` / `len(x)` | `x[2..5]` 可解析（后续运行）；`len` E0401 链接错 | B5 range 索引词法/语法在；B6 len 非内建——组合子 obj_len 已在（555），s2s lower `len(x)`→`obj_len(x)` |
| p04 | matmul（桥直呼形态） | `134` ✓ | 桥基线绿；C5 `@` 糖未探（词法位待 T07） |
| p05 | for-in over tensor | `0 1 2` ✓ | **E3 array 通道今日已工作**（539 GIL len+getitem）——T10 只需双通道分派（__iter__ 语义源）+ py_iter 糖 |
| p06 | Err 形态（try-catch 拦 py IndexError） | caught，e=`FFI("Python getitem on Tensor failed: IndexError: ...")` | 可捕获但载荷=裸 FFI 串（缺 PyException 类型前缀形态）+ 无隐式传播（手工 try）——T11 目标形态的基线 |
| — | `with` | 未探（预期解析错——关键字未落） | T09 前置：parser `with` 关键字 |



## 待澄清事项

1. **跨模式 import 物化裁出**：`.at`→`.as` 导出自动 `!T`、`.as`→
   `.at` 签名检查（§2）建议**裁出本波**——迁移后真实需求才显形
   （py 套件是 `.as` 自包含形态），且本波已满载；待 W3 前按实际用例
   独立小计划。确认时如需抢入请指出。
2. **py_str/py_is 新桥 ID**：D7/C7 建议占 471（py_is）/472（py_str），
   py 段 450-499 余量充足；如倾向复用现有通道（type_name/组合子）
   请定夺。
3. **Err 载荷形态**：W2 最小面=带类型前缀字符串（`PyException
   TypeError: msg`）；结构化对象（.err_type/.message 字段）是否
   W3 升级——建议按 §4"绑定结构化 Err"留 W3 决（本波字符串先行，
   探针钉格式）。
4. **C3/C4 真值实现位**：建议 s2s lower 到显式调用（零 engine 改动，
   产物可审查）；如倾向 engine JMP 臂直查 handle（性能好但进热臂）
   请定夺。

### 执行期追加（2026-09-05）

5. **550 门控硬化的语料裁定（T14）**：~~需裁定~~ **已裁定（2026-09-05
   用户裁决：a 语料改 .as 后缀 + b 本计划内补完再复审）**。落地实证：
   影响面比预判小——门控经**文件上下文限定**（内联/eval 无 path 不门）
   +tv 文件测试改带 path 通道后，唯 json_is_null 语料需迁 .as（已迁+
   discover .as 回退）；aavm2 词法语料与 keyword_map 经各自通道不受门
   （keyword_map 的 nil 系字符串字面量，解析器不计信号）。
6. **with-as 绑定语法（P560-D1）**：`as` Cast 中缀歧义——裁定方向
   见债务条目（with 限定解析 vs 换绑定关键字）。
