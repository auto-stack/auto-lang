# auto-a2c 标量 HIR→C11 后端合同

> 依据：PLAN-921 §5.1（CA-A）；实现 `a2x-c/src/{lower,runtime,provider}.rs`

## C 整数语义（无 UB）

- **checked 运算**：`a2x_add/sub/mul/neg` 在 `int64_t` 中间量计算并
  范围检查**后**收窄（C 有符号溢出永不执行）；`a2x_div/rem` 先排零
  （DivByZero/RemByZero）再排 `INT32_MIN/-1`（DivOverflow/RemOverflow）；
  C 截断语义保 `-7/3==-2`、`-7%3==-1`；陷阱分类与 Rust 运行时同名
  （Overflow/DivByZero/DivOverflow/RemByZero/RemOverflow）。
- **字面量**：`INT32_MIN` 以常量表达式 `(int32_t)(-2147483647 - 1)`
  表达（不经过更宽类型的字面量往返）。
- **宿主边界**：main 包装器 `_setmode(_fileno(stdout/stderr), _O_BINARY)`
  后输出 `value:%<PRId32>` / `trap:%s`——与 Rust 通道字节协议一致
  （无 CRLF 翻译）；marks 于触发时刻 stderr 顺序输出（trap 前的 marks
  保持可观测）。

## 源序、单次求值与短路

- **调用参数**：`eval_args` 源序逐参 pin 为临时量（效果先于落位、单次
  求值），按 `bindings` 落位后调用——**不赌 C 函数参数求值顺序**。
- **短路**：`and/or` 的 RHS 语句全部位于守卫分支内（`if (t)`/`if (!t)`，
  t 为物化临时量），未取侧永不执行、永不提升陷阱。
- **if 条件物化**：条件先物化为 bool 临时量（源级常量条件如
  `while false` 不构成 C4127 的语法常量控制表达式）。

## 控制流与卫生

- `for (;;)` + C `break`/`continue`（HIR 保证目标=最近循环，非最近
  拒绝）；全部路径 return（verifier 信任，等同 Rust 目标；掉落路径由
  /W4 /WX 的 C4715 响亮失败而非静默值）。
- 命名：`fn_`/`mt_`/`lN_` 前缀 + ASCII 净化（`fn_`/`mt_` 前缀保证非
  C 关键字；无前导下划线保留名）；局部量函数顶声明、let 为初始化
  赋值。
- 生成码 /W4 /WX 零警告（`-O2 -MT`）。

## 文件集与确定性

- 3 文件：`main.c`（包装器）/ `a2x_c_runtime.h`（版本化 runtime 文本，
  marks 参数化，单一 static inline 头，内容 hash 进片段键）/
  `generated.c`（前向声明块=扫描各片段原型行——与 rt prelude 扫描
  文本同模式；随后按 def 序定义）。
- manifest（`auto.a2x.c.fileset/1`）：schema/input_hash/entry/functions
  （含 origins：冷视图派生或恢复时锚携带）/files+sha256——双新鲜会话
  字节同一（测试断言）。
