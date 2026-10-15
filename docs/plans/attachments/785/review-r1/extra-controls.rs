
// Independently selected controls. Appended to a frozen fixture copy, never production source.
#[test]
fn r1_concurrent_explicit_default_beats_compat_install() {
    use auto_lang::vm::host_capabilities::set_process_default_clipboard_text_provider_if_absent;
    let _g = serialized();
    let _saved = DefaultGuard::new();
    for _ in 0..32 {
        clear_process_default_clipboard_text_provider();
        let barrier = Arc::new(std::sync::Barrier::new(3));
        let b = barrier.clone();
        let compat = std::thread::spawn(move || {
            b.wait();
            for _ in 0..16 {
                set_process_default_clipboard_text_provider_if_absent(FakeClipboard::new(Some("compat")));
                auto_lang::ui::clipboard_provider::install_process_default();
            }
        });
        let b = barrier.clone();
        let explicit = std::thread::spawn(move || {
            b.wait();
            set_process_default_clipboard_text_provider(FakeClipboard::new(Some("explicit-race")));
        });
        barrier.wait();
        compat.join().unwrap();
        explicit.join().unwrap();
        assert_eq!(process_default_clipboard_text_provider().unwrap().text().as_deref(), Some("explicit-race"));
    }
}

#[test]
fn r1_clear_default_preserves_live_vm_and_ni_clone_is_independent() {
    let _g = serialized();
    let _saved = DefaultGuard::new();
    let code = compile_program(GET_PROGRAM);
    let a = FakeClipboard::new(Some("snapshot-clear-甲"));
    set_process_default_clipboard_text_provider(a.clone());
    let (vm, buf) = build_vm(&code);
    clear_process_default_clipboard_text_provider();
    drop(a);
    assert_eq!(run_with(&vm, &buf), "snapshot-clear-甲\n");
    let (new_vm, _) = build_vm(&code);
    assert!(new_vm.clipboard_text_provider().is_none());
    let ni = NativeInterface::production();
    ni.set_clipboard_text_provider(FakeClipboard::new(Some("original")));
    let clone = ni.clone();
    clone.set_clipboard_text_provider(FakeClipboard::new(Some("clone")));
    assert_eq!(ni.clipboard_text_provider().unwrap().text().as_deref(), Some("original"));
    assert_eq!(clone.clipboard_text_provider().unwrap().text().as_deref(), Some("clone"));
}

#[test]
fn r1_owned_text_survives_provider_mutation_and_embedded_nul() {
    let _g = serialized();
    let _saved = DefaultGuard::new();
    struct Changing(Mutex<String>);
    impl ClipboardTextProvider for Changing {
        fn text(&self) -> Option<String> { Some(self.0.lock().unwrap().clone()) }
        fn set_text(&self, text: &str) -> bool { *self.0.lock().unwrap() = text.to_owned(); false }
    }
    set_process_default_clipboard_text_provider(Arc::new(Changing(Mutex::new("旧\0文🎉".into()))));
    let (_, out) = auto_lang::run_with_capture(CYCLE_PROGRAM).unwrap();
    assert_eq!(out, "旧\0文🎉\nfalse\n甲乙-1\n旧\0文🎉\n");
}
