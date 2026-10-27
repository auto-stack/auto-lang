//! PLAN-843 结构拆分：native host 域实现（自 native.rs 原样搬移）。
use super::*;

/// `code_editor_text(key) -> String`
pub fn shim_code_editor_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_text: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    match provider.text(&key) {
        Some(text) => {
            let idx = vm.add_string(text.into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
            Ok(())
        }
        None => Err(VMError::RuntimeError(format!(
            "code_editor_text: no editor registered for key {key:?}"
        ))),
    }
}

/// `code_editor_delta(key) -> String` — Plan 673 §4.2: destructive read of
/// the unified delta queue as `{"revision":N,"deltas":[...]}` (empty queue →
/// `"deltas":[]` at the current watermark). PLAN-802: provider 单次调用即
/// 恰一次队列消费（text 不清队列；空队列保水位——实现方契约）。
pub fn shim_code_editor_delta(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_delta: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    match provider.delta(&key) {
        Some(json) => {
            let idx = vm.add_string(json.into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
            Ok(())
        }
        None => Err(VMError::RuntimeError(format!(
            "code_editor_delta: no editor registered for key {key:?}"
        ))),
    }
}

/// `code_editor_edit(key, start, end, replacement) -> Bool` — Plan 673 §4.3
/// structured write (insert/delete/replace per the interval shape). False on
/// invalid input (unknown key, start > end, out of bounds, non-char-boundary
/// offsets); the delta enters the same queue human typing uses.
/// 负 offset 钳位留在 shim（i32→usize 边界，PLAN-802 T-01 冻结——stderr
/// 形与迁移前一致，不触 provider）。
pub fn shim_code_editor_edit(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_edit: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    // Args push left-to-right: pop the LAST (replacement) first.
    let replacement = pop_string_arg(task, vm);
    let end = task.ram.pop_i32();
    let start = task.ram.pop_i32();
    let key = pop_string_arg(task, vm);
    let ok = if start < 0 || end < 0 {
        eprintln!("code_editor_edit: negative offsets [{start}, {end})");
        false
    } else {
        provider.edit(&key, start as usize, end as usize, &replacement)
    };
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `code_editor_cursor_line(key) -> Int` (0-based line)
pub fn shim_code_editor_cursor_line(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_cursor_line: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let line = provider.cursor(&key).map_or(0, |c| c.line);
    task.ram.push_i32(line as i32);
    Ok(())
}

/// `code_editor_cursor_col(key) -> Int` (char column)
pub fn shim_code_editor_cursor_col(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_cursor_col: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let col = provider.cursor(&key).map_or(0, |c| c.char_col);
    task.ram.push_i32(col as i32);
    Ok(())
}

/// `code_editor_selection_len(key) -> Int` (bytes)
pub fn shim_code_editor_selection_len(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_selection_len: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let sel = provider.cursor(&key).map_or(0, |c| c.selection_bytes);
    task.ram.push_i32(sel as i32);
    Ok(())
}

/// `code_editor_find(key) -> Bool` — jump to the next regex match (wraps).
pub fn shim_code_editor_find(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_find: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let found = provider.find(&key);
    task.ram.push_nv(auto_val::encode_bool(found));
    Ok(())
}

/// `code_editor_set_text(key, text)` — programmatic write (diff-guarded).
pub fn shim_code_editor_set_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_set_text: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    // Args push left-to-right: pop the LAST (text) first, then the key.
    let text = pop_string_arg(task, vm);
    let key = pop_string_arg(task, vm);
    if !provider.set_text(&key, &text) {
        return Err(VMError::RuntimeError(format!(
            "code_editor_set_text: no editor registered for key {key:?}"
        )));
    }
    Ok(())
}

/// PLAN-687: `code_editor_load_file(key, path) -> i64` — native-side bulk
/// load (total bytes; -1 = no editor / IO error / invalid UTF-8). Error is
/// a VALUE (not a raise): the caller's load loop treats -1 as the empty-tab
/// path, mirroring `File.read_text`'s `unwrap_or_default` convention.
pub fn shim_code_editor_load_file(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_load_file: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    // Args push left-to-right: pop the LAST (path) first, then the key.
    let path = pop_string_arg(task, vm);
    let key = pop_string_arg(task, vm);
    let n = provider.load_file(&key, &path).unwrap_or(-1);
    task.ram.push_i32(n as i32);
    Ok(())
}

/// PLAN-701 供①: `code_editor_save(key, path) -> Bool` — rope→disk direct
/// write (auto-edit PLAN-013 big-file guard relief). The full text is
/// materialized NATIVE-side from the rope and written in one pass — zero
/// full-text VM transit. Error is a VALUE (false), not a raise: the
/// caller's save flow treats false as the guarded/failed path, mirroring
/// the load side's -1 convention. PLAN-802: provider 供值（UI adapter 直写
/// rope 字节——BOM/EOL 包装归 front，701 Spec 不变）。
pub fn shim_code_editor_save(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_save: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    // Args push left-to-right: pop the LAST (path) first, then the key.
    let path = pop_string_arg(task, vm);
    let key = pop_string_arg(task, vm);
    let ok = provider.save(&key, &path);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// PLAN-701 供②a: `code_editor_set_cursor(key, line, col) -> Bool` —
/// programmatic cursor placement (session restore / goto-line). 0-based
/// line/char-col (read-side `code_editor_cursor_*` convention); clamped
/// into the document, selection dropped. false = no editor for key.
/// PLAN-802: provider 供值；负参 max(0) 钳位留 shim（T-01 冻结）。
pub fn shim_code_editor_set_cursor(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_set_cursor: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let col = pop_arg_i32(task).max(0) as usize;
    let line = pop_arg_i32(task).max(0) as usize;
    let key = pop_string_arg(task, vm);
    let ok = provider.set_cursor(&key, line, col);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// PLAN-701 供②b: `code_editor_scroll_offset_x(key) -> Float` /
/// `code_editor_scroll_offset_y(key)` — per-editor scroll offset readout.
/// Single source = the scroll controller registry projection (bind 预热 +
/// intent-drain 投影 + ScrollStateReader 读回校正，MCP 心跳 2s 节拍收敛；
/// 见 renderer `editor-scroll-{key}` 绑定臂)。Unbound/unmeasured → 0.0
/// (top) — the session-restore neutral value. PLAN-802: provider 供值
///（UI adapter 委托 editor_scroll_handle + controller_snapshot——投影
/// 单源不迁；两读共享一次 DTO 读取，T-01 冻结）。
pub fn shim_code_editor_scroll_offset_x(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_scroll_offset_x: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let offset = provider.scroll_offset(&key);
    task.ram.push_f64(offset.x);
    Ok(())
}

pub fn shim_code_editor_scroll_offset_y(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_scroll_offset_y: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let offset = provider.scroll_offset(&key);
    task.ram.push_f64(offset.y);
    Ok(())
}

/// PLAN-701 供②b: `code_editor_scroll_to(key, x, y) -> Bool` — apply a
/// scroll offset to the editor's hosted scroller. Routed through the
/// scroll controller intent queue; the renderer's existing drain resolves
/// it to `operation::scroll_to` on `editor-scroll-{key}` (unbound handle =
/// scroller not built — intent silently dropped by the drain's unprimed
/// rule; we report true = request enqueued, mirroring shim_scroll_to).
/// PLAN-802: provider 供值（UI adapter 双轴 ScrollTo intent 入队恒 true，
/// X 先 Y 后——794 ScrollIntentDto 委托同源）；**缺席=弹参后 push false**
/// （本族唯一值缺席臂，T-01 冻结=迁移前 feature-off 分支逐字面值）。
pub fn shim_code_editor_scroll_to(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            // 唯一值缺席臂：镜像迁移前 feature-off 分支（弹 3 参、push
            // false，不 raise）。
            let _ = pop_f64_operand(task);
            let _ = pop_f64_operand(task);
            let _ = pop_string_arg(task, vm);
            task.ram.push_nv(auto_val::encode_bool(false));
            return Ok(());
        }
    };
    let y = pop_f64_operand(task);
    let x = pop_f64_operand(task);
    let key = pop_string_arg(task, vm);
    let ok = provider.scroll_to(&key, x, y);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `code_editor_fold_toggle(key, line_1based) -> Bool` — toggle the fold
/// whose opener sits on `line` (1-based, gutter numbering). Returns `true`
/// when the region is folded afterwards. View state only — no undo entry.
/// PLAN-802: 1 基→0 基钳位（max(1)-1）留 shim，provider 收 0 基行。
pub fn shim_code_editor_fold_toggle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_fold_toggle: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    // Args push left-to-right: pop the LAST (line) first, then the key.
    let line_1based = task.ram.pop_i32();
    let key = pop_string_arg(task, vm);
    let folded = provider.fold_toggle(&key, line_1based.max(1) as usize - 1);
    task.ram.push_nv(auto_val::encode_bool(folded));
    Ok(())
}

/// `code_editor_fold_hidden_count(key) -> Int` — lines currently hidden by
/// folding (tests / MCP assertions read this).
pub fn shim_code_editor_fold_hidden_count(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_fold_hidden_count: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let count = provider.fold_hidden_count(&key);
    task.ram.push_i32(count as i32);
    Ok(())
}

/// `code_editor_undo(key) -> Bool` — true when an editor exists.
pub fn shim_code_editor_undo(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_undo: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let ok = provider.undo(&key);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `code_editor_redo(key) -> Bool`
pub fn shim_code_editor_redo(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_redo: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let ok = provider.redo(&key);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `code_editor_select_all(key) -> Bool`
pub fn shim_code_editor_select_all(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_select_all: the `code-editor` feature is disabled".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    let ok = provider.select_all(&key);
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `code_editor_cut(key) -> Bool` — selection → OS clipboard, then delete.
pub fn shim_code_editor_cut(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_cut: the `code-editor`+`ui-clipboard` features are required".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    match provider.cut(&key) {
        Some(ok) => {
            task.ram.push_nv(auto_val::encode_bool(ok));
            Ok(())
        }
        // None = 宿主无编辑器剪贴板面（错误路径栈弃置，可观察面=组合
        // 特征文案——T-01 冻结）。
        None => Err(VMError::RuntimeError(
            "code_editor_cut: the `code-editor`+`ui-clipboard` features are required".into(),
        )),
    }
}

/// `code_editor_copy(key) -> Bool` — selection → OS clipboard.
pub fn shim_code_editor_copy(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_copy: the `code-editor`+`ui-clipboard` features are required".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    match provider.copy(&key) {
        Some(ok) => {
            task.ram.push_nv(auto_val::encode_bool(ok));
            Ok(())
        }
        None => Err(VMError::RuntimeError(
            "code_editor_copy: the `code-editor`+`ui-clipboard` features are required".into(),
        )),
    }
}

/// `code_editor_paste(key) -> Bool` — OS clipboard → insert at cursor.
pub fn shim_code_editor_paste(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let provider = match vm.native_interface.code_editor_provider() {
        Some(p) => p,
        None => {
            return Err(VMError::RuntimeError(
                "code_editor_paste: the `code-editor`+`ui-clipboard` features are required".into(),
            ));
        }
    };
    let key = pop_string_arg(task, vm);
    match provider.paste(&key) {
        Some(ok) => {
            task.ram.push_nv(auto_val::encode_bool(ok));
            Ok(())
        }
        None => Err(VMError::RuntimeError(
            "code_editor_paste: the `code-editor`+`ui-clipboard` features are required".into(),
        )),
    }
}

/// `clipboard_text() -> String` (empty when the clipboard is unavailable).
pub fn shim_clipboard_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    match vm.native_interface.clipboard_text_provider() {
        Some(provider) => {
            let text = provider.text().unwrap_or_default();
            let idx = vm.add_string(text.into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
            Ok(())
        }
        None => Err(VMError::RuntimeError(
            "clipboard_text: the `ui-clipboard` feature is disabled".into(),
        )),
    }
}

/// `clipboard_set_text(text) -> Bool`
pub fn shim_clipboard_set_text(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    match vm.native_interface.clipboard_text_provider() {
        Some(provider) => {
            // pop_string_arg 已物化 owned String（utf8_lossy）——provider 收
            // 纯 `&str`，接口零 VM 索引/借用缓冲泄漏（V02 ownership）。
            let text = pop_string_arg(task, vm);
            let ok = provider.set_text(&text);
            task.ram.push_nv(auto_val::encode_bool(ok));
            Ok(())
        }
        None => Err(VMError::RuntimeError(
            "clipboard_set_text: the `ui-clipboard` feature is disabled".into(),
        )),
    }
}

/// `clipboard_files_get() -> List<str>`（Explorer Ctrl+C 后取路径列表）。
pub fn shim_clipboard_files_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // provider 缺席 = G3 空表（与迁移前非 Windows 臂同形）。
    let paths = match vm.native_interface.clipboard_media_provider() {
        Some(provider) => provider.files_get(),
        None => Vec::new(),
    };
    let mut list = crate::vm::types::ListData::<auto_val::Value>::new();
    for p in paths {
        list.push(auto_val::Value::Str(auto_val::AutoStr::from(
            p.to_string_lossy().into_owned(),
        )));
    }
    let list_id = vm.insert_heap_object(list);
    vm.rc_push_id(task, list_id as u64);
    Ok(())
}

/// `clipboard_files_set(paths: List<str>) -> Bool`（Explorer Ctrl+V 落地）。
pub fn shim_clipboard_files_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::types::ListData;
    let list_id = crate::vm::native::pop_arg_i32(task) as u64;
    let _stake_list_id = crate::vm::native::StakeGuard::new(vm, list_id);
    let mut paths: Vec<std::path::PathBuf> = Vec::new();
    if let Some(obj) = vm.get_heap_object(list_id) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<auto_val::Value>>() {
            for v in &list.elems {
                if let auto_val::Value::Str(s) = v {
                    paths.push(std::path::PathBuf::from(s.as_str()));
                }
            }
        } else if let Some(list) = guard.as_any().downcast_ref::<ListData<String>>() {
            paths.extend(list.elems.iter().map(std::path::PathBuf::from));
        }
    }
    // read 锁限 collect 域（guard 已随上面块尾 drop）；provider 回调在锁外，
    // 收 owned `Vec<PathBuf>`——接口零 VM 索引/借用缓冲泄漏（V02 ownership）。
    let ok = match vm.native_interface.clipboard_media_provider() {
        Some(provider) => provider.files_set(&paths),
        None => false,
    };
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `clipboard_image_get() -> {path,width,height} | None`（DIBV5→PNG 存
/// temp；Record 走 heap ObjectData，Plan 390 §15 H3b 形态）。
pub fn shim_clipboard_image_get(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use auto_val::{AutoStr, Value, ValueKey};
    match vm
        .native_interface
        .clipboard_media_provider()
        .and_then(|provider| provider.image_get())
    {
        Some(img) => {
            let mut rec = crate::vm::types::ObjectData::new();
            rec.set(
                ValueKey::Str(AutoStr::from("path")),
                Value::Str(AutoStr::from(img.path.to_string_lossy())),
            );
            rec.set(
                ValueKey::Str(AutoStr::from("width")),
                Value::Int(img.width as i32),
            );
            rec.set(
                ValueKey::Str(AutoStr::from("height")),
                Value::Int(img.height as i32),
            );
            let id = vm.insert_heap_object(rec);
            vm.rc_push_id(task, id as u64);
        }
        // provider 缺席或 None（无图像格式/不可用）= G3 null 同形。
        None => task.ram.push_nv(auto_val::encode_null()),
    }
    Ok(())
}

