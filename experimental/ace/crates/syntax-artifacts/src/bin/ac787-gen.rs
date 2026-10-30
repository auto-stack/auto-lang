//! PLAN-787 V-11 源生成器（独立进程，F787-R1-08：生成器与 runtime 分列）：
//! 流式写短声明到目标文件（不整源驻留），周期采样并上报自身 OS private
//! bytes 峰值。输出（stdout，末行）：
//! `GEN done decls=<n> bytes=<b> peak_private_bytes=<p>`

use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: ac787-gen <out-file> <target-mib>");
        std::process::exit(2);
    }
    let out_path = &args[1];
    let target_mib: usize = args[2].parse().expect("target-mib");
    let target = target_mib << 20;
    let mut f = std::io::BufWriter::new(std::fs::File::create(out_path).expect("create"));
    let mut i = 0usize;
    let mut written = 0usize;
    let mut peak = 0u64;
    let mut buf = String::with_capacity(1 << 16);
    while written < target {
        buf.clear();
        while buf.len() < (1 << 16) && written + buf.len() < target {
            buf.push_str(&format!(
                "api f{i}(a: i32, b: i32) -> i32;\nnote \"n{i}\";\n"
            ));
            i += 1;
        }
        f.write_all(buf.as_bytes()).expect("write");
        written += buf.len();
        if i % 20_000 == 0 {
            let m = ace_artifacts::sysinfo::current_process_memory();
            peak = peak.max(m.private_bytes);
        }
    }
    f.flush().expect("flush");
    let m = ace_artifacts::sysinfo::current_process_memory();
    peak = peak.max(m.private_bytes);
    println!("GEN done decls={i} bytes={written} peak_private_bytes={peak}");
}
