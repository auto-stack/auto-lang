//! 源文件选择规则（PLAN-809 T-05，R-04）。
//!
//! 选择规则单源：skips 精确名+glob 过滤、FileFilter 类型准入、头文件
//! 处理、标准子目录表（src/include/inc）、`.at` 收集排除表——全部纯规则；
//! FS 枚举/探测经 `ScanEntry` 输入与谓词注入留在 auto-man adapter
//! （`DirEffects`）。

use crate::file_types::{FileFilter, FileType};
use crate::target::TargetKind;
use auto_val::AutoStr;
use std::collections::HashSet;

/// 一个文件枚举项（adapter 由 read_dir 产出；core 不触碰 FS）。
#[derive(Debug, Clone, PartialEq)]
pub struct ScanEntry {
    /// 文件名（不含路径）。
    pub name: String,
    /// 统一路径形态的完整路径。
    pub path: AutoStr,
}

/// 平铺目录条目选择（原 auto-man/dir.rs `scan_dir_flat` 的规则半）：
/// skips 精确名与 glob 过滤 → FileFilter 准入 → 头文件标记/按 use_header
/// 入 srcs。返回 (选中文件路径集, 是否发现头文件)。
pub fn select_flat_entries(
    entries: Vec<ScanEntry>,
    kind: &TargetKind,
    skips: &HashSet<AutoStr>,
    use_header: bool,
) -> (HashSet<AutoStr>, bool) {
    let mut files_found = HashSet::new();
    let mut headers_found = false;
    let filter = FileFilter::for_target(kind);
    let skip_globs: Vec<&AutoStr> = skips.iter().filter(|s| s.contains("*")).collect();

    for entry in entries {
        let file_name = AutoStr::from(entry.name.as_str());
        if skips.contains(&file_name) {
            continue;
        }
        let mut is_skip = false;
        for skip in &skip_globs {
            if glob_match::glob_match(skip.as_str(), entry.name.as_str()) {
                is_skip = true;
                break;
            }
        }
        if is_skip {
            continue;
        }

        let ext = entry.name.rsplit_once('.').map(|(_, e)| e);
        if !filter.should_include(&entry.name, ext) {
            continue;
        }

        if let Some(file_type) = ext.and_then(FileType::from_extension) {
            if file_type.is_header() {
                headers_found = true;
                if use_header {
                    files_found.insert(entry.path);
                }
            } else if file_type.is_source() {
                files_found.insert(entry.path);
            }
        }
    }
    (files_found, headers_found)
}

/// 标准源码子目录表（原 scanner.rs `scan_standard_subdirs` 的常量半）。
pub const STD_SOURCE_SUBDIRS: [&str; 3] = ["src", "include", "inc"];

/// 标准子目录聚合规则：逐子目录扫描结果合并，头文件存在的子目录入
/// includes（原 `scan_standard_subdirs` 的规则半；is_dir 探测与
/// scan_flat 枚举由 adapter 注入）。
pub fn scan_std_subdirs_aggregate(
    results: Vec<(AutoStr, HashSet<AutoStr>, bool)>,
) -> (HashSet<AutoStr>, HashSet<AutoStr>) {
    let mut all_srcs = HashSet::new();
    let mut all_incs = HashSet::new();
    for (subdir, files, has_headers) in results {
        all_srcs.extend(files);
        if has_headers {
            all_incs.insert(subdir);
        }
    }
    (all_srcs, all_incs)
}

/// `.at` 自动源收集的排除表（原 Target::scan_auto_dir 规则半）：
/// pac.at/device.at/os.at 不收。
pub fn is_excluded_auto_config(name: &str) -> bool {
    matches!(name, "pac.at" | "device.at" | "os.at")
}

/// `.at` 递归下潜的目录过滤（原 scan_auto_dir：`.` 前缀与 `rust` 跳过）。
pub fn auto_recurse_into(dir_name: &str) -> bool {
    !(dir_name.starts_with('.') || dir_name == "rust")
}

/// incs 解析规则（原 `check_incs` 半）：绝对路径本身是文件 → 规则外
/// （调用方记录错误并跳过——保持旧行为的 error! 在 adapter 侧）；是目录
/// → 收；否则按 root 相对再试。谓词注入。
pub fn check_incs_with(
    incs: &[AutoStr],
    root: &AutoStr,
    is_file: impl Fn(&str) -> bool,
    is_dir: impl Fn(&str) -> bool,
) -> (HashSet<AutoStr>, Vec<AutoStr>) {
    let mut succ = HashSet::new();
    let mut file_form_violations = Vec::new();
    for inc in incs {
        let unified = normalize_slashes(inc.as_str());
        if is_file(&unified) {
            file_form_violations.push(unified.clone());
            continue;
        }
        if is_dir(&unified) {
            succ.insert(unified);
        } else {
            let rel = join_root(root, inc);
            let rel_u = normalize_slashes(&rel);
            if is_dir(&rel_u) {
                succ.insert(rel_u);
            }
        }
    }
    (succ, file_form_violations)
}

