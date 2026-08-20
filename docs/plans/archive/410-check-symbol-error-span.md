# Plan 410: check_symbol 未定义变量报错的 span 修正（调用点 span 提示）

> **状态**: ✅ Phase 1 完成（方案 A：调用点 span 提示）。📋 Phase 2（方案 B：系统性表达式 span）留作后续，触发条件见 §4。
> **前置**: jade-garden gap 清单（auto-down `plans/012`）"报错定位"系列——gap 17/18/29/37a/37b/46 已在 `error_spans_tests.rs` 配测试，本计划是该清单的最后一块残留。
> **仓库**: **auto-lang**（`crates/auto-lang/src/parser.rs` + `error_spans_tests.rs`）。
> **目标**: `NameError::undefined_variable` 的 span 指向表达式起始位置（即冒犯标识符），而非解析器游标当前位置（表达式之后的终结符）。

---

## 1. 问题陈述

`Parser::check_symbol`（`crates/auto-lang/src/parser.rs:10439`）负责未定义变量检查，内部 2 处产生 `NameError::undefined_variable`：

1. `Expr::Bina(l, Op::Dot, _)` —— 点表达式最左侧标识符未定义（`:10464`）；
2. `Expr::Ident(name)` —— 简单名称未定义（`:10480`）。

（第 3 处 Call 分支的函数名检查历史上被注释停用，见 `:10500` 附近的 TODO，不产生报错。）

两处报错的 span 都是 `pos_to_span(self.cur.pos)`——即 **check_symbol 被调用时解析器游标的位置**。但 check_symbol 的 4 个调用点（`:1734`、`:1740`、`:11174`、`:11198`）全部是**先 `expr_pratt(...)` 解析完整表达式、再 check_symbol**，此时游标已越过整个表达式、停在表达式后的终结符（换行/`;`/`}`）上。

### 具体示例

```auto
fn main() {
    var y = missingVar + 1
}
```

`parse_expr`（`:1740` 调用点）先以 `expr_pratt(0)` 解析完 `missingVar + 1`，游标停在行尾换行符上；随后 `check_symbol` 发现 `missingVar` 未定义，却把 span 定在**换行符位置**（第 3 行行尾），而不是 `missingVar`（第 3 行第 14 列）。jade-garden 编辑器据此标红时，波浪线落在语句收尾而非冒犯标识符上——这正是 gap 清单"报错定位"的残留项。

## 2. 现状先例

表达式节点带可选位置信息是项目既有惯例，并非新发明：

- `Call.pos: Option<Pos>`（`crates/auto-lang/src/ast/call.rs:21`，`fn call` 在 `:11198` 附近填充）；
- `Fn.span: Option<(usize, usize)>`；
- UI 节点（`ViewNode` 等）普遍带 span。

## 3. 方案分析

### 方案 A：调用点 span 提示（本计划实施）

在每个 check_symbol 调用点，`expr_pratt(...)` 之前记录表达式起始位置（`let start_pos = self.cur.pos`，或标识符刚被消费后的 `self.prev.pos`），把 `pos_to_span(start_pos)` 作为参数传入 check_symbol，替代内部的 `pos_to_span(self.cur.pos)`。

- **churn**：check_symbol 签名加 1 个参数 + 4 个调用点各加 1-2 行。无 AST 改动。
- **精度**：表达式起始位置。对常见形态——裸标识符（`missingVar`）、点表达式（`missing.x`）、调用（`missing(...)`）——表达式起始 token 就是冒犯标识符本身（check_symbol 只检查最左侧标识符），精度等于标识符位置。对 `1 + missing` 这类标识符不在开头的形态，span 落在表达式开头而非标识符上——可接受的近似（现状是落在表达式**之后**，更差）。

### 方案 B：系统性表达式 span

给 `Expr` 全面带 span：要么引入 `Spanned<Expr>` wrapper，要么仿 `Call.pos` 先例给关键 variant（`Ident`、`Bina`、`Call` 等）加 `Option<Pos>`，check_symbol 直接从节点取冒犯标识符的精确位置。

