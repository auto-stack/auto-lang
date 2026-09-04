---
plan_id: PLAN-550
status: execution_done      # drafting → executing → execution_done → reviewed → archived
feature_name: null-family-audit
author: [zhaopuming]
created_at: 2026-09-04
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm]       # 受影响的 specs 路径
current_step: 12
total_steps: 12
---

# [PLAN-550] null 家族审计与术语统一（脚本模式 W0）

## 变更摘要

脚本模式（AutoScript，设计源
[script-mode-interop.md](../design/strategy/script-mode-interop.md) §5/§6）
的地基波次：把 VM 对 null 家族的"静默垃圾"行为全部翻转为响亮的、可捕获的
TypeError，统一三个空值拼写的术语，并落下正常模式 null 禁令的第一层
（生产者门控）。不涉及任何脚本模式语法——`.as`/糖/动态分派归后续波次。

Plan 539 执行期实证的两个探针是本计划的立项依据（master 现状）：

```auto
var x = null
print((x + 1).to(str))     // → -2147483646（null 的 nanbox 位模式当 i32 解码再 +1）
var s = "a"
print((s + y).to(str))     // y=null → "a-2147483647"（垃圾数字入字符串拼接）
```

## 目标

1. null 参与算术/拼接/索引/调用/迭代 → `VMError`（消息按 Python 格式，
   现有 try-catch 立即可捕获，`catch e` 绑定消息）；
2. `.to(int)` 对 null 的静默 -1 臂翻案为 TypeError（排查 `?.` 内部路径
   依赖后实施）；
3. 数组越界从静默 0 哨兵翻转为 Err（对标 Python IndexError）；
4. `print(null)` / `null.to(str)` 输出 `"None"`（a2py 三方 parity）；
5. 术语统一：`null` 为正字（JSON/JS/GDScript/ArkTS 对齐）；`nil` 保留为
   别名并出 deprecated 警告；`None` 维持 Option 构造器身份不动；
6. 正常模式生产者门控（lint 级）：`#[script]` pragma 登记文件级脚本标注；
   无 pragma 文件含 `use.py` 或 null/nil 字面量 → 编译警告指向 .as 迁移
   （`.as` 扩展名与模式管线本身归 W1，本期只落 pragma + 警告通道，**只警
   告不拒绝**——硬拒会打断现有 py parity 套件）；
7. 未初始化 `var x`：盘存现状并处置（允许则正常语义禁止，不允许则记录）。

### 非目标（Out of Scope）

- `.as` 扩展名、模式管线、s2s 改写器（W1）；
- 糖与动态分派（W2）；注解预言机（W3）；
- `*T` 裸指针可空性（规则已在设计文档 §5 写死，实现随指针特性落地）；
- catch 拦值传播（Err 结构化载荷）——本期 TypeError 走 VMError/
  intercept_error 现有通道即可捕获，Err 值通道归 W1/W2；
- py 侧行为（`py_call(null, ...)` 已由 539 异常通道覆盖，不动）；
- P539-D2（`.len()` 类型谎言）——同族病灶但独立债务，不在本计划。

## 架构方案

### 现状基线（考古结论）

- **空值三层词汇**（token.rs:370-372,435）：`nil`→`TokenKind::Nil`→
  `Expr::Nil`（旧层裸空，开放区间 `x[a..]` 空端在用）；`null`→
  `TokenKind::Null`→`Expr::Null`（新一代裸空，TAG_NULL/encode_null 机器 +
  539 套件正字）；`None`→`TokenKind::NoneKW`→`Expr::None`（Plan 120
  Option 构造器，PUSH_NIL 压 TAG_NULL）。**运行期三者同落 null-family**
  （`nv_is_null_family`：TAG_NULL + i32(-1) + i32(MIN+1)；EQ 判等
  P-053-2 全族相等）——词法三层皮、运行期一家。
- **静默垃圾病灶**：算术族操作数弹栈（`pop_arith_operand`/各 opcode 臂）
  对 null 无防御，位模式直接 decode_i32 参与运算（探针实证）；GET_ELEM
  越界 → `push_i32(0)`；TYPE_TO_I32/F64 含 null→-1/-1.0 静默臂（Plan 539
  T05 自加，本计划翻案）。
