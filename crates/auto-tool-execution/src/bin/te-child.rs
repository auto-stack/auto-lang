//! PLAN-816 具名测试子进程(auto-tool-execution 寿命正反例的确定性对端;
//! integration test 经 `CARGO_BIN_EXE_te-child` 引用)。
//!
//! 行为由环境变量驱动(argv 保留给被测的 argv 透传语义,不混入协议):
//! - `TE_CHILD_MODE`:echo-args | exit | sleep | pwd | env | stdout-kib |
//!   stderr-line | stdin-echo | grandchild | heartbeat | stdout-holdout |
//!   fd-receiver(Unix) | signal-tree | terminate-wait
//! - `TE_CHILD_ARG1`/`TE_CHILD_ARG2`:模式的参数(见各分支)。
//!   `TE_CHILD_ARG3`(仅 Unix fd-receiver):持有秒数。
//!
//! 所有模式自带截止(sleep 秒数有限),不做无界等待。

use std::io::{Read, Write};
use std::process::Command;

const MODE: &str = "TE_CHILD_MODE";
const ARG1: &str = "TE_CHILD_ARG1";
const ARG2: &str = "TE_CHILD_ARG2";
#[cfg(unix)]
const ARG3: &str = "TE_CHILD_ARG3";

fn arg1() -> String {
    std::env::var(ARG1).unwrap_or_default()
}

fn arg2() -> String {
    std::env::var(ARG2).unwrap_or_default()
}

#[cfg(unix)]
fn arg3() -> String {
    std::env::var(ARG3).unwrap_or_default()
}

