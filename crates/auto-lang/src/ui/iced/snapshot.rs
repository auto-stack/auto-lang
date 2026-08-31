//! Plan 497 T2：每窗口真缩略——快照核心。
//!
//! **T1 定案（裁剪式整窗快照）**：headless/overlay 子树栅格化皆不可行
//! （详见 plan 497 待澄清③），改由宿主触发 `iced::window::screenshot`
//! （Plan 285 同通道，唯一公开栅格化 API）拿整窗 RGBA——**物理像素 +
//! scale_factor**——再按目标虚拟窗 `VWinState.rect`（宿主窗逻辑坐标）
//! 裁剪 → box 降采样（长边 ≤ [`THUMB_MAX`]）→ [`WindowSnapshot`] 入
//! TTL 缓存。抓取是异步 Task（渲染帧后回调），消费者 miss 时由宿主编排
//! 触发，本帧回退 fallback icon（渲染臂/注入面各自兜底）。
//!
//! 缓存为进程级单例：桌面会话单进程单会话，而消费点（`window_thumbnail`
//! 渲染臂、switcher `mru_thumbs` 注入）是自由函数，无 `DesktopSession`
//! 访问——全局可达是窗口缩略的必要条件。新鲜度 = 召唤时即时抓取 +
//! [`SNAPSHOT_TTL`] 短缓存 + 事件失效（relayout/close 由 renderer 调
//! [`invalidate`]/[`invalidate_all`]）；无后台定时刷新（非目标）。

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use crate::ui::session::Wid;

/// 缩略长边上限（计划详细设计 §1）。
pub const THUMB_MAX: u32 = 256;
/// 快照新鲜度 TTL（计划详细设计 §1：短 TTL 缓存，召唤式即取即用）。
pub const SNAPSHOT_TTL: Duration = Duration::from_secs(2);

/// 一枚窗口缩略：降采样后的 RGBA8 像素（预乘与否同截图原样——iced
/// `image::Handle::from_rgba` 直接受纳）。
#[derive(Debug, Clone)]
pub struct WindowSnapshot {
    pub rgba: Vec<u8>,
    pub w: u32,
    pub h: u32,
}

impl WindowSnapshot {
    /// 采样 (x, y) 像素 RGB（越界回 None）。
    pub fn pixel(&self, x: u32, y: u32) -> Option<(u8, u8, u8)> {
        if x >= self.w || y >= self.h {
            return None;
        }
        let i = ((y * self.w + x) * 4) as usize;
        Some((self.rgba[i], self.rgba[i + 1], self.rgba[i + 2]))
    }
}

fn cache() -> &'static Mutex<HashMap<Wid, (WindowSnapshot, Instant)>> {
    static CACHE: OnceLock<Mutex<HashMap<Wid, (WindowSnapshot, Instant)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 抓取请求冷却：同一 wid 两次入队的最小间隔（防"抓取失败→下帧再排队"
/// 的请求风暴；抓取本身是整窗截图，代价远超一次 HashMap 写）。
const REQUEST_COOLDOWN: Duration = Duration::from_millis(500);

fn pending() -> &'static Mutex<(Vec<Wid>, HashMap<Wid, Instant>)> {
    static PENDING: OnceLock<Mutex<(Vec<Wid>, HashMap<Wid, Instant>)>> = OnceLock::new();
    PENDING.get_or_init(|| Mutex::new((Vec::new(), HashMap::new())))
}

/// 渲染臂 miss 时标记"想要该窗快照"（非阻塞、冷却去重）。宿主 update
/// 周期 [`take_capture_requests`] 排空并发起 screenshot Task——回调把
/// 整窗 RGBA 按各窗 rect 裁剪入缓存。native wid（"N&lt;slot&gt;"）由调用
/// 侧 parse 失败自然不进来。
pub fn request_capture(wid: Wid) {
    let mut guard = pending().lock().unwrap();
    let (queue, last) = &mut *guard;
    if let Some(ts) = last.get(&wid) {
        if ts.elapsed() < REQUEST_COOLDOWN {
            return;
        }
    }
    if !queue.contains(&wid) {
        queue.push(wid);
    }
    last.insert(wid, Instant::now());
}

