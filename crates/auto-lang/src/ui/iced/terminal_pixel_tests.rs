//! PLAN-010 T8: terminal 组件像素级 headless 自动化(layout_tests 同款
//! `iced_test::simulator` 管线,Plan 414 §8.2 先例;T8 三形:固定文本行 /
//! 选中态 / 光标块)。
//!
//! 断言面:
//! 1. bounds 非零且等于 cols×CELL_W + 2 / rows×CELL_H + 2(widget 经
//!    `operate` 向 selector 暴露 bounds——canvas 型 widget 缺省不可见);
//! 2. 选中态与基线帧像素不同(高亮层进入渲染产物);
//! 3. 光标块与基线帧像素不同。
//!
//! iced_test 的 Snapshot 不开放逐像素读取,像素证据以
//! `matches_image` 落盘金样(base 首跑自建;选中/光标帧与 base 必须不同)
//! 呈现——base 的两次独立渲染必须逐字节一致,保证「不同」断言非噪声。
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
        on_select: None,
        on_menu: None,
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

/// bounds 非零且等于固定网格几何(cols×CELL_W+2, rows×CELL_H+2)。
#[test]
fn terminal_pixel_bounds_nonzero_and_exact() {
    use crate::ui::terminal::iced::{CELL_H, CELL_W};
    feed_baseline();
    let mut ui = simulator(terminal_view().into_iced());
    let store = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let _ = ui.find(BoundsCollector(store.clone()));
    let bounds = store.lock().unwrap().clone();
    let want_w = COLS as f32 * CELL_W + 2.0;
    let want_h = ROWS as f32 * CELL_H + 2.0;
    assert!(
        bounds.iter().any(|&(w, h)| (w - want_w).abs() < 0.6 && (h - want_h).abs() < 0.6),
        "terminal 组件 bounds 必须命中 {want_w}x{want_h}(±0.6),实际 {bounds:?}"
    );
}

/// 基线帧两次独立渲染逐字节一致(matches_image 自建金样后复验),
/// 随后的「选中/光标帧与基线不同」断言以此为噪声基线。
/// `name`: 用例私有金样名——nextest 每用例独立进程,共用路径会互踩。
fn write_baseline_golden(name: &str) -> std::path::PathBuf {
    feed_baseline();
    let mut ui = simulator(terminal_view().into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("baseline snapshot");
    let path = golden_path(name, "base");
    assert!(
        snap.matches_image(&path).expect("baseline matches_image"),
        "基线金样应自建/复验一致: {}",
        path.display()
    );
    // 再渲染一次同一内容,与金样必须仍一致(渲染确定性,噪声基线)。
    let snap2 = ui.snapshot(&iced::Theme::Light).expect("baseline snapshot 2");
    assert!(
        snap2.matches_image(&path).expect("baseline rematches_image"),
        "基线第二次渲染与金样不一致——渲染不确定,像素断言不可靠"
    );
    path
}

fn golden_path(name: &str, kind: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("test/ui/terminal_pixel/terminal_pixel_{name}_{kind}.png"))
}

/// 选中态:同一核心开启 Simple 选中后,渲染产物必须偏离基线(选中列带
/// 高亮进入像素)。金样 terminal_pixel_selection.png 首跑自建留档。
#[test]
fn terminal_pixel_selection_changes_pixels() {
    let base = write_baseline_golden("selection");

    let core = terminal(KEY, COLS, ROWS);
    terminal_selection_begin(core, TermSelectionType::Simple, 0, 0);
    terminal_selection_extend(core, 0, 10);
    terminal_selection_finish(core);

    let mut ui = simulator(terminal_view().into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("selection snapshot");
    let path = golden_path("selection", "frame");
    // 首跑:自建金样并返回 true;此后的运行:必须与自建时的选中帧一致。
    let identical = snap.matches_image(&path).expect("selection matches_image");
    assert!(identical, "选中帧与自身金样不一致: {}", path.display());
    // 关键断言:选中帧必须偏离基线(不然选中高亮没进渲染产物)。
    let same_as_base = snap.matches_image(&base).expect("selection vs base");
    assert!(
        !same_as_base,
        "选中帧与基线帧逐字节一致——选中高亮未进入渲染产物"
    );

    // 清场:核心上的选中不影响其他用例。
    terminal_selection_clear(core);
}

/// 光标块:光标位置/形状写入核心后,渲染产物必须偏离基线(光标块像素
/// 区域存在)。金样 terminal_pixel_cursor.png 首跑自建留档。
#[test]
fn terminal_pixel_cursor_changes_pixels() {
    let base = write_baseline_golden("cursor");

    let core = terminal(KEY, COLS, ROWS);
    terminal_set_cursor(core, 3, 5, TermCursorShape::Block);

    let mut ui = simulator(terminal_view().into_iced());
    let snap = ui.snapshot(&iced::Theme::Light).expect("cursor snapshot");
    let path = golden_path("cursor", "frame");
    let identical = snap.matches_image(&path).expect("cursor matches_image");
    assert!(identical, "光标帧与自身金样不一致: {}", path.display());
    let same_as_base = snap.matches_image(&base).expect("cursor vs base");
    assert!(
        !same_as_base,
        "光标帧与基线帧逐字节一致——光标块未进入渲染产物"
    );
}
