// Plan 485: native clipboard bridge — files (CF_HDROP) and images
// (CF_DIBV5 / registered "PNG"), Phase 2 of the native interop route
// (473). Complements the arboard text bridge (Plan 418, `clipboard.rs`).
//
// Layout mirrors `native_dock`: the pure codec helpers (DROPFILES blob,
// BITMAPV5HEADER ↔ RGBA) are plain Rust compiling and testing on every
// platform/feature tier (`cargo t clipboard_native` daily tier); the
// Win32 clipboard calls are double-gated `cfg(windows)` ×
// `feature = "native-clipboard"` (ui-iced implies it, same as native-dock).
//
// Degradation contract (vue/web remote端 & 非 Windows): the Win32 arms
// don't exist there and the VM shims (`vm/native.rs`) return the empty
// values — files_get → `[]`, files_set/image_set → `false`,
// image_get → `None` — so .at code needs no platform branches (G3).
//
// 剪贴板是进程全局资源：Win32 集成测试须 set→get 即时往返，且用进程内
// 互斥锁防同进程并行测试互相污染（nextest 每测试独立进程，锁为空转）。

// TempImage（G2 Record 元数据）跨平台编译——PathBuf 导入不随 Win32 桥
// 门控（Plan 509：Linux 编译缺口修补，Windows 行为零变化）。
use std::path::PathBuf;

// ── Win32 clipboard format IDs（稳定 ABI 常量，免引 Win32_System_Ole）──
#[cfg(all(windows, feature = "native-clipboard"))]
const CF_DIB: u32 = 8;
#[cfg(all(windows, feature = "native-clipboard"))]
const CF_DIBV5: u32 = 17;
#[cfg(all(windows, feature = "native-clipboard"))]
const CF_HDROP: u32 = 15;

/// DROPFILES header size: pFiles(u32) + POINT(2×i32) + fNC(i32) + fWide(i32).
pub const DROPFILES_HEADER_SIZE: usize = 20;

/// Build a wide-char DROPFILES blob: 20-byte header + double-NUL-terminated
/// UTF-16 path list. Pure — testable on all platforms (T1).
pub fn build_dropfiles(paths: &[String]) -> Vec<u8> {
    let wide_len: usize = paths
        .iter()
        .map(|p| p.encode_utf16().count() + 1) // + trailing NUL each
        .sum::<usize>()
        + 1; // list terminator NUL
    let mut bytes = Vec::with_capacity(DROPFILES_HEADER_SIZE + wide_len * 2);
    bytes.extend_from_slice(&(DROPFILES_HEADER_SIZE as u32).to_le_bytes()); // pFiles
    bytes.extend_from_slice(&0i32.to_le_bytes()); // pt.x
    bytes.extend_from_slice(&0i32.to_le_bytes()); // pt.y
    bytes.extend_from_slice(&0i32.to_le_bytes()); // fNC
    bytes.extend_from_slice(&1i32.to_le_bytes()); // fWide = TRUE
    for path in paths {
        for unit in path.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&0u16.to_le_bytes());
    }
    bytes.extend_from_slice(&0u16.to_le_bytes()); // end-of-list empty string
    bytes
}

/// Parse a wide-char DROPFILES blob back into paths. Best-effort: empty on
/// truncated blobs, ANSI lists (fWide=0) or bad offsets. Pure (T1).
pub fn parse_dropfiles(bytes: &[u8]) -> Vec<String> {
    let ok = bytes.len() >= DROPFILES_HEADER_SIZE
        && u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) == 1; // fWide
    if !ok {
        return Vec::new();
    }
    let p_files =
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
    if p_files < DROPFILES_HEADER_SIZE || p_files + 2 > bytes.len() {
        return Vec::new();
    }
    let units: Vec<u16> = bytes[p_files..]
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    let mut paths = Vec::new();
    let mut cur: Vec<u16> = Vec::new();
    for unit in units {
        if unit == 0 {
            if cur.is_empty() {
                break; // end-of-list marker
            }
            paths.push(String::from_utf16_lossy(&cur));
            cur.clear();
        } else {
            cur.push(unit);
        }
    }
    paths
}

