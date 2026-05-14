use auto_lang::run_file;
use std::path::Path;

fn main() {
    // If a file path is passed as arg, run just that file
    if let Some(path) = std::env::args().nth(1) {
        match run_file(&path) {
            Ok(output) => {
                println!("OK");
                if !output.trim().is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => {
                eprintln!("FAIL: {}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    // Default: scan all cookbook test files
    let base = Path::new("crates/auto-lang/test/cookbook");
    let mut files: Vec<String> = Vec::new();
    collect_at_files(base, &mut files);
    files.sort();

    let mut ok = 0;
    let mut fail = 0;
    let mut fails: Vec<String> = Vec::new();
    for path in &files {
        print!("{} ... ", path);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        match run_file(path) {
            Ok(output) => {
                println!("OK ({})", output.trim().len());
                ok += 1;
            }
            Err(e) => {
                let msg = format!("FAIL: {}", e);
                println!("{}", msg);
                fail += 1;
                fails.push(path.clone());
            }
        }
    }
    println!("\n{} OK, {} FAIL out of {}", ok, fail, files.len());
    if !fails.is_empty() {
        println!("\nFailing tests:");
        for f in &fails {
            println!("  {}", f);
        }
    }
}

fn collect_at_files(dir: &Path, out: &mut Vec<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                collect_at_files(&path, out);
            } else if path.extension().map_or(false, |e| e == "at") {
                if let Some(s) = path.to_str() {
                    out.push(s.replace('\\', "/"));
                }
            }
        }
    }
}
