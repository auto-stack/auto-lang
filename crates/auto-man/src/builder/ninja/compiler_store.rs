use auto_lang::config::AutoConfig;
use auto_val::{AutoPath, AutoStr};
use log::*;
use std::collections::HashMap;
use std::path::Path;

use crate::AutoResult;
use super::config::{CompilerConfig, CompilerKind, CompilerLocation, ExecutableType, FlagFormat, FlagMappings};

/// 编译器配置存储
///
/// 从用户配置文件 (~/.auto/auto-man/am.at) 加载编译器配置
pub struct CompilerStore {
    configs: HashMap<AutoStr, CompilerConfig>,
}

impl CompilerStore {
    /// 创建空的编译器存储
    pub fn new() -> Self {
        Self {
            configs: HashMap::new(),
        }
    }

    /// 从用户配置文件加载编译器配置
    ///
    /// # 参数
    /// - `am_at_path`: am.at 配置文件路径
    ///
    /// # 示例
    /// ```ignore
    /// let store = CompilerStore::load_from_am_config("~/.auto/auto-man/am.at")?;
    /// ```
    pub fn load_from_am_config(am_at_path: &Path) -> AutoResult<Self> {
        let mut store = Self::new();

        // 检查配置文件是否存在
        if !am_at_path.exists() {
            info!(
                "Compiler config file not found: {}, using defaults",
                am_at_path.display()
            );
            return Ok(store);
        }

        // 解析配置文件
        let config = AutoConfig::read(am_at_path)?;

        // 查找 compilers 节点
        use auto_val::{Kid, ValueKey};
        if let Some(Kid::Node(compilers_node)) = config.root.get_kid(&ValueKey::Str("compilers".into())) {
            for (_, kid) in compilers_node.kids_iter() {
                if let Kid::Node(node) = kid {
                    if node.name == "compiler" {
                        if let Ok(compiler_config) = Self::parse_compiler_node(&node) {
                            let name = compiler_config.name.clone();
                            store.configs.insert(name, compiler_config);
                        } else {
                            warn!("Failed to parse compiler node: {}", node.main_arg().to_astr());
                        }
                    }
                }
            }
            info!("Loaded {} compiler configs from am.at", store.configs.len());
        } else {
            info!("No compilers node found in am.at");
        }

        Ok(store)
    }

    /// 解析单个 compiler 节点
    fn parse_compiler_node(node: &auto_val::Node) -> AutoResult<CompilerConfig> {
        // 获取编译器名称
        let name = node.main_arg().to_astr();

        // 获取编译器类型
        let kind_str = node.get_prop("kind").to_astr();
        let kind = CompilerKind::from_str(kind_str.as_str())
            .ok_or(format!("Unknown compiler kind: {}", kind_str))?;

        // 解析 location
        let location = Self::parse_location(node)?;

        // 解析 executables
        let executables = Self::parse_executables(node)?;

        // 解析 flags
        let flags = Self::parse_flags(node)?;

        // 解析 default_cflags
        let default_cflags = Self::parse_default_cflags(node);

        Ok(CompilerConfig {
            kind,
            name: name.clone(),
            executables,
            flags,
            location,
            default_cflags,
        })
    }

    /// 解析 location 配置
    fn parse_location(node: &auto_val::Node) -> AutoResult<CompilerLocation> {
        use auto_val::{Kid, ValueKey};

        if let Some(Kid::Node(location_node)) = node.get_kid(&ValueKey::Str("location".into())) {
            let location_type = location_node.main_arg().to_astr();

            match location_type.as_str() {
                "Env" => Ok(CompilerLocation::Env),
                "Dir" => {
                    let dir_str = location_node.get_prop("path").to_astr();
                    Ok(CompilerLocation::Dir(AutoPath::new(dir_str.as_str())))
                }
                "Executable" => {
                    // Executable(location, path) 格式
                    let exec_type_str = location_node.get_prop("type").to_astr();
                    let exec_type = ExecutableType::from_str(exec_type_str.as_str())
                        .ok_or(format!("Unknown executable type: {}", exec_type_str))?;
                    let path = AutoPath::new(location_node.get_prop("path").to_astr().as_str());
                    Ok(CompilerLocation::Executable(exec_type, path))
                }
                _ => {
                    warn!("Unknown location type: {}, using Env", location_type);
                    Ok(CompilerLocation::Env)
                }
            }
        } else {
            // 默认使用 Env
            Ok(CompilerLocation::Env)
        }
    }

