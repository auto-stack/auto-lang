//! Plan 515 D1 —— native 窗口真图标（HICON → RGBA）缓存。
//!
//! 486 期 native 槽位条目的 `icon` 字段为 `"app-window"` lucide 占位
//!（473/486 两度延期的 HICON 增强项，本计划判定纳入清偿）。链路：
//! 投影期（`sync_shell_windows` native 槽位段）幂等 `ensure`（每槽一次，
//! 失败也记 None 防每帧重试）→ icon 字段 = `hicon:N<slot>`（有真图标）
//! 或占位回退；渲染臂（PUA button icon / `AbstractView::Image`）见
//! `hicon:` 前缀即读本缓存出 `image::Handle::from_rgba`。
//!
//! 全局单例（497 snapshot 同款必要条件：渲染臂是自由函数，无
//! `DesktopSession` 访问）。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// 一枚 native 窗口图标：RGBA8（straight，win32 层 BGRA 已换序）。
#[derive(Debug, Clone)]
pub struct NativeIcon {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

/// slot_id → 提取结果（None = 已试且失败——占位档，不重试）。
fn cache() -> &'static Mutex<HashMap<u64, Option<NativeIcon>>> {
    static CACHE: OnceLock<Mutex<HashMap<u64, Option<NativeIcon>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 幂等提取入缓存（每槽至多一次——Win32 调用微秒级，投影期同步执行
/// 可接受；失败占位防每帧重试）。
pub fn ensure(slot_id: u64, hwnd: crate::ui::native_dock::NativeHwnd) {
    let mut guard = cache().lock().unwrap();
    if guard.contains_key(&slot_id) {
        return;
    }
    let extracted = crate::ui::native_dock::win32::window_icon_rgba(hwnd).map(
        |(rgba, w, h)| NativeIcon { rgba, w, h },
    );
    guard.insert(slot_id, extracted);
}

/// 读缓存（miss = 未 ensure 或提取失败 → 调用方回退 lucide 占位）。
pub fn get(slot_id: u64) -> Option<NativeIcon> {
    cache().lock().unwrap().get(&slot_id).cloned().flatten()
}

/// 投影 icon 字段：有真图标 → `hicon:N<slot>`；否则 `app-window` 占位
///（`sync_shell_windows` native 槽位条目便捷入口）。
pub fn icon_field(slot_id: u64) -> String {
    if get(slot_id).is_some() {
        format!("hicon:{slot_id}")
    } else {
        "app-window".to_string()
    }
}

/// 渲染臂便捷入口：`hicon:<slot>` 串 → 图标（非该前缀/未命中 → None）。
pub fn parse_field(field: &str) -> Option<NativeIcon> {
    let slot = field.strip_prefix("hicon:")?;
    let id: u64 = slot.parse().ok()?;
    get(id)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试注入口（真提取链需真窗口——单元档注入占位 RGBA 验缓存语义）。
    fn put(slot_id: u64) {
        cache().lock().unwrap().insert(
            slot_id,
            Some(NativeIcon { rgba: vec![1, 2, 3, 4], w: 1, h: 1 }),
        );
    }

    /// slot_id 测试隔离段（生产槽位单调小值；测试用 9xxxxx 段）。
    const T1: u64 = 900_001;
    const T2: u64 = 900_002;

    /// 缓存语义：注入 → icon_field 出 hicon 方案 → parse 回读；
    /// 非 hicon 前缀/坏 id/未注入 → None/占位。
    #[test]
    fn icon_field_and_parse_roundtrip() {
        put(T1);
        assert_eq!(icon_field(T1), format!("hicon:{T1}"), "有真图标出方案串");
        let got = parse_field(&format!("hicon:{T1}")).expect("回读");
        assert_eq!((got.w, got.h), (1, 1), "像素尺寸");
        assert_eq!(got.rgba, vec![1, 2, 3, 4]);
        // 非 hicon / 坏 id / 未注入槽 → None；未注入 icon_field → 占位。
        assert!(parse_field("lucide:app-window").is_none(), "lucide 不误吞");
        assert!(parse_field("hicon:bad").is_none(), "坏 id 拒收");
        assert!(parse_field(&format!("hicon:{T2}")).is_none(), "未注入槽 miss");
        assert_eq!(icon_field(T2), "app-window", "未注入 = 占位回退");
    }

    /// ensure 失败路径（伪 hwnd 真提取链失败）→ 失败占位缓存（不重试、
    /// 不炸、icon 字段回退占位）。Windows 真链；no-op 平台恒 None 同断言。
    #[test]
    fn ensure_bogus_hwnd_caches_failure() {
        let bogus = crate::ui::native_dock::NativeHwnd(0x515_515);
        ensure(T2, bogus);
        assert_eq!(icon_field(T2), "app-window", "失败 = 占位");
        ensure(T2, bogus); // 二次 ensure 命中缓存不重试（幂等）。
        assert_eq!(icon_field(T2), "app-window");
    }
}
