//! AutoOS AI daemon (`aaid`) — shared LLM concurrency arbitration.
//!
//! All AutoOS apps route LLM requests through this daemon for:
//! - Global concurrency pools (per-model Semaphore).
//! - API Key vault (apps never touch secrets).
//! - Model routing + fallback.
//! - Cost/token tracking (per-app).
//!
//! Protocol: HTTP (axum) over TCP localhost (MVP) → Unix socket (future).
//! Apps use `auto-ai-client` which auto-discovers the daemon.

pub mod config;
pub mod pool;
pub mod server;
pub mod tracker;

pub use config::DaemonConfig;
pub use pool::ConcurrencyManager;
pub use server::AppState;
pub use tracker::UsageTracker;
