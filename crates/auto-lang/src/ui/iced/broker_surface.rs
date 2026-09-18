//! Plan 500 步骤 5 —— broker 表面两态合成（宿主渲染臂）。
//!
//! 虚拟窗客户区内容源：**broker client 的帧**（进程外 App 经协议上报）
//! 取代宿主本地重渲染（v1.2 的临时形态——host 侧 dynamic_view 同源重画，
//! child 帧仅消息级断言；本模块兑现"live-iced 消费"，host.rs 头注遗留点）。
//!
//! 两态（`Welcome.frame_mode` 协商）：
//! - **Commands（queue 臂）**：`DrawList` → canvas Program 降级——Quad =
//!   抗锯齿 fill、Text = `fill_text` 宿主侧 shaping（D1 定案 A：宿主
//!   iced 文本栈，零新依赖）、clear = 底色。damage v1.3 作重绘提示
//!   （每帧全量重建几何，正确性不受损；Cache 局部化归 Stage 5）。
//! - **Pixels（independent 臂）**：shm RGBA → `image::Handle::from_rgba`
//!   上传（497 快照同通道口径）→ Image 挂客户区。

use crate::ui::desktop_protocol::message::{DrawList, DrawOp, Rgba8};
use crate::ui::desktop_protocol::stage3::PixelsSurface;
use crate::ui::session::{DesktopMessage, DesktopSession, Wid};

fn to_color(c: Rgba8) -> iced::Color {
    iced::Color::from_rgba8(c.r, c.g, c.b, c.a as f32 / 255.0)
}

/// CSS 字重刻度（100..900）→ iced `Weight` 档（Plan 515 G2）。
fn css_weight_to_iced(w: u16) -> iced::font::Weight {
    match w {
        100 => iced::font::Weight::Thin,
        200 => iced::font::Weight::ExtraLight,
        300 => iced::font::Weight::Light,
        500 => iced::font::Weight::Medium,
        600 => iced::font::Weight::Semibold,
        700 => iced::font::Weight::Bold,
        800 => iced::font::Weight::ExtraBold,
        900 => iced::font::Weight::Black,
        _ => iced::font::Weight::Normal,
    }
}

/// DrawList → canvas 绘制程序（queue 臂栅格化：宿主 GPU 抗锯齿）。
struct DrawListPainter {
    list: DrawList,
}

// ---------------------------------------------------------------------------
// PLAN-028 图像通道 —— DrawOp::Image 宿主侧解析/缓存/降级（D1–D4 定案）。
// ---------------------------------------------------------------------------

use std::collections::{HashMap, HashSet};
use std::sync::{Mutex, OnceLock};

type ImageHandle = iced::widget::image::Handle;

/// src → Handle 进程级缓存（解码一次防每帧重上传；key = src——v1 无
/// radius/filter 维度，D1）。`None` = 已定失败负缓存（降级占位 + 观测
/// 去重的依据；http 负缓存与 `load_image_bytes` 双层独立）。
fn handle_cache() -> &'static Mutex<HashMap<String, Option<ImageHandle>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<ImageHandle>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// http(s) 在途集：后台解码去重（同 src 至多一线程；失败由负缓存止位，
/// 不重试风暴）。
fn http_inflight() -> &'static Mutex<HashSet<String>> {
    static INFLIGHT: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    INFLIGHT.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 未解析观测去重集（D4：同 src 首败打一行，后续静默——防每帧刷行）。
fn observed_unresolved() -> &'static Mutex<HashSet<String>> {
    static OBSERVED: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    OBSERVED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// 观测双落（remote.rs enable_remote_ws 先例同款：stderr + ui_console）。
fn observe(line: &str) {
    let full = format!("[drawlist-image] {line}");
    eprintln!("{full}");
    crate::vm::ui_console::ui_console_push(&full);
}

fn observe_unresolved(src: &str) {
    let fresh = observed_unresolved().lock().unwrap().insert(src.to_string());
    if fresh {
        observe(&format!("unresolved src (placeholder fallback): {src}"));
    }
}

/// 编码字节 → Handle（`from_bytes`：解码/上传在 iced 渲染器侧按 Handle
/// 身份去重——child 免解码的"轻 child"哲学在宿主侧的镜像）。
fn decode_handle(bytes: &[u8]) -> Option<ImageHandle> {
    if bytes.is_empty() {
        return None;
    }
    Some(ImageHandle::from_bytes(bytes.to_vec()))
}

