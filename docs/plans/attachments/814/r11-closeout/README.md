# PLAN-814 R11 work 修复轮证据（R9-01 收口 + F814-R11-01）

来源：探针复制自 review-r11 冻结快照（复审员原件未动），在本目录用修复后
生成器（worktree `D:/autostack/.wt/lang-814/auto-lang` @ 本修复提交）再生
全部 `.at.rs` 并独立编译。修复基线（复现用）= R11 受审提交 `49206c6992`，
七项失败 feature 全部 exit101 复现后再修。

## 复现命令

```bash
cargo build --offline --locked --manifest-path <本目录>/Cargo.toml \
  --bin r10-generate --target-dir <原树>/target
<原树>/target/debug/r10-generate.exe <fixture>.at [--prop]   # prop-* 加 --prop
cargo check --offline --locked --manifest-path <本目录>/Cargo.toml \
  --target-dir <原树>/target --bin r10-check [--features <name>]
```

注：`Cargo.lock` 按 attachments/.gitignore 约定不入库——复现者可从
review-r11 冻结件复制同锁，或去掉 `--locked` 重新解析（依赖均为本地
path 指向原树四包，解析结果确定）。

## 回放收据

`comparison-checks.log`：无 feature 正控 + 26 feature 全 exit0（27/27）。
R9/R10 全部旧反例与正控保持（unknown-negative/float/rhs-float/neg-float
[-rhs]/integer-arithmetic/prop-negative/prop-float + 四无条件正控 +
string-control/bool-condition-control）。

## 覆盖矩阵（fixture → 维度）

| 维度 | fixture |
|---|---|
| 字面量 int（含负） | unknown-negative、unknown-integer-arithmetic、int-comparison |
| 字面量 float（含负、RHS 位） | unknown-float、unknown-rhs-float、unknown-neg-float、unknown-neg-float-rhs、numeric-silent |
| 字面量 bool / str | bool-comparison、bool-rhs、unknown-bool、unknown-positive、string-control、bool-condition-control |
| 声明 int（loop 左/右位、prop） | unknown-int-state、unknown-int-state-rhs、prop-int-state |
| 声明 bool（loop 左/右位、prop） | unknown-bool-state、unknown-bool-state-rhs、prop-bool-state |
| 声明 float=f32（loop、prop、分组左位） | unknown-float-state、prop-float-state、paren-state-lhs |
| 声明 str（正控不变） | unknown-string-state |
| 声明宽度 i64 | width-i64-state |
| 分组×字面量 float/int/bool（左/右位） | unknown-paren-float、paren-rhs-float、paren-int、paren-bool |
| 分组×声明（loop、prop、左/右位） | paren-state、paren-state-lhs、prop-paren-state |
| 分组×prop×float | prop-paren-float |
| prop 基线 | prop-positive、prop-bool、prop-float、prop-negative |

投影形态（loop 形；prop 形为 `self.<prop>["<field>"]` 同构）：
int→`(p["f"].as_i64().unwrap_or(0) as i32)`；float→`as_f64()`；
声明 f32→`as_f64() as f32`；声明 u32/i64/u64→`as_i64() as <宽度>`；
bool→`as_bool()`；默认/str→`as_str()`。分组括号原样保留（透明）。
