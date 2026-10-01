# MCP 合成指针坐标编码

> PLAN-095 T-05 交付（auto-musk-095-dev@919ade13a）。本文是 autoui_action
> drag/pen 合成坐标的权威编码规则；AutoVM 源码 server toolset
> （mcp/design/toolset.md）不承载此合同。

## 编码规则

- `__mcp_drag`（move 段）与 `__mcp_pen`（start/move/end 三段）坐标以
  **`Value::Float(x + 1e-3)`** 编码——与真实指针通道
  （mouse_area_move_arm）完全同一编码/解码口径（PAYLOAD_SEP typechar
  "f" → push_f32）。
- 禁止 Double 编码：合成/真实双通道位型分叉会在 float 形参上产生
  nanbox 位型错读（实证：Double 通道同 handler 收到垃圾 int
  -1062209585 族）。
- `+1e-3` 分数化保留（nanbox 整值丢标签防御，0.001px 不可见）。
- 动作顺序 Down→Move×n→Up；down 实参 = `$event` 冻结标记（handler
  声明 1 形参时成帧）；up 无实参（0 形参）。非坐标的 Double 消费者
  （ghost/colresize 状态写入）不属本合同，不受影响。

## 验收口径

坐标正确性用**类型化 float 状态断言**（容差 ≤0.5px），不用 moves 计数、
不用 float→字符串拼接读数（.at float→str 有独立位型缺陷，另行登记）。
验收锚：auto-musk canvas-runtime-probe coords 探针（小数/零/负值三点）。
