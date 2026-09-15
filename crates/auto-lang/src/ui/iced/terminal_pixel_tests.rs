//! PLAN-010 T8 → PLAN-634 T-01: terminal 组件像素级 headless 自动化
//! (layout_tests 同款 `iced_test::simulator` 管线,Plan 414 §8.2 先例)。
//!
//! 断言面(634 稳健化):像素金样的**逐字节**比对把环境敏感渲染面(wgpu
//! MSAA×4 合成、适配器枚举、系统字体栅格)钉进了门禁——015 收尾实测同日
//! 晨绿午后红、bisect 定案非代码(f0dc16732 洁净树复红),634 执行期探针
//! 进一步证实 vulkan/dx12/gl 三后端在本机均逐字节复现金样(漂移是环境态
//! 翻转,非后端字符串可复现),归因与再生成配方见
//! `docs/plans/evidence/634/`。故断言改为环境无关的语义契约:
//!
//! 1. bounds 非零且等于 cols×CELL_W + 2×PAD / rows×CELL_H + 2×PAD(widget
//!    经 `operate` 向 selector 暴露 bounds——canvas 型 widget 缺省不可见);
//! 2. 基线帧文本层入像素:亮(墨水)像素占比显著高于纯底色(字体无关);
//! 3. 渲染确定性:同进程两次独立渲染逐字节一致(噪声基线;同环境同适配器,
//!    字节级成立);
//! 4. 选中态/光标帧相对基线产生容差像素差(高亮/光标层进入渲染产物);
//! 5. 金样留档:首跑自建;此后按容差比对,超预算**仅告警**并给出再生成
//!    配方(留档与粗回归探测,不再作为硬门禁)。
//!
//! Run with:
//! `cargo test -p auto-lang --features ui-iced,iced-layout-tests --lib terminal_pixel`
#![cfg(all(test, feature = "iced-layout-tests"))]

use crate::ui::iced::renderer::IntoIcedElement;
use crate::ui::terminal::{
    terminal, terminal_feed, terminal_selection_begin, terminal_selection_clear,
    terminal_selection_extend, terminal_selection_finish, terminal_set_cursor,
    TermCursorShape, TermSelectionType,
};
use crate::ui::view::View;
use iced_test::simulator;
use iced_test::selector::{Candidate, Selector};

const KEY: &str = "t8px";
const COLS: u16 = 40;
const ROWS: u16 = 10;

/// 容差像素差分的单通道容差(0-255)。环境漂移(MSAA/适配器)典型表现为
/// 边缘若干灰阶的抖动,≤8 视为同像素;高亮带/光标块的信号远超此粒度。
const CHANNEL_TOL: i32 = 8;
/// 金样告警预算:实测帧与金样容差差分超过该比例时 eprintln 告警(非致命)。
/// 字体替换级漂移通常 >20%;同环境应为 0。
const GOLDEN_DRIFT_BUDGET: f64 = 0.05;
/// 基线文本层墨水占比下限(634 实测金样校准:双行文本+基线光标 =
/// 0.153%;文本缺失时仅剩光标块 ≈0.018%)。字体替换只改字形分布,
/// 不改量级,故环境无关。
const TEXT_INK_FLOOR: f64 = 0.0005;
/// 选中层相对基线的容差差分下限(634 新帧实测:信号 ≈0.212%;缺层残差
/// = 缺省光标位移 ≈0.036%,须判红)。
const SELECTION_DIFF_FLOOR: f64 = 0.0005;
/// 光标层相对基线的容差差分下限(634 新帧实测:光标位移信号本身即
/// ≈0.036%——两枚块位置互换;缺层时帧与基线逐字节同,差分=0。同进程
/// 渲染字节确定,无噪声带)。
const CURSOR_DIFF_FLOOR: f64 = 0.0001;

fn fixed_lines() -> Vec<String> {
    let mut lines = vec![
        "hello world".to_string(),
        "second row of the pixel fixture".to_string(),
    ];
    lines.resize(ROWS as usize, String::new());
    lines
}

/// 固定文本行 View(每次调用重新构造;registry 按 key 复用同一 core)。
fn terminal_view() -> View<()> {
    View::Terminal {
        key: KEY.to_string(),
        cols: COLS,
        rows: ROWS,
        lines: fixed_lines(),
        scroll_offset: 0,
        preedit: None,
        scheme: crate::ui::terminal::TERMINAL_SCHEME_FOLLOW_THEME,
        on_select: None,
        on_menu: None,
        on_input: None,
        cursor_row: 0,
        cursor_col: 0,
        style: None,
    }
}

