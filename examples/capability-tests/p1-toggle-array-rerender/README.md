# p1-toggle-array-rerender(Plan 011/os-config ③ 复现工程,RED)

## 症状(RED,确定性复现 6/6)

两个 widget 消费同一 store 数组字段(`view_groups`)时:
- App(根)以 **button 循环**渲染该数组(卡片体:button+key+嵌套 row/col/span);
- SidePanel(子)对同一数组做 `if g.open` 条件渲染成员。

子 widget 内 press → handler whole-replace `view_groups`(探针实证 handler
执行、模型更新全部正确:expanded=[]、重建投影 open=false),**但 SidePanel
的 `if g.open` 条件子树不重渲染**(旧成员持续显示,快照陈旧)。

## 二分数据(全部 vm 轨 MCP 探针)

| 形态 | 结果 |
|---|---|
| v1 单数组 whole-replace + if 翻转(单消费者) | 🟢 GREEN |
| v2 双写(expanded+view_groups)+ 循环重建(单消费者) | 🟢 GREEN |
| v3 App 文本循环 + SidePanel 条件(双消费者,纯文本) | 🟢 GREEN |
| v4 = v3 + 按钮在子 widget | 🟢 GREEN |
| **v5 = v4 + App 循环体含 button(key/onclick/嵌套结构)** | 🔴 **RED 4/4** |
| v5 去 key | 🔴 RED(key 非触发) |
| v5 去 onclick | 🔴 RED(事件参数非触发) |

触发面:**第二消费者的循环体含 button 元素**(交互元件登记/身份分配与
第一消费者的条件子树失效互相干扰之嫌疑)。press 后无任何错误日志
(-D 亦无)——静默失效,需在 DynamicComponent::update/view 与 store
写路径插桩定位(与 Plan 419/423/443 的重渲染/RC 史一脉相承)。

## os-config 侧映射

plan010 T18 概要页(app.at 概要卡循环 key: m.id/button)即 v5 形态,
Sidebar 成员条件渲染自此失效(e2e-vm「group collapse」断言 RED)。
移除 App 级 view_groups 循环(P2 实验)即恢复。
