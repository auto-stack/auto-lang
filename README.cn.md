# Auto 编程语言

![icon](docs/icon.png)

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)]()
[![License](https://img.shields.io/badge/license-MIT-blue)]()
![Gitcode star](https://gitcode.com/auto-stack/auto-lang/star/badge.svg)
[![Gitee stars](https://gitee.com/auto-stack/auto-lang/badge/star.svg)](https://gitee.com/auto-stack/auto-lang)

Auto 是一门面向自动化开发的多场景编程语言，致力于成为"万物自动化"的统一解决方案。

> **One Lang to Rule Them All**


---

## 目录

- [特性](#特性)
- [快速开始](#快速开始)
- [应用场景](#应用场景)
- [语法概览](#语法概览)
- [使用与安装](#使用与安装)
- [开发路线图](#开发路线图)
- [相关项目](#相关项目)
- [许可证](#许可证)

---

## 特性

### 多场景支持

Auto 采用**场景导向（Scenario Oriented）**的设计理念，针对不同场景提供专门的语言特性：

- **Auto2C** - 作为"Better C"，转译为 C 源码，支持 Auto/C 混合工程
- **AutoConfig** - 作为配置语言，替代 JSON/XML/YAML，支持可编程配置
- **AutoScript** - 作为脚本语言，替代 Python/JavaScript，提供动态解释执行
- **AutoShell** - 作为跨平台 Shell，替代 Bash/PowerShell
- **AutoTemplate** - 作为模板语言，替代 Jinja2/Mustache。参看[教程](docs/tutorials/autogen-tutorial.cn.md)
- **AutoUI** - 作为 UI 描述语言，替代 QML/XAML/Vue

### 设计理念

- **语言即系统** - 微内核、模块化、多外设
- **动静结合** - 动态和静态类型相辅相成，动态解释和静态编译有机结合
- **生态融合** - 面向 C、Rust、JavaScript、Python 等多个生态
- **简单高效** - 脚本模式下简单易用（媲美 Python），静态模式下性能卓越（媲美 C/Rust）

---

## 快速开始

### 安装

**前置条件：** 需要安装 Rust 和 Cargo

```bash
# 克隆仓库
git clone https://gitee.com/auto-stack/auto-lang.git
cd auto-lang

# 运行 REPL（交互式解释器）
cargo run

# 运行测试
cargo test

# 构建发布版本
cargo build --release
```

### 运行

你可以直接运行一个 AutoLang 脚本：

```bash
auto hello.at
```

#### 工程管理 (AutoMan 集成)

AutoMan 的功能现在已集成到 `auto` 命令中：

```bash
auto new myapp    # 创建新工程
auto build         # 构建当前工程
auto run           # 运行构建后的工程
auto fetch         # 下载依赖
```

你也可以使用 REPL（交互式解释器）：

```bash
auto
```

---

## 应用场景

### 1️⃣ Auto2C - 转译为 C 源码

将 Auto 代码转译为高质量的 C 源码，用于嵌入式和高性能场景。

**源码**（`math.at`）：
```rust
pub fn add(a int, b int) int {
    a + b
}
```

**源码**（`main.at`）：
```rust
use math::add

fn main() {
    println(add(1, 2))
}
```

**生成的 C 代码**：
```c
// math.h
#pragma once
#include <stdint.h>
int32_t add(int32_t a, int32_t b);

// math.c
#include "stdint.h"
#include "math.h"
int32_t add(int32_t a, int32_t b) {
    return a + b;
}
```

### 2️⃣ AutoConfig - 可编程配置

作为 JSON 的超集，支持动态计算和函数调用。

```rust
use std.fs::list, is_dir

var dir = "~/code/auto"

// 支持函数调用
src: dir.join("src")
assets: `$dir/assets`

// 支持循环和条件
subs: dir.list().filter(is_dir)

// 支持嵌套对象
project: {
    name: "auto"
    skip: [".git", ".auto"]
}
```

### 3️⃣ AutoMan - 构建工具

Auto 语言的构建系统和包管理器，可作为 CMake 的替代品。

```rust
project: "myproject"
version: "v1.0.0"

// 依赖管理
dep(FreeRTOS, "v0.0.3") {
    heap: "heap_5"
}

// 库配置
lib mylib {
    dir src
    dir tests
}

// 多平台支持
port("cmake", "win32") {}
port("iar", "stm32") {}
```

### 4️⃣ AutoShell - 跨平台脚本

统一的跨平台 Shell 脚本语法。

```rust
#!auto

print "Hello, world!"

# 转换为 mkdir("src/app", p=true)
mkdir -p src/app

# 支持变量和函数
let ext = ".c"
fn find_files(dir) {
    ls(dir).filter(|f| f.endswith(ext))
}

# 支持循环
for f in find_files("src") {
    print(f)
}
```

### 5️⃣ AutoTemplate - 代码生成模板

支持任意文本格式的模板引擎。

```html
<html>
<head>
    <title>${title}</title>
</head>
<body>
    <h1>${title}</h1>
    <ul>
    $ for n in 1..10 {
        <li>Item $n</li>
    $ }
    </ul>
</body>
</html>
```

### 6️⃣ AutoUI - UI 框架

基于 Zed/GPUI 的跨平台 UI 框架，类似 Jetpack Compose。

```rust
widget Counter {
    model {
        var count: i32 = 0
    }

    view {
        col {
            button("➕") { on_click: => count += 1 }
            text(f"Count: {count}")
            button("➖") { on_click: => count -= 1 }
        }
    }
}
```

---

## 语法概览

### 存量类型

Auto 提供四种存量类型用于存储和访问数据：

| 类型 | 关键字 | 可变性 | 类型可变性 | 用途 |
|------|--------|--------|-----------|------|
| 定量 | `let` | ❌ 不可变 | ❌ 不可变 | 默认选项，类似 Rust 的 `let` |
| 变量 | `var` | ✅ 可变 | ❌ 不可变 | 需要修改值的场景 |
| 常量 | `const` | ❌ 不可变 | ❌ 不可变 | 全局常量 |
| 幻量 | (已移除) | - | - | 已合并到变量类型 |

```rust
// 定量 - 不可变
let a = 1

// 变量 - 值可变，类型不可变
var b = 2
b = 3
```

### 基本类型

```rust
// 数值类型
let a int = 42
let b float = 3.14
let c bool = true

// 数组
let arr = [1, 2, 3, 4, 5]
println(arr[0])   // 1
println(arr[-1])  // 5（最后一个元素）

// 切片
let slice = arr[1..3]  // [2, 3]

// 对象
var obj = {
    name: "John",
    age: 30
}
println(obj.name)  // "John"

// Grid（二维数组）
let grid = grid(a, b, c) {
    [1, 2, 3]
    [4, 5, 6]
}
println(grid(0))  // [1, 4]
```

### 函数

```rust
// 函数定义
fn add(a int, b int) int {
    a + b
}

// Lambda 表达式
let mul = |a int, b int| a * b

// 高阶函数
fn calc(op |int, int| int, a int, b int) int {
    op(a, b)
}

// 函数调用
calc(add, 2, 3)     // 5
calc(mul, 2, 3)     // 6
calc(|a, b| a/b, 6, 3)  // 2
```

### 控制流

```rust
// 条件判断
if a > 0 {
    println("positive")
} else if a == 0 {
    println("zero")
} else {
    println("negative")
}

// 循环
for n in 0..5 {
    println(n)
}

// 遍历数组
for i, n in arr {
    println(f"arr[{i}] = {n}")
}

// 模式匹配
is a {
    1 => println("one")
    in 2..9 => println("small")
    if a > 10 => println("big")
    as str => println("string")
    else => println("other")
}
```

### 面向对象编程

Auto 提供完整的面向对象编程支持，包括类型定义、继承、组合和特征系统。

#### 类型定义

```rust
// 定义类型
type Point {
    x int
    y int

    // 实例方法
    fn distance(other Point) float {
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }

    fn info() str {
        f"Point(.x, .y)"
    }
}

// 构造实例
var p = Point()
p.x = 1
p.y = 2
println(p.info())        // "Point(1, 2)"
println(p.distance(p))   // 0.0
```

#### 单继承（Inheritance）

使用 `is` 关键字实现单继承，子类自动获得父类的所有字段和方法：

```rust
// 父类
type Animal {
    name str

    fn speak() {
        print("Animal sound")
    }

    fn info() str {
        f"{.name}"
    }
}

// 子类继承父类
type Dog is Animal {
    breed str

    // 可以重写父类方法
    fn speak() {
        print("Woof!")
    }

    // 可以添加新方法
    fn fetch() {
        print("Fetching...")
    }
}

fn main() {
    let dog = Dog()
    dog.name = "Buddy"
    dog.breed = "Labrador"

    // 访问继承的字段
    print(dog.name)

    // 调用继承的方法（被重写）
    dog.speak()  // "Woof!"

    // 调用自己的方法
    dog.fetch()
}
```

**继承特性**：
- ✅ 字段继承：子类自动包含父类的所有字段
- ✅ 方法继承：子类自动获得父类的所有方法
- ✅ 方法重写：子类可以重写父类方法
- ✅ 类型检查：继承关系在编译时验证

#### 组合（Composition）

使用 `has` 关键字实现组合，将其他类型的功能集成到当前类型：

```rust
type Engine {
    power int

    fn start() {
        print("Engine started")
    }
}

type Car {
    has engine Engine

    fn drive() {
        .engine.start()
        print("Driving...")
    }
}
```

#### 特征系统（Spec）

Spec 定义接口契约，类型可以实现多个 spec：

```rust
// 定义 spec
spec Reader {
    fn read() str
    fn is_eof() bool
}

spec Writer {
    fn write(s str)
    fn flush()
}

// 实现 spec（使用 as 关键字）
type File as Reader, Writer {
    path str

    fn read() str {
        // 读取文件
    }

    fn is_eof() bool {
        // 检查是否结束
    }

    fn write(s str) {
        // 写入文件
    }

    fn flush() {
        // 刷新缓冲
    }
}

// 多态函数
fn copy(src Reader, dst Writer) {
    while !src.is_eof() {
        let line = src.read()
        dst.write(line)
    }
    dst.flush()
}
```

#### 转译器支持

Auto 的 OOP 特性同时支持 C 和 Rust 转译：

**C 转译**（扁平结构体 + 方法前缀）：
```c
struct Dog {
    char* name;      // 继承的字段
    char* breed;     // 自己的字段
};

void Dog_Speak(struct Dog *self) {
    printf("%s\n", "Woof!");
}
```

**Rust 转译**（扁平结构体 + impl 块）：
```rust
struct Dog {
    name: String,      // 继承的字段
    breed: String,     // 自己的字段
}

impl Dog {
    fn speak(&self) {
        println!("Woof!");
    }
}
```

> 📖 **更多 OOP 特性**？查看 [单继承实现文档](docs/plans/021-single-inheritance.md) 和 [Spec 多态文档](docs/plans/020-stdlib-io-expansion.md)

---

## 使用与安装

### 系统要求

- **Rust** 1.70 或更高版本
- **Cargo**（随 Rust 一起安装）
- **CMake** 3.15+（可选，用于 C 版本构建）
- **Visual Studio** 或 **MinGW**（Windows 可选）

### 安装步骤

```bash
# 1. 克隆仓库
git clone https://gitee.com/auto-stack/auto-lang.git
cd auto-lang

# 2. 构建并运行 REPL
cargo run

# 3. 运行示例
cargo run -- examples/hello.at

# 4. 运行测试
cargo test

# 5. 构建 C 版本（可选）
cd autoc
mkdir build && cd build
cmake ..
cmake --build .
```

### 编辑器支持

- **VS Code** - 即将支持
- **Zed** - 计划中（基于 GPUI）
- **其他编辑器** - 欢迎贡献语法高亮配置

---

## 开发路线图

### 当前进度

| 功能 | 状态 | 备注 |
|------|------|------|
| **Auto2C** | 🟡 v0.1 | 基础功能可用，v0.2 计划支持完整特性 |
| **AutoConfig** | 🟢 已完成 | 静态版（Atom）和动态版都已实现 |
| **AutoScript** | 🟡 可用 | 基础解释器完成，生态集成待实现 |
| **AutoUI** | 🟡 基础版 | 支持组件、样式、事件响应 |
| **AutoTemplate** | 🟢 已完成 | 已在实际项目中使用 |
| **AutoShell** | 🔵 开发中 | 核心语法支持，内置命令完善中 |
| **自举编译器** | 🔵 早期阶段 | `auto/` 目录，刚开始实现 |

### 计划中的功能

- [ ] 完整的 Auto2C 转译器（v0.2）
- [ ] Rust 生态集成（FFI）
- [ ] Python/JavaScript 生态集成
- [ ] 异步支持（async/await）
- [ ] 生成器（yield）
- [ ] IDE 和插件系统
- [ ] 包管理器（AutoPM）
- [ ] WebAssembly 支持

---

## 相关项目

Auto 是 [AutoStack](https://gitee.com/auto-stack) 生态系统的一部分：

- **[AutoMan](https://gitee.com/auto-stack/auto-man)** - 构建工具和包管理器
- **[AutoUI](https://gitee.com/auto-stack/auto-ui)** - 跨平台 UI 框架
- **[AutoGen](https://gitee.com/auto-stack/auto-gen)** - 代码生成工具
- **[AutoShell](https://gitee.com/auto-stack/auto-shell)** - 跨平台 Shell

---

## 贡献

欢迎贡献代码、报告问题或提出建议！

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

### 开发指南

- 代码规范：待补充
- 提交规范：使用清晰的提交信息
- 测试要求：所有新功能需要添加测试

---

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 联系方式

- **Gitee**: https://gitee.com/auto-stack/auto-lang
- **Issues**: https://gitee.com/auto-stack/auto-lang/issues
- **讨论**: 欢迎在 Issues 中提出问题或建议

---

## 致谢

Auto 语言由 Soutek 公司开发并开源，感谢所有贡献者的支持！

**Soutek AutoStack** - 让自动化开发更简单