/// 重置注册表核心到「无选中 + 光标块在 (1,0)」基线并喂入固定文本。
fn feed_baseline() {
    let core = terminal(KEY, COLS, ROWS);
    terminal_feed(core, &fixed_lines());
    terminal_set_cursor(core, 1, 0, TermCursorShape::Block);
    terminal_selection_clear(core);
}

/// 收集布局树里所有容器 bounds 的 selector(terminal widget 经 operate
/// 把自身 bounds 以 container 形态暴露)。
#[derive(Clone)]
struct BoundsCollector(std::sync::Arc<std::sync::Mutex<Vec<(f32, f32)>>>);
impl Selector for BoundsCollector {
    type Output = ();
    fn select(&mut self, candidate: Candidate<'_>) -> Option<()> {
        if let Candidate::Container { bounds, .. } = candidate {
            self.0
                .lock()
                .unwrap()
                .push((bounds.width, bounds.height));
        }
        None
    }
    fn description(&self) -> String {
        "terminal-pixel-bounds-collector".into()
    }
}

/// bounds 非零且等于固定网格几何(cols×cell_w()+2×PAD, rows×CELL_H+2×PAD)。
/// 014:横向用实测 advance——先预热测量(否则首帧布局还是 8.0 近似,
/// find 看到的 bounds 与 want_w 不同源)。
#[test]
fn terminal_pixel_bounds_nonzero_and_exact() {
    use crate::ui::terminal::iced::{CELL_H, PAD, cell_w};
    let want_w = COLS as f32 * cell_w() + 2.0 * PAD;
    let want_h = ROWS as f32 * CELL_H + 2.0 * PAD;
    feed_baseline();
    let mut ui = simulator(terminal_view().into_iced());
    let store = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let _ = ui.find(BoundsCollector(store.clone()));
    let bounds = store.lock().unwrap().clone();
    assert!(
        bounds.iter().any(|&(w, h)| (w - want_w).abs() < 0.6 && (h - want_h).abs() < 0.6),
        "terminal 组件 bounds 必须命中 {want_w}x{want_h}(±0.6),实际 {bounds:?}"
    );
}

// —— 帧取证工具(634 T-01):解码、容差差分、墨水占比、金样审计 ——

/// 一帧解码后的 RGBA 产物(matches_image 落盘 PNG 是无损往返)。
struct Frame {
    rgba: Vec<u8>,
    width: u32,
    height: u32,
}

fn temp_png(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("auto-lang-terminal-pixel");
    std::fs::create_dir_all(&dir).expect("temp dir");
    // nextest 每用例独立进程:pid 隔离避免并发进程互踩同名临时帧。
    dir.join(format!("{}_{}.png", tag, std::process::id()))
}

/// matches_image 落盘时按渲染器名给路径追加后缀(如 `-wgpu`)且不回传
/// 实际路径——统一走「落盘 → 按前缀 glob 解析 → 解码」取证。
fn write_frame(snap: &iced_test::simulator::Snapshot, tag: &str) -> Frame {
    let base = temp_png(tag);
    let _ = snap.matches_image(&base).expect("frame 落盘");
    let prefix = format!("{}_{}-", tag, std::process::id());
    let dir = base.parent().expect("temp parent");
    let written = std::fs::read_dir(dir)
        .expect("temp dir")
        .filter_map(|e| e.ok())
        .find_map(|e| {
            let n = e.file_name().to_string_lossy().to_string();
            (n.starts_with(&prefix) && n.ends_with(".png")).then(|| e.path())
        })
        .expect("matches_image 落盘路径");
    decode_rgba(&written)
}

fn decode_rgba(path: &std::path::Path) -> Frame {
    let file = std::fs::File::open(path).expect("open png");
    let mut decoder = png::Decoder::new(std::io::BufReader::new(file));
    let mut reader = decoder.read_info().expect("png info");
    let mut bytes = vec![0; reader.output_buffer_size().expect("frame buffer size")];
    let info = reader.next_frame(&mut bytes).expect("png frame");
    bytes.truncate(info.buffer_size());
    Frame {
        rgba: bytes,
        width: info.width,
        height: info.height,
    }
}

/// 容差差分占比:单通道差 >CHANNEL_TOL 的像素比例(两帧尺寸必须一致)。
fn diff_fraction(a: &Frame, b: &Frame) -> f64 {
    assert_eq!(a.rgba.len(), b.rgba.len(), "帧尺寸必须一致");
    let px = a.rgba.len() / 4;
    let diff = (0..px)
        .filter(|&i| {
            [0, 1, 2].iter().any(|&c| {
                (a.rgba[i * 4 + c] as i32 - b.rgba[i * 4 + c] as i32).abs() > CHANNEL_TOL
            })
        })
        .count();
    diff as f64 / px as f64
}