/// DrawOp::Image src 解析总入口（D2 词汇表；paint 路径调用——禁阻塞）。
/// pub(crate)：p028_image_arm e2e 宿主侧解析/降级腿与度量直接驱动。
pub(crate) fn resolve_drawlist_image(src: &str) -> Option<ImageHandle> {
    // thumbnail://{wid} 虚拟引用（D3）：快照缓存直查，不进永久 Handle
    // 缓存——SWR 刷新语义，冻结句柄会锁死旧图（WindowThumbnail 消费臂
    // from_rgba 每帧重建同律）。
    if let Some(rest) = src.strip_prefix("thumbnail://") {
        return resolve_thumbnail(src, rest);
    }
    if let Some(entry) = handle_cache().lock().unwrap().get(src) {
        return entry.clone();
    }
    if src.starts_with("http://") || src.starts_with("https://") {
        // D1：占位先行——paint 路径禁阻塞（reqwest 3s 同步会把渲染线程
        // 挂满 3s，os-007 P534-D4 教训）；后台线程复用 load_image_bytes
        //（自带进程缓存含负缓存）→ 落缓存 → 翻真由宿主任一后续重绘兑现
        //（鼠标/toast tick/MCP 心跳/帧泵驱动，零新增触发器）。
        if http_inflight().lock().unwrap().insert(src.to_string()) {
            observe(&format!("http fetch started: {src}"));
            let owned = src.to_string();
            std::thread::spawn(move || {
                let handle = crate::ui::iced::renderer::load_image_bytes(&owned)
                    .and_then(|bytes| decode_handle(&bytes));
                let hit = handle.is_some();
                handle_cache().lock().unwrap().insert(owned.clone(), handle);
                http_inflight().lock().unwrap().remove(&owned);
                if hit {
                    observe(&format!("http image ready: {owned}"));
                }
            });
        }
        return None;
    }
    // 本地词汇（文件 / builtin: / data: / media 票据）：同步快路径
    //（本地字节，无网络等待）。
    let handle = crate::ui::iced::renderer::load_image_bytes(src)
        .and_then(|bytes| decode_handle(&bytes));
    if handle.is_none() {
        observe_unresolved(src);
    }
    handle_cache().lock().unwrap().insert(src.to_string(), handle.clone());
    handle
}

/// `thumbnail://{wid}` 解析（D3）：命中（含过期）→ `from_rgba` 直绘；
/// 过期 → request_capture 静默重抓（SWR）；真 miss → request_capture +
/// 占位当帧，重抓 cache_put 落地后下帧翻真。wid 非法 = 未解析降级。
fn resolve_thumbnail(src: &str, rest: &str) -> Option<ImageHandle> {
    use crate::ui::iced::snapshot;
    use crate::ui::session::Wid;
    let Some(wid) = rest.parse::<u64>().ok().map(Wid) else {
        observe_unresolved(src);
        return None;
    };
    match snapshot::snapshot_window_stale(wid) {
        Some((snap, fresh)) => {
            if !fresh {
                snapshot::request_capture(wid);
            }
            Some(ImageHandle::from_rgba(snap.w, snap.h, snap.rgba))
        }
        None => {
            snapshot::request_capture(wid);
            observe_unresolved(&format!("{src} (capture requested)"));
            None
        }
    }
}

impl iced::widget::canvas::Program<DesktopMessage> for DrawListPainter {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas::{Frame, Path, Text};
        let mut frame = Frame::new(renderer, bounds.size());
        // clear 底色：整面铺（None = 透明，窗体容器底透出）。
        if let Some(clear) = self.list.clear {
            frame.fill_rectangle(
                iced::Point::ORIGIN,
                bounds.size(),
                to_color(clear),
            );
        }
        paint_ops(&mut frame, &self.list.ops);
        let _ = Path::new(|_| {});
        vec![frame.into_geometry()]
    }
}

