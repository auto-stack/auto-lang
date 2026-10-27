# M841-GATE-06 修复受影响复审 — pass

2026-10-27；独立复审 agent（未参与实施）；对象：lang `9d96d0f7602aad96231b875d812b81683da04d10`（父 ecc2f54a7）——auto-val `serde = ["dep:serde", "ecow/serde"]`。

## 结论

**pass**。别名归属（AutoStr=ecow::EcoString 纯 re-export，无 orphan 选项；解析版本 ecow 0.2.6，impl 在其 serde feature 后）、修复语法（`cargo tree -i ecow -e features` 实证 `ecow/serde ← auto-val/serde` 边）、其它别名排查（外部别名仅 ecow；auto-atom 裸 serde 为惰性声明零 serde 面）、主工作区无漂移（根 Cargo.toml:262 workspace ecow 本带 serde；diff 恰 1 文件 5+/1-；全部使用面经 workspace 继承）全部成立。

**双向因果证明**：仓库外临时独立闭包复刻（Man 根式模板）——修复态 `cargo check -p auto-ast` 全绿；反事实退回旧 feature 行复现 345 错，首错 `AutoStr: serde::Serialize not satisfied` 与 finding 签名吻合。

无 oracle 弱化（man-distribution 14 例计数不变、DST-2 命令逐字一致）；Man 门禁树 HEAD `9a75d3bab` 未动。

## 影响面裁定

- 可复用：语言档 tv/tt/tu 的功能面证据（解析集不变+零源码变更）；最终基线仍按 fix-test-tiering 重跑裸 t 系（收据绑 delivery 身份）。
- 须重跑：发行门 14 例全档、777、792、统一裸 t。
- Advisory：lang worktree 7 个 `docs/reports/plan796|plan818/turnaround-*.json` 为门禁运行副产物，fold 前 `git checkout` 还原（wt-guard/移除需 clean）。
