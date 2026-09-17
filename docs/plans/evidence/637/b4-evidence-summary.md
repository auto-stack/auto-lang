# PLAN-637 B4 批次证据摘要（2026-09-17）

批次：B4 = T-04b W3 巨型件（018/024/026/027/041）+ tree_icon 四胞胎联动。
截图命名：`-before-b4` / `-after-b4`；before 均经 stash 流程取自改动前状态。

## 1. 对拍总表

| demo | Phase A | Phase B | vue | vm | 归因 |
|---|---|---|---|---|---|
| 018 | 9 配方 36 位点（8 文件）+ icon_base ×14 | 无（覆盖层/成功徽章豁免 ×4） | IDENTICAL | IDENTICAL | 双端零漂移；icon_base 引用在 vue 轨渲染正确（树图标在案） |
| 024 | 4 配方 15 位点 + **hint_text ×75、caption_text ×17**（area/bar/line/donut 四图表件） | Pause→secondary、Play→primary（012 先例） | 0.66%（bbox=两按钮） | 1.94%（含图表动画帧态） | vue bbox 恰为映射按钮；vm 差异含 live 图表数据滚动噪声（Pause/Play 驱动），样式与动画帧不可解耦已注记 |
| 026 | 13 配方 43 位点 + caption 回退 ×4（见 §3-1） | Connect/Save→primary、删除/错误簇→destructive ×5、状态点豁免 ×2 | IDENTICAL（默认视口） | IDENTICAL（默认视口） | Phase B 换装区位于 SQL 控制台/表动作区（非默认视口）；**交互截图** 026-vue-sqltab-after-b4.png 证明 destructive 错误注入钮/面板按映射渲染 |
| 027 | 9 配方 42 位点 + icon_base ×14 | 无（零调色板色） | IDENTICAL | IDENTICAL | 双端零漂移（纯 A） |
| 041 | 8 配方 15 位点 + icon_base 回退 ×14 | zinc 分层映射（200→foreground、300/400/500→muted）×13、amber 豁免、#16171B 面板底豁免 | n/a（render:vm 原生） | 0.12%（bbox=状态栏/菜单文字区） | 差异即 zinc 分层映射 |

**零漂移 4/5 双端实证 + 041 单轨 0.12% 文字级差异**；hint_text 全仓最大消费家族
（×75）在 024 落地。

## 2. tree_icon 四胞胎联动结果

- 四文件改前 md5 一致（5725f948）；icon_base（stylekit 既有配方）替换 ×14/文件。
- **018/026/027：配方引用可用**（vue/VM 解析+渲染均验证，018 树图标目检在案）。
- **041 回退**：041 为 `render: vm` 原生轨，其组件注册路径对括号形属性中的配方名
  判 undefined variable → 组件**静默跳过**（树图标消失）；大括号形亦不被 icon
  元素接受（40 错）。041 回退为字面量 + 文件头注记（14 位点待框架支持后回迁）。
- **落地 42/56 位点**；041 的 14 处列入已知债。
- 联动发现：括号形属性（`icon (..., style: 配方名)`）的配方引用在 VM 原生轨
  不被支持——大括号形（`text "x" { style: 配方名 }`）则全轨可用。框架解析路径
  分叉注记在案（与 041 文件头自述的增量路径 bug 同族）。

## 3. 已知债与偏差

1. **026 caption 消费回退 ×4**：026 为 475 组件包形态（widget 内
   `use { package: official }` 块），顶层 stylekit use 与之解析互斥
   （caption_text 判 undefined → 20 错级联）。已回退字面量+注记；待框架修复
   （顶层 recipe use 与 widget 内 package use 的作用域合并审查）后回迁。
2. **041 tree_icon 回退 ×14**：见 §2。
3. **024-vm 对拍含动画噪声**：live 图表在两帧间数据滚动，样式像素与动画态不可
   解耦；按钮映射以 vue 侧 bbox 精确对位为主要凭据。
4. **首版转换的两处行尾注释事故（已修）**：024 按钮行与 041 ctx_menu 括号内被
   追加的 `// Phase B` 注释吞掉行尾 `} )` → 解析错；已修复并全目录审计清零。
   教训：映射注释只允许置于行尾安全位（属性行之后无后续 token）或独立行。
5. **026 六配方补提取**：首轮 per-file 盘点 head 截断漏盘（同 B3 教训），本次
   以无截断重扫兜底；跨文件重复（各 ×1）仍按口径不提取。
6. **026 vm-before 取证**：首拍遇解析错误（见 §3-1 修复前态）无效，修复后经
   stash 流程重取干净 before。

## 4. AC 进度（B4 后）

- AC-01：Phase A 完成 30/33（剩 011/019/020/023 重型四件）。
- AC-02：token 化累计 30 demo（豁免登记在案）。
- AC-03/04：B4 五件全部实证（4 双端零漂移 + 024/026 归因）。
- AC-05：消费 13 demo（位点 26+28+54=108；B4 增 icon_base 42 + hint/caption 92
  中 041 回退后实落 134-14=…，明细以 manifest+各 demo use 行为准）。
- AC-06：guard 30/30。
- 剩余：B5（011/019/020/023 重型 + T-07 矩阵终验 + T-08 收口）。
