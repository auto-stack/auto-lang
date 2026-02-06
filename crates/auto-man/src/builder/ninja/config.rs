use auto_val::{AutoPath, AutoStr};
use std::collections::HashMap;
use std::fmt;

/// 编译器类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilerKind {
    MSVC,
    GCC,
    Clang,
    GHS,
    IAR,
    Targeting,
    Hightec,
}

impl CompilerKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "msvc" => Some(CompilerKind::MSVC),
            "gcc" | "gnu" => Some(CompilerKind::GCC),
            "clang" | "llvm" => Some(CompilerKind::Clang),
            "ghs" => Some(CompilerKind::GHS),
            "iar" => Some(CompilerKind::IAR),
            "targeting" => Some(CompilerKind::Targeting),
            "hightec" => Some(CompilerKind::Hightec),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            CompilerKind::MSVC => "msvc",
            CompilerKind::GCC => "gcc",
            CompilerKind::Clang => "clang",
            CompilerKind::GHS => "ghs",
            CompilerKind::IAR => "iar",
            CompilerKind::Targeting => "targeting",
            CompilerKind::Hightec => "hightec",
        }
    }
}

impl fmt::Display for CompilerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// 可执行文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExecutableType {
    Compiler,    // cc
    Archiver,    // ar
    Linker,      // link
    Assembler,   // as
}

impl ExecutableType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExecutableType::Compiler => "cc",
            ExecutableType::Archiver => "ar",
            ExecutableType::Linker => "link",
            ExecutableType::Assembler => "as",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "cc" | "compiler" => Some(ExecutableType::Compiler),
            "ar" | "archiver" => Some(ExecutableType::Archiver),
            "link" | "linker" => Some(ExecutableType::Linker),
            "as" | "assembler" => Some(ExecutableType::Assembler),
            _ => None,
        }
    }
}

/// 标志格式类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FlagFormat {
    Prefix(String),           // "-I{}" -> -I/path
    Postfix(String),          // "{}.lib" -> path.lib
    Both(String, String),     // "/Fo{}", ".obj" -> /Fopath.obj
}

impl FlagFormat {
    /// 应用标志格式到值
    pub fn apply(&self, value: &str) -> String {
        match self {
            FlagFormat::Prefix(template) => {
                template.replace("{}", value)
            }
            FlagFormat::Postfix(template) => {
                template.replace("{}", value)
            }
            FlagFormat::Both(prefix, postfix) => {
                format!("{}{}{}", prefix, value, postfix)
            }
        }
    }
}

/// 编译器标志映射配置
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlagMappings {
    pub include: FlagFormat,
    pub define: FlagFormat,
    pub library: FlagFormat,
    pub library_path: FlagFormat,
    pub output: FlagFormat,
    pub compile_only: FlagFormat,
}

