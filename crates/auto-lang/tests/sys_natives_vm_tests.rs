//! Plan 541: VM tests for auto.sys.* native functions

use auto_lang::run_autovm_capture;

#[test]
fn test_sys_natives_vm() {
    let code = r#"
fn main() {
    let cpu = sys.cpu_usage()
    assert(cpu >= 0.0)
    assert(cpu <= 100.0)

    let cores = sys.cpu_count()
    assert(cores > 0)

    let core0 = sys.cpu_core_usage(0)
    assert(core0 >= 0.0)
    assert(core0 <= 100.0)

    let brand = sys.cpu_brand()
    assert(brand.len() > 0)

    let total_mb = sys.mem_total_mb()
    assert(total_mb > 0)

    let used_mb = sys.mem_used_mb()
    assert(used_mb > 0)
    assert(used_mb <= total_mb)

    let net_sent = sys.net_sent_kbs()
    assert(net_sent >= 0.0)

    let net_recv = sys.net_recv_kbs()
    assert(net_recv >= 0.0)

    let os = sys.os_name()
    assert(os.len() > 0)

    let ver = sys.os_version()
    assert(ver.len() > 0)

    let kernel = sys.kernel_version()
    assert(kernel.len() > 0)

    let host = sys.hostname()
    assert(host.len() > 0)

    let up = sys.uptime_s()
    assert(up >= 0)

    let procs = sys.processes()
    assert(procs.len() > 0)
    let p0 = procs[0]
    assert(p0.pid >= 0)
    assert(p0.name.len() > 0)
    assert(p0.cpu >= 0.0)
    assert(p0.mem_mb >= 0)

    let dead_kill = sys.kill(0)
    assert(dead_kill == false)

    let disks = sys.disks()
    assert(disks.len() > 0)
    let d0 = disks[0]
    assert(d0.name.len() > 0 || d0.mount.len() > 0)
    assert(d0.total_mb >= 0)

    let users = sys.users()
    assert(users.len() >= 0)

    print("sys_natives_vm_ok")
}
"#;
    let (res, stdout) = run_autovm_capture(code).expect("sys_natives execution failed");
    assert!(stdout.contains("sys_natives_vm_ok") || res.contains("sys_natives_vm_ok"));
}

#[test]
fn test_sys_snapshot_struct_assembly() {
    let code = r#"
pub type SysSummary = {
    cpu: float
    cpu_count: int
    cpu_brand: str
    mem_used_mb: int
    mem_total_mb: int
    net_sent_kbs: float
    net_recv_kbs: float
    proc_count: int
    os_name: str
    os_version: str
    kernel: str
    hostname: str
    uptime_s: int
}

pub type ProcInfo = {
    pid: int
    name: str
    cpu: float
    mem_mb: int
    disk_kbs: float
    net_kbs: float
    status: str
    user: str
}

pub type DiskInfo = {
    name: str
    mount: str
    total_mb: int
    avail_mb: int
}

pub type Snapshot = {
    summary: SysSummary
    procs: []ProcInfo
    disks: []DiskInfo
    users: []str
    cores: []float
}

fn get_snapshot() Snapshot {
    let cpu = sys.cpu_usage()
    let count = sys.cpu_count()
    let brand = sys.cpu_brand()
    let mem_used = sys.mem_used_mb()
    let mem_total = sys.mem_total_mb()
    let net_sent = sys.net_sent_kbs()
    let net_recv = sys.net_recv_kbs()
    let os_n = sys.os_name()
    let os_v = sys.os_version()
    let kern = sys.kernel_version()
    let host = sys.hostname()
    let up = sys.uptime_s()

    var cores []float = []
    var ci = 0
    while ci < count {
        cores.push(sys.cpu_core_usage(ci))
        ci = ci + 1
    }

    var procs []ProcInfo = []
    let raw_procs = sys.processes()
    for p in raw_procs {
        procs.push(ProcInfo {
            pid: p.pid,
            name: p.name,
            cpu: p.cpu,
            mem_mb: p.mem_mb,
            disk_kbs: p.disk_kbs,
            net_kbs: p.net_kbs,
            status: p.status,
            user: p.user
        })
    }

    var disks []DiskInfo = []
    let raw_disks = sys.disks()
    for d in raw_disks {
        disks.push(DiskInfo {
            name: d.name,
            mount: d.mount,
            total_mb: d.total_mb,
            avail_mb: d.avail_mb
        })
    }

    var users []str = []
    let raw_users = sys.users()
    for u in raw_users {
        users.push(u.name)
    }

    let summary = SysSummary {
        cpu: cpu,
        cpu_count: count,
        cpu_brand: brand,
        mem_used_mb: mem_used,
        mem_total_mb: mem_total,
        net_sent_kbs: net_sent,
        net_recv_kbs: net_recv,
        proc_count: procs.len(),
        os_name: os_n,
        os_version: os_v,
        kernel: kern,
        hostname: host,
        uptime_s: up
    }

    return Snapshot {
        summary: summary,
        procs: procs,
        disks: disks,
        users: users,
        cores: cores
    }
}

fn main() {
    let snap = get_snapshot()
    print("DEBUG cpu_brand: " + snap.summary.cpu_brand)
    print("DEBUG procs len: " + snap.procs.len().str())
    print("DEBUG cores len: " + snap.cores.len().str())
    assert(snap.summary.cpu_count > 0)
    assert(snap.procs.len() > 0)
    assert(snap.cores.len() > 0)
    print("snapshot_assembly_ok")
}
"#;
    let (res, stdout) = run_autovm_capture(code).expect("snapshot assembly failed");
    println!("stdout: {}", stdout);
    println!("res: {}", res);
    assert!(stdout.contains("snapshot_assembly_ok") || res.contains("snapshot_assembly_ok"));
}

