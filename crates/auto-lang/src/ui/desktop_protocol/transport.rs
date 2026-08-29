// Plan 386 S8 —— 桌面协议传输层：通道语义面 + Windows 命名管道实现。
//
// Stage 1 的 loopback（内存管道）收编为本 trait 的首个实现；Stage 2 的
// 真两进程走 Windows 命名管道——复用 tokio `net::windows::named_pipe`
// （autovm_daemon Plan 269 同款依赖），但姿势不同：**每端常驻一个
// worker 的多线程 runtime**（worker 持续驱动 reactor，block_on 随处
// 可用），读线程 `block_on(select!{read, shutdown})`——异步读由
// reactor 正确唤醒（对端关闭 → 0 字节/错误 → EOF），shutdown oneshot
// 由 Drop 触发干净取消（tokio 读 cancel-safe）。教训（本模块调试
// 实证）：同步句柄的阻塞 ReadFile 不被对端关闭可靠唤醒；手写
// PeekNamedPipe 轮询在空管道上会阻塞；他线程 CancelIoEx/
// CancelSynchronousIo 均打不断同步 IO；runtime 不 entered 时
// try_read/try_write 直接 panic。非 Windows 平台本模块只编译
// trait + loopback（Linux 侧 transport 随 libcosmic 宿主单独生长）。
//
// 分帧：u32 LE 长度前缀 + 信封（codec::encode_envelope 产物）。流式
// 管道按帧切分累积，残帧留在缓冲等下一段。

use std::collections::VecDeque;

use super::codec::CodecError;
use super::loopback::LoopbackEnd;
use super::message::ProtocolMsg;

/// 传输层错误（EOF = 对端断开——弹性语义的输入，见计划"弹性"行）。
#[derive(Debug, Clone, PartialEq)]
pub enum TransportError {
    Io(String),
    /// 对端已关闭（读得 0 字节 / 管道破裂）。
    Eof,
}

/// 一条通道的两端同语义面（loopback 与命名管道共用）。
pub trait Transport {
    /// 编码并完整写出（命名管道实现为阻塞写）。
    fn send(&mut self, msg: &ProtocolMsg) -> Result<(), TransportError>;

    /// 非阻塞弹出一条；无完整消息返回 None。
    fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>>;

    /// 已完整到帧、待解码的消息数。
    fn pending(&self) -> usize;

    /// 对端断开检测（EOF 后调用方按"等待重连"弹性语义处理）。
    fn is_eof(&self) -> bool {
        false
    }

    /// 有界等待弹出一条。管道交付是异步的（读线程从 OS 缓冲搬运），
    /// send 之后立即可读是 loopback 才有的语义；两进程 pump 与测试
    /// 一律走本方法。
    fn recv_wait(&mut self, timeout_ms: u32) -> Option<Result<ProtocolMsg, CodecError>> {
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            if let Some(loaded) = self.try_recv() {
                return Some(loaded);
            }
            if self.is_eof() || std::time::Instant::now() >= deadline {
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
    }
}

// ---------------------------------------------------------------------------
// 分帧
// ---------------------------------------------------------------------------

/// u32 LE 长度前缀 + 载荷（信封字节）。
pub fn frame_bytes(envelope: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(4 + envelope.len());
    out.extend_from_slice(&(envelope.len() as u32).to_le_bytes());
    out.extend_from_slice(envelope);
    out
}

/// 从流累积缓冲切出全部完整帧（残帧保留在 buf 头部）。
fn drain_frames(buf: &mut Vec<u8>, out: &mut VecDeque<Vec<u8>>) {
    loop {
        if buf.len() < 4 {
            return;
        }
        let len = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]) as usize;
        if buf.len() < 4 + len {
            return;
        }
        let frame: Vec<u8> = buf[4..4 + len].to_vec();
        buf.drain(..4 + len);
        out.push_back(frame);
    }
}

