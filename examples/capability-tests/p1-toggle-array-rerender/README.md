# p1-toggle-array-rerender(Plan 011/os-config ③ 回归金丝雀,🟢 GREEN)

> **2026-08-29 定性反转**:本 canary 早期记录的「RED 确定性复现(6/6,v5 4/4)」与
> 「`if g.open` 条件子树不重渲染」**均为判定假阳性,VM renderer 无 bug**。
> 本文件据此改写;canary 保留作双消费者条件渲染的回归金丝雀(判定见下)。

## 结构

两个 widget 消费同一 store 数组字段(`view_groups`):
- App(根)以 **button 循环**渲染该数组(卡片体:button+key+嵌套 row/col/span,
  无条件展示 `g.label`/`g.m1`);
- SidePanel(子)对同一数组做 `if g.open` 条件渲染成员(col+key+text m1/m2)。

子 widget 内 press → handler whole-replace `view_groups`。

## 当时的「RED」是怎么来的(教训)

判定用**全快照文本包含**(如 `"alpha" in snapshot` / button label 行匹配)。
App 卡片**无条件**展示 `g.m1`(alpha/gamma),所以无论 SidePanel 的条件子树
是否正确移除,全快照永远能匹配到这些词——RED 是**第二消费者的合法文本**
击穿了**不分区**的断言,与 os-config e2e-vm `navVisible` 假阳性同构。
v1→v5 的「二分」实为判定语义随形态漂移,不构成 VM 缺陷证据。

正确判定:**按消费区切片**。SidePanel 段内成员文本应随 `if g.open`
消失/重现;App 卡片段恒在。按此判定(2026-08-29,debug 探针构建与
master 构建双验):

| 步骤 | 期望 | 实测 |
|---|---|---|
| 初始(g1 open) | SidePanel 有 alpha/beta | ✅ |
| press#1(g1 收起) | 成员消失(vnode 移除) | ✅ |
| press#2(g1 重开) | 成员重现 | ✅ |
| 4 连快速 press | 终态一致 | ✅ |

[P011]/[P011B]/[P011W] 三层探针(dynamic.rs / aura_view_builder.rs /
engine.rs / vm_bridge.rs)全程实证 handler 执行、dirty 传播、重建求值、
builder 双消费者读到同一新鲜 elems——与「VM 无 bug」结论一致。
(探针自身曾有一处 downcast 认错 `object_data::ObjectData`,已修正为
`types::ObjectData`,见 vm_bridge.rs `debug_obj_snapshot`。)

## os-config 侧映射(③ 真正根因)

plan010 T18 概要页(app.at 概要卡循环 key: m.id/button)上线后,
e2e-vm「group collapse」断言 RED——根因是 `navVisible('Roles')` 对**全
快照所有 button 的多行 label** 做行匹配,概要卡 button
`"🎭\nRoles\nAgent roles…"` 恒命中。侧栏折叠本身一直正确(折叠后侧栏
vnode 从树中移除,概要卡 vnode 原样保留,双构建实证)。修复:断言改
计数法(折叠恰好 −1/重开恢复),见 auto-os-config `scripts/e2e-vm.mjs`
`navCount` 与 plan011 文档 2026-08-29 条目。

## 判定方法(复跑)

vm 轨 MCP:`auto run -r vm` 后 `autoui_snapshot`,在快照中定位
`Toggle g1 (child)` 按钮所在的 SidePanel col 子树,**仅在该切片内**断言
alpha/beta 的出现与消失;press 用 `autoui_action` 于该按钮。
