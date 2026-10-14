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
    println!("OWNER adjacent a={} b={} same_encoded_handle={} foreign={:?}", a.instance_id, b.instance_id, na==nb, resource::resolve_resource(&b, na));
    // A and B remain live; creating 127 more VMs makes final owner A+128.
    let mut last = b;
    for _ in 0..127 { last = vm(); }
    let mut tc = task();
    resource::push_response_resource(&mut tc, &last, 203, vec![], b"owner-a-plus-128".to_vec()).unwrap();
    let nc = tc.ram.pop_nv();
    println!("OWNER wrap a={} later={} same_encoded_handle={} foreign={:?}", a.instance_id, last.instance_id, na==nc, resource::resolve_resource(&last, na));
    let limit = vm(); let mut tl = task();
    limit.http_credential_gen.store((1<<24)-1, Ordering::Relaxed);
    println!("LIMIT last={:?}", resource::push_response_resource(&mut tl, &limit, 200, vec![], vec![]));
    let before = limit.heap_objects.len();
    println!("LIMIT next={:?} before={before} after={}", resource::push_response_resource(&mut tl, &limit, 200, vec![], vec![]), limit.heap_objects.len());
    let setter_vm = vm(); let mut ts = task();
    resource::push_response_resource(&mut ts, &setter_vm, 200, vec![], vec![]).unwrap();
    ts.ram.push_i32(201);
    stdlib::shim_http_response_status(&mut ts, &setter_vm).unwrap();
    let nv = ts.ram.read_nv(ts.ram.sp-1);
    let id = setter_vm.heap_objects.iter().next().unwrap().key().to_owned();
    println!("SETTER before_reap rc={} stake={} snapshot={:?}", setter_vm.rc_count(id), ts.ram.stake_at(ts.ram.sp-1), resource::read_response_snapshot(&setter_vm,nv));
    setter_vm.reap_dying(100_000);
    println!("SETTER after_reap returned_value_present={}", setter_vm.heap_objects.contains_key(&id));
    let bytes_vm=vm();let mut bs=task();
    resource::push_response_resource(&mut bs,&bytes_vm,201,vec![],vec![]).unwrap();
    let response_nv=bs.ram.read_nv(bs.ram.sp-1);
    let list_id=bytes_vm.insert_heap_object(auto_lang::vm::types::ListData::<i32>{elems:vec![65,66],storage:None});
    bytes_vm.rc_retain_id(list_id); // Retained argument control avoids an unrelated list-lifetime failure.
    bytes_vm.rc_push_id(&mut bs,list_id);
    let result=stdlib::shim_http_response_bytes(&mut bs,&bytes_vm);
    let returned=bs.ram.pop_nv();
    println!("BYTES result={result:?} returned_resource={:?} original_snapshot={:?}",resource::resolve_resource(&bytes_vm,returned),resource::read_response_snapshot(&bytes_vm,response_nv));
    for (name, code) in [
        ("empty", "type Response {}\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Response { return Response {} }"),
        ("alias-int", "type Response = int;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Response { return 9 }"),
        ("alias-file", "type Download = FileResponse;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Download { return http.file_response(\".\", \"x\", \"{}\") }"),
        ("qualified", "#[api(method = \"GET\", path = \"/p\")]\nfn p() http.FileResponse { return http.file_response(\".\", \"x\", \"{}\") }"),
    ] {
        let mut p=auto_lang::parser::Parser::from(code);
        match p.parse() { Ok(ast)=>{
            let m=auto_lang::api::ApiExtractor::new().extract("probe",&ast.stmts);
            println!("SOURCE {name} types={:?} endpoints={:?}",m.types.iter().map(|t|&t.name).collect::<Vec<_>>(),m.endpoints.iter().map(|e|(&e.return_type,e.response)).collect::<Vec<_>>());
        },Err(e)=>println!("SOURCE {name} error={e:?}") }
    }
}
fn wire(mode: &str) {
    let listener=TcpListener::bind("127.0.0.1:0").unwrap();
    let port=listener.local_addr().unwrap().port();drop(listener);
    std::env::set_var("AUTO_HTTP_PORT",port.to_string());
    let mut upstream_url=String::new();
    if mode=="client" {
        let upstream=TcpListener::bind("127.0.0.1:0").unwrap();
        upstream_url=format!("http://127.0.0.1:{}/",upstream.local_addr().unwrap().port());
        std::thread::spawn(move||{let (mut s,_)=upstream.accept().unwrap();let mut buf=[0;4096];s.read(&mut buf).unwrap();s.write_all(b"HTTP/1.1 201 Created\r\nContent-Length: 8\r\nConnection: close\r\n\r\nupstream").unwrap();});
    }
    let code=match mode {
        "iterator" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let live = stream()\nreturn 0 }\nfn stream() ~Iter<int> { yield 42 }".to_owned(),
        "declared-iterator" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() ~Iter<int> { let live = stream()\nreturn 0 }\nfn stream() ~Iter<int> { yield 42 }".into(),
        "legacy" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let live = http.response()\nreturn 1 }".into(),
        "internal-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() ~int { return ~{ delay_async(10).await\nhttp.response_redirect(\"/x\", 302) } }".into(),
        "error-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let held = http.response_redirect(\"/x\", 302)\nlet bad = 1 / 0\nreturn held }".into(),
        "future-cache" => "var cached = 0\n#[api(method = \"GET\", path = \"/p\")]\nfn p() int { cached = ~{ http.response_redirect(\"/cached\", 302) }\nreturn cached.await }\n#[api(method = \"GET\", path = \"/replay\")]\nfn replay() int { return cached.await }".into(),
        "bytes" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let r = http.response()\nhttp.response_status(r, 201)\nreturn http.response_bytes(r, [65, 66]) }".into(),
        "setter-life" => "#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let r = http.response_status(http.response(), 201)\nvar i = 0\nwhile i < 9000 { i = i + 1 }\nreturn r }".into(),
        "record" => "type Box { response int }\n#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let b = Box { response: http.response_redirect(\"/box\", 302) }\nreturn b.response }".into(),
        "nested-error-life" => "type Box { response int }\n#[api(method = \"GET\", path = \"/p\")]\nfn p() int { let b = Box { response: http.response_redirect(\"/box\", 302) }\nlet bad = 1 / 0\nreturn 9 }".into(),
        "client" => format!("#[api(method = \"GET\", path = \"/p\")]\nfn p() int {{ return http.get(\"{upstream_url}\") }}"),
        _=>panic!("unknown mode"),
    };
    let code=code+"\n#[api(method = \"GET\", path = \"/drain\")]\nfn drain() int { var i = 0\nwhile i < 9000 { i = i + 1 }\nreturn i }\n";
    let(tx,rx)=std::sync::mpsc::channel();
    std::thread::Builder::new().stack_size(8*1024*1024).spawn(move||{let r=std::panic::catch_unwind(||auto_lang::run(&code));let _=tx.send(format!("{r:?}"));}).unwrap();
    let deadline=Instant::now()+Duration::from_secs(8);
    while TcpStream::connect(("127.0.0.1",port)).is_err() {
        if let Ok(e)=rx.try_recv(){println!("STARTUP {e}");return;}
        assert!(Instant::now()<deadline,"readiness deadline");std::thread::sleep(Duration::from_millis(25));
    }
    fn get(port:u16,path:&str)->Vec<u8>{
        let mut s=TcpStream::connect(("127.0.0.1",port)).unwrap();s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        write!(s,"GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n").unwrap();
        let mut data=Vec::new();let mut buf=[0;4096];loop{match s.read(&mut buf){Ok(0)=>break,Ok(n)=>data.extend_from_slice(&buf[..n]),Err(e)=>{println!("READ_END {e}");break;}}}data
    }
    println!("WIRE baseline_live={}",resource::live_resource_count());
    println!("WIRE response={}",String::from_utf8_lossy(&get(port,"/p")));
    if mode.ends_with("life") {
        for _ in 0..10{get(port,"/p");}get(port,"/drain");println!("WIRE after_11_and_drain_live={}",resource::live_resource_count());
    }
    if mode=="future-cache"{get(port,"/drain");println!("WIRE after_drain_live={}",resource::live_resource_count());println!("WIRE replay={}",String::from_utf8_lossy(&get(port,"/replay")));}
    if let Ok(e)=rx.try_recv(){println!("SERVER_END {e}");}
}
fn main(){let mode=std::env::args().nth(1).unwrap_or("units".into());if mode=="units"{units();}else{wire(&mode);}}
