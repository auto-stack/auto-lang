# 043-canvas-paint — 自由画板 SPEC(Plan 563)

> canvas 元素能力样板:状态驱动画布 + pen 三件套事件。经典 Paint 的
> 流线模式地基(与 031-paint 像素模式互补;031 双模式升级另立 plan)。
> 编号说明:计划文本写 031-canvas-paint,但 capability-tests 轨道 031
> 已被 dyn-component-watch 占用,按空号顺延 043。

## 功能清单

- 自由绘画:按下起笔 / 按住走笔(≤30Hz)/ 抬起或**移出画布**收笔
  (双端统一语义,Plan 563 T1d)。
- 工具:pencil / eraser(v1 eraser = 背景色笔画,不真挖除)。
- 颜色 5 色 + 线宽 3 档(2/4/8);undo(弹尾笔)、清空。
- 持久化:storage `canvaspaint.strokes.v1`。

## 场景数据契约(canvas 元素核心规约)

- **状态即画布**:`scene: .strokes` 绑定**状态前缀**,引擎读
  `<前缀>_pts` / `<前缀>_meta` 平行字符串双表(B12 规避形态,028 先例):
  - `strokes_pts = ["x1,y1|x2,y2|...", ...]` 每笔画点列,`|` 分隔点,
    `,` 分隔坐标(逻辑坐标,`coords` 声明值域);
  - `strokes_meta = ["#rrggbb,width,eraser", ...]` 颜色, 线宽, eraser 位
    (`"0"`/`"1"`;缺省 `#111827,3,0`,宽容解析)。
- **渲染 = 纯函数**:双端各自独立实现(vue `<canvas>` 2D context /
  iced `canvas::Program` Path stroke),不共享绘制代码;共享映射规约:
  逻辑→px 按 extent 线性缩放、lineCap/lineJoin round、**单点笔画 =
  直径线宽圆点**(零长线 round cap 不可见)、eraser = clear 色(缺省白)。
- **pen 事件**:`onpenstart(x,y)` / `onpenmove(x,y)`(仅 pen-down 期,
  底层门控 + 33ms 时间闸)/ `onpenend(x,y)`(画布内抬起或出界收笔;
  出界坐标 clamp 到值域边界)。
- 与 chart/diagram 家族互补:svg 家族 = 派生几何 → svgdoc 光栅化
  (结构化图形);canvas = 状态点列 → 逐帧直绘(自由笔迹/高频流)。

## VM 变更纪律(031 SPEC 同款,B12 家族)

- pen 事件/undo 一律**本地构建新表 → 整体赋值**(运行时 str 状态列表
  禁止下标写);penstart 追加走 push(031 undo_stack 先例)。
- 走笔重绘 = 全量重建尾项(每 move 一次全表拷贝;几百点 × 30Hz 冒烟
  可跑,性能面 v1 不设限)。
- storage 序列化:`join("~")` 两区以 **`;`** 分隔(点列内含 `|` 不能用
  `|` 连接;**色值 `#hex` 里的 `#` 不能作分隔符**——T8 实测踩坑;点列/
  元数据内均无 `~` 与 `;`)。
- **split 产物整体赋值塌缩**(B12 家族,P553-D1 同族):`split` 产物列表
  直接整体赋值后状态读取端拿到内部 id(`[-112]` 形态,T8 实测)。绕法:
  **清空后逐笔 push**(状态列表 push 已证可靠)——Load 走此路,往返绿。

## 双端注记

- 出界收笔坐标:iced = 最后已知界内点, vue = clamp 边界点——语义等价
  近似(边界附近收笔),对拍不受影响(播种状态驱动)。
- devicePixelRatio:vue 端重绘含 dpr 缩放(高分屏清晰);iced 端
  bounds 即物理像素(框架内建)。
- 颜色支持 CSS hex(#rgb/#rrggbb);坏色值回退黑(iced)/忽略(vue
  ctx 默认黑)。

## 测试

- `tests/desktop_mcp.py`:VM 轨 MCP 断言(pen 事件 → strokes 状态;
  553 harness 经验:stdout DEVNULL、交互组间重取快照)。
- 双端截图对拍:播种固定 strokes → vue(playwright)/iced(autoui_
  screenshot)一致性(scratch/p563/ 归档)。
