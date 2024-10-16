# Auto编程语言

Auto编程语言（Auto Lang）有如下特点：

- 设计目标：自动化
- 类型：类C
- 生态：C/C++/Rust
- 实现语言：Rust

Auto语言是Soutek公司推出的技术产品Soutek Auto Stack的一部分。


## 用途

### 1. 作为配置语言，替代JSON/YAML

```rust
// 标准库
use std::str::upper;

// 变量
var dir = "/home/user/data"

// {key : value}对
root: dir
// 函数调用
root_upper: root.upper()

// 字符串
views: f"${dir}/views"
// 可以在配置中查找key
styles: f"${views}/styles"

// 对象
attrs: {
    prefix: "auto"
    // 数组
    excludes: [".git", ".auto"]
}
```

Auto语言的配置文件（Auto Config）后缀名为`.ac`。

### 2. 作为构建器

配合Auto Builder，可以实现类似CMake的C/C++工程构建：

```rust
project: "osal"
version: "v0.0.1"

// 依赖项目，可以指定参数
dep(FreeRTOS, "v0.0.3") {
    heap: "heap_5"
    config_inc: "demo/inc"
}

// 本工程中的库
lib(osal) {
    pac(hsm) {
        skip: ["hsm_test.h", "hsm_test.c"]
    }
    pac(log)
    link: FreeRTOS
}

// 可以输出到不同的平台，指定不同的编译工具链、架构和芯片
port(windows, cmake, x64, win32, "v1.0.0")
port(stm32, iar, arm_cortex_m4, f103RE, "v1.0.0")

// 可执行文件
exe(demo) {
    // 静态链接
    link: osal
    // 指定输出文件名
    outfile: "demo.bin"
}
```

### 3. 作为脚本

```rust
#!auto

// 脚本模式下内置了常用的库
print "Hello, world!"

// 下面的命令会自动转化为函数调用：`mkdir("src/app", p=true)`
mkdir -p src/app

cd src/app
touch main.rs

// 也可以定义变量和函数
var ext = ".c"
fn find_c_files(dir) {
    ls(dir).filter(|f| f.endswith(ext)).sort()
}

// 可以顺序调用命令
touch "merged.txt"
for f in find_c_files("src/app") {
    cat f >> "merged.txt"
}

// 可以异步调用多个命令
let downloads = for f in readlines("remote_files.txt").map(trim) {
    async curl f"http://database.com/download?file=${f}"
}

// 可以选择等待所有的文件都下载完成
await downloads.join()

```

Auto语言的脚本（Auto Script）文件后缀名为`.as`。
Auto语言提供了一个动态执行环境（Auto Shell），可以用于脚本执行、开发调试等。

### 4. 作为模板

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
    }
    </ul>
</body>
</html>
```

模板可以替代任意形式的文本。

Auto语言的模板（Auto Template）文件后缀名为`.at`。
Auto模板是Auto Gen代码生成系统的基础。

### 5. 作为UI系统的DSL

在Auto UI系统中，Auto模板用来描述UI界面。
Auto UI模板的语法风格类似Kotlin，组织模式类似于Vue.js。

```rust
// 定义一个组件
widget counter(id) {
    // 数据模型
    model {
        var count: i32 = 0

        fn reset() {
            count = 0
        }
    }

    // 试图，用来描述UI的布局
    view {
        cols(gap=1) {
            button("➕") {
                on_click: || count += 1
            }
            text(f"Count: {count}")
            button("➖") {
                on_click: || count -= 1
            }
            icon("🔄") {
                on_click: || reset()
            }
        }
    }

    // 样式，支持TailwindCSS的语法
    style {
        w-24
        h-24        
    }   
}
```

## 语法概览

### 变量

```rust
// 变量定义
var a = 1
// 指定类型
var b bool = false
// 多变量
var c, d = 2, 3

// 常量定义
const PI = 3.14

```

### 函数

```rust
// 函数定义
fn add(a int, b int) int {
    a + b
}

// 函数变量（Lambda)
var my_mul = |a int, b int| a * b

// 函数作为参数
fn calc(a int, b int, op fn(int, int) int) int {
    op(a, b)
}

// 函数调用
calc(2, 3, add)
calc(2, 3, my_mul)
```

### 数组

```rust
// 数组
var arr = [1, 2, 3, 4, 5]

// 下标
println(arr[0])
println(arr[-1]) // 最后一个元素

// 切片
var slice = arr[1..3] // [2, 3]
var slice1 = arr[..4] // [1, 2, 3, 4]
var slice2 = arr[3..] // [4, 5]
var slice3 = arr[..] // [1, 2, 3, 4, 5]

// 范围（Range）
var r = 0..10  // 0 <= r < 10
var r1 = 0..=10 // 0 <= r <= 10
```

### 控制流

```rust
// 条件判断
if a > 0 {
    println("a is positive")
} else if a == 0 {
    println("a is zero")
} else {
    println("a is negative")
}

// 循环
for i in r {
    println(i)
}

// 循环
for n in 0..5 {
    println(n)
}

// 带下标循环
for (i, n) in arr() {
    println(f"arr[{i}] = {n}")
}