- **修复模式**：算术/拼接守卫收敛到共享弹栈助手（`pop_arith_operand`
  家族加"拒收 null"路径）；错误统一
  `VMError::RuntimeError("TypeError: unsupported operand type(s) for
  <op>: 'NoneType' and '<T>'")` 风格，intercept_error 现有 try-catch
  直接可捕（539 p5b 同型实证）。
- **栈帧纪律**（539 三次溢出教训）：守卫体 2-3 行，消息构造放模块级助手
  函数，不内联大块进热递归 match 臂。

### 设计要点

1. **共享助手优先**：null 检查尽量收敛进 `pop_arith_operand`/
   `pop_tagged` 等共享路径（一处修全族继承）；个别入口（GET_ELEM、
   CALL、迭代源、ADD 拼接臂内嵌解码）单独加守卫。
2. **越界翻转的兼容边界**：GET_ELEM 越界→IndexError 改变现有语料行为
   ——依赖 0 哨兵的在档用例由 `cargo tv` 全量门禁裁决；若存量面过大，
   降级为：脚本/动态路径维持 0 并登记债务，正常路径（可判定）Err，
   复审裁定。
3. **print/to(str) 保真**：null 家族打印统一 `"None"`（a2py `str(None)`
   对齐）；TYPE_TO_STR null 臂 + print unified shim 同步。
4. **术语层**：parser 对 `nil` 发 deprecation 警告，语义保持 null 同义
   （运行期本就一家）；`Value::Nil` 作为 `#[default]` 内部表示短期不动。
5. **生产者门控（lint 级）**：compile session 登记 `#[script]` 文件级
   pragma；无 pragma 文件含 `use.py`/null/nil → 编译警告（文案含 .as
   迁移指引）。codegen 仅传递标志不分支——热臂零增长。

## 技术栈

- `crates/auto-lang/src/vm/engine.rs`（算术/GET_ELEM/CALL/迭代源守卫 +
  TYPE_TO 翻案 + print 分支 + 越界翻转）；
- `crates/auto-lang/src/vm/virt_memory.rs`（pop_arith_operand 家族
  null 拒收）；
- `crates/auto-lang/src/parser.rs` + `crates/auto-lang/src/lexer.rs`
  （nil 警告、#[script] pragma、未初始化 var 盘存与禁止）；
- `crates/auto-lang/src/compile.rs`（session 脚本标注字段）；
- 探针：`scratch/p550/`（539 惯例，先证行为再改码）；
- 回归：`cargo tv`（全量 VM 门禁）+ `cargo tt`（a2py print 形态）+
  py 五套件三方（math/torch/infer/train/numpy——T18 None→null 映射的
  消费侧是 null 语义变更最大回归面）。

## 需求分析与背景调查

- 设计源：`docs/design/strategy/script-mode-interop.md` §5（null 体系）/
  §6（W0 审计清单）/§9（波次骨架）——2026-09-04/05 与用户逐条裁决
  定稿（裁决存档 §10）。
- spec 关联：GOAL-005（Python parity 线，539 推进到训练/推理脚本级；
  null 正确性是其消费侧地基）；VM 运行时正确性属 auto-lang/vm spec 面。
- 539 在案债务联动：P539-D2（`.len()` 类型谎言——同"静默垃圾"家族，
  边界外不修）；DIV-PY-CLOSURE-1（无关）。
- 立项实证：`scratch/p539/null_probe.at`（双探针 2026-09-04 master
  二进制复跑在案：`null+1`→-2147483646、`"a"+null`→"a-2147483647"）。

## 详细设计

### 守卫矩阵（opcode/路径 → 行为）

