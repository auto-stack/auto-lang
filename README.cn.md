# Auto编程语言

![icon](docs/icon.png)

Auto语言是一门跨生态的“融合语言”，基本理念是：

- 语言即系统（Language as OS）：微内核、模块化、多外设。
- 灵活多变（Scenario Oriented）：面向不同场景，提供所需的语言特性。
- 动静结合（Dynamic+Static）：动态和静态类型相辅相成，动态解释和静态编译有机结合。
- 生态融合（Ecosytem Friendly）：面向C、Rust、Javascript、Python等多个生态。

Auto语言应用在汽车行业，未来会不断探索，拓宽边界。

Auto语言的远景目标是“万物自动化”（*One Lang to Rule Them All*）。

Auto语言是Soutek公司推出的技术产品Soutek AutoStack的开源版本。

## 用途

Auto语言可用于如下场景：
- 作为`Better C`，生成C/Rust源码
- 作为配置语言，替代JSON/XML/YAML
- 作为数据格式语言，替代JSON/ProtoBuf
- 作为构建工具，替代CMake/XMake
- 作为UI界面描述语言，替代QML/XAML/Vue
- 作为脚本语言，替代Python/Javascript
- 作为模板语言，替代Jinja2/Mustache
- 作为跨平台Shell，替代Bash/PowerShell

### 1. AutoLang生成C源码

例如，如下两个AutoLang语言文件：

```rust
// math.at
pub fn add(a int, b int) int {
    a + b
}
```

```rust
// main.at
use math.add

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

### 2. AutoConfig，可以转换成JSON或YAML

```rust
// 可以调用标准库
use std.fs: list, join, is_dir

// 变量
mut dir = "~/code/auto"

// {key : value}对，这是配置数据的一部分
root: dir
// 函数调用
src: dir.join("src")

// 字符串拼接，$表示变量查找
assets: `$dir/assets`
// 字符串拼接，#表示在配置中查找key
styles: `#assets/styles`

// 数组
docs: ["README.md", "LICENSE"]

// 所有的子目录
subs: dir.list().filter(is_dir)

// 对象
project: {
    name: "auto"
    skip: [".git", ".auto"]
}
```

上面的配置对应的JSON文件如下：

```json
{
    "dir": "/home/user/data",
    "src": "/home/user/data/src",
    "assets": "/home/user/data/assets",
    "styles": "/home/user/data/assets/styles",
    "docs": ["README.md", "LICENSE"],
    "subs": ["/home/user/data/src/app", "/home/user/data/src/lib"],
    "project": {
        "name": "auto",
        "skip": [".git", ".auto"]
    }
}
```

为了更方便得生成类XML/YAML的树状结构，AutoConfig提供了*节点*（Node）的概念。
节点和数组、对象结合起来，形成AutoConfig的树状结构。

```rust
parent(k1: v1, k2: v2) {
    kid(k1: v1) {
        // more kids
    }

    kid(k2: v2) {
        // more kids
    }
}
```

这和XML的语法结构是一一对应的，相当于：

```xml
<parent k1="v1" k2="v2">
    <kid k1="v1" />
    <kid k2="v2" />
</parent>
```

例如，下面的配置文件表示一张成绩表：

```rust
class(name: "三3班", count: 55) {
    student(name: "张三", age: 18) {
        score(subject: "语文", score: 80) {}
        score(subject: "数学", score: 90) {}
        score(subject: "英语", score: 85) {}
    }

    student(name: "李四", age: 19) {
        score(subject: "语文", score: 85) {}
        score(subject: "数学", score: 95) {}
        score(subject: "英语", score: 80) {}
    }

    student(name: "王五", age: 20) {
        score(subject: "语文", score: 85) {}
        score(subject: "数学", score: 95) {}
        score(subject: "英语", score: 80) {}
    }
}
```

对应的XML如下：

```xml
<class name="三3班" count="55">
    <student name="张三" age="18">
        <score subject="语文" score="80" />
        <score subject="数学" score="90" />
        <score subject="英语" score="85" />
    </student>
    <student name="李四" age="19">
        <score subject="语文" score="85" />
        <score subject="数学" score="95" />
        <score subject="英语" score="80" />
    </student>
