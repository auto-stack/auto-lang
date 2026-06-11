# Plan 290: Auto → GDScript (a2gd) Transpiler

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现 AutoLang 到 GDScript 2.0 (Godot 4.x) 的转译器 (a2gd)，让 Auto 脚本能够生成有效的 GDScript 代码并挂载到 Godot 节点上运行。

**Architecture:** 基于 a2py (Python 转译器) 的架构模式，创建 `GDScriptTrans` 结构体实现 `Trans` trait。采用分阶段增量实现：Phase 1 实现核心转译（函数、变量、控制流），Phase 2 添加 Godot 特有特性（信号、export、生命周期），Phase 3 实现类型映射和高级特性。与 a2py 共享相似但不相同的代码路径——暂不抽取公共模块，待两个转译器都稳定后再重构共享代码。

**Tech Stack:** Rust, AutoLang AST, GDScript 2.0 (Godot 4.x)

---

## 一、GDScript 与 Python 关键差异对照表

这是实现 a2gd 的核心参考。GDScript 语法上类似 Python，但有以下关键差异：

| 特性 | Python | GDScript | a2gd 转译策略 |
|------|--------|----------|---------------|
| 函数定义 | `def foo():` | `func foo():` | 替换关键字 |
| 布尔值 | `True`/`False` | `true`/`false` | 替换为小写 |
| 空值 | `None` | `null` | 替换关键字 |
| 字符串类型 | `str` | `String` | 类型名大写 |
| 变量声明 | `x = 10` | `var x = 10` | 添加 `var` 关键字 |
| 常量声明 | `X = 10` | `const X = 10` | 添加 `const` 关键字 |
| 缩进 | 4 空格 | Tab (`\t`) | 使用 `\t` 缩进 |
| F-string | `f"hello {x}"` | 无（用 `%s` % x） | 转换为 `%` 格式化 |
| 类声明 | `class Foo:` | `class_name Foo` | 不同的声明语法 |
| 继承 | `class Foo(Bar):` | `extends Bar` | 不同的继承语法 |
| 枚举 | `class E(Enum):` | `enum { A, B, C }` | 完全不同的语法 |
| 逻辑运算 | `and`/`or`/`not` | `and`/`or`/`not`（也支持 `&&`/`||`/`!`） | 保持 `and`/`or` |
| 除法 | `10/3` → `3.333` | `10/3` → `3` (整数除法) | 注意行为差异 |
| 数组长度 | `len(arr)` | `arr.size()` | 方法调用替代函数调用 |
| 数组删除 | `arr.remove(val)` | `arr.erase(val)` | 方法名不同 |
| Lambda | `lambda x: x+1` | `func (x): return x+1` | 语法不同，且调用需 `.call()` |
| match | `match/case` (3.10+) | `match x:` / `pattern:` | `case` 关键字不同 |
| 导入 | `import/from` | `preload/load` | 完全不同 |
| 入口保护 | `if __name__ == "__main__":` | 无需（`_ready()` 自动调用） | 去掉 main guard |
| 类型推断 | 无 | `var x := 10` | 可选使用 `:=` |
| 信号 | N/A | `signal my_signal(arg)` | GDScript 特有 |
| @export | N/A | `@export var speed: float = 200.0` | GDScript 特有 |
| @onready | N/A | `@onready var sprite = $Sprite2D` | GDScript 特有 |
| 节点引用 | N/A | `$NodeName` 或 `%UniqueName` | GDScript 特有 |

## 二、Auto → GDScript 类型映射表

| Auto 类型 | GDScript 类型 | 备注 |
|-----------|---------------|------|
| `int` | `int` | 直接映射 |
| `uint` | `int` | GDScript 无无符号整数 |
| `i64`/`u64` | `int` | GDScript int 为 64 位 |
| `byte` | `int` | GDScript 无 byte 类型 |
| `float` | `float` | 直接映射 |
| `double` | `float` | GDScript float 为 64 位 |
| `bool` | `bool` | 直接映射 |
| `str` | `String` | 注意大写 |
| `void` | `void` | 直接映射 |
| `List<T>` | `Array` 或 `Array[T]` | GDScript 支持类型化数组 |
| `Map<K,V>` | `Dictionary` | 直接映射 |
| `Option<T>` | `Variant` (可为 null) | GDScript Variant 天然支持 null |
| `Result<T>` | 抛出异常或返回 Variant | 需要特殊处理 |
| 用户类型 | 同名类型 | 直接映射 |
| 枚举 | `enum` | GDScript 原生枚举 |

