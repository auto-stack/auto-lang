# null 家族语义契约

> 来源：[PLAN-550](../../../../plans/archive/550-null-family-audit.md)（2026-09-05，
> 脚本模式 W0 地基波次；设计源 `docs/design/strategy/script-mode-interop.md` §5/§6）。
> 对应代码：`vm/virt_memory.rs`（共享弹栈助手 + 消息助手）、`vm/engine.rs`
> （opcode 守卫臂 + `tests_null_guards`）、`vm/native.rs`（迭代器通道防御）、
> `parser.rs`/`compile.rs`/`lib.rs`（术语与门控）。

## 范围

AutoVM 对 null 家族（`null`/`nil`/`None` 三拼写）在算术、拼接、索引、调用、
迭代、显式转换、打印八类路径上的运行时语义；`nil` 术语退役；`#[script]`
生产者门控 lint。不涉及脚本模式语法（`.as`/糖/动态分派归 W1+）。

## 值表示前提

- 三拼写经 codegen 归一：`Expr::Nil`/`Expr::Null`/`Expr::None`（Option 构造器）
  → `PUSH_NIL` → `encode_null()` = `NANBOX_BASE | TAG_NULL | (i32::MIN+1)`
  （PLAN-053 P-053-2）。
- **守卫边界 = TAG_NULL only（tag 4）**。历史持久化遗留的 i32 哨兵编码
  （-1 / i32::MIN+1）与真实整数在算术槽不可区分，**不在守卫范围**——
  `nv_is_null_family` 的 i32 分支仅服务 EQ/NE/NULL_COALESCE 判等兼容，
  持久化旧数据经算术仍产垃圾（P550-D3，结构性限制）。
- f64 槽（非 nanboxed）不可能是 null，共享助手经 `!is_f64` 短路。

## 守卫矩阵（opcode/路径 → 行为）

| opcode/路径 | null 输入行为 | 消息（Python 格式） |
|---|---|---|
| ADD/SUB/MUL/DIV/MOD/NEG（含 `_F`/`_D`/`_U64` 变体）任一操作数 null | **TypeError**（`pop_arith_pair_non_null`/`pop_arith_operand_non_null`/`null_guard_peek_pair`） | `TypeError: unsupported operand type(s) for <op>: '<Ta>' and '<Tb>'`（一元：`bad operand type for unary -`） |
| STR_CAT（含 str 的 `+` 的 codegen 静态路由落点；ADD 内嵌拼接分支亦被 pair 守卫覆盖） | **TypeError** | 同上（op 报 `+`，null 方报 `'NoneType'`，另一方报 `'str'`） |
| GET_ELEM 索引对象为 null | **TypeError** | `TypeError: 'NoneType' object is not subscriptable` |
| GET_ELEM 越界（ListData 四型 i32/str/bool/Value） | **IndexError**（负索引合法语义保留） | `IndexError: index N out of range`（报原始索引值） |
| CALL_CLOSURE callee 为 null（动态/脚本路径；正常模式被编译期 E0401 先拦） | **TypeError** | `TypeError: 'NoneType' object is not callable` |
| ARRAY_LEN null（array 通道 for-in 的长度探针；同臂承接 `null.len()`） | **TypeError** | `TypeError: 'NoneType' object is not iterable` |
| shim_iterator_next 接收者 null（迭代器通道防御） | **TypeError** | 同上 |
| TYPE_TO_I32 / TYPE_TO_F64 输入 null（仅 `Expr::To` 显式转换发射） | **TypeError**（原 -1/-1.0 静默臂翻案） | `TypeError: int()/float() argument must be a string or a real number, not 'NoneType'` |
| TYPE_TO_STR / print 输入 null | 输出 `"None"` | a2py `str(None)` 三方对齐 |
| EQ/NE/LT/GT/LE/GE 与 null | **不变**——`x == null` 是合法判空通道 |
| 真值/逻辑（JMP_IF_Z 等） | **不变**——null 为假 |

## 消息与栈帧纪律

- 消息构造集中在 `virt_memory.rs` 模块级助手（`null_binop_type_error`/
  `null_unop_type_error`/`null_to_type_error`/`nv_py_type_name`）与
  `engine.rs` 的 `index_out_of_range_error`（#[cold]）——热递归 match 臂
  保持 2-3 行守卫体（539 三次栈溢出教训）。
- 错误统一走 `VMError::RuntimeError`，现有 try-catch/intercept_error 通道
  直接可捕获，`catch e` 绑定完整消息。

## 术语与生产者门控（lint 级）

- `nil`：parser 双臂（literal/atom）发 W0005 DeprecatedFeature，语义不变；
  CLI 直跑路径可见化 parser 警告（stderr 按名一次性去重；LSP 侧本就消费
  `parser.warnings`）。
- `null`：正字（JSON/JS/GDScript/ArkTS 对齐）。`None`/`Some`：Option 构造器
  身份不动。
- `#[script]` 文件级 pragma：annotation `script` 臂登记
  `parser.script_pragma` → `CompileSession.script_marked`；无 pragma 文件
  含三信号（`use.py` / null / nil 字面量，None/Some 不计入）→ stderr
  迁移提示（.as 指引）。**只警告不拒绝**（硬拒会打断 py parity 套件；
  stderr 不入三方对拍）。codegen 零分支。

## 测试载具

- Rust 单测：`engine.rs tests_null_guards`（13 例——消息格式逐字断言 +
  合法路径回归，含合法 i32(-1)/i32::MIN+1 算术零误伤钉）。
- 探针语料：`scratch/p550/p1..p16`（539 双探针复跑 + 六新探针 + 门控矩阵）。
- 门禁：`cargo tv`（3585）+ `cargo tt`（3772）+ py 五套件三方 64/64。

## 已知边界（债务在案 P550-D1..D7）

- str 索引越界/by-name 缺字段未翻（矩阵限 Auto 数组，P550-D2）。
- CALL null 守卫无 .at 探针面（编译期 E0401 先拦，单测钉，P550-D4）。
- `null.len()` 随 ARRAY_LEN 臂顺带翻转（P550-D5）。
- 越界/TYPE_TO 翻转是计划内行为变更：存量依赖 0/-1 哨兵的仓外语料属预期
  翻案面（P550-D1）。
