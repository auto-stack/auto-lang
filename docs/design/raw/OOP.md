# OOP 设计

## 核心原则

AutoLang 是**面向对象的语言**，设计上类似于 Java：

1. **类型和方法绑定**：方法定义在类型内部（type 里面）
2. **静态方法**：使用 `static fn` 定义在类型内部
3. **实例方法**：使用 `fn` 定义在类型内部
4. **泛型支持**：类型和方法都支持泛型参数 `<T>`
5. **模块化**：通过类型和模块组织代码，不需要手动添加前缀

## 类型定义

### 基本类型定义

```auto
tag Point<T> {
    x T
    y T

    // 静态方法在类型内部
    static fn new() Point<T> { ... }

    // 实例方法在类型内部
    fn is_empty() bool { ... }
}
```

**关键点**：
- ✅ 方法定义在 type 内部（类似 Java）
- ❌ 独立的类似Rust的impl块，未来再实现
- ✅ 静态方法和实例方法都写在 type 里面

## 静态方法

### 语法

```auto
type Point<T> {
    x T
    y T

    // 静态方法
    static fn zero() Point<T> {
        Point<T> {
            x: T.default
            y: T.default
        }
    }
}
```

### 调用方式

```auto
// 方式 A: Java 风格（推荐）
let p = Point<int>.zero()
```

**设计决策**：
- 使用 `Type<T>.method()` 语法（类似 Java 的 `Type.method()`）
- 泛型参数在类型名后面：`May<int>` 而不是 `May<int>`（待确认）

## 实例方法

### 语法

```auto
type Point<T: isNumber> {
    x T
    y T

    // 实例方法
    fn is_zero() bool {
        .x == 0 && .y == 0
    }

    fn modulus() T {
        .x * .x + .y *.y
    }
}
```

### 调用方式

```auto
let p = May<int>.zero()

// 实例方法调用
if may.is_zero() {
    let val = may.x
    print(val)  // 0
}

```

**关键点**：
- 使用 `self` 关键字引用当前实例
- 访问方法和字段时，可以省略`self`，直接用`.field`这种语法
- 方法可以访问类型的字段（`.tag`, `.value`）

## a2c 转译规则

### 命名规则

**关键原则**：AutoLang 源码中**不需要模块前缀**，a2c 转译到 C 时**自动添加前缀**。

```auto
// AutoLang 源码（我们写的）
type May<T> {
    static fn empty() May<T> { ... }
    fn is_empty() bool { ... }
}
```

```c
// a2c 自动生成的 C 代码
typedef struct {
    uint8_t tag;
    void* value;
    char* error;
} May;

// a2c 自动添加 May_ 前缀
May May_empty(void) {
    // ...
}

bool May_is_empty(May* self) {
    // ...
}
```

### 转译对照表

| AutoLang 源码 | 生成的 C 代码 | 说明 |
|--------------|--------------|------|
| `type May<T>` | `typedef struct { ... } May;` | 类型定义 |
| `May.empty()` | `May_empty()` | 静态方法（加前缀） |
| `May.value(42)` | `May_value(42)` | 静态方法（加前缀） |
| `may.is_empty()` | `May_is_empty(&may)` | 实例方法（加前缀，传指针） |
| `this.tag` | `self->tag` | this 转为 self 指针 |
| `return May<T> { ... }` | `May may; ...; return may;` | 结构体初始化 |

### 完整转译示例

**AutoLang 源码**：
```auto
type May<T> {
    tag uint8
    value *T

    static fn empty() May<T> {
        return May<T> { tag: 0, value: null }
    }

    fn is_empty() bool {
        return this.tag == 0
    }
}

let may = May<int>.empty()
if may.is_empty() {
    print("empty")
}
```

**生成的 C 代码**：
```c
typedef struct {
    uint8_t tag;
    void* value;
} May;

May May_empty(void) {
    May may;
    may.tag = 0;
    may.value = NULL;
    return may;
}

bool May_is_empty(May* self) {
    return self->tag == 0;
}

// 使用
int main() {
    May may = May_empty();
    if (May_is_empty(&may)) {
        printf("empty\n");
    }
    return 0;
}
```

