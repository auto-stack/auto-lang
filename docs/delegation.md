# Delegation (成员级委托)

## 概述

AutoLang 的 **delegation** 功能允许类型将成员变量作为特定 trait/spec 的实现来使用。这是一种轻量级的组合方式，使类型可以通过成员变量获得功能，而无需自己实现所有方法。

## 语法

```auto
type TypeName {
    has member_name MemberType for SpecName
}
```

- `member_name`: 成员变量的名称
- `MemberType`: 成员变量的类型
- `SpecName`: 要委托的 spec/trait 名称

## 示例

### 基础用法

```auto
spec Engine {
    fn start()
}

type WarpDrive {
    fn start() {
        print("WarpDrive engaging")
    }
}

type Starship {
    has core WarpDrive for Engine
}

fn main() {
    let ship = Starship()
    ship.start()  // 调用 ship.core.start()
}
```

**生成的 Rust 代码**:
```rust
trait Engine {
    fn start(&self);
}

struct WarpDrive {}

impl WarpDrive {
    fn start(&self) {
        println!("WarpDrive engaging");
    }
}

struct Starship {
    core: WarpDrive,
}

impl Engine for Starship {
    fn start(&self) {
        self.core.start()
    }
}
```

### 多个委托

一个类型可以委托给多个成员：

```auto
spec Engine {
    fn start()
}

spec Weapon {
    fn fire()
}

type WarpDrive {
    fn start() {
        print("Engaging")
    }
}

type LaserCannon {
    fn fire() {
        print("Pew!")
    }
}

type Starship {
    has core WarpDrive for Engine
    has weapon LaserCannon for Weapon
}

fn main() {
    let ship = Starship()
    ship.start()   // 委托给 ship.core.start()
    ship.fire()    // 委托给 ship.weapon.fire()
}
```

### 带参数的方法

Delegation 支持带参数的方法转发：

```auto
spec Calculator {
    fn add(a int, b int) int
    fn multiply(a int, b int) int
}

type MathEngine {
    fn add(a int, b int) int {
        a + b
    }

    fn multiply(a int, b int) int {
        a * b
    }
}

type Computer {
    has engine MathEngine for Calculator
}

fn main() {
    let comp = Computer()
    let result = comp.add(5, 3)  // 调用 comp.engine.add(5, 3)
    print(result)
}
```

**生成的 Rust 代码**:
```rust
impl Calculator for Computer {
    fn add(&self, a: i32, b: i32) -> i32 {
        self.engine.add(a, b)
    }

    fn multiply(&self, a: i32, b: i32) -> i32 {
        self.engine.multiply(a, b)
    }
}
```

## 与传统 `has` 的区别

### 传统的 `has` (类型级别组合)

```auto
type Wing {
    fn fly() { print("flying") }
}

type Duck has Wing {
}

fn main() {
    let d = Duck()
    d.fly()  // 方法来自 Wing
}
```

**特点**:
- 继承 Wing 的**所有**字段和方法
- 字段和方法混合到 Duck 中
- 适合"是一个"关系 (is-a)

### Delegation (成员级委托)

```auto
spec Flyer {
    fn fly()
}

type Wing {
    fn fly() { print("flying") }
}

type Duck {
    has wing Wing for Flyer
}

fn main() {
    let d = Duck()
    d.fly()  // 通过 wing 成员调用
}
```

**特点**:
- 只委托**指定 spec** 的方法
- 成员保持独立：`d.wing.fly()` 也可以调用
- 适合"有一个"关系 (has-a)
- 更明确的接口

## 转译器支持

### C Transpiler

生成的 C 代码使用包装函数：

```c
struct Starship {
    struct WarpDrive core;
};

// 委托包装方法
void Starship_start(struct Starship *self) {
    WarpDrive_start(&self->core);
}

int main(void) {
    struct Starship ship = {};
    Starship_start(&ship);
    return 0;
}
```

### Rust Transpiler

生成的 Rust 代码使用原生 trait：

```rust
struct Starship {
    core: WarpDrive,
}

impl Engine for Starship {
    fn start(&self) {
        self.core.start()
    }
}
```

### Evaluator

运行时方法解析会检查委托链：

1. 首先在类型自身查找方法
2. 如果没找到，检查每个委托的 spec
3. 找到匹配的 spec 后，从实例获取成员变量
4. 递归调用成员的方法

## 使用场景

### 1. 组合优于继承

当你想要复用实现但不希望继承整个类型时：

```auto
type Database {
    fn query(sql str) { /* ... */ }
}

type UserRepository {
    has db Database for Database
}

fn main() {
    let repo = UserRepository()
    repo.query("SELECT * FROM users")
}
```

### 2. Mixin 模式

组合多个小功能：

```auto
spec Loggable {
    fn log(msg str)
}

spec Cacheable {
    fn get(key str) any
    fn set(key str, value any)
}

type Logger {
    fn log(msg str) { print(msg) }
}

type MemoryCache {
    fn get(key str) any { /* ... */ }
    fn set(key str, value any) { /* ... */ }
}

type Service {
    has logger Logger for Loggable
    has cache MemoryCache for Cacheable
}
```

### 3. 适配器模式

将现有类型适配到新接口：

```auto
spec JsonSerializable {
    fn to_json() str
}

type XmlParser {
    fn to_xml() str { /* ... */ }
}

type DataAdapter {
    has parser XmlParser for JsonSerializable
}

// 实现 JsonSerializable trait，但内部使用 XML
```

## 限制

1. **单向委托**: 只能从类型委托给成员，不能反向
2. **无方法重写**: 委托的方法不能在类型中重写（需要使用传统 `has`）
3. **显式接口**: 必须指定委托哪个 spec，不能委托"所有方法"

## 未来改进

- [ ] 方法重写与 `super` 关键字
- [ ] 委托链 (delegation chain)
- [ ] 条件委托
- [ ] 委托检查与类型推导
