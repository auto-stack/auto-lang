// Plan 386 Stage 1 —— 桌面协议 wire format v1（二进制信封 + LE 原语）。
//
// 信封布局（小端）：
// ```text
// [0..4)   magic  "APDL"
// [4..6)   u16    PROTOCOL_VERSION
// [6]      u8     channel（1..=5，见 [`Channel`]）
// [7]      u8     保留（0）
// [8..12)  u32    payload 长度
// [12..)   bytes  payload（各通道消息自带 encode/decode）
// ```
// 零新依赖；文本编解码先例 = `session::DesktopCommand`（ControlMsg 的
// DesktopBus 载荷沿用其记录格式，见 `message`）。

/// 信封魔数（"Auto Protocol Desktop Loopback/Link"——Stage 2 命名管道同头）。
pub const MAGIC: [u8; 4] = *b"APDL";

/// 信封固定头长度（magic + version + channel + reserved + len）。
pub const HEADER_LEN: usize = 12;

/// 五通道（Design 25 §7 表的线序编号）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Channel {
    Handshake = 1,
    Frame = 2,
    Input = 3,
    Control = 4,
    Observe = 5,
}

impl Channel {
    pub fn from_u8(v: u8) -> Result<Self, CodecError> {
        match v {
            1 => Ok(Self::Handshake),
            2 => Ok(Self::Frame),
            3 => Ok(Self::Input),
            4 => Ok(Self::Control),
            5 => Ok(Self::Observe),
            other => Err(CodecError::UnknownChannel(other)),
        }
    }
}

/// 编解码失败形态（版本不符拒收是版本化协议的硬约束）。
#[derive(Debug, Clone, PartialEq)]
pub enum CodecError {
    /// 字节数不足以读出期待的字段。
    TooShort,
    /// 魔数不是 "APDL"。
    BadMagic,
    /// 版本不一致（携带收到的版本号）。
    UnsupportedVersion(u16),
    /// 未知通道号。
    UnknownChannel(u8),
    /// 未知消息 tag。
    UnknownTag(u8),
    /// 字符串载荷不是合法 UTF-8。
    BadUtf8,
    /// 消息解码后仍有剩余字节（携带剩余数——结构漂移的哨兵）。
    TrailingBytes(usize),
}

// ---------------------------------------------------------------------------
// LE 原语写出
// ---------------------------------------------------------------------------

pub fn put_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

pub fn put_u16(out: &mut Vec<u8>, v: u16) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_u64(out: &mut Vec<u8>, v: u64) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_f32(out: &mut Vec<u8>, v: f32) {
    out.extend_from_slice(&v.to_le_bytes());
}

pub fn put_f64(out: &mut Vec<u8>, v: f64) {
    out.extend_from_slice(&v.to_le_bytes());
}

/// bool = 1 字节（0/1；解码时非 0/1 视为 1 的宽容语义不采用——只认 0/1）。
pub fn put_bool(out: &mut Vec<u8>, v: bool) {
    out.push(v as u8);
}

/// String = u32 长度前缀 + UTF-8 字节。
pub fn put_string(out: &mut Vec<u8>, v: &str) {
    put_u32(out, v.len() as u32);
    out.extend_from_slice(v.as_bytes());
}

/// 字节串 = u32 长度前缀 + 原始字节。
pub fn put_bytes(out: &mut Vec<u8>, v: &[u8]) {
    put_u32(out, v.len() as u32);
    out.extend_from_slice(v);
}

// ---------------------------------------------------------------------------
// LE 原语读取（游标式；越界统一 `CodecError::TooShort`）
// ---------------------------------------------------------------------------

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], CodecError> {
        if self.pos + n > self.data.len() {
            return Err(CodecError::TooShort);
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    /// 消息尾部断言：解码结束必须恰好耗尽载荷。
    pub fn finish(&self) -> Result<(), CodecError> {
        if self.pos != self.data.len() {
            return Err(CodecError::TrailingBytes(self.data.len() - self.pos));
        }
        Ok(())
    }

    pub fn u8(&mut self) -> Result<u8, CodecError> {
        Ok(self.take(1)?[0])
    }

    pub fn u16(&mut self) -> Result<u16, CodecError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn u32(&mut self) -> Result<u32, CodecError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Result<u64, CodecError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn f32(&mut self) -> Result<f32, CodecError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn f64(&mut self) -> Result<f64, CodecError> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn bool(&mut self) -> Result<bool, CodecError> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(CodecError::UnknownTag(u8::MAX)),
        }
    }

    pub fn string(&mut self) -> Result<String, CodecError> {
        let len = self.u32()? as usize;
        let bytes = self.take(len)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| CodecError::BadUtf8)
    }

    pub fn bytes(&mut self) -> Result<Vec<u8>, CodecError> {
        let len = self.u32()? as usize;
        Ok(self.take(len)?.to_vec())
    }
}

// ---------------------------------------------------------------------------
// 信封
// ---------------------------------------------------------------------------

