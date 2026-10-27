//! PLAN-843 结构拆分：file_io 域实现（原样搬移，算法/行为零变化）。
use super::*;

/// Read text content from a file
/// PLAN-738 裁决②（2026-10-08）：单一权威名 `auto.fs.read_text`——原多名
/// （File.read_text/auto.file.read_text/auto.fs.read）系标准库多轮搭建期
/// 混淆；File 的 read_text **方法**（self，无 path 参）语义不同，不得与本
/// 独立函数共名。`File.read_text(p)` 等历史调用点经 canonical
/// File.*→auto.file.* + NATIVE_ID_ENTRIES 固定 id 别名解析到同一 shim 槽。
#[auto_macros::rust_fn("auto.fs.read_text")]
pub fn shim_file_read_text(path: String) -> String {
    resolve_request_file_path(&path)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .unwrap_or_default()
}

/// Read at most `limit` bytes of UTF-8 text from `path` starting at byte
/// `offset` (Plan 673 §5 chunked read).
///
/// Shapes: success `{"text":...,"total":<file byte len>,"next_offset":
/// <offset+text.len()>|null}`; EOF / parameter / IO error per the design
/// table (`next_offset` null ONLY at true EOF — disk-tail short reads must
/// not fake EOF; `offset<0` or `limit<=0` → `total:-1`). Chunk edges never
/// split a char: a mid-char `offset` walks back to the preceding boundary,
/// and the chunk end backs off likewise. Invalid UTF-8 → error shape.
///
/// PLAN-687 true chunked IO (byte-identical with the a2r-std mirror,
/// P670-D1): the disk window actually read is
/// `[offset − 3, min(total, offset + limit + 1))` via `File::seek` +
/// exact-size read; `total` comes from metadata — no whole-file `fs::read`,
/// so a chunked scan costs O(limit) IO per call. Streaming semantic note:
/// UTF-8 validation is per-chunk-region; invalid bytes outside the window
/// surface as the error shape only when their containing chunk is read.
#[auto_macros::rust_fn("File.read_text_range", "auto.file.read_text_range")]
pub fn shim_file_read_text_range(path: String, offset: i32, limit: i32) -> String {
    let err = || read_text_range_json(String::new(), -1, None);
    if offset < 0 || limit <= 0 {
        return err();
    }
    use std::io::{Read, Seek, SeekFrom};
    let path = match resolve_request_file_path(&path) {
        Ok(path) => path,
        Err(_) => return err(),
    };
    let mut file = match fs::File::open(&path) {
        Ok(f) => f,
        Err(_) => return err(),
    };
    let total = match file.metadata() {
        Ok(m) => m.len(),
        Err(_) => return err(),
    };
    let start = (offset as u64).min(total);
    if start >= total {
        // offset past end (empty file included): EOF shape, real total.
        return read_text_range_json(String::new(), total as i64, None);
    }
    let end = (start as usize)
        .saturating_add(limit as usize)
        .min(total as usize) as u64;
    // Boundary margins: 3 bytes before `start` (max UTF-8 continuation run)
    // + 1 byte at `end` (decides whether `end` sits inside a multi-byte
    // char). Continuation bytes = 0b10xxxxxx (std-free walk, same result
    // as the previous whole-text `is_char_boundary` back-off).
    let win_start = start.saturating_sub(3);
    let win_end = (end + 1).min(total);
    let mut win = vec![0u8; (win_end - win_start) as usize];
    if file.seek(SeekFrom::Start(win_start)).is_err() {
        return err();
    }
    if file.read_exact(&mut win).is_err() {
        return err();
    }
    let is_cont = |b: u8| b & 0xC0 == 0x80;
    let mut s = (start - win_start) as usize;
    while s > 0 && is_cont(win[s]) {
        s -= 1;
    }
    let mut e = (end - win_start) as usize;
    if e < win.len() {
        while e > s && is_cont(win[e]) {
            e -= 1;
        }
    }
    let chunk = match String::from_utf8(win[s..e].to_vec()) {
        Ok(t) => t,
        Err(_) => return err(),
    };
    let end_abs = (e + win_start as usize) as u64;
    let next_offset = if end_abs >= total {
        None
    } else {
        Some(end_abs)
    };
    read_text_range_json(chunk, total as i64, next_offset)
}

