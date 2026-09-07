use auto_val::AutoPath;
use crate::AutoResult;
use log::*;
use std::env;

use super::config::{CompilerConfig, CompilerKind, CompilerLocation, ExecutableType};

/// 编译器路径解析器
pub struct CompilerResolver;

impl CompilerResolver {
    /// 解析可执行文件路径
    ///
    /// 根据 CompilerConfig 的 location 配置查找可执行文件
    pub fn resolve_executable(
        config: &CompilerConfig,
        exec_type: ExecutableType,
    ) -> AutoPath {
        // MSVC Linker/Archiver 同目录优先（Plan 580 缺陷#3）：编译器已定位到
        // 具体目录时先查同目录 link.exe/lib.exe——VS 工具链布局
        // （VC\Tools\MSVC\<ver>\bin\Hostx64\<arch>）下四件套同目录，避免
        // PATH 上 Git coreutils 的 link.exe 抢先；未命中再走原探测逻辑
        if config.kind == CompilerKind::MSVC {
            let tool_name = match exec_type {
                ExecutableType::Linker => Some("link.exe"),
                ExecutableType::Archiver => Some("lib.exe"),
                _ => None,
            };
            if let Some(tool_name) = tool_name {
                let compiler = Self::resolve_executable(config, ExecutableType::Compiler);
                if let Some(tool) = Self::msvc_sibling_tool(&compiler, tool_name) {
                    return tool;
                }
            }
        }
        match &config.location {
            CompilerLocation::Env => {
                // 从 PATH 环境变量查找
                let default = Self::default_executable(config.kind, exec_type);
                let exe_name = config.get_executable(exec_type)
                    .unwrap_or(&default);
                Self::find_in_path(exe_name)
            }
            CompilerLocation::Dir(dir) => {
                // 从指定目录查找
                let default = Self::default_executable(config.kind, exec_type);
                let exe_name = config.get_executable(exec_type)
                    .unwrap_or(&default);
                dir.join(exe_name.to_astr().as_str())
            }
            CompilerLocation::Executable(e_type, path) => {
                if *e_type == exec_type {
                    path.clone()
                } else {
                    Self::resolve_executable(config, exec_type)
                }
            }
        }
    }

    /// 在编译器所在目录查 MSVC 配套工具（VS 布局下 link.exe/lib.exe 与
    /// cl.exe 同目录）。编译器是裸名（未定位到具体目录）时返回 None。
    fn msvc_sibling_tool(compiler_path: &AutoPath, tool_name: &str) -> Option<AutoPath> {
        let s = compiler_path.to_astr();
        let s = s.as_str();
        if !s.contains('/') && !s.contains('\\') {
            return None;
        }
        let candidate = compiler_path.parent().join(tool_name);
        if candidate.exists() {
            Some(candidate)
        } else {
            None
        }
    }

    /// 从 PATH 环境变量查找可执行文件
    fn find_in_path(exe_name: &AutoPath) -> AutoPath {
        if let Ok(path_var) = env::var("PATH") {
            for path in env::split_paths(&path_var) {
                // 转换 path 为字符串，处理非 UTF-8 路径
                if let Some(path_str) = path.to_str() {
                    let exe_path = AutoPath::new(path_str).join(exe_name.to_astr().as_str());
                    if exe_path.exists() {
                        return exe_path;
                    }
                }
            }
        }

        // 如果找不到，返回可执行文件名本身（假设在 PATH 中）
        exe_name.clone()
    }

