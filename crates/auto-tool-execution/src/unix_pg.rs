//! Unix 进程组树终止(零依赖:libc 符号显式 extern 声明)。
//!
//! owned 树语义:子进程在 exec 前入新进程组(setpgid(0,0),消除了
//! 父侧 setpgid 与 exec 的竞态);取消 = killpg。注意进程组不在终端
//! 前台组,Ctrl+C 不再直达子进程——交互式命令保持
//! `TreeScope::DirectChildOnly`(lib.rs 模块文档边界)。

use std::io;

extern "C" {
    fn setpgid(pid: i32, pgid: i32) -> i32;
    fn kill(pgid: i32, sig: i32) -> i32;
}

const SIGKILL: i32 = 9;

/// pre_exec 钩子:子进程自身入新进程组。setpgid 是 async-signal-safe,
/// 在 pre_exec 中合法。
pub(crate) fn child_setpgid() -> io::Result<()> {
    unsafe {
        if setpgid(0, 0) == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

/// 向整个进程组发送指定信号(group leader pid == pgid)。
///
/// F-816-B-R9-01:`kill` 传**正值**只向同号 PID(根)发信号——owned 后代
/// 收不到;必须传负 PGID 才是组信号(killpg 语义)。信号只达组内成员;
/// ESRCH=无此进程组,视为成功(收尾场景组空即无事可做)。
pub(crate) fn signal_group(pgid: u32, sig: i32) -> io::Result<()> {
    unsafe {
        if kill(-(pgid as i32), sig) == -1 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(3) {
                Ok(())
            } else {
                Err(err)
            }
        } else {
            Ok(())
        }
    }
}

/// 终止整个进程组(group leader pid == pgid)。
///
/// F-816-B-R9-01:负 PGID 组信号——正值只杀根,owned 后代继续运行。
/// 组已空(ESRCH)视为成功:调用场景含"根已退出、仅剩/已无后代"的
/// 收尾(F-816-R1-02)——组在任一成员存活期间持续存在(根退出/已收尸
/// 不撤销 PGID),无成员即无事可做,不是失败。
pub(crate) fn kill_group(pgid: u32) -> io::Result<()> {
    unsafe {
        if kill(-(pgid as i32), SIGKILL) == -1 {
            let err = io::Error::last_os_error();
            if err.raw_os_error() == Some(3) {
                // ESRCH: 组内已无成员。
                Ok(())
            } else {
                Err(err)
            }
        } else {
            Ok(())
        }
    }
}

/// 将 fd 设为非阻塞（stdin 写端的可停止写：EAGAIN + poll 轮询，
/// F-816-R5；Windows 侧由自建重叠管道承担同语义）。
#[cfg(unix)]
pub(crate) fn set_nonblocking(fd: i32) {
    const F_GETFL: i32 = 3;
    const F_SETFL: i32 = 4;
    const O_NONBLOCK: i32 = 0o4000;
    extern "C" {
        fn fcntl(fd: i32, cmd: i32, arg: i32) -> i32;
    }
    unsafe {
        let flags = fcntl(fd, F_GETFL, 0);
        if flags >= 0 {
            fcntl(fd, F_SETFL, flags | O_NONBLOCK);
        }
    }
}
