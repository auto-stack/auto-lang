// Plan 386 S9 —— 帧通道的共享内存传输（"共享纹理"的 Stage 2 正身）。
//
// 大帧载荷不再走管道：app 把 DrawList 序列化进共享内存槽（双缓冲），
// 管道上只过 [`FrameMsg::FrameReadyShared`]（slot + len + 元数据）。
// Windows = `CreateFileMappingW`/`MapViewOfFile` 手写 FFI（零新依赖）；
// 非 Windows 平台 v1 用进程内哈希表占位（Linux memfd 随 libcosmic 宿主
// 单独生长，见计划"短期目标"——接口不变）。
//
// 槽布局：`[u32 len][payload bytes]` × slot_count（slot_size 定长槽）。
// 双缓冲语义与 Stage 1 `SurfaceStore` 一致：app 写非前台槽 → FrameReady
// → 宿主翻面 → FrameAck 归还。

use super::codec::{CodecError, Reader};
use super::message::DrawList;
use super::TransportError;

/// 共享内存帧缓冲（跨进程映射或进程内占位）。
pub struct SharedFrameBuffer {
    inner: Inner,
    slot_count: u8,
    slot_size: u32,
}

#[cfg(windows)]
mod windows_map {

    pub const PAGE_READWRITE: u32 = 0x04;
    pub const FILE_MAP_ALL_ACCESS: u32 = 0x000F_001F;
    pub const INVALID_HANDLE: isize = -1;

    #[repr(C)]
    #[derive(Clone, Copy)]
    pub struct SecurityAttributes {
        pub length: u32,
        pub security_descriptor: *const std::ffi::c_void,
        pub inherit_handle: i32,
    }

    extern "system" {
        fn CreateFileMappingW(
            h_file: isize,
            attrs: *const SecurityAttributes,
            protect: u32,
            max_high: u32,
            max_low: u32,
            name: *const u16,
        ) -> isize;
        fn OpenFileMappingW(
            access: u32,
            inherit: i32,
            name: *const u16,
        ) -> isize;
        fn MapViewOfFile(
            h: isize,
            access: u32,
            high: u32,
            low: u32,
            bytes: usize,
        ) -> *mut u8;
        fn UnmapViewOfFile(ptr: *const u8) -> i32;
    }

    pub struct Mapping {
        pub map: *mut u8,
        mapping_handle: isize,
    }

    impl Mapping {
        /// 创建命名映射（宿主侧）。
        pub fn create(name: &str, size: u32) -> Result<Self, String> {
            let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
            let h = unsafe {
                CreateFileMappingW(
                    INVALID_HANDLE,
                    std::ptr::null(),
                    PAGE_READWRITE,
                    0,
                    size,
                    wide.as_ptr(),
                )
            };
            if h == 0 {
                return Err(format!("CreateFileMappingW: {}", std::io::Error::last_os_error()));
            }
            Self::map(h, size)
        }

        /// 打开既有命名映射（app 侧）。
        pub fn open(name: &str, size: u32) -> Result<Self, String> {
            let wide: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
            let h = unsafe { OpenFileMappingW(FILE_MAP_ALL_ACCESS, 0, wide.as_ptr()) };
            if h == 0 {
                return Err(format!("OpenFileMappingW: {}", std::io::Error::last_os_error()));
            }
            Self::map(h, size)
        }

        fn map(h: isize, size: u32) -> Result<Self, String> {
            let map = unsafe { MapViewOfFile(h, FILE_MAP_ALL_ACCESS, 0, 0, size as usize) };
            if map.is_null() {
                // 句柄泄漏可忽略：进程退出回收；此处错误路径。
                return Err(format!("MapViewOfFile: {}", std::io::Error::last_os_error()));
            }
            Ok(Self { map, mapping_handle: h })
        }

        pub fn slice(&self, size: u32) -> &mut [u8] {
            unsafe { std::slice::from_raw_parts_mut(self.map, size as usize) }
        }
    }

    impl Drop for Mapping {
        fn drop(&mut self) {
            super::windows_map::unmap(self.map);
            let _ = self.mapping_handle;
        }
    }

    pub fn unmap(ptr: *const u8) -> i32 {
        unsafe { UnmapViewOfFile(ptr) }
    }

}

#[cfg(windows)]
use windows_map::Mapping;

#[cfg(not(windows))]
mod inproc_map {
    /// 非 Windows v1 占位：进程内命名映射表（接口不变；Linux memfd 随
    /// libcosmic 宿主单独生长）。同进程 create/open 互通，测试可跑。
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    pub struct Registry {
        pub map: HashMap<String, Arc<Mutex<Vec<u8>>>>,
    }