fn main() {
    let mode = std::env::var(MODE).unwrap_or_default();
    match mode.as_str() {
        "stdout-invalid-utf8" => {
            std::io::stdout()
                .write_all(b"{\"value\":\"\xff\"}")
                .unwrap();
        }
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
        // ARG1 = 根存活秒数;ARG2 = heartbeat 文件路径。派生一个**仅继承
        // stdin** 的后代(heartbeat 模式,stdout/stderr 置 null)后根保持
        // 睡眠——供上层写端在"根活着 + 非拥有持读者不读"语境下验证
        // 取消真停(816 R5)。
        "stdin-holder" => {
            let secs: u64 = arg1().trim().parse().unwrap_or(5).min(30);
            let hb = arg2();
            let exe = std::env::current_exe().unwrap();
            let _child = Command::new(&exe)
                .env(MODE, "heartbeat")
                .env(ARG1, secs.to_string())
                .env(ARG2, &hb)
                .stdin(std::process::Stdio::inherit()) // 持有根的 stdin 读端
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();
            drop(_child);
            std::thread::sleep(std::time::Duration::from_secs(secs));
        }
        // ARG1 = 转交后存活秒数(0=转交后立即退 0);ARG2 = Unix: receiver
        // 的 unix socket 路径 / Windows: receiver PID。stdout 先打 marker
        // 行,再把写端转交给**不在本树内**的外部持有者(Unix=SCM_RIGHTS
        // 经 unix socket / Windows=DuplicateHandle 转入 receiver 句柄表)
        // ——上层杀 owned 树不给 EOF,reader 停止只能靠停止协议
        // (PLAN-816 B-R7-04 真实超时出口反例)。
        "stdout-holdout" => {
            let secs: u64 = arg1().trim().parse().unwrap_or(0).min(30);
            println!("holdout output before sleep");
            let _ = std::io::stdout().flush();
            #[cfg(unix)]
            holdout_unix::transfer_stdout(&arg2()).expect("transfer stdout write fd");
            #[cfg(windows)]
            holdout_win::transfer_stdout_to(arg2().trim().parse::<u32>().expect("receiver pid"))
                .expect("transfer stdout write handle");
            if secs > 0 {
                std::thread::sleep(std::time::Duration::from_secs(secs));
            }
        }
        // ARG1 = socket 路径;ARG2 = marker 文件;ARG3 = 持有秒数(Unix)。
        // bind+listen 后 accept 一连接,recvmsg 收 SCM_RIGHTS 写 fd,写
        // marker 确认真实收到,随后持有 fd 存活 ARG3 秒(自带截止)。
        // 握手(accept+recvmsg)自有真实截止(F-816-B-R8-01):无人连接/
        // 静默连接时**整体有界退出**(exit 4 具名),"持有 N 秒"只在握手
        // 成功后开始。默认 10s,可经 TE_CHILD_HANDSHAKE_SECS 调短(≤30)。
        "fd-receiver" => {
            #[cfg(unix)]
            {
                let secs: u64 = arg3().trim().parse().unwrap_or(5).min(30);
                let handshake_secs: u64 = std::env::var("TE_CHILD_HANDSHAKE_SECS")
                    .ok()
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(10)
                    .min(30);
                if let Err(e) = holdout_unix::receive_fd(
                    &arg1(),
                    &arg2(),
                    std::time::Duration::from_secs(handshake_secs),
                ) {
                    eprintln!("te-child fd-receiver: {e}");
                    std::process::exit(4);
                }
                std::thread::sleep(std::time::Duration::from_secs(secs));
            }
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
        // 安装设施终止原语后等待真实终止信号（Ctrl+C/SIGTERM/SIGINT）：
        // 收到即干净退出码 42；10s 未收到=7（截止自守）。
        // ARG1（可选）= ready marker 路径：安装完成后写文件——供上层
        // 确定性等到 handler 已挂载再发信号（消除安装/送达竞态）。
        "terminate-wait" => {
            assert!(auto_tool_execution::terminate::install(), "install failed");
            let ready = arg1();
            if !ready.is_empty() {
                std::fs::write(&ready, "handler installed").unwrap();
            }
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !auto_tool_execution::terminate::requested() {
                if std::time::Instant::now() >= deadline {
                    std::process::exit(7);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            std::process::exit(42);
        }
        // ARG1 = ready marker 路径（F-816-B-R9-01 组信号送达证明对端）：
        // 安装终止原语后派生 terminate-wait 后代（同样安装，ARG1 透传为
        // "<marker>.child"），等待后代 handler 就绪再写自身 marker（双端
        // 已挂载的确定性握手），随后等待后代退出并传播其退出码——组
        // SIGINT/SIGTERM 送达整组时后代经 handler 干净退 42；根同在组内
        // 靠自身 handler 存活以报告。未送达=后代 10s 截止退 7。
        "signal-tree" => {
            assert!(auto_tool_execution::terminate::install(), "install failed");
            let marker = arg1();
            let child_marker = format!("{marker}.child");
            let exe = std::env::current_exe().unwrap();
            let mut child = Command::new(&exe)
                .env(MODE, "terminate-wait")
                .env(ARG1, &child_marker)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .unwrap();
            let until = std::time::Instant::now() + std::time::Duration::from_secs(10);
            while !std::path::Path::new(&child_marker).exists() && std::time::Instant::now() < until
            {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            assert!(
                std::path::Path::new(&child_marker).exists(),
                "descendant handler must install within 10s"
            );
            std::fs::write(&marker, "both handlers installed").unwrap();
            let status = child.wait().unwrap();
            std::process::exit(status.code().unwrap_or(-1));
        }
        _ => {
            eprintln!("te-child: unknown or missing {MODE}");
            std::process::exit(64);
        }
    }
}

/// stdout 写端 SCM_RIGHTS 转交/接收(PLAN-816 B-R7-04 持有者反例;
/// 零依赖:libc 符号显式 extern 声明,镜像 unix_pg.rs 模式)。
#[cfg(unix)]
mod holdout_unix {
    use std::io;
    use std::os::unix::io::AsRawFd;

    const AF_UNIX: i32 = 1;
    const SOCK_STREAM: i32 = 1;
    const SOL_SOCKET: i32 = 1;
    const SCM_RIGHTS: i32 = 1;

    #[repr(C)]
    struct SockAddrUn {
        family: u16,
        path: [u8; 108],
    }

    #[repr(C)]
    struct IoVec {
        base: *mut u8,
        len: usize,
    }

    /// cmsg 缓冲:cmsghdr(16B) + 1 个 fd(4B) + 对齐填充 = CMSG_SPACE(int)。
    #[repr(C)]
    struct CmsgFd {
        len: usize,
        level: i32,
        kind: i32,
        fd: i32,
        pad: i32,
    }

    #[repr(C)]
    struct MsgHdr {
        name: *mut u8,
        namelen: u32,
        iov: *mut IoVec,
        iovlen: usize,
        control: *mut u8,
        controllen: usize,
        flags: i32,
    }

    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }

    const POLLIN: i16 = 0x001;
    const EINTR: i32 = 4;

    extern "C" {
        fn socket(domain: i32, ty: i32, protocol: i32) -> i32;
        fn connect(fd: i32, addr: *const SockAddrUn, len: u32) -> i32;
        fn sendmsg(fd: i32, msg: *const MsgHdr, flags: i32) -> isize;
        fn bind(fd: i32, addr: *const SockAddrUn, len: u32) -> i32;
        fn listen(fd: i32, backlog: i32) -> i32;
        fn accept(fd: i32, addr: *mut u8, len: *mut u32) -> i32;
        fn recvmsg(fd: i32, msg: *mut MsgHdr, flags: i32) -> isize;
        fn poll(fds: *mut PollFd, nfds: u64, timeout_ms: i32) -> i32;
        fn close(fd: i32) -> i32;
    }

    /// 等待 fd 可读,最迟 `deadline`(F-816-B-R8-01:握手期无真实截止会
    /// 在无人连接/静默连接时无限阻塞)。EINTR 重试;到期返回具名错误。
    fn wait_readable(fd: i32, deadline: std::time::Instant, stage: &str) -> io::Result<()> {
        let mut pfd = PollFd {
            fd,
            events: POLLIN,
            revents: 0,
        };
        loop {
            let now = std::time::Instant::now();
            if now >= deadline {
                return Err(io::Error::other(format!(
                    "handshake deadline exceeded at {stage}"
                )));
            }
            let timeout_ms = ((deadline - now).as_millis().min(i32::MAX as u128)) as i32;
            let ready = unsafe { poll(&mut pfd, 1, timeout_ms) };
            if ready < 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(EINTR) {
                    continue;
                }
                return Err(e);
            }
            if ready > 0 {
                return Ok(());
            }
            // ready == 0:超时,下一轮循环的 deadline 检查具名退出。
        }
    }

    fn sun_addr(path: &str) -> io::Result<(SockAddrUn, u32)> {
        let bytes = path.as_bytes();
        if bytes.len() >= 108 {
            return Err(io::Error::other("unix socket path too long"));
        }
        let mut addr = SockAddrUn {
            family: AF_UNIX as u16,
            path: [0; 108],
        };
        addr.path[..bytes.len()].copy_from_slice(bytes);
        // sockaddr 长度 = family(2) + 路径字节 + NUL。
        Ok((addr, 2 + bytes.len() as u32 + 1))
    }

    /// 把自身 stdout 写端经 SCM_RIGHTS 发给 socket 对端(receiver)。
    pub fn transfer_stdout(socket_path: &str) -> io::Result<()> {
        let (addr, addr_len) = sun_addr(socket_path)?;
        unsafe {
            let fd = socket(AF_UNIX, SOCK_STREAM, 0);
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            if connect(fd, &addr, addr_len) != 0 {
                let e = io::Error::last_os_error();
                close(fd);
                return Err(e);
            }
            let stdout_fd = std::io::stdout().as_raw_fd();
            // 至少 1 字节数据捎带 cmsg(零长度发送不保证携带辅助数据)。
            let byte = [0u8];
            let mut iov = IoVec {
                base: byte.as_ptr() as *mut u8,
                len: 1,
            };
            let mut cmsg = CmsgFd {
                len: 20, // CMSG_LEN(sizeof(int)) = 头(16) + fd(4)
                level: SOL_SOCKET,
                kind: SCM_RIGHTS,
                fd: stdout_fd,
                pad: 0,
            };
            let msg = MsgHdr {
                name: std::ptr::null_mut(),
                namelen: 0,
                iov: &mut iov,
                iovlen: 1,
                control: &mut cmsg as *mut CmsgFd as *mut u8,
                controllen: 24, // CMSG_SPACE(sizeof(int))
                flags: 0,
            };
            if sendmsg(fd, &msg, 0) < 0 {
                let e = io::Error::last_os_error();
                close(fd);
                return Err(e);
            }
            close(fd);
            Ok(())
        }
    }

    /// bind+listen `socket_path`,accept 一连接并 recvmsg 收 SCM_RIGHTS
    /// 写 fd;marker 写入"真实收到"确认(含 fd 号)。fd 故意不关闭:持有
    /// 该写端直到进程退出(持有者语义——root 被杀后管道仍无 EOF)。
    /// `handshake` 为 accept+recvmsg 的整体真实截止(F-816-B-R8-01):
    /// 无人连接/静默连接时到期具名退出,不再无限阻塞在内核等待点。
    pub fn receive_fd(
        socket_path: &str,
        marker: &str,
        handshake: std::time::Duration,
    ) -> io::Result<()> {
        let (addr, addr_len) = sun_addr(socket_path)?;
        let deadline = std::time::Instant::now() + handshake;
        unsafe {
            let fd = socket(AF_UNIX, SOCK_STREAM, 0);
            if fd < 0 {
                return Err(io::Error::last_os_error());
            }
            if bind(fd, &addr, addr_len) != 0 || listen(fd, 1) != 0 {
                let e = io::Error::last_os_error();
                close(fd);
                return Err(e);
            }
            if let Err(e) = wait_readable(fd, deadline, "accept") {
                close(fd);
                return Err(e);
            }
            let conn = accept(fd, std::ptr::null_mut(), std::ptr::null_mut());
            if conn < 0 {
                let e = io::Error::last_os_error();
                close(fd);
                return Err(e);
            }
            if let Err(e) = wait_readable(conn, deadline, "recvmsg") {
                close(conn);
                close(fd);
                return Err(e);
            }
            let mut byte = [0u8; 1];
            let mut iov = IoVec {
                base: byte.as_mut_ptr(),
                len: 1,
            };
            let mut cmsg: CmsgFd = std::mem::zeroed();
            let mut msg = MsgHdr {
                name: std::ptr::null_mut(),
                namelen: 0,
                iov: &mut iov,
                iovlen: 1,
                control: &mut cmsg as *mut CmsgFd as *mut u8,
                controllen: 24,
                flags: 0,
            };
            if recvmsg(conn, &mut msg, 0) < 0 {
                let e = io::Error::last_os_error();
                close(conn);
                close(fd);
                return Err(e);
            }
            close(conn);
            close(fd);
            if cmsg.level != SOL_SOCKET || cmsg.kind != SCM_RIGHTS || cmsg.len < 20 || cmsg.fd <= 2
            {
                return Err(io::Error::other("no SCM_RIGHTS fd received"));
            }
            std::fs::write(marker, format!("actual write fd received: {}", cmsg.fd))?;
            Ok(())
        }
    }
}

/// stdout 写端经 DuplicateHandle 转入 receiver 进程句柄表(PLAN-816
/// B-R7-04 持有者反例;符号显式 extern "system" 声明,零依赖)。
#[cfg(windows)]
mod holdout_win {
    use std::io;

    const STD_OUTPUT_HANDLE: u32 = 0xFFFF_FFF5; // (DWORD)-11
    const PROCESS_DUP_HANDLE: u32 = 0x0040;
    const DUPLICATE_SAME_ACCESS: u32 = 0x0002;

    extern "system" {
        fn GetCurrentProcess() -> isize;
        fn GetStdHandle(n_std_handle: u32) -> isize;
        fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> isize;
        fn DuplicateHandle(
            source_process: isize,
            source_handle: isize,
            target_process: isize,
            target_handle: *mut isize,
            desired_access: u32,
            inherit_handle: i32,
            options: u32,
        ) -> i32;
        fn CloseHandle(handle: isize) -> i32;
    }

    /// 把自身 stdout 写句柄复制进 receiver(按 PID)句柄表——receiver 持有
    /// 该写端,本进程退场/被杀不给 EOF(root 的 owned 树终止不波及它)。
    pub fn transfer_stdout_to(receiver_pid: u32) -> io::Result<()> {
        unsafe {
            let stdout_handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if stdout_handle == 0 || stdout_handle == -1 {
                return Err(io::Error::last_os_error());
            }
            let receiver = OpenProcess(PROCESS_DUP_HANDLE, 0, receiver_pid);
            if receiver == 0 {
                return Err(io::Error::last_os_error());
            }
            let mut target: isize = 0;
            let ok = DuplicateHandle(
                GetCurrentProcess(),
                stdout_handle,
                receiver,
                &mut target,
                0,
                0,
                DUPLICATE_SAME_ACCESS,
            );
            CloseHandle(receiver);
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            // target 位于 receiver 句柄表:本端无法也不需要关闭它。
            Ok(())
        }
    }
}
