//! auto-bindgen: C header manifest generator CLI.
//!
//! Usage:
//!   auto-bindgen --header <stdio.h> --output stdio.json
//!
//! Plan 216 Phase 1.

use auto_bindgen::extractor::get_builtin_manifest;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "auto-bindgen", about = "Generate C header manifests for AutoLang C FFI")]
struct Cli {
    /// C header to generate manifest for (e.g. "string.h")
    #[arg(long)]
    header: String,

    /// Output JSON file path
    #[arg(long)]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let manifest = match get_builtin_manifest(&cli.header) {
        Some(m) => m,
        None => {
            eprintln!(
                "No built-in manifest for '{}'. Only standard headers are supported: \
                 string.h, math.h, stdio.h, stdlib.h, time.h",
                cli.header
            );
            std::process::exit(1);
        }
    };

    let json = serde_json::to_string_pretty(&manifest).unwrap();

    match cli.output {
        Some(path) => {
            std::fs::write(&path, &json).unwrap_or_else(|e| {
                eprintln!("Failed to write {}: {}", path, e);
                std::process::exit(1);
            });
            println!("Wrote manifest to {}", path);
        }
        None => println!("{}", json),
    }
}
