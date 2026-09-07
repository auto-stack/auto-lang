---
plan_id: PLAN-590
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: Stage B P-5——资产搬迁本体批（台账+launcher+apps+画廊+L8 指针+行数实测）
author: [zhaopuming, ZCode]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-man/examples]  # 受影响的 specs 路径
current_step: 0
total_steps: 8
---

# [PLAN-590] Stage B P-5——资产搬迁本体批（Design 01 §7 P-5）

## 变更摘要

Stage B 收口批：治理资产（A1 台账）+ 物理资产（A3 launcher + B 批 apps +
画廊两件）迁入 auto-os；框架仓引用清零（V7 后半）；框架层行数实测归档。
前置全就绪：P-2/P-3 ✅、P-4 ✅、P-7 ✅、541/582 已合并（唯一硬窗口满足）。

**枚举实测定案（2026-09-07，Design 01 §1-B「以实测为准」授权）**：

| 资产 | 源（auto-lang） | 去向（auto-os） | 实测依据 |
|---|---|---|---|
| A1 台账 | docs/plans/autos-desktop-program.md | docs/plans/（同名） | 活账整体迁移；auto-lang INDEX.md 追加台账指针行 |
| A3 launcher | examples/ui/028-launcher/ | apps/028-launcher/ | §1-A3 |
| B1 sys-monitor | examples/ui/025-sys-monitor/ | apps/025-sys-monitor/ | 541 终态主目录 |
| B1' 残骸 | examples/ui/025-dashboard/（仅 gitignored gen/） | 不迁（磁盘残留另清） | git 零跟踪实证 |
| B2 minesweeper | examples/ui/038-minesweeper/ | apps/038-minesweeper/ | §1-B |
| B4 画廊两件 | examples/{ui-gallery,widgets-gallery}/ | 顶层 {ui-gallery,widgets-gallery}/ | **os-008 定案口径**：578 变更摘要=两 examples 级画廊（非 029-photo-gallery——后者为教学 demo 留架 L7）；顶层落位保 578 的 `apps_dir.parent()/{ui-gallery,widgets-gallery}` 锚定语义自然延续 |
| B3/B5 | 无实物 | —— | B3（clock/tetris/klondike）计划已迁无目录；**B5 前提修正**：582 产出=website playground（packages/auto-playground-vue），非 examples/ui 资产——Design 01 §1-B B5 行注记修正 |
| common | examples/ui/common/（仅 settings/） | 视画廊引用实测 | 三 app（028/038/025-sys）源码零引用实证；画廊引用执行期查，有则抽 apps/common/ |

**测试面清单（执行期重锚主体）**：①examples manifest 不变量（api=manifest
计数，0cb047cc8 口径）——搬 4 app + 2 画廊后计数与断言更新；②
gallery_pages_compile_tests 锚 `examples/widgets-gallery/src/front/pages`
路径——迁移后改锚 auto-os 路径或迁移测试本体（后者为正：画廊测试随画廊
走，auto-os 侧承接）；③desktop_mcp/tests 随 app 目录走；④docs_gen/
kitchen-sink 若含画廊页需同步；⑤scan_examples_ui ≥34 锚（42−4=38 仍过）。

## 目标

1. 七项资产物理就位 auto-os（git mv 保历史）；auto-lang 活动区引用清零
   （V7：grep 028-launcher/038-minesweeper/025-sys-monitor/ui-gallery/
   widgets-gallery/autos-desktop-program 引用=0，INDEX 指针行除外）。
2. 框架测试面重锚后 `cargo tv`+`cargo tf` 基线口径绿（预存红台账豁免）。
3. 台账迁移后 auto-os 侧台账接棒（Design 01 §5 迁移机制 4）；INDEX 追加
   台账指针行。
4. 框架层行数实测归档（tokei/cloc，落 auto-os Design 01 §2 规模注记或
   本仓报告）。
5. L8 指针登记 + Design 01 §1-B 枚举修正注记（B5）+§7 P-5 行回填。

## 执行步骤

1. [ ] common 引用终审（画廊两件源码 grep common/）→ 抽取或不迁裁定。
2. [ ] 物理迁移 git mv：A1/A3/B1/B2/B4 七件 → auto-os（一提交/仓）。
3. [ ] auto-lang 引用清零：测试锚改址/迁移（gallery_pages_compile 随画廊
   迁 auto-os 或改路径锚）、examples manifest 再生、docs_gen 同步、
   INDEX.md 台账指针行。
4. [ ] auto-os 侧承接：apps/ 注册（P-3 容器探测自然命中验证=boot log
   entries +N）、画廊测试落位、台账接棒注记。
5. [ ] 测试面重锚验证：tv/tf 基线口径（charts/lucide 预存豁免）。
6. [ ] V7 引用清零 grep + 行数实测归档（tokei/cloc）。
7. [ ] Design 01 §1-B（B5 修正+枚举定案表）+§7 P-5 行回填；KNOWN-DEBT
       P584 全结案核对。
8. [ ] 收口：specs 沉淀 + plans.md 回写 + INDEX 重生 + 归档。

## 验收标准

- [ ] 七件资产在 auto-os 就位（git mv 历史）；框架仓 git status 零残留。
- [ ] V7 grep 引用清零（INDEX 指针行除外）；examples manifest 不变量
      再生一致；tv/tf 基线口径绿。
- [ ] 桌面 boot：registry entries 含迁移 apps（容器探测命中）——P-3
      机制端到端实证。
- [ ] 台账在 auto-os 接棒；INDEX 指针行在案；行数实测归档。
- [ ] Design 01 §1-B/§7 回填；两仓提交在案。

## 复审记录

（待 /auto-plan:review 填写。）

## 待澄清事项

（枚举已按 §1-B 授权实测定案；B5 修正与 B4 顶层落位为执行期裁定，
依据在案可复核。）

## 变更摘要

## 目标

## 架构方案

## 需求分析与背景调查
（从 docs/specs/overview.md 与相关 module spec 取材）

## 详细设计

## 测试设计

## 验收标准

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

## 复审记录

## 待澄清事项
