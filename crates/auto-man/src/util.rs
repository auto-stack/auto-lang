use std::fs;
use std::io::{self, Write};
use std::path::Path;

use colored::Colorize;

use crate::AutoResult;

/// Compute BLAKE3 hash of a file's contents
/// Returns the first 64 bits as u64 for compact storage
pub fn hash_file(path: &Path) -> std::io::Result<u64> {
    let content = fs::read(path)?;
    let hash = blake3::hash(&content);
    Ok(u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap()))
}

/// Compute BLAKE3 hash of a string
/// Returns the first 64 bits as u64 for compact storage
pub fn hash_string(content: &str) -> u64 {
    let hash = blake3::hash(content.as_bytes());
    u64::from_be_bytes(hash.as_bytes()[0..8].try_into().unwrap())
}

pub fn split_first(in_string: &str, sep: char) -> (&str, &str) {
    let mut splitter = in_string.splitn(2, sep);
    let first = splitter.next().unwrap();
    let second = splitter.next().unwrap();
    (first, second)
}

pub fn split_last(in_string: &str, sep: char) -> (&str, &str) {
    let mut splitter = in_string.rsplitn(2, sep);
    let last = splitter.next().unwrap();
    let second = match splitter.next() {
        Some(s) => s,
        None => "",
    };
    (second, last)
}

/// Select a port from available ports.
///
/// If `input` is provided, it will be used directly.
/// If there's only one port available, it will be selected automatically.
/// Otherwise, prompts the user to select from available ports.
///
/// # Arguments
/// * `input` - Optional user-specified port name
/// * `available_ports` - List of available port names
/// * `prompt_msg` - Message to display for interactive selection
///
/// # Returns
/// The selected port name
pub fn select_or_default_port(
    input: Option<String>,
    available_ports: &[auto_val::AutoStr],
    prompt_msg: &str,
) -> auto_val::AutoResult<auto_val::AutoStr> {
    use dialoguer::Select;

    let port = if let Some(input) = input {
        input.into()
    } else {
        if available_ports.len() == 1 {
            available_ports[0].clone()
        } else {
            let selection = Select::new()
                .with_prompt(prompt_msg)
                .default(0)
                .items(available_ports)
                .interact()?;

            available_ports[selection].clone()
        }
    };
    Ok(port)
}


/// Select a backend from available frontends.
///
/// Returns the index into `frontends` for the selected backend.
///
/// Logic:
/// 1. Only 1 option → auto-select
/// 2. `AUTO_RENDER` env var → match (case-insensitive)
/// 3. Non-TTY → auto-select first
/// 4. Print numbered list, read number input from user
pub fn select_backend(
    frontends: &[auto_lang::config::BackendType],
    action: &str,
) -> AutoResult<usize> {
    // 1. Auto-select if only one option
    if frontends.len() == 1 {
        return Ok(0);
    }

    let backend_names: Vec<&str> = frontends.iter()
        .map(|t| t.as_str())
        .collect();

    // 2. Check AUTO_RENDER environment variable
    if let Ok(render_env) = std::env::var("AUTO_RENDER") {
        let render_lower = render_env.to_lowercase();
        for (i, name) in backend_names.iter().enumerate() {
            if name.to_lowercase() == render_lower {
                println!("{} Using render target from AUTO_RENDER: {}",
                    "→".bright_green(), name.bright_cyan());
                return Ok(i);
            }
        }
        eprintln!("Warning: AUTO_RENDER='{}' not found, falling back to selection", render_env);
    }

    // 3. Non-TTY fallback
    if !atty_check() {
        println!("{} Auto-selecting render target: {}",
            "→".bright_green(), backend_names[0].bright_cyan());
        return Ok(0);
    }

    // 4. Interactive number selection
    println!("{}", format!("Select backend to {}:", action).bright_cyan());
    for (i, name) in backend_names.iter().enumerate() {
        println!("  {}. {}", i + 1, name);
    }

    loop {
        print!("{}", "  Enter number: ".bright_cyan());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input.parse::<usize>() {
            Ok(n) if n >= 1 && n <= frontends.len() => {
                println!("{} Selected: {}",
                    "→".bright_green(), backend_names[n - 1].bright_cyan());
                return Ok(n - 1);
            }
            _ => {
                println!("  {} Please enter a number between 1 and {}",
                    "Invalid.".bright_yellow(), frontends.len());
            }
        }
    }
}

