// Cookbook VM Output Comparison Tests
// Runs each .at file via AutoVM and compares stdout against .expected.out (if present).
// Tests without .expected.out are smoke tests (only check no crash).
// Non-deterministic tests (random, time, network) use #[ignore] for output comparison.

use crate::error::AutoResult;
use crate::run_with_capture;
use std::fs;
use std::path::PathBuf;

const COOKBOOK_DIR: &str = "test/cookbook";

fn test_cookbook(category: &str, name: &str) -> AutoResult<()> {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let case_dir = d.join("test/cookbook").join(category).join(name);

    // Find the .at file in the case directory (maxdepth 1, like batch script)
    let at_file = fs::read_dir(&case_dir)
        .map_err(|e| std::io::Error::new(e.kind(), format!("Cannot read dir {:?}: {}", case_dir, e)))?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map_or(false, |ext| ext == "at"))
        .ok_or_else(|| std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("No .at file found in {}/{}", category, name),
        ))?;
    let src = fs::read_to_string(at_file.path())?;

    let original_cwd = std::env::current_dir()?;
    std::env::set_current_dir(&case_dir)
        .map_err(|e| std::io::Error::new(e.kind(), format!("Cannot cd to {:?}: {}", case_dir, e)))?;

    let result = run_with_capture(&src);
    let (_vm_result, stdout) = match result {
        Ok(v) => v,
        Err(e) => {
            std::env::set_current_dir(&original_cwd)?;
            return Err(e);
        }
    };

    std::env::set_current_dir(&original_cwd)?;

    let out_path = case_dir.join("expected.out");
    let generate = std::env::var("GENERATE_EXPECTED").is_ok();

    if generate && !out_path.is_file() {
        fs::write(&out_path, &stdout)?;
        eprintln!("GENERATED {}", out_path.display());
    } else if out_path.is_file() {
        let expected = fs::read_to_string(&out_path)?;
        if stdout != expected {
            let wrong_path = case_dir.join("wrong.out");
            fs::write(&wrong_path, &stdout)?;
        }
        assert_eq!(stdout, expected, "Output mismatch for {}/{}", category, name);
    }

    Ok(())
}

// === algorithms ===
#[test] fn cb_algorithms_sort_int() { test_cookbook("algorithms", "001_sort_int").unwrap(); }
#[test] fn cb_algorithms_sort_float() { test_cookbook("algorithms", "002_sort_float").unwrap(); }
#[test] fn cb_algorithms_sort_struct() { test_cookbook("algorithms", "003_sort_struct").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand() { test_cookbook("algorithms", "004_rand").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_choose() { test_cookbook("algorithms", "005_rand_choose").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_custom() { test_cookbook("algorithms", "006_rand_custom").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_dist() { test_cookbook("algorithms", "007_rand_dist").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_passwd() { test_cookbook("algorithms", "008_rand_passwd").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_range() { test_cookbook("algorithms", "009_rand_range").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_custom_2() { test_cookbook("algorithms", "010_rand_custom").unwrap(); }
#[test] #[ignore = "random output"] fn cb_algorithms_rand_dist_2() { test_cookbook("algorithms", "011_rand_dist").unwrap(); }

// === asynchronous ===
#[test] fn cb_asynchronous_channel() { test_cookbook("asynchronous", "channel").unwrap(); }
#[test] fn cb_asynchronous_fs() { test_cookbook("asynchronous", "fs").unwrap(); }
#[test] fn cb_asynchronous_ftc() { test_cookbook("asynchronous", "ftc").unwrap(); }
#[test] fn cb_asynchronous_rt() { test_cookbook("asynchronous", "rt").unwrap(); }

// === cli ===
#[test] fn cb_cli_clap_basic() { test_cookbook("cli", "001_clap_basic").unwrap(); }
#[test] fn cb_cli_ansi_term() { test_cookbook("cli", "002_ansi_term").unwrap(); }

// === compression ===
#[test] fn cb_compression_tar_compress() { test_cookbook("compression", "001_tar_compress").unwrap(); }
#[test] fn cb_compression_tar_decompress() { test_cookbook("compression", "002_tar_decompress").unwrap(); }
#[test] #[ignore = "needs tar fixture"] fn cb_compression_tar_strip_prefix() { test_cookbook("compression", "003_tar_strip_prefix").unwrap(); }

