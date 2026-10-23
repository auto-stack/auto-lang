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
fn thread_ids() -> Vec<u32> {
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
        let mut ids = Vec::new();
        let mut more = Thread32First(h, &mut e);
        while more != 0 {
            if e.pid == pid {
                ids.push(e.tid);
            }
            more = Thread32Next(h, &mut e);
        }
        CloseHandle(h);
        ids
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 3 && args[1] == "--probe-host" {
        return host(&args[2]);
    }
    let mode = args[1].clone();
    let baseline = thread_ids();
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
    let spawn_ids = thread_ids();
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
    println!("shutdown={shutdown:?}");

    // F-816-T05-R2-03 判定（抗噪 oracle）：进程级线程快照含与负载相关
    // 的 OS 瞬态线程（基线期可缺席、spawn 后出现并长存——实测 SPAWN 集
    // 内非客户端 id 持续>1.2s），“前后计数相等”会假红。改为两条仍区分
    // 泄漏的不变量：①收尾后不得存在任何 **spawn 快照之后新建**且持续
    // 的线程（泵在 run 期创建——泄漏即被捕获；w/r/e 在 spawn 期创建
    // ——其退出由 shutdown 的 join 协议独立保证）；②spawn 时在册的线
    // 程至少 3 条（writer/reader/stderr）已消失（缺一即客户端线程未退）。
    let known: std::collections::HashSet<u32> =
        baseline.iter().chain(spawn_ids.iter()).copied().collect();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    let mut post = thread_ids();
    let mut fresh: Vec<u32>;
    loop {
        fresh = post
            .iter()
            .copied()
            .filter(|id| !known.contains(id))
            .collect();
        if fresh.is_empty() || std::time::Instant::now() >= deadline {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
        post = thread_ids();
    }
    let gone = known.len() - post.iter().filter(|id| known.contains(id)).count();
    if !fresh.is_empty() || gone < 3 {
        println!(
            "PROBE_VERDICT mode={mode} leak fresh={fresh:?} gone={gone} baseline={baseline:?} spawn={spawn_ids:?} post={post:?}"
        );
    } else {
        println!(
            "PROBE_VERDICT mode={mode} ok baseline={} spawn={} post={}",
            baseline.len(),
            spawn_ids.len(),
            post.len()
        );
    }
    if mode == "held" {
        // 父进程观察窗口：stdin 仍被持有（pump 必须已被有界回收）。
        std::thread::sleep(std::time::Duration::from_millis(300));
    }
}
