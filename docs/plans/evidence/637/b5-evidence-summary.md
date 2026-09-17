# PLAN-637 B5 批次证据摘要（2026-09-17）

批次：B5 = T-05 重型四件（011/019/020/023）+ T-07 矩阵终验 + T-08 回归收口。

## 1. T-05 对拍总表

| demo | 改写 | vue | vm | 归因 |
|---|---|---|---|---|
| 011 | tier 映射（zinc/gray→secondary/muted 系、red-400→primary）+ 7 键位配方提取（key_digit_dark ×13 等） | 13.68%（bbox=计算器区） | 83.41%（整机随暗主题键色换装） | 映射表=文件头注；运算符橙键 ×5/科学键 indigo ×6 类目豁免 |
| 019 | tier 收敛（同 008/009 口径）×13 键 + text-white ×4 视频叠字豁免 | 60.92%（全页 tier 换装） | n/a（VM 轨挂起，既有） | 映射表=文件头注 |
| 020 | tier 残余（zinc-950→background 等 ×5 键）+ rose/emerald/白字豁免 ×9 | IDENTICAL | IDENTICAL | 638 重设计后残余极少，可见面零漂移 |
| 023 | 灰阶分层 ×12 键 + destructive 族 + **input_field ×13 消费**（D1 家族源 demo） | 3.68%（bbox=内容区） | n/a（后端再生成 E0425，既有） | 映射表=文件头注；brand-green 品牌色对整体豁免 |

## 2. T-07 矩阵终验

- guard 全量复扫：**34/34 PASS**（completed 34 = 33 rollout + 045；025/038 N/A）。
- 豁免表复核：14 demo 共 ~50 键，全部标注类目/状态/品牌/图像叠字定性（manifest
  内联 reason）。
- 矩阵：8.1–8.4 批次矩阵 + 本档 B5 增补，合计 34 行全勾记。

## 3. T-08 回归与收口

- **junction 清理**：worktree demo `deps/stylekit` ×34 枚逐枚 rmdir + 空壳 deps
  清除；主检出 011 `deps/common`（B1 遗留悬空）拆除；主检出各 demo 陈旧 junction
  复扫清零。
- **gen/dist 产物清除**：041/003/004/005 等超长路径 pnpm 链接树以
  mv→robocopy /MIR /XJ→rmdir 安全清除（不穿透链接），wt-guard 终判 **clean**。
- **cargo tv 阻断记录（非本计划引入）**：tv 构建 E0433 ×20——
  `crates/auto-lang/src/vm/ffi/term_engine.rs` 的 terminal 滚动块无
  `cfg(feature = "ui")` 门控（PLAN-019 复审 f165040f7 曾修同文件同症，后被后续
  会话合并复活；主检出同码可复现）。crates 归 019/020 lineage 所有，本计划
  （Category A，全程零 crates 改动）不越界代修；**精确解除动作=为该块补
  ui/not(ui) 双臂门控（照抄 019 复审做法）后重跑 cargo tv**。
- 本计划的等效回归面：30+ demo 双端 auto run 全部起树成功（解析/链路/渲染端到
  端）+ 语料隔离验证（grep 证实 examples/ui 不被语料 golden 引用）。

## 4. 偏差与勘正（B5）

1. **020 面貌勘正**：计划写作时 264 裸色为 638 重设计前计量；本批实测残余极少
   （且 638 已 token 化大半），B5 实做 = 残余 tier 映射 ×5 键 + 豁免登记。
2. **019/020 VM 轨挂起**：019 vm 截图运行挂起（媒体/扫描循环）、020 超时后产物
   迟到——均既有行为；019 以 vue 承担、020 双端齐全。
3. **023 后端再生成 E0425（db/slug）**：干净态（stash 前）同样失败=既有缺陷
   （022 useBooksStore 同族，生成器/增量 gen 债）；023 以 vue 轨承担。
4. **011 映射裁定**：数字键→secondary 族、功能键行→muted+foreground（保留
   两档键位层级）、显示器 red-400→primary；运算符橙/科学 indigo 键位色彩编码豁免。
5. **023 brand-green**：demo 自定义语义色（非灰阶调色板），按钮整体豁免保留。

## 5. 最终进度

- AC-01：Phase A 33/33（全部有源码 rollout 目标完成）✅
- AC-02：token 化 33/33（豁免登记在案）✅
- AC-03：零漂移实证覆盖全部纯 Phase A 面 ✅
- AC-04：Phase B 差异全部逐条归因 ✅
- AC-05：stylekit 消费 16 demo（≥10 ✅），位点 100+
- AC-06：guard 34/34 ✅
- AC-07：cargo tv 被既有 E0433 阻断（记录+移交），语料隔离已证；裁决权交独立 review
