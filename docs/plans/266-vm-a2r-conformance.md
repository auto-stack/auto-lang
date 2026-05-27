# Plan 266: AutoVM ↔ a2r Semantic Conformance

**Status**: Phase 1 Complete
**Created**: 2026-05-25
**Related**: [Plan 265 (AutoVM MCP Server)](265-autovm-mcp-server.md)
**Scope**: `docs/spec/`, `crates/auto-lang/test/a2r/conformance/`, `crates/auto-lang/src/trans/rust.rs`

## Problem Statement

Auto 的核心卖点之一是"用 AutoVM 快速迭代，用 a2r 编译发布 Rust"。这要求 **AutoVM 和 a2r 对同一份 Auto 代码产生完全一致的运行结果**。

当前保证方式是手写测试用例（相同代码分别在 AutoVM 和 a2r 运行，比较输出），存在两个问题：
1. **覆盖率不可控** — 新特性可能遗漏对偶测试
2. **没有系统性规范** — AutoVM 和 a2r 各自独立实现，没有共同的"标准答案"

此外，如果在脚本模式中引入动态类型（省略类型注解），将产生一个额外的语义间隙需要验证，大幅增加 conformance 的复杂度。

## Decision: Static Types Only

**Auto 始终使用静态类型。不支持动态类型。**

AI 不需要省略类型来提速（AI 的瓶颈是验证循环速度，不是打字速度）。显式类型反而让 VM 给出更精确的诊断，加速验证循环。

```
AI 生成: fn add(a int, b int) int { a + b }     ← 总是带类型
VM 执行: 直接解释运行，无需编译                     ← 周转率同样快
a2r 编译: 类型已在代码中，直接转译                  ← 无翻译间隙
```

**类型推断的角色**：作为诊断辅助，不是行为特性。

- AI 可以省略类型 → VM 推断并返回警告（不是报错）
- `auto_typecheck`（Plan 265 工具）告诉 AI 推断结果
- AI 根据警告补上类型 → a2r 可直接编译
- 推断类型和显式类型的行为必须完全一致

这样做消除了"动态→静态翻译"这个 conformance 难题。

## Architecture: Three-Layer Conformance

```
Layer 3: 差分测试 (Differential Testing)
         随机生成程序 → AutoVM 和 a2r 都跑 → 比较输出
         覆盖长尾 edge case

Layer 2: 对偶执行测试 (Dual-Execution Tests)
         每个 .at 测试用例 → AutoVM 执行 + a2r 编译执行 → 输出必须一致
         特性级覆盖率 100%

Layer 1: 语义规范 (Semantic Specification)
         每个操作的精确定义 → AutoVM 和 a2r 的共同标准
         单一真相源 (Single Source of Truth)
```

## Layer 1: Semantic Specification

### Directory Structure

```
docs/spec/
├── README.md                    # 规范索引和阅读指南
├── 01-arithmetic.md             # 算术运算语义
├── 02-strings.md                # 字符串操作语义
├── 03-collections.md            # 数组/List/Map 操作语义
├── 04-control-flow.md           # 控制流语义 (if/for/loop/is)
├── 05-functions.md              # 函数调用、闭包、递归
├── 06-types.md                  # 类型系统 (struct/enum/type)
├── 07-pattern-matching.md       # is 表达式语义
├── 08-methods.md                # 方法调用、dot notation
├── 09-error-handling.md         # 错误处理 (Option/Result/panic)
└── 10-builtins.md               # 内置函数语义 (print/len/str/...)
```

### Specification Format

每个操作用统一格式定义：

```markdown
## 运算符 `+`

### `int + int` → `int`
- **语义**: 数学加法
- **溢出**: wrap around（与 Rust `i32::wrapping_add` 一致）
- **AutoVM 实现**: `opcode::ADD` → 检查两操作数为 int → wrapping_add
- **a2r 实现**: 直接转译为 `a + b`（Rust debug 模式会 panic，需统一为 wrapping）
- **示例**: `1 + 2` → `3`; `2147483647 + 1` → `-2147483648`

### `str + str` → `str`
- **语义**: 字符串拼接，返回新字符串
- **AutoVM 实现**: 检查两操作数为 str → 拼接 heap strings
- **a2r 实现**: 转译为 `format!("{}{}", a, b)`
- **示例**: `"a" + "b"` → `"ab"`

### `int + str` → 编译错误 E001
- **语义**: 类型不匹配
- **诊断**: "cannot concatenate int and str"
- **建议**: `str(x)` 或 `f"$x..."`
```