/// Plan 515 G1 —— scissor 栈栅格化：`Scissor` 起一段 `with_clip`（匹配
/// pop 之间的 op 裁剪到矩形内；嵌套 push 自然取交——draft/paste 的组合
/// 裁剪语义）。空栈 pop / 未闭合 push（编码端违约）宽容不炸：pop =
/// no-op，未闭合 = 裁到序列尾。
fn paint_ops(frame: &mut iced::widget::canvas::Frame, ops: &[DrawOp]) {
    use iced::widget::canvas::Text;
    let mut i = 0;
    while i < ops.len() {
        match &ops[i] {
            DrawOp::Scissor { rect } => {
                // 深度扫描找配对 pop（含嵌套层）。
                let mut depth = 1usize;
                let mut end = ops.len();
                for (j, op) in ops.iter().enumerate().take(ops.len()).skip(i + 1) {
                    match op {
                        DrawOp::Scissor { .. } => depth += 1,
                        DrawOp::ScissorPop => {
                            depth -= 1;
                            if depth == 0 {
                                end = j;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                let region = iced::Rectangle::new(
                    iced::Point::new(rect.x, rect.y),
                    iced::Size::new(rect.w.max(0.0), rect.h.max(0.0)),
                );
                frame.with_clip(region, |f| paint_ops(f, &ops[i + 1..end]));
                // 跳过配对 pop（未闭合时 end = ops.len()，循环自然收）。
                i = end + 1;
            }
            // 本层游离 pop（编码端违约）= no-op。
            DrawOp::ScissorPop => i += 1,
            DrawOp::Quad { rect, color } => {
                // widget 本地坐标 → canvas 原点平移（越界面出 canvas
                // 自动裁剪）。
                let at = iced::Point::new(rect.x, rect.y);
                frame.fill_rectangle(
                    at,
                    iced::Size::new(rect.w, rect.h),
                    to_color(*color),
                );
                i += 1;
            }
            DrawOp::Text { x, y, size, line_height, color, text } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute(
                        (*line_height).into(),
                    ),
                    ..Default::default()
                });
                i += 1;
            }
            // Plan 515 G2 —— typography 差分：weight/style 映射 iced Font
            //（宿主字体栈按 face 选择——cosmic-text 家族回退取最接近档）。
            DrawOp::TextStyled { x, y, size, line_height, color, weight, italic, text } => {
                frame.fill_text(Text {
                    content: text.clone(),
                    position: iced::Point::new(*x, *y),
                    color: to_color(*color),
                    size: (*size).into(),
                    line_height: iced::widget::text::LineHeight::Absolute(
                        (*line_height).into(),
                    ),
                    font: iced::Font {
                        weight: css_weight_to_iced(*weight),
                        style: if *italic {
                            iced::font::Style::Italic
                        } else {
                            iced::font::Style::Normal
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                });
                i += 1;
            }
            // PLAN-028 图像通道（D1–D4）：src 引用 → 宿主侧解析（词汇 +
            // 进程级缓存 + thumbnail:// 虚拟引用）→ `Frame::draw_image`
            // 仓内首用；未解析降级 = 占位 Quad + 观测行（I3 禁静默错绘）。
            DrawOp::Image { rect, src, .. } => {
                let at = iced::Point::new(rect.x, rect.y);
                let size =
                    iced::Size::new(rect.w.max(0.0), rect.h.max(0.0));
                match resolve_drawlist_image(src) {
                    Some(handle) => frame.draw_image(
                        iced::Rectangle::new(at, size),
                        &handle,
                    ),
                    None => frame.fill_rectangle(
                        at,
                        size,
                        to_color(
                            crate::ui::desktop_protocol::client_runtime::IMAGE_PLACEHOLDER,
                        ),
                    ),
                }
                i += 1;
            }
        }
    }
}

/// queue 臂内容：DrawList → canvas 元素（Fill×Fill 客户区）。
pub fn drawlist_element(list: &DrawList) -> iced::Element<'_, DesktopMessage> {
    iced::widget::canvas(
        DrawListPainter { list: list.clone() },
    )
    .width(iced::Length::Fill)
    .height(iced::Length::Fill)
    .into()
}

/// independent 臂内容：RGBA 前缓冲 → Image（`from_rgba` 直接纳 straight
/// 非预乘；预乘换算在 iced 渲染器内部，协议层不感知）。
pub fn pixels_element(surface: &PixelsSurface) -> iced::Element<'_, DesktopMessage> {
    let handle = iced::widget::image::Handle::from_rgba(
        surface.w,
        surface.h,
        surface.rgba.clone(),
    );
    iced::widget::image(handle)
        .width(iced::Length::Fill)
        .height(iced::Length::Fill)
        .into()
}

/// 虚拟窗 wid → broker 表面内容（非 broker 窗 = None，调用方走本地
/// dynamic_view 既有路径）。
pub fn broker_client_content(
    state: &DesktopSession,
    wid: Wid,
) -> Option<iced::Element<'_, DesktopMessage>> {
    let client = state
        .broker_clients
        .values()
        .find(|c| c.wid == Some(wid))?;
    // 像素前缓冲优先（independent 臂），回退命令帧（queue 臂）。
    if let Some(px) = client.composed_pixels() {
        return Some(pixels_element(px));
    }
    client.composed().map(drawlist_element)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 1×1 PNG（合法编码字节——from_bytes 不即解码，宿主渲染器按 Handle
    /// 身份懒解码；单测断言解析/缓存/降级分派，真栅格归 e2e 截图腿）。
    const TINY_PNG_DATA_URI: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";

    /// T-03：data: 词汇解析 → Handle；二次调用缓存命中（同 Handle 实例）。
    #[test]
    fn t028_data_uri_resolves_and_caches() {
        let h1 = resolve_drawlist_image(TINY_PNG_DATA_URI);
        assert!(h1.is_some(), "data: 解码命中");
        let h2 = resolve_drawlist_image(TINY_PNG_DATA_URI);
        assert_eq!(h1, h2, "二次调用 = 缓存命中（同 Handle）");
    }

    /// T-03：builtin: 内嵌壁纸词汇（与 518 壁纸线同源）。
    #[test]
    fn t028_builtin_wallpaper_resolves() {
        assert!(resolve_drawlist_image("builtin:ricepaper").is_some());
    }

    /// T-03/D4：未解析降级——缺文件/未知 scheme/字形词汇（lucide not-yet，
    /// P026-D1 后半维持）/空 src 全部 None + 负缓存落位（防每帧重读与
    /// 观测重刷）。
    #[test]
    fn t028_unresolved_negative_cache_and_notyet_lexicon() {
        let missing = "Z:/definitely/missing-028.png";
        assert!(resolve_drawlist_image(missing).is_none(), "缺文件 None");
        assert!(
            handle_cache().lock().unwrap().contains_key(missing),
            "负缓存落位（占位 + 观测去重依据）"
        );
        assert!(resolve_drawlist_image("foo://bar").is_none(), "未知 scheme");
        assert!(resolve_drawlist_image("lucide:home").is_none(), "字形词汇 not-yet");
        assert!(resolve_drawlist_image("").is_none(), "空 src 容错");
    }

    /// T-03/D1：http miss 占位先行（不阻塞 paint）→ 后台解码落缓存。
    /// 载体 = 回环拒绝端口（快速失败，无网络依赖）。
    #[test]
    fn t028_http_placeholder_first_then_background_fill() {
        let src = "http://127.0.0.1:1/zero28.png";
        assert!(resolve_drawlist_image(src).is_none(), "首帧占位（无阻塞等待）");
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if handle_cache().lock().unwrap().contains_key(src) {
                break;
            }
            assert!(std::time::Instant::now() < deadline, "后台解码未落缓存");
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(
            handle_cache().lock().unwrap().get(src).unwrap().is_none(),
            "连接拒绝 = 负缓存（不重试风暴）"
        );
    }

    /// T-04/D3：thumbnail:// 三路径——真 miss（占位 + request_capture
    /// 入队）/ 新鲜命中（直绘零重抓）/ SWR 过期（续绘 + 静默重抓）。
    #[test]
    fn t028_thumbnail_miss_hit_and_swr() {
        use crate::ui::iced::snapshot::{self, WindowSnapshot};
        use crate::ui::session::Wid;

        // 真 miss：当帧占位 + 抓取请求入队（宿主 update 排空）。
        let miss_wid = Wid(428_001);
        let _ = snapshot::take_capture_requests(); // 清场
        let src = format!("thumbnail://{}", miss_wid.0);
        assert!(resolve_drawlist_image(&src).is_none(), "miss 当帧占位");
        assert!(
            snapshot::take_capture_requests().iter().any(|w| *w == miss_wid),
            "miss 触发 request_capture"
        );

        // 新鲜命中：缓存注入替身 → Handle 直出，零新增抓取请求。
        let hit_wid = Wid(428_002);
        let src = format!("thumbnail://{}", hit_wid.0);
        snapshot::cache_put(
            hit_wid,
            WindowSnapshot { rgba: vec![1, 2, 3, 255], w: 1, h: 1 },
        );
        let _ = snapshot::take_capture_requests(); // 清场
        assert!(resolve_drawlist_image(&src).is_some(), "命中直绘");
        assert!(snapshot::take_capture_requests().is_empty(), "新鲜命中零重抓");

        // SWR：过期条目续绘（不跌占位）+ request_capture 静默重抓。
        let swr_wid = Wid(428_003);
        let src = format!("thumbnail://{}", swr_wid.0);
        snapshot::cache_put(
            swr_wid,
            WindowSnapshot { rgba: vec![9, 9, 9, 255], w: 1, h: 1 },
        );
        snapshot::__test_backdate(swr_wid);
        let _ = snapshot::take_capture_requests(); // 清场
        assert!(resolve_drawlist_image(&src).is_some(), "过期条目续绘");
        assert!(
            snapshot::take_capture_requests().iter().any(|w| *w == swr_wid),
            "过期触发静默重抓"
        );

        // wid 非法 = 未解析降级。
        assert!(resolve_drawlist_image("thumbnail://not-a-wid").is_none());
    }
}
