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
        // May be skipped in headless CI where arboard cannot init.
        if let Some(_) = arboard::Clipboard::new().ok() {
            assert!(clipboard_set("plan418-clipboard-roundtrip"));
            assert_eq!(clipboard_get().as_deref(), Some("plan418-clipboard-roundtrip"));
        }
    }
}
