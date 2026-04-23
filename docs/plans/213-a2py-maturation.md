# Plan 213: a2py Python 转译器成熟化

## Status: 🔧 PARTIAL (base transpiler exists, maturation ongoing)

Verified 2026-04-23:
- ✅ PythonTrans in `trans/python.rs` with indent management and import tracking
- ✅ Test suite in `test/a2p/` with basic test cases (hello, array, struct, enum, method, if, for, is, str)
- ❌ Option/Result support not yet done
- ❌ Closure/Lambda support not yet done
- ❌ Test count at ~9, target is 80+

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 将 a2py（Auto→Python 转译器）从 18% a2r 功能覆盖提升到 80%+，补齐 Option/Result、闭包、async/await、泛型、specs、元组、对象字面量、错误传播等关键特性，测试数从 10 个增长到 80+ 个。

**Architecture:** 逐功能模块扩展 `PythonTrans`，每个功能模块对应一组 a2r 已有测试的 Python 等价测试。遵循现有 a2py 模式：`PythonTrans` 实现 `Trans` trait，遍历 AST 生成 Python 代码。

**Tech Stack:** Rust, `PythonTrans`, `Parser`, `Sink`, 标准 `#[test]` 框架

---

## 当前状态

| 指标 | a2py | a2r | 比例 |
|------|------|-----|------|
| 代码行数 | 841 | 4,642 | 1:5.5 |
| 测试分类 | 10 | 19 | 1:1.9 |
| 测试文件 | 10 | 141 | 1:14.1 |

### 已有 a2py 功能

- 基本类型（int, float, bool, str, char）
- 变量赋值（let/var → 直接赋值）
- 函数声明与调用
- if/elif/else
- for 循环（范围）
- 结构体（→ @dataclass）
- 枚举（→ class Enum）
- 方法（→ class methods）
- 模式匹配 is（→ match/case）
- F-string（`f"hello $name"` → `f"hello {name}"`）
- 数组（→ list）
- break

### 缺失功能（按优先级）

| 功能 | a2r 分类 | Python 映射策略 | 优先级 |
|------|---------|----------------|--------|
| Option/Result | 09_option_result | `Some(x)` → `x`, `None` → `None`, `Ok/Err` → 值/异常 | P0 |
| 闭包/lambda | 05_expressions | `(x) => x + 1` → `lambda x: x + 1` | P0 |
| 元组 | 02_types | `(a, b)` → `(a, b)` 直接映射 | P1 |
| 对象字面量 | 02_types | `{key: val}` → `{"key": val}` | P1 |
| for cond {} (while) | 03_control_flow | `for cond {}` → `while cond:` | P1 |
| continue | 03_control_flow | `continue` → `continue` | P1 |
| return | 03_control_flow | `return expr` → `return expr` | P1 |
| null 合并 `??` | 05_expressions | `x ?? default` → `x if x is not None else default` | P1 |
| 错误传播 `?.` | 09_option_result | `x?.method()` → `x.method() if x is not None else None` | P1 |
| 类型注解 | — | 参数和返回值加 Python type hints | P1 |
| 泛型 | 08_generics | 类型擦除或 TypeVar | P2 |
| async/await | — | `~{ }` → `async def`, `.await` → `await` | P2 |
| specs/traits | 12_specs | `spec` → `Protocol` 或 ABC | P2 |
| union/tag | 02_types | dataclass + discriminated field | P2 |
| import/use | 14_modules | `use` → `from ... import ...` | P2 |
| 显式所有权 | 07_ownership | 忽略（Python 无所有权语义） | 跳过 |
| 指针/Box/Arc | 02_types | 忽略（Python 无对应概念） | 跳过 |

### 测试目录和注册

- **测试目录：** `crates/auto-lang/test/a2p/{category}/{number}_{name}/`
- **文件：** `{name}.at` + `{name}.expected.py`
- **测试注册：** 当前在 `python.rs` 的 `#[cfg(test)] mod tests` 中
- **Runner：** `fn test_a2p(case: &str)` — 解析、转译、与 `.expected.py` 比较

---

## Task 1: 补充 P0 — Option/Result 支持

**文件：**
- 修改：`crates/auto-lang/src/trans/python.rs` — 添加 Option/Result 表达式处理
- 创建：`test/a2p/` 下新增测试目录

**Python 映射策略：**

| Auto 构造 | Python 输出 |
|-----------|------------|
| `Some(x)` | `x` |
| `None` | `None` |
| `Ok(x)` | `x` |
| `Err(msg)` | `raise Exception(msg)` |
| `is x { Some(v) -> ... None -> ... }` | `if x is not None: v = x; ... else: ...` |
| `x ?? default` | `x if x is not None else default` |
| `x?.method()` | `x.method() if x is not None else None` |

**Step 1: 在 python.rs 中处理 Option/Result 构造器**

在 `compile_expr()` 中添加：

