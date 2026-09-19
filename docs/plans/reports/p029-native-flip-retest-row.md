# PLAN-029 T-08 翻转数据门复测行（D7 dual-exit）

复测日 2026-09-18；仪器 = `coverage::native_flip_coverage_data_row`
（`cargo test -p auto-lang --features ui-iced --lib
native_flip_coverage_data_row -- --nocapture`，基线 lang plan-029-dev
@ 6190fd4e5 + T-08 变更）。口径同 026（judged ≥95% 且缺项全在册
not-yet 才翻；禁无数据翻转）。

## 数据行

- **overall：Covered 16/36 = 44.4%**
- **judged（剔除仪器桶：parse-fail ×10 + bridge-fail ×3 + no-widget
  ×1 = 14）：Covered 16/22 = 72.7%**
- 阈值：≥95%（026 D3 定案沿用）
- **裁定：两口径均未达阈值 → 维持 native auto = independent（不翻）**
  ——client_entry.rs Covered 臂维持 `FrameMode::Pixels`（翻转点
  one-line 不动，下次复评随 ramp v3）。

## 与 026 数据行对差（p026-native-flip-data-row.md）

| 维度 | 026 | 029 复测 | 变因 |
|---|---|---|---|
| 样本 | 35 | 36 | 031-image-viewer/043/044/045/046 等新例入场（026 后 examples 扩容） |
| overall | 16/35 = 45.7% | 16/36 = 44.4% | 新例稀释（分母 +1 而覆盖未增——新例全落仪器/not-yet 桶） |
| judged | 16/21 = 76.2% | 16/22 = 72.7% | 046-tabs-variants（tag:tabs 真缺项）入 judged 分母 |
| 009-hero-section | NotCovered（opacity-50） | **Covered** | 029 T-08 opacity 降级放行（渲染 not-yet，prefixes ⑦） |
| 041-auto-edit | NotCovered（hidden+popover） | NotCovered（hidden） | 029 四 kind 入册清偿 popover 半句；hidden 定位族维持 |

注：012-clock 自 026 后被重做（新类落 `native-unstyled`——真实未映射
缺口，如实入行非静默配平；映射与否随该例的 native 适配另行裁定）。

## shell pack 单列行（不入 examples 分母——§10-⑤ 定案）

| 件 | 判定 |
|---|---|
| shell.at | **Covered** |
| desktop.at | **Covered** |
| switcher.at | **Covered** |
| dashboard.at | **Covered** |
| notification_center.at | **Covered** |

载体 = `coverage::tests::shell_pack_native_covered`（解析序 $AUTO_OS_ROOT
→ 兄弟 auto-os → D:/autostack/auto-os；B 程序覆盖门预演断言）。

## judged 缺项在册核对（72.7% → 95% 的缺口面）

| 例 | 缺项 | 在册 |
|---|---|---|
| 018-book-reader | fixed/hidden/z | §1.8 定位族 not-yet |
| 021-blog-viewer | sticky/top/z | 同上 |
| 024-charts | style-grid | grid 样式 not-yet（D5 整 kind 线） |
| 041-auto-edit | hidden | 同 018 族 |
| 046-tabs-variants | tag:tabs | tabs 整 kind not-yet |
| 012-clock | native-unstyled | 本报告在册（新例类映射缺口） |
