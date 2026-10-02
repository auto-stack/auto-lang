//! PLAN-730 T-02/T-05：VM 上传桥——`http.upload_*` 的句柄登记与 park 面。
//!
//! 桥接形状（决策报告 §2/§3，727 transfer 样板同构）：
//! - **注册表**：`VM_UPLOAD_REQUESTS`（注入句柄→pending body 能力，take 消费）、
//!   `VM_UPLOAD_SESSIONS`（会话快照）、`VM_UPLOAD_RECEIPTS`（编组取出）。
//! - **park**：receive/commit/reject 复用 live-op 单次终结协议——facade future
//!   在共享内核 runtime 上驱动 → `complete_live_op` 唤醒 parked task
//!   （`waiting_http_request_id` + `Waiting("http")`，CALL_NAT 重入消费）。
//! - **scope**：注入/会话经 `register_scope_upload` 登记，`finalize_scope`
//!   组收口（取消在途接收 + 清理 staged + 释放 body 能力），不越过请求生命期。
//! - **识别**：UploadReceipt 编组 = 声明返回类型含 "UploadReceipt" + i32 +
//!   `take_upload_receipt` 登记命中三重闸（729 file 门同形）；UploadRequest
//!   参数注入由 `bind_api_args_by_name` 的类型分支在 path/body/query/meta
//!   规则**之前**处理。

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use a2r_std::http::{
    UploadReceipt, UploadRequest, UploadSession,
};

use crate::vm::ffi::convert::VMConvertible;
use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;

use super::async_http;
use super::stdlib::{alloc_async_id, AsyncResult};

lazy_static::lazy_static! {
    /// 注入的 UploadRequest 能力（bridge→start_handler 建；receive take 消费）。
    /// 载荷持有原始 body 流（宿主 adapter 已投影为 UploadBodyStream）。
    pub(crate) static ref VM_UPLOAD_REQUESTS: std::sync::Mutex<HashMap<u64, UploadRequest>> =
        std::sync::Mutex::new(HashMap::new());
    /// 会话快照（receive 交付时 insert；commit/reject/scope 收口移除）。
    pub(crate) static ref VM_UPLOAD_SESSIONS: std::sync::Mutex<HashMap<u64, UploadSession>> =
        std::sync::Mutex::new(HashMap::new());
    /// 收据（error/commit/reject 构造 insert；编组 take 取出）。
    pub(crate) static ref VM_UPLOAD_RECEIPTS: std::sync::Mutex<HashMap<u64, UploadReceipt>> =
        std::sync::Mutex::new(HashMap::new());
}

static UPLOAD_ID_GEN: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_upload_id() -> u64 {
    UPLOAD_ID_GEN.fetch_add(1, Ordering::SeqCst)
}

/// 登记注入能力（start_handler：路由声明 UploadRequest 参数时建；scope 组绑定）。
pub(crate) fn insert_upload_request(req: UploadRequest) -> u64 {
    let id = next_upload_id();
    if let Ok(mut m) = VM_UPLOAD_REQUESTS.lock() {
        m.insert(id, req);
    }
    super::http_server::register_scope_upload(id);
    id
}

/// 一次性消费（receive 首调；未知/已消费 id → None）。
fn take_upload_request(id: u64) -> Option<UploadRequest> {
    VM_UPLOAD_REQUESTS.lock().ok().and_then(|mut m| m.remove(&id))
}

/// 会话登记（receive 交付）+ scope 组绑定。
pub(crate) fn insert_upload_session(session: UploadSession) -> u64 {
    let sid = session.id();
    if let Ok(mut m) = VM_UPLOAD_SESSIONS.lock() {
        m.insert(sid, session);
    }
    super::http_server::register_scope_upload(sid);
    sid
}

/// 会话快照查询（metadata 同步路径）。
fn lookup_upload_session(id: u64) -> Option<UploadSession> {
    VM_UPLOAD_SESSIONS.lock().ok().and_then(|m| m.get(&id).cloned())
}

fn take_upload_session(id: u64) -> Option<UploadSession> {
    VM_UPLOAD_SESSIONS.lock().ok().and_then(|mut m| m.remove(&id))
}

/// 收据登记（push 句柄给 VM 前调用）。
pub(crate) fn insert_upload_receipt(receipt: UploadReceipt) -> u64 {
    let id = next_upload_id();
    if let Ok(mut m) = VM_UPLOAD_RECEIPTS.lock() {
        m.insert(id, receipt);
    }
    id
}

/// 编组取出（单次交付；未登记 id → None）。
pub(crate) fn take_upload_receipt(id: u64) -> Option<UploadReceipt> {
    VM_UPLOAD_RECEIPTS.lock().ok().and_then(|mut m| m.remove(&id))
}

// ── live-op 结果表：op_id → 交付物（唤醒经 complete_live_op；结果不走
//    JSON 往返——typed 会话/收据直接存取，take 消费）。─────────────────

pub(crate) enum UploadOpResult {
    Session(UploadSession),
    Receipt(UploadReceipt),
}

