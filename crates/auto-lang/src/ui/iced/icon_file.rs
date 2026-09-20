//! PLAN-018 —— `iconfile:<stem>` 位图图标后端（双主题 PNG 文件）。
//!
//! icon 字符串协议族第三后端（`lucide:` svg、`hicon:` native raster 之后
//! 的文件位图）：`iconfile:<stem>` → `{root}/{light|dark}/<stem>.png` →
//! `image::Handle`。资产根解析序：`AUTO_OS_ICON_ROOT` env（桌面 boot 注入
//! 绝对路径）→ `AUTO_OS_ROOT/assets/icons` → 均缺席 = None（回退链下沉
//! lucide，零回归）。主题位由 `crate::ui::style::theme::dark_mode()` 进程
//! 级链供给（boot 期 set_dark_mode，PLAN-615 链）。全局缓存（native_icon
//! 同款 OnceLock 模式）：键 (stem, dark)，读失败占位防每帧重试。

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use iced::widget::image::Handle;

/// 一枚已解析的 iconfile 引用（stem 已过字符白名单）。
#[derive(Debug, Clone, PartialEq)]
pub struct IconFileRef {
    pub stem: String,
}

/// 渲染臂便捷入口：`iconfile:<stem>` 串 → 引用（非该前缀/空/非法字符
/// → None）。白名单 `[A-Za-z0-9_-]`——stem 直接拼路径，拒收穿越与杂字符。
pub fn parse_field(field: &str) -> Option<IconFileRef> {
    let stem = field.strip_prefix("iconfile:")?.trim();
    if stem.is_empty()
        || !stem
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return None;
    }
    Some(IconFileRef {
        stem: stem.to_string(),
    })
}

fn cache() -> &'static Mutex<HashMap<(String, bool, u32), Option<Handle>>> {
    static CACHE: OnceLock<Mutex<HashMap<(String, bool, u32), Option<Handle>>>> =
        OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 资产根解析序：AUTO_OS_ICON_ROOT → AUTO_OS_ROOT/assets/icons →