## 三、Auto → GDScript 关键字映射

| Auto 关键字/语法 | GDScript 输出 | 说明 |
|------------------|---------------|------|
| `fn foo()` | `func foo():` | 函数定义 |
| `let x = 10` | `var x = 10` | 不可变变量（GDScript 无不可变概念） |
| `var x = 10` | `var x = 10` | 可变变量 |
| `const X = 10` | `const X = 10` | 常量 |
| `if cond { ... }` | `if cond:\n\t...` | 条件语句 |
| `elif cond { ... }` | `elif cond:\n\t...` | else if |
| `else { ... }` | `else:\n\t...` | else |
| `for x in 0..10 { ... }` | `for x in range(0, 10):\n\t...` | 范围循环 |
| `for cond { ... }` | `while cond:\n\t...` | 条件循环 |
| `loop { ... }` | `while true:\n\t...` | 无限循环 |
| `is x { ... }` | `match x:\n\t...` | 模式匹配 |
| `Some(x)` | `x` | GDScript 无 Option 包装 |
| `None` | `null` | 空值 |
| `Ok(x)` | `x` | GDScript 无 Result 包装 |
| `print(...)` | `print(...)` | 直接映射 |
| `self.x` | `self.x` | 直接映射 |

## 四、实现计划

### Task 1: 创建 GDScriptTrans 基础框架

**文件**: `crates/auto-lang/src/trans/gdscript.rs`

创建 `GDScriptTrans` 结构体，实现 `Trans` trait 的骨架：

```rust
pub struct GDScriptTrans {
    indent: usize,
    name: AutoStr,
}

impl GDScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self { indent: 0, name }
    }
}

impl Trans for GDScriptTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // 1. 文件头注释
        // 2. 遍历 AST 生成代码
        // 3. 输出到 sink
    }
}
```

**要点**:
- 注册模块到 `trans.rs`：添加 `pub mod gdscript;`
- 缩进使用 `\t`（Tab 字符），不用 4 空格
- 文件头添加 `# Auto-generated from {name}.at — do not edit\n\n`

### Task 2: 表达式转译 (`expr` 方法)

实现核心的 `expr` 方法，处理所有 `Expr` 枚举变体：

**关键映射**:
- `Expr::Bool(b)` → `"true"` / `"false"` （注意小写，与 Python 的 `True`/`False` 不同）
- `Expr::None` / `Expr::Nil` / `Expr::Null` → `"null"` （不是 Python 的 `"None"`）
- `Expr::Str(s)` → `"\"{s}\""`
- `Expr::FStr(fstr)` → 转换为 GDScript 的 `%` 格式化（见下文）
- `Expr::Some(e)` → 直接输出表达式（GDScript 无 Option 包装）
- `Expr::Ok(e)` → 直接输出表达式
- `Expr::Err(e)` → 输出为错误字符串或注释标注
- `Expr::NullCoalesce(lhs, rhs)` → `lhs if lhs != null else rhs`
- `Expr::Closure(closure)` → `func ({params}): {body}` (GDScript lambda)
- `Expr::Await { expr }` → `await {expr}`
- `Expr::Go { expr }` → 直接输出（GDScript 无并发原语）
- `Expr::Bina` → 逻辑运算使用 `and`/`or`（GDScript 同时支持 `&&`/`||`）

**F-string 转换**:
```
Auto:  f"Hello $name, age ${age + 1}"
GDScript: "Hello %s, age %s" % [name, str(age + 1)]
```
实现方式：收集所有表达式部分，用 `%s` 占位，最后用 `% [expr1, expr2, ...]` 格式化。单参数时用 `% expr` 不需要数组。

### Task 3: 语句转译 (`stmt` 方法)

实现 `stmt` 方法处理所有 `Stmt` 枚举变体：

**Store 处理**:
- Auto 的 `let x = 10` → `var x = 10`
- Auto 的 `var x = 10` → `var x = 10`
- 赋值 `x = 10` → `x = 10`
- 需要区分首次声明和重新赋值（通过检查 `store` 的属性）

**Fn 处理**:
- `fn foo(a int, b int) int { ... }` → `func foo(a: int, b: int) -> int:\n\t...`
- 参数类型注解使用 `: Type` 语法
- 返回类型使用 `-> Type` 语法
- `async` / `~T` 返回类型 → GDScript 的 `async`（使用 `await`）
- 实例方法自动添加 `self` 参数（与 Python 不同，GDScript 不需要显式 self 参数）

