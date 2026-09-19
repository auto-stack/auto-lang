# PLAN-032 T-06 翻转数据行（ramp v3 数据门 · dual-exit 达标腿）

复测日 2026-09-19；仪器 = `coverage::native_flip_coverage_data_row`
（`cargo test -p auto-lang --features ui-iced --lib
native_flip_coverage_data_row -- --nocapture`，基线 lang plan-032-dev
@ master 0c6b03fd3 + T-01..T-06 变更）。口径沿 026 D3/029（judged
≥95% 且缺项全在册；禁无数据翻转）——**仪器本批升级 judged 计算**
（此前只算 overall；仪器桶剔除集 = parse-fail/extract-fail/
bridge-fail/no-widget，029 报告剔除行同集）。

## 数据行

- **overall：Covered 22/37 = 59.5%**（37 = 029 后 047-bp-admin 新例
  入场 36→37；分母含仪器桶 15 = parse-fail×10 + bridge-fail×4 +
  no-widget×1）
- **judged：Covered 22/22 = 100.0% ≥ 95% 阈值 → 过门翻转**
- **裁定：native auto 缺省 = queue**（`resolve_native_frame_mode`
  Covered 臂 `FrameMode::Pixels` → `Commands`；观测行
  "queue-covered, default flipped@ramp3: judged 22/22 = 100%"）
- 防漏断言反转：`assert!(!flip)` → `assert!(flip)`——翻转后 judged
  跌破 95% 门即红（降级需显式裁定，禁静默回归）

## 与 029 复测行对差（p029-native-flip-retest-row.md）

| 维度 | 029 复测 | 032 | 变因 |
|---|---|---|---|
| 样本 | 36 | 37 | 047-bp-admin 入场（bridge-fail 桶） |
| overall | 16/36 = 44.4% | 22/37 = 59.5% | 六缺项清偿 |
| judged | 16/22 = 72.7% | **22/22 = 100%** | 六缺项全翻绿 |
| 012-clock | NotCovered（native-unstyled） | **Covered** | T-02 D5：SelfCenter 映射臂（self-center → center_children 真渲） |
| 018-book-reader | NotCovered（fixed/hidden/z） | **Covered** | T-02 D3 hidden（display 族响应式覆盖）+ T-05 D1 定位族（fixed 降级放行/z 放行） |
| 021-blog-viewer | NotCovered（sticky/top/z） | **Covered** | T-05 D1：sticky/top- 放行（in-flow 降级随注） |
| 024-charts | NotCovered（style-grid） | **Covered** | T-04 D4：样式 grid 分岔复用 Grid walker 真渲 |
| 041-auto-edit | NotCovered（hidden） | **Covered** | T-02 D3：hidden display:none 真渲 |
| 046-tabs-variants | NotCovered（tag:tabs） | **Covered** | T-03 D2：tabs kind 全链（kinds + View::Tabs 投影臂 + a2r 断裂修复） |

## 保真边界随注（判定翻绿 ≠ 渲染全真——I3 显式边界）

- **fixed/sticky**：降级放行——渲染 in-flow 原位（视口锚定真渲债
  另立，§10-④；018 fixed×2 / 021 sticky×2 判定翻绿但渲染保真边界
  显式）。
- **z 完整栈序**：absolute 同层相对层级已渲（覆盖序内稳定排序）；
  in-flow z 与跨层完整栈序 not-yet。
- **定位锚定**：absolute 锚定最近父块内容盒（018 bookshelf 封面徽条
  形态）；嵌套定位链（CSS nearest positioned ancestor 爬升）v1 近似
  为最近父块。堆叠族自身 fixed 高度（h-32 类）为既有丢弃边界——锚定
  按内容盒自洽。
- **tabs position Left/Right**：渲染降级 Top（托盘上置）。

## shell pack 单列行（不入 examples 分母——029 §10-⑤ 沿承）

五件维持 **Covered**（shell.at / desktop.at / switcher.at /
dashboard.at / notification_center.at；载体
`coverage::tests::shell_pack_native_covered`）。

## 翻转执行清单（D6 全项核销）

1. ✅ `client_entry.rs` Covered 臂 Pixels → Commands（翻转点）
2. ✅ 观测行文案 → "native auto -> queue … flipped@ramp3"
3. ✅ `coverage.rs` 防漏断言反转（assert!(flip) + judged 口径计算）
4. ✅ 台账 M7-a 裁定行 + 本报告（台账行随 T-08 SD-02 落笔）
5. ⏳ auto 档抽样 e2e（缺省 queue 生效断言——T-07 翻转抽样腿）