/// 指定 srcs 解析规则（原 `scan_specific_srcs` 半）：root 相对路径存在
/// 即收（统一路径）。谓词注入。
pub fn scan_specific_srcs_with(
    srcs: &[AutoStr],
    root: &AutoStr,
    is_file: impl Fn(&str) -> bool,
) -> HashSet<AutoStr> {
    let mut succ = HashSet::new();
    for src in srcs {
        let joined = join_root(root, src);
        let unified = normalize_slashes(&joined);
        if is_file(&unified) {
            succ.insert(unified);
        }
    }
    succ
}

fn join_root(root: &AutoStr, rel: &AutoStr) -> String {
    format!("{}/{}", root.as_str(), rel.as_str())
}

fn normalize_slashes(p: &str) -> AutoStr {
    p.replace('\\', "/").into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str) -> ScanEntry {
        ScanEntry {
            name: name.to_string(),
            path: format!("dir/{}", name).into(),
        }
    }

    #[test]
    fn flat_selection_rules() {
        let mut skips: HashSet<AutoStr> = HashSet::new();
        skips.insert(AutoStr::from("gen.c"));
        skips.insert(AutoStr::from("skip_me.c"));
        skips.insert(AutoStr::from("*.txt"));
        let (files, headers) = select_flat_entries(
            vec![
                entry("main.c"),
                entry("util.h"),
                entry("gen.c"),
                entry("skip_me.c"),
                entry("pac.at"),
                entry("data.txt"),
                entry("notes.txt"),
            ],
            &TargetKind::App,
            &skips,
            false,
        );
        let names: HashSet<String> = files.iter().map(|f| f.to_string()).collect();
        assert!(names.contains("dir/main.c"));
        assert!(
            !names.contains("dir/util.h"),
            "头文件不入 srcs（use_header=false）"
        );
        assert!(headers, "头文件被发现（incs 语义）");
        assert!(
            !names.contains("dir/gen.c") && !names.contains("dir/skip_me.c"),
            "skips 生效"
        );
        assert!(
            !names.contains("dir/pac.at") && !names.contains("dir/data.txt"),
            "过滤/排除生效"
        );
    }

    #[test]
    fn headers_as_sources_when_requested() {
        let (files, headers) = select_flat_entries(
            vec![entry("a.h"), entry("b.c")],
            &TargetKind::App,
            &HashSet::new(),
            true,
        );
        assert!(headers);
        assert!(files.iter().any(|f| f.as_str().ends_with("a.h")));
    }

    #[test]
    fn device_kind_keeps_gpj_filtered_but_850_in() {
        let (files, _) = select_flat_entries(
            vec![entry("x.850"), entry("p.gpj")],
            &TargetKind::Device,
            &HashSet::new(),
            false,
        );
        assert!(files.iter().any(|f| f.as_str().ends_with("x.850")));
        assert!(
            files.iter().all(|f| !f.as_str().ends_with(".gpj")),
            "GhsProject 排除"
        );
    }

    #[test]
    fn std_subdirs_table_and_aggregate() {
        assert_eq!(STD_SOURCE_SUBDIRS, ["src", "include", "inc"]);
        let mut f1 = HashSet::new();
        f1.insert(AutoStr::from("src/a.c"));
        let mut f2 = HashSet::new();
        f2.insert(AutoStr::from("inc/b.c"));
        let (srcs, incs) =
            scan_std_subdirs_aggregate(vec![("src".into(), f1, true), ("inc".into(), f2, false)]);
        assert_eq!(srcs.len(), 2);
        assert!(incs.contains(&AutoStr::from("src")));
        assert!(!incs.contains(&AutoStr::from("inc")));
    }

    #[test]
    fn auto_exclusion_table() {
        assert!(is_excluded_auto_config("pac.at"));
        assert!(is_excluded_auto_config("device.at"));
        assert!(is_excluded_auto_config("os.at"));
        assert!(!is_excluded_auto_config("main.at"));
        assert!(!auto_recurse_into(".git"));
        assert!(!auto_recurse_into("rust"));
        assert!(auto_recurse_into("src"));
    }

    #[test]
    fn incs_resolution_rules() {
        let (succ, violations) = check_incs_with(
            &["inc_abs".into(), "inc_rel".into(), "a_file".into()],
            &"root".into(),
            |p| p == "a_file",
            |p| p == "inc_abs" || p == "root/inc_rel",
        );
        let set: HashSet<String> = succ.iter().map(|s| s.to_string()).collect();
        assert!(set.contains("inc_abs"));
        assert!(set.contains("root/inc_rel"));
        assert_eq!(violations, vec!["a_file".to_string()]);
    }

    #[test]
    fn specific_srcs_rules() {
        let succ = scan_specific_srcs_with(&["a.c".into(), "missing.c".into()], &"t".into(), |p| {
            p == "t/a.c"
        });
        assert_eq!(succ.len(), 1);
        assert!(succ.contains(&AutoStr::from("t/a.c")));
    }
}