/// 宿主排空抓取请求（每 update 周期一次；返回本轮要抓的 wid 集——
/// 宿主用一次整窗 screenshot 服务全部请求）。
pub fn take_capture_requests() -> Vec<Wid> {
    std::mem::take(&mut pending().lock().unwrap().0)
}

/// 消费口：读一枚窗口缩略（TTL 内才返回；过期条目惰性清除）。
/// 等价于"当前是否有可用的真缩略"；miss 者由宿主编排异步抓取。
pub fn snapshot_window(wid: Wid) -> Option<WindowSnapshot> {
    let mut guard = cache().lock().unwrap();
    match guard.get(&wid) {
        Some((snap, ts)) if ts.elapsed() <= SNAPSHOT_TTL => Some(snap.clone()),
        Some(_) => {
            guard.remove(&wid);
            None
        }
        None => None,
    }
}

/// 抓取回调落缓存（renderer screenshot 回调臂调用）。
pub fn cache_put(wid: Wid, snap: WindowSnapshot) {
    cache().lock().unwrap().insert(wid, (snap, Instant::now()));
}

/// 事件失效：单窗内容/几何变化（relayout/dirty/close）。
pub fn invalidate(wid: Wid) {
    cache().lock().unwrap().remove(&wid);
}

/// 事件失效：全场（分区切换/dock 位置热切换等整窗重排场景）。
/// 同时清抓取队列 + 冷却表（失效后应允许立即重抓，
/// 否则 500ms 冷却会把重排后的首轮请求挡在队外）。
pub fn invalidate_all() {
    cache().lock().unwrap().clear();
    let mut guard = pending().lock().unwrap();
    guard.0.clear();
    guard.1.clear();
}

/// 整窗截图 → 窗口缩略（T1 定案核心）：
/// 逻辑 `rect` × `scale` → 物理像素裁剪（越界 clamp）→ box 降采样。
/// 零尺寸/整区越界回 None（411 零尺寸守卫同族——调用方保 fallback）。
pub fn thumbnail_from_screenshot(
    rgba: &[u8],
    img_w: u32,
    img_h: u32,
    rect: iced::Rectangle,
    scale: f32,
) -> Option<WindowSnapshot> {
    let (cropped, cw, ch) = crop_physical(rgba, img_w, img_h, rect, scale)?;
    let (thumb, tw, th) = downsample_box(&cropped, cw, ch, THUMB_MAX);
    Some(WindowSnapshot { rgba: thumb, w: tw, h: th })
}

/// 逻辑矩形 → 物理像素裁剪（行拷贝；负起点/超界 clamp，零有效区 None）。
fn crop_physical(
    rgba: &[u8],
    img_w: u32,
    img_h: u32,
    rect: iced::Rectangle,
    scale: f32,
) -> Option<(Vec<u8>, u32, u32)> {
    let f = |v: f32| v.max(0.0);
    let px = (f(rect.x) * scale).round() as i64;
    let py = (f(rect.y) * scale).round() as i64;
    let pw = (f(rect.width) * scale).round() as i64;
    let ph = (f(rect.height) * scale).round() as i64;
    if pw <= 0 || ph <= 0 || px >= img_w as i64 || py >= img_h as i64 {
        return None;
    }
    let x0 = px.max(0) as u32;
    let y0 = py.max(0) as u32;
    let x1 = (px + pw).clamp(0, img_w as i64) as u32;
    let y1 = (py + ph).clamp(0, img_h as i64) as u32;
    let w = x1.checked_sub(x0)?;
    let h = y1.checked_sub(y0)?;
    if w == 0 || h == 0 {
        return None;
    }
    if rgba.len() < (img_w as usize * img_h as usize * 4) {
        return None;
    }
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for y in y0..y1 {
        let start = ((y * img_w + x0) * 4) as usize;
        out.extend_from_slice(&rgba[start..start + (w * 4) as usize]);
    }
    Some((out, w, h))
}