/// Check if stdin is a TTY (terminal).
fn atty_check() -> bool {
    use std::io::IsTerminal;
    io::stdin().is_terminal()
}

/// Default HTTP port for the backend API server.
const DEFAULT_HTTP_PORT: u16 = 8080;

/// Resolve the backend HTTP port.
///
/// Reads `AUTO_HTTP_PORT` (u16) from the environment so multiple `auto run`
/// instances — or other services on the same host — can coexist. Falls back to
/// the default (8080) when unset or invalid.
///
/// All generated artifacts (backend bind address, vite proxy target, Rust UI
/// client base URL, readiness probe) must source the port from here so the
/// frontend and backend always agree.
pub fn http_port() -> u16 {
    match std::env::var("AUTO_HTTP_PORT") {
        Ok(v) => v.trim().parse::<u16>().unwrap_or(DEFAULT_HTTP_PORT),
        Err(_) => DEFAULT_HTTP_PORT,
    }
}

/// Kill any process listening on the given port (Plan 354).
/// Uses `netstat` + `taskkill` on Windows, `lsof`/`fuser` on Unix.
pub fn kill_process_on_port(port: u16) {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // netstat -ano | findstr :PORT | findstr LISTENING
        if let Ok(output) = Command::new("cmd")
            .args(["/C", &format!(
                "netstat -ano | findstr :{} | findstr LISTENING", port
            )])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                // Last column is the PID
                if let Some(pid_str) = line.split_whitespace().last() {
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        let _ = Command::new("taskkill")
                            .args(["/F", "/PID", &pid.to_string()])
                            .output();
                    }
                }
            }
            if !stdout.is_empty() {
                println!("  {} Killed stale process on port {}", "⚠".bright_yellow(), port);
                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Unix: try lsof then fuser
        let _ = std::process::Command::new("lsof")
            .args(["-ti", &format!("tcp:{}", port)])
            .output()
            .and_then(|out| {
                let pids = String::from_utf8_lossy(&out.stdout);
                for pid in pids.split_whitespace() {
                    let _ = std::process::Command::new("kill")
                        .args(["-9", pid])
                        .output();
                }
                Ok(())
            });
    }
}

/// The backend base URL (e.g. `http://127.0.0.1:8080`) for the resolved port.
pub fn http_base_url() -> String {
    format!("http://127.0.0.1:{}", http_port())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_once() {
        let (first, second) = split_first("assets/templates/lib/mylib.h", '/');
        assert_eq!(first, "assets");
        assert_eq!(second, "templates/lib/mylib.h");
    }

    #[test]
    fn test_split_last() {
        let (first, second) = split_last("assets/templates/lib/mylib.h", '/');
        assert_eq!(first, "assets/templates/lib");
        assert_eq!(second, "mylib.h");
    }

    #[test]
    fn test_http_port_default() {
        // When unset, falls back to 8080. (May differ if env happens to be set
        // in CI, so we only assert the documented default when the var is absent.)
        std::env::remove_var("AUTO_HTTP_PORT");
        assert_eq!(http_port(), DEFAULT_HTTP_PORT);
        assert_eq!(http_base_url(), "http://127.0.0.1:8080");
    }

    #[test]
    fn test_http_port_override() {
        std::env::set_var("AUTO_HTTP_PORT", "18080");
        assert_eq!(http_port(), 18080);
        std::env::remove_var("AUTO_HTTP_PORT");
    }

    #[test]
    fn test_http_port_invalid_falls_back() {
        std::env::set_var("AUTO_HTTP_PORT", "not-a-port");
        assert_eq!(http_port(), DEFAULT_HTTP_PORT);
        std::env::remove_var("AUTO_HTTP_PORT");
    }
}
