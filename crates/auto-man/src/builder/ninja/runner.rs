// ninja 构建器宿主程序探测（Plan 580）
//
// NinjaBuilder 对 ninja 的调用经由 resolve_ninja_host() 解析宿主程序：
//   AUTO_NINJA env 覆盖 → PATH ninja → PATH n2 → cargo install n2 兜底
// 决策结果进程级缓存（pkg.rs auto_detect 的 OnceLock 先例）。
// 探测核心 find_in 设计为可注入路径列表——单测零全局状态污染。

use crate::AutoResult;
use log::*;
use std::path::PathBuf;
use std::sync::OnceLock;

/// 最终用于 Command::new 的 ninja 宿主程序（程序名或绝对路径）
#[derive(Debug, Clone)]
pub struct NinjaHost {
    pub program: String,
}

/// 进程级决策缓存（仅缓存成功决策，失败路径保持可重试）
static HOST: OnceLock<NinjaHost> = OnceLock::new();

/// 兜底安装的 n2 上游（未上 crates.io，只能 git 安装；
/// rev 钉死在本机与 automan 生成子集对拍验证过的 commit）
const N2_GIT_URL: &str = "https://github.com/evmar/n2";
const N2_REV: &str = "b1fead52";

/// 全无形态的指引文案（含 ninja 与 n2 两条手动安装路径）
fn no_runner_msg() -> String {
    format!(
        "no build runner found: install ninja (https://ninja-build.org), \
         or `cargo install --locked --git {} --rev {}`, \
         or point AUTO_NINJA at a ninja-compatible executable",
        N2_GIT_URL, N2_REV
    )
}

/// 解析 ninja 宿主程序（首次决策后进程级缓存）
pub fn resolve_ninja_host() -> AutoResult<NinjaHost> {
    if let Some(host) = HOST.get() {
        return Ok(host.clone());
    }
    let host = resolve_impl()?;
    let _ = HOST.set(host.clone());
    Ok(host)
}

fn resolve_impl() -> AutoResult<NinjaHost> {
    // 1. AUTO_NINJA 显式覆盖：值视为程序名/路径直接采用，不做存在性校验
    //    （与 CC 惯例一致，让 spawn 错误自然暴露）
    if let Ok(prog) = std::env::var("AUTO_NINJA") {
        if !prog.trim().is_empty() {
            info!("[ninja] build runner from AUTO_NINJA: {}", prog);
            return Ok(NinjaHost { program: prog });
        }
    }

    // 2/3. PATH 上的 ninja → n2
    for base in ["ninja", "n2"] {
        if let Some(found) = find_on_path(base) {
            info!("[ninja] build runner: {}", found.display());
            return Ok(NinjaHost {
                program: found.to_string_lossy().into_owned(),
            });
        }
    }

    // 4-6. cargo install n2 兜底臂（try_install_n2，Plan 580 T3）
    if let Some(found) = try_install_n2() {
        info!("[ninja] build runner after install: {}", found.display());
        return Ok(NinjaHost {
            program: found.to_string_lossy().into_owned(),
        });
    }

    Err(no_runner_msg().into())
}

/// 从 PATH 环境变量探测 base（生产包装；单测用可注入的 find_in）
fn find_on_path(base: &str) -> Option<PathBuf> {
    let path_var = std::env::var("PATH").ok()?;
    find_in(std::env::split_paths(&path_var), base)
}

/// 在给定目录列表中查找 base 可执行文件（windows 补 .exe 变体）
fn find_in(paths: impl IntoIterator<Item = PathBuf>, base: &str) -> Option<PathBuf> {
    for dir in paths {
        for name in exe_names(base) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

/// base 的可执行文件名变体：windows 上 [.exe 优先，裸名兜底]，unix 上 [裸名]
fn exe_names(base: &str) -> Vec<String> {
    if cfg!(windows) {
        vec![format!("{}.exe", base), base.to_string()]
    } else {
        vec![base.to_string()]
    }
}

/// ~/.cargo/bin（cargo install 的默认安装目录）
fn cargo_bin_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".cargo").join("bin"))
}

