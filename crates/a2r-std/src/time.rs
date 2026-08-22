/// Time module - Time-related operations
/// Transpiled from auto-lang/stdlib/auto/time.at + time.rs.at
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get current time in milliseconds since Unix epoch
/// Plan 396 §2.6: i64 per stdlib/auto/time.rs.at (`now_ms() i64`) and
/// time.vm.at — the old i32 truncation wrapped epoch ms every ~24.8 days
/// and E0308'd against a2r's i64 local inference (auto-ai-client daemon).
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
}

/// Get current time in seconds since Unix epoch
/// Plan 396 §2.6: i64 per stdlib/auto/time.rs.at (`now_sec() i64`).
pub fn now_sec() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Sleep for specified milliseconds
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

/// Alias: time_now → now_sec (transpiler compatibility)
pub fn time_now() -> String {
    now_sec().to_string()
}
