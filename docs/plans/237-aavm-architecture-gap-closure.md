# Plan 237: AAVM Architecture Gap Closure — 分阶段拉近与 Rust AutoVM 的距离

## 状态: Phase A-E6 已完成, Phase D (泛型) 基础映射完成 (2026-05-09)

### 已完成
- Phase A: 值多态编码 (eval_expr 支持 int/str/bool)
- Phase B: 类型推断 (typeinfer.at, 54-55 测试)
- Phase C: 字节码编译器 + 解释器 (codegen.at + vm.at, 测试 060-068)
- Phase D: BVM String/Map/List 操作 (7 新 opcode 72-78, 测试 069-074)
  - 额外修复: 051a/052 eval 字符串函数参数测试 (Map.get 返回类型推断 + .len() fallback)
- Phase E1-E4: a2r 转译器 (测试 081-099, 88 个 bootstrap 测试全部通过)
- Phase E5: struct 构造函数 (测试 102, `Point(1,2)` → `Point { x: 1, y: 2 }`)
- Phase E6: 多语句 match arm (测试 101), use.c/use.py FFI (测试 100), 泛型类型映射 (测试 103)
- Phase D 基础: a2r_type 泛型类型映射 `List` → `Vec<i32>`, `Map` → `HashMap<String, i32>`

### 未完成
- Phase D 完整泛型单态化: generics.at 未实现（parser 不解析 `<T>` 泛型参数，无法做运行时单态化）
- Phase E5: Option/Result 匹配, 泛型 struct/enum 完整支持

## 目标

让 AAVM（用 Auto 写的 Auto 编译器）逐步具备与 Rust 版 Auto 编译器相同的架构能力。最终目标是 AAVM 能独立编译和执行 AutoLang 程序，不依赖 Rust 编译器。

## 现状对比

| 维度 | Rust 版 | AAVM 现状 | 差距 |
|------|---------|----------|------|
| 代码量 | 33,834 行 | 5,547 行 | 6x |
| 类型系统 | Type enum (30+ 变体) | TypeInfo (5 种: int/str/bool/void/unknown) | 基础可用 |
| 表达式 | Expr enum (40+ 变体) | 39 NodeKind 枚举 | 大部分对齐 |
| Codegen | 8,716 行，100+ OpCode | codegen.at 703 行 + vm.at 607 行, ~30 OpCode | 最小子集 |
| VM | 4,929 行，task 系统 | 依赖 Rust VM | N/A |
| 泛型/单态化 | GenericRegistry 1,680 行 | 无 | 完全缺失 |
| 类型推断 | infer/ 子系统 8 个文件 | typeinfer.at 207 行 | 简化版 |
| a2r 转译器 | 5,272 行，79 个测试 | a2r.at ~700 行，22 个 bootstrap 测试 | 核心特性覆盖 |
| 值表示 | NaN-boxing u64 (Plan 221) | eval_expr -> int (沿用 VM 编码) | 已解决 |

### 核心瓶颈

**eval_expr 只返回 int，无法区分类型** — evaluator 无法正确表示 str、List、bool 等非 int 值。

## 值表示架构（沿用 Rust VM 编码方案）

### Rust 版方案（Plan 221: NaN-boxing）

Rust 版 AutoVM 有两套值编码，通过 `nanbox` feature flag 切换：

**NanBox 模式**（`Vec<u64>`，64-bit 栈）：利用 IEEE 754 NaN 的空闲 bit pattern 编码类型标签，f64 零开销，所有其他类型用 tagged NaN-boxed 编码。见 `crates/auto-val/src/nano_value.rs`。

**Non-NanBox 模式**（`Vec<i32>`，32-bit 栈，当前默认）：利用 i32 值的范围区分类型：

| 值类型 | i32 编码 | 示例 |
|--------|---------|------|
| `int 42` | 直接存值（>= 0） | `42` |
| `str "hello"` | `-(pool_idx + 1)`（负数） | `-3` = 字符串池索引 2 |
| `bool true` | `i32::MIN` = `-2147483648` | 特殊哨兵值 |
| `bool false` | `i32::MIN + 1` = `-2147483647` | 特殊哨兵值 |
| `nil` | `-2147483647` | 同 false |
| 堆对象 (List/Map/type) | `>= 4000000` | 堆对象 ID |