</class>
```

乍一看大家可能会觉得，“这和直接写XML有什么区别”？
一旦数据多起来，AutoConfig的可编程优势就体现出来了：

```rust

// 调用函数获取学生信息
let info = fetch_class_scores("三3班")

class(name: info.name, count: info.count) {
    for s in info.students {
        student(name: s.name, age: s.age) {
            for score in s.scores {
                score(subject: score.subject, score: score.score) {}
            }
        }
    }
}
```

这种基于节点的格式，非常适合用来描述UI等树状配置。
和XML、YAML不同，AutoConfig是可编程的，因此要更加灵活强大。

### 3. AutoMan，作为构建器的配置文件

`AutoMan`是Auto语言的构建工具，支持编译、依赖包管理和Auto/C混合编程。
`AutoMan`可以看作是CMake的替代品。

`AutoMan`的配置文件一般叫做`pac.at`，用来描述一个工程包。

```rust
project: "osal"
version: "v0.0.1"

// 依赖项目，可以指定参数
dep("FreeRTOS", "v0.0.3") {
    heap: "heap_5"
    config_inc: "demo/inc"
}

// 本工程中的库
lib("osal") {
    // 子目录
    dir("hsm") {
        skip: ["hsm_test.h", "hsm_test.c"]
    }
    dir("log") {}
    link: FreeRTOS
}

// 可以输出到不同的平台，指定不同的编译工具链、架构和芯片
port("cmake", "win32") {}
port("iar", "ls1480") {
    // 芯片对应的SDK
    device("Lanshan-LS1480-SDK") {
    }
}

// 可执行文件
app("demo") {
    // 静态链接
    links: ["osal"]
    // 指定输出文件名
    out: "demo.bin"
}
```

### 4. AutoShell

AutoShell是类似Bash的Shell脚本语言，可以试下跨平台统一语法。
相对于AutoScript，添加了对Shell命令调用格式的支持：

```bash
mkdir -p src/app
```

上述的Shell脚本语法会被转换为AutoScript中的如下代码：

```rust
mkdir("src/app", p=true)
```

这样设计有4个好处：

1. 用户可以使用熟悉的命令行风格调用命令，又可以保留AutoScript的强大编程能力
2. 跨平台：不论是在Windows的PowerShell中，还是Linux的Bash中，都可以执行相同的AutoShell脚本
3. AutoShell可以和其他的Auto程序无缝集成

下面是AutoShell的一些用法展示：

```rust
// Auto的Shell模式
#!auto

// 脚本模式下内置了常用的库
print "Hello, world!"

// 下面的命令会自动转化为函数调用：`mkdir("src/app", p=true)`
mkdir -p src/app

// 更多的命令
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

// 如果是AutoShell没有支持的命令，也可以调用底层真正的shell程序：
// NOTE: 这个模式下，语法就不是跨平台的了，因此需要做平台判断
is sys.shell() {
    sys.POWERSHELL => shell("del -Force -Recurse ./logs")
    sys.BASH => shell("rm -rf ./logs")
}
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

### 5. AutoTemplate，可以生成任意格式文本的模板语言

类似于Python的`Jinja2`模板。

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

AutoTemplate是`AutoGen`代码生成系统的基础。

### 6. AutoUI，用Auto语言来描述UI界面

`AutoUI`是Auto语言的UI框架，基于`Zed/GPUI`实现。
可以支持Windows/Linxu/MacOS/Web等多种平台。

AutoUI的语法风格类似Kotlin，代码组织模式类似于Vue.js。

```rust
// 任何一个实现了`View`特征的类型，都可以作为一个UI组件来使用
type Counter as View {
    // 类型的成员，相当于MVC中的Model
    count int = 0

    // `view`方法，返回一个节点数据。相当于MVC中的View
    fn view() {
        col {
            button("+") {
                onclick: "click:inc" // 点击事件，暂时用字符串类型，未来会扩展成枚举数据类型
            }
            label(`Count: ${count}`) {}
            button("-") {
                onclick: "click:dec" // 点击事件，暂时用字符串类型，未来会扩展成枚举数据类型
            }
            button("reset") {
                onclick: "click:reset" // 点击事件，暂时用字符串类型，未来会扩展成枚举数据类型
            }
        }
    }

    // 事件处理函数
    fn on(ev str) {
        is ev {
            "click:inc" => count += 1
            "click:dec" => count -= 1
            "click:reset" => count = 0
            else print(`Unknown event: ${ev}`)
        }
    }
}

fn main() {
    app("Counter Example") {
        counter() {}
    }
}
```

