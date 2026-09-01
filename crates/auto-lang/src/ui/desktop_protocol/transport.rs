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
    /// 载荷解码失败（透传 codec 语义）。
    Codec(CodecError),
}

impl From<CodecError> for TransportError {
    fn from(e: CodecError) -> Self {
        Self::Codec(e)
    }
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

    /// 原始字节直写（线级破坏注入 / 测试残帧用；默认不支持）。
    fn write_raw(&self, _bytes: &[u8]) -> Result<(), TransportError> {
        Err(TransportError::Io("write_raw unsupported".into()))
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

impl Transport for Box<dyn Transport + Send> {
    fn send(&mut self, msg: &ProtocolMsg) -> Result<(), TransportError> {
        (**self).send(msg)
    }
    fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>> {
        (**self).try_recv()
    }
    fn pending(&self) -> usize {
        (**self).pending()
    }
    fn is_eof(&self) -> bool {
        (**self).is_eof()
    }
    fn recv_wait(&mut self, timeout_ms: u32) -> Option<Result<ProtocolMsg, CodecError>> {
        (**self).recv_wait(timeout_ms)
    }
    fn write_raw(&self, bytes: &[u8]) -> Result<(), TransportError> {
        (**self).write_raw(bytes)
    }
}

#[cfg(windows)]
#[cfg(windows)]
mod pipe {
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
    use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions};
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
        pub fn wait_connect(mut self) -> Result<Box<dyn Transport + Send>, TransportError> {
            let server = &mut self.server;
            self.rt
                .block_on(async { server.connect().await })
                .map_err(|e| TransportError::Io(e.to_string()))?;
            Ok(PipeEnd::spawn_boxed(self.rt, self.server))
        }

        pub fn addr(&self) -> &str {
            &self.addr
        }
    }

