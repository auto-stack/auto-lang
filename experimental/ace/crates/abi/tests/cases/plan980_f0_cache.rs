//! PLAN-980 HELLO：F0 provider 构建的**新鲜度缓存**（显式预构建阶段的测试
//! 纪律支撑——测试及其子进程不得编译前置，见 2026-11-14 裁定）。
//!
//! 缓存身份 = (provider digest, shim digest, rustc identity) + 双产物在位。
//! 只认输入身份，不认时间戳；编辑源（identity/rebind 腿）digest 变化即重建。

use ace_abi::rust_provider::{build_f0, build_source, f0_provider_source};
use std::time::Instant;

struct Scope(std::path::PathBuf);
impl Drop for Scope {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn scope(tag: &str) -> Scope {
    let root = std::env::temp_dir().join(format!("p980-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    Scope(root)
}

#[test]
fn p980_f0_cache_second_build_skips_rustc_and_tamper_rebuilds() {
    let root = scope("cache");
    let root_path = root.0.clone();

    // 首建：走真实 rustc（两次编译），产出 stamp 与双产物。
    let t0 = Instant::now();
    let first = build_f0(&root_path).expect("first build compiles");
    let first_wall = t0.elapsed();
    let stamp_path = root_path.join("out/f0-build-stamp.txt");
    assert!(stamp_path.is_file(), "stamp written on real build");
    assert!(first.cdylib.is_file() && first.staticlib.is_file());

    // 二建（同身份）：命中缓存——不重新编译，快路径返回同一产物身份。
    let t1 = Instant::now();
    let second = build_f0(&root_path).expect("cached build returns");
    let second_wall = t1.elapsed();
    assert_eq!(second.cdylib, first.cdylib, "same artifact identity");
    assert_eq!(second.provider_digest, first.provider_digest);
    assert_eq!(second.rustc_identity, first.rustc_identity);
    assert!(
        second_wall < first_wall,
        "cached build must be faster: first={first_wall:?} second={second_wall:?}"
    );
    assert!(
        second_wall.as_secs_f64() < 3.0,
        "cache path stays fast: {second_wall:?}"
    );

    // 篡改 stamp：身份失配 → 强制真实重建并恢复 stamp。
    std::fs::write(&stamp_path, "tampered\n").unwrap();
    let third = build_f0(&root_path).expect("tamper forces rebuild");
    assert_eq!(third.provider_digest, first.provider_digest);
    assert!(std::fs::read_to_string(&stamp_path)
        .unwrap()
        .starts_with(&first.provider_digest));

    // 删除 cdylib：产物缺失 → 重建恢复。
    std::fs::remove_file(&first.cdylib).unwrap();
    build_f0(&root_path).expect("missing artifact forces rebuild");
    assert!(first.cdylib.is_file(), "cdylib restored");
}

#[test]
fn p980_f0_cache_edited_source_rebuilds_by_digest() {
    let root = scope("cache-edit");
    // 冻结源先建（缓存），再以**编辑源**（identity/rebind 腿的合法用途）重建：
    // digest 变化必须绕过缓存走真实 rustc，产出新身份的 stamp。
    build_f0(&root.0).expect("frozen build");
    let edited = format!("{}\n// plan980 cache probe edit\n", f0_provider_source());
    let rebuilt = build_source(&root.0, &edited).expect("edited source rebuilds");
    assert_ne!(
        rebuilt.provider_digest, "",
        "edited build carries its own digest"
    );
    let stamp = std::fs::read_to_string(root.0.join("out/f0-build-stamp.txt")).unwrap();
    assert!(
        stamp.starts_with(&rebuilt.provider_digest),
        "stamp tracks the edited identity"
    );
    // 同一编辑源再建：再次命中缓存（快路径）。
    let t = Instant::now();
    build_source(&root.0, &edited).expect("edited identity caches too");
    assert!(
        t.elapsed().as_secs_f64() < 3.0,
        "second edited build is cached"
    );
}
