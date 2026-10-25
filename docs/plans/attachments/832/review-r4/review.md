# PLAN-832 merge blueprint 修复独立复审 R4

2026-10-25。**outcome: pass**。 reviewer 为独立 explore 子上下文，未参与实现；未修改实现、Plan、Spec 或 ledger，未创建第二 checkout，未运行整档。

## 基线

- Man=`ceb8dfdd469b2dec1f2490aae1126b0b3303e35e`；lang=`adb5ce0f3ba8935f3f546b4bc40217c4d20235e9`。
- UI=`aee499e87093ff727f68ac8467c506e9ecf6ecdc`；down=`c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`；四树进出 clean。
- 范围仅限 merge blueprint finding；R3 F01..F04 保持，不机械重跑旧族。

## Finding closure

**F832-MERGE-BP-01 CLOSED**。上一版通用 UI 回退可在安装态先命中开发语言库，使 SDK 回退失效。最终优先级为显式 env→真实项目库→安装 SDK→开发语言 owner，Man 占位不视为真实项目库。

独立实测：

- `cargo nextest run -p auto --bin auto cmd_bp::tests`：4/4，20 skipped。
- 安装 SDK 唯一 marker 经真实 `bp show form/login` 出现；同一二进制在真实语言项目根或显式 `AUTO_BLUEPRINTS_ROOT` 下不出现 SDK marker，证明项目库/env 优先。
- marker 恢复后 spec hash 回到 `84747cdf…86c3e`；安装恢复后 `bp list` 共19包。
- Man 根清 `AUTO_BLUEPRINTS_ROOT`/`AUTO_LANG_ROOT`/`CARGO_MANIFEST_DIR` 后 `bp list` 包含 `form/login` 与 `layout/gallery-shell`；显式缺失根输出 no blueprints，不回落 SDK 或语言库。
- 开发与安装 auto.exe 同 SHA256=`44c1ab0e…25d0`。
- SD-01/03 与实际行为一致；无第二 blueprint 生产源。`git diff --check`、`cmd_bp.rs` rustfmt、Python AST 全过。

原始 marker driver 与日志：[repair](../merge/repair-report.md)、[SDK marker](../merge/installed-sdk-marker.log.txt)、[定向测试](../merge/repair-bp-install.log.txt)。

## 路由与剩余门

T-10 对最终 blueprint 修复通过，Plan 可恢复 reviewed 10/12。merge 仍须重跑 Man `cargo t` 及适用 `tcore`/`ti`、必要 consumer/UI/跨仓发行门；lang 冻结回归另受 B845-FEATURE-01/PLAN-847 外部 blocker 约束，不是 832 豁免或回归。后续合入、来源读回、Spec/ledger/索引、归档和 guard 清理均未完成。
