//! Plan 277: UTF-16 安全的位置偏移转换。
//!
//! LSP 协议使用 UTF-16 code units 计数 `position.character`，
//! 但 Rust 字符串是 UTF-8。直接 `&line[..character as usize]` 在非 ASCII
//! （如中文）字符上会 panic 或切到错误位置。
//!
//! 本模块提供 `utf16_to_byte_offset` 和 `slice_line_at_char` 两个辅助函数，
//! 安全地在 UTF-8 字符串上按 UTF-16 偏移切片。

/// 将 UTF-16 code unit 偏移转换为 UTF-8 字节偏移。
///
/// `char_offset` 是 LSP position.character（UTF-16 code units）。
/// 返回对应的 UTF-8 字节索引。如果 `char_offset` 超出字符串范围，返回行尾。
pub fn utf16_to_byte_offset(line: &str, char_offset: u32) -> usize {
    let mut current_utf16 = 0u32;
    for (byte_idx, ch) in line.char_indices() {
        if current_utf16 >= char_offset {
            return byte_idx;
        }
        current_utf16 += ch.len_utf16() as u32;
    }
    line.len()
}

/// 安全地在行字符串上按 UTF-16 character 偏移切片（到行尾）。
///
/// 等价于 `&line[utf16_to_byte_offset(line, char_offset)..]`，但不会 panic。
pub fn slice_line_at_char(line: &str, char_offset: u32) -> &str {
    let byte_offset = utf16_to_byte_offset(line, char_offset);
    &line[byte_offset..]
}

/// 安全地取行的前缀（到 UTF-16 character 偏移为止）。
///
/// 等价于 `&line[..utf16_to_byte_offset(line, char_offset)]`，但不会 panic。
pub fn slice_line_before_char(line: &str, char_offset: u32) -> &str {
    let byte_offset = utf16_to_byte_offset(line, char_offset);
    &line[..byte_offset]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii() {
        assert_eq!(utf16_to_byte_offset("hello world", 5), 5);
        assert_eq!(slice_line_at_char("hello world", 6), "world");
        assert_eq!(slice_line_before_char("hello world", 5), "hello");
    }

    #[test]
    fn test_chinese() {
        // 每个 UTF-16 code unit = 1 char for BMP characters
        // 中文在 UTF-16 里每个字 1 个 code unit，在 UTF-8 里每个字 3 字节
        let line = "你好世界test";
        assert_eq!(utf16_to_byte_offset(line, 2), 6); // 第3个字"世"的字节起始
        assert_eq!(slice_line_at_char(line, 2), "世界test");
        assert_eq!(slice_line_before_char(line, 4), "你好世界");
    }

    #[test]
    fn test_emoji() {
        // 😂 是 surrogate pair (2 UTF-16 code units)
        let line = "a😂b";
        assert_eq!(utf16_to_byte_offset(line, 1), 1);  // 'a' 后
        assert_eq!(utf16_to_byte_offset(line, 3), 5);  // emoji 后（'b' 的字节起始）
        assert_eq!(slice_line_at_char(line, 3), "b");
    }

    #[test]
    fn test_out_of_bounds() {
        assert_eq!(utf16_to_byte_offset("abc", 100), 3);
        assert_eq!(slice_line_at_char("abc", 100), "");
        assert_eq!(slice_line_before_char("abc", 100), "abc");
    }
}
