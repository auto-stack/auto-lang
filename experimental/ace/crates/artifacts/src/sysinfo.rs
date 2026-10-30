//! OS-level process/system memory measurement (PLAN-783 §5.2: real
//! private-bytes evidence, not custom counters only).

#[cfg(windows)]
pub mod imp {
    use windows_sys::Win32::System::ProcessStatus::{
        GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows_sys::Win32::System::SystemInformation::GlobalMemoryStatusEx;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetCurrentProcessId};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ProcessMemory {
        /// Commit charge attributable to this process
        /// (`PROCESS_MEMORY_COUNTERS_EX::PrivateUsage`).
        pub private_bytes: u64,
        pub working_set: u64,
        pub peak_working_set: u64,
        pub page_faults: u64,
    }

    pub fn current_process_memory() -> ProcessMemory {
        unsafe {
            let mut counters: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
            counters.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
            let ok = GetProcessMemoryInfo(
                GetCurrentProcess(),
                &mut counters as *mut _ as *mut _,
                counters.cb,
            );
            assert_ne!(ok, 0, "GetProcessMemoryInfo failed");
            ProcessMemory {
                private_bytes: counters.PrivateUsage as u64,
                working_set: counters.WorkingSetSize as u64,
                peak_working_set: counters.PeakWorkingSetSize as u64,
                page_faults: counters.PageFaultCount as u64,
            }
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct SystemMemory {
        pub total_phys: u64,
        pub avail_phys: u64,
    }

    pub fn system_memory() -> SystemMemory {
        unsafe {
            let mut st: windows_sys::Win32::System::SystemInformation::MEMORYSTATUSEX =
                std::mem::zeroed();
            st.dwLength = std::mem::size_of::<
                windows_sys::Win32::System::SystemInformation::MEMORYSTATUSEX,
            >() as u32;
            let ok = GlobalMemoryStatusEx(&mut st);
            assert_ne!(ok, 0, "GlobalMemoryStatusEx failed");
            SystemMemory {
                total_phys: st.ullTotalPhys,
                avail_phys: st.ullAvailPhys,
            }
        }
    }

    /// Current process id (lease/lock receipts).
    pub fn current_pid() -> u32 {
        unsafe { GetCurrentProcessId() }
    }
}

#[cfg(not(windows))]
pub mod imp {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ProcessMemory {
        pub private_bytes: u64,
        pub working_set: u64,
        pub peak_working_set: u64,
        pub page_faults: u64,
    }

    pub fn current_process_memory() -> ProcessMemory {
        // Non-Windows hosts are outside this probe's native scope; the
        // numbers stay zero and layout tests that need real evidence are
        // Windows-gated.
        ProcessMemory {
            private_bytes: 0,
            working_set: 0,
            peak_working_set: 0,
            page_faults: 0,
        }
    }

    #[derive(Clone, Copy, Debug)]
    pub struct SystemMemory {
        pub total_phys: u64,
        pub avail_phys: u64,
    }

    pub fn system_memory() -> SystemMemory {
        SystemMemory {
            total_phys: 0,
            avail_phys: 0,
        }
    }

    pub fn current_pid() -> u32 {
        std::process::id()
    }
}

pub use imp::*;
