use auto_lang::vm::{engine::AutoVM, task::AutoTask, virt_memory::VirtualFlash};
use auto_lang::vm::ffi::{http_return_resource as resource, stdlib};
use std::sync::atomic::Ordering;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};

fn vm() -> AutoVM { AutoVM::new(VirtualFlash::new(1024), 1024) }
fn task() -> AutoTask { AutoTask::new(0, 1024, 0) }
fn units() {
    let a = vm(); let b = vm(); let mut ta = task(); let mut tb = task();
    resource::push_response_resource(&mut ta, &a, 201, vec![], b"owner-a".to_vec()).unwrap();
    resource::push_response_resource(&mut tb, &b, 202, vec![], b"owner-b".to_vec()).unwrap();
    let na = ta.ram.pop_nv(); let nb = tb.ram.pop_nv();
    println!("OWNER same_encoded_handle={} foreign_resolves={:?}", na == nb, resource::resolve_resource(&b, na));
    let c = vm(); let mut tc = task();
    c.heap_object_id_gen.store(u32::MAX as u64 + 1, Ordering::Relaxed);
    let result = resource::push_response_resource(&mut tc, &c, 203, vec![], vec![]);
    let nc = tc.ram.pop_nv();
    println!("EXHAUST at_2pow32 result={result:?} resolves={:?} heap_len={}", resource::resolve_resource(&c, nc), c.heap_objects.len());
    let before = resource::live_resource_count();
    let obj = a.get_heap_object(4_000_000).unwrap();
    let clone = obj.read().unwrap().as_any().downcast_ref::<resource::HttpReturnResource>().unwrap().clone();
    drop(clone);
    println!("CLONE before={before} after={}", resource::live_resource_count());
    let mut access = task();
    resource::push_response_resource(&mut access, &a, 307, vec![], b"body".to_vec()).unwrap();
    stdlib::shim_response_status_code(&mut access, &a).unwrap();
    println!("ACCESS expected=307 actual={}", access.ram.pop_i32());
    let old = stdlib::insert_http_response(307, vec![], vec![]);
    let mut control = task(); control.ram.push_i32(old as i32);
    stdlib::shim_response_status_code(&mut control, &a).unwrap();
    println!("ACCESS legacy_control={}", control.ram.pop_i32());
    let created = stdlib::shim_http_created("created".into());
    println!("PRODUCER created_id={created} still_in_legacy_table={}", stdlib::lookup_http_response(created as u64).is_some());
    let d = vm(); let mut td = task();
    resource::push_response_resource(&mut td, &d, 302, vec![], vec![]).unwrap();
    let resource_nv = td.ram.pop_nv();
    let actual_id = d.heap_object_id_gen.load(Ordering::Relaxed) - 1;
    // The remaining stake represents a retained global, not the plain-number slot.
    d.globals.insert("cached".into(), resource_nv);
    let mut numeric_slot = task(); numeric_slot.ram.push_i32(actual_id as i32);
    let plain_nv = numeric_slot.ram.pop_nv();
    let rc_before = d.rc_count(actual_id);
    d.rc_release(plain_nv); // New unconditional dispatch cleanup in PLAN-776.
    let rc_after = d.rc_count(actual_id); d.reap_dying(100_000);
    println!("PLAIN_RELEASE actual_id={actual_id} rc_before={rc_before} rc_after={rc_after} retained_global_resource_present={}", d.heap_objects.contains_key(&actual_id));
    for (name, code) in [
        ("empty", "type Response {}\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Response { return Response {} }"),
        ("alias", "type Response = int;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Response { return 9 }"),
        ("qualified", "#[api(method = \"GET\", path = \"/p\")]\nfn p() http.FileResponse { return 9 }"),
    ] {
        let mut p = auto_lang::parser::Parser::from(code);
        match p.parse() {
            Ok(ast) => {
                let m = auto_lang::api::ApiExtractor::new().extract("probe", &ast.stmts);
                println!("SOURCE {name} types={:?} endpoints={:?}", m.types.iter().map(|t| &t.name).collect::<Vec<_>>(), m.endpoints.iter().map(|e| (&e.return_type, e.response)).collect::<Vec<_>>());
            },
            Err(e) => println!("SOURCE {name} parse_error={e:?}"),
        }
    }
}