#[test]
fn test_nv_to_json_snapshot() {
    let code = r#"
pub type SysSummary = {
    cpu: float
    cpu_count: int
    cpu_brand: str
    mem_used_mb: int
    mem_total_mb: int
    net_sent_kbs: float
    net_recv_kbs: float
    proc_count: int
    os_name: str
    os_version: str
    kernel: str
    hostname: str
    uptime_s: int
}

pub type ProcInfo = {
    pid: int
    name: str
    cpu: float
    mem_mb: int
    disk_kbs: float
    net_kbs: float
    status: str
    user: str
}

pub type DiskInfo = {
    name: str
    mount: str
    total_mb: int
    avail_mb: int
}

pub type Snapshot = {
    summary: SysSummary
    procs: []ProcInfo
    disks: []DiskInfo
    users: []str
    cores: []float
}

pub fn system_snapshot() Snapshot {
    let cpu = sys.cpu_usage()
    let count = sys.cpu_count()
    let brand = sys.cpu_brand()
    let mem_used = sys.mem_used_mb()
    let mem_total = sys.mem_total_mb()
    let net_sent = sys.net_sent_kbs()
    let net_recv = sys.net_recv_kbs()
    let os_n = sys.os_name()
    let os_v = sys.os_version()
    let kern = sys.kernel_version()
    let host = sys.hostname()
    let up = sys.uptime_s()

    var cores []float = []
    var ci = 0
    while ci < count {
        cores.push(sys.cpu_core_usage(ci))
        ci = ci + 1
    }

    var procs []ProcInfo = []
    let raw_procs = sys.processes()
    for p in raw_procs {
        procs.push(ProcInfo {
            pid: p.pid
            name: p.name
            cpu: p.cpu
            mem_mb: p.mem_mb
            disk_kbs: p.disk_kbs
            net_kbs: p.net_kbs
            status: p.status
            user: p.user
        })
    }

    var disks []DiskInfo = []
    let raw_disks = sys.disks()
    for d in raw_disks {
        disks.push(DiskInfo {
            name: d.name
            mount: d.mount
            total_mb: d.total_mb
            avail_mb: d.avail_mb
        })
    }

    var users []str = []
    let raw_users = sys.users()
    for u in raw_users {
        users.push(u.name)
    }

    let summary = SysSummary {
        cpu: cpu
        cpu_count: count
        cpu_brand: brand
        mem_used_mb: mem_used
        mem_total_mb: mem_total
        net_sent_kbs: net_sent
        net_recv_kbs: net_recv
        proc_count: procs.len()
        os_name: os_n
        os_version: os_v
        kernel: kern
        hostname: host
        uptime_s: up
    }

    return Snapshot {
        summary: summary
        procs: procs
        disks: disks
        users: users
        cores: cores
    }
}
"#;
    let (vm, _, _, _) = auto_lang::create_vm_from_source(code).expect("compile failed");
    let task_id = vm.spawn_task(0, 65536);
    let json = if let Some(t_arc) = vm.tasks.get(&task_id) {
        let mut t = t_arc.blocking_lock();
        vm.call_fn_by_name(&mut t, "system_snapshot", 0).expect("call_fn_by_name failed");
        let nv = t.ram.pop_nv();
        auto_lang::vm::ffi::http_server::nv_to_json(&vm, nv, 0)
    } else {
        None
    };
    println!("Serialized JSON:\n{:?}", json);
    let json_str = json.expect("JSON was None");
    assert!(json_str.contains("cpu_brand"));
}