**If 处理**:
- 与 a2py 基本相同，但缩进用 Tab

**For 处理**:
- `for x in 0..10 { ... }` → `for x in range(0, 10):\n\t...`
- `for x in 0..=10 { ... }` → `for x in range(0, 10 + 1):\n\t...`
- `for cond { ... }` → `while cond:\n\t...`
- `for x in arr { ... }` → `for x in arr:\n\t...`
- `for (i, x) in arr { ... }` → 暂不处理（GDScript 无 enumerate，需手动计数）

**Is 处理 (模式匹配)**:
- Auto: `is x { 0 -> ..., _ -> ... }`
- GDScript: `match x:\n\t0:\n\t\t...\n\t_:\n\t\t...`
- 注意 GDScript 的 match 不使用 `case` 关键字，直接写模式值后跟冒号

### Task 4: 类型声明转译

**Struct → Class**:
```
Auto:
  type Player {
    name str
    health int
  }

GDScript:
  class_name Player
  var name: String
  var health: int
```

- 有方法时生成带 `func _init()` 的完整类
- 无方法时只生成成员变量

**Enum**:
```
Auto:
  enum Color { Red, Green, Blue }

GDScript:
  enum Color { RED, GREEN, BLUE }
```
- 注意 GDScript enum 习惯用全大写

**Tag/Union**:
- 暂时映射为带 `kind` 字段的 class + 工厂方法（与 a2py 策略类似）

### Task 5: GDScript 特有功能

**信号支持**:
- 在 Auto 中通过注解或特定语法声明信号
- 生成 `signal health_changed(old_value, new_value)`
- 信号声明必须放在文件顶部（类成员区域）

**生命周期函数映射**:
- Auto 的 `fn _ready()` → GDScript 的 `func _ready():`
- Auto 的 `fn _process(delta float)` → GDScript 的 `func _process(delta: float):`
- Auto 的 `fn main()` → 映射为 `_ready()` 或保留为普通函数

**节点路径引用**:
- `$NodeName` 语法在 GDScript 原生支持，直接透传
- `get_node("Path")` 同样直接透传

**注释格式**:
- Auto 的 `//` 注释 → GDScript 的 `#` 注释

### Task 6: Trans trait 主流程 (`trans` 方法)

实现完整的 `trans` 方法：

1. **文件头**：`# Auto-generated from {name}.at — do not edit`
2. **extends 声明**：默认 `extends Node`（或根据 Auto 源文件的注解确定）
3. **信号声明**：收集所有信号，放在 extends 之后
4. **导出变量**：收集 @export 变量，放在信号之后
5. **类型声明**：class 定义
6. **函数声明**：所有函数（包括生命周期函数）
7. **不需要 main guard**：GDScript 通过 `_ready()` 入口，不需要 `if __name__` 保护

**与 a2py 的关键区别**:
- 不生成 `from typing import ...`
- 不生成 `from dataclasses import dataclass`
- 不生成 `if __name__ == "__main__":`
- 不生成 `from enum import Enum, auto`
- 缩进用 Tab 而非 4 空格

### Task 7: 测试框架搭建

**测试目录**: `crates/auto-lang/test/a2gd/`

**测试文件结构**:
```
test/a2gd/
├── 000_hello/
│   ├── hello.at
│   └── hello.expected.gd
├── 001_var/
│   ├── var.at
│   └── var.expected.gd
├── 002_func/
│   ├── func.at
│   └── func.expected.gd
├── 010_if/
│   ├── if.at
│   └── if.expected.gd
├── 011_for/
│   ├── for.at
│   └── for.expected.gd
├── 012_match/
│   ├── match.at
│   └── match.expected.gd
├── 013_struct/
│   ├── struct.at
│   └── struct.expected.gd
├── 014_enum/
│   ├── enum.at
│   └── enum.expected.gd
└── 015_string/
    ├── string.at
    └── string.expected.gd
```

