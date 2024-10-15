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
views: "${root}/views"
styles: "${dir}/styles"

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

arg(heap, "heap_5")
arg(config_inc, "demo")

// Dependencies
dep(FreeRTOS, "v0.0.3") {
    heap: heap
    config_inc: config_inc
}

// Libs
lib(osal) {
    pac(hsm) {
        skip: ["hsm_test.h", "hsm_test.c"]
    }
    pac(log)
    link: FreeRTOS
}

// Ports
port(windows, cmake, x64, win32, "v1.0.0")

// Executables
exe(demo) {
    link: osal
}
```

### 3. 作为脚本

```rust
#!auto

// 脚本模式下内置了常用的库
print "Hello, world!"

mkdir src
cd src
touch main.rs

// 也可以定义变量和函数
var ext = ".c"
fn find_c_files(dir) {
    let ret = for file in ls(dir) {
        if (file.endswith(ext)) {
            yield file
        }
    }
    ret.sort()
}

// 可以循环调用
for f in find_c_files(".") {
    rename f ".c" ".cc"
}
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


## 使用与安装

Auto语言编译器本身只依赖于Rust和Cargo。

```bash
> git clone git@gitee.com:auto-stack/auto-lang.git
> cd auto-lang
> cargo run
```
