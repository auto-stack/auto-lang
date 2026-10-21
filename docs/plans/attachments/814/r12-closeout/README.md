# PLAN-814 R12 work 修复轮证据（R9-01 终局收口）

来源：探针复制自 review-r12 冻结快照（复审员原件未动），在本目录用
修复后生成器（worktree `D:/autostack/.wt/lang-814/auto-lang` @ `feae28e0f`）
再生全部 `.at.rs` 并独立编译。修复基线（复现用）= R12 受审提交
`1bbf98620`：8 败/28 过逐项复现后再修。

## 复现命令

```bash
cargo build --offline --locked --manifest-path <本目录>/Cargo.toml \
  --bin r10-generate --bin r10-overlay --target-dir <原树>/target
python <本目录>/replay.py     # 生成+逐 feature check+overlay，保留真实退出码
```

注：`Cargo.lock` 按 attachments/.gitignore 约定不入库——复现者可从
review-r12 冻结件复制同锁，或去掉 `--locked` 重新解析（依赖均为本地
path 指向原树四包）。

## 回放收据

`replay-summary.log`（replay.py 过滤视图；完整逐项 results.json 由
replay.py 生成）：**38 feature + 无 feature 正控 + overlay = 40/40
exit0**。R12 八败全修（group-field-int/-rhs、group-field-float/-rhs、
group-both-int、prop-group-field、float-arithmetic-lhs、
prop-float-arithmetic）+ R9/R10/R11 全反例正控保持。

## 覆盖矩阵（fixture → 维度）

| 维度 | fixture |
|---|---|
| 字面量 int/float/bool/str（含负、左右位、prop、整数算式） | unknown-negative/float/rhs-float/neg-float[-rhs]/integer-arithmetic/prop-negative/prop-float、bool-comparison/rhs、int-comparison、string-control、unknown-positive、numeric-silent、bool-condition-control、unknown-bool、unknown-rhs-negative |
| 声明值 int/bool/float=f32/str/i64（左右位、prop） | unknown-int-state[-rhs]/prop-int-state、unknown-bool-state[-rhs]/prop-bool-state、unknown-float-state/prop-float-state、paren-state-lhs、unknown-string-state、width-i64-state |
| 另一侧分组 | unknown-paren-float、paren-rhs-float、paren-int、paren-bool、paren-state、prop-paren-state、prop-paren-float |
| **字段自身分组**（R12） | group-field-int[-rhs]、group-field-float[-rhs]、group-field-bool、group-both-int（双侧嵌套）、prop-group-field |
| **已支持算式**（R12） | float-arithmetic-lhs、float-arithmetic-rhs（对称正控）、prop-float-arithmetic、arith-int-lhs、group-mixed（字段分组×算式分组） |

关键对称性：`(.threshold + 0.5) < p.score` 与 `p.score >
(.threshold + 0.5)` 双向均 `as_f64() as f32`（f32 声明宽度跟随，
位置不对称消除）。调用实参括号（`len(` 前邻标识符）判别为非分组，
实参位保持 truthy（源码级判别，见 rust.rs opens/closes_transparent_
group）。