// ── DIB (CF_DIB / CF_DIBV5) codec 纯函数（全平台，T1） ─────────────────
// 尽力而为边界（Plan 485 待澄清#2 定稿）：只支持 32bpp 未压缩
// （BI_RGB / BI_BITFIELDS）+ 标准 BGR 掩码；24bpp/16bpp/调色板/RLE 等
// 罕见变体一律 None（截图工具与现代浏览器均写 DIBV5 或 PNG）。

/// BITMAPINFOHEADER（CF_DIB 旧头）大小。
pub const BITMAPINFOHEADER_SIZE: usize = 40;
/// BITMAPV5HEADER 大小（bV5Size 字段合法上限，亦为本方 image_set 写出值）。
pub const BITMAPV5HEADER_SIZE: usize = 124;
/// Plan 485 待澄清#3 定稿：>64MP 直接 None（防误爆内存）。
pub const MAX_IMAGE_PIXELS: u64 = 64_000_000;

const BI_RGB: u32 = 0;
const BI_BITFIELDS: u32 = 3;
/// 标准 32bpp 内存掩码（小端 DWORD 即 BGRA 字节序）。
const STD_RGB_MASKS: (u32, u32, u32) = (0x00FF_0000, 0x0000_FF00, 0x0000_00FF);

/// 解析出的 DIB 几何与像素定位。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DibInfo {
    pub width: i32,
    pub height: i32, // 正 = bottom-up（Win32 常规），负 = top-down
    pub bit_count: u16,
    pub compression: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub alpha_mask: u32, // 0 = 无 alpha 字节语义
    pub pixel_offset: usize,
}

fn le_u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}
fn le_i32(b: &[u8], at: usize) -> i32 {
    i32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}
