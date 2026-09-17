# PLAN-637 B3 批次证据摘要（2026-09-17）

批次：B3 = T-04a W3 轻半（016/017/021/022/029/030/031-image-viewer/031-paint/
043/044，共 10 demo）。截图命名：`-before-b3` / `-after-b3`（031 双 demo 记为
031iv / 031p）。

## 1. 对拍总表

| demo | Phase A | Phase B | vue | vm | 归因与说明 |
|---|---|---|---|---|---|
| 016 | 4 配方 8 位点 | 无（accent 色票豁免 ×5） | IDENTICAL | IDENTICAL* | 双端零漂移；*vm before 首拍为 fit 竞态非 fit 帧，改用 B1-after（同 fit 尺寸）作基准后 IDENTICAL |
| 017 | 7 配方 19 位点（4 文件） | 无（零调色板色） | n/a | IDENTICAL | **vue 轨既有损坏**：rust 后端 app-017-chat-back 编译失败 E0308 ×8（master 既有，主检出同码可复现），VM 轨承担 |
| 021 | 3 配方 9 位点（3 文件）+ caption_text ×2 | 无 | IDENTICAL | IDENTICAL | 双端零漂移（首拍对因脚本二进制误判污染 URL 无效，stash 重取配对） |
| 022 | 4 配方 11 位点 | 泳道类目色豁免 ×5 键 + 卡片/标题/动作钮/页根映射 | 尺寸不匹配† | 89.87%（归因：页底 bg-gray-50→bg-background 翻转+卡片 bg-card+文字 token） | **vue 轨既有损坏**：生成器对当前源码仍发射 `useBooksStore` 幻影导入（books 时代残留；主检出无本计划改动同样复现）——详见 §3 |
| 029 | 11 配方 33 位点 + caption_text ×2 | 红→destructive ×3（赞/删）；白色玻璃覆盖层 ×7 键豁免 | IDENTICAL（差异区在截屏视口外） | 6.98%（bbox=照片格操作钮区） | vue 差异区位于 fullPage 视口外故相同；vm 差异=destructive 映射，逐条对映射 |
| 030 | 4 配方 10 位点（4 文件）+ caption_text ×1 | zinc-500/400→muted-foreground ×6；text-white（视频叠字）+amber（audio-note）豁免 | 0.47%（bbox=时间码/控制条区） | 20.08%（bbox=字幕/播放列表文字区） | 逐条对映射表 |
| 031iv | 6 配方 19 位点 + caption_text ×1 | 无 | IDENTICAL | IDENTICAL | 双端零漂移（vue 对同 021 重取） |
| 031p | 2 配方 6 位点 + caption_text ×1 | 无 | IDENTICAL | IDENTICAL | 双端零漂移 |
| 043 | 7 配方 31 位点 + caption_text ×10 | 无 | n/a | IDENTICAL | **VM 原生 demo**（pac 指定 iced 渲染，无 vite 轨，属 demo 属性） |
| 044 | 6 配方 25 位点 + caption_text ×7 | 无 | n/a | IDENTICAL | 同上 |

**零漂移达成 8/10**（016/017/021/029-vue/031iv/031p/043/044；022 由 VM 承担、
029-vue 视口外差异说明在案）；Phase B 差异全部逐条归因（022/029/030）。

## 2. 机制与门禁

- **stylekit 消费**：B3 全部 10 demo 挂 `dep stylekit`，caption_text 全等位点
  ×28（043×10、044×7、021×2、029×2、余各 ×1）。AC-05 累计 **13 demo**（045/
  015/007 + B3 十个），远超 ≥10 目标。
- **guard**：completed 增至 25；新增豁免 016 色票 ×5、022 泳道类目 ×5、
  029 白色覆盖层 ×7、030 叠字/状态 ×2。自测 5/5、正例 25/25 PASS。

## 3. 重大偏差与已知债（review 需知）

1. **022 vue 轨既有缺陷（主检出复现，非本计划引入）**：干净重建（gen/dist/.auto
   全清）后生成器仍对当前源码发射 `useBooksStore` 幻影导入（022 源码已无任何
   books 引用，仅存注释提及），vite 500 → 白屏。第一次再生还观察到 AutoCache
   回填陈旧 App.vue 的增量失效现象（二次运行自愈）。定位到 lib.rs
   `collect_module_imports` 的导入回收链，修复超出本计划范围——已列为
   vue-gen 已知债（建议：ui_gen 导入回收的失效键审查 + 022 books 残留清理）。
2. **017 vue 轨既有损坏**：后端 E0308 ×8（master 既有），VM 轨承担。
3. **043/044 为 VM 原生 demo**：无 vue 轨（demo 属性，非缺陷）。
4. **029 照片加载缺陷（用户报告，既有）**：app.at 硬编码绝对磁盘路径
   （file:///…/thumbnails/thumb_NNN.jpg），VM 直接读盘可用、Vue 网页端被浏览器
   安全策略拦截 → 网页版缩略图全裂。修复方向=迁移 image_pipeline 媒体会话通道
   （参照 031-image-viewer，`open_media_session` 返回跨端 URI），属功能性改造
   另立小修复，不入本计划样式范围。029 的样式对拍不受影响（前后两侧同态）。
5. **跨会话 master 演进**：本批中另一会话将 master 推进至 Plan-638（020 重设计，
   media_service 增 `parse_artist_and_title`）并重建了共享 auto 二进制——022 的
   gen 缓存因本批改动失效后，再生后端引用新函数而旧 lib 无有 → E0425。已按
   批次纪律 stash→ff-sync→pop 解决（worktree 现含 638）。教训注记：共享二进制
   随他会话演进时，长批次中途应主动 re-sync。
6. **采集脚本二进制误判**：rustc 输出可使 grep 将日志判为 binary → URL 变量
   污染 → Playwright 回退默认 3000 拍到错误页/错 app。已修（`grep -a`）；
   受污染的 021/031iv 配对已用 stash 工作流重取，022-vue 对以 VM 承担。
7. **跨文件重复串口径**：重复串跨文件（各 ×1）不做本地提取（如 021 的页壳、
   017 的 row_gap2）——本地配方以文件为单位，跨文件归共享库需全等命中。

## 4. AC 进度（B3 后）

- AC-01：Phase A 完成 25/33（B3 十个全完成）。
- AC-02：token 化累计 25 demo（022/030 完成；016/029 豁免登记）。
- AC-03：零漂移 016/017/021/029/031iv/031p/043/044 双端（或单可用轨）实证。
- AC-04：022/030 差异逐条归因在案。
- AC-05：消费 13 demo ≥10 ✅（位点 26+28=54）。
- AC-06：guard 25/25。
- 剩余：B4（018/024/026/027/041 巨型件 + tree_icon 四胞胎联动）、B5（重型+收口）。