### Priority

Phase 1 先覆盖最常用的 20 个操作（算术、比较、字符串、数组索引、print），其余随语言特性扩展逐步补充。

## Layer 2: Dual-Execution Test Framework

### Test Format

每个对偶测试包含：
```
test/a2r/conformance/
├── 001_int_add/
│   ├── input.at              # Auto 源码
│   └── expected_output.txt   # 预期的 stdout 输出
├── 002_str_concat/
│   ├── input.at
│   └── expected_output.txt
└── ...
```

`input.at` 必须包含 `print()` 语句来产生可观测输出。

### Test Runner

```rust
// 在 crates/auto-lang/tests/conformance.rs 或集成到现有测试框架

fn run_dual_test(name: &str) {
    let dir = format!("test/a2r/conformance/{}", name);
    let code = fs::read_to_string(format!("{}/input.at", dir));
    let expected = fs::read_to_string(format!("{}/expected_output.txt", dir));

    // Path 1: AutoVM 执行
    let vm_output = run_autovm_capture(&code)
        .expect("AutoVM should not crash")
        .1;  // captured stdout

    // Path 2: a2r 编译 + 执行
    let rust_code = transpile_a2r(&code);
    let rust_output = compile_and_run_rust(&rust_code, /* temp dir */);

    // 三方比较
    assert_eq!(vm_output.trim(), expected.trim(),
        "AutoVM output doesn't match expected");
    assert_eq!(rust_output.trim(), expected.trim(),
        "a2r output doesn't match expected");
    // 以上两个 assert 已经隐含 vm_output == rust_output
}
```

### Feature Coverage Matrix

```markdown
| 特性                    | 规范 | 对偶测试 | AutoVM 单测 | a2r 单测 |
|-------------------------|------|---------|-------------|---------|
| int 算术 (+,-,*,/)      | ✅   | ✅      | ✅          | ✅      |
| int 比较 (<,>,<=,>=)    | ✅   | ✅      | ✅          | ✅      |
| str 拼接                | ✅   | ✅      | ✅          | ✅      |
| str 索引                | ✅   | ❌      | ✅          | ✅      |
| 数组创建和索引           | ✅   | ✅      | ✅          | ✅      |
| 数组切片                | ✅   | ❌      | ✅          | ❌      |
| List 动态操作            | ✅   | ❌      | ✅          | ❌      |
| for range loop          | ✅   | ✅      | ✅          | ✅      |
| if/else 表达式          | ✅   | ✅      | ✅          | ✅      |
| enum 定义和匹配          | ✅   | ❌      | ✅          | ❌      |
| type (struct) 定义       | ✅   | ❌      | ✅          | ✅      |
| 方法调用                 | ✅   | ❌      | ✅          | ❌      |
| f-string                | ✅   | ✅      | ✅          | ✅      |
| 闭包/lambda             | ❌   | ❌      | ✅          | ❌      |
| panic/error handling    | ❌   | ❌      | ❌          | ❌      |
```

目标：对偶测试列逐步填满 ✅。新增语言特性必须同时补规范和对偶测试。

### 与现有 a2r 测试的关系

现有 a2r 测试（`test/a2r/NNN_name/`）验证的是"转译出的 Rust 代码语法正确"，格式为 `input.at → expected.rs`。

对偶测试验证的是"运行结果一致"，格式为 `input.at → expected_output.txt`。两者互补，不冲突。

建议：
- 现有 a2r 测试保留不动
- 对偶测试放在 `test/a2r/conformance/` 子目录
- 可以从现有 a2r 测试中选取部分（那些包含 print 输出的），补充 `expected_output.txt` 升级为对偶测试

## Layer 3: Differential Testing (Program Generator)

### Design

自动生成类型正确的随机 Auto 程序，分别在 AutoVM 和 a2r 执行，比较输出。

