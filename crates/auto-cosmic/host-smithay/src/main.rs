//! `auto-smithay-host` —— Plan 509 路线 B 宿主二进制（Linux 合成；非
//! Linux 为 cfg stub 报错退出，保 Windows dev 绿）。

fn main() {
    // 手工解析（骨架零额外依赖）：--frame <png> / --frames <n>。
    let mut frame_png: Option<std::path::PathBuf> = None;
    let mut max_frames: Option<u64> = None;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--frame" if i + 1 < args.len() => {
                frame_png = Some(args[i + 1].clone().into());
                i += 2;
            }
            "--frames" if i + 1 < args.len() => {
                max_frames = args[i + 1].parse().ok();
                i += 2;
            }
            other => {
                eprintln!("[auto-smithay-host] unknown arg: {other}");
                std::process::exit(2);
            }
        }
    }

    if let Err(e) = auto_cosmic_host_smithay::run_host(frame_png.as_deref(), max_frames) {
        eprintln!("[auto-smithay-host] {e}");
        std::process::exit(1);
    }
}
