//! PLAN-729 T-04：VM 文件响应桥——`http.file_response` 的描述符登记面。
//!
//! 桥接形状（决策报告 §3；模板 = PLAN-727 http_transfer.rs）：
//! - **注册表**：`VM_FILE_RESPONSES`（描述符 id → owned 描述符）。native
//!   构造**零 I/O**（描述符只存配置+构造期纯校验）；编组单次取出
//!   （`take_file_response`），handler 未返回的闲置描述符由 scope 组收口
//!   移除（防注册表无界增长）。
//! - **返回类型识别**（决策报告 §6 的"普通 int 反例"防线）：编组取描述符
//!   需要三重命中——handler **声明返回类型**含 FileResponse
//!   （`fn_is_api_file_return`，codegen 的 `API_RETURN_TYPES` 侧信道）+
//!   值是 i32 且**登记命中**。普通 int 即使数值撞上描述符 id 也走 JSON 兜底；
//!   声明文件返回但值非登记句柄 → 500 诊断（不 JSON 200）。
//! - **执行**：本模块不碰网络/磁盘——transport 侧
//!   `crate::http_file_service::serve_file_response` 完成打开/协议决策/发送
//!   （AC-04：owner 不执行 I/O）。

use std::collections::HashMap;

use crate::vm::engine::{AutoVM, VMError};
use crate::vm::task::AutoTask;

use a2r_std::http::FileResponse;

lazy_static::lazy_static! {
    /// VM 文件响应描述符注册表：id（全局单调，描述符构造器分配）→ 描述符。
    /// 生命周期：insert 于 native 构造；remove 于编组取出 / scope 组收口。
    pub(crate) static ref VM_FILE_RESPONSES: std::sync::Mutex<HashMap<u64, FileResponse>> =
        std::sync::Mutex::new(HashMap::new());
}

/// 登记并返回描述符 id。
pub(crate) fn insert_file_response(descriptor: FileResponse) -> u64 {
    let id = descriptor.id();
    if let Ok(mut map) = VM_FILE_RESPONSES.lock() {
        map.insert(id, descriptor);
    }
    id
}

/// 编组取出（单次交付；未登记 id → None）。
pub(crate) fn take_file_response(id: u64) -> Option<FileResponse> {
    VM_FILE_RESPONSES
        .lock()
        .ok()
        .and_then(|mut m| m.remove(&id))
}

/// `http.file_response(root, relative_path, options) -> FileResponse`
/// （非阻塞构造；严格 options/词法路径校验在描述符内，adapter 映射）。
pub fn shim_http_file_response(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    use crate::vm::ffi::convert::VMConvertible;
    let options: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let relative_path: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let root: String = VMConvertible::pop_from_stack(task, vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let descriptor = a2r_std::http::file_response(&root, &relative_path, &options);
    super::http_server::register_scope_file_response(descriptor.id());
    let id = insert_file_response(descriptor);
    task.ram.push_i32(id as i32);
    Ok(())
}

/// scope 组收口：移除组内闲置描述符（幂等；编组已取出的 no-op）。
pub(crate) fn scope_finalize_file_responses(group: &[u64]) {
    if group.is_empty() {
        return;
    }
    if let Ok(mut map) = VM_FILE_RESPONSES.lock() {
        for id in group {
            map.remove(id);
        }
    }
}

/// 存活描述符数（资源回基线探针）。
#[cfg(test)]
pub(crate) fn vm_file_response_count() -> usize {
    VM_FILE_RESPONSES.lock().map(|m| m.len()).unwrap_or(0)
}