    /// 客户端连入（实例忙/未就绪时短暂重试直至超时）。
    pub fn connect(
        name: &str,
        timeout_ms: u32,
    ) -> Result<Box<dyn Transport + Send>, TransportError> {
        let rt = std::sync::Arc::new(make_rt());
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            match rt.block_on(async { ClientOptions::new().open(addr_of(name)) }) {
                Ok(c) => return Ok(PipeEnd::spawn_boxed(rt, c)),
                Err(e) if matches!(e.raw_os_error(), Some(231) | Some(2)) => {
                    // 231 = ERROR_PIPE_BUSY；2 = ERROR_FILE_NOT_FOUND
                    // （serve 侧 listen 未就绪的启动竞态）——等一会儿重试。
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
        /// 类型擦除构造（broker/request 返回值统一为 Box<dyn Transport + Send>）。
        pub fn spawn_boxed(
            rt: std::sync::Arc<tokio::runtime::Runtime>,
            pipe: T,
        ) -> Box<dyn Transport + Send> {
            Box::new(Self::spawn(rt, pipe))
        }

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

    }

    impl<T: AsyncRead + AsyncWrite + Unpin + Send + 'static> Transport for PipeEnd<T> {
        fn write_raw(&self, bytes: &[u8]) -> Result<(), TransportError> {
            let mut w = self.writer.lock().unwrap();
            self.rt
                .block_on(async { w.write_all(bytes).await })
                .map_err(|e| TransportError::Io(e.to_string()))
        }

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

/// 非 Windows 占位（Plan 509 Linux 编译缺口修补）：接口形状与 pipe 模
/// 块一致，调用即返回错误——真 Linux 传输（UDS）随 Smithay 宿主线生长
/// （本文件头注既有预留；Stage 2 立项）。同进程测试走 loopback。
#[cfg(not(windows))]
mod pipe_stub {
    use super::TransportError;

    const UNAVAILABLE: &str =
        "named-pipe transport is Windows-only (Linux UDS transport: smithay host line, Stage 2+)";

    pub struct PendingServer;

    pub fn listen(_name: &str) -> Result<PendingServer, TransportError> {
        Err(TransportError::Io(UNAVAILABLE.to_string()))
    }

    impl PendingServer {
        pub fn wait_connect(self) -> Result<Box<dyn super::Transport + Send>, TransportError> {
            Err(TransportError::Io(UNAVAILABLE.to_string()))
        }
    }

    pub fn connect(
        _name: &str,
        _timeout_ms: u32,
    ) -> Result<Box<dyn super::Transport + Send>, TransportError> {
        Err(TransportError::Io(UNAVAILABLE.to_string()))
    }
}

#[cfg(not(windows))]
pub use pipe_stub::{connect, listen, PendingServer};

// ---------------------------------------------------------------------------
// WebSocket（Plan 508 G3/G4 远程线）：tokio-tungstenite + 线程桥接同步
// trait——pipe 同族阻塞语义。
//
// 消息映射：**一条 WS Binary 消息 = 一个 codec 信封**（WS 自带消息
// 边界，无需 u32 长度前缀分帧——信封不变、传输层追加式，§1 演进
// 纪律）。Text 帧按未知载荷拒收（解码错误路径）；Close/读错 → EOF。
//
// token（v1）：回环 + HTTP 升级期 query 校验（`?token=<值>`——浏览器
// WebSocket API 不允许自定义头，query 为最简携带位）；静态 token 由
// 调用方传入（宿主 boot 读 `shell.remote.token`，缺省不监听=拒绝）。
// 跨网/TLS 另立计划（计划待澄清③边界）。
// ---------------------------------------------------------------------------

pub mod ws {
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;

    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio::sync::oneshot;
    use tokio_tungstenite::tungstenite::handshake::server::{Request, Response};
    use tokio_tungstenite::tungstenite::http::StatusCode;
    use tokio_tungstenite::tungstenite::Message;

    use super::{Transport, TransportError};
    use crate::ui::desktop_protocol::codec::CodecError;
    use crate::ui::desktop_protocol::message::ProtocolMsg;

    /// 常驻 runtime（pipe 同款：worker 持续驱动 reactor；WS accept 循环
    /// 的 select 节拍需要 timer，故 enable_all）。
    fn make_rt() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .expect("tokio runtime")
    }

    /// 读写半类型（`WebSocketStream::split` 产物；S = 底层 IO 流——
    /// 服务端 TcpStream / 客户端 MaybeTlsStream<TcpStream>）。
    type WsSink<S> = futures::stream::SplitSink<S, Message>;

    struct Inbox {
        envelopes: VecDeque<Vec<u8>>,
        eof: bool,
    }

    /// WS 通道端（PipeEnd 同族）：读线程 `select!{read, shutdown}` 泵
    /// Binary 消息 → inbox 信封；写侧 split sink（Mutex + block_on，无
    /// 跨 await 持锁）。
    pub struct WsEnd<S>
    where
        S: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin
            + Send
            + 'static,
    {
        inbox: Arc<Mutex<Inbox>>,
        shutdown_tx: Mutex<Option<oneshot::Sender<()>>>,
        writer: Arc<Mutex<WsSink<S>>>,
        reader: Option<JoinHandle<()>>,
        rt: Arc<tokio::runtime::Runtime>,
    }

    impl<S> WsEnd<S>
    where
        S: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin
            + Send
            + 'static,
    {
        /// 类型擦除构造（与 pipe 端统一为 `Box<dyn Transport + Send>`）。
        fn spawn_boxed(rt: Arc<tokio::runtime::Runtime>, stream: S) -> Box<dyn Transport + Send> {
            let (sink, stream) = stream.split();
            let inbox = Arc::new(Mutex::new(Inbox { envelopes: VecDeque::new(), eof: false }));
            let inbox_reader = Arc::clone(&inbox);
            let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
            let writer = Arc::new(Mutex::new(sink));
            let rt_reader = Arc::clone(&rt);
            let reader = std::thread::spawn(move || {
                let rt = rt_reader;
                let mut stream = stream;
                let mut shutdown_rx = shutdown_rx;
                rt.block_on(async {
                    loop {
                        tokio::select! {
                            res = stream.next() => match res {
                                Some(Ok(Message::Binary(data))) => {
                                    inbox_reader.lock().unwrap().envelopes.push_back(data.to_vec());
                                }
                                // Ping/Pong：tungstenite 读侧自动应答，此处吞掉。
                                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                                // Close / Text（未知载荷形态）/ 读错 = 对端关闭。
                                Some(Ok(_)) | None => break,
                                Some(Err(_)) => break,
                            },
                            _ = &mut shutdown_rx => break,
                        }
                    }
                });
                inbox_reader.lock().unwrap().eof = true;
            });
            Box::new(Self {
                inbox,
                shutdown_tx: Mutex::new(Some(shutdown_tx)),
                writer,
                reader: Some(reader),
                rt,
            })
        }
    }

    impl<S> Transport for WsEnd<S>
    where
        S: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin
            + Send
            + 'static,
    {
        fn send(&mut self, msg: &ProtocolMsg) -> Result<(), TransportError> {
            self.write_raw(&msg.encode())
        }

        /// 原始字节直写 = 一条 Binary 消息（信封形态直送；线级破坏注入同款）。
        fn write_raw(&self, bytes: &[u8]) -> Result<(), TransportError> {
            let mut w = self.writer.lock().unwrap();
            self.rt
                .block_on(async { w.send(Message::Binary(bytes.to_vec().into())).await })
                .map_err(|e| TransportError::Io(e.to_string()))
        }

        fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>> {
            let mut inbox = self.inbox.lock().unwrap();
            inbox.envelopes.pop_front().map(|b| ProtocolMsg::decode(&b))
        }

        fn pending(&self) -> usize {
            self.inbox.lock().unwrap().envelopes.len()
        }

        fn is_eof(&self) -> bool {
            self.inbox.lock().unwrap().eof
        }
    }

    impl<S> Drop for WsEnd<S>
    where
        S: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>>
            + futures::Sink<Message, Error = tokio_tungstenite::tungstenite::Error>
            + Unpin
            + Send
            + 'static,
    {
        fn drop(&mut self) {
            if let Some(tx) = self.shutdown_tx.lock().unwrap().take() {
                let _ = tx.send(());
            }
            if let Some(handle) = self.reader.take() {
                let _ = handle.join();
            }
        }
    }

    /// WS 服务端监听实例：accept 线程常驻（token 在 HTTP 升级期校验，
    /// 失败回 401），accepted 连接排队——属主线程 `try_accept` 消费。
    pub struct WsListener {
        accepted: Arc<Mutex<VecDeque<Box<dyn Transport + Send>>>>,
        stop: Arc<AtomicBool>,
        port: u16,
    }

    /// query 串中的 token 提取（`?token=<值>`；简单前缀匹配——v1 静态
    /// token 无编码形态）。
    fn query_token(uri: &str) -> Option<&str> {
        let query = uri.split_once('?')?.1;
        for pair in query.split('&') {
            if let Some(v) = pair.strip_prefix("token=") {
                return Some(v);
            }
        }
        None
    }

    impl WsListener {
        /// 绑定回环端口（port 0 = 系统分配，实际端口经 [`Self::port`]）。
        pub fn bind(port: u16, token: &str) -> Result<Self, TransportError> {
            let rt = Arc::new(make_rt());
            let listener = rt
                .block_on(async { TcpListener::bind(("127.0.0.1", port)).await })
                .map_err(|e| TransportError::Io(e.to_string()))?;
            let port = listener
                .local_addr()
                .map_err(|e| TransportError::Io(e.to_string()))?
                .port();
            let accepted: Arc<Mutex<VecDeque<Box<dyn Transport + Send>>>> =
                Arc::new(Mutex::new(VecDeque::new()));
            let stop = Arc::new(AtomicBool::new(false));
            let accepted_loop = Arc::clone(&accepted);
            let stop_loop = Arc::clone(&stop);
            let token = token.to_string();
            std::thread::Builder::new()
                .name("autodesk-ws-accept".into())
                .spawn(move || {
                    let rt = &rt;
                    let listener = listener;
                    while !stop_loop.load(Ordering::Relaxed) {
                        // 200ms 节拍 select：停机旗标可退出（bind 独占线程
                        // 不可 Join 属主——Drop 置位后至多一拍退出）。
                        let incoming = rt.block_on(async {
                            tokio::select! {
                                res = listener.accept() => Some(res),
                                _ = tokio::time::sleep(std::time::Duration::from_millis(200)) => None,
                            }
                        });
                        let (stream, _addr) = match incoming {
                            Some(Ok(v)) => v,
                            Some(Err(_)) => {
                                std::thread::sleep(std::time::Duration::from_millis(20));
                                continue;
                            }
                            None => continue,
                        };
                        let want = token.clone();
                        let token_check = move |req: &Request, resp: Response| {
                            let uri = req
                                .uri()
                                .path_and_query()
                                .map(|pq| pq.as_str().to_string())
                                .unwrap_or_default();
                            if query_token(&uri) == Some(want.as_str()) {
                                Ok(resp)
                            } else {
                                Err(http_reject_response())
                            }
                        };
                        let end = rt.block_on(async {
                            tokio_tungstenite::accept_hdr_async(stream, token_check).await
                        });
                        match end {
                            Ok(ws) => {
                                let boxed = WsEnd::spawn_boxed(Arc::clone(&rt), ws);
                                accepted_loop.lock().unwrap().push_back(boxed);
                            }
                            Err(_) => {} // 401/升级失败：连接已拒（测试断言点）
                        }
                    }
                })
                .map_err(|e| TransportError::Io(e.to_string()))?;
            Ok(Self { accepted, stop, port })
        }

        /// 实际监听端口（bind(0) 时为系统分配值）。
        pub fn port(&self) -> u16 {
            self.port
        }

        /// 已受理待消费连接数（宿主泵节流位）。
        pub fn accepted_len(&self) -> usize {
            self.accepted.lock().unwrap().len()
        }

        /// 客户端 URL（token query 直拼——测试/宿主提示共用）。
        pub fn url(&self, token: &str) -> String {
            format!("ws://127.0.0.1:{}/?token={token}", self.port)
        }

        /// 弹出一个已受理连接（属主线程轮询消费；无连接 None）。
        pub fn try_accept(&mut self) -> Option<Box<dyn Transport + Send>> {
            self.accepted.lock().unwrap().pop_front()
        }
    }

    impl Drop for WsListener {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Relaxed);
        }
    }

