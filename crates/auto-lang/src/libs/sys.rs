//! System operation functions for AutoLang
//!
//! Provides built-in functions for system-level operations and sysinfo-backed metrics (Plan 541).

use auto_val::{Args, Value};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::time::Instant;
use sysinfo::{Disks, Networks, Pid, System, Users};

/// Process summary structure for sys.processes()
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProcData {
    pub pid: i32,
    pub name: String,
    pub cpu: f64,
    pub mem_mb: i32,
    pub disk_kbs: f64,
    pub net_kbs: f64,
    pub status: String,
    pub user: String,
}

/// Disk summary structure for sys.disks()
#[derive(Debug, Clone, serde::Serialize)]
pub struct DiskData {
    pub name: String,
    pub mount: String,
    pub total_mb: i32,
    pub avail_mb: i32,
}

/// User summary structure for sys.users()
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserData {
    pub name: String,
}

pub struct SystemSampler {
    pub sys: System,
    pub networks: Networks,
    pub disks: Disks,
    pub users: Users,
    pub last_net_sample: Instant,
    pub last_disk_sample: Instant,
    pub last_proc_sample: Instant,
    pub last_cpu_sample: Instant,
}

impl SystemSampler {
    fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        let networks = Networks::new_with_refreshed_list();
        let disks = Disks::new_with_refreshed_list();
        let users = Users::new_with_refreshed_list();
        let now = Instant::now();
        Self {
            sys,
            networks,
            disks,
            users,
            last_net_sample: now,
            last_disk_sample: now,
            last_proc_sample: now,
            last_cpu_sample: now,
        }
    }

    fn refresh_cpu_if_needed(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_cpu_sample).as_millis() >= 200 {
            self.sys.refresh_cpu_all();
            self.last_cpu_sample = now;
        }
    }

    fn refresh_networks(&mut self) -> (f64, f64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_net_sample).as_secs_f64();
        let elapsed_sec = if elapsed < 0.05 { 0.05 } else { elapsed };
        self.networks.refresh(true);
        self.last_net_sample = now;

        let mut total_recv_bytes: u64 = 0;
        let mut total_sent_bytes: u64 = 0;
        for (_, data) in &self.networks {
            total_recv_bytes += data.received();
            total_sent_bytes += data.transmitted();
        }
        let sent_kbs = (total_sent_bytes as f64 / 1024.0) / elapsed_sec;
        let recv_kbs = (total_recv_bytes as f64 / 1024.0) / elapsed_sec;
        (sent_kbs, recv_kbs)
    }

    fn refresh_memory(&mut self) {
        self.sys.refresh_memory();
    }

    fn refresh_disks(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_disk_sample).as_secs() >= 2 {
            self.disks.refresh(true);
            self.last_disk_sample = now;
        }
    }

    fn refresh_users(&mut self) {
        self.users.refresh();
    }
}

lazy_static! {
    static ref SAMPLER: Mutex<SystemSampler> = Mutex::new(SystemSampler::new());
}

/// Get process ID
pub fn sys_getpid(_args: &Args) -> Value {
    use std::process;
    Value::Int(process::id() as i32)
}

/// Global CPU usage (0.0 - 100.0)
pub fn cpu_usage() -> f64 {
    let mut s = SAMPLER.lock().unwrap();
    s.refresh_cpu_if_needed();
    ((s.sys.global_cpu_usage() as f64) * 10.0).round() / 10.0
}

/// Number of logical CPU cores
pub fn cpu_count() -> i32 {
    let s = SAMPLER.lock().unwrap();
    s.sys.cpus().len() as i32
}

/// Single CPU core usage (0.0 - 100.0)
pub fn cpu_core_usage(core_idx: usize) -> f64 {
    let mut s = SAMPLER.lock().unwrap();
    s.refresh_cpu_if_needed();
    s.sys
        .cpus()
        .get(core_idx)
        .map(|c| ((c.cpu_usage() as f64) * 10.0).round() / 10.0)
        .unwrap_or(0.0)
}

/// CPU brand string (e.g. "AMD Ryzen ...")
pub fn cpu_brand() -> String {
    let s = SAMPLER.lock().unwrap();
    s.sys
        .cpus()
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_default()
}

/// Total system memory in MB
pub fn mem_total_mb() -> i32 {
    let mut s = SAMPLER.lock().unwrap();
    s.refresh_memory();
    (s.sys.total_memory() / (1024 * 1024)) as i32
}

