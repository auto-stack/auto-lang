//! PLAN-012 W3：整桌面等比预览（`workspace_preview` 布局件）宿主数据面。
//!
//! DSL 无重叠布局（desktop.at:13 注"z 序宿主侧兑现"），分区卡片的整桌面
//! 合成必须宿主 widget（PLAN-012 §5 W3-2）。数据链直连 wm + snapshot 缓存
//! （`sync_shell_windows` 发布 [`publish`]，渲染臂消费 [`current`]）——
//! **协议零字段增量**（SD-02：widget 入 DSL 合同面清单）。
//!
//! - [`PreviewTile`]：窗口在桌面可用区（usable）内的逻辑像素矩形；
//! - [`Published`]：全部分区 tile 集 + usable 尺寸 + 壁纸基色（#hex 解析，
//!   图片路径 = None → 渲染臂主色占位）；
//! - [`tile_rect`]：Contain 等比贴合纯函数（盒内居中，可单测钉死几何）。

use std::collections::BTreeMap;
use std::sync::Mutex;

/// 单窗 tile：usable 区内逻辑像素矩形（左上原点）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PreviewTile {
    pub wid: u64,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// 渲染臂消费快照（publish 整体替换；渲染臂 clone 读）。
#[derive(Debug, Clone, Default)]
pub struct Published {
    /// 桌面可用区（viewport 扣任务栏，`usable_rect` 同款）逻辑像素。
    pub usable: (f32, f32),
    /// 壁纸基色（config.wallpaper_path `#rrggbb` 直铺；None = 主色占位）。
    pub wallpaper: Option<(u8, u8, u8)>,
    /// 分区 id（"0"/"1"/…）→ z 序 tile 集（常驻隐藏窗已排除）。
    pub workspaces: BTreeMap<String, Vec<PreviewTile>>,
    /// 2026-09-22：解析后的壁纸 spec（`#hex` | `builtin:` | 图片路径，
    /// `desktop_wallpaper` 同源镜像）——预览底层直绘真壁纸（非 `#` 时
    /// 渲染臂走 `desktop_wallpaper_element` 图片臂；空 = 旧行为色底）。
    pub wallpaper_src: String,
}

static CURRENT: Mutex<Option<Published>> = Mutex::new(None);

/// 宿主发布（sync_shell_windows 每次同步整体覆盖）。
pub fn publish(p: Published) {
    if let Ok(mut slot) = CURRENT.lock() {
        *slot = Some(p);
    }
}

/// 渲染臂读取（clone 出——锁内短临界，避免持锁渲染）。
pub fn current() -> Option<Published> {
    CURRENT.lock().ok().and_then(|s| s.clone())
}

/// 壁纸 `#rrggbb` 基色解析（图片路径/空值 = None；`#` 缺席或非 hex = None）。
pub fn wallpaper_rgb(path: &str) -> Option<(u8, u8, u8)> {
    let hex = path.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some((byte(0)?, byte(2)?, byte(4)?))
}

/// Contain 等比贴合纯函数：usable 矩形按比例缩进 `box_w × box_h` 盒内
/// 居中，返回 tile 的盒内像素矩形 (x, y, w, h)。盒或 usable 非正 = 原点
/// 空矩形（守卫，不 panic）。
pub fn tile_rect(
    tile: &PreviewTile,
    usable: (f32, f32),
    box_w: f32,
    box_h: f32,
) -> (f32, f32, f32, f32) {
    let (uw, uh) = usable;
    if uw <= 0.0 || uh <= 0.0 || box_w <= 0.0 || box_h <= 0.0 {
        return (0.0, 0.0, 0.0, 0.0);
    }
    let scale = (box_w / uw).min(box_h / uh);
    let off_x = (box_w - uw * scale) / 2.0;
    let off_y = (box_h - uh * scale) / 2.0;
    (
        off_x + tile.x * scale,
        off_y + tile.y * scale,
        tile.w * scale,
        tile.h * scale,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// PLAN-012 §6 W3：合成几何纯函数单测——1280×720 usable 贴 176×66 盒
    /// （scale = min(0.1375, 0.091666) = 0.091666，受纵向约束），tile 等
    /// 比换算 + 盒内居中。
    #[test]
    fn tile_rect_contain_fit_and_centering() {
        let usable = (1280.0_f32, 720.0_f32);
        let (bw, bh) = (176.0_f32, 66.0_f32);
        let scale = 66.0_f32 / 720.0; // 0.091666…（Contain 受短边约束）
        // 全桌面 tile → 117.33×66，横向居中（off_x = (176-117.33)/2）。
        let full = PreviewTile { wid: 1, x: 0.0, y: 0.0, w: 1280.0, h: 720.0 };
        let (x, y, w, h) = tile_rect(&full, usable, bw, bh);
        assert!((w - 1280.0 * scale).abs() < 0.01, "w 受 scale: {w}");
        assert!((h - 66.0).abs() < 0.01, "纵向贴满: {h}");
        assert!((x - (bw - w) / 2.0).abs() < 0.01, "横向居中: {x}");
        assert!((y - 0.0).abs() < 0.01);
        // 左半窗（0,0,640,720）→ 紧贴居中块左缘，纵向满高。
        let left = PreviewTile { wid: 2, x: 0.0, y: 0.0, w: 640.0, h: 720.0 };
        let (x, _, w, h) = tile_rect(&left, usable, bw, bh);
        assert!((x - (bw - 1280.0 * scale) / 2.0).abs() < 0.01, "{x}");
        assert!((w - 640.0 * scale).abs() < 0.01, "{w}");
        assert!((h - 66.0).abs() < 0.01);
        // 宽盒（Contain 受横向约束时纵向留白居中）。
        let (x, y, w, _) = tile_rect(&full, usable, 400.0, 72.0);
        // scale = min(400/1280, 72/720) = 0.1 → w=128，off_x = (400-128)/2 = 136。
        assert!((w - 128.0).abs() < 0.01, "{w}");
        assert!((x - 136.0).abs() < 0.01, "{x}");
        // off_y = (72-72)/2 = 0。
        assert!((y - 0.0).abs() < 0.01, "{y}");
    }

    /// 退化守卫：零尺寸盒/usable 不 panic、返回空矩形。
    #[test]
    fn tile_rect_guards_zero_dims() {
        let t = PreviewTile { wid: 1, x: 1.0, y: 1.0, w: 10.0, h: 10.0 };
        assert_eq!(tile_rect(&t, (0.0, 100.0), 100.0, 100.0), (0.0, 0.0, 0.0, 0.0));
        assert_eq!(tile_rect(&t, (100.0, 100.0), 0.0, 100.0), (0.0, 0.0, 0.0, 0.0));
    }

    /// 壁纸基色解析：#hex 三态（好值/非 hex/图片路径）。
    #[test]
    fn wallpaper_rgb_parse() {
        assert_eq!(wallpaper_rgb("#12abFF"), Some((0x12, 0xab, 0xff)));
        assert_eq!(wallpaper_rgb("#xyz123"), None);
        assert_eq!(wallpaper_rgb("C:/wp/sea.png"), None);
        assert_eq!(wallpaper_rgb(""), None);
    }
}