- **churn 规模**：`Expr` 枚举在 `crates/auto-lang/src/ast.rs:318`，全库约 **3559 处 `Expr::` 使用**。wrapper 方案所有构造/匹配点都要过一遍；variant 加字段方案虽可靠 `Option` 默认 `None` 减轻，但每个构造点仍需显式填 `pos: None`（或引入 builder），parser/代码生成/VM 多 crate 连锁改动。
- **收益**：任意嵌套位置（如 `1 + missing` 右操作数）也能精确定位；为后续所有表达式级报错提供统一位置基础设施。
- **风险**：大 diff 引入回归的概率高；且 check_symbol 目前只查最左标识符，Phase 1 已覆盖其全部实际报错路径，Phase 2 的增量收益要等"检查更深层表达式"的需求出现才兑现。

### 对比

| 维度 | A：调用点提示 | B：系统性 span |
|---|---|---|
| 改动量 | ~10 行，1 个文件 | 数百处，跨 crate |
| 常见形态精度 | = 标识符位置 | = 标识符位置 |
| 非常见形态精度 | 表达式开头 | 任意嵌套位置 |
| 回归风险 | 极低 | 高 |
| 附带收益 | 无 | 统一表达式位置基础设施 |

## 4. 推荐路线

- **Phase 1（本计划实施）= 方案 A**。覆盖 check_symbol 全部实际报错路径，成本极低。
- **Phase 2（留作后续，不实施）= 方案 B**。触发条件（任一满足再立项）：
  1. check_symbol 或类型检查扩展到检查**非最左**标识符（嵌套表达式内部的未定义名），方案 A 的近似不再够用；
  2. 编辑器侧（jade-garden）对表达式级精确标红有进一步需求（如 hover/跳转需要每个标识符的位置）；
  3. 其他报错类别也需要表达式内部位置，累积到值得一次性建基础设施。
  届时优先评估 `Spanned<Expr>` wrapper（语义最干净），以 `Call.pos` 式局部 `Option<Pos>` 作为降级路径。

## 5. Phase 1 实施记录（2026-08-11）

- `check_symbol` 签名改为 `check_symbol(&mut self, expr: Expr, err_span: SourceSpan)`（全库仅 parser.rs 内 4 个调用点，直接改签名比保留旧入口转调更干净）。
- 4 个调用点：
  - `:1734`（`move` 闭包前缀）、`:1740`（`parse_expr` 主路径）：`expr_pratt` 前记 `let start_pos = self.cur.pos`；
  - `:11174`（`node_or_call_expr`）：标识符被消费后记 `let start_pos = self.prev.pos`（prev = 表达式首个 token）；
  - `:11198`（`fn call`）：Call 分支不产生 undefined_variable 报错，保持传入 `pos_to_span(self.cur.pos)` 原行为，不为零收益改动 `fn call` 签名（3 个上游调用点）。
- check_symbol 内 2 处 `NameError::undefined_variable` 的 span 改用传入的 `err_span`；`skip_check`/`Config` 早退分支不受影响。
- 测试：`error_spans_tests.rs` 新增 2 个防回退测试——`if missingVar {`（Ident 形态，`parse_expr` 主路径）与 `log(missingVar)`（调用参数位置，`args()` → `parse_expr` 嵌套路径），断言 label offset = 标识符 offset。
- **调研修正**：check_symbol 的 `Bina(Op::Dot)` 分支（点表达式左标识符）在当前解析器下**从源码不可达**——字段访问 `a.b` 在 `expr_pratt_with_left` 中归约为专门的 `Expr::Dot` variant（仅 `expr!` collect 脱糖产生 `Bina(Op::Dot)`，且外层包 Call），而 `Expr::Dot` 不经 check_symbol 检查（`x = a.b` 中 `a` 未定义今天也能解析通过）。该分支的 span 已防御性切换为传入 hint，但无法配源码级测试；把 `Expr::Dot` 纳入检查是语义变更，超出本计划范围，如需应单独立项（可并入 Phase 2 评估）。