| opcode/路径 | 现状 | 目标 |
|---|---|---|
| ADD/SUB/MUL/DIV/MOD/NEG（+_F 变体）任一操作数 null | 位模式垃圾 | `TypeError: unsupported operand type(s) for <op>: 'NoneType' and <T>` |
| ADD 字符串拼接臂（null 参与拼接） | 垃圾数字入串 | 同上（op 报 `+`，类型报 str） |
| EQ/NE/LT/GT/LE/GE 与 null 比较 | null-family 判等 | **不变**——`x == null` 是合法判空通道 |
| 真值/逻辑（JMP_IF_Z 等） | null 为假 | **不变** |
| GET_ELEM 索引对象为 null | 未验证/疑垃圾 | `TypeError: 'NoneType' object is not subscriptable` |
| GET_ELEM 越界（Auto 数组） | push_i32(0) | `IndexError: index N out of range`（存量面门禁裁决） |
| CALL callee 为 null | 未验证 | `TypeError: 'NoneType' object is not callable` |
| for-in 迭代源为 null | 疑静默零迭代 | `TypeError: 'NoneType' object is not iterable` |
| TYPE_TO_I32 / TYPE_TO_F64 输入 null | -1 / -1.0 | TypeError（`?.` 内部依赖前置排查） |
| TYPE_TO_STR / print 输入 null | 待探针 | `"None"` |

### 术语与门控

- `nil` → parser 发 `warning: 'nil' is deprecated, use 'null'`（复用
  parser 现有警告通道；无则 stderr 一次性去重）；语义不变。
- `#[script]` 文件级 pragma：parser 收集 → compile session 字段；门控
  lint：无 pragma + 文件含 use.py/null/nil → 编译警告（文案：疑似脚本
  内容，W1 起建议改名 .as 或标注 #[script]）。
- 未初始化 `var x`：T02 探针盘存（允许与否、未初始化值形态），正常
  语义 parser 报错 `variable 'x' must be initialized`；#[script] 豁免。

## 测试设计

- **探针先行**（scratch/p550/）：改码前复跑 539 双探针 + 新增六探针
  （null[0] / null(x) / for-in-null / null.to(int) / print(null) /
  arr[999]），现状记录入执行注记；
- **Rust 单测**：engine 守卫单测（构造 task/vm 直推 null 调 opcode，
  断言 VMError 消息格式——539 py_ffi 单测同型）+ TYPE_TO 翻案回归；
- **try-catch 端到端**：探针 .at 验证 `try { null + 1 } catch e` 捕获
  且 e 含 "TypeError: unsupported operand"；
- **三方 parity**：`print(null)` 扩例入 py_torch_infer（a2py 出
  `print(str(None))`="None"，三方对齐）；
- **门禁**：`cargo tv` 全量 + `cargo tt` + py 五套件三方。

## 验收标准

1. 539 双探针翻转：`null+1` 与 `"a"+null` 从静默垃圾变为可 try-catch
   捕获的 TypeError（消息含 'NoneType'）；
2. 守卫矩阵全表落地（六新探针各得其所：subscriptable/callable/
   iterable 三 TypeError + 越界 IndexError + to(int) TypeError +
   print "None"）；
3. `nil` 出 deprecated 警告且语义不变；`None`/`Some` 行为零回归；
4. 生产者门控 lint 生效（无 pragma 三信号出迁移警告、#[script] 豁免），
   py 五套件三方全绿（警告不破坏运行）；
5. `cargo tv` + `cargo tt` 全绿（越界翻转引入的存量红：属计划内行为
   变更的用例修复，或按设计要点 2 降级并登记债务，复审裁定）；
6. 未初始化 `var` 盘存结论在案：允许则禁止已实施，不允许则记录证据。

## 执行步骤

- [x] T01 探针基线：`scratch/p550/` 复跑 539 双探针 + 新增六探针
      （null[0]/null(x)/for-in-null/null.to(int)/print(null)/arr[999]），
      现状输出记录入执行注记（验证：八探针逐条可复跑）
      [✅ 已完成] 2026-09-05 worktree master HEAD（5d4919080）二进制八探针全复跑在案 + p9 try-catch 基线 + p12/p13 nil/None 基线，逐条输出见下方执行注记 T01
- [x] T02 未初始化 `var x` 盘存探针 + 结论记录入执行注记（验证：探针
      .at 在案）
      [✅ 已完成] p10/p11 探针在案：`var x` 与 `var x: int` 均被 parser E0007 拒绝——现状即禁止，无未初始化值形态；T11 走记录证据分支