```rust
// Some(x) → x
Expr::Call(name, args) if name == "Some" => {
    // Just output the inner value
    self.compile_expr(&args[0], sink)?;
}

// None → None
Expr::Ident(name) if name == "None" => {
    sink.print(b"None")?;
}

// Ok(x) → x
Expr::Call(name, args) if name == "Ok" => {
    self.compile_expr(&args[0], sink)?;
}

// Err(msg) → raise Exception(msg)
Expr::Call(name, args) if name == "Err" => {
    sink.print(b"raise Exception(")?;
    self.compile_expr(&args[0], sink)?;
    sink.print(b")")?;
}
```

**Step 2: 处理 is 模式中的 Option 匹配**

```auto
is opt {
    Some(val) -> print(val)
    None -> print("none")
}
```
→
```python
if opt is not None:
    val = opt
    print(val)
else:
    print("none")
```

**Step 3: 创建测试**

```
test/a2p/020_option/option.at:
fn main() {
    let x = Some(42)
    let y = None
    if x is not None {
        print(x)
    }
    let val = x ?? 0
    print(val)
}
```
```
test/a2p/020_option/option.expected.py:
def main():
    x = 42
    y = None
    if x is not None:
        print(x)
    val = x if x is not None else 0
    print(val)
```

**Step 4: 注册测试，运行，提交**

```bash
cargo test -p auto-lang --lib -- test_a2p_020_option --nocapture
git commit -m "feat(a2py): add Option/None support with null coalescing"
```

---

## Task 2: 补充 P0 — 闭包/Lambda 支持

**文件：**
- 修改：`crates/auto-lang/src/trans/python.rs`

**Python 映射策略：**

| Auto 构造 | Python 输出 |
|-----------|------------|
| `(x) => x + 1` | `lambda x: x + 1` |
| `(a, b) => a + b` | `lambda a, b: a + b` |
| `(x) { ...多行... }` | `def _lambda(x): ...` + 引用 |

**Step 1: 添加 Lambda 处理**

```rust
Expr::Lambda(params, body) => {
    sink.print(b"lambda ")?;
    // params
    for (i, param) in params.iter().enumerate() {
        if i > 0 { sink.print(b", ")?; }
        sink.print(param.name.as_bytes())?;
    }
    sink.print(b": ")?;
    // body (single expression)
    self.compile_expr(body, sink)?;
}
```

**Step 2: 创建测试**

```
test/a2p/021_lambda/lambda.at:
fn main() {
    let add = (a int, b int) int => a + b
    print(add(3, 4))
    let nums = [1, 2, 3]
    let doubled = nums.map(x => x * 2)
    print(doubled)
}
```
```
test/a2p/021_lambda/lambda.expected.py:
def main():
    add = lambda a, b: a + b
    print(add(3, 4))
    nums = [1, 2, 3]
    doubled = list(map(lambda x: x * 2, nums))
    print(doubled)
```

**Step 3: 注册测试，运行，提交**

```bash
git commit -m "feat(a2py): add lambda/closure support"
```

---

## Task 3: 补充 P1 — 元组、对象字面量、continue、return

### 3a: 元组

```auto
let pair = (1, "hello")
print(pair.0)
```
→
```python
pair = (1, "hello")
print(pair[0])
```

### 3b: 对象字面量

```auto
let obj = {"name": "auto", "ver": 1}
print(obj["name"])
```
→
```python
obj = {"name": "auto", "ver": 1}
print(obj["name"])
```

### 3c: continue + return

直接映射，无需特殊处理。

### 3d: for cond {} → while

```auto
var x = 0
for x < 10 {
    x = x + 1
    if x == 5 {
        continue
    }
}
```
→
```python
x = 0
while x < 10:
    x = x + 1
    if x == 5:
        continue
```

**创建 4 个测试，注册，运行，提交：**

```bash
git commit -m "feat(a2py): add tuple, object literal, continue, return, while-loop support"
```

---

## Task 4: 补充 P1 — 类型注解

**目标：** 生成的 Python 代码带有 type hints，提高可读性和工具支持。

```auto
fn add(a int, b int) int {
    a + b
}
```
→
```python
def add(a: int, b: int) -> int:
    return a + b
```

**类型映射表：**

| Auto 类型 | Python type hint |
|-----------|-----------------|
| `int` | `int` |
| `float` / `double` | `float` |
| `str` | `str` |
| `bool` | `bool` |
| `[T]` | `list[T]` |
| `{K: V}` | `dict[K, V]` |
| `T?` / `Option<T>` | `Optional[T]` |
| `Result<T, E>` | `T` (with raises comment) |

**创建 2 个测试，提交：**

```bash
git commit -m "feat(a2py): add Python type annotations to function signatures"
```

---

## Task 5: 补充 P1 — 错误传播 `?.`

**映射策略：**

```auto
let result = parse_int("42")?.abs()
```
→
```python
_result = parse_int("42")
result = _result.abs() if _result is not None else None
```