fn le_u16(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

/// Parse a DIB header (BITMAPINFOHEADER 40 / V4 108 / V5 124) and locate
/// the pixel array. None on truncation or 尽力而为边界外的形状。
pub fn parse_dib_header(bytes: &[u8]) -> Option<DibInfo> {
    if bytes.len() < BITMAPINFOHEADER_SIZE {
        return None;
    }
    let size = le_u32(bytes, 0) as usize;
    let width = le_i32(bytes, 4);
    let height = le_i32(bytes, 8);
    let bit_count = le_u16(bytes, 14);
    let compression = le_u32(bytes, 16);
    let clr_used = le_u32(bytes, 32);
    if width <= 0 || height == 0 || bit_count != 32 {
        return None;
    }
    if compression != BI_RGB && compression != BI_BITFIELDS {
        return None; // RLE / JPEG / PNG 压缩等变体退 None
    }
    let (r, g, b, a, header_end) = if size == BITMAPINFOHEADER_SIZE {
        if compression == BI_BITFIELDS {
            // INFOHEADER + BITFIELDS：3 掩码 DWORD 紧跟头后（无 alpha 掩码）。
            if bytes.len() < 52 {
                return None;
            }
            (le_u32(bytes, 40), le_u32(bytes, 44), le_u32(bytes, 48), 0, 52)
        } else {
            let (r, g, b) = STD_RGB_MASKS;
            (r, g, b, 0, 40)
        }
    } else if (52..=bytes.len()).contains(&size) && size >= 56 {
        // V4(108) / V5(124)：掩码在头内固定偏移 40..52。
        (
            le_u32(bytes, 40),
            le_u32(bytes, 44),
            le_u32(bytes, 48),
            le_u32(bytes, 52),
            size,
        )
    } else {
        return None; // 未知 biSize
    };
    let mut pixel_offset = header_end;
    if clr_used > 0 {
        pixel_offset += clr_used as usize * 4;
    }
    if pixel_offset > bytes.len() {
        return None;
    }
    Some(DibInfo {
        width,
        height,
        bit_count,
        compression,
        red_mask: r,
        green_mask: g,
        blue_mask: b,
        alpha_mask: a,
        pixel_offset,
    })
}

/// 32bpp BGRA DIB 像素 → 紧凑 top-down RGBA8。行序翻转与 stride 在此
/// 纯函数处理；非标准掩码 / 截断 / 超 64MP → None。
pub fn dib_bgra_to_rgba(info: &DibInfo, bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    let w = info.width as u32;
    let h = info.height.unsigned_abs();
    if w as u64 * h as u64 > MAX_IMAGE_PIXELS {
        return None;
    }
    if (info.red_mask, info.green_mask, info.blue_mask) != STD_RGB_MASKS {
        return None;
    }
    if info.alpha_mask != 0 && info.alpha_mask != 0xFF00_0000 {
        return None;
    }
    let stride = w as usize * 4; // 32bpp 行必 4 对齐
    let rows_bytes = stride.checked_mul(h as usize)?;
    let pixels = bytes.get(info.pixel_offset..info.pixel_offset + rows_bytes)?;
    let top_down = info.height < 0;
    let mut rgba = Vec::with_capacity(w as usize * h as usize * 4);
    for row in 0..h as usize {
        let src_row = if top_down { row } else { h as usize - 1 - row };
        let line = &pixels[src_row * stride..src_row * stride + stride];
        for px in line.chunks_exact(4) {
            rgba.push(px[2]); // R
            rgba.push(px[1]); // G
            rgba.push(px[0]); // B
            // 无 alpha 掩码（BI_RGB / 旧头）时 alpha 字节不可信，视为不透明。
            rgba.push(if info.alpha_mask == 0 { 255 } else { px[3] });
        }
    }
    Some((w, h, rgba))
}

/// 紧凑 top-down RGBA8 → CF_DIBV5 blob（124 头 + BI_BITFIELDS 标准
/// 掩码 + bottom-up BGRA 行）。image_set 的写出口。
pub fn rgba_to_dibv5(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let stride = width as usize * 4;
    let mut blob = vec![0u8; BITMAPV5HEADER_SIZE + stride * height as usize];
    blob[0..4].copy_from_slice(&(BITMAPV5HEADER_SIZE as u32).to_le_bytes());
    blob[4..8].copy_from_slice(&(width as i32).to_le_bytes());
    blob[8..12].copy_from_slice(&(height as i32).to_le_bytes()); // 正 = bottom-up
    blob[12..14].copy_from_slice(&1u16.to_le_bytes()); // planes
    blob[14..16].copy_from_slice(&32u16.to_le_bytes()); // bit count
    blob[16..20].copy_from_slice(&BI_BITFIELDS.to_le_bytes());
    blob[20..24].copy_from_slice(&(rows_bytes_of(stride, height)).to_le_bytes());
    blob[40..44].copy_from_slice(&STD_RGB_MASKS.0.to_le_bytes());
    blob[44..48].copy_from_slice(&STD_RGB_MASKS.1.to_le_bytes());
    blob[48..52].copy_from_slice(&STD_RGB_MASKS.2.to_le_bytes());
    blob[52..56].copy_from_slice(&0xFF00_0000u32.to_le_bytes()); // alpha mask
    blob[56..60].copy_from_slice(&0x7352_4742u32.to_le_bytes()); // LCS_sRGB
    for row in 0..height as usize {
        let src = &rgba[row * stride..row * stride + stride];
        let dst_row = height as usize - 1 - row;
        let dst = &mut blob[BITMAPV5HEADER_SIZE + dst_row * stride
            ..BITMAPV5HEADER_SIZE + dst_row * stride + stride];
        for (d, s) in dst.chunks_exact_mut(4).zip(src.chunks_exact(4)) {
            d[0] = s[2]; // B
            d[1] = s[1]; // G
            d[2] = s[0]; // R
            d[3] = s[3]; // A
        }
    }
    blob
}

fn rows_bytes_of(stride: usize, height: u32) -> u32 {
    (stride * height as usize) as u32
}

// ── Win32 bridge (windows × native-clipboard 双门控) ───────────────────
// 以下每函数都遵守同一约定：任一步失败返回空值（G3 的 Windows 内延展——
// 剪贴板被占用/无权限时同样静默降级，不 panic）。

#[cfg(all(windows, feature = "native-clipboard"))]
use windows::core::w;
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::Foundation::{GlobalFree, HANDLE, HGLOBAL, WAIT_OBJECT_0};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
    OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::System::Threading::{CreateMutexW, ReleaseMutex, WaitForSingleObject};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

/// 跨进程命名互斥（测试专用）：nextest 每测试一进程并行，剪贴板是机器级
/// 全局资源——本方 files/image 测试的 EmptyClipboard 会清掉相邻进程
/// set→get 窗口里的内容（Plan 485 T9 实测打红了 418 的 arboard 往返）。
/// 同会话所有剪贴板集成测试（含 ui/clipboard.rs 的 418 用例）经
/// [`Self::acquire`] 串行化；进程内 Mutex 不足以覆盖此形态。
#[cfg(all(windows, feature = "native-clipboard"))]
pub struct GlobalClipboardTestLock(HANDLE);

#[cfg(all(windows, feature = "native-clipboard"))]
impl GlobalClipboardTestLock {
    /// 同会话命名互斥；等待至多 30s（并行测试队头让行），拿不到返回 None
    /// （调用方按 headless 语义跳过）。
    pub fn acquire() -> Option<Self> {
        unsafe {
            let h = CreateMutexW(None, false, w!("auto-lang-clipboard-tests")).ok()?;
            if WaitForSingleObject(h, 30_000) != WAIT_OBJECT_0 {
                let _ = windows::Win32::Foundation::CloseHandle(h);
                return None;
            }
            Some(Self(h))
        }
    }
}

#[cfg(all(windows, feature = "native-clipboard"))]
impl Drop for GlobalClipboardTestLock {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = windows::Win32::Foundation::CloseHandle(self.0);
        }
    }
}

