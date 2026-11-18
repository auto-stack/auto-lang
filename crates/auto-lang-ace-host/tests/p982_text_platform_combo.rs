//! PLAN-982（983 组合入口，用户裁定）：**通用 AV 入口的最小组合用例**——
//! 一次调用收**借用 `str` 路径**实参，在**同一次执行**里做**受检平台 I/O**
//! （显式授权 `platform.file`/`platform.mem`），并返回响应。
//! Jade Garden 的索引与提交顺序留在业务层（983）。
//!
//! 三条验收（用户给定）：
//! 1. **授权成功**：带授权 ⇒ 打开并逐字节读取文件，按内容返回；
//! 2. **缺授权 ⇒ 执行前拒绝**：同源不给 `platform.file` ⇒ `host.unauthorized`
//!    具名拒绝，不进入执行；
//! 3. **退出后资源释放**：handler 显式 `mem_free`/`file_close`，session summary
//!    报 `leaked_regions_cleaned=0`、`live_mapped_at_exit=0`，arena 收支归零。
//!
//! 载荷：AVHC（host-console）面——文本面（`str` 参数）+ 平台原语（`file_*`/`mem_*`
//! 具名 intrinsic，能力键随 import 声明），经 `console_host::compile` →
//! `load()` → `execute_entry_text_platform`（文本实参 × 平台授权的组合入口）。

#![cfg(all(windows, target_arch = "x86_64"))]

use ace_av_core::av::{execute_entry_text_platform, CancelFlag, ExecLimits};
use ace_av_core::contract::ExecOutcome;
use ace_av_core::host::AdmissionError;
use ace_av_core::platform_io::Win32PlatformIo;

/// 一次调用：收借用 `str` 路径 → 在 Auto 源内做 UTF-16LE 暂存（1041 纪律：
/// 转换发生在 Auto 源里，绝不在宿主助手）→ `file_open_w16` 打开 → 逐字节读出
/// → 显式释放 → 按内容应答。文件内容固定为 `"ok"`（0x6F 0x6B = 111/107）。
///
/// （Windows 面：`file_open_bytes` 是 Linux 腿——Win32 后端具名 `ERR_UNSUPPORTED`；
/// 本用例因此走 `file_open_w16`，与冻结 sys 标准库同一纪律。）
const SRC: &str = r#"
fn main(path str) int {
    let n = path.len()
    if n <= 0 { return -101 }
    let region = mem_alloc((n * 2 + 2) * 2)
    if region <= 0 { return -102 }
    var units = 0
    var i = 0
    for i < n {
        let cp = path.char_at(i).to_codepoint()
        if cp >= 65536 {
            let v = cp - 65536
            let hi = 55296 + v / 1024
            let lo = 56320 + v % 1024
            let e1 = mem_write_u8(region, units * 2, hi % 256)
            if e1 != 0 { return -8 }
            let e2 = mem_write_u8(region, units * 2 + 1, (hi / 256) % 256)
            if e2 != 0 { return -8 }
            let e3 = mem_write_u8(region, units * 2 + 2, lo % 256)
            if e3 != 0 { return -8 }
            let e4 = mem_write_u8(region, units * 2 + 3, (lo / 256) % 256)
            if e4 != 0 { return -8 }
            units = units + 2
        } else {
            let e1 = mem_write_u8(region, units * 2, cp % 256)
            if e1 != 0 { return -8 }
            let e2 = mem_write_u8(region, units * 2 + 1, (cp / 256) % 256)
            if e2 != 0 { return -8 }
            units = units + 1
        }
        i = i + 1
    }
    let h = file_open_w16(region, 0, units, 1)
    let f1 = mem_free(region)
    if f1 != 0 { return -104 }
    if h <= 0 { return h }
    let n2 = file_size(h)
    if n2 != 2 {
        let c1 = file_close(h)
        if c1 != 0 { return -105 }
        return -106
    }
    let buf = mem_alloc(2)
    if buf <= 0 { return -107 }
    let r = file_read_some(h, buf, 0, 2)
    if r != 2 {
        let c2 = file_close(h)
        let f2 = mem_free(buf)
        if c2 != 0 { return -108 }
        if f2 != 0 { return -109 }
        return -110
    }
    let a = mem_read_u8(buf, 0)
    let b = mem_read_u8(buf, 1)
    let f3 = mem_free(buf)
    if f3 != 0 { return -111 }
    let c3 = file_close(h)
    if c3 != 0 { return -112 }
    if a != 111 { return -1 }
    if b != 107 { return -2 }
    return 1
}
"#;

