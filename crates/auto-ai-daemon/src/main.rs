//! `aaid` — AutoOS AI Daemon binary.
//!
//! Shared LLM concurrency arbitration service for all AutoOS apps.
//!
//! Usage:
//!   aaid                        Start the daemon (foreground)
//!   aaid --config <path>        Use specific config file
//!   aaid --listen 127.0.0.1:PORT  Override listen address

use std::sync::Arc;

use auto_ai_daemon::{DaemonConfig, AppState};

#[tokio::main]
async fn main() {
    // Parse CLI args (minimal, no clap dependency).
    let mut listen_override: Option<String> = None;
    let mut config_path: Option<String> = None;
    let mut log_level = "info".to_string();

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--listen" => {
                listen_override = args.next();
            }
            "--config" => {
                config_path = args.next();
            }
            "--log-level" => {
                log_level = args.next().unwrap_or("info".into());
            }
            "--help" | "-h" => {
                println!("aaid — AutoOS AI Daemon");
                println!();
                println!("USAGE:");
                println!("  aaid [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  --listen <addr>    Override listen address (default: 127.0.0.1:17654)");
                println!("  --config <path>    Config file path (default: ~/.config/autoos/ai-daemon.at)");
                println!("  --log-level <lvl>  Log level: trace/debug/info/warn/error");
                std::process::exit(0);
            }
            _ => {}
        }
    }

    // Load config.
    let mut config = if let Some(path) = &config_path {
        let content = std::fs::read_to_string(path)
            .expect(&format!("cannot read config: {}", path));
        auto_ai_daemon::config::DaemonConfig::parse(&content)
            .unwrap_or_else(|| panic!("failed to parse config: {}", path))
    } else {
        DaemonConfig::load()
    };

    // Apply overrides.
    if let Some(addr) = &listen_override {
        config.listen_addr = addr.clone();
    }

    // Init logging.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(&log_level));
    tracing_subscriber::fmt().with_env_filter(filter).init();

    // Check we have at least one provider configured.
    if config.providers.is_empty() {
        eprintln!("aaid: no providers configured.");
        eprintln!("  Set env vars (ZHIPU_API_KEY / ANTHROPIC_API_KEY / OPENAI_API_KEY)");
        eprintln!("  or create ~/.config/autoos/ai-daemon.at");
        std::process::exit(1);
    }

    let listen_addr = config.listen_addr.clone();
    let state = Arc::new(AppState::new(config));

    // Log config before moving Arc into router.
    tracing::info!("aaid listening on http://{}", listen_addr);
    for (name, p) in &state.config.providers {
        tracing::info!(
            "  provider: {} (kind={}, models={:?}, max_concurrency={})",
            name, p.kind, p.models, p.max_concurrency
        );
    }

    // Build router (takes ownership of the Arc).
    let app = auto_ai_daemon::server::router(state);

    let listener = tokio::net::TcpListener::bind(&listen_addr)
        .await
        .expect(&format!("failed to bind {}", listen_addr));
    axum::serve(listener, app)
        .await
        .expect("server error");
}
