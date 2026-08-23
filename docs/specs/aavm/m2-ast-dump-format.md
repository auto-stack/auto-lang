# M2 闸门目标格式:Rust AST S-expression 清单(plan-432 S2 前置考古)

基线 `b3bd64f5`;来源 = 各 Display impl(文件:行)。S2 的 AAVM parser
必须逐字复刻这些格式(M2 diff=0 的唯一依据)。

## 顶层与语句

| 构造 | 格式 | 来源 |
|---|---|---|
| Program | `(code s1 s2 ...)`(空格分隔,无尾空格) | ast.rs:169 |
| 表达式语句 | 直接内嵌 expr | ast.rs Stmt::Expr |
| let/var | `(let (name x) e)` / `(var (name x) e)`;带类型 `(let (name x) (type T) e)` | ast/store.rs:39-40 |
| 赋值 | `Stmt::Expr(Bina(Asn))` → `(bina l (op =) r)` | ast.rs:484 |
| fn 声明 | `(fn (name main) (params p1 p2) (ret T)? body)`;params/ret 省略当空/Unknown | ast/fun.rs:96 |
| Param | `(param (name n) (type T) (mode mut))`;mode ∈ view/mut/move/take/copy | ast/fun.rs:280 |
| return | `(return e)`;break/continue → `(break)`/`(continue)` | ast.rs |
| if | `(if branch1 branch2 (else body)?)` | ast/if_.rs:11 |
| Body/块 | `(body s1 s2 ...)` | ast/body.rs:52 |

## 表达式(ast.rs:461 起)

| 构造 | 格式 |
|---|---|
| Int/Uint/I8/U8/Byte | `(int 1)` `(uint 1)` `(i8 1)` `(u8 1)` `(byte 1)` |
| Float/Double | `(float 2.5)` `(double 1.0)`(值 Display,非原文本) |
| Bool | `(true)`/`(false)` |
| Char | `(char 'x')` |
| Str | `(str "hello")` |
| Ident | `(name x)` |
| Bina | `(bina l (op +) r)` |
| Unary | `(una (op !) e)` |
| Call | `(call (args a1 a2))`;Arg: Pos=直接 / `(name n)` / `(pair (name n) e)`(ast/call.rs:190-207) |
| Array | `(array e1 e2)`;Tuple `(tuple e1 e2)`;Index `(index a i)` |
| Dot | `(dot obj.field)` |
| nil/null | `(nil)`/`(null)` |

## Op(auto-val/src/value.rs:815)

全部形如 `(op X)`:+ - * / % += -= *= /= %= ! && || = == != < > <= >= .. ..
= . ?? ?. .? in .view .mut .move .take(注意 `(op ()`/`(op {)` 的不平衡括号 quirk)。

## S2 实施决策(已定)

1. **D20**:parser 直出 S-expression dump(`parse_dump(source) -> str`),
   与 S1 的 D14 同构——M2 判据层;真实 AST 结构推迟到 S4 codegen 需要时定
   (届时若 Auto 无数据和枚举,以 per-kind type + kind 字段的 discriminated union)。
2. **D21**:token 流承载 = `List.new()/push/get/len`(VM 侧 auto.list.* natives
   可装结构体——v1 已证;a2r 侧 List→Vec 直译,trans/rust.rs:1888 已有映射)。
   divergence-rules C3 的禁用清单据此修正:禁 auto.*/Result.*/Map.* 便利层,
   List.* 为 sanctioned(转译干净)。
3. parser 状态:`type P { toks List, pos int }`,字段变异语义 v1 parser.at 已证。
4. Pratt 优先级表:自基线 parser.rs 的 prefix_power/postfix_power/infix_power 直译
   (S2 开工时提取具体数值)。
5. M2 语料:复用 99_bootstrap 009-021 的嵌入式 source 串 + corpus_m1 同源。

## S2 未决(开工首日定)

- Newline 跳过策略(parser 侧 skip_empty_lines 语义);
- if 分支 Branch 结构的 Display(branches 元素格式,ast.rs Branch);
- fn 嵌套/前向引用(单遍定义序)。
