//! Windows Job Object 树终止（零依赖：仅 kernel32 显式声明）。
//!
//! owned 树语义：子进程（及其后代，job 成员的子进程自动入 job）随
//! `TerminateJobObject` 一并终止；`KILL_ON_JOB_CLOSE` 保证本进程意外
//! 退出时树也被回收（镜像 ChildGuard 的 drop 兜底，粒度到全树）。
//!
//! 归属时序（F-816-R1-04）：树范围子进程以 `CREATE_SUSPENDED` 启动，
//! 在其任何用户代码运行（从而可能派生后代）**之前**完成建 Job、配置与
//! `AssignProcessToJobObject`，再经线程快照恢复主线程。先前后代逃出
//! Job 的窗口由此关闭；准备/入组/恢复失败的路径对仍挂起的根终止+收尸
//! （挂起进程不可能已有后代）。

use std::ffi::c_void;
use std::io;
use std::os::windows::io::AsRawHandle;
use std::process::Child;

type Handle = *mut c_void;

const JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS: u32 = 9;
const JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE: u32 = 0x0000_2000;

const CREATE_SUSPENDED: u32 = 0x0000_0004;
const TH32CS_SNAPTHREAD: u32 = 0x0000_0004;
const THREAD_SUSPEND_RESUME: u32 = 0x0002;

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

#[repr(C)]
#[allow(non_snake_case)]
struct THREADENTRY32 {
    dwSize: u32,
    cntUsage: u32,
    th32ThreadID: u32,
    th32OwnerProcessID: u32,
    tpBasePri: i32,
    tpDeltaPri: i32,
    dwFlags: u32,
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
    fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> Handle;
    fn Thread32First(hSnapshot: Handle, lpte: *mut THREADENTRY32) -> i32;
    fn Thread32Next(hSnapshot: Handle, lpte: *mut THREADENTRY32) -> i32;
    fn OpenThread(dwDesiredAccess: u32, bInheritHandle: i32, dwThreadId: u32) -> Handle;
    fn ResumeThread(hThread: Handle) -> u32;
}

pub(crate) struct JobHandle(Handle);

impl JobHandle {
    /// 终止 job 内全部进程（树）；退出码统一为 1（调用方以上下文分类，
    /// 不读 code）。
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

/// 树范围启动所需的 `creation_flags`（挂起启动，见模块文档）。
pub(crate) const fn suspended_creation_flags() -> u32 {
    CREATE_SUSPENDED
}

/// 对**挂起中**的根进程建立 owned Job 归属并恢复执行。
///
/// 顺序合同：`AssignProcessToJobObject` 成功之前根进程不运行任何用户
/// 代码——后代只可能在归属建立后创建，从而必在 Job 内。任一步失败时根
/// 仍挂起（不可能已有后代），关闭 Job 句柄后返回错误；终止+收尸由调用
/// 方（lib.rs spawn 错误路径）完成。
pub(crate) fn assign_suspended_and_resume(child: &Child) -> io::Result<JobHandle> {
    unsafe {
        let job = CreateJobObjectW(std::ptr::null_mut(), std::ptr::null());
        if job.is_null() {
            return Err(io::Error::last_os_error());
        }
        let fail = |e: io::Error| -> io::Error {
            CloseHandle(job);
            e
        };
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if SetInformationJobObject(
            job,
            JOB_OBJECT_EXTENDED_LIMIT_INFORMATION_CLASS,
            &mut info as *mut _ as *mut c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        ) == 0
        {
            return Err(fail(io::Error::last_os_error()));
        }
        if AssignProcessToJobObject(job, child.as_raw_handle()) == 0 {
            return Err(fail(io::Error::last_os_error()));
        }
        if let Err(e) = resume_all_threads(child.id()) {
            return Err(fail(e));
        }
        Ok(JobHandle(job))
    }
}

/// 恢复 `pid` 的全部线程。`CREATE_SUSPENDED` 下启动瞬间只有主线程，
/// 快照枚举防御性地覆盖（已挂起进程不会新增线程）。
unsafe fn resume_all_threads(pid: u32) -> io::Result<()> {
    let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
    if snapshot.is_null() {
        return Err(io::Error::last_os_error());
    }
    let mut entry: THREADENTRY32 = std::mem::zeroed();
    entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
    let mut result = Ok(());
    if Thread32First(snapshot, &mut entry) != 0 {
        loop {
            if entry.th32OwnerProcessID == pid {
                let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                if thread.is_null() {
                    result = Err(io::Error::last_os_error());
                    break;
                }
                // 0xFFFFFFFF = 失败；否则为先前的挂起计数（≥1 表示已恢复）。
                if ResumeThread(thread) == u32::MAX {
                    result = Err(io::Error::last_os_error());
                    CloseHandle(thread);
                    break;
                }
                CloseHandle(thread);
            }
            if Thread32Next(snapshot, &mut entry) == 0 {
                break;
            }
        }
    } else {
        result = Err(io::Error::last_os_error());
    }
    CloseHandle(snapshot);
    result
}
