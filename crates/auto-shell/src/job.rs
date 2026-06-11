//! Job control for ASH shell.
//!
//! Provides background execution (`cmd &`), job listing (`jobs`),
//! foreground bring-back (`fg`), and background resume (`bg`).
//! Cross-platform: Windows uses SuspendThread/ResumeThread,
//! Unix uses SIGTSTP/SIGCONT.

use miette::{miette, Result};
use std::collections::HashMap;
use std::process::Child;

/// State of a background or stopped job.
#[derive(Debug, Clone, PartialEq)]
pub enum JobState {
    /// Job is running in the background.
    Running,
    /// Job was stopped (suspended) and is waiting to be resumed.
    Stopped,
    /// Job has finished. Will be cleaned up on next `jobs` call.
    Done,
}

/// A shell job — a child process tracked by the job manager.
pub struct Job {
    pub id: u32,
    pub command: String,
    pub child: Child,
    pub state: JobState,
}

/// Manages all background and stopped jobs.
pub struct JobManager {
    jobs: HashMap<u32, Job>,
    next_id: u32,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new background job. Returns its job ID.
    pub fn add(&mut self, command: String, child: Child) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.jobs.insert(
            id,
            Job {
                id,
                command,
                child,
                state: JobState::Running,
            },
        );
        id
    }

    /// Get a mutable reference to a job by ID.
    pub fn get_mut(&mut self, id: u32) -> Option<&mut Job> {
        self.jobs.get_mut(&id)
    }

    /// Remove a job by ID (returns it so the caller can wait on it).
    pub fn remove(&mut self, id: u32) -> Option<Job> {
        self.jobs.remove(&id)
    }

    /// Get the most recent job ID (highest number).
    pub fn last_job_id(&self) -> Option<u32> {
        self.jobs.keys().copied().max()
    }

    /// Poll all jobs and update their state. Remove finished jobs.
    /// Returns notifications for newly-finished jobs.
    pub fn reap_finished(&mut self) -> Vec<(u32, String, i32)> {
        let mut finished = Vec::new();
        let mut to_remove = Vec::new();

        for (&id, job) in &mut self.jobs {
            if job.state == JobState::Done {
                continue;
            }
            match job.child.try_wait() {
                Ok(Some(status)) => {
                    let code = status.code().unwrap_or(-1);
                    let cmd = job.command.clone();
                    finished.push((id, cmd, code));
                    to_remove.push(id);
                }
                Ok(None) => {
                    // Still running
                }
                Err(_) => {
                    to_remove.push(id);
                }
            }
        }

        for id in &to_remove {
            self.jobs.remove(id);
        }

        finished
    }

    /// Format the job list for display.
    pub fn format_jobs(&mut self) -> String {
        // Reap finished jobs first
        let finished = self.reap_finished();

        let mut output = String::new();

        // Report newly finished jobs
        for (id, cmd, _code) in &finished {
            output.push_str(&format!("[{}]  Done    {}\n", id, cmd));
        }

        // List remaining jobs
        let mut ids: Vec<u32> = self.jobs.keys().copied().collect();
        ids.sort();

        for id in &ids {
            if let Some(job) = self.jobs.get(id) {
                let state = match job.state {
                    JobState::Running => "Running",
                    JobState::Stopped => "Stopped",
                    JobState::Done => "Done",
                };
                output.push_str(&format!("[{}]  {:<8}{}\n", job.id, state, job.command));
            }
        }

        if output.is_empty() {
            output = "No active jobs.\n".to_string();
        }

        output
    }

    /// Suspend a running job (platform-specific).
    pub fn suspend_job(&mut self, id: u32) -> Result<()> {
        let job = self.jobs.get_mut(&id).ok_or_else(|| miette!("fg: job {} not found", id))?;
        if job.state != JobState::Running {
            miette::bail!("job {} is not running", id);
        }
        platform::suspend_child(&mut job.child)?;
        job.state = JobState::Stopped;
        Ok(())
    }

    /// Resume a stopped job (platform-specific).
    pub fn resume_job(&mut self, id: u32) -> Result<()> {
        let job = self.jobs.get_mut(&id).ok_or_else(|| miette!("bg: job {} not found", id))?;
        if job.state != JobState::Stopped {
            miette::bail!("job {} is not stopped", id);
        }
        platform::resume_child(&mut job.child)?;
        job.state = JobState::Running;
        Ok(())
    }

    /// Check if there are any active jobs.
    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    /// Raw read-only access to the jobs map (for iteration in builtins).
    pub fn jobs_raw(&self) -> &HashMap<u32, Job> {
        &self.jobs
    }
}

// ── Platform-specific suspend/resume ────────────────────────

mod platform {
    use miette::{miette, Result};
    use std::process::Child;