**测试函数** (在 `gdscript.rs` 底部 `#[cfg(test)] mod tests` 中):
```rust
fn test_a2gd(case: &str) -> AutoResult<()> {
    let last_segment = case.rsplit('/').next().unwrap_or(case);
    let parts: Vec<&str> = last_segment.split("_").collect();
    let name = parts[1..].join("_");

    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let src_path = format!("test/a2gd/{}/{}.at", case, name);
    let src = read_to_string(d.join(&src_path))?;

    let _scope = crate::scope_manager::ScopeManager::new();
    let mut parser = Parser::from(src.as_str());
    let ast = parser.parse()?;
    let mut sink = Sink::new(name.into());
    let mut trans = GDScriptTrans::new(name.into());
    trans.trans(ast, &mut sink)?;
    let gd_code = sink.done()?;

    let expected_path = format!("test/a2gd/{}/{}.expected.gd", case, name);
    let expected = read_to_string(d.join(&expected_path))?;

    if gd_code != expected.as_bytes() {
        let gen_path = format!("test/a2gd/{}/{}.wrong.gd", case, name);
        std::fs::write(d.join(&gen_path), gd_code)?;
    }

    assert_eq!(String::from_utf8_lossy(gd_code), expected);
    Ok(())
}
```

### Task 8: lib.rs 和 CLI 集成

**lib.rs 添加**:
```rust
pub fn trans_gdscript(path: &str) -> AutoResult<String> {
    let code = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    let gdname = path.replace(".at", ".gd");
    let fname = AutoPath::new(path).filename();

    let _scope = Rc::new(RefCell::new(crate::scope_manager::ScopeManager::new()));
    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = Sink::new(fname.clone());
    let mut trans = crate::trans::gdscript::GDScriptTrans::new(fname);
    trans.trans(ast, &mut sink)?;

    std::fs::write(&gdname, sink.done()?)?;
    Ok(format!("[trans] {} -> {}", path, gdname))
}
```

**CLI main.rs 添加**:
- 在 `TransTarget` enum 中添加 `Gdscript { output: Option<String> }`
- 在 `Commands::Trans` 匹配中添加 `TransTarget::Gdscript` 分支
- 也可添加独立的 `GDScript { path: String }` 命令

### Task 9: MVP 测试用例

创建首批测试用例，确保核心转译流程跑通：

**000_hello** — 最简单的 hello world:
```auto
// Auto input
fn main() {
    print("Hello, Godot!")
}
```
```gdscript
# Expected GDScript output
# Auto-generated from hello.at — do not edit

func _ready():
	print("Hello, Godot!")
```

**001_var** — 变量声明:
```auto
fn main() {
    let x = 10
    var y = 20
    print(x + y)
}
```

**002_func** — 函数定义和调用:
```auto
fn add(a int, b int) int {
    a + b
}

fn main() {
    print(add(3, 4))
}
```

**010_if** — 条件语句:
```auto
fn main() {
    let x = 10
    if x > 5 {
        print("big")
    } else if x > 3 {
        print("medium")
    } else {
        print("small")
    }
}
```

**011_for** — 循环:
```auto
fn main() {
    for i in 0..5 {
        print(i)
    }
}
```

**012_match** — 模式匹配:
```auto
fn main() {
    let x = 3
    is x {
        0 -> print("zero")
        1 -> print("one")
        _ -> print("other")
    }
}
```

**013_struct** — 类型声明:
```auto
type Player {
    name str
    health int
}
```

**014_enum** — 枚举:
```auto
enum Color { Red, Green, Blue }
```

**015_string** — 字符串和 f-string:
```auto
fn main() {
    let name = "World"
    print(f"Hello $name!")
}
```

### Task 10: 类型映射完善 (`gdscript_type_name` 方法)

实现 `gdscript_type_name` 方法，将 Auto 类型映射为 GDScript 类型名：

```rust
fn gdscript_type_name(&self, ty: &Type) -> AutoStr {
    match ty {
        Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::Byte => "int".into(),
        Type::Float | Type::Double => "float".into(),
        Type::Bool => "bool".into(),
        Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => "String".into(),
        Type::Void => "void".into(),
        Type::User(type_decl) => type_decl.name.clone(),
        Type::Enum(enum_decl) => enum_decl.borrow().name.clone(),
        Type::List(_) => "Array".into(),        // 或 "Variant" 作 fallback
        Type::Map(_, _) => "Dictionary".into(),
        Type::Option(_) => "Variant".into(),      // GDScript Variant 天然可空
        Type::Result(_) => "Variant".into(),      // 暂用 Variant
        _ => "Variant".into(),                    // Fallback
    }
}
```

---

## 五、文件清单