fn wire(which: &str) {
    let socket = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = socket.local_addr().unwrap().port(); drop(socket);
    std::env::set_var("AUTO_HTTP_PORT", port.to_string());
    let (tx, rx) = std::sync::mpsc::channel();
    let which = which.to_owned();
    std::thread::Builder::new().stack_size(8*1024*1024).spawn(move || {
        let code: String = match which.as_str() {
            "legacy" | "legacy-async" => {
                if which == "legacy" {
                    "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let live = http.response()\nreturn 1 }".into()
                } else {
                    "#[api(method = \"GET\", path = \"/p\")]\nfn p() ~int { delay_async(10)\nlet live = http.response()\nreturn 1 }".into()
                }
            },
            "iterator" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let live = stream()\nreturn 0 }\nfn stream() ~Iter<int> { yield 42 }".into(),
            "local" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let local = http.response_redirect(\"/x\", 302)\nreturn 9 }\n#[api(method = \"GET\", path = \"/drain\")]\nfn drain() int { var i = 0\nwhile i < 9000 { i = i + 1 }\nreturn i }".into(),
            "cancel-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() ~int { let held = http.response_redirect(\"/x\", 302)\ndelay_async(300).await\nreturn held }\n#[api(method = \"GET\", path = \"/drain\")]\nfn drain() int { var i = 0\nwhile i < 9000 { i = i + 1 }\nreturn i }".into(),
            "internal-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() ~int { return ~{ delay_async(10).await\nhttp.response_redirect(\"/x\", 302) } }\n#[api(method = \"GET\", path = \"/drain\")]\nfn drain() int { var i = 0\nwhile i < 9000 { i = i + 1 }\nreturn i }".into(),
            "error-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let held = http.response_redirect(\"/x\", 302)\nlet bad = 1 / 0\nreturn held }\n#[api(method = \"GET\", path = \"/drain\")]\nfn drain() int { var i = 0\nwhile i < 9000 { i = i + 1 }\nreturn i }".into(),
            "async-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() ~int { delay_async(10)\nreturn http.response_redirect(\"/x\", 302) }\n#[api(method = \"GET\", path = \"/drain\")]\nfn drain() int { var i = 0\nwhile i < 9000 { i = i + 1 }\nreturn i }".into(),
            _ => panic!("unknown mode"),
        };
        let result = auto_lang::run(&code);
        let _ = tx.send(format!("{result:?}"));
    }).unwrap();
    let until = Instant::now() + Duration::from_secs(8);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        if let Ok(err) = rx.try_recv() { panic!("server exited: {err}"); }
        assert!(Instant::now() < until, "server readiness deadline");
        std::thread::sleep(Duration::from_millis(25));
    }
    fn get(port: u16, path: &str) -> Vec<u8> {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        write!(s, "GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut data = Vec::new(); let mut buf = [0; 4096];
        loop { match s.read(&mut buf) { Ok(0) => break, Ok(n) => data.extend_from_slice(&buf[..n]), Err(_) => break } }
        data
    }
    println!("WIRE before_live={}", resource::live_resource_count());
    if std::env::args().nth(1).unwrap() == "cancel-life" {
        let mut s = TcpStream::connect(("127.0.0.1", port)).unwrap();
        write!(s, "GET /p HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while resource::live_resource_count() == 0 {
            assert!(Instant::now() < deadline, "resource allocation deadline");
            std::thread::sleep(Duration::from_millis(2));
        }
        println!("CANCEL live_before_disconnect={}", resource::live_resource_count());
        s.shutdown(std::net::Shutdown::Both).unwrap(); drop(s);
        std::thread::sleep(Duration::from_millis(500));
        get(port, "/drain");
        println!("CANCEL live_after_completion_and_drain={}", resource::live_resource_count());
        return;
    }
    let response = get(port, "/p");
    println!("WIRE response={}", String::from_utf8_lossy(&response));
    if std::env::args().nth(1).unwrap().ends_with("life") || std::env::args().nth(1).unwrap() == "local" {
        for _ in 0..10 { get(port, "/p"); }
        get(port, "/drain");
        println!("WIRE after_11_and_drain_live={}", resource::live_resource_count());
    }
}
fn main() {
    let mode = std::env::args().nth(1).unwrap_or("units".into());
    if mode == "units" { units(); } else { wire(&mode); }
}