/// `clipboard_image_set(png_path: str) -> Bool`
pub fn shim_clipboard_image_set(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // pop_string_arg 已物化 owned String（utf8_lossy）+ StakeGuard.nv 配平
    //——provider 收纯 `&Path`（缺席臂同样消费实参保持栈纪律）。
    let path = pop_string_arg(task, vm);
    let ok = match vm.native_interface.clipboard_media_provider() {
        Some(provider) => provider.image_set(std::path::Path::new(&path)),
        None => false,
    };
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `dnd_start(payload_json) -> Bool`：发起系统拖拽（**调用线程内联阻塞**
/// 至落下/取消——OLE 拖拽循环必须跑在收到按下的输入线程上，488 步骤 9
/// E2E 定案；桌面模式 VM handler 在 UI 线程执行即被点击线程，DoDragDrop
/// 自带消息泵，模态期与原生 App 一致）。完成效果经 `on_dnd_finished`
/// 事件回 App。返回 true = 执行过会话（落下**或取消**）——效果另报，
/// 不得把 cancel 映射 false 或把 true 当 drop 成功（T-01 冻结）。
/// 载荷 JSON 字段（text/files/virtual_files 组合）见 `ui::native_dnd`
/// 模块的 parse_payload_json 文档。
pub fn shim_dnd_start(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // pop_string_arg 已物化 owned String + StakeGuard.nv 配平；provider 接
    // owned JSON（缺席臂同样消费实参保持栈纪律——与迁移前 off 臂同形）。
    let payload_json = pop_string_arg(task, vm);
    let ok = match vm.native_interface.dnd_host_provider() {
        Some(provider) => provider.start(payload_json),
        None => false,
    };
    task.ram.push_nv(auto_val::encode_bool(ok));
    Ok(())
}

/// `scroll_controller() -> Str` — 分配 controller 句柄（UI 宿主
/// "@scrollctl:N" 形；任意非空 opaque 串等价——T-01 冻结）。缺席 = Int 0
///（迁移前 no-ui 臂原形，非空串）。
pub fn shim_scroll_controller(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    match vm.native_interface.scroll_host_provider() {
        Some(provider) => {
            let handle = provider.next_controller_handle();
            let idx = vm.add_string(handle.into_bytes());
            vm.rc_push_str_idx(task, idx as usize);
        }
        None => task.ram.push_i32(0),
    }
    Ok(())
}

/// `scroll_to_start(handle)` — 到起点（Y 简写；可选第二实参轴）。
/// 弹序与 codegen 压序（左→右、末参在顶）对齐：可选轴在顶先弹、再弹
/// handle（scroll_to 的 b,a,handle 同款）。T-01 勘定：旧 UI 臂 handle
/// 先弹把轴串误路由为句柄（记录型 provider 探针实证 `scroll_to_end(h,
/// "x")` 落 handle="x"）——库内全部用法为单轴简写形零破坏，此处按签名
/// 合同修正路由；空句柄+可选轴组合同样按声明 arity 配平。
pub fn shim_scroll_to_start(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::host_capabilities::{ScrollAxisDto, ScrollIntentDto};
    let arity = task.pending_native_arg_count as usize;
    let provider = vm.native_interface.scroll_host_provider();
    let axis = if arity >= 2 {
        pop_axis_arg(task, vm)
    } else {
        ScrollAxisDto::Y
    };
    let handle = pop_controller_handle(task, vm);
    if let (Some(p), Some(handle)) = (provider.as_ref(), handle) {
        p.enqueue_intent(&handle, ScrollIntentDto::ToStart { axis });
    }
    task.ram.push_nv(auto_val::encode_bool(provider.is_some()));
    Ok(())
}

/// `scroll_to_end(handle, axis?)` — 到终点（弹序勘定同 to_start）。
pub fn shim_scroll_to_end(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::host_capabilities::{ScrollAxisDto, ScrollIntentDto};
    let arity = task.pending_native_arg_count as usize;
    let provider = vm.native_interface.scroll_host_provider();
    let axis = if arity >= 2 {
        pop_axis_arg(task, vm)
    } else {
        ScrollAxisDto::Y
    };
    let handle = pop_controller_handle(task, vm);
    if std::env::var("P656_DEBUG").is_ok() {
        eprintln!(
            "[P656-NATIVE] to_end handle={}",
            handle.as_deref().unwrap_or("")
        );
    }
    if let (Some(p), Some(handle)) = (provider.as_ref(), handle) {
        p.enqueue_intent(&handle, ScrollIntentDto::ToEnd { axis });
    }
    task.ram.push_nv(auto_val::encode_bool(provider.is_some()));
    Ok(())
}

/// `scroll_by(handle, delta)` / `scroll_by(handle, axis, delta)` — 相对
/// 滚动（弹序勘定：delta 顶弹→axis→handle，与压序对齐）。
pub fn shim_scroll_by(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::host_capabilities::{ScrollAxisDto, ScrollIntentDto};
    let arity = task.pending_native_arg_count as usize;
    let provider = vm.native_interface.scroll_host_provider();
    let delta = pop_f64_operand(task); // 最后压栈，先弹
    let axis = if arity >= 3 {
        pop_axis_arg(task, vm)
    } else {
        ScrollAxisDto::Y
    };
    let handle = pop_controller_handle(task, vm);
    if let (Some(p), Some(handle)) = (provider.as_ref(), handle) {
        p.enqueue_intent(&handle, ScrollIntentDto::ScrollBy { axis, delta });
    }
    task.ram.push_nv(auto_val::encode_bool(provider.is_some()));
    Ok(())
}

/// `scroll_to(handle, x, y)`（双轴绝对）/ `scroll_to(handle, axis, offset)`
/// （单轴绝对；axis 实参为字符串时按轴形态）。args 压序 handle,a,b → 弹序
/// b,a,handle；a 为字符串 → 轴形态，否则坐标形态（双 intent，X 先 Y 后
/// ——drain 顺序折叠依赖此序，T-01 冻结）。缺席 = false（仍弹三实参）。
pub fn shim_scroll_to(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::host_capabilities::{ScrollAxisDto, ScrollIntentDto};
    let provider = vm.native_interface.scroll_host_provider();
    let b = pop_f64_operand(task);
    let a_nv = crate::vm::native::pop_arg_nv(task);
    let a_is_string = auto_val::is_string(a_nv);
    let a_str = if a_is_string {
        let idx = auto_val::decode_string(a_nv);
        vm.get_string(idx)
            .map(|s| String::from_utf8_lossy(&s).into_owned())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let _stake_a = crate::vm::native::StakeGuard::nv(vm, a_nv);
    let handle = pop_controller_handle(task, vm);
    if let (Some(p), Some(handle)) = (provider.as_ref(), handle) {
        if a_is_string {
            // 池份额已由 StakeGuard 释放；轴形态：a="x"/"y"，b=offset。
            let axis = if a_str == "x" {
                ScrollAxisDto::X
            } else {
                ScrollAxisDto::Y
            };
            p.enqueue_intent(&handle, ScrollIntentDto::ScrollTo { axis, offset: b });
        } else {
            let a = if auto_val::is_f64(a_nv) {
                auto_val::decode_f64(a_nv)
            } else if auto_val::is_f32(a_nv) {
                auto_val::decode_f32(a_nv) as f64
            } else {
                auto_val::decode_i32(a_nv) as f64
            };
            p.enqueue_intent(
                &handle,
                ScrollIntentDto::ScrollTo {
                    axis: ScrollAxisDto::X,
                    offset: a,
                },
            );
            p.enqueue_intent(
                &handle,
                ScrollIntentDto::ScrollTo {
                    axis: ScrollAxisDto::Y,
                    offset: b,
                },
            );
        }
    }
    task.ram.push_nv(auto_val::encode_bool(provider.is_some()));
    Ok(())
}

/// `scroll_state(handle) -> ScrollState` — 最近测量快照读出（8 具名字段
/// record，GenericInstanceData 堆路径——ProcInfo 同款，.at 端 `s.progress_y`
/// 具名可读；未测量/未绑定为全零——capability 验证断言面；progress 由
/// provider 侧几何单源投影）。缺席 = Float 0.0（旧 no-ui 臂原形，非 record）。
pub fn shim_scroll_state(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    let handle = pop_string_arg(task, vm);
    if std::env::var("P656_DEBUG").is_ok() {
        eprintln!("[P656-NATIVE] state handle={handle}");
    }
    match vm.native_interface.scroll_host_provider() {
        Some(provider) => {
            let snap = provider.controller_snapshot(&handle);
            let fields = vec![
                auto_val::Value::Float(snap.offset_x),
                auto_val::Value::Float(snap.offset_y),
                auto_val::Value::Float(snap.viewport_w),
                auto_val::Value::Float(snap.viewport_h),
                auto_val::Value::Float(snap.content_w),
                auto_val::Value::Float(snap.content_h),
                auto_val::Value::Float(snap.progress_x),
                auto_val::Value::Float(snap.progress_y),
            ];
            let names: Vec<String> = [
                "offset_x",
                "offset_y",
                "viewport_w",
                "viewport_h",
                "content_w",
                "content_h",
                "progress_x",
                "progress_y",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect();
            let inst =
                GenericInstanceData::new_with_names("ScrollState".to_string(), fields, names);
            let id = vm.insert_heap_object(inst);
            vm.rc_push_id(task, id as u64);
        }
        None => task.ram.push_f64(0.0),
    }
    Ok(())
}

/// PLAN-781 T-01 冻结：`to_child(handle, key) -> str`——锚点定位请求入队
/// 回执（非完成证明）。返回 "queued" / "error:<reason>"（bad_handle /
/// empty_key / unsupported——reason 由 provider 校验透传，UI adapter 委托
/// 既有 controller::enqueue_anchor_request 原语义）。args 压序 handle,key →
/// 弹序 key,handle。缺席 = "error:unsupported"（旧 no-ui 臂原形）。
pub fn shim_scroll_to_child(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key = pop_string_arg(task, vm);
    let handle = pop_string_arg(task, vm);
    let reply = match vm.native_interface.scroll_host_provider() {
        Some(provider) => match provider.enqueue_anchor_request(&handle, &key) {
            Ok(()) => "queued".to_string(),
            Err(reason) => format!("error:{reason}"),
        },
        None => "error:unsupported".to_string(),
    };
    let idx = vm.add_string(reply.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// PLAN-781 T-01 冻结：`anchor_state(handle, key) -> str`——请求状态查询。
/// "pending" / "applied" / "invalid:<reason>" / "unknown"（781 状态机完整
/// 继承——queued≠完成、一次性消费、换绑/失锚 invalid reason 语义在 provider
/// 侧状态机，本 shim 仅编组）。缺席 = "unknown"（旧 no-ui 臂原形）。
pub fn shim_scroll_anchor_state(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::host_capabilities::ScrollAnchorStateDto;
    let key = pop_string_arg(task, vm);
    let handle = pop_string_arg(task, vm);
    let reply = match vm.native_interface.scroll_host_provider() {
        Some(provider) => match provider.anchor_request_state(&handle, &key) {
            Some(ScrollAnchorStateDto::Pending) => "pending".to_string(),
            Some(ScrollAnchorStateDto::Applied) => "applied".to_string(),
            Some(ScrollAnchorStateDto::Invalid(reason)) => format!("invalid:{reason}"),
            None => "unknown".to_string(),
        },
        None => "unknown".to_string(),
    };
    let idx = vm.add_string(reply.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `dialog_open(filter) -> String` — filter is a `,. ;`-separated extension
/// list (".at, .au"); empty disables filtering. "" on cancel.
pub fn shim_dialog_open(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let Some(provider) = vm.native_interface.dialog_host_provider() else {
        return Err(VMError::RuntimeError(
            "dialog_open: the `ui-dialog` feature is disabled".into(),
        ));
    };
    // pop_string_arg 已物化 owned String + StakeGuard.nv 配平；provider 接
    // raw filter（&str 借用 owned 实参，不泄漏 VM 池索引）。
    let filter = pop_string_arg(task, vm);
    let path = provider.open(&filter);
    let idx = vm.add_string(path.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `dialog_save(default_name) -> String` — "" on cancel.
pub fn shim_dialog_save(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let Some(provider) = vm.native_interface.dialog_host_provider() else {
        return Err(VMError::RuntimeError(
            "dialog_save: the `ui-dialog` feature is disabled".into(),
        ));
    };
    let default_name = pop_string_arg(task, vm);
    let path = provider.save(&default_name);
    let idx = vm.add_string(path.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `ui.focus(target_key)` — Stack: key -> (void). 平台中立程序化聚焦：
/// 目标键=输入控件的 .at 输入 handler 事件键（如 ".Input($event)"，比较时
/// 剥点前缀与实参尾）或 "Widget.event" 限定形。web 轨经 ts_adapter 直译
/// querySelector（等价 dom.focus_first 语义）。
pub fn shim_ui_focus(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    {
        let nv = crate::vm::native::pop_arg_nv(task);
        let _stake = crate::vm::native::StakeGuard::nv(vm, nv);
        let key = if auto_val::is_string(nv) {
            let idx = auto_val::decode_string(nv) as usize;
            vm.strings
                .read()
                .unwrap()
                .get(idx)
                .cloned()
                .map(|b| String::from_utf8_lossy(&b).to_string())
                .unwrap_or_default()
        } else {
            String::new()
        };
        if !key.is_empty() {
            if let Ok(mut slot) = UI_FOCUS_REQUEST.lock() {
                *slot = Some(key);
            }
        }
    }
    Ok(())
}
