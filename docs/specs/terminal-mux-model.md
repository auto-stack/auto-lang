# 终端 Mux 模型(分屏/焦点/虚拟滚动契约)

## 范围

终端多路复用的模型契约:pane 树/矩形投影/焦点/虚拟滚动。来源
PLAN-022(SD-02,T-06 收口定稿 2026-09-19);矩形投影消费面见
auto-term app/src/back/db.at rects_recompute(BFS,槽序契约
1..6=pane/7..11=divider)。交互投递上篇:auto-lang
docs/specs/auto-lang/ui/design/overlay-interaction.md(浮层间归属)。

## 虚拟滚动契约(§V1.10)

1. **display_offset 单源**:滚动位置唯一真相在引擎;终端组件与
   iced scrollable 皆投影,不持独立真相。
2. **视图投影缓存**:画布高 = (rows + history) × CELL_H + PAD;
   scrollable 原生像素滚动(全程可滚,无半屏钳位),自绘条退役。
3. **坐标映射**(行量化,CELL_H=16):y=0 = 最旧(offset=history)、
   y = history×CELL_H = 贴底实时(offset=0);往返量化无损
   (`view_y_to_offset`/`offset_to_view_y`,半行 round)。
4. **读出写臂双臂**:draw 期观察 viewport → 行差值回灌引擎;
   build 期引擎 offset 变化 → scroll_to 程序化绑回视图。
5. **回声抑制 = 判别式**(T-06 用户实点修订,2026-09-19):bind 时
   登记期望回声位(px)与滚轮代数;observe 时**代数未进且落点贴合
   (±4px)**才算 scroll_to 回声(吞没对齐基线);代数已进(用户
   下一格滚轮先到)或错位超容差一律按真实增量回灌。旧一次性吞没
   把用户增量当回声吃掉——上翻一格即卡/下翻跳格实录,
   `echo_does_not_eat_user_wheel` 三段回归锁死。
6. **键入贴底**:输出/键入 → display_offset 归零(贴底实时),
   scroll_to 回声经抑制对齐,不回灌(防环路)。

## 已知限制

- 极快连滚时存在行量化粒度感(CELL_H 粒度,非缺陷);
- iced flex 零宽 Space 主轴失效怪异见 overlay-interaction.md KD 023-R8。
