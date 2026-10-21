# PLAN-814 R14 work 修复轮证据（R9-01：比较子表达式 bool 结果传播）

来源：探针复制自 review-r14 冻结快照（复审员原件未动），在本目录用
修复后生成器（worktree `D:/autostack/.wt/lang-814/auto-lang` @ `6f1cac8cb`）
再生全部 `.at.rs` 并独立编译。修复基线（复现用）= R14 受审提交
`42a4ef792`：5 败/92 过逐项复现后再修。

## 复现命令

```bash
cargo build --offline --locked --manifest-path <本目录>/Cargo.toml \
  --bin r10-generate --bin r10-overlay --target-dir <原树>/target
python <本目录>/replay.py     # 生成+逐 feature check+overlay，保留真实退出码
```

注：`Cargo.lock` 按 attachments/.gitignore 约定不入库——复现者可从
review-r14 冻结件复制同锁，或去掉 `--locked` 重新解析。

## 回放收据

`replay-summary.log`：**97 feature + 无 feature 正控 + overlay =
99/99 exit0**（fmt 后重建复放同结果）。R14 五败全修（比较子表达式
两方向/prop/取反包裹/逻辑左子树）+ R9-R13 全反例正控保持。

## 修复点与矩阵

| 修复/审计 | fixture |
|---|---|
| 比较节点结果=Bool（`has_top_comparison`：六符全集、括号外、字符串字面量外；`!`/逻辑递归自然传播） | bool-state-comparison、bool-comparison-rhs、prop-bool-comparison、not-bool-comparison、bool-comparison-logical |
| 边界审计（R14 work 新增） | bool-cmp-arith（比较×算术子树 `((.threshold + 0.5) > 1.0)`）、string-cmp-guard（字符串内 `<` 防误报） |

关键形态：`(p["dirty"].as_bool().unwrap_or(false)) == ( self.limit > 3 )`、
`p["name"].as_str().unwrap_or_default() == "a<b"`、
`(p["dirty"].as_bool()...) == ( ( self.threshold + 0.5 ) > 1.0 )`。
形状正控（known-bool-comparison[-logical]、logical-nested-control、
prop-logical-rhs-control）与 shaped 路径不受影响。
