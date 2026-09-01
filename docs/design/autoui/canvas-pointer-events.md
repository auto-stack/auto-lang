# Canvas 交互 v2:通用指针事件原语与 chart 跟随交互设计

> Plan 499 M1 调研产出(2026-09-01)。前置:Plan 386(桌面协议/五通道)、
> Plan 484(chart 交互 v1:锚定 tooltip/竖带命中)、Plan 492(引擎三族);
> 设计输入:[diagram-components.md](diagram-components.md) §7.4/§7.5(统一面)。
> 本文档落定:mousemove 限频流原语、指针坐标语义、P-list 图元协议草案、
> donut 扇区-边命中同源、axisPointer 与动画双轨的实现方案。

## 0. 结论先行

1. **mousemove 流不需要新协议**——in-process 臂(独立窗/路线 A 桌面)沿既有
   `mouse_area → IcedMessage → decode_payload → VM handler` 通路,补一个
   **自定义包装 widget**(iced 0.14 `on_move` 闭包拿不到 bounds 尺寸,坐标换算
   必须在 widget 内做)+ **引擎侧限频**(30Hz + 量化去重);
2. **donut 逐扇区 hover 不需要 canvas 渲染先行**——"代码命中"与渲染后端解耦:
   全图单 mouse-area + 组件内极坐标命中测试 + 动态 svg path 重发,即解除
   svgdoc 静态限制(ECharts 模型:命中在代码不在 DOM)。canvas P-list 渲染桥
   (437 v2 决议)维持后置,本计划只落 **schema 草案**与坐标/命中约定;
3. **axisPointer 十字线走 svg 内动态 path**;每次移动 doc 串变化 → resvg
   handle 缓存(按整串 doc 键)miss → 重解析重栅格,小图 30Hz 可承受,
   由 M2 压测门禁验证;
4. **两进程(路线 B)widget 树输入注入不在本计划**——输入通道 `InputMsg::PointerMoved`
   已在协议中,当前消费方是编辑器 FrameSource(`editor_frame.rs:196`);
   通用 widget 树注入随 canvas 桥一并后置(挂 KNOWN-DEBT)。

## 1. 通用指针移动事件原语(mousemove 流)

### 1.1 DSL 面(mouse-area 扩展,非新元素)

```
mouse-area (coords: "560x300", onmouseenter: ..., onmouseleave: ...,
            onmousemove: .PointerMove) {}
```

- **`onmousemove: .Handler`**:事件臂(与 onmouseenter/onmouseleave/ondblclick
  同族)。Handler 声明两参 `(x: float, y: float)` 时接收逻辑坐标;
- **`coords: "WxH"`**:逻辑幅面声明(组件 viewBox 系,如折线图 `"560x300"`)。
  声明时事件参数 = 逻辑坐标;缺省时 = mouse-area bounds 局部物理 px
  (diagram 画布等无 viewBox 消费方的 raw 模式);
- schema `mouse-area` ElementDef 增注两属性描述(reactive props 表同步)。

### 1.2 视图层(View::MouseArea 扩展)

```rust
View::MouseArea {
    content, on_enter, on_exit, on_double_click, style,   // 既有
    on_move: Option<M>,            // M1 新增:闭包形态(逻辑坐标已换算)
    logical_extent: Option<(f32, f32)>,  // M1 新增:coords 解析结果
}
```

`on_move` 为 `Box<dyn Fn(f32, f32) -> M>` 形态(坐标由引擎层换算好再进闭包,
组件/DSL 层不感知屏幕)。未带 on_move 的 mouse-area 映射不变(零回归面)。

### 1.3 iced 臂:自定义 PointerArea widget

**为什么不能用 iced 原生 `mouse_area.on_move`**:其闭包签名
`Fn(Point) -> Message`(`iced_widget-0.14.2/src/mouse_area.rs:113`,
`cursor.position_in(bounds)` 后直接传入,`mouse_area.rs:358`)只给 bounds
局部 px,**闭包拿不到 bounds 尺寸**——px→逻辑换算无从进行。故新建
`crates/auto-lang/src/ui/iced/pointer_area.rs`(precedent:`popover.rs` 自定义
widget),单 widget 承载四臂:

- enter/exit/dblclick 转发内层 iced mouse_area 同款语义;
- **on_move**:`CursorMoved` 且 hovered 时
  `logical = local_px / bounds_size × extent`(extent 缺省时恒等 = px),
  随后**限频发布**;
- renderer 映射规则(`renderer.rs` MouseArea 臂):带 `on_move` → PointerArea
  widget;否则 → 既有 iced mouse_area(存量用法零改动)。

### 1.4 限频(引擎侧,VM 臂专属)

PointerArea widget State 持 `{ last_pub: Instant, last_pos: (f32,f32) }`:

1. **时间闸**:距上次发布 < 33ms(≈30Hz)则丢弃——目标采样率(计划目标 1);
2. **量化去重**:逻辑坐标量化到 0.5px 后与 last_pos 相同则丢弃(静止悬停
   零事件流);
3. vue 臂**不限频**:浏览器原生 mousemove(~60–125Hz)不经过 VM/RenderQueue,
   Vue 反应式更新自身可承受(ECharts 同款);双端契约是"handler 收到逻辑
   坐标",事件率差异为已记录的实现细节,非行为分歧。

in-process 模式下"Plan 386 RenderQueue 批量通道"退化为 update() 直派发
(单进程无协议跳);限频保护的是 **VM handler 执行 + view 重建节奏**。
两进程形态的批量通道消费挂 DEBT(§5)。

### 1.5 事件进 VM(既有机制,零新协议)

PointerArea 发布
`IcedMessage { widget, event: "{name}\u{1F}f\u{1F}{x:.2}\u{1F}f\u{1F}{y:.2}" }`
→ update → `on_with_input_for` → `decode_payload`(`dynamic.rs:1404`,
类型化 pipe-payload)→ handler 以 `(Float x, Float y)` 实参执行。
组件侧:`.PointerMove(x, y) -> { … }`,x/y 即逻辑坐标。

### 1.6 vue 臂:生成端内联换算

mouse-area 的 vue 发射(div + 事件属性)增 onmousemove 臂:

```html
<div @mousemove="e => PointerMove(e.offsetX / e.currentTarget.clientWidth * 560,
                                  e.offsetY / e.currentTarget.clientHeight * 300)" …>
```

(coords 缺省时直接 `e.offsetX/e.offsetY`。)引擎层(生成端)完成屏幕→逻辑
换算,与 §7.4 坐标语义条一致;组件代码两端同构。

## 2. 坐标语义(§7.4 增补输入③落定)

- 指针事件携带**组件局部逻辑坐标**(viewBox 系);
- **屏幕→逻辑换算在引擎层完成**(iced:PointerArea widget 按 bounds 归一 ×
  extent;vue:生成端内联 `offsetX/clientWidth × W`);
- 组件代码不感知屏幕/缩放:future canvas 容器的 pan/zoom transform 是其上
  一层映射(extent 语义不变,pan/zoom 时代码改写 extent 或由容器层叠加),
  组件命中代码零改动;
- 逻辑坐标原点 = mouse-area 自身左上角。组件按自己的布局常量换算
  (折线图 plot 区 `left=40..right=550` 等 Init 常量)。

## 3. P-list 图元协议(schema 草案,M1 交付)

charts `segs` 与 diagram render 记录的归一内部桥接协议(**非终点**——未来
通用 `canvas` DSL 容器/a2ui-composer 方向以其为基底,diagram-components.md
§7.4)。本计划不实现渲染桥(437 v2 决议后置),只定 schema 与坐标系约定:

```
P := path | rect | ellipse | text
path    { kind:"path", d:str, fill?:str, stroke?:str, sw?:f32,
          opacity?:f32, cap?:"round", join?:"round", hit?:HitRef }
rect    { kind:"rect", x:f32, y:f32, w:f32, h:f32, r?:f32, fill?, stroke?, sw?, opacity?, hit? }
ellipse { kind:"ellipse", cx:f32, cy:f32, rx:f32, ry:f32, fill?, stroke?, sw?, opacity?, hit? }
text    { kind:"text", x:f32, y:f32, text:str, size:f32, color?:str,
          anchor?:"start"|"middle"|"end" }
HitRef  { id:str }        // 命中记录锚:命中回调携带,charts=数据索引/diagram=边 id
```

- 坐标系:与组件 svg viewBox 同一逻辑系(§2);
- charts 现役 `segs` 记录 `{d, c, i}` 即 P-list path 的特例映射
  (`d→d, c→fill, i→hit.id`)——迁移是记录形态收敛,非语义变更;
- diagram render 段记录(`{kind,d,c,…}`)同构并入;
- 命中语义(§4)只依赖 `hit.id` + 组件几何常量,**不依赖渲染后端**——
  svg 轨与未来 canvas 轨共用同一命中代码。

## 4. 命中能力:donut 扇区与 diagram 边同源(§7.4 增补输入②)

**能力 = "逻辑坐标指针事件 + 组件内代码几何命中测试"**,与渲染后端解耦:

| 消费方 | 测试 | 几何输入(Init 预计算) |
|---|---|---|
| donut 扇区(M4) | is-point-in-annular-sector:`rIn² ≤ dx²+dy² ≤ rOut²` 且 `atan2(dx,dy)` 落累进角区间 | `cx,cy,rIn,rOut,angEnds[]` |
| diagram 边(未来) | point-to-polyline 距离 ≤ ε | 边折线顶点表 |