```
┌─────────────────┐
│ Program Generator│
│                 │
│ 1. 生成类型声明  │
│ 2. 生成函数      │
│ 3. 生成 main    │
│    表达式        │
└────────┬────────┘
         │ Auto 代码
         ├──────────► AutoVM ──────► output_vm
         │
         └──────────► a2r + rustc ─► output_rust

                                         │
                              output_vm == output_rust ?
                              ├─ YES → 下一个测试
                              └─ NO  → minimize → 保存回归用例
```

### Generator Strategy

```rust
struct ProgramGenerator {
    rng: StdRng,
    types: Vec<TypeDef>,      // 已生成的类型
    functions: Vec<FuncDef>,  // 已生成的函数
    depth_limit: u32,         // 表达式递归深度限制
}

impl ProgramGenerator {
    fn generate_program(&mut self) -> String {
        // 1. 可选：生成 0-2 个 type 定义
        // 2. 生成 1-5 个函数
        //    - 参数从已知类型中随机选取
        //    - 函数体由 generate_expr(type, depth) 递归生成
        // 3. 生成 main 代码，调用函数并 print 结果
    }

    fn generate_expr(&mut self, target_type: &Type, depth: u32) -> String {
        if depth == 0 { return self.generate_literal(target_type); }

        match target_type {
            Type::Int => self.pick_one_of([
                format!("{} + {}", self.gen_expr(Int, depth-1), self.gen_expr(Int, depth-1)),
                format!("{} * {}", self.gen_expr(Int, depth-1), self.gen_expr(Int, depth-1)),
                self.gen_function_call(Int),  // 调用已知返回 int 的函数
                self.gen_literal(Int),
            ]),
            Type::Str => self.pick_one_of([
                format!("f\"${}\"", self.gen_expr(Int, depth-1)),
                format!("{} + {}", self.gen_expr(Str, depth-1), self.gen_expr(Str, depth-1)),
            ]),
            Type::Bool => self.pick_one_of([
                format!("{} < {}", self.gen_expr(Int, depth-1), self.gen_expr(Int, depth-1)),
                format!("{} == {}", self.gen_expr(Int, depth-1), self.gen_expr(Int, depth-1)),
            ]),
            // ... 其他类型
        }
    }
}
```

关键约束：**生成的程序必须类型正确**，否则两边都会报错（不是 conformance 问题）。我们测的是"相同语义的两个实现是否一致"。

### Minimizer

发现不一致时，自动缩小到最小复现用例：

```rust
fn minimize(original: &str, oracle: fn(&str) -> bool) -> String {
    let mut minimal = original.to_string();
    // 策略 1: 删除不影响不一致性的语句
    // 策略 2: 替换子表达式为字面量
    // 策略 3: 简化控制流
    minimal
}
```

最小用例自动保存到 `test/a2r/conformance/regression/NNN_name/`。

### Integration with CI

```yaml
# 每次 PR 跑 1000 个随机程序的差分测试
# 发现不一致则 CI 失败，自动生成回归用例
cargo test -p auto-lang -- conformance_differential --count 1000
```

## Implementation Phases

### Phase 1: Semantic Specification Skeleton (1-2 days) — ✅ COMPLETE

**Goal**: 前 20 个最常用操作有精确语义定义。

**Tasks**:
1. ✅ 创建 `docs/spec/` 目录和 `README.md` 索引
2. ✅ 编写 `01-arithmetic.md` — int/float 算术、溢出行为
3. ✅ 编写 `02-strings.md` — 拼接、索引、f-string
4. ✅ 编写 `03-collections.md` — 数组创建、索引、切片
5. ✅ 编写 `04-control-flow.md` — if/for/loop 基本语义
6. ✅ 编写 `10-builtins.md` — print/len/str/类型转换

**Deliverable**: 6 个规范文件，覆盖核心运算

### Phase 2: Dual-Execution Test Framework (2-3 days)

**Goal**: 对偶测试基础设施就位，迁移现有测试。

**Tasks**:
1. 创建 `test/a2r/conformance/` 目录结构
2. 实现 `run_dual_test()` 测试框架函数
   - AutoVM 执行（复用 `run_autovm_capture`）
   - a2r 编译 + rustc 编译 + 执行（需要临时目录）
   - 输出比较和错误报告