impl FlagMappings {
    /// 为 MSVC 创建默认标志映射
    pub fn msvc_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("/I{}".to_string()),
            define: FlagFormat::Prefix("/D{}".to_string()),
            library: FlagFormat::Postfix("{}.lib".to_string()),
            library_path: FlagFormat::Prefix("/LIBPATH:{}".to_string()),
            output: FlagFormat::Both("/Fo".to_string(), ".obj".to_string()),
            compile_only: FlagFormat::Prefix("/c".to_string()),
        }
    }

    /// 为 GCC 创建默认标志映射
    pub fn gcc_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("-I{}".to_string()),
            define: FlagFormat::Prefix("-D{}".to_string()),
            library: FlagFormat::Prefix("-l{}".to_string()),
            library_path: FlagFormat::Prefix("-L{}".to_string()),
            output: FlagFormat::Prefix("-o{}".to_string()),
            compile_only: FlagFormat::Prefix("-c".to_string()),
        }
    }

    /// 为 Clang 创建默认标志映射
    pub fn clang_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("-I{}".to_string()),
            define: FlagFormat::Prefix("-D{}".to_string()),
            library: FlagFormat::Prefix("-l{}".to_string()),
            library_path: FlagFormat::Prefix("-L{}".to_string()),
            output: FlagFormat::Prefix("-o{}".to_string()),
            compile_only: FlagFormat::Prefix("-c".to_string()),
        }
    }

    /// 为 IAR 创建默认标志映射
    pub fn iar_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("-I{}".to_string()),
            define: FlagFormat::Prefix("-D{}".to_string()),
            library: FlagFormat::Prefix("-l{}".to_string()),
            library_path: FlagFormat::Prefix("-L{}".to_string()),
            output: FlagFormat::Prefix("-o{}".to_string()),
            compile_only: FlagFormat::Prefix("--debug".to_string()),
        }
    }

    /// 为 GHS 创建默认标志映射
    pub fn ghs_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("-I{}".to_string()),
            define: FlagFormat::Prefix("-D{}".to_string()),
            library: FlagFormat::Prefix("-l{}".to_string()),
            library_path: FlagFormat::Prefix("-L{}".to_string()),
            output: FlagFormat::Prefix("-o{}".to_string()),
            compile_only: FlagFormat::Prefix("-c".to_string()),
        }
    }

    /// 为 Targeting 创建默认标志映射
    pub fn targeting_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("-I{}".to_string()),
            define: FlagFormat::Prefix("-D{}".to_string()),
            library: FlagFormat::Prefix("-l{}".to_string()),
            library_path: FlagFormat::Prefix("-L{}".to_string()),
            output: FlagFormat::Prefix("-o{}".to_string()),
            compile_only: FlagFormat::Prefix("-c".to_string()),
        }
    }

    /// 为 Hightec 创建默认标志映射
    pub fn hightec_default() -> Self {
        FlagMappings {
            include: FlagFormat::Prefix("-I{}".to_string()),
            define: FlagFormat::Prefix("-D{}".to_string()),
            library: FlagFormat::Prefix("-l{}".to_string()),
            library_path: FlagFormat::Prefix("-L{}".to_string()),
            output: FlagFormat::Prefix("-o{}".to_string()),
            compile_only: FlagFormat::Prefix("-c".to_string()),
        }
    }
}

/// 编译器位置配置
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompilerLocation {
    Env,                          // 从环境变量 PATH 查找
    Dir(AutoPath),                // 指定安装目录
    Executable(ExecutableType, AutoPath),  // 直接指定可执行文件
}

/// 编译器配置
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilerConfig {
    pub kind: CompilerKind,
    pub name: AutoStr,
    pub executables: HashMap<ExecutableType, AutoPath>,
    pub flags: FlagMappings,
    pub location: CompilerLocation,
    pub default_cflags: Vec<AutoStr>,
}

impl CompilerConfig {
    /// 创建新的编译器配置
    pub fn new(
        kind: CompilerKind,
        name: AutoStr,
        executables: HashMap<ExecutableType, AutoPath>,
        flags: FlagMappings,
        location: CompilerLocation,
        default_cflags: Vec<AutoStr>,
    ) -> Self {
        Self {
            kind,
            name,
            executables,
            flags,
            location,
            default_cflags,
        }
    }

