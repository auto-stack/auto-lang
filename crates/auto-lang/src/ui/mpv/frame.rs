//! 帧目标缓冲：把 mpv SW renderer 对**内存对齐**的硬要求编码进类型。
//!
//! `include/mpv/render.h:393-404` 对 `MPV_RENDER_PARAM_SW_POINTER` /
//! `SW_STRIDE` 的规定：
//!
//! > Both stride and pointer value should be a multiple of 64 to facilitate fast
//! > SIMD operation. Lower alignment might trigger slower code paths, and in the
//! > worst case, will copy the entire target frame.
//!
//! 这一条很容易被忽略，因为 **`Vec<u8>` 的自然对齐只有 1**（实测在本机是 16），
//! 拿 `vec.as_mut_ptr()` 直接交给 mpv 就会静默掉进慢路径——T-15 的 spike 里
//! 已经踩过一次，所以 T-16 把它变成类型不变量：想拿到 [`SwTarget`]，只能经由
//! [`FrameBuffer`]（或显式 `unsafe`）。

/// mpv 要求的对齐（render.h:393-404）。
pub const REQUIRED_ALIGN: usize = 64;

/// 一次 SW 渲染的目标缓冲。
///
/// T-17 会把 `ptr` 指向 wgpu 的**持久映射 staging buffer**（而不是 CPU 缓冲），
/// 从而让 mpv 直接写进 GPU 可见内存、省掉一次中间拷贝。
pub struct SwTarget {
    pub ptr: *mut u8,
    pub width: u32,
    pub height: u32,
    pub stride: usize,
}

impl SwTarget {
    /// 包一个已按 [`REQUIRED_ALIGN`] 对齐的缓冲。
    ///
    /// # Safety
    /// `ptr` 必须指向至少 `stride * height` 字节的**可写**内存，并在整个
    /// [`super::engine::MpvEngine::render_sw_frame`] 调用期间保持有效；
    /// 且 `ptr`/`stride` 都应是 [`REQUIRED_ALIGN`] 的倍数，否则可能掉进慢路径。
    /// 常规用法是经 [`FrameBuffer::as_target`]，那里这些条件由类型保证。
    pub unsafe fn new(ptr: *mut u8, width: u32, height: u32, stride: usize) -> Self {
        Self {
            ptr,
            width,
            height,
            stride,
        }
    }
}

/// 一帧 RGBA（4 字节/像素）的 CPU 侧目标缓冲。
///
/// 缓冲**超分配** `REQUIRED_ALIGN` 字节后从对齐点切分，因此
/// `as_target()` 给出的一定满足 mpv 的 64 字节对齐要求。
pub struct FrameBuffer {
    storage: Vec<u8>,
    offset: usize,
    width: u32,
    height: u32,
    stride: usize,
}

impl FrameBuffer {
    /// 按目标尺寸分配；`stride` 向上取到 64 的倍数。
    pub fn new(width: u32, height: u32) -> Self {
        let stride = Self::stride_for(width);
        let needed = stride * height as usize;
        // 超分配，保证一定能找到对齐起点。
        let storage = vec![0u8; needed + REQUIRED_ALIGN];
        let base = storage.as_ptr() as usize;
        let offset = (REQUIRED_ALIGN - (base % REQUIRED_ALIGN)) % REQUIRED_ALIGN;
        Self {
            storage,
            offset,
            width,
            height,
            stride,
        }
    }

    /// 给定宽度下应使用的 stride（4 字节/像素，向上取到 [`REQUIRED_ALIGN`] 的倍数）。
    pub fn stride_for(width: u32) -> usize {
        let raw = width as usize * 4;
        raw.div_ceil(REQUIRED_ALIGN) * REQUIRED_ALIGN
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    /// 基址是否满足 mpv 的对齐要求（应当恒为 `true`；测试用它自证）。
    pub fn base_is_aligned(&self) -> bool {
        self.as_slice().as_ptr() as usize % REQUIRED_ALIGN == 0
    }

    pub fn as_slice(&self) -> &[u8] {
        let start = self.offset;
        &self.storage[start..start + self.stride * self.height as usize]
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        let start = self.offset;
        let len = self.stride * self.height as usize;
        &mut self.storage[start..start + len]
    }

    /// 交给 `MpvEngine::render_sw_frame` 的目标描述子。
    ///
    /// 这里**不需要** `unsafe`：指针来自本结构体持有的、长度足够的切片，
    /// 且对齐已由构造保证——这正是把 unsafe 收敛到构造处的好处。
    pub fn as_target(&mut self) -> SwTarget {
        let ptr = self.as_mut_slice().as_mut_ptr();
        // SAFETY: ptr 指向本结构体持有的 stride*height 字节可写内存，
        // 在 &mut self 的借用期内有效；对齐由 new() 保证。
        unsafe { SwTarget::new(ptr, self.width, self.height, self.stride) }
    }

    /// 画面是否已被写入过内容（测试用：区分「渲染成功」与「渲染成一片黑」）。
    pub fn has_content(&self) -> bool {
        self.as_slice().iter().any(|&b| b != 0)
    }
}

impl std::fmt::Debug for FrameBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrameBuffer")
            .field("size", &format_args!("{}x{}", self.width, self.height))
            .field("stride", &self.stride)
            .field("base_mod_64", &(self.as_slice().as_ptr() as usize % REQUIRED_ALIGN))
            .finish()
    }
}
