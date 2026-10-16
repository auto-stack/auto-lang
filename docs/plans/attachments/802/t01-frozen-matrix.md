# PLAN-802 T-01：22 项旧臂冻结矩阵

取材：主检出 `1dd123361`（work 入场时点；预检指纹 `2d246b4e8` 时点相关 editor
源码未被 800 改变，native.rs/native_catalog.rs SHA 与 inputs.json 记录一致口径）。
来源文件：`crates/auto-lang/src/vm/native.rs`（feature-on 臂 1591–2150 行区、
feature-off 臂 2999–3098 行区）、`crates/auto-lang/src/vm/native_catalog.rs`
（26–115 行登记 + 864–933 签名表）、`docs/specs/auto-lang/ui/design/
code-editor-session-endpoints.md`（701 端点契约）。

本矩阵冻结**每项**的：弹参顺序（codegen 左→右压、shim 右→左弹）、位宽/编码、
provider 方法映射、在位值语义、无 provider 缺席臂（= 迁移前 feature-off 分支
逐字面值）。方法粒度登记（计划 §5"具体方法划分 T01 登记"）：22 项全部映射到
`CodeEditorHostProvider` 具名方法；`cursor` 三读共用一次 DTO 读取，
`scroll_offset` 双读共用一次 DTO 读取——与迁移前 `code_editor_cursor` /
`controller_snapshot` 单调用形态同构，非同族合并语义。

## 全 22 项矩阵

弹序记法：`←` 依次弹出（右先）。缺席臂错误文本逐字面值引用原 feature-off stub。

| ID | canonical / .at 名 | 签名（弹序） | provider 方法 | 在位值语义 | 无 provider（= 原 feature-off） |
|---|---|---|---|---|---|
| 2910 | auto.code_editor.text / code_editor_text | (key) ←key | `text(key) -> Option<String>` | Some→池化 push Str；None→raise `code_editor_text: no editor registered for key {key:?}` | raise `code_editor_text: the `code-editor` feature is disabled`（不弹参） |
| 2911 | cursor_line | (key) ←key | `cursor(key) -> Option<CursorDto>` | Some→line **0 基** push i32；None→push 0 | raise `code_editor_cursor_line: …disabled` |
| 2912 | cursor_col | (key) ←key | 同上 | Some→**char 列**；None→0 | raise 同族 |
| 2913 | selection_len | (key) ←key | 同上 | Some→**bytes**；None→0 | raise 同族 |
| 2914 | find | (key) ←key | `find(key) -> bool` | push Bool（下一正则匹配跳转，wrap） | raise |
| 2915 | set_text | (key, text) ←text, ←key | `set_text(key, &str) -> bool` | false→raise `code_editor_set_text: no editor registered for key {key:?}`；true→**无 push**（Void） | raise（不弹参） |
| 2919 | undo | (key) ←key | `undo(key) -> bool` | push Bool | raise |
| 2920 | redo | (key) ←key | `redo(key) -> bool` | push Bool | raise |
| 2921 | select_all | (key) ←key | `select_all(key) -> bool` | push Bool | raise |
| 2922 | cut | (key) ←key | `cut(key) -> Option<bool>` | Some(b)→push；None（宿主无编辑器剪贴板面）→raise `code_editor_cut: the `code-editor`+`ui-clipboard` features are required` | raise 同文本 |
| 2923 | copy | (key) ←key | `copy(key) -> Option<bool>` | 同上（copy 不删选区） | raise |
| 2924 | paste | (key) ←key | `paste(key) -> Option<bool>` | 同上（插入光标处） | raise |
| 2932 | fold_toggle | (key, line_1based) ←line, ←key | `fold_toggle(key, line_0) -> bool`；shim 做 `max(1)-1` 1基→0基钳位 | push Bool（折叠后状态） | raise |
| 2933 | fold_hidden_count | (key) ←key | `fold_hidden_count(key) -> usize` | push i32 | raise |
| 2939 | delta | (key) ←key | `delta(key) -> Option<String>`（**destructive read 恰一次**） | Some→池化 push Str（`{"revision":N,"deltas":[...]}`，空队列保水位）；None→raise `code_editor_delta: no editor registered for key {key:?}` | raise（不弹参） |
| 9906 | edit | (key, start, end, replacement) ←repl, ←end(i32), ←start(i32), ←key | `edit(key, usize, usize, &str) -> bool`；负 offset 钳在 **shim**（`start<0‖end<0`→eprintln+false 不触 provider）；反区间/越界/非 UTF-8 边界=provider 返 false | push Bool；成功进原人机统一 delta 队列 | raise |
| 9908 | load_file | (key, path) ←path, ←key | `load_file(key, &str) -> Option<i64>` | Some(n)→push i32；None（无 key/IO/非 UTF-8）→**push -1（值非 raise）** | raise |
| 9909 | save | (key, path) ←path, ←key | `save(key, &str) -> bool` | push Bool（rope→文件直写，无全文 VM 往返；BOM/EOL 归 front） | raise |
| 9910 | set_cursor | (key, line, col) ←col, ←line, ←key（**max(0) 钳在 shim**） | `set_cursor(key, line, char_col) -> bool` | push Bool（0基行/char列，钳位入文档，弃选区） | raise |
| 9911 | scroll_offset_x | (key) ←key | `scroll_offset(key) -> ScrollOffsetDto` | push f64 DTO.x（未绑定/未测量=0.0） | raise |
| 9912 | scroll_offset_y | (key) ←key | 同上 | push f64 DTO.y | raise |
| 9913 | scroll_to | (key, x, y) ←y(f64), ←x(f64), ←key | `scroll_to(key, x, y) -> bool` | push Bool（UI 宿主恒 true=已入队；未绑定由 drain 静默丢） | **弹 3 参 + push false（不 raise）**——原 feature-off 唯一值缺席臂 |

