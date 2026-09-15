# PLAN-634 跨仓协调记录：auto-term DEBTS #17/#18 清偿回执

来源：auto-term `DEBTS.md` #17（同族 badge/preedit 待修）、#18（a2r 块尾
分号，处置节）与 PLAN-015 复审记录 R015-F2（merge 前置）。本计划清偿
auto-lang 侧根因；auto-term 仓**零代码改动**，解除动作随其下一改动执行。

## #17 同族 badge/preedit（本计划 T-02 清偿）

- auto-term 挂账：「badge(滚动偏移指示)与 preedit 自绘覆盖层同为 draw
  局部段落,同根因不可见——归属 auto-lang widget 顺手项,修法同款(缓存
  强引用)」。
- **清偿**：auto-lang worktree commit `e64e0f33f`（PLAN-634 T-02）——
  `PLAIN_PARAS` 静态缓存（键 (text,width)，封顶 64），badge/preedit 文字
  进像素产物经新像素用例正反双向实证（负验证=复现局部段落旧形态双红）。
- auto-term 解除动作（随其下一改动）：DEBTS #17 标题「同族 badge/preedit
  待修」→ 全清；可按 DEBTS #17 同款实机清单复验 rust 轨 badge（滚动后
  可见）+ preedit（IME 组合串可见）。

## #18 a2r 块尾值返回调用缺分号（本计划 T-03 清偿）

- auto-term 挂账处置：「归 auto-lang a2r 语句发射器修缮(块尾表达式依
  所在块的表达式/语句位置决定是否补 `;`),独立小项;修后回删 app.at
  规避注记」。
- **清偿**：auto-lang worktree commit（PLAN-634 T-03，`join_stmt_block`
  语句位置块尾恒补 `;`）——#18 主形态（`if cond { api.fn() }`）发射
  `{ get_lines(); }`，整件 rustc 实编过（i32 桩下旧发射必 E0308，新发射
  编译通过）。规范增量 SD-03 落 `docs/specs/a2r-std/project.md`。
- auto-term 解除动作（随其下一改动，1-2 行）：
  1. `app/src/front/app.at` .Menu 处理器尾部「() 型语句收尾规避」解除
     （59cc630 引入的规避恢复自然形态——块尾可直接写值返回调用）；
  2. DEBTS #18 处置节补记「已根修（auto-lang PLAN-634），规避回删」。
- 前置条件：本计划 merge 回 master 后生效（auto-term 的 auto-lang 依赖
  解析序走 env/主检出）。

## R015-F2 像素金样环境漂移（本计划 T-01 清偿）

- 015 复审 R015-F2（merge 前置）：master terminal 门 2 红归因修复。
- **清偿**：commit `52ca422a1`——金样门禁稳健化（语义断言硬门禁 + 金样
  审计制）+ 归因报告 `pixel-golden-drift-attribution.md`。
- auto-term 解除动作：无（auto-term 侧无此门禁;其 DEBTS #17 的环境注记
  「像素金样 2 红为独立环境漂移」随本清偿自然了结）。
