---
plan_id: PLAN-512
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: examples-desktop-batch-2
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-512] examples 桌面化批二——fit 动态重测机制 + 剩余 20 示例迁移

## 变更摘要

承接 506（批一已归档，顺产批二范围清单）与 P504-2 债务。两部分：

1. **机制增量——P504-2 fit 动态重测**：现 `window: "fit"` 为**首帧一次性
   测量**（011 切 Scientific 模式内容增高后窗口不跟随，底部 `=` 键裁剪，
   实机证据在案）。本期落"内容尺寸变化 → 重测 → 窗口跟随"：view 重建后
   重测，尺寸差超阈值（滞回）才触发——虚拟窗（session 窗口矩形更新 +
   relayout 联动）与独立窗（iced resize）双路径，复用 504 S4/S5 测量管线。
2. **剩余 20 示例迁移**（506 顺产清单）：分两线——
   - **title-only 线**（内容充满型 app：chat/dashboard/video 等，纯补
     `title:`，零行为变化，快速批）；
   - **fit 线**（固定内容型小工具：逐个判内容形态 + 拆居中外壳 + 补
     `window: "fit"`，含 016-calendar 切月增高这类**动态重测的天然验证
     对象**）。
   逐示例判定表先行（T3 产出），交互型子集的验证前置 **P506-1 UAF**
   （038 Reveal VM RC UAF，master 预存，疑 511 线）修复确认。

## 目标

- **G1 动态重测机制**：`window: "fit"` app 内容尺寸变化后窗口跟随
   （阈值滞回防抖动）；011 Scientific 切换（P504-2 原始复现）与 016 切月
   双实证，`=` 键不再裁剪。
- **G2 title-only 线**：判定表归入该线的示例 pac.at 全部补 `title:`，
   桌面 chrome 正确显示，行为零变化。
- **G3 fit 线**：归入该线的示例逐个拆除外壳、声明 fit、实测收缩留痕
   （506 形态：尺寸数字 + 截图）。
- **G4 判定表成文**：20 示例逐个"内容形态（fixed-intrinsic / fill）/
   外壳现状 / 归线 / 验证口径"四列成表，回写本计划（复审对照物）。
- **G5 债务清偿**：P504-2 回写已清偿；P506-1 状态核对（修复则交互型
   验证放行，未修复则交互型验证项降级标注并留痕）。
- **非目标**：`window: "fill"` 等新取值（判定表如证需要，登记待澄清不
  夹带）；outproc 形态的尺寸提示协议消息（508 进程模型联动，归协议线）；
  043/044（485/488 新式示例，本就无此债）；示例功能改动。

## 架构方案

```
动态重测（复用 504 S4/S5 测量管线）:
  view 重建(view-dirty/revision 变) ──▶ 重测内容尺寸 ──差值>阈值(滞回)──▶
    虚拟窗: session 窗口矩形更新 + relayout 联动（既有 apply_layout 消费）
    独立窗: iced window resize 指令
  阈值滞回: 尺寸差 ≤8px 忽略 + 帧边界合并（防输入态抖动 resize 循环）
```

- **改动面**：`crates/auto-lang/src/ui/iced/renderer.rs`（重建后重测挂点）
  + `session.rs`（虚拟窗矩形更新链，504 替换 session.rs:1716 的同区）；
  examples 侧 = pac.at 与根容器外壳（Category A 形态）。
- **与在途计划**：507/508 均在执行且同碰 renderer/session——**轻交叠，
  后合者 rebase**（本期改动集中，hunk 级预期）；建议领取顺序上不阻塞。

## 技术栈

既有栈（504 测量管线 + session relayout + iced resize）。零新依赖。

## 需求分析与背景调查

（取材 506 归档件批二建议 + P504-2 债务条目 + 现场核验 2026-08-31）

- **506 顺产批二清单（20 个）**：001、004、005、006、007、013、014、015、
  016、017、018、019、020、021、022、023、024、025、028、042（p051/p493/p507
  为计划工作目录不计）。506 建议两线分批 + 注意 P504-2 动态高度债与
  P506-1 UAF。
- **P504-2 证据**：fit = 首帧一次性收缩；011 Scientific 切换增高不重测、
  `=` 键裁剪（实机截图在案）；候选后续"内容尺寸变化信号 → 宿主重测"即
  本期 G1。
- **504 样板资产**：`window: "fit"` 双路径（VM 独立窗首帧 shrink + 虚拟
  桌面窗测量值替代 60% 写死初始）；测量管线在 renderer/session，本期复用
  扩展不重造。
- **P506-1**：038 Reveal VM RC UAF（master 预存，疑 511 线回归）——交互型
  app（017 chat/019 video/020 music 等）验证前核对；511（池族修复线）状态
  执行期查。
- **排程**：507 已归档、508 execution_done（起草时的 session/renderer 轻交叠
  风险基本出清，后合者 rebase 压力小）；502/509/510 drafting/归档待领。
  本计划可并行领取。（2026-09-01 开工时更新）

