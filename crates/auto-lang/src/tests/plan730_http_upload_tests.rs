//! PLAN-730: HTTP 服务端上传（T-01 平台探针 + 后续单元/e2e 族）。
//!
//! 本文件随任务推进扩充：T-01 = no-replace 原语/同卷判定/rename 语义探针
//! （决策报告 §3 的平台证据）；T-03/T-04 追加 parser/预算/存储矩阵；
//! `http_e2e_plan730` 真 TCP 族挂在 `test-http-e2e` feature 下。

// ============================================================================
// T-01 平台探针：create-only 原子发布原语（Windows/Linux 双面证据）
// ============================================================================

mod plan730_probe {
    use std::path::{Path, PathBuf};

    fn temp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "plan730-probe-{tag}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("probe temp dir");
        dir
    }

    /// P1（Windows 核心）：`std::fs::hard_link` 对已存在目标失败
    /// （ERROR_ALREADY_EXISTS / EEXIST）——std 级**原子 create-only** 原语。
    /// 成功路径：hard_link 后 staging 与 target 同内容，删 staging 后
    /// target 完整保留（发布语义 = link + unlink）。
    #[test]
    fn plan730_probe_hard_link_is_create_only() {
        let dir = temp_dir("hardlink");
        let staging = dir.join("staging.part");
        std::fs::write(&staging, b"plan730 probe payload").unwrap();

        // 已存在目标 → AlreadyExists（不覆盖、不改既有内容）。
        let occupied = dir.join("occupied.bin");
        std::fs::write(&occupied, b"original").unwrap();
        let err = std::fs::hard_link(&staging, &occupied).expect_err("must refuse existing target");
        assert_eq!(
            err.kind(),
            std::io::ErrorKind::AlreadyExists,
            "hard_link over existing target must be AlreadyExists (got {err:?})"
        );
        assert_eq!(std::fs::read(&occupied).unwrap(), b"original");

        // 新目标 → 成功；删 staging 后 target 保留（link 计数语义）。
        let target = dir.join("published.bin");
        std::fs::hard_link(&staging, &target).expect("fresh hard_link succeeds");
        assert_eq!(std::fs::read(&target).unwrap(), b"plan730 probe payload");
        std::fs::remove_file(&staging).unwrap();
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"plan730 probe payload",
            "target survives staging removal"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// P2（反证）：`std::fs::rename` 在 Windows = MoveFileExW
    /// (REPLACE_EXISTING)、在 Linux = rename(2)——**两者都覆盖已存在
    /// 目标**，不满足 create-only 合同（决策报告 §3 的拒绝理由）。
    #[test]
    fn plan730_probe_rename_replaces_existing() {
        let dir = temp_dir("rename");
        let staging = dir.join("staging.part");
        std::fs::write(&staging, b"new content").unwrap();
        let target = dir.join("target.bin");
        std::fs::write(&target, b"original").unwrap();
        std::fs::rename(&staging, &target).expect("std rename overwrites by platform semantics");
        assert_eq!(
            std::fs::read(&target).unwrap(),
            b"new content",
            "rename replaced the target — NOT create-only"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// P3：同卷前置判定证据——Windows 走 canonicalize 前缀
    /// （`\\?\C:\` / `\\?\Volume{GUID}\`），unix 走 st_dev。同卷 staging
    /// 才能用 hard_link 原子发布（跨卷 → 配置错误，须在读取 body 前拒绝）。
    #[test]
    fn plan730_probe_same_volume_detection() {
        let root = temp_dir("volume");
        let staging = root.join("private-staging");
        std::fs::create_dir_all(&staging).unwrap();
        let same_volume = same_volume_probe(&root, &staging);
        assert!(same_volume, "sibling dirs on the same volume");

        // 嵌套关系（staging 在 root 之内）必须被词法检查拒绝（私有性合同）。
        let nested = root.join("public/nested");
        std::fs::create_dir_all(&nested).unwrap();
        assert!(is_lexically_inside(&root, &nested), "lexical nesting detected");
        let _ = std::fs::remove_dir_all(&root);
    }

    fn volume_key(p: &Path) -> String {
        let canon = std::fs::canonicalize(p).expect("canonicalize probe dir");
        let s = canon.to_string_lossy().to_string();
        #[cfg(windows)]
        {
            // `\\?\C:\x` / `\\?\Volume{GUID}\x` → 卷键 = 第三个反斜杠前的段。
            if let Some(rest) = s.strip_prefix(r"\\?\") {
                if let Some(idx) = rest.find('\\') {
                    return rest[..idx].to_ascii_lowercase();
                }
            }
            s.to_ascii_lowercase()
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let dev = std::fs::metadata(&canon).map(|m| m.dev().to_string());
            dev.unwrap_or(s)
        }
    }

    fn same_volume_probe(a: &Path, b: &Path) -> bool {
        volume_key(a) == volume_key(b)
    }

    fn is_lexically_inside(parent: &Path, child: &Path) -> bool {
        let p = parent.canonicalize().unwrap();
        let c = child.canonicalize().unwrap();
        c.starts_with(&p)
    }
}