// 无限循环
var i = 0
loop {
    println("loop")
    if i > 10 {
        break
    }
    i += 1
}

// 模式匹配
when a {
    is 41 => println("a is 41"),
    in 0..9 => println("a is a single digit"),
    if a > 10 => println("a is a big number"),
    as str => println("a is a string"),
    else => println("a is a weired number")
}
```

### 对象

```rust
// 对象
var obj = {
    name: "John",
    age: 30,
    is_student: false
}

// 访问对象成员
println(obj.name)
// 成员赋值
obj.name = "Tom"

// get or else
println(obj.get_or("name", "Unknown"))
// get or insert
println(obj.get_or_insert("name", 10))

// 所有成员
println(obj.keys())
println(obj.values())
println(obj.items())

// 遍历对象
for k, v in obj {
    println(f"obj[{k}] = {v}")
}

// 删除
obj.remove("name")
```

### 枚举（TODO）

```rust
enum Axis {
    Vertical   // 0
    Horizontal // 1
}

// 带成员的枚举
enum Scale {
    name str

    S("Small")
    M("Medium")
    L("Large")
}

// 枚举变量
var a = Scale.M

// 访问枚举成员
println(a.name)

// 枚举匹配
when a {
    is Scale::S => println("a is small")
    is Scale::M => println("a is medium")
    is Scale::L => println("a is large")
    else => println("a is not a Scale")
}


// 联合枚举
enum Shape union {
    Point(x int, y int)
    Rect(x int, y int, w int, h int)
    Circle(x int, y int, r int)
}

// 联合枚举匹配
var s = get_shape(/*...*/)
when s as Shape {
    is Point(x, y) => println(f"Point($x, $y)")
    is Rect(x, y, w, h) => println(f"Rect($x, $y, $w, $h)")
    is Circle(x, y, r) => println(f"Circle($x, $y, $r)")
    else => println("not a shape")
}
// 获取联合枚举的数据
var p = s as Shape::Point
println(p.x, p.y)
```

### 类型（TODO）

```rust
// 类型别名
type MyInt = int

// 类型组合
type Num = int | float

// 类型判断
trait Printable {
    fn print()
}

type MyInt {
    data int
}

MyInt as Printable {
    pub fn print() {
        println(.data)
    }
}

// 类型判断
var myint = MyInt{10}
print(myint)

trait Indexable {
    fn get(index int) any
}

type MyArray {
    data []any

    as Indexable {
        pub fn get(index int) any {
            .data[index]
        }
    }
}

// 复杂类型判断，参数为type，且返回bool的函数，可以用来做任意逻辑的类型判断
fn IsArray(T type) bool {
    when T {
        is []E => true
        as Iterable => true
        else => false
    }
}

// 这里参数arr的类型只要通过了IsArray(T)的判断，就能够调用，否则报错
fn add_all(arr if IsArray) {
    var sum = 0
    for n in arr {
        sum += n
    }
    return sum
}

add_all([1, 2, 3, 4, 5])

var d = 15
add_all(d) // Error! d既不是[]int数组，也没有实现Iterable接口

type MySet {
    data [int]int
    cur int

    pub static fn new(data int...) MySet {
        MySet{data: data.pack(), cur: 0}
    }

    // ...

    as Iterable {
        pub fn next() int {
            var n = .data[.cur]
            .cur += 1
            return n
        }
    }
}

// MySet实现了Iterable接口，所以可以用于for循环
add_all(MySet::new(1, 2, 3, 4, 5))
```

### 生成器（TODO）

```rust
// 生成器
fn fib() {
    var a, b = 0, 1
    loop {
        yield b
        a, b = b, a + b
    }
}

// 使用生成器
for n in fib() {
    println(n)
}

// 或者函数式
fib().take(10).foreach(|n| println(n))
```

### 异步（TODO）

```rust
// 任意函数
fn fetch(url str) str {
    // ...
}

// do关键字表示异步调用
let r = do fetch("https://api.github.com")

// 返回的是一个Future，需要等待结果
println(wait r)

// 多个异步调用
let tasks = for i in 1..10 {
    do fetch(f"https://api.github.com/$i")
}
// 等待所有任务都完成（或者超时）
let results = wait tasks
println(results)
```

### 节点

```rust
// 节点
node button(id) {
    text str
    scale Scale
    onclick fn()
}

// 新建节点
button("btn1") {
    text: "Click me"
    scale: Scale.M
    onclick: || println("button clicked")
}

// 多层节点
node div(id) {
    kids: []any
}

node li(id) {
    text str
    kids: []div
}

node ul(id) {
    kids: []li
}

ul("ul1") {
    li("li1") {
        text: "Item 1"
        button("btn1") {
            text: "Click me"
            onclick: || println("button clicked")
        }
        div("div1") {
            "div1"
        }
    }
    li("li2") {
        text: "Item 2"
    }
    li("li3") {
        text: "Item 3"
    }
}
```

## 使用与安装

Auto语言编译器本身只依赖于Rust和Cargo。

```bash
> git clone git@gitee.com:auto-stack/auto-lang.git
> cd auto-lang
> cargo run
```
