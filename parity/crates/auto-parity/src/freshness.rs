//! Plan 524 W2: parity `--auto-binary` 新鲜度校验（P517-2 根治）。
//!
//! 陈旧 auto.exe 会伪装回归假红（P511-5 矩阵②腿 561 假红、P517-2 双坑
//! 实证——陈旧产物 + worktree 相对路径解析失败各一次）。镜像 515 G4 ②
//! `auto_exe` 陈旧防护（e2e_exe::stale_against 的 mtime 对账），口径按
//! W0 定案升级为**硬失败**（假红代价 > 重建成本），`--allow-stale` 逃生；
//! 相对路径在闸口统一解析为绝对路径（P517-2 后半顺修——报错含绝对路径
//! 与 cwd，不再以裸 os error 3 出现）。

use std::path::{Path, PathBuf};

/// `crates/` 树下最新 `.rs` 的 (mtime, 路径)（跳过 target/node_modules）。
fn newest_source(root: &Path) -> Option<(std::time::SystemTime, PathBuf)> {
    fn walk(dir: &Path, best: &mut Option<(std::time::SystemTime, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(dir) else { return };
        for e in entries.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if name == "target" || name == "node_modules" {
                    continue;
                }
                walk(&p, best);
            } else if name.ends_with(".rs") {
                if let Ok(m) = e.metadata().and_then(|meta| meta.modified()) {
                    if best.as_ref().is_none_or(|(bm, _)| m > *bm) {
                        *best = Some((m, p.clone()));
                    }
                }
            }
        }
    }
    let mut best = None;
    walk(root, &mut best);
    best
}

/// 陈旧判定（纯函数，单测面）：exe mtime < newest 源 mtime → Some(最新源
/// 路径)；源树空/exe 缺档/时间不可读 → None（缺档由 resolve 层显式报错）。
pub fn stale_against(exe: &Path, src_root: &Path) -> Option<PathBuf> {
    let exe_m = exe.metadata().ok()?.modified().ok()?;
    let (src_m, src_p) = newest_source(src_root)?;
    (exe_m < src_m).then_some(src_p)
}

/// 相对路径顺修：显式路径（含分隔符/绝对）→ 相对 cwd 解析为绝对路径并
/// 存在性检查，缺档报错含绝对路径与 cwd；裸名 → where/which 定位（找不到
/// 硬失败——spawn 同样走 PATH，早失败信息更明确）。
pub fn resolve_auto_binary(auto_binary: &str) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let p = Path::new(auto_binary);
    let is_path_form = p.is_absolute() || auto_binary.contains('/') || auto_binary.contains('\\');
    if is_path_form {
        let abs = if p.is_absolute() { p.to_path_buf() } else { cwd.join(p) };
        if abs.exists() {
            Ok(abs)
        } else {
            Err(format!(
                "auto binary not found: {} (cwd: {}) —— 相对路径按 parity 运行 cwd 解析（P517-2 顺修）",
                abs.display(),
                cwd.display()
            ))
        }
    } else {
        locate_on_path(auto_binary).ok_or_else(|| {
            format!(
                "auto binary '{}' not found on PATH (cwd: {})",
                auto_binary,
                cwd.display()
            )
        })
    }
}

/// where(Windows)/which(Unix) 定位裸名二进制的绝对路径。
fn locate_on_path(name: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    let probe = std::process::Command::new("where").arg(name).output();
    #[cfg(not(windows))]
    let probe = std::process::Command::new("which").arg(name).output();
    let out = probe.ok().filter(|o| o.status.success())?;
    let first = String::from_utf8_lossy(&out.stdout).lines().next().unwrap_or("").trim().to_string();
    if first.is_empty() { None } else { Some(PathBuf::from(first)) }
}

