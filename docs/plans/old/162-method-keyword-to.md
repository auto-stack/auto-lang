# Plan 162: `.to(Type)` 方法关键字 — 显式类型转换

> **日期**: 2026-04-12
> **状态**: 待实现
> **前置**: Plan 161 `.as(Type)` 已完成

## 1. 目标

为 Auto 添加 `.to(Type)` 方法关键字，提供显式类型转换能力。与 `.as(Type)` 互补：

| 操作 | 语义 | 开销 | 类比 |
|------|------|------|------|
| `x.as(u32)` | reinterpret，位模式不变 | 零开销 | Rust `as`, C cast |
| `x.to(str)` | convert，可能分配/计算 | 有开销 | D `.to!string`, Rust `.to_string()` |

## 2. 设计决策

### 2.1 方法关键字概念

`.to()` 是**方法关键字**（method keyword），不是普通方法。在 `x.yyy` 或 `x.yyy()` 的环境下，方法关键字优先于同名方法查找——这与普通语句中关键字优先于变量名的规则一致。

```
15.to(str)        → 方法关键字 .to，编译期类型转换
15.to_string()    → 普通方法调用，参数是值不是类型
15.to(str, 16)    → 未来扩展：带参数的转换（如进制）
```

**不冲突的原因**：parser 能区分两种模式：
- `.to(` + **类型名** + `)` → 方法关键字
- `.to_string(` + **值参数** + `)` → 普通方法调用

### 2.2 与 `.as()` 的关系

两者复用相同的 parser 拦截模式（Pratt parser + dot_item peek），但语义不同：

```
x.as(u32)   → Rust: (x as u32)           // 零开销 reinterpret
x.to(str)   → Rust: x.to_string()         // 分配 String
x.to(int)   → Rust: x.parse::<i32>().unwrap()  // 解析字符串
```

### 2.3 AST 设计

新增独立变体（不复用 `Cast`）：

```rust
/// expr.to(Type) — 显式类型转换（有开销）
To { expr: Box<Expr>, target_type: Type },
```

与 `Cast` 区分：
- `Cast` = `.as()` = 零开销 reinterpret
- `To` = `.to()` = 有开销 convert

## 3. 转译映射

### 3.1 a2r（Auto → Rust）

| Auto | Rust | 说明 |
|------|------|------|
| `x.to(str)` | `x.to_string()` | 任意类型 → String |
| `x.to(int)` | `x.parse::<i32>().unwrap()` | str → int（运行时解析） |
| `x.to(float)` | `x.parse::<f64>().unwrap()` | str → float |
| `x.to(u32)` | `x as u32` | 数值 → 数值（其实和 .as 一样） |
| `x.to(i64)` | `x as i64` | 数值 → 数值 |
| `x.to(String)` | `String::from(x)` | 显式 String 类型 |
| `x.to(bool)` | 根据源类型处理 | int→bool: `x != 0`, str→bool: `parse` |

**简化规则**：
- 目标类型是 `str`/`String` → `.to_string()`
- 目标类型是数值类型，源是 `str` → `.parse::<T>().unwrap()`
- 目标类型是数值类型，源是数值 → `as T`（退化为 .as）
- 其他 → `T::from(x)` 或编译错误

### 3.2 a2c（Auto → C）

| Auto | C | 说明 |
|------|---|------|
| `x.to(str)` | 需要运行时支持 | sprintf / snprintf |
| `x.to(int)` | `atoi(x)` | str → int |
| `x.to(float)` | `atof(x)` | str → float |
| `x.to(u32)` | `(unsigned int)(x)` | 数值 → 数值 |

### 3.3 VM

| 目标类型 | 操作码 | 实现 |
|----------|--------|------|
| str | TYPE_TO_STR | value → AutoStr |
| int | TYPE_TO_I32 | value → i32 |
| float | TYPE_TO_F64 | value → f64 |
| u32 | TYPE_TO_U32 | value → u32 |

## 4. 实施步骤

### Step 1: Token — 新增 `TokenKind::To`

**文件**: `crates/auto-lang/src/token.rs`

1. 在 `TokenKind` 枚举中添加 `To`
2. 在 `keyword_kind()` 中添加 `"to" => Some(TokenKind::To)`
3. 在 `Display for Token` 中添加显示格式

**注意**: `to` 成为关键字后，不能再用作变量名。需评估是否影响现有代码。

### Step 2: AST — 新增 `Expr::To` 变体

**文件**: `crates/auto-lang/src/ast.rs`

1. 在 `Expr::Cast` 后添加：
   ```rust
   To { expr: Box<Expr>, target_type: Type },  // expr.to(Type) — explicit conversion
   ```
2. 更新 `Display for Expr`
3. 更新 `ToNode for Expr`
4. 更新所有 match Expr 的完整匹配（编译器会报错提示）

### Step 3: Parser — 拦截 `.to(Type)`

**文件**: `crates/auto-lang/src/parser.rs`

复用 `.as(Type)` 的两处拦截模式：

**3a. Pratt parser**（约 line 1802，`.as()` 拦截后面）：
```rust
// 在 .as(Type) 拦截之后添加
if matches!(op, Op::Dot) && self.is_kind(TokenKind::To) {
    self.next(); // consume 'to'
    self.expect(TokenKind::LParen)?;
    let target_type = self.parse_type()?;
    self.expect(TokenKind::RParen)?;
    lhs = Expr::To { expr: Box::new(lhs), target_type };
    continue;
}
```

