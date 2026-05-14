use auto_lang::run_file;

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

    // Default: run all A-tier files
    let tier_a = [
        "crates/auto-lang/test/cookbook/algorithms/001_sort_int/sort_int.at",
        "crates/auto-lang/test/cookbook/algorithms/002_sort_float/sort_float.at",
        "crates/auto-lang/test/cookbook/algorithms/003_sort_struct/sort_struct.at",
        "crates/auto-lang/test/cookbook/file/001_read_lines/read_lines.at",
        "crates/auto-lang/test/cookbook/os/001_env_variable/env_variable.at",
        "crates/auto-lang/test/cookbook/os/002_process_continuous/process_continuous.at",
        "crates/auto-lang/test/cookbook/os/003_error_file/error_file.at",
        "crates/auto-lang/test/cookbook/datetime/001_elapsed_time/elapsed_time.at",
        "crates/auto-lang/test/cookbook/science/mathematics/statistics/001_central_tendency/central_tendency.at",
        "crates/auto-lang/test/cookbook/science/mathematics/statistics/002_standard_deviation/standard_deviation.at",
        "crates/auto-lang/test/cookbook/science/mathematics/trigonometry/001_tan_sin_cos/tan_sin_cos.at",
        "crates/auto-lang/test/cookbook/science/mathematics/trigonometry/002_side_length/side_length.at",
        "crates/auto-lang/test/cookbook/science/mathematics/trigonometry/003_latitude_longitude/latitude_longitude.at",
        "crates/auto-lang/test/cookbook/mem/001_lazy_cell/lazy_cell.at",
        "crates/auto-lang/test/cookbook/errors/001_boxed_error/boxed_error.at",
    ];
    let mut ok = 0;
    let mut fail = 0;
    for path in &tier_a {
        print!("{} ... ", path);
        std::io::Write::flush(&mut std::io::stdout()).ok();
        match run_file(path) {
            Ok(output) => {
                println!("OK ({})", output.trim().len());
                ok += 1;
            }
            Err(e) => {
                println!("FAIL: {}", e);
                fail += 1;
            }
        }
    }
    println!("\n{} OK, {} FAIL out of {}", ok, fail, tier_a.len());
}
