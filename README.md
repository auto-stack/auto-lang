# Auto编程语言

Auto编程语言（Auto Lang）有如下特点：

- 设计目标：自动化
- 类型：类C
- 生态：C/C++/Rust
- 实现语言：Rust

Auto语言是Soutek公司推出的技术产品Soutek Auto Stack的一部分。


## 用途

### 1. 直接生成C源码

例如，如下两个Auto语言文件：

```rust
// math.a
pub fn add(a int, b int) int {
    a + b
}
```

```rust
// main.a
use math::add

fn main {
    println(add(1, 2))
}
```

可以生成三个C文件：math.h, math.c和main.c：

```c
// math.h
#pragma once
#include <stdint.h>

int32_t add(int32_t a, int32_t b);
```

```c
// math.c
#include <stdint.h>
#include "math.h"

int32_t add(int32_t a, int32_t b) {
    return a + b;
}
```

```c
#include <stdio.h>
#include <stdint.h>
#include "math.h"

int main(void) {
    printf("%d\n", add(1, 2));
    return 0;
}
```

Auto语言的构建器`AutoBuild`可以实现Auto/C语言项目的混合开发。

TODO: 直接生成Rust源码。

### 2. 作为配置语言，替代JSON/YAML

```rust
// 标准库
use std::str::upper;

// 变量
mut dir = "/home/user/data"

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
经过解析，上述的`.ac`文件可以解析成`JSON`格式，也可以直接提供给C/Rust代码使用。


### 3. 作为构建器

`AutoBuild`的配置文件即用Auto配置文件内`build.ac`书写。

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

### 4. 作为脚本

```rust
#!auto

// 脚本模式下内置了常用的库
print "Hello, world!"

// 下面的命令会自动转化为函数调用：`mkdir("src/app", p=true)`
mkdir -p src/app

cd src/app
touch main.rs

// 也可以定义变量和函数
let ext = ".c"
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

Auto语言根据后缀名，采用了不同的“场景”，因此可以支持不同的语法。

Auto语言的脚本（Auto Script）文件后缀名为`.as`。
在这个场景下，所有一级语句中的函数调用，都可以写成类似`bash`命令的风格。

例如：

```bash
grep -Hirn TODO .
```

会被转化为如下函数：

```rust
grep(key="TODO", dir=".", H=true, i=true, r=true, n=true)
```

Auto语言提供了一个动态执行环境（Auto Shell），可以用于脚本执行、开发调试等。

### 5. 作为模板

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
Auto模板是`AutoGen`代码生成系统的基础。

### 5. 作为UI系统的DSL

`AutoUI`是Auto语言的UI框架，基于`Zed/GPUI`实现。
可以支持Windows/Linxu/MacOS/Web等多种平台。

其中，Auto模板用来描述UI界面。

Auto模板的语法风格类似Kotlin，代码组织模式类似于Vue.js。

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

    // 视图，用来描述UI的布局
    view {
        cols {
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
            style {gap-2 w-full}
        }
        // 样式，支持Tailwind CSS语法
        style {w-24 h-24 border-1 border-color-gray-300}
    }
}
```

上面的Auto代码会被解析成一个动态的`DynamicWidget`对象，可以直接在`AutoUI`中绘制出来。

`AutoUI`支持自动重载，因此修改了`counter.a`文件后，`AutoUI`会自动重绘，不需要重新编译。

TODO：在`Release`模式中，编译器将`counter.a`代码编译成Rust代码，直接和`AutoUI`的库一起打包成可执行的UI界面程序。


## 语法概览

### 存量

在auto语言里，有四种不同类型的“存量”，用来存放与访问数据：

- 定量（`let`）：定量是声明之后就不能再改变的量，但是可以取地址和访问。相当于Rust中的`let`。
- 变量（`mut`）：这种存量的值可以任意改变，但是类型一旦确定就不能再改变。这其实就是C/C++中的普通变量。在Rust中，这样的变量用`let mut`声明。
- 常量（`const`）：常量是声明之后就不能再改变的量，但是可以取地址和访问。相当于Rust中的`const`。
- 幻量（`var`）：幻量是最自由的量，可以任意改变值和类型，一般用于脚本环境，如配置文件、DSL、脚本代码等。

```rust
// 定量
let b = 1
// Error! 定量不能修改
b = 2
// 可以用来计算新的存量
let f = e + 4
// 定量可以重新声明，但类型不能改变
let b = b * 2

