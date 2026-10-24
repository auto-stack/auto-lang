//! Locate dependency declaration lines in code, without interpreting inert text.
//! Returned slices retain the original spelling (including quoted dep properties).
//! This is preprocessing only; Parser remains responsible for syntax validation.
pub(crate) fn code_lines(source: &str) -> Vec<&str> {
    #[derive(Clone, Copy)]
    enum State {
        Code,
        LineComment,
        BlockComment,
        Quoted(u8, bool, bool),
        Interpolation(usize),
    }
    let bytes = source.as_bytes();
    let mut state = State::Code;
    let mut stack = Vec::new();
    let mut first = None;
    let mut occupied = false;
    let mut lines = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\n' {
            if let Some(start) = first.take() {
                lines.push(&source[start..i]);
            }
            occupied = matches!(state, State::Quoted(..) | State::Interpolation(_))
                || stack
                    .iter()
                    .any(|s| matches!(s, State::Quoted(..) | State::Interpolation(_)));
            if matches!(state, State::LineComment) {
                state = stack.pop().unwrap_or(State::Code);
            }
            i += 1;
            continue;
        }
        match state {
            State::LineComment => {
                i += 1;
            }
            State::BlockComment => {
                if bytes[i..].starts_with(b"*/") {
                    state = stack.pop().unwrap_or(State::Code);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            State::Quoted(quote, triple, formatted) => {
                if b == b'\\' && quote != b'`' {
                    // Keep newline processing centralized, even after an escape.
                    if bytes.get(i + 1) == Some(&b'\n') {
                        i += 1;
                    } else {
                        i = (i + 2).min(bytes.len());
                    }
                } else if b == quote && (!triple || bytes[i..].starts_with(b"\"\"\"")) {
                    if triple {
                        // Extra quotes in a closing run are literal content in Auto.
                        while bytes.get(i) == Some(&b'"') {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                    state = stack.pop().unwrap_or(State::Code);
                } else if formatted && bytes[i..].starts_with(b"${") {
                    stack.push(state);
                    state = State::Interpolation(1);
                    i += 2;
                } else {
                    i += 1;
                }
            }
            State::Code | State::Interpolation(_) => {
                if bytes[i..].starts_with(b"//") {
                    stack.push(state);
                    state = State::LineComment;
                    i += 2;
                } else if bytes[i..].starts_with(b"/*") {
                    stack.push(state);
                    state = State::BlockComment;
                    i += 2;
                } else if matches!(b, b'"' | b'\'' | b'`') {
                    occupied = true;
                    let triple = b == b'"' && bytes[i..].starts_with(b"\"\"\"");
                    let formatted = b == b'`'
                        || (b == b'"'
                            && i > 0
                            && bytes[i - 1] == b'f'
                            && (i < 2
                                || !(bytes[i - 2].is_ascii_alphanumeric()
                                    || bytes[i - 2] == b'_')));
                    stack.push(state);
                    state = State::Quoted(b, triple, formatted);
                    i += if triple { 3 } else { 1 };
                } else {
                    if let State::Interpolation(depth) = state {
                        if b == b'{' {
                            state = State::Interpolation(depth + 1);
                        }
                        if b == b'}' {
                            state = if depth == 1 {
                                stack.pop().unwrap_or(State::Code)
                            } else {
                                State::Interpolation(depth - 1)
                            };
                        }
                    }
                    if matches!(state, State::Code) && !b.is_ascii_whitespace() && !occupied {
                        first = Some(i);
                        occupied = true;
                    }
                    i += 1;
                }
            }
        }
    }
    if let Some(start) = first {
        lines.push(&source[start..]);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nested_formatted_expressions_do_not_expose_literal_lines() {
        let source = "var f=f\"${helper(\"quoted\")}\nuse hidden\ndep hidden\n\"\nuse real\nvar t=f\"\"\"${helper(f\"${1}\")}\ndep hidden\n\"\"\"\ndep real\n";
        let lines = code_lines(source);
        assert_eq!(
            lines
                .iter()
                .filter(|s| s.starts_with("use "))
                .copied()
                .collect::<Vec<_>>(),
            ["use real"]
        );
        assert_eq!(
            lines
                .iter()
                .filter(|s| s.starts_with("dep "))
                .copied()
                .collect::<Vec<_>>(),
            ["dep real"]
        );
    }
    #[test]
    fn inert_text_and_real_declarations_preserve_spelling() {
        let source = "/*\nuse hidden.one\ndep hidden\n*/\r\nuse real.one\nvar s=\"escaped \\\"\nuse hidden.two\ndep hidden\n\"\nvar t=\"\"\"\nuse hidden.three\n\"\"\"\nvar r=`\ndep hidden\n`\ndep real(path: \"folder//data\")\n";
        let lines = code_lines(source);
        assert_eq!(
            lines
                .iter()
                .filter(|s| s.starts_with("use "))
                .copied()
                .collect::<Vec<_>>(),
            ["use real.one"]
        );
        assert_eq!(
            lines
                .iter()
                .filter(|s| s.starts_with("dep "))
                .copied()
                .collect::<Vec<_>>(),
            ["dep real(path: \"folder//data\")"]
        );
    }
}
