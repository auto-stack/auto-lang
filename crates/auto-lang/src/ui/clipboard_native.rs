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

use std::path::PathBuf;

// ── Win32 clipboard format IDs（稳定 ABI 常量，免引 Win32_System_Ole）──
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

// ── Win32 bridge (windows × native-clipboard 双门控) ───────────────────
// 以下每函数都遵守同一约定：任一步失败返回空值（G3 的 Windows 内延展——
// 剪贴板被占用/无权限时同样静默降级，不 panic）。

#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::Foundation::{GlobalFree, HANDLE};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
    OpenClipboard, SetClipboardData,
};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE};
#[cfg(all(windows, feature = "native-clipboard"))]
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};

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
        let hglobal = match GlobalAlloc(GMEM_MOVEABLE, blob.len()) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let ptr = GlobalLock(hglobal);
        if ptr.is_null() {
            let _ = GlobalFree(hglobal);
            return false;
        }
        std::ptr::copy_nonoverlapping(blob.as_ptr(), ptr as *mut u8, blob.len());
        let _ = GlobalUnlock(hglobal);
        match SetClipboardData(CF_HDROP, HANDLE(hglobal.0)) {
            Ok(_) => true,
            Err(_) => {
                let _ = GlobalFree(hglobal); // 失败时所有权未转移，自 free
                false
            }
        }
    }
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
}
