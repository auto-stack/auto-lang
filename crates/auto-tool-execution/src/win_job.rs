//! Windows Job Object 树终止(零依赖:仅 kernel32 显式声明)。
//!
//! owned 树语义:子进程(及其后代,job 成员的子进程自动入 job)随
//! `TerminateJobObject` 一并终止;`KILL_ON_JOB_CLOSE` 保证本进程意外
//! 退出时树也被回收(镜像 ChildGuard 的 drop 兜底,粒度到全树)。

use std::ffi::c_void;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::process::Child;

type Handle = *mut c_void;

const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: u32 = 9;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;

#[repr(C)]
#[allow(non_snake_case)]
struct JOBOBJECT_BASIC_LIMIT_INFORMATION {
    PerProcessUserTimeLimit: i64,
    PerJobUserTimeLimit: i64,
    LimitFlags: u32,
    MinimumWorkingSetSize: usize,
    MaximumWorkingSetSize: usize,
    ActiveProcessLimit: u32,
    Affinity: usize,
    PriorityClass: u32,
    SchedulingClass: u32,
}

#[repr(C)]
#[allow(non_snake_case)]
struct IO_COUNTERS {
    ReadOperationCount: u64,
    WriteOperationCount: u64,
    OtherOperationCount: u64,
    ReadTransferCount: u64,
    WriteTransferCount: u64,
    OtherTransferCount: u64,
}

#[repr(C)]
#[allow(non_snake_case)]
struct JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
    BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION,
    IoInfo: IO_COUNTERS,
    ProcessMemoryLimit: usize,
    JobMemoryLimit: usize,
    PeakProcessMemoryUsed: usize,
    PeakJobMemoryUsed: usize,
}

extern "system" {
    fn CreateJobObjectW(lpJobAttributes: *mut c_void, lpName: *const u16) -> Handle;
    fn SetInformationJobObject(
        hJob: Handle,
        JobObjectInformationClass: u32,
        lpJobObjectInformation: *mut c_void,
        cbJobObjectInformationLength: u32,
    ) -> i32;
    fn AssignProcessToJobObject(hJob: Handle, hProcess: Handle) -> i32;
    fn TerminateJobObject(hJob: Handle, uExitCode: u32) -> i32;
    fn CloseHandle(hObject: Handle) -> i32;
}

pub(crate) struct JobHandle(Handle);

impl JobHandle {
    /// 终止 job 内全部进程(树);退出码统一为 1(调用方以上下文分类,
    /// 不读 code)。
    pub(crate) fn terminate(&self) -> io::Result<()> {
        let r = unsafe { TerminateJobObject(self.0, 1) };
        if r == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

impl Drop for JobHandle {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { CloseHandle(self.0) };
        }
    }
}

/// 创建 KILL_ON_JOB_CLOSE 的 job 并把 `child` 加入。
pub(crate) fn create_and_assign(child: &Child) -> io::Result<JobHandle> {
    unsafe {
        let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            job,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            let e = io::Error::last_os_error();
            CloseHandle(job);
            return Err(e);
        }
        if AssignProcessToJobObject(job, child.as_raw_handle()) == 0 {
            let e = io::Error::last_os_error();
            CloseHandle(job);
            return Err(e);
        }
        Ok(JobHandle(job))
    }
}
