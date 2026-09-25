# code_editor 会话端点三件（save 直写 / set-cursor / scroll 读·写）

> PLAN-701（2026-09-25）。auto-edit 013/010/009 三期 want 集中清偿的
> 供料件——save 直写（013 大文件护栏解除件）、光标写端点（010 会话
> 恢复转正件）、scroll 读/写双端点（010 会话滚动条件形）。消费侧验收
> （端点形探针/矩阵断言）属 auto-edit PLAN-014，不在本件。

## 范围

`ui/code_editor/core/mod.rs` 三个 registry 级函数 +
`vm/native.rs` 四个 shim（9909-9913）+ `vm/native_catalog.rs` 登记 +
`vm/codegen.rs` intrinsics + `ui_gen/rust.rs` `vm_builtin_host_call`
a2r/merged 臂——三面同步（VM shim/catalog/a2r 映射，687 形）。

## 端点契约

### `code_editor_save(key, path) -> bool`（供①）

- **写路径**：storage_key 取 rope（文档事实源，Plan 673 §3.1）→
  `std::fs::write` 一次落盘。全文物化发生在 NATIVE 侧，**零全文 VM
  字符串往返**——与既有「`code_editor_text` 读出 + `fs.write_text`」
  两步链的本质差（50MB 域 1.3-4.2s → 原生直写量级）。
- **字节保真归属（T-03 勘定）**：端点**裸写 rope 字节**——不前置
  BOM、不改写 EOL（rope 持有装载/编辑后的内容，装载时 CRLF 保真）。
  BOM/EOL 包装是 **front（WriteFidelity）侧职责**：下游按 per-file
  bom/eol 元数据在 front 包装后调本端点，对拍须防双重包装。
- **返回语义**：true=写通；false=无此 key / IO 失败（stderr 报错不静
  默，镜像 load 侧 -1 值形约定——错误是值不是 raise）。

### `code_editor_set_cursor(key, line, col) -> bool`（供②a）

- **基面（T-04 勘定）**：**0 基** line / **char 列** col——与读侧
  `code_editor_cursor`（cursor_info：line 0 基、char col、sel bytes）
  完全同口径。下游 SyncCursor 持久化 1 基，换算归下游边界（014 T-06
  注记已预设换算）。
- **钳位**：line 越界 → 末行；col 越界 → 行尾 char 数；char 边界对
  齐（多字节安全，rewrite 侧同款 walk-back）。选区一并弃置
  （`Selection::None`）。
- **不 republish on_cursor**：下一次 widget 事件流（或调用方读侧）
  观察新位。false=无此 key。

### `code_editor_scroll_offset_x(key) / _y(key) -> float` +
### `code_editor_scroll_to(key, x, y) -> bool`（供②b）

- **读=注册表投影单源**：editor 寄宿 scroller 在 renderer 构建期以
  handle=widget id=`editor-scroll-{key}` 入 scroll controller 注册表
  （`bind_controller`；handle 约定单源 `core::editor_scroll_handle`）。
  读端点返回 `controller_snapshot` 的 offset——**不依赖 onscroll 回
  声**（Q-2 裁定形，见下）。
- **写=intent 队列**：`enqueue_intent` 双轴 ScrollTo → 既有 renderer
  drain 消费者解析为 `operation::scroll_to`（命令值先行投影
  `note_controller_offset`，读回一 tick 后以真实 clamp 校正）。
  true=已入队（未绑定 handle 由 drain 的 unprimed 规则静默丢弃）。
- **投影新鲜度**：测量回填走 ScrollStateReader 读回通道（bind 预热 +
  drain 后校正 + MCP 心跳 2s 节拍持续读；editor 句柄放开 offset 回写
  ——renderer 的 editor 臂是 M 泛型，无法合成 on_scroll 回声闭包）。
  用户滚动后 ≤2s 收敛；程序化写后同帧投影=命令值。**会话保存读出的
  偏移有 ≤2s 窗口的旧值可能**——下游断言面按此宽收。
- **Q-2 回声裁定**：iced 程序化 scroll_to 的 onscroll 回声随构建漂移
  （1914 无/2044 有，m1-supply §14 观察件）——本实现读出单源注册表
  投影，**对回声行为零依赖**，两形态构建下读端点语义一致。

## 双轨边界

a2r/merged 臂（`vm_builtin_host_call`）直调同名 core 函数——VM shim
与 a2r 实现同源（供料回执表 T-00② 形探针=下游门；端点返回形与本契
约漂移时双方向有界修订回 new）。