/// 墨水占比:亮度(通道和)>480 的像素比例。终端底色近黑(通道和 <60),
/// 文本/光标为亮前景——字体替换只改字形分布,不改量级,故环境无关。
fn ink_fraction(f: &Frame) -> f64 {
    let px = f.rgba.len() / 4;
    let ink = (0..px)
        .filter(|&i| {
            f.rgba[i * 4] as u32 + f.rgba[i * 4 + 1] as u32 + f.rgba[i * 4 + 2] as u32 > 480
        })
        .count();
    ink as f64 / px as f64
}

fn golden_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/ui/terminal_pixel")
}

fn golden_path(name: &str, kind: &str) -> std::path::PathBuf {
    golden_dir().join(format!("terminal_pixel_{name}_{kind}.png"))
}

/// 金样实际落盘路径:matches_image 按渲染器名追加后缀(如 `-wgpu`),
/// 而 Snapshot 不暴露渲染器名——按前缀 glob 解析已有金样。
fn resolve_golden(name: &str, kind: &str) -> Option<std::path::PathBuf> {
    let prefix = format!("terminal_pixel_{name}_{kind}-");
    std::fs::read_dir(golden_dir())
        .ok()?
        .filter_map(|e| e.ok())
        .find_map(|e| {
            let fname = e.file_name().to_string_lossy().to_string();
            (fname.starts_with(&prefix) && fname.ends_with(".png")).then(|| e.path())
        })
}

/// 金样审计(留档 + 粗回归探测,非硬门禁):缺失则自建(首跑留档);已有
/// 则容差比对,超预算仅告警并附再生成配方——逐字节硬比对正是 015 收尾
/// 2 红的根因(环境态翻转,非代码回归),语义门禁由墨水/差分断言承担。
fn golden_audit(snap: &iced_test::simulator::Snapshot, name: &str, kind: &str) {
    if resolve_golden(name, kind).is_none() {
        let created = snap.matches_image(golden_path(name, kind)).expect("golden 自建");
        assert!(created, "金样首跑自建必须成功: {name}/{kind}");
        return;
    }
    let golden = resolve_golden(name, kind).expect("golden 已存在");
    let actual = write_frame(snap, &format!("{name}_{kind}_audit"));
    let d = diff_fraction(&decode_rgba(&golden), &actual);
    if d > GOLDEN_DRIFT_BUDGET {
        eprintln!(
            "[terminal_pixel] 金样漂移 {:.1}% 超预算 {:.0}%\
             (环境敏感面: wgpu MSAA/系统字体栅格;非代码回归。再生成: 删除\
             test/ui/terminal_pixel/terminal_pixel_{name}_{kind}-*.png 后重跑;\
             归因: docs/plans/evidence/634/)",
            d * 100.0,
            GOLDEN_DRIFT_BUDGET * 100.0
        );
    }
}

/// 渲染基线帧并跑三项环境无关断言:墨水占比(文本层入像素)、同进程
/// 双渲染字节一致(确定性噪声基线)、金样审计。返回基线帧供差分。
fn baseline_frame(tag: &str) -> Frame {
    // 预热实测字距:首帧渲染中测量会翻转 cell_w(8.0→实测),两次渲染
    // 逐字节一致性会因此假失败。
    let _ = crate::ui::terminal::iced::cell_w();
    feed_baseline();
    let mut ui = simulator(terminal_view().into_iced());
    let snap1 = ui.snapshot(&iced::Theme::Light).expect("baseline snapshot 1");
    let snap2 = ui.snapshot(&iced::Theme::Light).expect("baseline snapshot 2");

    let frame = write_frame(&snap1, &format!("{tag}_base1"));
    let f2 = temp_png(&format!("{tag}_base2"));
    let _ = snap2.matches_image(&f2).expect("baseline frame 2 落盘");

    let ink = ink_fraction(&frame);
    assert!(
        ink > TEXT_INK_FLOOR,
        "基线帧墨水占比 {:.2}% 低于 {:.1}%——文本层未进入渲染产物",
        ink * 100.0,
        TEXT_INK_FLOOR * 100.0
    );
    // 同进程两次渲染必须逐字节一致(snap1 vs snap2 的落盘产物)。
    let identical = snap1.matches_image(&f2).expect("determinism matches_image");
    assert!(
        identical,
        "同进程两次渲染不一致——渲染不确定,像素断言不可靠"
    );

    golden_audit(&snap1, tag, "base");
    frame
}

