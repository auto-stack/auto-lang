# Gotchas — layout/gallery-shell

## 1. aside h-full 必写；root 双界链不拆

**wrong**: 把骨架 `aside` 的 `h-full` 删掉，或把 root col 的
`w-full h-full ... overflow-hidden` 改成内容自适应。

**why**: PLAN-625 T-04 实证——aside 无显式高度类时，sidebar_provider 的
`h-full` 在 iced 布局塌缩为 0（Row 交叉轴无 CSS stretch 语义），整棵
子树在树中存在但零像素绘制；663 视口契约同族（scroll 双界
`flex-1 min-h-0` 是内容滚动的最小条件）。

**right**: 骨架样式不改；满窗由消费方根决定（外层 `h-screen
overflow-hidden`，嵌套固定高单元 `h-full` 同语义）。

## 2. item schema 全键书写；循环变量事件走 msg 带参

**wrong**: items 元素缺键（如漏 `badge`）或在 onclick 里用 lambda 捕获
循环变量（`() => { .Select(item.id) }`）。

**why**: 缺键字段访问 = VM 硬错（filetree gotcha#3 同型）；lambda 捕获
循环变量是 handler 合成不支持形态——合成期 "Undefined variable" →
poisoned export（PLAN-625 T-06 实证）。

**right**: schema `{ id, title, category, badge }` 全键书写（badge 空串
= 不渲染）；点击上抛写 `onclick: .Select(item.id)`（027 OpenItem
惯用法），宿主经 `on_select` msg-ref 接收。

## 3. 设置弹层：非受控 popover；VM 断言先触发再 needle

**wrong**: 给 trigger button 绑 onclick 翻自持 `settings_open`（与
popover 内建开合双轨打架）；或 VM 快照直接 needle 弹层内文案。

**why**: popover 开合由 popover-trigger 原生管理（ui-gallery donor
形态，Plan 422 anchored popover 双臂在册）；VM 臂弹层内容仅在触发后
进入渲染树。

**right**: trigger 不绑 onclick、不声明 open；VM 快照断言弹层内容
（Theme/Accent 按钮）需先派发 trigger 点击再 needle。Overlay fallback
（PLAN-536 T6）无需启用——popover 双臂维护中实证（P672）。

## 4. 主题态归宿主，bp 只镜像

**wrong**: 在消费方给壳传字面量 `dark_mode: true` 后又期望弹层切换
生效；或在 bp/gate fixture 里直接改壳内状态。

**why**: 契约 #3 平台服务禁 bp 私有主题态副本；Plan 458 约定
dark_mode 声明即由宿主侧 codegen 翻转（vue html.dark / VM iced 种子），
壳只渲染 active 态并经 action 上抛意图。

**right**: 宿主 model 持有 dark_mode/accent_color 单例，值面下传
props；`SetTheme/SetAccent` 在宿主 `on{}` 落地改 model。

## 5. on_* msg-ref 形参匹配；搜索 value 双向靠壳内回写

**wrong**: 宿主绑 `on_select: .OpenItem` 但 OpenItem 无参；或删除壳内
`search_query` 回写以为 value 绑定自带双向同步。

**why**: msg-ref 回调按载荷形参匹配（TreeView PLAN-037 T3 契约）；
input `value:` 绑定在 VM 臂不隐式回写 model——壳在 `.Search(q)` 内
回写 `.search_query` 再上抛（受控显示一致）。

**right**: 宿主四个 msg 全带 str 形参：`Select(str)/Search(str)/
SetTheme(str)/SetAccent(str)`；壳内 oninput → `.Search` 链路不拆。