    /// 401 升级拒绝（token 不符；ErrorResponse = http::Response）。
    fn http_reject_response() -> tokio_tungstenite::tungstenite::http::Response<Option<String>> {
        tokio_tungstenite::tungstenite::http::Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(Some("token mismatch".into()))
            .expect("static 401 response")
    }

    /// 客户端连入（Rust 侧消费者 + 测试；token 经 URL query 携带）。
    pub fn connect(url: &str, timeout_ms: u32) -> Result<Box<dyn Transport + Send>, TransportError> {
        let rt = Arc::new(make_rt());
        let deadline =
            std::time::Instant::now() + std::time::Duration::from_millis(timeout_ms as u64);
        loop {
            match rt.block_on(async { tokio_tungstenite::connect_async(url).await }) {
                Ok((ws, _)) => return Ok(WsEnd::spawn_boxed(rt, ws)),
                Err(e) => {
                    // 升级拒绝（401 等）为终态错误：直返不重试（token 拒收
                    // 路径的判定依据）。
                    if matches!(e, tokio_tungstenite::tungstenite::Error::Http(_)) {
                        return Err(TransportError::Io(format!("ws upgrade rejected: {e}")));
                    }
                    if std::time::Instant::now() >= deadline {
                        return Err(TransportError::Io(format!("connect {url}: {e}")));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
            }
        }
    }
}

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

    // -----------------------------------------------------------------------
    // Plan 508 T1 —— WsTransport（loopback/pipe 同族单测）。
    // -----------------------------------------------------------------------

    mod ws_tests {
        use super::super::ws;
        use super::*;
        use crate::ui::desktop_protocol::message::{ControlMsg, HandshakeMsg};

        /// 等 listener 受理出一个服务端端点（accept 线程异步受理）。
        fn accept_once(listener: &mut ws::WsListener) -> Box<dyn Transport + Send> {
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
            loop {
                if let Some(end) = listener.try_accept() {
                    return end;
                }
                assert!(std::time::Instant::now() < deadline, "accept 超时");
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
        }

        #[test]
        fn ws_pair_round_trip() {
            let mut listener = ws::WsListener::bind(0, "t1-token").expect("bind");
            let mut client =
                ws::connect(&listener.url("t1-token"), 3000).expect("client connect");
            let mut server = accept_once(&mut listener);

            // client → server：FIFO + 完整信封解码。
            fifo_check(&mut client, &mut server);
            // server → client。
            server.send(&ProtocolMsg::Handshake(HandshakeMsg::Ready)).unwrap();
            assert_eq!(
                client.recv_wait(2000).unwrap().unwrap(),
                ProtocolMsg::Handshake(HandshakeMsg::Ready)
            );
            assert!(!client.is_eof() && !server.is_eof());
        }

        /// 信封不变纪律（golden bytes）：WS Binary 消息体 = codec 信封原样
        /// ——`write_raw(msg.encode())` 与 `send(msg)` 过线字节恒等，TS 侧
        /// 镜像解码的契约锚点。
        #[test]
        fn ws_binary_message_is_envelope() {
            let mut listener = ws::WsListener::bind(0, "t1-golden").expect("bind");
            let mut client =
                ws::connect(&listener.url("t1-golden"), 3000).expect("client connect");
            let mut server = accept_once(&mut listener);

            let msg = ProtocolMsg::Control(ControlMsg::Close { wid: 7 });
            let golden = msg.encode();
            client.write_raw(&golden).unwrap();
            assert_eq!(
                server.recv_wait(2000).unwrap().unwrap(),
                msg,
                "信封原样过线（无二次分帧）"
            );
        }

        #[test]
        fn ws_eof_after_peer_drop() {
            let mut listener = ws::WsListener::bind(0, "t1-eof").expect("bind");
            let client = ws::connect(&listener.url("t1-eof"), 3000).expect("client connect");
            let mut server = accept_once(&mut listener);
            drop(client);
            let mut waited = 0;
            while !server.is_eof() && waited < 3000 {
                let _ = server.try_recv();
                waited += 1;
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
            assert!(server.is_eof(), "对端 drop 后应检测到 EOF");
        }

        /// token 拒收路径：错 token 的升级被 401 拒绝（终态错误，不重试）。
        #[test]
        fn ws_token_rejected() {
            let mut listener = ws::WsListener::bind(0, "t1-right").expect("bind");
            let err = ws::connect(&listener.url("t1-wrong"), 3000).err().expect("拒收");
            match err {
                crate::ui::desktop_protocol::transport::TransportError::Io(msg) => {
                    assert!(msg.contains("rejected"), "升级拒绝终态错误: {msg}");
                }
                other => panic!("应为 Io(rejected): {other:?}"),
            }
            // 正确 token 仍可连（拒收不伤监听）。
            let _client = ws::connect(&listener.url("t1-right"), 3000).expect("正确 token 可连");
            let _server = accept_once(&mut listener);
        }
    }
}