    /// 获取默认可执行文件名称
    fn default_executable(kind: super::config::CompilerKind, exec_type: ExecutableType) -> AutoPath {
        use crate::builder::ninja::config::CompilerKind;

        match (kind, exec_type) {
            // MSVC
            (CompilerKind::MSVC, ExecutableType::Compiler) => AutoPath::new("cl.exe"),
            (CompilerKind::MSVC, ExecutableType::Linker) => AutoPath::new("link.exe"),
            (CompilerKind::MSVC, ExecutableType::Archiver) => AutoPath::new("lib.exe"),
            (CompilerKind::MSVC, ExecutableType::Assembler) => AutoPath::new("ml64.exe"),

            // GCC
            (CompilerKind::GCC, ExecutableType::Compiler) => AutoPath::new("gcc.exe"),
            (CompilerKind::GCC, ExecutableType::Linker) => AutoPath::new("gcc.exe"),
            (CompilerKind::GCC, ExecutableType::Archiver) => AutoPath::new("ar.exe"),
            (CompilerKind::GCC, ExecutableType::Assembler) => AutoPath::new("as.exe"),

            // Clang
            (CompilerKind::Clang, ExecutableType::Compiler) => AutoPath::new("clang.exe"),
            (CompilerKind::Clang, ExecutableType::Linker) => AutoPath::new("clang.exe"),
            (CompilerKind::Clang, ExecutableType::Archiver) => AutoPath::new("ar.exe"),
            (CompilerKind::Clang, ExecutableType::Assembler) => AutoPath::new("clang.exe"),

            // IAR
            (CompilerKind::IAR, ExecutableType::Compiler) => AutoPath::new("iccarm.exe"),
            (CompilerKind::IAR, ExecutableType::Linker) => AutoPath::new("ilinkarm.exe"),
            (CompilerKind::IAR, ExecutableType::Archiver) => AutoPath::new("iarchive.exe"),
            (CompilerKind::IAR, ExecutableType::Assembler) => AutoPath::new("iasmarm.exe"),

            // GHS
            (CompilerKind::GHS, ExecutableType::Compiler) => AutoPath::new("ccarm.exe"),
            (CompilerKind::GHS, ExecutableType::Linker) => AutoPath::new("ccarm.exe"),
            (CompilerKind::GHS, ExecutableType::Archiver) => AutoPath::new("cxarm.exe"),
            (CompilerKind::GHS, ExecutableType::Assembler) => AutoPath::new("asarm.exe"),

            // Targeting
            (CompilerKind::Targeting, ExecutableType::Compiler) => AutoPath::new("tcpp.exe"),
            (CompilerKind::Targeting, ExecutableType::Linker) => AutoPath::new("tlink.exe"),
            (CompilerKind::Targeting, ExecutableType::Archiver) => AutoPath::new("tlib.exe"),
            (CompilerKind::Targeting, ExecutableType::Assembler) => AutoPath::new("tas.exe"),

            // Hightec (使用 GCC 兼容工具链)
            (CompilerKind::Hightec, ExecutableType::Compiler) => AutoPath::new("gcc.exe"),
            (CompilerKind::Hightec, ExecutableType::Linker) => AutoPath::new("gcc.exe"),
            (CompilerKind::Hightec, ExecutableType::Archiver) => AutoPath::new("ar.exe"),
            (CompilerKind::Hightec, ExecutableType::Assembler) => AutoPath::new("as.exe"),

            // TICLang
            (CompilerKind::TICLang, ExecutableType::Compiler) => AutoPath::new("tiarmclang.exe"),
            (CompilerKind::TICLang, ExecutableType::Linker) => AutoPath::new("tiarmclang.exe"),
            (CompilerKind::TICLang, ExecutableType::Archiver) => AutoPath::new("tiarmar.exe"),
            (CompilerKind::TICLang, ExecutableType::Assembler) => AutoPath::new("tiarmclang.exe"),
        }
    }

    /// 验证编译器是否可用
    ///
    /// 尝试运行编译器的 --version 或类似命令来验证
    pub fn validate_compiler(kind: super::config::CompilerKind) -> AutoResult<bool> {
        use crate::builder::ninja::config::CompilerKind;

        let (compiler, version_flag) = match kind {
            CompilerKind::MSVC => ("cl", "/?"),
            CompilerKind::GCC => ("gcc", "--version"),
            CompilerKind::Clang => ("clang", "--version"),
            CompilerKind::GHS => ("ccarm", "--version"),
            CompilerKind::IAR => ("iccarm", "--version"),
            CompilerKind::Targeting => ("tcpp", "--version"),
            CompilerKind::Hightec => ("gcc", "--version"),
            CompilerKind::TICLang => ("tiarmclang", "--version"),
        };

        let result = std::process::Command::new(compiler)
            .arg(version_flag)
            .output();

        match result {
            Ok(output) => Ok(output.status.success()),
            Err(e) => {
                info!("Failed to run compiler {}: {}", compiler, e);
                Ok(false)
            }
        }
    }