对于链式 `?.`：

```auto
let val = obj?.method()?.data
```
→
```python
val = (obj.method() if obj is not None else None).data if (obj.method() if obj is not None else None) is not None else None
```

链式场景下 Python 代码可读性差，但功能正确。后续可优化为 temp 变量模式。

**创建测试，提交：**

```bash
git commit -m "feat(a2py): add error propagation (?.) support"
```

---

## Task 6: 补充 P2 — 泛型（类型擦除）

**策略：** Python 不支持 Rust 风格的泛型。对 a2py 来说，泛型参数在转译时直接擦除：

```auto
fn first<T>(list [T]) T {
    list[0]
}
```
→
```python
def first(list):
    return list[0]
```

如果需要 type hints，可以用 `TypeVar`：

```python
from typing import TypeVar
T = TypeVar('T')
def first(list: list[T]) -> T:
    return list[0]
```

本 Task 只做简单擦除，不做 TypeVar（后者可作为增强）。

**创建 3 个测试，提交：**

```bash
git commit -m "feat(a2py): add generic function support (type erasure)"
```

---

## Task 7: 补充 P2 — async/await

**映射策略：**

```auto
fn fetch(url str) ~str {
    // async body
}
```
→
```python
async def fetch(url: str) -> str:
    # async body
```

```auto
let data = fetch(url).await
```
→
```python
data = await fetch(url)
```

- `~T` 返回类型 → `async def` + `-> T`
- `.await` → `await`
- `~{ ... }` async block → 需要特殊处理（提取为临时 async 函数）

**创建 2 个测试，提交：**

```bash
git commit -m "feat(a2py): add async/await support"
```

---

## Task 8: 补充 P2 — Specs → Protocol

**映射策略：**

```auto
spec Comparable {
    fn compare(self, other Self) int
}
```
→
```python
from typing import Protocol

class Comparable(Protocol):
    def compare(self, other: 'Comparable') -> int: ...
```

**创建 2 个测试，提交：**

```bash
git commit -m "feat(a2py): add spec/trait → Protocol support"
```

---

## Task 9: 补充 P2 — Union/Tag

**映射策略：**

```auto
union Shape {
    Circle(radius float)
    Rect(w float, h float)
}
```
→
```python
from dataclasses import dataclass

@dataclass
class Shape:
    kind: str
    radius: float = 0.0
    w: float = 0.0
    h: float = 0.0

    @staticmethod
    def Circle(radius):
        return Shape('Circle', radius=radius)

    @staticmethod
    def Rect(w, h):
        return Shape('Rect', w=w, h=h)
```

**创建 2 个测试，提交：**

```bash
git commit -m "feat(a2py): add union/tag → dataclass support"
```

---

## Task 10: 批量新增测试达到 80+ 覆盖

按 a2r 的测试分类，逐类对齐：

| a2r 分类 | 测试数 | 对应 a2py 新增测试 |
|----------|-------|-------------------|
| 01_basics | 13 | +8（注释、一元运算、多表达式等） |
| 02_types | 10 | +5（tuple, object, union） |
| 03_control_flow | 8 | +3（while, continue, nested） |
| 04_strings | 5 | +3（string methods, f-string edge cases） |
| 05_expressions | 10 | +5（lambda, null coalesce, ternary） |
| 06_pattern_matching | 6 | +3（struct destructuring, option pattern） |
| 08_generics | 3 | +2（generic functions） |
| 09_option_result | 33 | +8（Option/Result basics, propagate） |
| 10_collections | 4 | +3（list, dict comprehensions） |
| 11_methods | 5 | +3（static, mut self） |
| 12_specs | 3 | +2（Protocol） |

目标：从 10 个测试增长到 **~80 个测试**。

**批量创建测试，提交：**

```bash
git commit -m "test(a2py): expand test coverage to 80+ cases matching a2r categories"
```

---

## 总结：各 Task 产出

| Task | 功能 | 新增测试 | 累计测试 |
|------|------|---------|---------|
| 当前 | 基础功能 | 10 | 10 |
| 1 | Option/Result | +5 | 15 |
| 2 | Lambda/闭包 | +3 | 18 |
| 3 | 元组/对象/while/continue/return | +6 | 24 |
| 4 | 类型注解 | +2 | 26 |
| 5 | 错误传播 `?.` | +4 | 30 |
| 6 | 泛型 | +3 | 33 |
| 7 | async/await | +3 | 36 |
| 8 | Specs/Protocol | +2 | 38 |
| 9 | Union/Tag | +3 | 41 |
| 10 | 批量对齐 a2r 测试 | +40 | ~81 |

## 不在范围内

以下功能 Python 无对应概念，转译时静默忽略：
- 所有权/借用（`view`, `mut`, `take`）
- 指针类型（`*T`, `Box<T>`, `Arc<T>`）
- 模块系统（`use.c`, `dep`）
- FFI（`extern "c"`, `#[vm]`）
