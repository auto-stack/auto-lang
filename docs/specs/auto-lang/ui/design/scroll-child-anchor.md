# 子内容锚点（scroll-anchor / auto.scroll.to_child）双端合同

> PLAN-781 交付。接口冻结合同全文（含修订史 r1-r6）见
> `docs/plans/reports/781-interface-decision.md`（归档后于
> `docs/plans/archive/` 对应附件）；本文为持久层规范。

## 公共 API

### DSL 层（双端同形）

- 子元素属性 `scroll-anchor: "<key>"`：非空字符串，可挂任意元素（DSL 级
  通用属性——vue 生成器 `UNIVERSAL_PROPS` 与 schema 校验同位登记）。
  pane 归属 = **包含它的最近 ordinary scroll-pane**。
- key 作用域 = pane 内；不同 pane 同名 key 不串线（寻址二元组 =
  (controller handle, key)）。
- 同 pane 内重复 key：后注册覆盖（文档序最后一个生效）+ stderr 告警；
  不失效页面。
- 无 controller 的 pane 上的 scroll-anchor：告警 + 无操作。
- managed content / 虚拟列表 / terminal 锚点：不支持（显式 unsupported）。

### Native 层（id 9922/9923；既有 9900-9905 offset 族零改动）

| id | canonical 名 | 签名 | 语义 |
|---|---|---|---|
| 9922 | `auto.scroll.to_child` | `(handle str, key str) -> str` | 入队回执（**非完成证明**）：`"queued"` / `"error:<reason>"`；reason ∈ `bad_handle`（仅空串——任意非空串为合法句柄）/ `empty_key` / `unsupported` |
| 9923 | `auto.scroll.anchor_state` | `(handle, key) -> str` | `"pending"` / `"applied"` / `"invalid:<reason>"` / `"unknown"`；reason ∈ `not_found` / `not_bound` / `unsupported` / `unmeasured` |

**调用边界（PLAN-794）**：`to_child/anchor_state` 调用边界迁
`ScrollHostProvider`（core shim 编组 `queued`/`error:<reason>`/`pending`/
`applied`/`invalid:<reason>`/`unknown` 字符串）；781 状态机（queued≠完成、
managed unsupported 入队即拒、换绑代际、gen=0 采当前绑定、未预热/无采集
pending 晚一拍、not_found/not_bound 一次性、后写覆盖重新 pending、applied
不驻留）与完成 oracle（真实 offset 变化+目标正面积交集）原样留在
controller.rs/renderer——**零语义迁移**。

完成证明 = `autoui_scroll_state` 读出 offset 变化 + `autoui_visibility`
目标正面积交集；**禁止**仅以 progress 或入队回执证明。

## 状态机与代际

`queued → pending →（applied | invalid:<reason>）`，一次性消费：

- 入队于任何绑定之前（Init 先发目标）的请求**采用当前绑定**（首次
  bind 的 0→1 自增不是换绑）；入队时已绑定、随后换绑的请求 →
  `invalid:not_bound`（不落到新内容）。
- key 解析基于**当帧**锚点注册表；key 消失 → `invalid:not_found`
  （一次性失败不重试）。applied 后用户滚动/视图重建不被旧请求拉回。
- 同 (handle,key) 重复入队 → 覆盖旧状态（重新 pending）。
- **身份编码（F-781-R01，阶段 D 修订）**：锚点身份 = 不透明序号
  `scrollanchor:{seq}` + build 期注册表（seq→(pane,key) 查表，collector
  回传 rect+seq）——冒号歧义拆分与 Debug 字符串取值两类面消除；重注册
  驱逐旧 seq、stale miss 跳过、dup last-wins。标准 `scroll_controller()`
  句柄（`@scrollctl:N`）与任意含 `:`/引号/反斜杠/CJK 的 key 往返无损
  （单测锁定）。

## nearest 定位（冻结公式，双端同源）

输入：目标 content-space rect `(ty, h)` + pane 测量 `(offset_y, vh, content_h)`：

```
h > vh（高于视口）        → target = ty            （顶对齐，起始区域可见）
ty + h > offset_y + vh   → target = ty + h − vh   （底对齐）
ty < offset_y            → target = ty            （顶对齐）
否则                      → 不动（已正面积相交）
clamp [0, content_h − vh]
```

单轴 Y；X 轴/双轴 pane 显式 unsupported。

## Vue 轨实现语义（镜像而非复刻）

- `scroll-anchor` → `data-scroll-anchor` attr（静态 attr / 动态 v-bind）；
  helper 族并入 scroll_ctl 块（`scroll_to_child`/`scroll_anchor_state`
  蛇形直通 + `__sctl*` 内核）。
- 滚动原语 = **pane 作用域公式直算**（`scrollIntoView` 会级联滚动全部
  可滚祖先——违反跨 pane 隔离；决策报告 r5）。
- 无 pending 窗口：`.Init` → onMounted 后回执即生效；pane/key 缺席 =
  一次性 not_bound/not_found。
- managed 桥判据 = pane 内 `data-scroll-managed` 标记
  （`scroll-test-content` 发射）。
- **最近 pane 归属（F-781-R02，阶段 D 修订）**：plain/shadcn 双臂**全部**
  scroll-pane 元素（含无 controller 者）发射 `data-scroll-pane="1"` 所有权
  标记；`scroll_to_child`/`__scrollCtlEl` 改最近 pane 归属过滤
  （`el.closest('[data-scroll-pane]') === pane`）+ 全量遍历
  `getAttribute()` 属性比对（弃 CSS selector 拼接——任意合法 key/句柄
  结构性安全）。嵌套同名 key 零串线；仅内层有 key → 外层 `not_found`；
  内层 managed 不误伤外层。

## 验证面

- 单测：`cargo t scroll`（生命周期/nearest/builder 包装）、`cargo t plan781`；
- 双端几何驱动：`examples/capability-tests/p781-scroll-child-anchor/`
  （`tests/vm_probe.py` 41 项 + `tests/vue_probe.mjs` 43 项，V-01~06 全矩阵）。
