---
plan_id: PLAN-634
status: drafting
feature_name: 015 遗留三账清偿——像素金样环境漂移归因修复 + badge/preedit WeakParagraph 同族根修 + a2r 块尾分号
author: [zcode-session]
created_at: 2026-09-15T12:00:00Z
updated_at: 2026-09-15T12:00:00Z
plan_revision: 1
current_step: 0
total_steps: 3
supersedes_spec_components: []
new_spec_components:
  - docs/specs/widgets/terminal-iced-draw.md
touched_goals: []
---

# PLAN-634 · 015 遗留三账清偿——像素金样环境漂移归因修复 + badge/preedit WeakParagraph 同族根修 + a2r 块尾分号

## 0. 变更摘要

auto-term PLAN-015(已 archived/delivered)在 auto-lang 侧留下三笔账,
本计划一次性清偿:

1. **016 像素金样环境漂移(最高优先)**:terminal_pixel
   selection/cursor 两金样在 200% DPI 桌面持续 2 红,bisect 定案
   非代码(f0dc16732 晨绿点洁净 worktree 复红,跨树复现)——master
   的 terminal 门当前是红的。归因(字体栅格/远程桌面/显示设置候选)+
   修复(受控渲染或金样策略),恢复 master 门绿;
2. **#17 同族 badge/preedit**:015 根修实证 iced wgpu
   `fill_paragraph` 排队 WeakParagraph,draw 局部段落析构后 flush
   静默丢弃——菜单标签已用静态缓存根修(11a2b9bb6),同族 badge
   (滚动偏移指示)与 preedit 自绘覆盖层未修不可见,同款缓存模式
   补齐;
3. **#18 a2r 块尾分号**:handler 内 `if cond { api.fn() }`(块尾值
   返回裸调用)a2r 发射缺 `;` → rustc E0308;语句位置分号发射规则
   根修 + 语料;auto-term app.at 规避注记解除作跨仓协调记录。

非目标:菜单命中/载荷/中断链(015 已交付且实机全通);主题跟随
pal_fg 语义(018 契约维持);badge/preedit 的功能增强(只修可见性)。

## 1. 目标

- G1 **master 门复绿**:terminal_pixel 3/3 绿,且归因工件在案
  (漂移维度定案 + 稳定复现配方或受控渲染方案,二选一落地);
- G2 **badge/preedit 可见**:滚动偏移指示与 IME 组合串自绘覆盖层
  渲染进像素产物(016 偏离基线法同款验证);
- G3 **a2r 语句发射正确**:if 块尾值返回调用发射含 `;`,既有语料
  零回归;auto-term 侧规避解除的协调记录在案。

## 非目标

- 菜单命中/载荷/中断功能链(015 已交付);
- pal_fg 主题跟随语义(018 D10 契约);
- badge/preedit 功能增强(只修可见性,不动交互);
- a2r 其余变形类(F2 int 宽度/F3 可见性等 010 既有账);
- auto-term 仓任何代码改动(app.at 规避解除只记协调,随 auto-term
  下一计划/小改动执行)。

## 2. 架构方案

无新架构,三条修复线:

```
T-01 像素金样:terminal_pixel_tests.rs + 金样策略
  归因 → 二选一:(a)模拟器渲染受控化(字体钉死/关 AA);
           (b)比对策略稳健化(容差/关键区域断言)+金样受控再生成
T-02 badge/preedit:widget.rs 绘制段 → 段落缓存
  静态/状态缓存键 (text,width) → 强引用存活到 flush(菜单
  MENU_PARAS 同款,11a2b9bb6 先例)
T-03 a2r 分号:ui_gen/rust.rs 语句发射器
  语句位置块尾恒补 `;`(值消费位才可省)→ 语料 + 金样
```

约束:**全程 worktree 执行**(87eba67ab 用户裁定:master 零 WIP
代码;`.wt/lang-634/auto-lang`,branch plan-634-dev,按相位 fold)。

## 3. 技术栈

auto-lang 单仓:iced 0.14(wgpu/tiny_skia 渲染层 + iced_test
simulator)、cosmic-text 字体管线、a2r/ui_gen 语句发射器。
验证:nextest(terminal/pixel/menu 定向)+ cargo tf/tt 按触面。

## 4. 需求分析与背景调查

- **授权**:用户 2026-09-15 在 PLAN-015 merge 收尾后指示"到 auto-lang
  侧建立一个新计划"清偿三账(016 门环境归因、DEBTS #17 同族、
  #18 a2r);三笔账分别在 auto-term DEBTS #17(同族待修注记)、#18
  (处置节)与 PLAN-015 复审记录 R015-F2(merge 前置)中挂账。