3. 从现有 a2r 测试中选取 10-15 个有 print 输出的用例，补充 `expected_output.txt`
4. 实现特性覆盖率矩阵追踪（脚本或 CI 报告）
5. 在 CI 中加入对偶测试步骤

**Deliverable**: 对偶测试框架 + 15+ 初始测试用例 + CI 集成

**Files created**:
- `test/a2r/conformance/` (测试用例目录)
- `tests/conformance.rs` 或集成到 `tests/trans.rs`

**Files modified**:
- CI 配置

### Phase 3: Conformance Test Expansion (3-5 days)

**Goal**: 对偶测试覆盖所有已支持的语言特性。

**Tasks**:
1. 为每个已有 a2r 测试用例评估是否可升级为对偶测试
2. 为缺失的特性编写新的对偶测试用例：
   - enum 定义和 pattern matching
   - type (struct) 创建和方法调用
   - 数组切片
   - List 动态操作（如果 a2r 已支持）
   - 嵌套函数调用
   - 递归
3. 更新特性覆盖率矩阵至 ≥ 80%
4. 每发现不一致，修复后补充语义规范

**Deliverable**: 50+ 对偶测试用例，覆盖率 ≥ 80%

### Phase 4: Differential Testing Engine (3-5 days)

**Goal**: 自动化长尾覆盖。

**Tasks**:
1. 实现 `ProgramGenerator` — 类型正确的随机 Auto 程序生成
   - 支持的类型：int, str, bool, []T, struct
   - 支持的操作：算术、比较、字符串拼接、数组索引、函数调用
   - 递归深度限制（max 3 层）
2. 实现 `Minimizer` — 不一致用例自动缩小
3. 集成到测试框架：
   - `cargo test -p auto-lang -- conformance_differential`
   - 支持 `--seed` 参数复现特定测试
   - 支持 `--count` 参数控制生成数量
4. 回归用例自动保存机制
5. CI 配置：每次 PR 跑 500 个随机程序

**Deliverable**: 差分测试引擎 + CI 集成

**Files created**:
- `crates/auto-lang/src/test_util/program_generator.rs`
- `crates/auto-lang/src/test_util/minimizer.rs`

### Phase 5: Spec-Driven Development (ongoing)

**Goal**: 语义规范成为新特性的开发流程的一部分。

**Tasks**:
1. 新特性开发流程变为：
   - 先写语义规范（`docs/spec/`）
   - 再写对偶测试用例
   - 然后实现 AutoVM 支持
   - 最后实现 a2r 支持
   - 对偶测试自动验证两者一致
2. 逐步完善已有特性的语义规范
3. 定期运行差分测试，积累回归测试集

## Success Metrics

1. **Layer 1**: 核心操作（算术、比较、字符串、数组、控制流）100% 有语义规范
2. **Layer 2**: 语言特性对偶测试覆盖率 ≥ 90%
3. **Layer 3**: 差分测试引擎每次 CI 跑 500+ 随机程序，0 不一致
4. **回归测试集**: 累计 20+ 历史不一致的回归用例
5. **新特性流程**: 每个新 a2r 特性都有对应的规范和对偶测试

## Risks and Mitigations

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| 语义规范与实际实现不一致 | 规范失去权威性 | 规范由对偶测试验证；发现不一致时以测试结果为准更新规范 |
| 差分测试生成器产生无效程序 | 浪费 CI 时间 | 生成器保证类型正确；无效程序跳过不计 |
| a2r 尚不支持某些特性 | 对偶测试无法运行 | 对偶测试标记为 `#[ignore]`，跟踪在特性矩阵中 |
| Minimizer 无法缩小某些用例 | 回归测试包含冗余代码 | 手动审查；设置代码行数上限 |

## Relationship to Plan 265

- **Plan 265** 的 `auto_typecheck` 工具是这个计划中"始终静态类型"策略的 AI 接口
- **Plan 265** 的 `auto_evaluate` 可以被对偶测试框架复用（AutoVM 端执行）
- **Plan 265** 的 `auto_snapshot` 导出的 `.at` 文件可以直接作为对偶测试的 `input.at`
- 两个计划的 Phase 1 可以并行推进，Phase 2 开始有依赖（MCP 工具需要 conformance 保证）
