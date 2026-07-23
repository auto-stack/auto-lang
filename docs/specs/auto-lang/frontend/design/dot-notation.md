# 统一点表示法（Dot Notation）

## 范围

所有后缀操作收敛到 `.` 之下的设计：字段访问、属性符号（`.?`/`.*`/`.@`）、
属性关键字（`.view`/`.mut`/`.move`/`.as`/`.to`）。设计哲学：*每个点是一步数据精炼
或权限变更*（docs/design/10）。

## 原则

- 消除前缀运算符与嵌套括号，后缀链一律走 `.`。
- 视觉规则：属性链点两侧无空格（`ptr.*.as.f32`）；二元运算必须有空格（`a * b`）。
- 函数式链式调用同样走点链（`.map().filter()!`），不引入 `|>` pipe（docs/design/10
  §Auto Flow 的 dot-vs-pipe 决策）。

## 现状（代码核对 2026-07，以代码为准）

| 语法 | AST | 状态 |
|---|---|---|
| `.field` 字段访问 | `Expr::Dot(Box<Expr>, Name)` | 已实现（plan-056） |
| `.view` / `.mut` | `Expr::View` / `Expr::Mut` | 已实现：lexer 直接产出复合 token `TokenKind::DotView`/`DotMut`（lexer.rs:701 起），parser 作无右值后缀运算符（parser.rs:2172 起） |
| `.move`（所有权转移） | `Expr::Move` | 已实现（plan-122 由 `.take` 更名） |
| `.take` | 同 `.move` | 已废弃：parser 发 `DeprecatedFeature` 警告后按 `.move` 处理（parser.rs:2187-2193） |
| `.as(Type)` | `Expr::Cast` | 已实现（plan-162，parser.rs:2243-2251；零开销重解释） |
| `.to(Type)` | `Expr::To` | 已实现（plan-162，parser.rs:2253-2260；可分配的显式转换） |
| `expr.?` / `?.(default)` | `Expr::ErrorPropagate` 等 | 已实现（`Op::DotQuestion` 路径，parser.rs:2197 起） |
| `.*` / `.@` | — | 未实现（lexer/parser 均无对应 token） |
| `.fixed` / `.dynamic` | — | 未实现 |

**分歧记录**：docs/design/10 §Status 称 `.?`/`.*`/`.@`/`.as`/`.view` 等"not yet in the
parser"——与代码不符。`.view`/`.mut`/`.move`/`.as`/`.to` 及 `?` 系均已实现；仅 `.*`、
`.@`、`.fixed`/`.dynamic` 未实现。design/10 该节已过时。plan-162 文件头标注"待实现"，
实际已落地，同为过时标注。

## 细节

- 复合 token 路线：`.view` 等不是"`.`+关键字"两个 token 在 parser 组合，而是 lexer
  直接产出 `DotView`/`DotMut`/`DotMove`/`DotTake`/`DotQuestion` 单 token
  （token.rs:163 附近），parser 在二元表达式循环中按 `Op` 派发。
- `.as`/`.to` 走另一条路：`Op::Dot` 后探 `TokenKind::As`/`To`，再解析括号内目标类型。
- 视觉一致性由 lexer 约定支撑：`.view ` 带尾随空格进 token 文本（lexer.rs:701），
  属性链无空格、二元运算有空格的规则在 token 层即固定。

## 显式非目标

- 位操作方法（`shl`/`ror` 等）与位域视图（`bits()`/`bit()`）不属于本主题——
  docs/design/10 将其列为未实现的独立层。
- Auto Flow 的 `Iter<T>` spec 与惰性适配器未实现，点链决策目前只有设计依据。
- 不在表达式位置复用语句级 `.?` 解包语义之外的 Option/Result 语法糖。

> 来源: docs/design/10-language-syntax.md §Unified Dot Notation；代码核对 lexer.rs:701、token.rs、parser.rs:2160-2260、ast.rs（Expr::View/Mut/Move/Cast/To/ErrorPropagate）
