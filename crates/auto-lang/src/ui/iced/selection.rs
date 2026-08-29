//! Plan 481: SelectableText 的选区纯逻辑（全平台单测，无 iced 依赖）。
//!
//! 选区状态是 widget 本地状态（G2：不进 DesktopSession/WmState）。本模块只做
//! 字节偏移层面的状态机：归一、词边界（双击语义）、拖选扩展、清空。索引语义
//! 与 cosmic-text `hit()` 一致——UTF-8 字节偏移，且所有边界保证落在 char 边界。
//!
//! 词边界语义（v1 固化，待澄清事项已裁定）：按字符类别分段——
//! - ASCII 字母/数字/下划线连成一段（拉丁词）；
//! - CJK（Han/平假名/片假名/谚文）连成一段（UAX#29 默认连字成词）；
//! - 空白、标点各自按同类连续分段。
//! 双击落在哪一段就选整段；中英混合文本按类别切开（"abc你好def" 双击
//! `你` → "你好"）。

use std::ops::Range;

/// 选区：anchor = 按下点，head = 当前点（拖动/双击后可移动）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    pub anchor: usize,
    pub head: usize,
}

impl Selection {
    pub fn new() -> Self {
        Self { anchor: 0, head: 0 }
    }

    /// 选区是否为空（anchor == head）。
    pub fn is_empty(&self) -> bool {
        self.anchor == self.head
    }

    /// 归一化区间（min..max 字节偏移）。
    pub fn range(&self) -> Range<usize> {
        if self.anchor <= self.head {
            self.anchor..self.head
        } else {
            self.head..self.anchor
        }
    }

    /// 拖选扩展：按下不动 anchor，head 追随光标。
    pub fn extend_to(&mut self, pos: usize) {
        self.head = pos;
    }

    /// 双击选词：anchor/head 落在 pos 所在词段的起止。
    pub fn select_word(&mut self, text: &str, pos: usize) {
        let r = word_range(text, pos);
        self.anchor = r.start;
        self.head = r.end;
    }

    /// 清空选区（Esc / 单击）。
    pub fn clear(&mut self) {
        self.head = self.anchor;
    }

    /// 选中文本切片（空选区返回空串）。
    pub fn selected_text<'a>(&self, text: &'a str) -> &'a str {
        let r = self.range();
        &text[r]
    }
}

/// 字符类别（词段切分粒度）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharClass {
    /// ASCII 字母/数字/下划线（拉丁词字符）。
    Alnum,
    /// CJK 表意文字（Han/假名/谚文，连字成词）。
    Cjk,
    /// 空白。
    Space,
    /// 其他（标点/符号）。
    Punct,
}

fn char_class(c: char) -> CharClass {
    if c.is_ascii_alphanumeric() || c == '_' {
        CharClass::Alnum
    } else if is_cjk(c) {
        CharClass::Cjk
    } else if c.is_whitespace() {
        CharClass::Space
    } else {
        CharClass::Punct
    }
}

fn is_cjk(c: char) -> bool {
    let cp = c as u32;
    // Han 统一表意 + 扩展A + 平假名 + 片假名 + 谚文音节块 + 兼容谚文。
    (0x4E00..=0x9FFF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x3040..=0x30FF).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
        || (0x1100..=0x11FF).contains(&cp)
}