**3b. `dot_item()` 函数**（约 line 1548，`.as()` peek 后面）：
```rust
// 在 next_is_as 检查之后添加 next_is_to 检查
let next_is_to = if let Ok(tok) = self.lexer.next() {
    let is_to = matches!(tok.kind, TokenKind::To);
    self.lexer.push_token(tok);
    is_to
} else { false };
if next_is_as || next_is_to { break; }
```

### Step 4: a2r 转译

**文件**: `crates/auto-lang/src/trans/rust.rs`

在 `expr()` 方法的 `Expr::Cast` 后添加：

```rust
Expr::To { expr, target_type } => {
    match target_type {
        Type::Str(_) | Type::String => {
            // x.to(str) → x.to_string()
            self.expr(expr, out)?;
            write!(out, ".to_string()")?;
        }
        Type::Int => {
            // 如果源是 str，生成 parse；否则退化为 as
            self.expr(expr, out)?;
            write!(out, ".parse::<i32>().unwrap()")?;
        }
        Type::Float | Type::Double => {
            self.expr(expr, out)?;
            write!(out, ".parse::<f64>().unwrap()")?;
        }
        _ => {
            // 退化为 as（数值之间）
            write!(out, "(")?;
            self.expr(expr, out)?;
            write!(out, " as {})", self.rust_type_name(target_type))?;
        }
    }
    Ok(())
}
```

**优化方向**（后续迭代）：根据源表达式类型决定生成 `parse` 还是 `as`。初期用简单规则。

### Step 5: a2c 转译

**文件**: `crates/auto-lang/src/trans/c.rs`

```rust
Expr::To { expr, target_type } => {
    match target_type {
        Type::Str(_) | Type::String => {
            // x.to(str) → 需要辅助函数
            write!(out, "auto_to_str(")?;
            self.expr(expr, out)?;
            write!(out, ")")?;
        }
        Type::Int => {
            write!(out, "atoi(")?;
            self.expr(expr, out)?;
            write!(out, ")")?;
        }
        _ => {
            // 退化为 C cast
            write!(out, "(({})", self.c_type_name(target_type))?;
            self.expr(expr, out)?;
            write!(out, ")")?;
        }
    }
    Ok(())
}
```

### Step 6: VM 支持

**文件**: `crates/auto-lang/src/vm/opcode.rs`

新增转换操作码：
```rust
TYPE_TO_STR = 0xEC,
TYPE_TO_I32 = 0xED,
TYPE_TO_F64 = 0xEE,
```

**文件**: `crates/auto-lang/src/vm/codegen.rs`

`Expr::To` → 根据 target_type 发射对应 opcode

**文件**: `crates/auto-lang/src/vm/engine.rs`

执行转换：Value 的类型标签转换（str→int 用 parse，int→str 用 format 等）

### Step 7: 其他受影响文件

| 文件 | 修改 |
|------|------|
| `crates/auto-lang/src/dep.rs` | `Expr::To` 遍历依赖 |
| `crates/auto-lang/src/infer/expr.rs` | `Expr::To` → 返回 target_type |

### Step 8: 测试用例

新增 a2r 测试：

**`test/a2r/139_to_convert/to_convert.at`**：
```auto
// Plan 162: .to(Type) explicit conversion

fn main() {
    let x int = 42

    // int → str
    let s str = x.to(str)

    // str → int
    let n int = "123".to(int)

    // int → float
    let f float = x.to(float)

    // int → u32 (退化, 等价 .as)
    let u u32 = x.to(u32)

    print(s)
    print(n)
}
```

**`test/a2r/139_to_convert/to_convert.expected.rs`**：
```rust
// Auto-generated by a2r transpiler

// a2r Standard Library (from crate)
#[allow(unused_imports)]
use auto_lang::a2r_std::*;

fn main() {
    let x: i32 = 42;
    let s: String = x.to_string();
    let n: i32 = "123".parse::<i32>().unwrap();
    let f: f64 = (x as f64);
    let u: u32 = (x as u32);
    println!("{}", s);
    println!("{}", n);
}
```

新增 a2c 测试：

**`test/a2c/153_to_convert/to_convert.at`** — 验证 C 侧转换

## 5. 实施顺序

```
Step 1 (token) → Step 2 (AST) → Step 3 (parser) → Step 4 (a2r) → Step 5 (a2c) → Step 6 (VM) → Step 7 (其他) → Step 8 (测试)
```

预计影响文件：8-10 个源文件 + 2 个测试目录

## 6. 验证标准

- [ ] `cargo build -p auto-lang` 编译通过
- [ ] `x.to(str)` 正确转译为 `x.to_string()`
- [ ] `x.to(int)` 正确转译为 `x.parse::<i32>().unwrap()`
- [ ] `x.to(u32)` 退化为 `(x as u32)`
- [ ] `.to()` 不与 `.to_string()` 等方法冲突
- [ ] 40 个现有 transpiler 测试无回归
- [ ] 新增 139、153 测试通过

## 7. 未来扩展

- `x.to(str, 16)` → 带参数转换（如进制转换：`format!("{:x}", x)`）
- `x.to(bytes)` → 字节数组转换
- 编译期源类型推断：如果编译器知道源是 str，用 `parse`；如果源是 int，用 `as`
- 自定义 `To` spec：用户可为自定义类型定义 `.to()` 行为
