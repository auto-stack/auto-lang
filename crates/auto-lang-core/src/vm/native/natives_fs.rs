//! PLAN-843 结构拆分：native fs 域实现（自 native.rs 原样搬移）。
use super::*;

/// `auto.diff_files(path_a, path_b, ctx) -> Str` — envelope JSON for two
/// files (rows render-ready, three-segment marks, CR tolerated).
// PLAN-814: diff 族为纯文本 core 能力（auto-text-core/provider 端口），历史 code-editor 门随 Core 迁 SDK 解除
pub fn shim_diff_files(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Args push left-to-right: pop the LAST (ctx) first.
    let ctx = pop_arg_i32(task).max(0) as usize;
    let path_b = pop_string_arg(task, vm);
    let path_a = pop_string_arg(task, vm);
    // PLAN-807: 纯路径面直调文本 core（零 crate::ui 引用；envelope 语义
    // 冻结不变）。
    let json =
        auto_text_core::diff::envelope::diff_files_envelope_from_paths(&path_a, &path_b, ctx);
    let idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `auto.diff_snapshots(key_a, key_b) -> Str` — envelope JSON for two live
/// editor buffers (direct registry read, zero full-text VM transit).
// PLAN-814: diff 族为纯文本 core 能力（auto-text-core/provider 端口），历史 code-editor 门随 Core 迁 SDK 解除
pub fn shim_diff_snapshots(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let key_b = pop_string_arg(task, vm);
    let key_a = pop_string_arg(task, vm);
    // PLAN-807: 快照经 EditorSnapshotProvider 端口供值（同源 owned 快照，
    // 零全文字符串 VM 中转）；缺 provider = 无法找到相应 editor → 既有
    // err envelope（missing=key_a，与旧两键皆缺路径同形）；判错优先
    // key_a 先（冻结序）。
    let pair = match vm.native_interface.editor_snapshot_provider() {
        Some(provider) => match (provider.snapshot(&key_a), provider.snapshot(&key_b)) {
            (Some(sa), Some(sb)) => auto_text_core::diff::envelope::DiffSnapshotsPair::Both(sa, sb),
            (None, _) => auto_text_core::diff::envelope::DiffSnapshotsPair::Missing(key_a.clone()),
            (Some(_), None) => {
                auto_text_core::diff::envelope::DiffSnapshotsPair::Missing(key_b.clone())
            }
        },
        None => auto_text_core::diff::envelope::DiffSnapshotsPair::Missing(key_a.clone()),
    };
    let json = auto_text_core::diff::envelope::diff_snapshots_envelope_snapshots(pair);
    let idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `auto.diff_dirs(path_a, path_b) -> Str` — envelope JSON for a recursive
/// directory comparison (five-state classification, cap 5000).
// PLAN-814: diff 族为纯文本 core 能力（auto-text-core/provider 端口），历史 code-editor 门随 Core 迁 SDK 解除
pub fn shim_diff_dirs(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_b = pop_string_arg(task, vm);
    let path_a = pop_string_arg(task, vm);
    // PLAN-807: 纯目录面直调文本 core。
    let json = auto_text_core::diff::envelope::diff_dirs_envelope(&path_a, &path_b);
    let idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `auto.diff_files_window(path_a, path_b, ctx, rows_offset, rows_limit) -> Str`
/// — 窗口投影 envelope（PLAN-716 组C；rows_total+truncated 激活语义）。
// PLAN-814: diff 族为纯文本 core 能力（auto-text-core/provider 端口），历史 code-editor 门随 Core 迁 SDK 解除
pub fn shim_diff_files_window(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Args push left-to-right: pop the LAST (rows_limit) first.
    let limit = pop_arg_i32(task).max(0) as usize;
    let offset = pop_arg_i32(task).max(0) as usize;
    let ctx = pop_arg_i32(task).max(0) as usize;
    let path_b = pop_string_arg(task, vm);
    let path_a = pop_string_arg(task, vm);
    // PLAN-807: 纯窗口面直调文本 core。
    let json = auto_text_core::diff::envelope::diff_files_envelope_from_paths_window(
        &path_a, &path_b, ctx, offset, limit,
    );
    let idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `file_basename(path) -> String` — final path segment, treating both `/`
/// and `\` as separators (Plan 418: tab titles after Open/Save show the
/// file name, not the full OS path). No feature gate — pure string logic.
pub fn shim_file_basename(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_string_arg(task, vm);
    let base = path.rsplit(['/', '\\']).next().unwrap_or("").to_string();
    let idx = vm.add_string(base.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// File.create(path) → opaque handle with BufWriter<File>
/// Stack: path_string -> handle_id
pub fn shim_file_create_handle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_string(task, vm);
    let file = std::fs::File::create(&path)
        .map_err(|e| VMError::RuntimeError(format!("File.create failed: {}", e)))?;
    let writer: Box<dyn std::io::Write + Send + Sync> = Box::new(std::io::BufWriter::new(file));
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("std::fs::File", writer);
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// File.open(path) → opaque handle
/// Stack: path_string -> handle_id
pub fn shim_file_open_handle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path = pop_string(task, vm);
    let file = std::fs::File::open(&path)
        .map_err(|e| VMError::RuntimeError(format!("File.open failed: {}", e)))?;
    let content = {
        let mut buf = String::new();
        std::io::Read::read_to_string(&mut std::io::BufReader::new(file), &mut buf)
            .map_err(|e| VMError::RuntimeError(format!("File.open read failed: {}", e)))?;
        buf
    };
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new("std::fs::FileContent", content);
    let id = vm.insert_heap_object(obj);
    vm.rc_push_id(task, id as u64); // Plan 419
    Ok(())
}

/// handle.write(data) → write string to file handle
/// Stack: data_string, handle_id -> void
pub fn shim_file_write_handle(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let data = pop_string(task, vm);
    let handle = crate::vm::native::pop_arg_i32(task) as u64;

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    let obj = vm
        .get_heap_object(handle)
        .ok_or_else(|| VMError::RuntimeError("Invalid File handle".to_string()))?;
    let mut guard = obj.write().unwrap();
    let rust_obj = guard
        .as_any_mut()
        .downcast_mut::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        .ok_or_else(|| VMError::RuntimeError("Not a RustStdlibObject".to_string()))?;
    let writer = rust_obj
        .downcast_mut::<Box<dyn std::io::Write + Send + Sync>>()
        .ok_or_else(|| VMError::RuntimeError("Not a File writer".to_string()))?;
    writer
        .write_all(data.as_bytes())
        .map_err(|e| VMError::RuntimeError(format!("write failed: {}", e)))?;
    drop(guard);
    task.ram.push_i32(0);
    Ok(())
}

/// handle.try_clone() → clone the handle (returns same content for read handles)
/// Stack: handle_id -> handle_id (clone)
pub fn shim_file_try_clone(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // For simplicity, just push a copy of the handle ID
    let handle = crate::vm::native::pop_arg_i32(task);

    let _stake_handle = crate::vm::native::StakeGuard::new(vm, handle as i64 as u64);
    task.ram.push_i32(handle);
    Ok(())
}

/// Get system temp directory.
/// Stack: -> str_idx
pub fn shim_fs_temp_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let temp = std::env::temp_dir();
    let s = temp.to_string_lossy().to_string();
    let str_idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Create a temporary file and return its path.
/// Stack: -> str_idx
pub fn shim_fs_temp_file(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let temp_dir = std::env::temp_dir();
    let id =
        std::sync::atomic::AtomicU64::new(0).fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let path = temp_dir.join(format!("auto_tmp_{}", id));
    // Create the file
    std::fs::File::create(&path)
        .map_err(|e| VMError::RuntimeError(format!("temp_file failed: {}", e)))?;
    let s = path.to_string_lossy().to_string();
    let str_idx = vm.add_string(s.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Rename a file or directory.
/// Stack: str_idx(new), str_idx(old) -> void
pub fn shim_fs_rename(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let new_idx = task.ram.pop_str_idx() as u32;
    let old_idx = task.ram.pop_str_idx() as u32;
    let old_path = vm
        .get_string(old_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let new_path = vm
        .get_string(new_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    std::fs::rename(&old_path, &new_path).map_err(|e| {
        VMError::RuntimeError(format!(
            "rename failed: {} -> {}: {}",
            old_path, new_path, e
        ))
    })?;
    task.ram.push_i32(0);
    Ok(())
}

/// Read directory entries (non-recursive) — returns JSON array of filenames.
/// Stack: str_idx(path) -> str_idx (JSON)
pub fn shim_fs_read_dir(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let entries: Vec<String> = std::fs::read_dir(&path)
        .map_err(|e| VMError::RuntimeError(format!("read_dir failed: {}: {}", path, e)))?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    let json = serde_json::to_string(&entries)
        .map_err(|e| VMError::RuntimeError(format!("read_dir json: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Canonicalize a path (absolute, resolved symlinks).
/// Stack: str_idx -> str_idx
pub fn shim_fs_canonical(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    // PLAN-016 T-05：剥 Windows 扩展长度前缀 \\\\?\\（canonicalize 原样返回，
    // 面包屑/地址栏/拼接面不消费该形态）。
    // PLAN-659 T-06：失败返 ""（语料哨兵权威——027 NavTo 的 `can == ""`
    // 判空分支可达；与族内 Path.canonicalize 的 unwrap_or_default 对齐）。
    let canonical = std::fs::canonicalize(&path)
        .map(|p| {
            let s = p.to_string_lossy().to_string();
            s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
        })
        .unwrap_or_default();
    let str_idx = vm.add_string(canonical.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Get file extension.
/// Stack: str_idx -> str_idx
pub fn shim_fs_ext(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let ext = std::path::Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(ext.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Get filename stem (name without extension).
/// Stack: str_idx -> str_idx
pub fn shim_fs_stem(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let stem = std::path::Path::new(&path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(stem.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Walk directory recursively, returning only files — JSON array of paths.
/// Stack: str_idx(dir) -> str_idx (JSON)
pub fn shim_fs_walk_files(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dir_idx = task.ram.pop_str_idx() as u32;
    let dir = vm
        .get_string(dir_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let mut files: Vec<String> = Vec::new();
    if let Ok(entries) = walkdir_recursive(&dir) {
        files = entries;
    }
    let json = serde_json::to_string(&files)
        .map_err(|e| VMError::RuntimeError(format!("walk_files json: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Walk directory recursively, return JSON array of ALL paths (files + dirs).
/// Stack: str_idx(dir) -> str_idx (JSON)
pub fn shim_fs_walk(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dir_idx = task.ram.pop_str_idx() as u32;
    let dir = vm
        .get_string(dir_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let mut paths: Vec<String> = Vec::new();
    if let Ok(entries) = walkdir_all(&dir) {
        paths = entries;
    }
    let json = serde_json::to_string(&paths)
        .map_err(|e| VMError::RuntimeError(format!("walk json: {}", e)))?;
    let str_idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Recursive directory tree as nested JSON aligned with the TreeView node
/// schema: `{id,label,children,kind,icon,is_leaf,badge}`. `id` is the path
/// relative to `root` ('/'-separated); VCS/build noise and dotfiles are
/// skipped; `max_depth` caps recursion (1 = root entries only).
/// Stack: str_idx(root), int(max_depth) -> str_idx (JSON)
pub fn shim_fs_tree(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Args push left-to-right: pop the LAST (max_depth) first, then root.
    let max_depth = crate::vm::native::pop_arg_i32(task).clamp(1, 8) as usize;
    let root = pop_string_arg(task, vm);
    let root_path = std::path::PathBuf::from(&root);
    let mut buf = String::new();
    if root_path.is_dir() {
        fs_tree_walk(&root_path, &root_path, max_depth, &mut buf);
    }
    let json = if buf.is_empty() {
        "[]".to_string()
    } else {
        format!("[{}]", buf)
    };
    let str_idx = vm.add_string(json.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Get file metadata as JSON string.
/// Stack: str_idx(path) -> str_idx (JSON with len, is_dir, is_file, readonly)
pub fn shim_fs_metadata(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let metadata = std::fs::metadata(&path)
        .map_err(|e| VMError::RuntimeError(format!("metadata failed: {}: {}", path, e)))?;
    // PLAN-016 T-05：补 modified（epoch 秒；1970 前/取值失败 = -1，.at 侧
    // fmt_date 以 "--" 呈现）——修改日期列为文件管理器基本列，metadata
    // JSON 原仅 len/is_dir/is_file/readonly。
    let modified = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(-1);
    let json = serde_json::json!({
        "len": metadata.len(),
        "is_dir": metadata.is_dir(),
        "is_file": metadata.is_file(),
        "readonly": metadata.permissions().readonly(),
        "modified": modified,
    });
    let result = serde_json::to_string(&json)
        .map_err(|e| VMError::RuntimeError(format!("metadata json: {}", e)))?;
    let str_idx = vm.add_string(result.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Recursive copy of directory or file.
/// Stack: str_idx(dst), str_idx(src) -> void
pub fn shim_fs_copy_recursive(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let dst_idx = task.ram.pop_str_idx() as u32;
    let src_idx = task.ram.pop_str_idx() as u32;
    let dst = vm
        .get_string(dst_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let src = vm
        .get_string(src_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    copy_recursive(&src, &dst)
        .map_err(|e| VMError::RuntimeError(format!("copy_recursive failed: {}", e)))?;
    Ok(())
}

/// Extract filename from path.
/// Stack: str_idx(path) -> str_idx (filename)
pub fn shim_fs_filename(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let filename = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(filename.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Get parent directory of path.
/// Stack: str_idx(path) -> str_idx (parent)
pub fn shim_fs_parent(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path_idx = task.ram.pop_str_idx() as u32;
    let path = vm
        .get_string(path_idx)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_default();
    let parent = std::path::Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();
    let str_idx = vm.add_string(parent.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}

/// Join path components.
/// Stack: str_idx(b), str_idx(a) -> str_idx (joined path)
pub fn shim_fs_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let b_idx = task.ram.pop_str_idx() as u32;
    let a_idx = task.ram.pop_str_idx() as u32;
    let b = vm
        .get_string(b_idx)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default();
    let a = vm
        .get_string(a_idx)
        .map(|bytes| String::from_utf8_lossy(&bytes).to_string())
        .unwrap_or_default();
    let joined = std::path::Path::new(&a)
        .join(&b)
        .to_string_lossy()
        .to_string();
    let str_idx = vm.add_string(joined.into_bytes());
    vm.rc_push_str_idx(task, str_idx as usize);
    Ok(())
}
