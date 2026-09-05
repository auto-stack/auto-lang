# 031-paint — 像素画板 SPEC（Plan 553）

> AutoOS "画图"应用的 v1 形态：16×16 像素画板。真画布原语（连续笔迹）见
> Plan 553 待澄清①，不在本应用范围。

## 功能清单

- 工具四件：铅笔（点击染当前色）/ 橡皮（点击染回白）/ 油漆桶（连通同色区
  泛洪）/ 吸管（取格色为当前色）。
- 调色板 16 色 + 当前色块；undo/redo（快照栈上限 20）；清空 / 新建。
- 作品持久化：storage `paint.canvas.v1`（整串 `#rrggbb` ×256 以 `|` join）。

## 状态模型（单组件内聚，028/025 形态）

- **颜色表示**：颜色一律以完整 Tailwind 类串存状态（`" bg-[#ef4444] "`，
  前后各一空格）——字面量随 handler 源进 vue 生成码命中 Tailwind JIT 扫描
  （运行时拼接的任意值类不可见，T3 实测）；段间空格编进值本身，绕开生成器
  `${}` 段接 `+` 拼接丢边界空白的问题（gen App.vue 实证）；VM 侧同机制
  （028 launcher chip 先例）。
- `px`：256 长度**类串**平行列表（B12 规避——VM handler 对 Obj 数组字段读
  的失效面）。
- `cells`：handler 自建行对象 `{i, chip}`（view 侧读自建 Obj 数组已证可用，
  028 `ranked` 先例），每次变更后 `RebuildCells` 全量重建。
- 画布 grid 用**表达式 cols**（`cols: .canvas_cols` → 内联
  `grid-template-columns: repeat(16,…)`）——字面量 int 走 `grid-cols-N` 类，
  N=16 超 Tailwind 默认刻度(≤12)不生成（ui_gen/vue.rs 10289 注记；038 动态
  列同型）。
- undo/redo 栈：`px.join("|")` 快照串列表（044 `.join` / `Str.split` 实存）。

## 双端注记

- 拖画：v1 点击画（T1c——无按压门控的指针流，hover 刷不合画笔语义）。
- 格子着色经 style 插值 `bg-[<色>]`（028 chip 同机制，双端同源）。
- icon：lucide VM 闭集无 `brush`，取 `pencil`（T1b）。
- **VM 变更纪律（T6 实测）**：运行时 **str 状态列表禁止下标写**（`.px[i]=v`
  静默不生效——saved 翻转证分支执行、px 不变）；一律本地构建 → 整体赋值
  （028 `.recent = nr` 同型）。单格 `SetPx` 全量拷贝，泛洪 BFS 只算 seen 再
  单趟重建。
- **VM 债 P553-D1**：split 产物重建的**全同串列表**整体赋值后塌缩为单元素
  （uniform 快照恢复 px len=1；`+""` 新鲜句柄无效；逐格 SetPx 恢复触预算
  截断）。影响：全同盘的 Redo/Load 恢复（desktop_mcp T6-redo 按债引用式
  SKIP，013 audit-B12 同型惯例）。混合内容快照不受影响。
- **MCP harness 经验**：VM app 首帧前日志量大，子进程 stdout 必须 DEVNULL
  （PIPE 塞满致写阻塞、UI 永不渲染）；重渲染改变 vnode 哈希 id——每次交互
  组前必须重取快照重发现元素。

## 测试

- `tests/desktop_mcp.py`：VM 轨八组断言（结构/初始态/染格/橡皮/泛洪/
  撤销重做/吸管/存取），30 PASS + 1 SKIP（P553-D1）。
- vue 轨：playwright 冒烟（scratch/p553/ 截图族——手画/泛洪/撤销/存取像素
  校验）。