- [x] T03 算术族守卫：`virt_memory.rs` pop_arith_operand 家族 null 拒收
      （TypeError VMError，Python 格式消息助手），engine 算术臂 `?`
      传播；ADD 拼接臂单独守卫（验证：双探针翻转 + try-catch 端到端
      探针）
      [✅ 已完成] 双探针翻转在案：p1 `null+1`→TypeError('NoneType' and 'int')、p2 `"a"+null`→TypeError('str' and 'NoneType')；p9 try-catch 捕获且 e 绑定消息。拼接病灶实际落点=STR_CAT 臂（codegen 对含 str 的 + 静态路由），与 ADD 内嵌拼接臂双守卫。守卫只拒 TAG_NULL（三拼写经 PUSH_NIL 同落 tag-null，PLAN-053 归一）；历史 i32 哨兵与真实 -1 不可区分故不守卫（合法 `-2147483647+1` 回归探针绿）。_F/_D/_U64/MOD 族经 null_guard_peek_pair 前缀守卫覆盖
- [x] T04 GET_ELEM/CALL/迭代源守卫：null 对象三分支 TypeError（验证：
      三探针翻转）
      [✅ 已完成] p3 subscriptable / p5 not iterable 双探针翻转在案（p5b try-catch 捕获验证）；callable 探针不可达（p4 = 编译期 E0401），守卫落 CALL_CLOSURE 动态路径以 T08 单测验证。迭代病灶实际落点=ARRAY_LEN null 静默 0 臂（array 通道 for-in 的长度探针，顺带翻 null.len()）；shim_iterator_next 迭代器通道另加防御守卫。合法 for-in（list/range）回归绿 + cargo t engine 25/25
- [x] T05 越界翻转：GET_ELEM 越界 0→IndexError（验证：探针 + `cargo tv`
      存量面裁决——红则按设计要点 2 处置）
      [✅ 已完成] p8 `arr[999]`→IndexError: index 999 out of range；p8b 矩阵：负索引合法面（a[-1]/a[-3]）保留、a[-5] 可 catch、str 索引非翻转面不动。**cargo tv 3572/3572 全绿**——存量面零撞击，全量翻转成立，设计要点 2 降级路径不需要（str 越界/by-name 缺字段未翻，越界语义按矩阵限 Auto 数组）
- [x] T06 TYPE_TO_I32/F64 null 臂翻案：-1/-1.0 → TypeError；先排查
      `?.`/NULL_COALESCE 内部依赖（验证：探针 + `cargo tv`）
      [✅ 已完成] 前置排查：TYPE_TO_I32/F64 仅由 Expr::To（显式 .to(int)/.to(float)）发射；NULL_COALESCE 是独立 opcode（自带 null-family 逻辑），`?.` 语言尚不存在——**无内部哨兵依赖**（待澄清#2 关闭）。p6 `null.to(int)`→TypeError 翻转；p6b 矩阵：to(float) null 同翻、合法转换（"42"/"1.5"/true/3.to(str)）零回归。cargo tv 3572/3572 全绿
- [x] T07 print/to(str) 保真：null 打 `"None"`（TYPE_TO_STR null 臂 +
      print unified shim）（验证：探针 + py_torch_infer 扩例三方绿）
      [✅ 已完成] p7 双形态翻转：print(null)→"None"（print shim 已然）+ null.to(str)→"None"（TYPE_TO_STR 新 null 臂，原落 i32 兜底 -2147483647）；py_torch_infer 扩例 test_print_none（.to(str) 断言形态，a2py 出 str(None)）——三方 17/17 (100%) 绿
- [x] T08 Rust 单测：守卫消息格式断言 + TYPE_TO 翻案回归（engine 模块，
      539 单测同型）（验证：`cargo t vm` 含新增用例全绿）
      [✅ 已完成] engine.rs 新增 tests_null_guards 模块 13 例：算术双形态消息（NoneType 左/右）、SUB/MUL/DIV/MOD 参数化、NEG 一元、STR_CAT 拼接（'str' and 'NoneType'）、合法 -1/MIN+1 算术零误伤、subscriptable/callable（p4 编译期不可达路径的单测钉住）/iterable、IndexError 999、TYPE_TO_I32/F64 翻案+合法回归、TYPE_TO_STR→"None"。**cargo t vm 800/800 全绿**（ui-iced 档经待澄清#4 修复已可运行）