    /// 获取编译器版本信息
    pub fn get_compiler_version(kind: super::config::CompilerKind) -> AutoResult<Option<String>> {
        use crate::builder::ninja::config::CompilerKind;

        let (compiler, version_flag) = match kind {
            CompilerKind::MSVC => ("cl", "/?"),
            CompilerKind::GCC => ("gcc", "--version"),
            CompilerKind::Clang => ("clang", "--version"),
            CompilerKind::GHS => ("ccarm", "--version"),
            CompilerKind::IAR => ("iccarm", "--version"),
            CompilerKind::Targeting => ("tcpp", "--version"),
            CompilerKind::Hightec => ("gcc", "--version"),
            CompilerKind::TICLang => ("tiarmclang", "--version"),
        };

        let result = std::process::Command::new(compiler)
            .arg(version_flag)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    let version_str = String::from_utf8_lossy(&output.stdout).to_string();
                    // 提取第一行作为版本信息
                    let first_line = version_str.lines().next().unwrap_or("");
                    Ok(Some(first_line.to_string()))
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    /// 查找系统中的可用编译器
    ///
    /// 返回所有找到的编译器类型
    pub fn find_available_compilers() -> Vec<super::config::CompilerKind> {
        use crate::builder::ninja::config::CompilerKind;

        let all_kinds = vec![
            CompilerKind::MSVC,
            CompilerKind::GCC,
            CompilerKind::Clang,
            CompilerKind::GHS,
            CompilerKind::IAR,
            CompilerKind::Targeting,
            CompilerKind::Hightec,
            CompilerKind::TICLang,
        ];

        all_kinds
            .into_iter()
            .filter(|kind| {
                Self::validate_compiler(*kind).unwrap_or(false)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn test_resolve_executable_env() {
        let config = CompilerConfig::gcc_default();
        let cc_path = CompilerResolver::resolve_executable(
            &config,
            ExecutableType::Compiler
        );
        // 路径应该包含 gcc 或 gcc.exe
        let path_astr = cc_path.to_astr();
        let path_str = path_astr.as_str();
        assert!(path_str.contains("gcc"));
    }

    #[test]
    fn test_default_executable_names() {
        use crate::builder::ninja::config::CompilerKind;

        // Test GCC
        let gcc_path = CompilerResolver::default_executable(CompilerKind::GCC, ExecutableType::Compiler);
        assert_eq!(gcc_path.to_astr().as_str(), "gcc.exe");
        let ar_path = CompilerResolver::default_executable(CompilerKind::GCC, ExecutableType::Archiver);
        assert_eq!(ar_path.to_astr().as_str(), "ar.exe");

        // Test MSVC
        let cl_path = CompilerResolver::default_executable(CompilerKind::MSVC, ExecutableType::Compiler);
        assert_eq!(cl_path.to_astr().as_str(), "cl.exe");
        let link_path = CompilerResolver::default_executable(CompilerKind::MSVC, ExecutableType::Linker);
        assert_eq!(link_path.to_astr().as_str(), "link.exe");
    }

    #[test]
    fn test_validate_compiler() {
        use crate::builder::ninja::config::CompilerKind;

        // GCC 通常在大多数系统上可用
        let gcc_available = CompilerResolver::validate_compiler(CompilerKind::GCC)
            .unwrap_or(false);
        // 测试应该能够运行，不管 GCC 是否实际安装
        info!("GCC available: {}", gcc_available);
    }

    #[test]
    fn test_find_available_compilers() {
        let available = CompilerResolver::find_available_compilers();
        info!("Available compilers: {:?}", available);
        // 至少应该有一个编译器在大多数系统上可用
        // 但在某些 CI 环境中可能没有，所以只记录结果
    }

    // 单测 4（Plan 580 T5）：MSVC Linker/Archiver 同目录优先——Env 位置下把
    // 编译器钉到 tempdir 的绝对 cl.exe（含 link.exe/lib.exe 假文件，探测只查
    // 存在不执行），断言 Linker/Archiver 命中同目录而非 PATH；同目录缺失
    // link.exe 时回退 PATH 探测（结果脱离该目录）
    #[test]
    fn msvc_linker_archiver_prefer_compiler_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("msvc");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cl.exe"), b"").unwrap();
        std::fs::write(dir.join("link.exe"), b"").unwrap();
        std::fs::write(dir.join("lib.exe"), b"").unwrap();
        let dir_norm = dir.to_str().unwrap().replace('\\', "/");
        let cl_abs = format!("{}/cl.exe", dir_norm);

        let mut config = CompilerConfig::msvc_default();
        config
            .executables
            .insert(ExecutableType::Compiler, AutoPath::new(&cl_abs));

        let link = CompilerResolver::resolve_executable(&config, ExecutableType::Linker);
        let link_norm = link.to_astr().as_str().replace('\\', "/");
        assert!(link_norm.starts_with(&dir_norm), "linker should stay in compiler dir, got {}", link_norm);
        assert!(link_norm.ends_with("link.exe"));

        let ar = CompilerResolver::resolve_executable(&config, ExecutableType::Archiver);
        let ar_norm = ar.to_astr().as_str().replace('\\', "/");
        assert!(ar_norm.starts_with(&dir_norm), "archiver should stay in compiler dir, got {}", ar_norm);
        assert!(ar_norm.ends_with("lib.exe"));

        // 同目录无 link.exe → 回退 PATH 探测，结果脱离该目录
        std::fs::remove_file(dir.join("link.exe")).unwrap();
        let link2 = CompilerResolver::resolve_executable(&config, ExecutableType::Linker);
        let l2_norm = link2.to_astr().as_str().replace('\\', "/");
        assert!(!l2_norm.starts_with(&dir_norm), "linker should fall back to PATH, got {}", l2_norm);
    }

    // 单测 4b（Plan 580 T5）：同目录查工具的裸名守门——编译器是裸名
    // （未定位到具体目录）时不做同目录探测
    #[test]
    fn msvc_sibling_tool_requires_located_compiler() {
        assert!(CompilerResolver::msvc_sibling_tool(&AutoPath::new("cl.exe"), "link.exe").is_none());

        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("link.exe"), b"").unwrap();
        let cl = tmp.path().join("cl.exe");
        let hit = CompilerResolver::msvc_sibling_tool(&AutoPath::new(cl.to_str().unwrap()), "link.exe")
            .expect("sibling link.exe should hit");
        assert!(hit.to_astr().as_str().replace('\\', "/").ends_with("link.exe"));
    }
}
