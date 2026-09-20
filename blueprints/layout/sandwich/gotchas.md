# Gotchas — layout/sandwich

## 1. 三条高度红线：fill 列禁 justify-center/end / items-stretch 行须有确定高 / 滚动归 overflow

**wrong**: 在骨架的 col/row 上加 `justify-center`/`justify-end`（fill 列高度
让渡塌缩，663 家族）；把弹性主行写成没有外部定高约束的 `items-stretch` 行
（655 StretchLine 语义：`flex-1` 列内直接子位转写 + `items-stretch` 行自身
须有确定高——df90a448b 之前手写即塌）；让内容高反向决定骨架（内容撑开
主区而非 overflow 裁剪）。

**why**: 041 回归实证（主行塌缩成内容高）与 663 视口边界家族塌缩同根：
高度语义的责任划分——骨架供给确定高（h-screen/h-8/h-6 + flex-1 链），
内容区只许在 `overflow-hidden`/`overflow-y-auto` 内部滚动。

**right**: 骨架样式不改；消费方在 content 出口内部自行分栏时用
`row flex-1 items-stretch`（4xx 分栏样板），需要滚动把 overflow 放在
自己的内层容器上。

## 2. VM 快照断言：snapshot v2(rendered) 可见子树，root 投影仍可叠加

**wrong**: 断言只依赖旧口径"bp 子件子树对 MCP 快照不可见"（F-1）而放弃
内容断言；或反过来只信子树 needle 而不设 root 标记。

**why**: PLAN-665 T-00 实证修正：本仓 snapshot v2(rendered) 已完整暴露
bp 子件渲染子树（col>row>scrollable>…逐层可 needle）；但快照契约在演进
（664 menubar-snapshot 契约同期），单一断言面有漂移风险。

**right**: 消费断言双保险——root 单元标记行 + 投影字段为锚，slot 填充
文本 needle 为增强（bp-gate sandwich 单元形态）。

## 3. CJK 词表纪律：fixture 文案 ASCII

**wrong**: gate/测试 fixture 用 CJK 文案再对快照做行级索引或长度算术。

**why**: VM 字符串语义纪律（G-2，row_list 单元同款）：CJK 域索引算术
跨轨易碎。

**right**: fixture 刻意 ASCII 互斥词表（`sw toolbar`/`sw sidebar`/
`sw content`/`sw status`）。

## 4. sidebar 宽度 w-56 钉骨架内；组合建议

**wrong**: 为宽度差异 fork 包或提案 props 化（违反 Q4：结构性小差异走
slot，样式小差异走 token/recipe）。

**why**: 宽度是样式小差异；零 props 是本 bp 的纯度卖点（四口全结构性
出口，值 props 无处安放）。statusbar 文本形态已有 `layout/status-bar`
（bp 组合优于 props 重复）。

**right**: 宽度差异走宿主 `style:`/token 覆盖；确有跨 app 收敛需求时先
做变体提升评审（spec `promotions:` 提案），或消费方 fork 并登记。