### 新建文件
| 文件路径 | 用途 |
|----------|------|
| `crates/auto-lang/src/trans/gdscript.rs` | GDScript 转译器主文件 |
| `crates/auto-lang/test/a2gd/` | 测试目录 |
| `crates/auto-lang/test/a2gd/000_hello/hello.at` | hello world 测试输入 |
| `crates/auto-lang/test/a2gd/000_hello/hello.expected.gd` | hello world 期望输出 |
| `crates/auto-lang/test/a2gd/001_var/var.at` | 变量测试 |
| `crates/auto-lang/test/a2gd/001_var/var.expected.gd` | 变量期望输出 |
| `crates/auto-lang/test/a2gd/002_func/func.at` | 函数测试 |
| `crates/auto-lang/test/a2gd/002_func/func.expected.gd` | 函数期望输出 |
| `crates/auto-lang/test/a2gd/010_if/if.at` | 条件语句测试 |
| `crates/auto-lang/test/a2gd/010_if/if.expected.gd` | 条件语句期望输出 |
| `crates/auto-lang/test/a2gd/011_for/for.at` | 循环测试 |
| `crates/auto-lang/test/a2gd/011_for/for.expected.gd` | 循环期望输出 |
| `crates/auto-lang/test/a2gd/012_match/match.at` | 模式匹配测试 |
| `crates/auto-lang/test/a2gd/012_match/match.expected.gd` | 模式匹配期望输出 |
| `crates/auto-lang/test/a2gd/013_struct/struct.at` | 类型声明测试 |
| `crates/auto-lang/test/a2gd/013_struct/struct.expected.gd` | 类型声明期望输出 |
| `crates/auto-lang/test/a2gd/014_enum/enum.at` | 枚举测试 |
| `crates/auto-lang/test/a2gd/014_enum/enum.expected.gd` | 枚举期望输出 |
| `crates/auto-lang/test/a2gd/015_string/string.at` | 字符串测试 |
| `crates/auto-lang/test/a2gd/015_string/string.expected.gd` | 字符串期望输出 |

### 修改文件
| 文件路径 | 修改内容 |
|----------|----------|
| `crates/auto-lang/src/trans.rs` | 添加 `pub mod gdscript;` |
| `crates/auto-lang/src/lib.rs` | 添加 `trans_gdscript()` 函数 |
| `crates/auto/src/main.rs` | 添加 `Gdscript` TransTarget 和 CLI 命令 |

## 六、实现顺序与依赖关系

```
Task 1 (框架骨架)
  ↓
Task 2 (表达式) ← 核心依赖
  ↓
Task 3 (语句) ← 依赖 Task 2
  ↓
Task 10 (类型映射) ← 依赖 Task 3
  ↓
Task 4 (类型声明) ← 依赖 Task 10
  ↓
Task 5 (GDScript 特有) ← 依赖 Task 4
  ↓
Task 6 (Trans 主流程) ← 依赖所有前述
  ↓
Task 7 (测试框架) ← 依赖 Task 6
  ↓
Task 9 (MVP 测试用例) ← 依赖 Task 7
  ↓
Task 8 (CLI 集成) ← 可与 Task 9 并行
```

## 七、风险与注意事项

1. **Tab 缩进**：GDScript 强制使用 Tab 缩进。所有生成的代码必须使用 `\t`，不能用空格。Rust 的 `print_indent` 方法中必须用 `\t`。

2. **F-string 转换**：GDScript 不支持 f-string。Auto 的 `f"Hello $name"` 需要转换为 `"Hello %s" % name`。多参数时用 `"Hello %s, age %s" % [name, str(age)]`。注意需要用 `str()` 包裹非字符串表达式。

3. **整数除法**：GDScript 中 `10 / 3 = 3`（整数除法），而 Auto 中 `10 / 3` 可能期望浮点结果。这需要在生成代码时根据操作数类型判断是否需要转换。MVP 阶段暂不处理，记录为已知差异。

4. **信号声明位置**：GDScript 要求 `signal` 声明必须在文件顶层（类成员区域），不能在函数内部。需要收集所有信号声明并提升到顶部。

5. **main 函数映射**：Auto 的 `main()` 函数需要映射为 GDScript 的 `_ready()` 函数，或者映射为一个普通函数由 `_ready()` 调用。MVP 阶段采用后者。

6. **不生成 main guard**：GDScript 不需要 Python 的 `if __name__ == "__main__":` 保护。入口是 `_ready()` 生命周期函数。

7. **与 a2py 的关系**：虽然 GDScript 和 Python 语法相似，但由于上述关键差异（关键字、布尔值、null、f-string、缩进方式、main guard 等），直接复用 a2py 代码会导致大量条件分支。建议独立实现，但在代码结构和模式上保持一致，便于未来抽取共享代码。