/// 选中态:同一核心开启 Simple 选中后,渲染产物必须相对基线产生容差
/// 像素差(选中列带高亮进入像素)。金样 terminal_pixel_selection_frame
/// 首跑自建留档,其后审计制。
#[test]
fn terminal_pixel_selection_changes_pixels() {
    let base = baseline_frame("selection");

    let core = terminal(KEY, COLS, ROWS);
    terminal_selection_begin(core, TermSelectionType::Simple, 0, 0);
    terminal_selection_extend(core, 0, 10);
    terminal_selection_finish(core);

    let mut ui = simulator(terminal_view().into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("selection snapshot");
    let frame = write_frame(&snap, "selection_frame");

    // 关键断言:选中帧必须偏离基线(不然选中高亮没进渲染产物)。
    let d = diff_fraction(&frame, &base);
    assert!(
        d > SELECTION_DIFF_FLOOR,
        "选中帧与基线帧容差差分 {:.3}% 低于 {:.3}%——选中高亮未进入渲染产物",
        d * 100.0,
        SELECTION_DIFF_FLOOR * 100.0
    );

    golden_audit(&snap, "selection", "frame");

    // 清场:核心上的选中不影响其他用例。
    terminal_selection_clear(core);
}

/// 光标块:光标位置/形状写入核心后,渲染产物必须相对基线产生容差像素
/// 差(光标块像素区域存在)。金样 terminal_pixel_cursor_frame 首跑自建
/// 留档,其后审计制。
#[test]
fn terminal_pixel_cursor_changes_pixels() {
    let base = baseline_frame("cursor");

    let core = terminal(KEY, COLS, ROWS);
    terminal_set_cursor(core, 3, 5, TermCursorShape::Block);

    let mut ui = simulator(terminal_view().into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("cursor snapshot");
    let frame = write_frame(&snap, "cursor_frame");

    let d = diff_fraction(&frame, &base);
    assert!(
        d > CURSOR_DIFF_FLOOR,
        "光标帧与基线帧容差差分 {:.3}% 低于 {:.3}%——光标块未进入渲染产物",
        d * 100.0,
        CURSOR_DIFF_FLOOR * 100.0
    );

    golden_audit(&snap, "cursor", "frame");
}

// —— badge/preedit 可见性(PLAN-634 T-02;016 偏离基线法同款)——
//
// 根因同 015 R015-F1:draw 局部段落 fill 后析构,iced wgpu flush 时
// upgrade 失败静默丢弃文字(bg quad 值拷贝不受影响,故旧帧只剩底色块)。
// 断言 = 目标格区域亮像素计数:锚定在空单元格上,基线 ≈0;修复前仅
// quad 无文字也 ≈0(badge 底色块)或负贡献(preedit 蓝底盖字),修复后
// 文字字形进入像素,亮像素显著非零。

/// 矩形区域(物理像素,左闭右开)内的亮像素数。
fn ink_in_rect(f: &Frame, x0: u32, y0: u32, x1: u32, y1: u32) -> usize {
    let stride = f.width as usize;
    let mut ink = 0usize;
    for y in y0 as usize..(y1 as usize).min(f.height as usize) {
        for x in x0 as usize..(x1 as usize).min(stride) {
            let i = (y * stride + x) * 4;
            if i + 2 < f.rgba.len()
                && f.rgba[i] as u32 + f.rgba[i + 1] as u32 + f.rgba[i + 2] as u32 > 480
            {
                ink += 1;
            }
        }
    }
    ink
}

/// 滚动偏移 badge(右上角指示)必须渲染进像素产物:目标区域亮像素
/// 显著非零(修复前只有 pal_bg 底色块,近黑底近黑块,亮像素≈0)。
#[test]
fn terminal_pixel_badge_text_reaches_pixels() {
    use crate::ui::terminal::iced::{CELL_H, PAD, cell_w};

    let _ = crate::ui::terminal::iced::cell_w();
    feed_baseline();

    // badge 帧:scroll_offset=7(渲染 "↑7";核心内容/光标同基线)。
    let mut view = terminal_view();
    if let View::Terminal { scroll_offset, .. } = &mut view {
        *scroll_offset = 7;
    }
    let mut ui = simulator(view.into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("badge snapshot");
    let frame = write_frame(&snap, "badge_frame");

    // badge 几何:右上角,x0 = 宽 - 3×cw(字宽) - cw(右边距)。
    let cw = cell_w();
    let right = COLS as f32 * cw + 2.0 * PAD;
    let scale = 2.0f32; // simulator 硬编码 scale_factor
    let x0 = ((right - 4.0 * cw) * scale) as u32;
    let x1 = (right * scale) as u32;
    let y1 = ((PAD + CELL_H) * scale) as u32;

    let ink = ink_in_rect(&frame, x0, 0, x1, y1);
    assert!(
        ink > 100,
        "badge 区域亮像素 {ink} ≤ 100——滚动偏移指示文字未进入渲染产物\
         (WeakParagraph 同族丢弃;widget.rs fill_cached_para)"
    );
}

/// IME preedit 覆盖层文字必须渲染进像素产物:锚点在空单元格,基线
/// ≈0 亮像素;修复前蓝底 quad 盖场但文字被丢弃,亮像素仍≈0。
#[test]
fn terminal_pixel_preedit_text_reaches_pixels() {
    use crate::ui::terminal::iced::{CELL_H, PAD, cell_w};

    let _ = crate::ui::terminal::iced::cell_w();
    feed_baseline();
    // preedit 锚点 = 核心光标格:设在空行(行3,列3),避免基线文字干扰。
    let core = terminal(KEY, COLS, ROWS);
    terminal_set_cursor(core, 3, 3, TermCursorShape::Block);

    let mut view = terminal_view();
    if let View::Terminal { preedit, .. } = &mut view {
        *preedit = Some("ab".to_string());
    }
    let mut ui = simulator(view.into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("preedit snapshot");
    let frame = write_frame(&snap, "preedit_frame");

    // 锚点几何:x = PAD + 3×cw,y = PAD + 3×CELL_H,w = 2×cw("ab")。
    let cw = cell_w();
    let scale = 2.0f32;
    let x0 = ((PAD + 3.0 * cw) * scale) as u32;
    let x1 = ((PAD + 5.0 * cw) * scale) as u32;
    let y0 = ((PAD + 3.0 * CELL_H) * scale) as u32;
    let y1 = ((PAD + 4.0 * CELL_H) * scale) as u32;

    let ink = ink_in_rect(&frame, x0, y0, x1, y1);
    assert!(
        ink > 100,
        "preedit 区域亮像素 {ink} ≤ 100——IME 组合串文字未进入渲染产物\
         (WeakParagraph 同族丢弃;widget.rs fill_cached_para)"
    );
}

/// 诊断工具(#[ignore],漂移取证时手动跑):打印金样间墨水占比与容差
/// 差分统计——634 归因报告的数据来源,再生成金样后可复跑印证。
#[test]
#[ignore = "diagnostic: 漂移取证/金样再生成印证(见 docs/plans/evidence/634/)"]
fn terminal_pixel_dump_golden_stats() {
    let _ = crate::ui::terminal::iced::cell_w();
    feed_baseline();
    let mut ui = simulator(terminal_view().into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("snapshot");
    let actual = write_frame(&snap, "dump_stats");
    eprintln!(
        "[stats] 实测帧墨水占比 {:.2}%",
        ink_fraction(&actual) * 100.0
    );
    for (name, kind) in [("selection", "base"), ("selection", "frame"), ("cursor", "base"), ("cursor", "frame")] {
        match resolve_golden(name, kind) {
            Some(p) => {
                let g = decode_rgba(&p);
                eprintln!(
                    "[stats] {name}/{kind}: ink={:.2}% diff-vs-actual={:.2}%",
                    ink_fraction(&g) * 100.0,
                    diff_fraction(&g, &actual) * 100.0
                );
            }
            None => eprintln!("[stats] {name}/{kind}: 金样缺失"),
        }
    }
    let sel_base = resolve_golden("selection", "base").map(|p| decode_rgba(&p));
    let sel_frame = resolve_golden("selection", "frame").map(|p| decode_rgba(&p));
    if let (Some(b), Some(f)) = (sel_base, sel_frame) {
        eprintln!(
            "[stats] selection frame-vs-base diff={:.2}%",
            diff_fraction(&b, &f) * 100.0
        );
    }
    let cur_base = resolve_golden("cursor", "base").map(|p| decode_rgba(&p));
    let cur_frame = resolve_golden("cursor", "frame").map(|p| decode_rgba(&p));
    if let (Some(b), Some(f)) = (cur_base, cur_frame) {
        eprintln!(
            "[stats] cursor frame-vs-base diff={:.2}%",
            diff_fraction(&b, &f) * 100.0
        );
    }
}
