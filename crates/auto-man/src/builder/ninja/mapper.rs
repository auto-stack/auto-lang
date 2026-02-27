use auto_val::AutoStr;
use super::config::{CompilerConfig, FlagFormat};

/// 标志映射器 - 将通用标志转换为编译器特定格式
pub struct FlagMapper;

impl FlagMapper {
    /// 映射 include 路径标志
    ///
    /// # 示例
    /// ```
    /// // MSVC: "/Iinclude"
    /// // GCC: "-Iinclude"
    /// ```
    pub fn map_include(config: &CompilerConfig, path: &AutoStr) -> String {
        config.flags.include.apply(path.as_str())
    }

    /// 映射 define 标志
    ///
    /// # 示例
    /// ```
    /// // MSVC: "/DDEBUG"
    /// // GCC: "-DDEBUG"
    /// ```
    pub fn map_define(config: &CompilerConfig, define: &AutoStr) -> String {
        config.flags.define.apply(define.as_str())
    }

    /// 映射 library 标志
    ///
    /// # 示例
    /// ```
    /// // MSVC: "mylib.lib"
    /// // GCC: "-lmylib"
    /// ```
    pub fn map_library(config: &CompilerConfig, lib: &AutoStr) -> String {
        config.flags.library.apply(lib.as_str())
    }

    /// 映射 library path 标志
    ///
    /// # 示例
    /// ```
    /// // MSVC: "/LIBPATH:path"
    /// // GCC: "-Lpath"
    /// ```
    pub fn map_library_path(config: &CompilerConfig, path: &AutoStr) -> String {
        config.flags.library_path.apply(path.as_str())
    }

    /// 映射输出文件标志
    ///
    /// # 示例
    /// ```
    /// // GCC: "-ooutput"
    /// // MSVC with both: "/Fooutput.obj"
    /// ```
    pub fn map_output(config: &CompilerConfig, output: &AutoStr, obj_ext: &str) -> String {
        // 移除已有的对象文件扩展名（如果存在）
        let base_output = output.as_str().trim_end_matches(obj_ext);

        match &config.flags.output {
            FlagFormat::Prefix(template) => {
                template.replace("{}", base_output)
            }
            FlagFormat::Postfix(template) => {
                template.replace("{}", output.as_str())
            }
            FlagFormat::Both(prefix, postfix) => {
                format!("{}{}{}", prefix, base_output, postfix)
            }
        }
    }

    /// 映射 compile_only 标志
    ///
    /// # 示例
    /// ```
    /// // MSVC: "/c"
    /// // GCC: "-c"
    /// // IAR: "--debug"
    /// ```
    pub fn map_compile_only(config: &CompilerConfig) -> String {
        match &config.flags.compile_only {
            FlagFormat::Prefix(template) => template.clone(),
            FlagFormat::Postfix(template) => template.clone(),
            FlagFormat::Both(prefix, postfix) => format!("{}{}", prefix, postfix),
        }
    }

    /// 批量构建 include 标志
    pub fn build_includes(config: &CompilerConfig, includes: &[AutoStr]) -> Vec<String> {
        includes.iter()
            .map(|inc| Self::map_include(config, inc))
            .collect()
    }

    /// 批量构建 define 标志
    pub fn build_defines(config: &CompilerConfig, defines: &[AutoStr]) -> Vec<String> {
        defines.iter()
            .map(|def| Self::map_define(config, def))
            .collect()
    }

    /// 批量构建 library 标志
    pub fn build_libraries(config: &CompilerConfig, libs: &[AutoStr]) -> Vec<String> {
        libs.iter()
            .map(|lib| Self::map_library(config, lib))
            .collect()
    }

    /// 批量构建 library path 标志
    pub fn build_library_paths(config: &CompilerConfig, paths: &[AutoStr]) -> Vec<String> {
        paths.iter()
            .map(|path| Self::map_library_path(config, path))
            .collect()
    }

    /// 格式化标志列表为字符串（用空格连接）
    pub fn format_flags(flags: &[String]) -> String {
        flags.join(" ")
    }

    /// 格式化 include 列表为字符串
    pub fn format_includes(config: &CompilerConfig, includes: &[AutoStr]) -> String {
        Self::format_flags(&Self::build_includes(config, includes))
    }

    /// 格式化 define 列表为字符串
    pub fn format_defines(config: &CompilerConfig, defines: &[AutoStr]) -> String {
        Self::format_flags(&Self::build_defines(config, defines))
    }