`shim_print` 区分逻辑（`native.rs:813-830`）：
```rust
if val < 0 { /* 字符串：从池取出 */ }
else { /* 整数：直接打印 */ }
```

### AAVM 方案：沿用 Non-NanBox i32 编码

AAVM 在 Rust VM 上运行，使用 non-nanbox 模式（默认）。**AAVM 的 `eval_expr -> int` 返回值直接沿用 VM 的 i32 编码**：

```auto
// eval_expr 返回的 int 本身就是类型化的：
//   正数/零   → int 值
//   负数      → 字符串（-(pool_idx+1)）
//   -2147483648 → bool true
//   -2147483647 → bool false / nil
//   >= 4000000 → 堆对象引用
```

**不需要 ValueType type**（之前尝试引入 ValueType struct 导致 VM 栈溢出——type 实例字段占 4 个栈槽，递归调用超出 1024 栈大小限制）。

**eval_expr 保持返回 int**，通过辅助函数区分类型：

```auto
// 判断值的类型
fn val_is_str(v int) int {
    if v < 0 {
        if v > -2147483647 { return 1 }
    }
    return 0
}

fn val_is_bool(v int) int {
    if v == -2147483648 { return 1 }
    if v == -2147483647 { return 1 }
    return 0
}

fn val_is_heap_obj(v int) int {
    if v >= 4000000 { return 1 }
    return 0
}
```

**字符串处理**：evaluator 需要访问 VM 字符串池来存取字符串。通过 FFI native（`auto.str.*`）或直接利用 VM 的 `push_str_idx`/`pop_str_idx` 栈操作。

## 阶段路线图

### Phase A: 值多态编码（i32 Bit Pattern）
> 让 eval_expr 能正确返回多种类型的值

**目标**：evaluator 的 `eval_expr -> int` 返回值能编码 int/str/bool/nil/heap-object。

**Rust 对标**：`vm/engine.rs` 的 `push_str_tag` / `pop_tagged`（non-nanbox 模式）

**改动文件**：
- `auto/lib/eval.at` — 修改 eval_expr 使 StrExpr 返回字符串索引

**关键改动**：

1. `eval_expr` 中 `StrExpr` 分支：把字符串内容加入 VM 字符串池，返回 `-(idx+1)`
2. `eval_call` 的 `print` 路径：检测负数返回值，从字符串池取内容
3. `eval_binop` 的 `+` 操作：检测是否有一方是字符串，做拼接而非加法
4. `eval_dot` 的 `.len()`：检测字符串类型，返回字符串长度

**字符串池访问**：evaluator 通过 Auto 的 str 操作（`.len()`, `+` 拼接）直接操作字符串，只在返回值编码时需要字符串池索引。但 Auto 层面无法直接操作 VM 字符串池的 `push_str_idx`...

**替代方案（更实际）**：evaluator 不直接操作 VM 字符串池。而是：
- 变量存储使用平行的两个 Map（`env.int_vars` 存 int, `env.str_vars` 存 str）
- `eval_bind` 根据值类型分别存入不同 Map
- `eval_lookup` 根据变量类型标签从对应 Map 取值
- 类型标签存在 `env.var_types` Map 中

**验证**：
- 现有 eval 测试 038-047 全部通过（print 已支持 StrExpr）
- 新增 048_eval_str_print 通过

**已完成**：
- print 支持 StrExpr 直接输出
- 测试 048_eval_str_print 通过

**下一步**：
- 变量存储双 Map 支持
- eval_expr_str 辅助函数
- 字符串拼接支持

**估计代码量**：eval.at +60~80 行
> 让 AAVM 能推导变量类型和函数签名

**目标**：在 AST 节点上标注类型信息，为后续 codegen 和 List<T> 单态化做准备。

**Rust 对标**：
- `infer/` 子系统（context.rs, expr.rs, stmt.rs, functions.rs, constraints.rs, unification.rs, registry.rs）
- `Codegen.var_types: HashMap<String, Type>`
- `Codegen.fn_return_types: HashMap<String, Type>`

**改动文件**：
- `auto/lib/typeinfer.at` — 新建，类型推断逻辑
- `auto/lib/eval.at` — 集成类型信息

**范围**：

