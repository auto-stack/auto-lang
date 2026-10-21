# PLAN-814 R13 work 修复轮证据（R9-01：完整比较符消费+表达式节点类型传播）

来源：探针复制自 review-r13 冻结快照（复审员原件未动），在本目录用
修复后生成器（worktree `D:/autostack/.wt/lang-814/auto-lang` @ `42a4ef792`）
再生全部 `.at.rs` 并独立编译。修复基线（复现用）= R13 受审提交
`feae28e0f`：9 败/49 过逐项复现后再修。

## 复现命令

```bash
cargo build --offline --locked --manifest-path <本目录>/Cargo.toml \
  --bin r10-generate --bin r10-overlay --target-dir <原树>/target
python <本目录>/replay.py     # 生成+逐 feature check+overlay，保留真实退出码
```

注：`Cargo.lock` 按 attachments/.gitignore 约定不入库——复现者可从
review-r13 冻结件复制同锁，或去掉 `--locked` 重新解析。

## 回放收据

`replay-summary.log`：**62 feature + 无 feature 正控 + overlay =
64/64 exit0**（fmt 后重建复放同结果）。R13 九败全修（le/ge loop/prop/
分组、not-bool 两方向/prop）+ R9-R12 全反例正控保持。

## 修复点与矩阵

| 修复 | fixture |
|---|---|
| 前向 `<=`/`>=` 双字符整体消费（残留 `=` 污染区间回归） | le-loop[-rhs]、ge-loop[-rhs]、prop-le、prop-ge、group-le、group-ge-float |
| 布尔取反 `!` 类型传播（bool 保型） | not-bool[-rhs]、prop-not-bool |
| 六比较符×两方向审计补齐（R13 复审指示"不只改九条产物"） | ne-loop[-rhs]、prop-ne（`!=`）、ge-float-arith（`>=`×float 算式前向） |
| 同不变量补齐：括号内 `&&`/`||` 逻辑组合 | logical-bool、not-logical（取反×逻辑） |

关键形态：`(p["score"].as_f64().unwrap_or(0.0) as f32) >= ( self.threshold
+ 0.5 )`、`(p["dirty"].as_bool().unwrap_or(false)) == ( ! self.allowed )`、
`( self.a && self.b ) == (p["dirty"].as_bool()...)`。形状正控
（known-ge/known-not-bool）与既有 shaped 路径不受影响。
