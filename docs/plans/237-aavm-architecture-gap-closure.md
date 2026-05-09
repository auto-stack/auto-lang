# Plan 237: AAVM Architecture Gap Closure — 分阶段拉近与 Rust AutoVM 的距离

## 状态: Phase A-E6 + D1 已完成, 95 个 bootstrap 测试通过 (2026-05-09)

### 已完成

| Phase | 测试编号 | 内容 | 代码量 |
|-------|---------|------|--------|
| A 值多态 | 048-052 | eval_expr 支持 int/str/bool/nil/heap | eval.at 703 行 |
| B 类型推断 | 053-057 | 字面量→变量→函数签名类型传播 | typeinfer.at 227 行 |
| C 字节码 | 058-074 | codegen + vm + BVM str/map/list (30 OpCode) | codegen.at 704 行 + vm.at 608 行 |
| D 基础 | 075-080 | 补充测试 | — |
| E1-E4 a2r | 081-099 | 基础表达式→struct/enum/impl/trait/closure/array/object | a2r.at ~750 行 |
| E5 构造函数 | 102 | `Point(1,2)` → `Point { x: 1, y: 2 }` | +struct_fields Map |
| E6 高级特性 | 100-101, 103 | 多语句 match arm, use.c/py FFI, 泛型类型映射 | — |
| D1 泛型解析 | 104-106 | Parser `<T>` 读取 + a2r 泛型转译 + enum/spec/ext 泛型输出 | +parser_try_read_generic_type, a2r_type char_at 解析 |

**总计**: 12 个 Auto 源文件, ~5,700 行, 95 个 bootstrap 测试

### 未完成

| 编号 | 内容 | 优先级 | 估计量 |
|------|------|--------|--------|
| D2 | generics.at 泛型注册表 (类型字符串替换) | 中 | ~300 行 |
| E5 | Option/Result 匹配 (`is opt { Some(x) -> ... }`) | 高 | ~150 行 |
| E5 | 借用语义 (`.view`/`.mut`/`.take` → `&`/`&mut`) | 中 | ~100 行 |
| E6 | Grid/矩阵表达式 | 低 | ~200 行 |

## 目标

让 AAVM（用 Auto 写的 Auto 编译器）逐步具备与 Rust 版 Auto 编译器相同的架构能力。最终目标是 AAVM 能独立编译和执行 AutoLang 程序，不依赖 Rust 编译器。

## 现状对比（2026-05-09）

### 代码量

| 组件 | Auto 版 | Rust 版 | 比例 |
|------|---------|---------|------|
| Parser | ~1,370 行 | 12,255 行 | 11% |
| Type Inference | 227 行 | 5,251 行 | 4% |
| VM/Engine | 608 行 | 5,372 行 | 11% |
| Codegen | 704 行 | 8,863 行 | 8% |
| a2r Transpiler | ~750 行 | 5,662 行 | 13% |
| Generic Registry | 0 行 | 1,680 行 | 0% |
| FFI | — | 4,717 行 | — |
| **总计** | **~5,700 行** | **45,365 行** | **13%** |

### 功能差距

| 维度 | Rust 版 | AAVM 现状 | 差距 |
|------|---------|----------|------|
| 类型系统 | Type enum (30+ 变体) | 5 种 (int/str/bool/void/unknown) | 基础可用 |
| 表达式 | Expr enum (40+ 变体) | 39 NodeKind | 大部分对齐 |
| Codegen | 131 OpCode | ~30 OpCode | 23%，缺浮点/Option/Result/闭包/Task |
| 泛型 | GenericRegistry 1,680 行 | D1 已做 parser+a2r, 无 generics.at | 解析✅ 单态化❌ |
| 类型推断 | unification + constraints | 简化版类型传播 | 缺泛型约束 |
| a2r 转译器 | 5,662 行，79 测试 | ~750 行，25 bootstrap 测试 | 核心 60-70% |

### a2r 转译器特性覆盖

**已覆盖**: 基础表达式, 函数, 变量, if/else, for, struct, enum, impl, trait, match, F-string, 闭包, 数组, 对象, 错误传播, struct 构造函数, 多语句 match, use.c/py FFI, 泛型类型映射 (`List<int>` → `Vec<i32>`, `Map<str, int>` → `HashMap<String, i32>`), 泛型 enum/spec/ext 输出

**未覆盖**:

| 特性 | 难度 | 优先级 |
|------|------|--------|
| Option/Result 匹配 (`Some(x) -> ...`) | 中 | P0 |
| 泛型单态化 (generics.at) | 高 | P1 |
| 借用语义 (`.view`/`.mut`/`.take`) | 中 | P1 |
| 泛型 type alias | 中 | P2 |
| 原始指针 (`*T`) | 低 | P2 |

### 关键缺失能力

| 缺失 | 影响 | 优先级 |
|------|------|--------|
| generics.at 不存在 | 无法做 `List<T>` 运行时单态化 | P1 |
| 类型推断无 unification | 不支持泛型约束 | P2 |
| 无 Task/Channel 字节码 | 不支持并发编程 | P2 |
| 闭包字节码未实现 | codegen 不生成闭包字节码 | P1 |
| OpCode 覆盖 23% | 缺浮点/Option/Result/闭包/并发 | P2 |

### 已知 VM Bug

- **`.find()` 返回值在算术中编码错误**: `s.find("<")` 可正确用于 `int_to_str` 打印，但参与 `+ 1` 等算术时结果错误。Workaround: 使用 `char_at` 遍历替代。

## 文件结构

```
auto/lib/
├── ast.at          # ✅ 345 行 — 39 NodeKind + 构造函数
├── parser.at       # ✅ ~1,370 行 — P0+P1+泛型参数解析
├── lexer.at        # ✅ 598 行 — P0 Lexer
├── token.at        # ✅ 306 行 — 129 种 TokenKind
├── pos.at          # ✅ 8 行 — Pos 位置类型
├── error.at        # ✅ 8 行 — Error 类型
├── eval.at         # ✅ 703 行 — tree-walking evaluator
├── typeinfer.at    # ✅ 227 行 — 简化版类型推断
├── codegen.at      # ✅ 704 行 — 字节码生成器
├── vm.at           # ✅ 608 行 — 字节码解释器
├── opcode.at       # ✅ 78 行 — OpCode 常量
├── a2r.at          # ✅ ~750 行 — Auto-to-Rust 转译器 (E1-E6+D1)
└── generics.at     # ❌ 待新建 (~300 行) — 泛型单态化注册表 (D2)
```

## 值表示架构

AAVM 在 Rust VM 上运行，使用 non-nanbox i32 编码：

| 值类型 | i32 编码 | 示例 |
|--------|---------|------|
| `int 42` | 直接存值 (>= 0) | `42` |
| `str "hello"` | `-(pool_idx + 1)` | `-3` |
| `bool true` | `i32::MIN` = `-2147483648` | 哨兵值 |
| `bool false` | `i32::MIN + 1` = `-2147483647` | 哨兵值 |
| `nil` | `-2147483647` | 同 false |
| 堆对象 | `>= 4000000` | 堆对象 ID |

eval_expr 保持返回 int，通过辅助函数 (`val_is_str`, `val_is_bool`, `val_is_heap_obj`) 区分类型。

## Phase D2 待做: generics.at 泛型注册表

**目标**: 记录泛型类型定义，在使用点做类型字符串替换。

**改动文件**: `auto/lib/generics.at`（新建）

**范围**:
1. `generic_register(reg, name, params)` — 注册泛型定义
2. `generic_lookup(reg, name)` — 查询泛型信息
3. `generic_instantiate(reg, name, args)` — 实例化：返回替换后的类型字符串

**注意**: 这是最小实现，仅做类型字符串替换，不涉及 codegen 字节码单态化。

**测试**: `107_generic_registry`

## Phase E5 待做: Option/Result 匹配

**目标**: `is opt { Some(x) -> ... None -> ... }` → `match opt { Some(x) => ... None => ... }`

**改动文件**: `auto/lib/a2r.at`

**Rust 对标**: `trans/rust.rs` 的 `CREATE_SOME`, `CREATE_NONE`, `IS_SOME`, `UNWRAP_SOME` 等

**难度**: 中 — 需要在 a2r_is 中添加 Some/None/Ok/Err 特殊处理

## 里程碑

| 里程碑 | 完成标志 | 状态 |
|--------|---------|------|
| M1: 值多态 | eval_expr 正确返回 int/str/bool/list | ✅ |
| M2: 类型感知 | 编译器知道每个表达式的类型 | ✅ |
| M3: 字节码执行 | AAVM 能编译+运行简单程序 | ✅ |
| M4: 泛型支持 | List<T> 正确实例化和操作 | 🔄 D1 完成 (parser+a2r), D2 待做 |
| M5: 自举能力 | AAVM 能转译自身为 Rust 代码 | 🔄 95 测试通过, 核心特性 60-70% |
