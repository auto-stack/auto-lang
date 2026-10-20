//! Windows stdin 写端取消（F-816-R5：写端真停）。
//!
//! 实测结论（本仓探针，2026-10-21）：
//! - 同步匿名管道句柄对 `WaitForSingleObject` 恒 signaled → 不可轮询；
//! - `CancelSynchronousIo(thread)` 对管道写返回 ERROR_NOT_FOUND → 无效；
//! - **`CancelIoEx(handle, NULL)` 从任意线程取消该文件对象上挂起的
//!   同步 `WriteFile`（阻塞写 ~10ms 内以错误解除）** —— 本模块的全部
//!   职责就是把这一原语封装为可保存、可从停止侧调用的句柄。

use std::ffi::c_void;
use std::io;
use std::os::windows::io::{AsRawHandle, RawHandle};
use std::process::ChildStdin;

type Handle = *mut c_void;

const DUPLICATE_SAME_ACCESS: u32 = 0x0000_0002;

extern "system" {
    fn DuplicateHandle(
        hSourceProcessHandle: Handle,
        hSourceHandle: Handle,
        hTargetProcessHandle: Handle,
        lpTargetHandle: *mut Handle,
        dwDesiredAccess: u32,
        bInheritHandle: i32,
        dwOptions: u32,
    ) -> i32;
    fn GetCurrentProcess() -> Handle;
    fn CancelIoEx(hFile: Handle, lpOverlapped: *mut c_void) -> i32;
    fn CloseHandle(hObject: Handle) -> i32;
}

/// stdin 写端的取消句柄：`cancel()` 取消写线程在管道上挂起的同步写
/// （DuplicateHandle 与原句柄同指一个文件对象——取消按对象生效）。
pub struct CancelHandle(Handle);

// 原始句柄按值拥有;保存于 OwnedProcess/client,由停止侧调用。
unsafe impl Send for CancelHandle {}
unsafe impl Sync for CancelHandle {}

impl CancelHandle {
    /// 复制 `stdin` 句柄为取消凭证。
    pub(crate) fn from_stdin(stdin: &ChildStdin) -> io::Result<Self> {
        unsafe {
            let mut dup: Handle = std::ptr::null_mut();
            let this = GetCurrentProcess();
            if DuplicateHandle(
                this,
                stdin.as_raw_handle() as Handle,
                this,
                &mut dup,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(CancelHandle(dup))
        }
    }

    /// 取消挂起的同步写；目标不在写中时无害（返回 false）。
    pub fn cancel(&self) -> bool {
        unsafe { CancelIoEx(self.0, std::ptr::null_mut()) != 0 }
    }
}

impl Clone for CancelHandle {
    fn clone(&self) -> Self {
        unsafe {
            let mut dup: Handle = std::ptr::null_mut();
            let this = GetCurrentProcess();
            if DuplicateHandle(this, self.0, this, &mut dup, 0, 0, DUPLICATE_SAME_ACCESS) == 0 {
                // 复制失败:空句柄(取消调用将无操作)。
                return CancelHandle(std::ptr::null_mut());
            }
            CancelHandle(dup)
        }
    }
}

impl Drop for CancelHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// 供 RawHandle 语义对齐的小工具（内部使用）。
#[allow(dead_code)]
pub(crate) fn as_raw(h: &CancelHandle) -> RawHandle {
    h.0 as RawHandle
}