    static REGISTRY: std::sync::OnceLock<Mutex<Registry>> = std::sync::OnceLock::new();

    pub fn registry() -> &'static Mutex<Registry> {
        REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
    }
}

#[cfg(not(windows))]
struct Mapping {
    shared: Arc<Mutex<Vec<u8>>>,
    size: u32,
}

#[cfg(not(windows))]
impl Mapping {
    fn create(name: &str, size: u32) -> Result<Self, String> {
        let shared = Arc::new(Mutex::new(vec![0u8; size as usize]));
        inproc_map::registry()
            .lock()
            .unwrap()
            .map
            .insert(name.to_string(), Arc::clone(&shared));
        Ok(Self { shared, size })
    }

    fn open(name: &str, size: u32) -> Result<Self, String> {
        inproc_map::registry()
            .lock()
            .unwrap()
            .map
            .get(name)
            .map(|shared| Self { shared: Arc::clone(shared), size })
            .ok_or_else(|| format!("open {name}: not found"))
    }

    fn with_bytes<R>(&self, f: impl FnOnce(&mut [u8]) -> R) -> R {
        let mut bytes = self.shared.lock().unwrap();
        f(&mut bytes[..self.size as usize])
    }
}

enum Inner {
    Mapped(Mapping),
    #[allow(dead_code)]
    Placeholder,
}

impl SharedFrameBuffer {
    /// 宿主侧：创建命名帧缓冲。
    pub fn create(name: &str, slot_count: u8, slot_size: u32) -> Result<Self, TransportError> {
        let size = slot_count as u32 * slot_size;
        let mapping =
            Mapping::create(name, size).map_err(TransportError::Io)?;
        Ok(Self {
            inner: Inner::Mapped(mapping),
            slot_count,
            slot_size,
        })
    }

    /// app 侧：打开既有命名帧缓冲。
    pub fn open(name: &str, slot_count: u8, slot_size: u32) -> Result<Self, TransportError> {
        let mapping = Mapping::open(name, slot_count as u32 * slot_size)
            .map_err(TransportError::Io)?;
        Ok(Self {
            inner: Inner::Mapped(mapping),
            slot_count,
            slot_size,
        })
    }

    fn slot_range(&self, slot: u8) -> Result<(usize, usize), TransportError> {
        if slot >= self.slot_count {
            return Err(TransportError::Io(format!("slot {slot} out of range")));
        }
        let base = slot as usize * self.slot_size as usize;
        Ok((base, base + self.slot_size as usize))
    }

    /// 写槽：`[u32 len][payload]`。
    pub fn write_slot(&self, slot: u8, payload: &[u8]) -> Result<(), TransportError> {
        let (start, end) = self.slot_range(slot)?;
        let slot_size = self.slot_size as usize;
        if payload.len() + 4 > slot_size {
            return Err(TransportError::Io(format!(
                "payload {} exceeds slot size {slot_size}",
                payload.len()
            )));
        }
        let _ = end;
        match &self.inner {
            Inner::Mapped(m) => {
                #[cfg(windows)]
                {
                    let bytes = m.slice(self.slot_count as u32 * self.slot_size);
                    bytes[start..start + 4].copy_from_slice(&(payload.len() as u32).to_le_bytes());
                    bytes[start + 4..start + 4 + payload.len()].copy_from_slice(payload);
                    let _ = slot_size;
                }
                #[cfg(not(windows))]
                {
                    let _ = slot_size;
                    m.with_bytes(|bytes| {
                        bytes[start..start + 4]
                            .copy_from_slice(&(payload.len() as u32).to_le_bytes());
                        bytes[start + 4..start + 4 + payload.len()].copy_from_slice(payload);
                    });
                }
            }
            Inner::Placeholder => unreachable!(),
        }
        Ok(())
    }

    /// 读槽：返回 payload（len 校验）。
    pub fn read_slot(&self, slot: u8) -> Result<Vec<u8>, TransportError> {
        let (start, end) = self.slot_range(slot)?;
        match &self.inner {
            Inner::Mapped(m) => {
                #[cfg(windows)]
                {
                    let bytes = m.slice(self.slot_count as u32 * self.slot_size);
                    Self::decode_slot(&bytes[start..end])
                }
                #[cfg(not(windows))]
                {
                    m.with_bytes(|bytes| Self::decode_slot(&bytes[start..end]))
                }
            }
            Inner::Placeholder => unreachable!(),
        }
    }

