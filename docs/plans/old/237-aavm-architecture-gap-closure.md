# Plan 237: AAVM Architecture Gap Closure — 分阶段拉近与 Rust AutoVM 的距离

## 状态: Phase A-E6 + D1 + D2 + E5 已完成, 108 个 bootstrap 测试 (2026-05-29)

### 已完成

| Phase | 测试编号 | 内容 | 代码量 |
|-------|---------|------|--------|
| A 值多态 | 048-052 | eval_expr 支持 int/str/bool/nil/heap | eval.at 702 行 |
| B 类型推断 | 053-057 | 字面量→变量→函数签名类型传播 | typeinfer.at 226 行 |
| C 字节码 | 058-074 | codegen + vm + BVM str/map/list (30 OpCode) | codegen.at 703 行 + vm.at 607 行 |
| D 基础 | 075-080 | 补充测试 | — |
| E1-E4 a2r | 081-099 | 基础表达式→struct/enum/impl/trait/closure/array/object | a2r.at 769 行 |
| E5 构造函数 | 102 | `Point(1,2)` → `Point { x: 1, y: 2 }` | +struct_fields Map |
| E6 高级特性 | 100-101, 103 | 多语句 match arm, use.c/py FFI, 泛型类型映射 | — |
| D1 泛型解析 | 104-106 | Parser `<T>` 读取 + a2r 泛型转译 + enum/spec/ext 泛型输出 | +parser_try_read_generic_type, a2r_type char_at 解析 |
| D2 泛型注册表 | 112 | generics.at — 注册/查询/实例化类型字符串替换 | generics.at 302 行 |
| E5 Option/Result | 107 | parser SomeKW/NoneKW/OkKW/ErrKW + a2r CallExpr 输出 | parser.at +a2r.at |
| E5 借用语义 | 108 | lexer DotView/DotMut/DotMove/DotTake + parser + a2r | lexer.at +parser.at +ast.at +a2r.at |

**总计**: 13 个 Auto 源文件, ~6,120 行, 108 个 bootstrap 测试

### 不做

| 编号 | 内容 | 原因 |
|------|------|------|
| E6 Grid/矩阵表达式 | 低优先级，需要重新设计 | 暂不实施 |

## 目标

让 AAVM（用 Auto 写的 Auto 编译器）逐步具备与 Rust 版 Auto 编译器相同的架构能力。最终目标是 AAVM 能独立编译和执行 AutoLang 程序，不依赖 Rust 编译器。

## 现状对比（2026-05-29）

### 代码量

| 组件 | Auto 版 | Rust 版 | 比例 |
|------|---------|---------|------|
| Parser | 1,430 行 | 12,255 行 | 12% |
| Type Inference | 226 行 | 5,251 行 | 4% |
| VM/Engine | 607 行 | 5,372 行 | 11% |
| Codegen | 703 行 | 8,863 行 | 8% |
| a2r Transpiler | 769 行 | 5,662 行 | 14% |
| Generic Registry | 302 行 | 1,680 行 | 18% |
| Lexer | 616 行 | — | — |
| AST | 370 行 | — | — |
| Token | 305 行 | — | — |
| Eval | 702 行 | — | — |
| Opcode | 76 行 | — | — |
| **总计** | **~6,120 行** | **45,365 行** | **13%** |

### 功能差距

| 维度 | Rust 版 | AAVM 现状 | 差距 |
|------|---------|----------|------|
| 类型系统 | Type enum (30+ 变体) | 5 种 (int/str/bool/void/unknown) | 基础可用 |
| 表达式 | Expr enum (40+ 变体) | 39 NodeKind | 大部分对齐 |
| Codegen | 131 OpCode | ~30 OpCode | 23%，缺浮点/Option/Result/闭包/Task |
| 泛型 | GenericRegistry 1,680 行 | D1 parser+a2r ✅ D2 注册表 ✅ | 字符串替换可用 |
| 类型推断 | unification + constraints | 简化版类型传播 | 缺泛型约束 |
| a2r 转译器 | 5,662 行，271 测试 | 769 行，32 bootstrap 测试 | 核心 60-70% |

### a2r 转译器特性覆盖

**已覆盖**: 基础表达式, 函数, 变量, if/else, for, struct, enum, impl, trait, match, F-string, 闭包, 数组, 对象, 错误传播, struct 构造函数, 多语句 match, use.c/py FFI, 泛型类型映射, 泛型 enum/spec/ext 输出, Option/Result 匹配, 借用语义 (.view→&, .mut→&mut, .move/.take→passthrough)

**未覆盖**:

| 特性 | 难度 | 优先级 |
|------|------|--------|
| 泛型 type alias | 中 | P2 |
| 原始指针 (`*T`) | 低 | P2 |
| Grid/矩阵表达式 | 低 | P3 |

### 关键缺失能力

| 缺失 | 影响 | 优先级 |
|------|------|--------|
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
└── generics.at     # ✅ 302 行 — 泛型注册表 (D2)
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

## Phase D2 ✅ 已完成: generics.at 泛型注册表 (2026-05-29)

