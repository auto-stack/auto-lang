# Widget 组合（同文件兄弟与 use 两形态）

> PLAN-095 T-07 交付（auto-musk-095-dev@e2da944e3）。本文是 widget 视图
> 引用另一 widget 在 VM 轨的权威解析规则。

## 两形态等价实例化

1. **同文件兄弟**：根文件内首个 WidgetDecl 之外的其他 WidgetDecl
   （兄弟）——根提取不再 break，兄弟一并提取注册进 WidgetRegistry 并
   并入 child_decls（handler 编入单 VM，与 use 导入子件同面）。
2. **跨文件 use 行**：`use module: WidgetName` 经 use 导入链注册
   （既有路径，不变）。

显式 use 与同文件兄弟同名时，use 导入后注册（覆写兄弟注册）——显式
声明优先。

## 未解析引用诊断

组件臂 registry miss（含折叠键兜底未命中）产生 stderr 可定位诊断：

```
[AURA-CHILD-MISS] component 'Name' not resolvable at this site —
declare it in the same file (sibling widget) or import via `use <module>: Name`
```

不静默降级为零节点；`<Name />` 文本占位保留为最后渲染面（诊断先于
占位）。

## 边界

- 传递性模块（孙组件经 use 的 use）的注册沿用 PLAN-507
  register_transitive_widgets 路径，不在本文重复。
- VM snapshot 工具的 raw_class 表在 hoist 场景存在样式错配（PLAN-095
  T-03 实测：运行时渲染正确、快照样式错配）——快照消费方以运行时
  截图核对样式，勿单信 raw_class。

## 验收锚

`g11_sibling_vm_tests`（进程内子树断言）+ auto-musk 095-evidence
t07 端到端快照（修复前零节点对照）+ t01 use 行对照快照。