/// CWD/assets/icons → P-3 OS 根解析序（`resolve_os_manifest_root`：
/// 兄弟 → 主检出 `D:/autostack/auto-os`，PLAN-035 T-09——原双 env 臂外
/// 无回退，裸 exec/缺 env 启动形态全部 iconfile 位图空白实机在案）。
/// 均缺席或目录不存在 → None（回退链下沉，不告警——未配置是常态）。
pub fn icon_root() -> Option<PathBuf> {
    if let Some(r) = std::env::var_os("AUTO_OS_ICON_ROOT") {
        let p = PathBuf::from(r);
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Some(r) = std::env::var_os("AUTO_OS_ROOT") {
        let p = PathBuf::from(r).join("assets").join("icons");
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Ok(cwd) = std::env::current_dir() {
        let p = cwd.join("assets").join("icons");
        if p.is_dir() {
            return Some(p);
        }
    }
    if let Some(os_root) =
        crate::ui::app_registry::resolve_os_manifest_root(std::path::Path::new("."))
    {
        let p = os_root.join("assets").join("icons");
        if p.is_dir() {
            return Some(p);
        }
    }
    None
}

/// 桌面 boot 接线（PLAN-018 W4）：读 `<root>/mapping.json`（id → stem；
/// `_` 前缀键为注释忽略；坏 JSON 静默跳过），把命中 id 的条目 icon 改写为
/// `iconfile:<stem>`。未映射/缺席 = 原样（lucide 兜底零回归）。
pub fn apply_icon_mapping(entries: &mut [crate::ui::app_registry::AppRegistryEntry], root: &PathBuf) {
    use std::collections::HashMap as Map;
    let Ok(raw) = std::fs::read_to_string(root.join("mapping.json")) else {
        return;
    };
    let Ok(parsed) = serde_json::from_str::<Map<String, String>>(&raw) else {
        eprintln!("[icon-file] mapping.json parse failed — lucide fallback");
        return;
    };
    for e in entries.iter_mut() {
        if let Some(stem) = parsed.get(&e.id) {
            if parse_field(&format!("iconfile:{stem}")).is_some() {
                e.icon = format!("iconfile:{stem}");
            }
        }
    }
}

/// 渲染臂主入口：icon 串 + 主题位 → 位图 Handle（512 原生档）。非
/// iconfile 前缀 / 根未配置 / 文件缺失 → None（调用方按回退链下沉
/// hicon/lucide）。显示档消费面走 [`load_sized`]。
pub fn load(field: &str, dark: bool) -> Option<Handle> {
    let r = parse_field(field)?;
    load_stem(&r.stem, dark)
}

/// PLAN-018-FU7 —— 显示档装载：tex = 2×显示档（HiDPI 物理像素覆盖，
/// 上限 512 = 资产原生档）。根因：512 直挂纹理经 iced 缺省
/// `FilterMethod::Linear` + wgpu 无 mipmap min_filter 缩 ~28×（任务栏
/// 18px 档）时每屏幕像素仅 2×2 tap——圆角高对比边缘欠采样呈硬阶梯
/// （GPU 行为模拟：18px 档半透明覆盖像素 5 个 vs 面积平均 58 个；
/// 证据图 auto-os `.auto/icon-gpu-vs-box.png`）。装载期 CPU 一次
/// Triangle 重采样，GPU 侧零/温和缩小（≤2×），毛刺消除。
pub fn load_sized(field: &str, dark: bool, display_px: f32) -> Option<Handle> {
    let r = parse_field(field)?;
    let tex_px = ((display_px.max(1.0) * 2.0).ceil() as u32).clamp(24, 512);
    load_stem_sized(&r.stem, dark, tex_px)
}

/// 缓存优先的 stem 装载（512 原生档；键 (stem, dark, 512)）。
pub fn load_stem(stem: &str, dark: bool) -> Option<Handle> {
    load_stem_sized(stem, dark, 512)
}

/// 缓存优先的 stem 档位装载（键 (stem, dark, tex_px)；None 也入缓存
/// 防每帧重试）。
pub fn load_stem_sized(stem: &str, dark: bool, tex_px: u32) -> Option<Handle> {
    let key = (stem.to_string(), dark, tex_px);
    if let Ok(guard) = cache().lock() {
        if let Some(hit) = guard.get(&key) {
            return hit.clone();
        }
    }
    let loaded = load_uncached(stem, dark, tex_px);
    if let Ok(mut guard) = cache().lock() {
        guard.insert(key, loaded.clone());
    }
    loaded
}

fn load_uncached(stem: &str, dark: bool, tex_px: u32) -> Option<Handle> {
    let root = icon_root()?;
    let theme = if dark { "dark" } else { "light" };
    let bytes = std::fs::read(root.join(theme).join(format!("{stem}.png"))).ok()?;
    let (w, h, rgba) = decode_resized(&bytes, tex_px)?;
    Some(Handle::from_rgba(w, h, rgba))
}

/// 解码 + 目标档重采样（纯函数——单测面；`image` 为 image-pipeline 门
/// 内依赖，ui-iced 传递启用）。目标档 ≥ 源最短边 = 原样解码返回
/// （不放大不重采样）。图标资产为正方形，`resize_exact` 方形档安全。
fn decode_resized(bytes: &[u8], tex_px: u32) -> Option<(u32, u32, Vec<u8>)> {
    let img = image::load_from_memory(bytes).ok()?;
    let (w, h) = (img.width(), img.height());
    if tex_px >= w.min(h) {
        let rgba = img.to_rgba8();
        return Some((w, h, rgba.into_raw()));
    }
    let resized =
        img.resize_exact(tex_px, tex_px, image::imageops::FilterType::Triangle);
    let rgba = resized.to_rgba8();
    Some((tex_px, tex_px, rgba.into_raw()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试注入口（真装载需资产树+env——单元档注入 Handle 验缓存/回退语义）。
    fn put(stem: &str, dark: bool) {
        cache().lock().unwrap().insert(
            (stem.to_string(), dark, 512),
            Some(Handle::from_rgba(1, 1, vec![9, 8, 7, 6])),
        );
    }

    #[test]
    fn parse_field_whitelist_and_prefix_isolation() {
        let r = parse_field("iconfile:system-monitor").expect("合法 stem");
        assert_eq!(r.stem, "system-monitor");
        assert!(parse_field("iconfile:").is_none(), "空 stem 拒收");
        assert!(parse_field("iconfile:../escape").is_none(), "穿越拒收");
        assert!(parse_field("iconfile:has space").is_none(), "非法字符拒收");
        assert!(
            parse_field("lucide:app-window").is_none(),
            "lucide 不误吞"
        );
        assert!(parse_field("hicon:5").is_none(), "hicon 不误吞");
        assert!(parse_field("app-window").is_none(), "裸 lucide 名不误吞");
    }

    #[test]
    fn load_cache_hit_and_miss_placeholder() {
        // 测试段 stem（9xxxx 语义段）；缓存注入 → load 命中不经文件系统。
        put("zz-test-hit", true);
        assert!(load("iconfile:zz-test-hit", true).is_some(), "缓存命中");
        // 未注入 + 资产根缺席（测试进程无 AUTO_OS_* env 注入）→ None。
        assert!(
            load("iconfile:zz-test-miss", false).is_none(),
            "miss = None（回退链下沉）"
        );
        // None 占位入缓存：二次 load 不再触文件系统（幂等，语义同 native_icon）。
        assert!(load("iconfile:zz-test-miss", false).is_none());
        // 非 iconfile 串不经缓存直接 None。
        assert!(load("lucide:app-window", true).is_none());
    }
    /// PLAN-018-FU7：目标档重采样——尺寸正确 + 边缘覆盖保持平滑
    ///（半透明像素数显著高于点采样）+ 原样档不放大。
    #[test]
    fn decode_resized_downscales_with_smooth_edges() {
        // 合成 64×64：左 8 列 alpha 斜坡（模拟 AA 边）+ 右侧不透明。
        let mut img = image::RgbaImage::new(64, 64);
        for y in 0..64 {
            for x in 0..64 {
                let a = if x < 8 { (x as f32 / 8.0 * 255.0) as u8 } else { 255 };
                img.put_pixel(x, y, image::Rgba([200, 120, 60, a]));
            }
        }
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut png, image::ImageFormat::Png)
            .expect("encode png");
        let bytes = png.into_inner();

        // 缩小档：16×16，边界带应保留半透明覆盖（点采样会丢失）。
        let (w, h, rgba) = decode_resized(&bytes, 16).expect("decode 16");
        assert_eq!((w, h), (16, 16));
        let partial = rgba
            .chunks_exact(4)
            .filter(|px| px[3] > 0 && px[3] < 250)
            .count();
        assert!(partial >= 8, "16px 档边缘半透明覆盖 ≥8px，实测 {partial}");

        // 原样档（≥ 源）：不放大不重采样。
        let (w2, h2, _) = decode_resized(&bytes, 512).expect("decode native");
        assert_eq!((w2, h2), (64, 64));
    }
}