/// RAII clipboard open guard（Drop 时 CloseClipboard）。
#[cfg(all(windows, feature = "native-clipboard"))]
struct ClipboardGuard;

#[cfg(all(windows, feature = "native-clipboard"))]
impl ClipboardGuard {
    fn open() -> Option<Self> {
        unsafe { OpenClipboard(None).ok()? };
        Some(Self)
    }
}

#[cfg(all(windows, feature = "native-clipboard"))]
impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseClipboard();
        }
    }
}

/// Read the file list from the OS clipboard (CF_HDROP). Empty when the
/// format is absent or any Win32 step fails.
#[cfg(all(windows, feature = "native-clipboard"))]
pub fn clipboard_files_get() -> Vec<String> {
    let _guard = match ClipboardGuard::open() {
        Some(g) => g,
        None => return Vec::new(),
    };
    if !unsafe { IsClipboardFormatAvailable(CF_HDROP) }.is_ok() {
        return Vec::new();
    }
    let handle = match unsafe { GetClipboardData(CF_HDROP) } {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    let hdrop = HDROP(handle.0);
    let count = unsafe { DragQueryFileW(hdrop, u32::MAX, None) };
    let mut paths = Vec::with_capacity(count as usize);
    for i in 0..count {
        let len = unsafe { DragQueryFileW(hdrop, i, None) };
        let mut buf = vec![0u16; len as usize + 1];
        let copied = unsafe { DragQueryFileW(hdrop, i, Some(&mut buf)) };
        if copied == len {
            paths.push(String::from_utf16_lossy(&buf[..len as usize]));
        }
    }
    paths
}

/// Write a file list to the OS clipboard as CF_HDROP. `false` on empty
/// input or any Win32 failure.
#[cfg(all(windows, feature = "native-clipboard"))]
pub fn clipboard_files_set(paths: &[PathBuf]) -> bool {
    if paths.is_empty() {
        return false;
    }
    let strs: Vec<String> = paths
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    let blob = build_dropfiles(&strs);
    let _guard = match ClipboardGuard::open() {
        Some(g) => g,
        None => return false,
    };
    unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        set_hglobal_data(CF_HDROP, &blob)
    }
}