| 推断场景 | 优先级 | 说明 |
|----------|--------|------|
| 字面量类型 | P0 | `42` → int, `"hello"` → str, `true` → bool |
| let/var 绑定 | P0 | `let x = 42` → x: int |
| 函数参数类型 | P0 | `fn add(a int, b int) int` |
| 函数返回类型 | P0 | `fn add(...) int { a + b }` |
| 算术传播 | P1 | int + int → int, int + float → float |
| 调用传播 | P1 | `add(1, 2)` → int (查 fn_return_types) |
| 泛型实例化 | P2 | `List<int>` vs `List<str>` |

**推断策略**（简化版，不做 unification）：

```auto
type TypeInfo {
    kind int      // 0=int, 1=float, 2=str, 3=bool, 4=void, 5=unknown
    elem_type int // for List<T>: element TypeInfo id
}

// 推断在 eval 时自然获得：
// 1. 字面量: 从 NodeKind 直接知道
// 2. 变量: eval_bind 时记录 name -> TypeInfo
// 3. 函数: 解析 fn 签名时记录 params -> TypeInfo, ret -> TypeInfo
// 4. 表达式: 递归组合子表达式类型
```

**验证**：现有测试 + 新增类型推断专用测试。

**估计代码量**：typeinfer.at ~200 行

---

### Phase C: Codegen（字节码生成）
> 从 tree-walking 转向真正的编译器

**目标**：AAVM 能将 AST 编译为 bytecode，像 Rust 版 Codegen 一样。

**Rust 对标**：
- `vm/codegen.rs` (8,716 行) — 完整的 Codegen 结构体
- 100+ OpCode 指令集
- String pool、Object pool、Relocation table

**这是架构转型最关键的一步**。Tree-walking evaluator 在正确性验证完成后，应该被 bytecode emitter 替代。

**改动文件**：
- `auto/lib/codegen.at` — 新建，字节码生成器
- `auto/lib/opcode.at` — 新建，OpCode 常量定义
- `auto/lib/module.at` — 新建，编译产物（Module 结构）

**最小化 Bytecode 指令集**：

| 分类 | 指令 | 说明 |
|------|------|------|
| 常量 | CONST_I32, CONST_STR, CONST_BOOL, CONST_NIL | 加载常量 |
| 算术 | ADD, SUB, MUL, DIV, MOD | 整数运算 |
| 比较 | EQ, NE, LT, GT, LE, GE | 比较操作 |
| 逻辑 | AND, OR, NOT | 逻辑运算 |
| 变量 | LOAD_LOCAL, STORE_LOCAL | 局部变量 |
| 控制 | JUMP, JUMP_IF_FALSE, CALL, RETURN | 控制流 |
| 字符串 | STR_CAT, STR_LEN | 字符串操作 |
| 列表 | LIST_NEW, LIST_PUSH, LIST_GET, LIST_LEN | 列表操作 |
| 内置 | PRINT, DROP | IO 和清理 |

~30 个指令足够覆盖 Phase C 的范围。

**分步实施**：

**C1: 最小编译器**
- 编译 `let x = 1 + 2` → CONST_I32(1), CONST_I32(2), ADD, STORE_LOCAL(0)
- 编译 `print(x)` → LOAD_LOCAL(0), PRINT
- 在 VM 中执行生成的 bytecode

**C2: 控制流**
- if/else → JUMP_IF_FALSE + patching
- for/for-in → 循环标签 + JUMP
- 函数调用 → CALL/RETURN

**C3: 复合类型**
- 字符串操作 → STR_CAT, STR_LEN
- List 操作 → LIST_NEW, LIST_PUSH, LIST_GET, LIST_LEN

**验证**：复用 eval 测试 038-047，但改为通过 bytecode 执行。

**估计代码量**：
- opcode.at ~50 行
- codegen.at ~800 行
- module.at ~60 行

---

### Phase D: 泛型单态化（Generic Monomorphization）
> 支持 List<T>、Map<K,V> 等泛型类型的编译期实例化

**目标**：让 AAVM 能正确处理带类型参数的容器类型。

**Rust 对标**：
- `vm/generic_registry.rs` (1,680 行) — ClassTemplate, ClassType, FieldDef, MethodInfo
- `Codegen.monomorphize()` — 泛型实例化
- `Codegen.track_generic()` / `get_monomorphic_name()`