// === concurrency ===
#[test] fn cb_concurrency_rayon_any_all() { test_cookbook("concurrency", "001_rayon_any_all").unwrap(); }
#[test] fn cb_concurrency_rayon_map_reduce() { test_cookbook("concurrency", "002_rayon_map_reduce").unwrap(); }
#[test] fn cb_concurrency_rayon_parallel_sort() { test_cookbook("concurrency", "003_rayon_parallel_sort").unwrap(); }
#[test] fn cb_concurrency_crossbeam_spsc() { test_cookbook("concurrency", "004_crossbeam_spsc").unwrap(); }
#[test] fn cb_concurrency_rayon_iter_mut() { test_cookbook("concurrency", "005_rayon_iter_mut").unwrap(); }
#[test] fn cb_concurrency_rayon_parallel_search() { test_cookbook("concurrency", "006_rayon_parallel_search").unwrap(); }
#[test] fn cb_concurrency_crossbeam_complex() { test_cookbook("concurrency", "007_crossbeam_complex").unwrap(); }
#[test] fn cb_concurrency_crossbeam_spawn() { test_cookbook("concurrency", "008_crossbeam_spawn").unwrap(); }
#[test] fn cb_concurrency_global_mut_state() { test_cookbook("concurrency", "009_global_mut_state").unwrap(); }
#[test] fn cb_concurrency_threadpool_walk() { test_cookbook("concurrency", "010_threadpool_walk").unwrap(); }

// === cryptography ===
#[test] fn cb_cryptography_sha_digest() { test_cookbook("cryptography", "001_sha_digest").unwrap(); }
#[test] fn cb_cryptography_pbkdf2() { test_cookbook("cryptography", "002_pbkdf2").unwrap(); }
#[test] fn cb_cryptography_hmac() { test_cookbook("cryptography", "003_hmac").unwrap(); }

// === database ===
#[test] #[ignore = "needs postgres"] fn cb_database_postgres() { test_cookbook("database", "postgres").unwrap(); }
#[test] #[ignore = "needs sqlite"] fn cb_database_sqlite() { test_cookbook("database", "sqlite").unwrap(); }

// === datetime ===
#[test] #[ignore = "time-dependent"] fn cb_datetime_elapsed_time() { test_cookbook("datetime", "001_elapsed_time").unwrap(); }
#[test] fn cb_datetime_checked() { test_cookbook("datetime", "002_checked").unwrap(); }
#[test] #[ignore = "time-dependent"] fn cb_datetime_timezone() { test_cookbook("datetime", "003_timezone").unwrap(); }
#[test] #[ignore = "time-dependent"] fn cb_datetime_current() { test_cookbook("datetime", "004_current").unwrap(); }
#[test] #[ignore = "time-dependent"] fn cb_datetime_format() { test_cookbook("datetime", "005_format").unwrap(); }
#[test] fn cb_datetime_parse_string() { test_cookbook("datetime", "006_parse_string").unwrap(); }
#[test] #[ignore = "time-dependent"] fn cb_datetime_timestamp() { test_cookbook("datetime", "007_timestamp").unwrap(); }

// === devtools ===
#[test] fn cb_devtools_log_debug() { test_cookbook("devtools", "001_log_debug").unwrap(); }
#[test] fn cb_devtools_log_error() { test_cookbook("devtools", "002_log_error").unwrap(); }
#[test] fn cb_devtools_log_stdout() { test_cookbook("devtools", "003_log_stdout").unwrap(); }
#[test] fn cb_devtools_log_custom() { test_cookbook("devtools", "004_log_custom").unwrap(); }
#[test] fn cb_devtools_log_syslog() { test_cookbook("devtools", "005_log_syslog").unwrap(); }
#[test] fn cb_devtools_log_env() { test_cookbook("devtools", "006_log_env").unwrap(); }
#[test] fn cb_devtools_log_mod() { test_cookbook("devtools", "007_log_mod").unwrap(); }
#[test] fn cb_devtools_log_timestamp() { test_cookbook("devtools", "008_log_timestamp").unwrap(); }
#[test] fn cb_devtools_log_custom_location() { test_cookbook("devtools", "009_log_custom_location").unwrap(); }
#[test] fn cb_devtools_tracing_console() { test_cookbook("devtools", "010_tracing_console").unwrap(); }

// === encoding ===
#[test] fn cb_encoding_json() { test_cookbook("encoding", "001_json").unwrap(); }
#[test] fn cb_encoding_toml() { test_cookbook("encoding", "002_toml").unwrap(); }
#[test] fn cb_encoding_csv_read() { test_cookbook("encoding", "003_csv_read").unwrap(); }
#[test] fn cb_encoding_base64() { test_cookbook("encoding", "004_base64").unwrap(); }
#[test] fn cb_encoding_hex() { test_cookbook("encoding", "005_hex").unwrap(); }
#[test] fn cb_encoding_endian_byte() { test_cookbook("encoding", "006_endian_byte").unwrap(); }
#[test] fn cb_encoding_csv_delimiter() { test_cookbook("encoding", "007_csv_delimiter").unwrap(); }
#[test] fn cb_encoding_csv_filter() { test_cookbook("encoding", "008_csv_filter").unwrap(); }
#[test] fn cb_encoding_csv_invalid() { test_cookbook("encoding", "009_csv_invalid").unwrap(); }
#[test] fn cb_encoding_csv_serde_serialize() { test_cookbook("encoding", "010_csv_serde_serialize").unwrap(); }
#[test] fn cb_encoding_csv_serialize() { test_cookbook("encoding", "011_csv_serialize").unwrap(); }
#[test] fn cb_encoding_csv_transform() { test_cookbook("encoding", "012_csv_transform").unwrap(); }
#[test] fn cb_encoding_percent_encode() { test_cookbook("encoding", "013_percent_encode").unwrap(); }
#[test] fn cb_encoding_url_encode() { test_cookbook("encoding", "014_url_encode").unwrap(); }

