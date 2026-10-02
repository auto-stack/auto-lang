//! PLAN-727 T-05：VM 文件传输桥——a2r-std 共享传输核心（`a2r_std::http`
//! `transfer` 模块）的宿主面。
//!
//! 桥接形状（PLAN-727 §2 冻结）：
//! - **注册表**：`VM_TRANSFERS`（transfer id → `TransferEntry`）。条目持有
//!   [`FileTransfer`]（保活防 Drop 取消）+ 非拥有观察句柄；wait 消费 /
//!   scope 组收口时移除。
//! - **等待**：复用 705/707 的 live-op 单次终结协议——观察句柄在共享内核
//!   runtime 上等终态 → `complete_live_op` 唤醒 parked task
//!   （`waiting_http_request_id` + `Waiting("http")`，CALL_NAT 重入消费）。
//!   丢弃等待不取消传输；取消传输走 `transfer_cancel` / scope 级联。
//! - **scope**：handler 段内提交的传输经 `register_scope_transfer` 登记，
//!   `finalize_scope` 组收口（取消 + 出注册表），不越过请求生命期。
//! - **legacy 迁移**：`http.download/download_resume/upload` 与进度迭代器
//!   的执行全部迁入共享核心；bool/Response/iterator 形状逐字节保留
//!   （差异登记见 docs/plans/reports/727-transfer-decision.md §4）。

use std::collections::HashMap;

use crate::vm::ffi::convert::VMConvertible;
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;

use std::sync::atomic::Ordering;

use super::async_http;
use super::stdlib::{alloc_async_id, AsyncResult};

lazy_static::lazy_static! {
    /// VM 文件传输注册表：transfer id（全局单调，不复用）→ 条目。
    /// 生命周期：insert 于提交；remove 于 wait 消费 / scope 组收口。
    pub(crate) static ref VM_TRANSFERS: std::sync::Mutex<HashMap<u64, TransferEntry>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 注册表条目：own 句柄（Drop 取消权归宿主）+ 非拥有观察句柄（watcher 用）。
pub(crate) struct TransferEntry {
    pub transfer: FileTransfer,
    pub observer: TransferObserver,
}

use a2r_std::http::{FileTransfer, TransferObserver};

/// 登记并返回 transfer id。
pub(crate) fn insert_transfer(transfer: FileTransfer) -> u64 {
    let id = transfer.id();
    let observer = transfer.observer();
    if let Ok(mut map) = VM_TRANSFERS.lock() {
        map.insert(id, TransferEntry { transfer, observer });
    }
    id
}

/// 消费移除（wait 交付 / scope 收口）。
fn take_transfer(id: u64) {
    if let Ok(mut map) = VM_TRANSFERS.lock() {
        map.remove(&id);
    }
}

/// 非夺取式观察（progress/cancel/error/wait 快路径可重复查询）。
fn with_observer<R>(id: u64, f: impl FnOnce(&TransferObserver) -> R, default: R) -> R {
    let guard = VM_TRANSFERS.lock().ok();
    match guard.as_ref().and_then(|m| m.get(&id)) {
        Some(entry) => f(&entry.observer),
        None => default,
    }
}

// 等待令牌映射：live-op req_id → transfer id（消费时出注册表）。
static WATCH_TO_TRANSFER: std::sync::LazyLock<std::sync::Mutex<HashMap<u64, u64>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(HashMap::new()));

/// 提交终态 watcher（park 通知面）：观察句柄在共享内核 runtime 上等终态
/// JSON → live-op `Body` → owner loop 唤醒 → CALL_NAT 重入消费。
fn spawn_terminal_watcher(observer: &TransferObserver, transfer_id: u64) -> u64 {
    let req_id = alloc_async_id();
    async_http::register_live_op(req_id);
    WATCH_TO_TRANSFER
        .lock()
        .unwrap()
        .insert(req_id, transfer_id);
    let obs = observer.clone();
    a2r_std::http::client::kernel_handle().spawn(async move {
        let json = obs.wait_json().await;
        async_http::complete_live_op(req_id, Ok(AsyncResult::Body(json)));
    });
    req_id
}