/// Used system memory in MB
pub fn mem_used_mb() -> i32 {
    let mut s = SAMPLER.lock().unwrap();
    s.refresh_memory();
    (s.sys.used_memory() / (1024 * 1024)) as i32
}

/// Sent network throughput (KB/s)
pub fn net_sent_kbs() -> f64 {
    let mut s = SAMPLER.lock().unwrap();
    let (sent, _) = s.refresh_networks();
    (sent * 10.0).round() / 10.0
}

/// Received network throughput (KB/s)
pub fn net_recv_kbs() -> f64 {
    let mut s = SAMPLER.lock().unwrap();
    let (_, recv) = s.refresh_networks();
    (recv * 10.0).round() / 10.0
}

/// OS name
pub fn os_name() -> String {
    System::name().unwrap_or_else(|| "Unknown".to_string())
}

/// OS version
pub fn os_version() -> String {
    System::os_version().unwrap_or_else(|| "Unknown".to_string())
}

/// Kernel version
pub fn kernel_version() -> String {
    System::kernel_version().unwrap_or_else(|| "Unknown".to_string())
}

/// Hostname
pub fn hostname() -> String {
    System::host_name().unwrap_or_else(|| "localhost".to_string())
}

/// System uptime in seconds
pub fn uptime_s() -> i32 {
    System::uptime() as i32
}

/// Disks list
pub fn disks() -> Vec<DiskData> {
    let mut s = SAMPLER.lock().unwrap();
    s.refresh_disks();
    s.disks
        .iter()
        .map(|d| DiskData {
            name: d.name().to_string_lossy().into_owned(),
            mount: d.mount_point().to_string_lossy().into_owned(),
            total_mb: (d.total_space() / (1024 * 1024)) as i32,
            avail_mb: (d.available_space() / (1024 * 1024)) as i32,
        })
        .collect()
}

/// Users list
pub fn users() -> Vec<UserData> {
    let mut s = SAMPLER.lock().unwrap();
    s.refresh_users();
    s.users
        .iter()
        .map(|u| UserData {
            name: u.name().to_string(),
        })
        .collect()
}

/// Process list: sorted by CPU descending, capped at 512 entries
pub fn processes() -> Vec<ProcData> {
    let mut s = SAMPLER.lock().unwrap();
    let now = Instant::now();
    let elapsed = now.duration_since(s.last_proc_sample).as_secs_f64();
    let elapsed_sec = if elapsed < 0.05 { 0.05 } else { elapsed };
    s.sys
        .refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    s.last_proc_sample = now;

    let cpu_cores = s.sys.cpus().len().max(1) as f64;
    let mut list = Vec::new();

    for (&pid, proc_) in s.sys.processes() {
        let norm_cpu = (proc_.cpu_usage() as f64 / cpu_cores).clamp(0.0, 100.0);
        let mem_mb = (proc_.memory() / (1024 * 1024)) as i32;

        let disk_usage = proc_.disk_usage();
        let disk_bytes = disk_usage.read_bytes + disk_usage.written_bytes;
        let disk_kbs = (disk_bytes as f64 / 1024.0) / elapsed_sec;

        let status_str = match proc_.status() {
            sysinfo::ProcessStatus::Run => "running",
            sysinfo::ProcessStatus::Sleep => "sleeping",
            sysinfo::ProcessStatus::Stop => "stopped",
            sysinfo::ProcessStatus::Zombie => "zombie",
            sysinfo::ProcessStatus::Idle => "idle",
            _ => "sleeping",
        };

        let user_str = proc_.user_id().map(|u| u.to_string()).unwrap_or_default();

        list.push(ProcData {
            pid: pid.as_u32() as i32,
            name: proc_.name().to_string_lossy().into_owned(),
            cpu: (norm_cpu * 10.0).round() / 10.0,
            mem_mb,
            disk_kbs: (disk_kbs * 10.0).round() / 10.0,
            net_kbs: 0.0,
            status: status_str.to_string(),
            user: user_str,
        });
    }

    // Sort by CPU descending
    list.sort_by(|a, b| {
        b.cpu
            .partial_cmp(&a.cpu)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if list.len() > 512 {
        list.truncate(512);
    }
    list
}

/// Kill process by PID
pub fn kill(pid: i32) -> bool {
    let mut s = SAMPLER.lock().unwrap();
    s.sys.refresh_processes(
        sysinfo::ProcessesToUpdate::Some(&[Pid::from_u32(pid as u32)]),
        true,
    );
    if let Some(proc_) = s.sys.process(Pid::from_u32(pid as u32)) {
        proc_.kill()
    } else {
        false
    }
}