- **既有地基(已核实)**:
  - 菜单标签根修先例 = MENU_PARAS 静态缓存(11a2b9bb6),iced_wgpu
    layer.rs:76 `downgrade()` + text.rs:469 `upgrade()` 弱引用语义
    已实证;
  - badge fill 在 widget.rs:703、preedit fill 在 :686(均为 draw
    局部段落,同根因);
  - a2r 块尾缺分号现场:ui_gen/rust.rs 块发射器(块尾语句省 `;`
    惯例);plan627 语料测试范式可复用(inline .at → 断言发射文本);
  - 像素金样:terminal_pixel_tests.rs `matches_image` 逐字节比对,
    baseline 首跑自建;selection/cursor 两金样 200% DPI 桌面持续红;
    bisect 定案非代码(f0dc16732 洁净树复红)。
- **漂移候选源(待 T-01 归因)**:字体缓存/字体集变更、ToDesk 远程
  会话、显示设置(DPI/缩放)、DWM/驱动状态。

## 5. 详细设计

| # | 改动 | 文件:符号(auto-lang) | 说明 |
|---|---|---|---|
| D1 | 金样归因+策略 | `src/ui/iced/terminal_pixel_tests.rs` + `test/ui/terminal_pixel/` | 有界调查:dump 实际帧 vs 金样逐像素差分定位漂移维度(字形/AA/尺寸);按定案落地修复(受控渲染或稳健比对+受控再生成),归因报告入 evidence/634 |
| D2 | badge 缓存 | `src/ui/terminal/iced/widget.rs` badge fill(:703 附近) | 段落缓存键 (text,width),静态 Mutex 单entry或小 LRU;强引用存活到 flush |
| D3 | preedit 缓存 | 同文件 preedit fill(:686 附近) | 同款缓存(preedit 内容动态,键含内容);不变式:缓存段落只增不引用悬垂 |
| D4 | a2r 分号规则 | `src/ui_gen/rust.rs` 块/语句发射器 | 语句位置块尾恒补 `;`(表达式位置才可省);新增语料(块尾值调用,plan627 范式);全量语料回归 |

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/widgets/terminal-iced-draw.md | 无 → draw 期段落强引用规则:fill_paragraph 排队 WeakParagraph,段落须强引用存活到 flush(静态/状态缓存模式);badge/preedit/菜单标签三处范式 | 015/634 两次实证的持久性 widget 绘制契约 | AC-02 |
| SD-02 | add | docs/specs/widgets/terminal-iced-draw.md(同文件节) | 无 → 像素金样环境契约:渲染环境敏感维度、受控配方(或容差策略)、漂移复现/归因配方 | R015-F2 定案+本计划归因结论 | AC-01 |
| SD-03 | modify | docs/specs/a2r-std/project.md | 语句发射:块尾语句省 `;` → 语句位置块尾恒补 `;`(仅表达式位置可省) | E0308 实证(#18) | AC-03 |

## 6. 测试设计

- 像素:terminal_pixel 3/3 绿;新增强断言(badge/preedit 偏离基线法
  或等价 snapshot 证据);
- a2r:新语料(if 块尾值调用)发射含 `;` 且 rustc 实编过;既有
  transpile 语料零回归(cargo tt 相关档);
- 回归:terminal 套件(nextest)、menu/金样不破;cargo tf 全量档;
- 实机(可选):rust 轨 badge(滚动后可见)+ preedit(IME 组合串
  可见)人工清单,证据入 evidence/634。

## 7. 验收标准

- AC-01 terminal_pixel 3/3 绿;归因工件(evidence/634)含漂移维度
  定案与稳定配方(受控渲染或再生成流程),master 门复绿;
- AC-02 badge/preedit 渲染进像素产物(偏离基线法同款验证),
  菜单标签回归不破(menu 单测+金样绿);
- AC-03 新语料:if 块尾值调用发射含 `;`;既有 tt 语料零回归;
  auto-term 侧协调记录(app.at 规避解除条件)在案;
- AC-04 全量门:cargo tf 绿 + 触面档(tt)绿;双仓影响面说明
  (auto-term 本计划零代码改动,仅跨仓协调记录)。

## 8. 执行步骤

- T-01 像素金样归因+修复(D1)——AC-01;优先级最高(master 门
  当前红);含归因报告工件
- T-02 badge/preedit 同族修复(D2/D3)——AC-02;依赖 T-01 的
  像素验证环境可信
- T-03 a2r 分号根修+语料(D4)——AC-03;独立可并行
- (跨仓尾注)auto-term app.at 规避解除——T-03 落地后随 auto-term
  下一改动执行,本计划只记协调

## 9. 复审记录

- 2026-09-15 stage:new · outcome:pass(ready for work;全程
  worktree 执行,按 87eba67ab 裁定) · next:/auto-plan:work。

## 10. 待澄清事项

- T-01 归因结论二选一路径(受控渲染 vs 稳健比对)为执行期裁定,
  以归因报告工件为准,rev 不增;若两案皆不可行(漂移不可控)，
  返回 needs_replan 重议金样策略。
- auto-term app.at 规避解除依赖本计划 T-03 merge 后在 auto-term
  侧执行(1 行),跨仓协调随 merge 收据记录。
