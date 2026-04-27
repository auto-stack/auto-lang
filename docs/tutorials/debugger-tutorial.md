# Auto Debugger 交互式教程

本教程通过一个 Fibonacci 程序，演示 `auto debug` 的完整使用流程。

## 准备示例程序

创建 `fibonacci.at`：

```auto
fn fibonacci(n int) int {
    if n <= 1 {
        return n
    }
    var a = 0
    var b = 1
    for i in 2..=n {
        var temp = b
        b = a + b
        a = temp
    }
    return b
}

var result = fibonacci(7)
print(result)
```

## 启动调试器

```bash
auto debug fibonacci.at
```

输出：

```
----------------------
Debugging Auto fibonacci.at
----------------------

--- Paused at ip=0000 | FN_PROLOG ---
(auto-dbg)
```

调试器在程序第一条指令处自动暂停，等待你的命令。

## 第一组命令：查看帮助和源码

### `help` — 查看所有可用命令

```
(auto-dbg) help
GDB-like commands:
  run (r)                Start / continue execution
  continue (c)           Continue to next breakpoint
  step (s)               Step into (one instruction)
  next (n)               Step over (next source line)
  ...
```

### `list` — 查看当前源码上下文

```
(auto-dbg) list
```

显示当前暂停位置前后各 5 行源码，当前行用 `>` 标记。

## 设置断点

### `break <行号>` — 在指定行设断点

```
(auto-dbg) break 8
Breakpoint 0 at line 8
```

第 8 行是循环体 `var temp = b`，程序执行到这里会暂停。

### `break <函数名>` — 在函数入口设断点

```
(auto-dbg) break fibonacci
Breakpoint 1 at function fibonacci
```

程序执行到 `fibonacci` 函数被调用时暂停。

### `break <函数名>/<偏移>` — 在函数内指定行设断点

```
(auto-dbg) break fibonacci/3
Breakpoint 2 at line 5 (fibonacci + 3)
```

`fibonacci/3` 表示 fibonacci 函数入口往下第 3 行。如果函数定义在第 2 行，则断点在第 5 行（`var a = 0`）。

### 不支持的语法

```
(auto-dbg) break main:1
Error: multi-file breakpoints not yet supported.
  Use: b <line> or b <function> or b <function/N>

(auto-dbg) break nonexistent
Error: function 'nonexistent' not found.
  Available: fibonacci
```

- `file:line` 语法（多文件断点）暂不支持，会明确报错
- 函数名不存在时会列出所有可用的函数名

### `info breakpoints` — 查看所有断点

```
(auto-dbg) info breakpoints
  #0 at line 8
  #1 at function fibonacci
  #2 at line 5
```

## 运行和暂停

### `run` — 开始执行到断点

```
(auto-dbg) run

--- Paused at line 8 | ip=004b | LOAD_LOC_1 ---
          var temp = b
```

程序运行到第 8 行的断点处暂停。输出包含：
- **line 8** — 当前源码行号
- **ip=004b** — 当前指令指针（字节码偏移）
- **LOAD_LOC_1** — 当前要执行的 opcode
- 源码文本 — 对应行的内容

## 检查程序状态

### `info locals` — 查看局部变量

```
(auto-dbg) info locals
Locals (4 slots from bp+1):
  [0] = 0
  [1] = 1
  [2] = 2
  [3] = 0
```

变量按 slot 索引显示（编译器分配的局部变量槽位）。

### `info stack` — 查看调用栈

```
(auto-dbg) info stack
Call stack:
  #0 fibonacci at line 8
```

### `info registers` — 查看寄存器

```
(auto-dbg) info registers
  IP  = 004b (75)
  BP  = 0013 (19)
  SP  = 001f (31)
  Line = 8
```

- **IP** — Instruction Pointer，下一条要执行的指令地址
- **BP** — Base Pointer，当前函数帧的基址
- **SP** — Stack Pointer，栈顶位置

## 单步执行

### `next` — 单步跳过（下一行源码）

```
(auto-dbg) next

--- Paused at line 9 | ip=0051 | LOAD_LOC_0 ---
          b = a + b
```

执行到第 9 行。再次查看变量：

```
(auto-dbg) info locals
Locals (4 slots from bp+1):
  [0] = 0
  [1] = 1
  [2] = 2
  [3] = 1
```

继续 step：

```
(auto-dbg) next

--- Paused at line 10 | ip=005a | LOAD_LOCAL ---
          a = temp

(auto-dbg) info locals
Locals (4 slots from bp+1):
  [0] = 0
  [1] = 1
  [2] = 3
  [3] = 1
```