pub fn shim_file_write_text(path: String, content: String) -> i32 {
    if let Ok(path) = resolve_request_file_path(&path) {
        let _ = fs::write(path, &content);
    }
    0
}

/// Check if a file exists
#[auto_macros::rust_fn("File.exists", "auto.file.exists", "auto.fs.exists")]
pub fn shim_file_exists(path: String) -> i32 {
    if resolve_request_file_path(&path).is_ok_and(|path| fs::metadata(path).is_ok()) {
        1
    } else {
        0
    }
}

/// Delete a file
#[auto_macros::rust_fn("File.delete", "auto.fs.delete")]
pub fn shim_file_delete(path: String) -> i32 {
    if let Ok(path) = resolve_request_file_path(&path) {
        let _ = fs::remove_file(path);
    }
    0
}

/// Create a directory
#[auto_macros::rust_fn("File.create_dir", "auto.fs.create_dir")]
pub fn shim_file_create_dir(path: String) -> i32 {
    if let Ok(path) = resolve_request_file_path(&path) {
        let _ = fs::create_dir_all(path);
    }
    0
}

/// Remove an empty directory
#[auto_macros::rust_fn("File.remove_dir")]
pub fn shim_file_remove_dir(path: String) -> Result<(), String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.remove_dir failed: {} - {}", path, e))?;
    fs::remove_dir(&path).map_err(|e| format!("File.remove_dir failed: {} - {}", path.display(), e))
}

/// Remove a directory and all its contents recursively
#[auto_macros::rust_fn("File.remove_dir_all")]
pub fn shim_file_remove_dir_all(path: String) -> Result<(), String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.remove_dir_all failed: {} - {}", path, e))?;
    fs::remove_dir_all(&path)
        .map_err(|e| format!("File.remove_dir_all failed: {} - {}", path.display(), e))
}

/// Read file contents as bytes
#[auto_macros::rust_fn("File.read_bytes")]
pub fn shim_file_read_bytes(path: String) -> Result<Vec<i32>, String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.read_bytes failed: {} - {}", path, e))?;
    match fs::read(&path) {
        Ok(bytes) => Ok(bytes.into_iter().map(|b| b as i32).collect()),
        Err(e) => Err(format!(
            "File.read_bytes failed: {} - {}",
            path.display(),
            e
        )),
    }
}

/// Write bytes to a file
#[auto_macros::rust_fn("File.write_bytes")]
pub fn shim_file_write_bytes(path: String, byte_list: Vec<i32>) -> Result<(), String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.write_bytes failed: {} - {}", path, e))?;
    let bytes: Vec<u8> = byte_list.into_iter().map(|b| b as u8).collect();
    fs::write(&path, &bytes)
        .map_err(|e| format!("File.write_bytes failed: {} - {}", path.display(), e))
}

/// Copy a file
#[auto_macros::rust_fn("File.copy")]
pub fn shim_file_copy(src: String, dst: String) -> Result<(), String> {
    let src_path = resolve_request_file_path(&src)
        .map_err(|e| format!("File.copy failed: {} -> {} - {}", src, dst, e))?;
    let dst_path = resolve_request_file_path(&dst)
        .map_err(|e| format!("File.copy failed: {} -> {} - {}", src, dst, e))?;
    fs::copy(&src_path, &dst_path).map(|_| ()).map_err(|e| {
        format!(
            "File.copy failed: {} -> {} - {}",
            src_path.display(),
            dst_path.display(),
            e
        )
    })
}

/// Get file size in bytes
#[auto_macros::rust_fn("File.size")]
pub fn shim_file_size(path: String) -> Result<i64, String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.size failed: {} - {}", path, e))?;
    fs::metadata(&path)
        .map(|meta| meta.len() as i64)
        .map_err(|e| format!("File.size failed: {} - {}", path.display(), e))
}

