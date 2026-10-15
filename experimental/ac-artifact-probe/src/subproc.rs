//! Deadlined subprocess execution with concurrent pipe drain and Job-Object
//! containment (the PLAN-741 link.rs pattern, compact probe form).
//!
//! Every external process this probe spawns (the node helper, produced PEs,
//! the linker) runs under a deadline; a killed child never leaves the drain
//! threads waiting on inherited pipes.

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub struct RunResult {
    pub code: Option<i32>,
    pub timed_out: bool,
    pub stdout: String,
    pub stderr: String,
    pub elapsed_ms: u64,
}

#[cfg(windows)]
mod job {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    /// Job guard: descendants of the child die with it (kill-on-close), so
    /// a deadline kill cannot leave grandchildren holding our pipes.
    pub struct JobGuard(pub usize);

    impl JobGuard {
        pub fn for_child(child: &Child) -> Option<JobGuard> {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                let ok = SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                );
                if ok == 0 {
                    return None;
                }
                let ok = AssignProcessToJobObject(job, child.as_raw_handle() as _);
                if ok == 0 {
                    return None;
                }
                Some(JobGuard(job as usize))
            }
        }
    }

    impl Drop for JobGuard {
        fn drop(&mut self) {
            // Closing the last handle kills the job's remaining processes
            // (KILL_ON_JOB_CLOSE). We never terminate the job manually.
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.0 as _);
            }
        }
    }
}

#[cfg(not(windows))]
mod job {
    use std::process::Child;
    pub struct JobGuard;
    impl JobGuard {
        pub fn for_child(_child: &Child) -> Option<JobGuard> {
            None
        }
    }
}

/// Run a command under a wall-clock deadline with drained pipes.
pub fn run_deadlined(mut cmd: Command, timeout: Duration) -> std::io::Result<RunResult> {
    use std::sync::mpsc;
    let start = Instant::now();
    cmd.stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // Children spawned for capture must not inherit OUR stdout/stderr pipe
    // handles beyond the child itself — the job guard kills the tree on
    // timeout, and drain threads below always finish at pipe EOF.
    let mut child = cmd.spawn()?;
    let _guard = job::JobGuard::for_child(&child);

    let mut out_pipe = child.stdout.take().unwrap();
    let mut err_pipe = child.stderr.take().unwrap();
    let (tx_out, rx_out) = mpsc::channel::<String>();
    let (tx_err, rx_err) = mpsc::channel::<String>();
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out_pipe.read_to_string(&mut s);
        let _ = tx_out.send(s);
    });
    std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err_pipe.read_to_string(&mut s);
        let _ = tx_err.send(s);
    });

    let deadline = start + timeout;
    let mut timed_out = false;
    let code;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                code = status.code();
                break;
            }
            Ok(None) => {
                if Instant::now() >= deadline {
                    timed_out = true;
                    let _ = child.kill();
                    let _ = child.wait();
                    code = None;
                    break;
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            Err(e) => return Err(e),
        }
    }
    // Drain threads finish at EOF (job guard guarantees grandchildren die).
    let stdout = rx_out.recv_timeout(Duration::from_secs(10)).unwrap_or_default();
    let stderr = rx_err.recv_timeout(Duration::from_secs(10)).unwrap_or_default();
    Ok(RunResult {
        code,
        timed_out,
        stdout,
        stderr,
        elapsed_ms: start.elapsed().as_millis() as u64,
    })
}