- donut 落地:全图**单** mouse-area(`coords` = svg viewBox 幅面,viewBox 带
  偏移时组件按 Init 常量换算)替换图例兼任命中区的 v1 形态;`.PointerMove`
  → 扇区索引 → emphasis(命中扇区 path 动态 `fill-opacity`/外扩重发,
  其余 downplay)——低频(逐扇区 enter/leave 级),svgdoc 重栅格可承受;
- 图例 hover 命中保留(双入口,行为不变);
- **svgdoc 静态限制就此解除**:限制的本体是"DOM 事件不可达",代码命中
  绕开它;canvas 渲染桥的价值回到性能/文本(§7.4 触发判据),不再承担
  交互前置依赖。

## 5. 两进程(路线 B)边界

- 输入通道 `InputMsg::PointerMoved { wid, x, y }`(`desktop_protocol/message.rs:541`)
  协议已备;当前消费方 = 编辑器 FrameSource(`editor_frame.rs:196`),
  **widget 树输入注入**(PointerMoved → 客户进程 iced 事件合成)未实现;
- 本计划 M2–M5 全部落在 in-process 臂(独立窗 + 路线 A 桌面);
- 路线 B 泛化挂 KNOWN-DEBT(与 canvas 桥同批偿还——两进程客户的 widget
  树渲染本身仍待 FrameSource 之外的一般化)。

## 6. axisPointer(M3 实现方案)

- **命中层重构**:折线图 N 条竖带 mouse-area(484 v1)→ **单**全图
  mouse-area(`coords: "560x300"` + `onmousemove: .PointerMove`);
  enter/exit 沿用(显隐门控);
- `.PointerMove(x, y)`:`i = clamp(round((x-left)/step))` 索引吸附
  (ECharts axisPointer line 对齐刻度语义)→ `cx = left + i×step`;
- **十字线**:svg 内动态 path(`d: .crossV`,竖线 `M cx 20 V 260`,
  可选水平线/cross 形态)——逻辑坐标直书,无覆盖层对位问题;
  竖带 shadow 形态(x 索引槽位 rect)为可选发射;
- **tooltip 跟随**:固定锚(`top-[28px] right-[60px]`)→ 跟随
  `left-[${clamp(cx)}px]`(动态类串,`w-[${slot}px]` 同款先例);
- resvg handle 缓存按整串 doc 键(`renderer.rs:4486`)→ 每次移动 miss 重解析;
  小图(≤40 path)30Hz 重栅格实测可承受(压测门禁,验收 2);
- 响应式边界(继承 v1 已知项):svg `w-full` 在窗口窄于 max-w 时缩放,
  覆盖层类(固定 px)不随动——坐标语义(§2)已免疫(命中/十字线随
  bounds),tooltip 定位在极窄窗下的偏移记已知边缘,不在本计划修。

## 7. 动画过渡(M5 双轨策略)

- **vue 轨**:CSS transition(tailwind `transition-*` 类直加在 emphasis 元素:
  tooltip 淡入 `transition-opacity duration-150`、donut 扇区
  `transition-[fill-opacity] duration-200` 等);
- **VM 轨**:数值插值,驱动源 = 组件 timer 原语
  `timer { AnimTick (every_ms: 33, when: .animating) }`(017-chat 先例,
  `when` 门控零空闲开销)——donut 扇区 emphasis 外扩/淡入、tooltip 淡入
  用状态插值表(`from/to/t0` 三元组,AnimTick 步进);
- **对齐口径**:观感相当(目检)+ 帧率基线对照,**非逐帧一致**(渲染管线
  不同,验收 2 的基线对照覆盖);
- 首版范围:opacity/位移类数值过渡;颜色过渡 vue-only(CSS),VM 不插值。

## 8. 压测与验收(M2 门禁)

1. **限频断言**(单元级):PointerArea widget 合成 125Hz CursorMoved 流 →
   发布率 ≤ 30Hz;静止悬停(同量化坐标)零发布;
2. **坐标断言**(单元级):bounds 400×214、extent 560×300、事件 px (200,107)
   → 逻辑 (280.0, 150.0) ± 0.5;
3. **e2e 节奏断言**(plan437 风格 in-process):持续 mousemove 下 handler
   状态写入次数/秒 ≤ 30,view 重建不塌帧(帧时长 P95 对照基线);
4. **svgdoc 重栅格**:axisPointer 移动流下无帧率塌陷(验收 2 基线对照)。

## 9. 非目标

- canvas Program 渲染桥(437 v2):维持后置;触发判据不变(svgdoc 序列化
  性能不达标/高频 .Tick/≥80 节点大图,diagram-components.md §7.4);
- 路线 B(两进程)widget 树输入注入(§5);
- pie/散点/水平条形等图类型扩展(484 已后置项,不因交互升级复活);
- chart 家族既有契约变更:props 追加式,既有用法零破坏。