**改动文件**：
- `auto/lib/generics.at` — 新建，泛型注册表

**核心概念**：

```
// 源码: let list List<int> = List.new()
// 编译时: 实例化 List<int> → 生成 List_int 类型
// 运行时: 所有 List<int> 操作使用特化的 List_int 方法
```

**范围**：
1. 泛型参数收集（从 type 定义和 fn 签名中提取 `<T>` 参数）
2. 使用点实例化（遇到 `List<int>` 时创建 `List_int` 特化）
3. 方法特化（为每个实例化生成类型正确的 `push`/`get`/`len` 方法）

**验证**：新增泛型专用测试（List<int>, Map<str, int>, 嵌套泛型）。

**估计代码量**：generics.at ~300 行

---

### Phase E: a2r 转译器（Auto-to-Rust Transpiler）
> AAVM 自身能生成 Rust 代码

**目标**：用 Auto 写的编译器能把 Auto 源码转译为 Rust 代码。

**Rust 对标**：
- `trans/rust.rs` (5,272 行) — 完整的 a2r 转译器
- 79 个 a2r 测试用例

**前置条件**：Phase A-D 完成后，AAVM 已有完整的类型推断和 codegen 基础设施。

**改动文件**：
- `auto/lib/a2r.at` — 新建，Auto-to-Rust 转译器

**范围**：
1. 类型映射（Auto int → Rust i32, Auto str → Rust String, 等）
2. 表达式转译（算术、比较、调用、闭包）
3. 语句转译（let/var, if/else, for, fn）
4. 类型定义转译（type → struct, enum → enum, ext → impl）
5. 标准库映射（print → println!, List → Vec, Map → HashMap）

**验证**：复用 Rust 版 79 个 a2r 测试用例的输入文件。

**估计代码量**：a2r.at ~1000 行

#### Phase E 已完成

- **E1** (测试 081-086): 基础表达式、函数、变量、if/else、for 循环
- **E2** (测试 087-093): type→struct, enum→enum, use→use, is→match, ext→impl, spec→trait, f-string→format!
  - 同时修复了 AAVM parser：ASTNode 构造函数现在存储结构化数据（字段列表、变体列表、分支列表、方法列表、f-string 部分列表）
- **E3+E4** (测试 094-099): 数组字面量、对象字面量、闭包、错误传播、self字段替换、别名
  - 新增 NodeKind: ArrayExpr, ErrorPropagateExpr (值 37-38)
  - Parser 增强: 数组→ArrayExpr, `.?` 后缀, leading `.` → self.field, 对象存结构化 children, 闭包存 params List
  - a2r 转译: ClosureExpr→`|params| body`, ArrayExpr→`vec![...]`, ObjectExpr→`{ k: v }`, PairExpr→`key: val`, ErrorPropagateExpr→`expr?`, AliasStmt→`type Alias = Type`

#### Phase E 剩余工作

**E5 — 类型系统增强**

| 功能 | Auto | Rust | 难度 | 状态 |
|------|------|------|------|------|
| struct 构造函数 | `Point(1, 2)` | `Point { x: 1, y: 2 }` | 低 | ✅ 测试 102 |
| 类型别名 | `alias List<T> = ...` | `type List<T> = ...` | 中 | ❌ |
| Option/Result 匹配 | `is opt { Some(x) -> ... }` | `match opt { Some(x) => ... }` | 中 | ❌ |
| 泛型支持 | `HashMap<K,V>` | `HashMap<K,V>` | 高 | ❌ |

**E6 — 高级特性**

| 功能 | 难度 | 状态 |
|------|------|------|
| 多语句 match arm | 低 | ✅ 测试 101 |
| use.c / use.py 等变体 | 低 | ✅ 测试 100 |
| F-string 边界情况（转义等） | 低 | ✅ 已有（无需修改） |
| 泛型类型映射 | 低 | ✅ 测试 103 |
| 借用语义 (.view/.mut/.take) | 中 | ❌ |
| Grid/矩阵表达式 | 中 | ❌ |

---

## 实施顺序和依赖关系