/// 等待消费：取回收据 JSON 并出注册表（Pending → None 保持 live）。
fn take_watch(req_id: u64) -> Option<String> {
    // Pending → check 返回 None；Completed → take 消费（Err 也消费为空）。
    let json = match super::stdlib::check_async_http_result(req_id)? {
        Ok(j) => j,
        Err(_) => String::new(),
    };
    if let Ok(mut m) = WATCH_TO_TRANSFER.lock() {
        if let Some(transfer_id) = m.remove(&req_id) {
            take_transfer(transfer_id);
        }
    }
    Some(json)
}

// ============================================================================
// 新面 shims（http.transfer_* 自由函数词汇；声明见 http.at / http.vm.at）
// ============================================================================

/// `http.transfer_download(url, path, options) -> FileTransfer`（非阻塞提交）。
pub fn shim_http_transfer_download(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let options: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let transfer = a2r_std::http::transfer_download(&url, &path, &options);
    super::http_server::register_scope_transfer(transfer.id());
    let id = insert_transfer(transfer);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// `http.transfer_upload(url, path, options) -> FileTransfer`（非阻塞提交）。
pub fn shim_http_transfer_upload(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let options: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let url: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let transfer = a2r_std::http::transfer_upload(&url, &path, &options);
    super::http_server::register_scope_transfer(transfer.id());
    let id = insert_transfer(transfer);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// `http.transfer_wait(t) -> str`：park 至终态，重入交付收据 JSON。
/// 等待令牌挂 `waiting_http_request_id`（单 task 同时至多一个 native 等待
/// ——CALL_NAT 语义保证；与 builder send 复用同字段不同生命周期）。
pub fn shim_http_transfer_wait(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    // 重入先于弹栈（首调已弹 handle；引擎回卷 IP 重入）。
    if let Some(req_id) = task.waiting_http_request_id {
        match take_watch(req_id) {
            Some(json) => {
                task.waiting_http_request_id = None;
                VMConvertible::push_to_stack(&json, task, _vm)?;
                return Ok(());
            }
            None => {
                task.status = crate::vm::task::TaskStatus::Waiting("http".into());
                return Ok(());
            }
        }
    }
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    // 终态快路径：无需 park（终态单次交付经注册表移除保证）。
    let terminal = with_observer(
        handle,
        |o| {
            if o.is_terminal() {
                Some(o.next_progress_json())
            } else {
                None
            }
        },
        None,
    );
    if let Some(json) = terminal {
        take_transfer(handle);
        VMConvertible::push_to_stack(&json, task, _vm)?;
        return Ok(());
    }
    let entry_present = VM_TRANSFERS
        .lock()
        .map(|m| m.contains_key(&handle))
        .unwrap_or(false);
    if !entry_present {
        // 未知/已消费 id：可观察错误（不挂死）。
        VMConvertible::push_to_stack(
            &r#"{"kind":"failed","status":null,"bytes":0,"total":null,"headers":{},"error":{"kind":"options","message":"unknown transfer id"},"body":""}"#.to_string(),
            task,
            _vm,
        )?;
        return Ok(());
    }
    let req_id = {
        let guard = VM_TRANSFERS.lock().unwrap();
        let entry = guard.get(&handle).expect("checked above");
        spawn_terminal_watcher(&entry.observer, handle)
    };
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `http.transfer_next_progress(t) -> str`（非阻塞；终态收据单次交付）。
pub fn shim_http_transfer_next_progress(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    let json = with_observer(handle, |o| o.next_progress_json(), String::new());
    VMConvertible::push_to_stack(&json, task, _vm)?;
    Ok(())
}

/// `http.transfer_cancel(t)`（幂等；原目标保持，在途 FS 收口后终态）。
pub fn shim_http_transfer_cancel(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    with_observer(handle, |o| o.cancel(), ());
    Ok(())
}

/// `http.transfer_error(t) -> str`（"" = 无/未终结）。
pub fn shim_http_transfer_error(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    let msg = with_observer(handle, |o| o.error_message(), String::new());
    VMConvertible::push_to_stack(&msg, task, _vm)?;
    Ok(())
}

// ============================================================================
// legacy 迁移面（stdlib.rs 旧 shim 的执行体；形状保留见决策报告 §4）
// ============================================================================

/// legacy download/download_resume 的提交 + park。`options` 由调用方按
/// legacy ABI 组装（download = "{}"；resume = `{"offset":N}`，offset 自
/// i32 栈——既有 32 位 ABI 事实，负数经 options 解析成为可观察失败）。
pub(crate) fn start_legacy_download_and_park(
    task: &mut AutoTask,
    url: &str,
    path: &str,
    options: &str,
) -> Result<(), VMError> {
    let transfer = a2r_std::http::transfer_download(url, path, options);
    super::http_server::register_scope_transfer(transfer.id());
    let id = insert_transfer(transfer);
    let req_id = {
        let guard = VM_TRANSFERS.lock().unwrap();
        let entry = guard.get(&id).expect("just inserted");
        spawn_terminal_watcher(&entry.observer, id)
    };
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// legacy download 重入消费：kind==success → true（含 Err/未知 → false）。
pub(crate) fn consume_legacy_download(task: &mut AutoTask, req_id: u64) -> bool {
    let ok = match take_watch(req_id) {
        Some(json) => json.contains("\"kind\":\"success\""),
        None => false,
    };
    task.waiting_http_request_id = None;
    ok
}

/// legacy upload 的提交 + park（multipart field=file，对齐旧 Form::file）。
pub(crate) fn start_legacy_upload_and_park(
    task: &mut AutoTask,
    url: &str,
    path: &str,
) -> Result<(), VMError> {
    let transfer = a2r_std::http::transfer_upload(url, path, r#"{"field":"file"}"#);
    super::http_server::register_scope_transfer(transfer.id());
    let id = insert_transfer(transfer);
    let req_id = alloc_async_id();
    async_http::register_live_op(req_id);
    WATCH_TO_TRANSFER.lock().unwrap().insert(req_id, id);
    let observer = {
        let guard = VM_TRANSFERS.lock().unwrap();
        guard.get(&id).expect("just inserted").observer.clone()
    };
    a2r_std::http::client::kernel_handle().spawn(async move {
        // typed 等待：上传 body 需字节保真（JSON lossy 会破坏二进制响应体）。
        let receipt = observer.wait_typed().await;
        let result = Ok(AsyncResult::Structured {
            status: receipt.status.unwrap_or(500),
            headers: receipt.headers,
            body: receipt.body,
        });
        async_http::complete_live_op(req_id, result);
    });
    task.waiting_http_request_id = Some(req_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// legacy upload 重入消费（take 令牌映射 + 注册表条目；Structured 变体）。
pub(crate) fn consume_legacy_upload(
    req_id: u64,
) -> Option<Result<(u16, Vec<(String, String)>, Vec<u8>), String>> {
    let taken = super::stdlib::check_async_http_result_handle(req_id);
    if taken.is_some() {
        if let Ok(mut m) = WATCH_TO_TRANSFER.lock() {
            if let Some(transfer_id) = m.remove(&req_id) {
                take_transfer(transfer_id);
            }
        }
    }
    taken
}

/// 进度迭代器迁移（stdlib.rs spawn_download_with_progress 的执行体）：
/// 共享核心下载 + 观察句柄在内核 runtime 上泵进度/终态进既有
/// AsyncStreamHandle 通道（进度 try_send 满则合并丢弃；终态不丢）。
pub(crate) fn spawn_core_download_with_progress(
    url: &str,
    file_path: &str,
    tx: tokio::sync::mpsc::Sender<crate::vm::ffi::stdlib::AsyncStreamEvent>,
    stream_handle: std::sync::Arc<crate::vm::ffi::stdlib::AsyncStreamHandle>,
) {
    let transfer = a2r_std::http::transfer_download(url, file_path, "");
    let observer = transfer.observer();
    a2r_std::http::client::kernel_handle().spawn(async move {
        // 保活：own 句柄随泵任务存活至终态（终态后 Drop 为 no-op）。
        let _owned = transfer;
        loop {
            observer.progress_or_terminal().await;
            if observer.is_terminal() {
                break;
            }
            let p = observer.latest_progress_json();
            if !p.is_empty() {
                // 进度保留最新值：通道满 = 慢消费者合并丢弃，不反压落盘。
                let _ = tx.try_send(crate::vm::ffi::stdlib::AsyncStreamEvent::Data(p));
            }
        }
        // 终态（不因通道满丢失）：失败 → Error+Done；成功/取消 → 收据+Done。
        let receipt = observer.wait_typed().await;
        match receipt.kind {
            a2r_std::http::TransferOutcome::Failed => {
                let msg = receipt
                    .error
                    .as_ref()
                    .map(|e| e.message.clone())
                    .unwrap_or_else(|| "download failed".to_string());
                let _ = tx.send(crate::vm::ffi::stdlib::AsyncStreamEvent::Error(msg)).await;
                let _ = tx.send(crate::vm::ffi::stdlib::AsyncStreamEvent::Done).await;
            }
            _ => {
                let _ = tx.try_send(crate::vm::ffi::stdlib::AsyncStreamEvent::Data(receipt.json()));
                let _ = tx.send(crate::vm::ffi::stdlib::AsyncStreamEvent::Done).await;
            }
        }
        stream_handle.done.store(true, Ordering::SeqCst);
    });
}

/// scope 组收口：取消组内传输并出注册表（finalize_scope 调用；幂等）。
pub(crate) fn scope_finalize_transfers(group: &[u64]) {
    for id in group {
        // 已终结/未注册 id no-op。
        let _ = a2r_std::http::cancel_transfer_by_id(*id);
        take_transfer(*id);
    }
}

/// 存活注册表条目数（资源回基线探针）。
#[cfg(test)]
pub(crate) fn vm_transfer_count() -> usize {
    VM_TRANSFERS.lock().map(|m| m.len()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// 进度迭代器 producer（迁移面）：relay 泵产出进度/终态事件并以 Done
    /// 终结；目标文件经 staging 提交落盘。消费端（AsyncStreamIterator 臂/
    /// for-in）为既有机制不在本测范围。
    #[test]
    fn plan727_progress_relay_data_then_done() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let body = b"relay-body-123".to_vec();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf);
                let resp = format!(
                    "HTTP/1.1 200 OK
Content-Length: {}
Connection: close

",
                    body.len()
                );
                let _ = stream.write_all(resp.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
        });
        let dir = std::env::temp_dir().join(format!("plan727-relay-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("f.bin");
        let (tx, mut rx) = tokio::sync::mpsc::channel(64);
        let done = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        spawn_core_download_with_progress(
            &format!("http://127.0.0.1:{port}/f"),
            path.to_str().unwrap(),
            tx,
            std::sync::Arc::new(crate::vm::ffi::stdlib::AsyncStreamHandle {
                rx: std::sync::Mutex::new(tokio::sync::mpsc::channel(1).1),
                done: std::sync::atomic::AtomicBool::new(false),
            }),
        );
        let _ = done.clone();
        let mut saw_progress_or_receipt = false;
        let mut saw_done = false;
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while std::time::Instant::now() < deadline {
            match rx.try_recv() {
                Ok(crate::vm::ffi::stdlib::AsyncStreamEvent::Data(ev)) => {
                    assert!(ev.starts_with("{"), "事件须为 JSON: {ev}");
                    saw_progress_or_receipt = true;
                }
                Ok(crate::vm::ffi::stdlib::AsyncStreamEvent::Done) => {
                    saw_done = true;
                    break;
                }
                Ok(crate::vm::ffi::stdlib::AsyncStreamEvent::Error(e)) => {
                    panic!("意外错误事件: {e}");
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
            }
        }
        assert!(saw_progress_or_receipt, "进度/收据事件缺失");
        assert!(saw_done, "Done 终结缺失");
        assert_eq!(std::fs::read(&path).unwrap(), b"relay-body-123", "staging 提交落盘");
        let _ = done;
        std::fs::remove_dir_all(dir).ok();
    }
}
