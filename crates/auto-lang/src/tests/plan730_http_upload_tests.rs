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

// ============================================================================
// T-02：路由能力分类 + 加载期合同诊断 + VM 纯面 native 探针
// ============================================================================

mod plan730_t02 {
    /// 路由分类门：方法+参数类型双条件——声明 UploadRequest 参数的 fn 才
    /// 是上传路由；同名用户函数/普通参数不误判（AC-01 反例）。
    #[test]
    fn plan730_route_classification_by_param_type() {
        use crate::vm::ffi::http_server::{
            record_api_param_sigs, route_declares_upload, ApiParamSig,
        };
        let sig = |name: &str, ty: &str| ApiParamSig {
            name: name.to_string(),
            ty: ty.to_string(),
        };
        record_api_param_sigs(
            "upload_doc",
            vec![sig("req", "UploadRequest"), sig("meta", "str")],
        );
        record_api_param_sigs("plain_json", vec![sig("data", "str")]);
        record_api_param_sigs("req_named_str", vec![sig("req", "str")]);
        assert!(route_declares_upload("upload_doc"));
        assert!(!route_declares_upload("plain_json"));
        // 名为 req 的普通 str 参数不是注入参数（类型识别优先于命名约定）。
        assert!(!route_declares_upload("req_named_str"));
        assert!(!route_declares_upload("no_such_fn"));
    }

    /// 加载期合同诊断：GET 上传方法、body 参数混用、缺 UploadRequest 的
    /// UploadReceipt 返回——编译错误指名（AC-01/AC-07 的加载面）。
    #[test]
    fn plan730_codegen_upload_contract_diagnostics() {
        let get_upload = r#"
#[api(method = "GET", path = "/up")]
fn up(req UploadRequest) UploadReceipt {
    return http.upload_error(500, "x")
}
"#;
        let err = crate::run_with_capture(get_upload).expect_err("GET upload must be rejected");
        let msg = format!("{err:?}");
        assert!(msg.contains("POST/PUT"), "method diagnostic: {msg}");

        let mixed_body = r#"
#[api(method = "POST", path = "/up")]
fn up(req UploadRequest, note str) UploadReceipt {
    return http.upload_error(500, "x")
}
"#;
        let err = crate::run_with_capture(mixed_body).expect_err("body param must be rejected");
        let msg = format!("{err:?}");
        assert!(msg.contains("upload endpoints allow only path params"), "{msg}");

        let no_param = r#"
#[api(method = "POST", path = "/up")]
fn up() UploadReceipt {
    return http.upload_error(500, "x")
}
"#;
        let err = crate::run_with_capture(no_param).expect_err("receipt without request rejected");
        let msg = format!("{err:?}");
        assert!(msg.contains("no UploadRequest param"), "{msg}");
    }

    /// 合法形态的编译/运行证明经 T-05/T-08 e2e（start_server fixture——
    /// run_with_capture 对含 #[api] 路由的程序会自动起服并阻塞，不适用）。

    /// VM 纯面 native：upload_error 构造登记收据（零 I/O）；upload_metadata
    /// 未知句柄返回可观察冲突形态（不挂死）。
    #[test]
    fn plan730_vm_upload_error_and_metadata_probe() {
        let code = r#"
let receipt = http.upload_error(401, "missing token")
let unknown = http.upload_metadata(99999)
print("done")
"#;
        let (_, out) = crate::run_with_capture(code).expect("run");
        assert!(out.contains("done"), "{out}");
        let (reqs, sessions, receipts) = crate::vm::ffi::http_upload::vm_upload_counts();
        assert_eq!(reqs, 0, "no injected capability in plain script");
        assert!(receipts >= 1, "upload_error receipt registered");
        assert_eq!(sessions, 0);
        // 编组门反例：普通 fn（非 #[api]）不在上传返回表。
        assert!(!crate::vm::ffi::http_server::fn_is_api_upload_return("main"));
    }
}