// ---------------------------------------------------------------------------
// loopback 实现（Stage 1 语义原样并入 trait）
// ---------------------------------------------------------------------------

impl Transport for LoopbackEnd {
    fn send(&mut self, msg: &ProtocolMsg) -> Result<(), TransportError> {
        LoopbackEnd::send(self, msg);
        Ok(())
    }

    fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>> {
        LoopbackEnd::try_recv(self)
    }

    fn pending(&self) -> usize {
        LoopbackEnd::pending(self)
    }
}

// ---------------------------------------------------------------------------
// Windows 命名管道（tokio named_pipe + 常驻多线程 runtime）
// ---------------------------------------------------------------------------

#[cfg(windows)]
#[cfg(windows)]
mod pipe {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeServer, ServerOptions};
    use tokio::sync::oneshot;

    use crate::ui::desktop_protocol::codec::CodecError;
    use crate::ui::desktop_protocol::message::ProtocolMsg;
    use crate::ui::desktop_protocol::transport::{
        drain_frames, frame_bytes, Transport, TransportError,
    };

    /// 常驻 runtime（每端一个；worker 持续驱动 reactor，读线程 block_on
    /// 与孵化连接共用）。
    fn make_rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_io()
            .build()
            .expect("tokio runtime")
    }

    /// 服务端监听实例。
    pub struct PendingServer {
        server: NamedPipeServer,
        rt: std::sync::Arc<tokio::runtime::Runtime>,
        addr: String,
    }

    /// 创建命名管道服务端实例（双工、字节模式；broker 多实例 = 每次
    /// accept 后再 listen 一个）。
    pub fn listen(name: &str) -> Result<PendingServer, TransportError> {
        let rt = std::sync::Arc::new(make_rt());
        let server = rt
            .block_on(async { ServerOptions::new().create(addr_of(name)) })
            .map_err(|e| TransportError::Io(e.to_string()))?;
        Ok(PendingServer { server, rt, addr: addr_of(name) })
    }

    impl PendingServer {
        /// 等待客户端连入（孵化路径一次性），随后 split 出读写两半。
        pub fn wait_connect(mut self) -> Result<PipeEnd<NamedPipeServer>, TransportError> {
            let server = &mut self.server;
            self.rt
                .block_on(async { server.connect().await })
                .map_err(|e| TransportError::Io(e.to_string()))?;
            Ok(PipeEnd::spawn(self.rt, self.server))
        }

        pub fn addr(&self) -> &str {
            &self.addr
        }
    }

    /// 客户端连入（实例忙/未就绪时短暂重试直至超时）。
    pub fn connect(
        name: &str,
        timeout_ms: u32,
    ) -> Result<PipeEnd<tokio::net::windows::named_pipe::NamedPipeClient>, TransportError> {
        let rt = std::sync::Arc::new(make_rt());
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            match rt.block_on(async { ClientOptions::new().open(addr_of(name)) }) {
                Ok(c) => return Ok(PipeEnd::spawn(rt, c)),
                Err(e) if e.raw_os_error() == Some(231) => {
                    // ERROR_PIPE_BUSY：等一会儿重试。
                    if std::time::Instant::now() >= deadline {
                        return Err(TransportError::Io(format!("connect {name}: {e}")));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(e) => return Err(TransportError::Io(format!("connect {name}: {e}"))),
            }
        }
    }

    fn addr_of(name: &str) -> String {
        format!(r"\\.\pipe\{name}")
    }

    struct Inbox {
        frames: VecDeque<Vec<u8>>,
        acc: Vec<u8>,
        eof: bool,
    }

    /// 命名管道通道端：`tokio::io::split` 读写两半——读线程
    /// `select!{read, shutdown}`，写侧 block_on 阻塞写。EOF = 对端关闭。
    pub struct PipeEnd<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> {
        inbox: Arc<Mutex<Inbox>>,
        shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
        writer: Arc<Mutex<tokio::io::WriteHalf<T>>>,
        reader: Option<JoinHandle<()>>,
        rt: std::sync::Arc<tokio::runtime::Runtime>,
    }

    impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> PipeEnd<T> {
        fn spawn(rt: std::sync::Arc<tokio::runtime::Runtime>, pipe: T) -> Self {
            let rt_reader = std::sync::Arc::clone(&rt);
            let (read_half, write_half) = tokio::io::split(pipe);
            let inbox = Arc::new(Mutex::new(Inbox {
                frames: VecDeque::new(),
                acc: Vec::new(),
                eof: false,
            }));
            let inbox_reader = Arc::clone(&inbox);
            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            let reader = std::thread::spawn(move || {
                let rt = rt_reader;
                let mut half = read_half;
                let mut buf = [0u8; 8192];
                let mut shutdown_rx = shutdown_rx;
                rt.block_on(async {
                    loop {
                        tokio::select! {
                            res = half.read(&mut buf) => match res {
                                Ok(0) => break,
                                Ok(n) => {
                                    let mut inbox = inbox_reader.lock().unwrap();
                                    let Inbox { acc, frames, .. } = &mut *inbox;
                                    acc.extend_from_slice(&buf[..n]);
                                    drain_frames(acc, frames);
                                }
                                Err(_) => break, // 管道破裂 = 对端关闭
                            },
                            _ = &mut shutdown_rx => break,
                        }
                    }
                });
                inbox_reader.lock().unwrap().eof = true;
            });
            Self {
                inbox,
                shutdown_tx: Mutex::new(Some(shutdown_tx)),
                writer: Arc::new(Mutex::new(write_half)),
                reader: Some(reader),
                rt,
            }
        }

        /// 原始字节直写（分帧破坏注入 / 测试残帧用）。
        pub fn write_raw(&self, bytes: &[u8]) -> Result<(), TransportError> {
            let mut w = self.writer.lock().unwrap();
            self.rt
                .block_on(async { w.write_all(bytes).await })
                .map_err(|e| TransportError::Io(e.to_string()))
        }
    }

    impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Transport for PipeEnd<T> {
        fn send(&mut self, msg: &ProtocolMsg) -> Result<(), TransportError> {
            let framed = frame_bytes(&msg.encode());
            let mut w = self.writer.lock().unwrap();
            self.rt
                .block_on(async { w.write_all(&framed).await })
                .map_err(|e| TransportError::Io(e.to_string()))
        }

        fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>> {
            let mut inbox = self.inbox.lock().unwrap();
            inbox.frames.pop_front().map(|b| ProtocolMsg::decode(&b))
        }

        fn pending(&self) -> usize {
            self.inbox.lock().unwrap().frames.len()
        }

        fn is_eof(&self) -> bool {
            self.inbox.lock().unwrap().eof
        }
    }

    impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Drop for PipeEnd<T> {
        fn drop(&mut self) {
            // shutdown oneshot → select! 干净取消挂起读（cancel-safe）→
            // 读线程退出；写半随后关闭 → 对端读得破裂（EOF 传播）。
            if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            if let Some(handle) = self.reader.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(windows)]
pub use pipe::{connect, listen, PendingServer, PipeEnd};

/// 管道地址（与 autovm_daemon::pipe_addr 同款；此处单源避免反向依赖）。
#[allow(dead_code)]
pub fn pipe_addr(name: &str) -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\{name}")
    }
    #[cfg(not(windows))]
    {
        let _ = name;
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::message::{ControlMsg, HandshakeMsg};

    /// 每测试唯一管道名（防泄漏实例/并行运行串扰）。
    fn test_name(tag: &str) -> String {
        format!("autodesk-test-386-{tag}-{}", std::process::id())
    }

    fn fifo_check<A: Transport, B: Transport>(a: &mut A, b: &mut B) {
        for wid in 1..=3u64 {
            a.send(&ProtocolMsg::Control(ControlMsg::Close { wid })).unwrap();
        }
        for wid in 1..=3u64 {
            assert_eq!(
                b.recv_wait(2000).unwrap().unwrap(),
                ProtocolMsg::Control(ControlMsg::Close { wid })
            );
        }
        assert!(b.recv_wait(50).is_none(), "空管道 None");
    }

    #[test]
    fn loopback_via_transport_trait() {
        let (mut a, mut b) = super::super::loopback::loopback_pair();
        fifo_check(&mut a, &mut b);
        // 反向同样可达。
        Transport::send(&mut b, &ProtocolMsg::Handshake(HandshakeMsg::Ready)).unwrap();
        assert_eq!(
            a.recv_wait(50).unwrap().unwrap(),
            ProtocolMsg::Handshake(HandshakeMsg::Ready)
        );
    }

    #[cfg(windows)]
    #[test]
    fn named_pipe_pair_round_trip() {
        let name = test_name("roundtrip");
        let listener = listen(&name).expect("listen");
        let mut client = connect(&name, 2000).expect("client connect");
        let mut server = listener.wait_connect().expect("server connect");

        // app → host：FIFO + 完整信封解码。
        fifo_check(&mut client, &mut server);
        // host → app。
        server.send(&ProtocolMsg::Handshake(HandshakeMsg::Ready)).unwrap();
        assert_eq!(
            client.recv_wait(2000).unwrap().unwrap(),
            ProtocolMsg::Handshake(HandshakeMsg::Ready)
        );
        assert!(!client.is_eof() && !server.is_eof());
    }

    #[cfg(windows)]
    #[test]
    fn named_pipe_survives_partial_frames() {
        // 残帧：先写半个帧，再补全——接收端必须等完整帧才交付。
        let name = test_name("partial");
        let listener = listen(&name).expect("listen");
        let client = connect(&name, 2000).expect("client connect");
        let mut server = listener.wait_connect().expect("server connect");

        let framed = frame_bytes(&ProtocolMsg::Control(ControlMsg::Close { wid: 9 }).encode());
        let (head, tail) = framed.split_at(3);
        client.write_raw(head).unwrap();
        assert!(server.recv_wait(100).is_none(), "残帧不交付");
        client.write_raw(tail).unwrap();
        let msg = server
            .recv_wait(2000)
            .expect("补全后应可达")
            .unwrap();
        assert_eq!(msg, ProtocolMsg::Control(ControlMsg::Close { wid: 9 }));
    }

    #[cfg(windows)]
    #[test]
    fn named_pipe_eof_after_peer_drop() {
        let name = test_name("eof");
        let listener = listen(&name).expect("listen");
        let client = connect(&name, 2000).expect("client connect");
        let mut server = listener.wait_connect().expect("server connect");
        drop(client);
        // 对端关闭后：reactor 唤醒读线程 → EOF 标志（弹性语义"等待
        // 重连"的输入）。
        let mut waited = 0;
        while !server.is_eof() && waited < 2000 {
            let _ = server.try_recv();
            waited += 1;
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        assert!(server.is_eof(), "对端 drop 后应检测到 EOF");
    }

    #[test]
    fn drain_frames_splits_and_keeps_residual() {
        let f1 = frame_bytes(&[1, 2, 3]);
        let f2 = frame_bytes(&[9]);
        let mut buf: Vec<u8> = Vec::new();
        buf.extend_from_slice(&f1);
        buf.extend_from_slice(&f2[..2]); // 残帧
        let mut out = VecDeque::new();
        drain_frames(&mut buf, &mut out);
        assert_eq!(out.len(), 1);
        assert_eq!(out.pop_front().unwrap(), vec![1, 2, 3]);
        assert_eq!(buf, f2[..2].to_vec(), "残帧留在缓冲");
        out.clear();
        buf.extend_from_slice(&f2[2..]);
        drain_frames(&mut buf, &mut out);
        assert_eq!(out.pop_front().unwrap(), vec![9]);
    }
}
