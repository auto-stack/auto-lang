//! PLAN-816 T-05 R1（F-02/F-03）永久回归探针：一个二进制两种角色。
//! `--probe-host <mode>` = 具名协议宿主（input 模式：run_ui 后等真实
//! Input 帧，先回 InputAck 再回 RunOk——合法交错反例的正主）；
//! `<mode>` = 消费者（run_ui(request, forward_stdin)，观测线程数与
//! shutdown 终态，输出判定行）。测试进程持其 stdin 管道并按模式注入 q。
use std::io::{Read, Write};

fn reply(id: u64, body: serde_json::Value) {
    let v = serde_json::to_vec(&serde_json::json!({"id":id,"body":body})).unwrap();
    let mut out = std::io::stdout().lock();
    out.write_all(&(v.len() as u32).to_be_bytes()).unwrap();
    out.write_all(&v).unwrap();
    out.flush().unwrap();
}
fn run_ok(id: u64) {
    reply(
        id,
        serde_json::json!({"type":"run_ok","report":{"profile":"vue","outcome":"completed","duration_ms":0,"launch":null,"error":null}}),
    );
}
fn host(mode: &str) {
    let mut input = std::io::stdin().lock();
    loop {
        let mut h = [0; 4];
        if input.read_exact(&mut h).is_err() {
            break;
        }
        let mut bytes = vec![0u8; u32::from_be_bytes(h) as usize];
        input.read_exact(&mut bytes).unwrap();
        let v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let id = v["id"].as_u64().unwrap();
        match v["body"]["op"].as_str().unwrap() {
            "run_ui" if mode == "input" => {
                eprintln!("HOST_RUN_READY");
            }
            "run_ui" => run_ok(id),
            "input" => {
                let target = v["body"]["target_id"].as_u64().unwrap();
                reply(
                    id,
                    serde_json::json!({"type":"input_ack","target_id":target,"bytes":2}),
                );
                run_ok(target);
            }
            _ => panic!("unexpected body {v}"),
        }
    }
}

#[cfg(windows)]
fn threads() -> u32 {
    #[repr(C)]
    struct Entry {
        size: u32,
        usage: u32,
        tid: u32,
        pid: u32,
        base: i32,
        delta: i32,
        flags: u32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn CreateToolhelp32Snapshot(flags: u32, pid: u32) -> isize;
        fn Thread32First(h: isize, e: *mut Entry) -> i32;
        fn Thread32Next(h: isize, e: *mut Entry) -> i32;
        fn GetCurrentProcessId() -> u32;
        fn CloseHandle(h: isize) -> i32;
    }
    unsafe {
        let h = CreateToolhelp32Snapshot(4, 0);
        assert_ne!(h, -1);
        let mut e = Entry {
            size: std::mem::size_of::<Entry>() as u32,
            usage: 0,
            tid: 0,
            pid: 0,
            base: 0,
            delta: 0,
            flags: 0,
        };
        let pid = GetCurrentProcessId();
        let mut count = 0;
        let mut more = Thread32First(h, &mut e);
        while more != 0 {
            if e.pid == pid {
                count += 1;
            }
            more = Thread32Next(h, &mut e);
        }
        CloseHandle(h);
        count
    }
}
#[cfg(unix)]
fn threads() -> u32 {
    std::fs::read_dir("/proc/self/task")
        .map(|d| d.count() as u32)
        .unwrap_or(0)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--probe-host" {
        return host(&args[2]);
    }
    let mode = args[1].clone();
    let before = threads();
    let mut spec = auto_tool_client::ToolClientSpec::new(std::env::current_exe().unwrap());
    spec.argv = vec![
        std::ffi::OsString::from("--probe-host"),
        std::ffi::OsString::from(&mode),
    ];
    spec.request_timeout = Some(std::time::Duration::from_secs(3));
    spec.shutdown_timeout = std::time::Duration::from_millis(250);
    spec.live_log = Some(std::sync::Arc::new(|b: &[u8]| {
        eprint!("{}", String::from_utf8_lossy(b))
    }));
    let mut client = auto_tool_client::ToolClient::spawn(spec).unwrap();
    let root = std::env::current_dir().unwrap();
    let req = auto_project_model::GenerationRequest::from_descriptor(
        &auto_project_model::parse_document(
            "name: \"probe\"\nrender: \"vue\"\n",
            auto_project_model::descriptor::SourceInfo::from_path(root.join("pac.at")),
        )
        .unwrap(),
    );
    let result = client.run_ui(req, mode != "control");
    println!("run_result={result:?}");
    let receipts = client.take_receipts();
    println!("receipts={}", receipts.len());
    let shutdown = client.shutdown();
    std::thread::sleep(std::time::Duration::from_millis(100));
    println!(
        "shutdown={shutdown:?}; threads_before={before}; threads_after={}",
        threads()
    );
    std::io::stdout().flush().unwrap();
    if mode == "held" {
        // 父进程观察窗口：stdin 仍被持有（pump 必须已被有界回收）。
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}
