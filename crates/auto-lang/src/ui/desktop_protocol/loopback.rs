// Plan 386 Stage 1 —— loopback 传输层：同进程双向字节管道。
//
// "共享纹理" 的 Stage 1 模拟：`send` 把 [`ProtocolMsg`] 编码成字节推入
// 管道，`try_recv` 从对端管道弹出并解码——编解码真实过线（协议语义被
// 完整行使），只是搬运发生在内存里。Stage 2 换命名管道/共享内存时，
// 只替换本类型内部实现，`send`/`try_recv` 签名不变（状态机零改动）。

use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use super::codec::CodecError;
use super::message::ProtocolMsg;

type SharedPipe = Rc<RefCell<VecDeque<Vec<u8>>>>;

pub struct LoopbackEnd {
    /// 我方写、对端读。
    out: SharedPipe,
    /// 对端写、我方读。
    inn: SharedPipe,
}

impl LoopbackEnd {
    /// 编码并入队（无阻塞、无背压——内存管道语义）。
    pub fn send(&self, msg: &ProtocolMsg) {
        self.out.borrow_mut().push_back(msg.encode());
    }

    /// 原始字节入队（线级破坏注入 / Stage 2 transport 兼容测试用）。
    pub fn send_raw(&self, bytes: Vec<u8>) {
        self.out.borrow_mut().push_back(bytes);
    }

    /// 弹出一条并解码（FIFO——协议管道保序）；空管道返回 None。
    pub fn try_recv(&mut self) -> Option<Result<ProtocolMsg, CodecError>> {
        let bytes = self.inn.borrow_mut().pop_front()?;
        Some(ProtocolMsg::decode(&bytes))
    }

    /// 待收消息数（测试/观测用）。
    pub fn pending(&self) -> usize {
        self.inn.borrow().len()
    }
}

/// 建立一对相连的 loopback 端（返回 (app 侧, host 侧)）。
pub fn loopback_pair() -> (LoopbackEnd, LoopbackEnd) {
    let a2h: SharedPipe = Rc::new(RefCell::new(VecDeque::new()));
    let h2a: SharedPipe = Rc::new(RefCell::new(VecDeque::new()));
    let app = LoopbackEnd { out: Rc::clone(&a2h), inn: Rc::clone(&h2a) };
    let host = LoopbackEnd { out: h2a, inn: a2h };
    (app, host)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::message::{ControlMsg, HandshakeMsg};

    #[test]
    fn pair_delivers_encoded_bytes() {
        let (app, mut host) = loopback_pair();
        assert_eq!(host.pending(), 0);
        app.send(&ProtocolMsg::Control(ControlMsg::Close { wid: 7 }));
        assert_eq!(host.pending(), 1, "一条过线");
        let got = host.try_recv().unwrap().unwrap();
        assert_eq!(got, ProtocolMsg::Control(ControlMsg::Close { wid: 7 }));

        // host → app 方向同样可达（握手 Welcome 走这条线）。
        host.send(&ProtocolMsg::Handshake(HandshakeMsg::Ready));
        let mut app = app;
        assert_eq!(
            app.try_recv().unwrap().unwrap(),
            ProtocolMsg::Handshake(HandshakeMsg::Ready)
        );
        assert!(app.try_recv().is_none(), "空管道 None");
    }

    #[test]
    fn fifo_ordering_preserved() {
        let (app, mut host) = loopback_pair();
        app.send(&ProtocolMsg::Control(ControlMsg::Close { wid: 1 }));
        app.send(&ProtocolMsg::Control(ControlMsg::Close { wid: 2 }));
        app.send(&ProtocolMsg::Control(ControlMsg::Close { wid: 3 }));
        for wid in 1..=3 {
            assert_eq!(
                host.try_recv().unwrap().unwrap(),
                ProtocolMsg::Control(ControlMsg::Close { wid })
            );
        }
    }

    #[test]
    fn corrupted_bytes_surface_codec_error() {
        let (app, mut host) = loopback_pair();
        // 线上破坏魔数 → 对端解码报错（不 panic）。
        let mut bytes = ProtocolMsg::Control(ControlMsg::Close { wid: 9 }).encode();
        bytes[0] = b'X';
        app.send(&ProtocolMsg::Control(ControlMsg::Close { wid: 1 }));
        app.send_raw(bytes);
        assert_eq!(
            host.try_recv().unwrap().unwrap(),
            ProtocolMsg::Control(ControlMsg::Close { wid: 1 }),
            "先入先出：完好消息先达"
        );
        assert_eq!(host.try_recv().unwrap(), Err(CodecError::BadMagic));
    }
}
