# Gotchas — data-display/row-list

## 1. row schema 缺键 = VM 硬错（全键书写纪律）

**wrong**: 构造 row 字面量省略 `badge`（JS 习惯省缺省字段）。

**why**: VM Obj 字面量为形状锁定类型实例，缺键字段访问 = 硬错（P614 实证；
filetree gotcha#3 同型）。

**right**: 全键书写 `{ label: str, badge: str }`；badge 无值传空串
（视图层空串即不渲染徽标，不靠缺键表达"无"）。

## 2. 行派生 fn 归消费方自备（.at 无跨文件导入）

**wrong**: 指望从骨架侧 import 行构造/过滤 helper，或在骨架里沉淀模块 fn
让消费方复用。

**why**: .at 模块间 import 缺位（074-sink-mode G-3）；行 fn 随消费方文件
各沉一份是既定模式（tab_file_stem 双件各沉先例）。

**right**: 行派生逻辑沉在消费方 widget 文件顶层模块 fn（Plan 367 P2-4；
PLAN-075 起 dep 通道同文件模块 fn 已可转译进 SFC——048 夹具），过滤/排序
编排沉消费方 watch 块（074-sink-mode 步骤 2）。

## 3. VM 字符串索引算术禁令（非 ASCII 域）

**wrong**: 在 VM 轨可达的行派生 fn 里对 CJK 文本做 `find`/`slice`/
`char_at` 索引算术（如截取标题前 N 字）。

**why**: VM 轨 `length` 是字符数、`find`/`slice`/`char_at` 是字节索引
（P614 探针实证，074-sink-mode G-2）——非 ASCII 域四者混用双轨不可移植。

**right**: 标题派生走 store title 权威或 ASCII 后缀域比较
（tabs_store `strip_ext` 纪律）；vue-only 的展示派生不受此限但须注记。

## 4. VM 轨消费方断言走 root 投影/内联行（子件子树不可见）

**wrong**: VM 轨把 RowListSkeleton 挂进组件后，靠 MCP snapshot/press 驱动
行交互断言。

**why**: vm 组件子树对 MCP 快照不可见（041 README「vm 组件边界」②），
组件行 press 静默崩溃（P614-C1/P618-D4 家族）。

**right**: VM 轨需要驱动行时在根视图内联行（消费方 for 循环 + 自备行 fn，
filetree gotcha#1 right 形态）；纯展示断言走 root 投影计数字段。
