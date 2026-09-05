# 031-paint — 像素画板 SPEC（Plan 553）

> AutoOS "画图"应用的 v1 形态：16×16 像素画板。真画布原语（连续笔迹）见
> Plan 553 待澄清①，不在本应用范围。

## 功能清单

- 工具四件：铅笔（点击染当前色）/ 橡皮（点击染回白）/ 油漆桶（连通同色区
  泛洪）/ 吸管（取格色为当前色）。
- 调色板 16 色 + 当前色块；undo/redo（快照栈上限 20）；清空 / 新建。
- 作品持久化：storage `paint.canvas.v1`（整串 `#rrggbb` ×256 以 `|` join）。

## 状态模型（单组件内聚，028/025 形态）

- `px`：256 长度颜色串列表（平行字符串列表规避 B12——VM handler 对 Obj
  数组字段读的失效面）。
- `cells`：handler 自建行对象 `{i, chip}`（view 侧读自建 Obj 数组已证可用，
  028 `ranked` 先例），每次变更后 `RebuildCells` 全量重建。
- undo/redo 栈：`px.join("|")` 快照串列表（044 `.join` / `Str.split` 实存）。

## 双端注记

- 拖画：v1 点击画（T1c 探针——无按压门控的指针流，hover 刷不合画笔语义）。
- 格子着色经 style 插值 `bg-[<色>]`（028 chip 同机制，双端同源）。
- icon：lucide VM 闭集无 `brush`，取 `pencil`（T1b）。

## 测试

- `tests/desktop_mcp.py`：五断言组（染格/泛洪/吸管/undo-redo/存取），vue+vm 双轨。
