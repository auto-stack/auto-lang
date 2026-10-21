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
    Ok(u64::from_be_bytes(
        hash.as_bytes()[0..8].try_into().unwrap(),
    ))
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
    frontends: &[auto_lang_core::config::BackendType],
    action: &str,
) -> AutoResult<usize> {
    // 1. Auto-select if only one option
    if frontends.len() == 1 {
        return Ok(0);
    }

    let backend_names: Vec<&str> = frontends.iter().map(|t| t.as_str()).collect();

    // 2. Check AUTO_RENDER environment variable
    if let Ok(render_env) = std::env::var("AUTO_RENDER") {
        let render_lower = render_env.to_lowercase();
        for (i, name) in backend_names.iter().enumerate() {
            if name.to_lowercase() == render_lower {
                println!(
                    "{} Using render target from AUTO_RENDER: {}",
                    "→".bright_green(),
                    name.bright_cyan()
                );
                return Ok(i);
            }
        }
        eprintln!(
            "Warning: AUTO_RENDER='{}' not found, falling back to selection",
            render_env
        );
    }

    // 3. Non-TTY fallback
    if !atty_check() {
        println!(
            "{} Auto-selecting render target: {}",
            "→".bright_green(),
            backend_names[0].bright_cyan()
        );
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
                println!(
                    "{} Selected: {}",
                    "→".bright_green(),
                    backend_names[n - 1].bright_cyan()
                );
                return Ok(n - 1);
            }
            _ => {
                println!(
                    "  {} Please enter a number between 1 and {}",
                    "Invalid.".bright_yellow(),
                    frontends.len()
                );
            }
        }
    }
}

/// Check if stdin is a TTY (terminal).
fn atty_check() -> bool {
    use std::io::IsTerminal;
    io::stdin().is_terminal()
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
}

/// Transpile AutoLang file to TypeScript（PLAN-816 T-03：语言 adapter 面——
/// 与外层 auto-lang 的历史包装同构，全部算法经 auto-lang-core 公开面：
/// Parser/TypeScriptTrans/ts_runtime；供 target.ts 臂消费）。
pub fn trans_typescript(path: &str) -> AutoResult<String> {
    use auto_lang_core::parser::Parser;
    use auto_lang_core::trans::typescript::ts_runtime;
    use auto_lang_core::trans::typescript::TypeScriptTrans;

    let code = std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {e}"))?;
    let tsname = path.replace(".at", ".ts");
    let fname = auto_val::AutoPath::new(path).filename();

    let mut parser = Parser::from(code.as_str());
    let ast = parser.parse().map_err(|e| e.to_string())?;
    let mut sink = auto_lang_core::trans::Sink::new(fname.clone());
    let mut trans = TypeScriptTrans::new(fname);
    use auto_lang_core::trans::Trans as _;
    trans.trans(ast, &mut sink)?;

    std::fs::write(&tsname, sink.done()?.clone())?;

    if trans.needs_range || trans.needs_print {
        let runtime_dir = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        let runtime_path = runtime_dir.join("runtime.ts");
        std::fs::write(&runtime_path, ts_runtime::runtime_file_content())?;
    }

    Ok(format!("[trans] {} -> {}", path, tsname))
}