/// GlobalAlloc→拷贝→SetClipboardData。成功后所有权归系统；失败自 free
/// 返回 false。
#[cfg(all(windows, feature = "native-clipboard"))]
fn set_hglobal_data(format: u32, bytes: &[u8]) -> bool {
    unsafe {
        let hglobal = match GlobalAlloc(GMEM_MOVEABLE, bytes.len()) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            let _ = GlobalFree(hglobal);
            return false;
        }
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
        let _ = GlobalUnlock(hglobal);
        match SetClipboardData(format, HANDLE(hglobal.0)) {
            Ok(_) => true,
            Err(_) => {
                let _ = GlobalFree(hglobal); // 失败时所有权未转移，自 free
                false
            }
        }
    }
}

/// Write an image file (PNG) to the OS clipboard as CF_DIBV5（32bpp
/// BI_BITFIELDS BGRA）+ registered "PNG" 双挂（兼容只认 PNG 的接收方）。
/// `false` on unreadable file / oversized (>64MP) / Win32 failure.
#[cfg(all(windows, feature = "native-clipboard"))]
pub fn clipboard_image_set(png_path: &std::path::Path) -> bool {
    let file_bytes = match std::fs::read(png_path) {
        Ok(b) => b,
        Err(_) => return false,
    };
    let img = match image::load_from_memory(&file_bytes) {
        Ok(i) => i,
        Err(_) => return false,
    };
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    if w == 0 || h == 0 || w as u64 * h as u64 > MAX_IMAGE_PIXELS {
        return false;
    }
    let blob = rgba_to_dibv5(w, h, &rgba);
    let _guard = match ClipboardGuard::open() {
        Some(g) => g,
        None => return false,
    };
    unsafe {
        if EmptyClipboard().is_err() {
            return false;
        }
        if !set_hglobal_data(CF_DIBV5, &blob) {
            return false;
        }
        // DIBV5 为主格式已落地；registered "PNG" 双挂失败不视为整体失败
        //（只认 PNG 的接收方是少数；DIBV5 是 Win32 通用契约）。
        let png_fmt = RegisterClipboardFormatW(w!("PNG"));
        if png_fmt != 0 {
            let _ = set_hglobal_data(png_fmt, &file_bytes);
        }
        true
    }
}

// ── Win32 image bridge (windows × native-clipboard) ────────────────────

/// image_get 产物：PNG 落 temp 后的元数据（G2 Record：path/width/height）。
pub struct TempImage {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
}

/// Read an image from the OS clipboard as a temp PNG file. Format
/// priority: CF_DIBV5 → CF_DIB → registered "PNG". None when no image
/// format is present or anything fails (含 64MP 防爆拒收).
#[cfg(all(windows, feature = "native-clipboard"))]
pub fn clipboard_image_get() -> Option<TempImage> {
    let _guard = ClipboardGuard::open()?;
    if let Ok(h) = unsafe { GetClipboardData(CF_DIBV5) } {
        if let Some(img) = dib_handle_to_png(h) {
            return Some(img);
        }
    }
    if let Ok(h) = unsafe { GetClipboardData(CF_DIB) } {
        if let Some(img) = dib_handle_to_png(h) {
            return Some(img);
        }
    }
    let png_fmt = unsafe { RegisterClipboardFormatW(w!("PNG")) };
    if png_fmt != 0 {
        if let Ok(h) = unsafe { GetClipboardData(png_fmt) } {
            if let Some(img) = png_handle_to_file(h) {
                return Some(img);
            }
        }
    }
    None
}

/// HGLOBAL 内容拷出（GlobalSize 定长，GlobalLock 后整块复制）。
#[cfg(all(windows, feature = "native-clipboard"))]
fn hglobal_bytes(h: HANDLE) -> Option<Vec<u8>> {
    let hglobal = HGLOBAL(h.0);
    let size = unsafe { GlobalSize(hglobal) };
    if size == 0 {
        return None;
    }
    let ptr = unsafe { GlobalLock(hglobal) };
    if ptr.is_null() {
        return None;
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, size).to_vec() };
    unsafe {
        let _ = GlobalUnlock(hglobal);
    }
    Some(bytes)
}

