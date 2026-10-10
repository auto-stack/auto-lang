# AutoLang：语言概览

[English](https://github.com/auto-stack/auto-lang/blob/master/docs/language/overview.md) · [Auto 生态](https://github.com/auto-stack/auto-lang/blob/master/README.cn.md) · [动手巡礼](../tour/ch01-hello.md)

本文介绍当前语言与主要执行路径，依据 **2026-10-10** 的仓库源码及
[v0.5 里程碑说明](../releases/v0.5.md) 核查整理。
这是一篇入门概览，各特性在不同后端的覆盖范围仍需分别确认。

## 一门语言，多种用途

Auto 将脚本开发与显式类型、静态发布路径结合起来，语法也用于结构化配置和声明式界面。
生态中包含 AutoVM、转译器、AutoMan 构建工具、AutoUI 和面向应用的 AI 服务。

| 用途 | 主要路径 |
|---|---|
| 脚本与应用逻辑 | 在 AutoVM 中运行 `.at` 源码 |
| 动态脚本便利语法 | `.as` 文件或 `#[script]`，经脚本 lowering 管线处理 |
| 原生发布 | a2r 转译为 Rust，再用 Rust 工具链构建 |
| Rust 库 | 内建 natives 与构建期 `dep` / `use.rs` 垫片 |
| Python 库 | 通过 `use.py` 嵌入 CPython；a2py 生成 Python |
| 界面 | AutoUI 状态/事件/视图，通过 Vue 或原生 iced 渲染 |
| 配置与结构化内容 | 对象、命名值和 Node/Atom 形式 |

Rust 是主要静态发布路径。C、TypeScript、JavaScript、Python、GDScript 和 Godot 场景输出
各有生成器与测试面，接入某个生态不代表自动兼容所有库，也不代表全部目标行为完全一致。

## 运行与构建

已有 PATH 中的 `auto` 二进制时，在仓库根目录直接运行源码：

```sh
auto docs/tour/ch01-hello/01_hello.at
```

程序内容如下：

```auto
fn main() {
    print("Hello, World!")
}
```

`print` 输出一行。脚本也可以直接包含顶层语句，`main` 是组织程序的方便方式，
并非每个脚本都必须具备的入口。

```sh
# 查看 CLI 和可用的转译目标。
auto --help
auto trans --help

# 从单个源码文件生成 Rust。
auto trans --path docs/tour/ch01-hello/01_hello.at rust
```

最后一个命令生成源码，编译与打包还需要后续步骤。工程开发用 `auto new`、`auto build`
和 `auto run`，配置写在项目的 `pac.at` 中。
[从脚本到发布](https://github.com/auto-stack/auto-lang/blob/master/docs/script-to-ship/README.cn.md) 介绍完整工作方式。

从源码构建时，将 `auto-lang` 与 `auto-down` 克隆为同级目录；当前 workspace 的
`autodown-core` 通过相邻仓库的相对路径解析。在 `auto-lang` 内运行：

```sh
cargo build -p auto --release
```

默认 CLI 的功能包含 `ui-iced`、`python` 和 `autodown`，需准备 Rust/原生编译环境以及
匹配的 CPython 开发环境。产物为 `target/release/auto`（Windows 上为 `auto.exe`）。
若不需要可选的 Python/AutoDown 集成，CLI 包可使用 `--no-default-features`；
语言 crate 的默认 iced 依赖仍在，Cargo 仍需解析相邻路径依赖。
实际选项以 [当前 manifest](https://github.com/auto-stack/auto-lang/blob/master/crates/auto/Cargo.toml) 为准。

## 绑定、值与集合

`let` 声明不可变绑定，`var` 声明可变绑定，`const` 定义常量。
类型可以推断，也可以写在名称后面，名称与类型之间不需要冒号：

```auto
fn main() {
    let name = "Auto"
    var count int = 0
    count = count + 1
    print(f"Hello, $name!")
    print(count)
}
```

基本值包括整数、浮点数、布尔、字符串、数组和对象。
普通类型绑定与动态脚本便利语法有不同的类型规则；可变不意味着类型可以任意改变。
`None`/`Some` 表示可选值，`nil` 则是另一种值，不能把它们当作同一概念的不同写法。

```auto
fn main() {
    let numbers = [1, 2, 3]
    var total = 0
    for n in numbers {
        total += n
    }
    let person = {name: "Auto", age: 5}
    print(total)
    print(person.name)
}
```

字符串支持插值，集合提供索引和内建操作。
[集合巡礼](../tour/ch07-collections.md) 展示数组、列表、映射和遍历。

## 函数与控制流

函数参数采用 `名称 类型`，返回类型写在参数列表之后。可以显式 `return`，
也可以让最后一个表达式成为结果。闭包使用 `(参数) => 表达式`：

```auto
fn add(a int, b int) int {
    a + b
}

fn main() {
    let base = 10
    let add_base = (x int) => x + base
    print(add(3, 4))
    print(add_base(5))
}
```

控制流包含 `if`/`else`、`for`、`while`、`loop`、`break` 和 `continue`。
`is` 执行模式匹配，各分支使用 `->`：

```auto
fn describe(n int) str {
    is n {
        0 -> return "zero"
        1 -> return "one"
    }
    return "other"
}

fn main() {
    print(describe(0))
    print(describe(42))
}
```

深入示例见 [函数](../tour/ch03-functions.md)、[控制流](../tour/ch04-control.md)
与 [模式匹配](../tour/ch05-patterns.md)。

## 类型、方法、Spec 与泛型

`type` 定义字段和方法，`enum` 定义备选项，也支持携带数据的变体。
`ext` 添加方法，`spec` 描述接口契约，类型参数用于泛型代码。

```auto
type Point {
    x int
    y int
}

fn identity<T>(x T) T {
    return x
}

fn main() {
    let p = Point { x: 3, y: 4 }
    print(p.x)
    print(identity(42))
}
```

所有权与借用属于语言的静态路径。`view`、`mut` 和 `take` 分别表达共享访问、可变访问和转移。
VM 值处理与生成 Rust 的所有权模型是不同实现，具体代码行为应通过相应例子和对拍确认。

可继续阅读 [类型](../tour/ch02-types.md)、[方法与 Spec](../tour/ch08-methods.md)、
[泛型](../tour/ch09-generics.md) 和 [类型与所有权](https://github.com/auto-stack/auto-lang/blob/master/docs/script-to-ship/ch03-types-ownership.cn.md)。

## 错误与可选值

`!int` 一类返回类型表示函数可能出错，`Err(...)` 返回失败，`.?` 解开成功值或向上传播错误。
`?T`、`Some`、`None` 和 `??` 用于可选值。

```auto
fn try_parse(s str) !int {
    if s == "" {
        return Err("empty string")
    }
    Ok(42)
}

fn chain() !int {
    let a = try_parse("hello").?
    let b = try_parse("").?
    Ok(a + b)
}

let result = chain()
is result {
    Ok(n) -> print(n)
    Err(e) -> print("propagated error")
}
```

这段[错误传播语料](https://github.com/auto-stack/auto-lang/blob/master/crates/auto-lang/test/vm/16_option_result/020_result_propagate/result_propagate.at)的输出是 `propagated error`。

参见 [错误处理](../tour/ch06-errors.md) 和 [发布示例](https://github.com/auto-stack/auto-lang/blob/master/docs/script-to-ship/ch04-errors.cn.md)。
面向 Python 的脚本模式另有异常、上下文管理器处理，使用独立的 lowering 和互操作测试。

## 异步、Actor 与生成器

异步值使用 `~T`，通过 `.await` 获取结果。Actor 程序使用 `task` 状态、消息和处理器，
其 VM 调度与 Rust 发布路径分别有验证面。
[Actor 语料](https://github.com/auto-stack/auto-lang/tree/master/crates/auto-lang/test/vm/23_actor) 展示真实消息驱动的例子。

生成器通过 `yield` 产生值，随着遍历按需执行：

```auto
fn counter() ~Iter<int> {
    yield 1
    yield 2
    yield 3
}

fn main() {
    var sum = 0
    for n in counter() {
        sum = sum + n
    }
    print(sum)
}
```

这是已有的 [VM 生成器示例](https://github.com/auto-stack/auto-lang/blob/master/crates/auto-lang/test/vm/22_generator/001_sum/sum.at)。
更多内容见 [异步示例](../tour/ch11-async.md) 和 [生成器语料](https://github.com/auto-stack/auto-lang/tree/master/crates/auto-lang/test/vm/22_generator)。
这些能力已有实现，使用时仍需检查具体操作的执行和转译覆盖范围。

## 模块与外部生态

用 `use` 导入 Auto 模块，AutoVM 负责多模块加载与链接；项目依赖和构建配置位于 `pac.at`。
[模块巡礼](../tour/ch10-modules.md) 介绍可见性、文件组织和导入。

| 互操作 | 能力 | 验证与资料 |
|---|---|---|
| Rust | natives、`dep`/`use.rs` 垫片、签名元数据、布局探针与具体泛型实例 | [Rust 生态](https://github.com/auto-stack/auto-lang/blob/master/website/zh/rust.md) · [Rust 库语料](https://github.com/auto-stack/auto-lang/tree/master/parity/libs/rust) |
| Python | `use.py` 接入 CPython 对象；`.as` lowering、kwargs、异常、上下文管理器与 a2py | [Python 生态](https://github.com/auto-stack/auto-lang/blob/master/website/zh/python.md) · [Python/PyTorch 语料](https://github.com/auto-stack/auto-lang/tree/master/parity/libs/python) |
| C | C 绑定与 C ABI 互操作，C 转译是另一条路径 | [FFI 指南](../guides/ffi-usage-guide.md) · [C 绑定示例](../tour/ch12-interop.md) |

兼容性取决于具体签名、所有权/布局要求、库、运行时和后端。
已有桥接不意味着所有生命周期敏感或 unsafe Rust 接口可直接使用，也不意味着全部 Python
语言和库兼容。[Parity](https://github.com/auto-stack/auto-lang/blob/master/parity/README.md) 给出 AutoVM、转译产物与原生实现的具体对照。

## 配置、Node 与 AutoUI

对象、命名值和 Node 形式让 Auto 可以表达配置与结构化内容。
[`pac.at`](https://github.com/auto-stack/auto-lang/blob/master/examples/ui/002-counter/pac.at) 配置项目，
[`auto-atom`](https://github.com/auto-stack/auto-lang/tree/master/crates/auto-atom) 提供结构化 Atom 数据表示。
编译期求值与注解支持元编程，详见 [comptime 模块](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/auto-lang/comptime/overview.md)。

AutoUI 的 `widget` 将模型状态、视图和事件连接起来。以下摘自
[当前计数器](https://github.com/auto-stack/auto-lang/blob/master/examples/ui/002-counter/src/front/app.at)：

```auto,ignore
widget App {
    model {
        var count int = 0
    }
    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
            style: "items-center gap-4 p-6"
        }
    }
}
```

通过 `auto run examples/ui/002-counter -r vue` 或 `-r vm` 运行。
Vue 路径生成 Web 组件，VM 路径经 iced 渲染，Rust UI 路径则生成原生代码。
状态与事件共用编写模型，各后端覆盖分别验证。
[AutoUI](https://github.com/auto-stack/auto-lang/blob/master/website/zh/ui/index.md)、[UI 项目](https://github.com/auto-stack/auto-lang/blob/master/examples/ui/README.md) 和
[Blueprint](https://github.com/auto-stack/auto-lang/blob/master/blueprints/README.md) 介绍组件、样式、图表及可复用设计。
Android/ArkTS 当前属于 Demo 与可行性建设。

## 工具、参考与下一步

- **AutoMan / AutoCache**：项目构建、依赖准备、代码生成和产物缓存。
- **LSP / 调试器**：编辑器诊断、导航和执行检视。
- **DevTools / MCP**：实时 UI 树、布局、事件和截图，参见 [验证指南](../guides/autoui-verification-and-mcp-guide.md)。
- **Playground / 书籍**：[浏览器示例页](https://github.com/auto-stack/auto-lang/blob/master/website/zh/playground.md)、[本地网站开发](https://github.com/auto-stack/auto-lang/blob/master/website/README.md) 和 [Auto 书籍](https://github.com/auto-stack/books)。
- **自举**：[`auto/`](https://github.com/auto-stack/auto-lang/tree/master/auto) 包含实验性 Auto VM/转译器；[当前范围](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/aavm/project.md)
  区分这条 Rust 辅助的自举环路与参考实现。

从 [巡礼](../tour/README.md) 动手学习，用 [从脚本到发布](https://github.com/auto-stack/auto-lang/blob/master/docs/script-to-ship/README.cn.md)
了解发布工作流，用 [v0.5 说明](../releases/v0.5.md) 了解进展与已知边界。
[完整规范](specification.md) 是较早的版本化草案，并非冻结的 v0.5 规范。
当草案与当前行为不一致时，模块 [Specs](https://github.com/auto-stack/auto-lang/blob/master/docs/specs/auto-lang/project.md)、实际源码与验证语料
提供更精确的实现信息。