## 关键裁定（本矩阵冻结，实施不得偏移）

1. **缺席≠一律报错**：9913 scroll_to 缺席=弹参返 false；9908 load_file 的
   provider 内失败=-1 值；9909 save 失败=false 值。仅"无 provider"按上表
   镜像 feature-off。
2. **单位不统一**：光标 0 基行/char 列；selection_len=bytes；fold 入口 1 基
   （shim 转 0 基）；edit offset=byte offset（char 边界校验在实现侧）。
3. **cut/copy/paste 的 None 臂**：`Option<bool>` 表达"宿主有编辑器 provider 但
   无 OS 剪贴板面"（= 原 code-editor on + ui-clipboard off 组合）；错误文本
   保持组合特征文案。宿主 adapter 仅在 `ui-clipboard` 下委托
   `code_editor_clipboard_op`，否则返 None。弹参后 raise（错误路径栈弃置，
   可观察面=错误文本逐字面值——原臂 raise 前不弹参，仅此差别，V02 断言错误
   文本）。
4. **delta 纪律**：每次 native 调用恰一次 destructive read；text 不清队列。
   provider 接口无"peek"面，UI adapter 委托 `code_editor_delta` 单调用即满足。
5. **9907 不占用**：Env.track 已占（P673-D2）；本族 9906/9908–9913 邻位不动。
6. **a2r/merged 直接 UI 臂**（ui_gen/rust.rs `vm_builtin_host_call` 22 项中在
   表的映射臂）**不改**——双轨同源语义由"两侧都委托同一 ui::code_editor
   注册表函数"保持；V07 核对其未被误删。
7. **catalog/codegen 零改动**：canonical 名/签名/ID 全部不变；shim 函数名不变
   （单态化后同名符号），`native_catalog.rs` 与 `vm/codegen.rs` intrinsics
   表无需适配。

## provider 接口形态（owned 值域，无 UI 类型/VM 堆句柄）

```rust
pub struct CodeEditorCursorDto { pub line: usize, pub char_col: usize, pub selection_bytes: usize }
pub struct CodeEditorScrollOffsetDto { pub x: f64, pub y: f64 }

pub trait CodeEditorHostProvider: Send + Sync {
    fn text(&self, key: &str) -> Option<String>;
    fn delta(&self, key: &str) -> Option<String>;
    fn edit(&self, key: &str, start: usize, end: usize, replacement: &str) -> bool;
    fn cursor(&self, key: &str) -> Option<CodeEditorCursorDto>;
    fn find(&self, key: &str) -> bool;
    fn set_text(&self, key: &str, text: &str) -> bool;
    fn load_file(&self, key: &str, path: &str) -> Option<i64>;
    fn save(&self, key: &str, path: &str) -> bool;
    fn set_cursor(&self, key: &str, line: usize, char_col: usize) -> bool;
    fn scroll_offset(&self, key: &str) -> CodeEditorScrollOffsetDto;
    fn scroll_to(&self, key: &str, x: f64, y: f64) -> bool;
    fn fold_toggle(&self, key: &str, line_0: usize) -> bool;
    fn fold_hidden_count(&self, key: &str) -> usize;
    fn undo(&self, key: &str) -> bool;
    fn redo(&self, key: &str) -> bool;
    fn select_all(&self, key: &str) -> bool;
    fn cut(&self, key: &str) -> Option<bool>;
    fn copy(&self, key: &str) -> Option<bool>;
    fn paste(&self, key: &str) -> Option<bool>;
}
```

19 方法覆盖 22 natives（cursor×3、scroll_offset×2 共享读取）。进程默认槽
五联 API（set/set_if_absent/get/clear）+ NativeInterface per-VM 槽 +
AutoVM 构造期快照/实例覆盖，与 779/785/790/794 逐条同构。

## 装配漏斗清单（T-03 落点核对用）

| 漏斗 | 位置 | 门控 |
|---|---|---|
| install_ui_host_capabilities | lib.rs:459（解释器/back_proxy/autovm_persistent 三处调用） | `code-editor` |
| run_file_dynamic_ui_inner | lib.rs:5591 | `code-editor`（外层 ui-iced 已含） |
| VmBridge::new_with_children | ui/vm_bridge.rs:~649 | `code-editor` |
| VmBridge::new_from_decl | ui/vm_bridge.rs:~781 | `code-editor` |

原始 `AutoVM::new` 只构造期快照，不回接 UI（既有纪律）。装配后改默认不
影响已建 VM；NI Clone 保 provider；VM merge 保目标 provider；注入不 bump
binding_revision/proof。