可以看到 `b`（slot 1）从 1 变为 1（a+b=0+1），但 slot 2 变成了 3（这是 `i` 的变化）。

### `step` — 单步进入（逐条指令）

`step` 每次只执行一条 VM 指令，适合精确定位字节码级别的问题。

### `finish` — 执行到当前函数返回

```
(auto-dbg) finish

--- Paused at line 12 | ip=007c | STORE_LOC_0 ---
      return b
```

跳过循环体的剩余迭代，直接到 `return b` 处暂停。

### `until <行号>` — 执行到指定行

```
(auto-dbg) until 12
```

运行到第 12 行后暂停。等价于临时断点。

## 查看反汇编

### `disassemble` — 反汇编附近的字节码

```
(auto-dbg) disassemble
  0036  .line        5
  0039  const.i32    0
  003e  store.local  0
  0041  .line        6
  0044  const.i32    1
  0049  store.local  1
> 004c  .line        8
  004f  load.local   1
  0052  store.local  3
  0055  .line        9
```

`>` 标记当前 IP 位置。每行显示：偏移地址、助记符、操作数、行号注释。

## 删除断点

### `delete <编号>` — 删除指定断点

```
(auto-dbg) delete 1
Deleted breakpoint 1
```

断点编号从 `info breakpoints` 中查看。

## 退出调试器

### `quit` — 退出

```
(auto-dbg) quit
Exiting debugger.
```

## 完整调试会话示例

下面是一个典型的调试流程——跟踪 Fibonacci 循环的变量变化：

```
$ auto debug fibonacci.at
----------------------
Debugging Auto fibonacci.at
----------------------

--- Paused at ip=0000 | FN_PROLOG ---
(auto-dbg) b 8                    ← 在循环体设断点
Breakpoint 0 at line 8
(auto-dbg) r                      ← 运行到断点

--- Paused at line 8 | ip=004b | LOAD_LOC_1 ---
          var temp = b
(auto-dbg) i l                    ← 查看变量（a=0, b=1）
Locals (4 slots from bp+1):
  [0] = 0
  [1] = 1
  [2] = 2
  [3] = 0

(auto-dbg) n                      ← 单步到下一行

--- Paused at line 9 | ip=0051 | LOAD_LOC_0 ---
          b = a + b
(auto-dbg) n                      ← 再单步

--- Paused at line 10 | ip=005a | LOAD_LOCAL ---
          a = temp
(auto-dbg) i l                    ← 查看变量变化
Locals (4 slots from bp+1):
  [0] = 0
  [1] = 1
  [2] = 3
  [3] = 1

(auto-dbg) c                      ← 继续运行到下一次循环

--- Paused at line 8 | ip=004b | LOAD_LOC_1 ---
          var temp = b
(auto-dbg) i l                    ← 第二次迭代（a=1, b=1）
Locals (4 slots from bp+1):
  [0] = 1
  [1] = 1
  [2] = 3
  [3] = 1

(auto-dbg) fin                    ← 跳到函数返回

--- Paused at line 12 | ip=007c | STORE_LOC_0 ---
      return b
(auto-dbg) i l                    ← 最终结果
Locals (4 slots from bp+1):
  [0] = 8
  [1] = 13
  [2] = 9
  [3] = 8

(auto-dbg) c                      ← 继续执行
13                                 ← fibonacci(7) = 13

(auto-dbg) q                      ← 退出
Exiting debugger.
```

## 命令速查表

| 命令 | 缩写 | 说明 |
|------|------|------|
| `run` | `r` | 开始/继续执行 |
| `continue` | `c` | 继续到下一个断点 |
| `step` | `s` | 单步进入（逐条指令） |
| `next` | `n` | 单步跳过（逐行源码） |
| `finish` | `fin` | 执行到函数返回 |
| `until <line>` | `u` | 执行到指定行号 |
| `break <line\|fn\|fn/N>` | `b` | 设置断点（行号、函数名、或函数+偏移） |
| `delete <n>` | `d` | 删除断点 |
| `info breakpoints` | `i b` | 列出断点 |
| `info stack` | `i s` | 显示调用栈 |
| `info locals` | `i l` | 显示局部变量 |
| `info registers` | `i r` | 显示 IP/BP/SP |
| `list` | `l` | 显示源码上下文 |
| `disassemble` | `disas` | 反汇编字节码 |
| `print <slot>` | `p` | 打印指定 slot 变量 |
| `quit` | `q` | 退出调试器 |
| `help` | `h` | 显示帮助 |