/// pos 所在词段的字节偏移区间。pos 越界或落在 char 中间时吸附到最近的
/// char 边界。空文本返回 0..0。
pub fn word_range(text: &str, pos: usize) -> Range<usize> {
    let len = text.len();
    if len == 0 {
        return 0..0;
    }
    // 吸附到 char 边界（向下取整到不大于 pos 的最近边界）。
    let mut p = pos.min(len);
    while p > 0 && !text.is_char_boundary(p) {
        p -= 1;
    }
    if p >= len {
        // 指向末尾：取最后一个字符的类别。
        let last_start = prev_char_start(text, len);
        let class = text[last_start..].chars().next().map(char_class).unwrap_or(CharClass::Punct);
        let mut start = last_start;
        while start > 0 {
            let ps = prev_char_start(text, start);
            if text[ps..].chars().next().map(char_class) == Some(class) {
                start = ps;
            } else {
                break;
            }
        }
        return start..len;
    }

    // p 处字符的类别，向两侧扩展同类连续段。
    let class = text[p..].chars().next().map(char_class).unwrap_or(CharClass::Punct);
    let mut start = p;
    while start > 0 {
        let ps = prev_char_start(text, start);
        if text[ps..].chars().next().map(char_class) == Some(class) {
            start = ps;
        } else {
            break;
        }
    }
    let mut end = p;
    while end < len {
        let ch_len = text[end..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        if text[end..].chars().next().map(char_class) == Some(class) {
            end += ch_len;
        } else {
            break;
        }
    }
    start..end
}

fn prev_char_start(text: &str, pos: usize) -> usize {
    let mut i = pos - 1;
    while i > 0 && !text.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t01_empty_and_clear() {
        let mut sel = Selection::new();
        assert!(sel.is_empty());
        assert_eq!(sel.range(), 0..0);
        sel.extend_to(5);
        assert_eq!(sel.range(), 0..5);
        assert!(!sel.is_empty());
        sel.clear();
        assert!(sel.is_empty());
        assert_eq!(sel.range(), 0..0);
    }

    #[test]
    fn t02_normalized_range_backward_drag() {
        // 从右往左拖:anchor > head,range 仍归一为 min..max。
        let mut sel = Selection { anchor: 10, head: 3 };
        assert_eq!(sel.range(), 3..10);
        sel.extend_to(0);
        assert_eq!(sel.range(), 0..10);
        assert_eq!(sel.anchor, 10);
    }

    #[test]
    fn t03_drag_extend_selects_slice() {
        let text = "hello world";
        let mut sel = Selection::new();
        sel.extend_to(5); // anchor 0 → head 5
        assert_eq!(sel.selected_text(text), "hello");
        sel.extend_to(11);
        assert_eq!(sel.selected_text(text), "hello world");
    }

    #[test]
    fn t04_word_range_latin() {
        // 词内任意点 → 整词;空格自成一段。
        assert_eq!(word_range("hello world", 0), 0..5);
        assert_eq!(word_range("hello world", 4), 0..5);
        assert_eq!(word_range("hello world", 5), 5..6); // 空格
        assert_eq!(word_range("hello world", 7), 6..11);
    }

    #[test]
    fn t05_word_range_punct_run() {
        // 标点连续段自成一段。
        assert_eq!(word_range("a...b", 2), 1..4);
        assert_eq!(word_range("a...b", 0), 0..1);
        assert_eq!(word_range("a...b", 4), 4..5);
    }

    #[test]
    fn t06_word_range_cjk_joined() {
        // UAX#29 默认连字:连续汉字为一段;字节偏移对齐 char 边界。
        let text = "你好世界";
        assert_eq!(text.len(), 12);
        assert_eq!(word_range(text, 0), 0..12);
        assert_eq!(word_range(text, 3), 0..12); // `好` 中间吸附 0
        assert_eq!(word_range(text, 6), 0..12);
    }

    #[test]
    fn t07_word_range_mixed_script_boundary() {
        // 中英混合:类别切换即词界。
        let text = "abc你好def";
        assert_eq!(word_range(text, 1), 0..3); // "abc"
        assert_eq!(word_range(text, 3), 3..9); // "你好"
        assert_eq!(word_range(text, 9), 9..12); // "def"
        // 双击选词固化:点 `你` 选 "你好"。
        let mut sel = Selection::new();
        sel.select_word(text, 3);
        assert_eq!(sel.selected_text(text), "你好");
    }

    #[test]
    fn t08_word_range_edges() {
        // 越界/末尾吸附。
        assert_eq!(word_range("", 5), 0..0);
        assert_eq!(word_range("hi", 99), 0..2);
        assert_eq!(word_range("hi", 2), 0..2); // 末尾取最后一段
        assert_eq!(word_range("a b", 3), 2..3); // 末尾=最后字符
    }

    #[test]
    fn t09_select_word_latin() {
        let text = "drag double click";
        let mut sel = Selection::new();
        sel.select_word(text, 8); // "double" 内
        assert_eq!(sel.selected_text(text), "double");
        assert_eq!(sel.range(), 5..11);
        sel.select_word(text, 0);
        assert_eq!(sel.selected_text(text), "drag");
    }

    #[test]
    fn t10_mixed_cjk_selected_slice() {
        // 汉字区间的切片按字节偏移切,UTF-8 边界安全。
        let text = "姓名:张三";
        // "姓名" = 0..6, ":" = 6..7, "张三" = 7..13
        let mut sel = Selection { anchor: 0, head: 13 };
        assert_eq!(sel.selected_text(text), "姓名:张三");
        sel = Selection { anchor: 7, head: 13 };
        assert_eq!(sel.selected_text(text), "张三");
    }
}
