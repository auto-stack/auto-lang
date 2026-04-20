# C

问题1：如何在2C模式下，调用C语言函数？


三个要素：

1. `#include`
2. 函数调用方式
3. 参数和结果是否需要转换？

解决方案：

1. `#include`：引入新的`use`语法：

```rust
use c <stdio.h>
```

2. 函数调用方式，auto的函数调用应当与C兼容

```rust
let x = 10
sqrt(x)
```

相当于C的：

```c
int x = 10;
sqrt(x);
``` 

3. 类型转换：
    - 在2C模式下，auto语言的标准类型会转换为C的相对类型
    - 在2C模式下，编译器添加几个C独有的类型

- auto标准类型：
    - int: uint32_t
    - byte: uint8_t
    - i8: int8_t
    - i16: int16_t
    - i32: int32_t
    - i64: int64_t
    - u8: uint8_t
    - u16: uint16_t
    - u32: uint32_t
    - u64: uint64_t
    - f32: float
    - f64: double
    - bool: 使用<stdbool.h>定义的bool类型
    - void: void

- C独有的类型：
    - char: char
    - cstr: char*

注意：Auto语言的`str`类型与C语言的`char*`不同，
前者包含长度信息的`slice`，后者则是指向一个以`\0`结尾的字符串的指针。

也就是说，用C语言的结构体来模拟Auto语言的`str`类型的话，应该是类似这样：

```c
typedef struct {
    char* data;
    int len;
} str;
```

使用cstr时需要手动转换：

```rust
let s = "hello"  // str类型
let c = s.cstr() // 转换成cstr
```

在2C模式下，编译器支持cstr的字面量：

```rust
let s = "hello"c
```

这里`s`的类型是`cstr`。


### 假想示例

调用C语言`<math.h>`中的`sqrt`函数。

纯C版本的实现如下：

```c
#include <stdio.h>
#include <math.h>

int main() {
    double x = 4.0;
    double y = sqrt(x);
    printf("The square root of %f is %f\n", x, y);
    return 0;
}
```

对应的Auto版本的实现如下：

```rust
use c <math.h>

fn main {
    let x : f64 = 4.0
    let y = sqrt(x)
    print(`The square root of ${x} is ${y}`)
}
```
