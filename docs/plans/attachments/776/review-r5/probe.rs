use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration, Instant};
fn main() {
    let mode = std::env::args().nth(1).unwrap();
    if mode == "identity" {
        use auto_lang::api::{types::ApiType, contract::ResponseKind};
        let ts = vec![ApiType{name:"FileResponse".into(), fields:vec![], doc:None, alias_of:None}];
        println!("qualified-shadow {:?}", ResponseKind::from_return_identity("http.FileResponse", &ts));
        for (name, code) in [
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
        _ => panic!("unknown mode"),
    };
    let code = format!("fn events() ~Iter<int> {{ yield 42 }}\nfn helper() ~Iter<int> {{ let s = events()\nreturn s }}\n#[api(method = \"GET\", path = \"/p\")]\nfn p() ~Iter<int> {{ {tail} }}");
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
