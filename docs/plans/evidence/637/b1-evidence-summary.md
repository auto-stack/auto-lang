# PLAN-637 B1 批次证据摘要（2026-09-17）

批次：B1 奠基+试点（T-00 / T-01 / T-01b / T-02=W1 / T-06）。
worktree：`D:/autostack/.wt/lang-637/auto-lang`（plan-637-dev）。
截图命名：`<demo>-<vue|vm>-<before|after>.png`；before 取自本 worktree 改动前状态
（015/016 的 before 因后补，取自同 commit 主检出，内容等价）。

## 1. 像素对拍总表（PIL RGB 直方图法；"changed"=差异像素占比）

| demo | 任务 | vue | vm | 归因 |
|---|---|---|---|---|
| 001 | no-op（无重复串/无调色板色） | —（未改未拍） | — | 矩阵 no-op 行 |
| 002 | no-op（同上） | — | — | 同上 |
| 003 | T-02 Phase A：本地配方 field_col/field_label（×2+×2 位点） | IDENTICAL | IDENTICAL | **零漂移达成**（字节等价+像素等价双证） |
| 004 | T-02 Phase B：8 簇映射（文件头注） | DIFF 11.4%（bbox=卡片区） | DIFF 10.5% | 逐条对映射表：徽章→secondary、Follow→primary、卡面→bg-card/border-border 等 |
| 005 | T-02 A+B 合并：3 本地配方（×6 位点）+9 簇唯一串 | DIFF 16.2% | DIFF 81.4% | vue 差异=按钮/文本/边框逐条映射；vm 大差异=**bg-white→bg-card 在 VM 暗主题下整卡翻转**（字面量退休、颜色随主题的目标行为）+同左映射 |
| 006 | T-01b：Theme 按钮+SettingsPopover 移除 | DIFF 3.9% | DIFF 3.2% | 顶栏控件移除+内容上移，before/after 对比在案 |
| 010 | T-01b：同上 | DIFF 25.8% | IDENTICAL | vue=移除+上移；**vm 侧该按钮区在 iced 渲染中本不可见**——编辑前 VM 树 "Theme" 出现 0 次（快照存档 /tmp 会话证据，见 §3） |
| 013 | T-06 Phase B：16 簇映射（两文件头注） | DIFF 95.4% | DIFF 98.3% | 主因=页面底 bg-gray-100→bg-background 全屏翻转（Playwright dark / VM dark 下语底色变深），其余=映射表逐条 |
| 015 | T-06：caption_text ×5（stylekit 消费）+豁免 ×5 | IDENTICAL | DIFF 22.8% | vue **零漂移达成**；vm 差异=有状态 demo（rust db）笔记列表/相对时间戳内容漂移，样式区稳定——差值可视化 `015-vm-diff-vis.png` 在案，非样式像素 |
| 016 | T-01b：仅 pac.at 悬空 dep 移除 | IDENTICAL | IDENTICAL | **零渲染影响实证**（内置 settings 面板为 store 本地实现，未动） |
| 045 | T-00 回归：stylekit 扩容后既有消费 | IDENTICAL | IDENTICAL | **既有消费不破实证** |

## 2. 基建与门禁

- **T-00**：stylekit/src/front/styles.at 首批 8 recipe（card_base/pill 既有 + pill_ghost/hint_text/caption_text/icon_base/input_field/section_title 新增）。input_field 按 D1 注记直接落 Phase B 定案形（border-border），消费方在 W4/023。
- **T-01**：`scripts/style_palette_guard.py` + `scripts/style_palette_manifest.json`。自测 4/4（正例零命中/反例 4 命中/不扫面边界/hover 变体命中）；已完成集正例 8/8 PASS。注释行不扫（映射表头注引用旧字面量属正常）。
- **T-01b junction 清理**：主检出悬空 `deps/settings` ×5 拆除（006/010/015/016/**019**——019 为计划外发现，同款悬空），空壳 deps 目录随行。`011-calculator/deps/common` 同类悬空，留 W4 波处理（011 在 W4 顺带自足化）。

## 3. 偏差与勘正记录（review 需知）

1. **r2「standalone 解析必失败」失实**：006/010/016 在无 junction、无 `examples/ui/common` 的新 worktree 中 `auto run` 双端均可起（dep 疑经 index/AmConfig 缓存兜底解析）。T-01b 移除仍按用户裁定执行（裁定依据为「demo 跟随系统」而非仅解析失败）。
2. **006/010 保留 SetTheme/SetAccent msg+handler**：T-01b 枚举删除面之外；作为后续「宿主→内嵌 demo 主题传播」路线（待澄清#1 注记②）的既有通道保留，本 demo 内无发射方。
3. **016 内置 settings 面板保留**：其为 store 驱动的 demo 本地实现（非共享包消费），不在 T-01b 枚举面内；仅摘除悬空 `dep settings`。
4. **004 状态点 bg-green-400 豁免**：registry 无 success 语义键，按装饰性指示色豁免登记（manifest 带回迁注记）；渐变 from-/to- 豁免为 r2 既有口径。
5. **T-00 验证命令勘正**：计划所写 `cargo test -p auto-man --lib vue` 中 auto-man 在 auto-os 仓不在本仓；本仓侧改动物为 examples 层 .at，验证以 045 双端 run（IDENTICAL）+ guard 为准。依 AGENTS Category A（未动 crates/**）不跑 cargo t。
6. **主检出构建副产物还原**：主检出跑 015 时工具链自动改写 `examples/rust-workspace/Cargo.toml`（扫入历年 *-back 残留目录），已 `git checkout --` 还原（可再生副产物，非人改）。
7. **截图脚本陷阱留档**：vite Local URL 被 ANSI 色码截断需先剥离；`auto run` 会在 3000+ 顺延占端口，批量采集必须按端口清场；PIL RGBA 差值图 `getbbox()` 不可靠，用 RGB 直方图。

## 4. AC 进度（B1 范围）

- AC-01（Phase A 覆盖）：B1 完成集 8/33（001-005/013/015/045）；003 零漂移双证；013/015/045 依 607/635 既有达标。
- AC-02（token 化）：004/005/013 完成；豁免登记 004×2、015×5（accent 色票，字面量即语义）。
- AC-03（零漂移）：003 vue+vm IDENTICAL；015 vue IDENTICAL；016/045 双端 IDENTICAL。
- AC-04（受控变更）：004/005/013 映射表=两 demo 文件头注 + 本摘要归因列；截图对在案。
- AC-05（机制消费）：dep stylekit+use 实装 2 demo（045 既有 + 015 新增 caption_text ×5）；W1 各 demo 无全等命中，按 r2 注记不强行挂 dep。
- AC-06（门禁）：guard 落地+自测绿+manifest 驱动；随批次追加 completed 行。
- AC-07（回归）：demo 层无语料 golden 牵连（grep 证实）；Category A 免 cargo t；B5 终态再付全量门禁。