/// 信封编码：payload 由调用方按所属通道先行编码。
pub fn encode_envelope(version: u16, channel: Channel, payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(HEADER_LEN + payload.len());
    out.extend_from_slice(&MAGIC);
    put_u16(&mut out, version);
    put_u8(&mut out, channel as u8);
    put_u8(&mut out, 0);
    put_u32(&mut out, payload.len() as u32);
    out.extend_from_slice(payload);
    out
}

/// 信封解码：校验魔数/版本/通道/长度自洽，返回 (channel, version, payload)。
pub fn decode_envelope(bytes: &[u8]) -> Result<(Channel, u16, &[u8]), CodecError> {
    if bytes.len() < HEADER_LEN {
        return Err(CodecError::TooShort);
    }
    if bytes[0..4] != MAGIC {
        return Err(CodecError::BadMagic);
    }
    let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    let channel = Channel::from_u8(bytes[6])?;
    let len = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let payload = bytes
        .get(HEADER_LEN..HEADER_LEN + len)
        .ok_or(CodecError::TooShort)?;
    Ok((channel, version, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primitive_round_trip() {
        let mut buf = Vec::new();
        put_u8(&mut buf, 7);
        put_u16(&mut buf, 0xBEEF);
        put_u32(&mut buf, 0xDEAD_BEEF);
        put_u64(&mut buf, u64::MAX);
        put_f32(&mut buf, 1.5);
        put_f64(&mut buf, -2.25);
        put_bool(&mut buf, true);
        put_bool(&mut buf, false);
        put_string(&mut buf, "计数/IME ✓");
        put_bytes(&mut buf, vec![1, 2, 3].as_slice());
        let mut r = Reader::new(&buf);
        assert_eq!(r.u8().unwrap(), 7);
        assert_eq!(r.u16().unwrap(), 0xBEEF);
        assert_eq!(r.u32().unwrap(), 0xDEAD_BEEF);
        assert_eq!(r.u64().unwrap(), u64::MAX);
        assert_eq!(r.f32().unwrap(), 1.5);
        assert_eq!(r.f64().unwrap(), -2.25);
        assert!(r.bool().unwrap());
        assert!(!r.bool().unwrap());
        assert_eq!(r.string().unwrap(), "计数/IME ✓");
        assert_eq!(r.bytes().unwrap(), vec![1, 2, 3]);
        r.finish().unwrap();
    }

    #[test]
    fn reader_rejects_short_and_trailing() {
        // 1 字节读 u16 → 越界。
        let mut r = Reader::new(&[7]);
        assert_eq!(r.u16(), Err(CodecError::TooShort));
        let mut r = Reader::new(&[5, 0, 0, 0, b'x']);
        assert_eq!(r.string(), Err(CodecError::TooShort));
        // 尾部剩余 1 字节 → TrailingBytes(1)。
        let mut r = Reader::new(&[2, 0, 0, 0, b'a', b'b', b'X']);
        assert_eq!(r.string().unwrap(), "ab");
        assert_eq!(r.finish(), Err(CodecError::TrailingBytes(1)));
    }

    #[test]
    fn bool_rejects_non_01() {
        let mut r = Reader::new(&[2]);
        assert!(matches!(r.bool(), Err(CodecError::UnknownTag(_))));
    }

    #[test]
    fn envelope_round_trip_and_golden() {
        // Close{wid:7} 的信封 golden：APDL + v1 + channel 4 + 保留 0 +
        // len 9 + tag 1(Close) + u64 7。信封格式冻结的锚点。
        let payload = {
            let mut p = Vec::new();
            put_u8(&mut p, 1);
            put_u64(&mut p, 7);
            p
        };
        let bytes = encode_envelope(1, Channel::Control, &payload);
        let expect: Vec<u8> = [
            b'A', b'P', b'D', b'L', 0x01, 0x00, 0x04, 0x00, 0x09, 0x00, 0x00, 0x00, 0x01, 0x07,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ]
        .to_vec();
        assert_eq!(bytes, expect);
        let (ch, ver, got) = decode_envelope(&bytes).unwrap();
        assert_eq!(ch, Channel::Control);
        assert_eq!(ver, 1);
        assert_eq!(got, payload.as_slice());
    }

    #[test]
    fn envelope_rejects_bad_magic_version_channel() {
        let mut b = encode_envelope(1, Channel::Frame, &[]);
        b[0] = b'X';
        assert_eq!(decode_envelope(&b), Err(CodecError::BadMagic));

        // 信封层只携带版本号（拒收点在 ProtocolMsg::decode，message 层测试）。
        let b = encode_envelope(2, Channel::Frame, &[]);
        assert_eq!(decode_envelope(&b), Ok((Channel::Frame, 2, &[][..])));

        let mut b = encode_envelope(1, Channel::Frame, &[]);
        b[6] = 99;
        assert_eq!(decode_envelope(&b), Err(CodecError::UnknownChannel(99)));

        assert_eq!(decode_envelope(&[1, 2, 3]), Err(CodecError::TooShort));

        let b = encode_envelope(1, Channel::Frame, &[1, 2]);
        let mut truncated = b.clone();
        truncated.pop();
        assert_eq!(decode_envelope(&truncated), Err(CodecError::TooShort));
    }
}
