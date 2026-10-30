# auto-a2r 标量 HIR 后端合同

> 状态：active（AX-A；PLAN-892 §5.1 落地形态）。

## 域准入（视图二道门，前端之外）

| profile | 运算/循环 | 拒绝 |
|---|---|---|
| core（`auto.source.core-i32.draft`） | `add/mul/lt`、signed 字面量、if/while/loop | 其余全部（含 neg/unary/短路/比较扩面） |
| scalar（`auto.source.scalar-i32.draft`） | ＋`sub/div/rem`、`le/gt/ge`、同类 `eq/ne`、`neg/not`、lazy `&&/||`、条件 `for` | `while` 具名拒绝 |

准入失败=具名 `A2xReject`（域/表达式/语句/类型/place/算子/未授权 intrinsic/
入口），**emit 前拒**；静态非法源在前端诊断轴验证，不用"Rust 编译拒绝"证明
Auto 检查（Rust 编译失败只能=infra）。

## 降阶规则（lower）

- 全值表达式 `Result` 化：checked 助手 `a2x_add/sub/mul/neg/div/rem`
  （fallible 根经 `?` 传播）；比较/非/短路渲染为纯值（fallible 子式内联 `?`）。
- **短路**：`(if lhs { rhs } else { false })` / `(if lhs { true } else { rhs })`
  ——RHS 临时钉在分支内（不执行零求值，V-04 实证：未取 RHS 零标记）。
- **调用**：`eval_args` 源序求值各一次（fallible/trace 根钉临时），后按
  `bindings` 落位（具名反序参数实证：标记序 `[mark_b, mark_a]`＝源序而非落位序）。
- **循环**：`Loop`→`loop`；break/continue 断言最近环目标（HIR 显式 LoopId 与
  栈顶一致，否则 `a2x.internal` 拒绝）；while/条件 for 由 HIR 的 Loop+If+Break
  结构直译（HIR 形状探针钉住）。
- **命名卫生**：函数 `fn_<源名>`、局部 `l{idx}_<名>`、临时 `t{n}`、runtime
  保留 `a2x_` 前缀——用户 `ExitProcess`/`sf_ac_start`/用户 `mark_a` 均不劫持
  （语料实证）。
- **无尾 unreachable**：全路径 return 由 rustc 独立复核（缺失=降阶 bug 编译报错）。

## trap 语义（冻结）

add/sub/mul/neg 越界→`Overflow`；除零→`DivByZero`；MIN/-1→`DivOverflow`；
MIN%-1→`RemOverflow`；%-零→`RemByZero`。`-7/3=-2`、`-7%3=-1`（向零截断/
随被除数）。未执行分支的常量陷阱不提升为诊断（`if false { MAX+1 }` 仍是值）。