    /// 为 MSVC 创建默认配置
    pub fn msvc_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("cl.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("link.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("lib.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("ml64.exe"));

        Self {
            kind: CompilerKind::MSVC,
            name: "msvc".into(),
            executables,
            flags: FlagMappings::msvc_default(),
            location: CompilerLocation::Env,
            default_cflags: vec!["/O2".into(), "/W3".into(), "/MD".into()],
        }
    }

    /// 为 GCC 创建默认配置
    pub fn gcc_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("gcc.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("gcc.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("ar.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("as.exe"));

        Self {
            kind: CompilerKind::GCC,
            name: "gcc".into(),
            executables,
            flags: FlagMappings::gcc_default(),
            location: CompilerLocation::Env,
            default_cflags: vec!["-O2".into(), "-Wall".into()],
        }
    }

    /// 为 Clang 创建默认配置
    pub fn clang_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("clang.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("clang.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("ar.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("clang.exe"));

        Self {
            kind: CompilerKind::Clang,
            name: "clang".into(),
            executables,
            flags: FlagMappings::clang_default(),
            location: CompilerLocation::Env,
            default_cflags: vec!["-O2".into(), "-Wall".into()],
        }
    }

    /// 为 IAR 创建默认配置
    pub fn iar_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("iccarm.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("ilinkarm.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("iarchive.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("iasmarm.exe"));

        let default_cflags = vec![
            "--no_cse".into(),
            "--no_unroll".into(),
            "--no_inline".into(),
            "--no_code_motion".into(),
            "--no_tbaa".into(),
            "--no_clustering".into(),
            "--no_scheduling".into(),
            "--debug".into(),
            "--endian=little".into(),
            "--cpu=Cortex-M4".into(),
            "-e".into(),
            "--fpu=VFPv4_sp".into(),
        ];

        Self {
            kind: CompilerKind::IAR,
            name: "iar".into(),
            executables,
            flags: FlagMappings::iar_default(),
            location: CompilerLocation::Env,
            default_cflags,
        }
    }

    /// 为 GHS 创建默认配置
    pub fn ghs_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("ccarm.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("ccarm.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("cxarm.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("asarm.exe"));

        Self {
            kind: CompilerKind::GHS,
            name: "ghs".into(),
            executables,
            flags: FlagMappings::ghs_default(),
            location: CompilerLocation::Env,
            default_cflags: vec!["-O".into(), "-g".into()],
        }
    }

    /// 为 Targeting 创建默认配置
    pub fn targeting_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("tcpp.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("tlink.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("tlib.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("tas.exe"));

        Self {
            kind: CompilerKind::Targeting,
            name: "targeting".into(),
            executables,
            flags: FlagMappings::targeting_default(),
            location: CompilerLocation::Env,
            default_cflags: vec!["-O2".into(), "-Wall".into()],
        }
    }

    /// 为 Hightec 创建默认配置
    pub fn hightec_default() -> Self {
        let mut executables = HashMap::new();
        executables.insert(ExecutableType::Compiler, AutoPath::new("gcc.exe"));
        executables.insert(ExecutableType::Linker, AutoPath::new("gcc.exe"));
        executables.insert(ExecutableType::Archiver, AutoPath::new("ar.exe"));
        executables.insert(ExecutableType::Assembler, AutoPath::new("as.exe"));

        Self {
            kind: CompilerKind::Hightec,
            name: "hightec".into(),
            executables,
            flags: FlagMappings::hightec_default(),
            location: CompilerLocation::Env,
            default_cflags: vec!["-O2".into(), "-Wall".into()],
        }
    }

    /// 获取可执行文件名称
    pub fn get_executable(&self, exec_type: ExecutableType) -> Option<&AutoPath> {
        self.executables.get(&exec_type)
    }

    /// 获取对象文件扩展名
    pub fn get_object_extension(&self) -> &'static str {
        match self.kind {
            CompilerKind::MSVC => ".obj",
            _ => ".o",
        }
    }

    /// 获取可执行文件扩展名
    pub fn get_executable_extension(&self) -> &'static str {
        match self.kind {
            CompilerKind::MSVC => ".exe",
            _ => "",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_kind_from_str() {
        assert_eq!(CompilerKind::from_str("msvc"), Some(CompilerKind::MSVC));
        assert_eq!(CompilerKind::from_str("GCC"), Some(CompilerKind::GCC));
        assert_eq!(CompilerKind::from_str("Clang"), Some(CompilerKind::Clang));
        assert_eq!(CompilerKind::from_str("iar"), Some(CompilerKind::IAR));
        assert_eq!(CompilerKind::from_str("ghs"), Some(CompilerKind::GHS));
        assert_eq!(CompilerKind::from_str("targeting"), Some(CompilerKind::Targeting));
        assert_eq!(CompilerKind::from_str("hightec"), Some(CompilerKind::Hightec));
        assert_eq!(CompilerKind::from_str("unknown"), None);
    }

    #[test]
    fn test_executable_type_from_str() {
        assert_eq!(ExecutableType::from_str("cc"), Some(ExecutableType::Compiler));
        assert_eq!(ExecutableType::from_str("ar"), Some(ExecutableType::Archiver));
        assert_eq!(ExecutableType::from_str("link"), Some(ExecutableType::Linker));
        assert_eq!(ExecutableType::from_str("as"), Some(ExecutableType::Assembler));
        assert_eq!(ExecutableType::from_str("unknown"), None);
    }

    #[test]
    fn test_flag_format_apply() {
        let prefix = FlagFormat::Prefix("-I{}".to_string());
        assert_eq!(prefix.apply("include"), "-Iinclude");

        let postfix = FlagFormat::Postfix("{}.lib".to_string());
        assert_eq!(postfix.apply("mylib"), "mylib.lib");

        let both = FlagFormat::Both("/Fo".to_string(), ".obj".to_string());
        assert_eq!(both.apply("output"), "/Fooutput.obj");
    }

    #[test]
    fn test_compiler_config_msvc() {
        let config = CompilerConfig::msvc_default();
        assert_eq!(config.kind, CompilerKind::MSVC);
        assert_eq!(config.get_object_extension(), ".obj");
        assert_eq!(config.get_executable_extension(), ".exe");
        assert_eq!(
            config.get_executable(ExecutableType::Compiler),
            Some(&"cl.exe".into())
        );
    }

    #[test]
    fn test_compiler_config_gcc() {
        let config = CompilerConfig::gcc_default();
        assert_eq!(config.kind, CompilerKind::GCC);
        assert_eq!(config.get_object_extension(), ".o");
        assert_eq!(config.get_executable_extension(), "");
        assert_eq!(
            config.get_executable(ExecutableType::Compiler),
            Some(&"gcc.exe".into())
        );
    }

    #[test]
    fn test_flag_mappings_msvc() {
        let flags = FlagMappings::msvc_default();
        assert_eq!(flags.include.apply("path"), "/Ipath");
        assert_eq!(flags.define.apply("DEBUG"), "/DDEBUG");
        assert_eq!(flags.library.apply("mylib"), "mylib.lib");
        assert_eq!(flags.library_path.apply("path"), "/LIBPATH:path");
    }

    #[test]
    fn test_flag_mappings_gcc() {
        let flags = FlagMappings::gcc_default();
        assert_eq!(flags.include.apply("path"), "-Ipath");
        assert_eq!(flags.define.apply("DEBUG"), "-DDEBUG");
        assert_eq!(flags.library.apply("mylib"), "-lmylib");
        assert_eq!(flags.library_path.apply("path"), "-Lpath");
    }

    #[test]
    fn test_compiler_config_clang() {
        let config = CompilerConfig::clang_default();
        assert_eq!(config.kind, CompilerKind::Clang);
        assert_eq!(config.get_object_extension(), ".o");
        assert_eq!(config.get_executable_extension(), "");
        assert_eq!(
            config.get_executable(ExecutableType::Compiler),
            Some(&"clang.exe".into())
        );
    }

    #[test]
    fn test_compiler_config_iar() {
        let config = CompilerConfig::iar_default();
        assert_eq!(config.kind, CompilerKind::IAR);
        assert_eq!(config.get_object_extension(), ".o");
        assert!(!config.default_cflags.is_empty());
        assert!(config.default_cflags.iter().any(|f| f.as_str() == "--debug"));
    }

    #[test]
    fn test_compiler_config_ghs() {
        let config = CompilerConfig::ghs_default();
        assert_eq!(config.kind, CompilerKind::GHS);
        assert_eq!(config.get_object_extension(), ".o");
        assert_eq!(
            config.get_executable(ExecutableType::Compiler),
            Some(&"ccarm.exe".into())
        );
    }

    #[test]
    fn test_compiler_config_targeting() {
        let config = CompilerConfig::targeting_default();
        assert_eq!(config.kind, CompilerKind::Targeting);
        assert_eq!(config.get_object_extension(), ".o");
        assert_eq!(
            config.get_executable(ExecutableType::Compiler),
            Some(&"tcpp.exe".into())
        );
    }

    #[test]
    fn test_compiler_config_hightec() {
        let config = CompilerConfig::hightec_default();
        assert_eq!(config.kind, CompilerKind::Hightec);
        assert_eq!(config.get_object_extension(), ".o");
        assert_eq!(
            config.get_executable(ExecutableType::Compiler),
            Some(&"gcc.exe".into())
        );
    }

    #[test]
    fn test_flag_mappings_targeting() {
        let flags = FlagMappings::targeting_default();
        assert_eq!(flags.include.apply("path"), "-Ipath");
        assert_eq!(flags.define.apply("DEBUG"), "-DDEBUG");
        assert_eq!(flags.library.apply("mylib"), "-lmylib");
    }

    #[test]
    fn test_flag_mappings_hightec() {
        let flags = FlagMappings::hightec_default();
        assert_eq!(flags.include.apply("path"), "-Ipath");
        assert_eq!(flags.define.apply("DEBUG"), "-DDEBUG");
        assert_eq!(flags.library.apply("mylib"), "-lmylib");
    }
}
