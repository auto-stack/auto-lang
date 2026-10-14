use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
fn main() {
    let mode = std::env::args().nth(1).unwrap();
    if mode == "native" || mode == "shared-native" {
        use auto_lang::vm::{engine::{AutoVM, Iterator, ListIterator}, task::AutoTask, virt_memory::VirtualFlash};
        let vm = AutoVM::new(VirtualFlash::new(1024),1024);
        let list_id=vm.insert_heap_object(auto_lang::vm::types::ListData::<i32>{elems:vec![42],storage:None});
        vm.iterators.insert(0, Iterator::List(ListIterator{list_id,current_index:0}));
        vm.iterator_id_gen.store(1,std::sync::atomic::Ordering::Relaxed);
        let mut raw = AutoTask::new(0,1024,0);
        raw.ram.push_i32(0);
        let a = auto_lang::vm::native::shim_iterator_enumerate(&mut raw,&vm);
        println!("raw enumerate result={a:?} handle={}",raw.ram.pop_i32());
        let mut wrapped = AutoTask::new(0,1024,0);
        auto_lang::vm::ffi::http_return_resource::push_sse_resource(&mut wrapped,&vm,0).unwrap();
        let source_nv=wrapped.ram.peek_nv(0);
        let source_id=source_nv as u32 as u64;
        if mode == "shared-native" {
            vm.rc_retain_id(source_id);
            vm.globals.insert("kept".into(),source_nv);
        }
        println!("source credential rc before consumed argument={}",vm.rc_count(source_id));
        let b = auto_lang::vm::native::shim_iterator_enumerate(&mut wrapped,&vm);
        let nv=wrapped.ram.pop_nv();
        println!("credential enumerate result={b:?} claim={:?}",auto_lang::vm::ffi::http_return_resource::resolve_resource(&vm,nv));
        println!("source credential rc after consumed argument={}",vm.rc_count(source_id));
        vm.rc_release(nv);
        vm.reap_all();
        println!("source credential retained after output release/reap={} rc={}",vm.heap_objects.contains_key(&source_id),vm.rc_count(source_id));
        let mut clear = AutoTask::new(0,1024,0);
        auto_lang::vm::ffi::http_return_resource::push_sse_resource(&mut clear,&vm,0).unwrap();
        let original=clear.ram.peek_nv(0);
        let receiver=auto_lang::vm::native::pop_iterator_receiver(&mut clear,&vm);
        println!("receiver={receiver:?} popped-slot-actually-zero={} original-tag-retained={}",clear.ram.raw_nv[clear.ram.sp]==0,clear.ram.raw_nv[clear.ram.sp]==original);
        return;
    }
    if mode == "identity" {
        use auto_lang::api::{types::ApiType, contract::ResponseKind};
        let ts = vec![ApiType{name:"FileResponse".into(), fields:vec![], doc:None, alias_of:None}];
        println!("qualified-shadow {:?}", ResponseKind::from_return_identity("http.FileResponse", &ts));
        for (name, code) in [
            ("alias-future", "type Download = Future<FileResponse>;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Download { return 0 }"),
            ("alias-nine", "type A0 = FileResponse;\ntype A1 = A0;\ntype A2 = A1;\ntype A3 = A2;\ntype A4 = A3;\ntype A5 = A4;\ntype A6 = A5;\ntype A7 = A6;\ntype A8 = A7;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() A8 { return 0 }"),
            ("qualified-shadow", "type FileResponse {}\n#[api(method = \"GET\", path = \"/p\")]\nfn p() http.FileResponse { return 0 }"),
            ("alias-qualified", "type Download = http.FileResponse;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Download { return 0 }"),
            ("alias-chain", "type First = FileResponse;\ntype Download = First;\n#[api(method = \"GET\", path = \"/p\")]\nfn p() Download { return 0 }"),
        ] {
            let mut p = auto_lang::parser::Parser::from(code);
            let ast = p.parse().unwrap();
            let m = auto_lang::api::ApiExtractor::new().extract("probe", &ast.stmts);
            println!("{name}: types={:?} endpoints={:?}", m.types, m.endpoints.iter().map(|e| (&e.return_type, e.response)).collect::<Vec<_>>());
        }
        return;
    }
    let tail = match mode.as_str() {
        "direct" => "return events()",
        "local" => "let stream = events()\nreturn stream",
        "helper" => "return helper()",
        "internal" => "return ~{ events() }",
        "plain" => "let stream = events()\nreturn 0",
        "legacy-int" => "let stream = events()\nreturn stream",
        "helper-int" => "return legacy_helper()",
        "future-number" | "future-number-int" => "let active = ~{ 42 }\nreturn 496",
        "future-branch" => "let active = ~{ 42 }\nif false { return ~{ 99 } }\nreturn 496",
        "future-helper" => "return future_helper()",
        "future-number-async" => "let active = ~{ 42 }\nreturn 496",
        "future-array" => "let values = [~{ 42 }]\nreturn values[0]",
        "future-record" => "let holder = Box { f: ~{ 42 } }\nreturn holder.f",
        "future-closure" => "let actual = ~{ 42 }\nlet get = fn() int { return actual }\nreturn get()",
        _ => panic!("unknown mode"),
    };
    let ret = if mode == "future-number-async" { "~int" } else if mode == "legacy-int" || mode == "future-number-int" { "int" } else { "~Iter<int>" };
    let code = format!("type Box {{ f Future<int> }}\nfn events() ~Iter<int> {{ yield 42 }}\nfn helper() ~Iter<int> {{ let s = events()\nreturn s }}\nfn legacy_helper() int {{ let s = events()\nreturn s }}\nfn future_helper() int {{ return ~{{ 42 }} }}\n#[api(method = \"GET\", path = \"/p\")]\nfn p() {ret} {{ {tail} }}");
    let l = TcpListener::bind("127.0.0.1:0").unwrap(); let port = l.local_addr().unwrap().port(); drop(l);
    std::env::set_var("AUTO_HTTP_PORT", port.to_string());
    let (tx,rx) = std::sync::mpsc::channel();
    std::thread::Builder::new().stack_size(8*1024*1024).spawn(move || {let r=std::panic::catch_unwind(|| auto_lang::run(&code)); let _=tx.send(format!("{r:?}"));}).unwrap();
    let deadline = Instant::now()+Duration::from_secs(8);
    loop {
        if TcpStream::connect(("127.0.0.1",port)).is_ok() { break; }
        if let Ok(r)=rx.try_recv() { panic!("startup: {r}"); }
        assert!(Instant::now()<deadline,"readiness deadline");
        std::thread::sleep(Duration::from_millis(20));
    }
    let mut s=TcpStream::connect(("127.0.0.1",port)).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    write!(s,"GET /p HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n").unwrap();
    let mut raw=Vec::new(); let mut buf=[0;4096];
    loop { match s.read(&mut buf) {Ok(0)=>break,Ok(n)=>{raw.extend_from_slice(&buf[..n]); if String::from_utf8_lossy(&raw).contains("data: 42") { break; }},Err(e)=>panic!("read: {e}")}}
    println!("{mode}: {}",String::from_utf8_lossy(&raw));
}