**目标**: 记录泛型类型定义，在使用点做类型字符串替换。
**参考**: Rust 版 `generic_registry.rs` (1687 行) 的最小子集

### AAVM 约束
- 数据结构：只有 `List<T>`、`Map<str, str>` 和原始类型（int/str/bool）
- 值编码：int 直接存值，str 用 string pool

### 改动文件

1. **`auto/lib/generics.at`**（新建，~300 行）
   - `GenericEntry` type: name, params("K,V"), fields("key:K,val:V")
   - `GenericRegistry` type: entries List<GenericEntry>
   - `generic_new()` — 创建空注册表
   - `generic_register(reg, name, params, fields)` — 注册泛型定义
   - `generic_lookup(reg, name)` — 按名称查询泛型
   - `generic_instantiate(reg, name, type_args)` — 实例化：类型字符串替换
   - `str_replace_all(s, old, new)` — 字符串替换辅助

2. **`test/vm/99_bootstrap/112_a2r_generic_registry/`** — 测试用例
   - 注册 `Pair<K,V>` → 实例化 `Pair<int,str>` → 验证字段替换
   - 注册 `Container<T>` → 实例化 `Container<bool>` → 验证
   - 查询不存在的类型

3. **`crates/auto-lang/src/tests/vm_file_tests.rs`** — 注册测试函数

### generic_instantiate 核心逻辑
- 解析 type_args 如 `"int,str"` → 按 `,` 分割
- 解析注册的 params 如 `"K,V"` → 按 `,` 分割
- 在 fields 中做字符串替换：`K→i32`, `V→String`
- 调用 `a2r_type()` 对替换后的类型做最终映射

### 验证
```bash
cargo test -p auto-lang --lib -- test_aavm_99_bootstrap_112
```

## 里程碑

| 里程碑 | 完成标志 | 状态 |
|--------|---------|------|
| M1: 值多态 | eval_expr 正确返回 int/str/bool/list | ✅ |
| M2: 类型感知 | 编译器知道每个表达式的类型 | ✅ |
| M3: 字节码执行 | AAVM 能编译+运行简单程序 | ✅ |
| M4: 泛型支持 | List<T> 正确实例化和操作 | ✅ D1+D2 完成 |
| M5: 自举能力 | AAVM 能转译自身为 Rust 代码 | 🔄 108 测试通过, 核心特性 60-70% |

## AAVM ↔ Rust 同步机制（差异快照法）

### 原则

AAVM 是 Rust 编译器的**有意子集**，不是镜像。同步不是"全部追平"，而是"按需拉取有价值的新特性"。

### 格式：每个 .at 文件头部的快照注释

```auto
// ╔═══════════════════════════════════════════════════════════╗
// ║ AAVM Sync Snapshot                                       ║
// ║ Component: vm.at (Bytecode Interpreter)                  ║
// ║ Rust ref:  crates/auto-lang/src/vm/engine.rs, opcode.rs  ║
// ║ Baseline:  fe666a6f (2026-05-26)                         ║
// ║ Coverage:  35/~100 opcodes (35%)                         ║
// ║ Missing:   float, async, closures, generics, concurrency ║
// ╚═══════════════════════════════════════════════════════════╝
```

字段说明：
- **Rust ref**: 对应的 Rust 源文件路径
- **Baseline**: 最后一次同步时的 Rust commit hash
- **Coverage**: 已实现/总量
- **Missing**: 未覆盖的主要特性列表（关键词级别，不需详尽）

### 同步流程

1. **定期检查**（如每月或大版本发布前）：
   ```bash
   git log --oneline <baseline>..HEAD -- <rust-ref-paths>
   ```
2. **审查变更**：阅读 commit 历史，筛选出 AAVM 需要跟进的特性
3. **选择性同步**：只拉取与 AAVM 目标相关的改动（排除 UI/debugger/FFI 等 AAVM 暂不需要的部分）
4. **更新文件头**：更新 Baseline commit 和 Coverage/Missing
5. **补充测试**：为新增特性添加 bootstrap 测试

### 初始基准（2026-05-29）

| AAVM 文件 | Rust 对照 | Baseline |
|-----------|----------|----------|
| vm.at | src/vm/engine.rs, opcode.rs | fe666a6f |
| codegen.at | src/trans/rust.rs (部分) | 7cc484b1 |
| parser.at | src/parser.rs | ddbb161a |
| eval.at | src/eval.rs, src/interpreter/ | fe666a6f |
| a2r.at | src/trans/rust.rs | 7cc484b1 |
| lexer.at | src/lexer.rs | fe666a6f |
| typeinfer.at | src/type_inference/ | fe666a6f |
| generics.at | src/vm/generic_registry.rs | fe666a6f |

### 不在同步范围内的 Rust 特性

以下 Rust 特性是 AAVM 设计上暂不追踪的（除非未来需求变更）：
- UI 系统 (iced/gpui)
- C FFI / Python FFI
- 调试器/性能分析器
- 数据库驱动的编译缓存
- 多文件项目支持
- Task/Channel 并发模型
