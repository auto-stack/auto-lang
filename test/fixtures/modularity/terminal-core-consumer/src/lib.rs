//! PLAN-808 external terminal-core consumer probe.
//!
//! Proves the core can be consumed without auto-lang/UI/auto-man: owned value
//! contract, deterministic absent-capability matrix, explicit-DLL loading and
//! (env-gated) a real PTY round trip through the runtime DLL.

#[cfg(test)]
use std::path::PathBuf;
#[cfg(test)]
use std::sync::Arc;
#[cfg(test)]
use std::time::{Duration, Instant};

#[cfg(test)]
use auto_terminal_core::{
    decode_style_color, EngineService, TerminalCell, TerminalColor, TerminalHostProvider,
    MAX_RESIZE_COLS, MAX_RESIZE_ROWS, MIN_RESIZE_COLS, PALETTE_SLOTS,
};

#[test]
fn owned_value_contract_is_self_contained() {
    assert_eq!(TerminalColor::Default, decode_style_color(0));
    assert_eq!(TerminalColor::Indexed(7), decode_style_color(1 << 24 | 7));
    assert_eq!(
        TerminalColor::Rgb(0x10, 0x20, 0x30),
        decode_style_color(2 << 24 | 0x10_20_30)
    );
    let cell = TerminalCell::plain('中');
    assert_eq!(cell.fg, TerminalColor::Default);
    assert_eq!(cell.bg, TerminalColor::Default);
    assert_eq!(PALETTE_SLOTS, 18);
    assert_eq!(MIN_RESIZE_COLS, 2);
    assert_eq!(MAX_RESIZE_COLS, 500);
    assert_eq!(MAX_RESIZE_ROWS, 200);
}

#[test]
fn absent_engine_matrix_is_deterministic() {
    let service = EngineService::without_dll();
    assert!(!service.has_library());
    assert_eq!(service.spawn(80, 24), 0);
    assert_eq!(service.spawn_ex("cmd", &[], "", 80, 24), 0);
    assert!(service.rows(1).is_empty());
    assert_eq!(service.interrupt(1), -1);
    assert!(service.is_exited(1));
    assert_eq!(service.history(1), 0);
    assert_eq!(service.scroll_offset(1), 0);
    assert_eq!(service.cursor(1), (0, 0));
    assert_eq!(service.viewport(1), (0, 0));
    assert_eq!(service.backlog_pending_mb(1), 0);
    assert_eq!(service.backlog_paused(1), 0);
    assert_eq!(service.backlog_take_alerts(1), 0);
    assert_eq!(service.backlog_dropped(1), 0);
    assert!(service.palette_tables().is_empty());
    assert_eq!(service.palette_color(0, 0, 0), None);
    // The port's non-UI defaults are usable without any UI registry.
    let provider: Arc<dyn TerminalHostProvider> = Arc::new(service);
    assert_eq!(provider.window_width(), 1024);
    assert_eq!(provider.window_height(), 768);
    assert_eq!(provider.pump_input(1), 0);
    assert_eq!(provider.apply_resize(1), 0);
    assert_eq!(provider.menu_take(), -1);
    assert!(provider.config_profiles().is_empty());
    assert_eq!(provider.config_default_profile(), "");
}

#[test]
fn load_failure_is_not_a_silent_success() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let service = EngineService::with_candidates(vec![manifest]);
    assert!(!service.has_library(), "a non-library file must not load");
    assert_eq!(service.spawn(80, 24), 0);
}

/// Env-gated real DLL smoke: `AUTOTERM_ENGINE_DLL` must point at a built
/// `autoterm_core.dll`. Without it the test prints PENDING and asserts
/// nothing (never a silent pass counted as evidence).
#[test]
fn real_dll_smoke_roundtrip() {
    let Some(path) = std::env::var("AUTOTERM_ENGINE_DLL")
        .ok()
        .map(PathBuf::from)
        .filter(|p| p.is_file())
    else {
        eprintln!(
            "PENDING(terminal-core-consumer): AUTOTERM_ENGINE_DLL not set — \
             real-DLL smoke not produced in this run"
        );
        return;
    };
    let service = EngineService::with_candidates(vec![path]);
    assert!(service.has_library(), "explicit DLL must load");
    let handle = service.spawn_ex(
        "cmd.exe",
        &["/C".into(), "echo PLAN808_CONSUMER".into()],
        ".",
        80,
        24,
    );
    assert_ne!(handle, 0, "spawn_ex must return a handle");
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut found = false;
    while Instant::now() < deadline {
        let rows = service.rows(handle);
        if rows.iter().any(|l| l.contains("PLAN808_CONSUMER")) {
            found = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    service.free(handle);
    assert!(found, "real DLL output must round-trip through the core");
    assert_eq!(service.live_handle_count(), 0, "handle table drained");
}
