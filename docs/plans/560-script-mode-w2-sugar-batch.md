---
plan_id: PLAN-560
status: executing             # drafting → executing → execution_done → reviewed → archived
feature_name: script-mode-w2-sugar-batch
author: [zhaopuming]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/frontend, auto-lang/trans, auto-cli]   # 受影响的 specs 路径
current_step: 0
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

- [ ] T01 探针基线：`scratch/p560/` §3 全表糖形态 + Err 六条形态在
      现状 `.as`（passthrough）下的行为逐条记录（验证：探针可复跑，
      基线入执行注记）
- [ ] T02 s2s 升级：AST 发射器（`trans/emit.rs` 新模块，AST→Auto 源
      打印）+ 链式规则语义（规则表有序全过+产物再解析）——P555-D2
      裁定落地（验证：identity round-trip 稳定单测 + 链式注入单测绿）
- [ ] T03 管线激活：`lib.rs` execute_autovm_with_path 对
      `ScriptMode::Script` 源走 `lower_source` 产物编译；
      `--dump-lowered` 翻转真产物（验证：`.as` 空规则=行为与 W1 逐
      字节同回归探针 + dump 留档）
- [ ] T04 B7/B8 桥补：`py_ffi.rs` py_contains(470) + 裸模块句柄
      （B8，use.py torch 绑模块对象）+ B9 dotted 递归（验证：桥单测
      + `use.py torch` 后 `torch.nn.Linear` 可达探针）
- [ ] T05 A/B 族规则（一）：A1-A4 调用 + B1/B9 属性 + B2/B3/B4 赋值
      索引 lowering 规则 + 单测（验证：source-to-source 单测 + torch
      探针 `t.sum(dim: 0)`/`w.shape`/`x[i]`）
- [ ] T06 A/B 族规则（二）：B5 切片族（含步长 `a..b..c` 实现——词法
      步长位落地）+ B6 len + A5 闭包自动 py_callable + D7 句柄
      print/f-string GIL str()（验证：单测 + `x[1..3]`/`len(x)`/
      print(tensor) 探针）
- [ ] T07 C 族（一）：`@` 词法消歧（`@T` 引用不动）+ `a @ b`→
      py_matmul lowering；`a ** b`→Math.pow/`__pow__`（py_ffi dunder
      表 POW 臂）（验证：单测 + `a @ b`/`a ** 2` 探针）
- [ ] T08 C 族（二）：`a is b`→GIL is（py_is 471）+ C3/C4 句柄真值
      （s2s lower 到显式 GIL bool 调用，多元素张量按 Python 抛）
      （验证：单测 + `if t:`/`a is None` 探针）
- [ ] T09 E2 with：parser `with` 关键字（`with expr {}`/`with expr
      as x {}`）+ s2s 语句规则 → py_enter + try-finally + py_exit
      （异常抑制不做）（验证：单测 + no_grad 探针——infer 套件 14 例
      形态）
- [ ] T10 E3 for-in 双通道：handle 源 `py_iter()` 强制迭代器 + Auto
      源 array 通道（现状）分派（验证：tensor for-in 探针 + dict
      items 迭代探针）
- [ ] T11 Err 值通道：隐式 `!T` 传播四作用域（ERROR_PROPAGATE 发射）+
      catch 拦值绑定 `PyException <type>: <msg>` + main 带错退出 +
      a2py 映射（验证：六条探针 + py 套件 15/16 例 May 通道回归）
- [ ] T12 迁移：py 五套件 `tests/auto/*.at`→`*.as` 逐套件改名 +
      parity runner glob（`.at|.as`）+ a2py 接受 `.as`（验证：五套件
      三方逐套件全绿）
- [ ] T13 窥孔+改名：use.py 静态已知调用点 s2s 产物直呼 py_xxx +
      CALL_PY→CALL_NAT_COUNTED 全链改名（发射/执行/既有 450-469 面）
      （验证：`--dump-lowered` 直达产物抽查 + `cargo t vm` 全绿）
- [ ] T14 硬化：550 门控 `.at` 含 use.py/null/nil 警告→诊断错误
      （`.as`/`#[script]`/`#[rust]` 豁免不变）+ P550-D4 CALL null
      端到端探针补案（验证：硬化探针矩阵 + `cargo tv` 存量零残留红）
- [ ] T15 折叠：全量门禁 `cargo tv` + `cargo tt` + py 五套件三方 +
      KNOWN-DEBT 回写（P555-D2/D5 销号、P550-D4 结案、本波债务登记）
      （验证：门禁输出留档执行注记）

## 复审记录

（/auto-plan:review 填写）

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
