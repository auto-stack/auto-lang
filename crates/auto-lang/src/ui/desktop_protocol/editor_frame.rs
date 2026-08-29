// Plan 386 Stage 1 —— `EditorDrawList` → 协议帧载荷 lowering（413 §7
// 三点落位 + §7.3 缓存键；"薄切片"施工面：纯函数，无 transport 依赖）。
//
// draw.rs 头注的承接："a future RenderCommand lowering (Plan 386 Stage 1)
// serializes it to quads and text runs" —— 本文件即该 lowering 的 v1：
// `EditorDrawList`（quad/text run 语义早已齐备）→ [`DrawList`]。
// 413 §7 三点：①IME 下行 = 协议 `InputMsg::ImePreedit/ImeCommit/
// ImeCancelled` → `EditorInput` 同名输入；②字体注册 = 握手 `FontBlob`
// （`AppEndpoint::with_fonts`，Stage 2 真上传字体数据）；③按行缓存 =
// [`editor_cache_keys`]（revision × fold_hidden 组合键）随
// `FrameMsg::CacheControl` 过线。

use cosmic_text::FontSystem;

use super::endpoint::FrameSource;
use super::message::{ControlMsg, DrawList, DrawOp, InputMsg, Rgba8, WRect};
use crate::ui::code_editor::core::{
    CodeEditorConfig, CodeEditorCore, CoreOutput, EditorButton, EditorInput, EditorKey,
    EditorModifiers, NullClipboard,
};
use crate::ui::code_editor::draw::EditorDrawList;
use crate::ui::code_editor::theme::Rgba;

/// f32 线性色 → 线格式 8bit RGBA（四舍五入 + 钳位）。
pub fn rgba8(c: &Rgba) -> Rgba8 {
    let ch = |v: f32| -> u8 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 };
    Rgba8::new(ch(c.r), ch(c.g), ch(c.b), ch(c.a))
}

/// 编辑器帧 → 协议 DrawList（绘制序与 iced 适配器一致：底色 → gutter →
/// 当前行 → 选区/搜索 → 文本 → caret → preedit → 滚动条）。
pub fn lower_editor_frame(list: &EditorDrawList) -> DrawList {
    let mut ops: Vec<DrawOp> = Vec::new();
    let quad = |rect: crate::ui::code_editor::draw::Rect, color: &Rgba, ops: &mut Vec<DrawOp>| {
        ops.push(DrawOp::Quad {
            rect: WRect::new(rect.x, rect.y, rect.w, rect.h),
            color: rgba8(color),
        });
    };

    if let Some((bg_rect, bg)) = &list.background {
        quad(*bg_rect, bg, &mut ops);
    }
    if let Some(gutter) = &list.gutter {
        quad(gutter.bounds, &gutter.background, &mut ops);
        for n in &gutter.numbers {
            ops.push(DrawOp::Text {
                x: gutter.bounds.x + 4.0,
                y: n.y,
                size: gutter.font_size,
                line_height: gutter.line_height,
                color: rgba8(&gutter.foreground),
                text: n.number.to_string(),
            });
        }
        for f in &gutter.folds {
            ops.push(DrawOp::Text {
                x: gutter.bounds.x + gutter.bounds.w - 14.0,
                y: f.y,
                size: gutter.font_size,
                line_height: gutter.line_height,
                color: rgba8(&gutter.foreground),
                text: if f.folded { "▸".into() } else { "▾".into() },
            });
        }
    }
    if let Some((rect, color)) = &list.current_line {
        quad(*rect, color, &mut ops);
    }
    for (rect, color) in &list.selection {
        quad(*rect, color, &mut ops);
    }
    for (rect, color) in &list.search_matches {
        quad(*rect, color, &mut ops);
    }
    for run in &list.text_runs {
        ops.push(DrawOp::Text {
            x: run.x,
            y: run.y,
            size: run.size,
            line_height: run.line_height,
            color: rgba8(&run.color),
            text: run.text.clone(),
        });
    }
    if let Some(caret) = &list.caret {
        quad(caret.rect, &caret.color, &mut ops);
    }
    if let Some(pre) = &list.preedit {
        quad(pre.underline, &pre.color, &mut ops);
        ops.push(DrawOp::Text {
            x: pre.origin.x,
            y: pre.origin.y,
            size: pre.font_size,
            line_height: pre.font_size * 1.3,
            color: rgba8(&pre.color),
            text: pre.text.clone(),
        });
    }
    if let Some(sb) = &list.scrollbar_v {
        quad(sb.thumb, &sb.color, &mut ops);
    }
    if let Some(sb) = &list.scrollbar_h {
        quad(sb.thumb, &sb.color, &mut ops);
    }

    DrawList { clear: None, ops }
}