    /// 格式化 library 列表为字符串
    pub fn format_libraries(config: &CompilerConfig, libs: &[AutoStr]) -> String {
        Self::format_flags(&Self::build_libraries(config, libs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    fn create_test_msvc_config() -> CompilerConfig {
        CompilerConfig::msvc_default()
    }

    fn create_test_gcc_config() -> CompilerConfig {
        CompilerConfig::gcc_default()
    }

    #[test]
    fn test_map_include_msvc() {
        let config = create_test_msvc_config();
        assert_eq!(FlagMapper::map_include(&config, &"include".into()), "/Iinclude");
        assert_eq!(FlagMapper::map_include(&config, &"/usr/include".into()), "/I/usr/include");
    }

    #[test]
    fn test_map_include_gcc() {
        let config = create_test_gcc_config();
        assert_eq!(FlagMapper::map_include(&config, &"include".into()), "-Iinclude");
        assert_eq!(FlagMapper::map_include(&config, &"/usr/include".into()), "-I/usr/include");
    }

    #[test]
    fn test_map_define_msvc() {
        let config = create_test_msvc_config();
        assert_eq!(FlagMapper::map_define(&config, &"DEBUG".into()), "/DDEBUG");
        assert_eq!(FlagMapper::map_define(&config, &"VERSION=1".into()), "/DVERSION=1");
    }

    #[test]
    fn test_map_define_gcc() {
        let config = create_test_gcc_config();
        assert_eq!(FlagMapper::map_define(&config, &"DEBUG".into()), "-DDEBUG");
        assert_eq!(FlagMapper::map_define(&config, &"VERSION=1".into()), "-DVERSION=1");
    }

    #[test]
    fn test_map_library_msvc() {
        let config = create_test_msvc_config();
        assert_eq!(FlagMapper::map_library(&config, &"mylib".into()), "mylib.lib");
    }

    #[test]
    fn test_map_library_gcc() {
        let config = create_test_gcc_config();
        assert_eq!(FlagMapper::map_library(&config, &"mylib".into()), "-lmylib");
    }

    #[test]
    fn test_map_library_path_msvc() {
        let config = create_test_msvc_config();
        assert_eq!(
            FlagMapper::map_library_path(&config, &"C:/libs".into()),
            "/LIBPATH:C:/libs"
        );
    }

    #[test]
    fn test_map_library_path_gcc() {
        let config = create_test_gcc_config();
        assert_eq!(
            FlagMapper::map_library_path(&config, &"/usr/lib".into()),
            "-L/usr/lib"
        );
    }

    #[test]
    fn test_map_output_msvc() {
        let config = create_test_msvc_config();
        // MSVC 使用 Both 格式：/Fo + base + .obj
        assert_eq!(
            FlagMapper::map_output(&config, &"main.obj".into(), ".obj"),
            "/Fomain.obj"
        );
        assert_eq!(
            FlagMapper::map_output(&config, &"output".into(), ".obj"),
            "/Fooutput.obj"
        );
    }

    #[test]
    fn test_map_output_gcc() {
        let config = create_test_gcc_config();
        // GCC 使用 Prefix 格式：-o + full output
        assert_eq!(
            FlagMapper::map_output(&config, &"main.o".into(), ".o"),
            "-omain"
        );
        assert_eq!(
            FlagMapper::map_output(&config, &"output".into(), ".o"),
            "-ooutput"
        );
    }

    #[test]
    fn test_map_compile_only_msvc() {
        let config = create_test_msvc_config();
        assert_eq!(FlagMapper::map_compile_only(&config), "/c");
    }

    #[test]
    fn test_map_compile_only_gcc() {
        let config = create_test_gcc_config();
        assert_eq!(FlagMapper::map_compile_only(&config), "-c");
    }

    #[test]
    fn test_build_includes() {
        let config = create_test_gcc_config();
        let includes = vec!["include1".into(), "include2".into()];
        let result = FlagMapper::build_includes(&config, &includes);
        assert_eq!(result, vec!["-Iinclude1", "-Iinclude2"]);
    }

    #[test]
    fn test_build_defines() {
        let config = create_test_gcc_config();
        let defines = vec!["DEBUG".into(), "VERSION=1".into()];
        let result = FlagMapper::build_defines(&config, &defines);
        assert_eq!(result, vec!["-DDEBUG", "-DVERSION=1"]);
    }

    #[test]
    fn test_format_includes() {
        let config = create_test_gcc_config();
        let includes = vec!["include1".into(), "include2".into()];
        let result = FlagMapper::format_includes(&config, &includes);
        assert_eq!(result, "-Iinclude1 -Iinclude2");
    }

    #[test]
    fn test_format_defines() {
        let config = create_test_gcc_config();
        let defines = vec!["DEBUG".into(), "VERSION=1".into()];
        let result = FlagMapper::format_defines(&config, &defines);
        assert_eq!(result, "-DDEBUG -DVERSION=1");
    }

    #[test]
    fn test_format_libraries() {
        let config = create_test_gcc_config();
        let libs = vec!["mylib1".into(), "mylib2".into()];
        let result = FlagMapper::format_libraries(&config, &libs);
        assert_eq!(result, "-lmylib1 -lmylib2");
    }
}