- [x] T09 `nil` deprecated 警告（parser，语义不变）（验证：探针出警告
      + 行为不变 + `cargo tv`）
      [✅ 已完成] parser 双臂（literal/atom）发 W0005 DeprecatedFeature；CLI 直跑路径可见化 parser 警告（stderr 按名一次性去重——p12b 两次 nil 仅一条警告；LSP 侧本就消费 parser.warnings；tv 语料对拍 stdout 不受 stderr 影响）。p12b `nil==null`→true 语义不变；p13 None/Some 零回归无警告。**cargo tv 3585/3585 全绿**
- [x] T10 `#[script]` pragma + 生产者门控 lint（警告级，.as 指引文案）
      （验证：无 pragma 含 use.py 文件出警告、pragma 豁免探针）
      [✅ 已完成] p14 use.py 信号→警告（运行不受阻）；p15 `#[script]` 豁免（use.py+null 双信号静默）+ 正常运行；p16 裸 null 信号→警告；无信号文件零误报。parser.script_pragma/saw_bare_null → CompileSession.script_marked（mark_script 登记，codegen 零分支）；annotation match 新增 script 臂（否则 Unknown annotation 硬错——顺带使其可解析）。py_math 20/20 三方烟测：stderr 警告不破坏 parity 对拍
- [x] T11 未初始化 var 处置：按 T02 结论实施禁止或记录（验证：探针 /
      `cargo tv`）
      [✅ 已完成] 记录证据分支：T02 盘存结论=现状即禁止（p10/p11 探针，裸 `var x` 与 `var x: int` 均 parser E0007 拒绝，无未初始化值形态），证据在执行注记 T02 节——无需实施禁止，验收标准 6 后半分支成立
- [x] T12 折叠：KNOWN-DEBT 回写（越界/翻案行为变更登记）+ 全量门禁
      `cargo tv` + `cargo tt` + py 五套件三方（验证：门禁输出留档）
      [✅ 已完成] KNOWN-DEBT P550-D1..D7 登记在案（翻案存量语义/str 越界未覆盖/i32 哨兵不可守卫/CALL 守卫单测钉/null.len 顺带翻/门控信号面/master cargo t 修复注记）；**cargo tv 3585/3585 + cargo tt 3772/3772 全绿**；py 五套件三方全绿（py_math 20/20 + py_torch 7/7 + py_torch_infer 17/17 + py_torch_train 10/10 + py_numpy 10/10 = 64/64，生产者门控 stderr 警告不破坏对拍）；终态全探针复播矩阵见执行注记

## 复审记录

（/auto-plan:review 填写）

## 执行注记

### T01 探针基线（2026-09-05，worktree master HEAD 5d4919080 干净构建二进制）

探针文件 `scratch/p550/p1..p13`（未跟踪 scratch，复跑命令
`<bin> scratch/p550/<file>.at`）：

| 探针 | 现状输出 | 判读 |
|---|---|---|
| p1 `null+1` → to(str) | `-2147483646` | 539 探针复现：TAG_NULL 位模式当 i32 解码（-2147483647）再 +1 |
| p2 `"a"+null` → to(str) | `a-2147483647` | 539 探针复现：垃圾数字入字符串拼接 |
| p3 `null[0]` | `0` | GET_ELEM 静默哨兵：null 位模式当 obj_id 解码→查找落空→push_i32(0) |
| p4 `var f = null; f(1)` | **编译期 E0401 Undefined symbol: f** | 非 VM 路径！p4b（闭包中转变量）/p4c（map 字段中转）同样 E0401——静态解析拦截。T04 的 callable 守卫落 VM 动态路径（CALL_CLOSURE/CALL_SPEC），.at 探针不可达，改以 Rust 单测验证（并入 T08） |
| p5 for-in 迭代 null | `done`（零迭代） | 疑静默零迭代证实 |
| p6 `null.to(int)` | `-1` | TYPE_TO_I32 null 静默臂（engine.rs:3596） |
| p7 `print(null)` / `null.to(str)` | `None` / `-2147483647` | **print shim 已输出 "None"**（T07 实际工作量=TYPE_TO_STR null 臂，engine.rs:3565 else 兜底解码 i32） |
| p8 `arr[999]` | `0` | GET_ELEM 越界 0 哨兵证实 |
| p9 try-catch 基线 | `-2147483646` + `no-throw` | 现状 null+1 不抛错，catch 不触发 |
| p12 `nil+1` / `nil==null` | `-2147483646` / `true` | T09 改动前语义参考（nil=i32(MIN+1)，全族判等） |
| p13 `None==null` / `Some(1)==null` / `None==None` | `true` / `false` / `true` | Option 构造器行为参考（T09 零回归基线） |

