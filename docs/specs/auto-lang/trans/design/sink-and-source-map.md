# Sink 输出协议与 source map

> 范围：`trans.rs` 中 `Sink` / `MultiSink` / `SourceMapEntry` 的输出缓冲与行号映射协议。

## 数据结构

- `Sink`：单文件输出缓冲。字段 `includes` / `header` / `body` / `source` 四段字节缓冲 +
  `source_map: Vec<SourceMapEntry>`。`done()` 时按需拼 `#include "<name>.h"` 前缀并整体移入 `source`。
- `SourceMapEntry`：`{ source_line, output_line, source_file }`，均 1-based；
  `source_file` 供多文件项目把输出行回溯到具体输入模块。
- `MultiSink`：多文件项目输出，`files: Vec<(String, Sink)>`，
  `done_with_source_maps()` 返回 `(name, content, source_map)` 三元组。

## 记录协议（后端必须遵守）

1. 转译器在输出每条语句**之前**调 `sink.set_source_line(line)`。
2. 在语句边界（如遍历 stmts 的每轮开头）调 `sink.record()`，把自上次记录以来
   body 中新增的 `\n` 全部记到当前 source line 名下。
3. 顶层条目之间调 `clear_source_line()`，避免把 import 等生成行误记到上一条语句。
4. 需要后补头部/import 行时用 `prepend_body()`——它会同步平移已记录的 output_line，
   保证映射不错位（`done()` 的 include 前缀同样平移）。

## 不变量

- output_line 始终对应当前 `source` 拼接后的最终行号，任何前缀插入都必须走平移路径。
- `Sink::dummy()` 用于临时语句处理，不产生 source map。
- header/body 分段是 C 后端的历史形态；其他后端只用 body，协议不变。

## 显式非目标

- 不做列级映射，只有行级。
- source map 不序列化进输出文件；由调用方（playground，plan-219）消费。

> 来源: crates/auto-lang/src/trans.rs；docs/plans/old/167-module-system.md、old/168-shared-variable.md（MultiSink/escape_str 代码注释）、old/219-playground-source-map.md