#[cfg(all(windows, feature = "native-clipboard"))]
fn dib_handle_to_png(h: HANDLE) -> Option<TempImage> {
    let bytes = hglobal_bytes(h)?;
    let info = parse_dib_header(&bytes)?;
    let (w, h_px, rgba) = dib_bgra_to_rgba(&info, &bytes)?;
    let path = temp_png_path();
    image::save_buffer(&path, &rgba, w, h_px, image::ColorType::Rgba8).ok()?;
    Some(TempImage {
        path,
        width: w,
        height: h_px,
    })
}

#[cfg(all(windows, feature = "native-clipboard"))]
fn png_handle_to_file(h: HANDLE) -> Option<TempImage> {
    let bytes = hglobal_bytes(h)?;
    // 只读头部拿尺寸（不整图解码），过 64MP 防线后才落盘。
    let dims = image::ImageReader::new(std::io::Cursor::new(&bytes))
        .with_guessed_format()
        .ok()?
        .into_dimensions()
        .ok()?;
    if dims.0 as u64 * dims.1 as u64 > MAX_IMAGE_PIXELS {
        return None;
    }
    let path = temp_png_path();
    std::fs::write(&path, &bytes).ok()?;
    Some(TempImage {
        path,
        width: dims.0,
        height: dims.1,
    })
}