### T02 未初始化 var 盘存结论（2026-09-05）

- p10 `var x`（裸）→ **E0007** `Variable 'x' must have either a type annotation or an initial value`（parser 拒绝）
- p11 `var x: int`（带注解无初值）→ 同 E0007 拒绝

**结论：现状即"不允许"**——语言当前不存在未初始化 var 形态（带注解无初值也被拒），
T11 无需实施禁止，仅记录证据（本节即证据；验收标准 6 后半分支成立）。

### 终态探针矩阵（2026-09-05，plan-550-dev 全 12 任务落地后复播）

| 探针 | 基线（master） | 终态（plan-550-dev） |
|---|---|---|
| p1 `null+1` | `-2147483646` 静默垃圾 | **TypeError: unsupported operand type(s) for +: 'NoneType' and 'int'** |
| p2 `"a"+null` | `a-2147483647` 垃圾入串 | **TypeError: ... for +: 'str' and 'NoneType'**（STR_CAT 臂） |
| p3 `null[0]` | `0` 静默哨兵 | **TypeError: 'NoneType' object is not subscriptable** |
| p4 null callee | 编译期 E0401 | 不变（VM 动态路径守卫由单测钉住，P550-D4） |
| p5 for-in null | 静默零迭代 | **TypeError: 'NoneType' object is not iterable** |
| p6 `null.to(int)` | `-1` 静默 | **TypeError: int() argument must be ..., not 'NoneType'** |
| p7 `print(null)` / `null.to(str)` | `None` / `-2147483647` | `None` / **`None`** |
| p8 `arr[999]` | `0` 哨兵 | **IndexError: index 999 out of range** |
| p9 try-catch | no-throw | **caught** + e 绑定完整消息 |
| p12b nil 双写 | 无警告 | `'nil' is deprecated` 警告一次 + 语义不变（nil==null true） |
| p13 None/Some | 全家判等正常 | 行为零回归（含 `o == null` 裸字面量触发的门控提示，属预期） |
| p14/15/16 门控 | — | use.py/null 信号出迁移警告、#[script] 豁免、无信号零误报 |

终态门禁读数：`cargo tv` 3585/3585 · `cargo tt` 3772/3772 ·
`cargo t vm` 800/800 · py 五套件三方 64/64 · `cargo t engine` 25/25。

## 待澄清事项

1. **越界翻转存量面**：GET_ELEM 0→IndexError 若撞大量在档语料，降级
   路径（可判定路径 Err/动态路径 0 + 债务登记）已预案，由 T05 门禁
   数据裁定。
2. **`null.to(int)` 内部依赖**：`?.`/迭代哨兵是否依赖 -1 静默臂——T06
   前置排查，若依赖则该内部路径改显式哨兵不经过 TYPE_TO。
3. **未初始化 var 现状**：T02 探针定（允许则禁、不允许则记录证据）。
   ——已裁定：现状即禁止（E0007），T11 记录证据分支。
4. **master cargo t 断裂（执行中发现，非本计划病灶）**：plan051 合入
   （26211362c）在 renderer.rs:8606/14650 留下对
   `autodown_editor::retheme_all_fence_buffers` 的**无条件调用**，而模块
   门控 `#[cfg(all(feature="autodown", feature="code-editor"))]`——
   ui-iced-only 档（`cargo t` 别名档）编译断裂。本计划已在 worktree
   以同 cfg 补门修复（e1d8fd097，随分支折入 master）；plan051 复审方
   知悉，若该修复与其意图不符请回馈。
