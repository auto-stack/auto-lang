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
        "stdout-kib" => {
            let kib: usize = arg1().trim().parse().unwrap_or(1).min(8192);
            let chunk = [b'x'; 1024];
            let mut out = std::io::stdout();
            for _ in 0..kib {
                out.write_all(&chunk).unwrap();
            }
            out.flush().unwrap();
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