/// Get file modification time as epoch seconds (PLAN-016 T-05).
/// 列表物化不走 metadata JSON（JsonValue 可选字段读取在 VM 轨产出 None
/// 级联 TypeError）——mtime/len/is_dir 各走直连 native。失败/1970 前 = -1。
#[auto_macros::rust_fn("auto.fs.mtime")]
pub fn shim_fs_mtime(path: String) -> Result<i64, String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.mtime failed: {} - {}", path, e))?;
    let secs = fs::metadata(&path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64)
        .unwrap_or(-1);
    Ok(secs)
}

/// Check if path is a directory
#[auto_macros::rust_fn("File.is_dir")]
pub fn shim_file_is_dir(path: String) -> bool {
    resolve_request_file_path(&path)
        .ok()
        .and_then(|path| fs::metadata(path).ok())
        .map(|m| m.is_dir())
        .unwrap_or(false)
}

/// Walk a directory tree, returning all file paths as a JSON array.
#[auto_macros::rust_fn("auto.file.walk")]
pub fn shim_file_walk(path: String) -> Result<String, String> {
    let path_resolved =
        resolve_request_file_path(&path).map_err(|e| format!("File.walk failed: {}", e))?;
    let root = Path::new(&path_resolved);
    if !root.exists() {
        return Err(format!("File.walk failed: path not found: {}", path));
    }

    let mut files: Vec<String> = Vec::new();
    if root.is_file() {
        files.push(path_resolved.to_string_lossy().to_string());
    } else {
        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Some(p) = entry.path().to_str() {
                    files.push(p.to_string());
                }
            }
        }
    }

    serde_json::to_string(&files).map_err(|e| format!("JSON serialization failed: {}", e))
}

/// Append text content to a file (creates if doesn't exist)
#[auto_macros::rust_fn("File.append_text")]
pub fn shim_file_append_text(path: String, content: String) -> Result<(), String> {
    let path_buf = resolve_request_file_path(&path)
        .map_err(|e| format!("File.append_text failed: {} - {}", path, e))?;
    // Ensure parent directory exists
    if let Some(parent) = path_buf.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("File.append_text failed to create dir: {}", e))?;
        }
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path_buf)
        .map_err(|e| format!("File.append_text failed: {} - {}", path_buf.display(), e))?;
    use std::io::Write;
    file.write_all(content.as_bytes())
        .map_err(|e| format!("File.append_text failed: {} - {}", path, e))?;
    Ok(())
}

/// Read file contents as an array of lines (JSON string array)
#[auto_macros::rust_fn("auto.file.read_lines")]
pub fn shim_file_read_lines(path: String) -> Result<String, String> {
    let path = resolve_request_file_path(&path)
        .map_err(|e| format!("File.read_lines failed: {} - {}", path, e))?;
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("File.read_lines failed: {} - {}", path.display(), e))?;
    let lines: Vec<String> = content.lines().map(|l| l.to_string()).collect();
    serde_json::to_string(&lines).map_err(|e| format!("JSON serialization failed: {}", e))
}

/// Check if a file contains null bytes (binary detection). Returns 1 if binary, 0 if text.
#[auto_macros::rust_fn("auto.fs.is_binary")]
pub fn shim_fs_is_binary(path: String) -> i32 {
    let Ok(path) = resolve_request_file_path(&path) else {
        return 0;
    };
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(_) => return 0,
    };
    let check_len = bytes.len().min(8192);
    if bytes[..check_len].contains(&0) {
        1
    } else {
        0
    }
}

/// Read a line from stdin
#[auto_macros::rust_fn("IO.read_line")]
pub fn shim_io_read_line() -> String {
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).ok();
    // Trim trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}

/// Read one line from the current request's finite input, falling back to
/// process stdin for existing CLI callers.
pub fn shim_io_read_line_request(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::convert::VMConvertible;
    let line = match vm.read_request_input_line() {
        Some(Some(line)) => line,
        Some(None) => String::new(),
        None => read_process_stdin_line().unwrap_or_default(),
    };
    line.push_to_stack(task, vm)
        .map_err(|error| VMError::RuntimeError(error.to_string()))
}