## 详细设计

### 1. 动态重测（G1）

- 挂点：view 重建路径既有"fit 首帧测量"处扩为"每次重建后重测"；
- 滞回：|Δw|、|Δh| 均 ≤ 8px 忽略；同一帧多次重建合并（帧边界取末值）；
- 虚拟窗更新：改 `WmState` 窗口尺寸 → 走既有 relayout（相邻窗口让位
  语义与用户手动 resize 一致，不自造新布局分支）；
- 用户手动 resize 后的语义：**用户尺寸优先**——手动 resize 过的窗口
  （fit 已被用户覆盖）不再自动跟随（一次性锁定，记 per-window 标志）；
  待澄清③若定"可重解锁"再扩。

### 2. 判定表（T3 产出，G4）

四列：内容形态（fixed-intrinsic / fill）/ 外壳现状（居中外壳行数）/
归线（fit / title-only）/ 验证口径（收缩实测 or title 显示断言）。预期
fit 线 ≈ 001/004/005/007/013/015/016/042，title-only 线 ≈ 006/014/017–025/028
——以逐个扫描为准，不预设。

### 3. 迁移执行

- title-only 线：pac.at 补 `title:`（桌面 chrome 显示断言走既有装载测/
  MCP 快照）；零 .at 行为改动。
- fit 线：拆根容器居中外壳（`min-h-screen`/center 外层，506 对 012 的
  同型核查法）+ pac.at `window: "fit"` + 收缩实测留痕（尺寸数字+截图）；
  016-calendar 额外跑"切月 → 窗口增高跟随"断言（G1 双实证之一）。

## 测试设计

1. **T1 机制单测**：重测决策（阈值滞回/帧合并/用户锁定标志）纯函数单测。
2. **T2 机制集成**：011 Scientific 切换前后窗口高度断言（desktop_mcp/
   MCP 快照测）；016 切月同型。
3. **T3 判定表**：20 示例四列成文回写。
4. **T4 迁移验证**：title 线 grep+装载断言全绿；fit 线逐个收缩实测留痕。
5. **T5 回归**：批一 7 示例既有脚本不回归（504/506 行为零变化）。

## 验收标准

1. G1 双实证绿（011 `=` 键不裁剪 + 016 切月跟随），P504-2 回写清偿。
2. 20 示例全部归线处理完毕（title 齐 / fit 实测），判定表成文对账。
3. 批一 7 示例零回归；`cargo check -p auto-lang` 零警告；
   `cargo t ui`、`cargo t session` 不回归（机制改动门禁）。
4. P506-1 状态核对留痕（修复放行 or 降级标注）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **机制——重测决策纯逻辑**：`crates/auto-lang/src/ui/iced/renderer.rs`
   fit 测量区抽"重测决策"纯函数（阈值/帧合并/用户锁定）+ T1 单测。
   验证：`cargo t fit_remeasure`（或并入既有套件名）。
2. **机制——双路径接线**：虚拟窗矩形更新（`session.rs` 504 同区）+
   独立窗 iced resize + 用户 resize 锁定标志。
   验证：`cargo t ui && cargo t session`。
3. **机制实证**：011 Scientific / 016 切月前后窗口尺寸断言（T2）。
   验证：desktop_mcp/MCP 快照断言绿 + 截图留痕。
4. **判定表**：20 示例扫描成表回写本计划（T3）。
   验证：四列表成文。
5. **title-only 线**：按表批量补 `title:` + 装载断言。
   验证：grep 20 示例 pac.at title 齐 + 断言绿。
6. **fit 线迁移**：按表逐个拆外壳 + `window: "fit"` + 收缩实测留痕。
   验证：逐示例尺寸数字+截图（506 形态）。
7. **P506-1 核对 + 批一回归**：UAF 修复状态查证留痕；504/506 示例脚本
   全绿（T5）。
   验证：既有示例验证脚本 + 核对注记。
8. **收尾**：P504-2 回写清偿；spec 沉淀归 merge；健康检查；状态翻
   execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① fill 取值需求**：判定表若出现"既非固定内容也非现充满"的中间形态
  （如 006-hero-section），是否需要 `window: "fill"` 取值——默认不新增，
  登记形态后由用户裁定（可顺延批三）。
- **② outproc 联动**：508 进程模型下 fit app 走 outproc 时动态重测需
  协议"尺寸提示"消息（app→host，追加式）——本期仅进程内语义，协议线
  （Stage 6 后续或 508 增补）收口。
- **③ 用户锁定语义**：手动 resize 后锁定为 v1 裁定；"重解锁"（如双击
  标题栏恢复 fit）列为增强候选。
- **④ P506-1 UAF**：511 线修复状态执行期查——未修复则交互型示例
  （判定表标注）验证降级为静态断言 + 债务留痕，不阻塞本批。
- **⑤ 阈值参数**：8px 滞回为初值，T1 时以 011 实测微调后定稿。
