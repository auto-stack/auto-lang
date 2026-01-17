## ASTL (Auto Syntax Tree Language)

## 问题的来源

Auto语言的C转译器基本实现之后，我们发现需要做反过来的过程：将C解析并翻译成Auto。

这是因为Auto语言如果要调用C语言的函数（或其他Symbol），需要先用Auto语言声明一遍。

例如，C标准库`<string.h>`里的`memcpy`函数，定义如下：

```c
void *memcpy(void *dest, const void *src, size_t count);
```

我们需要写出对应的Auto语言的函数声明，才能在Auto语言里使用：

```auto
fn memcpy(dest mut *u8, src *u8, count usize)
```

现在这个过程是手动的，但是如果要实现完整的标准库，手动来写就不太合理了。

因此我们需要一个工具，能够自动扫描C语言的头文件，找到所有函数的声明，并把它们转换为Auto语言的声明。

这就需要一个基本完整的C->Auto的转译器了。

现在Auto->C的转译器的实现，有三个部分：

- Parser(Auto): 将Auto语言解析成Auto语言的语法树，即 `Code(Auto) -> AST(Auto)`
- Trans(Auto, C)：读取Auto语法树，翻译成C语言的代码并打印出来，即`AST(Auto) -> Code(C)`。

如果按照这个方式把C->Auto的转译器再重新实现一遍，需要三个部分：

1. AST(C)，由于当前的语法树是和语言本身深度绑定的，因此需要重新设计一套C的语法树。
2. Parser(C），实现解析流程：`Code(C) -> AST(C)`。
3. Trans(C, Auto)：读取C语言的语法树，翻译成Auto语言的代码并打印出来，即`AST(C) -> Code(Auto)`。

总结一下，如果需要实现(Auto, C)的互相转译，就需要实现6个组件：

- AST(Auto)
- Parser(Auto)
- Trans(Auto, C)
- AST(C)
- Parser(C)
- Trans(C, Auto)

在此基础上，如果要实现（Auto, RASTL)的互相转译，那也需要6个组件：

- AST(RASTL)
- Parser(RASTL)
- Trans(RASTL, Auto)
- AST(Auto)  // 已实现
- Parser(Auto) // 已实现
- Trans(Auto, RASTL)

这其中有4个组件是需要重新做的的，有2个组件是已经做好的。

此时我就滋生了一个想法，如果实现一套统一的语法树，兼容Auto和C语言，是不是转译就方便多了？
假设这个统一的语法树叫ASTL（Auto Syntax Tree Lanuage），那么每添加一个新的语言，我们只需要实现两个组件：

- Parser(NewLang)： 把NewLang语言解析为ASTL
- Codegen(NewLang)：用ASTL生成NewLang语言的代码

由于ASTL是统一的，此时任何已经实现的语言都可以和NewLang语言进行互相转译了。

这个就是典型的`MxN`到`M+N`的架构简化。

## ASTL语法树

ASTL的主要特性有：

1. 格式为Atom格式，Auto语言为它提供了专门的API，方便读写与转换。
2. 用Auto语言来定义ASTL的约束（Schema）。
3. 尽量做到所有支持语言的AST的并集。
4. 对每门语言，需要实现一个Parser和一个CodeGen：
  - Parser(A)：Code(A) -> ASTL 
  - CodeGen(A)：ASTL -> Code(A)
5. 对任意两门语言（A，B），只要实现了它们对ASTL的解析和生成组件，就能自动实现转译。
6. （扩展）Auto编译器可以根据Schema自动生成Parser。方法类似于Tree-Sitter。
7. 实际情况中，两门不同的语言虽然共用ASTL语法树，但是细节上会有区别，实际还是要做一些ASTL内部的调整转换的。
  - 但这个工作量，显然比实现多组Trans(A, B)转译组件要小很多。


## ASTL Languange

由于ASTL使用的Atom格式，因此它本身也可以看作是一个独立的编程语言，只不过相对于Auto/Python这样的编程语言来说，
相对啰嗦一些而已。

例如下面的C语言代码：

```c
#include <stdio.h>

int main() {
    printf("Hello, World!\n");
    return 0;
}
```

用标准的ASTL语言展示如下：

```ASTL
stmt {
  use {
    kind: "c-include" // due to include
    items: [
      item {
        name: "stdio.h"
        kind: "sys" // due to `<..>`
      }
    ]
  }
}

stmt {
  fn {
    name: "main"
    return: int
    args: []
  
    body {
      stmt {
        kind: call
        call {
          name: printf
          args: ["Hello, World!\n"]
        }
      }
      stmt {
        kind: return
        return {
          value: 0
        }
      }
    }
  }
}
```

如果用简化版的Atom格式，展示如下：

```ASTL
use.c {
  sys "stdio.h"
}

fn main int {
  call printf ("Hello, World!")
  ret 0
}
```

对应的Auto语言：

```auto
use c <stdio.h>

fn main int {
  print("Hello, World!")
  0
}
```

可以看出，简化版的Atom格式ASTL，和Auto语言已经比较类似了。
而与Auto语言不同，ASTL理论上可以支持大多数编程语言。

因此我们可以进一步强化ASTL的概念，
它不再仅仅是一个语法树（用来存放数据），
而是一门独立的编程语言。
这也是为什么ASTL而不是AST的原因。

此时我们发现，ASTL的结构和Lisp的S表达式实际上是相通的。
我们可以借鉴Lisp的设计，来优化ASTL的设计；
甚至可以借鉴Clojure等现代化Lisp方言的设计，给ASTL语言添加语法糖。