/// 尝试安装 n2：探测 cargo → `cargo install --locked --git ... --rev ...`
/// → 复检 PATH 与 ~/.cargo/bin。成败以复检探测结果为准（已安装时 cargo
/// 以非零退出提示 already exists，不解析 cargo 输出）。任何一步不可行
/// 都返回 None，由调用方落 Err 指引文案。
fn try_install_n2() -> Option<PathBuf> {
    // 4. 没有 cargo 就没有自动安装臂
    if find_on_path("cargo").is_none() {
        warn!("[ninja] cargo not found on PATH, cannot auto-install n2");
        return None;
    }

    // 5. 一次性安装（stdio 直通，让用户看到 cargo 进度）
    info!(
        "[ninja] no ninja/n2 on PATH, installing n2 (rev {}) via cargo",
        N2_REV
    );
    let status = std::process::Command::new("cargo")
        .args(["install", "--locked", "--git", N2_GIT_URL, "--rev", N2_REV])
        .status();
    match status {
        Ok(st) if st.success() => {}
        Ok(st) => {
            // 已安装/网络失败等都可能非零——以复检结果为准，这里只留日志
            warn!("[ninja] cargo install n2 exited with {}", st);
        }
        Err(e) => {
            warn!("[ninja] failed to run cargo install: {}", e);
            return None;
        }
    }

    // 6. 复检：PATH 上的 n2，或显式查 ~/.cargo/bin（装完但 bin 目录
    //    不在 PATH 的环境仍探测不到 → 走 Err 指引文案）
    if let Some(found) = find_on_path("n2") {
        return Some(found);
    }
    let bin = cargo_bin_dir()?;
    find_in([bin], "n2")
}

/// 非空且含空格 → `"s"` 包裹；裸名/无空格/空串原样返回。
/// ninja/n2 均把 command 行交给 shell，`"cl.exe" /c ...` 与裸名语义一致。
pub fn quote_if_spaced(s: &str) -> String {
    if !s.is_empty() && s.contains(' ') {
        format!("\"{}\"", s)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 单测 1：find_in 注入 fixture 路径列表——ninja 目录优先命中；
    // 仅含 n2 的列表命中 n2；空目录全 None
    #[test]
    fn find_in_prefers_ninja_and_falls_back_to_n2() {
        let tmp = tempfile::tempdir().unwrap();
        let ninja_dir = tmp.path().join("a");
        let n2_dir = tmp.path().join("b");
        std::fs::create_dir_all(&ninja_dir).unwrap();
        std::fs::create_dir_all(&n2_dir).unwrap();
        std::fs::write(ninja_dir.join(exe_names("ninja")[0].as_str()), b"").unwrap();
        std::fs::write(n2_dir.join(exe_names("n2")[0].as_str()), b"").unwrap();

        // ninja 命中优先（即使 n2 目录排在前面）
        let hit = find_in([n2_dir.clone(), ninja_dir.clone()], "ninja").unwrap();
        assert!(hit.starts_with(&ninja_dir));

        // 仅含 n2 的目录列表命中 n2
        let hit = find_in([n2_dir.clone()], "n2").unwrap();
        assert!(hit.starts_with(&n2_dir));

        // 全空 → None
        assert!(find_in([tmp.path().join("nowhere")], "ninja").is_none());
    }

    // 单测 2：exe_names 的 windows 形态含 .exe 变体
    #[test]
    fn exe_names_windows_variants() {
        let names = exe_names("ninja");
        if cfg!(windows) {
            assert_eq!(names[0], "ninja.exe");
            assert!(names.contains(&"ninja".to_string()));
        } else {
            assert_eq!(names, vec!["ninja".to_string()]);
        }
    }

    // 单测 3：quote_if_spaced——裸名原样、含空格加引号、空串原样
    #[test]
    fn quote_if_spaced_wraps_only_spaced() {
        assert_eq!(quote_if_spaced("cl.exe"), "cl.exe");
        assert_eq!(
            quote_if_spaced("C:/Program Files/x/link.exe"),
            "\"C:/Program Files/x/link.exe\""
        );
        assert_eq!(quote_if_spaced(""), "");
    }

    // 手动探针（--ignored 显式运行，CI 不跑）：按当前 PATH 解析宿主并打印，
    // 供集成形态 A/B/验收2/验收3 的 resolver 级取证（PATH 无 ninja/n2 且
    // cargo 在时会触发真实安装臂，只在可控环境运行）。
    #[test]
    #[ignore = "manual probe: resolves per current PATH; may invoke cargo install"]
    fn probe_resolve_logs_host() {
        match resolve_ninja_host() {
            Ok(host) => println!("resolved ninja host: {}", host.program),
            Err(e) => println!("resolve error: {}", e),
        }
    }
}