// 变量定义，编译器可以自动推导类型
mut a = 1
// 变量的定义可以指定类型
mut b bool = false
// 声明多个变量
mut c, d = 2, 3

// 变量可以修改，也叫“赋值”
a = 10
// 甚至可以交换两个变量的值
c, d = d, c

// 常量定义：常量只能是全局量
const PI = 3.14

// 幻量：幻量是最自由的量，可以任意改变值和类型，一般用于脚本环境
var x = 1
x = "hello"
x = [x+"1", x+"2", x+"3"] 
```

### 数组

```rust
// 数组
let arr = [1, 2, 3, 4, 5]

// 下标
println(arr[0])
println(arr[-1]) // 最后一个元素

// 切片
let slice = arr[1..3] // [2, 3]
let slice1 = arr[..4] // [1, 2, 3, 4]
let slice2 = arr[3..] // [4, 5]
let slice3 = arr[..] // [1, 2, 3, 4, 5]

// 范围（Range）
let r = 0..10  // 0 <= r < 10
let r1 = 0..=10 // 0 <= r <= 10
```

### 对象

```rust
// 对象
mut obj = {
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

### 函数

```rust
// 函数定义
fn add(a int, b int) int {
    a + b
}

// 函数变量（Lambda）
let mul = |a int, b int| a * b

// 函数作为参数
fn calc(op |int, int| int, a int, b int) int {
    op(a, b)
}

// 函数调用
calc(add, 2, 3)
calc(mul, 2, 3)
```

### 数值的传递

在Auto语言中，值的传递可以有如下几种形式：

- 拷贝（copy）：拷贝传递，直接拷贝一份数据。
- 引用（ref）：引用传递，不需要拷贝数据，但是不可以修改原始数据。
- 转移（move）：转移传递，把值的所有权转移到目标存量，转移后原始存量就不能再用了
- 指针（ptr）：新建一个指向同一个地址的指针。可以进行底层的操作。指针只在底层的系统编程中使用，因此要放在`sys`代码块中。

引用比拷贝节省了内存空间和复制时间，但引用实际上也是通过地址进行间接访问的，所以访问时间会比拷贝略慢。

对于较小的数据，如int、float、bool，或者类似于`Point{x, y}`这种简单的数据类型，传递时进行拷贝的代价很小，往往比引用更合适。
我们把这种类型叫做“数值类型”。

对于较大的数据，如`Vec<T>`、`HashMap<K, V>`、`String`等，传递时进行拷贝的代价较大，往往用引用更合适。
我们把这种类型叫做“引用类型”。

因此，Auto语言针对不同的数据，采取了不同的传递方式：

1. 对于较小的“数值类型”的存量，默认用拷贝传递。
2. 对于较大的“引用类型”的存量，默认用引用传递。

下面举两个例子：

```rust
// 数值类型：默认拷贝传递
let a = 1
let b = a // 这里b是a的一份拷贝
mut c = a // 这里c是a的一份拷贝，而且c可以修改
c = 2
println(c) // 2
println(a) // 1 // a没有变化
```

```rust
// 引用类型：默认引用传递
let a = [1, 2, 3, 4, 5] // 数组默认是引用类型
let b = a // 这里b是a的一个引用，在使用b的时候，就和使用a一样。内存中只存在一个数组。
mut c = a // 错误！由于a是不可修改的，所以可修改的c不能引用它。
mut d = copy a // 如果想进行修改，可以显式地复制它。
d[0] = 9 // d = [9, 2, 3, 4, 5]
println(a) // a = [1, 2, 3, 4, 5]， a数组没变
```

上面的例子中，使用`copy`关键字，显式地进行了拷贝。
但这样效率显然不高，因此我们还有一个“两全其美”的办法，那就是转移：

```rust
// 转移传递
let a = [1, 2, 3, 4, 5]
let b = move a // 转移后，a不能再使用
println(a) // Error! a已经不能再使用
mut c = move b // b转移给了c，由于是转移，c可以选择修改
c[0] = 9 // c = [9, 2, 3, 4, 5]
println(b) // Error! b已经不能再使用
```

我们可以看到，`a`的值在转移到`b`之后，它的声明周期就结束了。
从此存量`a`不复存在，但它的“灵魂”会继续在`b`中存活。

同样，`b`转移给`c`时，由于转移操作实际上一种“转世重生”、“借尸还魂”，
因此`c`可以拥有和`b`不一样的属性，比如`mut`。

转移相当于把拷贝和引用的好处结合在一起了，但代价是什么呢？
代价是需要编译器能够逐行分析每个存量的生命周期。
也需要程序员能够分辨出来，某个存量，什么时候就已经挂掉了。

Rust程序员很多时候在跟编译器斗争，就是因为没搞清楚每个存量的生命周期。

由于转移和指针都是比较高阶的功能，Auto语言的早期版本暂时不会实现他们，
只是作为设计放在这里。

### 引用和指针

上面讲的拷贝和转移，都是直接操作数据，而引用和指着，则是间接地操作数据。

引用和指针的主要区别有两个：

1. 引用的作用主要是为了避免复制（例如函数传参时），方便访问。因此它用起来和原值的体验应该是一样的，所以指针虽然实际上是间接访问，但编译器做了体验优化，看起来跟直接使用一样。
2. 指针则有更多底层的功能：它可以获取地址，甚至进行地址运算。这些操作是系统级的底层代码才需要的，因此需要在`sys`代码块中执行（类似于Rust的`unsafe`块）。


```rust
// 引用
let a = [0..99999] // 我们用一个很大的数组
let b = a // 如果直接新建一个b的值，那么会把a的值拷贝一份
let c = ref a // 此时c只是a的一个“参考视图”，它本身并不存数据，也没有拷贝操作。
b = 2  // Error: 引用不能修改原始量的值

// 这里的`buf`参数，实际上是个引用
fn read_buffer(buf Buffer) {
    for n in buf.data {
        println(n)
    }
}

// mut ref可以用来修改变量：

mut x = 1
fn inc(a mut ref int) {
    a += 1
}
inc(x)
println(x) // 2
```

```rust
// 指针

// 指针和引用不同的地方在于，因为它和原始量指向同一个地址，因此可以修改原始量的值。

mut x = 1
sys {
    mut p = ptr x
    p.target += 1 // 间接修改x的值，注意这里和C不一样，用的是`.target`
}
println(x) // 2

// 在函数调用时，指针类型的参数，可以修改原始量
mut m = 10
fn inc(a ptr int) {
    a += 10
}
inc(m)
println(m) // 20

// 指针还可以直接进行地址运算
sys { // 注意：地址运算要放在sys块中
    mut arr = [1, 2, 3, 4, 5]
    mut p = ptr arr // p的类型是 Ptr<[5]int>
    println(p) // [1, 2, 3, 4, 5]
    p[0] = 101 // 直接修改arr[0]的值
    println(arr) // [101, 2, 3, 4, 5]

    mut o = p // 记住p的地址

    p.inc(2) // 地址自增2，此时p指向的是arr[2]
    println(p) // [3, 4, 5]

    println(o[0]) // 101
    p.jump(o) // 跳回到o
    println(p) // [101, 2, 3, 4, 5]
}
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

// 循环访问数组
for n in [1, 2, 3] {
    println(n)
}

// 循环修改数组的值
mut arr = [1, 2, 3, 4, 5]
for ref n in arr {
    n = n * n
}
println(arr) // [1, 4, 9, 16, 25]

// 循环一个范围
for n in 0..5 {
    println(n)
}

// 带下标的循环
for i, n in arr {
    println(f"arr[{i}] = {n}")
}

// 无限循环
mut i = 0
loop {
    println("loop")
    if i > 10 {
        break
    }
    i += 1
}

// 模式匹配，类似switch/match
when a {
    // is 用于精确匹配
    is 41 => println("a is 41"),
    // in 用于范围匹配
    in 0..9 => println("a is a single digit"),
    // if 用于条件匹配
    if a > 10 => println("a is a big number"),
    // as 用于类型判断
    as str => println("a is a string"),
    // 其他情况
    else => println("a is a weired number")
}
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
mut a = Scale.M

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
mut s = get_shape(/*...*/)
when s as Shape {
    is Point(x, y) => println(f"Point($x, $y)")
    is Rect(x, y, w, h) => println(f"Rect($x, $y, $w, $h)")
    is Circle(x, y, r) => println(f"Circle($x, $y, $r)")
    else => println("not a shape")
}
// 获取联合枚举的数据
mut p = s as Shape::Point
println(p.x, p.y)
```

### 类型（TODO）

```rust
// 类型别名
type MyInt = int

// 类型组合
type Num = int | float

// 自定以类型
type Point {
    x int
    y int

    // 方法
    fn distance(other Point) float {
        use std::math::sqrt;
        // 注意：这里的`.x`表示“在当前类型的视野中寻找变量x”，即相当于其他语言的`this.x`或`self.x`
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }
}
```

```rust
// 接口
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

// 多个方法的接口
trait Indexable {
    fn size() int
    fn get(n int) T
    fn set(n int, value T)
}

type IntArray {
    data []int

    pub fn :: new(data int...) IntArray {
        IntArray{data: data.pack()}
    }

    as Indexable<int> {
        pub fn size() int {
            .data.len()
        }

        pub fn get(n int) int {
            .data[n]
        }

        pub fn set(n int, value int) {
            .data[n] = value
        }
    }
}
```

```rust
// 新建类型的实例

// 直接赋值
mut myint = MyInt{10}
print(myint)

// 类似object的赋值
mut p = Point{x: 1, y: 2}
println(p.distance(Point{x: 4, y: 6}))

// 不同的构造函数。注意：`::`表示方法是静态方法，一般用于构造函数。静态方法里不能用`.`来访问实例成员
Point {
    pub fn :: new(x int, y int) Point {
        Point{x, y}
    }

    pub fn :: stretch(p Point, scale float) Point {
        Point{x: p.x * scale, y: p.y * scale}
    }
}

// 使用构造函数
mut p1 = Point::new(1, 2)
mut p2 = Point::stretch(p1, 2.0)

// 复杂类型判断，参数为type，且返回bool的函数，可以用来做任意逻辑的类型判断
fn IsArray(t type) bool {
    when t {
        // 数组，其元素类型可以任意
        is []_ => true
        // 实现了Iterable接口
        as Indexable => true
        else => false
    }
}

// 这里参数arr的类型只要通过了IsArray(T)的判断，就能够调用，否则报错
fn add_all(arr if IsArray) {
    mut sum = 0
    for n in arr {
        sum += n
    }
    return sum
}

// OK，因为参数是一个`[]int`数组
add_all([1, 2, 3, 4, 5])

mut d = 15
add_all(d) // Error! d既不是[]int数组，也没有实现Indexable接口

// 由于IntArray实现了Indexable接口，所以可以用于add_all
mut int_array = IntArray::new(1, 2, 3, 4, 5)
add_all(int_array)
```

### 生成器（TODO）

```rust
// 生成器
fn fib() {
    mut a, b = 0, 1
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

node ul(id=nil) {
    kids: []li
}

node label(content) {
}

ul {
    li {
        label("Item 1: ")
        button("btn1") {
            text: "Click me"
            onclick: || println("button clicked")
        }
        div { label("div1")}
    }
    li { label("Item 2") }
    li { label("Item 3") }
}
```

## 使用与安装

Auto语言编译器本身只依赖于Rust和Cargo。

```bash
> git clone git@gitee.com:auto-stack/auto-lang.git
> cd auto-lang
> cargo run
```
