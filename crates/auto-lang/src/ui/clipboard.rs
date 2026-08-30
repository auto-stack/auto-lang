// Plan 418: OS clipboard bridge for VM natives and menu-driven editor
// actions (arboard). The iced widget path keeps its own EditorClipboard
// adapter (iced clipboard handles from event context); this module serves
// the handler-side natives, which run outside any iced event.
#![cfg(feature = "ui-clipboard")]

/// Read the OS clipboard text (None when empty or unavailable).
pub fn clipboard_get() -> Option<String> {
    arboard::Clipboard::new().ok().and_then(|mut c| c.get_text().ok())
}

/// Write text to the OS clipboard. Returns false when unavailable.
pub fn clipboard_set(text: &str) -> bool {
    match arboard::Clipboard::new() {
        Ok(mut c) => c.set_text(text.to_owned()).is_ok(),
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_then_get_roundtrip() {
        // Plan 485 T9: 跨进程命名互斥——nextest 每测试一进程并行时，其他
        // 剪贴板集成测试（files/image 的 EmptyClipboard）会清掉本测试
        // set→get 窗口的内容。锁在本仓 native-clipboard 档下提供；其余
        // 档（无该 feature）保持原裸跑形态。
        #[cfg(all(windows, feature = "native-clipboard"))]
        let _global = match crate::ui::clipboard_native::GlobalClipboardTestLock::acquire() {
            Some(g) => g,
            None => return, // 拿锁超时视同 headless 跳过
        };
        // May be skipped in headless CI where arboard cannot init.
        if let Some(_) = arboard::Clipboard::new().ok() {
            assert!(clipboard_set("plan418-clipboard-roundtrip"));
            assert_eq!(clipboard_get().as_deref(), Some("plan418-clipboard-roundtrip"));
        }
    }
}
