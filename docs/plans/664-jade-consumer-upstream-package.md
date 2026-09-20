---
plan_id: PLAN-664
status: drafting
feature_name: jade-consumer-upstream-package（jade/auto-edit 消费侧上游提名单：menubar 快照回归修复 + strict 域词位勘定补全 + JSON/storage 值域 + P-15/P-16 报错化 + DEBTS 651 回填）
author: [zhaopuming（auto-down 会话代拟，待 auto-lang 侧会话实勘修订）]
created_at: 2026-09-20
updated_at: 2026-09-20
plan_revision: 1
current_step: 0
total_steps: 6

supersedes_spec_components: []
new_spec_components: []   # 待实勘后定（menubar 快照遍历语义若涉契约 → ui/design 面增量）
touched_goals: []

affects: [auto-lang（vm/codegen、ui 渲染快照面、DEBTS.md、docs/specs 注记面）]
---

# [PLAN-664] jade-consumer-upstream-package——消费侧上游提名单（五件合包）

> 来源：jade-garden L3 收口（PLAN-080 delivered，差异表后置面）+ auto-edit
> bootstrap（PLAN-001/002 delivered）两线消费侧的上游欠账合单。2026-09-20
> 用户裁定结构与分立（"比较多 → 独立计划"）。⚠ 本草案由 auto-down 侧会话
> 代拟——auto-lang 侧接手时须按惯例**起草前实勘**修订（T-00 即勘定任务，
> 部分缺口疑似已随近期提交部分落地，见 §4 表）。
>
> 消费方契约锚：jade 差异表（auto-down docs/plans/attachments/
> 079-l3-assembly-mode.md §8，D-05/D-11/D-12 等）+ auto-edit 041 矩阵
> 六失败实录（auto-down 会话四次 + 本包起草侧两次，39/5↔39/6 波动）。

## 0. 变更摘要

| # | 件 | 性质 | 消费方痛点 |
| --- | --- | --- | --- |
| U-1 | menubar 展开项快照回归修复 | **bug**（渲染器/vtree） | 041 矩阵六失败（MENUBAR_OPEN 条件分支不进 vm 快照遍历）；jade desktop menubar 同族 exe 可见=回归面特定于条件分支形态 |
| U-2 | strict 域词位勘定+补全（typeof 对象接收者 / Array.isArray 全域） | 勘定+补全（**部分疑似已落地**） | jade properties 编辑面差异 D-11（077 §6.1 在案） |
| U-3 | JSON/storage 值域勘定（json.encode/decode DSL 可用性 + storage.get str 域） | 勘定+收尾 | auto-edit graph settings 持久化差异 D-12；Plan 401 session KV str 域读回 |
| U-4 | P-15/P-16 静默死面报错化 | 健壮性 | P-15 跨项目 fn 导入静默死（079 E-8）；P-16 dep/导入撞名 link 期 Undefined symbol 无编译期告警（080 实录） |
| U-5 | DEBTS.md 651 矩阵 5 条回填 | **文档欠账**（五次提醒） | 651 交付时承诺的回填未落（DEBTS.md 现零 651 行，2026-09-20 实勘） |

## 1. 目标

1. 041 矩阵回 ≈48/2 口径（menubar 项入快照——U-1）。
2. jade 差异表 strict 域/值域两类升级前置解除或明确收口路径（U-2/U-3）。
3. 静默死面两类（P-15/P-16）至少编译期告警化（U-4）。
4. DEBTS 651 五条落册（U-5）。

**非目标**：fcose 力导布局、局部图谱 BFS（消费侧形态裁定，非上游）；
池版本锁定协议（方向一附款，另单）；651 矩阵本身的度量扩展。

## 2. 架构方案（待 auto-lang 实勘修订）

- U-1：渲染侧 vtree 快照遍历对条件分支（`if` 于 view 块内）展开内容的
  收录语义——修复面候选 = vm snapshot walk / menubar Popover lowering
  交互；**jade 对照**（menubar-menu DSL 元素族直建可见）供定位。
- U-2/U-3：词位现状勘定表驱动（见 §4）——已落地者补 e2e/文档锚，
  未落地者补全（跨 a2r/VM/vue 三轨一致性口径）。
- U-4：P-15 = fn 导入解析失败应报错；P-16 = dep/导入名与符号表撞名应
  编译期诊断（两处皆现为运行期/静默死）。