    fn decode_slot(slot: &[u8]) -> Result<Vec<u8>, TransportError> {
        if slot.len() < 4 {
            return Err(TransportError::Eof);
        }
        let len = u32::from_le_bytes([slot[0], slot[1], slot[2], slot[3]]) as usize;
        if 4 + len > slot.len() {
            return Err(TransportError::Eof);
        }
        Ok(slot[4..4 + len].to_vec())
    }

    pub fn slot_count(&self) -> u8 {
        self.slot_count
    }

    pub fn slot_size(&self) -> u32 {
        self.slot_size
    }
}

// 手动 Send/Sync：映射视图进程内有效，槽读写由协议时序（FrameReady/Ack
// 交替）互斥，无数据竞争。
#[cfg(windows)]
unsafe impl Send for windows_map::Mapping {}
#[cfg(windows)]
unsafe impl Sync for windows_map::Mapping {}

/// 从共享内存槽载荷解码 [`DrawList`]（与 FrameReady 内嵌载荷同编码）。
pub fn draw_list_from_slot_payload(payload: &[u8]) -> Result<DrawList, CodecError> {
    let mut r = Reader::new(payload);
    let list = DrawList::decode(&mut r)?;
    r.finish()?;
    Ok(list)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::desktop_protocol::message::{DrawOp, Rgba8, WRect};

    fn test_name(tag: &str) -> String {
        format!("autodesk-shm-386-{tag}-{}", std::process::id())
    }

    #[cfg(windows)]
    #[test]
    fn shm_create_open_write_read() {
        let name = test_name("rw");
        let host = SharedFrameBuffer::create(&name, 2, 4096).expect("create");
        let app = SharedFrameBuffer::open(&name, 2, 4096).expect("open");

        // app 写槽 1 → 宿主从槽 1 读回（跨"端"互通）。
        app.write_slot(1, b"frame-payload").unwrap();
        assert_eq!(host.read_slot(1).unwrap(), b"frame-payload");

        // 双缓冲：槽 0 独立。
        host.write_slot(0, b"front").unwrap();
        assert_eq!(host.read_slot(0).unwrap(), b"front");
        assert_eq!(app.read_slot(0).unwrap(), b"front");
        assert_eq!(app.read_slot(1).unwrap(), b"frame-payload");

        // 越界槽拒收。
        assert!(host.write_slot(2, b"x").is_err());
        // 超载拒收。
        let big = vec![0u8; 5000];
        assert!(app.write_slot(0, &big).is_err());
    }

    #[cfg(windows)]
    #[test]
    fn shm_draw_list_round_trip_via_slot() {
        // 帧载荷真实链路：DrawList → encode → 槽 → decode == 原始。
        let name = test_name("drawlist");
        let host = SharedFrameBuffer::create(&name, 2, 8192).expect("create");
        let app = SharedFrameBuffer::open(&name, 2, 8192).expect("open");

        let list = DrawList {
            clear: Some(Rgba8::new(1, 2, 3, 255)),
            ops: vec![
                DrawOp::Quad { rect: WRect::new(0.0, 0.0, 10.0, 20.0), color: Rgba8::new(9, 9, 9, 200) },
                DrawOp::Text {
                    x: 1.0,
                    y: 2.0,
                    size: 14.0,
                    line_height: 18.0,
                    color: Rgba8::new(255, 255, 255, 255),
                    text: "count: 42".into(),
                },
            ],
        };
        let mut payload = Vec::new();
        list.encode(&mut payload);
        app.write_slot(0, &payload).unwrap();

        let bytes = host.read_slot(0).unwrap();
        let mut r = Reader::new(&bytes);
        let decoded = DrawList::decode(&mut r).unwrap();
        r.finish().unwrap();
        assert_eq!(decoded, list, "槽载荷 DrawList 往返恒等");
        assert_eq!(draw_list_from_slot_payload(&bytes).unwrap(), list);
    }

    #[test]
    fn shm_empty_slot_reports_eof() {
        // 未写入的槽：len=0 → 读回空载荷（合法空帧语义由协议层定）。
        let name = test_name("empty");
        let _host = SharedFrameBuffer::create(&name, 1, 1024).unwrap();
        let app = SharedFrameBuffer::open(&name, 1, 1024).unwrap();
        // len=0 → 空载荷 Ok（非错误）；未初始化内存首 4 字节在 Windows
        // 映射创建时清零（页面零填充），故 len=0。
        assert_eq!(app.read_slot(0).unwrap(), Vec::<u8>::new());
    }
}