上述的Auto代码会被翻译成对应的RustUI库代码，然后编译成可执行的UI程序。

下面是生成的UI界面：

![Counter](https://foruda.gitee.com/images/1730021021429704035/4625e3ce_142056.png)

Note: 在上一版实现（参见分支`gpui2`），`AutoUI`不是通过翻译成Rust，而是在Rust代码中动态解析，直接渲染出来的。

这样的动态解释有如下好处：

1. 可以实时重载，修改了`counter.at`文件后，`AutoUI`会自动重绘，不需要重新编译。
2. 可以动态改变UI的渲染方式，比如在Debug模式下，可以渲染出组件的边界等信息，方便调试。
3. 甚至可以做成动态拖拽的UI设计工具，直接一边浏览一边修改。

但这样的坏处是：

1. 性能较低，UI的数据都是动态解析的。
2. 占用更多内存，此时的动态数据格式和事件响应代码，都是用解释器执行的，因此会占用更多的资源。

在最近的版本`gpui3`中，我尝试了转译成Rust的方案。这么做运行效率最高，但是开发时的响应周期会比较低。

因此，未来考虑将两种方案都实现，并根据场景选择使用。

- 在`Debug`模式中，使用动态解释，方便开发调试。
- 在`Release`模式中，使用转译成Rust，生成静态的UI程序。

注意：上述所有场景用途，并未完全实现，现在进度大致如下：

- `Better C`：v0.1初步完成了Auto2C的转译器，可以翻译简单的函数、控制流等。预计v0.2版本实现绝大部分功能。
- 配置语言：已完成，有静态版（Atom）和动态版（AutoConfig）两个层次。
- 脚本语言：有一个初步可用的动态解释器，但对Rust/Python/JS等生态的动态调用功能还未实现。
- UI界面：完成了简单的界面，可以配置UI组件、设置Style、组合、布局、模块化、事件响应等。
- 模板：基本可用，已经用在一些简单项目中了。
- Shell：正在开发中。

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

### Grid

Grid是Auto语言的二维数组，可以用于表格数据。
Grid可以扩展为类似DataFrame/Tensor的多维结构，用来和Python交互，进行AI相关的开发。

```rust
// 定义一个Grid
let data = grid {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// 转化为JSON
var json = data.to_json()
```

生成的JSON如下：

```JSON
{
    "data": [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ]
}
```

这样搞看起来似乎没什么区别，但grid提供了比JSON更多的功能：

```rust
// 获取grid的信息
println(data.shape()) // (3, 3)
println(data.width()) // 3
println(data.height()) // 3

// 按行访问
println(data[0]) // [1, 2, 3]
println(data[1]) // [4, 5, 6]
println(data[2]) // [7, 8, 9]
// 或者
println(data.row(0)) // [1, 2, 3]
println(data.row(1)) // [4, 5, 6]
println(data.row(2)) // [7, 8, 9]

// 按列访问
println(data(0)) // [1, 4, 7]
println(data(1)) // [2, 5, 8]
println(data(2)) // [3, 6, 9]
// 或者
println(data.col(0)) // [1, 4, 7]
println(data.col(1)) // [2, 5, 8]
println(data.col(2)) // [3, 6, 9]

// 矩阵转换
let transposed = data.transpose()
// 其他矩阵操作
let sum = data.sum()
let mean = data.mean()
let std = data.std()
let min = data.min()
let max = data.max()
```

除了这些操作之外，Grid还支持更丰富的行列信息配置：

```rust
let data = grid("a", "b", "c") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// 转换成JSON数组时，可以指定命名形式，即每行数据都是一个对象：

let json = data.to_json(named:true)
```

得到如下JSON：

```JSON
{
    "grid": {
        "cols": ["a", "b", "c"],
        "rows": [
            {"a": 1, "b": 2, "c": 3},
            {"a": 4, "b": 5, "c": 6},
            {"a": 7, "b": 8, "c": 9}
        ]
    }
}
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
fn calc(a int, b int, op |int, int| int) int {
    op(a, b)
}

// 函数调用
calc(2, 3, add)
calc(4, 5, mul)
calc(6, 7, |a, b| a / b)
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
is a {
    // is 用于精确匹配
    41 => print("a is 41")
    // 多个不同值
    42 or 43 or 44 => print("a is a little bigger")
    // in 用于范围匹配
    in 0..9 => print("a is a single digit")
    // if 用于条件匹配
    if a > 10 => print("a is a big number")
    // as 用于类型判断
    as str => print("a is a string")
    // 其他情况
    else => print("a is a weired number")
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
is a as Scale {
    S => println("a is small")
    M => println("a is medium")
    L => println("a is large")
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
is s as Shape {
    Point(x, y) => println(f"Point($x, $y)")
    Rect(x, y, w, h) => println(f"Rect($x, $y, $w, $h)")
    Circle(x, y, r) => println(f"Circle($x, $y, $r)")
    else => println("not a shape")
}
// 获取联合枚举的数据
mut p = s as Shape.Point
println(p.x, p.y)
```

### 类型

```rust
// 类型别名
type MyInt = int

// 类型组合
type Num = int | float

// 自定义类型
type Point {
    x int
    y int

    // 方法
    fn distance(other Point) float {
        use std.math.sqrt;
        // 这里的`x`相当于其他语言的`this.x`
        sqrt((x - other.x) ** 2 + (y - other.y) ** 2)
    }
}
```

在类型的方法中，对象实例的成员，如上面例子里的`x`和`y`，可以直接访问。这是因为Auto语言在方法调用时，会把对象实例的视野也加入到方法的视野中。

假如实例成员的名称和参数或者局部存量名字冲突，则可以使用`.x`来区分。
此时`.x`表示成员，即相当于`this.x`或`self.x`；
而`x`则表示参数或普通存量。

例如：

```rust
// 自定义类型
type Point {
    x int
    y int

    // 方法
    fn move(x int, y int) int {
        .x = x
        .y = y
    }
}
```

如果想要直接使用实例自身，则可以用`self`表示。

```rust
type Node {
    parent *Node
    kids []*Node

    pub fn mut add(mut kid *Node) {
        kid.parent = &self
        .kids.add(kid)
    }
}
```

注意方法中的两个`mut`：

- `fn`与方法名之间的`mut`表示要修改实例自身
- 参数前的`mut`表示要修改这个参数

```rust
// 新建类型的实例

// 默认构造函数
mut myint = MyInt(10)
print(myint)

// 命名构造函数
mut p = Point(x:1, y:2)
println(p.distance(Point(x:4, y:6)))

// 自定义构造函数。注意：`static`表示方法是静态方法，一般用于构造函数。静态方法里不能用`.`来访问实例成员
Point {
    pub static fn new(x int, y int) Point {
        Point{x, y}
    }

    pub static fn stretch(p Point, scale float) Point {
        Point{x: p.x * scale, y: p.y * scale}
    }
}

// 使用自定义构造函数
mut p1 = Point.new(1, 2)
mut p2 = Point.stretch(p1, 2.0)
```

除了在类型内部定义方法，我们还可以在外部给类型“扩展”新的方法。
扩展方法的关键字是`ext`，即`extends`

```rust
type Point {
    pub x int
    pub y int
}

ext Point {
    pub fn to_str() str {
        `Point($x, $y)`
    }
}
```

注意，和内部方法不同，扩展方法只能访问`Point`中公开的成员，
因此我们上面的例子给`x`和`y`添加了`pub`修饰。

如果需要访问私有的变量，那么直接把方法定义在类型内部即可。

扩展方法的用处是可以给第三方库定义好的类型，
甚至系统类型添加新的功能。

例如，如果要给系统的字符串类型`str`添加一个新功能：

```rust
ext str {
    pub fn shape_shift() {
        for c in self {
            if c.is_up() {
                c.lower()
            } else {
                c.upper()
            }
        }
    }
}

let s = "HellO"
let t = s.shape_shift()
assert_eq(t, "hELLo")
```


### 特征（Spec）

Auto语言扩展了Rust的接口（trait）概念，可以支持更多的模式匹配。
在Auto语言中，这种用来判断类型特征的结构，被称为一个类型的特征（Spec）。

Auto的特征有三类：

1. 接口（Interface Spec）：类似于Java的Interface和Rust的trait，可以通过类型支持的方法列表来判别。

```rust
// 接口特征 Interface Spec
spec Printer {
    // 符合Printable特征的类型，必须有print方法
    fn print()
}

// 自定义的类型
type MyInt {
    data int

    // 直接实现接口的方法
    pub fn print() {
        println(.data)
    }
}

// 也可以通过扩展类型方法来实现
ext MyInt {
    pub fn print() {
        println(.data)
    }
}

// 接口可以包含多个方法
spec Indexer<T> {
    fn size() usize
    fn get(n usize) T
    fn set(n usize, value T)
}

type IntArray {
    data []int

    pub static fn new(data int...) IntArray {
        IntArray{data: data.pack()}
    }

    // 实现Indexer接口
    pub fn size() int {
        data.len()
    }

    pub fn get(n int) int {
        data[n]
    }

    pub fn set(n int, value int) {
        data[n] = value
    }
}
```

2. 表达式特征（Expression Spec）：类似于TypeScript的联合类型。

```rust
// 表达式特征

spec Number = int | uint | byte | float

// 使用表达式特征
fn add(a Number, b Number) Number {
    a + b
}

add(1, 2) // OK
add(1, 2.0) // OK
add(1, "2") // Error!

// 如果名字太长，也可以这么写：
fn <T = Number> add(a T, b T) T {
    a + b
}
```

表达式特征还可以用来实现类型别名：

```rust
spec MyInt = int
```

此时，MyInt就等价于int，可以用于任何需要int的地方。
对于C/C++语言程序员，这就相当于一个宏。


3. 判别函数特征（Predicate Spec或Function Spec）：在编译期调用一个判别函数，如果返回true，则表示类型判定通过。

用于类型特征的判别函数，其参数是`type`类型，返回值是`bool`类型。

```rust
fn predicate(t type) bool {
    // ...
    true
}
```

例如，下面的`IsArray`函数用来判别是不是可以线性迭代：

```rust
// 判别函数
fn IsIterable(t type) bool {
    is t {
        // 是一个数组，其元素类型可以任意
        as []any => true
        // 或者有next()方法
        if t.has_method("next") => true
        // 或者实现了Indexer接口
        as Indexer => true
        else => false
    }
}

// 这里参数arr的类型只要通过了IsArray(T)的判断，就能够调用，否则报错
// 注意：这里使用了`if`表达式，表示在编译期调用判别函数
fn add_all(arr if IsIterable) {
    mut sum = 0
    for n in arr {
        sum += n
    }
    return sum
}

let arr = [1, 2, 3, 4, 5]
// OK，因为arr是一个`[]int`数组
add_all(arr)

mut my_arr = IntArray.new(1, 2, 3, 4, 5)
// OK，因为my_arr实现了Indexer接口
add_all(my_arr)

mut d = "hello"
// Error! d既不是[]int数组，也没有实现Indexer接口
add_all(d)
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
// 节点定义，可以提前指定节点的属性对应的类型
node button {
    text str
    scale Scale
    onclick str
}

// 新建节点，其id为btn1
button btn1 {
    text: "Click me"
    scale: Scale.M
    onclick: "click:btn1"
}

// 多层节点
ul {
    li {
        label {"Item 1"}
        button btn1 {
            text: "Click me"
            onclick: "click:btn1"
        }
        div { label {"div1"} }
    }
    li { label {"Item 1"} }
    li { label {"Item 3"} }
}
```

## 使用与安装

Auto语言编译器本身只依赖于Rust和Cargo。

```bash
> git clone git@gitee.com:auto-stack/auto-lang.git
> cd auto-lang
> cargo run
```
