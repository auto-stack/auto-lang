// Plan 413 follow-up: process-wide UI console ring buffer.
//
// Feeds in-app console panels (041 auto-edit). Two write paths converge
// here: `vm_print` mirrors every DSL `print()` line, and the
// `console_log()` native appends explicit messages. The buffer is NOT
// feature-gated and carries at most `CAP` lines, so non-UI CLI runs pay a
// capped, negligible cost. Reads are newest-first: DSL scrollables have no
// scroll-to-bottom primitive, so the freshest lines must stay at the top.

use std::collections::VecDeque;
use std::sync::Mutex;

/// Maximum retained lines (oldest dropped on overflow).
const CAP: usize = 500;
/// Default lines returned by [`ui_console_lines`].
pub const DEFAULT_LINES: usize = 200;

lazy_static::lazy_static! {
    static ref UI_CONSOLE: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
}

/// Append one line (single \n-separated payload is kept as one entry).
pub fn ui_console_push(line: &str) {
    let mut buf = UI_CONSOLE.lock().unwrap();
    if buf.len() >= CAP {
        buf.pop_front();
    }
    buf.push_back(line.to_owned());
}

/// Newest-first snapshot of the last `n` lines, joined with `\n`.
pub fn ui_console_lines(n: usize) -> String {
    let buf = UI_CONSOLE.lock().unwrap();
    let take = n.min(buf.len());
    let mut out = String::new();
    for line in buf.iter().rev().take(take) {
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(line);
    }
    out
}

/// Drop everything.
pub fn ui_console_clear() {
    UI_CONSOLE.lock().unwrap().clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_caps_and_orders_newest_first() {
        ui_console_clear();
        for i in 0..(CAP + 50) {
            ui_console_push(&format!("line-{i}"));
        }
        let all = ui_console_lines(usize::MAX);
        let lines: Vec<&str> = all.split('\n').collect();
        assert_eq!(lines.len(), CAP, "buffer must cap at {CAP}");
        assert_eq!(lines[0], format!("line-{}", CAP + 49), "newest first");
        assert_eq!(lines[CAP - 1], "line-50", "oldest survived entry");
        // Requesting fewer lines returns only the freshest.
        let head = ui_console_lines(3);
        assert_eq!(
            head,
            format!("line-{}\nline-{}\nline-{}", CAP + 49, CAP + 48, CAP + 47)
        );
    }

    #[test]
    fn clear_empties() {
        ui_console_push("x");
        ui_console_clear();
        assert_eq!(ui_console_lines(DEFAULT_LINES), "");
    }
}