// === errors ===
#[test] fn cb_errors_boxed_error() { test_cookbook("errors", "001_boxed_error").unwrap(); }
#[test] fn cb_errors_anyhow() { test_cookbook("errors", "002_anyhow").unwrap(); }
#[test] fn cb_errors_backtrace() { test_cookbook("errors", "003_backtrace").unwrap(); }
#[test] fn cb_errors_retain() { test_cookbook("errors", "004_retain").unwrap(); }

// === file ===
#[test] fn cb_file_read_lines() { test_cookbook("file", "001_read_lines").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_find_files() { test_cookbook("file", "002_find_files").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_recursive_size() { test_cookbook("file", "003_recursive_size").unwrap(); }
#[test] fn cb_file_modified() { test_cookbook("file", "004_modified").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_duplicate_name() { test_cookbook("file", "005_duplicate_name").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_find_file() { test_cookbook("file", "006_find_file").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_ignore_case() { test_cookbook("file", "007_ignore_case").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_loops() { test_cookbook("file", "008_loops").unwrap(); }
#[test] fn cb_file_png() { test_cookbook("file", "009_png").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_recursive() { test_cookbook("file", "010_recursive").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_sizes() { test_cookbook("file", "011_sizes").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_skip_dot() { test_cookbook("file", "012_skip_dot").unwrap(); }
#[test] #[ignore = "CWD-dependent"] fn cb_file_same_file() { test_cookbook("file", "013_same_file").unwrap(); }
#[test] fn cb_file_read_lines_temp() { test_cookbook("file", "014_read_lines_temp").unwrap(); }

// === hardware ===
#[test] fn cb_hardware_cpu_count() { test_cookbook("hardware", "001_cpu_count").unwrap(); }

// === mem ===
#[test] fn cb_mem_lazy_cell() { test_cookbook("mem", "001_lazy_cell").unwrap(); }

// === os ===
#[test] fn cb_os_env_variable() { test_cookbook("os", "001_env_variable").unwrap(); }
#[test] fn cb_os_process_continuous() { test_cookbook("os", "002_process_continuous").unwrap(); }
#[test] fn cb_os_error_file() { test_cookbook("os", "003_error_file").unwrap(); }
#[test] #[ignore = "OS-dependent"] fn cb_os_piped() { test_cookbook("os", "004_piped").unwrap(); }
#[test] #[ignore = "OS-dependent"] fn cb_os_process_output() { test_cookbook("os", "005_process_output").unwrap(); }
#[test] #[ignore = "OS-dependent"] fn cb_os_send_input() { test_cookbook("os", "006_send_input").unwrap(); }

// === safety ===
#[test] fn cb_safety_heapless() { test_cookbook("safety", "001_heapless").unwrap(); }

// === text ===
#[test] fn cb_text_regex_replace() { test_cookbook("text", "001_regex_replace").unwrap(); }
#[test] fn cb_text_regex_email() { test_cookbook("text", "002_regex_email").unwrap(); }
#[test] fn cb_text_regex_hashtags() { test_cookbook("text", "003_regex_hashtags").unwrap(); }
#[test] fn cb_text_graphemes() { test_cookbook("text", "004_graphemes").unwrap(); }
#[test] fn cb_text_filter_log() { test_cookbook("text", "005_filter_log").unwrap(); }
#[test] fn cb_text_phone() { test_cookbook("text", "006_phone").unwrap(); }
#[test] fn cb_text_from_str() { test_cookbook("text", "007_from_str").unwrap(); }

// === versioning ===
#[test] fn cb_versioning_semver_parse() { test_cookbook("versioning", "001_semver_parse").unwrap(); }
#[test] fn cb_versioning_semver_increment() { test_cookbook("versioning", "002_semver_increment").unwrap(); }
#[test] fn cb_versioning_semver_latest() { test_cookbook("versioning", "003_semver_latest").unwrap(); }
#[test] fn cb_versioning_semver_command() { test_cookbook("versioning", "004_semver_command").unwrap(); }
#[test] fn cb_versioning_semver_complex() { test_cookbook("versioning", "005_semver_complex").unwrap(); }
#[test] fn cb_versioning_semver_prerelease() { test_cookbook("versioning", "006_semver_prerelease").unwrap(); }

// === web ===
#[test] fn cb_web_mime() { test_cookbook("web", "mime").unwrap(); }
#[test] #[ignore = "needs network"] fn cb_web_scraping() { test_cookbook("web", "scraping").unwrap(); }