/// Read one line while distinguishing a blank line from EOF.
pub fn shim_io_read_line_opt(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::generic_registry::GenericInstanceData;
    let line = match vm.read_request_input_line() {
        Some(line) => line,
        None => read_process_stdin_line(),
    };
    let (variant, fields, names) = match line {
        Some(line) => (
            "Option.Some",
            vec![auto_val::Value::Str(auto_val::AutoStr::from(line))],
            vec!["_0".to_string()],
        ),
        None => ("Option.None", Vec::new(), Vec::new()),
    };
    let option = GenericInstanceData::new_with_names(variant.to_string(), fields, names);
    let handle = vm.insert_heap_object(option) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

/// `io.lines(path: String) -> iterator_id`
/// Stream lines from a file using BufReader. Non-blocking (Plan 348 yield).
pub fn shim_io_lines(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path = resolve_request_file_path(&path).map_err(VMError::RuntimeError)?;

    let stream_id = alloc_async_id();
    let (tx, rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);
    let handle = Arc::new(AsyncStreamHandle {
        rx: std::sync::Mutex::new(rx),
        done: std::sync::atomic::AtomicBool::new(false),
    });
    let handle_clone = handle.clone();

    std::thread::Builder::new()
        .name("auto-io-lines".into())
        .spawn(move || {
            use std::io::BufRead;
            let file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                    let _ = tx.blocking_send(AsyncStreamEvent::Done);
                    handle_clone.done.store(true, Ordering::SeqCst);
                    return;
                }
            };
            let reader = std::io::BufReader::new(file);
            for line_result in reader.lines() {
                match line_result {
                    Ok(line) => {
                        let _ = tx.blocking_send(AsyncStreamEvent::Data(line));
                    }
                    Err(e) => {
                        let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
            let _ = tx.blocking_send(AsyncStreamEvent::Done);
            handle_clone.done.store(true, Ordering::SeqCst);
        })
        .expect("spawn io-lines thread");

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// `io.chunks(path: String, chunk_size: int) -> iterator_id`
/// Stream fixed-size chunks from a binary file. Non-blocking.
pub fn shim_io_chunks(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let chunk_size: i32 = crate::vm::native::pop_arg_i32(task);
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path = resolve_request_file_path(&path).map_err(VMError::RuntimeError)?;
    let chunk_size = chunk_size.max(1) as usize;

    let stream_id = alloc_async_id();
    let (tx, rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);
    let handle = Arc::new(AsyncStreamHandle {
        rx: std::sync::Mutex::new(rx),
        done: std::sync::atomic::AtomicBool::new(false),
    });
    let handle_clone = handle.clone();

    std::thread::Builder::new()
        .name("auto-io-chunks".into())
        .spawn(move || {
            use std::io::Read;
            let mut file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                    let _ = tx.blocking_send(AsyncStreamEvent::Done);
                    handle_clone.done.store(true, Ordering::SeqCst);
                    return;
                }
            };
            let mut buf = vec![0u8; chunk_size];
            loop {
                match file.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        let chunk_str = String::from_utf8_lossy(&buf[..n]).to_string();
                        let _ = tx.blocking_send(AsyncStreamEvent::Data(chunk_str));
                    }
                    Err(e) => {
                        let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
            let _ = tx.blocking_send(AsyncStreamEvent::Done);
            handle_clone.done.store(true, Ordering::SeqCst);
        })
        .expect("spawn io-chunks thread");

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// `fs.entries(path: String) -> iterator_id`
/// Stream directory entries (name, is_dir, size). Non-blocking.
pub fn shim_fs_entries(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path = resolve_request_file_path(&path).map_err(VMError::RuntimeError)?;

    let stream_id = alloc_async_id();
    let (tx, rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);
    let handle = Arc::new(AsyncStreamHandle {
        rx: std::sync::Mutex::new(rx),
        done: std::sync::atomic::AtomicBool::new(false),
    });
    let handle_clone = handle.clone();

    std::thread::Builder::new()
        .name("auto-fs-entries".into())
        .spawn(move || {
            let entries = match std::fs::read_dir(&path) {
                Ok(e) => e,
                Err(e) => {
                    let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                    let _ = tx.blocking_send(AsyncStreamEvent::Done);
                    handle_clone.done.store(true, Ordering::SeqCst);
                    return;
                }
            };
            for entry_result in entries {
                match entry_result {
                    Ok(entry) => {
                        let name = entry.file_name().to_string_lossy().to_string();
                        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                        let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                        let json = format!(
                            r#"{{"name":"{}","is_dir":{},"size":{}}}"#,
                            name.replace('"', "\\\""),
                            is_dir,
                            size
                        );
                        let _ = tx.blocking_send(AsyncStreamEvent::Data(json));
                    }
                    Err(e) => {
                        let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                        break;
                    }
                }
            }
            let _ = tx.blocking_send(AsyncStreamEvent::Done);
            handle_clone.done.store(true, Ordering::SeqCst);
        })
        .expect("spawn fs-entries thread");

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// `io.buffered_writer(path: String) -> handle (i32)`
/// Create a buffered file writer (BufWriter<File>).
pub fn shim_io_buffered_writer(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path = resolve_request_file_path(&path).map_err(VMError::RuntimeError)?;

    let file = std::fs::File::create(&path).map_err(|e| {
        VMError::RuntimeError(format!("Failed to create {}: {}", path.display(), e))
    })?;
    let writer = std::io::BufWriter::new(file);
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
        "BufWriter",
        std::sync::Mutex::new(writer),
    );
    let handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

/// `io.buffered_reader(path: String) -> handle (i32)`
/// Create a buffered file reader (BufReader<File>).
pub fn shim_io_buffered_reader(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path = resolve_request_file_path(&path).map_err(VMError::RuntimeError)?;

    let file = std::fs::File::open(&path)
        .map_err(|e| VMError::RuntimeError(format!("Failed to open {}: {}", path.display(), e)))?;
    let reader = std::io::BufReader::new(file);
    let obj = crate::vm::ffi::rust_stdlib::RustStdlibObject::new(
        "BufReader",
        std::sync::Mutex::new(reader),
    );
    let handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

/// `io.open(path: String) -> handle (i32)`
/// Open a file for random access (seek + read/write).
pub fn shim_io_open(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path = resolve_request_file_path(&path).map_err(VMError::RuntimeError)?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .map_err(|e| VMError::RuntimeError(format!("Failed to open {}: {}", path.display(), e)))?;
    let obj =
        crate::vm::ffi::rust_stdlib::RustStdlibObject::new("File", std::sync::Mutex::new(file));
    let handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

/// `File.seek(handle: i32, offset: i64) -> i64`
/// Seek to absolute offset. Returns new position.
pub fn shim_file_seek(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let offset: i64 = crate::vm::native::pop_arg_i32(task) as i64;
    let handle: i32 = crate::vm::native::pop_arg_i32(task);
    use std::io::{Seek, SeekFrom};
    let pos = if let Some(obj) = vm.get_heap_object(handle as u64) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<std::fs::File>>() {
                if let Ok(mut f) = mutex.lock() {
                    f.seek(SeekFrom::Start(offset as u64)).unwrap_or(0) as i64
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };
    task.ram.push_i32(pos as i32);
    Ok(())
}

/// `File.read(handle: i32, count: int) -> String`
/// Read `count` bytes from file at current position.
pub fn shim_file_read(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let count: i32 = crate::vm::native::pop_arg_i32(task);
    let handle: i32 = crate::vm::native::pop_arg_i32(task);
    use std::io::Read;
    let data = if let Some(obj) = vm.get_heap_object(handle as u64) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<std::fs::File>>() {
                if let Ok(mut f) = mutex.lock() {
                    let mut buf = vec![0u8; count.max(0) as usize];
                    let n = f.read(&mut buf).unwrap_or(0);
                    String::from_utf8_lossy(&buf[..n]).to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let idx = vm.add_string(data.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `File.position(handle: i32) -> i64`
/// Get current file position.
pub fn shim_file_position(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i32 = crate::vm::native::pop_arg_i32(task);
    use std::io::Seek;
    let pos = if let Some(obj) = vm.get_heap_object(handle as u64) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<std::fs::File>>() {
                if let Ok(mut f) = mutex.lock() {
                    f.stream_position().unwrap_or(0) as i64
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        }
    } else {
        0
    };
    task.ram.push_i32(pos as i32);
    Ok(())
}

/// `Path.new(path: String) -> handle`
/// Create a Path object for chainable path operations.
pub fn shim_path_new(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let obj =
        crate::vm::ffi::rust_stdlib::RustStdlibObject::new("Path", std::sync::Mutex::new(path));
    let handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(handle));
    Ok(())
}

/// `Path.join(handle: i32, other: String) -> handle`
/// Join a path component. Returns new Path handle.
pub fn shim_path_join(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let other: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle: i32 = crate::vm::native::pop_arg_i32(task);
    let new_path = if let Some(obj) = vm.get_heap_object(handle as u64) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<String>>() {
                if let Ok(p) = mutex.lock() {
                    std::path::Path::new(&*p)
                        .join(&other)
                        .to_string_lossy()
                        .to_string()
                } else {
                    other.clone()
                }
            } else {
                other.clone()
            }
        } else {
            other.clone()
        }
    } else {
        other
    };
    let obj =
        crate::vm::ffi::rust_stdlib::RustStdlibObject::new("Path", std::sync::Mutex::new(new_path));
    let new_handle = vm.insert_heap_object(obj) as u32;
    vm.rc_push(task, auto_val::encode_object(new_handle));
    Ok(())
}

/// `Path.exists(handle: i32) -> bool`
pub fn shim_path_exists(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i32 = crate::vm::native::pop_arg_i32(task);
    let exists = if let Some(obj) = vm.get_heap_object(handle as u64) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<String>>() {
                if let Ok(p) = mutex.lock() {
                    std::path::Path::new(&*p).exists()
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };
    task.ram.push_nv(auto_val::encode_bool(exists));
    Ok(())
}

/// `Path.to_string(handle: i32) -> String`
pub fn shim_path_to_string(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle: i32 = crate::vm::native::pop_arg_i32(task);
    let path_str = if let Some(obj) = vm.get_heap_object(handle as u64) {
        let guard = obj.read().unwrap();
        if let Some(rso) = guard
            .as_any()
            .downcast_ref::<crate::vm::ffi::rust_stdlib::RustStdlibObject>()
        {
            if let Some(mutex) = rso.downcast_ref::<std::sync::Mutex<String>>() {
                if let Ok(p) = mutex.lock() {
                    p.clone()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    let idx = vm.add_string(path_str.into_bytes());
    vm.rc_push_str_idx(task, idx as usize);
    Ok(())
}

/// `io.scanner(path: String, delimiter: String) -> iterator_id`
/// Stream tokens split by delimiter from a file. Non-blocking.
pub fn shim_io_scanner(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let delimiter: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let stream_id = alloc_async_id();
    let (tx, rx) = tokio::sync::mpsc::channel::<AsyncStreamEvent>(64);
    let handle = Arc::new(AsyncStreamHandle {
        rx: std::sync::Mutex::new(rx),
        done: std::sync::atomic::AtomicBool::new(false),
    });
    let handle_clone = handle.clone();

    std::thread::Builder::new()
        .name("auto-io-scanner".into())
        .spawn(move || {
            use std::io::Read;
            let mut file = match std::fs::File::open(&path) {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.blocking_send(AsyncStreamEvent::Error(e.to_string()));
                    let _ = tx.blocking_send(AsyncStreamEvent::Done);
                    handle_clone.done.store(true, Ordering::SeqCst);
                    return;
                }
            };
            let mut content = String::new();
            let _ = file.read_to_string(&mut content);
            for token in content.split(delimiter.as_str()) {
                let _ = tx.blocking_send(AsyncStreamEvent::Data(token.to_string()));
            }
            let _ = tx.blocking_send(AsyncStreamEvent::Done);
            handle_clone.done.store(true, Ordering::SeqCst);
        })
        .expect("spawn io-scanner thread");

    if let Ok(mut map) = ASYNC_STREAMS.lock() {
        map.insert(stream_id, handle);
    }

    let async_iter = crate::vm::engine::AsyncStreamIterator {
        stream_id,
        done: false,
    };
    let iter_id = {
        let next_id = vm
            .iterator_id_gen
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        vm.iterators.insert(
            next_id,
            crate::vm::engine::Iterator::AsyncHttpStream(async_iter),
        );
        next_id
    };
    task.ram.push_i32(iter_id as i32);
    Ok(())
}

/// `io.read_text_async(path: String) -> String`
/// Async read file to string. Non-blocking (yield pattern from Plan 349 step7).
pub fn shim_io_read_text_async(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping args. The engine retries this CALL_NAT
    // after a yield (rewinds IP), so on re-entry the `path` arg was already
    // popped on the first call. Popping again would eat the wrong stack value
    // and underflow. (Same fix as Plan 340's HTTP shims.)
    if let Some(req_id) = task.waiting_http_request_id {
        let result = crate::vm::ffi::async_http::take_live_op(req_id);
        if let Some(Ok(AsyncResult::Body(content))) = result {
            task.waiting_http_request_id = None;
            let idx = vm.add_string(content.into_bytes());
            vm.rc_push_str_idx(task, idx);
            return Ok(());
        }
        if let Some(Err(e)) = result {
            task.waiting_http_request_id = None;
            let idx = vm.add_string(e.into_bytes());
            vm.rc_push_str_idx(task, idx);
            return Ok(());
        }
        // Still pending — yield again.
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    // First call — pop the path, register the live token, spawn the read and
    // yield. PLAN-705 T-02: 登记先于提交 worker（协议顺序，完成端只在
    // live 令牌上投递）。
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_id);
    let path_for_thread = path.clone();
    std::thread::spawn(move || {
        let result = std::fs::read_to_string(&path_for_thread)
            .map(|s| s)
            .map_err(|e| e.to_string());
        // 迟到完成（已被取消）由 complete_live_op 的 presence 守卫丢弃。
        let _ = crate::vm::ffi::async_http::complete_live_op(req_id, result.map(AsyncResult::Body));
    });
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `io.write_text_async(path: String, content: String) -> bool`
/// Async write string to file. Non-blocking.
pub fn shim_io_write_text_async(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // Re-entry check BEFORE popping args (see shim_io_read_text_async / Plan 340
    // HTTP shims for rationale: the engine rewinds IP on yield, so args were
    // already popped on the first call).
    if let Some(req_id) = task.waiting_http_request_id {
        let result = crate::vm::ffi::async_http::take_live_op(req_id);
        if let Some(result) = result {
            task.waiting_http_request_id = None;
            let ok = result.is_ok();
            task.ram.push_nv(auto_val::encode_bool(ok));
            return Ok(());
        }
        task.status = crate::vm::task::TaskStatus::Waiting("http".into());
        return Ok(());
    }

    let content: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path: String = super::super::convert::VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let req_id = alloc_async_id();
    crate::vm::ffi::async_http::register_live_op(req_id);
    let path_t = path.clone();
    let content_t = content.clone();
    std::thread::spawn(move || {
        let result = std::fs::write(&path_t, &content_t)
            .map(|_| "ok".to_string())
            .map_err(|e| e.to_string());
        let _ = crate::vm::ffi::async_http::complete_live_op(req_id, result.map(AsyncResult::Body));
    });
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// Get parent directory of a path
#[auto_macros::rust_fn("Path.parent")]
pub fn shim_path_parent(path: String) -> String {
    Path::new(&path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Get file extension of a path
#[auto_macros::rust_fn("Path.extension")]
pub fn shim_path_extension(path: String) -> String {
    Path::new(&path)
        .extension()
        .map(|e| e.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Get filename from a path
#[auto_macros::rust_fn("Path.filename")]
pub fn shim_path_filename(path: String) -> String {
    Path::new(&path)
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Canonicalize a path (resolve symlinks, .., .)
#[auto_macros::rust_fn("Path.canonicalize")]
pub fn shim_path_canonicalize(path: String) -> String {
    std::fs::canonicalize(&path)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Plan 744 D9 runtime surface: flat child file names (receipts enumeration
/// for the NOTES-001 Phase-1 JSON -> SQLite upgrade). Delegates to
/// a2r_std::fs::dir_names (same fn the a2r track routes to).
pub fn shim_fs_dir_names(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let names: Vec<String> = a2r_std::fs::dir_names(path.as_str());
    VMConvertible::push_to_stack(&names, task, vm).map_err(|e| VMError::RuntimeError(e.to_string()))
}
