//! Redis backend for the a2r stdlib `redis` module (Plan 415-B2).
//!
//! Wraps redis-rs (sync API) behind the Auto-facing sentinel-error
//! conventions used across a2r-std: never panic, return sentinels and
//! expose the last error via [`last_error`] (see `fs::read_text` and the
//! Plan 415-B1 `sqlite` module for the precedent).
//!
//! Hand-copied from `stdlib/auto/redis.at` + `redis.rs.at` — keep the
//! signatures aligned (checked by the KNOWN-DEBT 396 signature-parity test).

use std::sync::Mutex;

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn record_error(err: &redis::RedisError) {
    *LAST_ERROR.lock().unwrap() = err.to_string();
}

/// Opaque handle to a Redis connection. The connection is opened eagerly by
/// [`open`] and closed on drop (RAII) — the Auto surface has no explicit
/// `close`. No auto-reconnect: once the connection breaks, operations keep
/// reporting failure sentinels.
pub struct RedisClient {
    con: Option<redis::Connection>,
}

/// Open a connection to a Redis server ("redis://host:port/db"). On failure
/// (unreachable server, bad URL) the returned client holds no connection and
/// every operation reports its failure sentinel, with the error recorded for
/// [`last_error`].
pub fn open(addr: &str) -> RedisClient {
    let con = redis::Client::open(addr)
        .and_then(|client| client.get_connection())
        .map_err(|e| {
            record_error(&e);
            e
        })
        .ok();
    RedisClient { con }
}

/// The most recent error message recorded by this module ("" if none).
pub fn last_error() -> String {
    LAST_ERROR.lock().unwrap().clone()
}

impl RedisClient {
    fn con_mut(&mut self) -> Option<&mut redis::Connection> {
        self.con.as_mut()
    }

    /// PING the server. False when not connected or on error.
    pub fn ping(&mut self) -> bool {
        match self.con_mut() {
            Some(con) => {
                let pong: Result<String, _> = redis::cmd("PING").query(con);
                match pong {
                    Ok(pong) => pong == "PONG",
                    Err(e) => {
                        record_error(&e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    /// Set a string key to a string value. True on success.
    pub fn set(&mut self, key: &str, val: &str) -> bool {
        match self.con_mut() {
            Some(con) => {
                use redis::Commands;
                match con.set(key, val) {
                    Ok(()) => true,
                    Err(e) => {
                        record_error(&e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    /// Get a string value by key. "" when missing or on failure.
    pub fn get(&mut self, key: &str) -> String {
        match self.con_mut() {
            Some(con) => {
                use redis::Commands;
                // GET on a missing key is redis::Nil — not an error — and
                // maps to "" (Auto missing-value convention).
                match con.get(key) {
                    Ok(v) => v,
                    Err(e) => {
                        record_error(&e);
                        String::new()
                    }
                }
            }
            None => String::new(),
        }
    }

    /// Delete a key. Number of keys removed, or -1 on failure.
    pub fn del(&mut self, key: &str) -> i64 {
        match self.con_mut() {
            Some(con) => {
                use redis::Commands;
                match con.del(key) {
                    Ok(n) => n,
                    Err(e) => {
                        record_error(&e);
                        -1
                    }
                }
            }
            None => -1,
        }
    }

    /// Check whether a key exists. False when missing or on failure.
    pub fn exists(&mut self, key: &str) -> bool {
        match self.con_mut() {
            Some(con) => {
                use redis::Commands;
                match con.exists(key) {
                    Ok(b) => b,
                    Err(e) => {
                        record_error(&e);
                        false
                    }
                }
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sentinels for the no-server path run unconditionally: a connection
    /// to a closed port exercises the full failure convention (open falls
    /// back to a connection-less client, every op reports its sentinel,
    /// last_error records the cause) without needing a Redis server.
    #[test]
    fn redis_failure_sentinels_without_server() {
        let mut r = open("redis://127.0.0.1:1");
        assert!(!r.ping());
        assert!(!r.set("k", "v"));
        assert_eq!(r.get("k"), "");
        assert_eq!(r.del("k"), -1);
        assert!(!r.exists("k"));
        assert!(!last_error().is_empty());
    }

    #[test]
    fn redis_bad_url_recorded() {
        let _r = open("not a redis url");
        assert!(!last_error().is_empty());
    }

    /// Live roundtrip, guarded: only runs when AUTO_TEST_REDIS_URL points at
    /// a reachable server (e.g. `redis://127.0.0.1:6379`). Skips silently
    /// otherwise so the daily tiers never depend on a running Redis.
    #[test]
    fn redis_live_roundtrip() {
        let Ok(url) = std::env::var("AUTO_TEST_REDIS_URL") else {
            eprintln!("skipping: AUTO_TEST_REDIS_URL not set");
            return;
        };
        let mut r = open(&url);
        assert!(r.ping(), "ping failed: {}", last_error());
        assert!(r.set("a2r_415b2_k", "Michi"));
        assert_eq!(r.get("a2r_415b2_k"), "Michi");
        assert!(r.exists("a2r_415b2_k"));
        assert_eq!(r.del("a2r_415b2_k"), 1);
        assert!(!r.exists("a2r_415b2_k"));
        // Missing key → "" (Nil is not an error).
        assert_eq!(r.get("a2r_415b2_missing"), "");
    }
}