/// 413 §7.3：宿主侧缓存键（gutter 图等按行资产以 (revision, fold_hidden)
/// 失效——fold 翻转不改 revision 但必须重绘 gutter，draw.rs 注释同源）。
pub fn editor_cache_keys(list: &EditorDrawList) -> Vec<u64> {
    vec![list.revision.wrapping_shl(16) ^ (list.fold_hidden as u64)]
}

/// 编辑器会话的协议侧 [`FrameSource`]：协议输入 → `EditorInput`，
/// `core::render` → [`DrawList`]。Stage 2 里这是编辑器 exe 的渲染循环。
pub struct EditorFrameSource {
    pub core: CodeEditorCore,
    pub font_system: FontSystem,
    clipboard: NullClipboard,
    /// 最近一次产出帧的缓存键（观测面）。
    pub last_cache_keys: Vec<u64>,
}

impl EditorFrameSource {
    pub fn new(key: &str, lang: &str, font_size: f32) -> Self {
        let mut font_system = FontSystem::new();
        let config = CodeEditorConfig {
            lang: lang.to_string(),
            font_size,
            ..CodeEditorConfig::default()
        };
        let core = CodeEditorCore::new(key, config, &mut font_system);
        Self {
            core,
            font_system,
            clipboard: NullClipboard,
            last_cache_keys: Vec::new(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.core.set_text(text, &mut self.font_system);
    }

    /// 协议 IME 下行 → core（413 §7.① 的落位点）。
    fn feed(&mut self, input: EditorInput) -> CoreOutput {
        self.core
            .handle_input(&mut self.font_system, input, &mut self.clipboard)
    }
}

impl FrameSource for EditorFrameSource {
    fn revision(&self) -> u64 {
        self.core.revision()
    }

    fn render_frame(&mut self) -> DrawList {
        let list = crate::ui::code_editor::core::render::render(
            &self.core,
            &mut self.font_system,
            480.0,
            320.0,
            None,
        );
        self.last_cache_keys = editor_cache_keys(&list);
        lower_editor_frame(&list)
    }

    fn on_input(&mut self, input: &InputMsg) {
        let editor_input = match input {
            InputMsg::KeyPressed { key, modifiers, .. } => EditorInput::KeyPressed {
                key: EditorKey::Other(key.to_string()),
                text: None,
                modifiers: wire_mods(*modifiers),
            },
            InputMsg::KeyReleased { .. } => EditorInput::KeyReleased,
            InputMsg::CharTyped { ch, .. } => EditorInput::KeyPressed {
                key: EditorKey::Char(*ch),
                text: Some(ch.to_string()),
                modifiers: EditorModifiers::none(),
            },
            InputMsg::PointerPressed { button, x, y, .. } => EditorInput::MousePressed {
                button: match button {
                    super::message::MouseButton::Left => EditorButton::Left,
                    super::message::MouseButton::Right => EditorButton::Right,
                    super::message::MouseButton::Middle => EditorButton::Middle,
                },
                x: *x,
                y: *y,
            },
            InputMsg::PointerReleased { .. } => EditorInput::MouseReleased {
                button: EditorButton::Left,
            },
            InputMsg::PointerMoved { x, y, .. } => EditorInput::MouseMoved { x: *x, y: *y },
            InputMsg::Scroll { dx, dy, .. } => EditorInput::WheelScrolled {
                dx: *dx,
                dy: *dy,
                shift: false,
            },
            InputMsg::ImePreedit { text, .. } => EditorInput::ImePreedit(text.clone()),
            InputMsg::ImeCommit { text, .. } => EditorInput::ImeCommit(text.clone()),
            InputMsg::ImeCancelled { .. } => EditorInput::ImeClosed,
        };
        self.feed(editor_input);
    }

    fn on_control(&mut self, _control: &ControlMsg) {}
}

/// 线格式修饰位（bit0 shift / bit1 ctrl / bit2 alt / bit3 logo）→ core 修饰。
fn wire_mods(bits: u8) -> EditorModifiers {
    EditorModifiers {
        shift: bits & 1 != 0,
        control: bits & 2 != 0,
        alt: bits & 4 != 0,
        logo: bits & 8 != 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "fn main() {\n    let x = 1;\n}\n";

    fn source() -> EditorFrameSource {
        let mut s = EditorFrameSource::new("probe", "auto", 14.0);
        s.set_text(SRC);
        s
    }

    fn frame_text(list: &DrawList) -> String {
        list.ops
            .iter()
            .filter_map(|op| match op {
                DrawOp::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("|")
    }

    #[test]
    fn typed_text_lands_as_text_runs_in_frame() {
        let mut s = source();
        let list = s.render_frame();
        // 正文行按语法 span 切片后逐 run 过线；行号 1..=3 各一 run。
        let joined = frame_text(&list);
        assert!(joined.contains("fn"), "关键字 run 过线: {joined}");
        assert!(joined.contains("main"), "标识符 run 过线: {joined}");
        assert!(joined.contains("let"), "声明 run 过线: {joined}");
        for n in ["1", "2", "3"] {
            assert!(joined.split('|').any(|t| t == n), "行号 {n} 过线: {joined}");
        }
        // 同帧 golden：同输入 → 同 DrawList（结构等价，可序列化比较）。
        let again = s.render_frame();
        assert_eq!(again, list);
    }

    #[test]
    fn protocol_input_lowerset_matches_core_direct() {
        // 协议路径打字（CharTyped 'x'）与 core 直喂 EditorInput 同效
        // （含焦点门控：两路都先 FocusGained）。
        let mut via_protocol = source();
        via_protocol.feed(EditorInput::FocusGained);
        via_protocol.on_input(&InputMsg::CharTyped { wid: 1, ch: 'x' });
        let text_via_protocol = via_protocol.core.text();

        let mut direct = source();
        direct.feed(EditorInput::FocusGained);
        direct.feed(EditorInput::KeyPressed {
            key: EditorKey::Char('x'),
            text: Some("x".into()),
            modifiers: EditorModifiers::none(),
        });
        assert_eq!(text_via_protocol, direct.core.text(), "协议输入与直喂无差");
        assert!(text_via_protocol.starts_with('x'), "打字确实落盘: {text_via_protocol:?}");
    }

    #[test]
    fn ime_preedit_shows_in_frame_then_commit_lands_in_buffer() {
        let mut s = source();
        s.feed(EditorInput::FocusGained);
        // preedit 覆盖：帧里出现组合串（413 §7.① 上屏语义）。
        s.on_input(&InputMsg::ImePreedit {
            wid: 1,
            text: "让".into(),
            cursor: WRect::new(4.0, 0.0, 1.0, 14.0),
        });
        let preedit_frame = s.render_frame();
        assert!(frame_text(&preedit_frame).contains("让"), "preedit 过线");

        // commit：组合串落盘 + revision 前进。
        let rev_before = s.revision();
        s.on_input(&InputMsg::ImeCommit { wid: 1, text: "让 ".into() });
        assert!(s.core.text().starts_with("让 "), "IME commit 落盘: {:?}", s.core.text());
        assert!(s.revision() > rev_before, "落盘推版本");
        let committed_frame = s.render_frame();
        assert!(
            frame_text(&committed_frame).contains("让"),
            "落盘文本随帧过线"
        );
        assert!(
            !s.last_cache_keys.is_empty(),
            "缓存键随帧产出（413 §7.③）"
        );
    }

    #[test]
    fn cache_control_keys_stable_and_versioned() {
        let mut s = source();
        s.feed(EditorInput::FocusGained);
        let _f1 = s.render_frame();
        let k1 = s.last_cache_keys.clone();
        let _f2 = s.render_frame();
        assert_eq!(s.last_cache_keys, k1, "同帧同键");
        s.feed(EditorInput::KeyPressed {
            key: EditorKey::Char('z'),
            text: Some("z".into()),
            modifiers: EditorModifiers::none(),
        });
        let _f3 = s.render_frame();
        assert_ne!(s.last_cache_keys, k1, "revision 变 → 键变");
    }
}
