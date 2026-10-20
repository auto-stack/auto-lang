//! PLAN-816 具名测试子进程(auto-tool-execution 寿命正反例的确定性对端;
//! integration test 经 `CARGO_BIN_EXE_te-child` 引用)。
//!
//! 行为由环境变量驱动(argv 保留给被测的 argv 透传语义,不混入协议):
//! - `TE_CHILD_MODE`:echo-args | exit | sleep | pwd | env | stdout-kib |
//!   stderr-line | stdin-echo | grandchild | heartbeat
//! - `TE_CHILD_ARG1`/`TE_CHILD_ARG2`:模式的参数(见各分支)。
//!
//! 所有模式自带截止(sleep 秒数有限),不做无界等待。

use std::io::{Read, Write};
use std::process::Command;

const MODE: &str = "TE_CHILD_MODE";
const ARG1: &str = "TE_CHILD_ARG1";
const ARG2: &str = "TE_CHILD_ARG2";

fn arg1() -> String {
    std::env::var(ARG1).unwrap_or_default()
}

fn arg2() -> String {
    std::env::var(ARG2).unwrap_or_default()
}

fn main() {
    let mode = std::env::var(MODE).unwrap_or_default();
    match mode.as_str() {
        // 每个 argv 参数(含 argv[0] 之后的全部)单行打到 stdout;空参数
        // 打印空行(长度即身份)。
        "echo-args" => {
            let mut out = std::io::stdout();
            for a in std::env::args_os().skip(1) {
                writeln!(out, "{}", a.to_string_lossy()).unwrap();
            }
        }
        // ARG1 = 退出码。
        "exit" => {
            let code: i32 = arg1().trim().parse().unwrap_or(0);
            // 经内部 exit 保留已写 stdout 的冲洗语义。
            std::process::exit(code);
        }
        // ARG1 = 秒数(有限,测试用 30s 级,由设施的 deadline/cancel 终止)。
        "sleep" => {
            let secs: u64 = arg1().trim().parse().unwrap_or(1).min(30);
            std::thread::sleep(std::time::Duration::from_secs(secs));
        }
        // 打印当前工作目录(断言 cwd 生效)。
        "pwd" => {
            println!(
                "{}",
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<cwd-err>".into())
            );
        }
        // ARG1 = 环境变量名;打印其值(缺失 = 退出码 3)。
        "env" => match std::env::var(arg1()) {
            Ok(v) => println!("{v}"),
            Err(_) => std::process::exit(3),
        },
        // ARG1 = KiB 数;stdout 写满该量(大输出不死锁反例)。
        // ARG2 = 写完后小睡秒数(默认 0;供上层客户端流污染反例保持进程
        // 存活,使请求写入先于进程退出完成、时序确定)。
        "stdout-kib" => {
            let kib: usize = arg1().trim().parse().unwrap_or(1).min(8192);
            let nap: u64 = arg2().trim().parse().unwrap_or(0).min(30);
            let chunk = [b'x'; 1024];
            let mut out = std::io::stdout();
            for _ in 0..kib {
                out.write_all(&chunk).unwrap();
            }
            out.flush().unwrap();
            if nap > 0 {
                std::thread::sleep(std::time::Duration::from_secs(nap));
            }
        }
        // ARG1 = 一行文本打到 stderr;stdout 保持空(流分离断言)。
        "stderr-line" => {
            eprintln!("{}", arg1());
        }
        // stdin 全量读到 EOF 后原样写 stdout(stdin 管道往返)。
        "stdin-echo" => {
            let mut data = Vec::new();
            std::io::stdin().read_to_end(&mut data).unwrap();
            std::io::stdout().write_all(&data).unwrap();
        }
        // ARG1 = 环境变量名。两行打到 stderr 后小睡 300ms 再退出:
        //   cwd <当前目录>
        //   env <name>=<value>
        // 供上层客户端验证 cwd/env 经其 spec 真实到达子进程(816 T-12);
        // stdout 保持空(控制流不被污染),小睡让请求写入先行完成,稳定
        // 观察到 EOF(StreamClosed)而非 broken-pipe 竞态。
        "probe" => {
            let name = arg1();
            let value = std::env::var(&name).unwrap_or_else(|_| "<missing>".into());
            eprintln!(
                "cwd {}",
                std::env::current_dir()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| "<cwd-err>".into())
            );
            eprintln!("env {name}={value}");
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        // ARG1 = 孙进程存活秒数;ARG2 = heartbeat 文件路径。
        // 孙进程 = 自身 re-exec(heartbeat 模式),每 100ms 追加
        // "tick <millis>";子进程等孙进程结束后退出(带截止)。
        "grandchild" => {
            let secs: u64 = arg1().trim().parse().unwrap_or(5).min(30);
            let hb = arg2();
            let exe = std::env::current_exe().unwrap();
            let status = Command::new(&exe)
                .env(MODE, "heartbeat")
                .env(ARG1, secs.to_string())
                .env(ARG2, &hb)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .unwrap();
            std::process::exit(status.code().unwrap_or(0));
        }
        // ARG1 = 后代存活秒数;ARG2 = heartbeat 文件路径。
        // 启动即派生一个**继承 stdio** 的后代(heartbeat 模式),自身随即
        // 退出 0:后代持有本执行的 stdin/stdout/stderr 管道(F-816-R1-02
        // 持管道反例),同时证明 spawn 返回前已存在的后代属 owned 树
        // (F-816-R1-04 早派生——挂起入 Job 机制下不可能逃出)。
        // 最小协议宿主(816 R2-03 关闭路径反例):读一个请求帧(4 字节
        // 前缀+载荷),回一个合法 UnknownOperation 错误信封(id=1);随后
        // 读 stdin 到 EOF——ARG1=bad 时向 stdout 写 4 字节坏前缀
        // ff ff ff ff 再退 0(关闭阶段坏帧),clean 则干净退出。
        "proto-close" => {
            let bad = arg1() == "bad";
            // 读请求帧:前缀+载荷(忽略内容)。
            let mut prefix = [0u8; 4];
            std::io::stdin()
                .read_exact(&mut prefix)
                .expect("read frame prefix");
            let len = u32::from_be_bytes(prefix) as usize;
            let mut payload = vec![0u8; len];
            std::io::stdin()
                .read_exact(&mut payload)
                .expect("read frame payload");
            // 回合法错误信封 {"id":1,"body":{"type":"error","error":
            // {"kind":"unknown_operation","operation":"probe",...}}}。
            // ToolError/ErrorKind 为 internally-tagged 嵌套形状(serde 派生)。
            let reply = br#"{"id":1,"body":{"type":"error","error":{"kind":{"kind":"unknown_operation","operation":"probe"},"message":"te-child proto-close"}}}"#;
            let mut frame = (reply.len() as u32).to_be_bytes().to_vec();
            frame.extend_from_slice(reply);
            std::io::stdout().write_all(&frame).unwrap();
            std::io::stdout().flush().unwrap();
            // 等 EOF(客户端 shutdown 关闭 stdin)。
            let mut sink = Vec::new();
            std::io::stdin().read_to_end(&mut sink).unwrap();
            if bad {
                std::io::stdout()
                    .write_all(&[0xff, 0xff, 0xff, 0xff])
                    .unwrap();
                std::io::stdout().flush().unwrap();
            }
        }
        "orphan" => {
            let secs: u64 = arg1().trim().parse().unwrap_or(5).min(30);
            let hb = arg2();
            let exe = std::env::current_exe().unwrap();
            let _child = Command::new(&exe)
                .env(MODE, "heartbeat")
                .env(ARG1, secs.to_string())
                .env(ARG2, &hb)
                .spawn() // 默认继承 stdio:后代持有管道。
                .unwrap();
            drop(_child); // 不等待:自身立即退出。
        }
        // ARG1 = 存活秒数;ARG2 = heartbeat 文件路径。
        "heartbeat" => {
            let secs: u64 = arg1().trim().parse().unwrap_or(5).min(30);
            let hb = arg2();
            let started = std::time::Instant::now();
            loop {
                let elapsed_ms = started.elapsed().as_millis();
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&hb)
                {
                    writeln!(f, "tick {elapsed_ms}").unwrap();
                }
                if started.elapsed() >= std::time::Duration::from_secs(secs) {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        _ => {
            eprintln!("te-child: unknown or missing {MODE}");
            std::process::exit(64);
        }
    }
}