fn grants() -> Vec<String> {
    vec!["platform.file".to_string(), "platform.mem".to_string()]
}

fn scratch(tag: &str) -> std::path::PathBuf {
    let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../target/p982-combo/{tag}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn p982_combined_entry_receives_a_borrowed_str_and_reads_by_grants() {
    let tmp = scratch("ok");
    std::fs::write(tmp.join("f.txt"), "ok").unwrap();
    let path_text = tmp.join("f.txt").to_string_lossy().replace('\\', "/");

    // 编译一次、驻留：授权腿/缺授权腿/第二请求都用**同一份**产物。
    let program = ace_av::console_host::compile(SRC, "d_main")
        .expect("the str-path handler with platform file ops compiles");
    let loaded = program.load().expect("the compiled module loads");

    // (1) 授权成功：真打开 + 逐字节读取，按内容返回 1。
    let mut backend = Win32PlatformIo::default();
    let (out, trace, summary) = execute_entry_text_platform(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &grants(),
        &[path_text.as_bytes()],
        None,
        &mut backend,
    )
    .expect("the granted leg runs");
    let arena_live = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    eprintln!("p982 combo granted: {out:?} arena_live={arena_live} summary={summary:?}");
    assert!(
        matches!(
            out,
            ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(1),
                ..
            }
        ),
        "the granted leg reads the file byte-by-byte and answers by content: {out:?}"
    );
    assert_eq!(arena_live, 0, "no text allocation outlives the entry");

    // (3) 退出后资源释放：handler 显式释放，ledger 收干净。
    assert_eq!(
        summary.leaked_regions_cleaned, 0,
        "no leaked platform region"
    );
    assert_eq!(summary.live_mapped_at_exit, 0, "no live mapped bytes");

    // (2) 缺授权 ⇒ 执行前拒绝：`platform.file` 不给 ⇒ `host.unauthorized` 具名。
    let short_grants = vec!["platform.mem".to_string()];
    let mut backend2 = Win32PlatformIo::default();
    let refusal: AdmissionError = execute_entry_text_platform(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &short_grants,
        &[path_text.as_bytes()],
        None,
        &mut backend2,
    )
    .expect_err("the missing grant is refused before execution");
    eprintln!("p982 combo refused: {refusal:?}");
    assert_eq!(refusal.code, "host.unauthorized");
    assert!(
        refusal.message.contains("platform.file"),
        "the refusal names the missing capability: {refusal:?}"
    );

    // 同一驻留产物服务第二个请求；**不同内容**走另一支（同产物、逐请求数据）。
    std::fs::write(tmp.join("g.txt"), "AB").unwrap();
    let other = tmp.join("g.txt").to_string_lossy().replace('\\', "/");
    let mut backend3 = Win32PlatformIo::default();
    let (out3, _t3, s3) = execute_entry_text_platform(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &grants(),
        &[other.as_bytes()],
        None,
        &mut backend3,
    )
    .expect("the resident product serves the second call");
    assert!(
        matches!(
            out3,
            ExecOutcome::Success {
                value: ace_av_core::contract::Value::I32(-1),
                ..
            }
        ),
        "different bytes take the content branch: {out3:?}"
    );
    assert_eq!(s3.leaked_regions_cleaned, 0, "the second run releases too");

    let _ = std::fs::remove_dir_all(&tmp);
}