/// box 降采样：等比缩到长边 ≤ max（已达标则原样），逐 box 均值。
fn downsample_box(rgba: &[u8], w: u32, h: u32, max: u32) -> (Vec<u8>, u32, u32) {
    let long = w.max(h);
    if long <= max {
        return (rgba.to_vec(), w, h);
    }
    let ratio = max as f32 / long as f32;
    let tw = ((w as f32 * ratio).round() as u32).max(1);
    let th = ((h as f32 * ratio).round() as u32).max(1);
    let mut out = vec![0u8; (tw * th * 4) as usize];
    for ty in 0..th {
        let y0 = (ty as u64 * h as u64) / th as u64;
        let y1 = (((ty + 1) as u64 * h as u64) / th as u64).max(y0 + 1);
        for tx in 0..tw {
            let x0 = (tx as u64 * w as u64) / tw as u64;
            let x1 = (((tx + 1) as u64 * w as u64) / tw as u64).max(x0 + 1);
            let (mut r, mut g, mut b, mut a) = (0u64, 0u64, 0u64, 0u64);
            let mut n = 0u64;
            for y in y0..y1.min(h as u64) {
                for x in x0..x1.min(w as u64) {
                    let i = ((y as u32 * w + x as u32) * 4) as usize;
                    r += rgba[i] as u64;
                    g += rgba[i + 1] as u64;
                    b += rgba[i + 2] as u64;
                    a += rgba[i + 3] as u64;
                    n += 1;
                }
            }
            let o = ((ty * tw + tx) * 4) as usize;
            out[o] = (r / n) as u8;
            out[o + 1] = (g / n) as u8;
            out[o + 2] = (b / n) as u8;
            out[o + 3] = (a / n) as u8;
        }
    }
    (out, tw, th)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 合成整窗物理 RGBA：2×2 四色块（红/绿上、蓝/黄下）——T1 spike demo
    /// 同型（HiDPI：逻辑 400×300 × scale 2 = 物理 800×600）。
    fn quad_rgba(w: u32, h: u32) -> Vec<u8> {
        let mut v = vec![0u8; (w * h * 4) as usize];
        for y in 0..h {
            for x in 0..w {
                let (r, g, b) = if y < h / 2 {
                    if x < w / 2 { (0xE0, 0x20, 0x20) } else { (0x20, 0xC0, 0x20) }
                } else if x < w / 2 {
                    (0x20, 0x40, 0xE0)
                } else {
                    (0xE0, 0xD0, 0x20)
                };
                let i = ((y * w + x) * 4) as usize;
                v[i] = r;
                v[i + 1] = g;
                v[i + 2] = b;
                v[i + 3] = 0xFF;
            }
        }
        v
    }

    fn center_rect() -> iced::Rectangle {
        iced::Rectangle { x: 100.0, y: 75.0, width: 200.0, height: 150.0 }
    }

    /// T2-1：四色整窗 → 中央逻辑 rect 裁剪 ×scale → 尺寸 + 四象限中心色。
    #[test]
    fn t2_snapshot_thumbnail_size_and_quadrant_colors() {
        let (w, h) = (800u32, 600u32);
        let rgba = quad_rgba(w, h);
        let snap = thumbnail_from_screenshot(&rgba, w, h, center_rect(), 2.0)
            .expect("中央区应裁剪成功");
        // 200×150 逻辑 = 400×300 物理 → 长边 400 > 256 → 等比降采样 256×192。
        assert_eq!((snap.w, snap.h), (256, 192), "长边 400 → 256 等比");
        let dom = |x: u32, y: u32| snap.pixel(x, y).expect("in-bounds");
        let (qx, qy) = (snap.w / 4, snap.h / 4);
        // 四象限内缩点各自色相（box 混色容差 32——T1 实测同款）。
        assert!(dom(qx, qy).0.abs_diff(0xE0) <= 32, "左上=红: {:?}", dom(qx, qy));
        assert!(dom(snap.w - 1 - qx, qy).1.abs_diff(0xC0) <= 32, "右上=绿");
        assert!(dom(qx, snap.h - 1 - qy).2.abs_diff(0xE0) <= 32, "左下=蓝");
        assert!(dom(snap.w - 1 - qx, snap.h - 1 - qy).0.abs_diff(0xE0) <= 32, "右下=黄");
    }

    /// T2-2：小窗（长边 ≤256）不放大不缩放——原样返回。
    #[test]
    fn t2_snapshot_small_window_passthrough() {
        let (w, h) = (200u32, 150u32);
        let rgba = quad_rgba(w, h);
        let rect = iced::Rectangle { x: 0.0, y: 0.0, width: 100.0, height: 75.0 };
        let snap = thumbnail_from_screenshot(&rgba, w, h, rect, 2.0).unwrap();
        assert_eq!((snap.w, snap.h), (200, 150), "长边 200 ≤ 256 原样");
    }

    /// T2-3：越界 clamp 与零尺寸守卫——负起点贴边裁、零宽/整区越界 None。
    #[test]
    fn t2_snapshot_crop_edge_clamp_and_zero_guard() {
        let (w, h) = (400u32, 300u32);
        let rgba = quad_rgba(w, h);
        // 负起点：clamp 到 (0,0)，宽高取到图像右下。
        let neg = iced::Rectangle { x: -50.0, y: -40.0, width: 100.0, height: 80.0 };
        let snap = thumbnail_from_screenshot(&rgba, w, h, neg, 1.0).unwrap();
        assert_eq!((snap.w, snap.h), (100, 80), "负起点 clamp 后全尺寸保留");
        // 超右下：有效区只剩 50×60。
        let over = iced::Rectangle { x: 350.0, y: 240.0, width: 100.0, height: 100.0 };
        let snap = thumbnail_from_screenshot(&rgba, w, h, over, 1.0).unwrap();
        assert_eq!((snap.w, snap.h), (50, 60), "越界 clamp 到图像右下缘");
        // 零尺寸 / 完全越界：None。
        let zero = iced::Rectangle { x: 10.0, y: 10.0, width: 0.0, height: 10.0 };
        assert!(thumbnail_from_screenshot(&rgba, w, h, zero, 1.0).is_none());
        let outside = iced::Rectangle { x: 500.0, y: 0.0, width: 50.0, height: 50.0 };
        assert!(thumbnail_from_screenshot(&rgba, w, h, outside, 1.0).is_none());
        // 底层 RGBA 短缺（尺寸谎报）：None 而非 panic。
        let short = &rgba[..rgba.len() / 2];
        assert!(thumbnail_from_screenshot(short, w, h, center_rect(), 1.0).is_none());
    }

    /// T2-4：TTL 过期与事件失效路径。
    #[test]
    fn t2_snapshot_cache_ttl_and_invalidation() {
        let wid = Wid(97001);
        invalidate(wid);
        assert!(snapshot_window(wid).is_none(), "空缓存 miss");

        let snap = WindowSnapshot { rgba: vec![1, 2, 3, 255], w: 1, h: 1 };
        cache_put(wid, snap.clone());
        assert!(snapshot_window(wid).is_some(), "TTL 内命中");
        assert_eq!(snapshot_window(wid).unwrap().rgba, snap.rgba);

        // 伪造过期：直接改条目时间戳到 3s 前（> SNAPSHOT_TTL 2s）。
        cache().lock().unwrap().get_mut(&wid).unwrap().1 =
            Instant::now() - Duration::from_secs(3);
        assert!(snapshot_window(wid).is_none(), "TTL 过期 miss");
        assert!(
            !cache().lock().unwrap().contains_key(&wid),
            "过期条目惰性清除"
        );

        // 事件失效：单窗 + 全场。
        cache_put(wid, snap.clone());
        invalidate(wid);
        assert!(snapshot_window(wid).is_none(), "invalidate(wid) 生效");
        cache_put(wid, snap);
        invalidate_all();
        assert!(snapshot_window(wid).is_none(), "invalidate_all() 生效");
    }
}