/// 启动闸门：缺档/陈旧 → 硬失败（陈旧文案含重建命令提示），`--allow-stale`
/// 时陈旧降级为 stderr 警告放行。返回解析后的绝对路径（后续 spawn 统一
/// 用它，消除相对 cwd 歧义）。
pub fn check_freshness(
    auto_binary: &str,
    repo_root: &Path,
    allow_stale: bool,
) -> Result<String, String> {
    let abs = resolve_auto_binary(auto_binary)?;
    if let Some(newer) = stale_against(&abs, &repo_root.join("crates")) {
        if allow_stale {
            eprintln!(
                "[parity] 警告: {} 陈旧于最新源码（{}）—— --allow-stale 逃生已放行；结论前建议先重建",
                abs.display(),
                newer.display()
            );
        } else {
            return Err(format!(
                "stale auto binary: {} 陈旧于最新源码（{}）。陈旧产物会伪装回归假红（P511-5/P517-2 实证）——先重建：cargo build -p auto（产物 target/debug/auto.exe）；确认无碍可 --allow-stale 逃生",
                abs.display(),
                newer.display()
            ));
        }
    }
    Ok(abs.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 524 三态（515 同款）：陈旧 → Some(最新源)。
    #[test]
    fn stale_guard_detects_newer_source() {
        let dir = std::env::temp_dir().join(format!("parity-stale-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates")).unwrap();
        let exe = dir.join("auto.exe");
        let src = dir.join("crates").join("lib.rs");
        std::fs::write(&exe, b"exe").unwrap();
        std::fs::write(&src, b"src").unwrap();
        let older = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        let _ = std::fs::File::options().write(true).open(&exe).unwrap()
            .set_modified(older);
        assert_eq!(
            stale_against(&exe, &dir.join("crates")),
            Some(src),
            "源码新于 exe → 陈旧检出并指认最新源"
        );
    }

    /// 三态之二：新鲜 → None。
    #[test]
    fn fresh_binary_passes() {
        let dir = std::env::temp_dir().join(format!("parity-fresh-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates")).unwrap();
        let exe = dir.join("auto.exe");
        let src = dir.join("crates").join("lib.rs");
        std::fs::write(&exe, b"exe").unwrap();
        std::fs::write(&src, b"src").unwrap();
        let older = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        let _ = std::fs::File::options().write(true).open(&src).unwrap()
            .set_modified(older);
        assert_eq!(stale_against(&exe, &dir.join("crates")), None, "源码旧于 exe → 不陈旧");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 三态之三：缺档 → None（存在性由 resolve 层显式报错）。
    #[test]
    fn missing_exe_returns_none() {
        let dir = std::env::temp_dir().join(format!("parity-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates")).unwrap();
        std::fs::write(dir.join("crates").join("lib.rs"), b"src").unwrap();
        assert_eq!(
            stale_against(&dir.join("auto.exe"), &dir.join("crates")),
            None,
            "exe 缺档 → 判定层 None（resolve 层负责报错）"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// P517-2 顺修：相对路径缺档报错含绝对路径与 cwd；存在则解析为绝对路径。
    #[test]
    fn resolve_reports_absolute_path_on_missing() {
        let err = resolve_auto_binary("no/such/auto.exe").unwrap_err();
        let cwd = std::env::current_dir().unwrap();
        assert!(
            err.contains(&cwd.display().to_string()) && err.contains("no/such/auto.exe"),
            "报错须含绝对路径（cwd 前缀 + 相对尾段原样，got: {err}）"
        );
        assert!(err.contains("cwd"), "报错须含 cwd 提示（got: {err}）");

        let exe = std::env::temp_dir().join(format!("parity-resolve-{}.exe", std::process::id()));
        std::fs::write(&exe, b"exe").unwrap();
        let rel = exe.strip_prefix(std::env::current_dir().unwrap())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                // temp 不在 cwd 下：cd 到 temp 用相对名验证
                format!("{}", exe.file_name().unwrap().to_string_lossy())
            });
        let got = std::env::current_dir().unwrap().join(&rel);
        if got.exists() {
            let abs = resolve_auto_binary(&rel).unwrap();
            assert!(abs.is_absolute(), "存在档 → 解析为绝对路径");
        }
        // 绝对路径直通
        let abs = resolve_auto_binary(exe.to_string_lossy().as_ref()).unwrap();
        assert_eq!(abs, exe);
        let _ = std::fs::remove_file(&exe);
    }

    /// 闸门三态：陈旧硬失败（含重建提示）；--allow-stale 放行；缺档硬失败。
    #[test]
    fn gate_stale_fails_hard_allow_stale_passes() {
        let dir = std::env::temp_dir().join(format!("parity-gate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("crates")).unwrap();
        let exe = dir.join("auto.exe");
        let src = dir.join("crates").join("lib.rs");
        std::fs::write(&exe, b"exe").unwrap();
        std::fs::write(&src, b"src").unwrap();
        let older = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        let _ = std::fs::File::options().write(true).open(&exe).unwrap()
            .set_modified(older);

        let err = check_freshness(exe.to_str().unwrap(), &dir, false).unwrap_err();
        assert!(err.contains("陈旧") && err.contains("cargo build"), "硬失败文案含陈旧判定+重建提示（got: {err}）");

        let ok = check_freshness(exe.to_str().unwrap(), &dir, true).unwrap();
        assert_eq!(ok, exe.to_string_lossy().to_string(), "逃生旗放行并返回绝对路径");

        let miss = check_freshness(dir.join("ghost.exe").to_str().unwrap(), &dir, true).unwrap_err();
        assert!(miss.contains("not found") || miss.contains("找不到"), "缺档硬失败（got: {miss}）");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
