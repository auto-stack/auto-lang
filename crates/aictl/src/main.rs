//! `aictl` — CLI tool for managing the aaid AI daemon.
//!
//! Usage:
//!   aictl status              Show daemon + concurrency pool status
//!   aictl models              List available models
//!   aictl usage               Show per-app token usage
//!   aictl switch <model>      Switch default model (future)
//!   aictl ping                Check if daemon is running

use std::process;

const DEFAULT_URL: &str = "http://127.0.0.1:17654";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let subcmd = args.get(1).map(|s| s.as_str()).unwrap_or("status");
    let url = std::env::var("AAID_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());

    match subcmd {
        "status" | "s" => cmd_status(&url),
        "models" | "m" => cmd_models(&url),
        "usage" | "u" => cmd_usage(&url),
        "ping" => cmd_ping(&url),
        "help" | "--help" | "-h" => print_help(),
        other => {
            eprintln!("aictl: unknown command '{}'", other);
            eprintln!("Run 'aictl help' for usage.");
            process::exit(1);
        }
    }
}

fn get_json(url: &str, path: &str) -> serde_json::Value {
    let full = format!("{}{}", url, path);
    match reqwest::blocking::get(&full) {
        Ok(resp) if resp.status().is_success() => {
            resp.json().unwrap_or(serde_json::json!({"error": "parse failed"}))
        }
        Ok(resp) => {
            eprintln!("aictl: HTTP {} from {}", resp.status(), full);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("aictl: cannot connect to {} — {}", full, e);
            eprintln!("  Is aaid running? Start it with: aaid");
            process::exit(1);
        }
    }
}

fn cmd_status(url: &str) {
    let status = get_json(url, "/v1/status");
    println!("Daemon: {}", status.get("status").and_then(|v| v.as_str()).unwrap_or("?"));
    println!(
        "Model:  {}",
        status.get("current_model").and_then(|v| v.as_str()).unwrap_or("?")
    );
    if let Some(pools) = status.get("pools").and_then(|v| v.as_array()) {
        println!("\nConcurrency Pools:");
        for pool in pools {
            let name = pool.get("provider").and_then(|v| v.as_str()).unwrap_or("?");
            let avail = pool.get("available_permits").and_then(|v| v.as_u64()).unwrap_or(0);
            let max = pool.get("max_concurrency").and_then(|v| v.as_u64()).unwrap_or(0);
            let in_use = pool.get("in_use").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("  {:12} {}/{} in use ({} available)", name, in_use, max, avail);
        }
    }
}

fn cmd_models(url: &str) {
    let resp = get_json(url, "/v1/models");
    if let Some(models) = resp.get("models").and_then(|v| v.as_array()) {
        println!("Available Models:");
        for m in models {
            let provider = m.get("provider").and_then(|v| v.as_str()).unwrap_or("?");
            let model = m.get("model").and_then(|v| v.as_str()).unwrap_or("?");
            println!("  {:30} ({})", model, provider);
        }
    }
}

fn cmd_usage(url: &str) {
    let resp = get_json(url, "/v1/usage");
    if let Some(apps) = resp.get("usage").and_then(|v| v.as_array()) {
        if apps.is_empty() {
            println!("No usage recorded yet.");
            return;
        }
        println!("{:<15} {:>12} {:>12} {:>12} {:>10}", "App", "Input", "Output", "Total", "Requests");
        println!("{:-<65}", "");
        for app in apps {
            let name = app.get("app").and_then(|v| v.as_str()).unwrap_or("?");
            let input = app.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let output = app.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let total = app.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let reqs = app.get("requests").and_then(|v| v.as_u64()).unwrap_or(0);
            println!("{:<15} {:>12} {:>12} {:>12} {:>10}", name, input, output, total, reqs);
        }
    }
}

fn cmd_ping(url: &str) {
    let full = format!("{}/v1/status", url);
    match reqwest::blocking::get(&full) {
        Ok(resp) if resp.status().is_success() => {
            println!("aaid is running at {}", url);
        }
        _ => {
            println!("aaid is NOT running at {}", url);
            process::exit(1);
        }
    }
}

fn print_help() {
    println!("aictl — AI daemon control tool");
    println!();
    println!("USAGE:");
    println!("  aictl <command>");
    println!();
    println!("COMMANDS:");
    println!("  status    Show daemon status + concurrency pools");
    println!("  models    List available models");
    println!("  usage     Show per-app token usage");
    println!("  ping      Check if daemon is running");
    println!();
    println!("ENVIRONMENT:");
    println!("  AAID_URL  Override daemon URL (default: {})", DEFAULT_URL);
}