- U-5：从 651 归档计划/evidence 提取五条入 DEBTS.md。

## 3. 技术栈

auto-lang crates/auto-lang/src/vm/{codegen,native_catalog}.rs + ui 渲染
快照面 + DEBTS.md + docs/specs 相关注记；测试 = 041 矩阵复跑（auto-edit
侧，消费方验收）+ auto-lang 内部套件（cargo tf/tv 按面）。

## 4. 需求分析与背景调查（2026-09-20 auto-down 侧快勘——待修订）

**授权记录**：2026-09-20 用户裁定上游包独立成计划（与 bps 方向一分立）。
auto-lang 侧执行授权待用户向 auto-lang 会话发起。

| 勘定点 | 快勘结果（auto-lang master @3df7b21a2） | 待实勘 |
| --- | --- | --- |
| json 模块 | codegen stdlib 名单含 "json"（6288/7989 行）+ native auto.json.encode/decode（1900/1901） | DSL widget 域可调性/类型面；能否即解 D-12 |
| typeof | codegen:6400 "typeof 语义仅保留原始类型接收者" | 对象接收者排除是否仍成立；jade 需求面 |
| Array.isArray | codegen:9174 "补入——Array.isArray 走实例路径" | 落地完整度（vue 轨？strict 全域？） |
| P-15/P-16 | 无报错化迹象 | 报错化落点（编译期诊断面） |
| menubar 快照 | 回归在（041 六失败 ×6 次实测；quit 项波动） | vtree 遍历定位 |
| DEBTS 651 | 零行在册 | 五条原文提取源 |

## 5. 详细设计

### 规范增量

| delta_id | add/modify/retire | 目标文档 | before/after | rationale | acceptance |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add（候选） | docs/specs/auto-lang/ui/design/（快照遍历语义节，位置待实勘定） | 无 → 条件分支子树快照收录语义 | U-1 契约化 | AC-01 |

（U-2/U-3 词位若勘定需补 → 追加行；U-5 为 DEBTS.md 非规范增量。）

## 6. 测试设计

- U-1：auto-edit 041 矩阵复跑（消费方）≈48/2；auto-lang 侧最小复现件。
- U-2/U-3：三轨一致探针（jade gallery twin 形态可复用）。
- U-4：报错化负例（导入死/撞名 → 编译期诊断）。

## 7. 验收标准

| ID | 可观察行为 | 验证方法 |
| --- | --- | --- |
| AC-01 | 041 矩阵 menubar 六项全过（≈48/2 口径） | auto-edit 侧 desktop_mcp.py 复跑 |
| AC-02 | strict 域/值域勘定表定稿：各缺口"已落地（锚）/补全（提交）"二态明确 | 勘定表 + 补全面套件绿 |
| AC-03 | P-15/P-16 负例得编译期诊断（非静默/运行期死） | 负例探针 |
| AC-04 | DEBTS.md 含 651 五条 | 文件核对 |

## 8. 执行步骤（auto-lang 侧接手后按实勘修订）

- **T-00** [调查] 五件勘定（§4 表逐项实勘 + 勘定表定稿）。→ AC-02
- **T-01** [改] U-1 menubar 快照回归修复 + 最小复现件。依赖：T-00。
- **T-02** [改] U-2/U-3 勘定后补全面。依赖：T-00。
- **T-03** [改] U-4 报错化 + 负例。依赖：T-00。
- **T-04** [文] U-5 DEBTS 回填 + SD-01。依赖：T-00。
- **T-05** [验证] 消费方联测（041 矩阵 + jade 侧门抽样）。依赖：T-01..04。

## 9. 复审记录

- 2026-09-20 draft handoff（auto-down 侧代拟）：`stage: new |
  plan_id: PLAN-664 | plan_revision: 1 | outcome: pass（草案就绪；
  auto-lang 侧实勘修订后生效） | next: auto-lang 会话接手实勘 + 用户
  授权执行`。

## 10. 待澄清事项

| # | 事项 | owner |
| --- | --- | --- |
| Q-1 | 编号占位 664（.next-id 未见于仓根，663 在途）——接手时重扫两目录定号 | auto-lang 侧 |
| Q-2 | U-2/U-3 若勘定"已落地"，对应件收窄为文档锚——计划有界修订 | auto-lang 侧 |