    /// Suspend all threads of a child process.
    #[cfg(windows)]
    pub fn suspend_child(child: &mut Child) -> Result<()> {
        use std::os::windows::io::AsRawHandle;
        let _handle = child.as_raw_handle();

        // Use Windows SuspendThread via raw FFI
        extern "system" {
            fn OpenThread(dwDesiredAccess: u32, bInheritHandle: i32, dwThreadId: u32) -> *mut std::ffi::c_void;
            fn SuspendThread(hThread: *mut std::ffi::c_void) -> u32;
            fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
        }

        const THREAD_SUSPEND_RESUME: u32 = 0x0002;

        // Get process ID and enumerate threads
        let pid = child.id();

        // Take a snapshot of all threads in the system
        extern "system" {
            fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> *mut std::ffi::c_void;
        }
        const TH32CS_SNAPTHREAD: u32 = 0x00000004;

        #[repr(C)]
        struct ThreadEntry {
            dwSize: u32,
            cntUsage: u32,
            th32ThreadID: u32,
            th32OwnerProcessID: u32,
            tpBasePri: i32,
            tpDeltaPri: i32,
            dwFlags: u32,
        }

        extern "system" {
            fn Thread32First(hSnapshot: *mut std::ffi::c_void, lpte: *mut ThreadEntry) -> i32;
            fn Thread32Next(hSnapshot: *mut std::ffi::c_void, lpte: *mut ThreadEntry) -> i32;
        }

        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snapshot.is_null() {
            miette::bail!("failed to create thread snapshot");
        }

        let mut entry = ThreadEntry {
            dwSize: std::mem::size_of::<ThreadEntry>() as u32,
            cntUsage: 0,
            th32ThreadID: 0,
            th32OwnerProcessID: 0,
            tpBasePri: 0,
            tpDeltaPri: 0,
            dwFlags: 0,
        };

        let mut suspended = 0u32;
        let ok = unsafe { Thread32First(snapshot, &mut entry) };
        if ok != 0 {
            loop {
                if entry.th32OwnerProcessID == pid && entry.th32ThreadID != 0 {
                    let thread = unsafe {
                        OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID)
                    };
                    if !thread.is_null() {
                        unsafe {
                            SuspendThread(thread);
                            CloseHandle(thread);
                        }
                        suspended += 1;
                    }
                }
                entry.dwSize = std::mem::size_of::<ThreadEntry>() as u32;
                if unsafe { Thread32Next(snapshot, &mut entry) } == 0 {
                    break;
                }
            }
        }

        unsafe { CloseHandle(snapshot) };

        if suspended == 0 {
            miette::bail!("no threads found for process {}", pid);
        }
        Ok(())
    }

    /// Resume all threads of a child process.
    #[cfg(windows)]
    pub fn resume_child(child: &mut Child) -> Result<()> {
        extern "system" {
            fn OpenThread(dwDesiredAccess: u32, bInheritHandle: i32, dwThreadId: u32) -> *mut std::ffi::c_void;
            fn ResumeThread(hThread: *mut std::ffi::c_void) -> u32;
            fn CloseHandle(hObject: *mut std::ffi::c_void) -> i32;
            fn CreateToolhelp32Snapshot(dwFlags: u32, th32ProcessID: u32) -> *mut std::ffi::c_void;
        }

        const THREAD_SUSPEND_RESUME: u32 = 0x0002;
        const TH32CS_SNAPTHREAD: u32 = 0x00000004;

        #[repr(C)]
        struct ThreadEntry {
            dwSize: u32,
            cntUsage: u32,
            th32ThreadID: u32,
            th32OwnerProcessID: u32,
            tpBasePri: i32,
            tpDeltaPri: i32,
            dwFlags: u32,
        }

        extern "system" {
            fn Thread32First(hSnapshot: *mut std::ffi::c_void, lpte: *mut ThreadEntry) -> i32;
            fn Thread32Next(hSnapshot: *mut std::ffi::c_void, lpte: *mut ThreadEntry) -> i32;
        }

        let pid = child.id();
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
        if snapshot.is_null() {
            miette::bail!("failed to create thread snapshot");
        }

        let mut entry = ThreadEntry {
            dwSize: std::mem::size_of::<ThreadEntry>() as u32,
            cntUsage: 0,
            th32ThreadID: 0,
            th32OwnerProcessID: 0,
            tpBasePri: 0,
            tpDeltaPri: 0,
            dwFlags: 0,
        };

        let mut _resumed = 0u32;
        let ok = unsafe { Thread32First(snapshot, &mut entry) };
        if ok != 0 {
            loop {
                if entry.th32OwnerProcessID == pid && entry.th32ThreadID != 0 {
                    let thread = unsafe {
                        OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID)
                    };
                    if !thread.is_null() {
                        unsafe {
                            ResumeThread(thread);
                            CloseHandle(thread);
                        }
                        _resumed += 1;
                    }
                }
                entry.dwSize = std::mem::size_of::<ThreadEntry>() as u32;
                if unsafe { Thread32Next(snapshot, &mut entry) } == 0 {
                    break;
                }
            }
        }

        unsafe { CloseHandle(snapshot) };
        Ok(())
    }

    /// Suspend a child process (Unix: send SIGSTOP).
    #[cfg(unix)]
    pub fn suspend_child(child: &mut Child) -> Result<()> {
        const SIGSTOP: i32 = 19;
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        let ret = unsafe { kill(child.id() as i32, SIGSTOP) };
        if ret != 0 {
            miette::bail!("failed to suspend process {}", child.id());
        }
        Ok(())
    }

    /// Resume a child process (Unix: send SIGCONT).
    #[cfg(unix)]
    pub fn resume_child(child: &mut Child) -> Result<()> {
        const SIGCONT: i32 = 18;
        extern "C" {
            fn kill(pid: i32, sig: i32) -> i32;
        }
        let ret = unsafe { kill(child.id() as i32, SIGCONT) };
        if ret != 0 {
            miette::bail!("failed to resume process {}", child.id());
        }
        Ok(())
    }
}