lazy_static::lazy_static! {
    static ref UPLOAD_OP_RESULTS: std::sync::Mutex<HashMap<u64, UploadOpResult>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 重入消费（结果存在即取走；同时清理 async result 槽）。
fn take_upload_op(op_id: u64) -> Option<UploadOpResult> {
    let taken = UPLOAD_OP_RESULTS
        .lock()
        .ok()
        .and_then(|mut m| m.remove(&op_id));
    if taken.is_some() {
        // complete_live_op 落的 marker 出 ASYNC_RESULTS 槽（防泄漏）。
        let _ = super::stdlib::check_async_http_result(op_id);
    }
    taken
}

/// 驱动 facade future 至终态并落结果表 + live-op 唤醒（共享内核 runtime）。
/// 返回 live-op id（调用方挂 `waiting_http_request_id`）。
fn spawn_upload_watcher<F>(future: F) -> u64
where
    F: std::future::Future<Output = UploadOpResult> + Send + 'static,
{
    let op_id = alloc_async_id();
    async_http::register_live_op(op_id);
    // 结果先落表（completed 前）→ 唤醒后 take 必命中；迟到消费由 take 清理。
    a2r_std::http::client::kernel_handle().spawn(async move {
        let result = future.await;
        UPLOAD_OP_RESULTS.lock().unwrap().insert(op_id, result);
        // delivered=false = 已取消/已消费（presence 守卫丢弃）→ 结果条目
        // 同步清除，不保留无人消费的载荷（迟到完成不复活）。
        if !async_http::complete_live_op(op_id, Ok(AsyncResult::Body(String::new()))) {
            UPLOAD_OP_RESULTS.lock().unwrap().remove(&op_id);
        }
    });
    op_id
}

// ============================================================================
// shims（native id：upload_receive=9937 upload_metadata=9938 upload_commit=9939
// upload_reject=9940 upload_error=9941；声明见 http.vm.at/http.at）
// ============================================================================

/// `http.upload_receive(req, root, staging_root, options) -> UploadSession`
/// （park 至接收终态；失败也交付 failed 会话——inspect/reject 可用）。
pub fn shim_http_upload_receive(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // 重入先于弹栈（首调已弹参数；引擎回卷 IP 重入）。
    if let Some(op_id) = task.waiting_http_request_id {
        return match take_upload_op(op_id) {
            Some(UploadOpResult::Session(session)) => {
                task.waiting_http_request_id = None;
                let sid = insert_upload_session(session);
                task.ram.push_i32(sid as i32);
                Ok(())
            }
            Some(UploadOpResult::Receipt(_)) => {
                task.waiting_http_request_id = None;
                Err(VMError::RuntimeError(
                    "upload_receive: unexpected receipt result".into(),
                ))
            }
            None => {
                task.status = crate::vm::task::TaskStatus::Waiting("http".into());
                Ok(())
            }
        };
    }
    let options: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let staging_root: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let root: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    let Some(req) = take_upload_request(handle) else {
        // 未知/已消费句柄：可观察失败会话（不挂死；重复 receive 明确冲突）。
        let session = a2r_std::http::failed_session(
            a2r_std::http::UploadErrorKind::SessionConflict,
            "upload_receive: unknown or already-consumed UploadRequest handle",
        );
        let sid = insert_upload_session(session);
        task.ram.push_i32(sid as i32);
        return Ok(());
    };
    // 直连宿主 receive_with_phase（phase hook 透传——scope 期限切换）；
    // executor 未安装时立即返回诊断 failed 会话——watcher 同路径。
    let phase: a2r_std::http::UploadPhaseHook = match super::http_server::current_scope_id() {
        Some(scope_id) => std::sync::Arc::new(move |p| {
            if let a2r_std::http::UploadPhase::ReceiveStarted { total_deadline } = p {
                super::http_server::extend_scope_deadline(scope_id, total_deadline);
            }
        }),
        None => std::sync::Arc::new(|_| {}),
    };
    let op_id = spawn_upload_watcher(async move {
        let session = crate::http_upload_service::receive_with_phase(
            req,
            &root,
            &staging_root,
            &options,
            phase,
        )
        .await;
        UploadOpResult::Session(session)
    });
    task.waiting_http_request_id = Some(op_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `http.upload_metadata(session) -> str`（同步；快照 JSON，含失败建议状态）。
pub fn shim_http_upload_metadata(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    let json = match lookup_upload_session(handle) {
        Some(session) => a2r_std::http::upload_metadata_json(&session),
        None => {
            // 未知句柄：可观察失败形态（与会话冲突一致），不挂死。
            format!(
                "{{\"state\":\"failed\",\"kind\":\"session_conflict\",\"message\":\"unknown session {handle}\",\"suggested_status\":409}}"
            )
        }
    };
    VMConvertible::push_to_stack(&json, task, vm)?;
    Ok(())
}

/// `http.upload_commit(session, relative_target) -> UploadReceipt`（park 至
/// 发布终态；create-only，存在即 409）。
pub fn shim_http_upload_commit(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(op_id) = task.waiting_http_request_id {
        return match take_upload_op(op_id) {
            Some(UploadOpResult::Receipt(receipt)) => {
                task.waiting_http_request_id = None;
                let rid = insert_upload_receipt(receipt);
                task.ram.push_i32(rid as i32);
                Ok(())
            }
            Some(UploadOpResult::Session(_)) => {
                task.waiting_http_request_id = None;
                Err(VMError::RuntimeError(
                    "upload_commit: unexpected session result".into(),
                ))
            }
            None => {
                task.status = crate::vm::task::TaskStatus::Waiting("http".into());
                Ok(())
            }
        };
    }
    let target: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    let Some(session) = take_upload_session(handle) else {
        let receipt = a2r_std::http::UploadReceipt::failed(
            a2r_std::http::UploadErrorKind::SessionConflict,
            format!("upload_commit: unknown or already-terminal session {handle}"),
        );
        let rid = insert_upload_receipt(receipt);
        task.ram.push_i32(rid as i32);
        return Ok(());
    };
    let op_id = spawn_upload_watcher(async move {
        let receipt = a2r_std::http::upload_commit(session, &target).await;
        UploadOpResult::Receipt(receipt)
    });
    task.waiting_http_request_id = Some(op_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `http.upload_reject(session, status, message) -> UploadReceipt`（park 等
/// 清理收口；failed 会话 status=0 → 建议状态）。
pub fn shim_http_upload_reject(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if let Some(op_id) = task.waiting_http_request_id {
        return match take_upload_op(op_id) {
            Some(UploadOpResult::Receipt(receipt)) => {
                task.waiting_http_request_id = None;
                let rid = insert_upload_receipt(receipt);
                task.ram.push_i32(rid as i32);
                Ok(())
            }
            Some(UploadOpResult::Session(_)) => {
                task.waiting_http_request_id = None;
                Err(VMError::RuntimeError(
                    "upload_reject: unexpected session result".into(),
                ))
            }
            None => {
                task.status = crate::vm::task::TaskStatus::Waiting("http".into());
                Ok(())
            }
        };
    }
    let message: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let status = crate::vm::native::pop_arg_i32(task) as i64;
    let handle = crate::vm::native::pop_arg_i32(task) as u64;
    let Some(session) = take_upload_session(handle) else {
        let receipt = a2r_std::http::UploadReceipt::failed(
            a2r_std::http::UploadErrorKind::SessionConflict,
            format!("upload_reject: unknown or already-terminal session {handle}"),
        );
        let rid = insert_upload_receipt(receipt);
        task.ram.push_i32(rid as i32);
        return Ok(());
    };
    let op_id = spawn_upload_watcher(async move {
        let receipt = a2r_std::http::upload_reject(session, status, &message).await;
        UploadOpResult::Receipt(receipt)
    });
    task.waiting_http_request_id = Some(op_id);
    task.status = crate::vm::task::TaskStatus::Waiting("http".into());
    Ok(())
}

/// `http.upload_error(status, message) -> UploadReceipt`（零 I/O 早拒构造）。
pub fn shim_http_upload_error(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let message: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let status = crate::vm::native::pop_arg_i32(task) as i64;
    let receipt = a2r_std::http::upload_error(status, &message);
    let rid = insert_upload_receipt(receipt);
    task.ram.push_i32(rid as i32);
    Ok(())
}

/// scope 组收口：取消在途接收 + 清理 staged 会话 + 释放未消费 body 能力
/// （finalize_scope 调用；幂等）。
pub(crate) fn scope_finalize_uploads(group: &[u64]) {
    for id in group {
        // 未消费的注入能力：drop body 流（producer 释放）。
        if let Ok(mut m) = VM_UPLOAD_REQUESTS.lock() {
            m.remove(id);
        }
        // 会话（staged/receiving）：executor 取消+清理；快照出表。
        if VM_UPLOAD_SESSIONS.lock().map(|m| m.contains_key(id)).unwrap_or(false) {
            a2r_std::http::cancel_upload_session(*id);
            if let Ok(mut m) = VM_UPLOAD_SESSIONS.lock() {
                m.remove(id);
            }
        }
        // 收据：闲置描述符出表（防无界增长）。
        if let Ok(mut m) = VM_UPLOAD_RECEIPTS.lock() {
            m.remove(id);
        }
    }
}

/// 存活登记总数（资源回基线探针）。
#[cfg(test)]
pub(crate) fn vm_upload_counts() -> (usize, usize, usize) {
    let r = VM_UPLOAD_REQUESTS.lock().map(|m| m.len()).unwrap_or(0);
    let s = VM_UPLOAD_SESSIONS.lock().map(|m| m.len()).unwrap_or(0);
    let c = VM_UPLOAD_RECEIPTS.lock().map(|m| m.len()).unwrap_or(0);
    (r, s, c)
}