```
Phase A (值类型)
    │
    ├── Phase B (类型推断) ── 依赖 A 的 ValueType
    │       │
    │       └── Phase D (泛型单态化) ── 依赖 B 的 TypeInfo
    │
    └── Phase C (Codegen) ── 依赖 A 的值表示
            │
            └── Phase E (a2r 转译器) ── 依赖 C 的编译基础设施
```

**推荐执行顺序**：A → B → C → D → E

但 B 和 C 可以并行开始（C 的初期不依赖类型推断），D 和 E 也可以部分并行。

## 文件结构规划

```
auto/lib/
├── ast.at          # ✅ 344 行 — 39 NodeKind + 构造函数
├── parser.at       # ✅ 1,328 行 — P0+P1, 支持 struct/enum/match/use/impl/trait/fstr/array/object/closure
├── lexer.at        # ✅ 597 行 — P0 Lexer
├── token.at        # ✅ 305 行 — 129 种 TokenKind
├── pos.at          # ✅ 7 行 — Pos 位置类型
├── error.at        # ✅ 7 行 — Error 类型
├── eval.at         # ✅ 702 行 — tree-walking evaluator
├── typeinfer.at    # ✅ 207 行 — 简化版类型推断
├── codegen.at      # ✅ 703 行 — 字节码生成器
├── vm.at           # ✅ 607 行 — 字节码解释器
├── opcode.at       # ✅ 77 行 — OpCode 常量
├── a2r.at          # ✅ ~700 行 — Auto-to-Rust 转译器 (E1-E6)
└── generics.at     # ❌ 未实现 — 泛型单态化
```

## 测试规划

每个 Phase 有独立的测试目录，沿用 `test/vm/99_bootstrap/` 编号：

| Phase | 测试编号 | 测试内容 | 状态 |
|-------|---------|---------|------|
| A | 048-052 | 值类型：str 返回、bool 返回、List 返回、nil 返回、混合运算 | ✅ |
| B | 053-057 | 类型推断：字面量推断、变量推断、函数签名、返回类型、传播 | ✅ |
| C1 | 058-062 | 最小 codegen：常量加载、算术运算、变量存取、print、函数调用 | ✅ |
| C2 | 063-067 | 控制流 codegen：if/else、for 循环、嵌套函数、递归、break | ✅ |
| C3 | 068-074 | 复合类型 codegen + BVM str/map/list ops | ✅ |
| D | 075-080 | 补充测试 + list_str_loop | ✅ |
| E1 | 081-086 | a2r 基础：hello、fn、var、if、for、str | ✅ |
| E2 | 087-093 | a2r 结构化：use、type、enum、is、ext、fstr、spec | ✅ |
| E3+E4 | 094-099 | a2r 表达式补全：closure、alias、object、array、error_prop、self_field | ✅ |
| E5 | 102 | struct 构造函数：`Point(1,2)` → `Point { x: 1, y: 2 }` | ✅ |
| E6 | 100-101, 103 | use.c/use.py FFI、多语句 match arm、泛型类型映射 | ✅ |
| D (泛型) | TBD | 泛型单态化：List<T>、Map<K,V> 实例化 | ❌ |

## 风险与缓解

| 风险 | 概率 | 缓解措施 |
|------|------|---------|
| ValueType 在 Auto 中难以表达（无 enum） | 中 | 利用 VM 动态类型 + struct 组合模拟 |
| Codegen 指令集设计过小需反复扩展 | 低 | 参照 Rust 版指令集，一次性定义够用的子集 |
| 单态化复杂度爆炸 | 中 | 先只支持 List 和 Map，其他泛型延后 |
| a2r 转译器工作量大 | 高 | 分阶段，先支持脚本子集，再扩展到完整语言 |
| 现有 39 个测试回归 | 低 | 每个 Phase 变更后全量回归 |

## 里程碑

| 里程碑 | 完成标志 | 状态 |
|--------|---------|------|
| M1: 值多态 | eval_expr 正确返回 int/str/bool/list | ✅ Phase A |
| M2: 类型感知 | 编译器知道每个表达式的类型 | ✅ Phase B |
| M3: 字节码执行 | AAVM 能编译+运行简单程序 | ✅ Phase C |
| M4: 泛型支持 | List<T> 正确实例化和操作 | ❌ Phase D |
| M5: 自举能力 | AAVM 能转译自身为 Rust 代码 | 🔄 Phase E (E1-E6 完成, 92 测试通过) |
