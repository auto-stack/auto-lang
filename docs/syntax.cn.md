# Auto 语言语法详解

本文档详细介绍 Auto 语言的语法特性、类型系统和高级特性。

---

## 目录

- [存量类型](#存量类型)
  - [定量 (let)](#定量-let)
  - [变量 (mut)](#变量-mut)
  - [常量 (const)](#常量-const)
  - [幻量 (var)](#幻量-var)
- [基本类型](#基本类型)
  - [数值类型](#数值类型)
  - [数组](#数组)
  - [对象](#对象)
  - [Grid](#grid)
- [函数](#函数)
  - [函数定义](#函数定义)
  - [Lambda 表达式](#lambda-表达式)
  - [高阶函数](#高阶函数)
  - [函数调用](#函数调用)
- [数值传递](#数值传递)
  - [拷贝传递](#拷贝传递)
  - [引用传递](#引用传递)
  - [转移传递](#转移传递)
  - [指针](#指针)
- [引用和指针](#引用和指针)
- [控制流](#控制流)
- [类型系统](#类型系统)
- [特征 (Spec)](#特征-spec)
- [枚举](#枚举)
- [生成器](#生成器)
- [异步](#异步)
- [节点](#节点)

---

## 存量类型

在 Auto 语言中，有四种不同类型的"存量"，用于存储和访问数据。它们在可变性和类型稳定性上各有特点。

### 定量 (let)

**定量**是声明之后就不能再改变的量，但可以取地址和访问。相当于 Rust 中的 `let`。

```rust
// 定量声明
let b = 1

// Error! 定量不能修改
b = 2  // 编译错误

// 可以用来计算新的存量
let f = e + 4

// 定量可以重新声明，但类型不能改变
let b = b * 2  // shadowing，类型仍是 int
```

**适用场景**：
- 不需要修改的值
- 函数参数（默认情况）
- 计算中间结果

### 变量 (mut)

**变量**的值可以任意改变，但类型一旦确定就不能再改变。这其实就是 C/C++ 中的普通变量。在 Rust 中，这样的变量用 `let mut` 声明。

```rust
// 变量定义，编译器可以自动推导类型
mut a = 1

// 变量的定义可以指定类型
mut b bool = false

// 声明多个变量
mut c, d = 2, 3

// 变量可以修改，也叫"赋值"
a = 10

// 甚至可以交换两个变量的值
c, d = d, c

// Error! 类型不能改变
a = "hello"  // 编译错误：a 的类型是 int，不能赋值为 str
```

**适用场景**：
- 需要修改的局部状态
- 循环计数器
- 累加器等

### 常量 (const)

**常量**是声明之后就不能再改变的量，且只能在全局作用域声明。相当于 Rust 中的 `const`。

```rust
// 常量定义：常量只能是全局量
const PI = 3.14

// Error! 常量不能修改
PI = 3.15  // 编译错误

// 常量可以在任何地方使用
fn area(radius float) float {
    PI * radius * radius
}
```

**适用场景**：
- 全局配置
- 数学常量
- 魔法数字的替代

### 幻量 (var)

**幻量**是最自由的量，可以任意改变值和类型，一般用于脚本环境，如配置文件、DSL、脚本代码等。

```rust
// 幻量：幻量是最自由的量，可以任意改变值和类型
var x = 1
println(x)  // 1

x = "hello"
println(x)  // "hello"

x = [x+"1", x+"2", x+"3"]
println(x)  // ["hello1", "hello2", "hello3"]
```

**适用场景**：
- 动态配置文件
- 脚本语言模式
- DSL（领域特定语言）
- 快速原型开发

---

## 基本类型

### 数值类型

Auto 支持基本的数值类型：

```rust
// 整数类型
let a int = 42         // 有符号整数 (i32)
let b uint = 42        // 无符号整数 (u32)
let c byte = 42        // 字节 (u8)

// 浮点类型
let d float = 3.14     // 双精度浮点 (f64)

// 布尔类型
let e bool = true
let f bool = false

// 空类型
let g nil = nil        // nil 是特殊的零尺寸类型
```

**nil 类型的特殊行为**：
```rust
// nil 是一个特殊类型，它是零尺寸类型
var c = nil

// 包含 nil 的操作总会返回 nil
let d = nil + 1  // d 是 nil
```

### 数组

数组是固定大小的同类型元素序列。

```rust
// 数组字面量
let arr = [1, 2, 3, 4, 5]

// 下标访问
println(arr[0])    // 1（第一个元素）
println(arr[-1])   // 5（最后一个元素）

// 切片
let slice = arr[1..3]   // [2, 3]
let slice1 = arr[..4]    // [1, 2, 3, 4]
let slice2 = arr[3..]    // [4, 5]
let slice3 = arr[..]     // [1, 2, 3, 4, 5]（完整拷贝）

// 范围（Range）
let r = 0..10    // 0 <= r < 10（半开区间）
let r1 = 0..=10  // 0 <= r <= 10（闭区间）
```

**数组操作**：
```rust
mut arr = [1, 2, 3, 4, 5]

// 修改元素
arr[0] = 10

// 数组长度
let len = arr.len()

// 遍历数组
for n in arr {
    println(n)
}

// 带索引遍历
for i, n in arr {
    println(f"arr[{i}] = {n}")
}

// 修改数组的值（使用 ref）
mut arr = [1, 2, 3, 4, 5]
for ref n in arr {
    n = n * n
}
println(arr)  // [1, 4, 9, 16, 25]
```

### 对象

对象是键值对的集合，类似于其他语言中的 map、dict 或 hash。

```rust
// 对象字面量
mut obj = {
    name: "John",
    age: 30,
    is_student: false
}

// 访问对象成员
println(obj.name)   // "John"

// 成员赋值
obj.name = "Tom"

// 方法调用
println(obj.get_or("name", "Unknown"))     // "Tom"
println(obj.get_or("job", "Unknown"))      // "Unknown"（默认值）

println(obj.get_or_insert("name", 10))     // "Tom"（已存在）
println(obj.get_or_insert("job", "Dev"))   // "Dev"（插入并返回）

// 获取所有成员
println(obj.keys())    // ["name", "age", "is_student"]
println(obj.values())  // ["Tom", 30, false]
println(obj.items())   // [("name", "Tom"), ("age", 30), ...]

// 遍历对象
for k, v in obj {
    println(f"obj[{k}] = {v}")
}

// 删除成员
obj.remove("name")
```

### Grid

Grid 是 Auto 语言的二维数组，专门用于表格数据。Grid 可以扩展为类似 DataFrame/Tensor 的多维结构，用来和 Python 交互，进行 AI 相关的开发。

#### 基本 Grid

```rust
// 定义一个 Grid
let data = grid {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// 转化为 JSON
var json = data.to_json()
```

生成的 JSON：
```json
{
    "data": [
        [1, 2, 3],
        [4, 5, 6],
        [7, 8, 9]
    ]
}
```

#### 带列名的 Grid

```rust
// 定义带列名的 Grid
let data = grid("a", "b", "c") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// 转换成命名格式的 JSON
let json = data.to_json(named: true)
```

生成的 JSON：
```json
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

#### Grid 操作

```rust
let data = grid("a", "b", "c") {
    [1, 2, 3]
    [4, 5, 6]
    [7, 8, 9]
}

// 获取 grid 信息
println(data.shape())   // (3, 3)
println(data.width())   // 3
println(data.height())  // 3

// 按行访问
println(data[0])        // [1, 2, 3]
println(data[1])        // [4, 5, 6]
println(data[2])        // [7, 8, 9]

// 或者使用 row 方法
println(data.row(0))    // [1, 2, 3]

// 按列访问
println(data(0))        // [1, 4, 7]
println(data(1))        // [2, 5, 8]
println(data(2))        // [3, 6, 9]

// 或者使用 col 方法
println(data.col(0))    // [1, 4, 7]

// 矩阵操作
let transposed = data.transpose()
let sum = data.sum()
let mean = data.mean()
let std = data.std()
let min = data.min()
let max = data.max()
```

---

## 函数

### 函数定义

```rust
// 基本函数定义
fn add(a int, b int) int {
    a + b
}

// 无返回值函数
fn greet(name str) {
    println(f"Hello, ${name}!")
}

// 多返回值（使用元组）
fn divmod(a int, b int) (int, int) {
    (a / b, a % b)
}

// 使用
let (quot, rem) = divmod(10, 3)
```

### Lambda 表达式

Lambda 表达式（匿名函数）是一种简洁的函数定义方式。

```rust
// Lambda 表达式
let mul = |a int, b int| a * b

// 调用
println(mul(3, 4))  // 12

// 简化的 Lambda（类型推断）
let add = |a, b| a + b

// 多语句 Lambda
let complex = |a, b| {
    let temp = a + b
    temp * temp
}
```

### 高阶函数

函数可以作为参数传递，也可以作为返回值。

```rust
// 函数作为参数
fn calc(op |int, int| int, a int, b int) int {
    op(a, b)
}

// 函数作为返回值
fn get_op(op_type str) |int, int| int {
    is op_type {
        "add" => |a, b| a + b
        "sub" => |a, b| a - b
        "mul" => |a, b| a * b
        "div" => |a, b| a / b
        else => |a, b| 0
    }
}

// 使用
let add = |a int, b int| a * b
calc(add, 2, 3)         // 6
calc(|a, b| a / b, 6, 3)  // 2

let op = get_op("add")
op(1, 2)  // 3
```

### 函数调用

```rust
// 普通调用
let result = add(1, 2)

// 命名参数（如果支持）
let result = create_user(name: "Alice", age: 30)

// 方法调用
obj.method()
obj.method(arg1, arg2)

// 链式调用
obj.method1().method2().method3()

// 可选参数
fn greet(name str, greeting str = "Hello") {
    println(f"${greeting}, ${name}!")
}

greet("Bob")                  // Hello, Bob!
greet("Bob", "Hi")            // Hi, Bob!
```

---

## 数值传递

在 Auto 语言中，值的传递可以有几种形式：拷贝、引用、转移和指针。不同的传递方式在性能和语义上各有特点。

### 拷贝传递

**拷贝传递**直接复制一份数据。

```rust
// 数值类型：默认拷贝传递
let a = 1
let b = a     // 这里 b 是 a 的一份拷贝
mut c = a     // 这里 c 是 a 的一份拷贝，而且 c 可以修改
c = 2
println(c)    // 2
println(a)    // 1（a 没有变化）
```

**特点**：
- 简单直接，易于理解
- 对于小数据类型（int、bool 等）效率高
- 对于大数据类型（数组、对象）效率低

### 引用传递

**引用传递**不需要拷贝数据，但不可以修改原始数据。

```rust
// 引用类型：默认引用传递
let a = [1, 2, 3, 4, 5]  // 数组默认是引用类型
let b = a  // 这里 b 是 a 的一个引用，在使用 b 的时候，就和使用 a 一样。内存中只存在一个数组。

// Error! 由于 a 是不可修改的，所以可修改的 c 不能引用它
mut c = a  // 编译错误

// 如果想进行修改，可以显式地复制它
mut d = copy a
d[0] = 9  // d = [9, 2, 3, 4, 5]
println(a)  // a = [1, 2, 3, 4, 5]，a 数组没变
```

**数值类型 vs 引用类型**：

- **数值类型**：int、float、bool、byte、简单的 struct（如 `Point{x, y}`）
  - 默认拷贝传递
  - 拷贝代价小

- **引用类型**：数组、对象、字符串等
  - 默认引用传递
  - 拷贝代价大

### 转移传递

**转移传递**把值的所有权转移到目标存量，转移后原始存量就不能再使用了。

```rust
// 转移传递
let a = [1, 2, 3, 4, 5]
let b = move a  // 转移后，a 不能再使用

// Error! a 已经不能再使用
println(a)  // 编译错误

mut c = move b  // b 转移给了 c，由于是转移，c 可以选择修改
c[0] = 9  // c = [9, 2, 3, 4, 5]

// Error! b 已经不能再使用
println(b)  // 编译错误
```

**转移的特点**：
- 零拷贝（高性能）
- 转移后原变量失效
- 编译器会检查生命周期
- `c` 可以拥有和 `b` 不同的属性（如 `mut`）

> **注意**：转移和指针都是比较高阶的功能，Auto 语言的早期版本暂时不会完全实现，只是作为设计放在这里。

### 指针

**指针**新建一个指向同一个地址的指针，可以进行底层的操作。指针只在底层的系统编程中使用，因此要放在 `sys` 代码块中。

详见下一节 [引用和指针](#引用和指针)。

---

## 引用和指针

上面讲的拷贝和转移，都是直接操作数据，而引用和指针，则是间接地操作数据。

引用和指针的主要区别有两个：

1. **引用**的作用主要是为了避免复制（例如函数传参时），方便访问。因此它用起来和原值的体验应该是一样的，虽然实际上是间接访问，但编译器做了体验优化，看起来跟直接使用一样。

2. **指针**则有更多底层的功能：它可以获取地址，甚至进行地址运算。这些操作是系统级的底层代码才需要的，因此需要在 `sys` 代码块中执行（类似于 Rust 的 `unsafe` 块）。

### 引用

```rust
// 引用
let a = [0..99999]  // 我们用一个很大的数组
let b = a  // 如果直接新建一个 b 的值，那么会把 a 的值拷贝一份

let c = ref a  // 此时 c 只是 a 的一个"参考视图"，它本身并不存数据，也没有拷贝操作

// Error: 引用不能修改原始量的值
b = 2  // 编译错误（假设 b 是 ref）

// 函数参数中的引用
// 这里的 `buf` 参数，实际上是个引用
fn read_buffer(buf Buffer) {
    for n in buf.data {
        println(n)
    }
}

// 可变引用：用来修改变量
mut x = 1
fn inc(a mut ref int) {
    a += 1
}
inc(x)
println(x)  // 2
```

### 指针

指针和引用不同的地方在于，因为它和原始量指向同一个地址，因此可以修改原始量的值。

```rust
// 指针可以修改原始量
mut x = 1
sys {
    mut p = ptr x
    p.target += 1  // 间接修改 x 的值，注意这里和 C 不一样，用的是 `.target`
}
println(x)  // 2

// 函数参数中的指针
// 在函数调用时，指针类型的参数，可以修改原始量
mut m = 10
fn inc(a ptr int) {
    a += 10
}
inc(m)
println(m)  // 20

// 指针的地址运算
sys {  // 注意：地址运算要放在 sys 块中
    mut arr = [1, 2, 3, 4, 5]
    mut p = ptr arr  // p 的类型是 Ptr<[5]int>
    println(p)  // [1, 2, 3, 4, 5]

    p[0] = 101  // 直接修改 arr[0] 的值
    println(arr)  // [101, 2, 3, 4, 5]

    mut o = p  // 记住 p 的地址

    p.inc(2)  // 地址自增 2，此时 p 指向的是 arr[2]
    println(p)  // [3, 4, 5]

    println(o[0])  // 101
    p.jump(o)  // 跳回到 o
    println(p)  // [101, 2, 3, 4, 5]
}
```

---

## 控制流

Auto 提供了丰富的控制流语句。

### 条件判断

```rust
// if-else
if a > 0 {
    println("a is positive")
} else if a == 0 {
    println("a is zero")
} else {
    println("a is negative")
}

// if 表达式
let abs = if x >= 0 { x } else { -x }

// 嵌套 if
if condition1 {
    if condition2 {
        // ...
    }
}
```

### 循环

```rust
// for 循环：遍历数组
for n in [1, 2, 3] {
    println(n)
}

// for 循环：修改数组的值
mut arr = [1, 2, 3, 4, 5]
for ref n in arr {
    n = n * n
}
println(arr)  // [1, 4, 9, 16, 25]

// for 循环：范围
for n in 0..5 {
    println(n)  // 0, 1, 2, 3, 4
}

// for 循环：带下标的循环
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

// while 循环
while condition {
    // ...
}

// break 和 continue
for i in 0..10 {
    if i == 5 {
        break    // 退出循环
    }
    if i % 2 == 0 {
        continue // 跳过本次迭代
    }
    println(i)
}
```

### 模式匹配 (is)

`is` 语句用于模式匹配，类似于其他语言的 switch/match，但更强大。

```rust
// 精确匹配
is a {
    41 => println("a is 41")
    42 => println("a is 42")
}

// 多个值
is a {
    42 or 43 or 44 => println("a is a little bigger")
}

// 范围匹配
is a {
    in 0..9 => println("a is a single digit")
    in 10..99 => println("a is two digits")
}

// 条件匹配
is a {
    if a > 10 => println("a is a big number")
    if a < 0 => println("a is negative")
}

// 类型判断
is a {
    as str => println("a is a string")
    as int => println("a is an integer")
    as float => println("a is a float")
}

// else 分支
is a {
    1 => println("one")
    2 => println("two")
    else => println("other")
}

// 组合使用
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
    else => print("a is a weird number")
}
```

---

## 类型系统

Auto 提供了强大的类型系统，支持类型别名、类型组合、自定义类型等。

### 类型别名

```rust
// 类型别名
type MyInt = int

let a MyInt = 42
```

类型别名相当于 C/C++ 中的 `typedef`，不会创建新类型，只是给现有类型起个别名。

### 类型组合

```rust
// 类型组合（联合类型）
type Num = int | float

fn add(a Num, b Num) Num {
    a + b
}

add(1, 2)      // OK
add(1, 2.0)    // OK
add(1.0, 2.0)  // OK
// add(1, "2") // Error!
```

### 自定义类型

```rust
// 自定义类型
type Point {
    x int
    y int

    // 方法
    fn distance(other Point) float {
        use std.math.sqrt
        // 这里的 `x` 相当于其他语言的 `this.x` 或 `self.x`
        // 如果名字冲突，可以用 `.x` 表示成员，`x` 表示参数
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }
}
```

在类型的方法中，对象实例的成员（如上面例子里的 `x` 和 `y`）可以直接访问。这是因为 Auto 语言在方法调用时，会把对象实例的视野也加入到方法的视野中。

如果实例成员的名称和参数或者局部存量名字冲突，则可以使用 `.x` 来区分：
- `.x` 表示成员（即相当于 `this.x` 或 `self.x`）
- `x` 则表示参数或普通存量

#### 成员访问和 self

```rust
type Point {
    x int
    y int

    // 直接访问成员（无冲突）
    fn distance(other Point) float {
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }

    // 名字冲突时使用 .x 区分
    fn move(x int, y int) {
        .x = x  // 成员 x
        .y = y  // 成员 y
    }
}
```

如果想要直接使用实例自身，则可以用 `self` 表示：

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

#### 构造函数

```rust
type Point {
    x int
    y int

    // 方法
    fn distance(other Point) float {
        sqrt((.x - other.x) ** 2 + (.y - other.y) ** 2)
    }
}

// 默认构造函数
mut myint = MyInt(10)
print(myint)

// 命名构造函数
mut p = Point(x: 1, y: 2)
println(p.distance(Point(x: 4, y: 6)))

// 自定义构造函数
// 注意：`static` 表示方法是静态方法，一般用于构造函数
// 静态方法里不能用 `.` 来访问实例成员
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

### 扩展方法

除了在类型内部定义方法，我们还可以在外部给类型"扩展"新的方法。扩展方法的关键字是 `ext`，即 `extends`。

```rust
type Point {
    pub x int
    pub y int
}

ext Point {
    pub fn to_str() str {
        f"Point(${.x}, ${.y})"
    }
}
```

注意，和内部方法不同，扩展方法只能访问 `Point` 中公开的成员，因此我们上面的例子给 `x` 和 `y` 添加了 `pub` 修饰。

如果需要访问私有的变量，那么直接把方法定义在类型内部即可。

扩展方法的用处是可以给第三方库定义好的类型，甚至系统类型添加新的功能。

例如，如果要给系统的字符串类型 `str` 添加一个新功能：

```rust
ext str {
    pub fn shape_shift() {
        for c in self {
            if c.is_upper() {
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

---

## 特征 (Spec)

Auto 语言扩展了 Rust 的接口（trait）概念，可以支持更多的模式匹配。在 Auto 语言中，这种用来判断类型特征的结构，被称为一个类型的**特征（Spec）**。

Auto 的特征有三类：

1. **接口特征 (Interface Spec)**：类似于 Java 的 Interface 和 Rust 的 trait
2. **表达式特征 (Expression Spec)**：类似于 TypeScript 的联合类型
3. **判别函数特征 (Predicate Spec/Function Spec)**：编译期判别函数

### 接口特征 (Interface Spec)

类似于 Java 的 Interface 和 Rust 的 trait，可以通过类型支持的方法列表来判别。

```rust
// 定义接口特征
spec Printer {
    // 符合 Printable 特征的类型，必须有 print 方法
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
```

**多方法接口**：

```rust
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

    // 实现 Indexer 接口
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

### 表达式特征 (Expression Spec)

类似于 TypeScript 的联合类型。

```rust
// 表达式特征
spec Number = int | uint | byte | float

// 使用表达式特征
fn add(a Number, b Number) Number {
    a + b
}

add(1, 2)      // OK
add(1, 2.0)    // OK
// add(1, "2") // Error!

// 如果名字太长，也可以这么写：
fn <T = Number> add(a T, b T) T {
    a + b
}
```

**类型别名**：

表达式特征还可以用来实现类型别名：

```rust
spec MyInt = int
```

此时，MyInt 就等价于 int，可以用于任何需要 int 的地方。对于 C/C++ 语言程序员，这就相当于一个宏。

### 判别函数特征 (Predicate Spec)

在编译期调用一个判别函数，如果返回 true，则表示类型判定通过。

用于类型特征的判别函数，其参数是 `type` 类型，返回值是 `bool` 类型。

```rust
// 判别函数
fn predicate(t type) bool {
    // ...
    true
}
```

**示例：IsIterable 判别器**

```rust
// 判别函数
fn IsIterable(t type) bool {
    is t {
        // 是一个数组，其元素类型可以任意
        as []any => true
        // 或者有 next() 方法
        if t.has_method("next") => true
        // 或者实现了 Indexer 接口
        as Indexer => true
        else => false
    }
}

// 这里参数 arr 的类型只要通过了 IsArray(T) 的判断，就能够调用，否则报错
// 注意：这里使用了 `if` 表达式，表示在编译期调用判别函数
fn add_all(arr if IsIterable) {
    mut sum = 0
    for n in arr {
        sum += n
    }
    return sum
}

let arr = [1, 2, 3, 4, 5]
// OK，因为 arr 是一个 `[]int` 数组
add_all(arr)

mut my_arr = IntArray.new(1, 2, 3, 4, 5)
// OK，因为 my_arr 实现了 Indexer 接口
add_all(my_arr)

mut d = "hello"
// Error! d 既不是 []int 数组，也没有实现 Indexer 接口
// add_all(d)
```

---

## 枚举

> **注意**：枚举功能尚未实现，以下是设计规范。

### 基本枚举

```rust
enum Axis {
    Vertical   // 0
    Horizontal // 1
}

// 使用
let axis = Axis.Vertical
println(axis)  // 0
```

### 带成员的枚举

```rust
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
println(a.name)  // "Medium"

// 枚举匹配
is a as Scale {
    Scale.S => println("a is small")
    Scale.M => println("a is medium")
    Scale.L => println("a is large")
    else => println("a is not a Scale")
}
```

### 联合枚举

```rust
// 联合枚举（类似于 Rust 的 enum）
enum Shape union {
    Point(x int, y int)
    Rect(x int, y int, w int, h int)
    Circle(x int, y int, r int)
}

// 联合枚举匹配
mut s = get_shape(/*...*/)
is s as Shape {
    Shape.Point(x, y) => println(f"Point(${x}, ${y})")
    Shape.Rect(x, y, w, h) => println(f"Rect(${x}, ${y}, ${w}, ${h})")
    Shape.Circle(x, y, r) => println(f"Circle(${x}, ${y}, ${r})")
    else => println("not a shape")
}

// 获取联合枚举的数据
mut p = s as Shape.Point
println(p.x)
println(p.y)
```

---

## 生成器

> **注意**：生成器功能尚未实现，以下是设计规范。

生成器允许你惰性地生成值序列，节省内存。

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
    // 1, 1, 2, 3, 5, 8, 13, ...
}

// 或者函数式风格
fib().take(10).foreach(|n| println(n))
```

**生成器方法**：

- `take(n)` - 取前 n 个元素
- `skip(n)` - 跳过前 n 个元素
- `foreach(fn)` - 对每个元素执行函数
- `map(fn)` - 映射每个元素
- `filter(fn)` - 过滤元素
- `collect()` - 收集为数组

---

## 异步

> **注意**：异步功能尚未实现，以下是设计规范。

Auto 语言支持异步编程模型，类似于 JavaScript 的 async/await 或 Rust 的 async/await。

```rust
// 任意函数
fn fetch(url str) str {
    // ...
}

// do 关键字表示异步调用
let r = do fetch("https://api.github.com")

// 返回的是一个 Future，需要等待结果
println(wait r)

// 多个异步调用
let tasks = for i in 1..10 {
    do fetch(f"https://api.github.com/${i}")
}
// 等待所有任务都完成（或者超时）
let results = wait tasks
println(results)
```

**异步关键字**：

- `do` - 异步调用函数，返回 Future
- `wait` - 等待 Future 完成，获取结果
- `async` - 标记异步函数（可选）

**并发模式**：

```rust
// 并发执行多个任务
let tasks = [
    do fetch("url1"),
    do fetch("url2"),
    do fetch("url3")
]
let results = wait tasks

// 超时等待
let results = wait tasks timeout 5000  // 5 秒超时

// 选择第一个完成的
let result = wait race tasks
```

---

## 节点

节点（Node）是 Auto 语言中用于描述树状结构的特殊语法，特别适合描述 UI、XML、配置等场景。

### 节点定义

```rust
// 节点定义，可以提前指定节点的属性对应的类型
node button {
    text str
    scale Scale
    onclick str
}
```

### 创建节点

```rust
// 新建节点，其 id 为 btn1
button btn1 {
    text: "Click me"
    scale: Scale.M
    onclick: "click:btn1"
}
```

### 多层节点

```rust
// 多层节点结构
ul {
    li {
        label {"Item 1"}
        button btn1 {
            text: "Click me"
            onclick: "click:btn1"
        }
        div { label {"div1"} }
    }
    li { label {"Item 2"} }
    li { label {"Item 3"} }
}
```

### 节点和 XML 的对应关系

节点语法和 XML 语法一一对应：

```rust
// Auto 节点语法
parent(k1: "v1", k2: "v2") {
    kid(k1: "v1") {
        // more kids
    }
    kid(k2: "v2") {
        // more kids
    }
}
```

等价于 XML：

```xml
<parent k1="v1" k2="v2">
    <kid k1="v1" />
    <kid k2="v2" />
</parent>
```

### 节点在 UI 中的应用

节点语法特别适合描述 UI 界面：

```rust
// 定义 UI 组件
widget Counter {
    model {
        var count: i32 = 0
    }

    view {
        col {
            button("➕") {
                on_click: || count += 1
            }
            text(f"Count: ${count}")
            button("➖") {
                on_click: || count -= 1
            }
        }
    }
}
```

### 节点在配置中的应用

节点语法也非常适合描述 XML 类型的配置文件：

```rust
// 表示一张成绩表
class(name: "三3班", count: 55) {
    student(name: "子涵", age: 18) {
        score(subject: "语文", score: 80) {}
        score(subject: "数学", score: 90) {}
        score(subject: "英语", score: 85) {}
    }
    student(name: "翌晨", age: 19) {
        score(subject: "语文", score: 85) {}
        score(subject: "数学", score: 95) {}
        score(subject: "英语", score: 80) {}
    }
}
```

对应的 XML：

```xml
<class name="三3班" count="55">
    <student name="子涵" age="18">
        <score subject="语文" score="80" />
        <score subject="数学" score="90" />
        <score subject="英语" score="85" />
    </student>
    <student name="翌晨" age="19">
        <score subject="语文" score="85" />
        <score subject="数学" score="95" />
        <score subject="英语" score="80" />
    </student>
</class>
```

**节点 + 编程 = 强大配置**：

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

这种基于节点的格式，非常适合用来描述 UI 等树状配置。和 XML、YAML 不同，AutoConfig 是可编程的，因此要更加灵活强大。

---

## 总结

Auto 语言提供了丰富的语法特性：

- **四种存量类型**：let（定量）、mut（变量）、const（常量）、var（幻量）
- **基本数据类型**：数值、数组、对象、Grid
- **强大的函数系统**：支持 Lambda、高阶函数
- **灵活的值传递**：拷贝、引用、转移、指针
- **丰富的控制流**：if、for、loop、is（模式匹配）
- **类型系统**：类型别名、类型组合、自定义类型、扩展方法
- **特征系统**：接口特征、表达式特征、判别函数特征
- **高级特性**（规划中）：枚举、生成器、异步、节点

通过这些特性，Auto 可以适应不同的应用场景，从嵌入式系统到脚本语言，从配置文件到 UI 框架。