#[cfg(all(windows, feature = "native-clipboard"))]
fn temp_png_path() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("auto-clipboard-img-{nanos}.png"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── T1: DROPFILES codec 纯单元（全平台档） ──────────────────────

    #[test]
    fn dropfiles_roundtrip_ascii() {
        let paths = vec!["C:\\a\\b.txt".to_string(), "C:\\c d\\e.png".to_string()];
        let blob = build_dropfiles(&paths);
        assert_eq!(parse_dropfiles(&blob), paths);
    }

    #[test]
    fn dropfiles_roundtrip_cjk_paths() {
        // 中文路径宽串（含 emoji 代理对）：from_utf16_lossy 往返无损。
        let paths = vec![
            "D:\\照片\\风景 01.png".to_string(),
            "D:\\备份\\旧档\\说明🄲🄻.txt".to_string(),
        ];
        let blob = build_dropfiles(&paths);
        assert_eq!(parse_dropfiles(&blob), paths);
    }

    #[test]
    fn dropfiles_layout_matches_win32_shape() {
        // 头部字段：pFiles=20 / fWide=1，路径区从 20 起。
        let blob = build_dropfiles(&["C:\\x".to_string()]);
        assert_eq!(u32::from_le_bytes(blob[0..4].try_into().unwrap()), 20);
        assert_eq!(i32::from_le_bytes(blob[16..20].try_into().unwrap()), 1);
        // "C:\x" 4 units + NUL + list NUL = 6 units = 12 bytes。
        assert_eq!(blob.len(), 20 + 12);
    }

    #[test]
    fn dropfiles_parse_rejects_bad_shapes() {
        assert!(parse_dropfiles(&[]).is_empty()); // 截断
        let mut ansi = build_dropfiles(&["C:\\x".to_string()]);
        ansi[16..20].copy_from_slice(&0i32.to_le_bytes()); // fWide=0
        assert!(parse_dropfiles(&ansi).is_empty()); // ANSI 不支持
        let mut bad_off = build_dropfiles(&["C:\\x".to_string()]);
        bad_off[0..4].copy_from_slice(&2u32.to_le_bytes()); // pFiles < 头长
        assert!(parse_dropfiles(&bad_off).is_empty());
    }

    #[test]
    fn dropfiles_roundtrip_empty_list_blob() {
        // 空列表的 blob（仅头+终结符）解析为空——与降级语义一致。
        let blob = build_dropfiles(&[]);
        assert_eq!(blob.len(), 22);
        assert!(parse_dropfiles(&blob).is_empty());
    }

    // ── T2: Win32 files 集成（windows × native-clipboard 档） ────────
    // headless guard（418 arboard 同款语义）：剪贴板打不开（CI 服务会话/
    // 无窗口站）时 clipboard_files_set 返回 false，测试静默跳过而非红。
    // 剪贴板是进程全局资源：cargo test 同进程并行时用互斥锁串行化
    // （nextest 每测试独立进程，锁为空转）。
    #[cfg(all(windows, feature = "native-clipboard"))]
    static CLIPBOARD_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[cfg(all(windows, feature = "native-clipboard"))]
    #[test]
    fn clipboard_files_set_get_roundtrip() {
        let _global = match GlobalClipboardTestLock::acquire() {
            Some(g) => g,
            None => return, // 拿锁超时视同 headless 跳过
        };
        let _lock = CLIPBOARD_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // CF_HDROP 是纯路径列表——文件不必存在即可往返（Explorer 粘贴
        // 才需要真实文件）。中文路径覆盖宽串全链。
        let paths = vec![
            std::path::PathBuf::from("C:\\plan485-tests\\示例 目录\\文件一.txt"),
            std::path::PathBuf::from("C:\\plan485-tests\\second-file.png"),
        ];
        if !clipboard_files_set(&paths) {
            return; // headless CI guard
        }
        let got = clipboard_files_get();
        let want: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
        assert_eq!(got, want);
    }

    // ── T1: DIB codec 纯单元（全平台档） ────────────────────────────

    fn synthetic_rgba_2x3() -> (u32, u32, Vec<u8>) {
        // 合成 2×3 像素含 alpha 梯度（24 字节 RGBA，每字节不同）。
        let rgba: Vec<u8> = (0..2u32 * 3 * 4).map(|i| (i * 7 + 13) as u8).collect();
        (2, 3, rgba)
    }

    #[test]
    fn dibv5_roundtrip_2x3_with_alpha() {
        let (w, h, rgba) = synthetic_rgba_2x3();
        let blob = rgba_to_dibv5(w, h, &rgba);
        let info = parse_dib_header(&blob).unwrap();
        assert_eq!(info.width, 2);
        assert_eq!(info.height, 3); // bottom-up 写出为正高
        assert_eq!(info.compression, 3 /* BI_BITFIELDS */);
        assert_eq!(info.bit_count, 32);
        assert_eq!(info.pixel_offset, BITMAPV5HEADER_SIZE); // 124 头后即像素
        assert_eq!(blob.len(), BITMAPV5HEADER_SIZE + 2 * 3 * 4); // stride=8/行
        let (w2, h2, rgba2) = dib_bgra_to_rgba(&info, &blob).unwrap();
        assert_eq!((w2, h2), (2, 3));
        assert_eq!(rgba2, rgba); // 行序+字节序往返无损（含 alpha）
    }

    #[test]
    fn dib_row_order_bottom_up_vs_top_down() {
        // 1×2 两行：bottom-up blob 行 0 存源末行；负高 top-down 解读保持 blob 序。
        let src = [10u8, 20, 30, 40, 50, 60, 70, 80];
        let blob = rgba_to_dibv5(1, 2, &src);
        // bottom-up（正高）回读 → 与源完全一致（rgba_to_dibv5 已翻行）。
        let info = parse_dib_header(&blob).unwrap();
        let (_, _, rgba) = dib_bgra_to_rgba(&info, &blob).unwrap();
        assert_eq!(rgba, src);
        // 同一 blob 改负高（top-down 语义）→ 首行变 blob 首行 = 源末行。
        let mut td = blob.clone();
        td[8..12].copy_from_slice(&(-2i32).to_le_bytes());
        let info_td = parse_dib_header(&td).unwrap();
        let (_, _, rgba_td) = dib_bgra_to_rgba(&info_td, &td).unwrap();
        assert_eq!(&rgba_td[..4], &src[4..8]);
        assert_eq!(&rgba_td[4..], &src[..4]);
    }

    #[test]
    fn dib_parse_rejects_unsupported_shapes() {
        // 截断。
        assert!(parse_dib_header(&[0u8; 10]).is_none());
        let mk_header = |bit_count: u16, compression: u32, height: i32| {
            let mut b = vec![0u8; BITMAPINFOHEADER_SIZE];
            b[0..4].copy_from_slice(&40u32.to_le_bytes());
            b[4..8].copy_from_slice(&1i32.to_le_bytes()); // width
            b[8..12].copy_from_slice(&height.to_le_bytes());
            b[14..16].copy_from_slice(&bit_count.to_le_bytes());
            b[16..20].copy_from_slice(&compression.to_le_bytes());
            b
        };
        // 24bpp（bit_count）。
        assert!(parse_dib_header(&mk_header(24, 0, 1)).is_none());
        // RLE 压缩（compression=2）。
        assert!(parse_dib_header(&mk_header(32, 2, 1)).is_none());
        // 零高。
        assert!(parse_dib_header(&mk_header(32, 0, 0)).is_none());
        // 像素区截断：头声称 1×1 但 blob 只有头。
        assert!(dib_bgra_to_rgba(&parse_dib_header(&mk_header(32, 0, 1)).unwrap(), &mk_header(32, 0, 1)).is_none());
        // 64MP 防爆：解析几何成功但转换拒绝。
        let mut huge = mk_header(32, 0, 1);
        huge[4..8].copy_from_slice(&80_000i32.to_le_bytes()); // 80000×80000=6.4G
        huge[8..12].copy_from_slice(&80_000i32.to_le_bytes());
        let info = parse_dib_header(&huge).unwrap();
        assert!(dib_bgra_to_rgba(&info, &huge).is_none());
    }

    // ── T2: Win32 image 集成（windows × native-clipboard 档） ────────

    #[cfg(all(windows, feature = "native-clipboard", feature = "ui-clipboard"))]
    #[test]
    fn clipboard_image_get_none_without_image() {
        let _global = match GlobalClipboardTestLock::acquire() {
            Some(g) => g,
            None => return,
        };
        let _lock = CLIPBOARD_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // 文本剪贴板可用性即 headless guard：打不开则跳过。
        if !crate::ui::clipboard::clipboard_set("plan485-image-none") {
            return;
        }
        // 仅文本（EmptyClipboard 后 set 文本，无任何图像格式）→ None。
        assert!(clipboard_image_get().is_none());
    }

    #[cfg(all(windows, feature = "native-clipboard"))]
    #[test]
    fn clipboard_image_set_get_roundtrip() {
        let _global = match GlobalClipboardTestLock::acquire() {
            Some(g) => g,
            None => return,
        };
        let _lock = CLIPBOARD_TEST_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        // 合成 4×3 RGBA→PNG 文件（alpha 梯度），set→get 经 DIBV5 通道往返。
        let (w, h) = (4u32, 3u32);
        let rgba: Vec<u8> = (0..w * h * 4).map(|i| (i * 11 + 5) as u8).collect();
        let png_path = std::env::temp_dir().join("plan485-set-src.png");
        image::save_buffer(&png_path, &rgba, w, h, image::ColorType::Rgba8).unwrap();
        if !clipboard_image_set(&png_path) {
            return; // headless CI guard
        }
        let got = clipboard_image_get().expect("image back on clipboard");
        assert_eq!((got.width, got.height), (w, h));
        // 像素容差断言：DIBV5 通道是无损转换，走 0 容差精确比对；
        // 读回 temp PNG 解码比对。
        let img = image::ImageReader::open(&got.path)
            .unwrap()
            .decode()
            .unwrap()
            .to_rgba8();
        assert_eq!(img.dimensions(), (w, h));
        assert_eq!(img.into_raw(), rgba);
        let _ = std::fs::remove_file(&got.path);
        let _ = std::fs::remove_file(&png_path);
    }
}