    /// 解析 executables 配置
    fn parse_executables(node: &auto_val::Node) -> AutoResult<HashMap<ExecutableType, AutoPath>> {
        use auto_val::{Kid, ValueKey};

        let mut executables = HashMap::new();

        if let Some(Kid::Node(execs_node)) = node.get_kid(&ValueKey::Str("executables".into())) {
            // 解析 cc
            if execs_node.has_prop("cc") {
                let cc = AutoPath::new(execs_node.get_prop("cc").to_astr().as_str());
                executables.insert(ExecutableType::Compiler, cc);
            }

            // 解析 ar
            if execs_node.has_prop("ar") {
                let ar = AutoPath::new(execs_node.get_prop("ar").to_astr().as_str());
                executables.insert(ExecutableType::Archiver, ar);
            }

            // 解析 link
            if execs_node.has_prop("link") {
                let link = AutoPath::new(execs_node.get_prop("link").to_astr().as_str());
                executables.insert(ExecutableType::Linker, link);
            }

            // 解析 as
            if execs_node.has_prop("as") {
                let as_cmd = AutoPath::new(execs_node.get_prop("as").to_astr().as_str());
                executables.insert(ExecutableType::Assembler, as_cmd);
            }
        }

        Ok(executables)
    }

    /// 解析 flags 配置
    fn parse_flags(node: &auto_val::Node) -> AutoResult<FlagMappings> {
        use auto_val::ValueKey;

        let flags_kid = node.get_kid(&ValueKey::Str("flags".into()));

        let parse_flag_format = |flag_name: &str| -> FlagFormat {
            if let Some(auto_val::Kid::Node(flags_node)) = &flags_kid {
                if flags_node.has_prop(flag_name) {
                    let flag_value = flags_node.get_prop(flag_name).to_astr();
                    // 尝试解析为函数调用格式,如 Prefix("-I")
                    if flag_value.contains('(') {
                        // 简单的函数调用解析
                        let left_paren = flag_value.find('(').unwrap_or(0);
                        let right_paren = flag_value.find(')').unwrap_or(flag_value.len());
                        let format_type = &flag_value[0..left_paren];
                        let format_arg = &flag_value[left_paren + 1..right_paren];

                        match format_type {
                            "Prefix" => FlagFormat::Prefix(format_arg.to_string()),
                            "Postfix" => FlagFormat::Postfix(format_arg.to_string()),
                            "Both" => {
                                // Both 格式: Both("/Fo", ".obj")
                                let args: Vec<&str> = format_arg.split(", ").collect();
                                if args.len() == 2 {
                                    FlagFormat::Both(args[0].to_string(), args[1].to_string())
                                } else {
                                    // 默认使用 Prefix
                                    FlagFormat::Prefix(args[0].to_string())
                                }
                            }
                            _ => FlagFormat::Prefix(format_arg.to_string()),
                        }
                    } else {
                        // 简单字符串,当作 Prefix
                        FlagFormat::Prefix(flag_value.as_str().to_string())
                    }
                } else {
                    // 默认值将在后面由编译器类型决定
                    FlagFormat::Prefix("-{}".to_string())
                }
            } else {
                // 默认值
                FlagFormat::Prefix("-{}".to_string())
            }
        };

        Ok(FlagMappings {
            include: parse_flag_format("include"),
            define: parse_flag_format("define"),
            library: parse_flag_format("library"),
            library_path: parse_flag_format("library_path"),
            output: parse_flag_format("output"),
            compile_only: parse_flag_format("compile_only"),
        })
    }

    /// 解析 default_cflags 配置
    fn parse_default_cflags(node: &auto_val::Node) -> Vec<AutoStr> {
        use auto_val::{Kid, ValueKey};

        if let Some(Kid::Node(cflags_node)) = node.get_kid(&ValueKey::Str("default_cflags".into())) {
            // 尝试获取值并转换为字符串数组
            // 如果节点有value属性
            if cflags_node.has_prop("value") {
                return cflags_node.get_prop("value").to_str_vec();
            }
        }

        Vec::new()
    }

    /// 获取编译器配置
    pub fn get(&self, name: &AutoStr) -> Option<&CompilerConfig> {
        self.configs.get(name)
    }

    /// 获取所有编译器配置名称
    pub fn list_names(&self) -> Vec<AutoStr> {
        self.configs.keys().cloned().collect()
    }

    /// 检查编译器配置是否存在
    pub fn has(&self, name: &AutoStr) -> bool {
        self.configs.contains_key(name)
    }
}

impl Default for CompilerStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_store_new() {
        let store = CompilerStore::new();
        assert_eq!(store.configs.len(), 0);
        assert!(!store.has(&"gcc".into()));
    }

    #[test]
    fn test_list_names_empty() {
        let store = CompilerStore::new();
        assert!(store.list_names().is_empty());
    }
}