## 与其他语言的对比

### 与 Java 的对比

| 特性 | AutoLang | Java |
|------|----------|------|
| 方法定义位置 | 在 type 内部 | 在 class 内部 |
| 静态方法语法 | `static fn` | `static` |
| 实例方法语法 | `fn` | 普通 method |
| this 关键字 | `this` | `this` |
| 静态方法调用 | `May.empty()` | `May.empty()` |
| 泛型语法 | `May<int>` | `May<Integer>` |

### 与 Rust 的对比

| 特性 | AutoLang | Rust |
|------|----------|------|
| 方法定义位置 | 在 type 内部 | 在 impl 块内 |
| 静态方法语法 | `static fn` | `fn` (无 self) |
| 实例方法语法 | `fn` | `fn` (有 self) |
| this 关键字 | `this` | `self` |
| 模块前缀 | 不需要 | 需要 `impl Type` |

**注意**：AutoLang 不使用 Rust 的 impl 块语法，所有方法都直接写在 type 内部（类似 Java）。未来可能通过 `ext` 特性支持 impl 块。

### 与 C++ 的对比

| 特性 | AutoLang | C++ |
|------|----------|-----|
| 方法定义位置 | 在 type 内部 | 在 class 内部或外部 |
| 静态方法语法 | `static fn` | `static` |
| 实例方法语法 | `fn` | 普通 member function |
| this 关键字 | `this` | `this` (指针) |
| 命名约定 | 无前缀 | 无前缀 |

## 错误的写法（不要这样做）

### ❌ 错误 1：在 type 外部定义方法

```auto
// ❌ 错误：方法在 type 外部（Rust 风格，暂时不支持）
type May<T> { ... }

fn May_empty<T>() May<T> {
    // ...
}
```

**正确**：所有方法都写在 type 内部。

### ❌ 错误 2：手动添加模块前缀

```auto
// ❌ 错误：AutoLang 源码中不应该有前缀
type May<T> {
    static fn May_empty() May<T> { ... }  // ❌ 不要加 May_
    fn May_is_empty() bool { ... }        // ❌ 不要加 May_
}
```

**正确**：AutoLang 中不用前缀，a2c 会自动添加。

### ❌ 错误 3：手写 C 代码

```c
// ❌ 错误：不要手写 C 实现
// stdlib/may/may.c
May May_empty(void) {
    // ...
}
```

**正确**：用 AutoLang 写，a2c 自动生成 C 代码。

### ❌ 错误 4：使用 `spec extern`

```auto
// ❌ 错误：不应该用 spec extern
spec extern May_empty<T>() May<T>
```

**正确**：使用普通 `fn`，带或不带 body 都可以。

## 正确的设计流程

### 开发流程

```
1. 用 AutoLang 编写
   ↓
2. a2c 自动转译
   ↓
3. C 编译器编译
   ↓
4. 运行测试
```

### 具体步骤

1. **创建 AutoLang 文件**：`stdlib/may/may.at`
2. **用 OOP 风格编写**：方法在 type 内部，无前缀
3. **运行 a2c**：`auto.exe c may.at` → 生成 `may.c` 和 `may.h`
4. **编译 C 代码**：`gcc -c may.c -o may.o`
5. **链接和测试**：确保生成的代码正确

## 未来扩展

### ext 特性（计划中）

类似于 Rust 的 impl 块，允许在 type 外部扩展方法：

```auto
// 未来可能支持 ext 语法
ext May<T> {
    fn new_method() bool {
        // 扩展方法
    }
}
```

### Trait 系统（计划中）

类似于 Rust 的 trait，定义接口：

```auto
spec Reader {
    fn read_line() str
    fn is_eof() bool
}

impl File for Reader {
    fn read_line() str { ... }
    fn is_eof() bool { ... }
}
```

**注意**：`spec` 用于定义接口（trait），不是用于声明外部函数。
