/// Time module - Time-related operations
/// Transpiled from auto-lang/stdlib/auto/time.at + time.rs.at

use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Get current time in milliseconds since Unix epoch (truncated to i32)
pub fn now_ms() -> i32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as i32
}

/// Get current time in seconds since Unix epoch (truncated to i32)
pub fn now_sec() -> i32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32
}

/// Sleep for specified milliseconds
pub fn sleep_ms(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}
